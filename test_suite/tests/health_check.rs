use crate::helpers::spawn_app_for_integration_test;

/// ヘルス・チェック・ハンドラ
#[tokio::test]
#[ignore]
async fn health_check_works() -> anyhow::Result<()> {
    // 準備
    let app = spawn_app_for_integration_test().await?;
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get(&format!("{}/health_check", app.root_uri))
        .send()
        .await
        .expect("Failed to execute request.");

    // 検証
    assert!(response.status().is_success());
    assert_eq!(Some("It works!".len() as u64), response.content_length());

    Ok(())
}
