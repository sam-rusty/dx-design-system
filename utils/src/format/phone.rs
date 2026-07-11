pub fn format_phone(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return String::new();
    }
    let local = if digits.len() > 10 && digits.starts_with('1') {
        &digits[1..]
    } else {
        &digits
    };
    match local.len() {
        0 => String::new(),
        1..=3 => std::format!("+1 ({local})"),
        4..=6 => std::format!("+1 ({}) {}", &local[..3], &local[3..]),
        _ => std::format!(
            "+1 ({}) {}-{}",
            &local[..3],
            &local[3..6],
            &local[6..10.min(local.len())]
        ),
    }
}

pub fn parse_phone(formatted: &str) -> String {
    formatted
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .strip_prefix('1')
        .map(String::from)
        .unwrap_or_else(|| formatted.chars().filter(|c| c.is_ascii_digit()).collect())
}
