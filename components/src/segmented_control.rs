use dioxus::prelude::*;

use crate::hooks::{FocusState, use_focus_control, use_focus_entry_disabled, use_focus_provider};

const SEG_ACTIVE: &str = "relative z-10 flex flex-1 items-center justify-center rounded-md px-3 py-1.5 text-xs font-medium text-foreground transition-colors duration-150";
const SEG_DISABLED: &str = "relative z-10 flex flex-1 items-center justify-center rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground opacity-50 cursor-not-allowed transition-colors duration-150";
const SEG_IDLE: &str = "relative z-10 flex flex-1 items-center justify-center rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground transition-colors duration-150 cursor-pointer";

/// Shared, `Copy` context — `FocusState`/`EventHandler` aren't `PartialEq`, so
/// they can't be component props; the items read them from here instead.
#[derive(Clone, Copy)]
struct SegmentCtx {
    focus: FocusState,
    options: &'static [(&'static str, &'static str)],
    on_change: EventHandler<String>,
    disabled: bool,
}

/// Single-select control modelled as a WAI-ARIA radio group (not a tablist —
/// there are no associated tab panels). Arrow/Home/End move the selection with
/// roving focus; only the selected option is in the tab order.
#[component]
pub fn SegmentedControl(
    /// Current selected value.
    value: String,
    on_change: EventHandler<String>,
    #[props(default)] options: &'static [(&'static str, &'static str)],
    #[props(default)] disabled: bool,
) -> Element {
    let focus = use_focus_provider(use_signal(|| true).into());
    use_context_provider(|| SegmentCtx {
        focus,
        options,
        on_change,
        disabled,
    });
    let active_index = options.iter().position(|(ov, _)| *ov == value);
    let any_active = active_index.is_some();
    let indicator_style = active_index.map(|active_index| {
        let option_count = options.len();
        let total_gap_px = option_count.saturating_sub(1) * 2;
        let active_gap_px = active_index * 2;

        format!(
            "width: calc((100% - 4px - {total_gap_px}px) / {option_count}); transform: translateX(calc({}% + {active_gap_px}px));",
            active_index * 100
        )
    });

    rsx! {
        div {
            class: "relative isolate flex rounded-lg border border-border bg-muted/40 p-0.5 gap-0.5",
            role: "radiogroup",
            if let Some(indicator_style) = indicator_style {
                div {
                    "aria-hidden": "true",
                    class: "pointer-events-none absolute top-0.5 bottom-0.5 left-0.5 rounded-md bg-card shadow-sm transition-transform duration-200 ease-out will-change-transform",
                    style: "{indicator_style}",
                }
            }
            for (i , (opt_value , opt_label)) in options.iter().enumerate() {
                SegmentItem {
                    key: "{opt_value}",
                    index: i,
                    value: opt_value.to_string(),
                    label: *opt_label,
                    active: *opt_value == value,
                    roving_tab: *opt_value == value || (!any_active && i == 0),
                }
            }
        }
    }
}

#[component]
fn SegmentItem(
    index: usize,
    value: String,
    label: &'static str,
    active: bool,
    roving_tab: bool,
) -> Element {
    let ctx = use_context::<SegmentCtx>();
    let disabled = ctx.disabled;
    let options = ctx.options;
    let on_change = ctx.on_change;

    let idx = use_signal(|| index);
    use_focus_entry_disabled(ctx.focus, idx, move || disabled);
    let on_mounted = use_focus_control(ctx.focus, idx);

    let btn_class = if active {
        SEG_ACTIVE
    } else if disabled {
        SEG_DISABLED
    } else {
        SEG_IDLE
    };

    let value_click = value.clone();
    let mut focus = ctx.focus;

    rsx! {
        button {
            r#type: "button",
            role: "radio",
            "aria-checked": "{active}",
            tabindex: if roving_tab { "0" } else { "-1" },
            class: btn_class,
            disabled,
            onmounted: on_mounted,
            onclick: move |_| {
                if !disabled {
                    on_change.call(value_click.clone());
                }
            },
            onkeydown: move |ev| {
                if disabled || options.is_empty() {
                    return;
                }
                let len = options.len();
                let target = match ev.key() {
                    Key::ArrowDown | Key::ArrowRight => (index + 1) % len,
                    Key::ArrowUp | Key::ArrowLeft => (index + len - 1) % len,
                    Key::Home => 0,
                    Key::End => len - 1,
                    _ => return,
                };
                ev.prevent_default();
                focus.set_focus(Some(target));
                if let Some((v, _)) = options.get(target) {
                    on_change.call(v.to_string());
                }
            },
            "{label}"
        }
    }
}
