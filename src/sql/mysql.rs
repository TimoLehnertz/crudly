use crate::sql::{
    FormatPlaceholder, LastInsertedRowId, RowsAffected, reusable_executor::ReusableExecutor,
};
use crate::sql::{
    generic_delete_all, generic_delete_by_id, generic_id_exists, generic_insert_many_with_id,
    generic_insert_many_without_id, generic_insert_returning_id, generic_insert_with_id,
    generic_insert_without_id, generic_select_all, generic_select_all_no_id, generic_select_by_id,
    generic_select_by_ids, generic_update_by_id,
};
use crate::{
    CrudlyDefault, DBAssignedId, DeleteAll, DeleteById, ExternallyAssignedId, HasId, IdExists,
    Insert, InsertMany, InsertManyNoId, InsertManyWithoutIds, InsertNoId, InsertWithoutId, IntoRow,
    NoId, Schema, SelectAll, SelectAllNoId, SelectById, SelectByIds, UpdateById,
};
use sqlx::MySql;
use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use sqlx::{Encode, Executor, FromRow, Type};
use std::future::Future;

impl FormatPlaceholder for MySql {
    fn format_placeholder(_idx: usize) -> String {
        "?".to_string()
    }
}

impl RowsAffected for MySqlQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

impl LastInsertedRowId for MySqlQueryResult {
    fn last_insert_rowid(&self) -> i64 {
        self.last_insert_id() as i64
    }
}

impl<T> SelectAll<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
{
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_select_all(executor).await }
    }
}

impl<T> DeleteAll<MySql> for T
where
    T: CrudlyDefault + Schema + Send,
{
    fn delete_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_delete_all::<Self, MySql>(executor).await }
    }
}

impl<T> SelectById<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
{
    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_select_by_id(executor, id).await }
    }
}

impl<T> SelectByIds<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
{
    fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async move { generic_select_by_ids(executor, ids, batch_size).await }
    }
}

impl<T> IdExists<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
{
    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_id_exists::<Self, MySql>(executor, id).await }
    }
}

impl<T> UpdateById<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<MySql> + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
{
    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_update_by_id(executor, self).await }
    }
}

impl<T> DeleteById<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
{
    fn delete_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_delete_by_id::<Self, MySql>(executor, id).await }
    }
}

impl<T> InsertWithoutId<MySql> for T
where
    T: CrudlyDefault + Schema + IntoRow<MySql> + DBAssignedId + Send,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_insert_returning_id::<Self, MySql>(executor, self).await }
    }
}

impl<T> InsertManyWithoutIds<MySql> for T
where
    T: CrudlyDefault + Schema + IntoRow<MySql> + DBAssignedId + Send,
{
    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<MySql> + Send,
    {
        generic_insert_many_without_id::<Self, MySql, _>(executor, entities, batch_size).await
    }
}

impl<T> Insert<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<MySql> + ExternallyAssignedId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
    <T as HasId>::Id: 'static,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_insert_with_id::<Self, MySql>(executor, self).await }
    }
}

impl<T> InsertMany<MySql> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<MySql> + ExternallyAssignedId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, MySql> + Type<MySql>,
    <T as HasId>::Id: 'static,
{
    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<MySql> + Send,
    {
        generic_insert_many_with_id::<Self, MySql, _>(executor, entities, batch_size).await
    }
}

impl<T> SelectAllNoId<MySql> for T
where
    T: CrudlyDefault + Schema + NoId + for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
{
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_select_all_no_id(executor).await }
    }
}

impl<T> InsertNoId<MySql> for T
where
    T: CrudlyDefault + Schema + NoId + IntoRow<MySql> + Send,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async move {
            generic_insert_without_id::<Self, MySql>(executor, self)
                .await
                .map(|_| ())
        }
    }
}

impl<T> InsertManyNoId<MySql> for T
where
    T: CrudlyDefault + Schema + NoId + IntoRow<MySql> + Send,
{
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<MySql> + Send,
    {
        async move {
            generic_insert_many_without_id::<Self, MySql, _>(executor, entities, batch_size).await
        }
    }
}
