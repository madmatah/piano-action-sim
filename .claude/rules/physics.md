---
paths:
  - "crates/**"
  - "apps/bench/src/**"
  - "tools/dynamics-prototype/**"
  - "Cargo.toml"
  - "**/package.json"
---

# Physics

The rules that keep the engine's physics truthful and its numbers the same on every machine. Each ends with its check, or says it is *not mechanically checked*.

## One part definition feeds contacts and drawing

Both are built in Rust from the same fields and exported, and the browser only renders them, applying each body's pose as given. The drawn parts declare the ranges a contact may read, and a contact pair holds those ranges and nothing else, so nothing a contact reads can drift from the drawing: the invariant holds by structure. The static solve places the key on those contacts, and every procedure of the sheet, the regulation solve and the drawing read the key placed there. Check: `every_contact_surface_is_a_range_of_a_drawn_part`, `every_contact_lies_on_its_drawn_outline`.

## Dry friction is set-valued

At the axes and at the sliding contacts alike, never `sign()` or `tanh()`. The interval between down weight and up weight *is* that friction: a regularized law has no friction at zero speed, so a static solve with it collapses down weight onto up weight, and its threshold speed, which has no physical meaning, enters every sensitivity. Axis friction is stored as a torque with its measurement radius, because the trade's published pivot frictions are taken at different radii. Check: `grep -rn tanh crates/*/src` finds only comments; `AxisFriction` carries its radius.

## No general-purpose physics engine in the core

The planar solver is the project's own, because the quantities the trade is made of (lever arms, gaps, set-valued friction at the axes) are wanted exact and analytic (`docs/FOUNDATIONS.md` > A dedicated planar solver, not a general-purpose engine). MuJoCo only ever serves as an optional oracle. Check: `cargo tree`, the `package.json`s.

## Determinism

IEEE-754 only, a fixed order of operations and contacts, no `HashMap` iteration in the solver, every transcendental in `pa-core` (`pow` included) through the `libm` crate, no relaxed SIMD (`docs/FOUNDATIONS.md` > Determinism, stated correctly). Convergence-tested loops are allowed with a hard cap and a reported count; a metrological path converges, by tolerance under a cap or by a bisection whose fixed count resolves below what is read; fixed budgets elsewhere only where real time needs them. No time step, solver count or felt law is fixed by rule: each is a hypothesis of `docs/FOUNDATIONS.md` > What the dynamics prototype settles, and what it leaves open, settled by its test.

Check: `grep -rn HashMap crates/pa-core/src` finds nothing; Clippy refuses in `pa-core` every `f64` method IEEE-754 does not round exactly (`crates/pa-core/clippy.toml`), a test allowing one explicitly; the golden sheet (`crates/pa-app/tests/golden.rs`) fails when a reading moves by a bit, and its WebAssembly twin (`crates/pa-wasm/tests/golden.test.mjs`, in `pnpm test`) when the browser's build reads another; a change that means to move one rewrites it with `cargo run -p pa-cli -- golden` and says why; `the_fingerprint_names_the_frozen_numbers` fails when a cap, tolerance or count changes and the solver's fingerprint does not.
