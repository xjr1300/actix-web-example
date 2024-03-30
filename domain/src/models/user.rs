use time::OffsetDateTime;

use macros::{Builder, Getter};

use crate::common::{now_jst, DomainError, DomainResult};
use crate::models::passwords::PhcPassword;
use crate::models::primitives::*;

/// ユーザーID
pub type UserId = EntityId<User>;

/// ユーザー
///
/// # ドメイン・ルール
///
/// ユーザーは、固定電話番号または携帯電話番号の両方またはどちらかを記録しなければならない。
///
/// # 認証
///
/// ユーザーは、Eメール・アドレスとパスワードで認証する。
/// アクティブ・フラグが`true`のユーザーのみ認証できる。
/// アプリケーション設定の大認証繰り返し時間内に、最大認証繰り返し回数より多く認証に失敗したとき、
/// 認証に失敗させて、次回以降の認証をすべて拒否する。
///
/// # ユーザーの登録
///
/// ユーザーを登録するとき、PostgresSQLの場合、作成日時と更新日時に`STATEMENT_TIMESTAMP()`を使用して、
/// 同じ日時が記録されるようにする。
#[derive(Debug, Clone, Getter, Builder)]
#[builder_validation(func = "validate_user")]
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
    fixed_phone_number: Option<FixedPhoneNumber>,
    /// 携帯電話番号
    #[getter(ret = "ref")]
    mobile_phone_number: Option<MobilePhoneNumber>,
    /// 備考
    #[getter(ret = "ref")]
    remarks: Option<Remarks>,
    /// 最終ログイン日時
    #[getter(ret = "val")]
    last_logged_in_at: Option<OffsetDateTime>,
    /// 作成日時
    #[getter(ret = "val")]
    created_at: OffsetDateTime,
    /// 更新日時
    #[getter(ret = "val")]
    updated_at: OffsetDateTime,
}

fn validate_user(user: &User) -> DomainResult<()> {
    if user.fixed_phone_number.is_none() && user.mobile_phone_number.is_none() {
        return Err(DomainError::DomainRule(
            "must provide at least a fixed phone number or mobile phone number".into(),
        ));
    }

    Ok(())
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
        let dt = now_jst();

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
            last_logged_in_at: None,
            created_at: dt,
            updated_at: dt,
        })
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::*;
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
}
