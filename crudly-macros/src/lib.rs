//! Proc macros for [`crudly`](https://crates.io/crates/crudly).

extern crate proc_macro;

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
/// Fields: `rename`, `default`, `flatten`, `skip`, `try_from`, `try_into`, `json` / `json(nullable)`.
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
