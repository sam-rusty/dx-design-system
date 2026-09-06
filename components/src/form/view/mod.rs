#![allow(unpredictable_function_pointer_comparisons)]

mod binding;
mod components;
mod layout;
mod money;

pub use self::components::*;
pub use binding::{FieldBinding, use_field_binding};
pub(crate) use components::{ActionWiring, form_element, use_action_wiring};
pub use layout::*;
pub(crate) use money::MoneyControl;
pub use money::MoneyInput;

#[cfg(test)]
mod tests;
