use dioxus::prelude::*;
use ds_utils::format::merge;

use crate::form::typed::FieldHandle;
use crate::{Icon, IconName, Text, TextSize, TextTone};

#[derive(Props, Clone, PartialEq)]
pub struct PasswordStrengthProps {
    /// The bound password field: a bare lens or `form.field(...)`.
    #[props(into)]
    pub field: FieldHandle,
    /// Minimum length surfaced in the checklist row.
    #[props(default = 8)]
    pub min_len: usize,
    /// Extra classes merged onto the wrapper.
    #[props(default)]
    pub class: String,
    /// Classes for a filled meter segment.
    #[props(default = "bg-primary".to_string())]
    pub bar_class: String,
    /// Classes for an unfilled meter segment.
    #[props(default = "bg-muted".to_string())]
    pub bar_muted_class: String,
    /// Check-icon classes once the minimum length is met.
    #[props(default = "text-success".to_string())]
    pub check_class: String,
    /// Check-icon classes before the minimum length is met.
    #[props(default = "text-muted-foreground".to_string())]
    pub check_muted_class: String,
}

/// Strength meter + live minimum-length checklist for a form-bound password
/// field. Reads the field's value through its binding, so it composes beside
/// (not inside) the `PasswordInput` it describes.
pub fn PasswordStrength(props: PasswordStrengthProps) -> Element {
    let field = props.field.bind();
    let min_len = props.min_len;

    let value = use_memo(move || field.display());
    let score = use_memo(move || password_score(&value()));
    let long_enough = use_memo(move || long_enough(&value(), min_len));

    let bar_class = |filled: bool| {
        let tone = if filled {
            &props.bar_class
        } else {
            &props.bar_muted_class
        };
        merge(&["h-1 flex-1 rounded-full", tone])
    };
    let check_class = if long_enough() {
        merge(&["size-4", &props.check_class])
    } else {
        merge(&["size-4", &props.check_muted_class])
    };

    rsx! {
        div { class: props.class.clone(),
            div { class: "flex items-center gap-1.5 mt-2.5",
                for i in 1..=3u8 {
                    div { key: "{i}", class: bar_class(score() >= i) }
                }
                Text {
                    size: TextSize::Small,
                    tone: TextTone::Muted,
                    class: "w-12 text-right",
                    "{score_label(score())}"
                }
            }
            div { class: "flex items-center gap-2 mt-2",
                Icon { name: IconName::Check, class: check_class, stroke_width: 2.5 }
                Text { size: TextSize::Small, tone: TextTone::Muted, "At least {min_len} characters" }
            }
        }
    }
}

/// 0–3 heuristic: length is the primary signal, symbols/case/digits promote
/// long-enough passwords a tier.
fn password_score(p: &str) -> u8 {
    let len = p.chars().count();
    if len == 0 {
        return 0;
    }
    if len >= 16 || (len >= 12 && p.chars().any(|c| !c.is_ascii_alphanumeric())) {
        return 3;
    }
    if len >= 12 && (p.chars().any(|c| c.is_ascii_digit()) || p.chars().any(char::is_uppercase)) {
        return 2;
    }
    1
}

/// Minimum-length check in characters (not bytes), so multi-byte scripts
/// are not over-counted.
fn long_enough(p: &str, min_len: usize) -> bool {
    p.chars().count() >= min_len
}

fn score_label(score: u8) -> &'static str {
    match score {
        0 => "",
        1 => "Weak",
        2 => "Good",
        _ => "Strong",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_tiers_follow_length_and_character_classes() {
        assert_eq!(password_score(""), 0);
        assert_eq!(password_score("short"), 1);
        assert_eq!(password_score("elevenchars"), 1);
        assert_eq!(password_score("twelvecharsxx"), 1);
        assert_eq!(password_score("twelvechars1"), 2);
        assert_eq!(password_score("Twelvecharsx"), 2);
        assert_eq!(password_score("twelvechars!"), 3);
        assert_eq!(password_score("sixteencharslong"), 3);
    }

    #[test]
    fn score_counts_characters_not_bytes() {
        // 8 two-byte characters: 16 bytes, but only 8 characters — weak.
        assert_eq!(password_score(&"é".repeat(8)), 1);
        // 11 characters (21 bytes) → still below the 12-character tiers.
        assert_eq!(password_score(&format!("{}1", "é".repeat(10))), 1);
        // 12 characters incl. a non-alphanumeric → Strong.
        assert_eq!(password_score(&format!("{}1", "é".repeat(11))), 3);
    }

    #[test]
    fn min_length_counts_characters() {
        assert!(long_enough("密码密码密码密码", 8));
        assert!(!long_enough("密码密", 8));
    }

    #[test]
    fn labels_map_scores() {
        assert_eq!(score_label(0), "");
        assert_eq!(score_label(1), "Weak");
        assert_eq!(score_label(2), "Good");
        assert_eq!(score_label(3), "Strong");
    }
}
