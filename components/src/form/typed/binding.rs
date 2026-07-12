//! Per-field handles handed to view components.
//!
//! [`FieldBinding<F>`] is the typed handle returned by `Form::field` — typed
//! `value()`/`set()` plus everything string-shaped. [`BoundField`] is its
//! fully erased core (no `T`, no `F`): every closure speaks strings/paths,
//! which is exactly what DOM-facing controls need. `FormField` provides
//! `BoundField` through context so the whole existing control family
//! (inputs, checkbox, select, textarea) binds to a typed form without
//! knowing it.
//!
//! The internals are deliberately boxed closures — a future dynamic
//! (runtime-schema) form can construct the same `BoundField`s over a
//! `Value`-map store and every component keeps working.

use std::rc::Rc;

use dioxus::prelude::ScopeId;

/// Fully erased handle for one form field. All reads marked "reactive"
/// subscribe the calling scope to the owning form's signals.
pub struct BoundField {
    pub(super) path: Rc<str>,
    pub(super) label: &'static str,
    pub(super) required: bool,
    /// Identity of the owning form (origin scope of its data signal), for
    /// prop memoization. Value changes propagate through signal reads, not
    /// prop inequality.
    pub(super) form_scope: ScopeId,
    pub(super) display: Rc<dyn Fn() -> String>,
    pub(super) set_text: Rc<dyn Fn(&str)>,
    pub(super) touch: Rc<dyn Fn()>,
    pub(super) clear: Rc<dyn Fn()>,
    pub(super) error: Rc<dyn Fn() -> Option<String>>,
    pub(super) is_touched: Rc<dyn Fn() -> bool>,
    pub(super) has_value: Rc<dyn Fn() -> bool>,
    pub(super) register: Rc<dyn Fn()>,
    pub(super) unregister: Rc<dyn Fn()>,
}

impl BoundField {
    /// Dot-notation path of the field (aux-state key, DOM id).
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn required(&self) -> bool {
        self.required
    }

    /// Text to display in the control (reactive read): overlay text while
    /// unparseable, blank while pristine, else the typed value rendered.
    pub fn display(&self) -> String {
        (self.display)()
    }

    /// Write raw input text; parses into the typed value or parks the text
    /// in the overlay with a parse error. Empty text resets to pristine.
    pub fn set_text(&self, text: &str) {
        (self.set_text)(text);
    }

    /// Write raw input text and mark the field touched (change/commit event).
    pub fn commit_text(&self, text: &str) {
        (self.set_text)(text);
        (self.touch)();
    }

    /// Mark the field touched (blur) and revalidate.
    pub fn touch(&self) {
        (self.touch)();
    }

    /// Reset the field to pristine (clears value where the type allows).
    pub fn clear(&self) {
        (self.clear)();
    }

    /// Current validation message (reactive read).
    pub fn error(&self) -> Option<String> {
        (self.error)()
    }

    /// Whether the field has been touched (reactive read).
    pub fn is_touched(&self) -> bool {
        (self.is_touched)()
    }

    /// Touched and failing validation (reactive read).
    pub fn invalid(&self) -> bool {
        self.is_touched() && self.error().is_some()
    }

    /// Whether the field holds a non-empty written value (reactive read).
    pub fn has_value(&self) -> bool {
        (self.has_value)()
    }

    /// Register this field's metadata (required + emptiness probe) with the
    /// form for submit-time checks. Called by `FormField` on mount.
    pub fn register(&self) {
        (self.register)();
    }

    /// Remove this field's registration. Called by `FormField` on drop.
    pub fn unregister(&self) {
        (self.unregister)();
    }
}

impl Clone for BoundField {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            label: self.label,
            required: self.required,
            form_scope: self.form_scope,
            display: self.display.clone(),
            set_text: self.set_text.clone(),
            touch: self.touch.clone(),
            clear: self.clear.clone(),
            error: self.error.clone(),
            is_touched: self.is_touched.clone(),
            has_value: self.has_value.clone(),
            register: self.register.clone(),
            unregister: self.unregister.clone(),
        }
    }
}

impl std::fmt::Debug for BoundField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoundField").field(&self.path).finish()
    }
}

impl PartialEq for BoundField {
    fn eq(&self, other: &Self) -> bool {
        self.form_scope == other.form_scope && self.path == other.path
    }
}

/// Typed handle for one form field: the erased [`BoundField`] plus typed
/// read/write. Cheap to clone; recreated per render at call sites
/// (`form.field(...)`), deduped in props via identity-based [`PartialEq`].
pub struct FieldBinding<F: 'static> {
    pub(super) erased: BoundField,
    pub(super) read: Rc<dyn Fn() -> Option<F>>,
    pub(super) set: Rc<dyn Fn(F)>,
}

impl<F> FieldBinding<F> {
    /// Current typed value (reactive read). `None` when the lens misses
    /// (unset `Option` segment / out-of-range index).
    pub fn value(&self) -> Option<F> {
        (self.read)()
    }

    /// Write a typed value.
    pub fn set(&self, value: F) {
        (self.set)(value);
    }

    /// The fully erased handle (what view components consume).
    pub fn erased(&self) -> &BoundField {
        &self.erased
    }
}

impl<F> std::ops::Deref for FieldBinding<F> {
    type Target = BoundField;

    fn deref(&self) -> &BoundField {
        &self.erased
    }
}

impl<F> From<FieldBinding<F>> for BoundField {
    fn from(binding: FieldBinding<F>) -> Self {
        binding.erased
    }
}

impl<F> Clone for FieldBinding<F> {
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            read: self.read.clone(),
            set: self.set.clone(),
        }
    }
}

impl<F> std::fmt::Debug for FieldBinding<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldBinding")
            .field(&self.erased.path)
            .finish()
    }
}

impl<F> PartialEq for FieldBinding<F> {
    fn eq(&self, other: &Self) -> bool {
        self.erased == other.erased
    }
}
