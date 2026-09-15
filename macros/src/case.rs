/// Copy of `ds_utils::format::snake_to_title`: a proc-macro crate cannot depend on `ds-utils`
/// (it would pull `dioxus` into the compiler-side build). Keep the two in sync.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_to_title_matches_utils_copy() {
        assert_eq!(snake_to_title("first_name"), "First Name");
        assert_eq!(snake_to_title("__a_b"), "A B");
        assert_eq!(snake_to_title(""), "");
    }
}
