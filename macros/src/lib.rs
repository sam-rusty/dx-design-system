use proc_macro::TokenStream;

mod case;
mod cfg_feature;
mod form_fields;
mod form_options;
mod steps;

/// Applies `#[cfg(feature = "server")]` to every item inside the block.
///
/// ```ignore
/// macros::on_server! {
///     use std::str::FromStr;
///     use crate::AutomationStepKind;
/// }
/// ```
#[proc_macro]
pub fn on_server(input: TokenStream) -> TokenStream {
    cfg_feature::cfg_feature_items(input, "server")
}

/// Applies `#[cfg(feature = "web")]` to every item inside the block.
///
/// ```ignore
/// macros::on_web! {
///     use web_sys::window;
/// }
/// ```
#[proc_macro]
pub fn on_web(input: TokenStream) -> TokenStream {
    cfg_feature::cfg_feature_items(input, "web")
}

/// Generates typed field constants for each struct field. See `libs/macros/README.md` for usage.
#[proc_macro_derive(FormFields, attributes(field))]
pub fn derive_form_fields(input: TokenStream) -> TokenStream {
    form_fields::derive_form_fields(input)
}

/// Generates `const OPTIONS: &[(&str, &str)]` for unit enums. See `libs/macros/README.md` for usage.
#[proc_macro_derive(FormOptions)]
pub fn derive_form_options(input: TokenStream) -> TokenStream {
    form_options::derive_form_options(input)
}

/// Derives step metadata for multi-step form enums. See `libs/macros/README.md` for usage.
#[proc_macro_derive(Steps, attributes(step))]
pub fn derive_steps(input: TokenStream) -> TokenStream {
    steps::derive_steps(input)
}
