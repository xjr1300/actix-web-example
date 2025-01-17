use enum_display::EnumDisplay;
use time::OffsetDateTime;

use macros::{Builder, PrimitiveDisplay, StringPrimitive};
use validator::Validate;

use crate::models::primitives::*;
use crate::{DomainError, DomainResult};

/// ユーザーID
pub type UserId = EntityId<User>;

/// ユーザー
///
/// # ドメインルール
///
/// ユーザーは、固定電話番号または携帯電話番号の両方またはどちらかを記録しなければならない。
///
/// # 認証
///
/// ユーザーは、Eメールアドレスとパスワードで認証する。
/// アクティブフラグが`true`のユーザーのみ認証できる。
/// アプリケーション設定の大認証繰り返し時間内に、最大認証繰り返し回数より多く認証に失敗したとき、
/// 認証に失敗させて、次回以降の認証をすべて拒否する。
///
/// # ユーザーの登録
///
/// ユーザーを登録するとき、PostgresSQLの場合、作成日時と更新日時に`STATEMENT_TIMESTAMP()`を使用して、
/// 同じ日時が記録されるようにする。
#[derive(Debug, Clone, Builder)]
#[builder_validation(func = "validate_user")]
pub struct User {
    /// ユーザーID
    pub id: UserId,
    /// Eメールアドレス
    pub email: EmailAddress,
    /// アクティブフラグ
    pub active: bool,
    /// ユーザー権限
    pub user_permission: UserPermission,
    /// 苗字
    pub family_name: FamilyName,
    /// 名前
    pub given_name: GivenName,
    /// 郵便番号
    pub postal_code: PostalCode,
    /// 住所
    pub address: Address,
    /// 固定電話番号
    pub fixed_phone_number: OptionalFixedPhoneNumber,
    /// 携帯電話番号
    pub mobile_phone_number: OptionalMobilePhoneNumber,
    /// 備考
    pub remarks: OptionalRemarks,
    /// 最終サインイン日時
    pub last_sign_in_at: Option<OffsetDateTime>,
    /// 最初にサインインを試行した日時
    pub sign_in_attempted_at: Option<OffsetDateTime>,
    /// サインインに失敗した回数
    pub number_of_sign_in_failures: NumberOfSignInFailures,
    /// 作成日時
    pub created_at: OffsetDateTime,
    /// 更新日時
    pub updated_at: OffsetDateTime,
}

pub trait UserValidator {
    fn fixed_phone_number(&self) -> &OptionalFixedPhoneNumber;
    fn mobile_phone_number(&self) -> &OptionalMobilePhoneNumber;
    fn validate_user(&self) -> DomainResult<()> {
        if self.fixed_phone_number().is_none() && self.mobile_phone_number().is_none() {
            return Err(DomainError::DomainRule(
                "ユーザーは固定電話番号または携帯電話番号を指定する必要があります。".into(),
            ));
        }

        Ok(())
    }
}

impl UserValidator for User {
    fn fixed_phone_number(&self) -> &OptionalFixedPhoneNumber {
        &self.fixed_phone_number
    }
    fn mobile_phone_number(&self) -> &OptionalMobilePhoneNumber {
        &self.mobile_phone_number
    }
}

/// ユーザー権限コード
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDisplay)]
#[enum_display(case = "Lower")]
pub enum UserPermissionCode {
    Admin = 1,
    General = 2,
}

impl TryFrom<i16> for UserPermissionCode {
    type Error = DomainError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(UserPermissionCode::Admin),
            2 => Ok(UserPermissionCode::General),
            _ => Err(DomainError::Validation(
                "ユーザー権限区分コードが範囲外です。".into(),
            )),
        }
    }
}

impl TryFrom<&str> for UserPermissionCode {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "admin" => Ok(UserPermissionCode::Admin),
            "general" => Ok(UserPermissionCode::General),
            _ => Err(DomainError::Validation(
                "ユーザー権限区分が範囲外です。".into(),
            )),
        }
    }
}

/// ユーザー権限
#[derive(Debug, Clone)]
pub struct UserPermission {
    /// ユーザー権限コード
    pub code: UserPermissionCode,
    /// ユーザー権限名
    pub name: UserPermissionName,
}

impl UserPermission {
    /// ユーザー権限を構築する。
    ///
    /// # 引数
    ///
    /// * `code` - ユーザー権限コード
    /// * `name` - ユーザー権限名
    ///
    /// # 戻り値
    ///
    /// ユーザー権限
    pub fn new(code: UserPermissionCode, name: UserPermissionName) -> Self {
        Self { code, name }
    }
}

/// ユーザー権限名
#[derive(Debug, Clone, Validate, PrimitiveDisplay, StringPrimitive)]
#[primitive(
    name = "ユーザー権限名",
    message = "ユーザー権限名は1文字以上20文字以下です。"
)]
pub struct UserPermissionName {
    #[validate(length(max = 20))]
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::now_jst;

    /// ユーザーを構築できることを確認
    #[test]
    fn user_can_build_with_builder() {
        let id = UserId::default();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let active = true;
        let user_permission = UserPermission::new(
            UserPermissionCode::Admin,
            UserPermissionName::new("管理者").unwrap(),
        );
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
                .active(active)
                .user_permission(user_permission.clone())
                .family_name(family_name.clone())
                .given_name(given_name.clone())
                .postal_code(postal_code.clone())
                .address(address.clone())
                .fixed_phone_number(fixed_phone_number.clone())
                .mobile_phone_number(mobile_phone_number.clone())
                .remarks(remarks.clone())
                .sign_in_attempted_at(None)
                .number_of_sign_in_failures(NumberOfSignInFailures::new(0).unwrap())
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
        let active = true;
        let user_permission = UserPermission::new(
            UserPermissionCode::Admin,
            UserPermissionName::new("管理者").unwrap(),
        );
        let family_name = FamilyName::new("foo").unwrap();
        let given_name = super::GivenName::new("bar").unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let address = Address::new("foo bar baz qux").unwrap();
        let dt = now_jst();
        let user = UserBuilder::new()
            .id(id)
            .email(email.clone())
            .active(active)
            .user_permission(user_permission)
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
