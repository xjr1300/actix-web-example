use async_trait::async_trait;
use macros::Builder;
use time::OffsetDateTime;

use crate::models::primitives::*;
use crate::models::user::{User, UserId, UserPermissionCode, UserValidator};
use crate::DomainResult;

/// ユーザー・リポジトリ
#[async_trait]
pub trait UserRepository: Sync + Send {
    /// ユーザーのリストを取得する。
    ///
    /// # 戻り値
    ///
    /// ユーザーを格納したベクタ
    async fn list(&self) -> DomainResult<Vec<User>>;

    /// ユーザーを登録する。
    ///
    /// # 引数
    ///
    /// * `sign_up_user` - 登録するユーザー
    ///
    /// # 戻り値
    ///
    /// * 登録したユーザー
    async fn create(&self, user: SignUpInput) -> DomainResult<SignUpOutput>;
}

/// サイン・アップするユーザー
#[derive(Debug, Clone, Builder)]
#[builder_validation(func = "validate_user")]
pub struct SignUpInput {
    /// ユーザーID
    pub id: UserId,
    /// Eメールアドレス
    pub email: EmailAddress,
    /// パスワード
    pub password: PhcPassword,
    /// アクティブ・フラグ
    pub active: bool,
    /// ユーザー権限コード
    pub user_permission_code: UserPermissionCode,
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
}

impl UserValidator for SignUpInput {
    fn fixed_phone_number(&self) -> &OptionalFixedPhoneNumber {
        &self.fixed_phone_number
    }
    fn mobile_phone_number(&self) -> &OptionalMobilePhoneNumber {
        &self.mobile_phone_number
    }
}

pub struct SignUpOutput {
    /// ユーザーID
    pub id: UserId,
    /// Eメール・アドレス
    pub email: EmailAddress,
    /// アクティブ・フラグ
    pub active: bool,
    /// ユーザー権限コード
    pub user_permission_code: UserPermissionCode,
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
    /// 登録日時
    pub created_at: OffsetDateTime,
    /// 更新日時
    pub updated_at: OffsetDateTime,
}
