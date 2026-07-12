use dioxus::prelude::*;
use ds_utils::classes;

use crate::field_name::Field;
use crate::form::{FormFieldFrame, use_field_binding};
use crate::hooks::{use_escape_listener, use_outside_dismiss, use_unique_id};
use crate::icon::{Icon, IconName};
use crate::input::FieldSize;

fn parse_multi_value(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

fn join_multi_value(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_default()
}

fn toggle_value(current: &str, val: &str) -> String {
    let mut vals = parse_multi_value(current);
    if let Some(pos) = vals.iter().position(|v| v == val) {
        vals.remove(pos);
    } else {
        vals.push(val.to_string());
    }
    join_multi_value(&vals)
}

/// One registered `<SelectOption>`, in declaration (mount) order. Keyed by a
/// stable per-instance id (not by `value`) so two options sharing a value each
/// keep a distinct registry slot, index, and DOM id rather than collapsing into
/// one — which would give them the same `option_id` and active-highlight state.
#[derive(Clone, PartialEq)]
struct OptionEntry {
    key: String,
    value: String,
    label: String,
}

#[derive(Clone, Copy)]
struct SelectContext {
    selected_value: ReadSignal<String>,
    is_open: Signal<bool>,
    on_select: EventHandler<String>,
    search_query: Signal<String>,
    /// Ordered registry of options (kept in sync via `register`/`unregister`).
    options: Signal<Vec<OptionEntry>>,
    multiple: bool,
    parsed_values: Memo<Vec<String>>,
    /// Stable id for the listbox, for `aria-controls`/option-id generation.
    listbox_id: ReadSignal<String>,
    /// Active descendant index (into `options`) for keyboard navigation.
    active: Signal<Option<usize>>,
    /// Indices into `options` that are currently visible (filter + `limit`).
    visible: Memo<Vec<usize>>,
    display_label_memo: Memo<String>,
    selected_items_memo: Memo<Vec<(String, String)>>,
}

impl SelectContext {
    /// Idempotent upsert of an option into the ordered registry, keyed by the
    /// option's stable per-instance id.
    fn register(&self, key: String, value: String, label: String) {
        let mut options = self.options;
        let needs = options.with(|o| match o.iter().find(|e| e.key == key) {
            Some(e) => e.value != value || e.label != label,
            None => true,
        });
        if needs {
            let mut o = options.write();
            if let Some(e) = o.iter_mut().find(|e| e.key == key) {
                e.value = value;
                e.label = label;
            } else {
                o.push(OptionEntry { key, value, label });
            }
        }
    }

    fn unregister(&self, key: &str) {
        let mut options = self.options;
        options.write().retain(|e| e.key != key);
    }

    fn index_of_key(&self, key: &str) -> Option<usize> {
        self.options.read().iter().position(|e| e.key == key)
    }

    fn is_value_selected(&self, val: &str) -> bool {
        if self.multiple {
            self.parsed_values.read().iter().any(|v| v == val)
        } else {
            self.selected_value.read().as_str() == val
        }
    }

    fn display_label(&self) -> String {
        self.display_label_memo.read().clone()
    }

    fn selected_items(&self) -> Vec<(String, String)> {
        self.selected_items_memo.read().clone()
    }

    fn option_id(&self, index: usize) -> String {
        format!("{}-opt-{}", self.listbox_id.peek(), index)
    }

    fn active_descendant_id(&self) -> Option<String> {
        (*self.active.read()).map(|i| self.option_id(i))
    }

    fn set_active(&self, index: usize) {
        let mut active = self.active;
        if *active.peek() != Some(index) {
            active.set(Some(index));
        }
    }

    /// Move the active descendant to the next/previous visible option (wrapping).
    fn move_active(&self, forward: bool) {
        let vis = self.visible.read();
        if vis.is_empty() {
            return;
        }
        let mut active = self.active;
        let next = match (*active.peek()).and_then(|cur| vis.iter().position(|&i| i == cur)) {
            Some(pos) => {
                if forward {
                    vis.get(pos + 1).copied().unwrap_or(vis[0])
                } else if pos == 0 {
                    *vis.last().unwrap()
                } else {
                    vis[pos - 1]
                }
            }
            None => {
                if forward {
                    vis[0]
                } else {
                    *vis.last().unwrap()
                }
            }
        };
        active.set(Some(next));
    }

    fn set_active_first(&self) {
        let first = self.visible.read().first().copied();
        self.active.clone().set(first);
    }

    fn set_active_last(&self) {
        let last = self.visible.read().last().copied();
        self.active.clone().set(last);
    }

    fn select_active(&self) {
        if let Some(idx) = *self.active.peek()
            && let Some(value) = self.options.read().get(idx).map(|e| e.value.clone())
        {
            self.on_select.call(value);
        }
    }

    /// Jump the active descendant to the first visible option whose label
    /// starts with `ch` (case-insensitive) — listbox typeahead.
    fn typeahead(&self, ch: char) {
        let ch = ch.to_ascii_lowercase();
        let vis = self.visible.read();
        let options = self.options.read();
        let found = vis.iter().copied().find(|&i| {
            options
                .get(i)
                .map(|e| e.label.to_lowercase().starts_with(ch))
                .unwrap_or(false)
        });
        if let Some(idx) = found {
            self.active.clone().set(Some(idx));
        }
    }
}

/// Provide Select contexts for a signal-controlled (non-form) [`SelectBase`].
///
/// Call this hook before rendering [`SelectBase`] in your component. The
/// `value` signal is read to display the selected label; `on_change` fires
/// when an option is picked.
pub fn use_select_contexts(
    value: ReadSignal<String>,
    on_change: Callback<String>,
    dynamic: bool,
    limit: usize,
    multiple: bool,
) {
    use_select_contexts_inner(value, Some(on_change), dynamic, limit, multiple, None);
}

fn use_select_contexts_inner(
    value: ReadSignal<String>,
    on_change: Option<Callback<String>>,
    dynamic: bool,
    limit: usize,
    multiple: bool,
    open: Option<Signal<bool>>,
) -> SelectContext {
    let default_open = use_signal(|| false);
    let mut is_open = open.unwrap_or(default_open);
    let mut search_query = use_signal(String::new);
    let options = use_signal(Vec::<OptionEntry>::new);
    let mut active = use_signal(|| None::<usize>);
    let listbox_id: ReadSignal<String> = use_unique_id().into();

    let parsed_values = use_memo(move || {
        let raw = value();
        if raw.is_empty() || !multiple {
            Vec::new()
        } else {
            parse_multi_value(&raw)
        }
    });

    // P1: the visible/limited set is derived once per change here — no
    // signal writes during per-option render (the old `is_visible` anti-pattern).
    let visible = use_memo(move || {
        let query = search_query().trim().to_lowercase();
        let opts = options.read();
        let mut out = Vec::new();
        for (i, e) in opts.iter().enumerate() {
            let matches = dynamic
                || query.is_empty()
                || e.value.to_lowercase().contains(&query)
                || e.label.to_lowercase().contains(&query);
            if matches {
                out.push(i);
                if limit > 0 && out.len() >= limit {
                    break;
                }
            }
        }
        out
    });

    let display_label_memo = use_memo(move || {
        if multiple {
            parsed_values.with(|vals| {
                if vals.is_empty() {
                    return String::new();
                }
                let opts = options.read();
                vals.iter()
                    .map(|v| {
                        opts.iter()
                            .find(|e| &e.value == v)
                            .map(|e| e.label.clone())
                            .unwrap_or_else(|| v.clone())
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        } else {
            let raw = value();
            if raw.is_empty() {
                return String::new();
            }
            options
                .read()
                .iter()
                .find(|e| e.value == raw)
                .map(|e| e.label.clone())
                .unwrap_or(raw)
        }
    });

    let selected_items_memo = use_memo(move || {
        parsed_values.with(|vals| {
            if vals.is_empty() {
                return Vec::new();
            }
            let opts = options.read();
            vals.iter()
                .map(|v| {
                    let label = opts
                        .iter()
                        .find(|e| &e.value == v)
                        .map(|e| e.label.clone())
                        .unwrap_or_else(|| v.clone());
                    (v.clone(), label)
                })
                .collect()
        })
    });

    let ctx = SelectContext {
        selected_value: value,
        is_open,
        on_select: EventHandler::new(move |val: String| {
            if multiple {
                let new_value = toggle_value(&value.peek(), &val);
                if let Some(cb) = on_change {
                    cb.call(new_value);
                }
                *search_query.write() = String::new();
            } else {
                if let Some(cb) = on_change {
                    cb.call(val);
                }
                *is_open.write() = false;
                *search_query.write() = String::new();
            }
        }),
        search_query,
        options,
        multiple,
        parsed_values,
        listbox_id,
        active,
        visible,
        display_label_memo,
        selected_items_memo,
    };

    // Keep the active descendant on a visible option: seed it on open, and
    // re-home it to the first match when filtering hides the current one.
    use_effect(move || {
        if !is_open() {
            if active.peek().is_some() {
                active.set(None);
            }
            return;
        }
        let vis = visible.read();
        let valid = (*active.peek()).map(|i| vis.contains(&i)).unwrap_or(false);
        if !valid {
            let preferred = if !multiple {
                let raw = value();
                options
                    .read()
                    .iter()
                    .position(|e| e.value == raw)
                    .filter(|i| vis.contains(i))
            } else {
                None
            };
            active.set(preferred.or_else(|| vis.first().copied()));
        }
    });

    use_context_provider(|| ctx);
    ctx
}

// ---------------------------------------------------------------------------
// Core <SelectBase> — renders the trigger + dropdown panel
// ---------------------------------------------------------------------------

#[component]
pub fn SelectBase(
    /// Extra classes merged onto the trigger.
    #[props(default)]
    class: String,
    /// Whether the select is disabled.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Show a search input inside the trigger.
    #[props(default)]
    searchable: bool,
    /// Allow multiple selections (tags).
    #[props(default)]
    multiple: bool,
    /// Options are provided dynamically (skip client-side filtering).
    #[props(default)]
    dynamic: bool,
    /// Cap on visible options (0 = unlimited).
    #[props(default)]
    limit: usize,
    /// Visual size (shared `FieldSize` scale).
    #[props(default)]
    size: FieldSize,
    /// `SelectOption` children.
    #[props(default)]
    children: Option<Element>,
    /// `aria-describedby` target (the field's error element id).
    #[props(default)]
    aria_describedby: Option<String>,
    /// `aria-invalid` value (form bindings set `"true"` on validation failure).
    #[props(default)]
    aria_invalid: Option<String>,
    /// Accessible label for the combobox.
    #[props(default)]
    aria_label: Option<String>,
) -> Element {
    let ctx = use_context::<SelectContext>();

    let is_multiple = ctx.multiple;
    let mut is_open = ctx.is_open;
    let mut search_query = ctx.search_query;
    let listbox_id = ctx.listbox_id;

    // Shared overlay dismissal: capture-phase pointer-down outside the root, and
    // a stacked Escape listener (so a Select open inside a modal closes only the
    // Select). Replaces the legacy `use_outside_click` (bubbling click + rAF hack).
    let mut root_el = use_signal(|| None::<web_sys::Element>);
    use_outside_dismiss(root_el, is_open.into(), move || {
        if is_open() {
            is_open.set(false);
        }
    });
    use_escape_listener(move || {
        if is_open() {
            is_open.set(false);
            search_query.set(String::new());
        }
    });

    let is_open_val = is_open();
    let search_val = search_query();
    let active_descendant = ctx.active_descendant_id();

    let show_search = searchable || dynamic;

    // Shared keyboard handler for the searchable text inputs (combobox role).
    let on_input_key = use_callback(move |ev: KeyboardEvent| match ev.key() {
        Key::ArrowDown => {
            ev.prevent_default();
            if !is_open() {
                is_open.set(true);
            }
            ctx.move_active(true);
        }
        Key::ArrowUp => {
            ev.prevent_default();
            if !is_open() {
                is_open.set(true);
            }
            ctx.move_active(false);
        }
        Key::Enter if is_open() => {
            ev.prevent_default();
            ctx.select_active();
        }
        _ => {}
    });

    // Shared keyboard handler for the non-searchable trigger buttons (adds
    // Home/End + typeahead, since there is no text field to type into).
    let on_button_key = use_callback(move |ev: KeyboardEvent| match ev.key() {
        Key::ArrowDown => {
            ev.prevent_default();
            if !is_open() {
                is_open.set(true);
            } else {
                ctx.move_active(true);
            }
        }
        Key::ArrowUp => {
            ev.prevent_default();
            if !is_open() {
                is_open.set(true);
            } else {
                ctx.move_active(false);
            }
        }
        Key::Home => {
            if is_open() {
                ev.prevent_default();
                ctx.set_active_first();
            }
        }
        Key::End => {
            if is_open() {
                ev.prevent_default();
                ctx.set_active_last();
            }
        }
        Key::Enter => {
            ev.prevent_default();
            if is_open() {
                ctx.select_active();
            } else {
                is_open.set(true);
            }
        }
        Key::Character(s) => {
            if let Some(c) = s.chars().next().filter(|c| c.is_alphanumeric()) {
                if !is_open() {
                    is_open.set(true);
                }
                ctx.typeahead(c);
            }
        }
        _ => {}
    });

    // Size tokens on the shared FieldSize scale; every class is a literal so
    // the Tailwind scanner still sees each one.
    let (sz_fixed, sz_pad, sz_text, sz_multi) = match size {
        FieldSize::Default => ("h-12", "px-4 py-2", "text-sm", "h-auto min-h-12"),
        FieldSize::Sm => ("h-8", "px-3 py-1", "text-xs", "h-auto min-h-8"),
        FieldSize::Xs => ("h-7", "px-2 py-0.5", "text-xs", "h-auto min-h-7"),
    };

    let trigger_class = if show_search {
        classes!(
            "flex w-full min-w-0 items-center rounded-lg border border-input bg-transparent pe-10 text-foreground transition-all duration-200 outline-none cursor-text",
            sz_fixed,
            sz_pad,
            sz_text,
            "focus-within:border-primary focus-within:ring-1 focus-within:ring-primary",
            "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 disabled:bg-muted/50",
            "aria-invalid:border-destructive aria-invalid:ring-destructive/20 focus-within:aria-invalid:border-destructive focus-within:aria-invalid:ring-1",
            if is_multiple { sz_multi } else { "" },
            if is_multiple { "flex-wrap gap-1" } else { "" },
            &class,
        )
    } else {
        classes!(
            "flex w-full min-w-0 items-center justify-between rounded-lg border border-input bg-transparent text-foreground transition-all duration-200 outline-none cursor-pointer",
            sz_fixed,
            sz_pad,
            sz_text,
            "focus:border-primary focus:ring-1 focus:ring-primary",
            "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 disabled:bg-muted/50",
            "aria-invalid:border-destructive aria-invalid:ring-destructive/20 focus:aria-invalid:border-destructive focus:aria-invalid:ring-1",
            if is_multiple { sz_multi } else { "" },
            if is_multiple {
                "flex-wrap gap-1 pe-10"
            } else {
                ""
            },
            &class,
        )
    };

    let chevron_class = if is_open_val {
        "size-4 shrink-0 text-muted-foreground transition-transform duration-200 rotate-180"
    } else {
        "size-4 shrink-0 text-muted-foreground transition-transform duration-200"
    };

    let dropdown_class = if is_open_val {
        "absolute left-0 mt-2 w-full origin-top rounded-xl bg-popover text-popover-foreground p-1 transition-all duration-150 z-50 border border-border shadow-lg opacity-100 scale-100 pointer-events-auto flex flex-col"
    } else {
        "absolute left-0 mt-2 w-full origin-top rounded-xl bg-popover text-popover-foreground p-1 transition-all duration-150 z-50 border border-border shadow-lg opacity-0 scale-95 pointer-events-none flex flex-col"
    };

    rsx! {
        div {
            "data-name": "SelectBase",
            class: "relative w-full",
            onmounted: move |e| {
                if let Some(el) = e.downcast::<web_sys::Element>() {
                    root_el.set(Some(el.clone()));
                }
            },
            if show_search && is_multiple {
                // Multi-select + searchable: tags + inline input
                div {
                    class: "{trigger_class}",
                    "data-state": if is_open_val { "open" } else { "closed" },
                    for (val , label) in ctx.selected_items() {
                        {
                            let remove_val = val.clone();
                            rsx! {
                                span {
                                    key: "{val}",
                                    class: "inline-flex items-center gap-1 rounded-md bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground",
                                    "{label}"
                                    button {
                                        r#type: "button",
                                        class: "inline-flex items-center justify-center rounded-full hover:bg-accent hover:text-accent-foreground cursor-pointer size-4",
                                        onclick: move |ev| {
                                            ev.stop_propagation();
                                            if !disabled() {
                                                ctx.on_select.call(remove_val.clone());
                                            }
                                        },
                                        Icon { name: IconName::X, class: "size-3" }
                                    }
                                }
                            }
                        }
                    }
                    input {
                        r#type: "text",
                        role: "combobox",
                        autocomplete: "off",
                        "aria-expanded": "{is_open_val}",
                        "aria-haspopup": "listbox",
                        "aria-controls": "{listbox_id}",
                        "aria-activedescendant": active_descendant.clone(),
                        class: "flex-1 min-w-[60px] bg-transparent text-sm text-foreground placeholder:text-muted-foreground outline-none",
                        disabled: disabled(),
                        "aria-label": aria_label.clone(),
                        value: "{search_val}",
                        oninput: move |ev| {
                            if !is_open() {
                                *is_open.write() = true;
                            }
                            *search_query.write() = ev.value();
                        },
                        onfocus: move |_| {
                            if !disabled() && !is_open() {
                                *is_open.write() = true;
                                *search_query.write() = String::new();
                            }
                        },
                        onkeydown: move |ev| on_input_key.call(ev),
                    }
                }
                div { class: "absolute inset-y-0 end-0 flex items-center pe-3 pointer-events-none",
                    Icon { name: IconName::ChevronDown, class: "{chevron_class}" }
                }
            } else if show_search {
                // Single-select + searchable: inline input
                input {
                    r#type: "text",
                    role: "combobox",
                    autocomplete: "off",
                    "aria-expanded": "{is_open_val}",
                    "aria-haspopup": "listbox",
                    "aria-controls": "{listbox_id}",
                    "aria-activedescendant": active_descendant.clone(),
                    class: "{trigger_class}",
                    disabled: disabled(),
                    "data-state": if is_open_val { "open" } else { "closed" },
                    "aria-describedby": aria_describedby,
                    "aria-invalid": aria_invalid,
                    "aria-label": aria_label.clone(),
                    value: if is_open_val { search_val.clone() } else { ctx.display_label() },
                    oninput: move |ev| {
                        if !is_open() {
                            *is_open.write() = true;
                        }
                        *search_query.write() = ev.value();
                    },
                    onfocus: move |_| {
                        if !disabled() && !is_open() {
                            *is_open.write() = true;
                            *search_query.write() = String::new();
                        }
                    },
                    onkeydown: move |ev| on_input_key.call(ev),
                }
                div { class: "absolute inset-y-0 end-0 flex items-center pe-3 pointer-events-none",
                    Icon { name: IconName::ChevronDown, class: "{chevron_class}" }
                }
            } else if is_multiple {
                // Multi-select + non-searchable: button with tags
                button {
                    r#type: "button",
                    role: "combobox",
                    "aria-expanded": "{is_open_val}",
                    "aria-haspopup": "listbox",
                    "aria-controls": "{listbox_id}",
                    "aria-activedescendant": active_descendant.clone(),
                    class: "{trigger_class}",
                    disabled: disabled(),
                    "data-state": if is_open_val { "open" } else { "closed" },
                    "aria-describedby": aria_describedby,
                    "aria-invalid": aria_invalid,
                    "aria-label": aria_label.clone(),
                    onclick: move |ev| {
                        ev.stop_propagation();
                        if !disabled() {
                            *is_open.write() ^= true;
                        }
                    },
                    onkeydown: move |ev| on_button_key.call(ev),
                    div { class: "flex flex-1 flex-wrap gap-1 items-center min-w-0",
                        {
                            let items = ctx.selected_items();
                            if items.is_empty() {
                                rsx! {
                                    span { "\u{00A0}" }
                                }
                            } else {
                                rsx! {
                                    for (val , label) in items {
                                        {
                                            let remove_val = val.clone();
                                            let remove_key = remove_val.clone();
                                            rsx! {
                                                span {
                                                    key: "{val}",
                                                    class: "inline-flex items-center gap-1 rounded-md bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground",
                                                    "{label}"
                                                    span {
                                                        role: "button",
                                                        tabindex: 0,
                                                        "aria-label": "Remove {label}",
                                                        class: "inline-flex items-center justify-center rounded-full hover:bg-accent hover:text-accent-foreground cursor-pointer size-4",
                                                        onclick: move |ev| {
                                                            ev.stop_propagation();
                                                            ev.prevent_default();
                                                            if !disabled() {
                                                                ctx.on_select.call(remove_val.clone());
                                                            }
                                                        },
                                                        onkeydown: move |ev: KeyboardEvent| {
                                                            let is_activate = ev.key() == Key::Enter
                                                                || ev.key() == Key::Character(" ".to_string());
                                                            if is_activate {
                                                                ev.stop_propagation();
                                                                ev.prevent_default();
                                                                if !disabled() {
                                                                    ctx.on_select.call(remove_key.clone());
                                                                }
                                                            }
                                                        },
                                                        Icon { name: IconName::X, class: "size-3" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "absolute inset-y-0 end-0 flex items-center pe-3 pointer-events-none",
                    Icon { name: IconName::ChevronDown, class: "{chevron_class}" }
                }
            } else {
                // Single-select + non-searchable: plain button
                button {
                    r#type: "button",
                    role: "combobox",
                    "aria-expanded": "{is_open_val}",
                    "aria-haspopup": "listbox",
                    "aria-controls": "{listbox_id}",
                    "aria-activedescendant": active_descendant.clone(),
                    class: "{trigger_class}",
                    disabled: disabled(),
                    "data-state": if is_open_val { "open" } else { "closed" },
                    "aria-describedby": aria_describedby,
                    "aria-invalid": aria_invalid,
                    "aria-label": aria_label.clone(),
                    onclick: move |ev| {
                        ev.stop_propagation();
                        if !disabled() {
                            *is_open.write() ^= true;
                        }
                    },
                    onkeydown: move |ev| on_button_key.call(ev),
                    span { class: "truncate",
                        {
                            let display = ctx.display_label();
                            if display.is_empty() { "\u{00A0}".to_string() } else { display }
                        }
                    }
                    Icon { name: IconName::ChevronDown, class: "{chevron_class}" }
                }
            }
            div {
                class: "{dropdown_class}",
                role: "listbox",
                id: "{listbox_id}",
                "aria-multiselectable": if is_multiple { "true" } else { "false" },
                "aria-label": aria_label.clone(),
                div { class: "flex flex-col gap-0.5 max-h-60 overflow-y-auto",
                    if let Some(c) = children {
                        {c}
                    }
                    if ctx.visible.read().is_empty() {
                        div { class: "px-3 py-6 text-sm text-muted-foreground text-center",
                            "No records found"
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// <SelectOption> — static option rendered as a child
// ---------------------------------------------------------------------------

#[component]
pub fn SelectOption(value: String, label: String, #[props(default)] class: String) -> Element {
    let ctx = use_context::<SelectContext>();

    // Stable per-instance key: distinguishes options even when two share a value.
    let key = use_unique_id();
    let my_key = key.peek().clone();

    // Idempotent register on render; remove from the ordered registry on unmount
    // (keeps the registry accurate for dynamic option lists).
    ctx.register(my_key.clone(), value.clone(), label.clone());
    use_drop({
        let my_key = my_key.clone();
        move || ctx.unregister(&my_key)
    });

    let index = ctx.index_of_key(&my_key).unwrap_or(0);
    let is_selected = ctx.is_value_selected(&value);
    let is_visible = ctx.visible.read().contains(&index);
    let is_active = *ctx.active.read() == Some(index);
    let option_id = ctx.option_id(index);

    let merged_class = classes!(
        "flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm font-medium text-foreground transition-colors hover:bg-accent hover:text-accent-foreground cursor-pointer text-left focus:outline-none focus:bg-accent focus:text-accent-foreground aria-selected:bg-accent aria-selected:text-accent-foreground",
        if is_active {
            "bg-accent text-accent-foreground"
        } else {
            ""
        },
        &class,
    );

    let aria_selected = is_selected.to_string();
    // Emit an explicit display value in both states. Binding `""` for the visible
    // case does not reliably clear a previously-applied inline `display:none`,
    // leaving widened searches (e.g. "Brita" -> "Brit") stuck hidden.
    let style = if is_visible {
        "display:flex"
    } else {
        "display:none"
    };
    let check_style = if is_selected { "" } else { "visibility:hidden" };

    let val_clone = value.clone();
    rsx! {
        button {
            r#type: "button",
            role: "option",
            id: "{option_id}",
            "aria-selected": "{aria_selected}",
            "data-active": is_active,
            class: "{merged_class}",
            "data-value": "{value}",
            style: "{style}",
            onmouseenter: move |_| ctx.set_active(index),
            onclick: move |ev| {
                ev.stop_propagation();
                ctx.on_select.call(val_clone.clone());
            },
            span { class: "truncate", "{label}" }
            Icon {
                name: IconName::Check,
                class: "size-4 shrink-0 text-primary",
                style: "{check_style}",
            }
        }
    }
}

// ---------------------------------------------------------------------------
// <SelectControl> — form-context binding for <SelectBase>
// ---------------------------------------------------------------------------

#[component]
pub(crate) fn SelectControl(
    #[props(default)] class: String,
    #[props(default)] searchable: bool,
    #[props(default)] multiple: bool,
    #[props(default)] limit: usize,
    #[props(default)] dynamic: bool,
    #[props(default)] size: FieldSize,
    open: Signal<bool>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let binding = use_field_binding();

    // Create contexts BEFORE evaluating children so SelectOption can find them.
    use_select_contexts_inner(
        binding.value.into(),
        Some(binding.on_commit),
        dynamic,
        limit,
        multiple,
        Some(open),
    );

    rsx! {
        SelectBase {
            class,
            disabled: ReadSignal::from(binding.disabled),
            searchable,
            multiple,
            dynamic,
            size,
            aria_describedby: binding.aria_describedby.clone(),
            aria_invalid: binding.aria_invalid(),
            aria_label,
            {children}
        }
    }
}

// ---------------------------------------------------------------------------
// <Select> — form-bound convenience (frame + SelectControl)
// ---------------------------------------------------------------------------

/// Props for [`Select`], the form-bound select.
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    /// The bound form field.
    #[props(into)]
    pub field: Field,
    /// Extra classes merged onto the field wrapper.
    #[props(default)]
    pub class: String,
    /// Show a search input inside the trigger.
    #[props(default)]
    pub searchable: bool,
    /// Allow multiple selections (tags).
    #[props(default)]
    pub multiple: bool,
    /// Cap on visible options (0 = unlimited).
    #[props(default)]
    pub limit: usize,
    /// Show the copy-to-clipboard action.
    #[props(default)]
    pub copyable: bool,
    /// Show the clear action.
    #[props(default)]
    pub clearable: bool,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// `(value, label)` pairs (e.g. a `FormOptions` derive's `OPTIONS`).
    #[props(default)]
    pub options: &'static [(&'static str, &'static str)],
    /// Help tooltip rendered inline after the label.
    #[props(default)]
    pub tooltip: Option<Element>,
    /// Extra `SelectOption` children rendered after `options`.
    #[props(default)]
    pub children: Option<Element>,
}

/// Form-bound select with stacked label and inline error.
pub fn Select(props: SelectProps) -> Element {
    let field_label = props.field.label;
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame {
            field: props.field,
            tooltip: props.tooltip,
            copyable: props.copyable,
            clearable: props.clearable,
            class: props.class,
            actions_class: "end-9",
            SelectControl {
                searchable: props.searchable,
                multiple: props.multiple,
                limit: props.limit,
                size: props.size,
                open,
                aria_label: field_label.to_string(),
                for (value , opt_label) in props.options.iter() {
                    SelectOption {
                        key: "{value}",
                        value: value.to_string(),
                        label: opt_label.to_string(),
                    }
                }
                if let Some(c) = props.children {
                    {c}
                }
            }
        }
    }
}
