#![allow(unused_variables)]
use crudly::{CrudlyDefault, DBAssignedId, IntoRow, Schema, generic_insert_returning_id};
use sqlx::{Database, Executor, FromRow, Sqlite, SqlitePool, query};
use std::future::Future;

const CREATE_USERS_TABLE_SQL: &str =
    "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL);";

#[derive(FromRow, IntoRow, Schema)]
struct User {
    #[crudly(id)]
    id: i64,
    name: String,
}

impl CrudlyDefault<Sqlite> for User {}

/// Users can define their own local trait and implement it as a blanket impl.
trait InsertWithTheAnswerToEverything<DB: Database>: Sized {
    fn insert_with_answer<'e, 'c, E>(
        self,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        'c: 'e,
        DB: 'e,
        E: 'e + Executor<'c, Database = DB>;
}

impl<T> InsertWithTheAnswerToEverything<Sqlite> for T
where
    T: Schema + IntoRow<Sqlite> + DBAssignedId,
{
    fn insert_with_answer<'e, 'c, E>(
        self,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        'c: 'e,
        Sqlite: 'e,
        E: 'e + Executor<'c, Database = Sqlite>,
    {
        async move {
            let _ = generic_insert_returning_id::<T, Sqlite>(executor, self).await?;
            Ok(42)
        }
    }
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    query(CREATE_USERS_TABLE_SQL).execute(&pool).await.unwrap();

    let user = User {
        id: 0,
        name: "John Doe".to_string(),
    };

    let inserted_id = user.insert_with_answer(&pool).await.unwrap();
    assert_eq!(inserted_id, 42);
}
