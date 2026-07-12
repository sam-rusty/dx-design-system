use std::sync::Arc;

use crate::{Alert, AlertVariant};
use dioxus::prelude::*;
use ds_utils::format::{
    clamp_percent, filter_percent, format_number, format_percent, format_phone, merge,
    parse_number, parse_percent, parse_phone,
};

use crate::copyable::copy_to_clipboard;
use crate::field_name::Field;
use crate::form::{FieldContext, Form as FormHook, FormContext, FormData, FormSubmit};
use crate::icon::{Icon, IconName};
use crate::input::{InputBase, InputSize, InputType};
use crate::label::Label;
use crate::separator::Separator;
use crate::stepper::{auto_register_field, unregister_auto_field};
use crate::tooltip::Tooltip;

/// Help icon that reveals `tooltip` content on hover/focus, rendered inline after
/// a field label. `pointer-events-auto` re-enables interaction on floating labels
/// (which are `pointer-events-none`); `peer-placeholder-shown:pointer-events-none`
/// keeps it inert while the field is empty and the label sits over the input.
#[component]
pub(crate) fn LabelHint(tooltip: Element) -> Element {
    rsx! {
        Tooltip {
            title: tooltip,
            class: "pointer-events-auto peer-placeholder-shown:pointer-events-none align-middle",
            Icon {
                name: IconName::CircleHelp,
                class: "size-3.5 text-muted-foreground hover:text-foreground",
            }
        }
    }
}

#[component]
pub fn FormProvider<T>(
    form: FormHook<T>,
    #[props(default)] action: Option<FormSubmit<T>>,
    #[props(default)] loading: Option<Signal<bool>>,
    #[props(default = true)] inline_error: bool,
    children: Element,
) -> Element
where
    T: FormData + Send + Sync + 'static,
{
    use crate::form::{FormContext, SetValueFn, SubmitFn, TouchFieldFn};

    let set_value = use_hook(move || {
        CopyValue::new_in_scope(
            Box::new(move |field: &str, value: String| {
                form.set_string_value(field, value);
            }) as SetValueFn,
            ScopeId::ROOT,
        )
    });

    let touch_field = use_hook(move || {
        CopyValue::new_in_scope(
            Box::new(move |field: &str| {
                form.touch_field(field);
            }) as TouchFieldFn,
            ScopeId::ROOT,
        )
    });

    let submit = action.map(|action| {
        use_hook(move || {
            CopyValue::new_in_scope(
                Box::new(move || {
                    form.clear_global_error();
                    if let Some(data) = form.validate_and_get() {
                        action.call(data);
                    }
                }) as SubmitFn,
                ScopeId::ROOT,
            )
        })
    });

    use_effect(move || {
        if inline_error && let Some(Some(Err(err))) = action.map(|a| a.result()) {
            form.set_server_error(err);
        }
    });

    // Derive the disabled flag reactively instead of mirroring a prop into a signal
    // via `use_effect`. Precedence is unchanged: a submit `action`'s `pending()`
    // wins when present; otherwise an explicit `loading` signal is used. Both are
    // read through a `use_memo` so re-renders track them directly (aligns with
    // `use_controlled` semantics).
    let disabled: Option<Memo<bool>> = match (action, loading) {
        (Some(action), _) => Some(use_memo(move || action.pending())),
        (None, Some(sig)) => Some(use_memo(move || sig())),
        (None, None) => None,
    };

    let ctx = FormContext {
        values_signal: form.values_signal,
        errors_signal: form.errors_signal,
        touched_signal: form.touched_signal,
        set_value,
        touch_field,
        disabled,
        submit,
        step_field_registry: None,
    };

    use_context_provider(|| ctx);

    rsx! {
        {children}

    }
}

#[component]
pub fn Form(
    #[props(default)] class: String,
    #[props(default)] on_submit: Option<EventHandler<FormEvent>>,
    children: Element,
) -> Element {
    let ctx = use_context::<FormContext>();
    let merged_class = merge(&["w-full", &class]);

    let is_disabled = ctx.disabled.map(|d| d()).unwrap_or(false);

    let global_error = ctx
        .errors_signal
        .with(|e| e.get("__global").cloned().flatten());

    rsx! {
        form {
            class: "{merged_class}",
            action: "javascript:void(0);",
            onsubmit: move |ev| {
                ev.prevent_default();
                if ctx.disabled.map(|d| d()).unwrap_or(false) {
                    return;
                }
                if let Some(cb) = &on_submit {
                    cb.call(ev);
                } else if let Some(mut submit) = ctx.submit {
                    (submit.write())();
                }
            },
            fieldset {
                disabled: is_disabled,
                class: "border-0 p-0 m-0 min-w-0 w-full",
                {children}
            }
            if let Some(err) = global_error {
                Alert { variant: AlertVariant::Destructive, class: "mt-4", "{err}" }
            }
        }
    }
}

#[component]
fn FormFieldWrapper(
    #[props(default)] class: String,
    #[props(default)] data_name: Option<String>,
    #[props(default)] data_invalid: Option<String>,
    children: Element,
) -> Element {
    let merged = merge(&[
        "group/field flex flex-col gap-1 w-full data-[invalid=true]:text-destructive",
        &class,
    ]);
    rsx! {
        div {
            class: "{merged}",
            "data-name": data_name.as_deref().unwrap_or("FormFieldWrapper"),
            "data-invalid": data_invalid,
            {children}
        }
    }
}

#[component]
pub fn FormField(field: Field, children: Element) -> Element {
    let field_name: Arc<str> = Arc::from(field.name);
    // Add to field register
    auto_register_field(&field);
    use_context_provider(|| FieldContext {
        name: field_name.clone(),
    });

    use_drop(move || {
        unregister_auto_field(&field_name);
    });

    let ctx = use_context::<FormContext>();

    let is_touched = ctx.touched_signal.with(|t| t.contains(field.name));

    let has_error = ctx
        .errors_signal
        .with(|e| e.get(field.name).is_some_and(|err| err.is_some()));
    let data_invalid = (is_touched && has_error).then_some("true".to_string());

    rsx! {
        FormFieldWrapper { data_name: "FormField".to_string(), data_invalid, {children} }
    }
}

#[component]
pub fn Input(
    #[props(into)] field: Field,
    r#type: InputType,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] format: Option<fn(&str) -> String>,
    #[props(default)] parse: Option<fn(&str) -> String>,
    #[props(default)] filter: Option<fn(&str) -> String>,
    #[props(default)] inputmode: String,
    #[props(default)] size: InputSize,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let has_actions = copyable || clearable;

    let input_class = InputSize::form_floating_peer_merge(size, has_actions);

    rsx! {
        FormField { field,
            div { class: "relative w-full mt-2",
                InputFormControl {
                    input_type: r#type,
                    size,
                    class: input_class,
                    autofocus,
                    format,
                    parse,
                    filter,
                    inputmode,
                }
                FormLabel { tooltip, "{field.label}" }
                if has_actions {
                    FieldActions { copyable, clearable }
                }
            }
            FormError {}
        }
    }
}

fn filter_numeric(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut has_dot = false;
    for (i, c) in input.chars().enumerate() {
        if c.is_ascii_digit() || (c == '-' && i == 0) || (c == '.' && !has_dot) {
            if c == '.' {
                has_dot = true;
            }
            result.push(c);
        }
    }
    // Strip leading zeros from the integer part.
    let negative = result.starts_with('-');
    let abs = if negative { &result[1..] } else { &result[..] };
    if abs.is_empty() {
        return result;
    }
    let stripped = match abs.find('.') {
        Some(dot_pos) => {
            let int_part = abs[..dot_pos].trim_start_matches('0');
            let int_part = if int_part.is_empty() { "0" } else { int_part };
            format!("{}{}", int_part, &abs[dot_pos..])
        }
        None => {
            let s = abs.trim_start_matches('0');
            if s.is_empty() {
                "0".to_string()
            } else {
                s.to_string()
            }
        }
    };
    if negative {
        format!("-{}", stripped)
    } else {
        stripped
    }
}

#[component]
pub fn NumberInput(
    #[props(into)] field: Field,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let fmt: Option<fn(&str) -> String> = Some(format_number);
    let prs: Option<fn(&str) -> String> = Some(parse_number);
    let flt: Option<fn(&str) -> String> = Some(filter_numeric);
    rsx! {
        Input {
            field,
            r#type: InputType::Text,
            copyable,
            clearable,
            autofocus,
            format: fmt,
            parse: prs,
            filter: flt,
            inputmode: "decimal".to_string(),
            tooltip,
        }
    }
}

#[component]
pub fn PercentageInput(
    #[props(into)] field: Field,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] autofocus: bool,
    #[props(default = 0.0)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let fmt: Option<fn(&str) -> String> = Some(format_percent);
    let prs: Option<fn(&str) -> String> = Some(parse_percent);
    let flt: Option<fn(&str) -> String> = Some(filter_percent);

    // Read `FormContext` once in the render path. It is `Copy`, so the blur
    // handler captures a copy rather than calling `use_context` (a hook) outside
    // render — which would panic. `field.name` is `&'static str`, so it is
    // captured directly (no per-render `to_string`).
    let form_ctx = use_context::<FormContext>();
    let field_name = field.name;
    let on_blur = move |_: FocusEvent| {
        let raw = form_ctx
            .values_signal
            .peek()
            .get(field_name)
            .cloned()
            .unwrap_or_default();
        let clamped = clamp_percent(&raw, min, max);
        if clamped != raw {
            form_ctx.set_value.read()(field_name, clamped);
        }
    };

    rsx! {
        div { onfocusout: on_blur,
            Input {
                field,
                r#type: InputType::Text,
                copyable,
                clearable,
                autofocus,
                format: fmt,
                parse: prs,
                filter: flt,
                inputmode: "decimal".to_string(),
                tooltip,
            }
        }
    }
}

#[component]
pub fn PhoneInput(
    #[props(into)] field: Field,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let fmt: Option<fn(&str) -> String> = Some(format_phone);
    let prs: Option<fn(&str) -> String> = Some(parse_phone);
    rsx! {
        Input {
            field,
            r#type: InputType::Tel,
            copyable,
            clearable,
            autofocus,
            format: fmt,
            parse: prs,
            tooltip,
        }
    }
}

#[component]
pub fn TextInput(
    #[props(into)] field: Field,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    rsx! {
        Input {
            field,
            r#type: InputType::Text,
            copyable,
            clearable,
            autofocus,
            tooltip,
        }
    }
}

/// Chromeless text field — no floating label, no border/background of its own.
/// The caller styles it via `class` (the control renders `unstyled`), so it can
/// sit inside a custom surface (an editable card cell, an inline row).
#[component]
pub fn BareTextInput(
    #[props(into)] field: Field,
    #[props(default)] class: String,
    #[props(default)] placeholder: String,
    #[props(default)] autofocus: bool,
) -> Element {
    rsx! {
        FormField { field,
            InputFormControl {
                input_type: InputType::Text,
                class,
                placeholder,
                autofocus,
                unstyled: true,
            }
        }
    }
}

#[component]
pub fn EmailInput(
    #[props(into)] field: Field,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    rsx! {
        Input {
            field,
            r#type: InputType::Email,
            copyable,
            clearable,
            autofocus,
            tooltip,
        }
    }
}

#[component]
pub fn PasswordInput(
    #[props(into)] field: Field,
    #[props(default)] autofocus: bool,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    rsx! {
        Input {
            field,
            r#type: InputType::Password,
            autofocus,
            tooltip,
        }
    }
}

/// Computes the new cursor position after reformatting an input value.
#[allow(dead_code)]
fn compute_cursor(
    old_formatted: &str,
    new_formatted: &str,
    old_cursor: u32,
    parse: fn(&str) -> String,
) -> u32 {
    let pos = old_cursor as usize;
    let before = &old_formatted[..pos.min(old_formatted.len())];
    let raw_pos = parse(before).len();

    let mut prev_parsed_len = 0usize;
    let mut matched_end: Option<usize> = None;
    for (i, ch) in new_formatted.char_indices() {
        let end = i + ch.len_utf8();
        let parsed_len = parse(&new_formatted[..end]).len();
        if parsed_len > raw_pos {
            return matched_end.unwrap_or(i) as u32;
        }
        if parsed_len == raw_pos && prev_parsed_len < raw_pos && matched_end.is_none() {
            matched_end = Some(end);
        }
        if matched_end.is_some() && parsed_len == raw_pos {
            matched_end = Some(end);
        }
        prev_parsed_len = parsed_len;
    }
    new_formatted.len() as u32
}

#[component]
pub(crate) fn InputFormControl(
    input_type: InputType,
    #[props(default)] size: InputSize,
    #[props(default)] class: String,
    #[props(default)] autofocus: bool,
    #[props(default = " ".to_string())] placeholder: String,
    #[props(default)] unstyled: bool,
    #[props(default)] format: Option<fn(&str) -> String>,
    #[props(default)] parse: Option<fn(&str) -> String>,
    #[props(default)] filter: Option<fn(&str) -> String>,
    #[props(default)] inputmode: String,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let field_name = field_ctx.name.clone();
    let form_ctx = use_context::<FormContext>();

    let input_class = if class.is_empty() {
        size.form_control_fallback_merge()
    } else {
        class
    };

    let id = String::from(&*field_name);
    let aria_describedby = format!("{}-error", field_name);
    let inputmode_opt = if inputmode.is_empty() {
        None
    } else {
        Some(inputmode.clone())
    };

    let is_disabled = form_ctx.disabled.map(|d| d()).unwrap_or(false);

    let is_touched = form_ctx.touched_signal.with(|t| t.contains(&*field_name));
    let has_error = form_ctx
        .errors_signal
        .with(|e| e.get(&*field_name).is_some_and(|err| err.is_some()));
    let aria_invalid = if is_touched && has_error {
        Some("true".to_string())
    } else {
        None
    };

    let raw = form_ctx
        .values_signal
        .with(|v| v.get(&*field_name).cloned().unwrap_or_default());
    let display_value = match format {
        Some(fmt) => fmt(&raw),
        None => raw,
    };

    let fn_clone = field_name.clone();
    rsx! {
        InputBase {
            r#type: input_type,
            size,
            id,
            placeholder,
            unstyled,
            class: input_class,
            disabled: is_disabled,
            autofocus,
            aria_describedby,
            aria_invalid,
            inputmode: inputmode_opt,
            static_value: Some(display_value),
            on_change: EventHandler::new({
                let field_name = fn_clone.clone();
                move |raw_input: String| {
                    let val = match filter {
                        Some(f) => f(&raw_input),
                        None => raw_input.clone(),
                    };
                    let raw = match parse {
                        Some(p) => p(&val),
                        None => val.clone(),
                    };
                    form_ctx.set_value.read()(&field_name, raw.clone());
                    // Note: cursor manipulation (set_selection_range) would need web_sys eval
                    // in Dioxus; for now we just update the value
                }
            }),
            onchange: EventHandler::new({
                let field_name = fn_clone.clone();
                move |ev: FormEvent| {
                    let raw_input = ev.value();
                    let val = match filter {
                        Some(f) => f(&raw_input),
                        None => raw_input,
                    };
                    let raw = match parse {
                        Some(p) => p(&val),
                        None => val,
                    };
                    form_ctx.set_value.read()(&field_name, raw);
                    form_ctx.touch_field.read()(&field_name);
                }
            }),
            onblur: EventHandler::new({
                let field_name = fn_clone.clone();
                move |_: FocusEvent| {
                    form_ctx.touch_field.read()(&field_name);
                }
            }),
        }
    }
}

#[component]
pub fn FormLabel(
    #[props(default)] class: String,
    #[props(default)] html_for: String,
    #[props(default)] textarea: bool,
    #[props(default)] tooltip: Option<Element>,
    children: Element,
) -> Element {
    let field_name = if html_for.is_empty() {
        try_use_context::<FieldContext>()
            .map(|ctx| String::from(&*ctx.name))
            .unwrap_or_default()
    } else {
        html_for
    };

    let placeholder_shown = if textarea {
        "peer-placeholder-shown:top-[15px] peer-placeholder-shown:translate-y-0 peer-placeholder-shown:scale-100 peer-placeholder-shown:font-normal"
    } else {
        "peer-placeholder-shown:top-1/2 peer-placeholder-shown:-translate-y-1/2 peer-placeholder-shown:scale-100 peer-placeholder-shown:font-normal"
    };

    let merged_class = merge(&[
        "absolute start-3 top-0 z-10 origin-[0] -translate-y-1/2 transform bg-[var(--field-notch-bg)] peer-[:placeholder-shown:not(:focus)]:bg-transparent px-1 text-sm font-medium text-muted-foreground duration-200 scale-75 pointer-events-none",
        placeholder_shown,
        "peer-focus:top-0 peer-focus:-translate-y-1/2 peer-focus:scale-75 peer-focus:font-medium peer-focus:text-primary",
        "peer-data-[invalid=true]:text-destructive peer-focus:peer-data-[invalid=true]:text-destructive",
        "group-data-[disabled=true]/field:opacity-50",
        &class,
    ]);

    rsx! {
        Label {
            data_name: "FormLabel".to_string(),
            class: merged_class,
            html_for: field_name,
            {children}
            if let Some(t) = tooltip {
                LabelHint { tooltip: t }
            }
        }
    }
}

#[component]
pub fn FormSeparator(
    #[props(default)] class: String,
    #[props(default)] children: Option<Element>,
) -> Element {
    let has_content = children.is_some();

    let merged_class = merge(&[
        "relative -my-2 h-5 text-sm group-data-[variant=outline]/field-group:-mb-2",
        &class,
    ]);

    rsx! {
        div {
            "data-name": "FormSeparator",
            "data-content": has_content.to_string(),
            class: "{merged_class}",
            Separator { class: "absolute inset-0 top-1/2" }
            if let Some(c) = children {
                span {
                    class: "block relative px-2 mx-auto bg-card text-muted-foreground w-fit",
                    "data-name": "FormSeparatorContent",
                    {c}
                }
            }
        }
    }
}

/// Small copy + clear action buttons positioned inside a field wrapper.
/// Reads the current value from `FormContext` via `FieldContext`.
#[component]
pub fn FieldActions(
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] class: String,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let field_name = field_ctx.name.clone();
    let form_ctx = use_context::<FormContext>();

    let has_value = form_ctx
        .values_signal
        .with(|v| v.get(&*field_name).is_some_and(|s| !s.is_empty()));

    let copied = use_signal(|| false);

    let container_class = merge(&[
        "absolute end-2 top-1/2 -translate-y-1/2 flex items-center gap-0.5 z-10",
        &class,
    ]);

    let container_style = if has_value { "" } else { "display:none" };

    let copy_icon = if copied() {
        IconName::CopyCheck
    } else {
        IconName::Copy
    };
    let copy_title = if copied() { "Copied!" } else { "Copy" };

    rsx! {
        div { class: "{container_class}", style: "{container_style}",
            if copyable {
                button {
                    r#type: "button",
                    tabindex: -1,
                    title: "{copy_title}",
                    class: "p-1.5 rounded-md text-muted-foreground/60 hover:text-foreground hover:bg-accent transition-colors cursor-pointer",
                    onclick: {
                        let field_name = field_name.clone();
                        move |_| {
                            let val = form_ctx
                                .values_signal
                                .peek()
                                .get(&*field_name)
                                .cloned()
                                .unwrap_or_default();
                            if !val.is_empty() {
                                copy_to_clipboard(val, copied);
                            }
                        }
                    },
                    Icon { name: copy_icon, class: "size-3.5" }
                }
            }
            if clearable {
                button {
                    r#type: "button",
                    tabindex: -1,
                    title: "Clear",
                    class: "p-1.5 rounded-md text-muted-foreground/60 hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer",
                    onclick: {
                        let field_name = field_name.clone();
                        move |_| {
                            form_ctx.set_value.read()(&field_name, String::new());
                        }
                    },
                    Icon { name: IconName::X, class: "size-3.5" }
                }
            }
        }
    }
}

#[component]
pub fn FormError(
    #[props(default)] class: String,
    #[props(default)] children: Option<Element>,
    #[props(default)] errors: Option<Vec<String>>,
) -> Element {
    if let Some(children) = children {
        return rsx! {
            Alert { variant: AlertVariant::Destructive, class, {children} }
        };
    }

    if let Some(errors) = errors {
        if errors.is_empty() {
            return rsx! {};
        }
        let first = errors.first().cloned().unwrap_or_default();
        let err_class = merge(&["text-destructive text-xs font-medium ml-1 mt-1", &class]);
        return rsx! {
            div { role: "alert", "aria-live": "polite", class: "{err_class}",
                span { "{first}" }
            }
        };
    }

    let field_ctx = try_use_context::<FieldContext>();
    let form_ctx = try_use_context::<FormContext>();

    if let (Some(field_ctx), Some(ctx)) = (field_ctx, form_ctx) {
        let field_name = field_ctx.name.clone();
        let is_touched = ctx.touched_signal.with(|t| t.contains(&*field_name));
        let error = ctx
            .errors_signal
            .with(|e| e.get(&*field_name).cloned().flatten());

        if !is_touched {
            return rsx! {};
        }
        if let Some(err) = error {
            let err_class = merge(&["text-destructive text-xs font-medium ml-1 mt-1", &class]);
            let err_id = format!("{}-error", &*field_name);
            return rsx! {
                div {
                    role: "alert",
                    "aria-live": "polite",
                    id: "{err_id}",
                    class: "{err_class}",
                    span { "{err}" }
                }
            };
        }
    }

    rsx! {}
}
