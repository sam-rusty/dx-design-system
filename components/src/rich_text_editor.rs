// `rsx!` lowers `attr: if a { .. } else { .. }` into an expression whose layout
// trips `suspicious_else_formatting`; the source is rustfmt-clean.
#![allow(clippy::suspicious_else_formatting)]

use dioxus::prelude::*;

use crate::hooks::use_prop_signal;
use ds_macros::on_web;
use ds_utils::format::merge;

on_web! {
    use dioxus::web::WebEventExt;
}

use crate::field_name::Field;
use crate::form::dynamic::{FormContext, FormError, FormField};
use crate::form::{FieldContext, FieldLabel};
#[cfg(feature = "web")]
use crate::icon::{Icon, IconName};
#[cfg(feature = "web")]
use crate::input::{InputBase, InputType};
#[cfg(feature = "web")]
use crate::placement::Placement;
#[cfg(feature = "web")]
use crate::popover::Popover;

// ── Web-only JS: formatting commands + save/restore selection ─────────────────

on_web! {
    mod js {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(inline_js = r#"
            let _rte_saved_range = null;

            export function rteExecCmd(command) {
                document.execCommand(command, false, null);
            }
            export function rteExecCmdValue(command, value) {
                document.execCommand(command, false, value);
            }
            export function rteQueryCmd(command) {
                return document.queryCommandState(command);
            }
            export function rteSaveSelection() {
                const sel = window.getSelection();
                _rte_saved_range = (sel && sel.rangeCount > 0)
                    ? sel.getRangeAt(0).cloneRange()
                    : null;
            }
            export function rteRestoreSelection() {
                if (!_rte_saved_range) return;
                const sel = window.getSelection();
                sel.removeAllRanges();
                sel.addRange(_rte_saved_range);
            }
            export function rteSetFontSize(px) {
                const sel = window.getSelection();
                if (!sel || sel.rangeCount === 0) return;
                const range = sel.getRangeAt(0);
                if (range.collapsed) return;
                const fragment = range.extractContents();
                const span = document.createElement('span');
                span.style.fontSize = px;
                span.appendChild(fragment);
                range.insertNode(span);
                const newRange = document.createRange();
                newRange.selectNodeContents(span);
                sel.removeAllRanges();
                sel.addRange(newRange);
            }
            export function rteInsertText(elementId, text) {
                const el = document.getElementById(elementId);
                if (!el) return;
                const sel = window.getSelection();
                const inEl = sel && sel.rangeCount > 0 && el.contains(sel.anchorNode);
                if (!inEl) {
                    el.focus();
                    const range = document.createRange();
                    range.selectNodeContents(el);
                    range.collapse(false);
                    sel.removeAllRanges();
                    sel.addRange(range);
                }
                document.execCommand('insertText', false, text);
            }
        "#)]
        extern "C" {
            pub fn rteExecCmd(command: &str);
            pub fn rteExecCmdValue(command: &str, value: &str);
            pub fn rteQueryCmd(command: &str) -> bool;
            pub fn rteSaveSelection();
            pub fn rteRestoreSelection();
            pub fn rteSetFontSize(px: &str);
            pub fn rteInsertText(element_id: &str, text: &str);
        }
    }
}

// ── Thin helpers over JS ──────────────────────────────────────────────────────

on_web! {
    fn exec_cmd(cmd: &str) {
        js::rteExecCmd(cmd);
    }

    fn exec_cmd_value(cmd: &str, value: &str) {
        js::rteExecCmdValue(cmd, value);
    }

    fn query_cmd(cmd: &str) -> bool {
        js::rteQueryCmd(cmd)
    }

    fn set_font_size(px: &str) {
        js::rteSetFontSize(px);
    }
}

/// Insert plain text at the editor's current caret position.
///
/// `element_id` must match the [`RichTextEditorBase`] `id` prop. The caret is
/// preserved only when the trigger calls `prevent_default()` on its `mousedown`
/// (so focus stays in the editor); otherwise the text is appended at the end.
/// No-op on SSR.
#[cfg(feature = "web")]
pub fn rte_insert_text(element_id: &str, text: &str) {
    js::rteInsertText(element_id, text);
}

#[cfg(not(feature = "web"))]
pub fn rte_insert_text(_element_id: &str, _text: &str) {}

// ── Link sanitising ───────────────────────────────────────────────────────────

/// Allowlist the scheme of a user-entered link before it is written into the
/// editor's HTML (E1): `http(s):`/`mailto:` pass through, scheme-less input is
/// promoted to `https://`, anything else (`javascript:`, `data:`, …) is rejected.
#[cfg_attr(not(feature = "web"), allow(dead_code))]
pub(crate) fn safe_href(input: &str) -> Option<String> {
    let t = input.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
    {
        return Some(t.to_string());
    }
    // Any other explicit scheme (`javascript:`, `data:`, `vbscript:` …) is dropped.
    // A colon after the first `/` is a path/port, not a scheme.
    let head = t.split('/').next().unwrap_or(t);
    if head.contains(':') && !is_host_port(head) {
        return None;
    }
    Some(format!("https://{t}"))
}

/// `host:1234` — a colon followed only by digits is a port, not a scheme.
#[cfg_attr(not(feature = "web"), allow(dead_code))]
fn is_host_port(head: &str) -> bool {
    match head.split_once(':') {
        Some((host, port)) => {
            !host.is_empty() && !port.is_empty() && port.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

// ── Toolbar button ────────────────────────────────────────────────────────────

#[component]
fn ToolbarBtn(
    active: bool,
    title: String,
    onclick: EventHandler<()>,
    /// `true` (default): a toggle → `aria-pressed`; `false`: a popover trigger →
    /// `aria-haspopup` + `aria-expanded`.
    #[props(default = true)]
    toggle: bool,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            title: "{title}",
            "aria-label": "{title}",
            "aria-pressed": toggle.then(|| active.to_string()),
            "aria-haspopup": (!toggle).then_some("true"),
            "aria-expanded": (!toggle).then(|| active.to_string()),
            disabled,
            class: if active {
                "flex items-center justify-center size-6 rounded cursor-pointer \
                 bg-accent text-accent-foreground transition-colors"
            } else {
                "flex items-center justify-center size-6 rounded cursor-pointer \
                 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
            },
            onmousedown: |e| e.prevent_default(),
            onclick: move |_| onclick.call(()),
            {children}
        }
    }
}

// ── Base component ────────────────────────────────────────────────────────────

/// Raw contenteditable rich text editor. Use [`RichTextEditor`] for form-integrated usage.
#[component]
pub fn RichTextEditorBase(
    #[props(default)] class: String,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] autofocus: bool,
    /// Show bold button (default true).
    #[props(default = true)]
    show_bold: bool,
    /// Show italic button (default true).
    #[props(default = true)]
    show_italic: bool,
    /// Show font-size dropdown (default true).
    #[props(default = true)]
    show_font_size: bool,
    /// Show text-color picker (default true).
    #[props(default = true)]
    show_color: bool,
    /// Show alignment buttons (default true).
    #[props(default = true)]
    show_align: bool,
    /// Show link button (default true).
    #[props(default = true)]
    show_link: bool,
    /// HTML content signal. Read on mount; written on each input event.
    #[props(default)]
    value: Option<Signal<String>>,
    /// Fires with the current innerHTML on each input event.
    #[props(default)]
    on_change: Option<EventHandler<String>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] aria_invalid: Option<String>,
    #[props(default)] aria_describedby: Option<String>,
    /// Accessible name for the `role="textbox"` content (a `<label for>` cannot
    /// target a `div`). Form usage passes the field label.
    #[props(default)]
    aria_label: Option<String>,
    /// Inline mode: no border/background on wrapper; toolbar floats above the content.
    /// Pass `content_class` + `content_style` to match the surrounding element's visual state.
    #[props(default)]
    inline: bool,
    /// Extra classes applied to the contenteditable div. In inline mode these replace the
    /// default sizing classes; in normal mode they are appended.
    #[props(default)]
    content_class: String,
    /// Inline style applied to the contenteditable div.
    #[props(default)]
    content_style: String,
) -> Element {
    // SSR fallback — return before web-only code
    #[cfg(not(feature = "web"))]
    {
        let val = value.map(|s| s.peek().clone()).unwrap_or_default();
        return rsx! {
            textarea {
                class: merge(&[
                    "peer flex w-full min-h-[80px] min-w-0 rounded-lg border border-input \
                     bg-transparent px-4 py-2 text-sm text-foreground",
                    &class,
                ]),
                value: "{val}",
                id: id,
                disabled: disabled,
                "aria-invalid": aria_invalid,
                "aria-describedby": aria_describedby,
                "aria-label": aria_label,
            }
        };
    }

    // ── Web-only implementation ───────────────────────────────────────────────

    #[cfg(feature = "web")]
    {
        let mut editor_ref: Signal<Option<web_sys::HtmlElement>> = use_signal(|| None);
        let mut is_bold = use_signal(|| false);
        let mut is_italic = use_signal(|| false);
        let mut is_focused = use_signal(|| false);
        let mut is_empty = use_signal(move || value.map(|s| s.peek().is_empty()).unwrap_or(true));
        // Tracks the last HTML value that came from user input or external sync.
        // Used to distinguish user-driven changes from external signal changes.
        let mut last_user_html: Signal<String> =
            use_signal(move || value.map(|s| s.peek().clone()).unwrap_or_default());
        let mut link_open = use_signal(|| false);
        let mut link_url = use_signal(String::new);
        let link_url_value: ReadSignal<Option<String>> = use_memo(move || Some(link_url())).into();
        let mut color_open = use_signal(|| false);
        let mut font_size_open = use_signal(|| false);

        // Empty means "no visible text": browsers leave `<div><br></div>` / `<br>`
        // behind after select-all + delete, so the HTML alone is not a reliable test.
        fn editor_is_empty(el: &web_sys::HtmlElement) -> bool {
            el.inner_text().trim().is_empty()
        }

        // Sync external value changes (e.g. async template loading) into the editor.
        use_effect(move || {
            let new_val = value.map(|s| s()).unwrap_or_default();
            if new_val != *last_user_html.peek() {
                *last_user_html.write() = new_val.clone();
                if let Some(el) = editor_ref.peek().as_ref() {
                    el.set_inner_html(&new_val);
                    is_empty.set(editor_is_empty(el));
                }
            }
        });

        let mut update_format_state = move || {
            is_bold.set(query_cmd("bold"));
            is_italic.set(query_cmd("italic"));
        };

        let outer_class = if inline {
            merge(&["relative", &class])
        } else {
            merge(&[
                "relative rounded-lg border text-sm text-foreground transition-all duration-200 peer",
                if aria_invalid.as_deref() == Some("true") {
                    "border-destructive ring-1 ring-destructive/20"
                } else if is_focused() {
                    "border-primary ring-1 ring-primary"
                } else {
                    "border-input"
                },
                &class,
            ])
        };

        rsx! {
            div {
                "data-name": "RichTextEditor",
                class: "{outer_class}",
                "aria-invalid": aria_invalid.clone(),
                "aria-describedby": aria_describedby.clone(),
                onfocusin: move |_| is_focused.set(true),
                onfocusout: move |_| is_focused.set(false),

                // ── Toolbar ───────────────────────────────────────────────
                div {
                    class: if inline {
                        "absolute bottom-full left-0 z-50 mb-1 \
                         flex items-center gap-0.5 px-2 py-1 \
                         bg-card border border-border rounded-lg shadow-md flex-wrap"
                    } else {
                        "flex items-center gap-0.5 px-2 py-1.5 border-b border-input flex-wrap"
                    },
                    // Group 1: bold / italic
                    if show_bold {
                        ToolbarBtn {
                            active: is_bold(),
                            title: "Bold",
                            disabled,
                            onclick: move |_| {
                                exec_cmd("bold");
                                is_bold.set(query_cmd("bold"));
                            },
                            Icon { name: IconName::Bold, class: "size-3.5" }
                        }
                    }
                    if show_italic {
                        ToolbarBtn {
                            active: is_italic(),
                            title: "Italic",
                            disabled,
                            onclick: move |_| {
                                exec_cmd("italic");
                                is_italic.set(query_cmd("italic"));
                            },
                            Icon { name: IconName::Italic, class: "size-3.5" }
                        }
                    }
                    // Separator: G1 → G2
                    if (show_bold || show_italic)
                        && (show_font_size || show_color || show_align || show_link)
                    {
                        div { class: "w-px h-4 bg-border mx-0.5" }
                    }
                    // Group 2: font-size / color
                    if show_font_size {
                        Popover {
                            open: Some(font_size_open()),
                            on_open_change: move |v| font_size_open.set(v),
                            toggle_on_click: false,
                            placement: Placement::Bottom,
                            trigger: rsx! {
                                ToolbarBtn {
                                    active: font_size_open(),
                                    title: "Font Size",
                                    toggle: false,
                                    disabled,
                                    onclick: move |_| {
                                        js::rteSaveSelection();
                                        *font_size_open.write() ^= true;
                                    },
                                    span { class: "text-xs font-bold leading-none", "A" }
                                }
                            },
                            div { class: "grid grid-cols-4 gap-1 p-1 w-max",
                                for (px , label) in [
                                    ("10px", "10"),
                                    ("12px", "12"),
                                    ("14px", "14"),
                                    ("16px", "16"),
                                    ("18px", "18"),
                                    ("24px", "24"),
                                    ("32px", "32"),
                                    ("48px", "48"),
                                ] {
                                    button {
                                        r#type: "button",
                                        title: "{label}px",
                                        class: "flex items-center justify-center h-7 rounded text-xs \
                                                text-muted-foreground hover:bg-accent \
                                                hover:text-accent-foreground cursor-pointer transition-colors",
                                        onmousedown: |e| e.prevent_default(),
                                        onclick: move |_| {
                                            js::rteRestoreSelection();
                                            set_font_size(px);
                                            *font_size_open.write() = false;
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                    if show_color {
                        Popover {
                            open: Some(color_open()),
                            on_open_change: move |v| color_open.set(v),
                            toggle_on_click: false,
                            placement: Placement::Bottom,
                            trigger: rsx! {
                                ToolbarBtn {
                                    active: color_open(),
                                    title: "Text Color",
                                    toggle: false,
                                    disabled,
                                    onclick: move |_| {
                                        js::rteSaveSelection();
                                        *color_open.write() ^= true;
                                    },
                                    Icon { name: IconName::Palette, class: "size-3.5" }
                                }
                            },
                            div { class: "grid grid-cols-4 gap-1 p-1 w-max",
                                for (color, label) in [
                                    ("#000000", "Black"),
                                    ("#374151", "Dark gray"),
                                    ("#6b7280", "Gray"),
                                    ("#d1d5db", "Light gray"),
                                    ("#ef4444", "Red"),
                                    ("#f97316", "Orange"),
                                    ("#eab308", "Yellow"),
                                    ("#22c55e", "Green"),
                                    ("#3b82f6", "Blue"),
                                    ("#8b5cf6", "Purple"),
                                    ("#ec4899", "Pink"),
                                    ("#06b6d4", "Cyan"),
                                    ("#14b8a6", "Teal"),
                                    ("#a3e635", "Lime"),
                                    ("#f59e0b", "Amber"),
                                    ("#ffffff", "White"),
                                ] {
                                    button {
                                        r#type: "button",
                                        title: "{label}",
                                        class: "size-5 rounded cursor-pointer border border-border \
                                                hover:scale-110 transition-transform",
                                        style: "background-color: {color}",
                                        onclick: move |_| {
                                            js::rteRestoreSelection();
                                            exec_cmd_value("foreColor", color);
                                            *color_open.write() = false;
                                        },
                                    }
                                }
                            }
                        }
                    }
                    // Separator: G2 → G3/G4 (only when G2 has items)
                    if (show_font_size || show_color) && (show_align || show_link) {
                        div { class: "w-px h-4 bg-border mx-0.5" }
                    }
                    // Group 3: alignment
                    if show_align {
                        ToolbarBtn {
                            active: false,
                            title: "Align Left",
                            disabled,
                            onclick: move |_| exec_cmd("justifyLeft"),
                            Icon { name: IconName::AlignLeft, class: "size-3.5" }
                        }
                        ToolbarBtn {
                            active: false,
                            title: "Align Center",
                            disabled,
                            onclick: move |_| exec_cmd("justifyCenter"),
                            Icon { name: IconName::AlignCenter, class: "size-3.5" }
                        }
                        ToolbarBtn {
                            active: false,
                            title: "Align Right",
                            disabled,
                            onclick: move |_| exec_cmd("justifyRight"),
                            Icon { name: IconName::AlignRight, class: "size-3.5" }
                        }
                    }
                    // Separator: G3 → G4
                    if show_align && show_link {
                        div { class: "w-px h-4 bg-border mx-0.5" }
                    }
                    // Group 4: link
                    if show_link {
                        Popover {
                            open: Some(link_open()),
                            on_open_change: move |v| link_open.set(v),
                            toggle_on_click: false,
                            placement: Placement::Bottom,
                            trigger: rsx! {
                                ToolbarBtn {
                                    active: link_open(),
                                    title: "Insert Link",
                                    toggle: false,
                                    disabled,
                                    onclick: move |_| {
                                        js::rteSaveSelection();
                                        *link_open.write() ^= true;
                                    },
                                    Icon { name: IconName::LinkIcon, class: "size-3.5" }
                                }
                            },
                            div { class: "flex flex-col gap-2 w-56",
                                InputBase {
                                    r#type: InputType::Url,
                                    size: crate::input::FieldSize::Sm,
                                    placeholder: "https://example.com",
                                    value: link_url_value,
                                    on_value_change: move |v: String| link_url.set(v),
                                }
                                div { class: "flex justify-end gap-1.5",
                                    button {
                                        r#type: "button",
                                        class: "text-xs px-2.5 py-1 rounded-md hover:bg-accent \
                                                text-muted-foreground cursor-pointer transition-colors",
                                        onclick: move |_| {
                                            js::rteRestoreSelection();
                                            exec_cmd("unlink");
                                            *link_open.write() = false;
                                            link_url.set(String::new());
                                        },
                                        "Remove link"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "text-xs px-2.5 py-1 rounded-md bg-primary \
                                                text-primary-foreground hover:bg-primary/90 \
                                                cursor-pointer transition-colors",
                                        onclick: move |_| {
                                            if let Some(url) = safe_href(&link_url.peek()) {
                                                js::rteRestoreSelection();
                                                exec_cmd_value("createLink", &url);
                                            }
                                            *link_open.write() = false;
                                            link_url.set(String::new());
                                        },
                                        "Apply"
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Editable area ──────────────────────────────────────────
                div { class: "relative",
                    if !inline && is_empty() {
                        if let Some(ref ph) = placeholder {
                            span {
                                class: "absolute left-4 top-2 text-sm text-muted-foreground/70 \
                                        pointer-events-none select-none",
                                "{ph}"
                            }
                        }
                    }
                    div {
                        "data-name": "RichTextEditorContent",
                        role: "textbox",
                        "aria-multiline": "true",
                        "aria-label": aria_label.clone(),
                        "aria-disabled": if disabled { "true" },
                        class: if inline {
                            merge(&["outline-none", &content_class])
                        } else {
                            merge(&["min-h-[80px] px-4 py-2 text-sm outline-none", &content_class])
                        },
                        style: "{content_style}",
                        contenteditable: if disabled { "false" } else { "true" },
                        id: id.clone(),
                        autofocus: autofocus,
                        onmounted: move |e| {
                            use wasm_bindgen::JsCast;
                            let web_el = e.as_web_event();
                            if let Ok(el) = web_el.dyn_into::<web_sys::HtmlElement>() {
                                let html = value.map(|s| s.peek().clone()).unwrap_or_default();
                                el.set_inner_html(&html);
                                is_empty.set(editor_is_empty(&el));
                                *editor_ref.write() = Some(el);
                            }
                        },
                        oninput: move |_| {
                            if let Some(el) = editor_ref.peek().as_ref() {
                                let html = el.inner_html();
                                *last_user_html.write() = html.clone();
                                is_empty.set(editor_is_empty(el));
                                is_bold.set(query_cmd("bold"));
                                is_italic.set(query_cmd("italic"));
                                if let Some(mut sig) = value {
                                    sig.set(html.clone());
                                }
                                if let Some(ref cb) = on_change {
                                    cb.call(html);
                                }
                            }
                        },
                        onkeyup: move |_| update_format_state(),
                        onmouseup: move |_| update_format_state(),
                        onblur: move |e| {
                            if let Some(ref cb) = onblur {
                                cb.call(e);
                            }
                        },
                    }
                }
            }
        }
    }
}

// ── Form-integrated control ───────────────────────────────────────────────────

#[component]
pub(crate) fn RichTextEditorFormControl(
    #[props(default)] autofocus: bool,
    #[props(default)] aria_label: Option<String>,
    #[props(default = true)] show_bold: bool,
    #[props(default = true)] show_italic: bool,
    #[props(default = true)] show_font_size: bool,
    #[props(default = true)] show_color: bool,
    #[props(default = true)] show_align: bool,
    #[props(default = true)] show_link: bool,
) -> Element {
    let field_ctx = use_context::<FieldContext>();
    let field_name = field_ctx.name.clone();
    let form_ctx = use_context::<FormContext>();

    let id = String::from(&*field_name);
    let aria_describedby = format!("{}-error", field_name);

    let is_disabled = form_ctx.disabled.is_some_and(|d| *d.read());

    let is_touched = form_ctx.aux.with(|a| a.is_touched(&field_name));
    let has_error = form_ctx.aux.with(|a| a.error(&field_name).is_some());
    let aria_invalid = if is_touched && has_error {
        Some("true".to_string())
    } else {
        None
    };

    let value = form_ctx
        .values_signal
        .with(|v| v.get(&*field_name).cloned().unwrap_or_default());
    // Compare-and-set: an unconditional write every render would notify every
    // subscriber (and the editor's sync effect) even when nothing changed.
    let value_sig = use_prop_signal(value);

    rsx! {
        RichTextEditorBase {
            id: id,
            disabled: is_disabled,
            autofocus: autofocus,
            show_bold: show_bold,
            show_italic: show_italic,
            show_font_size: show_font_size,
            show_color: show_color,
            show_align: show_align,
            show_link: show_link,
            aria_invalid: aria_invalid,
            aria_describedby: aria_describedby,
            aria_label: aria_label,
            value: Some(value_sig),
            on_change: {
                let field_name = field_name.clone();
                EventHandler::new(move |html: String| {
                    form_ctx.set_value.read()(&field_name, html);
                })
            },
            onblur: {
                let field_name = field_name.clone();
                EventHandler::new(move |_: FocusEvent| {
                    form_ctx.touch_field.read()(&field_name);
                })
            },
        }
    }
}

// ── Public form-integrated component ─────────────────────────────────────────

/// Rich text editor with form integration. Mirrors the [`TextArea`] API.
/// Stores HTML content in the form field value.
///
/// [`TextArea`]: crate::textarea::TextArea
#[component]
pub fn RichTextEditor(
    #[props(into)] field: Field,
    #[props(default)] autofocus: bool,
    #[props(default = true)] show_bold: bool,
    #[props(default = true)] show_italic: bool,
    #[props(default = true)] show_font_size: bool,
    #[props(default = true)] show_color: bool,
    #[props(default = true)] show_align: bool,
    #[props(default = true)] show_link: bool,
) -> Element {
    let label = field.label.to_string();

    rsx! {
        FormField { field,
            FieldLabel { "{label}" }
            div { class: "relative w-full",
                RichTextEditorFormControl {
                    autofocus: autofocus,
                    aria_label: label.clone(),
                    show_bold: show_bold,
                    show_italic: show_italic,
                    show_font_size: show_font_size,
                    show_color: show_color,
                    show_align: show_align,
                    show_link: show_link,
                }
            }
            FormError {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::safe_href;

    #[test]
    fn safe_href_allows_http_https_mailto() {
        assert_eq!(
            safe_href("https://example.com/a?b=1").as_deref(),
            Some("https://example.com/a?b=1")
        );
        assert_eq!(safe_href("HTTP://x.y").as_deref(), Some("HTTP://x.y"));
        assert_eq!(safe_href(" mailto:a@b.c ").as_deref(), Some("mailto:a@b.c"));
    }

    #[test]
    fn safe_href_rejects_script_and_data_schemes() {
        assert_eq!(safe_href("javascript:alert(1)"), None);
        assert_eq!(safe_href("JavaScript:alert(1)"), None);
        assert_eq!(safe_href(" data:text/html;base64,AAAA"), None);
        assert_eq!(safe_href("vbscript:x"), None);
        assert_eq!(safe_href(""), None);
        assert_eq!(safe_href("   "), None);
    }

    #[test]
    fn safe_href_promotes_bare_hosts_to_https() {
        assert_eq!(
            safe_href("example.com/path:with:colons").as_deref(),
            Some("https://example.com/path:with:colons")
        );
        assert_eq!(
            safe_href("localhost:8080/x").as_deref(),
            Some("https://localhost:8080/x")
        );
        assert_eq!(safe_href("foo:bar"), None);
    }
}
