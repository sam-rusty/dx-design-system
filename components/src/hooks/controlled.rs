//! Controlled-or-uncontrolled state, ported from `dioxus_primitives`'s
//! `use_controlled` (`primitives/src/lib.rs`).
//!
//! A stateful widget should work both ways: the caller may *control* the value
//! by passing `Some(value)` + handling `on_change`, or leave it *uncontrolled*
//! (pass `None`) and let the widget own its state internally. This removes the
//! `use_effect`-prop-sync anti-pattern flagged across the audit (textarea, tabs,
//! popover, the date pickers, `MultiSearch`, …).

use dioxus::prelude::*;

/// The controlled-or-uncontrolled prop trio that always travels together.
///
/// `value` is the (optional) externally-controlled value, `default` the initial
/// value used when uncontrolled, and `on_change` is fired on every mutation.
#[derive(Clone, Copy)]
pub(crate) struct Controlled<T: Clone + PartialEq + 'static> {
    pub(crate) value: ReadSignal<Option<T>>,
    pub(crate) default: ReadSignal<T>,
    pub(crate) on_change: Callback<T>,
}

/// Allow some state to be either controlled or uncontrolled.
///
/// Returns the current value (reactive) and a setter. When `prop` is `Some`, the
/// returned value tracks it (controlled); otherwise it tracks an internal signal
/// seeded with `default` (uncontrolled). The setter always updates the internal
/// signal *and* calls `on_change`, so a controlled parent stays the source of
/// truth while an uncontrolled widget still works.
pub(crate) fn use_controlled<T: Clone + PartialEq + 'static>(
    prop: ReadSignal<Option<T>>,
    default: T,
    on_change: Callback<T>,
) -> (Memo<T>, Callback<T>) {
    let mut internal_value = use_signal(|| prop.cloned().unwrap_or(default));
    let value = use_memo(move || prop.cloned().unwrap_or_else(&*internal_value));

    let set_value = use_callback(move |x: T| {
        internal_value.set(x.clone());
        on_change.call(x);
    });

    (value, set_value)
}
