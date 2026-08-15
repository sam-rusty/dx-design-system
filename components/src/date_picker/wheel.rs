//! iOS-style wheel pickers: the generic [`Wheel`] column (pointer-drag +
//! tap-to-select, no scroll plumbing) and the form-bound [`WheelDateTime`]
//! (date / hour / minute / AM-PM columns committing the same wire value as
//! `DateTimePicker`, including its `utc` mode).

use dioxus::prelude::*;
use ds_utils::format::merge;
use time::{Date, Duration, Month, OffsetDateTime, Weekday};

use super::date_time_picker::{
    commit_value, device_offset, hour_12_to_24, hour_24_to_12, parse_value,
};
use crate::field_name::Field;
use crate::form::{FormField, use_field_binding};

/// Row height in px; the column shows five rows with the selected one centered.
const ITEM_H: f64 = 36.0;
const VISIBLE: f64 = 5.0;
/// Pointer travel below this is a tap, not a drag.
const TAP_SLOP: f64 = 6.0;

/// Vertical fade so the column reads as a cylinder.
const WHEEL_MASK: &str = "mask-image: linear-gradient(to bottom, transparent, #000 28%, #000 72%, transparent); -webkit-mask-image: linear-gradient(to bottom, transparent, #000 28%, #000 72%, transparent);";

/// One picker column: drag anywhere on it to spin, tap a visible row to jump.
/// Controlled — the parent owns `index` and applies `on_change`.
#[component]
pub fn Wheel(
    values: Vec<String>,
    index: usize,
    on_change: EventHandler<usize>,
    #[props(default)] class: String,
) -> Element {
    let count = values.len();
    let mut drag_from = use_signal(|| None::<f64>);
    let mut drag_y = use_signal(|| 0.0f64);
    // Set on drag release so the trailing click doesn't also jump-select.
    let mut just_dragged = use_signal(|| false);

    let clamp = move |i: i64| i.clamp(0, count.saturating_sub(1) as i64) as usize;
    let dragging = drag_from().is_some();
    let live_index = if dragging {
        clamp(index as i64 - (drag_y() / ITEM_H).round() as i64)
    } else {
        index
    };

    let translate = (VISIBLE - 1.0) / 2.0 * ITEM_H - index as f64 * ITEM_H + drag_y();
    let column_style = if dragging {
        format!("transform: translateY({translate}px); transition: none;")
    } else {
        format!("transform: translateY({translate}px);")
    };

    let mut settle = move || {
        if drag_from.take().is_some() {
            let dy = drag_y.take();
            if dy.abs() > TAP_SLOP {
                just_dragged.set(true);
                let next = clamp(index as i64 - (dy / ITEM_H).round() as i64);
                if next != index {
                    on_change.call(next);
                }
            }
        }
    };

    let container_class = merge(&["relative select-none touch-none overflow-hidden", &class]);
    let container_style = format!("height: {}px; {WHEEL_MASK}", ITEM_H * VISIBLE);

    rsx! {
        div {
            class: container_class,
            style: container_style,
            onpointerdown: move |e| {
                drag_from.set(Some(e.client_coordinates().y));
                drag_y.set(0.0);
                just_dragged.set(false);
            },
            onpointermove: move |e| {
                if let Some(start) = drag_from() {
                    drag_y.set(e.client_coordinates().y - start);
                }
            },
            onpointerup: move |_| settle(),
            onpointerleave: move |_| settle(),
            onpointercancel: move |_| {
                drag_from.set(None);
                drag_y.set(0.0);
            },
            div {
                class: "absolute inset-x-0 top-0 transition-transform duration-300 ease-[cubic-bezier(.32,.72,0,1)]",
                style: column_style,
                for (i , v) in values.iter().enumerate() {
                    button {
                        key: "{i}",
                        r#type: "button",
                        tabindex: -1,
                        class: merge(
                            &[
                                "flex w-full items-center justify-center whitespace-nowrap px-2 tabular-nums transition-colors duration-150 cursor-pointer",
                                if i == live_index {
                                    "text-[19px] font-semibold text-foreground"
                                } else {
                                    "text-[17px] text-muted-foreground"
                                },
                            ],
                        ),
                        style: "height: {ITEM_H}px",
                        onclick: move |_| {
                            if just_dragged.take() {
                                return;
                            }
                            if i != index {
                                on_change.call(i);
                            }
                        },
                        "{v}"
                    }
                }
            }
        }
    }
}

/// Centered highlight band + column row shared by every wheel cluster.
#[component]
pub fn WheelDeck(#[props(default)] class: String, children: Element) -> Element {
    let deck_class = merge(&["relative flex items-stretch justify-center", &class]);
    rsx! {
        div { class: deck_class,
            div {
                class: "pointer-events-none absolute inset-x-1 top-1/2 -translate-y-1/2 rounded-[10px] bg-glass-hi",
                style: "height: {ITEM_H}px",
            }
            {children}
        }
    }
}

fn weekday_abbr(w: Weekday) -> &'static str {
    match w {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

fn month_abbr(m: Month) -> &'static str {
    match m {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    }
}

/// "Fri Sep 4", year appended only when it differs from the anchor's.
fn date_label(d: Date, anchor_year: i32) -> String {
    let base = format!(
        "{} {} {}",
        weekday_abbr(d.weekday()),
        month_abbr(d.month()),
        d.day()
    );
    if d.year() == anchor_year {
        base
    } else {
        format!("{base} {}", d.year())
    }
}

/// Days either side of the anchor date the date column offers.
const DATE_SPAN: i64 = 365;

/// Wall-time (date, hour, minute) fallback when the field is empty: now.
fn now_wall() -> (Date, u32, u32) {
    let local = OffsetDateTime::now_utc().to_offset(device_offset());
    (local.date(), local.hour() as u32, local.minute() as u32)
}

/// Props for [`WheelDateTime`], the form-bound wheel date-time picker.
#[derive(Props, Clone, PartialEq)]
pub struct WheelDateTimeProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Store RFC3339 UTC, display device-local wall time — for form fields
    /// typed `OffsetDateTime` (matches `DateTimePicker`'s `utc`).
    #[props(default)]
    pub utc: bool,
    /// Show the hour / minute / AM-PM columns. `false` = date column only;
    /// commits preserve the stored time of day.
    #[props(default = true)]
    pub time: bool,
    /// Extra classes merged onto the wheel deck.
    #[props(default)]
    pub class: String,
}

/// Form-bound iOS wheel picker over the same wire value as `DateTimePicker`:
/// no label/error chrome of its own — meant to sit inside an inline expander
/// row, not a stacked form.
pub fn WheelDateTime(props: WheelDateTimeProps) -> Element {
    rsx! {
        FormField { field: props.field,
            WheelDateTimeControl { utc: props.utc, time: props.time, class: props.class }
        }
    }
}

#[component]
fn WheelDateTimeControl(utc: bool, time: bool, #[props(default)] class: String) -> Element {
    let binding = use_field_binding();
    let value = binding.value;
    let current = use_memo(move || parse_value(&value(), utc).unwrap_or_else(now_wall));

    // The date column is a fixed window anchored on the first-seen date, so
    // indices stay stable while the admin spins.
    let anchor = use_hook(|| current.peek().0);
    let start = anchor.saturating_sub(Duration::days(DATE_SPAN));
    let date_labels = use_hook(|| {
        (0..=2 * DATE_SPAN)
            .map(|i| date_label(start.saturating_add(Duration::days(i)), anchor.year()))
            .collect::<Vec<_>>()
    });

    let (date, hour, minute) = current();
    let date_idx = (date - start).whole_days().clamp(0, 2 * DATE_SPAN) as usize;
    let (h12, is_pm) = hour_24_to_12(hour);

    let commit = binding.on_commit;
    let pick = move |f: &dyn Fn((Date, u32, u32)) -> (Date, u32, u32)| {
        let (d, h, m) = f(*current.peek());
        commit.call(commit_value(d, h, m, utc));
    };

    let on_date = move |i: usize| {
        pick(&move |(_, h, m)| (start.saturating_add(Duration::days(i as i64)), h, m));
    };
    let on_hour = move |i: usize| {
        pick(&move |(d, h, m)| {
            let (_, pm) = hour_24_to_12(h);
            (d, hour_12_to_24(i as u32 + 1, pm), m)
        });
    };
    let on_minute = move |i: usize| {
        pick(&move |(d, h, _)| (d, h, i as u32));
    };
    let on_meridiem = move |i: usize| {
        pick(&move |(d, h, m)| {
            let (h12, _) = hour_24_to_12(h);
            (d, hour_12_to_24(h12, i == 1), m)
        });
    };

    let hours = use_hook(|| (1..=12).map(|h| h.to_string()).collect::<Vec<_>>());
    let minutes = use_hook(|| (0..60).map(|m| format!("{m:02}")).collect::<Vec<_>>());

    rsx! {
        WheelDeck { class,
            Wheel {
                values: date_labels.clone(),
                index: date_idx,
                on_change: on_date,
                class: "min-w-[136px]",
            }
            if time {
                Wheel {
                    values: hours.clone(),
                    index: (h12 - 1) as usize,
                    on_change: on_hour,
                    class: "min-w-[44px]",
                }
                Wheel {
                    values: minutes.clone(),
                    index: minute as usize,
                    on_change: on_minute,
                    class: "min-w-[44px]",
                }
                Wheel {
                    values: vec!["AM".to_string(), "PM".to_string()],
                    index: is_pm as usize,
                    on_change: on_meridiem,
                    class: "min-w-[52px]",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    /// The date column's label only grows a year suffix once the window
    /// crosses out of the anchor year — the wheel would otherwise show two
    /// identical "Wed Dec 31" rows a year apart.
    #[test]
    fn date_label_disambiguates_other_years() {
        assert_eq!(date_label(date(2026, 9, 4), 2026), "Fri Sep 4");
        assert_eq!(date_label(date(2027, 1, 2), 2026), "Sat Jan 2 2027");
    }
}
