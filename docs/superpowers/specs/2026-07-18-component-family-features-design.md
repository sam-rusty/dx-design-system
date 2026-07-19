# Per-family Cargo features for `ds-components`

**Date:** 2026-07-18
**Status:** Approved

## Goal

Let callers compile only the component families they use. All families on by
default (`default = ["full"]`); callers opt out with `default-features = false`
and an explicit feature list. Wins: compile time, dependency pruning
(`validator`, `time` become optional), and marginal binary size.

## Approach

Cargo features inside the existing `ds-components` crate (no crate split).
Each family gates its `mod` declarations and `pub use` re-exports with
`#[cfg(feature = "...")]`. The internal dependency graph is encoded as cargo
feature dependencies, so enabling a family transparently enables what it needs.

Rejected alternatives: crate-per-family split (better caching, but large
refactor — revisit if feature-based wins prove insufficient); gating only in
`sdk` re-exports (no compile-time win).

## Family map

Core (always compiled, no flag): `icon`, `icon_bubble`, `button`, `link`,
`spinner`, `text`, `title`, `label`, `layout`, `separator`, `card`, `badge`,
`placement`, `portal`, `focus`, `hooks`, `field_name`, `copyable`.
`copyable` is core because form imports `copy_to_clipboard`.

| Feature | Modules | Feature deps |
|---|---|---|
| `form` | form, input, input_types, checkbox, radio, select, textarea, number_stepper, slider, chip_toggle, toggle_card, color_swatch_picker, password_strength, file_upload, stepper, step_dots, use_action_feedback | `feedback`, `dep:validator` |
| `calendar` | calendar | `dep:time` |
| `date-picker` | date_picker | `calendar`, `form`, `overlay` |
| `charts` | charts | — |
| `data-table` | data_table, list_view, resource_view | `form`, `feedback` |
| `rich-text` | rich_text_editor | `form`, `overlay` |
| `nav` | nav_tabs, nav_sliding_indicator, tabs, segmented_control, back, app_shell, route_transition_outlet | `overlay`, `feedback` |
| `feedback` | toast, alert, tooltip, status_dot, loading_overlay, progress | — |
| `overlay` | modal, popover, dropdown | — |
| `display` | accordion, avatar, stat_tile, empty_state, section_header, fallback_view | — |
| `full` | meta-feature: all of the above | all families |

`default = ["full"]`. The existing `web` feature is orthogonal and composes
with any family set.

Notable merges vs. the first draft: `stepper` folds into `form` (mutual
imports would create an illegal cyclic feature dependency; stepper is
multi-step-form machinery).

## Dependency changes

- `validator` → optional, enabled by `form`. `field_name`'s test module uses
  the `FormFields` derive + `validator`, so it is gated
  `#[cfg(all(test, feature = "form"))]`.
- `time` → optional, enabled by `calendar` (date-picker gets it transitively).
- `ds_macros` re-exports (`FormFields`, `FormOptions`, `Steps`) gated behind
  `form`. The `ds-macros` crate itself remains an unconditional dependency.
- `serde_json` re-export stays unconditional (cheap, always a dep).

## Testing

- `cargo check --no-default-features` (core only) must pass.
- `cargo check --no-default-features --features <each family>` must pass for
  every family individually.
- Full test suite runs under default features, unchanged.
- `sdk` uses default features → unaffected.

## Non-goals

- Decoupling input components from the form binding layer (opting out of
  `form` while keeping `select`/`checkbox` is out of scope).
- Splitting `web-sys` features per family.
- Crate-per-family workspace split.
