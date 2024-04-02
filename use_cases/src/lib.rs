use std::borrow::Cow;

pub mod accounts;

pub type ProcessUseCaseResult<T> = Result<T, UseCaseError>;

/// ユース・ケース・エラー分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl std::fmt::Display for UseCaseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            UseCaseErrorKind::Unexpected => "unexpected",
            UseCaseErrorKind::Validation => "validation",
            UseCaseErrorKind::DomainRule => "domain_rule",
            UseCaseErrorKind::Repository => "repository",
        };
        write!(f, "{}", s)
    }
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
    /// アプリケーション・エラー・コード
    pub error_code: u32,
    /// メッセージ
    pub message: Cow<'static, str>,
}

impl UseCaseError {
    /// ユース・ケース・エラーを構築する。
    ///
    /// # 引数
    ///
    /// * `kind` - ユース・ケース・エラー分類
    /// * `error_code` - アプリケーション・エラー・コード
    /// * `message` - メッセージ
    ///
    /// # 戻り値
    ///
    /// ユース・ケース・エラー
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
            error_code: 0,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Validation,
            error_code: 1,
            message: message.into(),
        }
    }

    pub fn domain_rule(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::DomainRule,
            error_code: 2,
            message: message.into(),
        }
    }

    pub fn repository(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: UseCaseErrorKind::Repository,
            error_code: 3,
            message: message.into(),
        }
    }
}
