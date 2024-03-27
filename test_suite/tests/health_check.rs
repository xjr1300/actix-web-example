use std::net::TcpListener;

use anyhow::Context as _;
use once_cell::sync::Lazy;

use server::telemetry::{generate_log_subscriber, init_log_subscriber};

/// 統合テスト用アプリ
pub struct TestApp {
    /// アプリのルートURI
    pub root_uri: String,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_level = log::Level::Info;
    let subscriber_name = String::from("test");

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = generate_log_subscriber(subscriber_name, default_level, std::io::stdout);
        init_log_subscriber(subscriber);
    } else {
        let subscriber = generate_log_subscriber(subscriber_name, default_level, std::io::sink);
        init_log_subscriber(subscriber);
    }
});

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

/// 統合テスト用のHTTPサーバーを起動する。
///
/// # 戻り値
///
/// 統合テスト用のHTTPサーバーのルートURI
async fn spawn_app_for_integration_test() -> anyhow::Result<TestApp> {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("localhost:0").context("failed to bind random port")?;
    let port = listener.local_addr().unwrap().port();
    let server = server::startup::build_http_server(listener)?;
    tokio::spawn(server);

    Ok(TestApp {
        root_uri: format!("http://localhost:{}", port),
    })
}
