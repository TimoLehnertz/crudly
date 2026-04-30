use crate::derive_attr::input_derives;
use crate::into_row;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Data, DeriveInput, Fields, LitStr, Type, WherePredicate, parse_quote};

#[derive(Clone, Copy, PartialEq, Eq)]
enum IdStrategy {
    DbAssigned,
    External,
}

struct CrudlyAttrs {
    table: Option<String>,
    id_strategy: IdStrategy,
    executor: Option<Type>,
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
        let mut executor = None::<Type>;

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
                } else if meta.path.is_ident("executor") {
                    merge(&mut seen, "executor", path.span())?;
                    meta.input.parse::<syn::Token![=]>()?;
                    executor = Some(meta.input.parse()?);
                } else {
                    return Err(syn::Error::new_spanned(
                        path,
                        "unknown `#[crudly(...)]` for `Crudly` or `Schema` (expected table, db_ids, external_ids, executor)",
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

        Ok(CrudlyAttrs {
            table,
            id_strategy,
            executor,
        })
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

    fn crudly_and_insert_tokens(&self) -> syn::Result<TokenStream> {
        let CrudlyParsed {
            attrs,
            ident,
            generics,
            ..
        } = self;

        let db_ty: TokenStream = quote!(__CrudlyDb);

        let exec_ty: TokenStream = match &attrs.executor {
            None => quote!(::crudly::DefaultCRUDExecutor),
            Some(ty) => ty.to_token_stream(),
        };

        let mut impl_generics = generics.clone();
        impl_generics
            .params
            .insert(0, parse_quote!(__CrudlyDb: ::sqlx::Database));
        let (impl_gen, _, _) = impl_generics.split_for_impl();
        let (_, struct_ty_gen, _) = generics.split_for_impl();

        let extend_where = |extra: Vec<WherePredicate>| -> syn::Result<TokenStream> {
            let mut preds: Punctuated<WherePredicate, Comma> = Punctuated::new();
            if let Some(w) = &generics.where_clause {
                for p in w.predicates.iter() {
                    preds.push(p.clone());
                }
            }
            for p in extra {
                preds.push(p);
            }
            Ok(quote!(where #preds))
        };

        let insert_exec_pred: WherePredicate =
            syn::parse2(quote!(#exec_ty: ::crudly::CRUDExecutor<__CrudlyDb>))?;

        let crudly_preds: Vec<WherePredicate> = vec![
            parse_quote!(Self: ::crudly::Schema<#db_ty> + ::crudly::BindRow<#db_ty>),
            insert_exec_pred.clone(),
            parse_quote!(Self: for<'r> ::sqlx::FromRow<'r, <#db_ty as ::sqlx::Database>::Row>),
            parse_quote!(for<'q> <Self as ::crudly::Schema<#db_ty>>::Id: ::sqlx::Encode<'q, #db_ty> + ::sqlx::Type<#db_ty>),
        ];
        let crudly_where = extend_where(crudly_preds)?;

        let insert_impl = match attrs.id_strategy {
            IdStrategy::DbAssigned => {
                let insert_where = extend_where(vec![
                    parse_quote!(Self: ::crudly::Schema<#db_ty> + ::crudly::BindRow<#db_ty>),
                    insert_exec_pred.clone(),
                    parse_quote!(Self: ::crudly::DBAssignedId),
                ])?;
                quote! {
                    impl #impl_gen ::crudly::InsertWithoutId<#db_ty> for #ident #struct_ty_gen
                    #insert_where
                    {
                        type InsertManyResult = <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::InsertManyWithoutIdResult;

                        fn insert<'__crudly_c, __CrudlyE>(
                            self,
                            executor: __CrudlyE,
                        ) -> impl Future<Output = ::sqlx::Result<i64>> + Send
                        where
                            __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                        {
                            async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::insert_returning_id::<Self, _>(self, executor).await }
                        }

                        async fn insert_many<'__crudly_c, __CrudlyE>(
                            entities: ::std::vec::Vec<Self>,
                            batch_size: usize,
                            executor: __CrudlyE,
                        ) -> Self::InsertManyResult
                        where
                            __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty> + ::core::clone::Clone,
                        {
                            <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::insert_many_without_id::<Self, _>(entities, batch_size, executor).await
                        }
                    }
                }
            }
            IdStrategy::External => {
                let insert_where = extend_where(vec![
                    parse_quote!(<Self as ::crudly::Schema<#db_ty>>::Id: ::sqlx::Type<#db_ty>),
                    parse_quote!(for<'q> <Self as ::crudly::Schema<#db_ty>>::Id: ::sqlx::Encode<'q, #db_ty>),
                    parse_quote!(Self: ::crudly::Schema<#db_ty> + ::crudly::BindRow<#db_ty>),
                    insert_exec_pred,
                    parse_quote!(Self: ::crudly::ExternallyAssignedId),
                ])?;
                quote! {
                    impl #impl_gen ::crudly::InsertWithId<#db_ty> for #ident #struct_ty_gen
                    #insert_where
                    {
                        type InsertResult = <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::InsertWithIdResult;

                        fn insert<'__crudly_c, __CrudlyE>(
                            self,
                            executor: __CrudlyE,
                        ) -> impl Future<Output = Self::InsertResult> + Send
                        where
                            __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                        {
                            async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::insert_with_id::<Self, _>(self, executor).await }
                        }

                        async fn insert_many<'__crudly_c, __CrudlyE>(
                            entities: ::std::vec::Vec<Self>,
                            batch_size: usize,
                            executor: __CrudlyE,
                        ) -> ::sqlx::Result<()>
                        where
                            __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty> + ::core::clone::Clone,
                        {
                            <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::insert_many_with_id::<Self, _>(entities, batch_size, executor).await
                        }
                    }
                }
            }
        };

        Ok(quote! {
            impl #impl_gen ::crudly::Crudly<#db_ty> for #ident #struct_ty_gen
            #crudly_where
            {
                type Id = <Self as ::crudly::Schema<#db_ty>>::Id;
                type UpdateByIdResult = <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::UpdateByIdResult;
                type DeleteByIdResult = <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::DeleteByIdResult;

                fn select_all<'__crudly_c, __CrudlyE>(
                    executor: __CrudlyE,
                ) -> impl Future<Output = ::sqlx::Result<::std::vec::Vec<Self>>> + Send
                where
                    __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                {
                    async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::select_all::<Self, _>(executor).await }
                }

                fn delete_by_id<'__crudly_c, __CrudlyE>(
                    id: &Self::Id,
                    executor: __CrudlyE,
                ) -> impl Future<Output = Self::DeleteByIdResult> + Send
                where
                    __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                {
                    async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::delete_by_id::<Self, _>(id, executor).await }
                }

                fn id_exists<'__crudly_c, __CrudlyE>(
                    id: &Self::Id,
                    executor: __CrudlyE,
                ) -> impl Future<Output = ::sqlx::Result<bool>> + Send
                where
                    __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                {
                    async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::id_exists::<Self, _>(id, executor).await }
                }

                fn select_by_id<'__crudly_c, __CrudlyE>(
                    id: &Self::Id,
                    executor: __CrudlyE,
                ) -> impl Future<Output = ::sqlx::Result<::std::option::Option<Self>>> + Send
                where
                    __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                {
                    async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::select_by_id::<Self, _>(id, executor).await }
                }

                fn update_by_id<'__crudly_c, __CrudlyE>(
                    self,
                    executor: __CrudlyE,
                ) -> impl Future<Output = Self::UpdateByIdResult> + Send
                where
                    __CrudlyE: ::sqlx::Executor<'__crudly_c, Database = #db_ty>,
                {
                    async { <#exec_ty as ::crudly::CRUDExecutor<#db_ty>>::update_by_id::<Self, _>(self, executor).await }
                }
            }

            #insert_impl
        })
    }
}

pub fn expand_derive_schema(input: DeriveInput) -> syn::Result<TokenStream> {
    if input_derives(&input, "Crudly") {
        return Err(syn::Error::new(
            input.ident.span(),
            "`#[derive(Schema)]` cannot be used together with `#[derive(Crudly)]`: `Crudly` already implements `Schema` and the id marker (`DBAssignedId` or `ExternallyAssignedId`). Use only one of `Schema` or `Crudly`.",
        ));
    }

    let parsed = CrudlyParsed::parse(&input, "Schema")?;
    Ok(parsed.schema_and_marker_tokens())
}

pub fn expand_derive_crudly(input: DeriveInput) -> syn::Result<TokenStream> {
    if input_derives(&input, "Schema") {
        return Err(syn::Error::new(
            input.ident.span(),
            "`#[derive(Crudly)]` cannot be used together with `#[derive(Schema)]`: `Crudly` already implements `Schema` and the id marker (`DBAssignedId` or `ExternallyAssignedId`). Use only one of `Schema` or `Crudly`.",
        ));
    }

    let parsed = CrudlyParsed::parse(&input, "Crudly")?;
    let schema_marker = parsed.schema_and_marker_tokens();
    let rest = parsed.crudly_and_insert_tokens()?;
    Ok(quote! {
        #schema_marker
        #rest
    })
}

#[cfg(test)]
mod derive_conflict_tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    fn parse_derive_input(tokens: proc_macro2::TokenStream) -> DeriveInput {
        parse2(tokens).expect("parse DeriveInput")
    }

    #[test]
    fn crudly_errors_if_schema_also_in_derive_list() {
        let input = parse_derive_input(quote! {
            #[derive(Crudly, Schema)]
            struct Foo {
                #[crudly(id)]
                id: i64,
            }
        });
        let err = expand_derive_crudly(input).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Schema"),
            "expected message to mention Schema, got: {msg}"
        );
    }

    #[test]
    fn schema_errors_if_crudly_also_in_derive_list() {
        let input = parse_derive_input(quote! {
            #[derive(Schema, Crudly)]
            struct Foo {
                #[crudly(id)]
                id: i64,
            }
        });
        let err = expand_derive_schema(input).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Crudly"),
            "expected message to mention Crudly, got: {msg}"
        );
    }

    #[test]
    fn conflict_detected_with_separate_derive_attributes() {
        let input = parse_derive_input(quote! {
            #[derive(Crudly)]
            #[derive(Schema)]
            struct Foo {
                #[crudly(id)]
                id: i64,
            }
        });
        assert!(expand_derive_crudly(input.clone()).is_err());
        assert!(expand_derive_schema(input).is_err());
    }
}
