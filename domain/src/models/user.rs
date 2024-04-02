use time::OffsetDateTime;

use macros::{Builder, Getter};

use crate::models::passwords::PhcPassword;
use crate::models::primitives::*;
use crate::{DomainError, DomainResult};

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
    fixed_phone_number: OptionalFixedPhoneNumber,
    /// 携帯電話番号
    #[getter(ret = "ref")]
    mobile_phone_number: OptionalMobilePhoneNumber,
    /// 備考
    #[getter(ret = "ref")]
    remarks: OptionalRemarks,
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
            "ユーザーは固定電話番号または携帯電話番号を指定する必要があります。".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::*;
    use crate::models::passwords::tests::VALID_RAW_PASSWORD;
    use crate::models::passwords::{generate_phc_string, RawPassword};
    use crate::now_jst;

    /// ユーザーを構築できることを確認
    #[test]
    fn user_can_build_with_builder() {
        let id = UserId::default();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let plain_password = SecretString::new(String::from(VALID_RAW_PASSWORD));
        let raw_password = RawPassword::new(plain_password).unwrap();
        let password_pepper = SecretString::new(String::from("password-pepper"));
        let password = generate_phc_string(&raw_password, &password_pepper).unwrap();
        let active = true;
        let family_name = FamilyName::new("foo").unwrap();
        let given_name = super::GivenName::new("bar").unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let address = Address::new("foo bar baz qux").unwrap();
        let phone_number_pairs = [
            (
                OptionalFixedPhoneNumber::try_from("03-1234-5678").unwrap(),
                OptionalMobilePhoneNumber::try_from("090-1234-5678").unwrap(),
            ),
            (
                OptionalFixedPhoneNumber::try_from("03-1234-5678").unwrap(),
                OptionalMobilePhoneNumber::none(),
            ),
            (
                OptionalFixedPhoneNumber::none(),
                OptionalMobilePhoneNumber::try_from("090-1234-5678").unwrap(),
            ),
        ];
        let remarks = OptionalRemarks::try_from("remarks").unwrap();
        let dt = now_jst();
        for (fixed_phone_number, mobile_phone_number) in phone_number_pairs {
            let user = UserBuilder::new()
                .id(id)
                .email(email.clone())
                .password(password.clone())
                .active(active)
                .family_name(family_name.clone())
                .given_name(given_name.clone())
                .postal_code(postal_code.clone())
                .address(address.clone())
                .fixed_phone_number(fixed_phone_number.clone())
                .mobile_phone_number(mobile_phone_number.clone())
                .remarks(remarks.clone())
                .created_at(dt)
                .updated_at(dt)
                .build();

            assert!(
                user.is_ok(),
                "{:?}, {:?}",
                fixed_phone_number,
                mobile_phone_number
            );
        }
    }

    /// 固定電話番号と携帯電話番号の両方とも指定していない場合に、ユーザーを構築できないことを確認
    #[test]
    fn user_can_not_build_without_fixed_phone_number_and_mobile_phone_number() {
        let id = UserId::default();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let plain_password = SecretString::new(String::from(VALID_RAW_PASSWORD));
        let raw_password = RawPassword::new(plain_password).unwrap();
        let password_pepper = SecretString::new(String::from("password-pepper"));
        let password = generate_phc_string(&raw_password, &password_pepper).unwrap();
        let active = true;
        let family_name = FamilyName::new("foo").unwrap();
        let given_name = super::GivenName::new("bar").unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let address = Address::new("foo bar baz qux").unwrap();
        let dt = now_jst();
        let user = UserBuilder::new()
            .id(id)
            .email(email.clone())
            .password(password.clone())
            .active(active)
            .family_name(family_name.clone())
            .given_name(given_name.clone())
            .postal_code(postal_code.clone())
            .address(address.clone())
            .fixed_phone_number(OptionalFixedPhoneNumber::none())
            .mobile_phone_number(OptionalMobilePhoneNumber::none())
            .remarks(OptionalRemarks::none())
            .created_at(dt)
            .updated_at(dt)
            .build();
        assert!(user.is_err());
    }
}
