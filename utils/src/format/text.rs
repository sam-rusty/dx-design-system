pub fn merge(strs: &[&str]) -> String {
    strs.iter()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `snake_case` → `Title Case` (`"first_name"` → `"First Name"`). Empty segments are dropped.
///
/// Duplicated in `ds-macros` (`macros/src/case.rs`): a proc-macro crate cannot depend on this
/// crate without pulling `dioxus` into the compiler-side build, so the two copies stay in
/// sync by hand and by their tests.
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
    fn merge_skips_blank_entries() {
        assert_eq!(merge(&["a", "", "  ", "b"]), "a b");
        assert_eq!(merge(&[]), "");
        assert_eq!(merge(&["", "   "]), "");
        // Entries are not trimmed, only filtered.
        assert_eq!(merge(&[" a ", "b"]), " a  b");
    }

    #[test]
    fn snake_to_title_handles_edges() {
        assert_eq!(snake_to_title("first_name"), "First Name");
        assert_eq!(snake_to_title("__a_b"), "A B");
        assert_eq!(snake_to_title("a__b_"), "A B");
        assert_eq!(snake_to_title(""), "");
        assert_eq!(snake_to_title("already"), "Already");
        // Only the first character is upper-cased; the rest is preserved.
        assert_eq!(snake_to_title("userID"), "UserID");
        assert_eq!(snake_to_title("éclair_au_café"), "Éclair Au Café");
    }
}
