# A1. Physics engines, and the case for a dedicated solver

Research annex to `docs/FOUNDATIONS.md`, which holds wherever the two disagree.
It surveys the general-purpose physics engines and the Rust building blocks
available to a browser simulator of the grand piano action, and sets out what
the action's physics asks of a solver.

Software facts (versions, dates, sizes, API surfaces, defaults) are dated
observations: read on 11 October 2026 from the npm registry, crates.io, the
GitHub API, the published packages and their documentation, each with its
locator. Facts from the literature are read in the corpus (FOUNDATIONS §10),
each with its section, table, equation or page.

Markers: [V] verified against the primary source named beside it; [E, n]
computed by script n of `tools/derivations/`; [S] assumed; [?] not
established, with the reason beside it. A sentence beginning **"Do not use"**
marks a figure or an argument that is wrong and easy to derive again.

---

## 1. Answer to the central question

**No general-purpose engine in the core; MuJoCo at most as an optional oracle;
a dedicated planar solver.** FOUNDATIONS §3.9 states the grounds: per-contact
nonlinear felt laws with memory; exact analytic lever arms at sub-millimetre
scale, in double precision; and contact softness and margins that game engines
make global and millimetric. The evidence, engine by engine:

1. **Contact compliance is a numerical parameter, scene-wide, not a law per
   contact.** Rapier 0.36 exposes `IntegrationParameters::contact_softness`,
   whose default `SpringCoefficients::contact_defaults()` is a 30 Hz spring with
   damping ratio 10, and `contact_static_defaults()` a 60 Hz one [V, rapier
   0.36.1, `src/dynamics/integration_parameters.rs`]. A 30 Hz spring has a
   period of 33 ms, ten times the "few milliseconds" Thorin gives as the
   action's characteristic time [V, Thorin 2017 §5]. Box2D v3 has the same kind
   of setting, `contactHertz`, `contactDampingRatio` and `contactSpeed` in
   `b2WorldDef` [V, Box2D v3, `include/box2d/types.h`]. Jolt has none:
   `PhysicsSettings` holds `mBaumgarte`, `mPenetrationSlop`,
   `mSpeculativeContactDistance`, `mNumVelocitySteps` and `mNumPositionSteps`
   [V, jolt-physics 1.1.0, `dist/types.d.ts`]; it is sequential impulses with
   Baumgarte stabilization and no contact stiffness.
2. **No force law can be injected per contact.** The published felt laws are
   `F(δ) = k·δ^r + b·δ²·δ̇` [V, Thorin 2017 §2.4, Eq. 6] and
   `f_n = f_fit(x)·(1 + D·ẋ)` with `f_fit(x) = a·x³ + b·x² + c·x` [V, Hirschkorn
   2004 §2.2.2, Eqs 2.4–2.5]; which law the simulator uses is a hypothesis
   (FOUNDATIONS §4.3). In Rapier the only entry point would be
   `PhysicsHooks::modify_solver_contacts`, and the JS binding does not expose
   it: `ActiveHooks` holds only `NONE | FILTER_CONTACT_PAIRS |
   FILTER_INTERSECTION_PAIRS` [V, `@dimforge/rapier3d@0.21.0`,
   `pipeline/physics_hooks.d.ts`]. Jolt exposes `ContactListenerJS`, but its
   `ContactSettings` carries only the combined friction and restitution, the
   inverse mass and inertia scales, a sensor flag and surface velocities [V,
   jolt-physics 1.1.0, `dist/types.d.ts`]: no force law.
3. **Tolerances are global and millimetric.** Rapier's
   `normalized_allowed_linear_error = 0.005` and
   `normalized_prediction_distance = 0.02` are multiplied by `length_unit` [V,
   rapier 0.36.1, `src/dynamics/integration_parameters.rs`]: at metric scale,
   5 mm of tolerated penetration and contacts predicted 20 mm out, where let-off
   is regulated to a millimetre or two and drop to a tenth. A `lengthUnit` of
   0.01 to 0.05, which the JS binding can set [V, `@dimforge/rapier3d@0.21.0`
   `.d.ts`], brings both down together; neither becomes a law per contact.
4. **Single precision in the browser, and no analytic lever arms.** The JS glue
   of `@dimforge/rapier3d@0.21.0` uses `Float32Array` 108 times against 9 for
   `Float64Array` [V, `@dimforge/rapier3d@0.21.0`, JS glue]: the published build
   is f32. The crates `rapier3d-f64` and `rapier2d-f64` 0.36.1 exist [V,
   crates.io], but no f64 npm build is published. And a collision pipeline
   returns contact points with margins, where the lever arms every ratio of the
   trade is made of are wanted exact and analytic (FOUNDATIONS §3.9, §6.2).

**Do not use "no game engine has dry friction at a joint".** Jolt's hinge
constraint carries a maximum friction torque,
`HingeConstraintSettings.mMaxFrictionTorque` and
`HingeConstraint.SetMaxFrictionTorque` [V, jolt-physics 1.1.0,
`dist/types.d.ts`, lines 1486–1502], and its slider, swing-twist, six-degree-of-
freedom and path constraints carry the same bound. Box2D documents joint
friction as a motor held at zero speed with a bounded torque, the box constraint
of MuJoCo's friction loss, and joint friction does not distinguish a dedicated
solver (FOUNDATIONS §3.9). What a bounded motor inside an iterative solver does
not promise is an exact stick: Rapier's default is four solver iterations [V,
rapier 0.36.1, `src/dynamics/integration_parameters.rs`], and in Rapier the
motors of multibody joints are not reachable from JS (§3).

**MuJoCo, an engineering engine, is the one worth keeping**: joint friction
loss, polynomial springs, reduced coordinates, double precision and an official
WebAssembly build (§4). Its friction loss is a soft constraint, so a joint
creeps under a sub-threshold load: an optional oracle for cross-checks, not a
dependency (FOUNDATIONS §3.9).

**The dedicated solver is chosen for control over the formulation, not for
speed.** A technician is to be told which law a felt follows, with coefficients
measured on it, not that the contacts are a 30 Hz spring. Rust compiled to
WebAssembly is chosen for reproducible arithmetic, no garbage collector in a
fixed-step loop, and native runs of the code that ships (FOUNDATIONS §3.10);
whether 88 notes fit in real time is a hypothesis with its test (FOUNDATIONS
§4.6).

### The formulation: compliant along the normal, set-valued in friction

Neither everything regularized (stiff ordinary differential equations) nor
everything non-smooth: each direction gets the formulation its physics has. This
is the hypothesis of FOUNDATIONS §4.2, which the dynamics prototype settles.

| Direction | Physical nature | Formulation | Why |
|---|---|---|---|
| **Normal** (felt, cloth, leather) | felt compresses, with hysteresis and a memory of the loads it has seen (FOUNDATIONS §4.3) | **compliant**, a law per contact, treated implicitly | the compliance is the physics, not a regularization of a rigid contact; which law is FOUNDATIONS §4.3's hypothesis. Under the key front, Hirschkorn's loading curve reaches about 35 N at 0.86 mm [V, Hirschkorn 2004 §2.2.3, Figure 2.6, p. 22] |
| **Tangential** (sliding contacts) | Coulomb friction | **set-valued** (cone or box) | sticking is a real state, not a limit. Thorin keeps it at three contacts: hammer and back check, jack and knuckle (his "roller"), jack and let-off button [V, Thorin 2017 §2.4, Eq. 7, Fig. 2]; Hirschkorn's law covers the capstan and heel, the back check, the jack and knuckle, the lever and knuckle (FOUNDATIONS §6.5) |
| **Axes** (centre pins in their bushings) | dry and viscous friction | **set-valued**, \|λ\| ≤ c_d, plus a viscous term | the interval between down weight and up weight is this friction (FOUNDATIONS §3.5) |

MuJoCo comes close to this decomposition, with every constraint soft, friction
loss included (§4); Jolt's hinge friction and Box2D's bounded motor give the
axis term too. The decomposition rules out no engine by itself; the three
grounds of FOUNDATIONS §3.9 do.

---

## 2. What the physics demands: reference figures

### Thorin, Boutillon, Lozada, Merlhiot, "Non-smooth dynamics for an efficient simulation of the grand piano action", *Meccanica* 52, 2017

- **Model**: five degrees of freedom (key K, wippen W, jack J, repetition lever
  L, hammer H) and twelve contacts [V, Thorin 2017 §2.2, §2.4]; friction in the
  tangential direction at three contacts only, hammer and back check, jack and
  knuckle (his "roller"), jack and let-off button [V, Thorin 2017 §2.4, Eq. 7,
  Fig. 2].
- **The key has two degrees of freedom at its balance pin**: a rotation, and a
  vertical translation limited by the felt between the frame and the key. Thorin
  blocks the translation, replacing that felt with a metal ring of the same
  thickness, for repeatable measurements [V, Thorin 2017 §2.2]. The simulator
  keeps it: the key's vertical degree of freedom and a compliant rocking contact
  on the balance punching are planned (FOUNDATIONS §2.3).
- **Felt**: an auxiliary rigid body of 1 mg, a prismatic joint and a nonlinear
  Kelvin–Voigt element `F_felt(δ) = k·δ^r + b·δ²·δ̇`, with a unilateral
  constraint and an inelastic impact law on the auxiliary body [V, Thorin 2017
  §2.4, Eq. 6, Fig. 3].
- **Axes**: `C_d(θ) = c_d·sign(θ̇)`, `sign` set-valued at zero speed, plus
  `C_v = c_v·θ̇`; springs `C_s(θ) = j·(θ − θ₀)` [V, Thorin 2017 §2.3, Eqs 2–5].
- **Scheme**: Moreau–Jean after Jean, θ = 1 and γ = 1, constraints satisfied at
  the end of each step; a Newton loop yields a one-step non-smooth problem,
  reformulated with an augmented Lagrangian and solved by a Gauss–Seidel-like
  algorithm. Software: XDE Physics, CEA LIST, available on request to members of
  the academic community for research [V, Thorin 2017 §4].
- **Step 0.5 ms (2 kHz)**: the computed forces "do not significantly differ
  between 2 and 10 kHz" [V, Thorin 2017 §5, Fig. 12]. Thorin adds: "This is a
  great advantage of the non-smooth methods chosen: there is no numerical
  parameter for impact or dry friction calling for a very small time-step" [V,
  Thorin 2017 §5]. **Do not use the time step as an argument against regularized
  friction**: the tiny step holds only for explicit integration; treated
  implicitly, a velocity-only friction law sets no step limit (FOUNDATIONS §3.5)
  [E, 07].
- **Cost**: about twenty times slower than real time, in 3D, on a 2010 computer
  with a 2.1 GHz Intel Core i5. The authors' routes to real time are a more
  efficient implementation of the nonlinear spring laws and to "perform direct
  2D dynamics rather that [sic] 3D" [V, Thorin 2017 §6]; the planar model of
  FOUNDATIONS §1.4 takes the second.
- **Published run times**: Masoudi and Birkett, 910 s of CPU, 1720 s with a
  flexible shank, for a simulated duration Thorin estimates at 200 ms from their
  figure, a factor of 4550; Van den Berghe et al., one second simulated in about
  four hours on a 486DX33; Izadbakhsh, about 75 min on a 2.4 GHz PC [V, Thorin
  2017 §1, Table 1]. Thorin concludes that the non-smooth formalism allows
  simulations "about two orders of magnitude faster than the regularizing
  approach, as reported in the literature" [V, Thorin 2017 §6]. The table
  compares run times across machines, models and software; it does not isolate
  the friction law.
- **The weighing test**: "A basic adjustment of the key mechanism consists in
  measuring the maximum mass to be put on the key without initiating the
  down-motion from the resting position and conversely, the minimum mass that
  prevents the up-motion from the down-position. The result directly depends on
  the dry friction coefficient in the hinges. However, this simple test cannot
  be simulated with a regularized friction law" [V, Thorin 2017 §2.5]. Read
  exactly: a regularized law has zero friction at zero speed, so a static solve
  with it collapses D onto U; integrated dynamically with a threshold speed of
  10⁻⁶ rad/s it keeps the threshold, creeping under a micrometre in ten seconds
  under a gram of excess, but that speed has no physical meaning and enters
  every sensitivity (FOUNDATIONS §3.5) [E, 07].

### Hirschkorn, *Dynamic Model of a Piano Action Mechanism*, MASc thesis, University of Waterloo, 2004

- Five rigid bodies and thirteen contacts, with normal and friction forces [V,
  Hirschkorn 2004, Appendix A, Tables A.1 and A.8]. The equations are generated
  in symbolic form by DynaFlexPro, in Maple [V, Hirschkorn 2004 §2.1].
- **Contact law**: `f_n = f_fit(x)·(1 + D·ẋ)`, with
  `f_fit(x) = a·x³ + b·x² + c·x` [V, Hirschkorn 2004 §2.2.2, Eqs 2.4–2.5]. The
  cubic is a least-squares fit to the mid-line between the loading and unloading
  curves measured on each contact, one component held in a vice, the other on a
  motor-driven arm whose rotation an encoder reads [V, Hirschkorn 2004 §2.2.3].
  Under the key front the measured curve reaches about 35 N at 0.86 mm, a secant
  stiffness of about 4·10⁴ N/m (35 N / 0.86 mm) [V, Hirschkorn 2004 §2.2.3,
  Figure 2.6, p. 22]. Each of the thirteen contacts carries such a law, the
  capstan and the stops included [V, Hirschkorn 2004, Table A.8].
- **Weaknesses he names**: his law's damping depends on speed where felt's does
  not, after Stamm [V, Hirschkorn 2004 §2.2.2]; his friction parameters are less
  reliable than his dimensions and masses [V, Hirschkorn 2004 §5.2]. **His
  recommendations**: flexible bodies, the key and the hammer first; a contact
  law for felt's independence from speed, with other models of hysteresis "such
  as the Preisach Model"; and the felt bushings, the key on its balance rail
  among them, as compliant contacts instead of perfect revolute joints [V,
  Hirschkorn 2004, Abstract, p. iii; §5.2].

### The time step is a hypothesis

The step is chosen by the step-convergence study, not derived from a stability
limit (FOUNDATIONS §4.1). Its starting estimates come from Hirschkorn's action
assembled with every frame's origin at its own pivot. Along the contact normals,
the effective masses at rest run from 0.8 g (the jack in the lever's window) to
113 g (the back check), 8.9 g at the jack and knuckle [E, 10]; the stiffnesses
of his thirteen laws start between 2.4·10³ and 3.2·10⁴ N/m and rise with
penetration, at some contacts after a dip of up to about a quarter [E, 05]; and
the highest linearised mode at rest lies near 1.2 kHz with the capstan, the
front punching and the jack and knuckle loaded at assumed fortissimo forces,
near 2.2 to 2.3 kHz with the lever and the drop screw loaded too [E, 10].
Thorin's compliant model gives very similar results at 2 and 10 kHz and
significant discrepancies at 0.4 kHz [V, Thorin 2013 §6.4.1].

**Do not use a step of 50 to 100 µs derived from effective masses of 10 to
50 g**: at the contacts of the jack and the repetition lever the effective
masses along the contact normals fall to a few grams and below, 0.8 g in the
lever's window [E, 10]. **Nor 12 µs, nor 2/ω as the stability limit**
(FOUNDATIONS §4.1).

### Timmermans, Dehez and Fisette, "Multibody-Based Piano Action: Validation of a Haptic Key", *Machines* 8(4), 76, 2020

Not in the corpus (FOUNDATIONS §10); what follows comes from secondary sources
and is unverified [?]: a haptic key driven in real time by a multibody model,
its loop under 1 ms, under ROS, with a custom linear actuator; a grand action
model in MATLAB/Simulink simulating 1 s in about 1.15 s, which the team calls
still too slow, where a simpler Viennese action already runs in real time.
Fisette's group also modelled a grand action with six degrees of freedom in
Robotran, a generator of multibody equations in symbolic form (§7): Bokiau,
Poncelet, Fisette and Docquier, 2012 [V, Thorin 2017, Table 2 and reference 4].

---

## 3. A — "Game" engines: versions, licences, bindings (observed 11 October 2026)

| npm package | Version | Date | Licence | `.wasm` raw / gzip | Verdict for this project |
|---|---|---|---|---|---|
| `@dimforge/rapier3d` | **0.21.0** | 2026-09-25 | Apache-2.0 | 3.08 MB / **1.17 MB** | Not for the core; not used (§9) |
| `@dimforge/rapier2d` | **0.21.0** | 2026-09-25 | Apache-2.0 | 2.40 MB / **906 kB** | Same; 2D fits the sagittal plane |
| `@dimforge/rapier3d-compat` | 0.21.0 | 2026-09-25 | Apache-2.0 | wasm inlined, 4.34 MB bundle | A variant needing no bundler configuration, same engine |
| `@dimforge/rapier3d-simd` | 0.21.0 | 2026-09-25 | Apache-2.0 | — | SIMD128; incompatible with enhanced determinism |
| `@dimforge/rapier3d-deterministic` | 0.21.0 | 2026-09-25 | Apache-2.0 | — | An `enhanced-determinism` build, published |
| `jolt-physics` | **1.1.0** | 2026-07-11 | MIT | 2.02 MB / **742 kB** (mt: 2.48 MB / 712 kB) | Not for the core: no contact law; its hinges carry a friction torque |
| `@babylonjs/havok` | **1.3.14** | 2026-07-29 | MIT (npm field) | 2.09 MB / **658 kB** | **Eliminated**: no contact law, no published source |
| `physx-js-webidl` | **2.8.0** | 2026-09-27 | MIT | 5.40 MB / **1.61 MB** | Not for the core; its reduced-coordinate articulations are of interest |
| `cannon-es` | 0.20.0 | **2022-08-12** | MIT | — | **Abandoned**: no release in four years |
| `box2d-wasm` (Birch-san) | 7.0.0 | **2021-11-28** | Zlib | — | **Obsolete** (Box2D v2.4.1) |
| `box2d3-wasm` | **5.2.0** | 2026-02-16 | MIT | [?] not measured | Box2D v3 with SIMD and Web Workers; the closest general-purpose fit to a planar mechanism |
| `planck` | **1.5.0** | 2026-04-07 | MIT | — (pure TS) | `planck-js` is **deprecated** in favour of `planck`; a TypeScript port of Box2D v2 |
| `ammojs-typed` | 1.1.0 | 2026-09-23 | MIT | — | Bullet 2.8x transpiled; legacy |

### Rapier

**On the Rust side, Rapier 0.36.1 moves towards MuJoCo's feature set** [V,
Rapier CHANGELOG, 0.34.0 to 0.36.1]:
- **0.34.0 (4 July 2026)**: passive per-DoF springs ("MuJoCo-style
  spring/damper"), `MultibodyDofCoupling` (a holonomic coupling of degrees of
  freedom), per-DoF armature (`Multibody::armature()`). A holonomic coupling
  cannot stand for the capstan or the damper lift tray: both are unilateral
  contacts that lift off (FOUNDATIONS §6.6).
- **0.36.0 (24 September 2026)**: `Multibody::remove_dof_coupling`,
  `retain_dof_couplings`, `clear_dof_couplings`; extended FEM soft bodies.
- **0.36.1 (4 October 2026)**: Python access to the reduced-coordinate state
  (`Multibody.generalized_position`, `set_armature`).
- A `Multibody` is "An articulated body simulated using the
  reduced-coordinates approach", its bodies linked "following a tree structure"
  [V, docs.rs, rapier3d 0.36.1, `Multibody`, `MultibodyJointSet`]. The
  changelog never names Featherstone's articulated-body algorithm [?], though
  the description matches [S].
- `friction_model` selects `Simplified`, one Coulomb constraint per group of
  four contacts plus one twist constraint, or `Coulomb`, one per contact point:
  "This selection does not apply to multibodies that always rely on the
  `FrictionModel::Coulomb`" [V, rapier 0.36.1,
  `src/dynamics/integration_parameters.rs`].
- Dimforge publishes `rapier3d-mjcf` 0.36.1, an MJCF loader, and
  `rapier3d-urdf` 0.36.1 [V, crates.io].
- Determinism: locally deterministic by default; across platforms with the
  `enhanced-determinism` feature on IEEE 754-2008 targets, "which includes WASM
  targets"; with `parallel` as well, bitwise identical whatever the thread count
  since 0.35.0-beta.0; incompatible with `simd8`; initial values must use the
  trigonometric functions of `nalgebra::RealField`, not std's [V, Rapier user
  guide, "Determinism"; CHANGELOG, 0.35.0-beta.0].

**The official JS binding forgoes most of this** [V, the `.d.ts` files of
`@dimforge/rapier3d@0.21.0`]:
- `IntegrationParameters` exposes only `dt`, `contact_erp` (getter),
  `lengthUnit`, `normalizedAllowedLinearError`, `normalizedPredictionDistance`,
  `numSolverIterations`, `numInternalPgsIterations`, `maxCcdSubsteps`, the
  `softBodies*` fields, and a single setter with no getter,
  `contact_natural_frequency`. Absent: `contact_damping_ratio`,
  `static_contact_softness`, `friction_model`, `warmstart_coefficient`,
  `normalized_max_corrective_velocity`, `contact_clustering`,
  `contact_recycling`. The contacts can be stiffened, but the damping ratio
  stays at 10, heavily overdamped, and friction cannot be switched to `Coulomb`.
- `MultibodyJoint` exposes only `isValid()`, `setContactsEnabled()` and
  `contactsEnabled()`: no motor, limit, per-DoF spring, armature, coupling, nor
  any reading of the generalized coordinates. The 0.34 to 0.36 features are out
  of reach.
- No `MODIFY_SOLVER_CONTACTS` in `ActiveHooks`.
- f32 only.

**Rapier, in sum**: reaching its multibody features from the browser means
writing one's own Rust-to-WebAssembly wrapper. Even then its contact model is a
spring set in hertz with no force law per contact, which rules it out of the
core.

### Jolt Physics

Sequential impulses with Baumgarte stabilization (`mBaumgarte`,
`mNumVelocitySteps`, `mNumPositionSteps`, `mPenetrationSlop`,
`mSpeculativeContactDistance`, `mManifoldTolerance`, `mUseManifoldReduction`)
[V, jolt-physics 1.1.0, `dist/types.d.ts`, `PhysicsSettings`].
`ContactListenerJS` hooks the contacts from JS, which Rapier's binding does not
allow, but `ContactSettings` sets only friction, restitution, mass and inertia
scales, a sensor flag and surface velocities: no compliance. `SpringSettings`
exists on constraints, for joint limits, not on contacts. The constraints carry
dry friction: the hinge a maximum friction torque, the slider a maximum friction
force, the six-degree-of-freedom constraint one bound per axis [V, same file,
lines 1486–1502, 1584–1594, 1722–1724]. Cross-platform determinism is a build
flag, `JPH_CROSS_PLATFORM_DETERMINISTIC` (`-DCROSS_PLATFORM_DETERMINISTIC=ON`),
documented in the JoltPhysics.js README, and the release notes of JoltPhysics
5.1.0 say the Emscripten build can be compiled deterministic across platforms
[V, JoltPhysics.js README; JoltPhysics 5.1.0 release notes]; whether the npm
build enables it is not known [?]. Verdict: not for the core, for want of a
contact law; its joint friction is not the obstacle.

### Havok (through Babylon.js): eliminated

`@babylonjs/havok` 1.3.14, a 2.09 MB wasm, licence declared MIT on npm [V, npm
registry]. No source is published with it; that the engine is a closed binary
is confirmed only by a forum post [?]. An engine whose contact law can be
neither modified nor audited does not serve a project whose point is to control
that law.

### PhysX 5 (`physx-js-webidl`)

2.8.0 (27 September 2026), MIT, a 5.4 MB wasm wrapping PhysX 5.6.1, with
articulations and every joint type [V, `physx-js-webidl` README].
`PxArticulationReducedCoordinate` has "zero joint error by design, and can
handle larger mass ratios" [V, PhysX 5 documentation, Articulations]: attractive
for a lever chain. But its TGS and PGS solvers take hard contacts with a slop
and no contact force law. A user reports that revolute articulation joints with
very small masses and inertias are not driven, "regardless of the scene's solver
(PGS/TGS)" (PhysX issue #484); unreproduced [?], and the jack and the repetition
lever weigh a few grams. Verdict: not for the core.

### Box2D v3 (`box2d3-wasm`)

`box2d3-wasm` 5.2.0 (16 February 2026), MIT, "Box2D v3 for the Web, using WASM
+ SIMD + Web Workers" [V, npm registry]. Box2D v3 uses a soft-step solver whose
contact softness is set in `b2WorldDef` [V, Box2D v3, `include/box2d/types.h`]:
```
float contactHertz;        // "Contact stiffness. Cycles per second."
float contactDampingRatio; // "Contact bounciness. Non-dimensional."
float contactSpeed;        // cap on overlap resolution speed (m/s)
float maximumLinearSpeed;
```
It is planar, as the model is (FOUNDATIONS §1.4), small and fast. Its contacts
are springs set in hertz with no law per contact, and joint friction comes as a
motor held at zero speed with a bounded torque (FOUNDATIONS §3.9). Verdict: a
quick kinematic mock-up; not a touchweight, which needs a law per contact.

### cannon-es, ammo.js, planck: eliminated

`cannon-es`: last release 2022-08-12, abandoned. `ammojs-typed`: Bullet 2.8x
transpiled, a legacy solver. `planck` 1.5.0: a TypeScript port of Box2D **v2**,
readable as a reference for a 2D solver in TypeScript, of no use as an engine
here.

---

## 4. B — "Engineering" and robotics multibody engines

### MuJoCo: an optional oracle

The fact to retain: DeepMind publishes official WebAssembly bindings on npm.

- `@mujoco/mujoco` **3.15.0**, published **5 October 2026**, Apache-2.0, from
  the `wasm` directory of `google-deepmind/mujoco`, its npm maintainers including
  `yuvaltassa` [V, npm registry]. Releases from 3.6.0 (13 March 2026) to
  3.15.0, about monthly, with the engine's (3.15.0 of 5 October 2026) [V, npm
  registry; GitHub API; MuJoCo changelog].
- Its README: "These are the **canonical** JavaScript and TypeScript bindings
  for the MuJoCo physics engine […] developed and maintained by Google DeepMind
  and are always up to date"; still marked work in progress; developed and
  tested mostly on Linux and Chrome, Windows "experimental" [V,
  `@mujoco/mujoco@3.15.0` README].
- **Two builds**: single-threaded (`import loadMujoco from '@mujoco/mujoco'`)
  and multi-threaded (`'@mujoco/mujoco/mt'`, Web Workers and
  `SharedArrayBuffer`, needing `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp`) [V, same README].
- Sizes: `mujoco.wasm` **10.37 MB raw, 2.54 MB gzipped**; `mt/mujoco.wasm`
  10.48 MB and 2.60 MB [V, measured on the 3.15.0 tarball], 2.2 times Rapier
  3D's gzipped size: acceptable for a workshop tool, heavy for a public demo.
- Built from source with **emsdk 4.0.10**, `emcmake cmake -B build && cmake
  --build build`, `-DMUJOCO_WASM_THREADS=ON` for threads [V, same README].
- **Double precision**: the glue uses `HEAPF64`; `mjtNum` is `double` [V, the
  3.15.0 JS glue].

**What MuJoCo offers** (FOUNDATIONS §3.9):

1. **Friction loss, a box constraint solved softly.** Dry friction at a joint
   or a tendon is modelled "as a constraint, namely an upper limit on the
   absolute value of the force that friction can generate", its admissible set
   "the box constraint |λ_F| ≤ η" [V, MuJoCo documentation, Computation,
   constraint model]. That is the set of Thorin's `c_d·sign(θ̇)`, solved inside
   MuJoCo's convex program; but like every MuJoCo constraint it is soft, its
   impedance set by `solreffriction` and `solimpfriction`, so a joint creeps
   under a sub-threshold load (FOUNDATIONS §3.9). **Do not use MuJoCo's friction
   loss as an exact, unregularized model of axis friction**: a down weight or an
   up weight read on MuJoCo carries that creep. Whether the noslip pass (item 4)
   also holds friction loss without slip is not checked [?].
2. **Polynomial springs and dampers per degree of freedom**, added in **3.7.0
   (14 April 2026)**: `jnt_stiffnesspoly`, `tendon_stiffnesspoly`,
   `dof_dampingpoly`, `tendon_dampingpoly` [V, MuJoCo changelog, 3.7.0].
   Documented forms [V, MuJoCo XML reference, joint stiffness and damping]:
   - spring: `f(x) = −(a·x + b·x² + c·x³)`
   - damper: `f(v) = −(a·v + b·v·|v| + c·v³)` ("anti-symmetrization of the
     quadratic term")

   The spring has the form of Hirschkorn's `f_fit(x) = a·x³ + b·x² + c·x`:
   modelling each felt Thorin's way, a small auxiliary body on a prismatic joint
   with that spring, gives Hirschkorn's elastic term natively; not his damping
   (limit 4 below), nor the memory of past loads that FOUNDATIONS §4.3 targets.
3. **Reduced coordinates**, per-DoF `armature`, `springref`, tendons (for the
   lift tray and the trapwork), eight kinds of equality constraint
   (`mjEQ_CONNECT/WELD/JOINT/TENDON/FLEX/FLEXVERT/FLEXSTRAIN/DISTANCE`) [V,
   `@mujoco/mujoco@3.15.0` `.d.ts`].
4. **Convex soft contacts** with position-dependent impedance: `solref =
   (timeconst, dampratio)`, `solimp = (d₀, d_w, width, midpoint, power)`;
   `margin` and `gap`; `condim` ∈ {1, 3, 4, 6}; pyramidal or elliptic cones;
   **Newton** (the default, with a Cholesky Hessian), CG (Hager–Zhang) and PGS
   solvers; with `noslip_iterations > 0`, a PGS pass after the main solve on the
   friction dimensions with R = 0, hard constraints [V, MuJoCo documentation,
   Computation; XML reference, `option`].
5. **Integrators**: `Euler` (semi-implicit, joint damping implicit),
   **`implicitfast`** (recommended), `implicit`, `RK4`, and **`discrete`**,
   added in **3.13.0 (8 September 2026)**: "the constraint solve and the
   implicit velocity update merge into one operation" [V, MuJoCo changelog,
   3.13.0, Engine, item 4]. `implicitfast` with implicit damping suits stiff
   springs.
6. **Model editing from JS**: `parseXMLString(...) → MjSpec`,
   `mj_compile(MjSpec) → MjModel`, `mjs_addBody`, `mjs_addJoint`,
   `mjs_addGeom`, `mjs_addEquality`, `mjs_addActuator`, `mj_loadXML`,
   `mj_saveLastXML` [V, `.d.ts`]: the model is rebuilt and recompiled in the
   browser whenever the geometry changes.

**Its limits for this project:**

1. **Penetration by design**: "Contacts are soft, so during penetration both
   the normal force and velocity are positive"; the diagonal regularizer `R`
   cannot go to zero: "taking R → 0 would give a hard constraint, which MuJoCo
   does not allow" [V, MuJoCo documentation, Computation, soft constraints].
2. **Contact stiffness is a time constant, not N/m**: `timeconst` "should be at
   least two times larger than the simulation time step", enforced internally
   unless `refsafe` is turned off [V, MuJoCo XML reference, `solref`]. With the
   `discrete` integrator, constraint rows are implicit and stable at any
   `timeconst`, and `refsafe` replaces rows stiffer than the step can resolve by
   the stiffest zero-restitution row instead of clamping `timeconst` [V, MuJoCo
   changelog, 3.13.0, Engine, item 4]. As an order of magnitude [S],
   `k_eff ≈ m_eff/τ²`: for 10 g and τ = 2·10⁻⁴ s, twice an assumed step of
   10⁻⁴ s, k_eff ≈ 2.5·10⁵ N/m, 40 µm of compression under 10 N. Sub-millimetre
   is reachable, through a numerical parameter, not through the felt's physics.
   Thorin's remedy carries over: the compliance goes into the felt body's
   prismatic joint, a polynomial spring, and the contacts only enforce
   unilaterality.
3. **No custom passive force from JS**: the `.d.ts` files expose only
   `mj_resetCallbacks()`, no setter for `mjcb_passive`, and the glue contains no
   `mjcb_` [V, `@mujoco/mujoco@3.15.0`]. Plugins declare the
   `mjPLUGIN_PASSIVE` capability [V, MuJoCo changelog], but using one means
   compiling one's own WebAssembly build: writing C++ rather than using an
   engine as published.
4. **Forms not natively expressible**: Hirschkorn's `f(δ)·(1 + D·δ̇)` and
   Thorin's `b·δ²·δ̇` are damping terms that depend on position; MuJoCo's
   polynomial damper depends on velocity alone. Approximation is possible,
   exactness is not.
5. **Scale**: the oracle role needs one note. Constraint islands are the
   default since 3.3.6, `mjDSBL_ISLAND` being the flag that disables them [V,
   MuJoCo changelog, 3.3.6]. The 88-note runtime is the project's own:
   independent notes, the pedals as prescribed inputs (FOUNDATIONS §7.6).

### The others, one by one

| Tool | Status, October 2026 | WebAssembly? | Verdict |
|---|---|---|---|
| **Siconos** (non-smooth dynamics, Moreau–Jean) | tag **`2026.rc1`** (11 March 2026); documentation still at 4.5.0; FreeBSD port 4.5.0_4 maintained (18 July 2026) [V, Siconos GitHub tags; FreeBSD ports] | no WebAssembly build found [?] | The reference academic framework for this formulation, and the hardest to bring into a browser (BLAS/LAPACK, Boost, Python): a theoretical reference, or an oracle outside the browser |
| **Project Chrono** | **10.0.0** (27 March 2026), BSD-3 [V, Chrono GitHub releases] | **yes**: an official "Build Chrono for WASM" tutorial, `emcmake ccmake -G Ninja`, Eigen ≥ 3.4.0, GLM optional for WebGL views [V, Chrono documentation] | An **NSC** solver (complementarity, Anitescu–Tasora) and an **SMC** one (penalty). Large, a C++ API with no JS binding: a candidate for an offline cross-check, not a web runtime |
| **Pinocchio** | community WebAssembly ports: `Mostafasaad1/pinocchio-js` (release 1 March 2026: RNEA, forward kinematics, centre of mass, Jacobian) and the `ucalgary-shivers` fork (adds ABA and CRBA; Pinocchio v2 with Eigen 3.4; binary about 457 kB) [V, the two GitHub repositories] | unofficial | No unilateral contact: articulated-tree dynamics only. A building block (M, gradients, ABA) for a solver written around it, not an engine |
| **Drake** | — | none found [?] | Out of the running in a browser; its `MultibodyPlant` with the SAP solver fits the formulation conceptually |
| **Simbody / OpenSim** | — | none found [?]; LAPACK and BLAS dependencies | Out of the running |
| **Robotran** (UCLouvain) | used for a six-degree-of-freedom grand action model [V, Thorin 2017, Table 2] | no | Not a runtime; the reference for **symbolic** generation of the equations (§7) |
| **MBDyn** | — | none found [?] | Out of the running |
| **Kane and symbolic tools** (SymPy mechanics, Maple with DynaFlexPro) | SymPy `sympy.physics.mechanics`: `KanesMethod`, `LagrangesMethod`, code generation through `sympy.utilities.codegen`, `sympy.printing.c`, `sympy.printing.rust` [S] | through C or Rust compiled to WebAssembly | An option for deriving the equations (§7): the route of Hirschkorn and Izadbakhsh (DynaFlexPro) and of Bokiau and co-authors (Robotran) |

---

## 5. C — Rust building blocks (crates.io, observed 11 October 2026)

`pa-core` has one dependency, the `libm` crate, through which every
transcendental goes (FOUNDATIONS §4.4); `serde`, behind an optional feature for
persistence, is the planned other (FOUNDATIONS §7.4). The table records what
exists.

| Crate | Version | Last update | Role, verdict |
|---|---|---|---|
| **nalgebra** | **0.35.0** | 2026-05-24 | Not a dependency. Statically sized matrices (`Matrix6<f64>`, `SMatrix`) with `Cholesky` and `LU` on the stack, no allocation; `RealField` and `ComplexField` give the portable trigonometry Rapier's cross-platform determinism requires |
| nalgebra-sparse | 0.12.0 | 2026-05-24 | Of no use at six coordinates |
| ndarray | 0.17.2 | 2026-01-10 | Of no use here (dynamic n-dimensional arrays) |
| faer | 0.24.4 | 2026-06-24 | For large systems, beyond about 100×100; at 6×6 its call overhead would dominate |
| **ode_solvers** | 0.6.2 | 2026-06-07 | Dormand–Prince, Runge–Kutta, Radau: for an offline reference integration, not a fixed step |
| **diffsol** | **0.17.1** | 2026-09-28 | The most active Rust ODE solver; BDF and SDIRK with a sparse Jacobian; for an offline reference of stiff integration; whether its Sundials and LLVM features build for WebAssembly is not checked [?] |
| peroxide | 0.43.1 | 2026-08-02 | A general scientific toolbox; not needed |
| **num-dual** | 0.15.1 | **2026-10-05** | Dual and hyper-dual numbers: exact Jacobians of smooth contact laws. Not a dependency. Sensitivities of the trade's readings are planned by finite differences (FOUNDATIONS §5.3, level 5): a reading taken at a contact event or a friction threshold is not differentiable across events. **Do not use "+0.1 mm of capstan ⇒ −1.4 g of down weight"**: it is a display mock-up, not a computed or measured value, and raising the capstan increases aftertouch (FOUNDATIONS §5.3) |
| enzyme (Rust autodiff) | nightly, experimental [S] | — | Not for production |
| **rapier2d / rapier3d** | 0.36.1 | **2026-10-10** | Not used (§9); its multibody module does not replace a solver whose contact law is the point |
| rapier3d-f64 / rapier2d-f64 | 0.36.1 | 2026-10-10 | The f64 variants, to be compiled by their user |
| rapier3d-mjcf / rapier3d-urdf | 0.36.1 | 2026-10-10 | MJCF and URDF loading in Rapier. MJCF is also the route by which a note would reach the optional MuJoCo oracle (§8) |
| parry2d / parry3d | 0.31.1 | 2026-09-18 | Not used: the contact primitives are the project's own, circle against line and circle against circle built, the hybrid profile planned, with exact analytic lever arms (FOUNDATIONS §6.2, §6.6); Thorin's model uses the same three kinds of geometry [V, Thorin 2017 §2.4, Table 4] |
| **nphysics3d** | 0.24.0 | **2021-08-08** | Abandoned, "superseded by the rapier3d crate" [V, crates.io description] |
| **k** | 0.32.0 | **2024-09-10** | Kinematics only, no dynamics; stagnant |
| **rbdl-rs** | not on crates.io | — | — |
| mujoco (Rust bindings) | not checked [?] | — | The official WebAssembly bindings are the ones §4 relies on |
| **rayon** | 1.12.0 | 2026-04-14 | Not used; native parallelism only |
| **wasm-bindgen-rayon** | 1.3.0 | **2024-12-21** | Last published on that date; depends on `wasm-bindgen ^0.2.99`, so compatible with 0.2.129 by caret [V, crates.io API]. Not usable: WebAssembly threads need the standard library rebuilt with atomics, which needs nightly Rust (FOUNDATIONS §3.10); the planned runtime is one module per Web Worker, notes sharing nothing (FOUNDATIONS §7.6) |
| wasm-bindgen | 0.2.129 | 2026-09-25 | Used by `pa-wasm` (built) |
| argmin | 0.11.0 | 2025-09-28 | For identifying parameters offline, such as fitting felt load curves as Hirschkorn did |
| roots | 0.0.8 | **2022-12-21** | Stagnant; a bracketed search is a few lines (the settle solve's bisection is built, FOUNDATIONS §2.1) |

**WebAssembly SIMD**: fixed-width SIMD128 ships in Chrome 91, Firefox 89 and
Safari 16.4; relaxed SIMD reaches Safari only as a preview and iOS Safari not at
all [V, `@mdn/browser-compat-data` 8.1.5, `webassembly.fixed-width-SIMD`,
`webassembly.relaxed-SIMD`]. Relaxed SIMD is ruled out in any case: its fused
multiply-add rounds once on hardware that has one and twice elsewhere
(FOUNDATIONS §4.4). Vectorized across notes, SIMD128 carries two f64 lanes, two
notes at a time; whether it helps a scalar 6×6 kernel is not measured [?].

---

## 6. D — Pure TypeScript against Rust compiled to WebAssembly

**Do not use the per-step costs of 157.5 ns (JavaScript), 108.4 ns
(WebAssembly) and 92.4 ns (native Rust), the ratios drawn from them (1.45
between WebAssembly and JavaScript, 1.17 and 1.70 for native code), nor the
shares of a core built on them**: the benchmark kernel behind them is not in
the repository (FOUNDATIONS §4.6). The cost per step is one of the
dynamics prototype's tests, natively and in Chrome and Safari (FOUNDATIONS §4.6).

### Determinism

Addition, subtraction, multiplication, division and the square root are exactly
rounded under IEEE-754, in JavaScript, in WebAssembly and in native Rust alike,
so code using nothing else, with the same operations in the same order and no
multiply and add contracted into one fused operation, gives bit-identical
results in all three (FOUNDATIONS §4.4).

Transcendentals break this:
- ECMAScript leaves `Math.sin`, `Math.cos`, `Math.exp` and the rest
  implementation-approximated [V, ECMA-262, §21.3.2, Function Properties of the
  Math Object]; MDN warns that the precision is implementation-dependent and
  that "the same JavaScript engine on a different OS or architecture can give
  different results" [V, MDN, JavaScript reference, `Math`].
- In Rust, a transcendental called through the `libm` crate compiles the same
  code into the native binary and into the WebAssembly module, so both agree
  bit for bit, in every browser; the standard library's `f64::sin` calls the
  platform's library natively, and differs (FOUNDATIONS §4.4). `pa-core` calls
  its transcendentals through the `libm` crate, and Clippy refuses the
  standard library's there (FOUNDATIONS §4.4).

A technician who reads 52 g of down weight with a set of settings must read the
same 52 g on another machine the next day: reproducibility is structural to the
instrument.

**JavaScript integers**: a product such as `s * 1103515245` loses bits once it
exceeds 2⁵³, so a pseudo-random generator written that way diverges from the
same generator in Rust; `Math.imul` reproduces 32-bit integer multiplication.
The divergence is silent.

**Verdict D.** Pure TypeScript is viable and would iterate faster at first. Rust
compiled to WebAssembly is chosen for reasons that are not speed: arithmetic
reproducible across machines, no garbage collector in a fixed-step loop, and
native runs of the code that ships, for studies, sweeps and golden tests
(FOUNDATIONS §3.10).

---

## 7. Computation budget and execution

**The budget is a hypothesis** (FOUNDATIONS §4.6): one dynamic solver, notes
computed independently on a few workers, plays 88 notes in real time on a
laptop, because a note moves for a few hundred milliseconds and holds still
after, so far fewer than 88 move at once. Its test is the prototype's cost per
step, natively and in Chrome and Safari, and the number of notes active at once
over a corpus of MIDI files, pedal changes included. A note at rest, every
velocity zero and every contact steady, could sleep at almost no cost [S].

**Execution** (planned, FOUNDATIONS §7.6): one WebAssembly module per Web
Worker, neither on the main thread nor in an AudioWorklet; a lock-free ring in a
`SharedArrayBuffer` where the page is cross-origin isolated, transferable
buffers otherwise; the renderer interpolating between snapshots stamped in
simulation time. All the `AudioWorkletNode`s of one `AudioContext` run on its
single rendering thread [V, W3C Web Audio API, `AudioWorklet`], so the
AudioWorklet offers no parallelism. `SharedArrayBuffer` requires cross-origin
isolation, `Cross-Origin-Opener-Policy: same-origin` with
`Cross-Origin-Embedder-Policy: require-corp` [V, MDN, `SharedArrayBuffer`]. The
bench, one static key, runs its module on the main thread (built).

**Symbolic generation of the equations** is the route of Hirschkorn, whose
equations DynaFlexPro generates in symbolic form in Maple [V, Hirschkorn 2004
§2.1], of Izadbakhsh, also with DynaFlexPro, and of Bokiau and co-authors with
Robotran [V, Thorin 2017, Table 2]; Thorin uses XDE, a numerical engine [V,
Thorin 2017 §4]. `M(q)`, the Coriolis terms and the contact Jacobians can be
derived with `sympy.physics.mechanics` (Kane or Lagrange) and emitted as Rust
[S]: a dense 6×6 `M(q)` with no recursion, and analytic Jacobians. Derived by
hand or generated, the mass matrix is tested against finite differences of the
kinetic energy (FOUNDATIONS §3.6, §5.3); FOUNDATIONS §3.6 gives the couplings in
relative coordinates, with the formulas not to use.

---

## 8. Architecture

The layering this study arrives at is the one built (FOUNDATIONS §7.1): a domain
crate, `pa-core`, in Rust, with no I/O, no web and one dependency, `libm`,
tested natively; an application crate, `pa-app`; and two adapters that add no
physics, `pa-wasm` for the browser and `pa-cli` for the shell. The bench renders
with three.js 0.186.1 through react-three-fiber (FOUNDATIONS §3.11).

What the domain is to gain, each a plan or a hypothesis of FOUNDATIONS:
- contact geometry with exact analytic lever arms: circle against line and
  circle against circle built, the hybrid profile planned (§6.2, §6.6);
- a felt law, Hirschkorn's cubics as the elastic term, a rate-independent law
  with memory as the target (§4.3, a hypothesis);
- set-valued friction at every pivot, built for the key's balance (§3.5);
- one fixed-step dynamic solver with compliant contacts and friction as bounded
  unknowns at velocity level, the quasi-static solver as its static limit (§4.2,
  a hypothesis);
- sensitivities by finite differences (§5.3, level 5);
- arithmetic reproducible under the conditions of §4.4: every transcendental
  through the `libm` crate (built), native and WebAssembly results compared bit
  for bit (planned).

**Order of work**: FOUNDATIONS §8 holds the stages, by sub-assembly along
Stanwood's equation, with the dynamics prototype that settles the step, the
solver and the felt law (FOUNDATIONS §4) before the wippen. MuJoCo is not a
stage: it enters, if at all, as the optional oracle of FOUNDATIONS §3.9. Its
route is an MJCF model of one note, auxiliary felt bodies on polynomial springs
and friction loss at the pins, loaded with `@mujoco/mujoco`; MJCF is read by
MuJoCo and, through `rapier3d-mjcf` 0.36.1, by Rapier, and expresses joints,
friction loss, polynomial springs, tendons, equality constraints and
`solref`/`solimp`. Agreement between two solvers of one model is a consistency
check, not a validation (FOUNDATIONS §5.1); Thorin's and Hirschkorn's measured
curves and the trade's protocols validate (FOUNDATIONS §5.2).

---

## 9. General-purpose engines in secondary roles

1. **Cross-checking oracle: MuJoCo through MJCF** (planned, optional;
   FOUNDATIONS §3.9). One geometry, two independent solvers, double precision on
   both sides. A divergence on the hammer's speed at let-off is a difference to
   explain, in either model, since MuJoCo's friction loss creeps and its contact
   stiffness is numerical (§4).
2. **Trapwork, pedals and lyre: not given to an engine.** The trapwork is one to
   three degrees of freedom driven by timestamped pedal input and integrated
   identically in every worker (FOUNDATIONS §7.6), and no general-purpose engine
   is a dependency of the project (FOUNDATIONS §3.9).
3. **Kinematic exploration in Box2D v3 (`box2d3-wasm` 5.2.0) or
   `@dimforge/rapier2d`**: possible as a throwaway outside the project. The
   bench's own drawing, from the outlines the engine computes, is where a lever
   ratio, a wippen's layout or a capstan's height is checked by eye (FOUNDATIONS
   §3.2). A touchweight is never read off a game engine.
4. **Not to do**: bending Rapier or Jolt into the core through
   `contact_natural_frequency`, `lengthUnit` and bounded motors. Their contact
   model cannot be replaced, and every term of the core must be one a
   technician's question can be traced to.

---

## 10. Summary: who can do what

| Need | Rapier (JS) | Rapier (Rust) | Jolt | Havok | PhysX | Box2D v3 | **MuJoCo** | **Dedicated solver** |
|---|---|---|---|---|---|---|---|---|
| Reduced coordinates | partial (bare API) | yes | no | no | yes | no | **yes** | **relative coordinates** (planned, FOUNDATIONS §3.6) |
| Double precision | **no (f32)** | yes (f64 crate) | build option | ? | build option | no | **yes** | **yes** |
| Force law per contact | **no** | partial | **no** | **no** | **no** | **no** | partial (cubic polynomial) | **yes** (planned, FOUNDATIONS §4.3) |
| Polynomial spring per DoF | **no** | yes (0.34+) | no | no | no | no | **yes (3.7.0+)** | **yes** |
| Set-valued dry friction at the axes | no | bounded motor | **yes** (hinge maximum friction torque) | no | bounded motor | bounded motor at zero speed | **soft** (`frictionloss`) | **yes** (the key's balance built; every pivot planned) |
| Unregularized tangential friction | no | no | no | no | no | no | **with noslip** (cone) | **yes** (planned) |
| Geometric tolerance < 0.1 mm | after rescaling | after rescaling | slop | ? | slop | slop | through `solref`/`solimp` | **analytic** |
| Cross-platform determinism | not verified | yes (feature) | build flag | no | no | [?] | [?] | **under FOUNDATIONS §4.4's conditions** |
| Model editing from JS | partial | — | partial | partial | partial | partial | **yes (`MjSpec` + `mj_compile`)** | **yes** |
| Binary size (gzip) | 1.17 MB | the user's | 742 kB | 658 kB | 1.61 MB | [?] | 2.54 MB | 50–200 kB [S] |
| Licence | Apache-2.0 | Apache-2.0 | MIT | MIT (no published source) | MIT | MIT | **Apache-2.0** | undecided (FOUNDATIONS §9) |
