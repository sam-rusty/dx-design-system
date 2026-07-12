//! Standalone typed input bases. Each base owns its type-specific behavior
//! (formatting, filtering, clamping, reveal toggle) exactly once, so the same
//! behavior is available standalone and through the form-bound wrappers.

use dioxus::prelude::*;

use ds_utils::format::{
    clamp_percent, filter_percent, format_number, format_percent, format_phone, parse_number,
    parse_percent, parse_phone,
};

use crate::hooks::use_controlled;
use crate::icon::{Icon, IconName};
use crate::input::{FieldSize, InputBase, InputBaseProps, InputType};

/// Shared props for the typed standalone input family ([`TextInputBase`],
/// [`EmailInputBase`], [`PasswordInputBase`], [`PhoneInputBase`],
/// [`NumberInputBase`]). The `value` contract is always the *raw* value —
/// formatting is internal to each base.
#[derive(Props, Clone, PartialEq)]
pub struct TypedInputBaseProps {
    /// Controlled raw value. `Some` makes the caller the source of truth
    /// (pair with `on_value_change`); `None` leaves the input uncontrolled.
    #[props(default)]
    pub value: ReadSignal<Option<String>>,
    /// Initial raw value when uncontrolled.
    #[props(default)]
    pub default_value: String,
    /// Fired with the new raw (parsed) value on every input event.
    #[props(default)]
    pub on_value_change: Callback<String>,
    /// Fired with the committed raw value on the change event (blur / Enter).
    #[props(default)]
    pub on_commit: Callback<String>,
    /// Fired when the input loses focus.
    #[props(default)]
    pub on_blur: Callback<FocusEvent>,
    /// Fired on keydown.
    #[props(default)]
    pub on_key_down: Callback<KeyboardEvent>,
    /// Whether the input is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Extra classes merged into the base style; the full class list when `unstyled`.
    #[props(default)]
    pub class: String,
    /// Placeholder text.
    #[props(default)]
    pub placeholder: Option<String>,
    /// DOM id. Form bindings set this to the field name so labels target it.
    #[props(default)]
    pub id: Option<String>,
    /// Autofocus on mount (deferred by `AutofocusGate` when one is provided).
    #[props(default)]
    pub autofocus: bool,
    /// Skip the built-in styling entirely; `class` is used verbatim.
    #[props(default)]
    pub unstyled: bool,
    /// `aria-invalid` value (form bindings set `"true"` on validation failure).
    #[props(default)]
    pub aria_invalid: Option<String>,
    /// `aria-describedby` target (the field's error element id).
    #[props(default)]
    pub aria_describedby: Option<String>,
    /// Additional attributes (`name`, `min`, `max`, `step`, `readonly`, ...).
    #[props(extends = GlobalAttributes, extends = input)]
    pub attributes: Vec<Attribute>,
}

/// Type-specific behavior of one typed base: HTML type, soft-keyboard hint,
/// and the display/raw transforms.
#[derive(Clone, Copy)]
struct TypedBehavior {
    r#type: InputType,
    inputmode: Option<&'static str>,
    /// raw → display (applied when rendering).
    format: Option<fn(&str) -> String>,
    /// display → raw (applied to every input/change event).
    parse: Option<fn(&str) -> String>,
    /// Keystroke filter applied before `parse`.
    filter: Option<fn(&str) -> String>,
}

impl TypedBehavior {
    const fn plain(r#type: InputType) -> Self {
        Self {
            r#type,
            inputmode: None,
            format: None,
            parse: None,
            filter: None,
        }
    }

    fn normalize(&self, input: String) -> String {
        let filtered = match self.filter {
            Some(f) => f(&input),
            None => input,
        };
        match self.parse {
            Some(p) => p(&filtered),
            None => filtered,
        }
    }
}

/// Shared plumbing for every typed base: controlled raw value, display
/// formatting, and event normalization. `commit_map` post-processes the
/// committed raw value (percent clamping); the identity elsewhere.
fn TypedInput<M: Fn(String) -> String + Copy + 'static>(
    props: TypedInputBaseProps,
    behavior: TypedBehavior,
    trailing: Option<Element>,
    commit_map: M,
) -> Element {
    let (raw, set_raw) = use_controlled(
        props.value,
        props.default_value.clone(),
        props.on_value_change,
    );

    let display: ReadSignal<Option<String>> = use_memo(move || {
        Some(match behavior.format {
            Some(f) => f(&raw()),
            None => raw(),
        })
    })
    .into();

    let on_commit = props.on_commit;

    let mut attributes = props.attributes;
    if let Some(mode) = behavior.inputmode {
        attributes.push(Attribute::new("inputmode", mode, None, false));
    }

    let input_props = InputBaseProps {
        value: display,
        default_value: String::new(),
        on_value_change: Callback::new(move |v: String| set_raw(behavior.normalize(v))),
        on_commit: Callback::new(move |v: String| {
            let normalized = behavior.normalize(v);
            let mapped = commit_map(normalized.clone());
            if mapped != normalized {
                set_raw(mapped.clone());
            }
            on_commit.call(mapped);
        }),
        on_blur: props.on_blur,
        on_key_down: props.on_key_down,
        r#type: behavior.r#type,
        size: props.size,
        class: props.class,
        placeholder: props.placeholder,
        id: props.id,
        disabled: props.disabled,
        autofocus: props.autofocus,
        unstyled: props.unstyled,
        aria_invalid: props.aria_invalid,
        aria_describedby: props.aria_describedby,
        trailing,
        attributes,
    };

    rsx! {
        InputBase { ..input_props }
    }
}

/// Plain text input (`type="text"`).
pub fn TextInputBase(props: TypedInputBaseProps) -> Element {
    TypedInput(props, TypedBehavior::plain(InputType::Text), None, |v| v)
}

/// Email input (`type="email"`).
pub fn EmailInputBase(props: TypedInputBaseProps) -> Element {
    TypedInput(props, TypedBehavior::plain(InputType::Email), None, |v| v)
}

/// Phone input (`type="tel"`): displays the formatted number, propagates the
/// raw digits.
pub fn PhoneInputBase(props: TypedInputBaseProps) -> Element {
    let behavior = TypedBehavior {
        r#type: InputType::Tel,
        inputmode: Some("tel"),
        format: Some(format_phone),
        parse: Some(parse_phone),
        filter: None,
    };
    TypedInput(props, behavior, None, |v| v)
}

/// Numeric input: thousands-separated display, raw decimal string value,
/// keystrokes filtered to a valid number.
pub fn NumberInputBase(props: TypedInputBaseProps) -> Element {
    let behavior = TypedBehavior {
        r#type: InputType::Text,
        inputmode: Some("decimal"),
        format: Some(format_number),
        parse: Some(parse_number),
        filter: Some(filter_numeric),
    };
    TypedInput(props, behavior, None, |v| v)
}

/// Props for [`PercentageInputBase`]: [`TypedInputBaseProps`] plus clamp bounds.
#[derive(Props, Clone, PartialEq)]
pub struct PercentageInputBaseProps {
    /// Controlled raw value. `Some` makes the caller the source of truth
    /// (pair with `on_value_change`); `None` leaves the input uncontrolled.
    #[props(default)]
    pub value: ReadSignal<Option<String>>,
    /// Initial raw value when uncontrolled.
    #[props(default)]
    pub default_value: String,
    /// Fired with the new raw (parsed) value on every input event.
    #[props(default)]
    pub on_value_change: Callback<String>,
    /// Fired with the committed, clamped raw value on the change event.
    #[props(default)]
    pub on_commit: Callback<String>,
    /// Fired when the input loses focus.
    #[props(default)]
    pub on_blur: Callback<FocusEvent>,
    /// Fired on keydown.
    #[props(default)]
    pub on_key_down: Callback<KeyboardEvent>,
    /// Whether the input is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Extra classes merged into the base style; the full class list when `unstyled`.
    #[props(default)]
    pub class: String,
    /// Placeholder text.
    #[props(default)]
    pub placeholder: Option<String>,
    /// DOM id. Form bindings set this to the field name so labels target it.
    #[props(default)]
    pub id: Option<String>,
    /// Autofocus on mount (deferred by `AutofocusGate` when one is provided).
    #[props(default)]
    pub autofocus: bool,
    /// Skip the built-in styling entirely; `class` is used verbatim.
    #[props(default)]
    pub unstyled: bool,
    /// `aria-invalid` value (form bindings set `"true"` on validation failure).
    #[props(default)]
    pub aria_invalid: Option<String>,
    /// `aria-describedby` target (the field's error element id).
    #[props(default)]
    pub aria_describedby: Option<String>,
    /// Lower clamp bound applied on commit.
    #[props(default = 0.0)]
    pub min: f64,
    /// Upper clamp bound applied on commit.
    #[props(default = 100.0)]
    pub max: f64,
    /// Additional attributes.
    #[props(extends = GlobalAttributes, extends = input)]
    pub attributes: Vec<Attribute>,
}

/// Percentage input: percent-formatted display, raw decimal string value,
/// clamped into `[min, max]` when the value is committed.
pub fn PercentageInputBase(props: PercentageInputBaseProps) -> Element {
    let PercentageInputBaseProps {
        value,
        default_value,
        on_value_change,
        on_commit,
        on_blur,
        on_key_down,
        disabled,
        size,
        class,
        placeholder,
        id,
        autofocus,
        unstyled,
        aria_invalid,
        aria_describedby,
        min,
        max,
        attributes,
    } = props;
    let behavior = TypedBehavior {
        r#type: InputType::Text,
        inputmode: Some("decimal"),
        format: Some(format_percent),
        parse: Some(parse_percent),
        filter: Some(filter_percent),
    };
    let inner = TypedInputBaseProps {
        value,
        default_value,
        on_value_change,
        on_commit,
        on_blur,
        on_key_down,
        disabled,
        size,
        class,
        placeholder,
        id,
        autofocus,
        unstyled,
        aria_invalid,
        aria_describedby,
        attributes,
    };
    TypedInput(inner, behavior, None, move |v| clamp_percent(&v, min, max))
}

/// Password input with a reveal toggle rendered through the trailing slot.
/// [`InputBase`] wraps the input + trailing in its own `relative` box, so this
/// positions correctly standalone or inside a form frame.
pub fn PasswordInputBase(props: TypedInputBaseProps) -> Element {
    let mut revealed = use_signal(|| false);
    let behavior = TypedBehavior::plain(if revealed() {
        InputType::Text
    } else {
        InputType::Password
    });

    let trailing = rsx! {
        button {
            r#type: "button",
            tabindex: "-1",
            "aria-label": if revealed() { "Hide password" } else { "Show password" },
            class: "grid place-items-center size-9 rounded-full text-muted-foreground \
                    hover:text-foreground transition-colors cursor-pointer",
            onclick: move |_| revealed.toggle(),
            Icon {
                name: if revealed() { IconName::EyeOff } else { IconName::Eye },
                class: "size-5",
            }
        }
    };

    TypedInput(props, behavior, Some(trailing), |v| v)
}

/// Keystroke filter for [`NumberInputBase`]: digits, one leading `-`, one `.`,
/// leading zeros stripped from the integer part.
fn filter_numeric(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut has_dot = false;
    for (i, c) in input.chars().enumerate() {
        if c.is_ascii_digit() || (c == '-' && i == 0) || (c == '.' && !has_dot) {
            if c == '.' {
                has_dot = true;
            }
            result.push(c);
        }
    }
    let negative = result.starts_with('-');
    let abs = if negative { &result[1..] } else { &result[..] };
    if abs.is_empty() {
        return result;
    }
    let stripped = match abs.find('.') {
        Some(dot_pos) => {
            let int_part = abs[..dot_pos].trim_start_matches('0');
            let int_part = if int_part.is_empty() { "0" } else { int_part };
            format!("{}{}", int_part, &abs[dot_pos..])
        }
        None => {
            let s = abs.trim_start_matches('0');
            if s.is_empty() {
                "0".to_string()
            } else {
                s.to_string()
            }
        }
    };
    if negative {
        format!("-{}", stripped)
    } else {
        stripped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_numeric_strips_invalid_chars() {
        assert_eq!(filter_numeric("1a2b3"), "123");
        assert_eq!(filter_numeric("abc"), "");
    }

    #[test]
    fn filter_numeric_allows_single_dot() {
        assert_eq!(filter_numeric("1.2.3"), "1.23");
        assert_eq!(filter_numeric("."), "0.");
    }

    #[test]
    fn filter_numeric_allows_leading_minus_only() {
        assert_eq!(filter_numeric("-12"), "-12");
        assert_eq!(filter_numeric("1-2"), "12");
    }

    #[test]
    fn filter_numeric_strips_leading_zeros() {
        assert_eq!(filter_numeric("007"), "7");
        assert_eq!(filter_numeric("0.5"), "0.5");
        assert_eq!(filter_numeric("00.5"), "0.5");
        assert_eq!(filter_numeric("-007"), "-7");
        assert_eq!(filter_numeric("0"), "0");
    }

    #[test]
    fn number_behavior_normalize_round_trips_formatted_input() {
        let behavior = TypedBehavior {
            r#type: InputType::Text,
            inputmode: Some("decimal"),
            format: Some(format_number),
            parse: Some(parse_number),
            filter: Some(filter_numeric),
        };
        let raw = "1234567.89";
        let display = format_number(raw);
        assert_eq!(behavior.normalize(display), raw);
    }

    #[test]
    fn phone_behavior_normalize_round_trips_formatted_input() {
        let behavior = TypedBehavior {
            r#type: InputType::Tel,
            inputmode: Some("tel"),
            format: Some(format_phone),
            parse: Some(parse_phone),
            filter: None,
        };
        let raw = "5551234567";
        let display = format_phone(raw);
        assert_eq!(behavior.normalize(display), raw);
    }

    #[test]
    fn percent_clamp_bounds() {
        assert_eq!(clamp_percent("150", 0.0, 100.0), "100");
        assert_eq!(clamp_percent("-5", 0.0, 100.0), "0");
        assert_eq!(clamp_percent("42", 0.0, 100.0), "42");
    }
}
