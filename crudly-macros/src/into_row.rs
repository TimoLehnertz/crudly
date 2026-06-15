use std::collections::{BTreeSet, HashSet};

use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Field, Ident, LitStr, Token, Type, parenthesized, parse_quote};

#[derive(Copy, Clone)]
#[allow(clippy::enum_variant_names)] // mirrors sqlx `rename_all` spellings (e.g. camelCase)
pub(crate) enum RenameAll {
    LowerCase,
    SnakeCase,
    UpperCase,
    ScreamingSnakeCase,
    KebabCase,
    CamelCase,
    PascalCase,
}

impl RenameAll {
    fn parse(lit: &LitStr) -> syn::Result<Self> {
        match lit.value().as_str() {
            "lowercase" => Ok(Self::LowerCase),
            "snake_case" => Ok(Self::SnakeCase),
            "UPPERCASE" => Ok(Self::UpperCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            "camelCase" => Ok(Self::CamelCase),
            "PascalCase" => Ok(Self::PascalCase),
            _ => Err(syn::Error::new_spanned(
                lit,
                "unexpected value for rename_all",
            )),
        }
    }

    fn apply(self, ident: &str) -> String {
        match self {
            Self::LowerCase => ident.to_lowercase(),
            Self::SnakeCase => ident.to_snake_case(),
            Self::UpperCase => ident.to_uppercase(),
            Self::ScreamingSnakeCase => ident.to_shouty_snake_case(),
            Self::KebabCase => ident.to_kebab_case(),
            Self::CamelCase => ident.to_lower_camel_case(),
            Self::PascalCase => ident.to_upper_camel_case(),
        }
    }
}

#[derive(Clone, Copy)]
enum JsonAttr {
    NonNullable,
    Nullable,
}

pub(crate) struct ContainerAttrs {
    pub(crate) rename_all: Option<RenameAll>,
    pub(crate) _container_default: bool,
}

#[derive(Clone)]
pub(crate) struct FieldAttrs {
    rename: Option<String>,
    field_default: bool,
    pub(crate) flatten: bool,
    skip: bool,
    /// Primary key field for `#[derive(Schema)]`; omitted from `IntoRow::columns` / bind like `skip`.
    pub(crate) crudly_id: bool,
    try_from: Option<Type>,
    try_into: Option<Type>,
    json: Option<JsonAttr>,
    keys: HashSet<&'static str>,
}

impl FieldAttrs {
    fn empty() -> Self {
        Self {
            rename: None,
            field_default: false,
            flatten: false,
            skip: false,
            crudly_id: false,
            try_from: None,
            try_into: None,
            json: None,
            keys: HashSet::new(),
        }
    }
}

fn merge_field_key(
    attrs: &mut FieldAttrs,
    key: &'static str,
    span: Span,
    set: impl FnOnce(&mut FieldAttrs),
) -> syn::Result<()> {
    if !attrs.keys.insert(key) {
        return Err(syn::Error::new(
            span,
            format!(
                "duplicate `{key}` across `#[sqlx(...)]` and `#[crudly(...)]` on the same field"
            ),
        ));
    }
    set(attrs);
    Ok(())
}

fn parse_field_nested(
    attrs: &mut FieldAttrs,
    meta: syn::meta::ParseNestedMeta,
    allow_crudly_id: bool,
) -> syn::Result<()> {
    let path = meta.path.clone();
    if meta.path.is_ident("rename") {
        meta.input.parse::<Token![=]>()?;
        let lit: LitStr = meta.input.parse()?;
        merge_field_key(attrs, "rename", lit.span(), |a| {
            a.rename = Some(lit.value());
        })?;
    } else if meta.path.is_ident("try_from") {
        meta.input.parse::<Token![=]>()?;
        let lit: LitStr = meta.input.parse()?;
        let ty: Type = lit.parse()?;
        merge_field_key(attrs, "try_from", lit.span(), |a| a.try_from = Some(ty))?;
    } else if meta.path.is_ident("try_into") {
        meta.input.parse::<Token![=]>()?;
        let lit: LitStr = meta.input.parse()?;
        let ty: Type = lit.parse()?;
        merge_field_key(attrs, "try_into", lit.span(), |a| a.try_into = Some(ty))?;
    } else if meta.path.is_ident("default") {
        merge_field_key(attrs, "default", path.span(), |a| a.field_default = true)?;
    } else if meta.path.is_ident("flatten") {
        merge_field_key(attrs, "flatten", path.span(), |a| a.flatten = true)?;
    } else if meta.path.is_ident("skip") {
        merge_field_key(attrs, "skip", path.span(), |a| a.skip = true)?;
    } else if meta.path.is_ident("id") {
        if !allow_crudly_id {
            return Err(syn::Error::new_spanned(
                &path,
                "`id` on a field is only valid inside `#[crudly(id)]`, not `#[sqlx(...)]`",
            ));
        }
        if meta.input.peek(Token![=]) {
            return Err(syn::Error::new_spanned(
                &path,
                "use `#[crudly(id)]` without a value",
            ));
        }
        merge_field_key(attrs, "id", path.span(), |a| a.crudly_id = true)?;
    } else if meta.path.is_ident("json") {
        let j = if meta.input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in meta.input);
            let literal: Ident = content.parse()?;
            if literal != "nullable" {
                return Err(syn::Error::new_spanned(
                    literal,
                    "expected `json` or `json(nullable)`",
                ));
            }
            JsonAttr::Nullable
        } else {
            JsonAttr::NonNullable
        };
        merge_field_key(attrs, "json", path.span(), |a| a.json = Some(j))?;
    } else {
        return Err(syn::Error::new_spanned(
            path,
            "unknown field attribute for `#[derive(IntoRow)]` (expected rename, default, flatten, skip, id, try_from, try_into, json)",
        ));
    }
    Ok(())
}

pub(crate) fn parse_field_attrs(field: &Field) -> syn::Result<FieldAttrs> {
    let mut merged = FieldAttrs::empty();

    for attr in &field.attrs {
        if attr.path().is_ident("sqlx") {
            attr.parse_nested_meta(|m| parse_field_nested(&mut merged, m, false))?;
        } else if attr.path().is_ident("crudly") {
            attr.parse_nested_meta(|m| parse_field_nested(&mut merged, m, true))?;
        }
    }

    if merged.flatten && merged.json.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "cannot use `json` and `flatten` together on the same field",
        ));
    }
    if merged.flatten && (merged.try_from.is_some() || merged.try_into.is_some()) {
        return Err(syn::Error::new(
            field.span(),
            "cannot use `flatten` with `try_from` / `try_into`",
        ));
    }
    if let (Some(a), Some(b)) = (&merged.try_from, &merged.try_into) {
        // Avoid syn's `extra-traits` (`PartialEq` for `Type`): stringify so this works with lean syn + minimal-versions
        if a.to_token_stream().to_string() != b.to_token_stream().to_string() {
            return Err(syn::Error::new(
                field.span(),
                "`try_from` and `try_into` must refer to the same type when both are set",
            ));
        }
    }

    if merged.json.is_some() && (merged.try_from.is_some() || merged.try_into.is_some()) {
        return Err(syn::Error::new(
            field.span(),
            "`json` cannot be combined with `try_from` / `try_into` on the same field",
        ));
    }

    if merged.json.is_some() && !cfg!(feature = "json") {
        return Err(syn::Error::new(
            field.span(),
            "`json` field attribute requires the `json` feature on `crudly`",
        ));
    }

    if merged.crudly_id && merged.skip {
        return Err(syn::Error::new(
            field.span(),
            "`#[crudly(id)]` must not be combined with `skip` on the same field",
        ));
    }
    if merged.crudly_id && merged.flatten {
        return Err(syn::Error::new(
            field.span(),
            "`#[crudly(id)]` cannot be used on a flattened field",
        ));
    }
    if merged.crudly_id && merged.json.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "`#[crudly(id)]` cannot be combined with `json` on the same field",
        ));
    }

    Ok(merged)
}

fn merge_container_key(
    seen: &mut HashSet<&'static str>,
    key: &'static str,
    span: Span,
) -> syn::Result<()> {
    if !seen.insert(key) {
        return Err(syn::Error::new(
            span,
            format!(
                "duplicate `{key}` across `#[sqlx(...)]` and `#[crudly(...)]` on the same struct"
            ),
        ));
    }
    Ok(())
}

pub(crate) fn parse_container_attrs(input: &DeriveInput) -> syn::Result<ContainerAttrs> {
    let mut seen = HashSet::new();
    let mut rename_all = None;
    let mut _container_default = false;

    for attr in &input.attrs {
        let mut parse_crudly = |meta: syn::meta::ParseNestedMeta| -> syn::Result<()> {
            let path = meta.path.clone();
            if meta.path.is_ident("rename_all") {
                merge_container_key(&mut seen, "rename_all", path.span())?;
                meta.input.parse::<Token![=]>()?;
                let lit: LitStr = meta.input.parse()?;
                rename_all = Some(RenameAll::parse(&lit)?);
            } else if meta.path.is_ident("default") {
                merge_container_key(&mut seen, "default", path.span())?;
                _container_default = true;
            } else if meta.path.is_ident("table") {
                meta.input.parse::<Token![=]>()?;
                let _: LitStr = meta.input.parse()?;
            } else if meta.path.is_ident("external_ids") || meta.path.is_ident("db_ids") {
                // Crudly-only flags; ignored by IntoRow so both derives can share `#[crudly(...)]`.
            } else if meta.path.is_ident("executor") {
                meta.input.parse::<Token![=]>()?;
                let _: Type = meta.input.parse()?;
            } else {
                return Err(syn::Error::new_spanned(
                    &path,
                    format!(
                        "unknown or unsupported container attribute for `IntoRow`: {}",
                        quote!(#path)
                    ),
                ));
            }
            Ok(())
        };

        if attr.path().is_ident("sqlx") {
            attr.parse_nested_meta(|m| {
                let path = m.path.clone();
                if m.path.is_ident("rename_all") {
                    merge_container_key(&mut seen, "rename_all", path.span())?;
                    m.input.parse::<Token![=]>()?;
                    let lit: LitStr = m.input.parse()?;
                    rename_all = Some(RenameAll::parse(&lit)?);
                } else if m.path.is_ident("default") {
                    merge_container_key(&mut seen, "default", path.span())?;
                    _container_default = true;
                } else if m.path.is_ident("transparent")
                    || m.path.is_ident("type_name")
                    || m.path.is_ident("repr")
                    || m.path.is_ident("no_pg_array")
                {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "this `#[sqlx(...)]` container attribute is not supported on `#[derive(IntoRow)]`",
                    ));
                } else {
                    return Err(syn::Error::new_spanned(
                        &path,
                        format!("unknown `#[sqlx({})]` for IntoRow", quote!(#path)),
                    ));
                }
                Ok(())
            })?;
        } else if attr.path().is_ident("crudly") {
            attr.parse_nested_meta(&mut parse_crudly)?;
        }
    }

    Ok(ContainerAttrs {
        rename_all,
        _container_default,
    })
}

pub(crate) fn column_name_for_field(
    field: &Field,
    attrs: &FieldAttrs,
    rename_all: Option<RenameAll>,
) -> syn::Result<String> {
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(field.span(), "tuple structs are not supported"))?;
    let base = ident.to_string();
    if let Some(r) = &attrs.rename {
        return Ok(r.clone());
    }
    if let Some(ra) = rename_all {
        return Ok(ra.apply(&base));
    }
    Ok(base)
}

fn push_pred_unique(
    preds: &mut Vec<syn::WherePredicate>,
    seen: &mut BTreeSet<String>,
    p: syn::WherePredicate,
) {
    let key = p.to_token_stream().to_string();
    if seen.insert(key) {
        preds.push(p);
    }
}

/// `try_into = "String"` / `try_from = "String"`: bind `field.to_string()` instead of `TryInto`.
fn try_target_is_string(ty: &Type) -> bool {
    let Type::Path(p) = ty else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    last.ident == "String" && last.arguments.is_empty()
}

fn option_inner_type(ty: &Type) -> syn::Result<Type> {
    let Type::Path(p) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "`json(nullable)` requires field type `Option<T>`",
        ));
    };
    let Some(last) = p.path.segments.last() else {
        return Err(syn::Error::new_spanned(
            ty,
            "`json(nullable)` requires field type `Option<T>`",
        ));
    };
    if last.ident != "Option" {
        return Err(syn::Error::new_spanned(
            ty,
            "`json(nullable)` requires field type `Option<T>`",
        ));
    }
    let syn::PathArguments::AngleBracketed(ab) = &last.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            "`json(nullable)` requires field type `Option<T>`",
        ));
    };
    let Some(syn::GenericArgument::Type(inner)) = ab.args.first() else {
        return Err(syn::Error::new_spanned(
            ty,
            "`json(nullable)` requires field type `Option<T>`",
        ));
    };
    Ok(inner.clone())
}

fn add_field_encode_bounds(
    field: &Field,
    fa: &FieldAttrs,
    preds: &mut Vec<syn::WherePredicate>,
    seen: &mut BTreeSet<String>,
) -> syn::Result<()> {
    if fa.skip || fa.flatten || fa.crudly_id {
        return Ok(());
    }
    if let Some(u) = fa.try_from.as_ref().or(fa.try_into.as_ref()) {
        if try_target_is_string(u) {
            let fty = field.ty.clone();
            push_pred_unique(preds, seen, parse_quote!(#fty: ::std::string::ToString));
            push_pred_unique(
                preds,
                seen,
                parse_quote!(::std::string::String: for<'q> ::sqlx::Encode<'q, __CrudlyDb> + ::sqlx::Type<__CrudlyDb>),
            );
        } else {
            let u_ty = u.clone();
            push_pred_unique(
                preds,
                seen,
                parse_quote!(#u_ty: for<'q> ::sqlx::Encode<'q, __CrudlyDb> + ::sqlx::Type<__CrudlyDb>),
            );
        }
        return Ok(());
    }
    if let Some(json_attr) = fa.json {
        let json_inner: Type = match json_attr {
            JsonAttr::NonNullable => field.ty.clone(),
            JsonAttr::Nullable => option_inner_type(&field.ty)?,
        };
        push_pred_unique(
            preds,
            seen,
            parse_quote!(::sqlx::types::Json<#json_inner>: for<'q> ::sqlx::Encode<'q, __CrudlyDb> + ::sqlx::Type<__CrudlyDb>),
        );
        if matches!(json_attr, JsonAttr::Nullable) {
            push_pred_unique(
                preds,
                seen,
                parse_quote!(::std::option::Option<::sqlx::types::Json<#json_inner>>: for<'q> ::sqlx::Encode<'q, __CrudlyDb> + ::sqlx::Type<__CrudlyDb>),
            );
        }
        return Ok(());
    }
    let fty = field.ty.clone();
    push_pred_unique(
        preds,
        seen,
        parse_quote!(#fty: for<'q> ::sqlx::Encode<'q, __CrudlyDb> + ::sqlx::Type<__CrudlyDb>),
    );
    Ok(())
}

fn field_binding_expr(field: &Field, attrs: &FieldAttrs) -> syn::Result<TokenStream> {
    let ident = field.ident.as_ref().expect("named field");

    if attrs.skip || attrs.crudly_id {
        return Ok(quote! {});
    }

    if attrs.flatten {
        let ty = &field.ty;
        return Ok(quote! {
            <#ty as ::crudly::IntoRow<__CrudlyDb>>::bind_arguments(self.#ident, arguments)?;
        });
    }

    let intermediate = attrs.try_from.as_ref().or(attrs.try_into.as_ref());

    if let Some(u_ty) = intermediate {
        if try_target_is_string(u_ty) {
            return Ok(quote! {
                arguments.add((self.#ident).to_string()).map_err(::sqlx::Error::Encode)?;
            });
        }
        return Ok(quote! {
            {
                let __v: #u_ty = ::std::convert::TryInto::try_into(self.#ident)
                    .map_err(|e| ::sqlx::Error::Encode(::std::boxed::Box::new(e)))?;
                arguments.add(__v).map_err(::sqlx::Error::Encode)?;
            }
        });
    }

    if let Some(json_attr) = attrs.json {
        return Ok(match json_attr {
            JsonAttr::NonNullable => quote! {
                arguments
                    .add(::sqlx::types::Json(self.#ident))
                    .map_err(::sqlx::Error::Encode)?;
            },
            JsonAttr::Nullable => {
                let inner_ty = option_inner_type(&field.ty)?;
                quote! {
                    match self.#ident {
                        Some(__crudly_json_inner) => {
                            arguments
                                .add(::sqlx::types::Json(__crudly_json_inner))
                                .map_err(::sqlx::Error::Encode)?;
                        }
                        None => {
                            arguments
                                .add(::std::option::Option::<::sqlx::types::Json<#inner_ty>>::None)
                                .map_err(::sqlx::Error::Encode)?;
                        }
                    }
                }
            }
        });
    }

    Ok(quote! {
        arguments.add(self.#ident).map_err(::sqlx::Error::Encode)?;
    })
}

fn into_row_columns_tokens_for_field(
    field: &Field,
    attrs: &FieldAttrs,
    sql_name: &str,
) -> syn::Result<TokenStream> {
    if attrs.skip || attrs.crudly_id {
        return Ok(quote! {});
    }
    if attrs.flatten {
        let ty = &field.ty;
        return Ok(quote! {
            out.extend(<#ty as ::crudly::IntoRow<__CrudlyDb>>::columns());
        });
    }
    Ok(quote! {
        out.push(#sql_name);
    })
}

pub(crate) fn schema_columns_tokens_for_field(
    field: &Field,
    attrs: &FieldAttrs,
    sql_name: &str,
) -> syn::Result<TokenStream> {
    if attrs.skip {
        return Ok(quote! {});
    }
    if attrs.flatten {
        let ty = &field.ty;
        return Ok(quote! {
            out.extend(<#ty as ::crudly::IntoRow<::sqlx::Any>>::columns());
        });
    }
    Ok(quote! {
        out.push(#sql_name);
    })
}

/// Adds `__CrudlyDb: Database` for the `IntoRow<__CrudlyDb>` impl (name avoids colliding with user generics).
fn generics_with_crudly_db(input: &DeriveInput) -> syn::Generics {
    let mut g = input.generics.clone();
    g.params.push(parse_quote!(__CrudlyDb: ::sqlx::Database));
    g
}

pub fn expand_derive_into_row(input: DeriveInput) -> syn::Result<TokenStream> {
    let container = parse_container_attrs(&input)?;
    let rename_all = container.rename_all;

    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "only structs are supported for `IntoRow`",
        ));
    };
    let syn::Fields::Named(fields_named) = &data.fields else {
        return Err(syn::Error::new(
            input.span(),
            "only structs with named fields are supported for `IntoRow`",
        ));
    };

    let mut column_fragments = Vec::new();
    let mut saw_non_skip = false;

    let mut parsed: Vec<(&Field, FieldAttrs)> = Vec::new();

    for field in &fields_named.named {
        let fa = parse_field_attrs(field)?;
        if fa.flatten || (!fa.skip && !fa.crudly_id) {
            saw_non_skip = true;
        }
        let sql_name = column_name_for_field(field, &fa, rename_all)?;
        column_fragments.push(into_row_columns_tokens_for_field(field, &fa, &sql_name)?);
        parsed.push((field, fa));
    }

    if !saw_non_skip {
        return Err(syn::Error::new(
            input.span(),
            "`IntoRow` requires at least one non-id field that participates in row columns (not `skip` or `#[crudly(id)]`)",
        ));
    }

    let ident = &input.ident;
    let (_, hc_ty_generics, _) = input.generics.split_for_impl();

    let mut bind_fragments = Vec::new();
    let mut preds: Vec<syn::WherePredicate> = Vec::new();
    let mut pred_seen = BTreeSet::new();

    for (field, fa) in &parsed {
        bind_fragments.push(field_binding_expr(field, fa)?);
        add_field_encode_bounds(field, fa, &mut preds, &mut pred_seen)?;

        if fa.skip || fa.crudly_id {
            continue;
        }
        if fa.flatten {
            let ty = &field.ty;
            push_pred_unique(
                &mut preds,
                &mut pred_seen,
                parse_quote!(#ty: ::crudly::IntoRow<__CrudlyDb>),
            );
        }
        if let Some(u_ty) = fa.try_from.as_ref().or(fa.try_into.as_ref()) {
            if !try_target_is_string(u_ty) {
                let fty = &field.ty;
                push_pred_unique(
                    &mut preds,
                    &mut pred_seen,
                    parse_quote!(#fty: ::std::convert::TryInto<#u_ty>),
                );
            }
        }
    }

    let mut g = generics_with_crudly_db(&input);
    if !preds.is_empty() {
        let wc = g.make_where_clause();
        for p in preds {
            wc.predicates.push(p);
        }
    }
    let (impl_generics, _, where_clause) = g.split_for_impl();

    let into_row_impl = quote! {
        impl #impl_generics ::crudly::IntoRow<__CrudlyDb> for #ident #hc_ty_generics
        #where_clause
        {
            fn columns() -> ::std::vec::Vec<&'static str> {
                let mut out = ::std::vec::Vec::new();
                #(#column_fragments)*
                debug_assert!(!out.is_empty(), "IntoRow::columns must not be empty");
                out
            }

            fn bind_arguments(
                self,
                arguments: &mut <__CrudlyDb as ::sqlx::Database>::Arguments,
            ) -> ::sqlx::Result<()> {
                use ::sqlx::Arguments as _;
                #(#bind_fragments)*
                Ok(())
            }
        }
    };

    Ok(into_row_impl)
}
