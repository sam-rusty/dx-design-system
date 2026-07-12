use dioxus::prelude::*;
use ds_utils::format::merge;

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
pub enum TextTone {
    #[default]
    Default,
    Muted,
    Primary,
    Warning,
    Destructive,
    Success,
}

impl TextTone {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Muted => "text-muted-foreground",
            Self::Primary => "text-primary",
            Self::Warning => "text-warning",
            Self::Destructive => "text-destructive",
            Self::Success => "text-success",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextWeight {
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
}

impl TextWeight {
    fn class(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Medium => "font-medium",
            Self::Semibold => "font-semibold",
            Self::Bold => "font-bold",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    Xs,
    Small,
    #[default]
    Default,
    Large,
}

impl TextSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "text-xs",
            Self::Small => "text-sm",
            Self::Default => "text-base",
            Self::Large => "text-lg",
        }
    }
}

#[component]
pub fn Text(
    #[props(default)] variant: TextVariant,
    #[props(default)] size: TextSize,
    #[props(default)] tone: TextTone,
    #[props(default)] weight: TextWeight,
    #[props(default)] inline: bool,
    #[props(default)] class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let computed_class = merge(&[
        BASE,
        size.class(),
        tone.class(),
        weight.class(),
        variant.class(),
        &class,
    ]);

    if inline {
        rsx! {
            span {
                class: "{computed_class}",
                "data-name": "Text",
                onclick: move |e| onclick.call(e),
                {children}
            }
        }
    } else {
        rsx! {
            p {
                class: "{computed_class}",
                "data-name": "Text",
                onclick: move |e| onclick.call(e),
                {children}
            }
        }
    }
}
