use common::sqlite;
use crudly::{CrudlyDefault, Schema, SelectAll, SelectById, SelectByIds};
use sqlx::prelude::FromRow;

mod common;

/// A struct that doesn't have a single id column.
#[derive(Clone, PartialEq, Debug, FromRow, Schema)]
pub struct User {
    #[crudly(id)]
    id: i64,
    name: String,
}

impl CrudlyDefault for User {}

/// Test that even without implementing IntoRow, the select crudly helpers still work.
#[tokio::test]
async fn test_student_x_course() {
    let pool = sqlite::memory_with_schema(
        "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL);",
    )
    .await;

    let all = User::select_all(&pool).await.unwrap();
    assert!(all.is_empty());

    let by_id = User::select_by_id(&1, &pool).await.unwrap();
    assert!(by_id.is_none());

    let by_ids = User::select_by_ids(vec![1], 0, &pool).await.unwrap();
    assert!(by_ids.is_empty());
}
