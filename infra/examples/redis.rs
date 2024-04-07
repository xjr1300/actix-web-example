use deadpool_redis::{Config, Runtime};
use redis::{cmd, AsyncCommands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "redis://localhost:6379";
    let config = Config::from_url(url);
    let pool = config.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();
    }

    // 存在するキー
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(value, "42".to_string());
    }

    // 存在しないキー
    {
        let mut conn = pool.get().await.unwrap();
        let value: Option<String> = conn.get("unknown").await.unwrap();
        assert!(value.is_none());
    }

    Ok(())
}
