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
    type InsertManyWithoutIdResult = sqlx::Result<()>;

    async fn select_all<'c, S, E>(executor: E) -> sqlx::Result<Vec<S>>
    where
        E: Executor<'c, Database = Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        log("select_all");
        DefaultCRUDExecutor::select_all::<S, _>(executor).await
    }

    async fn select_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<Option<S>>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        log("select_by_id");
        DefaultCRUDExecutor::select_by_id::<S, _>(id, executor).await
    }

    async fn id_exists<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        log("id_exists");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::id_exists::<S, _>(id, executor).await
    }

    async fn insert_with_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite> + 'static,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        log("insert_with_id");
        DefaultCRUDExecutor::insert_with_id::<S, _>(entity, executor).await
    }

    async fn insert_returning_id<'e, 'c, S, E>(entity: S, executor: E) -> sqlx::Result<i64>
    where
        'c: 'e,
        E: 'e + Executor<'c, Database = Sqlite>,
        S: BindRow<Sqlite> + DBAssignedId,
    {
        log("insert");
        DefaultCRUDExecutor::insert_returning_id::<S, _>(entity, executor).await
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
        log("insert_many_without_id");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::insert_many_without_id::<S, _>(
            entities, batch_size, executor,
        )
        .await
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
        log("insert_many_with_id");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::insert_many_with_id::<S, _>(
            entities, batch_size, executor,
        )
        .await
    }

    async fn update_by_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite>,
    {
        log("update_by_id");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::update_by_id::<S, _>(entity, executor).await
    }

    async fn delete_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        log("delete_by_id");
        <DefaultCRUDExecutor as CRUDExecutor<Sqlite>>::delete_by_id::<S, _>(id, executor).await
    }
}
