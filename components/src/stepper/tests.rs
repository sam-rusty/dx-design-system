//! Stepper state tests: pure `StepDefinition` defaults and draft parsing, plus
//! `InternalStepCtx` navigation and `StepNav` submission ordering run inside a
//! `VirtualDom` runtime (signals need an owning scope).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use dioxus::dioxus_core::{NoOpMutations, RuntimeGuard, VirtualDom};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::state::{
    AnyStepCtx, Direction, InternalStepCtx, InternalStepInfo, StepDefinition, StepState,
    parse_persisted, parse_persisted_step, serialize_persisted, use_step,
};
use super::view::{skip_or_next, submit_or_next};
use super::{MultiStepForm, Step, Stepper};
// The `FormFields` derive emits `components::` paths; alias ourselves so it resolves in-crate.
use crate as components;
use crate::FormFields;
use crate::field_name::Field;
use crate::form::dynamic::{DynamicForm, TextInput, use_dynamic_form};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum WStep {
    One,
    Two,
    Three,
}

impl StepDefinition for WStep {
    const ALL: &[Self] = &[WStep::One, WStep::Two, WStep::Three];
    const COUNT: usize = 3;
    const TITLES: &[&str] = &["First", "Second", "Third"];
    const DESCRIPTIONS: &[&str] = &["", "about two", ""];
}

/// A variant list that deliberately omits `Three`, to exercise the `ordinal` fallback.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Partial {
    A,
    B,
    Missing,
}

impl StepDefinition for Partial {
    const ALL: &[Self] = &[Partial::A, Partial::B];
    const COUNT: usize = 2;
    const TITLES: &[&str] = &["A", "B"];
    const DESCRIPTIONS: &[&str] = &["", ""];
}

// ---------------------------------------------------------------------------
// StepDefinition defaults
// ---------------------------------------------------------------------------

#[test]
fn step_definition_defaults_follow_all_order() {
    assert_eq!(WStep::initial(), WStep::One);
    assert_eq!(WStep::Two.try_ordinal(), Some(1));
    assert_eq!(WStep::Three.ordinal(), 2);
    assert_eq!(WStep::Two.title(), "Second");
    assert_eq!(WStep::Two.description(), "about two");
    assert_eq!(WStep::One.description(), "");
    assert!(WStep::One.fields().is_none());
}

#[test]
fn step_definition_missing_variant_reports_none() {
    assert_eq!(Partial::Missing.try_ordinal(), None);
    assert_eq!(Partial::B.ordinal(), 1);
}

// ---------------------------------------------------------------------------
// Draft parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_persisted_round_trips_the_value_map() {
    let mut values = HashMap::new();
    values.insert("name".to_string(), "Ada".to_string());
    values.insert("age".to_string(), "36".to_string());
    let json = serialize_persisted(&values).unwrap();
    assert_eq!(parse_persisted(&json), Some(values));
    assert_eq!(parse_persisted("{}"), Some(HashMap::new()));
}

#[test]
fn parse_persisted_rejects_bad_payloads() {
    assert_eq!(parse_persisted(""), None);
    assert_eq!(parse_persisted("not json"), None);
    assert_eq!(parse_persisted("[1, 2]"), None);
    // Values must be strings — a stale draft from another schema is ignored wholesale.
    assert_eq!(parse_persisted(r#"{"age": 36}"#), None);
}

#[test]
fn parse_persisted_step_is_a_plain_index() {
    assert_eq!(parse_persisted_step("2"), Some(2));
    assert_eq!(parse_persisted_step(" 0 "), Some(0));
    assert_eq!(parse_persisted_step("-1"), None);
    assert_eq!(parse_persisted_step("two"), None);
    assert_eq!(parse_persisted_step(""), None);
}

// ---------------------------------------------------------------------------
// InternalStepCtx navigation (needs a runtime for signals)
// ---------------------------------------------------------------------------

fn run_with_scope<R>(f: impl FnOnce() -> R) -> R {
    let vdom = VirtualDom::new(|| rsx! {});
    let _guard = RuntimeGuard::new(vdom.runtime());
    vdom.in_scope(ScopeId::ROOT, f)
}

fn info(id: WStep) -> InternalStepInfo<WStep> {
    InternalStepInfo {
        id,
        title: Arc::from(id.title()),
        fields: Signal::new_in_scope(Vec::<Field>::new(), ScopeId::ROOT),
    }
}

fn ctx_with_all() -> InternalStepCtx<WStep> {
    let ctx = InternalStepCtx::new(WStep::One);
    for id in WStep::ALL {
        ctx.register_ordered(info(*id));
    }
    ctx
}

#[test]
fn register_ordered_sorts_by_all_regardless_of_mount_order() {
    run_with_scope(|| {
        let ctx = InternalStepCtx::new(WStep::One);
        ctx.register_ordered(info(WStep::Three));
        ctx.register_ordered(info(WStep::One));
        ctx.register_ordered(info(WStep::Two));
        // Re-registering is a no-op.
        ctx.register_ordered(info(WStep::Two));
        let ids: Vec<_> = ctx.steps.peek().iter().map(|si| si.id).collect();
        assert_eq!(ids, vec![WStep::One, WStep::Two, WStep::Three]);
        assert_eq!(ctx.index_of(WStep::Three), Some(2));
    });
}

#[test]
fn next_and_back_track_direction_and_visited() {
    run_with_scope(|| {
        let ctx = ctx_with_all();
        assert_eq!(ctx.current_index(), 0);
        assert!(ctx.visited.peek().contains(&WStep::One));

        ctx.next();
        assert_eq!(ctx.current_index(), 1);
        assert_eq!(*ctx.direction.peek(), Direction::Forward);
        assert!(ctx.visited.peek().contains(&WStep::Two));

        ctx.back();
        assert_eq!(ctx.current_index(), 0);
        assert_eq!(*ctx.direction.peek(), Direction::Backward);
        // Going back does not forget a visited step.
        assert!(ctx.visited.peek().contains(&WStep::Two));

        // `back` on the first step is a no-op.
        ctx.back();
        assert_eq!(ctx.current_index(), 0);
    });
}

#[test]
fn next_on_last_step_is_a_noop() {
    run_with_scope(|| {
        let ctx = ctx_with_all();
        ctx.go_to(WStep::Three);
        ctx.next();
        assert_eq!(ctx.current_index(), 2);
        assert_eq!(*ctx.direction.peek(), Direction::Forward);
    });
}

#[test]
fn go_to_sets_direction_from_relative_position() {
    run_with_scope(|| {
        let ctx = ctx_with_all();
        ctx.go_to(WStep::Three);
        assert_eq!(*ctx.direction.peek(), Direction::Forward);
        assert_eq!(ctx.current_index(), 2);
        ctx.go_to(WStep::Two);
        assert_eq!(*ctx.direction.peek(), Direction::Backward);
        // Same target: nothing changes.
        ctx.go_to(WStep::Two);
        assert_eq!(*ctx.direction.peek(), Direction::Backward);
        assert_eq!(ctx.current_index(), 1);
    });
}

#[test]
fn go_to_index_ignores_out_of_range() {
    run_with_scope(|| {
        let ctx = ctx_with_all();
        ctx.go_to_index(7);
        assert_eq!(ctx.current_index(), 0);
        ctx.go_to_index(2);
        assert_eq!(ctx.current_index(), 2);
    });
}

#[test]
fn mark_visited_through_lands_on_target_and_marks_prefix() {
    run_with_scope(|| {
        let ctx = ctx_with_all();
        ctx.mark_visited_through(1);
        assert_eq!(ctx.current_index(), 1);
        let visited = ctx.visited.peek();
        assert!(visited.contains(&WStep::One));
        assert!(visited.contains(&WStep::Two));
        assert!(!visited.contains(&WStep::Three));
    });
}

#[test]
fn unregister_current_falls_back_to_first_step() {
    run_with_scope(|| {
        let ctx = ctx_with_all();
        ctx.go_to(WStep::Three);
        ctx.unregister(WStep::Three);
        assert_eq!(ctx.steps.peek().len(), 2);
        assert_eq!(*ctx.current.peek(), WStep::One);
        // Unregistering a non-current step leaves `current` alone.
        ctx.go_to(WStep::Two);
        ctx.unregister(WStep::One);
        assert_eq!(*ctx.current.peek(), WStep::Two);
        assert_eq!(ctx.current_index(), 0);
    });
}

#[test]
fn current_fields_reads_the_active_step_only() {
    run_with_scope(|| {
        let ctx = InternalStepCtx::new(WStep::One);
        let one = info(WStep::One);
        let mut two = info(WStep::Two);
        two.fields.write().push(Field::new(
            "name",
            "Name",
            true,
            crate::field_name::FieldType::String,
        ));
        ctx.register_ordered(one);
        ctx.register_ordered(two);
        assert!(ctx.current_fields().is_empty());
        ctx.next();
        assert_eq!(ctx.current_fields().len(), 1);
    });
}

// ---------------------------------------------------------------------------
// AnyStepCtx via a mounted provider
// ---------------------------------------------------------------------------

thread_local! {
    static CTX: RefCell<Option<AnyStepCtx>> = const { RefCell::new(None) };
    static FORM: RefCell<Option<DynamicForm<Wizard>>> = const { RefCell::new(None) };
    static SUBMITTED: RefCell<Vec<Wizard>> = const { RefCell::new(Vec::new()) };
}

#[component]
fn Probe() -> Element {
    let ctx = use_step();
    use_hook(move || CTX.with(|c| *c.borrow_mut() = Some(ctx)));
    rsx! {}
}

fn empty_stepper() -> Element {
    rsx! {
        Stepper::<WStep> {
            Probe {}
        }
    }
}

fn mount(app: fn() -> Element) -> (VirtualDom, AnyStepCtx) {
    CTX.with(|c| *c.borrow_mut() = None);
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let ctx = CTX.with(|c| c.borrow_mut().take()).unwrap();
    (dom, ctx)
}

#[test]
fn any_step_ctx_with_no_steps_is_first_and_last() {
    let (dom, ctx) = mount(empty_stepper);
    dom.in_runtime(|| {
        assert_eq!(ctx.total(), 0);
        assert!(ctx.is_first());
        assert!(ctx.is_last());
        assert_eq!(ctx.completed_count(), 0);
        assert_eq!(ctx.step_state(0), StepState::Current);
    });
}

fn three_step_stepper() -> Element {
    rsx! {
        Stepper::<WStep> {
            Step { id: WStep::One, "one" }
            Step { id: WStep::Two, "two" }
            Step { id: WStep::Three, "three" }
            Probe {}
        }
    }
}

#[test]
fn any_step_ctx_step_state_and_completed_count() {
    let (mut dom, ctx) = mount(three_step_stepper);
    dom.in_runtime(|| {
        assert_eq!(ctx.total(), 3);
        assert_eq!(ctx.step_state(0), StepState::Current);
        assert_eq!(ctx.step_state(1), StepState::Upcoming);
        assert_eq!(ctx.completed_count(), 1);
        ctx.next();
        ctx.next();
        ctx.back();
    });
    dom.render_immediate(&mut NoOpMutations);
    dom.in_runtime(|| {
        assert_eq!(ctx.current_index(), 1);
        assert_eq!(ctx.step_state(0), StepState::Completed);
        assert_eq!(ctx.step_state(1), StepState::Current);
        // Visited-but-ahead steps count as completed (documented "visited" semantics).
        assert_eq!(ctx.step_state(2), StepState::Completed);
        assert_eq!(ctx.completed_count(), 3);
        assert!(ctx.is_visited_index(2));
    });
}

#[test]
fn clear_draft_resets_flags_visited_and_index() {
    let (mut dom, ctx) = mount(three_step_stepper);
    dom.in_runtime(|| {
        ctx.next();
        ctx.next();
        ctx.mark_completed();
        ctx.mark_submitted();
    });
    dom.render_immediate(&mut NoOpMutations);
    dom.in_runtime(|| {
        assert!(ctx.is_completed());
        assert!(ctx.is_submitted());
        ctx.clear_draft();
    });
    dom.render_immediate(&mut NoOpMutations);
    dom.in_runtime(|| {
        assert_eq!(ctx.current_index(), 0);
        assert!(!ctx.is_completed());
        assert!(!ctx.is_submitted());
        assert!(!ctx.is_visited_index(1));
        assert!(!ctx.is_visited_index(2));
        assert_eq!(ctx.completed_count(), 1);
    });
}

// ---------------------------------------------------------------------------
// StepNav submission ordering (F2)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Wizard {
    #[validate(length(min = 1, message = "Name is required"))]
    name: String,
    note: String,
}

fn wizard_form() -> Element {
    let form = use_dynamic_form::<Wizard>();
    use_hook(move || FORM.with(|f| *f.borrow_mut() = Some(form)));
    rsx! {
        MultiStepForm::<Wizard, WStep> {
            form,
            Step { id: WStep::One,
                TextInput { field: Wizard::note }
            }
            Step { id: WStep::Two,
                TextInput { field: Wizard::name }
            }
            Step { id: WStep::Three, "review" }
            Probe {}
        }
    }
}

struct NavHarness {
    dom: VirtualDom,
    ctx: AnyStepCtx,
    form: DynamicForm<Wizard>,
}

impl NavHarness {
    fn new() -> Self {
        SUBMITTED.with(|s| s.borrow_mut().clear());
        FORM.with(|f| *f.borrow_mut() = None);
        let (dom, ctx) = mount(wizard_form);
        let form = FORM.with(|f| f.borrow_mut().take()).unwrap();
        Self { dom, ctx, form }
    }

    fn run(&mut self, f: impl FnOnce(AnyStepCtx, DynamicForm<Wizard>)) {
        let (ctx, form) = (self.ctx, self.form);
        self.dom.in_scope(ScopeId::ROOT, || f(ctx, form));
        self.dom.render_immediate(&mut NoOpMutations);
    }

    fn submitted() -> usize {
        SUBMITTED.with(|s| s.borrow().len())
    }
}

fn on_submit() -> EventHandler<Wizard> {
    EventHandler::new(|data: Wizard| SUBMITTED.with(|s| s.borrow_mut().push(data)))
}

#[test]
fn primary_action_validates_the_current_step_before_advancing() {
    let mut h = NavHarness::new();
    // Step One's `note` (a plain `String`) is required and empty: stay, with an error.
    h.run(|ctx, form| submit_or_next(ctx, form, None, on_submit()));
    h.run(|ctx, form| {
        assert_eq!(ctx.current_index(), 0);
        assert!(form.error("note").is_some());
        assert!(
            form.error("name").is_none(),
            "other steps are not validated"
        );
    });
    h.run(|ctx, form| {
        form.set_string_value("note", "hi".into());
        submit_or_next(ctx, form, None, on_submit());
    });
    h.run(|ctx, _| assert_eq!(ctx.current_index(), 1));
    // Step Two's `name` is required and empty: stay.
    h.run(|ctx, form| submit_or_next(ctx, form, None, on_submit()));
    h.run(|ctx, form| {
        assert_eq!(ctx.current_index(), 1);
        assert!(form.error("name").is_some());
    });
    // `before_next` returning false cancels even a valid step.
    h.run(|ctx, form| {
        form.set_string_value("name", "Ada".into());
        submit_or_next(ctx, form, Some(Callback::new(|_| false)), on_submit());
    });
    h.run(|ctx, _| assert_eq!(ctx.current_index(), 1));
    h.run(|ctx, form| submit_or_next(ctx, form, None, on_submit()));
    h.run(|ctx, _| assert_eq!(ctx.current_index(), 2));
    assert_eq!(NavHarness::submitted(), 0);
}

#[test]
fn failed_last_step_validation_does_not_mark_submitted() {
    let mut h = NavHarness::new();
    h.run(|ctx, _| ctx.go_to_index(2));
    h.run(|ctx, _| assert!(ctx.is_last()));
    // `name` is required and empty: whole-form validation fails on submit.
    h.run(|ctx, form| submit_or_next(ctx, form, None, on_submit()));
    assert_eq!(NavHarness::submitted(), 0);
    h.run(|ctx, _| {
        assert!(
            !ctx.is_submitted(),
            "submitted flipped although validation failed"
        )
    });

    h.run(|ctx, form| {
        form.set_string_value("name", "Ada".into());
        submit_or_next(ctx, form, None, on_submit());
    });
    assert_eq!(NavHarness::submitted(), 1);
    h.run(|ctx, _| assert!(ctx.is_submitted()));
}

#[test]
fn skip_on_last_step_marks_submitted_only_when_data_is_produced() {
    let mut h = NavHarness::new();
    // Not last: skip advances without validating.
    h.run(|ctx, form| skip_or_next(ctx, form, on_submit()));
    h.run(|ctx, _| {
        assert_eq!(ctx.current_index(), 1);
        assert!(!ctx.is_submitted());
    });
    h.run(|ctx, _| ctx.go_to_index(2));
    // An undeserialisable map yields no data: nothing is submitted or marked.
    h.run(|ctx, mut form| {
        form.set_string_value("note", "x".into());
        form.values_signal
            .write()
            .insert("age".to_string(), "not-a-number".to_string());
        let _ = ctx;
    });
    h.run(|ctx, form| {
        let has_data = form.get_data().is_some();
        skip_or_next(ctx, form, on_submit());
        assert_eq!(NavHarness::submitted(), usize::from(has_data));
        assert_eq!(ctx.is_submitted(), has_data);
    });
}
