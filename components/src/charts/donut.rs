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

    // Precompute (dash_length, dash_offset) per segment; recomputed only on data change.
    let arcs = use_memo(use_reactive!(|segments| {
        let mut acc = 0.0_f64;
        segments
            .iter()
            .map(|seg: &ChartSegment| {
                let len = c * seg.pct.clamp(0.0, 100.0) / 100.0;
                let offset = -acc;
                acc += len;
                (len, offset)
            })
            .collect::<Vec<(f64, f64)>>()
    }));
    let arcs = arcs();

    rsx! {
        svg {
            class: "size-44 shrink-0",
            view_box: "0 0 100 100",
            role: "img",
            "aria-label": aria_label,
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
