use actix_web::{web, HttpResponse};
use secrecy::SecretString;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::common::now_jst;

/// アカウントスコープを返却する。
pub fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts").service(web::resource("/signup").route(web::post().to(signup)))
}

/// サインアップ
pub async fn signup(request_body: web::Json<SignupRequestBody>) -> HttpResponse {
    let dt = now_jst();
    let response_body = SignupResponseBody {
        id: Uuid::new_v4(),
        email: request_body.email.clone(),
        created_at: dt,
        updated_at: dt,
    };

    HttpResponse::Ok().json(response_body)
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
