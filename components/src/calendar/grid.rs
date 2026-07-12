use dioxus::prelude::*;
use time::Date;

pub const WEEKDAY_LABELS: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];

pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub fn month_name(month: u32) -> &'static str {
    MONTH_NAMES
        .get(month.wrapping_sub(1) as usize)
        .unwrap_or(&"")
}

// Re-export formatting/parsing functions from utils
pub use ds_utils::format::{
    format_date, format_date_range, format_header_date, parse_date, parse_date_range,
    parse_datetime,
};

#[cfg(target_arch = "wasm32")]
pub fn today() -> Date {
    let js_date = js_sys::Date::new_0();
    let year = js_date.get_full_year() as i32;
    let month = (js_date.get_month() + 1) as u8;
    let day = js_date.get_date() as u8;
    Date::from_calendar_date(year, time::Month::try_from(month).unwrap(), day).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn today() -> Date {
    let now = time::OffsetDateTime::now_utc();
    now.date()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarDay {
    pub date: Date,
    pub in_current_month: bool,
}

pub fn calendar_grid(year: i32, month: u32) -> Vec<CalendarDay> {
    let first =
        Date::from_calendar_date(year, time::Month::try_from(month as u8).unwrap(), 1).unwrap();
    let start_offset = first.weekday().number_days_from_sunday() as i64;
    let grid_start = first - time::Duration::days(start_offset);

    (0..42)
        .map(|i| {
            let date = grid_start + time::Duration::days(i);
            CalendarDay {
                in_current_month: date.month() as u8 as u32 == month && date.year() == year,
                date,
            }
        })
        .collect()
}

pub fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

pub fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct CalendarState {
    pub view_year: Signal<i32>,
    pub view_month: Signal<u32>,
    pub show_year_picker: Signal<bool>,
}

impl CalendarState {
    pub fn new(initial: Date) -> Self {
        Self {
            view_year: use_signal(move || initial.year()),
            view_month: use_signal(move || initial.month() as u8 as u32),
            show_year_picker: use_signal(|| false),
        }
    }

    pub fn go_prev(mut self) {
        let (y, m) = prev_month(*self.view_year.peek(), *self.view_month.peek());
        self.view_year.set(y);
        self.view_month.set(m);
    }

    pub fn go_next(mut self) {
        let (y, m) = next_month(*self.view_year.peek(), *self.view_month.peek());
        self.view_year.set(y);
        self.view_month.set(m);
    }

    pub fn set_year(mut self, year: i32) {
        self.view_year.set(year);
        self.show_year_picker.set(false);
    }

    pub fn navigate_to(mut self, date: &Date) {
        self.view_year.set(date.year());
        self.view_month.set(date.month() as u8 as u32);
    }
}

pub fn year_range() -> std::ops::RangeInclusive<i32> {
    let current = today().year();
    (current - 100)..=(current + 100)
}

#[cfg(test)]
mod tests {
    use ds_utils::format::format_datetime;
    use time::{Date, Month, PrimitiveDateTime, Time};

    use super::*;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn format_datetime_parse_roundtrip() {
        let d = date(2026, 3, 10);
        let formatted = format_datetime(&d, 14, 30);
        let parsed = parse_datetime(&formatted).expect("parse_datetime failed");
        assert_eq!(parsed.0, d);
        assert_eq!(parsed.1, 14);
        assert_eq!(parsed.2, 30);
    }

    #[test]
    fn format_datetime_midnight_parse_roundtrip() {
        let d = date(2025, 1, 1);
        let formatted = format_datetime(&d, 0, 0);
        let parsed = parse_datetime(&formatted).expect("parse_datetime midnight failed");
        assert_eq!(parsed.0, d);
        assert_eq!(parsed.1, 0);
        assert_eq!(parsed.2, 0);
    }

    #[test]
    fn format_date_parse_roundtrip() {
        let d = date(2026, 3, 10);
        let formatted = format_date(&d);
        let parsed = parse_date(&formatted).expect("parse_date failed");
        assert_eq!(parsed, d);
    }

    #[test]
    fn date_serde_serialize_then_deserialize() {
        let d = date(2026, 12, 25);
        let json = serde_json::to_string(&d).unwrap();
        let parsed: Date = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, d);
    }

    #[test]
    fn primitive_datetime_serde_serialize_then_deserialize() {
        let d = date(2026, 7, 4);
        let t = Time::from_hms(9, 15, 0).unwrap();
        let dt = PrimitiveDateTime::new(d, t);
        let json = serde_json::to_string(&dt).unwrap();
        let parsed: PrimitiveDateTime = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, dt);
    }

    #[test]
    fn format_date_range_parse_roundtrip() {
        let s = date(2026, 3, 1);
        let e = date(2026, 3, 15);
        let formatted = format_date_range(&s, &e);
        let parsed = parse_date_range(&formatted).expect("parse_date_range failed");
        assert_eq!(parsed, (s, e));
    }

    #[test]
    fn month_name_valid() {
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(6), "June");
        assert_eq!(month_name(12), "December");
    }

    #[test]
    fn month_name_out_of_range() {
        assert_eq!(month_name(0), "");
        assert_eq!(month_name(13), "");
    }

    #[test]
    fn calendar_grid_always_42_cells() {
        let grid = calendar_grid(2026, 2);
        assert_eq!(grid.len(), 42);
    }

    #[test]
    fn calendar_grid_first_day_is_sunday_or_before() {
        let grid = calendar_grid(2026, 3);
        assert_eq!(grid[0].date.weekday(), time::Weekday::Sunday);
    }

    #[test]
    fn calendar_grid_in_current_month_flags() {
        let grid = calendar_grid(2026, 3);
        let march_days: Vec<_> = grid.iter().filter(|d| d.in_current_month).collect();
        assert_eq!(march_days.len(), 31);
    }

    #[test]
    fn prev_month_wraps_year() {
        assert_eq!(prev_month(2026, 1), (2025, 12));
        assert_eq!(prev_month(2026, 6), (2026, 5));
    }

    #[test]
    fn next_month_wraps_year() {
        assert_eq!(next_month(2025, 12), (2026, 1));
        assert_eq!(next_month(2026, 6), (2026, 7));
    }

    #[test]
    fn calendar_grid_february_non_leap() {
        let grid = calendar_grid(2023, 2);
        assert_eq!(grid.len(), 42);
        let feb_days: Vec<_> = grid.iter().filter(|d| d.in_current_month).collect();
        assert_eq!(feb_days.len(), 28);
    }

    #[test]
    fn calendar_grid_february_leap() {
        let grid = calendar_grid(2024, 2);
        assert_eq!(grid.len(), 42);
        let feb_days: Vec<_> = grid.iter().filter(|d| d.in_current_month).collect();
        assert_eq!(feb_days.len(), 29);
    }

    #[test]
    fn calendar_grid_april_30_days() {
        let grid = calendar_grid(2026, 4);
        let april_days: Vec<_> = grid.iter().filter(|d| d.in_current_month).collect();
        assert_eq!(april_days.len(), 30);
    }

    #[test]
    fn calendar_grid_january_31_days() {
        let grid = calendar_grid(2026, 1);
        let jan_days: Vec<_> = grid.iter().filter(|d| d.in_current_month).collect();
        assert_eq!(jan_days.len(), 31);
    }

    #[test]
    fn calendar_grid_current_month_days_are_contiguous() {
        let grid = calendar_grid(2026, 3);
        let first_in_month = grid.iter().position(|d| d.in_current_month).unwrap();
        let last_in_month = grid.iter().rposition(|d| d.in_current_month).unwrap();

        assert!(
            grid[first_in_month..=last_in_month]
                .iter()
                .all(|d| d.in_current_month),
            "gap found in current month days"
        );
    }

    #[test]
    fn calendar_grid_first_in_month_day_is_1() {
        let grid = calendar_grid(2026, 5);
        let first = grid.iter().find(|d| d.in_current_month).unwrap();
        assert_eq!(first.date.day(), 1);
    }

    #[test]
    fn calendar_grid_last_in_month_day_matches() {
        let grid = calendar_grid(2026, 5);
        let last = grid.iter().rev().find(|d| d.in_current_month).unwrap();
        assert_eq!(last.date.day(), 31);
    }

    #[test]
    fn calendar_grid_dates_are_sequential() {
        let grid = calendar_grid(2026, 6);
        for i in 1..grid.len() {
            let prev = grid[i - 1].date;
            let curr = grid[i].date;
            assert_eq!(
                curr,
                prev + time::Duration::days(1),
                "dates not sequential at index {}",
                i
            );
        }
    }

    #[test]
    fn prev_month_normal() {
        assert_eq!(prev_month(2026, 7), (2026, 6));
        assert_eq!(prev_month(2026, 2), (2026, 1));
    }

    #[test]
    fn next_month_normal() {
        assert_eq!(next_month(2026, 1), (2026, 2));
        assert_eq!(next_month(2026, 11), (2026, 12));
    }

    #[test]
    fn month_name_all_months() {
        let expected = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        for (i, name) in expected.iter().enumerate() {
            assert_eq!(month_name((i + 1) as u32), *name);
        }
    }

    #[test]
    fn year_range_contains_current_year() {
        let range = year_range();
        let current = today().year();
        assert!(range.contains(&current));
    }

    #[test]
    fn year_range_span_is_201() {
        let range = year_range();
        let count = range.clone().count();
        assert_eq!(count, 201);
    }
}
