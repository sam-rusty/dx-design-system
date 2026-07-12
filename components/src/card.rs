use dioxus::prelude::*;
use ds_utils::format::merge;

const BASE_CLASS: &str = "rounded-xl min-w-0 min-h-0 bg-card text-card-foreground relative";
const DEFAULT_PADDING: &str = "p-4 sm:p-6";
const COMPACT_PADDING: &str = "p-3 sm:p-4";
const SHADOW: &str = "box-shadow: var(--shadow) 0px 8px 24px;";

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum CardVariant {
    /// Solid panel with a soft shadow + hairline border.
    #[default]
    Card,
    /// Elevated panel with a soft lift. Use for selected / hovered / focused surfaces.
    Elevated,
    /// Recessed inset track. Flat, no blur or shadow.
    Tile,
    /// Border + radius only, transparent background.
    Plain,
}

impl CardVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Card => {
                "bg-card rounded-2xl shadow-soft border-[.5px] border-[color:var(--glass-hair)]"
            }
            Self::Elevated => {
                "bg-glass backdrop-blur-[24px] backdrop-saturate-[1.5] rounded-2xl shadow-[0_8px_18px_rgba(23,20,17,.07),inset_0_1px_0_var(--color-hi)]"
            }
            Self::Tile => "bg-glass rounded-2xl",
            Self::Plain => "rounded-2xl",
        }
    }
}

#[component]
pub fn Card(
    /// When set, renders a single surface div with the variant's classes (glass
    /// family) and lets the caller own padding. When `None`, renders the default
    /// padded elevated panel.
    #[props(default)]
    variant: Option<CardVariant>,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<()>>,
    #[props(default)] full_height: bool,
    #[props(default)] compact: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    if let Some(variant) = variant {
        let class = merge(&[variant.class(), &class]);
        return rsx! {
            div {
                class,
                "data-name": "Box",
                onclick: move |_| {
                    if let Some(cb) = onclick {
                        cb.call(());
                    }
                },
                ..attributes,
                {children}
            }
        };
    }

    let height = if full_height { "h-full" } else { "" };
    let padding = if compact {
        COMPACT_PADDING
    } else {
        DEFAULT_PADDING
    };
    let clickable = onclick.is_some();
    let outer = merge(&["rounded-xl min-w-0", &class]);
    let inner = merge(&[BASE_CLASS, padding, height]);

    rsx! {
        div {
            class: "{outer}",
            style: SHADOW,
            role: clickable.then_some("button"),
            tabindex: clickable.then_some("0"),
            onclick: move |_| {
                if let Some(cb) = onclick {
                    cb.call(());
                }
            },
            onkeydown: move |e: KeyboardEvent| {
                let Some(cb) = onclick else { return };
                match e.key() {
                    Key::Enter => cb.call(()),
                    Key::Character(c) if c == " " => {
                        e.prevent_default();
                        cb.call(());
                    }
                    _ => {}
                }
            },
            ..attributes,
            div { class: "{inner}", {children} }
        }
    }
}
