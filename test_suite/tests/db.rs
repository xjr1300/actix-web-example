use infra::repositories::postgres::{IsolationLevel, PgRepository};

use crate::helpers::spawn_test_app;

/// トランザクションを開始して、コミットできるか確認
#[tokio::test]
#[ignore]
async fn transaction_works() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;
    let repo = PgRepository::<i32>::new(app.pool.clone());

    // リード・コミット
    {
        let _ = repo.begin().await?;
    }

    // リピータブル・リード
    {
        let _ = repo.begin_with_level(IsolationLevel::ReadCommit).await?;
    }

    // シリアライザブル
    {
        let _ = repo.begin_with_level(IsolationLevel::Serializable).await?;
    }

    Ok(())
}
