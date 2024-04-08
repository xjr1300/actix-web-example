use std::collections::HashMap;

use actix_web::cookie::SameSite;
use cookie::Cookie;
use regex::Regex;
use reqwest::header::{CONTENT_TYPE, SET_COOKIE};
use reqwest::StatusCode;

use infra::repositories::postgres::user::insert_user_query;
use infra::repositories::postgres::PgRepository;
use infra::routes::accounts::{SignInResBody, SignUpReqBody, SignUpResBody, UserResBody};
use infra::routes::ErrorResponseBody;
use time::{Duration, OffsetDateTime};
use use_cases::accounts::JWT_TOKEN_EXPRESSION;
use use_cases::{UseCaseErrorCode, ERR_SAME_EMAIL_ADDRESS_IS_REGISTERED};

use crate::helpers::{
    sign_up_input, sign_up_request_body, sign_up_request_body_json, spawn_test_app, split_response,
    tokyo_tower_sign_up_request_body, ResponseParts, CONTENT_TYPE_APPLICATION_JSON,
};

/// 妥当なユーザー情報で、ユーザーがサインアップできることを確認
#[tokio::test]
#[ignore]
async fn user_can_sign_up_with_the_valid_info() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let json_body = sign_up_request_body_json();
    let req_body: SignUpReqBody = serde_json::from_str(&json_body)?;

    // 実行
    let response = app.sign_up(json_body).await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let content_type = headers.get(CONTENT_TYPE);
    let inserted_user: SignUpResBody = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::OK, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(req_body.email, inserted_user.email);

    Ok(())
}

/// Eメールアドレスがすでに登録されている場合に、ユーザーがサインアップできないことを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_up_because_another_user_has_same_email_was_registered(
) -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let json_body = sign_up_request_body_json();

    // 実行
    let _ = app.sign_up(json_body.clone()).await?;
    let response = app.sign_up(json_body).await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let content_type = headers.get(CONTENT_TYPE);
    let response_body: ErrorResponseBody = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(
        Some(ERR_SAME_EMAIL_ADDRESS_IS_REGISTERED),
        response_body.error_code
    );
    assert_eq!(
        "同じEメールアドレスを持つユーザーが、すでに登録されています。",
        response_body.message
    );

    Ok(())
}

/// `actix-web`がエラー処理したときのレスポンスを確認するために、代表してEメールアドレスの形式が
/// 間違っている場合に、ユーザーがサインアップできないことを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_up_with_invalid_email() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let json_body = sign_up_request_body_json().replace("foo@example.com", "foo");

    // 実行
    let response = app.sign_up(json_body).await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let content_type = headers.get(CONTENT_TYPE);
    let response_body: ErrorResponseBody = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert_eq!(None, response_body.error_code);
    let content_type = content_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(None, response_body.error_code);
    assert_eq!(
        "Eメールアドレスの形式が間違っています。",
        response_body.message
    );

    Ok(())
}

/// 固定電話番号と携帯電話番号が設定されていない場合に、ユーザーがサインアップできないことを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_up_without_fixed_phone_number_and_mobile_phone_number(
) -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let json_body = sign_up_request_body_json()
        .replace(r#""099-472-1111""#, "null")
        .replace(r#""090-1234-5678""#, "null");

    let response = app.sign_up(json_body).await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let content_type = headers.get(CONTENT_TYPE);
    let response_body: ErrorResponseBody = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(
        Some(UseCaseErrorCode::DomainRule as u32),
        response_body.error_code
    );
    assert_eq!(
        "ユーザーは固定電話番号または携帯電話番号を指定する必要があります。",
        response_body.message
    );

    Ok(())
}

/// 妥当でないユーザー権限コードが設定されている場合に、ユーザーがサインアップできないことを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_up_when_user_permission_code_is_invalid() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let json_body = sign_up_request_body_json()
        .replace(r#""userPermissionCode": 1,"#, r#""userPermissionCode": 0,"#);

    // 実行
    let response = app.sign_up(json_body).await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let content_type = headers.get(CONTENT_TYPE);
    let response_body: ErrorResponseBody = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert_eq!(
        Some(UseCaseErrorCode::Validation as u32),
        response_body.error_code
    );
    let content_type = content_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(
        Some(UseCaseErrorCode::Validation as u32),
        response_body.error_code
    );
    assert_eq!(
        "ユーザー権限区分コードが範囲外です。",
        response_body.message
    );

    Ok(())
}

/// * ユーザーがサインインできて、アクセス及びリフレッシュトークンを取得できることを確認
/// * レスポンスヘッダに、アクセス及びリフレッシュトークンを適切な属性でクッキーに保存する
///   ことを指示する`Set-Cookie`が存在することを確認
///   * 確認するクッキーの属性は、`SameSite`、`Secure`、`HttpOnly`、`Expires`
/// * アクセス及びリフレッシュトークンをハッシュ化した値をキー、ユーザーIDを値としたペアが、
///   適切な有効期限で`Redis`に登録されていることを確認
#[tokio::test]
#[ignore]
async fn user_can_sign_in() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let http_server_settings = &app.settings.http_server;
    let authorization_settings = &app.settings.authorization;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let input = sign_up_input(body.clone(), &app.settings.password);
    app.register_user(input.clone()).await?;

    // クッキーにはミリ秒まで記録されないため1秒過去に設定
    let requesting_at = OffsetDateTime::now_utc() - Duration::seconds(1);

    // 実行
    let response = app
        .sign_in(body.email.clone(), body.password.clone())
        .await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let context_type = headers.get(CONTENT_TYPE);
    let tokens: SignInResBody = serde_json::from_str(&body)?;

    // クッキーにはミリ秒まで記録されないため1秒未来に設定
    let received_at = OffsetDateTime::now_utc() + Duration::seconds(1);

    // レスポンスを検証
    assert_eq!(StatusCode::OK, status_code);
    assert!(context_type.is_some());
    let content_type = context_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    // レスポンスヘッダを検証
    let set_cookie_values = headers.get_all(SET_COOKIE);
    let mut set_cookies: HashMap<String, Cookie> = HashMap::new();
    for value in set_cookie_values {
        let cookie = Cookie::parse(value.to_str()?)?;
        set_cookies.insert(cookie.name().to_string(), cookie);
    }
    // `Set-Cookie`にアクセス／リフレッシュトークンが存在するか確認
    let access_cookie = set_cookies.get("access").unwrap();
    let refresh_cookie = set_cookies.get("refresh").unwrap();
    // アクセストークンのクッキーを検証
    inspect_token_cookie_spec(
        access_cookie,
        http_server_settings.same_site,
        http_server_settings.secure,
        true,
        requesting_at,
        received_at,
        authorization_settings.access_token_seconds,
    );
    // リフレッシュトークンのクッキ＝を検証
    inspect_token_cookie_spec(
        refresh_cookie,
        http_server_settings.same_site,
        http_server_settings.secure,
        true,
        requesting_at,
        received_at,
        authorization_settings.refresh_token_seconds,
    );
    // アクセス／リフレッシュトークンを検証
    let regex = Regex::new(JWT_TOKEN_EXPRESSION).unwrap();
    assert!(regex.is_match(&tokens.access));
    assert!(regex.is_match(&tokens.refresh));
    assert_ne!(tokens.access, tokens.refresh);

    Ok(())
}

/// アクセス／リフレッシュトークン保存するクッキーの仕様を確認する。
///
/// # 引数
///
/// * `cookie` - アクセス／リフレッシュトークンを保存するクッキー
/// * `expected_same_site` - 予期する`SameSite`の値
/// * `expected_secure` - `Secure`を設定するかを示すフラグ
/// * `expected_http_only` - `HttpOnly`を設定するかを示すフラグ
/// * `requesting_at` - サインインをリクエストした日時
/// * `received_at` - サインインのレスポンスを受け取った日時
/// * `expiration` - アクセス／リフレッシュトークンの有効期限（秒）
///
/// `requesting_at`は、サインインをリクエストする直前の日時とする。
/// `received_at`は、サインインのレスポンスを受け取った日時とする。
/// よって、クッキーの有効期限は、`requesting_at` + `expiration`以上で、`received_at` + `expiration`以下となる。
fn inspect_token_cookie_spec(
    cookie: &Cookie<'_>,
    expected_same_site: SameSite,
    expected_secure: bool,
    expected_http_only: bool,
    requesting_at: OffsetDateTime,
    received_at: OffsetDateTime,
    expiration: u64,
) {
    assert_eq!(
        expected_same_site.to_string(),
        cookie.same_site().unwrap().to_string()
    );
    assert_eq!(expected_secure, cookie.secure().unwrap());
    assert_eq!(expected_http_only, cookie.http_only().unwrap());
    let duration = Duration::seconds(expiration as i64);
    let range_begin = requesting_at + duration;
    let range_end = received_at + duration;
    let cookie_expiration = cookie.expires_datetime().unwrap();
    assert!(
        range_begin <= cookie_expiration,
        "`{} <= {}` is not satisfied",
        range_begin,
        cookie_expiration
    );
    assert!(
        range_end >= cookie_expiration,
        "`{} >= {}` is not satisfied",
        range_end,
        cookie_expiration
    );
}

/// # サインイン統合テストリスト
///
/// * パスワードが間違っているときにユーザーがサインインできず、ユーザーがサインインを試行した日時と
///   サインイン試行回数の1がデータベースに記録されていることを確認
/// * 指定したEメールアドレスを持つユーザーが登録されていないときに、NOT FOUNDが返されることを確認
/// * 指定時間内にユーザーが2回サインインに失敗したときに、データベースに記録されているユーザーの
///   試行開始日時が変更されず、サインイン試行回数が2になっていることを確認
/// * アカウントがロックされているユーザーがサインインできないことを確認
/// * ユーザーが指定時間内に指定回数サインインに失敗したときに、アカウントがロックされていることを確認
/// * ユーザーが指定時間内に指定回数未満でサインインに成功したときに、データベースに記録された
///   サインイン試行開始日時が`NULL`、サインイン試行回数が0になっていることを確認
/// * `Redis`に登録されたアクセス及びリフレッシュトークンが、有効期限を超えたときに削除されている
///   ことを確認

/// データベースに登録したユーザーをリストできることを確認
#[tokio::test]
#[ignore]
async fn can_list_users() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let repo = PgRepository::<i32>::new(app.pool.clone());
    let json = sign_up_request_body_json();
    let body1 = sign_up_request_body(&json);
    let input1 = sign_up_input(body1.clone(), &app.settings.password);
    let body2 = tokyo_tower_sign_up_request_body();
    let input2 = sign_up_input(body2.clone(), &app.settings.password);

    // 2ユーザーがサインイン
    let mut tx = repo.begin().await?;
    insert_user_query(input1).fetch_one(&mut *tx).await?;
    insert_user_query(input2).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    // ユーザーのリストをリクエスト
    let response = app.list_users().await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let content_type = headers.get(CONTENT_TYPE);
    let users: Vec<UserResBody> = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(StatusCode::OK, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(2, users.len());
    assert!(
        user_res_body_is_match_sign_up_req_body(&body1, &users[0]),
        "{:?} is not match to {:?}",
        body1,
        users[0]
    );
    assert!(
        user_res_body_is_match_sign_up_req_body(&body2, &users[1]),
        "{:?} is not match to {:?}",
        body2,
        users[1]
    );

    Ok(())
}

fn user_res_body_is_match_sign_up_req_body(req: &SignUpReqBody, res: &UserResBody) -> bool {
    if req.email != res.email {
        return false;
    };
    if req.user_permission_code != res.user_permission.code {
        return false;
    }
    if req.family_name != res.family_name {
        return false;
    }
    if req.given_name != res.given_name {
        return false;
    }
    if req.postal_code != res.postal_code {
        return false;
    }
    if req.address != res.address {
        return false;
    }
    if req.fixed_phone_number != res.fixed_phone_number {
        return false;
    }
    if req.mobile_phone_number != res.mobile_phone_number {
        return false;
    };

    req.remarks == res.remarks
}
