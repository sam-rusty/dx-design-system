//! SSR pins for the nav family: `Tabs` ids/ARIA wiring and `SegmentedControl`'s
//! radiogroup contract.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "nav")]

use dioxus::dioxus_core::VirtualDom;
use dioxus::prelude::*;
use ds_components as components;

use components::{SegmentedControl, TabItem, Tabs};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn tabs_app() -> Element {
    let items = vec![
        TabItem {
            key: "one",
            label: rsx! { "One" },
            children: rsx! { p { "first panel" } },
            disabled: false,
        },
        TabItem {
            key: "two",
            label: rsx! { "Two" },
            children: rsx! { p { "second panel" } },
            disabled: false,
        },
    ];
    rsx! {
        Tabs { items, default_active_key: "two" }
    }
}

#[test]
fn tabs_wire_ids_and_aria_selected() {
    let html = render(tabs_app);
    assert!(html.contains(r#"role="tablist""#), "{html}");
    // Ids derive from the scope id: `cmp-<n>-tab-<key>` / `cmp-<n>-panel-<key>`.
    let tab_id_start = html.find(r#"id="cmp-"#).expect("tab id");
    let tab_id = &html[tab_id_start + 4..];
    let tab_id = &tab_id[..tab_id.find('"').unwrap()];
    assert!(tab_id.ends_with("-tab-one"), "{tab_id}");
    let base = tab_id.trim_end_matches("-tab-one");
    assert!(html.contains(&format!(r#"id="{base}-tab-two""#)), "{html}");
    assert!(
        html.contains(&format!(r#"id="{base}-panel-two""#)),
        "{html}"
    );
    assert!(
        html.contains(&format!(r#"aria-labelledby="{base}-tab-two""#)),
        "{html}"
    );
    // Only the active tab references its (rendered) panel.
    assert!(
        html.contains(&format!(r#"aria-controls="{base}-panel-two""#)),
        "{html}"
    );
    assert!(
        !html.contains(&format!(r#"aria-controls="{base}-panel-one""#)),
        "{html}"
    );

    assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1, "{html}");
    assert_eq!(
        html.matches(r#"aria-selected="false""#).count(),
        1,
        "{html}"
    );
    assert!(
        html.contains("second panel") && !html.contains("first panel"),
        "{html}"
    );
    // Roving tabindex: the active tab is the single tab stop in the strip.
    assert_eq!(html.matches(r#"role="tab""#).count(), 2);
    assert_eq!(html.matches(r#"tabindex="-1""#).count(), 1, "{html}");
}

fn segmented_app() -> Element {
    rsx! {
        SegmentedControl {
            value: "week".to_string(),
            on_change: |_| {},
            options: &[("day", "Day"), ("week", "Week"), ("month", "Month")],
        }
    }
}

#[test]
fn segmented_control_is_a_radiogroup_with_one_tab_stop() {
    let html = render(segmented_app);
    assert!(html.contains(r#"role="radiogroup""#), "{html}");
    assert_eq!(html.matches(r#"role="radio""#).count(), 3, "{html}");
    assert_eq!(html.matches(r#"aria-checked="true""#).count(), 1, "{html}");
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 1, "{html}");
    assert_eq!(html.matches(r#"tabindex="-1""#).count(), 2, "{html}");
    // The checked option is the tab stop.
    let checked = html.find(r#"aria-checked="true""#).unwrap();
    let tag_end = checked + html[checked..].find('>').unwrap();
    assert!(
        html[checked..tag_end].contains(r#"tabindex="0""#),
        "{}",
        &html[checked..tag_end]
    );
}
