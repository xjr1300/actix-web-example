pub mod accounts;
pub mod jwt;
pub mod passwords;
pub mod settings;

use std::borrow::Cow;

use domain::DomainError;
use enum_display::EnumDisplay;

pub type UseCaseResult<T> = Result<T, UseCaseError>;

#[repr(u32)]
pub enum UseCaseErrorCode {
    Unexpected = 0,
    Validation = 1,
    DomainRule = 2,
    Repository = 3,
    NotFound = 4,
    Unauthorized = 5,
}

/// ユースケースエラー分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDisplay)]
#[enum_display(case = "Lower")]
pub enum UseCaseErrorKind {
    /// 予期していないエラー
    Unexpected,

    /// 検証エラー
    Validation,

    /// ドメインルール・エラー
    DomainRule,

    /// リポジトリエラー
    Repository,

    /// 未検出
    NotFound,

    /// 不許可／未認証
    Unauthorized,
}

/// ユースケースエラー
///
/// 一般的なエラーの場合は、`unexpected`、`validation`など、それぞれのユースケースエラー分類
/// 別のメソッドを呼び出して、ユースケースエラーを構築する。
///
/// ユースケースで特殊なエラーの場合は、`new`メソッドを呼び出してユースケース・エラーを構築する。
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct UseCaseError {
    /// ユースケースエラー分類
    pub kind: UseCaseErrorKind,
    /// ユースケースエラー・コード
    pub error_code: u32,
    /// メッセージ
    pub message: Cow<'static, str>,
}

impl UseCaseError {
    /// ユースケースエラーを構築する。
    ///
    /// # 引数
    ///
    /// * `kind` - ユースケースエラー分類
    /// * `error_code` - ユースケースエラー・コード
    /// * `message` - メッセージ
    ///
    /// # 戻り値
    ///
    /// ユースケースエラー
    pub fn new(
        kind: UseCaseErrorKind,
        error_code: u32,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind,
            error_code,
            message: message.into(),
        }
    }

    pub fn unexpected(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Unexpected,
            error_code: UseCaseErrorCode::Unexpected as u32,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Validation,
            error_code: UseCaseErrorCode::Validation as u32,
            message: message.into(),
        }
    }

    pub fn domain_rule(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::DomainRule,
            error_code: UseCaseErrorCode::DomainRule as u32,
            message: message.into(),
        }
    }

    pub fn repository(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Repository,
            error_code: UseCaseErrorCode::Repository as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::NotFound,
            error_code: UseCaseErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Unauthorized,
            error_code: UseCaseErrorCode::Unauthorized as u32,
            message: message.into(),
        }
    }
}

impl From<DomainError> for UseCaseError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::Unexpected(error) => Self::unexpected(error.to_string()),
            DomainError::Validation(message) => Self::validation(message),
            DomainError::DomainRule(message) => Self::domain_rule(message),
            DomainError::Repository(error) => Self::repository(error.to_string()),
        }
    }
}

/// サインアップ
pub const ERR_SAME_EMAIL_ADDRESS_IS_REGISTERED: u32 = 1000;
pub const ERR_SPECIFY_FIXED_OR_MOBILE_NUMBER: u32 = 1001;
