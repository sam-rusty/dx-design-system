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
