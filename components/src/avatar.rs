use dioxus::prelude::*;
use ds_utils::format::merge;

const BASE_CLASS: &str = "flex items-center justify-center overflow-hidden h-10 w-10 rounded-full bg-primary/15 text-primary select-none";

/// Avatar chip. Renders `src` as an image when provided, falling back to the
/// `children` (typically initials) if the image fails to load or no `src` is
/// given. When `alt` is set the container is exposed to AT as `role="img"`.
#[component]
pub fn Avatar(
    #[props(default)] class: String,
    #[props(default)] style: String,
    #[props(default)] src: Option<String>,
    #[props(default)] alt: Option<String>,
    children: Element,
) -> Element {
    let mut errored = use_signal(|| false);

    let class = merge(&[BASE_CLASS, &class]);
    let has_src = src.as_deref().is_some_and(|s| !s.is_empty());
    let show_img = has_src && !errored();
    let label = alt.clone().filter(|a| !a.is_empty());

    rsx! {
        div {
            class,
            style,
            role: label.is_some().then_some("img"),
            "aria-label": label,
            if show_img {
                img {
                    class: "h-full w-full object-cover",
                    src: src.unwrap_or_default(),
                    alt: "",
                    "aria-hidden": "true",
                    onerror: move |_| errored.set(true),
                }
            } else {
                {children}
            }
        }
    }
}
