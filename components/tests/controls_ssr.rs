//! SSR pins for the inputs & controls family: static markup that consumers
//! and assistive tech depend on (closed-state Select a11y, `Label` `for`
//! omission, FileUpload label/input wiring, roving tabindex on RadioGroup,
//! InputBase attribute pass-through).

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "form")]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use components::form::dynamic::{Form, FormProvider, use_dynamic_form};
use components::{
    CheckboxBase, Field, FieldType, FileInfo, FileUpload, FormSchema, InputBase, Label, Radio,
    RadioGroup, SelectBase, SelectOption, Switch, use_select_contexts,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Value of the first `attr="..."` occurrence after `after` in `html`.
fn attr_after<'a>(html: &'a str, after: &str, attr: &str) -> Option<&'a str> {
    let start = html.find(after)?;
    let rest = &html[start..];
    let key = format!(r#"{attr}=""#);
    let pos = rest.find(&key)? + key.len();
    let end = rest[pos..].find('"')?;
    Some(&rest[pos..pos + end])
}

// ---------------------------------------------------------------------------
// Select (B1): closed listbox must be out of the tab order and the a11y tree.
// ---------------------------------------------------------------------------

fn select_app() -> Element {
    let value = use_signal(|| "b".to_string());
    let on_change = use_callback(move |_v: String| {});
    use_select_contexts(value.into(), on_change, false, 0, false);
    rsx! {
        SelectBase { aria_label: "Letter",
            SelectOption { value: "a", label: "Alpha" }
            SelectOption { value: "b", label: "Bravo" }
        }
    }
}

fn multi_select_app() -> Element {
    let value = use_signal(|| r#"["a","b"]"#.to_string());
    let on_change = use_callback(move |_v: String| {});
    use_select_contexts(value.into(), on_change, false, 0, true);
    rsx! {
        SelectBase { multiple: true, aria_label: "Letters",
            SelectOption { value: "a", label: "Alpha" }
            SelectOption { value: "b", label: "Bravo" }
        }
    }
}

#[test]
fn closed_select_options_are_not_tab_stops() {
    let html = render(select_app);
    let option_count = html.matches(r#"role="option""#).count();
    assert_eq!(option_count, 2);
    for (i, _) in html.match_indices(r#"role="option""#) {
        let tag_end = html[i..].find('>').unwrap();
        let tag = &html[i..i + tag_end];
        assert!(
            tag.contains(r#"tabindex="-1""#),
            "option must carry tabindex=-1 (activedescendant pattern): {tag}"
        );
    }
}

#[test]
fn closed_select_listbox_is_hidden_from_the_a11y_tree() {
    let html = render(select_app);
    let start = html.find(r#"role="listbox""#).unwrap();
    // Walk back to the opening `<div` of the listbox element.
    let tag_start = html[..start].rfind("<div").unwrap();
    let tag_end = start + html[start..].find('>').unwrap();
    let tag = &html[tag_start..tag_end];
    assert!(
        tag.contains("invisible"),
        "closed listbox uses visibility:hidden: {tag}"
    );
    assert!(
        tag.contains(r#"aria-hidden="true""#),
        "closed listbox is aria-hidden: {tag}"
    );
    assert!(
        tag.contains("opacity-0"),
        "closed appearance retained: {tag}"
    );
    assert!(tag.contains("pointer-events-none"));
}

#[test]
fn select_trigger_shows_selected_label_and_aria_state() {
    let html = render(select_app);
    assert!(html.contains(r#"role="combobox""#));
    assert!(html.contains(r#"aria-expanded="false""#));
    assert!(html.contains(r#"aria-multiselectable="false""#));
    // The selected value's label is displayed, not its raw value.
    assert!(html.contains(">Bravo<"));
    assert!(html.contains(r#"aria-selected="true""#));
}

#[test]
fn multi_select_renders_tags_without_nested_interactive_elements() {
    let html = render(multi_select_app);
    assert!(html.contains(r#"aria-multiselectable="true""#));
    assert!(html.contains("Alpha"));
    assert!(html.contains("Bravo"));
    // The trigger is a <button>; nothing focusable may nest inside it.
    let trigger = html.find(r#"role="combobox""#).unwrap();
    let trigger_end = trigger + html[trigger..].find("</button>").unwrap();
    let inner = &html[trigger..trigger_end];
    assert!(
        !inner.contains(r#"role="button""#) && !inner.contains(r#"tabindex="0""#),
        "no interactive descendant inside the trigger button"
    );
}

// ---------------------------------------------------------------------------
// Label (B6): no `for=""`.
// ---------------------------------------------------------------------------

fn label_app() -> Element {
    rsx! {
        Label { "Wrapping"
            CheckboxBase {}
        }
        Label { html_for: "target", "Pointing" }
    }
}

#[test]
fn label_omits_for_when_no_target() {
    let html = render(label_app);
    assert!(
        !html.contains(r#"for="""#),
        "empty for attribute breaks wrap association"
    );
    assert!(html.contains(r#"for="target""#));
}

// ---------------------------------------------------------------------------
// FileUpload (B3): label targets the hidden input.
// ---------------------------------------------------------------------------

fn file_upload_app() -> Element {
    rsx! {
        FileUpload { on_change: move |_f: Vec<FileInfo>| {} }
    }
}

#[test]
fn file_upload_label_targets_its_input() {
    let html = render(file_upload_app);
    let input_id = attr_after(&html, r#"type="file""#, "id")
        .or_else(|| attr_after(&html, "<input", "id"))
        .expect("input has an id");
    let label_for = attr_after(&html, "<label", "for").expect("label has for");
    assert_eq!(input_id, label_for);
    assert!(!html.contains(r#"accept="""#), "accept omitted when unset");
}

// ---------------------------------------------------------------------------
// RadioGroup: roving tabindex + ids.
// ---------------------------------------------------------------------------

#[derive(Clone, Default, Serialize, Deserialize, Validate)]
struct Signup {
    plan: String,
}

impl FormSchema for Signup {
    const FIELD_TYPE: FieldType = FieldType::String;
    fn json_schema() -> Value {
        serde_json::to_value(Self::default()).unwrap()
    }
}

fn radio_app() -> Element {
    let form = use_dynamic_form::<Signup>();
    let action = use_action(|_: Signup| async { Ok::<(), dioxus::CapturedError>(()) });
    rsx! {
        FormProvider { form, action,
            Form {
                RadioGroup {
                    field: Field::new("plan", "Plan", true, FieldType::String),
                    options: &[("free", "Free"), ("pro", "Pro")],
                }
            }
        }
    }
}

#[test]
fn radio_group_has_single_tab_stop() {
    let html = render(radio_app);
    assert!(html.contains(r#"role="radiogroup""#));
    assert_eq!(html.matches(r#"role="radio""#).count(), 2);
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 1);
    assert_eq!(html.matches(r#"tabindex="-1""#).count(), 1);
    assert!(html.contains(r#"id="plan-free""#));
    assert!(html.contains(r#"for="plan-free""#));
    // Nothing selected yet → the first item is the tab stop.
    let free = html.find(r#"id="plan-free""#).unwrap();
    let tag_start = html[..free].rfind("<button").unwrap();
    let tag_end = free + html[free..].find('>').unwrap();
    let tag = &html[tag_start..tag_end];
    assert!(tag.contains(r#"tabindex="0""#));
    assert!(!html.contains(r#"for="""#));
}

// ---------------------------------------------------------------------------
// InputBase / Checkbox / Switch attribute pass-through.
// ---------------------------------------------------------------------------

fn input_app() -> Element {
    rsx! {
        InputBase {
            id: "email",
            aria_invalid: "true",
            aria_describedby: "email-error",
            disabled: true,
            name: "email",
        }
        InputBase { id: "plain" }
        CheckboxBase { id: "cb", aria_labelledby: "cb-label", default_checked: true }
        Switch { checked: false, id: "sw", "aria-label": "Notifications" }
        Radio { value: "x", id: "rd", "aria-label": "Option X" }
    }
}

#[test]
fn input_base_passes_through_ids_and_aria() {
    let html = render(input_app);
    assert!(html.contains(r#"id="email""#));
    assert!(html.contains(r#"aria-invalid="true""#));
    assert!(html.contains(r#"aria-describedby="email-error""#));
    assert!(html.contains(r#"name="email""#));
    assert!(html.contains("disabled"));
    // No trailing adornment → no `relative` wrapper.
    assert!(!html.contains(r#"class="relative w-full""#));
}

#[test]
fn checkbox_and_switch_expose_roles_and_names() {
    let html = render(input_app);
    assert!(html.contains(r#"role="checkbox""#));
    assert!(html.contains(r#"aria-checked="true""#));
    assert!(html.contains(r#"aria-labelledby="cb-label""#));
    assert!(html.contains(r#"role="switch""#));
    assert!(html.contains(r#"id="sw""#));
    assert!(html.contains(r#"aria-label="Notifications""#));
    assert!(html.contains(r#"id="rd""#));
    assert!(html.contains(r#"aria-label="Option X""#));
}
