//! Proc macros for [`crudly`](https://crates.io/crates/crudly).

extern crate proc_macro;

mod crudly;
mod derive_attr;
mod into_row;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Derives [`HasColumns`](https://docs.rs/crudly/0.2.0/crudly/trait.HasColumns.html) and [`IntoRow`](https://docs.rs/crudly/0.2.0/crudly/trait.IntoRow.html) for a struct.
///
/// Emits a single `impl<__CrudlyDb: Database> IntoRow<__CrudlyDb>` that uses [`Arguments::add`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Arguments.html#tymethod.add),
/// so the struct is usable with any SQLx [`Database`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Database.html) for which every serialized
/// field type implements [`Encode`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Encode.html) and [`Type`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Type.html) (with a
/// higher-ranked `for<'q> Encode<'q, __CrudlyDb>` bound where needed). Row **reading** uses
/// [`Decode`](https://docs.rs/sqlx/0.8.6/sqlx/trait.Decode.html) via [`FromRow`](https://docs.rs/sqlx/0.8.6/sqlx/trait.FromRow.html), not this derive.
///
/// Container: `rename_all`, `default` (via `#[sqlx(...)]` or `#[crudly(...)]`, not both for the same key).
/// Fields: `rename`, `default`, `flatten`, `skip`, `id` (for `#[derive(Crudly)]` only, via `#[crudly(id)]`), `try_from`, `try_into`, `json` / `json(nullable)`.
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

/// Derive Schema and the appropriate insert trait / id marker ([`InsertWithoutId`](https://docs.rs/crudly/latest/crudly/trait.InsertWithoutId.html) when ids are DB-assigned, otherwise [`InsertWithId`](https://docs.rs/crudly/latest/crudly/trait.InsertWithId.html)).
///
/// **Note:** Schema requires the `HasColumns` trait which can be derived by using the `IntoRow` derive.
///
/// Mark the primary key with `#[crudly(id)]` on **exactly one** field (that field must not use `skip`).
/// Container `#[crudly(...)]`: `table`, `db_ids` (default), `external_ids`, `executor`.
/// if executor is not specified, `crudly::DefaultCRUDExecutor` will be used.
/// Impls are generic over `sqlx::Database`; a custom `executor` must implement
/// `crudly::CRUDExecutor<DB>` for each database you use with that type.
/// Derives [`Schema`](https://docs.rs/crudly/latest/crudly/trait.Schema.html) and the appropriate id marker
/// ([`DBAssignedId`](https://docs.rs/crudly/latest/crudly/trait.DBAssignedId.html) or [`ExternallyAssignedId`](https://docs.rs/crudly/latest/crudly/trait.ExternallyAssignedId.html)).
///
/// Same `#[crudly(...)]` container options as the `Crudly` derive for table name and id strategy (`db_ids` / `external_ids`),
/// but does **not** derive [`Crudly`](https://docs.rs/crudly/latest/crudly/trait.Crudly.html) or insert traits.
///
/// Do not combine with `#[derive(Crudly)]`—`Crudly` already implements `Schema` and the id marker.
#[proc_macro_derive(Schema, attributes(crudly))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    crudly::expand_derive_schema(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Crudly, attributes(crudly))]
pub fn derive_crudly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    crudly::expand_derive_crudly(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
