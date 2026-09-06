//! SSR pins for the stepper family: `Stepper`/`Step` visibility, `StepProgress`
//! list semantics and `aria-current`, `StepNav` labels, `StepDots`, `StepSuccess`
//! and `ClearDraftButton` markup.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "form")]

use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use validator::Validate;

use components::form::dynamic::use_dynamic_form;
use components::{
    ClearDraftButton, FormFields, MultiStepForm, Step, StepDefinition, StepDots, StepMeta, StepNav,
    StepProgress, StepProgressVariant, StepSuccess, Stepper, Steps, SummaryField, SummarySection,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Wizard {
    name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Steps)]
enum WStep {
    #[step(title = "Personal")]
    Personal,
    #[step(title = "Review", description = "Check everything")]
    Review,
    Done,
}

impl StepDefinition for WStep {
    const ALL: &[Self] = Self::ALL;
    const COUNT: usize = Self::COUNT;
    const TITLES: &[&str] = Self::TITLES;
    const DESCRIPTIONS: &[&str] = Self::DESCRIPTIONS;
}

impl StepMeta for WStep {
    const ALL: &'static [Self] = Self::ALL;
    const COUNT: usize = Self::COUNT;
    const TITLES: &'static [&'static str] = Self::TITLES;
    const DESCRIPTIONS: &'static [&'static str] = Self::DESCRIPTIONS;
}

#[test]
fn steps_derive_defaults_title_from_the_variant_name() {
    assert_eq!(WStep::ALL, &[WStep::Personal, WStep::Review, WStep::Done]);
    assert_eq!(WStep::COUNT, 3);
    assert_eq!(WStep::TITLES, &["Personal", "Review", "Done"]);
    assert_eq!(WStep::DESCRIPTIONS, &["", "Check everything", ""]);
    assert_eq!(WStep::Done.ordinal(), 2);
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    // `Step`s register during their own render; settle so `StepProgress` sees them.
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

/// The whole opening tag (`<tag ...>`) that contains `needle`.
fn tag_containing<'a>(html: &'a str, needle: &str) -> &'a str {
    let at = html
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} in {html}"));
    let start = html[..at].rfind('<').unwrap();
    let end = html[at..].find('>').unwrap();
    &html[start..at + end + 1]
}

fn headless() -> Element {
    rsx! {
        Stepper::<WStep> {
            StepProgress { variant: StepProgressVariant::Horizontal }
            Step { id: WStep::Personal, p { "first" } }
            Step { id: WStep::Review, p { "second" } }
            Step { id: WStep::Done, p { "third" } }
            StepSuccess { p { "done" } }
        }
    }
}

#[test]
fn stepper_shows_only_the_first_step() {
    let html = render(headless);
    let steps: Vec<&str> = html.split(r#"data-name="Step""#).skip(1).collect();
    assert_eq!(steps.len(), 3, "{html}");
    let head = |s: &str| s[..s.find('>').unwrap()].to_string();
    assert!(head(steps[0]).contains(r#"data-active="true""#), "{html}");
    assert!(!head(steps[0]).contains("hidden"), "{html}");
    assert!(head(steps[1]).contains("hidden"), "{html}");
    assert!(head(steps[2]).contains("hidden"), "{html}");
}

#[test]
fn step_progress_horizontal_is_a_nav_over_a_list() {
    let html = render(headless);
    let nav_start = html.find(r#"data-name="StepProgress""#).expect("progress");
    let nav = &html[nav_start..];
    let nav_head = &nav[..nav.find('>').unwrap()];
    assert!(nav_head.contains(r#"aria-label="Progress""#), "{nav_head}");
    assert!(nav_head.contains(r#"data-current-step="0""#), "{nav_head}");
    assert!(
        !nav_head.contains("role="),
        "landmark keeps its role: {nav_head}"
    );
    assert!(html.contains(r#"<ol class="#), "{html}");
    assert_eq!(count(&html, "<li"), 3, "{html}");
    assert_eq!(
        count(&html, r#"aria-current="step""#),
        2,
        "li + button: {html}"
    );
    // Steps are labelled and the current one is announced on the focusable button.
    let button = tag_containing(&html, r#"aria-label="Step 1: Personal""#);
    assert!(button.starts_with("<button"), "{button}");
    assert!(button.contains(r#"aria-current="step""#), "{button}");
    assert!(html.contains(r#"aria-label="Step 2: Review""#), "{html}");
    // Upcoming steps are not clickable.
    assert_eq!(count(&html, "disabled=true"), 2, "{html}");
}

fn counter() -> Element {
    rsx! {
        Stepper::<WStep> {
            StepProgress { variant: StepProgressVariant::Counter }
            Step { id: WStep::Personal, "a" }
            Step { id: WStep::Review, "b" }
            Step { id: WStep::Done, "c" }
        }
    }
}

#[test]
fn step_progress_counter_reads_step_x_of_n() {
    let html = render(counter);
    assert!(html.contains("Step 1 of 3"), "{html}");
    assert!(html.contains(r#"data-current-step="0""#), "{html}");
}

fn dots() -> Element {
    rsx! {
        Stepper::<WStep> {
            StepProgress { variant: StepProgressVariant::Dots }
            Step { id: WStep::Personal, "a" }
            Step { id: WStep::Review, "b" }
            Step { id: WStep::Done, "c" }
        }
    }
}

#[test]
fn step_progress_dots_label_every_dot() {
    let html = render(dots);
    assert_eq!(count(&html, "<li"), 3, "{html}");
    assert!(html.contains(r#"aria-label="Step 1""#), "{html}");
    assert!(html.contains(r#"aria-label="Step 3""#), "{html}");
    assert_eq!(count(&html, r#"aria-current="step""#), 1, "{html}");
    assert_eq!(
        count(
            &html,
            "h-1.5 w-6 rounded-full bg-primary transition-all duration-300"
        ),
        1
    );
    assert_eq!(
        count(
            &html,
            "h-1.5 w-1.5 rounded-full bg-muted transition-all duration-300"
        ),
        2
    );
}

fn step_dots() -> Element {
    rsx! {
        StepDots { current: WStep::Review }
    }
}

#[test]
fn step_dots_render_count_dots_with_the_active_pill() {
    let html = render(step_dots);
    assert!(html.contains(r#"role="list""#), "{html}");
    assert_eq!(count(&html, r#"role="listitem""#), 3, "{html}");
    assert_eq!(
        count(&html, "h-1.5 w-6 rounded-full bg-primary"),
        1,
        "{html}"
    );
    assert_eq!(
        count(&html, "h-1.5 w-1.5 rounded-full bg-muted"),
        2,
        "{html}"
    );
    let active = tag_containing(&html, r#"aria-label="Step 2""#);
    assert!(active.contains(r#"aria-current="step""#), "{active}");
    assert_eq!(count(&html, r#"aria-current="step""#), 1, "{html}");
}

fn step_dots_failed() -> Element {
    rsx! {
        StepDots { current: WStep::Personal, failed: true }
    }
}

#[test]
fn step_dots_failed_uses_the_destructive_pill() {
    let html = render(step_dots_failed);
    assert_eq!(count(&html, "bg-destructive"), 1, "{html}");
}

#[test]
fn step_success_is_hidden_and_focusable_programmatically() {
    let html = render(headless);
    let success = tag_containing(&html, r#"data-name="StepSuccess""#);
    assert!(success.contains("hidden"), "{success}");
    assert!(success.contains(r#"tabindex="-1""#), "{success}");
}

fn wizard_first() -> Element {
    let form = use_dynamic_form::<Wizard>();
    rsx! {
        MultiStepForm::<Wizard, WStep> {
            form,
            Step { id: WStep::Personal, "a" }
            Step { id: WStep::Review,
                SummarySection { step: WStep::Personal,
                    SummaryField { label: "Name", name: "name" }
                }
            }
            ClearDraftButton {}
            StepNav { form, on_submit: move |_: Wizard| {} }
        }
    }
}

fn wizard_last() -> Element {
    let form = use_dynamic_form::<Wizard>();
    rsx! {
        MultiStepForm::<Wizard, WStep> {
            form,
            initial: Some(WStep::Done),
            Step { id: WStep::Personal, "a" }
            Step { id: WStep::Review, "b" }
            Step { id: WStep::Done, "c" }
            StepNav { form, on_submit: move |_: Wizard| {} }
        }
    }
}

#[test]
fn step_nav_hides_back_on_first_and_says_submit_on_last() {
    let first = render(wizard_first);
    assert!(!first.contains(">Back<"), "{first}");
    assert!(first.contains(">Next<"), "{first}");
    assert!(!first.contains(">Submit<"), "{first}");

    let last = render(wizard_last);
    assert!(last.contains(">Back<"), "{last}");
    assert!(last.contains(">Submit<"), "{last}");
    assert!(!last.contains(">Next<"), "{last}");
}

#[test]
fn clear_draft_and_summary_edit_are_real_labelled_buttons() {
    let html = render(wizard_first);
    let clear = tag_containing(&html, r#"data-name="ClearDraftButton""#);
    assert!(clear.starts_with("<button"), "{clear}");
    assert!(clear.contains(r#"type="button""#), "{clear}");
    assert!(clear.contains("text-muted-foreground"), "{clear}");
    assert!(html.contains("Clear Draft</button>"), "{html}");

    assert!(html.contains(r#"aria-label="Edit Personal""#), "{html}");
    // Empty summary values render an em dash.
    assert!(html.contains("\u{2014}"), "{html}");
}
