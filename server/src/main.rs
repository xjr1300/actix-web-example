use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use server::telemetry::{generate_log_subscriber, init_log_subscriber};
use tracing_actix_web::TracingLogger;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // サブスクライバを初期化
    let subscriber = generate_log_subscriber("actix_web_example".into(), "info");
    init_log_subscriber(subscriber);

    // HttpServerを起動
    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/// ヘルス・チェック
#[tracing::instrument(name = "health check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("It works!")
}
