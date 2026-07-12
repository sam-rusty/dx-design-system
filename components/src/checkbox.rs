use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::field_name::Field;
use crate::form::{FormError, FormField, LabelHint, use_field_binding};
use crate::hooks::use_controlled;
use crate::icon::{Icon, IconName};
use crate::label::Label;

/// Props for [`CheckboxBase`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxBaseProps {
    /// Controlled checked state. `Some` makes the caller the source of truth
    /// (pair with `on_checked_change`); `None` leaves it uncontrolled.
    #[props(default)]
    pub checked: ReadSignal<Option<bool>>,
    /// Initial checked state when uncontrolled.
    #[props(default)]
    pub default_checked: bool,
    /// Fired with the new checked state on toggle.
    #[props(default)]
    pub on_checked_change: Callback<bool>,
    /// Whether the checkbox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Extra classes merged onto the checkbox button.
    #[props(default)]
    pub class: String,
    /// DOM id. Form bindings set this to the field name so labels target it.
    #[props(default)]
    pub id: Option<String>,
    /// `aria-invalid` value (form bindings set `"true"` on validation failure).
    #[props(default)]
    pub aria_invalid: Option<String>,
    /// `aria-describedby` target (the field's error element id).
    #[props(default)]
    pub aria_describedby: Option<String>,
    /// `aria-labelledby` target (the row label).
    #[props(default)]
    pub aria_labelledby: Option<String>,
    /// Additional attributes (`name`, `required`, ...).
    #[props(extends = GlobalAttributes, extends = button)]
    pub attributes: Vec<Attribute>,
}

pub fn CheckboxBase(props: CheckboxBaseProps) -> Element {
    let (checked, set_checked) = use_controlled(
        props.checked,
        props.default_checked,
        props.on_checked_change,
    );

    let wrapper_class = merge(&[
        "group/checkbox relative inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        &props.class,
    ]);

    let is_checked = checked();
    let span_class = if is_checked {
        "flex items-center justify-center size-[17px] rounded-sm bg-primary border-2 border-primary transition-all duration-150"
    } else {
        "flex items-center justify-center size-[17px] rounded-sm border-2 border-muted-foreground/40 bg-transparent transition-all duration-150"
    };

    let icon_class = if is_checked {
        "size-2 text-primary-foreground opacity-100 transition-opacity duration-150"
    } else {
        "size-2 text-primary-foreground opacity-0 transition-opacity duration-150"
    };

    let disabled = props.disabled;

    rsx! {
        button {
            "data-name": "Checkbox",
            r#type: "button",
            role: "checkbox",
            "aria-checked": "{is_checked}",
            class: "{wrapper_class}",
            id: props.id.clone(),
            disabled: disabled(),
            "aria-labelledby": props.aria_labelledby.clone(),
            "aria-invalid": props.aria_invalid.clone(),
            "aria-describedby": props.aria_describedby.clone(),
            onclick: move |ev| {
                ev.stop_propagation();
                set_checked(!is_checked);
            },
            onkeydown: move |ev| {
                // ARIA checkbox: Space toggles (native button click), Enter must not.
                if ev.key() == Key::Enter {
                    ev.prevent_default();
                }
            },
            ..props.attributes,
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

/// Form-context binding for [`CheckboxBase`].
#[component]
pub(crate) fn CheckboxControl(
    #[props(default)] class: String,
    #[props(default)] aria_labelledby: Option<String>,
) -> Element {
    let binding = use_field_binding();

    let value = binding.value;
    let checked: ReadSignal<Option<bool>> = use_memo(move || Some(value() == "true")).into();
    let on_commit = binding.on_commit;

    rsx! {
        CheckboxBase {
            class,
            id: binding.id.clone(),
            disabled: ReadSignal::from(binding.disabled),
            checked,
            aria_describedby: binding.aria_describedby.clone(),
            aria_invalid: binding.aria_invalid(),
            aria_labelledby,
            on_checked_change: move |checked: bool| on_commit.call(checked.to_string()),
        }
    }
}

/// Props for [`Checkbox`], the form-bound checkbox row.
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Extra classes merged onto the row.
    #[props(default)]
    pub class: String,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
}

/// Form-bound checkbox with trailing label and inline error.
pub fn Checkbox(props: CheckboxProps) -> Element {
    let label = props.field.label.to_string();
    rsx! {
        FormField { field: props.field,
            CheckboxRow { class: props.class, label, tooltip: props.tooltip }
            FormError {}
        }
    }
}

#[component]
pub(crate) fn CheckboxRow(
    label: String,
    #[props(default)] class: String,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let binding = use_field_binding();
    let label_id = format!("{}-label", binding.name);
    // The control's id is the field name (see `CheckboxControl`); a real `<label for>`
    // forwards clicks to it, so the row needs no second toggle handler.
    let html_for = binding.id.clone();

    rsx! {
        div {
            class: merge(&["flex items-center gap-2 mt-2 mb-2", &class]),
            CheckboxControl { aria_labelledby: Some(label_id.clone()) }
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
