> Translated from the original French research annex. Part of the `piano-action-sim` foundations
> study (October 2026). See `docs/FOUNDATIONS.md` for the decisions this annex supports.

> **Legend** — **[V]** = verified against a first-hand source (npm/crates.io registry, `@mdn/browser-compat-data` 8.1.5, published JSON schema, official documentation, or local execution). **[S]** = assumed / architectural judgement.
> All versions below were recorded on **11 October 2026**.

---

## 0. Inventory of observed versions

### Local machine **[V]**
| Tool | Version | Note |
|---|---|---|
| Node | v26.3.1 | **non-LTS line**, and behind v26.11.1 |
| pnpm | 10.34.5 | `catalog:` works (tested) |
| rustc / cargo | 1.94.1 (e408947bf, 2026-03-25) | LLVM **21.1.8** |
| installed targets | `wasm32-unknown-unknown`, `x86_64-unknown-linux-gnu` | |
| wasm-pack / wasm-bindgen / wasm-opt / trunk | not installed by rustup | obtained separately, at the versions in the Rust table |

`rustup target list` offers `wasm32-wasip1`, `wasip1-threads`, `wasip2` — but **not `wasm32-wasip3`**, even though `rustc --print target-list` knows about it: no precompiled std, hence tier 3 / `build-std`. **[V]**

### JS/TS ecosystem **[V]**
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

### Rust ecosystem **[V]**
| Crate | Version | Updated | License / note |
|---|---|---|---|
| wasm-bindgen / js-sys / web-sys | **0.2.129** / 0.3.106 / 0.3.106 | 2026-09-25 | MSRV 1.81 |
| wasm-bindgen-cli | 0.2.129 | 2026-09-25 | |
| wasm-pack | **0.15.0** | 2026-05-15 | repo moved → `wasm-bindgen` org |
| wasm-bindgen-test | 0.3.79 | 2026-09-25 | |
| **wasm-bindgen-rayon** | 1.3.0 | **2024-12-21** | requires wasm-bindgen ^0.2.99 → compatible, but stagnant |
| binaryen (wasm-opt) | **version_133** | 2026-09-21 | the `wasm-opt` *crate* is stuck at 0.116.1 (2024-03-31) |
| wasm-tools | 1.261.0 | 2026-10-02 | |
| trunk | 0.21.14 | 2026-10-06 | |
| rapier2d / rapier3d | **0.36.1** | 2026-10-04 | Apache-2.0 |
| parry2d / nalgebra / glam / simba | 0.31.1 / 0.35.0 / 0.34.1 / 0.10.2 | — | |
| ts-rs | **12.0.1** | 2026-01-31 | stable, maintained |
| **specta** | stable **1.0.5** ; `2.0.0-rc.25` | rc.22 on 2025-01-12 → rc.25 on 2026-05-07 | `specta-typescript` = **0.0.12** |
| typeshare | 1.0.5 | 2026-01-02 | multi-language |
| schemars | **1.2.2** | 2026-07-27 | MIT |
| tsify | 0.5.8 | 2026-08-23 | wasm-bindgen ^0.2.104 |
| criterion / proptest / quickcheck / approx | 0.8.2 / 1.12.0 / 1.1.0 / 0.5.1 | — | |
| cargo-nextest | 0.9.148 | 2026-10-08 | |
| bevy / wgpu | 0.20.0 / 30.0.1 | 2026-10-08 / 2026-08-22 | MIT OR Apache-2.0 |
| dioxus / leptos | 0.7.10 / 0.8.22 | — | |

### Current browsers (BCD 8.1.5) **[V]**
Chrome **155** (2026-10-06) · Edge **154** · Firefox **157** (2026-09-29) · Safari / iOS Safari **27** (2026-09-14).

---

## 1. WASM features: what is actually usable

### rustc 1.94.1 defaults for `wasm32-unknown-unknown` **[V]** (output of `rustc --print cfg`)
```
bulk-memory, multivalue, mutable-globals, nontrapping-fptoint, reference-types, sign-ext
```
`simd128`, `atomics`, `exception-handling`, `tail-call`, `relaxed-simd`, `wide-arithmetic` are **available but not enabled by default**.

### Browser support (BCD 8.1.5) **[V]**
| Feature | Chrome | Firefox | Safari | iOS Safari | Verdict |
|---|---|---|---|---|---|
| fixed-width SIMD (**simd128**) | 91 | 89 | **16.4** | 16.4 | ✅ **enable** |
| bulk-memory | 75 | 78 | 15.1 | 15.1 | ✅ default |
| reference-types | 96 | 79 | 15.1 | 15.1 | ✅ default |
| multi-value | 86 | 78 | 13.1 | 13.4 | ✅ default |
| sign-ext / nontrapping-fptoint | 69 / 75 | 62 / 64 | 14.1 / 15.1 | — | ✅ default |
| threads-and-atomics | 74 | 79 | 15.2 | 15.2 | ⚠️ supported by browsers, **blocked on the Rust side** (see C) |
| extended-const | 114 | 112 | 17.4 | 17.4 | ✅ possible, not needed |
| tail-calls | 112 | 121 | 18.2 | 18.2 | ✅ possible, not needed |
| GC | 119 | 120 | 18.2 | 18.2 | ✅ not applicable (Rust makes no use of it) |
| exceptionsFinal | 137 | 131 | 18.4 | 18.4 | ❌ not needed with `panic=abort` |
| **relaxed-SIMD** | 114 | **146** | **preview** | **false** | ❌ **do not enable** |
| **memory64** | 133 | 134 | **preview** | **false** | ❌ do not enable (and not needed: < 4 GB) |
| multiMemory | 120 | 125 | preview | false | ❌ |
| branch-hinting | 137 | 148 | 16 | 16 | ⚪ marginal |
| JSPI | 137 | 153 | 27 | 27 | ⚪ not useful here |

### Recommended Cargo profile **[S]**
```toml
[profile.release]
opt-level = 3        # compute-bound: NOT "z" nor "s"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 1        # sim playable in dev, debug_assertions kept
```
And `[package.metadata.wasm-pack.profile.release] wasm-opt = ["-O3", "--enable-simd", "--enable-bulk-memory", "--enable-nontrapping-float-to-int", "--enable-reference-types", "--enable-multivalue"]`.
`--split-debug-info` (new in **wasm-bindgen 0.2.128**) extracts the DWARF into `*_bg.debug.wasm` with `--debug-info-url`: the shipped wasm stays small without losing debuggability. **[V]**
`wasm-split` does exist (Binaryen `src/tools/wasm-split/`) **[V]** but is useless for a single module — keep it in reserve.

---

## 2. Why WASM threads are blocked **[V]**

Test run locally on rustc 1.94.1 stable, `cdylib` crate with `std::thread::spawn` + `thread_local!`:

```
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals \
           -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 \
           -C link-arg=--import-memory"
cargo build --target wasm32-unknown-unknown --release
```
```
warning: unstable feature specified for `-Ctarget-feature`: `atomics`
error: linking with `rust-lld` failed: exit status: 1
  = note: rust-lld: error: --shared-memory is disallowed by
          std-cc68130c01d32363.std.fd93868c89d87b3f-cgu.0.rcgu.o
          because it was not compiled with 'atomics' or 'bulk-memory' features.
```

→ The std must be **recompiled** (`-Zbuild-std=panic_abort,std`), which is **Nightly** (the rustc book explicitly calls it "a Nightly Rust feature"). Combined with wasm-bindgen-rayon frozen since December 2024, **conclusion: single-threaded for the foundations**, with a domain written so that rayon can be added in one line.

### Web Worker vs AudioWorklet **[S]** (on the basis of **[V]** facts)
`AudioWorkletProcessor.process()` runs in quanta of 128 frames (≈ **2.67 ms** at 48 kHz) on a real-time thread — tempting for a 1–10 kHz integrator. To be rejected for the sim loop: any missed deadline becomes an *audible dropout*, allocation is forbidden there, and `AudioWorkletGlobalScope.port` only exists in Firefox 138+ **[V]**. Keep the AudioWorklet for audio, when audio arrives.

### Worker → renderer transport
One interface, two implementations **[S]**:
1. `crossOriginIsolated === true` → `SharedArrayBuffer` + triple buffer / seqlock (lock-free SPSC). SAB: Chrome 68 / FF 79 / Safari 15.2 **[V]**; `SharedArrayBuffer.growable`: 111 / 128 / 16.4 **[V]**; `Atomics.waitAsync`: Chrome 90 / Safari 16.4 / **Firefox 145** (very recent — must be gated) **[V]**.
2. Otherwise → `postMessage` with a transferable `ArrayBuffer` (zero copy).

**The decisive order of magnitude:** 88 notes × ~8 parts × 1 f32 angle ≈ **2.8 kB/frame**, i.e. **~170 kB/s at 60 Hz**. `postMessage` is plainly sufficient. COOP/COEP therefore becomes an *optimisation*, not a prerequisite.

### OffscreenCanvas **[V]** → **do not render in a worker in v1** **[S]**
Support: Chrome 69 / FF 105 / Safari 16.4 (Baseline). Three.js's `WebGPURenderer` handles it (issue #31605 "OffscreenCanvas support broken in r179" has been **closed** since 2025-08-08). But you lose OrbitControls and raycasting unless you rewrite pointer-event forwarding, DOM overlays become painful, and R3F-in-a-worker is a research topic. The main thread has headroom: the sim runs in a worker.

---

## 3. Monorepo — concrete tree

```
piano-action-sim/
├─ pnpm-workspace.yaml        # packages: apps/*, packages/*, crates/*  + catalog:
├─ package.json               # private, devEngines.packageManager: pnpm@10.34.5
├─ turbo.json
├─ Cargo.toml                 # [workspace] members = ["crates/*"]
├─ rust-toolchain.toml        # channel="1.94.1", targets=["wasm32-unknown-unknown"],
│                             # components=["rustfmt","clippy"]
├─ biome.json
├─ .nvmrc                     # 24  (LTS Krypton)
├─ schemas/                   # config.v1.json … generated AND committed
├─ .github/workflows/ci.yml
│
├─ crates/
│  ├─ action-core/            # PURE DOMAIN. no_std-friendly. 0 web dependencies.
│  │   └─ src/{geometry,leverage,contact,integrator,touchweight}.rs
│  ├─ action-config/          # regulation types + serde + schemars + ts-rs + migrations/
│  ├─ action-sim/             # 88-note + pedal orchestration, flat SoA state
│  ├─ action-wasm/            # THE ONLY cdylib crate. wasm-bindgen. Adapter.
│  │   ├─ Cargo.toml
│  │   └─ package.json        # ← pnpm wrapper: "build": "wasm-pack build --target web"
│  └─ xtask-schema/           # bin: generates schemas/*.json + .ts bindings
│
├─ packages/
│  ├─ config-types/           # generated .ts (ts-rs) + types derived from the JSON Schema
│  ├─ sim-client/             # Worker host, transport (SAB | postMessage), clock
│  ├─ geometry-gen/           # 2D profiles → THREE.Shape | manifold-3d
│  ├─ viewer-core/            # scene, ortho cameras, section planes, instancing, overlays
│  ├─ ui-kit/                 # regulation panel generated from the JSON Schema
│  └─ midi/                   # MIDI parsing + clock (adapter)
│
└─ apps/
   ├─ single-key/             # 1-key test bench
   └─ keyboard-88/            # full keyboard + pedals + dampers
```

`turbo.json` — the point that really matters, the wasm build cache:
```jsonc
{
  "tasks": {
    "build": { "dependsOn": ["^build"], "outputs": ["dist/**", "pkg/**"] },
    "action-wasm#build": {
      "dependsOn": ["^build"],
      "inputs": ["$TURBO_DEFAULT$", "src/**", "Cargo.toml",
                 "../../Cargo.lock", "../../rust-toolchain.toml"],
      "outputs": ["pkg/**"]
    }
  }
}
```
Reminders from the documentation **[V]**: `inputs` defaults to `[]` = "all the files in the package that are tracked by git"; **specifying `inputs` loses that default**, hence `$TURBO_DEFAULT$`; an empty or omitted `outputs` ⇒ **nothing is cached**; `package.json`, `turbo.json` and the lockfiles are **always** inputs. Since `inputs` paths are relative to the package's `package.json`, give `action-core` and `action-config` a thin `package.json` too and reproduce the Cargo edges as pnpm dependencies — **that is the accepted price** (3 files of duplication), and it is exactly what `experimentalCargoWorkspaces` will remove once its bugs are fixed.

pnpm catalog, tested locally **[V]**:
```yaml
# pnpm-workspace.yaml
packages: ["apps/*", "packages/*", "crates/*"]
catalog:
  three: 0.186.1
  "@types/three": 0.186.0
  "@react-three/fiber": 9.8.1
  "@react-three/drei": 10.7.9
  react: 19.3.0        # R3F 9.8.1 requires react ">=19 <19.4"
```
→ does indeed produce `catalogs: default: three: specifier: 0.186.1` in `pnpm-lock.yaml`.

---

## 4. Why WebGPU cannot be the only path **[V]**

The `api.GPU` entry of BCD 8.1.5, verbatim:
- **Chrome / Edge 144+**: "Supported on ChromeOS, macOS, Windows, and **Linux (Intel Gen12+ GPUs only)**" (113–143 were `partial_implementation`, removed in 144).
- **Firefox 141** (`partial_implementation: true`): "Supports Windows since Firefox 141" · "Supports macOS Tahoe on Apple silicon since Firefox 145" · "Does not support macOS on Intel CPUs (bug 2004105)" · "**Does not support Linux (bug 2006676)**" · no service workers (bug 1942431).
- **Safari / iOS Safari 26+**: supported. **firefox_android: false.**

You are developing on Linux → Three.js's unified `WebGPURenderer` ("by default, the renderer tries to use a WebGPU backend if the browser supports WebGPU. If not, WebGPURenderer falls back to a WebGL 2 backend") is the only reasonable abstraction. WebGL2: Chrome 56 / FF 51 / Safari 15 **[V]**.

Verified packaging detail: the `three@0.186.1` tarball exposes `exports` = `[".", "./examples/fonts/*", "./examples/jsm/*", "./addons", "./addons/*", "./src/*", "./webgpu", "./tsl"]`, builds `three.module.js` / `three.webgpu.js` / `three.webgpu.nodes.js` / `three.tsl.js`, `sideEffects: ["./src/nodes/**/*"]`, and **0 `.d.ts` files**. `@types/three@0.186.0` provides `three.webgpu.d.ts` and `three.tsl.d.ts`. **[V]**

---

## 5. Geometry — the API that settles the debate **[V]**

`manifold-3d@3.5.4/manifold-encapsulated-types.d.ts` (1472 lines) contains:
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
  slice(height: number): CrossSection;   // ← real section at a given Z level
  project(): CrossSection;               // ← projected outline
}
setMinCircularAngle(angle), setMinCircularEdgeLength(len)  // tessellation control
```
Package exports: `.`, `./lib/*`, `./manifoldCAD`, **`./manifold-gltf`**.

`slice()` / `project()` are exactly the tool for **orthographic technical section views**: the section displayed is computed from the same solid as the physics, not redrawn by hand. **[V]**

Verified pitfall: issue **#1855**, open — the WASM bindings leak temporaries on `extrude` / `revolve` / `toPolygons` / polygon inputs; the repro is a loop of 1000 `circle.extrude(1).delete()`. Generation happens once per parameter change, so it is manageable, but wrap it in a disposal scope.

---

## 6. The unit trap **[S]**

Name the fields with their unit, systematically:
`let_off_mm`, `drop_mm`, `key_dip_mm`, `hammer_blow_distance_mm`, `back_check_distance_mm`, `capstan_height_mm`, `repetition_spring_force_mN`, `touchweight_down_g`, `touchweight_up_g`, `key_lead_mass_g`, `jack_window_clearance_mm`, `aftertouch_mm`.
Mechanical consequence: a change of unit becomes a **rename**, hence a mandatory migration — it is impossible to silently break a saved preset.

---

## 7. Visual tests: the GPU trap **[S]**

Playwright 1.64.0's `toHaveScreenshot()` is **not** pixel-for-pixel stable on a 3D viewport across drivers/GPUs. Recipe:
1. Screenshot the **DOM/SVG layer** and the parameter panel — deterministic.
2. For the 3D: software rasteriser (`--use-gl=angle --use-angle=swiftshader`), a single pinned container image, generous `maxDiffPixelRatio`.
3. **Better: assert numbers.** Expose a debug hook that returns the projected 2D positions of named pivot points and assert on them. A kinematics regression then fails as a number, not as a blurry diff.

Vitest 5 — structural change **[V]**: the browser-mode providers are now separate packages (`@vitest/browser-playwright`, `@vitest/browser-webdriverio`, `@vitest/browser-preview`), peer `vite: ^6.4.0 || ^7.0.0 || ^8.0.0`, `@types/node: ^22 || >=24`.

---

## 8. Decision summary

| Axis | Decision | Deciding fact |
|---|---|---|
| A Monorepo | pnpm 10.34.5 + Turborepo 2.11.7, `package.json` wrappers | library builds **uncached** under `experimentalCargoWorkspaces` **[V]** |
| B Pipeline | wasm-pack 0.15.0 `--target web`, `+simd128` only | rustc 1.94.1 defaults recorded locally **[V]** |
| C Execution | 1 Worker, single-threaded, transport with 2 impls | `--shared-memory is disallowed by std…` **[V]** |
| D Rendering | Three.js 0.186.1 + R3F 9.8.1, `three/webgpu` | WebGPU on Linux = Intel Gen12+ / Firefox nothing **[V]** |
| E Geometry | parametric; `ExtrudeGeometry` → manifold-3d 3.5.4 | `Manifold.slice()` / `.project()` **[V]** |
| F Types | schemars 1.2.2 (config) + ts-rs 12.0.1 (DTO) | specta v2 in RC for 2 years **[V]** |
| G UI | React 19.3 + leva (scaffolding) + zustand 5.0.15 + uPlot 1.6.32 | Plot 0.6.17 frozen since 2025-02-14 **[V]** |
| H MIDI | MIDI file yes, Web MIDI gated, audio = empty port | Safari 27: `requestMIDIAccess` = **false** **[V]** |
| I Quality | Biome 2.5.15, nextest, criterion, bit-exact goldens via `libm` | ts-eslint 8.71.1 peer `typescript <6.1.0` **[V]** |
| Deployment | Cloudflare Pages `_headers` (100 rules, 2000 chars/line) | GH Pages: no custom headers **[V]** |
