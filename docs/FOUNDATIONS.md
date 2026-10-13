# Foundations: a grand piano action simulator

This document states what the simulator is for, what is built, what is planned,
and why. Every component and mechanism is labelled **built** or **planned**
where it is described. A decision that is not settled is labelled a
**hypothesis**, with the test that will settle it.

Numbers carry a reliability marker: **[V]** verified against a primary source,
named with its locator; **[E]** computed by the named script in
`tools/derivations/`, or **[E, prototype]** by the dynamics prototype in
`tools/dynamics-prototype/` (its `RESULTS.md` prints every figure);
**[S]** assumed, pending measurement. A number describing
the code, a default or a slider's range, carries none: the tree is its source;
nor does one whose arithmetic is written beside it. A sentence beginning **"Do
not"** marks a number or an argument that is wrong and easy to derive again,
where a reader would otherwise derive it. The
research annexes, `docs/research/A1` to `A5`, hold the study this document draws
on; where one disagrees with this document, this document holds.

## 1. What the simulator is for

### 1.1 An engine for piano applications

The project is an **engine for building applications around the grand piano
action**, educational and playful alike, rather than one application. It is
built for a piano technician, who is also its first user, and it provides five
things an application composes:

- **the trade's physics**: the action's parts, contacts, friction and motion,
  measuring itself by the trade's protocols (§1.3);
- **a configurator**: every part and every regulation setting adjustable, the
  way a technician regulates and a designer draws (§3.4, §7.3);
- **a state**: a regulated action that can be saved, reopened, compared and
  replayed, its coordinates normative (§3.4, §7.4);
- **diagnostics**: what is out of regulation and what cannot work, named in the
  trade's terms and quantities (§3.3);
- **a realistic rendering in 3D**, beside the technical section that validates
  the geometry (§3.11).

Built: the physics, configurator and diagnostics of the key on its frame, and
the technical section and solid view of the bench, the engine's first
application. Planned: the rest, and an interface shaped so that a new
application composes these five without changing the engine (§7).

### 1.2 Uses

The applications in view, others to follow:

1. **Regulate, measure and train on a single key**, set up as a technician sets
   one up and read as he reads it. Regulating and measuring built for the key on
   its frame (§2); training planned (§8).
2. **Understand the action's static and dynamic equilibria**: why a key rests
   where it does, why a weight between up weight and down weight moves nothing,
   what the jack does at let-off. Key statics built; dynamics planned.
3. **Discover the action**: parts in section and in solid, contacts visible,
   motion slowed until escapement can be watched. Section and solid of the key
   and frame, with the key's contacts marked on them, built; motion planned.
4. **Design an action.** Parts and frame are edited freely. What cannot work is
   avoided or limited, refused or clamped with its reason; what can be handled
   is handled, so a frame that does not fit its key is drawn as it is, its
   faults named in millimetres. Faults for the key and frame built; a
   validation layer naming every impossible configuration planned (§7.3).
5. **Play a score on 88 notes**, dampers and three pedals included. Planned.
6. **A game whose task is to regulate a piano.** Planned.

Out of scope: upright actions, a soundboard model, tuning, voicing. Whether
sound is in scope, and how, is open (§9).

### 1.3 The acceptance criterion

> **The simulator must measure its own regulation using the trade's own
> protocols, and come back with the expected values**: key height, key dip,
> blow distance, let-off, drop, aftertouch, checking, damper lift point, down
> weight, up weight, friction, action ratio.

If that holds, the simulator is an instrument; if not, an animation. Hence a
reported value is the protocol run on the model (a let-off is the
hammer-to-string distance at the instant the jack leaves the knuckle, not a
parameter read back), and where a displayed value is a formula instead, this
document says so (§2.2). A protocol run on the model is a consistency check, not
a validation, which needs measurements taken independently of the model (§5). A
simplification is judged against the protocol's resolution, the gram for
touchweights and a punching paper for key height and dip (the thinnest is
0.08 mm [V, `data/params/punching-papers.json`]); one that can move a reading by
that much is named where it is made. And the user sees the trade's quantities:
grams, millimetres, the trade's names, fractional inches planned as a second
unit system.

### 1.4 What the model is

**Planar and single-note**: the sagittal section of one note. Adjustments out of
that plane or across notes (squaring and spacing keys, bushing play, glides,
travelling and burning shanks, hammer spacing, the una corda shift) are planned
as parameters that change what is observed, not as geometry. **String height
and keybed crown** form one vertical gap, whose sum is all that blow distance,
let-off and aftertouch see; both are planned. **Key frame bedding is out of
scope**: a deflecting frame couples neighbouring notes, breaking the
independence the 88-note runtime rests on (§7.6), and no published frame
compliance exists. Its main symptom, dip absorbed by something soft, stays
reachable through a softer front punching law; the bench is to state *perfectly
bedded frame* beside the settings it teaches (planned).

## 2. What is built: stage 1a, the key on its frame

### 2.1 Parts and statics

**The key**, `KeyDesign`, is a natural: a keystick of one wood thickness with
what is glued to it or cut from it (the covering; the key front over the end
grain, the covering's lip overhanging it; the capsule where the balance mortise
is cut; the back check block; the thinned tail and its damper felt; the chamfer
that lets the tail bed flat on its cloth) [V, `data/params/key-section.json`].
The capstan is a turned steel part, integrated as a solid of revolution; leads
are point masses added, moved and removed on the model. Planned: leads leave
the model, a key being measured whole, its leads included (§2.3). Each piece
is a part of its own, with its own material, on the key's body: the covering
and the key front are of their own materials, the capsule and the back check
block of one wood that is not the keystick's, and the front mortise is a cut,
a void in the keystick open at its underside.

**The frame**, `FrameDesign`, is three different rails: the balance rail with
its crown and round pin, the front rail with its oblong pin, the back rail with
its cloth. A punching is a felt washer plus papers from a drawer of eight
thicknesses, 0.08 to 1.25 mm, each known by its colour [V,
`data/params/key-frame-sections.json`, `punching-papers.json`].

**Statics at one degree of freedom.** The key turns about a fixed point at its
balance pin. The front punching and the back rail cloth are unilateral stops,
solved with the circle-against-line contact that ten of Hirschkorn's thirteen
contacts use (§6.6); the front pin meeting the top of its mortise is a third
stop. Friction at the balance is set-valued: any torque within ±b holds the key.
With T the net torque of gravity and of the weight on the measuring point, the
key stays on the front punching if and only if T ≥ −b, on the cloth if and only
if T ≤ b, and between them if and only if |T| ≤ b; a floating key is located by
a fixed 24-step bisection. **Regulation is solved from targets**: key height and
key dip go in; the punching thicknesses come out of a small-angle closed form
and are laid as papers from the drawer, what it cannot reach showing as the
miss.

**The frame is placed under the key once, then left alone.** Building a frame
puts the front rail under the key's front mortise and the back rail a set
distance in from the tail; the positions are then the frame's own coordinates.
Lengthen the key and the rails stay put, the mismatch reported as faults in
millimetres past the limit (pin outside its mortise, punching off the flat of
the underside, cloth outside the chamfer); placing the frame again is a
deliberate act.

**The general mechanism, its first part.** Bodies in a tree, the ground first,
each hung from its parent by a pin or a slide with one coordinate, an angle or a
length; poses by one forward pass, whose only transcendentals are one sine and
one cosine per pin, through `libm` (§4.4); the Jacobian column of any material
point for any coordinate, by the lever-arm identity of §6.2 extended to every
ancestor joint; gravity at each body's centre of mass and forces fixed in the
world, with their potential and generalized forces; and coordinates prescribed
for one procedure, with the reaction that holds each. The mechanism never
computes a body's mass; it is handed one. The composite key's is derived from
its volume model by one adapter in `pa-app`, the composite key's derivation
(§2.3). Its first client is the jig (§2.2). Planned: coordinate limits, axis
friction and the static solve that finds the coordinates a procedure leaves
free (§8); until then the key on its frame keeps the one-degree-of-freedom
statics above.

**Contacts, read off the drawn parts.** Each of the key's contacts with its
frame is a pair of ranges of drawn parts and nothing else: the front punching's
top against the underside's flat; the back rail cloth's face, its rounded edges
and the sides they lead into, against the whole underside, chamfer included; the
front pin's top against the top of its mortise, and against the wood beside the
slot, which only a pin standing outside its mortise but under the key reaches.
Where a surface runs across parts, as the underside runs from the keystick onto
the key front, the joint is no corner. A mechanism carrying the parts reads
every pair at any pose into rows: one for each pair of features that meet (a
disc, which is an arc or a convex corner, against a straight face; two discs; a
face's centre against a face), each a signed gap with its normal and its contact
point, and over every coordinate the gap's gradient by the lever-arm identity of
§6.2 and the sliding of the two surfaces past each other. A row's gap keeps its
sign however deep it is read, though a corner driven past the centre of an edge
it overlaps can have no row at all (§6.6). The front punching is read at the
centre of its drawn top, a rigid reading of a felt washer that spreads the load
of a key landing level over its whole face. How the key bears on its cloth (on
the cloth's edge; flat on it, its two edges within 2.5 µm of closing, about a
thirtieth of the thinnest paper; or on the key's own corner), where, and the
slope of the key's face over the cloth's are read off those rows, and so are the
two gaps on the sheet and the contacts the view marks. The statics above still
place the key, and the rows are read at the attitude they give.

### 2.2 The protocols, as they are

- **Front weight and key weight ratio** are Stanwood's jig run on the model
  [V, Stanwood 2000; `data/params/touchweight-protocol.json`]. The key hangs
  from the jig by a pin at its balance pin's axis, a mechanism of §2.1 whose
  pin turns without friction, an assumption; its angle is held level, the
  covering's top horizontal where his bubble level sits. Holding it there takes
  a reaction on that angle, and the scale, standing under the measuring point,
  reads that reaction over g times the measuring point's arm: signed, so a
  back-heavy key reads negative, as his protocol for a negative front weight
  reads it. The ratio is the reading tared with the key on the jig, then with
  10 g centred on the capstan's contact point, over the load with its sign
  turned. The closed forms, static moment over arm and capstan arm over arm,
  are test oracles the jig matches to round-off. A jig pivots the key at the
  pin, so it cannot see where the key bears on its balance punching; only a
  reading on the frame can (§5.2 a).
- **Key friction weight** is a procedure run on the model, Stanwood's: zero the
  front weight with a temporary weight, put the key back on its frame, find the
  least weight on the measuring point that makes it leave its back rail cloth (a
  fixed 32-step bisection over the settle solve). With a constant friction bound
  it returns the bound over the arm almost by construction, checking datum,
  attitude and signs, not friction physics. It cannot measure a back-heavy key,
  the temporary weight going behind the pin; and it zeroes the front weight with
  the key nose-up on its frame, where Stanwood zeroes it on the levelled jig,
  about 0.3 g apart on the default key, under the gram.
- **The key dip "measured by pressing"** is the analytic dip computed again: the
  settle solve returns the stop angles the travel uses, and the two agree bit
  for bit. It is not an independent measurement.
- **D, U, BW and F**, on the sheet and in `pa-cli weigh`, come from a closed
  form taken with the key level, not from the weighing run on the model; F is
  the input friction torque over the measuring arm: the key friction weight as
  set, echoed back, until the measuring point moves. On the default bare key
  the sheet's D, −0.48 g, is about a quarter of a gram below the −0.22 g the
  bench's own settle solve gives from the key's real, nose-up rest attitude.
- **Datums.** Key height and dip are read at the key front, the face of the
  plate over the end grain, 258 mm in front of the balance pin on the default
  key. Every touchweight is read at the measuring point, 13 mm by default
  (§3.1) back from the covering's lip, the front edge of the keytop, which
  stands 2 mm proud of the key front: 247 mm in front of the balance pin. The
  distance is one setting, `TouchweightProtocol` on the key assembly, set on
  the bench from 5 to 30 mm, and every touchweight reads it there: front weight
  and the key weight ratio on the jig, D, U, BW and F, key friction weight, and
  the weight on the measuring point. Moving it changes each by the lever, the
  same static moment and the same friction torque read on another arm, and
  leaves key height and dip where they are.

**Planned:** D, U, BW and F become procedures run on the model too (D the
least weight that starts the key down from rest, U the greatest that lets it
rise from the bottom), their closed forms joining the jig's as test oracles.

### 2.3 Named simplifications

- **The key's mass model cannot predict front weight.** The keystick is a
  uniform 22 mm prism over its whole length, of an assumed 430 kg/m³ [S], when a
  grand keystick is far narrower behind its head; the covering counts as wood;
  the hollow behind the capstan is absent: several grams of doubt on a front
  weight read to the gram, and a measured stick mass rescales the total without
  moving the centre of mass. Planned: per-key mass properties, measured by the
  trade's protocol and never derived from a wood volume, each key being cut to
  its own plan shape: its mass; its front weight, which gives its static moment
  about the balance pin; and, for the dynamics, its inertia. The three are
  independent: a key carrying 16 g of front weight in leads alone has 8,000,
  5,700 or 2,920 g·cm² about its balance point depending on where the leads
  sit [V, Nakamura, *Practical application of moment of inertia*, Part 2,
  Fig. 9]. The trade computes inertia from a traced template and wood densities
  [V, Nakamura, Part 1] or rates it from static weights [V, Stanwood, SNAP,
  2023], and measures it nowhere; Hirschkorn swung each part as a pendulum on a
  string, I = m·g·l·(T/2π)² [V, Hirschkorn 2004 §A.1], which a string between
  support and part biases unless the part hangs from a knife edge. Inertia is
  therefore neither measured nor shown: it is an internal parameter of the
  dynamics, each part's taken from Hirschkorn's published action and the key's
  estimated from its outline matched to its mass and front weight, marked
  estimated (planned). The volume model then draws a key and never weighs it,
  and leads leave the model: a key arrives leaded, with lead masses nobody
  knows, and is measured whole.
- **The default key is a composite**: the owner's measured key section, a
  capstan 129.5 mm behind the balance pin that no source gives, and a key front
  258 mm out, which puts the measuring point at 247 mm. Planned, with the
  quasi-static core: the default key becomes Hirschkorn's (§5.2 b), whose mass,
  centre of gravity, inertia and contact points were measured and published on
  one action, and the owner's measured section a second profile.
- **The balance bearing is a rigid pivot at the pin.** The key bears on a felt
  washer whose pressure centre moves with attitude and load; moving the bearing
  behind the pin by cutting the punchings drops the action ratio by three to
  four tenths [V, Stanwood 2014, "Cutting the Balance Rail Punching to Effect
  Ratio"]. The offset is a parameter, set to zero. Planned: a compliant rocking
  contact.
- **The linings are rigid**: drawn at their thickness, acting as rigid stops.
  Planned with the felt laws of the quasi-static core.
- **The front punching bears at its centre.** Its contact reads the centre of
  the felt's drawn top, a rigid stand-in for a washer that spreads the load of
  a key landing level over its whole face. A key landing out of level would
  overlap the drawn felt's edge by what a real felt compresses, unreported
  (planned).
- **The bilateral pivot is assumed, not checked.** The key cannot lift off its
  balance punching; the guard meant to check it is called by nothing in the
  bench and ignores the stops' reactions, and at the −60 g end of the weight
  slider the key's own statics say it lifts off. A frame whose stops fight is
  reported, not weighed, the key drawn riding on its two stops as if its
  underside were straight: under the chamfer it is drawn about 0.8 mm clear of
  its cloth, and the contacts the view marks show it open by that much.
  Planned: the key's vertical degree of freedom.
- **The statics read stops of their own.** The key's attitude comes from the
  one-coordinate statics, whose front stop is the higher of the punching and
  the front pin's tip, read at the pin's middle, and which do not see a pin
  standing outside its mortise; the contacts, read at that attitude, see the
  drawn parts. A pin bottoming in its mortise therefore stops the key with one
  of its corners into the mortise's top by half the pin's length times the
  key's tilt, about a micron with the pin a millimetre prouder than the
  default frame's, and a key shortened under its frame is drawn with its front
  pin up in its wood; both show as overlaps among the contacts the view marks.
  Planned: the static solve on the contacts, which stops the key where its
  drawn parts meet.
- **The inverse solve ignores the mortise stop**, reporting as solved a dip the
  pin prevents; elsewhere its small-angle term misses by about 0.02 mm.
- **No validation layer**: a front edge close to the capsule makes the outline
  cross itself, and the front pin and mortise at their maximum put the pin
  through the top of the key, both weighed with no fault raised.
- **Drawing**: one row of pins; flat-topped pins; a flat balance crown only.
  The balance mortise is not drawn, none of its dimensions being measured, so
  the balance pin stands hidden in the key's wood; the front mortise's place
  and length along the key are assumed [S, `data/params/key-section.json`],
  its width across the keyboard not modelled.

### 2.4 The bench

**Built.** A browser page in React and react-three-fiber draws the key on its
frame in section and in solid from parts computed in Rust, each in its own
body's frame and placed by the pose the statics give that body. The section is
the sagittal plane itself: a washer shows the hole its pin stands in, and the
front pin stands in its mortise, a void open at the key's underside. Both views
mark the key's contacts with its frame where the contact rows find them: where
the key bears, a gap of under a millimetre still open, and drawn parts that
overlap. Colours
follow each part's material: the covering and the key front white, the capsule
and the back check block in a wood other than the keystick's. Its panels: the
weighing sheet, with the measuring point's distance back from the front lip of
the key; regulation (punching papers, key friction weight); punchings and
cloth (felts, front rail pin and mortise, back rail cloth); the key; the frame
(rails, placing the frame); touch (a weight on the measuring point, which
stop holds the key, and the gaps at the front punching and the back rail
cloth); the leading workshop, which goes with the leads
(§2.3). Faults are listed in millimetres,
every number on the sheet is computed in Rust, and the page is in English.
`pa-cli weigh` prints the sheet's weights, ratio, key height, dip and mass.

## 3. Decisions that stand

Each decision is in force; its reason is given, and the alternative it rules
out where that matters.

### 3.1 The trade's protocols are the acceptance test

The simulator reports what a technician's procedure reads when run on the model
(§1.3), so the person who uses the tool can judge it; ruled out are probes chosen
for convenience, a parameter read back or an internal angle. The protocols are
Stanwood's *Standard Protocols of the New Touchweight Metrology* (Piano
Technicians Journal, February 2000), which supersede those of his 1996 paper:
front weight levelled with a bubble level, the key weight ratio read with 10 g at
the capstan, a negative front weight measured by its own protocol, and Key
Friction Weight and Support Spring Balance Weight defined [V, Stanwood 2000,
pp. 21–22]. With every weight centred on the measuring point, 13 mm or 1/2 inch
back from "the front lip of the key" [V, Stanwood 2000, glossary, p. 23]:

```
BW + FW = (WW × KR) + (SW × SR)
BW = (D + U) / 2        F = (D − U) / 2
```

**The lip is the covering's**: the front edge of the keytop, which overhangs
the key front by 2 mm on the default key. The glossary does not say which lip
it means, and this one is the owner's choice; the 1996 paper's "front vertical
edge of the key" goes with the protocols the 2000 ones supersede. Measured from
the key front, the point would stand 2 mm further back and every touchweight
would be read on an arm 0.8 % shorter. **The distance is a setting**, 13 mm by
default and the glossary's 1/2 inch as its alternative
[`data/params/touchweight-protocol.json`], held in one place because every
touchweight protocol reads it, so no two can be taken at different points
(§2.2). Key height and key dip keep the key front as their datum.

### 3.2 Staged by sub-assembly, with the view at every stage

The action is built one sub-assembly at a time, engine and view together, along
the terms of Stanwood's equation: key, wippen, hammer (§8). A discrepancy is
then attributable (a whole action's down weight has dozens of candidate causes,
one key's front weight a handful); the decomposition is the trade's own, each
term a sub-assembly and a measurement; and the owner validates by looking, so
with one rigid body checkable by hand a disagreement between drawing and numbers
is a pipeline bug. Ruled out: the whole action's statics, then dynamics, then
rendering, which cannot attribute a discrepancy and withholds the owner's only
channel of validation. Geometry comes from the owner's measurements, asked for
before a part is drawn; the drawing catches what slips through and does not
replace measuring.

### 3.3 Upstream settings drift; faults are reported, not refused

Change the key dip and the aftertouch keeps its old value, out of specification;
shorten a key and its front rail pin stands outside its mortise by a named
distance. An action whose upstream setting changed is out of regulation until
someone redoes the downstream one, and a designer pushing a geometry until it
breaks needs to know where: both are what the tool teaches. Ruled out:
cascading, which hides the lesson, and refusing every state out of
specification, which forbids design. The limit is honesty: what cannot be
computed or drawn truthfully is refused or clamped, with its reason (§7.3).
Built for the frame (§2.1); planned for regulation settings, as badges.

### 3.4 Regulation as targets; the coordinate is normative

Regulation is stated in the trade's targets (key height and dip, then blow
distance, let-off, drop, checking) and the geometry (punching stacks, capstan,
buttons and screws) is solved to meet them. A saved state's normative value is
the coordinate; the target travels beside it as intent, with the value achieved
and a fingerprint of the solver, because re-solving with another solver could
silently change every number downstream. Training turns an actuator and reads
the result; the solver aims at a target. Built: key height and dip solved to
punching stacks. Planned: the other targets, actuators with their quanta (a
paper, a turn of a screw), persistence (§7.4).

### 3.5 Set-valued dry friction, at the axes and at sliding contacts

Friction at a pivot is a set, any torque within [−b, b] while the pivot is
still; never a `sign()` or a `tanh()`. The interval between down weight and up
weight *is* that friction, F = (D − U)/2, a threshold measured quasi-statically.
A regularized law has zero friction at zero speed, so a static solve with it
collapses D onto U: the threshold test "cannot be simulated with a regularized
friction law" [V, Thorin 2017 §2.5]. Dynamically, with a threshold speed of 10⁻⁶
rad/s, it keeps the threshold, creeping under a micrometre in ten seconds under
a gram of excess [E, 07], but that speed has no physical meaning and enters
every sensitivity. **Do not argue against regularization from the time step**:
its tiny step holds only for explicit integration; treated implicitly, a
velocity-only friction law sets no step limit [E, 07]. Friction is stored as a
torque with the radius it was measured at, never as grams alone, because the
trade's published pivot frictions are taken at different radii.

**The four sliding contacts too** (capstan and heel, back check, jack and
knuckle, lever and knuckle). Hirschkorn's contact friction is regularized by a
threshold speed he does not print [V, Hirschkorn 2004, Eq. 2.10]; integrated
explicitly at 10⁻⁵ m/s, it reverses sign from one step to the next wherever a
loaded contact sticks (218,419 reversals in the 485,000 steps of a forte press
at 1 µs), and its readings still move at a quarter of a microsecond
[E, prototype]. Its limit as that speed goes to zero is a set: any friction up to
the curve's peak while the contact sticks, the kinetic coefficient while it
slides (§6.5). That set is what the engine uses, solved with the axes' in one
box-constrained problem at velocity level. Built: the key's balance friction,
set-valued in the settle solve, stored as a key friction weight with its arm.
Planned: every pivot and the four sliding contacts, in every solver.

### 3.6 Relative coordinates for the jack and the repetition lever

Planned, with the dynamics. One note has six coordinates: key, wippen, jack and
repetition lever each relative to the wippen, hammer, damper under-lever. The
bodies are linked by contacts, not joints; the only pins are those of key,
wippen, hammer and damper lever to the frame and of jack and lever to the
wippen, so in these coordinates the tree of pins is complete: joints add no
algebraic constraint, and the motion is an ordinary differential equation in
minimal coordinates, constrained only by unilateral contacts. Jack and lever,
siblings, have no mass coupling, exactly; couplings and Coriolis terms live in
the (wippen, jack, lever) block alone. Ruled out: absolute angles with pin
constraints, a differential-algebraic system with its drift.

Couplings come from the kinetic energy, tested against its finite differences.
A child pinned at **r** on the wippen, centre of mass at **u** from its pin,
turns at θ̇_w + q̇ while its centre of mass moves at
θ̇_w ẑ × (**r** + **u**) + q̇ ẑ × **u**: its coupling to the wippen is
I_cm + m(|**u**|² + **r**·**u**), its share of the wippen's diagonal
I_cm + m|**r** + **u**|². **Do not use** I_cm + m(**r**·**u**) and
I_cm + m|**r**|²: they fit neither relative nor absolute coordinates, and a
positivity test passes on them anyway.

### 3.7 The two key ratio conventions; ratios are outputs

The **force convention**, KR = force at the front over force at the capstan,
read with 10 g at the capstan ("−5.5 … 0.55") [V, Stanwood 2000], about 0.50 to
0.60 on most actions and 0.68 on an 1878 Steinway concert grand, "unusually
high" [V, Igrec, *Practical Touch*, 2016, scenario 5], is the one in the
equation of balance. The **geometric convention**, front
arm over capstan arm, "nominally 2:1" [V, Stanwood, pianotech mailing list], is
its reciprocal: put into the equation it inflates the wippen term by 1/KR²,
about 3.3, and makes a wippen balance weight larger than the wippen weight, a
reducing lever appearing to amplify at a plausible value. Built:
`Touchweight::conventions_are_sane` asserts WBW < WW and 0.3 < KR < 1.0, a
window that separates the two conventions without rejecting a real action;
`the_inverted_key_ratio_is_rejected` and
`an_unusually_high_real_key_ratio_passes` test it.

**Ratios are outputs, never inputs.** The key weight ratio changes only through
the capstan's position, the measuring point and the bearing point, the action
ratio through those and the knuckle's distance from the shank centre. The
trade's definitions of the action ratio (lever product, a gauge read at the top
of the stroke, Stanwood's strike ratio, the ratio of useful travels) differ on
one action; each is to be reported, labelled (planned). Built: the key weight
ratio is computed from the arms and has no setter.

### 3.8 Every number has its provenance

Every value the engine starts from has a record in `data/params/` with its unit,
source and reliability marker (`V`, `E` or `S`), and a derived value has its
script in `tools/derivations/`. The owner's measurements are primary sources,
marked `V` with the instrument and the date. A value whose source is unpublished
is provenanced; one whose source is lost is not. Built: records for the key
section, the key frame, the papers, the touchweight protocol's measuring point
and key-ratio load, and Stanwood's weight curves, which the engine does not
read: its defaults are Rust literals repeating them, and several defaults have
no record (listed in `data/params/README.md`). Planned: the parameter registry
loads them, and a test fails on a default with no record.

### 3.9 A dedicated planar solver, not a general-purpose engine

Six coordinates and seventeen contacts per note (§6.6) make a small multibody
problem; the work is in contact geometry, material laws, friction and
protocols, and three things a general-purpose engine lacks decide it.
(1) **Per-contact nonlinear felt laws with memory**: what the finger feels and
when escapement happens come from felt and leather compressing, each interface
with its own law of penetration, rate and history, while game engines expose a
scene-wide numerical softness (Rapier's default: a 30 Hz spring, damping ratio
10 [V, Rapier source]) and accept no force law per contact. (2) **Exact analytic
lever arms at sub-millimetre scale, in double precision** (§6.2), where a
collision pipeline returns contact points with margins and Rapier's browser
build is single precision. (3) **Contact softness and margins are global and
millimetric**: at metric scale Rapier tolerates 5 mm of penetration and
predicts contacts 20 mm out [V, Rapier source]; both scale with a length unit,
neither becomes a law per contact.

**Do not use "no game engine has dry friction at a joint".** Jolt's hinge
constraint has a maximum friction torque, and Box2D documents joint friction as
a motor held at zero speed with a bounded torque, the box constraint of MuJoCo's
friction loss [V, Jolt `HingeConstraintSettings`; Box2D manual]; joint friction
does not distinguish a dedicated solver, nor does the reflected-mass ratio at
the knuckle (§4.1). **MuJoCo** has joint friction loss, polynomial springs,
reduced coordinates, double precision and an official WebAssembly build, but its
friction loss is a soft constraint, so a joint creeps under a sub-threshold
load: an optional oracle for cross-checks (planned), not a dependency.

### 3.10 Rust compiled to WebAssembly; notes on independent workers

**Built.** The domain is Rust, compiled to WebAssembly for the bench and
natively for `pa-cli` and the tests, for reasons that are not speed: arithmetic
reproducible across machines (ECMAScript leaves every transcendental but `sqrt`
implementation-approximated, and engines differ; Rust with the `libm` crate
does not, §4.4); no garbage collector in a fixed-step loop; and native runs of
the code that ships, for studies, sweeps and golden tests. **WebAssembly threads:
no.** On the pinned Rust 1.99.0, linking `pa-wasm` with shared memory fails
(`rust-lld: error: --shared-memory is disallowed by std-… because it was not
compiled with 'atomics' or 'bulk-memory' features`), and rebuilding the standard
library needs nightly; reproduce it on the `cdylib`, since a library crate does
not link and passes with the same flags. The planned 88-note runtime is
therefore one module per Web Worker (§7.6).

### 3.11 React and react-three-fiber for the bench

**Built.** React, with three.js through react-three-fiber and drei, covers a 3D
scene inside a dense interface of panels and readouts; `three`, its types,
react-three-fiber and drei are pinned together through pnpm's catalog. Labels
and values sit in HTML over the canvas, never as 3D text. Ruled out: Babylon.js,
with a better inspector and bundled types but no equivalent of the React
ecosystem for the interface. Planned: linings drawn compressed by the solver's
penetration, dimensions on the section, one instanced mesh per part family for
88 notes, and the realistic rendering of §1.1: materials and detailed shapes
as skins riding on the parametric solids without replacing them, so the
realistic view cannot drift from the physics any more than the section can.

### 3.12 A bilingual interface

**Planned.** English and French: the browser's language is detected, English is
the default, a selector overrides it. Rust sends codes and numbers across the
boundary, never sentences; all wording lives in one catalogue per language, the
French one in the owner's trade vocabulary (glossary:
`docs/research/A3-nomenclature-and-regulation.md` §6). The primary user works in
French and the vocabulary is the point; contributors read English; wording built
in Rust cannot be translated. Built: an English bench, its faults and verdicts
sentences built in Rust (§7.2).

## 4. What the dynamics prototype settles, and what it leaves open

**Built**, and throwaway: a one-note dynamics prototype on Hirschkorn's Boston
GP-178, note 52, in Python, reading his record and assembled with the pivots
read correctly (§6.4), in `tools/dynamics-prototype/`. Its level-0 checks pass,
and its `RESULTS.md` holds the step-convergence and solver study, generated by
`study.py`: four integrators, four scenarios (a slow weighing-like press, a
forte and a piano press under position drive, a forte driven by force, each with
its release), fixed steps from 0.25 to 500 µs. Every result in it is a
consistency check (§5.1): it says how the model behaves under its integrators,
not that the model is right. §4.1, §4.2 and §4.6 are settled by it; §4.3 and §4.5
remain hypotheses.

### 4.1 The time step

**Settled: one fixed step of 50 µs, no sub-steps.** With the solver of §4.2,
every trade observable (let-off, aftertouch, drop, checking, the rest regained
after release, down and up weight), the hammer's speeds within 1 %, and the key
force of the slow press within 1 g and of the piano press within 0.1 N stay
within their resolution up to 50 µs in all four scenarios; 70 µs holds too, and
the integrator stays finite up to 160 µs, first diverging at 180 µs in the forte
press [E, prototype]. One curve asks for less: the forte press's key force under
position drive, within 0.1 N only at 2 µs, its error first order in the step
through the whole stroke rather than gathered at the impacts; when a forte key
force is compared with a measured one, the same solver runs offline at that
step. The stiffest mode met is 1475 Hz, the repetition lever on the drop screw
just after let-off, and the explicit limit it and the contact damping set falls
as the force rises: harder blows than the forte press are outside what the study
ran, and the study is rerun when the felt law or the geometry changes.

**How the starting estimates compared.** With the pivots read correctly and effective masses
taken along the contact normals, Hirschkorn's action at rest has its highest
linearised mode near 1.2 kHz with the capstan, the front punching and the
jack/knuckle loaded at assumed fortissimo forces, and near 2.2 to 2.3 kHz with
the repetition lever and drop screw loaded too, or all thirteen contacts at once
[E, 10]. Those forces are assumptions: 100 N at the jack/knuckle means a
hammer-head acceleration of about 1,600 m/s² [E, 08].

**Do not use 12 µs.** It comes from taking Hirschkorn's `Pw2`, the jack's pin,
for the wippen's pivot, and from an effective mass I/r² with r the distance from
a pin to the contact point instead of the moment arm of the contact normal. That
gives 0.73 g and 3.9 kHz at the jack/knuckle; at rest, along the contact normal,
that effective mass is 8.9 g [E, 10]. **Do not use 570:1**
for the reflected-mass ratio there: at rest it is about 9 g on the driving side
against 467 g on the hammer side, 51:1 [E, 10]. **Do not use 2/ω as the
stability limit**: it is the undamped limit of an explicit spring, the limit
widens with the contact damping implicit in velocity, and stability is not
accuracy [E, 05].

### 4.2 One dynamic solver

**Settled: explicit on the contact laws, with both frictions as one set.** One
dynamic solver serves the bench and the keyboard: semi-implicit Euler on the
compliant contact laws, and the axis friction and the four sliding contacts'
friction (§3.5) as box-bounded unknowns at velocity level, solved together by a
projected Gauss–Seidel sweep finished by an exact active-set pass, at the fixed
step of §4.1 [E, prototype]. **Rejected on accuracy: compliant contacts treated
implicitly by backward Euler.** It is stable at every step tried, but it damps
every contact mode: the back check catches the hammer 25 µs late per
microsecond of step, and it holds the observables only to 2 µs where the
explicit solver holds them to 50 µs or more [E, prototype]. A trapezoidal step
was not tried; it would matter for a much stiffer felt law. The quasi-static
solver is a separate static solve on the same contacts and friction sets,
inertia dropped, solved to convergence under an iteration cap (planned). With compliant contacts
the configuration depends on the forces, so each load increment of a weighing
is a nonlinear complementarity problem; reading D and U off two linear programs
holds only for rigid contacts on one coordinate.

**Still to test.** Hirschkorn's forte, piano and double-blow trajectories,
Thorin's key force under position drive, and the trade's protocols at states the
static and dynamic solvers share. A second dynamic solver comes only if a
measured criterion fails. Thorin's model, compliant felts and set-valued friction, gives
very similar results at 2 and 10 kHz and significant discrepancies at 0.4 kHz
[V, Thorin 2013 §6.4.1]. Ruled out: three solvers specified before any
trajectory existed, doubling the costliest code for a cross-check that
convergence studies and published data provide.

### 4.3 The felt law

What felt does, in Hirschkorn's words: it "does not follow any simple power
function"; it "requires more force to compress than it returns on
decompression"; and "whether the felt is loaded slowly or quickly seems to have
little effect on the loading profile. Instead, it is dependent on the maximum
and minimum loads the felt has previously undergone" [V, Hirschkorn 2004
§2.2.2].

**Hypothesis.** Hirschkorn's cubics (his Table A.8), each fitted to the mid-line
between a contact's measured loading and unloading curves [V, Hirschkorn 2004
§2.2.3], are the elastic term: all thirteen are positive and strictly increasing
for any penetration [E, 04, 05]. The target is a rate-independent law with
loading and unloading branches and memory of its turning points, after Masoudi
and McPhee's published speed-independent model of felt hysteresis, identified
for hammer felt and applied to an upright piano action [V, Masoudi & McPhee].
Its identification needs loading and unloading curves contact by contact, which
Hirschkorn plots for the key front alone and no source in the corpus gives for a
grand action.
**Test.** Each law in the prototype against Hirschkorn's piano and forte blows,
which exist only as figures in his thesis and are not digitized yet. What the
prototype shows meanwhile: without hysteresis, the back check holds the hammer
with 61 N through a forte hold [E, prototype], most of which a law with an
unloading branch would release; and every candidate law reruns the step study,
since its tangent stiffness sets the stiffest modes.
His model fails on the piano blow; damping recalibrated for piano speeds, four
to five times larger, left the discrepancy, and he points instead at unloading
curves that differ at lower peak forces, and at friction [V, Hirschkorn 2004
§4.2.2]. A rate-independent law with memory answers the first directly.

Whatever the law, **the clamp max(0, f) is mandatory** with any viscous term:
Hunt and Crossley's f(δ)(1 + Dδ̇) turns attractive below δ̇ = −1/D, between
−0.11 and −5.7 m/s on Hirschkorn's contacts [E, 04], and Thorin's additive
damping b_d δ² δ̇ does not escape it, since with one constant b_d the two forms
agree only at the penetration b_d was calibrated at and the additive one turns
attractive too [E, 07]. **Do not use** the claim that the additive form,
calibrated to the same damping, turns attractive "at the same speed at every
penetration": that needs b_d recalibrated at every penetration. **Stulov's
hereditary law**, if used for the string, needs an exact update of its memory
variable with the input varying linearly across the step; holding the input
constant over the step removes most of its dissipation. **A non-integer power is
allowed**: the `libm` crate makes `pow` as reproducible as `sin` (§4.4), and the
law is chosen on physics and measured cost.

**Do not use "no published rate-independent felt model"**: Masoudi and McPhee
publish one. **Do not use "a factor of 56"** between Hirschkorn's and Thorin's
jack/knuckle stiffness: it is the ratio of the cubic coefficients, 3.92e11
against 7e9; at equal force, 10 to 100 N, the tangent stiffnesses differ by 3.1
to 3.6 [E, 05]. The two laws come from different actions, benches and law forms,
and enter the data as two labelled material sets, never merged.

### 4.4 Determinism, stated correctly

The arithmetic is identical bit for bit on every machine when all of these hold:
IEEE-754 operations only, with no fast-math and no contraction of a multiply and
an add into one fused operation (Rust guarantees both on x86-64 and wasm32); a
fixed order of operations, contacts and reductions, never the iteration of a
`HashMap` in the solver; every transcendental through the `libm` crate, `pow`
included, since `sin`, `cos`, `exp` or `pow` from a platform library or a
JavaScript engine differ between machines (`sqrt` is exact under IEEE-754); no
relaxed SIMD, whose fused multiply-add rounds once on hardware that has one and
twice elsewhere; and any randomness seeded, the seed saved with the scene.

Under those conditions a loop that stops on a tolerance stops at the same
iteration on every machine. So **convergence-tested loops are allowed**, with a
hard iteration cap and the iteration count reported, and **required in the
metrological paths** (the quasi-static solver, the inverse regulation solve,
the protocols), where an unconverged answer is worse than a slow one. **Fixed
iteration budgets belong to the real-time path**, where the cost per step must
be bounded; they also keep outputs smooth in the parameters, as
finite-difference sensitivities need. **Do not use** the argument that a
convergence test destroys reproducibility: with identical operations in the same
order it does not, and with different arithmetic a fixed count would not restore
it either.

The toolchain pins, Rust 1.99.0 and Node 26, keep tools, lints and formatting
identical between machines; the `libm` crate, not the pins, makes native and
WebAssembly results agree. Built: `libm` is `pa-core`'s one dependency, and the
sine and cosine of every pose go through it (§2.1); Clippy refuses in `pa-core`
every `f64` method IEEE-754 does not round exactly, `sin` to `hypot`, `powi`
included (`crates/pa-core/clippy.toml`), so no transcendental in the domain
reaches the platform's maths library, and a test that wants one for a reference
value allows it explicitly. `pa-app` still calls the platform's `atan` and
`asin` for two displayed angles. No `HashMap` in the domain's production code;
two bisections, the settle solve's 24 steps and the friction protocol's 32,
resolve far below anything read. The golden sheet (§7.1) holds the engine's
readings bit for bit wherever the tests run.

### 4.5 Rigid bodies

**Hypothesis.** The key and the hammer are rigid. Thorin's criterion, a part's
first bending mode against the duration of the event, holds for the key (first
mode 355 Hz) and fails for the hammer (39 Hz, (f·τ)⁻¹ = 5.1) [V, Thorin 2017
§2.1 and Appendix], and before let-off, the phase that matters for touch, a
flexible shank sits in series with the drive. **Test.** The prototype with and
without a degree of freedom between head and shank, compared on key force under
position drive; untested so far, the prototype having no head–shank coordinate
and the record no shank stiffness. **Do not use 280 Hz** for the hammer's first mode, nor the
180 µs derived from it as a twentieth of its period: the figure is in no source.

### 4.6 The 88-note budget

**Hypothesis.** One dynamic solver, notes computed independently on a few
workers, plays 88 notes in real time on a laptop: a note is active for a few
hundred milliseconds and static in its hold, so far fewer than 88 move at once.
**Counted:** about 2,400 floating-point operations a step with the solver of
§4.2, plus 15 transcendental calls: 49 million per simulated second for a note
at 50 µs, 4.3 billion if all 88 notes moved at once [E, prototype], which they
rarely do. **Still to measure:** the cost per step natively and in Chrome and
Safari, and the notes active at once over a corpus of MIDI files, pedal changes
included.
**Do not use** the per-step costs quoted in annex A1: the benchmark kernel
behind them is not in the repository.

## 5. Validation

### 5.1 What a test proves; drive in position, compare forces

A **consistency check** shows that the model agrees with itself or with
mathematics: a round trip, a closed form against a solver, a derivative against
a finite difference, a conservation law. It catches coding errors, and a wrong
model passes it. A **validation test** predicts outputs measured independently
from inputs measured independently, on the same instrument; only validation says
the model is right. A validation test says so; every other test is a consistency
check, as the whole stage-1a suite is. The first validation test is §5.2(a).

**Drive in position, compare forces.** The action is dominated by inertia until
escapement: driven by a force and judged on displacements, a deliberately wrong
model, its felts made rigid (on a real action, Thorin notes, the hammer would
probably not even reach the string), scores 9 % error against 3 % for the
complete model [V, Thorin 2017 §5, after Thorin, Boutillon and Lozada 2014]. So
the dynamics are validated by driving the key in position and comparing the
reaction force at the finger, the curve a technician reads, and the **Bad-Model
guard** belongs in the suite: stiffen the felts, and if validation still passes,
the validation is worthless. Planned.

### 5.2 Validation instruments

**(a) One real key, measured by the owner** (planned). Its mass and its front
weight by the trade's protocol are inputs (§2.3); its key weight ratio on the
jig and its down and up weights on its frame are outputs, the bench's measuring
point set to where he weighs before anything is weighed (§3.1). It is a modest
validation: the ratio is close to a lever identity, and a bare key's balance
weight differs from minus its front weight only by what the bearing point and
the rest attitude add. But it is the first comparison of computed readings with
independent ones, and it exercises what stage 1a builds: arms, datum, bearing,
protocols.

**(b) Hirschkorn's Boston GP-178, note 52** (planned): geometry, masses,
thirteen contact laws and five axis frictions measured on one action [V,
Hirschkorn 2004, Appendix A]. With every frame's origin at its own pivot, his
geometry closes on contacts 1, 3, 5, 7 and 10 with the hammer 47.0 mm from the
string and one interpenetration, 0.88 mm, where the repetition lever rests on
the wippen (contact 6, his "Whippen – Repetition") [E, 10]. That pose is not a
rest state: the overlap carries about 16 N the repetition spring cannot hold.
At the frictionless equilibrium the hammer stands 48.16 mm from the string,
against the 48.0 mm he regulated [E, prototype]. Let-off, though, comes about
13 mm from the string where he regulated 1.0 mm, and the drop screw engages
after let-off: he regulated his model by editing geometric parameters he does
not list [V, Hirschkorn 2004 §2.3], so the action as printed is regulated to his
targets (key dip 10.0 mm, hammer at rest 48.0, let-off 1.0, drop 3.0, checking
15.0) before its trajectories are compared with his (planned). His experiments drive the key with a force and measure
positions (forte, piano, double blow): the dynamics prototype's reference. It
is also the engine's reference action: the base profile the engine is built and
checked on before any other action is compared with it (planned), its values
transcribed with their locators in `data/params/`.

**(c) Thorin's Renner demonstrator** (planned): the only published key force
under position drive, and escapement velocities measured and simulated, 1.01 and
1.00 m/s piano, 1.56 and 1.56 m/s forte [V, Thorin 2017, Table 5]. It was
adjusted by a professional technician and carries no key leads [V, Thorin 2013
§1.6]: do not read its heavy touchweights as wear. Its parameters have lineage
traps (§6.3).

### 5.3 Levels

| Level | What it checks | Kind | Status |
|---|---|---|---|
| 0 | lever arms against central finite differences; the net torque against the gradient of the potential; the mechanism's Jacobian columns against central differences of its poses, and its generalized forces against the gradient of its potential; every contact row's gradient and sliding against central differences, at depths past any radius, its sign against the side of its drawn piece; the contact rows against a brute-force sampling of the drawn outlines; every contact surface a range of a drawn part; the unilateral virtual-work inequality at every settled state; every stop's face against a declared range of a drawn part; the key ratio guard | consistency | built |
| 0 | the mass matrix against finite differences of the kinetic energy; normals continuous across the hybrid profiles; every coordinate acted on by a contact or a spring | consistency | planned |
| 1 | step convergence: one scenario at several steps (§4.1) | consistency | planned |
| 2 | energy in free flight, contacts and friction off: bounded for the hammer alone; for the (wippen, jack, lever) block, whose mass matrix depends on its configuration, a drift falling as O(dt) | consistency | planned |
| 3 | published data, §5.2 (b) and (c) | validation | planned |
| 4 | the trade's protocols run on the model: the jig, Key Friction Weight | consistency | built |
| 4 | every weighing quantity by procedure; let-off, drop, checking, aftertouch and blow distance as the trade measures them; §5.2 (a) | consistency, and validation where the measurement is independent | planned |
| 5 | sensitivities by finite differences: what moves when a setting changes | consistency | planned |

**Non-confusion tests** (built, on the closed-form weighing algebra and through
the whole chain in the smoke test): 1 g of lead (of mass on the key, once
leads leave the model) moves BW and not F; axis friction
times 1.2 moves F and not BW. With a constant friction bound both hold by
construction in the closed form, guarding against coding mix-ups until friction
depends on load.

**Sensitivities.** Spurlock's rule, +1 g of hammer weight for +5 to +7 g on both
down weight and up weight [V, Spurlock], checks the lever chain's consistency:
it is the strike ratio. **Do not use "+0.1 mm of capstan ⇒ −1.4 g of down weight
and −0.3 mm of aftertouch"**: it comes from a display mock-up, and raising the
capstan increases aftertouch. **Do not read Thorin's ΔJ/J as a ranking of
sensitivities**: it is the gap between the cost at a parameter's measured value
and its minimum, its large values (up to 42 %) due by his account to a
mis-setting of his demonstrator [V, Thorin 2013 §6.5.3]; a validation target is
a property of the physics or of a protocol, never one instrument's regulation
error. Nor are settings optimised by gradient descent: Thorin finds his cost
function convex along a few parameters and flat along many, and concludes that
"common methods of convex optimisation should probably not be applied to the
piano action" with it [V, Thorin 2013 §6.5.6]. A scripted regulation sequence
or a grid search does the job.

## 6. Physics notes for the next stages

### 6.1 Springs inside the wippen assembly

**The lemma.** A massless spring pivoted on one of the wippen, the jack and the
repetition lever, and bearing on two of them, has exactly zero generalized force
on the wippen's coordinate of §3.6: turning the wippen with the jack and lever
angles held turns the whole assembly rigidly, spring included, without
deforming it, whatever the spring's shape.

**Its limit.** The lemma removes only the spring's direct term. Whenever the
repetition lever bears on the knuckle or on the drop screw during a weighing
stroke, its angle changes along the stroke, the spring stores or releases
energy, and the force at the finger moves: D, U and BW. The trade says as much:
the repetition spring "aids in forcing the repetition onto the capstan screw and
key. This force assists in moving the key towards its rest position" [V,
Steinway technical reference, ch. 2], and hammer height at rest "is affected by
the tension of the repetition spring, its friction" [V, Igrec, *Pianos Inside
Out*, p. 173]. **Do not use "the support spring is the only spring that can move
BW".** The wippen support spring, anchored to the frame, acts on the wippen
directly, and "engaging the support spring equally reduces Up Weight (U), Down
Weight (D) and Balance Weight (BW)" [V, Stanwood 2000]; but that holds for any
static load depending on position under friction not depending on load, so it
cannot tell spring arrangements apart. The quasi-static solver carries every
spring and measures its effect.

**What the arrangement changes.** One spring shared by jack and lever ties their
torques in a fixed geometric ratio, two springs are independent, and a loop that
can only pull and go slack differs from branches that push and lift off: two
contact-state machines, diverging at drop and checking. A spring's datum is its
geometry (pivot, arms, bearing points), not a flat stiffness. Hirschkorn
measures T = −0.073(θ_r − 0.832) N·m on the lever-to-wippen angle and a roughly
constant 0.0042 N·m on the jack [V, Hirschkorn 2004 §A.4]; Thorin's set has
0.1171 N·m/rad [V, Thorin 2013, Table 4.12]; a factor of 1.6 is no anomaly,
spring tension being a regulation setting adjusted by bending the wire.

**Its criterion is published**: strike the key, let the hammer check, release
the key slowly; the hammer must rise to the drop point evenly, without jumping
or catching [V, Steinway technical reference ch. 2; Kawai grand regulation
manual]. At stage 2 it becomes observables: rise time; jitter, counted on
reversals of contact state, not on velocity sign changes, which count the
solver's chatter; float; and the finger's impulse, since a peak against a stiff
stop scales with the step.

### 6.2 The lever arm, and the gradient of a gap

For a contact between bodies A and B, with contact point P, unit normal n
pointing from B to A, and O the pivot about which a coordinate turns its body:

```
∂g/∂θ_A = ((P − O_A) × n) · ẑ        ∂g/∂θ_B = −((P − O_B) × n) · ẑ
```

That is the instantaneous lever arm, what every ratio of the trade is made of.
For a coordinate relative to a parent (jack, lever), O is its own pin, and the
parent's coordinate turns the child's points about the parent's pivot. Built for
circle against line and circle against circle at one coordinate, as the
mechanism's Jacobian column for any material point over every ancestor joint,
and for every contact row over every coordinate, `n · (J_A − J_B)` at the
contact point, a coordinate that moves both bodies cancelling exactly (§2.1);
each tested against central finite differences.
**Do not use** ∂g/∂θ_B = −(ẑ × (x_c − o_B))·n_B −
(c_A − p_B)·(ẑ × n_B) for a circle on A against a line on B: the second term is
spurious (a circle at (0, 2) of radius 1 against a line through (5, 0) on a body
turning about the origin has a true derivative of 0; that formula gives −5). And
the arm is not the distance to the pivot: at rest the knuckle sits on the jack's
flat top, whose normal runs close to the jack's pin, so the jack's own arm is
3.5 mm though its contact point is 49.5 mm from the pin [E, 10]. Arms change as
contact points migrate, so a lever ratio is never a constant; the literature's
small-angle ratios are linearizations about rest.

### 6.3 Parameter lineage

Planned: each value transcribed with its lineage: measured, assumed, placeholder.

- **Lozada measured the hammer's axis friction and assumed the rest**: he states
  that he identified the hammer-to-frame joint and considered the other joints
  to behave the same [V, Lozada 2007, ch. 3 §1.3]. The wippen, lever and
  damper frictions in his Table 2.3 copy the hammer's 9.92e-4 N·m, and Thorin
  gives the jack the same value as a placeholder, so
  their agreement with Hirschkorn compares nothing measured, and the wippen and
  damper values printed as 9.92e-2 N·m, a hundred times the hammer's, are by his
  own assumption a transcription error.
- **Thorin's key viscous friction, 0.22 N·m·s, is a measurement, not a typo**:
  Lozada's regression on the key gives 0.05 N·m dry and 0.22 N·m·s viscous [V,
  Lozada 2007, Table 3.1]. That key's dry friction has three published values,
  0.05 N·m (Lozada), 0.012 (Thorin 2013, Table 2.1) and 0.006 (Thorin 2013
  §4.5.2), kept side by side with their sources. It weighs 103 g in Lozada and
  83 g in Thorin §4.5.2, which keeps Lozada's key inertia.
- **Thorin's jack and lever mass, inertia and centre of mass are placeholders**,
  his category IV, "fixed parameters not to be studied": round values, centres
  of mass at the pins [V, Thorin 2013 §4.4.1]. His wippen inertia, 3.97e-4 kg·m²
  for 19.8 g with the centre of mass 53.5 mm from the axis [V, Thorin 2013
  §4.5.3], implies a radius of gyration about the centre of mass of
  √((3.97e-4 − 0.0198 × 0.0535²) / 0.0198) ≈ 131 mm, for a part about 100 mm
  long.
- **Felt damping**: the 5e7 Thorin attributes to Lozada differs from Lozada's
  own 2e7 for felt and 1e3 and 3e3 for the knuckle's and drop screw's leathers
  [V, Thorin 2013 Eqs 4.9–4.10; Lozada 2007, Table 2.3].

### 6.4 Hirschkorn's frames

Every body frame in Hirschkorn's Appendix A has its origin at the body's pivot,
as his Figure A.1 draws it: the ground pins are Pg1 (key), Pg2 (wippen) and Pg3
(hammer), and in the wippen's frame Pw2 is the jack's pin and Pw3 the lever's,
both listed as "Pin Location" [V, Hirschkorn 2004, Figure A.1, Tables A.2–A.4].
Read that way the action closes at rest (§5.2 b). **Do not take Pw2 for the
wippen's pivot**: that misreading is what gives the 12 µs step, the 570:1 ratio
and the claim that his rest pose cannot be reconstructed. **Do not use 5.84 as a
closed test of the action ratio**: that lever product multiplies radial
distances, takes Pw2 for the wippen's pivot and the jack's pin for its contact,
and uses the front punching, 217.5 mm out, as the key's front arm instead of the
measuring point, and a band of 5.0 to 7.0 could not see a 10 % error in an arm
anyway. The action ratio is read off an assembled model, by virtual work.

### 6.5 Friction

- **At the axes**: set-valued (§3.5). Hirschkorn measures five pivots, key
  0.0122, wippen 3.97e-4, jack 3.97e-4, repetition lever 2.47e-3 and hammer
  1.01e-3 N·m, kinetic only, leaving static friction out of his joints [V,
  Hirschkorn 2004, Table A.9]; joint friction is set by the tightness of the
  bushings, constant once the action is assembled [V, Hirschkorn 2004 §2.2.4].
- **At sliding contacts** (capstan and heel, back check, jack and knuckle, lever
  and knuckle), his law, μ = A(tanh(s_t/v_t) + B1(s_t/v_t)/(1 + B2(s_t/v_t)⁴))
  [V, Hirschkorn 2004, Eq. 2.10], gives static peaks 1.69, 1.72 and 1.13 times
  the kinetic coefficient at the back check, the jack/knuckle and the
  lever/knuckle, and none at the capstan [E, 07]. **Do not drop the bracket**:
  A multiplies both terms on the typeset page, and a plain-text extraction of the
  thesis loses it, giving a different law with peaks of 3.0, 6.6 and 1.9.
- **Do not use "~40 % of friction unexplained by the published axis torques"**,
  nor the prediction F ≈ 9 g, D ≈ 44 g, U ≈ 26 g drawn from the same sum: it
  leaves out two of the five pivots, the repetition lever among them, mixes
  Hirschkorn's torques with Thorin's ratios and arm, and counts measured
  sliding-contact friction as missing [E, 06]. Any unexplained share comes from
  an assembled model over all five pivots and the sliding contacts, against a
  measured F.
- **At rest** the equilibrium is an interval, not a point: where a hammer
  settles depends on how it got there, as a technician bouncing a key sees. The
  engine fixes one initialisation (frictionless equilibrium, then friction, then
  relaxation) and reports the interval's width (planned).

### 6.6 Seventeen contacts per note

Hirschkorn's thirteen, and four for the damper (planned):

| # | Contact | Regulation it carries |
|---|---|---|
| 1 | capstan, wippen heel | capstan height |
| 2 | key front, front rail punching | key dip, aftertouch |
| 3 | key tail, back rail cloth | rest position |
| 4 | back check, hammer tail | checking; the back check's angle |
| 5 | wippen, jack rest | the jack's position under the knuckle |
| 6 | wippen, repetition lever stop | the lever's height |
| 7 | jack, knuckle | the main force path |
| 8 | jack toe, let-off button | let-off |
| 9 | jack, repetition lever window | — |
| 10 | repetition lever, knuckle | repetition |
| 11 | repetition lever, drop screw | drop |
| 12 | hammer, string | blow distance |
| 13 | hammer shank, its rest | hammer rest height, on a rail only |
| 14 | key tail, damper under-lever | each damper's lift point |
| 15 | lift tray, damper under-lever | collective lift, pedal travel |
| 16 | damper felt, string | damper seating |
| 17 | damper under-lever, stop rail | damper stop clearance |

Three primitives cover Hirschkorn's thirteen: a circle against a line (ten), a
circle against a circle (the capstan on the heel), and a hybrid, an arc joined
to two tangent lines, for the jack and the lever under the knuckle, whose normal
never jumps. The first two are built as signed distances, which a compliant law
needs. A disc against a straight face, the disc an arc or a convex corner taken
as a disc of radius zero, measures to the face's line and keeps its sign at any
depth, as does a face's centre against a face. Two discs keep it until their
centres meet, a penetration of both radii added, and have no row past that.
**Do not use** "a corner deeper than an edge arc's centre stands over a face
beside the arc, whose own row carries the penetration" as a guarantee: that row
exists only while the face's normal, turned round, lies in the corner's own
angle, and a key's tail corner sunk 0.9 mm into the end of its cloth with the
key nose-up reads no row at all, though the cloth's sides are in its range. The
rigid statics stop every corner at contact and never go that deep; a felt law
that lets a corner sink past an edge's radius needs a reading that sees it. The
hybrid is planned. The older circle-against-line gap, which still serves the
one-coordinate statics, is an unsigned distance from the circle's centre less
the radius, so past one radius of penetration it grows again: right at contact,
where those statics read it, and wrong for depth. The shank rests on a cushion
on the wippen or on a felted rail, depending on the action, which changes the
contact's gradient, the shock at the finger, the rest state and the regulation
offered. Contacts 14 and 15 both lift the damper lever from below and the higher
wins, so pedal force falls as notes are played.

## 7. Architecture

### 7.1 Crates, apps and gates

**Built.**

```
crates/pa-core     the domain: geometry, mechanism, contact, parts, measure. No I/O, no web;
                   one dependency, libm (§4.4)
crates/pa-app      the application: the single-key bench, KeyBench; the golden sheet
crates/pa-wasm     the browser's adapter of pa-app; the only crate that knows JavaScript
crates/pa-cli      the shell's adapter of pa-app: `weigh`, `golden`
apps/bench         the bench page: React, react-three-fiber
tools/smoke        drives the real page over the Chrome DevTools protocol
scripts/verify.sh  every gate, run locally by `pnpm verify`
scripts/smoke.sh   the browser smoke test: the built bench in a headless Chrome
```

The adapters add no physics and the React tree computes none of the numbers it
shows; `pa-wasm` carries a `package.json`, so Turborepo builds it with
wasm-pack. The CI workflow (`.github/workflows/ci.yml`) runs the format check,
Clippy with warnings as errors, the tests and the WebAssembly build for Rust,
and Biome, the type-check, the build, the tests and the smoke test for the web;
`scripts/verify.sh` runs the same gates locally.

**The golden sheet** records every reading of the default bench and of four
fixture benches (a frame whose stops fight, a back-heavy key, a dip stopped by
the front pin in its mortise, a key shortened under its frame) in full bits, in
`crates/pa-app/tests/golden/`. `pa-cli golden` writes it and `cargo test`
compares the engine with it, so a change shows every number it moves; one that
means to move a number rewrites the sheet and says why. The two angles the
sheet shows in degrees are recorded as the slope and the sine they come from,
because the platform's `atan` and `asin` may differ in the last bit.

**The engine and its applications.** The crates are the engine (§1.1) and
`apps/bench` its first application. Planned: the engine's interface is what
applications are written against. The boundary of §7.2, the parameter registry
and validation layer of §7.3 and persistence (§7.4) make the configurator, the
diagnostics and the state an application composes; a rendering package and an
interface package are extracted from the bench when a second application needs
them, not created empty before.

**One definition per part.** The key and its frame are built as parts: one
closed outline of lines and arcs for each piece of material, on the body that
carries it, several per body, a mortise a cut among them and a turned part read
through its sagittal silhouette. Each part declares the ranges of its outline a
contact may read and the named points on it, the lip and the key front's top
among them. The bench draws them and the faults read them; whatever a stop
rounds is drawn rounded, the back rail cloth's edges included. The contacts read
the declared ranges and nothing else: a contact pair holds nothing but ranges of
parts, and the mechanism its rows are read on carries the very parts the view
draws, which `every_contact_surface_is_a_range_of_a_drawn_part` holds on every
bench of the golden sheet. The one-coordinate statics that still place the key
read stops of their own, built from the same fields, the underside's corners
and the frame's bearing heights, and one test holds every face they read to a
declared range of a drawn part,
`every_stop_face_is_a_declared_range_of_a_drawn_part`: for the statics, the
invariant holds by shared fields and that test, not by structure. The composite
key's mass is integrated over a third polygon, its volume model's, which the
drawn parts tile exactly. Planned: the static solve on the contacts, after which
nothing reads anything but a drawn range, and each part's inertia derived from
its outline or measured, by an explicit choice, a key's always measured (§2.3).

### 7.2 The WebAssembly boundary

**Built.** The bench boots the module once (the boot promise is cached, so a
double mount cannot construct two engines) and drives it with one setter per
value. It reads two JSON documents: the scene and the readout. The scene holds
every part's outline in its own body's frame, in millimetres, with its material
and the provenance of each stretch; one pose per body, an origin, a cosine and
a sine, which the view applies and computes none of; the leads and the
measuring point on the key; the contacts within a millimetre of closing, each
its two surface points in the keybed's frame, its gap and whether it is closed,
open or overlapping; and the faults. The readout holds the settings, the
weighing sheet and what holds the key, all absent when the frame cannot be
weighed. Faults and verdicts are English sentences built in Rust, in `pa-app`
and in the adapter, and the TypeScript types mirror both documents by hand.

**Planned.** Settings and weighing come back separately: the settings always
valid, each adjustable value with its unit, admissible range and step; the
weighing nullable, with its reason when absent. Codes and numbers cross, never
sentences (§3.12), with TypeScript types generated from the Rust ones. Outlines
cross once per change of geometry, not with every scene, and poses at each
step. Per-step state is a flat `f64` buffer read through a pointer and a
length, the JavaScript view rebuilt whenever `memory.buffer` changes, since
growing linear memory detaches old views. A step that produces a non-finite
value stops and keeps the last finite state, so none is ever published.

### 7.3 The parameter registry and the validation layer

**Planned.** Every setting has a stable path, a unit, an admissible range, a
layer, an actuator with its step, and a provenance; the WebAssembly interface
(`set(path, value)`, `describe()`), the panels and the saved files are generated
from it, and the defaults load from `data/params/` with a test that each has a
record. `validate(design, regulation)` runs on every load and every edit, before
any solve: the outline does not cross itself, features lie inside the keystick,
the capstan clears the blocks, dimensions are positive, inputs are finite. A
violation names the limiting dimension and the shortfall; an action regulated
into jamming is a state to report, not an error.

### 7.4 Persistence

**Planned.** A saved file holds the coordinates, which are normative, and each
target with its achieved value and the solver's fingerprint (§3.4). On opening,
the coordinates are used as they are and the targets re-solved and compared; a
divergence is offered as a choice, never resolved silently. The schema is
versioned, each migration a pure function with a fixture. Drop is stored with
its reference, from the string or below let-off, because makers write it both
ways. Golden scenarios start from literal coordinates, so a change in the
regulation solver cannot pass for a change in the physics. `pa-core` gains
`serde` behind an optional feature for this.

### 7.5 The data model

**Planned.**

- **Layers**: *action design*, what the choice of parts fixes (geometry, measured
  masses, bearing point); *regulation state*, what the technician turns (each
  setting with value, range, source, keyboard zone, actuator); *material set*,
  each interface's contact law, with its lining's datum, where on the felt the
  gap is zero; *per-note data*, graduation curves and discrete attributes
  (natural or sharp, strings, damper type, section); *string plan*, string
  height, keybed crown, strike point, last damped note.
- **Three kinds of setting**, which decide what the interface offers: actuated
  (a control turned), derived (read-only), target (aimed at through an actuator).
- **A profile is a base plus a measured difference**, the base being
  Hirschkorn's action (§5.2 b). All layers but one are
  measurable with a calliper and the trade's gauges; the material set needs a
  compression bench, so it is inherited, and anything computed from an
  inherited layer says so and shows as indicative.
- **A measurement carries its provenance**: value and unit, the solver, its
  conditions (dampers and repetition spring engaged or not, where the stroke
  stopped), and its trust, metrological or indicative. A weighing without its
  conditions is not a datum.

### 7.6 The 88-note runtime

**Planned.**

- **Independent notes**: one WebAssembly module per Web Worker, with its own
  memory and notes. Notes share nothing because the damper lift tray is a
  prescribed position: the trapwork, one to three degrees of freedom driven by
  timestamped pedal input, is integrated identically in every worker.
- **Synchronized at the frame, never at each step.** One simulation-time master
  gives every worker the same target step; reductions across notes run in a
  fixed note order, in one place; the renderer interpolates between snapshots
  stamped in simulation time. Which worker computes a note may vary; the order of
  reductions may not. Transport is a lock-free ring in a `SharedArrayBuffer`
  where the page is cross-origin isolated, transferable buffers otherwise.
- **Hidden state** (felt memory, stick or slip at each friction set, contact
  flags) is saved with the state: positions and velocities cannot replay a
  session. **Time control** comes with the first dynamic stage: slowed to a
  thousandth, step by step, replayed, events marked.
- **Pedals** are continuous inputs from 0 to 1; the sostenuto latches the
  dampers raised when it is pressed; the una corda shifts the key frame and the
  hammers, not the dampers.
- **MIDI drives a force on the key, never a hammer velocity**, which would
  bypass the lever chain that regulation acts on. Each note and velocity's force
  is found by bisection on the model, lazily, warm-started, and again when the
  regulation changes, so a badly regulated let-off is heard and seen. Releases
  follow a profile; a repeated note starts from the action's current state.

## 8. Stages

One table, in order. Each stage ships its engine and its view together and ends
on a number the owner can judge by the trade's protocol.

| Stage | What is added | What becomes measurable | It ends when | Status |
|---|---|---|---|---|
| 1a | the key on its frame: rails, punchings, papers | key height, key dip, front weight, key weight ratio, key friction weight | a key the owner measures (§5.2 a) agrees with the bench by the protocol | built; its validation planned |
| dynamics prototype | one note of Hirschkorn's action in motion, thrown away afterwards | the step, the solver, the felt law, the cost per step (§4) | each hypothesis of §4 is settled by its test | built: step, solver and cost settled; the felt law (§4.3) and the flexible hammer (§4.5) open |
| quasi-static core | bodies in a tree, contacts over all coordinates, set-valued axis friction, solved to convergence under a cap, the key as first client and Hirschkorn's key as the default (§2.3); the registry, the validation layer, the new boundary, the bilingual interface (§7) | every weighing quantity by procedure (§2.2) | the key's readings come from procedures run on the model, the closed forms passing as oracles | the tree, the jig and the contacts over all coordinates built (§2.1, §2.2); the rest planned |
| 1b | the wippen on its rail, the capstan contact, the support spring where fitted | wippen weight, support spring balance weight | a weighed wippen agrees with its model by the protocol | planned |
| 1c | the hammer and its shank | strike weight, strike ratio | a weighed hammer agrees with its model by the protocol | planned |
| 1d | nothing new | the equation of balance, closed | D, U, BW and FW of an assembled key, wippen and hammer agree with the same parts weighed | planned |
| 2 | the jack, the repetition lever, let-off, drop, checking: the action in motion | let-off, drop, checking, the slow-release test, the key force under position drive | Thorin's key force and Hirschkorn's trajectories are reproduced within their measured scatter (§5.2) | planned |
| 3 | regulation training | every setting as a target and an actuator; exercises; faults named as the trade names them | the trade's regulation sequence runs end to end on the model | planned |
| 4 | dampers, pedals, 88 notes | damper lift point, the pedals, a score played | a score plays in real time with the stage-2 readings unchanged | planned |

## 9. Open questions for the owner

Each is stated so it can be answered without opening anything else.

1. **The repetition spring**, where one spring serves jack and lever: is it
   pivoted on the repetition lever, pulling the jack through a loop, or in the
   lever's support fork, with two branches? In the slow-release test, should the
   hammer rise more slowly in the bass, or at nearly the treble's speed?
2. **Pedals.** On the makes you work on, what moves with the una corda besides
   the key frame and the hammers: the sostenuto rod? From which note is the
   treble undamped?
3. **Training.** Is a regulating screw turned continuously or in clicks of a
   fixed fraction of a turn? Which measuring tools does a trainee have, and does
   measuring cost a gesture? Do exercises follow a full regulation sequence or a
   short path (blow distance, springs, let-off, checking)? What passes an
   exercise: targets within tolerances, evenness from note to note, or both?
4. **Sound.** In scope or not, and if so, samples chosen by the computed hammer
   velocity or a physical string?
5. **Licence.** Under which licence may the project be published? Nothing is
   published until one is chosen.

The French terms in doubt are listed in `docs/vocabulary-worksheet.md`. And an
opportunity: a compression bench, even a rudimentary one, would first repay the
four contacts that dominate the force the finger feels (capstan on heel, jack
and lever on knuckle, front rail punching); without one, every contact law is
borrowed (§7.5).

## 10. Sources

The corpus is in `data/references/`, outside version control: copyrighted
documents kept for study, never redistributed; the numbers taken from them are
committed in `data/params/`, each with its locator. No source is cited for a
trade term: the trade's vocabulary belongs to the trade.

- **Hirschkorn**, *Dynamic Model of a Piano Action Mechanism*, MASc thesis,
  University of Waterloo, 2004: Appendix A, the data.
- **Thorin**, *Non-smooth model of the grand piano action*, PhD thesis, École
  Polytechnique, 2013; with **Boutillon, Lozada, Merlhiot**, "Non-smooth
  dynamics for an efficient simulation of the grand piano action", *Meccanica*
  52:2837–2854, 2017, the model to match; with **Boutillon, Lozada**, "Modelling
  the dynamics of the piano action: is apparent success real?", *Acta Acustica
  united with Acustica* 100(6):1162–1171, 2014, on driving in position.
- **Lozada**, *Modeling, haptic control and novel concepts for musical
  keyboards*, PhD thesis, École Polytechnique, 2007.
- **Masoudi and McPhee**, "A novel micromechanical model of nonlinear
  compression hysteresis in compliant interfaces of multibody systems",
  *Multibody System Dynamics*, doi:10.1007/s11044-015-9483-6.
- **Stulov**, "Experimental and computational studies of piano hammers", *Acta
  Acustica united with Acustica* 91(6), 2005.
- **Stanwood**: *The New Touchweight Metrology* (Piano Technicians Journal, June
  1996); *Standard Protocols of the New Touchweight Metrology* (February 2000),
  the protocols; *Cutting the Balance Rail Punching to Effect Ratio* (2014); the
  strike-weight and hammer-weight curve standards; the rest of his series.
- **Inertia in the trade**: Nakamura, *Practical application of moment of
  inertia*, Parts 1 and 2 (2015), and *Understanding MoI* (2018); Igrec,
  *Practical Touch* (2016). They compute or rate inertia; none measures it.
- **Manuals**: Steinway's technical reference, chapters 1 to 4; Kawai's grand
  regulation manual; a compiled sheet of grand regulation specifications; the
  WNG instructions; Redekop's damper guide; Reyburn's geometry sheet; Spurlock on
  touchweight; the Potter checklist; a French trade glossary; Igrec, *Pianos
  Inside Out* (sample pages).

Not in the corpus, and wanted by the stages ahead: Askenfelt and Jansson on the
grand action's timing (1990); Gillespie et al.'s key force profiles (2011);
Timmermans, Dehez and Fisette's real-time haptic key (*Machines*, 2020);
Pfeiffer, *The Piano Key and Whippen* (1967); the Preisach model, which
Hirschkorn proposes for felt; quasi-static frictional contact and its
non-uniqueness (Pang and Trinkle 1996; Klarbring 1990); implicit compliant
contact (Todorov 2014; Castro et al. 2023); sensitivities across contact events
(saltation matrices; Leine and Nijmeijer 2004).
