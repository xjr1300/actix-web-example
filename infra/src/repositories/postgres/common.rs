use std::marker::PhantomData;

use sqlx::PgPool;

/// PostgreSQLリポジトリ構造体
#[derive(Debug)]
pub struct PgRepository<T> {
    /// データベース接続プール
    pub pool: PgPool,
    /// マーカー。
    _phantom: PhantomData<T>,
}

impl<T> PgRepository<T> {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            _phantom: Default::default(),
        }
    }
}
