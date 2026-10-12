> Translated from the original French research annex. Part of the `piano-action-sim` foundations
> study (October 2026). See `docs/FOUNDATIONS.md` for the decisions this annex supports.

> Versions verified on October 11, 2026 via the registry.npmjs.org API, the crates.io API and the GitHub API; `.wasm` sizes measured by downloading and decompressing the npm tarballs; Rapier/Jolt/MuJoCo API surfaces measured by reading the published `.d.ts` files. The performance figures in section D are **my own measurements** on this machine (Intel Core Ultra 7 265K, node v26.3.1, rustc 1.94.1).
>
> Convention: **[V]** = verified with a source, **[S]** = assumed / inferred, **[?]** = not found, and recorded as such rather than guessed.

---

## 1. Answer to the central question

**NUANCED, with a clear split.**

**NO** for the fidelity core (sub-millimetre let-off + touch weight + back check), with a general-purpose game engine. Five verifiable technical reasons:

1. **Contact compliance there is a numerical parameter, not a physical one.** Rapier 0.36 exposes `IntegrationParameters::contact_softness: SpringCoefficients` whose defaults are `contact_defaults() = 30 Hz, damping_ratio 10` and `contact_static_defaults() = 60 Hz` **[V]**. 30 Hz = a relaxation constant of ~33 ms, i.e. *ten times longer than the entire let-off event* (the characteristic time of the action is "about a few milliseconds" according to Thorin et al. **[V]**). Box2D v3 has `contactHertz` / `contactDampingRatio` / `contactSpeed` **[V]** (same philosophy). Jolt does not even have that: `PhysicsSettings` contains `mBaumgarte`, `mPenetrationSlop`, `mSpeculativeContactDistance`, `mNumVelocitySteps`, `mNumPositionSteps` **[V]** — it is sequential impulses + Baumgarte stabilization, with no notion of contact stiffness.
2. **No injectable custom normal-force law.** The true felt law is `F(δ) = k·δ^r + b·δ²·δ̇` (Thorin, eq. 6) or `F = f_fit(δ)·(1 + D·δ̇)` with `f_fit(x) = ax³ + bx² + cx` (Hirschkorn, eqs. 2.4-2.5) **[V]**. In Rapier, the only entry point would be `PhysicsHooks::modify_solver_contacts` — **and it is not exposed in the JS binding**: `ActiveHooks` on the JS side contains only `NONE | FILTER_CONTACT_PAIRS | FILTER_INTERSECTION_PAIRS` (read in `@dimforge/rapier3d@0.21.0/pipeline/physics_hooks.d.ts`) **[V]**. Jolt exposes `ContactListenerJS` with `ContactSettings`, but that only contains `mCombinedFriction`, `mCombinedRestitution` and inverse-mass scale factors **[V]** — no force law.
3. **No dry friction at the pins.** Thorin models each hinge with `C_d(θ) = c_d·sign(θ̇)` (a *set-valued* function: `sign(0) ∈ [−1,1]`) + `C_v = c_v·θ̇` **[V]**. No game engine has the equivalent of `frictionloss`. The workaround — a joint motor with target velocity 0 and a bounded max force — does give a box constraint, but: (a) the bound is enforced by an iterative solver with a low iteration count (Rapier: `num_solver_iterations = 4` by default **[V]**), so the sticking is not exact; (b) in Rapier the motors of **multibody joints** are not accessible from JS (see §3).
4. **Game-grade geometric tolerances.** Rapier: `normalized_allowed_linear_error = 0.005` and `normalized_prediction_distance = 0.02`, multiplied by `length_unit` **[V]**. At metric scale that is 5 mm of tolerated penetration and 20 mm of predictive contacts — whereas let-off is regulated to 1–2 mm and drop is regulated to a tenth of a millimetre. You absolutely need `lengthUnit ≈ 0.01–0.05` to bring the tolerances back down, which is feasible (`set lengthUnit` is exposed in JS **[V]**) but reveals that you are working against the engine's design.
5. **Single precision.** The JS glue of `@dimforge/rapier3d@0.21.0` uses 108 occurrences of `Float32Array` versus 9 of `Float64Array` **[V]**: the engine is f32. The crates `rapier3d-f64` / `rapier2d-f64` 0.36.1 exist **[V]** but no f64 npm build is published. Conversely, the glue of `@mujoco/mujoco` uses `HEAPF64` (7 occ.) and `HEAPF32` (6 occ.) **[V]**: `mjtNum = double`.

**YES, with one exception that is not a game engine: MuJoCo.** See §4. MuJoCo is a general-purpose engine, but an engineering one, and it natively contains exactly the three missing primitives.

**And therefore: a dedicated solver.** But not for the reasons one might think. It is not a performance question (§7: the budget is comfortable even in JS). It is a question of **control over the formulation**: you want to be able to tell the technician "that stiffness is 42,000 N/m with an exponent of 2.7, measured on your felt" and not "I set 30 Hz".

### The decisive formulation point: mixed, and physical

One must **neither** regularize everything (the stiff-ODE approach) **nor** make everything non-smooth. The right decomposition is **mixed**, and it is physical:

| Direction | Physical nature | Formulation | Why |
|---|---|---|---|
| **Normal** (felt, leather, buckskin) | Felt **is** a non-linear spring | **Compliant**, explicit law `k·δ^r` or `aδ³+bδ²+cδ` | This is not a regularization: it is the correct model. ~40 N for 1 mm of compression measured by Hirschkorn **[V]** |
| **Tangential** (jack/knuckle, hammer/back check, jack/button) | Coulomb friction | **Set-valued** (cone / box, prox or LCP) | Sticking is a real state, not a limit |
| **Pins** (centres, bushings) | Dry + viscous friction | **Set-valued** (box \|λ\| ≤ c_d) + viscous term | This is what determines the downweight / upweight spread |

This is precisely what MuJoCo does (convex soft contacts + `frictionloss` as a box constraint), and precisely what no game engine does.

---

## 2. What the physics actually demands — reference figures

Drawn from the literature, all **[V]** (the Thorin 2017 PDF and the Hirschkorn thesis read in full):

**Thorin, Boutillon, Lozada, Merlhiot, "Non-smooth dynamics for an efficient simulation of the grand piano action", *Meccanica* (2017)**
- Model: **5 DOF** (key K, wippen W, jack J, repetition lever L, hammer H), **12 contacts**, tangential friction retained on **3 contacts only** (hammer/back check, jack/knuckle, jack/let-off button).
- The key in fact has **2 DOF** at the balance rail pin (rotation + vertical translation); they **blocked** the translation for experimental repeatability — note: in your case you will have to keep it (it is a regulation item: the play at the balance rail pin).
- Felt: rigid auxiliary body of **1 mg** + prismatic joint + non-linear Kelvin–Voigt element `F_felt(δ) = k·δ^r + b·δ²·δ̇`, plus a unilateral constraint and an **inelastic impact law** on the auxiliary body.
- Hinges: `C_d(θ) = c_d·sign(θ̇)` set-valued + `C_v = c_v·θ̇`. Springs: `C_s(θ) = j·(θ − θ₀)`.
- Scheme: **Moreau–Jean** (Jean 1999), θ = 1, γ = 1 (constraints satisfied at the end of the step), Newton linearization → *One-Step Non-Smooth Problem*, reformulated as an augmented Lagrangian, solved by **Gauss–Seidel**. Software: **XDE Physics** (CEA LIST, not public, available on request to the academic world).
- **Time step 0.5 ms (2 kHz)**, converged: "They do not significantly differ between 2 and 10 kHz". "This is a great advantage of the non-smooth methods chosen: there is no numerical parameter for impact or dry friction calling for a very small time-step."
- Performance: **≈ 20× slower than real time** in 3D on a 2010-era 2.1 GHz Core i5. Announced routes to reaching real time: a more efficient implementation of the non-linear springs and **"perform direct 2D dynamics rather than 3D"** → explicit validation of your planar-model choice.
- Comparison with the regularized approaches in the literature (their Table 1): Masoudi & Birkett, **910 s of CPU for ~200 ms simulated** = factor **≈ 4550**; 1720 s with a flexible shank. Van den Berghe: 1 s simulated = 4 h on a 486DX33. Izadbakhsh: 75 min on a 2.4 GHz PC. The authors' conclusion: non-smooth is **"about two orders of magnitude faster than the regularizing approach"**.
- **The sentence that settles it for a tool aimed at a technician**: "A basic adjustment of the key mechanism consists in measuring the maximum mass to be put on the key without initiating the down-motion […] and conversely, the minimum mass that prevents the up-motion […]. The result directly depends on the dry friction coefficient in the hinges. However, **this simple test cannot be simulated with a regularized friction law**."

**Hirschkorn, "Dynamic Model of a Piano Action Mechanism", thesis, U. Waterloo (2004)**
- 5 rigid bodies, **13 contact points**, normal and friction forces.
- Final contact model: `f_n = f_fit(x)·(1 + D·ẋ)` with `f_fit(x) = a·x³ + b·x² + c·x`, coefficients obtained by **least squares on the measured load curve of each contact** (one load test per contact, part mounted in a vice + motorized arm + encoder). The fit is performed on the **median line** between loading and unloading.
- Measured order of magnitude (felt under the front of the key): **≈ 40 N for 1 mm of compression** → secant stiffness ≈ 4·10⁴ N/m, tangent stiffness at 1 mm ≈ 1.2·10⁵ N/m.
- Acknowledged weakness: the model's hysteretic damping is velocity-dependent whereas real felt is almost not (Stamm); the author's recommendations: flexible bodies for key and hammer, and replacing the idealized bushings with flexible contact surfaces.

**Stability at the stiffnesses that actually occur.** The *physical* stiffnesses at play are on the order of **1·10⁴ to ~1·10⁶ N/m** (felts, leathers), not 10⁸. With `m_eff` from 0.01 to 0.05 kg, the contact natural frequencies fall to **~140 Hz (key side) to ~500 Hz–2 kHz (hammer side)**. A stable explicit scheme therefore requires dt ≲ 50–100 µs, i.e. **10–20 kHz** — not 1 MHz. That is what makes the computation budget comfortable (§7). The only genuinely stiff contacts are wood/metal (wippen stop, capstan on wippen if the leather is crushed), which must be handled either non-smoothly or with a capped stiffness.

**Timmermans, Bokiau, Fisette (UCLouvain / Robotran)** — *"Multibody-Based Piano Action: Validation of a Haptic Key"*, **Machines** 2020, 8(4), 76 **[V via secondary sources; MDPI PDF inaccessible, HTTP 403]**
- Haptic key driven by a **real-time** multibody model, loop **< 1 ms** (≈ 1 kHz), under ROS, custom linear actuator.
- The grand action model in MATLAB/Simulink: **1 s simulated in ≈ 1.15 s** — "still too slow" according to the team; the Viennese action (simpler) is already real time.
- Robotran = **symbolic** generation of the equations (the approach recommended in §7).

---

## 3. A — "Game" engines: versions, licences, binding status (Oct 11, 2026)

| npm package | Version | Date | Licence | `.wasm` raw / gzip | Verdict for OUR case |
|---|---|---|---|---|---|
| `@dimforge/rapier3d` | **0.21.0** | 2026-09-25 | Apache-2.0 | 3.08 MB / **1.17 MB** | No for the core. Useful for trapwork/visuals |
| `@dimforge/rapier2d` | **0.21.0** | 2026-09-25 | Apache-2.0 | 2.40 MB / **906 kB** | same, but 2D = good geometric fit |
| `@dimforge/rapier3d-compat` | 0.21.0 | 2026-09-25 | Apache-2.0 | wasm inlined, 4.34 MB bundle | A "no-bundler-config" variant, same engine |
| `@dimforge/rapier3d-simd` | 0.21.0 | 2026-09-25 | Apache-2.0 | — | SIMD128; **incompatible with enhanced-determinism in simd8** |
| `@dimforge/rapier3d-deterministic` | 0.21.0 | 2026-09-25 | Apache-2.0 | — | `enhanced-determinism` build published: a good sign |
| `jolt-physics` | **1.1.0** | 2026-07-11 | MIT | 2.02 MB / **742 kB** (mt: 2.48 MB / 712 kB) | No. No contact compliance whatsoever |
| `@babylonjs/havok` | **1.3.14** | 2026-07-29 | MIT (npm field) | 2.09 MB / **658 kB** | **Eliminated**: closed binary, no extensibility |
| `physx-js-webidl` | **2.8.0** | 2026-09-27 | MIT | 5.40 MB / **1.61 MB** | No. But the reduced-coordinate articulations are interesting |
| `cannon-es` | 0.20.0 | **2022-08-12** | MIT | — | **Dead**. No release in 4 years |
| `box2d-wasm` (Birch-san) | 7.0.0 | **2021-11-28** | Zlib | — | **Obsolete** (Box2D v2.4.1) |
| `box2d3-wasm` | **5.2.0** | 2026-02-16 | MIT | **[?]** | Box2D v3 + SIMD + Web Workers. Best "2D general-purpose" engine for prototyping |
| `planck` | **1.5.0** | 2026-04-07 | MIT | — (pure TS) | `planck-js` is **deprecated** → use `planck`. TS port of Box2D v2 |
| `ammojs-typed` | 1.1.0 | 2026-09-23 | MIT | — | Transpiled Bullet 2.8x. Legacy, avoid |

### Rapier — detailed analysis

**On the Rust side, Rapier 0.36.1 (Oct 10, 2026) has become surprisingly relevant** — and is converging toward MuJoCo **[V, CHANGELOG]**:
- **v0.34.0 (Jul 4, 2026)**: "passive **per-DoF springs** (MuJoCo-style spring/damper)", `MultibodyDofCoupling` (holonomic DOF coupling — exactly what is needed for a *capstan* or a damper lift tray), **per-DoF armature** (`Multibody::armature()`).
- **v0.36.0 (Sep 24, 2026)**: `Multibody::remove_dof_coupling` / `retain_dof_couplings` / `clear_dof_couplings`, + extended FEM soft bodies.
- **v0.36.1 (Oct 4, 2026)**: Python access to the reduced-coordinates state (`Multibody.generalized_position`, `set_armature`).
- `MultibodyJointSet` = **true reduced coordinates** ("A set of rigid bodies… Multibody: An articulated body simulated using the reduced-coordinates approach", constraint: "bodies must be linked following a tree structure") **[V, docs.rs]**. The changelog never names "Featherstone/ABA" explicitly **[?]**, but the description matches **[S]**.
- The `friction_model` is now selectable: `Simplified` (one Coulomb constraint per group of 4 points + one twist constraint) vs `Coulomb` (one per point); **"This setting doesn't apply to multibodies, which always use Coulomb"** **[V]**.
- Dimforge announces: "one of our main focus for Rapier in 2026 is the improvement of its robotics-oriented features", with the crates **`rapier3d-mjcf` 0.36.1** (an MJCF loader!) and **`rapier3d-urdf` 0.36.1** **[V]**.
- Determinism: locally deterministic by default; cross-platform with the `enhanced-determinism` feature and IEEE 754-2008 targets — **"which includes WASM targets"**; `parallel` + `enhanced-determinism` gives **bitwise** identical results independent of the thread count since 0.35.0-beta.0; incompatible with `simd8`; requires using the trigonometric functions of `nalgebra::RealField` rather than those of std for the initial values **[V]**.

**But the official JS binding cancels most of these advantages.** Measured by reading the `.d.ts` files of `@dimforge/rapier3d@0.21.0`:
- `IntegrationParameters` on the JS side exposes only: `dt`, `contact_erp` (getter), `lengthUnit`, `normalizedAllowedLinearError`, `normalizedPredictionDistance`, `numSolverIterations`, `numInternalPgsIterations`, `maxCcdSubsteps`, the `softBodies*`, **plus a single `set contact_natural_frequency` (setter with no getter)**. **Absent** are: `contact_damping_ratio`, `static_contact_softness`, `friction_model`, `warmstart_coefficient`, `normalized_max_corrective_velocity`, `contact_clustering`, `contact_recycling`. → you can stiffen the contacts, **but you cannot get out of damping_ratio = 10** (heavily overdamped) nor switch to `Coulomb` friction.
- `MultibodyJoint` on the JS side exposes only `isValid()`, `setContactsEnabled()`, `contactsEnabled()`. **No** motor, limit, per-DoF spring, armature, DoF coupling, nor any reading of the generalized coordinates. The 0.34–0.36 features are inaccessible.
- No `MODIFY_SOLVER_CONTACTS` in `ActiveHooks`.
- f32 only.

→ **Rapier conclusion**: if you want Rapier *seriously*, you have to **write your own Rust→WASM wrapper** (which is the right architecture anyway if you go the Rust route) rather than consume `@dimforge/rapier3d`. Even then, the contact model remains an Hz/ζ spring with no customizable force law → unsuited to the core, good for the trapwork.

### Jolt Physics — verdict

Sequential impulse solver + Baumgarte (`mBaumgarte`, `mNumVelocitySteps`, `mNumPositionSteps`, `mPenetrationSlop`, `mSpeculativeContactDistance`, `mManifoldTolerance`, `mUseManifoldReduction`) **[V, types.d.ts]**. `ContactListenerJS` makes it possible to hook the contacts from JS (better than Rapier on that point) **[V]**, but `ContactSettings` only lets you set combined friction/restitution and the inverse-mass factors → **no compliance**. `SpringSettings` exists on the constraints (joint limits), not on the contacts. Cross-platform determinism: build flag `JPH_CROSS_PLATFORM_DETERMINISTIC` / `-DCROSS_PLATFORM_DETERMINISTIC=ON`, documented in the JoltPhysics.js README, and the 5.1.0 release notes announce that the Emscripten build can be compiled cross-platform deterministic **[V]** — but **it is not known whether the published npm build enables it** **[?]**. Verdict: **no** for the core. A good engine if you only wanted game rigid bodies.

### Havok (via Babylon.js) — eliminated

`@babylonjs/havok` 1.3.14, 2.09 MB wasm, licence declared MIT on the npm side, **but a closed binary with no sources** **[V for the version, S for the closed nature — confirmed only by a three.js forum post]**. For a project whose goal is to *control* the contact law, an engine that cannot be modified and cannot be audited is disqualifying, regardless of its quality.

### PhysX 5 (physx-js-webidl) — verdict

2.8.0 (Sep 27, 2026), MIT, 5.4 MB wasm, wraps **PhysX 5.6.1**, covers articulations and all joint types **[V]**. `PxArticulationReducedCoordinate` has "zero joint error by design, and can handle larger mass ratios" **[V]** — attractive on paper for a lever chain. **But**: a TGS/PGS solver with hard contacts + slop, no contact force law, and a direct warning signal for our case — PhysX issue #484 reports that **revolute articulation joints with very small masses/inertias are not driven**, "regardless of the scene's solver (PGS/TGS)" **[V, user report]**. And the jack weighs a few grams and the repetition lever even less. Verdict: **no**, with a specific risk on small inertias.

### Box2D v3 (box2d3-wasm) — the best "general-purpose" engine for prototyping

`box2d3-wasm` 5.2.0 (Feb 16, 2026), MIT, "Box2D v3 for the Web, using WASM + SIMD + Web Workers" **[V]**. Box2D v3 (Erin Catto) uses a *soft-step* TGS solver with, in `b2WorldDef` **[V, upstream types.h]**:
```
float contactHertz;        // "Contact stiffness. Cycles per second."
float contactDampingRatio; // "Contact bounciness. Non-dimensional."
float contactSpeed;        // cap on overlap resolution speed (m/s)
float maximumLinearSpeed;
```
It is **2D**, which matches the planar nature of the action exactly, it is small, fast and determinism-friendly. But it has the same fundamental limitation: stiffness in Hz, no felt law, no pin friction. **Verdict: excellent for a kinematic mock-up in 2 days; impossible for touch weight.**

### cannon-es / ammo.js / planck — eliminated

`cannon-es`: last release 2022-08-12 **[V]** → abandoned. `ammojs-typed`: transpiled Bullet 2.8x, legacy solver. `planck` 1.5.0: TypeScript port of Box2D **v2** — interesting as a *readability reference* for a 2D solver in TS, useless as an engine here.

---

## 4. B — "Engineering / robotics" multibody engines

### MuJoCo — the serious candidate

**The central fact to retain**: official DeepMind WASM bindings are published on npm.

- `@mujoco/mujoco` **3.15.0**, published **Oct 5, 2026**, **Apache-2.0**, repo `google-deepmind/mujoco` directory `wasm`, npm maintainers including `yuvaltassa` **[V, npm registry]**. History: 3.6.0 (Mar 13, 2026) → 3.15.0 (Oct 5, 2026), i.e. ~monthly, aligned with the engine releases (3.15.0 of Oct 5, 2026 on the GitHub side) **[V, GitHub API]**.
- Package README: "These are the **canonical** JavaScript and TypeScript bindings for the MuJoCo physics engine […] developed and maintained by Google DeepMind and are always up to date"; **still marked WIP**; development mostly tested under Linux/Chrome, Windows "experimental" **[V]**.
- **Two builds**: single-thread at the root (`import loadMujoco from '@mujoco/mujoco'`) and **multi-thread** (`'@mujoco/mujoco/mt'`, Web Workers + SharedArrayBuffer, requires `Cross-Origin-Opener-Policy: same-origin` + `Cross-Origin-Embedder-Policy: require-corp`) **[V]**.
- Measured sizes: `mujoco.wasm` **10.37 MB raw / 2.54 MB gzip**; `mt/mujoco.wasm` 10.48 MB / 2.60 MB **[V, measured]**. That is 2.2× Rapier3d's gzip — acceptable for a workshop app, heavy for a mass-market demo.
- Build from source documented: **emsdk 4.0.10**, `emcmake cmake -B build && cmake --build build`, flag `-DMUJOCO_WASM_THREADS=ON` **[V]**.
- **Double precision** (`HEAPF64` in the glue) **[V, measured]**.

**Why MuJoCo ticks the boxes the game engines miss:**

1. **`frictionloss` = a true set-valued constraint.** The documentation is explicit: the dry friction of a joint or of a tendon is modelled "as a constraint, namely an upper limit on the absolute value of the force that friction can generate", and within the admissible set Ω: "**For friction loss we have the box constraint** |λ_F| ≤ η" **[V]**. That is *exactly* Thorin's set-valued `c_d·sign(θ̇)`, unregularized, solved inside the convex program. It is **the** feature that makes the downweight / upweight test simulable.
2. **Per-DoF polynomial springs and dampers** (added in **3.7.0, Apr 14, 2026**: `jnt_stiffnesspoly`, `tendon_stiffnesspoly`, `dof_dampingpoly`, `tendon_dampingpoly`) **[V, changelog]**. Documented forms **[V, XMLreference]**:
   - spring: `f(x) = −(a·x + b·x² + c·x³)`
   - damper: `f(v) = −(a·v + b·v·|v| + c·v³)` ("anti-symmetrization of the quadratic term")
   The spring is **structurally identical** to Hirschkorn's `f_fit(x) = a·x³ + b·x² + c·x`. By modelling each felt the Thorin way (small auxiliary body + prismatic joint + spring), you get the true felt law **natively**.
3. **Reduced coordinates**, per-DoF `armature`, `springref`, tendons (perfect for the damper lift tray and the trapwork), 8 types of equality constraint (`mjEQ_CONNECT/WELD/JOINT/TENDON/FLEX/FLEXVERT/FLEXSTRAIN/DISTANCE`) **[V, d.ts]**.
4. **Convex soft contacts** with position-dependent impedance: `solref = (timeconst, dampratio)`, `solimp = (d₀, d_w, width, midpoint, power)`; `margin`/`gap`; `condim` ∈ {1,3,4,6}; **pyramidal** or **elliptic** cones; **Newton** (default, Cholesky Hessian), CG (Hager-Zhang), PGS solvers; `noslip_iterations > 0` = a PGS post-processing pass on the friction dimensions with **R = 0 (hard constraints)** **[V]**.
5. **Integrators**: `Euler` (semi-implicit, implicit joint damping), **`implicitfast`** (recommended), `implicit`, `RK4`, and **`discrete`** (new in **3.13.0, Sep 8, 2026**: "the constraint solve and the implicit velocity update merge into one operation") **[V]**. `implicitfast` + implicit damping is valuable with stiff springs.
6. **Complete model-editing API from JS**: `parseXMLString(...) → MjSpec`, `mj_compile(MjSpec) → MjModel`, `mjs_addBody`, `mjs_addJoint`, `mjs_addGeom`, `mjs_addEquality`, `mjs_addActuator`, `mj_loadXML`, `mj_saveLastXML` **[V, d.ts]**. → for a tool with "every regulation parameter configurable", you can rebuild the model whenever the geometry changes and recompile, all inside the browser.

**MuJoCo's real limitations for our case — not to be underestimated:**

1. **Penetration is assumed by design**: "Contacts are soft, so during penetration both the normal force and velocity are positive"; the diagonal regularizer `R` cannot tend toward 0: "taking R → 0 would give a hard constraint, which MuJoCo does not allow" **[V]**.
2. **Contact stiffness is not an N/m**: it is `timeconst`, and it **"should be at least two times larger than the simulation time step", a constraint enforced internally unless `refsafe` = false** **[V]**. With dt = 10⁻⁴ s → τ ≥ 2·10⁻⁴ s. Order-of-magnitude estimate **[S, derivation]**: `k_eff ≈ m_eff/τ²` → for m_eff = 0.01 kg and τ = 2·10⁻⁴ s, k_eff ≈ 2.5·10⁵ N/m, i.e. **40 µm of compression under 10 N** — so sub-millimetre is reachable, but through a numerical parameter, not through the physics of the felt. The clean remedy is Thorin's: the true compliance goes into the **prismatic joint of the felt body** (polynomial spring), and the MuJoCo contacts then serve only for unilaterality → you are then back under the control of the model.
3. **No custom passive force law from JS**: the `.d.ts` files expose **only** `mj_resetCallbacks()`, no setter for `mjcb_passive` (no occurrence of `mjcb_` in the glue) **[V, measured]**. The plugin capability bits exist (`mjPLUGIN_PASSIVE`) **[V]**, but the realistic route is to **recompile your own wasm** with a plugin — feasible (documented build) but that is already "writing C++ and compiling it", so no longer "using an existing engine".
4. **Forms not natively expressible**: the Hunt–Crossley/Hirschkorn product `f(δ)·(1 + D·δ̇)` and Thorin's `b·δ²·δ̇` are **position-dependent** damping terms; the MuJoCo polynomial damper depends only on `v`. Approximation is possible (linearize D around the playing regime), exactness is not.
5. **Performance with many contacts**: single-thread native figures reported by users **[V, GitHub reports, hardware unspecified]** — Cassie 32 DOF ≈ **53,200 steps/s (18.8 µs/step)**; 27-DOF humanoid ≈ **29,700 steps/s (33.6 µs/step)**, real-time factor ~150×; 242-DOF scene / **390 contacts** ≈ **253 steps/s (3.96 ms/step)**. Extrapolation: an 88-note model = ~500–700 DOF and **~1100 contacts** potentially active → unusable as a single `mjModel` at 10 kHz **[S]**. The **islands** can be enabled (`mjDSBL_ISLAND`) and the 88 notes are disjoint subtrees, but collision detection and jacobian assembly remain global.
   → **Architectural workaround**: **one `mjModel`/`mjData` per note** (or per group of notes), step only the active notes, and treat the couplings as **prescribed scalar inputs**: damper (sustaining) pedal = position of the damper lift tray (one value), sostenuto = per-note latch state, una corda = lateral shift (rendering + strike position only, no out-of-plane DOF). This decomposition is exact and parallelizes 88 ways.

### The others, one by one

| Tool | Status October 2026 | WASM? | Verdict |
|---|---|---|---|
| **Siconos** (non-smooth dynamics, Moreau–Jean) | tag **`2026.rc1`** (Mar 11, 2026); docs still at 4.5.0; FreeBSD port 4.5.0_4 maintained (Jul 18, 2026) **[V]** | **No trace of a WASM build** **[?]** | It is *the* reference academic framework for exactly our formulation, and it is also the hardest to bring into a browser (BLAS/LAPACK, Boost, Python). **To be used as a theoretical reference / out-of-browser oracle**, not as a runtime |
| **Project Chrono** | **10.0.0** (Mar 27, 2026), BSD-3 **[V]** | **YES, official "Build Chrono for WASM" tutorial** with `emcmake ccmake -G Ninja`, Eigen ≥ 3.4.0, GLM optional for the WebGL visualization **[V]** | Has an **NSC** solver (complementarity, Anitescu–Tasora) *and* an **SMC** one (penalty). Huge (several MB+), C++ API with no JS binding. **Candidate for an offline cross-validation oracle**, not a web runtime |
| **Pinocchio** | **Community** WASM ports: `Mostafasaad1/pinocchio-js` (release Mar 1, 2026, RNEA/FK/CoM/Jacobian) and the `ucalgary-shivers` fork (adds **ABA**, CRBA; Pinocchio v2 + Eigen 3.4 via FetchContent; **binary ≈ 457 kB**) **[V]** | Yes, unofficial | Pinocchio does **not** do unilateral contact: it is articulated-tree dynamics. Useful as a **building block** (M, ∇, ABA) if you write the solver, useless as a complete engine |
| **Drake** | — | **No trace** **[?]** | Out of the running for the browser. (Its `MultibodyPlant` + SAP solver would nonetheless be conceptually ideal) |
| **Simbody / OpenSim** | — | **No trace of a WASM build**; LAPACK/BLAS dependencies **[V, absence of a source]** | Out of the running |
| **Robotran** (UCLouvain) | Used by the team that has *already built* a grand piano action + real-time haptic key **[V]** | No | **Not a runtime, but THE methodological reference**: **symbolic** generation of the equations. See §7 |
| **MBDyn** | — | **No trace** **[?]** | Out of the running |
| **Kane / symbolic (SymPy mechanics, Maple/DynaFlexPro)** | SymPy `sympy.physics.mechanics`: `KanesMethod`, `LagrangesMethod`, and code generation via `sympy.utilities.codegen` / `sympy.printing.c` / `sympy.printing.rust` **[S, stable knowledge]** | Via C/Rust → WASM compilation | **Strongly recommended** as a build step (§7). This is exactly the DynaFlexPro route (Hirschkorn, Izadbakhsh) and the Robotran one (UCLouvain) |

---

## 5. C — Rust building blocks for writing the dedicated solver (crates.io versions as of Oct 11, 2026)

| Crate | Version | Last update | Role / verdict |
|---|---|---|---|
| **nalgebra** | **0.35.0** | 2026-05-24 | **Selected.** Statically sized matrices (`Matrix6<f64>`, `SMatrix`) → zero allocation, `Cholesky` / `LU` on the stack. Provides `RealField`/`ComplexField` (portable trigonometric functions, a prerequisite for Rapier's cross-platform determinism) |
| nalgebra-sparse | 0.12.0 | 2026-05-24 | Useless at 6–8 DOF. Relevant only if you move to a monolithic 88-note model |
| ndarray | 0.17.2 | 2026-01-10 | Useless here (oriented toward dynamic n-D arrays) |
| faer | 0.24.4 | 2026-06-24 | Excellent beyond ~100×100. **Overkill** for 6×6; the call overhead would dominate |
| **ode_solvers** | 0.6.2 | 2026-06-07 | Dormand–Prince, RK, Radau. Useful for the **validation bench** outside the real-time loop; not for a fixed step |
| **diffsol** | **0.17.1** | 2026-09-28 | The most active of the Rust ODE solvers; BDF/SDIRK with a sparse jacobian. **Relevant for the reference oracle** (high-accuracy stiff integration); the Sundials/LLVM features complicate the WASM target **[?]** |
| peroxide | 0.43.1 | 2026-08-02 | General-purpose scientific toolbox; not necessary |
| **num-dual** | 0.15.1 | **2026-10-05** | Dual / hyper-dual numbers → **exact jacobians** of the contact laws and **sensitivities ∂(touch weight)/∂(regulation)**. Very relevant for a regulating tool: you can display "+0.1 mm of capstan ⇒ −1.4 g of downweight" analytically |
| enzyme (Rust autodiff) | Still **nightly / experimental** **[S]** | — | To be avoided in production; `num-dual` covers the need |
| **rapier2d / rapier3d** | 0.36.1 | **2026-10-10** | In a **secondary role** (§9). The multibody module is not a substitute for your solver (the contact model cannot be replaced) |
| rapier3d-f64 / rapier2d-f64 | 0.36.1 | 2026-10-10 | If you go through Rapier, **take the f64 variant** and compile it yourself |
| rapier3d-mjcf / rapier3d-urdf | 0.36.1 | 2026-10-10 | **MJCF as an interchange format** between your model, MuJoCo and Rapier: a genuine architectural opportunity |
| parry2d / parry3d | 0.31.1 | 2026-09-18 | Collision detection decoupled from Rapier. **Selected**: you can use Parry for the distances/normals (circle–segment, circle–circle: exactly the geometries of Thorin's Table 4) and write your own dynamics |
| **nphysics3d** | 0.24.0 | **2021-08-08** | **Dead**, "superseded by the rapier3d crate" **[V, crates.io description]**. To be ruled out |
| **k** | 0.32.0 | **2024-09-10** | Kinematics only (no dynamics). Stagnant. Of no interest here |
| **rbdl-rs** | **Not found on crates.io** **[V]** | — | To be ruled out |
| mujoco (Rust bindings) | **[?]** not verified | — | not checked; the official WASM bindings are what §4 relies on |
| **rayon** | 1.12.0 | 2026-04-14 | For the native/desktop version |
| **wasm-bindgen-rayon** | 1.3.0 | **2024-12-21** | Last published on that date, but dependencies `wasm-bindgen ^0.2.99` → **compatible with wasm-bindgen 0.2.129** (caret) **[V, crates.io API]**. Requires COOP/COEP. A more robust alternative: **one worker per group of notes with its own WASM module** and a `SharedArrayBuffer` for the shared state → no need for rayon at all |
| wasm-bindgen | 0.2.129 | 2026-09-25 | Reference |
| argmin | 0.11.0 | 2025-09-28 | Useful for **parameter identification** (fitting the felt load curves, as Hirschkorn did) |
| roots | 0.0.8 | **2022-12-21** | Stagnant; event detection (contact crossing) can be coded in 30 lines (Brent/bisection) |

**On WASM SIMD**: fixed SIMD128 is widely available (Chrome 91+, Firefox 89+, **Safari 16.4+**); **relaxed-SIMD is not supported by Safari** (listed "not supported" up to 26.5 and Technology Preview) **[V, caniuse]**. Do not count on relaxed-SIMD. And see §6: on a scalar 6×6 kernel SIMD brings **nothing** (measured). It would only help if you vectorize **across the notes** (4 or 8 notes in parallel in f64x2/f32x4 lanes) — a viable strategy but one to reserve for a later optimization.

---

## 6. D — Pure TypeScript vs Rust/WASM: real measurements

I wrote **the same time-step kernel** in JavaScript (Float64Array, monomorphic, zero allocation in the hot loop) and in Rust, then compiled Rust natively and to `wasm32-unknown-unknown`, and measured 20 million steps after 2 million warm-up steps.

**Kernel contents** (structurally representative of one note): 6 generalized coordinates; per-DOF passive forces (linear + viscous spring + **dry friction bounded by projection**); **13 contacts** with a gap test via jacobian dot product, cubic normal law by Horner `((c·d + b)·d + a)·d`, Hunt–Crossley factor `(1 + D·δ̇)`, bounded Coulomb tangential friction, projection onto the generalized forces; **6×6 Cholesky** + forward/back substitution; semi-implicit Euler with clamps.

### Measured results (Intel Core Ultra 7 265K, node v26.3.1 / V8, rustc 1.94.1, Oct 11, 2026)

| Implementation | ns / step | Msteps/s | 1 note @ 20 kHz | **88 notes @ 20 kHz** |
|---|---|---|---|---|
| **JavaScript** (Float64Array) | **157.5** | 6.35 | 0.32 % of one core | **28.5 % of one core** |
| **Rust → wasm32** (V8) | **108.4** | 9.22 | 0.22 % | **19.1 %** |
| Rust → wasm32 **+simd128** | 108.3 | 9.23 | 0.22 % | 19.1 % |
| **Native Rust** `-O3 -C target-cpu=native` | **92.4** | 10.83 | 0.19 % | **16.3 %** |

**Ratios: WASM / JS = 1.45×; native / WASM = 1.17×; native / JS = 1.70×.**

Three lessons, all counter-intuitive relative to the usual narrative:

1. **The expected JS→WASM performance ratio is ~1.4–1.5×, not 10×.** The "10–15×" figures one reads everywhere concern idiomatic JS code (objects, GC, polymorphism). On a numerical kernel written in a monomorphic typed-array style, V8 compiles code almost as good as LLVM. The academic literature is consistent: Jangda et al. (USENIX ATC'19) measure WASM as **slower** than native by 45 % (Firefox) to 55 % (Chrome) **[V]** — we are in the same range, seen from the other direction.
2. **`+simd128` brings nothing here** (108.3 vs 108.4 ns): LLVM does not vectorize a 6×6 Cholesky nor loops of 6 iterations with branches. Any useful vectorization will have to be explicit **across the notes**.
3. **The three implementations give a bit-for-bit identical result** after 22 million steps: checksum `q[0] = 0.000012260258495239365` in JS, in WASM and in native Rust (verified to 17 significant digits). Reason: the kernel only uses `+ − × ÷` and `sqrt`, **all exactly specified by IEEE-754**.

### But determinism breaks as soon as transcendentals are added

This is the decisive argument in favour of WASM, and it is not about performance:
- The ECMAScript spec allows **"implementation-approximated"** results for most `Math` functions; MDN confirms that the precision is "implementation-dependent" and that "the same JavaScript engine on a different OS or architecture can give different results" **[V]**. Recent public measurements show `Math.cos` rounding differently between V8/Node and V8/Chromium, and Chrome for Testing 153 diverging on `sin`, `cos`, `tan`, `exp`, `log`, `atan2` **[V, measurements from a third-party project, not independently replicated]**.
- In Rust→WASM, libm is **compiled into the module** → bit-identical on every browser, every platform, every version.

For a tool where a technician must be able to say "with these settings I get 52 g of downweight" and find **exactly** the same value tomorrow, on another machine, this is structural.

**Verdict D**: pure TypeScript is **viable** and would allow faster iteration at the start. But Rust/WASM wins on three axes that matter more than the 1.45× factor: (a) reproducible bit-exact determinism, (b) absence of GC pauses (critical if the physics loop runs in an AudioWorklet or at a fixed rate), (c) a type system suited to physics (units, newtypes, statically sized `Matrix6<f64>`, `#[inline]`, no JIT deoptimization surprises). And a real JS pitfall this benchmark exposes: an integer product such as `s * 1103515245` silently loses bits beyond 2⁵³, so a PRNG written that way diverges from the Rust one, and `Math.imul` is required to reproduce Rust's integer behaviour. This class of bug is invisible and endemic in numerical JS. This kind of bug is invisible and endemic in numerical JS.

---

## 7. Computation budget and recommended execution architecture

**Safety margin.** My kernel is simplified: it does not include the update of the contact geometry (rotations → `sin`/`cos`, circle–segment distances), event detection, or the prox/LCP iterations for the set-valued friction. A prudent realism factor: **×3 to ×4**. So, 88 notes @ 20 kHz:
- in Rust/WASM: 19.1 % × 3.5 ≈ **67 % of one core** — and with 4 Web Workers (22 notes each) ≈ **17 % per worker**.
- in JS: 28.5 % × 3.5 ≈ **100 % of one core** — feasible, with no margin.

To which is added a massive, free gain: **only a few notes are moving** at any given instant. A note at rest (all velocities zero, all contacts in a steady state) can be put to sleep at almost zero cost. The real budget when playing a piece is closer to **8–12 active notes** than to 88.

**Deployment.** Fixed-step physics loop in **Web Workers** (not in the main thread, not in the AudioWorklet) writing into a `SharedArrayBuffer` as a ring buffer; the render thread reads the latest state and interpolates for 60/120 Hz; if you add sound later, the AudioWorklet reads the same ring buffer. Caution: **all the `AudioWorkletNode`s of a single `AudioContext` share one real-time thread** **[V]** — the AudioWorklet is therefore not a place for parallelization. SharedArrayBuffer requires COOP/COEP (`Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Embedder-Policy: require-corp`) **[V]**; plan a single-worker fallback if `self.crossOriginIsolated === false`.

**Symbolic generation of the equations (strongly recommended).** This is the route taken by all the reference works: DynaFlexPro for Hirschkorn and Izadbakhsh, Robotran at UCLouvain **[V]**. Derive `M(q)`, the Coriolis terms and the contact jacobians with `sympy.physics.mechanics` (Kane or Lagrange), then generate Rust via `sympy.printing.rust` / C codegen, compiled to WASM. You get: zero hand-derivation errors, a dense 6×6 `M(q)` with no recursion loop, and exact analytical contact jacobians. For 6–8 DOF, the generated code is faster than a generic ABA.

---

## 8. Recommended three-layer architecture

```
┌─ DOMAIN (pure Rust, no_std-friendly, zero web dependency) ─────────────┐
│  • Parametric model: geometries, inertias, regulation dimensions       │
│  • M(q), Coriolis, contact jacobians  ← generated by SymPy             │
│  • Felt laws  F(δ) = k·δ^r + b·δ²·δ̇  (or a·δ³+b·δ²+c·δ)               │
│  • Set-valued friction: prox / box constraint (pins + tangential)      │
│  • Fixed-step integrator (Moreau–Jean or semi-implicit + prox)         │
│  • Sensitivities via num-dual: ∂(downweight)/∂(regulation)             │
│  → tested natively, 0 browser. Bit-exact.                              │
└────────────────────────────────────────────────────────────────────────┘
      ▲ ports (traits)                             │ adapters
┌─────┴──────────────┐  ┌────────────────────┐  ┌──┴────────────────────┐
│ wasm-bindgen       │  │ Native CLI         │  │ Export/Import MJCF    │
│ + Worker + SAB     │  │ (validation bench) │  │ (rapier3d-mjcf /      │
│                    │  │                    │  │  @mujoco/mujoco)      │
└────────────────────┘  └────────────────────┘  └───────────────────────┘
      ▲
┌─────┴─────────────────────────────────────────────────────────────────┐
│ RENDERING: three.js 0.186.1 (MIT, 2026-09-24) — app "1 key" and "88"  │
└───────────────────────────────────────────────────────────────────────┘
```

**Why MJCF as the pivot format**: it is the only format read by both MuJoCo (the reference) and Rapier (`rapier3d-mjcf` 0.36.1), and it can express your needs (joints, `frictionloss`, polynomial springs, tendons, equality, `solref`/`solimp`). You get a **three-way cross-validation** on the same geometry for free: your solver, MuJoCo WASM, Rapier.

**Suggested plan of attack (ordering, not a schedule)**
1. **Step 1 — MuJoCo mock-up.** Write an MJCF for one note (6–8 DOF, 12–13 contacts with auxiliary felt bodies + cubic springs + `frictionloss` at the pins), load it with `@mujoco/mujoco` 3.15.0, render with three.js. You validate the kinematics, the let-off and the order of magnitude of the touch weight in a few days, with zero lines of solver. **It is also your oracle for what follows.**
2. **Step 2 — dedicated solver.** Reproduce the same MJCF in your Rust domain, compare trajectories and forces against MuJoCo (and against the published curves of Thorin, figs. 8–12, and of Hirschkorn).
3. **Step 3 — 88 notes.** One `State` per note, worker pool, couplings (damper pedal / sostenuto / una corda) as prescribed scalar inputs.
4. **Step 4 — regulation & sensitivities.** `num-dual` to display live the derivative of the downweight with respect to each regulation dimension: that is the real differentiating product for a technician.

---

## 9. Using a general-purpose engine in secondary roles — yes, and which ones

1. **Cross-validation oracle: MuJoCo WASM, via MJCF.** The most useful role. Same geometry, two independent solvers, double precision on both sides. If your solver and MuJoCo diverge by more than a few % on the hammer velocity at let-off, the bug is on your side.
2. **Trapwork / pedals / lyre: Rapier (via your own Rust wrapper, f64) or Box2D v3.** The pedal trapwork is "coarse" mechanics (millimetres, not tenths of a millimetre), with no critical friction, with play and stops — exactly the domain where an impulse solver shines. And the damper lift tray is a rigid body with a spring and stops: no reason to put your precision solver there.
3. **Visual prototyping / kinematic exploration: Box2D v3 (`box2d3-wasm` 5.2.0) or `@dimforge/rapier2d`.** 2D = a natural fit for the sagittal plane. In one day you can check lever ratios, a wippen layout, a capstan height. Never measure a touch weight on that.
4. **What NOT to do**: do not try to force Rapier/Jolt onto the core by fiddling with `contact_natural_frequency`, `lengthUnit` and bounded motors. You will spend months fighting a contact model you cannot replace, for zero gain compared with writing 1,500 lines of dedicated solver in which you master every term.

---

## 10. Summary table: who can do what

| Need | Rapier (JS) | Rapier (Rust) | Jolt | Havok | PhysX | Box2D v3 | **MuJoCo** | **Dedicated solver** |
|---|---|---|---|---|---|---|---|---|
| Reduced coordinates | partial (bare API) | yes | no | no | yes | no | **yes** | **yes** |
| Double precision | **no (f32)** | yes (f64 crate) | opt. | ? | opt. | no | **yes** | **yes** |
| Custom normal force law | **no** | partial | **no** | **no** | **no** | **no** | partial (cubic poly) | **yes** |
| Per-DoF polynomial spring | **no** | yes (0.34+) | no | no | no | no | **yes (3.7.0+)** | **yes** |
| Set-valued dry friction at the pins | no | workaround possible | workaround possible | no | workaround possible | workaround possible | **yes (`frictionloss`)** | **yes** |
| Unregularized tangential friction | no | no | no | no | no | no | **yes (cone + noslip)** | **yes** |
| Geometric tolerance < 0.1 mm | needs rescaling | needs rescaling | slop | ? | slop | slop | **yes** | **yes** |
| Cross-platform determinism | **not verified** | yes (feature) | build flag | no | no | **[?]** | **[?]** | **yes (measured)** |
| Hot model editing from JS | partial | — | partial | partial | partial | partial | **yes (`MjSpec` + `mj_compile`)** | **yes** |
| Binary weight (gzip) | 1.17 MB | up to you | 742 kB | 658 kB | 1.61 MB | **[?]** | 2.54 MB | **~50–200 kB [S]** |
| Licence | Apache-2.0 | Apache-2.0 | MIT | MIT (closed binary) | MIT | MIT | **Apache-2.0** | yours |
