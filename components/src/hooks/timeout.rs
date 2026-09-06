//! Managed `setTimeout` timers that are cancelled on unmount.
//!
//! [`use_timer`] is a re-armable one-shot: `start()` (re)schedules `on_elapsed`
//! after `delay_ms`, `cancel()` clears a pending run, and unmount cancels
//! whatever is pending, so a callback can never fire into a dropped scope
//! (`Copyable`'s "Copied!" reset, form-field copy feedback). [`use_timeout`]
//! is the arm-once-on-mount shape used by `Toast`'s auto-dismiss.

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
struct TimeoutState {
    delay_ms: u32,
    on_elapsed: Rc<RefCell<Box<dyn FnMut()>>>,
    handle: Option<i32>,
    // Kept alive so the JS callback isn't freed before it fires.
    _closure: Option<Closure<dyn FnMut()>>,
}

#[cfg(feature = "web")]
impl TimeoutState {
    fn schedule(state: &Rc<RefCell<Self>>) {
        let Some(win) = web_sys::window() else {
            return;
        };
        state.borrow_mut().cancel();
        let fire_state = state.clone();
        let closure = Closure::once(move || {
            let on_elapsed = {
                let mut s = fire_state.borrow_mut();
                s.handle = None;
                s._closure = None;
                s.on_elapsed.clone()
            };
            (on_elapsed.borrow_mut())();
        });
        let mut s = state.borrow_mut();
        s.handle = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                s.delay_ms as i32,
            )
            .ok();
        s._closure = Some(closure);
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

/// Handle to a re-armable timer created by [`use_timer`].
#[derive(Clone, Copy)]
pub(crate) struct Timer {
    start: Callback<()>,
    cancel: Callback<()>,
}

impl Timer {
    /// (Re)schedule the callback `delay_ms` from now; a pending run is replaced.
    pub(crate) fn start(&self) {
        self.start.call(());
    }

    /// Clear the pending run, if any.
    pub(crate) fn cancel(&self) {
        self.cancel.call(());
    }
}

/// A one-shot timer that can be armed any number of times and is cancelled on
/// unmount. Nothing is scheduled until [`Timer::start`] is called.
pub(crate) fn use_timer(delay_ms: u32, on_elapsed: impl FnMut() + 'static) -> Timer {
    timer(delay_ms, Box::new(on_elapsed))
}

#[inline(never)]
fn timer(delay_ms: u32, on_elapsed: Box<dyn FnMut()>) -> Timer {
    #[cfg(feature = "web")]
    {
        let state = use_hook(|| {
            Rc::new(RefCell::new(TimeoutState {
                delay_ms,
                on_elapsed: Rc::new(RefCell::new(on_elapsed)),
                handle: None,
                _closure: None,
            }))
        });

        use_drop({
            let state = state.clone();
            move || state.borrow_mut().cancel()
        });

        let start = use_callback({
            let state = state.clone();
            move |()| TimeoutState::schedule(&state)
        });
        let cancel = use_callback(move |()| state.borrow_mut().cancel());
        Timer { start, cancel }
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = (delay_ms, on_elapsed);
        Timer {
            start: use_callback(|()| {}),
            cancel: use_callback(|()| {}),
        }
    }
}

/// Schedule `on_elapsed` to run once after `delay_ms`, cancelled automatically on
/// unmount. Returns a callback that cancels the pending timer early.
pub(crate) fn use_timeout(delay_ms: u32, on_elapsed: impl FnMut() + 'static) -> Callback<()> {
    let timer = use_timer(delay_ms, on_elapsed);
    use_hook(move || timer.start());
    use_callback(move |()| timer.cancel())
}
