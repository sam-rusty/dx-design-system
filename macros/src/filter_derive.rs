use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitStr, Type};

// --- FilterOption ---

pub(crate) fn derive_filter_option_impl(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new(name.span(), "FilterOption only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let rename_all = find_serde_rename_all(&input.attrs);

    let entries: Vec<_> = variants
        .iter()
        .filter(|v| v.fields.is_empty())
        .map(|v| {
            let variant_name = v.ident.to_string();
            let value = find_option_value(&v.attrs)
                .unwrap_or_else(|| apply_rename_all(&variant_name, rename_all.as_deref()));
            let label =
                find_option_label(&v.attrs).unwrap_or_else(|| pascal_to_title_words(&variant_name));
            quote! {
                ::utils::FilterEnumOption { value: #value, label: #label }
            }
        })
        .collect();

    let expanded = quote! {
        impl ::utils::FilterOption for #name {
            fn options() -> &'static [::utils::FilterEnumOption] {
                &[#(#entries),*]
            }
        }
    };

    TokenStream::from(expanded)
}

fn find_option_value(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("option") {
            continue;
        }
        let mut v = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("value") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                v = Some(lit.value());
            }
            Ok(())
        });
        if v.is_some() {
            return v;
        }
    }
    None
}

fn find_option_label(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("option") {
            continue;
        }
        let mut v = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("label") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                v = Some(lit.value());
            }
            Ok(())
        });
        if v.is_some() {
            return v;
        }
    }
    None
}

// --- FilterColumns ---

#[derive(Default)]
struct EnumFilterMeta {
    alias: Option<String>,
}

#[derive(Default)]
struct VariantFilterMeta {
    label: Option<String>,
    key: Option<String>,
    sql_col: Option<String>,
    col_alias: Option<String>,
    hidden: bool,
    ty: Option<Ident>,
    enum_ty: Option<Type>,
    widget: Option<Ident>,
}

pub(crate) fn derive_filter_columns_impl(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    let enum_meta = parse_enum_filter_meta(&input.attrs);
    let default_table_alias = enum_meta.alias.unwrap_or_default();

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new(name.span(), "FilterColumns only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let mut key_arms = Vec::new();
    let mut label_arms = Vec::new();
    let mut col_type_arms = Vec::new();
    let mut hidden_arms = Vec::new();
    let mut sql_arms = Vec::new();
    let mut all_variants = Vec::new();

    for v in variants {
        if !matches!(v.fields, Fields::Unit) {
            return syn::Error::new(
                v.ident.span(),
                format!(
                    "FilterColumns only supports unit variants, found `{}` with fields",
                    v.ident
                ),
            )
            .to_compile_error()
            .into();
        }
        let vid = &v.ident;
        all_variants.push(quote! { Self::#vid });

        let vm = parse_variant_filter_meta(&v.attrs);
        let variant_name = v.ident.to_string();
        let key = vm
            .key
            .clone()
            .unwrap_or_else(|| apply_rename_all(&variant_name, Some("snake_case")));
        let label = vm
            .label
            .clone()
            .unwrap_or_else(|| pascal_to_title_words(&variant_name));

        let sql_col = vm.sql_col.clone().unwrap_or_else(|| key.clone());

        let table = vm
            .col_alias
            .as_deref()
            .unwrap_or(if default_table_alias.is_empty() {
                ""
            } else {
                default_table_alias.as_str()
            });
        let sql_expr = if table.is_empty() {
            sql_col.clone()
        } else {
            format!("{table}.{sql_col}")
        };

        key_arms.push(quote! { Self::#vid => #key });
        label_arms.push(quote! { Self::#vid => #label });

        let hidden = vm.hidden;
        hidden_arms.push(quote! { Self::#vid => #hidden });

        let col_type = build_column_type_quote(&vm);
        col_type_arms.push(quote! { Self::#vid => #col_type });

        sql_arms.push(quote! { Self::#vid => #sql_expr });
    }

    let expanded = quote! {
        impl ::utils::FilterColumns for #name {
            fn key(self) -> &'static str {
                match self {
                    #(#key_arms),*
                }
            }
            fn label(self) -> &'static str {
                match self {
                    #(#label_arms),*
                }
            }
            fn col_type(self) -> ::utils::ColumnType {
                match self {
                    #(#col_type_arms),*
                }
            }
            fn hidden(self) -> bool {
                match self {
                    #(#hidden_arms),*
                }
            }
            fn all() -> &'static [Self] {
                &[#(#all_variants),*]
            }
        }

        #[cfg(feature = "server")]
        impl ::utils::FilterColumnsSql for #name {
            fn sql_expr(self) -> &'static str {
                match self {
                    #(#sql_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn build_column_type_quote(vm: &VariantFilterMeta) -> proc_macro2::TokenStream {
    if let Some(ref et) = vm.enum_ty {
        let w = vm
            .widget
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "Select".to_string());
        let widget = match w.as_str() {
            "Checkbox" => quote! { ::utils::EnumWidget::Checkbox },
            "Radio" => quote! { ::utils::EnumWidget::Radio },
            _ => quote! { ::utils::EnumWidget::Select },
        };
        quote! {
            ::utils::ColumnType::Enum {
                options: <#et as ::utils::FilterOption>::options(),
                widget: #widget,
            }
        }
    } else {
        let ty = vm
            .ty
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "Text".to_string());
        match ty.as_str() {
            "Email" => quote! { ::utils::ColumnType::Email },
            "Number" => quote! { ::utils::ColumnType::Number },
            "Date" => quote! { ::utils::ColumnType::Date },
            "Bool" => quote! { ::utils::ColumnType::Bool },
            _ => quote! { ::utils::ColumnType::Text },
        }
    }
}

fn parse_enum_filter_meta(attrs: &[Attribute]) -> EnumFilterMeta {
    let mut m = EnumFilterMeta::default();
    for attr in attrs {
        if !attr.path().is_ident("filter") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("alias") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                m.alias = Some(lit.value());
            }
            Ok(())
        });
    }
    m
}

fn parse_variant_filter_meta(attrs: &[Attribute]) -> VariantFilterMeta {
    let mut m = VariantFilterMeta::default();
    for attr in attrs {
        if !attr.path().is_ident("filter") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("hidden") {
                m.hidden = true;
                return Ok(());
            }
            if meta.path.is_ident("label") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                m.label = Some(lit.value());
                return Ok(());
            }
            if meta.path.is_ident("key") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                m.key = Some(lit.value());
                return Ok(());
            }
            if meta.path.is_ident("sql_col") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                m.sql_col = Some(lit.value());
                return Ok(());
            }
            if meta.path.is_ident("alias") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                m.col_alias = Some(lit.value());
                return Ok(());
            }
            if meta.path.is_ident("ty") {
                let value = meta.value()?;
                let expr: syn::Expr = value.parse()?;
                if let syn::Expr::Path(p) = expr
                    && let Some(seg) = p.path.segments.last()
                {
                    m.ty = Some(seg.ident.clone());
                }
                return Ok(());
            }
            if meta.path.is_ident("enum_ty") {
                let value = meta.value()?;
                let ty: Type = value.parse()?;
                m.enum_ty = Some(ty);
                return Ok(());
            }
            if meta.path.is_ident("widget") {
                let value = meta.value()?;
                let expr: syn::Expr = value.parse()?;
                if let syn::Expr::Path(p) = expr
                    && let Some(seg) = p.path.segments.last()
                {
                    m.widget = Some(seg.ident.clone());
                }
                return Ok(());
            }
            Ok(())
        });
    }
    m
}

fn find_serde_rename_all(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut result = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
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
                        out.push_str(c.as_str().to_lowercase().as_str());
                    }
                }
            }
            out
        }
        Some("PascalCase") => variant.to_string(),
        Some("snake_case") | None => words
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

fn pascal_to_title_words(s: &str) -> String {
    split_pascal(s).join(" ")
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
