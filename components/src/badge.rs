use dioxus::prelude::*;
use ds_utils::format::merge;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Accent,
    Destructive,
    Success,
    Warning,
    Outline,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-muted text-muted-foreground",
            Self::Primary => "bg-primary/10 text-primary",
            Self::Secondary => "bg-secondary text-secondary-foreground",
            Self::Accent => "bg-accent text-accent-foreground",
            Self::Destructive => "bg-destructive/10 text-destructive",
            Self::Success => "bg-success/10 text-success",
            Self::Warning => "bg-warning/10 text-warning",
            Self::Outline => "border border-border text-muted-foreground bg-transparent",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BadgeSize {
    Xs,
    #[default]
    Sm,
    Md,
    Lg,
}

impl BadgeSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "text-[10px] px-1.5 py-0.5 font-semibold uppercase tracking-wide",
            Self::Sm => "text-xs px-2 py-0.5 font-medium",
            Self::Md => "text-xs px-2.5 py-1 font-medium",
            Self::Lg => "text-sm px-3 py-1.5 font-semibold gap-1.5",
        }
    }
}

const BASE: &str = "inline-flex items-center rounded-full shrink-0";

#[component]
pub fn Badge(
    #[props(default)] variant: BadgeVariant,
    #[props(default)] size: BadgeSize,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let class = merge(&[BASE, variant.class(), size.class(), &class]);

    rsx! {
        span { class, {children} }
    }
}
