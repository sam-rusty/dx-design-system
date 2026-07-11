use dioxus::prelude::*;

use crate::form::captured_app_error;
use crate::toast::{ToastVariant, use_toast};

/// Standard success/error wiring for a [`use_action`](dioxus::prelude::use_action) result.
///
/// Replaces the copy-pasted
/// `use_effect(|| match action.value() { Some(Ok)… Some(Err)… None })` block.
///
/// - **Success**: runs `on_success`, then (when `success_toast` is set) shows it.
/// - **Error**: shows `"{error_prefix}: {e}"` — or just the error when `error_prefix` is
///   `None` — and resets the action so the user can retry.
///
/// Pass `None` for either message to opt out of that toast/prefix.
///
/// ```rust, ignore
/// use_action_feedback(create_action, "Note posted", "Failed to post note", move || {
///     form.reset();
///     resource.restart();
/// });
/// ```
pub fn use_action_feedback<I, T, S>(
    mut action: Action<I, T>,
    success_toast: impl Into<Option<&'static str>>,
    error_prefix: impl Into<Option<&'static str>>,
    mut on_success: S,
) where
    I: 'static,
    T: 'static,
    S: FnMut() + 'static,
{
    let toast = use_toast();
    let success_toast = success_toast.into();
    let error_prefix = error_prefix.into();

    use_effect(move || match action.value() {
        Some(Ok(_)) => {
            on_success();
            if let Some(msg) = success_toast {
                toast.push(msg.into(), ToastVariant::Success);
            }
        }
        Some(Err(e)) => {
            let app_err = captured_app_error(&e);
            let msg = match error_prefix {
                Some(prefix) => format!("{prefix}: {app_err}"),
                None => app_err.to_string(),
            };
            toast.push(msg, ToastVariant::Error);
            action.reset();
        }
        None => {}
    });
}
