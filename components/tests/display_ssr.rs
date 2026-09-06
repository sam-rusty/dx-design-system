//! SSR pins for the display family: `Avatar`, `Accordion`, `EmptyState`, `StatTile`.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "display")]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;

use components::{Accordion, AccordionItem, Avatar, EmptyState, IconName, StatTile, StatTone};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn avatar_role_img_only_with_alt() {
    fn named() -> Element {
        rsx! {
            Avatar { src: "/a.png", alt: "Ada", "AL" }
        }
    }
    let html = render(named);
    assert!(html.contains(r#"role="img""#), "{html}");
    assert!(html.contains(r#"aria-label="Ada""#), "{html}");
    assert!(
        html.contains(r#"<img"#) && html.contains(r#"alt="""#),
        "{html}"
    );
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    assert!(!html.contains("AL"), "{html}");

    fn unnamed() -> Element {
        rsx! {
            Avatar { "AL" }
        }
    }
    let html = render(unnamed);
    assert!(!html.contains("role="), "{html}");
    assert!(!html.contains("<img"), "{html}");
    assert!(html.contains("AL"), "{html}");
}

#[test]
fn accordion_wires_ids_and_regions() {
    fn app() -> Element {
        let items = vec![
            AccordionItem {
                key: "a",
                label: rsx! { "First" },
                children: rsx! { p { "one" } },
                disabled: false,
                summary: None,
            },
            AccordionItem {
                key: "b",
                label: rsx! { "Second" },
                children: rsx! { p { "two" } },
                disabled: true,
                summary: Some(rsx! { "1 item" }),
            },
        ];
        rsx! {
            Accordion { items, default_active_keys: vec!["a"], class: "extra" }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-expanded=true"), "{html}");
    assert!(html.contains("aria-expanded=false"), "{html}");
    // Open panel is a region labelled by its header; the header controls that panel.
    let panel_id_start = html.find(r#"role="region""#).expect("region");
    let region = &html[panel_id_start.saturating_sub(120)..panel_id_start + 200];
    let id = region
        .split("id=\"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("panel id");
    assert!(id.ends_with("-panel"), "{id}");
    assert!(html.contains(&format!(r#"aria-controls="{id}""#)), "{html}");
    let header = id.trim_end_matches("-panel");
    assert!(
        html.contains(&format!(r#"aria-labelledby="{header}-header""#)),
        "{html}"
    );
    // Closed panel content is not rendered; collapsed summary is.
    assert!(html.contains("<p>one</p>"), "{html}");
    assert!(!html.contains("<p>two</p>"), "{html}");
    assert!(html.contains("1 item"), "{html}");
    // Disabled section is a disabled button.
    assert!(html.contains("disabled"), "{html}");
    // `class` merges with the base classes.
    assert!(html.contains("overflow-hidden extra"), "{html}");
}

#[test]
fn empty_state_both_layouts() {
    fn plain() -> Element {
        rsx! {
            EmptyState { message: "Nothing yet" }
        }
    }
    let html = render(plain);
    assert!(html.contains("border-dashed"), "{html}");
    assert!(html.contains("Nothing yet"), "{html}");

    fn iconed() -> Element {
        rsx! {
            EmptyState { message: "Nothing yet", description: "Add one", icon: IconName::Plus,
                button { "Add" }
            }
        }
    }
    let html = render(iconed);
    assert!(!html.contains("border-dashed"), "{html}");
    assert!(html.contains("<svg"), "{html}");
    assert!(html.contains("Add one"), "{html}");
    assert!(html.contains("<button>Add</button>"), "{html}");
}

#[test]
fn stat_tile_tone_class() {
    fn app() -> Element {
        rsx! {
            StatTile { label: "Revenue", value: "$1", sub: "this month", tone: StatTone::Success }
            StatTile { label: "Loss", value: "$2", tone: StatTone::Destructive }
        }
    }
    let html = render(app);
    assert!(html.contains("text-success"), "{html}");
    assert!(html.contains("text-destructive"), "{html}");
    assert!(html.contains("this month"), "{html}");
}
