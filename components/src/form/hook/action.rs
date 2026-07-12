use dioxus::CapturedError;
use dioxus::prelude::*;
use ds_utils::DsError;

pub type SubmitFn = Box<dyn FnMut()>;

/// Type-erased wrapper around `Action<(T,), O>`.
/// Erases output generic so FormProvider doesn't need to know `O`.
pub struct FormSubmit<T: 'static> {
    call_fn: CopyValue<Box<dyn FnMut(T)>>,
    pending_fn: CopyValue<Box<dyn Fn() -> bool>>,
    result_fn: CopyValue<Box<dyn Fn() -> Option<ds_utils::Result<()>>>>,
}

impl<T: 'static> Clone for FormSubmit<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for FormSubmit<T> {}

impl<T: 'static> PartialEq for FormSubmit<T> {
    fn eq(&self, other: &Self) -> bool {
        self.call_fn.origin_scope() == other.call_fn.origin_scope()
            && self.pending_fn.origin_scope() == other.pending_fn.origin_scope()
    }
}

impl<T: 'static> FormSubmit<T> {
    pub fn call(&self, data: T) {
        (self.call_fn.write_unchecked())(data);
    }

    pub fn pending(&self) -> bool {
        (self.pending_fn.read())()
    }

    /// Last action result, with success payload erased. `None` while pending/reset.
    pub fn result(&self) -> Option<ds_utils::Result<()>> {
        (self.result_fn.read())()
    }

    /// Create from action whose input differs from `T`.
    /// Transform closure maps validated form data `T` into action's input `U`.
    pub fn with_transform<U: 'static, O: 'static>(
        mut action: Action<(U,), O>,
        transform: impl Fn(T) -> U + 'static,
    ) -> Self {
        let pending_action = action;
        let result_action = action;
        Self {
            call_fn: CopyValue::new(Box::new(move |data: T| {
                action.call(transform(data));
            })),
            pending_fn: CopyValue::new(Box::new(move || pending_action.pending())),
            result_fn: CopyValue::new(Box::new(move || action_result(&result_action))),
        }
    }
}

impl<T: 'static, O: 'static> From<Action<(T,), O>> for FormSubmit<T> {
    fn from(mut action: Action<(T,), O>) -> Self {
        let pending_action = action;
        let result_action = action;
        Self {
            call_fn: CopyValue::new(Box::new(move |data: T| {
                action.call(data);
            })),
            pending_fn: CopyValue::new(Box::new(move || pending_action.pending())),
            result_fn: CopyValue::new(Box::new(move || action_result(&result_action))),
        }
    }
}

/// Recover the original [`AppError`] from a Dioxus [`CapturedError`], falling back to a generic
/// internal-server error when the captured error is not an `AppError`.
pub fn captured_app_error(captured: &CapturedError) -> DsError {
    captured
        .downcast_ref::<DsError>()
        .cloned()
        .unwrap_or_else(|| DsError::InternalServer(captured.to_string()))
}

fn action_result<I: 'static, O: 'static>(action: &Action<I, O>) -> Option<ds_utils::Result<()>> {
    match action.value() {
        Some(Ok(_)) => Some(Ok(())),
        Some(Err(captured)) => Some(Err(captured_app_error(&captured))),
        None => None,
    }
}
