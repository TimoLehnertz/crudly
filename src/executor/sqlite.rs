use crate::executor::reusable_executor::ReusableExecutor;
use crate::executor::{
    generic_delete_by_id, generic_insert_many_without_id, generic_insert_returning_id,
    generic_insert_with_id, generic_update_by_id,
};
use crate::{
    BindRow, Crudly, CrudlyDefault, DBAssignedId, ExternallyAssignedId, FormatPlaceholder,
    InsertWithId, InsertWithoutId, LastInsertedRowId, RowsAffected, Schema, generic_id_exists,
    generic_insert_many_with_id, generic_select_all, generic_select_by_id,
};
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Encode, Executor, FromRow, Sqlite, Type};
use std::future::Future;

impl FormatPlaceholder for Sqlite {
    fn format_placeholder(_idx: usize) -> String {
        "?".to_string()
    }
}

impl RowsAffected for SqliteQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

impl LastInsertedRowId for SqliteQueryResult {
    fn last_insert_rowid(&self) -> i64 {
        self.last_insert_rowid()
    }
}

impl<T> Crudly<Sqlite> for T
where
    T: CrudlyDefault<Sqlite>
        + Schema<Sqlite>
        + BindRow<Sqlite>
        + for<'r> FromRow<'r, SqliteRow>
        + Unpin,
    for<'q> <T as Schema<Sqlite>>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    type Id = <Self as Schema<Sqlite>>::Id;

    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_select_all(executor).await }
    }

    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_select_by_id(executor, id).await }
    }

    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_id_exists::<Self, Sqlite>(executor, id).await }
    }

    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_update_by_id(executor, self).await }
    }

    fn delete_by_id<'c, E>(id: &Self::Id, executor: E) -> impl Future<Output = sqlx::Result<bool>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_delete_by_id::<Self, Sqlite>(executor, id).await }
    }
}

impl<T> InsertWithoutId<Sqlite> for T
where
    T: CrudlyDefault<Sqlite> + Schema<Sqlite> + BindRow<Sqlite> + DBAssignedId,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_insert_returning_id::<Self, Sqlite>(executor, self).await }
    }

    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<Sqlite> + Send,
    {
        // This can't be refactored to an async fn because that triggers the following when used in certain scenarios:
        // lifetime bound not satisfied
        // this is a known limitation that will be removed in the future (see issue #100013 <https://github.com/rust-lang/rust/issues/100013> for more information)
        async move {
            generic_insert_many_without_id::<Self, Sqlite, _>(executor, entities, batch_size).await
        }
    }
}

impl<T> InsertWithId<Sqlite> for T
where
    T: CrudlyDefault<Sqlite> + Schema<Sqlite> + BindRow<Sqlite> + ExternallyAssignedId,
    for<'q> <T as Schema<Sqlite>>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
    <T as Schema<Sqlite>>::Id: 'static,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_insert_with_id::<Self, Sqlite>(executor, self).await }
    }

    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>>
    where
        E: ReusableExecutor<Sqlite> + Send,
    {
        async move {
            generic_insert_many_with_id::<Self, Sqlite, _>(executor, entities, batch_size).await
        }
    }
}
