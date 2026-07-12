use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::icon::IconName;
use crate::icon_bubble::IconBubble;

#[component]
pub fn SectionHeaderTitle(
    icon: IconName,
    title: &'static str,
    #[props(default)] count: Option<usize>,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-2.5",
            IconBubble { icon: icon }
            div { class: "flex items-center gap-2",
                p { class: "text-sm font-semibold text-foreground", "{title}" }
                if let Some(count_val) = count {
                    if count_val > 0 {
                        span {
                            "aria-label": "{count_val} items",
                            class: "inline-flex items-center justify-center min-w-[20px] h-5 px-1 rounded-full bg-muted text-xs font-medium text-muted-foreground",
                            "{count_val}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SectionHeader(
    icon: IconName,
    title: &'static str,
    #[props(default)] count: Option<usize>,
    #[props(default)] children: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    let wrapper_class = merge(&["flex items-center justify-between", &class]);
    rsx! {
        div { class: "{wrapper_class}",
            SectionHeaderTitle {
                icon: icon,
                title: title,
                count: count,
            }
            if let Some(c) = children {
                {c}
            }
        }
    }
}
