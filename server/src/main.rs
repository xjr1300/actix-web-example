use std::net::TcpListener;
use std::path::Path;

use anyhow::anyhow;

use server::settings::{
    retrieve_app_settings, AppEnvironment, ENV_APP_ENVIRONMENT, ENV_APP_ENVIRONMENT_DEFAULT,
    SETTINGS_DIR_NAME,
};
use server::startup::build_http_server;
use server::telemetry::{generate_log_subscriber, init_log_subscriber, LOG_SUBSCRIBER_NAME};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 環境変数を設定
    dotenvx::dotenv()?;

    // 環境変数からアプリケーションの動作環境を取得
    let app_env: AppEnvironment = std::env::var(ENV_APP_ENVIRONMENT)
        .unwrap_or_else(|_| String::from(ENV_APP_ENVIRONMENT_DEFAULT))
        .into();

    // アプリケーション設定を取得
    let settings_dir = Path::new(SETTINGS_DIR_NAME);
    let app_settings = retrieve_app_settings(app_env, settings_dir)?;

    // サブスクライバを初期化
    let subscriber = generate_log_subscriber(
        LOG_SUBSCRIBER_NAME.into(),
        app_settings.logging.level,
        std::io::stdout,
    );
    init_log_subscriber(subscriber);

    // データベース接続プールを取得
    let pool = app_settings.database.connection_pool();

    // Httpサーバーがリッスンするポートをバインド
    let address = format!("localhost:{}", app_settings.http_server.port);
    let listener = TcpListener::bind(&address).map_err(|e| anyhow!(e))?;
    tracing::info!("Http server is listening on `{}`", &address);

    // HTTPサーバーを起動
    build_http_server(listener, pool)?
        .await
        .map_err(|e| e.into())
}
