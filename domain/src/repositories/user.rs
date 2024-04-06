use async_trait::async_trait;
use macros::Builder;
use time::OffsetDateTime;

use crate::models::primitives::*;
use crate::models::user::{User, UserId, UserPermissionCode, UserValidator};
use crate::DomainResult;

/// ユーザーリポジトリ
#[async_trait]
pub trait UserRepository: Sync + Send {
    /// ユーザーのリストを取得する。
    ///
    /// # 戻り値
    ///
    /// ユーザーを格納したベクタ
    async fn list(&self) -> DomainResult<Vec<User>>;

    /// ユーザーのクレデンシャルを取得する。
    ///
    /// # 引数
    ///
    /// * `email` - ユーザーのEメールアドレス
    ///
    /// # 戻り値
    ///
    /// ユーザーのクレデンシャル
    async fn user_credential(&self, email: EmailAddress) -> DomainResult<Option<UserCredential>>;

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

/// サインアップするユーザー
#[derive(Debug, Clone, Builder)]
#[builder_validation(func = "validate_user")]
pub struct SignUpInput {
    /// ユーザーID
    pub id: UserId,
    /// Eメールアドレス
    pub email: EmailAddress,
    /// パスワード
    pub password: PhcPassword,
    /// アクティブフラグ
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

/// サインアップしたユーザー
pub struct SignUpOutput {
    /// ユーザーID
    pub id: UserId,
    /// Eメールアドレス
    pub email: EmailAddress,
    /// アクティブフラグ
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

/// ユーザークレデンシャル
pub struct UserCredential {
    /// ユーザーID
    pub user_id: UserId,
    /// Eメールアドレス
    pub email: EmailAddress,
    /// ユーザーのPHCパスワード文字列
    pub password: PhcPassword,
    /// アクティブフラグ
    pub active: bool,
    /// ユーザーが最初にサインインの試行に失敗した日時
    pub attempted_at: Option<OffsetDateTime>,
    /// ユーザーが最初にサインインの試行に失敗した日時から、サインインに失敗した回数
    pub number_of_failures: i16,
}
