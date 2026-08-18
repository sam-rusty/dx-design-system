//! SSR regression tests for the stacked-label form family: every form-bound
//! field renders a static `FieldLabel` above a bordered control — no floating
//! label, no notch fieldset.

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use ds_components::form::dynamic::PasswordInput;
use ds_components::form::dynamic::{FormProvider, TextInput, use_dynamic_form};
use ds_components::{Field, FieldType, FormSchema};

#[derive(Clone, Default, Serialize, Deserialize, Validate)]
struct LoginForm {
    username: String,
    password: String,
}

impl FormSchema for LoginForm {
    const FIELD_TYPE: FieldType = FieldType::String;
    fn json_schema() -> Value {
        serde_json::to_value(Self::default()).unwrap()
    }
}

fn app() -> Element {
    let form = use_dynamic_form::<LoginForm>();
    let action = use_action(|_: LoginForm| async { Ok::<(), dioxus::CapturedError>(()) });
    rsx! {
        FormProvider { form, action,
            ds_components::form::dynamic::Form {
                TextInput { field: Field::new("username", "Username", true, FieldType::String) }
                PasswordInput { field: Field::new("password", "Password", true, FieldType::String) }
            }
        }
    }
}

fn render_login() -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn form_inputs_render_stacked_label_above_control() {
    let html = render_login();

    assert!(html.contains(r#"data-name="FieldLabel""#));
    assert!(html.contains(r#"for="username""#));
    assert!(html.contains(">Username<"));

    // The label element precedes its input in the DOM (stacked above).
    let label_pos = html.find(r#"for="username""#).unwrap();
    let input_pos = html.find(r#"id="username""#).unwrap();
    assert!(label_pos < input_pos);

    // The control owns its border again.
    assert!(html.contains("border-[color:var(--field-border-color)]"));
}

#[test]
fn floating_label_machinery_is_gone() {
    let html = render_login();

    assert!(!html.contains("FloatingLabel"));
    assert!(!html.contains("FieldOutline"));
    assert!(!html.contains("field-outline"));
    assert!(!html.contains("field-frame"));
    assert!(!html.contains("peer-placeholder-shown"));

    // Regression: the standalone base must not be merged under the form class
    // (double borders / conflicting focus styles).
    assert!(!html.contains("read-only:bg-muted/50 peer block"));
}
