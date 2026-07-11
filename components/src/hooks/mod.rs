//! Reusable behavior hooks shared across the component library.
//!
//! These are styling-agnostic primitives ported from the patterns in
//! `dioxus-primitives` (the first-party headless library) and adapted to this
//! crate's house style (web-sys + `#[cfg(target_arch = "wasm32")]` for DOM
//! access, `pub(crate)` visibility — they are internal infrastructure, not part
//! of the public component API).
//!
//! See `docs/audits/components-audit.md` §3 for the rationale: these hooks are
//! the common denominator behind most of the audit's accessibility,
//! controlled-state, and overlay findings.

mod controlled;
mod dismiss;
mod focus;
mod overlay;
mod timeout;
mod unique_id;

pub(crate) use controlled::use_controlled;
use dioxus::prelude::*;
pub(crate) use dismiss::{
    use_dismiss_on_viewport_change, use_escape_listener, use_outside_dismiss,
    use_outside_dismiss_panel,
};
pub(crate) use focus::{
    FocusState, use_focus_control, use_focus_control_disabled, use_focus_entry_disabled,
    use_focus_provider,
};
pub(crate) use overlay::use_overlay;
pub(crate) use timeout::use_timeout;
pub(crate) use unique_id::use_unique_id;

/// Run `effect`, and run the cleanup it returns before the next run and on
/// unmount. Mirrors `dioxus_primitives`'s `use_effect_with_cleanup`.
pub(crate) fn use_effect_with_cleanup<F, C>(mut effect: F)
where
    F: FnMut() -> C + 'static,
    C: FnOnce() + 'static,
{
    let mut cleanup = use_hook(|| CopyValue::new(None as Option<C>));
    use_effect(move || {
        if let Some(cleanup) = cleanup.take() {
            cleanup();
        }
        cleanup.set(Some(effect()));
    });
    use_drop(move || {
        if let Some(cleanup) = cleanup.take() {
            cleanup();
        }
    });
}
