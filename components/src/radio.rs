use dioxus::prelude::*;
use ds_utils::format::merge;
use strum_macros::AsRefStr;

use crate::Field;
use crate::form::{FieldContext, FormContext, FormError, FormField, LabelHint};
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

#[component]
pub fn RadioGroup(
    #[props(into)] field: Field,
    #[props(default)] direction: RadioGroupDirection,
    #[props(default)] class: String,
    #[props(default)] options: &'static [(&'static str, &'static str)],
    #[props(default)] tooltip: Option<Element>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let aria_label = field.label;
    rsx! {
        FormField { field,
            RadioGroupBody { direction, class, options, aria_label, tooltip, children }
            FormError {}
        }
    }
}

/// Inner body — lives *inside* [`FormField`] so it can read the field's
/// [`FieldContext`]/[`FormContext`], build the shared value memo, and provide
/// the roving [`FocusState`] to its items.
#[component]
fn RadioGroupBody(
    direction: RadioGroupDirection,
    #[props(default)] class: String,
    options: &'static [(&'static str, &'static str)],
    aria_label: &'static str,
    #[props(default)] tooltip: Option<Element>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let form_ctx = use_context::<FormContext>();
    let field_name = field_ctx.name.clone();

    let selected = use_memo(move || {
        form_ctx
            .values_signal
            .with(|v| v.get(&*field_name).cloned().unwrap_or_default())
    });
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
    let field_ctx = use_context::<FieldContext>();
    let form_ctx = use_context::<FormContext>();
    let ctx = use_context::<RadioGroupCtx>();
    let field_name = field_ctx.name.clone();
    let is_disabled = form_ctx.disabled.map(|d| d()).unwrap_or(false);

    // Register with roving focus + receive DOM focus when this index becomes active.
    let idx = use_signal(|| index);
    use_focus_entry_disabled(ctx.focus, idx, move || is_disabled);
    let on_mounted = use_focus_control(ctx.focus, idx);

    let radio_id = format!("{}-{}", field_name, value);
    let radio_id_label = radio_id.clone();
    let name_str = String::from(&*field_name);
    let aria_describedby = format!("{}-error", field_name);

    let is_checked = ctx.selected.with(|s| s == &value);
    let any_selected = ctx.selected.with(|s| !s.is_empty());
    // Roving tabindex: only one item is in the tab order — the selected one, or
    // the first when nothing is selected yet (WAI-ARIA radio pattern).
    let tab_index = if is_checked || (!any_selected && index == 0) {
        "0"
    } else {
        "-1"
    };

    let is_touched = form_ctx.touched_signal.with(|t| t.contains(&*field_name));
    let has_error = form_ctx
        .errors_signal
        .with(|e| e.get(&*field_name).is_some_and(|err| err.is_some()));
    let aria_invalid: Option<String> = (is_touched && has_error).then(|| "true".to_string());

    let select = use_callback({
        let field_name = field_name.clone();
        move |val: String| {
            form_ctx.set_value.read()(&field_name, val);
            form_ctx.touch_field.read()(&field_name);
        }
    });

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
                disabled: is_disabled,
                class: "{wrapper_class}",
                onmounted: on_mounted,
                onclick: move |ev| {
                    ev.stop_propagation();
                    if !is_disabled {
                        select.call(value_click.clone());
                    }
                },
                onkeydown: move |ev| {
                    if is_disabled || options.is_empty() {
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

/// Standalone controlled radio button for non-form contexts that own their own
/// selection state (e.g. the inline option pickers in `multi_search`).
#[component]
pub fn Radio(
    #[props(default)] class: String,
    #[props(default)] value: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] checked: bool,
    #[props(default)] on_select: Option<EventHandler<String>>,
) -> Element {
    let wrapper_class = merge(&[
        "group/radio relative inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        &class,
    ]);
    let radio_value = value.unwrap_or_default();
    let span_class = if checked {
        "size-[18px] rounded-full border-[5px] border-primary transition-all duration-150"
    } else {
        "size-[18px] rounded-full border-2 border-muted-foreground/40 transition-all duration-150"
    };

    rsx! {
        button {
            "data-name": "Radio",
            r#type: "button",
            role: "radio",
            "aria-checked": "{checked}",
            class: "{wrapper_class}",
            disabled,
            onclick: move |ev: MouseEvent| {
                ev.stop_propagation();
                if let Some(cb) = &on_select {
                    cb.call(radio_value.clone());
                }
            },
            span { class: "{span_class}" }
        }
    }
}
