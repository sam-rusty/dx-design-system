//! Behavioural SSR pins for the typed form: reactive `has_value` (A1), path
//! re-keys keeping label/control/error ids aligned (A3), and the markup
//! conventions of the shared form element / inline error.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "form")]

use std::cell::RefCell;

use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use validator::Validate;

use components::FormFields;
use components::form::{Form, FormProvider, LensExt, NumberInput, TextInput, use_form};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Item {
    name: String,
    qty: Option<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Order {
    #[validate(length(min = 1, message = "Title is required"))]
    title: String,
    items: Vec<Item>,
}

type Handles = (components::form::Form<Order>, Signal<usize>);

thread_local! {
    static HANDLES: RefCell<Option<Handles>> = const { RefCell::new(None) };
}

fn app() -> Element {
    let form = use_form::<Order>();
    let idx = use_signal(|| 0usize);
    use_hook(move || {
        form.rows(Order::items).push(Item::default());
        form.rows(Order::items).push(Item::default());
        HANDLES.with(|h| *h.borrow_mut() = Some((form, idx)));
    });
    rsx! {
        FormProvider { form,
            Form {
                TextInput { field: Order::title }
                TextInput {
                    field: form.field(Order::items.nth(idx()).then(Item::name)),
                    clearable: true,
                }
                NumberInput { field: form.field(Order::items.nth(idx()).then(Item::qty)) }
            }
        }
    }
}

struct Harness {
    dom: VirtualDom,
    form: components::form::Form<Order>,
    idx: Signal<usize>,
}

impl Harness {
    fn new() -> Self {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let (form, idx) = HANDLES.with(|h| h.borrow_mut().take()).unwrap();
        Self { dom, form, idx }
    }

    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }

    fn update(&mut self, f: impl FnOnce(components::form::Form<Order>, Signal<usize>)) -> String {
        let (form, idx) = (self.form, self.idx);
        self.dom.in_runtime(|| f(form, idx));
        self.dom.render_immediate(&mut NoOpMutations);
        self.html()
    }
}

/// The `FieldActions` container for the `items.N.name` field.
fn actions_div(html: &str) -> &str {
    let start = html
        .find("absolute end-2 top-1/2")
        .expect("FieldActions container rendered");
    let tag_start = html[..start].rfind("<div").unwrap();
    let end = html[tag_start..].find('>').unwrap();
    &html[tag_start..tag_start + end + 1]
}

#[test]
fn field_actions_follow_has_value_reactively() {
    let mut h = Harness::new();
    let html = h.html();
    assert!(
        actions_div(&html).contains("display:none"),
        "pristine field hides the actions: {}",
        actions_div(&html)
    );

    let html = h.update(|form, _| {
        form.field(Order::items.nth(0).then(Item::name))
            .commit_text("widget");
    });
    assert!(
        !actions_div(&html).contains("display:none"),
        "written field shows the actions: {}",
        actions_div(&html)
    );

    let html = h.update(|form, _| {
        form.field(Order::items.nth(0).then(Item::name)).clear();
    });
    assert!(
        actions_div(&html).contains("display:none"),
        "cleared field hides the actions again"
    );
}

#[test]
fn rekeyed_field_keeps_label_control_and_error_ids_aligned() {
    let mut h = Harness::new();
    let html = h.html();
    assert!(html.contains(r#"for="items.0.name""#));
    assert!(html.contains(r#"id="items.0.name""#));

    let html = h.update(|_, mut idx| idx.set(1));
    assert!(
        html.contains(r#"for="items.1.name""#),
        "label follows the re-keyed path"
    );
    assert!(
        html.contains(r#"id="items.1.name""#),
        "control id follows the re-keyed path"
    );
    assert!(!html.contains(r#"for="items.0.name""#));
    assert!(!html.contains(r#"id="items.0.name""#));

    // The error slot follows too: park a parse error on the re-keyed number field.
    let html = h.update(|form, _| {
        form.field(Order::items.nth(1).then(Item::qty))
            .commit_text("3x");
    });
    assert!(html.contains(r#"id="items.1.qty-error""#));
    assert!(html.contains(r#"aria-describedby="items.1.qty-error""#));
    assert!(html.contains(r#"aria-invalid="true""#));
}

#[test]
fn form_element_and_inline_error_markup() {
    let mut h = Harness::new();
    let html = h.html();
    assert!(!html.contains("javascript:void"), "no inline script action");
    assert!(html.contains("<fieldset"));
    assert!(
        !html.contains("aria-describedby"),
        "no describedby while valid"
    );
    assert!(!html.contains(r#"role="alert""#));

    let html = h.update(|form, _| form.field(Order::title).commit_text(""));
    assert!(html.contains(r#"role="alert""#));
    assert!(
        !html.contains("aria-live"),
        "role=alert carries its own liveness"
    );
    assert!(html.contains(r#"id="title-error""#));
    assert!(html.contains(r#"aria-describedby="title-error""#));
    assert!(html.contains("Title is required"));
}

#[test]
fn global_error_renders_alert_below_fields() {
    let mut h = Harness::new();
    let html =
        h.update(|form, _| form.set_server_error(ds_utils::DsError::Other("Server down".into())));
    assert!(html.contains("Server down"));
    let fieldset = html.find("</fieldset>").unwrap();
    let err = html.find("Server down").unwrap();
    assert!(fieldset < err, "global error renders after the fieldset");
}
