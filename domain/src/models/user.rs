use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use secrecy::{ExposeSecret, SecretString};
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
#[derive(Debug, Clone, Validate, DomainPrimitive)]
pub struct RawPassword {
    #[value_getter(ret = "ref")]
    value: SecretString,
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
const PASSWORD_MAX_NUMBER_OF_SAME_CHAR: u64 = 3;

/// パスワードがドメイン・ルールを満たしているか確認する。
fn validate_plain_password(s: &str) -> DomainResult<()> {
    // パスワードの文字数を確認
    if s.len() < PASSWORD_MIN_LENGTH {
        return Err(DomainError::DomainRule(
            format!("password must be at least {PASSWORD_MIN_LENGTH} characters").into(),
        ));
    }
    // 大文字のアルファベットが含まれるか確認
    if !s.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(DomainError::DomainRule(
            "password must contain at least one uppercase alphabetic character".into(),
        ));
    }
    // 小文字のアルファベットが含まれるか確認
    if !s.chars().any(|ch| ch.is_ascii_lowercase()) {
        return Err(DomainError::DomainRule(
            "password must contain at least one lowercase alphabetic character".into(),
        ));
    }
    // シンボルが1つ以上含まれるか確認
    if !s.chars().any(|ch| PASSWORD_SYMBOLS_CANDIDATES.contains(ch)) {
        return Err(DomainError::DomainRule(
            "password must contain at least one symbol character".into(),
        ));
    }
    // 文字の出現回数を確認
    let mut number_of_chars: HashMap<char, u64> = HashMap::new();
    s.chars().for_each(|ch| {
        *number_of_chars.entry(ch).or_insert(0) += 1;
    });
    let max_number_of_same_char = number_of_chars.values().max().unwrap();
    if PASSWORD_MAX_NUMBER_OF_SAME_CHAR < *max_number_of_same_char {
        return Err(DomainError::DomainRule(
            format!("password cannot contain more than {PASSWORD_MAX_NUMBER_OF_SAME_CHAR} of the same character").into()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use secrecy::{ExposeSecret, SecretString};

    use super::{EmailAddress, RawPassword};
    use crate::common::error::DomainError;

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
        assert!(EmailAddress::new("invalid-email-address").is_err())
    }

    /// 未加工なパスワードとして使用できる文字列
    const VALID_RAW_PASSWORD: &str = "Az3#Za3@";

    /// 有効な文字列から、未加工なパスワードを構築できることを確認
    #[test]
    fn construct_raw_password_from_valid_string() {
        let secret = SecretString::from_str(VALID_RAW_PASSWORD).unwrap();
        let instance = RawPassword::new(secret).unwrap();
        assert_eq!(VALID_RAW_PASSWORD, instance.value().expose_secret());
    }

    /// 文字数が足りない文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_from_short_string() {
        let candidate = &VALID_RAW_PASSWORD[0..VALID_RAW_PASSWORD.len() - 1];
        let secret = SecretString::from_str(candidate).unwrap();
        let instance = RawPassword::new(secret);
        match instance {
            Ok(_) => panic!("password must not construct from short string"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from short string")
                }
            }
        }
    }

    /// 大文字のアルファベットが含まれていない文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_from_no_uppercase_alphabet_string() {
        let candidate = VALID_RAW_PASSWORD.to_ascii_lowercase();
        let secret = SecretString::from_str(&candidate).unwrap();
        let instance = RawPassword::new(secret);
        match instance {
            Ok(_) => panic!("password must not construct from no uppercase alphabet string"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no uppercase alphabet string")
                }
            }
        }
    }

    /// 小文字のアルファベットが含まれていない文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_from_no_lowercase_alphabet_string() {
        let candidate = VALID_RAW_PASSWORD.to_ascii_lowercase();
        let secret = SecretString::from_str(&candidate).unwrap();
        let instance = RawPassword::new(secret);
        match instance {
            Ok(_) => panic!("password must not construct from no lowercase alphabet string"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no lowercase alphabet string")
                }
            }
        }
    }

    /// 記号が含まれていない文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_from_no_symbol_character_string() {
        let candidate = VALID_RAW_PASSWORD.replace('#', "Q").replace('@', "q");
        let secret = SecretString::from_str(&candidate).unwrap();
        let instance = RawPassword::new(secret);
        match instance {
            Ok(_) => panic!("password must not construct from no symbol character string"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no symbol character string")
                }
            }
        }
    }

    /// 同じ文字が指定した数より多く含まれている文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_containing_same_character_more_than_specified_times() {
        // 最初の要素のパスワードは許容して、2つ目の要素のパスワードを拒否(cspell: disable-next-line)
        let candidates = [("Aa1#zaab", true), ("Aa1#zaaa", false)];
        for (candidate, result) in candidates {
            let secret = SecretString::from_str(candidate).unwrap();
            let instance = RawPassword::new(secret);
            if result && instance.is_err() {
                panic!("raw password should be constructed when containing the same character less equal specified times");
            }
            if !result && instance.is_ok() {
                panic!("raw password must not be constructed when containing the same character more than specified times");
            }
            if instance.is_err() {
                match instance.err().unwrap() {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when containing the same character more than specified times")
                }
            }
        }
    }
}
