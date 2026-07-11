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
/// On mount the wrapper element is physically moved into `container` via
/// `appendChild`, and detached again on unmount (the move takes it out of the
/// subtree Dioxus reconciles, so it must be cleaned up explicitly). The wrapper
/// id comes from [`use_unique_id`] — stable across an SSR/hydration pair, unlike
/// the previous `Math.random()` id.
#[component]
pub fn Portal(props: PortalProps) -> Element {
    let mut mounted = use_signal(|| false);
    let gen_id = use_unique_id();

    // The relocated node, kept so it can be detached when this component drops —
    // Dioxus no longer owns it once we move it out of its rendered parent.
    #[cfg(target_arch = "wasm32")]
    let moved = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None::<web_sys::Element>)));

    #[cfg(target_arch = "wasm32")]
    use_drop({
        let moved = moved.clone();
        move || {
            if let Some(node) = moved.borrow_mut().take() {
                node.remove();
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
                        && let Some(target) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.get_element_by_id(&container))
                        && target.append_child(el).is_ok()
                    {
                        *moved.borrow_mut() = Some(el.clone());
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = (&evt, &container);
            },

            {props.children}
        }
    }
}
