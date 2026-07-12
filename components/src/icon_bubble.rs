use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::icon::{Icon, IconName};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum IconBubbleSize {
    #[default]
    Md,
    Sm,
    Lg,
}

impl IconBubbleSize {
    fn wrapper_class(self) -> &'static str {
        match self {
            Self::Sm => "size-6",
            Self::Md => "size-7",
            Self::Lg => "size-9",
        }
    }

    fn icon_class(self) -> &'static str {
        match self {
            Self::Sm => "size-3",
            Self::Md => "size-3.5",
            Self::Lg => "size-[18px]",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum IconBubbleColor {
    #[default]
    Primary,
    Muted,
}

impl IconBubbleColor {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "bg-primary/10 text-primary",
            Self::Muted => "bg-muted text-muted-foreground",
        }
    }
}

#[component]
pub fn IconBubble(
    icon: IconName,
    #[props(default)] size: IconBubbleSize,
    #[props(default)] color: IconBubbleColor,
    #[props(default)] class: String,
) -> Element {
    let wrapper = merge(&[
        "flex items-center justify-center rounded-lg shrink-0",
        size.wrapper_class(),
        color.class(),
        &class,
    ]);
    let icon_cls = size.icon_class().to_string();

    rsx! {
        div { class: "{wrapper}",
            Icon { name: icon, class: icon_cls }
        }
    }
}
