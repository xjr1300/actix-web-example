use anyhow::anyhow;
use async_trait::async_trait;
use deadpool_redis::{Connection as RedisConnection, Pool as RedisPool};
use redis::AsyncCommands;
use secrecy::{ExposeSecret as _, SecretString};
use sha2::{Digest, Sha256};

use domain::models::user::{UserId, UserPermissionCode};
use domain::repositories::token::{TokenContent, TokenPairWithTtl, TokenRepository, TokenType};
use domain::{DomainError, DomainResult};

/// Redisトークンリポジトリ
pub struct RedisTokenRepository {
    /// Redis接続プール
    pool: RedisPool,
}

impl RedisTokenRepository {
    /// Redisトークンリポジトリを構築する。
    ///
    /// # 引数
    ///
    /// * `pool` - Redis接続プール
    ///
    /// # 戻り値
    ///
    /// Redis接続プール
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Redisに接続する。
    ///
    /// # 戻り値
    ///
    /// Redis接続
    async fn connection(&self) -> DomainResult<RedisConnection> {
        self.pool.get().await.map_err(|e| {
            tracing::error!("{} {}({}:{})", CONNECTION_ERROR, e, file!(), line!());
            DomainError::Repository(anyhow!("{}", CONNECTION_ERROR))
        })
    }
}

#[async_trait]
impl TokenRepository for RedisTokenRepository {
    /// アクセストークンとリフレッシュトークンを登録する。
    ///
    /// # 引数
    ///
    /// * `tokens` - トークンペア
    async fn register_token_pair<'a>(
        &self,
        user_id: UserId,
        token_pair: TokenPairWithTtl<'a>,
        user_permission_code: UserPermissionCode,
    ) -> DomainResult<()> {
        let access_key = generate_key(token_pair.access);
        let access_value = generate_value(user_id, TokenType::Access, user_permission_code);
        let refresh_key = generate_key(token_pair.refresh);
        let refresh_value = generate_value(user_id, TokenType::Refresh, user_permission_code);
        let mut conn = self.connection().await?;
        store(&mut conn, &access_key, &access_value, token_pair.access_ttl).await?;
        store(
            &mut conn,
            &refresh_key,
            &refresh_value,
            token_pair.refresh_ttl,
        )
        .await?;

        Ok(())
    }

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
    ) -> DomainResult<Option<TokenContent>> {
        let mut conn = self.connection().await?;
        let key = generate_key(token);
        let value = retrieve(&mut conn, &key).await?;
        if value.is_none() {
            return Ok(None);
        }
        let (user_id, token_type, user_permission_code) = split_value(&value.unwrap())?;

        Ok(Some(TokenContent {
            user_id,
            token_type,
            user_permission_code,
        }))
    }
}

/// Redisに登録するキーを生成する。
///
/// # 引数
///
/// * `token` - トークン
///
/// # 戻り値
///
/// トークンをハッシュ化した文字列
fn generate_key(token: &SecretString) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.expose_secret().as_bytes());

    format!("{:x}", hasher.finalize())
}

/// Redisに登録する値を生成する。
fn generate_value(
    user_id: UserId,
    token_type: TokenType,
    user_permission_code: UserPermissionCode,
) -> String {
    format!("{}:{}:{}", user_id.value, token_type, user_permission_code)
}

/// Redisにキーと値を保存する。
///
/// # 引数
///
/// * `conn` - Redisコネクション
/// * `key` - キー
/// * `value` - 値
/// * `ttl` - 生存期間（秒）
async fn store(conn: &mut RedisConnection, key: &str, value: &str, ttl: u64) -> DomainResult<()> {
    conn.set_ex(key, value, ttl).await.map_err(|e| {
        tracing::error!("{} {}({}:{}", STORE_ERROR, e, file!(), line!());
        DomainError::Repository(anyhow!("{}", STORE_ERROR))
    })
}

/// Redisからキーで値を取得する。
async fn retrieve(conn: &mut RedisConnection, key: &str) -> DomainResult<Option<String>> {
    let value: Option<String> = conn.get(key).await.map_err(|e| {
        tracing::error!("{} {}({}:{})", RETRIEVE_ERROR, e, file!(), line!());
        DomainError::Repository(anyhow!("{}", RETRIEVE_ERROR))
    })?;

    Ok(value)
}

/// 値をユーザーID、トークンの種類及びユーザーの権限に分離する。
fn split_value(value: &str) -> DomainResult<(UserId, TokenType, UserPermissionCode)> {
    let mut values = value.split(':');
    let user_id = values.next().ok_or_else(|| {
        tracing::error!("{} ({}:{})", USER_ID_NOT_FOUND, file!(), line!());
        DomainError::Unexpected(anyhow!("{}", USER_ID_NOT_FOUND))
    })?;
    let user_id = UserId::try_from(user_id).map_err(|_| {
        tracing::error!("{} ({}:{})", USER_ID_CONSTRUCTION_FAILED, file!(), line!());
        DomainError::Unexpected(anyhow!("{}", USER_ID_CONSTRUCTION_FAILED))
    })?;
    let token_type = values.next().ok_or_else(|| {
        tracing::error!("{} ({}:{})", TOKEN_TYPE_NOT_FOUND, file!(), line!());
        DomainError::Unexpected(anyhow!("{}", TOKEN_TYPE_NOT_FOUND))
    })?;
    let token_type = TokenType::try_from(token_type).map_err(|_| {
        tracing::error!(
            "{} ({}:{})",
            TOKEN_TYPE_CONSTRUCTION_FAILED,
            file!(),
            line!()
        );
        DomainError::Unexpected(anyhow!("{}", TOKEN_TYPE_CONSTRUCTION_FAILED))
    })?;
    let user_permission = values.next().ok_or_else(|| {
        tracing::error!("{} ({}:{})", USER_PERMISSION_NOT_FOUND, file!(), line!());
        DomainError::Unexpected(anyhow!("{}", USER_PERMISSION_NOT_FOUND))
    })?;
    println!("user_permission: {}", user_permission);
    let user_permission_code = UserPermissionCode::try_from(user_permission).map_err(|_| {
        tracing::error!("{} ({}:{})", USER_PERMISSION_NOT_FOUND, file!(), line!());
        DomainError::Unexpected(anyhow!("{}", USER_PERMISSION_CONSTRUCTION_FAILED))
    })?;

    Ok((user_id, token_type, user_permission_code))
}

const CONNECTION_ERROR: &str = "Redisに接続するときにエラーが発生しました。";
const STORE_ERROR: &str = "Redisにキーと値を保存するときにエラーが発生しました。";
const RETRIEVE_ERROR: &str = "Redisからキーで値を取得するときにエラーが発生しました。";
const USER_ID_NOT_FOUND: &str = "Redisに登録された値からユーザーIDを取得できませんでした。";
const TOKEN_TYPE_NOT_FOUND: &str = "Redisに登録された値からトークンの種類を取得できませんでした。";
const USER_PERMISSION_NOT_FOUND: &str =
    "Redisに登録された値からユーザーの権限を取得できませんでした。";
const USER_ID_CONSTRUCTION_FAILED: &str =
    "Redisに登録された値からユーザーIDを確認できませんでした。";
const TOKEN_TYPE_CONSTRUCTION_FAILED: &str =
    "Redisに登録された値からトークンの種類を確認できませんでした。";
const USER_PERMISSION_CONSTRUCTION_FAILED: &str =
    "Redisに登録された値からユーザー権限を確認できませんでした。";

#[cfg(test)]
mod tests {
    use super::*;

    /// Redisに登録するユーザーIDとトークンの種類を示す文字列を生成できることを確認
    #[test]
    fn can_generate_user_id_and_token_type_string() -> anyhow::Result<()> {
        let user_id = UserId::default();
        let token_type = TokenType::Access;
        let user_permission_code = UserPermissionCode::Admin;
        let expected = format!("{}:{}:{}", user_id, token_type, user_permission_code);
        let actual = generate_value(user_id, token_type, user_permission_code);
        assert_eq!(expected, actual);

        Ok(())
    }

    /// Redisに登録されている文字列の形式を、ユーザーIDとトークンの種類に分割できることを確認
    #[test]
    fn can_split_user_id_and_token_type() -> anyhow::Result<()> {
        let expected_user_id = UserId::default();
        let expected_token_type = TokenType::Refresh;
        let expected_user_permission_code = UserPermissionCode::General;
        let input = format!(
            "{}:{}:{}",
            expected_user_id, expected_token_type, expected_user_permission_code
        );
        let (user_id, token_type, user_permission_code) = split_value(&input)?;
        assert_eq!(expected_user_id, user_id);
        assert_eq!(expected_token_type, token_type);
        assert_eq!(expected_user_permission_code, user_permission_code);

        Ok(())
    }
}
