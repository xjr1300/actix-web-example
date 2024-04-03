use secrecy::SecretString;

use domain::models::passwords::{generate_phc_string, RawPassword};
use domain::models::primitives::*;
use domain::models::user::{UserId, UserPermissionCode};
use domain::repositories::user::{SignUpUserBuilder, SignedUpUser, UserRepository};
use macros::Builder;

use crate::{ProcessUseCaseResult, UseCaseError};

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
    input: SignUpInput,
    pepper: &SecretString,
    repository: impl UserRepository,
) -> ProcessUseCaseResult<SignedUpUser> {
    let email =
        EmailAddress::new(input.email).map_err(|e| UseCaseError::validation(e.to_string()))?;
    let user_permission_code = UserPermissionCode::new(input.user_permission_code);
    let password =
        RawPassword::new(input.password).map_err(|e| UseCaseError::validation(e.to_string()))?;
    let password = generate_phc_string(&password, pepper)
        .map_err(|e| UseCaseError::unexpected(e.to_string()))?;
    let family_name =
        FamilyName::new(input.family_name).map_err(|e| UseCaseError::validation(e.to_string()))?;
    let given_name =
        GivenName::new(input.given_name).map_err(|e| UseCaseError::validation(e.to_string()))?;
    let postal_code =
        PostalCode::new(input.postal_code).map_err(|e| UseCaseError::validation(e.to_string()))?;
    let address =
        Address::new(input.address).map_err(|e| UseCaseError::validation(e.to_string()))?;
    let fixed_phone_number = OptionalFixedPhoneNumber::try_from(input.fixed_phone_number)
        .map_err(|e| UseCaseError::validation(e.to_string()))?;
    let mobile_phone_number = OptionalMobilePhoneNumber::try_from(input.mobile_phone_number)
        .map_err(|e| UseCaseError::validation(e.to_string()))?;
    let remarks = OptionalRemarks::try_from(input.remarks)
        .map_err(|e| UseCaseError::validation(e.to_string()))?;

    let user = SignUpUserBuilder::new()
        .id(UserId::default())
        .email(email)
        .password(password)
        .active(true)
        .user_permission_code(user_permission_code)
        .family_name(family_name)
        .given_name(given_name)
        .postal_code(postal_code)
        .address(address)
        .fixed_phone_number(fixed_phone_number)
        .mobile_phone_number(mobile_phone_number)
        .remarks(remarks)
        .build()
        .map_err(|e| UseCaseError::domain_rule(e.to_string()))?;

    // ユーザーを登録
    match repository.create(user).await {
        Ok(signed_up_user) => Ok(signed_up_user),
        Err(e) => {
            let message = e.to_string();
            match message.contains("ak_users_email") {
                true => Err(UseCaseError::domain_rule(
                    "同じEメール・アドレスを持つユーザーが、すでに登録されています。",
                )),
                false => Err(UseCaseError::unexpected(message)),
            }
        }
    }
}

/// サインアップ・リクエスト・ボディ
///
/// ```json
/// {"email": "foo@example.com", "password": "p@ssw0rd", "userPermissionCode": 1, "familyName": "Yamada", "givenName": "Taro", "postalCode": "899-7103", "address": "鹿児島県志布志市志布志町志布志2-1-1", "fixedPhoneNumber": "099-472-1111", "mobilePhoneNumber": "090-1234-5678", "remarks": "日本に実際に存在するややこしい地名です。"}
/// ```
#[derive(Debug, Clone, serde::Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct SignUpInput {
    /// Eメールアドレス
    pub email: String,
    /// 未加工なパスワード
    pub password: SecretString,
    /// ユーザー権限コード
    pub user_permission_code: i16,
    /// 苗字
    pub family_name: String,
    /// 名前
    pub given_name: String,
    /// 郵便番号
    pub postal_code: String,
    /// 住所
    pub address: String,
    /// 固定電話番号
    pub fixed_phone_number: Option<String>,
    /// 携帯電話番号
    pub mobile_phone_number: Option<String>,
    /// 備考
    pub remarks: Option<String>,
}
