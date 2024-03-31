use secrecy::SecretString;

use domain::models::primitives::*;
use macros::{Builder, Getter};

#[derive(Debug, Clone, Getter, Builder)]
pub struct SignupUser {
    /// Eメールアドレス
    #[getter(ret = "ref")]
    email: EmailAddress,
    /// 未加工なパスワード
    #[getter(ret = "ref")]
    password: SecretString,
    /// 苗字
    #[getter(ret = "ref")]
    family_name: FamilyName,
    /// 名前
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
    fixed_phone_number: FixedPhoneNumber,
    /// 携帯電話番号
    #[getter(ret = "ref")]
    mobile_phone_number: MobilePhoneNumber,
    /// 備考
    #[getter(ret = "ref")]
    remarks: Remarks,
}
