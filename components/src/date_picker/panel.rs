#![allow(clippy::suspicious_else_formatting)]
use dioxus::prelude::*;
use time::Date;

use crate::calendar::{
    CalendarDay, CalendarState, WEEKDAY_LABELS, calendar_grid, month_name, today, year_range,
};
use crate::icon::{Icon, IconName};
use crate::layout::{FlexGap, FlexGridCols, Grid};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DayState {
    Normal,
    Today,
    Selected,
    RangeStart,
    RangeEnd,
    InRange,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SelectionState {
    Single(ReadSignal<Option<Date>>),
    Range {
        start: Signal<Option<Date>>,
        end: Signal<Option<Date>>,
    },
}

impl SelectionState {
    fn day_state(&self, date: Date, today_date: Date) -> DayState {
        match self {
            SelectionState::Single(selected) => {
                if selected.read().is_some_and(|s| s == date) {
                    DayState::Selected
                } else if date == today_date {
                    DayState::Today
                } else {
                    DayState::Normal
                }
            }
            SelectionState::Range { start, end } => {
                let s = *start.read();
                let e = *end.read();
                match (s, e) {
                    (Some(sv), Some(ev)) if sv == ev && date == sv => DayState::Selected,
                    (Some(sv), Some(ev)) => {
                        if date == sv {
                            DayState::RangeStart
                        } else if date == ev {
                            DayState::RangeEnd
                        } else if date > sv && date < ev {
                            DayState::InRange
                        } else if date == today_date {
                            DayState::Today
                        } else {
                            DayState::Normal
                        }
                    }
                    (Some(sv), None) if date == sv => DayState::Selected,
                    _ => {
                        if date == today_date {
                            DayState::Today
                        } else {
                            DayState::Normal
                        }
                    }
                }
            }
        }
    }
}

const NAV_BTN: &str = "size-9 flex items-center justify-center rounded-full text-foreground hover:bg-secondary transition-colors cursor-pointer";
const YEAR_BTN: &str = "flex items-center gap-1 text-sm font-medium text-foreground hover:bg-secondary rounded-full px-3 py-1.5 transition-colors cursor-pointer select-none";

#[component]
fn ChevronUpIcon() -> Element {
    rsx! { Icon { name: IconName::ChevronUp, class: "size-5" } }
}

#[component]
fn ChevronDownIcon() -> Element {
    rsx! { Icon { name: IconName::ChevronDown, class: "size-5" } }
}

#[component]
fn ChevronLeftIcon() -> Element {
    rsx! { Icon { name: IconName::ChevronLeft, class: "size-4" } }
}

#[component]
fn ChevronRightIcon() -> Element {
    rsx! { Icon { name: IconName::ChevronRight, class: "size-4" } }
}

#[allow(clippy::suspicious_else_formatting)]
#[component]
pub fn CalendarPanel(
    state: CalendarState,
    selection: SelectionState,
    on_day_click: EventHandler<Date>,
    #[props(default)] min_date: Option<Date>,
    #[props(default)] max_date: Option<Date>,
) -> Element {
    let year = state.view_year;
    let month = state.view_month;
    let show_year_picker = state.show_year_picker;

    let current_days = calendar_grid(year(), month());

    rsx! {
        div { class: "flex flex-col select-none",
            MonthHeader { state: state }
            if show_year_picker() {
                YearGrid { state: state }
            } else {
                DayGrid {
                    days: current_days,
                    selection: selection,
                    on_day_click: on_day_click,
                    min_date: min_date,
                    max_date: max_date,
                }
            }
        }
    }
}

#[component]
fn MonthHeader(state: CalendarState) -> Element {
    let year = state.view_year;
    let month = state.view_month;
    let mut show_year_picker = state.show_year_picker;

    rsx! {
        div { class: "flex items-center justify-between px-2 mb-2",
            button {
                r#type: "button",
                class: YEAR_BTN,
                onclick: move |_| {
                    let cur = show_year_picker();
                    show_year_picker.set(!cur);
                },
                "{month_name(month())} {year()}"
                span {
                    class: if show_year_picker() {
                        "size-4 transition-transform duration-200 rotate-180 inline-flex"
                    } else {
                        "size-4 transition-transform duration-200 inline-flex"
                    },
                    ChevronDownIcon {}
                }
            }
            div { class: "flex items-center gap-1",
                button {
                    r#type: "button",
                    class: NAV_BTN,
                    onclick: move |_| state.go_prev(),
                    ChevronLeftIcon {}
                }
                button {
                    r#type: "button",
                    class: NAV_BTN,
                    onclick: move |_| state.go_next(),
                    ChevronRightIcon {}
                }
            }
        }
    }
}

/// Move focus to the previous/next/up/down/first/last enabled day button inside
/// `grid` in response to an arrow / Home / End key (WAI-ARIA grid navigation).
#[cfg(target_arch = "wasm32")]
fn grid_focus_move(grid: &web_sys::Element, key: &Key) {
    use wasm_bindgen::{JsCast, JsValue};

    let Ok(list) = grid.query_selector_all("button") else {
        return;
    };
    let items: Vec<web_sys::HtmlElement> = (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .collect();
    let n = items.len();
    if n == 0 {
        return;
    }
    let enabled = |i: usize| !items[i].has_attribute("disabled");
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());
    let current = active.as_ref().and_then(|a| {
        let av: &JsValue = a.as_ref();
        items.iter().position(|it| {
            let iv: &JsValue = it.as_ref();
            iv == av
        })
    });
    let step = |from: i32, delta: i32| -> Option<usize> {
        let mut i = from + delta;
        while i >= 0 && (i as usize) < n {
            if enabled(i as usize) {
                return Some(i as usize);
            }
            i += delta;
        }
        None
    };
    let first = (0..n).find(|&i| enabled(i));
    let last = (0..n).rev().find(|&i| enabled(i));
    let from = current.map(|c| c as i32).unwrap_or(-1);
    let target = match key {
        Key::ArrowRight => step(from, 1).or(first),
        Key::ArrowLeft => step(from, -1).or(last),
        Key::ArrowDown => step(from, 7).or(first),
        Key::ArrowUp => step(from, -7).or(last),
        Key::Home => first,
        Key::End => last,
        _ => None,
    };
    if let Some(t) = target {
        let _ = items[t].focus();
    }
}

#[component]
fn DayGrid(
    days: Vec<CalendarDay>,
    selection: SelectionState,
    on_day_click: EventHandler<Date>,
    #[props(default)] min_date: Option<Date>,
    #[props(default)] max_date: Option<Date>,
) -> Element {
    let today_date = today();
    let mut grid_el = use_signal(|| None::<web_sys::Element>);

    let on_nav = move |e: KeyboardEvent| match e.key() {
        Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => {
            e.prevent_default();
            #[cfg(target_arch = "wasm32")]
            if let Some(grid) = grid_el.peek().clone() {
                grid_focus_move(&grid, &e.key());
            }
        }
        _ => {}
    };

    rsx! {
        div {
            class: "px-1",
            role: "grid",
            "aria-label": "Calendar",
            onkeydown: on_nav,
            onmounted: move |e| {
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    grid_el.set(Some(el.clone()));
                }
            },
            Grid { cols: FlexGridCols::C7, gap: FlexGap::None, class: "mb-1",
                for label in WEEKDAY_LABELS.iter() {
                    div {
                        role: "columnheader",
                        class: "flex items-center justify-center h-9 text-xs font-medium text-muted-foreground",
                        "{label}"
                    }
                }
            }
            Grid { cols: FlexGridCols::C7, gap: FlexGap::None,
                for week in days.chunks(7) {
                    div { role: "row", class: "contents",
                        for day in week.iter() {
                            {
                                let date = day.date;
                                let in_month = day.in_current_month;
                                let ds = selection.day_state(date, today_date);
                                let is_disabled = !in_month
                                    || min_date.is_some_and(|mn| date < mn)
                                    || max_date.is_some_and(|mx| date > mx);
                                let is_selected = matches!(
                                    ds,
                                    DayState::Selected | DayState::RangeStart | DayState::RangeEnd
                                );
                                let bg_class = range_bg_class(ds, in_month);
                                let cell_class = day_cell_class(ds, in_month, is_disabled);
                                let aria_label = format!(
                                    "{} {}, {}",
                                    month_name(date.month() as u8 as u32),
                                    date.day(),
                                    date.year(),
                                );
                                rsx! {
                                    div { role: "gridcell", "aria-selected": is_selected, class: "{bg_class}",
                                        button {
                                            r#type: "button",
                                            class: "{cell_class}",
                                            disabled: is_disabled,
                                            "aria-label": aria_label,
                                            tabindex: if is_disabled { "-1" } else { "0" },
                                            onclick: move |_| on_day_click.call(date),
                                            "{date.day()}"
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
}

fn day_cell_class(ds: DayState, in_month: bool, disabled: bool) -> &'static str {
    if !in_month {
        return "size-9 text-sm rounded-full invisible";
    }

    if disabled {
        return "size-9 text-sm rounded-full flex items-center justify-center text-muted-foreground/40 cursor-not-allowed";
    }

    match ds {
        DayState::Selected => {
            "size-9 text-sm rounded-full flex items-center justify-center bg-primary text-primary-foreground font-medium cursor-pointer"
        }
        DayState::RangeStart | DayState::RangeEnd => {
            "size-9 text-sm rounded-full flex items-center justify-center bg-primary text-primary-foreground font-medium cursor-pointer relative z-10"
        }
        DayState::InRange => {
            "size-9 text-sm flex items-center justify-center text-foreground hover:bg-secondary cursor-pointer"
        }
        DayState::Today => {
            "size-9 text-sm rounded-full flex items-center justify-center border border-primary text-primary font-medium hover:bg-secondary cursor-pointer"
        }
        DayState::Normal => {
            "size-9 text-sm rounded-full flex items-center justify-center text-foreground hover:bg-secondary cursor-pointer"
        }
    }
}

fn range_bg_class(ds: DayState, in_month: bool) -> &'static str {
    if !in_month {
        return "flex items-center justify-center";
    }

    match ds {
        DayState::InRange => "flex items-center justify-center bg-secondary",
        DayState::RangeStart => {
            "flex items-center justify-center bg-gradient-to-r from-transparent from-50% to-secondary to-50%"
        }
        DayState::RangeEnd => {
            "flex items-center justify-center bg-gradient-to-l from-transparent from-50% to-secondary to-50%"
        }
        _ => "flex items-center justify-center",
    }
}

#[component]
fn YearGrid(state: CalendarState) -> Element {
    let current_year = *state.view_year.peek();
    let years = year_range();

    rsx! {
        div { class: "h-[252px] overflow-y-auto px-1",
            Grid { cols: FlexGridCols::C3, gap: FlexGap::Xs,
                for y in years.into_iter() {
                    {
                        let is_selected = y == current_year;
                        let cls = if is_selected {
                            "py-2 text-sm rounded-full font-medium bg-primary text-primary-foreground cursor-pointer"
                        } else {
                            "py-2 text-sm rounded-full font-medium text-foreground hover:bg-secondary cursor-pointer"
                        };
                        rsx! {
                            button {
                                r#type: "button",
                                class: "{cls}",
                                onclick: move |_| state.set_year(y),
                                "{y}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PickerFooter(on_cancel: EventHandler<()>, on_confirm: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex items-center justify-end gap-2 px-2 pt-2",
            button {
                r#type: "button",
                class: "px-4 py-2 text-sm font-medium text-foreground hover:bg-secondary rounded-full transition-colors cursor-pointer",
                onclick: move |_| on_cancel.call(()),
                "Cancel"
            }
            button {
                r#type: "button",
                class: "px-4 py-2 text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 rounded-full transition-colors cursor-pointer",
                onclick: move |_| on_confirm.call(()),
                "OK"
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn day_cell_class_not_in_month_returns_invisible() {
        let result = day_cell_class(DayState::Normal, false, false);
        assert!(result.contains("invisible"));
    }

    #[test]
    fn day_cell_class_disabled_returns_cursor_not_allowed() {
        let result = day_cell_class(DayState::Normal, true, true);
        assert!(result.contains("cursor-not-allowed"));
    }

    #[test]
    fn day_cell_class_selected_returns_primary_bg() {
        let result = day_cell_class(DayState::Selected, true, false);
        assert!(result.contains("bg-primary"));
    }

    #[test]
    fn day_cell_class_range_start_returns_primary_bg_with_z10() {
        let result = day_cell_class(DayState::RangeStart, true, false);
        assert!(result.contains("bg-primary"));
        assert!(result.contains("z-10"));
    }

    #[test]
    fn day_cell_class_range_end_returns_primary_bg_with_z10() {
        let result = day_cell_class(DayState::RangeEnd, true, false);
        assert!(result.contains("bg-primary"));
        assert!(result.contains("z-10"));
    }

    #[test]
    fn day_cell_class_in_range_does_not_contain_rounded_full() {
        let result = day_cell_class(DayState::InRange, true, false);
        assert!(!result.contains("rounded-full"));
    }

    #[test]
    fn day_cell_class_today_contains_border_primary() {
        let result = day_cell_class(DayState::Today, true, false);
        assert!(result.contains("border-primary"));
    }

    #[test]
    fn day_cell_class_normal_contains_hover_bg_secondary() {
        let result = day_cell_class(DayState::Normal, true, false);
        assert!(result.contains("hover:bg-secondary"));
    }

    #[test]
    fn range_bg_class_not_in_month_returns_base_without_bg() {
        let result = range_bg_class(DayState::Normal, false);
        assert!(result.contains("flex"));
        assert!(!result.contains("bg-secondary"));
        assert!(!result.contains("bg-gradient"));
    }

    #[test]
    fn range_bg_class_in_range_contains_bg_secondary() {
        let result = range_bg_class(DayState::InRange, true);
        assert!(result.contains("bg-secondary"));
    }

    #[test]
    fn range_bg_class_range_start_contains_gradient_to_r() {
        let result = range_bg_class(DayState::RangeStart, true);
        assert!(result.contains("bg-gradient-to-r"));
    }

    #[test]
    fn range_bg_class_range_end_contains_gradient_to_l() {
        let result = range_bg_class(DayState::RangeEnd, true);
        assert!(result.contains("bg-gradient-to-l"));
    }

    #[test]
    fn range_bg_class_normal_does_not_contain_bg_secondary() {
        let result = range_bg_class(DayState::Normal, true);
        assert!(!result.contains("bg-secondary"));
    }

    #[test]
    fn range_bg_class_selected_does_not_contain_bg_gradient() {
        let result = range_bg_class(DayState::Selected, true);
        assert!(!result.contains("bg-gradient"));
    }

    #[test]
    fn day_cell_class_not_in_month_ignores_disabled() {
        let result = day_cell_class(DayState::Normal, false, true);
        assert!(result.contains("invisible"));
        assert!(!result.contains("cursor-not-allowed"));
    }

    #[test]
    fn day_cell_class_not_in_month_ignores_state() {
        for state in [
            DayState::Selected,
            DayState::RangeStart,
            DayState::RangeEnd,
            DayState::InRange,
            DayState::Today,
        ] {
            let result = day_cell_class(state, false, false);
            assert!(
                result.contains("invisible"),
                "expected invisible for {:?} out of month",
                state
            );
        }
    }

    #[test]
    fn day_cell_class_disabled_overrides_all_states() {
        for state in [
            DayState::Selected,
            DayState::RangeStart,
            DayState::RangeEnd,
            DayState::InRange,
            DayState::Today,
            DayState::Normal,
        ] {
            let result = day_cell_class(state, true, true);
            assert!(
                result.contains("cursor-not-allowed"),
                "expected cursor-not-allowed for {:?} when disabled",
                state
            );
        }
    }

    #[test]
    fn day_cell_class_selected_has_primary_foreground() {
        let result = day_cell_class(DayState::Selected, true, false);
        assert!(result.contains("text-primary-foreground"));
    }

    #[test]
    fn day_cell_class_range_start_has_primary_foreground() {
        let result = day_cell_class(DayState::RangeStart, true, false);
        assert!(result.contains("text-primary-foreground"));
    }

    #[test]
    fn day_cell_class_range_end_has_primary_foreground() {
        let result = day_cell_class(DayState::RangeEnd, true, false);
        assert!(result.contains("text-primary-foreground"));
    }

    #[test]
    fn day_cell_class_in_range_has_cursor_pointer() {
        let result = day_cell_class(DayState::InRange, true, false);
        assert!(result.contains("cursor-pointer"));
    }

    #[test]
    fn day_cell_class_today_has_font_medium() {
        let result = day_cell_class(DayState::Today, true, false);
        assert!(result.contains("font-medium"));
    }

    #[test]
    fn day_cell_class_normal_has_cursor_pointer() {
        let result = day_cell_class(DayState::Normal, true, false);
        assert!(result.contains("cursor-pointer"));
    }

    #[test]
    fn day_cell_class_selected_not_in_month_is_invisible_not_primary() {
        let result = day_cell_class(DayState::Selected, false, false);
        assert!(result.contains("invisible"));
        assert!(!result.contains("bg-primary"));
    }

    #[test]
    fn range_bg_class_not_in_month_for_all_states() {
        for state in [
            DayState::Normal,
            DayState::Selected,
            DayState::RangeStart,
            DayState::RangeEnd,
            DayState::InRange,
            DayState::Today,
        ] {
            let result = range_bg_class(state, false);
            assert!(
                !result.contains("bg-secondary"),
                "expected no bg-secondary for {:?} out of month",
                state
            );
            assert!(
                !result.contains("bg-gradient"),
                "expected no bg-gradient for {:?} out of month",
                state
            );
        }
    }

    #[test]
    fn range_bg_class_today_no_special_bg() {
        let result = range_bg_class(DayState::Today, true);
        assert!(!result.contains("bg-secondary"));
        assert!(!result.contains("bg-gradient"));
    }

    #[test]
    fn range_bg_class_range_start_has_to_secondary() {
        let result = range_bg_class(DayState::RangeStart, true);
        assert!(result.contains("to-secondary"));
    }

    #[test]
    fn range_bg_class_range_end_has_to_secondary() {
        let result = range_bg_class(DayState::RangeEnd, true);
        assert!(result.contains("to-secondary"));
    }
}
