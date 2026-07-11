use dioxus::prelude::*;
use utils::format::merge;

use crate::hooks::use_unique_id;
use crate::placement::Placement;
#[cfg(target_arch = "wasm32")]
use crate::placement::{Anchor, Rect};
use crate::portal::Portal;

const PANEL_BASE: &str = "w-max max-w-xs rounded-md border border-border \
     bg-popover text-popover-foreground px-2.5 py-1.5 text-xs shadow-md \
     pointer-events-none transition-opacity duration-150";

/// Current viewport size `(width, height)` in CSS pixels; `(0.0, 0.0)` off-wasm
/// or when the window is unavailable (the panel never mounts there, so the value
/// is unused).
#[cfg(target_arch = "wasm32")]
fn viewport_size() -> (f64, f64) {
    web_sys::window()
        .map(|w| {
            (
                w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0),
                w.inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            )
        })
        .unwrap_or((0.0, 0.0))
}

#[component]
pub fn Tooltip(
    /// Content shown in the tooltip panel.
    title: Element,
    /// Preferred side; flips to the opposite side if it would overflow the viewport.
    #[props(default = Placement::Top)]
    placement: Placement,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let mut visible = use_signal(|| false);
    let mut trigger_el = use_signal(|| None::<web_sys::Element>);
    // (top, left, resolved side) once measured; None until the panel reports in.
    let mut coords = use_signal(|| None::<(f64, f64, Placement)>);
    let panel_id = use_unique_id();

    let mut hide = move || {
        visible.set(false);
        coords.set(None);
    };

    let root_class = merge(&["relative inline-flex", &class]);
    let (top, left, side) = coords().unwrap_or((0.0, 0.0, placement));
    let opacity = if visible() && coords().is_some() {
        "opacity-100"
    } else {
        "opacity-0"
    };

    rsx! {
        div {
            class: "{root_class}",
            tabindex: 0,
            "aria-describedby": panel_id,
            onmounted: move |e| {
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    trigger_el.set(Some(el.clone()));
                }
            },
            onmouseenter: move |_| visible.set(true),
            onmouseleave: move |_| hide(),
            onfocusin: move |_| visible.set(true),
            onfocusout: move |_| hide(),
            {children}
            if visible() {
                Portal {
                    class: "pointer-events-none".to_string(),
                    div {
                        id: panel_id,
                        role: "tooltip",
                        "data-side": side.data_side(),
                        style: "position: fixed; top: {top}px; left: {left}px;",
                        class: "{PANEL_BASE} {opacity}",
                        onmounted: move |_e| {
                            #[cfg(target_arch = "wasm32")]
                            if let Some(panel) = _e.downcast::<web_sys::Element>()
                                && let Some(trigger) = trigger_el.peek().clone()
                            {
                                let t = Rect::from(trigger.get_bounding_client_rect());
                                let p = Rect::from(panel.get_bounding_client_rect());
                                if t.width > 0.0 && p.width > 0.0 {
                                    let anchor = Anchor {
                                        trigger: t,
                                        panel: p,
                                        viewport: viewport_size(),
                                        gap: 8.0,
                                    };
                                    let r = anchor.resolve(placement);
                                    coords.set(Some((r.top, r.left, r.side)));
                                }
                            }
                        },
                        {title}
                    }
                }
            }
        }
    }
}
