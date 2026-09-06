use dioxus::prelude::*;

use crate::hooks::use_focus_restore;
use crate::spinner::Spinner;

/// Full-viewport busy overlay.
///
/// While `when` is `true` it covers the page, takes focus (so a screen reader
/// announces the message and keyboard input stops reaching the page beneath)
/// and locks body scroll; when `when` flips back focus returns to the element
/// that had it. The scroll lock is reference-counted app-wide, so two
/// overlays (or an overlay plus another locker) never unlock each other.
#[component]
pub fn LoadingOverlay(
    when: ReadSignal<bool>,
    #[props(default = "Processing\u{2026}")] message: &'static str,
) -> Element {
    // Hooks run before the early return so their count is stable across renders.
    // Order matters: the focus save runs in an effect queued *before* the
    // self-focus effect below, so it records the previously focused element.
    use_focus_restore(when);
    // Mutated only in the web-gated `onmounted` below.
    #[allow(unused_mut)]
    let mut overlay_el = use_signal(|| None::<web_sys::Element>);

    #[cfg(feature = "web")]
    {
        use std::cell::Cell;
        use std::rc::Rc;
        use wasm_bindgen::JsCast;

        use_effect(move || {
            if when()
                && let Some(el) = overlay_el()
                && let Some(html) = el.dyn_ref::<web_sys::HtmlElement>()
            {
                let _ = html.focus();
            }
        });

        let lock = use_hook(scroll_lock);
        let held: Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(false)));
        use_effect({
            let lock = lock.clone();
            let held = held.clone();
            move || {
                let want = when();
                if want && !held.replace(true) {
                    if lock.acquire() {
                        set_body_scroll_locked(true);
                    }
                } else if !want && held.replace(false) && lock.release() {
                    set_body_scroll_locked(false);
                }
            }
        });
        use_drop(move || {
            if held.replace(false) && lock.release() {
                set_body_scroll_locked(false);
            }
        });
    }

    if !when() {
        return rsx! {};
    }

    rsx! {
        div {
            role: "status",
            "aria-live": "polite",
            "aria-busy": "true",
            tabindex: "-1",
            class: "fixed inset-0 z-[200] flex items-center justify-center bg-background/75 backdrop-blur-sm animate-in fade-in duration-150 outline-none",
            onmounted: move |e| {
                #[cfg(feature = "web")]
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    overlay_el.set(Some(el.clone()));
                }
                #[cfg(not(feature = "web"))]
                let _ = (&e, &overlay_el);
            },
            div { class: "flex flex-col items-center gap-3 rounded-2xl border border-border bg-card shadow-2xl px-12 py-8",
                Spinner { class: "size-10 text-primary" }
                p { class: "text-sm font-medium text-foreground", "{message}" }
            }
        }
    }
}

/// App-wide count of components currently holding the body scroll lock.
/// `acquire`/`release` report the transitions that actually change the DOM
/// class, so the lock survives until the last holder lets go.
#[derive(Default)]
#[cfg_attr(not(feature = "web"), allow(dead_code))]
struct ScrollLockCount(std::cell::Cell<usize>);

#[cfg_attr(not(feature = "web"), allow(dead_code))]
impl ScrollLockCount {
    /// Returns `true` when this call took the lock from unlocked to locked.
    fn acquire(&self) -> bool {
        let n = self.0.get() + 1;
        self.0.set(n);
        n == 1
    }

    /// Returns `true` when this call released the last hold.
    fn release(&self) -> bool {
        let n = self.0.get();
        if n == 0 {
            return false;
        }
        self.0.set(n - 1);
        n == 1
    }
}

/// The shared counter lives in the root scope so overlays in unrelated
/// subtrees agree on it.
#[cfg(feature = "web")]
fn scroll_lock() -> std::rc::Rc<ScrollLockCount> {
    crate::hooks::root_context(std::rc::Rc::default)
}

#[cfg(feature = "web")]
fn set_body_scroll_locked(locked: bool) {
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        let list = body.class_list();
        let _ = if locked {
            list.add_1("overflow-hidden")
        } else {
            list.remove_1("overflow-hidden")
        };
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollLockCount;

    #[test]
    fn lock_changes_only_on_first_acquire_and_last_release() {
        let lock = ScrollLockCount::default();
        assert!(lock.acquire());
        assert!(!lock.acquire());
        assert!(!lock.release());
        assert!(lock.release());
    }

    #[test]
    fn release_without_hold_is_a_noop() {
        let lock = ScrollLockCount::default();
        assert!(!lock.release());
        assert!(lock.acquire());
        assert!(lock.release());
        assert!(!lock.release());
    }
}
