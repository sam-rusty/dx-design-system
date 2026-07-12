use dioxus::prelude::*;
use strum_macros::AsRefStr;

use ds_utils::format::merge;

use crate::icon::{Icon, IconName};

/// Reactive gate an app can provide around a screen: while it reads `false`,
/// [`InputBase`] holds its autofocus. Lets a nav host defer the keyboard until
/// the screen's enter transition settles, so the keyboard slide doesn't fight
/// the push animation. Apps that never provide it get plain on-mount autofocus.
#[derive(Clone, Copy)]
pub struct AutofocusGate(pub ReadSignal<bool>);

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum InputSize {
    #[default]
    Default,
    Sm,
    Xs,
}

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

impl InputSize {
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
    /// `has_actions` reserves trailing padding for an overlaid action button.
    fn standalone_full(self, has_actions: bool) -> String {
        let t = self.tokens();
        let padding = if has_actions { t.pr } else { t.px };
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

    /// Floating-label form `Input` control (`peer block`).
    pub fn form_floating_peer_merge(self, has_actions: bool) -> String {
        let t = self.tokens();
        let padding = if has_actions { t.pr } else { t.px };
        merge(&[FORM_BASE, t.height, t.py, t.text, padding])
    }

    /// Full class string when [`InputFormControl`] receives an empty override (matches legacy default).
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

#[component]
pub fn InputBase(
    #[props(default)] class: String,
    #[props(default)] r#type: InputType,
    #[props(default)] size: InputSize,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] title: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] required: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] min: Option<String>,
    #[props(default)] max: Option<String>,
    #[props(default)] step: Option<String>,
    #[props(default)] value: Option<Signal<String>>,
    #[props(default)] static_value: Option<String>,
    #[props(default)] on_change: Option<EventHandler<String>>,
    #[props(default)] onchange: Option<EventHandler<FormEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] inputmode: Option<String>,
    #[props(default)] aria_invalid: Option<String>,
    #[props(default)] aria_describedby: Option<String>,
    #[props(default)] unstyled: bool,
    #[props(default)] enterkeyhint: Option<String>,
    #[props(default)] has_actions: bool,
) -> Element {
    let merged_class = if unstyled {
        class
    } else {
        let base_class = size.standalone_full(has_actions);
        if class.is_empty() {
            base_class
        } else {
            merge(&[&base_class, &class])
        }
    };

    let type_str = r#type.as_ref();
    // Floating-label hack needs a non-empty placeholder; default to a static space.
    let actual_placeholder = placeholder.as_deref().unwrap_or(" ");
    let current_value = value.map(|s| s()).or(static_value).unwrap_or_default();

    // Autofocus waits for the AutofocusGate (when provided) so the keyboard
    // doesn't open mid-transition; without a gate it fires on mount.
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

    rsx! {
        input {
            "data-name": "Input",
            r#type: type_str,
            class: "{merged_class}",
            placeholder: actual_placeholder,
            onmounted: move |evt: MountedEvent| mounted.set(Some(evt.data())),
            name,
            id,
            title,
            disabled,
            readonly,
            required,
            min,
            max,
            step,
            inputmode,
            "aria-invalid": aria_invalid,
            "aria-describedby": aria_describedby,
            "enterkeyhint": enterkeyhint,
            value: "{current_value}",
            oninput: move |ev| {
                if let Some(mut signal) = value {
                    signal.set(ev.value());
                }
                if let Some(handler) = &on_change {
                    handler.call(ev.value());
                }
            },
            onchange: move |ev| {
                if let Some(handler) = &onchange {
                    handler.call(ev);
                }
            },
            onblur: move |ev| {
                if let Some(handler) = &onblur {
                    handler.call(ev);
                }
            },
            onkeydown: move |ev| {
                if let Some(handler) = &onkeydown {
                    handler.call(ev);
                }
            },
        }
    }
}

/// Standalone password input with a reveal toggle — the signal-bound sibling of
/// the form-bound `PasswordInput`. The trailing eye button swaps the control
/// between `Password` and `Text`; the input reserves room for it via `has_actions`.
#[component]
pub fn PasswordInputBase(
    #[props(default)] class: String,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] value: Option<Signal<String>>,
    #[props(default)] disabled: bool,
    #[props(default)] autofocus: bool,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
) -> Element {
    let mut revealed = use_signal(|| false);
    let input_type = if revealed() {
        InputType::Text
    } else {
        InputType::Password
    };
    rsx! {
        div { class: "relative w-full",
            InputBase {
                r#type: input_type,
                class,
                placeholder,
                value,
                disabled,
                autofocus,
                onkeydown,
                has_actions: true,
            }
            button {
                r#type: "button",
                tabindex: "-1",
                "aria-label": if revealed() { "Hide password" } else { "Show password" },
                class: "absolute end-2 top-1/2 -translate-y-1/2 grid place-items-center size-9 \
                        rounded-full text-muted-foreground hover:text-foreground transition-colors",
                onclick: move |_| revealed.toggle(),
                Icon {
                    name: if revealed() { IconName::EyeOff } else { IconName::Eye },
                    class: "size-5",
                }
            }
        }
    }
}
