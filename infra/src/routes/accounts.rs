use actix_web::cookie::Cookie;
use actix_web::{web, HttpResponse};
use secrecy::{ExposeSecret, SecretString};
use time::OffsetDateTime;
use uuid::Uuid;

use configurations::settings::HttpServerSettings;
use domain::models::primitives::*;
use domain::models::user::{User, UserPermissionCode};
use use_cases::accounts::{
    SignInUseCaseInput, SignInUseCaseOutput, SignUpUseCaseInputBuilder, SignUpUseCaseOutput,
};
use use_cases::UseCaseError;

use crate::routes::extractors::{AdminContext, UserOwnContext};
use crate::routes::{
    ProcessRequestError, ProcessRequestResult, ACCESS_TOKEN_KEY, REFRESH_TOKEN_KEY,
};
use crate::RequestContext;

/// アカウントスコープを返却する。
pub fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts")
        .service(web::resource("/sign-up").route(web::post().to(sign_up)))
        .service(web::resource("/sign-in").route(web::post().to(sign_in)))
        .service(
            web::scope("/users")
                .service(web::resource("").route(web::get().to(list_users)))
                .service(
                    web::scope("/{user_id}")
                        .service(web::resource("").route(web::get().to(user_detail))),
                ),
        )
}

/// サインアップ
pub async fn sign_up(
    context: web::Data<RequestContext>,
    request_body: web::Json<SignUpReqBody>,
) -> ProcessRequestResult<HttpResponse> {
    let password_settings = &context.password_settings;
    let user_repository = context.user_repository();
    let input = request_body.0;

    let email = EmailAddress::new(input.email).map_err(ProcessRequestError::from)?;
    let user_permission_code = UserPermissionCode::try_from(input.user_permission_code)
        .map_err(ProcessRequestError::from)?;
    let password = RawPassword::new(input.password).map_err(ProcessRequestError::from)?;
    let family_name = FamilyName::new(input.family_name).map_err(ProcessRequestError::from)?;
    let given_name = GivenName::new(input.given_name).map_err(ProcessRequestError::from)?;
    let postal_code = PostalCode::new(input.postal_code).map_err(ProcessRequestError::from)?;
    let address = Address::new(input.address).map_err(ProcessRequestError::from)?;
    let fixed_phone_number = OptionalFixedPhoneNumber::try_from(input.fixed_phone_number)
        .map_err(ProcessRequestError::from)?;
    let mobile_phone_number = OptionalMobilePhoneNumber::try_from(input.mobile_phone_number)
        .map_err(ProcessRequestError::from)?;
    let remarks = OptionalRemarks::try_from(input.remarks).map_err(ProcessRequestError::from)?;

    let input = SignUpUseCaseInputBuilder::new()
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

    use_cases::accounts::sign_up(password_settings, user_repository, input)
        .await
        .map(|user| HttpResponse::Ok().json(SignUpResBody::from(user)))
        .map_err(|e| e.into())
}

/// サインアップリクエスト・ボディ
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

/// サインアップレスポンス・ボディ
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpResBody {
    /// ユーザーID
    pub id: Uuid,
    /// Eメールアドレス
    pub email: String,
    /// アクティブフラグ
    pub active: bool,
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
    /// 作成日時
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// 更新日時
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<SignUpUseCaseOutput> for SignUpResBody {
    fn from(value: SignUpUseCaseOutput) -> Self {
        Self {
            id: value.id.value,
            email: value.email.value,
            active: value.active,
            user_permission_code: value.user_permission_code as i16,
            family_name: value.family_name.value,
            given_name: value.given_name.value,
            postal_code: value.postal_code.value,
            address: value.address.value,
            fixed_phone_number: value.fixed_phone_number.owned_value(),
            mobile_phone_number: value.mobile_phone_number.owned_value(),
            remarks: value.remarks.owned_value(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// サインイン
pub async fn sign_in(
    context: web::Data<RequestContext>,
    request_body: web::Json<SignInReqBody>,
) -> ProcessRequestResult<HttpResponse> {
    let http_server_settings = &context.http_server_settings;
    let password_settings = &context.password_settings;
    let authorization_settings = &context.authorization_settings;
    let user_repository = context.user_repository();
    let token_repository = context.token_repository();
    let email = EmailAddress::new(request_body.0.email).map_err(ProcessRequestError::from)?;
    let password = RawPassword::new(request_body.0.password).map_err(ProcessRequestError::from)?;
    let input = SignInUseCaseInput { email, password };

    let output = use_cases::accounts::sign_in(
        password_settings,
        authorization_settings,
        user_repository,
        token_repository,
        input,
    )
    .await
    .map_err(ProcessRequestError::from)?;

    // レスポンスヘッダに、クッキーにアクセス及びリクエストトークンを設定する`Set-Cookie`を追加する。
    let access_cookie = generate_token_cookie(
        ACCESS_TOKEN_KEY,
        &output.access,
        output.access_expiration,
        http_server_settings,
    );
    let refresh_cookie = generate_token_cookie(
        REFRESH_TOKEN_KEY,
        &output.access,
        output.refresh_expiration,
        http_server_settings,
    );
    // レスポンスボディを構築
    let body = SignInResBody::from(&output);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(body))
}

fn generate_token_cookie<'a>(
    name: &'a str,
    token: &'a SecretString,
    expiration: OffsetDateTime,
    http_settings: &HttpServerSettings,
) -> Cookie<'a> {
    Cookie::build(name, token.expose_secret())
        .same_site(http_settings.same_site)
        .secure(http_settings.secure)
        .http_only(true)
        .expires(expiration)
        .finish()
}

/// サインインリクエスト・ボディ
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SignInReqBody {
    /// Eメールアドレス
    pub email: String,
    /// パス話ワード
    pub password: SecretString,
}

/// JWTトークンペア・レスポンス・ボディ
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignInResBody {
    /// アクセストークン
    pub access: String,
    /// リフレッシュトークン
    pub refresh: String,
}

impl From<&SignInUseCaseOutput> for SignInResBody {
    fn from(value: &SignInUseCaseOutput) -> Self {
        Self {
            access: value.access.expose_secret().to_string(),
            refresh: value.refresh.expose_secret().to_string(),
        }
    }
}

/// ユーザーリスト
async fn list_users(
    request_context: web::Data<RequestContext>,
    _admin_context: AdminContext,
) -> ProcessRequestResult<HttpResponse> {
    let repo = request_context.user_repository();
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
                code: value.user_permission.code as i16,
                name: value.user_permission.name.value,
            },
            family_name: value.family_name.value,
            given_name: value.given_name.value,
            postal_code: value.postal_code.value,
            address: value.address.value,
            fixed_phone_number: value.fixed_phone_number.owned_value(),
            mobile_phone_number: value.mobile_phone_number.owned_value(),
            remarks: value.remarks.owned_value(),
            last_logged_in_at: value.last_sign_in_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

async fn user_detail(
    _request_context: web::Data<RequestContext>,
    user_own_context: UserOwnContext,
) -> String {
    format!("user_id: {}", user_own_context.user_id,)
}
