//! The dynamic (string-map) form flavor.
//!
//! [`use_dynamic_form`] keeps a flat `HashMap<String, String>` keyed by
//! free-form dot-notation field names; values round-trip through serde at the
//! submit boundary (`T::json_schema()` drives coercion). The view components
//! here are the string-`Field`-prop family binding to that store — the typed
//! defaults live at the `form` root.

pub use super::hook::{
    DynamicForm, FormContext, FormData, SetValueFn, TouchFieldFn, use_dynamic_form,
};
pub use super::view::{
    BareTextInput, BareTextInputProps, EmailInput, FieldActions, FieldLabelProps, Form, FormError,
    FormField, FormProvider, Input, InputProps, MoneyInput, NumberInput, PasswordInput,
    PercentageInput, PercentageInputProps, PhoneInput, TextInput, TypedInputProps,
};
