use std::sync::atomic::{AtomicU32, Ordering};

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

/// Bytes read from a selected or dropped file.
#[derive(Clone, Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: usize,
    pub mime_type: String,
    pub data: Vec<u8>,
}

/// File upload zone with drag-and-drop support.
///
/// Renders a dashed drop-zone by default. Pass `children` to use a custom trigger
/// (wrapped in a `<label>` that opens the file picker on the bubbled click). A custom
/// trigger must let the click propagate — calling `stop_propagation` on it (or a child)
/// prevents the picker from opening.
///
/// `on_change` fires once all selected/dropped files have been read into memory.
/// `max_size_mb: 0` means no limit. Oversized files fire `on_error` if provided.
#[component]
pub fn FileUpload(
    on_change: EventHandler<Vec<FileInfo>>,
    #[props(default)] accept: Option<&'static str>,
    #[props(default)] multiple: bool,
    /// Client-side size limit in megabytes. 0 = no limit.
    #[props(default)]
    max_size_mb: usize,
    #[props(default)] disabled: bool,
    /// Called for each rejected (oversized) file with a human-readable message.
    #[props(default)]
    on_error: Option<EventHandler<String>>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let mut dragging = use_signal(|| false);

    // Signals declared at component scope — Dioxus requires hooks at top level.
    let mut collected: Signal<Vec<FileInfo>> = use_signal(Vec::new);
    let mut done_count: Signal<usize> = use_signal(|| 0usize);
    // Generation counter to discard stale callbacks from a previous batch.
    let mut batch_gen: Signal<u32> = use_signal(|| 0u32);

    // Shared file-reading logic; only called from web event handlers.
    // Both signals are Copy so they are captured by copy into the closure.
    #[cfg(feature = "web")]
    let mut process_file_list = move |file_list: web_sys::FileList| {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;

        let count = file_list.length();
        if count == 0 {
            return;
        }

        let files: Vec<web_sys::File> = (0..count).filter_map(|i| file_list.get(i)).collect();

        let (ok_files, rejected): (Vec<_>, Vec<_>) = if max_size_mb == 0 {
            (files, vec![])
        } else {
            let limit = (max_size_mb * 1024 * 1024) as f64;
            files.into_iter().partition(|f| f.size() <= limit)
        };

        for f in &rejected {
            let msg = format!(
                "'{}' exceeds the {max_size_mb} MB limit and was not uploaded.",
                f.name()
            );
            if let Some(cb) = on_error {
                cb.call(msg);
            }
        }

        let total = ok_files.len();
        if total == 0 {
            return;
        }

        // Increment generation so in-flight callbacks from a previous batch become stale.
        let this_gen = *batch_gen.read() + 1;
        batch_gen.set(this_gen);
        *collected.write() = Vec::with_capacity(total);
        done_count.set(0);

        for file in ok_files {
            let name = file.name();
            let size = file.size() as usize;
            let mime_type = file.type_();

            let reader = web_sys::FileReader::new().unwrap();
            let reader_clone = reader.clone();

            // `once_into_js` frees the Rust closure after the single `onload` fires,
            // so there is no per-file leak (unlike `Closure::wrap(..).forget()`).
            let onload = Closure::once_into_js(move || {
                // Discard if a newer batch started.
                if *batch_gen.read() != this_gen {
                    return;
                }
                if let Ok(result) = reader_clone.result() {
                    let Ok(array_buf) = result.dyn_into::<js_sys::ArrayBuffer>() else {
                        return;
                    };
                    let bytes = js_sys::Uint8Array::new(&array_buf).to_vec();
                    collected.write().push(FileInfo {
                        name,
                        size,
                        mime_type,
                        data: bytes,
                    });
                    *done_count.write() += 1;
                    let next = *done_count.read();
                    if next == total {
                        on_change.call(collected.read().clone());
                    }
                }
            });

            reader.set_onload(Some(onload.unchecked_ref()));
            let _ = reader.read_as_array_buffer(&file);
        }
    };

    let input_id = use_hook(|| {
        let n = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("file-upload-input-{n}")
    });

    let zone_class = if disabled {
        "relative flex flex-col items-center justify-center w-full h-36 rounded-xl \
         border-2 border-dashed border-border/40 bg-muted/10 opacity-50 cursor-not-allowed"
    } else if dragging() {
        "relative flex flex-col items-center justify-center w-full h-36 rounded-xl \
         border-2 border-dashed border-primary bg-primary/5 scale-[1.01] transition-all cursor-pointer"
    } else {
        "relative flex flex-col items-center justify-center w-full h-36 rounded-xl \
         border-2 border-dashed border-border/60 hover:border-border hover:bg-muted/20 \
         transition-all cursor-pointer"
    };

    let input_id_for_input = input_id.clone();
    let input_id_for_custom = input_id.clone();

    rsx! {
        div { class: "w-full",
            // Hidden native file input — FileUpload IS the abstraction layer over it.
            input {
                id: input_id_for_input,
                r#type: "file",
                class: "sr-only",
                "aria-label": if multiple { "Upload files" } else { "Upload file" },
                accept: accept.unwrap_or(""),
                multiple,
                disabled,
                onchange: move |e: Event<FormData>| {
                    #[cfg(feature = "web")]
                    {
                        use dioxus::web::WebEventExt;
                        use wasm_bindgen::JsCast;
                        let native = e.as_web_event();
                        let target = native.target().unwrap();
                        let input: web_sys::HtmlInputElement = target.unchecked_into();
                        if let Some(files) = input.files() {
                            process_file_list(files);
                        }
                        input.set_value("");
                    }
                    #[cfg(not(feature = "web"))]
                    let _ = e;
                },
            }

            if let Some(custom_trigger) = children {
                // A nested interactive trigger (e.g. a `<button>`) swallows the implicit
                // `<label for>` activation, so the picker never opens. Open it explicitly
                // on the bubbled click instead of relying on `for`.
                label {
                    class: "cursor-pointer",
                    onclick: move |_e| {
                        #[cfg(feature = "web")]
                        if !disabled {
                            use wasm_bindgen::JsCast;
                            if let Some(doc) = web_sys::window().and_then(|w| w.document())
                                && let Some(el) = doc.get_element_by_id(&input_id_for_custom)
                                && let Ok(input) = el.dyn_into::<web_sys::HtmlElement>()
                            {
                                input.click();
                            }
                        }
                        #[cfg(not(feature = "web"))]
                        let _ = &input_id_for_custom;
                    },
                    {custom_trigger}
                }
            } else {
                label {
                    r#for: input_id,
                    class: zone_class,
                    ondragover: move |e: Event<DragData>| {
                        e.prevent_default();
                        if !disabled { dragging.set(true); }
                    },
                    ondragleave: move |_| dragging.set(false),
                    ondrop: move |e: Event<DragData>| {
                        e.prevent_default();
                        dragging.set(false);
                        #[cfg(feature = "web")]
                        if !disabled {
                            use dioxus::web::WebEventExt;
                            let native = e.as_web_event();
                            if let Some(dt) = native.data_transfer()
                                && let Some(files) = dt.files() {
                                    process_file_list(files);
                                }
                        }
                        #[cfg(not(feature = "web"))]
                        let _ = e;
                    },
                    Icon {
                        name: IconName::FileUp,
                        class: "size-6 text-muted-foreground/50 mb-2 pointer-events-none",
                        stroke_width: 1.5,
                    }
                    span { class: "text-sm text-muted-foreground pointer-events-none",
                        "Drop a file or "
                        span { class: "text-primary font-medium", "browse" }
                    }
                    if let Some(a) = accept {
                        span { class: "mt-1 text-xs text-muted-foreground/60 pointer-events-none",
                            "Accepted: {a}"
                        }
                    }
                }
            }
        }
    }
}
