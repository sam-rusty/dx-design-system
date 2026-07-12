use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::icon::{Icon, IconName};

#[component]
pub fn Spinner(
    #[props(default)] class: String,
    /// Screen-reader announcement while the spinner is visible.
    #[props(default = "Loading\u{2026}")]
    label: &'static str,
) -> Element {
    let class = merge(&["animate-spin motion-reduce:animate-none", &class]);
    rsx! {
        span { role: "status",
            Icon { name: IconName::Spinner, class: class }
            span { class: "sr-only", "{label}" }
        }
    }
}
