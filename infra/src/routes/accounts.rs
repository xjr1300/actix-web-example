use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use domain::models::passwords::RawPassword;
use secrecy::SecretString;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::primitives::*;
use domain::models::user::User;
use use_cases::accounts::{SignupUser, SignupUserBuilder};
use use_cases::UseCaseError;

use crate::routes::common::{ErrorResponseBody, ProcessRequestError, ProcessRequestResult};
use crate::RequestContext;

/// アカウントスコープを返却する。
pub fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts").service(web::resource("/signup").route(web::post().to(signup)))
}

/// サインアップ
pub async fn signup(
    context: web::Data<RequestContext>,
    request_body: web::Json<SignupRequestBody>,
) -> ProcessRequestResult<HttpResponse> {
    let repository = context.user_repository();
    let signup_user = SignupUser::try_from(request_body.0).map_err(ProcessRequestError::from)?;
    use_cases::accounts::signup(signup_user, &context.pepper, repository)
        .await
        .map(|user| HttpResponse::Ok().json(SignupResponseBody::from(user)))
        .map_err(|e| e.into())
}

/// サインアップ・リクエスト・ボディ
///
/// ```json
/// {"email": "foo@example.com", "password": "p@ssw0rd", "familyName": "Yamada", "givenName": "Taro", "postalCode": "899-7103", "address": "鹿児島県志布志市志布志町志布志2-1-1", "fixedPhoneNumber": "099-472-1111", "mobilePhoneNumber": "090-1234-5678", "remarks": "日本に実際に存在するややこしい地名です。"}
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequestBody {
    /// Eメールアドレス
    email: String,
    /// 未加工なパスワード
    password: SecretString,
    /// 苗字
    family_name: String,
    /// 名前
    given_name: String,
    /// 郵便番号
    postal_code: String,
    /// 住所
    address: String,
    /// 固定電話番号
    fixed_phone_number: Option<String>,
    /// 携帯電話番号
    mobile_phone_number: Option<String>,
    /// 備考
    remarks: Option<String>,
}

/// サインアップ・レスポンス・ボディ
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupResponseBody {
    /// ユーザーID
    id: Uuid,
    /// Eメール・アドレス
    email: String,
    /// 作成日時
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    /// 更新日時
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

/// サインアップ・リクエスト・ボディの内容を、ユース・ケース層で扱うDTOに変換する。
impl TryFrom<SignupRequestBody> for SignupUser {
    type Error = ProcessRequestError;

    fn try_from(value: SignupRequestBody) -> Result<Self, Self::Error> {
        let email = EmailAddress::new(value.email)?;
        let password = RawPassword::new(value.password)?;
        let family_name = FamilyName::new(value.family_name)?;
        let given_name = GivenName::new(value.given_name)?;
        let postal_code = PostalCode::new(value.postal_code)?;
        let address = Address::new(value.address)?;
        let fixed_phone_number = value.fixed_phone_number.try_into()?;
        let mobile_phone_number = value.mobile_phone_number.try_into()?;
        let remarks = value.remarks.try_into()?;

        let mut builder = SignupUserBuilder::new();
        builder
            .email(email)
            .password(password)
            .family_name(family_name)
            .given_name(given_name)
            .postal_code(postal_code)
            .address(address)
            .fixed_phone_number(fixed_phone_number)
            .mobile_phone_number(mobile_phone_number)
            .remarks(remarks);

        Ok(builder.build().unwrap())
    }
}

impl From<User> for SignupResponseBody {
    fn from(value: User) -> Self {
        Self {
            id: value.id().value(),
            email: value.email().value().to_string(),
            created_at: value.created_at(),
            updated_at: value.updated_at(),
        }
    }
}

impl From<UseCaseError> for ProcessRequestError {
    fn from(value: UseCaseError) -> Self {
        let body = ErrorResponseBody {
            error_code: Some(value.error_code),
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
