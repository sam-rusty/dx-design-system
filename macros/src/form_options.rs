use ds_enum_options::EnumOption;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

fn options_const(name: &syn::Ident, options: &[EnumOption]) -> proc_macro2::TokenStream {
    let entries = options.iter().map(|option| {
        let (value, label) = (&option.value, &option.label);
        quote! { (#value, #label) }
    });
    quote! {
        impl #name {
            pub const OPTIONS: &[(&str, &str)] = &[#(#entries),*];
        }
    }
}

pub fn derive_enum_options(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match ds_enum_options::options(&input, true) {
        Ok(options) => options_const(&input.ident, &options).into(),
        Err(error) => error.to_compile_error().into(),
    }
}

pub fn derive_form_options(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let mut existing = false;
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("form_options"))
    {
        if let Err(error) = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("existing") {
                existing = true;
                Ok(())
            } else {
                Err(meta.error("expected `existing`"))
            }
        }) {
            return error.to_compile_error().into();
        }
    }

    let options = match ds_enum_options::options(&input, false) {
        Ok(options) => options,
        Err(error) => return error.to_compile_error().into(),
    };
    let first_value = &options[0].value;

    // FormValue (typed form store) needs a wire string for every variant; data
    // variants and `serde(skip)` variants have none. Such enums still get
    // `OPTIONS` / `FormSchema` for the dynamic form, but binding them to a
    // typed control fails with a `FormValue` bound error at the `field:` prop.
    let all_unit = matches!(&input.data, Data::Enum(data) if data.variants.len() == options.len());
    let form_value_impl = if all_unit {
        let to_arms = options.iter().map(|option| {
            let (ident, value) = (&option.ident, &option.value);
            quote! { Self::#ident => #value.to_string() }
        });
        let from_arms = options.iter().map(|option| {
            let (ident, value) = (&option.ident, &option.value);
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

    let options_impl = if existing {
        quote! {}
    } else {
        options_const(name, &options)
    };
    let expanded = quote! {
        #options_impl

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
