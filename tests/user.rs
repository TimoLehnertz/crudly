use crudly::{
    BindRow, CRUDExecutor, Crudly, DBAssignedId, DefaultCRUDExecutor, ExternallyAssignedId,
    HasColumns, InsertReturningId, InsertWithId, IntoRow, Schema,
};
use sqlx::sqlite::{Sqlite, SqliteArguments, SqlitePool};
use sqlx::{Arguments, Database, Executor, FromRow};
use sqlx::{Encode, IntoArguments, Type, query};

#[derive(FromRow, Default)]
pub struct Address {
    pub street: String,
    pub city: String,
}

#[derive(Default, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    #[sqlx(flatten)]
    pub address: Address,
    pub email: String,
}

impl<'a> IntoRow<SqliteArguments<'a>> for User
where
    SqliteArguments<'a>: IntoArguments<'a, Sqlite>,
    String: Encode<'a, Sqlite> + Type<Sqlite>,
{
    fn bind_arguments(self, arguments: &mut SqliteArguments<'a>) -> sqlx::Result<()> {
        arguments.add(self.name).map_err(sqlx::Error::Encode)?;
        self.address.bind_arguments(arguments)?;
        arguments.add(self.email).map_err(sqlx::Error::Encode)?;
        Ok(())
    }
}

impl<'a> IntoRow<SqliteArguments<'a>> for Address
where
    SqliteArguments<'a>: IntoArguments<'a, Sqlite>,
    String: Encode<'a, Sqlite> + Type<Sqlite>,
{
    fn bind_arguments(self, arguments: &mut SqliteArguments<'a>) -> sqlx::Result<()> {
        arguments.add(self.street).map_err(sqlx::Error::Encode)?;
        arguments.add(self.city).map_err(sqlx::Error::Encode)?;
        Ok(())
    }
}

impl HasColumns for User {
    fn columns() -> Vec<&'static str> {
        let mut columns = Vec::new();
        columns.push("name");
        columns.extend(Address::columns());
        columns.push("email");
        columns
    }
}

impl HasColumns for Address {
    fn columns() -> Vec<&'static str> {
        let mut columns = Vec::new();
        columns.push("street");
        columns.push("city");
        columns
    }
}

impl<DB: Database> Schema<DB> for User {
    type Id = i64;

    fn table_name() -> &'static str {
        "users"
    }

    fn id_column() -> &'static str {
        "id"
    }

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl DBAssignedId for User {}

impl ExternallyAssignedId for User {}

impl Crudly<Sqlite> for User
where
    User: Schema<Sqlite> + BindRow<Sqlite>,
    DefaultCRUDExecutor<Sqlite>: CRUDExecutor<Sqlite>,
    Self: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>,
    for<'q> <User as Schema<Sqlite>>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    type Id = <Self as Schema<Sqlite>>::Id;
    type UpdateByIdResult = <DefaultCRUDExecutor<Sqlite> as CRUDExecutor<Sqlite>>::UpdateByIdResult;
    type DeleteByIdResult = <DefaultCRUDExecutor<Sqlite> as CRUDExecutor<Sqlite>>::DeleteByIdResult;

    async fn find_all(
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Vec<Self>> {
        DefaultCRUDExecutor::<Sqlite>::find_all::<Self>(executor).await
    }

    async fn delete_by_id(
        id: &Self::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> Self::DeleteByIdResult {
        DefaultCRUDExecutor::<Sqlite>::delete_by_id::<Self>(id, executor).await
    }

    async fn id_exists(
        id: &Self::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool> {
        DefaultCRUDExecutor::<Sqlite>::id_exists::<Self>(id, executor).await
    }

    async fn select_by_id(
        id: &Self::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Option<Self>> {
        DefaultCRUDExecutor::<Sqlite>::select_by_id::<Self>(id, executor).await
    }

    async fn update_by_id(
        entity: Self,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> Self::UpdateByIdResult {
        DefaultCRUDExecutor::<Sqlite>::update_by_id::<Self>(entity, executor).await
    }
}

impl InsertReturningId<Sqlite> for User
where
    User: Schema<Sqlite> + BindRow<Sqlite>,
    DefaultCRUDExecutor<Sqlite>: CRUDExecutor<Sqlite>,
{
    async fn insert_returning_id(
        self,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<i64> {
        DefaultCRUDExecutor::<Sqlite>::insert_returning_id::<Self>(self, executor).await
    }
}

impl InsertWithId<Sqlite> for User
where
    <User as Schema<Sqlite>>::Id: sqlx::Type<Sqlite>,
    for<'q> <User as Schema<Sqlite>>::Id: sqlx::Encode<'q, Sqlite>,
    User: Schema<Sqlite> + BindRow<Sqlite>,
    DefaultCRUDExecutor<Sqlite>: CRUDExecutor<Sqlite>,
{
    type Result = <DefaultCRUDExecutor<Sqlite> as CRUDExecutor<Sqlite>>::InsertWithIdResult;

    async fn insert_with_id(
        self,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> Self::Result {
        DefaultCRUDExecutor::<Sqlite>::insert_with_id::<Self>(self, executor).await
    }
}

/// Returns a pool backed by a private in-memory database.
/// `cache=shared` keeps a single DB for every connection taken from the pool.
async fn sqlite_memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:?cache=shared")
        .await
        .unwrap();

    query("CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL, street TEXT NOT NULL, city TEXT NOT NULL, email TEXT NOT NULL);")
    .execute(&pool).await.unwrap();

    pool
}

#[tokio::test]
async fn insert_user_returning_id() {
    let pool = sqlite_memory_pool().await;

    let first_user_id = User::default().insert_returning_id(&pool).await.unwrap();
    assert_eq!(first_user_id, 1);

    let second_user_id = User::default().insert_returning_id(&pool).await.unwrap();

    assert_eq!(second_user_id, 2);
}
