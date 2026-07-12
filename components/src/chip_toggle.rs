use dioxus::prelude::*;
use ds_utils::format::merge;

#[component]
pub fn ChipToggle(
    selected: ReadSignal<bool>,
    on_click: EventHandler<()>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    children: Element,
) -> Element {
    let base = merge(&[
        "inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium border transition-colors cursor-pointer",
        &class,
    ]);

    let is_selected = selected();
    let btn_class = if is_selected {
        format!("{base} border-primary bg-primary/10 text-primary")
    } else {
        format!("{base} border-border bg-card text-muted-foreground hover:text-foreground")
    };

    rsx! {
        button {
            r#type: "button",
            "aria-label": aria_label,
            "aria-pressed": "{is_selected}",
            "data-state": if is_selected { "on" } else { "off" },
            class: "{btn_class}",
            onclick: move |_| on_click.call(()),
            {children}
        }
    }
}
