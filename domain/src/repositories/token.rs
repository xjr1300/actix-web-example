use async_trait::async_trait;
use enum_display::EnumDisplay;
use secrecy::SecretString;

use crate::models::user::UserId;
use crate::{DomainError, DomainResult};

/// トークンリポジトリ
#[async_trait]
pub trait TokenRepository: Sync + Send {
    /// アクセストークンとリフレッシュトークンを登録する。
    ///
    /// # 引数
    ///
    /// * `tokens` - トークンペア
    async fn register_token_pair<'a>(
        &self,
        user_id: UserId,
        tokens: TokenPairWithTtl<'a>,
    ) -> DomainResult<()>;

    /// トークンからユーザーIDとトークンの種類を取得する。
    ///
    /// # 引数
    ///
    /// * `token` - トークン
    ///
    /// # 戻り値
    ///
    /// ユーザーIDとトークンの種類
    async fn retrieve_token_content(
        &self,
        token: &SecretString,
    ) -> DomainResult<Option<TokenContent>>;
}

/// アクセストークン及びリフレッシュトークンとそれぞれの生存
pub struct TokenPairWithTtl<'a> {
    /// アクセストークン
    pub access: &'a SecretString,
    /// アクセストークンの生存期間（秒）
    pub access_ttl: u64,
    /// リフレッシュトークン
    pub refresh: &'a SecretString,
    /// リフレッシュトークンの生存期間（秒）
    pub refresh_ttl: u64,
}

/// トークンが保有している値
pub struct TokenContent {
    /// ユーザーID
    pub user_id: UserId,
    /// トークンの種類
    pub token_type: TokenType,
}

/// トークンの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDisplay)]
#[enum_display(case = "Lower")]
pub enum TokenType {
    /// アクセストークン
    Access,
    /// リフレッシュトークン
    Refresh,
}

impl TryFrom<&str> for TokenType {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "access" => Ok(Self::Access),
            "refresh" => Ok(Self::Refresh),
            _ => Err(DomainError::Validation(
                format!("トークンの種類を示す文字列ではありません。({})", value).into(),
            )),
        }
    }
}
