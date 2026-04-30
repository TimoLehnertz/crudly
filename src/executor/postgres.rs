use crate::executor::{
    format_placeholders, generic_delete_by_id, generic_insert_many_without_id,
    generic_insert_with_id, generic_update_by_id,
};
use crate::{
    BindRow, CRUDExecutor, DBAssignedId, DefaultCRUDExecutor, ExternallyAssignedId,
    FormatPlaceholder, RowsAffected, Schema, generic_id_exists, generic_insert_many_with_id,
    generic_select_all, generic_select_by_id,
};
use sqlx::postgres::{PgArguments, PgQueryResult, PgRow};
use sqlx::{Encode, Executor, FromRow, Postgres, Type, query_scalar_with};

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
    S: BindRow<Postgres> + DBAssignedId,
{
    let table_name = S::table_name();
    let columns = S::columns()
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<String>>()
        .join(",");

    let id_column = S::id_column();

    let placeholders = format_placeholders::<Postgres>(S::columns().len());

    let sql = format!(
        "INSERT INTO {table_name} ({columns}) VALUES ({placeholders}) RETURNING \"{id_column}\""
    );

    let mut arguments = PgArguments::default();
    entity.bind_arguments(&mut arguments)?;

    let inserted_id: i64 = query_scalar_with(&sql, arguments)
        .fetch_one(executor)
        .await?;

    Ok(inserted_id)
}

impl CRUDExecutor<Postgres> for DefaultCRUDExecutor {
    type InsertWithIdResult = sqlx::Result<()>;
    type UpdateByIdResult = sqlx::Result<bool>;
    type DeleteByIdResult = sqlx::Result<bool>;
    type InsertManyWithoutIdResult = sqlx::Result<()>;

    async fn select_all<'c, S, E>(executor: E) -> sqlx::Result<Vec<S>>
    where
        E: Executor<'c, Database = Postgres>,
        S: Schema<Postgres> + for<'r> FromRow<'r, PgRow> + Unpin,
    {
        generic_select_all(executor).await
    }

    async fn select_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<Option<S>>
    where
        E: Executor<'c, Database = Postgres>,
        S::Id: for<'q> Encode<'q, Postgres> + Type<Postgres>,
        S: Schema<Postgres> + for<'r> FromRow<'r, PgRow> + Unpin,
    {
        generic_select_by_id(executor, id).await
    }

    async fn id_exists<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Postgres>,
        S::Id: for<'q> Encode<'q, Postgres> + Type<Postgres>,
        S: Schema<Postgres>,
    {
        generic_id_exists::<S, Postgres>(executor, id).await
    }

    async fn insert_with_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Postgres>,
        S::Id: for<'q> Encode<'q, Postgres> + Type<Postgres> + 'static,
        S: BindRow<Postgres> + ExternallyAssignedId,
    {
        generic_insert_with_id::<S, Postgres>(executor, entity).await
    }

    async fn insert_returning_id<'e, 'c, S, E>(entity: S, executor: E) -> sqlx::Result<i64>
    where
        E: 'e + Executor<'c, Database = Postgres>,
        S: BindRow<Postgres> + DBAssignedId,
    {
        pg_insert(entity, executor).await
    }

    async fn insert_many_without_id<'c, S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Postgres> + Clone,
        S: BindRow<Postgres> + DBAssignedId,
    {
        generic_insert_many_without_id::<S, Postgres, _>(executor, entities, batch_size).await
    }

    async fn insert_many_with_id<'c, S, E>(
        entities: Vec<S>,
        batch_size: usize,
        executor: E,
    ) -> sqlx::Result<()>
    where
        E: Executor<'c, Database = Postgres> + Clone,
        S::Id: for<'q> Encode<'q, Postgres> + Type<Postgres>,
        S: BindRow<Postgres> + ExternallyAssignedId,
    {
        generic_insert_many_with_id::<S, Postgres, _>(executor, entities, batch_size).await
    }

    async fn update_by_id<'c, S, E>(entity: S, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Postgres>,
        S::Id: for<'q> Encode<'q, Postgres> + Type<Postgres>,
        S: BindRow<Postgres>,
    {
        generic_update_by_id(executor, entity).await
    }

    async fn delete_by_id<'c, S, E>(id: &S::Id, executor: E) -> sqlx::Result<bool>
    where
        E: Executor<'c, Database = Postgres>,
        S::Id: for<'q> Encode<'q, Postgres> + Type<Postgres>,
        S: Schema<Postgres>,
    {
        generic_delete_by_id::<S, Postgres>(executor, id).await
    }
}
