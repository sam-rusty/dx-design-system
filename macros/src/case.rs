pub fn split_pascal(s: &str) -> Vec<String> {
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

pub fn pascal_to_title(s: &str) -> String {
    split_pascal(s).join(" ")
}

pub fn snake_to_title(s: &str) -> String {
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

pub fn apply_rename_all(variant: &str, rule: Option<&str>) -> String {
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

pub fn find_serde_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
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

pub fn find_serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
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
pub fn find_strum_to_string(attrs: &[syn::Attribute]) -> Option<String> {
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
