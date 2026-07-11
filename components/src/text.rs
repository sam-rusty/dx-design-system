use dioxus::prelude::*;
use utils::format::merge;

const BASE: &str = "tracking-tight text-foreground antialiased";

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextVariant {
    #[default]
    Default,
    Secondary,
    Strong,
    Italic,
    Underlined,
}

impl TextVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Secondary => "text-muted-foreground",
            Self::Strong => "font-bold",
            Self::Italic => "italic",
            Self::Underlined => "underline underline-offset-4",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    #[default]
    Default,
    Small,
    Large,
}

impl TextSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "text-base",
            Self::Small => "text-sm",
            Self::Large => "text-lg",
        }
    }
}

#[component]
pub fn Text(
    #[props(default)] variant: TextVariant,
    #[props(default)] size: TextSize,
    #[props(default)] class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let computed_class = merge(&[BASE, variant.class(), size.class(), &class]);

    rsx! {
        p { class: "{computed_class}", "data-name": "Text",
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}
