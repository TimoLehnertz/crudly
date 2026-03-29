use sqlx::{Database, Executor};

/// Column list for a row mapping. Split out so callers can use `Self::columns()` unambiguously
/// when [`IntoRow`] is implemented for many argument types (e.g. `DB::Arguments<'a>` per lifetime).
pub trait HasColumns {
    /// The returned columns **must never** be empty!
    ///
    /// # Returns
    /// the column names for this entity (does **NOT** include the id column).
    fn columns() -> Vec<&'static str>;
}

/// Binds non-id values onto a concrete arguments buffer (e.g. [`Database::Arguments`]).
///
/// Generic over `A` so tests can implement this trait for a **mock** buffer and assert bind order
/// without a database. With sqlx, implement for [`Database::Arguments<'a>`](Database::Arguments)
/// per driver (e.g. [`sqlx::sqlite::SqliteArguments<'a>`]).
///
/// You generally **cannot** write `impl<'a, DB: Database> IntoRow<DB::Arguments<'a>> for MyType`
/// (rustc [E0207]); use one impl per concrete database or a small macro that enumerates `DB`.
pub trait IntoRow<A: ?Sized>: HasColumns {
    /// Binds values in the same order as [`HasColumns::columns`].
    /// Does **not** bind the id.
    fn bind_arguments(self, arguments: &mut A) -> sqlx::Result<()>;
}

/// Describes how a rust struct maps to a database table.
/// Currently requires that the corresponding table has
/// a **single** primary key.
pub trait Schema<DB: Database>: HasColumns + Send {
    type Id: Clone + Send + Sync;

    /// # Returns
    /// the name of the table for this entity.
    fn table_name() -> &'static str;

    /// # Returns
    /// the name of the id column
    fn id_column() -> &'static str;

    /// # Returns
    /// the db id of `self`
    fn id(&self) -> Self::Id;
}

/// [`Schema`] plus binding into `DB::Arguments` for every lifetime (what insert/update need).
///
/// Plain [`Schema`] is enough for reads; rustc does not infer higher-ranked [`IntoRow`] from [`Schema`]
/// supertraits, so this trait encodes the combined bound explicitly.
pub trait BindRow<DB: Database>:
    Schema<DB> + for<'a> IntoRow<<DB as Database>::Arguments<'a>>
{
}

impl<T, DB: Database> BindRow<DB> for T
where
    T: Schema<DB>,
    for<'a> T: IntoRow<<DB as Database>::Arguments<'a>>,
{
}

/// Marker trait that indicates that the id of an entity
/// is assigned by the database using something like an
/// AUTOINCREMENT or SERIAL column.
pub trait DBAssignedId {}

/// Marker trait that indicates that the id of an entity
/// is assigned not by the database but instead inside rust
/// using something like a uuid.
pub trait ExternallyAssignedId {}

pub trait Crudly<DB: Database>: Sized {
    type Id: Clone + Send + Sync;

    /// Most likely sqlx::Result<bool> But one could also use the
    /// sql RETURNING clause to return the actual entity after it was updated.
    type UpdateByIdResult;

    /// The result type of the delete operation.
    ///
    /// This could be sqlx::Result<()> or something else that additionally indicates if
    /// the entity was indeed deleted or didn't exist in the first place.
    type DeleteByIdResult;

    fn find_all(
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>>;

    fn select_by_id(
        id: &Self::Id,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>>;

    fn id_exists(
        id: &Self::Id,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<bool>>;

    fn update_by_id(
        entity: Self,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = Self::UpdateByIdResult>;

    fn delete_by_id(
        id: &Self::Id,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = Self::DeleteByIdResult>;
}

pub trait InsertReturningId<DB: Database> {
    fn insert_returning_id(
        self,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = sqlx::Result<i64>>;
}

pub trait InsertWithId<DB: Database> {
    /// Most likely sqlx::Result<()> But one could also use the
    /// sql RETURNING clause to return the actual entity after it was inserted.
    type Result;

    fn insert_with_id(
        self,
        executor: impl for<'e> Executor<'e, Database = DB>,
    ) -> impl Future<Output = Self::Result>;
}

// todo: add insert_many and delete_many
