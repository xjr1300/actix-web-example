use async_trait::async_trait;
use secrecy::SecretString;

use crate::models::user::UserId;
use crate::DomainResult;

/// トークンリポジトリ
#[async_trait]
pub trait TokenRepository: Sync + Send {
    /// アクセストークンとリフレッシュトークンを登録する。
    ///
    /// # 引数
    ///
    /// * `tokens` - トークンペア
    async fn register_token_pair(&self, tokens: TokenPair) -> DomainResult<()>;

    /// アクセストークンからユーザーのIDを取得する。
    ///
    /// # 引数
    ///
    /// * `access_token` - アクセストークン
    ///
    /// # 戻り値
    ///
    /// ユーザーID
    async fn retrieve_user_id_by_access_token(
        &self,
        token: SecretString,
    ) -> DomainResult<Option<UserId>>;

    /// リフレッシュトークンからユーザーのIDを取得する。
    ///
    /// # 引数
    ///
    /// * `access_token` - リフレッシュトークン
    ///
    /// # 戻り値
    ///
    /// ユーザーID
    async fn retrieve_user_id_by_refresh_token(
        &self,
        token: SecretString,
    ) -> DomainResult<Option<UserId>>;
}

/// JWTのトークンペア
pub struct TokenPair {
    /// アクセストークン
    pub access: SecretString,
    /// リフレッシュトークン
    pub refresh: SecretString,
}
