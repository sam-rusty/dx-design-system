use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

use crate::case::{apply_rename_all, find_serde_rename, find_serde_rename_all, find_strum_to_string, pascal_to_title};

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
