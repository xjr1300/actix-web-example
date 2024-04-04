use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use domain::models::passwords::{generate_phc_string, RawPassword};
use domain::models::user::{UserId, UserPermissionCode};
use secrecy::SecretString;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::primitives::*;
use domain::repositories::user::{SignUpInputBuilder, SingUpOutput};
use use_cases::UseCaseError;

use crate::routes::{ErrorResponseBody, ProcessRequestError, ProcessRequestResult};
use crate::RequestContext;

/// アカウントスコープを返却する。
pub fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts").service(web::resource("/sign-up").route(web::post().to(sign_up)))
}

/// サインアップ
pub async fn sign_up(
    context: web::Data<RequestContext>,
    request_body: web::Json<SignUpReqBody>,
) -> ProcessRequestResult<HttpResponse> {
    let repository = context.user_repository();
    let input = request_body.0;
    let pepper = &context.pepper;

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

    let input = SignUpInputBuilder::new()
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

    use_cases::accounts::sign_up(input, &context.pepper, repository)
        .await
        .map(|user| HttpResponse::Ok().json(SignUpResBody::from(user)))
        .map_err(|e| e.into())
}

/// サインアップ・リクエスト・ボディ
///
/// ```json
/// {"email": "foo@example.com", "password": "p@ssw0rd", "userPermissionCode": 1, "familyName": "Yamada", "givenName": "Taro", "postalCode": "899-7103", "address": "鹿児島県志布志市志布志町志布志2-1-1", "fixedPhoneNumber": "099-472-1111", "mobilePhoneNumber": "090-1234-5678", "remarks": "日本に実際に存在するややこしい地名です。"}
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpReqBody {
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

/// サインアップ・レスポンス・ボディ
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpResBody {
    /// ユーザーID
    pub id: Uuid,
    /// Eメール・アドレス
    pub email: String,
    /// 作成日時
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// 更新日時
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<SingUpOutput> for SignUpResBody {
    fn from(value: SingUpOutput) -> Self {
        Self {
            id: value.id.value,
            email: value.email.value,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<UseCaseError> for ProcessRequestError {
    fn from(value: UseCaseError) -> Self {
        let body = ErrorResponseBody {
            error_code: Some(value.error_code as u32),
            message: value.message,
        };
        match value.kind {
            use_cases::UseCaseErrorKind::Unexpected | use_cases::UseCaseErrorKind::Repository => {
                Self {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    body,
                }
            }
            use_cases::UseCaseErrorKind::Validation | use_cases::UseCaseErrorKind::DomainRule => {
                Self {
                    status_code: StatusCode::BAD_REQUEST,
                    body,
                }
            }
        }
    }
}
