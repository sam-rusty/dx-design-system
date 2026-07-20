use dioxus::prelude::*;

use crate::hooks::use_unique_id;

/// Props for the Portal component
#[derive(Props, Clone, PartialEq)]
pub struct PortalProps {
    /// The content to be portaled
    children: Element,

    /// Id of the container element the content is moved into (defaults to the
    /// app's `main` element). Matched with `getElementById`, so pass the bare id
    /// (`"main"`, `"root"`), not a CSS selector.
    #[props(default = "main".to_string())]
    container: String,

    /// Optional class name for the portal wrapper div
    #[props(default)]
    class: Option<String>,

    /// Optional id for the portal wrapper div (defaults to a generated unique id)
    #[props(default)]
    id: Option<String>,
}

/// Portal component that renders children into a different part of the DOM tree,
/// equivalent to `ReactDOM.createPortal`.
///
/// Used to lift overlays (e.g. [`Modal`](crate::Modal)) out of any ancestor that
/// establishes a containing block (a `transform`/`filter`) so `position: fixed`
/// resolves against the viewport.
///
/// # Implementation
/// On mount a marker comment is planted at the wrapper's rendered slot and the
/// wrapper element is physically moved into `container` via `appendChild`; on
/// unmount the wrapper is moved back to the marker so Dioxus's positional
/// removal edit (`replaceWith` placeholder — the anchor later diffs insert
/// relative to) applies at the real slot, not wherever the overlay was parked.
/// The wrapper id comes from [`use_unique_id`] — stable across an SSR/hydration
/// pair, unlike the previous `Math.random()` id.
#[component]
pub fn Portal(props: PortalProps) -> Element {
    let mut mounted = use_signal(|| false);
    let gen_id = use_unique_id();

    // The relocated node plus a marker comment left at its original slot.
    // Dioxus keeps addressing the wrapper by element id (inner updates work
    // anywhere in the DOM), but its *unmount* edit — `replaceWith(placeholder)`
    // — is positional: the placeholder becomes the fragment anchor later diffs
    // insert relative to. So on drop the wrapper must be moved back to the
    // marker BEFORE Dioxus's removal edit flushes (scope drops run during the
    // diff, DOM edits flush after), or the placeholder lands detached and the
    // next route swap renders into nowhere.
    #[cfg(target_arch = "wasm32")]
    let moved = use_hook(|| {
        std::rc::Rc::new(std::cell::RefCell::new(
            None::<(web_sys::Element, web_sys::Comment)>,
        ))
    });

    #[cfg(target_arch = "wasm32")]
    use_drop({
        let moved = moved.clone();
        move || {
            if let Some((node, marker)) = moved.borrow_mut().take() {
                match marker.parent_node() {
                    Some(parent) => {
                        let _ = parent.insert_before(&node, Some(&marker));
                        marker.remove();
                    }
                    // Original slot is gone (whole subtree already torn down) —
                    // detaching keeps the overlay from leaking into #main.
                    None => node.remove(),
                }
            }
        }
    });

    // Render to the portal target only after the first client render.
    use_effect(move || mounted.set(true));
    if !mounted() {
        return rsx! {};
    }

    let wrapper_id = props.id.clone().unwrap_or_else(|| gen_id.peek().clone());
    let container = props.container.clone();

    rsx! {
        div {
            class: props.class.clone(),
            id: wrapper_id,
            style: "position: fixed; z-index: 9999; inset: 0;",
            onmounted: move |evt| {
                #[cfg(target_arch = "wasm32")]
                {
                    let container = container.clone();
                    let moved = moved.clone();
                    if let Some(el) = evt.downcast::<web_sys::Element>()
                        && let Some(document) = web_sys::window().and_then(|w| w.document())
                        && let Some(target) = document.get_element_by_id(&container)
                    {
                        let marker = document.create_comment("portal");
                        let planted = el
                            .parent_node()
                            .is_some_and(|p| p.insert_before(&marker, Some(el)).is_ok());
                        if planted && target.append_child(el).is_ok() {
                            *moved.borrow_mut() = Some((el.clone(), marker));
                        } else {
                            marker.remove();
                        }
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = (&evt, &container);
            },

            {props.children}
        }
    }
}
