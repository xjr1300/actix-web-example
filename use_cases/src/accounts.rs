use time::OffsetDateTime;

use domain::models::primitives::*;
use domain::models::user::{User, UserId, UserPermissionCode};
use domain::repositories::user::{SignUpInputBuilder, SignUpOutput, UserRepository};
use macros::Builder;

use crate::passwords::{generate_phc_string, PasswordSettings};
use crate::{ProcessUseCaseResult, UseCaseError};

/// サイン・アップ・ユース・ケース入力
#[derive(Debug, Clone, Builder)]
pub struct SignUpUseCaseInput {
    /// Eメールアドレス
    pub email: EmailAddress,
    /// パスワード
    pub password: RawPassword,
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

/// サイン・アップ・ユース・ケース出力
pub struct SignUpUseCaseOutput {
    /// ユーザーID
    pub id: UserId,
    /// Eメールアドレス
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
    /// 作成日時
    pub created_at: OffsetDateTime,
    /// 更新日時
    pub updated_at: OffsetDateTime,
}

impl From<SignUpOutput> for SignUpUseCaseOutput {
    fn from(value: SignUpOutput) -> Self {
        Self {
            id: value.id,
            email: value.email,
            active: value.active,
            user_permission_code: value.user_permission_code,
            family_name: value.family_name,
            given_name: value.given_name,
            postal_code: value.postal_code,
            address: value.address,
            fixed_phone_number: value.fixed_phone_number,
            mobile_phone_number: value.mobile_phone_number,
            remarks: value.remarks,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// ユーザーを登録する。
///
/// # 引数
///
/// * `user` - 登録するユーザー
/// * `pepper` - 未加工なパスワードに付与するペッパー
/// * `repository` - ユーザー・リポジトリ
///
/// # 戻り値
///
/// * 登録したユーザー
#[tracing::instrument(
    name = "sign up use case", skip(input, repository),
    fields(user.email = %input.email)
)]
pub async fn sign_up(
    settings: &PasswordSettings,
    repository: impl UserRepository,
    input: SignUpUseCaseInput,
) -> ProcessUseCaseResult<SignUpUseCaseOutput> {
    let id = UserId::default();
    let password = generate_phc_string(&input.password, settings).map_err(UseCaseError::from)?;

    let input = SignUpInputBuilder::new()
        .id(id)
        .email(input.email)
        .password(password)
        .active(input.active)
        .user_permission_code(input.user_permission_code)
        .family_name(input.family_name)
        .given_name(input.given_name)
        .postal_code(input.postal_code)
        .address(input.address)
        .fixed_phone_number(input.fixed_phone_number)
        .mobile_phone_number(input.mobile_phone_number)
        .remarks(input.remarks)
        .build()
        .map_err(|e| UseCaseError::domain_rule(e.to_string()))?;

    // ユーザーを登録
    match repository.create(input).await {
        Ok(inserted_user) => Ok(inserted_user.into()),
        Err(e) => {
            let message = e.to_string();
            if message.contains("ak_users_email") {
                Err(UseCaseError::domain_rule(
                    "同じEメール・アドレスを持つユーザーが、すでに登録されています。",
                ))
            } else if message.contains("fk_users_permission") {
                Err(UseCaseError::validation(
                    "ユーザー権限区分コードが範囲外です。",
                ))
            } else if message.contains("ck_users_either_phone_numbers_must_be_not_null") {
                // インフラストラクチャ層で検証されるため、実際にはここは実行されない
                Err(UseCaseError::domain_rule(
                    "固定電話番号または携帯電話番号を指定する必要があります。",
                ))
            } else {
                Err(UseCaseError::repository(message))
            }
        }
    }
}

/// ユーザーのリストを取得する。
///
/// # 引数
///
/// * `repository` - ユーザー・リポジトリ
///
/// # 戻り値
///
/// * ユーザーを格納したベクタ
#[tracing::instrument(name = "list users use case", skip(repository))]
pub async fn list_users(repository: impl UserRepository) -> ProcessUseCaseResult<Vec<User>> {
    repository
        .list()
        .await
        .map_err(|e| UseCaseError::repository(e.to_string()))
}
