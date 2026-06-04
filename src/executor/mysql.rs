use crate::executor::reusable_executor::ReusableExecutor;
use crate::executor::{
    generic_delete_by_id, generic_insert_many_without_id, generic_insert_returning_id,
    generic_insert_with_id, generic_update_by_id,
};
use crate::{
    BindRow, CRUDExecutor, DBAssignedId, DefaultCRUDExecutor, ExternallyAssignedId,
    FormatPlaceholder, LastInsertedRowId, RowsAffected, Schema, generic_id_exists,
    generic_insert_many_with_id, generic_select_all, generic_select_by_id,
};
use sqlx::MySql;
use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use sqlx::{Encode, Executor, FromRow, Type};

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

impl CRUDExecutor<MySql> for DefaultCRUDExecutor {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;
    type InsertManyWithoutIdResult = sqlx::Result<()>;

    async fn select_all<'c, S, E>(executor: E) -> sqlx::Result<Vec<S>>
    where
        E: Executor<'c, Database = MySql>,
        S: Schema<MySql> + for<'r> FromRow<'r, MySqlRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<Option<S>>
    where
        E: Executor<'c, Database = MySql>,
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: Schema<MySql> + for<'r> FromRow<'r, MySqlRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = MySql>,
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: Schema<MySql>,
    {
        generic_id_exists::<S, MySql>(executor, id).await
    }

    async fn insert_with_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = MySql>,
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql> + 'static,
        S: BindRow<MySql> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, MySql>(executor, entity).await
    }

    async fn insert_returning_id<'e, 'c, S, E>(entity: S, executor: E) -> sqlx::Result<i64>
    where
        E: 'e + Executor<'c, Database = MySql>,
        S: BindRow<MySql> + DBAssignedId,
    {
        generic_insert_returning_id::<S, MySql>(executor, entity).await
    }

    async fn insert_many_without_id<S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: ReusableExecutor<MySql> + Send,
        S: BindRow<MySql> + DBAssignedId,
    {
        generic_insert_many_without_id::<S, MySql, _>(executor, entities, batch_size).await
    }

    async fn insert_many_with_id<S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: ReusableExecutor<MySql> + Send,
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: BindRow<MySql> + ExternallyAssignedId,
    {
        generic_insert_many_with_id::<S, MySql, _>(executor, entities, batch_size).await
    }

    async fn update_by_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = MySql>,
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: BindRow<MySql>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = MySql>,
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: Schema<MySql>,
    {
        generic_delete_by_id::<S, MySql>(executor, id).await
    }
}
