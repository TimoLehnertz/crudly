use crate::{BindRow, DBAssignedId, ExternallyAssignedId, Schema};
use sqlx::{Database, Encode, Executor, FromRow, Type};
use std::future::Future;

mod generic;

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

pub use generic::*;

pub struct DefaultCRUDExecutor;

pub trait CRUDExecutor<DB: Database> {
    /// Most likely `sqlx::Result<()>` But one could also use the
    /// sql RETURNING clause to return the actual entity after it was inserted.
    type InsertWithIdResult;

    /// Most likely `sqlx::Result<bool>` But one could also use the
    /// sql RETURNING clause to return the actual entity after it was updated.
    type UpdateByIdResult;

    /// The result type of the delete operation.
    ///
    /// This could be `sqlx::Result<()>` or something else that additionally indicates if
    /// the entity was indeed deleted or didn't exist in the first place.
    type DeleteByIdResult;

    /// One might want to implement this in a way that returns all the inserted ids.
    type InsertManyWithoutIdResult;

    fn select_all<S>(
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

    /// `batch_size`: max rows per `INSERT`; `0` means one statement for the full input.
    fn insert_many_without_id<S>(
        entities: Vec<S>,
        batch_size: usize,
        executor: impl for<'e> Executor<'e, Database = DB> + Clone,
    ) -> impl Future<Output = Self::InsertManyWithoutIdResult>
    where
        S: BindRow<DB> + DBAssignedId;

    /// `batch_size`: max rows per `INSERT`; `0` means one statement for the full input.
    fn insert_many_with_id<S>(
        entities: Vec<S>,
        batch_size: usize,
        executor: impl for<'e> Executor<'e, Database = DB> + Clone,
    ) -> impl Future<Output = sqlx::Result<()>>
    where
        S::Id: for<'q> Encode<'q, DB> + Type<DB>,
        S: BindRow<DB> + ExternallyAssignedId;

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
