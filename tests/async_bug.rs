use crudly::sql::{FormatPlaceholder, LastInsertedRowId, generic_insert_returning_id};
use crudly::{
    CrudlyDefault, DeleteById, IdExists, Insert, InsertWithoutId, IntoRow, Schema, SelectAll,
    SelectById, UpdateById,
};
use sqlx::{Database, Executor, IntoArguments, prelude::FromRow};

#[derive(IntoRow, Schema)]
#[allow(dead_code)]
struct Bar {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}

impl<DB: Database> Foo<DB> for Bar
where
    DB: FormatPlaceholder,
    DB::QueryResult: LastInsertedRowId,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
    for<'q> std::string::String: sqlx::Encode<'q, DB>,
    std::string::String: sqlx::Type<DB>,
{
    fn insert<'e, 'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        'c: 'e,
        DB: 'e,
        E: 'e + Executor<'c, Database = DB>,
    {
        // sqlx::query("SELECT 1").execute(executor).await?;
        async { generic_insert_returning_id::<Self, DB>(executor, self).await }
        // Ok(1)
    }
}

pub trait Foo<DB: Database>: Send {
    fn insert<'e, 'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        'c: 'e,
        DB: 'e,
        E: 'e + Executor<'c, Database = DB>;
}

fn _do_stuff(bar: Bar, pool: sqlx::SqlitePool) {
    tokio::spawn(async move {
        // This also used to not compile
        let mut con = pool.acquire().await.unwrap();
        bar.insert(&mut *con).await.unwrap();
    });
}

#[derive(FromRow, IntoRow, Schema, Default)]
pub struct User {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}

impl CrudlyDefault for User {}

#[derive(FromRow, IntoRow, Schema, Default)]
#[crudly(external_ids)]
pub struct UserExternalIds {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}

impl CrudlyDefault for UserExternalIds {}

/// This used to not compile with the very unhelpful error: lifetime bound not satisfied
/// this is a known limitation that will be removed in the future (see issue #100013 <https://github.com/rust-lang/rust/issues/100013> for more information)
/// The compiler error was resolved by using trait methods returning `impl Future<Output = ...> + Send`.
async fn _should_compile(pool: sqlx::SqlitePool) {
    tokio::spawn(async move {
        let mut con = pool.acquire().await.unwrap();

        User::default().insert(&mut *con).await.unwrap();
        UserExternalIds::default().insert(&mut *con).await.unwrap();
        User::default().update_by_id(&mut *con).await.unwrap();
        User::delete_by_id(&1, &mut *con).await.unwrap();
        User::id_exists(&1, &mut *con).await.unwrap();
        User::select_all(&mut *con).await.unwrap();
        User::select_by_id(&1, &mut *con).await.unwrap();

        // The fix is not applied for insert_many because it didn't work there for god knows why...
        // User::insert_many(vec![User::default()], 10, &pool)
        //     .await
        //     .unwrap();
    });
}
