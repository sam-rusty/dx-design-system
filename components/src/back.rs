use components::{Icon, IconName, Link};
use dioxus::prelude::*;

const BASE_CLASS: &str = "flex inline-flex items-center gap-2 mb-2 text-muted-foreground font-medium hover:text-foreground transition-colors";

#[component]
pub fn Back(#[props(into)] to: NavigationTarget, #[props(default)] class: String) -> Element {
    let route = router();
    let class = format!("{BASE_CLASS} {class}");
    let can_go_back = route.can_go_back();

    rsx! {
        Link { class, to, onclick_only: can_go_back,
            onclick: move |_| route.go_back(),
            Icon { name: IconName::ArrowLeft, class: "size-4" }
            "Back"
        }
    }
}
