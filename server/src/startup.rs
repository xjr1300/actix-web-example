use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::ErrorHandlers;
use actix_web::{web, App, HttpServer};
use routes::accounts::accounts_scope;
use routes::common::{default_error_handler, health_check};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

/// HTTPサーバーを構築する。
///
/// # 引数
///
/// * `listener` - HTTPサーバーがリッスンするポートをバインドしたリスナー
/// * `pool` - データベース接続プール
///
/// # 戻り値
///
/// HTTPサーバー
pub fn build_http_server(listener: TcpListener, pool: PgPool) -> anyhow::Result<Server> {
    // HttpServerを構築
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(ErrorHandlers::new().default_handler(default_error_handler))
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .service(accounts_scope())
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run())
}
