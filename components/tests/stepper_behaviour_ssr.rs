//! Behaviour pins for `MultiStepForm` run inside a `VirtualDom`: `on_step_change`
//! does not fire for the mount / `initial` step (F6) and reports real navigation
//! as `(prev, cur)`.

// Integration tests are not feature-gated by cargo; gate on the family this
// file renders so `--no-default-features --features <family>` still builds.
#![cfg(feature = "form")]

use std::cell::RefCell;

use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use ds_components as components;
use serde::{Deserialize, Serialize};
use validator::Validate;

use components::form::dynamic::{TextInput, use_dynamic_form};
use components::{AnyStepCtx, FormFields, MultiStepForm, Step, StepDefinition, Steps, use_step};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate, FormFields)]
struct Wizard {
    name: String,
    note: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Steps)]
enum WStep {
    One,
    Two,
    Three,
}

impl StepDefinition for WStep {
    const ALL: &[Self] = Self::ALL;
    const COUNT: usize = Self::COUNT;
    const TITLES: &[&str] = Self::TITLES;
    const DESCRIPTIONS: &[&str] = Self::DESCRIPTIONS;
}

thread_local! {
    static CTX: RefCell<Option<AnyStepCtx>> = const { RefCell::new(None) };
    static CHANGES: RefCell<Vec<(usize, usize)>> = const { RefCell::new(Vec::new()) };
}

/// Captures the provided `AnyStepCtx` for the test body.
#[component]
fn Probe() -> Element {
    let ctx = use_step();
    use_hook(move || CTX.with(|c| *c.borrow_mut() = Some(ctx)));
    rsx! {}
}

fn app() -> Element {
    let form = use_dynamic_form::<Wizard>();
    rsx! {
        MultiStepForm::<Wizard, WStep> {
            form,
            initial: Some(WStep::Two),
            on_step_change: move |(prev, cur): (usize, usize)| {
                CHANGES.with(|c| c.borrow_mut().push((prev, cur)));
            },
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

struct Harness {
    dom: VirtualDom,
    ctx: AnyStepCtx,
}

impl Harness {
    fn new() -> Self {
        CHANGES.with(|c| c.borrow_mut().clear());
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        // Run the queued effects (restore, required fields, on_step_change).
        dom.render_immediate(&mut NoOpMutations);
        let ctx = CTX.with(|c| c.borrow_mut().take()).unwrap();
        Self { dom, ctx }
    }

    fn update(&mut self, f: impl FnOnce(AnyStepCtx)) {
        let ctx = self.ctx;
        self.dom.in_runtime(|| f(ctx));
        // A signal write wakes the effect's task, which queues the effect for the render after
        // that one — two passes flush it.
        self.dom.render_immediate(&mut NoOpMutations);
        self.dom.render_immediate(&mut NoOpMutations);
    }

    fn changes() -> Vec<(usize, usize)> {
        CHANGES.with(|c| c.borrow().clone())
    }
}

#[test]
fn on_step_change_is_silent_for_the_initial_step() {
    let mut h = Harness::new();
    assert_eq!(h.ctx.current_index(), 1);
    assert_eq!(Harness::changes(), Vec::<(usize, usize)>::new());

    h.update(|ctx| ctx.next());
    assert_eq!(Harness::changes(), vec![(1, 2)]);

    h.update(|ctx| ctx.back());
    assert_eq!(Harness::changes(), vec![(1, 2), (2, 1)]);
}

// ---------------------------------------------------------------------------
// F4: provider-scope ownership — unmount order and captured handles (verifier-added)
// ---------------------------------------------------------------------------

thread_local! {
    static SHOW: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
}

fn toggled_app() -> Element {
    let form = use_dynamic_form::<Wizard>();
    let show = use_signal(|| true);
    use_hook(move || SHOW.with(|s| *s.borrow_mut() = Some(show)));
    rsx! {
        if show() {
            MultiStepForm::<Wizard, WStep> {
                form,
                Step { id: WStep::One,
                    TextInput { field: Wizard::note }
                }
                Step { id: WStep::Two,
                    TextInput { field: Wizard::name }
                }
                Probe {}
            }
        }
    }
}

/// `Step::use_drop` writes the provider-owned `steps` signal; children are dropped before the
/// provider scope (and its signals), so unmounting the whole wizard must not panic.
#[test]
fn unmounting_the_provider_drops_steps_before_its_signals() {
    CHANGES.with(|c| c.borrow_mut().clear());
    let mut dom = VirtualDom::new(toggled_app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let ctx = CTX.with(|c| c.borrow_mut().take()).unwrap();
    dom.in_runtime(|| assert_eq!(ctx.total(), 2));

    let mut show = SHOW.with(|s| s.borrow().unwrap());
    dom.in_runtime(|| show.set(false));
    dom.render_immediate(&mut NoOpMutations);
    dom.render_immediate(&mut NoOpMutations);

    // Signals now live in the (dropped) provider scope: a handle captured by a child that
    // outlived it (e.g. a spawned task) panics on read rather than silently working.
    let read_after_unmount = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dom.in_runtime(|| ctx.total())
    }));
    assert!(
        read_after_unmount.is_err(),
        "provider signals should be dropped on unmount"
    );
}
