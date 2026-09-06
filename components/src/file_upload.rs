#[cfg(feature = "web")]
use std::cell::RefCell;
#[cfg(feature = "web")]
use std::rc::Rc;

use dioxus::prelude::*;
#[cfg(feature = "web")]
use wasm_bindgen::closure::Closure;

use crate::hooks::use_unique_id;
use crate::icon::{Icon, IconName};

/// Bytes read from a selected or dropped file.
#[derive(Clone, Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: usize,
    pub mime_type: String,
    pub data: Vec<u8>,
}

/// In-flight read batch. Plain shared state (not signals): it is only touched
/// from `FileReader` callbacks, which may fire after the component unmounted —
/// a signal write there would panic, an `Rc` write is harmless.
#[cfg(feature = "web")]
#[derive(Default)]
struct Batch {
    /// Generation counter so callbacks from a superseded batch are ignored.
    generation: u32,
    total: usize,
    done: usize,
    collected: Vec<FileInfo>,
    /// Readers of the current batch with their `load`/`error` handler, aborted
    /// when a new batch starts or on unmount. Holding the `Closure` here (rather
    /// than `once_into_js`) means the handler that never fires — exactly one of
    /// load/error does per file — is freed with the batch instead of leaking.
    readers: Vec<PendingRead>,
}

#[cfg(feature = "web")]
type PendingRead = (web_sys::FileReader, Closure<dyn FnMut(web_sys::Event)>);

#[cfg(feature = "web")]
impl Batch {
    fn abort_readers(&mut self) {
        for (reader, _handler) in self.readers.drain(..) {
            reader.set_onload(None);
            reader.set_onerror(None);
            reader.abort();
        }
    }
}

/// File upload zone with drag-and-drop support.
///
/// Renders a dashed drop-zone by default. Pass `children` to use a custom trigger
/// (wrapped in a `<label>` that opens the file picker on the bubbled click). A custom
/// trigger must let the click propagate — calling `stop_propagation` on it (or a child)
/// prevents the picker from opening.
///
/// `on_change` fires once all selected/dropped files have been read into memory
/// (files that fail to read are skipped, so it still fires). `max_size_mb: 0` means
/// no limit. Oversized and unreadable files fire `on_error` if provided.
#[component]
pub fn FileUpload(
    on_change: EventHandler<Vec<FileInfo>>,
    #[props(default)] accept: Option<&'static str>,
    #[props(default)] multiple: bool,
    /// Client-side size limit in megabytes. 0 = no limit.
    #[props(default)]
    max_size_mb: usize,
    #[props(default)] disabled: bool,
    /// Called with a human-readable message for each rejected (oversized) file
    /// and for each file that could not be read.
    #[props(default)]
    on_error: Option<EventHandler<String>>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let mut dragging = use_signal(|| false);

    #[cfg(feature = "web")]
    let batch = use_hook(|| Rc::new(RefCell::new(Batch::default())));
    #[cfg(feature = "web")]
    use_drop({
        let batch = batch.clone();
        move || batch.borrow_mut().abort_readers()
    });

    // Shared file-reading logic; only called from web event handlers.
    #[cfg(feature = "web")]
    let process_file_list = {
        let batch = batch.clone();
        move |file_list: web_sys::FileList| {
            use wasm_bindgen::JsCast;

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

            // Start a new generation: in-flight readers of the previous batch are
            // aborted and their callbacks (if any still land) discarded.
            let this_gen = {
                let mut b = batch.borrow_mut();
                b.abort_readers();
                b.generation = b.generation.wrapping_add(1);
                b.total = total;
                b.done = 0;
                b.collected = Vec::with_capacity(total);
                b.generation
            };

            // Count one file as finished (read or failed) and fire `on_change`
            // when the batch completes.
            let finish_one = {
                let batch = batch.clone();
                move |generation: u32, file: Option<FileInfo>| {
                    let mut b = batch.borrow_mut();
                    if b.generation != generation {
                        return;
                    }
                    if let Some(file) = file {
                        b.collected.push(file);
                    }
                    b.done += 1;
                    if b.done == b.total {
                        let files = std::mem::take(&mut b.collected);
                        b.readers.clear();
                        drop(b);
                        on_change.call(files);
                    }
                }
            };

            for file in ok_files {
                let name = file.name();
                let size = file.size() as usize;
                let mime_type = file.type_();

                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();

                // One handler serves both `load` and `error`; the batch owns it
                // (see `Batch::readers`) so whichever event does not fire is
                // still freed. Dropping it from inside its own invocation (the
                // batch completing) is safe: wasm-bindgen defers the free until
                // the call returns.
                let handler = {
                    let finish_one = finish_one.clone();
                    let name = name.clone();
                    Closure::wrap(Box::new(move |ev: web_sys::Event| {
                        let file = (ev.type_() == "load")
                            .then(|| reader_clone.result().ok())
                            .flatten()
                            .and_then(|r| r.dyn_into::<js_sys::ArrayBuffer>().ok())
                            .map(|array_buf| FileInfo {
                                name: name.clone(),
                                size,
                                mime_type: mime_type.clone(),
                                data: js_sys::Uint8Array::new(&array_buf).to_vec(),
                            });
                        if file.is_none()
                            && let Some(cb) = on_error
                        {
                            cb.call(format!("'{name}' could not be read."));
                        }
                        finish_one(this_gen, file);
                    }) as Box<dyn FnMut(web_sys::Event)>)
                };

                reader.set_onload(Some(handler.as_ref().unchecked_ref()));
                reader.set_onerror(Some(handler.as_ref().unchecked_ref()));
                batch.borrow_mut().readers.push((reader.clone(), handler));
                let _ = reader.read_as_array_buffer(&file);
            }
        }
    };

    let input_id = use_unique_id();
    let input_id = input_id.peek().clone();

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
    #[cfg(feature = "web")]
    let process_on_change = process_file_list.clone();
    #[cfg(feature = "web")]
    let process_on_drop = process_file_list;

    rsx! {
        div { class: "w-full",
            // Hidden native file input — FileUpload IS the abstraction layer over it.
            input {
                id: input_id_for_input,
                r#type: "file",
                class: "sr-only",
                "aria-label": if multiple { "Upload files" } else { "Upload file" },
                accept,
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
                            process_on_change(files);
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
                                    process_on_drop(files);
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
