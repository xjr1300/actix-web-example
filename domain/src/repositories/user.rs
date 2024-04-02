use async_trait::async_trait;

use crate::models::user::User;
use crate::DomainResult;

/// ユーザー・リポジトリ
#[async_trait]
pub trait UserRepository: Sync + Send {
    /// ユーザーを登録する。
    ///
    /// # 引数
    ///
    /// * `user` - 登録するユーザー
    ///
    /// # 戻り値
    ///
    /// * 登録したユーザー
    async fn create(&self, user: User) -> DomainResult<User>;
}
