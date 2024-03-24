use std::borrow::Cow;

/// ドメイン・エラー
#[derive(Debug, thiserror::Error)]
pub enum DomainError<'a> {
    /// 検証エラー
    ///
    /// 文字列を数値に変換できない場合など、ドメイン・ルールを伴わない検証エラーを表現する。
    #[error("{0}")]
    Validation(Cow<'a, str>),

    /// ドメイン・ルールエラー
    ///
    /// ドメイン・ルールに違反したことを表現する。
    #[error("{0}")]
    DomainRule(Cow<'a, str>),

    /// リポジトリ・エラー
    ///
    /// リポジトリで発生したエラーを表現する。
    #[error("{0}")]
    Repository(anyhow::Error),
}
