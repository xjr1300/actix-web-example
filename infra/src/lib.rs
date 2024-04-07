pub mod repositories;
pub mod routes;

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use configurations::settings::HttpServerSettings;
use domain::repositories::user::UserRepository;
use repositories::postgres::user::PgUserRepository;
use use_cases::settings::{AuthorizationSettings, PasswordSettings};

/// リクエストコンテキスト
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// HTTPサーバー設定
    pub http_server_settings: HttpServerSettings,
    /// パスワード設定
    pub password_settings: PasswordSettings,
    /// 認証設定
    pub authorization_settings: AuthorizationSettings,
    /// PostgreSQL接続プール
    pg_pool: PgPool,
    /// Redis接続プール
    redis_pool: RedisPool,
}

impl RequestContext {
    /// リクエストコンテキストを構築する。
    ///
    /// # 引数
    ///
    /// * `http_server_settings` - HTTPサーバー設定
    /// * `password_settings` - パスワード設定
    /// * `authorization_settings` - 認証設定
    /// * `pg_pool` - PostgreSQL接続プール
    /// * `redis_pool` - Redis接続プール
    ///
    /// # 戻り値
    ///
    /// リクエストコンテキスト
    pub fn new(
        http_server_settings: HttpServerSettings,
        password_settings: PasswordSettings,
        authorization_settings: AuthorizationSettings,
        pg_pool: PgPool,
        redis_pool: RedisPool,
    ) -> Self {
        Self {
            http_server_settings,
            password_settings,
            authorization_settings,
            pg_pool,
            redis_pool,
        }
    }

    /// ユーザーリポジトリを返す。
    ///
    /// # 戻り値
    ///
    /// ユーザーリポジトリ
    pub fn user_repository(&self) -> impl UserRepository {
        PgUserRepository::new(self.pg_pool.clone())
    }
}
