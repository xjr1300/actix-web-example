use std::path::Path;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing_actix_web::TracingLogger;

use server::settings::{retrieve_app_settings, AppEnvironment};
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
    let app_settings = retrieve_app_settings(app_env, Path::new("settings"))?;

    // サブスクライバを初期化
    let subscriber =
        generate_log_subscriber("actix_web_example".into(), app_settings.logging.level);
    init_log_subscriber(subscriber);

    // HttpServerを起動
    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(health_check))
    })
    .bind(("127.0.0.1", app_settings.http_server.port))?
    .run()
    .await
    .map_err(|e| e.into())
}

/// ヘルス・チェック
#[tracing::instrument(name = "health check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("It works!")
}
