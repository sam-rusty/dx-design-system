use dioxus::prelude::*;
use ds_macros::on_web;
use ds_utils::format::merge;

use crate::field_name::Field;
use crate::form::use_field_binding;
use crate::form::view::FormFieldFrame;
use crate::hooks::use_controlled;
use crate::input::FieldSize;

on_web! {
    mod js {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(inline_js = r#"
            export function textareaInsertAtCursor(elementId, text) {
                const el = document.getElementById(elementId);
                if (!el) return;
                const start = el.selectionStart ?? el.value.length;
                const end = el.selectionEnd ?? el.value.length;
                el.value = el.value.slice(0, start) + text + el.value.slice(end);
                const caret = start + text.length;
                el.focus();
                el.setSelectionRange(caret, caret);
                el.dispatchEvent(new Event('input', { bubbles: true }));
            }
        "#)]
        extern "C" {
            pub fn textareaInsertAtCursor(element_id: &str, text: &str);
        }
    }
}

/// Insert plain text at the current caret of the textarea with the given `id`,
/// dispatching an `input` event so a bound value signal stays in sync. No-op on SSR.
#[cfg(feature = "web")]
pub fn textarea_insert_at_cursor(element_id: &str, text: &str) {
    js::textareaInsertAtCursor(element_id, text);
}

#[cfg(not(feature = "web"))]
pub fn textarea_insert_at_cursor(_element_id: &str, _text: &str) {}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextAreaResize {
    #[default]
    Vertical,
    Horizontal,
    Both,
    None,
}

impl TextAreaResize {
    pub fn as_class(self) -> &'static str {
        match self {
            Self::Vertical => "resize-y",
            Self::Horizontal => "resize-x",
            Self::Both => "resize",
            Self::None => "resize-none",
        }
    }
}

const TEXTAREA_BASE: &str = "peer flex w-full min-h-[80px] min-w-0 rounded-lg border border-input \
     bg-transparent text-foreground transition-all duration-200 outline-none \
     placeholder:text-transparent focus:placeholder:text-muted-foreground/70 focus:border-primary \
     focus:ring-1 focus:ring-primary disabled:pointer-events-none disabled:cursor-not-allowed \
     disabled:opacity-50 disabled:bg-muted/50 aria-invalid:border-destructive \
     aria-invalid:ring-destructive/20 focus:aria-invalid:border-destructive \
     focus:aria-invalid:ring-1 read-only:bg-muted/50";

const TEXTAREA_FORM_BASE: &str = "peer block w-full min-h-[80px] appearance-none rounded-lg border \
     border-input bg-transparent text-foreground transition-colors \
     focus-visible:border-primary focus-visible:outline-none focus-visible:ring-1 \
     focus-visible:ring-primary aria-invalid:border-destructive aria-invalid:ring-destructive/20";

/// Padding + text classes for each [`FieldSize`] on a textarea.
fn size_classes(size: FieldSize) -> &'static str {
    match size {
        FieldSize::Default => "px-4 py-2 text-sm",
        FieldSize::Sm => "px-3 py-1.5 text-xs",
        FieldSize::Xs => "px-2 py-1 text-xs",
    }
}

/// Props for [`TextAreaBase`].
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaBaseProps {
    /// Controlled value. `Some` makes the caller the source of truth (pair
    /// with `on_value_change`); `None` leaves the textarea uncontrolled.
    #[props(default)]
    pub value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    pub default_value: String,
    /// Fired with the new value on every input event.
    #[props(default)]
    pub on_value_change: Callback<String>,
    /// Fired with the committed value on the change event (blur).
    #[props(default)]
    pub on_commit: Callback<String>,
    /// Fired when the textarea loses focus.
    #[props(default)]
    pub on_blur: Callback<FocusEvent>,
    /// Fired on keydown.
    #[props(default)]
    pub on_key_down: Callback<KeyboardEvent>,
    /// Whether the textarea is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Resize handle behavior.
    #[props(default)]
    pub resize: TextAreaResize,
    /// Extra classes merged into the base style; the full class list when `unstyled`.
    #[props(default)]
    pub class: String,
    /// Placeholder text.
    #[props(default)]
    pub placeholder: Option<String>,
    /// DOM id. Form bindings set this to the field name so labels target it.
    #[props(default)]
    pub id: Option<String>,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Skip the built-in styling entirely; `class` is used verbatim.
    #[props(default)]
    pub unstyled: bool,
    /// `aria-invalid` value (form bindings set `"true"` on validation failure).
    #[props(default)]
    pub aria_invalid: Option<String>,
    /// `aria-describedby` target (the field's error element id).
    #[props(default)]
    pub aria_describedby: Option<String>,
    /// Additional attributes (`rows`, `cols`, `minlength`, `maxlength`,
    /// `name`, `readonly`, `required`, `autofocus`, ...).
    #[props(extends = GlobalAttributes, extends = textarea)]
    pub attributes: Vec<Attribute>,
}

pub fn TextAreaBase(props: TextAreaBaseProps) -> Element {
    let merged_class = if props.unstyled {
        props.class.clone()
    } else {
        merge(&[
            TEXTAREA_BASE,
            props.resize.as_class(),
            size_classes(props.size),
            &props.class,
        ])
    };

    let (value, set_value) = use_controlled(
        props.value,
        props.default_value.clone(),
        props.on_value_change,
    );

    let actual_placeholder = props.placeholder.clone().unwrap_or_default();
    let on_commit = props.on_commit;
    let on_blur = props.on_blur;
    let on_key_down = props.on_key_down;
    let disabled = props.disabled;

    rsx! {
        textarea {
            "data-name": "TextArea",
            class: "{merged_class}",
            placeholder: actual_placeholder,
            id: props.id.clone(),
            autofocus: props.autofocus,
            disabled: disabled(),
            "aria-invalid": props.aria_invalid.clone(),
            "aria-describedby": props.aria_describedby.clone(),
            value: value(),
            oninput: move |ev| set_value(ev.value()),
            onchange: move |ev: FormEvent| on_commit.call(ev.value()),
            onblur: move |ev| on_blur.call(ev),
            onkeydown: move |ev| on_key_down.call(ev),
            ..props.attributes,
        }
    }
}

/// Form-context binding for [`TextAreaBase`].
#[component]
pub(crate) fn TextAreaControl(
    #[props(default)] autofocus: bool,
    #[props(default)] rows: Option<u32>,
    #[props(default)] cols: Option<u32>,
    #[props(default)] minlength: Option<u32>,
    #[props(default)] maxlength: Option<u32>,
    #[props(default)] size: FieldSize,
    #[props(default)] resize: TextAreaResize,
) -> Element {
    let binding = use_field_binding();

    let form_class = merge(&[TEXTAREA_FORM_BASE, resize.as_class(), size_classes(size)]);

    let touch = binding.touch;

    rsx! {
        TextAreaBase {
            id: binding.id.clone(),
            class: form_class,
            unstyled: true,
            value: binding.controlled_value,
            on_value_change: binding.on_value_change,
            on_commit: binding.on_commit,
            on_blur: move |_: FocusEvent| touch.call(()),
            disabled: ReadSignal::from(binding.disabled),
            aria_invalid: binding.aria_invalid(),
            aria_describedby: binding.aria_describedby.clone(),
            autofocus,
            rows,
            cols,
            minlength,
            maxlength,
        }
    }
}

/// Props for [`TextArea`], the form-bound textarea.
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Autofocus on mount.
    #[props(default)]
    pub autofocus: bool,
    /// Visible rows.
    #[props(default)]
    pub rows: Option<u32>,
    /// Visible columns.
    #[props(default)]
    pub cols: Option<u32>,
    /// Minimum accepted length.
    #[props(default)]
    pub minlength: Option<u32>,
    /// Maximum accepted length.
    #[props(default)]
    pub maxlength: Option<u32>,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Resize handle behavior.
    #[props(default)]
    pub resize: TextAreaResize,
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
}

/// Form-bound textarea with stacked label and inline error.
pub fn TextArea(props: TextAreaProps) -> Element {
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            class: props.class,
            TextAreaControl {
                autofocus: props.autofocus,
                rows: props.rows,
                cols: props.cols,
                minlength: props.minlength,
                maxlength: props.maxlength,
                size: props.size,
                resize: props.resize,
            }
        }
    }
}
