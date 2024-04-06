use secrecy::SecretString;

use crate::{UseCaseError, UseCaseResult};

/// パスワード設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PasswordSettings {
    /// ペッパー
    pub pepper: SecretString,
    /// パスワードをハッシュ化するときのメモリサイズ
    pub hash_memory: u32,
    /// パスワードをハッシュ化するときの反復回数
    pub hash_iterations: u32,
    /// パスワードをハッシュ化するときの並列度
    pub hash_parallelism: u32,
}

/// 認証設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthorizationSettings {
    /// ユーザーのサインインの試行を許可する期間（秒）
    pub attempting_seconds: u32,
    /// ユーザーのアカウントをロックするまでのサインイン失敗回数
    pub number_of_failures: u8,
    /// JWTトークンを生成するときの秘密鍵
    pub jwt_token_secret: SecretString,
    /// アクセストークンの有効期限（秒）
    pub access_token_seconds: u64,
    /// リフレッシュトークンの有効期限（秒）
    pub refresh_token_seconds: u64,
}

impl AuthorizationSettings {
    /// 認証設定を検証する。
    pub fn validate(&self) -> UseCaseResult<()> {
        if self.refresh_token_seconds <= self.access_token_seconds {
            tracing::error!("{} ({}:{})", INVALID_TOKEN_EXPIRATIONS, file!(), line!());
            return Err(UseCaseError::unexpected(INVALID_TOKEN_EXPIRATIONS));
        }

        Ok(())
    }
}

const INVALID_TOKEN_EXPIRATIONS: &str =
    "リフレッシュトークンの有効期限は、アクセストークンの有効期限よりも長くなければなりません。";

#[cfg(test)]
pub mod tests {
    use secrecy::SecretString;

    use super::*;

    pub fn authorization_settings() -> AuthorizationSettings {
        AuthorizationSettings {
            attempting_seconds: 300,
            number_of_failures: 5,
            jwt_token_secret: SecretString::new(String::from("asdf")),
            access_token_seconds: 300,
            refresh_token_seconds: 400,
        }
    }

    /// 認証設定が適切であることを検証できるか確認
    #[test]
    fn authorization_settings_is_valid() {
        let settings = authorization_settings();
        assert!(settings.validate().is_ok());
    }

    /// 認証設定が適切でないことを検証できるか確認
    #[test]
    fn authorization_settings_is_invalid() {
        let mut settings = authorization_settings();
        settings.refresh_token_seconds = 0;
        assert!(settings.validate().is_err());
        settings.refresh_token_seconds = 300;
        assert!(settings.validate().is_err());
    }
}
