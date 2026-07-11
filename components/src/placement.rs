/// Side an anchored overlay panel opens toward, relative to its trigger
/// (popover, dropdown menu, tooltip, calendar panel, …).
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    /// No explicit side requested; resolves to [`Placement::Bottom`].
    ///
    /// This is a *default*, not collision-aware auto-flipping — there is no
    /// viewport measurement yet, so `Auto` simply means "open downward".
    #[default]
    Auto,
    Top,
    Bottom,
    Left,
    Right,
}

/// Cross-axis alignment of the panel against the trigger edge.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    /// Centered on the trigger (tooltips, popovers).
    #[default]
    Center,
    /// Aligned to the trigger's start edge (menus, calendar panels).
    Start,
}

/// A DOM rectangle in viewport coordinates (mirrors `web_sys::DomRect`).
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(target_arch = "wasm32")]
impl From<web_sys::DomRect> for Rect {
    fn from(r: web_sys::DomRect) -> Self {
        Rect {
            x: r.x(),
            y: r.y(),
            width: r.width(),
            height: r.height(),
        }
    }
}

/// Resolved fixed-position coordinates for an anchored panel, plus the side it
/// ended up on (after any flip) for the `data-side` attribute.
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Resolved {
    pub top: f64,
    pub left: f64,
    pub side: Placement,
}

/// Geometry inputs for placing a tooltip panel: the `trigger` and `panel` rects
/// in viewport coordinates, the `viewport` size `(width, height)`, and the
/// `gap` between trigger and panel.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) struct Anchor {
    pub trigger: Rect,
    pub panel: Rect,
    pub viewport: (f64, f64),
    pub gap: f64,
}

#[cfg(any(target_arch = "wasm32", test))]
impl Anchor {
    /// Resolve fixed coords for the panel on `preferred` side, flipping to the
    /// opposite side only when `preferred` would overflow the viewport. The
    /// cross-axis is centered on the trigger (no shift/clamp). `Auto` prefers
    /// `Bottom`.
    pub(crate) fn resolve(&self, preferred: Placement) -> Resolved {
        let t = &self.trigger;
        let p = &self.panel;
        let (vw, vh) = self.viewport;
        let g = self.gap;
        let centered_left = (t.x + t.width / 2.0) - p.width / 2.0;
        let centered_top = (t.y + t.height / 2.0) - p.height / 2.0;

        let preferred = if preferred == Placement::Auto {
            Placement::Bottom
        } else {
            preferred
        };

        match preferred {
            Placement::Top => {
                let top = t.y - p.height - g;
                if top < 0.0 {
                    Resolved {
                        top: t.y + t.height + g,
                        left: centered_left,
                        side: Placement::Bottom,
                    }
                } else {
                    Resolved {
                        top,
                        left: centered_left,
                        side: Placement::Top,
                    }
                }
            }
            Placement::Bottom => {
                let top = t.y + t.height + g;
                if top + p.height > vh {
                    Resolved {
                        top: t.y - p.height - g,
                        left: centered_left,
                        side: Placement::Top,
                    }
                } else {
                    Resolved {
                        top,
                        left: centered_left,
                        side: Placement::Bottom,
                    }
                }
            }
            Placement::Left => {
                let left = t.x - p.width - g;
                if left < 0.0 {
                    Resolved {
                        top: centered_top,
                        left: t.x + t.width + g,
                        side: Placement::Right,
                    }
                } else {
                    Resolved {
                        top: centered_top,
                        left,
                        side: Placement::Left,
                    }
                }
            }
            Placement::Right => {
                let left = t.x + t.width + g;
                if left + p.width > vw {
                    Resolved {
                        top: centered_top,
                        left: t.x - p.width - g,
                        side: Placement::Left,
                    }
                } else {
                    Resolved {
                        top: centered_top,
                        left,
                        side: Placement::Right,
                    }
                }
            }
            Placement::Auto => unreachable!("Auto is mapped to Bottom above"),
        }
    }
}

impl Placement {
    /// Tailwind position classes for this side + alignment, computed once so
    /// popover / dropdown / tooltip don't each re-`match` the placement.
    pub fn classes(self, align: Align) -> &'static str {
        use Align::{Center, Start};
        use Placement::{Auto, Bottom, Left, Right, Top};
        match (self, align) {
            (Top, Center) => "bottom-full left-1/2 -translate-x-1/2 mb-2",
            (Bottom | Auto, Center) => "top-full left-1/2 -translate-x-1/2 mt-2",
            (Left, Center) => "right-full top-1/2 -translate-y-1/2 mr-2",
            (Right, Center) => "left-full top-1/2 -translate-y-1/2 ml-2",
            (Top, Start) => "bottom-full left-0 mb-2",
            (Bottom | Auto, Start) => "top-full left-0 mt-2",
            (Left, Start) => "right-full top-0 mr-2",
            (Right, Start) => "left-full top-0 ml-2",
        }
    }

    /// Value for the `data-side` attribute (styling / animation origin hook).
    pub fn data_side(self) -> &'static str {
        match self {
            Placement::Top => "top",
            Placement::Bottom | Placement::Auto => "bottom",
            Placement::Left => "left",
            Placement::Right => "right",
        }
    }

    /// `transform-origin` Tailwind class so the open/close scale animation grows
    /// from the trigger-facing edge: a panel below the trigger (`Bottom`) expands
    /// from its top, a panel to the `Left` from its right edge, and so on.
    pub fn transform_origin(self) -> &'static str {
        match self {
            Placement::Top => "origin-bottom",
            Placement::Bottom | Placement::Auto => "origin-top",
            Placement::Left => "origin-right",
            Placement::Right => "origin-left",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Align, Anchor, Placement, Rect};

    #[test]
    fn auto_resolves_to_bottom() {
        assert_eq!(
            Placement::Auto.classes(Align::Center),
            Placement::Bottom.classes(Align::Center)
        );
        assert_eq!(Placement::Auto.data_side(), "bottom");
    }

    #[test]
    fn alignment_picks_distinct_classes() {
        assert_ne!(
            Placement::Bottom.classes(Align::Center),
            Placement::Bottom.classes(Align::Start)
        );
    }

    #[test]
    fn data_side_matches_named_placement() {
        assert_eq!(Placement::Top.data_side(), "top");
        assert_eq!(Placement::Left.data_side(), "left");
        assert_eq!(Placement::Right.data_side(), "right");
    }

    fn anchor(trigger: Rect, panel: Rect) -> Anchor {
        Anchor {
            trigger,
            panel,
            viewport: (1000.0, 800.0),
            gap: 8.0,
        }
    }

    fn mid_trigger() -> Rect {
        Rect {
            x: 400.0,
            y: 400.0,
            width: 100.0,
            height: 20.0,
        }
    }

    fn panel() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 40.0,
        }
    }

    #[test]
    fn top_places_above_and_centers_when_room() {
        let r = anchor(mid_trigger(), panel()).resolve(Placement::Top);
        assert_eq!(r.side, Placement::Top);
        assert_eq!(r.top, 352.0); // 400 - 40 - 8
        assert_eq!(r.left, 390.0); // cx 450 - panel_w/2 60
    }

    #[test]
    fn top_flips_to_bottom_when_no_room_above() {
        let trigger = Rect {
            x: 400.0,
            y: 10.0,
            width: 100.0,
            height: 20.0,
        };
        let r = anchor(trigger, panel()).resolve(Placement::Top);
        assert_eq!(r.side, Placement::Bottom);
        assert_eq!(r.top, 38.0); // 10 + 20 + 8
    }

    #[test]
    fn bottom_places_below_when_room() {
        let r = anchor(mid_trigger(), panel()).resolve(Placement::Bottom);
        assert_eq!(r.side, Placement::Bottom);
        assert_eq!(r.top, 428.0); // 400 + 20 + 8
    }

    #[test]
    fn bottom_flips_to_top_when_no_room_below() {
        let trigger = Rect {
            x: 400.0,
            y: 770.0,
            width: 100.0,
            height: 20.0,
        };
        let r = anchor(trigger, panel()).resolve(Placement::Bottom);
        assert_eq!(r.side, Placement::Top);
        assert_eq!(r.top, 722.0); // 770 - 40 - 8
    }

    #[test]
    fn left_places_and_centers_vertically_when_room() {
        let r = anchor(mid_trigger(), panel()).resolve(Placement::Left);
        assert_eq!(r.side, Placement::Left);
        assert_eq!(r.left, 272.0); // 400 - 120 - 8
        assert_eq!(r.top, 390.0); // cy 410 - panel_h/2 20
    }

    #[test]
    fn left_flips_to_right_when_no_room() {
        let trigger = Rect {
            x: 10.0,
            y: 400.0,
            width: 100.0,
            height: 20.0,
        };
        let r = anchor(trigger, panel()).resolve(Placement::Left);
        assert_eq!(r.side, Placement::Right);
        assert_eq!(r.left, 118.0); // 10 + 100 + 8
    }

    #[test]
    fn right_flips_to_left_when_no_room() {
        let trigger = Rect {
            x: 900.0,
            y: 400.0,
            width: 100.0,
            height: 20.0,
        };
        let r = anchor(trigger, panel()).resolve(Placement::Right);
        assert_eq!(r.side, Placement::Left);
        assert_eq!(r.left, 772.0); // 900 - 120 - 8
    }

    #[test]
    fn auto_behaves_like_bottom() {
        let auto = anchor(mid_trigger(), panel()).resolve(Placement::Auto);
        let bottom = anchor(mid_trigger(), panel()).resolve(Placement::Bottom);
        assert_eq!(auto.side, bottom.side);
        assert_eq!(auto.top, bottom.top);
        assert_eq!(auto.left, bottom.left);
    }
}
