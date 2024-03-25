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
/// ユーザーは、固定電話番号または携帯電話番号の両方またはどちらかを記録しなければならない。
/// ユーザーは、Eメール・アドレスとパスワードで認証する。
/// アクティブ・フラグが`true`のユーザーのみ認証できる。
/// アプリケーション設定の大認証繰り返し時間内に、最大認証繰り返し回数より多く認証に失敗したとき、
/// 認証に失敗させて、次回以降の認証をすべて拒否する。

/// Eメール・アドレス
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct EmailAddress {
    #[validate(email)]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// ユーザーの名前の性
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct FamilyName {
    #[validate(length(min = 1, max = 40))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

#[cfg(test)]
mod tests {
    use crate::common::error::DomainError;

    use super::{EmailAddress, FamilyName};

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
                DomainError::DomainRule(_) => {},
                _ =>panic!("DomainError::DomainRule should be raised when constructing from invalid string")
            }
        }
    }

    /// ユーザーの名前の性
    #[test]
    fn construct_family_name_from_valid_string() {
        let candidates = [
            "family_name",
            " family_name",
            "family_name ",
            " family_name ",
        ];
        let expected = "family_name";
        for candidate in candidates {
            let instance = FamilyName::new(candidate).unwrap();
            assert_eq!(expected, instance.value(), "`{}`", candidate);
        }
    }
    #[test]
    fn can_not_construct_family_name_from_invalid_strings() {
        let candidates = [String::from(""), "a".repeat(41), String::from("          ")];
        for candidate in candidates.iter() {
            assert!(FamilyName::new(candidate).is_err(), "`{}`", candidate);
        }
    }
}
