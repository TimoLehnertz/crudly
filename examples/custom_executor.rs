#![allow(unused_variables)]
use crudly::{
    BindRow, CRUDExecutor, Crudly, DBAssignedId, ExternallyAssignedId, InsertReturningId, IntoRow,
    Schema, generic_delete_by_id, generic_id_exists, generic_insert_returning_id,
    generic_insert_with_id, generic_select_all, generic_select_by_id, generic_update_by_id,
};
use sqlx::{Encode, Executor, FromRow, Sqlite, SqlitePool, Type, query, sqlite::SqliteRow};

const CREATE_USERS_TABLE_SQL: &str =
    "CREATE TABLE users (id INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL);";

#[derive(FromRow, IntoRow, Crudly)]
#[crudly(executor = ExecutorWithTheAnswerToEverything)] // use the custom executor for Users.
struct User {
    #[crudly(id)]
    id: i64,
    name: String,
}

struct ExecutorWithTheAnswerToEverything;

/// You can implement CRUDExecutor for multiple databases just like done with [crudly::DefaultCRUDExecutor].
impl CRUDExecutor<Sqlite> for ExecutorWithTheAnswerToEverything {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;

    async fn insert_returning_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<i64>
    where
        S: BindRow<Sqlite> + DBAssignedId,
    {
        let _ = generic_insert_returning_id::<S, Sqlite>(executor, entity).await?;
        // ---------------------------------- The answer to everything is 42 ----------------------------------
        Ok(42)
    }

    async fn select_all<S>(
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Vec<S>>
    where
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Option<S>>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_id_exists::<S, Sqlite>(executor, id).await
    }

    async fn insert_with_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<()>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite> + 'static,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, Sqlite>(executor, entity).await
    }

    async fn update_by_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_delete_by_id::<S, Sqlite>(executor, id).await
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

    let inserted_id = user.insert_returning_id(&pool).await.unwrap();

    assert_eq!(inserted_id, 42); // Check the answer
}
