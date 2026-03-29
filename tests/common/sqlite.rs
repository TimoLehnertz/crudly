//! In-memory SQLite helpers for integration tests (each pool is an isolated database).

#![allow(dead_code)]

use sqlx::sqlite::SqlitePool;

/// Open a pool on a private `:memory:` database (not shared with other pools).
pub async fn connect_memory() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect sqlite memory pool")
}

/// Connect, run `sql` (typically `CREATE TABLE …`), and return the pool.
pub async fn memory_with_schema(sql: &str) -> SqlitePool {
    let pool = connect_memory().await;
    sqlx::query(sql)
        .execute(&pool)
        .await
        .expect("run sqlite schema DDL");
    pool
}
