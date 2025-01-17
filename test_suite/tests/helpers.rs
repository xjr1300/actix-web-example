use std::net::TcpListener;
use std::path::Path;

use anyhow::Context as _;
use deadpool_redis::Pool as RedisPool;
use infra::repositories::redis::token::RedisTokenRepository;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Connection as _, Executor as _, PgConnection, PgPool};
use use_cases::accounts::SignInUseCaseInput;
use uuid::Uuid;

use configurations::settings::{
    read_app_settings, AppEnvironment, AppSettings, DatabaseSettings, ENV_APP_ENVIRONMENT,
    SETTINGS_DIR_NAME,
};
use domain::models::primitives::*;
use domain::models::user::{UserId, UserPermission, UserPermissionCode, UserPermissionName};
use domain::repositories::token::TokenContent;
use domain::repositories::token::TokenRepository;
use domain::repositories::user::{SignUpInput, SignUpInputBuilder, SignUpOutput, UserRepository};
use infra::repositories::postgres::user::{insert_user_query, InsertedUserRow, PgUserRepository};
use infra::routes::accounts::SignUpReqBody;
use infra::RequestContext;
use server::startup::build_http_server;
use server::telemetry::{generate_log_subscriber, init_log_subscriber};
use use_cases::passwords::generate_phc_string;
use use_cases::settings::PasswordSettings;

/// 分解したレスポンス
pub struct ResponseParts {
    /// ステータスコード
    pub status_code: reqwest::StatusCode,
    /// ヘッダ
    pub headers: reqwest::header::HeaderMap,
    /// ボディ
    pub body: String,
}

/// レスポンスをステータスコード、ヘッダ、ボディに分割する。
pub async fn split_response(response: reqwest::Response) -> anyhow::Result<ResponseParts> {
    Ok(ResponseParts {
        status_code: response.status(),
        headers: response.headers().clone(),
        body: response.text().await?,
    })
}

/// ログサブスクライバ
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_level = log::Level::Info;
    let subscriber_name = String::from("test");

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = generate_log_subscriber(subscriber_name, default_level, std::io::stdout);
        init_log_subscriber(subscriber);
    } else {
        let subscriber = generate_log_subscriber(subscriber_name, default_level, std::io::sink);
        init_log_subscriber(subscriber);
    }
});

pub const CONTENT_TYPE_APPLICATION_JSON: &str = "application/json";

/// 統合テスト用アプリ
pub struct TestApp {
    /// アプリのルートURI
    pub root_uri: String,
    /// アプリの設定
    pub settings: AppSettings,
    /// PostgreSQL接続プール
    pub pg_pool: PgPool,
    /// Redis接続プール
    pub redis_pool: RedisPool,
}

impl TestApp {
    pub async fn sign_up(&self, body: String) -> anyhow::Result<reqwest::Response> {
        let client = reqwest::Client::new();
        client
            .post(&format!("{}/accounts/sign-up", self.root_uri))
            .body(body)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .send()
            .await
            .map_err(|e| e.into())
    }

    pub async fn sign_in(
        &self,
        email: String,
        password: SecretString,
    ) -> anyhow::Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let body = format!(
            r#"{{"email": "{}", "password": "{}" }}"#,
            email,
            password.expose_secret()
        );
        client
            .post(&format!("{}/accounts/sign-in", self.root_uri))
            .body(body)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .send()
            .await
            .map_err(|e| e.into())
    }

    /// ユーザーの一覧をリクエストする。
    ///
    /// # 引数
    ///
    /// * `token` - アクセストークン
    /// * `auth_header` - アクセストークンを`Authorization`ヘッダに追加して送信するかを示すフラグ
    ///
    /// アクセストークンを送信しない場合は、`token`と`auth_header`に`None`を指定する。
    ///
    /// アクセストークンを`Authorization`ヘッダに追加して送信する場合は`true`を指定して、
    /// クッキーに追加して送信する場合は`false`を指定する。
    pub async fn list_users(
        &self,
        token: Option<SecretString>,
        auth_header: Option<bool>,
    ) -> anyhow::Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let mut builder = client.get(format!("{}/accounts/users", self.root_uri));
        if token.is_some() {
            builder = append_access_token(builder, token.unwrap(), auth_header.unwrap());
        }
        builder.send().await.map_err(|e| e.into())
    }

    pub async fn register_user(&self, input: SignUpInput) -> anyhow::Result<SignUpOutput> {
        let repo = PgUserRepository::new(self.pg_pool.clone());

        repo.create(input).await.map_err(|e| e.into())
    }

    /// トークンを元にRedisに登録されている値を取得する。
    pub async fn retrieve_token_content(&self, token: &SecretString) -> Option<TokenContent> {
        let repo = RedisTokenRepository::new(self.redis_pool.clone());
        repo.retrieve_token_content(token).await.unwrap()
    }
}

pub fn app_settings() -> anyhow::Result<AppSettings> {
    // 環境変数からアプリケーションの動作環境を取得
    let app_env: AppEnvironment = std::env::var(ENV_APP_ENVIRONMENT)
        .unwrap_or_else(|_| String::from("development"))
        .into();
    // 環境変数や設定ファイルからアプリケーション設定を読み込み
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let settings_dir = dir.join("..").join(SETTINGS_DIR_NAME);
    read_app_settings(app_env, settings_dir)
}

/// 統合テスト用のHTTPサーバーを起動する。
///
/// # 引数
///
/// * `settings` - アプリケーション設定
///
/// # 戻り値
///
/// 統合テスト用アプリ
pub async fn spawn_test_app(mut settings: AppSettings) -> anyhow::Result<TestApp> {
    dotenvx::dotenv()?;
    Lazy::force(&TRACING);

    // テスト用のデータベースの名前を設定
    settings.database.name = format!("awe_test_{}", Uuid::new_v4()).replace('-', "_");
    // テスト用のデータベースを作成して、接続及び構成
    let pg_pool = configure_database(&settings.database).await?;
    // Redis接続プールを構築
    let redis_pool = settings.redis.connection_pool()?;
    // テスト用のデータベースに接続するリポジトリのコンテナを構築
    let context = RequestContext::new(
        settings.http_server.clone(),
        settings.password.clone(),
        settings.authorization.clone(),
        pg_pool.clone(),
        redis_pool.clone(),
    );

    // ポート0を指定してTCPソケットにバインドすることで、OSにポート番号の決定を委譲
    let listener = TcpListener::bind("localhost:0").context("failed to bind random port")?;
    let port = listener.local_addr().unwrap().port();
    let server = build_http_server(listener, context)?;
    // 統合テストが終了すると、HTTPサーバーがリッスンするポートが閉じられる。
    // すると、actix-webが提供する`Server`が終了して、ここで生み出したスレッドが終了する。
    tokio::spawn(server);

    Ok(TestApp {
        root_uri: format!("http://localhost:{}", port),
        settings,
        pg_pool,
        redis_pool,
    })
}

/// データベースを作成して、接続及び構成する。
///
/// # 引数
///
/// * `settings` - データベース設定
///
/// # 戻り値
///
/// PostgreSQL接続プール
pub async fn configure_database(settings: &DatabaseSettings) -> anyhow::Result<PgPool> {
    // データベースを構築
    let mut connection = PgConnection::connect_with(&settings.without_db())
        .await
        .expect("Fail to connect to postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, settings.name).as_str())
        .await
        .expect("Failed to create test database.");

    // データベースに接続
    let pool = PgPool::connect_with(settings.with_db()).await?;
    // データベースをマイグレート
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let migrations_dir = crate_dir.join("..").join("migrations");
    if migrations_dir.is_dir() {
        sqlx::migrate!("../migrations").run(&pool).await?;
    }

    Ok(pool)
}

/// cspell: disable-next-line
pub const RAW_PHC_PASSWORD: &str = "$argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno";

/// ユーザーのEメールアドレス
pub const ADMIN_USER_EMAIL_ADDRESS: &str = "admin@example.com";
pub const GENERAL_USER_EMAIL_ADDRESS: &str = "general@example.com";

/// 未加工なパスワードとして使用できる文字列
pub const ADMIN_USER_RAW_PASSWORD: &str = "Az3#Za3@";
pub const GENERAL_USER_RAW_PASSWORD: &str = "Yd3*_#Za";

#[allow(dead_code)]
pub fn generate_phc_password() -> PhcPassword {
    PhcPassword::new(SecretString::new(String::from(RAW_PHC_PASSWORD))).unwrap()
}

#[allow(dead_code)]
pub fn generate_user_permission() -> UserPermission {
    UserPermission::new(
        UserPermissionCode::Admin,
        UserPermissionName::new("管理者").unwrap(),
    )
}

#[allow(dead_code)]
pub fn generate_family_name() -> FamilyName {
    FamilyName::new("山田").unwrap()
}

#[allow(dead_code)]
pub fn generate_given_name() -> GivenName {
    GivenName::new("太郎").unwrap()
}

#[allow(dead_code)]
pub fn generate_postal_code() -> PostalCode {
    PostalCode::new("105-0011").unwrap()
}

#[allow(dead_code)]
pub fn generate_address() -> Address {
    Address::new("東京都港区芝公園4-2-8").unwrap()
}

#[allow(dead_code)]
pub fn generate_optional_fixed_phone_number() -> OptionalFixedPhoneNumber {
    OptionalFixedPhoneNumber::try_from("03-3433-5111").unwrap()
}

#[allow(dead_code)]
pub fn generate_optional_mobile_phone_number() -> OptionalMobilePhoneNumber {
    OptionalMobilePhoneNumber::try_from("090-1234-5678").unwrap()
}

#[allow(dead_code)]
pub fn generate_optional_remarks() -> OptionalRemarks {
    OptionalRemarks::try_from("すもももももももものうち。もももすももももものうち。").unwrap()
}

pub fn admin_user_sign_up_body_json() -> String {
    format!(
        r#"
        {{
            "email": "{}",
            "password": "{}",
            "userPermissionCode": 1,
            "familyName": "山田",
            "givenName": "太郎",
            "postalCode": "899-7103",
            "address": "鹿児島県志布志市志布志町志布志2-1-1",
            "fixedPhoneNumber": "099-472-1111",
            "mobilePhoneNumber": "090-1234-5678",
            "remarks": "日本に実際に存在するややこしい地名です。"
        }}
        "#,
        ADMIN_USER_EMAIL_ADDRESS, ADMIN_USER_RAW_PASSWORD
    )
}

pub fn admin_user_sign_up_body(body: &str) -> SignUpReqBody {
    serde_json::from_str::<SignUpReqBody>(body).unwrap()
}

pub fn general_user_sign_up_body() -> SignUpReqBody {
    SignUpReqBody {
        email: String::from(GENERAL_USER_EMAIL_ADDRESS),
        password: SecretString::new(String::from(GENERAL_USER_RAW_PASSWORD)),
        user_permission_code: 2,
        family_name: String::from("東京"),
        given_name: String::from("タワー"),
        postal_code: String::from("105-0011"),
        address: String::from("東京都港区芝公園4-2-8"),
        fixed_phone_number: Some(String::from("03-3433-5111")),
        mobile_phone_number: None,
        remarks: Some(String::from("1958年12月23日に開業しました。")),
    }
}

pub fn sign_up_input(body: SignUpReqBody, settings: &PasswordSettings) -> SignUpInput {
    let email = EmailAddress::new(body.email).unwrap();
    let user_permission_code = UserPermissionCode::try_from(body.user_permission_code).unwrap();
    let password = RawPassword::new(body.password).unwrap();
    let password = generate_phc_string(&password, settings).unwrap();
    let family_name = FamilyName::new(body.family_name).unwrap();
    let given_name = GivenName::new(body.given_name).unwrap();
    let postal_code = PostalCode::new(body.postal_code).unwrap();
    let address = Address::new(body.address).unwrap();
    let fixed_phone_number = OptionalFixedPhoneNumber::try_from(body.fixed_phone_number).unwrap();
    let mobile_phone_number =
        OptionalMobilePhoneNumber::try_from(body.mobile_phone_number).unwrap();
    let remarks = OptionalRemarks::try_from(body.remarks).unwrap();

    SignUpInputBuilder::new()
        .id(UserId::default())
        .email(email)
        .password(password)
        .active(true)
        .user_permission_code(user_permission_code)
        .family_name(family_name)
        .given_name(given_name)
        .postal_code(postal_code)
        .address(address)
        .fixed_phone_number(fixed_phone_number)
        .mobile_phone_number(mobile_phone_number)
        .remarks(remarks)
        .build()
        .unwrap()
}

pub fn admin_user_sign_in_use_case_input() -> SignInUseCaseInput {
    SignInUseCaseInput {
        email: EmailAddress::new(String::from(ADMIN_USER_EMAIL_ADDRESS)).unwrap(),
        password: RawPassword::new(SecretString::new(String::from(ADMIN_USER_RAW_PASSWORD)))
            .unwrap(),
    }
}

pub fn general_user_sign_in_use_case_input() -> SignInUseCaseInput {
    SignInUseCaseInput {
        email: EmailAddress::new(String::from(GENERAL_USER_EMAIL_ADDRESS)).unwrap(),
        password: RawPassword::new(SecretString::new(String::from(GENERAL_USER_RAW_PASSWORD)))
            .unwrap(),
    }
}

fn append_access_token(
    mut builder: reqwest::RequestBuilder,
    token: SecretString,
    auth_header: bool,
) -> reqwest::RequestBuilder {
    if auth_header {
        builder = builder.header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token.expose_secret()),
        );
    } else {
        builder = builder.header(
            reqwest::header::COOKIE,
            format!("access={}", token.expose_secret()),
        );
    }

    builder
}

pub async fn register_admin_and_general_user(
    password_settings: &PasswordSettings,
    repo: &PgUserRepository,
) -> anyhow::Result<(InsertedUserRow, InsertedUserRow)> {
    let admin = register_admin_user(password_settings, repo).await?;
    let general = register_general_user(password_settings, repo).await?;

    Ok((admin, general))
}

pub async fn register_admin_user(
    password_settings: &PasswordSettings,
    repo: &PgUserRepository,
) -> anyhow::Result<InsertedUserRow> {
    let json = admin_user_sign_up_body_json();
    let body = admin_user_sign_up_body(&json);
    let input = sign_up_input(body, password_settings);

    let mut tx = repo.begin().await?;
    let user = insert_user_query(input).fetch_one(&mut *tx).await?;
    tx.commit().await?;

    Ok(user)
}

pub async fn register_general_user(
    password_settings: &PasswordSettings,
    repo: &PgUserRepository,
) -> anyhow::Result<InsertedUserRow> {
    let body = general_user_sign_up_body();
    let input = sign_up_input(body, password_settings);

    let mut tx = repo.begin().await?;
    let user = insert_user_query(input).fetch_one(&mut *tx).await?;
    tx.commit().await?;

    Ok(user)
}
