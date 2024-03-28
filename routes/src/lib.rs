use std::{borrow::Cow, str::FromStr as _};

use actix_web::dev::ServiceResponse;
use actix_web::http::header::{self, HeaderMap};
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{HttpResponse, Responder};
use mime::Mime;

/// エラー・レスポンス・ボディ
///
/// アプリケーションから返されるエラー・レスポンスのボディを表現する。
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseBody {
    /// アプリ独自のエラー・コード
    ///
    /// `actix-web`がエラー処理した場合は`None`である。
    error_code: Option<u32>,

    /// エラー・メッセージ
    message: Cow<'static, str>,
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
    HttpResponse::Ok().body("It works!")
}
