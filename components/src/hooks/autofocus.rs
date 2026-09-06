//! Gated autofocus for text controls.
//!
//! A screen that animates in (mobile push transition) provides an
//! [`AutofocusGate`] that flips to `true` once the enter transition settles, so
//! the keyboard slide does not fight the push animation. Apps that never provide
//! a gate get plain on-mount autofocus.

use std::rc::Rc;

use dioxus::prelude::*;

/// Context that defers `autofocus` until its signal reads `true`.
#[derive(Clone, Copy)]
pub struct AutofocusGate(pub ReadSignal<bool>);

/// Focus the mounted element once, when `autofocus` is set and the
/// [`AutofocusGate`] (if any) opens. Wire the returned handler to `onmounted`.
pub(crate) fn use_gated_autofocus(autofocus: bool) -> Callback<MountedEvent> {
    let gate = try_consume_context::<AutofocusGate>();
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    // Written only from inside the effect; a plain cell avoids re-running the
    // effect on our own write.
    let fired = use_hook(|| Rc::new(std::cell::Cell::new(false)));

    use_effect(move || {
        if !autofocus || fired.get() || gate.is_some_and(|g| !(g.0)()) {
            return;
        }
        let Some(md) = mounted() else { return };
        fired.set(true);
        spawn(async move {
            let _ = md.set_focus(true).await;
        });
    });

    use_callback(move |evt: MountedEvent| mounted.set(Some(evt.data())))
}
