use dioxus::prelude::*;
use ds_utils::format::merge;
use strum_macros::AsRefStr;

use crate::Field;
use crate::form::use_field_binding;
use crate::form::view::{FormError, FormField, LabelHint};
use crate::hooks::{FocusState, use_focus_control, use_focus_entry_disabled, use_focus_provider};
use crate::label::Label;

#[derive(Default, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum RadioGroupDirection {
    #[default]
    Vertical,
    Horizontal,
}

impl RadioGroupDirection {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Vertical => "flex flex-col gap-2",
            Self::Horizontal => "flex flex-row flex-wrap gap-4",
        }
    }
}

/// Group-level context shared by every [`RadioGroupItem`]: the current value (one
/// memo for the whole group), the roving-focus state, and the option list (so an
/// item can resolve its neighbour by index for arrow-key navigation).
#[derive(Clone, Copy)]
struct RadioGroupCtx {
    selected: Memo<String>,
    focus: FocusState,
    options: &'static [(&'static str, &'static str)],
}

/// Props for [`RadioGroup`], the form-bound radio group.
#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Layout direction of the options.
    #[props(default)]
    pub direction: RadioGroupDirection,
    /// Extra classes merged onto the group container.
    #[props(default)]
    pub class: String,
    /// `(value, label)` pairs (e.g. a `FormOptions` derive's `OPTIONS`).
    #[props(default)]
    pub options: &'static [(&'static str, &'static str)],
    /// Help tooltip rendered inline after the group label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra items rendered after the options.
    #[props(default)]
    pub children: Option<Element>,
}

/// Form-bound radio group with inline error.
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    let aria_label = props.field.label;
    rsx! {
        FormField { field: props.field,
            RadioGroupControl {
                direction: props.direction,
                class: props.class,
                options: props.options,
                aria_label,
                tooltip: props.tooltip,
                children: props.children,
            }
            FormError {}
        }
    }
}

/// Form-context binding — lives *inside* `FormField` so it can read the field
/// binding, build the shared value memo, and provide the roving focus state.
#[component]
fn RadioGroupControl(
    direction: RadioGroupDirection,
    #[props(default)] class: String,
    options: &'static [(&'static str, &'static str)],
    aria_label: &'static str,
    #[props(default)] tooltip: Option<Element>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let binding = use_field_binding();

    let selected = binding.value;
    // Radio groups wrap on arrow nav.
    let focus = use_focus_provider(use_signal(|| true).into());
    use_context_provider(|| RadioGroupCtx {
        selected,
        focus,
        options,
    });

    let merged_class = merge(&[direction.class(), &class]);
    let direction_attr = direction.as_ref();

    rsx! {
        if let Some(t) = tooltip {
            Label {
                class: "mt-2 text-foreground",
                "{aria_label}"
                LabelHint { tooltip: t }
            }
        }
        div {
            "data-name": "RadioGroup",
            "data-direction": direction_attr,
            role: "radiogroup",
            "aria-label": aria_label,
            class: "{merged_class}",
            for (i , (value , label)) in options.iter().enumerate() {
                RadioGroupItem { key: "{value}", index: i, label: *label, value: value.to_string() }
            }
            if let Some(c) = children {
                {c}
            }
        }
    }
}

#[component]
fn RadioGroupItem(
    index: usize,
    label: &'static str,
    value: String,
    #[props(default)] class: String,
) -> Element {
    let binding = use_field_binding();
    let ctx = use_context::<RadioGroupCtx>();
    let is_disabled = binding.disabled;

    // Register with roving focus + receive DOM focus when this index becomes active.
    let idx = use_signal(|| index);
    use_focus_entry_disabled(ctx.focus, idx, move || is_disabled());
    let on_mounted = use_focus_control(ctx.focus, idx);

    let radio_id = format!("{}-{}", binding.name, value);
    let radio_id_label = radio_id.clone();
    let name_str = binding.id.clone();
    let aria_describedby = binding.aria_describedby.clone();

    let is_checked = ctx.selected.with(|s| s == &value);
    let any_selected = ctx.selected.with(|s| !s.is_empty());
    // Roving tabindex: only one item is in the tab order — the selected one, or
    // the first when nothing is selected yet (WAI-ARIA radio pattern).
    let tab_index = if is_checked || (!any_selected && index == 0) {
        "0"
    } else {
        "-1"
    };

    let aria_invalid = binding.aria_invalid();
    let select = binding.on_commit;

    let wrapper_class = merge(&[
        "group/radio relative inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        &class,
    ]);
    let span_class = if is_checked {
        "size-[18px] rounded-full border-[5px] border-primary transition-all duration-150"
    } else {
        "size-[18px] rounded-full border-2 border-muted-foreground/40 transition-all duration-150"
    };

    let value_click = value.clone();
    let options = ctx.options;
    let mut focus = ctx.focus;

    rsx! {
        div { class: "flex items-center gap-2 mt-2 mb-2",
            button {
                "data-name": "Radio",
                r#type: "button",
                role: "radio",
                id: "{radio_id}",
                name: name_str,
                "aria-checked": "{is_checked}",
                "aria-invalid": aria_invalid,
                "aria-describedby": aria_describedby,
                tabindex: tab_index,
                disabled: is_disabled(),
                class: "{wrapper_class}",
                onmounted: on_mounted,
                onclick: move |ev| {
                    ev.stop_propagation();
                    if !is_disabled() {
                        select.call(value_click.clone());
                    }
                },
                onkeydown: move |ev| {
                    if is_disabled() || options.is_empty() {
                        return;
                    }
                    let len = options.len();
                    let target = match ev.key() {
                        Key::ArrowDown | Key::ArrowRight => (index + 1) % len,
                        Key::ArrowUp | Key::ArrowLeft => (index + len - 1) % len,
                        Key::Home => 0,
                        Key::End => len - 1,
                        _ => return,
                    };
                    ev.prevent_default();
                    // Move roving focus (DOM focus follows via the focus-control hook)
                    // and select on focus, per the WAI-ARIA radio pattern.
                    focus.set_focus(Some(target));
                    if let Some((v, _)) = options.get(target) {
                        select.call(v.to_string());
                    }
                },
                span { class: "{span_class}" }
            }
            Label {
                html_for: radio_id_label,
                class: "text-foreground cursor-pointer font-medium leading-none",
                "{label}"
            }
        }
    }
}

/// Props for [`Radio`], the standalone controlled radio button.
#[derive(Props, Clone, PartialEq)]
pub struct RadioProps {
    /// Extra classes merged onto the radio button.
    #[props(default)]
    pub class: String,
    /// Value reported through `on_select` when clicked.
    #[props(default)]
    pub value: Option<String>,
    /// Whether the radio is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Whether the radio renders checked.
    #[props(default)]
    pub checked: bool,
    /// Fired with `value` when the radio is clicked.
    #[props(default)]
    pub on_select: Callback<String>,
}

/// Standalone controlled radio button for non-form contexts that own their own
/// selection state (e.g. inline option pickers).
pub fn Radio(props: RadioProps) -> Element {
    let wrapper_class = merge(&[
        "group/radio relative inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        &props.class,
    ]);
    let radio_value = props.value.clone().unwrap_or_default();
    let span_class = if props.checked {
        "size-[18px] rounded-full border-[5px] border-primary transition-all duration-150"
    } else {
        "size-[18px] rounded-full border-2 border-muted-foreground/40 transition-all duration-150"
    };

    let checked = props.checked;
    let disabled = props.disabled;
    let on_select = props.on_select;

    rsx! {
        button {
            "data-name": "Radio",
            r#type: "button",
            role: "radio",
            "aria-checked": "{checked}",
            class: "{wrapper_class}",
            disabled: disabled(),
            onclick: move |ev: MouseEvent| {
                ev.stop_propagation();
                on_select.call(radio_value.clone());
            },
            span { class: "{span_class}" }
        }
    }
}
