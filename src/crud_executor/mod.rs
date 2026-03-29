use crate::{BindRow, DBAssignedId, ExternallyAssignedId, Schema};
use sqlx::{Database, Encode, Executor, FromRow, Type};
use std::future::Future;
use std::marker::PhantomData;

mod generic;

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

pub use generic::*;

pub struct DefaultCRUDExecutor<DB: Database> {
    _db: PhantomData<DB>,
}

pub trait CRUDExecutor<DB: Database> {
    /// Most likely sqlx::Result<()> But one could also use the
    /// sql RETURNING clause to return the actual entity after it was inserted.
    type InsertWithIdResult;

    /// Most likely sqlx::Result<bool> But one could also use the
    /// sql RETURNING clause to return the actual entity after it was updated.
    type UpdateByIdResult;

    /// The result type of the delete operation.
    ///
    /// This could be sqlx::Result<()> or something else that additionally indicates if
    /// the entity was indeed deleted or didn't exist in the first place.
    type DeleteByIdResult;

    fn find_all<S>(
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<Vec<S>>>
    where
        S: Schema<DB> + for<'r> FromRow<'r, DB::Row> + Unpin;

    fn select_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<Option<S>>>
    where
        S::Id: for<'q> sqlx::Encode<'q, DB> + Type<DB>,
        S: Schema<DB> + for<'r> FromRow<'r, DB::Row> + Unpin;

    fn id_exists<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<bool>>
    where
        S::Id: for<'q> sqlx::Encode<'q, DB> + Type<DB>,
        S: Schema<DB>;

    fn insert_with_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = Self::InsertWithIdResult>
    where
        S::Id: for<'q> Encode<'q, DB> + Type<DB> + 'static,
        S: BindRow<DB> + ExternallyAssignedId;

    fn insert_returning_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<i64>>
    where
        S: BindRow<DB> + DBAssignedId;

    // todo: add insert_many and delete_many

    fn update_by_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = Self::UpdateByIdResult>
    where
        S::Id: for<'q> Encode<'q, DB> + Type<DB>,
        S: BindRow<DB>;

    fn delete_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = Self::DeleteByIdResult>
    where
        S::Id: for<'q> Encode<'q, DB> + Type<DB>,
        S: Schema<DB>;
}
