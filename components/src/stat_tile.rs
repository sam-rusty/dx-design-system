use dioxus::prelude::*;
use ds_utils::format::merge;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum StatTone {
    #[default]
    Default,
    Success,
    Destructive,
}

impl StatTone {
    fn value_class(self) -> &'static str {
        match self {
            Self::Default => "text-foreground",
            Self::Success => "text-success",
            Self::Destructive => "text-destructive",
        }
    }
}

/// Label + big number tile.
#[component]
pub fn StatTile(
    #[props(into)] label: String,
    #[props(into)] value: String,
    #[props(default)] sub: Option<String>,
    #[props(default)] tone: StatTone,
    #[props(default)] class: String,
) -> Element {
    let root = merge(&["bg-muted border border-border rounded-xl px-4 py-3", &class]);
    let value_class = merge(&[
        "text-2xl font-semibold tabular-nums tracking-tight mt-1",
        tone.value_class(),
    ]);
    rsx! {
        div { class: root,
            div { class: "text-xs font-medium text-muted-foreground", "{label}" }
            div { class: value_class, "{value}" }
            if let Some(s) = sub {
                div { class: "text-xs text-muted-foreground mt-0.5", "{s}" }
            }
        }
    }
}
