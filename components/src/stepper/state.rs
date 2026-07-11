use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dioxus::prelude::*;

use crate::field_name::Field;
use crate::form::FormContext;

// ---------------------------------------------------------------------------
// StepFieldRegistry — context provided by `Step` so input components can
// self-register their `Field` for step validation without manual bookkeeping.
// ---------------------------------------------------------------------------

/// Provided by every `Step` component. Any input that wraps `FormField`
/// (directly or via `Input`) writes its `Field` here on mount and removes
/// it on cleanup. `Step` reads this signal when building the live field list.
#[derive(Clone, Copy)]
pub struct StepFieldRegistry(pub Signal<Vec<Field>>);

/// Registers `field` into the step field list (if any).
/// Call this from `FormField` (and any component that doesn't go through it).
///
/// Prefers [`FormContext::step_field_registry`] (works with slotted `Element` children under
/// `MultiStepForm`) then falls back to [`StepFieldRegistry`] (e.g. `Stepper` without a form).
pub(crate) fn auto_register_field(field: &Field) {
    let sig = try_consume_context::<FormContext>()
        .and_then(|ctx| ctx.step_field_registry)
        .or_else(|| try_consume_context::<StepFieldRegistry>().map(|r| r.0));

    let Some(sig) = sig else {
        return;
    };

    let f = *field;
    let name = f.name.to_string();
    // UI thread only: unchecked write avoids subscription churn; must not run concurrently.
    sig.write_unchecked().retain(|x| x.name != name);
    sig.write_unchecked().push(f);
}

/// Removes `name` from the nearest step registry (if any). Call from `use_drop` when an input unmounts.
pub(crate) fn unregister_auto_field(name: &str) {
    let sig = try_consume_context::<FormContext>()
        .and_then(|ctx| ctx.step_field_registry)
        .or_else(|| try_consume_context::<StepFieldRegistry>().map(|r| r.0));
    let Some(sig) = sig else {
        return;
    };
    sig.write_unchecked().retain(|x| x.name != name);
}

// ---------------------------------------------------------------------------
// Persistence helpers (localStorage)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(target_arch = "wasm32")]
fn storage_get(key: &str) -> Option<String> {
    get_storage()?.get_item(key).ok().flatten()
}

#[cfg(target_arch = "wasm32")]
fn storage_set(key: &str, value: &str) {
    if let Some(storage) = get_storage() {
        let _ = storage.set_item(key, value);
    }
}

#[cfg(target_arch = "wasm32")]
fn storage_remove(key: &str) {
    if let Some(storage) = get_storage() {
        let _ = storage.remove_item(key);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_get(_key: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_set(_key: &str, _value: &str) {}

#[cfg(not(target_arch = "wasm32"))]
fn storage_remove(_key: &str) {}

pub(crate) fn load_persisted(key: &str) -> Option<HashMap<String, String>> {
    let raw = storage_get(key)?;
    serde_json::from_str(&raw).ok()
}

pub(crate) fn save_persisted(key: &str, values: &HashMap<String, String>) {
    if let Ok(json) = serde_json::to_string(values) {
        storage_set(key, &json);
    }
}

pub(crate) fn load_persisted_step(key: &str) -> Option<usize> {
    let step_key = format!("{key}:step");
    let raw = storage_get(&step_key)?;
    raw.parse().ok()
}

pub(crate) fn save_persisted_step(key: &str, index: usize) {
    let step_key = format!("{key}:step");
    storage_set(&step_key, &index.to_string());
}

pub(crate) fn remove_persisted(key: &str) {
    storage_remove(key);
    let step_key = format!("{key}:step");
    storage_remove(&step_key);
}

// ---------------------------------------------------------------------------
// StepId — marker trait for step enum identity
// ---------------------------------------------------------------------------

pub trait StepId: Clone + Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static {}
impl<T> StepId for T where T: Clone + Copy + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static
{}

// ---------------------------------------------------------------------------
// StepDefinition — trait that carries all step metadata.
// ---------------------------------------------------------------------------

pub trait StepDefinition: StepId + Sized {
    const ALL: &[Self];
    const COUNT: usize;
    const TITLES: &[&str];
    const DESCRIPTIONS: &[&str];
    const RENDER_TITLE_DESCRIPTION_IN_STEP: bool = false;

    fn title(&self) -> &'static str {
        Self::TITLES[self.ordinal()]
    }

    fn description(&self) -> &'static str {
        Self::DESCRIPTIONS[self.ordinal()]
    }

    fn initial() -> Self {
        Self::ALL[0]
    }

    fn try_ordinal(&self) -> Option<usize> {
        Self::ALL.iter().position(|s| *s == *self)
    }

    fn ordinal(&self) -> usize {
        match self.try_ordinal() {
            Some(i) => i,
            None => {
                debug_assert!(false, "step variant must appear in ALL");
                tracing::warn!(
                    target: "components::stepper",
                    "StepDefinition::ordinal: variant not in ALL; using index 0 (check Steps macro / ALL const)"
                );
                0
            }
        }
    }

    fn fields(&self) -> Option<Vec<Field>> {
        None
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Completed,
    Current,
    Upcoming,
}

#[derive(Clone, PartialEq)]
pub struct StepInfo {
    pub index: usize,
    pub title: Arc<str>,
    pub fields: Vec<Field>,
}

// ---------------------------------------------------------------------------
// AnyStepCtx — type-erased step context (no generics)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct AnyStepCtx {
    pub(crate) current_index: Memo<usize>,
    pub(crate) total: Memo<usize>,
    pub(crate) completed: Signal<bool>,
    pub(crate) submitted: Signal<bool>,
    pub(crate) visited: Memo<HashSet<usize>>,
    pub(crate) direction: Signal<Direction>,
    pub(crate) steps_info: Memo<Vec<StepInfo>>,
    /// Lightweight (index, title) list for the progress bar — avoids subscribing
    /// to each step's `fields` signal (which `steps_info` clones every change).
    pub(crate) steps_meta: Memo<Vec<(usize, Arc<str>)>>,
    pub(crate) go_to_fn: CopyValue<Box<dyn Fn(usize)>>,
    pub(crate) next_fn: CopyValue<Box<dyn Fn()>>,
    pub(crate) back_fn: CopyValue<Box<dyn Fn()>>,
    pub(crate) current_fields_fn: CopyValue<Box<dyn Fn() -> Vec<Field>>>,
    pub(crate) persist_key: CopyValue<Option<String>>,
    pub(crate) clear_form_fn: CopyValue<Box<dyn Fn()>>,
    pub(crate) reset_visited_fn: CopyValue<Box<dyn Fn()>>,
}

impl AnyStepCtx {
    pub fn current_index(&self) -> usize {
        *self.current_index.read()
    }

    pub fn direction(&self) -> Direction {
        *self.direction.read()
    }

    pub fn is_completed(&self) -> bool {
        *self.completed.read()
    }

    pub fn is_submitted(&self) -> bool {
        *self.submitted.read()
    }

    pub fn total(&self) -> usize {
        *self.total.read()
    }

    pub fn is_first(&self) -> bool {
        self.current_index() == 0
    }

    pub fn is_last(&self) -> bool {
        let total = self.total();
        total == 0 || self.current_index() == total - 1
    }

    pub fn is_visited_index(&self, idx: usize) -> bool {
        self.visited.read().contains(&idx)
    }

    pub fn completed_count(&self) -> usize {
        let total = self.total();
        let v = self.visited.read();
        (0..total).filter(|i| v.contains(i)).count()
    }

    pub fn step_state(&self, idx: usize) -> StepState {
        let cur = self.current_index();
        if idx == cur {
            StepState::Current
        } else if self.visited.read().contains(&idx) {
            StepState::Completed
        } else {
            StepState::Upcoming
        }
    }

    pub fn steps_info(&self) -> Vec<StepInfo> {
        self.steps_info.read().clone()
    }

    /// (index, title) pairs for the progress bar — cheaper than [`Self::steps_info`]
    /// (`Arc<str>` clone, no per-step `fields` read).
    pub fn steps_meta(&self) -> Vec<(usize, Arc<str>)> {
        self.steps_meta.read().clone()
    }

    pub fn go_to_index(&self, idx: usize) {
        (self.go_to_fn.read())(idx);
    }

    pub fn next(&self) {
        (self.next_fn.read())();
    }

    pub fn back(&self) {
        (self.back_fn.read())();
    }

    pub fn mark_completed(mut self) {
        self.completed.write().clone_from(&true);
    }

    pub fn mark_submitted(mut self) {
        self.submitted.write().clone_from(&true);
    }

    pub fn current_fields(&self) -> Vec<Field> {
        (self.current_fields_fn.read())()
    }

    pub fn clear_persisted(&self) {
        if let Some(k) = self.persist_key.read().clone() {
            remove_persisted(&k);
        }
        (self.clear_form_fn.read())();
    }

    pub fn clear_draft(mut self) {
        self.clear_persisted();
        self.completed.set(false);
        self.submitted.set(false);
        (self.reset_visited_fn.read())();
        self.go_to_index(0);
    }
}

pub fn use_step() -> AnyStepCtx {
    use_context::<AnyStepCtx>()
}

// ---------------------------------------------------------------------------
// StepCtx<S> — typed step context for navigation by enum variant
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct StepCtx<S: StepId> {
    pub(crate) internal: InternalStepCtx<S>,
}

impl<S: StepId> StepCtx<S> {
    pub fn go_to(&self, step: S) {
        self.internal.go_to(step);
    }

    pub fn go_to_first(&self) {
        let first = self.internal.steps.read().first().map(|si| si.id);
        if let Some(id) = first {
            self.internal.go_to(id);
        }
    }
}

pub fn use_step_ctx<S: StepId>() -> StepCtx<S> {
    use_context::<StepCtx<S>>()
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct InternalStepInfo<S> {
    pub(crate) id: S,
    pub(crate) title: Arc<str>,
    /// Reactive field list.
    pub(crate) fields: Signal<Vec<Field>>,
}

#[derive(Clone, Copy)]
pub(crate) struct InternalStepCtx<S: StepId> {
    pub(crate) current: Signal<S>,
    pub(crate) steps: Signal<Vec<InternalStepInfo<S>>>,
    pub(crate) completed: Signal<bool>,
    pub(crate) visited: Signal<HashSet<S>>,
    pub(crate) direction: Signal<Direction>,
    pub(crate) submitted: Signal<bool>,
}

impl<S: StepId> InternalStepCtx<S> {
    pub(crate) fn new(initial: S) -> Self {
        let mut initial_visited = HashSet::new();
        initial_visited.insert(initial);
        Self {
            current: Signal::new_in_scope(initial, ScopeId::ROOT),
            steps: Signal::new_in_scope(Vec::new(), ScopeId::ROOT),
            completed: Signal::new_in_scope(false, ScopeId::ROOT),
            visited: Signal::new_in_scope(initial_visited, ScopeId::ROOT),
            direction: Signal::new_in_scope(Direction::Forward, ScopeId::ROOT),
            submitted: Signal::new_in_scope(false, ScopeId::ROOT),
        }
    }

    pub(crate) fn index_of(&self, id: S) -> Option<usize> {
        self.steps.read().iter().position(|si| si.id == id)
    }

    pub(crate) fn current_index(&self) -> usize {
        let cur = *self.current.read();
        self.index_of(cur).unwrap_or(0)
    }

    pub(crate) fn go_to(mut self, step: S) {
        let cur = *self.current.peek();
        if cur == step {
            return;
        }
        let cur_idx = self.index_of(cur);
        let target_idx = self.index_of(step);
        if let (Some(ci), Some(ti)) = (cur_idx, target_idx) {
            self.direction.set(if ti > ci {
                Direction::Forward
            } else {
                Direction::Backward
            });
        }
        self.visited.write().insert(step);
        self.current.set(step);
    }

    pub(crate) fn go_to_index(self, idx: usize) {
        let target = self.steps.read().get(idx).map(|si| si.id);
        if let Some(id) = target {
            self.go_to(id);
        }
    }

    /// Mark every step `0..=idx` visited and land on `idx` — used on persisted
    /// restore. One `visited` write + one `current` set, instead of stepping
    /// through each intermediate index.
    pub(crate) fn mark_visited_through(mut self, idx: usize) {
        let ids: Vec<S> = self
            .steps
            .peek()
            .iter()
            .take(idx + 1)
            .map(|si| si.id)
            .collect();
        let Some(&target) = ids.last() else {
            return;
        };
        {
            let mut visited = self.visited.write();
            visited.extend(ids.iter().copied());
        }
        self.current.set(target);
    }

    pub(crate) fn next(mut self) {
        let cur = *self.current.peek();
        let next = {
            let steps = self.steps.read();
            let pos = steps.iter().position(|si| si.id == cur);
            pos.and_then(|p| steps.get(p + 1).map(|si| si.id))
        };
        if let Some(next_id) = next {
            self.direction.set(Direction::Forward);
            self.visited.write().insert(next_id);
            self.current.set(next_id);
        }
    }

    pub(crate) fn back(mut self) {
        let cur = *self.current.peek();
        let prev = {
            let steps = self.steps.read();
            let pos = steps.iter().position(|si| si.id == cur);
            pos.and_then(|p| {
                if p > 0 {
                    steps.get(p - 1).map(|si| si.id)
                } else {
                    None
                }
            })
        };
        if let Some(prev_id) = prev {
            self.direction.set(Direction::Backward);
            self.current.set(prev_id);
        }
    }

    pub(crate) fn current_fields(&self) -> Vec<Field> {
        let cur = *self.current.read();
        self.steps
            .read()
            .iter()
            .find(|si| si.id == cur)
            .map(|si| si.fields.peek().clone())
            .unwrap_or_default()
    }

    pub(crate) fn register_ordered(mut self, info: InternalStepInfo<S>)
    where
        S: StepDefinition,
    {
        let mut steps = self.steps.write();
        if steps.iter().any(|si| si.id == info.id) {
            return;
        }
        let ordinal = info.id.ordinal();
        let pos = steps
            .iter()
            .position(|si| si.id.ordinal() > ordinal)
            .unwrap_or(steps.len());
        steps.insert(pos, info);
    }

    pub(crate) fn unregister(mut self, id: S) {
        self.steps.write().retain(|si| si.id != id);
        if *self.current.peek() == id {
            let fallback = self.steps.peek().first().map(|si| si.id);
            if let Some(fb) = fallback {
                self.current.set(fb);
            }
        }
    }
}

/// Build [`AnyStepCtx`] for a stable `internal` handle. Must run from a `#[component]` body — not
/// inside `use_hook` — because this registers `use_memo` hooks.
pub(crate) fn use_any_step_ctx<S: StepId>(
    internal: InternalStepCtx<S>,
    persist_key: Option<String>,
    clear_form_fn: CopyValue<Box<dyn Fn()>>,
) -> AnyStepCtx {
    let visited_idx = use_memo(move || {
        let visited_ids = internal.visited.read().clone();
        internal
            .steps
            .read()
            .iter()
            .enumerate()
            .filter(|(_, si)| visited_ids.contains(&si.id))
            .map(|(i, _)| i)
            .collect::<HashSet<usize>>()
    });

    let current_index = use_memo(move || internal.current_index());
    let total = use_memo(move || internal.steps.read().len());

    let steps_info = use_memo(move || {
        internal
            .steps
            .read()
            .iter()
            .enumerate()
            .map(|(i, si)| StepInfo {
                index: i,
                title: si.title.clone(),
                fields: si.fields.read().clone(),
            })
            .collect::<Vec<_>>()
    });

    let steps_meta = use_memo(move || {
        internal
            .steps
            .read()
            .iter()
            .enumerate()
            .map(|(i, si)| (i, si.title.clone()))
            .collect::<Vec<_>>()
    });

    let go_to_fn: Box<dyn Fn(usize)> = Box::new(move |idx: usize| internal.go_to_index(idx));
    let next_fn: Box<dyn Fn()> = Box::new(move || internal.next());
    let back_fn: Box<dyn Fn()> = Box::new(move || internal.back());
    let current_fields_fn: Box<dyn Fn() -> Vec<Field>> =
        Box::new(move || internal.current_fields());
    let reset_visited_fn: Box<dyn Fn()> = Box::new(move || {
        let initial = internal.steps.peek().first().map(|si| si.id);
        let mut v = internal.visited;
        v.set(initial.into_iter().collect::<HashSet<_>>());
    });

    AnyStepCtx {
        current_index,
        total,
        completed: internal.completed,
        submitted: internal.submitted,
        visited: visited_idx,
        direction: internal.direction,
        steps_info,
        steps_meta,
        go_to_fn: CopyValue::new_in_scope(go_to_fn, ScopeId::ROOT),
        next_fn: CopyValue::new_in_scope(next_fn, ScopeId::ROOT),
        back_fn: CopyValue::new_in_scope(back_fn, ScopeId::ROOT),
        current_fields_fn: CopyValue::new_in_scope(current_fields_fn, ScopeId::ROOT),
        persist_key: CopyValue::new_in_scope(persist_key, ScopeId::ROOT),
        clear_form_fn,
        reset_visited_fn: CopyValue::new_in_scope(reset_visited_fn, ScopeId::ROOT),
    }
}
