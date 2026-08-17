//! Two form flavors over one shared core; the typed flavor is the default.
//!
//! - `use components::form::{use_form, Form, FormProvider, TextInput, ...}` —
//!   strict typed store (`Signal<T>`, lens field access, no serde). These are
//!   re-exports of [`typed`].
//! - `use components::form::dynamic::{...}` — the dynamic string-map flavor
//!   ([`dynamic::use_dynamic_form`]): free-form field names, serde coercion
//!   into `T` at the submit boundary.
//!
//! Shared internals: [`AuxState`] (touched/errors — the dynamic form leaves
//! overlay/pristine unused), validation-error flattening (`errors`), and the
//! low-level control binding ([`use_field_binding`]), which speaks both
//! flavors.

mod aux;
pub mod dynamic;
mod errors;
mod form_utils;
mod hook;
pub mod typed;
pub(crate) mod view;

pub use aux::{AuxState, OverlayEntry};
pub use errors::GLOBAL_ERROR;
pub use hook::{
    DynamicForm, FieldContext, FormData, FormSubmit, SubmitFn, captured_app_error, use_dynamic_form,
};
pub use typed::view::*;
pub use typed::{BoundField, FieldHandle, Form, Lens, LensExt, TypedFormData, use_form};
pub use view::{
    FieldBinding, FieldLabel, FormContent, FormDescription, FormGroup, FormSeparator, FormSet,
    FormTitle, use_field_binding,
};
