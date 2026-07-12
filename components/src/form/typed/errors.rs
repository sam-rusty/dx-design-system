//! `validator::ValidationErrors` → dot-notation path map.
//!
//! Unlike the legacy `form_utils` flattening, list indices use dots
//! (`items.2.qty`, not `items[2].qty`) so error keys line up with lens paths
//! and the aux-state maps.

use std::collections::HashMap;

use validator::{ValidationErrors, ValidationErrorsKind};

pub fn flatten_validation_errors(errors: &ValidationErrors) -> HashMap<String, String> {
    let mut map = HashMap::new();
    extract_recursive(errors, "", &mut map);
    map
}

/// Resolves the message for one `field` without flattening the whole tree.
/// Byte-identical to `flatten_validation_errors(errors).get(field)` for the
/// same key, including the transparent-wrapper bubble-up.
pub fn field_validation_error(errors: &ValidationErrors, field: &str) -> Option<String> {
    let mut out = None;
    find_recursive(errors, "", field, &mut out);
    out
}

fn extract_recursive(errors: &ValidationErrors, prefix: &str, map: &mut HashMap<String, String>) {
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
            ValidationErrorsKind::Struct(nested) => extract_recursive(nested, &path, map),
            ValidationErrorsKind::List(list) => {
                for (idx, nested) in list {
                    extract_recursive(nested, &format!("{path}.{idx}"), map);
                }
            }
        }
    }
}

fn find_recursive(errors: &ValidationErrors, prefix: &str, target: &str, out: &mut Option<String>) {
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
                find_recursive(nested, &path, target, out);
            }
            ValidationErrorsKind::List(list) => {
                for (idx, nested) in list {
                    find_recursive(nested, &format!("{path}.{idx}"), target, out);
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
