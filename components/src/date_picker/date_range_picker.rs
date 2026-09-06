use super::Date as WireDate;
use dioxus::prelude::*;
use ds_utils::format::merge;
use time::Date;

use super::panel::{CalendarPanel, PickerFooter, SelectionState};
use super::shared::{format_display_date, *};
use crate::calendar::{
    format_date, format_date_range, format_header_date, parse_date, parse_date_range, today,
    use_calendar_state,
};
use crate::form::use_field_binding;
use crate::form::view::FormFieldFrame;
use crate::{Field, InputBase};

const TAB_ACTIVE: &str = "flex-1 py-2 text-sm font-medium text-primary border-b-2 border-primary cursor-pointer text-center transition-colors";

const TAB_INACTIVE: &str = "flex-1 py-2 text-sm font-medium text-muted-foreground border-b-2 border-transparent hover:text-foreground cursor-pointer text-center transition-colors";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RangeTab {
    Start,
    End,
}

/// Apply a day click to the staged range. Picking a start clears an end that
/// would precede it and moves on to the end tab; picking an "end" before the
/// current start restarts the range from that day. Returns the new
/// `(start, end, active_tab)`.
fn apply_range_pick(
    tab: RangeTab,
    start: Option<Date>,
    end: Option<Date>,
    picked: Date,
) -> (Option<Date>, Option<Date>, RangeTab) {
    match tab {
        RangeTab::Start => {
            let end = end.filter(|e| *e >= picked);
            (Some(picked), end, RangeTab::End)
        }
        RangeTab::End => {
            if start.is_some_and(|s| picked < s) {
                (Some(picked), None, RangeTab::End)
            } else {
                (start, Some(picked), RangeTab::End)
            }
        }
    }
}

/// What OK commits: both ends, or a single-day range when only the start was
/// picked. Nothing staged commits nothing.
fn range_commit(start: Option<Date>, end: Option<Date>) -> Option<(Date, Date)> {
    let s = start?;
    Some((s, end.unwrap_or(s)))
}

#[component]
pub fn DateRangePickerBase(
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
    #[props(default)] min: Option<Signal<WireDate>>,
    #[props(default)] max: Option<Signal<WireDate>>,
    /// Shared open state; setting it to `true` externally opens the dialog
    /// with fresh staging state.
    #[props(default)]
    is_open: Option<Signal<bool>>,
    /// DOM id for the trigger (targeted by the field label).
    #[props(default)]
    id: Option<String>,
    /// `aria-invalid` for the trigger.
    #[props(default)]
    aria_invalid: Option<String>,
    /// `aria-describedby` for the trigger (the field's error element).
    #[props(default)]
    aria_describedby: Option<String>,
) -> Element {
    let default_is_open = use_signal(|| false);
    let mut is_open_sig = is_open.unwrap_or(default_is_open);

    let mut input_mode = use_signal(|| false);
    let mut active_tab = use_signal(|| RangeTab::Start);
    let mut input_start = use_signal(String::new);
    let mut input_end = use_signal(String::new);
    let input_start_read: ReadSignal<Option<String>> = use_memo(move || Some(input_start())).into();
    let input_end_read: ReadSignal<Option<String>> = use_memo(move || Some(input_end())).into();

    let current_value = use_memo(move || value().unwrap_or_default());

    let parsed_range = use_memo(move || parse_date_range(&current_value()));

    let cal_state =
        use_calendar_state((*parsed_range.peek()).map(|(s, _)| s).unwrap_or_else(today));

    let mut staging_start: Signal<Option<Date>> = use_signal(|| None);
    let mut staging_end: Signal<Option<Date>> = use_signal(|| None);

    let selection = SelectionState::Range {
        start: staging_start,
        end: staging_end,
    };

    let min_date = use_memo(move || min.map(|s| s().into_inner()));
    let max_date = use_memo(move || max.map(|s| s().into_inner()));

    let is_date_disabled = move |date: Date| out_of_range(date, min_date(), max_date());

    let display_value = use_memo(move || {
        let val = current_value();
        if val.is_empty() {
            String::new()
        } else {
            parse_date_range(&val)
                .map(|(s, e)| format!("{} – {}", format_display_date(&s), format_display_date(&e)))
                .unwrap_or(val)
        }
    });

    let reset = use_callback(move |_: ()| {
        if let Some((s, e)) = *parsed_range.peek() {
            staging_start.set(Some(s));
            staging_end.set(Some(e));
            cal_state.navigate_to(&s);
        } else {
            staging_start.set(None);
            staging_end.set(None);
            cal_state.navigate_to(&today());
        }
        active_tab.set(RangeTab::Start);
        input_mode.set(false);
    });
    let open = use_open_reset(is_open_sig, reset);

    let trigger_class = merge(&[TRIGGER_CLASS, &class]);
    let is_open_val = is_open_sig();

    let header_text = {
        let s = *staging_start.read();
        let e = *staging_end.read();
        match (s, e) {
            (Some(sd), Some(ed)) => {
                format!("{} – {}", format_header_date(&sd), format_header_date(&ed))
            }
            (Some(sd), None) => format!("{} – ...", format_header_date(&sd)),
            _ => "Pick dates".to_string(),
        }
    };

    let start_active = active_tab() == RangeTab::Start;
    let start_tab_class = if start_active {
        TAB_ACTIVE
    } else {
        TAB_INACTIVE
    };
    let end_tab_class = if start_active {
        TAB_INACTIVE
    } else {
        TAB_ACTIVE
    };

    let start_tab_label = (*staging_start.read())
        .as_ref()
        .map(format_header_date)
        .unwrap_or_else(|| "Start".to_string());
    let end_tab_label = (*staging_end.read())
        .as_ref()
        .map(format_header_date)
        .unwrap_or_else(|| "End".to_string());

    rsx! {
        div { "data-name": "DateRangePicker", class: "relative w-full",
            PickerTrigger {
                class: trigger_class,
                display: display_value(),
                disabled: disabled(),
                is_open: is_open_val,
                id,
                aria_invalid,
                aria_describedby,
                on_open: move |_| open.call(()),
                CalendarIcon {}
            }

            if is_open_val {
                PickerDialog {
                    title: "Select date range",
                    header: header_text,
                    on_close: move |_| is_open_sig.set(false),
                    input_mode: Some(input_mode.into()),
                    on_edit_toggle: move |_| {
                        let entering = !*input_mode.peek();
                        if entering {
                            let start_str = (*staging_start.peek())
                                .as_ref()
                                .map(format_date)
                                .unwrap_or_default();
                            let end_str = (*staging_end.peek())
                                .as_ref()
                                .map(format_date)
                                .unwrap_or_default();
                            input_start.set(start_str);
                            input_end.set(end_str);
                        } else {
                            let s = parse_date(&input_start.peek());
                            let e = parse_date(&input_end.peek());
                            if let Some(sd) = s
                                && !is_date_disabled(sd)
                            {
                                staging_start.set(Some(sd));
                                cal_state.navigate_to(&sd);
                            }
                            if let Some(ed) = e
                                && !is_date_disabled(ed)
                                && s.is_some_and(|sd| ed >= sd)
                            {
                                staging_end.set(Some(ed));
                            }
                        }
                        input_mode.set(entering);
                    },
                    if input_mode() {
                        div { class: "flex flex-col gap-4 py-4",
                            div {
                                label { class: "block text-xs font-medium text-muted-foreground mb-2", "Start date" }
                                InputBase {
                                    placeholder: "YYYY-MM-DD".to_string(),
                                    value: input_start_read,
                                    on_value_change: move |v: String| input_start.set(v),
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-muted-foreground mb-2", "End date" }
                                InputBase {
                                    placeholder: "YYYY-MM-DD".to_string(),
                                    value: input_end_read,
                                    on_value_change: move |v: String| input_end.set(v),
                                }
                            }
                        }
                    } else {
                        div { class: "flex border-b border-border mb-3", role: "tablist", "aria-label": "Range bound",
                            button {
                                r#type: "button",
                                role: "tab",
                                "aria-selected": start_active,
                                class: "{start_tab_class}",
                                onclick: move |_| active_tab.set(RangeTab::Start),
                                "{start_tab_label}"
                            }
                            button {
                                r#type: "button",
                                role: "tab",
                                "aria-selected": !start_active,
                                class: "{end_tab_class}",
                                onclick: move |_| active_tab.set(RangeTab::End),
                                "{end_tab_label}"
                            }
                        }
                        CalendarPanel {
                            state: cal_state,
                            selection: selection,
                            on_day_click: move |date: Date| {
                                let (s, e, tab) = apply_range_pick(
                                    *active_tab.peek(),
                                    *staging_start.peek(),
                                    *staging_end.peek(),
                                    date,
                                );
                                staging_start.set(s);
                                staging_end.set(e);
                                active_tab.set(tab);
                            },
                            min_date: min_date(),
                            max_date: max_date(),
                        }
                    }
                    PickerFooter {
                        on_cancel: move |_| is_open_sig.set(false),
                        on_confirm: move |_| {
                            if let Some((s, e)) = range_commit(*staging_start.peek(), *staging_end.peek()) {
                                on_value_change.call(format_date_range(&s, &e));
                            }
                            is_open_sig.set(false);
                        },
                    }
                }
            }
        }
    }
}

/// Props for [`DateRangePicker`], the form-bound date-range picker.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
    /// Earliest selectable value.
    #[props(default)]
    pub min: Option<WireDate>,
    /// Latest selectable value.
    #[props(default)]
    pub max: Option<WireDate>,
    /// Whether the picker is disabled (OR-ed with the form's disabled state).
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
}

/// Form-bound date-range picker with stacked label and inline error.
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            class: props.class,
            DateRangePickerControl {
                min: props.min,
                max: props.max,
                disabled: props.disabled,
                open,
            }
        }
    }
}

#[component]
pub(crate) fn DateRangePickerControl(
    #[props(default)] min: Option<WireDate>,
    #[props(default)] max: Option<WireDate>,
    #[props(default)] disabled: ReadSignal<bool>,
    open: Signal<bool>,
) -> Element {
    let binding = use_field_binding();
    let form_disabled = binding.disabled;
    let is_disabled: ReadSignal<bool> = use_memo(move || disabled() || form_disabled()).into();

    let min_opt = use_optional_signal(min);
    let max_opt = use_optional_signal(max);

    rsx! {
        DateRangePickerBase {
            value: binding.controlled_value,
            on_value_change: binding.on_commit,
            disabled: is_disabled,
            is_open: open,
            min: min_opt,
            max: max_opt,
            id: binding.id.clone(),
            aria_invalid: binding.aria_invalid(),
            aria_describedby: binding.aria_describedby_when_invalid(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn pick_start_then_end_in_order() {
        let (s, e, tab) = apply_range_pick(RangeTab::Start, None, None, date(2026, 3, 1));
        assert_eq!((s, e, tab), (Some(date(2026, 3, 1)), None, RangeTab::End));
        let (s, e, tab) = apply_range_pick(tab, s, e, date(2026, 3, 15));
        assert_eq!(
            (s, e, tab),
            (
                Some(date(2026, 3, 1)),
                Some(date(2026, 3, 15)),
                RangeTab::End
            )
        );
    }

    #[test]
    fn picking_start_after_end_clears_end() {
        let (s, e, tab) = apply_range_pick(
            RangeTab::Start,
            Some(date(2026, 3, 1)),
            Some(date(2026, 3, 10)),
            date(2026, 3, 20),
        );
        assert_eq!((s, e, tab), (Some(date(2026, 3, 20)), None, RangeTab::End));
    }

    #[test]
    fn picking_start_before_end_keeps_end() {
        let (s, e, _) = apply_range_pick(
            RangeTab::Start,
            Some(date(2026, 3, 5)),
            Some(date(2026, 3, 10)),
            date(2026, 3, 2),
        );
        assert_eq!((s, e), (Some(date(2026, 3, 2)), Some(date(2026, 3, 10))));
    }

    #[test]
    fn picking_end_before_start_restarts_range() {
        let (s, e, tab) = apply_range_pick(
            RangeTab::End,
            Some(date(2026, 3, 10)),
            Some(date(2026, 3, 12)),
            date(2026, 3, 4),
        );
        assert_eq!((s, e, tab), (Some(date(2026, 3, 4)), None, RangeTab::End));
    }

    #[test]
    fn picking_end_equal_to_start_is_a_single_day_range() {
        let d = date(2026, 3, 10);
        let (s, e, _) = apply_range_pick(RangeTab::End, Some(d), None, d);
        assert_eq!((s, e), (Some(d), Some(d)));
    }

    #[test]
    fn commit_rules() {
        let s = date(2026, 3, 1);
        let e = date(2026, 3, 9);
        assert_eq!(range_commit(None, None), None);
        assert_eq!(range_commit(None, Some(e)), None);
        assert_eq!(range_commit(Some(s), None), Some((s, s)));
        assert_eq!(range_commit(Some(s), Some(e)), Some((s, e)));
    }
}
