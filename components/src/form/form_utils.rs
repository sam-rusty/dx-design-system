use std::collections::{HashMap, HashSet};

use ds_utils::{DsError, Result};
use serde_json::{Value, json};
use validator::ValidationErrors;

use crate::field_name::FieldType;
use crate::form::FormData;
use crate::form::errors::ListIndexStyle;

fn parent_key_set(values: &HashMap<String, String>) -> HashSet<&str> {
    let mut keys: Vec<&str> = values.keys().map(|s| s.as_str()).collect();
    keys.sort_unstable();
    let mut prefix_buf = String::new();
    keys.iter()
        .copied()
        .filter(|&path| {
            prefix_buf.clear();
            prefix_buf.push_str(path);
            prefix_buf.push('.');
            let start = keys.partition_point(|k| *k < prefix_buf.as_str());
            keys.get(start)
                .is_some_and(|k| k.starts_with(prefix_buf.as_str()))
        })
        .collect()
}

/// Parses the flat string map into the strictly typed Struct `T`.
///
/// Uses `default_schema` (from `FormSchema::json_schema()`) only for type
/// inference. The initial data tree comes from `T::default()` so that
/// prototype array elements in the schema don't leak into the result.
pub fn parse_form_data<T: FormData>(
    values: &HashMap<String, String>,
    default_schema: &Value,
) -> Result<T> {
    let mut root_val = serde_json::to_value(T::default()).unwrap_or(Value::Null);

    let mut entries: Vec<_> = values.iter().collect();
    entries.sort_by_key(|(path, _)| path.len());

    let parent_keys = parent_key_set(values);

    for (path, input_str) in &entries {
        if parent_keys.contains(path.as_str()) {
            continue;
        }

        let expected_type = get_nested_type(default_schema, path).unwrap_or(&Value::Null);
        let field_type = FieldType::from_value(expected_type);

        if let Some(coerced_val) = coerce_value(input_str, field_type) {
            set_nested_value(&mut root_val, path, coerced_val);
        }
    }

    serde_json::from_value(root_val).map_err(|e| DsError::Other(e.to_string()))
}

pub fn is_field_empty(values: &HashMap<String, String>, field: &str) -> bool {
    match values.get(field) {
        None => true,
        Some(v) => v.trim().is_empty() || v == "[]",
    }
}

pub fn flatten_json_value(value: &Value, prefix: &str, out: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_json_value(val, &path, out);
            }
        }
        Value::Array(arr) => {
            let serialized = serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string());
            out.insert(prefix.to_string(), serialized);
            for (i, item) in arr.iter().enumerate() {
                flatten_json_value(item, &format!("{prefix}.{i}"), out);
            }
        }
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        Value::Number(n) => {
            out.insert(prefix.to_string(), n.to_string());
        }
        Value::Bool(b) => {
            out.insert(prefix.to_string(), b.to_string());
        }
        Value::Null => {
            out.insert(prefix.to_string(), String::new());
        }
    }
}

/// Helper: Fetches the expected type from a dot-notation path (supports numeric segments for arrays)
pub fn get_nested_type<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |curr, part| {
        if let Ok(idx) = part.parse::<usize>() {
            curr.get(idx).or_else(|| curr.get(0))
        } else {
            curr.get(part)
        }
    })
}

/// Helper: Safely maps a string into the exact JSON type the struct expects
pub fn coerce_value(input: &str, field_type: FieldType) -> Option<Value> {
    if input.is_empty() {
        return None;
    }
    match field_type {
        FieldType::String => Some(Value::String(input.to_string())),
        FieldType::Bool => input.parse::<bool>().ok().map(Value::Bool),
        FieldType::Number => {
            if let Ok(n) = input.parse::<i64>() {
                Some(Value::Number(n.into()))
            } else if let Ok(f) = input.parse::<f64>() {
                serde_json::Number::from_f64(f).map(Value::Number)
            } else {
                None
            }
        }
        FieldType::Array => match serde_json::from_str::<Value>(input) {
            Ok(v @ Value::Array(_)) => Some(v),
            _ => None,
        },
        FieldType::Null | FieldType::Object => {
            Some(serde_json::from_str(input).unwrap_or_else(|_| Value::String(input.to_string())))
        }
    }
}

/// Helper: Inserts a JSON value deep into an object/array using a dot-notation path
/// (supports numeric segments as array indices, e.g. "items.0.name")
pub fn set_nested_value(root: &mut Value, path: &str, value: Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;

        if let Ok(idx) = part.parse::<usize>() {
            if !current.is_array() {
                *current = json!([]);
            }
            let Some(arr) = current.as_array_mut() else {
                debug_assert!(
                    false,
                    "set_nested_value: expected array at segment {i} of {path:?}"
                );
                return;
            };
            while arr.len() <= idx {
                arr.push(json!(null));
            }
            if is_last {
                arr[idx] = value;
                return;
            }
            if let Some(next) = parts.get(i + 1)
                && arr[idx].is_null()
            {
                arr[idx] = if next.parse::<usize>().is_ok() {
                    json!([])
                } else {
                    json!({})
                };
            }
            current = &mut arr[idx];
        } else {
            if !current.is_object() {
                debug_assert!(
                    false,
                    "set_nested_value: expected object at segment {i} of {path:?} (got non-object)"
                );
                return;
            }
            if is_last {
                let Some(obj) = current.as_object_mut() else {
                    debug_assert!(
                        false,
                        "set_nested_value: expected object at segment {i} of {path:?}"
                    );
                    return;
                };
                obj.insert(part.to_string(), value);
                return;
            }
            if current.get(*part).is_none() || current.get(*part).is_some_and(Value::is_null) {
                let next_val = if let Some(next) = parts.get(i + 1) {
                    if next.parse::<usize>().is_ok() {
                        json!([])
                    } else {
                        json!({})
                    }
                } else {
                    json!({})
                };
                let Some(obj) = current.as_object_mut() else {
                    debug_assert!(
                        false,
                        "set_nested_value: expected object at segment {i} of {path:?}"
                    );
                    return;
                };
                obj.insert(part.to_string(), next_val);
            }
            let Some(next_current) = current.get_mut(*part) else {
                debug_assert!(
                    false,
                    "set_nested_value: missing key {part:?} after insert in {path:?}"
                );
                return;
            };
            current = next_current;
        }
    }
}

/// Recursively flattens nested ValidationErrors into a simple HashMap.
/// Delegates to the shared walker; the dynamic form keeps the legacy
/// bracket-style list keys (`items[2].qty`).
pub fn flatten_validation_errors(errors: &ValidationErrors) -> HashMap<String, String> {
    crate::form::errors::flatten_validation_errors(errors, ListIndexStyle::Brackets)
}

/// Resolves the validation message for a single `field` (dot-notation) without
/// flattening the whole error tree. Byte-identical to
/// `flatten_validation_errors(errors).get(field)` for the same key, including
/// the transparent-wrapper bubble-up.
pub fn field_validation_error(errors: &ValidationErrors, field: &str) -> Option<String> {
    crate::form::errors::field_validation_error(errors, field, ListIndexStyle::Brackets)
}

#[cfg(test)]
mod parent_key_tests {
    use std::collections::HashMap;

    use super::parent_key_set;

    #[test]
    fn parent_key_set_treats_dot_child_as_descendant_not_sibling_prefix() {
        let mut m = HashMap::new();
        m.insert("p".into(), "1".into());
        m.insert("p.x".into(), "2".into());
        m.insert("pa".into(), "3".into());
        let pk = parent_key_set(&m);
        assert!(
            pk.contains("p"),
            "`p` has child `p.x` so it must be skipped as leaf in parse"
        );
        assert!(
            !pk.contains("pa"),
            "`pa` is not a prefix of `p.x`; must not be marked parent"
        );
    }

    #[test]
    fn parent_key_set_leaf_without_children_not_parent() {
        let mut m = HashMap::new();
        m.insert("a".into(), "1".into());
        m.insert("b".into(), "2".into());
        let pk = parent_key_set(&m);
        assert!(!pk.contains("a"));
        assert!(!pk.contains("b"));
    }

    #[test]
    fn parent_key_set_nested_path() {
        let mut m = HashMap::new();
        m.insert("user".into(), "{}".into());
        m.insert("user.name".into(), "x".into());
        let pk = parent_key_set(&m);
        assert!(pk.contains("user"));
    }
}
