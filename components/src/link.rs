use crate::button::BUTTON_BASE_CLASS;
use crate::{ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use ds_utils::format::merge;

#[derive(Default, Clone, PartialEq)]
pub enum LinkType {
    Button,
    #[default]
    Link,
}

#[component]
pub fn Link(
    #[props(into)] to: NavigationTarget,
    #[props(optional)] title: Option<String>,
    #[props(optional)] r#type: Option<LinkType>,
    #[props(optional)] class: String,
    #[props(optional)] new_tab: bool,
    #[props(optional)] onclick_only: bool,
    #[props(optional)] active_class: Option<String>,
    #[props(optional)] style: Option<String>,
    #[props(optional)] onclick: EventHandler<MouseEvent>,
    #[props(optional)] variant: ButtonVariant,
    #[props(optional)] size: ButtonSize,
    children: Element,
) -> Element {
    let is_button = r#type
        .map(|t| matches!(t, LinkType::Button))
        .unwrap_or_default();

    let class = if is_button {
        merge(&[
            "cursor-pointer",
            &class,
            BUTTON_BASE_CLASS,
            variant.class(),
            size.class(),
        ])
    } else {
        merge(&["cursor-pointer", &class])
    };

    rsx! {
        dioxus::prelude::Link {
            to,
            onclick_only,
            active_class,
            onclick,
            class,
            new_tab,
            title,
            style,
            {children}
        }
    }
}
