//! SSR smoke tests for the typed form store's view layer: lens-bound fields
//! render through the shared control family (labels, ids, controlled values)
//! without the legacy string-map form.

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use validator::Validate;

use components::form::{
    Checkbox, Form, FormProvider, LensExt, NumberInput, Select, TextInput, use_form,
};
use components::{FormFields, FormOptions};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FormOptions)]
#[serde(rename_all = "lowercase")]
enum Plan {
    Free,
    Pro,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Profile {
    #[validate(length(min = 1))]
    username: String,
    age: Option<i32>,
    newsletter: bool,
    plan: Option<Plan>,
    #[validate(nested)]
    address: Address,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Address {
    city: String,
}

fn app() -> Element {
    let form = use_form::<Profile>();
    use_hook(move || {
        form.default_values(Profile {
            username: "ada".into(),
            age: None,
            newsletter: true,
            plan: Some(Plan::Pro),
            address: Address {
                city: "London".into(),
            },
        });
    });

    rsx! {
        FormProvider { form,
            Form {
                TextInput { field: Profile::username }
                NumberInput { field: Profile::age }
                Checkbox { field: Profile::newsletter }
                Select { field: Profile::plan, options: Plan::OPTIONS }
                TextInput { field: form.field(Profile::address.then(Address::city)) }
            }
        }
    }
}

fn render_app() -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn typed_fields_render_labels_and_ids() {
    let html = render_app();

    assert!(
        html.contains(r#"for="username""#),
        "label targets field path"
    );
    assert!(html.contains(r#"id="username""#));
    assert!(html.contains(">Username<"));
    assert!(html.contains(">Age<"));
    assert!(html.contains(">Newsletter<"));

    // Nested lens path becomes the control id.
    assert!(html.contains(r#"id="address.city""#));
    assert!(html.contains(">City<"));
}

#[test]
fn typed_fields_render_prefilled_values() {
    let html = render_app();

    assert!(html.contains(r#"value="ada""#), "written string shows");
    assert!(
        html.contains(r#"value="London""#),
        "nested written string shows"
    );
    // `age: None` renders blank, not "0".
    assert!(
        !html.contains(r#"value="0""#),
        "pristine/None number stays blank"
    );
}

#[test]
fn typed_select_and_checkbox_bind() {
    let html = render_app();

    // Select shows the label of the serde-named stored value.
    assert!(html.contains("Pro"));
    // Checkbox reflects the typed bool.
    assert!(html.contains(r#"aria-checked="true""#) || html.contains("data-state=\"checked\""));
}
