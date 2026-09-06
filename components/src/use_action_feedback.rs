use dioxus::CapturedError;
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
/// The action is *not* reset after success (so `action.value()` stays readable
/// by the caller). A component that remounts while the action still holds
/// `Some(Ok)` therefore re-runs `on_success` and re-shows the toast; create the
/// action in the same scope as this hook, or `reset()` it inside `on_success`
/// when that matters.
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
    on_success: S,
) where
    I: 'static,
    T: 'static,
    S: FnMut() + 'static,
{
    action_feedback(
        Box::new(move || action.value().map(|r| r.map(|_| ()))),
        Box::new(move || action.reset()),
        success_toast.into(),
        error_prefix.into(),
        Box::new(on_success),
    );
}

/// Outcome of the last action call with the success payload erased.
type ActionOutcome = Option<Result<(), CapturedError>>;

/// Body of [`use_action_feedback`] with `I`/`T`/`S` erased at the boundary,
/// so the effect and toast wiring compile once per app instead of once per
/// call site.
#[inline(never)]
fn action_feedback(
    outcome: Box<dyn Fn() -> ActionOutcome>,
    mut reset: Box<dyn FnMut()>,
    success_toast: Option<&'static str>,
    error_prefix: Option<&'static str>,
    mut on_success: Box<dyn FnMut()>,
) {
    let toast = use_toast();

    use_effect(move || match outcome() {
        Some(Ok(())) => {
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
            reset();
        }
        None => {}
    });
}
