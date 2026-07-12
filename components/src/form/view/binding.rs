//! The one place a form-bound control reads/writes its field's state.
//! Replaces the touched/error/aria derivation that was previously copy-pasted
//! into every `*FormControl`.

use std::sync::Arc;

use dioxus::prelude::*;

use crate::form::{FieldContext, FormContext};

/// Controlled-prop bundle adapting the surrounding form to any base component:
/// reactive value / invalid / disabled reads plus write-back callbacks.
pub(crate) struct FieldBinding {
    /// The field name (map key into the form's value store).
    pub name: Arc<str>,
    /// DOM id for the control (the field name), targeted by labels.
    pub id: String,
    /// Id of the field's error element, for `aria-describedby`.
    pub aria_describedby: String,
    /// Current raw value of the field.
    pub value: Memo<String>,
    /// The field value in the controlled-input prop shape (`Some(value)`).
    pub controlled_value: ReadSignal<Option<String>>,
    /// Whether the field is touched and failing validation.
    pub invalid: Memo<bool>,
    /// Whether the surrounding form is disabled (submitting / loading).
    pub disabled: Memo<bool>,
    /// Write a new value (input event).
    pub on_value_change: Callback<String>,
    /// Write a new value and mark the field touched (change event).
    pub on_commit: Callback<String>,
    /// Mark the field touched (blur).
    pub touch: Callback<()>,
}

impl FieldBinding {
    /// `aria-invalid` attribute value derived from `invalid`.
    pub fn aria_invalid(&self) -> Option<String> {
        self.invalid.read().then(|| "true".to_string())
    }
}

pub(crate) fn use_field_binding() -> FieldBinding {
    // Typed form store: `typed::FormField` provides a `Signal<BoundField>` in
    // context; adapt it to the same string-shaped binding. The branch is
    // stable for a control instance's lifetime (it mounts under one form
    // flavor), so hook order is consistent.
    if let Some(bound) = try_use_context::<Signal<crate::form::typed::BoundField>>() {
        return use_typed_field_binding(bound);
    }

    let field_ctx = use_context::<FieldContext>();
    let form_ctx = use_context::<FormContext>();
    let name = field_ctx.name.clone();

    let value = {
        let name = name.clone();
        use_memo(move || {
            form_ctx
                .values_signal
                .with(|v| v.get(&*name).cloned().unwrap_or_default())
        })
    };
    let invalid = {
        let name = name.clone();
        use_memo(move || {
            form_ctx.touched_signal.with(|t| t.contains(&*name))
                && form_ctx
                    .errors_signal
                    .with(|e| e.get(&*name).is_some_and(|err| err.is_some()))
        })
    };
    let disabled = use_memo(move || form_ctx.disabled.map(|d| d()).unwrap_or(false));
    let controlled_value: ReadSignal<Option<String>> = use_memo(move || Some(value())).into();

    let on_value_change = use_callback({
        let name = name.clone();
        move |v: String| form_ctx.set_value.read()(&name, v)
    });
    let on_commit = use_callback({
        let name = name.clone();
        move |v: String| {
            form_ctx.set_value.read()(&name, v);
            form_ctx.touch_field.read()(&name);
        }
    });
    let touch = use_callback({
        let name = name.clone();
        move |_: ()| form_ctx.touch_field.read()(&name)
    });

    FieldBinding {
        id: String::from(&*name),
        aria_describedby: format!("{}-error", name),
        name,
        value,
        controlled_value,
        invalid,
        disabled,
        on_value_change,
        on_commit,
        touch,
    }
}

/// Adapts a typed-form [`crate::form::typed::BoundField`] (provided as a
/// signal so row re-keys propagate) to the string-shaped [`FieldBinding`]
/// every control consumes.
fn use_typed_field_binding(bound: Signal<crate::form::typed::BoundField>) -> FieldBinding {
    let form_ctx = try_use_context::<crate::form::typed::view::FormContext>();
    let name: Arc<str> = Arc::from(bound.peek().path());

    let value = use_memo(move || bound.read().display());
    let invalid = use_memo(move || bound.read().invalid());
    let disabled = use_memo(move || {
        form_ctx
            .and_then(|ctx| ctx.disabled.map(|d| d()))
            .unwrap_or(false)
    });
    let controlled_value: ReadSignal<Option<String>> = use_memo(move || Some(value())).into();

    let on_value_change = use_callback(move |v: String| bound.peek().set_text(&v));
    let on_commit = use_callback(move |v: String| bound.peek().commit_text(&v));
    let touch = use_callback(move |_: ()| bound.peek().touch());

    FieldBinding {
        id: String::from(&*name),
        aria_describedby: format!("{}-error", name),
        name,
        value,
        controlled_value,
        invalid,
        disabled,
        on_value_change,
        on_commit,
        touch,
    }
}
