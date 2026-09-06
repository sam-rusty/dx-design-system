use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::checkbox::CheckboxBase;
use crate::icon::{Icon, IconName};
use crate::link::Link;
use crate::list_view::{ListStatus, ListStatusKind};
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

/// Row / pagination class variants share one base literal; `concat!` keeps each
/// variant a `&'static str` (no per-render allocation).
macro_rules! row_class {
    () => {
        "group flex items-center gap-3 px-4 transition-colors"
    };
    ($suffix:literal) => {
        concat!(row_class!(), " ", $suffix)
    };
}
macro_rules! page_link_class {
    () => {
        "text-sm font-medium transition-colors px-3 py-1.5 rounded-lg"
    };
    ($suffix:literal) => {
        concat!(page_link_class!(), " ", $suffix)
    };
}

const ROW_BASE: &str = row_class!();
const ROW_LINK: &str = row_class!("cursor-pointer hover:bg-accent/30");
const ROW_SELECTED: &str = row_class!("bg-primary/5");

#[cfg(test)]
const PAGE_LINK_BASE: &str = page_link_class!();
const PAGE_LINK_ACTIVE: &str = page_link_class!("text-foreground hover:bg-accent cursor-pointer");
const PAGE_LINK_DISABLED: &str = page_link_class!("text-muted-foreground/40 pointer-events-none");

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

/// A column as registered by [`Col`]: `render` is the `ColRenderFn<T>` boxed
/// as `Any`, so the registry and the whole table render path compile once
/// instead of once per row type. `DataTable<T>` downcasts it back.
#[derive(Clone)]
struct ErasedColumn {
    key: &'static str,
    label: &'static str,
    sortable: bool,
    col_class: &'static str,
    render: Arc<dyn Any + Send + Sync>,
}

impl ErasedColumn {
    fn render<T: 'static>(&self, item: &T) -> Element {
        match self.render.downcast_ref::<ColRenderFn<T>>() {
            Some(f) => (f.0)(item),
            None => {
                debug_assert!(
                    false,
                    "Col {:?} registered into a DataTable of another row type",
                    self.key
                );
                rsx! {}
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ColumnRegistry {
    columns: Signal<Vec<ErasedColumn>>,
    /// Which `Col` instance currently owns each column id. When the table
    /// switches template (loading ↔ ready) Dioxus mounts the new `Col`
    /// instances before dropping the old ones, so an unowned `use_drop`
    /// would delete the freshly registered column.
    owners: Signal<HashMap<&'static str, usize>>,
}

/// `DataTable`'s half of the registry: owns the signals and provides them to
/// `Col` children.
#[inline(never)]
fn use_column_registry() -> ColumnRegistry {
    let columns = use_signal(Vec::new);
    let owners = use_signal(HashMap::new);
    use_context_provider(|| ColumnRegistry { columns, owners })
}

/// `Col`'s half of the registry: register on mount, track prop changes,
/// unregister on drop. `T`-free (see [`ErasedColumn`]) and never inlined so
/// the hook bodies exist once.
#[inline(never)]
fn use_col(
    id: &'static str,
    label: &'static str,
    sortable: bool,
    class: &'static str,
    render: Arc<dyn Any + Send + Sync>,
) {
    let mut registry = use_context::<ColumnRegistry>();
    // Per-instance token so `use_drop` only removes a registration this
    // instance still owns (see `ColumnRegistry::owners`).
    let token = use_hook(|| dioxus::core::current_scope_id().0);
    use_hook(move || {
        registry.owners.write_unchecked().insert(id, token);
        let new_col = ErasedColumn {
            key: id,
            label,
            sortable,
            col_class: class,
            render,
        };
        let mut cols = registry.columns.write();
        if let Some(pos) = cols.iter().position(|c| c.key == id) {
            cols[pos] = new_col;
        } else {
            cols.push(new_col);
        }
    });
    // Static fields track prop changes; the write is skipped when nothing differs so
    // the mount pass does not trigger a second table render.
    use_effect(use_reactive!(|(id, label, sortable, class)| {
        let changed = registry
            .columns
            .peek()
            .iter()
            .find(|c| c.key == id)
            .is_some_and(|c| c.label != label || c.sortable != sortable || c.col_class != class);
        if changed && let Some(c) = registry.columns.write().iter_mut().find(|c| c.key == id) {
            c.label = label;
            c.sortable = sortable;
            c.col_class = class;
        }
    }));
    use_drop(move || {
        let mut owners = registry.owners.write_unchecked();
        if owners.get(id) == Some(&token) {
            owners.remove(id);
            registry.columns.write().retain(|c| c.key != id);
        }
    });
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
///
/// `label`, `sortable` and `class` are re-applied whenever they change. `render` is
/// captured on mount (closures cannot be compared); read reactive state *inside* the
/// closure, or remount the column with a `key`, when its output depends on other state.
#[component]
pub fn Col<T: Clone + PartialEq + 'static>(
    id: &'static str,
    label: &'static str,
    #[props(default)] sortable: bool,
    #[props(default)] class: &'static str,
    #[props(into)] render: ColRenderFn<T>,
) -> Element {
    use_col(id, label, sortable, class, Arc::new(render));
    rsx! {}
}

// ── Selection logic (pure, unit-tested) ─────────────────────────────────────

/// Insertion-ordered selection so the `Vec<K>` handed to the parent is deterministic.
pub(crate) fn toggle_key<K: PartialEq + Copy>(selected: &mut Vec<K>, id: K) {
    if let Some(pos) = selected.iter().position(|k| *k == id) {
        selected.remove(pos);
    } else {
        selected.push(id);
    }
}

pub(crate) fn all_selected<K: PartialEq>(selected: &[K], ids: &[K]) -> bool {
    !ids.is_empty() && ids.iter().all(|id| selected.contains(id))
}

pub(crate) fn select_keys<K: PartialEq + Copy>(selected: &mut Vec<K>, ids: &[K]) {
    for id in ids {
        if !selected.contains(id) {
            selected.push(*id);
        }
    }
}

// ── Internal Selection State ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct SelectionState<K: 'static> {
    selected: Signal<Vec<K>>,
}

impl<K: Eq + Hash + Copy + Send + Sync + 'static> SelectionState<K> {
    /// Hooks run unconditionally (same count/order every render) so a parent may
    /// switch `selection` between `Some`/`None` without corrupting the hook list.
    fn new(external: Option<Signal<Vec<K>>>) -> Self {
        let selected = use_signal(Vec::new);

        // Sync internal → external Vec whenever selection changes.
        use_effect(move || {
            let ids: Vec<K> = selected.read().clone();
            if let Some(mut ext) = external
                && *ext.peek() != ids
            {
                ext.set(ids);
            }
        });

        Self { selected }
    }

    fn is_selected(&self, id: K) -> bool {
        self.selected.read().contains(&id)
    }

    fn toggle(mut self, id: K) {
        toggle_key(&mut self.selected.write(), id);
    }

    /// `ids` are the keys currently on the page (derived from `items` by the caller).
    fn all_on_page_selected(&self, ids: &[K]) -> bool {
        all_selected(&self.selected.read(), ids)
    }

    fn some_selected(&self) -> bool {
        !self.selected.read().is_empty()
    }

    fn select_all(mut self, ids: &[K]) {
        select_keys(&mut self.selected.write(), ids);
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
    /// Parent-owned signal that receives the current selection as a `Vec<K>`
    /// (insertion order).
    #[props(default)]
    selection: Option<Signal<Vec<K>>>,
    // ── Row Navigation ──
    #[props(default, into)] row_href: RowHrefProp<T>,
    // ── Per-row actions column ──
    #[props(default, into)] row_actions: RowActionsProp<T>,
    /// Content rendered at the start of each row (e.g. a per-row selection
    /// checkbox driven by an external context). Clicks inside the slot do not
    /// reach the row's own click handler (expand toggle).
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
    // Every hook runs before any early return so the hook list is stable across
    // `loading` / empty / populated renders.
    let selection_state = SelectionState::<K>::new(selection);
    // Provide context so `Col` children can register themselves on mount.
    let registry = use_column_registry();
    let expanded = use_signal(HashSet::<K>::new);

    let sel = selectable.then_some(selection_state);
    // Keys on this page, derived from `items` (no render-time signal write).
    let page_ids: Vec<K> = items.iter().map(|item| (item_key.0)(item)).collect();

    // Read registered columns; subscribes to changes when `Col` children register.
    let columns = registry.columns.read().clone();

    // Loading contract: show the skeleton while the parent reports `loading`. `Col`
    // children still mount so the skeleton can mirror the real column count.
    if loading {
        return render_loading(children, skeleton, skeleton_rows, columns.len());
    }

    let page_val = page.map(|p| p()).unwrap_or(1);

    // Empty state
    if items.is_empty() && page_val <= 1 {
        return render_empty(children, empty);
    }

    // Everything below that needs `T` or `K` is resolved here into
    // `ErasedRow`s; the markup itself is built once in `render_table`.
    let expandable = row_expand.0.is_some();
    let rows: Vec<ErasedRow> = items
        .iter()
        .map(|item| {
            let row_id = (item_key.0)(item);
            let is_open = expanded.read().contains(&row_id);
            ErasedRow {
                key: row_id.to_string(),
                href: row_href.0.as_ref().map(|rh| rh(item)),
                actions: row_actions.0.as_ref().map(|ra| ra(item)),
                left: row_left.0.as_ref().map(|rl| rl(row_id)),
                checkbox: sel.map(|s| RowCheckbox {
                    checked: s.is_selected(row_id),
                    toggle: Rc::new(move || s.toggle(row_id)),
                }),
                cells: columns
                    .iter()
                    .map(|c| (c.col_class, c.render(item)))
                    .collect(),
                is_open,
                // Panel closure invoked ONLY when expanded → lazy mount of any
                // inner component.
                panel: (expandable && is_open)
                    .then(|| row_expand.0.as_ref().map(|f| f(item)))
                    .flatten(),
                toggle_expand: expandable.then(|| {
                    Rc::new(move || {
                        let mut expanded = expanded;
                        let mut set = expanded.write();
                        if !set.remove(&row_id) {
                            set.insert(row_id);
                        }
                    }) as Rc<dyn Fn()>
                }),
            }
        })
        .collect();

    let select_all = sel.map(|s| {
        let all = s.all_on_page_selected(&page_ids);
        let ids = page_ids.clone();
        SelectAll {
            all,
            indeterminate: s.some_selected() && !all,
            select_all: Rc::new(move || s.select_all(&ids)),
            deselect_all: Rc::new(move || s.deselect_all()),
        }
    });

    render_table(TableView {
        class,
        children,
        header_left,
        select_all,
        columns,
        sort_key,
        sort_dir,
        sort_href,
        has_actions: row_actions.0.is_some(),
        rows,
        page: page.is_some(),
        page_val,
        has_more,
        page_href,
    })
}

// ── Type-erased render path ──────────────────────────────────────────────────
//
// `DataTable<T, K>` is instantiated once per row/key type an app uses. The
// markup below is the bulk of its code and needs neither `T` nor `K`, so it
// is built from erased inputs in non-generic, never-inlined functions.

struct RowCheckbox {
    checked: bool,
    toggle: Rc<dyn Fn()>,
}

struct ErasedRow {
    key: String,
    href: Option<NavigationTarget>,
    actions: Option<Element>,
    left: Option<Element>,
    checkbox: Option<RowCheckbox>,
    /// `(column class, rendered cell)` per registered column.
    cells: Vec<(&'static str, Element)>,
    is_open: bool,
    panel: Option<Element>,
    /// `Some` when rows expand on click (`row_expand` set).
    toggle_expand: Option<Rc<dyn Fn()>>,
}

struct SelectAll {
    all: bool,
    indeterminate: bool,
    select_all: Rc<dyn Fn()>,
    deselect_all: Rc<dyn Fn()>,
}

struct TableView {
    class: String,
    children: Element,
    header_left: Option<Element>,
    select_all: Option<SelectAll>,
    columns: Vec<ErasedColumn>,
    sort_key: Option<ReadSignal<String>>,
    sort_dir: Option<ReadSignal<SortDir>>,
    sort_href: SortHrefProp,
    has_actions: bool,
    rows: Vec<ErasedRow>,
    page: bool,
    page_val: u32,
    has_more: bool,
    page_href: PageHrefProp,
}

#[inline(never)]
fn render_loading(
    children: Element,
    skeleton: Option<Element>,
    skeleton_rows: usize,
    registered: usize,
) -> Element {
    match skeleton {
        Some(s) => rsx! {
            {children}
            {s}
        },
        None => rsx! {
            {children}
            DataTableSkeleton {
                rows: skeleton_rows,
                columns: if registered == 0 { 3 } else { registered },
            }
        },
    }
}

#[inline(never)]
fn render_empty(children: Element, empty: Option<Element>) -> Element {
    match empty {
        Some(e) => rsx! {
            {children}
            {e}
        },
        None => rsx! {
            {children}
            ListStatus {
                kind: ListStatusKind::Empty,
                class: "bg-card rounded-xl border border-border",
            }
        },
    }
}

fn render_header_cell(
    c: &ErasedColumn,
    sort_key: Option<ReadSignal<String>>,
    sort_dir: Option<ReadSignal<SortDir>>,
    sort_href: &SortHrefProp,
) -> Element {
    let col_class = c.col_class;
    let label = c.label;
    let col_key = c.key;

    if !c.sortable {
        return rsx! {
            div { class: "{col_class}", "{label}" }
        };
    }

    let current_key = sort_key.as_ref().map(|sk| sk()).unwrap_or_default();
    let current_dir = sort_dir.map(|sd| sd()).unwrap_or_default();
    let is_current = current_key == col_key;
    let new_dir = if is_current {
        current_dir.toggle()
    } else {
        SortDir::Asc
    };
    let sort_icon = if is_current {
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
    let href_val = sort_href
        .0
        .as_ref()
        .map(|f| f(col_key, new_dir))
        .unwrap_or_default();
    rsx! {
        Link {
            to: href_val,
            class: "{col_class} flex items-center gap-1 hover:text-foreground transition-colors cursor-pointer",
            "{label}"
            {sort_icon}
        }
    }
}

fn render_row(row: ErasedRow) -> Element {
    let ErasedRow {
        key: row_id,
        href,
        actions: actions_view,
        left: row_left_view,
        checkbox,
        cells,
        is_open,
        panel: panel_view,
        toggle_expand,
    } = row;
    let expandable = toggle_expand.is_some();
    let is_link = href.is_some() && !expandable;
    let is_selected = checkbox.as_ref().is_some_and(|c| c.checked);
    let row_class = if is_selected {
        ROW_SELECTED
    } else if is_link || expandable {
        ROW_LINK
    } else {
        ROW_BASE
    };

    let cells_view = match (is_link, href) {
        (true, Some(h)) => rsx! {
            Link {
                to: h,
                style: Some("display:contents".to_string()),
                for (class, cell) in cells {
                    div { class: "{class}", {cell} }
                }
            }
        },
        _ => rsx! {
            for (class, cell) in cells {
                div { class: "{class}", {cell} }
            }
        },
    };

    let checkbox_view = checkbox.map(|c| {
        let toggle = c.toggle;
        rsx! {
            div { class: "flex items-center justify-center w-6 shrink-0",
                CheckboxBase {
                    checked: Some(c.checked),
                    on_checked_change: move |_: bool| toggle(),
                }
            }
        }
    });

    // The chevron is a real button so the panel is reachable by keyboard and
    // announces its state; its click is stopped so the row handler does not
    // toggle a second time.
    let chevron_view = toggle_expand.clone().map(|toggle| {
        let cls = if is_open {
            "size-4 shrink-0 text-muted-foreground rotate-90 transition-transform"
        } else {
            "size-4 shrink-0 text-muted-foreground transition-transform"
        };
        rsx! {
            button {
                r#type: "button",
                class: "inline-flex shrink-0 cursor-pointer",
                "aria-expanded": is_open,
                "aria-label": "Toggle details",
                onclick: move |e: Event<MouseData>| {
                    e.stop_propagation();
                    toggle();
                },
                Icon { name: IconName::ChevronRight, class: "{cls}" }
            }
        }
    });

    let on_row_click = move |_: Event<MouseData>| {
        if let Some(toggle) = &toggle_expand {
            toggle();
        }
    };

    rsx! {
        div {
            key: "{row_id}",
            class: row_class,
            onclick: on_row_click,
            {chevron_view}
            // Slots are interactive (checkboxes, action buttons): their clicks
            // must not bubble into the row's expand/navigate handler.
            if let Some(v) = row_left_view {
                div { class: "contents", onclick: |e| e.stop_propagation(), {v} }
            }
            if let Some(v) = checkbox_view {
                div { class: "contents", onclick: |e| e.stop_propagation(), {v} }
            }
            {cells_view}
            if let Some(v) = actions_view {
                div { class: "contents", onclick: |e| e.stop_propagation(), {v} }
            }
        }
        if let Some(panel) = panel_view {
            div {
                key: "{row_id}-panel",
                {panel}
            }
        }
    }
}

#[inline(never)]
fn render_table(view: TableView) -> Element {
    let TableView {
        class,
        children,
        header_left,
        select_all,
        columns,
        sort_key,
        sort_dir,
        sort_href,
        has_actions,
        rows,
        page,
        page_val,
        has_more,
        page_href,
    } = view;

    let outer = merge(&[
        "bg-card flex flex-col rounded-xl border border-border overflow-hidden",
        &class,
    ]);

    let header_cells: Vec<Element> = columns
        .iter()
        .map(|c| render_header_cell(c, sort_key, sort_dir, &sort_href))
        .collect();

    let rows: Vec<Element> = rows.into_iter().map(render_row).collect();

    // Pagination: only when page signal, href builder, and nav is meaningful
    let pagination_view = match (page, page_href.0.as_ref()) {
        (true, Some(page_href_fn)) if page_val > 1 || has_more => {
            let prev_p = page_val.saturating_sub(1).max(1);
            let next_p = page_val + 1;
            let prev_href = page_href_fn(prev_p);
            let next_href = page_href_fn(next_p);
            let prev_disabled = page_val <= 1;
            let next_disabled = !has_more;
            rsx! {
                nav {
                    "aria-label": "Pagination",
                    class: "flex items-center justify-between px-4 py-3 border-t border-border bg-muted/20",
                    PageLink { href: prev_href, disabled: prev_disabled, "\u{2190} Previous" }
                    Text { variant: TextVariant::Secondary, size: TextSize::Small,
                        "Page {page_val}"
                    }
                    PageLink { href: next_href, disabled: next_disabled, "Next \u{2192}" }
                }
            }
        }
        _ => rsx! {},
    };

    let select_all_view = select_all.map(|s| {
        if s.indeterminate {
            let deselect_all = s.deselect_all;
            rsx! {
                div { class: "flex items-center justify-center w-6 shrink-0",
                    button {
                        r#type: "button",
                        "aria-label": "Clear selection",
                        class: "inline-flex items-center justify-center size-[18px] shrink-0 cursor-pointer select-none",
                        onclick: move |e| {
                            e.stop_propagation();
                            deselect_all();
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
            let SelectAll {
                all,
                select_all,
                deselect_all,
                ..
            } = s;
            rsx! {
                div { class: "flex items-center justify-center w-6 shrink-0",
                    CheckboxBase {
                        checked: Some(all),
                        on_checked_change: move |checked: bool| {
                            if checked { select_all() } else { deselect_all() }
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

/// Prev/next pagination link. A disabled side keeps its dimmed classes and is also
/// removed from the tab order and announced as disabled (E9).
#[component]
fn PageLink(href: String, disabled: bool, children: Element) -> Element {
    let class = if disabled {
        PAGE_LINK_DISABLED
    } else {
        PAGE_LINK_ACTIVE
    };
    rsx! {
        Link {
            to: href,
            class: "{class}",
            "aria-disabled": disabled.then_some("true"),
            tabindex: disabled.then_some("-1"),
            {children}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_dir_toggles() {
        assert_eq!(SortDir::Asc.toggle(), SortDir::Desc);
        assert_eq!(SortDir::Desc.toggle(), SortDir::Asc);
        assert_eq!(SortDir::default(), SortDir::Asc);
    }

    #[test]
    fn toggle_key_keeps_insertion_order() {
        let mut v = Vec::new();
        toggle_key(&mut v, 3);
        toggle_key(&mut v, 1);
        toggle_key(&mut v, 2);
        assert_eq!(v, vec![3, 1, 2]);
        toggle_key(&mut v, 1);
        assert_eq!(v, vec![3, 2]);
    }

    #[test]
    fn all_selected_requires_non_empty_page() {
        assert!(!all_selected::<u32>(&[], &[]));
        assert!(!all_selected(&[1, 2], &[]));
        assert!(all_selected(&[2, 1, 9], &[1, 2]));
        assert!(!all_selected(&[1], &[1, 2]));
    }

    #[test]
    fn select_keys_is_idempotent() {
        let mut v = vec![2];
        select_keys(&mut v, &[1, 2, 3]);
        assert_eq!(v, vec![2, 1, 3]);
        select_keys(&mut v, &[1, 2, 3]);
        assert_eq!(v, vec![2, 1, 3]);
    }

    #[test]
    fn row_and_page_classes_share_base() {
        assert!(ROW_LINK.starts_with(ROW_BASE));
        assert!(ROW_SELECTED.starts_with(ROW_BASE));
        assert!(PAGE_LINK_ACTIVE.starts_with(PAGE_LINK_BASE));
        assert!(PAGE_LINK_DISABLED.starts_with(PAGE_LINK_BASE));
    }
}
