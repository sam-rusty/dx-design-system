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

/// Keyboard step for the `role="slider"` thumb (WAI-ARIA slider pattern):
/// Arrow keys move one `step`, PageUp/PageDown ten, Home/End jump to the
/// bounds. `None` for keys the slider does not handle.
fn slider_key_value(key: &Key, value: f32, min: f32, max: f32, step: f32) -> Option<f32> {
    if step <= 0.0 || max <= min {
        return None;
    }
    let target = match key {
        Key::ArrowRight | Key::ArrowUp => value + step,
        Key::ArrowLeft | Key::ArrowDown => value - step,
        Key::PageUp => value + step * 10.0,
        Key::PageDown => value - step * 10.0,
        // Bounds are reachable even when they are not step multiples.
        Key::Home => return Some(min),
        Key::End => return Some(max),
        _ => return None,
    };
    Some(((target / step).round() * step).clamp(min, max))
}

/// Labelled horizontal slider: label + value readout on top, scrubbable track
/// below. Pointer-driven (mouse, pen and touch share one path; the track keeps
/// `touch-none` so a drag never scrolls the page) and keyboard-operable via the
/// focusable `role="slider"` thumb. The track rect is re-measured on every
/// pointer-down so scroll position never stales the mapping.
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

    let aria_valuetext = match unit.as_deref() {
        Some(u) => format!("{readout} {u}"),
        None => readout.clone(),
    };

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
                onpointerdown: move |e: PointerEvent| {
                    // Capture the pointer on the track container so a drag that
                    // leaves the element keeps delivering move/up events.
                    #[cfg(feature = "web")]
                    {
                        use dioxus::web::WebEventExt;
                        use wasm_bindgen::JsCast;
                        let native = e.as_web_event();
                        if let Some(el) = native
                            .current_target()
                            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        {
                            let _ = el.set_pointer_capture(e.pointer_id());
                        }
                    }
                    begin(e.client_coordinates().x);
                },
                onpointermove: move |e: PointerEvent| {
                    if !*dragging.peek() {
                        return;
                    }
                    if let Some((left, width)) = *rect.peek() {
                        on_change
                            .call(slider_value(e.client_coordinates().x, left, width, min, max, step));
                    }
                },
                onpointerup: move |_| dragging.set(false),
                onpointercancel: move |_| dragging.set(false),
                div {
                    onmounted: move |e| track.set(Some(e.data())),
                    class: "relative flex-1 h-[5px] rounded-full bg-muted-foreground/25",
                    div {
                        class: "absolute inset-y-0 left-0 rounded-full bg-primary",
                        style: "width: {pct}%;",
                    }
                    div {
                        class: thumb_class,
                        style: "left: {pct}%;",
                        role: "slider",
                        tabindex: "0",
                        "aria-label": "{label}",
                        "aria-valuemin": "{min}",
                        "aria-valuemax": "{max}",
                        "aria-valuenow": "{value}",
                        "aria-valuetext": "{aria_valuetext}",
                        "aria-orientation": "horizontal",
                        onkeydown: move |e: KeyboardEvent| {
                            if let Some(next) = slider_key_value(&e.key(), value, min, max, step) {
                                e.prevent_default();
                                if next != value {
                                    on_change.call(next);
                                }
                            }
                        },
                    }
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

    #[test]
    fn arrow_keys_step_and_clamp() {
        assert_eq!(
            slider_key_value(&Key::ArrowRight, 5.0, 0.0, 10.0, 1.0),
            Some(6.0)
        );
        assert_eq!(
            slider_key_value(&Key::ArrowUp, 5.0, 0.0, 10.0, 1.0),
            Some(6.0)
        );
        assert_eq!(
            slider_key_value(&Key::ArrowLeft, 5.0, 0.0, 10.0, 1.0),
            Some(4.0)
        );
        assert_eq!(
            slider_key_value(&Key::ArrowDown, 0.0, 0.0, 10.0, 1.0),
            Some(0.0)
        );
        assert_eq!(
            slider_key_value(&Key::ArrowRight, 10.0, 0.0, 10.0, 1.0),
            Some(10.0)
        );
        assert_eq!(
            slider_key_value(&Key::ArrowRight, 1.0, 0.0, 10.0, 0.5),
            Some(1.5)
        );
    }

    #[test]
    fn home_end_and_page_keys() {
        assert_eq!(slider_key_value(&Key::Home, 5.0, 1.0, 10.0, 1.0), Some(1.0));
        assert_eq!(slider_key_value(&Key::End, 5.0, 1.0, 10.0, 1.0), Some(10.0));
        assert_eq!(
            slider_key_value(&Key::PageUp, 5.0, 0.0, 100.0, 1.0),
            Some(15.0)
        );
        assert_eq!(
            slider_key_value(&Key::PageDown, 5.0, 0.0, 100.0, 1.0),
            Some(0.0)
        );
    }

    #[test]
    fn unhandled_keys_and_degenerate_ranges_yield_none() {
        assert_eq!(slider_key_value(&Key::Enter, 5.0, 0.0, 10.0, 1.0), None);
        assert_eq!(
            slider_key_value(&Key::ArrowRight, 5.0, 0.0, 10.0, 0.0),
            None
        );
        assert_eq!(
            slider_key_value(&Key::ArrowRight, 5.0, 10.0, 0.0, 1.0),
            None
        );
    }
}
