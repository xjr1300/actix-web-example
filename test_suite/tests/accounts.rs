use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;

use infra::repositories::postgres::user::insert_user_query;
use infra::repositories::postgres::PgRepository;
use infra::routes::accounts::{SignUpReqBody, SignUpResBody, UserResBody};
use infra::routes::ErrorResponseBody;
use use_cases::UseCaseErrorCode;

use crate::helpers::{
    sign_up_input, sign_up_request_body, sign_up_request_body_json, spawn_test_app, split_response,
    tokyo_tower_sign_up_request_body, ResponseParts, CONTENT_TYPE_APPLICATION_JSON,
};

/// 妥当なユーザー情報で、ユーザーがサイン・アップできることを確認
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

/// Eメールアドレスがすでに登録されている場合に、ユーザーがサイン・アップできないことを確認
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
/// 間違っている場合に、ユーザーがサイン・アップできないことを確認
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
        "Eメール・アドレスの形式が間違っています。",
        response_body.message
    );

    Ok(())
}

/// 固定電話番号と携帯電話番号が設定されていない場合に、ユーザーがサイン・アップできないことを確認
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

/// 妥当でないユーザー権限コードが設定されている場合に、ユーザーがサイン・アップできないことを確認
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

    // 2ユーザーがサイン・イン
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
