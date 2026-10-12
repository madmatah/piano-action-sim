> Translated from the original French research annex. Part of the `piano-action-sim` foundations
> study (October 2026). See `docs/FOUNDATIONS.md` for the decisions this annex supports.

RESEARCH: SCIENTIFIC LITERATURE ON GRAND PIANO ACTION DYNAMICS
=========================================================================================

Method: I downloaded and worked through the primary PDFs in full (not just the abstracts).
Every number below marked **[V]** is VERIFIED, extracted by me from the source document.
Marked **[D]** = DERIVED/CALCULATED by me (calculation reproduced and verified).
Marked **[S]** = SUPPOSED / to be confirmed.

Sources, in `data/references/` — gitignored, third-party copyright, local only:
```
data/references/papers/
  hirschkorn_thesis.pdf    Waterloo MASc thesis — the complete parameter set
  thorin_thesis_2013.pdf   Polytechnique PhD thesis
  thorin2017.pdf           Meccanica article
  lozada_thesis.pdf        primary source of Thorin's parameters
  masoudi_preprint.pdf     micromechanical model of hysteretic felt
  stulov_actaa2005.pdf     hammer felt, hereditary law
  goebl2005.pdf            experimental validation data
  stanwood2014.pdf
  stanwood_blueprint.pdf   touchweight metrology
data/references/extracts/
  h.txt · m.txt · s.txt    text extractions
```

---

## 1. Overview of the published models (synthesis, after Thorin 2017 Table 2 + my own reading)

| Work | Input | Output | DOF | Numerical method | Usable? |
|---|---|---|---|---|---|
| Dijksterhuis 1965 | force | — | — | — | no (kinematic) |
| Oledzki 1973 | force while running | displ., vel. (10 points) | 2 | 2 ODE | no |
| Hayashi 1999 | constant vel./accel. | hammer displ. | 2 | closed form | no |
| Van den Berghe 1995 | force profile | key+hammer displ. | 3 | DYNAST | no |
| Oboe 2006 | measured force | hammer displ. | 2 | — | no |
| Gillespie 2011 | measured force | hammer+key displ. | 5 | MATLAB Simulink/StateFlow | partial (regularising rubber) |
| **Hirschkorn 2004/2006** | **measured force** | **hammer/key/wippen displ.** | **5** | **DynaFlexPro → MATLAB ODE15s** | **YES — complete parameter set** |
| Izadbakhsh 2006/2008 | measured force | + flexible shank | 5 + Rayleigh beam | DynaFlexPro → Maple lsode | extension |
| Links 2011 | force | kinematics | 5 | index-3 DAE → ODE | — |
| Bokiau 2012 / Fisette 2019 | force profile | kinematics | 6 | ROBOTRAN (relative coord.) | YES (pedagogical) |
| Masoudi & Birkett 2015 | measured force | displ. | 5 | MATLAB ODE15s | upright only |
| **Thorin 2013/2017** | **force AND position** | **FORCES + kinematics** | **5 (+1 damper)** | **MDI / Moreau-Jean, XDE** | **YES — REFERENCE** |
| Timmermans 2020 | real time | haptic force | 5 | symbolic ROBOTRAN + C | YES (real-time proof) |

---

## 2. RECOMMENDED REFERENCE MODEL: Thorin et al. 2017

**Exact citation [V]**: A. Thorin, X. Boutillon, J. Lozada, X. Merlhiot, "Non-smooth dynamics for an efficient simulation of the grand piano action", **Meccanica** 52, 2837-2854 (2017), DOI 10.1007/s11012-017-0641-1, received 09/07/2016, accepted 17/02/2017.

### 2.1 Breakdown into bodies and DOF [V]

5 planar rigid bodies + 1 (damper, treated in the thesis but removed from the article):

| Body | Coordinate | Axis of rotation | Remark |
|---|---|---|---|
| Key | θ_K | fixed to the frame | in reality **2 DOF** (rotation + vertical translation on the balance rail pin) |
| Wippen | θ_W | fixed to the frame | |
| Jack | θ_J | **carried by the wippen** | non-inertial frame |
| Repetition lever | θ_L | **carried by the wippen** | non-inertial frame |
| Hammer | θ_H | fixed to the frame | |
| Damper | θ_D | fixed to the frame | thesis ch. 4.5.7 |

Choice of coordinates: **absolute angles per body** (1 DOF each), with the jack and lever frames attached to the wippen (double-pendulum analogy, demonstrated in Appendix B of the thesis). This is the architecture to copy: the kinematic tree is trivial, and there is **no joint between the bodies apart from the contacts**.

**Crucial point confirmed by the literature [V]**: Thorin blocks the vertical translation of the key (replacing the balance-rail felt with a metal ring of the same thickness) because the 2 DOF made the experiments irreproducible. He admits that "*such a modification would not be acceptable in practice (it makes the action fairly noisy)*". Hirschkorn makes the same observation the other way round: the key lifts off the balance rail, which "*allows motion in the key that is not permitted by the perfect revolute joint in the model*". **Both groups report this same modelling gap.**

### 2.2 EXHAUSTIVE list of contacts [V]

Thorin, 14 named contacts (thesis Table 4.2):

| Name | Body A | Body B | Tangential friction |
|---|---|---|---|
| KS+ | key | frame at the key front (front rail punching) | no |
| KW | key (capstan) | wippen | no |
| KS− | key | frame at rest (back rail cloth) | no |
| KH | key (back check) | hammer | **YES (ν_KH)** |
| KD | key (spoon) | damper | no |
| WJ | wippen | jack | no |
| WL | wippen | repetition lever | no |
| JS | jack | frame at the let-off button | **YES (ν_JS)** |
| JL | jack | repetition lever | no |
| JH | jack | hammer knuckle (knuckle/roller) | **YES (ν_JH)** |
| LS | lever | frame at the drop screw | no |
| LH | lever | hammer knuckle | no |
| HS | hammer | frame (string) | no |
| DS | damper | frame at rest | no |

Hirschkorn, 13 contacts (Table 2.1) — same topology, without the damper, but with the two key stops in addition:
`1 key-wippen at the capstan (circle-circle) | 2 key-frame at the front (line-circle) | 3 key-frame at the rear | 4 key-hammer at the back check | 5 wippen-jack | 6 wippen-lever | 7 jack-hammer at the knuckle (hybrid-circle) | 8 jack-frame at the button | 9 jack-lever | 10 lever-hammer at the knuckle (hybrid-circle) | 11 lever-frame at the stop | 12 hammer-frame at the string | 13 hammer-frame at rest`

Tangential friction is retained only on the contacts with significant sliding: **1, 4, 7, 10** for Hirschkorn; **KH, JH, JS** for Thorin. Thorin's justification [V]: *"Friction between the jack and the roller is easily felt by the pianist and must also be considered in the model, because the piano technicians adjust it carefully."*

**Contact geometries** — three primitives are enough (the two authors converge):
- **circle** (centre in polar coordinates ρ, θ from the centre of the contact zone P, + radius)
- **segment / line** (angle θ in the body frame + length λ)
- **hybrid** = arc + two tangent straight lines (for the jack nose and the lever under the knuckle)

Thorin's breakdown [V]: back check-hammer = 2 circles; jack-knuckle = 2 perpendicular straight lines joined by an arc; **the other 10 = 1 circle + 1 segment**. Hirschkorn's trick [V]: the felt under the key front is physically a flat horizontal disc, but "*treating it as a circular contact can still produce the correct penetration depth*" — the geometry only serves detection and penetration depth, not the force law.

---

## 3. FELT / LEATHER FORCE LAWS — the two families

### 3.1 Thorin / Brenon — three-parameter visco-elastic law [V]

```
F_felt(δ) = k·δ^r + b·δ²·δ̇            (Thorin 2017 Eq. 6 ; thesis Eq. 2.3)
```
δ = felt compression (m), δ̇ = compression velocity. Experimental source: C. Brenon, *Mécanique de la touche de piano*, dissertation, Université Pierre et Marie Curie Paris VI, 2002; re-measured by Lozada 2007.

Published values **[V]** (Thorin thesis Eqs. 4.9/4.10):

| Felt | k | r | b |
|---|---|---|---|
| **hammer knuckle (knuckle, JH) and lever-frame** | 7 × 10⁹ SI | **3.0** | 5 × 10⁷ SI |
| **all other felts** | 1.6 × 10¹⁰ SI | **2.7** | 5 × 10⁷ SI |

(unit of k: N·m^−r; of b: N·s·m⁻³ — Thorin writes "SI unit"; Table 2.2 gives b = 2 × 10⁹ N·s·m⁻³ for the key-front felt *after* an arbitrary ×100 increase to absorb the dissipation of the back check, NOT to be taken over as such.)

Energy balance verified by the author **[V]**: for 1 mm of compression applied linearly over 0.1 s, W_elastic / W_viscous = **22**. The viscous share is therefore not negligible, but remains secondary.

Contact implementation **[V]**: each felt is a **small 1 mg auxiliary rigid body "A"**, connected to its support by a **prismatic joint** + a **Kelvin-Voigt element**, with a unilateral non-penetration constraint against body "B" and an **inelastic impact law** (to allow permanent-contact phases). Acknowledged limitation: in XDE, the nonlinear law Eq. (6) is implemented by **readjusting the stiffness and the viscosity of the linear element at every time step**.

### 3.2 Hirschkorn — modified Hunt-Crossley, fitted cubic polynomial [V]

```
f_n = f_fit(x) · (1 + D·ẋ)              (Eq. 2.4)
f_fit(x) = a·x³ + b·x² + c·x            (Eq. 2.5)
```
He first tried pure Hunt-Crossley `f_n = k·x^n (1 + D·ẋ)` with n = 3/2, and rejected it: "*the power function for the spring force was not the same form as the actual felt profile*". He **explicitly rejects**:
- the **Dunlop** model: wrong form (it gives δ for F, whereas F is needed for δ), not calibratable, and Stamm's tests cast doubt on its ability to reproduce Dunlop's own data;
- the **Gonthier** model (≈ Lankarani-Nikravesh): the damping factor depends on the **initial contact velocity**, so it is unusable when the bodies **start out already in contact** and may stay so for the whole simulation. ← **decisive argument for our case** (at rest, key/wippen/jack/lever/hammer are all in contact).

Calibration of D **[V]**: `D = (f_loading(x_max)/f_fit(x_max) − 1) / ẋ_ave`, where x_max is the point of maximum deviation between the loading curve and the fitted curve, and ẋ_ave the mean normal contact velocity measured on a real forte blow.

**TABLE A.8 — the 13 contact parameter sets, measured one by one [V]** (a in N/m³, b in N/m², c in N/m):

| # | Points | a | b | c | D | A (μ_k) | B1 | B2 |
|---|---|---|---|---|---|---|---|---|
| 1 | Pk3-Pw5 (capstan) | 1.26E+10 | 3.26E+07 | 7.92E+03 | 1.877 | 0.167 | 0 | 1 |
| 2 | Pk4-Pg4 (key front) | 4.41E+10 | −1.02E+07 | 1.57E+04 | 1.219 | 0 | 0 | 0 |
| 3 | Pk5-Pg5 (key rear) | 2.42E+10 | −1.11E+07 | 6.54E+03 | 2.502 | 0 | 0 | 0 |
| 4 | Pk6-Ph3 (back check) | 5.86E+11 | −3.70E+07 | 3.11E+04 | 2.446 | **0.434** | 1.78 | 1 |
| 5 | Pw4-Pj4 (wippen-jack) | 5.28E+09 | −7.54E+05 | 4.42E+03 | 5.781 | 0 | 0 | 0 |
| 6 | Pw6-Pr3 (wippen-lever) | 2.53E+10 | −9.89E+06 | 7.07E+03 | 7.732 | 0 | 0 | 0 |
| 7 | Pj2-Ph2 (**jack-knuckle**) | 3.92E+11 | 1.10E+07 | 3.23E+04 | 4.694 | **0.177** | 1.85 | 1 |
| 8 | Pj3-Pg6 (let-off button) | 7.46E+10 | −2.29E+07 | 1.32E+04 | 1.527 | 0 | 0 | 0 |
| 9 | Pj5-Pr4 (jack-lever) | 6.02E+10 | −1.68E+07 | 6.96E+03 | 0.779 | 0 | 0 | 0 |
| 10 | Pr2-Ph2 (**lever-knuckle**) | 1.09E+12 | −7.51E+07 | 2.23E+04 | 2.935 | **0.332** | 0.73 | 1 |
| 11 | Pr5-Pg7 (lever stop) | 8.82E+11 | −7.59E+07 | 1.67E+04 | 4.985 | 0 | 0 | 0 |
| 12 | Ph4-Pg9 (**string**) | 1.81E+10 | 8.19E+07 | 1.16E+04 | 0.175 | 0 | 0 | 0 |
| 13 | Ph5-Pg8 (hammer at rest) | 8.58E+08 | −1.11E+06 | 2.39E+03 | 8.964 | 0 | 0 | 0 |

This is the **only published set calibrated contact by contact**. Consistent order of magnitude: Fig. 2.5 shows ~0 to 40 N for 0 to 1 mm of penetration on the key-front felt. The negative b values mean that the polynomial is not monotone-convex everywhere → **careful, f_fit must be clamped to ≥ 0 and monotonicity must be checked over the working range**.

### 3.3 Crucial result on the relevance of damping [V]

Hirschkorn recalibrated all the D values for a *piano* blow: the new values are **4 to 5 times larger** than those for the forte, and yet "*retuning the damping parameters has very little effect on the behaviour of the system. The results are nearly identical.*"
→ **Consequence for us: do not invest early in the exact hysteresis of the felt if the goal is kinematics and touchweight.** The elastic part f_fit(x) / k·δ^r does all the work.

### 3.4 The real behaviour of felt (for the optional extension)

The three characteristics observed by Hirschkorn **[V]**:
1. **nonlinear spring** — "*does not follow any simple power function*";
2. **hysteretic** — more force on loading than on unloading, energy loss per cycle;
3. **INDEPENDENT OF VELOCITY** — "*Whether the felt is loaded slowly or quickly seems to have little effect on the loading profile. Instead, it is dependent on the maximum and minimum loads the felt has previously undergone.*"

The two laws above (3.1 and 3.2) are therefore **knowingly wrong on point 3** — both authors state so in black and white. The physically correct model is that of **Masoudi & McPhee**, "A Novel Micromechanical Model of Nonlinear Compression Hysteresis in Compliant Interfaces of Multibody Systems", *Multibody System Dynamics*, DOI 10.1007/s11044-015-9483-6 **[V, PDF worked through]**:
- fibre-network micromechanics (Carnaby-Pan theory), hysteresis through **fibre-fibre friction** → **intrinsically velocity-independent** response;
- `P_loading = σ_el + r_sc·n·λ³·C_s` and `P_unloading = σ_eu − r_sc·n·λ³·C_s` (Eqs. 9-10), with σ_el ≠ σ_eu;
- works in **macroscopic pressure (Pa) vs volume fraction**; piano samples identified: hammer felt v_f0 = **0.389**, medium red felt (bushings) v_f0 = **0.086**, pressures up to ~3.5 × 10⁵ Pa;
- requires an **experimental loading/unloading curve per felt** + a loading/unloading transition algorithm.
→ High integration cost, limited benefit (cf. 3.3). **To be kept as a pluggable strategy, not as an initial target.**

---

## 4. FRICTION AT THE PIVOTS — and two probable typos in the published tables

### 4.1 Analytical forms

**Thorin [V]** — NON-regularised (set-valued) dry friction + linear viscous:
```
C_d(θ) = c_d · sign(θ̇)     with sign(0) = [−1, 1]   (a set!)      (Eq. 2-3)
C_v(θ) = c_v · θ̇                                                        (Eq. 4)
```
**Hirschkorn [V]** — Coulomb regularised by tanh (after Cull & Tucker):
```
μ = A·tanh(s_t/v_t) + B1·(s_t/v_t) / (1 + B2·(s_t/v_t)⁴)      f_t = μ·f_n   (Eqs. 2.10-2.11)
T = A·tanh(ω/ω_t) + B1·(ω/ω_t) / (1 + B2·(ω/ω_t)⁴)            (pivots, Eq. 2.12)
```
A → μ_kinetic as the velocity rises; B1, B2 tuned so that the **peak** of the curve reaches μ_static. For the pivots, the torque **does not depend on any normal force**: "*The friction in the joints is primarily determined by the tightness of the felt bushings (ie., large pin in a small hole), which is constant once the action is assembled.*" ← **worth remembering: a constant friction torque per pivot, not proportional to a load.**

### 4.2 Cross-check of the two measurement campaigns (dry torques, N·m)

| Body | Hirschkorn Table A.9 [V] | Thorin per body [V] | Thorin Table 2.1 [V] | Verdict |
|---|---|---|---|---|
| Key | **0.0122** | 0.006 | **0.012** | ✅ Hirschkorn ≈ Thorin T2.1 |
| Wippen | **3.97E−4** | **0.0992** ⚠️ | — | ⚠️ factor ~250 |
| Jack | 3.97E−4 | 9.9E−4 | — | ✅ same order |
| Repetition lever | 2.47E−3 | 9.9E−4 | — | ✅ same order |
| Hammer | **1.01E−3** | **9.9E−4** | — | ✅✅ almost identical |
| Damper | — | **0.0992** ⚠️ | — | ⚠️ |

**Probable typos [S]** — to be corrected before any use:
1. `c_dW = c_dD = 0.0992 N·m` is **physically impossible**: the weight torque of the wippen is C_w2 = −0.0103 N·m, so a friction 10× larger would make it completely immobile. The jack/lever/hammer values are all 9.9E−4; 0.0992 = 9.92E−2 is exactly **100×** 9.92E−4. → read **c_dW = c_dD ≈ 9.9E−4 N·m**.
2. `c_vK = 0.22 N·m·s` (per-body table) against `c_v1 = 0.022` (Table 2.1) → **factor 10**. At 1.6 rad/s (typical forte blow) 0.22 would give 0.35 N·m, i.e. 2.5× the total weight torque C_w = 0.138 N·m: absurd. → read **c_vK ≈ 0.022 N·m·s**.

**Recommended set after adjudication [D]** (N·m, dry Coulomb torque): key 0.012 · wippen 4E−4 to 1E−3 · jack 4E−4 to 1E−3 · lever 1E−3 to 2.5E−3 · hammer 1E−3 · damper ~1E−3. Viscous: key 0.022 N·m·s; all the others **4.93E−5 N·m·s**.

### 4.3 The missing friction — a quantified gap [D]

Sum of the pivot frictions referred to the key through Thorin's leverage ratios (α21 = 2.15; α31 = 10.1):
`0.012 + 2.15 × 4E−4 + 10.1 × 1E−3 = 0.0230 N·m`
Yet Thorin **measures** C_d = 0.039 N·m by static test (Eq. 2.8: `C_d = L(F_down − F_up)/2`).
→ **the pivots account for only ~59 % of the total friction felt.** The remaining ~41 % comes from the sliding contacts (capstan, knuckle, back check) and from the **key mortises (balance rail + front guide)** — precisely what Hirschkorn identifies as the improvement priority and what Thorin neutralised experimentally. **This is the main modelling gap in the literature, and it falls exactly in the register you are interested in (low force, repetition, double escapement).**

Confirmed by Hirschkorn's failure on the double blow **[V]**: "*This would indicate that the friction in the model action is too low, or that there is some friction in the experimental action that is not modelled. This low friction is less significant under the high forces that occur in a single forte blow, but are more important during the piano blow, which has lower forces, and the double blow*".

Dedicated work: **Masoudi & Birkett** on bushings and compression hysteresis (Waterloo PhD thesis 2012, "Micromechanics of Fiber Networks Including Nonlinear Hysteresis and its Application to Multibody Dynamic Modeling of Piano Mechanisms") — **I was unable to retrieve the full text or any numerical mortise-friction values** (no published value for the key bushings).

---

## 5. REPETITION SPRING (and the other springs)

**Hirschkorn [V]**:
- jack spring: **constant torque ≈ 0.0042 N·m**;
- repetition lever spring: **linear**, `T = −0.073·(θ_r − 0.832)` N·m, θ_r = lever/wippen angle in rad → κ = **0.073 N·m/rad**, θ₀ = **0.832 rad**.

**Thorin [V]** — generic form `C_s(θ) = κ·(θ − θ₀)` (Eq. 5), three springs (Table 4.12, measured by Lozada 2007):

| Coupling spring | κ (N·m/rad) | θ₀ (rad) |
|---|---|---|
| wippen — frame | 0.0355 | 0.419 |
| **jack — wippen** | 0.0299 | 0.980 |
| **lever — wippen** | **0.1171** | **0.725** |

Cross-check: lever 0.1171 (Thorin) vs 0.073 (Hirschkorn) — same order, ×1.6. θ₀: 0.725 vs 0.832 rad — consistent. ✅
Note that Thorin has an additional **wippen-frame spring** (present on his Renner, absent from Hirschkorn's Boston action) — to be made **optional** in the configuration; it is a genuine variant of make.

---

## 6. DAMPER AND PEDALS

**Damper — treated only in the Thorin thesis (ch. 4.5.7) [V]**, removed from the 2017 article "*for the sake of simplicity and clarity of the conclusions*" (the article therefore simulates the situation with the **damper pedal depressed**).

| | value |
|---|---|
| mass m_D | **50.06 g** ← the 2nd heaviest body after the key (!) |
| inertia I_D,DO | 2.04 × 10⁻⁴ kg·m² |
| CoG (DG) | (50.0 ; 0.0) mm from axis DO |
| spoon contact (DK) | (78.1 ; 0.0) mm from DO, circle ρ = 5 mm, θ = 1.57 rad |
| rest stop (DS) | (43.0 ; −22.0) mm, circle ρ = 100 mm, θ = 1.57 rad |
| angle at rest θ_D(0) | −0.23 rad |
| frictions | c_dD (read ~9.9E−4); c_vD = 4.93E−5 N·m·s |
| regulation | the spoon touches the key when **the hammer is at mid-travel (23.5 mm)** |

The damper's 50 g explain why the regulation procedure specifies the downweight **"without damper"**.

**PEDALS: NOTHING IN THE LITERATURE.**
I found **no** publication dynamically modelling the damper pedal (lift tray), the sostenuto, or the una corda on a grand piano. Thorin and Hirschkorn simulate **a single isolated note**. So does Bokiau/Fisette. This is therefore:
- **your share of original development** — but the good news is that it is geometrically simple:
  - **damper pedal**: a common lift surface (lift rail) pushing the 88 damper under-levers → **1 scalar** for the tray position, + unilateral spoon/tray contacts;
  - **sostenuto**: a rail that **captures** the tabs of the damper under-levers already raised → **1 scalar** + one state bit per note;
  - **una corda**: **rigid lateral translation of the whole action**, out of the sagittal plane → **1 scalar**, does NOT affect the sagittal dynamics, only the impact point on the strings (and the felt area being used).
- the half-depressed damper pedal (half-pedalling) and the "sympathetic" pedal are subjects in their own right, not covered.

---

## 7. THE COMPLETE PARAMETER SETS (the two gold mines)

### 7.1 Hirschkorn 2004, Appendix A — **Boston** action (Steinway's secondary brand), key no. 52, in collaboration with **Steinway & Sons New York** [V]

**Table A.1 — Inertias** (I about the CoG; CoG in the body frame):

| Body | mass (kg) | I_cm (kg·m²) | CoG x (m) | CoG y (m) |
|---|---|---|---|---|
| Key | 0.12008 | 2.75E−03 | 0.0514 | 0.0158 |
| Wippen | 0.01196 | 1.69E−05 | 0.0502 | −0.0018 |
| Jack | 0.00265 | 7.78E−07 | 0.0008 | 0.0195 |
| Repetition lever | 0.00371 | 4.10E−06 | 0.0021 | −0.0012 |
| Hammer | 0.01174 | 2.90E−05 | −0.1022 | 0.0145 |

Measurement method **[V]**: masses on a balance; CoG by suspension from a thread at two points; inertias by **pendulum oscillation**, `I_p = m·g·l·(T/2π)²` then `I_cm = I_p − m·l²` (parallel axes). Geometry: bodies **scanned flat**, images scaled, traced and measured in AutoCAD.

**Tables A.2 to A.7 — geometry**: coordinates (x, y) in m, unit normal vector, radius, arc range, for **all** the contact points of the 5 bodies + frame (Pg1…Pg9, Pk2…Pk6, Pw2…Pw6, Pj2…Pj5, Pr2…Pr5, Ph2…Ph5). Extracted in full into `h.txt`. Examples: hammer knuckle Ph2 = (−0.0170 ; −0.0091) r = 0.0044 m; hammer head Ph4 = (−0.1321 ; 0.0356) r = 0.0102 m; jack nose Pj2 = hybrid r = 0.0010 m, arc 1.571-3.142 rad; lever Pr2 = hybrid r = 0.0552 m, arc 1.300-1.571 rad.

⚠️ **Reading pitfall [S]**: the frame of each body is defined by **Figure A.1** (a figure, lost in the text extraction). I verified by cross-checking that **the origin of the key frame is the balance rail pin** (Pk4 at the front = +0.2175 m coincides with Pg4 = +0.2172 m in the frame's coordinates ✅, capstan Pk3 = −0.1176 m to the rear ✅). On the other hand `Pk2 "Pin Location" = (0.2278 ; 0.0261)` is **inconsistent** with a pivot role and probably corresponds to the **front guide pin** (of no use in 2D). **To be confirmed against the figure in the PDF before coding.**

### 7.2 Thorin 2013, ch. 4.5 — isolated **Renner** action (worn felts) [V] — THE MOST COMPLETE

Convention: lengths in **mm** in each body's frame, **origin = the body's axis of rotation** (BO), angles in rad.

**Inertias and frictions:**

| Body | mass | I about the axis | CoG (mm) | c_d (N·m) | c_v (N·m·s) |
|---|---|---|---|---|---|
| Key | 0.083 kg | 3.36E−03 kg·m² | (19.0 ; 14.0) | 0.006 *(cf. §4.2)* | 0.22 ⚠️ → 0.022 |
| Wippen | 19.8 g | 3.97E−04 | (53.0 ; 7.0) | 0.0992 ⚠️ → 9.9E−4 | 4.93E−05 |
| Jack | 5.0 g | 5.0E−06 | (0 ; 0) | 9.9E−04 | 4.93E−05 |
| Lever | 10 g | 5.0E−06 | (0 ; 0) | 9.9E−04 | 4.93E−05 |
| Hammer | 14.24 g | 1.65E−04 | (−95.7 ; 23.7) | 9.9E−04 | 4.93E−05 |
| Damper | 50.06 g | 2.04E−04 | (50.0 ; 0) | 0.0992 ⚠️ → 9.9E−4 | 4.93E−05 |

**Contact points (extract; everything is in `tt.txt`):**
- **Key**: KD (damper) −229.5/27.6 · KH (back check) −220.0/84.2 · KS− (rest) −183.0/0 · **KW (capstan) −128.7/44.6** · KO (axis) 0/0 · **KS+ (front) +241.5/0**
- **Wippen**: WO 0/0 · WL 16.1/2.0 · **LO (lever axis) 47.5/29.8** · WK (capstan) 59.0/−20.8 · WJ 75.8/20.8 · **JO (jack axis) 99.1/0**
- **Jack**: JW −13.4/24 · JH (knuckle) −8.0/48.8 · JL −3.4/43.7 · JO 0/0 · **JS (let-off button) +26.8/0**
- **Lever**: LW −41.0/−13.3 · LO 0/0 · LH 35.6/2.0 · LJ 47.3/−3.0 · **LS (drop screw) 54.5/0**
- **Hammer**: **HK (knuckle) −127.9/−21.3** · **HS (string) −126.4/54.9** · HJ −16.3/−12.0 · HO 0/0
- **Frame**: axes S_OD −308.1/39.6 · S_OW −188.5/62.06 · S_OH −96.9/125.4 · S_OK 0/0; stops S_D −265.1/7.2 · S_S (string) −260.0/178.8 · S_K− −183.3/6.16 · **S_L (drop screw) −93.5/117.1** · **S_J (button) −63.5/73.52** · S_K+ +242.0/−0.38

**Angles at rest [V]**: θ_D(0) = −0.23 · θ_W(0) = −0.03 · θ_L(0) = 0.40 · θ_H(0) = 0.40 · θ_J(0) = 0.22 · θ_K(0) = 0.03 rad.

**Published leverage ratios [V]** (Table 2.1, small angles): L = 0.245 m · l₁₂ = 0.129 · l₂₁ = 0.060 · l₂₃ = 0.080 · l₃₂ = 0.017 · **α₂₁ = 2.15** · **α₃₁ = 10.1**
Reduced inertias: J₁ = 3.36E−3 · J₂ = 3.97E−4 · J₃ = 1.65E−4 kg·m²; `J = J₁ + α₂₁²J₂ + α₃₁²J₃` = **0.0221 kg·m²** (equivalent inertia of the "simplistic key").

### 7.3 Cross-check of the two sets — remarkable convergence [D]

| Quantity | Hirschkorn (Boston) | Thorin (Renner) | Deviation |
|---|---|---|---|
| Hammer head radius from the axis | **136.8 mm** | **137.8 mm** | **0.7 %** ✅✅ |
| Knuckle radius from the hammer axis | 19.3 mm | 17.0 mm | 12 % |
| Key inertia referred to the axis | 3.10E−3 kg·m² | 3.36E−3 | **8 %** ✅ |
| Capstan arm (from the key axis) | 117.6 mm | 128.7 mm | 9 % |
| Key-front arm | 217.5 mm | 241.5 mm | 10 % |
| **Key ratio (front/capstan)** | **1.849** | **1.876** | **1.5 %** ✅✅ |
| Dry friction of the hammer pivot | 1.01E−3 N·m | 9.9E−4 N·m | **2 %** ✅✅ |

Two completely independent campaigns, two different makes, two decades apart: **convergence to ~10 % on the geometry and to better than 10 % on the inertias and the pivot frictions.** That is a solid basis of confidence for seeding the default configuration.

### 7.4 The action ratio, recomputed from the raw geometry [D]

I verified numerically that the lever chain can be reconstructed exactly from Thorin's coordinates:
```
α₂₁ = (key arm at the capstan)/(wippen arm at the capstan) = 128.7/59.0 = 2.181   [published 2.15 → 1.4 %]
α₃₂ = (wippen arm at the jack)/(knuckle radius)            = 80.0/17.0 = 4.706
α₃₁ = α₂₁ × α₃₂                                            = 10.27           [published 10.1 → 1.7 %]
R_linear(hammer head / key front) = α₃₁ × 137.8/245 = 5.68
```
**R = 5.68 falls squarely inside Stanwood's "normal zone" 5.0-7.0, observed mean 6.0.** The academic literature and PTG metrology cross-check → **the two validate each other**, and we have an explicit geometric formula for the action ratio as a function of only those parameters the technician regulates.

⚠️ **Major architectural warning [V/D]**: α₂₁ and α₃₁ are **small-angle linearisations about the rest position**. Check: with α₃₁ = 10.1 held constant, a key dip of 10 mm would give 57.6 mm of hammer travel, whereas the regulated blow distance is 45-47 mm. **The instantaneous ratio therefore varies strongly over the travel** (and that is precisely what makes the "touch"). **The engine must NEVER use a constant α**: it must recompute the instantaneous ratio from the contact geometry at every step. The published α values are only good for non-regression tests at rest.

---

## 8. SOLVERS, TIME STEPS, COMPUTATIONAL COST — the real-time budget

### 8.1 Costs reported in the literature [V] (Thorin 2017 Table 1 + sources)

| Work | Cost | Hardware | Ratio / real time |
|---|---|---|---|
| Van den Berghe 1995 (full) | 4 h for 1 s | 486DX33 | **14,400×** |
| Van den Berghe 1995 (reduced) | 15 s for 1 s | DEC 5000/33 | 15× |
| **Hirschkorn 2004** | **~6 min / blow** | 2.4 GHz, MATLAB ODE15s | ~**1000×** |
| Izadbakhsh 2006 (flexible shank) | 75 min / blow | 2.4 GHz, Maple | ~10,000× |
| Masoudi & Birkett 2015 (rigid) | 910 s for ~200 ms | — | **~4550×** |
| Masoudi & Birkett 2015 (flexible shank) | 1720 s | — | ~8600× (×1.9) |
| **Thorin 2017 (non-smooth, XDE 3D)** | **20 s for 1 s** | Core i5 2.1 GHz (2010) | **20×** |
| Thorin, **before** adding the felt laws | — | same | **1.2×** |
| UCLouvain 2018 (symbolic ROBOTRAN, compiled C, adaptive stiff solver) | **1.15 s for 1 s** | — | **1.15×** |
| Timmermans 2020 (haptic keyboard) | **real time** | — | **1×** |

**Reading: the move to non-smooth dynamics gains ~2 orders of magnitude** (Thorin: "*about two orders of magnitude faster than the regularizing approach, as reported in the literature*"), and the residual ×20 gap is attributable to the general-purpose engine, not to the method.

### 8.2 Thorin's numerical scheme [V]

- Formalism: **measure differential inclusion (MDI)** — `M(x)dv = F_H(x, ẋ, t)dt + H(x)di`, `v⁺ = (ẋ)⁺`, `(g(x), Hᵀ(x)v⁺, di) ∈ K`.
- The 4 inclusions: pivot dry friction `r_P ∈ c_d·sign(ẋ)` (∈ ℝ¹²); non-penetration `g(x) ∈ ℝ₊¹²`; normal reactions `r_N ∈ N_{ℝ₊¹²}(g(x))` (normal cone); tangential Coulomb `[Hᵀ(x)v]_{3i−2,3i−1} ∈ N_{B(μ(r_N)_i)}((r_T)_{2i−1,2i})`.
- Discretisation: **Jean's (1999) time-stepping** / Moreau-Jean. Linear interpolations `x_{n+α} = x_n + α·Δt·λ_x`, `v_{n+α} = v_n + α·Δt·λ_v`.
- Scheme parameters: **θ = 1** (stability of the smooth dynamics), **γ = 1** → "*constraints are satisfied at the end of every time step*".
- Solution: linearisation of H and of the smooth dynamics about the previous iterate, **Newton loop** → **One-Step Non-Smooth Problem**, reformulated as an **augmented Lagrangian**, solved by a **Gauss-Seidel type algorithm** (ref. Studer 2009, ch. 4).
- **Time step: Δt = 0.5 ms (2 kHz)** [V]. Explicit convergence study (Fig. 12): 2 kHz and 10 kHz **indistinguishable**; **0.4 kHz insufficient** (significant deviations). Characteristic time of the action: "*a few milliseconds*".
- Key advantage: "*there is no numerical parameter for impact or dry friction calling for a very small time-step*".

Hirschkorn, by contrast **[V]**: MATLAB **ODE15s** (stiff), absolute tolerance **1 × 10⁻⁵ rad**; "*the solvers dedicated to stiff systems were much more efficient*"; stiffness dominated not by the contacts but by **the regularised friction model** — "*Even more significant in contributing stiffness is the friction model.*" ← **a further argument for the non-smooth approach.**

### 8.3 Budget for 88 notes in the browser [D]

- Target: **2 kHz** per note (converged according to Thorin), i.e. 88 × 2000 = **176,000 note-steps/s** in the worst case.
- **Decisive lever: putting notes to sleep.** The 88 notes are almost independent (coupling only through the damper-pedal lift tray / sostenuto / una corda shift = **3 scalars**). In a real piece, **~10-15 notes at most are moving simultaneously**; the others are at rest in permanent contact and can be frozen (a steady state verifiable by a residual criterion). Realistic active budget: **~10-15 notes × 2 kHz = 20-30 k note-steps/s**, i.e. a factor of 6-9 in margin.
- Per note-step: 5-6 DOF, 12-14 unilateral contacts + 3 frictional ones. The cost is dominated by the projected Gauss-Seidel sweeps over ~15-17 contacts.
- Conclusion **[S, to be confirmed by prototype]**: a dedicated 2D solver (not a general-purpose 3D engine) should hold the 88 notes in real time in WASM, especially with SIMD and a few workers. The two benchmarks from the literature — Thorin's 1.2× estimated for dedicated 2D on 2010 hardware, and UCLouvain's 1.15× measured in compiled C — make this target credible with a wide margin on 2026 hardware.
- The two acceleration routes that Thorin identifies himself are exactly ours **[V]**: "*include a more efficient implementation of the nonlinear springs laws and perform direct 2D dynamics rather than 3D*" (expected gains > 15× and > 3× respectively).

---

## 9. EXPERIMENTAL DATA AVAILABLE TO VALIDATE OUR ENGINE

### 9.1 Hirschkorn 2004 — complete test rig [V]

Equipment: **Boston grand piano** keyboard, key no. 52 · **Kollmorgen** electric motor (actuation) · **load cell** on the key · **MicroE optical encoders** on key, wippen, hammer · **Photron high-speed camera** for jack and lever · LabVIEW 7.0. Measurements judged "*highly repeatable, reliable*" (5 superimposed trials, Fig. 3.9).

Usable data (figures from the thesis, to be re-digitised):
- **Fig. 3.6**: force profiles measured on an amateur pianist;
- **Fig. 3.7 / 3.8**: force profiles for pianist **and** motor, **forte** and **piano** blows;
- **Fig. 4.1-4.4**: key / wippen / hammer rotations, **forte** blow, experiment vs model;
- **Fig. 4.6-4.8, 4.9-4.11**: same for the **piano** blow (+ recalibrated damping);
- **Fig. 4.13-4.15**: same for the **double blow** (the difficult case);
- **Fig. 2.5 / 2.6**: **loading / unloading / fit** curves of the key-front felt, 0-40 N over 0-1 mm — directly usable to validate a felt law.

Characteristics of the inputs **[V]**: the *piano* blow rises progressively to ~1.5 N before the key-bottom peak; amplitude ~**7× smaller** than the forte, and ~**3× longer** to reach the bottom. 60 Hz interference visible in the piano signal.
Output scales: key rotation ~0.05 rad, wippen ~0.10 rad, **hammer ~0.40 rad**, over 0-0.7 s.

### 9.2 Thorin 2013/2017 — the set richest in FORCES [V]

Equipment: isolated **Renner** action · **Keyence LB12 + LB72** laser sensors (resolution 50 μm, integration time 0.15 ms, sensitivities 0.398 and 0.449 V/mm) · **Endevco 2250A** accelerometer (0.4 g, 0.316 V/m·s⁻², Brüel & Kjær Nexus conditioner 0.1 Hz-3 kHz) · **Kistler 9211 piezoelectric force sensor** (1.2 g, Kistler 5015 charge amplifier, 1 Hz-3 kHz, sensitivity 10 mN, nonlinearity + hysteresis ±1 % of full scale, scale set to 10 N) · **Simi HCC-1000 high-speed camera** (923 fps, 1024×512, 4 CMOS sensors) + **KLT** tracking · **NI USB-6211** acquisition at 50 kHz.
Post-processing **[V]**: identification and subtraction of the **50 Hz** and **0 Hz** components on the basis of the signal preceding the blow; velocity obtained by **total variation regularisation** (Chartrand 2011: 30 iterations, 200 sub-iterations, regularisation parameter 5 × 10⁵, ε = 10⁻⁹).

Usable data:
- **Fig. 7**: key-end position, *piano* blow, **force**-driven (model relative error **3 %**; the rigid-felt "Bad Model": **9 %**);
- **Fig. 8 / 9**: **reaction forces** at the key end, *piano* and *forte* blows, **position**-driven ← **the reference curves to aim at**;
- **Fig. 10**: simulated angles vs KLT tracking for **key, hammer, wippen, jack, lever** over 0-0.45 s;
- **Fig. 11**: film / simulation comparison frame by frame at the 5 notable events;
- **Fig. 12**: time-step convergence (0.4 / 2 / 10 kHz).

### 9.3 Askenfelt & Jansson, Goebl — reference data on touch [V]

- Askenfelt & Jansson, "From touch to string vibrations. II: The motion of the key and hammer", **JASA 90(5), 2383-2393 (1991)**, DOI 10.1121/1.402043 — two accelerometers per note (key + hammer).
- Goebl, Bresin, Galembo, "Touch and temporal behavior of grand piano actions", **JASA 118(2), 1154-1165 (2005)** — free PDF retrieved. Pianos: **Bösendorfer** (SE) and **Yamaha** (Disklavier), + Steinway. *Pressed* vs *struck* touches.

Usable numerical values **[V]**:
| Quantity | Value |
|---|---|
| Duration rest → key bottom, **forte** (~5 m/s FHV) | **~25 ms** |
| Duration rest → key bottom, **piano** (~1 m/s FHV) | **~160 ms** |
| Hammer-string contact vs key bottom, **piano** (1 m/s) | **12 ms BEFORE** |
| Hammer-string contact vs key bottom, **forte** (5 m/s) | **3 ms AFTER** |
| Travel time finger-key → hammer-string | **20 ms to ~200 ms** (up to 230 ms on the Steinway) |
| Maximum measured hammer velocity | up to **6.4-7.8 m/s** (C1 / G6, Yamaha and Steinway) |
| Finger-key force in *legato* playing | ~1 N |
| Correlation key velocity → hammer velocity | *pressed* > 0.6; *struck* < 0.6 |

### 9.4 Let-off velocities — the cleanest validation target [V]

Thorin Table 6.1 / Table 5:

| Dynamic level | Measured | Simulated |
|---|---|---|
| *piano* | **1.01 m/s** | **1.00 m/s** |
| *forte* | **1.56 m/s** | **1.56 m/s** |

"*The prediction is very good*" / "*perfectly estimated*". **This is THE quantified test to reproduce first.**

---

## 10. TOUCHWEIGHT — Stanwood's exact equations [V]

Extracted from **Appendix A** of D. Stanwood, "Down Weight versus Equation Piano Key Balancing: Shifting the Paradigm" (June 2014), with the protocols published in the *Piano Technicians Journal*, February 2000 ("Standard Protocols of the New Touchweight Metrology").

**Terms**: SW = Strike Weight · R = Strike Weight Ratio · FW = Front Weight · KR = Key Weight Ratio · WW = Wippen Radius Weight · WBW = Wippen Balance Weight · D = Down Weight · U = Up Weight · BW = Balance Weight · F = Friction Weight.

```
BW  = (D + U) / 2
F   = (D − U) / 2
WBW = KR × WW
R   = (BW + FW − WBW) / SW
```
→ **Balance equation**: `BW + FW = KR × WW + SW × R`
→ Patent **US 5,585,582** ("Methods for inertial balancing of piano key mechanisms"): `FW = ((SW × R) + (WW × KR)) − BW`

**Measurement protocols [V]**:
- **SW**: the **hammer mounted on its shank is tipped** onto a digital scale;
- **FW**: the **keystick is tipped onto its balance point** and the key front is rested on the scale.

**Reference values and tolerances [V]**:
| Quantity | Value |
|---|---|
| Normal zone for R | **5.0 to 7.0** (above 7.0: action too heavy even with very light hammers) |
| Observed mean R (high-end concert grand) | **6.0** (middle of the "medium" zone) |
| Perceptible change in R | **0.3** → "*a significant change in the feel of the action*" |
| Perceptible change in SW | **0.5 g** → "*Pianists can easily feel a 0.5 hammer weight change*" |
| SW variation between neighbouring notes (common) | **1.0 g** |
| SW spread between identical sets (same make/model/period) | up to **2.0 g** |

**Weight regulation target (Thorin, Appendix A, transcript of 2 professional technicians) [V]**: "*the minimum downward weight of the action without damper is close to 50 g and the maximum upward weight is 25 g*" → BW = 37.5 g, F = 12.5 g.

**Actual weights of Thorin's experimental action [D]**, derived from C_w = 0.132 N·m (static test Eq. 2.7), C_d = 0.039 N·m (Eq. 2.8), L = 0.245 m:
```
BW = C_w/(L·g) = 54.9 g      F = C_d/(L·g) = 16.2 g
→ D ≈ 71 g      U ≈ 39 g
```
Markedly heavier and more frictional than the 50/25 g target: consistent with the author's admission about the "*time-worn felts of the used action*". **Worth remembering: Thorin's parameter set describes a real, worn Renner action, not an ideally regulated one.** That is excellent news for a regulation-training tool (one starts from a case to be corrected), but it must be documented explicitly.

---

## 11. THE METHODOLOGICAL TRAP NOT TO BE REPEATED (Thorin, Boutillon, Lozada 2014)

Ref.: "Modelling the dynamics of the piano action: is apparent success real?", **Acta Acustica united with Acustica 100(6), 1162-1171 (2014)** — reproduced in full as **chapter 2 of the Thorin thesis**, which I worked through.

**Table 2.3 — sensitivity of the results depending on the driving mode [V]**:

| Input → Output | F → y | ÿ and y → F | y → F | ÿ → F |
|---|---|---|---|---|
| sensitivity to the **MODEL** | **0** | **+** | **+** | **+** |
| sensitivity to the **initial conditions** | + | 0 | 0 | + |
| sensitivity to **NOISE** | 0 | 0 | + | 0 |

("+" = sensitive, "0" = little or not sensitive.)

**Demonstration [V]**: the dynamics are dominated by inertia up to let-off, so the acceleration is smooth; the **double time integration filters** everything out. "*any model, provided that it is inertia-dominated, is likely to appear as very good when checking its validity by means of comparisons of motion-results obtained in force-driven simulations*". Proof by absurdity with the **"Bad Model"** (felts made rigid — an action that, in reality, would not even reach the string): **9 % position error against 3 % for the good model.** Under position-driven conditions, conversely, the Bad Model is "*off the measured force*".

**Imperative consequences for our validation protocol:**
1. **Drive in POSITION, compare the FORCES.** Never the other way round as the primary validation.
2. Comparing the accelerations is acceptable **provided** one is at a level ≥ mezzoforte.
3. The hammer-displacement comparisons published by Oboe, Van den Berghe, Gillespie and Hirschkorn **do not constitute validations** in the strong sense — including Hirschkorn's, whose parameter set nevertheless remains excellent.
4. Corollary for our UX: **the curve to display to the technician is the reaction force on the finger**, not the trajectory. That is where let-off, key bottom and checking are visible.

---

## 12. SENSITIVITY ANALYSIS WITH RESPECT TO THE REGULATION — the pedagogical core, quantified in the literature [V]

Thorin, thesis ch. 6.5. Cost function: `J = Σ|F_mes(t_i) − F_sim(t_i)| / Σ|F_mes(t_i)|` (1-norm, "*because we did not want to penalise large discrepancies too much*"), t_b = 0 s, t_e = 0.22 s (*piano*) / 0.13 s (*forte*), measurements at 50 kHz down-sampled to 2 kHz. Each parameter varied **in steps of 0.1 mm over ±2 mm** → 82 simulations per parameter.

**The 8 "category I" parameters = those a technician actually regulates [V]**:
| Symbol | Regulation |
|---|---|
| S_K+y | height of the key-front felt (**key dip**) |
| K_Wx, K_Wy | coordinates of the top of the **capstan** |
| S_Jy | vertical position of the **let-off button** |
| J_Wx | horizontal position of the **jack regulating screw** |
| S_Ly | vertical position of the **drop screw** |
| K_Hx | horizontal position of the **back check** |
| L_Wy | **repetition lever regulating button** |

**Quantified results (Tables 6.2 / 6.3) — "blocking length" = the variation that JAMS the action [V]**:

| Parameter | Range tested | Shape of J | Blocking length | ΔJ/J |
|---|---|---|---|---|
| J_Oy (jack axis, vertical) | ±1 mm | irregular/flat | **0.8 mm** | 24 % |
| J_Ox (jack axis, horizontal) | ±1 mm | irregular/flat | **0.2 mm** | **37 %** |
| J_Hy (jack-knuckle contact, vertical) | ±1 mm | irreg./linear/irreg. | **0.4 mm** | **42 %** |
| J_Hx (jack-knuckle contact, horizontal) | ±1 mm | irregular | **0.2 mm** | 18 % |
| L_Hy (lever under the knuckle) | ±1 mm | **convex** | — | 26 % |
| J_Sx / J_Sy (let-off button) | ±1 mm | irregular/flat | **0.2 mm** | 2.5 / 6.7 % |
| H_Jx / H_Jy (knuckle on the hammer) | ±1 mm | flat/irreg.; irreg. | **0.4 mm** | 10 / 3.0 % |
| W_Ox (wippen axis) | ±1 mm | irregular/flat | 0.4 mm | 3.9 % |
| K_Ox (key axis) | ±1 mm | flat/irregular | 0.4 mm | 2.0 % |
| K_Wθ (angle of the capstan contact) | ±0.5 rad | irregular/convex | 0.3 rad | **17 %** |
| L_Hθ (lever angle) | ±0.5 rad | convex | 0.0 mm | 18 % |
| I_H,HO (hammer inertia) | ±10 % | linear | — | 8.9 % |
| m_H (hammer mass) | ±10 % | flat | — | 4.7 % |
| ν_JH (jack-knuckle friction) | ±20 % | slope | — | 4.9 % |
| masses of the auxiliary felts | −50 % to +1000 % | flat | — | **< 1.6 %** |

**Quotation to frame [V]**: "*Table 6.2 also highlights the high accuracy required for the regulation of the piano action: for many parameters, a variation of 0.2 mm leads to a completely different dynamical behaviour.*"

**Directly actionable lessons:**
- **All the most sensitive parameters concern let-off** (jack axis, jack-knuckle contact, lever under the knuckle): "*these parameters are interestingly all related to the escapement, which is known to be very delicate*". That is therefore where the best numerical accuracy **and** the best visualisation are needed.
- J is **convex** only along: S_K+y (key dip), L_Wy (lever button), S_K− y (back rest felt), K_Oy (height of the key axis), K_Wθ (capstan angle). Everywhere else it is **flat or irregular** → "*common methods of convex optimisation should probably not be applied*". **Consequence: no auto-regulation by gradient descent.** A grid search / scripted expert procedure will be needed.
- The mass of the auxiliary felt bodies has **no influence** (< 1.6 % over +1000 %) → a robust choice, 1 mg will do.
- Let-off (S_Jy) and the jack screw (J_Wx) are **critical for the escapement but almost invisible in the reaction force** (2.5 % / 6.7 %): the UI therefore needs **two families of indicators**, force **and** kinematics/events. Thorin says so: "*Both S_Jy and J_Wx are critical for the escapement of the hammer, and therefore for the control offered by the piano action, but do not change significantly the reaction force of the key.*"

---

## 13. REFERENCE REGULATION PROCEDURE (values to put in the default configuration)

### Thorin, Appendix A [V] — transcript of exchanges with two professional technicians

| Step | Specification |
|---|---|
| Key balance rail | when lifted, the key must come back down **very slowly** |
| Hammer pivot | unscrewed and held vertically downwards, the hammer must come to a stop after **3 pseudo-periods** |
| General alignment | all bodies parallel and aligned |
| **Jack-hammer** | at rest, the jack aligned with the wooden rectangle of the knuckle |
| **Repetition lever** | in contact with the knuckle; adjust the screw so that the jack is **0.1-0.3 mm** from the knuckle |
| **Capstan** | for a key dip of **10 mm** (tolerance **9.5-10.5 mm**), hammer-string distance = **45-47 mm** |
| **Let-off button** | let-off **as close to the string as possible, typically ≤ 1.5 mm** |
| **Drop screw** | with the key depressed, the hammer drops "definitely but not too much" |
| **Back check** | wire set so that, in *forte*, the hammer is checked **15 mm** from the string, with contact on the **upper third** of the back check |
| **Damper (spoon)** | contact with the key when the hammer is at **mid-travel (23.5 mm)** |
| **Key leads** | minimum downweight **≈ 50 g** without damper, maximum upweight **25 g** (destructive operation, as a last resort) |

### Hirschkorn, §2.3 [V] — Steinway & Sons procedure, 7 points

| # | Specification |
|---|---|
| 1 | **Key dip**: paper punchings under the front felt; travel = **10.0 mm** |
| 2 | **Capstan** screwed up until **hammer height at rest = 48.0 mm** |
| 3 | **Jack at rest** (cushion screw): right-hand side of the jack aligned with the right-hand edge of the knuckle |
| 4 | **Lever at rest** (felted cushion screw): lever at the same height as the jack under the knuckle |
| 5 | **Let-off button**: the jack lets off when the hammer height is **1.0 mm** |
| 6 | **Drop screw**: after the jack is released, hammer height = **3.0 mm** on the lever (→ drop of **2.0 mm**) |
| 7 | **Back check** (bent wire): under a medium blow, hammer stopped at **15.0 mm** |

An important remark **[V]**: Hirschkorn **re-regulates the model** after measuring the geometry, because "*regulation adjustments are extremely fine, and are beyond the accuracy of any measurements. This is especially true for the location of the felt parts, since the "surface" of the felt is not clearly defined.*"
→ **Architectural consequence: a "regulation mode" is needed that adjusts the geometric parameters to reach TARGETS (key dip, blow distance, let-off, drop, checking), rather than taking raw coordinate entry.** That is exactly the ergonomics a technician expects, and it is technically mandatory.
Thorin does the same **[V]**: his virtual regulation led him to lower the let-off button by **1 mm** and the drop screw by **2.5 mm** relative to the measured values — "*Except for the let-off button, these modifications lie within the margin of uncertainties of the measurements.*"

---

## 14. HAMMER-STRING CONTACT (optional sound extension)

Both reference models **avoid** the string:
- Hirschkorn **[V]**: the string is replaced by a fixed rigid body; the damping factor of contact 12 was **tuned iteratively** to obtain the correct rebound velocity; "*the results from the simulation can only be compared to experimental results with the hammer striking a solid body, not a string*".
- Thorin **[V]**: "*a simple ad hoc Newton elastic law suffices*"; justification: the hammer has **already let off** when it touches the string, so "*fine modelling of the hammer-string interaction is not relevant in this work*" and the flexibility of the shank "*is assumed to have no effect on the touch*".

If you want the sound, the reference is **A. Stulov**, "Experimental and computational studies of piano hammers", *Acta Acustica united with Acustica* **91(6), 1086-1097 (2005)** [V, PDF worked through] — a **hereditary (material with memory)** model:
```
F(u(t)) = F₀ · [ u^p(t) − (ε/τ₀) ∫₀ᵗ u^p(ξ) · exp((ξ−t)/τ₀) dξ ]
```
F₀ = stiffness (N/mm^p) · p = exponent of compliance nonlinearity · ε = hereditary amplitude · τ₀ = relaxation time.
Limiting cases: instantaneous `F = F₀u^p`; static `F = F₀(1−ε)u^p`.

**Table 1 — 4 manufacturers, new unvoiced hammers, m₀ = 9 g ± 0.2 g, V_h ≈ 1 m/s [V]**:

| | Abel | Renner old type | Renner new type | Imadegawa |
|---|---|---|---|---|
| F₀ (kN/mm^p) | 15.0 | 17.0 | 14.0 | 11.3 |
| p | 4.25 | 4.00 | 3.85 | 4.30 |
| ε | 0.994 | 0.995 | 0.995 | 0.992 |
| τ₀ (μs) | 2.00 | 2.30 | 2.25 | 2.40 |

**Scaling laws over the 88 notes (unvoiced Abel set) [V]** — valuable for seeding a whole keyboard:
```
m₀ = 11.074 − 0.074N + 0.0001N²  [g]        (11.0 g at A0 → 5.3 g at C8)
p  = 3.7 + 0.015N                           (3.72 → 4.98)
ε  = 0.9894 + 0.000088N                     (0.9895 → 0.9972)
τ₀ = 2.72 − 0.02N + 0.00009N²  [μs]         (2.70 → 1.65)
F₀ = 15500 · exp(0.059N)  [N/mm^p]          (16,440 → 2,787,300)
```
→ The **stiffness F₀ grows by a factor of 170 from the bass to the treble**: by far the dominant parameter. Identification uncertainties: ±1 % on F₀, ±0.02 on p, ±0.002 on ε, ±0.1 μs on τ₀.
Other benchmarks **[V]**: new hammers from different manufacturers are **almost indistinguishable**; Stulov's test rig covers 0.3-1.5 m/s whereas real playing goes **up to 5 m/s**; p = 1.5-2.8 for new hammers and 2.5-3.9 for worn hammers (ISMA 2014); the influence of the dummy string diameter is small, "*testing of hammers against a flat surface would give a fair approximation*".

---

## 15. QUANTIFIED VALIDATION PROTOCOL

To be implemented as an automated domain test suite, in order of increasing cost:

**Level 0 — static, no integration (instantaneous, to be done first)**
| Test | Target | Source |
|---|---|---|
| V0.1 Geometric action ratio at rest | α₂₁ = 2.15 ±2 % · α₃₁ = 10.1 ±2 % | Thorin T2.1 [V] |
| V0.2 Linear hammer head / key front ratio | R = 5.68; must fall within **5.0-7.0** | [D] + Stanwood [V] |
| V0.3 Equivalent inertia reduced to the key | J = 0.0221 kg·m² | Thorin T2.2 [V] |
| V0.4 Downweight / upweight (quasi-static ramp) | **D ≈ 71 g, U ≈ 39 g** with Thorin's parameters | [D] from C_w, C_d, L [V] |
| V0.5 Stanwood consistency | BW = (D+U)/2, F = (D−U)/2, R = (BW+FW−WBW)/SW must **all** come out of the same model | Stanwood [V] |
| V0.6 Set regulated to the targets | D ≈ 50 g, U ≈ 25 g (without damper) | Thorin App. A [V] |

**Level 1 — dynamic, single isolated note**
| Test | Target | Source |
|---|---|---|
| V1.1 **Let-off velocity** | *piano* **1.01 m/s** · *forte* **1.56 m/s** (±3 %) | Thorin T6.1 [V] |
| V1.2 Order and timing of the 5 non-smooth events | (a) jack/button → (b) key/front felt → (c) hammer/string → (d) checking → (e) jack repositioning | Thorin Fig. 10-11 [V] |
| V1.3 Touch timing | key bottom at **25 ms** (forte, 5 m/s) and **160 ms** (piano, 1 m/s); string contact **12 ms before** the bottom at 1 m/s, **3 ms after** at 5 m/s | Askenfelt & Jansson / Goebl [V] |
| V1.4 Kinematic amplitudes | hammer rotation ~0.40 rad · key ~0.05 rad · wippen ~0.10 rad | Hirschkorn Fig. 4.1-4.4 [V] |
| V1.5 **Reaction force, POSITION-driven** | `J = Σ|F_mes−F_sim|/Σ|F_mes| ` (1-norm, at 2 kHz); aim at Thorin's order of magnitude, and above all **reproduce the irregularities** (let-off, key bottom, checking) | Thorin Fig. 8-9, Eq. 6.4 [V] |
| V1.6 Anti-trap "Bad Model" test | with the felts made rigid, the position under force-driving must stay good (~9 %) **but** the force under position-driving must degrade markedly — if it does not, the test harness is wrong | Thorin 2014 [V] |

**Level 2 — convergence and numerical robustness**
| Test | Target | Source |
|---|---|---|
| V2.1 Time-step convergence | results at **2 kHz ≡ 10 kHz**; **0.4 kHz must visibly diverge** | Thorin Fig. 12 [V] |
| V2.2 Insensitivity to the auxiliary felt masses | **< 1.6 %** of ΔJ for −50 % to +1000 % | Thorin T6.3 [V] |
| V2.3 Insensitivity to contact viscous damping | recalibrating D by ×4-5 must change the kinematics negligibly | Hirschkorn §4.2.2 [V] |

**Level 3 — sensitivity to the regulation (the heart of the product)**
| Test | Target | Source |
|---|---|---|
| V3.1 **Jamming of the action** | must jam at: J_Ox 0.2 mm · J_Oy 0.8 mm · J_Hx 0.2 mm · J_Hy 0.4 mm · J_Sx/J_Sy 0.2 mm · H_Jx/H_Jy 0.4 mm · W_Ox 0.4 mm · K_Ox 0.4 mm | Thorin T6.2 [V] |
| V3.2 Shape of the cost function | convex along S_K+y, L_Wy, S_K−y, K_Oy, K_Wθ; flat elsewhere | Thorin §6.5.6 [V] |
| V3.3 Hierarchy of the sensitivities | ΔJ/J: J_Hy 42 % > J_Ox 37 % > L_Hy 26 % > J_Oy 24 % > J_Hx 18 % | Thorin T6.2 [V] |
| V3.4 "Trade" perceptibility | ΔR = 0.3 and ΔSW = 0.5 g must produce a **measurable** effect on the outputs | Stanwood [V] |
| V3.5 Convergence of the regulation procedure | the scripted procedure (Thorin App. A / Hirschkorn §2.3) must reach all the targets starting from an out-of-regulation state | [V] |

**Level 4 — repetition and low force (the area where the literature fails)**
| Test | Target | Source |
|---|---|---|
| V4.1 Double blow | reproduce Hirschkorn Fig. 4.13-4.15 — **knowing that his model fails there**, through underestimated friction. If our model does better, that is a genuine contribution | Hirschkorn §4.3 [V] |
| V4.2 Friction balance | the sum of the pivot frictions must account for only **~60 %** of the total measured friction; the rest must come from the sliding contacts and the mortises | [D] |
| V4.3 Static threshold test | the maximum mass that can be placed without initiating the descent, and the minimum mass preventing the return. **Impossible to simulate with regularised friction** — this is the discriminating test that justifies the non-smooth approach | Thorin §4 [V] |

---

## 16. CONCLUSIONS

### Reference model to adopt
**Thorin et al. 2017 / Thorin thesis 2013**, parameterised with the values from his chapter 4.5 (Renner, 6 bodies including the damper), **supplemented by Table A.8 of Hirschkorn 2004** for the contact-by-contact force laws.

Six reasons:
1. **It is the only model validated on the FORCES**, and the only group to have demonstrated (2014) that validation on displacement proves nothing.
2. It **retains genuine dry friction** (set-valued, stick-slip), which is the condition for the sensitivity to the regulation to be real and for the static threshold tests to be simulable — precisely the core of regulation training.
3. It is **converged at 2 kHz** with a demonstrated route to real time (1.2× estimated for dedicated 2D; 1.15× measured independently by UCLouvain in compiled C; real time achieved by Timmermans 2020 on a haptic key).
4. Its parameter set is **complete, public and internally consistent** (I checked: the geometry reproduces his own α values to 2 %), and it **includes the damper**.
5. It comes with a **quantified sensitivity analysis** of every regulation adjustment, including the blocking lengths — a ready-made functional specification.
6. It **cross-checks to ~10 %** against Hirschkorn's completely independent set, and its recomputed action ratio (5.68) falls within Stanwood's normal zone (5.0-7.0, mean 6.0) — the academic × PTG-metrology triangulation holds.

### Extensions, ranked by benefit/cost ratio
1. **Key mortises + 2nd DOF of the key (vertical translation on the balance rail)** — *the highest priority.* This is the only gap that **both** groups report, it carries ~40 % of the total friction [D], and it falls exactly in the low-force / repetition register where Hirschkorn fails. Ref.: Masoudi & Birkett.
2. **Pedals + coupling of the 88 notes** — not an extension but **mandatory original development**: no literature at all. Geometrically simple (3 scalars), but to be specified from scratch with the technician.
3. **Velocity-independent felt hysteresis** (Masoudi micromechanics, or Stulov hereditary) — physically correct, but Hirschkorn shows that recalibrating the damping by ×4-5 changes the kinematics almost not at all. Useful for the **force feel** and for packed/worn felt, not for the geometry. **Pluggable strategy, phase 2.**
4. **Flexible hammer shank** (Izadbakhsh 2008, Rayleigh beam; Chabassier & Duruflé 2014, Timoshenko) — Thorin computes f₁ ≈ **39 Hz** and (f₁τ)⁻¹ = **5.1**, so the rigid assumption is **marginal** for the hammer; but the flexibility acts **only after let-off**, hence **no effect on the touch**. Measured cost: **×1.9 CPU time** (Masoudi & Birkett). Interest: the visual wobble of the hammer in the 3D rendering, and the key oscillation after checking that Hirschkorn attributes to the shank. **Low priority, and more a rendering effect than a physics effect.**
5. **String** (Vyasarayani 2009) — out of scope for the action. Replace it with a restitution coefficient. Add Stulov **only** if you want to synthesise the sound.
6. **Flexible key** — **to be set aside**: f₁ ≈ 355 Hz, (f₁τ)⁻¹ = 0.56, and Thorin shows that the felts contribute **6× more** to the compliance than the wood (w_f = 9.5E−5 vs w_e = 1.5E−5 m/N). For the hammer, the ratio is **4×** (4.8E−3 vs 1.2E−3 m/N).

### Three architectural decisions the literature imposes
1. **The felt law must be a pluggable strategy.** Hirschkorn (cubic polynomial × (1+Dẋ)) and Thorin/Brenon (kδ^r + bδ²δ̇) are **not interchangeable** without re-identification, and the physically correct model (velocity-independent) is a third one. All three must coexist behind the same interface.
2. **The regulation adjustments are expressed as TARGETS, not as coordinates.** Both authors re-regulate their model after measurement because the surface of the felt cannot be defined to better than the measurement accuracy. The domain API must expose "key dip = 10 mm", "let-off = 1.5 mm", "drop = 2 mm", "checking = 15 mm" and solve for the coordinates — not the other way round.
3. **Never a constant leverage ratio.** The published α values are small-angle linearisations; the discrepancy is blatant (57.6 mm predicted against 45-47 mm as regulated). The instantaneous ratio must be recomputed from the contact geometry at every step. That is, moreover, the heart of what the tool must **show**.
