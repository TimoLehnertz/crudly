use crate::HasId;
use crate::IntoRow;
use crate::Schema;
use crate::sql::reusable_executor::ReusableExecutor;
use sqlx::AssertSqlSafe;
use sqlx::{
    Arguments, Database, Encode, Executor, FromRow, IntoArguments, Type, query, query_as,
    query_with,
};

/// There is a similar function here: [sqlx::Arguments::format_placeholder] but thats
/// only really usable via the [sqlx::QueryBuilder] which is highly
/// undocumented and does not seem to be of great use here.
pub trait FormatPlaceholder {
    /// # Arguments
    /// - `idx`: The index of the placeholder starting at 0
    fn format_placeholder(idx: usize) -> String;
}

pub trait RowsAffected {
    fn rows_affected(&self) -> u64;
}

pub trait LastInsertedRowId {
    fn last_insert_rowid(&self) -> i64;
}

/// # Returns
/// A string of `n` placeholders separated by commas.
pub(super) fn format_placeholders<DB: FormatPlaceholder>(n: usize) -> String {
    (0..n)
        .map(|idx| DB::format_placeholder(idx))
        .collect::<Vec<String>>()
        .join(",")
}

/// # Returns
/// A comma-delimited, double-quoted string of every column returned by [`Schema::columns`].
/// For types with an id field this includes the id column (it is listed first by the derive).
pub(super) fn comma_delimited_columns<S>() -> String
where
    S: Schema,
{
    <S as Schema>::columns()
        .into_iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<String>>()
        .join(",")
}

pub async fn generic_select_all<S, DB: Database>(
    executor: impl Executor<'_, Database = DB>,
) -> sqlx::Result<Vec<S>>
where
    S: Schema + HasId + for<'r> FromRow<'r, DB::Row> + Unpin,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let columns = comma_delimited_columns::<S>();
    let table_name = S::table_name();
    let id_column = S::id_column();

    let sql = format!("SELECT {columns} FROM {table_name} ORDER BY {id_column} ASC;");
    query_as(AssertSqlSafe(sql)).fetch_all(executor).await
}

/// Like [`generic_select_all`] but for entities without an id column; selects all columns from
/// [`Schema::columns`] with no `ORDER BY`.
pub async fn generic_select_all_no_id<S, DB: Database>(
    executor: impl Executor<'_, Database = DB>,
) -> sqlx::Result<Vec<S>>
where
    S: Schema + for<'r> FromRow<'r, DB::Row> + Unpin,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let columns = comma_delimited_columns::<S>();
    let table_name = S::table_name();
    let sql = format!("SELECT {columns} FROM {table_name};");
    query_as(AssertSqlSafe(sql)).fetch_all(executor).await
}

pub async fn generic_delete_all<S, DB: Database>(
    executor: impl Executor<'_, Database = DB>,
) -> sqlx::Result<()>
where
    S: Schema,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let table_name = S::table_name();
    let sql = format!("DELETE FROM {table_name};");
    query(AssertSqlSafe(sql)).execute(executor).await?;
    Ok(())
}

pub async fn generic_select_by_id<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    id: &S::Id,
) -> sqlx::Result<Option<S>>
where
    DB: Database + FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema + HasId + for<'r> FromRow<'r, DB::Row> + Unpin,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let columns = comma_delimited_columns::<S>();
    let table_name = S::table_name();
    let id_column = S::id_column();

    let placeholder = DB::format_placeholder(0);

    let sql = format!("SELECT {columns} FROM {table_name} WHERE {id_column} = {placeholder};");
    query_as(AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(executor)
        .await
}

pub async fn generic_select_by_ids<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    ids: Vec<S::Id>,
    batch_size: usize,
) -> sqlx::Result<Vec<S>>
where
    DB: Database + FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema + HasId + for<'r> FromRow<'r, DB::Row> + Unpin,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let columns = comma_delimited_columns::<S>();
    let table_name = S::table_name();
    let id_column = S::id_column();
    let ids_len = ids.len();
    let in_group_size = if batch_size == 0 {
        ids_len
    } else {
        batch_size.min(ids_len)
    };

    let mut next_placeholder_idx = 0;
    let mut in_clauses = Vec::new();
    let mut remaining = ids_len;
    while remaining > 0 {
        let current_group = in_group_size.min(remaining);
        let placeholders = (0..current_group)
            .map(|_| {
                let placeholder = DB::format_placeholder(next_placeholder_idx);
                next_placeholder_idx += 1;
                placeholder
            })
            .collect::<Vec<String>>()
            .join(",");
        in_clauses.push(format!("{id_column} IN ({placeholders})"));
        remaining -= current_group;
    }

    let sql = format!(
        "SELECT {columns} FROM {table_name} WHERE {} ORDER BY {id_column} ASC;",
        in_clauses.join(" OR ")
    );

    let mut query = query_as(AssertSqlSafe(sql));
    for id in ids {
        query = query.bind(id);
    }

    query.fetch_all(executor).await
}

pub async fn generic_id_exists<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    id: &S::Id,
) -> sqlx::Result<bool>
where
    DB: Database + FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema + HasId,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let table_name = S::table_name();
    let id_column = S::id_column();

    let placeholder = DB::format_placeholder(0);

    let sql = format!("SELECT 1 FROM {table_name} WHERE \"{id_column}\" = {placeholder};");

    query(AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(executor)
        .await
        .map(|option| option.is_some())
}

pub async fn generic_insert_with_id<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    entity: S,
) -> sqlx::Result<()>
where
    DB: Database + FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema + IntoRow<DB> + HasId,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let table_name = S::table_name();
    let columns = comma_delimited_columns::<S>();
    let placeholders = format_placeholders::<DB>(<S as Schema>::columns().len());

    let sql = format!("INSERT INTO {table_name} ({columns}) VALUES ({placeholders})");

    let mut arguments = DB::Arguments::default();
    arguments.add(entity.id()).map_err(sqlx::Error::Encode)?;
    entity.bind_arguments(&mut arguments)?;

    query_with(AssertSqlSafe(sql), arguments)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn generic_insert_without_id<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    entity: S,
) -> sqlx::Result<DB::QueryResult>
where
    DB: Database + FormatPlaceholder,
    S: Schema + IntoRow<DB>,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let table_name = S::table_name();
    let non_id_columns = <S as IntoRow<DB>>::columns();
    let columns = non_id_columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<String>>()
        .join(",");

    let placeholders = format_placeholders::<DB>(non_id_columns.len());

    let sql = format!("INSERT INTO {table_name} ({columns}) VALUES ({placeholders})");

    let mut arguments = DB::Arguments::default();
    entity.bind_arguments(&mut arguments)?;

    query_with(AssertSqlSafe(sql), arguments)
        .execute(executor)
        .await
}

pub async fn generic_insert_returning_id<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    entity: S,
) -> sqlx::Result<i64>
where
    DB: Database + FormatPlaceholder,
    DB::QueryResult: LastInsertedRowId,
    S: Schema + IntoRow<DB>,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let result = generic_insert_without_id::<S, DB>(executor, entity).await?;
    Ok(result.last_insert_rowid())
}

/// Inserts many rows, omitting the id column (same shape as [`generic_insert_returning_id`] per row).
///
/// When `batch_size` is `0`, runs a single `INSERT` for all rows. Otherwise splits into multiple
/// statements of at most `batch_size` rows each.
pub async fn generic_insert_many_without_id<S, DB, E>(
    mut executor: E,
    entities: Vec<S>,
    batch_size: usize,
) -> sqlx::Result<()>
where
    E: ReusableExecutor<DB>,
    DB: Database + FormatPlaceholder,
    S: Schema + IntoRow<DB>,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    if entities.is_empty() {
        return Ok(());
    }

    let table_name = S::table_name();
    let non_id_columns = <S as IntoRow<DB>>::columns();
    let columns = non_id_columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<String>>()
        .join(",");

    let cols_per_row = non_id_columns.len();

    let mut rest = entities;

    while !rest.is_empty() {
        let chunk_len = if batch_size == 0 {
            rest.len()
        } else {
            batch_size.min(rest.len())
        };
        let chunk = rest.split_off(chunk_len);
        let chunk_entities = std::mem::replace(&mut rest, chunk);

        let mut placeholder_idx = 0;
        let mut value_tuples = Vec::with_capacity(chunk_len);
        for _ in 0..chunk_len {
            let placeholders = (0..cols_per_row)
                .map(|_| {
                    let p = DB::format_placeholder(placeholder_idx);
                    placeholder_idx += 1;
                    p
                })
                .collect::<Vec<String>>()
                .join(",");
            value_tuples.push(format!("({placeholders})"));
        }

        let sql = format!(
            "INSERT INTO {table_name} ({columns}) VALUES {}",
            value_tuples.join(",")
        );

        let mut arguments = DB::Arguments::default();
        for entity in chunk_entities {
            entity.bind_arguments(&mut arguments)?;
        }
        executor
            .execute_query_with(AssertSqlSafe(sql), arguments)
            .await?;
    }

    Ok(())
}

/// Inserts many rows, with the id column (same shape as [`generic_insert_with_id`] per row).
///
/// When `batch_size` is `0`, runs a single `INSERT` for all rows. Otherwise splits into multiple
/// statements of at most `batch_size` rows each.
pub async fn generic_insert_many_with_id<'c, S, DB, E>(
    mut executor: E,
    entities: Vec<S>,
    batch_size: usize,
) -> sqlx::Result<()>
where
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    E: ReusableExecutor<DB> + Send,
    DB: Database + FormatPlaceholder,
    S: Schema + IntoRow<DB> + HasId,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    if entities.is_empty() {
        return Ok(());
    }

    let table_name = S::table_name();
    let columns = comma_delimited_columns::<S>();

    let cols_per_row = <S as Schema>::columns().len();

    let mut rest = entities;

    while !rest.is_empty() {
        let chunk_len = if batch_size == 0 {
            rest.len()
        } else {
            batch_size.min(rest.len())
        };
        let remainder = rest.split_off(chunk_len);

        let mut placeholder_idx = 0;
        let mut value_tuples = Vec::with_capacity(chunk_len);
        for _ in 0..chunk_len {
            let placeholders = (0..cols_per_row)
                .map(|_| {
                    let p = DB::format_placeholder(placeholder_idx);
                    placeholder_idx += 1;
                    p
                })
                .collect::<Vec<String>>()
                .join(",");
            value_tuples.push(format!("({placeholders})"));
        }

        let sql = format!(
            "INSERT INTO {table_name} ({columns}) VALUES {}",
            value_tuples.join(",")
        );

        let mut arguments = DB::Arguments::default();
        for entity in rest {
            arguments.add(entity.id()).map_err(sqlx::Error::Encode)?;
            entity.bind_arguments(&mut arguments)?;
        }
        executor
            .execute_query_with(AssertSqlSafe(sql), arguments)
            .await?;
        rest = remainder;
    }

    Ok(())
}

/// # Returns
/// - Ok(true) if the entity was updated
/// - Ok(false) if it didn't exist
/// - Err(e) on error
pub async fn generic_update_by_id<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    entity: S,
) -> sqlx::Result<bool>
where
    DB: Database + FormatPlaceholder,
    DB::QueryResult: RowsAffected,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema + IntoRow<DB> + HasId,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let table_name = S::table_name();
    let id_column = S::id_column();

    let mut set_sql = String::new();

    // columns are expected not to be empty
    for (idx, column) in <S as IntoRow<DB>>::columns().iter().enumerate() {
        if idx > 0 {
            set_sql.push(',');
        }
        set_sql.push_str(&format!(
            "\"{column}\"={placeholder}",
            placeholder = DB::format_placeholder(idx)
        ));
    }

    let entity_id = entity.id();

    let id_placeholder = DB::format_placeholder(<S as IntoRow<DB>>::columns().len());
    let sql = format!("UPDATE {table_name} SET {set_sql} WHERE {id_column} = {id_placeholder}");

    let mut arguments = DB::Arguments::default();
    entity.bind_arguments(&mut arguments)?;
    arguments.add(entity_id).map_err(sqlx::Error::Encode)?;

    let result = query_with(AssertSqlSafe(sql), arguments)
        .execute(executor)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// # Returns
/// - Ok(true) if the entity was deleted
/// - Ok(false) if it didn't exist
/// - Err(e) on error
pub async fn generic_delete_by_id<S, DB>(
    executor: impl Executor<'_, Database = DB>,
    id: &S::Id,
) -> sqlx::Result<bool>
where
    DB: Database + FormatPlaceholder,
    DB::QueryResult: RowsAffected,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema + HasId,
    <DB as Database>::Arguments: IntoArguments<DB>,
{
    let table_name = S::table_name();
    let id_column = S::id_column();
    let id_placeholder = DB::format_placeholder(0);

    let sql = format!("DELETE FROM {table_name} WHERE {id_column} = {id_placeholder}");

    let result = query(AssertSqlSafe(sql)).bind(id).execute(executor).await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{HasId, IntoRow, Schema};
    use sqlx::{Sqlite, SqliteConnection, SqlitePool};

    pub struct Dummy;

    impl IntoRow<Sqlite> for Dummy {
        fn columns() -> Vec<&'static str> {
            unimplemented!()
        }

        fn bind_arguments(
            self,
            _arguments: &mut <Sqlite as Database>::Arguments,
        ) -> sqlx::Result<()> {
            unimplemented!()
        }
    }

    impl Schema for Dummy {
        fn table_name() -> &'static str {
            unimplemented!()
        }

        fn columns() -> Vec<&'static str> {
            unimplemented!()
        }
    }

    impl HasId for Dummy {
        type Id = i64;

        fn id(&self) -> Self::Id {
            unimplemented!()
        }

        fn id_column() -> &'static str {
            unimplemented!()
        }
    }

    #[allow(dead_code)]
    async fn test_exec_pool_owned(pool: SqlitePool) {
        generic_insert_many_without_id(&pool, vec![Dummy], 0)
            .await
            .unwrap();
    }

    #[allow(dead_code)]
    async fn test_exec_pool_borrowed(pool: &SqlitePool) {
        generic_insert_many_without_id(pool, vec![Dummy], 0)
            .await
            .unwrap();

        generic_insert_many_without_id(pool, vec![Dummy], 0)
            .await
            .unwrap();
    }

    #[allow(dead_code)]
    async fn test_exec_pool_exclusive(pool: &mut SqlitePool) {
        generic_insert_many_without_id(&mut *pool, vec![Dummy], 0)
            .await
            .unwrap();

        generic_insert_many_without_id(&mut *pool, vec![Dummy], 0)
            .await
            .unwrap();
    }

    #[allow(dead_code)]
    async fn test_exec_con(con: &mut SqliteConnection) {
        generic_insert_many_without_id(&mut *con, vec![Dummy], 100)
            .await
            .unwrap();

        generic_insert_many_without_id(&mut *con, vec![Dummy], 100)
            .await
            .unwrap();
    }
}
