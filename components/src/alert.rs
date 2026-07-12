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
            AlertVariant::Destructive => "bg-destructive/8",
            AlertVariant::Warning => "bg-warning/8",
            AlertVariant::Info => "bg-primary/8",
            AlertVariant::Success => "bg-success/8",
        }
    }

    fn icon_class(self) -> &'static str {
        match self {
            AlertVariant::Destructive => "text-destructive",
            AlertVariant::Warning => "text-warning",
            AlertVariant::Info => "text-primary",
            AlertVariant::Success => "text-success",
        }
    }

    fn icon(self) -> IconName {
        match self {
            AlertVariant::Destructive => IconName::OctagonAlertFilled,
            AlertVariant::Warning => IconName::TriangleAlertFilled,
            AlertVariant::Info => IconName::InfoFilled,
            AlertVariant::Success => IconName::CheckCircleFilled,
        }
    }
}

const BASE_CLASS: &str = "flex items-start gap-3 rounded-xl px-3.5 py-3";
const ICON_CLASS: &str = "size-5 shrink-0";
const TEXT_CLASS: &str = "text-sm font-medium text-foreground leading-snug";

#[component]
pub fn Alert(
    #[props(default)] variant: AlertVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let class = merge(&[BASE_CLASS, variant.container_class(), &class]);
    let icon_class = merge(&[ICON_CLASS, variant.icon_class()]);

    rsx! {
        div { class, role: "alert",
            Icon { name: variant.icon(), class: icon_class }
            span { class: TEXT_CLASS, {children} }
        }
    }
}
