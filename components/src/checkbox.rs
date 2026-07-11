use dioxus::prelude::*;
use utils::format::merge;

use crate::field_name::Field;
use crate::form::{FieldContext, FormContext, FormError, FormField, LabelHint};
use crate::icon::{Icon, IconName};
use crate::label::Label;

#[component]
pub fn CheckboxBase(
    #[props(default)] class: String,
    #[props(default)] id: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] checked: bool,
    #[props(default)] on_change: Option<EventHandler<bool>>,
    #[props(default)] aria_describedby: Option<String>,
    #[props(default)] aria_invalid: Option<String>,
    #[props(default)] aria_labelledby: Option<String>,
) -> Element {
    let wrapper_class = merge(&[
        "group/checkbox relative inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        &class,
    ]);

    let span_class = if checked {
        "flex items-center justify-center size-[17px] rounded-sm bg-primary border-2 border-primary transition-all duration-150"
    } else {
        "flex items-center justify-center size-[17px] rounded-sm border-2 border-muted-foreground/40 bg-transparent transition-all duration-150"
    };

    let icon_class = if checked {
        "size-2 text-primary-foreground opacity-100 transition-opacity duration-150"
    } else {
        "size-2 text-primary-foreground opacity-0 transition-opacity duration-150"
    };

    let aria_checked = checked.to_string();

    rsx! {
        button {
            "data-name": "Checkbox",
            r#type: "button",
            role: "checkbox",
            "aria-checked": "{aria_checked}",
            class: "{wrapper_class}",
            id: id,
            name: name,
            disabled: disabled,
            "aria-required": required,
            "aria-labelledby": aria_labelledby,
            "aria-invalid": aria_invalid,
            "aria-describedby": aria_describedby,
            onclick: move |ev| {
                ev.stop_propagation();
                let new_val = !checked;
                if let Some(cb) = &on_change {
                    cb.call(new_val);
                }
            },
            onkeydown: move |ev| {
                // ARIA checkbox: Space toggles (native button click), Enter must not.
                if ev.key() == Key::Enter {
                    ev.prevent_default();
                }
            },
            span { class: "{span_class}",
                Icon {
                    name: IconName::Check,
                    stroke_width: 3.0,
                    class: "{icon_class}",
                }
            }
        }
    }
}

#[component]
pub(crate) fn CheckboxFormControl(
    #[props(default)] class: String,
    #[props(default)] aria_labelledby: Option<String>,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let field_name = field_ctx.name.clone();
    let form_ctx = use_context::<FormContext>();
    let is_disabled = form_ctx.disabled.map(|d| d()).unwrap_or(false);

    let id = String::from(&*field_name);
    let aria_describedby = format!("{}-error", field_name);

    let is_checked = form_ctx
        .values_signal
        .with(|v| v.get(&*field_name).map(|v| v == "true").unwrap_or(false));

    let is_touched = form_ctx.touched_signal.with(|t| t.contains(&*field_name));
    let has_error = form_ctx
        .errors_signal
        .with(|e| e.get(&*field_name).is_some_and(|err| err.is_some()));
    let aria_invalid: Option<String> = if is_touched && has_error {
        Some("true".to_string())
    } else {
        None
    };

    rsx! {
        CheckboxBase {
            class: class,
            id: id,
            disabled: is_disabled,
            checked: is_checked,
            aria_describedby: aria_describedby,
            aria_invalid: aria_invalid,
            aria_labelledby: aria_labelledby,
            on_change: {
                let field_name = field_name.clone();
                EventHandler::new(move |checked: bool| {
                    form_ctx.set_value.read()(&field_name, checked.to_string());
                    form_ctx.touch_field.read()(&field_name);
                })
            },
        }
    }
}

#[component]
pub fn Checkbox(
    field: Field,
    #[props(default)] class: String,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let label = field.label.to_string();
    rsx! {
        FormField { field,
            CheckboxRow { class: class, label, tooltip }
            FormError {}
        }
    }
}

#[component]
fn CheckboxRow(
    label: String,
    #[props(default)] class: String,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let field_name = field_ctx.name.clone();
    let label_id = format!("{}-label", field_name);
    // The control's id is the field name (see `CheckboxFormControl`); a real `<label for>`
    // forwards clicks to it, so the row needs no second toggle handler.
    let html_for = String::from(&*field_name);

    rsx! {
        div {
            class: merge(&["flex items-center gap-2 mt-2 mb-2", &class]),
            CheckboxFormControl { aria_labelledby: Some(label_id.clone()) }
            Label {
                html_for,
                id: label_id,
                class: "text-foreground cursor-pointer",
                "{label}"
                if let Some(t) = tooltip {
                    LabelHint { tooltip: t }
                }
            }
        }
    }
}
