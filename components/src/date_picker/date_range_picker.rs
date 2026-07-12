use super::Date as WireDate;
use dioxus::prelude::*;
use ds_utils::format::merge;
use time::Date;

use super::panel::{CalendarPanel, PickerFooter, SelectionState};
use super::shared::{format_display_date, *};
use crate::calendar::{
    CalendarState, format_date, format_date_range, format_header_date, parse_date,
    parse_date_range, today,
};
use crate::form::{FormFieldFrame, use_field_binding};
use crate::{Field, InputBase, Modal, ModalSize};

const TAB_ACTIVE: &str = "flex-1 py-2 text-sm font-medium text-primary border-b-2 border-primary cursor-pointer text-center transition-colors";

const TAB_INACTIVE: &str = "flex-1 py-2 text-sm font-medium text-muted-foreground border-b-2 border-transparent hover:text-foreground cursor-pointer text-center transition-colors";

#[derive(Clone, Copy, PartialEq, Eq)]
enum RangeTab {
    Start,
    End,
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
    #[props(default)] is_open: Option<Signal<bool>>,
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
        CalendarState::new((*parsed_range.peek()).map(|(s, _)| s).unwrap_or_else(today));

    let mut staging_start: Signal<Option<Date>> = use_signal(|| None);
    let mut staging_end: Signal<Option<Date>> = use_signal(|| None);

    let selection = SelectionState::Range {
        start: staging_start,
        end: staging_end,
    };

    let min_date = use_memo(move || min.map(|s| s().into_inner()));
    let max_date = use_memo(move || max.map(|s| s().into_inner()));

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
            parse_date_range(&val)
                .map(|(s, e)| format!("{} – {}", format_display_date(&s), format_display_date(&e)))
                .unwrap_or(val)
        }
    });

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

    let start_tab_class = if active_tab() == RangeTab::Start {
        TAB_ACTIVE
    } else {
        TAB_INACTIVE
    };
    let end_tab_class = if active_tab() == RangeTab::End {
        TAB_ACTIVE
    } else {
        TAB_INACTIVE
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
            button {
                r#type: "button",
                class: "{trigger_class}",
                disabled: disabled(),
                "data-state": if is_open_val { "open" } else { "closed" },
                onclick: move |ev| {
                    ev.stop_propagation();
                    if disabled() { return; }
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
                    is_open_sig.set(true);
                },
                span { class: "truncate", "{display_or_nbsp(display_value())}" }
                CalendarIcon {}
            }

            if is_open_val {
                Modal {
                    on_close: move || is_open_sig.set(false),
                    headerless: true,
                    size: ModalSize::Sm,
                    PickerHeader { title: "Select date range",
                        div { class: HEADER_DATE, "{header_text}" }
                        EditToggleButton {
                            input_mode: input_mode,
                            on_click: move |_| {
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
                                        && !is_date_disabled(sd) {
                                            staging_start.set(Some(sd));
                                            cal_state.navigate_to(&sd);
                                        }
                                    if let Some(ed) = e
                                        && !is_date_disabled(ed) && s.is_some_and(|sd| ed >= sd) {
                                            staging_end.set(Some(ed));
                                        }
                                }
                                input_mode.set(entering);
                            },
                        }
                    }
                    div { class: "p-4",
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
                            div { class: "flex border-b border-border mb-3",
                                button {
                                    r#type: "button",
                                    class: "{start_tab_class}",
                                    onclick: move |_| active_tab.set(RangeTab::Start),
                                    "{start_tab_label}"
                                }
                                button {
                                    r#type: "button",
                                    class: "{end_tab_class}",
                                    onclick: move |_| active_tab.set(RangeTab::End),
                                    "{end_tab_label}"
                                }
                            }
                            CalendarPanel {
                                state: cal_state,
                                selection: selection,
                                on_day_click: move |date: Date| {
                                    let tab = *active_tab.peek();
                                    match tab {
                                        RangeTab::Start => {
                                            staging_start.set(Some(date));
                                            if staging_end.peek().is_some_and(|e| e < date) {
                                                staging_end.set(None);
                                            }
                                            active_tab.set(RangeTab::End);
                                        }
                                        RangeTab::End => {
                                            if staging_start.peek().is_some_and(|s| date < s) {
                                                staging_start.set(Some(date));
                                                staging_end.set(None);
                                                active_tab.set(RangeTab::End);
                                            } else {
                                                staging_end.set(Some(date));
                                            }
                                        }
                                    }
                                },
                                min_date: min_date(),
                                max_date: max_date(),
                            }
                        }
                        PickerFooter {
                            on_cancel: move |_| is_open_sig.set(false),
                            on_confirm: move |_| {
                                let start = *staging_start.peek();
                                let end = *staging_end.peek();
                                if let (Some(s), Some(e)) = (start, end) {
                                    let formatted = format_date_range(&s, &e);
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
fn DateRangePickerControl(
    #[props(default)] min: Option<WireDate>,
    #[props(default)] max: Option<WireDate>,
    #[props(default)] disabled: ReadSignal<bool>,
    open: Signal<bool>,
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
        DateRangePickerBase {
            value: binding.controlled_value,
            on_value_change: binding.on_commit,
            disabled: is_disabled,
            is_open: open,
            min: min_opt,
            max: max_opt,
        }
    }
}
