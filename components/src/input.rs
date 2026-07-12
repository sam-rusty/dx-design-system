use dioxus::prelude::*;
use strum_macros::AsRefStr;

use ds_utils::format::merge;

use crate::hooks::use_controlled;

/// Reactive gate an app can provide around a screen: while it reads `false`,
/// [`InputBase`] holds its autofocus. Lets a nav host defer the keyboard until
/// the screen's enter transition settles, so the keyboard slide doesn't fight
/// the push animation. Apps that never provide it get plain on-mount autofocus.
#[derive(Clone, Copy)]
pub struct AutofocusGate(pub ReadSignal<bool>);

/// Visual size scale shared by the field family (`InputBase`, the typed input
/// bases, and their form-bound wrappers).
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FieldSize {
    #[default]
    Default,
    Sm,
    Xs,
}

#[deprecated(note = "renamed to `FieldSize`")]
pub type InputSize = FieldSize;

/// Field surface is token-driven so each app themes it (radius / background /
/// border) without forking the component. `theme.css` supplies the defaults
/// that reproduce the standard rounded-bordered field; an app can override the
/// `--field-*` tokens (e.g. a pill/glass look) in its own Tailwind layer.
const STANDALONE_BASE: &str = "peer flex w-full min-w-0 rounded-[var(--field-radius)] \
     border-[length:var(--field-border-width)] border-[color:var(--field-border-color)] \
     bg-[var(--field-bg)] text-foreground transition-all duration-200 outline-none \
     placeholder:text-transparent focus:placeholder:text-muted-foreground/70 \
     file:inline-flex file:border-0 file:bg-transparent file:font-medium \
     focus:bg-[var(--field-bg-focus)] focus:border-primary focus:ring-1 focus:ring-primary \
     disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 \
     disabled:bg-muted/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
     focus:aria-invalid:border-destructive focus:aria-invalid:ring-1 read-only:bg-muted/50";

const FORM_BASE: &str = "peer block w-full appearance-none rounded-[var(--field-radius)] \
     border-[length:var(--field-border-width)] border-[color:var(--field-border-color)] \
     bg-[var(--field-bg)] text-foreground transition-colors \
     focus-visible:bg-[var(--field-bg-focus)] focus-visible:border-primary \
     focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary \
     data-[invalid=true]:border-destructive";

/// Per-size Tailwind tokens. Every field is a literal so the Tailwind scanner still sees each class.
struct SizeTokens {
    height: &'static str,
    px: &'static str,
    py: &'static str,
    pr: &'static str,
    text: &'static str,
    file_h: &'static str,
    file_text: &'static str,
}

impl FieldSize {
    fn tokens(self) -> SizeTokens {
        match self {
            Self::Default => SizeTokens {
                height: "h-12",
                px: "px-4",
                py: "py-2",
                pr: "pl-4 pr-16",
                text: "text-sm",
                file_h: "file:h-7",
                file_text: "file:text-sm",
            },
            Self::Sm => SizeTokens {
                height: "h-8",
                px: "px-3",
                py: "py-1",
                pr: "pl-3 pr-14",
                text: "text-xs",
                file_h: "file:h-6",
                file_text: "file:text-xs",
            },
            Self::Xs => SizeTokens {
                height: "h-7",
                px: "px-2",
                py: "py-0.5",
                pr: "pl-2 pr-12",
                text: "text-xs",
                file_h: "file:h-5",
                file_text: "file:text-xs",
            },
        }
    }

    /// Complete standalone [`InputBase`] class (root + file-picker + interaction states).
    /// `trailing` reserves end padding for an overlaid trailing adornment.
    fn standalone_full(self, trailing: bool) -> String {
        let t = self.tokens();
        let padding = if trailing { t.pr } else { t.px };
        merge(&[
            STANDALONE_BASE,
            t.height,
            padding,
            t.py,
            t.text,
            t.file_h,
            t.file_text,
        ])
    }

    /// Floating-label form input control (`peer block`).
    /// `trailing` reserves end padding for an overlaid trailing adornment.
    pub fn form_floating_peer_merge(self, trailing: bool) -> String {
        let t = self.tokens();
        let padding = if trailing { t.pr } else { t.px };
        merge(&[FORM_BASE, t.height, t.py, t.text, padding])
    }

    /// Full class string for form-bound controls that receive no class override.
    pub fn form_control_fallback_merge(self) -> String {
        let t = self.tokens();
        merge(&[FORM_BASE, t.height, t.px, t.py, t.text])
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum InputType {
    #[default]
    Text,
    Email,
    Password,
    Number,
    Url,
    Tel,
    Search,
    Hidden,
}

/// Props for [`InputBase`].
#[derive(Props, Clone, PartialEq)]
pub struct InputBaseProps {
    /// Controlled value. `Some` makes the caller the source of truth (pair with
    /// `on_value_change`); `None` leaves the input uncontrolled.
    #[props(default)]
    pub value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    pub default_value: String,
    /// Fired with the new value on every input event.
    #[props(default)]
    pub on_value_change: Callback<String>,
    /// Fired with the committed value on the change event (blur / Enter).
    #[props(default)]
    pub on_commit: Callback<String>,
    /// Fired when the input loses focus.
    #[props(default)]
    pub on_blur: Callback<FocusEvent>,
    /// Fired on keydown.
    #[props(default)]
    pub on_key_down: Callback<KeyboardEvent>,
    /// HTML input type.
    #[props(default)]
    pub r#type: InputType,
    /// Visual size (shared [`FieldSize`] scale).
    #[props(default)]
    pub size: FieldSize,
    /// Extra classes merged into the base style; the full class list when `unstyled`.
    #[props(default)]
    pub class: String,
    /// Placeholder text. Defaults to a single space so the floating-label
    /// `placeholder-shown` mechanism keeps working.
    #[props(default)]
    pub placeholder: Option<String>,
    /// DOM id. Form bindings set this to the field name so labels target it.
    #[props(default)]
    pub id: Option<String>,
    /// Whether the input is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Autofocus on mount (deferred by [`AutofocusGate`] when one is provided).
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
    /// Trailing adornment (reveal / copy / clear buttons). Rendered absolutely
    /// positioned after the input and reserves end padding — the nearest
    /// `relative` ancestor is the positioning context (form wrappers provide
    /// one; standalone callers must supply their own).
    #[props(default)]
    pub trailing: Option<Element>,
    /// Additional attributes (`name`, `min`, `max`, `step`, `inputmode`,
    /// `readonly`, `required`, `enterkeyhint`, ...).
    #[props(extends = GlobalAttributes, extends = input)]
    pub attributes: Vec<Attribute>,
}

pub fn InputBase(props: InputBaseProps) -> Element {
    let has_trailing = props.trailing.is_some();
    let merged_class = if props.unstyled {
        props.class.clone()
    } else {
        let base_class = props.size.standalone_full(has_trailing);
        if props.class.is_empty() {
            base_class
        } else {
            merge(&[&base_class, &props.class])
        }
    };

    let (value, set_value) = use_controlled(
        props.value,
        props.default_value.clone(),
        props.on_value_change,
    );

    let type_str = props.r#type.as_ref();
    // Floating-label hack needs a non-empty placeholder; default to a static space.
    let actual_placeholder = props.placeholder.clone().unwrap_or_else(|| " ".to_string());

    // Autofocus waits for the AutofocusGate (when provided) so the keyboard
    // doesn't open mid-transition; without a gate it fires on mount.
    let autofocus = props.autofocus;
    let gate = try_consume_context::<AutofocusGate>();
    let mut mounted = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut focus_fired = use_signal(|| false);
    use_effect(move || {
        if !autofocus || focus_fired() || gate.is_some_and(|g| !(g.0)()) {
            return;
        }
        let Some(md) = mounted() else { return };
        focus_fired.set(true);
        spawn(async move {
            let _ = md.set_focus(true).await;
        });
    });

    let on_commit = props.on_commit;
    let on_blur = props.on_blur;
    let on_key_down = props.on_key_down;
    let disabled = props.disabled;

    rsx! {
        input {
            "data-name": "Input",
            r#type: type_str,
            class: "{merged_class}",
            placeholder: actual_placeholder,
            onmounted: move |evt: MountedEvent| mounted.set(Some(evt.data())),
            id: props.id.clone(),
            disabled: disabled(),
            "aria-invalid": props.aria_invalid.clone(),
            "aria-describedby": props.aria_describedby.clone(),
            value: value(),
            oninput: move |ev| set_value(ev.value()),
            onchange: move |ev: FormEvent| on_commit.call(ev.value()),
            onblur: move |ev| on_blur.call(ev),
            onkeydown: move |ev| on_key_down.call(ev),
            ..props.attributes,
        }
        if let Some(trailing) = props.trailing {
            div {
                class: "absolute end-2 top-1/2 -translate-y-1/2 flex items-center gap-0.5 z-10",
                {trailing}
            }
        }
    }
}
