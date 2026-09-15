//! Framework-independent enum metadata shared by web forms and API generators.
//! [`options`] takes a parsed `DeriveInput`; [`parse_options`] / [`labels`] take
//! tokens so consumers can use a different version of `syn`.

use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use syn::ext::IdentExt;
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Fields, Meta, MetaList, Token, punctuated::Punctuated,
};

pub mod case;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumOption {
    pub ident: Ident,
    pub value: String,
    pub label: String,
}

/// Token boundary for [`options`].
pub fn parse_options(input: TokenStream, strict: bool) -> syn::Result<Vec<EnumOption>> {
    options(&syn::parse2(input)?, strict)
}

/// Resolve labels independently, before another macro injects Display attributes.
pub fn labels(input: TokenStream) -> syn::Result<Vec<String>> {
    let input: DeriveInput = syn::parse2(input)?;
    enum_data(&input)?
        .variants
        .iter()
        .map(|v| label(&v.attrs, &v.ident.unraw().to_string()))
        .collect()
}

fn enum_data(input: &DeriveInput) -> syn::Result<&DataEnum> {
    match &input.data {
        Data::Enum(data) => Ok(data),
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "options require an enum",
        )),
    }
}

fn nested(list: &MetaList) -> syn::Result<Punctuated<Meta, Token![,]>> {
    list.parse_args_with(Punctuated::parse_terminated)
}

fn metas(attrs: &[Attribute], namespace: &str) -> syn::Result<Vec<Meta>> {
    let mut result = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident(namespace)) {
        result.extend(nested(attr.meta.require_list()?)?);
    }
    Ok(result)
}

fn is_one_of(meta: &Meta, keys: &[&str]) -> bool {
    keys.iter().any(|key| meta.path().is_ident(key))
}

fn conditional_metadata(meta: &Meta) -> syn::Result<bool> {
    if is_one_of(meta, &["serde", "strum", "ts"]) {
        return Ok(true);
    }
    if let Meta::List(list) = meta
        && list.path.is_ident("cfg_attr")
    {
        for meta in nested(list)?.iter().skip(1) {
            if conditional_metadata(meta)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn string(meta: &Meta) -> syn::Result<String> {
    if let Meta::NameValue(nv) = meta
        && let syn::Expr::Lit(expr) = &nv.value
        && let syn::Lit::Str(value) = &expr.lit
    {
        return Ok(value.value());
    }
    Err(syn::Error::new_spanned(meta, "expected a string literal"))
}

fn set_once(slot: &mut Option<String>, value: String, at: &Meta, what: &str) -> syn::Result<()> {
    if slot.replace(value).is_some() {
        return Err(syn::Error::new_spanned(at, format!("duplicate {what}")));
    }
    Ok(())
}

fn label(attrs: &[Attribute], variant: &str) -> syn::Result<String> {
    let mut result = None;
    for meta in metas(attrs, "strum")? {
        if meta.path().is_ident("to_string") {
            set_once(&mut result, string(&meta)?, &meta, "option label")?;
        }
    }
    Ok(result.unwrap_or_else(|| case::pascal_to_title(variant)))
}

fn names(metas: &[Meta], key: &str) -> syn::Result<(Option<String>, Option<String>)> {
    let mut serialize = None;
    let mut deserialize = None;
    for meta in metas.iter().filter(|m| m.path().is_ident(key)) {
        let (ser, de) = if let Meta::List(list) = meta {
            let mut ser = None;
            let mut de = None;
            for item in nested(list)? {
                let slot = if item.path().is_ident("serialize") {
                    &mut ser
                } else if item.path().is_ident("deserialize") {
                    &mut de
                } else {
                    return Err(syn::Error::new_spanned(
                        item,
                        "expected serialize or deserialize",
                    ));
                };
                set_once(slot, string(&item)?, &item, "directional rename")?;
            }
            (ser, de)
        } else {
            let value = string(meta)?;
            (Some(value.clone()), Some(value))
        };
        for (slot, next) in [(&mut serialize, ser), (&mut deserialize, de)] {
            if let Some(next) = next {
                set_once(slot, next, meta, "rename")?;
            }
        }
    }
    Ok((serialize, deserialize))
}

/// Structural requirements for selectable wire enums: a non-generic, externally
/// tagged string enum whose non-skipped variants are all plain unit variants.
fn check_wire_shape(input: &DeriveInput, data: &DataEnum) -> syn::Result<()> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "wire options do not support generic enums",
        ));
    }
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg_attr"))
    {
        if conditional_metadata(&attr.meta)? {
            return Err(syn::Error::new_spanned(
                attr,
                "wire options do not support conditional serialization or label metadata",
            ));
        }
    }
    for meta in metas(&input.attrs, "serde")? {
        if !is_one_of(&meta, &["rename", "rename_all"]) {
            return Err(syn::Error::new_spanned(
                meta,
                "wire options only support externally tagged string enums with serde rename/rename_all",
            ));
        }
    }
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "wire options only support unit variants",
            ));
        }
        if variant
            .attrs
            .iter()
            .any(|a| is_one_of(&a.meta, &["cfg", "cfg_attr"]))
        {
            return Err(syn::Error::new_spanned(
                variant,
                "wire options do not support conditional variants",
            ));
        }
        let attrs = metas(&variant.attrs, "serde")?;
        if attrs.iter().any(|m| m.path().is_ident("skip")) {
            continue;
        }
        for meta in attrs {
            if !is_one_of(&meta, &["rename", "alias"]) {
                return Err(syn::Error::new_spanned(
                    meta,
                    "wire options only support symmetric variants with serde rename/alias/skip",
                ));
            }
        }
    }
    Ok(())
}

/// Strict mode is for selectable wire enums: the shape must pass
/// [`check_wire_shape`] and every option must round-trip as its canonical JSON
/// string. Legacy FormOptions can still expose the unit variants of a mixed
/// enum, without claiming its full wire type.
pub fn options(input: &DeriveInput, strict: bool) -> syn::Result<Vec<EnumOption>> {
    let data = enum_data(input)?;
    if strict {
        check_wire_shape(input, data)?;
    }
    let (ser_case, de_case) = names(&metas(&input.attrs, "serde")?, "rename_all")?;
    for rule in [&ser_case, &de_case].into_iter().flatten() {
        if !case::RENAME_ALL_RULES.contains(&rule.as_str()) {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "unsupported serde rename_all rule",
            ));
        }
    }
    let mut result = Vec::new();
    // Strict only: every accepted wire string → owning variant. Aliases are
    // checked after all values so a value/alias clash blames the alias.
    let mut accepted = HashMap::new();
    let mut aliases = Vec::new();
    for variant in &data.variants {
        if !variant.fields.is_empty() {
            continue;
        }
        let attrs = metas(&variant.attrs, "serde")?;
        if attrs.iter().any(|m| m.path().is_ident("skip")) {
            continue;
        }
        let (ser, de) = names(&attrs, "rename")?;
        let name = variant.ident.unraw().to_string();
        let value = ser.unwrap_or_else(|| case::apply_rename_all(&name, ser_case.as_deref()));
        if strict {
            let input_value =
                de.unwrap_or_else(|| case::apply_rename_all(&name, de_case.as_deref()));
            if value != input_value {
                return Err(syn::Error::new_spanned(
                    variant,
                    "wire options require matching serialize and deserialize values",
                ));
            }
            if accepted.insert(value.clone(), &variant.ident).is_some() {
                return Err(syn::Error::new_spanned(
                    variant,
                    "duplicate wire option value",
                ));
            }
            for meta in attrs.iter().filter(|m| m.path().is_ident("alias")) {
                aliases.push((string(meta)?, &variant.ident, meta.clone()));
            }
        }
        result.push(EnumOption {
            ident: variant.ident.clone(),
            value,
            label: label(&variant.attrs, &name)?,
        });
    }
    if result.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "options require at least one non-skipped unit variant",
        ));
    }
    for (alias, owner, meta) in aliases {
        if accepted
            .insert(alias, owner)
            .is_some_and(|previous| previous != owner)
        {
            return Err(syn::Error::new_spanned(
                meta,
                "serde alias collides with another option value or alias",
            ));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn triples(options: &[EnumOption]) -> Vec<(String, &str, &str)> {
        options
            .iter()
            .map(|o| (o.ident.to_string(), o.value.as_str(), o.label.as_str()))
            .collect()
    }

    #[test]
    fn resolves_canonical_values_and_display_labels_independently() {
        let options = parse_options(quote! {
            #[serde(rename_all = "camelCase")]
            enum Choice {
                HTTPServer,
                #[serde(alias = "old", rename = "wire")]
                #[strum(serialize = "old-long-alias", to_string = "Display", serialize = "another")]
                UserID,
                #[serde(skip)] Hidden,
            }
        }, true).unwrap();
        assert_eq!(
            triples(&options),
            vec![
                ("HTTPServer".into(), "hTTPServer", "HTTP Server"),
                ("UserID".into(), "wire", "Display"),
            ]
        );
    }

    #[test]
    fn rejects_shapes_that_cannot_be_selectable_wire_strings() {
        for (input, message) in [
            (
                quote!(
                    struct No;
                ),
                "require an enum",
            ),
            (
                quote!(
                    enum Empty {}
                ),
                "at least one",
            ),
            (
                quote!(
                    enum Payload {
                        Unit,
                        Data(String),
                    }
                ),
                "unit variants",
            ),
            (
                quote!(
                    enum EmptyTuple {
                        Empty(),
                    }
                ),
                "unit variants",
            ),
            (
                quote!(
                    enum Generic<T> {
                        Value(T),
                    }
                ),
                "generic enums",
            ),
            (
                quote!(
                    #[serde(tag = "kind")]
                    enum Tagged {
                        Value,
                    }
                ),
                "externally tagged",
            ),
            (
                quote!(
                    #[serde(untagged)]
                    enum Untagged {
                        Value,
                    }
                ),
                "externally tagged",
            ),
            (
                quote!(
                    #[serde(into = "String")]
                    enum Converted {
                        Value,
                    }
                ),
                "externally tagged",
            ),
            (
                quote!(
                    enum Conditional {
                        #[cfg(feature = "x")]
                        Value,
                    }
                ),
                "conditional variants",
            ),
            (
                quote!(
                    enum Direction {
                        #[serde(rename(serialize = "out", deserialize = "in"))]
                        Value,
                    }
                ),
                "matching serialize",
            ),
            (
                quote!(
                    #[serde(rename_all(serialize = "snake_case"))]
                    enum Direction {
                        SomeValue,
                    }
                ),
                "matching serialize",
            ),
            (
                quote!(
                    enum Skipped {
                        #[serde(skip_serializing)]
                        Value,
                    }
                ),
                "symmetric variants",
            ),
            (
                quote!(
                    enum Skipped {
                        #[serde(skip_deserializing)]
                        Value,
                    }
                ),
                "symmetric variants",
            ),
            (
                quote!(
                    enum Duplicate {
                        #[serde(rename = "x")]
                        One,
                        #[serde(rename = "x")]
                        Two,
                    }
                ),
                "duplicate wire",
            ),
            (
                quote!(
                    enum Alias {
                        #[serde(alias = "First")]
                        Other,
                        First,
                    }
                ),
                "alias collides",
            ),
            (
                quote!(
                    enum Alias {
                        #[serde(alias = "old")]
                        First,
                        #[serde(alias = "old")]
                        Other,
                    }
                ),
                "alias collides",
            ),
            (
                quote!(
                    #[serde(rename_all = "unknown")]
                    enum Bad {
                        Value,
                    }
                ),
                "unsupported serde",
            ),
        ] {
            let error = parse_options(input.clone(), true).unwrap_err();
            assert!(error.to_string().contains(message), "{input}: {error}");
        }
    }

    #[test]
    fn retains_legacy_mixed_form_options_without_claiming_full_wire_type() {
        let input = quote!(
            enum Mixed {
                First,
                Payload(String),
            }
        );
        assert_eq!(parse_options(input.clone(), false).unwrap().len(), 1);
        assert!(parse_options(input, true).is_err());
    }

    #[test]
    fn matching_directional_names_and_self_aliases_are_supported() {
        let options = parse_options(quote! {
            #[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
            enum Same { #[serde(alias = "value", rename(serialize = "value", deserialize = "value"))] SomeValue }
        }, true).unwrap();
        assert_eq!(options[0].value, "value");
    }

    #[test]
    fn raw_identifiers_keep_their_ident_but_drop_the_prefix_from_values() {
        let options = parse_options(
            quote! {
                #[serde(rename_all = "snake_case")]
                enum Keyword { r#type }
            },
            true,
        )
        .unwrap();
        assert_eq!(options[0].ident.to_string(), "r#type");
        assert_eq!(options[0].value, "type");
        assert_eq!(options[0].label, "type");
    }

    #[test]
    fn conditional_form_derives_are_allowed_but_wire_renames_are_rejected() {
        assert!(
            parse_options(
                quote! {
                    #[cfg_attr(feature = "web", derive(FormOptions))]
                    #[cfg_attr(feature = "web", form_options(existing))]
                    enum Choice { Value }
                },
                true
            )
            .is_ok()
        );
        assert!(
            parse_options(
                quote! {
                    #[cfg_attr(feature = "web", serde(rename_all = "snake_case"))]
                    enum Choice { SomeValue }
                },
                true
            )
            .unwrap_err()
            .to_string()
            .contains("conditional serialization")
        );
    }
}
