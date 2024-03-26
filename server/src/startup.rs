use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing_actix_web::TracingLogger;

/// HTTPサーバーを構築する。
///
/// # 引数
///
/// * `listener` - HTTPサーバーがリッスンするポートをバインドしたリスナー
///
/// # 戻り値
///
/// HTTPサーバー
pub fn build_http_server(listener: TcpListener) -> anyhow::Result<Server> {
    // HttpServerを構築
    Ok(HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run())
}

/// ヘルス・チェック
#[tracing::instrument(name = "health check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("It works!")
}
