# Derivations

Every number marked `[E]` in `docs/FOUNDATIONS.md` comes from a script here, and
its marker names the script by number: `[E, 10]` is
`10-hirschkorn-assembled-at-rest.py`. A number whose derivation lives only
in prose is not reproducible, so a computed number without a script here does
not carry `[E]`. A value an instrument starts from that is derived by a rule
short enough to check in one's head (the measured section's tail end, its
key's overall length laid back from its lip) states the rule beside it, in its
record or its profile (`data/profiles/`), marked `E`; one a reader could not
check that way is a script here, which the record or the profile names.

The scripts are pure arithmetic on the standard library, so each runs anywhere
with `python3 -I <script>`. Each one states its sources at the top. Scripts 10
and 11 read every value of Hirschkorn's they use from
`data/params/hirschkorn-boston-gp178-note52.json`, with `json`, by a path
relative to themselves, so the record and the scripts cannot drift apart;
script 11 reads the pixels it fits from `data/params/hirschkorn-key-outline.json`
and his profile from `data/profiles/hirschkorn.json` the same way, and fails
where the numbers that record gives are not the ones its pixels give, or the
numbers that profile states by a rule are not the ones the rule gives; the
other scripts type their inputs.

| Script | What it establishes |
|---|---|
| `01-inertia-about-pivot.py` | the inertia of each of Hirschkorn's five bodies about its own pivot, from his Table A.1, by the parallel-axis theorem |
| `04-elastic-force-sign.py` | that none of the thirteen published cubic contact laws (Hirschkorn, Table A.8) pulls for a positive penetration, although several have a negative quadratic term; and the separation speed, −1/D, below which each damped law turns attractive without a clamp |
| `05-stiffness-table-check.py` | that all thirteen elastic branches are strictly increasing, and where their stiffness dips before it rises; that Hirschkorn's and Thorin's jack/knuckle laws differ by 3.1 to 3.6 in tangent stiffness at equal force, the 56 being a ratio of cubic coefficients; and that `2/ω` is the undamped stability limit of a semi-implicit step, not its limit |
| `06-alpha31-argument-check.py` | three figures of the research annexes, recomputed: hammer travel from a constant ratio once aftertouch is taken off the dip; the annex's friction sum, which leaves out two of the five pivots; and two compute-cost figures that cannot both hold |
| `07-regularized-friction-check.py` | what a regularized joint friction costs (a non-physical threshold speed, and a tiny step only if integrated explicitly); that with one damping constant the additive and the multiplicative contact damping disagree away from the penetration it was calibrated at, so the clamp is the remedy; and the static friction peaks of Hirschkorn's sliding contacts, found by maximising his law |
| `08-peak-force-plausibility.py` | the hammer-head acceleration a given force at the jack/knuckle implies, with the arm taken as the moment arm of the contact normal |
| `10-hirschkorn-assembled-at-rest.py` | Hirschkorn's action assembled at rest with every frame origin at its own pivot: the rest pose that closes, every gap there, the mass matrix, the lever arms, the effective mass along each contact normal, the reflected masses on the two sides of the jack/knuckle contact, the highest linearised mode at assumed contact forces, and the key between its two stops on his rigid frame, its attitudes at rest and at the bottom and the dip at Pk2, which the bench's cross-implementation test reads |
| `11-hirschkorn-key-outline.py` | Hirschkorn's key traced from his Figure A.1: the similarity fitted on Pg1, Pk2 and Pk5, the residuals at the other markers as the trace's uncertainty, every feature of the key's outline in his key frame and the numbers his profile binds from them (`[E, 11]`), among them where the keystick's full height ends behind the capstan, its top lowered from there to the key's end, the top the figure draws 0.75 mm higher over the full height's last 29.65 mm printed as dropped, within the trace's uncertainty; the figure's centre-of-mass marker against Table A.1, which governs the mass, and Pk4's marker along its line; the front weight, key weight ratio and key friction weight his numbers give in closed form at the bench's measuring point; his frame between its stops on his rounds and on the flat felts his profile draws, the cloth wholly under the chamfer with its rounded front edge over Pg5, which reproduce his dip at Pk2 to 1.7 µm, and where the key bears at the back on each; the rail heights, the back rail's place and the front mortise's place his profile states by a rule from his frame, and the thinned tail's length it states from the trace, from the key's end to where the full height ends, which it fails on where they are not its own; and the capstan's height above its seat, the keystick's full height |

## The trap this directory exists to avoid

**Hirschkorn's body frames have their origin at the body's own pivot**, the
wippen's included: his Figure A.1 draws them there. In the wippen's frame, `Pw2`
and `Pw3` are the pins of the jack and of the repetition lever, not the wippen's
pivot. Read the other way, every lever arm, reflected mass and time step
derived from it is wrong; read this way, his action closes at rest (script 10).

The second half of the same trap: **an effective mass is not `I_pivot / r²`
with `r` the distance from a pivot to a contact point.** The arm that counts is
the moment arm of the contact normal, `((P − O) × n)·ẑ`, taken for each
coordinate with `O` the pivot that coordinate turns about; and the jack and the
lever ride on the wippen. At rest the knuckle sits on the jack's flat top, whose
normal runs almost through the jack's pin, so the jack's own arm there is a few
millimetres although its contact point is 49.5 mm from the pin. Script 10 does
it the right way: `1 / (∇g · M⁻¹ ∇g)` on the assembled mass matrix.
