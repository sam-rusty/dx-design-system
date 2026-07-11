//! Roving-focus state, ported from `dioxus_primitives` (`primitives/src/focus.rs`).
//!
//! A single shared `FocusState` drives keyboard navigation for any widget that
//! has a set of focusable items: tabs, accordion headers, select/listbox
//! options, radio groups, segmented controls, menus, calendar grids, etc. Items
//! register themselves by index; the state tracks which is focused and exposes
//! `focus_next`/`focus_prev`/`focus_first`/`focus_last` (skipping disabled
//! items, wrapping when `roving_loop` is on). This is the missing layer behind
//! every "no keyboard navigation" finding in the audit.

use std::collections::BTreeMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::use_effect_with_cleanup;

pub(crate) fn use_focus_provider(roving_loop: ReadSignal<bool>) -> FocusState {
    use_context_provider(|| FocusState::new(roving_loop))
}

pub(crate) fn use_focus_entry_disabled(
    mut ctx: FocusState,
    index: impl Readable<Target = usize> + Copy + 'static,
    disabled: impl Fn() -> bool + Copy + 'static,
) {
    use_effect_with_cleanup(move || {
        let idx = index.cloned();
        ctx.add_update_item(idx, disabled());
        move || {
            ctx.remove_item(idx);
        }
    });
}

pub(crate) fn use_focus_control(
    ctx: FocusState,
    index: impl Readable<Target = usize> + Copy + 'static,
) -> impl FnMut(MountedEvent) {
    let disabled = || false;
    use_focus_control_disabled(ctx, index, disabled)
}

pub(crate) fn use_focus_control_disabled(
    ctx: FocusState,
    index: impl Readable<Target = usize> + Copy + 'static,
    disabled: impl Fn() -> bool + Copy + 'static,
) -> impl FnMut(MountedEvent) {
    let mut controlled_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    use_effect(move || {
        if disabled() {
            return;
        }
        ctx.control_mount_focus(index.cloned(), controlled_ref);
    });

    move |data: Event<MountedData>| controlled_ref.set(Some(data.data()))
}

fn first_enabled<'a>(iter: impl IntoIterator<Item = (&'a usize, &'a bool)>) -> Option<usize> {
    iter.into_iter()
        .find_map(|(&idx, &disabled)| (!disabled).then_some(idx))
}

fn next_index(indices: &[usize], current: Option<usize>, roving_loop: bool) -> Option<usize> {
    match current {
        Some(current) => {
            let next_position = indices.partition_point(|&index| index <= current);
            indices
                .get(next_position)
                .copied()
                .or_else(|| roving_loop.then(|| indices.first().copied()).flatten())
        }
        None => indices.first().copied(),
    }
}

fn prev_index(indices: &[usize], current: Option<usize>, roving_loop: bool) -> Option<usize> {
    match current {
        Some(current) => {
            let prev_position = indices.partition_point(|&index| index < current);
            prev_position
                .checked_sub(1)
                .and_then(|position| indices.get(position).copied())
                .or_else(|| roving_loop.then(|| indices.last().copied()).flatten())
        }
        None if roving_loop => indices.last().copied(),
        None => indices.first().copied(),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FocusState {
    pub(crate) roving_loop: ReadSignal<bool>,
    pub(crate) recent_focus: Signal<Option<usize>>,
    pub(crate) current_focus: Signal<Option<usize>>,
    items: Signal<BTreeMap<usize, bool>>,
}

impl FocusState {
    pub(crate) fn new(roving_loop: ReadSignal<bool>) -> Self {
        Self {
            roving_loop,
            recent_focus: Signal::new(None),
            current_focus: Signal::new(None),
            items: Signal::new(BTreeMap::new()),
        }
    }

    pub(crate) fn set_focus(&mut self, index: Option<usize>) {
        // An index pointing at a disabled item collapses to None.
        let target = match index {
            Some(idx) if self.items.peek().get(&idx) == Some(&true) => None,
            other => other,
        };
        if let Some(idx) = target {
            self.recent_focus.set(Some(idx));
        }
        // Only notify subscribers when the value actually changes — a redundant
        // `blur()` still wakes any effect reading `any_focused()`, which on some
        // browsers caused menus to auto-close right after opening.
        if *self.current_focus.peek() != target {
            self.current_focus.set(target);
        }
    }

    pub(crate) fn first_enabled_index(&self) -> Option<usize> {
        first_enabled(self.items.read().iter())
    }

    pub(crate) fn last_enabled_index(&self) -> Option<usize> {
        first_enabled(self.items.read().iter().rev())
    }

    fn enabled_indices(&self) -> Vec<usize> {
        self.items
            .read()
            .iter()
            .filter_map(|(&index, &disabled)| (!disabled).then_some(index))
            .collect()
    }

    fn focus_next_from(&mut self, current: Option<usize>, indices: &[usize]) {
        self.set_focus(next_index(indices, current, (self.roving_loop)()));
    }

    fn focus_prev_from(&mut self, current: Option<usize>, indices: &[usize]) {
        self.set_focus(prev_index(indices, current, (self.roving_loop)()));
    }

    pub(crate) fn focus_next(&mut self) {
        let indices = self.enabled_indices();
        self.focus_next_from(self.recent_focus(), &indices);
    }

    pub(crate) fn focus_prev(&mut self) {
        let indices = self.enabled_indices();
        self.focus_prev_from(self.recent_focus(), &indices);
    }

    pub(crate) fn focus_first(&mut self) {
        self.set_focus(self.first_enabled_index());
    }

    pub(crate) fn focus_last(&mut self) {
        self.set_focus(self.last_enabled_index());
    }

    pub(crate) fn focus_next_from_current(&mut self, indices: &[usize]) {
        self.focus_next_from(self.current_focus(), indices);
    }

    pub(crate) fn focus_prev_from_current(&mut self, indices: &[usize]) {
        self.focus_prev_from(self.current_focus(), indices);
    }

    pub(crate) fn blur(&mut self) {
        self.set_focus(None);
    }

    pub(crate) fn any_focused(&self) -> bool {
        self.current_focus.read().is_some()
    }

    pub(crate) fn is_focused(&self, id: usize) -> bool {
        (self.current_focus)().map(|x| x == id).unwrap_or(false)
    }

    pub(crate) fn current_focus(&self) -> Option<usize> {
        (self.current_focus)()
    }

    pub(crate) fn recent_focus(&self) -> Option<usize> {
        (self.recent_focus)()
    }

    pub(crate) fn recent_focus_or_default(&self) -> usize {
        self.recent_focus()
            .filter(|&index| self.is_enabled(index))
            .or_else(|| self.first_enabled_index())
            .unwrap_or_default()
    }

    pub(crate) fn is_enabled(&self, index: usize) -> bool {
        self.items.peek().get(&index) == Some(&false)
    }

    /// Pick the next enabled item after `from`, wrapping when `roving_loop` is on.
    /// Used to redirect focus parked on a now-disabled item.
    fn next_focus_skipping(&self, from: usize) -> Option<usize> {
        let items = self.items.peek();
        first_enabled(items.range(from.saturating_add(1)..)).or_else(|| {
            self.roving_loop
                .peek()
                .then(|| first_enabled(items.iter()))
                .flatten()
        })
    }

    pub(crate) fn add_update_item(&mut self, index: usize, disabled: bool) {
        if self.items.peek().get(&index) == Some(&disabled) {
            return;
        }
        self.items.write().insert(index, disabled);

        let Some(focused) = *self.current_focus.peek() else {
            return;
        };
        if disabled && focused == index {
            // Focus cannot remain on a disabled item.
            self.blur();
        } else if !disabled && self.items.peek().get(&focused) == Some(&true) {
            // Focus is parked on a known-disabled item; advance to the nearest enabled one.
            if let Some(next) = self.next_focus_skipping(focused) {
                self.set_focus(Some(next));
            }
        }
    }

    pub(crate) fn remove_item(&mut self, index: usize) {
        let removed = self.items.write().remove(&index).is_some();
        if removed && (self.current_focus)() == Some(index) {
            self.set_focus(None);
        }
    }

    pub(crate) fn control_mount_focus(
        &self,
        index: usize,
        controlled_ref: Signal<Option<Rc<MountedData>>>,
    ) {
        if self.is_focused(index)
            && self.is_enabled(index)
            && let Some(md) = controlled_ref()
        {
            spawn(async move {
                let _ = md.set_focus(true).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{first_enabled, next_index, prev_index};

    #[test]
    fn next_advances_to_following_index() {
        let indices = [0usize, 1, 2, 3];
        assert_eq!(next_index(&indices, Some(1), false), Some(2));
        assert_eq!(next_index(&indices, None, false), Some(0));
    }

    #[test]
    fn next_at_end_stops_without_loop_and_wraps_with_loop() {
        let indices = [0usize, 1, 2];
        assert_eq!(next_index(&indices, Some(2), false), None);
        assert_eq!(next_index(&indices, Some(2), true), Some(0));
    }

    #[test]
    fn prev_moves_back_and_handles_start() {
        let indices = [0usize, 1, 2];
        assert_eq!(prev_index(&indices, Some(2), false), Some(1));
        assert_eq!(prev_index(&indices, Some(0), false), None);
        assert_eq!(prev_index(&indices, Some(0), true), Some(2));
        assert_eq!(prev_index(&indices, None, true), Some(2));
        assert_eq!(prev_index(&indices, None, false), Some(0));
    }

    #[test]
    fn next_prev_skip_gaps_in_sparse_indices() {
        // Disabled items are absent from the list, so navigation skips them.
        let indices = [0usize, 2, 5];
        assert_eq!(next_index(&indices, Some(0), false), Some(2));
        assert_eq!(next_index(&indices, Some(2), false), Some(5));
        assert_eq!(prev_index(&indices, Some(5), false), Some(2));
        // current points at an absent (disabled) index — resolves relative to neighbours.
        assert_eq!(next_index(&indices, Some(3), false), Some(5));
        assert_eq!(prev_index(&indices, Some(3), false), Some(2));
    }

    #[test]
    fn first_enabled_picks_lowest_enabled_index() {
        let mut items = BTreeMap::new();
        items.insert(0usize, true); // disabled
        items.insert(1usize, false); // enabled
        items.insert(2usize, false); // enabled
        assert_eq!(first_enabled(items.iter()), Some(1));
        assert_eq!(first_enabled(items.iter().rev()), Some(2));
    }

    #[test]
    fn empty_indices_yield_none() {
        let indices: [usize; 0] = [];
        assert_eq!(next_index(&indices, None, true), None);
        assert_eq!(prev_index(&indices, Some(0), true), None);
    }
}
