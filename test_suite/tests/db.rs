use infra::repositories::postgres::{IsolationLevel, PgRepository};

use crate::helpers::{app_settings, spawn_test_app};

/// トランザクションを開始して、コミットできるか確認
#[tokio::test]
#[ignore]
async fn transaction_works() -> anyhow::Result<()> {
    // 準備
    let settings = app_settings()?;
    let app = spawn_test_app(settings).await?;
    let repo = PgRepository::<i32>::new(app.pg_pool.clone());

    // リードコミット
    {
        let _ = repo.begin().await?;
    }

    // リピータブルリード
    {
        let _ = repo.begin_with_level(IsolationLevel::ReadCommit).await?;
    }

    // シリアライザブル
    {
        let _ = repo.begin_with_level(IsolationLevel::Serializable).await?;
    }

    Ok(())
}
