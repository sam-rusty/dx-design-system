use dioxus::prelude::*;
use ds_utils::format::merge;

const LABEL_BASE: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none \
     peer-disabled:cursor-not-allowed peer-disabled:opacity-50 \
     group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50";

#[component]
pub fn Label(
    #[props(default)] class: String,
    #[props(default)] html_for: String,
    #[props(default)] id: Option<String>,
    #[props(default)] data_name: Option<String>,
    children: Element,
) -> Element {
    // Avoid an allocation on the common (no caller class) path.
    let class = if class.is_empty() {
        LABEL_BASE.to_string()
    } else {
        merge(&[LABEL_BASE, &class])
    };

    rsx! {
        label { class: "{class}", r#for: "{html_for}", id, "data-name": data_name,
            {children}
        }
    }
}
