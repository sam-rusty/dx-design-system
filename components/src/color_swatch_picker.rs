use dioxus::prelude::*;
use utils::format::merge;

/// One selectable color swatch. `value` is an opaque identifier the caller maps
/// back to its own color type; `class` is the Tailwind background class to paint.
#[derive(Clone, PartialEq)]
pub struct ColorSwatchOption {
    pub value: String,
    pub class: String,
    pub label: String,
}

/// A row of round color swatches. The first option is selected by default: on
/// mount, when `value` is `None`, the component emits `on_select(first.value)`.
/// A selection therefore always exists — clicking a swatch selects it; there is
/// no click-to-deselect.
#[component]
pub fn ColorSwatchPicker(
    options: Vec<ColorSwatchOption>,
    value: ReadSignal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    let first = options.first().map(|o| o.value.clone());
    use_effect(move || {
        if value().is_none()
            && let Some(v) = first.clone()
        {
            on_select.call(v);
        }
    });

    rsx! {
        div { class: "flex flex-wrap gap-1.5",
            for opt in options.iter().cloned() {
                {
                    let selected = value().as_deref() == Some(opt.value.as_str());
                    let base = "size-5 rounded-full cursor-pointer transition-all hover:scale-110";
                    let class = if selected {
                        merge(&[base, "scale-110 ring-2 ring-offset-1 ring-foreground/40", &opt.class])
                    } else {
                        merge(&[base, &opt.class])
                    };
                    let v = opt.value.clone();
                    rsx! {
                        button {
                            r#type: "button",
                            title: "{opt.label}",
                            class: "{class}",
                            onclick: move |_| on_select.call(v.clone()),
                        }
                    }
                }
            }
        }
    }
}
