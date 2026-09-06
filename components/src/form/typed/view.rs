//! View layer for the typed form store.
//!
//! Thin: `FormField` puts a `Signal<BoundField>` in context and the existing
//! control family (`BoundInput`, `CheckboxControl`, `SelectControl`,
//! `TextAreaControl`) binds to it through `use_field_binding`'s typed path —
//! the controls themselves are shared with the legacy string-map form.

use std::sync::Arc;

use dioxus::prelude::*;
use ds_utils::format::{PhoneFormat, merge};

use super::binding::{BoundField, FieldHandle};
use super::form::{Form as FormStore, TypedFormData};
use crate::checkbox::CheckboxRow;
use crate::form::view::{ActionWiring, BoundInput, FormFieldWrapper, TypedKind, form_element};
use crate::form::{FieldContext, FieldLabel, FormSubmitProp, SubmitFn};
use crate::input::{FieldSize, InputType};
use crate::select::{SelectControl, SelectOption};
use crate::textarea::{TextAreaControl, TextAreaResize};

// Shared with the dynamic flavour: both read the field through
// `use_field_binding` / the field error snapshot.
pub use crate::form::view::{FieldActions, FieldActionsProps, FormError, FormErrorProps};

/// Form-level context for typed forms: submit wiring and the derived
/// disabled flag. Field-level state travels through `Signal<BoundField>`
/// provided by [`FormField`], not through this context.
#[derive(Clone, Copy)]
pub struct FormContext {
    /// Reactive disabled flag, derived from an explicit `loading` signal or
    /// the submit action's `pending()`.
    pub disabled: Option<Memo<bool>>,
    pub submit: Option<CopyValue<SubmitFn>>,
    /// Reads the form's global (non-field) error.
    pub global_error: CopyValue<Box<dyn Fn() -> Option<String>>>,
}

#[component]
pub fn FormProvider<T>(
    form: FormStore<T>,
    #[props(default, into)] action: FormSubmitProp<T>,
    #[props(default)] loading: Option<Signal<bool>>,
    #[props(default = true)] inline_error: bool,
    children: Element,
) -> Element
where
    T: TypedFormData + Send + Sync + 'static,
{
    let ActionWiring { disabled, submit } = crate::form::view::use_action_wiring(
        action.0,
        loading,
        inline_error,
        move |action| form.submit(move |data| action.call(data)),
        move |err| form.set_server_error(err),
    );

    let global_error = use_hook(move || {
        CopyValue::new(Box::new(move || form.global_error()) as Box<dyn Fn() -> Option<String>>)
    });

    // The store itself rides context too, so controls can resolve bare-lens
    // `field:` props (`MyForm::name`) without an explicit `form.field(...)`.
    use_context_provider(|| form);
    use_context_provider(|| FormContext {
        disabled,
        submit,
        global_error,
    });

    rsx! {
        {children}
    }
}

/// The `<form>` element for a typed form: submit wiring, disabled fieldset,
/// global error alert.
#[component]
pub fn Form(
    #[props(default)] class: String,
    #[props(default)] on_submit: Option<EventHandler<FormEvent>>,
    children: Element,
) -> Element {
    let ctx = use_context::<FormContext>();
    let global_error = (ctx.global_error.read())();
    form_element(
        ctx.disabled,
        ctx.submit,
        global_error,
        class,
        on_submit,
        children,
    )
}

/// Field scope for a typed form: provides the erased binding to descendant
/// controls, registers the field for submit-time required checks, and keeps
/// both in sync when the bound path changes (e.g. row re-indexing).
#[component]
pub fn FormField(#[props(into)] field: FieldHandle, children: Element) -> Element {
    let field = field.bind();
    let init = field.clone();
    let ctx_field: Signal<BoundField> = use_hook(move || {
        init.register();
        Signal::new(init)
    });
    use_context_provider(|| ctx_field);
    // Legacy field context so shared pieces (`FieldLabel`) resolve the name.
    use_context_provider(|| FieldContext {
        name: Arc::from(field.path()),
    });
    // Path changed under the same component instance (row re-key): swap the
    // context binding, move the registration (owner-aware, so a sibling that
    // already took this path keeps its entry) and refresh the name context.
    if *ctx_field.peek() != field {
        ctx_field.peek().unregister();
        field.register();
        *ctx_field.write_unchecked() = field.clone();
        provide_context(FieldContext {
            name: Arc::from(field.path()),
        });
    }
    use_drop(move || {
        ctx_field.peek().unregister();
    });

    let invalid = use_memo(move || ctx_field.read().invalid());
    let data_invalid = invalid().then_some("true".to_string());

    rsx! {
        FormFieldWrapper { data_name: "FormField".to_string(), data_invalid, {children} }
    }
}

/// Shared skeleton of a typed form-bound field: [`FormField`] context +
/// stacked [`FieldLabel`] + relative wrapper + control (children) + optional
/// [`FieldActions`] + [`FormError`].
#[component]
pub fn FormFieldFrame(
    #[props(into)] field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    label: Option<String>,
    #[props(default)] tooltip: Option<Element>,
    #[props(default)] copyable: bool,
    #[props(default)] clearable: bool,
    #[props(default)] class: String,
    #[props(default)] actions_class: String,
    children: Element,
) -> Element {
    let field = field.bind();
    let label = label.unwrap_or_else(|| field.label().to_string());
    let wrapper_class = merge(&["relative w-full", &class]);
    rsx! {
        FormField { field,
            FieldLabel { tooltip, "{label}" }
            div { class: "{wrapper_class}",
                {children}
                if copyable || clearable {
                    FieldActions { copyable, clearable, class: actions_class }
                }
            }
            FormError {}
        }
    }
}

/// Shared props for the typed form-bound inputs.
#[derive(Props, Clone, PartialEq)]
pub struct TypedInputProps {
    /// The bound form field: a bare lens (`MyForm::name`) or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    pub label: Option<String>,
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
    let actions_class = if is_password { "end-9" } else { "" };
    rsx! {
        FormFieldFrame {
            field: props.field,
            label: props.label,
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

/// Typed form-bound text input.
pub fn TextInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Text)
}

/// Typed form-bound email input.
pub fn EmailInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Email)
}

/// Props for [`PhoneInput`]: [`TypedInputProps`] plus the country format.
#[derive(Props, Clone, PartialEq)]
pub struct PhoneInputProps {
    /// The bound form field: a bare lens (`MyForm::name`) or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    pub label: Option<String>,
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

/// Typed form-bound phone input (formatted display, raw national digits
/// stored).
pub fn PhoneInput(props: PhoneInputProps) -> Element {
    let PhoneInputProps {
        field,
        label,
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
            label,
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

/// Typed form-bound numeric input (thousands-separated display, typed number
/// stored).
pub fn NumberInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Number)
}

/// Typed form-bound password input with reveal toggle.
pub fn PasswordInput(props: TypedInputProps) -> Element {
    typed_form_input(props, TypedKind::Password)
}

/// Props for [`Input`], the generic typed form-bound input with an explicit
/// HTML type.
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    /// The bound form field: a bare lens (`MyForm::name`) or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
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

/// Generic typed form-bound input for HTML types without a dedicated wrapper.
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
            label: None,
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

/// Props for [`PercentageInput`]: [`TypedInputProps`] plus clamp bounds.
#[derive(Props, Clone, PartialEq)]
pub struct PercentageInputProps {
    /// The bound form field: a bare lens (`MyForm::name`) or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
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

/// Typed form-bound percentage input (percent display, clamped).
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
            label: None,
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

/// Props for [`BareTextInput`].
#[derive(Props, Clone, PartialEq)]
pub struct BareTextInputProps {
    /// The bound form field: a bare lens (`MyForm::name`) or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
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

/// Chromeless typed text field — no label, no border/background of its own.
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

/// Props for [`TextArea`], the typed form-bound textarea.
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// The bound form field: a bare lens (`MyForm::name`) or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    pub label: Option<String>,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Visible rows.
    #[props(default)]
    pub rows: Option<u32>,
    /// Visible columns.
    #[props(default)]
    pub cols: Option<u32>,
    /// Minimum accepted length.
    #[props(default)]
    pub minlength: Option<u32>,
    /// Maximum accepted length.
    #[props(default)]
    pub maxlength: Option<u32>,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Resize handle behavior.
    #[props(default)]
    pub resize: TextAreaResize,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
}

/// Typed form-bound textarea with stacked label and inline error.
pub fn TextArea(props: TextAreaProps) -> Element {
    rsx! {
        FormFieldFrame {
            field: props.field,
            label: props.label,
            tooltip: props.tooltip,
            class: props.class,
            TextAreaControl {
                autofocus: props.autofocus,
                rows: props.rows,
                cols: props.cols,
                minlength: props.minlength,
                maxlength: props.maxlength,
                size: props.size,
                resize: props.resize,
            }
        }
    }
}

/// Props for [`Checkbox`], the typed form-bound checkbox row.
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    /// The bound form field, `bool`-typed: a bare lens or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
    /// Extra classes merged onto the row.
    #[props(default)]
    pub class: String,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
}

/// Typed form-bound checkbox with trailing label and inline error.
pub fn Checkbox(props: CheckboxProps) -> Element {
    let field = props.field.bind();
    let label = field.label().to_string();
    rsx! {
        FormField { field,
            CheckboxRow { class: props.class, label, tooltip: props.tooltip }
            FormError {}
        }
    }
}

/// Props for [`Select`], the typed form-bound select.
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    /// The bound form field: a bare lens or `form.field(...)`. Single-select
    /// enum or `Option<enum>` fields; the stored value uses the enum's serde
    /// name.
    #[props(into)]
    pub field: FieldHandle,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
    /// Show a search input inside the trigger.
    #[props(default)]
    pub searchable: bool,
    /// Cap on visible options (0 = unlimited).
    #[props(default)]
    pub limit: usize,
    /// Show the copy-to-clipboard action.
    #[props(default)]
    pub copyable: bool,
    /// Show the clear action.
    #[props(default)]
    pub clearable: bool,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// `(value, label)` pairs (e.g. a `FormOptions` derive's `OPTIONS`).
    #[props(default)]
    pub options: &'static [(&'static str, &'static str)],
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra `SelectOption` children rendered after `options`.
    #[props(default)]
    pub children: Option<Element>,
}

/// Typed form-bound single select with stacked label and inline error.
/// (Multi-select needs `Vec` text encoding and stays on the legacy form for
/// now.)
pub fn Select(props: SelectProps) -> Element {
    let field = props.field.bind();
    let field_label = field.label();
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame {
            field,
            tooltip: props.tooltip,
            copyable: props.copyable,
            clearable: props.clearable,
            class: props.class,
            actions_class: "end-9",
            SelectControl {
                searchable: props.searchable,
                limit: props.limit,
                size: props.size,
                open,
                aria_label: field_label.to_string(),
                for (value , opt_label) in props.options.iter() {
                    SelectOption {
                        key: "{value}",
                        value: value.to_string(),
                        label: opt_label.to_string(),
                    }
                }
                if let Some(c) = props.children {
                    {c}
                }
            }
        }
    }
}

/// Props for [`MoneyInput`], the typed form-bound money input.
#[derive(Props, Clone, PartialEq)]
pub struct MoneyInputProps {
    /// The bound form field, storing minor units: a bare lens or
    /// `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    pub label: Option<String>,
    /// Minor-unit exponent of the currency (2 → cents, 0 → zero-decimal).
    pub decimals: u32,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
}

/// Typed form-bound money input: major-unit display, minor-unit store.
pub fn MoneyInput(props: MoneyInputProps) -> Element {
    rsx! {
        FormFieldFrame {
            field: props.field,
            label: props.label,
            tooltip: props.tooltip,
            class: props.class,
            crate::form::view::MoneyControl {
                decimals: props.decimals,
                size: props.size,
                autofocus: props.autofocus,
            }
        }
    }
}

#[cfg(feature = "date-picker")]
pub use crate::date_picker::typed::{DatePicker, DateRangePicker, DateTimePicker};
