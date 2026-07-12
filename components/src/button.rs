use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::icon::{Icon, IconName};
use crate::spinner::Spinner;

pub(super) const BUTTON_BASE_CLASS: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full text-sm font-semibold transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 cursor-pointer active:scale-[0.97] antialiased";

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Accent,
    Link,
    Warning,
    Success,
    Bordered,
}

impl ButtonVariant {
    pub(crate) fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground hover:opacity-90",
            Self::Destructive => "bg-destructive text-destructive-foreground hover:opacity-90",
            Self::Outline => {
                "border border-border bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground"
            }
            Self::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            Self::Ghost => "text-foreground hover:bg-accent hover:text-accent-foreground",
            Self::Accent => "bg-accent text-accent-foreground hover:bg-accent/80",
            Self::Link => "text-primary underline-offset-4 hover:underline px-0",
            Self::Warning => "bg-warning text-warning-foreground hover:opacity-90",
            Self::Success => "bg-success text-success-foreground hover:opacity-90",
            Self::Bordered => {
                "bg-transparent border border-border text-muted-foreground hover:border-foreground hover:text-foreground"
            }
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
    Badge,
}

impl ButtonSize {
    pub(crate) fn class(self) -> &'static str {
        match self {
            Self::Default => "h-12 px-5",
            Self::Sm => "h-8 px-3 text-xs",
            Self::Lg => "h-12 px-8 text-base",
            Self::Icon => "h-10 w-10",
            Self::Badge => "h-6 px-2.5 text-[10px] uppercase tracking-wider font-bold",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] class: String,
    #[props(default)] disabled: bool,
    /// When true the button is disabled and a leading spinner replaces `icon`.
    #[props(default)]
    loading: bool,
    /// When true, render no base/variant/size classes — only `class`. For bespoke
    /// controls (segmented toggles, chips, nav rows) that need full styling control.
    #[props(default)]
    bare: bool,
    /// Optional leading icon, rendered before `children`. Swapped for a spinner while `loading`.
    #[props(default)]
    icon: Option<IconName>,
    #[props(default = "button")] button_type: &'static str,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = if bare {
        merge(&[&class])
    } else {
        merge(&[BUTTON_BASE_CLASS, variant.class(), size.class(), &class])
    };

    rsx! {
        button {
            r#type: button_type,
            class,
            "data-name": "Button",
            disabled: disabled || loading,
            "aria-busy": loading,
            onclick: move |e| onclick.call(e),
            ..attributes,
            if loading {
                Spinner {}
            } else if let Some(name) = icon {
                Icon { name }
            }
            {children}
        }
    }
}
