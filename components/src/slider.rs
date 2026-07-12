use std::rc::Rc;

use dioxus::prelude::*;
use ds_utils::format::merge;

/// Map a horizontal client-x onto the slider range. Pure so the mapping is
/// testable: clamps into `[min, max]` and snaps to `step` increments.
fn slider_value(x: f64, left: f64, width: f64, min: f32, max: f32, step: f32) -> f32 {
    if width <= 0.0 || step <= 0.0 || max <= min {
        return min;
    }
    let ratio = ((x - left) / width).clamp(0.0, 1.0) as f32;
    let raw = min + ratio * (max - min);
    ((raw / step).round() * step).clamp(min, max)
}

/// Labelled horizontal slider: label + value readout on top, scrubbable track
/// below. Touch-driven; the track rect is re-measured on every touch-start so
/// scroll position never stales the mapping.
#[component]
pub fn Slider(
    label: String,
    value: f32,
    min: f32,
    max: f32,
    #[props(default = 1.0)] step: f32,
    #[props(default)] unit: Option<String>,
    on_change: EventHandler<f32>,
) -> Element {
    let mut track = use_signal(|| None::<Rc<MountedData>>);
    let mut rect = use_signal(|| None::<(f64, f64)>); // (left, width)
    let mut dragging = use_signal(|| false);

    let pct = if max > min {
        ((value - min) / (max - min) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let mut begin = move |x: f64| {
        dragging.set(true);
        let Some(md) = track.peek().clone() else {
            return;
        };
        spawn(async move {
            if let Ok(r) = md.get_client_rect().await {
                rect.set(Some((r.origin.x, r.size.width)));
                on_change.call(slider_value(x, r.origin.x, r.size.width, min, max, step));
            }
        });
    };

    let readout = if step.fract() == 0.0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    };

    let thumb_class = merge(&[
        "absolute top-1/2 -translate-x-1/2 -translate-y-1/2 size-7 rounded-full \
         bg-white shadow-float transition-transform",
        if dragging() { "scale-115" } else { "" },
    ]);

    rsx! {
        div { class: "flex flex-col gap-2 py-3",
            div { class: "flex items-baseline justify-between",
                span { class: "text-[15px] font-semibold", "{label}" }
                span { class: "text-sm font-bold text-primary tabular-nums",
                    "{readout}"
                    if let Some(u) = unit.as_deref() {
                        span { class: "ml-0.5 font-medium text-muted-foreground", "{u}" }
                    }
                }
            }
            div {
                class: "h-9 flex items-center touch-none",
                ontouchstart: move |e| {
                    if let Some(t) = e.touches().first() {
                        begin(t.client_coordinates().x);
                    }
                },
                ontouchmove: move |e| {
                    if let (Some((left, width)), Some(t)) = (rect(), e.touches().first()) {
                        on_change
                            .call(
                                slider_value(t.client_coordinates().x, left, width, min, max, step),
                            );
                    }
                },
                ontouchend: move |_| dragging.set(false),
                ontouchcancel: move |_| dragging.set(false),
                div {
                    onmounted: move |e| track.set(Some(e.data())),
                    class: "relative flex-1 h-[5px] rounded-full bg-muted-foreground/25",
                    div {
                        class: "absolute inset-y-0 left-0 rounded-full bg-primary",
                        style: "width: {pct}%;",
                    }
                    div { class: thumb_class, style: "left: {pct}%;" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_x_across_track_with_clamp_and_step() {
        assert_eq!(slider_value(150.0, 100.0, 100.0, 0.0, 100.0, 1.0), 50.0);
        assert_eq!(slider_value(50.0, 100.0, 100.0, 6.0, 120.0, 1.0), 6.0);
        assert_eq!(slider_value(999.0, 100.0, 100.0, 6.0, 120.0, 1.0), 120.0);
        assert_eq!(slider_value(133.0, 100.0, 100.0, 0.0, 10.0, 1.0), 3.0);
    }

    #[test]
    fn degenerate_track_returns_min() {
        assert_eq!(slider_value(10.0, 0.0, 0.0, 5.0, 50.0, 1.0), 5.0);
        assert_eq!(slider_value(10.0, 0.0, 100.0, 5.0, 50.0, 0.0), 5.0);
    }
}
