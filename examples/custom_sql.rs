use crudly::sql::generic_insert_returning_id;
use crudly::{DBAssignedId, IntoRow, Schema};
use sqlx::{FromRow, Sqlite, SqliteExecutor, SqlitePool, query};
use std::future::Future;

const SQL_SCHEMA: &str =
    "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL);";

#[derive(FromRow, IntoRow, Schema)]
struct User {
    #[crudly(id)]
    id: i64,
    name: String,
}

/// Users can define their own local trait and implement it as a blanket impl.
trait MyCustomInsert: Sized {
    fn insert<'e>(
        self,
        executor: impl SqliteExecutor<'e>,
    ) -> impl Future<Output = sqlx::Result<i64>> + Send;
}

impl<T> MyCustomInsert for T
where
    T: Schema + IntoRow<Sqlite> + DBAssignedId,
{
    fn insert<'e>(
        self,
        executor: impl SqliteExecutor<'e>,
    ) -> impl Future<Output = sqlx::Result<i64>> + Send {
        async move {
            let _inserted_id = generic_insert_returning_id::<T, Sqlite>(executor, self).await?;
            // ... why are all my ids 42?
            Ok(42)
        }
    }
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    query(SQL_SCHEMA).execute(&pool).await.unwrap();

    let user = User {
        id: 0,
        name: "John Doe".to_string(),
    };

    let inserted_id = user.insert(&pool).await.unwrap();
    assert_eq!(inserted_id, 42);
}
