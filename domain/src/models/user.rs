use validator::Validate;

use macros::{DomainPrimitive, StringPrimitive};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, DomainPrimitive, StringPrimitive)]
pub struct EmailAddress {
    #[validate(email)]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

}

#[cfg(test)]
mod tests {
    use super::EmailAddress;

    /// 有効な文字列でEメール・アドレスを構築できることを確認
    #[test]
    fn construct_email_address_from_valid_string() {
        let expected = "foo@example.com";
        let instance = EmailAddress::new(expected).unwrap();
        assert_eq!(expected, instance.value());
    }

    /// 無効な文字列でEメールアドレスを構築できないことを確認
    #[test]
    fn can_not_construct_email_address_from_invalid_string() {
        assert!(EmailAddress::new("invalid-email-address").is_err())
    }
}
