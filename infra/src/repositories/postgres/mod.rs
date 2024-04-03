pub mod user;

use std::marker::PhantomData;

use sqlx::{PgPool, Postgres, Transaction};

use domain::{DomainError, DomainResult};

/// PostgreSQLトランザクション型
pub type PgTransaction<'c> = Transaction<'c, Postgres>;

/// PostgreSQLリポジトリ構造体
#[derive(Debug, Clone)]
pub struct PgRepository<T> {
    /// データベース接続プール
    pub pool: PgPool,
    /// マーカー
    _phantom: PhantomData<T>,
}

impl<T> PgRepository<T> {
    /// PostgreSQLリポジトリを構築する。
    ///
    /// # 引数
    ///
    /// * `pool` - データベース接続プール
    ///
    /// # 戻り値
    ///
    /// PostgreSQLリポジトリ
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            _phantom: Default::default(),
        }
    }

    /// デフォルトのトランザクション分離レベルとアクセス・モードで、トランザクションを開始する。
    ///
    /// # 戻り値
    ///
    /// トランザクション
    pub async fn begin<'c>(&self) -> DomainResult<PgTransaction<'c>> {
        self.pool
            .begin()
            .await
            .map_err(|e| DomainError::Repository(e.into()))
    }

    /// 指定したトランザクション分離モデルとデフォルトのアクセス・モードで、トランザクションを開始する。
    ///
    /// # 引数
    ///
    /// * `isolation_level` - トランザクション分離レベル
    ///
    /// # 戻り値
    ///
    /// トランザクション
    pub async fn begin_with_level<'c>(
        &self,
        isolation_level: IsolationLevel,
    ) -> DomainResult<PgTransaction<'c>> {
        // トランザクションを開始
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Repository(e.into()))?;
        // トランザクション分離モデルを設定
        let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", isolation_level);
        sqlx::query(&sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Repository(e.into()))?;

        Ok(tx)
    }

    /// 指定したザクション分離レベルとアクセス・モードで、トランザクションを開始する。
    ///
    /// # 引数
    ///
    /// * `isolation_level` - トランザクション分離レベル
    /// * `access_mode` - トランザクションのアクセス・モード
    ///
    /// # 戻り値
    ///
    /// トランザクション
    pub async fn begin_with_mode<'c>(
        &self,
        isolation_level: IsolationLevel,
        access_mode: AccessMode,
    ) -> DomainResult<PgTransaction<'c>> {
        // トランザクションを開始
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Repository(e.into()))?;
        // トランザクション分離モデルを設定
        let sql = format!(
            "SET TRANSACTION ISOLATION LEVEL {} {}",
            isolation_level, access_mode
        );
        sqlx::query(&sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Repository(e.into()))?;

        Ok(tx)
    }

    /// トランザクションを開始する。
    ///
    /// # 引数
    ///
    /// * `isolation_level` - トランザクション分離レベル
    /// * `access_mode` - トランザクションのアクセス・モード
    /// * `deferrable` - トランザクションがコミットされるまでチェックを延期(defer)
    ///
    /// # 戻り値
    ///
    /// トランザクション
    pub async fn begin_with_full<'c>(
        &self,
        isolation_level: IsolationLevel,
        access_mode: AccessMode,
        deferrable: bool,
    ) -> DomainResult<PgTransaction<'c>> {
        // デフェラブルは、シリアライザブルかつ読み込み専用のトランザクションでのみ有効
        if deferrable
            && (isolation_level != IsolationLevel::Serializable
                || access_mode != AccessMode::ReadOnly)
        {
            panic!("Deferrable transaction can execute in serializable and read only");
        }

        // トランザクションを開始
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Repository(e.into()))?;
        // トランザクション分離モデルを設定
        let deferrable_token = if deferrable {
            "DEFERRABLE"
        } else {
            "NOT DEFERRABLE"
        };
        let sql = format!(
            "SET TRANSACTION ISOLATION LEVEL {} {} {}",
            isolation_level, access_mode, deferrable_token
        );
        sqlx::query(&sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Repository(e.into()))?;

        Ok(tx)
    }
}

/// トランザクションをコミットする。
///
/// # 引数
///
/// * `tx` - トランザクション
pub async fn commit_transaction(tx: PgTransaction<'_>) -> DomainResult<()> {
    tx.commit()
        .await
        .map_err(|e| DomainError::Repository(e.into()))
}

/// トランザクションをロールバックする。
///
/// FIXME: ロールバックをトランザクションのドロップでするなど、この関数を呼び出さない場合は削除する。
///
/// # 引数
///
/// * `tx` - トランザクション
#[allow(dead_code)]
async fn rollback_transaction(tx: PgTransaction<'_>) -> DomainResult<()> {
    tx.commit()
        .await
        .map_err(|e| DomainError::Repository(e.into()))
}

/// トランザクション分離レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsolationLevel {
    /// リード・アンコミッティッド
    ///
    /// PostgreSQLは、リード・アンコミッティッドをリード・コミットとして扱う。
    ReadUncommitted,

    /// リード・コミット
    ReadCommit,

    /// リピータブル・リード
    RepeatableRead,

    /// シリアライザブル
    Serializable,
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            IsolationLevel::ReadUncommitted => write!(f, "READ UNCOMMITTED"),
            IsolationLevel::ReadCommit => write!(f, "READ COMMITTED"),
            IsolationLevel::RepeatableRead => write!(f, "REPEATABLE READ"),
            IsolationLevel::Serializable => write!(f, "SERIALIZABLE"),
        }
    }
}

/// トランザクションのアクセス・モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessMode {
    /// 読み書き
    ReadWrite,
    /// 読み込み専用
    ReadOnly,
}

impl std::fmt::Display for AccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AccessMode::ReadWrite => write!(f, "READ WRITE"),
            AccessMode::ReadOnly => write!(f, "READ ONLY"),
        }
    }
}
