use dioxus::prelude::*;
use utils::format::merge;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    #[allow(unused)]
    Vertical,
}

impl SeparatorOrientation {
    fn class(self) -> &'static str {
        match self {
            SeparatorOrientation::Horizontal => "w-full h-[1px]",
            SeparatorOrientation::Vertical => "h-full w-[1px]",
        }
    }

    fn aria_orientation(self) -> &'static str {
        match self {
            SeparatorOrientation::Horizontal => "horizontal",
            SeparatorOrientation::Vertical => "vertical",
        }
    }
}

#[component]
pub fn Separator(
    #[props(default)] orientation: Option<SeparatorOrientation>,
    /// Purely visual divider: announces as `role="none"` so assistive tech ignores it.
    #[props(default)]
    decorative: bool,
    #[props(default)] class: String,
) -> Element {
    let orientation = orientation.unwrap_or_default();
    let merged_class = merge(&["shrink-0 bg-border", orientation.class(), &class]);
    let aria_orientation = (!decorative).then(|| orientation.aria_orientation());

    rsx! {
        div {
            class: "{merged_class}",
            role: if decorative { "none" } else { "separator" },
            "aria-orientation": aria_orientation,
        }
    }
}
