use super::DateTime as WireDateTime;
use dioxus::prelude::*;
use time::{Date, PrimitiveDateTime, Time};
use utils::format::merge;

use super::panel::{CalendarPanel, PickerFooter, SelectionState};
use super::shared::*;
use crate::calendar::{CalendarState, format_header_date, parse_datetime, today};
use crate::form::{FormError, FormField};
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
    #[props(default)] value: Option<ReadSignal<String>>,
    #[props(default)] on_change: Option<EventHandler<String>>,
    #[props(default)] disabled: bool,
    #[props(default)] min: Option<Signal<WireDateTime>>,
    #[props(default)] max: Option<Signal<WireDateTime>>,
    #[props(default)] is_open: Option<Signal<bool>>,
) -> Element {
    let default_is_open = use_signal(|| false);
    let mut is_open_sig = is_open.unwrap_or(default_is_open);

    let mut input_mode = use_signal(|| false);
    let mut input_value = use_signal(String::new);
    let mut step = use_signal(|| PickerStep::Date);

    let current_value: ReadSignal<String> =
        value.unwrap_or_else(|| ReadSignal::from(use_signal(String::new)));

    let parsed = use_memo(move || parse_datetime(&current_value()));

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
            parse_datetime(&val)
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
                disabled: disabled,
                "data-state": if is_open_val { "open" } else { "closed" },
                onclick: move |ev| {
                    ev.stop_propagation();
                    if disabled { return; }
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
                                                value: input_value,
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
                                                let formatted = format_wire_datetime(date, h, m);
                                                if let Some(cb) = on_change {
                                                    cb.call(formatted);
                                                }
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

#[component]
pub fn DateTimePicker(
    field: Field,
    #[props(default)] class: String,
    #[props(default)] min: Option<WireDateTime>,
    #[props(default)] max: Option<WireDateTime>,
    #[props(default)] disabled: Option<Signal<bool>>,
) -> Element {
    let label = field.label.to_string();
    rsx! {
        FormField { field,
            DateTimePickerControl { label, class, min, max, disabled }
            FormError {}
        }
    }
}

#[component]
fn DateTimePickerControl(
    label: String,
    class: String,
    #[props(default)] min: Option<WireDateTime>,
    #[props(default)] max: Option<WireDateTime>,
    #[props(default)] disabled: Option<Signal<bool>>,
) -> Element {
    let is_open = use_signal(|| false);

    let (field_name, form_ctx) = use_form_field();
    let value_signal = form_value_signal(&field_name, form_ctx);
    let on_change = form_on_change(&field_name, form_ctx);
    let form_is_disabled = form_disabled(form_ctx);

    let is_disabled = disabled.map(|d| d()).unwrap_or(false) || form_is_disabled();

    let has_min = min.is_some();
    let has_max = max.is_some();
    let min_sig = use_signal(|| min.unwrap_or_default());
    let max_sig = use_signal(|| max.unwrap_or_default());
    let min_opt = has_min.then_some(min_sig);
    let max_opt = has_max.then_some(max_sig);

    rsx! {
        div { class: "{merge(&[\"relative w-full mt-2\", &class])}",
            DateTimePickerBase {
                value: value_signal,
                on_change: on_change,
                disabled: is_disabled,
                is_open: is_open,
                min: min_opt,
                max: max_opt,
            }
            FloatingLabel { label: label, is_open: is_open, data_name: "DateTimePickerLabel" }
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
