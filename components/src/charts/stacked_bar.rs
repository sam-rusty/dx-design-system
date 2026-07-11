use dioxus::prelude::*;

use super::donut::ChartSegment;

/// Horizontal stacked bar chart with a legend.
///
/// Bar segment widths animate via CSS (`transition-all duration-500`).
/// Legend dots use `text-chart-N bg-current` so the color token only needs to be
/// defined as a text utility (same pattern as donut's `stroke: currentColor`).
#[component]
pub fn StackedBarChart(
    segments: Vec<ChartSegment>,
    /// Accessible description of the chart for screen readers.
    #[props(default)]
    aria_label: Option<String>,
) -> Element {
    rsx! {
        div { class: "space-y-3", role: "img", "aria-label": aria_label,
            div { class: "flex h-8 rounded-lg overflow-hidden",
                for (i, seg) in segments.iter().enumerate() {
                    if seg.pct > 0.1 {
                        div {
                            key: "{i}",
                            class: "{seg.color.bg_class()} {seg.color.bar_text_class()} transition-all duration-500 ease-out flex items-center justify-center text-xs font-medium",
                            style: "width: {seg.pct:.1}%",
                            if seg.pct > 8.0 { "{seg.pct:.0}%" }
                        }
                    }
                }
            }
            div { class: "flex flex-wrap gap-x-4 gap-y-1 text-xs",
                for (i, seg) in segments.iter().enumerate() {
                    div {
                        key: "{i}",
                        class: "{seg.color.text_class()} flex items-center gap-1.5",
                        span { class: "size-2.5 rounded-full bg-current inline-block shrink-0" }
                        span { class: "text-muted-foreground", "{seg.label}" }
                    }
                }
            }
        }
    }
}
