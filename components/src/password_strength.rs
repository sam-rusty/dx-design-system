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
    let long_enough = use_memo(move || value().len() >= min_len);

    rsx! {
        div { class: merge(&[&props.class]),
            div { class: "flex items-center gap-1.5 mt-2.5",
                for i in 1..=3u8 {
                    div {
                        key: "{i}",
                        class: if score() >= i {
                            merge(&["h-1 flex-1 rounded-full", &props.bar_class])
                        } else {
                            merge(&["h-1 flex-1 rounded-full", &props.bar_muted_class])
                        },
                    }
                }
                Text {
                    size: TextSize::Small,
                    tone: TextTone::Muted,
                    class: "w-12 text-right",
                    "{score_label(score())}"
                }
            }
            div { class: "flex items-center gap-2 mt-2",
                Icon {
                    name: IconName::Check,
                    class: if long_enough() {
                        merge(&["size-4", &props.check_class])
                    } else {
                        merge(&["size-4", &props.check_muted_class])
                    },
                    stroke_width: 2.5,
                }
                Text { size: TextSize::Small, tone: TextTone::Muted, "At least {min_len} characters" }
            }
        }
    }
}

/// 0–3 heuristic: length is the primary signal, symbols/case/digits promote
/// long-enough passwords a tier.
fn password_score(p: &str) -> u8 {
    if p.is_empty() {
        return 0;
    }
    if p.len() >= 16 || (p.len() >= 12 && p.chars().any(|c| !c.is_ascii_alphanumeric())) {
        return 3;
    }
    if p.len() >= 12 && (p.chars().any(|c| c.is_ascii_digit()) || p.chars().any(char::is_uppercase))
    {
        return 2;
    }
    1
}

fn score_label(score: u8) -> &'static str {
    match score {
        0 => "",
        1 => "Weak",
        2 => "Good",
        _ => "Strong",
    }
}
