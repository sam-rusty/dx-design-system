//! SSR pins for always-compiled primitives: `Button`, `Separator`, `Copyable`,
//! `Badge`, `Card`, `Icon`, `Link`, layout attribute passthrough, and the
//! `RichTextEditorBase` server fallback.

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;

use components::{
    Badge, BadgeVariant, Button, Card, CardVariant, Copyable, Grid, Icon, IconName, Link, LinkType,
    Row, Separator, SeparatorOrientation,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn button_loading_is_busy_disabled_with_spinner_and_spreads_attributes() {
    fn app() -> Element {
        rsx! {
            Button { loading: true, icon: IconName::Plus, "aria-label": "Save", "Save" }
        }
    }
    let html = render(app);
    // Boolean attributes serialise unquoted under SSR.
    assert!(html.contains("aria-busy=true"), "{html}");
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains("animate-spin"), "{html}");
    assert!(html.contains(r#"aria-label="Save""#), "{html}");
    assert!(html.contains(r#"data-name="Button""#), "{html}");
}

#[test]
fn separator_roles() {
    fn app() -> Element {
        rsx! {
            Separator {}
            Separator { orientation: SeparatorOrientation::Vertical }
            Separator { decorative: true }
        }
    }
    let html = render(app);
    assert!(
        html.contains(r#"role="separator" aria-orientation="horizontal""#),
        "{html}"
    );
    assert!(
        html.contains(r#"role="separator" aria-orientation="vertical""#),
        "{html}"
    );
    assert!(html.contains(r#"role="none""#), "{html}");
    assert_eq!(html.matches("aria-orientation").count(), 2, "{html}");
}

#[test]
fn copyable_labels_and_status_region() {
    fn app() -> Element {
        rsx! {
            Copyable { text: "abc-123" }
        }
    }
    let html = render(app);
    assert!(html.contains(r#"aria-label="Copy abc-123""#), "{html}");
    assert!(html.contains(r#"title="Copy""#), "{html}");
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(!html.contains("Copied"), "{html}");
}

#[test]
fn badge_variant_and_size_classes() {
    fn app() -> Element {
        rsx! {
            Badge { variant: BadgeVariant::Success, class: "extra", "ok" }
        }
    }
    let html = render(app);
    assert!(html.contains("inline-flex items-center rounded-full shrink-0 bg-success/10 text-success text-xs px-2 py-0.5 font-medium extra"), "{html}");
}

#[test]
fn card_variant_path_has_keyboard_parity() {
    fn app() -> Element {
        rsx! {
            Card { variant: CardVariant::Plain, onclick: |_| {}, "a" }
            Card { onclick: |_| {}, "b" }
            Card { variant: CardVariant::Tile, "c" }
        }
    }
    let html = render(app);
    assert_eq!(html.matches(r#"role="button""#).count(), 2, "{html}");
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 2, "{html}");
    assert!(html.contains(r#"data-name="Box""#), "{html}");
}

#[test]
fn icon_is_hidden_from_assistive_tech() {
    fn app() -> Element {
        rsx! {
            Icon { name: IconName::Check }
            Icon { name: IconName::Spinner }
            Icon { name: IconName::Behance }
        }
    }
    let html = render(app);
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 3, "{html}");
    assert_eq!(html.matches(r#"focusable="false""#).count(), 3, "{html}");
}

#[test]
fn link_button_type_gets_button_classes() {
    #[derive(Routable, Clone, PartialEq)]
    enum Route {
        #[route("/")]
        Home {},
    }
    #[component]
    fn Home() -> Element {
        rsx! {
            Link { to: Route::Home {}, r#type: LinkType::Button, "go" }
        }
    }
    fn app() -> Element {
        rsx! {
            Router::<Route> {}
        }
    }
    let html = render(app);
    assert!(html.contains("rounded-full"), "{html}");
    assert!(
        html.contains("bg-primary text-primary-foreground"),
        "{html}"
    );
}

#[test]
fn layout_primitives_forward_attributes() {
    fn app() -> Element {
        rsx! {
            Row { "aria-label": "toolbar", role: "toolbar", "x" }
            Grid { "aria-busy": "true", "y" }
        }
    }
    let html = render(app);
    assert!(html.contains(r#"role="toolbar""#), "{html}");
    assert!(html.contains(r#"aria-label="toolbar""#), "{html}");
    assert!(html.contains(r#"aria-busy="true""#), "{html}");
}

#[cfg(feature = "rich-text")]
#[test]
fn rich_text_editor_ssr_fallback_is_a_textarea() {
    use components::RichTextEditorBase;
    fn app() -> Element {
        let value = use_signal(|| "<b>hi</b>".to_string());
        rsx! {
            RichTextEditorBase { id: "body", value: Some(value), aria_label: "Body", disabled: true }
        }
    }
    let html = render(app);
    assert!(html.starts_with("<textarea"), "{html}");
    assert!(html.contains(r#"id="body""#), "{html}");
    assert!(html.contains(r#"aria-label="Body""#), "{html}");
    assert!(html.contains("disabled"), "{html}");
    assert!(!html.contains("contenteditable"), "{html}");
}
