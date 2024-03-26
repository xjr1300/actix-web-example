use std::path::Path;

use server::settings::SETTINGS_DIR_NAME;

#[tokio::test]
#[ignore]
async fn health_check_works() -> anyhow::Result<()> {
    // 準備
    spawn_app().await?;
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get("http://localhost:8000/")
        .send()
        .await
        .expect("Failed to execute request.");

    // 検証
    assert!(response.status().is_success());
    assert_eq!(Some("It works!".len() as u64), response.content_length());

    Ok(())
}

async fn spawn_app() -> anyhow::Result<()> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let settings_dir = dir.join("..").join(SETTINGS_DIR_NAME);
    let server = server::startup::run(settings_dir).await?;
    tokio::spawn(server);

    Ok(())
}
