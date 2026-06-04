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

impl CRUDExecutor<Sqlite> for DefaultCRUDExecutor {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;
    type InsertManyWithoutIdResult = sqlx::Result<()>;

    async fn select_all<'c, S, E>(executor: E) -> sqlx::Result<Vec<S>>
    where
        E: Executor<'c, Database = Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<Option<S>>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite> + for<'r> FromRow<'r, SqliteRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_id_exists::<S, Sqlite>(executor, id).await
    }

    async fn insert_with_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite> + 'static,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, Sqlite>(executor, entity).await
    }

    async fn insert_returning_id<'e, 'c, S, E>(entity: S, executor: E) -> sqlx::Result<i64>
    where
        'c: 'e,
        E: 'e + Executor<'c, Database = Sqlite>,
        S: BindRow<Sqlite> + DBAssignedId,
    {
        generic_insert_returning_id::<S, Sqlite>(executor, entity).await
    }

    async fn insert_many_without_id<S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: ReusableExecutor<Sqlite> + Send,
        S: BindRow<Sqlite> + DBAssignedId,
    {
        generic_insert_many_without_id::<S, Sqlite, _>(executor, entities, batch_size).await
    }

    async fn insert_many_with_id<S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: ReusableExecutor<Sqlite> + Send,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite> + ExternallyAssignedId,
    {
        generic_insert_many_with_id::<S, Sqlite, _>(executor, entities, batch_size).await
    }

    async fn update_by_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: BindRow<Sqlite>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Sqlite>,
        S::Id: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
        S: Schema<Sqlite>,
    {
        generic_delete_by_id::<S, Sqlite>(executor, id).await
    }
}
