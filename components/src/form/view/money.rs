//! Form-bound money input: the form stores minor units (i64-compatible
//! string), the admin types major units. `decimals` is the minor-unit
//! exponent (2 for USD cents, 0 for zero-decimal currencies).

use dioxus::prelude::*;
use ds_utils::format::{major_to_minor, minor_to_major};

use crate::field_name::Field;
use crate::input::FieldSize;
use crate::input_types::{NumberInputBase, TypedInputBaseProps};

use super::binding::use_field_binding;
use super::components::FormFieldFrame;

/// Props for [`MoneyInput`], the form-bound money input.
#[derive(Props, Clone, PartialEq)]
pub struct MoneyInputProps {
    /// The bound form field (stores minor units).
    #[props(into)]
    pub field: Field,
    /// Minor-unit exponent of the currency (2 → cents, 0 → zero-decimal).
    pub decimals: u32,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
}

/// Form-bound money input: major-unit display, minor-unit form value.
pub fn MoneyInput(props: MoneyInputProps) -> Element {
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            class: props.class,
            MoneyControl {
                decimals: props.decimals,
                size: props.size,
                autofocus: props.autofocus,
            }
        }
    }
}

#[component]
fn MoneyControl(
    decimals: u32,
    #[props(default)] size: FieldSize,
    #[props(default)] autofocus: bool,
) -> Element {
    let binding = use_field_binding();

    // Raw text while the admin is typing; None = mirror the (converted) store.
    let mut editing = use_signal(|| None::<String>);
    let stored = binding.controlled_value;
    let display: ReadSignal<Option<String>> = use_memo(move || {
        Some(match editing() {
            Some(text) => text,
            None => stored()
                .map(|m| minor_to_major(&m, decimals))
                .unwrap_or_default(),
        })
    })
    .into();

    let commit = binding.on_commit;
    let touch = binding.touch;
    let on_value_change = Callback::new(move |v: String| editing.set(Some(v)));
    let on_commit = Callback::new(move |v: String| {
        commit.call(major_to_minor(&v, decimals).unwrap_or_default());
        editing.set(None);
    });
    let on_blur = Callback::new(move |_: FocusEvent| touch.call(()));

    let base = TypedInputBaseProps {
        value: display,
        default_value: String::new(),
        on_value_change,
        on_commit,
        on_blur,
        on_key_down: Callback::default(),
        disabled: binding.disabled.into(),
        size,
        class: size.form_control_merge(false),
        placeholder: None,
        id: Some(binding.id.clone()),
        autofocus,
        unstyled: true,
        aria_invalid: binding.aria_invalid(),
        aria_describedby: Some(binding.aria_describedby.clone()),
        attributes: Vec::new(),
    };
    rsx! {
        NumberInputBase { ..base }
    }
}
