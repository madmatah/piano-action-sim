> Translated from the original French research annex. Part of the `piano-action-sim` foundations
> study (October 2026). See `docs/FOUNDATIONS.md` for the decisions this annex supports.

> **Reliability convention used throughout this document**
> **[V]** = verified, source cited at the end of the document. **[S]** = assumed / inferred by me, to be validated. **[?]** = term or value I am not sure of, to be validated by the client (piano technician).
> All the inch values from the American manuals are converted to mm (factor 25.4) and the original value is retained, because the data model will have to handle both units.

---

# 0. Methodological caveat: what "the" specification is not

No regulation dimension is universal. Steinway states it in black and white in its own guide: the specifications are important but may vary within reasonable limits to suit the player, and since each step affects the others, the operations must be repeated to reach stability **[V]**. Direct consequence for the architecture: the data model must carry, for each regulation setting, **(value, acceptable range, source/preset, keyboard zone)** — not a constant.

I have therefore built a **set of presets** (Steinway NY, Steinway Hamburg, Yamaha, Kawai, "generic range") rather than a set of default values.

---

# 1. Exhaustive part inventory (modern grand piano)

Table format: **FR | EN | role | DOF** (degrees of freedom) **| to be parameterised**.
DOF convention: `R` = planar rotation (1 DOF), `T` = translation, `0` = fixed/structural part (serves as a reference frame or as a stop), `C` = contact only (no body of its own, can be modelled as a surface).

## 1.1 Key frame, rails, structural members

| FR | EN | Role | DOF | To be parameterised |
|---|---|---|---|---|
| Plateau de clavier / fond de clavier | keybed | Reference frame of the keyboard; its flatness conditions everything | 0 | Crowned/hollow: Steinway NY deliberately creates a **crown of ~1/32" (0.79 mm) at the centre of the front lip**, a central hollow of ~1/32" between the front rail and the back rail; Hamburg/Boston/Essex are **flat** **[V]** |
| Châssis de clavier | key frame | Carries the 3 rails; **translates laterally** with the una corda | **T** (una corda) | Shift travel, friction on the dag blocks (graphited), stiffness |
| Barre de balancier | balance rail | Axis of rotation of the keys | 0 | Height, longitudinal position, crown of the key leveling (Steinway: **centre ~1/32" higher than the ends**) **[V]** |
| Pointe de balancier | balance (rail) pin | Axis + lateral guidance | 0 | Ø **2–3 mm** **[V]**, lateral position (squaring of the keys) |
| Mouche/rondelle de balancier, "balance rail bearing" (NY) | balance rail punching / bearing | Felt bearing + paper punching for key leveling | C | Thickness of the felt, stack of paper punchings (this is **the** key-leveling parameter) |
| Barre d'enfoncement | front rail | End-of-travel stop + guidance | 0 | Height |
| Pointe d'enfoncement | front rail pin | Lateral guidance (oval pin) | 0 | Tolerated lateral play: Steinway **≤ 1/32" (0.79 mm)**, Kawai **0.2–0.3 mm** with a clean little "knock" **[V]** |
| Mouche d'enfoncement (+ cales papier) | front rail punching (+ paper punchings) | **Defines the key dip** | C | Felt thickness + stack of punchings (Kawai rule: max 4 punchings of the same thickness, the thickest at the bottom) **[V]** |
| Barre de repos arrière + drap | back rail + back rail cloth | Upper stop for the key tail | C | Thickness/compliance of the cloth |
| Vis de réglage de portée du châssis ("pistons", glides) | balance rail studs / glide bolts | Press the floating balance rail down onto the keybed | 0 adjustable | Height; **must never be used to set the key dip** **[V]** |
| Barre de contre-touche | key stop rail | Limits the upward return of the keys | 0 adjustable | Clearance: Steinway **~1/16" (1.59 mm)** above the keys **[V]** |
| Blocs/joues de clavier | keyblocks / cheekblocks | Lock the keyboard laterally, carry the pressure screw | 0 | Pressure screw (contact with the guide pins) |
| Montants de mécanique | action brackets | Carry the rails | 0 | Height (changes the ratio, cf. §4) |
| Barre des chevalets | wippen rail | Axis of the wippens | 0 | Position ("spread") |
| Barre des marteaux | hammer (shank) rail / main action rail | Axis of the hammers | 0 | Position |
| Barre d'échappement (barre des boutons d'échappement) | let-off rail / regulating rail | Carries the let-off buttons | 0 adjustable | Overall height |
| Barre de repos des marteaux | hammer rest rail / cushion | Rest for the shanks | 0 adjustable | Kawai: **5 mm below the shanks at rest** **[V]**; compliance of the felt |
| Barre de butée des étouffoirs | damper (up)stop rail | Limits the lift of the dampers | 0 adjustable | cf. §2 |
| Barre de guidage des étouffoirs + garnitures | damper guide rail + bushings | Guides the damper wires | 0 | Bore Ø (**~3 mm** for a **~1 mm** wire in Igrec's example) **[V]**, friction, deliberate off-centring of the wire |

> **Settled.** "Limbe" is not the trade term for the *action frame/rails*. Standard
> French usage, as carried by the numbered sectional plate of a grand and by the
> chapter on action rails, is: **plateau** (keybed), **châssis de clavier** (key
> frame), **barres de mécanique** (action rails — *barre des marteaux*, *barre des
> chevalets*, *barre des boutons d'échappement*) and **montants de mécanique**
> (action brackets). **Beware of "sommier"**: on a grand it is the **pinblock**,
> never a rail. Using it for the wippen or hammer rail is the error to avoid, and
> a common one.

## 1.2 Key

| FR | EN | Role | DOF | To be parameterised |
|---|---|---|---|---|
| Touche (corps) | key (stick / lever) | Input lever | **R** about the balance rail pin | Front/rear length (see §4: "key front → balance hole" ≈ 259.5 mm and "key front → capstan" ≈ 398 mm in the Reyburn example) **[V]**, mass, inertia, wood |
| Mortaise de balancier + garniture | balance mortise + bushing | Front/rear bearing | C | Play, friction |
| Mortaise d'enfoncement + garniture | front mortise + bushing | Lateral bearing | C | Play, friction |
| Capsule de touche | key button / chase | Carries the balance mortise | 0 | — |
| Patin ("shoe") de balancier | key shoe | Extra thickness under the key (older pianos) | 0 | Typically **4–6 mm**; only the **external** shoes count in the geometry **[V]** |
| Plombs de touche | key leads | Counterbalance the hammers | 0 | Ø **10 and 12 mm** usual; longitudinal position and number — **these are the parameters of `front_weight`** **[V]** |
| Pilote (de touche) | capstan (screw) | Transmits to the wippen heel | 0 adjustable (screwing) | **Height** (sets the blow distance), longitudinal position (sets the ratio), angle (straight or inclined), surface condition (friction) |
| Talon (queue) de touche | key end / tail | Lifts the damper under-lever | C | Key-end felt |
| Revêtement | key covering / keytop | — | 0 | Thickness; height of the sharps |
| Fronton | key front | — | 0 | — |

## 1.3 Wippen (= chevalet de mécanique) and the let-off sub-assembly

| FR | EN | Role | DOF | To be parameterised |
|---|---|---|---|---|
| Chevalet (de mécanique) | wippen / whippen | Intermediate lever | **R** | Mass, inertia, "spread" (see §4) |
| Talon de chevalet | wippen heel (+ heel cloth) | Receives the capstan | C | **Height**: WNG offers 9 heels, contact surface **16 → 24 mm below the centre of the jack, in 1 mm steps**, sharps typically **+2 mm**; **position**: 14 locations from **50.0 → 69.5 mm** from the centre of rotation, in 1.5 mm steps (standard **62.0 mm**) **[V]** |
| Fourche de chevalet | wippen flange | Axis | 0 | Position; friction of the pivot |
| Bâton (ou doigt) d'échappement | jack / fly (Steinway) | Pushes the knuckle, then lets off | **R** on the wippen | Length, mass; fore/aft position under the knuckle; **WNG standard: centre of the repetition flange → centre of the jack = 99 mm ± 1 mm** **[V]** |
| Pied du bâton d'échappement | jack tender / jack heel | Comes to touch the let-off button | C | Position; angle of the tender (WNG uses a "low angle" tender) **[V]** |
| Ressort de bâton / ressort combiné | jack spring / combination spring | Returns the jack | — | Stiffness, preload |
| Bouton d'échappement (aussi *pilote d'échappement*, *poupée*) | let-off button / regulating button | **Triggers the let-off** | 0 adjustable | **Height**; compliance of the felt/cloth |
| Levier de répétition | repetition lever / balancier (Steinway) | Carries the knuckle after let-off, lifts the hammer back up | **R** on the wippen | Mass, inertia; **height relative to the jack** |
| Vis de chute | drop screw | Stop for the repetition lever → sets the **drop** | 0 adjustable | Height |
| Ressort de répétition | repetition spring | Lifts the repetition lever and assists the return of the key | — | **Preload/stiffness** (adjusted by twisting the coil, not the curve) **[V]**; graduation: slower in the bass, faster towards the treble **[V]** |
| Vis de tension du ressort | repetition spring tension screw | Screwed variant (some actions) | 0 adjustable | — |
| Attrape + garniture (peau/cuir) | back check + leather | Catches the hammer tail | 0 adjustable (bent wire) | **Height, angle ("slope"), lateral alignment, squaring**; coefficient of friction of the skin |
| Fil d'attrape | back check wire | Support adjustable by bending | 0 adjustable | — |
| Coussin de chevalet / coussin de repos | wippen cushion / rest cushion | Rest for the wippen or for the lever | C | Compliance; WNG offers a "rest rail" **or** "cushion" variant **[V]** |
| Ressort auxiliaire de chevalet | wippen assist / helper spring | Reduces the downweight without adding inertia | — | Preload; WNG kit = **90 springs** **[V]** |

## 1.4 Hammer

| FR | EN | Role | DOF | To be parameterised |
|---|---|---|---|---|
| Ensemble marteau (olive + manche + tête) | hammer assembly | Strikes the string | **R** | Mass, inertia, position of the centre of gravity |
| Olive (du manche de marteau) | hammer shank flange | Fixed part hinged on the rail | 0 | Width **12 mm** (Renner type); traveling shim ("traveling paper") |
| Manche de marteau | hammer shank | Elastic lever arm | — | **Thickness at the centre: 6 mm** expected **[V]**; **flexibility**: WNG indexes 3 stiffnesses (*Flex 1* = the stiffest wooden shank, *Flex 2* = the stiffest Renner, *Flex 3* = the Renner average, over **>100 sets measured**) **[V]**; head+shank vibration mode **~260 Hz**, audible from about A4 upwards **[V]** |
| Rouleau (= noix de manche) | knuckle | Interface jack/lever ↔ hammer | C | **Distance to the centre pin: 17.0 mm standard; 16.2 / 16.0 / 15.5 mm also common; WNG: 15 → 19 mm in 0.5 mm steps**; **Ø 10.0 mm standard, 9.5 mm frequent, WNG 8/9/10/11 mm** **[V]**; leather + buckskin, lubrication (talc/PTFE) |
| Tête de marteau + feutre | hammer head + felt | Striking mass | — | Mass, "bore distance" (drilling distance), "hang distance", Shore A hardness, density of the moulding core (mahogany/walnut/soft maple vs birch/hard maple/hornbeam → leverage on the mass) **[V]** |
| Queue de marteau | hammer tail | Checking surface | C | Profile, friction |
| Manchon / garniture de l'axe | center pin bushing | Bearing | C | Friction (cf. §2.5) |

## 1.5 Damper

| FR | EN | Role | DOF | To be parameterised |
|---|---|---|---|---|
| Bascule (ou levier) d'étouffoir | damper (under)lever | Lever lifted by the key tail or by the lift tray | **R** | Mass, under-lever leads (WNG standard: **2 leads on 32 bass notes, 1 front lead on 12 notes, 1 rear lead on 12 notes, 0 thereafter**) **[V]** |
| Fourche de bascule + fourche supérieure | underlever flange + top flange | Axes | 0 | Friction (cf. §2.5) |
| Pilote/vis de bascule d'étouffoir | underlever capstan | Aligns the **line** of the under-levers | 0 adjustable | Height; **is not** the tool for the collective lift **[V]** |
| "Cuillère" de bascule | underlever spoon | Interface with the lift tray | C | Centre→tip length: WNG **73.6 / 76.6 / 81.6 mm** **[V]** |
| Languette de sostenuto | sostenuto tab / lip | Engages on the sostenuto rod | 0 | Height (2 WNG heights: tall = US Steinway replacement, because the sostenuto is mounted on the action frame) **[V]** |
| Fil d'étouffoir | damper wire | Transmits to the head block | 0 adjustable (4 bends) | 4 bends: (1)(2) take up the misalignment under-lever ↔ bushing, (3)(4) verticality/centring of the head; deliberate bearing on one side of the bushing (efficiency gain ≈ ×2 on the horizontal excursion) **[V]** |
| Bloc / bouton d'étouffoir + vis | damper block / stem block + damper wire screw | Fixes the height of the wire in the under-lever → **individual lift timing** | 0 adjustable | **This is the damping-timing parameter** **[V]** |
| Tête d'étouffoir + feutres | damper head + felts | Damps | — | Type: **wedge** (3-string unisons), **flat** (flat/block, 1- and 2-string, and treble), mixed **wedge+flat**; 3 families according to 1/2/3 strings **[V]** |
| Barre/plateau de levée des étouffoirs **[?]** | damper lift rail / underlever tray | Collective lift (damper pedal) | **T**/R | Rest height, lost motion |
| Tige de sostenuto | sostenuto rod / blade | Blocks the tabs | **R** (rotation of the rod) | Height, distance to the tabs, rotation (angle of the blade) |
| Ressort d'étouffoir (optionnel sur queue) | damper assist spring | Assists the fall | — | Preload (WNG option, requires the thick rail) **[V]** |

> **[?]** I have not found a standardised French term for the *underlever tray* on a grand piano. "Tringle de forte" / "barre de forte" is attested (Nebout & Hamm gives *damper rod* ↔ **tringle de forte**, Ger. *Abhebestange*) but corresponds rather to the upright **[V]**. "Barre de levée des étouffoirs" or "plateau de levée" are plausible translations **[S]**.

## 1.6 Pedals, lyre, trapwork

| FR | EN | Role | DOF | To be parameterised |
|---|---|---|---|---|
| Lyre + balustres | lyre + lyre braces | Support | 0 | — |
| Pédale forte (droite) | damper / sustaining pedal | Lifts all the dampers | **R** | Travel, engagement point (**lifts the dampers at 1/3 of its travel** **[V]**), lost motion |
| Pédale de sostenuto (milieu) | sostenuto pedal | Holds the dampers already lifted | **R** | Travel, stop (blocking felt under the tray) |
| Pédale una corda / de décalage **[?]** | soft / shift / una corda pedal | **Translates the key frame** | **R** → T | Travel, shift stop screw in the right cheekblock |
| Tringles / mouvements / bascules de pédalier | pedal rods / trapwork levers | Transmission | R/T | Lengths, clearances, blocking felts |
| Tête de tringle (capstan) | pedal rod head / capstan | Fine adjustment of the travel | 0 adjustable | Thread |
| Pitman / goupille de sostenuto | pitman | Vertical transmission | T | Clearance (Kawai: **2 mm** paper punching between the pitman and the lift tray during regulation) **[V]** |

> **[?] Term to be validated**: the French reference document (Valdeyron's translation of Randy Potter) names the una corda **"pédale de transposition"** **[V]**. One also finds "pédale de décalage", "petite pédale", "pédale douce". Three candidates for the FR identifier.

---

# 2. All the regulation settings (regulation specs)

## 2.1 Master table of the keyboard and action regulation settings

| # | FR | EN | Operational definition (how it is measured) | Typical value / range | Zone | Breaks / depends on |
|---|---|---|---|---|---|---|
| 1 | Hauteur de touche (blanches) | key height | From the keybed (or from the underside of the keyslip) to the underside of the lip of the keytop, key at rest | Steinway NY **2 19/32" = 65.88 mm** (ch.1) or **2 21/32" = 67.47 mm** (model sheet) **→ discrepancy [V]**; Hamburg **63 mm** (S,M,L,O,A,B) / **65 mm** (C,D); Yamaha **64 mm** (+1/−3); Kawai **66 mm** or **68 mm** depending on the model | key leveling crowned ~1/32" at the centre (Steinway) | Conditions the key dip, the aftertouch, the damper lift timing |
| 2 | Hauteur des dièses | sharp height | Above the plane of the naturals | Steinway **1/2" = 12.7 mm**; Kawai **12.5 mm**; Yamaha **12 mm** (+0.5/−0) | — | If too low → sharps "drowned" at key dip |
| 3 | Dressage / niveau | key leveling | Straightedge + paper punchings under the balance rail punching | ±0.1–0.2 mm **[S]**; deliberate crown at Steinway | — | The basis of everything |
| 4 | Équerrage / jours des touches | squaring / spacing | Twisting the balance rail pin, bending the front rail pin | — | — | Lateral friction |
| 5 | Jeu des mortaises d'enfoncement | front bushing play | Lateral movement at rest **and** depressed | Steinway **≤ 1/32" = 0.79 mm**; Kawai **0.2–0.3 mm** | uniform | Friction, noise |
| 6 | Jeu des mortaises/trous de balancier | balance bushing & hole play | Slight lateral play; free-fall test of the key tail raised by 1–2 mm | "slight play", no fore-and-aft play | uniform, a function of humidity | Friction (F) |
| 7 | **Chasse** (distance de frappe) | **blow distance** | Underside of the string → top of the hammer head, hammer at rest; **set by the capstan** | Steinway NY **1 3/4" = 44.45 mm** (S,M,L,O,A,B) / **1 7/8" = 47.63 mm** (C,D); Hamburg **47 mm**; Yamaha **48 mm**; Kawai **46 mm**; generic range **42–47 mm**; FR: "45 mm on average", "47–49 mm" | **set section by section** because the string height varies | Determines the power; **more blow distance = less aftertouch**; interacts with the key dip |
| 8 | Ligne des marteaux | hammer line | Horizontal alignment of the heads at rest (straightedge on the hammer flanges, lid in the background) | — | per section | Cosmetic + acceleration; **raising the line = more aftertouch** |
| 9 | Hauteur de la barre de repos des marteaux | hammer rest rail height | Below the shanks at rest | Kawai **5 mm** | — | Must leave the weight on the capstans |
| 10 | Position du bâton sous le rouleau | jack (fly) position / alignment | Rear edge of the jack aligned with the rear edge of the **core** (spline) of the knuckle | "aligned" | uniform | **Too far forward** = let-off too early, little control, misses; **too deep** = friction, poor repetition **[V]** |
| 11 | Centrage du bâton dans la fenêtre du levier | jack centering | Visually, traveling shim under the flange | centred | — | Noise, friction |
| 12 | Hauteur du bâton / levier de répétition | jack height to repetition lever | Top of the jack **slightly below** the surface of the lever | Yamaha **0.2 mm**; Kawai **0.1–0.2 mm**; Steinway: final adjustment by "rolling the flies" — a slight drag is felt as it passes under the knuckle, the hammer must "wink" | uniform | **Too low** = the hammer falls and does not come back up; **too high** = the hammer does not move when the jack heel is pressed; creates/removes the **lost motion at the capstan** |
| 13 | **Échappement** | **let-off** | Height of the hammer below the string at the instant the jack leaves the knuckle; set by the let-off button | Steinway NY **1/32"–1/16" = 0.79–1.59 mm** (ch.2) or **1/16"** (sheet); Hamburg **1.5 mm** bass / **1.0 mm** mid + treble; Yamaha **3 mm** bass → **1.5 mm** treble; Kawai **2.0–2.5 mm** bass, **1.0–1.5 mm** mid/treble; generic **3 → 1.5 mm**; concert hall **2 / 1.5 / 1**; FR **1–3 mm** | **yes, decreases towards the treble** (greater string excursion in the bass) | **To be done in the piano**; conditions the ppp; too far = loss of power + no control in pp; too close = blocking, hard tone, risk of breaking a string |
| 14 | **Chute** (ou rechute) | **drop** | Height of the hammer after the fall that follows let-off, key **fully down** | **⚠ two definitions**: *absolute from the string* → Hamburg **2.0 mm**, Yamaha **let-off + 2.0 mm**; *relative below the let-off* → Steinway NY **≤ 1/16" = 1.59 mm**, Igrec **~1.5 mm**; Kawai: adjust so that the drop screw touches the lever **at the moment** the jack touches the button → the initial drop **equals the let-off** | follows the let-off | **Must never "eclipse" the let-off** (Steinway: check that the drop is not set so high that it interferes); too much drop = loss of the 2nd let-off |
| 15 | **Enfoncement** (blanches) | **key dip** | Total travel of the front of the key, measured with the dip block, at constant pressure | Steinway factory **0.400" = 10.16 mm**, range **0.390–0.420" = 9.91–10.67 mm**; "13/32" = 10.32 mm the most common"; Hamburg **9.9–10.6 mm**; Yamaha **10 mm**; Kawai block **10.1 mm**; generic **9.3–10.6 mm**; FR "~10 mm" | uniform | **Finger pressure changes the measurement** (light = deeper); the cheekblocks must be tight |
| 16 | Enfoncement des dièses | sharp dip | **By feel**: the aftertouch sensation of the naturals is duplicated | Kawai: sharps **~2 mm** above the naturals at the bottom of the keystroke | — | The sharps must not drown |
| 17 | **Gras / jeu après échappement** | **aftertouch** | Remaining travel of the front of the key from the highest point of the hammer (let-off) down to the bottom, moderate pressure | Steinway **3/64"–1/16" = 1.19–1.59 mm** (ch.2) or **0.040–0.055" = 1.02–1.40 mm** (sheet); Hamburg **0.8–1.2 mm**; Igrec **≈ 0.040" = 1.0 mm**, obtained **0.8–1.5 mm**; BYU **1.25–1.5 mm** | uniform (by feel) | **Set by the key dip and/or the blow distance — never by the let-off nor the drop** **[V]**; very sensitive to the crown of the keybed → **to be set in the piano** |
| 18 | Course libre du bâton en fond | jack free travel | Some free travel must remain between the jack and its stop felt, key fully down | "a little" | — | Guarantees that the jack is properly clear |
| 19 | **Attrapé** | **checking / back check distance** | Height of the head below the string when the hammer is checked, key held | Steinway NY **1/2"–5/8" = 12.7–15.9 mm**; Hamburg **15 mm**; Kawai **15 mm**; Yamaha **16 mm** bass → **14 mm** treble; Igrec **12–15 mm**; generic **12.7–16 mm**; FR **10–15 mm** | slight bass→treble gradation (Yamaha) | Too high/too close = the tail rubs or blocks in ff; too low = long return time, slow repetition |
| 20 | Alignement/équerrage/pente des attrapes | backcheck squaring, alignment, angle | Parallel pliers / bending pliers; test by rocking key + hammer | "square to the tail", back parallel to the hammer line | — | If not square: wear of the shank-axis bushings and of the back check skin |
| 21 | Tension des ressorts de répétition | repetition spring tension | Play normally, check, release **slowly**: the hammer must rise steadily until the lever is stopped by the drop screw | No published numerical value **[V]**; criterion: a rise that is **steady and quick but not abrupt**; **slower in the bass, progressively faster towards the treble** | **yes, graduated** | Adjusted by twisting the **coil** only; re-checked **after** the back checks and **in the piano** |
| 22 | Vis de réglage de portée du châssis | balance rail studs / glides | "Knock" test: lower until the knocking appears, then until it disappears; check with the una corda pedal depressed and with the damper pedal | just touching, **without lifting** the rail | — | Kawai order: tenor break → bass → re-check → towards the treble; **must not create key dip** |
| 23 | Barre de contre-touche | key stop rail | Clearance above the keys | Steinway **~1/16" = 1.59 mm** | — | — |
| 24 | Décalage una corda | shift / keyframe shift | Shift screw (treble cheekblock or rail): in the shifted position, the left string of the 3-string unisons is no longer struck | Kawai: strikes only **2 strings out of 3**; Yamaha (reported): the left side of the hammer must **graze** the left string; Steinway: a partial shift is acceptable (changes the colour rather than the volume) **[V]** | tenor + treble only | **No dimension in mm found** → see *open questions* |
| 25 | Point de frappe / position du châssis | strike point / keyframe position | Kawai: hammer 88 strikes **2.16–3.0 mm** from the capo d'astro, final position chosen **by ear** | 2.16–3.0 mm | — | Shimming/planing of the stop block |
| 26 | Jours / mise des marteaux sur les cordes | spacing hammers to strings | Kawai: 3-string unisons → the hammer **overlaps the left string by 1 mm**; Steinway: off-centre towards the treble, **~2× more felt exposed on the treble side** | — | mono/bi: centred | Conditions the una corda voicing |
| 27 | Défauchage / fauchage | traveling / burning shanks | Lift a group with a straightedge, observe the lateral drift; paper shim under the hammer flange | straight vertical movement | — | Mandatory prerequisite to everything |
| 28 | Portée du châssis | bedding the keyframe | "Knock" test with the fingers on the front rail/keybed | no knocking | — | To be redone according to the humidity |

## 2.2 Damper regulation settings

| FR | EN | Definition | Value | Dependencies |
|---|---|---|---|---|
| Départ individuel (par la touche) | individual damper lift timing | Position of the hammer when the damper head starts to move; set by the **depth of the wire in the under-lever** | **At 1/2 of the blow distance**: Kawai **23 mm**, Yamaha **24 mm** (±1); Steinway: the key tail picks up the under-lever at **~1/8" = 3.18 mm** of key dip → the damper starts at **~1/3** of the hammer travel; **acceptable range 1/3 to 1/2** | Depends on **blow distance AND key dip**; late timing = lighter touch; early timing = more "sostenuto"; **too light interferes with the let-off** **[V]** |
| Ligne des bascules | underlever line | Straightedge laid on the under-levers; set by the underlever capstans | straight and parallel to the keybed | the capstans **are not used** for the collective lift |
| Départ général (par la pédale) | collective lift | All the under-levers start together; set by the **wire block screws**, 4 reference notes marked in pencil at 23 mm | — | Detection: "flutter" reveals the early ones, "squeeze" reveals the late ones **[V]** |
| Jeu du plateau de levée | lift tray lost motion | Clearance between the tray and the line of the under-levers at rest | Steinway **~1/8" = 3.18 mm**; Redekop **~2 mm**; Kawai **1–2 mm** (set by the pedal rod head) | — |
| Barre de butée des étouffoirs | damper (up)stop rail | Clearance remaining when the head is lifted by hand, sharp depressed | Steinway **~1/16" = 1.59 mm** treble, **~3/32" = 2.38 mm** bass; Kawai **1–2 mm**; Redekop **0.5 mm** sharps / **1 mm** naturals | Must be set **after** the key dip **[V]** |
| Course de la pédale forte | damper pedal travel | Blocking felt under the tray | The pedal lifts the dampers **at 1/3 of its travel**; the pedal lift must reach **the same height** as the lift by the sharps (Kawai stop capstan) but stay **below** the stop rail | If the pedal lifts further → the dampers bounce |
| Hauteur de levée de la tête | damper lift height off strings | — | **No standardised spec**; Schimmel **3.5–4 mm**; in practice the flat dampers rise **~8 mm** **[V]** | — |
| Alignement du fil, 4 coudes | damper wire bends 1–4 | Verticality, centring, simultaneous front/back seating | **Light** bearing on one side of the bushing: bass → treble side, treble → bass side (Steinway) | In the bass, loading the guide doubles the horizontal efficiency |
| Assise / seating | damper seating | Tap the front and the back of the head: the same "sponginess" | the back lifts **very slightly before** the front (Kawai) | String leveling is indispensable |

## 2.3 Sostenuto

| Setting | Value | Source |
|---|---|---|
| Rotation of the rod (blade visible towards the tabs), at rest | **~1 mm** of blade protrudes, seen from above through the strings | Steinway Hamburg **[V]** |
| Height: tabs below the rod | **~1.5 mm** below the rod | Steinway Hamburg **[V]** |
| Distance rod ↔ tabs at rest | **~2 mm** | Steinway Hamburg **[V]**; Redekop also gives ~2 mm |
| Angle of the blade | Redekop: **45° at rest / 90° depressed** — then, further down in the same document, **45° played / 90° at rest** → **internal contradiction [?]** | Redekop |
| Pedal travel | The damper must rise **slightly higher** than with the damper pedal, **without** jamming against the stop rail | Steinway **[V]** |
| Height of the rod | Kawai: **slightly higher** than the tabs; angle adjusted by the capstan under the tray; the pedal must hold the dampers **at the same height as the key** | Kawai **[V]** |
| Functional tests | (a) damper pedal depressed + sostenuto, release the damper pedal → all hold; (b) release the sostenuto slowly → all fall back; (c) sostenuto held + forte attack on the notes not held → **no catching** | Steinway + Kawai **[V]** |

## 2.4 Touchweight quantities — this is the quantitative core

### Static definitions and equations **[V]**

| FR quantity | EN | Symbol | Definition | Value |
|---|---|---|---|---|
| Poids descendant | downweight | **Pd / D** | Mass placed on the **front edge** of the key that makes it go down as far as the jack-heel / let-off-button contact. Dampers **out of circuit** (pedal depressed or action on the bench) | **50–60 g** (Spurlock); Steinway Hamburg **47 g** all sections (S–B), **48–52 g** (C,D); Steinway NY **46–51 g** graduated bass→treble; FR "~50 g" |
| Poids ascendant | upweight | **Pa / U** | Maximum mass that the key will **lift** from the partially depressed position | **≥ 20 g** (minimum for good repetition, narrow margin); Steinway **20 g +** all models; poor repetition below **~18 g** |
| Frottement | friction | **F** | **F = (Pd − Pa)/2** | **12–15 g** in the bass → **8–10 g** in the extreme treble; evening-out tolerance **± 1 g** |
| Poids d'équilibre | balance weight | **BW** | **BW = (Pd + Pa)/2** | measured example: **36 g** (natural), **38.5 g** (sharp) |
| Poids avant | front weight | **FW** | Static weight of the levelled key, balanced on its pin, measured at the measurement point | Stanwood recommended ceiling: **41.3 g (note 1) → 3.7 g (note 88)** |
| Poids de frappe | strike weight | **SW** | Weight of the shank + hammer assembly, pivoted at the centre, measured at the radius of the strike line | Normal zone **8.0–14.8 g (note 1) → 3.5–6.9 g (note 88)** |
| Rapport de poids de frappe | strike weight ratio | **R** | **R = SBW / SW** | Normal zone **5.0–7.0**. Zones: *low* 5.0–5.7 (high leverage), *medium* 5.7–6.3, *high* 6.3–7.0 (low leverage) |
| Poids du chevalet | wippen weight | **WW** | Levelled wippen, pivoted at the centre, measured at the capstan/heel contact | example **14.6 g** |
| Rapport de touche | key weight ratio | **KR** | Downward force at the capstan / upward force at the measurement point | **≈ 0.50–0.53** (0.50–0.52 targeted with a 17 mm knuckle); sharps **≈ 0.59** |
| Poids de frappe du manche seul | shank strike weight | **SS** | — | generic average **1.7 g** |

**Stanwood's balance equation (1996) [V]**:
`BW + FW = (WW × KR) + (SW × R)`
that is, with `TBW = BW + FW` (top action balance weight), `WBW = KR × WW`, `SBW = SW × R`:
`TBW = WBW + SBW` and `FW = (WW × KR) + (SW × R) − BW`

**Lever-by-lever calculation chain (Grandwork method) [V]**:
`HR = SL·HW + SS` → `WR = WL·HR + WW` → `DW = WR·KL`, then `BW = (DW+UW)/2`, `F = (DW−UW)/2`
(example: HW 8.9 g, SS 0.7 g, WW 14.6 g, UW 20 g → HR ≈ 58.2 g, WR ≈ 98.4 g, **DW ≈ 52 g**, BW 36 g, F 16 g)

### Sensitivities (very useful for the UI sliders) **[V]**
- **+1 g of hammer weight → +5 to +7 g on Pd *and* Pa.**
- Moderate filing removes **~0.25 g** → **−1.5 g** on Pd/Pa. After several filings, **>1 g** lost → **−5 to −10 g** of touchweight.
- A 50-year-old hammer can give a touchweight **10 g lighter** than the original.
- **Adding lead lowers Pd *and* Pa**, and **increases the inertia**: it is the wrong remedy if the leading is already average.
- An abnormally **low** friction on a single note = **loose** shank axis (reliable signature).
- Touching the knuckles with a finger adds **~2 g** of temporary friction.

### Hierarchy of the sources of friction (decreasing order) **[V]**
**knuckle > hammer pivot > wippen pivot > key bushings > capstan.** Friction is higher in the bass for two reasons: tighter shank axes (for an equal number of oscillations, a heavy hammer requires a tighter pivot) **and** greater knuckle/lever contact force.

## 2.5 Friction of the axes (centres) — numerical values

**Steinway factory standard**: all centres between **1 and 4 g of rotational friction**, minimal lateral play, measured with a gram gauge applied to the hammer flange/flange; Steinway **recommends the low end of the range** **[V]**. An action contains **350 to 600 centres** **[V]**.

**Igrec's table (*Pianos Inside Out*, Table 5)** — torque = force × distance **[V]**:

| Part | Distance from the centre | Friction (g) | Torque (mm·g) |
|---|---|---|---|
| Hammer shank flange | 25 mm | 1.5–4.0 | 37–100 |
| Repetition lever (spring disengaged) | buckskin at the drop screw (58 mm) | 2.0–6.0 | 120–360 |
| Wippen flange | 25 mm | 1.5–4.0 | 37–100 |
| Damper under-lever flange | 25 mm | 1.0–4.0 | 25–100 |
| Under-lever top flange | 25 mm | 1.0–4.0 | 25–100 |
| Sostenuto tab (spring disengaged) | tip (15 mm) | 1.0–6.0 | 15–90 |
| Jack (spring disengaged) | tip of the heel (25 mm) | 0.5–4.0 | 12–100 |
| Upright damper lever | ~50 mm | 1.0–8.0 | 50–400 |

Allowable deviation between the two directions of rotation: **≤ 50 mm·g**; beyond that, look for debris/splinters **[V]**. WNG measures the hammer flanges at **20 mm**, Steinway at **~32 mm** → hence the value of storing a **torque**, not a force **[V]**.

---

# 3. Leverage ratios and geometry (the foundation of the physics engine)

## 3.1 The 10 canonical geometry parameters (Reyburn sheet) **[V]**

This is, to my knowledge, **the minimal sufficient set** published for analysing a grand action. I recommend adopting it as it stands as the input to the model.

| # | Parameter | Precision | Example / expected value |
|---|---|---|---|
| 1 | Key front → centre of the balance hole | 0.5 mm | **259.5 mm** (example) |
| 2 | Key front → capstan lift point | 0.5 mm | **398 mm** (example) |
| 3 | Underside of the key → top of the capstan | 0.1 mm | — |
| 4 | Height of the key shoe (if external) | 0.5 mm | **4–6 mm**, otherwise 0 |
| 5 | Wippen centre → back of the jack | 0.1 mm | e.g. 95.25 mm; **WNG standard: 99 ± 1 mm** |
| 6 | Wippen centre → capstan lift point | 0.1 mm | e.g. 66.3 mm; **WNG heel 50.0–69.5 mm** |
| 7 | Shank axis → hammer centre line ("hang") | 0.1 mm | e.g. 133.4 mm |
| 8 | Shank axis → centre of the knuckle | 0.1 mm | **17.0 mm** usual; also 16.2 / 16.0 / 15.5 |
| 9 | Strike / boring distance ("bore") | 0.1 mm | e.g. 50.8 mm; otherwise = string height − shank axis height |
| 10 | Ø of the knuckle | 0.1 mm | **10.0 mm**; also 9.5 mm |
| (bis) | Thickness of the shank at the axis | — | **6 mm** expected |

Plus, in addition: the **"spread"** = distance from the wippen centre pin ↔ shank centre pin — **Yamaha G1–C7: 112.5 mm** **[V]**.

## 3.2 The action ratio: three definitions, three values

| Method | Formula | Result |
|---|---|---|
| **Useful displacements** (BYU) | `(blow distance − let-off) / (key dip − aftertouch)` | e.g. (45−2)/(10−1.5) = **5.06 : 1**; the naive 45/10 = 4.5 is wrong |
| **Product of the levers** | `AR = KR × WR × SR` | **AR ≈ 5**; best domain **4.9–5.1**; KR ≈ 0.5, WR ≈ 1.4, SR ≈ 6–6.5 |
| **Field measurement** (6 mm gauge) | `hammer travel / 6 mm` | **5 to 6.2**, most of them **5.3–5.9** — a **higher** value because it is measured at the start of the travel |

**Key point for the engine: the ratio is not constant.** It starts at **~5.5 : 1** and falls towards **~5.0 : 1** at let-off **[V]**. Modern actions are globally between **4.9 and 5.3** **[V]**.

**Knuckle → effective lever arm conversion** (Ø 10 mm; for Ø 9.5 mm subtract 0.30 mm) **[V]**:
| axis→knuckle core distance | effective arm |
|---|---|
| 15.5 mm | 20.2 mm |
| 16.2 mm | 20.8 mm |
| 17.0 mm | 21.4 mm |

Cross-check: the Grandwork example gives the knuckle at **18.0 mm** from the centre and **13.0 mm** in height → hypotenuse **22.1 mm** — consistent **[V]**.

## 3.3 Regulation trade-offs imposed by the ratio **[V]**

| Ratio | Consequences |
|---|---|
| **≤ 5** | Few weight problems, but **very deep key dip** and/or **short blow distance**; the sharps have to be levelled very high so they do not drown; reduced power |
| **5.3–5.9** | Normal domain |
| **≥ 6.2** | Heavy touch unless the hammers are very light; if compensated with lead → **excessive inertia**; best compromise: less lead and accept **Pd 60–65 g** |

**Two levers for changing the ratio**: move the **capstan** (the simplest) or change the **knuckle ↔ shank axis distance** **[V]**.

## 3.4 Half-stroke line ("half-stroke line" / "magic line" / "line of centres") **[V]**

Design condition: **the balance point of the key, the capstan/heel contact point, and the centre of rotation of the wippen must be aligned when the key is at half-stroke.** Objectives: minimise the capstan/heel contact force (hence the friction) and make the two lever arms vary in a concordant way. Spurlock tolerates **±2 mm** relative to the line. Since the balance point of the naturals and that of the sharps differ, there are in practice **two lines**: the heels of the sharps are raised by **~+2 mm** **[V]**. Possible corrections: wippen heels, Ø of the knuckle, boring of the hammers, **height of the action brackets on the key frame** **[V]**.

## 3.5 Comparative table of the manufacturer presets (to be implemented as it stands)

| Dimension | Steinway NY (S,M,L,O,A,B / C,D) | Steinway Hamburg (S–B / C,D) | Yamaha G1–C7 | Kawai RX/SK/GX/EX |
|---|---|---|---|---|
| Natural height | 2 19/32" = 65.88 mm **or** 2 21/32" = 67.47 mm **[?]** | 63 / 65 mm | 64 mm (+1/−3) | 66 or 68 mm |
| Sharps above | 1/2" = 12.7 mm | — | 12 mm (+0.5/−0) | 12.5 mm |
| Key dip | 0.400" = 10.16 mm (0.390–0.420"); 13/32" common | 9.9–10.6 mm | 10 mm | 10.1 mm (block) |
| Blow distance | 1 3/4" = 44.45 mm / 1 7/8" = 47.63 mm | 47 mm | 48 mm | 46 mm |
| Let-off | 1/32–1/16" = 0.79–1.59 mm | 1.5 / 1.0 / 1.0 mm | 3 → 1.5 mm | 2.0–2.5 / 1.0–1.5 mm |
| Drop | ≤ 1/16" **below** the let-off | 2.0 mm (absolute) | let-off **+ 2.0 mm** | = let-off (initial drop) |
| Aftertouch | 0.040–0.055" = 1.02–1.40 mm (or 3/64–1/16") | 0.8–1.2 mm | — | by feel |
| Checking | 1/2–5/8" = 12.7–15.9 mm | 15 mm | 16 → 14 mm | 15 mm |
| Damper lift timing | 1/8" of key dip → ~1/3 of the blow distance | id. | 24 mm (±1) | 23 mm |
| Lever/jack height | "slightly above" + *rolling the flies* | id. | 0.2 mm | 0.1–0.2 mm |
| Jack position | outer edge aligned with the knuckle insert | id. | rear edge aligned with the rear edge of the core | id. |
| Spread | — | — | 112.5 mm | — |
| Pd / Pa | 46–51 g graduated / ≥20 g | 47 g (48–52 C,D) / ≥20 g | — | — |

---

# 4. Operating order of regulation (basis of the "training" mode)

## 4.1 French reference sequence — *The Piano Action Handbook* (Randy Potter School), translation by M. Valdeyron 1999, 53 steps **[V]**

**Preparing the piano**: 1 remove the action · 2 clean the interior · 3 tighten the screws
**Preparing the action**: 4 file the hammers · 5 repairs · 6 clean the action · 7 key frame: bedding the front rail · 8 key frame: bedding the balance rail · 9 keytops · 10 key bushings · 11 key spacing / alignment · 12 front mortise play · 13 balance mortise play · 14 balance hole play · 15 damper guide play · 16 traveling the hammers · 17 burning the hammer shanks · 18 align action / piano (struck) · 19 string spacing · 20 spacing the hammers to the strings · 21 check the spring tension (indispensable minimum) · 22 check let-off / drop (indispensable minimum)
**Action regulation** (first regulate 3 to 5 reference notes with steps 23–46 in order to check the dimensions): 23 level the naturals · 24 level the sharps · 25 wippen / knuckle / capstan alignment · 26 jack / repetition lever centring · 27 jack / knuckle alignment · 28 jack / repetition lever height · 29 **blow distance** · 30 hammer stop rail height · 31 **let-off** · 32 **drop** · 33 **key dip, naturals** · 34 sharp dip · 35 check the **aftertouch** · 36 back check alignment · 37 back check slopes · 38 **checking** · 39 repetition spring tension · 40 key stop rail · 41 dampers — individual lift timing (key) · 42 dampers — collective lift (pedal) · 43 damper pedal · 44 damper stop rails · 45 sostenuto · 46 shift pedal (una corda)
**Voicing**: 47 fine tuning 440 Hz · 48 hammer seating/impregnation · 49 pre-voicing · 50 voicing finish
**Final check**: 51 repetition · 52 touch of individual keys · 53 test play
**"Quick regulation" (4 steps)**: blow distance (29) → springs (39) → let-off (31) → checking (38)
**"Fine regulation check"**: 29 blow distance/keyboard level, 31 even let-off, 32 even drop, 33/34 key dip, 35 even aftertouch

> This is **the** skeleton to implement for the training mode: it is in French, it is ordered, it distinguishes preparation / regulation / checking, and it already provides two short paths ("quick regulation" and "fine check").

## 4.2 Kawai sequence, 24 steps **[V]**
1 tighten the frame screws · 2 strike point & key frame position · 3 easing the keys (front mortises then balance) · 4 bedding the key frame · 5 traveling & hammer angle · 6 spacing the hammers to the strings · 7 alignment of levers/knuckles and wippens/capstans · 8 repetition spring · 9 jack alignment & height · 10 key height/squaring · 11 blow distance · 12 hammer rest rail · 13 let-off · 14 drop · 15 key dip (naturals then sharps) · 16 back checks (angle, alignment) · 17 checking (back stop, 15 mm) · 18 repetition spring (re-check) · 19 balance rail glides · 20 dampers · 21 damper stop rail · 22 damper pedal · 23 sostenuto · 24 una corda

## 4.3 Steinway sequence (chapter 1, then 2) **[V]**
**Preparation (ch.1)**: preliminaries → bedding the key frame → balance rail glides → easing the keys → leveling the naturals → sharp height → servicing the centres (friction 1–4 g) → traveling → burning → spacing the hammers to the strings → spacing the wippens → spacing the jacks.
**Regulation (ch.2)**: jack position → height of the *balancier* (repetition lever) → blow distance / hammer line → let-off → drop → key dip → sharp dip → aftertouch → back checks → repetition springs → key frame shift screw.
**Dampers & sostenuto (ch.3)**: regulating the damper "touch" (individual lift timing) → bends below the guide rail → bends above it → re-check of the lift → stop rail → damper pedal travel → height of the lift tray → sostenuto (alignment, tabs in line, height to the gauge, fore/aft position, travel).

## 4.4 50-point "complete shop" checklist (Jurjens) **[V]**
A grouping that is useful for the UI: **I. Keys & key frame** (1–14) · **II. Action off the key frame** (15–25: centres, traveling, re-graphiting the top of the jack and the window of the lever, rounding/filing the knuckles…) · **III. Alignments, action reinstalled** (26–30) · **IV. Touch** (31–39: jack/lever height, blow distance, let-off, drop, aftertouch, checking, spring, **weighing the 88 notes**, stop rail) · **V. Dampers & pedals** (40–50).

## 4.5 What is done "on the bench" vs "in the piano" — a direct map of the two apps

**On the bench / action removed (→ the "single key" app)** **[V]**:
jack position & centring · jack/repetition lever height · spacing of the wippens/jacks/levers · traveling & burning · hammer rest rail height · re-graphiting, lubrication of the knuckles and of the spring notches · **preliminary** adjustment of the repetition spring · **drop** (Steinway and Igrec: action removed, keyboard half on the knees or on the padded fallboard) · **most of the damper settings** ("almost all damper regulation must be done with the action out") · height of the sostenuto rod (to the gauge, action on the bench) · Pd/Pa weighing (dampers out of circuit).

**Necessarily in the piano** **[V]**:
**let-off** ("must be done in the piano for exacting regulation") · **aftertouch** ("very sensitive to the crown of the keybed/bench, to be set in the piano") · blow distance of the **sample** hammers of each section (then duplicated on the bench) · back check height of the samples · **balance rail glides** (cheekblocks tight, to be re-checked with the una corda pedal depressed 3–4 times, then with the damper pedal depressed) · re-check of the **drop** · **final inspection of the repetition springs** · collective damper lift, pedals, sostenuto (fore/aft position), una corda stop · ppp tests ("playing off the jacks").

**Documented test-bench pitfalls [V]** (to be reproduced as "artefacts" in the single-key simulator):
- keyboard half on the knees → the front rail bows, **the key dip decreases**: the key reaches the bottom before let-off;
- finger pressure changes the key dip measurement (light pressure → deeper dip);
- the rest height of the hammer **varies from one bounce to the next**: it depends on the tension of the repetition spring, on its friction against the lever, on the lever/knuckle friction and on the friction of the lever's axis.

---

# 5. Pathologies to reproduce (diagnostic engine)

I recommend defining each pathology as a **predicate on simulation observables** (strike velocity, number of string contacts, instant of loss of jack/knuckle contact, checking height, return time), **not** as a rule on the parameters — that way it "emerges" from the model, which is the whole pedagogical point.

| Symptom (FR / EN) | Simulated observable | Parameter(s) at fault | Source |
|---|---|---|---|
| **Blocking** of the hammer against the string, "pinched", hard tone | string contact maintained > threshold; near-zero velocity at impact | let-off **too close**; drop too small; jack too deep under the knuckle | **[V]** |
| **Loss of power** + impossibility of playing pp, difficult trills | reduced strike velocity; sound-emission threshold too high | let-off **too far**; blow distance too short | **[V]** |
| **Hammer "fluttering" / bobbling / "rattling"** in soft playing, involuntary double strike | ≥2 string contacts for a single attack; hammer↔jack rebound | **insufficient aftertouch** (the jack does not clear the knuckle enough); insufficient key dip; repetition spring **too strong**; centres **too loose** | **[V]** |
| **Failed double escapement** / soft repetition | the repetition lever does not lift the knuckle back up; high jack re-arming time | **drop too large**; repetition spring too weak; checking too low | **[V]** |
| The hammer **does not check** → light double/triple strike | no checking detected; the hammer returns towards the string | checking too far; wrong back check slope; worn/nicked skin | **[V]** |
| The hammer tail **rubs / blocks** on the back check in ff | non-zero tail/back check contact force during the rise | back check too close; **slope too steep** (Kawai test: tap the hammer while it is checked; if it goes all the way down, the angle is too vertical) | **[V]** |
| **Lost motion at the capstan**: loss of precision and of power | delay between the start of the key travel and the start of the hammer travel | repetition lever height too low; compaction of the felts; capstan too low | **[V]** |
| The hammer **falls and does not come back up** when the jack heel is pressed | static test | repetition lever **too low** | **[V]** |
| The hammer **does not move** when the jack heel is pressed | static test | repetition lever **too high** | **[V]** |
| **Sharps drowned** below the naturals at key dip | end-of-travel position of the sharp < natural | key dip too deep; ratio too low (sharps to be levelled higher) | **[V]** |
| **Heavy touch** despite a normal Pd | high inertia, normal Pd/Pa | heavy hammers **+ excess lead**; ratio ≥ 6.2 | **[V]** |
| **Heavy touch** with an abnormally high Pd | F > 15 g (bass) / > 12 g (treble) | friction: dirty/unlubricated knuckles, tight mortises, tight axes, neighbouring parts rubbing, wool fluff | **[V]** |
| **Light touch**, low Pd | Pd < range | very worn/filed hammers; **very loose** shank axes | **[V]** |
| **Slow repetition** | high key return time | **Pa < ~18 g**: excess lead, hammers too light, high friction | **[V]** |
| One note **much heavier** than its neighbours | local F outside the ±1 g tolerance | tight mortise **or** friction between neighbouring parts (test: depress the adjacent keys) | **[V]** |
| One note with an **abnormally low F** | local F < average | **loose shank axis** (near-certain signature) | **[V]** |
| **Dull tone / reduced sustain** with no acoustic cause | lengthened hammer/string contact time | **high friction of the shank axis** | **[V]** |
| **Late** damping → notes that linger / lighter touch | instant of damper lift-off > 1/2 of the blow distance | individual lift timing set late | **[V]** |
| **Too early** damping → loss of control, **interferes with the let-off** | the force of the damper under-lever is felt before the let-off point | individual lift timing too early / tray too high | **[V]** |
| **Damper bounce** on the pedal | oscillation of the damper head | the pedal lifts **higher** than the keys; pedal stop badly adjusted | **[V]** |
| **Catching** on the sostenuto (notes not held that stay on) | tab captured when it should not have been | rod **too close** to the tabs | **[V]** |
| Some dampers **fall back** when the damper pedal is released with the sostenuto depressed | loss of hold | rod **too far** from the tabs | **[V]** |
| **"Whoosh" / "oink"** of the wedge dampers on the 3-string unisons | felt/string friction during the lift | seating of the head, excess felt below the string line, una corda shift | **[V]** |
| **Knocking / noises** from the key frame | intermittent key frame/keybed contact | bedding of the key frame, balance rail glides, humidity (the front rail rises in dry weather) | **[V]** |
| "**Zing**" / vibration felt in the key at let-off in the bass | hammer/string contact during the slow passage through let-off | let-off too close in the bass (greater string excursion) | **[V]** |

**Diagnostic tests to implement as the "instruments" of the app** **[V]**:
"playing off the jacks" (playing on the jacks at ppp, damper pedal depressed: every note must sound) · "dropping the weight" (~100 g dropped onto the key, pedal depressed: a faint sound must come out; reduce until notes are missing) · "bump test" (4 adjacent naturals, compare the position of the *bump* within the travel) · fast double-strike test with the pedal depressed (let-off too close → hard/distorted tone on the 2nd strike) · free-fall test of the key tail raised by 1–2 mm.

---

# 6. Bilingual glossary FR ↔ EN (+ DE) and code identifiers

> Proposed convention: **English snake_case** identifiers, two namespaces — `part.*` for the parts, `spec.*` for the regulation settings, `tw.*` for the touchweight quantities, `geom.*` for the geometry.

## 6.1 Parts

| FR | EN | DE | `identifier` |
|---|---|---|---|
| Plateau de clavier / fond de clavier | keybed | Stuhlboden | `keybed` |
| Châssis de clavier | key frame | Klaviaturrahmen | `key_frame` |
| Barre de balancier | balance rail | — | `balance_rail` |
| Pointe de balancier | balance rail pin | — | `balance_rail_pin` |
| Mouche / rondelle de balancier | balance rail punching / bearing | — | `balance_rail_punching` |
| Barre d'enfoncement | front rail | Tastenfront (fronton) | `front_rail` |
| Pointe d'enfoncement | front rail pin | — | `front_rail_pin` |
| Mouche d'enfoncement | front rail punching | — | `front_rail_punching` |
| Cale papier | paper punching | — | `paper_punching` |
| Barre de repos arrière | back rail | — | `back_rail` |
| Drap de repos arrière | back rail cloth | — | `back_rail_cloth` |
| Vis de réglage de portée du châssis | balance rail stud / glide bolt | — | `balance_rail_glide` |
| Barre de contre-touche | key stop rail | — | `key_stop_rail` |
| Bloc / joue de clavier | keyblock / cheekblock | Klaviaturbacke | `keyblock` |
| Montant de mécanique | action bracket | Mechanikbacken | `action_bracket` |
| Barre des chevalets | wippen rail | — | `wippen_rail` |
| Barre des marteaux | hammer rail / main action rail | Hammerleiste | `hammer_rail` |
| Barre des boutons d'échappement | let-off rail | — | `let_off_rail` |
| **Sommier** | **pinblock / wrest plank** | Stimmstock | `pinblock` — *not* an action rail; see the warning below |
| Barre d'échappement | let-off rail / regulating rail | — | `let_off_rail` |
| Barre de repos des marteaux | hammer rest rail / cushion | Hammerleiste (Polster) | `hammer_rest_rail` |
| Touche | key | Taste | `key` |
| Touche blanche / marche | natural / white key | weiße Taste | `natural_key` |
| Dièse / feinte | sharp / black key | schwarze Taste | `sharp_key` |
| Revêtement de touche | keytop / key covering | Tastenbelag | `keytop` |
| Fronton | key front | Tastenfront | `key_front` |
| Capsule de touche | key button / chase | Bäckchen | `key_button` |
| Mortaise (de balancier / d'enfoncement) | balance / front mortise | — | `balance_mortise`, `front_mortise` |
| Garniture de mortaise | bushing (cloth) | Garnierung | `mortise_bushing` |
| Patin de balancier | key shoe | — | `key_shoe` |
| Plomb de touche | key lead | Bleigewicht | `key_lead` |
| **Pilote (de touche)** | **capstan (screw)** | **Pilote** | `capstan` |
| Queue / talon de touche | key end / tail | — | `key_tail` |
| **Chevalet (de mécanique)** | **wippen assembly** | **Hebeglied / Unterglied** | `wippen_assembly` — the whole wooden sub-assembly: body, heel, jack, repetition lever, springs. See the note below |
| Corps du chevalet | wippen (the body alone) | — | `wippen` — the single rigid body that carries one generalized coordinate |
| Chevalet (de table d'harmonie) | bridge | Steg | `bridge` — the wooden bar coupling the strings to the soundboard |

> **« Chevalet » names two different things in a grand, and the owner states both
> precisely (2026-10-11):** on the *structure harmonique*, the wooden bar that links
> the strings to the soundboard; in the *action*, **the assembly of wooden parts** —
> heel, repetition lever, jack and the rest.
>
> The second half of that is not only a naming point. The trade's unit of assembly
> is exactly the **coupled block of the mass matrix**: the wippen body, the jack
> relative to the wippen and the repetition lever relative to the wippen are the
> three coordinates that share off-diagonal terms, and `m_jr = 0` exactly — the jack
> and the lever couple to the body but not to each other. The arrowhead structure of
> `MassArrow` is therefore the *chevalet*, written as a matrix. A technician replaces
> a chevalet as one part because its pieces are inertially coupled, and the solver
> solves them as one block for the same reason.
>
> Consequence for identifiers: `wippen` is the **body alone**, since it is the body
> that carries a generalized coordinate. Anything addressing the assembly —
> a preset, a part swap, a price, a mass — is `wippen_assembly`.
| Talon de chevalet | wippen heel | — | `wippen_heel` |
| Feutre / drap de talon | heel cloth | — | `wippen_heel_cloth` |
| Fourche de chevalet | wippen flange | — | `wippen_flange` |
| Coussin de chevalet | wippen cushion | — | `wippen_cushion` |
| **Bâton (doigt) d'échappement** | **jack** (Steinway: *fly*) | **Stoßzunge / Stößer** | `jack` |
| Pied du bâton d'échappement | jack tender / heel | — | `jack_tender` |
| Ressort de bâton | jack spring | Stößerfeder | `jack_spring` |
| **Bouton d'échappement** | **let-off button / regulating button** | **Auslösepuppe** | `let_off_button` — the generic term |
| **Pilote d'échappement** | long wooden let-off dowel | — | `let_off_button` with `form: Dowel` — the Schwander form specifically, screwed to a threaded point in the underside of the hammer rail. **Not a pure synonym of *bouton*** |
| Ressort auxiliaire (de chevalet) | auxiliary / wippen support spring; Renner USA: "turbo wippen" | — | `auxiliary_spring` |
| Poids d'enfoncement, poids descendant | **downweight** | Niederdruckgewicht | `tw.downweight_g` — defined as the weight needed to depress the key slowly at the key front, **dampers raised** |
| Tige de bouton d'échappement | let-off dowel / wire | — | `let_off_dowel` |
| **Levier de répétition** | **repetition lever** (Steinway: *balancier*) | **Repetitionsschenkel** | `repetition_lever` |
| **Vis de chute** | **drop screw** | **Abstellschraube** | `drop_screw` |
| **Ressort de répétition** | **repetition spring** | **Repetierfeder** | `repetition_spring` |
| Vis de tension du ressort | repetition spring tension screw | — | `repetition_spring_screw` |
| Ressort auxiliaire de chevalet | wippen assist / helper spring | — | `wippen_assist_spring` |
| **Attrape** | **back check** | **Fänger** | `back_check` |
| Garniture / peau d'attrape | back check leather / felt | Fängerfilz | `back_check_leather` |
| Fil d'attrape | back check wire | — | `back_check_wire` |
| Contre-attrape *(piano droit uniquement)* | catcher | — | `catcher` *(upright)* |
| Lanière *(piano droit uniquement)* | bridle (tape) | Fangband | `bridle_tape` *(upright)* |
| Marteau (ensemble) | hammer assembly | Hammer | `hammer` |
| Tête de marteau | hammer head | Hammerkopf | `hammer_head` |
| Feutre de marteau | hammer felt | Hammerfilz | `hammer_felt` |
| Manche de marteau | hammer shank | Hammerstiel | `hammer_shank` |
| Olive (du manche de marteau) | hammer shank flange | Hammerkapsel | `hammer_flange` |
| **Rouleau** | **knuckle** | **Hammerrolle** | `knuckle` |
| Noyau du rouleau | knuckle core / spline | — | `knuckle_core` |
| Queue de marteau | hammer tail | — | `hammer_tail` |
| Noix (de marteau) *(piano droit)* | hammer butt | Hammernuß | `hammer_butt` *(upright)* |
| Axe / centre / pivot | center pin | Achse | `center_pin` |
| Bascule / levier d'étouffoir | damper (under)lever | Dämpferhebel | `damper_underlever` |
| Fourche de bascule (d'étouffoir) | underlever flange | — | `underlever_flange` |
| Noix d'étouffoir | damper block (clamps the wire) | — | `damper_block` |
| Tige d'étouffoir | damper wire | Dämpferdraht | `damper_wire` |
| Galerie des étouffoirs | damper guide rail | — | `damper_guide_rail` |
| Tête d'étouffoir | damper head | Dämpferkopf | `damper_head` |
| Barre de forte | damper lift rail | — | `damper_lift_rail` |
| Barre des bascules d'étouffoir | under-lever rail | — | `underlever_rail` |
| Barre de butée des bascules d'étouffoir | under-lever stop rail | — | `underlever_stop_rail` |
| Bec de tonale (de sostenuto) | sostenuto tab | — | `sostenuto_tab` |
| Barre de tonale | sostenuto rod | — | `sostenuto_rod` |
| Fourche supérieure | underlever top flange | — | `underlever_top_flange` |
| Pilote de bascule d'étouffoir | underlever capstan | — | `underlever_capstan` |
| Cuillère de bascule | underlever spoon | Dämpferlöffel | `underlever_spoon` |
| Languette de sostenuto | sostenuto tab / lip | — | `sostenuto_tab` |
| Fil d'étouffoir | damper wire | Dämpferdraht | `damper_wire` |
| Bloc / bouton d'étouffoir | damper block / stem block | Dämpferpuppe | `damper_block` |
| Vis de fil d'étouffoir | damper wire screw | — | `damper_wire_screw` |
| Tête d'étouffoir | damper head | — | `damper_head` |
| Feutre d'étouffoir (plat / coin) | damper felt (flat/block, wedge) | Dämpferfilz | `damper_felt_flat`, `damper_felt_wedge` |
| Barre de guidage des étouffoirs | damper guide rail | Dämpferführungsleiste | `damper_guide_rail` |
| Garniture de guidage | guide rail bushing | — | `damper_guide_bushing` |
| Barre de butée des étouffoirs | damper (up)stop rail | Dämpferpralleiste | `damper_stop_rail` |
| Plateau / barre de levée **[?]** | damper lift rail / underlever tray | Abhebestange | `damper_lift_rail` |
| Tige / lame de sostenuto | sostenuto rod / blade | — | `sostenuto_rod` |
| Ressort d'étouffoir | damper (assist) spring | Dämpferfeder | `damper_spring` |
| Lyre | lyre | Lyra | `lyre` |
| Pédale forte | damper / sustaining pedal | rechtes Pedal | `damper_pedal` |
| Pédale de sostenuto | sostenuto pedal | Mittelpedal | `sostenuto_pedal` |
| Pédale una corda / de décalage **[?]** | soft / shift / una corda pedal | linkes Pedal | `shift_pedal` |
| Tringle de pédale | pedal rod | — | `pedal_rod` |
| Tête de tringle | pedal rod head / capstan | — | `pedal_rod_head` |
| Mouvement / levier de pédale | trapwork lever | Wippe | `trapwork_lever` |
| Prisonnier (tourillon de transmission) | dowel to the damper lift | — | `lift_dowel` |
| Tige de pédale | pedal rod | Pedalstange | `pedal_rod` |
| Coupelle de pédale | pedal cup | — | `pedal_cup` |
| Pitman | pitman | — | `pitman` |
| Vis de décalage du châssis | keyframe shift screw | — | `keyframe_shift_screw` |
| Bloc d'arrêt du châssis | action stop block | — | `action_stop_block` |
| Dag blocks (patins arrière) **[?]** | dag blocks | — | `dag_block` |

## 6.2 Regulation settings and quantities

| FR | EN | `identifier` | Unit |
|---|---|---|---|
| Dressage (niveau) des touches | key leveling / key height | `spec.key_height` | mm |
| Hauteur des dièses | sharp height | `spec.sharp_height` | mm |
| Enfoncement | key dip | `spec.key_dip` | mm |
| Enfoncement des dièses | sharp dip | `spec.sharp_dip` | mm |
| **Chasse** | **blow distance** | `spec.blow_distance` | mm |
| Ligne des marteaux | hammer line | `spec.hammer_line` | mm |
| **Échappement** | **let-off** | `spec.let_off` | mm (from the string) |
| **Chute** | **drop** | `spec.drop_abs` / `spec.drop_below_let_off` | mm |
| **Gras / jeu après échappement** | **aftertouch** | `spec.aftertouch` | mm |
| **Attrapé** | **checking / back check distance** | `spec.check_height` | mm (from the string) |
| Pente d'attrape | back check angle | `spec.back_check_angle` | ° |
| Hauteur du bâton / levier | jack-to-repetition-lever height | `spec.jack_height` | mm |
| Position du bâton | jack position | `spec.jack_position` | mm |
| Jeu au pilote | lost motion | `spec.lost_motion` | mm |
| Tension du ressort de répétition | repetition spring tension | `spec.rep_spring_tension` | N·mm or g-equivalent |
| Hauteur barre de repos marteaux | hammer rest rail height | `spec.hammer_rest_height` | mm |
| Barre de contre-touche | key stop rail clearance | `spec.key_stop_clearance` | mm |
| Départ individuel d'étouffoir | individual damper lift timing | `spec.damper_lift_timing` | mm of blow distance |
| Départ général | collective damper lift | `spec.collective_lift` | mm |
| Jeu du plateau de levée | lift tray lost motion | `spec.tray_lost_motion` | mm |
| Butée d'étouffoir | damper stop rail clearance | `spec.damper_stop_clearance` | mm |
| Course de pédale forte | damper pedal travel | `spec.damper_pedal_travel` | mm |
| Jeu de pédale | pedal lost motion | `spec.pedal_lost_motion` | mm |
| Décalage una corda | shift travel | `spec.shift_travel` | mm |
| Hauteur / rotation / distance du sostenuto | sostenuto rod height / rotation / clearance | `spec.sostenuto_*` | mm, ° |
| Jeu des mortaises | bushing play | `spec.bushing_play` | mm |
| Portée du châssis | keyframe bedding | `spec.keyframe_bedding` | — |
| Poids descendant (Pd) | downweight | `tw.downweight` | g |
| Poids ascendant (Pa) | upweight | `tw.upweight` | g |
| Frottement (F) | friction weight | `tw.friction` | g |
| Poids d'équilibre | balance weight | `tw.balance_weight` | g |
| Poids avant | front weight | `tw.front_weight` | g |
| Poids de frappe | strike weight | `tw.strike_weight` | g |
| Rapport de poids de frappe | strike weight ratio | `tw.strike_weight_ratio` | — |
| Poids du chevalet | wippen weight | `tw.wippen_weight` | g |
| Rapport de touche | key weight ratio | `tw.key_ratio` | — |
| Inertie | inertia | `tw.inertia` | kg·m² |
| **Rapport de mécanique** | **action ratio** | `geom.action_ratio` | — |
| Rapport de touche / de chevalet / de manche | key / wippen / shank ratio | `geom.key_ratio`, `geom.wippen_ratio`, `geom.shank_ratio` | — |
| Écart des barres ("spread") | spread | `geom.spread` | mm |
| Distance rouleau ↔ axe de manche | knuckle distance | `geom.knuckle_distance` | mm |
| Ø du rouleau | knuckle diameter | `geom.knuckle_diameter` | mm |
| Distance de perçage | bore distance | `geom.bore_distance` | mm |
| Distance de pendage | hang distance | `geom.hang_distance` | mm |
| Ligne de mi-course (ligne magique) | half-stroke line / magic line | `geom.half_stroke_line` | mm of deviation |

## 6.3 Operations

| FR | EN | `identifier` |
|---|---|---|
| Défauchage | traveling (hammers) | `op.travel_hammers` |
| Fauchage / chauffage des manches | burning shanks | `op.burn_shanks` |
| Dégagement des touches | easing keys | `op.ease_keys` |
| Portée du châssis | bedding the keyframe | `op.bed_keyframe` |
| Mise des marteaux sur les cordes | spacing hammers to strings | `op.space_hammers` |
| Jours (de touches, de cordes) | spacing / clearances | `op.spacing` |
| Ponçage des têtes | hammer filing / reshaping | `op.file_hammers` |
| Harmonisation | voicing | `op.voicing` |
| Plombage | key leading / weighing off | `op.key_leading` |
| Pesée | weigh-off | `op.weigh_off` |
| Rebrochage / regoupillage | repinning | `op.repin` |
| Regarnissage | rebushing | `op.rebush` |

---

# 7. French terms from the initial brief

Resolved against standard French trade usage, with the numbered sectional plate of
a grand action as the reference. **No citation is owed and none is given**: these
are the ordinary terms of the trade in France, not any one author's coinage. What
follows records *what the vocabulary is*, and — where two words are genuinely in
circulation — which one this project uses.

| Term used in the brief | Verdict | Term adopted |
|---|---|---|
| **« sautereau »** for the *jack* | **Corrected.** "Sautereau" is the vertical slip of the harpsichord and spinet, carrying the plectrum. | **bâton d'échappement** (also *doigt d'échappement*) → `jack` |
| **« bascule »** for the *wippen* | **Corrected, and the two parts are genuinely distinct words.** The plate lists « Chevalet » for the wippen and « Bascule d'étouffoir » for the damper under-lever, as separate items. The action chapter is explicit: *"Le chevalet est le cœur de la mécanique. Il agit comme un support et un levier du bâton d'échappement"*, with a translator's note glossing *wippen* as *chevalet*. | **chevalet** → `wippen`; **bascule d'étouffoir** → `damper_underlever` |
| **« limbe »** for *action frame/rails* | **Corrected.** Not in trade use. | **barres de mécanique** + **montants de mécanique**; **plateau**, **châssis de clavier** |
| **« noix »** for a *flange* | **Corrected, and the word does exist on a grand — elsewhere.** The flanges are the **fourche de chevalet** (wippen) and the **olive du manche de marteau** (shank). « Noix » on a grand is the **noix d'étouffoir**, the block clamping the damper wire to the under-lever; the *hammer butt* sense is the upright. | **fourche de chevalet**, **olive**; **noix d'étouffoir** gains its own identifier |
| **« distance de frappe »** | Understandable, not the trade term | **chasse** |
| **« post-échappement »** | Attested synonym | **jeu après échappement**, **gras**, **réserve**. A translator's note glosses *aftertouch* as **gras** |
| **« chute »** vs **« rechute »** | Both circulate; the part is standard | **chute** for the quantity, **vis de chute** for the part |
| **« cuillère »** | **Open — see §7.1.** The plate puts a « Cuillère de chevalet » in the *wippen* sub-assembly, which is not where an earlier draft of this annex placed it. | pending the owner |
| **« una corda »** | Still open | **pédale de transposition** / **de décalage** / **douce** — to be settled |
| **« feutre, cuir, buffle »** | Correct | — |

## 7.1 What the plate settles, and the four things it does not

Settled, and carried through the glossary of §6:

| Question | Answer |
|---|---|
| let-off button | **bouton d'échappement** — the plate's term; **pilote d'échappement** appears as a synonym in the rails chapter. Both are in use; the project writes *bouton d'échappement* |
| shank flange | **olive du manche de marteau**. *"L'olive du manche ou la fourche du chevalet"* is fixed to the flat of the rail — the two flanges, named in one breath |
| under-lever tray / lift rail | **barre de forte**, and the assembly is the **barre de levée des étouffoirs**. This had no French term in the corpus at all |
| action brackets | **montants de mécanique** |
| key stop rail | **barre de contre-touche** |
| glide bolt | **vis de réglage de portée du châssis** — named after the very thing it corrects, which also confirms *portée du châssis* for frame bedding |
| key bearing | **support de touche au balancier**: a hemispherical stud on Steinway, **mouches de balancier** on other makes. The key bears on the *punching*, not on the pin — which is what `KeyBearing` models |
| sostenuto | **la tonale**: *barre de tonale*, *support de tonale*, *bec de tonale*. *Bec de sostenuto* and *barre de sostenuto* also appear |
| trapwork | **mouvement de pédale** / **levier de pédale**; the dowel onto the damper lift is the **prisonnier** |
| damper parts | **tête**, **feutre**, **tige**, **galerie**, **noix**, **fourche**, **bascule**, **plomb de bascule**, **barre de butée des bascules** |

**Still open, and genuinely so — these four are questions for the owner:**

1. **« Cuillère de chevalet ».** The plate lists it as a *wippen* part, between
   the jack regulating button and the repetition lever flange — not on the damper
   under-lever, where the upright's spoon sits and where it is easy to assume the
   grand's belongs. Which of the two placements is right cannot be settled from
   the plate, because the plate does not say what the part *does*.
2. **« Vis de chute » against « vis de réglage de la hauteur du levier de
   répétition ».** The plate carries both, as separate numbered items, and both
   govern drop. Two names for one adjustment, or two parts?
3. **« Taquet d'étouffoir ».** Appears on the Schwander action figure, in the
   damper group. Not identified.
4. **« Sommier ».** The plate uses it for the **pinblock**, but the damper chapter
   writes *"le sommier des bascules"* for the under-lever frame — which the plate
   itself calls *châssis des étouffoirs*. Two senses, or a loose usage?

And one that no sectional plate can answer, because it is about the interface and
not about the part: **« chevalet » names two different things** — the wippen
(action) and the bridge (soundboard), both in the same plate. The identifiers are
unaffected, `wippen` and `bridge`, but the French UI must choose between a prefix
on each and relying on context.

---

# 8. Direct consequences for the data model

1. **Four layers, not one.**
   `ActionDesign` (geometry, invariant) → `RegulationState` (the ~28 regulation settings, mutable, this is what the technician touches) → `MaterialSet` (contact laws: felt, leather, buckskin, cloth, back check skin, graphite/PTFE) → `GraduationCurves` (curves over the note index 1…88).
2. **Every dimension carries a range.** `{ value, min, max, unit, source_preset, section }`. The sections: `bass`, `tenor`, `mid`, `treble`, `high_treble` (this is the division used in the Steinway sheets).
3. **Two first-class units.** mm **and** fractional inches (1/32, 3/64, 1/16, 13/32…): the Steinway and Potter manuals are in inches, Yamaha/Kawai/Hamburg in mm.
4. **The action ratio is an output**, computed at each time step (it varies along the travel). Provide the 3 measures (useful displacements, product of the levers, 6 mm gauge) as *three different readings of the same model* — which is exactly what the technician experiences in the field.
5. **The drop has two representations.** Store `drop_abs` (from the string) as the truth, expose `drop_below_let_off = drop_abs − let_off` as a read-only value, and warn if `drop_abs ≤ let_off` ("the drop eclipses the let-off").
6. **Dependency graph of the regulation settings.** Each spec declares `invalidates: [...]`. Examples taken from the manuals: `blow_distance → aftertouch, damper_lift_timing, hammer_line`; `key_dip → aftertouch, damper_stop_clearance`; `let_off → drop`; `check_height → rep_spring_tension`; `key_height → key_dip, aftertouch`; `jack_height → lost_motion`; `glides → key_dip (forbidden!)`. This graph **is** the engine of the training mode.
7. **The 88 notes share the topology, not the values.** Have the graduation curve carry: hammer mass (11 g → 3.5 g), `strike_weight` (8–14.8 g → 3.5–6.9 g), `front_weight` (41.3 g → 3.7 g), friction (12–15 g → 8–10 g), let-off (3 → 1.5 mm or 1.5 → 1.0 mm), checking (16 → 14 mm), spring tension (slow → fast), damper under-lever leads (2/1/1/0).
8. **The WNG part ranges constitute an excellent validation domain**: knuckle 15–19 mm (0.5 steps), Ø 8/9/10/11 mm, heel 50.0–69.5 mm (1.5 steps), heel height 16–24 mm (1 step), standard 99 mm ± 1, spoon 73.6/76.6/81.6 mm, sharps +2 mm. These are **industrially achievable** values, and therefore good bounds for the sliders.
9. **Provide for the "bench artefacts"** as environment parameters: bowing of the front rail (keyboard on the knees), finger pressure when measuring the key dip, humidity (tightness of the mortises, bedding of the key frame). These are real sources of error that the pedagogical tool should be able to exhibit.
10. **Friction must be stored as a torque (mm·g or N·mm), not in grams**, because the manufacturers measure at different radii (Steinway ~32 mm, WNG 20 mm, Igrec 25 mm).
