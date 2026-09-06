//! Mirror a plain prop into a signal that follows later prop changes.
//!
//! Effects and memos capture their inputs once, so a component that reads a
//! prop inside one would keep the first render's value. Mirroring the prop
//! through a signal (compare-and-set, so an unchanged prop never notifies
//! subscribers) lets the effect/memo subscribe to the current value instead.

use dioxus::prelude::*;

/// A signal seeded with `value` and updated whenever `value` differs from the
/// last one seen. Pass a `Clone + PartialEq` prop; read the signal in effects
/// and memos.
pub(crate) fn use_prop_signal<T: Clone + PartialEq + 'static>(value: T) -> Signal<T> {
    let mut sig = use_signal(|| value.clone());
    if *sig.peek() != value {
        sig.set(value);
    }
    sig
}

/// Fetch (or lazily create) an app-wide singleton stored in the root scope, so
/// unrelated subtrees share one instance (overlay stacks, the body scroll-lock
/// count). A scope-local `provide_context` would give each subtree its own.
pub(crate) fn root_context<T: Clone + 'static>(make: impl FnOnce() -> T) -> T {
    try_consume_context::<T>().unwrap_or_else(|| dioxus::core::provide_root_context(make()))
}
