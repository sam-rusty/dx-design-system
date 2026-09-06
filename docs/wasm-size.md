# WASM size — how to measure

Consumers ship one wasm per app; the library's share is whatever LTO keeps.
Measure on a real consumer, never on the library alone.

## Raw, named build (attribution)

```sh
cd ../kyosk   # git dep → patch to the working tree
PATCH='patch."https://github.com/sam-rusty/dx-design-system.git".design-system.path="'"$PWD"'/../design-system/sdk"'
cargo build --release --target wasm32-unknown-unknown -p kyosk-web \
  --config "$PATCH" --config 'profile.release.strip=false' --config 'profile.release.debug=false'
twiggy top -n 100000 -f csv target/wasm32-unknown-unknown/release/kyosk-web.wasm > top.csv
twiggy monos -n 40 target/wasm32-unknown-unknown/release/kyosk-web.wasm   # generic bloat
```

`strip = true` (the consumers' release profile) drops the name section, so
twiggy shows only `code[N]`; the two `--config` overrides keep names without
touching the consumer's manifest. Aggregate `top.csv` by the first path
segment of `Name` to get a per-crate table; group `Name` up to `::<` to see
per-instantiation cost. Restore the consumer's `Cargo.lock` afterwards.

## Shipped size (what the browser downloads)

`dx bundle --platform web --release` runs wasm-bindgen + `wasm-opt -Oz`.
For a git-dep consumer, drop a temporary `.cargo/config.toml` in the consumer
with the same `[patch]` (dx does not forward `--config`), run the consumer's
bundle target (`make news-room-assets` in sujag, `make web-assets` in kyosk),
and read `target/dx/<app>/release/web/public/wasm/*_bg.wasm`. Compare against
a `git worktree` of the baseline commit.

## What moved the needle (2026-09, kyosk-web, unstripped raw wasm)

| Change | Saved |
|---|---|
| `FormProvider<T>` / `use_action_wiring` → non-generic hook body | ~300 KB (15 instances × 22 KB → 1 × 16 KB) |
| Typed field bindings: 11 closures per `(T, lens)` → one `LensOps` vtable, path-keyed ops on `FormCore` | ~120 KB |
| `DataTable<T, K>` / `Col<T>` render + registry erased | ~120 KB |
| Overlay/timer/focus hooks: box the callback, `#[inline(never)]` body | ~150 KB (Popover 58→18 KB, DropdownMenu 68→37 KB, Select 64→32 KB) |
| `use_action_feedback` erased | ~30 KB |

Shipped (`dx bundle --release`, wasm-opt -Oz):

| App | raw before → after | gzip before → after |
|---|---|---|
| kyosk-web (15 form types, 4 tables) | 4.26 MB → 3.90 MB (−8.4%) | 1.39 MB → 1.37 MB (−2%) |
| sujag news-room (`panic=immediate-abort`) | 3.89 MB → 3.76 MB (−3.3%) | 1.37 MB → 1.35 MB (−1.4%) |

Monomorphized copies are near-identical bytes, so gzip hides most of the raw
win; the raw size is what the browser has to parse and compile.

Not ours: `regex` + `idna` (~250 KB) come from a consumer's `#[validate(email)]`;
`.rodata` is class-name / SVG literals and the dioxus interpreter JS.
