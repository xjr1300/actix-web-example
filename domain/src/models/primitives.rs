use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr as _;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use secrecy::{ExposeSecret as _, SecretString};
use uuid::Uuid;

use macros::{OptionalStringPrimitive, PrimitiveDisplay, StringPrimitive};
use validator::Validate;

use crate::{DomainError, DomainResult};

/// エンティティID
///
/// UUID v4でエンティティを識別するIDを表現する。
/// `PhantomData`でエンティティの型を識別する。
#[derive(Debug)]
pub struct EntityId<T> {
    pub value: Uuid,
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

impl<T> std::fmt::Display for EntityId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for EntityId<T> {}

impl<T> std::hash::Hash for EntityId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self._phantom.hash(state);
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

/// コード
///
/// ジェネリック引数`T1`はコードテーブルの型を指定する。
/// ジェネリック引数`T2`はコードの型を指定する。
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NumericCode<T1, T2>
where
    T2: Clone + Copy,
{
    pub value: T2,
    _phantom: PhantomData<T1>,
}

impl<T1, T2: Clone + Copy> From<T2> for NumericCode<T1, T2> {
    fn from(value: T2) -> Self {
        Self {
            value,
            _phantom: Default::default(),
        }
    }
}

impl<T1, T2: Clone + Copy> Copy for NumericCode<T1, T2> {}

impl<T1, T2: Clone + Copy> Clone for NumericCode<T1, T2> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T1, T2: Clone + Copy + std::fmt::Display> std::fmt::Display for NumericCode<T1, T2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T1, T2: Clone + Copy> NumericCode<T1, T2> {
    pub fn new(value: T2) -> Self {
        Self {
            value,
            _phantom: Default::default(),
        }
    }
}

/// Eメールアドレスの長さ
///
/// Eメールアドレスの文字数の最小値は規定されていないため、"a@a.jp"のようなアドレスを想定して6文字とした。
/// Eメールアドレスの文字数の最大値は、次を参照して設定した。
/// <https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address>
const EMAIL_ADDRESS_MIN_LEN: u64 = 6;
const EMAIL_ADDRESS_MAX_LEN: u64 = 254;

/// Eメールアドレス
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "Eメールアドレス",
    message = "Eメールアドレスの形式が間違っています。"
)]
pub struct EmailAddress {
    #[validate(email)]
    #[validate(length(min = EMAIL_ADDRESS_MIN_LEN, max = EMAIL_ADDRESS_MAX_LEN))]
    pub value: String,
}

/// 未加工なパスワード
///
/// 未加工なパスワードは、次を満たさなければならない。
///
/// * 8文字以上
/// * 大文字、小文字のアルファベットをそれぞれ1つ以上含む
/// * 数字を1つ以上含む
/// * 次の記号を1つ以上含む
///   * ~`!@#$%^&*()_-+={[}]|\:;"'<,>.?/
/// * 同じ文字が4つ以上ない
#[derive(Debug, Clone, Validate)]
pub struct RawPassword {
    pub value: SecretString,
}

impl RawPassword {
    pub fn new(value: SecretString) -> DomainResult<Self> {
        let value = value.expose_secret().trim();
        validate_plain_password(value)?;
        let value =
            SecretString::from_str(value).map_err(|e| DomainError::Unexpected(anyhow!(e)))?;

        Ok(Self { value })
    }
}

/// パスワードの最小文字数
const PASSWORD_MIN_LENGTH: usize = 8;
/// パスワードに含めるシンボルの候補
const PASSWORD_SYMBOLS_CANDIDATES: &str = r#"~`!@#$%^&*()_-+={[}]|\:;"'<,>.?/"#;
/// パスワードに同じ文字が存在することを許容する最大数
/// 指定された数だけ同じ文字をパスワードに含めることを許可
const PASSWORD_MAX_NUMBER_OF_CHAR_APPEARANCES: u64 = 3;

/// パスワードがドメインルールを満たしているか確認する。
fn validate_plain_password(s: &str) -> DomainResult<()> {
    // パスワードの文字数を確認
    if s.len() < PASSWORD_MIN_LENGTH {
        return Err(DomainError::DomainRule(
            format!("パスワードは少なくとも{PASSWORD_MIN_LENGTH}文字以上指定してください。").into(),
        ));
    }
    // 大文字のアルファベットが含まれるか確認
    if !s.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(DomainError::DomainRule(
            "パスワードは大文字のアルファベットを1文字以上含めなくてはなりません。".into(),
        ));
    }
    // 小文字のアルファベットが含まれるか確認
    if !s.chars().any(|ch| ch.is_ascii_lowercase()) {
        return Err(DomainError::DomainRule(
            "パスワードは小文字のアルファベットを1文字以上含めなくてはなりません。".into(),
        ));
    }
    // 数字が含まれるか確認
    if !s.chars().any(|ch| ch.is_ascii_digit()) {
        return Err(DomainError::DomainRule(
            "パスワードは数字を1文字以上含めなくてはなりません。".into(),
        ));
    }
    // シンボルが含まれるか確認
    if !s.chars().any(|ch| PASSWORD_SYMBOLS_CANDIDATES.contains(ch)) {
        return Err(DomainError::DomainRule(
            format!(
                "パスワードは記号({})を1文字以上含めなくてはなりません。",
                PASSWORD_SYMBOLS_CANDIDATES
            )
            .into(),
        ));
    }
    // 文字の出現回数を確認
    let mut number_of_chars: HashMap<char, u64> = HashMap::new();
    s.chars().for_each(|ch| {
        *number_of_chars.entry(ch).or_insert(0) += 1;
    });
    let max_number_of_appearances = number_of_chars.values().max().unwrap();
    if PASSWORD_MAX_NUMBER_OF_CHAR_APPEARANCES < *max_number_of_appearances {
        return Err(DomainError::DomainRule(
            format!("パスワードは同じ文字を{PASSWORD_MAX_NUMBER_OF_CHAR_APPEARANCES}個より多く含めることはできません。").into()
        ));
    }

    Ok(())
}

/// PHC文字列正規表現(cspell: disable-next-line)
const PHC_STRING_EXPRESSION: &str = r#"^\$argon2id\$v=(?:16|19)\$m=\d{1,10},t=\d{1,10},p=\d{1,3}(?:,keyid=[A-Za-z0-9+/]{0,11}(?:,data=[A-Za-z0-9+/]{0,43})?)?\$[A-Za-z0-9+/]{11,64}\$[A-Za-z0-9+/]{16,86}$"#;

/// PHCパスワード文字列
#[derive(Debug, Clone)]
pub struct PhcPassword {
    pub value: SecretString,
}

impl PhcPassword {
    pub fn new(value: SecretString) -> DomainResult<Self> {
        let raw_phc = value.expose_secret();
        let re = Regex::new(PHC_STRING_EXPRESSION).unwrap();
        if !re.is_match(raw_phc) {
            return Err(DomainError::Validation(
                "PHC文字列に設定する文字列がPHC文字列の形式ではありません。".into(),
            ));
        }

        Ok(Self { value })
    }
}

/// ユーザーの氏名の性
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "ユーザーの氏名の姓",
    message = "ユーザーの氏名の姓は1文字以上40文字以下です。"
)]
pub struct FamilyName {
    #[validate(length(min = 1, max = 40))]
    pub value: String,
}

/// ユーザーの氏名の名
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "ユーザーの氏名の名",
    message = "ユーザーの氏名の名は1文字以上40文字以下です。"
)]
pub struct GivenName {
    #[validate(length(min = 1, max = 40))]
    pub value: String,
}

/// 郵便番号の正規表現
static POSTAL_CODE_EXPRESSION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[0-9]{3}-[0-9]{4}$").unwrap());

/// 郵便番号
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PrimitiveDisplay, StringPrimitive)]
#[primitive(name = "郵便番号", message = "郵便番号の形式が間違っています。")]
pub struct PostalCode {
    #[validate(regex(path = "*POSTAL_CODE_EXPRESSION",))]
    pub value: String,
}

/// 住所
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PrimitiveDisplay, StringPrimitive)]
#[primitive(name = "住所", message = "住所は1文字以上80文字未満です。")]
pub struct Address {
    #[validate(length(min = 1, max = 80))]
    pub value: String,
}

/// 固定電話番号
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(
    name = "固定電話番号",
    regex = r"^0([0-9]-[0-9]{4}|[0-9]{2}-[0-9]{3}|[0-9]{3}-[0-9]{2}|[0-9]{4}-[0-9])-[0-9]{4}$"
)]
pub struct OptionalFixedPhoneNumber(Option<String>);

/// 携帯電話番号
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "携帯電話番号", regex = r"^0[789]0-[0-9]{4}-[0-9]{4}$")]
pub struct OptionalMobilePhoneNumber(Option<String>);

/// 備考
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "備考", max = 400)]
pub struct OptionalRemarks(Option<String>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainError;

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

    /// Eメールアドレスとして妥当な文字列から、Eメール・アドレスを構築できることを確認
    #[test]
    fn construct_email_address_from_valid_strings() {
        let candidates = ["a@a.jp", "foo@example.com"];
        for candidate in candidates {
            let instance = EmailAddress::new(candidate).unwrap();
            assert_eq!(candidate, instance.value);
        }
    }

    /// Eメールアドレスとして無効な文字列から、Eメールアドレスを構築できないことを確認
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
                DomainError::Validation(_) => {},
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
            assert_eq!(expected, instance.value, "`{}`", candidate);
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
            assert_eq!(expected, instance.value, "`{}`", expected);
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
            let instance = OptionalFixedPhoneNumber::try_from_str(expected).unwrap();
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
                OptionalFixedPhoneNumber::try_from(expected).is_err(),
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
            let instance = OptionalMobilePhoneNumber::try_from_str(expected).unwrap();
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
                OptionalMobilePhoneNumber::try_from_str(expected).is_err(),
                "`{}`",
                expected
            );
        }
    }
}
