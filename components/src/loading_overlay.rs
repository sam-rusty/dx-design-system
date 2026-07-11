use dioxus::prelude::*;

use crate::spinner::Spinner;

#[component]
pub fn LoadingOverlay(
    when: ReadSignal<bool>,
    #[props(default = "Processing\u{2026}")] message: &'static str,
) -> Element {
    // Lock background scroll while the overlay covers the viewport.
    #[cfg(feature = "web")]
    {
        use_effect(move || set_body_scroll_locked(when()));
        use_drop(|| set_body_scroll_locked(false));
    }

    if !when() {
        return rsx! {};
    }

    rsx! {
        div {
            role: "status",
            "aria-live": "polite",
            "aria-busy": "true",
            tabindex: "-1",
            class: "fixed inset-0 z-[200] flex items-center justify-center bg-background/75 backdrop-blur-sm animate-in fade-in duration-150 outline-none",
            onmounted: move |e| {
                #[cfg(feature = "web")]
                {
                    use dioxus::web::WebEventExt;
                    use wasm_bindgen::JsCast;
                    if let Ok(el) = e.as_web_event().dyn_into::<web_sys::HtmlElement>() {
                        let _ = el.focus();
                    }
                }
                #[cfg(not(feature = "web"))]
                {
                    let _ = e;
                }
            },
            div { class: "flex flex-col items-center gap-3 rounded-2xl border border-border bg-card shadow-2xl px-12 py-8",
                Spinner { class: "size-10 text-primary" }
                p { class: "text-sm font-medium text-foreground", "{message}" }
            }
        }
    }
}

#[cfg(feature = "web")]
fn set_body_scroll_locked(locked: bool) {
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        let list = body.class_list();
        let _ = if locked {
            list.add_1("overflow-hidden")
        } else {
            list.remove_1("overflow-hidden")
        };
    }
}
