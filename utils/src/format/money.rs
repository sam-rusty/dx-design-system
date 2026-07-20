/// Major-unit decimal string → minor-unit integer string ("25.5", 2 → "2550").
/// Rounds half away from zero at the cutoff. `None` on empty/malformed input.
pub fn major_to_minor(major: &str, decimals: u32) -> Option<String> {
    let s = major.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let all_digits = |p: &str| p.chars().all(|c| c.is_ascii_digit());
    if !all_digits(int_part) || !all_digits(frac_part) {
        return None;
    }
    let d = decimals as usize;
    let mut frac: String = frac_part.chars().take(d).collect();
    let round_up = frac_part.as_bytes().get(d).is_some_and(|b| *b >= b'5');
    while frac.len() < d {
        frac.push('0');
    }
    let mut minor: i128 = format!("{int_part}{frac}").parse().ok()?;
    if round_up {
        minor += 1;
    }
    if neg {
        minor = -minor;
    }
    Some(minor.to_string())
}

/// Minor-unit integer string → major-unit decimal string ("2550", 2 → "25.5").
/// Passes empty/malformed input through unchanged (the input shows what's stored).
pub fn minor_to_major(minor: &str, decimals: u32) -> String {
    let s = minor.trim();
    let (neg, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    if decimals == 0 || digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return s.to_string();
    }
    let d = decimals as usize;
    let padded = format!("{:0>width$}", digits, width = d + 1);
    let (int_part, frac_part) = padded.split_at(padded.len() - d);
    let frac_trimmed = frac_part.trim_end_matches('0');
    let sign = if neg { "-" } else { "" };
    if frac_trimmed.is_empty() {
        format!("{sign}{int_part}")
    } else {
        format!("{sign}{int_part}.{frac_trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn major_to_minor_scales_and_rounds() {
        assert_eq!(major_to_minor("25", 2).as_deref(), Some("2500"));
        assert_eq!(major_to_minor("25.5", 2).as_deref(), Some("2550"));
        assert_eq!(major_to_minor("25.005", 2).as_deref(), Some("2501")); // half up
        assert_eq!(major_to_minor("25.004", 2).as_deref(), Some("2500"));
        assert_eq!(major_to_minor("999.995", 2).as_deref(), Some("100000")); // carry
        assert_eq!(major_to_minor("6500", 0).as_deref(), Some("6500"));
        assert_eq!(major_to_minor("6500.7", 0).as_deref(), Some("6501"));
        assert_eq!(major_to_minor("-3.5", 2).as_deref(), Some("-350")); // sign preserved; validator rejects later
    }

    #[test]
    fn major_to_minor_rejects_junk() {
        assert_eq!(major_to_minor("", 2), None);
        assert_eq!(major_to_minor("  ", 2), None);
        assert_eq!(major_to_minor(".", 2), None);
        assert_eq!(major_to_minor("12a", 2), None);
        assert_eq!(major_to_minor("1.2.3", 2), None);
    }

    #[test]
    fn minor_to_major_trims_cleanly() {
        assert_eq!(minor_to_major("2550", 2), "25.5");
        assert_eq!(minor_to_major("2500", 2), "25");
        assert_eq!(minor_to_major("5", 2), "0.05");
        assert_eq!(minor_to_major("6500", 0), "6500");
        assert_eq!(minor_to_major("-350", 2), "-3.5");
        assert_eq!(minor_to_major("", 2), "");
    }
}
