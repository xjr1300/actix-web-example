use std::collections::HashMap;
use std::str::FromStr as _;

use anyhow::anyhow;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use regex::Regex;
use secrecy::{ExposeSecret as _, SecretString};
use validator::Validate;

use configurations::settings::PasswordSettings;

use crate::{DomainError, DomainResult};

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

/// パスワードがドメイン・ルールを満たしているか確認する。
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

/// Argon2idアルゴリズムでパスワードをハッシュ化した、PHC文字列を生成する。
///
/// # 引数
///
/// * `raw_password` - 未加工なパスワード
/// * `pepper` - パスワードに付与するペッパー
///
/// # 戻り値
///
/// PHC文字列
pub fn generate_phc_string(
    raw_password: &RawPassword,
    settings: &PasswordSettings,
) -> DomainResult<PhcPassword> {
    // パスワードにペッパーを振りかけ
    let peppered_password = sprinkle_pepper_on_password(raw_password, &settings.pepper);
    // ソルトを生成
    let salt = SaltString::generate(&mut rand::thread_rng());
    // ハッシュ化パラメーターを設定
    let params = Params::new(
        settings.hash_memory,
        settings.hash_iterations,
        settings.hash_parallelism,
        None,
    )
    .map_err(|e| {
        DomainError::Unexpected(anyhow!(
            "Argon2パラメーターを構築しているときに、エラーが発生しました。{}",
            e
        ))
    })?;
    // PHC文字列を生成
    let phc = Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password(peppered_password.expose_secret().as_bytes(), &salt)
        .map_err(|e| {
            DomainError::Unexpected(anyhow!(
                "Argon2でハッシュ化してPHC文字列を生成するときに、エラーが発生しました。{}",
                e
            ))
        })?
        .to_string();

    Ok(PhcPassword {
        value: SecretString::new(phc),
    })
}

/// パスワードを検証する。
///
/// # 引数
///
/// * `raw_password` - 検証する未加工なパスワード
/// * `pepper` - 未加工なパスワードに振りかけるペッパー
/// * `target_phc` - パスワードを検証する対象のPHC文字列
///
/// # 戻り値
///
/// パスワードの検証に成功した場合は`true`、それ以外の場合は`false`
pub fn verify_password(
    raw_password: &RawPassword,
    pepper: &SecretString,
    target_phc: &PhcPassword,
) -> DomainResult<bool> {
    // PHC文字列をパースしてハッシュ値を取得
    let expected_hash = PasswordHash::new(target_phc.value.expose_secret()).map_err(|e| {
        DomainError::Unexpected(anyhow!(
            "PHC文字列からハッシュ・アルゴリズムを取得するときに、エラーが発生しました。{}",
            e
        ))
    })?;
    // パスワードにコショウを振りかけ
    let expected_password = sprinkle_pepper_on_password(raw_password, pepper);

    Ok(Argon2::default()
        .verify_password(expected_password.expose_secret().as_bytes(), &expected_hash)
        .is_ok())
}

/// パスワードにコショウを振りかける。
fn sprinkle_pepper_on_password(raw_password: &RawPassword, pepper: &SecretString) -> SecretString {
    let mut password = raw_password.value.expose_secret().to_string();
    password.push_str(pepper.expose_secret());

    SecretString::new(password)
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr as _;

    use configurations::settings::PasswordSettings;
    use secrecy::{ExposeSecret as _, SecretString};

    use crate::models::passwords::{generate_phc_string, verify_password, RawPassword};
    use crate::DomainError;

    /// 未加工なパスワードとして使用できる文字列
    pub const VALID_RAW_PASSWORD: &str = "Az3#Za3@";

    /// PHC文字列
    /// cspell: disable-next-line
    pub const RAW_PHC_PASSWORD: &str = "$argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno";

    /// 有効な文字列から、未加工なパスワードを構築できることを確認
    #[test]
    fn construct_raw_password_from_valid_string() {
        let secret = SecretString::from_str(VALID_RAW_PASSWORD).unwrap();
        let instance = RawPassword::new(secret).unwrap();
        assert_eq!(VALID_RAW_PASSWORD, instance.value.expose_secret());
    }

    /// 文字数が足りない文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_from_short_string() {
        let candidate = &VALID_RAW_PASSWORD[0..VALID_RAW_PASSWORD.len() - 1];
        let secret = SecretString::from_str(candidate).unwrap();
        let instance = RawPassword::new(secret);
        match instance {
            Ok(_) => panic!("password must not construct from short character"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from short character")
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
            Ok(_) => panic!("password must not construct from no uppercase alphabet character"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no uppercase alphabet character")
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
            Ok(_) => panic!("password must not construct from no lowercase alphabet character"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no lowercase alphabet character")
                }
            }
        }
    }

    /// 数字が含まれていない文字列から、未加工なパスワードを構築できないことを確認
    #[test]
    fn can_not_construct_raw_password_from_no_digit_string() {
        let candidate = VALID_RAW_PASSWORD.replace('3', "a");
        let secret = SecretString::from_str(&candidate).unwrap();
        let instance = RawPassword::new(secret);
        match instance {
            Ok(_) => panic!("password must not construct from no digit character"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no digit character")
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
            Ok(_) => panic!("password must not construct from no symbol character"),
            Err(err) => {
                match err {
                    DomainError::DomainRule(_) => {},
                    _ => panic!("DomainError::DomainRule should be returned when construct raw password from no symbol character")
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

    pub fn password_settings() -> PasswordSettings {
        PasswordSettings {
            pepper: SecretString::new(String::from("asdf")),
            hash_memory: 12288,
            hash_iterations: 3,
            hash_parallelism: 1,
        }
    }

    /// パスワードをハッシュ化したPHC文字列を生成した後、同じパスワードで検証に成功することを確認
    #[test]
    fn generate_a_phc_string_and_check_that_verification_is_successful_with_the_same_password() {
        // PHC文字列を生成
        let raw_password =
            RawPassword::new(SecretString::new(String::from(VALID_RAW_PASSWORD))).unwrap();
        let settings = password_settings();
        let phc_string = generate_phc_string(&raw_password, &settings).unwrap();
        // 同じパスワードで検証に成功するか確認
        assert!(verify_password(&raw_password, &settings.pepper, &phc_string).unwrap());
    }

    /// パスワードをハッシュ化したPHC文字列を生成した後、PHC文字列を生成したパスワードと異なるパスワードが検証に失敗することを確認
    #[test]
    fn generate_a_phc_string_and_check_that_verification_is_failure_with_different_passwords() {
        // PHC文字列を生成
        let raw_password =
            RawPassword::new(SecretString::new(String::from(VALID_RAW_PASSWORD))).unwrap();
        let settings = password_settings();
        let phc_string = generate_phc_string(&raw_password, &settings).unwrap();
        // 同じパスワードで検証に失敗するか確認
        let different_password = "fooBar123%";
        let different_password =
            RawPassword::new(SecretString::new(String::from(different_password))).unwrap();
        assert!(!verify_password(&different_password, &settings.pepper, &phc_string).unwrap());
    }
}
