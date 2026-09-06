//! SSR pins for the date-picker family: closed-trigger markup consumers style
//! against, deterministic `utc` rendering on the server (D3), and the
//! label/trigger wiring inside a typed form (D5).

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "date-picker")]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use validator::Validate;

use components::form::{DatePicker, DateTimePicker, Form, FormProvider, use_form};
use components::{DatePickerBase, DateTimePickerBase, FormFields};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The opening tag of the first element after `after` in `html`.
fn tag_after<'a>(html: &'a str, after: &str) -> &'a str {
    let start = html
        .find(after)
        .unwrap_or_else(|| panic!("{after} not found in {html}"));
    let open = html[..start].rfind('<').unwrap_or(0);
    let end = start + html[start..].find('>').unwrap();
    &html[open..=end]
}

// ---------------------------------------------------------------------------
// DatePickerBase, closed
// ---------------------------------------------------------------------------

fn empty_picker() -> Element {
    rsx! { DatePickerBase {} }
}

fn valid_picker() -> Element {
    let value = use_signal(|| Some("2026-03-10".to_string()));
    rsx! { DatePickerBase { value } }
}

fn invalid_picker() -> Element {
    let value = use_signal(|| Some("not a date".to_string()));
    let disabled = use_signal(|| true);
    rsx! { DatePickerBase { value, disabled } }
}

#[test]
fn closed_date_picker_markup() {
    let html = render(empty_picker);
    assert!(html.contains(r#"data-name="DatePicker""#));
    let trigger = tag_after(&html, r#"data-state="closed""#);
    assert!(trigger.starts_with("<button"), "{trigger}");
    assert!(trigger.contains(r#"aria-haspopup="dialog""#), "{trigger}");
    assert!(trigger.contains("aria-expanded=false"), "{trigger}");
    // (`aria-invalid:` in the class string is a Tailwind variant, not the attribute.)
    assert!(
        !trigger.contains(r#"aria-invalid=""#),
        "clean picker: {trigger}"
    );
    assert!(!trigger.contains(" id="), "no id unless given: {trigger}");
    // Empty value holds the height with a non-breaking space.
    assert!(
        html.contains("\u{00A0}") || html.contains("&nbsp;"),
        "{html}"
    );
    // The dialog is not part of the server output.
    assert!(!html.contains(r#"role="dialog""#));
}

#[test]
fn closed_date_picker_formats_valid_and_echoes_invalid_values() {
    let html = render(valid_picker);
    assert!(html.contains("03/10/2026"), "{html}");

    let html = render(invalid_picker);
    assert!(html.contains("not a date"), "{html}");
    let trigger = tag_after(&html, r#"data-state="closed""#);
    assert!(trigger.contains("disabled=true"), "{trigger}");
}

// ---------------------------------------------------------------------------
// DateTimePickerBase { utc: true } renders UTC wall time on the server (D3)
// ---------------------------------------------------------------------------

fn utc_picker() -> Element {
    let value = use_signal(|| Some("2026-07-19T04:30:00Z".to_string()));
    rsx! { DateTimePickerBase { value, utc: true } }
}

#[test]
fn utc_date_time_picker_renders_utc_wall_time_on_the_server() {
    let html = render(utc_picker);
    assert!(html.contains(r#"data-name="DateTimePicker""#));
    assert!(html.contains("07/19/2026, 04:30 AM"), "{html}");
}

// ---------------------------------------------------------------------------
// Typed form: label `for` targets the trigger `id` (D5)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Booking {
    start_date: String,
    starts_at: Option<time::OffsetDateTime>,
}

fn booking_form() -> Element {
    let form = use_form::<Booking>();
    let action = use_action(|_: Booking| async { Ok::<(), dioxus::CapturedError>(()) });
    use_hook(move || {
        form.default_values(Booking {
            start_date: "2026-03-10".into(),
            starts_at: Some(time::macros::datetime!(2026-07-19 04:30 UTC)),
        });
    });
    rsx! {
        FormProvider { form, action,
            Form {
                DatePicker { field: Booking::start_date }
                DateTimePicker { field: Booking::starts_at, utc: true }
            }
        }
    }
}

#[test]
fn typed_pickers_wire_label_to_trigger_id() {
    let html = render(booking_form);
    for name in ["start_date", "starts_at"] {
        assert!(
            html.contains(&format!(r#"for="{name}""#)),
            "label for {name}: {html}"
        );
        let trigger = tag_after(&html, &format!(r#"id="{name}""#));
        assert!(
            trigger.starts_with("<button"),
            "trigger carries the id: {trigger}"
        );
        assert!(
            !trigger.contains(r#"aria-invalid=""#) && !trigger.contains("aria-describedby"),
            "clean field emits neither: {trigger}"
        );
    }
    assert!(html.contains("03/10/2026"), "{html}");
    assert!(html.contains("07/19/2026, 04:30 AM"), "{html}");
}
