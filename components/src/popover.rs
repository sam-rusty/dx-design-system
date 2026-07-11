use dioxus::prelude::*;

use crate::button::{Button, ButtonVariant};
use crate::hooks::{
    use_controlled, use_dismiss_on_viewport_change, use_escape_listener, use_outside_dismiss_panel,
    use_unique_id,
};
use crate::placement::Placement;
#[cfg(target_arch = "wasm32")]
use crate::placement::{Anchor, Rect};
use crate::portal::Portal;

/// Current viewport size `(width, height)` in CSS pixels; `(0.0, 0.0)` off-wasm
/// (the panel only measures on the client, so the value is unused there).
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

/// A generic popover panel anchored to a trigger element.
///
/// Open state is controlled-or-uncontrolled via [`use_controlled`]: pass `open`
/// + `on_open_change` to control it, or leave them unset and let the popover own
/// its state (seeded by `default_open`). `toggle_on_click` controls whether the
/// trigger wrapper flips the state on click — disable it when the trigger is
/// itself interactive (a menu) and the popover is opened from elsewhere.
#[component]
pub fn Popover(
    trigger: Element,
    children: Element,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default = true)] toggle_on_click: bool,
    #[props(default)] placement: Placement,
    #[props(default)] class: Option<String>,
) -> Element {
    let (is_open, set_open) = use_controlled(open, default_open, on_open_change);
    let mut root_el = use_signal(|| None::<web_sys::Element>);
    let mut trigger_el = use_signal(|| None::<web_sys::Element>);
    // Mutated only in the wasm-gated `onmounted` below; `mut` is unused off-wasm.
    #[allow(unused_mut)]
    let mut panel_el = use_signal(|| None::<web_sys::Element>);
    // Fixed-position coords + resolved side, once the panel reports its size; None
    // re-measures on each open so a moved trigger never reuses stale coordinates.
    let mut coords = use_signal(|| None::<(f64, f64, Placement)>);
    let panel_id = use_unique_id();

    use_escape_listener(move || {
        if is_open() {
            set_open.call(false);
        }
    });
    use_outside_dismiss_panel(root_el, panel_el, is_open.into(), move || {
        if is_open() {
            set_open.call(false);
        }
    });
    // Fixed coords are measured once on open; a scroll/resize would detach the
    // panel from its trigger, so close instead of chasing the trigger per-frame.
    use_dismiss_on_viewport_change(is_open.into(), move || {
        if is_open() {
            set_open.call(false);
        }
    });

    // Reset to None on close so a reopen re-measures from scratch: without this the
    // panel would paint one frame at the previous (now stale) coords before the
    // remount's onmounted re-anchors it — a visible jump when the trigger has moved.
    use_effect(move || {
        if !is_open() {
            coords.set(None);
        }
    });

    let open_now = is_open();
    let extra = class.unwrap_or_default();
    let resolved_side = if placement == Placement::Auto {
        Placement::Bottom
    } else {
        placement
    };
    let (top, left, side) = coords().unwrap_or((0.0, 0.0, resolved_side));
    let origin = side.transform_origin();
    // Hidden until measured so the unpositioned first paint at (0,0) never flashes.
    let visibility = if open_now && coords().is_some() {
        "opacity-100 scale-100 pointer-events-auto"
    } else {
        "opacity-0 scale-95 pointer-events-none"
    };
    let toggle = move |_| {
        if toggle_on_click {
            set_open.call(!is_open());
        }
    };

    rsx! {
        div {
            class: "relative inline-block",
            onmounted: move |e| {
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    root_el.set(Some(el.clone()));
                }
            },
            div {
                class: if toggle_on_click { "cursor-pointer" } else { "" },
                role: toggle_on_click.then_some("button"),
                tabindex: toggle_on_click.then_some("0"),
                "aria-haspopup": toggle_on_click.then_some("dialog"),
                "aria-expanded": toggle_on_click.then(|| open_now.to_string()),
                "aria-controls": if toggle_on_click { Some(panel_id()) } else { None },
                onmounted: move |e| {
                    if let Some(el) = e.downcast::<web_sys::Element>() {
                        trigger_el.set(Some(el.clone()));
                    }
                },
                onclick: toggle,
                onkeydown: move |e| {
                    if !toggle_on_click {
                        return;
                    }
                    let key = e.key();
                    if key == Key::Enter || key == Key::Character(" ".to_string()) {
                        e.prevent_default();
                        set_open.call(!is_open());
                    }
                },
                {trigger}
            }
            if open_now {
                Portal {
                    class: "pointer-events-none".to_string(),
                    div {
                        id: panel_id,
                        role: "dialog",
                        "data-side": side.data_side(),
                        style: "position: fixed; top: {top}px; left: {left}px;",
                        class: "z-[9999] rounded-xl bg-popover text-popover-foreground p-3 border border-border shadow-lg transition-all duration-150 {origin} {visibility} {extra}",
                        onmounted: move |_e| {
                            #[cfg(target_arch = "wasm32")]
                            if let Some(panel) = _e.downcast::<web_sys::Element>() {
                                panel_el.set(Some(panel.clone()));
                                if let Some(trigger) = trigger_el.peek().clone() {
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
                            }
                        },
                        {children}
                    }
                }
            }
        }
    }
}

/// Pre-built destructive confirmation popover. Opened externally via `open`
/// (e.g. a menu item sets the bound signal), then confirmed or cancelled from
/// the panel — so the trigger does not toggle on click.
#[component]
pub fn PopoverConfirm(
    children: Element,
    message: String,
    #[props(default)] confirm_label: Option<String>,
    on_confirm: EventHandler<()>,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] placement: Placement,
) -> Element {
    let confirm_label = confirm_label.unwrap_or_else(|| "Confirm".into());
    let (_is_open, set_open) = use_controlled(open, default_open, on_open_change);

    rsx! {
        Popover {
            open: Some(_is_open()),
            on_open_change: move |v| set_open.call(v),
            toggle_on_click: false,
            placement,
            trigger: children,
            div { class: "flex flex-col gap-2 min-w-48 max-w-64",
                p { class: "text-sm text-foreground", "{message}" }
                div { class: "flex items-center justify-end gap-1.5",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: crate::ButtonSize::Sm,
                        onclick: move |_| set_open.call(false),
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: crate::ButtonSize::Sm,
                        onclick: move |_| {
                            on_confirm.call(());
                            set_open.call(false);
                        },
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}
