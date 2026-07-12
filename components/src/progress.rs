use dioxus::prelude::*;
use ds_utils::format::merge;

/// Determinate progress bar. `value` is clamped to `0.0..=1.0`.
#[component]
pub fn Progress(value: f32, #[props(default)] class: String) -> Element {
    let pct = (value.clamp(0.0, 1.0) * 100.0).round() as i32;
    let track = merge(&[
        "w-full h-2 rounded-full bg-secondary overflow-hidden",
        &class,
    ]);
    rsx! {
        div {
            class: "{track}",
            role: "progressbar",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{pct}",
            div {
                class: "h-full rounded-full bg-primary transition-[width] duration-200",
                style: "width: {pct}%",
            }
        }
    }
}
