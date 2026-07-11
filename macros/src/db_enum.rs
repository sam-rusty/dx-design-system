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

    let rename_all = match find_db_enum_meta(&input.attrs) {
        Ok(v) => v.or_else(|| find_serde_rename_all(&input.attrs)),
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

    };

    TokenStream::from(expanded)
}

/// Parses enum-level `#[db_enum(...)]` attributes, returning `rename_all`.
/// Errors on any key other than `rename_all`.
fn find_db_enum_meta(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let mut rename_all = None;
    for attr in attrs {
        if !attr.path().is_ident("db_enum") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let lit: LitStr = meta.value()?.parse()?;
                rename_all = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error("unknown db_enum attribute (expected `rename_all`)"))
            }
        })?;
    }
    Ok(rename_all)
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
