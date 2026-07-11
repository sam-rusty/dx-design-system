use time::Date;

const WEEKDAY_SHORT: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Human-readable short date: "Tue, Mar 10"
pub fn format_header_date(date: &Date) -> String {
    let wd = WEEKDAY_SHORT[date.weekday().number_from_monday() as usize - 1];
    let mo = MONTH_SHORT[date.month() as u8 as usize - 1];
    format!("{}, {} {}", wd, mo, date.day())
}

/// Parse a calendar date from `s`.
///
/// Accepts ISO `YYYY-MM-DD` or US-style `MM-DD-YYYY`. Disambiguation: if the first
/// field is greater than 12 it is the year (ISO); otherwise if the last field is
/// greater than 31 it is the year (MDY); otherwise ISO is assumed.
pub fn parse_date(s: &str) -> Option<Date> {
    let mut parts = s.splitn(3, &['/', '-'][..]);
    let a: i32 = parts.next()?.parse().ok()?;
    let b: i32 = parts.next()?.parse().ok()?;
    let c: i32 = parts.next()?.parse().ok()?;
    let (year, month, day) = if a > 12 {
        (a, u8::try_from(b).ok()?, u8::try_from(c).ok()?)
    } else if c > 31 {
        (c, u8::try_from(a).ok()?, u8::try_from(b).ok()?)
    } else {
        (a, u8::try_from(b).ok()?, u8::try_from(c).ok()?)
    };
    Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()
}

/// Format a `time::Date` as "MM-DD-YYYY".
pub fn format_date(date: &Date) -> String {
    format!(
        "{:02}-{:02}-{:04}",
        date.month() as u8,
        date.day(),
        date.year(),
    )
}

/// Parse `YYYY-MM-DD` or `MM-DD-YYYY` date + `HH:MM...` (space or `T`) into (Date, hour, minute).
pub fn parse_datetime(s: &str) -> Option<(Date, u32, u32)> {
    if let Some((date_part, time_part)) = s.split_once(' ').or_else(|| s.split_once('T')) {
        let date = parse_date(date_part)?;
        let parts: Vec<&str> = time_part.split(':').collect();
        let hour: u32 = parts.first().and_then(|h| h.parse().ok()).unwrap_or(0);
        let minute: u32 = parts.get(1).and_then(|m| m.parse().ok()).unwrap_or(0);
        if hour > 23 || minute > 59 {
            return None;
        }
        Some((date, hour, minute))
    } else {
        let date = parse_date(s)?;
        Some((date, 0, 0))
    }
}

/// Format a date + time as "MM-DD-YYYY HH:MM:00.0".
pub fn format_datetime(date: &Date, hour: u32, minute: u32) -> String {
    format!("{} {:02}:{:02}:00.0", format_date(date), hour, minute)
}

/// Parse a date range from JSON array format or slash-separated format.
pub fn parse_date_range(s: &str) -> Option<(Date, Date)> {
    if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 2 {
            let start = parse_date(parts[0].trim().trim_matches('"'))?;
            let end = parse_date(parts[1].trim().trim_matches('"'))?;
            return Some((start, end));
        }
    }
    let (start, end) = s.split_once('/')?;
    let start = parse_date(start.trim())?;
    let end = parse_date(end.trim())?;
    Some((start, end))
}

/// Format a date range as a JSON array string.
pub fn format_date_range(start: &Date, end: &Date) -> String {
    format!("[\"{}\",\"{}\"]", format_date(start), format_date(end))
}

#[cfg(test)]
mod tests {
    use time::{Date, Month};

    use super::*;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn parse_date_valid() {
        assert_eq!(parse_date("2026-03-10"), Some(date(2026, 3, 10)));
        assert_eq!(parse_date("2000-01-01"), Some(date(2000, 1, 1)));
        assert_eq!(parse_date("1999-12-31"), Some(date(1999, 12, 31)));
    }

    #[test]
    fn parse_date_invalid() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("not-a-date"), None);
        assert_eq!(parse_date("2026-13-01"), None);
        assert_eq!(parse_date("2026-02-30"), None);
        assert_eq!(parse_date("2026"), None);
        assert_eq!(parse_date("2026-03"), None);
    }

    #[test]
    fn format_date_roundtrip() {
        let d = date(2026, 3, 10);
        let s = format_date(&d);
        assert_eq!(s, "03-10-2026");
        assert_eq!(parse_date(&s), Some(d));
    }

    #[test]
    fn parse_date_mdy() {
        assert_eq!(parse_date("03-10-2026"), Some(date(2026, 3, 10)));
        assert_eq!(parse_date("01-05-2026"), Some(date(2026, 1, 5)));
    }

    #[test]
    fn format_date_zero_pads() {
        assert_eq!(format_date(&date(2026, 1, 5)), "01-05-2026");
    }

    #[test]
    fn parse_datetime_space_separator() {
        let result = parse_datetime("2026-03-10 14:30:00.0");
        assert_eq!(result, Some((date(2026, 3, 10), 14, 30)));
    }

    #[test]
    fn parse_datetime_t_separator() {
        let result = parse_datetime("2026-03-10T09:05:00");
        assert_eq!(result, Some((date(2026, 3, 10), 9, 5)));
    }

    #[test]
    fn parse_datetime_date_only() {
        let result = parse_datetime("2026-03-10");
        assert_eq!(result, Some((date(2026, 3, 10), 0, 0)));
    }

    #[test]
    fn parse_datetime_invalid_hour() {
        assert_eq!(parse_datetime("2026-03-10 24:00:00"), None);
    }

    #[test]
    fn parse_datetime_invalid_minute() {
        assert_eq!(parse_datetime("2026-03-10 12:60:00"), None);
    }

    #[test]
    fn parse_datetime_invalid_date() {
        assert_eq!(parse_datetime("bad 12:00:00"), None);
    }

    #[test]
    fn format_datetime_roundtrip() {
        let d = date(2026, 3, 10);
        let s = format_datetime(&d, 14, 30);
        assert_eq!(s, "03-10-2026 14:30:00.0");
        let (pd, h, m) = parse_datetime(&s).unwrap();
        assert_eq!(pd, d);
        assert_eq!(h, 14);
        assert_eq!(m, 30);
    }

    #[test]
    fn parse_date_range_json_format() {
        let result = parse_date_range("[\"2026-03-01\",\"2026-03-15\"]");
        assert_eq!(result, Some((date(2026, 3, 1), date(2026, 3, 15))));
    }

    #[test]
    fn parse_date_range_slash_format() {
        let result = parse_date_range("2026-03-01/2026-03-15");
        assert_eq!(result, Some((date(2026, 3, 1), date(2026, 3, 15))));
    }

    #[test]
    fn parse_date_range_invalid() {
        assert_eq!(parse_date_range(""), None);
        assert_eq!(parse_date_range("not-a-range"), None);
        assert_eq!(parse_date_range("[\"bad\",\"data\"]"), None);
    }

    #[test]
    fn format_date_range_roundtrip() {
        let s = date(2026, 3, 1);
        let e = date(2026, 3, 15);
        let formatted = format_date_range(&s, &e);
        assert_eq!(formatted, "[\"03-01-2026\",\"03-15-2026\"]");
        let (ps, pe) = parse_date_range(&formatted).unwrap();
        assert_eq!(ps, s);
        assert_eq!(pe, e);
    }

    #[test]
    fn format_header_date_output() {
        let d = date(2026, 3, 10);
        let header = format_header_date(&d);
        assert_eq!(header, "Tue, Mar 10");
    }

    #[test]
    fn format_header_date_new_year() {
        let d = date(2026, 1, 1);
        let header = format_header_date(&d);
        assert_eq!(header, "Thu, Jan 1");
    }

    #[test]
    fn format_header_date_end_of_year() {
        let d = date(2025, 12, 31);
        let header = format_header_date(&d);
        assert_eq!(header, "Wed, Dec 31");
    }

    #[test]
    fn format_header_date_leap_day() {
        let d = date(2024, 2, 29);
        let header = format_header_date(&d);
        assert_eq!(header, "Thu, Feb 29");
    }

    #[test]
    fn parse_date_leap_year_valid() {
        assert_eq!(parse_date("2024-02-29"), Some(date(2024, 2, 29)));
    }

    #[test]
    fn parse_date_leap_year_invalid() {
        assert_eq!(parse_date("2023-02-29"), None);
    }

    #[test]
    fn parse_date_no_leading_zeros() {
        assert_eq!(parse_date("2026-3-5"), Some(date(2026, 3, 5)));
    }

    #[test]
    fn parse_date_us_slash_format() {
        assert_eq!(parse_date("12/03/1995"), Some(date(1995, 12, 3)));
        assert_eq!(parse_date("01/15/2026"), Some(date(2026, 1, 15)));
        assert_eq!(parse_date("6/5/2000"), Some(date(2000, 6, 5)));
    }

    #[test]
    fn format_date_large_year() {
        let d = date(9999, 1, 1);
        assert_eq!(format_date(&d), "01-01-9999");
    }

    #[test]
    fn parse_datetime_hour_only() {
        let result = parse_datetime("2026-03-10 14");
        assert_eq!(result, Some((date(2026, 3, 10), 14, 0)));
    }

    #[test]
    fn parse_datetime_with_subseconds() {
        let result = parse_datetime("2026-03-10 14:30:00.123");
        assert_eq!(result, Some((date(2026, 3, 10), 14, 30)));
    }

    #[test]
    fn parse_datetime_midnight() {
        let result = parse_datetime("2026-03-10 00:00");
        assert_eq!(result, Some((date(2026, 3, 10), 0, 0)));
    }

    #[test]
    fn parse_datetime_end_of_day() {
        let result = parse_datetime("2026-03-10 23:59");
        assert_eq!(result, Some((date(2026, 3, 10), 23, 59)));
    }

    #[test]
    fn format_datetime_midnight() {
        let d = date(2026, 3, 10);
        assert_eq!(format_datetime(&d, 0, 0), "03-10-2026 00:00:00.0");
    }

    #[test]
    fn format_datetime_end_of_day() {
        let d = date(2026, 3, 10);
        assert_eq!(format_datetime(&d, 23, 59), "03-10-2026 23:59:00.0");
    }

    #[test]
    fn parse_date_range_with_spaces() {
        let result = parse_date_range("[\"2026-03-01\" , \"2026-03-15\"]");
        assert_eq!(result, Some((date(2026, 3, 1), date(2026, 3, 15))));
    }

    #[test]
    fn parse_date_range_slash_with_spaces() {
        let result = parse_date_range("2026-03-01 / 2026-03-15");
        assert_eq!(result, Some((date(2026, 3, 1), date(2026, 3, 15))));
    }

    #[test]
    fn parse_date_range_same_date() {
        let result = parse_date_range("2026-03-10/2026-03-10");
        assert_eq!(result, Some((date(2026, 3, 10), date(2026, 3, 10))));
    }

    #[test]
    fn parse_date_range_json_single_element() {
        assert_eq!(parse_date_range("[\"2026-03-01\"]"), None);
    }

    #[test]
    fn parse_date_range_json_three_elements() {
        assert_eq!(
            parse_date_range("[\"2026-03-01\",\"2026-03-15\",\"2026-03-20\"]"),
            None
        );
    }

    #[test]
    fn format_date_range_same_date() {
        let d = date(2026, 3, 10);
        assert_eq!(format_date_range(&d, &d), "[\"03-10-2026\",\"03-10-2026\"]");
    }

    #[test]
    fn parse_date_negative_month() {
        assert_eq!(parse_date("2026--1-01"), None);
    }

    #[test]
    fn parse_date_extra_dashes() {
        assert_eq!(parse_date("2026-03-10-extra"), None);
    }

    #[test]
    fn format_date_roundtrip_boundary_dates() {
        let dates = [
            date(2000, 1, 1),
            date(2000, 12, 31),
            date(1970, 1, 1),
            date(2099, 12, 31),
        ];
        for d in dates {
            let s = format_date(&d);
            assert_eq!(parse_date(&s), Some(d), "roundtrip failed for {s}");
        }
    }
}
