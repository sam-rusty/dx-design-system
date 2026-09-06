//! Typed form store: `Signal<T>` is the single source of truth.
//!
//! No serde in any path — reads and writes go through fn-pointer lenses,
//! submit is a clone of `T`, validation runs `validator::Validate` on the
//! live struct. UI-only state (overlay/pristine/touched/errors) lives in
//! [`AuxState`] keyed by dot-notation paths.
//!
//! Everything that only needs a path (touched/error/overlay bookkeeping,
//! revalidation, submit-time required checks, server errors) lives on the
//! `T`-free [`FormCore`], so it compiles once for the whole app instead of
//! once per form struct and once more per field lens.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;
use ds_utils::DsError;
use ds_utils::format::snake_to_title;
use validator::{Validate, ValidationErrors};

use super::binding::{BoundField, FieldBinding, FieldOps, TypedFieldOps};
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

/// `T::validate` over the form's data signal with `T` erased.
pub(super) type ValidateFn = Box<dyn Fn() -> Option<ValidationErrors>>;

/// Submit-time metadata for a mounted field: whether it's required and how
/// to probe its emptiness without knowing its type.
#[derive(Clone)]
pub(super) struct RegisteredField {
    pub label: &'static str,
    pub required: bool,
    /// Emptiness probe (`FieldOps::is_empty`) of the registering binding.
    pub is_empty: Rc<dyn FieldOps>,
    /// The binding instance that registered this entry; `unregister` only
    /// removes an entry it owns, so a sibling that re-keyed into this path
    /// first (row swap/remove) keeps its registration.
    pub owner: Rc<()>,
}

pub struct Form<T: 'static> {
    pub data: Signal<T>,
    pub aux: Signal<AuxState>,
    pub(super) registry: Signal<HashMap<String, RegisteredField>>,
    /// Erased validator over `data` (see [`FormCore`]).
    pub(super) validate: CopyValue<ValidateFn>,
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

pub(super) fn erased_validator<T: TypedFormData>(data: Signal<T>) -> ValidateFn {
    Box::new(move || data.peek().validate().err())
}

pub fn use_form<T: TypedFormData>() -> Form<T> {
    let data = use_signal(T::default);
    Form {
        data,
        aux: use_signal(AuxState::default),
        registry: use_signal(HashMap::new),
        validate: use_hook(move || CopyValue::new(erased_validator(data))),
    }
}

/// The `T`-free half of a [`Form`]: aux state, field registry and the erased
/// validator. Copy, cheap, and shared by every closure a field binding hands
/// out.
#[derive(Clone, Copy)]
pub(super) struct FormCore {
    aux: Signal<AuxState>,
    registry: Signal<HashMap<String, RegisteredField>>,
    validate: CopyValue<ValidateFn>,
}

impl FormCore {
    pub(super) fn touch_field(&self, path: &str) {
        {
            let mut aux = self.aux;
            aux.write().touch(path);
        }
        self.revalidate_field(path);
    }

    pub(super) fn error(&self, path: &str) -> Option<String> {
        self.aux.read().error(path)
    }

    pub(super) fn is_touched(&self, path: &str) -> bool {
        self.aux.read().is_touched(path)
    }

    pub(super) fn global_error(&self) -> Option<String> {
        self.aux.read().error(GLOBAL_ERROR)
    }

    pub(super) fn clear_global_error(&self) {
        let mut aux = self.aux;
        if aux.peek().error(GLOBAL_ERROR).is_some() {
            aux.write().set_error(GLOBAL_ERROR, None);
        }
    }

    /// Populate form errors from a server error without touching values.
    pub(super) fn set_server_error(&self, error: DsError) {
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

    /// Empty-text write: back to pristine (overlay + written flag cleared).
    fn reset_pristine(&self, path: &str) {
        let needs_aux = {
            let a = self.aux.peek();
            a.overlay.contains_key(path) || a.is_written(path)
        };
        if needs_aux {
            let mut aux = self.aux;
            let mut a = aux.write();
            a.clear_overlay(path);
            a.unmark_written(path);
        }
    }

    /// Unparseable text: park it in the overlay with its parse error, unless
    /// that exact text is already parked.
    fn park_overlay(&self, path: &str, text: &str, message: String) {
        let stale = {
            let a = self.aux.peek();
            a.overlay.get(path).map(|e| e.text.as_str()) != Some(text) || !a.is_written(path)
        };
        if stale {
            let mut aux = self.aux;
            let mut a = aux.write();
            a.set_overlay(path, text.to_string(), message);
            a.mark_written(path);
        }
    }

    /// Display text decided by aux state alone (reactive read): overlay text
    /// while unparseable, blank while pristine. `None` means "render the
    /// typed value".
    fn display_override(&self, path: &str) -> Option<String> {
        let aux = self.aux.read();
        if let Some(entry) = aux.overlay.get(path) {
            return Some(entry.text.clone());
        }
        if !aux.is_written(path) {
            return Some(String::new());
        }
        None
    }

    fn clear_field(&self, path: &str) {
        {
            let mut aux = self.aux;
            aux.write().clear_field(path);
        }
        self.revalidate_if_touched(path);
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

        if let Some(msg) = (self.validate.read())()
            .and_then(|errs| field_validation_error(&errs, path, INDEX_STYLE))
        {
            return Some(msg);
        }

        let registry = self.registry.peek();
        if let Some(field) = registry.get(path)
            && field.required
            && field.is_empty.is_empty()
        {
            return Some(required_message(path, field.label));
        }

        None
    }

    /// Submit-time error set: overlay parse errors first, then `Validate`
    /// errors, then required-but-empty. Empty when the form is valid.
    fn collect_errors(&self) -> HashMap<String, String> {
        let mut new_errors: HashMap<String, String> = HashMap::new();

        for (path, entry) in &self.aux.peek().overlay {
            new_errors.insert(path.clone(), entry.message.clone());
        }

        if let Some(errs) = (self.validate.read())() {
            for (path, msg) in flatten_validation_errors(&errs, INDEX_STYLE) {
                new_errors.entry(path).or_insert(msg);
            }
        }

        for (path, field) in self.registry.peek().iter() {
            if field.required && field.is_empty.is_empty() {
                new_errors
                    .entry(path.clone())
                    .or_insert_with(|| required_message(path, field.label));
            }
        }

        new_errors
    }

    fn publish_errors(&self, new_errors: HashMap<String, String>) {
        let mut aux = self.aux;
        let mut a = aux.write();
        for (path, msg) in new_errors {
            a.touch(&path);
            a.set_error(&path, Some(msg));
        }
        a.set_error(GLOBAL_ERROR, Some("Validation Error".to_string()));
    }

    /// Register a mounted field for submit-time required checks.
    pub(super) fn register(
        &self,
        path: &str,
        label: &'static str,
        required: bool,
        is_empty: Rc<dyn FieldOps>,
        owner: Rc<()>,
    ) {
        let registry = self.registry;
        registry.write_unchecked().insert(
            path.to_string(),
            RegisteredField {
                label,
                required,
                is_empty,
                owner,
            },
        );
    }

    /// Remove a registration, but only one `owner` still holds (see
    /// `RegisteredField::owner`).
    pub(super) fn unregister(&self, path: &str, owner: &Rc<()>) {
        let registry = self.registry;
        let mut reg = registry.write_unchecked();
        if reg.get(path).is_some_and(|f| Rc::ptr_eq(&f.owner, owner)) {
            reg.remove(path);
        }
    }

    /// Assemble the erased handle for one field around its typed ops.
    fn bound_field(
        self,
        path: Rc<str>,
        label: &'static str,
        required: bool,
        form_scope: ScopeId,
        ops: Rc<dyn FieldOps>,
    ) -> BoundField {
        BoundField {
            path,
            label,
            required,
            form_scope,
            owner: Rc::new(()),
            core: self,
            ops,
        }
    }
}

impl<T: 'static> Form<T> {
    pub(super) fn core(&self) -> FormCore {
        FormCore {
            aux: self.aux,
            registry: self.registry,
            validate: self.validate,
        }
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
        let core = self.core();
        core.mark_written_guarded(&path);
        core.revalidate_if_touched(&path);
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
        let core = self.core();
        if text.is_empty() {
            core.reset_pristine(&path);
            if let Some(empty) = L::Value::empty() {
                let mut data = self.data;
                *lens.get_mut(&mut data.write()) = empty;
            }
            core.revalidate_if_touched(&path);
            return;
        }

        match L::Value::from_input(text) {
            Ok(value) => {
                {
                    let mut data = self.data;
                    *lens.get_mut(&mut data.write()) = value;
                }
                core.mark_written_guarded(&path);
            }
            Err(err) => core.park_overlay(&path, text, err.message),
        }
        core.revalidate_if_touched(&path);
    }

    /// Text to display in a control bound to `lens`: overlay text while
    /// unparseable, blank while pristine, else the typed value rendered.
    pub fn display<L>(&self, lens: L) -> String
    where
        L: Lens<T>,
        L::Value: FormValue,
    {
        if let Some(text) = self.core().display_override(&lens.path()) {
            return text;
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
        if let Some(empty) = L::Value::empty() {
            let mut data = self.data;
            *lens.get_mut(&mut data.write()) = empty;
        }
        self.core().clear_field(&lens.path());
    }

    pub fn touch_field(&self, path: &str) {
        self.core().touch_field(path);
    }

    pub fn error(&self, path: &str) -> Option<String> {
        self.core().error(path)
    }

    pub fn is_touched(&self, path: &str) -> bool {
        self.core().is_touched(path)
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
        aux.write().mark_all_written();
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
        let core = self.core();
        let new_errors = core.collect_errors();
        if new_errors.is_empty() {
            return Some(self.data.peek().clone());
        }
        core.publish_errors(new_errors);
        None
    }

    /// Clear any previous global error, validate, and submit valid data.
    pub fn submit(&self, on_submit: impl Fn(T) + 'static) {
        self.clear_global_error();
        if let Some(payload) = self.validate_and_get() {
            on_submit(payload);
        }
    }

    /// Populate form errors from a server error without touching values.
    pub fn set_server_error(&self, error: DsError) {
        self.core().set_server_error(error);
    }

    pub fn global_error(&self) -> Option<String> {
        self.core().global_error()
    }

    pub fn clear_global_error(&self) {
        self.core().clear_global_error();
    }
}

/// "`<Label>` is required", falling back to the title-cased leaf segment of
/// `path` when the lens carries no label.
fn required_message(path: &str, label: &str) -> String {
    if label.is_empty() {
        let leaf = path.rsplit('.').next().unwrap_or(path);
        format!("{} is required", snake_to_title(leaf))
    } else {
        format!("{label} is required")
    }
}

/// The typed side of a field: a lens over a form, behind the erased
/// [`FieldOps`] / [`TypedFieldOps`] vtables.
struct LensOps<T: 'static, L> {
    form: Form<T>,
    lens: L,
    path: Rc<str>,
}

impl<T, L> FieldOps for LensOps<T, L>
where
    T: TypedFormData,
    L: Lens<T>,
    L::Value: FormValue,
{
    fn display(&self) -> String {
        self.form.display(self.lens)
    }

    fn set_text(&self, text: &str) {
        self.form.set_text(self.lens, text);
    }

    fn clear(&self) {
        self.form.clear(self.lens);
    }

    /// Reactive twin of `is_empty`: subscribes the reader (e.g.
    /// `FieldActions`) to the aux + data signals.
    fn has_value(&self) -> bool {
        self.form.aux.read().is_written(&self.path)
            && self
                .lens
                .get(&self.form.data.read())
                .is_some_and(|v| !v.is_empty_value())
    }

    fn is_empty(&self) -> bool {
        if !self.form.aux.peek().is_written(&self.path) {
            return true;
        }
        self.lens
            .get(&self.form.data.peek())
            .map(FormValue::is_empty_value)
            .unwrap_or(true)
    }
}

impl<T, L> TypedFieldOps<L::Value> for LensOps<T, L>
where
    T: TypedFormData,
    L: Lens<T>,
    L::Value: FormValue,
{
    fn read(&self) -> Option<L::Value> {
        self.lens.get(&self.form.data.read()).cloned()
    }

    fn set(&self, value: L::Value) {
        self.form.set(self.lens, value);
    }
}

/// Field bindings
impl<T: TypedFormData> Form<T> {
    /// Erase `T` and the lens type into a per-field handle for components.
    /// One allocation and one vtable per `(T, lens)`; the path-keyed half
    /// lives on [`FormCore`].
    pub fn field<L>(&self, lens: L) -> FieldBinding<L::Value>
    where
        L: Lens<T>,
        L::Value: FormValue,
    {
        let path: Rc<str> = lens.path().into();
        let ops = Rc::new(LensOps {
            form: *self,
            lens,
            path: path.clone(),
        });
        let erased = self.core().bound_field(
            path,
            lens.label(),
            lens.required(),
            self.data.origin_scope(),
            ops.clone(),
        );
        FieldBinding { erased, ops }
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
        let index = {
            let mut data = self.form.data;
            let mut w = data.write();
            let vec = self.lens.get_mut(&mut w);
            vec.push(item);
            vec.len() - 1
        };
        let mut aux = self.form.aux;
        let mut a = aux.write();
        a.mark_written(&path);
        a.mark_row_pristine(&path, index);
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
        a.mark_row_pristine(&path, index);
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
