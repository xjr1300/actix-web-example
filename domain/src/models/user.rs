use validator::Validate;

use macros::{DomainPrimitive, PrimitiveDisplay, StringPrimitive};

use crate::common::error::{DomainError, DomainResult};

/// ユーザー
///
/// ユーザーが保有するフィールドを次に示す。
///
/// * ユーザーID
/// * Eメール・アドレス
/// * パスワード
/// * アクティブ・フラグ
/// * 名前（姓）
/// * 名前（名）
/// * 郵便番号
/// * 住所
/// * 固定電話番号
/// * 携帯電話番号
/// * 備考
/// * 参加日時
/// * 作成日時
/// * 更新日時
///
/// ユーザーは、Eメール・アドレスとパスワードで認証する。
/// アクティブ・フラグが`true`のユーザーのみ認証できる。
/// アプリケーション設定の大認証繰り返し時間内に、最大認証繰り返し回数より多く認証に失敗したとき、
/// 認証に失敗させて、次回以降の認証をすべて拒否する。

/// Eメール・アドレス
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive,
)]
pub struct EmailAddress {
    #[validate(email)]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

#[cfg(test)]
mod tests {
    use crate::common::error::DomainError;

    use super::EmailAddress;

    /// Eメール・アドレスとして有効な文字列から、Eメール・アドレスを構築できることを確認
    #[test]
    fn construct_email_address_from_valid_string() {
        let expected = "foo@example.com";
        let instance = EmailAddress::new(expected).unwrap();
        assert_eq!(expected, instance.value());
    }

    /// Eメール・アドレスとして無効な文字列から、Eメールアドレスを構築できないことを確認
    #[test]
    fn can_not_construct_email_address_from_invalid_string() {
        match EmailAddress::new("invalid-email-address") {
            Ok(_) => panic!("EmailAddress must not be constructed from invalid string"),
            Err(err) => match err {
                DomainError::Validation(_) => {},
                _ =>panic!("DomainError::Validation should be raised when constructing from invalid string")
            }
        }
    }
}
