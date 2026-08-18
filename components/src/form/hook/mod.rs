//! Dynamic form store: a flat `HashMap<String, String>` keyed by free-form
//! dot-notation field names. Values round-trip through serde at the submit
//! boundary (`T::json_schema()` drives coercion); touched/error state lives
//! in the shared [`AuxState`] (overlay/pristine stay unused — value-map
//! emptiness plays the pristine role here).

mod action;
use std::collections::HashMap;
use std::sync::Arc;

pub use action::{FormSubmit, SubmitFn, captured_app_error};
use dioxus::prelude::*;
use ds_utils::DsError;
use ds_utils::format::snake_to_title;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use validator::{Validate, ValidationError, ValidationErrors};

use super::aux::AuxState;
use super::errors::GLOBAL_ERROR;
use super::form_utils::*;
use crate::field_name::{Field, FormSchema};
use crate::{FieldKey, FieldType};

pub trait FormData:
    Validate + Clone + Default + Serialize + for<'de> Deserialize<'de> + FormSchema + 'static
{
}
impl<T> FormData for T where
    T: Validate + Clone + Default + Serialize + for<'de> Deserialize<'de> + FormSchema + 'static
{
}

pub type SetValueFn = Box<dyn Fn(&str, String)>;
pub type TouchFieldFn = Box<dyn Fn(&str)>;

pub struct DynamicForm<T> {
    pub values_signal: Signal<HashMap<String, String>>,
    /// Shared per-field UI state (touched + errors; overlay/pristine unused).
    pub aux: Signal<AuxState>,
    pub default_schema: Signal<Arc<Value>>,
    pub required_fields: Signal<HashMap<String, bool>>,
    _phantom: std::marker::PhantomData<T>,
}

pub trait ToggleDefault: Serialize + Sized {
    fn on_value() -> Self;
    fn off_value() -> Self;
}

impl<T: Serialize + Default> ToggleDefault for Option<T> {
    fn on_value() -> Self {
        Some(T::default())
    }

    fn off_value() -> Self {
        None
    }
}

// Boilerplate trait implementations
impl<T> Clone for DynamicForm<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DynamicForm<T> {}

impl<T> PartialEq for DynamicForm<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values_signal == other.values_signal
            && self.aux == other.aux
            && self.required_fields == other.required_fields
    }
}

impl<T> Default for DynamicForm<T>
where
    T: FormData,
{
    fn default() -> Self {
        let schema = T::json_schema();
        Self {
            values_signal: Signal::new_in_scope(Default::default(), ScopeId::ROOT),
            aux: Signal::new_in_scope(Default::default(), ScopeId::ROOT),
            default_schema: Signal::new_in_scope(Arc::new(schema), ScopeId::ROOT),
            required_fields: Signal::new_in_scope(Default::default(), ScopeId::ROOT),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub fn use_dynamic_form<T: FormData>() -> DynamicForm<T> {
    DynamicForm {
        values_signal: use_signal(Default::default),
        aux: use_signal(Default::default),
        default_schema: use_signal(|| Arc::new(T::json_schema())),
        required_fields: use_signal(Default::default),
        _phantom: std::marker::PhantomData,
    }
}

impl<T: FormData> DynamicForm<T> {
    pub fn error(&self, field: &str) -> Option<String> {
        self.aux.with(|a| a.error(field))
    }

    pub fn is_touched(&self, field: &str) -> bool {
        self.aux.with(|a| a.is_touched(field))
    }

    pub fn set<D: Serialize>(&self, field: &str, value: D) {
        let value = match serde_json::to_value(value) {
            // The backing map stores the text a native input would emit. Keep
            // strings unquoted so enum values can round-trip through `get`.
            Ok(Value::String(s)) => s,
            Ok(Value::Null) => String::new(),
            Ok(value) => value.to_string(),
            Err(e) => {
                debug_assert!(
                    false,
                    "Form::set: serialization failed for field {field:?}: {e}"
                );
                eprintln!(
                    "Form::set serialization failed for field {field:?}: {e}; storing empty string"
                );
                String::new()
            }
        };
        let mut s = self.values_signal;
        s.write().insert(field.to_string(), value);
        if self.is_touched(field) {
            self.trigger_field_validation(field);
        }
    }

    /// set string value, without serialization
    pub fn set_string_value(&self, field: &str, value: String) {
        let mut s = self.values_signal;
        s.write().insert(field.to_string(), value);
        if self.is_touched(field) {
            self.trigger_field_validation(field);
        }
    }

    pub fn get<K: FieldKey<T>>(&self, field: K) -> Option<K::Value> {
        let raw = self.values_signal.with(|v| v.get(field.key()).cloned())?;
        if raw.is_empty() {
            return None;
        }
        let coerced = coerce_value(&raw, field.field_type())?;
        serde_json::from_value(coerced).ok()
    }

    /// Like `get`, but reads without registering a reactive dependency.
    /// Use in event handlers or other places where no reactive tracking context is active.
    pub fn get_untracked<K: FieldKey<T>>(&self, field: K) -> Option<K::Value> {
        let raw = self.values_signal.peek().get(field.key()).cloned()?;
        if raw.is_empty() {
            return None;
        }
        let coerced = coerce_value(&raw, field.field_type())?;
        serde_json::from_value(coerced).ok()
    }

    pub fn get_or<K: FieldKey<T>>(&self, field: K, fallback: K::Value) -> K::Value {
        self.get(field).unwrap_or(fallback)
    }

    pub fn has_value<K: FieldKey<T>>(&self, field: K) -> bool {
        let key = field.key();
        match field.field_type() {
            FieldType::Object | FieldType::Array => {
                let prefix = format!("{key}.");
                self.values_signal.with(|v| {
                    v.iter()
                        .any(|(k, val)| k.starts_with(&prefix) && !val.is_empty())
                })
            }
            _ => self
                .values_signal
                .with(|v| v.get(key).is_some_and(|val| !val.is_empty())),
        }
    }

    pub fn toggle_optional<K>(&self, enabled: Signal<bool>, field: K) -> impl Fn() + 'static
    where
        T: Send + Sync,
        K: Into<Field> + FieldKey<T> + 'static,
        K::Value: ToggleDefault + Send + Sync + 'static,
    {
        let form = *self;
        let name = field.into().name;
        move || {
            let new_val = !*enabled.peek();
            let mut e = enabled;
            *e.write() = new_val;
            if new_val {
                form.set(name, K::Value::on_value());
            } else {
                form.set(name, K::Value::off_value());
                let prefix = format!("{name}.");
                let mut vs = form.values_signal;
                vs.write().retain(|k, _| !k.starts_with(&prefix));
            }
        }
    }

    pub fn touch_field(&self, field: &str) {
        let mut aux = self.aux;
        aux.write().touch(field);
        self.trigger_field_validation(field);
    }

    pub fn reset(&self) {
        let mut vs = self.values_signal;
        let mut aux = self.aux;
        *vs.write() = Default::default();
        *aux.write() = AuxState::default();
    }

    pub fn default_values<F: Serialize>(&self, default_values: F) {
        if let Ok(val) = serde_json::to_value(default_values) {
            let mut map = HashMap::new();
            flatten_json_value(&val, "", &mut map);
            let mut vs = self.values_signal;
            *vs.write() = map;
        }
    }
}

/// Validation
impl<T: FormData> DynamicForm<T> {
    pub fn get_data(&self) -> Option<T> {
        let schema = self.default_schema.peek().clone();
        self.values_signal
            .with(|v| parse_form_data::<T>(v, &schema))
            .ok()
    }

    pub fn validate_and_get(&self) -> Option<T> {
        let schema = self.default_schema.peek().clone();
        let data = match self
            .values_signal
            .with(|v| parse_form_data::<T>(v, &schema))
        {
            Ok(d) => d,
            Err(parse_err) => {
                self.set_global_error(&format!("Invalid data format: {}", parse_err));
                return None;
            }
        };

        match data.validate() {
            Ok(()) => Some(data),
            Err(errors) => {
                self.set_global_error("Validation Error");
                self.add_validation_errors(errors);
                None
            }
        }
    }

    /// Validate set of fields
    pub fn validate_fields(&self, fields: &[Field]) -> bool {
        // Touch the fields manually, otherwise the errors won't return.
        // Also register required field signals so trigger_field_validation can use them later.
        {
            let mut aux = self.aux;
            let mut a = aux.write();
            for field in fields {
                a.touch(field.name);
            }
        }
        {
            let mut rf = self.required_fields;
            let mut r = rf.write();
            for field in fields {
                r.insert(field.name.to_string(), field.required);
            }
        }

        let default_schema = self.default_schema.peek().clone();

        // for any field that is none Option, must have a value, empty string it not allowed.
        let mut required_empty: HashMap<String, String> = HashMap::new();
        self.values_signal.with(|values| {
            for field in fields {
                let is_required = field.required;
                if is_required && is_field_empty(values, field.name) {
                    required_empty.insert(
                        field.name.to_string(),
                        format!("{} is required", field.label),
                    );
                }
            }
        });

        // continue getting the data and validate other fields that has values
        let data = match self
            .values_signal
            .with(|v| parse_form_data::<T>(v, &default_schema))
        {
            Ok(d) => d,
            Err(_) => {
                // Only set errors for fields we know are required-but-empty.
                // Do NOT fallback to "Invalid value" — the parse failure may come
                // from a different step's struct entirely (e.g. missing #[serde(default)]).
                let has_errors = !required_empty.is_empty();
                let mut aux = self.aux;
                let mut a = aux.write();
                for field in fields {
                    let error = required_empty.remove(field.name);
                    a.set_error(field.name, error);
                }
                return !has_errors;
            }
        };

        let all_errors = match data.validate() {
            Ok(()) => HashMap::new(),
            Err(errs) => flatten_validation_errors(&errs),
        };

        let mut has_errors = !required_empty.is_empty();
        {
            let mut aux = self.aux;
            let mut a = aux.write();
            for field in fields {
                let error = all_errors
                    .get(field.name)
                    .cloned()
                    .or_else(|| required_empty.get(field.name).cloned());

                if !has_errors && error.is_some() {
                    has_errors = true;
                }

                a.set_error(field.name, error);
            }
        }

        !has_errors
    }

    /// Updates the field's error slot: `Validate` message for this field if any, else
    /// required-but-empty (same precedence as the Leptos form hook).
    fn trigger_field_validation(&self, field: &str) {
        let is_required = self
            .required_fields
            .with(|r| r.get(field).copied().unwrap_or(false));
        let empty_required = self.values_signal.with(|values| {
            if is_required && is_field_empty(values, field) {
                let label = field.rsplit('.').next().unwrap_or(field);
                Some(format!("{} is required", snake_to_title(label)))
            } else {
                None
            }
        });

        let error_msg = self
            .get_data()
            .and_then(|data| data.validate().err())
            .and_then(|errs| field_validation_error(&errs, field))
            .or(empty_required);

        let mut aux = self.aux;
        aux.write().set_error(field, error_msg);
    }

    fn add_validation_errors(&self, errors: ValidationErrors) {
        let flat_errors = flatten_validation_errors(&errors);

        let mut aux = self.aux;
        let mut a = aux.write();
        for (field, msg) in flat_errors {
            a.touch(&field);
            a.set_error(&field, Some(msg));
        }
    }

    /// global error is error that doesn't belong to a specific field.
    fn set_global_error(&self, msg: &str) {
        let mut errors = ValidationErrors::new();
        errors.add(
            GLOBAL_ERROR,
            ValidationError::new("error").with_message(msg.to_string().into()),
        );
        self.add_validation_errors(errors);
    }
}

// Form submission and response
impl<T: FormData> DynamicForm<T> {
    /// Clear any previous global error, validate, and submit valid data.
    pub fn submit(&self, on_submit: impl Fn(T) + 'static) {
        self.clear_global_error();
        if let Some(payload) = self.validate_and_get() {
            on_submit(payload);
        }
    }

    /// Populate form errors from a server `AppError` without touching values/reset.
    /// `Validation` errors flow to per-field slots plus `__global`; everything else
    /// surfaces in the `__global` slot only.
    pub fn set_server_error(&self, error: DsError) {
        match error {
            DsError::Validation(msg, errors) => {
                self.set_global_error(&msg);
                self.add_validation_errors(errors);
            }
            other => self.set_global_error(&other.to_string()),
        }
    }

    pub fn clear_global_error(&self) {
        let mut aux = self.aux;
        if aux.peek().error(GLOBAL_ERROR).is_some() {
            aux.write().set_error(GLOBAL_ERROR, None);
        }
    }
}

// Context Structs
#[derive(Clone, Copy)]
pub struct FormContext {
    pub values_signal: Signal<HashMap<String, String>>,
    /// Shared per-field UI state (touched + errors).
    pub aux: Signal<AuxState>,
    pub set_value: CopyValue<SetValueFn>,
    pub touch_field: CopyValue<TouchFieldFn>,
    /// Reactive disabled flag, derived via `use_memo` from either an explicit
    /// `loading` signal or the submit action's `pending()` — no `use_effect`
    /// prop→signal mirroring (aligns with `use_controlled` semantics).
    pub disabled: Option<Memo<bool>>,
    pub submit: Option<CopyValue<SubmitFn>>,
    /// When set (e.g. by [`crate::stepper::Step`]), `FormField` registers into this signal for
    /// step validation. Slotted step bodies share the outer `FormContext` scope; this field threads
    /// the registry without relying on a separate context type that slotted `Element`s may not see.
    pub step_field_registry: Option<Signal<Vec<Field>>>,
}

#[derive(Clone, PartialEq)]
pub struct FieldContext {
    pub name: Arc<str>,
}

#[cfg(test)]
mod tests;
