/// Cleans up double spaces between words
pub fn remove_extra_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_clean_double_spaces() {
        assert_eq!(
            remove_extra_space("This  is   a   test    string"),
            "This is a test string"
        );
    }

    #[test]
    fn should_clean_double_spaces_with_leading_and_trailing_spaces() {
        assert_eq!(
            remove_extra_space("  This  is   a   test    string  "),
            "This is a test string"
        );
    }

    #[test]
    fn should_clean_double_spaces_with_multiple_spaces() {
        assert_eq!(
            remove_extra_space("This    is   a   test    string"),
            "This is a test string"
        );
    }

    #[test]
    fn should_clean_double_spaces_with_no_spaces() {
        assert_eq!(remove_extra_space("Thisisateststring"), "Thisisateststring");
    }

    #[test]
    fn should_clean_double_spaces_with_single_space() {
        assert_eq!(
            remove_extra_space("This is a test string"),
            "This is a test string"
        );
    }

    #[test]
    fn should_clean_double_spaces_with_empty_string() {
        assert_eq!(remove_extra_space(""), "");
    }

    #[test]
    fn should_clean_double_spaces_with_only_spaces() {
        assert_eq!(remove_extra_space("     "), "");
    }

    #[test]
    fn should_clean_double_spaces_with_special_characters() {
        assert_eq!(
            remove_extra_space("This  is   a   test    string! @# $%^ &*()"),
            "This is a test string! @# $%^ &*()"
        );
    }
}
