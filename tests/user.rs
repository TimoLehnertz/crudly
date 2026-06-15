//! This file is a sandbox for testing what the derived code should look like.
use crudly::sql::{
    ReusableExecutor, generic_delete_by_id, generic_id_exists, generic_insert_many_with_id,
    generic_insert_many_without_id, generic_insert_returning_id, generic_insert_with_id,
    generic_select_all, generic_select_by_id, generic_select_by_ids, generic_update_by_id,
};
use crudly::{
    DBAssignedId, DeleteById, ExternallyAssignedId, HasColumns, HasId, IdExists, Insert,
    InsertMany, InsertManyWithoutIds, InsertWithoutId, IntoRow, Schema, SelectAll, SelectById,
    SelectByIds, UpdateById,
};
use serde::Serialize;
use sqlx::sqlite::{Sqlite, SqlitePool};
use sqlx::{Arguments, Database, Executor, FromRow, SqliteConnection};
use sqlx::{Encode, Type, query};

#[derive(FromRow, Default)]
pub struct Address {
    pub street: String,
    pub city: String,
}

#[derive(Serialize, Default)]
pub struct JsonObject {
    pub name: String,
}

#[derive(Default, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    #[sqlx(flatten)]
    pub address: Address,
    pub email: String,
    #[sqlx(json)]
    pub json: JsonObject,
}

impl HasColumns for User {
    fn columns() -> Vec<&'static str> {
        let mut columns = Vec::new();
        columns.push("name");
        columns.extend(Address::columns());
        columns.push("email");
        columns.push("json");
        columns
    }
}

impl IntoRow<Sqlite> for User
where
    for<'q> String: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn bind_arguments(self, arguments: &mut <Sqlite as Database>::Arguments) -> sqlx::Result<()> {
        arguments.add(self.name).map_err(sqlx::Error::Encode)?;
        self.address.bind_arguments(arguments)?;
        arguments.add(self.email).map_err(sqlx::Error::Encode)?;
        arguments
            .add(sqlx::types::Json(self.json))
            .map_err(sqlx::Error::Encode)?;
        Ok(())
    }
}

impl HasColumns for Address {
    fn columns() -> Vec<&'static str> {
        vec!["street", "city"]
    }
}

impl IntoRow<Sqlite> for Address
where
    for<'q> String: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn bind_arguments(self, arguments: &mut <Sqlite as Database>::Arguments) -> sqlx::Result<()> {
        arguments.add(self.street).map_err(sqlx::Error::Encode)?;
        arguments.add(self.city).map_err(sqlx::Error::Encode)?;
        Ok(())
    }
}

impl Schema for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn columns() -> Vec<&'static str> {
        let mut cols = vec!["id"];
        cols.extend(<Self as HasColumns>::columns());
        cols
    }
}

impl HasId for User {
    type Id = i64;

    fn id_column() -> &'static str {
        "id"
    }

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl DBAssignedId for User {}

impl ExternallyAssignedId for User {}

impl SelectAll<Sqlite> for User
where
    User: Schema + HasId + IntoRow<Sqlite>,
    Self: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin + Send,
{
    async fn select_all<'c, E>(executor: E) -> sqlx::Result<Vec<Self>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_select_all(executor).await
    }
}

impl DeleteById<Sqlite> for User
where
    User: Schema + HasId,
    for<'q> <User as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    async fn delete_by_id<'c, E>(id: &Self::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_delete_by_id::<Self, Sqlite>(executor, id).await
    }
}

impl IdExists<Sqlite> for User
where
    User: Schema + HasId,
    for<'q> <User as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    async fn id_exists<'c, E>(id: &Self::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_id_exists::<Self, Sqlite>(executor, id).await
    }
}

impl SelectById<Sqlite> for User
where
    User: Schema + HasId,
    Self: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin + Send,
    for<'q> <User as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    async fn select_by_id<'c, E>(id: &Self::Id, executor: E) -> sqlx::Result<Option<Self>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_select_by_id(executor, id).await
    }
}

impl SelectByIds<Sqlite> for User
where
    User: Schema + HasId,
    Self: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin + Send,
    for<'q> <User as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    async fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<Vec<Self>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_select_by_ids(executor, ids, batch_size).await
    }
}

impl UpdateById<Sqlite> for User
where
    User: Schema + HasId + IntoRow<Sqlite>,
    for<'q> <User as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    async fn update_by_id<'c, E>(self, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_update_by_id(executor, self).await
    }
}

impl InsertWithoutId<Sqlite> for User
where
    User: Schema + IntoRow<Sqlite>,
{
    async fn insert<'c, E>(self, executor: E) -> sqlx::Result<i64>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_insert_returning_id::<Self, Sqlite>(executor, self).await
    }
}

impl InsertManyWithoutIds<Sqlite> for User
where
    User: Schema + IntoRow<Sqlite>,
{
    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<Sqlite> + Send,
    {
        generic_insert_many_without_id::<Self, Sqlite, _>(executor, entities, batch_size).await
    }
}

impl Insert<Sqlite> for User
where
    <User as HasId>::Id: sqlx::Type<Sqlite>,
    for<'q> <User as HasId>::Id: sqlx::Encode<'q, Sqlite>,
    User: Schema + HasId + IntoRow<Sqlite>,
{
    async fn insert<'c, E>(self, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        generic_insert_with_id::<Self, Sqlite>(executor, self).await
    }
}

impl InsertMany<Sqlite> for User
where
    <User as HasId>::Id: sqlx::Type<Sqlite>,
    for<'q> <User as HasId>::Id: sqlx::Encode<'q, Sqlite>,
    User: Schema + HasId + IntoRow<Sqlite>,
{
    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<Sqlite> + Send,
    {
        generic_insert_many_with_id::<Self, Sqlite, _>(executor, entities, batch_size).await
    }
}

/// Returns a pool backed by a private in-memory database.
/// `cache=shared` keeps a single DB for every connection taken from the pool.
async fn sqlite_memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:?cache=shared")
        .await
        .unwrap();

    query("CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL, street TEXT NOT NULL, city TEXT NOT NULL, email TEXT NOT NULL, json TEXT NOT NULL);")
    .execute(&pool).await.unwrap();

    pool
}

#[tokio::test]
async fn test_use_con_as_impl_executor() {
    let pool = sqlite_memory_pool().await;
    let mut con = pool.acquire().await.unwrap();
    let con: &mut SqliteConnection = &mut *con;

    let first_user_id = InsertWithoutId::insert(User::default(), con).await.unwrap();
    assert_eq!(first_user_id, 1);
}

#[tokio::test]
async fn test_use_pool_con_as_impl_executor() {
    let pool = sqlite_memory_pool().await;
    let mut con = pool.acquire().await.unwrap();

    let first_user_id = InsertWithoutId::insert(User::default(), &mut *con)
        .await
        .unwrap();
    assert_eq!(first_user_id, 1);
}

#[tokio::test]
async fn insert_user_returning_id() {
    let pool = sqlite_memory_pool().await;

    let first_user_id = InsertWithoutId::insert(User::default(), &pool)
        .await
        .unwrap();
    assert_eq!(first_user_id, 1);

    let second_user_id = InsertWithoutId::insert(User::default(), &pool)
        .await
        .unwrap();

    assert_eq!(second_user_id, 2);
}

#[tokio::test]
async fn insert_many_users_without_id() {
    let pool = sqlite_memory_pool().await;

    InsertManyWithoutIds::insert_many(vec![User::default(), User::default()], 0, &pool)
        .await
        .unwrap();

    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 2);

    let max_id: i64 = sqlx::query_scalar("SELECT MAX(id) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(max_id, 2);
}
