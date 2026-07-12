use dioxus::prelude::*;
use ds_utils::format::merge;

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
        div { class: container,
            for i in 0..S::COUNT {
                {
                    let dot = if i == active {
                        if failed {
                            "h-1.5 w-6 rounded-full bg-destructive"
                        } else {
                            "h-1.5 w-6 rounded-full bg-primary"
                        }
                    } else {
                        "h-1.5 w-1.5 rounded-full bg-muted"
                    };
                    rsx! {
                        div { key: "{i}", class: dot }
                    }
                }
            }
        }
    }
}
