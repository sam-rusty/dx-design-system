//! Phone number display formatting.
//!
//! A [`PhoneFormat`] is a display pattern: `#` stands for a national digit,
//! every other character is a literal. [`PhoneFormat::US`] (`+1 (###) ###-####`)
//! is the default; any other country is the pattern for it, written at the call
//! site — there is no country table to keep in sync.
//!
//! A country whose numbers come in several lengths gets several patterns, and
//! the tightest one the number fits in is used:
//!
//! ```ignore
//! use ds_utils::format::{PhoneFormat, format_phone_with};
//!
//! let uk = PhoneFormat::new("+44 #### ######");
//! assert_eq!(format_phone_with("7911123456", uk), "+44 7911 123456");
//!
//! let de = PhoneFormat::many(&["+49 ### #######", "+49 ### ########"]);
//! assert_eq!(format_phone_with("1512345678", de), "+49 151 2345678");
//! assert_eq!(format_phone_with("15112345678", de), "+49 151 12345678");
//! ```

/// The character a national digit fills in a [`PhoneFormat`] pattern.
const DIGIT_SLOT: char = '#';

/// The display format of a phone number: one pattern, or several for a country
/// whose numbers come in different lengths. Each pattern spends one
/// [`DIGIT_SLOT`] (`#`) per national digit and renders every other character
/// literally; the digits it writes before its first `#` are the calling code —
/// display-only, never part of the stored value.
///
/// Patterns of one format are layouts of the same country, so they are expected
/// to share a calling code.
///
/// ```ignore
/// PhoneFormat::new("+44 #### ######")   // +44 7911 123456
/// PhoneFormat::new("+33 # ## ## ## ##") // +33 6 12 34 56 78
/// PhoneFormat::new("### ### ####")      // no calling code at all
/// PhoneFormat::many(&["+49 ### #######", "+49 ### ########"])
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PhoneFormat {
    /// A single pattern, used at every length.
    One(&'static str),
    /// Several patterns; the tightest one the number fits in wins.
    Many(&'static [&'static str]),
}

impl PhoneFormat {
    /// United States / Canada — `+1 (555) 123-4567`. The default.
    pub const US: Self = Self::new("+1 (###) ###-####");

    /// A format from one pattern (`"+44 #### ######"`).
    pub const fn new(pattern: &'static str) -> Self {
        Self::One(pattern)
    }

    /// A format from several patterns, one per number length.
    pub const fn many(patterns: &'static [&'static str]) -> Self {
        Self::Many(patterns)
    }

    /// Every pattern this format can render, in the order given.
    pub fn patterns(&self) -> &[&'static str] {
        match self {
            Self::One(pattern) => std::slice::from_ref(pattern),
            Self::Many(patterns) => patterns,
        }
    }

    /// The pattern this format renders `raw` with: the one of the smallest
    /// capacity that still holds every digit, or — when the number outgrows
    /// them all — the largest.
    pub fn pattern_for(&self, raw: &str) -> &'static str {
        let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
        let mut tightest_fit: Option<(&'static str, usize)> = None;
        let mut largest: Option<(&'static str, usize)> = None;

        for &pattern in self.patterns() {
            let capacity = Self::national_len(pattern);
            let fits = national_digits(&digits, pattern).len() <= capacity;
            if fits && tightest_fit.is_none_or(|(_, best)| capacity < best) {
                tightest_fit = Some((pattern, capacity));
            }
            if largest.is_none_or(|(_, best)| capacity > best) {
                largest = Some((pattern, capacity));
            }
        }
        tightest_fit.or(largest).map_or("", |(pattern, _)| pattern)
    }

    /// How many national digits `pattern` holds — its `#` count.
    pub fn national_len(pattern: &str) -> usize {
        pattern.chars().filter(|c| *c == DIGIT_SLOT).count()
    }

    /// The calling code `pattern` writes before its first `#`, without the `+`.
    /// Empty when the pattern has no calling code.
    pub fn calling_code(pattern: &str) -> String {
        pattern
            .chars()
            .take_while(|c| *c != DIGIT_SLOT)
            .filter(char::is_ascii_digit)
            .collect()
    }
}

impl Default for PhoneFormat {
    fn default() -> Self {
        Self::US
    }
}

impl From<&'static str> for PhoneFormat {
    fn from(pattern: &'static str) -> Self {
        Self::One(pattern)
    }
}

impl From<&'static [&'static str]> for PhoneFormat {
    fn from(patterns: &'static [&'static str]) -> Self {
        Self::Many(patterns)
    }
}

impl<const N: usize> From<&'static [&'static str; N]> for PhoneFormat {
    fn from(patterns: &'static [&'static str; N]) -> Self {
        Self::Many(patterns)
    }
}

/// Formats raw national digits for display in the US / Canada format.
/// See [`format_phone_with`] for other countries.
pub fn format_phone(raw: &str) -> String {
    format_phone_with(raw, PhoneFormat::US)
}

/// Strips the display formatting of a US / Canada number back to raw national
/// digits. See [`parse_phone_with`] for other countries.
pub fn parse_phone(formatted: &str) -> String {
    parse_phone_with(formatted, PhoneFormat::US)
}

/// Formats `raw` against the pattern `format` picks for it, filling the `#`
/// slots left to right so a partially typed number formats too. A slot's
/// closing bracket is emitted as soon as the slot has a digit (`+1 (5)`); other
/// literals wait for the digit that follows them. A leading calling code in
/// `raw` is dropped, and digits past the pattern's capacity are not displayed.
pub fn format_phone_with(raw: &str, format: PhoneFormat) -> String {
    let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return String::new();
    }
    let chosen = format.pattern_for(&digits);
    let mut national = national_digits(&digits, chosen).chars();
    let pattern: Vec<char> = chosen.chars().collect();

    let mut out = String::with_capacity(pattern.len());
    // Literals are held back until a digit follows them, so a number that ends
    // on a group boundary doesn't render a dangling separator.
    let mut pending = String::new();
    let mut ran_out_at = None;

    for (i, ch) in pattern.iter().copied().enumerate() {
        if ch != DIGIT_SLOT {
            pending.push(ch);
            continue;
        }
        match national.next() {
            Some(digit) => {
                out.push_str(&pending);
                pending.clear();
                out.push(digit);
            }
            None => {
                ran_out_at = Some(i);
                break;
            }
        }
    }

    if let Some(i) = ran_out_at {
        if pending.is_empty() {
            // Stopped mid-group: the group's own closing literals are further
            // along the pattern, past the slots no digit reached.
            pending = pattern[i..]
                .iter()
                .skip_while(|c| **c == DIGIT_SLOT)
                .take_while(|c| **c != DIGIT_SLOT)
                .collect();
        }
        out.extend(pending.chars().take_while(is_closing_bracket));
    }
    out
}

/// Strips `format`'s display formatting back to raw national digits. The
/// calling code is display-only, so a leading one is always dropped.
pub fn parse_phone_with(formatted: &str, format: PhoneFormat) -> String {
    let digits: String = formatted.chars().filter(char::is_ascii_digit).collect();
    let calling_code = PhoneFormat::calling_code(format.pattern_for(&digits));
    digits
        .strip_prefix(&calling_code)
        .map(String::from)
        .unwrap_or(digits)
}

/// `digits` with `pattern`'s calling code dropped — but only when the number is
/// too long without it, so a full national number that happens to start with
/// the calling code stays intact.
fn national_digits<'a>(digits: &'a str, pattern: &str) -> &'a str {
    match digits.strip_prefix(&PhoneFormat::calling_code(pattern)) {
        Some(rest) if digits.len() > PhoneFormat::national_len(pattern) => rest,
        _ => digits,
    }
}

fn is_closing_bracket(c: &char) -> bool {
    matches!(c, ')' | ']' | '}')
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: PhoneFormat = PhoneFormat::new("+44 #### ######");
    const FR: PhoneFormat = PhoneFormat::new("+33 # ## ## ## ##");
    const NO_CODE: PhoneFormat = PhoneFormat::new("### ### ####");
    /// Two lengths, listed longest first — order must not decide the pick.
    const DE: PhoneFormat = PhoneFormat::many(&["+49 ### ########", "+49 ### #######"]);

    #[test]
    fn default_format_is_us() {
        assert_eq!(PhoneFormat::default(), PhoneFormat::US);
    }

    #[test]
    fn builds_from_a_pattern_or_a_pattern_list() {
        assert_eq!(PhoneFormat::from("+44 #### ######"), GB);
        assert_eq!(
            PhoneFormat::from(&["+49 ### ########", "+49 ### #######"]),
            DE
        );
        assert_eq!(GB.patterns(), &["+44 #### ######"]);
        assert_eq!(DE.patterns().len(), 2);
    }

    #[test]
    fn national_len_counts_the_slots() {
        assert_eq!(PhoneFormat::national_len("+1 (###) ###-####"), 10);
        assert_eq!(PhoneFormat::national_len("+33 # ## ## ## ##"), 9);
    }

    #[test]
    fn calling_code_is_the_digits_before_the_first_slot() {
        assert_eq!(PhoneFormat::calling_code("+1 (###) ###-####"), "1");
        assert_eq!(PhoneFormat::calling_code("+44 #### ######"), "44");
        assert_eq!(PhoneFormat::calling_code("### ### ####"), "");
    }

    #[test]
    fn picks_the_tightest_pattern_the_number_fits() {
        // 10 digits fit the shorter layout; 11 need the longer one.
        assert_eq!(DE.pattern_for("1512345678"), "+49 ### #######");
        assert_eq!(DE.pattern_for("15112345678"), "+49 ### ########");
        // Partially typed numbers stay on the shorter layout.
        assert_eq!(DE.pattern_for("151"), "+49 ### #######");
        // Past every capacity, the largest pattern takes it.
        assert_eq!(DE.pattern_for("151123456789999"), "+49 ### ########");
    }

    #[test]
    fn picks_the_same_pattern_from_a_formatted_number() {
        for raw in ["1512345678", "15112345678"] {
            let display = format_phone_with(raw, DE);
            assert_eq!(DE.pattern_for(&display), DE.pattern_for(raw), "{display}");
        }
    }

    #[test]
    fn formats_each_length_with_its_own_pattern() {
        assert_eq!(format_phone_with("1512345678", DE), "+49 151 2345678");
        assert_eq!(format_phone_with("15112345678", DE), "+49 151 12345678");
        // The calling code is dropped whichever pattern wins.
        assert_eq!(format_phone_with("4915112345678", DE), "+49 151 12345678");
    }

    #[test]
    fn formats_against_a_single_pattern() {
        assert_eq!(format_phone_with("7911123456", GB), "+44 7911 123456");
        assert_eq!(format_phone_with("612345678", FR), "+33 6 12 34 56 78");
        assert_eq!(format_phone_with("5551234567", NO_CODE), "555 123 4567");
    }

    #[test]
    fn formats_partial_input() {
        assert_eq!(format_phone_with("7", GB), "+44 7");
        assert_eq!(format_phone_with("7911", GB), "+44 7911");
        assert_eq!(format_phone_with("79111", GB), "+44 7911 1");
        assert_eq!(format_phone_with("6123", FR), "+33 6 12 3");
        assert_eq!(format_phone_with("15", DE), "+49 15");
    }

    #[test]
    fn closes_a_bracket_the_digits_already_opened() {
        let bracketed = PhoneFormat::new("+61 (##) #### ####");
        assert_eq!(format_phone_with("2", bracketed), "+61 (2)");
        assert_eq!(format_phone_with("29", bracketed), "+61 (29)");
        assert_eq!(format_phone_with("291", bracketed), "+61 (29) 1");
    }

    #[test]
    fn strips_a_leading_calling_code_only_when_the_number_is_over_length() {
        // 12 digits: the leading "44" is the calling code.
        assert_eq!(format_phone_with("447911123456", GB), "+44 7911 123456");
        // 10 digits: a national number that merely starts with "44".
        assert_eq!(format_phone_with("4479111234", GB), "+44 4479 111234");
    }

    #[test]
    fn drops_digits_past_the_pattern_capacity() {
        assert_eq!(format_phone_with("79111234567890", GB), "+44 7911 123456");
    }

    #[test]
    fn parses_back_to_national_digits() {
        assert_eq!(parse_phone_with("+44 7911 123456", GB), "7911123456");
        assert_eq!(parse_phone_with("+33 6 12 34 56 78", FR), "612345678");
        assert_eq!(parse_phone_with("555 123 4567", NO_CODE), "5551234567");
        assert_eq!(parse_phone_with("+49 151 12345678", DE), "15112345678");
        assert_eq!(parse_phone_with("", GB), "");
    }

    #[test]
    fn format_parse_round_trips_every_prefix() {
        for format in [PhoneFormat::US, GB, FR, NO_CODE, DE] {
            let raw = "98765432109";
            let longest = format
                .patterns()
                .iter()
                .map(|p| PhoneFormat::national_len(p))
                .max()
                .unwrap();
            for len in 1..=longest {
                let raw = &raw[..len];
                let display = format_phone_with(raw, format);
                assert_eq!(parse_phone_with(&display, format), raw, "{display}");
            }
        }
    }
}
