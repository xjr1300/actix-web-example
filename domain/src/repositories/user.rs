use async_trait::async_trait;

use crate::common::DomainResult;
use crate::models::user::User;

/// ユーザー・リポジトリ
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    /// ユーザーを登録する。
    ///
    /// # 引数
    ///
    /// * `user` - 登録するユーザー
    ///
    /// # 戻り値
    ///
    /// * 登録したユーザー
    async fn create(user: User) -> DomainResult<User>;
}
