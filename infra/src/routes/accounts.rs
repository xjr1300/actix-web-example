use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use secrecy::SecretString;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::passwords::{generate_phc_string, RawPassword};
use domain::models::primitives::*;
use domain::models::user::{User, UserId, UserPermissionCode};
use domain::repositories::user::{SignUpInputBuilder, SingUpOutput};
use domain::DomainError;
use use_cases::UseCaseError;

use crate::routes::{ErrorResponseBody, ProcessRequestError, ProcessRequestResult};
use crate::RequestContext;

/// アカウントスコープを返却する。
pub fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts")
        .service(web::resource("/sign-up").route(web::post().to(sign_up)))
        .service(web::resource("/users").route(web::get().to(list_users)))
}

/// サインアップ
pub async fn sign_up(
    context: web::Data<RequestContext>,
    request_body: web::Json<SignUpReqBody>,
) -> ProcessRequestResult<HttpResponse> {
    let repository = context.user_repository();
    let input = request_body.0;
    let password_settings = &context.password_settings;

    let email = EmailAddress::new(input.email).map_err(ProcessRequestError::from)?;
    let user_permission_code = UserPermissionCode::new(input.user_permission_code);
    let password = RawPassword::new(input.password).map_err(ProcessRequestError::from)?;
    let password =
        generate_phc_string(&password, password_settings).map_err(ProcessRequestError::from)?;
    let family_name = FamilyName::new(input.family_name).map_err(ProcessRequestError::from)?;
    let given_name = GivenName::new(input.given_name).map_err(ProcessRequestError::from)?;
    let postal_code = PostalCode::new(input.postal_code).map_err(ProcessRequestError::from)?;
    let address = Address::new(input.address).map_err(ProcessRequestError::from)?;
    let fixed_phone_number = OptionalFixedPhoneNumber::try_from(input.fixed_phone_number)
        .map_err(ProcessRequestError::from)?;
    let mobile_phone_number = OptionalMobilePhoneNumber::try_from(input.mobile_phone_number)
        .map_err(ProcessRequestError::from)?;
    let remarks = OptionalRemarks::try_from(input.remarks).map_err(ProcessRequestError::from)?;

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

    use_cases::accounts::sign_up(input, repository)
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

/// ユーザー・リスト
async fn list_users(context: web::Data<RequestContext>) -> ProcessRequestResult<HttpResponse> {
    let repo = context.user_repository();
    let users = use_cases::accounts::list_users(repo)
        .await?
        .into_iter()
        .map(UserResBody::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(users))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserResBody {
    pub id: Uuid,
    pub email: String,
    pub active: bool,
    pub user_permission: UserPermissionBody,
    pub family_name: String,
    pub given_name: String,
    pub postal_code: String,
    pub address: String,
    pub fixed_phone_number: Option<String>,
    pub mobile_phone_number: Option<String>,
    pub remarks: Option<String>,
    pub last_logged_in_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPermissionBody {
    pub code: i16,
    pub name: String,
}

impl From<User> for UserResBody {
    fn from(value: User) -> Self {
        Self {
            id: value.id.value,
            email: value.email.value,
            active: value.active,
            user_permission: UserPermissionBody {
                code: value.user_permission.code.value,
                name: value.user_permission.name.value,
            },
            family_name: value.family_name.value,
            given_name: value.given_name.value,
            postal_code: value.postal_code.value,
            address: value.address.value,
            fixed_phone_number: value.fixed_phone_number.owned_value(),
            mobile_phone_number: value.mobile_phone_number.owned_value(),
            remarks: value.remarks.owned_value(),
            last_logged_in_at: value.last_logged_in_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<DomainError> for ProcessRequestError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::Unexpected(e) => UseCaseError::unexpected(e.to_string()).into(),
            DomainError::Validation(m) => UseCaseError::validation(m).into(),
            DomainError::DomainRule(m) => UseCaseError::domain_rule(m).into(),
            DomainError::Repository(e) => UseCaseError::repository(e.to_string()).into(),
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
