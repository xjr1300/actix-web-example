use std::net::TcpListener;

use anyhow::Context as _;

/// ヘルス・チェック・ハンドラ
#[tokio::test]
#[ignore]
async fn health_check_works() -> anyhow::Result<()> {
    // 準備
    let address = spawn_http_server().await?;
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get(&format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // 検証
    assert!(response.status().is_success());
    assert_eq!(Some("It works!".len() as u64), response.content_length());

    Ok(())
}

/// 統合テスト用のHTTPサーバーを起動する。
///
/// # 戻り値
///
/// 統合テスト用のHTTPサーバーのルートURI
async fn spawn_http_server() -> anyhow::Result<String> {
    let listener = TcpListener::bind("localhost:0").context("failed to bind random port")?;
    let port = listener.local_addr().unwrap().port();
    let server = server::startup::build_http_server(listener)?;
    tokio::spawn(server);

    Ok(format!("http://localhost:{}", port))
}
