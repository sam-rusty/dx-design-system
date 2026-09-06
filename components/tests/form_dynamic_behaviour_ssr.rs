//! SSR pins for the dynamic (string-map) form: shared `FieldActions` /
//! `FormError` / form element render the same markup as the typed flavour,
//! and a re-keyed `FormField` re-provides its name context.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "form")]

use std::cell::RefCell;

use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use validator::Validate;

use components::form::FieldLabel;
use components::form::dynamic::{
    DynamicForm, Form, FormError, FormField, FormProvider, TextInput, use_dynamic_form,
};
use components::{Field, FieldType, FormFields};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Note {
    #[validate(length(min = 1, message = "Title is required"))]
    title: String,
    body: String,
}

type Handles = (DynamicForm<Note>, Signal<&'static str>);

thread_local! {
    static HANDLES: RefCell<Option<Handles>> = const { RefCell::new(None) };
}

fn app() -> Element {
    let form = use_dynamic_form::<Note>();
    let name = use_signal(|| "body");
    use_hook(move || HANDLES.with(|h| *h.borrow_mut() = Some((form, name))));
    rsx! {
        FormProvider { form,
            Form {
                TextInput { field: Note::title, clearable: true }
                // Re-keyable field: the `FormField` instance stays, its name changes.
                FormField { field: Field::new(name(), "Dynamic", false, FieldType::String),
                    FieldLabel { "Dynamic" }
                    FormError {}
                }
            }
        }
    }
}

struct Harness {
    dom: VirtualDom,
    form: DynamicForm<Note>,
    name: Signal<&'static str>,
}

impl Harness {
    fn new() -> Self {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let (form, name) = HANDLES.with(|h| h.borrow_mut().take()).unwrap();
        Self { dom, form, name }
    }

    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }

    fn update(&mut self, f: impl FnOnce(DynamicForm<Note>, Signal<&'static str>)) -> String {
        let (form, name) = (self.form, self.name);
        self.dom.in_runtime(|| f(form, name));
        self.dom.render_immediate(&mut NoOpMutations);
        self.html()
    }
}

fn actions_div(html: &str) -> &str {
    let start = html
        .find("absolute end-2 top-1/2")
        .expect("FieldActions container rendered");
    let tag_start = html[..start].rfind("<div").unwrap();
    let end = html[tag_start..].find('>').unwrap();
    &html[tag_start..tag_start + end + 1]
}

#[test]
fn field_actions_follow_the_value_map() {
    let mut h = Harness::new();
    assert!(actions_div(&h.html()).contains("display:none"));
    let html = h.update(|form, _| form.set_string_value("title", "x".into()));
    assert!(!actions_div(&html).contains("display:none"));
    let html = h.update(|form, _| form.set_string_value("title", String::new()));
    assert!(actions_div(&html).contains("display:none"));
}

#[test]
fn form_element_and_inline_error_markup() {
    let mut h = Harness::new();
    let html = h.html();
    assert!(!html.contains("javascript:void"));
    assert!(html.contains("<fieldset"));
    assert!(
        !html.contains("aria-describedby"),
        "no describedby while valid"
    );
    assert!(!html.contains(r#"role="alert""#));

    let html = h.update(|form, _| form.touch_field("title"));
    assert!(html.contains(r#"role="alert""#));
    assert!(!html.contains("aria-live"));
    assert!(html.contains(r#"id="title-error""#));
    assert!(html.contains(r#"aria-describedby="title-error""#));
    assert!(html.contains(r#"aria-invalid="true""#));
    assert!(html.contains("Title is required"));
}

#[test]
fn rekeyed_form_field_reprovides_its_name() {
    let mut h = Harness::new();
    assert!(h.html().contains(r#"for="body""#));

    let html = h.update(|form, mut name| {
        name.set("title");
        form.touch_field("title");
    });
    assert!(
        html.contains(r#"for="title""#),
        "label reads the fresh context"
    );
    assert!(!html.contains(r#"for="body""#));
    // The re-keyed FormError now reports the new field's error.
    assert!(html.matches(r#"id="title-error""#).count() >= 1);
}
