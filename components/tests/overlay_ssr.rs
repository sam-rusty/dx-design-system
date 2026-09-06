//! SSR pins for the overlay and feedback families: portaled panels render
//! nothing on the server, and the always-rendered pieces keep their ARIA
//! contract.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(all(feature = "overlay", feature = "feedback"))]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;

use components::{
    Alert, AlertVariant, DropdownMenu, DropdownMenuItem, LoadingOverlay, Modal, Popover,
    PopoverConfirm, Progress, ToastStore, Toaster, Tooltip,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn modal_renders_no_panel_on_server() {
    fn app() -> Element {
        rsx! {
            Modal { title: "Confirm", on_close: |_| {},
                p { "body" }
            }
        }
    }
    let html = render(app);
    assert!(!html.contains(r#"role="dialog""#), "{html}");
    assert!(!html.contains("body"), "{html}");
}

#[test]
fn popover_renders_trigger_but_no_panel_on_server() {
    fn app() -> Element {
        rsx! {
            Popover { trigger: rsx! { span { "open" } }, default_open: true,
                p { "panel" }
            }
        }
    }
    let html = render(app);
    assert!(html.contains(">open<"), "{html}");
    assert!(html.contains(r#"role="button""#), "{html}");
    assert!(html.contains(r#"aria-haspopup="dialog""#), "{html}");
    assert!(!html.contains(r#"role="dialog""#), "{html}");
    assert!(!html.contains("panel"), "{html}");
}

#[test]
fn popover_confirm_trigger_is_not_a_toggle() {
    fn app() -> Element {
        rsx! {
            PopoverConfirm { message: "Sure?", on_confirm: |_| {},
                button { "Delete" }
            }
        }
    }
    let html = render(app);
    assert!(html.contains(">Delete<"), "{html}");
    assert!(!html.contains(r#"aria-haspopup"#), "{html}");
    assert!(!html.contains("Sure?"), "{html}");
}

#[test]
fn tooltip_renders_no_panel_and_no_dangling_describedby() {
    fn app() -> Element {
        rsx! {
            Tooltip { title: rsx! { "Hint" },
                button { "hover me" }
            }
        }
    }
    let html = render(app);
    assert!(html.contains(">hover me<"), "{html}");
    assert!(!html.contains(r#"role="tooltip""#), "{html}");
    assert!(!html.contains("aria-describedby"), "{html}");
}

#[test]
fn dropdown_closed_panel_is_hidden_and_link_item_is_menuitem() {
    fn app() -> Element {
        rsx! {
            Router::<Route> {}
        }
    }
    #[derive(Routable, Clone, PartialEq)]
    enum Route {
        #[route("/")]
        Home {},
    }
    #[component]
    fn Home() -> Element {
        rsx! {
            DropdownMenu { trigger: rsx! { button { "menu" } },
                DropdownMenuItem {
                    to: NavigationTarget::from(Route::Home {}),
                    icon: rsx! {},
                    label: rsx! { "Go home" },
                }
                DropdownMenuItem {
                    on_click: |_| {},
                    icon: rsx! {},
                    label: rsx! { "Act" },
                }
            }
        }
    }
    let html = render(app);
    assert!(html.contains(r#"role="menu""#), "{html}");
    assert!(html.contains("aria-expanded=false"), "{html}");
    let panel = html.find(r#"role="menu""#).unwrap();
    let panel_tag =
        &html[html[..panel].rfind('<').unwrap()..html[panel..].find('>').unwrap() + panel];
    assert!(panel_tag.contains(" hidden"), "{panel_tag}");
    assert_eq!(html.matches(r#"role="menuitem""#).count(), 2, "{html}");
}

#[test]
fn alert_has_alert_role() {
    fn app() -> Element {
        rsx! {
            Alert { variant: AlertVariant::Warning, "Careful" }
        }
    }
    let html = render(app);
    assert!(html.contains(r#"role="alert""#), "{html}");
    assert!(html.contains("Careful"), "{html}");
}

#[test]
fn progress_clamps_aria_valuenow() {
    fn app() -> Element {
        rsx! {
            Progress { value: 1.5 }
            Progress { value: -0.2 }
            Progress { value: 0.25 }
        }
    }
    let html = render(app);
    assert!(html.contains(r#"aria-valuenow="100""#), "{html}");
    assert!(html.contains(r#"aria-valuenow="0""#), "{html}");
    assert!(html.contains(r#"aria-valuenow="25""#), "{html}");
    assert!(html.contains("width: 100%"), "{html}");
    assert!(!html.contains("150"), "{html}");
}

#[test]
fn toaster_is_a_polite_live_region() {
    fn app() -> Element {
        use_context_provider(ToastStore::new);
        rsx! {
            Toaster {}
        }
    }
    let html = render(app);
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains(r#"aria-live="polite""#), "{html}");
}

#[test]
fn loading_overlay_renders_only_when_active() {
    fn hidden() -> Element {
        let when = use_signal(|| false);
        rsx! {
            LoadingOverlay { when }
        }
    }
    fn shown() -> Element {
        let when = use_signal(|| true);
        rsx! {
            LoadingOverlay { when, message: "Saving" }
        }
    }
    assert!(!render(hidden).contains(r#"role="status""#));
    let html = render(shown);
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains(r#"aria-busy="true""#), "{html}");
    assert!(html.contains("Saving"), "{html}");
}
