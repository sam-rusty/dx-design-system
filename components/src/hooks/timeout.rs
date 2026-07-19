//! A managed `setTimeout` that is cancelled on unmount (or manually), replacing
//! the fire-and-forget timers flagged in the audit — `toast`'s auto-dismiss
//! (armed in the render body, re-armed every render, never cancelled) and
//! `copyable`'s reset timer.
//!
//! The timer is scheduled once when the component mounts and cleared on unmount;
//! the returned [`Callback`] lets the caller cancel early (e.g. on hover).

#![allow(unused_variables)]

#[cfg(feature = "web")]
use std::cell::RefCell;
#[cfg(feature = "web")]
use std::rc::Rc;

use dioxus::prelude::*;
#[cfg(feature = "web")]
use wasm_bindgen::JsCast;
#[cfg(feature = "web")]
use wasm_bindgen::closure::Closure;

#[cfg(feature = "web")]
#[derive(Default)]
struct TimeoutState {
    handle: Option<i32>,
    // Kept alive so the JS callback isn't freed before it fires.
    _closure: Option<Closure<dyn FnMut()>>,
}

#[cfg(feature = "web")]
impl TimeoutState {
    fn schedule(state: &Rc<RefCell<Self>>, delay_ms: u32, on_elapsed: impl FnOnce() + 'static) {
        let Some(win) = web_sys::window() else {
            return;
        };
        let fire_state = state.clone();
        let closure = Closure::once(move || {
            fire_state.borrow_mut().handle = None;
            on_elapsed();
        });
        let handle = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms as i32,
            )
            .ok();
        let mut state = state.borrow_mut();
        state.handle = handle;
        state._closure = Some(closure);
    }

    fn cancel(&mut self) {
        if let Some(handle) = self.handle.take()
            && let Some(win) = web_sys::window()
        {
            win.clear_timeout_with_handle(handle);
        }
        self._closure = None;
    }
}

/// Schedule `on_elapsed` to run once after `delay_ms`, cancelled automatically on
/// unmount. Returns a callback that cancels the pending timer early.
pub(crate) fn use_timeout(delay_ms: u32, on_elapsed: impl FnOnce() + 'static) -> Callback<()> {
    #[cfg(feature = "web")]
    {
        let state = use_hook(|| Rc::new(RefCell::new(TimeoutState::default())));

        use_hook({
            let state = state.clone();
            move || TimeoutState::schedule(&state, delay_ms, on_elapsed)
        });

        use_drop({
            let state = state.clone();
            move || state.borrow_mut().cancel()
        });

        use_callback(move |()| state.borrow_mut().cancel())
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = (delay_ms, on_elapsed);
        use_callback(|()| {})
    }
}
