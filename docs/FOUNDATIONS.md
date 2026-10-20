# Foundations: a grand piano action simulator

This document states what the simulator is for, what is built, what is planned,
and why. Every component and mechanism is labeled **built** or **planned**
where it is described. A decision that is not settled is labeled a
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

## What the simulator is for

### An engine for piano applications

The project is an **engine for building applications around the grand piano
action**, educational and playful alike, rather than one application. It is
built for a piano technician, who is also its first user, and it provides five
things an application composes:

- **the trade's physics**: the action's parts, contacts, friction and motion,
  measuring itself by the trade's protocols ([The acceptance criterion](#the-acceptance-criterion));
- **a configurator**: every part and every regulation setting adjustable, the
  way a technician regulates and a designer draws ([Regulation as targets; the coordinate is normative](#regulation-as-targets-the-coordinate-is-normative), [The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer));
- **a state**: a regulated action that can be saved, reopened, compared and
  replayed, its coordinates normative ([Regulation as targets; the coordinate is normative](#regulation-as-targets-the-coordinate-is-normative), [Persistence](#persistence));
- **diagnostics**: what is out of regulation and what cannot work, named in the
  trade's terms and quantities ([Upstream settings drift; faults are reported, not refused](#upstream-settings-drift-faults-are-reported-not-refused));
- **a realistic rendering in 3D**, beside the technical section that validates
  the geometry ([React and react-three-fiber for the bench](#react-and-react-three-fiber-for-the-bench)).

Built: the physics, configurator and diagnostics of the key on its frame, the
technical section and solid view of the bench, the engine's first
application, and the interface it composes them through: every setting
addressed by a path, every operation run as an action, each edit validated
before any solve, logged and replayable to the same state ([The WebAssembly boundary](#the-webassembly-boundary), [The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)).
Planned: the rest, the state saved and reopened ([Persistence](#persistence)) among it.

### Uses

The applications in view, others to follow:

1. **Regulate, measure and train on a single key**, set up as a technician sets
   one up and read as he reads it. Regulating and measuring built for the key on
   its frame ([What is built: stage 1a, the key on its frame](#what-is-built-stage-1a-the-key-on-its-frame)); training planned ([Stages](#stages)).
2. **Understand the action's static and dynamic equilibria**: why a key rests
   where it does, why a weight between up weight and down weight moves nothing,
   what the jack does at let-off. Key statics built; dynamics planned.
3. **Discover the action**: parts in section and in solid, contacts visible,
   motion slowed until escapement can be watched. Section and solid of the key
   and frame, with the key's contacts marked on them, built; motion planned.
4. **Design an action.** Parts and frame are edited freely. What no part could
   be built as is refused with its reason, the limiting dimension and the
   shortfall in millimeters; what can be handled is handled, so a frame that
   does not fit its key is drawn as it is, its faults named in millimeters.
   Faults for the key and frame, and the validation layer that refuses a key
   no one could cut, built ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)).
5. **Play a score on 88 notes**, dampers and three pedals included. Planned.
6. **A game whose task is to regulate a piano.** Planned.

Out of scope: upright actions, a soundboard model, tuning, voicing. Whether
sound is in scope, and how, is open ([Open questions for the owner](#open-questions-for-the-owner)).

### The acceptance criterion

> **The simulator must measure its own regulation using the trade's own
> protocols, and come back with the expected values**: key height, key dip,
> blow distance, let-off, drop, aftertouch, checking, damper lift point, down
> weight, up weight, friction, action ratio.

If that holds, the simulator is an instrument; if not, an animation. Hence a
reported value is the protocol run on the model (a let-off is the
hammer-to-string distance at the instant the jack leaves the knuckle, not a
parameter read back), and where a displayed value is a formula instead, this
document says so ([The protocols, as they are](#the-protocols-as-they-are)). A protocol run on the model is a consistency check, not
a validation, which needs measurements taken independently of the model ([Validation](#validation)). A
simplification is judged against the protocol's resolution, the gram for
touchweights and a punching paper for key height and dip (the thinnest is
0.08 mm [V, `data/params/punching-papers.json`]); one that can move a reading by
that much is named where it is made. And the user sees the trade's quantities:
grams, millimeters, the trade's names, fractional inches planned as a second
unit system.

### What the model is

**Planar and single-note**: the sagittal section of one note. Adjustments out of
that plane or across notes (squaring and spacing keys, bushing play, glides,
traveling and burning shanks, hammer spacing, the una corda shift) are planned
as parameters that change what is observed, not as geometry. **String height
and keybed crown** form one vertical gap, whose sum is all that blow distance,
let-off and aftertouch see; both are planned. **Key frame bedding is out of
scope**: a deflecting frame couples neighboring notes, breaking the
independence the 88-note runtime rests on ([The 88-note runtime](#the-88-note-runtime)), and no published frame
compliance exists. Its main symptom, dip absorbed by something soft, stays
reachable through a softer front punching law; the bench is to state *perfectly
bedded frame* beside the settings it teaches (planned).

## What is built: stage 1a, the key on its frame

The bench opens on **Hirschkorn's key, note 52 of a Boston GP-178, on his
printed frame** ([Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52"): a key whose mass, center of mass, inertia,
contact points and bushings' friction were all measured and published on one
action. His printed points, lines and circles are read as printed; what his
tables give no number for, the key's front, covering and blocks, is traced
from his Figure A.1; his frame bears where his model bears, on real felts; his
keybed, rails, papers, pins and the key's width are borrowed from the owner's
measured section, or assumed there. Beside it, chosen on the bench, stands
**the owner's measured section**: his key section and capstan as he measured
them on his keys, on the key frame he measured, regulated on arrival, its key
front assumed where his balance pin stands along the key is not measured, and
**not weighed**: until one of its keys is weighed whole on the bench, every
reading that needs the key's mass is refused, saying so, and the others are
read ([The protocols, as they are](#the-protocols-as-they-are)). Each is a profile (`data/profiles/`), every number of which is a
record or a stated assumption ([Every number has its provenance](#every-number-has-its-provenance)), and every drawn line can be marked by how
its number is known ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)).

### Parts and statics

**The key**, `KeyDesign`, is a natural: a keystick of one wood thickness with
what is glued to it or cut from it (the covering; the key front over the end
grain, the covering's lip overhanging it; the key button, flat on top, its two
ends rounded to a radius, where the balance mortise is cut; the back check block; the thinned tail and its damper felt; the chamfer
that lets the tail bed flat on its cloth) [V, `data/params/key-section.json`,
`data/params/balance-mortise.json`].
The back check block stands where the action puts the hammer's tail, so it is
placed from the balance pin and not from the key's end: a key lengthened at the
back keeps its block in place, the keystick running at its full height between
the block and the thinned tail. On the measured section its back end stands
220 mm behind the pin, where the measured tail ends, a place derived and not
measured (stated with its rule in `data/profiles/measured-section.json`). The
capstan is a turned part, drawn as a solid of revolution, screwed into the
keystick's full height between the key button and the back check block,
nothing glued under it. A key may carry what the measured section lacks, each
a feature its description holds only where the key has it: a bare thinned
tail, no felt on it, running on under the back check block, which stands
wholly on the tail or wholly on the keystick's full height; a capstan head
turned to a dome, whose top bears as a circle does; a capsule standing off
the pin; and a clearance cut in the keystick's top between the capstan and
the back check block, not right behind the capstan, so that the wippen rail,
which runs across the keyboard over the keys, cannot rub the key, the key
coming back to its full height behind it. Fairly standard on a grand key,
though not on every one, the clearance is built as a feature any key may
have, placed by its two ends from the balance pin, as the back check block is
and for the same reason, the action placing the rail it clears, and by its
depth below the full top; it is a shape of the keystick's outline, drawn in
the section and in the solid and taken out of the drawn section whose
centroid estimates a weighed key's center-of-mass height ([Named simplifications](#named-simplifications)). Neither
shipped key has one drawn: the
measured section's waits on the owner's measurement of its place and depth,
and Hirschkorn's figure draws his key's top lowered from just behind the
capstan to the key's end instead, kept as drawn. The validation layer refuses
a block across the step down to the tail, a felted tail under a block, a
capstan over the thinned tail or across the step down to it, a dome narrower
than its head, and a clearance run into the capstan's seat, into a block or
back over the thinned tail, cut through the keystick, whose wood under it the
chamfer under the tail thins where it runs on under it, or no cut ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)),
each by how far. Hirschkorn's key has all of them but the clearance ([Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52"). No
lead is modeled: a key's leads are in its weighing (below). Each piece is a part of its own, with its own material, on the key's
body: the covering and the key front are of their own materials, the capsule and
the back check block of one wood that is not the keystick's, and the front
mortise is a cut, a void in the keystick open at its underside, milled from
below: a cutter turning about a vertical axis, moved along the key, so that
the mortise is as wide across the keyboard as the cutter, as long along the
key as the cutter's travel and diameter, its two ends rounded to the
cutter's radius. It is built as the owner describes it
(`data/params/key-section.json`, front_mortise_construction): milled through
the floor of a groove cut across the underside from one side of the key to
the other, as long along the key as the mortise's straight walls, so that its
rounded ends reach past the groove's walls; and on its straight walls, in
their lower part only, the rounded ends left bare, the bushing that guides the
key across the keyboard on the pin's oval head, an L on each side: its cloth
runs up the mortise's wall from the groove, then turns and is glued flat on
the groove's floor toward the key's side. The groove beside the mortise is a
cut and the bushing a lining, both beside the sagittal plane, and count in no
area of the section. Its depth and the groove's are measured, its cutter and
travel assumed [S, `data/params/key-section.json`], and its bushing assumed
by each profile [S, `data/profiles/measured-section.json`,
`data/profiles/hirschkorn.json`]. On other keys
the groove runs only a few millimeters either side of the mortise; the model
cuts it across the whole key, as on the owner's
([Named simplifications](#named-simplifications)).

**The balance mortise** is built as the owner describes it
(`data/params/balance-mortise.json`, construction): an oblong slot through the key
button along the key, milled as the front mortise is, by a cutter as wide as
the slot, its two ends rounded and its long sides straight walls between
them, open through the button's flat top, which it stays within; under it a
cavity in the keystick as long and as wide as the slot, down to a thin
floor; through that floor the round balance hole, milled by a cutter of its
diameter that is not moved, fitted to the pin, which holds the key fore and
aft; and on the
slot's straight walls, in their upper part only, the rounded ends left bare,
the bushing that guides the key across the keyboard, an L on each side: its
cloth runs down the slot's wall, then turns into a side hole cut through the
key button's wall along the straight wall, taller than the
cloth so that the cloth passes through it easily, and runs out to the key
button's side glued against the hole's top, the hole open under it. The
slot, the cavity and the hole are cuts, taken out of the drawn section whose
centroid estimates a weighed key's center-of-mass height; the side holes are
cuts too, and the bushing a lining, both beside the sagittal plane, and count
in no area of the section. Every one of its dimensions is a setting or follows
from one, and none is measured: each profile assumes them with its reasons. The balance pin is
straight, its tip rounded, to its own radius as the model assumes.

**The key is weighed whole**, as a key that arrives leaded is weighed, its leads
in it and their masses unknown: its mass on a scale, and its front weight on
Stanwood's jig ([The protocols, as they are](#the-protocols-as-they-are)) with the distance back from the covering's lip it was read
at, typed on the bench in grams and millimeters. The bench keeps the mass and
the static moment about the balance pin, the front weight times the arm it was
read on, with that reading as its datum, as it keeps a friction as a torque with
the radius it was read at ([The protocols, as they are](#the-protocols-as-they-are)): moving the measuring point afterwards, or
editing the key, reads the same moment on another arm, and the front weight the
jig reads follows by the lever. The moment over the key's weight stands its
center of mass along the key. Its height, which no scale gives, is estimated
from the key's drawn section: the area centroid of its parts as the section
draws them, the front mortise, the balance mortise and its hole, and the
clearance in the top, where the key has one, taken out, and the turned capstan,
and the bushings, the key button's side holes and the front mortise's groove
beside the sagittal plane, left out: where
the center of mass of a key of one material and one width would stand ([Named simplifications](#named-simplifications)).
Every reading it reaches names it ([The protocols, as they are](#the-protocols-as-they-are)): it acts only through the key's tilt
on its frame, about 0.02 g of down weight per millimeter on a key of 120 g
standing 2.3° nose-up at its top seat, as the measured section does. A key
nobody has weighed has no mass in the model at all: no volume and no density
stands in for one, and what reads the key's mass is refused until it is
weighed. A weighing no key could have is refused with its reason, the bench
left as it was: a mass of nothing, a front weight read off the covering, a
center of mass the two readings put off the key. What one more lead would do
somewhere along the key is not computed. A key a source weighed and located
whole, as Hirschkorn's Table A.1 gives his key's mass, its center of mass in
the key's frame and its inertia, is read as published, the height of its center
of mass included, and nothing of it is estimated; weighed again on the bench,
it keeps that published height.

**The frame**, `FrameDesign`, is three different rails: the balance rail with
its crown and round pin, the front rail with its oval pin, the back rail with
its cloth. A punching is a felt washer plus papers from a drawer, the owner's
of eight thicknesses, 0.08 to 1.25 mm, each known by its color [V,
`data/params/key-frame-sections.json`, `punching-papers.json`]; the drawer is
data the profile binds, and each paper is drawn in the color its record
gives.

**The key stands on its balance punching, free to lift off it.** A key is not
pinned to its frame: its balance punching can push it up and cannot hold it
down, and the balance pin only guides it. So the key on its frame hangs from a
slide up the pin's axis, without mass or friction, and turns about the slide's
origin in its bushings; the balance punching is a contact like the others. Where
the punching carries the key, the key turns about the punching's center exactly
as a key on a pivot there would. A key whose front stop and back rail cloth hold
it higher than its punching does rides on them, lifted off the punching, on the
parts as drawn; a pull the punching cannot answer lifts the key off it. The
slide's range ends where the key leaves its balance pin: where its balance
hole, at the key frame's origin on the pin's axis, clears the pin's drawn top,
read off the drawn pin, which stands above the key button, 33 mm above the
punching on both shipped frames. A pull or
stops that would take the key past it lift it off its pin and its frame, where
the model has no equilibrium to give, and so does a punching stacked above the
pin's top, which has the key off its pin wherever it stands on it. Everything
else, where the key rests and every reading, is the static solve below.

**The frame is placed under the key once, then left alone.** Building a frame
puts the front rail under the key's front mortise and the back rail a set
distance in from the tail; the positions are then the frame's own coordinates.
Lengthen the key and the rails stay put, the mismatch reported as faults in
millimeters past the limit (pin outside its mortise, punching off the flat of
the underside, cloth outside the chamfer); placing the frame again is a
deliberate act.

**The general mechanism, its first part.** Bodies in a tree, the ground first,
each hung from its parent by a pin or a slide with one coordinate, an angle or a
length; poses by one forward pass, whose only transcendentals are one sine and
one cosine per pin, through `libm` ([Determinism, stated correctly](#determinism-stated-correctly)); the Jacobian column of any material
point for any coordinate, by the lever-arm identity of [The lever arm, and the gradient of a gap](#the-lever-arm-and-the-gradient-of-a-gap) extended to every
ancestor joint; gravity at each body's center of mass and forces fixed in the
world, with their potential and generalized forces; and coordinates prescribed
for one procedure, with the reaction that holds each. The mechanism never
computes a body's mass; it is handed one, the key's as a source publishes it or
as it was weighed, the height of a weighed key's center of mass estimated from
its drawn section; a key nobody has weighed is handed none, and the procedures
that need its weight refuse. Its first client is the jig ([The protocols, as they are](#the-protocols-as-they-are)). Each coordinate carries its
limits, an interval it cannot leave, and its joint's dry friction, a torque
bound that holds it still against any generalized force inside it; the static
solve below finds the coordinates a procedure leaves free. The key on its frame
has two: the height of its slide and its angle.

**Contacts, read off the drawn parts.** Each of the key's contacts with its
frame is a pair of ranges of drawn parts and nothing else: the front punching's
top and the balance punching's against the underside's flat; the back rail
cloth's face, its rounded edges and the sides they lead into, against the whole
underside, chamfer included; the front pin's tip against the top of its mortise,
and against the wood round the slot: the underside beside it, which a pin
standing outside its mortise but under the key reaches; the corners of the wood
at the slot's mouth, which a pin straddling the slot's edge meets; and the
slot's walls along the key, which the pin's rounded tip approaches as the key
turns, the slot moving along the pin by the pin's height above the bearing times
the turn. Where a surface runs across parts, as the underside runs from the
keystick onto the key front, the joint is no corner. A mechanism carrying the
parts reads every pair at any pose into rows: one for each pair of features that
meet (a disc, which is an arc or a convex corner, against a straight face; two
discs; a face's center against a face), each a signed gap with its normal and
its contact point, and over every coordinate the gap's gradient by the lever-arm
identity of [The lever arm, and the gradient of a gap](#the-lever-arm-and-the-gradient-of-a-gap) and the sliding of the two surfaces past each other. A row's
gap keeps its sign however deep it is read, though a corner driven past the
center of an edge it overlaps can have no row at all ([Seventeen contacts per note](#seventeen-contacts-per-note)). A row deeper than
the row margin is kept only where its point stands inside the other side's drawn
material, the shallowest of a corner's, which is the face it comes out through;
a face's center gives one only where no other side of the material stands nearer
it by more than the margin, the face being then the way it came in, which it is
not for a punching's center a hair past the key front once the key turns. Both
punchings are read at the center of their drawn tops, a rigid reading of a felt
washer that spreads the load of a key landing level over its whole face; a key
standing out of level on one has the washer's drawn edge in its underside by
what the felt would compress, half the face's width times the tilt, which the
bench reports beside the contacts. How the key bears on its cloth (on the
cloth's edge; flat on it, its two edges within 2.5 µm of closing, about a
thirtieth of the thinnest paper; or on the key's own corner), where, and the
slope of the key's face over the cloth's are read off those rows, and so are the
two gaps on the sheet and the contacts the view marks.

**The static solve.** One load increment finds the coordinates a procedure
leaves free: those at which the loads' potential, plus each joint's friction
bound times how far its coordinate moves from where the increment starts, is
least, every contact row and every limit kept open or closed. It is solved by
proximal sequential steps, each linearizing the rows, accepted on a merit that
charges every overlap, and bounded so that no stop is stepped over: a declared
pair of surfaces within the 1 mm row margin of each other approaches by at most
that margin in one step, and a pair farther apart stops at least a margin short
of meeting.
Each step's multipliers solve a box problem, a contact's in [0, ∞), a friction's
in [−b, b], by projected Gauss–Seidel sweeps finished by an exact active-set
pass; the split of a load that rigid stops leave indeterminate is the one of
least utilization, one admissible split among many, and where no motion
satisfies the rows the stops that fight are named. A step is accepted when it
lowers the merit, or raises it by no more than its rounding, eight units of
`ε` in the size of its terms: a key loaded a hair past its breakaway starts
with steps whose energy is a hundredth of the rounding of a merit of heights
times weights, and compared bit for bit, rounding alone would turn each back
and the key never leave its seat. It stops when a step moves
no point by more than a picometer, no gap overlaps by more than a picometer,
every row carrying more than the residual's tolerance of the loads is closed to
within a picometer, and the stationarity residual is negligible against the
loads, under a cap of 100 steps and of 200 sweeps of each box problem, its
counts reported ([Determinism, stated correctly](#determinism-stated-correctly)). The closed rows are part of the test because the
residual alone cannot see them: the box problem accepts its multipliers to a
part in 10⁹ of their scale, and under a metric many accepted steps have
softened, that accepts rows no motion can close together as closed while they
stand microns open, their loads balancing the key to the rounding. A contact
pushes only where it touches, so such a state is no equilibrium, and a step
that moves nothing from it is turned back, which stiffens the metric until the
rows close or let go. The residual is read with each axis's friction taking
what its set allows:
anything within its bound on an axis that has not moved since the increment
started, the bound against the motion on one that has. The box problem places
a multiplier within a part in 10⁹ of its bound on the bound, and an axis held
by a friction that much short of its bound would otherwise carry the
difference as a force out of balance, at a state that is an equilibrium.
Where the step and the gaps pass and the step's own residual does not, the
residual is read again with every gap already closed to within that picometer
taken as closed: closing what rounding leaves of a gap, a few hundredths of a
femtometer, through a row whose arm is a centimeter, moves the key's farthest
point fifteen times as far, and the force of that motion is no force out of
balance.

**The key is solved on its punching first.** While the punching carries the key,
the key turns about the punching's center, which stands on the slide's axis. So
an increment of a key standing on its punching is first solved with the slide
held there and every other contact read as it is: the key on a pivot at the
punching. Where the slide's reaction then pushes the key up, or is nothing, that
equilibrium meets every condition of the key free to lift, the punching's
multiplier being that reaction, and it is the pivot's to the bit, so a key
bearing on its punching reads what it reads on a pivot. A seat started from the
key lowered onto its frame from above, where no attitude on its punching clears
the frame, is solved free from the start, and lands where the pivot's would
within the solve's tolerance, not to the bit. Where the punching would have to
pull the key down, the punching is released and the key solved free to
lift; so too where the stops fight with the key on its punching, unless the
start stands sunk in the frame deeper than the row margin, which no row reads
truthfully: the refusal then stands. That depth is read off the drawn outlines,
the key against every part of its frame, the punchings' felts and papers whole,
since a key sunk right through a felt thicker than itself leaves the punching's
center, which is all its pair reads, clear. A key turning on a closed stop moves
its farthest point a row margin a step at most, and a key pulled off its
punching turns up on its cloth until its friction holds it or it leaves its
balance pin, 33 mm above the punching and some 70 mm at its front on the
measured section's frame; a key riding on its front punching, its front pin's
tip against the wall of its slot, solved from where it is lowered onto its
frame, takes three quarters of a cap to come to rest with every row that carries
it closed. So the key free to
lift is solved in up to four pieces of the cap, each going on from where the
last stopped, and reported as one increment.

**The seated pose**, from which key height and dip are read, seats the key by
the trade's gestures, friction off: a load on the capstan's contact point
standing for the action, which holds the back of the key down on its cloth, and
for the bottom a press on the measuring point besides. Each load seats the key
with a margin of 0.01 N·m over what holds it the other way, and with rigid stops
the pose depends on neither. The top seat starts from the key's nominal pose: on
its balance punching, the attitude nearest level, level itself first and then a
degree at a time, at which its drawn parts overlap the frame's, rails and keybed
included, by no more than the row margin, an overlap the solve then resolves
through its rows. Where no attitude on its punching clears, its stops fighting
or a pin standing in its wood, the key is lowered onto its frame from above
instead: at each attitude the least lift that clears, found by halving the
slide's range to within the margin, and of those level, unless another clears a
margin lower, since lifts closer than the margin are told apart by where the
halving's steps fall and not by the frame; so the key starts within a margin or
two of the stops it will rest on. A key riding on two stops
clears them only near the attitude it rides at, each whole degree either side of
it costing the angle between them times the stops' arms of lift, up to a
millimeter or two; so where no attitude a degree apart clears below the top of
the slide's range, the attitude is narrowed, a degree either side of the one
that overlaps least there, by 22 golden-section steps toward the attitude of
least overlap. An overlap within the margin is not always one a motion of the
key resolves: a front pin a few tenths of a millimeter into the wood beside its
slot, well up in the slot, stands its rounded tip in the slot's wall along the
key, which a turn draws back only through the wall's short arm about the bearing
and no lift short of the pin's top clears, so that from there the stops leave
the key no motion, or the solve reaches its cap of steps. Where the top seat
finds no seat from the nominal pose, a stop standing in the key, a solve that
reaches its cap with a stop standing in the key, or the key carried off its pin
from a start at the top of the slide's range, the key is lowered onto its frame
from above, clear of everything, as a technician puts a key on a frame it does
not go onto, and it lands on what stands highest, the stalled steps counted with
the seat's; where it finds none from there either, the first refusal stands.
The key is lowered only through its slide's range, so a key that clears its
frame only past where it leaves its balance pin has no pose on its pin to start
from: where no attitude, a whole degree or narrowed, stands within the row
margin of clearing its frame below the pin's top, its stops would hold it past
its pin and it is off its frame. Clear of everything, the felts whole, a key
tilted on a punching clears the felt's drawn edge only half the washer's width
times the tilt above where it rides on its center: a key riding that close under
its pin's top that has no seat from its nominal pose finds none from above
either, as on a balance pin 9 mm above its punching under a front punching
27 mm thicker than the bench's, the key riding 4.6° nose-up on it within 0.4 mm
of the pin's top, refused with the punching standing in it. The bottom seat starts from the top. Key height is the height of
the key front's top at the top seat, and the dip how far it falls
to the bottom seat, so the dip ends on whichever stop binds first: the front
punching, the pin bottoming in its mortise, or the pin meeting the wood round
its slot. A key whose stops fight seats riding on them, off its balance
punching, at the top and at the bottom alike: its key height is read where it
rides, and its dip is nothing. A seat whose drawn parts stay sunk in one another
deeper than a micron, where no row reads them, is refused with the stops sunk;
so is a bottom seat on the angle limit, reached when the punching's center
stands off the underside, and a seat whose stops would hold the key past where
it leaves its balance pin, or whose punching stands above the pin's top, off its
frame; the bench says why. The seats read nothing the touch weight has done.

**Regulation is solved from targets, by the seated pose itself.** Key height and
key dip go in, and the paper on each punching comes out the way a technician
finds it: first the paper under the balance punching that stands the key front's
top at the height asked, with no paper on the front punching, by 40 halvings of
20 mm of paper, laid from the drawer as the closest total from below; then, with
the key as that paper leaves it, the paper on the front punching that gives the
dip asked, by the same bisection, laid the same way. Each stack keeps its felt.
A bisection needs only the order, a stack at which the key rides on its stops,
off its balance punching, counting as one at which they fight: while the stops
do not fight, key height rises with the balance stack, and the dip does not grow
with the front stack and stops changing once the front pin bottoms in its
mortise or meets the wood beside its slot. A stack that stands the key off its
balance pin, a balance punching raised past the pin's top or a front stack that
makes the key ride past where it leaves the pin, is no seat: for key height it
counts as one past the target, more paper only raising the punching further,
and for the dip as one at which the stops fight. So the solve sees whatever the
seats see. The papers laid are read again by the seated pose, and the
regulation lands within one paper of its targets: the key height at or under its
target, the dip at or past it, each by less than the thinnest paper on that
stack moves it. A target no stack reaches is refused and the bench left as it
was, naming what keeps it out and by how many millimeters: the front pin
bottoming in its mortise or meeting the wood beside it, a felt thicker than the
whole stack the target asks for, stops that fight, nothing under the front of
the key, the balance pin, which the key leaves before the paper raises it that
high, a rail too low for 20 mm of paper to reach. A key still riding on its
stops with all the paper the search lays rides at every stack, at the height its
stops set: a key height under that is kept out by the stops, one over it by the
rail, or by the pin where more paper would raise the punching past the pin's top
before it reached the key. A key off its pin with no paper at all has no seat at
any stack. A closed form taking the key top as level would miss by the key top's
height times the square of the attitude, about 0.02 mm of key height and dip on
the measured section, a quarter of the thinnest paper, and would not see the
pin.

**The touch weight** is one static increment from where the key stands, friction
on. Loading the bench puts the key on its frame from its nominal pose under the
weight, friction off, or from above where that finds no state, as the seats do,
in a state that needs no friction to stay; every later action, a weight, a
paper, a weighing, any edit, is an increment from the state the last one left; a
key standing on its balance punching rises and falls with it as papers are laid
under it. Between the up weight and the down weight the key stays where it is,
on either stop. A weight that lifts the key off its frame finds no equilibrium:
the key stays drawn where the last one left it, faded and bearing on nothing,
and the bench says the weight
lifts it off, whether the increment finds it so or, the increment failing, the
loading rule does. A key that no equilibrium has held since it was put on its
frame, for instance one its frame lifts off its pin, has no place to be drawn: it is drawn
faded, level on its balance punching, to show its parts against the frame's,
and only the contacts where its drawn parts overlap the frame's are marked,
none as bearing. An edit that leaves the key sunk deeper than the row margin
in its frame, read as the nominal pose reads it, its rails and the keybed
included, or from which no increment can be solved, puts it back by the loading
rule instead; the bench's application layer reports that it did, and the page
does not show it. Where the loading rule finds no pose on the key's pin within
the row margin of clearing its frame, as the seats do, the key is off its
frame, and the bench says so. What holds the key is read from determinate
quantities only, never from how the solve split a load: with every stop removed,
the bearing kept, the loads' generalized force either pushes the key into a
closed stop past its friction, which it is then *resting on*, or lies inside the
friction's band, *held by friction*, touching a stop or not, with the margin in
grams at the measuring point to the edge past which the key moves: on its cloth,
the down weight the sheet reads, which is walked from there; on its front stop,
the up weight; touching neither, the nearer edge. Friction is set-valued: any
torque within ±b holds the key. A key off its balance punching is read by its
lift: on both its stops where they fight, *riding* on them; otherwise on what it
touches, each stop closed where it stands named, held there by them alone where
pushes along their rows, none pulling, balance the loads with no friction, and
with its friction where they cannot: its front punching and its front pin
against the wall of its slot, alone, where a front rail raised under it has
tipped it forward off its punching; its cloth and its friction where a pull has
turned it up on it, the cloth pushing at one place and the pull turning the key
about it. Where a weight leaves a key depends on the key's own balance, which only a
weighing gives, so a key nobody has weighed gets no verdict for a weight on
it: it is drawn where its top seat puts it, its back held
down on its cloth by the load standing for the action, as key height is read,
whatever weight stands on it, and the bench gives no verdict.

### The protocols, as they are

Every reading on the weighing sheet is a protocol run on the model, and comes
back as a measurement: its value in full; what the trade reads it as, by the
protocol's own rounding; its unit and resolution; its conditions (where the
key was read, the stops holding the seat it was read at or walked from, the
loads that seated it, the walk, the temporary weight); its trust; the counts
of the static solves it took; and the fingerprint of the solver that took it
([Determinism, stated correctly](#determinism-stated-correctly)). Or it is refused with its reason, and the others are read all the
same. `pa-cli weigh` prints each with its procedure, its trust and its counts.
Their closed forms are test oracles, compiled for the domain's tests only.

- **Front weight and key weight ratio** are Stanwood's jig run on the model
  [V, Stanwood 2000; `data/params/touchweight-protocol.json`]. The key hangs
  from the jig by a pin at its balance pin's axis, a mechanism of [Parts and statics](#parts-and-statics) whose
  pin turns without friction: Stanwood's own tooling fastens the pin to a
  wedge that turns on a roller bearing with the key, so the key's bushings do
  not rub [V, Stanwood 2000, p. 21]. Its angle is held level, the covering's
  top horizontal where his bubble level sits. Holding it there takes a
  reaction on that angle, and the scale, standing under the measuring point,
  reads that reaction over g times the measuring point's arm: signed, so a
  back-heavy key reads negative, as his protocol for a negative front weight
  reads it. The ratio is the reading tared with the key on the jig, then with
  10 g centered on the capstan's contact point, over the load with its sign
  turned. Front weight is read to the nearest 0.1 g, the ratio to 0.01. The
  closed forms, static moment over arm and capstan arm over arm, are test
  oracles the jig matches to round-off. A jig pivots the key at the pin, so it
  cannot see where the key bears on its balance punching; only a reading on
  the frame can ([Validation instruments](#validation-instruments), "One real key, measured by the owner").
- **Key height and dip** are the seated pose run on the model ([Parts and statics](#parts-and-statics)): the back
  of the key held down at the capstan, then the front pressed down until a stop
  takes it, the two read at the key front's top. They see whichever stop binds
  first: a front pin bottoming in its mortise meets the mortise's top on its
  rounded tip, a little in front of its top as the key tilts nose-down onto
  it, and a front pin standing in the key's wood, outside its mortise or a
  few tenths of a millimeter past the front of its slot, lifts the key off its
  balance punching, the key riding on the pin and its cloth, its key height
  read there and its dip nothing. Read as found, the thinnest paper their
  resolution.
- **D and U** are the breakaway walk. The key is seated, at the top for D and
  at the bottom for U; on its measuring point goes the weight at which its
  balance there needs no friction, then a gram more at a time for D, a gram
  less for U, each a static increment from where the last one left the key,
  friction on, until one carries the key to the other seat; the last gram is
  then halved sixteen times, each trial from where that gram started. An
  increment that moves the key and leaves it short of the other seat is kept,
  and the walk goes on from there. So D is the least weight that starts the
  key down from rest and keeps it going to the bottom of its stroke, the
  greatest the down stroke needs anywhere, and U the greatest that lets it
  rise to the top: the window is the stroke between the two seats while the
  action has no let-off button (planned: the trade stops the stroke where the
  jack's tender meets the button). A walk is refused after 500 steps. On rigid
  stops the key breaks away at its seat and runs on, so D and U are the edges
  of the band at the two seats' attitudes, `(±b − M(θ)) / (g a(θ))`, the
  key's balance `M` and the measuring point's arm `a` taken at the attitude,
  matched by hand to the walk's resolution, a part in 65 536 of a gram. On
  Hirschkorn's key D is −21.61 g, which the closed form taken with the key
  level gives as −21.78 g: at the top seat the key stands 1.63° nose-up, its
  center of mass 15.8 mm above the pin, and the measuring point's arm is
  0.4 % shorter there. A key riding on its stops has no stroke: D and U and both
  key friction weights are refused, the bench naming the stops it rides on and
  how far above its balance punching, and BW and F with them. A weight that
  would lift the key off its frame refuses the walk that put it there.
- **BW and F** are `(D + U) / 2` and `(D − U) / 2`, shown from D and U as the
  trade reads them, so to the half gram, their values from D and U in full.
  With no action on the capstan, BW is minus the front weight but for what the
  two seats' attitudes add, 0.06 g on Hirschkorn's key, −27.22 g against his
  front weight's 27.28 g; F is the bushings' friction read through the two
  seats' arms, with what the key's balance does between them: 5.61 g on his
  bushings, Table A.9's 0.0122 N·m, which hold 5.50 g at the measuring point
  with the key level.
- **The band** is the driven sweep, a second procedure for D and U and the
  force curve the bench draws: the static form of driving in position and
  comparing forces ([What a test proves; drive in position, compare forces](#what-a-test-proves-drive-in-position-compare-forces)). The key is seated at the top and at the bottom of
  its stroke; from the top seat its angle is held where the measuring point
  stands 0.05 mm lower, then twice that, through every whole step strictly
  inside the window between the two seats, each step solved from where the
  last left the key, its lift off its balance punching free; then back from
  the bottom seat through the same travels. The angle a travel is held at is
  exact, the measuring point turning on a circle about the key's bearing. At
  each step the angle's reaction, read through the measuring point's arm at
  that step, is the force a finger there would apply to hold the key: with
  the bushings' friction at its bound against the motion, the down curve on
  the way down and the up curve on the way back; between the two the key
  stays where it is, their distance apart its friction and their middle its
  balance. Its extremes, the greatest force of the down curve and the least
  of the up curve, stand within a few thousandths of a gram of the walk's D
  and U in full, and are shown in full, to the hundredth of a gram, never as
  a gram and never as the sheet's D and U. Within that of a whole gram the
  two fall either side of it: the measured section weighed at 130 g with a
  front weight of 40 g at 13 mm reads its U −46.00019 g by the walk, −47 g
  on the sheet, and its up curve's least force is −45.99997 g. The gram is
  the walk's, the trade's protocol with gram weights; the band is a curve
  that no weight reads. A torque on the angle and a
  force at the measuring point turn the key alike and do not press it on its
  punching alike, so at every step the punching's load under the finger's
  force is checked, and a sweep under which it would carry nothing, a pull
  past the key's weight, is refused with where and how hard. At a seat the
  stop takes whatever the finger leaves, so the curves start a step inside
  each seat, and where an extreme lies at a seat, as on both shipped keys,
  the band reads it within the curve's change over that step of the walk's:
  on Hirschkorn's key the greatest down force −21.6099 g against the walk's
  D −21.6085 g a step off the top seat, the least up force −32.8309 g
  against its U −32.8319 g within a step of the bottom one, 184 readings
  each way. Every step is the band by hand at the
  attitude it holds, `(±b − M(θ)) / (g a(θ))`, to rounding. The step is one
  of the solver's frozen numbers ([Determinism, stated correctly](#determinism-stated-correctly)). The sweep runs when the band is
  shown, never with the sheet; `pa-cli band` prints it.
- **Read as gram weights give them**, the owner's reading: D and the key
  friction weights rounded up to the gram, U down, BW and F from those two
  readings, the value in full shown small beside each. A technician stacking
  gram weights reads a down weight whose threshold is 46.3 g as 47 g, since
  46 g does not move the key; rounded to the nearest gram, a down weight would
  read a gram light about half the time, a weight that does not make the key
  go down. On Hirschkorn's key D reads −21 g: the bare key, leaded for its
  action, goes down on its own, and a pull of 21 g does not hold it up. A
  threshold within the walk's last bracket of a whole gram, a part in 65 536
  of it, reads as that gram, up or down: the walk reads a threshold to half a
  bracket, and what it cannot tell from a whole gram is not rounded into the
  next. Rounding noise is not a gram: on bushings with no friction the key
  friction weight zeroed on the frame, a threshold of nothing walked half a
  bracket up, reads 0 g.
- **Key friction weight** is read two ways, labeled. Stanwood's zeroes the
  front weight on the leveled jig with a temporary weight on the key's top,
  puts the key back on its frame, its back pushed down onto its cloth, and
  walks the least weight that makes it fall [V, Stanwood 2000, p. 22]. The
  variant zeroes instead the key's balance where it rests on its frame, and so
  reads the bushings alone, through the arm the measuring point has at rest.
  The weight stands 100 mm behind the pin on a key heavy in front and 150 mm
  in front of it, on the covering, on one heavy behind, its mass the one that
  zeroes, exactly, the reading being linear in it: a technician slides a
  weight he has, the bench sizes one where it stands. On Hirschkorn's
  bushings, which hold 5.50 g at the measuring point with the key level, the
  two read 5.94 g and 5.52 g, both 6 g: the tilt at rest leaves Stanwood's a
  little of the key's and the weight's balance, 0.43 g with no friction at
  all, and shortens the arm both read through.
- **Key friction is typed as the trade measures it**, a key friction weight in
  grams at the measuring point by Stanwood's protocol, and stored as a torque
  with the measuring point it was typed at. The torque is the one at which
  that protocol, run on the model, reads the typed grams in full: the protocol
  inverted, by 32 halvings of the bushings from none to those holding 100 g at
  the measuring point with the key level, each trial the protocol itself, the
  one of the last two whose reading stands nearer kept. So the sheet reads a
  typed key friction weight back to the walk's resolution, and shows it as
  typed: 6 g typed reads 6 g. Turned into a torque on the level arm instead,
  the grams typed would read back heavier by the key's tilt at rest, as his
  bushings, 5.50 g on the level arm, read 5.94 g. A weight no bushings read is
  refused with its reason, the bushings left as they were: under what the
  protocol reads with no friction at all, past the search, or on a key and
  frame the protocol cannot read. Afterwards moving the measuring point or
  editing the key changes the grams every protocol reads, and not the
  bushings. The shipped bushings are not typed: Hirschkorn's are Table A.9's
  torque, the measured section's an assumed one, 6 g at the measuring point
  with the key level.
- **Trust.** Each procedure declares what it reads, the key's mass and static
  moment, the height of its center of mass, its friction, the dimensions of key
  and frame, and a reading resting on an estimated or assumed input is
  indicative and names it. Front weight reads the key level, and so not the
  height of its center of mass; D, U, BW, F and Stanwood's key friction weight
  read the key tilted on its frame, and that height with it. Key height, dip
  and the key weight ratio read no mass, nor does the key friction weight
  zeroed on the frame, whose temporary weight cancels the key's balance where
  it rests: on a key nobody has weighed these are read, and front weight, D,
  U, BW, F, Stanwood's key friction weight and the band are refused, the key
  not weighed. On Hirschkorn's key every reading is indicative for its
  geometry alone, his mass, center of mass and bushings being published and
  his keybed, rails, felts and papers borrowed or assumed ([Named simplifications](#named-simplifications), [Every number has its provenance](#every-number-has-its-provenance)). On the
  measured section a key weighed on the bench has its mass and static moment
  set by hand, and the height of its center of mass is the estimate from its
  drawn section, which D, U, BW, F and Stanwood's key friction weight name;
  its friction is assumed until it is set, and its key front has no record.
- **Datums.** Key height and dip are read at the key front, the face of the
  plate over the end grain, 237.80 mm in front of the balance pin on
  Hirschkorn's key and 258 mm, assumed, on the measured section. Every
  touchweight is read at the measuring point, 13 mm by default ([The trade's protocols are the acceptance test](#the-trades-protocols-are-the-acceptance-test)) back
  from the covering's lip, the front edge of the keytop, which stands 1.48 mm
  proud of the key front on his key and 2 mm on the measured section: 226.28
  and 247 mm in front of the balance pin. The distance is
  one setting, `TouchweightProtocol`, held beside the key and its frame, set on
  the bench from 5 to 30 mm, and every touchweight reads it there: front weight
  and the key weight ratio on the jig, D, U, BW and F, both key friction
  weights, and the weight on the measuring point. Moving it changes each by the
  lever, the same static moment and the same friction torque read on another
  arm, and leaves key height and dip where they are.

### Named simplifications

- **A key's mass is weighed, never derived from its outline.** A volume model
  cannot predict a front weight read to the gram: a grand keystick is far
  narrower behind its head than in front, its covering is not wood, its
  hollows and leads are where its plan puts them, several grams of doubt in
  all. So a key's mass is weighed, per key, by the trade's protocol, each key
  being cut to its own plan shape: built, its mass and its front weight, which
  give its static moment about the balance pin ([Parts and statics](#parts-and-statics)); planned, for the
  dynamics, its inertia. The three are independent: a key carrying 16 g of
  front weight in leads alone has 8,000, 5,700 or 2,920 g·cm² about its
  balance point depending on where the leads sit [V, Nakamura, *Practical
  application of moment of inertia*, Part 2, Fig. 9]. The trade computes
  inertia from a traced template and wood densities [V, Nakamura, Part 1] or
  rates it from static weights [V, Stanwood, SNAP, 2023], and measures it
  nowhere; Hirschkorn swung each part as a pendulum on a string,
  I = m·g·l·(T/2π)² [V, Hirschkorn 2004 §A.1], which a string between support
  and part biases unless the part hangs from a knife edge. Inertia is
  therefore neither measured nor shown: it is an internal parameter of the
  dynamics, each part's taken from Hirschkorn's published action and the
  key's estimated from its outline matched to its mass and front weight,
  marked estimated (planned). The one number of a weighed key no scale gives,
  the height of its center of mass, is estimated from the area centroid of
  its drawn section, the front mortise, the balance mortise and its hole, and
  any clearance in its top taken out, and what stands beside the sagittal
  plane left out (built), which treats the key as one
  material of one width, each void taken out across the whole of it: a covering denser than the wood raises the
  true center, and leads set low in the keystick lower it, each millimeter
  moving D by about 0.02 g on a key of 120 g ([Parts and statics](#parts-and-statics)). No lead is modeled:
  a key arrives leaded, with lead masses nobody knows, and is weighed whole.
- **The default key is Hirschkorn's**, note 52 of a Boston GP-178, on his
  printed frame ([Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52"). Published, with their tables: his key's printed
  points, lines and circles, its mass, center of mass and inertia (Table
  A.1), its bushings' friction (Table A.9), his frame's bearings and his dip
  target. Traced from his Figure A.1 [E, 11]: the key front, 237.80 mm in
  front of the balance pin, the covering's lip 1.48 mm proud of it, the key
  button, where the keystick's full height ends behind the capstan, the back
  check block, the tail's end. Borrowed
  from the owner's measured section, measured there or assumed there: his
  keybed, which no source gives, so that his key height is an indication;
  his rails, felts, cloth, papers and pins, and the front mortise, placed
  over his front bearing. Assumed: the key's width, which the solid view
  extrudes the key by, the mortises are milled no wider than and the front
  mortise's groove runs across, and no reading depends on. His felts' tops stand where his rounds' tops stand, each placed by a
  rule from his frame stated in his profile. Beside it, chosen on the bench,
  **the owner's measured section**: his key section, his front mortise's
  depth and his capstan, 122 mm behind the balance pin, as he measured them
  on his keys [V, `data/params/key-section.json`], on the key frame he
  measured [V, `data/params/key-frame-sections.json`], its rails placed under
  the key by the rule a frame is built to and their heights assumed, its
  felts, cloth, capstan proportions, bushings and targets assumed, its key
  front assumed 258 mm in front of the pin and its tail following from its
  measured length of 500 mm, and its mass unknown until one of its keys is
  weighed: what would replace each assumption is stated beside it in
  `data/profiles/measured-section.json`. With his capstan and the assumed key
  front its key weight ratio is 0.494, a hundredth under the 0.50 to 0.60
  published for a natural: where his key front stands is the measurement that
  settles it.
- **The balance punching bears at its center, on the pin's axis.** The key
  bears on a felt washer whose pressure center moves with attitude and load;
  the model reads it rigidly, at the center of the washer's drawn top. Moving
  the bearing behind the pin by cutting the punchings drops the action ratio by
  three to four tenths [V, Stanwood 2014, "Cutting the Balance Rail Punching to
  Effect Ratio"], and the model has no cut punching. Planned: a cut punching
  drawn as the washer it is, its face's center behind the pin, read where it is
  drawn; and a compliant rocking contact.
- **The linings are rigid**: drawn at their thickness, acting as rigid stops.
  Planned with the felt laws of the quasi-static core.
- **Both punchings bear at their center.** Each contact reads the center of
  the felt's drawn top, a rigid stand-in for a washer that spreads the load of
  a key landing level over its whole face. A key standing out of level on one
  overlaps the drawn felt's edge by what a real felt compresses, half the
  washer's width times the tilt: about a quarter of a millimeter under the
  measured section's key at its top seat on its balance punching. The bench reports it beside the
  contacts, so that the idealization does not pass for a drawing error. The
  front punching reads the line of the underside across the front mortise's
  groove, on which the key bears either side of it: a groove too long for the
  punching to reach past it beside the mortise, which the punching would
  stand in, is refused.
- **The key lifts without friction.** The bushings' friction against the key
  sliding along its balance pin is not modeled: it would act only while the
  key leaves its balance punching, a state the bench reports and the sheet
  never weighs. The bushings' friction acts on the key's turning alone, as the
  key friction weight reads it. Where the key leaves its pin is read off the
  drawn pin, whose height, 35.2 mm above its rail, 2.5 mm above the key
  button as the owner's sketch draws it, is assumed until it is measured
  (`data/profiles/measured-section.json`).
- **A front pin's sides carry no contact.** The pin declares its rounded tip,
  which is all a pin standing in its slot can meet. One a few tenths of a
  millimeter into the wood behind its slot meets the wood's corner at the
  slot's mouth with its side as the key comes down, which no
  contact reads: with little paper under the balance punching the key has no
  seat, refused with the pin sunk in it, and with some more the bottom seat
  can reach the static solve's cap.
- **The rails carry no contact.** The key's contacts are its punchings, its
  cloth and its front pin; the rails they stand on are drawn, and read for
  overlap only by the key's nominal pose, where a solve starts. A key riding
  nose-up by 2.7° or more on a front rail raised with its punching meets the
  rail's back edge, 62 mm behind the punching and a stack's height below it on
  the measured section's frame, which the model does not see; so does a key turned far
  nose-up on its balance punching, as a strong pull turns a key on a raised
  balance rail or a lowered cloth, its underside meeting the top of the
  balance rail behind the pin.
- **What the validation layer does not see**: the rails against one another,
  so a front rail moved back onto the balance rail is drawn through it; and
  the capstan's thread against its hole, so a capstan screwed up past its
  shaft's length, within the 2 to 30 mm the bench admits, is drawn standing
  above the wood ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)).
- **Hirschkorn's key is drawn square** where his figure draws its blocks coved
  and its covering beveled, each traced to its feet [E, 11]; its key button's
  ends are rounded to the 11.64 mm radius traced from where his figure's
  flat top ends [E, 11], and its balance mortise is assumed as the measured
  section's is; its back check, wire and head, belongs to the
  action and is not drawn, though Table A.1's mass includes it. His front
  mortise is the owner's, placed over his front bearing. Placing the frame
  under his key follows the rule the measured section's frame is built by, the
  back rail 40 mm in from the tail [S], which stands his back rail in front of
  his 37 mm chamfer.
- **Drawing**: one row of pins; the front rail pin's tip a half sphere of half
  the pin's length along the key, squeezed across to the oval, assumed
  (`data/params/key-frame-sections.json`); a flat balance crown only. The
  balance mortise's dimensions are assumed, none measured, and so are the
  front mortise's cutter, travel and bushing
  [S, E, `data/profiles/measured-section.json`, `data/profiles/hirschkorn.json`].
  Its slot runs along the model's
  `x`, the key's front axis, on a straight key: a cranked key, bent beside its
  covering, keeps its slot along that axis and not along its own edges there,
  and is planned with the keys side by side. No contact reads the slot's ends
  or either mortise's bushing, so the balance pin is drawn in its slot without
  guiding the key, and nothing reads how the balance hole and the bushings fit
  their pins: a tight or an eased mortise moves no reading, the bushings'
  friction being typed as a key friction weight. The front mortise's groove is
  cut across the whole key, as on the owner's: a groove only a few
  millimeters either side of the mortise, as other keys have, is drawn across
  the key all the same. The solid carves the key button's side holes and the
  front mortise's groove from the key's wood, each a rectangle swept across
  its width, and the two mortises
  and the balance hole, each milled with its ends rounded to its cutter's
  radius, by leaving the wood unpainted inside each cut and drawing the
  cut's walls from inside rather than by solid geometry operations, which
  would bring a library for a few boxes with rounded ends.

### The bench

**Built.** A browser page in React and react-three-fiber draws the key on its
frame in section and in solid from parts computed in Rust, each in its own
body's frame and placed by the pose the static solve gives that body. The
section is the sagittal plane itself: a washer shows the hole its pin stands in,
the front pin stands in its mortise, a void open at the key's underside, and
the balance pin in the balance hole and the cavity over it, each mortise's
bushing on its far side seen beyond the plane, behind the pin, the near one
cut away with the near half of the key. The solid carves the two mortises and
the balance hole, milled, their ends rounded, the key button's side holes and
the front mortise's groove out of the key's wood and draws their walls, so
the slot opens through the key button's top, the bushing on its straight
walls and the pin standing in it, the balance mortise's cloth shows flush on
the key button's sides, against the top of each side hole, and the front
mortise opens under the key around its pin, in the groove across the key,
its cloth turned flat on the groove's floor.
Both views mark the key's contacts with its frame where the contact rows find
them: where the key bears, a gap of under a millimeter still open, and drawn
parts that overlap. A key no equilibrium places is drawn faded, a note on the
view saying why, and the reader can draw any key as faint as wanted, to see
the pins inside it, and its two mortises' bushings apart from the rest of it,
whole at first, so that the cloth shows inside the faded wood, or as faint as
wanted where it stands in front of a pin. The solid draws its faded parts apart,
by weighted blended order-independent transparency (McGuire and Bavoil, in
[Sources](#sources)): three's own transparency blends each part over what is
drawn in the order of their middles' distance from the camera, so the key
button, glued on the keystick in another wood, would change color all at once as
the camera turns past the angle that swaps them. Every face of a faded part
stays drawn, its mortises' walls included, in colors that no angle changes where
the browser draws into images of half-precision colors; where it cannot, the
solid is drawn as three sorts it. Faded parts that overlap are drawn in the mean
of their colors, weighted by their opacities, whatever their order, so a nearly
opaque key shows its front face mixed with what stands behind it: the mean is
what no order changes. The edges of the parts drawn whole are drawn with them,
under the faded parts, so a faded key dims the frame's edges behind it as it
dims its faces. Colors follow each part's material: the covering and the
key front white, the capsule and the back check block in a wood other than the
keystick's. It opens on Hirschkorn's key; its first panel chooses the
instrument on the bench among those shipped, each offered by its whole name.
Its panels: the weighing sheet, with the measuring point's distance
back from the front lip of the key, each reading as the trade reads it with its
value in full small beside it, both key friction weights labeled, a note naming
what makes the readings indicative, and why any is refused, a frame whose stops
fight drawn riding on them, off its balance punching, with why its weights are
refused, and a frame that cannot seat its key still showing every setting and its
sheet, the jig's readings read and the rest refused line by line with why;
regulation (key height and dip targets, laid as papers or refused with what keeps
them out and by how much, and not sent while a target field holds no number; the
punching papers; the key friction weight, typed in grams at the measuring point
and shown as Stanwood's protocol reads it, to the gram with its value in full
beside it, refused with why where no bushings read it and not offered where the
protocol cannot be read); punchings and cloth (felts, front rail pin and mortise,
balance rail pin, back rail cloth); the key, and the key weighed whole (its mass and its front
weight typed in grams, with the distance back from the lip the front weight was
read at in millimeters, what each is for said beside them, and refused with why
where no key could weigh so; on a key nobody has weighed, the line under them
says so, the fields of the mass and the front weight open empty and the
weighing waits for them, the key mass on the sheet is refused, and every
reading that needs it is refused, saying the key is not weighed); the frame
(rails, placing the frame); touch (a
weight on the measuring point; what holds the key: a stop or its friction with
how far the weight stands from moving it, its two stops where they fight with how
far above its balance punching it rides, the stops it touches where it stands
lifted off its punching otherwise and its friction where they need it, or
nothing, a key off its pin and its frame; the gaps at the front punching and the
back rail cloth; the bushings' friction, named once, as the key friction weight
Stanwood's protocol reads; how far the punchings' drawn edges overlap the key;
and, on a key nobody has weighed, why a weight on it does nothing). The panels
and their controls are the ones the engine's registry
describes, in its order, each control's range, step and unit the engine's
([The WebAssembly boundary](#the-webassembly-boundary)); the page adds the readings and the notes around them. Every slider
shows the value the engine holds, the key friction weight's as the sheet reads
it and every other as the settings hold it, and keeps no copy of it; a slider
moved where no key could be built is refused, the slider back where the
setting stands and the page saying why right under it, with the limiting
dimension and the shortfall ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)); the target and weighing fields hold what
is typed until it is sent.
In place of the section and the solid the view draws the band ([The protocols, as they are](#the-protocols-as-they-are)): force
on the measuring point in grams against the key's travel there in
millimeters, the down curve and the up curve, the two seats at the window's
ends, the down curve's greatest force and the up curve's least marked where
they fall and labeled in full, to the hundredth of a gram, never as the
sheet's D and U, which read the gram ([The protocols, as they are](#the-protocols-as-they-are)), the weight on the measuring
point drawn across it and a dot where that weight has left the key; the
pointer reads both curves at the travel nearest it, and for a reader without
the chart the two extremes are written out and a table carries every step.
The engine sweeps the key when the band is chosen and not with each edit:
an edit of the key, its frame, its bushings or the protocol leaves the band
drawn faded and saying so until it is chosen again, and the weight on the
measuring point, which the band does not read, moves only its line and dot. A
key with no stroke, or no mass, has no band, and the view says why.
Faults are listed in millimeters, every number on the page is computed in
Rust, and every sentence is the catalog's of the language the page is read in,
English or French, composed from the codes the engine sends
([A bilingual interface](#a-bilingual-interface)).
`pa-cli weigh` prints every reading of Hirschkorn's key, or of the shipped
profile named, with its procedure, its trust and the static solves it took;
`pa-cli band` prints the band, its two extremes in full beside the walk's D
and U, and both curves step by step.

## Decisions that stand

Each decision is in force; its reason is given, and the alternative it rules
out where that matters.

### The trade's protocols are the acceptance test

The simulator reports what a technician's procedure reads when run on the model
([The acceptance criterion](#the-acceptance-criterion)), so the person who uses the tool can judge it; ruled out are probes chosen
for convenience, a parameter read back or an internal angle. The protocols are
Stanwood's *Standard Protocols of the New Touchweight Metrology* (Piano
Technicians Journal, February 2000), which supersede those of his 1996 paper:
front weight leveled with a bubble level, the key weight ratio read with 10 g at
the capstan, a negative front weight measured by its own protocol, and Key
Friction Weight and Support Spring Balance Weight defined [V, Stanwood 2000,
pp. 21–22]. With every weight centered on the measuring point, 13 mm or 1/2 inch
back from "the front lip of the key" [V, Stanwood 2000, glossary, p. 23]:

```
BW + FW = (WW × KR) + (SW × SR)
BW = (D + U) / 2        F = (D − U) / 2
```

**The lip is the covering's**: the front edge of the keytop, which overhangs
the key front by 1.48 mm on Hirschkorn's key and 2 mm on the owner's measured
section. The glossary does not say which lip
it means, and this one is the owner's choice; the 1996 paper's "front vertical
edge of the key" goes with the protocols the 2000 ones supersede. Measured from
the key front, the point would stand 2 mm further back on the measured section
and every touchweight would be read on an arm 0.8 % shorter. **The distance is a setting**, 13 mm by
default and the glossary's 1/2 inch as its alternative
[`data/params/touchweight-protocol.json`], held in one place because every
touchweight protocol reads it, so no two can be taken at different points
([The protocols, as they are](#the-protocols-as-they-are)). Key height and key dip keep the key front as their datum.

### Staged by sub-assembly, with the view at every stage

The action is built one sub-assembly at a time, engine and view together, along
the terms of Stanwood's equation: key, wippen, hammer ([Stages](#stages)). A discrepancy is
then attributable (a whole action's down weight has dozens of candidate causes,
one key's front weight a handful); the decomposition is the trade's own, each
term a sub-assembly and a measurement; and the owner validates by looking, so
with one rigid body checkable by hand a disagreement between drawing and numbers
is a pipeline bug. Ruled out: the whole action's statics, then dynamics, then
rendering, which cannot attribute a discrepancy and withholds the owner's only
channel of validation. Geometry comes from the owner's measurements, asked for
before a part is drawn; the drawing catches what slips through and does not
replace measuring.

### Upstream settings drift; faults are reported, not refused

Change the key dip and the aftertouch keeps its old value, out of specification;
shorten a key and its front rail pin stands outside its mortise by a named
distance. An action whose upstream setting changed is out of regulation until
someone redoes the downstream one, and a designer pushing a geometry until it
breaks needs to know where: both are what the tool teaches. Ruled out:
cascading, which hides the lesson, and refusing every state out of
specification, which forbids design. The limit is honesty: what cannot be
computed or drawn truthfully is refused, never clamped, with its reason, the
limiting dimension and the shortfall, before any solve ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)). Built for the
key and its frame ([Parts and statics](#parts-and-statics), [The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)); planned for regulation settings, as badges.

### Regulation as targets; the coordinate is normative

Regulation is stated in the trade's targets (key height and dip, then blow
distance, let-off, drop, checking) and the geometry (punching stacks, capstan,
buttons and screws) is solved to meet them. A saved state's normative value is
the coordinate; the target travels beside it as intent, with the value achieved
and a fingerprint of the solver, because re-solving with another solver could
silently change every number downstream. Training turns an actuator and reads
the result; the solver aims at a target. Built: key height and dip solved to
punching stacks by the seated pose itself, laid from the drawer within one
paper of their targets, a target a stop forbids refused with the stop and the
shortfall ([Parts and statics](#parts-and-statics)); the solver's fingerprint, which every reading carries
([Determinism, stated correctly](#determinism-stated-correctly)). Planned: the other targets, actuators with their quanta (a paper, a
turn of a screw), persistence ([Persistence](#persistence)).

### Set-valued dry friction, at the axes and at sliding contacts

Friction at a pivot is a set, any torque within [−b, b] while the pivot is
still; never a `sign()` or a `tanh()`. The interval between down weight and up
weight *is* that friction, F = (D − U)/2, a threshold measured quasi-statically.
A regularized law has zero friction at zero speed, so a static solve with it
collapses D onto U: the threshold test "cannot be simulated with a regularized
friction law" [V, Thorin 2017 §2.5]. Dynamically, with a threshold speed of 10⁻⁶
rad/s, it keeps the threshold, creeping under a micrometer in ten seconds under
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
slides ([Friction](#friction)). That set is what the engine uses, solved with the axes' in one
box-constrained problem at velocity level. Built: the key's balance friction,
set-valued, a box row of the static solve, stored as a torque with the radius
it was typed at, typed as a key friction weight in grams at the measuring
point, the torque at which Stanwood's protocol reads it back ([The protocols, as they are](#the-protocols-as-they-are)).
Planned: every pivot and the four sliding contacts, in every solver.

### Relative coordinates for the jack and the repetition lever

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
A child pinned at **r** on the wippen, center of mass at **u** from its pin,
turns at θ̇_w + q̇ while its center of mass moves at
θ̇_w ẑ × (**r** + **u**) + q̇ ẑ × **u**: its coupling to the wippen is
I_cm + m(|**u**|² + **r**·**u**), its share of the wippen's diagonal
I_cm + m|**r** + **u**|². **Do not use** I_cm + m(**r**·**u**) and
I_cm + m|**r**|²: they fit neither relative nor absolute coordinates, and a
positivity test passes on them anyway.

### The two key ratio conventions; ratios are outputs

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
ratio through those and the knuckle's distance from the shank center. The
trade's definitions of the action ratio (lever product, a gauge read at the top
of the stroke, Stanwood's strike ratio, the ratio of useful travels) differ on
one action; each is to be reported, labeled (planned). Built: the key weight
ratio is computed from the arms and has no setter.

### Every number has its provenance

Every value the engine starts from has a record in `data/params/` with its unit,
source and reliability marker (`V`, `E` or `S`), or is an assumption a profile
states with its reason, and a derived value has its script in
`tools/derivations/` or its rule written beside it. The owner's measurements are
primary sources, marked `V` with the instrument and the date. A value whose
source is unpublished is provenanced; one whose source is lost is not. Built:
records for the key section, the key frame, the papers and their colors, the
touchweight protocol's measuring point and key-ratio load, Hirschkorn's action,
and Stanwood's weight curves; and the profiles (`data/profiles/`), compiled
into `pa-app`, which bind every number an instrument starts from, every field
of its key, frame, drawer, protocol and bushings, its targets and the weight on
it, to a record by a JSON pointer, its unit checked against the number's
quantity and converted to SI rounding once, or to an assumption marked `S`, or
`E` with its rule, and its reason. The engine has no default of its own: the
standard instrument is Hirschkorn's profile, and every value without a record
is stated in the profile that uses it. `every_default_has_a_record_or_a_stated_assumption` fails on
a number of a shipped profile that is neither, the numbers taken from the
validation layer's walk of every field, which the compiler holds exhaustive.
Hirschkorn's profile reads his record and the outline traced from his
Figure A.1 ([Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52"), each traced number `[E, 11]`. A drawn stretch of a part
is placed by one number of the design, which the part names; how it is known,
measured, published, traced, derived, borrowed, assumed, or set on the bench
since the instrument was built, is read from the instrument's profile, and the
view marks every line by it when asked ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)). A profile names the
instruments it describes as its records name them, so a number it reads from
a record of another instrument, as Hirschkorn's reads the owner's key frame's
pins and punchings, is marked borrowed rather than measured.

### A dedicated planar solver, not a general-purpose engine

Six coordinates and seventeen contacts per note ([Seventeen contacts per note](#seventeen-contacts-per-note)) make a small multibody
problem; the work is in contact geometry, material laws, friction and
protocols, and three things a general-purpose engine lacks decide it.
(1) **Per-contact nonlinear felt laws with memory**: what the finger feels and
when escapement happens come from felt and leather compressing, each interface
with its own law of penetration, rate and history, while game engines expose a
scene-wide numerical softness (Rapier's default: a 30 Hz spring, damping ratio
10 [V, Rapier source]) and accept no force law per contact. (2) **Exact analytic
lever arms at sub-millimeter scale, in double precision** ([The lever arm, and the gradient of a gap](#the-lever-arm-and-the-gradient-of-a-gap)), where a
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
the knuckle ([The time step](#the-time-step)). **MuJoCo** has joint friction loss, polynomial springs,
reduced coordinates, double precision and an official WebAssembly build, but its
friction loss is a soft constraint, so a joint creeps under a sub-threshold
load: an optional oracle for cross-checks (planned), not a dependency.

### Rust compiled to WebAssembly; notes on independent workers

**Built.** The domain is Rust, compiled to WebAssembly for the bench and
natively for `pa-cli` and the tests, for reasons that are not speed: arithmetic
reproducible across machines (ECMAScript leaves every transcendental but `sqrt`
implementation-approximated, and engines differ; Rust with the `libm` crate
does not, [Determinism, stated correctly](#determinism-stated-correctly)); no garbage collector in a fixed-step loop; and native runs of
the code that ships, for studies, sweeps and golden tests. **WebAssembly threads:
no.** On the pinned Rust 1.99.0, linking `pa-wasm` with shared memory fails
(`rust-lld: error: --shared-memory is disallowed by std-… because it was not
compiled with 'atomics' or 'bulk-memory' features`), and rebuilding the standard
library needs nightly; reproduce it on the `cdylib`, since a library crate does
not link and passes with the same flags. The planned 88-note runtime is
therefore one module per Web Worker ([The 88-note runtime](#the-88-note-runtime)).

### React and react-three-fiber for the bench

**Built.** React, with three.js through react-three-fiber and drei, covers a 3D
scene inside a dense interface of panels and readouts; `three`, its types,
react-three-fiber and drei are pinned together through pnpm's catalog. Labels
and values sit in HTML over the canvas, never as 3D text. Ruled out: Babylon.js,
with a better inspector and bundled types but no equivalent of the React
ecosystem for the interface. The canvas is drawn on demand, when a prop, the
camera or its size changes, so a still view costs nothing. Planned: linings
drawn compressed by the solver's penetration, dimensions on the section, one
instanced mesh per part family for 88 notes, and the realistic rendering of
[An engine for piano applications](#an-engine-for-piano-applications): materials and detailed shapes
as skins riding on the parametric solids without replacing them, so the
realistic view cannot drift from the physics any more than the section can.

### A bilingual interface

English and French: the primary user works in French and the vocabulary is the
point; contributors read English; wording built in Rust could not be translated.
**Built:** Rust sends codes and numbers across the boundary, never sentences
([The WebAssembly boundary](#the-webassembly-boundary)), and the page composes every sentence it shows from one catalog per
language (`apps/bench/src/i18n/`), each family of codes typed from the generated
types, so that a code the engine gains and a catalog does not word fails the
type-check. Emphasis is marked in the wording itself, so a translation carries
it with its words. Both catalogs are built: English, and French in the
owner's trade vocabulary (glossary:
[A3 > Bilingual glossary FR ↔ EN (+ DE) and code identifiers](research/A3-nomenclature-and-regulation.md#bilingual-glossary-fr--en--de-and-code-identifiers)), its names beside Stanwood's English
symbols as the glossary keeps them, its numbers written with a decimal
comma. The page opens in the first of the browser's languages it
speaks, English where it speaks none, and a selector changes it. The catalog
chosen reaches every component by context, so a language chosen rewords the
page and asks the engine nothing.

## What the dynamics prototype settles, and what it leaves open

**Built**, and throwaway: a one-note dynamics prototype on Hirschkorn's Boston
GP-178, note 52, in Python, reading his record and assembled with the pivots
read correctly ([Hirschkorn's frames](#hirschkorns-frames)), in `tools/dynamics-prototype/`. Its level-0 checks pass,
and its `RESULTS.md` holds the step-convergence and solver study, generated by
`study.py`: four integrators, four scenarios (a slow weighing-like press, a
forte and a piano press under position drive, a forte driven by force, each with
its release), fixed steps from 0.25 to 500 µs. Every result in it is a
consistency check ([What a test proves; drive in position, compare forces](#what-a-test-proves-drive-in-position-compare-forces)): it says how the model behaves under its integrators,
not that the model is right. [The time step](#the-time-step), [One dynamic solver](#one-dynamic-solver) and [The 88-note budget](#the-88-note-budget) are settled by it; [The felt law](#the-felt-law) and [Rigid bodies](#rigid-bodies)
remain hypotheses.

### The time step

**Settled: one fixed step of 50 µs, no sub-steps.** With the solver of [One dynamic solver](#one-dynamic-solver),
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
linearized mode near 1.2 kHz with the capstan, the front punching and the
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

### One dynamic solver

**Settled: explicit on the contact laws, with both frictions as one set.** One
dynamic solver serves the bench and the keyboard: semi-implicit Euler on the
compliant contact laws, and the axis friction and the four sliding contacts'
friction ([Set-valued dry friction, at the axes and at sliding contacts](#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)) as box-bounded unknowns at velocity level, solved together by a
projected Gauss–Seidel sweep finished by an exact active-set pass, at the fixed
step of [The time step](#the-time-step) [E, prototype]. **Rejected on accuracy: compliant contacts treated
implicitly by backward Euler.** It is stable at every step tried, but it damps
every contact mode: the back check catches the hammer 25 µs late per
microsecond of step, and it holds the observables only to 2 µs where the
explicit solver holds them to 50 µs or more [E, prototype]. A trapezoidal step
was not tried; it would matter for a much stiffer felt law. The quasi-static
solver is a separate static solve on the same contacts and friction sets,
inertia dropped, solved to convergence under an iteration cap (built for the
key on its frame, [Parts and statics](#parts-and-statics)). With compliant contacts
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

### The felt law

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
allowed**: the `libm` crate makes `pow` as reproducible as `sin` ([Determinism, stated correctly](#determinism-stated-correctly)), and the
law is chosen on physics and measured cost.

**Do not use "no published rate-independent felt model"**: Masoudi and McPhee
publish one. **Do not use "a factor of 56"** between Hirschkorn's and Thorin's
jack/knuckle stiffness: it is the ratio of the cubic coefficients, 3.92e11
against 7e9; at equal force, 10 to 100 N, the tangent stiffnesses differ by 3.1
to 3.6 [E, 05]. The two laws come from different actions, benches and law forms,
and enter the data as two labeled material sets, never merged.

### Determinism, stated correctly

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
sine and cosine of every pose go through it ([Parts and statics](#parts-and-statics)); Clippy refuses in `pa-core`
every `f64` method IEEE-754 does not round exactly, `sin` to `hypot`, `powi`
included (`crates/pa-core/clippy.toml`), so no transcendental in the domain
reaches the platform's math library, and a test that wants one for a reference
value allows it explicitly. `pa-app` still calls the platform's `atan` for one
displayed angle, the bedding at the back. No `HashMap` in the domain's
production code; three bisections, the regulation solve's 40 halvings of 20 mm
of paper, the breakaway walk's 16 halvings of its last gram and the 32 halvings
of the bushings up to 100 g at the measuring point that turn a typed key
friction weight into a torque, resolve far below anything read, and a walk's
threshold within its last bracket of a whole gram reads as that gram; the static
solve converges under its caps and reports its counts, a key free to lift in up
to four pieces of the step cap, each going on from where the last stopped,
counted as one increment ([Parts and statics](#parts-and-statics)); and a test holds the seats of every key of its
property corpus a keyboard could be built from, and of every frame whose stops
fight, to a quarter of the step cap, and the walks' increments to half of it: a
trial a hair past breakaway starts the key with a step of a femtometer, each
accepted step three times the last. A fingerprint names the solver with its
frozen numbers and travels with every reading, one name for one hash, never
given back to other numbers (`statics/3`); a test hashes the caps,
tolerances, margins and counts, the free key's pieces, the temporary weight's
stations and the searches' reaches, and fails when one changes and the
fingerprint does not; where the key leaves its balance pin is read off the
drawn pin, a dimension and not a frozen number, and the key weight ratio's
load is the protocol's, bound by the instrument's profile to its record. The golden
sheet ([Crates, apps and gates](#crates-apps-and-gates)) holds the engine's readings bit for bit wherever the tests run, and
the WebAssembly build to the same bits as the native one.

### Rigid bodies

**Hypothesis.** The key and the hammer are rigid. Thorin's criterion, a part's
first bending mode against the duration of the event, holds for the key (first
mode 355 Hz) and fails for the hammer (39 Hz, (f·τ)⁻¹ = 5.1) [V, Thorin 2017
§2.1 and Appendix], and before let-off, the phase that matters for touch, a
flexible shank sits in series with the drive. **Test.** The prototype with and
without a degree of freedom between head and shank, compared on key force under
position drive; untested so far, the prototype having no head–shank coordinate
and the record no shank stiffness. **Do not use 280 Hz** for the hammer's first mode, nor the
180 µs derived from it as a twentieth of its period: the figure is in no source.

### The 88-note budget

**Hypothesis.** One dynamic solver, notes computed independently on a few
workers, plays 88 notes in real time on a laptop: a note is active for a few
hundred milliseconds and static in its hold, so far fewer than 88 move at once.
**Counted:** about 2,400 floating-point operations a step with the solver of
[One dynamic solver](#one-dynamic-solver), plus 15 transcendental calls: 49 million per simulated second for a note
at 50 µs, 4.3 billion if all 88 notes moved at once [E, prototype], which they
rarely do. **Still to measure:** the cost per step natively and in Chrome and
Safari, and the notes active at once over a corpus of MIDI files, pedal changes
included.
**Do not use** the per-step costs quoted in annex A1: the benchmark kernel
behind them is not in the repository.

## Validation

### What a test proves; drive in position, compare forces

A **consistency check** shows that the model agrees with itself or with
mathematics: a round trip, a closed form against a solver, a derivative against
a finite difference, a conservation law. It catches coding errors, and a wrong
model passes it. A **validation test** predicts outputs measured independently
from inputs measured independently, on the same instrument; only validation says
the model is right. A validation test says so; every other test is a consistency
check, as the whole stage-1a suite is. The first validation test is [Validation instruments](#validation-instruments), "One real key, measured by the owner".

**Drive in position, compare forces.** The action is dominated by inertia until
escapement: driven by a force and judged on displacements, a deliberately wrong
model, its felts made rigid (on a real action, Thorin notes, the hammer would
probably not even reach the string), scores 9 % error against 3 % for the
complete model [V, Thorin 2017 §5, after Thorin, Boutillon and Lozada 2014]. So
the dynamics are validated by driving the key in position and comparing the
reaction force at the finger, the curve a technician reads, and the **Bad-Model
guard** belongs in the suite: stiffen the felts, and if validation still passes,
the validation is worthless. Planned for the dynamics; its static form is built,
the band ([The protocols, as they are](#the-protocols-as-they-are)), which drives the key through its stroke in position and reads
the force that holds it there.

### Validation instruments

**One real key, measured by the owner** (planned). Its mass and its front
weight by the trade's protocol are inputs ([Named simplifications](#named-simplifications)); its key weight ratio on the
jig and its down and up weights on its frame are outputs, the bench's measuring
point set to where he weighs before anything is weighed ([The trade's protocols are the acceptance test](#the-trades-protocols-are-the-acceptance-test)). Its entry
point is built: the owner's measured section ([Named simplifications](#named-simplifications)), which arrives unweighed,
takes the three numbers of one of its keys weighed whole, its mass, its front
weight by Stanwood's protocol and how far back from the covering's lip the
weight was centered, and reads the rest; with the distance from the balance
pin to its key front, which its profile assumes, measured on the same key. It is a modest
validation: the ratio is close to a lever identity, and a bare key's balance
weight differs from minus its front weight only by what the bearing point and
the rest attitude add. But it is the first comparison of computed readings with
independent ones, and it exercises what stage 1a builds: arms, datum, bearing,
protocols.

**Hirschkorn's Boston GP-178, note 52**: geometry, masses,
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

Built: his key on his printed frame, the bench's standard instrument
(`data/profiles/hirschkorn.json`). Its printed points, lines and circles are
read as printed: the underside through Pk4, the chamfer along Pk5's line,
meeting the flat at −196.43 mm, the capstan's dome Pk3's circle of 9.7 mm, the
bearings at Pg1, Pg4 and Pg5. What his tables give no number for, the key
front, the covering and its lip, the key button, the keystick at its full
height to 132.45 mm behind the pin and lowered from there into a bare thinned
tail the back check block stands on, the tail's end and the capstan's head,
is traced from his Figure A.1 [E, 11]: a
similarity fitted on Pg1, Pk2 and Pk5 leaves 0.27 mm at Pg1, and puts the key
front 237.80 mm and the covering's lip 239.28 mm in front of the pin, 10.0 mm
ahead of Pk2. The figure draws the full height's top over its last 29.65 mm,
behind a line drawn down the wood, 0.75 mm above the keystick's: within the
trace's uncertainty, it is read as the keystick's one height, and his capstan
screws into it, its top Pk3's whatever its seat. Moved along the key, his
capstan stands anywhere from 42.01 mm behind the pin, where its head clears
the key button, to 128.53 mm, where it reaches the thinned tail; the slider,
from 60 mm by the millimeter, takes it to 128 mm. The
figure's center-of-mass marker stands 13.7 mm behind Table A.1's center, about
7 g of front weight: Table A.1 governs the mass. His bushings are Table A.9's
0.0122 N·m, read at no published radius; his dip target 10.0 mm; no key
height is published, so the key height he is regulated to is the one his key
stands at. His frame bears on **real felts**, a flat front punching washer over
Pg4 and a flat back rail cloth strip, each with its top where his round of 5 mm
has its, since what is drawn is what is touched; its keybed, rails, papers,
pins and the key's width are those of the owner's measured section, so his
key height is an indication. The borrowed 24 mm cloth lies wholly under his chamfer, its
rounded front edge over Pg5, so the key, nose-up at rest, bears on that edge,
0.06 mm from where his round meets his chamfer, as most grand keys rest on an
edge of their cloth: it is the papers under the balance punching that set a
key's height, and the cloth's work is to damp the key's return and give it
the same rest every time; a key bedded flat on its cloth would only wear it
more slowly. The felts reproduce his printed dip to two micrometers: at Pk2
his rounds let the key fall 9.306 mm and the felts 9.308 mm, 1.7 µm more,
because a line resting on a round passes over its center above its top, by
0.39 µm on his rounds of 5 mm, the chamfer and the flat standing 0.7 degrees
from level, by 0.04 µm on the cloth's edge of 0.5 mm, and not at all on the
punching read at its center; and because script 10 reads Pk5's normal as
printed, 0.99984 long, which stands the chamfer 0.8 µm further off his round
[E, 11]. A cross-implementation test closes that difference from his printed
points and the cloth's edge radius alone, never from the bench's frame, and
holds the bench to it, against the rigid closure of script 10 to its last
printed digit and the dynamics prototype's compliant contacts within their
compression (`the_bench_differs_from_his_rounds_by_the_felts_alone`), so a
rail read a micrometer off, in height or along the key, fails it; and its
front weight to Table A.1's static moment: consistency, not validation, every
input being his. On his frame as printed the bench reads FW 27.3 g, KR 0.52,
a key height of 61.66 mm, indicative, and a dip of 9.72 mm at the key front,
D −21 g (−21.61), U −33 g (−32.83), BW −27.0 g, F 6.0 g (5.61) and both key
friction weights 6 g: the bare key is leaded for its action, rests nose-down on
its front punching under no weight, and falls on its own. His dip of 10.0 mm
is one regulation away. Nothing on the key validates
until his action's own readings are compared (stage 2).

**Thorin's Renner demonstrator** (planned): the only published key force
under position drive, and escapement velocities measured and simulated, 1.01 and
1.00 m/s piano, 1.56 and 1.56 m/s forte [V, Thorin 2017, Table 5]. It was
adjusted by a professional technician and carries no key leads [V, Thorin 2013
§1.6]: do not read its heavy touchweights as wear. Its parameters have lineage
traps ([Parameter lineage](#parameter-lineage)).

### Levels

| Level | What it checks | Kind | Status |
|---|---|---|---|
| 0 | the mechanism's Jacobian columns against central differences of its poses, and its generalized forces against the gradient of its potential; every contact row's gradient and sliding against central differences, at depths past any radius, its sign against the side of its drawn piece; the contact rows against a brute-force sampling of the drawn outlines; every contact surface a range of a drawn part, and every row's points on the drawn outlines wherever the key stands, riding on stops that fight and lifted off its punching included; the box problem against an exhaustive enumeration of its active sets; the static solve against closed forms (a pendulum on its stop, a lever breaking away at its friction band, two levers through an arc in the ratio of their arms); the unilateral virtual-work inequality at every settled state; the seated pose against a brute-force reading of the drawn outlines over a property corpus, frames whose stops fight among them, the key riding on its stops with no dip, and the breakaway walk over the same corpus, D above U by the band its friction makes; a front pin a few tenths into the wood beside its slot, the key lowered onto it from above; the key on its balance punching against the key on a pivot there, the same bits wherever the punching carries it; a pull past what the punching carries, by hand, lifting the key off its frame and the bench saying so; the key ratio guard; the solver's fingerprint against a hash of its frozen numbers | consistency | built |
| 0 | the mass matrix against finite differences of the kinetic energy; normals continuous across the hybrid profiles; every coordinate acted on by a contact or a spring | consistency | planned |
| 1 | step convergence: one scenario at several steps ([The time step](#the-time-step)) | consistency | planned |
| 2 | energy in free flight, contacts and friction off: bounded for the hammer alone; for the (wippen, jack, lever) block, whose mass matrix depends on its configuration, a drift falling as O(dt) | consistency | planned |
| 3 | published data, [Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52" and "Thorin's Renner demonstrator" | validation | planned |
| 4 | the trade's protocols run on the model: the jig, reading a key weighed whole back where its front weight was read and by the lever elsewhere, the seated pose that reads key height and dip, the breakaway walk that reads D and U and both key friction weights, against the band at the seats' attitudes by hand and the level closed form within its derived tilt, the driven sweep that draws the band, every step against the band by hand at the attitude it holds and its extremes against the walk's D and U in full within the walk's resolution and the curve's change over one step, a step half as long within the same, on both shipped keys, the measured section weighed by a test weighing and in whole grams, a key friction weight typed read back by Stanwood's protocol and its bushings against that band by hand, and the regulation that lays papers to key height and dip targets, against an exact inverse of the key on a pivot and with the small-angle placement held within the terms it drops; every reading of the golden sheet the same bits natively and in WebAssembly | consistency | built |
| 4 | let-off, drop, checking, aftertouch and blow distance as the trade measures them; [Validation instruments](#validation-instruments), "One real key, measured by the owner" | consistency, and validation where the measurement is independent | planned |
| 5 | sensitivities by finite differences: what moves when a setting changes | consistency | planned |

**Non-confusion tests** (built, through the procedures and through the whole
chain in the smoke test): 1 g of mass set on the key, a test weight standing on
it while it is seated and walked, moves BW and not F, and the smoke test runs it
as the trade would, the key re-weighed with its front weight a gram heavier;
axis friction times 1.2 moves F and not BW. With a constant friction bound both
hold but for the tilt between the key's two seats, where D and U are read: a few
thousandths of a gram of F for the gram of mass, a few hundredths of BW for the
friction, each tolerance derived from the two seats' arms and the walk's
resolution, never widened. They guard against coding mix-ups until friction
depends on load. Both are run through the band too: the gram moves its middle
at every travel by its moment there over the measuring point's arm, and its
width by nothing; its extremes' middle moves by the mean of that at the two
attitudes they are read at, and their width by the difference, a few
thousandths of a gram; tighter bushings widen it at every travel by the extra
bound over the measuring point's arm and leave its middle, its extremes' middle
moving by half the difference of that extra at their two attitudes, under a
hundredth of a gram.

**Sensitivities.** Spurlock's rule, +1 g of hammer weight for +5 to +7 g on both
down weight and up weight [V, Spurlock], checks the lever chain's consistency:
it is the strike ratio. **Do not use "+0.1 mm of capstan ⇒ −1.4 g of down weight
and −0.3 mm of aftertouch"**: it comes from a display mock-up, and raising the
capstan increases aftertouch. **Do not read Thorin's ΔJ/J as a ranking of
sensitivities**: it is the gap between the cost at a parameter's measured value
and its minimum, its large values (up to 42 %) due by his account to a
mis-setting of his demonstrator [V, Thorin 2013 §6.5.3]; a validation target is
a property of the physics or of a protocol, never one instrument's regulation
error. Nor are settings optimized by gradient descent: Thorin finds his cost
function convex along a few parameters and flat along many, and concludes that
"common methods of convex optimization should probably not be applied to the
piano action" with it [V, Thorin 2013 §6.5.6]. A scripted regulation sequence
or a grid search does the job.

## Physics notes for the next stages

### Springs inside the wippen assembly

**The lemma.** A massless spring pivoted on one of the wippen, the jack and the
repetition lever, and bearing on two of them, has exactly zero generalized force
on the wippen's coordinate of [Relative coordinates for the jack and the repetition lever](#relative-coordinates-for-the-jack-and-the-repetition-lever): turning the wippen with the jack and lever
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

### The lever arm, and the gradient of a gap

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
contact point, a coordinate that moves both bodies canceling exactly ([Parts and statics](#parts-and-statics));
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

### Parameter lineage

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
- **Thorin's jack and lever mass, inertia and center of mass are placeholders**,
  his category IV, "fixed parameters not to be studied": round values, centers
  of mass at the pins [V, Thorin 2013 §4.4.1]. His wippen inertia, 3.97e-4 kg·m²
  for 19.8 g with the center of mass 53.5 mm from the axis [V, Thorin 2013
  §4.5.3], implies a radius of gyration about the center of mass of
  √((3.97e-4 − 0.0198 × 0.0535²) / 0.0198) ≈ 131 mm, for a part about 100 mm
  long.
- **Felt damping**: the 5e7 Thorin attributes to Lozada differs from Lozada's
  own 2e7 for felt and 1e3 and 3e3 for the knuckle's and drop screw's leathers
  [V, Thorin 2013 Eqs 4.9–4.10; Lozada 2007, Table 2.3].

### Hirschkorn's frames

Every body frame in Hirschkorn's Appendix A has its origin at the body's pivot,
as his Figure A.1 draws it: the ground pins are Pg1 (key), Pg2 (wippen) and Pg3
(hammer), and in the wippen's frame Pw2 is the jack's pin and Pw3 the lever's,
both listed as "Pin Location" [V, Hirschkorn 2004, Figure A.1, Tables A.2–A.4].
Read that way the action closes at rest ([Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52"). **Do not take Pw2 for the
wippen's pivot**: that misreading is what gives the 12 µs step, the 570:1 ratio
and the claim that his rest pose cannot be reconstructed. **Do not use 5.84 as a
closed test of the action ratio**: that lever product multiplies radial
distances, takes Pw2 for the wippen's pivot and the jack's pin for its contact,
and uses the front punching, 217.5 mm out, as the key's front arm instead of the
measuring point, and a band of 5.0 to 7.0 could not see a 10 % error in an arm
anyway. The action ratio is read off an assembled model, by virtual work.

### Friction

- **At the axes**: set-valued ([Set-valued dry friction, at the axes and at sliding contacts](#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)). Hirschkorn measures five pivots, key
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
  engine fixes one initialization (frictionless equilibrium, then friction, then
  relaxation), built for the key on its frame ([Parts and statics](#parts-and-statics)), and reports the
  interval's width (planned).

### Seventeen contacts per note

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
depth, as does a face's center against a face. Two discs keep it until their
centers meet, a penetration of both radii added, and have no row past that.
**Do not use** "a corner deeper than an edge arc's center stands over a face
beside the arc, whose own row carries the penetration" as a guarantee: that row
exists only while the face's normal, turned round, lies in the corner's own
angle, and a key's tail corner sunk 0.9 mm into the end of its cloth with the
key nose-up reads no row at all, though the cloth's sides are in its range. The
rigid statics stop every corner at contact and never go that deep; a felt law
that lets a corner sink past an edge's radius needs a reading that sees it. The
hybrid is planned. A circle-against-line gap taken as the unsigned distance
from the circle's center less the radius grows again past one radius of
penetration: right at contact, and wrong for depth, which is why every row is
signed. The shank rests on a cushion
on the wippen or on a felted rail, depending on the action, which changes the
contact's gradient, the shock at the finger, the rest state and the regulation
offered. Contacts 14 and 15 both lift the damper lever from below and the higher
wins, so pedal force falls as notes are played.

## Architecture

### Crates, apps and gates

**Built.**

```
crates/pa-core     the domain: geometry, mechanism, contact, solve, parts, measure,
                   regulation, registry, validate. No I/O, no web; one
                   dependency, libm (Determinism, stated correctly); no number to start from of its
                   own, its tests' fixture compiled for tests only
crates/pa-app      the application: the single-key bench as an instrument, set by
                   path and by action, built from a profile, and its registry;
                   the golden sheet
data/params        the records, each value with its source
data/profiles      what each instrument starts from, every number a record or a
                   stated assumption (Every number has its provenance)
crates/pa-wasm     the browser's adapter of pa-app; the only crate that knows JavaScript
crates/pa-cli      the shell's adapter of pa-app: `weigh`, `band`, `golden`
apps/bench         the bench page: React, react-three-fiber
tools/smoke        drives the real page over the Chrome DevTools protocol
scripts/verify.sh  every gate, run locally by `pnpm verify`
scripts/smoke.sh   the browser smoke test: the built bench in a headless Chrome
docs               this document, the research annexes A1 to A5, the vocabulary
                   worksheet
tools/derivations  the script behind every [E] number, run with python3 -I
tools/dynamics-prototype  the one-note dynamics prototype, in Python, and its
                   results
data/references    the primary corpus: ignored by git, third-party copyright
CLAUDE.md, .claude the agents' manual, the rules files with their checks, the
                   skills, the agents that plan, review and fix, and the
                   session settings
```

The adapters add no physics and the React tree computes none of the numbers it
shows; `pa-wasm` carries a `package.json`, so Turborepo builds it with
wasm-pack. The CI workflow (`.github/workflows/ci.yml`) runs the format check,
Clippy with warnings as errors, the tests and the WebAssembly build for Rust,
and Biome, the type-check, the build, the tests and the smoke test for the web;
`scripts/verify.sh` runs the same gates locally.

**Review.** The gates check what a tool can; the rules no tool can check, which
`.claude/rules/` marks *not mechanically checked*, are checked by
`/expert-review` before a branch reaches `main`. It runs in a subagent that
never saw the code being written, and waits for its experts, so the session
that wrote the code keeps its context. It sends one read-only expert per domain
in parallel: the generalists (correctness, conventions, architecture, tests,
security, documentation, performance) and the project's own (metrology,
physics, provenance, vocabulary), each reading the rules files of its domain.
Each agent runs on the model and at the effort its task needs, as
`.claude/rules/configuration.md` > Claude Code automation, "Each agent runs on
the model and at the effort its task needs" says, because every turn of every
run pays for both and a review multiplies the runs; a low effort saves Haiku
and Sonnet little and costs them much quality.
An expert pays a full reading of its rules, the diff and the code even when it
has nothing to say, so a triage on Sonnet first weighs the experts the paths
pick by what the diff holds; it keeps an expert when unsure, and never drops
correctness nor security on its paths, nor narrows what they read, since a
missed expert costs more than a needless one and a triage that reads the diff
can be steered by it. The experts start without the session's `CLAUDE.md`
files, which would cost tokens on every turn and could steer them: the rules
that apply are the checkout's. The report ends with what each agent cost, so
that the cost of a review shows while it builds up. A round of fixes is
reviewed against the findings it answers, by their experts, by correctness, and
by the experts whose concerns the fixes newly touch, rather than by every
expert whose paths they touch: a fix of three files can touch the paths of
every domain, and only the experts it concerns have something to check. Every
finding that would block or should be fixed is verified before it is reported,
so the report claims nothing it has not checked, and every fix it suggests is
held to the project's rules, since an expert reads only those of its domain.

**Rounds of fixes.** A fix can bring the bug that the next round finds, so each
round of fixes is reviewed. A defect that a fix brought into the code it fixed
says the method is wrong: a test of the whole range, or a simpler design, ends
the chase sooner than more rounds, and past a few rounds the owner weighs what
remains against what the rounds cost. A new agent fixes each round, from a
brief: an agent resumed round after round re-reads its whole, growing context
on every turn, which costs more than the review it answers.

**Plans.** A mistake in a plan costs every step built on it: the code, its
review and the rounds of fixes. A plan is short, so the thorough reasoning of
its planner and its critic costs less there than anywhere else. A plan
chooses the method before the code, when a test of the whole range, a simpler
design or a library costs nothing to choose; chosen after, it costs the rounds
of review that showed the need for it. A critic that did not write the plan
checks it against the code and the rules, since an assumption about the code
that no one checked is the cheapest mistake to catch there, and the code
follows the plan or the plan changes first, since a design changed while it is
built escapes the critic. Within one planning, the planner answers the critic
itself, as it already holds the code the plan rests on. A change that leaves
the design as it is, such as a bug fix inside one function, goes without a
plan, which would cost more there than the mistakes it could catch. The plan
lives with the project's tracking, outside this repository, since this
repository holds only the present and a plan describes work to come, and each
plan has one writer, the session that owns its branch, so plans written in
parallel never collide.

**The golden sheet** records every reading of the default bench, Hirschkorn's
key as it arrives, and of six fixture benches in full bits, in
`crates/pa-app/tests/golden/`: on his key, a frame whose stops fight, a dip
stopped by the front pin in its mortise and a key shortened under its frame;
the owner's measured section as it arrives, every reading that needs the
key's mass refused, the same weighed by a stated typical weighing, and weighed
back-heavy. For each, each reading of the weighing sheet, its value in
full and as the trade reads it, its trust, the walk's start and the temporary
weight, or why it is refused; then what holds the key, how it rides where its
stops fight, the faults, the papers and the pose; last, D and U read on the
band, with the sweep's window and where each fell, or why the band cannot be
drawn. `pa-cli golden` writes it and
`cargo test` compares the engine with it, so a change shows every number it
moves; one that means to move a number rewrites the sheet and says why. A Node
test in `pnpm test` loads the WebAssembly module the browser loads and holds it
to the same sheets, bit for bit. The bedding at the back, which the sheet shows
in degrees, is recorded as the slope it comes from, because the platform's
`atan` may differ in the last bit.

**The engine and its applications.** The crates are the engine ([An engine for piano applications](#an-engine-for-piano-applications)) and
`apps/bench` its first application. Built: the engine's interface is what an
application is written against, the instrument of `pa-app` ([The WebAssembly boundary](#the-webassembly-boundary)): its
registry and validation layer ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)) make the configurator, its documents the
diagnostics, and its log and state hash a state that replays. Planned:
persistence ([Persistence](#persistence)); a rendering package and an interface package extracted
from the bench when a second application needs them, not created empty
before.

**One definition per part.** The key and its frame are built as parts: one
closed outline of lines and arcs for each piece of material, on the body that
carries it, several per body, a mortise a cut among them and a turned part read
through its sagittal silhouette; a milled cut, a mortise or the balance hole,
is given by its section through its middle and its cutter's diameter, so that
its mouth stays one segment, which the wood it opens through shares, and the
front rail pin's oval head by its section along the key and its width
across it. Each part declares the ranges of its outline a
contact may read and the named points on it, the lip and the key front's top
among them. The bench draws them and the faults read them; whatever a stop
rounds is drawn rounded, the back rail cloth's edges included. The contacts read
the declared ranges and nothing else: a contact pair holds nothing but ranges of
parts, and the mechanism its rows are read on carries the very parts the view
draws, which `every_contact_surface_is_a_range_of_a_drawn_part` holds on every
bench of the golden sheet. The static solve places the key on those contacts,
and every procedure of the sheet and the drawing read the key placed there, a
key riding on a frame whose stops fight included. Nothing reads anything but a
drawn range, so the invariant holds by structure; and
`every_contact_lies_on_its_drawn_outline` holds every row's two points on the
drawn outlines wherever the key stands, seated, riding, lifted off its
punching or turned across its stroke, by a sampling of the drawing that shares
no code with the rows. No outline is weighed: the drawn parts tile one polygon
through the key's fields, and their area centroid, the mortises and the balance
hole taken out, stands in for the height of a weighed key's center of mass. Planned: each
part's inertia derived from its outline or measured, by an explicit choice, a
key's always measured ([Named simplifications](#named-simplifications)).

### The WebAssembly boundary

**Built.** The bench boots the module once (the boot promise is cached, so a
double mount cannot construct two engines) and reads six JSON documents, each
apart (`crates/pa-app/src/boundary.rs`), and a seventh when it shows the band.
The **registry**: every setting's path, its unit in its name
(`frame.front_rail.pin_height_mm`), what it measures, its layer, the range it
admits and how it moves, and whether `set` reaches it or an action does; and the page that presents them, panel by
panel, each control with the code its label is worded by and the range its
slider offers ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)). The **settings**: every value the controls set, nested
as the paths are, so the value at a path is read at that path, the key's
weighing as typed and the bushings' friction as the torque it is stored as
among them, always valid whatever the frame does; the key friction weight
its control shows is the sheet's reading of that torque. The **sheet**: every
reading by its procedure's code, read (its value in full, what the trade
reads it as by the protocol's rounding, its unit and resolution, its trust,
its conditions and the solves it took) or refused with its reason, whether
or not the key can be seated, so that a frame that cannot seat its key reads
its jig and refuses the rest line by line. The **diagnostics**: why the key
cannot be seated, how it rides where its stops fight, the faults, how it
stands at its two seats, what holds it under the weight on its measuring
point and the gaps there, how far the punchings' drawn edges overlap it, and
the last edit refused since one was taken, with why. The **state**: one pose
per body, an origin, a cosine and a sine, which the view applies and computes
none of; where the key stands on its frame, placed by an equilibrium, with
none since it was put on it, or lifted off it by the weight on its measuring
point; the measuring point on the key; the contacts within a millimeter of
closing, or, where no equilibrium places the key, only those where its drawn
parts overlap the frame's, each its two surface points in the keybed's frame, its gap, whether
it is closed, open or overlapping, its row's normal, and the force the row
carried in the last equilibrium, one admissible split of the loads where
friction or redundant rows share them ([Parts and statics](#parts-and-statics)), never read as a verdict. The
**scene**: every part's outline in its own body's frame, in millimeters, with
its material and the provenance of each stretch, which the page keeps while
the geometry does not change. The **band**, asked for only when the page shows
it: D and U read on it, each read like a reading of the sheet, with the sweep
it was read by, its step, its window and where on the stroke it fell, or
refused with why; and the two curves, each step's travel in millimeters and
force in grams. The touch's diagnostics carry where the weight has left the
key on the band's travel, so the dot follows the weight without the band
being swept again. Every number in a trade unit carries it in its name. Faults, verdicts, refusals and problems cross as codes with their
parameters in the trade's units, each unit in its parameter's name
(`{"code":"pin_outside_mortise","by_mm":16.5}`); they mirror the domain's
enums through matches with no catch-all arm, so a refusal added in `pa-core`
and not mapped is a compile error. The page words them from its catalog
([A bilingual interface](#a-bilingual-interface)), writes each reading of the sheet to its protocol's resolution, and
the band's two in full: the band is a curve, which no weight reads.

The page changes the bench by two calls. `set(path, value)` sets a setting in
the unit its path names; `act(action)` runs an operation: laying a paper from
the drawer or lifting one, regulating to a key height and a key dip, weighing
the key whole, typing the key friction weight Stanwood's protocol reads,
placing the frame under the key, putting another shipped profile's instrument
on the bench. Each answers that it was done, the papers a
regulation laid and what they give, or why it was refused, by its code, the
bench left as it was to the bit. After every edit taken the key is solved one
increment from where it stood, so it is drawn at an equilibrium wherever one
has held it since it was put on its frame. Every
edit taken is logged, and the log replayed on the bench it started from gives
the same state, which `state_hash` names: FNV-1a over the bit patterns of
every normative coordinate, of where the last equilibrium left the key and
of what each contact row carried there, in a fixed order, a hash whose
algorithm no Rust release changes. The
TypeScript types of every document and of the actions are generated from the
Rust ones by `ts-rs` into `crates/pa-wasm/types/`, committed and re-exported
by `@pas/wasm` as `@pas/wasm/types`, never edited by hand: a test in
`pa-app` generates them and fails, rewriting them, wherever the committed
files differ. `pa-cli` changes the bench by the same calls and prints the same
codes. A function returns any fixture's golden sheet, for the test that holds
the module to the native bits.

The page is drawn from the registry (`apps/bench/src/Control.tsx`): one
component draws whichever control the registry describes, a slider over a
setting, a punching's papers laid from the drawer and lifted, fields sent
together by one action, the key friction weight, a button, the choice of the
profile on the bench; its value read
from the settings at its path, its range, step and unit the engine's, its
label the catalog's by the code the registry gives it. A form's fields
open on what the bench holds for each quantity, as the registry says where to
read it. An edit refused is said right after the control that made it, from
the diagnostics. Each control carries the engine's identifier, `data-path`
for a setting and `data-action` for an action, as each reading carries its
procedure's, and the smoke test drives the page by those, never by its
wording, so it drives a page in any language.

**Planned.** Outlines cross only when the geometry
changes, and poses at each step. Per-step state is a flat `f64` buffer read
through a pointer and a length, the JavaScript view rebuilt whenever
`memory.buffer` changes, since growing linear memory detaches old views. A
step that produces a non-finite value stops and keeps the last finite state,
so none is ever published.

### The parameter registry and the validation layer

**Built.** Every setting is declared in `pa-core` beside the field it sets
(`pa_core::registry`): what it measures, the range a part can physically have
in SI, how it moves, its layer (design, regulation, protocol, experiment) and
whether it is set directly, aimed at as a target or typed as it was read on
a real part. A regulating screw turns continuously, with no quantum; a
paper is one the drawer holds. `pa-app` composes the declarations under
paths, each with the trade unit in its name, converting once between the
domain's SI and that unit, a position behind the balance pin read as the
distance behind it; and adds what only an application knows, the page that
presents them: its panels in order, each control with the path it sets or the
action it runs, the range a slider offers inside the one the setting admits,
the code it is worded by, and where a form's field reads its first value. A key friction weight is typed and shown in grams at the
measuring point, as Stanwood's protocol reads it, through the action that
inverts the protocol; what the registry stores is the torque. The papers are
a list on each punching, held once in the frame and changed only by laying or
lifting a paper; ratios have no path, being outputs
(`.claude/rules/metrology.md` > Ratios are outputs).

`validate` (`pa_core::validate`) runs on every edit and on every design an
instrument is built from, before any solve, and refuses what no part could be
built as, and the one pairing of a key with its frame the contacts could not
read, naming the limiting dimension and the shortfall: a value that is not a
number or an infinity, anywhere in the design; a value outside its setting's
range; a paper no drawer holds; a key weighed at no mass; an outline crossing
itself, a glued part run into the next along the key's top (a key front brought
back until its covering runs into the key button), a clearance cut in the top
run into a block or under the capstan's head, a felted thinned tail run on
under a glued block, a block standing across the step down to a bare one, or
the chamfer past the key front; a clearance run back over the thinned tail,
cut through the keystick, or no cut; the capstan in a glued block, wholly
behind the back check block or over the thinned tail, or its dome narrower
than its head; the front mortise off the flat of
the underside, cut through the top of the keystick, milled wider than it, or
by a cutter not moved along the key, with no straight wall for its bushing;
its groove deeper than the mortise, or too long for the front rail punching,
which past the mortise's straight walls would find no wood beside the mortise
and stand in the groove, under its floor, where its contact reads the line of
the underside; its bushing's two thicknesses
wider than the mortise, its cloth thicker than the groove is deep, run up the
walls past the mortise's top or laid flat past the key's side; a key button whose
rounded ends overlap; a part of the balance mortise that does not fit where it
is cut, the slot past the key button's flat top or wider than the key, the
cavity's floor above the keystick's top, the hole wider than the slot, the
bushing's two thicknesses wider than the slot, the slot milled by a cutter
not moved along the key, with no straight wall for the bushing, the cloth thicker than the
side holes it turns out through are high, the bushing deeper than the key
button is high, a side hole reaching below the key button, or the bushing
less deep than it is thick, its side hole's top then past the key button's;
the measuring point off the covering. A refused edit leaves the instrument as it was, to the bit of its
state hash, and the diagnostics keep it. Everything an assembled keyboard can
be, however badly regulated, is computed and reported, not refused: a pin
standing proud of its mortise is a state, the key riding on it; an action
regulated into jamming is a state to report, not an error.

An instrument starts from a profile ([Every number has its provenance](#every-number-has-its-provenance), [The data model](#the-data-model)), every number of which is a
record or a stated assumption; the profile it was built from is kept with
each value's provenance, and resetting the instrument returns to it, not to
another.

Each drawn stretch of every part is sent with the number that places it and
how its profile knows it, which the view marks when asked: every line of the
section, and in the solid the outlines of the extruded parts, turned and
oval parts and the walls of the carved cuts staying unmarked there; the profile on the
bench is chosen among those shipped, by an action that puts it there as it
arrives (built).

**Planned.** Each setting's provenance sent beside it to the page; the paths a
profile needs beyond the bench's page, the tail's end, the back check block's
place, the front mortise's place, the key's row; the rails checked
against one another and the capstan's thread against its hole ([Named simplifications](#named-simplifications)).

### Persistence

**Planned.** A saved file holds the coordinates, which are normative, and each
target with its achieved value and the solver's fingerprint ([Regulation as targets; the coordinate is normative](#regulation-as-targets-the-coordinate-is-normative)). On opening,
the coordinates are used as they are and the targets re-solved and compared; a
divergence is offered as a choice, never resolved silently. The schema is
versioned, each migration a pure function with a fixture. Drop is stored with
its reference, from the string or below let-off, because makers write it both
ways. Golden scenarios start from literal coordinates, so a change in the
regulation solver cannot pass for a change in the physics. `pa-core` gains
`serde` behind an optional feature for this.

### The data model

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
  Hirschkorn's action ([Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52"). Built: a profile (`data/profiles/`) may name
  a base and bind only what differs, each binding and each choice (the key's
  row, how its mass is known, whether it arrives regulated) replacing the
  base's; the two shipped, Hirschkorn's key on his printed frame and the
  owner's measured section, have none, and Hirschkorn's action as the base
  every other profile is a difference from is planned. All layers but one are
  measurable with a calliper and the trade's gauges; the material set needs a
  compression bench, so it is inherited, and anything computed from an
  inherited layer says so and shows as indicative.
- **A measurement carries its provenance**: value and unit, the solver, its
  conditions (dampers and repetition spring engaged or not, where the stroke
  stopped), and its trust, metrological or indicative. A weighing without its
  conditions is not a datum. Built for the key's readings ([The protocols, as they are](#the-protocols-as-they-are)), each with
  what the trade reads it as, its resolution, the mount, the seat, the walk
  and the temporary weight, its trust from the bases of what it reads, and
  the solver's fingerprint; the conditions an action adds, dampers and
  springs, come with it, and the bases from the registry's records ([The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)).

### The 88-note runtime

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

## Stages

One table, in order. Each stage ships its engine and its view together and ends
on a number the owner can judge by the trade's protocol.

| Stage | What is added | What becomes measurable | It ends when | Status |
|---|---|---|---|---|
| 1a | the key on its frame: rails, punchings, papers | key height, key dip, front weight, key weight ratio, key friction weight | a key the owner measures ([Validation instruments](#validation-instruments), "One real key, measured by the owner") agrees with the bench by the protocol | built; its validation planned |
| dynamics prototype | one note of Hirschkorn's action in motion, thrown away afterwards | the step, the solver, the felt law, the cost per step ([What the dynamics prototype settles, and what it leaves open](#what-the-dynamics-prototype-settles-and-what-it-leaves-open)) | each hypothesis of [What the dynamics prototype settles, and what it leaves open](#what-the-dynamics-prototype-settles-and-what-it-leaves-open) is settled by its test | built: step, solver and cost settled; the felt law ([The felt law](#the-felt-law)) and the flexible hammer ([Rigid bodies](#rigid-bodies)) open |
| quasi-static core | bodies in a tree, contacts over all coordinates, set-valued axis friction, solved to convergence under a cap, the key as first client and Hirschkorn's key as the default ([What is built: stage 1a, the key on its frame](#what-is-built-stage-1a-the-key-on-its-frame)); the registry, the validation layer, the boundary in codes and numbers, the bilingual interface ([Architecture](#architecture)) | every weighing quantity by procedure ([The protocols, as they are](#the-protocols-as-they-are)) | the key's readings come from procedures run on the model, the closed forms passing as oracles | the tree, the jig, the contacts over all coordinates, the static solve, the key free to lift off its balance punching, the seated pose that reads key height and dip, the regulation solve on it, the touch weight as an increment, every weighing quantity by procedure, as measurements with their trust ([Parts and statics](#parts-and-statics), [The protocols, as they are](#the-protocols-as-they-are)), the boundary in codes and numbers, registry, settings, sheet and diagnostics apart, with its generated types and its catalogs in English and French ([The WebAssembly boundary](#the-webassembly-boundary), [A bilingual interface](#a-bilingual-interface)), and the instrument set by path and by action, its registry and validation layer, its log and state hash ([The WebAssembly boundary](#the-webassembly-boundary), [The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)), and the profiles that bind every number it starts from to a record or a stated assumption ([Every number has its provenance](#every-number-has-its-provenance)), Hirschkorn's key on his printed frame the default and the owner's measured section beside it, unweighed until he weighs one of its keys, chosen on the bench, every drawn line marked by how its number is known ([What is built: stage 1a, the key on its frame](#what-is-built-stage-1a-the-key-on-its-frame), [Validation instruments](#validation-instruments), "Hirschkorn's Boston GP-178, note 52", [The parameter registry and the validation layer](#the-parameter-registry-and-the-validation-layer)), a key's mass weighed or published and never derived from its outline ([Named simplifications](#named-simplifications)), and the band, drawn by the driven sweep that reads D and U a second way ([The protocols, as they are](#the-protocols-as-they-are), [The bench](#the-bench)), built; the rest planned |
| 1b | the wippen on its rail, the capstan contact, the support spring where fitted | wippen weight, support spring balance weight | a weighed wippen agrees with its model by the protocol | planned |
| 1c | the hammer and its shank | strike weight, strike ratio | a weighed hammer agrees with its model by the protocol | planned |
| 1d | nothing new | the equation of balance, closed | D, U, BW and FW of an assembled key, wippen and hammer agree with the same parts weighed | planned |
| 2 | the jack, the repetition lever, let-off, drop, checking: the action in motion | let-off, drop, checking, the slow-release test, the key force under position drive | Thorin's key force and Hirschkorn's trajectories are reproduced within their measured scatter ([Validation instruments](#validation-instruments)) | planned |
| 3 | regulation training | every setting as a target and an actuator; exercises; faults named as the trade names them | the trade's regulation sequence runs end to end on the model | planned |
| 4 | dampers, pedals, 88 notes | damper lift point, the pedals, a score played | a score plays in real time with the stage-2 readings unchanged | planned |

## Open questions for the owner

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

The French terms in doubt are listed in `docs/vocabulary-worksheet.md`. And an
opportunity: a compression bench, even a rudimentary one, would first repay the
four contacts that dominate the force the finger feels (capstan on heel, jack
and lever on knuckle, front rail punching); without one, every contact law is
borrowed ([The data model](#the-data-model)).

## Sources

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
- **Timmermans, Ceulemans and Fisette**, "Upright and grand piano actions dynamic
  performances assessments using a multibody approach", *Mechanism and Machine
  Theory* 160, 2021, doi:10.1016/j.mechmachtheory.2021.104296; and Fisette,
  Bokiau and Timmermans, "The grand piano action functioning demystified thanks
  to the multibody approach" (IFToMM 2019), the authors' manuscript.
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

Not in the corpus, and drawn on by the bench's view: **McGuire and Bavoil**,
"Weighted Blended Order-Independent Transparency", *Journal of Computer
Graphics Techniques* 2(2):122–141, 2013, http://jcgt.org/published/0002/02/09/,
the way the solid draws its faded parts ([The bench](#the-bench)).

Not in the corpus, and wanted by the stages ahead: Askenfelt and Jansson on the
grand action's timing (1990); Gillespie et al.'s key force profiles (2011);
Timmermans, Dehez and Fisette's real-time haptic key (*Machines*, 2020);
Pfeiffer, *The Piano Key and Whippen* (1967); the Preisach model, which
Hirschkorn proposes for felt; quasi-static frictional contact and its
non-uniqueness (Pang and Trinkle 1996; Klarbring 1990); implicit compliant
contact (Todorov 2014; Castro et al. 2023); sensitivities across contact events
(saltation matrices; Leine and Nijmeijer 2004).
