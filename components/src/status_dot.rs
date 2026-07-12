use dioxus::prelude::*;
use ds_utils::format::merge;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DotTone {
    #[default]
    Success,
    Primary,
    Warning,
    Destructive,
    Muted,
    /// No color class — caller supplies the background via `class`
    /// (used for the white dot on a selected primary chip).
    None,
}

impl DotTone {
    fn class(self) -> &'static str {
        match self {
            Self::Success => "bg-success",
            Self::Primary => "bg-primary",
            Self::Warning => "bg-warning",
            Self::Destructive => "bg-destructive",
            Self::Muted => "bg-muted-foreground",
            Self::None => "",
        }
    }

    /// Tone-matched class for the expanding ring used when `pulse` is set.
    fn pulse_class(self) -> &'static str {
        match self {
            Self::Success => "dot-pulse dot-pulse-success",
            Self::Primary => "dot-pulse dot-pulse-primary",
            Self::Warning => "dot-pulse dot-pulse-warning",
            Self::Destructive => "dot-pulse dot-pulse-destructive",
            Self::Muted => "dot-pulse dot-pulse-muted",
            Self::None => "dot-pulse",
        }
    }
}

#[component]
pub fn StatusDot(
    #[props(default)] tone: DotTone,
    #[props(default)] pulse: bool,
    #[props(default = 8)] size_px: u32,
    #[props(default)] class: String,
) -> Element {
    let anim = if pulse { tone.pulse_class() } else { "" };
    let class = merge(&[
        "inline-block rounded-full shrink-0",
        tone.class(),
        anim,
        &class,
    ]);
    rsx! {
        span { class, style: "width: {size_px}px; height: {size_px}px;" }
    }
}

#[cfg(test)]
mod tests {
    use super::DotTone;

    #[test]
    fn tone_classes() {
        assert_eq!(DotTone::Success.class(), "bg-success");
        assert_eq!(DotTone::None.class(), "");
    }

    #[test]
    fn pulse_classes() {
        assert_eq!(
            DotTone::Success.pulse_class(),
            "dot-pulse dot-pulse-success"
        );
        assert_eq!(DotTone::None.pulse_class(), "dot-pulse");
    }
}
