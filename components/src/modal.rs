use components::{Button, Icon, IconName, Portal, Title, TitleSize};
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

const PANEL_BASE: &str = "relative w-full mx-4 flex flex-col rounded-2xl border border-border bg-card shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 max-h-[85vh]";

#[component]
pub fn Modal(
    #[props(optional)] title: String,
    on_close: EventHandler<()>,
    #[props(default)] headerless: bool,
    #[props(default)] size: ModalSize,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    // The modal is mounted only while open, so drive `use_overlay` with a signal
    // that flips false on Escape / outside-pointer, then forward that to
    // `on_close`. `use_overlay` also saves the trigger's focus and restores it
    // when the modal unmounts.
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
                    onmounted: move |e| {
                        if let Some(el) = e.downcast::<web_sys::Element>() {
                            root_el.set(Some(el.clone()));
                            #[cfg(target_arch = "wasm32")]
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
                    {children}
                }
            }
        }
    }
}
