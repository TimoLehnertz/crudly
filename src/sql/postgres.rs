use crate::sql::{FormatPlaceholder, RowsAffected, reusable_executor::ReusableExecutor};
use crate::sql::{
    format_placeholders, generic_delete_all, generic_delete_by_id, generic_id_exists,
    generic_insert_many_with_id, generic_insert_many_without_id, generic_insert_with_id,
    generic_insert_without_id, generic_select_all, generic_select_all_no_id, generic_select_by_id,
    generic_select_by_ids, generic_update_by_id,
};
use crate::{
    CrudlyDefault, DBAssignedId, DeleteAll, DeleteById, ExternallyAssignedId, HasId, IdExists,
    Insert, InsertMany, InsertManyNoId, InsertManyWithoutIds, InsertNoId, InsertWithoutId, IntoRow,
    NoId, Schema, SelectAll, SelectAllNoId, SelectById, SelectByIds, UpdateById,
};
use sqlx::postgres::{PgArguments, PgQueryResult, PgRow};
use sqlx::{AssertSqlSafe, Encode, Executor, FromRow, Postgres, Type, query_scalar_with};
use std::future::Future;

impl FormatPlaceholder for Postgres {
    fn format_placeholder(idx: usize) -> String {
        format!("${}", idx + 1)
    }
}

impl RowsAffected for PgQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

pub async fn pg_insert<'c, S, E>(entity: S, executor: E) -> sqlx::Result<i64>
where
    E: Executor<'c, Database = Postgres>,
    S: Schema + IntoRow<Postgres> + HasId + DBAssignedId,
{
    let table_name = S::table_name();
    let non_id_columns = <S as IntoRow<Postgres>>::columns();
    let columns = non_id_columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<String>>()
        .join(",");

    let id_column = S::id_column();

    let placeholders = format_placeholders::<Postgres>(non_id_columns.len());

    let sql = format!(
        "INSERT INTO {table_name} ({columns}) VALUES ({placeholders}) RETURNING \"{id_column}\""
    );

    let mut arguments = PgArguments::default();
    entity.bind_arguments(&mut arguments)?;

    let inserted_id: i64 = query_scalar_with(AssertSqlSafe(sql), arguments)
        .fetch_one(executor)
        .await?;

    Ok(inserted_id)
}

impl<T> SelectAll<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_select_all(executor).await }
    }
}

impl<T> DeleteAll<Postgres> for T
where
    T: CrudlyDefault + Schema + Send,
{
    fn delete_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_delete_all::<Self, Postgres>(executor).await }
    }
}

impl<T> SelectById<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, PgRow> + Unpin + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
{
    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_select_by_id(executor, id).await }
    }
}

impl<T> SelectByIds<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + for<'r> FromRow<'r, PgRow> + Unpin + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
{
    fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async move { generic_select_by_ids(executor, ids, batch_size).await }
    }
}

impl<T> IdExists<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
{
    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_id_exists::<Self, Postgres>(executor, id).await }
    }
}

impl<T> UpdateById<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Postgres> + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
{
    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_update_by_id(executor, self).await }
    }
}

impl<T> DeleteById<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
{
    fn delete_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_delete_by_id::<Self, Postgres>(executor, id).await }
    }
}

impl<T> InsertWithoutId<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Postgres> + DBAssignedId + Send,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { pg_insert(self, executor).await }
    }
}

impl<T> InsertManyWithoutIds<Postgres> for T
where
    T: CrudlyDefault + Schema + IntoRow<Postgres> + DBAssignedId + Send,
{
    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<Postgres> + Send,
    {
        generic_insert_many_without_id::<Self, Postgres, _>(executor, entities, batch_size).await
    }
}

impl<T> Insert<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Postgres> + ExternallyAssignedId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
    <T as HasId>::Id: 'static,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_insert_with_id::<Self, Postgres>(executor, self).await }
    }
}

impl<T> InsertMany<Postgres> for T
where
    T: CrudlyDefault + Schema + HasId + IntoRow<Postgres> + ExternallyAssignedId + Send,
    for<'q> <T as HasId>::Id: Encode<'q, Postgres> + Type<Postgres>,
    <T as HasId>::Id: 'static,
{
    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<Postgres> + Send,
    {
        generic_insert_many_with_id::<Self, Postgres, _>(executor, entities, batch_size).await
    }
}

impl<T> SelectAllNoId<Postgres> for T
where
    T: CrudlyDefault + Schema + NoId + for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async { generic_select_all_no_id(executor).await }
    }
}

impl<T> InsertNoId<Postgres> for T
where
    T: CrudlyDefault + Schema + NoId + IntoRow<Postgres> + Send,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = Postgres>,
    {
        async move {
            generic_insert_without_id::<Self, Postgres>(executor, self)
                .await
                .map(|_| ())
        }
    }
}

impl<T> InsertManyNoId<Postgres> for T
where
    T: CrudlyDefault + Schema + NoId + IntoRow<Postgres> + Send,
{
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<Postgres> + Send,
    {
        async move {
            generic_insert_many_without_id::<Self, Postgres, _>(executor, entities, batch_size)
                .await
        }
    }
}
