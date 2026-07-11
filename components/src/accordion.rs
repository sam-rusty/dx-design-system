use dioxus::prelude::*;

use crate::hooks::{
    FocusState, use_focus_control_disabled, use_focus_entry_disabled, use_focus_provider,
    use_unique_id,
};
use crate::icon::{Icon, IconName};

#[derive(Clone, PartialEq)]
pub struct AccordionItem {
    pub key: &'static str,
    pub label: Element,
    pub children: Element,
    pub disabled: bool,
    /// Optional right-aligned summary shown only while the section is collapsed
    /// (e.g. "2 phones · 1 email"). `None` renders nothing.
    pub summary: Option<Element>,
}

/// Collapsible section list.
///
/// - `accordion: true` — at most one panel open at a time (default).
/// - `accordion: false` — multiple panels may be open simultaneously.
///
/// Controlled: pass `active_keys` + `on_change`. Uncontrolled: pass `default_active_keys`.
#[component]
pub fn Accordion(
    items: Vec<AccordionItem>,
    /// When true, opening one panel closes all others.
    #[props(default = true)]
    accordion: bool,
    #[props(default)] class: String,
    /// Render a 1-based step badge before each label and apply the active-step styling.
    #[props(default)]
    numbered: bool,
    /// Uncontrolled: keys open on first render.
    #[props(default)]
    default_active_keys: Vec<&'static str>,
    /// Controlled: signal holding currently open keys.
    #[props(default)]
    active_keys: Option<Signal<Vec<&'static str>>>,
    /// Fired when open-key set changes. Receives the new complete set.
    #[props(default)]
    on_change: Option<EventHandler<Vec<&'static str>>>,
) -> Element {
    let mut internal_keys: Signal<Vec<&'static str>> =
        use_signal(move || default_active_keys.clone());

    #[allow(clippy::redundant_closure)]
    let current_keys = active_keys.map(|s| s()).unwrap_or_else(|| internal_keys());

    use_focus_provider(use_signal(|| true).into());

    let classes =
        format!("divide-y divide-border rounded-xl border border-border overflow-hidden {class}",);

    rsx! {
        div { class: classes,
            for (index , item) in items.iter().enumerate() {
                AccordionSection {
                    key: "{item.key}",
                    index,
                    numbered,
                    label: item.label.clone(),
                    summary: item.summary.clone(),
                    children: item.children.clone(),
                    is_open: current_keys.contains(&item.key),
                    disabled: item.disabled,
                    on_toggle: {
                        let key = item.key;
                        move |_| {
                            // Read the live set at click time so we never clone it per render.
                            #[allow(clippy::redundant_closure)]
                            let mut keys =
                                active_keys.map(|s| s()).unwrap_or_else(|| internal_keys());
                            if let Some(pos) = keys.iter().position(|k| *k == key) {
                                keys.remove(pos);
                            } else if accordion {
                                keys = vec![key];
                            } else {
                                keys.push(key);
                            }
                            if active_keys.is_none() {
                                internal_keys.set(keys.clone());
                            }
                            if let Some(cb) = on_change {
                                cb.call(keys);
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn AccordionSection(
    index: usize,
    numbered: bool,
    label: Element,
    summary: Option<Element>,
    children: Element,
    is_open: bool,
    disabled: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    let mut focus = use_context::<FocusState>();
    let uid = use_unique_id();
    let base = uid.peek().clone();
    let header_id = format!("{base}-header");
    let panel_id = format!("{base}-panel");

    let index_sig = use_signal(|| index);
    use_focus_entry_disabled(focus, index_sig, move || disabled);
    let onmounted = use_focus_control_disabled(focus, index_sig, move || disabled);

    let idx = index_sig();
    let is_entry =
        focus.is_focused(idx) || (!focus.any_focused() && focus.recent_focus_or_default() == idx);
    let tabindex = if is_entry { "0" } else { "-1" };

    let chevron_class = if is_open {
        "size-4 text-muted-foreground transition-transform duration-200 rotate-180"
    } else {
        "size-4 text-muted-foreground transition-transform duration-200"
    };
    let row_class = if is_open && numbered {
        // Active-step styling is opt-in via `numbered`; non-numbered accordions keep the
        // flat open row so existing call sites render unchanged.
        "relative flex w-full items-center gap-3 px-4 py-3.5 text-sm font-medium text-foreground \
         bg-muted/30 cursor-pointer select-none text-left transition-colors"
    } else if disabled {
        "relative flex w-full items-center gap-3 px-4 py-3.5 text-sm font-medium \
         text-muted-foreground opacity-50 cursor-not-allowed select-none text-left"
    } else {
        "relative flex w-full items-center gap-3 px-4 py-3.5 text-sm font-medium \
         text-foreground hover:bg-muted/20 transition-colors cursor-pointer select-none text-left"
    };

    let badge_class = if is_open {
        "flex size-6 shrink-0 items-center justify-center rounded-full bg-primary \
         text-primary-foreground text-[11px] font-semibold transition-colors"
    } else {
        "flex size-6 shrink-0 items-center justify-center rounded-full border border-border \
         text-muted-foreground text-[11px] font-semibold transition-colors"
    };

    rsx! {
        div {
            button {
                r#type: "button",
                id: "{header_id}",
                class: row_class,
                disabled,
                tabindex,
                "aria-expanded": is_open,
                "aria-controls": "{panel_id}",
                onmounted,
                onkeydown: move |e: KeyboardEvent| {
                    match e.key() {
                        Key::ArrowDown => {
                            e.prevent_default();
                            focus.focus_next();
                        }
                        Key::ArrowUp => {
                            e.prevent_default();
                            focus.focus_prev();
                        }
                        Key::Home => {
                            e.prevent_default();
                            focus.focus_first();
                        }
                        Key::End => {
                            e.prevent_default();
                            focus.focus_last();
                        }
                        _ => {}
                    }
                },
                onclick: move |_| {
                    if !disabled {
                        on_toggle.call(());
                    }
                },

                if is_open && numbered {
                    span { class: "absolute left-0 top-0 bottom-0 w-[3px] rounded-r bg-primary" }
                }
                if numbered {
                    span { class: badge_class, "{index + 1}" }
                }
                span { class: "min-w-0 flex-1", {label} }
                if !is_open {
                    if let Some(s) = summary.clone() {
                        span { class: "ml-auto max-w-[45%] truncate text-right text-sm text-muted-foreground",
                            {s}
                        }
                    }
                }
                Icon { name: IconName::ChevronDown, class: chevron_class }
            }
            if is_open {
                div {
                    id: "{panel_id}",
                    role: "region",
                    "aria-labelledby": "{header_id}",
                    class: "px-4 pb-5 pt-1 text-sm text-foreground",
                    {children}
                }
            }
        }
    }
}
