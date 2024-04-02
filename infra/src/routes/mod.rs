pub mod accounts;

use std::{borrow::Cow, str::FromStr as _};

use actix_web::dev::ServiceResponse;
use actix_web::http::header::{self, HeaderMap, TryIntoHeaderValue as _};
use actix_web::http::StatusCode;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{HttpResponse, Responder, ResponseError};
use mime::Mime;

use domain::DomainError;

/// リクエスト処理結果
pub type ProcessRequestResult<T> = Result<T, ProcessRequestError>;

/// リクエスト処理エラー
///
/// * ドメイン層で発生したエラーは、`DomainError` -> `ProcessRequestError`に変換する。
/// * ユース・ケース層で発生したエラーは、次のように変換する。
///   * ユース・ケースでエラーが発生した場合、`UseCaseError` -> `ProcessRequestError`
///   * ユース・ケースがドメイン層のエラーを取得した場合、`DomainError` -> `UseCaseError` -> `ProcessRequestError`
#[derive(Debug, Clone, thiserror::Error)]
pub struct ProcessRequestError {
    /// HTTPステータス・コード
    pub status_code: StatusCode,
    /// レスポンス・ボディ
    pub body: ErrorResponseBody,
}

/// リクエスト処理エラーを、`actix-web`のエラー・レスポンスとして扱えるように`ResponseError`を実装する。
impl ResponseError for ProcessRequestError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse {
        let res = HttpResponse::new(self.status_code());
        let mut res = res.set_body(serde_json::to_string(&self.body).unwrap());
        let mime = mime::APPLICATION_JSON.try_into_value().unwrap();
        res.headers_mut().insert(header::CONTENT_TYPE, mime);

        res.map_into_boxed_body()
    }
}

impl std::fmt::Display for ProcessRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status_code.canonical_reason() {
            Some(reason) => {
                write!(
                    f,
                    "status_code={}, reason={}, {}",
                    self.status_code, reason, self.body
                )
            }
            None => {
                write!(f, "status_code={}, {}", self.status_code, self.body)
            }
        }
    }
}

/// ドメイン層で発生したエラーをリクエスト処理エラーに変換する。
impl From<DomainError> for ProcessRequestError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::Unexpected(err) | DomainError::Repository(err) => ProcessRequestError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                body: ErrorResponseBody {
                    error_code: None,
                    message: format!("{err}").into(),
                },
            },
            DomainError::Validation(message) | DomainError::DomainRule(message) => {
                ProcessRequestError {
                    status_code: StatusCode::BAD_REQUEST,
                    body: ErrorResponseBody {
                        error_code: None,
                        message,
                    },
                }
            }
        }
    }
}

/// エラー・レスポンス・ボディ
///
/// アプリケーションから返されるエラー・レスポンスのボディを表現する。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseBody {
    /// アプリ独自のエラー・コード
    ///
    /// `actix-web`がエラー処理した場合は`None`である。
    pub error_code: Option<u32>,

    /// エラー・メッセージ
    pub message: Cow<'static, str>,
}

impl std::fmt::Display for ErrorResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_code {
            Some(error_code) => {
                write!(
                    f,
                    r#"error_code={}, message="{}""#,
                    error_code, self.message
                )
            }
            None => {
                write!(f, r#"message="{}""#, self.message)
            }
        }
    }
}

impl ErrorResponseBody {
    pub fn new<T>(error_code: Option<u32>, message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            error_code,
            message: message.into(),
        }
    }
}

/// HTTPヘッダからContent-Typeを取得する。
///
/// # 引数
///
/// * `headers` - HTTPヘッダ
///
/// # 戻り値
///
/// * `Mime`
/// * Content-Typeが設定されていない場合は`None`
fn retrieve_content_type(headers: &HeaderMap) -> Option<Mime> {
    let content_type = headers.get(header::CONTENT_TYPE)?;
    let content_type = content_type.to_str().ok()?;
    match Mime::from_str(content_type) {
        Ok(mime) => Some(mime),
        Err(_) => None,
    }
}

/// カスタム・デフォルト・エラー・ハンドラ
pub fn default_error_handler<B>(
    res: ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    // コンテンツ・タイプがapplication/jsonの場合はそのまま返す
    let content_type = retrieve_content_type(res.headers());
    if content_type.is_some() && content_type.unwrap() == mime::APPLICATION_JSON {
        return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
    }

    // レスポンス・ボディを生成
    let message = res
        .status()
        .canonical_reason()
        .unwrap_or("Unexpected error raised");
    let body = ErrorResponseBody::new(None, message);
    let body = serde_json::to_string(&body).unwrap();
    let (req, res) = res.into_parts();
    let mut res = res.set_body(body);
    // レスポンスのヘッダを`application/json`に設定
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
    );
    // レスポンスを構築
    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(res))
}

/// ヘルス・チェック
#[tracing::instrument(name = "health check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::APPLICATION_JSON))
        .body(r#"{"message": "It works!"}"#)
}
