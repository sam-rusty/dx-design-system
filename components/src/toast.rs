use dioxus::prelude::*;
use utils::format::merge;

use crate::hooks::use_timeout;
use crate::icon::{Icon, IconName};

/// Upper bound on simultaneously visible toasts; older toasts are trimmed first.
const MAX_TOASTS: usize = 5;

// ── Variant ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ToastVariant {
    #[default]
    Success,
    Error,
    Warning,
    Info,
}

impl ToastVariant {
    fn icon(self) -> IconName {
        match self {
            Self::Success => IconName::CheckCircle,
            Self::Error | Self::Warning | Self::Info => IconName::CircleAlert,
        }
    }

    fn icon_bg(self) -> &'static str {
        match self {
            Self::Success => "bg-success/10",
            Self::Error => "bg-destructive/10",
            Self::Warning => "bg-warning/10",
            Self::Info => "bg-primary/10",
        }
    }

    fn icon_color(self) -> &'static str {
        match self {
            Self::Success => "text-success",
            Self::Error => "text-destructive",
            Self::Warning => "text-warning",
            Self::Info => "text-primary",
        }
    }
}

// ── Placement ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ToastPlacement {
    #[default]
    BottomCenter,
    BottomLeft,
    BottomRight,
    TopCenter,
    TopLeft,
    TopRight,
}

impl ToastPlacement {
    fn container_class(self) -> &'static str {
        match self {
            Self::BottomCenter => "bottom-4 left-1/2 -translate-x-1/2 items-center flex-col",
            Self::BottomLeft => "bottom-4 left-4 items-start flex-col",
            Self::BottomRight => "bottom-4 right-4 items-end flex-col",
            Self::TopCenter => "top-4 left-1/2 -translate-x-1/2 items-center flex-col-reverse",
            Self::TopLeft => "top-4 left-4 items-start flex-col-reverse",
            Self::TopRight => "top-4 right-4 items-end flex-col-reverse",
        }
    }

    fn slide_class(self) -> &'static str {
        match self {
            Self::BottomCenter | Self::BottomLeft | Self::BottomRight => "slide-in-from-bottom-4",
            Self::TopCenter | Self::TopLeft | Self::TopRight => "slide-in-from-top-4",
        }
    }
}

// ── Store ─────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub struct ToastItem {
    pub id: u64,
    pub message: String,
    pub variant: ToastVariant,
}

/// Reactive store for the toast stack.
#[derive(Clone, Copy, PartialEq)]
pub struct ToastStore {
    items: Signal<Vec<ToastItem>>,
    next_id: Signal<u64>,
}

impl ToastStore {
    pub fn new() -> Self {
        Self {
            items: Signal::new_in_scope(Vec::new(), ScopeId::ROOT),
            next_id: Signal::new_in_scope(0, ScopeId::ROOT),
        }
    }

    pub fn push(mut self, message: String, variant: ToastVariant) {
        let id = *self.next_id.peek();
        *self.next_id.write() += 1;
        let mut items = self.items.write();
        items.push(ToastItem {
            id,
            message,
            variant,
        });
        if items.len() > MAX_TOASTS {
            items.remove(0);
        }
    }

    pub fn dismiss(mut self, id: u64) {
        self.items.write().retain(|t| t.id != id);
    }
}

impl Default for ToastStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_toast() -> ToastStore {
    use_context::<ToastStore>()
}

// ── Components ────────────────────────────────────────────────────────────────

#[component]
pub fn Toaster(
    #[props(default)] placement: ToastPlacement,
    #[props(default = 2500_i32)] dismiss_after_ms: i32,
) -> Element {
    let store = use_context::<ToastStore>();

    let container_class = merge(&[
        "fixed z-[200] flex gap-2 pointer-events-none",
        placement.container_class(),
    ]);

    let slide_class = placement.slide_class();
    let items = store.items.read().clone();

    rsx! {
        div { class: "{container_class}", role: "status", "aria-live": "polite", "aria-atomic": "false",
            for item in items {
                ToastItemView {
                    key: "{item.id}",
                    item: item,
                    store: store,
                    dismiss_after_ms: dismiss_after_ms,
                    slide_class: slide_class,
                }
            }
        }
    }
}

#[component]
fn ToastItemView(
    item: ToastItem,
    store: ToastStore,
    dismiss_after_ms: i32,
    slide_class: &'static str,
) -> Element {
    let id = item.id;
    let variant = item.variant;

    // Armed once on mount, cancelled on unmount or manual close.
    let cancel_dismiss = use_timeout(dismiss_after_ms.max(0) as u32, move || store.dismiss(id));

    let card_class = merge(&[
        "flex items-center gap-3 px-5 py-3.5 rounded-xl border border-border bg-card shadow-xl pointer-events-auto animate-in fade-in duration-300",
        slide_class,
    ]);
    let icon_wrapper_class = merge(&[
        "flex items-center justify-center size-7 rounded-full shrink-0",
        variant.icon_bg(),
    ]);
    let icon_class = merge(&["size-4", variant.icon_color()]);

    rsx! {
        div { class: "{card_class}",
            div { class: "{icon_wrapper_class}",
                Icon { name: variant.icon(), class: "{icon_class}" }
            }
            p { class: "text-sm font-medium text-foreground whitespace-nowrap",
                "{item.message}"
            }
            button {
                r#type: "button",
                "aria-label": "Dismiss",
                class: "ml-2 flex items-center justify-center text-muted-foreground hover:text-foreground cursor-pointer transition-colors",
                onclick: move |_| {
                    cancel_dismiss.call(());
                    store.dismiss(id);
                },
                Icon { name: IconName::X, class: "size-3" }
            }
        }
    }
}
