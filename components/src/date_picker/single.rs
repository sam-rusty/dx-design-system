use super::Date as WireDate;
use crate::{InputBase, Modal, ModalSize};
use dioxus::prelude::*;
use ds_utils::format::merge;
use time::Date;

use super::panel::{CalendarPanel, SelectionState};
use super::shared::{format_display_date, *};
use crate::calendar::{CalendarState, format_header_date, parse_date, today};
use crate::field_name::Field;
use crate::form::{FormError, FormField};

#[component]
pub fn DatePickerBase(
    #[props(default)] class: String,
    #[props(default)] value: Option<ReadSignal<String>>,
    #[props(default)] on_change: Option<EventHandler<String>>,
    #[props(default)] disabled: bool,
    #[props(default)] min: Option<Signal<WireDate>>,
    #[props(default)] max: Option<Signal<WireDate>>,
    #[props(default)] is_open: Option<Signal<bool>>,
) -> Element {
    let default_is_open = use_signal(|| false);
    let mut is_open_sig = is_open.unwrap_or(default_is_open);

    let mut input_mode = use_signal(|| false);
    let mut input_value = use_signal(String::new);

    let current_value = value.unwrap_or_else(|| use_signal(String::new).into());

    let selected_date = use_memo(move || parse_date(&current_value()));

    // The selected date is derived from the form value — expose it directly as a
    // read view instead of mirroring it into a shadow signal via `use_effect`.
    let selection = SelectionState::Single(selected_date.into());

    let cal_state = CalendarState::new((*selected_date.peek()).unwrap_or_else(today));

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
            parse_date(&val)
                .map(|d| format_display_date(&d))
                .unwrap_or(val)
        }
    });

    let trigger_class = merge(&[TRIGGER_CLASS, &class]);
    let is_open_val = is_open_sig();
    let header_date = selected_date()
        .map(|d| format_header_date(&d))
        .unwrap_or_else(|| "Pick a date".to_string());

    rsx! {
        div { "data-name": "DatePicker", class: "relative w-full",
            button {
                r#type: "button",
                class: "{trigger_class}",
                disabled: disabled,
                "data-state": if is_open_val { "open" } else { "closed" },
                onclick: move |ev| {
                    ev.stop_propagation();
                    if disabled { return; }
                    let nav_to = (*selected_date.peek()).unwrap_or_else(today);
                    cal_state.navigate_to(&nav_to);
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
                    PickerHeader { title: "Select date",
                        div { class: HEADER_DATE, "{header_date}" }
                        EditToggleButton {
                            input_mode: input_mode,
                            on_click: move |_| {
                                let entering = !*input_mode.peek();
                                if entering {
                                    let val = (*selected_date.peek())
                                        .as_ref()
                                        .map(|d| WireDate::from(*d).to_string())
                                        .unwrap_or_default();
                                    input_value.set(val);
                                } else if let Some(d) = parse_date(&input_value.peek())
                                    && !is_date_disabled(d) {
                                        let formatted = WireDate::from(d).to_string();
                                        if let Some(cb) = on_change {
                                            cb.call(formatted);
                                        }
                                        cal_state.navigate_to(&d);
                                    }
                                input_mode.set(entering);
                            },
                        }
                    }
                    div { class: "p-4",
                        if input_mode() {
                            div { class: "py-4",
                                label { class: "block text-xs font-medium text-muted-foreground mb-2", "Date" }
                                InputBase {
                                    placeholder: "YYYY-MM-DD".to_string(),
                                    value: input_value,
                                }
                            }
                        } else {
                            CalendarPanel {
                                state: cal_state,
                                selection: selection,
                                on_day_click: move |date: Date| {
                                    let formatted = WireDate::from(date).to_string();
                                    if let Some(cb) = on_change {
                                        cb.call(formatted);
                                    }
                                    is_open_sig.set(false);
                                },
                                min_date: min_date(),
                                max_date: max_date(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DatePicker(
    #[props(into)] field: Field,
    #[props(default)] class: String,
    #[props(default)] min: Option<WireDate>,
    #[props(default)] max: Option<WireDate>,
    #[props(default)] disabled: Option<Signal<bool>>,
) -> Element {
    let label = field.label.to_string();

    rsx! {
        FormField { field,
            DatePickerControl { label, class, min, max, disabled }
            FormError {}
        }
    }
}

#[component]
fn DatePickerControl(
    label: String,
    class: String,
    #[props(default)] min: Option<WireDate>,
    #[props(default)] max: Option<WireDate>,
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
            DatePickerBase {
                value: value_signal,
                on_change: on_change,
                disabled: is_disabled,
                is_open: is_open,
                min: min_opt,
                max: max_opt,
            }
            FloatingLabel { label: label, is_open: is_open, data_name: "DatePickerLabel" }
        }
    }
}
