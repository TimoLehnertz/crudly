use sqlx::{Database, Executor};

use crate::ReusableExecutor;

/// Column list for a row mapping. Split out from IntoRow so callers can use `Self::columns()` unambiguously
/// when [`IntoRow`] is implemented for several [`Database`] types.
pub trait HasColumns {
    /// The returned columns **MUST NEVER** be empty!
    /// That is because the sql query builders rely on the fact that there will be at least one column.
    ///
    /// # Returns
    /// the column names for this entity (does **NOT** include the id column).
    fn columns() -> Vec<&'static str>;
}

/// Tries to be the counterpart to [sqlx::FromRow](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html).
/// Binds non-id values onto an SQLx [`sqlx::Arguments`] buffer for [`Database`] `DB`
/// (`DB::Arguments<'q>`).
pub trait IntoRow<DB: Database>: HasColumns {
    /// Binds values in the same order as [`HasColumns::columns`].
    /// Does **not** bind the id.
    fn bind_arguments<'q>(self, arguments: &mut DB::Arguments<'q>) -> sqlx::Result<()>;
}

pub trait HasId {
    type Id: Clone + Send + Sync;

    /// # Returns
    /// the name of the id column
    fn id_column() -> &'static str;

    /// # Returns
    /// the db id of `self`
    fn id(&self) -> Self::Id;
}

/// Describes how a rust struct maps to a database table.
pub trait Schema: Send {
    /// # Returns
    /// the name of the table for this entity.
    fn table_name() -> &'static str;

    /// # Returns
    /// All column names for this entity **including** the id column. Unlike
    /// [HasColumns::columns] which excludes the id column.
    fn columns() -> Vec<&'static str>;
}

/// Marker trait that indicates that the id of an entity
/// is assigned by the database using something like an
/// AUTOINCREMENT or SERIAL column.
pub trait DBAssignedId: HasId {}

/// Marker trait that indicates that the id of an entity
/// is assigned not by the database but instead inside rust
/// using something like a uuid.
pub trait ExternallyAssignedId: HasId {}

/// Marker trait for entities that have no id column.
/// Emitted automatically by `#[derive(Schema)]` when no field is marked with `#[crudly(id)]`.
pub trait NoId: Send {}

/// Marker trait that opts an entity into the default `Crudly` and insert trait blanket impls.
pub trait CrudlyDefault<DB: Database> {}

pub trait SelectAll<DB: Database>: Sized {
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;
}

pub trait DeleteAll<DB: Database>: Sized {
    fn delete_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = DB>;
}

pub trait SelectById<DB: Database>: Sized + HasId {
    fn select_by_id<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;
}

pub trait SelectByIds<DB: Database>: Sized + HasId {
    /// `batch_size`: max ids per `IN (...)` group; `0` means one group for all ids.
    fn select_by_ids<'c, E>(
        ids: Vec<Self::Id>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;
}

pub trait IdExists<DB: Database>: Sized + HasId {
    fn id_exists<'c, E>(
        id: &Self::Id,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = DB>;
}

pub trait UpdateById<DB: Database>: Sized + HasId {
    fn update_by_id<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<bool>> + Send
    where
        E: Executor<'c, Database = DB>;
}

// Todo: Add DeleteByIds

pub trait DeleteById<DB: Database>: Sized + HasId {
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
}

pub trait InsertManyWithoutIds<DB: Database>: Sized {
    /// `batch_size`: max rows per `INSERT`; `0` means one statement for all rows.
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<DB> + Send;
}

pub trait Insert<DB: Database>: Sized {
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = DB>;
}
pub trait InsertMany<DB: Database>: Sized {
    /// `batch_size`: max rows per `INSERT`; `0` means one statement for all rows.
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>>
    where
        E: ReusableExecutor<DB> + Send;
}

/// `select_all` for entities with no id column. Blanket impl provided for types that implement
/// [`NoId`] — no opt-in via [`CrudlyDefault`] required.
pub trait SelectAllNoId<DB: Database>: Sized {
    fn select_all<'c, E>(executor: E) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send
    where
        E: Executor<'c, Database = DB>;
}

/// `insert` (returning `()`) for entities with no id column. Blanket impl provided for types that
/// implement [`NoId`] — no opt-in via [`CrudlyDefault`] required.
pub trait InsertNoId<DB: Database>: Sized {
    fn insert<'c, E>(self, executor: E) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: Executor<'c, Database = DB>;
}

/// Batch `insert_many` for entities with no id column. Blanket impl provided for types that
/// implement [`NoId`] — no opt-in via [`CrudlyDefault`] required.
pub trait InsertManyNoId<DB: Database>: Sized {
    /// `batch_size`: max rows per `INSERT`; `0` means one statement for all rows.
    fn insert_many<E>(
        entities: Vec<Self>,
        batch_size: usize,
        executor: E,
    ) -> impl Future<Output = sqlx::Result<()>> + Send
    where
        E: ReusableExecutor<DB> + Send;
}
