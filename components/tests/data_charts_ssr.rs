//! SSR pins for the charts family: accessible names default from the data and
//! bad input never reaches the SVG attributes.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "charts")]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;

use components::{
    AreaLineChart, ChartSegment, DonutChart, LinePoint, LineSeries, SegmentColor, StackedBarChart,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn segments() -> Vec<ChartSegment> {
    vec![
        ChartSegment {
            label: "Stocks".into(),
            pct: 60.0,
            color: SegmentColor::Primary,
        },
        ChartSegment {
            label: "Bonds".into(),
            pct: f64::NAN,
            color: SegmentColor::Accent,
        },
        ChartSegment {
            label: "Cash".into(),
            pct: 70.0,
            color: SegmentColor::Success,
        },
    ]
}

#[test]
fn donut_defaults_label_and_never_emits_nan() {
    fn app() -> Element {
        rsx! {
            DonutChart { segments: segments(), center_label: "Total", center_pct: f64::NAN }
        }
    }
    let html = render(app);
    assert!(
        html.contains(r#"aria-label="Donut chart: Stocks 60%, Bonds 0%, Cash 70%""#),
        "{html}"
    );
    assert!(html.contains("<title>Donut chart: Stocks 60%"), "{html}");
    assert!(!html.contains("NaN"), "{html}");
    assert!(html.contains(">0.0%<"), "{html}");
}

#[test]
fn stacked_bar_caps_widths_and_keeps_legend_outside_image() {
    fn app() -> Element {
        rsx! {
            StackedBarChart { segments: segments() }
        }
    }
    let html = render(app);
    // 60 + (NaN→0) + 70 → third segment capped at the remaining 40.
    assert!(html.contains("width: 60.0%"), "{html}");
    assert!(html.contains("width: 40.0%"), "{html}");
    assert!(!html.contains("width: 70.0%"), "{html}");
    assert!(!html.contains("NaN"), "{html}");
    // role="img" is on the track, the legend is a sibling.
    let img = html.find(r#"role="img""#).expect("role img");
    let legend = html.find("Stocks</span>").expect("legend");
    assert!(img < legend, "{html}");
    assert!(
        html.contains(r#"aria-label="Stacked bar: Stocks 60%"#),
        "{html}"
    );
}

#[test]
fn area_chart_flat_zero_has_distinct_ticks_and_default_label() {
    fn fmt(v: f64) -> String {
        format!("{v:.1}")
    }
    fn app() -> Element {
        let series = vec![LineSeries {
            label: "Sales".into(),
            color: SegmentColor::Primary,
            points: (0..3)
                .map(|i| LinePoint {
                    x: i as f64,
                    y: 0.0,
                })
                .collect(),
            fill: true,
        }];
        rsx! {
            AreaLineChart { series, y_format: fmt }
        }
    }
    let html = render(app);
    assert!(
        html.contains(r#"aria-label="Line chart: Sales (3 points)""#),
        "{html}"
    );
    assert!(
        html.contains("<title>Line chart: Sales (3 points)</title>"),
        "{html}"
    );
    assert!(html.contains(">1.0<"), "{html}");
    assert_eq!(html.matches(">0.0<").count(), 1, "{html}");
    assert!(!html.contains("NaN"), "{html}");
}
