//! Proc macros for [`crudly`](https://crates.io/crates/crudly).

extern crate proc_macro;

mod crudly;
mod into_row;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Derive [`crudly::HasColumns`] and [`crudly::IntoRow`] for a struct.
///
/// Emits a single `impl<__CrudlyDb: Database> IntoRow<__CrudlyDb>` that uses [`sqlx::Arguments::add`],
/// so the struct is usable with any SQLx [`Database`](::sqlx::Database) for which every serialized
/// field type implements [`Encode`](::sqlx::Encode) and [`Type`](::sqlx::Type) (with a
/// higher-ranked `for<'q> Encode<'q, __CrudlyDb>` bound where needed). Row **reading** uses
/// [`Decode`](::sqlx::Decode) via [`FromRow`](::sqlx::FromRow), not this derive.
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

/// Derive [`crudly::Schema`], [`crudly::Crudly`], and the appropriate insert trait / id marker.
///
/// Mark the primary key with `#[crudly(id)]` on **exactly one** field (that field must not use `skip`).
/// Container `#[crudly(...)]`: `table`, `db_ids` (default), `external_ids`, `executor`.
/// Impls are generic over `sqlx::Database`; a custom `executor` must implement `crudly::CRUDExecutor<DB>` for each database you use with that type.
#[proc_macro_derive(Crudly, attributes(crudly))]
pub fn derive_crudly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    crudly::expand_derive_crudly(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
