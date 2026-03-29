//! Test double that records [`CRUDExecutor`] calls and delegates to [`DefaultCRUDExecutor`].
#![allow(dead_code)]

use std::sync::Mutex;

use crudly::{
    BindRow, CRUDExecutor, DBAssignedId, DefaultCRUDExecutor, ExternallyAssignedId, Schema,
};
use sqlx::sqlite::SqliteRow;
use sqlx::{Encode, Executor, FromRow, Sqlite, Type};

static CALL_LOG: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

pub fn clear_log() {
    CALL_LOG.lock().unwrap().clear();
}

pub fn take_log() -> Vec<&'static str> {
    std::mem::take(&mut *CALL_LOG.lock().unwrap())
}

fn log(name: &'static str) {
    CALL_LOG.lock().unwrap().push(name);
}

#[derive(Debug, Default)]
pub struct MockCrudExecutor;

impl CRUDExecutor<Sqlite> for MockCrudExecutor {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;

    async fn select_all<S>(
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Vec<S>>
    where
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        log("select_all");
        DefaultCRUDExecutor::select_all(executor).await
    }

    async fn select_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Option<S>>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        log("select_by_id");
        DefaultCRUDExecutor::select_by_id(id, executor).await
    }

    async fn id_exists<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        log("id_exists");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::id_exists::<S>(id, executor).await
    }

    async fn insert_with_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<()>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite> + 'static,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        log("insert_with_id");
        DefaultCRUDExecutor::insert_with_id(entity, executor).await
    }

    async fn insert_returning_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<i64>
    where
        S: BindRow<Sqlite> + DBAssignedId,
    {
        log("insert_returning_id");
        DefaultCRUDExecutor::insert_returning_id(entity, executor).await
    }

    async fn update_by_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite>,
    {
        log("update_by_id");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::update_by_id::<S>(entity, executor).await
    }

    async fn delete_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        log("delete_by_id");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::delete_by_id::<S>(id, executor).await
    }
}
