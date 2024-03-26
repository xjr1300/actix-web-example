use std::net::TcpListener;
use std::path::Path;

use anyhow::anyhow;

use server::settings::{retrieve_app_settings, AppEnvironment, SETTINGS_DIR_NAME};
use server::startup::build_http_server;
use server::telemetry::{generate_log_subscriber, init_log_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 環境変数を設定
    dotenvx::dotenv().ok();

    // 環境変数からアプリケーションの動作環境を取得
    let app_env: AppEnvironment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| String::from("development"))
        .into();
    // アプリケーション設定を取得
    let settings_dir = Path::new(SETTINGS_DIR_NAME);
    let app_settings = retrieve_app_settings(app_env, settings_dir)?;

    // サブスクライバを初期化
    let subscriber =
        generate_log_subscriber("actix_web_example".into(), app_settings.logging.level);
    init_log_subscriber(subscriber);

    // Httpサーバーがリッスンするポートをバインド
    let address = format!("localhost:{}", app_settings.http_server.port);
    let listener = TcpListener::bind(address).map_err(|e| anyhow!(e))?;

    // HTTPサーバーを起動
    build_http_server(listener)?.await.map_err(|e| e.into())
}
