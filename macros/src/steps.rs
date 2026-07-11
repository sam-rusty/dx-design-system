use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

use crate::case::pascal_to_title;

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
