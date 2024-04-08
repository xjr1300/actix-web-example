use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::{HttpMessage, ResponseError};
use deadpool_redis::Pool as RedisPool;
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
pub struct AuthenticatedUser;

impl<S> Transform<S, ServiceRequest> for AuthenticatedUser
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = ServiceResponse> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = ServiceResponse;
    type Transform = AuthenticatedUserMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticatedUserMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticatedUserMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthenticatedUserMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = ServiceResponse> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = ServiceResponse;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, service_req: ServiceRequest) -> Self::Future {
        tracing::info!("AuthenticatedUserMiddlewareの処理を開始");

        let service = Rc::clone(&self.service);

        #[allow(clippy::redundant_closure)]
        Box::pin(async move {
            // リクエストヘッダのクッキーからアクセストークンを取得
            let token = access_token_from_cookie(&service_req);
            if token.is_err() {
                let (request, _) = service_req.into_parts();
                return Err(ServiceResponse::new(
                    request,
                    token.err().unwrap().error_response(),
                ));
            }
            // Redisからアクセストークンの内容を取得
            let token = token.unwrap();
            let content = token_content_from_redis(&service_req, &token).await;
            if content.is_err() {
                let (request, _) = service_req.into_parts();
                return Err(ServiceResponse::new(
                    request,
                    content.err().unwrap().error_response(),
                ));
            }
            // アクセストークンの内容を取得できたか確認
            let content = content.unwrap();
            if content.is_none() {
                let e = ProcessRequestError {
                    status_code: StatusCode::UNAUTHORIZED,
                    body: ErrorResponseBody {
                        error_code: None,
                        message: "リクエストされたURIにアクセスする権限がありません。".into(),
                    },
                };
                let (request, _) = service_req.into_parts();
                return Err(ServiceResponse::new(request, e.error_response()));
            }
            // クッキーに保存されていたトークンがアクセストークンか確認
            let content = content.unwrap();
            if content.token_type != TokenType::Access {
                let e = ProcessRequestError {
                    status_code: StatusCode::BAD_REQUEST,
                    body: ErrorResponseBody {
                        error_code: None,
                        message: "トークンがアクセストークンではありません。".into(),
                    },
                };
                let (request, _) = service_req.into_parts();
                return Err(ServiceResponse::new(request, e.error_response()));
            }

            // リクエストにユーザーIDをデータとして追加
            service_req.extensions_mut().insert(content.user_id);

            // 後続のミドルウェアなどにリクエストの処理を移譲
            let future = service.call(service_req);

            // リクエストの処理が完了した後、リクエストの処理を移譲した先から返却されたフューチャーを、
            // レスポンスとして返却
            let resp = future.await?;

            tracing::info!("AuthenticatedUserMiddlewareの処理を完了");
            Ok(resp)
        })
    }
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
