use dioxus::prelude::*;

use crate::hooks::{
    FocusState, use_controlled, use_focus_control_disabled, use_focus_entry_disabled,
    use_focus_provider, use_unique_id,
};

#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub key: &'static str,
    pub label: Element,
    pub children: Element,
    pub disabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TabType {
    #[default]
    Card,
    Line,
}

impl TabType {
    fn trigger_class(self, active: bool, disabled: bool) -> &'static str {
        match (self, active, disabled) {
            (Self::Card, true, _) => {
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors cursor-pointer select-none bg-card text-foreground shadow-sm"
            }
            (Self::Card, false, true) => {
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors cursor-not-allowed select-none text-muted-foreground opacity-50"
            }
            (Self::Card, false, false) => {
                "px-3 py-1.5 rounded-md text-sm font-medium transition-colors cursor-pointer select-none text-muted-foreground hover:text-foreground"
            }
            (Self::Line, true, _) => {
                "px-4 py-2 text-sm font-medium border-b-2 border-primary text-foreground -mb-px transition-colors cursor-pointer select-none"
            }
            (Self::Line, false, true) => {
                "px-4 py-2 text-sm font-medium text-muted-foreground opacity-50 cursor-not-allowed select-none"
            }
            (Self::Line, false, false) => {
                "px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer select-none"
            }
        }
    }
}

#[component]
pub fn Tabs(
    items: Vec<TabItem>,
    #[props(default)] tab_type: TabType,
    #[props(default)] active_key: Option<Signal<&'static str>>,
    #[props(default)] on_change: Option<EventHandler<&'static str>>,
    #[props(default)] default_active_key: Option<&'static str>,
) -> Element {
    let first_key = items.first().map(|i| i.key).unwrap_or("");
    let initial = default_active_key.unwrap_or(first_key);

    let prop: ReadSignal<Option<&'static str>> = use_memo(move || active_key.map(|s| s())).into();
    let on_change_cb = use_callback(move |key: &'static str| {
        if let Some(cb) = &on_change {
            cb.call(key);
        }
    });
    let (current, set_current) = use_controlled(prop, initial, on_change_cb);
    let current_key = current();

    use_focus_provider(use_signal(|| true).into());
    let base_id = use_unique_id();
    let base = base_id.peek().clone();

    let strip_class = match tab_type {
        TabType::Card => "inline-flex p-0.5 rounded-lg bg-muted/40 border border-border w-fit mb-4",
        TabType::Line => "flex border-b border-border mb-4",
    };

    let active = items.iter().find(|i| i.key == current_key).cloned();

    rsx! {
        div {
            div { class: strip_class, role: "tablist",
                for (index , item) in items.iter().enumerate() {
                    TabTrigger {
                        key: "{item.key}",
                        index,
                        tab_id: format!("{base}-tab-{}", item.key),
                        panel_id: format!("{base}-panel-{}", item.key),
                        label: item.label.clone(),
                        is_active: current_key == item.key,
                        disabled: item.disabled,
                        class: tab_type.trigger_class(current_key == item.key, item.disabled),
                        on_activate: {
                            let key = item.key;
                            move |_| set_current.call(key)
                        },
                    }
                }
            }
            if let Some(active) = active {
                div {
                    id: "{base}-panel-{active.key}",
                    role: "tabpanel",
                    tabindex: "0",
                    "aria-labelledby": "{base}-tab-{active.key}",
                    {active.children.clone()}
                }
            }
        }
    }
}

#[component]
fn TabTrigger(
    index: usize,
    tab_id: String,
    panel_id: String,
    label: Element,
    is_active: bool,
    disabled: bool,
    class: &'static str,
    on_activate: EventHandler<()>,
) -> Element {
    let mut focus = use_context::<FocusState>();
    let index = use_signal(|| index);
    use_focus_entry_disabled(focus, index, move || disabled);
    let onmounted = use_focus_control_disabled(focus, index, move || disabled);

    let idx = index();
    let tabindex = if focus.is_focused(idx) || (!focus.any_focused() && is_active) {
        "0"
    } else {
        "-1"
    };

    rsx! {
        button {
            r#type: "button",
            role: "tab",
            id: "{tab_id}",
            "aria-controls": "{panel_id}",
            "aria-selected": if is_active { "true" } else { "false" },
            class,
            disabled,
            tabindex,
            onmounted,
            onkeydown: move |e: KeyboardEvent| {
                match e.key() {
                    Key::ArrowRight | Key::ArrowDown => {
                        e.prevent_default();
                        focus.focus_next();
                    }
                    Key::ArrowLeft | Key::ArrowUp => {
                        e.prevent_default();
                        focus.focus_prev();
                    }
                    Key::Home => {
                        e.prevent_default();
                        focus.focus_first();
                    }
                    Key::End => {
                        e.prevent_default();
                        focus.focus_last();
                    }
                    _ => {}
                }
            },
            onclick: move |_| {
                if !disabled {
                    on_activate.call(());
                }
            },
            {label}
        }
    }
}
