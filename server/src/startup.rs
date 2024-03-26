use std::path::Path;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing_actix_web::TracingLogger;

use crate::settings::{retrieve_app_settings, AppEnvironment};
use crate::telemetry::{generate_log_subscriber, init_log_subscriber};

pub async fn run<P: AsRef<Path>>(settings_dir: P) -> anyhow::Result<Server> {
    // 環境変数を設定
    dotenvx::dotenv().ok();

    // 環境変数からアプリケーションの動作環境を取得
    let app_env: AppEnvironment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| String::from("development"))
        .into();
    // アプリケーション設定を取得
    let settings_dir = settings_dir.as_ref();
    let app_settings = retrieve_app_settings(app_env, settings_dir)?;

    // サブスクライバを初期化
    let subscriber =
        generate_log_subscriber("actix_web_example".into(), app_settings.logging.level);
    init_log_subscriber(subscriber);

    // HttpServerを構築
    Ok(HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(health_check))
    })
    .bind(("127.0.0.1", app_settings.http_server.port))?
    .run())
}

/// ヘルス・チェック
#[tracing::instrument(name = "health check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("It works!")
}
