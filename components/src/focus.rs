/// Id of the document's currently focused element, or `None` when nothing is
/// focused, the focused element has no id, or on SSR.
#[cfg(feature = "web")]
pub fn active_element_id() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element())
        .map(|el| el.id())
        .filter(|id| !id.is_empty())
}

#[cfg(not(feature = "web"))]
pub fn active_element_id() -> Option<String> {
    None
}

/// Selector for the elements a Tab press can land on inside an overlay.
// Overlay-only helpers (Modal, Popover, DropdownMenu).
#[cfg(all(feature = "web", feature = "overlay"))]
pub(crate) const FOCUSABLE_SELECTOR: &str = "a[href], button:not([disabled]), input:not([disabled]), select, textarea, [tabindex]:not([tabindex=\"-1\"])";

/// Tabbable descendants of `root` in document order, skipping `hidden` ones.
// Overlay-only helpers (Modal, Popover, DropdownMenu).
#[cfg(all(feature = "web", feature = "overlay"))]
pub(crate) fn focusable_descendants(root: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    use wasm_bindgen::JsCast;

    let Ok(list) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|el| !el.hidden())
        .collect()
}

/// Focus the first tabbable descendant of `root`; returns whether one existed.
// Overlay-only helpers (Modal, Popover, DropdownMenu).
#[cfg(all(feature = "web", feature = "overlay"))]
pub(crate) fn focus_first_descendant(root: &web_sys::Element) -> bool {
    match focusable_descendants(root).first() {
        Some(el) => {
            let _ = el.focus();
            true
        }
        None => false,
    }
}

/// Keep Tab / Shift+Tab cycling inside `root`: when focus is on the last
/// (resp. first) tabbable descendant — or on `root` itself — move it to the
/// first (resp. last) one. Returns `true` when focus was moved, so the caller
/// cancels the browser's default move. A `root` with no tabbable descendants
/// never steals focus, and neither does a key press while focus is *outside*
/// `root`'s DOM subtree: a `Portal`ed child overlay (PopoverConfirm inside a
/// Modal) bubbles its keydown to the modal by VDOM ancestry even though its DOM
/// lives elsewhere, and that Tab belongs to the child.
// Overlay-only helpers (Modal, Popover, DropdownMenu).
#[cfg(all(feature = "web", feature = "overlay"))]
pub(crate) fn trap_tab(root: &web_sys::Element, backwards: bool) -> bool {
    use wasm_bindgen::JsValue;

    let items = focusable_descendants(root);
    let Some(last) = items.len().checked_sub(1) else {
        return false;
    };
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());
    if !active.as_ref().is_some_and(|a| root.contains(Some(a))) {
        return false;
    }
    let current = active.as_ref().and_then(|a| {
        let av: &JsValue = a.as_ref();
        items.iter().position(|it| {
            let iv: &JsValue = it.as_ref();
            iv == av
        })
    });
    let target = match (backwards, current) {
        (false, Some(i)) if i < last => return false,
        (true, Some(i)) if i > 0 => return false,
        (false, _) => 0,
        (true, _) => last,
    };
    let _ = items[target].focus();
    true
}
