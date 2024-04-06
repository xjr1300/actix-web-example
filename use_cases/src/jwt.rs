use std::{collections::BTreeMap, str::FromStr as _};

use hmac::{Hmac, Mac};
use jwt::{SignWithKey as _, VerifyWithKey as _};
use secrecy::{ExposeSecret as _, SecretString};
use sha2::Sha256;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::user::UserId;

use crate::{UseCaseError, UseCaseResult};

const SUBJECT_KEY: &str = "sub";
const EXPIRATION_KEY: &str = "exp";

type HmacKey = Hmac<Sha256>;

/// クレイム
#[derive(Debug, Clone, Copy)]
pub struct Claim {
    /// ユーザーID
    pub user_id: UserId,
    /// 有効期限を示すUNIXエポック秒
    pub expiration: u64,
}

/// ユーザーIDと有効期限を指定したJWTを生成する。
///
/// # 引数
///
/// * `claim` - クレイム
/// * `secret_key` - JWTを生成するときの秘密鍵
///
/// # 戻り値
///
/// JWT
fn generate_token(claim: Claim, secret_key: &SecretString) -> UseCaseResult<SecretString> {
    let key: HmacKey = generate_hmac_key(secret_key)?;
    let mut claims = BTreeMap::new();
    claims.insert(SUBJECT_KEY, claim.user_id.value.to_string());
    claims.insert(EXPIRATION_KEY, claim.expiration.to_string());
    let token = claims.sign_with_key(&key).map_err(|e| {
        tracing::error!("{} ({}:{})", e, file!(), line!());
        UseCaseError::unexpected(e.to_string())
    })?;

    Ok(SecretString::new(token))
}

fn generate_hmac_key(secret_key: &SecretString) -> UseCaseResult<HmacKey> {
    Hmac::new_from_slice(secret_key.expose_secret().as_bytes()).map_err(|e| {
        tracing::error!("{} ({}:{})", e, file!(), line!());
        UseCaseError::unexpected(
            "JWTを生成するためにHMACを秘密鍵から構築するときにエラーが発生しました。",
        )
    })
}

/// JWTトークンのペア
pub struct TokenPair {
    /// アクセストークン
    pub access: SecretString,
    /// リフレッシュトークン
    pub refresh: SecretString,
}

/// JWTのアクセストークンとリフレッシュトークンを生成する。
///
/// # 引数
///
/// * `user_id` - ユーザーID
/// * `access_expiration` - アクセストークンの有効期限
/// * `refresh_expiration` - リフレッシュトークンの有効期限
/// * `secret_key` - JWTを作成する秘密鍵
pub fn generate_token_pair(
    user_id: UserId,
    access_expiration: OffsetDateTime,
    refresh_expiration: OffsetDateTime,
    secret_key: &SecretString,
) -> UseCaseResult<TokenPair> {
    // アクセストークンを生成
    let claim = Claim {
        user_id,
        expiration: access_expiration.unix_timestamp() as u64,
    };
    let access_token = generate_token(claim, secret_key)?;
    // リフレッシュトークンを生成
    let claim = Claim {
        user_id,
        expiration: refresh_expiration.unix_timestamp() as u64,
    };
    let refresh_token = generate_token(claim, secret_key)?;

    Ok(TokenPair {
        access: access_token,
        refresh: refresh_token,
    })
}

/// JWTからクレイムを取り出す。
///
/// # 引数
///
/// * `token` - JWT
/// * `secret_key` - JWTを生成するときの秘密鍵
///
/// # 戻り値
///
/// クレイム
pub fn retrieve_claim_from_token(
    token: &SecretString,
    secret_key: &SecretString,
) -> UseCaseResult<Claim> {
    let key: HmacKey = generate_hmac_key(secret_key)?;
    let claims: BTreeMap<String, String> =
        token.expose_secret().verify_with_key(&key).map_err(|e| {
            tracing::error!("{} ({}:{})", e, file!(), line!());
            UseCaseError::unexpected("JWTを検証するときにエラーが発生しました。")
        })?;
    // ユーザーIDを取得
    let user_id = claims.get(SUBJECT_KEY).ok_or_else(|| {
        tracing::error!("{} ({}:{})", USER_ID_NOT_FOUND_IN_PAYLOAD, file!(), line!());
        UseCaseError::unexpected(USER_ID_NOT_FOUND_IN_PAYLOAD)
    })?;
    let user_id = Uuid::from_str(user_id).map_err(|_| {
        tracing::error!("{} ({}:{})", INVALID_USER_ID_IN_PAYLOAD, file!(), line!());
        UseCaseError::unexpected(INVALID_USER_ID_IN_PAYLOAD)
    });
    let user_id = UserId::new(user_id.unwrap());
    // 有効期限を取得
    let expiration = claims.get(EXPIRATION_KEY).ok_or_else(|| {
        tracing::error!(
            "{} ({}:{})",
            EXPIRATION_NOT_FOUND_IN_PAYLOAD,
            file!(),
            line!()
        );
        UseCaseError::unexpected(EXPIRATION_NOT_FOUND_IN_PAYLOAD)
    })?;
    let expiration = expiration.parse::<u64>().map_err(|_| {
        tracing::error!(
            "{} ({}:{})",
            INVALID_EXPIRATION_IN_PAYLOAD,
            file!(),
            line!()
        );
        UseCaseError::unexpected(INVALID_USER_ID_IN_PAYLOAD)
    })?;

    Ok(Claim {
        user_id,
        expiration,
    })
}

const USER_ID_NOT_FOUND_IN_PAYLOAD: &str = "JWTのペイロードにユーザーIDが記録されていません。";
const INVALID_USER_ID_IN_PAYLOAD: &str =
    "JWTのペイロードに記録されているユーザーIDがUUIDv4の形式になっていません。";
const EXPIRATION_NOT_FOUND_IN_PAYLOAD: &str = "JWTのペイロードに有効期限が記録されていません。";
const INVALID_EXPIRATION_IN_PAYLOAD: &str =
    "JWTのペイロードに記録されている有効期限が正の数値でありません。";

#[cfg(test)]
mod tests {
    use time::Duration;

    use crate::settings::tests::authorization_settings;

    use super::*;

    /// JWTを生成できることを確認
    #[test]
    fn can_generate_token() -> anyhow::Result<()> {
        // JWTを生成
        let user_id = UserId::default();
        let dt = OffsetDateTime::now_utc();
        let expiration = dt.unix_timestamp() as u64 + 300u64;
        let claim = Claim {
            user_id,
            expiration,
        };
        let secret_key = SecretString::new(String::from("some-secret"));
        let token = generate_token(claim, &secret_key).unwrap();

        // JWTを検証
        let claim = retrieve_claim_from_token(&token, &secret_key).unwrap();
        assert_eq!(claim.user_id, user_id);
        assert_eq!(claim.expiration, expiration);

        Ok(())
    }

    /// アクセストークンとリフレッシュトークンを生成できることを確認
    #[test]
    fn can_generate_token_pair() -> anyhow::Result<()> {
        let settings = authorization_settings();
        let user_id = UserId::default();
        let dt = OffsetDateTime::now_utc();
        let access_expiration = dt + Duration::seconds(settings.access_token_seconds as i64);
        let refresh_expiration = dt + Duration::seconds(settings.refresh_token_seconds as i64);
        let tokens = generate_token_pair(
            user_id,
            access_expiration,
            refresh_expiration,
            &settings.jwt_token_secret,
        )?;
        assert_ne!(
            tokens.access.expose_secret(),
            tokens.refresh.expose_secret(),
            "アクセストークンとリフレッシュトークンが同じです。"
        );

        Ok(())
    }
}
