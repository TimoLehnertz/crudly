use sqlx::{Database, Executor};

use crate::ReusableExecutor;

/// Column list for a row mapping. Split out so callers can use `Self::columns()` unambiguously
/// when [`IntoRow`] is implemented for several [`Database`] types.
pub trait HasColumns {
    /// The returned columns **must never** be empty!
    ///
    /// # Returns
    /// the column names for this entity (does **NOT** include the id column).
    fn columns() -> Vec<&'static str>;
}

/// Binds non-id values onto an SQLx [`sqlx::Arguments`] buffer for [`Database`] `DB`
/// (`DB::Arguments<'q>`).
pub trait IntoRow<DB: Database>: HasColumns {
    /// Binds values in the same order as [`HasColumns::columns`].
    /// Does **not** bind the id.
    fn bind_arguments<'q>(self, arguments: &mut DB::Arguments<'q>) -> sqlx::Result<()>;
}

/// Describes how a rust struct maps to a database table.
/// Currently requires that the corresponding table has
/// a **single** primary key. This restriction might get
/// lifted in the future.
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

/// [`Schema`] plus binding into `DB::Arguments` (what insert/update need).
///
/// Plain [`Schema`] is enough for reads; rustc does not infer [`IntoRow`] from [`Schema`]
/// supertraits, so this trait encodes the combined bound explicitly.
pub trait BindRow<DB: Database>: Schema<DB> + IntoRow<DB> {}

impl<T, DB: Database> BindRow<DB> for T where T: Schema<DB> + IntoRow<DB> {}

/// Marker trait that indicates that the id of an entity
/// is assigned by the database using something like an
/// AUTOINCREMENT or SERIAL column.
pub trait DBAssignedId {}

/// Marker trait that indicates that the id of an entity
/// is assigned not by the database but instead inside rust
/// using something like a uuid.
pub trait ExternallyAssignedId {}

/// Marker trait that opts an entity into the default `Crudly` and insert trait blanket impls.
///
/// Implement this per database backend:
/// `impl CrudlyDefault<sqlx::Sqlite> for MyEntity {}`
pub trait CrudlyDefault<DB: Database> {}

pub trait Crudly<DB: Database>: Sized {
    type Id: Clone + Send + Sync;

    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;

    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;

    /// `batch_size`: max ids per `IN (...)` group; `0` means one group for all ids.
    fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;

    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = DB>;

    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = DB>;

    fn delete_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = DB>;
}

/// Insert rows without supplying the id column so the database can assign ids (e.g. `AUTOINCREMENT`,
/// `SERIAL`). This trait is available through the default blanket impls for types that implement
/// [`CrudlyDefault`] and [`DBAssignedId`].
pub trait InsertWithoutId<DB: Database>: Sized {
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<i64>> + Send
    where
        E: Executor<'c, Database = DB>;

    /// `batch_size`: max rows per `INSERT`; `0` means one statement for all rows.
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<DB> + Send;
}

pub trait InsertWithId<DB: Database>: Sized {
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = DB>;

    /// `batch_size`: max rows per `INSERT`; `0` means one statement for all rows.
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>>
    where
        E: ReusableExecutor<DB> + Send;
}

// todo: add delete_many
