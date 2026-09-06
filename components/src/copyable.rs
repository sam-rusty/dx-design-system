use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::hooks::use_timer;
use crate::icon::{Icon, IconName};

/// How long the "Copied!" state stays on before flipping back.
const COPIED_RESET_MS: u32 = 1500;

#[cfg(feature = "web")]
fn write_clipboard(text: &str) {
    if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
        let _ = clipboard.write_text(text);
    }
}

/// Copy-to-clipboard feedback state: `copied` flips true on [`CopyFeedback::copy`]
/// and back after [`COPIED_RESET_MS`]. The reset timer is re-armable and cancelled
/// on unmount (via `hooks::use_timer`), so it can never write a dropped signal.
#[derive(Clone, Copy)]
pub(crate) struct CopyFeedback {
    pub(crate) copied: Signal<bool>,
    copy: Callback<String>,
}

impl CopyFeedback {
    /// Write `text` to the clipboard (no-op for empty text) and show feedback.
    pub(crate) fn copy(&self, text: String) {
        self.copy.call(text);
    }
}

/// Hook-shaped replacement for [`copy_to_clipboard`]: owns the `copied` signal and
/// the reset timer. Form-field copy actions should adopt this and drop their own
/// `copied` signal.
#[cfg_attr(not(feature = "form"), allow(dead_code))]
pub(crate) fn use_copy_feedback() -> CopyFeedback {
    let mut copied = use_signal(|| false);
    let timer = use_timer(COPIED_RESET_MS, move || copied.set(false));
    let copy = use_callback(move |text: String| {
        if text.is_empty() {
            return;
        }
        #[cfg(feature = "web")]
        write_clipboard(&text);
        copied.set(true);
        timer.start();
    });
    CopyFeedback { copied, copy }
}

/// Best-effort clipboard write that flips `set_copied` true and back after 1.5s.
///
/// Hook-free variant retained for the form-field copy actions (`form/view`). The
/// reset callback is fire-and-forget, so it tolerates the caller having unmounted
/// in the meantime (`try_write` instead of `write`). Prefer [`use_copy_feedback`]
/// in new code.
#[cfg_attr(not(feature = "form"), allow(dead_code))]
pub(crate) fn copy_to_clipboard(text: String, mut set_copied: Signal<bool>) {
    #[cfg(feature = "web")]
    {
        use wasm_bindgen::JsCast;
        write_clipboard(&text);
        set_copied.set(true);
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
            // The owning scope may be gone by now; a dropped signal is not an error.
            if let Ok(mut w) = set_copied.try_write() {
                *w = false;
            }
        });
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                COPIED_RESET_MS as i32,
            );
        }
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = text;
        set_copied.set(true);
    }
}

#[component]
pub fn Copyable(text: String, #[props(default)] class: String) -> Element {
    let feedback = use_copy_feedback();
    let copied = feedback.copied;

    let text_clone = text.clone();
    let on_click = move |_| feedback.copy(text_clone.clone());

    let btn_class = merge(&[
        "inline-flex items-center gap-1.5 cursor-pointer group transition-colors",
        &class,
    ]);

    let icon_name = if copied() {
        IconName::CopyCheck
    } else {
        IconName::Copy
    };
    let title_text = if copied() { "Copied!" } else { "Copy" };

    rsx! {
        button {
            r#type: "button",
            title: "{title_text}",
            "aria-label": "Copy {text}",
            class: "{btn_class}",
            onclick: on_click,
            Icon {
                name: icon_name,
                class: "size-3.5 shrink-0 text-muted-foreground group-hover:text-foreground transition-colors",
            }
            span { "{text}" }
            // Announce the state change; `title` alone is not read by screen readers.
            span { role: "status", class: "sr-only",
                if copied() { "Copied" }
            }
        }
    }
}
