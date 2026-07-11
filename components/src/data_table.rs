use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;

use dioxus::prelude::*;
use utils::format::merge;

use crate::checkbox::CheckboxBase;
use crate::icon::{Icon, IconName};
use crate::link::Link;
use crate::text::{Text, TextSize, TextVariant};

type RowHrefFn<T> = Arc<dyn Fn(&T) -> NavigationTarget + Send + Sync>;
type RowActionsFn<T> = Arc<dyn Fn(&T) -> Element + Send + Sync>;
type RowLeftFn<K> = Arc<dyn Fn(K) -> Element + Send + Sync>;

// ── PartialEq-safe fn prop wrappers ─────────────────────────────────────────

/// A wrapper around `Option<Arc<dyn Fn(&'static str, SortDir) -> String + Send + Sync>>`
/// that implements `PartialEq` by always returning `false` (fn equality is untracked).
#[derive(Clone, Default)]
pub struct SortHrefProp(pub Option<Arc<dyn Fn(&'static str, SortDir) -> String + Send + Sync>>);
impl PartialEq for SortHrefProp {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<F: Fn(&'static str, SortDir) -> String + Send + Sync + 'static> From<F> for SortHrefProp {
    fn from(f: F) -> Self {
        Self(Some(Arc::new(f)))
    }
}

/// A wrapper around `Option<Arc<dyn Fn(u32) -> String + Send + Sync>>`.
#[derive(Clone, Default)]
pub struct PageHrefProp(pub Option<Arc<dyn Fn(u32) -> String + Send + Sync>>);
impl PartialEq for PageHrefProp {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<F: Fn(u32) -> String + Send + Sync + 'static> From<F> for PageHrefProp {
    fn from(f: F) -> Self {
        Self(Some(Arc::new(f)))
    }
}

/// A wrapper around `Option<RowHrefFn<T>>` — closure returns `NavigationTarget` via `Into` (paths or `Routable` enums).
#[derive(Clone)]
pub struct RowHrefProp<T: 'static>(pub Option<RowHrefFn<T>>);
impl<T: 'static> Default for RowHrefProp<T> {
    fn default() -> Self {
        Self(None)
    }
}
impl<T: 'static> PartialEq for RowHrefProp<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<T: 'static, F, R> From<F> for RowHrefProp<T>
where
    F: Fn(&T) -> R + Send + Sync + 'static,
    R: Into<NavigationTarget>,
{
    fn from(f: F) -> Self {
        Self(Some(Arc::new(move |t| f(t).into())))
    }
}

/// A wrapper around `Option<RowActionsFn<T>>`.
#[derive(Clone)]
pub struct RowActionsProp<T: 'static>(pub Option<RowActionsFn<T>>);
impl<T: 'static> Default for RowActionsProp<T> {
    fn default() -> Self {
        Self(None)
    }
}
impl<T: 'static> PartialEq for RowActionsProp<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<T: 'static, F: Fn(&T) -> Element + Send + Sync + 'static> From<F> for RowActionsProp<T> {
    fn from(f: F) -> Self {
        Self(Some(Arc::new(f)))
    }
}

type RowExpandFn<T> = Arc<dyn Fn(&T) -> Element + Send + Sync>;

/// A wrapper around `Option<RowExpandFn<T>>`. When present, rows become expandable (toggle on
/// click) instead of navigable, and the closure renders the panel shown under an expanded row.
#[derive(Clone)]
pub struct RowExpandProp<T: 'static>(pub Option<RowExpandFn<T>>);
impl<T: 'static> Default for RowExpandProp<T> {
    fn default() -> Self {
        Self(None)
    }
}
impl<T: 'static> PartialEq for RowExpandProp<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<T: 'static, F> From<F> for RowExpandProp<T>
where
    F: Fn(&T) -> Element + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        Self(Some(Arc::new(f)))
    }
}

/// A wrapper around `Option<RowLeftFn<K>>`.
#[derive(Clone)]
pub struct RowLeftProp<K: 'static>(pub Option<RowLeftFn<K>>);
impl<K: 'static> Default for RowLeftProp<K> {
    fn default() -> Self {
        Self(None)
    }
}
impl<K: 'static> PartialEq for RowLeftProp<K> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<K: Copy + 'static, F: Fn(K) -> Element + Send + Sync + 'static> From<F> for RowLeftProp<K> {
    fn from(f: F) -> Self {
        Self(Some(Arc::new(f)))
    }
}

/// A wrapper for item_key functions — implements PartialEq by always returning false.
#[derive(Clone)]
pub struct ItemKeyProp<T: 'static, K: 'static>(pub Arc<dyn Fn(&T) -> K + Send + Sync>);
impl<T: 'static, K: 'static> PartialEq for ItemKeyProp<T, K> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<T: 'static, K: Copy + 'static, F: Fn(&T) -> K + Send + Sync + 'static> From<F>
    for ItemKeyProp<T, K>
{
    fn from(f: F) -> Self {
        Self(Arc::new(f))
    }
}

// ── Row class variants (state → &'static str, no per-render alloc) ───────────

const ROW_BASE: &str = "group flex items-center gap-3 px-4 transition-colors";
const ROW_LINK: &str =
    "group flex items-center gap-3 px-4 transition-colors cursor-pointer hover:bg-accent/30";
const ROW_SELECTED: &str = "group flex items-center gap-3 px-4 transition-colors bg-primary/5";

// ── Sort Direction ───────────────────────────────────────────────────────────

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggle(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

// ── Column Definition ────────────────────────────────────────────────────────

pub struct TableColumn<T: 'static> {
    key: &'static str,
    label: &'static str,
    sortable: bool,
    col_class: &'static str,
    render: Arc<dyn Fn(&T) -> Element + Send + Sync>,
}

impl<T: 'static> Clone for TableColumn<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            label: self.label,
            sortable: self.sortable,
            col_class: self.col_class,
            render: Arc::clone(&self.render),
        }
    }
}

impl<T: 'static> PartialEq for TableColumn<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T: 'static> TableColumn<T> {
    pub fn new(
        key: &'static str,
        label: &'static str,
        render: impl Fn(&T) -> Element + Send + Sync + 'static,
    ) -> Self {
        Self {
            key,
            label,
            sortable: false,
            col_class: "",
            render: Arc::new(render),
        }
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn class(mut self, class: &'static str) -> Self {
        self.col_class = class;
        self
    }
}

/// Convenience constructor — shorter than `TableColumn::new(…)`.
pub fn col<T: 'static>(
    key: &'static str,
    label: &'static str,
    render: impl Fn(&T) -> Element + Send + Sync + 'static,
) -> TableColumn<T> {
    TableColumn::new(key, label, render)
}

// ── Col render fn prop wrapper ──────────────────────────────────────────────

#[derive(Clone)]
pub struct ColRenderFn<T: 'static>(pub Arc<dyn Fn(&T) -> Element + Send + Sync>);
impl<T: 'static> PartialEq for ColRenderFn<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl<T: 'static, F: Fn(&T) -> Element + Send + Sync + 'static> From<F> for ColRenderFn<T> {
    fn from(f: F) -> Self {
        Self(Arc::new(f))
    }
}

// ── Column registry (context-shared between DataTable and Col children) ─────

struct ColumnRegistry<T: 'static> {
    columns: Signal<Vec<TableColumn<T>>>,
}
impl<T: 'static> Copy for ColumnRegistry<T> {}
impl<T: 'static> Clone for ColumnRegistry<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Column declaration for `DataTable`. Use as a child of `DataTable` to register a column.
///
/// `id` is the column identifier (used as the sort key when `sortable` is set). `label` is the
/// header text.
///
/// ```ignore
/// DataTable {
///     items, item_key: |p: &Person| p.id,
///     Col { id: "name", label: "Name", sortable: true,
///         render: |p: &Person| rsx! { "{p.name}" } }
/// }
/// ```
#[component]
pub fn Col<T: Clone + PartialEq + 'static>(
    id: &'static str,
    label: &'static str,
    #[props(default)] sortable: bool,
    #[props(default)] class: &'static str,
    #[props(into)] render: ColRenderFn<T>,
) -> Element {
    let mut registry = use_context::<ColumnRegistry<T>>();
    use_hook(move || {
        let new_col = TableColumn {
            key: id,
            label,
            sortable,
            col_class: class,
            render: render.0.clone(),
        };
        let mut cols = registry.columns.write();
        if let Some(pos) = cols.iter().position(|c| c.key == id) {
            cols[pos] = new_col;
        } else {
            cols.push(new_col);
        }
    });
    use_drop(move || {
        registry.columns.write().retain(|c| c.key != id);
    });
    rsx! {}
}

// ── Internal Selection State ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct SelectionState<K: 'static> {
    selected: Signal<HashSet<K>>,
    page_ids: Signal<Vec<K>>,
}

impl<K: Eq + Hash + Copy + Send + Sync + 'static> SelectionState<K> {
    fn new(external: Option<Signal<Vec<K>>>) -> Self {
        let selected = use_signal(HashSet::new);
        let page_ids = use_signal(Vec::new);

        // Sync internal HashSet → external Vec whenever selection changes
        if let Some(mut ext) = external {
            use_effect(move || {
                let ids: Vec<K> = selected.read().iter().copied().collect();
                ext.set(ids);
            });
        }

        Self { selected, page_ids }
    }

    fn is_selected(&self, id: K) -> bool {
        self.selected.read().contains(&id)
    }

    fn toggle(mut self, id: K) {
        let mut s = self.selected.write();
        if !s.remove(&id) {
            s.insert(id);
        }
    }

    fn all_on_page_selected(&self) -> bool {
        let page = self.page_ids.read();
        if page.is_empty() {
            return false;
        }
        let s = self.selected.read();
        page.iter().all(|id| s.contains(id))
    }

    fn some_selected(&self) -> bool {
        !self.selected.read().is_empty()
    }

    fn select_all(mut self) {
        let ids = self.page_ids.peek().clone();
        let mut s = self.selected.write();
        for id in ids {
            s.insert(id);
        }
    }

    fn deselect_all(mut self) {
        self.selected.write().clear();
    }
}

// ── DataTable Component ──────────────────────────────────────────────────────

#[component]
pub fn DataTable<T, K>(
    items: Vec<T>,
    #[props(into)] item_key: ItemKeyProp<T, K>,
    /// `Col` children — see [`Col`].
    children: Element,
    // ── Sorting ──
    #[props(default)] sort_key: Option<ReadSignal<String>>,
    #[props(default)] sort_dir: Option<ReadSignal<SortDir>>,
    /// `fn(column_key, new_direction) -> href` — rendered as `<a>` on sortable headers.
    #[props(default, into)]
    sort_href: SortHrefProp,
    // ── Pagination ──
    /// Current page (1-based). When `None`, the pagination footer is never shown.
    #[props(default)]
    page: Option<ReadSignal<u32>>,
    #[props(default)] has_more: bool,
    /// `fn(page_number) -> href` for prev/next links. With `page: Some(_)`, pagination
    /// renders when `page > 1` or `has_more`. If `page` is `Some` but this is `None`,
    /// the footer is omitted (no panic).
    #[props(default, into)]
    page_href: PageHrefProp,
    // ── Row Selection ──
    #[props(default)] selectable: bool,
    /// Parent-owned signal that receives the current selection as a `Vec<K>`.
    #[props(default)]
    selection: Option<Signal<Vec<K>>>,
    // ── Row Navigation ──
    #[props(default, into)] row_href: RowHrefProp<T>,
    // ── Per-row actions column ──
    #[props(default, into)] row_actions: RowActionsProp<T>,
    /// Content rendered at the start of each row (e.g. a per-row selection
    /// checkbox driven by an external context). Clicks are automatically
    /// stopped from propagating to the row link via an undelegated listener.
    #[props(default, into)]
    row_left: RowLeftProp<K>,
    /// When set, rows expand on click into a panel rendering this closure's output, instead of
    /// navigating. Mutually exclusive with `row_href` per row (expand wins).
    #[props(default, into)]
    row_expand: RowExpandProp<T>,
    // ── Slots ──
    /// Content rendered at the start of the header row (e.g. a select-all checkbox).
    #[props(default)]
    header_left: Option<Element>,
    // ── Appearance ──
    #[props(default)] class: String,
    #[props(default)] empty: Option<Element>,
    /// Render the loading skeleton instead of rows/empty state. Pass a custom
    /// `skeleton` to override the built-in [`DataTableSkeleton`].
    #[props(default)]
    loading: bool,
    #[props(default)] skeleton: Option<Element>,
    #[props(default = 8)] skeleton_rows: usize,
) -> Element
where
    T: Clone + Send + Sync + PartialEq + 'static,
    K: Eq + Hash + Copy + Send + Sync + Display + 'static,
{
    // Hooks must run unconditionally; gate *use* of the state on `selectable`.
    let selection_state = SelectionState::<K>::new(selection);
    let sel = selectable.then_some(selection_state);

    // Populate page_ids so select-all knows which rows exist
    if let Some(mut sel) = sel {
        let key_fn = item_key.0.clone();
        let ids: Vec<K> = items.iter().map(|item| key_fn(item)).collect();
        sel.page_ids.write().clone_from(&ids);
    }

    // Loading contract: show the skeleton while the parent reports `loading`.
    if loading {
        return match skeleton {
            Some(s) => s,
            None => rsx! {
                DataTableSkeleton { rows: skeleton_rows }
            },
        };
    }

    // Provide context so `Col` children can register themselves on mount.
    let columns_sig = use_signal(Vec::<TableColumn<T>>::new);
    let registry = use_context_provider(|| ColumnRegistry::<T> {
        columns: columns_sig,
    });
    let mut expanded = use_signal(HashSet::<K>::new);

    let page_val = page.map(|p| p()).unwrap_or(1);
    let has_actions = row_actions.0.is_some();

    // Empty state
    if items.is_empty() && page_val <= 1 {
        return match empty {
            Some(e) => e,
            None => rsx! {
                div { class: "bg-card flex flex-col items-center justify-center rounded-xl border border-border py-12",
                    Text { variant: TextVariant::Secondary, size: TextSize::Small,
                        "No items found"
                    }
                }
            },
        };
    }

    // Wrap shared closures in Arc so they're Clone-able inside views
    let sort_href_arc = sort_href.0;
    let row_href_arc = row_href.0;
    let row_actions_arc = row_actions.0;
    let row_left_arc = row_left.0;
    let row_expand_arc = row_expand.0;
    // Read registered columns; subscribes to changes when `Col` children register.
    let columns_arc = Arc::new(registry.columns.read().clone());

    let outer = merge(&[
        "bg-card flex flex-col rounded-xl border border-border overflow-hidden",
        &class,
    ]);

    // Pre-compute header cells
    let header_cells: Vec<Element> = columns_arc.iter().map(|c| {
        let col_class = c.col_class;
        let label = c.label;
        let col_key = c.key;
        let sortable = c.sortable;

        if sortable {
            let current_key = sort_key.as_ref().map(|sk| sk()).unwrap_or_default();
            let current_dir = sort_dir.map(|sd| sd()).unwrap_or_default();
            let new_dir = if current_key == col_key {
                current_dir.toggle()
            } else {
                SortDir::Asc
            };
            let sort_icon = if current_key == col_key {
                match current_dir {
                    SortDir::Asc => rsx! {
                        Icon { name: IconName::ChevronUp, class: "size-3 text-primary" }
                    },
                    SortDir::Desc => rsx! {
                        Icon { name: IconName::ChevronDown, class: "size-3 text-primary" }
                    },
                }
            } else {
                rsx! {}
            };
            let href_val = sort_href_arc.as_ref().map(|f| f(col_key, new_dir)).unwrap_or_default();
            rsx! {
                Link {
                    to: href_val,
                    class: "{col_class} flex items-center gap-1 hover:text-foreground transition-colors cursor-pointer",
                    "{label}"
                    {sort_icon}
                }
            }
        } else {
            rsx! {
                div { class: "{col_class}", "{label}" }
            }
        }
    }).collect();

    // Pre-compute rows
    let rows: Vec<Element> = items
        .iter()
        .map(|item| {
            let row_id = (item_key.0)(item);
            let href = row_href_arc.as_ref().map(|rh| rh(item));
            let actions_view = row_actions_arc.as_ref().map(|ra| ra(item));
            let row_left_view = row_left_arc.as_ref().map(|rl| rl(row_id));
            let expandable = row_expand_arc.is_some();
            let is_open = expanded.read().contains(&row_id);
            let is_link = href.is_some() && !expandable;

            let is_selected = sel.map(|s| s.is_selected(row_id)).unwrap_or(false);
            let row_class = if is_selected {
                ROW_SELECTED
            } else if is_link || expandable {
                ROW_LINK
            } else {
                ROW_BASE
            };

            let cells: Vec<Element> = columns_arc
                .iter()
                .map(|c| {
                    let cell = (c.render)(item);
                    let class = c.col_class;
                    rsx! { div { class: "{class}", {cell} } }
                })
                .collect();

            let cells_view = match (is_link, href) {
                (true, Some(h)) => rsx! {
                    Link {
                        to: h,
                        style: Some("display:contents".to_string()),
                        for cell in cells { {cell} }
                    }
                },
                _ => rsx! {
                    for cell in cells { {cell} }
                },
            };

            let checkbox_view = sel.map(|s| {
                let checked = s.is_selected(row_id);
                rsx! {
                    div { class: "flex items-center justify-center w-6 shrink-0",
                        CheckboxBase {
                            checked: checked,
                            on_change: move |_: bool| {
                                s.toggle(row_id);
                            },
                        }
                    }
                }
            });

            let chevron_view = expandable.then(|| {
                let cls = if is_open {
                    "size-4 shrink-0 text-muted-foreground rotate-90 transition-transform"
                } else {
                    "size-4 shrink-0 text-muted-foreground transition-transform"
                };
                rsx! { Icon { name: IconName::ChevronRight, class: "{cls}" } }
            });

            // Panel closure invoked ONLY when expanded → lazy mount of any inner component.
            let panel_view = (expandable && is_open)
                .then(|| row_expand_arc.as_ref().map(|f| f(item)))
                .flatten();

            let on_row_click = move |_: Event<MouseData>| {
                if expandable {
                    let mut set = expanded.write();
                    if set.contains(&row_id) {
                        set.remove(&row_id);
                    } else {
                        set.insert(row_id);
                    }
                }
            };

            rsx! {
                div {
                    key: "{row_id}",
                    class: row_class,
                    onclick: on_row_click,
                    {chevron_view}
                    {row_left_view}
                    {checkbox_view}
                    {cells_view}
                    {actions_view}
                }
                if let Some(panel) = panel_view {
                    div {
                        key: "{row_id}-panel",
                        {panel}
                    }
                }
            }
        })
        .collect();

    // Pagination: only when page signal, href builder, and nav is meaningful
    let pagination_view = match (&page, page_href.0.as_ref()) {
        (Some(_), Some(page_href_fn)) if page_val > 1 || has_more => {
            let prev_p = page_val.saturating_sub(1).max(1);
            let next_p = page_val + 1;
            let prev_href = page_href_fn(prev_p);
            let next_href = page_href_fn(next_p);
            let prev_class = if page_val <= 1 {
                "text-sm font-medium transition-colors px-3 py-1.5 rounded-lg text-muted-foreground/40 pointer-events-none"
            } else {
                "text-sm font-medium transition-colors px-3 py-1.5 rounded-lg text-foreground hover:bg-accent cursor-pointer"
            };
            let next_class = if has_more {
                "text-sm font-medium transition-colors px-3 py-1.5 rounded-lg text-foreground hover:bg-accent cursor-pointer"
            } else {
                "text-sm font-medium transition-colors px-3 py-1.5 rounded-lg text-muted-foreground/40 pointer-events-none"
            };
            rsx! {
                div { class: "flex items-center justify-between px-4 py-3 border-t border-border bg-muted/20",
                    Link { to: prev_href, class: "{prev_class}", "\u{2190} Previous" }
                    Text { variant: TextVariant::Secondary, size: TextSize::Small,
                        "Page {page_val}"
                    }
                    Link { to: next_href, class: "{next_class}", "Next \u{2192}" }
                }
            }
        }
        _ => rsx! {},
    };

    let select_all_view = sel.map(|s| {
        let is_indeterminate = s.some_selected() && !s.all_on_page_selected();
        let is_checked = s.all_on_page_selected();
        if is_indeterminate {
            rsx! {
                div { class: "flex items-center justify-center w-6 shrink-0",
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
                        onclick: move |e| {
                            e.stop_propagation();
                            s.deselect_all();
                        },
                        span { class: "flex items-center justify-center size-[18px] rounded-sm bg-primary border-2 border-primary transition-all duration-150",
                            Icon {
                                name: IconName::Minus,
                                stroke_width: 3.0,
                                class: "size-3 text-primary-foreground",
                            }
                        }
                    }
                }
            }
        } else {
            rsx! {
                div { class: "flex items-center justify-center w-6 shrink-0",
                    CheckboxBase {
                        checked: is_checked,
                        on_change: move |checked: bool| {
                            if checked { s.select_all() } else { s.deselect_all() }
                        },
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "{outer}",
            // Mount `Col` children so they register into the registry context.
            // `Col` itself renders an empty fragment; this is non-visual.
            {children}
            // ── Header ──
            div { class: "flex items-center gap-3 px-4 py-2.5 bg-muted/30 border-b border-border text-xs font-medium text-muted-foreground select-none",
                {header_left}
                {select_all_view}
                for cell in header_cells { {cell} }
                if has_actions {
                    div { class: "w-10 shrink-0" }
                }
            }

            // ── Rows ──
            div { class: "divide-y divide-border/50",
                for row in rows { {row} }
            }

            // ── Pagination ──
            {pagination_view}
        }
    }
}

// ── Skeleton (standalone, for use in loading fallback) ────────────────────

#[component]
pub fn DataTableSkeleton(
    /// Number of columns to show shimmer blocks for
    #[props(default = 3)]
    columns: usize,
    #[props(default = 8)] rows: usize,
    #[props(default)] class: String,
) -> Element {
    let outer = merge(&[
        "flex flex-col rounded-xl border border-border overflow-hidden animate-pulse",
        &class,
    ]);
    rsx! {
        div { class: "{outer}",
            div { class: "flex items-center gap-3 px-4 py-2.5 bg-muted/30 border-b border-border",
                for i in 0..columns {
                    {
                        let w = if i == 0 { "flex-1" } else { "w-24 hidden sm:block" };
                        rsx! { div { class: "{w} h-3 rounded bg-muted/60" } }
                    }
                }
            }
            div { class: "divide-y divide-border/50",
                for _ in 0..rows {
                    div { class: "flex items-center gap-3 px-4 py-3",
                        div { class: "size-8 rounded-full bg-muted/60 shrink-0" }
                        div { class: "flex-1 min-w-0 flex flex-col gap-1.5",
                            div { class: "h-3.5 w-36 rounded bg-muted/60" }
                            div { class: "h-3 w-24 rounded bg-muted/40" }
                        }
                        for _ in 1..columns {
                            div { class: "w-24 h-3 rounded bg-muted/40 hidden sm:block" }
                        }
                    }
                }
            }
        }
    }
}
