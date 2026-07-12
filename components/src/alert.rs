use crate::{Icon, IconName};
use dioxus::prelude::*;
use ds_utils::format::merge;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Destructive,
    Warning,
    Info,
    Success,
}

impl AlertVariant {
    fn container_class(self) -> &'static str {
        match self {
            AlertVariant::Destructive => "border-destructive/40 bg-destructive/10 text-destructive",
            AlertVariant::Warning => "border-warning/40 bg-warning/10 text-warning",
            AlertVariant::Info => "border-primary/40 bg-primary/10 text-primary",
            AlertVariant::Success => "border-success/40 bg-success/10 text-success",
        }
    }

    fn icon(self) -> IconName {
        match self {
            AlertVariant::Destructive | AlertVariant::Warning | AlertVariant::Info => {
                IconName::CircleAlert
            }
            AlertVariant::Success => IconName::Check,
        }
    }
}

const BASE_CLASS: &str = "flex items-start gap-3 rounded-lg border px-4 py-3";
const ICON_CLASS: &str = "mt-0.5 size-4 shrink-0";
const TEXT_CLASS: &str = "text-sm font-medium";

#[component]
pub fn Alert(
    #[props(default)] variant: AlertVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let class = merge(&[BASE_CLASS, variant.container_class(), &class]);

    rsx! {
        div { class, role: "alert",
            Icon { name: variant.icon(), class: ICON_CLASS }
            span { class: TEXT_CLASS, {children} }
        }
    }
}
