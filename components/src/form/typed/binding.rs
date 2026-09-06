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

use super::form::FormCore;

/// The typed side of one field (reads/writes through its lens), behind one
/// vtable per `(form type, lens)`. Everything else a control needs is keyed
/// by path on the `T`-free [`FormCore`], so it compiles once.
pub(super) trait FieldOps {
    fn display(&self) -> String;
    fn set_text(&self, text: &str);
    fn clear(&self);
    fn has_value(&self) -> bool;
    /// Untracked emptiness probe for the submit-time registry.
    fn is_empty(&self) -> bool;
}

/// [`FieldOps`] plus the typed accessors behind [`FieldBinding`].
pub(super) trait TypedFieldOps<F>: FieldOps {
    fn read(&self) -> Option<F>;
    fn set(&self, value: F);
}

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
    /// Registration owner token (see `RegisteredField::owner`).
    pub(super) owner: Rc<()>,
    pub(super) core: FormCore,
    pub(super) ops: Rc<dyn FieldOps>,
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
        self.ops.display()
    }

    /// Write raw input text; parses into the typed value or parks the text
    /// in the overlay with a parse error. Empty text resets to pristine.
    pub fn set_text(&self, text: &str) {
        self.ops.set_text(text);
    }

    /// Write raw input text and mark the field touched (change/commit event).
    pub fn commit_text(&self, text: &str) {
        self.ops.set_text(text);
        self.touch();
    }

    /// Mark the field touched (blur) and revalidate.
    pub fn touch(&self) {
        self.core.touch_field(&self.path);
    }

    /// Reset the field to pristine (clears value where the type allows).
    pub fn clear(&self) {
        self.ops.clear();
    }

    /// Current validation message (reactive read).
    pub fn error(&self) -> Option<String> {
        self.core.error(&self.path)
    }

    /// Whether the field has been touched (reactive read).
    pub fn is_touched(&self) -> bool {
        self.core.is_touched(&self.path)
    }

    /// Touched and failing validation (reactive read).
    pub fn invalid(&self) -> bool {
        self.is_touched() && self.error().is_some()
    }

    /// Whether the field holds a non-empty written value (reactive read).
    pub fn has_value(&self) -> bool {
        self.ops.has_value()
    }

    /// Register this field's metadata (required + emptiness probe) with the
    /// form for submit-time checks. Called by `FormField` on mount.
    pub fn register(&self) {
        self.core.register(
            &self.path,
            self.label,
            self.required,
            self.ops.clone(),
            self.owner.clone(),
        );
    }

    /// Remove this field's registration. Called by `FormField` on drop.
    pub fn unregister(&self) {
        self.core.unregister(&self.path, &self.owner);
    }
}

impl Clone for BoundField {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            label: self.label,
            required: self.required,
            form_scope: self.form_scope,
            owner: self.owner.clone(),
            core: self.core,
            ops: self.ops.clone(),
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
    pub(super) ops: Rc<dyn TypedFieldOps<F>>,
}

impl<F> FieldBinding<F> {
    /// Current typed value (reactive read). `None` when the lens misses
    /// (unset `Option` segment / out-of-range index).
    pub fn value(&self) -> Option<F> {
        self.ops.read()
    }

    /// Write a typed value.
    pub fn set(&self, value: F) {
        self.ops.set(value);
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
            ops: self.ops.clone(),
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

/// The `field:` prop type for typed controls: either an explicit binding
/// (`form.field(MyForm::name)` / a `BoundField`), or a bare lens
/// (`MyForm::name`) resolved lazily against the surrounding `FormProvider`'s
/// store — `FormProvider` puts its `Form<T>` in context, and the control
/// resolves in its own scope, below the provider.
pub struct FieldHandle {
    kind: HandleKind,
}

enum HandleKind {
    Bound(BoundField),
    Deferred {
        /// `TypeId` of the form struct `T` + lens path: prop identity for
        /// memoization (the binding itself is context-dependent).
        root: std::any::TypeId,
        path: String,
        resolve: Rc<dyn Fn() -> Option<BoundField>>,
    },
}

impl FieldHandle {
    /// Resolve to the erased binding. Bare lenses read the surrounding form
    /// store from context, so call this from a component mounted under the
    /// matching `FormProvider`.
    pub fn bind(&self) -> BoundField {
        match &self.kind {
            HandleKind::Bound(b) => b.clone(),
            HandleKind::Deferred { resolve, path, .. } => {
                let Some(bound) = resolve() else {
                    panic!("field {path:?}: bare lens used outside its FormProvider");
                };
                bound
            }
        }
    }
}

impl Clone for FieldHandle {
    fn clone(&self) -> Self {
        Self {
            kind: match &self.kind {
                HandleKind::Bound(b) => HandleKind::Bound(b.clone()),
                HandleKind::Deferred {
                    root,
                    path,
                    resolve,
                } => HandleKind::Deferred {
                    root: *root,
                    path: path.clone(),
                    resolve: resolve.clone(),
                },
            },
        }
    }
}

impl PartialEq for FieldHandle {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (HandleKind::Bound(a), HandleKind::Bound(b)) => a == b,
            (
                HandleKind::Deferred {
                    root: ra, path: pa, ..
                },
                HandleKind::Deferred {
                    root: rb, path: pb, ..
                },
            ) => ra == rb && pa == pb,
            _ => false,
        }
    }
}

impl std::fmt::Debug for FieldHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HandleKind::Bound(b) => f.debug_tuple("FieldHandle").field(&b.path).finish(),
            HandleKind::Deferred { path, .. } => {
                f.debug_tuple("FieldHandle::Deferred").field(path).finish()
            }
        }
    }
}

impl From<BoundField> for FieldHandle {
    fn from(bound: BoundField) -> Self {
        Self {
            kind: HandleKind::Bound(bound),
        }
    }
}

impl<F> From<FieldBinding<F>> for FieldHandle {
    fn from(binding: FieldBinding<F>) -> Self {
        Self {
            kind: HandleKind::Bound(binding.erased),
        }
    }
}

/// Bare lenses (`MyForm::name`, `MyForm::items`) become deferred handles:
/// the control resolves them against the `Form<T>` its `FormProvider` put in
/// context. Composed lenses (`.then(...)`, `.nth(...)`) don't carry their
/// root type in a nameable way — pass those as `form.field(lens)` instead.
impl<T, F> From<crate::field_name::FieldName<T, F>> for FieldHandle
where
    T: super::form::TypedFormData,
    F: super::value::FormValue,
{
    fn from(lens: crate::field_name::FieldName<T, F>) -> Self {
        use super::lens::Lens;
        Self {
            kind: HandleKind::Deferred {
                root: std::any::TypeId::of::<T>(),
                path: lens.path(),
                resolve: Rc::new(move || {
                    dioxus::prelude::try_consume_context::<super::form::Form<T>>()
                        .map(|form| form.field(lens).erased)
                }),
            },
        }
    }
}

impl<T, E> From<crate::field_name::FieldArray<T, E>> for FieldHandle
where
    T: super::form::TypedFormData,
    E: Clone + 'static,
{
    fn from(lens: crate::field_name::FieldArray<T, E>) -> Self {
        use super::lens::Lens;
        Self {
            kind: HandleKind::Deferred {
                root: std::any::TypeId::of::<T>(),
                path: lens.path(),
                resolve: Rc::new(move || {
                    dioxus::prelude::try_consume_context::<super::form::Form<T>>()
                        .map(|form| form.field(lens).erased)
                }),
            },
        }
    }
}
