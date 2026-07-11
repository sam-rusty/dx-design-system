use dioxus::prelude::*;
use time::Date;

use crate::form::{FieldContext, FormContext};
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

pub fn use_form_field() -> (String, FormContext) {
    let field_name = use_context::<FieldContext>().name;
    let form_ctx = use_context::<FormContext>();
    (String::from(&*field_name), form_ctx)
}

pub fn form_value_signal(field_name: &str, form_ctx: FormContext) -> ReadSignal<String> {
    let field_name = field_name.to_string();
    use_memo(move || {
        form_ctx
            .values_signal
            .read()
            .get(&*field_name)
            .cloned()
            .unwrap_or_default()
    })
    .into()
}

pub fn form_on_change(field_name: &str, form_ctx: FormContext) -> EventHandler<String> {
    let field_name = field_name.to_string();
    EventHandler::new(move |val: String| {
        form_ctx.set_value.read()(&field_name, val.clone());
        form_ctx.touch_field.read()(&field_name);
    })
}

pub fn form_disabled(form_ctx: FormContext) -> ReadSignal<bool> {
    use_memo(move || form_ctx.disabled.map(|d| d()).unwrap_or(false)).into()
}

#[component]
pub fn FloatingLabel(
    label: String,
    is_open: ReadSignal<bool>,
    #[props(default)] data_name: Option<&'static str>,
) -> Element {
    let (field_name, form_ctx) = use_form_field();

    let has_value = {
        let field_name = field_name.clone();
        use_memo(move || {
            form_ctx
                .values_signal
                .read()
                .get(&*field_name)
                .is_some_and(|s| !s.is_empty())
        })
    };

    let is_floated = use_memo(move || has_value() || is_open());

    let label_class = use_memo(move || {
        let base = "absolute start-3 z-10 pointer-events-none bg-card px-1 text-muted-foreground transition-all duration-200 origin-[0]";
        if is_floated() {
            format!(
                "{} {} {}",
                base,
                "top-0 -translate-y-1/2 scale-75 text-sm font-medium",
                if is_open() { "text-primary" } else { "" }
            )
        } else {
            format!(
                "{} {}",
                base, "top-1/2 -translate-y-1/2 scale-100 text-sm font-normal"
            )
        }
    });

    let data_name = data_name.unwrap_or("PickerLabel");

    rsx! {
        label {
            "data-name": "{data_name}",
            class: "{label_class()}",
            r#for: "{field_name}",
            "{label}"
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
