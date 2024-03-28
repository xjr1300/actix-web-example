use actix_web::{web, HttpResponse};
use secrecy::SecretString;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::common::now_jst;
use domain::models::primitives::*;
use use_cases::accounts::SignupUser;

use crate::common::{ProcessRequestError, ProcessRequestResult};

/// アカウントスコープを返却する。
pub fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts").service(web::resource("/signup").route(web::post().to(signup)))
}

/// サインアップ
pub async fn signup(
    request_body: web::Json<SignupRequestBody>,
) -> ProcessRequestResult<HttpResponse> {
    let email = request_body.email.clone();
    let _signup_user = SignupUser::try_from(request_body.0).map_err(ProcessRequestError::from)?;

    let dt = now_jst();
    let response_body = SignupResponseBody {
        id: Uuid::new_v4(),
        email: email.to_string(),
        created_at: dt,
        updated_at: dt,
    };

    Ok(HttpResponse::Ok().json(response_body))
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
        let family_name = FamilyName::new(value.family_name)?;
        let given_name = GivenName::new(value.given_name)?;
        let postal_code = PostalCode::new(value.postal_code)?;
        let address = Address::new(value.address)?;
        let fixed_phone_number = to_option_fixed_phone_number(value.fixed_phone_number)?;
        let mobile_phone_number = to_option_mobile_phone_number(value.mobile_phone_number)?;
        let remarks = to_option_remarks(value.remarks)?;

        let mut builder = SignupUser::builder();
        builder
            .email(email)
            .password(value.password)
            .family_name(family_name)
            .given_name(given_name)
            .postal_code(postal_code)
            .address(address);
        if let Some(fixed_phone_number) = fixed_phone_number {
            builder.fixed_phone_number(fixed_phone_number);
        }
        if let Some(mobile_phone_number) = mobile_phone_number {
            builder.mobile_phone_number(mobile_phone_number);
        }
        if let Some(remarks) = remarks {
            builder.remarks(remarks);
        }
        Ok(builder.build().unwrap())
    }
}
