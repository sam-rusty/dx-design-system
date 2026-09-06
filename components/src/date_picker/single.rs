use super::Date as WireDate;
use crate::InputBase;
use dioxus::prelude::*;
use ds_utils::format::merge;
use time::Date;

use super::panel::{CalendarPanel, SelectionState};
use super::shared::{format_display_date, *};
use crate::calendar::{format_header_date, parse_date, today, use_calendar_state};
use crate::field_name::Field;
use crate::form::use_field_binding;
use crate::form::view::FormFieldFrame;

#[component]
pub fn DatePickerBase(
    /// Extra classes merged onto the trigger.
    #[props(default)]
    class: String,
    /// Controlled wire-date value (`YYYY-MM-DD`).
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Fired with the new wire-date value when a date is picked.
    #[props(default)]
    on_value_change: Callback<String>,
    /// Whether the picker is disabled.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Earliest selectable date.
    #[props(default)]
    min: Option<Signal<WireDate>>,
    /// Latest selectable date.
    #[props(default)]
    max: Option<Signal<WireDate>>,
    /// Shared open state (lets a wrapper observe/control the dialog). Setting
    /// it to `true` externally opens the dialog with fresh staging state.
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
    let mut input_value = use_signal(String::new);
    let input_value_read: ReadSignal<Option<String>> = use_memo(move || Some(input_value())).into();

    let selected_date = use_memo(move || parse_date(&value().unwrap_or_default()));

    // The selected date is derived from the form value — expose it directly as a
    // read view instead of mirroring it into a shadow signal via `use_effect`.
    let selection = SelectionState::Single(selected_date.into());

    let cal_state = use_calendar_state((*selected_date.peek()).unwrap_or_else(today));

    let min_date = use_memo(move || min.map(|s| s().into_inner()));
    let max_date = use_memo(move || max.map(|s| s().into_inner()));

    let is_date_disabled = move |date: Date| out_of_range(date, min_date(), max_date());

    let display_value = use_memo(move || {
        let val = value().unwrap_or_default();
        if val.is_empty() {
            String::new()
        } else {
            parse_date(&val)
                .map(|d| format_display_date(&d))
                .unwrap_or(val)
        }
    });

    let reset = use_callback(move |_: ()| {
        let nav_to = (*selected_date.peek()).unwrap_or_else(today);
        cal_state.navigate_to(&nav_to);
        input_mode.set(false);
    });
    let open = use_open_reset(is_open_sig, reset);

    let trigger_class = merge(&[TRIGGER_CLASS, &class]);
    let is_open_val = is_open_sig();
    let header_date = selected_date()
        .map(|d| format_header_date(&d))
        .unwrap_or_else(|| "Pick a date".to_string());

    rsx! {
        div { "data-name": "DatePicker", class: "relative w-full",
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
                    title: "Select date",
                    header: header_date,
                    on_close: move |_| is_open_sig.set(false),
                    input_mode: Some(input_mode.into()),
                    on_edit_toggle: move |_| {
                        let entering = !*input_mode.peek();
                        if entering {
                            let val = (*selected_date.peek())
                                .as_ref()
                                .map(|d| WireDate::from(*d).to_string())
                                .unwrap_or_default();
                            input_value.set(val);
                        } else if let Some(d) = parse_date(&input_value.peek())
                            && !is_date_disabled(d)
                        {
                            let formatted = WireDate::from(d).to_string();
                            on_value_change.call(formatted);
                            cal_state.navigate_to(&d);
                        }
                        input_mode.set(entering);
                    },
                    if input_mode() {
                        div { class: "py-4",
                            label { class: "block text-xs font-medium text-muted-foreground mb-2", "Date" }
                            InputBase {
                                placeholder: "YYYY-MM-DD".to_string(),
                                value: input_value_read,
                                on_value_change: move |v: String| input_value.set(v),
                            }
                        }
                    } else {
                        CalendarPanel {
                            state: cal_state,
                            selection: selection,
                            on_day_click: move |date: Date| {
                                let formatted = WireDate::from(date).to_string();
                                on_value_change.call(formatted);
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

/// Props for [`DatePicker`], the form-bound date picker.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
    /// Earliest selectable date.
    #[props(default)]
    pub min: Option<WireDate>,
    /// Latest selectable date.
    #[props(default)]
    pub max: Option<WireDate>,
    /// Whether the picker is disabled (OR-ed with the form's disabled state).
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
}

/// Form-bound date picker with stacked label and inline error.
pub fn DatePicker(props: DatePickerProps) -> Element {
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            class: props.class,
            DatePickerControl {
                min: props.min,
                max: props.max,
                disabled: props.disabled,
                open,
            }
        }
    }
}

#[component]
pub(crate) fn DatePickerControl(
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
        DatePickerBase {
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
