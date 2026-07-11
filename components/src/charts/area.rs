#![allow(unpredictable_function_pointer_comparisons)]

use std::borrow::Cow;

use dioxus::prelude::*;

use super::donut::SegmentColor;

#[derive(Clone, PartialEq)]
pub struct LinePoint {
    pub x: f64,
    pub y: f64,
}

/// `label` accepts both `&'static str` literals and owned `String`s via `.into()` —
/// owned variant is needed when the label is user-controlled.
#[derive(Clone, PartialEq)]
pub struct LineSeries {
    pub label: Cow<'static, str>,
    pub color: SegmentColor,
    pub points: Vec<LinePoint>,
    pub fill: bool,
}

#[derive(Clone, PartialEq)]
pub struct LineMarker {
    pub x: f64,
    pub label: Cow<'static, str>,
    pub color: SegmentColor,
}

// SVG canvas dimensions (user units).
const W: f64 = 500.0;
const H: f64 = 296.0;
const PAD_LEFT: f64 = 54.0;
const PAD_RIGHT: f64 = 16.0;
const PAD_TOP: f64 = 12.0;
const PAD_BOTTOM: f64 = 56.0;

const PLOT_W: f64 = W - PAD_LEFT - PAD_RIGHT;
const PLOT_H: f64 = H - PAD_TOP - PAD_BOTTOM;

const VIEW_BOX: &str = "0 0 500 296";

fn svg_x(data_x: f64, x_min: f64, x_max: f64) -> f64 {
    if (x_max - x_min).abs() < f64::EPSILON {
        return PAD_LEFT;
    }
    PAD_LEFT + (data_x - x_min) / (x_max - x_min) * PLOT_W
}

fn svg_y(data_y: f64, y_min: f64, y_max: f64) -> f64 {
    if (y_max - y_min).abs() < f64::EPSILON {
        return PAD_TOP + PLOT_H;
    }
    // y grows downward in SVG, so invert.
    PAD_TOP + (1.0 - (data_y - y_min) / (y_max - y_min)) * PLOT_H
}

/// Compute up to `count` nicely-rounded tick values spanning [data_min, data_max].
fn nice_ticks(data_min: f64, data_max: f64, count: usize) -> Vec<f64> {
    if (data_max - data_min).abs() < f64::EPSILON {
        return vec![data_min];
    }
    if count < 2 {
        return vec![data_min, data_max];
    }
    let range = data_max - data_min;
    let raw_step = range / (count as f64 - 1.0);
    if !raw_step.is_normal() {
        return vec![data_min, data_max];
    }
    let magnitude = 10_f64.powf(raw_step.log10().floor());
    let step = {
        let s = raw_step / magnitude;
        if s <= 1.0 {
            magnitude
        } else if s <= 2.0 {
            2.0 * magnitude
        } else if s <= 5.0 {
            5.0 * magnitude
        } else {
            10.0 * magnitude
        }
    };
    let start = (data_min / step).floor() * step;
    let mut ticks = Vec::new();
    let mut t = start;
    // Collect ticks within range, with a small epsilon to handle floating-point boundary.
    while t <= data_max + step * 0.01 {
        if t >= data_min - step * 0.01 {
            ticks.push(t);
        }
        t += step;
        if ticks.len() > count + 2 {
            break;
        }
    }
    ticks
}

/// Build an SVG `d` path string (M + L segments) from precomputed SVG coordinates.
fn polyline_d(pts: &[(f64, f64)]) -> String {
    let Some((&(fx, fy), rest)) = pts.split_first() else {
        return String::new();
    };
    let mut d = format!("M {fx:.2} {fy:.2}");
    for (x, y) in rest {
        d.push_str(&format!(" L {x:.2} {y:.2}"));
    }
    d
}

// ── Pre-rendered data structs (computed before rsx!) ──────────────────────────

#[derive(Clone, PartialEq)]
struct GridLine {
    sy: f64,
    label: String,
}

#[derive(Clone, PartialEq)]
struct XLabel {
    sx: f64,
    label: String,
}

#[derive(Clone, PartialEq)]
struct SeriesRender {
    label: Cow<'static, str>,
    color: SegmentColor,
    /// Set when the series has exactly one point; renders as a circle instead of a path.
    single_point: Option<(f64, f64)>,
    line_d: String,
    fill_d: Option<String>,
}

#[derive(Clone, PartialEq)]
struct MarkerRender {
    sx: f64,
    label: Cow<'static, str>,
    color: SegmentColor,
}

#[derive(Clone, PartialEq)]
struct LegendEntry {
    cx: f64,
    tx: f64,
    color: SegmentColor,
    label: Cow<'static, str>,
}

/// All path/label math for the chart, computed once and memoized on the inputs.
#[derive(Clone, PartialEq)]
struct AreaRender {
    plot_bottom: f64,
    plot_right: f64,
    grid_lines: Vec<GridLine>,
    x_labels: Vec<XLabel>,
    series_renders: Vec<SeriesRender>,
    marker_renders: Vec<MarkerRender>,
    legend_y: f64,
    legend_entries: Vec<LegendEntry>,
}

fn compute_area_render(
    series: &[LineSeries],
    markers: &[LineMarker],
    y_format: fn(f64) -> String,
    x_labels: Option<&Vec<String>>,
) -> AreaRender {
    let (x_min, x_max, y_min, y_max) = {
        let mut xn = f64::INFINITY;
        let mut xx = f64::NEG_INFINITY;
        let mut yn = f64::INFINITY;
        let mut yx = f64::NEG_INFINITY;
        for s in series {
            for p in &s.points {
                xn = xn.min(p.x);
                xx = xx.max(p.x);
                yn = yn.min(p.y);
                yx = yx.max(p.y);
            }
        }
        if xn == f64::INFINITY {
            (0.0, 1.0, 0.0, 1.0)
        } else {
            // Always include zero on y-axis so filled areas anchor to the baseline.
            (
                xn,
                xx.max(xn + f64::EPSILON),
                0.0_f64.min(yn),
                yx.max(yn + f64::EPSILON),
            )
        }
    };

    let plot_bottom = PAD_TOP + PLOT_H;
    let plot_right = PAD_LEFT + PLOT_W;

    let grid_lines: Vec<GridLine> = nice_ticks(y_min, y_max, 5)
        .into_iter()
        .map(|t| GridLine {
            sy: svg_y(t, y_min, y_max),
            label: y_format(t),
        })
        .collect();

    let x_labels: Vec<XLabel> = {
        let mut xs: Vec<f64> = series
            .iter()
            .flat_map(|s| s.points.iter().map(|p| p.x))
            .collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        xs.dedup();
        let step = if xs.len() <= 8 {
            1
        } else {
            xs.len().div_ceil(8)
        };
        xs.into_iter()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .map(|(i, xv)| {
                let label = x_labels
                    .and_then(|labels| labels.get(i).cloned())
                    .unwrap_or_else(|| format!("{xv:.0}"));
                XLabel {
                    sx: svg_x(xv, x_min, x_max),
                    label,
                }
            })
            .collect()
    };

    let series_renders: Vec<SeriesRender> = series
        .iter()
        .filter(|s| !s.points.is_empty())
        .map(|s| {
            let svg_pts: Vec<(f64, f64)> = s
                .points
                .iter()
                .map(|p| (svg_x(p.x, x_min, x_max), svg_y(p.y, y_min, y_max)))
                .collect();

            if svg_pts.len() == 1 {
                return SeriesRender {
                    label: s.label.clone(),
                    color: s.color,
                    single_point: Some(svg_pts[0]),
                    line_d: String::new(),
                    fill_d: None,
                };
            }

            let line_d = polyline_d(&svg_pts);
            let fill_d = if s.fill {
                let first_x = svg_pts.first().map(|(x, _)| *x).unwrap_or(PAD_LEFT);
                let last_x = svg_pts.last().map(|(x, _)| *x).unwrap_or(PAD_LEFT);
                Some(format!(
                    "{line_d} L {last_x:.2} {plot_bottom:.2} L {first_x:.2} {plot_bottom:.2} Z"
                ))
            } else {
                None
            };
            SeriesRender {
                label: s.label.clone(),
                color: s.color,
                single_point: None,
                line_d,
                fill_d,
            }
        })
        .collect();

    let marker_renders: Vec<MarkerRender> = markers
        .iter()
        .filter(|m| m.x >= x_min && m.x <= x_max)
        .map(|m| MarkerRender {
            sx: svg_x(m.x, x_min, x_max),
            label: m.label.clone(),
            color: m.color,
        })
        .collect();

    // Sit the legend well below the x-axis labels (which render at plot_bottom + 14).
    let legend_y = plot_bottom + 34.0;
    let legend_entries: Vec<LegendEntry> = {
        let mut acc = PAD_LEFT;
        series
            .iter()
            .map(|s| {
                let cx = acc + 5.0;
                let tx = acc + 14.0;
                acc += 14.0 + s.label.len() as f64 * 7.0 + 10.0;
                LegendEntry {
                    cx,
                    tx,
                    color: s.color,
                    label: s.label.clone(),
                }
            })
            .collect()
    };

    AreaRender {
        plot_bottom,
        plot_right,
        grid_lines,
        x_labels,
        series_renders,
        marker_renders,
        legend_y,
        legend_entries,
    }
}

/// SVG area+line chart supporting multiple series and optional vertical markers.
///
/// The chart is responsive (`width="100%"`) with a fixed SVG height of 296 units.
/// Series are rendered back-to-front (first in list is painted first).
#[component]
pub fn AreaLineChart(
    series: Vec<LineSeries>,
    #[props(default)] markers: Vec<LineMarker>,
    y_format: fn(f64) -> String,
    /// Optional per-x-value labels, indexed by sorted unique x position.
    /// When set, replaces the default numeric x-axis labels.
    #[props(default)]
    x_labels: Option<Vec<String>>,
    /// Accessible description of the chart for screen readers.
    #[props(default)]
    aria_label: Option<String>,
) -> Element {
    // Path/label math recomputed only when the data inputs change.
    let render = use_memo(use_reactive!(|(series, markers, x_labels)| {
        compute_area_render(&series, &markers, y_format, x_labels.as_ref())
    }));
    let AreaRender {
        plot_bottom,
        plot_right,
        grid_lines,
        x_labels,
        series_renders,
        marker_renders,
        legend_y,
        legend_entries,
    } = render();

    rsx! {
        svg {
            width: "100%",
            height: "{H}",
            view_box: VIEW_BOX,
            role: "img",
            "aria-label": aria_label,

            // ── Y-axis grid lines + labels ─────────────────────────────────
            for (i, gl) in grid_lines.iter().enumerate() {
                g { key: "grid-{i}",
                    line {
                        x1: "{PAD_LEFT:.2}",
                        y1: "{gl.sy:.2}",
                        x2: "{plot_right:.2}",
                        y2: "{gl.sy:.2}",
                        stroke: "currentColor",
                        stroke_width: "1",
                        class: "text-border",
                        stroke_opacity: "0.15",
                    }
                    text {
                        x: "{PAD_LEFT - 6.0:.2}",
                        y: "{gl.sy + 4.0:.2}",
                        text_anchor: "end",
                        class: "fill-muted-foreground",
                        font_size: "11",
                        "{gl.label}"
                    }
                }
            }

            // ── X-axis baseline ────────────────────────────────────────────
            line {
                x1: "{PAD_LEFT:.2}",
                y1: "{plot_bottom:.2}",
                x2: "{plot_right:.2}",
                y2: "{plot_bottom:.2}",
                stroke: "currentColor",
                class: "text-border",
                stroke_opacity: "0.3",
                stroke_width: "1",
            }

            // ── X-axis labels ──────────────────────────────────────────────
            for xl in x_labels.iter() {
                text {
                    key: "xlabel-{xl.label}",
                    x: "{xl.sx:.2}",
                    y: "{plot_bottom + 14.0:.2}",
                    text_anchor: "middle",
                    class: "fill-muted-foreground",
                    font_size: "11",
                    "{xl.label}"
                }
            }

            // ── Series: fill then stroke ───────────────────────────────────
            for (i, sr) in series_renders.iter().enumerate() {
                g { key: "series-{i}",
                    if let Some((cx, cy)) = sr.single_point {
                        circle {
                            key: "dot-{i}",
                            cx: "{cx:.2}",
                            cy: "{cy:.2}",
                            r: "3",
                            class: "{sr.color.text_class()}",
                            fill: "currentColor",
                        }
                    } else {
                        if let Some(fd) = &sr.fill_d {
                            path {
                                key: "fill-{i}",
                                d: "{fd}",
                                class: "{sr.color.text_class()}",
                                fill: "currentColor",
                                fill_opacity: "0.2",
                                stroke: "none",
                            }
                        }
                        path {
                            d: "{sr.line_d}",
                            class: "{sr.color.text_class()}",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linejoin: "round",
                            stroke_linecap: "round",
                        }
                    }
                }
            }

            // ── Vertical markers ───────────────────────────────────────────
            for mr in marker_renders.iter() {
                g { key: "marker-{mr.label}",
                    line {
                        x1: "{mr.sx:.2}",
                        y1: "{PAD_TOP:.2}",
                        x2: "{mr.sx:.2}",
                        y2: "{plot_bottom:.2}",
                        class: "{mr.color.text_class()}",
                        stroke: "currentColor",
                        stroke_width: "1.5",
                        stroke_dasharray: "4 3",
                    }
                    text {
                        x: "{mr.sx + 3.0:.2}",
                        y: "{PAD_TOP + 10.0:.2}",
                        class: "{mr.color.text_class()}",
                        fill: "currentColor",
                        font_size: "10",
                        "{mr.label}"
                    }
                }
            }

            // ── Legend ─────────────────────────────────────────────────────
            for (i, le) in legend_entries.iter().enumerate() {
                g { key: "legend-{i}",
                    circle {
                        cx: "{le.cx:.2}",
                        cy: "{legend_y - 4.0:.2}",
                        r: "4",
                        class: "{le.color.text_class()}",
                        fill: "currentColor",
                    }
                    text {
                        x: "{le.tx:.2}",
                        y: "{legend_y:.2}",
                        class: "fill-muted-foreground",
                        font_size: "11",
                        "{le.label}"
                    }
                }
            }
        }
    }
}
