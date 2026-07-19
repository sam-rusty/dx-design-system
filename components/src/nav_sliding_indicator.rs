//! Sliding bar positioned from [`web_sys::Element`] geometry — shared by [`NavTabs`](super::nav_tabs::NavTabs)
//! and any custom vertical/horizontal nav that cannot use `NavTabs` directly.
//!
//! The active segment spans **~35%** of the nav item’s width (horizontal) or height (vertical), centered on the item.

/// Which axis the indicator animates along (matches [`super::nav_tabs::NavTabsDirection`] bar behavior).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlidingIndicatorAxis {
    Horizontal,
    Vertical,
}

/// Tailwind classes for the horizontal sliding underline (`NavTabs` top menu).
pub const HORIZONTAL_SLIDING_INDICATOR_CLASS: &str = "absolute bottom-0 left-0 h-[2px] w-0 bg-primary opacity-0 transition-[left,width,opacity] duration-300 ease-out z-20 pointer-events-none will-change-[left,width,opacity]";

/// Tailwind classes for the vertical sliding bar (`NavTabs` vertical / sidebar-style).
pub const VERTICAL_SLIDING_INDICATOR_CLASS: &str = "absolute left-[2px] top-0 w-[2px] h-0 bg-primary opacity-0 transition-[top,height,opacity] duration-300 ease-out z-20 pointer-events-none will-change-[top,height,opacity]";

#[must_use]
pub fn sliding_indicator_class(axis: SlidingIndicatorAxis) -> &'static str {
    match axis {
        SlidingIndicatorAxis::Horizontal => HORIZONTAL_SLIDING_INDICATOR_CLASS,
        SlidingIndicatorAxis::Vertical => VERTICAL_SLIDING_INDICATOR_CLASS,
    }
}

#[cfg(feature = "web")]
const INDICATOR_ITEM_FRACTION: f64 = 0.45;

/// Inline `style` for the active tab’s indicator (WASM only; uses layout rects).
#[cfg(feature = "web")]
#[must_use]
pub fn sliding_indicator_style(
    axis: SlidingIndicatorAxis,
    nav: &web_sys::Element,
    label: &web_sys::Element,
) -> String {
    let nav_rect = nav.get_bounding_client_rect();
    let label_rect = label.get_bounding_client_rect();
    match axis {
        SlidingIndicatorAxis::Horizontal => {
            let item_w = label_rect.width();
            let width = item_w * INDICATOR_ITEM_FRACTION;
            let left = label_rect.left() - nav_rect.left() + (item_w - width) * 0.5;
            format!("left:{left}px;width:{width}px;opacity:1")
        }
        SlidingIndicatorAxis::Vertical => {
            let item_h = label_rect.height();
            let height = item_h * INDICATOR_ITEM_FRACTION;
            let top = label_rect.top() - nav_rect.top() + (item_h - height) * 0.5;
            format!("top:{top}px;height:{height}px;opacity:1")
        }
    }
}
