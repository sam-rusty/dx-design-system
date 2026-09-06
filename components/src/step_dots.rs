use dioxus::prelude::*;
use ds_utils::format::merge;

/// Active-dot class as a literal so `stepper::view` can `concat!` its transition onto it.
macro_rules! dot_active {
    () => {
        "h-1.5 w-6 rounded-full bg-primary"
    };
}
pub(crate) use dot_active;

/// Dot classes shared with `StepProgressVariant::Dots` (`stepper/view.rs`).
pub(crate) const DOT_ACTIVE: &str = dot_active!();
pub(crate) const DOT_ACTIVE_FAILED: &str = "h-1.5 w-6 rounded-full bg-destructive";
macro_rules! dot_inactive {
    () => {
        "h-1.5 w-1.5 rounded-full bg-muted"
    };
}
pub(crate) use dot_inactive;
pub(crate) const DOT_INACTIVE: &str = dot_inactive!();

/// Marker for step-enum identity. Blanket-implemented; you never write this by hand.
/// (Named `StepKey` to avoid colliding with the multi-step-form `StepId`.)
pub trait StepKey: Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static {}
impl<T> StepKey for T where T: Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static {}

/// Metadata for a small dot-progress wizard enum. The `Steps` derive produces the
/// inherent consts; forward them into this trait with a small impl block:
///
/// ```ignore
/// impl StepMeta for MyStep {
///     const ALL: &'static [Self] = Self::ALL;
///     const COUNT: usize = Self::COUNT;
///     const TITLES: &'static [&'static str] = Self::TITLES;
///     const DESCRIPTIONS: &'static [&'static str] = Self::DESCRIPTIONS;
/// }
/// ```
///
/// (Named `StepMeta` to avoid colliding with the multi-step-form `StepDefinition`.)
pub trait StepMeta: StepKey + Sized {
    const ALL: &'static [Self];
    const COUNT: usize;
    const TITLES: &'static [&'static str];
    const DESCRIPTIONS: &'static [&'static str];

    fn try_ordinal(self) -> Option<usize> {
        Self::ALL.iter().position(|s| *s == self)
    }

    fn ordinal(self) -> usize {
        self.try_ordinal().unwrap_or(0)
    }

    fn title(self) -> &'static str {
        Self::TITLES
            .get(self.ordinal())
            .copied()
            .unwrap_or_default()
    }

    fn description(self) -> &'static str {
        Self::DESCRIPTIONS
            .get(self.ordinal())
            .copied()
            .unwrap_or_default()
    }

    fn initial() -> Self {
        Self::ALL[0]
    }
}

/// Minimal dot progress for a step wizard. Active step is a wide pill; the rest
/// are small dots. When `failed`, the active dot turns destructive.
#[component]
pub fn StepDots<S: StepMeta>(
    current: S,
    #[props(default)] failed: bool,
    #[props(default)] class: String,
) -> Element {
    let active = current.ordinal();
    let container = merge(&["flex items-center justify-center gap-1.5", &class]);
    rsx! {
        div {
            "data-name": "StepDots",
            class: container,
            role: "list",
            "aria-label": "Progress",
            for i in 0..S::COUNT {
                {
                    let is_active = i == active;
                    let dot = if is_active {
                        if failed { DOT_ACTIVE_FAILED } else { DOT_ACTIVE }
                    } else {
                        DOT_INACTIVE
                    };
                    rsx! {
                        div {
                            key: "{i}",
                            role: "listitem",
                            "aria-label": "Step {i + 1}",
                            "aria-current": if is_active { "step" },
                            class: dot,
                        }
                    }
                }
            }
        }
    }
}
