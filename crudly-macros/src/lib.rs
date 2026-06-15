//! Proc macros for [`crudly`](https://crates.io/crates/crudly).

extern crate proc_macro;

mod crudly;
mod into_row;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Derives [`IntoRow`](https://docs.rs/crudly/latest/crudly/trait.IntoRow.html) for a struct,
/// including [`IntoRow::columns`](https://docs.rs/crudly/latest/crudly/trait.IntoRow.html#tymethod.columns).
///
/// Emits a single `impl<__CrudlyDb: Database> IntoRow<__CrudlyDb>` that uses [`Arguments::add`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Arguments.html#tymethod.add),
/// so the struct is usable with any SQLx [`Database`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Database.html) for which every serialized
/// field type implements [`Encode`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Encode.html) and [`Type`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Type.html) (with a
/// higher-ranked `for<'q> Encode<'q, __CrudlyDb>` bound where needed). Row **reading** uses
/// [`Decode`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Decode.html) via [`FromRow`](https://docs.rs/sqlx/0.8.6/sqlx/trait.FromRow.html), not this derive.
///
/// Container: `rename_all`, `default` (via `#[sqlx(...)]` or `#[crudly(...)]`, not both for the same key).
/// Fields: `rename`, `default`, `flatten`, `skip`, `id` (for `#[derive(Schema)]` only, via `#[crudly(id)]`), `try_from`, `try_into`, `json` / `json(nullable)`.
/// If `try_from` / `try_into` target type is `String`, the expansion calls [`ToString::to_string`] on the
/// field and binds that `String` (otherwise it uses [`TryInto`] to the given type).
/// Enable the `json` feature on `crudly` / `crudly-macros` to expand `json` attributes.
#[proc_macro_derive(IntoRow, attributes(crudly, sqlx))]
pub fn derive_into_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    into_row::expand_derive_into_row(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives [`Schema`](https://docs.rs/crudly/latest/crudly/trait.Schema.html) (`table_name()` and `columns()`).
///
/// If exactly one field is marked with `#[crudly(id)]`, also derives
/// [`HasId`](https://docs.rs/crudly/latest/crudly/trait.HasId.html) and the appropriate id marker trait
/// ([`DBAssignedId`](https://docs.rs/crudly/latest/crudly/trait.DBAssignedId.html) or
/// [`ExternallyAssignedId`](https://docs.rs/crudly/latest/crudly/trait.ExternallyAssignedId.html)).
/// Without a `#[crudly(id)]` field, `HasId` is **not** derived and neither is any marker trait.
///
/// Container `#[crudly(...)]`: `table`, `db_ids` (default when id field present), `external_ids`.
/// `db_ids` / `external_ids` require a `#[crudly(id)]` field to be present.
#[proc_macro_derive(Schema, attributes(crudly))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    crudly::expand_derive_schema(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
