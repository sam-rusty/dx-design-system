#[cfg(any(
    feature = "form-fields",
    feature = "form-options",
    feature = "on-server",
    feature = "on-web",
    feature = "steps"
))]
use proc_macro::TokenStream;

#[cfg(feature = "form-fields")]
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

/// Generates typed field constants for each struct field. See `macros/README.md` for usage.
#[cfg(feature = "form-fields")]
#[proc_macro_derive(FormFields, attributes(field))]
pub fn derive_form_fields(input: TokenStream) -> TokenStream {
    form_fields::derive_form_fields(input)
}

/// Generates `const OPTIONS: &[(&str, &str)]` (value, label) for enums. See `macros/README.md`
/// for usage.
///
/// Values follow `#[serde(rename_all)]` / `#[serde(rename)]`; labels use
/// `#[strum(to_string = "..")]` or the variant name as `Title Case`.
///
/// Enums with data-carrying variants are accepted: they get `OPTIONS` and the `FormSchema`
/// impl, but no `FormValue` impl (a value cannot round-trip through a plain string). At least
/// one unit variant is required — it is the default/first option — otherwise the derive is a
/// compile error.
#[cfg(feature = "form-options")]
#[proc_macro_derive(FormOptions, attributes(strum, form_options))]
pub fn derive_form_options(input: TokenStream) -> TokenStream {
    form_options::derive_form_options(input)
}

/// Derives step metadata (`ALL`, `COUNT`, `TITLES`, `DESCRIPTIONS`) for unit enums used as
/// wizard steps. See `macros/README.md` for usage.
///
/// `#[step(title = "..", description = "..")]` overrides the defaults (variant name as
/// `Title Case`, empty description). Any other key, a non-string value, or a non-unit variant
/// is a compile error.
#[cfg(feature = "steps")]
#[proc_macro_derive(Steps, attributes(step))]
pub fn derive_steps(input: TokenStream) -> TokenStream {
    steps::derive_steps(input)
}

/// Generates only the framework-independent `OPTIONS` constant, with no Dioxus traits.
/// Supports canonical, symmetric, string-valued unit enums. Add the web form traits with
/// `#[derive(FormOptions)] #[form_options(existing)]`.
#[cfg(feature = "form-options")]
#[proc_macro_derive(EnumOptions, attributes(serde, strum))]
pub fn derive_enum_options(input: TokenStream) -> TokenStream {
    form_options::derive_enum_options(input)
}
