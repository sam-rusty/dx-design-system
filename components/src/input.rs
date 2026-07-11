use dioxus::prelude::*;
use strum_macros::AsRefStr;
use utils::format::merge;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum InputSize {
    #[default]
    Default,
    Sm,
    Xs,
}

impl InputSize {
    /// Complete standalone [`InputBase`] class (root + file-picker + interaction states).
    /// The whole matrix is static, so this is a single `&'static str` — callers only
    /// allocate when they pass an extra `class`.
    fn standalone_full(self) -> &'static str {
        match self {
            Self::Default => {
                "peer flex w-full h-12 min-w-0 rounded-lg border border-input bg-transparent \
                 px-4 py-2 text-sm text-foreground transition-all duration-200 outline-none \
                 placeholder:text-transparent focus:placeholder:text-muted-foreground/70 \
                 file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm \
                 file:font-medium focus:border-primary focus:ring-1 focus:ring-primary \
                 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 \
                 disabled:bg-muted/50 aria-invalid:border-destructive \
                 aria-invalid:ring-destructive/20 focus:aria-invalid:border-destructive \
                 focus:aria-invalid:ring-1 read-only:bg-muted/50"
            }
            Self::Sm => {
                "peer flex w-full h-8 min-w-0 rounded-lg border border-input bg-transparent \
                 px-3 py-1 text-xs text-foreground transition-all duration-200 outline-none \
                 placeholder:text-transparent focus:placeholder:text-muted-foreground/70 \
                 file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-xs \
                 file:font-medium focus:border-primary focus:ring-1 focus:ring-primary \
                 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 \
                 disabled:bg-muted/50 aria-invalid:border-destructive \
                 aria-invalid:ring-destructive/20 focus:aria-invalid:border-destructive \
                 focus:aria-invalid:ring-1 read-only:bg-muted/50"
            }
            Self::Xs => {
                "peer flex w-full h-7 min-w-0 rounded-lg border border-input bg-transparent \
                 px-2 py-0.5 text-xs text-foreground transition-all duration-200 outline-none \
                 placeholder:text-transparent focus:placeholder:text-muted-foreground/70 \
                 file:inline-flex file:h-5 file:border-0 file:bg-transparent file:text-xs \
                 file:font-medium focus:border-primary focus:ring-1 focus:ring-primary \
                 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 \
                 disabled:bg-muted/50 aria-invalid:border-destructive \
                 aria-invalid:ring-destructive/20 focus:aria-invalid:border-destructive \
                 focus:aria-invalid:ring-1 read-only:bg-muted/50"
            }
        }
    }

    /// Floating-label form `Input` control (`peer block`).
    pub fn form_floating_peer_merge(self, has_actions: bool) -> String {
        let base = match self {
            Self::Default => {
                "peer block w-full h-12 appearance-none rounded-lg border border-input \
                 bg-transparent py-2 text-sm text-foreground transition-colors \
                 focus-visible:border-primary focus-visible:outline-none \
                 focus-visible:ring-1 focus-visible:ring-primary \
                 data-[invalid=true]:border-destructive"
            }
            Self::Sm => {
                "peer block w-full h-8 appearance-none rounded-lg border border-input \
                 bg-transparent py-1 text-xs text-foreground transition-colors \
                 focus-visible:border-primary focus-visible:outline-none \
                 focus-visible:ring-1 focus-visible:ring-primary \
                 data-[invalid=true]:border-destructive"
            }
            Self::Xs => {
                "peer block w-full h-7 appearance-none rounded-lg border border-input \
                 bg-transparent py-0.5 text-xs text-foreground transition-colors \
                 focus-visible:border-primary focus-visible:outline-none \
                 focus-visible:ring-1 focus-visible:ring-primary \
                 data-[invalid=true]:border-destructive"
            }
        };
        let padding = match (self, has_actions) {
            (_, true) => match self {
                Self::Default => "pl-4 pr-16",
                Self::Sm => "pl-3 pr-14",
                Self::Xs => "pl-2 pr-12",
            },
            (Self::Default, false) => "px-4",
            (Self::Sm, false) => "px-3",
            (Self::Xs, false) => "px-2",
        };
        merge(&[base, padding])
    }

    /// Full class string when [`InputFormControl`] receives an empty override (matches legacy default).
    pub fn form_control_fallback_merge(self) -> String {
        match self {
            Self::Default => {
                "peer block w-full h-12 appearance-none rounded-lg border border-input \
                 bg-transparent px-4 py-2 text-sm text-foreground transition-colors \
                 focus-visible:border-primary focus-visible:outline-none \
                 focus-visible:ring-1 focus-visible:ring-primary \
                 data-[invalid=true]:border-destructive"
                    .into()
            }
            Self::Sm => "peer block w-full h-8 appearance-none rounded-lg border border-input \
                 bg-transparent px-3 py-1 text-xs text-foreground transition-colors \
                 focus-visible:border-primary focus-visible:outline-none \
                 focus-visible:ring-1 focus-visible:ring-primary \
                 data-[invalid=true]:border-destructive"
                .into(),
            Self::Xs => "peer block w-full h-7 appearance-none rounded-lg border border-input \
                 bg-transparent px-2 py-0.5 text-xs text-foreground transition-colors \
                 focus-visible:border-primary focus-visible:outline-none \
                 focus-visible:ring-1 focus-visible:ring-primary \
                 data-[invalid=true]:border-destructive"
                .into(),
        }
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
) -> Element {
    let base_class = size.standalone_full();
    let merged_class = if class.is_empty() {
        base_class.to_string()
    } else {
        merge(&[base_class, &class])
    };

    let type_str = r#type.as_ref();
    // Floating-label hack needs a non-empty placeholder; default to a static space.
    let actual_placeholder = placeholder.as_deref().unwrap_or(" ");
    let current_value = value.map(|s| s()).or(static_value).unwrap_or_default();

    rsx! {
        input {
            "data-name": "Input",
            r#type: type_str,
            class: "{merged_class}",
            placeholder: actual_placeholder,
            onmounted: move |evt: MountedEvent| {
                if autofocus {
                    let md = evt.data();
                    spawn(async move {
                        let _ = md.set_focus(true).await;
                    });
                }
            },
            name: name,
            id: id,
            title: title,
            disabled: disabled,
            readonly: readonly,
            required: required,
            autofocus: autofocus,
            min: min,
            max: max,
            step: step,
            inputmode: inputmode,
            "aria-invalid": aria_invalid,
            "aria-describedby": aria_describedby,
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
