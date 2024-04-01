use std::marker::PhantomData;

use once_cell::sync::Lazy;
use regex::Regex;
use uuid::Uuid;

use macros::{DomainPrimitive, OptionalStringPrimitive, PrimitiveDisplay, StringPrimitive};
use validator::Validate;

use crate::common::{DomainError, DomainResult};

/// エンティティID
///
/// UUID v4でエンティティを識別するIDを表現する。
/// `PhantomData`でエンティティの型を識別する。
#[derive(Debug, PartialEq, Eq, Hash, DomainPrimitive)]
pub struct EntityId<T> {
    #[value_getter(ret = "val")]
    value: Uuid,
    _phantom: PhantomData<T>,
}

impl<'a, T> TryFrom<&'a str> for EntityId<T> {
    type Error = DomainError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(s) {
            Ok(value) => Ok(Self {
                value,
                _phantom: PhantomData,
            }),
            Err(_) => Err(DomainError::Validation(
                "文字列の形式がUUIDv4形式でありません。".into(),
            )),
        }
    }
}

impl<T> Copy for EntityId<T> {}

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for EntityId<T> {
    fn default() -> Self {
        Self {
            value: Uuid::new_v4(),
            _phantom: Default::default(),
        }
    }
}

impl<T> EntityId<T> {
    pub fn new(value: Uuid) -> Self {
        Self {
            value,
            _phantom: Default::default(),
        }
    }
}

/// Eメール・アドレスの長さ
///
/// Eメール・アドレスの文字数の最小値は規定されていないため、"a@a.jp"のようなアドレスを想定して6文字とした。
/// Eメール・アドレスの文字数の最大値は、次を参照して設定した。
/// <https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address>
const EMAIL_ADDRESS_MIN_LEN: u64 = 6;
const EMAIL_ADDRESS_MAX_LEN: u64 = 254;

/// Eメール・アドレス
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "Eメール・アドレス",
    message = "Eメール・アドレスの形式が間違っています。"
)]
pub struct EmailAddress {
    #[validate(email)]
    #[validate(length(min = EMAIL_ADDRESS_MIN_LEN, max = EMAIL_ADDRESS_MAX_LEN))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// ユーザーの氏名の性
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "ユーザーの氏名の姓",
    message = "ユーザーの氏名の姓は1文字以上40文字以下です。"
)]
pub struct FamilyName {
    #[validate(length(min = 1, max = 40))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// ユーザーの氏名の名
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "ユーザーの氏名の名",
    message = "ユーザーの氏名の名は1文字以上40文字以下です。"
)]
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
#[primitive(name = "郵便番号", message = "郵便番号の形式が間違っています。")]
pub struct PostalCode {
    #[validate(regex(path = "*POSTAL_CODE_EXPRESSION",))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 住所
#[derive(Debug, Clone, Validate, DomainPrimitive, PrimitiveDisplay, StringPrimitive)]
#[primitive(name = "住所", message = "住所は1文字以上80文字未満です。")]
pub struct Address {
    #[validate(length(min = 1, max = 80))]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

/// 固定電話番号
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(
    name = "固定電話番号",
    regex = r"^0([0-9]-[0-9]{4}|[0-9]{2}-[0-9]{3}|[0-9]{3}-[0-9]{2}|[0-9]{4}-[0-9])-[0-9]{4}$"
)]
pub struct FixedPhoneNumber(Option<String>);

/// 携帯電話番号
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "携帯電話番号", regex = r"^0[789]0-[0-9]{4}-[0-9]{4}$")]
pub struct MobilePhoneNumber(Option<String>);

/// 備考
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "備考", max = 400)]
pub struct Remarks(Option<String>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::DomainError;

    /// UUID v4形式の文字列からエンティティIDを構築できるか確認
    #[test]
    fn construct_entity_id_from_valid_string() {
        let expected = "27db4b5f-1ff8-4691-ba07-f54b56884241";
        let entity_id: EntityId<i32> = expected.try_into().unwrap();
        let value_str = entity_id.value.to_string();
        assert_eq!(expected, value_str);
    }

    /// UUID v4形式でない文字列からエンティティIDを構築できないことを確認
    #[test]
    fn can_not_construct_entity_id_from_invalid_string() {
        let invalid_string = "invalid uuid v4 string";
        let result: Result<EntityId<i32>, DomainError> = invalid_string.try_into();
        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::Validation(_) => {}
            _ => panic!("expected DomainError::Validation"),
        }
    }

    /// Eメール・アドレスとして妥当な文字列から、Eメール・アドレスを構築できることを確認
    #[test]
    fn construct_email_address_from_valid_strings() {
        let candidates = ["a@a.jp", "foo@example.com"];
        for candidate in candidates {
            let instance = EmailAddress::new(candidate).unwrap();
            assert_eq!(candidate, instance.value());
        }
    }

    /// Eメール・アドレスとして無効な文字列から、Eメールアドレスを構築できないことを確認
    #[test]
    fn can_not_construct_email_address_from_invalid_strings() {
        let domain = "@example.com";
        let length_of_user_name = EMAIL_ADDRESS_MAX_LEN as usize + 1 - domain.len();
        let mut invalid_email_address = "a".repeat(length_of_user_name);
        invalid_email_address.push_str(domain);
        assert_eq!(
            EMAIL_ADDRESS_MAX_LEN + 1,
            invalid_email_address.len() as u64
        );

        let candidates = ["", "a", "a@a.a", "aaaaaa", invalid_email_address.as_str()];
        for candidate in candidates {
            match EmailAddress::new(candidate) {
            Ok(_) => panic!("EmailAddress must not be constructed from invalid string: {}", candidate),
            Err(err) => match err {
                DomainError::DomainRule(_) => {},
                _ =>panic!("DomainError::DomainRule should be raised when constructing from invalid string: {}", candidate)
            }
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
            let instance = FixedPhoneNumber::try_from_str(expected).unwrap();
            assert_eq!(expected, instance.value().unwrap(), "`{}`", expected);
        }
    }

    /// 固定電話番号として無効な文字列から、固定電話番号を構築できないことを確認
    #[test]
    fn can_not_construct_fixed_phone_number_from_invalid_strings() {
        let candidates = [
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
            assert!(
                FixedPhoneNumber::try_from(expected).is_err(),
                "`{}`",
                expected
            );
        }
    }

    /// 携帯電話番号として妥当な文字列から、携帯電話番号を構築できることを確認
    #[test]
    fn construct_mobile_phone_number_from_valid_strings() {
        let candidates = ["070-1234-5678", "080-1234-5678", "090-1234-5678"];
        for expected in candidates {
            let instance = MobilePhoneNumber::try_from_str(expected).unwrap();
            assert_eq!(expected, instance.value().unwrap(), "`{}`", expected);
        }
    }

    /// 携帯電話番号として無効な文字列から、携帯電話番号を構築できないことを確認
    #[test]
    fn can_not_construct_mobile_phone_number_from_invalid_strings() {
        let candidates = [
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
            assert!(
                MobilePhoneNumber::try_from_str(expected).is_err(),
                "`{}`",
                expected
            );
        }
    }
}
