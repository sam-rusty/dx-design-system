//! Escape-to-close and outside-pointer dismissal for overlays.
//!
//! Ports the behavior of `dioxus_primitives`' `use_global_escape_listener` and
//! `use_outside_dismiss` (`primitives/src/lib.rs`) in this crate's house style
//! (web-sys, gated on `wasm32` with a no-op fallback so the SSR build compiles).
//!
//! Two improvements over the existing `use_outside_click`:
//! - listens on the **capture phase** of `pointerdown` (not bubbling `click`),
//!   which fires before the opening interaction can complete — removing the
//!   need for the `requestAnimationFrame` defer hack and fixing the
//!   text-selection-drag false dismiss.
//! - escape listeners form a **stack** so only the top-most overlay closes,
//!   making nested overlays (popover-in-dialog) behave correctly.

#![allow(unused_variables)]

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;

/// Stack of active escape-listener instance ids; the last entry is the top-most
/// overlay and the only one allowed to react to an Escape press.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct EscapeStack(Rc<RefCell<Vec<usize>>>);

#[cfg(target_arch = "wasm32")]
fn escape_stack() -> EscapeStack {
    try_consume_context::<EscapeStack>()
        .unwrap_or_else(|| provide_context(EscapeStack(Rc::new(RefCell::new(Vec::new())))))
}

/// Stack of currently-*open* outside-dismiss overlays; the last entry is the
/// top-most one and the only one allowed to react to an outside pointer-down.
/// Mirrors [`EscapeStack`] so outside-dismiss nests the same way Escape does —
/// a click inside a stacked child overlay (a sibling DOM subtree via `Portal`)
/// never dismisses its parent.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct OverlayStack(Rc<RefCell<Vec<usize>>>);

#[cfg(target_arch = "wasm32")]
fn overlay_stack() -> OverlayStack {
    try_consume_context::<OverlayStack>()
        .unwrap_or_else(|| provide_context(OverlayStack(Rc::new(RefCell::new(Vec::new())))))
}

/// Call `on_escape` when the user presses Escape, but only while this listener
/// is the top-most one (the most recently mounted overlay wins).
pub(crate) fn use_escape_listener(on_escape: impl FnMut() + 'static) {
    #[cfg(target_arch = "wasm32")]
    {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = use_hook(|| NEXT.fetch_add(1, Ordering::Relaxed));

        let stack = use_hook(|| {
            let stack = escape_stack();
            stack.0.borrow_mut().push(id);
            stack
        });

        let registered: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>> =
            use_hook(|| Rc::new(RefCell::new(None)));

        use_hook({
            let stack = stack.clone();
            let registered = registered.clone();
            let mut on_escape = on_escape;
            move || {
                let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                    return;
                };
                let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let Ok(ev) = e.dyn_into::<web_sys::KeyboardEvent>() else {
                        return;
                    };
                    if ev.key() != "Escape" {
                        return;
                    }
                    if stack.0.borrow().last() == Some(&id) {
                        ev.prevent_default();
                        on_escape();
                    }
                }) as Box<dyn FnMut(web_sys::Event)>);
                let _ = doc
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                *registered.borrow_mut() = Some(closure);
            }
        });

        use_drop({
            let stack = stack.clone();
            let registered = registered.clone();
            move || {
                stack.0.borrow_mut().retain(|other| *other != id);
                if let Some(doc) = web_sys::window().and_then(|w| w.document())
                    && let Some(closure) = registered.borrow_mut().take()
                {
                    let _ = doc.remove_event_listener_with_callback(
                        "keydown",
                        closure.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = on_escape;
    }
}

/// Call `on_dismiss` when a `pointerdown` lands outside `root_el`, but only while
/// this overlay is the top-most open one.
///
/// `root_el` is the overlay root the caller wires via `onmounted`; anything it
/// contains counts as "inside". `is_open` reports whether this overlay is
/// currently open — it gates membership in [`OverlayStack`] so "top-most" tracks
/// the most-recently-opened overlay, not merely the last mounted (dropdowns and
/// selects stay mounted while closed). Uses the capture phase so the dismiss
/// check runs before the event reaches application handlers.
pub(crate) fn use_outside_dismiss(
    root_el: Signal<Option<web_sys::Element>>,
    is_open: ReadSignal<bool>,
    on_dismiss: impl FnMut() + 'static,
) {
    let panel_el = use_signal(|| None::<web_sys::Element>);
    use_outside_dismiss_panel(root_el, panel_el, is_open, on_dismiss);
}

/// Like [`use_outside_dismiss`] but treats a second `panel_el` as "inside" too.
///
/// Needed when the overlay panel is rendered through a [`Portal`](crate::portal::Portal):
/// the panel lives in a sibling DOM subtree, so `root_el` (the trigger wrapper) no
/// longer contains it. Without counting `panel_el`, a pointer-down on the panel's own
/// controls would read as "outside" and dismiss the overlay before the click lands.
pub(crate) fn use_outside_dismiss_panel(
    root_el: Signal<Option<web_sys::Element>>,
    panel_el: Signal<Option<web_sys::Element>>,
    is_open: ReadSignal<bool>,
    on_dismiss: impl FnMut() + 'static,
) {
    #[cfg(target_arch = "wasm32")]
    {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = use_hook(|| NEXT.fetch_add(1, Ordering::Relaxed));
        let stack = use_hook(overlay_stack);

        // Keep this overlay on the shared stack only while it is open, so the
        // top-most entry is the overlay the user is actually interacting with.
        use_effect({
            let stack = stack.clone();
            move || {
                let open = is_open();
                let mut entries = stack.0.borrow_mut();
                entries.retain(|other| *other != id);
                if open {
                    entries.push(id);
                }
            }
        });

        let registered: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>> =
            use_hook(|| Rc::new(RefCell::new(None)));

        use_hook({
            let stack = stack.clone();
            let registered = registered.clone();
            let mut on_dismiss = on_dismiss;
            move || {
                let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                    return;
                };
                let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                    // Only the top-most open overlay reacts; a pointer-down inside
                    // a stacked child overlay is "outside" this root (sibling DOM
                    // subtree via Portal), and without this gate the parent would
                    // dismiss itself and unmount the child.
                    if stack.0.borrow().last() != Some(&id) {
                        return;
                    }
                    let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                    else {
                        return;
                    };
                    let Some(root) = root_el.peek().clone() else {
                        return;
                    };
                    let in_panel = panel_el
                        .peek()
                        .as_ref()
                        .is_some_and(|p| p.contains(Some(&target)));
                    if !root.contains(Some(&target)) && !in_panel {
                        on_dismiss();
                    }
                }) as Box<dyn FnMut(web_sys::Event)>);
                let _ = doc.add_event_listener_with_callback_and_bool(
                    "pointerdown",
                    closure.as_ref().unchecked_ref(),
                    true,
                );
                *registered.borrow_mut() = Some(closure);
            }
        });

        use_drop({
            let stack = stack.clone();
            let registered = registered.clone();
            move || {
                stack.0.borrow_mut().retain(|other| *other != id);
                if let Some(doc) = web_sys::window().and_then(|w| w.document())
                    && let Some(closure) = registered.borrow_mut().take()
                {
                    let _ = doc.remove_event_listener_with_callback_and_bool(
                        "pointerdown",
                        closure.as_ref().unchecked_ref(),
                        true,
                    );
                }
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (root_el, panel_el, is_open, on_dismiss);
    }
}

/// Dismiss an open overlay when the viewport shifts under it.
///
/// A `Portal`ed panel positions itself once via fixed coordinates (see
/// [`Popover`](crate::popover::Popover)), so scrolling any ancestor or resizing the
/// window would leave it detached from its trigger. Rather than re-measure every
/// frame, close it: while `is_open`, a scroll or resize fires `on_dismiss`. Scroll
/// is listened to on the capture phase because scroll events do not bubble — capture
/// is the only way to see scrolls on arbitrary scroll containers, not just `window`.
pub(crate) fn use_dismiss_on_viewport_change(
    is_open: ReadSignal<bool>,
    on_dismiss: impl FnMut() + 'static,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let registered: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>> =
            use_hook(|| Rc::new(RefCell::new(None)));

        use_hook({
            let registered = registered.clone();
            let mut on_dismiss = on_dismiss;
            move || {
                let Some(win) = web_sys::window() else {
                    return;
                };
                let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    if *is_open.peek() {
                        on_dismiss();
                    }
                }) as Box<dyn FnMut(web_sys::Event)>);
                let cb = closure.as_ref().unchecked_ref();
                let _ = win.add_event_listener_with_callback_and_bool("scroll", cb, true);
                let _ = win.add_event_listener_with_callback("resize", cb);
                *registered.borrow_mut() = Some(closure);
            }
        });

        use_drop(move || {
            if let Some(win) = web_sys::window()
                && let Some(closure) = registered.borrow_mut().take()
            {
                let cb = closure.as_ref().unchecked_ref();
                let _ = win.remove_event_listener_with_callback_and_bool("scroll", cb, true);
                let _ = win.remove_event_listener_with_callback("resize", cb);
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (is_open, on_dismiss);
    }
}
