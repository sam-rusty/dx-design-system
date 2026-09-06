use crate::{Button, Icon, IconName, Portal, Title, TitleSize};
use dioxus::prelude::*;
use strum_macros::AsRefStr;

use crate::hooks::{use_overlay, use_unique_id};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, AsRefStr)]
pub enum ModalSize {
    #[strum(serialize = "max-w-sm")]
    Sm,
    #[strum(serialize = "max-w-lg")]
    #[default]
    Md,
    #[strum(serialize = "max-w-2xl")]
    Lg,
    #[strum(serialize = "max-w-4xl")]
    Xl,
    #[strum(serialize = "max-w-6xl")]
    Xxl,
    #[strum(serialize = "max-w-[calc(100vw-2rem)]")]
    Full,
}

#[cfg(feature = "web")]
use crate::focus::trap_tab;

const PANEL_BASE: &str = "relative w-full mx-4 flex flex-col rounded-2xl border border-border bg-card shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 max-h-[85vh]";

/// Modal dialog rendered through a [`Portal`].
///
/// A `Modal` is open for as long as it is mounted: render it conditionally and
/// unmount it from `on_close` (which fires on Escape, an outside pointer-down,
/// and the header close button). It never hides itself — a `Modal` kept mounted
/// after `on_close` stays visible and, having left the Escape/outside stacks,
/// stops reacting to either.
///
/// Focus moves onto the panel on open, Tab / Shift+Tab cycle within it, and
/// the previously focused element gets focus back on unmount.
#[component]
pub fn Modal(
    #[props(optional)] title: String,
    on_close: EventHandler<()>,
    #[props(default)] headerless: bool,
    /// Drop the default padded body wrapper — for edge-to-edge content that
    /// brings its own padding (e.g. the date pickers).
    #[props(default)]
    unpadded: bool,
    #[props(default)] size: ModalSize,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    // The modal is mounted only while open, so drive `use_overlay` with a signal
    // that flips false on Escape / outside-pointer, then forward that to
    // `on_close`. `use_overlay` also saves the trigger's focus and restores it
    // when the modal unmounts.
    //
    // Ordering note: `use_overlay` reads `document.activeElement` in an effect
    // after this scope's first render. The panel does not exist yet at that
    // point only because `Portal` defers its children by one render (it mounts
    // them from its own effect, and effects run ancestor-first), so the saved
    // element is still the trigger rather than the panel focused in
    // `onmounted` below. Keep the Portal and this hook order if you touch either.
    let open = use_signal(|| true);
    let mut root_el = use_overlay(open);
    use_effect(move || {
        if !open() {
            on_close.call(());
        }
    });

    let title_id = use_unique_id();
    let size_class = size.as_ref();
    let labelledby = if headerless { None } else { Some(title_id()) };

    rsx! {
        Portal {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 animate-in fade-in duration-200",
                div {
                    class: "{PANEL_BASE} {size_class} {class}",
                    role: "dialog",
                    "aria-modal": "true",
                    "aria-labelledby": labelledby,
                    tabindex: -1,
                    onkeydown: move |e| {
                        #[cfg(feature = "web")]
                        if e.key() == Key::Tab
                            && let Some(panel) = root_el.peek().clone()
                            && trap_tab(&panel, e.modifiers().shift())
                        {
                            e.prevent_default();
                        }
                        #[cfg(not(feature = "web"))]
                        let _ = &e;
                    },
                    onmounted: move |e| {
                        if let Some(el) = e.downcast::<web_sys::Element>() {
                            root_el.set(Some(el.clone()));
                            #[cfg(feature = "web")]
                            {
                                use wasm_bindgen::JsCast;
                                if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
                                    let _ = html.focus();
                                }
                            }
                        }
                    },
                    ..attributes,
                    if !headerless {
                        div { class: "flex items-center justify-between px-6 py-4 border-b border-border",
                            div { id: title_id,
                                Title { size: TitleSize::H5, class: "mb-0", "{title}" }
                            }
                            Button {
                                variant: crate::ButtonVariant::Ghost,
                                onclick: move |_| on_close.call(()),
                                Icon { name: IconName::X, class: "size-4" }
                            }
                        }
                    }
                    if unpadded {
                        {children}
                    } else {
                        div { class: "overflow-y-auto px-6 py-5", {children} }
                    }
                }
            }
        }
    }
}
