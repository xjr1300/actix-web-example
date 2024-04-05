pub mod accounts;
pub mod passwords;

use std::borrow::Cow;

use enum_display::EnumDisplay;

pub type ProcessUseCaseResult<T> = Result<T, UseCaseError>;

/// ユース・ケース・エラー・コード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum UseCaseErrorCode {
    Unexpected = 0,
    Validation = 1,
    DomainRule = 2,
    Repository = 3,
}

/// ユース・ケース・エラー分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDisplay)]
#[enum_display(case = "Lower")]
pub enum UseCaseErrorKind {
    /// 予期していないエラー
    ///
    /// アプリケーション・エラー・コードは常に0とする。
    Unexpected,

    /// 検証エラー
    ///
    /// アプリケーション・エラー・コードは常に1とする。
    Validation,

    /// ドメイン・ルール・エラー
    ///
    /// アプリケーションエラーコードは常に2とする。
    DomainRule,

    /// リポジトリ・エラー
    ///
    ///
    /// アプリケーションエラーコードは常に3とする。
    Repository,
}

/// ユース・ケース・エラー
///
/// 一般的なエラーの場合は、`unexpected`、`validation`など、それぞれのユース・ケース・エラー分類
/// 別のメソッドを呼び出して、ユース・ケース・エラーを構築する。
///
/// ユース・ケースで特殊なエラーの場合は、`new`メソッドを呼び出してユース・ケース・エラーを構築する。
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct UseCaseError {
    /// ユース・ケース・エラー分類
    pub kind: UseCaseErrorKind,
    /// ユース・ケース・エラー・コード
    pub error_code: UseCaseErrorCode,
    /// メッセージ
    pub message: Cow<'static, str>,
}

impl UseCaseError {
    /// ユース・ケース・エラーを構築する。
    ///
    /// # 引数
    ///
    /// * `kind` - ユース・ケース・エラー分類
    /// * `error_code` - ユース・ケース・エラー・コード
    /// * `message` - メッセージ
    ///
    /// # 戻り値
    ///
    /// ユース・ケース・エラー
    pub fn new(
        kind: UseCaseErrorKind,
        error_code: UseCaseErrorCode,
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
            error_code: UseCaseErrorCode::Unexpected,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Validation,
            error_code: UseCaseErrorCode::Validation,
            message: message.into(),
        }
    }

    pub fn domain_rule(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::DomainRule,
            error_code: UseCaseErrorCode::DomainRule,
            message: message.into(),
        }
    }

    pub fn repository(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Repository,
            error_code: UseCaseErrorCode::Repository,
            message: message.into(),
        }
    }
}
