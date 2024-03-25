use once_cell::sync::Lazy;
use regex::Regex;
use time::OffsetDateTime;
use validator::Validate;

use macros::{DomainPrimitive, Getter, PrimitiveDisplay, StringPrimitive};

use crate::common::{DomainError, DomainResult};
use crate::models::passwords::PhcPassword;
use crate::models::EntityId;

/// ユーザーID
pub type UserId = EntityId<User>;

/// ユーザー
///
/// ユーザーは、固定電話番号または携帯電話番号の両方またはどちらかを記録しなければならない。
/// ユーザーは、Eメール・アドレスとパスワードで認証する。
/// アクティブ・フラグが`true`のユーザーのみ認証できる。
/// アプリケーション設定の大認証繰り返し時間内に、最大認証繰り返し回数より多く認証に失敗したとき、
/// 認証に失敗させて、次回以降の認証をすべて拒否する。
/// ユーザーを登録するとき、PostgresSQLの場合、作成日時と更新日時に`STATEMENT_TIMESTAMP()`を使用して、
/// 同じ日時が記録されるようにする。
#[derive(Debug, Clone, Getter)]
pub struct User {
    /// ユーザーID
    #[getter(ret = "val")]
    id: UserId,
    /// Eメール・アドレス
    #[getter(ret = "ref")]
    email: EmailAddress,
    /// パスワード（PHC文字列）
    #[getter(ret = "ref")]
    password: PhcPassword,
    /// アクティブ・フラグ
    #[getter(ret = "val")]
    active: bool,
    /// 氏名の苗字
    #[getter(ret = "ref")]
    family_name: FamilyName,
    /// 氏名の名前
    #[getter(ret = "ref")]
    given_name: GivenName,
    /// 郵便番号
    #[getter(ret = "ref")]
    postal_code: PostalCode,
    /// 住所
    #[getter(ret = "ref")]
    address: Address,
    /// 固定電話番号
    #[getter(ret = "ref")]
    fixed_phone_number: Option<FixedPhoneNumber>,
    /// 携帯電話番号
    #[getter(ret = "ref")]
    mobile_phone_number: Option<MobilePhoneNumber>,
    /// 備考
    #[getter(ret = "ref")]
    remarks: Option<Remarks>,
    /// 作成日時
    #[getter(ret = "val")]
    created_at: OffsetDateTime,
    /// 更新日時
    #[getter(ret = "val")]
    updated_at: OffsetDateTime,
}

impl User {
    /// ユーザーを構築する。
    ///
    /// 作成日時と更新日時は、UTCの現在日時を設定する。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: UserId,
        email: EmailAddress,
        password: PhcPassword,
        active: bool,
        family_name: FamilyName,
        given_name: GivenName,
        postal_code: PostalCode,
        address: Address,
        fixed_phone_number: Option<FixedPhoneNumber>,
        mobile_phone_number: Option<MobilePhoneNumber>,
        remarks: Option<Remarks>,
    ) -> DomainResult<Self> {
        // 固定電話番号または携帯電話番号を指定していない場合はエラー
        if fixed_phone_number.is_none() && mobile_phone_number.is_none() {
            return Err(DomainError::DomainRule(
                "must provide at least a fixed phone number or mobile phone number".into(),
            ));
        }
        // 現在の日時を取得
        // FIXME: ローカルな日時を設定したいが、`OffsetDateTime::now_local()`が動作しない。
        // よって、`time`クレートのfeaturesに`local-offset`を設定していない。
        let dt = OffsetDateTime::now_utc();

        Ok(Self {
            id,
            email,
            password,
            active,
            family_name,
            given_name,
            postal_code,
            address,
            fixed_phone_number,
            mobile_phone_number,
            remarks,
            created_at: dt,
            updated_at: dt,
        })
    }
}

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
    use secrecy::SecretString;

    use super::{
        Address, EmailAddress, FamilyName, FixedPhoneNumber, MobilePhoneNumber, PostalCode,
        Remarks, User, UserId,
    };
    use crate::common::DomainError;
    use crate::models::passwords::tests::VALID_RAW_PASSWORD;
    use crate::models::passwords::{generate_phc_string, PasswordPepper, RawPassword};

    /// ユーザーを構築できることを確認
    #[test]
    fn construct_user_from_valid_args() {
        let id = UserId::default();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let plain_password = SecretString::new(String::from(VALID_RAW_PASSWORD));
        let raw_password = RawPassword::new(plain_password).unwrap();
        let password_pepper = PasswordPepper(SecretString::new(String::from("password-pepper")));
        let password = generate_phc_string(&raw_password, &password_pepper).unwrap();
        let active = true;
        let family_name = FamilyName::new("foo").unwrap();
        let given_name = super::GivenName::new("bar").unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let address = Address::new("foo bar baz qux").unwrap();
        let phone_number_pairs = [
            (
                Some(FixedPhoneNumber::new("03-1234-5678").unwrap()),
                Some(MobilePhoneNumber::new("090-1234-5678").unwrap()),
            ),
            (Some(FixedPhoneNumber::new("03-1234-5678").unwrap()), None),
            (None, Some(MobilePhoneNumber::new("090-1234-5678").unwrap())),
        ];
        let remarks = Some(Remarks::new(String::from("remarks")).unwrap());
        for (fixed_phone_number, mobile_phone_number) in phone_number_pairs {
            assert!(
                User::new(
                    id,
                    email.clone(),
                    password.clone(),
                    active,
                    family_name.clone(),
                    given_name.clone(),
                    postal_code.clone(),
                    address.clone(),
                    fixed_phone_number.clone(),
                    mobile_phone_number.clone(),
                    remarks.clone()
                )
                .is_ok(),
                "{:?}, {:?}",
                fixed_phone_number,
                mobile_phone_number
            );
        }
    }

    /// 固定電話番号と携帯電話番号の両方とも指定していない場合に、ユーザーを構築できないことを確認
    #[test]
    fn can_not_construct_user_when_both_fixed_phone_number_and_mobile_is_none() {
        let id = UserId::default();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let plain_password = SecretString::new(String::from(VALID_RAW_PASSWORD));
        let raw_password = RawPassword::new(plain_password).unwrap();
        let password_pepper = PasswordPepper(SecretString::new(String::from("password-pepper")));
        let password = generate_phc_string(&raw_password, &password_pepper).unwrap();
        let active = true;
        let family_name = FamilyName::new("foo").unwrap();
        let given_name = super::GivenName::new("bar").unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let address = Address::new("foo bar baz qux").unwrap();
        assert!(User::new(
            id,
            email,
            password,
            active,
            family_name,
            given_name,
            postal_code,
            address,
            None,
            None,
            None,
        )
        .is_err());
    }

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
