//! Escape-to-close and outside-pointer dismissal for overlays.
//!
//! Ports the behavior of `dioxus_primitives`' `use_global_escape_listener` and
//! `use_outside_dismiss` (`primitives/src/lib.rs`) in this crate's house style
//! (web-sys, gated on the `web` feature with a no-op fallback so the SSR build
//! compiles).
//!
//! Model shared by every hook here:
//! - Overlays form an **app-wide stack** (root context) ordered by *open* time,
//!   not mount time. Only the top-most **open** overlay reacts to Escape or an
//!   outside pointer-down, so a closed `Select` mounted after an open `Modal`
//!   never swallows the key, and nested overlays (popover-in-dialog) close
//!   inner-first.
//! - Document/window listeners are attached only while the overlay is open, so
//!   a list of fifty rows each carrying a `PopoverConfirm` costs nothing until
//!   one opens.
//! - Outside dismissal listens on the **capture phase** of `pointerdown` (not
//!   bubbling `click`), which fires before the opening interaction can
//!   complete and does not misfire on a text-selection drag.

#[cfg(feature = "web")]
use std::cell::RefCell;
#[cfg(feature = "web")]
use std::rc::Rc;
#[cfg(feature = "web")]
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
#[cfg(feature = "web")]
use wasm_bindgen::JsCast;
#[cfg(feature = "web")]
use wasm_bindgen::closure::Closure;

/// Is `id` the top-most entry of an overlay stack?
///
/// Pure so the stack policy is unit-testable off wasm; the hooks below feed it
/// the shared stack and their own id.
#[cfg(any(feature = "web", test))]
fn is_top(stack: &[usize], id: usize) -> bool {
    stack.last() == Some(&id)
}

/// Keep `id` on `stack` exactly while `open`; an opening overlay moves to the top.
#[cfg(any(feature = "web", test))]
fn set_membership(stack: &mut Vec<usize>, id: usize, open: bool) {
    stack.retain(|other| *other != id);
    if open {
        stack.push(id);
    }
}

/// Stack of currently-open escape listeners; the last entry is the top-most
/// overlay and the only one allowed to react to an Escape press.
#[cfg(feature = "web")]
#[derive(Clone)]
struct EscapeStack(Rc<RefCell<Vec<usize>>>);

/// Stack of currently-open outside-dismiss overlays; the last entry is the
/// top-most one and the only one allowed to react to an outside pointer-down.
#[cfg(feature = "web")]
#[derive(Clone)]
struct OverlayStack(Rc<RefCell<Vec<usize>>>);

/// The stacks live in the root scope (see [`root_context`]) so overlays in
/// sibling subtrees (a `Portal`ed dialog and a filters panel elsewhere on the
/// page) share one ordering.
#[cfg(feature = "web")]
fn root_stack<S: Clone + 'static>(make: impl FnOnce() -> S) -> S {
    super::root_context(make)
}

#[cfg(feature = "web")]
fn next_overlay_id() -> usize {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// A set of DOM listeners on one target, removed on drop.
#[cfg(feature = "web")]
struct DomListeners {
    target: web_sys::EventTarget,
    /// `(event name, capture)`.
    events: &'static [(&'static str, bool)],
    closure: Closure<dyn FnMut(web_sys::Event)>,
}

#[cfg(feature = "web")]
impl DomListeners {
    fn attach(
        target: web_sys::EventTarget,
        events: &'static [(&'static str, bool)],
        handler: impl FnMut(web_sys::Event) + 'static,
    ) -> Self {
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::Event)>);
        for (name, capture) in events {
            let _ = target.add_event_listener_with_callback_and_bool(
                name,
                closure.as_ref().unchecked_ref(),
                *capture,
            );
        }
        Self {
            target,
            events,
            closure,
        }
    }
}

#[cfg(feature = "web")]
impl Drop for DomListeners {
    fn drop(&mut self) {
        for (name, capture) in self.events {
            let _ = self.target.remove_event_listener_with_callback_and_bool(
                name,
                self.closure.as_ref().unchecked_ref(),
                *capture,
            );
        }
    }
}

/// Keep the listeners built by `attach` alive exactly while `is_open` reads
/// `true`; they are removed when it flips to `false` and on unmount.
#[cfg(feature = "web")]
fn use_listeners_while_open(
    is_open: ReadSignal<bool>,
    attach: impl Fn() -> Option<DomListeners> + 'static,
) {
    let active: Rc<RefCell<Option<DomListeners>>> = use_hook(|| Rc::new(RefCell::new(None)));

    use_effect({
        let active = active.clone();
        move || {
            let open = is_open();
            let mut slot = active.borrow_mut();
            match (open, slot.is_some()) {
                (true, false) => *slot = attach(),
                (false, true) => *slot = None,
                _ => {}
            }
        }
    });

    use_drop(move || {
        active.borrow_mut().take();
    });
}

/// Call `on_escape` when the user presses Escape while `is_open` is `true` and
/// this overlay is the most recently opened one.
///
/// Membership in the escape stack follows `is_open`, so an overlay that stays
/// mounted while closed (`Select`, `DropdownMenu`, a row's `PopoverConfirm`)
/// never shadows an open one that mounted earlier.
pub fn use_escape_listener_open(is_open: ReadSignal<bool>, on_escape: impl FnMut() + 'static) {
    #[cfg(feature = "web")]
    {
        let id = use_hook(next_overlay_id);
        let stack = use_hook(|| root_stack(|| EscapeStack(Rc::new(RefCell::new(Vec::new())))));
        let on_escape: Rc<RefCell<dyn FnMut()>> = use_hook(|| Rc::new(RefCell::new(on_escape)));

        use_effect({
            let stack = stack.clone();
            move || set_membership(&mut stack.0.borrow_mut(), id, is_open())
        });

        use_listeners_while_open(is_open, {
            let stack = stack.clone();
            move || {
                let doc = web_sys::window()?.document()?;
                let stack = stack.clone();
                let on_escape = on_escape.clone();
                Some(DomListeners::attach(
                    doc.into(),
                    &[("keydown", false)],
                    move |e| {
                        let Ok(ev) = e.dyn_into::<web_sys::KeyboardEvent>() else {
                            return;
                        };
                        if ev.key() != "Escape" || !is_top(&stack.0.borrow(), id) {
                            return;
                        }
                        ev.prevent_default();
                        (on_escape.borrow_mut())();
                    },
                ))
            }
        });

        use_drop(move || {
            set_membership(&mut stack.0.borrow_mut(), id, false);
        });
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = (is_open, on_escape);
    }
}

/// Call `on_escape` when the user presses Escape, but only while this listener
/// is the top-most one. The listener counts as "open" for its whole lifetime;
/// components that stay mounted while closed should use
/// [`use_escape_listener_open`] so they do not shadow other overlays.
pub fn use_escape_listener(on_escape: impl FnMut() + 'static) {
    let always_open: ReadSignal<bool> = use_signal(|| true).into();
    use_escape_listener_open(always_open, on_escape);
}

/// Call `on_dismiss` when a `pointerdown` lands outside `root_el`, but only while
/// this overlay is the top-most open one.
///
/// `root_el` is the overlay root the caller wires via `onmounted`; anything it
/// contains counts as "inside". `is_open` reports whether this overlay is
/// currently open — it gates membership in the overlay stack so "top-most"
/// tracks the most-recently-opened overlay, not merely the last mounted
/// (dropdowns and selects stay mounted while closed). Uses the capture phase so
/// the dismiss check runs before the event reaches application handlers.
pub fn use_outside_dismiss(
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
    #[cfg(feature = "web")]
    {
        let id = use_hook(next_overlay_id);
        let stack = use_hook(|| root_stack(|| OverlayStack(Rc::new(RefCell::new(Vec::new())))));
        let on_dismiss: Rc<RefCell<dyn FnMut()>> = use_hook(|| Rc::new(RefCell::new(on_dismiss)));

        use_effect({
            let stack = stack.clone();
            move || set_membership(&mut stack.0.borrow_mut(), id, is_open())
        });

        use_listeners_while_open(is_open, {
            let stack = stack.clone();
            move || {
                let doc = web_sys::window()?.document()?;
                let stack = stack.clone();
                let on_dismiss = on_dismiss.clone();
                Some(DomListeners::attach(
                    doc.into(),
                    &[("pointerdown", true)],
                    move |e| {
                        // Only the top-most open overlay reacts; a pointer-down
                        // inside a stacked child overlay is "outside" this root
                        // (sibling DOM subtree via Portal), and without this gate
                        // the parent would dismiss itself and unmount the child.
                        if !is_top(&stack.0.borrow(), id) {
                            return;
                        }
                        let Some(target) = event_target_node(&e) else {
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
                            (on_dismiss.borrow_mut())();
                        }
                    },
                ))
            }
        });

        use_drop(move || {
            set_membership(&mut stack.0.borrow_mut(), id, false);
        });
    }
    #[cfg(not(feature = "web"))]
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
/// Scrolls that originate inside `panel_el` (a scrollable option list, touch
/// scrolling the panel, `scrollIntoView` on focus) are the panel's own and do
/// not dismiss it.
pub(crate) fn use_dismiss_on_viewport_change(
    is_open: ReadSignal<bool>,
    panel_el: Signal<Option<web_sys::Element>>,
    on_dismiss: impl FnMut() + 'static,
) {
    #[cfg(feature = "web")]
    {
        let on_dismiss: Rc<RefCell<dyn FnMut()>> = use_hook(|| Rc::new(RefCell::new(on_dismiss)));

        use_listeners_while_open(is_open, move || {
            let win = web_sys::window()?;
            let on_dismiss = on_dismiss.clone();
            Some(DomListeners::attach(
                win.into(),
                &[("scroll", true), ("resize", false)],
                move |e| {
                    let inside_panel = match (event_target_node(&e), panel_el.peek().as_ref()) {
                        (Some(target), Some(panel)) => panel.contains(Some(&target)),
                        _ => false,
                    };
                    if !inside_panel {
                        (on_dismiss.borrow_mut())();
                    }
                },
            ))
        });
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = (is_open, panel_el, on_dismiss);
    }
}

#[cfg(feature = "web")]
fn event_target_node(e: &web_sys::Event) -> Option<web_sys::Node> {
    e.target()?.dyn_into::<web_sys::Node>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_moves_to_top_and_closing_removes() {
        let mut stack = Vec::new();
        set_membership(&mut stack, 1, true);
        set_membership(&mut stack, 2, true);
        assert!(is_top(&stack, 2));
        assert!(!is_top(&stack, 1));

        // Re-opening an already-open overlay brings it to the top.
        set_membership(&mut stack, 1, true);
        assert_eq!(stack, vec![2, 1]);

        // A closed overlay is not on the stack at all, so a mounted-but-closed
        // overlay never shadows an open one.
        set_membership(&mut stack, 1, false);
        assert!(is_top(&stack, 2));
        set_membership(&mut stack, 3, false);
        assert_eq!(stack, vec![2]);
    }

    #[test]
    fn empty_stack_has_no_top() {
        assert!(!is_top(&[], 0));
    }
}
