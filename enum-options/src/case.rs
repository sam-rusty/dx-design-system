/// Splits on every uppercase character — exactly serde's `rename_all` word split, so
/// `apply_rename_all` output matches serde. Use `split_words_for_title` for display text.
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

/// Word split for human-readable titles: keeps acronyms and digit runs together
/// (`"HTTPServer"` → `["HTTP", "Server"]`, `"Step1A"` → `["Step", "1", "A"]`, `"UserID"` →
/// `["User", "ID"]`). Only for display strings — `apply_rename_all` must keep using
/// [`split_pascal`], whose output matches serde's `rename_all` exactly.
pub fn split_words_for_title(s: &str) -> Vec<String> {
    #[derive(Clone, Copy, PartialEq)]
    enum Kind {
        Upper,
        Lower,
        Digit,
        Other,
    }
    fn kind(c: char) -> Kind {
        if c.is_uppercase() {
            Kind::Upper
        } else if c.is_lowercase() {
            Kind::Lower
        } else if c.is_ascii_digit() {
            Kind::Digit
        } else {
            Kind::Other
        }
    }

    let chars: Vec<char> = s.chars().filter(|c| *c != '_').collect();
    let mut words = Vec::new();
    let mut current = String::new();
    for (i, &c) in chars.iter().enumerate() {
        let k = kind(c);
        let prev = if i > 0 {
            Some(kind(chars[i - 1]))
        } else {
            None
        };
        let next = chars.get(i + 1).map(|n| kind(*n));
        let boundary = match (prev, k) {
            (None, _) => false,
            // lower→Upper, digit→Upper, Upper→Digit, lower→Digit
            (Some(Kind::Lower), Kind::Upper)
            | (Some(Kind::Digit), Kind::Upper)
            | (Some(Kind::Upper), Kind::Digit)
            | (Some(Kind::Lower), Kind::Digit) => true,
            // digit→lower ("V2x" → "V 2 x")
            (Some(Kind::Digit), Kind::Lower) => true,
            // End of an acronym: "HTTPServer" → split before the 'S' that starts "Server".
            (Some(Kind::Upper), Kind::Upper) => next == Some(Kind::Lower),
            _ => false,
        };
        if boundary && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        current.push(c);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

pub fn pascal_to_title(s: &str) -> String {
    split_words_for_title(s).join(" ")
}

/// Every `serde(rename_all = "...")` rule [`apply_rename_all`] reproduces.
pub const RENAME_ALL_RULES: [&str; 8] = [
    "lowercase",
    "UPPERCASE",
    "PascalCase",
    "camelCase",
    "snake_case",
    "SCREAMING_SNAKE_CASE",
    "kebab-case",
    "SCREAMING-KEBAB-CASE",
];

/// Mirrors serde_derive's `RenameRule::apply_to_variant`; `None` and unknown rules
/// leave the variant untouched.
pub fn apply_rename_all(variant: &str, rule: Option<&str>) -> String {
    let join = |separator: &str, upper: bool| {
        split_pascal(variant)
            .iter()
            .map(|w| {
                if upper {
                    w.to_ascii_uppercase()
                } else {
                    w.to_ascii_lowercase()
                }
            })
            .collect::<Vec<_>>()
            .join(separator)
    };
    match rule {
        Some("lowercase") => variant.to_ascii_lowercase(),
        Some("UPPERCASE") => variant.to_ascii_uppercase(),
        Some("camelCase") => {
            let mut chars = variant.chars();
            chars
                .next()
                .map(|first| first.to_ascii_lowercase().to_string() + chars.as_str())
                .unwrap_or_default()
        }
        Some("snake_case") => join("_", false),
        Some("SCREAMING_SNAKE_CASE") => join("_", true),
        Some("kebab-case") => join("-", false),
        Some("SCREAMING-KEBAB-CASE") => join("-", true),
        _ => variant.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_pascal_splits_on_every_uppercase() {
        assert_eq!(split_pascal("SuperAdmin"), vec!["Super", "Admin"]);
        assert_eq!(
            split_pascal("HTTPServer"),
            vec!["H", "T", "T", "P", "Server"]
        );
        assert_eq!(split_pascal("Step1A"), vec!["Step1", "A"]);
        assert_eq!(split_pascal("lower"), vec!["lower"]);
        assert!(split_pascal("").is_empty());
    }

    #[test]
    fn pascal_to_title_keeps_acronyms_and_digits_together() {
        assert_eq!(pascal_to_title("SuperAdmin"), "Super Admin");
        assert_eq!(pascal_to_title("HTTPServer"), "HTTP Server");
        assert_eq!(pascal_to_title("UserID"), "User ID");
        assert_eq!(pascal_to_title("HTML"), "HTML");
        assert_eq!(pascal_to_title("Step1A"), "Step 1 A");
        assert_eq!(pascal_to_title("Version2Beta"), "Version 2 Beta");
        assert_eq!(pascal_to_title("Review"), "Review");
        assert_eq!(pascal_to_title("Step_Two"), "Step Two");
        assert_eq!(pascal_to_title(""), "");
    }

    /// Reference implementation of serde's `rename_all` rules (serde_derive `RenameRule`).
    fn serde_rename(variant: &str, rule: &str) -> String {
        // serde splits PascalCase on every uppercase character (`Case::Upper`), exactly like
        // `split_pascal`; this mirrors `RenameRule::apply_to_variant`.
        match rule {
            "lowercase" => variant.to_ascii_lowercase(),
            "UPPERCASE" => variant.to_ascii_uppercase(),
            "PascalCase" => variant.to_owned(),
            "camelCase" => variant[..1].to_ascii_lowercase() + &variant[1..],
            "snake_case" => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            "SCREAMING_SNAKE_CASE" => serde_rename(variant, "snake_case").to_ascii_uppercase(),
            "kebab-case" => serde_rename(variant, "snake_case").replace('_', "-"),
            "SCREAMING-KEBAB-CASE" => {
                serde_rename(variant, "SCREAMING_SNAKE_CASE").replace('_', "-")
            }
            _ => variant.to_owned(),
        }
    }

    #[test]
    fn apply_rename_all_matches_serde_for_every_rule() {
        for variant in [
            "Admin",
            "SuperAdmin",
            "HTTPServer",
            "UserID",
            "Step1A",
            "OnHold",
        ] {
            for rule in RENAME_ALL_RULES {
                assert_eq!(
                    apply_rename_all(variant, Some(rule)),
                    serde_rename(variant, rule),
                    "{variant} / {rule}"
                );
            }
            assert_eq!(apply_rename_all(variant, None), variant);
            assert_eq!(apply_rename_all(variant, Some("unknown")), variant);
        }
    }
}
