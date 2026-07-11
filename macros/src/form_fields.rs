use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

use crate::case::snake_to_title;

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
