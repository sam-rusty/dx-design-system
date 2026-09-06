//! Overlay focus and dismissal composition.
//!
//! [`use_focus_restore`] saves the element focused when an overlay opens and
//! gives focus back to it when the overlay closes or unmounts. [`use_overlay`]
//! composes it with the stacked Escape and outside-pointer dismissal hooks for
//! a caller-owned `open` signal (the `Modal` shape). Components whose panel
//! lives in a `Portal` compose the pieces themselves with
//! [`use_outside_dismiss_panel`](super::use_outside_dismiss_panel).
//!
//! TODO (follow-up): Tab focus *trapping* within the overlay and exit
//! animations layer on top of this without changing these signatures.

use dioxus::prelude::*;

use super::{use_escape_listener_open, use_outside_dismiss};

/// Restore focus to whatever was focused when `is_open` flipped to `true` once
/// it flips back to `false` (or the caller unmounts while open).
///
/// Acts only on the open/close *edge*: saving on every render-while-open would
/// clobber the trigger with whatever is now focused inside the overlay, and
/// restoring on every re-run would steal focus back mid-interaction.
pub(crate) fn use_focus_restore(is_open: ReadSignal<bool>) {
    #[cfg(feature = "web")]
    {
        use wasm_bindgen::JsCast;

        let saved: std::rc::Rc<std::cell::RefCell<Option<web_sys::HtmlElement>>> =
            use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None)));
        let was_open: std::rc::Rc<std::cell::Cell<bool>> =
            use_hook(|| std::rc::Rc::new(std::cell::Cell::new(false)));

        use_effect({
            let saved = saved.clone();
            let was_open = was_open.clone();
            move || {
                let open = is_open();
                let was = was_open.replace(open);
                if open && !was {
                    if let Some(active) = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                    {
                        *saved.borrow_mut() = Some(active);
                    }
                } else if !open
                    && was
                    && let Some(prev) = saved.borrow_mut().take()
                {
                    let _ = prev.focus();
                }
            }
        });

        use_drop(move || {
            if let Some(prev) = saved.borrow_mut().take() {
                let _ = prev.focus();
            }
        });
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = is_open;
    }
}

/// Escape + outside-pointer dismissal + focus restoration for a caller-owned
/// `open` signal. Returns the root-element signal the caller wires to the
/// overlay root's `onmounted` so the hook knows what counts as "inside".
pub(crate) fn use_overlay(mut open: Signal<bool>) -> Signal<Option<web_sys::Element>> {
    let root_el = use_signal(|| None::<web_sys::Element>);

    use_escape_listener_open(open.into(), move || {
        if open() {
            open.set(false);
        }
    });

    use_outside_dismiss(root_el, open.into(), move || {
        if open() {
            open.set(false);
        }
    });

    use_focus_restore(open.into());

    root_el
}
