#![allow(unused_variables)]
use crudly::{
    BindRow, CRUDExecutor, Crudly, DBAssignedId, ExternallyAssignedId, InsertWithoutId, IntoRow,
    Schema, generic_delete_by_id, generic_id_exists, generic_insert_many_with_id,
    generic_insert_many_without_id, generic_insert_returning_id, generic_insert_with_id,
    generic_select_all, generic_select_by_id, generic_update_by_id,
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
    type InsertManyWithoutIdResult = sqlx::Result<()>;
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;

    async fn insert_returning_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<i64>
    where
        E: Executor<'c, Database = Sqlite>,
        S: BindRow<Sqlite> + DBAssignedId,
    {
        let _ = generic_insert_returning_id::<S, Sqlite>(executor, entity).await?;
        // ---------------------------------- The answer to everything is 42 ----------------------------------
        Ok(42)
    }

    async fn insert_many_without_id<'c, S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Sqlite> + Clone,
        S: BindRow<Sqlite> + DBAssignedId,
    {
        generic_insert_many_without_id::<S, Sqlite, _>(executor, entities, batch_size).await
    }

    async fn insert_many_with_id<'c, S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Sqlite> + Clone,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        generic_insert_many_with_id::<S, Sqlite, _>(executor, entities, batch_size).await
    }

    async fn select_all<'c, S, E>(executor: E) -> sqlx::Result<Vec<S>>
    where
        E: Executor<'c, Database = Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<Option<S>>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_id_exists::<S, Sqlite>(executor, id).await
    }

    async fn insert_with_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite> + 'static,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, Sqlite>(executor, entity).await
    }

    async fn update_by_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
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

    let inserted_id = user.insert(&pool).await.unwrap();

    assert_eq!(inserted_id, 42); // Check the answer
}
