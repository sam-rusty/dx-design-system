pub fn format_percent(raw: &str) -> String {
    let raw = raw.trim().trim_end_matches('%').trim();
    if raw.is_empty() {
        return String::new();
    }
    format!("{raw}%")
}

pub fn parse_percent(formatted: &str) -> String {
    formatted.trim_end_matches('%').trim().to_string()
}

pub fn filter_percent(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut has_dot = false;
    for c in input.chars() {
        if c.is_ascii_digit() || (c == '.' && !has_dot) {
            if c == '.' {
                has_dot = true;
            }
            result.push(c);
        }
    }
    result
}

pub fn clamp_percent(raw: &str, min: f64, max: f64) -> String {
    let cleaned = raw.trim().trim_end_matches('%').trim();
    if cleaned.is_empty() {
        return String::new();
    }
    match cleaned.parse::<f64>() {
        Ok(v) => {
            let clamped = v.clamp(min, max);
            if (clamped - v).abs() < f64::EPSILON {
                raw.to_string()
            } else if clamped.fract() == 0.0 {
                format!("{}", clamped as i64)
            } else {
                format!("{clamped}")
            }
        }
        Err(_) => raw.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_percent_empty() {
        assert_eq!(format_percent(""), "");
        assert_eq!(format_percent("  "), "");
    }

    #[test]
    fn test_format_percent_basic() {
        assert_eq!(format_percent("50"), "50%");
        assert_eq!(format_percent("0"), "0%");
        assert_eq!(format_percent("100"), "100%");
    }

    #[test]
    fn test_format_percent_decimal() {
        assert_eq!(format_percent("33.5"), "33.5%");
        assert_eq!(format_percent("0.1"), "0.1%");
    }

    #[test]
    fn test_format_percent_strips_existing_suffix() {
        assert_eq!(format_percent("75%"), "75%");
        assert_eq!(format_percent("  75%  "), "75%");
    }

    #[test]
    fn test_parse_percent_basic() {
        assert_eq!(parse_percent("50%"), "50");
        assert_eq!(parse_percent("100%"), "100");
        assert_eq!(parse_percent("33.5%"), "33.5");
    }

    #[test]
    fn test_parse_percent_no_suffix() {
        assert_eq!(parse_percent("50"), "50");
        assert_eq!(parse_percent("  75  "), "75");
    }

    #[test]
    fn test_parse_percent_roundtrip() {
        for input in ["0", "50", "99.9", "100"] {
            let formatted = format_percent(input);
            let parsed = parse_percent(&formatted);
            assert_eq!(parsed, input, "roundtrip failed for {input}");
        }
    }

    #[test]
    fn test_filter_percent_digits_only() {
        assert_eq!(filter_percent("abc123def"), "123");
        assert_eq!(filter_percent("50%"), "50");
    }

    #[test]
    fn test_filter_percent_decimal() {
        assert_eq!(filter_percent("33.5"), "33.5");
        assert_eq!(filter_percent("33.5.6"), "33.56");
    }

    #[test]
    fn test_filter_percent_no_negative() {
        assert_eq!(filter_percent("-10"), "10");
    }

    #[test]
    fn test_clamp_percent_within_range() {
        assert_eq!(clamp_percent("50", 0.0, 100.0), "50");
        assert_eq!(clamp_percent("0", 0.0, 100.0), "0");
        assert_eq!(clamp_percent("100", 0.0, 100.0), "100");
    }

    #[test]
    fn test_clamp_percent_above_max() {
        assert_eq!(clamp_percent("150", 0.0, 100.0), "100");
        assert_eq!(clamp_percent("200.5", 0.0, 100.0), "100");
    }

    #[test]
    fn test_clamp_percent_below_min() {
        assert_eq!(clamp_percent("-5", 0.0, 100.0), "0");
        assert_eq!(clamp_percent("-0.1", 0.0, 100.0), "0");
    }

    #[test]
    fn test_clamp_percent_custom_range() {
        assert_eq!(clamp_percent("50", 10.0, 90.0), "50");
        assert_eq!(clamp_percent("5", 10.0, 90.0), "10");
        assert_eq!(clamp_percent("95", 10.0, 90.0), "90");
    }

    #[test]
    fn test_clamp_percent_empty() {
        assert_eq!(clamp_percent("", 0.0, 100.0), "");
        assert_eq!(clamp_percent("  ", 0.0, 100.0), "");
    }

    #[test]
    fn test_clamp_percent_with_suffix() {
        assert_eq!(clamp_percent("50%", 0.0, 100.0), "50%");
        assert_eq!(clamp_percent("150%", 0.0, 100.0), "100");
    }

    #[test]
    fn test_clamp_percent_decimal_preserved() {
        assert_eq!(clamp_percent("33.5", 0.0, 100.0), "33.5");
    }
}
