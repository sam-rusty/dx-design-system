mod panel;
mod shared;

mod date_range_picker;
mod date_time_picker;
mod single;
pub mod typed;

pub use date_range_picker::DateRangePicker;
pub use date_time_picker::{
    DateTimePicker, DateTimePickerBase, OffsetAt, commit_datetime_value, commit_datetime_value_in,
    hour_12_to_24, hour_24_to_12, parse_datetime_value, parse_datetime_value_in,
};
pub use single::{DatePicker, DatePickerBase};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(pub time::Date);

impl Date {
    pub fn new(date: time::Date) -> Self {
        Self(date)
    }

    pub fn inner(&self) -> &time::Date {
        &self.0
    }

    pub fn into_inner(self) -> time::Date {
        self.0
    }
}

/// Today, per [`crate::calendar::today`] (device-local under `web`, UTC
/// otherwise) — the same day the calendar highlights, so `min`/`max`
/// constraints built from it agree with the "today" ring.
impl Default for Date {
    fn default() -> Self {
        Self(crate::calendar::today())
    }
}

fn current_date() -> time::Date {
    crate::calendar::today()
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.0.year(),
            self.0.month() as u8,
            self.0.day()
        )
    }
}

/// Parse error for [`Date`] / [`DateTime`] `FromStr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDateError(String);

impl std::fmt::Display for ParseDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid date: {}", self.0)
    }
}

impl std::error::Error for ParseDateError {}

impl std::str::FromStr for Date {
    type Err = ParseDateError;

    /// Accepts `YYYY-MM-DD` or `MM-DD-YYYY` (see `ds_utils::format::parse_date`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ds_utils::format::parse_date(s)
            .map(Date)
            .ok_or_else(|| ParseDateError(s.to_string()))
    }
}

/// Deprecated: panics on invalid input. Prefer `s.parse::<Date>()`
/// ([`std::str::FromStr`]); kept for compatibility until the next major bump.
impl From<String> for Date {
    fn from(s: String) -> Self {
        s.parse().unwrap_or_else(|e| panic!("{e}"))
    }
}

impl serde::Serialize for Date {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> serde::Deserialize<'de> for Date {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        ds_utils::format::parse_date(&s)
            .map(Date)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid date: {s}")))
    }
}

impl From<time::Date> for Date {
    fn from(date: time::Date) -> Self {
        Self(date)
    }
}

impl From<Date> for time::Date {
    fn from(date: Date) -> Self {
        date.0
    }
}

impl std::ops::Deref for Date {
    type Target = time::Date;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(pub time::PrimitiveDateTime);

impl Serialize for DateTime {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

fn deserialize_datetime_wire(s: &str) -> Option<DateTime> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (date, hour, minute, second) = ds_utils::format::parse_datetime_seconds(s)?;
    let t = time::Time::from_hms(hour as u8, minute as u8, second as u8).ok()?;
    Some(DateTime::new(time::PrimitiveDateTime::new(date, t)))
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let s = String::deserialize(deserializer)?;
        if let Some(dt) = deserialize_datetime_wire(&s) {
            return Ok(dt);
        }
        serde_json::from_value::<time::PrimitiveDateTime>(serde_json::Value::String(s.clone()))
            .map(Self)
            .map_err(|e| D::Error::custom(format!("invalid datetime '{s}': {e}")))
    }
}

impl DateTime {
    pub fn new(dt: time::PrimitiveDateTime) -> Self {
        Self(dt)
    }

    pub fn inner(&self) -> &time::PrimitiveDateTime {
        &self.0
    }

    pub fn into_inner(self) -> time::PrimitiveDateTime {
        self.0
    }

    pub fn to_iso8601(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.0.year(),
            self.0.month() as u8,
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.second()
        )
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self(time::PrimitiveDateTime::new(
            current_date(),
            time::Time::MIDNIGHT,
        ))
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.0.year(),
            self.0.month() as u8,
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.second()
        )
    }
}

impl From<time::PrimitiveDateTime> for DateTime {
    fn from(dt: time::PrimitiveDateTime) -> Self {
        Self(dt)
    }
}

impl From<DateTime> for time::PrimitiveDateTime {
    fn from(dt: DateTime) -> Self {
        dt.0
    }
}

impl std::ops::Deref for DateTime {
    type Target = time::PrimitiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::str::FromStr for DateTime {
    type Err = ParseDateError;

    /// Accepts the picker wire shape `YYYY-MM-DD HH:MM[:SS]` (`T` separator
    /// and `MM-DD-YYYY` dates too); seconds are preserved.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        deserialize_datetime_wire(s).ok_or_else(|| ParseDateError(s.to_string()))
    }
}

/// Deprecated: panics on invalid input. Prefer `s.parse::<DateTime>()`
/// ([`std::str::FromStr`]); kept for compatibility until the next major bump.
impl From<String> for DateTime {
    fn from(s: String) -> Self {
        s.parse().unwrap_or_else(|e| panic!("{e}"))
    }
}

impl crate::field_name::FormSchema for DateTime {
    const FIELD_TYPE: crate::field_name::FieldType = crate::field_name::FieldType::String;

    fn json_schema() -> serde_json::Value {
        serde_json::Value::String(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u8, d: u8) -> time::Date {
        time::Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
    }

    /// D2: both defaults must agree with the calendar's notion of "today"
    /// (device-local under `web`); on native both are UTC so this pins the
    /// delegation rather than a divergence.
    #[test]
    fn date_default_matches_calendar_today() {
        assert_eq!(Date::default().0, crate::calendar::today());
        let dt = DateTime::default().0;
        assert_eq!(dt.date(), crate::calendar::today());
        assert_eq!(dt.time(), time::Time::MIDNIGHT);
    }

    #[test]
    fn date_deserializes_iso_and_mdy() {
        let d: Date = serde_json::from_str("\"2026-03-10\"").unwrap();
        assert_eq!(d.0, date(2026, 3, 10));
        let d: Date = serde_json::from_str("\"03-10-2026\"").unwrap();
        assert_eq!(d.0, date(2026, 3, 10));
        assert!(serde_json::from_str::<Date>("\"\"").is_err());
        assert!(serde_json::from_str::<Date>("\"03-10-26\"").is_err());
    }

    #[test]
    fn date_serializes_iso() {
        let d = Date(date(2026, 3, 5));
        assert_eq!(serde_json::to_string(&d).unwrap(), "\"2026-03-05\"");
        assert_eq!(d.to_string(), "2026-03-05");
    }

    #[test]
    fn date_from_str() {
        assert_eq!("2026-03-10".parse::<Date>().unwrap().0, date(2026, 3, 10));
        assert!("nope".parse::<Date>().is_err());
    }

    #[test]
    fn datetime_deserializes_wire_shapes() {
        let expect = time::PrimitiveDateTime::new(
            date(2026, 1, 1),
            time::Time::from_hms(10, 30, 0).unwrap(),
        );
        for s in [
            "\"2026-01-01 10:30:00\"",
            "\"2026-01-01T10:30:00\"",
            "\"2026-01-01 10:30\"",
            "\"01-01-2026 10:30:00.0\"",
        ] {
            let dt: DateTime = serde_json::from_str(s).unwrap();
            assert_eq!(dt.0, expect, "{s}");
        }
        assert!(serde_json::from_str::<DateTime>("\"\"").is_err());
        assert!(serde_json::from_str::<DateTime>("\"2026-01-01 zz\"").is_err());
    }

    #[test]
    fn datetime_keeps_seconds_through_serde() {
        // D12: seconds used to be dropped on deserialize.
        let dt: DateTime = serde_json::from_str("\"2026-01-01 10:30:45\"").unwrap();
        assert_eq!(dt.0.second(), 45);
        assert_eq!(dt.to_string(), "2026-01-01 10:30:45");
        assert_eq!(dt.to_iso8601(), "2026-01-01T10:30:45");
        let back: DateTime = serde_json::from_str(&serde_json::to_string(&dt).unwrap()).unwrap();
        assert_eq!(back, dt);
    }

    #[test]
    fn datetime_from_str() {
        let dt: DateTime = "2026-01-01 10:30:45".parse().unwrap();
        assert_eq!(dt.0.second(), 45);
        assert!("2026-01-01 zz".parse::<DateTime>().is_err());
    }
}
