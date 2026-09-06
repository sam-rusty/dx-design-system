use std::borrow::Cow;

use dioxus::prelude::*;

/// A single arc segment for a donut or stacked-bar chart.
///
/// `label` accepts both `&'static str` literals and owned `String`s via `.into()` —
/// owned variant is needed when the label is user-controlled (e.g. an investment name).
#[derive(Clone, PartialEq)]
pub struct ChartSegment {
    pub label: Cow<'static, str>,
    pub pct: f64,
    pub color: SegmentColor,
}

/// Semantic color slot for a chart segment.
#[derive(Clone, Copy, PartialEq)]
pub enum SegmentColor {
    Primary,
    Secondary,
    Accent,
    Destructive,
    Success,
    Warning,
}

impl SegmentColor {
    /// Tailwind text-color class (used for SVG `stroke: currentColor` and legend dots).
    pub fn text_class(self) -> &'static str {
        match self {
            SegmentColor::Primary => "text-primary",
            SegmentColor::Secondary => "text-secondary-foreground",
            SegmentColor::Accent => "text-accent-foreground",
            SegmentColor::Destructive => "text-destructive",
            SegmentColor::Success => "text-success",
            SegmentColor::Warning => "text-warning",
        }
    }

    /// Tailwind bg class (used for bar segment backgrounds).
    pub fn bg_class(self) -> &'static str {
        match self {
            SegmentColor::Primary => "bg-primary",
            SegmentColor::Secondary => "bg-secondary",
            SegmentColor::Accent => "bg-accent",
            SegmentColor::Destructive => "bg-destructive",
            SegmentColor::Success => "bg-success",
            SegmentColor::Warning => "bg-warning",
        }
    }

    /// Foreground text color for content rendered inside a filled bar segment.
    pub fn bar_text_class(self) -> &'static str {
        match self {
            SegmentColor::Primary => "text-primary-foreground",
            SegmentColor::Secondary => "text-secondary-foreground",
            SegmentColor::Accent => "text-accent-foreground",
            SegmentColor::Destructive => "text-destructive-foreground",
            SegmentColor::Success => "text-success-foreground",
            SegmentColor::Warning => "text-warning-foreground",
        }
    }
}

/// Percentage sanitised for rendering: non-finite → 0, otherwise clamped to `0..=100`.
pub(super) fn safe_pct(pct: f64) -> f64 {
    if pct.is_finite() {
        pct.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Default accessible label built from the data: `"<kind>: A 40%, B 60%"`.
pub(super) fn segments_label(kind: &str, segments: &[ChartSegment]) -> String {
    let parts: Vec<String> = segments
        .iter()
        .map(|s| format!("{} {:.0}%", s.label, safe_pct(s.pct)))
        .collect();
    format!("{kind}: {}", parts.join(", "))
}

/// `(dash_length, dash_offset)` per segment for a circle of circumference `c`.
///
/// Non-finite / negative percentages render as empty arcs and the running total
/// is capped at 100% so later segments never overlap earlier ones.
fn donut_arcs(segments: &[ChartSegment], c: f64) -> Vec<(f64, f64)> {
    let mut acc_pct = 0.0_f64;
    segments
        .iter()
        .map(|seg| {
            let pct = safe_pct(seg.pct).min(100.0 - acc_pct);
            let len = c * pct / 100.0;
            let offset = if acc_pct > 0.0 {
                -(c * acc_pct / 100.0)
            } else {
                0.0
            };
            acc_pct += pct;
            (len, offset)
        })
        .collect()
}

/// Multi-segment donut chart.
///
/// Arc transitions use CSS (`transition-[stroke-dasharray,stroke-dashoffset] duration-500`).
/// Animate `center_pct` in the caller if a count-up effect is desired.
#[component]
pub fn DonutChart(
    segments: Vec<ChartSegment>,
    center_label: &'static str,
    center_pct: f64,
    /// Accessible description of the chart for screen readers.
    #[props(default)]
    aria_label: Option<String>,
) -> Element {
    let r = 40.0_f64;
    let c = 2.0 * std::f64::consts::PI * r;

    // A handful of multiplications: cheaper than memoising (which clones + compares
    // the segment list every render).
    let arcs = donut_arcs(&segments, c);
    let label = aria_label.unwrap_or_else(|| segments_label("Donut chart", &segments));
    let center_pct = if center_pct.is_finite() {
        center_pct
    } else {
        0.0
    };

    rsx! {
        svg {
            class: "size-44 shrink-0",
            view_box: "0 0 100 100",
            role: "img",
            "aria-label": "{label}",
            title { "{label}" }
            circle {
                cx: "50",
                cy: "50",
                r: "{r}",
                fill: "none",
                class: "text-muted-foreground stroke-current",
                stroke_width: "14",
                stroke: "currentColor",
                opacity: "0.15",
            }
            g { transform: "rotate(-90 50 50)",
                for (i, (seg, (len, offset))) in segments.iter().zip(arcs.iter()).enumerate() {
                    circle {
                        key: "{i}",
                        cx: "50",
                        cy: "50",
                        r: "{r}",
                        fill: "none",
                        class: "{seg.color.text_class()} stroke-current transition-[stroke-dasharray,stroke-dashoffset] duration-500 ease-out",
                        stroke_width: "14",
                        stroke: "currentColor",
                        stroke_dasharray: "{len} {c}",
                        stroke_dashoffset: "{offset}",
                    }
                }
            }
            text {
                x: "50",
                y: "45",
                text_anchor: "middle",
                font_size: "7",
                class: "fill-muted-foreground font-medium",
                "{center_label}"
            }
            text {
                x: "50",
                y: "60",
                text_anchor: "middle",
                font_size: "14",
                class: "fill-foreground font-bold tabular-nums",
                "{center_pct:.1}%"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(pct: f64) -> ChartSegment {
        ChartSegment {
            label: "x".into(),
            pct,
            color: SegmentColor::Primary,
        }
    }

    #[test]
    fn donut_arcs_single_full_segment_covers_circumference() {
        let c = 100.0;
        let arcs = donut_arcs(&[seg(100.0)], c);
        assert_eq!(arcs, vec![(100.0, 0.0)]);
    }

    #[test]
    fn donut_arcs_offsets_accumulate() {
        let arcs = donut_arcs(&[seg(25.0), seg(50.0)], 100.0);
        assert_eq!(arcs, vec![(25.0, 0.0), (50.0, -25.0)]);
    }

    #[test]
    fn donut_arcs_caps_sum_at_100() {
        // E11: segments summing past 100% must not overlap.
        let arcs = donut_arcs(&[seg(80.0), seg(50.0)], 100.0);
        assert_eq!(arcs, vec![(80.0, 0.0), (20.0, -80.0)]);
        let arcs = donut_arcs(&[seg(150.0)], 100.0);
        assert_eq!(arcs, vec![(100.0, 0.0)]);
    }

    #[test]
    fn donut_arcs_non_finite_and_negative_become_zero() {
        // E11: NaN would otherwise leak into `stroke-dasharray`.
        let arcs = donut_arcs(&[seg(f64::NAN), seg(-5.0), seg(f64::INFINITY)], 100.0);
        assert_eq!(arcs, vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)]);
        assert!(arcs.iter().all(|(l, o)| l.is_finite() && o.is_finite()));
    }

    #[test]
    fn safe_pct_clamps() {
        assert_eq!(safe_pct(f64::NAN), 0.0);
        assert_eq!(safe_pct(-1.0), 0.0);
        assert_eq!(safe_pct(120.0), 100.0);
        assert_eq!(safe_pct(42.5), 42.5);
    }

    #[test]
    fn default_label_lists_segments() {
        let segs = [
            ChartSegment {
                label: "A".into(),
                pct: 40.0,
                color: SegmentColor::Primary,
            },
            ChartSegment {
                label: "B".into(),
                pct: 60.0,
                color: SegmentColor::Accent,
            },
        ];
        assert_eq!(segments_label("Donut", &segs), "Donut: A 40%, B 60%");
    }
}
