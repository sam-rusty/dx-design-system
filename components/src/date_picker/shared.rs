use dioxus::prelude::*;

use crate::hooks::use_prop_signal;
use time::Date;

use crate::icon::{Icon, IconName};
use crate::{Modal, ModalSize};

pub const TRIGGER_CLASS: &str = "peer flex w-full h-12 min-w-0 items-center justify-between rounded-lg border border-input bg-transparent px-4 py-2 text-sm text-foreground transition-all duration-200 outline-none cursor-pointer focus:border-primary focus:ring-1 focus:ring-primary disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20";

pub const HEADER: &str = "px-6 pt-5 pb-4 border-b border-border";

pub const HEADER_LABEL: &str = "text-xs font-medium text-muted-foreground mb-3";

pub const HEADER_DATE: &str = "text-2xl font-normal text-foreground";

pub const EDIT_BTN: &str = "size-10 flex items-center justify-center rounded-full text-muted-foreground hover:bg-accent transition-colors cursor-pointer";

#[component]
pub fn CalendarIcon(
    #[props(default = "size-4 shrink-0 text-muted-foreground".to_string())] class: String,
) -> Element {
    rsx! { Icon { name: IconName::Calendar, class: "{class}" } }
}

#[component]
pub fn CalendarClockIcon() -> Element {
    rsx! { Icon { name: IconName::CalendarClock, class: "size-4 shrink-0 text-muted-foreground" } }
}

#[component]
pub fn EditIcon() -> Element {
    rsx! { Icon { name: IconName::Edit, class: "size-5" } }
}

#[component]
pub fn EditToggleButton(input_mode: ReadSignal<bool>, on_click: EventHandler<()>) -> Element {
    let editing = input_mode();
    rsx! {
        button {
            r#type: "button",
            class: EDIT_BTN,
            "aria-label": if editing { "Switch to calendar" } else { "Type the date" },
            "aria-pressed": editing,
            onclick: move |_| on_click.call(()),
            if editing {
                CalendarIcon { class: "size-5".to_string() }
            } else {
                EditIcon {}
            }
        }
    }
}

#[component]
pub fn PickerHeader(title: &'static str, children: Element) -> Element {
    rsx! {
        div { class: HEADER,
            div { class: HEADER_LABEL, "{title}" }
            div { class: "flex items-center justify-between",
                {children}
            }
        }
    }
}

/// The closed-state control every picker renders: a `button` showing the
/// formatted value (or a non-breaking space) plus an icon, wired to the
/// surrounding form field through the optional `id` / `aria-*` props.
#[component]
pub(crate) fn PickerTrigger(
    /// Fully merged class string.
    class: String,
    /// Formatted value; empty renders a non-breaking space to hold the height.
    display: String,
    disabled: bool,
    is_open: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] aria_invalid: Option<String>,
    #[props(default)] aria_describedby: Option<String>,
    /// Fired when an enabled trigger is clicked.
    on_open: EventHandler<()>,
    /// The trailing icon.
    children: Element,
) -> Element {
    let display = display_or_nbsp(display);
    rsx! {
        button {
            r#type: "button",
            id,
            class: "{class}",
            disabled,
            "data-state": if is_open { "open" } else { "closed" },
            "aria-haspopup": "dialog",
            "aria-expanded": is_open,
            "aria-invalid": aria_invalid,
            "aria-describedby": aria_describedby,
            onclick: move |ev| {
                ev.stop_propagation();
                if disabled {
                    return;
                }
                on_open.call(());
            },
            span { class: "truncate", "{display}" }
            {children}
        }
    }
}

/// The picker dialog frame: headerless small [`Modal`] with a
/// [`PickerHeader`] (title + `header` line + optional [`EditToggleButton`])
/// above the padded body (`children`).
#[component]
pub(crate) fn PickerDialog(
    title: &'static str,
    /// The big header line (selected date / range / "Pick a date").
    header: String,
    on_close: EventHandler<()>,
    /// When `Some`, renders the calendar/text-input toggle bound to this
    /// signal; `None` hides it (e.g. the time step).
    #[props(default)]
    input_mode: Option<ReadSignal<bool>>,
    #[props(default)] on_edit_toggle: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        Modal {
            on_close: move |_| on_close.call(()),
            headerless: true,
            unpadded: true,
            size: ModalSize::Sm,
            PickerHeader { title,
                div { class: HEADER_DATE, "{header}" }
                if let Some(input_mode) = input_mode {
                    EditToggleButton { input_mode, on_click: move |_| on_edit_toggle.call(()) }
                }
            }
            div { class: "p-4", {children} }
        }
    }
}

/// Whether `date` falls outside the inclusive `[min, max]` window.
pub(crate) fn out_of_range(date: Date, min: Option<Date>, max: Option<Date>) -> bool {
    min.is_some_and(|mn| date < mn) || max.is_some_and(|mx| date > mx)
}

/// Lift an optional prop value into the `Option<Signal<T>>` shape the picker
/// bases take. The signal tracks later prop changes (compare-and-set), and
/// `None` in → `None` out.
pub(crate) fn use_optional_signal<T: Clone + Default + PartialEq + 'static>(
    value: Option<T>,
) -> Option<Signal<T>> {
    let sig = use_prop_signal(value.clone().unwrap_or_default());
    value.map(|_| sig)
}

/// Open-transition guard for the picker dialogs: `reset` (re-seed the staging
/// state from the committed value) runs exactly once per rising edge of
/// `is_open`, whether the trigger or an external writer flipped it. Returns
/// the `open` callback for the trigger, which resets synchronously so the
/// dialog's first frame is already fresh.
pub(crate) fn use_open_reset(mut is_open: Signal<bool>, reset: Callback<()>) -> Callback<()> {
    // `true` once `reset` has run for the current open session.
    let mut armed = use_signal(|| false);
    use_effect(move || {
        let open = is_open();
        if open && !*armed.peek() {
            reset.call(());
            armed.set(true);
        } else if !open && *armed.peek() {
            armed.set(false);
        }
    });
    use_callback(move |_: ()| {
        reset.call(());
        armed.set(true);
        is_open.set(true);
    })
}

pub fn format_display_date(date: &Date) -> String {
    format!(
        "{:02}/{:02}/{:04}",
        date.month() as u8,
        date.day(),
        date.year()
    )
}

pub fn display_or_nbsp(value: String) -> String {
    if value.is_empty() {
        "\u{00A0}".to_string()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn display_or_nbsp_empty_returns_nbsp() {
        let result = display_or_nbsp(String::new());
        assert_eq!(result, "\u{00A0}");
    }

    #[test]
    fn display_or_nbsp_non_empty_returns_value() {
        let result = display_or_nbsp("hello".to_string());
        assert_eq!(result, "hello");
    }

    #[test]
    fn display_or_nbsp_whitespace_returns_whitespace() {
        let result = display_or_nbsp(" ".to_string());
        assert_eq!(result, " ");
    }

    #[test]
    fn display_or_nbsp_date_string_returns_as_is() {
        let result = display_or_nbsp("2026-03-10".to_string());
        assert_eq!(result, "2026-03-10");
    }

    #[test]
    fn format_display_date_output() {
        let d = Date::from_calendar_date(2026, time::Month::June, 20).unwrap();
        assert_eq!(format_display_date(&d), "06/20/2026");
    }

    #[test]
    fn format_display_date_single_digit_month_day() {
        let d = Date::from_calendar_date(2026, time::Month::March, 5).unwrap();
        assert_eq!(format_display_date(&d), "03/05/2026");
    }

    #[test]
    fn display_or_nbsp_result_is_not_empty_for_empty_input() {
        let result = display_or_nbsp(String::new());
        assert!(!result.is_empty());
        assert_eq!(result.len(), 2); // \u{00A0} is 2 bytes in UTF-8
    }

    #[test]
    fn out_of_range_bounds_are_inclusive() {
        let d = date(2026, 3, 10);
        assert!(!out_of_range(d, None, None));
        assert!(!out_of_range(d, Some(d), Some(d)));
        assert!(out_of_range(d, Some(date(2026, 3, 11)), None));
        assert!(out_of_range(d, None, Some(date(2026, 3, 9))));
        assert!(!out_of_range(
            d,
            Some(date(2026, 1, 1)),
            Some(date(2026, 12, 31))
        ));
    }
}

/// Verifier-added (D10): `use_open_reset` runs `reset` exactly once per rising
/// edge — trigger path and external `is_open` writer — and never twice.
#[cfg(test)]
mod open_reset_tests {
    use super::*;
    use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
    use std::cell::RefCell;

    type Handles = (Signal<bool>, Signal<u32>, Callback<()>);
    thread_local! { static HANDLES: RefCell<Option<Handles>> = const { RefCell::new(None) }; }

    fn app() -> Element {
        let is_open = use_signal(|| false);
        let mut count = use_signal(|| 0u32);
        let reset = use_callback(move |_: ()| count += 1);
        let open = use_open_reset(is_open, reset);
        use_hook(move || HANDLES.with(|h| *h.borrow_mut() = Some((is_open, count, open))));
        rsx! { div { "{is_open()} {count()}" } }
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..3 {
            dom.render_immediate(&mut NoOpMutations);
        }
    }

    #[test]
    fn reset_runs_once_per_rising_edge() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let (mut is_open, count, open) = HANDLES.with(|h| h.borrow_mut().take()).unwrap();
        assert_eq!(*count.peek(), 0, "closed at mount: no reset");

        // Trigger path: synchronous reset, effect must not repeat it.
        dom.in_runtime(|| open.call(()));
        settle(&mut dom);
        assert!(*is_open.peek());
        assert_eq!(*count.peek(), 1, "trigger open resets exactly once");

        // Staying open re-renders without resetting.
        dom.in_runtime(|| is_open.set(true));
        settle(&mut dom);
        assert_eq!(*count.peek(), 1);

        dom.in_runtime(|| is_open.set(false));
        settle(&mut dom);
        assert_eq!(*count.peek(), 1, "closing does not reset");

        // External writer: reset via the rising-edge effect, once.
        dom.in_runtime(|| is_open.set(true));
        settle(&mut dom);
        assert_eq!(*count.peek(), 2, "external open resets exactly once");

        dom.in_runtime(|| is_open.set(false));
        dom.in_runtime(|| open.call(()));
        settle(&mut dom);
        assert_eq!(*count.peek(), 3, "reopen via trigger resets once more");
    }
}
