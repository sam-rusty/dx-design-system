//! SSR pins for `DataTable` / `Col` and the shared list status blocks.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "data-table")]

use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use ds_components as components;

use components::{Col, DataTable, ListEmpty, SortDir};

/// Render twice: `Col` children register into the table's column registry during
/// their own mount, which marks the table dirty for a second pass.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: &'static str,
}

fn people() -> Vec<Person> {
    vec![
        Person { id: 1, name: "Ada" },
        Person {
            id: 2,
            name: "Grace",
        },
    ]
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/page2")]
    Page2 {},
}

#[component]
fn Page2() -> Element {
    rsx! { "page 2" }
}

thread_local! {
    static SCENARIO: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

#[component]
fn Home() -> Element {
    let scenario = SCENARIO.with(|s| s.get());
    let sort_key = use_signal(|| "name".to_string());
    let sort_dir = use_signal(|| SortDir::Desc);
    let page1 = use_signal(|| 1u32);
    let page2 = use_signal(|| 2u32);
    match scenario {
        // sortable header + page 1 without more → no footer
        0 => rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                sort_key: Some(sort_key.into()),
                sort_dir: Some(sort_dir.into()),
                sort_href: |k: &'static str, d: SortDir| format!("/?sort={k}&dir={d:?}"),
                page: Some(page1.into()),
                has_more: false,
                page_href: |p: u32| format!("/?page={p}"),
                Col { id: "name", label: "Name", sortable: true, render: |p: &Person| rsx! { "{p.name}" } }
                Col { id: "id", label: "Id", sortable: true, render: |p: &Person| rsx! { "{p.id}" } }
            }
        },
        // page 2 → footer with a disabled "Next"
        1 => rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                page: Some(page2.into()),
                has_more: false,
                page_href: |p: u32| format!("/?page={p}"),
                Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            }
        },
        // page 1 with more → footer with disabled "Previous"
        _ => rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                page: Some(page1.into()),
                has_more: true,
                page_href: |p: u32| format!("/?page={p}"),
                Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            }
        },
    }
}

fn routed(scenario: u8) -> String {
    SCENARIO.with(|s| s.set(scenario));
    fn app() -> Element {
        rsx! {
            Router::<Route> {}
        }
    }
    render(app)
}

#[test]
fn header_labels_and_rows_render_from_col_children() {
    fn app() -> Element {
        rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                Col { id: "name", label: "Name", class: "flex-1", render: |p: &Person| rsx! { span { "{p.name}" } } }
                Col { id: "id", label: "Id", render: |p: &Person| rsx! { "{p.id}" } }
            }
        }
    }
    let html = render(app);
    assert!(html.contains(r#"<div class="flex-1">Name</div>"#), "{html}");
    assert!(html.contains(">Id</div>"), "{html}");
    assert!(html.contains("<span>Ada</span>"), "{html}");
    assert!(html.contains("<span>Grace</span>"), "{html}");
    // Cells carry the column class.
    assert!(
        html.contains(r#"<div class="flex-1"><span>Ada</span></div>"#),
        "{html}"
    );
    // No footer without `page`.
    assert!(!html.contains("Pagination"), "{html}");
}

#[test]
fn default_and_custom_empty_states() {
    fn app() -> Element {
        rsx! {
            DataTable {
                items: Vec::<Person>::new(),
                item_key: |p: &Person| p.id,
                Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            }
        }
    }
    let html = render(app);
    assert!(html.contains("No items found"), "{html}");
    assert!(html.contains("bg-card"), "{html}");

    fn custom() -> Element {
        rsx! {
            DataTable {
                items: Vec::<Person>::new(),
                item_key: |p: &Person| p.id,
                empty: rsx! { p { "nothing here" } },
                Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            }
        }
    }
    let html = render(custom);
    assert!(html.contains("<p>nothing here</p>"), "{html}");
    assert!(!html.contains("No items found"), "{html}");
}

#[test]
fn slots_are_wrapped_so_clicks_do_not_reach_the_row() {
    // E2: row_left / selection / actions live inside a `div.contents` that stops
    // propagation; the wrapper is the SSR-visible part of that contract.
    fn app() -> Element {
        rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                row_left: |id: u32| rsx! { i { "left-{id}" } },
                row_actions: |p: &Person| rsx! { b { "act-{p.id}" } },
                row_expand: |p: &Person| rsx! { "panel {p.id}" },
                Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            }
        }
    }
    let html = render(app);
    assert!(
        html.contains(r#"<div class="contents"><i>left-1</i></div>"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<div class="contents"><b>act-1</b></div>"#),
        "{html}"
    );
    // Expand chevron is a real, labelled button announcing its state.
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(html.contains(r#"aria-label="Toggle details""#), "{html}");
    // Panel is lazy: not rendered while closed.
    assert!(!html.contains("panel 1"), "{html}");
}

#[test]
fn selectable_renders_checkboxes_in_slot_wrappers() {
    fn app() -> Element {
        rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                selectable: true,
                Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            }
        }
    }
    let html = render(app);
    // header select-all + one per row
    assert!(html.matches("w-6 shrink-0").count() == 3, "{html}");
    assert!(
        html.contains(
            r#"<div class="contents"><div class="flex items-center justify-center w-6 shrink-0">"#
        ),
        "{html}"
    );
}

#[test]
fn sortable_headers_link_and_no_footer_on_first_page() {
    let html = routed(0);
    // `aria-sort` is only valid on a `columnheader`; the header stays a link
    // until the table gets grid semantics (deferred D1), so no sort attribute.
    assert!(!html.contains("aria-sort"), "{html}");
    // Current column toggles to the opposite direction.
    assert!(html.contains("sort=name&#38;dir=Asc"), "{html}");
    assert!(!html.contains("Pagination"), "{html}");
}

#[test]
fn pagination_footer_disables_inactive_side() {
    let html = routed(1);
    assert!(html.contains(r#"aria-label="Pagination""#), "{html}");
    assert!(html.contains("Page 2"), "{html}");
    // Next is disabled: dimmed classes + aria-disabled + out of tab order.
    let next = html.find("Next").expect("next link");
    let before = &html[next.saturating_sub(400)..next];
    assert!(before.contains(r#"aria-disabled="true""#), "{before}");
    assert!(before.contains(r#"tabindex="-1""#), "{before}");
    assert!(before.contains("pointer-events-none"), "{before}");
    // Previous is active.
    let prev = html.find("Previous").expect("prev link");
    let before = &html[prev.saturating_sub(400)..prev];
    assert!(!before.contains("aria-disabled"), "{before}");
    assert!(before.contains("hover:bg-accent"), "{before}");

    let html = routed(2);
    assert!(html.contains("Page 1"), "{html}");
    let prev = html.find("Previous").expect("prev link");
    let before = &html[prev.saturating_sub(400)..prev];
    assert!(before.contains(r#"aria-disabled="true""#), "{before}");
}

#[test]
fn loading_renders_skeleton_with_registered_column_count() {
    fn app() -> Element {
        rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                loading: true,
                skeleton_rows: 2,
                Col { id: "a", label: "A", render: |p: &Person| rsx! { "{p.name}" } }
                Col { id: "b", label: "B", render: |p: &Person| rsx! { "{p.name}" } }
            }
        }
    }
    let html = render(app);
    assert!(html.contains("animate-pulse"), "{html}");
    assert!(!html.contains("Ada"), "{html}");
    // Two skeleton rows, each with `columns - 1` trailing blocks → 2 columns → 1 each.
    assert_eq!(html.matches("size-8 rounded-full").count(), 2, "{html}");
    assert_eq!(
        html.matches("w-24 h-3 rounded bg-muted/40").count(),
        2,
        "{html}"
    );
    // Header shimmer: one block per column.
    assert_eq!(html.matches("h-3 rounded bg-muted/60").count(), 2, "{html}");

    fn custom() -> Element {
        rsx! {
            DataTable {
                items: people(),
                item_key: |p: &Person| p.id,
                loading: true,
                skeleton: rsx! { p { "custom skeleton" } },
                Col { id: "a", label: "A", render: |p: &Person| rsx! { "{p.name}" } }
            }
        }
    }
    let html = render(custom);
    assert!(html.contains("<p>custom skeleton</p>"), "{html}");
    assert!(!html.contains("animate-pulse"), "{html}");
}

#[test]
fn list_empty_default_and_custom() {
    fn app() -> Element {
        rsx! {
            ListEmpty {}
            ListEmpty { p { "custom" } }
        }
    }
    let html = render(app);
    assert!(html.contains("No items found"), "{html}");
    assert!(html.contains("<p>custom</p>"), "{html}");
}

// ── ListView (verifier-added, E5/E15 regression check) ─────────────────────

/// Poll a future to completion on the current thread (no executor dependency).
/// Returns `None` when the dom has no pending work.
fn block_on<F: Future>(fut: F) -> Option<F::Output> {
    use std::task::{Context, Poll, Waker};
    let mut fut = std::pin::pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..1_000 {
        if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
            return Some(v);
        }
    }
    None
}

/// Render, let the `use_resource` task resolve, re-render.
fn render_async(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..3 {
        if block_on(dom.wait_for_work()).is_none() {
            break;
        }
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

thread_local! {
    static LIST_EMPTY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn list_app() -> Element {
    let empty = LIST_EMPTY.with(|e| e.get());
    rsx! {
        components::ListView {
            title: rsx! { "My Title" },
            action: rsx! { button { "Create" } },
            per_page: 2u32,
            fetch: move |_page: u32, _per: u32| async move {
                Ok(components::ListPage {
                    items: if empty { vec![] } else { vec!["a".to_string(), "b".to_string()] },
                    has_more: false,
                })
            },
            render: |s: String| rsx! { span { "item-{s}" } },
            item_key: |s: &String| s.clone(),
            empty: rsx! { p { "nothing" } },
        }
    }
}

#[test]
fn list_view_ready_state_renders_title_action_and_items() {
    LIST_EMPTY.with(|e| e.set(false));
    let html = render_async(list_app);
    assert!(html.contains("<span>item-a</span>"), "{html}");
    assert!(html.contains("My Title"), "{html}");
    assert!(html.contains("<button>Create</button>"), "{html}");
}

#[test]
fn list_view_empty_state_keeps_title_and_action() {
    // HEAD rendered `ListEmpty` inside the title/action frame; the collapsed
    // `ListView` returns the bare `ListEmpty`, dropping both.
    LIST_EMPTY.with(|e| e.set(true));
    let html = render_async(list_app);
    assert!(html.contains("<p>nothing</p>"), "{html}");
    assert!(
        html.contains("My Title"),
        "title row lost in empty state: {html}"
    );
    assert!(
        html.contains("<button>Create</button>"),
        "action lost in empty state: {html}"
    );
}

// ── loading → ready transition (verifier-added, E7 remount check) ───────────

thread_local! {
    static LOADING_SIG: std::cell::Cell<Option<Signal<bool>>> = const { std::cell::Cell::new(None) };
}

fn loading_app() -> Element {
    let loading = use_signal(|| true);
    LOADING_SIG.with(|s| s.set(Some(loading)));
    rsx! {
        DataTable {
            items: people(),
            item_key: |p: &Person| p.id,
            loading: loading(),
            Col { id: "name", label: "Name", render: |p: &Person| rsx! { "{p.name}" } }
            Col { id: "id", label: "Id", render: |p: &Person| rsx! { "{p.id}" } }
        }
    }
}

#[test]
fn columns_survive_loading_to_ready_transition() {
    let mut dom = VirtualDom::new(loading_app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("animate-pulse"), "{html}");

    let sig = LOADING_SIG.with(|s| s.get()).expect("signal");
    dom.in_runtime(|| sig.clone().set(false));
    dom.render_immediate(&mut NoOpMutations);
    // Let any `Col` re-registration settle.
    while block_on(dom.wait_for_work()).is_some() {
        dom.render_immediate(&mut NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("animate-pulse"), "{html}");
    assert!(html.contains("Ada"), "{html}");
    assert!(
        html.contains(">Name</div>"),
        "header lost after loading→ready: {html}"
    );
    assert!(
        html.contains(">Id</div>"),
        "header lost after loading→ready: {html}"
    );
}

#[test]
fn columns_survive_ready_to_loading_to_ready() {
    let mut dom = VirtualDom::new(loading_app);
    dom.rebuild_in_place();
    let sig = LOADING_SIG.with(|s| s.get()).expect("signal");
    dom.in_runtime(|| sig.clone().set(false));
    dom.render_immediate(&mut NoOpMutations);
    while block_on(dom.wait_for_work()).is_some() {
        dom.render_immediate(&mut NoOpMutations);
    }
    dom.in_runtime(|| sig.clone().set(true));
    dom.render_immediate(&mut NoOpMutations);
    while block_on(dom.wait_for_work()).is_some() {
        dom.render_immediate(&mut NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    // Skeleton should mirror the registered column count (2), not the fallback 3.
    assert_eq!(html.matches("h-3 rounded bg-muted/60").count(), 2, "{html}");
    dom.in_runtime(|| sig.clone().set(false));
    dom.render_immediate(&mut NoOpMutations);
    while block_on(dom.wait_for_work()).is_some() {
        dom.render_immediate(&mut NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains(">Name</div>"), "header lost: {html}");
}
