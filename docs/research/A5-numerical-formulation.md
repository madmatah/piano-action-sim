# A5. Numerical formulation of one note

Research annex to `docs/FOUNDATIONS.md`, which holds wherever the two disagree.
It sets out the planar equations of one note of a grand action, the contact
geometry, the felt and friction laws, the time step and the solver options, the
compute budget, determinism, the excitation of the key and the validation
levels, each against the literature in the corpus.

## Sources and markers

The primary sources are in `data/references/`, outside version control (third-party
copyright; [FOUNDATIONS > Sources](../FOUNDATIONS.md#sources)):

| Document | Local path | What this annex takes from it |
|---|---|---|
| Thorin, Boutillon, Lozada, Merlhiot, *Non-smooth dynamics for an efficient simulation of the grand piano action*, Meccanica 52:2837–2854, 2017 | `papers/thorin2017.pdf` | the Moreau–Jean scheme, Δt = 0.5 ms and the 2 against 10 kHz comparison, the felt law and its 1 mg auxiliary bodies, the case against regularized friction, the Bad Model |
| Thorin, *Non-smooth model of the grand piano action*, PhD thesis, École Polytechnique, 2013 | `papers/thorin_thesis_2013.pdf` | the parameter values (ch. 4), the time steps and computing times (§6.4.1), the virtual regulation (§6.1) |
| Lozada, *Modeling, haptic control and novel concepts for musical keyboards*, PhD thesis, École Polytechnique, 2007 | `papers/lozada_thesis.pdf` | the measurements behind Thorin's frictions, springs and felts (Tab. 2.3, ch. 3) |
| Hirschkorn, *Dynamic Model of a Piano Action Mechanism*, MASc thesis, University of Waterloo, 2004 (85 pp.) | `papers/hirschkorn_thesis.pdf` | one action measured: 31 geometric points (Tables A.2–A.7), masses and inertias (Table A.1), thirteen contact laws (Table A.8), five axis frictions (Table A.9), two springs (§A.4), the regulation procedure ([Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step)) |
| Stulov, *Experimental and computational studies of piano hammers*, Acta Acustica united with Acustica 91(6), 2005 | `papers/stulov_actaa2005.pdf` | the hereditary law of hammer felt, fitted across the 88 notes of one set of hammers |
| Goebl, Bresin, Galembo, *Touch and temporal behavior of grand piano actions*, JASA 118(2):1154–1165, 2005 | `papers/goebl2005.pdf` | finger forces, hammer velocities, travel times, free flight |
| Goebl & Bresin, *Measurement and reproduction accuracy of computer-controlled grand pianos*, JASA 114(4):2273–2283, 2003 | `papers/goebl2003.pdf` | MIDI 77 against 2 m/s, FHV = 128/IHV, the solenoids' ceilings |
| Stanwood, *Standard Protocols of the New Touchweight Metrology*, Piano Technicians Journal, February 2000, and the rest of his series | `stanwood/` | the weighing protocols and reference values of [Quantified validation plan](#quantified-validation-plan) |
| Steinway's technical reference and the regulation sheets | `manuels/` | the trade's regulation values of [Quantified validation plan](#quantified-validation-plan) |

Markers, as in FOUNDATIONS: **[V]** verified against a primary source in the
corpus, with its locator; **[E, n]** computed by script `n` in
`tools/derivations/`, which runs with `python3 -I`; **[S]** assumed, pending a
measurement or a source; **[?]** given by a source that is not in the corpus, with
the reason. A number whose arithmetic is written beside it carries no marker.

The scripts this annex draws on: 01 (each body's inertia about its pivot), 04 (the
sign of the elastic branches, and the separation speeds at which the damped laws
turn attractive), 05 (monotonicity of the elastic branches; Hirschkorn's knuckle
against Thorin's; the stability limit of a semi-implicit step), 06 (three annex
figures recomputed), 07 (regularized friction; additive against multiplicative
damping; the static friction peaks of the sliding contacts), 08 (the hammer-head
acceleration a force at the knuckle implies), 10 (Hirschkorn's action assembled
at rest: the rest pose, the mass matrix, the lever arms, the effective mass along
each contact normal and the highest linearized modes).

---

## Planar model of one note

### Bodies and generalized coordinates

Six bodies, **all in pure rotation** in the sagittal plane, with **relative
coordinates** for the jack and the repetition lever ([FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever), planned).

| Body | Coordinate | Pivot | Nature |
|---|---|---|---|
| Key | `θ_k` | balance rail pin, fixed to the frame | absolute |
| Wippen | `θ_w` | wippen center pin, fixed to the frame | absolute |
| Jack | `θ_j` | center pin **carried by the wippen** | **relative to θ_w** |
| Repetition lever | `θ_r` | center pin **carried by the wippen** | **relative to θ_w** |
| Hammer (shank + head) | `θ_h` | hammer-flange center pin, fixed to the frame | absolute |
| Damper under-lever | `θ_d` | fixed to the frame | absolute |

Hirschkorn's five coordinates are these: key, wippen and hammer measured from
ground, jack and repetition lever from the wippen [V, Hirschkorn 2004 §2.1,
Eq. 2.1, Figure 2.3].

The point not to get wrong: **the jack and the repetition lever are pinned to the
wippen**. Those pins are real kinematic pivots (center pins in felt bushings), not
contacts. With one angle per body, and each child's pin placed by its parent's
angle, the tree of pins is complete and the motion is an ordinary differential
equation in minimal coordinates, constrained only by unilateral contacts. That
holds with absolute angles for the jack and the lever too, which is how Thorin
writes it [V, Thorin 2013 §4.6, Eqs. 4.17–4.18 and Appendix B]. What is ruled out
is absolute coordinates with pin constraints ([FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever)): the two pins
become four scalar constraints, and one inherits an index-3 DAE, constraint drift,
Baumgarte stabilization and its non-physical parameters, for no benefit.

### Structure of the mass matrix

Planned, with the dynamics ([FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever)). The (w, j, r) block is an
**arrowhead, not a full 3×3 matrix**, because `M_jr = 0` exactly: neither the jack
nor the repetition lever is an ancestor of the other in the tree; they are two
sibling children of the wippen.

```
       θ_k    θ_w      θ_j      θ_r    θ_h   θ_d
θ_k [  I_k     0        0        0      0     0  ]
θ_w [   0   M_ww(q)  M_wj(q)  M_wr(q)   0     0  ]
θ_j [   0   M_wj(q)    I_j       0      0     0  ]
θ_r [   0   M_wr(q)     0       I_r     0     0  ]
θ_h [   0      0        0        0     I_h    0  ]
θ_d [   0      0        0        0      0    I_d ]
```

Notation: `O_w` is the wippen's pivot, the origin of its frame (`Pg2` in
Hirschkorn's ground frame); `J` and `L` are the jack's and the lever's pins, `Pw2`
and `Pw3` in the wippen's frame; `G_b` is body b's center of mass. Every one of
Hirschkorn's frames has its origin at its body's pivot [V, Hirschkorn 2004,
Figure A.1, Tables A.2–A.4]. Then:

- `I_k = I_k^cm + m_k ‖G_k − O_k‖²`, `I_j = I_j^cm + m_j ‖G_j − J‖²`,
  `I_r = I_r^cm + m_r ‖G_r − L‖²`, `I_h`, `I_d`: **constants**
- `M_ww(q) = I_w^cm + m_w ‖G_w − O_w‖² + Σ_{b∈{j,r}} [ I_b^cm + m_b ‖G_b(q) − O_w‖² ]`
- `M_wj(q) = I_j^cm + m_j (G_j − O_w)·(G_j − J)`; `M_wr(q)` likewise, with `L`

With **r** the child's pin seen from the wippen's pivot and **u** its center of
mass seen from its pin, the coupling is `I_cm + m(‖u‖² + r·u)` and the child's
share of the wippen's diagonal `I_cm + m‖r + u‖²` ([FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever)). **Do not
use** `I_cm + m(r·u)` and `I_cm + m‖r‖²`: they fit neither relative nor absolute
coordinates, and a positivity test passes on them anyway ([FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever)).
**Do not take `Pw2` for the wippen's pivot**: it is the jack's pin, listed with
`Pw3` as "Pin Location" in the wippen's table, and the wippen turns about its
frame's origin ([FOUNDATIONS > Hirschkorn's frames](../FOUNDATIONS.md#hirschkorns-frames)).

On Hirschkorn's action at rest, `M_ww = 8.80e-5`, `M_wj = 2.10e-7`,
`M_wr = 4.52e-6`, `I_j = 1.79e-6` and `I_r = 4.12e-6 kg·m²` [E, 10]. About their
centers of mass, the jack and the lever have `7.78e-7` and `4.10e-6 kg·m²` [V,
Hirschkorn 2004, Table A.1].

Solving `M x = b` is a closed form. The jack's and the lever's rows give
`x_j = (b_j − M_wj x_w)/I_j` and `x_r = (b_r − M_wr x_w)/I_r`; put into the
wippen's row, they leave one scalar equation,
`S x_w = b_w − (M_wj/I_j) b_j − (M_wr/I_r) b_r` with
`S = M_ww − M_wj²/I_j − M_wr²/I_r`; the key, the hammer and the damper are one
division each. That is about twenty operations, an LDLᵀ factorization of the
arrowhead written out in closed form, with no pivoting and no allocation. `S` is
strictly positive for any physical parameters, since the (w, j, r) block is the
Gram matrix of a kinetic energy, so the solve has no singular case.

The configuration dependence of `M_ww`, `M_wj` and `M_wr` comes from the jack's
and the lever's angles relative to the wippen. It is kept: it costs the sines and
cosines the contact geometry computes anyway, and the Coriolis terms of [Equations of motion, implementable form](#equations-of-motion-implementable-form) come
from it.

### Equations of motion, implementable form

Lagrange in minimal coordinates:

```
M(q) q̈ + C(q, q̇) q̇ = τ_g(q) + τ_spring(q) + τ_fric(q̇) + Σ_i J_i(q)ᵀ f_i(g_i, ġ_i)
```

- `C(q,q̇) q̇`: Coriolis and centrifugal terms, **non-zero only within the (w, j, r)
  subtree**. Featherstone's RNEA would do (overkill for three bodies), or the
  analytic form: `C_ww = ∂M_ww/∂θ_j · θ̇_j + ∂M_ww/∂θ_r · θ̇_r`, and so on. They are
  small, but not negligible during let-off, when the jack turns fast relative to
  the wippen.
- `τ_g(q)`: gravity. The key's comes from its measured mass and front weight,
  leads included: a key arrives leaded and is weighed whole ([FOUNDATIONS > Parts and statics](../FOUNDATIONS.md#parts-and-statics), [FOUNDATIONS > Named simplifications](../FOUNDATIONS.md#named-simplifications)). A test weight set on the key, as in the first non-confusion test
  ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)), is a point mass `(x_i, m_i)` on top of it.
- `τ_spring`: Hirschkorn measures `T = −0.073·(θ_r − 0.832)` N·m on the
  lever-to-wippen angle and a roughly constant 0.0042 N·m on the jack [V,
  Hirschkorn 2004 §A.4]. Thorin's springs take the linear form
  `C_s(θ) = κ(θ − θ_0)` [V, Thorin 2017 Eq. 5], three of them on his action:
  wippen–frame, jack–wippen and lever–wippen [V, Thorin 2013, Table 4.12]. The
  wippen support spring is fitted on some actions only ([FOUNDATIONS > Springs inside the wippen assembly](../FOUNDATIONS.md#springs-inside-the-wippen-assembly)).
- `τ_fric(q̇)`: friction at the pins. With the friction at the sliding contacts
  ([PITFALL #6 — Tangential friction is not everywhere](#pitfall-6--tangential-friction-is-not-everywhere)) it makes the interval between down weight and up weight ([FOUNDATIONS > Friction](../FOUNDATIONS.md#friction)). Hirschkorn regularizes it:
  `T = A·(tanh(ω/ω_t) + B1(ω/ω_t)/(1 + B2(ω/ω_t)⁴))` [V, Hirschkorn 2004
  Eq. 2.12], with A measured per pin, key 0.0122, wippen 0.000397, jack 0.000397,
  repetition lever 0.00247 and hammer 0.00101 N·m, and `B1 = B2 = 0`: kinetic
  friction only, static friction left out of his joints [V, Hirschkorn 2004,
  Table A.9]. Thorin does not regularize: `C_d(θ) = c_d · sign(θ̇)`, with `sign`
  set-valued on `[−1, 1]` [V, Thorin 2017 Eqs. 2–3]. **The engine follows Thorin**:
  axis friction is a set, built for the key's balance friction and planned for every
  pivot ([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)). His `c_d` values are mostly assumed,
  not measured ([FOUNDATIONS > Parameter lineage](../FOUNDATIONS.md#parameter-lineage); annex [A2 > The published dry torques, and where each comes from (N·m)](A2-scientific-literature.md#the-published-dry-torques-and-where-each-comes-from-nm)).
- `J_i(q)ᵀ f_i`: the unilateral contacts, thirteen in Hirschkorn's model ([The 13 contacts (Hirschkorn nomenclature, Table 2.1 and Table A.8)](#the-13-contacts-hirschkorn-nomenclature-table-21-and-table-a8)).

### The 13 contacts (Hirschkorn nomenclature, Table 2.1 and Table A.8)

| # | Contact | Regulation it carries |
|---|---|---|
| 1 | capstan, wippen heel | capstan height |
| 2 | key front, front rail punching | **key dip**, aftertouch |
| 3 | key tail, back rail cloth | rest position |
| 4 | back check, hammer tail | **checking**; the back check's angle |
| 5 | wippen, jack rest | the jack's position under the knuckle |
| 6 | wippen, repetition lever stop | the lever's height |
| 7 | **jack, knuckle** | **the main force path** |
| 8 | jack toe, let-off button | **let-off** |
| 9 | jack, repetition lever window | — |
| 10 | **repetition lever, knuckle** | repetition |
| 11 | repetition lever, drop screw | **drop** |
| 12 | hammer, string | blow distance |
| 13 | hammer shank, its rest | hammer rest height |

Thorin's article counts twelve, these without the shank's rest [V, Thorin 2017
§2.4]; his thesis adds two for the damper, key against damper and damper against
its rest [V, Thorin 2013, Table 4.2]. [FOUNDATIONS > Seventeen contacts per note](../FOUNDATIONS.md#seventeen-contacts-per-note) plans four for the damper.

### Sliding contacts: the heart of the variable leverage

This is **entirely solvable analytically**. Built: the circle against a line and
the circle against a circle, their lever arms tested against central finite
differences; the hybrid profile is planned ([FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap), [FOUNDATIONS > Seventeen contacts per note](../FOUNDATIONS.md#seventeen-contacts-per-note)). Three
primitives cover Hirschkorn's
thirteen contacts [V, Hirschkorn 2004 §2.2.1, Table 2.1]: a circle against a line
(ten), a circle against a circle (the capstan on the wippen heel, contact 1), and a
hybrid profile, an arc joined to two tangent lines, against a circle (the jack and
the lever under the knuckle, contacts 7 and 10). Thorin's twelve use the same
kinds: two circles for the back check against the hammer, two perpendicular lines
joined by an arc for the jack under the roller, a circle and a line segment for the
ten others [V, Thorin 2017 §2.4, Table 4].

**(a) Circle against line.** Body A carries a circle (center `c_A`, radius `R_A`),
body B a line (point `p_B`, unit normal `n_B` pointing from B to A). In the world
frame, with `o_A`, `o_B` the pivots the bodies turn about:

```
g        = (c_A − p_B)·n_B − R_A           (gap, > 0 = separated)
n        = n_B
x_c      = c_A − R_A·n_B                   (contact point)
∂g/∂θ_A  =  ( ẑ × (c_A − o_A) ) · n_B      (instantaneous lever arm, A side)
∂g/∂θ_B  = −( ẑ × (x_c − o_B) ) · n_B      (instantaneous lever arm, B side)
```

**Do not use** `∂g/∂θ_B = −( ẑ × (x_c − o_B) )·n_B − (c_A − p_B)·( ẑ × n_B )`: the
second term is spurious. A circle at (0, 2) of radius 1 against a line through
(5, 0) on a body turning about the origin has a true derivative of 0; that formula
gives −5 ([FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap)).

`x_c` **moves along the line** as the bodies turn. That is what the variable
leverage is, physically: `∂g/∂θ` is the instantaneous lever arm, and it varies
continuously through the key's descent. About 15 flops, exact, with no iteration
and no closest-point search. For a coordinate relative to a parent (jack, lever),
`o` is the body's own pin, and the parent's coordinate turns the child's points
about the parent's pivot ([FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap)). **The arm is not the distance to the
pivot**: at rest the knuckle sits on the jack's flat top, whose normal runs close
to the jack's pin, so the jack's own arm is 3.5 mm though its contact point is
49.5 mm from the pin [E, 10].

**(b) Circle against circle** (Hirschkorn's capstan on the wippen heel, `Pk3`
against `Pw5`; Thorin's back check against the hammer).
```
d = c_A − c_B ;  g = ‖d‖ − (R_A + R_B) ;  n = d/‖d‖ ;  x_c = c_A − R_A·n
```

**(c) Hybrid profile, an arc and two tangent lines, against a circle** (jack and
lever under the knuckle). Hirschkorn: `Pj2` is an arc of radius 1.0 mm over the
angular interval [1.571; 3.142] rad with its two tangent lines, against the circle
`Ph2` of radius 4.4 mm, the knuckle; `Pr2` is an arc of radius 55.2 mm over
[1.300; 1.571] against the same knuckle [V, Hirschkorn 2004, Tables A.5–A.7]. Thorin
describes the jack under the roller as two perpendicular lines joined by a circular
arc [V, Thorin 2017 §2.4].
Algorithm: classify the angular position of `c_B` relative to the arc's center and
its angular extent, and fall back on (b) or (a). **The construction is C1**, since
the lines are tangent to the arc: the normal never jumps. The jack sliding under
the knuckle needs exactly that, since a jump of the normal would be a jump of the
transmitted torque.

**Relative tangential velocity** (for friction), for bodies turning about fixed
pivots; for the jack and the lever, add the wippen's rotation about its own pivot:
```
t    = ẑ × n
v_t  = [ (θ̇_A ẑ × (x_c − o_A)) − (θ̇_B ẑ × (x_c − o_B)) ] · t
```

**Observable to expose in the UI**: the instantaneous leverage ratio
```
ratio(t) = ( r_strike · θ̇_h ) / ( r_front · θ̇_k )
```
2 flops, plotted live against key dip. A lever ratio is never a constant, and the
literature's small-angle ratios are linearizations about rest ([FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap)):
the variation over the stroke is read off the assembled model, by virtual work. It
is what moves when the capstan height or the hammer's hang changes. Of the trade's
definitions of the action ratio, which differ on one action ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)),
the ratio of useful travels, (blow distance − let-off)/(key dip − aftertouch), is
5.06 with assumed mid-range regulation (46, 1.5, 10 and 1.2 mm) [E, 06]. **Do not
use** the "BYU Design Review" figures for the ratio's variation (about 5.5:1 at the
top of the travel, 5.06:1 "true", about 5.0:1 at let-off, modern targets of 4.9 to
5.3): they trace to no source in the corpus.

---

## Stiffnesses, effective masses and time steps

### Contact law and measured coefficients

Hirschkorn measures **each contact separately** on his disassembled Boston GP-178
action: one part in a vise, the other on an arm driven by a small torque-controlled
motor, an encoder on the arm. He fits a cubic to the mid-line between the
measured loading and unloading curves, and calibrates a damping factor `D` on a
forte blow [V, Hirschkorn 2004 §2.2.3]:

```
f_n = f_fit(x) · (1 + D·ẋ)        with   f_fit(x) = a·x³ + b·x² + c·x
```

x in m, f in N [V, Hirschkorn 2004 Eqs. 2.4–2.5]; a cubic, a few multiplications.
Table A.8, as published [V, Hirschkorn 2004, Table A.8]:

| # | contact | a | b | c | D (s/m) |
|---|---|---|---|---|---|
| 1 | capstan/wippen heel | 1.26e10 | 3.26e7 | 7.92e3 | 1.877 |
| 2 | key front/punching | 4.41e10 | −1.02e7 | 1.57e4 | 1.219 |
| 3 | key tail/back rail cloth | 2.42e10 | −1.11e7 | 6.54e3 | 2.502 |
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

All thirteen elastic branches are positive and strictly increasing for any
positive penetration; where `b < 0` the stiffness dips below `c` before it rises
[E, 04, 05]. Order of magnitude of the measured curves under the key front: about
35 N at 0.86 mm of penetration, where the measurement stops, with loading and
unloading clearly apart [V, Hirschkorn 2004, Figures 2.5 and 2.6, pp. 20 and 22].
**Do not use** "about 40 N near 1 mm": 40 N and 1 mm are where the figures' axes
end.

### Effective masses along the contact normals

What a contact's frequency depends on is its stiffness against the **effective
mass along its normal**, `m_eff = 1/(∇gᵀ M⁻¹ ∇g)`, with `∇g` the gradient of its
gap over the five coordinates (the lever arms of [Sliding contacts: the heart of the variable leverage](#sliding-contacts-the-heart-of-the-variable-leverage)) and `M` the assembled mass
matrix. On Hirschkorn's action at rest, every frame at its own pivot [E, 10]:

| # | contact | m_eff (g) |
|---|---|---|
| 1 | capstan/wippen heel | 21.3 |
| 2 | key front/punching | 65.8 |
| 3 | key tail/back rail cloth | 74.8 |
| 4 | back check/hammer tail | 112.7 |
| 5 | wippen/jack rest | 2.1 |
| 6 | wippen/rep. lever stop | 4.3 |
| 7 | jack/knuckle | 8.9 |
| 8 | jack toe/let-off button | 1.9 |
| 9 | jack/lever window | 0.8 |
| 10 | rep. lever/knuckle | 2.7 |
| 11 | rep. lever/drop screw | 1.5 |
| 12 | hammer felt/string | 8.2 |
| 13 | shank/rest rail | 12.5 |

The smallest belong to the contacts of the jack and the repetition lever. The
jack/knuckle contact is not among them at rest, because the knuckle sits on the
jack's flat top, whose normal runs close to the jack's pin: there the jack's own
arm is 3.5 mm, the wippen's 92.8 mm and the hammer's 18.2 mm per radian [E, 10].
Its effective mass falls as the knuckle rolls onto the jack's side wall at the end
of escapement, to 0.73 g, with the force collapsing ([FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step)).

At the jack/knuckle, the reflected mass is about 9.1 g on the driving side (key,
wippen, jack and lever, the hammer held) against 466.7 g on the hammer side,
**51:1** [E, 10]. **Do not use 570:1** for that ratio, nor the argument that it
puts the knuckle in the regime where the projected Gauss–Seidel solvers of game
engines converge badly: the reflected-mass ratio at the knuckle does not
distinguish a dedicated solver ([FOUNDATIONS > A dedicated planar solver, not a general-purpose engine](../FOUNDATIONS.md#a-dedicated-planar-solver-not-a-general-purpose-engine), [FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step)).

**Do not compute an effective mass as `I_pivot / r²`**, with `r` the distance
from a pivot to a contact point: the arm that counts is the moment arm of the
contact normal, taken for each coordinate, and the jack and the lever ride on the
wippen. That method, with `Pw2` taken for the wippen's pivot, gives 0.73 g at the
jack/knuckle at rest, a value the contact reaches only on the jack's side wall
([FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step), [FOUNDATIONS > Hirschkorn's frames](../FOUNDATIONS.md#hirschkorns-frames); `tools/derivations/README.md`).

### Contact stiffnesses, linearized modes, and the step

Method: do **not evaluate a stiffness at an arbitrary penetration**. That yields
absurdities: 1 mm at the lever/knuckle is 1037 N, which never happens. Evaluate it
at a plausible peak force, by solving `f_fit(x) = F_peak` by bisection, then
`k = 3ax² + 2bx + c`. The forces are assumptions [S], the ones script 10 loads its
contacts with:

| # | contact | assumed peak F (N) [S] | x (mm) | k (N/m) |
|---|---|---|---|---|
| **7** | **jack/knuckle** | 100 | 0.582 | 4.44e5 |
| **10** | **rep. lever/knuckle** | 25 | 0.283 | 2.42e5 |
| 11 | rep. lever/drop screw | 15 | 0.262 | 1.58e5 |
| 12 | hammer felt/string | 800 | 2.469 | 7.47e5 |
| 8 | jack toe/button | 25 | 0.715 | 9.49e4 |
| 9 | jack/window | 3 | 0.352 | 1.75e4 |
| 4 | back check/hammer tail | 15 | 0.253 | 1.25e5 |
| 6 | wippen/lever stop | 3 | 0.412 | 1.18e4 |
| 1 | capstan/wippen heel | 60 | 1.060 | 1.20e5 |
| 5 | wippen/jack rest | 3 | 0.540 | 8.23e3 |
| 2 | key front/punching | 50 | 1.003 | 1.28e5 |
| 3 | key tail/back rail cloth | 5 | 0.592 | 1.88e4 |
| 13 | shank/rest rail | 2 | 0.948 | 2.60e3 |

Those forces are assumptions: 100 N at the jack/knuckle means a hammer-head
acceleration of about 1,600 m/s² [E, 08].

With these stiffnesses on the contacts named and the assembled mass matrix, the
highest linearized mode of Hirschkorn's action at rest is 1174 Hz with the
capstan, the front punching and the jack/knuckle loaded (contacts 1, 2, 7), 2240 Hz
with the repetition lever and the drop screw loaded too (1, 2, 7, 10, 11), and
2268 Hz with all thirteen at once, an upper bound [E, 10]. These are the starting
estimates of [FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step). **The step itself is chosen by the step-convergence
study**: the same scenarios at decreasing steps, until the key force and the
observables (the let-off instant, the hammer's speed at let-off, drop, checking)
stop changing by more than their resolution, with sub-steps only where a measured
criterion asks for them ([FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step)).

**Do not use 12 µs**, nor the per-contact table it comes from (3931 Hz and a
twentieth of a period of 12.7 µs at the jack/knuckle on a reflected mass of
0.728 g; 2450 Hz at the lever/knuckle on 1.02 g): those figures take `Pw2` for the
wippen's pivot and divide `I_pivot` by a radial distance squared ([Effective masses along the contact normals](#effective-masses-along-the-contact-normals);
[FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step)). Nor the phase table that went with them (12.7 µs during key
depression, 12 to 20 µs at let-off, 2 to 5 µs at hammer–string impact): the step
is the convergence study's to choose. **Do not use 2/ω
as the stability limit**: it is the undamped limit of an explicit spring; with the
contact damping implicit in velocity the limit widens (at a damping ratio of 0.3,
ωΔt < 2.69), and stability is not accuracy [E, 05]. **Do not use 280 Hz** for the
hammer's first mode, nor the 180 µs derived from it as a twentieth of its period:
the figure is in no source; Thorin estimates the hammer's first mode at 39 Hz [V,
Thorin 2017 §2.1 and Appendix] ([FOUNDATIONS > Rigid bodies](../FOUNDATIONS.md#rigid-bodies)).

The non-smooth scheme of [Solver choice: the options and the hypothesis](#solver-choice-the-options-and-the-hypothesis) runs at Δt = 0.5 ms (2 kHz): Thorin's computed forces
at 2 and 10 kHz are very similar, while 0.4 kHz induces significant discrepancies
[V, Thorin 2013 §6.4.1, Figure 6.10; Thorin 2017 §5, Figure 12]. In his words,
"there is no numerical parameter for impact or dry friction calling for a very
small time-step" [V, Thorin 2017 §5]. **Do not use** "a gain of 25 to 100 times on
the time step" for the non-smooth scheme: it divides 500 µs by compliant steps
taken from the per-contact figures refuted above.

### Hammer–string impact

**[?]** Contact durations of about 4 ms in the bass and under 1 ms in the extreme
treble, varying by about ±20 % from piano to fortissimo, are attributed to
Askenfelt and Jansson, who are not in the corpus ([FOUNDATIONS > Sources](../FOUNDATIONS.md#sources)); Goebl et al.
cite them on contact durations without giving the figures [V, Goebl et al. 2005,
p. 1155].

Stulov's model, fitted across the 88 notes of one set of new, unvoiced Abel
hammers, struck at 0.85 m/s in the bass down to 0.50 m/s in the treble [V, Stulov
2005 §9, Eqs. 11–16]:
```
F(u(t)) = F0 · [ u^p(t) − (ε/τ0) ∫₀ᵗ u^p(ξ) exp((ξ−t)/τ0) dξ ]
m0 = 11.074 − 0.074·N + 0.0001·N²   [g]      (A0 = 11.0 g ; C8 = 5.3 g)
p  = 3.7 + 0.015·N                            (3.72 → 4.98, as Stulov states it)
ε  = 0.9894 + 0.000088·N                      (0.9895 → 0.9972)
τ0 = 2.72 − 0.02·N + 0.00009·N²    [µs]       (2.70 → 1.65)
F0 = 15500 · exp(0.059·N)          [N/mm^p]   (16440 → 2787300, factor of 170 bass→treble)
```

**Two-time-scale structure**: `δ = 1 − ε` (0.0028 to 0.0105) is the ratio of the
static to the instantaneous slope [V, Stulov 2005 §9]. Therefore:
- quasi-static stiffness = `F0·δ`
- instantaneous stiffness = `F0`, i.e. 1/δ, **about 100 to 350 times higher**, but
  acting only over `τ0 ≈ 2 µs`

Integrated naively, the hereditary integral shows `F0` and asks for `dt ≪ 2 µs`.
The exponential kernel is an auxiliary state variable instead:
`z(t) = (1/τ0) ∫₀ᵗ u^p(ξ) exp((ξ−t)/τ0) dξ` obeys `τ0 ż = u^p − z`, and
`F = F0 (u^p − ε z)`. Its exact update, with the input `w = u^p` varying linearly
across the step (first-order hold), is

```
z_{n+1} = e·z_n + (1 − e)·w_n + [1 − (τ0/Δt)(1 − e)]·(w_{n+1} − w_n),    e = exp(−Δt/τ0)
```

unconditionally stable, `e` and the bracket computed once. When `Δt ≫ τ0` it tends
to `z ≈ w − τ0 ẇ`, so `F ≈ F0 [ δ·u^p + ε τ0 · d(u^p)/dt ]`: the static branch and
the dissipation, both carried at the macro step ([FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law)). **Do not use**
the update `z ← z·exp(−dt/τ0) + (1 − exp(−dt/τ0))·u^p`, with `u^p` held constant
over the step: at a step much longer than `τ0` it sets `z` to `u^p` at every step,
leaves the static branch `F0·δ·u^p` alone, and removes most of the dissipation the
hereditary law exists to model ([FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law)). Nor read a contact duration off
the linearized half-period of the static branch against a rigid wall (0.1 to
0.3 ms across the compass at 5 m/s): for a hardening spring the linearization at
peak compression is the stiffest point of the stroke, so it understates the
contact, and it drops the hereditary term, which carries the dissipation.

**Practical consequence**: for an action simulator, as opposed to sound synthesis,
the string needs only a finite impedance or a calibrated restitution law. That is
what Hirschkorn does: his string is a rigid body, and only the damping factor of
contact 12 (0.175) "was iteratively adjusted to produce approximately the correct
rebound speed for the hammer" [V, Hirschkorn 2004 §2.2.1, §2.2.3]. Thorin's
"simple ad hoc Newton elastic law suffices to reproduce the kinematic measurements
of the velocities of the hammer after it escapes the mechanism and before it is
captured again" [V, Thorin 2017 §2.1]. The rebound speed matters because it governs
checking. Whether sound is in scope is open ([FOUNDATIONS > Open questions for the owner](../FOUNDATIONS.md#open-questions-for-the-owner)).

---

## Solver choice: the options and the hypothesis

[FOUNDATIONS > One dynamic solver](../FOUNDATIONS.md#one-dynamic-solver) states the hypothesis the dynamics prototype tests: **one dynamic
solver** serves the bench and the keyboard, with compliant contacts treated
implicitly at velocity level, axis friction as a bounded unknown at velocity level
(the set of [Equations of motion, implementable form](#equations-of-motion-implementable-form)), and a fixed step; the quasi-static solver is its static limit.
This section sets out the options it is drawn from.

### (a) Explicit adaptive step (Dormand–Prince RK45/RKF45) with event detection — not for the real-time path
Attractive on paper. In practice: 13 contacts, several engagements and
disengagements each per blow, several dozen events per keystroke, some of them in
chattering (the jack bottoming out, the hammer bouncing in the back check). Root
finding dominates the cost, and a variable step cannot promise "n steps per
frame", which the real-time path needs. Reproducibility is not the objection: with
identical operations in the same order, an adaptive step reproduces bit for bit
too ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).

### (b) Semi-implicit fixed step, contact force implicit in velocity — the form of the hypothesis
Semi-implicit (symplectic) Euler or a fixed-step RK2, with the contact force
treated implicitly in velocity. Simple to write, to parallelize and to keep
deterministic, and the felt's compliance is native. [FOUNDATIONS > One dynamic solver](../FOUNDATIONS.md#one-dynamic-solver) takes this
form with axis friction set-valued, not regularized. Its step is set by the
step-convergence study, from the starting estimates of [Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step). **Do not use**
"dt = 20–40 µs as a baseline, with sub-steps of 5 µs at let-off and 2–5 µs at
impact", nor "125 Mflop/s per note at 20 µs": the steps come from the per-contact
figures refuted in [Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step), and the cost from a flop count of a kernel never written
([Compute budget](#compute-budget)).

### (c) Stiff implicit, variable step (BDF / Radau IIA / Rosenbrock) — not as Hirschkorn ran it
This is Hirschkorn's choice: MATLAB `ODE15s`, absolute tolerance 1e-5 rad, and "a
full simulation for the system with a single force profile input requires about
6 minutes on a 2.4 GHz PC computer" [V, Hirschkorn 2004 §2.4]. He finds that "the
solvers dedicated to stiff systems were much more efficient for this system. The
contacts in the system are very stiff, with forces changing drastically with small
variations in position. Even more significant in contributing stiffness is the
friction model" [V, Hirschkorn 2004 §2.4]: his regularized friction makes the
system stiffer still than his contacts do. What rules his combination out here is
the variable step, which cannot keep a fixed frame cadence, and the regularized
friction law ([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)), not implicitness: a fixed-step, linearly implicit
step with set-valued friction is option (b), and it is deterministic.

Computing times reported in the literature:

| Work | Computation time | Slower than real time by |
|---|---|---|
| Van den Berghe et al. 1995 | 1 s simulated in about 4 h on a 486DX33 [V, Thorin 2017, Table 1] | about 14,400 |
| Van den Berghe et al., reduced model | 1 s in about 15 s on a DEC 5000/33 [V, Thorin 2017, Table 1] | about 15 |
| Hirschkorn 2004 (MATLAB ODE15s) | about 6 min per force profile on a 2.4 GHz PC [V, Hirschkorn 2004 §2.4]; his profiles last 0.70 s [V, §4.1] | about 500 (360 s / 0.70 s) |
| Izadbakhsh (Maple, flexible shank) | about 75 min on a 2.4 GHz PC [V, Thorin 2017, Table 1] | not given: the simulated duration is not stated |
| Masoudi & Birkett 2015 | 910 s with a rigid shank, 1720 s with a flexible one [V, Thorin 2017, Table 1] | about 4550 for the rigid model, Thorin's estimate from a 200 ms figure [V, Thorin 2017 §1] |
| **Thorin et al. 2017 (non-smooth, XDE, 3D)** | about **20 times slower than real time** on a 2010 computer with a 2.1 GHz Intel Core i5 [V, Thorin 2017 §6] | **about 20** |
| Timmermans, Dehez, Fisette 2020, haptic key | **[?]** reported to compute a multibody action model in real time on the controller of a force-feedback key; the paper is not in the corpus, and its figures are unchecked ([FOUNDATIONS > Sources](../FOUNDATIONS.md#sources)) | — |

The non-smooth formalism "proved to allow for simulations of about two orders of
magnitude faster than the regularizing approach, as reported in the literature"
[V, Thorin 2017 §6]. Real time on a laptop is Thorin's expectation, not his
result: before the felt laws and the synchronization were implemented, which
slowed the computation, his simulations ran at about 1.2 times real time, and he
expects a gain of more than 15 from implementing the felt laws in the engine's
kernel and of more than 3 from computing in 2D rather than 3D [V, Thorin 2013
§6.4.1, p. 110].

### (d) Non-smooth dynamics (Moreau–Jean) — Thorin's scheme
Thorin's scheme [V, Thorin 2017 §4, Eqs. 15–19]:
```
M(x_{n+θ}(k_x)) k_v  =  F_H(x_{n+θ}(k_x), v_{n+θ}(k_v), t_{n+θ})  +  H(x_{n+γ}(k_x)) r
v_{n+θ}(k_v) = k_x
( g(x_{n+γ}(k_x)) ,  Hᵀ(x_{n+γ}(k_x)) v_{n+γ}(k_v) ,  r )  ∈  K
```
with `x_{n+α}(k_x) = x_n + α·Δt·k_x`, `v_{n+α}(k_v) = v_n + α·Δt·k_v`, **θ = 1**
(stability of the smooth dynamics) and **γ = 1** (constraints satisfied at the end
of each step). `k_x` is eliminated; the contact kinematics `H` and the smooth
dynamics are linearized around the previous iterate; a Newton loop gives a
One-Step Non-Smooth Problem in `r` alone, reformulated with an **augmented
Lagrangian** and solved by a **Gauss–Seidel-like algorithm** (Studer 2009, ch. 4,
as cited there).

**The felt keeps its compliance**, because Thorin does not stiffen it. Each of the
twelve contacts gets "a small additional rigid body (labelled 'A' in Fig. 3), a
prismatic joint ('P') and a visco-elastic element 'V' (Kelvin–Voigt model) ...
inserted between the felt support and the contacting body ('B'). A unilateral
constraint prevents this additional body 'A' from penetrating the contacting body
'B'", with an impact law on A "that we choose to be inelastic to allow permanent
contact phases", and 1 mg for the mass of every A [V, Thorin 2017 §2.4]. What the
scheme gives up at a long step is the resolution of contact modes near or above
the step's frequency: implicit Euler (θ = 1) multiplies an undamped mode's
amplitude by 1/√(1 + (ωΔt)²) per step, 0.30 for a 1 kHz mode at 500 µs
(ωΔt = π). Thorin's convergence between 2 and 10 kHz says his observables do not
see it ([Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step)).

Felt law: `F_felt(δ) = k·δ^r + b·δ²·δ̇` [V, Thorin 2017 Eq. 6], three parameters
per felt, identified by minimizing a cost functional over a series of dynamic
measurements from piano to forte [V, Thorin 2017 §3.1]. The authors stress that
"numerical methods adopted here allow for an efficient simulation for any stiffness
and even rigid contacts, opening doors to efficient sensitivity analyses" [V,
Thorin 2017 §2.4].

**The argument for the trade.** Thorin writes:
> "A basic adjustment of the key mechanism consists in measuring the maximum mass
> to be put on the key without initiating the down-motion from the resting position
> and conversely, the minimum mass that prevents the up-motion from the
> down-position. The result directly depends on the dry friction coefficient in
> the hinges. However, this simple test cannot be simulated with a regularized
> friction law." [V, Thorin 2017 §2.5]

That is down weight and up weight. A regularized law has zero friction at zero
speed, so a static solve with it collapses D onto U ([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)). **Do not
use** "with regularized friction the 30 g gap between D and U vanishes" for a
dynamic simulation: with a threshold speed of 10⁻⁶ rad/s the key keeps the
threshold, creeping under a micrometer in ten seconds under a gram of excess
[E, 07]. The objection that stands is another on Thorin's list: "At least one
non-physical parameter (the so-called regularization parameter) is introduced,
weakening the robustness of the simulation. This might prove critical in a
sensitivity analysis." The rest of his list: the sticking phase and the stick–slip
transitions are lost, the friction vanishing with the velocity; spurious
oscillations if the regularized zone is too narrow, inaccurate results if it is
too wide; a smaller step to remove the oscillations, or for the scheme's
stability; "the condition number of the matrices involved increases as the slope
of the regularized function increases" [V, Thorin 2017 §2.5]. **Do not argue against
regularization from the time step**: its tiny step holds only for explicit
integration; treated implicitly, a velocity-only friction law sets no step limit
[E, 07] ([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)).

### (e) Hybrid: non-smooth hard stops, compliant felt — what Thorin already does
There is no choice between (d) and (e): (d) **is** (e). The 1 mg auxiliary bodies
realize the decomposition into rigid non-penetration (non-smooth) and felt
compliance (smooth), and the dry friction stays set-valued. It is published and
compared with measurements ([Quantified validation plan](#quantified-validation-plan), level 3).

### The hypothesis, and what it rules out

One dynamic solver, of form (b) with the friction of (d): compliant contacts
implicit in velocity, axis friction a bounded unknown at velocity level, a fixed
step ([FOUNDATIONS > One dynamic solver](../FOUNDATIONS.md#one-dynamic-solver)). Its test: step convergence; Hirschkorn's forte, piano and
double-blow trajectories; Thorin's key force under position drive; the trade's
protocols at states both solvers share. A second dynamic solver comes only if a
measured criterion fails.

The **quasi-static solver** is the dynamic solver's static limit: the same
contacts and friction sets, inertia dropped, solved to convergence under an
iteration cap. Measuring D and U by time integration would be noisy and slow; what
the protocols need is a continuation on `θ_k` (an angle increment, the static
equilibrium solved at each step with the friction sets), which yields the
descending and ascending curves and so D, U, BW and F. With compliant contacts the
configuration depends on the forces, so each load increment of a weighing is a
nonlinear complementarity problem; reading D and U off two linear programs holds
only for rigid contacts on one coordinate ([FOUNDATIONS > One dynamic solver](../FOUNDATIONS.md#one-dynamic-solver)).

**Do not use** the three-solver plan (a non-smooth main solver at 250 µs, a penalty
solver at 20–40 µs with a regularized `tanh` friction for the keyboard, and a
quasi-static solver apart from both), nor the same scenario played through two
dynamic solvers as a free non-regression test: it doubles the costliest code for a
cross-check that convergence studies and published data provide ([FOUNDATIONS > One dynamic solver](../FOUNDATIONS.md#one-dynamic-solver)),
and a regularized friction in one of them would make the cross-check compare two
different models ([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)).

**How the boundary falls** (planned). The pure domain, with no I/O and no web,
holds the analytic contact geometry (the three primitives), the material laws, the
kinematic tree and the regulation observables. A `Contact` port exposes
`{ gap(q), dgap_dq(q), normal(q), tangent(q), contact_point(q), relative_tangential_velocity(q, q̇) }`.
The dynamic solver and its static limit consume **the same contact geometry, felt
laws and friction sets**; they differ in whether inertia is kept.

---

## Regularization and stability: the concrete pitfalls

### PITFALL #1 — Hunt–Crossley produces an ATTRACTIVE normal force
`f_n = f_fit(x)·(1 + D·ẋ)` becomes **negative** (sticky) as soon as `ẋ < −1/D`.
With Hirschkorn's measured D values [E, 04]:

| # | contact | D (s/m) | critical ẋ |
|---|---|---|---|
| 13 | shank/rest rail | 8.964 | **−0.112 m/s** |
| 6 | wippen/lever stop | 7.732 | −0.129 |
| 5 | wippen/jack rest | 5.781 | −0.173 |
| 11 | rep. lever/drop screw | 4.985 | −0.201 |
| **7** | **jack/knuckle** | **4.694** | **−0.213** |
| 10 | rep. lever/knuckle | 2.935 | −0.341 |
| 4 | back check/hammer tail | 2.446 | −0.409 |
| 3 | key tail/back rail cloth | 2.502 | −0.400 |
| 1 | capstan/wippen heel | 1.877 | −0.533 |
| 8 | jack toe/button | 1.527 | −0.655 |
| 2 | key front/punching | 1.219 | −0.820 |
| 9 | jack/window | 0.779 | −1.284 |
| 12 | hammer felt/string | 0.175 | −5.714 |

Separation speeds are of the order of 0.2 to 1 m/s (the hammer tail leaving the
back check, the jack leaving the knuckle, the hammer leaving its rest rail), and
**11 of the 13 contacts turn attractive above −1 m/s** [E, 04]. The remedy is
mandatory, whatever the law: `f_n = max(0, f_fit(x)·(1 + D·ẋ))` ([FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law)).
Lankarani–Nikravesh, of the same multiplicative form, has the same flaw.

### PITFALL #2 — Real felt is rate-independent; the damped laws are not
Hirschkorn's qualitative observations from his felt-compression tests [V,
Hirschkorn 2004 §2.2.2]:
> **Non-linear spring**: "It also does not follow any simple power function."
> **Hysteretic**: "The felt requires more force to compress than it returns on decompression."
> **Speed independent**: "Whether the felt is loaded slowly or quickly seems to have little effect on the loading profile. Instead, it is dependent on the **maximum and minimum loads the felt has previously undergone**."

He names the weakness of his own law: "The biggest weakness of this model is that
the hysteretic damping behaviour is speed dependent, and it is known from previous
research (Stamm) that the behaviour of real felt is not" [V, Hirschkorn 2004
§2.2.2]. His workaround: choose D so that the law matches the measured loading
curve at the contact's average speed,
`D = [f_loading(x_max)/f_fit(x_max) − 1]/ẋ_ave`, with `ẋ_ave` measured on a real
forte blow with encoders and a high-speed camera, the instants out of contact
filtered out [V, Hirschkorn 2004 §2.2.3, Eq. 2.7]. Recalibrated for a piano blow,
the factors come out four to five times larger [V, Hirschkorn 2004 §4.2.2]: one
damping constant cannot serve from pianissimo to fortissimo [E, 07].

Options and judgment:
- **Hunt–Crossley / Lankarani–Nikravesh**: velocity-dependent, simple, calibrated
  as Hirschkorn does; one calibration holds at one dynamic level only.
- **Gonthier et al.**: rejected by Hirschkorn for a structural reason worth
  remembering: "one of the key parameters, the damping factor, is determined from
  the **initial contact velocity**. This means the model can only be used in
  situations where the contact bodies are initially apart before making contact
  and rebounding. However, in the piano action, bodies can **start in contact, and
  might stay in contact throughout the entire simulation**" [V, Hirschkorn 2004
  §2.2.2]. Fatal here: the capstan rests on the wippen heel.
- **Dunlop's model** (springs and friction elements): rejected by Hirschkorn for
  three reasons: the wrong form (it gives a displacement for a known force, where
  the contact needs the converse), no calibration method, and Stamm's tests casting
  doubt on its ability to reproduce Dunlop's own results [V, Hirschkorn 2004
  §2.2.2].
- **Masoudi and McPhee's micromechanical model**: a published speed-independent
  model of felt hysteresis, from fiber-to-fiber friction, with separate loading and
  unloading laws and an algorithm for the transitions between them, identified for
  hammer felt and applied to an upright piano action [V, Masoudi & McPhee, abstract,
  Eqs. 9–10, Figures 6–7]. **Do not use** "no published rate-independent felt
  model": this is one ([FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law)).
- **Loading and unloading curves with a memory of the turning points**: the
  literal description of what Hirschkorn observes, rate-independent by
  construction. The Preisach model is one such construction, which Hirschkorn
  proposes himself [V, Hirschkorn 2004 §5.2]. Bouc–Wen expresses rate-independent
  hysteresis with an internal state variable; the corpus holds no identification of
  it for piano felt.
- **Stulov's hereditary law**: rate-dependent, with an exploitable two-scale
  structure ([Hammer–string impact](#hammerstring-impact)); the only one fitted across 88 notes, for hammer felt against the
  string.

[FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law) states the hypothesis: Hirschkorn's cubics are the elastic term,
and the target is a rate-independent law with loading and unloading branches and a
memory of its turning points, after Masoudi and McPhee. Its identification needs
loading and unloading curves contact by contact, which Hirschkorn plots for the key
front alone and no source in the corpus gives for a grand action.

### PITFALL #3 — C1 continuity at contact entry
`f_fit(x) = ax³ + bx² + cx` has `f_fit(0) = 0` and `f'_fit(0) = c > 0`: the force is
C0 but **the stiffness jumps from 0 to c**, 2.4 to 32.3 kN/m depending on the contact
[V, Hirschkorn 2004, Table A.8]. That is an acceleration discontinuity at every
contact entry, and with 13 contacts engaging and disengaging several times per
blow it pollutes any error-controlled integrator. Remedies: (i) a C1 ramp over the
first 10–50 µm (`f ← f·s(x/x_ramp)`, `s` a smoothstep), whose stiffness must then
be the ramped law's own derivative; (ii) an elastic law with zero slope at zero
penetration, as Thorin's `k·δ^r` with r = 2.7 or 3 has; (iii) non-smooth contact,
where the question does not arise.

With Hirschkorn's cubic, the choice of damping form does not remove the jump.
Hunt–Crossley's damping term `D·f_fit(x)·ẋ` vanishes at x = 0, as an additive
`b·δ²·δ̇` does; at touchdown the stiffness is `c(1 + D·ẋ)` with the first and `c`
with the second [E, 07]. **Do not use** "Thorin's `b·δ²·δ̇` settles pitfalls 1 and 3
in one stroke". With one constant `b_d`, the additive and the multiplicative forms
agree only at the penetration `b_d` was calibrated at, and the additive one turns
attractive too, at a separation speed that varies with the penetration (on contact
7, −0.270 m/s at 0.1 mm and −0.171 m/s at 0.3 mm, against −0.213 m/s at the
calibration point) [E, 07]. The clamp `max(0, f)` is the remedy in both forms
([FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law)).

### PITFALL #4 — Chattering
The risk zones: (a) the hammer tail in the back check, which bounces, and where
tangential friction is, in Thorin's words, "crucial for stopping the motion of the
hammer after hitting the string" [V, Thorin 2017 §2.4]; Hirschkorn's model without
tangential friction fails to check the hammer [V, Hirschkorn 2004 §4.4]; (b) the
jack bottoming out on the let-off button; (c) the hammer settling onto the
repetition lever after drop. Remedies: an inelastic impact law (Thorin's, on the
auxiliary bodies), force saturation, and above all **robust lift-off detection**:
declare separation only if `g > 0` AND `ġ > ε` over several consecutive steps
(hysteresis on the contact state), never on the sign of `g` alone.

### PITFALL #5 — Conditioning
Thorin: "The condition number of the matrices involved increases as the slope of
the regularized function increases" [V, Thorin 2017 §2.5]. On one action the
effective masses along the contact normals span 0.8 to 112.7 g at rest [E, 10],
and the tangent stiffnesses at the assumed forces of [Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step) span 2.6e3 to 7.5e5 N/m:
the Delassus matrix is ill-conditioned, and Gauss–Seidel must be preconditioned
(diagonal scaling). A fixed iteration budget belongs to the real-time path, where
the cost per step must be bounded; a loop that stops on a tolerance, under a hard
cap and with its iteration count reported, is required in the metrological paths
([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)). **Do not use** the argument that a convergence test destroys
reproducibility: with identical operations in the same order it does not
([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).

### PITFALL #6 — Tangential friction is not everywhere
Hirschkorn applies tangential friction on **4 contacts** only (1, 4, 7, 10) [V,
Hirschkorn 2004 §2.2.4]; Thorin on **3** (jack/knuckle, back check/hammer,
jack/let-off button), because "friction is ignored in the other contacts which are
subject to a rather small translational motion" [V, Thorin 2017 §2.4]. Do not
over-model: each frictional contact adds unknowns and iterations. But **the
jack/knuckle friction must be there**: "Friction between the jack and the roller is
easily felt by the pianist and must also be considered in the model, because the
piano technicians adjust it carefully" [V, Thorin 2017 §2.4]. On Hirschkorn's law,
read with its bracket (A multiplies both terms, Eq. 2.10, p. 25), the static
peaks are 1.69, 1.72 and 1.13 times the kinetic coefficient at the back check,
the jack/knuckle and the lever/knuckle, and there is none at the capstan
[E, 07]; do not use 3.0, 6.6 and 1.9, which come from the bracket-less law a
plain-text extraction of the thesis prints.

---

## Compute budget

[FOUNDATIONS > The 88-note budget](../FOUNDATIONS.md#the-88-note-budget) states the hypothesis: one dynamic solver, the notes computed
independently on a few workers, plays 88 notes in real time on a laptop, because a
note is active for a few hundred milliseconds and static in its hold. Its test is
measured: the prototype's cost per step, natively and in Chrome and Safari, and the
notes active at once over a corpus of MIDI files, pedal changes included.

### What a step costs

Measured on the dynamics prototype, not estimated. What dominates is known in
kind: in a compliant step, the geometry of the thirteen contacts and their force
laws; in a non-smooth step, the projected Gauss–Seidel sweeps over the contacts. A
general-purpose 3D engine pays more: six degrees of freedom per body instead of
one, mesh-based collision detection instead of three analytic primitives, and, in
Thorin's simulations, the stiffness and viscosity of a linear element readjusted at
every step to emulate the nonlinear felt law [V, Thorin 2017 §4].

**Do not use** the hand-counted costs of a kernel never written (2,500 flops per
penalty step, 45,000 per non-smooth step), the Mflop/s per note built on them (125
at a 20 µs step, 90 at 500 µs), the per-thread throughputs they were divided by, or
the frame-time tables that follow: the steps behind them are those refuted in [Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step),
and annex A1's 108 ns per step and 125 Mflop/s per note cannot both hold on one
scalar core [E, 06]. Nor read Thorin's 20 seconds per simulated second as
2·10⁷ flops per step for a dedicated 2D solver to beat: that time is an
implementation's, a 3D engine readjusting its felt law at each step, and Thorin
gives the gains he expects from removing those two causes, more than 15 and more
than 3 [V, Thorin 2013 §6.4.1, p. 110].

### How many notes are really active?

The mechanical activity window of a note is about **300 ms** [S] (attack about
150 ms, then release, jack re-engagement and damper return about 150 ms). During
the hold the action is **static** (hammer in the back check, key at the bottom,
jack escaped): it can be frozen. At an assumed onset density, the notes moving at
once are that density times 0.3 s [S]:

| density | context | concurrent notes |
|---|---|---|
| 6 onsets/s | Mozart allegro | 1.8 |
| 12 onsets/s | Chopin étude | 3.6 |
| 20 onsets/s | Liszt / Alkan | 6.0 |
| 32 onsets/s | burst, 8-note chords × 4/s | 9.6 |

The densities are assumptions, to be replaced by counts over a corpus of MIDI files
([FOUNDATIONS > The 88-note budget](../FOUNDATIONS.md#the-88-note-budget)).

The 88 dampers move together only during a pedal change, and each has one degree of
freedom. The notes still share nothing: the damper lift tray is a prescribed
position, the trapwork (one to three degrees of freedom driven by timestamped pedal
input) integrated identically in every worker ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)). The sostenuto is
a latch that keeps the dampers raised when it is pressed. The una corda shifts the
key frame and the hammers, not the dampers; it is the only motion out of the plane,
and it adds no planar physics ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)).

### Is WASM NECESSARY? Arguments, not throughput

On the throughput criterion alone, nothing in the corpus decides between
TypeScript and WebAssembly for one note: the cost per step is the prototype's to
measure ([What a step costs](#what-a-step-costs)), and the published comparisons of JavaScript and WebAssembly on
numerical kernels disagree with one another; none is in the corpus. The domain is
Rust compiled to WebAssembly for reasons that are not speed ([FOUNDATIONS > Rust compiled to WebAssembly; notes on independent workers](../FOUNDATIONS.md#rust-compiled-to-webassembly-notes-on-independent-workers),
built):

1. **Determinism of the transcendentals.** `Math.pow` and `Math.exp` are
   "implementation-approximated" in ECMA-262: the specification does not mandate an
   algorithm, and engines differ. `sqrt` is exact under IEEE-754 on both sides.
   Every transcendental in the domain therefore goes through the `libm` crate,
   `pow` included, which makes `pow` as reproducible as `sin` ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).
   **Do not use** "avoid `pow` in the hot loop for determinism", nor its corollary
   that Stulov's `F0·u^p` with a non-integer `p`, or Thorin's `k·δ^r` with
   r = 2.7, is disqualified: a non-integer power is allowed, and a law is chosen on
   physics and measured cost ([FOUNDATIONS > The felt law](../FOUNDATIONS.md#the-felt-law)).
2. **Absence of a garbage collector, structurally.** In JS one can avoid the GC
   (TypedArray and pooling), but one accidental allocation in the stepping loop
   brings back an unpredictable pause. In Rust, "zero allocation in `step()`" can be
   checked at review.
3. **Explicit SIMD, an option and not one of the reasons.** JS has no SIMD (SIMD.js
   is dead); WASM has v128, and since the notes are independent, "inter-note" SIMD
   (two notes as f64x2) avoids shuffles and horizontal reductions, branches
   becoming masked selects, double precision kept ([FOUNDATIONS > A dedicated planar solver, not a general-purpose engine](../FOUNDATIONS.md#a-dedicated-planar-solver-not-a-general-purpose-engine)). It is a speed
   argument, so whether it pays is the prototype's to measure ([FOUNDATIONS > The 88-note budget](../FOUNDATIONS.md#the-88-note-budget)).
4. **Control of the time step and the order of operations.** Possible in both, but
   easier to keep in a strictly typed language without implicit coercions.
5. **Native runs of the code that ships**, for studies, sweeps and golden tests
   ([FOUNDATIONS > Rust compiled to WebAssembly; notes on independent workers](../FOUNDATIONS.md#rust-compiled-to-webassembly-notes-on-independent-workers)).

### Deployment constraints not to discover too late
- **Baseline SIMD** (v128): Chrome 91+, Firefox 89+, Safari 16.4+, Edge 91+.
- **Relaxed SIMD**: never ([Determinism and reproducibility](#determinism-and-reproducibility)).
- **WebAssembly threads**: not used. On the pinned Rust toolchain, linking `pa-wasm`
  with shared memory fails without a nightly rebuild of the standard library, so the
  88-note runtime is one module per Web Worker ([FOUNDATIONS > Rust compiled to WebAssembly; notes on independent workers](../FOUNDATIONS.md#rust-compiled-to-webassembly-notes-on-independent-workers)). Transport is a
  lock-free ring in a `SharedArrayBuffer` where the page is cross-origin isolated
  (COOP/COEP headers, `crossOriginIsolated` checked at runtime), transferable
  buffers otherwise ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)).

---

## Determinism and reproducibility

**Why it matters here, concretely**: a technician regulates a let-off in the
simulator, measures 1.2 mm, changes a parameter, goes back, and expects to find
1.2 mm again, not 1.19 or 1.21. And a recorded session must replay identically on
a colleague's machine. Without that, the tool is not a measuring instrument; it is
an animation.

The arithmetic is identical bit for bit on every machine when the conditions of
[FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly) hold. Built: no `HashMap` in the domain's production code;
every transcendental there through the `libm` crate, Clippy refusing the
standard library's; and two fixed bisections that resolve far below anything
read ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)). Checklist:

1. **A fixed time step on the real-time path.** No `dt` derived from
   `requestAnimationFrame`: a fixed-step accumulator, the renderer interpolating
   between snapshots stamped in simulation time ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)). The reasons are
   the frame cadence and a cost per step that is bounded, not reproducibility: with
   identical operations in the same order, a variable step reproduces too
   ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).
2. **Iteration counts.** Convergence-tested loops are allowed, with a hard
   iteration cap and the iteration count reported, and they are required in the
   metrological paths (the quasi-static solver, the inverse regulation solve, the
   protocols), where an unconverged answer is worse than a slow one. Fixed iteration
   budgets belong to the real-time path, where the cost per step must be bounded;
   they also keep outputs smooth in the parameters, as finite-difference
   sensitivities need. A residual is reported as a diagnostic either way. **Do not
   use** the argument that a convergence test makes the result depend on rounding
   and destroys reproducibility: with identical operations in the same order it
   does not, and with different arithmetic a fixed count would not restore it
   either ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).
3. **A frozen order of operations.** Contacts iterated in an order fixed by
   construction, not by insertion into a container; reductions (sums of torques) in
   a fixed order, never in parallel without a deterministic reduction tree. In Rust,
   never iterate a `HashMap` in the solver (randomized order): use a `Vec`.
4. **Strict IEEE-754 arithmetic.** WASM guarantees IEEE-754 round-to-nearest-even
   semantics for `f32`/`f64` (no extended-precision x87, no flush-to-zero). No
   `-ffast-math` or any equivalent, no contraction of a multiply and an add into one
   fused operation; in Rust, none of the `fast_*` intrinsics. Rust guarantees both on
   x86-64 and wasm32 ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).
5. **NEVER relaxed SIMD.** `qfma` rounds **once** on FMA hardware and **twice** on
   hardware without FMA. The non-determinism is bounded to the instruction and
   stable on a given machine, but **it varies across hardware**. So:
   **baseline v128 only, never `-mrelaxed-simd`** ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)).
6. **No platform transcendentals.** Neither `Math.pow` nor `Math.exp` in JS, nor
   `f64::powf` or `sin` backed by a system library: every transcendental in the
   domain goes through the `libm` crate, `pow` included ([FOUNDATIONS > Determinism, stated correctly](../FOUNDATIONS.md#determinism-stated-correctly)). `sqrt`
   is safe (exact under IEEE-754).
7. **No GC.** Zero allocation in `step()`.
8. **Explicit seeds.** No implicit random number generator. Any variability (note
   to note scatter of regulation, play in the mortises) is an explicit PRNG, seeded,
   the seed saved with the scene.
9. **Serialization of the complete state**, the hidden variables included: felt
   hysteresis memory, the stick or slip state of each friction set, contact flags.
   A "position + velocity" state is NOT enough to replay a session ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)).
10. **Several workers without non-determinism.** The notes are independent: each
    Web Worker runs its own module and notes, all synchronized at the frame, never
    at each step. One simulation-time master gives every worker the same target
    step; reductions across notes run in a fixed note order, in one place. Which
    worker computes a note may vary; the order of reductions may not ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)).

A regression test (planned): a **hash** of the complete state after N steps on a
canonical scenario, compared across machines and across Chrome, Firefox and Safari,
with finiteness asserted before hashing so that a `NaN` does not pass for a physics
regression. If the hash diverges, the divergence is found before it poisons a
measurement.

---

## Key excitation

### Three modes

Planned, with the dynamics ([FOUNDATIONS > What the dynamics prototype settles, and what it leaves open](../FOUNDATIONS.md#what-the-dynamics-prototype-settles-and-what-it-leaves-open), [FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)).

**(i) FORCE-driven: realistic playing.** Finger-key forces measured by Askenfelt
and Jansson (1992), as Goebl et al. report them [V, Goebl et al. 2005, p. 1156,
citing Askenfelt & Jansson 1992b, p. 345]:
- **mezzo forte staccato** attack: typically **15 N**
- very loud such attacks: **peaks up to 50 N (fortissimo)**
- very soft touches: **as low as 8 N (piano)**
- **legato** touch: **about a third of staccato**, usually with a peak when the key
  reaches the keybed
- **pianissimo**: "the force hardly exceeds 0.5 N"

Profile shape: a rapid rise, then a plateau or a decay; for struck touch, an
initial finger-percussion spike. Goebl and Bresin note that "especially in very
loud staccato tones, the first impact of the finger hitting the key resulted in a
very high-peak hammer velocity which decreases significantly until hammer–string
contact" [V, Goebl & Bresin 2003, p. 2278]: the maximum hammer velocity is NOT the
striking velocity, which matters for calibration.

**(ii) IMPOSED DISPLACEMENT: regulation measurements and validation.** Thorin's
central validation argument: position-driven simulations "prove far more reliable
than the usual force-driven simulations" [V, Thorin 2017 §6], because a force input
hides the model's defects behind the intrinsic smoothness of the resulting motion.
Put another way: **excite a bad model in force and the displacement will not show
it; excite it in position and compare the measured force, and it shows at once**
([FOUNDATIONS > What a test proves; drive in position, compare forces](../FOUNDATIONS.md#what-a-test-proves-drive-in-position-compare-forces)). An implementation detail to anticipate: XDE performed
force-driven simulations directly, and position-driven ones by inserting a
proportional-derivative corrector, mirrored in the experiment by a viscoelastic
piston between finger and key [V, Thorin 2017 §4, §3]. **In an in-house solver,
position drive is native**: a bilateral constraint on `θ_k` (or an additional
reaction unknown in the one-step problem), provided for in the solver's interface
from the start.

**(iii) QUASI-STATIC: down weight and up weight.** The static limit of the dynamic
solver ([Solver choice: the options and the hypothesis](#solver-choice-the-options-and-the-hypothesis)). Stanwood's protocol to reproduce [V, Stanwood 2000, glossary,
p. 22–23]: **D**, "the minimum amount of weight, to the nearest gram, placed on the
measuring point that causes the key to drop while maintaining a slow controlled
motion of the hammer"; **U**, "the maximum amount of weight, to the nearest gram,
placed on the measuring point that the key can lift while maintaining a slow,
controlled motion of the hammer".

### MIDI velocity 0–127 mapping

Anchors:

| Point | Hammer velocity | Source |
|---|---|---|
| MIDI 77 | **2 m/s** | [V, Goebl & Bresin 2003, p. 2276] |
| forte | **about 5 m/s** (FHV), key travel to the keybed in **25 ms** | [V, Goebl et al. 2005, p. 1154, citing Askenfelt & Jansson 1991, p. 2385] |
| piano | **about 1 m/s** (FHV), travel in **160 ms** | ibid. |
| loudest attacks measured | **Yamaha: C1 6.4 m/s, G6 7.8 m/s; Steinway: C1 6.8, G6 7.5 m/s; Bösendorfer: C1 6.0, G6 6.6, C4 7.6 m/s** | [V, Goebl et al. 2005, p. 1160] |
| lower threshold | 0.2–0.3 m/s, below which the hammer no longer reaches the string | [S], inferred from free flight |
| velocity at let-off | **piano 1.01 m/s; forte 1.56 m/s** (measured; simulated 1.00 and 1.56) | [V, Thorin 2017, Table 5] |

A useful operational definition: the Bösendorfer SE measures the average velocity
over the last 5 mm of the hammer's travel to the strings, "approximately the last
10% of that distance" [V, Goebl & Bresin 2003, p. 2278], so a total travel of about
50 mm, consistent with a blow distance of 45–47 mm; it stores it inverted,
`FHV [m/s] = 128 / IHV`, the counter running at 25.6 kHz [V, Goebl & Bresin 2003,
p. 2273]. Check: 0.005 m / (IHV/25600 s) = 128/IHV. A measurement definition usable
as it stands as a simulator observable.

**Why NOT to impose a hammer velocity**: the hammer velocity is the RESULT that
regulation is supposed to influence. Impose it, and changing the let-off or the
blow distance changes nothing in the sound, and the tool's teaching value is lost
([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)).

**Procedure**: impose a parameterized force profile (amplitude × shape) and find,
by bisection on the model, the amplitude that gives each note and velocity: lazily,
warm-started, and again when the regulation changes ([FOUNDATIONS > The 88-note runtime](../FOUNDATIONS.md#the-88-note-runtime)). The side
effect is the point: **a badly regulated let-off becomes audible and visible**,
since the forces change. A full 88 × 128 table is 11,264 entries, each a bisection
of several simulations of about 0.3 s, which is why the entries are computed as
they are needed.

On the analytic form: no simple function passes through the anchors (1; 0.2),
(77; 2.0) and (127; 5.5), the first and last assumed [S]. An exponential fitted on
the first two gives 9.1 m/s at 127 (too high); a power law `v = 5.5·(m/127)^2.02`
fitted on the last two gives 0.00031 m/s at m = 1 (absurd). Hence an interpolated
monotone table (PCHIP on `log v` against `m`) rather than a formula, and in any case
the table calibrated on the model is better.

### Timing checks to reproduce (quantified validation targets)
- Key travel to the keybed: **25 ms** at forte (5 m/s) → **160 ms** at piano
  (1 m/s) [V, Goebl et al. 2005, p. 1154, citing Askenfelt & Jansson 1991].
- **Hammer–string contact 12 ms BEFORE the keybed at piano, 3 ms AFTER it at
  forte** [V, Goebl et al. 2005, p. 1154, citing Askenfelt & Jansson 1990]: a
  discriminating test, because it changes SIGN and brings aftertouch into play.
- **Free flight** (let-off to string): from almost zero at loud dynamics up to
  **20 ms** at very soft ones, with outliers around 40 ms on the Yamaha [V, Goebl et
  al. 2005, p. 1162].
- Travel time (finger to string): **20 ms to about 200 ms** (up to 230 ms on the
  Steinway) [V, Goebl et al. 2005, p. 1160]; on the Bösendorfer the power-law fits
  are `tt_pressed = 89.96·HV^(−0.5595)` and `tt_struck = 58.39·HV^(−0.7377)` (ms, HV
  in m/s) [V, Goebl et al. 2005, Figure 6].

---

## Quantified validation plan

The levels are [FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)'s. A **consistency check** shows the model agrees
with itself or with mathematics, and a wrong model passes it; a **validation test**
predicts outputs measured independently from inputs measured independently, on the
same instrument ([FOUNDATIONS > What a test proves; drive in position, compare forces](../FOUNDATIONS.md#what-a-test-proves-drive-in-position-compare-forces)). Levels 0, 1, 2 and 5 are consistency; level 3
is validation; level 4 is consistency, and validation where the measurement is
independent. Built: the level-0 lever-arm, virtual-work and key-ratio tests, the
level-4 protocols of the weighing sheet (the jig, the seated pose, the breakaway
walk, both key friction weights), and the non-confusion tests; the rest is
planned ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)).

### Level 0 — Analytic tests (unit tests, exact)
- **Lever arms**: replace the profiles with rigid arms of known lengths and check
  that the computed `∂g/∂θ` equals the analytic value to 1e-12; check every lever
  arm against central finite differences of the gap.
- **Contact primitives**: circle/line, circle/circle, hybrid: check `g`, `n`, `x_c`,
  `∂g/∂θ` against configurations solved by hand, and the **C1 continuity** of the
  normal at the arc-to-line transition of the hybrid profile (a finite difference of
  `n` on either side).
- **Mass matrix**: `M_jr = 0` exactly; symmetry; `M` against finite differences of
  the kinetic energy (a positivity test alone passes on wrong couplings, [FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever)); the arrowhead solve against a dense LU to 1e-14.
- **Virtual work**: the static force ratio equals the displacement ratio (a Jacobian
  consistency test, which catches most sign and lever-arm errors); the net torque
  against the gradient of the potential.

### Level 1 — Time-step convergence
The same scenario at several steps, the key force and the observables (the let-off
instant, the hammer's speed at let-off, drop, checking) compared until they stop
changing by more than their resolution; the step is fixed there ([FOUNDATIONS > The time step](../FOUNDATIONS.md#the-time-step)).
Thorin's study is the model for it: 0.4, 2 and 10 kHz, the force on the key very
similar at 2 and 10 kHz and significantly off at 0.4 kHz [V, Thorin 2013 §6.4.1,
Figure 6.10; Thorin 2017, Figure 12]. A difference that persists as the step falls
points at a hidden regularization parameter or a convergence criterion that varies
with the step. **Do not use** "convergence toward 5–10 µs, visible degradation
beyond 20–40 µs, instability beyond about 80 µs (2/ω of the jack/knuckle contact)":
those figures come from the per-contact table and the stability limit refuted in
[Contact stiffnesses, linearized modes, and the step](#contact-stiffnesses-linearized-modes-and-the-step).

### Level 2 — Energy in free flight
Contacts and friction off, the hammer released from a given angle: `E = ½ I_h θ̇² +
m_h g h` stays bounded and oscillates, without monotonic drift, under a symplectic
integrator. For the (wippen, jack, lever) block, whose mass matrix depends on its
configuration, the semi-implicit step is not symplectic for that Hamiltonian, and
the test is a drift falling as O(dt) ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)); it checks the Coriolis
terms of the subtree.

### Level 3 — Comparison with published measurements (validation)
- **Velocity at let-off**, Thorin's demonstrator: piano **1.01 m/s** measured,
  **1.00** simulated; forte **1.56** and **1.56** [V, Thorin 2017, Table 5],
  reproduced within the measured scatter ([FOUNDATIONS > Stages](../FOUNDATIONS.md#stages)).
- **Forces on the key under position drive**: "The comparisons between measured and
  simulated forces in response to a given motion are successful" [V, Thorin 2017,
  abstract; Figures 8–9]. This is THE discriminating test ([FOUNDATIONS > Validation instruments](../FOUNDATIONS.md#validation-instruments), "Thorin's Renner demonstrator").
- Angular trajectories: Thorin's five bodies by KLT tracking on a high-speed camera
  [V, Thorin 2017, Figure 10]; Hirschkorn's key, wippen and hammer by optical
  encoders under force drive (forte, piano, double blow), the jack and the lever
  filmed at high speed but not compared [V, Hirschkorn 2004 §3.2.2, ch. 4].
- **Goebl's timing relations** ([Timing checks to reproduce (quantified validation targets)](#timing-checks-to-reproduce-quantified-validation-targets)), in particular the **change of sign** of the
  offset between hammer–string contact and key bottom from piano to forte.
- **Hammer–string contact duration**, if the string is modeled: **[?]** the
  figures (about 4 ms in the bass, under 1 ms in the treble) are Askenfelt and
  Jansson's, not in the corpus ([Hammer–string impact](#hammerstring-impact)).

### Level 4 — The simulator measures ITS OWN regulation
This is the level that counts for the technician, and it exists in no publication.
The simulator implements **the technician's measuring procedures**, not ad hoc
numerical probes ([FOUNDATIONS > The acceptance criterion](../FOUNDATIONS.md#the-acceptance-criterion)). Hirschkorn regulates the model and the action
by the same procedure, which is the right spirit [V, Hirschkorn 2004 §2.3]:

| Step | Hirschkorn's specification [V, Hirschkorn 2004 §2.3] | Published trade values |
|---|---|---|
| 1. Key dip | key front travel **10.0 mm** | Steinway: **.400"** at the factory, within **.390–.420"** [V, Steinway technical reference, ch. 2, p. 9]; Steinway Hamburg **9.9–10.6 mm**; generally 9.3 to 10.6 mm [V, regulation specification sheet, pp. 1–2] |
| 2. Hammer height at rest (set at the capstan) | **48.0 mm** | blow distance **1¾"** on Steinway models S, M, L, O, A, B and **1⅞"** on C, D [V, Steinway technical reference, ch. 2, p. 6]; Steinway Hamburg **47 mm**; generally 42 to 47 mm [V, regulation specification sheet, pp. 1–2] |
| 3. Jack at rest | right side of the jack aligned with the right edge of the knuckle | — |
| 4. Repetition lever at rest | same height as the jack under the knuckle | — |
| 5. **Let-off** | the jack escapes when the hammer height is **1.0 mm** | Steinway New York: between **1/32" and 1/16"** from the strings [V, Steinway technical reference, ch. 2, p. 7]; Steinway Hamburg **1.5 mm** bass, **1 mm** middle and treble; generally **3 to 1.5 mm** from bass to treble [V, regulation specification sheet, pp. 1–2] |
| 6. **Drop** | hammer height **3.0 mm** resting on the repetition lever after let-off | Steinway New York: no more than **1/16"** below let-off [V, Steinway technical reference, ch. 2, p. 8]; Steinway Hamburg **2.0 mm**, also a fall below let-off [V, Steinway technical reference, ch. 1, p. 30; regulation specification sheet, p. 2]; Yamaha let-off plus 2.0 mm, from the string [V, regulation specification sheet, p. 4] |
| 7. **Back check** | under a medium blow, hammer stopped at **15.0 mm** | Steinway New York ½" [V, Steinway technical reference, ch. 2, p. 13]; Steinway Hamburg **15 mm**; generally **16 to 12.7 mm** [V, regulation specification sheet, pp. 1–2] |
| Aftertouch | — | Steinway New York **3/64" to 1/16"** [V, Steinway technical reference, ch. 2, p. 11], or 0.04–0.055" [V, regulation specification sheet, p. 2]; Steinway Hamburg **0.8–1.2 mm** [V, regulation specification sheet, p. 2] |

(The Steinway technical reference is `manuels/sw_chp1.pdf` and
`manuels/steinway_chp2.pdf`, its chapters 1 and 2; the regulation
specification sheet is `manuels/regspecs.pdf`, "Grand Piano Regulation
Specifications".) **Do not use** the "BYU Design Review" regulation values (a blow
distance of 45 ± 2 mm, a let-off of 1.5–2 mm, an aftertouch of 1.25–1.5 mm): they
trace to no source in the corpus.

And the **touchweight** measurements, a first-class feature in Stanwood's
vocabulary [V, Stanwood 1996, p. 2; Stanwood 2000, glossary, pp. 22–23; Stanwood,
"Down Weight versus Equation Piano Key Balancing" (2014), Appendix A]:

```
Balance equation:        BW + FW = (WW × KR) + (SW × R)
BW  = (D + U)/2      Balance Weight
F   = (D − U)/2      Friction Weight
SW                   Strike Weight: shank and hammer pivoted at the hammer center, shank level, measured at the strike line radius
WW                   Wippen Weight: the level wippen pivoted at its center, measured at the capstan/heel contact point
KR                   Key Weight Ratio: the weight at the measuring point that balances 1.0 g at the capstan/heel contact point
R   = SBW/SW         Strike Weight Ratio
SBW = TBW − WBW      TBW = BW + FW ;  WBW = KR × WW
```

KR is read with 10 g at the capstan: a scale reading of −5.5 at the front gives
KR = 0.55 [V, Stanwood 2000, p. 22]. **Do not read** the glossary's first wording,
"the ratio of downward force on the capstan/heel versus the corresponding upward
force at the measuring point", as capstan force over front force: its own second
wording and the worked example give front over capstan, about 0.5 to 0.6, the
convention of the equation of balance ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)).

Stanwood's reference values [V, Stanwood, "Looking at Grand Pianos Through the Eyes
of the New Touchweight Metrology", PTJ March 2000, pp. 21–25, Tables I, III and IV]:
the normal **strike weight zone**, delineated at note 1 by 8.0 / 10.3 / 12.4 /
14.8 g, at note 45 by 6.5 / 8.3 / 10.1 / 12.0 g, at note 52 by 6.1 / 7.9 / 9.6 /
11.3 g and at note 88 by 3.5 / 4.6 / 5.8 / 6.9 g; a generic **shank strike weight**
of **1.7 g**; a normal **strike weight ratio** zone of **5.0 to 7.0**, divided into
high 7.0–6.3 (low leverage), medium 6.3–5.7 and low 5.7–5.0 (high leverage); a
**front weight ceiling** of **41.3 g** at note 1, 35.5 at note 23, **28.1** at note
45, 25.1 at note 52, 17.4 at note 67 and **3.7 g** at note 88. Perceptual
sensitivity: **1.0 g more strike weight** felt "like a different piano", **0.5 g**
perceived "even by the average amateur pianist"; widening the hammer by about
**1.6 mm** adds about **1.0 g** of strike weight (about 0.6 g/mm). Hammers hung
**125 mm** out on the shank in the bass give R = **5.5**, at **130 mm** in the
treble R = **6.1**; knuckles **15.5 mm** against **17 mm** from the center pin give
different ratio levels.

**A check on the convention, not on the model.** `KR` has two reciprocal
conventions, and the equation of balance takes only one: the **force** convention,
the front reading for a weight at the capstan, about 0.50 to 0.60 on most actions
and 0.68 on an 1878 Steinway concert grand ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs); Stanwood's example
reads 0.57 [V, Stanwood 1996, p. 5]). The **geometric** convention, front arm
over capstan arm, "nominally 2:1" [V, Stanwood, pianotech mailing list,
`stanwood/mailling-list/swr.txt`], is its reciprocal. On Hirschkorn's key, with the
capstan 117.6 mm behind the balance pin and the front rail punching 217.5 mm in
front of it [V, Hirschkorn 2004, Table A.3], the geometric figure is
217.5/117.6 = 1.85 and the force one 117.6/217.5 = 0.54, at the punching rather
than at Stanwood's measuring point. Put into the equation, the geometric convention
inflates the wippen term by 1/KR², about 3.3, and makes a wippen balance weight
larger than the wippen weight, a reducing lever appearing to amplify: the error the
engine's level-0 tests refuse with `WBW < WW` and `0.3 < KR < 1.0`, a window that
separates the two conventions without rejecting a real action ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)).

**Do not use 5.84** as a closed test of the action ratio, nor "19.06 under the
inverted convention" as proof that it discriminates: that lever product multiplies
radial distances, takes `Pw2` for the wippen's pivot and the jack's pin for its
contact, uses the front punching as the key's front arm instead of the measuring
point, and a band of 5.0 to 7.0 could not see a 10 % error in an arm anyway
([FOUNDATIONS > Hirschkorn's frames](../FOUNDATIONS.md#hirschkorns-frames)). The action ratio is read off an assembled model, by virtual
work. **Do not use** the front-weight "closure" built on it either (a posited BW of
35 g landing on 24.8 g against a ceiling of 24.7 g): it uses that 5.84, a wippen
weight no source gives for this action, and an interpolated ceiling where
Stanwood's Table IV gives 25.1 g at note 52.

Two traps when reading these bounds. They are read **at the note the data
describes**: Hirschkorn's action is note 52, so note-1 figures (an 8.0–14.8 g strike
weight zone, a 41.3 g front weight ceiling) do not apply to it. And **WW for
Hirschkorn's action is not published**: Stanwood weighs the level wippen at the
capstan/heel contact point, and on Hirschkorn's data that is read off the
assembled model, every frame at its own pivot ([Structure of the mass matrix](#structure-of-the-mass-matrix)).

The corpus supports a narrower claim than cross-validation, and a useful one:
Hirschkorn's parameter set, Stulov's felt model, the Steinway and Renner
regulation values and Stanwood's metrology are expressed in compatible quantities,
and Hirschkorn's action, assembled with its frames read correctly, closes at rest
with the hammer 47.0 mm from the string against the 48.0 mm he regulated [E, 10].
**Levels 0 and 4 can be written before the solver.**

### Level 5 — Sensitivities
For each regulation parameter, the change it produces in the observables, by
finite differences ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)). Questions the tool must answer:
- capstan height `+0.1 mm` → what change in hammer height, let-off, aftertouch, D,
  U? (**Do not use** "+0.1 mm of capstan ⇒ −1.4 g of down weight and −0.3 mm of
  aftertouch": it comes from a display mock-up, and raising the capstan increases
  aftertouch; [FOUNDATIONS > Levels](../FOUNDATIONS.md#levels).)
- let-off button `+0.1 mm` → change in the let-off and in the velocity at let-off?
- repetition spring force `+10 %` → change in U and in the re-engagement time?
- the key's front weight `+1 g`, its mass the same → change in BW (and not in F)?
- pin friction `×1.2` → change in F = (D−U)/2 (and not in BW)?

The last two are **non-confusion** tests: a change of mass must move BW without
touching F; a change of friction must move F without touching BW. A technician
detects at once a model that mixes the two, where a developer would never see it.
With a constant friction bound both hold but for the tilt between the key's two
seats, where D and U are read, a tolerance derived from the seats' arms; they
guard against coding mix-ups until friction depends on load ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)).
Settings are not optimized by gradient descent: Thorin's cost function is convex
along a few parameters and flat along many [V, Thorin 2013 §6.5.6]; a scripted
regulation sequence or a grid search does the job ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)).
