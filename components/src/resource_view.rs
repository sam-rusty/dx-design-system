use dioxus::prelude::*;
use utils::DsError;

/// Renders the four standard states of a `use_resource` result in one place:
/// loading (`skeleton`), error (`error`), empty (`empty`), and loaded (`view`).
///
/// Collapses the repeated
/// `match resource.value()() { None => skeleton, Some(Err) => err, Some(Ok) if empty => …, Some(Ok) => … }`
/// shape. The loaded value is handed to `is_empty` to choose between `empty` and `view`.
///
/// ```rust, ignore
/// ResourceView {
///     resource,
///     skeleton: rsx! { NotesSkeleton {} },
///     error: move |e| rsx! { "Could not load: {e}" },
///     empty: rsx! { EmptyState { message: "No notes yet" } },
///     is_empty: |notes: Vec<Note>| notes.is_empty(),
///     view: move |notes: Vec<Note>| rsx! { /* content */ },
/// }
/// ```
#[component]
pub fn ResourceView<T: Clone + PartialEq + 'static>(
    resource: Resource<Result<T, DsError>>,
    /// Shown while the resource is loading (value is `None`).
    skeleton: Element,
    /// Shown when the resource resolved to `Err`.
    error: Callback<DsError, Element>,
    /// Shown when the loaded value is empty (per `is_empty`).
    empty: Element,
    /// Returns `true` when the loaded value should render `empty` instead of `view`.
    is_empty: Callback<T, bool>,
    /// Renders the loaded, non-empty value.
    view: Callback<T, Element>,
) -> Element {
    match resource.value()() {
        None => skeleton,
        Some(Err(e)) => error.call(e),
        Some(Ok(data)) if is_empty.call(data.clone()) => empty,
        Some(Ok(data)) => view.call(data),
    }
}
