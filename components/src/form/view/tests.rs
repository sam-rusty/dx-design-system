use std::collections::HashMap;

use utils::format::{clamp_percent, format_number, format_phone, parse_number, parse_phone};

mod tooltip_render {
    use dioxus::dioxus_core::VirtualDom;
    use dioxus::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use validator::Validate;

    use crate::field_name::{Field, FieldType, FormSchema};
    use crate::form::{FormProvider, TextInput, use_form};
    use crate::{Checkbox, RadioGroup, Select, TextArea};

    #[derive(Clone, Default, Serialize, Deserialize, Validate)]
    struct Mock {
        note: String,
    }

    impl FormSchema for Mock {
        const FIELD_TYPE: FieldType = FieldType::String;
        fn json_schema() -> Value {
            serde_json::to_value(Self::default()).unwrap()
        }
    }

    fn note_field() -> Field {
        Field::new("note", "Note", false, FieldType::String)
    }

    const OPTS: &[(&str, &str)] = &[("a", "A"), ("b", "B")];

    /// Each label-bearing component, rendered once with a tooltip and once
    /// without. A successful `rebuild_in_place` proves the whole tooltip path
    /// (FormLabel/SelectItemLabel/Checkbox/RadioGroup → LabelHint → Tooltip →
    /// CircleHelp icon) mounts without panicking. The crate ships no SSR
    /// renderer, so output-string assertions aren't available here.
    fn mount(app: fn() -> Element) {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
    }

    #[test]
    fn text_input_with_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    TextInput { field: note_field(), tooltip: Some(rsx! { "help" }) }
                }
            }
        });
    }

    #[test]
    fn text_input_without_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    TextInput { field: note_field() }
                }
            }
        });
    }

    #[test]
    fn textarea_with_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    TextArea { field: note_field(), tooltip: Some(rsx! { "help" }) }
                }
            }
        });
    }

    #[test]
    fn textarea_without_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    TextArea { field: note_field() }
                }
            }
        });
    }

    #[test]
    fn select_with_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    Select { field: note_field(), options: OPTS, tooltip: Some(rsx! { "help" }) }
                }
            }
        });
    }

    #[test]
    fn select_without_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    Select { field: note_field(), options: OPTS }
                }
            }
        });
    }

    #[test]
    fn checkbox_with_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    Checkbox { field: note_field(), tooltip: Some(rsx! { "help" }) }
                }
            }
        });
    }

    #[test]
    fn checkbox_without_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    Checkbox { field: note_field() }
                }
            }
        });
    }

    #[test]
    fn radio_with_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    RadioGroup { field: note_field(), options: OPTS, tooltip: Some(rsx! { "help" }) }
                }
            }
        });
    }

    #[test]
    fn radio_without_tooltip() {
        mount(|| {
            let form = use_form::<Mock>();
            rsx! {
                FormProvider { form,
                    RadioGroup { field: note_field(), options: OPTS }
                }
            }
        });
    }
}

#[test]
fn test_format_number_empty() {
    assert_eq!(format_number(""), "");
    assert_eq!(format_number("  "), "");
}

#[test]
fn test_format_number_small() {
    assert_eq!(format_number("0"), "0");
    assert_eq!(format_number("1"), "1");
    assert_eq!(format_number("12"), "12");
    assert_eq!(format_number("123"), "123");
}

#[test]
fn test_format_number_with_commas() {
    assert_eq!(format_number("1234"), "1,234");
    assert_eq!(format_number("12345"), "12,345");
    assert_eq!(format_number("123456"), "123,456");
    assert_eq!(format_number("1234567"), "1,234,567");
    assert_eq!(format_number("1000000"), "1,000,000");
}

#[test]
fn test_format_number_negative() {
    assert_eq!(format_number("-1"), "-1");
    assert_eq!(format_number("-1234"), "-1,234");
    assert_eq!(format_number("-1234567"), "-1,234,567");
}

#[test]
fn test_format_number_decimal() {
    assert_eq!(format_number("1234.56"), "1,234.56");
    assert_eq!(format_number("1000000.99"), "1,000,000.99");
    assert_eq!(format_number("-1234.5"), "-1,234.5");
    assert_eq!(format_number("0.5"), "0.5");
}

#[test]
fn test_parse_number() {
    assert_eq!(parse_number("1,234"), "1234");
    assert_eq!(parse_number("1,234,567"), "1234567");
    assert_eq!(parse_number("1,000,000.99"), "1000000.99");
    assert_eq!(parse_number("-1,234"), "-1234");
    assert_eq!(parse_number("123"), "123");
    assert_eq!(parse_number(""), "");
}

#[test]
fn test_format_parse_number_roundtrip() {
    let raw = "1234567";
    assert_eq!(parse_number(&format_number(raw)), raw);

    let raw = "-9876543.21";
    assert_eq!(parse_number(&format_number(raw)), raw);
}

#[test]
fn test_format_phone_empty() {
    assert_eq!(format_phone(""), "");
}

#[test]
fn test_format_phone_partial() {
    assert_eq!(format_phone("5"), "+1 (5)");
    assert_eq!(format_phone("55"), "+1 (55)");
    assert_eq!(format_phone("555"), "+1 (555)");
    assert_eq!(format_phone("5551"), "+1 (555) 1");
    assert_eq!(format_phone("55512"), "+1 (555) 12");
    assert_eq!(format_phone("555123"), "+1 (555) 123");
    assert_eq!(format_phone("5551234"), "+1 (555) 123-4");
    assert_eq!(format_phone("5551234567"), "+1 (555) 123-4567");
}

#[test]
fn test_format_phone_with_country_code() {
    assert_eq!(format_phone("15551234567"), "+1 (555) 123-4567");
}

#[test]
fn test_format_phone_strips_non_digits() {
    assert_eq!(format_phone("(555) 123-4567"), "+1 (555) 123-4567");
    assert_eq!(format_phone("+1 555 123 4567"), "+1 (555) 123-4567");
}

#[test]
fn test_parse_phone() {
    assert_eq!(parse_phone("+1 (555) 123-4567"), "5551234567");
    assert_eq!(parse_phone("5551234567"), "5551234567");
    assert_eq!(parse_phone(""), "");
}

#[test]
fn test_parse_phone_strips_country_code() {
    assert_eq!(parse_phone("+1 (555) 123-4567"), "5551234567");
}

// --- PercentageInput blur-clamp logic ---
//
// This exercises pure functions; the crate has no event-dispatch harness (the
// `tooltip_render` module above can mount a component tree via `VirtualDom`, but
// cannot synthesize a `FocusEvent`), so a true blur-interaction test on
// `PercentageInput` is not possible here. Instead this mirrors
// the exact logic of the `on_blur` handler in `view/components.rs`: read the raw
// field value out of the form value map, `clamp_percent(&raw, min, max)`, and write
// the clamped value back only when it differs. The handler reads `FormContext` once
// in the render body now (P0 fix), so the observable contract under test is the
// clamp-and-write-back behavior, which this asserts against the same `clamp_percent`
// the component calls.

const FIELD: &str = "rate";

/// Reproduces the body of `PercentageInput::on_blur` over a plain value map
/// (the same shape as `FormContext.values_signal`).
fn blur_clamp(values: &mut HashMap<String, String>, min: f64, max: f64) {
    let raw = values.get(FIELD).cloned().unwrap_or_default();
    let clamped = clamp_percent(&raw, min, max);
    if clamped != raw {
        values.insert(FIELD.to_string(), clamped);
    }
}

#[test]
fn test_percentage_blur_clamps_above_max() {
    let mut values = HashMap::new();
    values.insert(FIELD.to_string(), "150".to_string());
    blur_clamp(&mut values, 0.0, 100.0);
    assert_eq!(values.get(FIELD).map(String::as_str), Some("100"));
}

#[test]
fn test_percentage_blur_clamps_below_min() {
    let mut values = HashMap::new();
    values.insert(FIELD.to_string(), "-5".to_string());
    blur_clamp(&mut values, 0.0, 100.0);
    assert_eq!(values.get(FIELD).map(String::as_str), Some("0"));
}

#[test]
fn test_percentage_blur_in_range_unchanged() {
    let mut values = HashMap::new();
    values.insert(FIELD.to_string(), "42".to_string());
    blur_clamp(&mut values, 0.0, 100.0);
    // Within range → handler must NOT rewrite the value (clamped == raw).
    assert_eq!(values.get(FIELD).map(String::as_str), Some("42"));
}

#[test]
fn test_percentage_blur_custom_range() {
    let mut values = HashMap::new();
    values.insert(FIELD.to_string(), "5".to_string());
    blur_clamp(&mut values, 10.0, 90.0);
    assert_eq!(values.get(FIELD).map(String::as_str), Some("10"));
}

#[test]
fn test_percentage_blur_empty_value_stays_empty() {
    let mut values = HashMap::new();
    values.insert(FIELD.to_string(), String::new());
    blur_clamp(&mut values, 0.0, 100.0);
    assert_eq!(values.get(FIELD).map(String::as_str), Some(""));
}
