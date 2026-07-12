use dioxus::prelude::*;

const CARD_ENABLED: &str =
    "rounded-xl border border-border bg-card shadow-sm transition-all duration-200 overflow-hidden";
const CARD_DISABLED: &str = "rounded-xl border border-dashed border-border bg-card transition-all duration-200 overflow-hidden hover:bg-accent/30";
const ICON_ENABLED: &str = "shrink-0 flex items-center justify-center size-10 rounded-xl bg-primary text-primary-foreground transition-all duration-200";
const ICON_DISABLED: &str = "shrink-0 flex items-center justify-center size-10 rounded-xl bg-muted text-muted-foreground transition-all duration-200";
const TITLE_ENABLED: &str =
    "text-sm font-semibold text-foreground block transition-colors duration-200";
const TITLE_DISABLED: &str =
    "text-sm font-medium text-muted-foreground block transition-colors duration-200";
const TOGGLE_WRAPPER_ENABLED: &str = "relative shrink-0 inline-flex h-5 w-9 rounded-full transition-colors duration-200 ease-in-out bg-primary";
const TOGGLE_WRAPPER_DISABLED: &str = "relative shrink-0 inline-flex h-5 w-9 rounded-full transition-colors duration-200 ease-in-out bg-muted";
const TOGGLE_KNOB_ENABLED: &str = "absolute top-0.5 size-4 rounded-full bg-white shadow-sm transition-transform duration-200 ease-in-out translate-x-[18px]";
const TOGGLE_KNOB_DISABLED: &str = "absolute top-0.5 size-4 rounded-full bg-white shadow-sm transition-transform duration-200 ease-in-out translate-x-0.5";

/// A card that can be toggled on or off. The header row is fully clickable.
/// When enabled, renders `children` below the header in an expanded panel.
#[component]
pub fn ToggleCard(
    icon: Element,
    title: &'static str,
    description: Option<&'static str>,
    #[props(into)] enabled: ReadSignal<bool>,
    on_toggle: EventHandler<()>,
    children: Option<Element>,
) -> Element {
    let is_on = enabled();
    rsx! {
        div { class: if is_on { CARD_ENABLED } else { CARD_DISABLED },
            button {
                r#type: "button",
                "aria-pressed": "{is_on}",
                class: "flex items-center justify-between w-full gap-3 p-4 sm:p-5 text-left cursor-pointer",
                onclick: move |_| on_toggle.call(()),
                div { class: "flex items-center gap-3.5 min-w-0",
                    div { class: if is_on { ICON_ENABLED } else { ICON_DISABLED },
                        {icon}
                    }
                    div { class: "min-w-0",
                        span { class: if is_on { TITLE_ENABLED } else { TITLE_DISABLED },
                            "{title}"
                        }
                        if let Some(desc) = description {
                            span { class: "text-xs text-muted-foreground leading-snug",
                                "{desc}"
                            }
                        }
                    }
                }
                // Decorative switch visual — interactive state lives on the header button
                // (`aria-pressed`). A real `Switch` here would nest a button inside a button.
                span {
                    aria_hidden: "true",
                    class: if is_on { TOGGLE_WRAPPER_ENABLED } else { TOGGLE_WRAPPER_DISABLED },
                    span {
                        aria_hidden: "true",
                        class: if is_on { TOGGLE_KNOB_ENABLED } else { TOGGLE_KNOB_DISABLED },
                    }
                }
            }

            if is_on && let Some(children) = children {
                div { class: "px-4 pb-4 sm:px-5 sm:pb-5",
                    div { class: "border-t border-border pt-4",
                        {children}
                    }
                }
            }
        }
    }
}

#[component]
pub fn Switch(
    checked: bool,
    #[props(default)] on_change: Option<EventHandler<bool>>,
    /// Alias of `on_change` for call sites that name the handler `on_toggle`.
    #[props(default)]
    on_toggle: Option<EventHandler<bool>>,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] checked_children: Option<Element>,
    #[props(default)] unchecked_children: Option<Element>,
) -> Element {
    let wrapper_class = if checked {
        TOGGLE_WRAPPER_ENABLED
    } else {
        TOGGLE_WRAPPER_DISABLED
    };

    rsx! {
        button {
            r#type: "button",
            role: "switch",
            "aria-checked": checked.to_string(),
            class: "inline-flex items-center gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50",
            disabled: disabled || loading,
            onclick: move |_| {
                if let Some(cb) = &on_change {
                    cb.call(!checked);
                }
                if let Some(cb) = &on_toggle {
                    cb.call(!checked);
                }
            },
            if loading {
                div { class: "{wrapper_class}",
                    span { class: "absolute inset-0 flex items-center justify-center",
                        div { class: "size-3 rounded-full border-2 border-white/60 border-t-white animate-spin" }
                    }
                }
            } else {
                div { class: wrapper_class,
                    span {
                        aria_hidden: "true",
                        class: if checked { TOGGLE_KNOB_ENABLED } else { TOGGLE_KNOB_DISABLED },
                    }
                }
            }
            if checked {
                if let Some(label) = &checked_children { {label.clone()} }
            } else {
                if let Some(label) = &unchecked_children { {label.clone()} }
            }
        }
    }
}
