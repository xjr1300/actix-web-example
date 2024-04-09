use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::ErrorHandlers;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;

use infra::routes::accounts::accounts_scope;
use infra::routes::{default_error_handler, health_check};
use infra::RequestContext;

/// HTTPサーバーを構築する。
///
/// # 引数
///
/// * `listener` - HTTPサーバーがリッスンするポートをバインドしたリスナー
/// * `context` - リクエストコンテキスト
///
/// # 戻り値
///
/// HTTPサーバー
pub fn build_http_server(listener: TcpListener, context: RequestContext) -> anyhow::Result<Server> {
    // HttpServerを構築
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(ErrorHandlers::new().default_handler(default_error_handler))
            .route("/health-check", web::get().to(health_check))
            .service(accounts_scope())
            .app_data(web::Data::new(context.clone()))
    })
    .listen(listener)?
    .run())
}
