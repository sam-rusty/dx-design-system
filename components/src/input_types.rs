//! Standalone typed input bases. Each base owns its type-specific behavior
//! (formatting, filtering, clamping, reveal toggle) exactly once, so the same
//! behavior is available standalone and through the form-bound wrappers.

use dioxus::prelude::*;

use ds_utils::format::{
    PhoneFormat, clamp_percent, filter_percent, format_number, format_percent, format_phone_with,
    parse_number, parse_percent, parse_phone_with,
};

use crate::hooks::{use_controlled, use_prop_signal};
use crate::icon::{Icon, IconName};
use crate::input::{FieldSize, InputBase, InputBaseProps, InputType};

/// Declares a typed-base props struct: the shared field set (written once,
/// here), then the struct's own extra fields, then `attributes`. With
/// `split => name`, also emits `fn name(self) -> (TypedInputBaseProps, (extras,))`
/// that peels the extras off and rebuilds the shared struct, so the public
/// structs cannot drift from [`TypedInputBaseProps`].
macro_rules! typed_input_props {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $( $(#[$field_meta:meta])* pub $field:ident : $ty:ty ),* $(,)?
        }
        split => $split:ident
    ) => {
        typed_input_props! {
            $(#[$meta])*
            pub struct $name { $( $(#[$field_meta])* pub $field : $ty ),* }
        }

        impl $name {
            /// Split into the shared base props and this struct's extra fields.
            fn $split(self) -> (TypedInputBaseProps, ( $($ty,)* )) {
                let $name {
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
                    $($field,)*
                } = self;
                let base = TypedInputBaseProps {
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
                (base, ( $($field,)* ))
            }
        }
    };
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $( $(#[$field_meta:meta])* pub $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Props, Clone, PartialEq)]
        pub struct $name {
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
            $( $(#[$field_meta])* pub $field: $ty, )*
            /// Additional attributes (`name`, `min`, `max`, `step`, `readonly`, ...).
            #[props(extends = GlobalAttributes, extends = input)]
            pub attributes: Vec<Attribute>,
        }
    };
}

typed_input_props! {
    /// Shared props for the typed standalone input family ([`TextInputBase`],
    /// [`EmailInputBase`], [`PasswordInputBase`], [`PhoneInputBase`],
    /// [`NumberInputBase`]). The `value` contract is always the *raw* value —
    /// formatting is internal to each base.
    pub struct TypedInputBaseProps {}
}

/// The raw ↔ display transform pair of one typed base. A variant rather than
/// two function pointers so a transform can carry its own configuration.
#[derive(Clone, Copy)]
enum Transform {
    /// Display is the raw value.
    None,
    /// A fixed format/parse pair (number, percentage).
    Fns {
        /// raw → display (applied when rendering).
        format: fn(&str) -> String,
        /// display → raw (applied to every input/change event).
        parse: fn(&str) -> String,
    },
    /// Phone, formatted for the caller's country.
    Phone(PhoneFormat),
}

// Function pointers are compared by address (cast to `usize`) — the plain `==`
// on `fn` items is flagged as unpredictable; here a false "changed" only costs a
// signal write, never wrong output.
impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (
                Self::Fns { format, parse },
                Self::Fns {
                    format: f2,
                    parse: p2,
                },
            ) => *format as usize == *f2 as usize && *parse as usize == *p2 as usize,
            (Self::Phone(a), Self::Phone(b)) => a == b,
            _ => false,
        }
    }
}

impl Transform {
    fn format(self, raw: &str) -> String {
        match self {
            Self::None => raw.to_string(),
            Self::Fns { format, .. } => format(raw),
            Self::Phone(format) => format_phone_with(raw, format),
        }
    }

    fn parse(self, display: String) -> String {
        match self {
            Self::None => display,
            Self::Fns { parse, .. } => parse(&display),
            Self::Phone(format) => parse_phone_with(&display, format),
        }
    }
}

/// Type-specific behavior of one typed base: HTML type, soft-keyboard hint,
/// and the display/raw transforms.
#[derive(Clone, Copy)]
struct TypedBehavior {
    r#type: InputType,
    inputmode: Option<&'static str>,
    /// raw ↔ display.
    transform: Transform,
    /// Keystroke filter applied before `transform`'s parse.
    filter: Option<fn(&str) -> String>,
}

impl PartialEq for TypedBehavior {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type
            && self.inputmode == other.inputmode
            && self.transform == other.transform
            && self.filter.map(|f| f as usize) == other.filter.map(|f| f as usize)
    }
}

impl TypedBehavior {
    const fn plain(r#type: InputType) -> Self {
        Self {
            r#type,
            inputmode: None,
            transform: Transform::None,
            filter: None,
        }
    }

    fn normalize(&self, input: String) -> String {
        let filtered = match self.filter {
            Some(f) => f(&input),
            None => input,
        };
        self.transform.parse(filtered)
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

    // The display memo's closure is created once; mirror `behavior` through a
    // signal so a transform that changes at runtime (e.g. a phone format picked
    // from a country selector) re-formats the display instead of staying stale.
    let behavior_sig = use_prop_signal(behavior);
    let display: ReadSignal<Option<String>> =
        use_memo(move || Some(behavior_sig.read().transform.format(&raw()))).into();

    let on_commit = props.on_commit;

    let mut attributes = props.attributes;
    if let Some(mode) = behavior.inputmode
        && !attributes.iter().any(|a| a.name == "inputmode")
    {
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

typed_input_props! {
    /// Props for [`PhoneInputBase`]: [`TypedInputBaseProps`] plus the country
    /// display format.
    pub struct PhoneInputBaseProps {
        /// Display format, as a pattern whose `#` are national digits
        /// (`"+44 #### ######"`), or a list of them for a country with several
        /// number lengths. Defaults to [`PhoneFormat::US`].
        #[props(default, into)]
        pub format: PhoneFormat,
    }
    split => into_base
}

/// Phone input (`type="tel"`): displays the number in `format`'s format,
/// propagates the raw national digits.
pub fn PhoneInputBase(props: PhoneInputBaseProps) -> Element {
    let (inner, (format,)) = props.into_base();
    let behavior = TypedBehavior {
        r#type: InputType::Tel,
        inputmode: Some("tel"),
        transform: Transform::Phone(format),
        filter: None,
    };
    TypedInput(inner, behavior, None, |v| v)
}

/// Numeric input: thousands-separated display, raw decimal string value,
/// keystrokes filtered to a valid number.
pub fn NumberInputBase(props: TypedInputBaseProps) -> Element {
    let behavior = TypedBehavior {
        r#type: InputType::Text,
        inputmode: Some("decimal"),
        transform: Transform::Fns {
            format: format_number,
            parse: parse_number,
        },
        filter: Some(filter_numeric),
    };
    TypedInput(props, behavior, None, |v| v)
}

typed_input_props! {
    /// Props for [`PercentageInputBase`]: [`TypedInputBaseProps`] plus clamp bounds.
    pub struct PercentageInputBaseProps {
        /// Lower clamp bound applied on commit.
        #[props(default = 0.0)]
        pub min: f64,
        /// Upper clamp bound applied on commit.
        #[props(default = 100.0)]
        pub max: f64,
    }
    split => into_base
}

/// Percentage input: percent-formatted display, raw decimal string value,
/// clamped into `[min, max]` when the value is committed.
pub fn PercentageInputBase(props: PercentageInputBaseProps) -> Element {
    let (inner, (min, max)) = props.into_base();
    let behavior = TypedBehavior {
        r#type: InputType::Text,
        inputmode: Some("decimal"),
        transform: Transform::Fns {
            format: format_percent,
            parse: parse_percent,
        },
        filter: Some(filter_percent_signed),
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

/// Keystroke filter for [`PercentageInputBase`]: `filter_percent` plus one
/// leading `-`, so a negative `min` is reachable (the commit clamp still
/// applies).
fn filter_percent_signed(input: &str) -> String {
    match input.strip_prefix('-') {
        Some(rest) => format!("-{}", filter_percent(rest)),
        None => filter_percent(input),
    }
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
            transform: Transform::Fns {
                format: format_number,
                parse: parse_number,
            },
            filter: Some(filter_numeric),
        };
        let raw = "1234567.89";
        let display = format_number(raw);
        assert_eq!(behavior.normalize(display), raw);
    }

    fn phone_behavior(format: PhoneFormat) -> TypedBehavior {
        TypedBehavior {
            r#type: InputType::Tel,
            inputmode: Some("tel"),
            transform: Transform::Phone(format),
            filter: None,
        }
    }

    #[test]
    fn phone_behavior_normalize_round_trips_formatted_input() {
        let behavior = phone_behavior(PhoneFormat::default());
        let raw = "5551234567";
        let display = behavior.transform.format(raw);
        assert_eq!(display, "+1 (555) 123-4567");
        assert_eq!(behavior.normalize(display), raw);
    }

    #[test]
    fn phone_behavior_honors_a_custom_config() {
        let behavior = phone_behavior(PhoneFormat::new("+44 #### ######"));
        let raw = "7911123456";
        let display = behavior.transform.format(raw);
        assert_eq!(display, "+44 7911 123456");
        assert_eq!(behavior.normalize(display), raw);
    }

    #[test]
    fn filter_numeric_keeps_partial_entries() {
        assert_eq!(filter_numeric("-"), "-");
        assert_eq!(filter_numeric("-."), "-0.");
        assert_eq!(filter_numeric("1."), "1.");
    }

    #[test]
    fn percent_behavior_normalizes_formatted_input() {
        let behavior = TypedBehavior {
            r#type: InputType::Text,
            inputmode: Some("decimal"),
            transform: Transform::Fns {
                format: format_percent,
                parse: parse_percent,
            },
            filter: Some(filter_percent_signed),
        };
        assert_eq!(behavior.normalize("50%".to_string()), "50");
        assert_eq!(behavior.normalize("-12.5%".to_string()), "-12.5");
        assert_eq!(behavior.normalize("1-2".to_string()), "12");
    }

    #[test]
    fn percent_clamp_bounds() {
        assert_eq!(clamp_percent("150", 0.0, 100.0), "100");
        assert_eq!(clamp_percent("-5", 0.0, 100.0), "0");
        assert_eq!(clamp_percent("42", 0.0, 100.0), "42");
    }
}
