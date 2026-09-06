use dioxus::dioxus_core::{RuntimeGuard, VirtualDom};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::value::FormValue;
use super::*;
use crate as components;
use crate::{FormFields, FormOptions};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Address {
    #[validate(length(min = 2, message = "Street too short"))]
    street: String,
    zip: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct LineItem {
    product: String,
    qty: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct OrderForm {
    #[validate(length(min = 1, message = "Name is required"))]
    name: String,
    age: Option<i32>,
    active: bool,
    #[validate(nested)]
    address: Address,
    items: Vec<LineItem>,
    tags: Option<Vec<String>>,
    shipping: Option<Address>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FormOptions)]
#[serde(rename_all = "lowercase")]
enum Role {
    Admin,
    User,
}

/// Runs `f` inside a minimal dioxus runtime so `Signal::new` works.
fn in_runtime<R>(f: impl FnOnce() -> R) -> R {
    let vdom = VirtualDom::new(|| rsx! {});
    let _guard = RuntimeGuard::new(vdom.runtime());
    f()
}

fn new_form() -> Form<OrderForm> {
    let data = Signal::new_in_scope(OrderForm::default(), ScopeId::ROOT);
    Form {
        data,
        aux: Signal::new_in_scope(AuxState::default(), ScopeId::ROOT),
        registry: Signal::new_in_scope(Default::default(), ScopeId::ROOT),
        validate: CopyValue::new_in_scope(super::form::erased_validator(data), ScopeId::ROOT),
    }
}

// ---- lens tests (no runtime needed) ----

#[test]
fn lens_paths() {
    assert_eq!(OrderForm::name.path(), "name");
    assert_eq!(
        OrderForm::address.then(Address::street).path(),
        "address.street"
    );
    assert_eq!(
        OrderForm::items.nth(2).then(LineItem::qty).path(),
        "items.2.qty"
    );
    assert_eq!(OrderForm::tags.nth(0).path(), "tags.0");
}

#[test]
fn lens_get_and_get_mut() {
    let mut order = OrderForm::default();
    let street = OrderForm::address.then(Address::street);
    *street.get_mut(&mut order) = "Main St".to_string();
    assert_eq!(street.get(&order).map(String::as_str), Some("Main St"));

    // Index writes past the end grow the Vec with defaults.
    let qty2 = OrderForm::items.nth(2).then(LineItem::qty);
    *qty2.get_mut(&mut order) = 7;
    assert_eq!(order.items.len(), 3);
    assert_eq!(order.items[2].qty, 7);
    assert_eq!(qty2.get(&order), Some(&7));

    // Option<Vec> materializes on write, reads None while unset.
    let tag0 = OrderForm::tags.nth(0);
    assert_eq!(tag0.get(&OrderForm::default()), None);
    *tag0.get_mut(&mut order) = "a".to_string();
    assert_eq!(order.tags, Some(vec!["a".to_string()]));
}

#[test]
fn lens_metadata() {
    assert_eq!(OrderForm::name.label(), "Name");
    assert!(OrderForm::name.required());
    assert!(!OrderForm::age.required());
    let nested = OrderForm::address.then(Address::street);
    assert_eq!(nested.label(), "Street");
}

// ---- FormValue ----

#[test]
fn form_value_roundtrips() {
    assert_eq!(String::from_input("x").unwrap(), "x");
    assert_eq!(i32::from_input(" 42 ").unwrap(), 42);
    assert!(
        f64::from_input("3.").is_ok(),
        "trailing dot parses as f64 in rust"
    );
    assert!(i32::from_input("abc").is_err());
    assert_eq!(Option::<i32>::from_input("5").unwrap(), Some(5));
    assert_eq!(Option::<i32>::empty(), Some(None));
    assert_eq!(i32::empty(), None);
    assert_eq!(Role::from_input("admin").unwrap(), Role::Admin);
    assert_eq!(Role::Admin.to_input(), "admin");
    assert!(Role::from_input("nope").is_err());
}

// ---- form core (runtime) ----

#[test]
fn set_and_get_typed() {
    in_runtime(|| {
        let form = new_form();
        form.set(OrderForm::name, "Ada".to_string());
        assert_eq!(form.get_untracked(OrderForm::name).unwrap(), "Ada");
        assert_eq!(form.get_data().name, "Ada");

        form.set(OrderForm::items.nth(0).then(LineItem::qty), 3);
        assert_eq!(form.get_data().items[0].qty, 3);
    });
}

#[test]
fn pristine_fields_display_blank() {
    in_runtime(|| {
        let form = new_form();
        // qty default is 0 but the field was never written
        assert_eq!(
            form.display(OrderForm::items.nth(0).then(LineItem::qty)),
            ""
        );
        form.set(OrderForm::items.nth(0).then(LineItem::qty), 0);
        assert_eq!(
            form.display(OrderForm::items.nth(0).then(LineItem::qty)),
            "0"
        );
    });
}

#[test]
fn set_text_parses_or_overlays() {
    in_runtime(|| {
        let form = new_form();
        let age = OrderForm::age;

        form.set_text(age, "30");
        assert_eq!(form.get_data().age, Some(30));
        assert_eq!(form.display(age), "30");

        // Unparseable text: typed value keeps last valid, display echoes text.
        form.set_text(age, "3x");
        assert_eq!(form.get_data().age, Some(30));
        assert_eq!(form.display(age), "3x");

        // Overlay blocks submit even though the struct itself validates.
        form.set(OrderForm::name, "Ada".to_string());
        form.set(
            OrderForm::address.then(Address::street),
            "Main St".to_string(),
        );
        assert!(form.validate_and_get().is_none());
        assert!(form.error("age").is_some());

        // Fixing the text clears the overlay and submit passes.
        form.set_text(age, "31");
        assert_eq!(form.get_data().age, Some(31));
        form.clear_global_error();
        assert!(form.validate_and_get().is_some());
    });
}

#[test]
fn empty_text_resets_to_pristine() {
    in_runtime(|| {
        let form = new_form();
        form.set_text(OrderForm::age, "30");
        assert_eq!(form.get_data().age, Some(30));

        form.set_text(OrderForm::age, "");
        assert_eq!(form.get_data().age, None, "Option resets to None");
        assert_eq!(form.display(OrderForm::age), "");
        assert!(!form.aux.peek().is_written("age"));
    });
}

#[test]
fn validate_and_get_flattens_errors_with_dots() {
    in_runtime(|| {
        let form = new_form();
        form.set(OrderForm::address.then(Address::street), "x".to_string());
        assert!(form.validate_and_get().is_none());
        assert_eq!(
            form.error("address.street"),
            Some("Street too short".into())
        );
        assert_eq!(form.error("name"), Some("Name is required".into()));
        assert!(form.global_error().is_some());
        assert!(
            form.is_touched("address.street"),
            "errored fields get touched"
        );
    });
}

#[test]
fn per_field_validation_on_touch() {
    in_runtime(|| {
        let form = new_form();
        let street = OrderForm::address.then(Address::street);
        form.set(street, "x".to_string());
        assert_eq!(
            form.error("address.street"),
            None,
            "untouched: no error yet"
        );
        form.touch_field("address.street");
        assert_eq!(
            form.error("address.street"),
            Some("Street too short".into())
        );
        form.set(street, "Long enough".to_string());
        assert_eq!(
            form.error("address.street"),
            None,
            "touched field revalidates on set"
        );
    });
}

#[test]
fn default_values_marks_all_written() {
    in_runtime(|| {
        let form = new_form();
        form.default_values(OrderForm {
            name: "Ada".into(),
            age: None,
            active: true,
            ..Default::default()
        });
        assert_eq!(form.display(OrderForm::name), "Ada");
        assert_eq!(form.display(OrderForm::active), "true");
        assert_eq!(form.display(OrderForm::age), "", "None renders blank");
        // clearing one field doesn't blank the others
        form.set_text(OrderForm::name, "");
        assert_eq!(form.display(OrderForm::active), "true");
        assert_eq!(form.display(OrderForm::name), "");
    });
}

#[test]
fn rows_mutations_rekey_aux() {
    in_runtime(|| {
        let form = new_form();
        let rows = form.rows(OrderForm::items);
        rows.push(LineItem {
            product: "a".into(),
            qty: 1,
        });
        rows.push(LineItem {
            product: "b".into(),
            qty: 2,
        });
        rows.push(LineItem {
            product: "c".into(),
            qty: 3,
        });

        // park an overlay + error on row 2
        form.set_text(OrderForm::items.nth(2).then(LineItem::qty), "xx");
        form.touch_field("items.2.qty");
        assert!(form.error("items.2.qty").is_some());

        rows.remove(0);
        assert_eq!(form.get_data().items.len(), 2);
        assert_eq!(form.get_data().items[1].product, "c");
        // aux entries followed the row from index 2 to index 1
        assert!(form.error("items.1.qty").is_some());
        assert_eq!(form.error("items.2.qty"), None);
        assert_eq!(
            form.display(OrderForm::items.nth(1).then(LineItem::qty)),
            "xx"
        );

        rows.swap(0, 1);
        assert!(form.error("items.0.qty").is_some());
        assert_eq!(form.error("items.1.qty"), None);
    });
}

#[test]
fn field_binding_reads_and_writes() {
    in_runtime(|| {
        let form = new_form();
        let binding = form.field(OrderForm::name);
        assert_eq!(binding.path(), "name");
        assert_eq!(binding.label(), "Name");
        assert!(binding.required());
        assert_eq!(binding.display(), "");
        assert!(!binding.has_value());

        binding.commit_text("Ada");
        assert_eq!(binding.value().unwrap(), "Ada");
        assert!(binding.has_value());
        assert!(binding.is_touched());
        assert_eq!(binding.error(), None);

        binding.commit_text("");
        assert_eq!(binding.error(), Some("Name is required".into()));
        assert!(binding.invalid());

        binding.set("Grace".to_string());
        assert_eq!(form.get_data().name, "Grace");
        assert_eq!(binding.error(), None);
    });
}

#[test]
fn registry_required_check_blocks_submit() {
    in_runtime(|| {
        let form = new_form();
        // name has a validator rule; qty on row 0 is required but has no rule
        let qty = form.field(OrderForm::items.nth(0).then(LineItem::qty));
        qty.register();
        form.set(OrderForm::name, "Ada".to_string());
        form.set(
            OrderForm::address.then(Address::street),
            "Main St".to_string(),
        );

        assert!(
            form.validate_and_get().is_none(),
            "pristine required qty blocks"
        );
        assert_eq!(form.error("items.0.qty"), Some("Qty is required".into()));

        qty.set(5);
        form.clear_global_error();
        assert!(form.validate_and_get().is_some());

        qty.unregister();
        form.reset();
        form.set(OrderForm::name, "Ada".to_string());
        form.set(
            OrderForm::address.then(Address::street),
            "Main St".to_string(),
        );
        assert!(
            form.validate_and_get().is_some(),
            "unregistered field not checked"
        );
    });
}

/// Two mounted `FormField` instances re-keying under `rows.swap` /
/// `rows.remove` (A2): the re-key order is "old instance unregisters, new
/// path registers" per instance, and the dropped instance's `use_drop`
/// fires after its sibling has already shifted into the freed path.
#[test]
fn registry_unregister_is_owner_aware() {
    in_runtime(|| {
        let form = new_form();
        let rows = form.rows(OrderForm::items);
        rows.push(LineItem::default());
        rows.push(LineItem::default());
        let qty0 = OrderForm::items.nth(0).then(LineItem::qty);
        let qty1 = OrderForm::items.nth(1).then(LineItem::qty);
        let a = form.field(qty0);
        a.register();
        let b = form.field(qty1);
        b.register();

        rows.swap(0, 1);
        let a2 = form.field(qty1);
        a.unregister();
        a2.register();
        let b2 = form.field(qty0);
        b.unregister();
        b2.register();
        {
            let reg = form.registry.peek();
            assert!(reg.contains_key("items.0.qty"), "swap kept row 0");
            assert!(reg.contains_key("items.1.qty"), "swap kept row 1");
        }

        rows.remove(0);
        let a3 = form.field(qty0);
        a2.unregister();
        a3.register();
        b2.unregister();
        {
            let reg = form.registry.peek();
            assert!(reg.contains_key("items.0.qty"), "survivor stays registered");
            assert!(!reg.contains_key("items.1.qty"));
        }

        // The surviving registration still drives the required check.
        form.set(OrderForm::name, "Ada".to_string());
        form.set(
            OrderForm::address.then(Address::street),
            "Main St".to_string(),
        );
        assert!(form.validate_and_get().is_none());
        assert_eq!(form.error("items.0.qty"), Some("Qty is required".into()));
    });
}

/// A8: after `default_values`, `rows.clear()` + `rows.push()` renders the new
/// row's fields blank, like the non-`default_values` path.
#[test]
fn rows_cleared_under_default_values_render_blank() {
    in_runtime(|| {
        let form = new_form();
        form.default_values(OrderForm {
            items: vec![LineItem {
                product: "a".into(),
                qty: 1,
            }],
            ..Default::default()
        });
        let rows = form.rows(OrderForm::items);
        let qty0 = OrderForm::items.nth(0).then(LineItem::qty);
        assert_eq!(form.display(qty0), "1");

        rows.clear();
        rows.push(LineItem::default());
        assert_eq!(form.display(qty0), "", "new row is pristine");
        assert_eq!(
            form.display(OrderForm::items.nth(0).then(LineItem::product)),
            ""
        );
        assert_eq!(form.display(OrderForm::name), "", "blanket still on");
        assert!(form.aux.peek().is_written("active"));

        form.set(qty0, 3);
        assert_eq!(form.display(qty0), "3");
    });
}

#[test]
fn reset_after_default_values_returns_to_pristine() {
    in_runtime(|| {
        let form = new_form();
        form.default_values(OrderForm {
            name: "Ada".into(),
            active: true,
            ..Default::default()
        });
        form.touch_field("name");
        form.reset();
        assert_eq!(form.display(OrderForm::name), "");
        assert_eq!(form.display(OrderForm::active), "");
        assert!(!form.is_touched("name"));
        assert_eq!(form.get_data(), OrderForm::default());
    });
}

#[test]
fn some_lens_reads_none_and_materializes_on_write() {
    in_runtime(|| {
        let form = new_form();
        let zip = OrderForm::shipping.some().then(Address::zip);
        assert_eq!(zip.path(), "shipping.zip", "`some()` is path-transparent");
        assert_eq!(form.get_untracked(zip), None);
        assert_eq!(form.display(zip), "");

        form.set_text(zip, "90210");
        assert_eq!(
            form.get_data().shipping.map(|a| a.zip),
            Some("90210".to_string())
        );
        assert_eq!(form.display(zip), "90210");

        let binding = form.field(zip);
        assert_eq!(binding.value(), Some("90210".to_string()));
        binding.clear();
        assert_eq!(form.get_data().shipping.map(|a| a.zip), Some(String::new()));
        assert_eq!(binding.display(), "");
    });
}

/// Precedence when every source fails at once: overlay parse error, then the
/// `Validate` message, then required-but-empty.
#[test]
fn error_precedence_overlay_validate_required() {
    in_runtime(|| {
        let form = new_form();
        // required-but-empty only (no `Validate` rule on zip)
        let zip = form.field(OrderForm::address.then(Address::zip));
        zip.register();
        form.touch_field("address.zip");
        assert_eq!(form.error("address.zip"), Some("Zip is required".into()));

        // `Validate` beats required: street is empty *and* too short
        let street = OrderForm::address.then(Address::street);
        form.field(street).register();
        form.touch_field("address.street");
        assert_eq!(
            form.error("address.street"),
            Some("Street too short".into())
        );

        // overlay beats `Validate`: an unparseable number on a touched field
        let age = form.field(OrderForm::age);
        age.register();
        age.commit_text("3x");
        assert_eq!(age.error(), Some("Enter a valid number".into()));

        // submit-time: overlay message wins over validate/required per field
        assert!(form.validate_and_get().is_none());
        assert_eq!(form.error("age"), Some("Enter a valid number".into()));
        assert_eq!(
            form.error("address.street"),
            Some("Street too short".into())
        );
        assert_eq!(form.error("address.zip"), Some("Zip is required".into()));
        assert_eq!(form.error("name"), Some("Name is required".into()));
    });
}

#[test]
fn server_error_other_sets_global_only() {
    in_runtime(|| {
        let form = new_form();
        form.set_server_error(ds_utils::DsError::Other("Boom".into()));
        assert_eq!(form.global_error(), Some("Boom".into()));
        assert_eq!(form.error("name"), None);
    });
}

#[test]
fn server_error_maps_to_fields() {
    in_runtime(|| {
        let form = new_form();
        form.set_server_error(ds_utils::DsError::Validation(
            "Bad request".into(),
            ds_utils::DsError::form_field_error("name", "Taken".into()),
        ));
        assert_eq!(form.global_error(), Some("Bad request".into()));
        assert_eq!(form.error("name"), Some("Taken".into()));
    });
}

#[test]
fn submit_clears_global_error() {
    in_runtime(|| {
        let form = new_form();
        form.set(OrderForm::name, "Ada".to_string());
        form.set(
            OrderForm::address.then(Address::street),
            "Main St".to_string(),
        );
        form.set_server_error(ds_utils::DsError::Other("Try again".into()));

        form.submit(|_| {});

        assert_eq!(form.global_error(), None);
    });
}

/// Worst-case keystroke probe: 200-row Vec form, per-keystroke cost of a
/// touched `set_text` (typed write + whole-struct validate + display memo
/// recompute). Run manually: `cargo test -p ds-components --lib -- --ignored
/// keystroke_probe --nocapture`.
#[test]
#[ignore = "perf probe, run manually with --nocapture"]
fn keystroke_probe_200_rows() {
    in_runtime(|| {
        let form = new_form();
        let rows = form.rows(OrderForm::items);
        for i in 0..200 {
            rows.push(LineItem {
                product: format!("product-{i}"),
                qty: i,
            });
        }
        let qty = OrderForm::items.nth(100).then(LineItem::qty);
        form.touch_field("items.100.qty");

        let start = std::time::Instant::now();
        const N: u32 = 1_000;
        for i in 0..N {
            form.set_text(qty, &format!("{i}"));
            // What each mounted field's display memo pays per keystroke.
            let _ = form.display(OrderForm::items.nth(50).then(LineItem::product));
        }
        let per_keystroke = start.elapsed() / N;
        println!("200-row form, touched set_text + 1 display read: {per_keystroke:?}/keystroke");
        assert!(
            per_keystroke < std::time::Duration::from_millis(2),
            "keystroke cost regressed: {per_keystroke:?}"
        );
    });
}

#[test]
fn binding_partial_eq_is_identity_based() {
    in_runtime(|| {
        let form = new_form();
        let a = form.field(OrderForm::name);
        let b = form.field(OrderForm::name);
        let c = form.field(OrderForm::address.then(Address::street));
        assert_eq!(a, b);
        assert_ne!(a, c);
    });
}
