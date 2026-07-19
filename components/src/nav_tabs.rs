use crate::{Icon, IconName, Link};
use dioxus::prelude::*;
use ds_utils::format::merge;
#[cfg(feature = "web")]
use wasm_bindgen::JsCast;

#[cfg(feature = "web")]
use super::nav_sliding_indicator::sliding_indicator_style;
use super::nav_sliding_indicator::{SlidingIndicatorAxis, sliding_indicator_class};
use crate::hooks::{use_escape_listener, use_outside_dismiss};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavTabsDirection {
    Horizontal,
    Vertical,
}

impl NavTabsDirection {
    fn base_class(self) -> &'static str {
        match self {
            Self::Horizontal => {
                "group relative flex h-10 items-center justify-center px-4 cursor-pointer whitespace-nowrap"
            }
            Self::Vertical => "group relative flex w-full items-center py-2 pl-4 cursor-pointer",
        }
    }

    fn tab_class(self, active: bool) -> &'static str {
        match (self, active) {
            (Self::Horizontal, true) => "text-primary font-bold",
            (Self::Horizontal, false) => "text-muted-foreground font-medium hover:text-foreground",
            (Self::Vertical, true) => {
                if cfg!(feature = "web") {
                    "text-primary font-bold"
                } else {
                    "text-primary font-bold -ml-px pl-[calc(1rem-1px)]"
                }
            }
            (Self::Vertical, false) => "text-muted-foreground font-medium hover:text-foreground",
        }
    }

    fn container_class(self) -> &'static str {
        match self {
            Self::Horizontal => "relative flex items-center mt-1",
            Self::Vertical => "relative flex flex-col space-y-1 w-full border-l-2 border-border/50",
        }
    }

    fn indicator_class(self) -> &'static str {
        sliding_indicator_class(match self {
            Self::Horizontal => SlidingIndicatorAxis::Horizontal,
            Self::Vertical => SlidingIndicatorAxis::Vertical,
        })
    }

    fn child_panel_class(self) -> &'static str {
        match self {
            Self::Horizontal => "absolute top-full left-1/2 -translate-x-1/2 pt-2 z-50",
            Self::Vertical => "absolute left-full top-0 pl-2 z-50",
        }
    }

    fn child_item_class(self, active: bool) -> &'static str {
        match (self, active) {
            (Self::Horizontal, true) => {
                "flex items-center w-full px-4 py-2 text-sm font-semibold text-foreground bg-secondary cursor-pointer"
            }
            (Self::Horizontal, false) => {
                "flex items-center w-full px-4 py-2 text-sm text-foreground hover:bg-secondary transition-colors font-normal cursor-pointer"
            }
            (Self::Vertical, true) => {
                "flex w-full items-center py-2 pl-6 text-sm font-bold text-foreground bg-secondary cursor-pointer"
            }
            (Self::Vertical, false) => {
                "flex w-full items-center py-2 pl-6 text-sm font-medium text-foreground hover:bg-secondary transition-colors cursor-pointer"
            }
        }
    }

    fn child_chevron_class(self) -> &'static str {
        match self {
            Self::Horizontal => {
                "size-3 transition-transform duration-200 group-hover/tab:rotate-180"
            }
            Self::Vertical => "size-3 transition-transform duration-200 group-hover/tab:-rotate-90",
        }
    }
}

fn is_path_active(current: &str, path: &str) -> bool {
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        current == "/"
    } else {
        current
            .strip_prefix(path)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    }
}

fn routable_path_only<R: Routable>(route: &R) -> String {
    let s = route.to_string();
    match s.split_once('?') {
        Some((p, _)) => p.to_string(),
        None => s,
    }
}

/// A single entry in a `NavTabs` list. `R` is the app or feature [`Routable`] (e.g. root `Route`).
#[derive(Clone, PartialEq)]
pub enum NavItem<R: Routable + Clone + PartialEq + 'static> {
    Link(R, &'static str),
    Group(&'static str, &'static [(R, &'static str)]),
}

impl<R: Routable + Clone + PartialEq> NavItem<R> {
    fn is_active(&self, current: &str) -> bool {
        match self {
            Self::Link(route, _) => is_path_active(current, &routable_path_only(route)),
            Self::Group(_, children) => children
                .iter()
                .any(|(route, _)| is_path_active(current, &routable_path_only(route))),
        }
    }
}

// `mut` is required on wasm32 for `.set` / `.with_mut`; native builds strip those cfg blocks.
#[allow(unused_mut)]
#[component]
pub fn NavTabs<R: Routable + Clone + PartialEq + 'static>(
    #[props(default = NavTabsDirection::Horizontal)] direction: NavTabsDirection,
    items: &'static [NavItem<R>],
    /// Current URL pathname (no query string). Use `ReadSignal::from(use_memo(...))` from the router.
    #[props(default)]
    current_path: Option<ReadSignal<String>>,
) -> Element {
    let n = items.len();
    let empty_path = use_signal(String::new);
    let path: ReadSignal<String> = current_path.unwrap_or(ReadSignal::from(empty_path));
    let mut nav_el = use_signal(|| None::<web_sys::Element>);
    let mut label_els = use_signal(|| vec![None::<web_sys::Element>; n]);
    let mut indicator_style = use_signal(|| None::<String>);

    use_effect(move || {
        let _ = path();
        #[cfg(feature = "web")]
        {
            // Subscribe to nav + label refs so we re-measure after `onmounted` fills them (vertical
            // stacks often lose a path-only `setTimeout(0)` race vs horizontal row layout).
            let _ = nav_el();
            let _ = label_els();
            let axis = match direction {
                NavTabsDirection::Horizontal => SlidingIndicatorAxis::Horizontal,
                NavTabsDirection::Vertical => SlidingIndicatorAxis::Vertical,
            };
            let items_ref = items;
            let path_c = path;
            let label_els_c = label_els;
            let nav_el_c = nav_el;
            let mut indicator_style_c = indicator_style;

            let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                let path_str = path_c();
                let labels = label_els_c();
                let nav = nav_el_c();
                let style = match (items_ref.iter().position(|it| it.is_active(&path_str)), nav) {
                    (Some(i), Some(nav)) if i < labels.len() => labels[i]
                        .as_ref()
                        .map(|label| sliding_indicator_style(axis, &nav, label)),
                    _ => None,
                };
                indicator_style_c.set(style);
            });
            if let Some(w) = web_sys::window() {
                let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    0,
                );
            }
        }
        #[cfg(not(feature = "web"))]
        {
            let _ = (path(), nav_el(), label_els());
        }
    });

    rsx! {
        nav {
            class: "{direction.container_class()}",
            onmounted: move |e| {
                #[cfg(feature = "web")]
                {
                    if let Some(el) = e.downcast::<web_sys::Element>() {
                        nav_el.set(Some(el.clone()));
                    }
                }
                #[cfg(not(feature = "web"))]
                {
                    let _ = e;
                }
            },
            div {
                class: "{direction.indicator_class()}",
                style: indicator_style().unwrap_or_default(),
            }
            for (i, item) in items.iter().enumerate() {
                {
                    let active = item.is_active(&path());
                    let idx = i;
                    match item {
                        NavItem::Link(route, label_str) => {
                            let tab_class = merge(&[direction.base_class(), direction.tab_class(active)]);
                            let to = route.clone();
                            rsx! {
                                Link {
                                    key: "{i}",
                                    to: to,
                                    class: tab_class,
                                    div { class: "absolute inset-x-1 inset-y-1 rounded-lg bg-secondary opacity-0 transition-opacity group-hover:opacity-100 -z-10" }
                                    span { class: "relative flex flex-col items-center justify-center text-sm pointer-events-none",
                                        span { class: "block font-bold h-0 overflow-hidden invisible", "aria-hidden": "true", "{label_str}" }
                                        span {
                                            class: "inline-block",
                                            onmounted: move |e| {
                                                #[cfg(feature = "web")]
                                                {
                                                    if let Some(el) = e.downcast::<web_sys::Element>() {
                                                        label_els.with_mut(|v| {
                                                            if idx < v.len() {
                                                                v[idx] = Some(el.clone());
                                                            }
                                                        });
                                                    }
                                                }
                                                #[cfg(not(feature = "web"))]
                                                {
                                                    let _ = (e, idx);
                                                }
                                            },
                                            "{label_str}"
                                        }
                                    }
                                }
                            }
                        }
                        NavItem::Group(label_str, children) => rsx! {
                            NavGroup::<R> {
                                key: "{i}",
                                direction,
                                label: *label_str,
                                routes: *children,
                                active,
                                path,
                                idx,
                                label_els,
                            }
                        },
                    }
                }
            }
        }
    }
}

/// A nav entry whose children are revealed in a flyout. The trigger is a real
/// `button` (keyboard-focusable, `aria-haspopup`/`aria-expanded`) and the panel
/// opens on click/keyboard as well as hover, so it is reachable without a pointer.
#[allow(unused_mut)]
#[component]
fn NavGroup<R: Routable + Clone + PartialEq + 'static>(
    direction: NavTabsDirection,
    label: &'static str,
    routes: &'static [(R, &'static str)],
    active: bool,
    path: ReadSignal<String>,
    idx: usize,
    mut label_els: Signal<Vec<Option<web_sys::Element>>>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut root_el = use_signal(|| None::<web_sys::Element>);

    use_outside_dismiss(root_el, open.into(), move || {
        if *open.peek() {
            open.set(false);
        }
    });
    // Stacked Escape: fires whether focus is on the trigger or inside the open
    // panel, and only closes the top-most overlay (unlike an inline onkeydown on
    // the trigger, which misses panel focus and ignores nesting).
    use_escape_listener(move || {
        if *open.peek() {
            open.set(false);
        }
    });

    let chevron = if direction == NavTabsDirection::Vertical {
        IconName::ChevronRight
    } else {
        IconName::ChevronDown
    };
    let trigger_class = merge(&[direction.base_class(), direction.tab_class(active)]);
    let panel_visibility = if open() {
        "block"
    } else {
        "hidden group-hover/tab:block"
    };
    let panel_class = merge(&[direction.child_panel_class(), panel_visibility]);

    rsx! {
        div {
            class: "group/tab relative",
            onmounted: move |e| {
                #[cfg(feature = "web")]
                {
                    if let Some(el) = e.downcast::<web_sys::Element>() {
                        root_el.set(Some(el.clone()));
                    }
                }
                #[cfg(not(feature = "web"))]
                {
                    let _ = e;
                }
            },
            button {
                r#type: "button",
                class: "{trigger_class}",
                "aria-haspopup": "true",
                "aria-expanded": open(),
                onclick: move |_| {
                    let next = !*open.peek();
                    open.set(next);
                },
                div { class: "absolute inset-x-1 inset-y-1 rounded-lg bg-secondary opacity-0 transition-opacity group-hover/tab:opacity-100 -z-10" }
                span { class: "relative flex flex-col items-center justify-center text-sm pointer-events-none",
                    span { class: "block font-bold h-0 overflow-hidden invisible", "aria-hidden": "true", "{label}" }
                    span { class: "flex items-center gap-1",
                        span {
                            class: "inline-block",
                            onmounted: move |e| {
                                #[cfg(feature = "web")]
                                {
                                    if let Some(el) = e.downcast::<web_sys::Element>() {
                                        label_els
                                            .with_mut(|v| {
                                                if idx < v.len() {
                                                    v[idx] = Some(el.clone());
                                                }
                                            });
                                    }
                                }
                                #[cfg(not(feature = "web"))]
                                {
                                    let _ = (e, idx);
                                }
                            },
                            "{label}"
                        }
                        Icon { name: chevron, class: direction.child_chevron_class() }
                    }
                }
            }
            div { class: "{panel_class}",
                div { class: "flex flex-col bg-popover rounded-xl shadow-lg py-1 min-w-[160px]",
                    for (child_route , child_label) in routes.iter() {
                        {
                            let child_path = routable_path_only(child_route);
                            let child_active = is_path_active(&path(), &child_path);
                            let child_class = direction.child_item_class(child_active);
                            let to = child_route.clone();
                            rsx! {
                                Link {
                                    key: "{child_path}",
                                    to: to,
                                    class: child_class.to_string(),
                                    onclick: move |_| open.set(false),
                                    "{child_label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
