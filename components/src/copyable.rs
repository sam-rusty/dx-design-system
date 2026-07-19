#[cfg(feature = "web")]
use std::cell::RefCell;
#[cfg(feature = "web")]
use std::rc::Rc;

use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::icon::{Icon, IconName};

/// A re-armable, cancel-on-unmount window timer used to flip the "Copied!" state
/// back after a delay. Replaces the leaked `closure::once_into_js` timer that
/// could fire `set_copied` on an already-dropped signal.
#[cfg(feature = "web")]
#[derive(Default)]
struct ResetTimer {
    handle: Option<i32>,
    _closure: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

#[cfg(feature = "web")]
impl ResetTimer {
    fn cancel(&mut self) {
        if let Some(handle) = self.handle.take()
            && let Some(win) = web_sys::window()
        {
            win.clear_timeout_with_handle(handle);
        }
        self._closure = None;
    }
}

#[cfg(feature = "web")]
fn write_clipboard(text: &str) {
    if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
        let _ = clipboard.write_text(text);
    }
}

/// Best-effort clipboard write that flips `set_copied` true and back after 1.5s.
///
/// Retained for form-field copy actions (`form/view`); [`Copyable`] uses its own
/// cancel-on-unmount timer instead of this fire-and-forget variant.
#[cfg_attr(not(feature = "form"), allow(dead_code))]
pub(crate) fn copy_to_clipboard(text: String, mut set_copied: Signal<bool>) {
    #[cfg(feature = "web")]
    {
        use wasm_bindgen::JsCast;
        write_clipboard(&text);
        set_copied.set(true);
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || set_copied.set(false));
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1500,
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
    let mut copied = use_signal(|| false);

    #[cfg(feature = "web")]
    let timer = use_hook(|| Rc::new(RefCell::new(ResetTimer::default())));
    #[cfg(feature = "web")]
    use_drop({
        let timer = timer.clone();
        move || timer.borrow_mut().cancel()
    });

    let text_clone = text.clone();
    let on_click = move |_| {
        let val = text_clone.clone();
        if val.is_empty() {
            return;
        }
        #[cfg(feature = "web")]
        {
            use wasm_bindgen::JsCast;
            write_clipboard(&val);
            copied.set(true);
            let mut timer_ref = timer.borrow_mut();
            timer_ref.cancel();
            let closure = wasm_bindgen::closure::Closure::once(move || copied.set(false));
            if let Some(win) = web_sys::window() {
                timer_ref.handle = win
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        1500,
                    )
                    .ok();
            }
            timer_ref._closure = Some(closure);
        }
        #[cfg(not(feature = "web"))]
        {
            copied.set(true);
        }
    };

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
        }
    }
}
