use std::collections::HashMap;

use dioxus::dioxus_core::{RuntimeGuard, VirtualDom};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::{Validate, ValidationError, ValidationErrors};

use super::*;
// The `FormFields` derive emits `components::` paths (the consuming app's crate
// alias for this library); alias ourselves so the derive resolves in-crate.
use crate as components;
use crate::FormFields;
use crate::field_name::FieldType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
enum MockRole {
    Admin,
    User,
    #[default]
    Guest,
}

impl FormSchema for MockRole {
    const FIELD_TYPE: FieldType = FieldType::String;
    fn json_schema() -> Value {
        serde_json::to_value(Self::default()).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(transparent)]
pub struct MockEmail(String);

impl Validate for MockEmail {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        if !self.0.contains('@') {
            let mut err = ValidationError::new("invalid_email");
            err.message = Some("Must contain @".into());
            let mut errs = ValidationErrors::new();
            errs.add("0", err);
            return Err(errs);
        }
        Ok(())
    }
}

impl FormSchema for MockEmail {
    const FIELD_TYPE: FieldType = FieldType::String;
    fn json_schema() -> Value {
        Value::String(String::new())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
struct MockAddress {
    #[validate(length(min = 5, message = "Street too short"))]
    street: String,
    zip: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
struct MockUser {
    #[validate(length(min = 2, message = "Name too short"))]
    name: String,
    age: i32,
    is_active: bool,
    #[validate(nested)]
    email: MockEmail,
    #[validate(nested)]
    address: MockAddress,
    role: MockRole,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
struct MockItem {
    #[validate(length(min = 1, message = "Item name required"))]
    name: String,
    qty: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
struct MockOrder {
    title: String,
    #[validate(nested)]
    items: Vec<MockItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
struct MockPhone(String);

impl Validate for MockPhone {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        Ok(())
    }
}

impl FormSchema for MockPhone {
    const FIELD_TYPE: FieldType = FieldType::String;
    fn json_schema() -> Value {
        Value::String(String::new())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
#[serde(default)]
struct MockSpouse {
    first_name: String,
    last_name: String,
    phone: MockPhone,
    age: i32,
    active: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
#[serde(default)]
struct MockChild {
    name: String,
    age: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, PartialEq, FormFields)]
struct MockClient {
    name: String,
    #[validate(nested)]
    spouse: Option<MockSpouse>,
    children: Option<Vec<MockChild>>,
}

fn run_with_scope<R>(f: impl FnOnce() -> R) -> R {
    let vdom = VirtualDom::new(|| rsx! {});
    let _guard = RuntimeGuard::new(vdom.runtime());
    f()
}

fn make_form_with_values(values: Vec<(&str, &str)>) -> DynamicForm<MockUser> {
    let mut form = DynamicForm::<MockUser>::default();
    for (k, v) in values {
        form.values_signal
            .write()
            .insert(k.to_string(), v.to_string());
    }
    form
}

fn make_order_form_with_values(values: Vec<(&str, &str)>) -> DynamicForm<MockOrder> {
    let mut form = DynamicForm::<MockOrder>::default();
    for (k, v) in values {
        form.values_signal
            .write()
            .insert(k.to_string(), v.to_string());
    }
    form
}

#[test]
fn test_get_string_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("name", "Alice")]);
        assert_eq!(form.get(MockUser::name), Some("Alice".to_string()));
    });
}

#[test]
fn test_get_number_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("age", "30")]);
        assert_eq!(form.get(MockUser::age), Some(30));
    });
}

#[test]
fn test_get_bool_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("is_active", "true")]);
        assert_eq!(form.get(MockUser::is_active), Some(true));
    });
}

#[test]
fn test_get_enum_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("role", "admin")]);
        assert_eq!(form.get(MockUser::role), Some(MockRole::Admin));
    });
}

#[test]
fn test_set_enum_value_round_trips_without_json_quotes() {
    run_with_scope(|| {
        let form = DynamicForm::<MockUser>::default();

        form.set(MockUser::role.as_str(), MockRole::Admin);

        assert_eq!(form.get(MockUser::role), Some(MockRole::Admin));
        assert_eq!(form.get_data().unwrap().role, MockRole::Admin);
    });
}

#[test]
fn test_get_newtype_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("email", "a@b.com")]);
        assert_eq!(
            form.get(MockUser::email),
            Some(MockEmail("a@b.com".to_string()))
        );
    });
}

#[test]
fn test_get_nested_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("address.street", "123 Main St")]);
        assert_eq!(
            form.get(MockUser::address.dot(MockAddress::street)),
            Some("123 Main St".to_string())
        );
    });
}

#[test]
fn test_get_nested_number_field() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("address.zip", "90210")]);
        assert_eq!(
            form.get(MockUser::address.dot(MockAddress::zip)),
            Some(90210)
        );
    });
}

#[test]
fn test_get_empty_returns_none_for_number() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("age", "")]);
        assert_eq!(form.get(MockUser::age), None::<i32>);
    });
}

#[test]
fn test_get_empty_returns_none_for_string() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("name", "")]);
        assert_eq!(form.get(MockUser::name), None::<String>);
    });
}

#[test]
fn test_get_missing_field_returns_none() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![]);
        assert_eq!(form.get(MockUser::name), None::<String>);
    });
}

#[test]
fn test_get_invalid_number_returns_none() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("age", "not_a_number")]);
        assert_eq!(form.get(MockUser::age), None::<i32>);
    });
}

#[test]
fn test_get_invalid_enum_returns_none() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("role", "superadmin")]);
        assert_eq!(form.get(MockUser::role), None::<MockRole>);
    });
}

#[test]
fn test_get_or_with_value() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("name", "Alice")]);
        assert_eq!(form.get_or(MockUser::name, "fallback".to_string()), "Alice");
    });
}

#[test]
fn test_get_or_with_fallback() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![]);
        assert_eq!(
            form.get_or(MockUser::name, "fallback".to_string()),
            "fallback"
        );
    });
}

#[test]
fn test_get_or_number_with_fallback() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("age", "")]);
        assert_eq!(form.get_or(MockUser::age, 42), 42);
    });
}

#[test]
fn test_get_or_enum_with_fallback() {
    run_with_scope(|| {
        let form = make_form_with_values(vec![("role", "invalid")]);
        assert_eq!(
            form.get_or(MockUser::role, MockRole::Guest),
            MockRole::Guest
        );
    });
}

#[test]
fn test_get_array_item_field() {
    run_with_scope(|| {
        let form =
            make_order_form_with_values(vec![("items.0.name", "Widget"), ("items.0.qty", "5")]);
        assert_eq!(
            form.get(MockOrder::items.at(0).dot(MockItem::name)),
            Some("Widget".to_string())
        );
        assert_eq!(form.get(MockOrder::items.at(0).dot(MockItem::qty)), Some(5));
    });
}

#[test]
fn test_get_or_array_item_field_with_fallback() {
    run_with_scope(|| {
        let form = make_order_form_with_values(vec![]);
        assert_eq!(
            form.get_or(
                MockOrder::items.at(0).dot(MockItem::name),
                "default".to_string()
            ),
            "default"
        );
    });
}

// --- FormSchema::json_schema() tests ---

#[test]
fn test_json_schema_expands_optional_struct() {
    let schema = MockClient::json_schema();

    let spouse = schema.get("spouse").unwrap();
    assert!(spouse.is_object(), "spouse should be an object, not null");
    assert_eq!(spouse.get("first_name").unwrap(), &json!(""));
    assert_eq!(spouse.get("last_name").unwrap(), &json!(""));
    assert_eq!(spouse.get("phone").unwrap(), &json!(""));
    assert_eq!(spouse.get("age").unwrap(), &json!(0));
    assert_eq!(spouse.get("active").unwrap(), &json!(false));
}

#[test]
fn test_json_schema_expands_optional_vec() {
    let schema = MockClient::json_schema();

    let children = schema.get("children").unwrap();
    assert!(children.is_array(), "children should be an array, not null");
    assert_eq!(children.as_array().unwrap().len(), 1);
    let prototype = &children[0];
    assert_eq!(prototype.get("name").unwrap(), &json!(""));
    assert_eq!(prototype.get("age").unwrap(), &json!(0));
}

#[test]
fn test_json_schema_preserves_non_optional() {
    let schema = MockClient::json_schema();

    assert_eq!(schema.get("name").unwrap(), &json!(""));
}

// --- parse_form_data with Option<Struct> ---

#[test]
fn test_parse_optional_struct_fields() {
    let schema = MockClient::json_schema();

    let mut input = HashMap::new();
    input.insert("name".to_string(), "Alice".to_string());
    input.insert("spouse.first_name".to_string(), "Bob".to_string());
    input.insert("spouse.last_name".to_string(), "Smith".to_string());
    input.insert("spouse.phone".to_string(), "7788777056".to_string());
    input.insert("spouse.age".to_string(), "35".to_string());
    input.insert("spouse.active".to_string(), "true".to_string());

    let result: MockClient = parse_form_data(&input, &schema).expect("Failed to parse");

    assert_eq!(result.name, "Alice");
    let spouse = result.spouse.expect("spouse should be Some");
    assert_eq!(spouse.first_name, "Bob");
    assert_eq!(spouse.last_name, "Smith");
    assert_eq!(spouse.phone, MockPhone("7788777056".to_string()));
    assert_eq!(spouse.age, 35);
    assert!(spouse.active);
}

#[test]
fn test_parse_optional_struct_phone_not_coerced_to_number() {
    let schema = MockClient::json_schema();

    let mut input = HashMap::new();
    input.insert("spouse.phone".to_string(), "9995551234".to_string());

    let result: MockClient = parse_form_data(&input, &schema).expect("Failed to parse");

    let spouse = result.spouse.expect("spouse should be Some");
    assert_eq!(spouse.phone, MockPhone("9995551234".to_string()));
}

#[test]
fn test_parse_optional_struct_partial_fills_defaults() {
    let schema = MockClient::json_schema();

    let mut input = HashMap::new();
    input.insert("spouse.first_name".to_string(), "Jane".to_string());

    let result: MockClient = parse_form_data(&input, &schema).expect("Failed to parse");

    let spouse = result.spouse.expect("spouse should be Some");
    assert_eq!(spouse.first_name, "Jane");
    assert_eq!(spouse.last_name, "");
    assert_eq!(spouse.phone, MockPhone(String::new()));
    assert_eq!(spouse.age, 0);
    assert!(!spouse.active);
}

#[test]
fn test_parse_optional_struct_untouched_is_none() {
    let schema = MockClient::json_schema();

    let mut input = HashMap::new();
    input.insert("name".to_string(), "Alice".to_string());

    let result: MockClient = parse_form_data(&input, &schema).expect("Failed to parse");

    assert_eq!(result.name, "Alice");
    assert_eq!(result.spouse, None);
}

// --- parse_form_data with Option<Vec<T>> ---

#[test]
fn test_parse_optional_vec_fields() {
    let schema = MockClient::json_schema();

    let mut input = HashMap::new();
    input.insert("name".to_string(), "Alice".to_string());
    input.insert("children.0.name".to_string(), "Charlie".to_string());
    input.insert("children.0.age".to_string(), "8".to_string());
    input.insert("children.1.name".to_string(), "Dana".to_string());
    input.insert("children.1.age".to_string(), "5".to_string());

    let result: MockClient = parse_form_data(&input, &schema).expect("Failed to parse");

    let children = result.children.expect("children should be Some");
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].name, "Charlie");
    assert_eq!(children[0].age, 8);
    assert_eq!(children[1].name, "Dana");
    assert_eq!(children[1].age, 5);
}

#[test]
fn test_parse_optional_vec_untouched_is_none() {
    let schema = MockClient::json_schema();

    let mut input = HashMap::new();
    input.insert("name".to_string(), "Alice".to_string());

    let result: MockClient = parse_form_data(&input, &schema).expect("Failed to parse");

    assert_eq!(result.children, None);
}

// --- set_nested_value with null intermediate nodes ---

#[test]
fn test_set_nested_value_through_null_object() {
    let mut root = json!({ "name": "Alice", "spouse": null });

    set_nested_value(&mut root, "spouse.first_name", json!("Bob"));
    set_nested_value(&mut root, "spouse.age", json!(30));

    assert_eq!(root["spouse"]["first_name"], "Bob");
    assert_eq!(root["spouse"]["age"], 30);
}

#[test]
fn test_set_nested_value_through_null_array() {
    let mut root = json!({ "name": "Alice", "children": null });

    set_nested_value(&mut root, "children.0.name", json!("Charlie"));
    set_nested_value(&mut root, "children.0.age", json!(8));

    assert_eq!(root["children"][0]["name"], "Charlie");
    assert_eq!(root["children"][0]["age"], 8);
}

// --- coerce_value with Null expected type ---

#[test]
fn test_coerce_value_null_type_fallback() {
    assert_eq!(coerce_value("hello", FieldType::Null), Some(json!("hello")));
    assert_eq!(coerce_value("42", FieldType::Null), Some(json!(42)));
    assert_eq!(coerce_value("", FieldType::Null), None);
}

#[test]
fn test_coerce_value_null_type_empty_returns_none() {
    assert_eq!(coerce_value("", FieldType::Null), None);
}

// --- get_nested_type prototype fallback ---

#[test]
fn test_get_nested_type_array_prototype_fallback() {
    let schema = json!({
        "items": [{ "name": "", "qty": 0 }]
    });

    assert_eq!(get_nested_type(&schema, "items.5.name"), Some(&json!("")));
    assert_eq!(get_nested_type(&schema, "items.5.qty"), Some(&json!(0)));
}

#[test]
fn test_get_nested_type_empty_array_returns_none() {
    let schema = json!({ "items": [] });

    assert_eq!(get_nested_type(&schema, "items.0.name"), None);
}

// --- Full round-trip: DynamicForm<MockClient> ---

#[test]
fn test_form_get_data_with_optional_struct() {
    run_with_scope(|| {
        let mut form = DynamicForm::<MockClient>::default();
        for (k, v) in [
            ("name", "Alice"),
            ("spouse.first_name", "Bob"),
            ("spouse.phone", "5551234567"),
            ("spouse.age", "40"),
        ] {
            form.values_signal
                .write()
                .insert(k.to_string(), v.to_string());
        }

        let data = form.get_data().expect("get_data should succeed");
        assert_eq!(data.name, "Alice");
        let spouse = data.spouse.expect("spouse should be Some");
        assert_eq!(spouse.first_name, "Bob");
        assert_eq!(spouse.phone, MockPhone("5551234567".to_string()));
        assert_eq!(spouse.age, 40);
    });
}

#[test]
fn test_form_get_data_with_optional_vec() {
    run_with_scope(|| {
        let mut form = DynamicForm::<MockClient>::default();
        for (k, v) in [
            ("name", "Alice"),
            ("children.0.name", "Charlie"),
            ("children.0.age", "8"),
            ("children.1.name", "Dana"),
            ("children.1.age", "5"),
        ] {
            form.values_signal
                .write()
                .insert(k.to_string(), v.to_string());
        }

        let data = form.get_data().expect("get_data should succeed");
        assert_eq!(data.name, "Alice");
        let children = data.children.expect("children should be Some");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Charlie");
        assert_eq!(children[0].age, 8);
        assert_eq!(children[1].name, "Dana");
        assert_eq!(children[1].age, 5);
    });
}

// --- field_validation_error parity with flatten_validation_errors ---

/// `field_validation_error` must return byte-identical messages to the full-map
/// lookup `flatten_validation_errors(errs).get(field).cloned()` for every kind of
/// path: direct leaf, nested `Struct` (dot path), `List` item (`items[0].x`), the
/// transparent-wrapper bubble-up to the parent prefix, and `None` for a miss.
fn assert_parity(errs: &ValidationErrors, field: &str) {
    let flat = flatten_validation_errors(errs);
    assert_eq!(
        field_validation_error(errs, field),
        flat.get(field).cloned(),
        "field_validation_error diverged from flatten_validation_errors for {field:?}"
    );
}

#[test]
fn test_field_validation_error_parity_leaf_struct_and_wrapper() {
    // `name` (leaf), `address.street` (nested Struct), `email` (transparent
    // wrapper that bubbles its `0` field message up to the parent prefix).
    let user = MockUser {
        name: "x".to_string(),
        email: MockEmail("no-at-sign".to_string()),
        address: MockAddress {
            street: "no".to_string(),
            zip: 0,
        },
        ..Default::default()
    };
    let errs = user
        .validate()
        .expect_err("MockUser should fail validation");

    // Sanity: the representative tree actually contains the paths we exercise.
    let flat = flatten_validation_errors(&errs);
    assert!(
        flat.contains_key("name"),
        "expected a leaf error for `name`"
    );
    assert!(
        flat.contains_key("address.street"),
        "expected a nested Struct error for `address.street`"
    );
    assert!(
        flat.contains_key("email"),
        "expected transparent-wrapper bubble-up at parent prefix `email`"
    );

    assert_parity(&errs, "name"); // direct leaf hit
    assert_parity(&errs, "address.street"); // nested Struct dot-path
    assert_parity(&errs, "email"); // transparent-wrapper parent prefix
    assert_parity(&errs, "email.0"); // wrapper's inner leaf path
    assert_parity(&errs, "address"); // parent of a nested struct (no own error)
    assert_parity(&errs, "missing_field"); // miss → None
    assert_parity(&errs, "age"); // present field, no error → None
}

#[test]
fn test_field_validation_error_parity_list_items() {
    // `items` is a `Vec<MockItem>`; an empty `name` produces a `List` error at
    // the `items[0].name` path.
    let order = MockOrder {
        title: "Order".to_string(),
        items: vec![
            MockItem {
                name: String::new(),
                qty: 1,
            },
            MockItem {
                name: "ok".to_string(),
                qty: 2,
            },
        ],
    };
    let errs = order
        .validate()
        .expect_err("MockOrder should fail validation");

    let flat = flatten_validation_errors(&errs);
    assert!(
        flat.contains_key("items[0].name"),
        "expected a List item error at `items[0].name`"
    );

    assert_parity(&errs, "items[0].name"); // List item dot-path
    assert_parity(&errs, "items[1].name"); // valid item → None
    assert_parity(&errs, "items"); // list parent, no own error → None
    assert_parity(&errs, "title"); // present field, no error → None
}
