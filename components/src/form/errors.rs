//! `validator::ValidationErrors` → dot-notation path map. Shared by both
//! form flavors ([`crate::form::use_form`] and [`crate::form::use_dynamic_form`]).
//!
//! The two flavors key list indices differently: typed lens paths use dots
//! (`items.2.qty`) while the dynamic string-map form keeps the legacy
//! bracket keys (`items[2].qty`). Everything else — message extraction,
//! transparent-wrapper bubble-up, per-field lookup parity — is identical, so
//! the walkers are parameterized by [`ListIndexStyle`].

use std::collections::HashMap;

use validator::{ValidationErrors, ValidationErrorsKind};

/// Error slot for failures that don't belong to a specific field.
pub const GLOBAL_ERROR: &str = "__global";

/// How list indices appear in flattened error paths.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ListIndexStyle {
    /// `items.2.qty` — matches typed lens paths / aux-state keys.
    Dots,
    /// `items[2].qty` — legacy string-map keys.
    Brackets,
}

impl ListIndexStyle {
    fn join_index(&self, path: &str, idx: usize) -> String {
        match self {
            ListIndexStyle::Dots => format!("{path}.{idx}"),
            ListIndexStyle::Brackets => format!("{path}[{idx}]"),
        }
    }
}

pub(crate) fn flatten_validation_errors(
    errors: &ValidationErrors,
    style: ListIndexStyle,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    extract_recursive(errors, "", style, &mut map);
    map
}

/// Resolves the message for one `field` without flattening the whole tree.
/// Byte-identical to `flatten_validation_errors(errors, style).get(field)` for
/// the same key, including the transparent-wrapper bubble-up.
pub(crate) fn field_validation_error(
    errors: &ValidationErrors,
    field: &str,
    style: ListIndexStyle,
) -> Option<String> {
    let mut out = None;
    find_recursive(errors, "", field, style, &mut out);
    out
}

fn extract_recursive(
    errors: &ValidationErrors,
    prefix: &str,
    style: ListIndexStyle,
    map: &mut HashMap<String, String>,
) {
    for (field, kind) in errors.errors() {
        let path = join(prefix, field);
        match kind {
            ValidationErrorsKind::Field(errs) => {
                if let Some(err) = errs.first() {
                    let msg = err
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| err.code.to_string());
                    map.insert(path.clone(), msg.clone());
                    // Bubble up for transparent wrappers (like `Email`).
                    if !prefix.is_empty() && !map.contains_key(prefix) {
                        map.insert(prefix.to_string(), msg);
                    }
                }
            }
            ValidationErrorsKind::Struct(nested) => extract_recursive(nested, &path, style, map),
            ValidationErrorsKind::List(list) => {
                for (idx, nested) in list {
                    extract_recursive(nested, &style.join_index(&path, *idx), style, map);
                }
            }
        }
    }
}

fn find_recursive(
    errors: &ValidationErrors,
    prefix: &str,
    target: &str,
    style: ListIndexStyle,
    out: &mut Option<String>,
) {
    for (field, kind) in errors.errors() {
        let path = join(prefix, field);
        match kind {
            ValidationErrorsKind::Field(errs) => {
                if let Some(err) = errs.first() {
                    let msg = err
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| err.code.to_string());
                    if path == target {
                        *out = Some(msg);
                        return;
                    }
                    if !prefix.is_empty() && prefix == target && out.is_none() {
                        *out = Some(msg);
                    }
                }
            }
            ValidationErrorsKind::Struct(nested) => {
                find_recursive(nested, &path, target, style, out);
            }
            ValidationErrorsKind::List(list) => {
                for (idx, nested) in list {
                    find_recursive(nested, &style.join_index(&path, *idx), target, style, out);
                }
            }
        }
        if out.is_some() {
            return;
        }
    }
}

fn join(prefix: &str, field: &str) -> String {
    if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{prefix}.{field}")
    }
}
