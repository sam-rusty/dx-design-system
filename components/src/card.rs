use dioxus::prelude::*;
use utils::format::merge;

const BASE_CLASS: &str = "rounded-xl min-w-0 min-h-0 bg-card text-card-foreground relative";
const DEFAULT_PADDING: &str = "p-4 sm:p-6";
const COMPACT_PADDING: &str = "p-3 sm:p-4";
const SHADOW: &str = "box-shadow: var(--shadow) 0px 8px 24px;";

#[component]
pub fn Card(
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<()>>,
    #[props(default)] full_height: bool,
    #[props(default)] compact: bool,
    children: Element,
) -> Element {
    let height = if full_height { "h-full" } else { "" };
    let padding = if compact {
        COMPACT_PADDING
    } else {
        DEFAULT_PADDING
    };
    let clickable = onclick.is_some();
    let outer = merge(&["rounded-xl min-w-0", &class]);
    let inner = merge(&[BASE_CLASS, padding, height]);

    rsx! {
        div {
            class: "{outer}",
            style: SHADOW,
            role: clickable.then_some("button"),
            tabindex: clickable.then_some("0"),
            onclick: move |_| {
                if let Some(cb) = onclick {
                    cb.call(());
                }
            },
            onkeydown: move |e: KeyboardEvent| {
                let Some(cb) = onclick else { return };
                match e.key() {
                    Key::Enter => cb.call(()),
                    Key::Character(c) if c == " " => {
                        e.prevent_default();
                        cb.call(());
                    }
                    _ => {}
                }
            },
            div { class: "{inner}", {children} }
        }
    }
}
