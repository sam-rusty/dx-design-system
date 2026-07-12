use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

use crate::case::{
    apply_rename_all, find_serde_rename, find_serde_rename_all, find_strum_to_string,
    pascal_to_title,
};

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

    // FormValue (typed form store) is only expressible when every variant is
    // a unit variant — data variants have no string form.
    let all_unit = variants.iter().all(|v| v.fields.is_empty());
    let form_value_impl = if all_unit {
        let to_arms = variants.iter().map(|v| {
            let ident = &v.ident;
            let value = find_serde_rename(&v.attrs)
                .unwrap_or_else(|| apply_rename_all(&ident.to_string(), rename_all.as_deref()));
            quote! { Self::#ident => #value.to_string() }
        });
        let from_arms = variants.iter().map(|v| {
            let ident = &v.ident;
            let value = find_serde_rename(&v.attrs)
                .unwrap_or_else(|| apply_rename_all(&ident.to_string(), rename_all.as_deref()));
            quote! { #value => Ok(Self::#ident) }
        });
        quote! {
            impl components::FormValue for #name {
                fn to_input(&self) -> String {
                    match self {
                        #(#to_arms),*
                    }
                }

                fn from_input(input: &str) -> Result<Self, components::ParseError> {
                    match input {
                        #(#from_arms,)*
                        _ => Err(components::ParseError::new("Invalid option")),
                    }
                }
            }
        }
    } else {
        quote! {}
    };

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

        #form_value_impl
    };

    TokenStream::from(expanded)
}
