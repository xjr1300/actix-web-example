pub mod repositories;
pub mod routes;

use sqlx::PgPool;

use domain::repositories::user::UserRepository;
use repositories::postgres::user::PgUserRepository;
use use_cases::settings::{AuthorizationSettings, PasswordSettings};

/// リクエストコンテキスト
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// パスワード設定
    pub password_settings: PasswordSettings,
    /// 認証設定
    pub authorization_settings: AuthorizationSettings,
    /// データベース接続プール
    pool: PgPool,
}

impl RequestContext {
    /// リクエストコンテキストを構築する。
    ///
    /// # 引数
    ///
    /// * `pool` - データベース接続プール
    ///
    /// # 戻り値
    ///
    /// リクエストコンテキスト
    pub fn new(
        password_settings: PasswordSettings,
        authorization_settings: AuthorizationSettings,
        pool: PgPool,
    ) -> Self {
        Self {
            password_settings,
            authorization_settings,
            pool,
        }
    }

    /// ユーザーリポジトリを返す。
    ///
    /// # 戻り値
    ///
    /// ユーザーリポジトリ
    pub fn user_repository(&self) -> impl UserRepository {
        PgUserRepository::new(self.pool.clone())
    }
}
