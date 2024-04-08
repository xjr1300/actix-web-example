use std::collections::HashMap;

use actix_web::cookie::SameSite;
use cookie::Cookie;
use regex::Regex;
use reqwest::header::{CONTENT_TYPE, SET_COOKIE};
use reqwest::StatusCode;
use secrecy::SecretString;
use time::{Duration, OffsetDateTime};

use domain::repositories::token::TokenType;
use domain::repositories::user::{UserCredential, UserRepository};
use infra::repositories::postgres::user::{insert_user_query, PgUserRepository};
use infra::repositories::postgres::PgRepository;
use infra::routes::accounts::{SignInResBody, SignUpReqBody, SignUpResBody, UserResBody};
use infra::routes::ErrorResponseBody;
use use_cases::accounts::JWT_TOKEN_EXPRESSION;
use use_cases::{UseCaseErrorCode, ERR_SAME_EMAIL_ADDRESS_IS_REGISTERED};

use crate::helpers::{
    app_settings, sign_up_input, sign_up_request_body, sign_up_request_body_json, spawn_test_app,
    split_response, tokyo_tower_sign_up_request_body, ResponseParts, CONTENT_TYPE_APPLICATION_JSON,
};

/// 妥当なユーザー情報で、ユーザーがサインアップできることを確認
#[tokio::test]
#[ignore]
async fn user_can_sign_up_with_the_valid_info() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
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
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
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
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
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
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
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
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
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
/// * ユーザーが最後にサインインした日時がデータベースに記録されていることを確認
#[tokio::test]
#[ignore]
async fn user_can_sign_in() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let http_server_settings = &app.settings.http_server;
    let authorization_settings = &app.settings.authorization;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let sign_in_input = sign_up_input(body.clone(), &app.settings.password);
    let sign_up_output = app.register_user(sign_in_input.clone()).await?;

    // 実行
    let started_at = OffsetDateTime::now_utc() - Duration::seconds(1);
    let response = app
        .sign_in(body.email.clone(), body.password.clone())
        .await?;
    let finished_at = OffsetDateTime::now_utc() + Duration::seconds(1);
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let context_type = headers.get(CONTENT_TYPE);
    let tokens: SignInResBody = serde_json::from_str(&body)?;
    let user_repo = PgUserRepository::new(app.pg_pool.clone());
    let user = user_repo.by_id(sign_up_output.id).await?;

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
        started_at,
        finished_at,
        authorization_settings.access_token_seconds,
    );
    // リフレッシュトークンのクッキ＝を検証
    inspect_token_cookie_spec(
        refresh_cookie,
        http_server_settings.same_site,
        http_server_settings.secure,
        true,
        started_at,
        finished_at,
        authorization_settings.refresh_token_seconds,
    );
    // アクセス／リフレッシュトークンを検証
    let regex = Regex::new(JWT_TOKEN_EXPRESSION).unwrap();
    assert!(regex.is_match(&tokens.access));
    assert!(regex.is_match(&tokens.refresh));
    assert_ne!(tokens.access, tokens.refresh);

    // Redisにアクセストークンが登録されており、アクセストークンをキーとした値が、
    // 適切なユーザーIDとトークンの種類であるか確認
    let access_token = SecretString::new(tokens.access.clone());
    let access_content = app.retrieve_token_content(&access_token).await;
    assert!(access_content.is_some());
    let access_content = access_content.unwrap();
    assert_eq!(sign_up_output.id, access_content.user_id);
    assert_eq!(TokenType::Access, access_content.token_type);

    // Redisにリフレッシュトークンが登録されており、リフレッシュトークンをキーとした値が、
    // 適切なユーザーIDとトークンの種類であるか確認
    let refresh_token = SecretString::new(tokens.refresh.clone());
    let refresh_content = app.retrieve_token_content(&refresh_token).await;
    assert!(refresh_content.is_some());
    let access_content = refresh_content.unwrap();
    assert_eq!(sign_up_output.id, access_content.user_id);
    assert_eq!(TokenType::Refresh, access_content.token_type);

    // データベースに最後にログインした日時が記録されているか確認
    assert!(user.is_some());
    let user = user.unwrap();
    assert!(user.last_sign_in_at.is_some());
    let last_logged_in_at = user.last_sign_in_at.unwrap();
    assert!(
        started_at <= last_logged_in_at,
        "{} < {} is not satisfied",
        started_at,
        last_logged_in_at
    );
    assert!(
        last_logged_in_at <= finished_at,
        "{} < {} is not satisfied",
        last_logged_in_at,
        finished_at
    );

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

/// 間違ったパスワードでサインインを試行したときに、サインインできないことを確認
///
/// * サインインに失敗した最初の日時とサインインに失敗した回数が記録されていることを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_in_with_wrong_password() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let sign_in_input = sign_up_input(body.clone(), &app.settings.password);
    let sign_up_output = app.register_user(sign_in_input.clone()).await?;

    // 実行
    let started_at = OffsetDateTime::now_utc() - Duration::seconds(1);
    let response = app
        .sign_in(
            body.email.clone(),
            SecretString::new(String::from("1a@sE4tea%c-")),
        )
        .await?;
    let finished_at = OffsetDateTime::now_utc() + Duration::seconds(1);
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let context_type = headers.get(CONTENT_TYPE);
    let body: ErrorResponseBody = serde_json::from_str(&body)?;
    let user_repo = PgUserRepository::new(app.pg_pool.clone());
    let user = user_repo.by_id(sign_up_output.id).await?;

    // 検証
    assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    assert!(context_type.is_some());
    let content_type = context_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(Some(UseCaseErrorCode::Unauthorized as u32), body.error_code);
    assert_eq!(
        "Eメールアドレスまたはパスワードが間違っています。",
        body.message
    );
    assert!(user.is_some());
    let user = user.unwrap();
    // 最初にサインインに失敗した日時と、サインイン失敗回数を確認
    let credential = user_repo.user_credential(user.email).await?;
    assert!(credential.is_some());
    let credential = credential.unwrap();
    let attempted_at = credential.attempted_at.unwrap();
    assert!(
        started_at <= attempted_at,
        "{} < {} is not satisfied",
        started_at,
        attempted_at
    );
    assert!(
        attempted_at <= finished_at,
        "{} < {} is not satisfied",
        attempted_at,
        finished_at
    );
    assert_eq!(1, credential.number_of_failures);

    Ok(())
}

/// 間違ったEメールアドレスでサインインを試行したときに、サインインできないことを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_in_with_wrong_email() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);

    // 実行
    let response = app
        .sign_in(
            String::from("wrong-email-address@example.com"),
            body.password.clone(),
        )
        .await?;
    let ResponseParts {
        status_code,
        headers,
        body,
    } = split_response(response).await?;
    let context_type = headers.get(CONTENT_TYPE);
    let body: ErrorResponseBody = serde_json::from_str(&body)?;

    // 検証
    assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    assert!(context_type.is_some());
    let content_type = context_type.unwrap();
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type.to_str()?);
    assert_eq!(Some(UseCaseErrorCode::Unauthorized as u32), body.error_code);
    assert_eq!(
        "Eメールアドレスまたはパスワードが間違っています。",
        body.message
    );

    Ok(())
}

/// 指定時間内にユーザーが2回サインインに失敗したときに、データベースに記録されているユーザーの試行開始日時が変更されず、
/// サインイン試行回数が2になっていることを確認
#[tokio::test]
#[ignore]
async fn number_of_sign_in_failures_was_incremented_when_the_user_failed_to_sign_in_twice(
) -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let sign_in_input = sign_up_input(body.clone(), &app.settings.password);
    let _ = app.register_user(sign_in_input.clone()).await?;
    let user_repo = PgUserRepository::new(app.pg_pool.clone());

    // 実行
    let mut credentials: Vec<UserCredential> = vec![];
    for _ in 0..2 {
        let _ = app
            .sign_in(
                body.email.clone(),
                SecretString::new(String::from("1a@sE4tea%c-")),
            )
            .await?;
        let credential = user_repo
            .user_credential(sign_in_input.email.clone())
            .await?
            .unwrap();
        credentials.push(credential);
    }

    // 検証
    assert_eq!(1, credentials[0].number_of_failures);
    assert_eq!(2, credentials[1].number_of_failures);
    assert_eq!(credentials[0].attempted_at, credentials[1].attempted_at);

    Ok(())
}

/// ユーザーがサインインに失敗した後にサインインに成功したとき、サインイン失敗履歴がクリアされていることを確認
#[tokio::test]
#[ignore]
async fn sign_in_failed_history_was_cleared_when_user_sign_in_succeeded() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let sign_in_input = sign_up_input(body.clone(), &app.settings.password);
    let _ = app.register_user(sign_in_input.clone()).await?;
    let user_repo = PgUserRepository::new(app.pg_pool.clone());

    // サインイン失敗
    let _ = app
        .sign_in(
            body.email.clone(),
            SecretString::new(String::from("1a@sE4tea%c-")),
        )
        .await?;
    // 1秒スリープ
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // サインイン成功
    let response = app
        .sign_in(body.email.clone(), body.password.clone())
        .await?;
    let ResponseParts { status_code, .. } = split_response(response).await?;
    assert_eq!(StatusCode::OK, status_code);
    // クレデンシャルを取得
    let credential = user_repo
        .user_credential(sign_in_input.email.clone())
        .await?
        .unwrap();

    // 検証
    assert_eq!(0, credential.number_of_failures);
    assert!(credential.attempted_at.is_none());

    Ok(())
}

/// ユーザーがサインインにユーザーのアカウントをロックする失敗回数より1つ少ない回数失敗して、
/// サインインの失敗回数をカウントする時間が経過した後で再度サインインを試みたとき、サインイン試行開始日時
/// が更新され、サインイン失敗回数が1になっていて、ユーザーのアカウントがロックされていないことを確認
#[tokio::test]
#[ignore]
async fn a_failed_sign_in_after_the_period_has_elapsed_is_considered_the_first_failed(
) -> anyhow::Result<()> {
    // 準備
    let mut settings = app_settings()?;
    settings.authorization.number_of_failures = 2;
    settings.authorization.attempting_seconds = 2;
    let app = spawn_test_app(settings).await?;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let sign_in_input = sign_up_input(body.clone(), &app.settings.password);
    let _ = app.register_user(sign_in_input.clone()).await?;
    let user_repo = PgUserRepository::new(app.pg_pool.clone());

    // サインイン失敗
    let _ = app
        .sign_in(
            body.email.clone(),
            SecretString::new(String::from("1a@sE4tea%c-")),
        )
        .await?;
    // 2.5秒スリープ
    tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;
    // サインイン失敗
    let started_at = OffsetDateTime::now_utc();
    let _ = app
        .sign_in(
            body.email.clone(),
            SecretString::new(String::from("1a@sE4tea%c-")),
        )
        .await?;
    let finished_at = OffsetDateTime::now_utc();
    // クレデンシャルを取得
    let credential = user_repo
        .user_credential(sign_in_input.email.clone())
        .await?
        .unwrap();

    // 検証
    assert_eq!(1, credential.number_of_failures);
    assert!(credential.attempted_at.is_some());
    let attempted_at = credential.attempted_at.unwrap();
    assert!(
        started_at <= attempted_at,
        "{} < {} is not satisfied",
        started_at,
        attempted_at
    );
    assert!(
        attempted_at <= finished_at,
        "{} < {} is not satisfied",
        attempted_at,
        finished_at
    );
    assert!(credential.active);

    Ok(())
}

/// アカウントがロックされているユーザーがサインインできないことを確認
#[tokio::test]
#[ignore]
async fn the_user_locked_account_can_not_sign_in() -> anyhow::Result<()> {
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let json = sign_up_request_body_json();
    let body = sign_up_request_body(&json);
    let mut sign_in_input = sign_up_input(body.clone(), &app.settings.password);
    sign_in_input.active = false;
    let _ = app.register_user(sign_in_input.clone()).await?;

    let response = app
        .sign_in(body.email.clone(), body.password.clone())
        .await?;
    let ResponseParts { status_code, .. } = split_response(response).await?;

    assert_eq!(StatusCode::UNAUTHORIZED, status_code);

    Ok(())
}

///
/// # サインイン統合テストリスト
///
/// * ユーザーが指定時間内に指定回数サインインに失敗したときに、アカウントがロックされていることを確認
/// * `Redis`に登録されたアクセス及びリフレッシュトークンが、有効期限を超えたときに削除されている
///   ことを確認

/// データベースに登録したユーザーをリストできることを確認
#[tokio::test]
#[ignore]
async fn can_list_users() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let repo = PgRepository::<i32>::new(app.pg_pool.clone());
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
