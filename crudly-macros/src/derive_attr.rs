//! Inspect `#[derive(...)]` attributes on a type.

use syn::parse::Parser;
use syn::{Attribute, DeriveInput, Meta, Path, punctuated::Punctuated};

/// Returns true if any `#[derive(...)]` on `input` lists a trait whose path ends with `name`
/// (e.g. `Schema`, `Crudly`), or is a single-segment path equal to `name`.
pub(crate) fn input_derives(input: &DeriveInput, name: &str) -> bool {
    input
        .attrs
        .iter()
        .any(|attr| derive_attr_contains(attr, name))
}

fn derive_attr_contains(attr: &Attribute, name: &str) -> bool {
    if !attr.path().is_ident("derive") {
        return false;
    }
    let Meta::List(list) = &attr.meta else {
        return false;
    };
    let parser = Punctuated::<Path, syn::Token![,]>::parse_terminated;
    let Ok(paths) = parser.parse2(list.tokens.clone()) else {
        return false;
    };
    paths.iter().any(|p| path_matches_trait(p, name))
}

fn path_matches_trait(path: &Path, name: &str) -> bool {
    if path.is_ident(name) {
        return true;
    }
    path.segments
        .last()
        .is_some_and(|seg| seg.ident == name && seg.arguments.is_empty())
}
