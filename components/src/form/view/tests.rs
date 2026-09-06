mod tooltip_render {
    use dioxus::dioxus_core::VirtualDom;
    use dioxus::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use validator::Validate;

    use crate::field_name::{Field, FieldType, FormSchema};
    use crate::form::dynamic::{FormProvider, TextInput, use_dynamic_form};
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
    /// (FieldLabel/SelectItemLabel/Checkbox/RadioGroup → LabelHint → Tooltip →
    /// CircleHelp icon) mounts without panicking. The crate ships no SSR
    /// renderer, so output-string assertions aren't available here.
    fn mount(app: fn() -> Element) {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
    }

    #[test]
    fn text_input_with_tooltip() {
        mount(|| {
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
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
            let form = use_dynamic_form::<Mock>();
            rsx! {
                FormProvider { form,
                    RadioGroup { field: note_field(), options: OPTS }
                }
            }
        });
    }
}
