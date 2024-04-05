pub mod repositories;
pub mod routes;

use sqlx::PgPool;

use configurations::settings::PasswordSettings;
use domain::repositories::user::UserRepository;
use repositories::postgres::user::PgUserRepository;

/// リクエスト・コンテキスト
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// パスワードに振りかけるペッパー
    pub password_settings: PasswordSettings,
    /// データベース接続プール
    pool: PgPool,
}

impl RequestContext {
    /// リクエスト・コンテキストを構築する。
    ///
    /// # 引数
    ///
    /// * `pool` - データベース接続プール
    ///
    /// # 戻り値
    ///
    /// リクエスト・コンテキスト
    pub fn new(password_settings: PasswordSettings, pool: PgPool) -> Self {
        Self {
            password_settings,
            pool,
        }
    }

    /// ユーザー・リポジトリを返す。
    ///
    /// # 戻り値
    ///
    /// ユーザー・リポジトリ
    pub fn user_repository(&self) -> impl UserRepository {
        PgUserRepository::new(self.pool.clone())
    }
}
