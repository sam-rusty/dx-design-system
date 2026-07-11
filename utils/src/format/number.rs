/// Formats a raw numeric string with comma-separated thousands.
/// e.g. "1234567" -> "1,234,567", "-1234.56" -> "-1,234.56"
pub fn format_number(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    let negative = raw.starts_with('-');
    let (integer, decimal) = match raw.find('.') {
        Some(pos) => (&raw[..pos], Some(&raw[pos..])),
        None => (raw, None),
    };
    let digits: String = integer.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() && decimal.is_none() {
        return raw.to_string();
    }
    // Strip leading zeros; keep at least one digit when a decimal part follows.
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let mut result = String::new();
    if negative {
        result.push('-');
    }
    let len = digits.len();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    if let Some(dec) = decimal {
        result.push_str(dec);
    }
    result
}

/// Strips comma grouping from a formatted number string.
/// e.g. "1,234,567" -> "1234567"
pub fn parse_number(formatted: &str) -> String {
    formatted.replace(',', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_empty() {
        assert_eq!(format_number(""), "");
        assert_eq!(format_number("  "), "");
    }

    #[test]
    fn test_format_number_small() {
        assert_eq!(format_number("0"), "0");
        assert_eq!(format_number("1"), "1");
        assert_eq!(format_number("12"), "12");
        assert_eq!(format_number("123"), "123");
    }

    #[test]
    fn test_format_number_with_commas() {
        assert_eq!(format_number("1234"), "1,234");
        assert_eq!(format_number("12345"), "12,345");
        assert_eq!(format_number("123456"), "123,456");
        assert_eq!(format_number("1234567"), "1,234,567");
        assert_eq!(format_number("1000000"), "1,000,000");
    }

    #[test]
    fn test_format_number_negative() {
        assert_eq!(format_number("-1"), "-1");
        assert_eq!(format_number("-1234"), "-1,234");
        assert_eq!(format_number("-1234567"), "-1,234,567");
    }

    #[test]
    fn test_format_number_decimal() {
        assert_eq!(format_number("1234.56"), "1,234.56");
        assert_eq!(format_number("1000000.99"), "1,000,000.99");
        assert_eq!(format_number("-1234.5"), "-1,234.5");
        assert_eq!(format_number("0.5"), "0.5");
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("1,234"), "1234");
        assert_eq!(parse_number("1,234,567"), "1234567");
        assert_eq!(parse_number("1,000,000.99"), "1000000.99");
        assert_eq!(parse_number("-1,234"), "-1234");
        assert_eq!(parse_number("123"), "123");
        assert_eq!(parse_number(""), "");
    }

    #[test]
    fn test_format_parse_number_roundtrip() {
        let raw = "1234567";
        assert_eq!(parse_number(&format_number(raw)), raw);

        let raw = "-9876543.21";
        assert_eq!(parse_number(&format_number(raw)), raw);
    }

}
