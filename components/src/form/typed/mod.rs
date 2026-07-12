//! Typed form store — `Signal<T>` as the source of truth, lens-based field
//! access, no serde anywhere in the read/write/submit path.
//!
//! Parallel to the legacy string-map form (`form::hook`); screens migrate
//! here incrementally and the legacy path is deleted once nothing uses it.

mod aux;
mod binding;
mod errors;
mod form;
mod lens;
mod value;
pub mod view;

pub use aux::{AuxState, OverlayEntry};
pub use binding::{BoundField, FieldBinding};
pub use form::{Form, GLOBAL_ERROR, Rows, TypedFormData, use_form};
pub use lens::{Compose, Index, Inner, Lens, LensExt};
pub use value::{FormValue, ParseError};

#[cfg(test)]
mod tests;
