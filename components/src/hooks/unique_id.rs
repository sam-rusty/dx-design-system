//! Runtime-unique element ids for ARIA wiring. Replaces ad-hoc per-component
//! atomic counters and the `js_sys::Math::random()` id in `portal.rs` (a
//! hydration-mismatch hazard).
//!
//! The id is derived from the component's [`ScopeId`], which dioxus assigns in
//! tree order and which is therefore identical across an SSR/hydration pair as
//! long as the rendered tree matches — the same property dioxus relies on for
//! its own node ids. A process-global monotonic counter (the previous approach)
//! is *not* reset per request on the server, so its value depends on how many
//! components rendered earlier in the long-lived server process; the client's
//! fresh WASM counter starts at 0, the two diverge, and hydration / ARIA wiring
//! (`aria-controls`, `aria-activedescendant`, …) point at the wrong ids.
//!
//! A component scope yields one id; a component that needs several ids derives
//! them by suffixing this base (see `tabs.rs`, `accordion.rs`, `dropdown.rs`).

use dioxus::prelude::*;

fn format_id(scope: ScopeId) -> String {
    format!("cmp-{}", scope.0)
}

/// Generate a runtime-unique id, fixed for the lifetime of the component.
pub(crate) fn use_unique_id() -> Signal<String> {
    let initial = use_hook(|| format_id(dioxus::core::current_scope_id()));
    use_signal(|| initial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_prefixed_and_distinct() {
        assert_eq!(format_id(ScopeId::ROOT), "cmp-0");
        assert_eq!(format_id(ScopeId(7)), "cmp-7");
        assert_ne!(format_id(ScopeId(1)), format_id(ScopeId(2)));
    }
}
