use crate::helpers::{spawn_test_app, split_response};

/// ヘルスチェック・ハンドラ
#[tokio::test]
#[ignore]
async fn health_check_works() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get(&format!("{}/health-check", app.root_uri))
        .send()
        .await
        .expect("Failed to execute request.");
    let response = split_response(response).await?;
    let content_type = response.headers.get(reqwest::header::CONTENT_TYPE);
    let body: serde_json::Value = serde_json::from_str(&response.body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::OK, response.status_code);
    assert!(content_type.is_some());
    assert_eq!(
        mime::APPLICATION_JSON.to_string(),
        content_type.unwrap().to_str().unwrap()
    );
    assert_eq!(serde_json::json!("It works!"), body["message"]);

    Ok(())
}

/// 存在しないURIにアクセスしたときに、正しいレスポンスが得られるか確認
#[tokio::test]
#[ignore]
async fn not_found_works() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get(&format!("{}/non-existent-uri", app.root_uri))
        .send()
        .await
        .expect("Failed to execute request.");
    let response = split_response(response).await?;
    let content_type = response.headers.get(reqwest::header::CONTENT_TYPE);
    let body: serde_json::Value = serde_json::from_str(&response.body)?;

    // 検証
    assert_eq!(reqwest::StatusCode::NOT_FOUND, response.status_code);
    assert!(content_type.is_some());
    assert_eq!(
        mime::APPLICATION_JSON.to_string(),
        content_type.unwrap().to_str().unwrap()
    );
    assert_eq!(body["message"], serde_json::json!("Not Found"));

    Ok(())
}
