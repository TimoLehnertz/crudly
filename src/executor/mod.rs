use crate::{BindRow, DBAssignedId, ExternallyAssignedId, Schema};
use sqlx::{Database, Encode, Executor, FromRow, Type};
use std::future::Future;

mod generic;
pub mod reusable_executor;
pub use reusable_executor::ReusableExecutor;

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

    fn select_all<'c, S, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<S>>> + Send
    where
        E: Executor<'c, Database = DB>,
        S: Schema<DB> + for<'r> FromRow<'r, DB::Row> + Unpin;

    fn select_by_id<'c, S, E>(
        id: &S::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<S>>> + Send
    where
        E: Executor<'c, Database = DB>,
        S::Id: for<'q> sqlx::Encode<'q, DB> + Type<DB>,
        S: Schema<DB> + for<'r> FromRow<'r, DB::Row> + Unpin;

    fn id_exists<'c, S, E>(
        id: &S::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = DB>,
        S::Id: for<'q> sqlx::Encode<'q, DB> + Type<DB>,
        S: Schema<DB>;

    fn insert_with_id<'c, S, E>(
        entity: S,
        executor: E,
    ) -> impl Future<Output = Self::InsertWithIdResult> + Send
    where
        E: Executor<'c, Database = DB>,
        S::Id: for<'q> Encode<'q, DB> + Type<DB> + 'static,
        S: BindRow<DB> + ExternallyAssignedId;

    fn insert_returning_id<'e, 'c, S, E>(
        entity: S,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        'c: 'e,
        E: 'e + Executor<'c, Database = DB>,
        S: BindRow<DB> + DBAssignedId;

    /// `batch_size`: max rows per `INSERT`; `0` means one statement for the full input.
    fn insert_many_without_id<S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = Self::InsertManyWithoutIdResult> + Send
    where
        E: ReusableExecutor<DB> + Send,
        S: BindRow<DB> + DBAssignedId;

    /// `batch_size`: max rows per `INSERT`; `0` means one statement for the full input.
    fn insert_many_with_id<S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<DB> + Send,
        S::Id: for<'q> Encode<'q, DB> + Type<DB>,
        S: BindRow<DB> + ExternallyAssignedId;

    fn update_by_id<'c, S, E>(
        entity: S,
        executor: E,
    ) -> impl Future<Output = Self::UpdateByIdResult> + Send
    where
        E: Executor<'c, Database = DB>,
        S::Id: for<'q> Encode<'q, DB> + Type<DB>,
        S: BindRow<DB>;

    fn delete_by_id<'c, S, E>(
        id: &S::Id,
        executor: E,
    ) -> impl Future<Output = Self::DeleteByIdResult> + Send
    where
        E: Executor<'c, Database = DB>,
        S::Id: for<'q> Encode<'q, DB> + Type<DB>,
        S: Schema<DB>;
}
