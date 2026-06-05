use crate::into_row;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, LitStr, parse_quote};

#[derive(Clone, Copy, PartialEq, Eq)]
enum IdStrategy {
    DbAssigned,
    External,
}

struct CrudlyAttrs {
    table: Option<String>,
    id_strategy: IdStrategy,
}

fn pluralize_ascii_identifier_base(snake_singular: &str) -> String {
    if snake_singular.is_empty() {
        return snake_singular.to_string();
    }
    let w = snake_singular;
    let last = w.as_bytes()[w.len() - 1];
    let penultimate_vowel = w.len() >= 2
        && !matches!(
            w.as_bytes()[w.len() - 2],
            b'a' | b'e' | b'i' | b'o' | b'u' | b'y'
        );
    if last == b'y' && penultimate_vowel {
        format!("{}ies", &w[..w.len() - 1])
    } else if w.ends_with("ch") || w.ends_with("sh") || last == b's' || last == b'x' || last == b'z'
    {
        format!("{w}es")
    } else {
        format!("{w}s")
    }
}

impl CrudlyAttrs {
    fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut seen = HashSet::<&'static str>::new();
        let mut table = None;
        let mut db_ids = false;
        let mut external_ids = false;

        for attr in &input.attrs {
            if !attr.path().is_ident("crudly") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                let path = meta.path.clone();

                fn merge(
                    seen: &mut HashSet<&'static str>,
                    key: &'static str,
                    span: proc_macro2::Span,
                ) -> syn::Result<()> {
                    if !seen.insert(key) {
                        return Err(syn::Error::new(
                            span,
                            format!("duplicate `#[crudly({key})]` on the same type"),
                        ));
                    }
                    Ok(())
                }

                if meta.path.is_ident("table") {
                    merge(&mut seen, "table", path.span())?;
                    meta.input.parse::<syn::Token![=]>()?;
                    let lit: LitStr = meta.input.parse()?;
                    table = Some(lit.value());
                } else if meta.path.is_ident("db_ids") {
                    merge(&mut seen, "db_ids", path.span())?;
                    db_ids = true;
                } else if meta.path.is_ident("external_ids") {
                    merge(&mut seen, "external_ids", path.span())?;
                    external_ids = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        path,
                        "unknown `#[crudly(...)]` for `Schema` (expected table, db_ids, external_ids)",
                    ));
                }
                Ok(())
            })?;
        }

        if db_ids && external_ids {
            return Err(syn::Error::new(
                input.span(),
                "`#[crudly(db_ids)]` and `#[crudly(external_ids)]` are mutually exclusive",
            ));
        }

        let id_strategy = if external_ids {
            IdStrategy::External
        } else {
            IdStrategy::DbAssigned
        };

        Ok(CrudlyAttrs { table, id_strategy })
    }
}

/// Parsed struct + id metadata shared by `Schema` and `Crudly` derives.
struct CrudlyParsed {
    attrs: CrudlyAttrs,
    ident: syn::Ident,
    generics: syn::Generics,
    id_ident: syn::Ident,
    id_ty: syn::Type,
    id_column_lit: String,
    table_lit: String,
}

impl CrudlyParsed {
    fn parse(input: &DeriveInput, derive_name: &str) -> syn::Result<Self> {
        let attrs = CrudlyAttrs::parse(input)?;
        let container = into_row::parse_container_attrs(input)?;
        let rename_all = container.rename_all;
        let ident = input.ident.clone();

        let data_struct = match &input.data {
            Data::Struct(ds) => ds,
            Data::Enum(e) => {
                return Err(syn::Error::new(
                    e.enum_token.span,
                    format!(
                        "`{derive_name}` derive is only supported on structs with named fields (not enums)"
                    ),
                ));
            }
            Data::Union(u) => {
                return Err(syn::Error::new(
                    u.union_token.span,
                    format!(
                        "`{derive_name}` derive is only supported on structs with named fields (not unions)"
                    ),
                ));
            }
        };

        let fields_named = match &data_struct.fields {
            Fields::Named(n) => n,
            _ => {
                return Err(syn::Error::new(
                    data_struct.fields.span(),
                    format!("`{derive_name}` derive requires a struct with named fields"),
                ));
            }
        };

        let mut id_marked: Vec<(&syn::Field, into_row::FieldAttrs)> = Vec::new();
        for field in &fields_named.named {
            let fa = into_row::parse_field_attrs(field)?;
            if fa.crudly_id {
                id_marked.push((field, fa));
            }
        }
        let (id_field, id_fa) = match id_marked.len() {
            0 => {
                return Err(syn::Error::new(
                    input.span(),
                    format!(
                        "`#[derive({derive_name})]` requires exactly one field marked with `#[crudly(id)]`"
                    ),
                ));
            }
            1 => (&id_marked[0].0, &id_marked[0].1),
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    format!(
                        "`#[derive({derive_name})]` may only mark one field with `#[crudly(id)]`"
                    ),
                ));
            }
        };
        let id_ident = id_field.ident.as_ref().unwrap().clone();
        let id_ty = id_field.ty.clone();
        let id_column_lit = into_row::column_name_for_field(id_field, id_fa, rename_all)?;

        let table_str = match &attrs.table {
            Some(t) => t.clone(),
            None => pluralize_ascii_identifier_base(&ident.to_string().to_snake_case()),
        };

        Ok(CrudlyParsed {
            attrs,
            ident,
            generics: input.generics.clone(),
            id_ident,
            id_ty,
            id_column_lit,
            table_lit: table_str,
        })
    }

    fn schema_and_marker_tokens(&self) -> TokenStream {
        let CrudlyParsed {
            attrs,
            ident,
            generics,
            id_ident,
            id_ty,
            id_column_lit,
            table_lit,
            ..
        } = self;

        let db_ty: TokenStream = quote!(__CrudlyDb);

        let mut impl_generics = generics.clone();
        impl_generics
            .params
            .insert(0, parse_quote!(__CrudlyDb: ::sqlx::Database));
        let (impl_gen, _, _) = impl_generics.split_for_impl();

        let (struct_impl_gen, struct_ty_gen, struct_wc_opt) = generics.split_for_impl();
        let struct_schema_where = struct_wc_opt
            .map(|w| quote!(#w))
            .unwrap_or_else(|| quote!());
        let struct_wc_tokens = struct_schema_where.clone();

        let marker_impl = match attrs.id_strategy {
            IdStrategy::DbAssigned => quote! {
                impl #struct_impl_gen ::crudly::DBAssignedId for #ident #struct_ty_gen #struct_wc_tokens {}
            },
            IdStrategy::External => quote! {
                impl #struct_impl_gen ::crudly::ExternallyAssignedId for #ident #struct_ty_gen #struct_wc_tokens {}
            },
        };

        quote! {
            impl #impl_gen ::crudly::Schema<#db_ty> for #ident #struct_ty_gen #struct_schema_where
            {
                type Id = #id_ty;

                fn table_name() -> &'static str {
                    #table_lit
                }

                fn id_column() -> &'static str {
                    #id_column_lit
                }

                fn id(&self) -> Self::Id {
                    self.#id_ident.clone()
                }
            }

            #marker_impl
        }
    }
}

pub fn expand_derive_schema(input: DeriveInput) -> syn::Result<TokenStream> {
    let parsed = CrudlyParsed::parse(&input, "Schema")?;
    Ok(parsed.schema_and_marker_tokens())
}
