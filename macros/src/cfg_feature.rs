use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Item, parse_macro_input};

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

pub fn cfg_feature_items(input: TokenStream, feature: &str) -> TokenStream {
    let Items(items) = parse_macro_input!(input as Items);
    let expanded = quote! {
        #( #[cfg(feature = #feature)] #items )*
    };
    expanded.into()
}
