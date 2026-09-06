use std::sync::Arc;

use crate::{Alert, AlertVariant};
use dioxus::prelude::*;
use ds_utils::DsError;
use ds_utils::format::{PhoneFormat, merge};

use super::binding::field_error_state;
use super::use_field_binding;
use crate::copyable::copy_to_clipboard;
use crate::field_name::Field;
use crate::form::aux::AuxState;
use crate::form::hook::{FormContext, FormSubmitState, SetValueFn, TouchFieldFn};
use crate::form::{
    DynamicForm, FieldContext, FormData, FormSubmit, FormSubmitProp, GLOBAL_ERROR, SubmitFn,
};
use crate::icon::{Icon, IconName};
use crate::input::{FieldSize, InputBase, InputType};
use crate::input_types::{
    EmailInputBase, NumberInputBase, PasswordInputBase, PercentageInputBase,
    PercentageInputBaseProps, PhoneInputBase, PhoneInputBaseProps, TextInputBase,
    TypedInputBaseProps,
};
use crate::label::Label;
use crate::separator::Separator;
use crate::stepper::{auto_register_field, unregister_auto_field};
use crate::tooltip::Tooltip;
use std::collections::HashMap;

/// Help icon that reveals `tooltip` content on hover/focus, rendered inline
/// after a field label.
#[component]
pub(crate) fn LabelHint(tooltip: Element) -> Element {
    rsx! {
        Tooltip {
            title: tooltip,
            class: "align-middle",
            Icon {
                name: IconName::CircleHelp,
                class: "size-3.5 text-muted-foreground hover:text-foreground",
            }
        }
    }
}

/// Submit/disabled wiring shared by both `FormProvider`s.
pub(crate) struct ActionWiring {
    /// `Some` when an `action` or `loading` source exists.
    pub disabled: Option<Memo<bool>>,
    /// `Some` when an `action` exists.
    pub submit: Option<CopyValue<SubmitFn>>,
}

/// Derives the disabled flag and the submit closure from the provider props,
/// and feeds a failed action result into `on_server_error`.
///
/// Hooks run unconditionally (no `Option::map(use_hook)`), so a prop flipping
/// between `None` and `Some` across renders cannot shift hook indices. The
/// props they capture go through a compare-and-set signal so a `Some -> Some`
/// change of `action`/`loading` reaches the memo and closures; `FormContext`
/// itself is provided once, so a `None <-> Some` flip does not update
/// `ctx.disabled`/`ctx.submit` (pre-existing, no consumer relies on it).
/// Everything is owned by the provider scope (`CopyValue::new`), not
/// `ScopeId::ROOT`.
///
/// Deliberately non-generic (`T` is erased into `run_submit` by the caller,
/// see [`use_current_action`]) and never inlined: the hook bodies below are
/// the bulk of a provider's code and would otherwise be duplicated per form
/// type.
#[inline(never)]
pub(crate) fn use_action_wiring(
    action: Option<FormSubmitState>,
    loading: Option<Signal<bool>>,
    inline_error: bool,
    run_submit: Box<dyn Fn()>,
    on_server_error: Box<dyn Fn(DsError)>,
) -> ActionWiring {
    let mut sources = use_signal(|| (action, loading, inline_error));
    if *sources.peek() != (action, loading, inline_error) {
        sources.set((action, loading, inline_error));
    }

    let submit = use_hook(move || CopyValue::new(run_submit as SubmitFn));

    use_effect(move || {
        let (action, _, inline_error) = *sources.read();
        if inline_error && let Some(Some(Err(err))) = action.map(|a| a.result()) {
            on_server_error(err);
        }
    });

    // Precedence: a submit `action`'s `pending()` wins; otherwise the explicit
    // `loading` signal.
    let disabled = use_memo(move || {
        let (action, loading, _) = *sources.read();
        match (action, loading) {
            (Some(action), _) => action.pending(),
            (None, Some(sig)) => sig(),
            (None, None) => false,
        }
    });

    ActionWiring {
        disabled: (action.is_some() || loading.is_some()).then_some(disabled),
        submit: action.is_some().then_some(submit),
    }
}

/// Latest `action` prop, kept in a `CopyValue` so the erased submit closure
/// handed to [`use_action_wiring`] sees a `Some -> Some` prop change.
pub(crate) fn use_current_action<T: 'static>(
    action: Option<FormSubmit<T>>,
) -> CopyValue<Option<FormSubmit<T>>> {
    let mut current = use_hook(|| CopyValue::new(action));
    if *current.peek() != action {
        current.set(action);
    }
    current
}

/// Hook half of the dynamic `FormProvider`, `T`-free.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn provide_dynamic_form_context(
    values_signal: Signal<HashMap<String, String>>,
    aux: Signal<AuxState>,
    set_value: CopyValue<SetValueFn>,
    touch_field: CopyValue<TouchFieldFn>,
    action: Option<FormSubmitState>,
    loading: Option<Signal<bool>>,
    inline_error: bool,
    run_submit: Box<dyn Fn()>,
    on_server_error: Box<dyn Fn(DsError)>,
) {
    let ActionWiring { disabled, submit } =
        use_action_wiring(action, loading, inline_error, run_submit, on_server_error);

    use_context_provider(|| FormContext {
        values_signal,
        aux,
        set_value,
        touch_field,
        disabled,
        submit,
        step_field_registry: None,
    });
}

#[component]
pub fn FormProvider<T>(
    form: DynamicForm<T>,
    #[props(default, into)] action: FormSubmitProp<T>,
    #[props(default)] loading: Option<Signal<bool>>,
    #[props(default = true)] inline_error: bool,
    children: Element,
) -> Element
where
    T: FormData + Send + Sync + 'static,
{
    let set_value = use_hook(move || {
        CopyValue::new(Box::new(move |field: &str, value: String| {
            form.set_string_value(field, value);
        }) as SetValueFn)
    });

    let touch_field = use_hook(move || {
        CopyValue::new(Box::new(move |field: &str| {
            form.touch_field(field);
        }) as TouchFieldFn)
    });

    let current = use_current_action(action.0);
    provide_dynamic_form_context(
        form.values_signal,
        form.aux,
        set_value,
        touch_field,
        action.0.map(|a| a.state()),
        loading,
        inline_error,
        Box::new(move || {
            if let Some(action) = *current.peek() {
                form.submit(move |data| action.call(data));
            }
        }),
        Box::new(move |err| form.set_server_error(err)),
    );

    rsx! {
        {children}
    }
}

/// The `<form>` element shared by both flavours: submit wiring, disabled
/// fieldset, global error alert. No `action` attribute — `prevent_default`
/// handles submission and an inline `javascript:` URL trips CSP.
pub(crate) fn form_element(
    disabled: Option<Memo<bool>>,
    submit: Option<CopyValue<SubmitFn>>,
    global_error: Option<String>,
    class: String,
    on_submit: Option<EventHandler<FormEvent>>,
    children: Element,
) -> Element {
    let merged_class = merge(&["w-full", &class]);
    let is_disabled = disabled.map(|d| d()).unwrap_or(false);

    rsx! {
        form {
            class: "{merged_class}",
            onsubmit: move |ev| {
                ev.prevent_default();
                if disabled.map(|d| d()).unwrap_or(false) {
                    return;
                }
                if let Some(cb) = &on_submit {
                    cb.call(ev);
                } else if let Some(mut submit) = submit {
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
pub fn Form(
    #[props(default)] class: String,
    #[props(default)] on_submit: Option<EventHandler<FormEvent>>,
    children: Element,
) -> Element {
    let ctx = use_context::<FormContext>();
    let global_error = ctx.aux.with(|a| a.error(GLOBAL_ERROR));
    form_element(
        ctx.disabled,
        ctx.submit,
        global_error,
        class,
        on_submit,
        children,
    )
}

#[component]
pub(crate) fn FormFieldWrapper(
    #[props(default)] class: String,
    #[props(default)] data_name: Option<String>,
    #[props(default)] data_invalid: Option<String>,
    children: Element,
) -> Element {
    let merged = merge(&[
        "group/field flex flex-col gap-2 w-full data-[invalid=true]:text-destructive",
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
    // The `field` prop changed under the same instance (re-key): refresh the
    // context for descendants that read it per render and retarget the
    // drop-time step unregister. Controls memoise their binding by name, so a
    // re-keyed dynamic control needs a `key` to remount.
    let current = use_signal(|| field_name.clone());
    if *current.peek() != field_name {
        let old = std::mem::replace(&mut *current.write_unchecked(), field_name.clone());
        unregister_auto_field(&old);
        provide_context(FieldContext {
            name: field_name.clone(),
        });
    }

    use_drop(move || {
        unregister_auto_field(&current.peek());
    });

    let ctx = use_context::<FormContext>();

    let data_invalid = ctx
        .aux
        .with(|a| a.is_touched(field.name) && a.error(field.name).is_some())
        .then_some("true".to_string());

    rsx! {
        FormFieldWrapper { data_name: "FormField".to_string(), data_invalid, {children} }
    }
}

/// Props for [`FieldLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLabelProps {
    /// Extra classes merged into the label style.
    #[props(default)]
    pub class: String,
    /// Explicit label target; defaults to the surrounding field's name.
    #[props(default)]
    pub html_for: String,
    /// Help tooltip rendered inline after the label text.
    #[props(default)]
    pub tooltip: Option<Element>,
    pub children: Element,
}

/// The one field label for the form family: a static label stacked above the
/// control. Dims when the form is disabled; the error state is carried by the
/// control border + `FormError`, not the label color.
pub fn FieldLabel(props: FieldLabelProps) -> Element {
    // Read fresh each render: a re-keyed `FormField` re-provides the context.
    let field_ctx = try_consume_context::<FieldContext>();

    let field_name = if props.html_for.is_empty() {
        field_ctx
            .as_ref()
            .map(|ctx| String::from(&*ctx.name))
            .unwrap_or_default()
    } else {
        props.html_for.clone()
    };

    let merged_class = merge(&[
        "text-muted-foreground group-data-[disabled=true]/field:opacity-50",
        &props.class,
    ]);

    rsx! {
        Label {
            data_name: "FieldLabel".to_string(),
            class: merged_class,
            html_for: field_name,
            {props.children}
            if let Some(t) = props.tooltip {
                LabelHint { tooltip: t }
            }
        }
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
    /// Extra classes for the actions container (e.g. an end offset).
    #[props(default)]
    pub actions_class: String,
    pub children: Element,
}

/// The shared skeleton of every form-bound field: `FormField` context +
/// stacked [`FieldLabel`] + relative wrapper + control (children) + optional
/// [`FieldActions`] + [`FormError`].
pub(crate) fn FormFieldFrame(props: FormFieldFrameProps) -> Element {
    let label = props.field.label;
    let wrapper_class = merge(&["relative w-full", &props.class]);
    rsx! {
        FormField { field: props.field,
            FieldLabel { tooltip: props.tooltip, "{label}" }
            div { class: "{wrapper_class}",
                {props.children}
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
    Phone {
        format: PhoneFormat,
    },
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

    // Always a complete class list (the caller's own, or the form control
    // style), so the control renders `unstyled` and the standalone base is
    // never merged underneath (its conflicting border/focus classes would
    // fight the form styling).
    let input_class = if unstyled || !class.is_empty() {
        class
    } else {
        size.form_control_merge(reserve_end)
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
        // `input_class` is always complete — never merge the standalone base.
        unstyled: true,
        aria_invalid: binding.aria_invalid(),
        aria_describedby: binding.aria_describedby_when_invalid(),
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
        TypedKind::Phone { format } => {
            let props = PhoneInputBaseProps {
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
                format,
                attributes: base.attributes,
            };
            rsx! {
                PhoneInputBase { ..props }
            }
        }
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

/// Props for [`PhoneInput`]: [`TypedInputProps`] plus the country format.
#[derive(Props, Clone, PartialEq)]
pub struct PhoneInputProps {
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
    /// Display format, as a pattern whose `#` are national digits — e.g.
    /// `format: "+44 #### ######"`. Pass a list for a country with several
    /// number lengths (`format: &["+49 ### #######", "+49 ### ########"]`) and
    /// the tightest fitting one is used. Defaults to [`PhoneFormat::US`].
    #[props(default, into)]
    pub format: PhoneFormat,
}

/// Form-bound phone input (formatted display, raw national digits in the form
/// value).
pub fn PhoneInput(props: PhoneInputProps) -> Element {
    let PhoneInputProps {
        field,
        copyable,
        clearable,
        autofocus,
        size,
        tooltip,
        class,
        format,
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
        TypedKind::Phone { format },
    )
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

/// Chromeless text field — no label, no border/background of its own.
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

/// Copy + clear action buttons for the surrounding `FormField` (either
/// flavour, via [`use_field_binding`]). Hidden while the field is empty.
#[component]
pub fn FieldActions(
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] class: String,
) -> Element {
    let binding = use_field_binding();
    let has_value = (binding.has_value)();
    let value = binding.value;
    let clear = binding.clear;

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
                    onclick: move |_| {
                        let val = value.peek().clone();
                        if !val.is_empty() {
                            copy_to_clipboard(val, copied);
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
                    onclick: move |_| clear.call(()),
                    Icon { name: IconName::X, class: "size-3.5" }
                }
            }
        }
    }
}

/// Inline error for the surrounding `FormField` (either flavour), shown once
/// the field is touched; or an explicit message via `children` / `errors`.
/// `role="alert"` is assertive on its own — no `aria-live` override.
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
        let Some(first) = errors.first().cloned() else {
            return rsx! {};
        };
        let err_class = merge(&["text-destructive text-xs font-medium ml-1 mt-1", &class]);
        return rsx! {
            div { role: "alert", class: "{err_class}",
                span { "{first}" }
            }
        };
    }

    let Some(state) = field_error_state() else {
        return rsx! {};
    };
    if !state.touched {
        return rsx! {};
    }
    let Some(err) = state.error else {
        return rsx! {};
    };
    let err_class = merge(&["text-destructive text-xs font-medium ml-1 mt-1", &class]);
    let err_id = format!("{}-error", state.path);
    rsx! {
        div { role: "alert", id: "{err_id}", class: "{err_class}",
            span { "{err}" }
        }
    }
}
