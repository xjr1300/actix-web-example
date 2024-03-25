use once_cell::sync::Lazy;
use regex::Regex;
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
/// * 作成日時
/// * 更新日時
///
/// ユーザーは、固定電話番号または携帯電話番号の両方またはどちらかを記録しなければならない。
/// ユーザーは、Eメール・アドレスとパスワードで認証する。
/// アクティブ・フラグが`true`のユーザーのみ認証できる。
/// アプリケーション設定の大認証繰り返し時間内に、最大認証繰り返し回数より多く認証に失敗したとき、
/// 認証に失敗させて、次回以降の認証をすべて拒否する。
/// ユーザーを登録するとき、PostgresSQLの場合、作成日時と更新日時に`STATEMENT_TIMESTAMP()`を使用して、
/// 同じ日時が記録されるようにする。

/// Eメール・アドレス
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct EmailAddress {
    #[validate(email)]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// ユーザーの氏名の苗字
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct FamilyName {
    #[validate(length(min = 1, max = 40))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// ユーザーの氏名の名前
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct GivenName {
    #[validate(length(min = 1, max = 40))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 郵便番号の正規表現
static POSTAL_CODE_EXPRESSION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[0-9]{3}-[0-9]{4}$").unwrap());

/// 郵便番号
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct PostalCode {
    #[validate(regex(path = "*POSTAL_CODE_EXPRESSION"))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 住所
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct Address {
    #[validate(length(min = 1, max = 80))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 固定電話番号の正規表現
static FIXED_PHONE_NUMBER_EXPRESSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^0([0-9]-[0-9]{4}|[0-9]{2}-[0-9]{3}|[0-9]{3}-[0-9]{2}|[0-9]{4}-[0-9])-[0-9]{4}$")
        .unwrap()
});

/// 固定電話番号
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct FixedPhoneNumber {
    #[validate(regex(path = "*FIXED_PHONE_NUMBER_EXPRESSION"))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 携帯電話番号の正規表現
static MOBILE_PHONE_NUMBER_EXPRESSION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^0[789]0-[0-9]{4}-[0-9]{4}$").unwrap());

/// 携帯電話番号
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct MobilePhoneNumber {
    #[validate(regex(path = "*MOBILE_PHONE_NUMBER_EXPRESSION"))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 備考
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
pub struct Remarks {
    #[validate(length(min = 1, max = 400))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

#[cfg(test)]
mod tests {
    use super::{EmailAddress, FamilyName, FixedPhoneNumber, PostalCode};
    use crate::{common::error::DomainError, models::user::MobilePhoneNumber};

    /// Eメール・アドレスとして妥当な文字列から、Eメール・アドレスを構築できることを確認
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

    /// ユーザーの名前の性として妥当な文字列から、ユーザー名の名前の姓を構築できることを確認
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

    /// ユーザーの名前の性として無効な文字列から、ユーザー名の名前の姓を構築できないことを確認
    #[test]
    fn can_not_construct_family_name_from_invalid_strings() {
        let candidates = [String::from(""), "a".repeat(41), String::from("          ")];
        for candidate in candidates.iter() {
            assert!(FamilyName::new(candidate).is_err(), "`{}`", candidate);
        }
    }

    /// 郵便番号として妥当な文字列から、郵便番号を構築できることを確認
    #[test]
    fn construct_postal_code_from_valid_strings() {
        let candidates = ["000-0000", "123-4567", "999-9999"];
        for expected in candidates {
            let instance = PostalCode::new(expected).unwrap();
            assert_eq!(expected, instance.value(), "`{}`", expected);
        }
    }

    /// 郵便番号として無効な文字列から、郵便番号を構築できないことを確認
    #[test]
    fn can_not_construct_postal_code_from_invalid_strings() {
        let candidates = ["", "11-1111", "111-111", "11a-1111", "111-111a"];
        for expected in candidates {
            assert!(PostalCode::new(expected).is_err(), "`{}`", expected);
        }
    }

    /// 固定電話番号として妥当な文字列から、固定電話番号を構築できることを確認
    #[test]
    fn construct_fixed_phone_number_from_valid_strings() {
        let candidates = [
            "01-2345-6789",
            "012-345-6789",
            "0123-45-6789",
            "01234-5-6789",
        ];
        for expected in candidates {
            let instance = FixedPhoneNumber::new(expected).unwrap();
            assert_eq!(expected, instance.value(), "`{}`", expected);
        }
    }

    /// 固定電話番号として無効な文字列から、固定電話番号を構築できないことを確認
    #[test]
    fn can_not_construct_fixed_phone_number_from_invalid_strings() {
        let candidates = [
            "",
            "---",
            "11-1111-1111",
            "0a-2345-6789",
            "01-234a-6789",
            "01-2345-678a",
            "01a-345-6789",
            "012-34a-6789",
            "012-345-678a",
            "012a-45-6789",
            "0123-4a-6789",
            "0123-45-678a",
            "0123a-5-6789",
            "01234-a-6789",
            "01234-5-678a",
            "01-234-6789",
            "01-23456-6789",
            "012-34-6789",
            "012-3456-6789",
            "0123-4-6789",
            "0123-456-6789",
            "01234--6789",
            "01234-56-6789",
        ];
        for expected in candidates {
            assert!(FixedPhoneNumber::new(expected).is_err(), "`{}`", expected);
        }
    }

    /// 携帯電話番号として妥当な文字列から、携帯電話番号を構築できることを確認
    #[test]
    fn construct_mobile_phone_number_from_valid_strings() {
        let candidates = ["070-1234-5678", "080-1234-5678", "090-1234-5678"];
        for expected in candidates {
            let instance = MobilePhoneNumber::new(expected).unwrap();
            assert_eq!(expected, instance.value(), "`{}`", expected);
        }
    }

    /// 携帯電話番号として無効な文字列から、携帯電話番号を構築できないことを確認
    #[test]
    fn can_not_construct_mobile_phone_number_from_invalid_strings() {
        let candidates = [
            "",
            "---",
            "09-1234-5678",
            "0900-1234-5678",
            "090-123-5678",
            "090-12345-5678",
            "090-1234-567",
            "090-1234-56789",
            "010-1234-5678",
            "190-1234-5678",
            "091-1234-5678",
            "09a-1234-5678",
            "090-123a-5678",
            "090-1234-567a",
        ];
        for expected in candidates {
            assert!(MobilePhoneNumber::new(expected).is_err(), "`{}`", expected);
        }
    }
}
