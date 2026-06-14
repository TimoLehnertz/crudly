use sqlx::Database;
use sqlx::Executor;
use sqlx::IntoArguments;
use sqlx::Pool;
use sqlx::query_with;
use std::future::Future;
use std::pin::Pin;

type ReusableExecutorFuture<'a> = Pin<Box<dyn Future<Output = sqlx::Result<()>> + Send + 'a>>;

pub trait ReusableExecutor<DB: Database> {
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: DB::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a;
}

#[cfg(feature = "sqlite")]
impl ReusableExecutor<sqlx::Sqlite> for &mut sqlx::SqliteConnection {
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: <sqlx::Sqlite as Database>::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a,
    {
        Box::pin(async move {
            query_with(sql, arguments).execute(&mut **self).await?;
            Ok(())
        })
    }
}

#[cfg(feature = "postgres")]
impl ReusableExecutor<sqlx::Postgres> for &mut sqlx::PgConnection {
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: <sqlx::Postgres as Database>::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a,
    {
        Box::pin(async move {
            query_with(sql, arguments).execute(&mut **self).await?;
            Ok(())
        })
    }
}

#[cfg(feature = "mysql")]
impl ReusableExecutor<sqlx::MySql> for &mut sqlx::MySqlConnection {
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: <sqlx::MySql as Database>::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a,
    {
        Box::pin(async move {
            query_with(sql, arguments).execute(&mut **self).await?;
            Ok(())
        })
    }
}

impl<DB: Database> ReusableExecutor<DB> for Pool<DB>
where
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
    for<'e> &'e mut DB::Connection: Executor<'e, Database = DB>,
{
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: DB::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a,
    {
        Box::pin(async move {
            query_with(sql, arguments).execute(&*self).await?;
            Ok(())
        })
    }
}

impl<DB: Database> ReusableExecutor<DB> for &Pool<DB>
where
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
    for<'e> &'e mut DB::Connection: Executor<'e, Database = DB>,
{
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: DB::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a,
    {
        Box::pin(async move {
            query_with(sql, arguments).execute(*self).await?;
            Ok(())
        })
    }
}

impl<DB: Database> ReusableExecutor<DB> for &mut Pool<DB>
where
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
    for<'e> &'e mut DB::Connection: Executor<'e, Database = DB>,
{
    fn execute_query_with<'a, 'q>(
        &'a mut self,
        sql: &'q str,
        arguments: DB::Arguments<'q>,
    ) -> ReusableExecutorFuture<'a>
    where
        'q: 'a,
    {
        Box::pin(async move {
            query_with(sql, arguments).execute(&**self).await?;
            Ok(())
        })
    }
}
