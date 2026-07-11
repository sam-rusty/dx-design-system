//! Filter chips bar with an **Add filter** dropdown and an **All filters** modal.

mod helpers;

use std::marker::PhantomData;

use dioxus::prelude::*;
use helpers::{
    clause_for_key, clause_primary_val, clause_summary, collect_values, default_op_for_column,
    remove_clause, sync_draft_row, upsert_clause,
};
use utils::{ColumnType, EnumWidget, FilterClause, FilterColumns, FilterOp, FilterSet, FilterType};

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::checkbox::CheckboxBase;
use crate::date_picker::DatePickerBase;
use crate::dropdown::{DropdownMenu, DropdownMenuItem, DropdownMenuSeparator};
use crate::hooks::{use_escape_listener, use_outside_dismiss};
use crate::icon::{Icon, IconName};
use crate::input::{InputBase, InputType};
use crate::layout::{FlexGap, Row};
use crate::modal::{Modal, ModalSize};
use crate::radio::Radio;
use crate::select::{SelectBase, SelectOption, use_select_contexts};
use crate::text::{Text, TextSize, TextVariant};
use crate::title::{Title, TitleSize};

#[component]
pub fn MultiSearch<C>(
    filters: Signal<FilterSet>,
    #[props(default)] on_filters_applied: Option<EventHandler<()>>,
    #[props(default)] _column_type: PhantomData<C>,
) -> Element
where
    C: FilterColumns + Copy + PartialEq + 'static,
{
    let mut open_key: Signal<Option<String>> = use_signal(|| None);
    let mut modal_open: Signal<bool> = use_signal(|| false);
    let mut modal_draft: Signal<FilterSet> = use_signal(FilterSet::default);
    let mut modal_version: Signal<u32> = use_signal(|| 0);

    let fire_applied = move || {
        if let Some(cb) = on_filters_applied {
            cb.call(());
        }
    };

    let mut open_modal = move || {
        modal_draft.set(filters());
        *modal_version.write() += 1;
        modal_open.set(true);
    };

    let mut close_modal = move || {
        modal_open.set(false);
    };

    let mut apply_modal = {
        let mut filters = filters;
        move || {
            filters.set(modal_draft());
            modal_open.set(false);
            fire_applied();
        }
    };

    let mut reset_modal = move || {
        modal_draft.write().clauses.clear();
        modal_draft.write().filter_type = FilterType::And;
        *modal_version.write() += 1;
    };

    let clauses = filters().clauses.clone();

    rsx! {
        div { class: "relative z-10",
        Row { gap: FlexGap::Sm, class: "flex-nowrap min-h-11 items-center pb-1 w-full",
            // Dropdown outside `overflow-x-auto` avoids a paired vertical scrollbar on the row.
            div { class: "flex shrink-0 items-center gap-2",
                DropdownMenu {
                    trigger: rsx! {
                        Button {
                            size: ButtonSize::Sm,
                            variant: ButtonVariant::Bordered,
                            Icon { name: IconName::Filter },
                            "Search"
                        }
                    },
                    for col in C::all().iter().copied() {
                        {
                            let k_icon = col.key().to_string();
                            let k_click = k_icon.clone();
                            let label = col.label();
                            let has_clause = clause_for_key(&filters(), k_icon.as_str()).is_some();
                            let col_icon: Element = if has_clause {
                                rsx! {
                                    Icon {
                                        name: IconName::CheckCircle,
                                        class: "size-4 text-primary",
                                        stroke_width: 2.0,
                                    }
                                }
                            } else {
                                rsx! {
                                    Icon {
                                        name: IconName::Plus,
                                        class: "size-4 text-muted-foreground",
                                        stroke_width: 2.0,
                                    }
                                }
                            };
                            rsx! {
                                DropdownMenuItem {
                                    icon: col_icon,
                                    label: rsx! { "{label}" },
                                    on_click: move |_| {
                                        open_key.set(Some(k_click.clone()));
                                    },
                                }
                            }
                        }
                    }
                    DropdownMenuSeparator {}
                    DropdownMenuItem {
                        icon: rsx! {
                            Icon {
                                name: IconName::TableGrid,
                                class: "size-4 text-muted-foreground",
                                stroke_width: 2.0,
                            }
                        },
                        label: rsx! { "All filters…" },
                        on_click: move |_| open_modal(),
                    }
                }

                // Anchor for popovers opened from the dropdown when no clause exists yet.
                div { class: "relative shrink-0",
                    {
                        let maybe_key = open_key();
                        if let Some(k) = maybe_key {
                            if clause_for_key(&filters(), k.as_str()).is_none() {
                                rsx! {
                                    FilterPopoverBody::<C> {
                                        col_key: k,
                                        filters: filters,
                                        on_close: move |_| open_key.set(None),
                                        on_applied: move |_| fire_applied(),
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }

            div { class: "min-w-0 flex-1 overflow-x-auto",
                Row { gap: FlexGap::Sm, class: "flex-nowrap items-center",
            // Active filter chips
            for clause in clauses {
                {
                    let key = clause.col.clone();
                    let key_clear = key.clone();
                    let key_edit = key.clone();
                    let key_show = key.clone();
                    let col = C::all().iter().copied().find(|c| c.key() == key.as_str());
                    let label = col.map(|c| c.label()).unwrap_or("?");
                    let summary = clause_summary(clause.op, &clause.val);
                    let mut filters_mut = filters;
                    rsx! {
                        div { class: "relative shrink-0",
                            // Pill chip: [Label · value | ×]
                            div { class: "inline-flex items-center h-7 rounded-full border border-border bg-accent/40 text-sm shrink-0 overflow-hidden",
                                // Body — click to open edit popover
                                button {
                                    class: "pl-3 pr-2 flex items-center gap-1 h-full rounded-l-full hover:bg-accent transition-colors",
                                    onclick: move |_| {
                                        let k = key_edit.clone();
                                        if open_key().as_deref() == Some(k.as_str()) {
                                            open_key.set(None);
                                        } else {
                                            open_key.set(Some(k));
                                        }
                                    },
                                    span { class: "text-xs font-medium text-muted-foreground", "{label}" }
                                    span { class: "text-xs text-muted-foreground/60 select-none", "·" }
                                    span { class: "text-xs font-medium text-foreground truncate max-w-28", "{summary}" }
                                }
                                // Thin divider
                                div { class: "w-px h-3.5 bg-border/70 shrink-0" }
                                // Dismiss — 1-click clear
                                button {
                                    class: "px-2 h-full flex items-center text-muted-foreground/60 hover:text-destructive hover:bg-accent transition-colors rounded-r-full",
                                    onclick: move |_| {
                                        let k = key_clear.clone();
                                        remove_clause(&mut filters_mut.write(), k.as_str());
                                        if open_key().as_deref() == Some(k.as_str()) {
                                            open_key.set(None);
                                        }
                                        fire_applied();
                                    },
                                    Icon { name: IconName::X, class: "size-3", stroke_width: 2.5 }
                                }
                            }
                            // Edit popover anchored below this chip
                            if open_key().as_deref() == Some(key_show.as_str()) {
                                FilterPopoverBody::<C> {
                                    col_key: key.clone(),
                                    filters: filters,
                                    on_close: move |_| open_key.set(None),
                                    on_applied: move |_| fire_applied(),
                                }
                            }
                        }
                    }
                }
            }
                }
            }
        }

        if modal_open() {
            Modal {
                title: "All filters".to_string(),
                on_close: move |_| close_modal(),
                size: ModalSize::Xl,
                div { class: "px-6 py-4 overflow-y-auto max-h-[60vh] flex flex-col gap-0",
                    {
                        let _v = modal_version();
                        rsx! {
                            for col in C::all().iter().copied() {
                                ModalFilterRow::<C> {
                                    column: col,
                                    draft: modal_draft,
                                    version: modal_version,
                                }
                            }
                        }
                    }
                }
                div { class: "flex items-center justify-between gap-3 px-6 py-4 border-t border-border bg-muted/20",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| reset_modal(),
                        "Reset"
                    }
                    Row { gap: FlexGap::Sm,
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            onclick: move |_| close_modal(),
                            "Cancel"
                        }
                        Button {
                            size: ButtonSize::Sm,
                            onclick: move |_| apply_modal(),
                            "Apply filters"
                        }
                    }
                }
            }
        }
        }
    }
}

#[component]
fn FilterPopoverBody<C>(
    col_key: String,
    filters: Signal<FilterSet>,
    on_close: EventHandler<()>,
    on_applied: EventHandler<()>,
    #[props(default)] _column_type: PhantomData<C>,
) -> Element
where
    C: FilterColumns + Copy + PartialEq + 'static,
{
    let Some(col) = C::all()
        .iter()
        .copied()
        .find(|c| c.key() == col_key.as_str())
    else {
        return rsx! {
            div { class: "p-3 text-sm text-muted-foreground", "Unknown filter column" }
        };
    };

    let ct = col.col_type();
    let col_static_key = col.key();

    let existing_clause = clause_for_key(&filters(), col_static_key);

    let mut op_sig = use_signal(|| {
        existing_clause
            .as_ref()
            .map(|c| c.op)
            .unwrap_or_else(|| default_op_for_column(ct))
    });
    let mut v0 = use_signal(|| {
        existing_clause
            .as_ref()
            .map(|c| clause_primary_val(ct, c))
            .unwrap_or_default()
    });
    let mut v1 = use_signal(|| {
        existing_clause
            .and_then(|c| c.val.get(1).cloned())
            .unwrap_or_default()
    });

    // Sync signals when the underlying filter changes
    use_effect(move || {
        if let Some(cl) = clause_for_key(&filters(), col_static_key) {
            op_sig.set(cl.op);
            v0.set(clause_primary_val(ct, &cl));
            v1.set(cl.val.get(1).cloned().unwrap_or_default());
        }
    });

    let existing_for_clear = clause_for_key(&filters(), col_static_key);
    let mut filters_mut = filters;

    // Outside-pointer + Escape dismissal via the shared overlay hooks, replacing
    // the hand-rolled full-screen click-catcher (which blocked scroll/interaction
    // behind it and ignored focus-out / Esc).
    let mut root_el: Signal<Option<web_sys::Element>> = use_signal(|| None);
    use_outside_dismiss(root_el, use_signal(|| true).into(), move || {
        on_close.call(())
    });
    use_escape_listener(move || on_close.call(()));

    rsx! {
        div {
            class: "absolute z-[100] w-64 rounded-lg border border-border bg-card p-3 shadow-lg top-full mt-1",
            onmounted: move |e| {
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    root_el.set(Some(el.clone()));
                }
            },
            Title { size: TitleSize::H6, class: "mb-3 text-muted-foreground font-medium",
                {format!("Filter by {}", col.label())}
            }
            OpSelect { col_type: ct, op: op_sig }
            div { class: "mt-2",
                ValueEditors { col_type: ct, op: op_sig, v0: v0, v1: v1 }
            }
            Button {
                size: ButtonSize::Sm,
                class: "mt-4 w-full",
                onclick: move |_| {
                    let op = op_sig();
                    let val = collect_values(ct, op, v0(), v1());
                    if val.is_none() && !matches!(op, FilterOp::IsEmpty | FilterOp::IsNotEmpty) {
                        return;
                    }
                    let clause = FilterClause {
                        col: col_static_key.to_string(),
                        op,
                        val: val.unwrap_or_default(),
                    };
                    upsert_clause(&mut filters_mut.write(), clause);
                    on_applied.call(());
                    on_close.call(());
                },
                "Apply"
            }
            if existing_for_clear.is_some() {
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "mt-2 w-full text-xs text-muted-foreground hover:text-foreground",
                    onclick: move |_| {
                        remove_clause(&mut filters_mut.write(), col_static_key);
                        on_applied.call(());
                        on_close.call(());
                    },
                    "Clear"
                }
            }
        }
    }
}

/// Single-select dropdown backed by [`SelectBase`] (no native `<select>`).
///
/// `options` are `(value, label)` pairs; `on_after` fires after the value is
/// written so callers can sync drafts.
#[component]
fn ChoiceSelect(
    value: Signal<String>,
    options: Vec<(String, String)>,
    aria_label: String,
    #[props(default)] on_after: Option<EventHandler<()>>,
) -> Element {
    let read: ReadSignal<String> = value.into();
    use_select_contexts(
        read,
        EventHandler::new(move |v: String| {
            let mut value = value;
            value.set(v);
            if let Some(cb) = on_after {
                cb.call(());
            }
        }),
        false,
        0,
        false,
    );

    rsx! {
        SelectBase { aria_label: Some(aria_label),
            for (val , label) in options.iter() {
                SelectOption {
                    key: "{val}",
                    value: val.clone(),
                    label: label.clone(),
                }
            }
        }
    }
}

#[component]
fn OpSelect(
    col_type: ColumnType,
    op: Signal<FilterOp>,
    /// Fires after the operator is written (used to sync the modal draft).
    #[props(default)]
    on_after: Option<EventHandler<()>>,
    /// Render even for widgets that normally hide the operator (modal layout).
    #[props(default)]
    force_show: bool,
) -> Element {
    let op_string: ReadSignal<String> = use_memo(move || op().as_str().to_string()).into();
    let mut op_mut = op;
    use_select_contexts(
        op_string,
        EventHandler::new(move |v: String| {
            if let Ok(p) = v.parse::<FilterOp>() {
                op_mut.set(p);
            }
            if let Some(cb) = on_after {
                cb.call(());
            }
        }),
        false,
        0,
        false,
    );

    let show = force_show
        || !matches!(
            col_type,
            ColumnType::Enum {
                widget: EnumWidget::Checkbox | EnumWidget::Radio,
                ..
            } | ColumnType::Bool
        );
    if !show {
        return rsx! {};
    }

    rsx! {
        SelectBase { aria_label: Some("Operator".to_string()),
            for o in col_type.valid_ops().iter() {
                SelectOption {
                    key: "{(*o).as_ref()}",
                    value: (*o).as_ref().to_string(),
                    label: ColumnType::op_label(*o).to_string(),
                }
            }
        }
    }
}

#[component]
fn ValueEditors(
    col_type: ColumnType,
    op: Signal<FilterOp>,
    v0: Signal<String>,
    v1: Signal<String>,
) -> Element {
    let mut v0_mut = v0;
    let mut v1_mut = v1;
    let op_val = op();
    let is_empty_op = matches!(op_val, FilterOp::IsEmpty | FilterOp::IsNotEmpty);

    rsx! {
        div { "data-filter-focus": "",
            if is_empty_op {
                Text { variant: TextVariant::Secondary, size: TextSize::Small, "No value needed" }
            } else {
                {match col_type {
                    ColumnType::Bool => rsx! {
                        ChoiceSelect {
                            value: v0,
                            aria_label: "Value".to_string(),
                            options: vec![
                                (String::new(), "—".to_string()),
                                ("true".to_string(), "True".to_string()),
                                ("false".to_string(), "False".to_string()),
                            ],
                        }
                    },
                    ColumnType::Enum { options, widget } => match widget {
                        EnumWidget::Checkbox => rsx! {
                            div { class: "flex flex-col gap-2 max-h-36 overflow-y-auto",
                                for o in options.iter() {
                                    {
                                        let val = o.value.to_string();
                                        let val_chk = val.clone();
                                        let checked = serde_json::from_str::<Vec<String>>(&v0())
                                            .map(|xs| xs.iter().any(|x| x == val_chk.as_str()))
                                            .unwrap_or(false);
                                        rsx! {
                                            div { class: "flex items-center gap-2 text-sm",
                                                CheckboxBase {
                                                    checked,
                                                    on_change: move |on: bool| {
                                                        let mut cur: Vec<String> = serde_json::from_str(&v0_mut()).unwrap_or_default();
                                                        if on {
                                                            if !cur.iter().any(|x| x == val.as_str()) {
                                                                cur.push(val.clone());
                                                            }
                                                        } else {
                                                            cur.retain(|x| x != val.as_str());
                                                        }
                                                        v0_mut.set(serde_json::to_string(&cur).unwrap_or_else(|_| "[]".into()));
                                                    },
                                                }
                                                span { "{o.label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        EnumWidget::Radio => rsx! {
                            div { class: "flex flex-col gap-2",
                                for o in options.iter() {
                                    {
                                        let val = o.value.to_string();
                                        let is_checked = v0() == val;
                                        rsx! {
                                            div { class: "flex items-center gap-2 text-sm",
                                                Radio {
                                                    value: val.clone(),
                                                    checked: is_checked,
                                                    on_select: move |v: String| v0_mut.set(v),
                                                }
                                                span { "{o.label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        EnumWidget::Select => {
                            let mut opts = vec![(String::new(), "—".to_string())];
                            opts.extend(options.iter().map(|o| (o.value.to_string(), o.label.to_string())));
                            rsx! {
                                ChoiceSelect { value: v0, aria_label: "Value".to_string(), options: opts }
                            }
                        }
                    },
                    ColumnType::Date => rsx! {
                        DatePickerBase {
                            class: "mt-0 w-full",
                            value: ReadSignal::from(v0),
                            on_change: move |s| v0_mut.set(s),
                        }
                    },
                    ColumnType::Number => rsx! {
                        InputBase { r#type: InputType::Number, value: v0, class: "mt-0" }
                    },
                    _ => {
                        let input_type = if matches!(col_type, ColumnType::Email) {
                            InputType::Email
                        } else {
                            InputType::Text
                        };
                        rsx! {
                            InputBase { r#type: input_type, value: v0, class: "mt-0" }
                        }
                    },
                }}
                if op() == FilterOp::Between && matches!(col_type, ColumnType::Date) {
                    div { class: "mt-2",
                        DatePickerBase {
                            class: "mt-0 w-full",
                            value: ReadSignal::from(v1),
                            on_change: move |s| v1_mut.set(s),
                        }
                    }
                }
                if op() == FilterOp::Between && !matches!(col_type, ColumnType::Date) {
                    div { class: "mt-2",
                        InputBase { r#type: InputType::Text, value: v1, class: "mt-0" }
                    }
                }
            }
        }
    }
}

#[component]
fn ModalFilterRow<C>(column: C, draft: Signal<FilterSet>, version: Signal<u32>) -> Element
where
    C: FilterColumns + Copy + PartialEq + 'static,
{
    let ct = column.col_type();
    let col_key = column.key();

    let mut op_sig = use_signal(|| default_op_for_column(ct));
    let mut v0 = use_signal(String::new);
    let mut v1 = use_signal(String::new);

    use_effect(move || {
        let _ = version();
        if let Some(cl) = clause_for_key(&draft(), col_key) {
            op_sig.set(cl.op);
            v0.set(clause_primary_val(ct, &cl));
            v1.set(cl.val.get(1).cloned().unwrap_or_default());
        } else {
            op_sig.set(default_op_for_column(ct));
            v0.set(String::new());
            v1.set(String::new());
        }
    });

    let push_draft = move || {
        sync_draft_row(draft, col_key, ct, op_sig(), v0(), v1());
    };

    let is_empty_op = matches!(op_sig(), FilterOp::IsEmpty | FilterOp::IsNotEmpty);

    rsx! {
        // 3-column grid: label | operator | value
        div { class: "grid grid-cols-1 sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_minmax(0,2fr)] gap-3 items-start border-b border-border/60 py-4 first:pt-0 last:border-0",
            Text { class: "font-medium text-sm pt-2", "{column.label()}" }
            div { class: "min-w-0",
                OpSelect {
                    col_type: ct,
                    op: op_sig,
                    force_show: true,
                    on_after: move |_| push_draft(),
                }
            }
            div { class: "min-w-0",
                if is_empty_op {
                    Text { variant: TextVariant::Secondary, size: TextSize::Small, "—" }
                } else {
                    {match ct {
                        ColumnType::Bool => rsx! {
                            ChoiceSelect {
                                value: v0,
                                aria_label: "Value".to_string(),
                                on_after: move |_| push_draft(),
                                options: vec![
                                    (String::new(), "—".to_string()),
                                    ("true".to_string(), "True".to_string()),
                                    ("false".to_string(), "False".to_string()),
                                ],
                            }
                        },
                        ColumnType::Enum { options, widget } => match widget {
                            EnumWidget::Checkbox => rsx! {
                                div { class: "flex flex-col gap-2 max-h-28 overflow-y-auto",
                                    for o in options.iter() {
                                        {
                                            let val = o.value.to_string();
                                            let val_chk = val.clone();
                                            let checked = serde_json::from_str::<Vec<String>>(&v0())
                                                .map(|xs| xs.iter().any(|x| x == val_chk.as_str()))
                                                .unwrap_or(false);
                                            rsx! {
                                                div { class: "flex items-center gap-2 text-sm",
                                                    CheckboxBase {
                                                        checked,
                                                        on_change: move |on: bool| {
                                                            let mut cur: Vec<String> = serde_json::from_str(&v0()).unwrap_or_default();
                                                            if on {
                                                                if !cur.iter().any(|x| x == val.as_str()) {
                                                                    cur.push(val.clone());
                                                                }
                                                            } else {
                                                                cur.retain(|x| x != val.as_str());
                                                            }
                                                            v0.set(serde_json::to_string(&cur).unwrap_or_else(|_| "[]".into()));
                                                            push_draft();
                                                        },
                                                    }
                                                    span { "{o.label}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            EnumWidget::Radio => rsx! {
                                div { class: "flex flex-col gap-2",
                                    for o in options.iter() {
                                        {
                                            let val = o.value.to_string();
                                            let is_checked = v0() == val;
                                            rsx! {
                                                div { class: "flex items-center gap-2 text-sm",
                                                    Radio {
                                                        value: val.clone(),
                                                        checked: is_checked,
                                                        on_select: move |v: String| {
                                                            v0.set(v);
                                                            push_draft();
                                                        },
                                                    }
                                                    span { "{o.label}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            EnumWidget::Select => {
                                let mut opts = vec![(String::new(), "—".to_string())];
                                opts.extend(options.iter().map(|o| (o.value.to_string(), o.label.to_string())));
                                rsx! {
                                    ChoiceSelect {
                                        value: v0,
                                        aria_label: "Value".to_string(),
                                        on_after: move |_| push_draft(),
                                        options: opts,
                                    }
                                }
                            }
                        },
                        ColumnType::Date => rsx! {
                            DatePickerBase {
                                class: "mt-0 w-full",
                                value: ReadSignal::from(v0),
                                on_change: move |s| {
                                    v0.set(s);
                                    push_draft();
                                },
                            }
                        },
                        ColumnType::Number => rsx! {
                            InputBase {
                                r#type: InputType::Number,
                                value: v0,
                                class: "mt-0",
                                on_change: move |_| push_draft(),
                            }
                        },
                        _ => {
                            let input_type = if matches!(ct, ColumnType::Email) {
                                InputType::Email
                            } else {
                                InputType::Text
                            };
                            rsx! {
                                InputBase {
                                    r#type: input_type,
                                    value: v0,
                                    class: "mt-0",
                                    on_change: move |_| push_draft(),
                                }
                            }
                        },
                    }}
                    if op_sig() == FilterOp::Between && matches!(ct, ColumnType::Date) {
                        div { class: "mt-2",
                            DatePickerBase {
                                class: "mt-0 w-full",
                                value: ReadSignal::from(v1),
                                on_change: move |s| {
                                    v1.set(s);
                                    push_draft();
                                },
                            }
                        }
                    }
                    if op_sig() == FilterOp::Between && !matches!(ct, ColumnType::Date) {
                        div { class: "mt-2",
                            InputBase {
                                r#type: InputType::Text,
                                value: v1,
                                class: "mt-0",
                                on_change: move |_| push_draft(),
                            }
                        }
                    }
                }
            }
        }
    }
}
