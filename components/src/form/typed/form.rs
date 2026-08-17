//! Typed form store: `Signal<T>` is the single source of truth.
//!
//! No serde in any path — reads and writes go through fn-pointer lenses,
//! submit is a clone of `T`, validation runs `validator::Validate` on the
//! live struct. UI-only state (overlay/pristine/touched/errors) lives in
//! [`AuxState`] keyed by dot-notation paths.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;
use ds_utils::DsError;
use ds_utils::format::snake_to_title;
use validator::Validate;

use super::binding::{BoundField, FieldBinding};
use super::lens::Lens;
use super::value::FormValue;
use crate::form::aux::AuxState;
use crate::form::errors::{
    GLOBAL_ERROR, ListIndexStyle, field_validation_error, flatten_validation_errors,
};

/// Typed lens paths key list indices with dots (`items.2.qty`).
const INDEX_STYLE: ListIndexStyle = ListIndexStyle::Dots;

pub trait TypedFormData: Validate + Clone + Default + PartialEq + 'static {}
impl<T> TypedFormData for T where T: Validate + Clone + Default + PartialEq + 'static {}

/// Submit-time metadata for a mounted field: whether it's required and how
/// to probe its emptiness without knowing its type.
#[derive(Clone)]
pub(super) struct RegisteredField {
    pub label: &'static str,
    pub required: bool,
    pub is_empty: Rc<dyn Fn() -> bool>,
}

pub struct Form<T: 'static> {
    pub data: Signal<T>,
    pub aux: Signal<AuxState>,
    pub(super) registry: Signal<HashMap<String, RegisteredField>>,
}

impl<T> Clone for Form<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Form<T> {}

impl<T> PartialEq for Form<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.aux == other.aux
    }
}

pub fn use_form<T: TypedFormData>() -> Form<T> {
    Form {
        data: use_signal(T::default),
        aux: use_signal(AuxState::default),
        registry: use_signal(HashMap::new),
    }
}

/// Reads / writes
impl<T: TypedFormData> Form<T> {
    /// Current typed value under `lens` (reactive read).
    pub fn get<L: Lens<T>>(&self, lens: L) -> Option<L::Value>
    where
        L::Value: Clone,
    {
        lens.get(&self.data.read()).cloned()
    }

    /// Like [`Form::get`] but without registering a reactive dependency.
    pub fn get_untracked<L: Lens<T>>(&self, lens: L) -> Option<L::Value>
    where
        L::Value: Clone,
    {
        lens.get(&self.data.peek()).cloned()
    }

    /// Write a typed value; marks the field written and revalidates when
    /// touched.
    pub fn set<L: Lens<T>>(&self, lens: L, value: L::Value) {
        let path = lens.path();
        {
            let mut data = self.data;
            *lens.get_mut(&mut data.write()) = value;
        }
        self.mark_written_guarded(&path);
        self.revalidate_if_touched(&path);
    }

    /// Write raw input text. Non-empty text parses via [`FormValue`]: success
    /// writes the typed value, failure parks the text in the overlay with a
    /// parse error. Empty text resets the field to pristine (and clears the
    /// value where the type has an empty representation).
    pub fn set_text<L>(&self, lens: L, text: &str)
    where
        L: Lens<T>,
        L::Value: FormValue,
    {
        let path = lens.path();
        if text.is_empty() {
            let needs_aux = {
                let a = self.aux.peek();
                a.overlay.contains_key(&path) || a.is_written(&path)
            };
            if needs_aux {
                let mut aux = self.aux;
                let mut a = aux.write();
                a.clear_overlay(&path);
                a.unmark_written(&path);
            }
            if let Some(empty) = L::Value::empty() {
                let mut data = self.data;
                *lens.get_mut(&mut data.write()) = empty;
            }
            self.revalidate_if_touched(&path);
            return;
        }

        match L::Value::from_input(text) {
            Ok(value) => {
                {
                    let mut data = self.data;
                    *lens.get_mut(&mut data.write()) = value;
                }
                self.mark_written_guarded(&path);
                self.revalidate_if_touched(&path);
            }
            Err(err) => {
                let stale = {
                    let a = self.aux.peek();
                    a.overlay.get(&path).map(|e| e.text.as_str()) != Some(text)
                        || !a.is_written(&path)
                };
                if stale {
                    let mut aux = self.aux;
                    let mut a = aux.write();
                    a.set_overlay(&path, text.to_string(), err.message);
                    a.mark_written(&path);
                }
                self.revalidate_if_touched(&path);
            }
        }
    }

    /// Marks the field written / clears its overlay, skipping the aux signal
    /// write entirely when nothing would change (keeps steady-state typing
    /// from broadcasting to every field subscriber).
    fn mark_written_guarded(&self, path: &str) {
        let needs_aux = {
            let a = self.aux.peek();
            a.overlay.contains_key(path) || !a.is_written(path)
        };
        if needs_aux {
            let mut aux = self.aux;
            let mut a = aux.write();
            a.clear_overlay(path);
            a.mark_written(path);
        }
    }

    /// Text to display in a control bound to `lens`: overlay text while
    /// unparseable, blank while pristine, else the typed value rendered.
    pub fn display<L>(&self, lens: L) -> String
    where
        L: Lens<T>,
        L::Value: FormValue,
    {
        let path = lens.path();
        {
            let aux = self.aux.read();
            if let Some(entry) = aux.overlay.get(&path) {
                return entry.text.clone();
            }
            if !aux.is_written(&path) {
                return String::new();
            }
        }
        lens.get(&self.data.read())
            .map(FormValue::to_input)
            .unwrap_or_default()
    }

    /// Reset the field under `lens` to pristine.
    pub fn clear<L>(&self, lens: L)
    where
        L: Lens<T>,
        L::Value: FormValue,
    {
        let path = lens.path();
        if let Some(empty) = L::Value::empty() {
            let mut data = self.data;
            *lens.get_mut(&mut data.write()) = empty;
        }
        {
            let mut aux = self.aux;
            let mut a = aux.write();
            a.clear_field(&path);
        }
        self.revalidate_if_touched(&path);
    }

    pub fn touch_field(&self, path: &str) {
        {
            let mut aux = self.aux;
            aux.write().touch(path);
        }
        self.revalidate_field(path);
    }

    pub fn error(&self, path: &str) -> Option<String> {
        self.aux.read().error(path)
    }

    pub fn is_touched(&self, path: &str) -> bool {
        self.aux.read().is_touched(path)
    }

    /// Snapshot of the current struct without validating.
    pub fn get_data(&self) -> T {
        self.data.peek().clone()
    }

    /// Pre-fill the whole form: every field counts as written and displays
    /// its value from `values`.
    pub fn default_values(&self, values: T) {
        let mut data = self.data;
        *data.write() = values;
        let mut aux = self.aux;
        aux.write().all_written = true;
    }

    pub fn reset(&self) {
        let mut data = self.data;
        *data.write() = T::default();
        let mut aux = self.aux;
        *aux.write() = AuxState::default();
    }
}

/// Validation
impl<T: TypedFormData> Form<T> {
    /// Validate everything; on success return the struct for submission.
    /// Failures surface per-field errors (overlay parse errors first, then
    /// `Validate` errors, then required-but-empty) plus a global error.
    pub fn validate_and_get(&self) -> Option<T> {
        let mut new_errors: HashMap<String, String> = HashMap::new();

        for (path, entry) in &self.aux.peek().overlay {
            new_errors.insert(path.clone(), entry.message.clone());
        }

        let data = self.data.peek().clone();
        if let Err(errs) = data.validate() {
            for (path, msg) in flatten_validation_errors(&errs, INDEX_STYLE) {
                new_errors.entry(path).or_insert(msg);
            }
        }

        for (path, field) in self.registry.peek().iter() {
            if field.required && (field.is_empty)() {
                new_errors
                    .entry(path.clone())
                    .or_insert_with(|| format!("{} is required", field.label));
            }
        }

        if new_errors.is_empty() {
            return Some(data);
        }

        let mut aux = self.aux;
        let mut a = aux.write();
        for (path, msg) in new_errors {
            a.touch(&path);
            a.set_error(&path, Some(msg));
        }
        a.set_error(GLOBAL_ERROR, Some("Validation Error".to_string()));
        None
    }

    pub fn submit(&self, on_submit: impl Fn(T) + 'static) {
        if let Some(payload) = self.validate_and_get() {
            on_submit(payload);
        }
    }

    /// Populate form errors from a server error without touching values.
    pub fn set_server_error(&self, error: DsError) {
        match error {
            DsError::Validation(msg, errors) => {
                let flat = flatten_validation_errors(&errors, INDEX_STYLE);
                let mut aux = self.aux;
                let mut a = aux.write();
                a.set_error(GLOBAL_ERROR, Some(msg));
                for (path, msg) in flat {
                    a.touch(&path);
                    a.set_error(&path, Some(msg));
                }
            }
            other => {
                let mut aux = self.aux;
                aux.write().set_error(GLOBAL_ERROR, Some(other.to_string()));
            }
        }
    }

    pub fn global_error(&self) -> Option<String> {
        self.aux.read().error(GLOBAL_ERROR)
    }

    pub fn clear_global_error(&self) {
        let mut aux = self.aux;
        if aux.peek().error(GLOBAL_ERROR).is_some() {
            aux.write().set_error(GLOBAL_ERROR, None);
        }
    }

    fn revalidate_if_touched(&self, path: &str) {
        if self.aux.peek().is_touched(path) {
            self.revalidate_field(path);
        }
    }

    /// Recompute one field's error. Precedence: overlay parse error, then
    /// `Validate` message, then required-but-empty. Skips the aux write when
    /// the message is unchanged.
    fn revalidate_field(&self, path: &str) {
        let error = self.compute_field_error(path);
        if self.aux.peek().errors.get(path) == Some(&error) {
            return;
        }
        let mut aux = self.aux;
        aux.write().set_error(path, error);
    }

    fn compute_field_error(&self, path: &str) -> Option<String> {
        if let Some(entry) = self.aux.peek().overlay.get(path) {
            return Some(entry.message.clone());
        }

        if let Some(msg) = self
            .data
            .peek()
            .validate()
            .err()
            .and_then(|errs| field_validation_error(&errs, path, INDEX_STYLE))
        {
            return Some(msg);
        }

        let registry = self.registry.peek();
        if let Some(field) = registry.get(path)
            && field.required
            && (field.is_empty)()
        {
            let label = if field.label.is_empty() {
                let leaf = path.rsplit('.').next().unwrap_or(path);
                snake_to_title(leaf)
            } else {
                field.label.to_string()
            };
            return Some(format!("{label} is required"));
        }

        None
    }
}

/// Field bindings
impl<T: TypedFormData> Form<T> {
    /// Erase `T` and the lens type into a per-field handle for components.
    pub fn field<L>(&self, lens: L) -> FieldBinding<L::Value>
    where
        L: Lens<T>,
        L::Value: FormValue,
    {
        let form = *self;
        let path: Rc<str> = lens.path().into();

        let is_empty = {
            let path = path.clone();
            move || {
                if !form.aux.peek().is_written(&path) {
                    return true;
                }
                lens.get(&form.data.peek())
                    .map(FormValue::is_empty_value)
                    .unwrap_or(true)
            }
        };

        let erased = BoundField {
            label: lens.label(),
            required: lens.required(),
            form_scope: self.data.origin_scope(),
            display: Rc::new(move || form.display(lens)),
            set_text: Rc::new(move |text| form.set_text(lens, text)),
            touch: {
                let path = path.clone();
                Rc::new(move || form.touch_field(&path))
            },
            clear: Rc::new(move || form.clear(lens)),
            error: {
                let path = path.clone();
                Rc::new(move || form.error(&path))
            },
            is_touched: {
                let path = path.clone();
                Rc::new(move || form.is_touched(&path))
            },
            has_value: {
                let is_empty = is_empty.clone();
                Rc::new(move || !is_empty())
            },
            register: {
                let path = path.clone();
                let is_empty = is_empty.clone();
                let label = lens.label();
                let required = lens.required();
                Rc::new(move || {
                    let registry = form.registry;
                    registry.write_unchecked().insert(
                        path.to_string(),
                        RegisteredField {
                            label,
                            required,
                            is_empty: Rc::new(is_empty.clone()),
                        },
                    );
                })
            },
            unregister: {
                let path = path.clone();
                Rc::new(move || {
                    let registry = form.registry;
                    registry.write_unchecked().remove(&*path);
                })
            },
            path,
        };

        FieldBinding {
            erased,
            read: Rc::new(move || lens.get(&form.data.read()).cloned()),
            set: Rc::new(move |value| form.set(lens, value)),
        }
    }

    /// Typed row operations over a `Vec` field; the one mutation path that
    /// keeps aux-state keys aligned with row indices.
    pub fn rows<L, E>(&self, lens: L) -> Rows<T, L>
    where
        L: Lens<T, Value = Vec<E>>,
        E: 'static,
    {
        Rows { form: *self, lens }
    }
}

/// Typed handle for editing a `Vec` field's rows. All mutations re-key the
/// aux state under the array's path prefix.
#[derive(Clone, Copy)]
pub struct Rows<T: 'static, L> {
    form: Form<T>,
    lens: L,
}

impl<T, L, E> Rows<T, L>
where
    T: TypedFormData,
    L: Lens<T, Value = Vec<E>>,
    E: Default + 'static,
{
    /// Number of rows (reactive read).
    pub fn len(&self) -> usize {
        self.lens
            .get(&self.form.data.read())
            .map(Vec::len)
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&self, item: E) {
        let path = self.lens.path();
        {
            let mut data = self.form.data;
            self.lens.get_mut(&mut data.write()).push(item);
        }
        let mut aux = self.form.aux;
        aux.write().mark_written(&path);
    }

    pub fn insert(&self, index: usize, item: E) {
        let path = self.lens.path();
        let index = {
            let mut data = self.form.data;
            let mut w = data.write();
            let vec = self.lens.get_mut(&mut w);
            let index = index.min(vec.len());
            vec.insert(index, item);
            index
        };
        let mut aux = self.form.aux;
        let mut a = aux.write();
        a.insert_row(&path, index);
        a.mark_written(&path);
    }

    pub fn remove(&self, index: usize) {
        let path = self.lens.path();
        {
            let mut data = self.form.data;
            let mut w = data.write();
            let vec = self.lens.get_mut(&mut w);
            if index >= vec.len() {
                return;
            }
            vec.remove(index);
        }
        let mut aux = self.form.aux;
        aux.write().remove_row(&path, index);
    }

    pub fn swap(&self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let path = self.lens.path();
        {
            let mut data = self.form.data;
            let mut w = data.write();
            let vec = self.lens.get_mut(&mut w);
            if a >= vec.len() || b >= vec.len() {
                return;
            }
            vec.swap(a, b);
        }
        let mut aux = self.form.aux;
        aux.write().swap_rows(&path, a, b);
    }

    pub fn clear(&self) {
        let path = self.lens.path();
        {
            let mut data = self.form.data;
            self.lens.get_mut(&mut data.write()).clear();
        }
        let mut aux = self.form.aux;
        aux.write().clear_field(&path);
    }
}
