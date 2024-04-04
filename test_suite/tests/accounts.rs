use reqwest::header::CONTENT_TYPE;

use infra::routes::accounts::{SignUpReqBody, SignUpResBody};
use infra::routes::ErrorResponseBody;
use use_cases::UseCaseErrorCode;

use crate::helpers::{sign_up_request_body_json, spawn_test_app, CONTENT_TYPE_APPLICATION_JSON};

/// 妥当なユーザー情報で、ユーザーがサインアップできることを確認
#[tokio::test]
#[ignore]
async fn user_can_sign_up_with_the_valid_info() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;

    let json_body = sign_up_request_body_json();
    let body: SignUpReqBody = serde_json::from_str(&json_body).unwrap();

    // 実行
    let response = app.request_accounts_sign_up(json_body).await?;
    let status_code = response.status();
    let headers = response.headers().clone();
    let content_type = headers.get(CONTENT_TYPE);
    let added_user = response.json::<SignUpResBody>().await?;

    // 検証
    assert_eq!(reqwest::StatusCode::OK, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap().to_str()?;
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type);
    assert_eq!(body.email, added_user.email);

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
    let _ = app.request_accounts_sign_up(json_body.clone()).await?;
    let response = app.request_accounts_sign_up(json_body).await?;
    let status_code = response.status();
    let headers = response.headers().clone();
    let content_type = headers.get(CONTENT_TYPE);
    let response_body = response.json::<ErrorResponseBody>().await?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap().to_str()?;
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type);
    assert_eq!(
        Some(UseCaseErrorCode::DomainRule as u32),
        response_body.error_code
    );
    assert_eq!(
        "同じEメール・アドレスを持つユーザーが、すでに登録されています。",
        response_body.message
    );

    Ok(())
}

/// `actix-web`がエラー処理したときのレスポンスを確認するために、代表してEメール・アドレスの形式が
/// 間違っている場合に、ユーザーがサインアップできないことを確認
#[tokio::test]
#[ignore]
async fn user_can_not_sign_up_with_invalid_email() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let json_body = sign_up_request_body_json().replace("foo@example.com", "foo");

    // 実行
    let response = app.request_accounts_sign_up(json_body).await?;
    let status_code = response.status();
    let headers = response.headers().clone();
    let content_type = headers.get(CONTENT_TYPE);
    let response_body = response.json::<ErrorResponseBody>().await?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert_eq!(
        Some(UseCaseErrorCode::Validation as u32),
        response_body.error_code
    );
    let content_type = content_type.unwrap().to_str()?;
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type);
    assert_eq!(
        Some(UseCaseErrorCode::Validation as u32),
        response_body.error_code
    );
    assert_eq!(
        "Eメール・アドレスの形式が間違っています。",
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

    let response = app.request_accounts_sign_up(json_body).await?;
    let status_code = response.status();
    let headers = response.headers().clone();
    let content_type = headers.get(CONTENT_TYPE);
    let response_body = response.json::<ErrorResponseBody>().await?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert!(content_type.is_some());
    let content_type = content_type.unwrap().to_str()?;
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type);
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
    let response = app.request_accounts_sign_up(json_body).await?;
    let status_code = response.status();
    let headers = response.headers().clone();
    let content_type = headers.get(CONTENT_TYPE);
    let response_body = response.json::<ErrorResponseBody>().await?;

    // 検証
    assert_eq!(reqwest::StatusCode::BAD_REQUEST, status_code);
    assert_eq!(
        Some(UseCaseErrorCode::Validation as u32),
        response_body.error_code
    );
    let content_type = content_type.unwrap().to_str()?;
    assert_eq!(CONTENT_TYPE_APPLICATION_JSON, content_type);
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
