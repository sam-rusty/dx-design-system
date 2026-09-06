use dioxus::prelude::*;
use ds_utils::format::merge;

const LABEL_BASE: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none \
     peer-disabled:cursor-not-allowed peer-disabled:opacity-50 \
     group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50";

/// Form label. `html_for` binds the label to a control id; when empty the
/// attribute is omitted so a wrapped control (`Label { CheckboxBase {} }`)
/// keeps its implicit association (a present-but-empty `for` disables it).
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

    let html_for = (!html_for.is_empty()).then_some(html_for);

    rsx! {
        label { class: "{class}", r#for: html_for, id, "data-name": data_name,
            {children}
        }
    }
}
