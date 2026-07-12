use dioxus::prelude::*;
use time::Date;

use crate::icon::{Icon, IconName};

pub const TRIGGER_CLASS: &str = "peer flex w-full h-12 min-w-0 items-center justify-between rounded-lg border border-input bg-transparent px-4 py-2 text-sm text-foreground transition-all duration-200 outline-none cursor-pointer focus:border-primary focus:ring-1 focus:ring-primary disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20";

pub const HEADER: &str = "px-6 pt-5 pb-4 border-b border-border";

pub const HEADER_LABEL: &str = "text-xs font-medium text-muted-foreground mb-3";

pub const HEADER_DATE: &str = "text-2xl font-normal text-foreground";

pub const EDIT_BTN: &str = "size-10 flex items-center justify-center rounded-full text-muted-foreground hover:bg-accent transition-colors cursor-pointer";

#[component]
pub fn CalendarIcon(#[props(default)] class: String) -> Element {
    let cls = if class.is_empty() {
        "size-4 shrink-0 text-muted-foreground".to_string()
    } else {
        class
    };
    rsx! { Icon { name: IconName::Calendar, class: "{cls}" } }
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
    rsx! {
        button {
            r#type: "button",
            class: EDIT_BTN,
            onclick: move |_| on_click.call(()),
            if input_mode() {
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
}
