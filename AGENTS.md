# AGENTS.md — stable rules for working in this repo

Shared Rust/Dioxus component library. Workspace crates: `macros` (proc macros),
`utils` (`ds-utils`), `components` (`ds-components`), `sdk` (`design-system`
facade that re-exports all three). Consumers import the facade under the alias
`components` (`use components::...`); the derive macros emit `components::`
paths, so root re-exports in `components/src/lib.rs` are load-bearing.

## Compatibility rules (public API is frozen unless the owner says otherwise)

- Do not rename, remove, or re-type any `pub` item re-exported from
  `components/src/lib.rs`, `components/src/form/mod.rs`, `utils/src/lib.rs`,
  `macros/src/lib.rs`, or `sdk/src/lib.rs`. Adding items is fine.
- Preserve prop names, prop defaults, callback signatures and firing order,
  rendered class names / `data-name` attributes (consumers style against them),
  and every supported feature-flag combination.
- Deprecated items (`InputSize`, `StepDefination`) stay until a major bump.
- Feature flags are additive. A module compiled under feature `X` may only use
  modules that `X` implies (see `components/Cargo.toml [features]`). Core
  (always-compiled) modules must not depend on any feature-gated module.
- DOM/`web-sys` access lives behind `#[cfg(target_arch = "wasm32")]` (or the
  `web` feature where the crate already does so); everything must compile and
  render under SSR (`dioxus-ssr`, non-wasm target).
- `README.md` in each crate documents usage only. Update a section only when
  changing that component's API.
- Consumers: `../producer-pro` (path dep, `default-features = false` + families),
  `../sujag` (git dep pinned to a rev), `../kyosk` (git dep tracking `main`).
  Pushing to `main` affects kyosk on its next `cargo update`.

## Conventions

- Component files: one component family per file/module; props struct named
  `<Component>Props`; `#[component]` fns; `pub(crate)` modules with `pub use`
  re-exports from `lib.rs`.
- Shared behavior lives in `components/src/hooks/`: `use_controlled`,
  `use_prop_signal` (mirror a prop into a signal for effects/memos),
  `use_outside_dismiss[_panel]`, `use_escape_listener[_open]` (use the `_open`
  variant for anything that stays mounted while closed), `use_overlay`,
  `use_focus_restore`, `use_dismiss_on_viewport_change`, `use_timer`/`use_timeout`,
  `use_unique_id`, `use_gated_autofocus`, `use_focus_*`, `use_effect_with_cleanup`,
  `root_context` (app-wide singletons). Prefer these over per-component
  reimplementations; do not add a new hook when one exists.
- Overlay stacks are root-scoped and open-gated: only the most recently *opened*
  overlay reacts to Escape/outside pointer-down. Listeners attach only while open.
- Registries filled by child components on mount (`Col`, `SelectOption`, `Step`)
  must be owner-aware: when a parent switches `rsx!` template, Dioxus mounts the
  new children before dropping the old ones, so an unowned `use_drop` deletes the
  fresh registration.
- Tab trapping and focus handlers must check `root.contains(activeElement)`:
  `Portal`ed children bubble events by VDOM ancestry, not DOM.
- Timers, event listeners, and DOM observers must be cleaned up on unmount
  (`use_drop` / `use_effect_with_cleanup`).
- Interactive elements need keyboard access and ARIA roles/states; labels bind
  via `for`/`id`; overlays manage focus and Escape.
- Tests: pure logic gets unit tests next to the code; render behavior gets SSR
  tests in `components/tests/` using `VirtualDom` + `dioxus_ssr::render`. Every
  integration test file starts with `#![cfg(feature = "<family>")]` so minimal
  feature builds still compile. Async render states (`use_resource`) need a
  `block_on` + `wait_for_work` harness (see `components/tests/data_table_ssr.rs`).
- Run `cargo fmt --all` before finishing; no new clippy warnings.

## Verification commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
# minimal-feature configurations
cargo check -p ds-components --no-default-features
for f in form calendar charts data-table date-picker display feedback nav overlay rich-text; do
  cargo check -p ds-components --no-default-features --features "$f" || break
done
# web / WASM
cargo check -p ds-components --target wasm32-unknown-unknown --features web
cargo check -p ds-components --target wasm32-unknown-unknown --no-default-features --features form,web
# server facade
cargo check -p design-system --features server
# consumers (git deps, checked against the working tree; back up/restore their Cargo.lock)
PATCH='patch."https://github.com/sam-rusty/dx-design-system.git".design-system.path="'"$PWD"'/sdk"'
(cd ../sujag && cargo check --workspace --config "$PATCH")
(cd ../kyosk && cargo check -p kyosk-web -p kyosk-mobile -p shared-ui -p authentication \
   -p account -p events -p workspaces -p utils --config "$PATCH")   # kyosk-server needs D1
# consumer (path dep) — currently fails on consumer-side feature/API drift, see docs/quality-run.md
(cd ../producer-pro && cargo check --workspace)
```
