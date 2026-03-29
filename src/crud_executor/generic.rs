use crate::{BindRow, Schema};
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
/// a comma delimited string of the columns for
/// the given schema starting with the id column.
pub(super) fn comma_delimited_columns_with_id<S: Schema<DB>, DB: Database>() -> String {
    std::iter::once(format!("\"{}\"", S::id_column()))
        .chain(S::columns().iter().map(|c| format!("\"{c}\"")))
        .collect::<Vec<String>>()
        .join(",")
}

pub async fn generic_select_all<S, DB: Database>(
    executor: impl Executor<'_, Database = DB>,
) -> sqlx::Result<Vec<S>>
where
    S: Schema<DB> + for<'r> FromRow<'r, DB::Row> + Unpin,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let columns = comma_delimited_columns_with_id::<S, DB>();
    let table_name = S::table_name();

    let sql = format!("SELECT {columns} FROM {table_name};");
    query_as(&sql).fetch_all(executor).await
}

pub async fn generic_select_by_id<S, DB: Database>(
    executor: impl Executor<'_, Database = DB>,
    id: &S::Id,
) -> sqlx::Result<Option<S>>
where
    DB: FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema<DB> + for<'r> FromRow<'r, DB::Row> + Unpin,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let columns = comma_delimited_columns_with_id::<S, DB>();
    let table_name = S::table_name();
    let id_column = S::id_column();

    let placeholder = DB::format_placeholder(0);

    let sql = format!("SELECT {columns} FROM {table_name} WHERE {id_column} = {placeholder};");
    query_as(&sql).bind(id).fetch_optional(executor).await
}

pub async fn generic_id_exists<S, DB: Database>(
    executor: impl Executor<'_, Database = DB>,
    id: &S::Id,
) -> sqlx::Result<bool>
where
    DB: FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema<DB>,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let table_name = S::table_name();
    let id_column = S::id_column();

    let placeholder = DB::format_placeholder(0);

    let sql = format!("SELECT 1 FROM {table_name} WHERE \"{id_column}\" = {placeholder};");

    query(&sql)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map(|option| option.is_some())
}

pub(super) async fn generic_insert_with_id<S, DB: Database>(
    executor: impl for<'e> Executor<'e, Database = DB>,
    entity: S,
) -> sqlx::Result<()>
where
    DB: FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: BindRow<DB>,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let table_name = S::table_name();
    let columns = comma_delimited_columns_with_id::<S, DB>();
    let placeholders = format_placeholders::<DB>(S::columns().len() + 1); // +1 for the id column

    let sql = format!("INSERT INTO {table_name} ({columns}) VALUES ({placeholders})");

    let mut arguments = DB::Arguments::default();
    arguments.add(entity.id()).map_err(sqlx::Error::Encode)?;
    entity.bind_arguments(&mut arguments)?;

    query_with(&sql, arguments).execute(executor).await?;
    Ok(())
}

pub(super) async fn generic_insert_returning_id<S, DB: Database>(
    executor: impl for<'e> Executor<'e, Database = DB>,
    entity: S,
) -> sqlx::Result<i64>
where
    DB::QueryResult: LastInsertedRowId,
    DB: FormatPlaceholder,
    S: BindRow<DB>,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let table_name = S::table_name();
    let columns = S::columns()
        .into_iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<String>>()
        .join(",");

    let placeholders = format_placeholders::<DB>(S::columns().len());

    let sql = format!("INSERT INTO {table_name} ({columns}) VALUES ({placeholders})");

    let mut arguments = DB::Arguments::default();
    entity.bind_arguments(&mut arguments)?;

    let result = query_with(&sql, arguments).execute(executor).await?;
    Ok(result.last_insert_rowid())
}

/// # Returns
/// - Ok(true) if the entity was updated
/// - Ok(false) if it didn't exist
/// - Err(e) on error
pub(super) async fn generic_update_by_id<S, DB: Database>(
    executor: impl for<'e> Executor<'e, Database = DB>,
    entity: S,
) -> sqlx::Result<bool>
where
    DB::QueryResult: RowsAffected,
    DB: FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: BindRow<DB>,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let table_name = S::table_name();
    let id_column = S::id_column();

    let mut set_sql = String::new();

    // columns are expected not to be empty
    for (idx, column) in S::columns().iter().enumerate() {
        if idx > 0 {
            set_sql.push(',');
        }
        set_sql.push_str(&format!(
            "\"{column}\"={placeholder}",
            placeholder = DB::format_placeholder(idx)
        ));
    }

    let entity_id = entity.id();

    let id_placeholder = DB::format_placeholder(S::columns().len());
    let sql = format!("UPDATE {table_name} SET {set_sql} WHERE {id_column} = {id_placeholder}");

    let mut arguments = DB::Arguments::default();
    entity.bind_arguments(&mut arguments)?;
    arguments.add(entity_id).map_err(sqlx::Error::Encode)?;

    let result = query_with(&sql, arguments).execute(executor).await?;
    Ok(result.rows_affected() > 0)
}

/// # Returns
/// - Ok(true) if the entity was deleted
/// - Ok(false) if it didn't exist
/// - Err(e) on error
pub(super) async fn generic_delete_by_id<S, DB: Database>(
    executor: impl for<'e> Executor<'e, Database = DB>,
    id: &S::Id,
) -> sqlx::Result<bool>
where
    DB::QueryResult: RowsAffected,
    DB: FormatPlaceholder,
    S::Id: for<'q> Encode<'q, DB> + Type<DB>,
    S: Schema<DB>,
    for<'e> <DB as Database>::Arguments<'e>: IntoArguments<'e, DB>,
{
    let table_name = S::table_name();
    let id_column = S::id_column();
    let id_placeholder = DB::format_placeholder(0);

    let sql = format!("DELETE FROM {table_name} WHERE {id_column} = {id_placeholder}");

    let result = query(&sql).bind(id).execute(executor).await?;

    Ok(result.rows_affected() > 0)
}
