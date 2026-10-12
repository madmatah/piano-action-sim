> Translated from the original French research annex. Part of the `piano-action-sim` foundations
> study (October 2026). See `docs/FOUNDATIONS.md` for the decisions this annex supports.

## 0. What is VERIFIED vs ASSUMED

The primary sources below were **downloaded and read in full** (not just research summaries):

| Document | Local path | What we take from it |
|---|---|---|
| Thorin, Boutillon, Lozada, Merlhiot, *Non-smooth dynamics for an efficient simulation of the grand piano action*, Meccanica 2017 | `data/references/papers/thorin2017.pdf` | Moreau-Jean scheme, **dt = 500 µs**, 2-10 kHz convergence, 20x real time on a 2010 i5, felt law, 1 mg auxiliary-body trick |
| Hirschkorn, *Dynamic Model of a Piano Action Mechanism*, MASc Waterloo 2004 (85 pp.) | `data/references/papers/hirschkorn_thesis.pdf` | **complete parameter set**: masses, inertias, 19 geometric points, 13 contact laws (a,b,c,D), friction, springs, regulation specs |
| Stulov, *Experimental and computational studies of piano hammers*, Acta Acustica 2005 | `data/references/papers/stulov_actaa2005.pdf` | hereditary hammer-felt model **parameterized over all 88 notes** |
| Goebl, Bresin, Galembo, *Touch and temporal behavior of grand piano actions*, JASA 118(2), 2005 | `data/references/papers/goebl2005.pdf` | finger forces, hammer velocities, travel durations, free flight |
| Goebl & Bresin, *Measurement and reproduction accuracy of computer-controlled grand pianos*, JASA 114(4), 2003 | `data/references/papers/goebl2003.pdf` | MIDI 77 ↔ 2 m/s anchor, FHV = 128/IHV, solenoid ceilings |

Reproducible computation scripts, in `tools/derivations/` — every number in this
report comes out of one of them, and they run anywhere with `python3 -I`:
- `03-contact-stiffness-and-time-step.py` — contact stiffnesses, natural
  frequencies, time steps, flop budget
- `09-action-ratio-and-balance-closure.py` — the action ratio as a lever product,
  and Stanwood's equation of balance with the Key Ratio in the force convention
- The per-note Stulov figures are **`[E]`**: derived by hand, not reproduced by a
  script. Treat them as indicative, not as verified values.

What is **ASSUMED** (flagged as such in the text): my flops-per-step estimates (45 kflop for non-smooth, 2500 for penalty), the per-thread Gflop/s throughputs, the number of concurrent notes, and the one-pole-filter trick for the hereditary integral.

---

## 1. Planar model formulation for ONE note

### 1.1 Bodies and generalized coordinates

Six bodies, **all in pure rotation** in the sagittal plane. Critical choice: **RELATIVE coordinates** for the wippen subtree.

| Body | Coordinate | Pivot | Nature |
|---|---|---|---|
| Key | `θ_k` | balance rail pin, fixed to the frame | absolute |
| Wippen | `θ_w` | wippen center pin, fixed to the frame | absolute |
| Jack | `θ_j` | center pin **carried by the wippen** | **relative to θ_w** |
| Repetition lever | `θ_r` | center pin **carried by the wippen** | **relative to θ_w** |
| Hammer (shank + head) | `θ_h` | hammer-flange center pin, fixed to the frame | absolute |
| Damper under-lever | `θ_d` | fixed to the frame | absolute |

The point you have to understand so as not to get the architecture wrong: **the jack→wippen and repetition lever→wippen pivots are REAL kinematic pivots** (center pins with felt bushings), not contacts. In relative coordinates the kinematic tree is complete and the system is a **minimal-coordinate ODE with no algebraic constraint whatsoever**. In absolute angles, those two pivots become 4 scalar constraints and you inherit an index-3 DAE, constraint drift, Baumgarte stabilization and its non-physical parameters. No benefit at all.

### 1.2 EXACT structure of the mass matrix

The (w, j, r) block is an **ARROWHEAD, not a full 3x3 matrix**, because `M_jr = 0` exactly — neither the jack nor the repetition lever is an ancestor of the other in the tree (they are two sibling children of the wippen).

```
       θ_k    θ_w      θ_j      θ_r    θ_h   θ_d
θ_k [  I_k     0        0        0      0     0  ]
θ_w [   0   M_ww(q)  M_wj(q)  M_wr(q)   0     0  ]
θ_j [   0   M_wj(q)    I_j       0      0     0  ]
θ_r [   0   M_wr(q)     0       I_r     0     0  ]
θ_h [   0      0        0        0     I_h    0  ]
θ_d [   0      0        0        0      0    I_d ]
```

with
- `I_k = I_k^cm + m_k d_k²`, `I_h`, `I_d`, `I_j = I_j^(Pw3)`, `I_r`: **constants**
- `M_ww(q) = I_w^(Pw2) + Σ_{b∈{j,r}} [ I_b^cm + m_b ‖p_b^cm(q) − P_w2‖² ]`
- `M_wj(q) = I_j^cm + m_j (p_j^cm − P_w3)·(p_j^cm − P_w2)` — depends on the wippen/jack angle
- `M_wr(q)` likewise

So: **closed-form inversion of M in ~10 flops** (3x3 arrowhead solver + 3 divisions). No LU factorization, no pivoting, no allocation. Structural determinism.

The configuration dependence of `M_ww, M_wj, M_wr` is weak (`I_j = 7.78e-7`, `I_r = 4.10e-6 kg·m²`, travel of a few degrees) but it must be kept for exactness — it costs 6 sin/cos per step, which we precompute anyway for the contact geometry.

### 1.3 Equations of motion, implementable form

Lagrange in minimal coordinates:

```
M(q) q̈ + C(q, q̇) q̇ = τ_g(q) + τ_spring(q) + τ_fric(q̇) + Σ_i J_i(q)ᵀ f_i(g_i, ġ_i)
```

- `C(q,q̇) q̇`: Coriolis/centrifugal terms. **Non-zero only within the (w,j,r) subtree**. Computable either by Featherstone's RNEA (overkill for 3 bodies) or analytically: `C_ww = ∂M_ww/∂θ_j · θ̇_j + ∂M_ww/∂θ_r · θ̇_r`, etc. With ω ≤ ~30 rad/s and inertias on the order of 1e-6, these terms are small but not negligible during let-off.
- `τ_g(q)`: gravity. For the key, it **explicitly includes the key leads** as parameterizable point masses `(x_i, m_i)` — this is a regulation item you want to manipulate, so it must be first-class data, not a center-of-mass correction.
- `τ_spring`: repetition spring. Hirschkorn measures `T = −0.073·(θ_r − 0.832)` N·m (linear) and `T_jack ≈ 0.0042 N·m` (quasi-constant for the jack). Thorin uses the same form `C_s(θ) = κ(θ − θ_0)`.
- `τ_fric(q̇)`: friction at the pins. **This is where all of the DW/UW lives.** Hirschkorn regularizes: `T = A·tanh(ω/ω_t) + B1(ω/ω_t)/(1 + B2(ω/ω_t)⁴)` with A measured per pin: key 0.0122; wippen 0.000397; jack 0.000397; repetition lever 0.00247; hammer 0.00101 N·m. Thorin does NOT regularize: `C_d(θ) = c_d · sign(θ̇)` with `sign` set-valued on `[−1, 1]`. **Thorin is the one to follow** (see §3).
- `J_i(q)ᵀ f_i`: unilateral contacts, 13 of them (see §1.5).

### 1.4 The 13 contacts (Hirschkorn nomenclature, Table 2.1/A.8)

| # | Contact | Regulation role |
|---|---|---|
| 1 | capstan → wippen heel | capstan height, weight |
| 2 | key front → front punching | **key dip**, aftertouch |
| 3 | key heel → rest rail cloth | rest position |
| 4 | back check → hammer tail | **back check regulation** |
| 5 | wippen → jack rest | jack rest position |
| 6 | wippen → repetition lever stop | — |
| 7 | **jack → knuckle (roller)** | **main force path** |
| 8 | jack toe → let-off button | **let-off** |
| 9 | jack → repetition lever window | — |
| 10 | **repetition lever → knuckle** | repetition |
| 11 | repetition lever → drop screw | **drop** |
| 12 | hammer felt → string | blow distance |
| 13 | hammer shank → rest rail | hammer rest height |

Thorin counts 12 (he omits the damper, and handles the keybed differently).

### 1.5 SLIDING contacts: the heart of the VARIABLE leverage ratio

This is the central technical point of your request, and it is **entirely solvable analytically**. Three primitives suffice — Hirschkorn and Thorin independently arrive at the same ones:

**(a) Circle against line** (10 of the 13 contacts in both authors).
Body A carries a circle (center `c_A`, radius `R_A`), body B carries a line (point `p_B`, unit normal `n_B`). In world frame:

```
g        = (c_A − p_B)·n_B − R_A           (gap, > 0 = separated)
n        = n_B
x_c      = c_A − R_A·n_B                   (contact point)
∂g/∂θ_A  = ( ẑ × (c_A − o_A) ) · n_B       (instantaneous lever arm, A side)
∂g/∂θ_B  = −( ẑ × (x_c − o_B) ) · n_B  −  (c_A − p_B)·( ẑ × n_B )
```

`x_c` **moves along the line** as the bodies rotate. That is exactly what the variable leverage ratio is, physically: `∂g/∂θ` is the instantaneous lever arm, and it varies continuously throughout key depression. Cost: ~15 flops. Exact, no iteration, no closest-point search.

**(b) Circle against circle** (back check/hammer tail in Thorin; Hirschkorn also uses `Pk6`/`Ph3`).
```
d = c_A − c_B ;  g = ‖d‖ − (R_A + R_B) ;  n = d/‖d‖ ;  x_c = c_A − R_A·n
```

**(c) "Hybrid" profile = arc + two tangent lines, against a circle** (jack/knuckle).
Hirschkorn: `Pj2` = arc of radius 1.0 mm over the angular interval [1.571; 3.142] rad plus two tangent lines, against the circle `Ph2` of radius 4.4 mm (the knuckle). `Pr2` = arc of radius 55.2 mm over [1.300; 1.571] against the same knuckle. Thorin: "jack-roller contact geometry consists of two perpendicular lines joined with a circular arc".
Algorithm: classify the angular position of `c_B` relative to the arc center and its angular extent → we fall back on either (b) or (a). **The construction is C1 by construction** since the lines are tangent to the arc: the normal never jumps. That is precisely WHY both authors chose this primitive — the jack tip sliding on the knuckle requires a continuously varying normal, otherwise the transmitted torque jumps.

**Relative tangential velocity** (for friction):
```
t    = ẑ × n
v_t  = [ (θ̇_A ẑ × (x_c − o_A)) − (θ̇_B ẑ × (x_c − o_B)) ] · t
```

**Observable to expose in the UI**: the instantaneous leverage ratio
```
ratio(t) = ( r_strike · θ̇_h ) / ( r_front · θ̇_k )
```
2 flops. To be plotted live against key dip. **Quantified validation target** (BYU Design Review): ≈ **5.5:1 at the top of the travel → 5.06:1 "true" → ≈ 5.0:1 at let-off**, modern target (4.9-5.3):1. That is a variation of ~10 % over the travel — exactly what you want to see move when you change the capstan height or the hammer hang.

---

## 2. Stiffnesses and time steps: the numbers

### 2.1 Contact law and measured coefficients

Hirschkorn measures **each contact individually** on a disassembled Boston GP-178 action (one part in a vise, the other on a torque-controlled motorized arm, encoder for the rotation), then fits:

```
f_n = f_fit(x) · (1 + D·ẋ)        with   f_fit(x) = a·x³ + b·x² + c·x
```

x in m, f in N. **No pow(): 4 flops.** Table A.8 (raw values):

| # | contact | a | b | c | D (s/m) |
|---|---|---|---|---|---|
| 1 | capstan/wippen heel | 1.26e10 | 3.26e7 | 7.92e3 | 1.877 |
| 2 | key front/punching | 4.41e10 | −1.02e7 | 1.57e4 | 1.219 |
| 3 | key heel/cloth | 2.42e10 | −1.11e7 | 6.54e3 | 2.502 |
| 4 | back check/hammer tail | 5.86e11 | −3.70e7 | 3.11e4 | 2.446 |
| 5 | wippen/jack rest | 5.28e9 | −7.54e5 | 4.42e3 | 5.781 |
| 6 | wippen/rep. lever stop | 2.53e10 | −9.89e6 | 7.07e3 | 7.732 |
| **7** | **jack/knuckle** | **3.92e11** | **1.10e7** | **3.23e4** | **4.694** |
| 8 | jack toe/let-off button | 7.46e10 | −2.29e7 | 1.32e4 | 1.527 |
| 9 | jack/lever window | 6.02e10 | −1.68e7 | 6.96e3 | 0.779 |
| **10** | **rep. lever/knuckle** | **1.09e12** | **−7.51e7** | **2.23e4** | **2.935** |
| 11 | rep. lever/drop screw | 8.82e11 | −7.59e7 | 1.67e4 | 4.985 |
| 12 | hammer felt/string | 1.81e10 | 8.19e7 | 1.16e4 | 0.175 |
| 13 | shank/rest rail | 8.58e8 | −1.11e6 | 2.39e3 | 8.964 |

Order of magnitude of the measured loading curve (Fig. 2.5, key-front punching): **40 N for 1 mm of penetration**, with a clearly visible loading/unloading hysteresis.

### 2.2 Reflected inertial masses: the key to the problem

The trap, and nobody says it explicitly in the literature: **the contact frequency is not driven by the stiffness but by the tiny REFLECTED masses of the jack and the repetition lever.** With `m_eff = I_pivot / r²`:

| Point | lever arm r | m_eff |
|---|---|---|
| key at the capstan (Pk3) | 120.4 mm | **213.6 g** |
| key at the front (Pk4) | 217.5 mm | 65.5 g |
| wippen at the capstan (Pw5) | 39.9 mm | 28.4 g |
| **jack at the tip (Pj2)** | 49.5 mm | **0.73 g** |
| **jack/window (Pj5)** | 47.8 mm | **0.78 g** |
| **rep. lever at the knuckle (Pr2)** | 63.5 mm | **1.02 g** |
| rep. lever/drop screw (Pr5) | 56.1 mm | 1.31 g |
| hammer at the knuckle (Ph2) | 19.3 mm | **414.4 g** |
| hammer at the tail (Ph3) | 81.3 mm | 23.3 g |
| hammer at the strike point (Ph4) | 136.8 mm | 8.2 g |

Reflected mass ratio **414 g / 0.73 g ≈ 570:1** at the jack/knuckle contact. This is exactly the regime in which game-engine PGS/TGS solvers converge badly (see §3).

### 2.3 Contact natural frequencies and required time steps

Method: do **NOT evaluate the stiffness at an arbitrary penetration** (that yields absurdities — 1 mm at the lever/knuckle contact corresponds to 1037 N, which never happens). Evaluate at a **physically plausible peak force at fff**, by solving `f_fit(x) = F_peak` by bisection, then `k = 3ax² + 2bx + c`, then `f = √(k/m_red)/2π`.

| # | contact | peak F (N) | x (mm) | k (N/m) | m_red (g) | **f (Hz)** | T (µs) | T/20 | T/50 | 2/ω |
|---|---|---|---|---|---|---|---|---|---|---|
| **7** | **jack/knuckle** | 100 | 0.582 | 4.44e5 | 0.728 | **3931** | 254 | **12.7** | 5.1 | 81 |
| **10** | **rep. lever/knuckle** | 25 | 0.283 | 2.42e5 | 1.020 | **2450** | 408 | 20.4 | 8.2 | 130 |
| 11 | rep. lever/drop screw | 15 | 0.262 | 1.58e5 | 1.308 | 1751 | 571 | 28.6 | 11.4 | 182 |
| 12 | hammer felt/string | 800 | 2.469 | 7.47e5 | 8.232 | 1516 | 660 | 33.0 | 13.2 | 210 |
| 8 | jack toe/button | 25 | 0.715 | 9.49e4 | 2.814 | 924 | 1082 | 54.1 | 21.6 | 345 |
| 9 | jack/window | 3 | 0.352 | 1.75e4 | 0.512 | 931 | 1074 | 53.7 | 21.5 | 342 |
| 4 | back check/hammer tail | 15 | 0.253 | 1.25e5 | 16.50 | 438 | 2283 | 114 | 45.7 | 727 |
| 6 | wippen/lever stop | 3 | 0.412 | 1.18e4 | 2.059 | 381 | 2625 | 131 | 52.5 | 836 |
| 1 | capstan/wippen heel | 60 | 1.060 | 1.20e5 | 25.07 | 347 | 2879 | 144 | 57.6 | 916 |
| 5 | wippen/jack rest | 3 | 0.540 | 8.23e3 | 1.727 | 347 | 2879 | 144 | 57.6 | 916 |
| 2 | key front/punching | 50 | 1.003 | 1.28e5 | 65.47 | 223 | 4487 | 224 | 89.8 | 1428 |
| 3 | key heel/cloth | 5 | 0.592 | 1.88e4 | 63.53 | 87 | 11540 | 577 | 231 | 3673 |
| 13 | shank/rest rail | 2 | 0.948 | 2.60e3 | 12.73 | 72 | 13901 | 695 | 278 | 4425 |

**TIME-STEP CONCLUSION, PENALTY APPROACH:**

| Phase | dominant constraint | dt accuracy (T/20) | dt tight accuracy (T/50) | dt explicit stability (2/ω) |
|---|---|---|---|---|
| **Free flight** (hammer after let-off, no contact) | 1st shank mode ≈ 280 Hz | ~180 µs | ~70 µs | ~1100 µs |
| **Key depression / hold** (contacts 1, 2, 7 loaded) | jack/knuckle 3.9 kHz | **12.7 µs** | 5.1 µs | 81 µs |
| **Let-off** (contacts 7, 8, 10, 11 simultaneous) | rep. lever/knuckle 2.5 kHz + button | **12-20 µs** | 5-8 µs | 81-130 µs |
| **Hammer-string impact** (1-4 ms) | §2.4 | **2-5 µs** | 1-2 µs | 10-40 µs |

**TIME-STEP CONCLUSION, NON-SMOOTH APPROACH** (Thorin, experimentally verified):
**dt = 500 µs (2 kHz)**, with a convergence study showing **no significant difference between 2 kHz and 10 kHz**. I recommend **250 µs** as a precaution (factor-of-2 margin). That is a gain of **25 to 100x on the time step**, and the authors quantify it: "the non-smooth formalism proved to allow for simulations of about **two orders of magnitude faster** than the regularizing approach".

### 2.4 Hammer-string impact

Measured durations (Askenfelt & Jansson, KTH): **~4 ms in the bass, <1 ms in the extreme treble**, a variation of **±20 %** over the p→ff range relative to mezzo forte, i.e. ~10 % of the fundamental period in the bass.

Stulov's model, parameterized over all 88 notes:
```
F(u(t)) = F0 · [ u^p(t) − (ε/τ0) ∫₀ᵗ u^p(ξ) exp((ξ−t)/τ0) dξ ]
m0 = 11.074 − 0.074·N + 0.0001·N²   [g]      (A0 = 11.0 g ; C8 = 5.3 g)
p  = 3.7 + 0.015·N                            (3.72 → 4.98)
ε  = 0.9894 + 0.000088·N                      (0.9895 → 0.9972)
τ0 = 2.72 − 0.02·N + 0.00009·N²    [µs]       (1.65 → 2.70)
F0 = 15500 · exp(0.059·N)          [N/mm^p]   (16440 → 2787300, factor of 170 bass→treble)
```

**Two-time-scale structure, and this is THE trap**: `δ = 1 − ε` (0.0029 to 0.0105) is the ratio of the static and instantaneous slopes. Therefore:
- quasi-static stiffness = `F0·δ`
- instantaneous stiffness = `F0`, i.e. **100 to 350 times higher**, but acting only over `τ0 ≈ 2 µs`

If you integrate the hereditary integral naively, you see `F0` and you need `dt << 2 µs`. **Remedy**: convert the exponential kernel into an auxiliary state variable (1st-order linear ODE forced by `u^p`) and integrate it EXACTLY with a one-pole filter `z ← z·exp(−dt/τ0) + (1 − exp(−dt/τ0))·u^p`. Unconditionally stable, 2 flops, and the apparent stiffness at your `dt` scale goes back to `F0·δ`.

Static branch, computations at v = 5 m/s (hand derivation, `[E]`, not
script-reproduced — see §0):

| note | N | m0 (g) | p | K_stat (N/mm^p) | u at 1 m/s | u at 5 m/s | k at 5 m/s | f (Hz) | T/2 (ms) |
|---|---|---|---|---|---|---|---|---|---|
| A0 | 1 | 11.00 | 3.72 | 172.8 | 0.669 mm | 1.324 mm | 1.38e6 N/m | 1779 | 0.281 |
| C2 | 28 | 9.08 | 4.12 | 658.0 | 0.521 | 0.976 | 2.51e6 | 2648 | 0.189 |
| C4 | 40 | 8.27 | 4.30 | 1162 | 0.473 | 0.868 | 3.13e6 | 3096 | 0.162 |
| C8 | 88 | 5.34 | 5.02 | 7961 | 0.357 | 0.609 | 5.44e6 | 5080 | 0.098 |

**Acknowledged discrepancy**: my half-period (0.10 to 0.28 ms) is ~10x shorter than the measured contact durations (4 ms → <1 ms). The explanation is physical: this calculation assumes a rebound against a RIGID wall, whereas the string is compliant — the stiffness seen by the hammer is the series combination of felt+string, hence much softer, and the end of contact is driven by the return of the reflected wave. **Practical consequence**: for an ACTION simulator (not sound synthesis), the string only needs to be a finite impedance or a calibrated restitution law — which is what Hirschkorn does, tuning only the `D` of contact 12 (0.175) "to produce approximately the correct rebound speed for the hammer". And the rebound speed matters because it governs checking.

---

## 3. Solver choice: an honest comparison

### (a) Explicit adaptive-step (Dormand-Prince RK45/RKF45) + event detection — **RULED OUT**
Attractive on paper. In practice: 13 contacts × several engagements/disengagements per blow = several dozen events per keystroke, some of them in chattering (the jack bottoming out, the hammer bouncing in the back check). Root finding dominates the cost. And above all: **the variable step destroys determinism and the real-time cadence** (you cannot guarantee "n steps per frame"). For a measuring tool, no.

### (b) Semi-implicit / symplectic fixed-step with sub-steps on the contacts — **SELECTED as the performance mode**
Semi-implicit Euler (symplectic Euler) or fixed-step RK2, with the contact force treated implicitly in velocity. `dt = 20-40 µs` as a baseline, **sub-steps at 5 µs during let-off and at 2-5 µs during impact**. Advantages: trivial to write, trivial to parallelize, trivial to make deterministic, and it is what gives the best felt feel since the compliance is native. Cost: 125 Mflop/s/note at 20 µs (see §5). **This is the right first cut and the right "88 animated keys" mode.**

### (c) Stiff implicit (BDF / Radau IIA / Rosenbrock) with analytical or autodiff Jacobian — **RULED OUT**
This is exactly what Hirschkorn did: MATLAB `ODE15s` (BDF), absolute tolerance 1e-5 rad, and the result is documented: **"A full simulation for the system with a single force profile input requires about 6 minutes on a 2.4 GHz PC"** for ONE note. He notes that stiff solvers are "much more efficient for this system" — which is true at a given tolerance — but also that "even more significant in contributing stiffness is the friction model". In other words: **it is the regularization of the friction that makes the system stiff**, not the contacts. Solving a stiffness you manufactured yourself is absurd. And the variable step remains incompatible with a fixed frame cadence and bit-exact reproducibility.

Comparative history of computation times in the literature (Thorin, Table 1 — very instructive):

| Work | Computation time | Real-time factor |
|---|---|---|
| Van den Berghe et al. | 1 s simulated = ~4 h on a 486DX33 | ~14,400x |
| Van den Berghe, reduced model | 1 s simulated = ~15 s on a DEC 5000/33 | 15x |
| Hirschkorn 2004 (MATLAB ODE15s) | ~6 min for one blow | ~700x |
| Izadbakhsh (Maple, flexible shank) | ~75 min | ~15,000x |
| Masoudi & Birkett | 910 s rigid / 1720 s flexible | — |
| **Thorin et al. 2017 (non-smooth, XDE 3D)** | **20x slower than real time, 2010 i5 2.1 GHz** | **20x** |
| **Timmermans/Fisette, haptic key** | **model computed "in real time, i.e. in less than 1 ms"** on an embedded controller | **< 1x** |

The trajectory is unambiguous, and the last entry is **the existence proof**: a multibody grand piano action already runs in real time at ≥1 kHz on embedded hardware, to drive a force-feedback haptic key (voice-coil actuator, force peaks up to 50 N in under 20 ms, refresh rate ≥1 kHz required for feedback stability).

### (d) NON-SMOOTH dynamics (Moreau-Jean, LCP/friction cone) — **SELECTED as the main strategy**

Thorin's exact scheme (read in the paper):
```
M(x_{n+θ}(k_x)) k_v  =  F_H(x_{n+θ}(k_x), v_{n+θ}(k_v), t_{n+θ})  +  H(x_{n+γ}(k_x)) r
v_{n+θ}(k_v) = k_x
( g(x_{n+γ}(k_x)) ,  Hᵀ(x_{n+γ}(k_x)) v_{n+γ}(k_v) ,  r )  ∈  K
```
with `x_{n+α}(k_x) = x_n + α·Δt·k_x`, `v_{n+α}(k_v) = v_n + α·Δt·k_v`, **θ = 1** (stability of the smooth dynamics) and **γ = 1** (constraints satisfied at the end of each step). We eliminate `k_x`, we linearize the contact kinematics `H` and the smooth dynamics around the previous iterate, Newton loop → One-Step Non-Smooth Problem in `r` alone, reformulated as an **augmented Lagrangian** and solved by **projected Gauss-Seidel**.

**Time-step gain: 25 to 100x** (500 µs validated vs 5-20 µs). **Announced time gain: two orders of magnitude.**

**Loss in felt feel: NONE, because Thorin does not stiffen the felt.** This is the classic misunderstanding about non-smooth methods. Their arrangement, which should be copied:
> a small auxiliary rigid body "A" of **1 mg**, a prismatic joint "P", and a visco-elastic element "V" (non-linear Kelvin-Voigt) inserted between the felt support and the contacting body "B". A unilateral constraint prevents A from penetrating B. An **inelastic** impact law on A to allow permanent-contact phases.

Felt law: `F_felt(δ) = k·δ^r + b·δ²·δ̇` (3 parameters per felt, identified by minimizing a cost functional over a series of dynamic measurements from piano to forte). The authors insist: "numerical methods adopted here allow for an efficient simulation for **any stiffness and even rigid contacts**, opening doors to efficient sensitivity analyses" — that is to say, exactly what you want for a regulation tool.

**THE DECISIVE ARGUMENT FOR YOUR TRADE.** Thorin writes verbatim:
> "A basic adjustment of the key mechanism consists in measuring the maximum mass to be put on the key without initiating the down-motion from the resting position and conversely, the minimum mass that prevents the up-motion from the down-position. The result directly depends on the dry friction coefficient in the hinges. However, **this simple test cannot be simulated with a regularized friction law**."

That is downweight / upweight. With regularized friction, `μ → 0` as velocity → 0, the sticking phase disappears, and both D and U converge towards BW: the 30 g gap — your daily measurement — vanishes. The other grievances Thorin lists against regularization, all of them relevant here: introduction of at least one non-physical parameter (which weakens any sensitivity analysis); spurious oscillations if the regularized zone is too narrow, wrong results if it is too wide; reduction of the time step to eliminate those oscillations; **degradation of the conditioning of the matrices as the regularized slope increases**.

### (e) Hybrid: non-smooth for the hard stops + penalty for the felt — **THIS IS WHAT THORIN ALREADY DOES**
There is no choice of (d) vs (e): (d) **is** (e). The 1 mg auxiliary-body arrangement realizes exactly the "rigid non-penetration (non-smooth) + felt compliance (smooth)" decomposition. And the dry friction stays set-valued. That is the right answer, and it is already published and experimentally validated.

### Verdict and swappability

| | Main | Performance |
|---|---|---|
| Scheme | Moreau-Jean, θ=γ=1, augmented Lagrangian + projected Gauss-Seidel | Semi-implicit Euler / fixed-step RK2 |
| dt | **250 µs** (Thorin validates 500) | **20-40 µs**, sub-steps 5 µs at let-off, 2-5 µs at impact |
| Friction | set-valued, Coulomb cone | regularized tanh (with exposed `v_t` threshold) |
| Felt | non-linear Kelvin-Voigt + unilateral constraint | **clamped** Hunt-Crossley |
| Use | 1-key bench, regulation measurements, DW/UW | animated 88-note keyboard |
| Cost/note | ~90 Mflop/s | ~125 Mflop/s |

The third solver, the **quasi-static** one, is often forgotten: measuring a DW/UW by time integration is absurd and noisy. What is needed is a **continuation on `θ_k`** (angle increment, solving the static equilibrium with an LCP on the friction cones at each step) which directly yields the ascending and descending force-displacement curves, hence D, U, BW and F. It is a different solver, not a mode of the previous one.

**How the architecture makes this swappable** — this is the real clean-architecture point:
- **Pure domain** (no dependency): analytical 2D contact geometry (3 primitives), material laws, kinematic tree, regulation observables.
- **Port** `Solver`: `step(&State, &Params, dt) -> State` plus a `Contact` port exposing `{ gap(q), dgap_dq(q), normal(q), tangent(q), contact_point(q), relative_tangential_velocity(q, q̇) }`.
- **Adapters**: `NonSmoothSolver`, `PenaltySolver`, `QuasiStaticSolver`.

Both dynamic solvers consume **exactly the same contact geometry and the same felt laws**: they differ only in how they resolve the unilaterality. That is what makes the boundary natural and not arbitrary. Corollary: the same scenario played through both solvers becomes a free and very powerful **cross non-regression test**.

---

## 4. Regularization and stability: the concrete pitfalls

### 4.1 PITFALL #1 — Hunt-Crossley produces an ATTRACTIVE normal force
`f_n = f_fit(x)·(1 + D·ẋ)` becomes **negative** (sticky) as soon as `ẋ < −1/D`. With Hirschkorn's measured D values:

| # | contact | D (s/m) | critical ẋ |
|---|---|---|---|
| 13 | shank/rest rail | 8.964 | **−0.112 m/s** |
| 6 | wippen/lever stop | 7.732 | −0.129 |
| 5 | wippen/jack rest | 5.781 | −0.173 |
| 11 | rep. lever/drop screw | 4.985 | −0.201 |
| **7** | **jack/knuckle** | **4.694** | **−0.213** |
| 10 | rep. lever/knuckle | 2.935 | −0.341 |
| 4 | back check/hammer tail | 2.446 | −0.409 |
| 3 | key heel/cloth | 2.502 | −0.400 |
| 1 | capstan/wippen heel | 1.877 | −0.533 |
| 8 | jack toe/button | 1.527 | −0.655 |
| 2 | key front/punching | 1.219 | −0.820 |
| 9 | jack/window | 0.779 | −1.284 |
| 12 | hammer felt/string | 0.175 | −5.714 |

Real separation velocities are on the order of 0.2 to 1 m/s (hammer tail leaving the back check, jack releasing from the knuckle, hammer leaving the rest rail). **11 contacts out of 13 can produce an attractive force.** Mandatory remedy: `f_n = max(0, f_fit(x)·(1 + D·ẋ))`. Same flaw in Lankarani-Nikravesh.

### 4.2 PITFALL #2 — Real felt is velocity-independent; all the practical models are not
Hirschkorn's qualitative observations from his felt-compression tests, to be read as they stand:
> **Non-linear spring**: "It also does not follow any simple power function."
> **Hysteretic**: "The felt requires more force to compress than it returns on decompression."
> **Velocity-independent**: "Whether the felt is loaded slowly or quickly seems to have little effect on the loading profile. Instead, it is dependent on the **maximum and minimum loads the felt has previously undergone**."

He explicitly acknowledges the weakness of his own model: "The biggest weakness of this model is that the hysteretic damping behaviour is speed dependent, and it is known from previous research (Stamm) that the behaviour of real felt is not." His pragmatic workaround: choose D so that the error is zero at the AVERAGE velocity observed at the contact (`D = [f_loading(x_max)/f_fit(x_max) − 1]/ẋ_ave`, with `ẋ_ave` measured on a real forte blow using encoders + high-speed camera, the out-of-contact instants filtered out).

Options and judgement:
- **Hunt-Crossley / Lankarani-Nikravesh**: velocity-dependent. Simple. To be calibrated as Hirschkorn does. **OK for the performance mode.**
- **Gonthier et al.**: rejected by Hirschkorn for a structural reason worth remembering — "one of the key parameters, the damping factor, is determined from the **initial contact velocity**. This means the model can only be used in situations where the contact bodies are initially apart before making contact and rebounding. However, in the piano action, bodies can **start in contact, and might stay in contact throughout the entire simulation**." Fatal here: the capstan touches the wippen heel permanently.
- **Dunlop's model (multi-spring/friction)**: rejected by Hirschkorn for three reasons — wrong form (it gives a displacement for a known force, whereas we need the converse), no calibration method, and Stamm casts doubt on its ability to reproduce Dunlop's own results.
- **Bouc-Wen**: expresses rate-independent hysteresis via an internal state variable. Mathematically the right framework. But **I found NO published Bouc-Wen identification for piano felt**: one would have to identify 5-7 parameters per contact with no reference data. To be kept in R&D.
- **Interpolated loading/unloading curves with a memory variable** (a pair of curves + a "max load / min unload reached" state): this is the literal description of what Hirschkorn OBSERVES, and it is rate-independent by construction. It is probably the right answer in the long run, but **not published for the piano**.
- **Stulov hereditary**: rate-dependent, but with an exploitable two-scale structure (§2.4). The only model parameterized over all 88 notes.

### 4.3 PITFALL #3 — C1 continuity at contact entry
`f_fit(x) = ax³ + bx² + cx` has `f_fit(0) = 0` and `f'_fit(0) = c > 0`: the force is C0 but **the stiffness jumps from 0 to c** (7.9 to 32.3 kN/m depending on the contact). This is an acceleration discontinuity at every contact entry, and with 13 contacts engaging/disengaging several times per blow, it pollutes any error-controlled integrator. Remedies: (i) a C1 ramp over the first 10-50 µm (`f ← f·s(x/x_ramp)` with `s` a smoothstep); (ii) a damping term proportional to `x` rather than constant (`b·x²·ẋ` as in Thorin — which does vanish at `x = 0`, unlike `D·ẋ`); (iii) non-smooth, where the question does not arise.

Note that **Thorin chose `b·δ²·δ̇`** and not `D·δ̇`: his damping vanishes at δ=0, which settles pitfalls 1 and 3 in one stroke. It is a better form than Hunt-Crossley and I recommend adopting it.

### 4.4 PITFALL #4 — Chattering
The identified risk zones: (a) the hammer tail in the back check, which bounces and where tangential friction is, according to Thorin, "crucial for stopping the motion of the hammer after hitting the string"; (b) the jack bottoming out on the let-off button; (c) the hammer settling onto the repetition lever after drop. Remedies: inelastic impact law (Thorin, on the auxiliary bodies), force saturation, and above all **robust lift-off detection**: declare separation only if `g > 0` AND `ġ > ε` over several consecutive steps (hysteresis on the contact state), never on the sign of `g` alone.

### 4.5 PITFALL #5 — Conditioning
Thorin: "the condition number of the matrices involved increases as the slope of the regularized function increases". With reflected mass ratios of 570:1 (§2.2) and stiffnesses from 2.6e3 to 4.4e5 N/m on the same action, the Delassus matrix is ill-conditioned. Consequence: Gauss-Seidel MUST be preconditioned (diagonal scaling), and a **fixed** iteration count is required (not a convergence criterion) for determinism — with a residual diagnostic reported to the UI so we know when we have not converged.

### 4.6 PITFALL #6 — Tangential friction is not everywhere
Hirschkorn applies tangential friction on only **4 contacts** (1, 4, 7, 10); Thorin on **3** (jack/knuckle, back check/hammer, jack/let-off button), on the grounds that "friction is ignored in the other contacts which are subject to a rather small translational motion". Do not over-model: each frictional contact adds an unknown to the cone and extra Gauss-Seidel iterations. But **the jack/knuckle friction must be there** — Thorin: "easily felt by the pianist and must also be considered in the model, because the piano technicians adjust it carefully".

---

## 5. Quantified compute budget

### 5.1 Flops per step, per note

**Penalty, ~2500 flops/step** (detailed estimate):
- 6 sin/cos (body orientations): ~100
- geometry of 13 contacts (circle/line or circle/circle, gap + normal + point + lever arm): 13 × ~60 = 780
- force laws (cubic 4 flops + damping 3 + rational friction ~30): 13 × ~25 = 325
- assembly of the generalized forces: 13 × ~10 = 130
- arrowhead M(q) + closed-form inversion: ~100
- Coriolis terms of the subtree: ~100
- integration (semi-implicit, 6 DOF): ~60
- margin/branching: ~900

**Non-smooth, ~45,000 flops/step** (estimate):
- Newton loop 2-3 iterations × linearization of M, H, F_H: ~1500
- Delassus matrix `W = H M⁻¹ Hᵀ` (13×13 via 6 DOF): ~2000
- projected Gauss-Seidel: 30-100 sweeps × 13 contacts × ~50 flops = 20,000 to 65,000

**Cross-validation of this estimate**: Thorin runs at 20x real time with dt = 500 µs, i.e. 2000 steps/s, so **10 ms of CPU per step** on a 2010 core at ~2 Gflop/s ≈ **2e7 flops/step**. My estimate of 4.5e4 is 450x lower. That gap is precisely what the authors themselves identify as their room for improvement: "future versions ... include a **more efficient implementation of the nonlinear springs laws** and perform **direct 2D dynamics rather than 3D**. Presumably, the resulting computing efficiency will be **more than sufficient for a real-time computation** of the piano action dynamics on a standard laptop." A general-purpose 3D engine (XDE) pays for it: 6 DOF per body instead of 1, mesh-based collision detection instead of 3 analytical primitives, a PD controller for position-driven simulations, and re-adjustment of the linear-spring coefficients at each step to emulate the non-linear law. **A factor of 450 of slack: that is credible, and that is our headroom.**

### 5.2 Mflop/s per note and aggregate Gflop/s

| Strategy | steps/s | flop/step | **Mflop/s/note** | 1 note | 12 active | 24 active | 88 active |
|---|---|---|---|---|---|---|---|
| penalty, dt=10 µs | 100,000 | 2500 | 250 | 0.25 Gf/s | 3.0 | 6.0 | **22.0** |
| penalty, dt=20 µs | 50,000 | 2500 | 125 | 0.13 | 1.5 | 3.0 | 11.0 |
| penalty, dt=40 µs + sub-steps | 25,000 | 2500 | 62.5 | 0.06 | 0.75 | 1.5 | 5.5 |
| **non-smooth, dt=500 µs** (Thorin-validated) | 2,000 | 45,000 | **90** | 0.09 | 1.08 | 2.16 | 7.92 |
| non-smooth, dt=250 µs (x2 margin) | 4,000 | 45,000 | 180 | 0.18 | 2.16 | 4.32 | 15.84 |

### 5.3 How many notes are REALLY active?

The mechanical activity window of a note ≈ **300 ms** (attack 150 ms, release + jack re-engagement + damper return 150 ms). During the hold, the action is **static** (hammer in the back check, key at the bottom, jack escaped): freezable.

| density | context | concurrent notes |
|---|---|---|
| 6 onsets/s | Mozart allegro | 1.8 |
| 12 onsets/s | Chopin étude | 3.6 |
| 20 onsets/s | Liszt / Alkan | 6.0 |
| 32 onsets/s | burst, 8-note chords × 4/s | 9.6 |

**Sizing for 24 gives 2.5x of margin over the realistic worst case.**

The 88 dampers: they only move together during a damper-pedal transition (~100-200 ms), and each has only 1 DOF. The coupling through the damper lift tray is an 88-to-1 unilateral contact → a Delassus matrix of **rank 1 + diagonal** → solved in O(88), not O(88³). Cost ≈ 1/6 of a full note each, i.e. ~15 note-equivalents **during the transition only**. Sostenuto is a state latch (a subset of levers held): zero cost. The una corda is a rigid lateral translation of the whole action: **1 single global DOF**, no additional planar physics (it is the only out-of-plane motion, and it is trivial).

### 5.4 ms per 16.7 ms frame

Assumption: **3 effective Gflop/s per worker thread** (see §5.5).

| Strategy | n active | 1 thread | 4 threads |
|---|---|---|---|
| non-smooth, dt=500 µs | 1 | **0.50 ms** | 0.13 ms |
| non-smooth, dt=500 µs | 12 | 6.01 ms | **1.50 ms** |
| non-smooth, dt=500 µs | 24 | 12.02 ms | **3.01 ms** |
| non-smooth, dt=500 µs | 88 | 44.09 ms ✗ | 11.02 ms |
| non-smooth, dt=250 µs | 24 | 24.05 ms ✗ | **6.01 ms** |
| penalty, dt=20 µs | 24 | 16.70 ms ✗ | **4.17 ms** |
| penalty, dt=40 µs + sub-steps | 24 | 8.35 ms | **2.09 ms** |
| penalty, dt=10 µs | 88 | 122.5 ms ✗✗ | 30.6 ms ✗ |

**VERDICT**:
- **1-key bench**: 0.5 to 1.4 ms per frame, **whatever the solver**. Entirely comfortable. We can even afford dt = 5 µs in penalty for a "reference" mode.
- **88-note keyboard, realistic case (6-10 active)**: 1 to 2 ms on 4 threads. **That leaves 14 ms for the 3D rendering.** Comfortable.
- **88-note keyboard, sized worst case (24 active)**: 2 to 6 ms on 4 threads. **Still 11 to 14 ms for the rendering.** OK.
- **All 88 notes active** (musically impossible, but useful as a ceiling): 11 to 22 ms on 4 threads. That does not hold at 60 fps, and it does not matter: that situation does not exist. We still need **a graceful degradation mechanism** (switching notes beyond a budget to a coarser `dt`, or to the penalty solver) rather than dropping frames.
- **Penalty at dt = 10 µs over 88 notes: 22 Gflop/s. Infeasible in a browser.** It is the only scenario that breaks, and it is exactly the one we would pick out of naivety.

### 5.5 Is WASM NECESSARY? Numbers, not intuition

Reference throughputs on one 2026 desktop core (f64) — **my estimates, not measurements**:
- scalar C/Rust: ~2-4 Gflop/s
- native + 128-bit SIMD (f64x2): ~4-8 Gflop/s
- baseline WASM (v128, f64x2): ~0.8-0.95x native → **~3.5-7 Gflop/s**
- JS/TS, monomorphic loops over TypedArray, **no SIMD in JS**: ~0.5-1.0x scalar native → **~1-4 Gflop/s**

The actual state of the benchmark literature, which I checked and which is **thin and contradictory**: a 2019 study finds that "JS is within a factor of 2 compared to C for numerical computations"; a July 2025 study finds WASM superior on integer and recursive CPU-bound tasks but the JS JIT "competitive in some floating-point work"; the blog posts claiming 8-10x (and 10-15x with SIMD) are single-machine and not reproducible. **I do not base the decision on those figures.**

Decision driven by the need, then:

| Target | Aggregate need | JS/TS 4 threads (~6 Gflop/s) | WASM 4 threads (~14-28 Gflop/s) |
|---|---|---|---|
| 1 key | 0.09-0.25 Gflop/s | **amply sufficient** | overkill |
| 88 notes, 10 active | 0.9-1.25 | sufficient | comfortable |
| 88 notes, 24 active | 2.2-6.0 | **marginal (~1-3x margin)** | **comfortable (3-10x margin)** |
| 88 notes, 88 ceiling | 8-22 | **insufficient** | **marginal to OK** |

**So on the throughput criterion alone: TypeScript would suffice for the 1-key bench, and would be marginal for the full keyboard.** That is not a strong enough argument to impose Rust.

**The real arguments for WASM are elsewhere, and they are decisive:**

1. **Determinism of the transcendentals.** `Math.pow` and `Math.exp` are "implementation-approximated" in ECMA-262: the spec recommends fdlibm but does not mandate it, and both V8 and SpiderMonkey use forks of fdlibm that have diverged. One reported measurement (20,000 seeded inputs) gives **9.8 % divergence on `pow` and 9.5 % on `exp` between JavaScriptCore and V8** — and **0 % on `sqrt`** (which the spec mandates as exact). For a regulation measuring tool, that is disqualifying. Practical corollary, independent of the language: **avoid `pow` in the hot loop**, which disqualifies Stulov's `F0·u^p` form with non-integer p (3.72 to 4.98) and argues for Hirschkorn's cubic polynomial form, which uses only multiplications.

2. **Absence of GC, structurally.** In JS you can avoid the GC (TypedArray + pooling) but it is fragile: one single accidental allocation in the stepping loop and you have an unpredictable pause. In Rust, "zero allocation in `step()`" is verifiable at compile time and by review convention.

3. **Explicit SIMD.** JS has no SIMD (SIMD.js is dead). WASM v128 does — and it is relevant here because the notes are independent: we can do **"inter-note" SIMD** (processing 2 notes as f64x2, or 4 as f32x4), which is the ideal arrangement (no shuffles, no horizontal reductions, branches replaced by masked selects).

4. **Control of the time step and of the operation order.** Trivial in both, but the discipline is easier to maintain in a strictly typed language with no implicit coercions.

**Final recommendation**: **Rust → WASM for the domain core**, even for the 1-key bench, because writing the solver twice (TS then Rust) would cost more than doing it well once, and because the 1-key bench is precisely the mode where measurement accuracy matters most.

### 5.6 Deployment constraints not to discover too late
- **Baseline SIMD** (v128): Chrome 91+, Firefox 89+, Safari 16.4+, Edge 91+. Available everywhere today.
- **relaxed SIMD**: Chrome 114+, Firefox 145/146+, **NOT Safari** (caniuse lists 26.3 and Technology Preview as unsupported). And in any case **to be banned** (§6).
- **Threads** = `SharedArrayBuffer` = **mandatory COOP/COEP headers**, and `crossOriginIsolated` must be checked at runtime. To be wired up from the dev server onwards, otherwise multithreading is lost in production without anyone noticing locally.

---

## 6. Determinism and reproducibility

**Why it matters here, concretely**: you are going to regulate a let-off in the simulator, measure 1.2 mm, change a parameter, go back, and expect to find 1.2 mm again — not 1.19 or 1.21. And a recorded session must replay identically on a colleague's machine. Without that, the tool is not a measuring instrument, it is an animation.

Checklist:

1. **FIXED time step.** Non-negotiable. No `dt` derived from `requestAnimationFrame`. Fixed-step accumulator, interpolated rendering. That also rules out every adaptive-step integrator.
2. **FIXED iteration count** in Gauss-Seidel / Newton. No `while (resid > tol)` criterion, which makes the result dependent on rounding. Fixed iterations + residual reported as a diagnostic.
3. **Frozen operation order.** Contact iteration order fixed by construction, not by insertion order in a container. Reductions (sum of torques) in a fixed order, never in parallel without a deterministic reduction tree. In Rust, beware of iterating over `HashMap` (randomized order): use `Vec` or `IndexMap`.
4. **Strict IEEE-754 arithmetic.** WASM guarantees IEEE-754 round-to-nearest-even semantics for `f32`/`f64` (no extended-precision x87, no FTZ). **Forbid `-ffast-math` and every equivalent.** In Rust, forbid the `fast_*` intrinsics.
5. **NEVER relaxed SIMD.** Verified: `qfma` does **one** rounding on FMA hardware and **two** on hardware without FMA. The non-determinism is bounded to the instruction and stable on a given machine (it is in fact a fingerprinting vector flagged by Mozilla), but **it varies across hardware**. The proposal's "deterministic profile" canonicalizes NaNs but **does not pin down FMA**; an issue proposing guaranteed exact FMA instructions (`relaxed-simd` #44) is ~5 years old and I could not verify that it had been merged. So: **baseline v128 only, never `-mrelaxed-simd`.**
6. **No platform transcendentals in the hot loop.** Neither `Math.pow`/`Math.exp` in JS, nor `f64::powf` backed by the system libm. If you need them: compile your own `libm` (the `libm` crate in Rust, bit-exact and portable) or tabulate. `sqrt` is safe (exact by spec, on both sides).
7. **No GC.** Zero allocation in `step()`.
8. **Explicit seeds.** No implicit RNG. If you add variability (note-to-note regulation scatter, play in the mortises), it is an explicit PRNG, seeded, serialized with the scene.
9. **Serialization of the complete state**, including the hidden variables: felt hysteresis memory variables, stick/slip states of each frictional contact, contact flags. A "position + velocity" state is NOT enough to replay. That is the classic mistake.
10. **Multithreading without non-determinism**: the 88 notes are quasi-independent → fixed static partitioning of the notes per worker (no work-stealing), a barrier per step, and the damper coupling resolved **at a single point** (a single worker, or on the synchronization thread) to avoid any variable reduction order.

A regression test to put in place from day 1: a **hash** (SHA-256) of the complete state after N steps, on a canonical scenario, compared in CI across Chrome, Firefox and Safari. If the hash diverges, we know about it before it poisons a measurement.

---

## 7. Key excitation

### 7.1 Three modes

**(i) FORCE-driven — realistic playing.** Sourced values (Askenfelt & Jansson 1992b, reported by Goebl et al. 2005):
- **mezzo forte staccato** attack: **~15 N**
- **very loud** attacks: **peaks up to 50 N (fortissimo)**
- **piano** touch: **~8 N**
- **legato** touch: **about 1/3 of staccato**, typically with a peak when the key reaches the keybed
- **pianissimo: "the force hardly exceeds 0.5 N"**

Profile shape: rapid rise then plateau/decay; for struck touch, an initial finger-percussion spike. Goebl notes that "in very loud staccato tones, the first impact of the finger hitting the key resulted in a very high-peak hammer velocity which decreases significantly until hammer-string contact" — the maximum hammer velocity is NOT the strike velocity, and that matters for calibration.

**(ii) IMPOSED-DISPLACEMENT — regulation measurements.** This is Thorin's central validation argument: position-driven simulations "prove far more reliable than the usual force-driven simulations", because a force input hides the model's defects behind the intrinsic smoothness of the resulting motion. Put another way: **if your model is bad and you excite it in force, you will not see it; if you excite it in position and compare the measured force, you see it immediately.** An implementation detail to anticipate: XDE only did force-driven and Thorin had to insert a PD controller (and its experimental equivalent, a viscoelastic piston). **In our in-house solver, position-driven must be NATIVE**: a bilateral constraint on `θ_k` (or an additional reaction unknown in the OSNSP). It is a few lines, but it has to be provided for in the `Solver` port interface from the very beginning.

**(iii) QUASI-STATIC — downweight/upweight.** A dedicated continuation solver (§3). Stanwood protocol to reproduce: **D** = "minimum weight, to the nearest gram, that makes the key drop with slow, controlled hammer motion"; **U** = "maximum weight, to the nearest gram, that the key can lift with slow, controlled hammer motion".

### 7.2 MIDI velocity 0-127 mapping

**Measured** anchors:

| Point | Hammer velocity | Source |
|---|---|---|
| MIDI 77 | **2 m/s** | Goebl & Bresin 2003 |
| forte | **~5 m/s** (FHV), key-to-keybed travel in **25 ms** | Askenfelt & Jansson 1991, p. 2385 |
| piano | **~1 m/s** (FHV), travel in **160 ms** | ibid. |
| maxima actually played | **C1: 6.4 m/s; G6: 7.8 m/s** | Goebl et al. 2005 |
| lower threshold | ~0.2-0.3 m/s (below that, the hammer no longer reaches the string) | inferred from free flight |
| velocity at let-off | **piano 1.01 m/s; forte 1.56 m/s** (measured AND simulated) | Thorin, Table 5 |

A useful operational definition: the Bösendorfer SE measures the average velocity **over the last 5 mm** of hammer travel ("approximately the last 10 % of that distance" → total travel ≈ 50 mm, consistent with a blow distance of 45-47 mm) and stores it inverted: `FHV [m/s] = 128 / IHV` at 25.6 kHz. Check: 0.005 m / (IHV/25600 s) = 128/IHV. Perfect internal consistency — so it is a measurement definition usable as-is as a simulator observable.

**Why you must NOT impose a hammer velocity**: the hammer velocity is the RESULT that regulation is supposed to influence. If you impose it, changing the let-off or the blow distance changes nothing in the sound — and you lose all of the pedagogical value.

**Recommended procedure**: impose a parameterized force profile (amplitude × shape), and build **offline, by bisection**, an **88 × 128** table `(note, velocity) → force amplitude`, recomputed on every regulation change. Cost: 11,264 simulations of ~0.3 s = ~3,400 s simulated; at 10x real time on 4 threads, **~85 s of recomputation**. Acceptable for a background operation, and the side effect is precious: **a badly regulated let-off becomes audible and visible**, since the table changes.

On the analytical form: no simple function passes through the three anchors (1; 0.2), (77; 2.0), (127; 5.5). An exponential fitted on the first two gives 9.1 m/s at 127 (too high); a power law `v = 5.5·(m/127)^2.02` fitted on the last two gives 0.00034 m/s at m=1 (absurd). Hence the **interpolated monotone table** (PCHIP on `log v` vs `m`) rather than a formula — and in any case the model-calibrated table is better.

### 7.3 Timing checks to reproduce (quantified validation targets)
- Key-to-keybed travel: **25 ms** at forte (5 m/s) → **160 ms** at piano (1 m/s)
- **Hammer-string contact 12 ms BEFORE the keybed at piano, 3 ms AFTER the keybed at forte** — a superb test, because it changes SIGN and because it brings aftertouch into play
- **Free flight** (let-off → string): **~0 ms** at loud dynamics → **20 ms** at very soft ones, with outliers at 40 ms
- Travel time (finger→string): **20 ms to ~200 ms**, power-law fits `tt_pressed = 89.96·HV^(−0.5595)` and `tt_struck = 58.39·HV^(−0.7377)` (ms, HV in m/s)

---

## 8. Quantified validation plan

### Level 0 — Analytical tests (unit tests, exact)
- **Leverage ratio with simplified geometry**: replace the profiles with rigid arms of known lengths, verify that the numerically computed `∂g/∂θ` = the analytical value to within 1e-12.
- **Contact primitives**: circle/line, circle/circle, hybrid — verify `g`, `n`, `x_c`, `∂g/∂θ` against hand-solved configurations. Verify the **C1 continuity** of the normal at the arc→line transition of the hybrid profile (finite difference of `n` on either side).
- **Mass matrix**: verify `M_jr = 0` exactly; verify symmetry; verify `M` positive definite over the whole configuration domain; compare the arrowhead inversion against a dense LU to 1e-14.
- **Virtual work**: verify that the static FORCE ratio = the DISPLACEMENT ratio (a Jacobian consistency test; it catches 90 % of sign and lever-arm errors).

### Level 1 — Time-step convergence
Reproduce Thorin's Figure 12: simulate the same scenario at **0.4 / 2 / 10 kHz** and plot the force on the key. **Expected**: no significant difference between 2 and 10 kHz with the non-smooth solver. If a difference shows up, it is a bug (typically: a hidden regularization parameter, or a variable convergence criterion).
The same study in penalty mode: expected, convergence towards 5-10 µs, visible degradation beyond 20-40 µs, instability beyond ~80 µs (= `2/ω` of the jack/knuckle contact).

### Level 2 — Energy conservation in the free phase
Disable all contacts and all friction, release the hammer from a given angular position: verify that `E = ½ I_h θ̇² + m_h g h` is constant. Expected drift: **zero at first order** for a symplectic integrator, bounded and oscillatory (not monotonic) — if the drift is monotonic, the integrator is not symplectic. Same test on the wippen+jack+lever (which checks the Coriolis terms of the subtree).

### Level 3 — Comparison with published trajectories
- **Velocity at let-off**, Thorin Table 5: piano **1.01 m/s** measured / **1.00** simulated; forte **1.56** / **1.56**. Reproduce to better than 5 %.
- Angular trajectories of the 5 bodies: Thorin Fig. 10 (KLT tracking on a high-speed camera), Hirschkorn ch. 4 (encoders on key/wippen/hammer + high-speed video on jack/lever).
- **Goebl's timing relations** (§7.3), in particular the **sign change** of the string-contact / keybed offset between piano and forte.
- **Hammer-string contact duration**: 4 ms in the bass → <1 ms in the treble, ±20 % over p→ff.
- Measured forces on the key in position-driven mode (Thorin: "The comparisons between measured and simulated forces in response to a given motion are successful"). This is THE discriminating test.

### Level 4 — The simulator must measure ITS OWN regulation
This is the level that counts for you, and it is the one that exists in no publication. The simulator must implement the **technician's measurement procedures**, not ad hoc numerical probes. Hirschkorn's protocol (identical on the real action and on the model, which is exactly the right spirit):

| Step | Spec in Hirschkorn | Trade spec (published ranges) |
|---|---|---|
| 1. Key dip | key front travel **10.0 mm** | Steinway **.400"** factory, range **.390-.420"**; Hamburg **9.9-10.6 mm** |
| 2. Hammer rest height (set at the capstan) | **48.0 mm** | blow distance **47 mm** (Hamburg), **1¾"** Steinway S/M/L/O/A/B, **1⅞"** C/D; general range **42-47 mm**; BYU **45 mm (±2)** |
| 3. Jack rest | right edge of the jack aligned with the right edge of the knuckle | — |
| 4. Repetition lever rest | same height as the jack under the knuckle | — |
| 5. **Let-off** | the jack escapes from under the knuckle at **1.0 mm** of hammer height | **1/16" ≈ 1.5 mm**; Hamburg **1.5 mm bass / 1 mm mid-treble**; range **3 → 1.5 mm** bass→treble; BYU **1.5-2 mm** |
| 6. **Drop** | hammer at **3.0 mm** after jack release | often **let-off + 2.0 mm**; New York **1/16"** |
| 7. **Back check** | under a medium blow, hammer stopped at **15.0 mm** | **15 mm**, range **12.7-16 mm** |
| Aftertouch | — | Hamburg **0.8-1.2 mm**; New York **.04-.055"**; BYU **1.25-1.5 mm** |

And the **touchweight** measurements, which must be a first-class feature using Stanwood's vocabulary (*Touch Weight Metrology*):

```
Balance equation:        BW + FW = (WW × KR) + (SW × R)
BW  = (D + U)/2      Balance Weight
F   = (D − U)/2      Friction Weight
SW                   Strike Weight: weight of shank + hammer at the strike-line radius
WW                   Wippen Weight: weight of the wippen laid flat, at the capstan contact point
KR                   Key Weight Ratio: force at the capstan / force at the measuring point
R   = SBW/SW         Strike Weight Ratio
SBW = TBW − WBW      TBW = BW + FW ;  WBW = KR × WW
```

Stanwood reference values: **Strike Weight zone** note 1 = 8.0 / 10.3 / 12.4 / 14.8 g; note 45 = 6.5 / 8.3 / 10.1 / 12.0 g; note 88 = 3.5 / 4.6 / 5.8 / 6.9 g. **Shank Strike Weight** average **1.7 g**. **Strike Weight Ratio** normal zone **5.0-7.0** (low reduction 7.0-6.3 / medium 6.3-5.7 / high reduction 5.7-5.0). **Front Weight Ceiling**: note 1 = **41.3 g**, note 23 = 35.5, note 45 = **28.1**, note 67 = 17.4, note 88 = **3.7 g**. Perceptual sensitivity: **1.0 g more Strike Weight is clearly audible, 0.5 g is perceptible to an average amateur** — and widening the head by about **1.6 mm increases the Strike Weight by about 1.0 g (≈0.6 g/mm)**. Hammer hang **125 mm (bass, R = 5.5)** vs **130 mm (treble, R = 6.1)**; knuckle core **15.5 mm vs 17 mm** produce different ratio levels.

**A check on the convention**, reproduced by
`tools/derivations/09-action-ratio-and-balance-closure.py`. It settles which `KR`
convention the equation of balance takes; it does not validate the model.

`KR` has **two reciprocal conventions** and the equation of balance takes only
one. From Hirschkorn's geometry: the *geometric* ratio, front over capstan, is
`217.5/120.4 = 1.806` — Stanwood's "nominally 2:1". The *force* convention, which
is the one in `BW + FW = (WW × KR) + (SW × R)`, is its reciprocal, **0.554**. His
own measured example reads 0.57. Conflating them is a factor of 3.26 on the wippen
term, and it is the error the engine's level-0 tests refuse: `WBW < WW` and
`0.45 < KR < 0.62`, because a *reducing* lever cannot amplify.

With the force convention, and `SW = 11.74 g × 103.2/136.8 = 8.86 g` (in band at
note 52, where Stanwood's thirteen levels run 6.1 to 11.3 g):

| posited `BW` | `WBW = WW × KR` | `SBW = SW × R` | `FW` | Stanwood ceiling, note 52 |
|---|---|---|---|---|
| 35.0 g | 8.08 g | 51.7 g | **24.8 g** | 24.7 g |
| 38.0 g | 8.08 g | 51.7 g | 21.8 g | 24.7 g |

The two results are of unequal strength:

- **The action ratio is a genuinely closed test.** Nothing is posited, only
  geometry: the lever product gives **5.84 : 1**, inside every published band
  (6 mm gauge 5.3–5.9, Stanwood 5.0–7.0 with 5.5 recommended). Under the inverted
  convention the same geometry gives 19.06 : 1, so the test discriminates.
- **The front weight agreement is weaker, and is not a closure.** `BW` is
  *posited* at 35 g and the ceiling is a one-sided bound, so landing at 24.8 g
  against 24.7 g is a plausibility check, not a validation. A percentage agreement
  computed from a posited `BW` is arithmetic on a chosen input, not evidence.

Two traps when reading these bounds. They must be read **at the note the data
describes**: Hirschkorn's action is note 52, so note-1 figures — an 8.0–14.8 g
strike-weight zone, a 41.3 g front-weight ceiling — do not apply to it. And `WW`
here is **not a measurement of this action**: 14.6 g is the generic Grandwork
example, and recomputing it from Hirschkorn's Tables A.1 and A.4 on the
capstan-contact arm gives either 10.1 g or 15.0 g depending on which arm the
annexes mean. `WW` for this action is therefore **not established**, and 14.6 g
must not be quoted as measured on it.

The corpus supports a narrower claim than cross-validation, and a useful one:
Hirschkorn's parameter set, Stulov's felt model, the Steinway/Renner regulation
specs and Stanwood's metrology are expressed in compatible quantities and do not
contradict each other where they overlap. **The test suite can be written before
the solver.**

### Level 5 — Sensitivity (the real product)
For each regulation parameter, plot the derivative of the observables. Examples of what the tool must be able to answer:
- capstan height `+0.1 mm` → what delta on the hammer height, the let-off, the aftertouch, D, U?
- let-off button `+0.1 mm` → delta on the let-off and on the velocity at let-off?
- key lead `+1 g` at `x` mm from the balance rail → delta on BW (and not on F)?
- repetition spring force `+10 %` → delta on U and on the re-engagement time?
- pin friction `×1.2` → delta on F = (D−U)/2 (and not on BW)?

The last two are **non-confusion** tests: a change of lead must move BW without touching F; a change of friction must move F without touching BW. If the model mixes the two, it is wrong — and it is a test a technician will detect immediately, where a developer would never see it.
