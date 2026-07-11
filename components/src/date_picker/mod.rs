mod panel;
mod shared;

mod date_range_picker;
mod date_time_picker;
mod single;

pub use date_range_picker::DateRangePicker;
pub use date_time_picker::{DateTimePicker, DateTimePickerBase};
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

impl Default for Date {
    fn default() -> Self {
        Self(time::OffsetDateTime::now_utc().date())
    }
}

fn current_date() -> time::Date {
    time::OffsetDateTime::now_utc().date()
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

impl From<String> for Date {
    fn from(s: String) -> Self {
        utils::format::parse_date(&s)
            .map(Date)
            .unwrap_or_else(|| panic!("invalid date: {s}"))
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
        utils::format::parse_date(&s)
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
    let (date, hour, minute) = utils::format::parse_datetime(s)?;
    let t = time::Time::from_hms(hour as u8, minute as u8, 0).ok()?;
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

impl From<String> for DateTime {
    fn from(s: String) -> Self {
        deserialize_datetime_wire(&s).unwrap_or_else(|| panic!("invalid datetime: {s}"))
    }
}
