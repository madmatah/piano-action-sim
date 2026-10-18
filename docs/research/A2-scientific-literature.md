# A2. Scientific literature on grand piano action dynamics

Research annex to `docs/FOUNDATIONS.md`, which holds wherever the two disagree.
It reads the published models of the grand action, their parameter sets, their
experiments and their numerical methods, as primary sources, for what the
simulator can take from them and what it must not.

Markers, as in FOUNDATIONS: **[V]** verified against a primary source in the
corpus, named with its locator; **[E, n]** computed by script `n` in
`tools/derivations/`, which runs with `python3 -I`; **[S]** assumed, pending a
measurement or a source; **[?]** given by a source that is not in the corpus, or
not found in it, with the reason. A number whose arithmetic is written beside it
carries no marker.

Sources, in `data/references/`, outside version control (third-party copyright;
FOUNDATIONS §10):
```
data/references/papers/
  hirschkorn_thesis.pdf    Waterloo MASc thesis (2004): one action, its parameters measured
  thorin_thesis_2013.pdf   Polytechnique PhD thesis: the parameter values, the sensitivity analysis
  thorin2017.pdf           Meccanica article: the non-smooth model and its validation on forces
  lozada_thesis.pdf        Polytechnique PhD thesis (2007): the measurements behind Thorin's parameters
  masoudi_preprint.pdf     Masoudi & McPhee: a micromechanical model of hysteretic felt
  stulov_actaa2005.pdf     hammer felt, hereditary law
  goebl2005.pdf            touch and timing measurements on three grand pianos
  goebl2003.pdf            computer-controlled grand pianos: hammer velocity measurement
  stanwood2014.pdf         down weight against equation balancing
  stanwood_blueprint.pdf   component touchweight balancing
  timmermans2021.pdf       Timmermans, Ceulemans & Fisette: an upright and a grand action, one multibody method
  fisette2019.pdf          Fisette, Bokiau & Timmermans: the grand action explained by removing parts (author's manuscript)
data/references/stanwood/  Stanwood's touchweight metrology series, 1996–2014
data/references/manuels/   regulation manuals and specification sheets
data/references/extracts/
  h.txt · m.txt · s.txt    text extractions (Hirschkorn, Masoudi & McPhee, Stulov)
```

---

## 1. Overview of the published models (after Thorin 2017, Table 2)

| Work | Input | Output | DOF | Numerical method | Role here |
|---|---|---|---|---|---|
| Dijksterhuis 1965 | force | — | — | — | none (kinematic) |
| Oledzki 1973 | force (step-wise) | displacements, velocities (10 points) | 2 | 2 ODEs | none |
| Mori 1997 | force (constant) | — | — | — | none |
| Hayashi 1999 | constant velocity or acceleration | hammer displacement, velocity | 2 | closed form | none |
| Van den Berghe 1995 | force profile | key and hammer displacements | 3 | DYNAST | none |
| Oboe 2006 | measured force | hammer displacement | 2 | — | none |
| Gillespie 2011 | measured force | hammer and key displacements | 5 | MATLAB Simulink/StateFlow | partial: rubber added between the bodies to regularize |
| **Hirschkorn 2004/2006** | **measured force** | **hammer, key (and wippen) displacements** | **5** | **DynaFlexPro → MATLAB ODE15s** | **the dynamics prototype's action: geometry, masses, 13 contact laws and 5 axis frictions measured on one action (FOUNDATIONS §5.2 b)** |
| Izadbakhsh 2006/2008 | measured force | hammer, key, wippen displacements, flexible shank | 5 | DynaFlexPro → Maple lsode | extension |
| Links 2011 | force | kinematics | 5 | index-3 DAE → ODE | — |
| Bokiau 2012 | force profile | kinematics | 6 | ROBOTRAN | — |
| Masoudi & Birkett 2015 | measured force | key, hammer, wippen displacements | 5 | MATLAB ODE15s | upright action only |
| **Thorin 2013/2017** | **force AND position** | **FORCES + kinematics** | **5 (+1 damper in the thesis)** | **measure differential inclusion, Moreau–Jean, XDE** | **the only published key force under position drive (FOUNDATIONS §5.2 c)** |
| Timmermans, Dehez, Fisette 2020 | real time | haptic force | — | symbolic ROBOTRAN, C | **[?]** a real-time haptic key; not in the corpus, unchecked (FOUNDATIONS §10) |
| Timmermans, Ceulemans, Fisette 2021 | force (1 kg at the key tip; a double blow) or key position (a sinusoid) | key and hammer angles, high-speed camera at 2000 fps | 6 (grand), 5 (upright) | symbolic ROBOTRAN, a two-stage W-method at a fixed 10 µs | the grand model is Bokiau's, its masses measured on a Renner demonstrator and 12 of its 13 contacts fitted on a tribometer; the curves are given as figures only [V, Timmermans 2021 §3–§6] (§7.5) |
| Fisette, Bokiau, Timmermans 2019 | key motion: measured under a 1 kg mass, or Goebl's key velocities integrated | hammer angle by high-speed camera (3000 fps); hammer velocity against Goebl's | 5 (no damper) | symbolic ROBOTRAN | teaching: one model of a demonstrator action, degraded by locking or removing parts into an action without escapement, then with a single escapement, then whole; no parameter published [V, Fisette 2019 §2–§3] |

Every row but the last four is in Thorin's Table 2 [V, Thorin 2017, Table 2]; the
gloss on Gillespie's rubber is Thorin's [V, Thorin 2013 §2.5, p. 29].

---

## 2. THORIN ET AL. 2017: THE MODEL TO MATCH

**Exact citation** [V, Thorin 2017, first page]: A. Thorin, X. Boutillon, J. Lozada,
X. Merlhiot, "Non-smooth dynamics for an efficient simulation of the grand piano
action", **Meccanica** 52, 2837–2854 (2017), DOI 10.1007/s11012-017-0641-1, received
9 July 2016, accepted 17 February 2017.

### 2.1 Bodies and degrees of freedom

Five planar rigid bodies, plus the damper in the thesis, removed from the article
"for the sake of simplicity and clarity of the conclusions" [V, Thorin 2017 §2]:

| Body | Coordinate | Axis of rotation | Remark |
|---|---|---|---|
| Key | θ_K | fixed to the frame | in reality **2 DOF** (rotation, and vertical translation on the balance rail pin) |
| Wippen | θ_W | fixed to the frame | |
| Jack | θ_J | **carried by the wippen** | non-inertial frame |
| Repetition lever | θ_L | **carried by the wippen** | non-inertial frame |
| Hammer | θ_H | fixed to the frame | |
| Damper | θ_D | fixed to the frame | thesis §4.5.7 |

Choice of coordinates: one angle per body, measured from ground, the jack's and
the lever's axes carried by the wippen, as in the double pendulum of the thesis's
Appendix B [V, Thorin 2013 §4.6, Eqs. 4.13, 4.17–4.18, Appendix B]. These are
minimal coordinates: the jack and the lever are hinged to the wippen, the other
bodies to the frame, and every other interaction is a contact. The simulator keeps
the same tree with the jack's and the lever's angles taken relative to the wippen,
as Hirschkorn does [V, Hirschkorn 2004 §2.1, Eq. 2.1] (planned, FOUNDATIONS §3.6).

**Both groups report the same modelling gap**: Thorin blocks the vertical
translation of the key, replacing the small felt under the key at the balance rail
with a metal ring of the same thickness, because the experiments proved hard to
repeat with it; he concedes that "such a modification would not be acceptable in
practice (it makes the action fairly noisy)" [V, Thorin 2017 §2.2]. Hirschkorn
sees the key lift off the balance rail on his encoders and his high-speed video,
which "allows motion in the key that is not permitted by the perfect revolute joint
in the model" [V, Hirschkorn 2004 §3.2.2, §4.1].

### 2.2 The contacts

Thorin, 14 named contacts [V, Thorin 2013, Table 4.2 and Eq. 4.20]:

| Name | Body A | Body B | Tangential friction |
|---|---|---|---|
| KS+ | key | frame at the key front (front rail punching) | no |
| KW | key (capstan) | wippen | no |
| KS− | key | frame at rest (back rail cloth) | no |
| KH | key (back check) | hammer | **YES (ν_KH)** |
| KD | key | damper | no |
| WJ | wippen | jack | no |
| WL | wippen | repetition lever | no |
| JG | jack | frame at the let-off button | **YES (ν_JS)** |
| JL | jack | repetition lever | no |
| JH | jack | hammer knuckle (roller) | **YES (ν_JH)** |
| LG | lever | frame at the drop screw | no |
| LH | lever | hammer knuckle | no |
| HG | hammer | frame (string) | no |
| DG | damper | frame at rest | no |

(The table prints the wippen's two as KJ and KL; the vector of compressions,
Eq. 4.20, names them WJ and WL. In §4.5 the frame's contact points are named S_J,
S_L, S_S, S_D, S_K±.)

Hirschkorn, 13 contacts (Table 2.1): the same topology without the damper's two,
and with the hammer shank's rest in addition [V, Hirschkorn 2004, Table 2.1]:
`1 key-wippen at the capstan (circle-circle) | 2 key-frame at the key front (line-circle) | 3 key-frame at the key back | 4 key-hammer at the back check | 5 wippen-jack | 6 wippen-lever | 7 jack-hammer at the knuckle (hybrid-circle) | 8 jack-frame at the button | 9 jack-lever | 10 lever-hammer at the knuckle (hybrid-circle) | 11 lever-frame at the stop | 12 hammer-frame at the string | 13 hammer-frame at rest`

Tangential friction is kept only on the contacts with significant sliding: **1, 4,
7, 10** for Hirschkorn [V, Hirschkorn 2004 §2.2.4]; **KH, JH and the jack against
the let-off button** for Thorin [V, Thorin 2013 §4.3.5; Thorin 2017 §2.4]. Thorin's
justification [V, Thorin 2017 §2.4]: *"Friction between the jack and the roller is
easily felt by the pianist and must also be considered in the model, because the
piano technicians adjust it carefully."*

**Contact geometries**: three primitives are enough, in both models:
- **circle** (centre in polar coordinates ρ, θ from the centre of the contact zone
  P, and a radius)
- **segment / line** (an angle θ in the body frame and a length λ)
- **hybrid**: an arc and two tangent straight lines (for the jack's top and the
  lever under the knuckle)

Thorin's breakdown [V, Thorin 2017 §2.4, Table 4]: back check against hammer, two
circles; jack against roller, two perpendicular straight lines joined by an arc;
**the ten others, a circle and a segment**. Hirschkorn's remark [V, Hirschkorn 2004
§2.2.1]: the felt under the key front is physically a flat horizontal disc, but
"*treating it as a circular contact can still produce the correct penetration
depth*"; the geometry serves detection and penetration depth, not the force law.

---

## 3. FELT / LEATHER FORCE LAWS: the two families

### 3.1 Thorin / Brenon: a three-parameter visco-elastic law

```
F_felt(δ) = k·δ^r + b·δ²·δ̇            (Thorin 2017 Eq. 6; thesis Eqs. 2.3 and 4.4)
```
δ = felt compression (m), δ̇ = compression velocity. Experimental source: C. Brenon,
*Mécanique de la touche de piano* (piano action), Master's thesis, Université
Pierre et Marie Curie, Paris VI, 2002; measured again by Lozada 2007 [V, Thorin
2017 §3.1].

Published values [V, Thorin 2013 §4.5.10, Eqs. 4.9–4.10]:

| Felt | k | r | b |
|---|---|---|---|
| **hammer knuckle (JH) and lever against the frame (drop screw)** | 7 × 10⁹ SI | **3.0** | 5 × 10⁷ SI |
| **all other felts** | 1.6 × 10¹⁰ SI | **2.7** | 5 × 10⁷ SI |

(Unit of k: N·m^−r; of b: N·s·m⁻³; Thorin writes "SI". Thorin's Table 2.2 gives
b = 2 × 10⁹ N·s·m⁻³ for the front rail punching of his one-degree-of-freedom model,
"arbitrarily taken 100 times more than that measured by Brenon" to absorb the back
check's dissipation; not to be taken over as such [V, Thorin 2013 §2.3, p. 24,
Table 2.2].) Lineage: Thorin attributes his b = 5 × 10⁷ to Lozada, whose own table
gives 2 × 10⁷ for the felts, and 1 × 10³ and 3 × 10³ for the leathers of the roller
and of the drop screw [V, Thorin 2013 Eqs. 4.9–4.10; Lozada 2007, Table 2.3]
(FOUNDATIONS §6.3).

Energy balance computed by Thorin: for 1 mm of compression applied linearly over
0.1 s, W_elastic / W_viscous = **22**; "the viscous part was not neglected as the
ratio was not judged sufficiently greater than 1" [V, Thorin 2013 Eq. 4.11].

Contact implementation [V, Thorin 2017 §2.4]: each felt is a **small 1 mg
auxiliary rigid body "A"**, connected to its support by a **prismatic joint** and a
**Kelvin–Voigt element**, with a unilateral non-penetration constraint against body
"B" and an **inelastic impact law** (to allow permanent-contact phases). A stated
limitation: in XDE, the nonlinear law Eq. (6) is implemented by **readjusting the
stiffness and viscosity of the linear element at every time step** [V, Thorin 2017
§4].

### 3.2 Hirschkorn: modified Hunt–Crossley with a fitted cubic

```
f_n = f_fit(x) · (1 + D·ẋ)              (Eq. 2.4)
f_fit(x) = a·x³ + b·x² + c·x            (Eq. 2.5)
```
An earlier version of his model used Hunt–Crossley proper, `f_n = k·xⁿ (1 + D·ẋ)`
with n usually 3/2; "the main discrepancy was that the power function for the
spring force was not the same form as the actual felt profile" [V, Hirschkorn 2004
§2.2.2]. He **explicitly rejects**:
- the **Dunlop** model: the wrong form (it gives a displacement for a force, where a
  force is needed for a displacement), no way to calibrate it, and Stamm's tests cast
  doubt on its ability to reproduce Dunlop's own data;
- the **Gonthier** model: its damping factor depends on the **initial contact
  velocity**, so it cannot be used when the bodies **start in contact** and may stay
  so for the whole simulation. ← **decisive for this case** (at rest, key, wippen,
  jack, lever and hammer are all in contact) [V, Hirschkorn 2004 §2.2.2].

Calibration of D [V, Hirschkorn 2004 §2.2.3, Eq. 2.7]:
`D = (f_loading(x_max)/f_fit(x_max) − 1) / ẋ_ave`, where x_max is the point of
largest difference between the loading curve and the fitted curve, and ẋ_ave the
mean normal contact speed measured on a real forte blow. The cubic itself is fitted
to the mid-line between the loading and unloading curves [V, Hirschkorn 2004
§2.2.3].

**TABLE A.8: the 13 contact laws, measured one by one [V, Hirschkorn 2004, Table A.8]**
(a in N/m³, b in N/m², c in N/m):

| # | Points | a | b | c | D | A (μ_k) | B1 | B2 |
|---|---|---|---|---|---|---|---|---|
| 1 | Pk3-Pw5 (capstan) | 1.26E+10 | 3.26E+07 | 7.92E+03 | 1.877 | 0.167 | 0 | 1 |
| 2 | Pk4-Pg4 (key front) | 4.41E+10 | −1.02E+07 | 1.57E+04 | 1.219 | 0 | 0 | 0 |
| 3 | Pk5-Pg5 (key back) | 2.42E+10 | −1.11E+07 | 6.54E+03 | 2.502 | 0 | 0 | 0 |
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

In the corpus, this is the **only set of a grand action calibrated contact by
contact**. Order of magnitude: under the key front, the measured loading curve
reaches about 35 N at 0.86 mm of penetration, where the measurement stops [V,
Hirschkorn 2004, Figures 2.5 and 2.6, pp. 20 and 22]. **Do not use** "about 40 N
near 1 mm": 40 N and 1 mm are where the figures' axes end, and contact 2's cubic
taken to 1 mm, 44.1 − 10.2 + 15.7 = 49.6 N, extrapolates past the measured range.
Despite the negative b values, all thirteen elastic branches are positive and
strictly increasing for any positive penetration [E, 04, 05]; where b < 0 the
stiffness dips below c before it rises [E, 05]. The clamp ≥ 0 is needed on the
damped law, which turns attractive below ẋ = −1/D [E, 04] (FOUNDATIONS §4.3).

Compared at equal force, from 10 to 100 N, Hirschkorn's jack/knuckle law is 3.1 to
3.6 times stiffer in tangent stiffness than Thorin's knuckle felt (§3.1) [E, 05].
**Do not use** "a factor of 56" between them: it is the ratio of the cubic
coefficients, 3.92e11 against 7e9. The two laws come from different actions,
benches and law forms, and enter the data as two labelled material sets, never
merged (FOUNDATIONS §4.3).

### 3.3 What recalibrating the damping shows

Hirschkorn's model, calibrated on forte speeds, lags the experiment on a piano blow
[V, Hirschkorn 2004 §4.2.1]. He recalibrates all the D values on piano speeds; the
new values are **four to five times larger**, and "it seems that retuning the
damping parameters has very little effect on the behaviour of the system. The
results are nearly identical to the positions of the model with the damping
calibrated to forte speeds" [V, Hirschkorn 2004 §4.2.2]. He calls those results
"less legitimate than the previous", the new speeds having been taken from the
model rather than the experiment; and "after retuning the damping parameters, the
same discrepancy exists, indicating that the previous explanation is not correct".
He points instead at unloading curves that would differ at the lower peak forces of
a piano blow, and at effects "such as friction" that "might be more significant at
lower forces" [V, Hirschkorn 2004 §4.2.2].

→ **Do not use** "recalibrating the damping by four to five times changes the
kinematics negligibly, so the exact hysteresis of the felt can wait and the
elastic part does all the work": the comparison is of positions under force drive,
which an inertia-dominated model passes whatever its felt (§11), and the
recalibrated model keeps the piano blow's discrepancy. One damping constant cannot
serve from pianissimo to fortissimo [E, 07], and the felt law is a hypothesis to
test on Hirschkorn's piano and forte blows (FOUNDATIONS §4.3).

### 3.4 The real behaviour of felt

The three characteristics Hirschkorn observes [V, Hirschkorn 2004 §2.2.2]:
1. **nonlinear spring**: "*It also does not follow any simple power function*";
2. **hysteretic**: "*The felt requires more force to compress than it returns on
   decompression*";
3. **INDEPENDENT OF SPEED**: "*Whether the felt is loaded slowly or quickly seems to
   have little effect on the loading profile. Instead, it is dependent on the
   maximum and minimum loads the felt has previously undergone.*"

Hirschkorn states that his law is wrong on point 3: "The biggest weakness of this
model is that the hysteretic damping behaviour is speed dependent, and it is known
from previous research (Stamm) that the behaviour of real felt is not" [V,
Hirschkorn 2004 §2.2.2], and he proposes other hysteresis models, "such as the
Preisach Model" [V, Hirschkorn 2004 §5.2]. Thorin notes that the loading curves of
hammer felt are "nearly velocity-independent", with a "small but significant
dependency in velocity (b ≠ 0)", and keeps a three-parameter law that spans the
dynamic range from piano to forte [V, Thorin 2017 §2.4].

A published speed-independent model is **Masoudi & McPhee**, "A Novel
Micromechanical Model of Nonlinear Compression Hysteresis in Compliant Interfaces of
Multibody Systems", *Multibody System Dynamics*, DOI 10.1007/s11044-015-9483-6 [V,
`masoudi_preprint.pdf`]:
- fibre-network micromechanics (after Carnaby and Pan), hysteresis through
  **fibre-to-fibre friction** → a **speed-independent** response [V, abstract];
- `P_loading = σ_el + r_sc·n·λ³·C_s` and `P_unloading = σ_eu − r_sc·n·λ³·C_s`
  (Eqs. 9–10), with σ_el ≠ σ_eu;
- it works in **macroscopic pressure (Pa) against volume fraction**; identified on a
  piano hammer felt (v_f0 = **0.389**) and a moderate red felt (v_f0 = **0.086**),
  pressures up to about 3.5 × 10⁵ Pa [V, Figures 6–7]; applied to an upright piano
  action (§4);
- it needs **a measured loading and unloading curve per felt**, and an algorithm
  for the transitions between loading and unloading [V, abstract, §3].

FOUNDATIONS §4.3 makes a rate-independent law with loading and unloading branches
and a memory of its turning points, after Masoudi and McPhee, the target of the
felt-law hypothesis, Hirschkorn's cubics being its elastic term. Its identification
needs loading and unloading curves contact by contact, which Hirschkorn plots for
the key front alone and no source in the corpus gives for a grand action. **Do not
use** "no published rate-independent felt model": Masoudi and McPhee publish one
(FOUNDATIONS §4.3).

---

## 4. FRICTION AT THE PIVOTS: two laws, and the lineage of the published values

### 4.1 Analytic forms

**Thorin [V, Thorin 2017 Eqs. 2–4]**: NON-regularized (set-valued) dry friction and
linear viscous friction:
```
C_d(θ) = c_d · sign(θ̇)     with sign(0) = [−1, 1]   (a set)      (Eqs. 2–3)
C_v(θ) = c_v · θ̇                                                   (Eq. 4)
```
**Hirschkorn [V, Hirschkorn 2004 Eqs. 2.10–2.12]**: Coulomb friction regularized by a
tanh (after Cull & Tucker):
```
μ = A·(tanh(s_t/v_t) + B1·(s_t/v_t) / (1 + B2·(s_t/v_t)⁴))    f_t = μ·f_n   (Eqs. 2.10-2.11)
T = A·(tanh(ω/ω_t) + B1·(ω/ω_t) / (1 + B2·(ω/ω_t)⁴))          (pivots, Eq. 2.12)
```
A is the kinetic coefficient, reached as the speed rises; B1 and B2 are adjusted so
that the **peak** of the curve matches the static coefficient [V, Hirschkorn 2004
§2.2.4]. At his four sliding contacts the peaks are 1.69, 1.72 and 1.13 times the
kinetic coefficient at the back check, the jack/knuckle and the lever/knuckle, and
none at the capstan [E, 07]. **Do not drop the bracket**: A multiplies both terms
on the typeset page (pp. 25–26), and a plain-text extraction of the thesis loses
it, giving a different law with peaks of 3.0, 6.6 and 1.9. For the pivots, the torque **does not depend on any
normal force**: "*The friction in the joints is primarily determined by the
tightness of the felt bushings (ie., large pin in a small hole), which is constant
once the action is assembled*" [V, Hirschkorn 2004 §2.2.4]. ← **a constant friction
torque per pivot, not proportional to a load.** His joints carry no static peak:
B1 = B2 = 0, A "derived directly from the kinetic frictional torque in the joint",
measured as the force that turns the joint at a known distance from its centre
[V, Hirschkorn 2004 §A.4, Table A.9].

### 4.2 The published dry torques, and where each comes from (N·m)

| Body | Hirschkorn, Table A.9 (kinetic, measured) | Lozada 2007, Table 2.3 | Thorin 2013, §4.5 | Lineage of the Lozada/Thorin value |
|---|---|---|---|---|
| Key | **0.0122** | 0.05 dry, **0.22 N·m·s** viscous | 0.006 dry, 0.22 N·m·s viscous (§4.5.2); 0.012 and 0.022 in Table 2.1 | measured on the key by Lozada (Table 3.1): three published dry values |
| Wippen | **3.97E−4** | 9.92E−2 | 0.0992 | assumed equal to the hammer's: printed 100 times too large |
| Jack | 3.97E−4 | — | 9.9E−4 | equal to the hammer's |
| Repetition lever | 2.47E−3 | 9.92E−4 | 9.9E−4 | assumed equal to the hammer's |
| Hammer | **1.01E−3** | **9.92E−4** | 9.9E−4 | **measured** (Lozada's free-pendulum test, Eq. 3.33) |
| Damper | — | 9.92E−2 | 0.0992 | assumed equal to the hammer's: printed 100 times too large |

Sources [V, each]: Hirschkorn 2004, Table A.9; Lozada 2007, Table 2.3, Table 3.1 and
Eq. 3.33; Thorin 2013 §4.5.2–§4.5.7 and Table 2.1.

**Lozada measured the hammer's axis friction and assumed the rest**: all the
pivots other than the key's "are made in a similar way and rarely checked by the
tuner. We therefore identified the hammer–frame joint and considered that the
other joints behave in the same way" (translated from his French) [V, Lozada 2007,
ch. 3 §1.3.1, p. 57].
So the wippen, jack, lever and damper values copy the hammer's 9.92e-4 N·m, and
their agreement with Hirschkorn's compares nothing measured; the wippen's and the
damper's 9.92e-2, a hundred times the hammer's, are by his own assumption a
transcription error (FOUNDATIONS §6.3). Thorin takes the pivot frictions from
Lozada's measurements "performed on a different copy of the same action model"
[V, Thorin 2017 §3.1].

**Thorin's key viscous friction, 0.22 N·m·s, is a measurement, not a typo**:
Lozada's linear regression on the key gives a dry torque of 0.05 N·m and a viscous
coefficient of 0.22 N·m·s [V, Lozada 2007 ch. 3 §1.1, Table 3.1], his measured
torque–speed curve reaching about 0.3 N·m in magnitude near 1.2 rad/s [V, Lozada
2007, Figure 3.5]. The key's dry friction has three published values, 0.05 N·m
(Lozada), 0.012 (Thorin 2013, Table 2.1) and 0.006 (Thorin 2013 §4.5.2), kept side
by side with their sources (FOUNDATIONS §6.3). **Do not use** "c_vK = 0.22 is a
typo for 0.022": it is Lozada's measurement. The wippen's and the damper's 0.0992
are a hundred times the hammer's, but what settles them is Lozada's statement that
he assumed those joints equal to the hammer's, not the argument that a friction
ten times the wippen's weight torque would freeze it. **Nor use** the "recommended
set after adjudication" (key 0.012, wippen and jack 4e-4 to 1e-3, lever 1e-3 to
2.5e-3, hammer 1e-3, damper about 1e-3 N·m, key viscous 0.022 N·m·s): it reads a
measurement as a typo and spreads one copied value over ranges as if each pivot
had been measured.

### 4.3 How much of the friction the axis torques explain

Thorin measures his demonstrator's friction torque at the key by the static test,
`C_d = L(F_down − F_up)/2 = 0.039 N·m` with L = 0.245 m [V, Thorin 2013 §2.2,
p. 21; §2.3, Eq. 2.8, Table 2.2]. **Do not use** "the pivots account for only
~59 % of the total friction, the remaining ~41 % coming from the sliding contacts
and the key mortises", nor the sum it rests on (0.012 + 2.15 × 4e-4 + 10.1 × 1e-3
= 0.0230 N·m):
it leaves out two of the five pivots, the repetition lever among them, mixes
Hirschkorn's torques with Thorin's small-angle ratios and arm, and counts measured
sliding-contact friction as missing [E, 06] (FOUNDATIONS §6.5). Any unexplained
share comes from an assembled model over all five pivots and the sliding contacts,
against a measured F.

Hirschkorn's own diagnosis of his double blow [V, Hirschkorn 2004 §4.3]: "*This
would indicate that the friction in the model action is too low, or that there is
some friction in the experimental action that is not modelled. This low friction
is less significant under the high forces that occur in a single forte blow, but
are more important during the piano blow, which has lower forces, and the double
blow*". He also calls his friction parameters "less reliable" than his dimensions
and masses [V, Hirschkorn 2004 §5.2].

Dedicated work: Masoudi's Waterloo PhD thesis (2012), *Micromechanics of Fiber
Networks Including Nonlinear Hysteresis and its Application to Multibody Dynamic
Modeling of Piano Mechanisms*, as cited by Thorin [V, Thorin 2013, bibliography]:
**[?]** not in the corpus. The corpus holds no measured friction value for the key
bushings (balance and front mortises).

---

## 5. REPETITION SPRING (and the other springs)

**Hirschkorn [V, Hirschkorn 2004 §A.4]**:
- jack spring: **roughly constant torque, 0.0042 N·m**;
- repetition lever spring: **linear**, `T = −0.073·(θ_r − 0.832)` N·m, θ_r the angle
  between the lever and the wippen in radians → κ = **0.073 N·m/rad**, θ₀ =
  **0.832 rad**.

**Thorin** [V, Thorin 2017 Eq. 5]: the generic form `C_s(θ) = κ·(θ − θ₀)`, three
springs measured by Lozada [V, Thorin 2013, Table 4.12; Lozada 2007, Table 2.3]:

| Coupling spring | κ (N·m/rad) | θ₀ (rad) |
|---|---|---|
| wippen — frame | 0.0355 | 0.419 |
| **jack — wippen** | 0.0299 | 0.980 |
| **lever — wippen** | **0.1171** | **0.725** |

Thorin's one-degree-of-freedom model lists the wippen spring as κ₂ = 0.087 with
θ₂₀ = 0.42 rad [V, Thorin 2013, Table 2.1], beside the 0.0355 of Table 4.12: two
values, kept with their sources.

Lever spring: 0.1171 (Thorin) against 0.073 N·m/rad (Hirschkorn), a factor of 1.6,
which is no anomaly: spring tension is a regulation setting, adjusted by bending the
wire (FOUNDATIONS §6.1). **Do not compare the rest angles**, 0.725 and 0.832 rad:
each is measured in its author's own body frames, which differ.
Thorin's action has a **wippen–frame spring**, which Hirschkorn's Boston action does
not; the support spring is fitted on some actions only, and is optional in the
configuration (FOUNDATIONS §6.1).

---

## 6. DAMPER AND PEDALS

**Damper: modelled in Thorin's thesis only** [V, Thorin 2013 §4.5.7], and left out
of the 2017 article "for the sake of simplicity and clarity of the conclusions",
which therefore simulates the action with the damper pedal engaged [V, Thorin 2017
§2].

| | value [V, Thorin 2013 §4.5.7, Figure 4.19] |
|---|---|
| mass m_D | **50.06 g** (Lozada's table: 50.6 g [V, Lozada 2007, Table 2.3]) ← the second heaviest body after the key |
| inertia I_D,DO | 2.04 × 10⁻⁴ kg·m² |
| centre of gravity (DG) | (50.0 ; 0.0) mm from the axis DO |
| contact with the key (DK) | (78.1 ; 0.0) mm from DO, circle ρ = 5 mm, θ = 1.57 rad |
| rest stop (DS) | (43.0 ; −22.0) mm, circle ρ = 100 mm, θ = 1.57 rad |
| angle at rest θ_D(0) | −0.23 rad [V, Thorin 2013 §4.5.9] |
| frictions | c_dD printed 0.0992 N·m, assumed equal to the hammer's 9.92e-4 (§4.2); c_vD = 4.93 × 10⁻⁵ N·m·s |
| regulation | the spoon touches the key when **the hammer is half-way (23.5 mm)** [V, Thorin 2013, Appendix A] |

Thorin's regulation procedure specifies the weighing **"without damper"** [V,
Thorin 2013, Appendix A]: a 50 g damper, lifted by the key from half-way through
the hammer's travel, would weigh in the down weight.

**PEDALS: NOTHING IN THE CORPUS.** No source in it models the damper pedal (lift
tray), the sostenuto or the una corda of a grand dynamically; Thorin and Hirschkorn
simulate **one isolated note**. This is therefore:
- **original development**, geometrically simple:
  - **damper pedal**: a common lift rail pushing the 88 damper under-levers → **one
    scalar** for the tray's position, prescribed by the pedal input, and a
    unilateral contact per under-lever (FOUNDATIONS §6.6, §7.6);
  - **sostenuto**: a rod that **captures** the tabs of the under-levers already
    raised → **one scalar** and one state bit per note;
  - **una corda**: the key frame and the hammers **shift sideways**, not the dampers,
    out of the sagittal plane → **one scalar**; it does NOT affect the sagittal
    dynamics, only where the hammer meets the strings and which part of its felt
    (FOUNDATIONS §7.6; what else moves with it is a question for the owner,
    FOUNDATIONS §9).
- half pedalling and the effect of the open dampers on sympathetic resonance are
  subjects of their own, not covered.

---

## 7. THE PARAMETER SETS

### 7.1 Hirschkorn 2004, Appendix A: the **Boston** GP-178 action, key 52, in collaboration with Steinway & Sons New York [V, Hirschkorn 2004 §2, acknowledgements]

**Table A.1, inertias** (I about the centre of mass; centre of mass in the body
frame) [V, Hirschkorn 2004, Table A.1]:

| Body | mass (kg) | I_cm (kg·m²) | CoG x (m) | CoG y (m) |
|---|---|---|---|---|
| Key | 0.12008 | 2.75E−03 | 0.0514 | 0.0158 |
| Wippen | 0.01196 | 1.69E−05 | 0.0502 | −0.0018 |
| Jack | 0.00265 | 7.78E−07 | 0.0008 | 0.0195 |
| Repetition lever | 0.00371 | 4.10E−06 | 0.0021 | −0.0012 |
| Hammer | 0.01174 | 2.90E−05 | −0.1022 | 0.0145 |

About their own pivots, by the parallel-axis theorem: key 3.0972e-3, wippen
4.7078e-5, jack 1.7874e-6, lever 4.1217e-6 and hammer 1.5409e-4 kg·m² [E, 01].

Measurement method [V, Hirschkorn 2004 §A.1]: masses on a lab scale; centres of
mass by suspending each body from a thread at two points; inertias by
**pendulum oscillation**, `I_p = m·g·l·(T/2π)²`, then `I_cm = I_p − m·l²` (parallel
axes). Geometry [V, Hirschkorn 2004 §A.2]: the bodies **scanned on a flatbed
scanner**, the images scaled to known dimensions, traced and measured in AutoCAD.

**Tables A.2 to A.7, geometry**: coordinates (x, y) in m, unit normals, radii and arc
ranges for **all** the contact points of the five bodies and the frame (Pg1…Pg9,
Pk2…Pk6, Pw2…Pw6, Pj2…Pj5, Pr2…Pr5, Ph2…Ph5), extracted in full in
`extracts/h.txt`. Examples [V, Hirschkorn 2004, Tables A.5–A.7]: knuckle Ph2 =
(−0.0170 ; −0.0091), r = 0.0044 m; hammer head Ph4 = (−0.1321 ; 0.0356), r =
0.0102 m; jack top Pj2 = hybrid, r = 0.0010 m, arc 1.571–3.142 rad; lever Pr2 =
hybrid, r = 0.0552 m, arc 1.300–1.571 rad.

**Reading the frames**: every body frame has its origin at the body's pivot, as
Figure A.1 draws it; the ground pins are Pg1 (key), Pg2 (wippen) and Pg3 (hammer),
and in the wippen's frame **Pw2 is the jack's pin and Pw3 the lever's**, both listed
as "Pin Location" [V, Hirschkorn 2004, Figure A.1, Tables A.2–A.4]. The key's frame
is centred on the balance pin: the front punching's contact Pk4 sits at +0.2175 m,
against the ground's Pg4 at +0.2172 m, and the capstan Pk3 at −0.1176 m. Pk2, listed
as "Pin Location" in the key's table at (0.2278 ; 0.0261), is drawn in Figure A.1 at
the top of the key front, under the arrow of the force the model is driven by [V,
Hirschkorn 2004, Figure A.1, Table A.3, §2.1]. **Do not take Pw2 for the wippen's
pivot** (FOUNDATIONS §6.4). Read this way, the action closes at rest: contacts 1, 3,
5, 7 and 10 closed, the hammer 47.0 mm from the string against the 48.0 mm he
regulated, the shank 0.93 mm above its rest rail, and one interpenetration, 0.88 mm,
at the repetition lever's stop [E, 10], which fits his regulating the model by
editing geometric parameters (§13).

### 7.2 Thorin 2013, §4.5: the isolated **Renner** demonstrator

Convention [V, Thorin 2013 §4.5.1]: lengths in **mm** in each body's frame, **origin
= the body's axis of rotation** (BO), angles in rad; the horizontal axis along a
main direction of the body. The demonstrator was "carefully adjusted by a
professional piano technician", and its key "is not weighted with key leads" [V,
Thorin 2013 §1.6, p. 15]. Its geometry was taken from photographs and, "because of
the time-worn felts of the used action", checked with a dial calliper [V, Thorin
2013 §4.1, p. 48]; the inertial and material properties come from Lozada, except
the key's mass [V, Thorin 2013 §4.1, p. 49].

**Inertias and frictions** [V, Thorin 2013 §4.5.2–§4.5.7, Figures 4.9–4.19], with
each value's lineage:

| Body | mass | I about the axis | CoG (mm) | c_d (N·m) | c_v (N·m·s) | lineage |
|---|---|---|---|---|---|---|
| Key | 0.083 kg | 3.36E−03 kg·m² | (19.0 ; 14.0) | 0.006 (§4.2) | 0.22 | mass estimated by Thorin, 20 % below Lozada's 0.103 kg; inertia Lozada's, measured on his 103 g key; viscous friction measured (§4.2) |
| Wippen | 19.8 g | 3.97E−04 | (53.0 ; 7.0) | 0.0992 printed | 4.93E−05 | Lozada's; the friction assumed (§4.2); the inertia implies an impossible radius of gyration (below) |
| Jack | 5.0 g | 5.0E−06 | (0 ; 0) | 9.9E−04 | 4.93E−05 | **placeholders**: category IV, "not to be studied", round values, centre of mass at the pin |
| Lever | 10 g | 5.0E−06 | (0 ; 0) | 9.9E−04 | 4.93E−05 | **placeholders**, as the jack's |
| Hammer | 14.24 g | 1.65E−04 | (−95.7 ; 23.7) | 9.9E−04 | 4.93E−05 | Lozada's free-pendulum identification, the one axis friction measured |
| Damper | 50.06 g | 2.04E−04 | (50.0 ; 0) | 0.0992 printed | 4.93E−05 | Lozada's; the friction assumed (§4.2) |

Thorin classes the jack's and the lever's mass, inertia, centre of mass and
friction in category IV, "fixed parameters not to be studied" [V, Thorin 2013
§4.4.1, Figures 4.13, 4.15]. The wippen's inertia, 3.97e-4 kg·m² about its axis
for 19.8 g with its centre of mass 53.5 mm away, implies a radius of gyration
about the centre of mass of √((3.97e-4 − 0.0198 × 0.0535²)/0.0198) ≈ 131 mm, for a
part about 100 mm long (FOUNDATIONS §6.3).

**Contact points** [V, Thorin 2013 §4.5.2–§4.5.8, Figures 4.9–4.21]:
- **Key**: KD (damper) −229.5/27.6 · KH (back check) −220.0/84.2 · KS− (rest)
  −183.0/0 · **KW (capstan) −128.7/44.6** · KO (axis) 0/0 · **KS+ (front)
  +241.5/0**
- **Wippen**: WO 0/0 · WL 16.1/2.0 · **LO (lever axis) 47.5/29.8** · WK (capstan)
  59.0/−20.8 · WJ 75.8/20.8 · **JO (jack axis) 99.1/0**
- **Jack**: JW −13.4/24 · JH (top, under the roller) −8.0/48.8 · JL −3.4/43.7 · JO
  0/0 · **JS (let-off button) +26.8/0**
- **Lever**: LW −41.0/−13.3 · LO 0/0 · LH 35.6/2.0 · LJ 47.3/−3.0 · **LS (drop
  screw) 54.5/0**
- **Hammer**: **HK (hammer tail, at the back check) −127.9/−21.3** · **HS (string)
  −126.4/54.9** · **HJ (knuckle) −16.3/−12.0**, 20.2 mm from the axis · HO 0/0
- **Frame**: axes S_OD −308.1/39.6 · S_OW −188.5/62.06 · S_OH −96.9/125.4 · S_OK 0/0;
  stops S_D −265.1/7.2 · S_S (string) −260.0/178.8 · S_K− −183.3/6.16 · **S_L (drop
  screw) −93.5/117.1** · **S_J (button) −63.5/73.52** · S_K+ +242.0/−0.38

**Angles at rest** [V, Thorin 2013 §4.5.9, Figure 4.23]: θ_D(0) = −0.23 · θ_W(0) =
−0.03 · θ_L(0) = 0.40 · θ_H(0) = 0.40 · θ_J(0) = 0.22 · θ_K(0) = 0.03 rad.

**Small-angle ratios of the one-degree-of-freedom model** [V, Thorin 2013 §2.3,
Table 2.1]: L = 0.245 m · l₁₂ = 0.129 · l₂₁ = 0.060 · l₂₃ = 0.080 · l₃₂ = 0.017 ·
**α₂₁ = l₁₂/l₂₁ = 2.15** · **α₃₁ = α₂₁·l₂₃/l₃₂ = 10.1**.
Reduced inertias: J₁ = 3.36E−3 · J₂ = 3.97E−4 · J₃ = 1.65E−4 kg·m²;
`J = J₁ + α₂₁²J₂ + α₃₁²J₃` = **0.0221 kg·m²**, the equivalent inertia of the
"simplistic key" [V, Thorin 2013, Eq. 2.2, Table 2.2].

### 7.3 The two sets side by side

| Quantity | Hirschkorn (Boston) | Thorin (Renner) |
|---|---|---|
| Hammer head, from the axis | 136.8 mm to the centre of its contact circle (Ph4) | 137.8 mm to the centre of its contact zone (HS) |
| Knuckle, from the hammer axis | 19.3 mm to the centre of its circle (Ph2) | 20.2 mm to the centre of its contact zone (HJ) |
| Key inertia about its axis | 3.10E−3 kg·m², 120 g key [E, 01] | 3.36E−3 kg·m², Lozada's 103 g key |
| Capstan, horizontal distance from the key axis | 117.6 mm | 128.7 mm |
| Front rail punching from the key axis | 217.5 mm | 241.5 mm |
| Front over capstan (geometric key ratio) | 1.849 | 1.876 |
| Hammer pivot dry friction | 1.01E−3 N·m | 9.92E−4 N·m, the one axis friction Lozada measured |

The two actions are of similar proportions. **Do not use** this table as two
independent campaigns converging "to better than 10 % on the inertias and the pivot
frictions": the Renner set's frictions are, but for the hammer's and the key's,
copies of one measurement, its jack and lever placeholders and its wippen's inertia
impossible (§4.2, §7.2), and distances between features say nothing about the moment
arms of the contact normals, which are what a lever ratio is made of (FOUNDATIONS
§6.2).

### 7.4 The action ratio

Thorin's published ratios are those of his simplified, small-angle model:
α₂₁ = 2.15 and α₃₁ = 10.1 (§7.2). From his §4.5 coordinates, α₂₁ comes out as
128.7/59.0 = 2.18, the horizontal distances of the capstan's contact on the key and
on the wippen. **Do not use** "α₃₁ ≈ 10.27, reconstructed from the coordinates":
its second factor, 80.0/17.0, is Table 2.1's own simplified model, so the agreement
is circular. **Nor** "R = α₃₁ × 137.8/245 = 5.68, squarely in Stanwood's normal zone,
the literature and the trade's metrology validating each other": a band of 5.0 to
7.0 cannot see a 10 % error in an arm (FOUNDATIONS §6.4). The action ratio is read
off an assembled model, by virtual work.

⚠️ **The ratio is not a constant.** α₂₁ and α₃₁ are small-angle linearisations about
the rest position [V, Thorin 2013 §2.3, Eq. 2.1]. Held constant over the stroke the
key drives before let-off (the dip less the aftertouch), α₃₁ would have the hammer
travel about 14 % further than it does to reach let-off, with mid-range regulation
assumed (46 mm blow, 1.5 mm let-off, 1.2 mm aftertouch) and a strike radius chosen to
reproduce the 57.6 mm below [E, 06]. **Do not use** "57.6 mm of hammer travel
against 45–47 mm regulated, about 25 %": it drives the hammer with the whole dip,
aftertouch included, at a strike radius no source gives [E, 06]. **The engine uses no
constant α**: the dynamics are to read the instantaneous ratio off the contact
geometry at every step (planned, FOUNDATIONS §6.2); the published α values serve only as consistency
checks of a model of Thorin's action at rest.

### 7.5 Timmermans 2021: a second Renner demonstrator, masses only

Timmermans, Ceulemans and Fisette measured the masses and moments of inertia of a
Renner grand demonstrator's six bodies [V, Timmermans 2021 §5.1, Table 3]:

| Body | Mass | Moment of inertia |
|---|---|---|
| Key | 94 g | 2110.62 kg·mm² |
| Wippen | 12 g | 10.78 kg·mm² |
| Jack | 2 g | 0.78 kg·mm² |
| Hammer | 17 g | 31.86 kg·mm² |
| Damper | 51 g | 126 kg·mm² |
| Repetition lever | 3 g | 4.32 kg·mm² |

The paper does not say about which point the inertias are taken, nor where any
centre of mass lies, nor any pivot or contact position: the set cannot build an
action. Read about the centre of mass, the radii of gyration are plausible, √(I/m):
key 150 mm, wippen 30 mm, hammer 43 mm; read about the axis, the wippen's 30 mm
would put its centre of mass within 30 mm of its flange, which Thorin's set (53 mm,
§7.2) does not. **[?]** which of the two, until the authors say. Beside Thorin's
Renner set it gives a second, independent wippen: 12 g against Lozada's 19.8 g, and
an inertia about 37 times smaller than Lozada's 3.97E−4 kg·m², the value §7.2 shows
impossible.

Its contact laws are smooth: Masoudi's exponential felt law F = a x e^(bx) + c x,
unloading switched by tanh(α ẋ), and Hirschkorn's tanh friction law at the
sliding contacts and the axes [V, Timmermans 2021 §4.2, Eqs. 2–6], which the
simulator does not use (FOUNDATIONS §3.5). The coefficients are fitted per contact
(one fit drawn, the key–wippen contact, Fig. 11) and not tabulated; the grand
model fits the loading curves only, unloading taken equal [V, §5.2]. One friction
coefficient is given in the text: about **0.65** between the hammer and the felt
of the back check, at sliding speeds the paper calls small beside those of play
[V, §5.2, Fig. 12].

---

## 8. SOLVERS, TIME STEPS, COMPUTATIONAL COST

### 8.1 Costs reported in the literature

| Work | Cost | Hardware | Slower than real time by |
|---|---|---|---|
| Van den Berghe 1995 (full) | about 4 h for 1 s [V, Thorin 2017, Table 1] | 486DX33 | **about 14,400** |
| Van den Berghe 1995 (reduced) | about 15 s for 1 s [V, Thorin 2017, Table 1] | DEC 5000/33 | about 15 |
| **Hirschkorn 2004** | **about 6 min per force profile** [V, Hirschkorn 2004 §2.4]; profiles of 0.70 s [V, §4.1] | 2.4 GHz PC, MATLAB ODE15s | about **500** (360 s / 0.70 s) |
| Izadbakhsh 2006 (flexible shank) | about 75 min [V, Thorin 2017, Table 1] | 2.4 GHz PC, Maple | not given: the simulated duration is not stated |
| Masoudi & Birkett 2015 (rigid) | 910 s [V, Thorin 2017, Table 1] | — | **about 4550**, Thorin's estimate from a 200 ms figure [V, Thorin 2017 §1] |
| Masoudi & Birkett 2015 (flexible shank) | 1720 s [V, Thorin 2017, Table 1] | — | about 8600 by the same estimate (1720/910 = 1.9 times the rigid model) |
| **Thorin 2017 (non-smooth, XDE 3D)** | **20 s for 1 s** [V, Thorin 2017 §6] | 2010 computer, 2.1 GHz Core i5 | **20** |
| Thorin, **before** the felt laws and the synchronisation were implemented | — | same | **1.2** [V, Thorin 2013 §6.4.1, p. 110] |
| Timmermans, Dehez, Fisette 2020 (haptic keyboard) | **[?]** real time reported; not in the corpus, unchecked (FOUNDATIONS §10) | — | — |
| Timmermans, Ceulemans, Fisette 2021 (grand, 6 rigid bodies, 13 contacts) | **0.87 s for 1 s** with the fixed-step W-method, 0.72 s with an error-controlled Rosenbrock integrator; the upright model 10 s for 1 s [V, Timmermans 2021 §3.2, Table 1] | not given | none: faster than real time (0.87) |

**Reading**: the move to non-smooth dynamics gains about two orders of magnitude, in
Thorin's words "about two orders of magnitude faster than the regularizing approach,
as reported in the literature" [V, Thorin 2017 §6]. Thorin attributes his remaining
factor of 20 to the implementation: the felt laws, re-set in a linear element at
every step, and the synchronisation slowed the computation down from about 1.2
times real time, and XDE works in 3D for a planar model; he expects more than 15
from the first and more than 3 from the second [V, Thorin 2013 §6.4.1, p. 110].
**Do not use** "UCLouvain 2018, compiled symbolic ROBOTRAN, 1.15 s for 1 s": no
source in the corpus gives it.

### 8.2 Thorin's numerical scheme

- Formalism: **measure differential inclusion (MDI)**:
  `M(x)dv = F_H(x, ẋ, t)dt + H(x)di`, `v⁺ = (ẋ)⁺`, `(g(x), Hᵀ(x)v⁺, di) ∈ K`
  [V, Thorin 2017 §2.5, Eqs. 8–14].
- The four inclusions: pivot dry friction `r_P ∈ c_d·sign(ẋ)` (r_P in ℝ¹², as
  printed); non-penetration `g(x) ∈ ℝ₊¹²`; normal reactions `r_N ∈ N_{ℝ₊¹²}(g(x))`
  (the normal cone); tangential Coulomb
  `[Hᵀ(x)v]_{3i−2,3i−1} ∈ N_{B(μ(r_N)_i)}((r_T)_{2i−1,2i})` [V, Thorin 2017 §2.5].
- Discretisation: **Jean's (1999) time-stepping**, Moreau–Jean. Linear
  interpolations `x_{n+α} = x_n + α·Δt·λ_x`, `v_{n+α} = v_n + α·Δt·λ_v` [V, Thorin 2017
  §4, Eqs. 15–16].
- Scheme parameters: **θ = 1** (stability of the smooth dynamics), **γ = 1** →
  "*constraints are satisfied at the end of every time step*" [V, Thorin 2017 §4].
- Solution: linearisation of H and of the smooth dynamics about the previous
  iterate, a **Newton loop** → a **One-Step Non-Smooth Problem**, reformulated with an
  **augmented Lagrangian** and solved by a **Gauss–Seidel-like algorithm** (Studer
  2009, ch. 4, as cited) [V, Thorin 2017 §4].
- **Time step: Δt = 0.5 ms (2 kHz)** [V, Thorin 2017 §5]. Convergence study
  (Fig. 12): the forces at 2 kHz and 10 kHz "do not significantly differ" [V,
  Thorin 2017 §5]; at 0.4 kHz the discrepancies are significant [V, Thorin 2013
  §6.4.1, Figure 6.10]. Characteristic time of the action: "*about a few
  milliseconds*" [V, Thorin 2017 §5].
- Key advantage: "*there is no numerical parameter for impact or dry friction calling
  for a very small time-step*" [V, Thorin 2017 §5].

Hirschkorn, by contrast [V, Hirschkorn 2004 §2.4]: MATLAB **ODE15s** (stiff, variable
step), absolute tolerance **1 × 10⁻⁵ rad**; "*the solvers dedicated to stiff systems
were much more efficient for this system. The contacts in the system are very stiff,
with forces changing drastically with small variations in position. Even more
significant in contributing stiffness is the friction model.*" His regularized
friction makes the system stiffer still than his contacts. The case for set-valued
friction rests on the non-physical threshold speed a regularized law brings into
every sensitivity, not on its time step: treated implicitly, a velocity-only
friction law sets no step limit [E, 07] (FOUNDATIONS §3.5).

### 8.3 Budget for 88 notes in the browser

- Per note, the step is set by a convergence study on the simulator's own solver
  (FOUNDATIONS §4.1); Thorin's 2 kHz holds for his scheme and his observables.
- **The decisive lever: freezing notes at rest.** The 88 notes share nothing but the
  damper lift tray, the sostenuto and the una corda shift, **three prescribed
  scalars** (FOUNDATIONS §7.6). A note at rest in permanent contact can be frozen, its
  steady state checked by a residual criterion. How many notes move at once in a real
  piece is an assumption [S] until counted over a corpus of MIDI files (FOUNDATIONS
  §4.6).
- Per note-step: 6 degrees of freedom, 13 to 17 unilateral contacts, of which 4 with
  tangential friction, and 5 or 6 pivots with set-valued friction.
- The hypothesis [S, to be tested by the prototype]: a dedicated 2D solver, not a
  general-purpose 3D engine, plays 88 notes in real time in WebAssembly on a few
  workers (FOUNDATIONS §4.6). The routes Thorin names for his own code [V, Thorin
  2017 §6]: "*include a more efficient implementation of the nonlinear springs laws
  and perform direct 2D dynamics*", with expected gains of more than 15 and more
  than 3 [V, Thorin 2013 §6.4.1, p. 110]. **Do not use** Thorin's 1.2 as "estimated
  for a dedicated 2D solver": it is his 3D engine's speed before the felt laws were
  implemented.

---

## 9. EXPERIMENTAL DATA AVAILABLE TO VALIDATE THE ENGINE

### 9.1 Hirschkorn 2004: complete test rig

Equipment [V, Hirschkorn 2004 §3.1, Appendix B]: a **Boston GP-178** keyboard
stripped to key 52, the string replaced by a solid metal bar · a **Kollmorgen
MT308A1** brushless motor driving the key through a 100 mm arm · a **Sensotec model
13 load cell** (1.1 g) on the key front, sampled up to 5 kHz · **MicroE optical
encoders** on key, wippen and hammer · a **Photron Fastcam-X 1280** high-speed camera
for jack and lever · LabVIEW Express 7.0. Five superimposed forte trials agree
closely (Figs. 3.9–3.12): "This inspires confidence that the experimental setup is
capable of producing reliable, consistent results" [V, Hirschkorn 2004 §3.2.2]; the
positions measured by the encoders "were also extremely repeatable" [V, §5.1].

Usable data (figures in the thesis, to be digitized):
- **Fig. 3.6**: force profiles measured on an amateur pianist;
- **Figs. 3.7 / 3.8**: force profiles of pianist **and** motor, **forte** and
  **piano** blows;
- **Figs. 4.1–4.4**: the forte blow's force, then key, wippen and hammer rotations,
  experiment against model;
- **Figs. 4.5–4.8, 4.9–4.11**: the same for the **piano** blow (with the recalibrated
  damping);
- **Figs. 4.12–4.15**: the same for the **double blow** (the difficult case);
- **Figs. 2.5 / 2.6**: the key front felt's **loading, unloading and fitted curves**,
  up to about 35 N at 0.86 mm, directly usable to check a felt law.

Characteristics of the inputs [V, Hirschkorn 2004 §3.2.1, §4.2.1]: the forte profile
rises to 10 N, then spikes to about 32–34 N at the key bottom; the piano blow rises
gradually to about 1.5 N before its peak at the key bottom, about **seven times
smaller** than the forte and about **three times longer** to reach the bottom, with a
60 Hz interference in its signal.
Output scales: key rotation about 0.05 rad, wippen about 0.10 rad, **hammer about
0.40 rad**, over 0–0.7 s [V, Hirschkorn 2004, Figures 3.10–3.12].

### 9.2 Thorin 2013/2017: the set richest in FORCES

Equipment [V, Thorin 2017 §3.2]: the isolated **Renner** action · **Keyence LB12 +
LB72** laser sensors (resolution 50 μm, integration time 0.15 ms, sensitivities
0.398 and 0.449 V/mm) · **Endevco 2250A** accelerometer (0.4 g, 0.316 V per m·s⁻²,
Brüel & Kjær Nexus conditioner, 0.1 Hz–3 kHz) · **Kistler 9211 piezoelectric force
sensor** (1.2 g, Kistler 5015 charge amplifier, 1 Hz–3 kHz, sensitivity 10 mN,
nonlinearity and hysteresis within ±1 % of full scale, the scale usually 10 N) ·
**Simi HCC-1000 high-speed camera** (923 fps, 1024×512, four CMOS sensors) and
**KLT** tracking · **NI USB-6211** acquisition at 50 kHz.
Post-processing [V, Thorin 2017 §3.2]: the **50 Hz** and **0 Hz** components
identified on the signal preceding the blow and subtracted; velocity by **total
variation regularisation** (Chartrand 2011: 30 iterations, 200 sub-iterations, a
regularisation parameter of 5 × 10⁻⁵, ε = 10⁻⁹; the thesis prints the same values
[V, Thorin 2013 §2.2, p. 21]).

Usable data [V, Thorin 2017 §5]:
- **Fig. 7**: key-end position, *piano* blow, **force**-driven (the model's relative
  error **3 %**; the rigid-felt "Bad Model" **9 %**);
- **Figs. 8 / 9**: **reaction forces** at the key end, *piano* and *forte* blows,
  **position**-driven ← **the reference curves to aim at**;
- **Fig. 10**: simulated angles against KLT tracking for **key, hammer, wippen,
  jack, lever** over 0–0.45 s;
- **Fig. 11**: film and simulation compared frame by frame at five notable events;
- **Fig. 12**: time-step convergence (0.4 / 2 / 10 kHz).

### 9.3 Askenfelt & Jansson, Goebl: reference data on touch

- Askenfelt & Jansson, "From touch to string vibrations. II. The motion of the key
  and hammer", **JASA 90, 2383–2393 (1991)** [V, Goebl et al. 2005, references].
  **[?]** Not in the corpus; its figures are cited here through Goebl et al.
- Goebl, Bresin, Galembo, "Touch and temporal behavior of grand piano actions",
  **JASA 118(2), 1154–1165 (2005)** [V, Goebl et al. 2005, p. 1154]. Pianos: a
  **Steinway** model C, a **Yamaha** Disklavier and a **Bösendorfer** SE290 [V,
  Goebl et al. 2005, p. 1157]. Two accelerometers per note, on the key and the
  hammer, in a setup similar to Askenfelt and Jansson's [V, Goebl et al. 2005,
  p. 1156]. *Pressed* and *struck* touches.

Numerical values:
| Quantity | Value |
|---|---|
| Rest to key bottom, **forte** (about 5 m/s FHV) | **about 25 ms** [V, Goebl et al. 2005, p. 1154, citing Askenfelt & Jansson 1991] |
| Rest to key bottom, **piano** (about 1 m/s FHV) | **about 160 ms** [ibid.] |
| Hammer–string contact against key bottom, **piano** (1 m/s) | **12 ms BEFORE** [V, Goebl et al. 2005, p. 1154, citing Askenfelt & Jansson 1990] |
| Hammer–string contact against key bottom, **forte** (5 m/s) | **3 ms AFTER** [ibid.] |
| Travel time, finger–key to hammer–string | **20 ms to about 200 ms** (up to 230 ms on the Steinway) [V, Goebl et al. 2005, p. 1160] |
| Highest hammer velocities measured | Yamaha **C1 6.4, G6 7.8 m/s**; Steinway C1 6.8, G6 7.5 m/s; Bösendorfer C1 6.0, G6 6.6, C4 7.6 m/s [V, Goebl et al. 2005, p. 1160] |
| Finger–key force, *legato* | about a third of the *staccato* force, which is typically 15 N mezzo forte [V, Goebl et al. 2005, p. 1156, citing Askenfelt & Jansson 1992] |
| Correlation of key and hammer velocity | *pressed* above about 0.6; *struck* below [V, Goebl et al. 2005, p. 1160, Fig. 5] |

### 9.4 Let-off velocities: the cleanest validation target

Thorin 2017, Table 5 (thesis Table 6.1):

| Dynamic level | Measured | Simulated |
|---|---|---|
| *piano* | **1.01 m/s** | **1.00 m/s** |
| *forte* | **1.56 m/s** | **1.56 m/s** |

"*The prediction is very good*" [V, Thorin 2017 §5]; "*perfectly estimated*" [V,
Thorin 2013 §6.4.1]. **The quantified test to reproduce first.**

### 9.5 Repetition frequency (Timmermans 2021)

The highest frequency at which a key driven by a sinusoidal position still repeats
its note, on the Renner demonstrators, simulated and measured [V, Timmermans 2021
§7.1, Table 4]:

| Action | Simulated | Measured |
|---|---|---|
| Upright | 11 Hz | 13 Hz |
| Grand, full stroke | 13 Hz | 14 Hz |
| Grand, from the bottom to halfway up the stroke | 19 Hz | ≥ 20 Hz (the actuator's limit) |

A dynamic target for the repetition lever (planned), measured on a demonstrator
whose geometry is not published: a comparison of order of magnitude, not a test.

---

## 10. TOUCHWEIGHT: Stanwood's equations

From **Appendix A** of D. Stanwood, "Down Weight versus Equation Piano Key
Balancing: Shifting the Paradigm" (2014) [V, `stanwood2014.pdf`, Appendix A], with
the protocols published in the *Piano Technicians Journal*, February 2000
("Standard Protocols of the New Touchweight Metrology").

**Terms**: SW = Strike Weight · R = Strike Weight Ratio · FW = Front Weight · KR = Key
Weight Ratio · WW = Wippen Radius Weight · WBW = Wippen Balance Weight · D = Down
Weight · U = Up Weight · BW = Balance Weight · F = Friction Weight.

```
BW  = (D + U) / 2
F   = (D − U) / 2
WBW = KR × WW
R   = (BW + FW − WBW) / SW
```
→ **Balance equation**: `BW + FW = KR × WW + SW × R`
→ Patent **US 5,585,582** ("Methods for inertial balancing of piano key mechanisms",
1996): `FW = ((SW × R) + (WW × KR)) − BW` [V, Stanwood, "Component TouchWeight
Balancing – Blueprint for the Future", PTJ April 2000; `stanwood/US5585582.pdf`]

**Measurement protocols** [V, Stanwood 2014, "Results and Analysis", p. 3]: **SW**: "tipping a shank mounted
hammer onto a digital scale"; **FW**: "tipping each keystick on it's balance point
and resting the front of the key onto a digital scale". The definitions, from the
glossary of the 2000 protocols [V, Stanwood 2000, pp. 22–23]: D and U are the least
and the greatest weight, to the nearest gram, on the measuring point (13 mm back
from the front lip of the key) that makes the key drop, or that the key can lift,
"while maintaining a slow controlled motion of the hammer"; KR is "the amount of
weight at the measuring point needed to balance 1.0 grams of weight at the
capstan/heel contact point", read with 10 g at the capstan, a reading of −5.5
giving 0.55 [V, Stanwood 2000, p. 22]: the force convention of FOUNDATIONS §3.7.

**Reference values and tolerances**:
| Quantity | Value |
|---|---|
| Normal zone for R | **5.0 to 7.0**; above 7.0 "actions become too heavy (even with very light hammers)" [V, Stanwood, PTJ March 2000, pp. 24–25] |
| Mean R in Stanwood 2014's analysis of a high-end, concert-prepped grand | **6.0**, the middle of the medium zone [V, Stanwood 2014, "Results and Analysis", p. 3] |
| Perceptible change in R | **0.3**: "a significant change in the feel of the action" [V, Stanwood 2014, "Results and Analysis", p. 3] |
| Perceptible change in SW | **0.5 g**: "Pianists can easily feel a 0.5 hammer weight change" [V, Stanwood 2014, "Results and Analysis", p. 3] |
| SW variation between adjacent notes (common) | **1.0 g** [V, Stanwood 2014, "Results and Analysis", p. 3] |
| SW spread between like sets (same make, model, period) | up to **2.0 g** [V, Stanwood 2014, "Results and Analysis", p. 3] |

**Weight regulation target** (Thorin, Appendix A, a transcription of exchanges with
two professional technicians) [V, Thorin 2013, Appendix A]: "*the minimum downward
weight of the action without damper is close to 50 g and the maximum upward weight is
25 g*" → BW = 37.5 g, F = 12.5 g; "the exact weights depend on the manufacturers and
the grand piano model".

**Weights of Thorin's demonstrator** [V, Thorin 2013 §2.2, p. 21]: measured by the
standard procedure (small masses added to and removed from the end of the key),
F_down ≈ 0.70 N and F_up ≈ 0.38 N, "both exceeding by about 0.15 N the values
normally adjusted by technicians":
```
D ≈ 0.70 N / 9.81 = 71 g      U ≈ 0.38 N / 9.81 = 39 g
BW = (D + U)/2 ≈ 55 g          F = (D − U)/2 ≈ 16 g
```
(Thorin's Table 2.2 gives the same as torques at L = 0.245 m: C_w = 0.132 N·m by
Eq. 2.7 and C_d = 0.039 N·m by Eq. 2.8, against C_w = 0.138 N·m summed over the
parts by Eq. 2.4.) The excess over the normal values falls on the balance weight;
the friction weight is not raised by it. The demonstrator was "carefully adjusted
by a professional piano technician", and its key "is not weighted with key leads"
[V, Thorin 2013 §1.6, p. 15]. **Do not read its heavy touchweights as wear**, nor
Thorin's parameter set as that of a worn action: the "time-worn felts" he mentions
are the reason he checked his photographed geometry with a dial calliper [V,
Thorin 2013 §4.1, p. 48] (FOUNDATIONS §5.2 c).

---

## 11. THE METHODOLOGICAL TRAP NOT TO REPEAT (Thorin, Boutillon, Lozada 2014)

Ref.: "Modelling the dynamics of the piano action: is apparent success real?", **Acta
Acustica united with Acustica 100(6), 1162–1171 (2014)** [V, Thorin 2017, ref. 36],
reproduced as **chapter 2 of the Thorin thesis**.

**Table 2.3: sensitivity of the results to the driving mode** [V, Thorin 2013 §2.5,
Table 2.3]:

| Input → Output | F → y | ÿ and y → F | y → F | ÿ → F |
|---|---|---|---|---|
| sensitivity to the **MODEL** | **0** | **+** | **+** | **+** |
| sensitivity to the **initial conditions** | + | 0 | 0 | + |
| sensitivity to **NOISE** | 0 | 0 | + | 0 |

("+" = sensitive, "0" = little or not sensitive.)

**The argument** [V, Thorin 2013 §2.5]: the dynamics are dominated by inertia up to
let-off, so the acceleration is smooth, and the **double time integration filters**
everything else out: "*any model, provided that it is inertia-dominated, is likely
to appear as very good when checking its validity by means of comparisons of
motion-results obtained in force-driven simulations and tests, before
escapement*". On the complete model, the **"Bad Model"**, felts made rigid (on a
real action "the hammer would probably not even reach the string"), scores **9 %**
error on position under force drive against **3 %** for the model, while under
position drive its force is "off the measured force" [V, Thorin 2017 §5; Thorin 2013
§6.4.2] (FOUNDATIONS §5.1).

**Consequences for the validation protocol:**
1. **Drive in POSITION, compare the FORCES.** Never the reverse as the primary
   validation.
2. Comparing accelerations is an alternative "provided that a level of force
   comparable to at least a mezzoforte musical level is used" [V, Thorin 2017 §1].
3. The hammer-displacement comparisons published by Oboe, Van den Berghe, Gillespie,
   Hirschkorn and Links **do not validate** their models in the strong sense [V,
   Thorin 2013 §2.5]; Hirschkorn's parameter set remains the most complete measured
   set of one action (§7.1).
4. Corollary for the interface: **the curve to show the technician is the reaction
   force on the finger**, not the trajectory. That is where let-off, key bottom and
   checking show.

---

## 12. SENSITIVITY TO THE REGULATION: what Thorin's analysis measures

Thorin, thesis §6.5. Cost function: `J = Σ|F_mes(t_i) − F_sim(t_i)| / Σ|F_mes(t_i)|`
(Eq. 6.4; the 1-norm, "*because we did not want to penalise large discrepancies too
much*"), t_b = 0 s, t_e = 0.22 s (*piano*) and 0.13 s (*forte*), measurements at
50 kHz downsampled to the simulation's 2 kHz [V, Thorin 2013 §6.5.1]. Each category I
parameter is varied **in steps of 0.1 mm up to ±2 mm** around its measured value:
82 simulations per parameter, 41 in piano and 41 in forte [V, Thorin 2013 §6.5.2,
p. 114].

**The eight category I parameters, those a technician regulates** [V, Thorin 2013
§6.5.2, Figure 4.6]:
| Symbol | Regulation |
|---|---|
| S_K+y | height of the front rail punching (**key dip**) |
| K_Wx, K_Wy | coordinates of the top of the **capstan** |
| S_Jy | vertical position of the **let-off button** |
| J_Wx | horizontal position of the **jack regulating screw** |
| S_Ly | vertical position of the **drop screw** |
| K_Hx | horizontal position of the **back check** |
| L_Wy | **repetition lever regulating button** |

**Category I** [V, Thorin 2013 §6.5.2, pp. 114–117]: J barely changes with S_Ly and
K_Wx; for K_Wy, S_Jy, J_Wx and K_Hx it is partly flat with large irregularities
where the action blocks, "a 0.5 mm change of the value of one of these parameters is
enough to block the action", the measured values always on the flat part; it is
(locally) convex in S_K+y and L_Wy, the measured S_K+y at the minimum and the
measured L_Wy 0.5 mm above the optimum.

**Categories II and III** (Tables 6.2–6.3): the "**typical blocking length**" is the
change that JAMS the action; **ΔJ/J** is, in Thorin's definition, "the relative
difference between the value of J for the measured value p0 and the minimum value of
J", the larger of piano and forte [V, Thorin 2013 §6.5.3, p. 117]:

| Parameter | Range tested | Shape of J | Typical blocking length | ΔJ/J |
|---|---|---|---|---|
| J_Oy (jack axis, vertical) | ±1 mm | irregular/flat | **0.8 mm** | 24 % |
| J_Ox (jack axis, horizontal) | ±1 mm | irregular/flat | **0.2 mm** | 37 % |
| J_Hy (jack's contact under the roller, vertical) | ±1 mm | irregular/linear/irregular | **0.4 mm** | 42 % |
| J_Hx (jack's contact under the roller, horizontal) | ±1 mm | irregular | **0.2 mm** | 18 % |
| L_Hy (lever under the knuckle) | ±1 mm | **convex** | — | 26 % |
| J_Sx / J_Sy (jack's contact with the let-off button) | ±1 mm | irregular/flat | **0.2 mm** | 2.5 / 6.7 % |
| H_Jx / H_Jy (knuckle on the hammer) | ±1 mm | flat/irregular; irregular | **0.4 mm** | 10 / 3.0 % |
| W_Ox (wippen axis) | ±1 mm | irregular/flat | 0.4 mm | 3.9 % |
| K_Ox (key axis) | ±1 mm | flat/irregular | 0.4 mm | 2.0 % |
| K_Wθ (angle of the capstan contact) | ±0.5 rad | irregular/convex | 0.3 rad | 17 % |
| L_Hθ (lever angle) | ±0.5 rad | convex | 0.0 mm | 18 % |
| J_H,HO (hammer inertia) | ±10 % | linear | — | 8.9 % |
| m_H (hammer mass) | ±10 % | flat | — | 4.7 % |
| ν_JH (jack–roller friction) | ±20 % | slope | — | 4.9 % |
| masses of the auxiliary felt bodies | −50 % to +1000 % | flat | — | **< 1.6 %** |

**Do not read ΔJ/J as a ranking of sensitivities**: it is the gap between the cost at
a parameter's measured value and its minimum, and Thorin attributes its large values,
26 to 42 % for J_Ox, J_Oy, J_Hx, J_Hy and L_Hy, to "a mis-setting which has been
highlighted previously" on his demonstrator [V, Thorin 2013 §6.5.3, p. 118]. A
validation target is a property of the physics or of a protocol, never one
instrument's regulation error (FOUNDATIONS §5.3).

**Quotation to keep** [V, Thorin 2013 §6.5.3, p. 118]: "*Table 6.2 also highlights
the high accuracy required for the regulation of the piano action: for many
parameters, a variation of 0.2 mm leads to a completely different dynamical
behaviour.*"

**What the analysis does show:**
- For many geometrical parameters, a change of less than a millimetre blocks the
  action and prevents the hammer from escaping properly [V, Thorin 2013 §6.5.6].
  The parameters whose measured values sat farthest from the minimum are
  "*interestingly all related to the escapement, which is known to be very
  delicate*" [V, Thorin 2013 §6.5.3]. That is where the best numerical accuracy
  **and** the best visualisation are needed.
- J is **convex** only along a few directions: S_K+y (key dip), L_Wy (lever button),
  S_K−y (back rail cloth), K_Oy (height of the key's axis), K_Wθ (capstan contact
  angle), and so also L_Hy [V, Thorin 2013 §6.5.6, p. 119 and its footnote].
  Everywhere else it is **flat or irregular**: "*common methods of convex
  optimisation should probably not be applied to the piano action using J*" [V,
  Thorin 2013 §6.5.6]. **No regulation by gradient descent**: a scripted regulation
  sequence or a grid search does the job (FOUNDATIONS §5.3).
- The mass of the auxiliary felt bodies has **no influence** (under 1.6 % over
  −50 % to +1000 %) → 1 mg is a robust choice [V, Thorin 2013, Table 6.3].
- "*Both S_Jy and J_Wx are critical for the escapement of the hammer, and therefore
  for the control offered by the piano action, but do not change significantly the
  reaction force of the key*" [V, Thorin 2013 §6.5.2, p. 117]: the interface needs
  **two families of indicators**, the force **and** the kinematics and events. **Do
  not attach** Table 6.2's 2.5 % and 6.7 % to them: those belong to J_Sx and J_Sy, the
  jack's point against the button, a category II parameter.
- Thorin's own caveat: the cost functional "does not reflect whether the escapement
  actually occurred, nor the escapement velocity", and the let-off "only appears as a
  small hump in the force diagram and has therefore a low weight in the cost
  function" [V, Thorin 2013 §6.5.6, p. 120].

---

## 13. REFERENCE REGULATION PROCEDURE (values for the default configuration)

### Thorin, Appendix A — a transcription of exchanges with two professional technicians

[V, Thorin 2013, Appendix A, pp. I–III]:

| Step | Specification |
|---|---|
| Key balance rail | when lifted, the key must come back down **very slowly** to its resting position |
| Hammer pivot | unscrewed and held vertically downwards, the hammer must stop after **three pseudo-periods** |
| General alignment | all bodies parallel and aligned |
| **Jack–hammer** | at rest, the jack aligned with the wooden rectangle of the knuckle |
| **Repetition lever** | in contact with the knuckle; its screw adjusted so that the jack is **0.1–0.3 mm** from the knuckle |
| **Capstan** | with a key dip of **10 mm** (between **9.5 and 10.5 mm**), hammer to string = **45–47 mm** |
| **Let-off button** | let-off **as close to the string as possible, typically 1.5 mm or less** |
| **Drop screw** | with the key depressed, the hammer "falls outright but not too much" |
| **Back check** | wire set so that in *forte* the hammer is caught **15 mm** from the string, contact on the **top third** of the back check |
| **Damper (spoon)** | contact with the key when the hammer is **half-way (23.5 mm)** |
| **Key leads** | minimum down weight **close to 50 g** without damper, maximum up weight **25 g**; destructive, done only if it cannot be avoided |

### Hirschkorn, §2.3 — the procedure of Steinway & Sons' technical reference guide, seven points

[V, Hirschkorn 2004 §2.3]:

| # | Specification |
|---|---|
| 1 | **Key dip**: paper washers under the felt at the key front; travel = **10.0 mm** |
| 2 | **Capstan** turned until the **hammer height at rest = 48.0 mm** |
| 3 | **Jack at rest** (rest pad screw): the right side of the jack aligned with the right edge of the knuckle |
| 4 | **Lever at rest** (screw on the felt pad): the lever at the same height as the jack under the knuckle |
| 5 | **Let-off button**: the jack lets off when the hammer height is **1.0 mm** |
| 6 | **Drop screw**: once the jack releases, hammer height **3.0 mm** resting on the lever (a drop of **2.0 mm**) |
| 7 | **Back check** (bent wire): under a medium blow, the hammer stopped at **15.0 mm** |

An important remark [V, Hirschkorn 2004 §2.3]: Hirschkorn **regulates the model**
after measuring the geometry, because "*regulation adjustments are extremely fine, and
are beyond the accuracy of any measurements. This is especially true for the location
of the felt parts, since the 'surface' of the felt is not clearly defined.*"
→ **Architectural consequence**: a "regulation mode" that adjusts the geometric
parameters to reach TARGETS (key dip, blow distance, let-off, drop, checking), rather
than taking raw coordinates, with the coordinate kept as the normative value and the
target beside it (FOUNDATIONS §3.4). That is the ergonomics a technician expects,
and it is technically necessary.
Thorin does the same [V, Thorin 2013 §6.1, p. 98]: the virtual escapement came a
little late, and regulating the virtual action by the full procedure of his
Appendix A moved the jack regulating screw horizontally by **0.4 mm**, the back check
horizontally by **−0.7 mm**, lowered the capstan by **0.2 mm**, and left an additional
gap of **1.9 mm** at the let-off button: "*Except for the let-off button, these
modifications lie within the margin of uncertainties of the lengths measurements.*"
**Do not use** "his virtual regulation lowered the let-off button by 1 mm and the
drop screw by 2.5 mm": those are differences between Lozada's measurements and
Thorin's, on two copies of the action [V, Thorin 2013 §4.1, p. 48].

---

## 14. HAMMER–STRING CONTACT (optional sound extension)

Both reference models **avoid** the string:
- Hirschkorn [V, Hirschkorn 2004 §2.2.1, §2.2.3]: the string is replaced by a rigid
  stationary body, and the damping factor of contact 12 was **adjusted iteratively**
  "*to produce approximately the correct rebound speed for the hammer*"; "*the
  results from the simulation can only be compared to experimental results with the
  hammer striking a solid body, not a string*".
- Thorin [V, Thorin 2017 §2.1]: "*a simple ad hoc Newton elastic law suffices*";
  because the hammer has **already escaped** when it meets the string, "*fine
  modelling of the hammer–string interaction is not relevant in this work*", and the
  flexibility of the hammer "*is assumed to have no effect on the touch*".

For the sound, the reference is **A. Stulov**, "Experimental and computational
studies of piano hammers", *Acta Acustica united with Acustica* **91(6), 1086–1097
(2005)** [V, Thorin 2017, ref. 34; `stulov_actaa2005.pdf`], a **hereditary**
(material with memory) model:
```
F(u(t)) = F₀ · [ u^p(t) − (ε/τ₀) ∫₀ᵗ u^p(ξ) · exp((ξ−t)/τ₀) dξ ]
```
F₀ = stiffness (N/mm^p) · p = exponent of the compliance nonlinearity · ε = hereditary
amplitude · τ₀ = relaxation time. Limiting cases: instantaneous `F = F₀u^p`; static
`F = F₀(1−ε)u^p` [V, Stulov 2005 §6, Eqs. 9–10]. Its time integration needs an exact
update of the memory variable with the input varying linearly across the step
(annex A5 §2.4; FOUNDATIONS §4.3).

**Table 1: four makes, new unvoiced hammers, m₀ = 9 g ± 0.2 g, V_h ≈ 1 m/s** [V,
Stulov 2005 §6, Table 1]:

| | Abel | Renner old type | Renner new type | Imadegawa |
|---|---|---|---|---|
| F₀ (kN/mm^p) | 15.0 | 17.0 | 14.0 | 11.3 |
| p | 4.25 | 4.00 | 3.85 | 4.30 |
| ε | 0.994 | 0.995 | 0.995 | 0.992 |
| τ₀ (μs) | 2.00 | 2.30 | 2.25 | 2.40 |

**Scaling across the 88 notes (one set of unvoiced Abel hammers)** [V, Stulov 2005 §9,
Eqs. 12–16], useful for seeding a whole keyboard:
```
m₀ = 11.074 − 0.074N + 0.0001N²  [g]        (11.0 g at A0 → 5.3 g at C8)
p  = 3.7 + 0.015N                           (3.72 → 4.98)
ε  = 0.9894 + 0.000088N                     (0.9895 → 0.9972)
τ₀ = 2.72 − 0.02N + 0.00009N²  [μs]         (2.70 → 1.65)
F₀ = 15500 · exp(0.059N)  [N/mm^p]          (16,440 → 2,787,300)
```
→ The **stiffness F₀ grows by a factor of 170 from bass to treble**, "the most
influential parameter for the force-compression characteristic" [V, Stulov 2005 §9].
Identification uncertainties: about ±1 % on F₀, ±0.02 on p, ±0.002 on ε, ±0.1 μs on
τ₀ [V, Stulov 2005 §4].
Other findings: the differences between new hammers from different makers "are
not very significant" [V, Stulov 2005 §6]; Stulov's rig covers 0.3–1.5 m/s where real
playing reaches **5 m/s** in grand pianos, the hammer parameters being "essentially
independent of the hammer velocity" [V, Stulov 2005 §2]; the dummy string's diameter
matters little, so "*testing of hammers against a flat surface would give a fair
approximation of the real conditions in a piano*" [V, Stulov 2005 §5].

---

## 15. QUANTIFIED VALIDATION PROTOCOL

As an automated test suite, in order of increasing cost. The levels are FOUNDATIONS
§5.3's; each test says whether it is a consistency check or a validation test
(FOUNDATIONS §5.1).

**Static, no integration (to be done first)**
| Test | Target | Kind | Source |
|---|---|---|---|
| V0.1 Small-angle ratios at rest, on a model of Thorin's action | α₂₁ ≈ 2.15 · α₃₁ ≈ 10.1, his simplified model's (§7.2) | consistency (transcription) | Thorin 2013, Table 2.1 [V] |
| V0.2 Equivalent inertia referred to the key | J = 0.0221 kg·m², on the simplified model | consistency | Thorin 2013, Table 2.2 [V] |
| V0.3 Down weight and up weight of Thorin's demonstrator, quasi-static | F_down ≈ 0.70 N and F_up ≈ 0.38 N at the key end (about 71 and 39 g); C_w = 0.132 N·m, C_d = 0.039 N·m | validation, of the parameter set's lineage too (§4.2, §7.2) | Thorin 2013 §2.2, Table 2.2 [V] |
| V0.4 Stanwood consistency | BW = (D+U)/2, F = (D−U)/2, R = (BW+FW−WBW)/SW, **all** from the same model | consistency | Stanwood 2014, Appendix A [V] |

The last step of Thorin's procedure leads the key, only where it cannot be
avoided, to a down weight close to 50 g and an up weight of 25 g without damper
(§13). No test leads a key: the bench models no lead, a key being weighed whole,
its leads in it (FOUNDATIONS §2.1).

**Dynamic, one isolated note**
| Test | Target | Kind | Source |
|---|---|---|---|
| V1.1 **Let-off velocity** | *piano* **1.01 m/s** · *forte* **1.56 m/s**, within the measured scatter | validation | Thorin 2017, Table 5 [V] |
| V1.2 Order and timing of the five non-smooth events | (a) jack meets the let-off button → (b) key meets the front rail punching → (c) hammer meets the string → (d) hammer caught → (e) jack returns to rest | validation | Thorin 2017 §5, Figs. 10–11 [V] |
| V1.3 Touch timing | key bottom at **25 ms** (forte, 5 m/s) and **160 ms** (piano, 1 m/s); string contact **12 ms before** the bottom at 1 m/s, **3 ms after** at 5 m/s | validation, on another action | Goebl et al. 2005, p. 1154, citing Askenfelt & Jansson [V] |
| V1.4 Kinematic amplitudes, force-driven | hammer about 0.40 rad · key about 0.05 rad · wippen about 0.10 rad | validation, weak: positions under force drive (§11) | Hirschkorn 2004, Figs. 4.2–4.4 [V] |
| V1.5 **Reaction force, POSITION-driven** | J = Σ abs(F_mes − F_sim) / Σ abs(F_mes) (1-norm, at 2 kHz); Thorin's order of magnitude (η = 21 % piano, 13 % forte up to the key's release), and above all **the irregularities reproduced** (let-off, key bottom, checking) | validation | Thorin 2017, Figs. 8–9; Thorin 2013 §6.4.1, Eqs. 6.3–6.4 [V] |
| V1.6 Anti-trap "Bad Model" test | with the felts made rigid, the position under force drive stays good (about 9 %) **but** the force under position drive degrades markedly; if it does not, the harness is wrong | validation: its guard, which a wrong model must fail | Thorin 2017 §5 [V] |

**Convergence and numerical robustness**
| Test | Target | Kind | Source |
|---|---|---|---|
| V2.1 Time-step convergence | the key force and the observables stop changing by more than their resolution as the step falls (FOUNDATIONS §4.1); Thorin's scheme: 2 kHz ≈ 10 kHz, 0.4 kHz significantly off | consistency | Thorin 2017, Fig. 12; Thorin 2013 §6.4.1 [V] |
| V2.2 Insensitivity to the auxiliary felt masses, if that scheme is used | **< 1.6 %** of J over −50 % to +1000 % | consistency | Thorin 2013, Table 6.3 [V] |

**Sensitivity to the regulation**
| Test | Target | Kind | Source |
|---|---|---|---|
| V3.1 **Jamming of the action** | on a model of Thorin's demonstrator, the action jams within his typical blocking lengths (0.2 to 0.8 mm on the escapement parameters); on any action, the escapement jams for changes well under a millimetre | consistency on his action; a property of the escapement | Thorin 2013, Table 6.2, §6.5.6 [V] |
| V3.2 Shape of the cost function | convex along S_K+y, L_Wy, S_K−y, K_Oy, K_Wθ (and L_Hy); flat or irregular elsewhere | consistency, on his action | Thorin 2013 §6.5.6 [V] |
| V3.3 Trade perceptibility | ΔR = 0.3 and ΔSW = 0.5 g produce a **measurable** change in the outputs | consistency | Stanwood 2014, "Results and Analysis", p. 3 [V] |
| V3.4 Convergence of the regulation procedure | the scripted procedure reaches every target from a state out of regulation | consistency | Thorin 2013, Appendix A; Hirschkorn 2004 §2.3 [V] |

**Do not use** "the hierarchy of the sensitivities: ΔJ/J of J_Hy 42 % > J_Ox 37 % >
L_Hy 26 % > J_Oy 24 % > J_Hx 18 %" as a test: ΔJ/J measures how far the demonstrator's
measured setting sat from the cost minimum, a mis-setting by Thorin's account, not a
sensitivity (§12; FOUNDATIONS §5.3). Nor require a model built on Hirschkorn's
geometry to jam at the blocking lengths of Thorin's demonstrator: they are that
instrument's. **Nor use** "recalibrating D by four to five times must change the
kinematics negligibly" as a robustness test: it compares positions under force drive
(§3.3, §11).

**Repetition and low force (where the literature fails)**
| Test | Target | Kind | Source |
|---|---|---|---|
| V4.1 Double blow | reproduce Hirschkorn's Figs. 4.13–4.15, **knowing that his model fails there**, by his own diagnosis through friction too low or not modelled; doing better is a contribution | validation | Hirschkorn 2004 §4.3 [V] |
| V4.2 Friction budget | the friction weight of an assembled model, over all five pivots and the sliding contacts, against a measured F | validation | FOUNDATIONS §6.5 |
| V4.3 Static threshold test | the greatest mass that can be placed without starting the descent, and the least that prevents the return: **impossible to simulate with regularized friction** in a static solve, the test that justifies set-valued friction | consistency, a protocol run on the model | Thorin 2017 §2.5 [V] |

**Do not use** "the pivot frictions must account for only about 60 % of the measured
friction, the rest coming from the sliding contacts and the mortises" (§4.3).

---

## 16. CONCLUSIONS

### Two reference datasets, two roles

- **Hirschkorn's Boston GP-178, note 52**: geometry, masses, thirteen contact laws
  and five axis frictions measured on one action [V, Hirschkorn 2004, Appendix A].
  With every frame at its own pivot, it closes at rest [E, 10]. His experiments drive
  the key with a force and measure positions (forte, piano, double blow). It is the
  dynamics prototype's action (FOUNDATIONS §4, §5.2 b).
- **Thorin's Renner demonstrator**: the only published key force under position
  drive, and escapement velocities measured and simulated [V, Thorin 2017, Figs.
  8–9, Table 5]; a non-smooth model with set-valued friction. Its parameter set has
  lineage traps: frictions mostly assumed equal to the hammer's, two printed a
  hundred times too large, the jack's and the lever's inertial values placeholders,
  the wippen's inertia impossible (§4.2, §7.2; FOUNDATIONS §5.2 c, §6.3).
- The two contact laws come from different actions, benches and law forms, and enter
  the data as **two labelled material sets, never merged** (FOUNDATIONS §4.3). **Do
  not use** "Thorin's parameter set, supplemented by Hirschkorn's Table A.8 for the
  contact laws": it merges two material sets.

What Thorin's work brings:
1. **Forces compared under position drive**, and the demonstration (2014) that
   positions compared under force drive cannot discriminate an inertia-dominated
   model (§11).
2. **Set-valued dry friction, with stick and slip**, the condition for the static
   threshold tests to be simulable at all (§4.1, §8.2).
3. A scheme **converged at 2 kHz** on his observables; real time is his expectation,
   not his result (§8.1).
4. A published parameter set that **includes the damper**, to be transcribed with each
   value's lineage: measured, assumed, placeholder (§4.2, §7.2).
5. A sensitivity analysis of the cost J against each regulation parameter, with
   blocking lengths for his demonstrator, and the finding that J is flat along most
   directions (§12).

**Do not use** "Thorin's parameter set is complete, public and internally consistent
(the geometry reproduces his α values to 2 %)", "it cross-checks to about 10 % with
Hirschkorn's completely independent set" or "its action ratio, 5.68, falls within
Stanwood's normal zone, so the literature and the trade's metrology triangulate"
(§4.2, §7.2–§7.4).

### Extensions, ranked by benefit against cost
1. **The key's second degree of freedom, vertical on the balance rail** — the gap
   **both** groups report (§2.1), planned (FOUNDATIONS §2.3). The friction of the
   key bushings (balance and front mortises) has no measured value in the corpus
   (§4.3).
2. **Pedals and the coupling of the 88 notes** — not an extension but **original
   development**: nothing in the corpus. Geometrically simple (three prescribed
   scalars), to be specified with the technician (§6; FOUNDATIONS §7.6).
3. **Rate-independent felt hysteresis** (Masoudi and McPhee's micromechanics, a
   Preisach model, or loading and unloading curves with a memory of the turning
   points) — the target of the felt-law hypothesis (FOUNDATIONS §4.3), tested on
   Hirschkorn's piano and forte blows; its identification needs loading and
   unloading curves contact by contact (§3.4). Stulov's hereditary law is for the
   string (§14).
4. **Flexible hammer shank** (Izadbakhsh et al. 2008; Chabassier & Duruflé 2014, a
   Timoshenko beam; both as cited by Thorin) — Thorin estimates the hammer's first
   mode at **39 Hz**, so (f₁τ)⁻¹ = **5.1** with τ = 5 ms, a quarter of the fastest
   keystroke, where a value above 1 means the rigid-body approximation alters the
   dynamics: "The ratio is correct for the key but not for the hammer" [V, Thorin
   2017 §2.1, Appendix]. Before let-off, the phase that matters for touch, a
   flexible shank sits in series with the drive; the test is the prototype with and
   without a degree of freedom between head and shank, compared on key force under
   position drive (FOUNDATIONS §4.5). Thorin lists "inserting a spring between the
   hammer's head and the knuckle" among the model's next improvements [V, Thorin
   2017 §6]. Measured cost elsewhere: **1.9 times the CPU time** (Masoudi & Birkett,
   1720 s against 910 s) [V, Thorin 2017, Table 1]. Hirschkorn attributes the
   oscillation of the key after checking to the shank's flexibility [V, Hirschkorn
   2004 §4.1].
5. **String** (Vyasarayani 2009, as cited by Thorin) — out of scope for the action;
   a restitution law or a calibrated contact replaces it. Stulov **only** if sound is
   synthesized, which is an open question (FOUNDATIONS §9).
6. **Flexible key** — **set aside**: f₁ ≈ 355 Hz, (f₁τ)⁻¹ = 0.56, and the felts
   contribute **6 times** more compliance than the wood (w_f = 9.5E−5 against w_e =
   1.5E−5 m/N); for the hammer the factor is **4** (4.8E−3 against 1.2E−3 m/N) [V,
   Thorin 2017 §2.1, Appendix].

### Three architectural decisions the literature imposes
1. **The felt law is a pluggable strategy.** Hirschkorn's (a cubic × (1 + Dẋ)) and
   Thorin/Brenon's (kδ^r + bδ²δ̇) are **not interchangeable** without re-identification,
   and a rate-independent law is a third; all three coexist behind the same interface,
   as labelled material sets (planned, FOUNDATIONS §4.3).
2. **Regulation is expressed as TARGETS; the coordinate is normative.** Both authors
   regulate their model after measuring it, because the surface of a felt cannot be
   located more finely than the measurement allows (§13). The domain exposes "key dip
   = 10 mm", "let-off = 1.5 mm", "drop = 2 mm", "checking = 15 mm" and solves for the
   coordinates, which are what a saved state keeps, the target beside them. Built for
   key height and key dip, solved to punching stacks; planned for the other targets
   (FOUNDATIONS §3.4).
3. **Never a constant leverage ratio.** The published α values are small-angle
   linearisations: held constant over the driven stroke, α₃₁ overshoots the hammer's
   travel to let-off by about 14 % [E, 06]. **Do not use** "the discrepancy is blatant,
   57.6 mm predicted against 45–47 mm regulated": it uses the whole dip, aftertouch
   included (§7.4). The instantaneous ratio is recomputed from the contact geometry at
   every step (FOUNDATIONS §6.2), and showing it is the heart of what the tool does.
   Built for the key, whose weight ratio is computed from its arms and has no setter
   (FOUNDATIONS §3.7); planned for the action.
