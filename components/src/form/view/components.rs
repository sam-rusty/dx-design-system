use std::sync::Arc;

use crate::{Alert, AlertVariant};
use dioxus::prelude::*;
use ds_utils::format::merge;

use super::use_field_binding;
use crate::copyable::copy_to_clipboard;
use crate::field_name::Field;
use crate::form::{FieldContext, Form as FormHook, FormContext, FormData, FormSubmit};
use crate::icon::{Icon, IconName};
use crate::input::{FieldSize, InputBase, InputType};
use crate::input_types::{
    EmailInputBase, NumberInputBase, PasswordInputBase, PercentageInputBase,
    PercentageInputBaseProps, PhoneInputBase, TextInputBase, TypedInputBaseProps,
};
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

/// Props for [`FloatingLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct FloatingLabelProps {
    /// Extra classes merged into the label style.
    #[props(default)]
    pub class: String,
    /// Explicit label target; defaults to the surrounding field's name.
    #[props(default)]
    pub html_for: String,
    /// Textarea positioning (top-anchored resting state) for the peer mechanism.
    #[props(default)]
    pub textarea: bool,
    /// Float trigger for button-based controls (open state). `None` uses the
    /// CSS `peer` mechanism (real inputs); `Some` floats when the field has a
    /// value or the signal reads true.
    #[props(default)]
    pub floated: Option<ReadSignal<bool>>,
    /// Help tooltip rendered inline after the label text.
    #[props(default)]
    pub tooltip: Option<Element>,
    pub children: Element,
}

/// The one floating label for the form family. Two mechanisms, one home for
/// the typography/positioning tokens: CSS `peer` for real inputs, signal-driven
/// for button-based controls (Select, date pickers).
pub fn FloatingLabel(props: FloatingLabelProps) -> Element {
    let field_ctx = try_use_context::<FieldContext>();
    let form_ctx = try_use_context::<FormContext>();

    let field_name = if props.html_for.is_empty() {
        field_ctx
            .as_ref()
            .map(|ctx| String::from(&*ctx.name))
            .unwrap_or_default()
    } else {
        props.html_for.clone()
    };

    let name_arc: Option<Arc<str>> = field_ctx.map(|ctx| ctx.name.clone());
    let has_value = {
        let name = name_arc.clone();
        use_memo(move || match (&name, &form_ctx) {
            (Some(n), Some(fc)) => fc
                .values_signal
                .with(|v| v.get(&**n).is_some_and(|s| !s.is_empty())),
            _ => false,
        })
    };
    let invalid = {
        let name = name_arc.clone();
        use_memo(move || match (&name, &form_ctx) {
            (Some(n), Some(fc)) => {
                fc.touched_signal.with(|t| t.contains(&**n))
                    && fc
                        .errors_signal
                        .with(|e| e.get(&**n).is_some_and(|err| err.is_some()))
            }
            _ => false,
        })
    };

    match props.floated {
        // Signal-driven: float when the field has a value or the control is open.
        Some(open) => {
            let is_floated = has_value() || open();
            let base = "absolute start-3 z-10 pointer-events-none bg-[var(--field-notch-bg)] \
                 px-1 text-muted-foreground transition-all duration-200 origin-[0] \
                 inline-flex items-center gap-1.5";
            let state = if is_floated {
                "top-0 -translate-y-1/2 scale-75 text-sm font-medium"
            } else {
                "top-1/2 -translate-y-1/2 scale-100 text-sm font-normal"
            };
            let accent = if invalid() {
                "text-destructive"
            } else if open() {
                "text-primary"
            } else {
                ""
            };
            let label_class = merge(&[base, state, accent, &props.class]);
            rsx! {
                label {
                    "data-name": "FloatingLabel",
                    class: "{label_class}",
                    r#for: "{field_name}",
                    {props.children}
                    if let Some(t) = props.tooltip {
                        LabelHint { tooltip: t }
                    }
                }
            }
        }
        // CSS peer: the input's placeholder/focus state drives the float.
        None => {
            let placeholder_shown = if props.textarea {
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
                &props.class,
            ]);

            rsx! {
                Label {
                    data_name: "FloatingLabel".to_string(),
                    class: merged_class,
                    html_for: field_name,
                    {props.children}
                    if let Some(t) = props.tooltip {
                        LabelHint { tooltip: t }
                    }
                }
            }
        }
    }
}

#[component]
#[deprecated(note = "use `FloatingLabel`")]
pub fn FormLabel(
    #[props(default)] class: String,
    #[props(default)] html_for: String,
    #[props(default)] textarea: bool,
    #[props(default)] tooltip: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        FloatingLabel { class, html_for, textarea, tooltip, {children} }
    }
}

/// Props for [`FormFieldFrame`].
#[derive(Props, Clone, PartialEq)]
pub(crate) struct FormFieldFrameProps {
    /// The bound form field (provides name + label).
    #[props(into)]
    pub field: Field,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Show the copy-to-clipboard action.
    #[props(default)]
    pub copyable: bool,
    /// Show the clear action.
    #[props(default)]
    pub clearable: bool,
    /// Extra classes merged onto the relative wrapper.
    #[props(default)]
    pub class: String,
    /// Textarea label positioning (peer mechanism only).
    #[props(default)]
    pub textarea: bool,
    /// Signal-driven label float trigger (button-based controls).
    #[props(default)]
    pub floated: Option<ReadSignal<bool>>,
    /// Extra classes for the actions container (e.g. an end offset).
    #[props(default)]
    pub actions_class: String,
    pub children: Element,
}

/// The shared skeleton of every form-bound field: `FormField` context +
/// relative wrapper + control (children) + [`FloatingLabel`] + optional
/// [`FieldActions`] + [`FormError`].
pub(crate) fn FormFieldFrame(props: FormFieldFrameProps) -> Element {
    let label = props.field.label;
    let wrapper_class = merge(&["relative w-full mt-2", &props.class]);
    rsx! {
        FormField { field: props.field,
            div { class: "{wrapper_class}",
                {props.children}
                FloatingLabel {
                    textarea: props.textarea,
                    floated: props.floated,
                    tooltip: props.tooltip,
                    "{label}"
                }
                if props.copyable || props.clearable {
                    FieldActions {
                        copyable: props.copyable,
                        clearable: props.clearable,
                        class: props.actions_class,
                    }
                }
            }
            FormError {}
        }
    }
}

/// Which typed base a [`BoundInput`] renders.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TypedKind {
    /// Raw [`InputBase`] with an explicit HTML type (url, search, hidden, ...).
    Custom(InputType),
    Text,
    Email,
    Phone,
    Number,
    Percentage {
        min: f64,
        max: f64,
    },
    Password,
}

/// The single form-context adapter for the whole input family: reads the
/// field binding and feeds the matching typed base as a controlled input.
#[component]
pub(crate) fn BoundInput(
    kind: TypedKind,
    #[props(default)] size: FieldSize,
    #[props(default)] autofocus: bool,
    /// Reserve end padding for a trailing adornment (actions / reveal button).
    #[props(default)]
    reserve_end: bool,
    #[props(default)] class: String,
    #[props(default)] unstyled: bool,
    #[props(default)] placeholder: Option<String>,
) -> Element {
    let binding = use_field_binding();

    let input_class = if unstyled {
        class
    } else if class.is_empty() {
        size.form_floating_peer_merge(reserve_end)
    } else {
        class
    };

    let touch = binding.touch;
    let base = TypedInputBaseProps {
        value: binding.controlled_value,
        default_value: String::new(),
        on_value_change: binding.on_value_change,
        on_commit: binding.on_commit,
        on_blur: Callback::new(move |_: FocusEvent| touch.call(())),
        on_key_down: Callback::default(),
        disabled: binding.disabled.into(),
        size,
        class: input_class,
        placeholder,
        id: Some(binding.id.clone()),
        autofocus,
        unstyled,
        aria_invalid: binding.aria_invalid(),
        aria_describedby: Some(binding.aria_describedby.clone()),
        attributes: Vec::new(),
    };

    match kind {
        TypedKind::Custom(t) => rsx! {
            InputBase {
                r#type: t,
                value: base.value,
                on_value_change: base.on_value_change,
                on_commit: base.on_commit,
                on_blur: base.on_blur,
                disabled: base.disabled,
                size: base.size,
                class: base.class,
                placeholder: base.placeholder,
                id: base.id,
                autofocus: base.autofocus,
                unstyled: base.unstyled,
                aria_invalid: base.aria_invalid,
                aria_describedby: base.aria_describedby,
            }
        },
        TypedKind::Text => rsx! {
            TextInputBase { ..base }
        },
        TypedKind::Email => rsx! {
            EmailInputBase { ..base }
        },
        TypedKind::Phone => rsx! {
            PhoneInputBase { ..base }
        },
        TypedKind::Number => rsx! {
            NumberInputBase { ..base }
        },
        TypedKind::Password => rsx! {
            PasswordInputBase { ..base }
        },
        TypedKind::Percentage { min, max } => {
            let props = PercentageInputBaseProps {
                value: base.value,
                default_value: base.default_value,
                on_value_change: base.on_value_change,
                on_commit: base.on_commit,
                on_blur: base.on_blur,
                on_key_down: base.on_key_down,
                disabled: base.disabled,
                size: base.size,
                class: base.class,
                placeholder: base.placeholder,
                id: base.id,
                autofocus: base.autofocus,
                unstyled: base.unstyled,
                aria_invalid: base.aria_invalid,
                aria_describedby: base.aria_describedby,
                min,
                max,
                attributes: base.attributes,
            };
            rsx! {
                PercentageInputBase { ..props }
            }
        }
    }
}

/// Shared props for the typed form-bound inputs ([`TextInput`], [`EmailInput`],
/// [`PhoneInput`], [`NumberInput`], [`PasswordInput`]).
#[derive(Props, Clone, PartialEq)]
pub struct TypedInputProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Show the copy-to-clipboard action.
    #[props(default)]
    pub copyable: bool,
    /// Show the clear action.
    #[props(default)]
    pub clearable: bool,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
}

fn typed_form_input(props: TypedInputProps, kind: TypedKind) -> Element {
    let is_password = matches!(kind, TypedKind::Password);
    let reserve_end = props.copyable || props.clearable || is_password;
    // The password reveal button occupies the end slot; shift actions inward.
    let actions_class = if is_password { "end-9" } else { "" };
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            copyable: props.copyable,
            clearable: props.clearable,
            class: props.class,
            actions_class,
            BoundInput {
                kind,
                size: props.size,
                autofocus: props.autofocus,
                reserve_end,
            }
        }
    }
}

/// Form-bound text input.
pub fn TextInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Text)
}

/// Form-bound email input.
pub fn EmailInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Email)
}

/// Form-bound phone input (formatted display, raw digits in the form value).
pub fn PhoneInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Phone)
}

/// Form-bound numeric input (thousands-separated display, raw decimal value).
pub fn NumberInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Number)
}

/// Form-bound password input with reveal toggle.
pub fn PasswordInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Password)
}

/// Props for [`PercentageInput`]: [`TypedInputProps`] plus clamp bounds.
#[derive(Props, Clone, PartialEq)]
pub struct PercentageInputProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Show the copy-to-clipboard action.
    #[props(default)]
    pub copyable: bool,
    /// Show the clear action.
    #[props(default)]
    pub clearable: bool,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
    /// Lower clamp bound applied on commit.
    #[props(default = 0.0)]
    pub min: f64,
    /// Upper clamp bound applied on commit.
    #[props(default = 100.0)]
    pub max: f64,
}

/// Form-bound percentage input (percent display, clamped into `[min, max]`).
pub fn PercentageInput(props: PercentageInputProps) -> Element {
    let PercentageInputProps {
        field,
        copyable,
        clearable,
        autofocus,
        size,
        tooltip,
        class,
        min,
        max,
    } = props;
    typed_form_input(
        TypedInputProps {
            field,
            copyable,
            clearable,
            autofocus,
            size,
            tooltip,
            class,
        },
        TypedKind::Percentage { min, max },
    )
}

/// Props for [`Input`], the generic form-bound input with an explicit HTML type.
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// HTML input type.
    pub r#type: InputType,
    /// Show the copy-to-clipboard action.
    #[props(default)]
    pub copyable: bool,
    /// Show the clear action.
    #[props(default)]
    pub clearable: bool,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
}

/// Generic form-bound input for HTML types without a dedicated wrapper
/// (url, search, hidden, ...). Prefer the typed wrappers where one exists.
pub fn Input(props: InputProps) -> Element {
    let InputProps {
        field,
        r#type,
        copyable,
        clearable,
        autofocus,
        size,
        tooltip,
        class,
    } = props;
    typed_form_input(
        TypedInputProps {
            field,
            copyable,
            clearable,
            autofocus,
            size,
            tooltip,
            class,
        },
        TypedKind::Custom(r#type),
    )
}

/// Props for [`BareTextInput`].
#[derive(Props, Clone, PartialEq)]
pub struct BareTextInputProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Full class list for the chromeless control.
    #[props(default)]
    pub class: String,
    /// Placeholder text.
    #[props(default)]
    pub placeholder: String,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
}

/// Chromeless text field — no floating label, no border/background of its own.
/// The caller styles it via `class` (the control renders `unstyled`), so it can
/// sit inside a custom surface (an editable card cell, an inline row).
pub fn BareTextInput(props: BareTextInputProps) -> Element {
    rsx! {
        FormField { field: props.field,
            BoundInput {
                kind: TypedKind::Text,
                class: props.class,
                unstyled: true,
                placeholder: props.placeholder,
                autofocus: props.autofocus,
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
