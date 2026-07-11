//! App-wide UI context: toast stack, dropdown coordination, and [`Toaster`] mount.

use dioxus::prelude::*;

use crate::dropdown::DropdownMenuCoordinatorProvider;
use crate::toast::{ToastStore, Toaster};

/// Provides [`ToastStore`], [`DropdownMenuCoordinatorProvider`], and a single [`Toaster`].
/// Use once at the app root around the router.
#[component]
pub fn AppShellProvider(children: Element) -> Element {
    use_context_provider(ToastStore::new);
    rsx! {
        DropdownMenuCoordinatorProvider { {children} }
        Toaster {}
    }
}
