use super::DateTime as WireDateTime;
use dioxus::prelude::*;
use ds_utils::format::merge;
use time::{Date, PrimitiveDateTime, Time};

use super::panel::{CalendarPanel, PickerFooter, SelectionState};
use super::shared::*;
use crate::calendar::{CalendarState, format_header_date, parse_datetime, today};
use crate::form::{FormFieldFrame, use_field_binding};
use crate::icon::{Icon, IconName};
use crate::{Field, InputBase, Modal, ModalSize};

const SPINNER_BTN: &str = "size-8 flex items-center justify-center rounded-full text-foreground hover:bg-accent transition-colors cursor-pointer select-none";

const SPINNER_VALUE: &str = "w-14 h-12 flex items-center justify-center text-2xl font-medium text-foreground bg-accent/50 rounded-lg select-none";

const SPINNER_COLON: &str = "text-2xl font-medium text-foreground select-none px-1";

const PERIOD_BTN_ACTIVE: &str = "px-3 py-1.5 text-sm font-medium rounded-full bg-primary text-primary-foreground cursor-pointer transition-colors";

const PERIOD_BTN_INACTIVE: &str = "px-3 py-1.5 text-sm font-medium rounded-full text-muted-foreground hover:bg-accent cursor-pointer transition-colors";

#[derive(Clone, Copy, PartialEq, Eq)]
enum PickerStep {
    Date,
    Time,
}

fn hour_24_to_12(h: u32) -> (u32, bool) {
    let is_pm = h >= 12;
    let h12 = match h {
        0 => 12,
        13..=23 => h - 12,
        _ => h,
    };
    (h12, is_pm)
}

fn hour_12_to_24(h12: u32, is_pm: bool) -> u32 {
    match (h12, is_pm) {
        (12, false) => 0,
        (12, true) => 12,
        (_, false) => h12,
        (_, true) => h12 + 12,
    }
}

fn format_wire_datetime(date: Date, hour: u32, minute: u32) -> String {
    let t = Time::from_hms(hour as u8, minute as u8, 0)
        .expect("hour and minute are constrained by the picker and parse_datetime");
    WireDateTime::from(PrimitiveDateTime::new(date, t)).to_string()
}

fn device_offset() -> time::UtcOffset {
    time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
}

/// RFC3339 UTC string → device-local (date, hour, minute) for display.
fn rfc3339_to_wall(s: &str, offset: time::UtcOffset) -> Option<(Date, u32, u32)> {
    let odt =
        time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()?;
    let local = odt.to_offset(offset);
    Some((local.date(), local.hour() as u32, local.minute() as u32))
}

/// Picked wall time → RFC3339 UTC string for the form store / wire.
fn wall_to_utc_rfc3339(date: Date, hour: u32, minute: u32, offset: time::UtcOffset) -> String {
    let t = Time::from_hms(hour as u8, minute as u8, 0).unwrap_or(Time::MIDNIGHT);
    let utc = PrimitiveDateTime::new(date, t)
        .assume_offset(offset)
        .to_offset(time::UtcOffset::UTC);
    utc.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Stored form value → picker (date, hour, minute): RFC3339 UTC shifted to
/// device-local wall time in `utc` mode, wall-time wire string otherwise.
fn parse_value(s: &str, utc: bool) -> Option<(Date, u32, u32)> {
    if utc {
        rfc3339_to_wall(s, device_offset())
    } else {
        parse_datetime(s)
    }
}

/// Picked wall time → stored form value (RFC3339 UTC in `utc` mode).
fn commit_value(date: Date, hour: u32, minute: u32, utc: bool) -> String {
    if utc {
        wall_to_utc_rfc3339(date, hour, minute, device_offset())
    } else {
        format_wire_datetime(date, hour, minute)
    }
}

fn format_display_datetime(date: &Date, hour: u32, minute: u32) -> String {
    let (h12, is_pm) = hour_24_to_12(hour);
    let period = if is_pm { "PM" } else { "AM" };
    format!(
        "{}, {:02}:{:02} {}",
        format_display_date(date),
        h12,
        minute,
        period
    )
}

#[component]
pub fn DateTimePickerBase(
    #[props(default)] class: String,
    /// Controlled wire value.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Fired with the new wire value when a selection is confirmed.
    #[props(default)]
    on_value_change: Callback<String>,
    /// Whether the picker is disabled.
    #[props(default)]
    disabled: ReadSignal<bool>,
    #[props(default)] min: Option<Signal<WireDateTime>>,
    #[props(default)] max: Option<Signal<WireDateTime>>,
    #[props(default)] is_open: Option<Signal<bool>>,
    /// Store RFC3339 UTC, display device-local wall time.
    #[props(default)]
    utc: bool,
) -> Element {
    let default_is_open = use_signal(|| false);
    let mut is_open_sig = is_open.unwrap_or(default_is_open);

    let mut input_mode = use_signal(|| false);
    let mut input_value = use_signal(String::new);
    let input_value_read: ReadSignal<Option<String>> = use_memo(move || Some(input_value())).into();
    let mut step = use_signal(|| PickerStep::Date);

    let current_value = use_memo(move || value().unwrap_or_default());

    let parsed = use_memo(move || parse_value(&current_value(), utc));

    let initial_date = (*parsed.peek()).map(|(d, _, _)| d).unwrap_or_else(today);
    let cal_state = CalendarState::new(initial_date);

    let mut staging_date: Signal<Option<Date>> = use_signal(|| None);
    let mut staging_hour: Signal<u32> = use_signal(|| 0);
    let mut staging_minute: Signal<u32> = use_signal(|| 0);

    let selection = SelectionState::Single(staging_date.into());

    let min_date = use_memo(move || min.map(|s| s().into_inner().date()));
    let max_date = use_memo(move || max.map(|s| s().into_inner().date()));

    let is_date_disabled = move |date: Date| -> bool {
        if let Some(mn) = min_date()
            && date < mn
        {
            return true;
        }
        if let Some(mx) = max_date()
            && date > mx
        {
            return true;
        }
        false
    };

    let display_value = use_memo(move || {
        let val = current_value();
        if val.is_empty() {
            String::new()
        } else {
            parse_value(&val, utc)
                .map(|(d, h, m)| format_display_datetime(&d, h, m))
                .unwrap_or(val)
        }
    });

    let trigger_class = merge(&[TRIGGER_CLASS, &class]);
    let is_open_val = is_open_sig();
    let current_step = step();

    let header_title = if current_step == PickerStep::Date {
        "Select date"
    } else {
        "Select time"
    };

    let header_date_display = match (*staging_date.read(), current_step) {
        (Some(d), PickerStep::Time) => {
            format_display_datetime(&d, staging_hour(), staging_minute())
        }
        (Some(d), _) => format_header_date(&d),
        _ => "Pick a date".to_string(),
    };

    rsx! {
        div { "data-name": "DateTimePicker", class: "relative w-full",
            button {
                r#type: "button",
                class: "{trigger_class}",
                disabled: disabled(),
                "data-state": if is_open_val { "open" } else { "closed" },
                onclick: move |ev| {
                    ev.stop_propagation();
                    if disabled() { return; }
                    if let Some((d, h, m)) = *parsed.peek() {
                        staging_date.set(Some(d));
                        staging_hour.set(h);
                        staging_minute.set(m);
                        cal_state.navigate_to(&d);
                    } else {
                        staging_date.set(None);
                        staging_hour.set(12);
                        staging_minute.set(0);
                        cal_state.navigate_to(&today());
                    }
                    step.set(PickerStep::Date);
                    input_mode.set(false);
                    is_open_sig.set(true);
                },
                span { class: "truncate", "{display_or_nbsp(display_value())}" }
                CalendarClockIcon {}
            }

            if is_open_val {
                Modal {
                    on_close: move || is_open_sig.set(false),
                    headerless: true,
                    unpadded: true,
                    size: ModalSize::Sm,
                    PickerHeader { title: header_title,
                        div { class: HEADER_DATE, "{header_date_display}" }
                        if current_step == PickerStep::Date {
                            EditToggleButton {
                                input_mode: input_mode,
                                on_click: move |_| {
                                    let entering = !*input_mode.peek();
                                    if entering {
                                        let val = match *staging_date.peek() {
                                            Some(d) => format_wire_datetime(
                                                d,
                                                *staging_hour.peek(),
                                                *staging_minute.peek(),
                                            ),
                                            None => String::new(),
                                        };
                                        input_value.set(val);
                                    } else if let Some((d, h, m)) = parse_datetime(&input_value.peek())
                                        && !is_date_disabled(d) {
                                            staging_date.set(Some(d));
                                            staging_hour.set(h);
                                            staging_minute.set(m);
                                            cal_state.navigate_to(&d);
                                        }
                                    input_mode.set(entering);
                                },
                            }
                        }
                    }
                    div { class: "p-4",
                        match current_step {
                            PickerStep::Date => {
                                if input_mode() {
                                    rsx! {
                                        div { class: "py-4",
                                            label { class: "block text-xs font-medium text-muted-foreground mb-2", "Date & time" }
                                            InputBase {
                                                placeholder: "YYYY-MM-DD HH:MM:SS".to_string(),
                                                value: input_value_read,
                                                on_value_change: move |v: String| input_value.set(v),
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        CalendarPanel {
                                            state: cal_state,
                                            selection: selection,
                                            on_day_click: move |date: Date| {
                                                staging_date.set(Some(date));
                                                step.set(PickerStep::Time);
                                            },
                                            min_date: min_date(),
                                            max_date: max_date(),
                                        }
                                    }
                                }
                            }
                            PickerStep::Time => {
                                rsx! {
                                    TimeSpinner {
                                        hour: staging_hour,
                                        minute: staging_minute,
                                    }
                                    PickerFooter {
                                        on_cancel: move |_| step.set(PickerStep::Date),
                                        on_confirm: move |_| {
                                            if let Some(date) = *staging_date.peek() {
                                                let h = *staging_hour.peek();
                                                let m = *staging_minute.peek();
                                                let formatted = commit_value(date, h, m, utc);
                                                on_value_change.call(formatted);
                                            }
                                            is_open_sig.set(false);
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TimeSpinner(hour: Signal<u32>, minute: Signal<u32>) -> Element {
    let mut hour_mut = hour;
    let mut minute_mut = minute;

    // Derive AM/PM from the hour so the toggle can never desync from it.
    let is_pm = use_memo(move || hour() >= 12);
    let display_hour = use_memo(move || hour_24_to_12(hour()).0);

    rsx! {
        div { class: "flex flex-col items-center gap-4 py-6",
            div { class: "flex items-center gap-2",
                div { class: "flex flex-col items-center gap-1",
                    button {
                        r#type: "button",
                        class: SPINNER_BTN,
                        onclick: move |_| {
                            let cur = display_hour();
                            let next = if cur >= 12 { 1 } else { cur + 1 };
                            hour_mut.set(hour_12_to_24(next, is_pm()));
                        },
                        Icon { name: IconName::ChevronUp, class: "size-5" }
                    }
                    div { class: SPINNER_VALUE,
                        {format!("{:02}", display_hour())}
                    }
                    button {
                        r#type: "button",
                        class: SPINNER_BTN,
                        onclick: move |_| {
                            let cur = display_hour();
                            let next = if cur <= 1 { 12 } else { cur - 1 };
                            hour_mut.set(hour_12_to_24(next, is_pm()));
                        },
                        Icon { name: IconName::ChevronDown, class: "size-5" }
                    }
                }

                span { class: SPINNER_COLON, ":" }

                div { class: "flex flex-col items-center gap-1",
                    button {
                        r#type: "button",
                        class: SPINNER_BTN,
                        onclick: move |_| {
                            let cur = minute();
                            minute_mut.set(if cur >= 59 { 0 } else { cur + 1 });
                        },
                        Icon { name: IconName::ChevronUp, class: "size-5" }
                    }
                    div { class: SPINNER_VALUE,
                        {format!("{:02}", minute())}
                    }
                    button {
                        r#type: "button",
                        class: SPINNER_BTN,
                        onclick: move |_| {
                            let cur = minute();
                            minute_mut.set(if cur == 0 { 59 } else { cur - 1 });
                        },
                        Icon { name: IconName::ChevronDown, class: "size-5" }
                    }
                }

                div { class: "flex flex-col gap-1 ml-3",
                    button {
                        r#type: "button",
                        class: if !is_pm() { PERIOD_BTN_ACTIVE } else { PERIOD_BTN_INACTIVE },
                        onclick: move |_| {
                            hour_mut.set(hour_12_to_24(display_hour(), false));
                        },
                        "AM"
                    }
                    button {
                        r#type: "button",
                        class: if is_pm() { PERIOD_BTN_ACTIVE } else { PERIOD_BTN_INACTIVE },
                        onclick: move |_| {
                            hour_mut.set(hour_12_to_24(display_hour(), true));
                        },
                        "PM"
                    }
                }
            }
        }
    }
}

/// Props for [`DateTimePicker`], the form-bound date-time picker.
#[derive(Props, Clone, PartialEq)]
pub struct DateTimePickerProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
    /// Earliest selectable value.
    #[props(default)]
    pub min: Option<WireDateTime>,
    /// Latest selectable value.
    #[props(default)]
    pub max: Option<WireDateTime>,
    /// Whether the picker is disabled (OR-ed with the form's disabled state).
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Store RFC3339 UTC, display device-local wall time — for form fields
    /// typed `OffsetDateTime`.
    #[props(default)]
    pub utc: bool,
}

/// Form-bound date-time picker with stacked label and inline error.
pub fn DateTimePicker(props: DateTimePickerProps) -> Element {
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            class: props.class,
            DateTimePickerControl {
                min: props.min,
                max: props.max,
                disabled: props.disabled,
                open,
                utc: props.utc,
            }
        }
    }
}

#[component]
fn DateTimePickerControl(
    #[props(default)] min: Option<WireDateTime>,
    #[props(default)] max: Option<WireDateTime>,
    #[props(default)] disabled: ReadSignal<bool>,
    open: Signal<bool>,
    #[props(default)] utc: bool,
) -> Element {
    let binding = use_field_binding();
    let form_disabled = binding.disabled;
    let is_disabled: ReadSignal<bool> = use_memo(move || disabled() || form_disabled()).into();

    let has_min = min.is_some();
    let has_max = max.is_some();
    let min_sig = use_signal(|| min.unwrap_or_default());
    let max_sig = use_signal(|| max.unwrap_or_default());
    let min_opt = has_min.then_some(min_sig);
    let max_opt = has_max.then_some(max_sig);

    rsx! {
        DateTimePickerBase {
            value: binding.controlled_value,
            on_value_change: binding.on_commit,
            disabled: is_disabled,
            is_open: open,
            min: min_opt,
            max: max_opt,
            utc,
        }
    }
}

#[cfg(test)]
mod tests {
    use time::Date;

    use super::*;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn utc_value_round_trips_through_wall_time() {
        let offset = time::UtcOffset::from_hms(5, 0, 0).unwrap();
        // Stored UTC 04:30 displays as 09:30 wall time at +05:00.
        let (d, h, m) = rfc3339_to_wall("2026-07-19T04:30:00Z", offset).unwrap();
        assert_eq!((d, h, m), (date(2026, 7, 19), 9, 30));
        // Committing that wall time re-produces the same UTC instant.
        assert_eq!(wall_to_utc_rfc3339(d, h, m, offset), "2026-07-19T04:30:00Z");
    }

    #[test]
    fn utc_commit_crosses_date_line() {
        let offset = time::UtcOffset::from_hms(5, 0, 0).unwrap();
        // Wall 02:00 on the 19th at +05:00 is 21:00 UTC on the 18th.
        assert_eq!(
            wall_to_utc_rfc3339(date(2026, 7, 19), 2, 0, offset),
            "2026-07-18T21:00:00Z"
        );
    }

    #[test]
    fn rfc3339_to_wall_rejects_picker_wall_strings() {
        let offset = time::UtcOffset::UTC;
        assert!(rfc3339_to_wall("2026-07-19 09:30:00", offset).is_none());
    }

    #[test]
    fn hour_24_to_12_midnight() {
        assert_eq!(hour_24_to_12(0), (12, false));
    }

    #[test]
    fn hour_24_to_12_1am() {
        assert_eq!(hour_24_to_12(1), (1, false));
    }

    #[test]
    fn hour_24_to_12_11am() {
        assert_eq!(hour_24_to_12(11), (11, false));
    }

    #[test]
    fn hour_24_to_12_noon() {
        assert_eq!(hour_24_to_12(12), (12, true));
    }

    #[test]
    fn hour_24_to_12_1pm() {
        assert_eq!(hour_24_to_12(13), (1, true));
    }

    #[test]
    fn hour_24_to_12_11pm() {
        assert_eq!(hour_24_to_12(23), (11, true));
    }

    #[test]
    fn hour_12_to_24_midnight() {
        assert_eq!(hour_12_to_24(12, false), 0);
    }

    #[test]
    fn hour_12_to_24_noon() {
        assert_eq!(hour_12_to_24(12, true), 12);
    }

    #[test]
    fn hour_12_to_24_1am() {
        assert_eq!(hour_12_to_24(1, false), 1);
    }

    #[test]
    fn hour_12_to_24_1pm() {
        assert_eq!(hour_12_to_24(1, true), 13);
    }

    #[test]
    fn hour_12_to_24_11am() {
        assert_eq!(hour_12_to_24(11, false), 11);
    }

    #[test]
    fn hour_12_to_24_11pm() {
        assert_eq!(hour_12_to_24(11, true), 23);
    }

    #[test]
    fn roundtrip_all_hours() {
        for h in 0..=23 {
            let (h12, is_pm) = hour_24_to_12(h);
            assert_eq!(
                hour_12_to_24(h12, is_pm),
                h,
                "roundtrip failed for hour {h}"
            );
        }
    }

    #[test]
    fn format_display_datetime_afternoon() {
        let d = date(2026, 3, 10);
        let result = format_display_datetime(&d, 14, 30);
        assert!(
            result.contains("02:30 PM"),
            "expected '02:30 PM' in '{result}'"
        );
        assert!(
            result.contains("03/10/2026"),
            "expected '03/10/2026' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_midnight() {
        let d = date(2026, 3, 10);
        let result = format_display_datetime(&d, 0, 0);
        assert!(
            result.contains("12:00 AM"),
            "expected '12:00 AM' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_noon() {
        let d = date(2026, 3, 10);
        let result = format_display_datetime(&d, 12, 0);
        assert!(
            result.contains("12:00 PM"),
            "expected '12:00 PM' in '{result}'"
        );
    }

    #[test]
    fn hour_24_to_12_all_am_hours() {
        for h in 1..12 {
            let (h12, is_pm) = hour_24_to_12(h);
            assert_eq!(h12, h, "12h value wrong for hour {h}");
            assert!(!is_pm, "should be AM for hour {h}");
        }
    }

    #[test]
    fn hour_24_to_12_all_pm_hours() {
        for h in 13..=23 {
            let (h12, is_pm) = hour_24_to_12(h);
            assert_eq!(h12, h - 12, "12h value wrong for hour {h}");
            assert!(is_pm, "should be PM for hour {h}");
        }
    }

    #[test]
    fn hour_12_to_24_all_am() {
        assert_eq!(hour_12_to_24(12, false), 0);
        for h in 1..=11 {
            assert_eq!(hour_12_to_24(h, false), h, "AM conversion failed for {h}");
        }
    }

    #[test]
    fn hour_12_to_24_all_pm() {
        assert_eq!(hour_12_to_24(12, true), 12);
        for h in 1..=11 {
            assert_eq!(
                hour_12_to_24(h, true),
                h + 12,
                "PM conversion failed for {h}"
            );
        }
    }

    #[test]
    fn format_display_datetime_1am() {
        let d = date(2026, 3, 10);
        let result = format_display_datetime(&d, 1, 5);
        assert!(
            result.contains("01:05 AM"),
            "expected '01:05 AM' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_11pm() {
        let d = date(2026, 3, 10);
        let result = format_display_datetime(&d, 23, 59);
        assert!(
            result.contains("11:59 PM"),
            "expected '11:59 PM' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_12pm() {
        let d = date(2026, 6, 15);
        let result = format_display_datetime(&d, 12, 30);
        assert!(
            result.contains("12:30 PM"),
            "expected '12:30 PM' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_12am() {
        let d = date(2026, 6, 15);
        let result = format_display_datetime(&d, 0, 0);
        assert!(
            result.contains("12:00 AM"),
            "expected '12:00 AM' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_contains_date() {
        let d = date(2026, 12, 25);
        let result = format_display_datetime(&d, 9, 0);
        assert!(
            result.contains("12/25/2026"),
            "expected '12/25/2026' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_minute_zero_padded() {
        let d = date(2026, 3, 10);
        let result = format_display_datetime(&d, 9, 5);
        assert!(result.contains("09:05"), "expected '09:05' in '{result}'");
    }

    #[test]
    fn format_display_datetime_all_boundary_hours() {
        let d = date(2026, 1, 1);
        for h in 0..=23 {
            let result = format_display_datetime(&d, h, 0);
            if h < 12 {
                assert!(
                    result.contains("AM"),
                    "hour {h} should show AM, got '{result}'"
                );
            } else {
                assert!(
                    result.contains("PM"),
                    "hour {h} should show PM, got '{result}'"
                );
            }
        }
    }

    #[test]
    fn format_display_datetime_weekday_included() {
        let d = date(2026, 1, 5);
        let result = format_display_datetime(&d, 10, 0);
        assert!(
            result.contains("01/05/2026"),
            "expected '01/05/2026' in '{result}'"
        );
    }

    #[test]
    fn format_display_datetime_hour_padded() {
        let d = date(2026, 3, 10);
        let r1 = format_display_datetime(&d, 1, 0);
        assert!(r1.contains("01:00"), "expected '01:00' in '{r1}'");
        let r2 = format_display_datetime(&d, 13, 0);
        assert!(r2.contains("01:00"), "expected '01:00' in '{r2}'");
    }
}
