//! SSR pins for the presentational calendar family: `MonthView` cell count,
//! overflow affordance and keyboard reachability; `TimeGrid` hour rows, lane
//! widths and the clamped hour range (D5, D8).

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "calendar")]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;

use components::{CalendarEvent, MonthView, TimeGrid, TimeGridEvent};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn date(y: i32, m: u8, d: u8) -> time::Date {
    time::Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
}

fn event(id: &str, d: time::Date) -> CalendarEvent {
    CalendarEvent {
        id: id.into(),
        date: d,
        label: id.into(),
        muted: false,
    }
}

fn month_view() -> Element {
    let year = use_signal(|| 2026);
    let month = use_signal(|| 3u32);
    let events = use_signal(|| {
        let d = date(2026, 3, 10);
        vec![
            event("a", d),
            event("b", d),
            event("c", d),
            event("d", d),
            event("e", d),
            event("solo", date(2026, 3, 11)),
        ]
    });
    rsx! {
        MonthView {
            year,
            month,
            events,
            on_day_click: move |_| {},
            on_event_click: move |_| {},
        }
    }
}

#[test]
fn month_view_renders_42_keyboard_reachable_cells() {
    let html = render(month_view);
    let cells = html.matches(r#"role="button" tabindex="0""#).count();
    assert_eq!(cells, 42, "{html}");
    assert!(html.contains(r#"aria-label="March 10, 2026""#), "{html}");
    // Five events on one day: three chips + "+2 more".
    assert!(html.contains("+2 more"), "{html}");
    assert!(
        !html.contains(">d<"),
        "fourth event is folded into the overflow: {html}"
    );
    assert!(html.contains("solo"), "{html}");
}

fn time_grid() -> Element {
    let day = use_signal(|| date(2026, 6, 27));
    let events = use_signal(|| {
        vec![
            TimeGridEvent {
                id: "one".into(),
                start_minute: 9 * 60,
                end_minute: 9 * 60 + 30,
                label: "One".into(),
                muted: false,
            },
            TimeGridEvent {
                id: "two".into(),
                start_minute: 9 * 60 + 15,
                end_minute: 10 * 60,
                label: "Two".into(),
                muted: true,
            },
            TimeGridEvent {
                id: "clipped".into(),
                start_minute: 20 * 60,
                end_minute: 21 * 60,
                label: "Clipped".into(),
                muted: false,
            },
        ]
    });
    rsx! {
        TimeGrid {
            day,
            events,
            start_hour: 8,
            end_hour: 18,
            on_slot_click: move |_| {},
            on_event_click: move |_| {},
        }
    }
}

#[test]
fn time_grid_renders_hour_rows_and_lane_widths() {
    let html = render(time_grid);
    let rows = html.matches(r#"role="button""#).count();
    assert_eq!(rows, 10, "end_hour - start_hour hour rows: {html}");
    assert!(html.contains(r#"aria-label="08:00""#), "{html}");
    assert!(html.contains(r#"aria-label="17:00""#), "{html}");
    assert!(!html.contains(r#"aria-label="18:00""#), "{html}");
    // Two overlapping events split the width in two.
    assert_eq!(
        html.matches("width: calc((100% - 3rem) / 2 - 2px)").count(),
        2,
        "{html}"
    );
    assert!(
        !html.contains("Clipped"),
        "outside the visible range: {html}"
    );
    assert!(html.contains("height: 480px"), "10h × 48px: {html}");
}

fn inverted_time_grid() -> Element {
    let day = use_signal(|| date(2026, 6, 27));
    let events = use_signal(Vec::<TimeGridEvent>::new);
    rsx! {
        TimeGrid {
            day,
            events,
            start_hour: 18,
            end_hour: 8,
            on_slot_click: move |_| {},
            on_event_click: move |_| {},
        }
    }
}

#[test]
fn time_grid_with_end_before_start_renders_empty() {
    let html = render(inverted_time_grid);
    assert!(html.contains("height: 0px"), "{html}");
    assert!(!html.contains(r#"role="button""#), "{html}");
}
