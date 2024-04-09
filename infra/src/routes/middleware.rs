//! actix-webのミドルウェアは、次のように処理される。
//! <https://docs.rs/actix-web/latest/actix_web/middleware/index.html#ordering>
//!
//! ```text
//! #[get("/")]
//! async fn service(a: ExtractorA, b: ExtractorB) -> impl Responder { "Hello, World!" }
//!
//! let app = App::new()
//!     .wrap(MiddlewareA)
//!     .wrap(MiddlewareB)
//!     .wrap(MiddlewareC)
//!     .service(service);
//! ```
//! リクエストは、`MiddlewareC` -> `MiddlewareB` -> MiddlewareA`の順でミドルウェアに処理され、
//! `ExtractorA` -> `ExtractorB`の順でデータが取り出されたあと、`service`に処理される。
//!
//! `service`が生成したレスポンスは、`MiddlewareA` -> `MiddlewareB` -> `MiddlewareC`の順んで
//! ミドルウェに処理される。
//! ```
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::HttpMessage;
use deadpool_redis::Pool as RedisPool;
use domain::models::user::UserPermissionCode;
use secrecy::SecretString;

use domain::repositories::token::{TokenContent, TokenRepository, TokenType};

use crate::repositories::redis::token::RedisTokenRepository;
use crate::routes::{
    ErrorResponseBody, ProcessRequestError, ProcessRequestResult, ACCESS_TOKEN_KEY,
};

/// 認証ガードミドルウェア
///
/// リクエストヘッダのクッキーに設定されたアクセストークンを取得して、認証済みユーザーであるか
/// 確認するとともに、ユーザーIDをリクエストハンドラに渡す。
/// 認証済みユーザーでない場合は、`401 Unauthorized`で応答する。
pub struct AuthenticatedGuard;

impl<S> Transform<S, ServiceRequest> for AuthenticatedGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Transform = AuthenticatedGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticatedGuardMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticatedGuardMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthenticatedGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, service_req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        #[allow(clippy::redundant_closure)]
        Box::pin(async move {
            // リクエストヘッダからアクセストークンを取得してトークンコンテンツを取得
            let content = retrieve_token_content(&service_req).await?;

            // リクエストにユーザーIDとユーザー権限コードををデータとして追加
            service_req.extensions_mut().insert(content.user_id);
            service_req
                .extensions_mut()
                .insert(content.user_permission_code);

            // 後続のミドルウェアなどにリクエストの処理を移譲
            let future = service.call(service_req);

            // リクエストの処理が完了した後、リクエストの処理を移譲した先から返却されたフューチャーを、
            // レスポンスとして返却
            let resp = future.await?;

            Ok(resp)
        })
    }
}

/// 管理者ガードミドルウェア
///
/// 管理者のみにアクセスを許可する場合、ユーザー権限コードを管理者ガードミドルウェアに渡す必要があるため、
/// 次の順番でミドルウェアを登録する。
///
/// ```text
/// let app = App::new()
///     .wrap(AdminGuard)
///     .wrap(AuthenticatedGuard)
///     .service(admin_only_service);
/// ```
pub struct AdminGuard;

impl<S> Transform<S, ServiceRequest> for AdminGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Transform = AdminGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminGuardMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminGuardMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AdminGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut service_req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        #[allow(clippy::redundant_closure)]
        Box::pin(async move {
            let user_permission_code = service_req
                .extract::<web::ReqData<UserPermissionCode>>()
                .await
                .map_err(|_| forbidden_error())?;
            if user_permission_code.into_inner() != UserPermissionCode::Admin {
                return Err(forbidden_error());
            }

            // 後続のミドルウェアなどにリクエストの処理を移譲
            let future = service.call(service_req);

            // リクエストの処理が完了した後、リクエストの処理を移譲した先から返却されたフューチャーを、
            // レスポンスとして返却
            let resp = future.await?;

            Ok(resp)
        })
    }
}

fn forbidden_error() -> actix_web::Error {
    actix_web::Error::from(ProcessRequestError::without_error_code(
        StatusCode::FORBIDDEN,
        "アクセスする権限がありません。",
    ))
}

async fn retrieve_token_content(service_req: &ServiceRequest) -> actix_web::Result<TokenContent> {
    // リクエストヘッダのクッキーからアクセストークンを取得
    let token = access_token_from_cookie(service_req);
    if token.is_err() {
        return Err(actix_web::Error::from(
            ProcessRequestError::without_error_code(
                StatusCode::UNAUTHORIZED,
                "アクセストークンがリクエストヘッダに含まれていません。",
            ),
        ));
    }
    // Redisからアクセストークンをキーに保存されている値を解析
    let token = token.unwrap();
    let content = token_content_from_redis(service_req, &token).await;
    if content.is_err() {
        return Err(actix_web::Error::from(
            ProcessRequestError::without_error_code(
                StatusCode::BAD_REQUEST,
                "アクセストークンの内容を解析できません。",
            ),
        ));
    }
    // アクセストークンの内容を解析できたか確認
    let content = content.unwrap();
    if content.is_none() {
        return Err(actix_web::Error::from(
            ProcessRequestError::without_error_code(
                StatusCode::UNAUTHORIZED,
                "アクセストークンが無効です。",
            ),
        ));
    }
    // クッキーに保存されていたトークンがアクセストークンか確認
    let content = content.unwrap();
    if content.token_type != TokenType::Access {
        return Err(actix_web::Error::from(
            ProcessRequestError::without_error_code(
                StatusCode::BAD_REQUEST,
            "リクエストヘッダのクッキーに含まれているアクセストークンは、アクセストークンとして使用できません。"
            ),
        ));
    }

    Ok(content)
}

/// クッキーからアクセストークンを取得する。
fn access_token_from_cookie(service_req: &ServiceRequest) -> ProcessRequestResult<SecretString> {
    let token = service_req
        .headers()
        .get(ACCESS_TOKEN_KEY)
        .ok_or_else(|| ProcessRequestError {
            status_code: StatusCode::UNAUTHORIZED,
            body: ErrorResponseBody {
                error_code: None,
                message: "リクエストされたURIにアクセスする権限がありません。".into(),
            },
        })?;
    let token = token.to_str().map_err(|e| {
        tracing::error!("{} ({}:{})", e, file!(), line!());
        ProcessRequestError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                error_code: None,
                message: "クッキーに記録されたアクセストークンを取得できませんでした。".into(),
            },
        }
    })?;

    Ok(SecretString::new(token.into()))
}

async fn token_content_from_redis(
    service_req: &ServiceRequest,
    token: &SecretString,
) -> ProcessRequestResult<Option<TokenContent>> {
    let pool = service_req.app_data::<RedisPool>().ok_or_else(|| {
        tracing::error!(
            "can not retrieve the pool of redis ({}:{})",
            file!(),
            line!()
        );
        ProcessRequestError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                error_code: None,
                message: "Redis接続プールを取得できませんでした。".into(),
            },
        }
    })?;
    let repo = RedisTokenRepository::new(pool.clone());
    repo.retrieve_token_content(token).await.map_err(move |e| {
        tracing::error!("{} ({}:{})", e, file!(), line!());
        ProcessRequestError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                error_code: None,
                message: "Redis接続プールを取得できませんでした。".into(),
            },
        }
    })
}
