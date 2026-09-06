# Quality run — 2026-09-06

Lead: Claude (architecture, integration, shared hooks/utils/exports, final verification).
Rules and verification commands: see `AGENTS.md`. This file is the single source
of current decisions for this run; every agent reads it before starting.

## Baseline (before any edit, commit 83cf4e8)

- `cargo fmt --check`: clean.
- `cargo clippy --workspace --all-targets`: 0 errors, 213 warnings in ds-components
  (201 `useless_format`, 3 `unused_mut`, 3 `redundant_closure`, 3 `suspicious_else_formatting`,
  3 unused variables in tests, 1 `field_reassign_with_default`).
- `cargo test --workspace`: all pass (ds-components 246 + 2 + 4 SSR, ds-utils 81, 0 failures).
- Feature matrix / wasm / server / producer-pro: see "Baseline matrix" below.

## Consumers (compatibility needs)

| App | Dep | Features used |
|---|---|---|
| producer-pro | path `../design-system/sdk`, default-features=false | (workspace) + `web` |
| sujag | git rev 83cf4e8 (= current HEAD) | libs: `form`; news-room app: `data-table, date-picker, display, nav, web` |
| kyosk | git, tracks `main` | libs: `form`, `date-picker`; apps: `date-picker, display, feedback, form, nav, overlay` (+`web`) |

Most-used symbols (consumer grep): FormProvider, FormFields, Icon/IconName, Form, FormSchema,
FormOptions, Button, TextInput, use_form, Flex*, FieldType, Link, Column, FormValue, NumberInput,
Title, PasswordInput, EmptyState, FormSubmit, Card, Back, Avatar, Switch, StatusDot, use_toast,
Modal, PageLoader, Grid, DateTimePicker, DatePicker, SegmentedControl, RouteTransitionOutlet,
ResourceView, StepDefinition, AppShellProvider, Alert, Popover/PopoverConfirm, ListView, TextArea.

## Shared conventions decided for this run

Reviews landed: `scratchpad/review_{A,B,C,D,E,F}.md` (session scratchpad). Reconciled rules:

1. **Overlay dismissal** (lead-owned, DONE in `hooks/dismiss.rs`): escape + outside stacks are
   app-wide (root context) and *open*-gated. Components that stay mounted while closed use
   `use_escape_listener_open(is_open, ..)`; `use_escape_listener` keeps its always-open
   semantics for external callers. Document/window listeners attach only while open.
   `use_dismiss_on_viewport_change(is_open, panel_el, ..)` ignores scrolls inside the panel.
2. **Focus restore**: `hooks::use_focus_restore(is_open)` (extracted from `use_overlay`). Use it
   in Popover and LoadingOverlay; do not reimplement.
3. **Autofocus**: `hooks::use_gated_autofocus(autofocus) -> Callback<MountedEvent>`; `AutofocusGate`
   now lives in hooks (still re-exported as `components::AutofocusGate`). TextArea must use it.
4. **Ids**: `use_unique_id()` only; no `static AtomicU32` counters, no random ids.
5. **Labels**: `Label`/`FieldLabel` omit `for` when the target id is empty. `aria-describedby`
   is emitted only when the referenced element renders (i.e. when invalid).
6. **Errors**: inline field error `role="alert"` without a contradicting `aria-live`; drop
   `action: "javascript:void(0)"` from form elements (CSP), `prevent_default` already handles it.
7. **State**: no derived state in signals; captured-once props that can change go through a
   signal (`use_signal` + compare-and-set) or `use_memo` over `ReadSignal` props.
8. **Cleanup**: any timer/listener/reader owned by a component is cancelled in `use_drop` or
   via `use_effect_with_cleanup`. Callbacks that write signals after an await/timeout must
   tolerate unmount (prefer `Rc<RefCell<..>>` batch state over signals for async writers).
9. **Text vs bytes**: character counts use `chars().count()`.
10. **Attributes**: interactive leaf components accept `#[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>` where consumers need `id`/`aria-*` (additive). `Link` has it now.
11. **Appearance is frozen**: no class-string changes that alter rendering (e.g. B-12 field
    surface tokens, toast `break-words`) — deferred.
12. **Gating**: new DOM code uses `#[cfg(feature = "web")]`; existing `target_arch` guards stay.
13. **Tests**: every bug fix ships a unit test (pure logic) or an SSR pin in `components/tests/`.
14. **Clippy**: `useless_format` inside `rsx!` string attributes is macro noise; crate-level
    allow added by lead. No other new warnings.

## Assignments

Status: `review` → `findings` → `implementing` → `verify` → `done` / `deferred`.

| # | Family | Files (exclusive edit ownership during implementation) | Reviewer | Implementer | Verifier | Status |
|---|---|---|---|---|---|---|
| A | Form core | `components/src/form/**`, `components/src/field_name.rs`, `macros/src/form_fields.rs`, `macros/src/form_options.rs` | R-A | I-A | V-A | done |
| B | Inputs & controls | `components/src/{input,input_types,select,textarea,checkbox,radio,slider,number_stepper,file_upload,toggle_card,chip_toggle,color_swatch_picker,password_strength,label,use_action_feedback}.rs` | R-B | I-B | V-B | done |
| C | Overlays, hooks, nav, feedback | `components/src/hooks/**`, `components/src/{dropdown,modal,popover,tooltip,toast,portal,placement,focus,nav_tabs,tabs,segmented_control,nav_sliding_indicator,app_shell,route_transition_outlet,loading_overlay,progress,status_dot,alert,fallback_view}.rs` | R-C | I-C | V-C | done |
| D | Calendar & date pickers | `components/src/calendar/**`, `components/src/date_picker/**`, `utils/src/format/date.rs` | R-D | I-D | V-D | done |
| E | Data, display, charts, core primitives | `components/src/{data_table,list_view,resource_view,accordion,avatar,empty_state,section_header,stat_tile,badge,button,card,copyable,icon,icon_bubble,layout,link,separator,spinner,text,title,back}.rs`, `components/src/charts/**` | R-E | I-E | V-E | done |
| F | Stepper, utils, macros | `components/src/stepper/**`, `components/src/step_dots.rs`, `utils/**` (except `format/date.rs`), `macros/src/{case,cfg_feature,steps,lib}.rs` | R-F | I-F | V-F | done |

Lead-owned (not delegated): `components/src/lib.rs`, `components/src/form/mod.rs`,
`components/src/hooks/mod.rs`, all `Cargo.toml`, `sdk/**`, `AGENTS.md`, this file.

## Findings log

Full reports in scratchpad `review_A..F.md`. Implementation briefs below list what is in scope
per assignment; anything not listed is either "already good" or deferred (see Deferred).

### Impl-A — Form core (owner: implementer A; files per assignment table + `components/README.md` Forms section)
- A1 BUG typed `has_value` non-reactive → reactive twin (`typed/form.rs:388-422`); test.
- A2 BUG registry loses entries on row swap/remove → owner token in `RegisteredField`; test with `rows.swap`/`rows.remove`.
- A3 BUG stale `FieldContext.name` after re-key → `provide_context` again in the re-key branch (typed + dynamic `FormField`).
- A4 RISK conditional hooks in both `FormProvider`s → unconditional hooks, branch inside.
- A5 RISK ROOT-scoped `CopyValue`s → `CopyValue::new` (provider scope) in both providers and `DynamicForm::default`.
- A8 RISK `AuxState::clear_field` under `all_written` → unwritten-prefix set; test.
- A10 QUALITY dedupe `typed/view.rs` vs `view/components.rs`: additive `FieldBinding { error, clear }`, one `FieldActions`/`FormError`, one `pub(crate) fn form_element(..)`. Each flavour keeps its own `clear` semantics inside its callback (no behaviour change).
- A11 QUALITY delete stale `view/tests.rs` percent/number/phone re-implementations.
- A12 QUALITY derives: unknown `#[field(..)]` key → `syn::Error`; zero unit variants → compile error; doc note for data variants.
- Nits: `required_message` helper; deterministic parent-error bubble (sort keys); `role="alert"` w/o `aria-live`; drop `action: "javascript:void(0)"`; `aria-describedby` only when invalid; `touch` redundant check; `toggle_optional` drop `Send + Sync` bound.
- README Forms section: document the typed API as primary (matches `form/mod.rs`), short `form::dynamic` subsection; fix prop tables (`FieldHandle`, lenses, `label:` override).
- Tests listed in review A "Test gaps" (typed + dynamic) — add the ones covering A1/A2/A3/A8 at minimum plus `reset()` after `default_values`, `.some()` lens, error precedence.

### Impl-B — Inputs & controls (owner: implementer B)
- B1 BUG closed Select listbox in tab order/a11y tree → option `tabindex=-1`, closed list `invisible` + `aria-hidden`; SSR pin.
- B2 BUG Select never closes on focus-out → `onfocusout` with `relatedTarget` check (web).
- B3 RISK FileUpload → `use_unique_id`, `Rc<RefCell<Batch>>` state, abort readers on drop, `onerror` counts + `on_error`.
- B4 RISK stale captured `behavior`/`first` → signal compare-and-set (`input_types.rs`, `color_swatch_picker.rs`).
- B5 RISK Slider → pointer events + keyboard + `role=slider` ARIA; keep `touch-none`; pure step fn + test.
- B6 RISK `Label` `for=""` → omit when empty; fix `radio.rs` call.
- B7 RISK password byte counts → `chars().count()`; tests.
- B8 RISK unnamed controls → `attributes` spread on `Switch` and `Radio`; `aria-label`/`aria-pressed` on swatches; `aria-label`s on stepper buttons; `saturating_*` in `stepped` + test.
- B9 QUALITY SelectOption register in `use_effect`; `index_of_key` via `peek`.
- B10 QUALITY `SelectContext::close()` single close path (clears search); searchable input `onclick` reopens.
- B11 QUALITY TextArea uses `use_gated_autofocus` (hook exists).
- B13 duplication: `input_types.rs` props/rebuild via `macro_rules!`; radio class consts; select value→label lookup; drop `merge(&[&props.class])` no-op and unused `form_control_fallback_merge`.
- B14 RadioGroup uses `FocusState::focus_next/prev/first/last`.
- B15 nits: `inputmode` before user attrs; `filter_percent` sign; checkbox `peek`; select `span role=button` nesting; `accept` None when unset.
- Tests: `parse_multi_value`, `toggle_value`, `move_active`, `typeahead`; password tiers; stepper overflow; percent `normalize`; SSR pins for Select closed state, Label omits `for`, FileUpload `label[for]==input[id]`, RadioGroup roving `tabindex`.

### Impl-C — Overlays, nav, feedback (owner: implementer C; hooks are lead-owned and DONE)
- C3 RISK NavTabs timer → `use_effect_with_cleanup` + `clear_timeout`; re-measure on `resize`.
- C5 RISK `Anchor::resolve` cross-axis clamp; tests.
- C6 RISK Modal focus trap (`onkeydown` Tab wrap over focusable descendants); comment the save-before-mount ordering.
- C7 RISK Popover: `use_focus_restore(is_open)`, focus first focusable on open.
- C8 RISK LoadingOverlay: `use_focus_restore`; scroll-lock refcount (root context counter).
- C9 QUALITY Tabs `onfocusout` → `blur()`; `aria-controls` only for rendered panels.
- C10 QUALITY Dropdown: `role="menuitem"` on the `Link` variant (Link now has `attributes`); ArrowRight/Left for `DropdownMenuSub`; item closes go through `close()`; `CHEVRON_LEFT` naming.
- C12 QUALITY Tooltip: `use_escape_listener_open(visible, hide)`; `aria-describedby` only while visible.
- C13 QUALITY NavTabs: `aria-current="page"` on active link; optional `aria_label` prop on `nav`.
- C15 doc: Modal must be unmounted to close.
- Tests: `is_path_active`/`routable_path_only`, `dropdown_panel_class`, `Anchor::resolve` clamp; SSR pins: Modal/Popover/Tooltip render no panel on server, Tabs ids/aria-selected, SegmentedControl radiogroup + single tabindex=0, Alert role, Progress aria-valuenow clamp, Toaster aria-live, DropdownMenu closed panel `hidden`.
- Out of scope (deferred): toast pause-on-hover/assertive/nowrap, tooltip wrapper tabindex, `Placement::classes`/`Align` removal, dropdown `open` prop trio.

### Impl-D — Calendar & date pickers (owner: implementer D)
- D1 BUG utc mode uses current offset → `local_offset_at(instant)` with refinement pass; keep `_with_offset` helpers; test two instants/two offsets.
- D2 BUG `Date::default()`/`DateTime::default()` UTC today vs calendar local today → both via `calendar::today()`.
- D3 RISK SSR offset nondeterminism → non-wasm returns `UtcOffset::UTC`; SSR pin.
- D5 a11y: `aria-label` on prev/next/edit-toggle/spinners, `aria-expanded` year toggle, `aria-pressed` AM/PM, `role=group`; range tabs `role=tab`/`aria-selected`; trigger gets `id`/`aria-haspopup=dialog`/`aria-expanded`/`aria-invalid`/`aria-describedby` via additive `*PickerBase` props wired from `binding` in the `*Control` wrappers; weekday header `abbr`; MonthView/TimeGrid cells `role=button`+`tabindex`+Enter/Space.
- D7 RISK `parse_date` 2-digit years → reject year<1000 unless 4-digit token; `parse_datetime` hour/minute failure → `None`; tests.
- D8 QUALITY TimeGrid `end.max(start)`; `assign_columns` tests.
- D9 QUALITY shared `out_of_range`, `use_optional_signal`, `PickerTrigger`, `PickerDialog` in `shared.rs`; legacy pickers get additive `label` prop.
- D10 QUALITY reset staging on open rising edge (effect) so external `is_open` works.
- D11 QUALITY `use_calendar_state()` hook; `CalendarState::new` delegates, `#[doc(hidden)]`.
- D12 QUALITY `parse_datetime` seconds via new fn (keep pub signature); add `FromStr` for `Date`/`DateTime`, deprecate panicking `From<String>`.
- D13 nits: `unwrap_or` consistency; range OK with start-only commits `(s,s)`; remove duplicate serde tests in `grid.rs`; single `suspicious_else_formatting` allow or fmt.
- Deferred: D4 range wire format (README ISO vs code MDY — product), D6 roving grid tabindex/cross-month arrows, `is_open`→`open` rename, week-start i18n.

### Impl-F — Stepper, utils, macros (owner: implementer F)
- F2 BUG `mark_submitted()` before validation → move into the submit closure; skip path marks only when `get_data()` is `Some`.
- F3 BUG `SummaryField.transform` dead → deferred (type change); remove from README table for now.
- F4 RISK ROOT-scoped signals per mount → provider-scope owner (`current_scope_id()` recorded in `InternalStepCtx::new`, `Signal::new_in_scope(v, owner)`), `Step` signals allocated in `ctx.owner`.
- F5 RISK render-time unconditional `steps.write()` → `peek` check first; effect compares before `set`.
- F6 RISK spurious `on_step_change((0, initial))` → seed `prev_index` with current, skip on restore.
- F7 `clear_draft` re-persists → gate persist on non-empty map.
- F8 a11y: `nav > ol` (no `role=list` on nav), `aria-current` on the focusable button, dots get `aria-label`, `StepDots` gets `role=list`/`aria-current`, `ClearDraftButton` renders a real `button` (same classes), `StepSuccess` `tabindex=-1` + focus on completion, `SummarySection` edit `aria-label`.
- F10 README/doc drift: utils README (`DsError`, `money`), macros README (drop route macro, Leptos syntax → rsx), components README Stepper section shows the dynamic form (`use_dynamic_form`) until F1 lands.
- F11 `Steps` derive propagates `syn::Error`.
- F12 `pascal_to_title` via new `split_words_for_title` (acronym/digit aware); `split_pascal` untouched (serde parity).
- F13 share dot class consts between `step_dots.rs` and `stepper/view.rs`; comment on `snake_to_title` duplication.
- F14 additive `From<String>`, `From<&str>`, `From<serde_json::Error>`, `From<validator::ValidationErrors>` for `DsError`.
- F15 nit `peek` `item_refs`. F16 `major_to_minor` strips `,`.
- Tests: `parse_persisted`/serialize split + tests; `StepDefinition` defaults; `case.rs` unit tests (acronyms, rename_all parity); utils edge inputs; SSR pins for Stepper/StepProgress/StepNav/StepDots/StepSuccess.
- Deferred: F1 typed-form support for `MultiStepForm`/`StepNav`/`SummaryField` (store trait), F3, StepMeta/StepDefinition unification, F9 Visited vs Completed, draft versioning.

### Impl-E — Data, display, charts, core primitives, rich text (owner: implementer E)
- E1 BUG RTE link href scheme allowlist (`safe_href`, unit tests).
- E2 BUG DataTable slot clicks bubble into row toggle → `div.contents` with `stop_propagation` around row_left/select/actions slots.
- E3 BUG `copy_to_clipboard` timer writes dropped signal → rebuild on `hooks::use_timer` (lead added: `use_timer(delay_ms, on_elapsed) -> Timer { start(), cancel() }`, cancel-on-unmount). Move `Copyable::ResetTimer` onto it too; `copy_to_clipboard` becomes a hook-shaped helper (`use_copy_feedback` or similar, `pub(crate)`) — coordinate: the two form call sites (`form/typed/view.rs`, `form/view/components.rs`) are implementer A's files; keep the existing `copy_to_clipboard(text, set_copied)` signature working (use `try_write`) so A's files need no change now, and note the follow-up.
- E4 BUG AreaLineChart flat-zero ticks + non-finite points → filter finite, `y_max = 1.0` for all-zero; tests for `nice_ticks`, `polyline_d`, `compute_area_render`.
- E5 RISK ListView paging pending state → disabled buttons while pending, `aria-busy`, pagination under `ListError` when page>1.
- E6 document `fetch`/`per_page` captured at mount (README ListView section).
- E7 RISK DataTable hooks after conditional return → hoist; `SelectionState` effect unconditional; skeleton columns from registry.
- E8 RISK `Col` re-registers static fields via `use_reactive`; document `render` captured at mount.
- E9 RISK disabled pagination links focusable → `aria-disabled` + `tabindex=-1`, `nav[aria-label=Pagination]`.
- E10 RISK Avatar `errored` reset on `src` change.
- E11 RISK Donut/StackedBar NaN/>100% → clamp; extract `donut_arcs` + tests.
- E12 RISK RTE empty detection via `inner_text().trim()`, `aria-labelledby`, `aria-pressed` on toolbar buttons, toolbar disabled with editor.
- E13 Accordion index compare-and-set.
- E14 a11y additive: `aria-sort` on sortable headers, expand chevron as `button` + `aria-expanded`, clear-selection `aria-label`; charts default `aria-label` from data + `<title>`; Copyable sr-only status; Card variant path keyboard parity; `Icon` `aria-hidden`/`focusable=false`.
- E15 dedupe: `pub(crate) fn ListStatus` (empty/error/spinner) used by data_table + list_view; collapse `ListViewBody`/`ListViewBodyReady`; drop dead `SuspenseBoundary`; pagination class consts.
- E16 state: derive `page_ids` from items (no render-time write); deterministic selection order; RTE `value_sig` guarded set; Accordion via `use_controlled` (optional).
- E17 nits: `merge` for class in accordion/back; `back.rs` class fix; stale `#[allow(unused)]` in separator; keys in area chart legends from index.
- Tests: units listed in review E "Test gaps"; SSR pins for DataTable (header/rows/empty/pagination/skeleton), ListEmpty/ListError, Button aria-busy, Separator, Avatar, Accordion ids, EmptyState, Copyable, Badge, RTE SSR textarea fallback.
- Deferred: D1 table semantics, D2 controlled `page`, D3 Send+Sync aliases, D4 `col()` deprecation, D5 `&'static str` props, D6 execCommand/sanitising, D7 Card dual path.

## Verification log

- V-B (controls): 7 findings. Fixed by lead: HIGH Select `onfocusout` closed the list on option mousedown in Safari/Firefox-macOS → `onmousedown: prevent_default` on options and tag-remove spans; MEDIUM FileUpload leaked one `once_into_js` closure per file and the aborted batch → single `Closure::wrap` handler per file owned by `Batch.readers`; NIT slider Home/End now return the exact bound; select registration comment corrected. Accepted as-is: rustdoc drift in the `typed_input_props!` macro (nit), radio `onfocus` churn (harmless), no README row for standalone `Radio` `attributes`. Consumer checks (sujag workspace, kyosk UI crates) passed.

- V-A (form core): all brief items verified with HEAD-vs-tree SSR markup diff (only the three intended convention changes). Two LOW findings fixed by lead: `Rows::push/insert` under the `default_values` blanket now start pristine (`AuxState::mark_row_pristine`, test added); `use_action_wiring` doc trimmed (a `None <-> Some` flip of `action`/`loading` still does not refresh `FormContext`, pre-existing). INFO: integration tests were not feature-gated, so `--no-default-features --features form` failed to build other families' SSR tests → lead added `#![cfg(feature = ..)]` to every `components/tests/*.rs`. Consumer checks (sujag, kyosk UI) passed.

- V-C (hooks + overlays/nav): shared hook model verified against dioxus scheduling at the pinned rev (no re-entrant borrows, closures never dropped mid-invocation, escape ordering scenario traced). HIGH finding fixed by lead: the Modal focus trap intercepted Tab from `Portal`ed children (PopoverConfirm inside a Modal, kyosk tickets/coupons) because Dioxus bubbles by VDOM ancestry → `trap_tab` now returns early unless `document.activeElement` is inside the modal panel's DOM subtree. LOW fixed: `popover.rs` import gating warning in consumer builds; dead `Controlled` struct removed; pure stack helpers gated `any(web, test)`. Documented behaviour change: a hovered Tooltip now takes the first Escape (WCAG 1.4.13), the next Escape reaches the overlay beneath. Unverified/accepted: Popover focus restore after scroll-dismiss may scroll the trigger back into view; `focusable_descendants` does not exclude `visibility:hidden` ancestors. Consumer checks (sujag, kyosk UI) passed.

- V-D (calendar/date pickers): D1 offset math traced for NY/Berlin DST edges and pinned by verifier-added tests; no high/medium findings. LOW fixed by lead: `MonthView` day cell was `role=button` wrapping event-chip buttons → role/tabindex/keydown moved to the day-number span. Documented limitation: a nonexistent spring-forward wall time (02:30 in the gap) is stored one hour earlier rather than resolved forward. Nits accepted: range tabs lack `aria-controls`; README range wire format stays deferred (D4). Consumer checks passed.

- V-E (data/display/charts/core): two HIGH regressions found with verifier-added async SSR tests and fixed by lead: (1) DataTable `loading` toggle wiped the column registry (E7 mounted `Col` children in every branch; Dioxus creates the new template before dropping the old one, so the old `Col`'s `use_drop` deleted the fresh entry) → `ColumnRegistry.owners` token map, `use_drop` removes only its own registration; (2) ListView empty state lost the title row and `action` (E15 collapse) → empty arm wrapped in `ListViewFrame`. LOW: `aria-sort` on a link header is invalid without `columnheader` → removed until table semantics (deferred D1); `tel:` added to `safe_href`; `core_ssr.rs` no longer gated on `rich-text` (only the RTE test is). Verifier tests kept as regression pins. Consumer checks passed.
- Cross-family pass (lead): `hooks::use_prop_signal` replaces five hand-rolled compare-and-set prop mirrors (input_types, color_swatch_picker, accordion, rich_text_editor, date_picker `use_optional_signal`); `hooks::root_context` replaces the duplicated root-context lookup in dismiss.rs and loading_overlay.rs; SSR tests feature-gated per family.

- V-F (stepper/utils/macros): all brief items verified; F4 drop order traced at the pinned rev and pinned by a verifier-added unmount test. LOW fixed by lead: `use_any_step_ctx` built six boxed callbacks per provider render → built once in `use_hook`; inactive dot literal now shared via `dot_inactive!`. Documented: an `AnyStepCtx` handle used after its provider unmounts now panics (was silently ROOT-owned before) — README Stepper section. Consumer checks passed.

## Final verification (working tree, uncommitted)

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets` | 0 warnings (baseline: 213) |
| `cargo clippy -p ds-components --target wasm32-unknown-unknown --features web` | 0 warnings (baseline: build failed) |
| wasm32 `--no-default-features --features form,web` / `overlay,web` / `nav,web` / `date-picker,web` / `data-table,rich-text,web` | 0 warnings |
| `cargo test --workspace` | 528 passed, 0 failed (baseline 333) |
| `cargo test -p ds-components --no-default-features --features <each family>` | all pass (SSR tests now feature-gated) |
| `cargo check -p ds-components --no-default-features` + each family alone; `design-system --features server`; `ds-macros` each feature alone | clean |
| sujag `cargo check --workspace` (patched to tree) | OK |
| kyosk UI crates (patched to tree) | OK (`kyosk-server` excluded: needs D1 at compile time) |
| producer-pro `cargo check --workspace` | FAIL, consumer-side (pre-existing; see Deferred → Consumers) |
| Browser interaction checks | not run (no browser harness in this repo); pointer/focus/DST paths verified by reasoning + wasm compile + SSR pins |

## Deferred (breaking changes / product decisions)

Form
- Root `use_form` is the typed store; producer-pro still calls the dynamic API (`set_string_value`, `get(..dot_array..)`, `get_data`) through it. Migrate producer-pro to `form::dynamic::use_dynamic_form` or re-export the dynamic hook under another root name.
- `FormSubmit` Copy → Clone (`Rc<dyn Fn>`) so `FormSubmit::from(action)` in render bodies stops allocating scope-owned `CopyValue`s per render (kyosk does this in ~6 screens).
- Pristine-required semantics: a non-`Option` `bool`/enum field without `default_values` blocks submit with "X is required".
- Unused pub API candidates: `Field::resolve`, `dot_array`, `toggle_optional`/`ToggleDefault`, `LensExt::some`, `impl Default for DynamicForm`.
- Typed flavour gaps: no `RadioGroup`/multi-select; `label:` override only on `TextInput`.
- `FieldContext` stays name-only; dynamic controls re-keyed by path still need a `key` remount.

Controls
- Naming: `on_value_change`/`on_checked_change`/`on_select`/`on_change`/`on_toggle`; `disabled` typed `ReadSignal`/`bool`/absent; `NumberStepper` two-way `Signal<i64>`.
- Field surface tokens: TextArea/Select hard-code `rounded-lg border border-input bg-transparent` while Input uses `--field-*` (appearance change for themed apps).
- Password reveal button `tabindex=-1`; caret restoration for formatted Number/Phone/Percentage inputs; `Radio` vs `CheckboxBase` shape; `ToggleCard` `aria-pressed` vs `role=switch`.

Overlays / nav / feedback
- `DropdownMenu.open: Option<Signal<bool>>` → `open/default_open/on_open_change` trio; unify Modal `on_close` / Popover `on_open_change`.
- Remove unused public `Placement::classes` / `Align`.
- Toast: pause-on-hover, `Error` → assertive, `whitespace-nowrap` wrapping (default/appearance changes).
- Tooltip wrapper `tabindex=0` default (double tab stop when the child is a Button).
- Popover dismiss-on-scroll vs re-anchoring.
- `route-transition-outlet` / `animate-wip-float` keyframes live in consumer stylesheets, not `theme.css`.

Calendar / date pickers
- `DateRangePicker` wire format: README says ISO, code emits `MM-DD-YYYY`.
- Roving `tabindex` + cross-month arrow navigation in the day grid; `is_open` → `open` rename; week-start/i18n; `From<String>` panics (now `FromStr` exists, removal needs a major bump).
- Spring-forward gap hour: a nonexistent wall time is stored one hour earlier rather than resolved forward.

Data / display / charts / rich text
- DataTable real table/grid semantics (`aria-sort` removed until then); controlled `page` prop for ListView (drops the `key` remount hack); drop `Send + Sync` from DataTable closure aliases on wasm; deprecate `col()`/`TableColumn`; `&'static str` props → `Cow`; RTE on deprecated `execCommand` with unsanitised output; `Card` dual render path.

Stepper / utils / macros
- `MultiStepForm`/`StepNav`/`SummaryField` take `DynamicForm<T>` while the root `use_form` is typed (README example now uses `use_dynamic_form`); typed support needs a store trait.
- `SummaryField.transform` is dead (`EventHandler<String>` cannot transform) — prop type change.
- Unify `StepMeta`/`StepDefinition` and `StepKey`/`StepId`; distinguish `Visited` from `Completed`; draft versioning; `clamp_percent` suffix inconsistency (frozen by tests).

Consumers
- producer-pro does not build against this tree: its `libs/utils` uses `design_system::{Date, DateTime}` with `default-features = false` and no `date-picker` feature (predates per-family flags), and its wizard calls the dynamic form API through the typed root `use_form`. Needs consumer-side feature/API updates.

## Baseline matrix

| Config | Result |
|---|---|
| `--no-default-features` and each family alone | OK (`form`-implying families: 6 pre-existing warnings, unused vars/mut in form tests) |
| `--features web` (host) and `--target wasm32-unknown-unknown --features web` | **FAIL** before edit: dioxus-web `history.rs` needs `web-sys/Location`, only enabled by dioxus-web's `devtools` feature. Fixed by lead: `web` feature now adds `web-sys/Location` (components/Cargo.toml). Consumers pass `devtools` to dioxus so they never hit it. |
| clippy wasm32 `--features web` | 216 warnings (205 `useless_format`, 5 `suspicious_else_formatting`, 3 `type_complexity`, 3 `redundant_closure`) |
| `design-system --features server` | OK |
| producer-pro `cargo check --workspace` | **FAIL, pre-existing / consumer-side**: producer-pro's `libs/utils` uses `design_system::{Date, DateTime}` with `default-features = false` and no `date-picker` feature. producer-pro's last commit (2026-07-11) predates the per-family feature flags. Gap recorded; not fixed here (consumer repo). |
| sujag `cargo check --workspace` (patched to working tree via `--config 'patch."https://github.com/sam-rusty/dx-design-system.git".design-system.path="…/design-system/sdk"'`) | OK |
| kyosk UI crates (`kyosk-web kyosk-mobile shared-ui authentication account events workspaces utils`, same patch) | OK. `kyosk-server` fails on `sqlx_d1 WorkerCrashed` (needs D1 at compile time) — unrelated to this library, excluded. |
