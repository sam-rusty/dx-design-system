use syn::{Attribute, LitStr};

pub(crate) fn split_pascal(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut words = Vec::new();
    let mut current = String::new();
    for i in 0..chars.len() {
        let ch = chars[i];
        if ch.is_uppercase() && !current.is_empty() {
            let prev_upper = chars[i - 1].is_uppercase();
            let next_lower = chars.get(i + 1).is_some_and(|c| c.is_lowercase());
            // Split on lower→upper (PhoneCall → Phone|Call) or end-of-acronym
            // (IRSPayment → IRS|Payment), but keep runs of caps together.
            if !prev_upper || next_lower {
                words.push(std::mem::take(&mut current));
            }
        }
        current.push(ch);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

pub(crate) fn apply_rename_all(variant: &str, rule: Option<&str>) -> String {
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
        Some("PascalCase") | None => variant.to_string(),
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

pub(crate) fn find_serde_rename_all(attrs: &[Attribute]) -> Option<String> {
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

pub(crate) fn find_serde_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut result = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
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
