use dioxus::prelude::*;

#[cfg(feature = "web")]
use crate::focus::focus_first_descendant;
use crate::hooks::{use_escape_listener_open, use_outside_dismiss, use_unique_id};
use crate::icon::{Icon, IconName};
use crate::link::Link;
use crate::placement::Placement;
use crate::{Text, TextSize};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownMenuSize {
    #[default]
    Default,
    Small,
    /// Content-sized panel (`w-max`, clamped) so variable-length items never wrap.
    Auto,
    /// Wider panel with scroll (e.g. header people search results).
    Search,
}

/// Horizontal edge the panel aligns to relative to its trigger. `End` (default) keeps the
/// historical right-aligned behavior; `Start` left-aligns the panel under the trigger — use it
/// for triggers anchored to the left (e.g. inline sentence-builder tokens).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownMenuAlign {
    Start,
    #[default]
    End,
}

const ITEM_CLASS: &str = "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-foreground transition-colors hover:bg-accent hover:text-accent-foreground cursor-pointer text-left focus:outline-none focus:bg-accent focus:text-accent-foreground";
const RADIO_ITEM_CLASS: &str = "flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm font-medium text-foreground transition-colors hover:bg-accent hover:text-accent-foreground cursor-pointer text-left focus:outline-none focus:bg-accent focus:text-accent-foreground";
const ICON_WRAP: &str = "text-muted-foreground [&>svg]:w-5 [&>svg]:h-5 [&>svg]:stroke-2";
const ITEM_CLASS_SM: &str = "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-accent hover:text-accent-foreground cursor-pointer text-left focus:outline-none focus:bg-accent focus:text-accent-foreground";
const ICON_WRAP_SM: &str = "text-muted-foreground [&>svg]:w-4 [&>svg]:h-4 [&>svg]:stroke-2";
/// The submenu trigger's chevron: a right chevron rotated to point left, since
/// [`DropdownMenuSub`] opens its panel to the left of the parent menu.
const SUB_CHEVRON_CLASS: &str =
    "w-4 h-4 text-muted-foreground transition-transform duration-150 rotate-180";

/// Wrap part of the tree (typically the app shell) so at most one [`DropdownMenu`] is open:
/// opening another menu closes the previous one. If this provider is absent, each menu toggles
/// independently.
#[derive(Clone, Copy)]
pub struct DropdownMenuCoordinator {
    open_id: Signal<Option<String>>,
}

impl DropdownMenuCoordinator {
    fn is_open(&self, id: &str) -> bool {
        self.open_id.read().as_deref() == Some(id)
    }

    fn open(&mut self, id: String) {
        self.open_id.set(Some(id));
    }

    fn close(&mut self, id: &str) {
        if self.open_id.peek().as_deref() == Some(id) {
            self.open_id.set(None);
        }
    }
}

#[component]
pub fn DropdownMenuCoordinatorProvider(children: Element) -> Element {
    let open_id = use_signal(|| None::<String>);
    use_context_provider(|| DropdownMenuCoordinator { open_id });
    rsx! {
        {children}
    }
}

fn dropdown_panel_class(
    size: DropdownMenuSize,
    is_open: bool,
    placement: Placement,
    align: DropdownMenuAlign,
) -> String {
    // Vertical edge + horizontal alignment, kept as explicit literals so Tailwind's source
    // scanner picks up every `origin-*` / edge class (composed strings would be missed).
    let pos = match (placement, align) {
        (Placement::Top, DropdownMenuAlign::Start) => "bottom-full mb-2 left-0 origin-bottom-left",
        (Placement::Top, DropdownMenuAlign::End) => "bottom-full mb-2 right-0 origin-bottom-right",
        (Placement::Bottom | Placement::Auto, DropdownMenuAlign::Start) => {
            "top-full mt-2 left-0 origin-top-left"
        }
        (Placement::Bottom | Placement::Auto, DropdownMenuAlign::End) => {
            "top-full mt-2 right-0 origin-top-right"
        }
        (Placement::Left, _) => "right-full mr-2 origin-top-right",
        (Placement::Right, _) => "left-full ml-2 origin-top-left",
    };
    let vis = if is_open {
        "opacity-100 scale-100 pointer-events-auto"
    } else {
        "opacity-0 scale-95 pointer-events-none hidden"
    };
    let dims = match size {
        DropdownMenuSize::Small => "w-44 max-w-[calc(100vw-2rem)]",
        DropdownMenuSize::Default => "w-56 max-w-[calc(100vw-2rem)]",
        DropdownMenuSize::Auto => "w-max min-w-[12rem] max-w-[min(20rem,calc(100vw-2rem))]",
        DropdownMenuSize::Search => {
            "min-w-[min(18rem,calc(100vw-2rem))] max-w-[calc(100vw-2rem)] max-h-[min(24rem,50vh)] overflow-y-auto overflow-x-hidden w-max"
        }
    };
    format!(
        "absolute {pos} {dims} rounded-2xl bg-popover text-popover-foreground p-2 transition-all duration-150 z-[100] border border-border shadow-lg {vis}"
    )
}

/// Shared by the menu with its items so every item close (link, button, radio,
/// close button) goes through the menu's single `close` path — which also
/// releases the [`DropdownMenuCoordinator`] slot, so a consumer-owned `open`
/// signal never reads a stale `true`.
#[derive(Clone, Copy)]
struct DropdownContext {
    close: Callback<()>,
}

/// Move focus to the previous/next/first/last `menuitem` inside `root` in
/// response to an arrow / Home / End key, so the menu is keyboard-operable.
#[cfg(feature = "web")]
fn menu_focus_move(root: &web_sys::Element, key: &Key) {
    use wasm_bindgen::{JsCast, JsValue};

    let Some(menu) = root.query_selector("[role='menu']").ok().flatten() else {
        return;
    };
    let Ok(list) = menu.query_selector_all("a[href], button:not([disabled])") else {
        return;
    };
    let items: Vec<web_sys::HtmlElement> = (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        // Skip items inside a closed submenu (display:none → no offset parent).
        .filter(|el| el.offset_parent().is_some())
        .collect();
    if items.is_empty() {
        return;
    }
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());
    let current = active.as_ref().and_then(|a| {
        let av: &JsValue = a.as_ref();
        items.iter().position(|it| {
            let iv: &JsValue = it.as_ref();
            iv == av
        })
    });
    let last = items.len() - 1;
    let target = match key {
        Key::ArrowDown => current.map(|c| (c + 1).min(last)).unwrap_or(0),
        Key::ArrowUp => current.map(|c| c.saturating_sub(1)).unwrap_or(last),
        Key::Home => 0,
        Key::End => last,
        _ => return,
    };
    let _ = items[target].focus();
}

#[component]
pub fn DropdownMenu(
    #[props(default)] size: DropdownMenuSize,
    #[props(default)] placement: Placement,
    #[props(default)] align: DropdownMenuAlign,
    #[props(default)] open: Option<Signal<bool>>,
    trigger: Element,
    children: Element,
) -> Element {
    let fallback_open = use_signal(|| false);
    let mut eff_open = open.unwrap_or(fallback_open);
    let coordinator = try_use_context::<DropdownMenuCoordinator>();

    let base_id = use_unique_id();
    let menu_id: ReadSignal<String> = use_memo(move || format!("{}-menu", base_id())).into();
    let panel_id: ReadSignal<String> = use_memo(move || format!("{}-panel", base_id())).into();
    let mut root_el = use_signal(|| None::<web_sys::Element>);

    // Visible only when our state is open *and* — if a coordinator governs us —
    // we are the menu it has registered as open. Deriving this (rather than a
    // write-back effect) is what lets a second menu opening close the first.
    let is_open = use_memo(move || {
        let base = eff_open();
        match coordinator {
            Some(c) => base && c.is_open(&menu_id.peek()),
            None => base,
        }
    });

    let close = use_callback(move |_: ()| {
        if is_open() {
            eff_open.set(false);
            if let Some(mut c) = coordinator {
                c.close(&menu_id.peek());
            }
        }
    });

    use_context_provider(|| DropdownContext { close });
    use_context_provider(|| size);

    use_escape_listener_open(is_open.into(), move || close.call(()));
    use_outside_dismiss(root_el, is_open.into(), move || close.call(()));

    let toggle = move |_| {
        if is_open() {
            close.call(());
        } else {
            if let Some(mut c) = coordinator {
                c.open(menu_id.peek().clone());
            }
            eff_open.set(true);
        }
    };

    let on_nav = move |e: KeyboardEvent| match e.key() {
        Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End => {
            e.prevent_default();
            #[cfg(feature = "web")]
            if let Some(root) = root_el.peek().clone() {
                menu_focus_move(&root, &e.key());
            }
        }
        _ => {}
    };

    let panel_class = dropdown_panel_class(size, is_open(), placement, align);

    rsx! {
        div {
            class: "relative inline-block text-left",
            onkeydown: on_nav,
            onmounted: move |e| {
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    root_el.set(Some(el.clone()));
                }
            },
            div {
                class: "cursor-pointer",
                "aria-haspopup": "menu",
                "aria-expanded": is_open(),
                "aria-controls": panel_id,
                onclick: toggle,
                {trigger}
            }
            div {
                id: panel_id,
                role: "menu",
                "data-side": placement.data_side(),
                class: "{panel_class}",
                div { class: "flex flex-col gap-0.5",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn DropdownMenuItem(
    #[props(default)] to: Option<NavigationTarget>,
    #[props(default)] on_click: Option<EventHandler<()>>,
    icon: Element,
    label: Element,
) -> Element {
    let close = use_context::<DropdownContext>().close;
    let size = try_use_context::<DropdownMenuSize>().unwrap_or_default();
    let item_class = match size {
        DropdownMenuSize::Small => ITEM_CLASS_SM,
        DropdownMenuSize::Default | DropdownMenuSize::Auto | DropdownMenuSize::Search => ITEM_CLASS,
    };
    let icon_wrap_class = match size {
        DropdownMenuSize::Small => ICON_WRAP_SM,
        DropdownMenuSize::Default | DropdownMenuSize::Auto | DropdownMenuSize::Search => ICON_WRAP,
    };

    if let Some(target) = to {
        rsx! {
            Link {
                to: target,
                class: "{item_class}",
                role: "menuitem",
                onclick: move |_| close.call(()),
                span { class: "{icon_wrap_class}", {icon} }
                Text { size: TextSize::Small, {label} }
            }
        }
    } else {
        rsx! {
            button {
                role: "menuitem",
                class: "{item_class}",
                onclick: move |_| {
                    close.call(());
                    if let Some(cb) = &on_click {
                        cb.call(());
                    }
                },
                span { class: "{icon_wrap_class}", {icon} }
                Text { size: TextSize::Small, {label} }
            }
        }
    }
}

/// Nested menu that flies out to the left of its trigger. Opens on hover,
/// click, or ArrowRight (which also moves focus into it); ArrowLeft from
/// anywhere inside closes it and returns focus to the trigger.
#[component]
pub fn DropdownMenuSub(icon: Element, label: Element, children: Element) -> Element {
    let mut is_sub_open = use_signal(|| false);
    // Set by ArrowRight: focus the first submenu item once the panel is visible
    // (it is `hidden` until the open state has rendered, so focusing in the
    // key handler itself would fail).
    let mut focus_pending = use_signal(|| false);
    let mut trigger_el = use_signal(|| None::<web_sys::Element>);
    let mut panel_el = use_signal(|| None::<web_sys::Element>);
    let is_open_val = is_sub_open();

    use_effect(move || {
        if is_sub_open() && focus_pending() {
            focus_pending.set(false);
            #[cfg(feature = "web")]
            if let Some(panel) = panel_el.peek().clone() {
                focus_first_descendant(&panel);
            }
        }
    });

    let visibility = if is_open_val {
        "opacity-100 scale-100 pointer-events-auto"
    } else {
        "opacity-0 scale-95 pointer-events-none hidden"
    };
    // Transparent positioning wrapper abuts the trigger (no margin gap); the 4px
    // visual offset comes from `pr-1` padding, which stays inside the hover surface
    // so the cursor never crosses a dead zone between trigger and panel.
    let sub_wrap_class = format!(
        "absolute right-full top-0 pr-1 z-50 transition-all duration-150 \
         origin-top-right {visibility}"
    );

    rsx! {
        div {
            class: "relative",
            onmouseenter: move |_| {
                *is_sub_open.write() = true;
            },
            onmouseleave: move |_| {
                *is_sub_open.write() = false;
            },
            onkeydown: move |e| {
                if e.key() == Key::ArrowLeft && *is_sub_open.peek() {
                    e.prevent_default();
                    e.stop_propagation();
                    is_sub_open.set(false);
                    #[cfg(feature = "web")]
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(html) = trigger_el
                            .peek()
                            .as_ref()
                            .and_then(|el| el.dyn_ref::<web_sys::HtmlElement>())
                        {
                            let _ = html.focus();
                        }
                    }
                }
            },
            button {
                role: "menuitem",
                "aria-haspopup": "menu",
                "aria-expanded": is_open_val,
                class: "{ITEM_CLASS}",
                onmounted: move |e| {
                    if let Some(el) = e.downcast::<web_sys::Element>() {
                        trigger_el.set(Some(el.clone()));
                    }
                },
                onclick: move |_| {
                    *is_sub_open.write() ^= true;
                },
                onkeydown: move |e| {
                    if e.key() == Key::ArrowRight {
                        e.prevent_default();
                        e.stop_propagation();
                        focus_pending.set(true);
                        is_sub_open.set(true);
                    }
                },
                span { class: "{ICON_WRAP}", {icon} }
                Text { size: TextSize::Small, class: "flex-1", {label} }
                Icon { name: IconName::ChevronRight, class: "{SUB_CHEVRON_CLASS}" }
            }
            div {
                class: "{sub_wrap_class}",
                div {
                    role: "menu",
                    class: "w-48 rounded-2xl bg-popover text-popover-foreground p-2 border border-border shadow-lg",
                    onmounted: move |e| {
                        if let Some(el) = e.downcast::<web_sys::Element>() {
                            panel_el.set(Some(el.clone()));
                        }
                    },
                    div { class: "flex flex-col gap-0.5",
                        {children}
                    }
                }
            }
        }
    }
}

#[component]
pub fn DropdownMenuRadioItem(
    label: Element,
    active: ReadSignal<bool>,
    #[props(default)] icon: Option<Element>,
    #[props(default)] on_select: Option<EventHandler<()>>,
) -> Element {
    let is_active = active();
    let span_class = if is_active {
        "w-4 h-4 rounded-full border-[5px] border-primary transition-all duration-150"
    } else {
        "w-4 h-4 rounded-full border-2 border-muted-foreground/40 transition-all duration-150"
    };
    let close = try_use_context::<DropdownContext>().map(|ctx| ctx.close);
    let size = try_use_context::<DropdownMenuSize>().unwrap_or_default();
    let icon_wrap_class = match size {
        DropdownMenuSize::Small => ICON_WRAP_SM,
        DropdownMenuSize::Default | DropdownMenuSize::Auto | DropdownMenuSize::Search => ICON_WRAP,
    };

    rsx! {
        button {
            role: "menuitemradio",
            "aria-checked": is_active,
            class: "{RADIO_ITEM_CLASS}",
            onclick: move |_| {
                if let Some(close) = close {
                    close.call(());
                }
                if let Some(cb) = &on_select {
                    cb.call(());
                }
            },
            div { class: "flex items-center gap-3 min-w-0",
                if let Some(icon) = icon {
                    span { class: "{icon_wrap_class}", {icon} }
                }
                Text { size: TextSize::Small, {label} }
            }
            span { class: "{span_class}" }
        }
    }
}

#[component]
pub fn DropdownMenuSeparator() -> Element {
    rsx! {
        div { role: "separator", class: "my-1 border-t border-border/50 mx-2" }
    }
}

/// Groups related menu items under an optional uppercase section label. Every group after the
/// first draws a top divider automatically, so callers just stack `DropdownMenuGroup`s without
/// interleaving `DropdownMenuSeparator`s.
#[component]
pub fn DropdownMenuGroup(#[props(optional)] label: Option<String>, children: Element) -> Element {
    rsx! {
        div {
            role: "group",
            class: "flex flex-col gap-0.5 [&:not(:first-child)]:mt-1 [&:not(:first-child)]:border-t [&:not(:first-child)]:border-border/60 [&:not(:first-child)]:pt-1.5",
            if let Some(label) = label {
                div {
                    class: "px-2.5 pt-1 pb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground select-none",
                    "{label}"
                }
            }
            {children}
        }
    }
}

#[component]
pub fn DropdownCloseButton(
    #[props(default)] on_click: Option<EventHandler<()>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let close = use_context::<DropdownContext>().close;
    rsx! {
        button {
            role: "menuitem",
            class: "{class}",
            onclick: move |_| {
                if let Some(cb) = &on_click {
                    cb.call(());
                }
                close.call(());
            },
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DropdownMenuAlign, DropdownMenuSize, dropdown_panel_class};
    use crate::placement::Placement;

    #[test]
    fn closed_panel_is_hidden_and_open_panel_is_not() {
        let closed = dropdown_panel_class(
            DropdownMenuSize::Default,
            false,
            Placement::Bottom,
            DropdownMenuAlign::End,
        );
        assert!(closed.split_whitespace().any(|c| c == "hidden"));
        assert!(closed.contains("pointer-events-none"));

        let open = dropdown_panel_class(
            DropdownMenuSize::Default,
            true,
            Placement::Bottom,
            DropdownMenuAlign::End,
        );
        assert!(!open.split_whitespace().any(|c| c == "hidden"));
        assert!(open.contains("pointer-events-auto"));
    }

    #[test]
    fn alignment_picks_edge_and_origin() {
        let start = dropdown_panel_class(
            DropdownMenuSize::Default,
            true,
            Placement::Bottom,
            DropdownMenuAlign::Start,
        );
        assert!(start.contains("left-0") && start.contains("origin-top-left"));
        let end = dropdown_panel_class(
            DropdownMenuSize::Default,
            true,
            Placement::Top,
            DropdownMenuAlign::End,
        );
        assert!(end.contains("right-0") && end.contains("origin-bottom-right"));
    }

    #[test]
    fn auto_matches_bottom_and_sizes_differ() {
        let auto = dropdown_panel_class(
            DropdownMenuSize::Small,
            true,
            Placement::Auto,
            DropdownMenuAlign::End,
        );
        let bottom = dropdown_panel_class(
            DropdownMenuSize::Small,
            true,
            Placement::Bottom,
            DropdownMenuAlign::End,
        );
        assert_eq!(auto, bottom);
        assert!(auto.contains("w-44"));
        let search = dropdown_panel_class(
            DropdownMenuSize::Search,
            true,
            Placement::Bottom,
            DropdownMenuAlign::End,
        );
        assert!(search.contains("overflow-y-auto"));
    }
}
