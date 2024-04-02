pub mod repositories;
pub mod routes;

use secrecy::SecretString;
use sqlx::PgPool;

use domain::repositories::user::UserRepository;
use repositories::postgres::user::PgUserRepository;

/// リクエスト・コンテキスト
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub pepper: SecretString,
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
    pub fn new(pepper: SecretString, pool: PgPool) -> Self {
        Self { pepper, pool }
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
