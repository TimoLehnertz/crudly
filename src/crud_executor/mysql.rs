use crate::crud_executor::{
    generic_delete_by_id, generic_insert_returning_id, generic_insert_with_id, generic_update_by_id,
};
use crate::{
    BindRow, CRUDExecutor, DBAssignedId, DefaultCRUDExecutor, ExternallyAssignedId,
    FormatPlaceholder, LastInsertedRowId, RowsAffected, Schema, generic_id_exists,
    generic_select_all, generic_select_by_id,
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

impl CRUDExecutor<MySql> for DefaultCRUDExecutor<MySql> {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;

    async fn find_all<S>(
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<Vec<S>>
    where
        S: Schema<MySql> + for<'r> FromRow<'r, MySqlRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<Option<S>>
    where
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: Schema<MySql> + for<'r> FromRow<'r, MySqlRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: Schema<MySql>,
    {
        generic_id_exists::<S, MySql>(executor, id).await
    }

    async fn insert_with_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<()>
    where
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql> + 'static,
        S: BindRow<MySql> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, MySql>(executor, entity).await
    }

    async fn insert_returning_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<i64>
    where
        S: BindRow<MySql> + DBAssignedId,
    {
        generic_insert_returning_id::<S, MySql>(executor, entity).await
    }

    async fn update_by_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: BindRow<MySql>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = MySql>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, MySql> + Type<MySql>,
        S: Schema<MySql>,
    {
        generic_delete_by_id::<S, MySql>(executor, id).await
    }
}
