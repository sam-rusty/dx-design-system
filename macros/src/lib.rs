mod db_enum;
mod filter_derive;
mod helpers;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, Item, PathArguments, Type, parse_macro_input,
};

struct Items(Vec<Item>);

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Items(items))
    }
}

fn cfg_feature_items(input: TokenStream, feature: &str) -> TokenStream {
    let Items(items) = parse_macro_input!(input as Items);
    let expanded = quote! {
        #( #[cfg(feature = #feature)] #items )*
    };
    expanded.into()
}

/// Applies `#[cfg(feature = "server")]` to every item inside the block.
///
/// ```ignore
/// macros::on_server! {
///     use std::str::FromStr;
///     use crate::AutomationStepKind;
/// }
/// ```
#[proc_macro]
pub fn on_server(input: TokenStream) -> TokenStream {
    cfg_feature_items(input, "server")
}

/// Applies `#[cfg(feature = "web")]` to every item inside the block.
///
/// ```ignore
/// macros::on_web! {
///     use web_sys::window;
/// }
/// ```
#[proc_macro]
pub fn on_web(input: TokenStream) -> TokenStream {
    cfg_feature_items(input, "web")
}

#[proc_macro_derive(FilterOption, attributes(option))]
pub fn derive_filter_option(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    filter_derive::derive_filter_option_impl(input)
}

/// Derives Display, FromStr, and AsRef<str> for unit enums. See `libs/macros/README.md`
/// for usage and attribute table.
#[proc_macro_derive(DbEnum, attributes(db_enum))]
pub fn derive_db_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    db_enum::derive_db_enum_impl(input)
}

#[proc_macro_derive(FilterColumns, attributes(filter))]
pub fn derive_filter_columns(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    filter_derive::derive_filter_columns_impl(input)
}

/// Generates typed field constants for each struct field. See `libs/macros/README.md` for usage.
#[proc_macro_derive(FormFields, attributes(field))]
pub fn derive_form_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new(
                    data.fields.span(),
                    "FormFields only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(name.span(), "FormFields only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let consts = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        let label = find_field_label(f).unwrap_or_else(|| snake_to_title(&field_name.to_string()));
        let field_ty = &f.ty;

        if let Some(inner_ty) = vec_inner_type(field_ty).or_else(|| option_vec_inner_type(field_ty)) {
            let required = !is_option_type(field_ty);
            quote! {
                #[allow(non_upper_case_globals)]
                pub const #field_name: components::FieldArray<Self, #inner_ty> = components::FieldArray::new(#field_str, #label, #required, <#inner_ty as components::FormSchema>::FIELD_TYPE);
            }
        } else {
            let required = !is_option_type(field_ty);
            quote! {
                #[allow(non_upper_case_globals)]
                pub const #field_name: components::FieldName<Self, #field_ty> = components::FieldName::new(#field_str, #label, #required, <#field_ty as components::FormSchema>::FIELD_TYPE);
            }
        }
    });

    let schema_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        let field_ty = &f.ty;
        quote! {
            (#field_str, <#field_ty as components::FormSchema>::json_schema())
        }
    });

    let expanded = quote! {
        impl #name {
            #(#consts)*
        }

        impl components::FormSchema for #name {
            const FIELD_TYPE: components::FieldType = components::FieldType::Object;

            fn json_schema() -> components::serde_json::Value {
                components::serde_json::Value::Object(
                    [#(#schema_fields),*]
                        .into_iter()
                        .collect::<Vec<(&str, components::serde_json::Value)>>()
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect()
                )
            }
        }
    };

    TokenStream::from(expanded)
}

fn vec_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
    {
        for arg in &args.args {
            if let GenericArgument::Type(inner) = arg {
                return Some(inner);
            }
        }
    }
    None
}

fn option_vec_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
    {
        for arg in &args.args {
            if let GenericArgument::Type(inner) = arg {
                return vec_inner_type(inner);
            }
        }
    }
    None
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Generates `const OPTIONS: &[(&str, &str)]` for unit enums. See `libs/macros/README.md` for usage.
#[proc_macro_derive(FormOptions)]
pub fn derive_form_options(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new(name.span(), "FormOptions only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let rename_all = find_serde_rename_all(&input.attrs);

    let unit_variants: Vec<_> = variants
        .iter()
        .filter(|v| v.fields.is_empty())
        .map(|v| {
            let variant_name = v.ident.to_string();
            let value = find_serde_rename(&v.attrs)
                .unwrap_or_else(|| apply_rename_all(&variant_name, rename_all.as_deref()));
            let label =
                find_strum_to_string(&v.attrs).unwrap_or_else(|| pascal_to_title(&variant_name));
            (value, label)
        })
        .collect();

    let entries: Vec<_> = unit_variants
        .iter()
        .map(|(value, label)| quote! { (#value, #label) })
        .collect();

    let first_value = unit_variants
        .first()
        .map(|(v, _)| v.clone())
        .unwrap_or_default();

    let expanded = quote! {
        impl #name {
            pub const OPTIONS: &[(&str, &str)] = &[#(#entries),*];
        }

        impl components::FormSchema for #name {
            const FIELD_TYPE: components::FieldType = components::FieldType::String;

            fn json_schema() -> components::serde_json::Value {
                components::serde_json::Value::String(#first_value.to_string())
            }
        }
    };

    TokenStream::from(expanded)
}

fn find_serde_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut result = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result = Some(lit.value());
            }
            Ok(())
        });
        if result.is_some() {
            return result;
        }
    }
    None
}

fn find_serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut result = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result = Some(lit.value());
            }
            Ok(())
        });
        if result.is_some() {
            return result;
        }
    }
    None
}

/// `#[strum(to_string = "...")]` on a variant — used as the radio/select label when present.
fn find_strum_to_string(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("strum") {
            continue;
        }
        let mut result = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("to_string") || meta.path.is_ident("serialize") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result = Some(lit.value());
            }
            Ok(())
        });
        if result.is_some() {
            return result;
        }
    }
    None
}

fn apply_rename_all(variant: &str, rule: Option<&str>) -> String {
    let words = split_pascal(variant);
    match rule {
        Some("lowercase") => words.join("").to_lowercase(),
        Some("UPPERCASE") => words.join("").to_uppercase(),
        Some("camelCase") => {
            let mut out = String::new();
            for (i, w) in words.iter().enumerate() {
                if i == 0 {
                    out.push_str(&w.to_lowercase());
                } else {
                    let mut c = w.chars();
                    if let Some(first) = c.next() {
                        out.extend(first.to_uppercase());
                        out.push_str(&c.as_str().to_lowercase());
                    }
                }
            }
            out
        }
        Some("PascalCase") => variant.to_string(),
        Some("snake_case") => words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("_"),
        Some("SCREAMING_SNAKE_CASE") => words
            .iter()
            .map(|w| w.to_uppercase())
            .collect::<Vec<_>>()
            .join("_"),
        Some("kebab-case") => words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("-"),
        Some("SCREAMING-KEBAB-CASE") => words
            .iter()
            .map(|w| w.to_uppercase())
            .collect::<Vec<_>>()
            .join("-"),
        _ => variant.to_string(),
    }
}

fn pascal_to_title(s: &str) -> String {
    split_pascal(s).join(" ")
}

fn snake_to_title(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{}{}", upper, chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn find_field_label(field: &syn::Field) -> Option<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("field") {
            continue;
        }
        let mut label = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("label") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                label = Some(lit.value());
            }
            Ok(())
        });
        if label.is_some() {
            return label;
        }
    }
    None
}

fn split_pascal(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in s.chars() {
        if ch.is_uppercase() && !current.is_empty() {
            words.push(current);
            current = String::new();
        }
        current.push(ch);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Derives step metadata for multi-step form enums. See `libs/macros/README.md` for usage.
#[proc_macro_derive(Steps, attributes(step))]
pub fn derive_steps(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new(name.span(), "Steps can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    for v in variants {
        if !v.fields.is_empty() {
            return syn::Error::new(
                v.ident.span(),
                format!(
                    "Steps only supports unit variants, but `{}` has fields",
                    v.ident
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let variant_idents: Vec<_> = variants.iter().map(|v| &v.ident).collect();
    let count = variant_idents.len();

    let mut titles = Vec::new();
    let mut descriptions = Vec::new();

    for v in variants {
        let ident = &v.ident;
        let mut title = None;
        let mut desc = None;

        for attr in &v.attrs {
            if !attr.path().is_ident("step") {
                continue;
            }
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("title") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    title = Some(lit.value());
                } else if meta.path.is_ident("description") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    desc = Some(lit.value());
                }
                Ok(())
            });
        }

        let t = title.unwrap_or_else(|| pascal_to_title(&ident.to_string()));
        let d = desc.unwrap_or_default();
        titles.push(t);
        descriptions.push(d);
    }

    let expanded = quote! {
        impl #name {
            pub const ALL: &[#name] = &[#(#name::#variant_idents),*];
            pub const COUNT: usize = #count;
            pub const TITLES: &[&str] = &[#(#titles),*];
            pub const DESCRIPTIONS: &[&str] = &[#(#descriptions),*];
        }
    };

    TokenStream::from(expanded)
}
