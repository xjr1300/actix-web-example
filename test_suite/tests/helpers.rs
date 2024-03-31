use std::net::TcpListener;
use std::path::Path;

use anyhow::Context as _;
use once_cell::sync::Lazy;
use secrecy::SecretString;
use sqlx::{Connection as _, Executor as _, PgConnection, PgPool};
use uuid::Uuid;

use domain::common::now_jst;
use domain::models::passwords::PhcPassword;
use domain::models::primitives::*;
use domain::models::user::{User, UserBuilder, UserId};
use server::settings::{
    retrieve_app_settings, AppEnvironment, DatabaseSettings, ENV_APP_ENVIRONMENT, SETTINGS_DIR_NAME,
};
use server::startup::build_http_server;
use server::telemetry::{generate_log_subscriber, init_log_subscriber};

/// ログ・サブスクライバ
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

/// 統合テスト用アプリ
pub struct TestApp {
    /// アプリのルートURI
    pub root_uri: String,
    /// データベース接続プール
    pub pool: PgPool,
}

/// 統合テスト用のHTTPサーバーを起動する。
///
/// # 戻り値
///
/// 統合テスト用アプリ
pub async fn spawn_test_app() -> anyhow::Result<TestApp> {
    dotenvx::dotenv()?;
    Lazy::force(&TRACING);

    // 環境変数からアプリケーションの動作環境を取得
    let app_env: AppEnvironment = std::env::var(ENV_APP_ENVIRONMENT)
        .unwrap_or_else(|_| String::from("development"))
        .into();

    // アプリケーション設定を取得
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let settings_dir = dir.join("..").join(SETTINGS_DIR_NAME);
    let mut app_settings = retrieve_app_settings(app_env, settings_dir)?;

    // テスト用のデータベースの名前を設定
    app_settings.database.name = format!("awe_test_{}", Uuid::new_v4()).replace('-', "_");
    // テスト用のデータベースを作成して、接続及び構成
    let pool = configure_database(&app_settings.database).await?;

    // ポート0を指定してTCPソケットにバインドすることで、OSにポート番号の決定を委譲
    let listener = TcpListener::bind("localhost:0").context("failed to bind random port")?;
    let port = listener.local_addr().unwrap().port();
    let server = build_http_server(listener, pool.clone())?;
    // 統合テストが終了すると、HTTPサーバーがリッスンするポートが閉じられる。
    // すると、actix-webが提供する`Server`が終了して、ここで生み出したスレッドが終了する。
    tokio::spawn(server);

    Ok(TestApp {
        root_uri: format!("http://localhost:{}", port),
        pool,
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
/// データベース接続プール
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

pub fn generate_phc_password() -> PhcPassword {
    PhcPassword::new(SecretString::new(String::from(RAW_PHC_PASSWORD))).unwrap()
}

pub fn generate_family_name() -> FamilyName {
    FamilyName::new("山田").unwrap()
}

pub fn generate_given_name() -> GivenName {
    GivenName::new("太郎").unwrap()
}

pub fn generate_postal_code() -> PostalCode {
    PostalCode::new("105-0011").unwrap()
}

pub fn generate_address() -> Address {
    Address::new("東京都港区芝公園4-2-8").unwrap()
}

pub fn generate_fixed_phone_number() -> FixedPhoneNumber {
    FixedPhoneNumber::try_from("03-3433-5111").unwrap()
}

pub fn generate_mobile_phone_number() -> MobilePhoneNumber {
    MobilePhoneNumber::try_from("090-1234-5678").unwrap()
}

pub fn generate_remarks() -> Remarks {
    Remarks::try_from("すもももももももものうち。もももすももももものうち。").unwrap()
}

pub fn generate_user(id: UserId, email: EmailAddress) -> User {
    let dt = now_jst();
    UserBuilder::new()
        .id(id)
        .email(email)
        .password(generate_phc_password())
        .active(true)
        .family_name(generate_family_name())
        .given_name(generate_given_name())
        .postal_code(generate_postal_code())
        .address(generate_address())
        .fixed_phone_number(generate_fixed_phone_number())
        .mobile_phone_number(generate_mobile_phone_number())
        .remarks(generate_remarks())
        .created_at(dt)
        .updated_at(dt)
        .build()
        .unwrap()
}
