# A4. Web toolchain

Research annex to `docs/FOUNDATIONS.md`, which holds wherever the two disagree.
It surveys the toolchain for a Rust domain compiled to WebAssembly under a
browser front end: versions, WebAssembly features, threads, the monorepo,
rendering, geometry, units and tests. Section 8 sums up what the tree uses,
labeled **built**, **planned**, **not used** or **ruled out**, consistently
with FOUNDATIONS §3.10, §3.11 and §7.

Versions, dates and browser support are dated observations, recorded on
11 October 2026 from the npm registry, crates.io, `@mdn/browser-compat-data`
8.1.5 (BCD below) and published documentation. A local execution names the
toolchain it ran on.

Markers: [V] verified against the source named beside it (a registry, the
browser compatibility data, a package's own files, official documentation, or a
local execution); [S] assumed, an architectural judgment; [?] not established,
with the reason beside it. A sentence beginning **"Do not use"** marks a figure
or an argument that is wrong and easy to derive again.

---

## 0. Inventory of observed versions (11 October 2026)

### The survey machine [V, local execution]
| Tool | Version | Note |
|---|---|---|
| Node | v26.3.1 | a non-LTS line on that date, behind v26.11.1 |
| pnpm | 10.34.5 | `catalog:` works (tested) |
| rustc / cargo | 1.94.1 (e408947bf, 2026-03-25) | LLVM **21.1.8** |
| installed targets | `wasm32-unknown-unknown`, `x86_64-unknown-linux-gnu` | |
| wasm-pack / wasm-bindgen / wasm-opt / trunk | not installed by rustup | obtained separately, at the versions in the Rust table |

The project pins Rust 1.99.0 (`rust-toolchain.toml`, with rustfmt, Clippy and
the `wasm32-unknown-unknown` target) and Node 26 (`.nvmrc`), so that tools,
lints and formatting match between machines (FOUNDATIONS §4.4); `package.json`
pins pnpm 10.34.5.

On Rust 1.94.1, `rustup target list` offers `wasm32-wasip1`, `wasip1-threads`
and `wasip2`, but not `wasm32-wasip3`, which `rustc --print target-list` knows:
no precompiled standard library, so tier 3 and `build-std` [V, local execution,
Rust 1.94.1].

### JS/TS ecosystem [V, npm registry]
Of these, the tree uses typescript, vite, vitest, turbo, react, three,
`@types/three`, `@react-three/fiber`, `@react-three/drei` and
`@biomejs/biome` (§8); the others are candidates surveyed, not dependencies.

| Package | Version | Published | License |
|---|---|---|---|
| typescript | **7.0.2** (`next` = 7.1.0-dev.20261010.1) | 2026-07-08 | Apache-2.0 |
| vite | **8.3.4** | 2026-10-08 | MIT |
| rolldown | 1.2.13 | 2026-10-07 | MIT |
| vitest | **5.0.3** | 2026-10 | MIT |
| turbo | **2.11.7** (`canary` 2.11.8-canary.2) | 2026-10-02 | MIT |
| nx | 23.3.0 | 2026-10-07 | MIT |
| @moonrepo/cli | 2.6.1 | 2026-10-07 | MIT |
| react | 19.3.0 | 2026-09 | MIT |
| three | **0.186.1** | 2026-09-24 | MIT |
| @types/three | 0.186.0 | — | MIT |
| @react-three/fiber | 9.8.1 | 2026-09-24 | MIT |
| @react-three/drei | 10.7.9 | 2026-09-25 | MIT |
| @babylonjs/core / babylonjs | 9.30.0 (9.0.0 on 2026-03-26) | 2026-10-08 | Apache-2.0 |
| playcanvas | 2.23.2 | 2026-10 | MIT |
| manifold-3d | **3.5.4** | 2026-09-25 | Apache-2.0 |
| three-bvh-csg | 0.0.18 | 2026-02-17 | MIT |
| replicad / replicad-opencascadejs | 1.1.0 | 2026-09-04 | MIT |
| opencascade.js | 1.1.1 (`beta` = 2.0.0-beta.b5ff984) | **2020-09-27** | LGPL-2.1 |
| @jscad/modeling | 2.13.0 | 2026-02-22 | MIT |
| @gltf-transform/core | 4.5.1 | 2026-09-28 | MIT |
| leva | 0.10.1 | **2025-10-31** | MIT |
| tweakpane | 4.0.5 | **2024-11-03** | MIT |
| uplot | 1.6.32 | 2025-03-14 | MIT |
| @observablehq/plot | 0.6.17 | **2025-02-14** | ISC |
| zustand / valtio | 5.0.15 / 2.3.2 | 2026-08-13 / 2026-05-01 | MIT |
| @biomejs/biome | **2.5.15** | 2026-10 | MIT OR Apache-2.0 |
| eslint / typescript-eslint / prettier / oxlint | 10.12.0 / 8.71.1 / 3.9.10 / 1.87.0 | 2026-10 | — |
| @playwright/test | 1.64.0 | 2026-10-07 | Apache-2.0 |
| @tonejs/midi / midi-file | 2.0.28 / 1.2.4 | — | MIT |
| json-schema-to-typescript / quicktype / ajv | 16.0.0 / 26.0.0 / 8.20.0 | 2026-08-28 / 2026-07-20 / 2026-04-24 | — |
| @dimforge/rapier2d | 0.21.0 — **~4.0 MB uncompressed** (`-compat` ~11.8 MB) | 2026-09-25 | Apache-2.0 |
| @bytecodealliance/jco / componentize-js | 1.37.0 / 0.23.0 | 2026-10-06 / 2026-09-21 | — |
| vite-plugin-wasm | 3.6.0 (peer `vite ^2…^8`) | 2026-03-15 | MIT |
| vite-plugin-top-level-await | 1.6.0 | **2025-07-17** | MIT |
| coi-serviceworker | 0.1.7 | **2023-07-09** | — |

### Rust ecosystem [V, crates.io]
Of these, the tree uses wasm-bindgen, in `pa-wasm`, and wasm-pack, to build it;
`serde` and `serde_json`, outside this survey, carry the boundary's JSON
(FOUNDATIONS §7.2).

| Crate | Version | Updated | License / note |
|---|---|---|---|
| wasm-bindgen / js-sys / web-sys | **0.2.129** / 0.3.106 / 0.3.106 | 2026-09-25 | MSRV 1.81 |
| wasm-bindgen-cli | 0.2.129 | 2026-09-25 | |
| wasm-pack | **0.15.0** | 2026-05-15 | repository moved to the `wasm-bindgen` organization |
| wasm-bindgen-test | 0.3.79 | 2026-09-25 | |
| **wasm-bindgen-rayon** | 1.3.0 | **2024-12-21** | requires wasm-bindgen ^0.2.99, so compatible, but no release since |
| binaryen (wasm-opt) | **version_133** | 2026-09-21 | the `wasm-opt` *crate* stays at 0.116.1 (2024-03-31) |
| wasm-tools | 1.261.0 | 2026-10-02 | |
| trunk | 0.21.14 | 2026-10-06 | |
| rapier2d / rapier3d | **0.36.1** | 2026-10-04 | Apache-2.0 |
| parry2d / nalgebra / glam / simba | 0.31.1 / 0.35.0 / 0.34.1 / 0.10.2 | — | |
| ts-rs | **12.0.1** | 2026-01-31 | stable, maintained |
| **specta** | stable **1.0.5**; `2.0.0-rc.25` | rc.22 on 2025-01-12, rc.25 on 2026-05-07 | `specta-typescript` = **0.0.12** |
| typeshare | 1.0.5 | 2026-01-02 | multi-language |
| schemars | **1.2.2** | 2026-07-27 | MIT |
| tsify | 0.5.8 | 2026-08-23 | wasm-bindgen ^0.2.104 |
| criterion / proptest / quickcheck / approx | 0.8.2 / 1.12.0 / 1.1.0 / 0.5.1 | — | |
| cargo-nextest | 0.9.148 | 2026-10-08 | |
| bevy / wgpu | 0.20.0 / 30.0.1 | 2026-10-08 / 2026-08-22 | MIT OR Apache-2.0 |
| dioxus / leptos | 0.7.10 / 0.8.22 | — | |

### Current browsers [V, BCD 8.1.5]
Chrome **155** (2026-10-06) · Edge **154** · Firefox **157** (2026-09-29) ·
Safari and iOS Safari **27** (2026-09-14).

---

## 1. WebAssembly features: what is usable

### Default target features of `wasm32-unknown-unknown` [V, `rustc --print cfg --target wasm32-unknown-unknown`, Rust 1.94.1 and the pinned 1.99.0, identical]
```
bulk-memory, multivalue, mutable-globals, nontrapping-fptoint, reference-types, sign-ext
```
`simd128`, `atomics`, `exception-handling`, `tail-call`, `relaxed-simd` and
`wide-arithmetic` are available but not enabled by default [V, `rustc --print
target-features`, Rust 1.99.0]. The build enables none of them (built).

### Browser support [V, BCD 8.1.5]
| Feature | Chrome | Firefox | Safari | iOS Safari | Verdict |
|---|---|---|---|---|---|
| fixed-width SIMD (**simd128**) | 91 | 89 | **16.4** | 16.4 | ✅ available; not enabled by the build |
| bulk-memory | 75 | 78 | 15.1 | 15.1 | ✅ default |
| reference-types | 96 | 79 | 15.1 | 15.1 | ✅ default |
| multi-value | 86 | 78 | 13.1 | 13.4 | ✅ default |
| sign-ext / nontrapping-fptoint | 69 / 75 | 62 / 64 | 14.1 / 15.1 | — | ✅ default |
| threads-and-atomics | 74 | 79 | 15.2 | 15.2 | ⚠️ supported by browsers, **blocked on the Rust side** (§2) |
| extended-const | 114 | 112 | 17.4 | 17.4 | ✅ possible, not needed |
| tail-calls | 112 | 121 | 18.2 | 18.2 | ✅ possible, not needed |
| GC | 119 | 120 | 18.2 | 18.2 | ✅ not applicable (Rust does not use it) |
| exceptionsFinal | 137 | 131 | 18.4 | 18.4 | ❌ not needed with `panic = "abort"` |
| **relaxed-SIMD** | 114 | **146** | **preview** | **false** | ❌ **ruled out**: its fused multiply-add rounds once on hardware that has one and twice elsewhere (FOUNDATIONS §4.4) |
| **memory64** | 133 | 134 | **preview** | **false** | ❌ not enabled, and not needed under 4 GB |
| multiMemory | 120 | 125 | preview | false | ❌ |
| branch-hinting | 137 | 148 | 16 | 16 | ⚪ marginal |
| JSPI | 137 | 153 | 27 | 27 | ⚪ of no use here |

### Cargo profiles (built, `Cargo.toml`)
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
```
`opt-level = 3`, not `"z"` or `"s"`, because the work is compute-bound; the
development profile is optimized enough to use the bench, debug assertions kept
[S]. `pa-wasm` sets no `[package.metadata.wasm-pack]` options, so wasm-pack runs
`wasm-opt` with its defaults (built). `--split-debug-info`, in wasm-bindgen since
0.2.128, moves the DWARF into `*_bg.debug.wasm`, with `--debug-info-url`,
keeping the shipped module small without losing debuggability [V, wasm-bindgen
0.2.128 changelog]; not used. `wasm-split` exists (Binaryen,
`src/tools/wasm-split/`) [V, Binaryen repository]; of no use for a single
module.

---

## 2. Why WebAssembly threads are blocked

On Rust 1.99.0, the pinned toolchain, building the `pa-wasm` cdylib with shared
memory [V, local execution, Rust 1.99.0]:
```
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals \
           -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 \
           -C link-arg=--import-memory"
cargo build -p pa-wasm --target wasm32-unknown-unknown --release
```
```
warning: unstable feature specified for `-Ctarget-feature`: `atomics`
error: linking with `rust-lld` failed: exit status: 1
  = note: rust-lld: error: --shared-memory is disallowed by
          std-52dc94f81b9196d2.std.359a46decee746ed-cgu.0.rcgu.o
          because it was not compiled with 'atomics' or 'bulk-memory' features.
```
The standard library has to be rebuilt (`-Zbuild-std=panic_abort,std`), which
the rustc book calls "a Nightly Rust feature" [V, the rustc book, Platform
Support, `wasm32-unknown-unknown`]. Reproduce it on the cdylib: a library crate
performs no link step and passes with the same flags (FOUNDATIONS §3.10).
`wasm-bindgen-rayon`, the usual route to threads in the browser, has had no
release since December 2024. The 88-note runtime is planned as one module per
Web Worker, notes sharing nothing (FOUNDATIONS §3.10, §7.6).

### Web Worker or AudioWorklet
`AudioWorkletProcessor.process()` runs in render quanta of 128 frames, about
2.67 ms at 48 kHz (128 / 48 000), on a real-time thread: tempting for an
integrator at 1 to 10 kHz. Rejected for the simulation loop [S]: a missed
deadline becomes an audible dropout, allocation is forbidden there, and
`AudioWorkletGlobalScope.port` exists only in Firefox 138 and later [V, BCD
8.1.5, `api.AudioWorkletGlobalScope.port`]. The AudioWorklet is for audio, if
sound comes (an open question, FOUNDATIONS §9).

### From the workers to the renderer (planned, FOUNDATIONS §7.6)
One interface, two implementations [S]:
1. `crossOriginIsolated === true`: a lock-free single-producer ring in a
   `SharedArrayBuffer`, a triple buffer or a seqlock. `SharedArrayBuffer`:
   Chrome 68, Firefox 79, Safari 15.2; `SharedArrayBuffer.growable`: 111, 128,
   16.4; `Atomics.waitAsync`: Chrome 90, Safari 16.4, Firefox **145**, recent
   enough to be gated [V, BCD 8.1.5].
2. Otherwise, `postMessage` with transferable `ArrayBuffer`s, without a copy.

**The order of magnitude.** Per-step state crosses as a flat `f64` buffer,
poses at each step (FOUNDATIONS §7.2). With about 8 parts per note and a planar
pose of three `f64` (an angle and a position), 88 notes make
88 × 8 × 3 × 8 bytes ≈ 16.9 kB per frame, about 1 MB/s at 60 Hz, which
`postMessage` with transferable buffers carries [S]. Cross-origin isolation is
then an optimization, not a prerequisite. **Do not use 2.8 kB per frame**, sized
on one f32 angle per part: poses cross as f64 (FOUNDATIONS §7.2), and the pose
of a part riding on the wippen is more than an angle.

The bench's development server and its `public/_headers` file send
`Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy:
require-corp` (built), for that transport; the bench itself uses no
`SharedArrayBuffer`.

### OffscreenCanvas: the bench renders on the main thread
OffscreenCanvas: Chrome 69, Firefox 105, Safari 16.4 [V, BCD 8.1.5]. Three.js's
`WebGPURenderer` supports it; issue #31605, "OffscreenCanvas support broken in
r179", is closed since 8 August 2025 [V, three.js GitHub issue #31605].
Rendering in a worker loses `OrbitControls` and raycasting unless pointer events
are forwarded, makes HTML overlays awkward, and react-three-fiber in a worker is
unexplored ground [S]. The bench renders on the main thread (built); the
planned simulation workers leave it the headroom (FOUNDATIONS §7.6).

---

## 3. The monorepo, as built

```
piano-action-sim/
├─ pnpm-workspace.yaml        # packages: apps/*, crates/pa-wasm; the catalog below
├─ package.json               # packageManager pnpm@10.34.5; devEngines: Node ^26
├─ turbo.json
├─ Cargo.toml                 # members: pa-core, pa-app, pa-wasm, pa-cli; profiles (§1)
├─ rust-toolchain.toml        # channel 1.99.0; rustfmt, clippy; wasm32-unknown-unknown
├─ .nvmrc                     # 26
├─ biome.json
├─ .github/workflows/ci.yml   # the gates of scripts/verify.sh, in two jobs
├─ crates/
│  ├─ pa-core/                # the domain: no I/O, no web; libm its one dependency
│  ├─ pa-app/                 # the application: the single-key bench
│  ├─ pa-wasm/                # the only cdylib: wasm-bindgen, serde_json
│  │   └─ package.json        # "build": "wasm-pack build --target web --out-dir pkg"
│  └─ pa-cli/                 # the shell's adapter: `weigh`, `golden`
├─ apps/
│  └─ bench/                  # React, react-three-fiber, Vite, Vitest
├─ tools/smoke/               # drives the built page over the Chrome DevTools protocol
└─ scripts/                   # verify.sh: every gate; smoke.sh: the browser smoke test
```
What each crate does is FOUNDATIONS §7.1. An 88-note application comes with
stage 4 (planned, FOUNDATIONS §8).

`turbo.json`, where it matters, the wasm build's cache (built):
```jsonc
{
  "tasks": {
    "build": { "dependsOn": ["^build"], "outputs": ["dist/**"] },
    "@pas/wasm#build": {
      "inputs": [
        "$TURBO_ROOT$/crates/**",
        "!pkg/**",
        "!.turbo/**",
        "$TURBO_ROOT$/Cargo.toml",
        "$TURBO_ROOT$/Cargo.lock",
        "$TURBO_ROOT$/rust-toolchain.toml"
      ],
      "outputs": ["pkg/**"]
    }
  }
}
```
From Turborepo's documentation [V, Turborepo 2.11.7 documentation, Reference,
Configuration, `inputs` and `outputs`]: by default the inputs are all the files
of the package checked into source control; naming `inputs` opts out of that
default, which `$TURBO_DEFAULT$` restores; a glob starting with `$TURBO_ROOT$`
is relative to the repository's root; `package.json`, `turbo.json` and the
lockfiles are always inputs; an omitted or empty `outputs` caches nothing. The
wasm build's inputs therefore cover every crate under `crates/`, the workspace's
manifest and lockfile and the toolchain pin: a new crate dependency cannot leave
a stale cached wasm, and no crate besides `pa-wasm` needs a `package.json`.
Turborepo's `experimentalCargoWorkspaces` is not used: it maps `build` to
`cargo build --package=<crate> --locked`, which does not produce wasm-pack's
`pkg/`, and leaves library builds uncached by default [V, Turborepo 2.11.7
documentation, Guides, Rust].

The pnpm catalog (built). The three.js tarball ships no `.d.ts` and declares no
types dependency, so `@types/three` is versioned apart and can drift from the
runtime; react-three-fiber 9.8.1 requires React `>=19 <19.4` [V,
`@react-three/fiber@9.8.1` `package.json`, `peerDependencies`]. The catalog
moves them together:
```yaml
# pnpm-workspace.yaml
packages:
  - "apps/*"
  - "crates/pa-wasm"
catalog:
  three: 0.186.1
  "@types/three": 0.186.0
  "@react-three/fiber": 9.8.1
  "@react-three/drei": 10.7.9
  react: 19.3.0
  react-dom: 19.3.0
  "@types/react": 19.3.0
  "@types/react-dom": 19.3.0
  vite: 8.3.4
  vitest: 5.0.3
```
The catalog appears in `pnpm-lock.yaml` as `catalogs: default:`, each entry with
its `specifier` [V, local execution].

---

## 4. WebGPU cannot be the only path

The `api.GPU` entry of BCD 8.1.5 [V, `@mdn/browser-compat-data` 8.1.5,
`api.GPU`]:
- **Chrome and Edge 144+**: "Supported on ChromeOS, macOS, Windows, and
  **Linux (Intel Gen12+ GPUs only)**"; versions 113 to 143 carry a partial
  entry, without Linux.
- **Firefox 141**, partial: "Supports Windows since Firefox 141" · "Supports
  macOS Tahoe on Apple silicon since Firefox 145" · "Supports older macOS
  versions on Apple silicon since Firefox 147" · "Does not support macOS on
  Intel CPUs" (bug 2004105) · "**Does not support Linux**" (bug 2006676) · no
  service workers (bug 1942431).
- **Safari and iOS Safari 26+**: supported. **Firefox for Android: no.**

**Built**: the bench renders with the `WebGLRenderer` that react-three-fiber's
`<Canvas>` creates, through an orthographic camera in both views
(FOUNDATIONS §3.11). WebGL 2: Chrome 56, Firefox 51, Safari 15 [V, BCD 8.1.5,
`api.WebGL2RenderingContext`]. Three.js's `WebGPURenderer` ("by default, the
renderer tries to use a WebGPU backend if the browser supports WebGPU. If not,
WebGPURenderer falls back to a WebGL 2 backend") [V, three.js manual,
`WebGPURenderer`] is the way to WebGPU, should the bench come to need it; not
used.

Packaging: the `three@0.186.1` tarball exposes `exports` = `[".",
"./examples/fonts/*", "./examples/jsm/*", "./addons", "./addons/*", "./src/*",
"./webgpu", "./tsl"]`, builds `three.module.js`, `three.webgpu.js`,
`three.webgpu.nodes.js` and `three.tsl.js` among others, declares `sideEffects:
["./src/nodes/**/*"]`, and contains no `.d.ts` file; `@types/three@0.186.0`
provides `three.webgpu.d.ts` and `three.tsl.d.ts` [V, the installed packages'
`package.json` and files].

---

## 5. Geometry: the section is the outline

**Built.** Every outline is computed in Rust and crosses the boundary in
millimeters. The view extrudes each part's sagittal outline across its width
(`THREE.ExtrudeGeometry`) and turns the capstan's profile about its axis
(`THREE.LatheGeometry`); the section view looks at those solids straight on,
through an orthographic camera, so what it shows is the outline itself. The
drawing and the contacts read one definition per part (FOUNDATIONS §7.1).

**manifold-3d: not used.** `manifold-3d@3.5.4/manifold-encapsulated-types.d.ts`
(1472 lines) offers solids from cross-sections and cuts through them [V,
manifold-3d 3.5.4]:
```ts
class CrossSection {
  extrude(height, nDivisions?, twistDegrees?, scaleTop?, center?): Manifold;
  revolve(circularSegments?, revolveDegrees?): Manifold;
  offset(...): CrossSection;        // via Clipper2
  hull(): CrossSection;
  delete(): void;
}
class Manifold {
  static extrude(polygons, ...): Manifold;
  static revolve(polygons, ...): Manifold;
  slice(height: number): CrossSection;   // the section at a given height
  project(): CrossSection;               // the projected outline
}
setMinCircularAngle(angle), setMinCircularEdgeLength(len)  // tessellation control
```
Package exports: `.`, `./lib/*`, `./manifoldCAD`, `./manifold-gltf`. A section
cut from a solid that is itself an extrusion of the outline returns that
outline, so `slice()` adds nothing to the planar model's section. Its
WebAssembly bindings leak temporaries on `extrude`, `revolve`, `toPolygons` and
polygon inputs (issue #1855, open on 11 October 2026; the repro is a loop of
1000 `circle.extrude(1).delete()`) [V, manifold GitHub issue #1855]: any use
belongs in a disposal scope.

---

## 6. Units in names

A value in a trade unit carries its unit in its name; geometry and mechanics
are SI otherwise (`.claude/rules/conventions.md` > Units). The bench's names follow it:
`key_dip_mm`, `key_height_mm`, `capstan_height_mm`, `down_weight_g`,
`up_weight_g`, `front_weight_g`, `kf_g`. The later stages' settings take the
same form [S]: `let_off_mm`, `drop_mm`, `hammer_blow_distance_mm`,
`back_check_distance_mm`, `aftertouch_mm`, `jack_window_clearance_mm`. A change
of unit is then a rename; once saved files exist (planned, FOUNDATIONS §7.4), a
rename is a schema change and so a migration, and a saved file cannot be
misread silently.

---

## 7. Tests in the browser

**Built.** The smoke test drives the built bench in a headless Chrome over the
DevTools protocol, with no dependency (`tools/smoke/`, run by
`scripts/smoke.sh`), and fails on the first interaction that changes nothing; it
also runs the non-confusion tests through the whole chain (FOUNDATIONS §5.3,
§7.1). Vitest runs the TypeScript unit tests in Node. Both are gates of
`scripts/verify.sh` and of CI.

**The GPU trap** [S]: a screenshot comparison such as Playwright 1.64.0's
`toHaveScreenshot()` is not pixel-stable on a 3D viewport across drivers and
GPUs. Where a visual test is wanted:
1. screenshot the HTML layer and the panels, which are deterministic;
2. for the 3D view, a software rasterizer (`--use-gl=angle
   --use-angle=swiftshader`), one pinned container image, a generous
   `maxDiffPixelRatio`;
3. better, assert numbers, as the smoke test does: a regression then fails as a
   number, not as a blurred difference.

Playwright is not used. Vitest 5 moves the browser-mode providers into separate
packages (`@vitest/browser-playwright`, `@vitest/browser-webdriverio`,
`@vitest/browser-preview`), with peers `vite: ^6.4.0 || ^7.0.0 || ^8.0.0` and
`@types/node: ^22.0.0 || >=24.0.0` [V, `vitest@5.0.3` `package.json`,
`peerDependencies`].

---

## 8. Decision summary

| Axis | Built | Planned, not used, ruled out | Deciding fact |
|---|---|---|---|
| A. Monorepo | pnpm 10.34.5 workspace and Turborepo 2.11.7; the wasm crate a package through its own `package.json` (§3) | `experimentalCargoWorkspaces`: not used | library builds uncached under it [V, Turborepo 2.11.7 documentation, Guides, Rust] |
| B. Pipeline | wasm-pack 0.15.0, `--target web`; the default target features; release at opt-level 3, fat LTO, one codegen unit, `panic = "abort"` (§1) | `simd128`: permitted, not enabled. Relaxed SIMD: ruled out (FOUNDATIONS §4.4). vite-plugin-wasm and vite-plugin-top-level-await: not needed, `--target web` emitting an ES module with an explicit `init()` | Rust 1.99.0's defaults [V, local execution] |
| C. Execution | the bench runs its module on the main thread: one static key, no stepping loop | one module per Web Worker, notes independent; a `SharedArrayBuffer` ring where the page is cross-origin isolated, transferable buffers otherwise (planned, FOUNDATIONS §7.6). WebAssembly threads: ruled out (FOUNDATIONS §3.10) | `--shared-memory is disallowed by std…` [V, local execution, Rust 1.99.0] |
| D. Rendering | three.js 0.186.1, react-three-fiber 9.8.1, drei 10.7.9 (`OrbitControls`), the `WebGLRenderer` of `<Canvas>`, orthographic views, labels in HTML (FOUNDATIONS §3.11) | `WebGPURenderer`: not used. Babylon.js: ruled out (FOUNDATIONS §3.11). Linings drawn compressed, dimensions on the section, instancing for 88 notes, skins: planned (FOUNDATIONS §3.11) | WebGPU on Linux: Chrome only on Intel Gen12+ GPUs, Firefox not at all [V, BCD 8.1.5] |
| E. Geometry | outlines computed in Rust, extruded or turned by three.js; the section is the outline (§5) | manifold-3d: not used | slicing an extrusion of the outline returns the outline (§5) |
| F. Types | TypeScript types mirror the boundary's two JSON documents by hand (FOUNDATIONS §7.2) | types generated from the Rust ones: planned (FOUNDATIONS §7.2), the tool not chosen; ts-rs 12.0.1, tsify 0.5.8 and specta are the candidates observed. A versioned schema for saved files: planned with persistence (FOUNDATIONS §7.4); schemars 1.2.2 a candidate. json-schema-to-typescript, quicktype and ajv: not used | specta 2.0 still a release candidate (rc.25, 7 May 2026) [V, crates.io] |
| G. Interface | React 19.3, panels written by hand, their state held in React (FOUNDATIONS §2.4) | leva, zustand and uPlot: not used (zustand arrives only as a dependency of react-three-fiber and drei) | Observable Plot's latest release, 0.6.17, dates from 14 February 2025 [V, npm registry] |
| H. MIDI and sound | — | MIDI drives a force on the key (planned, FOUNDATIONS §7.6): a MIDI file parser, Web MIDI gated. Sound: an open question (FOUNDATIONS §9) | Safari 27: `requestMIDIAccess` unsupported [V, BCD 8.1.5] |
| I. Quality | Biome 2.5.15 (lint and format), rustfmt, Clippy with warnings as errors, `cargo test`, `tsc --noEmit`, Vitest and the browser smoke test, all in `scripts/verify.sh` and CI (FOUNDATIONS §7.1) | golden tests from literal coordinates (planned, FOUNDATIONS §7.4); native and WebAssembly results agreeing through the `libm` crate (planned, FOUNDATIONS §4.4). cargo-nextest, criterion, proptest, Playwright and ESLint: not used | typescript-eslint 8.71.1 requires `typescript <6.1.0` [V, npm registry] |
| Deployment | `apps/bench/public/_headers` sends COOP and COEP, in Cloudflare Pages' format | none yet: the bench is not deployed | Cloudflare Pages `_headers`: 100 rules, 2000 characters a line; GitHub Pages sets no custom headers [V, Cloudflare Pages and GitHub Pages documentation] |
