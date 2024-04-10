use std::future::Future;
use std::pin::Pin;
use std::str::FromStr as _;

use actix_web::http::{header, StatusCode};
use actix_web::{web, FromRequest, HttpRequest};
use secrecy::SecretString;
use uuid::Uuid;

use domain::models::user::{UserId, UserPermissionCode};
use domain::repositories::token::{TokenContent, TokenRepository, TokenType};

use crate::repositories::redis::token::RedisTokenRepository;
use crate::routes::{
    ErrorResponseBody, ProcessRequestError, ProcessRequestResult, ACCESS_TOKEN_KEY,
};
use crate::RequestContext;

/// 認証済みユーザーのみがアクセス可能なコンテキスト
pub struct UserContext(pub TokenContent);

impl FromRequest for UserContext {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let request = req.clone();

        Box::pin(async move {
            // リクエストヘッダからアクセストークンを取得してトークンコンテンツを取得
            let content = retrieve_token_content(&request).await?;

            Ok(Self(content))
        })
    }
}

/// 管理権限を持つユーザーのアクセス可能なコンテキスト
pub struct AdminContext {
    pub user_id: UserId,
}

impl FromRequest for AdminContext {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let request = req.clone();

        Box::pin(async move {
            // リクエストヘッダからアクセストークンを取得してトークンコンテンツを取得
            let content = retrieve_token_content(&request).await?;
            if content.user_permission_code != UserPermissionCode::Admin {
                return Err(forbidden_actix_error());
            }

            Ok(Self {
                user_id: content.user_id,
            })
        })
    }
}

/// ユーザー自身のみアクセス可能なコンテキスト
///
/// パスに`{user_id}`を含み、それがユーザーIDであること。
pub struct UserOwnContext {
    pub user_id: UserId,
}

impl FromRequest for UserOwnContext {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let request = req.clone();

        Box::pin(async move {
            // リクエストヘッダからアクセストークンを取得してトークンコンテンツを取得
            let content = retrieve_token_content(&request).await?;
            // リクエストURIからユーザーIDを文字列で取得
            let user_id = request.match_info().get("user_id").ok_or_else(|| {
                ProcessRequestError::without_error_code(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "リクエストURIに`{user_id}`パスが必要です。",
                )
            })?;
            // 文字列で表現されたユーザーIDを変換
            let user_id = Uuid::from_str(user_id).map_err(|_| {
                ProcessRequestError::without_error_code(
                    StatusCode::BAD_REQUEST,
                    "リクエストURIで指定されたユーザーIDをUUIDに変換できません。",
                )
            })?;
            // リクエストURIで指定されたユーザーIDと、アクセストークンに紐付いたユーザーIDを比較して、
            // リクエストしたユーザー自身の情報をリクエストしているか確認
            let user_id = UserId::new(user_id);
            if content.user_id != user_id {
                return Err(forbidden_actix_error());
            }

            Ok(Self { user_id })
        })
    }
}

pub fn forbidden_error() -> ProcessRequestError {
    ProcessRequestError::without_error_code(StatusCode::FORBIDDEN, "アクセスする権限がありません。")
}
pub fn forbidden_actix_error() -> actix_web::Error {
    actix_web::Error::from(forbidden_error())
}

async fn retrieve_token_content(request: &HttpRequest) -> actix_web::Result<TokenContent> {
    // リクエストからアクセストークンを取得
    let token = retrieve_access_token(request)?;
    if token.is_none() {
        return Err(forbidden_actix_error());
    }
    let token = token.unwrap();

    // Redisからアクセストークンに紐付いたトークンの内容を取得
    let content = token_content_from_redis(request, &token).await?;
    if content.is_none() {
        return Err(forbidden_actix_error());
    }
    let content = content.unwrap();

    // アクセストークンが、本当にアクセストークンであるか確認
    if content.token_type != TokenType::Access {
        return Err(ProcessRequestError::without_error_code(
            StatusCode::BAD_REQUEST,
            "リフレッシュトークンが送信されました。",
        )
        .into());
    }

    Ok(content)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
enum ParseError {
    #[error("Authorizationヘッダの内容が誤っています。")]
    Invalid,
    #[error("Authorizationヘッダの内容が`Bearer`から始まっていません。")]
    MissingSchema,
    #[error("Authorizationヘッダの内容にアクセストークンが含まれていません。")]
    MissingToken,
}

// リクエストヘッダからアクセストークンを取得する。
fn retrieve_access_token(request: &HttpRequest) -> ProcessRequestResult<Option<SecretString>> {
    // クッキーからアクセストークンを取得
    let token = access_token_from_cookie(request);
    if token.is_some() {
        return Ok(token);
    }
    // `Authorization`ヘッダからアクセストークンを取得
    let token = access_token_from_auth_header(request).map_err(|e| {
        ProcessRequestError::without_error_code(StatusCode::BAD_REQUEST, format!("{}", e))
    })?;

    Ok(token)
}

/// クッキーからアクセストークンを取得する。
fn access_token_from_cookie(request: &HttpRequest) -> Option<SecretString> {
    request
        .cookie(ACCESS_TOKEN_KEY)
        .map(|c| SecretString::new(c.value().to_string()))
}

/// リクエストの`Authorization`ヘッダーからアクセストークンを取得する。
fn access_token_from_auth_header(
    request: &HttpRequest,
) -> Result<Option<SecretString>, ParseError> {
    let header_value = request.headers().get(header::AUTHORIZATION);
    if header_value.is_none() {
        return Ok(None);
    }
    let header_value = header_value.unwrap();
    // "Bearer *"
    //  12345678
    if header_value.len() < 8 {
        return Err(ParseError::Invalid);
    }
    let mut parts = header_value
        .to_str()
        .map_err(|_| ParseError::Invalid)?
        .splitn(2, ' ');
    if parts.next() != Some("Bearer") {
        return Err(ParseError::MissingSchema);
    }
    let token = parts.next().ok_or(ParseError::MissingToken)?;

    Ok(Some(SecretString::new(token.to_string())))
}

// Redisからアクセストークンに紐付いたトークンの内容を取得する。
async fn token_content_from_redis(
    request: &HttpRequest,
    token: &SecretString,
) -> ProcessRequestResult<Option<TokenContent>> {
    let context = request
        .app_data::<web::Data<RequestContext>>()
        .ok_or_else(|| {
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
    let repo = RedisTokenRepository::new(context.redis_pool.clone());
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
