//! `use_overlay` — composes the dismissal + focus concerns every overlay shares
//! (modal, popover, dropdown, tooltip, the calendar panel, `MultiSearch`).
//!
//! Wires, for a caller-owned `open` signal:
//! - Escape closes the top-most overlay (stacked — see [`use_escape_listener`]).
//! - A pointer-down outside the root closes it (see [`use_outside_dismiss`]).
//! - Focus is saved when the overlay opens and restored to the previously
//!   focused element when it closes or unmounts (fixes the modal "focus is not
//!   returned to the trigger" finding).
//!
//! Returns the root-element signal the caller wires to the overlay root's
//! `onmounted` so the hook knows what counts as "inside".
//!
//! TODO (follow-up, tracked in docs/audits/components-audit.md §3): Tab focus
//! *trapping* within the overlay and `use_animated_open` exit animations. This
//! v1 covers escape + outside-dismiss + focus restoration, which closes the
//! highest-severity overlay findings; trapping/animation layer on top without
//! changing this signature.

use dioxus::prelude::*;

use super::{use_escape_listener, use_outside_dismiss};

pub(crate) fn use_overlay(mut open: Signal<bool>) -> Signal<Option<web_sys::Element>> {
    let root_el = use_signal(|| None::<web_sys::Element>);

    use_escape_listener(move || {
        if open() {
            open.set(false);
        }
    });

    use_outside_dismiss(root_el, open.into(), move || {
        if open() {
            open.set(false);
        }
    });

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let saved: std::rc::Rc<std::cell::RefCell<Option<web_sys::HtmlElement>>> =
            use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None)));
        // Tracks the previous `open` value so we act only on the open<->close
        // *edge*. Saving on every render-while-open would clobber the trigger
        // with whatever is now focused inside the overlay; restoring on every
        // re-run would steal focus back to the trigger mid-interaction.
        let was_open: std::rc::Rc<std::cell::Cell<bool>> =
            use_hook(|| std::rc::Rc::new(std::cell::Cell::new(false)));

        use_effect({
            let saved = saved.clone();
            let was_open = was_open.clone();
            move || {
                let is_open = open();
                let was = was_open.replace(is_open);
                if is_open && !was {
                    if let Some(active) = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                    {
                        *saved.borrow_mut() = Some(active);
                    }
                } else if !is_open
                    && was
                    && let Some(prev) = saved.borrow_mut().take()
                {
                    let _ = prev.focus();
                }
            }
        });

        // Restore focus if the overlay unmounts while still open.
        use_drop({
            let saved = saved.clone();
            move || {
                if let Some(prev) = saved.borrow_mut().take() {
                    let _ = prev.focus();
                }
            }
        });
    }

    root_el
}
