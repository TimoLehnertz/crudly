use crate::into_row;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, LitStr};

#[derive(Clone, Copy, PartialEq, Eq)]
enum IdStrategy {
    DbAssigned,
    External,
}

struct CrudlyAttrs {
    table: Option<String>,
    /// `None` means neither `db_ids` nor `external_ids` was supplied.
    explicit_id_strategy: Option<IdStrategy>,
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

        let explicit_id_strategy = if external_ids {
            Some(IdStrategy::External)
        } else if db_ids {
            Some(IdStrategy::DbAssigned)
        } else {
            None
        };

        Ok(CrudlyAttrs {
            table,
            explicit_id_strategy,
        })
    }
}

struct IdInfo {
    id_ident: syn::Ident,
    id_ty: syn::Type,
    id_column_lit: String,
    strategy: IdStrategy,
}

/// Parsed struct + id metadata shared by `Schema` and `Crudly` derives.
struct CrudlyParsed {
    ident: syn::Ident,
    generics: syn::Generics,
    table_lit: String,
    /// `None` when no field is marked with `#[crudly(id)]`.
    id_info: Option<IdInfo>,
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

        let id_info = match id_marked.len() {
            0 => {
                if attrs.explicit_id_strategy.is_some() {
                    return Err(syn::Error::new(
                        input.span(),
                        "`#[crudly(db_ids)]` / `#[crudly(external_ids)]` require exactly one field marked with `#[crudly(id)]`",
                    ));
                }
                None
            }
            1 => {
                let (id_field, id_fa) = (&id_marked[0].0, &id_marked[0].1);
                let id_ident = id_field.ident.as_ref().unwrap().clone();
                let id_ty = id_field.ty.clone();
                let id_column_lit = into_row::column_name_for_field(id_field, id_fa, rename_all)?;
                let strategy = attrs.explicit_id_strategy.unwrap_or(IdStrategy::DbAssigned);
                Some(IdInfo {
                    id_ident,
                    id_ty,
                    id_column_lit,
                    strategy,
                })
            }
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    format!(
                        "`#[derive({derive_name})]` may only mark one field with `#[crudly(id)]`"
                    ),
                ));
            }
        };

        let table_str = match &attrs.table {
            Some(t) => t.clone(),
            None => pluralize_ascii_identifier_base(&ident.to_string().to_snake_case()),
        };

        Ok(CrudlyParsed {
            ident,
            generics: input.generics.clone(),
            table_lit: table_str,
            id_info,
        })
    }

    fn schema_and_marker_tokens(&self, column_fragments: &[TokenStream]) -> TokenStream {
        let CrudlyParsed {
            ident,
            generics,
            table_lit,
            id_info,
        } = self;

        let (struct_impl_gen, struct_ty_gen, struct_wc_opt) = generics.split_for_impl();
        let struct_wc = struct_wc_opt
            .map(|w| quote!(#w))
            .unwrap_or_else(|| quote!());

        let (schema_columns_fn, id_impls) = match id_info {
            None => {
                let columns_fn = quote! {
                    fn columns() -> ::std::vec::Vec<&'static str> {
                        let mut out = ::std::vec::Vec::new();
                        #(#column_fragments)*
                        out
                    }
                };
                let extra = quote! {
                    impl #struct_impl_gen ::crudly::NoId for #ident #struct_ty_gen #struct_wc {}
                };
                (columns_fn, extra)
            }
            Some(IdInfo {
                id_ident,
                id_ty,
                id_column_lit,
                strategy,
            }) => {
                let columns_fn = quote! {
                    fn columns() -> ::std::vec::Vec<&'static str> {
                        let mut out = ::std::vec![#id_column_lit];
                        #(#column_fragments)*
                        out
                    }
                };

                let marker_impl = match strategy {
                    IdStrategy::DbAssigned => quote! {
                        impl #struct_impl_gen ::crudly::DBAssignedId
                            for #ident #struct_ty_gen #struct_wc {}
                    },
                    IdStrategy::External => quote! {
                        impl #struct_impl_gen ::crudly::ExternallyAssignedId
                            for #ident #struct_ty_gen #struct_wc {}
                    },
                };

                let has_id_impl = quote! {
                    impl #struct_impl_gen ::crudly::HasId for #ident #struct_ty_gen #struct_wc {
                        type Id = #id_ty;

                        fn id_column() -> &'static str {
                            #id_column_lit
                        }

                        fn id(&self) -> Self::Id {
                            self.#id_ident.clone()
                        }
                    }

                    #marker_impl
                };
                (columns_fn, has_id_impl)
            }
        };

        let schema_impl = quote! {
            impl #struct_impl_gen ::crudly::Schema for #ident #struct_ty_gen #struct_wc {
                fn table_name() -> &'static str {
                    #table_lit
                }
                #schema_columns_fn
            }
        };

        quote! {
            #schema_impl
            #id_impls
        }
    }
}

pub fn expand_derive_schema(input: DeriveInput) -> syn::Result<TokenStream> {
    let container = into_row::parse_container_attrs(&input)?;
    let rename_all = container.rename_all;
    let parsed = CrudlyParsed::parse(&input, "Schema")?;

    let data_struct = match &input.data {
        Data::Struct(ds) => ds,
        _ => unreachable!("validated in CrudlyParsed::parse"),
    };
    let fields_named = match &data_struct.fields {
        Fields::Named(n) => n,
        _ => unreachable!("validated in CrudlyParsed::parse"),
    };

    let mut column_fragments = Vec::new();
    for field in &fields_named.named {
        let fa = into_row::parse_field_attrs(field)?;
        if fa.crudly_id {
            continue;
        }
        let sql_name = into_row::column_name_for_field(field, &fa, rename_all)?;
        column_fragments.push(into_row::schema_columns_tokens_for_field(
            field, &fa, &sql_name,
        )?);
    }

    Ok(parsed.schema_and_marker_tokens(&column_fragments))
}
