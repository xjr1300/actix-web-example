use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use time::OffsetDateTime;
use uuid::Uuid;

use domain::repositories::user::SignedUpUser;
use use_cases::accounts::SignUpInput;
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
    request_body: web::Json<SignUpInput>,
) -> ProcessRequestResult<HttpResponse> {
    let repository = context.user_repository();
    let body = request_body.0;

    use_cases::accounts::sign_up(body, &context.pepper, repository)
        .await
        .map(|user| HttpResponse::Ok().json(SignUpResult::from(user)))
        .map_err(|e| e.into())
}

/// サインアップ結果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpResult {
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

impl From<SignedUpUser> for SignUpResult {
    fn from(value: SignedUpUser) -> Self {
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
