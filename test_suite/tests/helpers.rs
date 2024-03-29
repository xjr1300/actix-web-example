use std::net::TcpListener;
use std::path::Path;

use anyhow::Context as _;
use once_cell::sync::Lazy;
use sqlx::{Connection as _, Executor as _, PgConnection, PgPool};
use uuid::Uuid;

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
}

/// 統合テスト用のHTTPサーバーを起動する。
///
/// # 戻り値
///
/// 統合テスト用のHTTPサーバーのルートURI
pub async fn spawn_app_for_integration_test() -> anyhow::Result<TestApp> {
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
    let server = build_http_server(listener, pool)?;
    // 統合テストが終了すると、HTTPサーバーがリッスンするポートが閉じられる。
    // すると、actix-webが提供する`Server`が終了して、ここで生み出したスレッドが終了する。
    tokio::spawn(server);

    Ok(TestApp {
        root_uri: format!("http://localhost:{}", port),
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
