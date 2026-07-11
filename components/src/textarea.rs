use dioxus::prelude::*;
use macros::on_web;
use utils::format::merge;

use crate::field_name::Field;
use crate::form::{FieldContext, FormContext, FormError, FormField, FormLabel};

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
     bg-transparent px-4 py-2 text-sm text-foreground transition-all duration-200 outline-none \
     placeholder:text-transparent focus:placeholder:text-muted-foreground/70 focus:border-primary \
     focus:ring-1 focus:ring-primary disabled:pointer-events-none disabled:cursor-not-allowed \
     disabled:opacity-50 disabled:bg-muted/50 aria-invalid:border-destructive \
     aria-invalid:ring-destructive/20 focus:aria-invalid:border-destructive \
     focus:aria-invalid:ring-1 read-only:bg-muted/50";

const TEXTAREA_FORM_BASE: &str = "peer block w-full min-h-[80px] appearance-none rounded-lg border \
     border-input bg-transparent px-4 py-2 text-sm text-foreground transition-colors \
     focus-visible:border-primary focus-visible:outline-none focus-visible:ring-1 \
     focus-visible:ring-primary data-[invalid=true]:border-destructive";

#[component]
pub fn TextAreaBase(
    #[props(default)] class: String,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] title: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] required: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] rows: Option<u32>,
    #[props(default)] cols: Option<u32>,
    #[props(default)] minlength: Option<u32>,
    #[props(default)] maxlength: Option<u32>,
    #[props(default)] resize: TextAreaResize,
    #[props(default)] value: Option<Signal<String>>,
    #[props(default)] static_value: Option<String>,
    #[props(default)] on_change: Option<EventHandler<String>>,
    #[props(default)] onchange: Option<EventHandler<FormEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] aria_invalid: Option<String>,
    #[props(default)] aria_describedby: Option<String>,
) -> Element {
    let merged_class = merge(&[TEXTAREA_BASE, resize.as_class(), &class]);

    let actual_placeholder = placeholder.as_deref().unwrap_or(" ");
    let current_value = value.map(|s| s()).or(static_value).unwrap_or_default();

    rsx! {
        textarea {
            "data-name": "TextArea",
            class: "{merged_class}",
            placeholder: actual_placeholder,
            name: name,
            id: id,
            title: title,
            disabled: disabled,
            readonly: readonly,
            required: required,
            autofocus: autofocus,
            rows: rows,
            cols: cols,
            minlength: minlength,
            maxlength: maxlength,
            "aria-invalid": aria_invalid,
            "aria-describedby": aria_describedby,
            value: "{current_value}",
            oninput: move |ev| {
                if let Some(mut signal) = value {
                    signal.set(ev.value());
                }
                if let Some(handler) = &on_change {
                    handler.call(ev.value());
                }
            },
            onchange: move |ev| {
                if let Some(handler) = &onchange {
                    handler.call(ev);
                }
            },
            onblur: move |ev| {
                if let Some(handler) = &onblur {
                    handler.call(ev);
                }
            },
            onkeydown: move |ev| {
                if let Some(handler) = &onkeydown {
                    handler.call(ev);
                }
            },
        }
    }
}

#[component]
pub(crate) fn TextAreaFormControl(
    #[props(default)] class: String,
    #[props(default)] autofocus: bool,
    #[props(default)] rows: Option<u32>,
    #[props(default)] cols: Option<u32>,
    #[props(default)] minlength: Option<u32>,
    #[props(default)] maxlength: Option<u32>,
    #[props(default)] resize: TextAreaResize,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let field_name = field_ctx.name.clone();
    let form_ctx = use_context::<FormContext>();

    let textarea_class = if class.is_empty() {
        merge(&[TEXTAREA_FORM_BASE, resize.as_class()])
    } else {
        class
    };

    let id = String::from(&*field_name);
    let aria_describedby = format!("{}-error", field_name);

    let is_disabled = form_ctx.disabled.map(|d| d()).unwrap_or(false);
    let is_touched = form_ctx.touched_signal.with(|t| t.contains(&*field_name));
    let has_error = form_ctx
        .errors_signal
        .with(|e| e.get(&*field_name).is_some_and(|err| err.is_some()));
    let aria_invalid = if is_touched && has_error {
        Some("true".to_string())
    } else {
        None
    };

    let field_value = form_ctx
        .values_signal
        .with(|v| v.get(&*field_name).cloned().unwrap_or_default());

    rsx! {
        TextAreaBase {
            id: id,
            placeholder: " ".to_string(),
            class: textarea_class,
            disabled: is_disabled,
            autofocus: autofocus,
            rows: rows,
            cols: cols,
            minlength: minlength,
            maxlength: maxlength,
            resize: resize,
            aria_invalid: aria_invalid,
            aria_describedby: aria_describedby,
            // Controlled read: form is the source of truth, `on_change` writes back.
            // No mirror signal / per-render `set` (the prop→signal sync anti-pattern).
            static_value: field_value,
            on_change: {
                let field_name = field_name.clone();
                EventHandler::new(move |v: String| {
                    form_ctx.set_value.read()(&field_name, v);
                })
            },
            onblur: {
                let field_name = field_name.clone();
                EventHandler::new(move |_: FocusEvent| {
                    form_ctx.touch_field.read()(&field_name);
                })
            },
        }
    }
}

#[component]
pub fn TextArea(
    #[props(into)] field: Field,
    #[props(default)] autofocus: bool,
    #[props(default)] rows: Option<u32>,
    #[props(default)] cols: Option<u32>,
    #[props(default)] minlength: Option<u32>,
    #[props(default)] maxlength: Option<u32>,
    #[props(default)] resize: TextAreaResize,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let label = field.label.to_string();

    rsx! {
        FormField { field,
            div { class: "relative w-full mt-2",
                TextAreaFormControl {
                    autofocus: autofocus,
                    rows: rows,
                    cols: cols,
                    minlength: minlength,
                    maxlength: maxlength,
                    resize: resize,
                }
                FormLabel { textarea: true, tooltip, "{label}" }
            }
            FormError {}
        }
    }
}
