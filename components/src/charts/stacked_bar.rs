use dioxus::prelude::*;

use super::donut::{ChartSegment, safe_pct, segments_label};

/// Bar widths in percent, sanitised (E11): non-finite → 0, each clamped to `0..=100`,
/// and the running total capped at 100% so the track never overflows.
fn bar_widths(segments: &[ChartSegment]) -> Vec<f64> {
    let mut acc = 0.0_f64;
    segments
        .iter()
        .map(|s| {
            let w = safe_pct(s.pct).min(100.0 - acc);
            acc += w;
            w
        })
        .collect()
}

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
    let widths = bar_widths(&segments);
    let label = aria_label.unwrap_or_else(|| segments_label("Stacked bar", &segments));
    rsx! {
        div { class: "space-y-3",
            // `role="img"` sits on the bar track only so the legend text below stays
            // readable by assistive tech instead of being flattened into the image.
            div {
                class: "flex h-8 rounded-lg overflow-hidden",
                role: "img",
                "aria-label": "{label}",
                for (i, (seg, pct)) in segments.iter().zip(widths.iter().copied()).enumerate() {
                    if pct > 0.1 {
                        div {
                            key: "{i}",
                            class: "{seg.color.bg_class()} {seg.color.bar_text_class()} transition-all duration-500 ease-out flex items-center justify-center text-xs font-medium",
                            style: "width: {pct:.1}%",
                            if pct > 8.0 { "{pct:.0}%" }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::SegmentColor;

    fn seg(pct: f64) -> ChartSegment {
        ChartSegment {
            label: "x".into(),
            pct,
            color: SegmentColor::Primary,
        }
    }

    #[test]
    fn bar_widths_cap_total_and_drop_nan() {
        assert_eq!(bar_widths(&[seg(70.0), seg(50.0)]), vec![70.0, 30.0]);
        assert_eq!(bar_widths(&[seg(f64::NAN), seg(20.0)]), vec![0.0, 20.0]);
        assert_eq!(bar_widths(&[seg(120.0), seg(5.0)]), vec![100.0, 0.0]);
    }
}
