use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, LitStr};

use crate::helpers::{apply_rename_all, find_serde_rename, find_serde_rename_all};

pub(crate) fn derive_db_enum_impl(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new(name.span(), "DbEnum only supports enums")
                .to_compile_error()
                .into();
        }
    };

    for v in variants {
        if !v.fields.is_empty() {
            return syn::Error::new(
                v.ident.span(),
                format!(
                    "DbEnum only supports unit variants, `{}` has fields",
                    v.ident
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let (rename_all, type_name_override) = match find_db_enum_meta(&input.attrs) {
        Ok((v, type_name)) => (v.or_else(|| find_serde_rename_all(&input.attrs)), type_name),
        Err(e) => return e.to_compile_error().into(),
    };

    let mut entries: Vec<(_, String, Vec<String>)> = Vec::with_capacity(variants.len());
    for v in variants {
        let (rename, aliases) = match parse_variant_attrs(&v.attrs) {
            Ok(parsed) => parsed,
            Err(e) => return e.to_compile_error().into(),
        };
        let primary = rename
            .or_else(|| find_serde_rename(&v.attrs))
            .unwrap_or_else(|| apply_rename_all(&v.ident.to_string(), rename_all.as_deref()));
        entries.push((&v.ident, primary, aliases));
    }

    let mut seen: HashMap<String, String> = HashMap::new();
    for (vid, primary, aliases) in &entries {
        for s in std::iter::once(primary).chain(aliases.iter()) {
            if let Some(prev) = seen.get(s) {
                return syn::Error::new(
                    vid.span(),
                    format!("DbEnum string {s:?} on variant `{vid}` collides with `{prev}`"),
                )
                .to_compile_error()
                .into();
            }
            seen.insert(s.clone(), vid.to_string());
        }
    }

    let display_arms = entries
        .iter()
        .map(|(vid, primary, _)| quote! { Self::#vid => f.write_str(#primary) });

    let as_ref_arms = entries
        .iter()
        .map(|(vid, primary, _)| quote! { Self::#vid => #primary });

    let from_str_arms = entries.iter().map(|(vid, primary, aliases)| {
        let mut all = vec![primary.clone()];
        all.extend(aliases.iter().cloned());
        quote! { #(#all)|* => Ok(Self::#vid), }
    });

    let name_str = name.to_string();

    let type_name: String = type_name_override
        .unwrap_or_else(|| apply_rename_all(&name.to_string(), Some("snake_case")));

    // Postgres array type name for a base type `foo` is `_foo`.
    let array_type_name = format!("_{type_name}");

    let expanded = quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(#display_arms),*
                }
            }
        }

        impl ::std::convert::AsRef<str> for #name {
            fn as_ref(&self) -> &str {
                match self {
                    #(#as_ref_arms),*
                }
            }
        }

        impl ::std::str::FromStr for #name {
            type Err = ::utils::AppError;
            fn from_str(s: &str) -> ::utils::Result<Self> {
                match s {
                    #(#from_str_arms)*
                    other => Err(::utils::AppError::BadRequest(
                        format!("Invalid {} value: {other}", #name_str)
                    )),
                }
            }
        }

        // sqlx Type + Encode + Decode for the Postgres database type. A PG enum
        // is bound/decoded by its label text, keyed to the enum's Postgres type
        // name (`#type_name`).
        #[cfg(feature = "server")]
        impl ::sqlx::Type<::sqlx::Postgres> for #name {
            fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                ::sqlx::postgres::PgTypeInfo::with_name(#type_name)
            }
            fn compatible(t: &::sqlx::postgres::PgTypeInfo) -> bool {
                use ::sqlx::TypeInfo;
                *t == <Self as ::sqlx::Type<::sqlx::Postgres>>::type_info()
                    || t.name().eq_ignore_ascii_case(#type_name)
            }
        }

        // Enables binding `Vec<#name>` / `&[#name]` as a Postgres `#type_name[]`
        // array (e.g. `unnest($1::#type_name[])`).
        #[cfg(feature = "server")]
        impl ::sqlx::postgres::PgHasArrayType for #name {
            fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                ::sqlx::postgres::PgTypeInfo::with_name(#array_type_name)
            }
        }

        #[cfg(feature = "server")]
        impl<'q> ::sqlx::Encode<'q, ::sqlx::Postgres> for #name {
            fn encode_by_ref(
                &self,
                buf: &mut ::sqlx::postgres::PgArgumentBuffer,
            ) -> ::std::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                let s = <Self as ::std::convert::AsRef<str>>::as_ref(self);
                <&str as ::sqlx::Encode<::sqlx::Postgres>>::encode_by_ref(&s, buf)
            }
        }

        #[cfg(feature = "server")]
        impl<'r> ::sqlx::Decode<'r, ::sqlx::Postgres> for #name {
            fn decode(
                value: ::sqlx::postgres::PgValueRef<'r>,
            ) -> ::std::result::Result<Self, ::sqlx::error::BoxDynError> {
                let s = <::std::string::String as ::sqlx::Decode<::sqlx::Postgres>>::decode(value)?;
                <Self as ::std::str::FromStr>::from_str(&s).map_err(|e| e.to_string().into())
            }
        }

        // Stable shims so existing call sites — `Status::from_row(&row, idx)` —
        // keep compiling.
        #[cfg(feature = "server")]
        impl #name {
            pub fn from_row(row: &::utils::Row, idx: usize) -> ::utils::Result<Self> {
                use ::sqlx::Row;
                let s: ::std::string::String = row.try_get_unchecked(idx).map_err(|e| {
                    ::utils::AppError::InternalServer(format!(
                        concat!(#name_str, " decode[{}]: {}"), idx, e
                    ))
                })?;
                <Self as ::std::str::FromStr>::from_str(&s)
            }

            pub fn from_row_opt(row: &::utils::Row, idx: usize) -> ::utils::Result<::std::option::Option<Self>> {
                use ::sqlx::Row;
                let opt: ::std::option::Option<::std::string::String> = row.try_get_unchecked(idx).map_err(|e| {
                    ::utils::AppError::InternalServer(format!(
                        concat!(#name_str, " decode_opt[{}]: {}"), idx, e
                    ))
                })?;
                match opt {
                    ::std::option::Option::None => ::std::result::Result::Ok(::std::option::Option::None),
                    ::std::option::Option::Some(s) => <Self as ::std::str::FromStr>::from_str(&s).map(::std::option::Option::Some),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parses enum-level `#[db_enum(...)]` attributes, returning
/// `(rename_all, type_name)`. Errors on any key other than `rename_all` or
/// `type_name`.
fn find_db_enum_meta(attrs: &[Attribute]) -> syn::Result<(Option<String>, Option<String>)> {
    let mut rename_all = None;
    let mut type_name = None;
    for attr in attrs {
        if !attr.path().is_ident("db_enum") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let lit: LitStr = meta.value()?.parse()?;
                rename_all = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("type_name") {
                let lit: LitStr = meta.value()?.parse()?;
                type_name = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error("unknown db_enum attribute (expected `rename_all` or `type_name`)"))
            }
        })?;
    }
    Ok((rename_all, type_name))
}

fn parse_variant_attrs(attrs: &[Attribute]) -> syn::Result<(Option<String>, Vec<String>)> {
    let mut rename = None;
    let mut aliases = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("db_enum") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let lit: LitStr = meta.value()?.parse()?;
                rename = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("alias") {
                let lit: LitStr = meta.value()?.parse()?;
                aliases.push(lit.value());
                Ok(())
            } else {
                Err(meta.error("unknown db_enum attribute (expected `rename` or `alias`)"))
            }
        })?;
    }
    Ok((rename, aliases))
}
