use crate::sql::{
    FormatPlaceholder, LastInsertedRowId, RowsAffected, reusable_executor::ReusableExecutor,
};
use crate::sql::{
    generic_delete_all, generic_delete_by_id, generic_id_exists, generic_insert_many_with_id,
    generic_insert_many_without_id, generic_insert_returning_id, generic_insert_with_id,
    generic_select_all, generic_select_all_no_id, generic_select_by_id, generic_select_by_ids,
    generic_update_by_id,
};
use crate::{
    CrudlyDefault, DBAssignedId, DeleteAll, DeleteById, ExternallyAssignedId, HasId, IdExists,
    Insert, InsertMany, InsertManyNoId, InsertManyWithoutIds, InsertNoId, InsertWithoutId, IntoRow,
    NoId, Schema, SelectAll, SelectAllNoId, SelectById, SelectByIds, UpdateById,
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

impl<T> SelectAll<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
{
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_select_all(executor).await }
    }
}

impl<T> DeleteAll<Sqlite> for T
where
    T: CrudlyDefault + Schema + Send,
{
    fn delete_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_delete_all::<Self, Sqlite>(executor).await }
    }
}

impl<T> SelectById<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_select_by_id(executor, id).await }
    }
}

impl<T> SelectByIds<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async move { generic_select_by_ids(executor, ids, batch_size).await }
    }
}

impl<T> IdExists<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_id_exists::<Self, Sqlite>(executor, id).await }
    }
}

impl<T> UpdateById<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Sqlite> + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_update_by_id(executor, self).await }
    }
}

impl<T> DeleteById<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
{
    fn delete_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_delete_by_id::<Self, Sqlite>(executor, id).await }
    }
}

impl<T> InsertWithoutId<Sqlite> for T
where
    T: CrudlyDefault + Schema + IntoRow<Sqlite> + DBAssignedId + Send,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_insert_returning_id::<Self, Sqlite>(executor, self).await }
    }
}

impl<T> InsertManyWithoutIds<Sqlite> for T
where
    T: CrudlyDefault + Schema + IntoRow<Sqlite> + DBAssignedId + Send,
{
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

impl<T> Insert<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Sqlite> + ExternallyAssignedId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
    <T as HasId>::Id: 'static,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_insert_with_id::<Self, Sqlite>(executor, self).await }
    }
}

impl<T> InsertMany<Sqlite> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Sqlite> + ExternallyAssignedId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Sqlite> + Type<Sqlite>,
    <T as HasId>::Id: 'static,
{
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

impl<T> SelectAllNoId<Sqlite> for T
where
    T: CrudlyDefault + Schema + NoId + for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
{
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async { generic_select_all_no_id(executor).await }
    }
}

impl<T> InsertNoId<Sqlite> for T
where
    T: CrudlyDefault + Schema + NoId + IntoRow<Sqlite> + Send,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = Sqlite>,
    {
        async move {
            crate::sql::generic_insert_without_id::<Self, Sqlite>(executor, self)
                .await
                .map(|_| ())
        }
    }
}

impl<T> InsertManyNoId<Sqlite> for T
where
    T: CrudlyDefault + Schema + NoId + IntoRow<Sqlite> + Send,
{
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<Sqlite> + Send,
    {
        async move {
            generic_insert_many_without_id::<Self, Sqlite, _>(executor, entities, batch_size).await
        }
    }
}
