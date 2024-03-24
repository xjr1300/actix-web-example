use validator::Validate;

use macros::DomainPrimitive;

use crate::common::error::{DomainError, DomainResult};

/// Eメール・アドレス
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, DomainPrimitive)]
pub struct EmailAddress {
    #[validate(email)]
    #[value_getter(ret = "ref", rty = "&str")]
    value: String,
}

impl EmailAddress {
    /// Eメール・アドレスを構築する。
    ///
    /// # 引数
    ///
    /// * `email` - Eメール・アドレス
    ///
    /// # 戻り値
    ///
    /// Eメールアドレス
    pub fn new<T: ToString>(email: T) -> DomainResult<Self> {
        let instance = Self {
            value: email.to_string(),
        };
        match instance.validate() {
            Ok(_) => Ok(instance),
            Err(_) => Err(DomainError::Validation("email address is invalid".into())),
        }
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
