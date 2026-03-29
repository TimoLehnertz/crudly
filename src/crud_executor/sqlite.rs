use crate::crud_executor::{
    generic_delete_by_id, generic_insert_returning_id, generic_insert_with_id, generic_update_by_id,
};
use crate::{
    CRUDExecutor, DBAssignedId, DefaultCRUDExecutor, ExternallyAssignedId, FormatPlaceholder,
    BindRow, LastInsertedRowId, RowsAffected, Schema, generic_id_exists, generic_select_all,
    generic_select_by_id,
};
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Encode, Executor, FromRow, Sqlite, Type};

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

impl CRUDExecutor<Sqlite> for DefaultCRUDExecutor<Sqlite> {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;

    async fn find_all<S>(
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Vec<S>>
    where
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<Option<S>>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_id_exists::<S, Sqlite>(executor, id).await
    }

    async fn insert_with_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<()>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite> + 'static,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, Sqlite>(executor, entity).await
    }

    async fn insert_returning_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<i64>
    where
        S: BindRow<Sqlite> + DBAssignedId,
    {
        generic_insert_returning_id::<S, Sqlite>(executor, entity).await
    }

    async fn update_by_id<S>(
        entity: S,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<S>(
        id: &S::Id,
        executor: impl for<'e> Executor<'e, Database = Sqlite>,
    ) -> sqlx::Result<bool>
    where
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_delete_by_id::<S, Sqlite>(executor, id).await
    }
}
