#[cfg(any(
    feature = "form-fields",
    feature = "form-options",
    feature = "on-server",
    feature = "on-web",
    feature = "steps"
))]
use proc_macro::TokenStream;

#[cfg(any(feature = "form-fields", feature = "form-options", feature = "steps"))]
mod case;
#[cfg(any(feature = "on-server", feature = "on-web"))]
mod cfg_feature;
#[cfg(feature = "form-fields")]
mod form_fields;
#[cfg(feature = "form-options")]
mod form_options;
#[cfg(feature = "steps")]
mod steps;

/// Applies `#[cfg(feature = "server")]` to every item inside the block.
///
/// ```ignore
/// crate::on_server! {
///     use std::str::FromStr;
///     use crate::AutomationStepKind;
/// }
/// ```
#[cfg(feature = "on-server")]
#[proc_macro]
pub fn on_server(input: TokenStream) -> TokenStream {
    cfg_feature::cfg_feature_items(input, "server")
}

/// Applies `#[cfg(feature = "web")]` to every item inside the block.
///
/// ```ignore
/// crate::on_web! {
///     use web_sys::window;
/// }
/// ```
#[cfg(feature = "on-web")]
#[proc_macro]
pub fn on_web(input: TokenStream) -> TokenStream {
    cfg_feature::cfg_feature_items(input, "web")
}

/// Generates typed field constants for each struct field. See `libs/macros/README.md` for usage.
#[cfg(feature = "form-fields")]
#[proc_macro_derive(FormFields, attributes(field))]
pub fn derive_form_fields(input: TokenStream) -> TokenStream {
    form_fields::derive_form_fields(input)
}

/// Generates `const OPTIONS: &[(&str, &str)]` for unit enums. See `libs/macros/README.md` for usage.
#[cfg(feature = "form-options")]
#[proc_macro_derive(FormOptions)]
pub fn derive_form_options(input: TokenStream) -> TokenStream {
    form_options::derive_form_options(input)
}

/// Derives step metadata for multi-step form enums. See `libs/macros/README.md` for usage.
#[cfg(feature = "steps")]
#[proc_macro_derive(Steps, attributes(step))]
pub fn derive_steps(input: TokenStream) -> TokenStream {
    steps::derive_steps(input)
}
