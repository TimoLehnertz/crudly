use crate::executor::reusable_executor::ReusableExecutor;
use crate::executor::{
    generic_delete_by_id, generic_insert_many_without_id, generic_insert_returning_id,
    generic_insert_with_id, generic_update_by_id,
};
use crate::{
    BindRow, Crudly, CrudlyDefault, DBAssignedId, ExternallyAssignedId, FormatPlaceholder,
    InsertWithId, InsertWithoutId, LastInsertedRowId, RowsAffected, Schema, generic_id_exists,
    generic_insert_many_with_id, generic_select_all, generic_select_by_id, generic_select_by_ids,
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

impl<T> Crudly<MySql> for T
where
    T: CrudlyDefault<MySql>
        + Schema<MySql>
        + BindRow<MySql>
        + for<'r> FromRow<'r, MySqlRow>
        + Unpin,
    for<'q> <T as Schema<MySql>>::Id: Encode<'q, MySql> + Type<MySql>,
{
    type Id = <Self as Schema<MySql>>::Id;

    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>>
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_select_all(executor).await }
    }

    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>>
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_select_by_id(executor, id).await }
    }

    fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>>
    where
        E: Executor<'c, Database = MySql>,
    {
        async move { generic_select_by_ids(executor, ids, batch_size).await }
    }

    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_id_exists::<Self, MySql>(executor, id).await }
    }

    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>>
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_update_by_id(executor, self).await }
    }

    fn delete_by_id<'c, E>(id: &Self::Id, executor: E) -> impl Future<Output = sqlx::Result<bool>>
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_delete_by_id::<Self, MySql>(executor, id).await }
    }
}

impl<T> InsertWithoutId<MySql> for T
where
    T: CrudlyDefault<MySql> + Schema<MySql> + BindRow<MySql> + DBAssignedId,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_insert_returning_id::<Self, MySql>(executor, self).await }
    }

    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<MySql> + Send,
    {
        generic_insert_many_without_id::<Self, MySql, _>(executor, entities, batch_size).await
    }
}

impl<T> InsertWithId<MySql> for T
where
    T: CrudlyDefault<MySql> + Schema<MySql> + BindRow<MySql> + ExternallyAssignedId,
    for<'q> <T as Schema<MySql>>::Id: Encode<'q, MySql> + Type<MySql>,
    <T as Schema<MySql>>::Id: 'static,
{
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>>
    where
        E: Executor<'c, Database = MySql>,
    {
        async { generic_insert_with_id::<Self, MySql>(executor, self).await }
    }

    async fn insert_many<E>(entities: Vec<Self>, batch_size: usize, executor: E) -> sqlx::Result<()>
    where
        E: ReusableExecutor<MySql> + Send,
    {
        generic_insert_many_with_id::<Self, MySql, _>(executor, entities, batch_size).await
    }
}
