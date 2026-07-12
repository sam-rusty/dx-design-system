use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::icon::{Icon, IconName};

#[component]
pub fn EmptyState(
    message: &'static str,
    #[props(default)] description: Option<&'static str>,
    #[props(default)] icon: Option<IconName>,
    #[props(default)] class: String,
    #[props(default)] children: Option<Element>,
) -> Element {
    match icon {
        Some(name) => {
            let wrapper = merge(&["flex flex-col items-center gap-3 text-center", &class]);
            rsx! {
                div { class: "{wrapper}",
                    div { class: "flex items-center justify-center size-14 rounded-full bg-muted/60 mx-auto",
                        Icon { name: name, class: "size-7 text-muted-foreground/40", stroke_width: 1.5f32 }
                    }
                    div {
                        p { class: "text-sm font-medium text-foreground", "{message}" }
                        if let Some(d) = description {
                            p { class: "text-sm text-muted-foreground mt-0.5", "{d}" }
                        }
                    }
                    if let Some(c) = children {
                        {c}
                    }
                }
            }
        }
        None => {
            let wrapper = merge(&[
                "py-8 text-center rounded-xl border border-dashed border-border",
                &class,
            ]);
            rsx! {
                div { class: "{wrapper}",
                    p { class: "text-sm text-muted-foreground", "{message}" }
                }
            }
        }
    }
}
