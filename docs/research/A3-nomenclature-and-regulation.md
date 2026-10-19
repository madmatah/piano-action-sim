# A3. Nomenclature and regulation of the grand action

> Research annex to `docs/FOUNDATIONS.md`: the parts of the grand action and their
> names in French, English and German; every regulation setting with the makers'
> specifications; the trade's touchweight metrology and its protocols; the
> regulation sequences; the faults a technician diagnoses. Where it disagrees with
> FOUNDATIONS, FOUNDATIONS holds.
>
> **Reliability.** **[V]**: verified against a primary source in the corpus, named
> with its locator; the short names are listed in [Sources](#sources). **[S]**: assumed, to be
> measured or confirmed. **[?]**: not verified, with the reason (no source in the
> corpus, or sources that disagree); on a French term, an open question in
> `docs/vocabulary-worksheet.md`. A number whose arithmetic is written beside it
> carries no marker. A trade term carries no source: the trade's vocabulary belongs
> to the trade, and the owner settles it.
>
> Inch values from the American sources are converted to millimeters (× 25.4) and
> the original is kept beside them: fractional inches are a planned second unit
> system ([FOUNDATIONS > The acceptance criterion](../FOUNDATIONS.md#the-acceptance-criterion)).

---

## What "the" specification is not

No regulation dimension is universal. Steinway's own guide says "the specifications
given are important; however, they may vary within reason to accommodate individual
needs of the piano player", and that since "individual regulation steps affect
others, repeating operations will be necessary in achieving stability" [V, Steinway
ch. 2, p. 3; ch. 1, p. 8]. A regulation setting therefore carries **(value,
acceptable range, source, keyboard zone)**, not a constant ([FOUNDATIONS > The data model](../FOUNDATIONS.md#the-data-model),
planned), and [The makers' specifications side by side](#the-makers-specifications-side-by-side) sets the makers' specifications side by side: the basis for
per-maker presets (planned) rather than one set of defaults.

---

## Part inventory (modern grand piano)

Table format: **FR | EN | role | DOF** (degrees of freedom) **| to be parameterized**.
DOF convention: `R` = planar rotation (1 DOF), `T` = translation, `0` = fixed or
structural part (a reference frame or a stop), `C` = contact only (no body of its
own, modeled as a surface).

### Key frame, rails, structural members

| FR | EN | Role | DOF | To be parameterized |
|---|---|---|---|---|
| Plateau de clavier / fond de clavier | keybed | Reference frame of the keyboard; its flatness conditions everything | 0 | Crowned or flat. Steinway New York crowns the keybed's front lip so that its center stands ~1/32" (0.79 mm) above the ends, hollows the middle ~1/32" front to back and bass to treble so that only the balance rail studs touch at the balance rail, and reverse-crowns the key frame's front rail ~1/32"; Hamburg, Boston and Essex keybeds are **flat** [V, Steinway ch. 1, p. 10] |
| Châssis de clavier | key frame | Carries the three rails; **translates laterally** with the una corda | **T** (una corda) | Shift travel; friction on the dag blocks, graphited [V, Steinway ch. 1, p. 11]; stiffness, out of the model's scope ([FOUNDATIONS > What the model is](../FOUNDATIONS.md#what-the-model-is)) |
| Barre de balancier | balance rail | Axis of rotation of the keys | 0 | Height, longitudinal position; Steinway levels the keys with a crown, the center ~1/32" higher than the ends [V, Steinway ch. 1, p. 16] |
| Pointe de balancier | balance (rail) pin | Axis + lateral guidance | 0 | Ø 2–3 mm [?, not in the corpus]; lateral position (squaring the keys) |
| Mouche/rondelle de balancier, "balance rail bearing" (NY) | balance rail punching / bearing | Felt bearing + paper punchings for key leveling | C | Thickness of the felt, stack of paper punchings (**the** key-leveling parameter) |
| Barre d'enfoncement | front rail | End-of-travel stop + guidance | 0 | Height |
| Pointe d'enfoncement | front rail pin | Lateral guidance (oval pin) | 0 | Lateral play, at rest and depressed: Steinway **≤ 1/32" (0.79 mm)** [V, Steinway ch. 1, p. 15]; Kawai **0.2–0.3 mm**, with a small but clear "knock" [V, Kawai, p. 5] |
| Mouche d'enfoncement (+ cales papier) | front rail punching (+ paper punchings) | **Sets the key dip** | C | Felt thickness + stack of papers; Kawai: no more than 4 papers of any one thickness, the thicker ones lowest [V, Kawai, pp. 17–18] |
| Barre de repos arrière + drap | back rail + back rail cloth | Upper stop for the key tail | C | Thickness and compliance of the cloth |
| Vis de réglage de portée du châssis ("pistons", glides) | balance rail studs / glide bolts | Support the floating balance rail on the keybed | 0 adjustable | Height: just touching the keybed, never lifting the rail; **never used to set the key dip** [V, Steinway ch. 1, pp. 12–13; Kawai, p. 21] |
| Barre de contre-touche | key stop rail | Limits the upward return of the keys | 0 adjustable | Clearance: Steinway **~1/16" (1.59 mm)** above the keys [V, Steinway ch. 1, p. 17] |
| Blocs/joues de clavier | keyblocks / cheekblocks | Lock the keyboard laterally, carry the pressure screw | 0 | Pressure screw, set for light pressure on the guide pins [V, Kawai, p. 7] |
| Montants de mécanique | action brackets | Carry the rails | 0 | Height (moves the half-stroke line, [Half-stroke line ("half-stroke line" / "magic line" / "line of centers")](#half-stroke-line-half-stroke-line--magic-line--line-of-centers)) |
| Barre des chevalets | wippen rail | Axis of the wippens | 0 | Position ("spread", [Reyburn's ten measurements](#reyburns-ten-measurements)) |
| Barre des marteaux | hammer (shank) rail / main action rail | Axis of the hammers | 0 | Position |
| Barre d'échappement (barre des boutons d'échappement) | let-off rail / regulating rail | Carries the let-off buttons | 0 adjustable | Overall height |
| Barre de repos des marteaux | hammer rest rail / cushion | Rest for the shanks | 0 adjustable | Kawai: **5 mm below the shanks at rest** [V, Kawai, p. 16]; compliance of the felt |
| Barre de butée des étouffoirs | damper (up)stop rail | Limits the lift of the dampers | 0 adjustable | [Damper regulation settings](#damper-regulation-settings) |
| Galerie des étouffoirs + coulisseaux | damper guide rail + bushings | Guides the damper wires | 0 | Bore Ø (**~3 mm** for a **~1 mm** wire in Igrec's example [V, Igrec, p. 184]), friction, deliberate off-centering of the wire |

> **Settled.** « Limbe » is not the trade term for the *action frame or rails*.
> Standard French usage is: **plateau** (keybed), **châssis de clavier** (key
> frame), **barres de mécanique** (action rails: *barre des marteaux*, *barre des
> chevalets*, *barre d'échappement*) and **montants de mécanique**
> (action brackets). **The rails are « barres »**: *barre des marteaux*, *barre
> des chevalets*, *barre des bascules d'étouffoir*. An older usage, still in
> print, calls them « sommier des marteaux », « sommier des chevalets »,
> « sommier d'étouffoirs » or « sommier des bascules »; the trade says « barre »
> today, and the interface keeps **sommier** for the **pinblock** alone.

### Key

| FR | EN | Role | DOF | To be parameterized |
|---|---|---|---|---|
| Touche | key, as one lever | Input lever | **R** about the balance rail pin | Front and rear lengths ([Reyburn's ten measurements](#reyburns-ten-measurements): key front to balance hole 259.5 mm and key front to capstan 398 mm in Reyburn's example [V, Reyburn, p. 3]), mass, inertia, wood |
| Mortaise de balancier + garniture | balance mortise + bushing | Guidance at the balance pin | C | Play (a little side play at the bushing, none fore and aft at the hole [V, Steinway ch. 1, p. 14]), friction |
| Mortaise d'enfoncement + garniture | front mortise + bushing | Lateral guidance | C | Play, friction |
| Capsule de touche | key button / chase | Carries the balance mortise | 0 | — |
| Patin ("shoe") de balancier | key shoe | Extra thickness under the key (older pianos) | 0 | Typically **4–6 mm**; only an **external** shoe counts in the geometry [V, Reyburn, p. 4] |
| Plombs de touche | key leads | Counterbalance the hammers | 0 | Ø ½" or ⅜" (12.7 or 9.5 mm) usual, 10 and 12 mm in the French edition [V, Spurlock, p. 8; Spurlock FR, p. 10]; position and number, which with the key's own mass set its front weight. The bench models no lead: a key is weighed whole, its leads in it, its mass and front weight typed ([FOUNDATIONS > Parts and statics](../FOUNDATIONS.md#parts-and-statics), [FOUNDATIONS > Named simplifications](../FOUNDATIONS.md#named-simplifications)) |
| Pilote (de touche) | capstan (screw) | Transmits to the wippen heel | 0 adjustable (screwing) | **Height**, which sets the blow distance [V, Steinway ch. 2, p. 6]; longitudinal position, which sets the ratio [V, Spurlock, p. 3]; straight or angled [V, Reyburn, p. 4]; surface condition (friction) |
| Bout de touche | key end / tail | Lifts the damper under-lever | C | Key-end felt |
| Revêtement | key covering / keytop | — | 0 | Thickness; height of the sharps |
| Fronton | key front | — | 0 | — |

### Wippen (= chevalet de mécanique) and the let-off sub-assembly

| FR | EN | Role | DOF | To be parameterized |
|---|---|---|---|---|
| Chevalet (de mécanique) | wippen / whippen | Intermediate lever | **R** | Mass, inertia, "spread" ([Reyburn's ten measurements](#reyburns-ten-measurements)) |
| Talon de chevalet | wippen heel (+ heel cloth) | Receives the capstan | C | **Height**: WNG makes 9 heels, their contact surface **16 to 24 mm below the jack center, in 1 mm steps**, the sharps' heels typically **+2 mm**; **position**: 14 locations from **50.0 to 69.5 mm** from the wippen center pin, in 1.5 mm steps, standard **62.0 mm** [V, WNG, pp. 3–6] |
| Fourche de chevalet | wippen flange | Axis | 0 | Position; friction of the pivot |
| Bâton (ou doigt) d'échappement | jack / fly (Steinway) | Pushes the knuckle, then lets off | **R** on the wippen | Length, mass; fore-and-aft position under the knuckle; WNG standard, repetition flange center to jack center, 99 ± 1 mm [?, not in the WNG document of the corpus] |
| Pied du bâton d'échappement | jack tender / jack heel | Meets the let-off button | C | Position; angle of the tender (WNG uses a "low angle tender" matched to its regulating rail [V, WNG, p. 3]) |
| Ressort de bâton / ressort combiné | jack spring / combination spring | Returns the jack | — | Stiffness, preload |
| Bouton d'échappement | let-off button / regulating button | **Triggers the let-off** | 0 adjustable | **Height**; compliance of the felt or cloth |
| Levier de répétition | repetition lever / balancier (Steinway) | Carries the knuckle after let-off, lifts the hammer back up | **R** on the wippen | Mass, inertia; **height relative to the jack** |
| Vis de chute | drop screw | Stop for the repetition lever: sets the **drop** | 0 adjustable | Height |
| Vis de réglage de la hauteur du levier de répétition | repetition lever height screw | Sets the lever's height against the top of the jack, taking out the lost motion between jack and knuckle | 0 adjustable | Height; at the back of a wippen with a chevron spring, toward the middle of one with a Schwander-type spring |
| Ressort de répétition | repetition spring | Lifts the repetition lever and helps the key return [V, Steinway ch. 2, p. 14] | — | **Preload/stiffness**, adjusted through the coil, never the curve [V, Kawai, p. 12; Steinway ch. 2, p. 14]; graduated, slower in the bass, faster toward the treble [V, Steinway ch. 2, p. 15] |
| Vis de tension du ressort | repetition spring tension screw | Screwed variant: at the back of a wippen with a Schwander-type spring, at the top of the chevron on a chevron spring that has one; a chevron spring without one is bent | 0 adjustable | — |
| Attrape + garniture (peau/cuir) | back check + leather | Catches the hammer tail | 0 adjustable (bent wire) | **Height, angle ("slope"), lateral alignment, squaring**; friction coefficient of the leather |
| Fil d'attrape | back check wire | Support, adjusted by bending | 0 adjustable | — |
| Coussin de chevalet / coussin de repos | wippen cushion / rest cushion | Rest for the wippen or for the lever | C | Compliance; WNG offers a "rest rail" **or** a "cushion" variant [V, WNG, p. 2] |
| Ressort auxiliaire de chevalet | wippen support spring (assist / helper spring) | Lowers the down weight without the inertia of added lead [V, Spurlock, p. 3] | — | Preload; WNG sells a helper spring kit of **90 springs** [V, WNG, p. 6] |

### Hammer

| FR | EN | Role | DOF | To be parameterized |
|---|---|---|---|---|
| Ensemble marteau (olive + manche + tête) | hammer assembly | Strikes the string | **R** | Mass, inertia, center of mass |
| Olive (du manche de marteau) | hammer shank flange | Fixed part hinged on the rail | 0 | Width 12 mm, Renner type [?, not in the corpus]; traveling paper under it [V, Steinway ch. 1, p. 20] |
| Manche de marteau | hammer shank | Flexible lever arm | — | **Thickness at the center: 6 mm** [V, Reyburn, p. 6]; **stiffness**: WNG grades three, *Flex 1* the stiffest wooden shank, *Flex 2* the stiffest Renner, *Flex 3* the Renner average, from over 100 sets measured [?, not in the WNG document of the corpus]. **Do not use** 260 Hz or 280 Hz as the hammer's first mode: neither figure is in the corpus, and the first bending mode of the hammer is 39 Hz [V, Thorin 2017, §2.1 and Appendix] ([FOUNDATIONS > Rigid bodies](../FOUNDATIONS.md#rigid-bodies)) |
| Rouleau | knuckle | Interface between jack or lever and hammer | C | **Distance from the center pin: 17.0 mm on most modern parts, also 16.2, 16.0 or 15.5 mm** [V, Reyburn, p. 6]; WNG 15 to 19 mm in 0.5 mm steps [V, WNG, p. 3]; **Ø 10.0 mm, also 9.5 mm** [V, Reyburn, p. 8]; WNG 8, 9, 10 or 11 mm [V, WNG, p. 3]; leather and buckskin, lubricated with talc or PTFE powder [V, Spurlock, p. 6] |
| Tête de marteau + feutre | hammer head + felt | Striking mass | — | Mass; bore (strike distance) and hang distance ([Reyburn's ten measurements](#reyburns-ten-measurements)); Shore A hardness; molding wood, mahogany, walnut or soft maple being lighter choices than birch, hard maple or hornbeam [V, Spurlock, p. 5] |
| Queue de marteau | hammer tail | Checking surface | C | Profile, friction |
| Manchon / garniture de l'axe | center pin bushing | Bearing | C | Friction ([Friction of the centers: numbers](#friction-of-the-centers-numbers)) |

### Damper

| FR | EN | Role | DOF | To be parameterized |
|---|---|---|---|---|
| Bascule (ou levier) d'étouffoir | damper (under)lever | Lever lifted by the key tail or by the lift rail | **R** | Mass; under-lever leads, WNG's standard leading being **two leads on 32 notes from the bass, one front lead on 12, one back lead on 12, none on the rest** [V, WNG, p. 10] |
| Fourche de bascule + fourche supérieure | underlever flange + top flange | Axes | 0 | Friction ([Friction of the centers: numbers](#friction-of-the-centers-numbers)) |
| Pilote/vis de bascule d'étouffoir | underlever capstan | Aligns the **line** of the under-levers | 0 adjustable | Height; **not** the means of the collective lift [V, Redekop, p. 2; Kawai, p. 24] |
| "Cuillère" de bascule | underlever spoon | Interface with the lift rail | C | Center to tip: WNG **73.6 / 76.6 / 81.6 mm** [V, WNG, p. 9] |
| Bec de sostenuto | sostenuto tab | Caught by the sostenuto rod | 0 | Height (WNG makes two: the tall one replaces American Steinway parts, whose sostenuto is mounted on the action frame [V, WNG, p. 9]) |
| Tige d'étouffoir | damper wire | Carries the damper head | 0 adjustable (4 bends) | Bends 1 and 2 take up the misalignment between under-lever and guide rail bushing, 3 and 4 set the head vertical and centered [V, Igrec, p. 184; Steinway ch. 3, pp. 4–5]; the wire bears lightly on one side of its bushing, bass wires on the treble side and treble wires on the bass side [V, Steinway ch. 3, p. 4], which halves the head's side-to-side motion in Igrec's bass-string example [V, Igrec, p. 184] |
| Noix d'étouffoir + vis | damper block + damper wire screw | Holds the wire in the under-lever at a set depth: **individual lift timing** | 0 adjustable | **The damper-timing parameter** [V, Steinway ch. 3, p. 3] |
| Tête d'étouffoir + feutres | damper head + felts | Damps | — | Wedge felts on the trichords, combined with flat felt [V, Steinway ch. 3, p. 7]; flat ("block") dampers [V, Redekop, p. 5]; which notes get which head [?, the corpus gives no rule] |
| Barre de forte (also *barre de levée des étouffoirs*) | damper lift rail / underlever tray | Collective lift (damper pedal) | **T**/R | Rest height, lost motion |
| Barre de sostenuto | sostenuto rod / blade | Holds the tabs | **R** (rotation of the rod) | Height, distance to the tabs, rotation (angle of the blade) |
| Ressort d'étouffoir (optionnel sur queue) | damper assist spring | Assists the fall | — | Preload; a WNG option, needing the thick damper flange rail [V, WNG, p. 10] |

> The rail that lifts all the under-levers is the **barre de forte**; « barre de
> levée des étouffoirs » is a second name for the same rail. Where the lift rail
> and the rail the under-levers are fixed to are one unit, pivoting together, as
> on Steinway, the unit is the **châssis (des bascules)**; « barre de forte » names
> a lift rail pivoted on its own. « Plateau de levée », a rendering of the English
> "lift tray", is not said. « Tringle de forte » names the upright's damper lift
> rod.

### Pedals, lyre, trapwork

| FR | EN | Role | DOF | To be parameterized |
|---|---|---|---|---|
| Lyre + balustres | lyre + lyre braces | Support | 0 | — |
| Pédale forte (droite; also *pédale des étouffoirs*) | damper / sustaining pedal | Lifts all the dampers | **R** | Travel; the pedal lifts the dampers at one third of its travel [V, Redekop, pp. 3–4]; lost motion |
| Pédale sostenuto (pédale centrale) | sostenuto pedal | Holds the dampers already lifted | **R** | Travel, set by a felt block between the sostenuto trapwork lever and the keybed [V, Steinway ch. 3, p. 10] |
| Pédale douce (gauche; also *pédale una corda*) | soft / shift / una corda pedal | **Shifts the key frame**: the *transposition* | **R** → T | Travel; its stop is a screw on the treble side ([Master table of the keyboard and action settings](#master-table-of-the-keyboard-and-action-settings), row 24) |
| Tringles / mouvements / bascules de pédalier | pedal rods / trapwork levers | Transmission | R/T | Lengths, clearances, blocking felts |
| Écrou de réglage de tige de pédale | pedal rod head / capstan | Fine adjustment of the travel | 0 adjustable | Thread; sets the pedal's lost motion [V, Kawai, p. 25] |
| Prisonnier | pitman | Vertical transmission, from the damper pedal's lever to the barre de forte | T | Clearance: Kawai puts a **2 mm** paper punching between the pitman and the damper lift rail while setting the dampers [V, Kawai, p. 23] |

> The left pedal is the **pédale douce**, the name the interface shows, whatever
> its mechanism; also **pédale una corda**. On a grand it shifts the key frame and
> the action sideways: that motion is the **transposition** (« régler la
> transposition », « châssis de clavier transposable »), and « pédale de
> transposition » is said too. On some old grands and on uprights the left pedal
> raises the hammers closer to the strings instead. « Pédale de décalage » and
> « petite pédale » are not said.

---

## The regulation settings

### Master table of the keyboard and action settings

| # | FR | EN | Operational definition (how it is measured) | Typical value / range | Zone | Breaks / depends on |
|---|---|---|---|---|---|---|
| 1 | Hauteur de touche (blanches) | key height | Steinway: from the cut-out in the keybed, under the keyslip, to the underside of the key cover's lip, key at rest [V, Steinway ch. 1, p. 16]; Kawai: from the keybed to the top surface of the white key [V, Kawai, p. 14]. The two datums differ by the covering's thickness at its lip | Steinway New York **2 19/32" = 65.88 mm** [V, Steinway ch. 1, p. 16] or **2 21/32" = 67.47 mm** [V, Spec sheet, p. 2] [?, the two sources disagree]; Hamburg **63 mm** (S, M, L, O, A, B) / **65 mm** (C, D) [V, Spec sheet, p. 2]; Yamaha **64 mm** (+1/−3) [V, Spec sheet, p. 3]; Kawai **66 mm** or **68 mm** by model [V, Kawai, p. 14] | key leveling crowned ~1/32" at the center (Steinway) [V, Steinway ch. 1, p. 16] | Conditions the key dip, the aftertouch [V, Steinway ch. 2, p. 11], the damper lift timing |
| 2 | Hauteur des dièses | sharp height | Above the plane of the naturals | Steinway **1/2" = 12.7 mm** [V, Steinway ch. 1, p. 17]; Kawai **12.5 mm** [V, Kawai, p. 15]; Yamaha **12 mm** (+0.5/−0) [V, Spec sheet, p. 3] | — | Too low: the sharps bury below the naturals at the bottom of the dip [V, Steinway ch. 2, p. 10] |
| 3 | Dressage / niveau | key leveling | Straightedge; paper punchings under the balance rail punching [V, Steinway ch. 1, p. 16] | ±0.1–0.2 mm **[S]**; Steinway's deliberate crown | — | The basis of everything |
| 4 | Équerrage / jours des touches | squaring / spacing | Tapping the balance rail pin sideways, bending the front rail pin with a spacing tool [V, Steinway ch. 1, p. 16; Kawai, p. 14] | — | — | Lateral friction |
| 5 | Jeu des mortaises d'enfoncement | front bushing play | Side play at rest **and** depressed | Steinway **≤ 1/32" = 0.79 mm** [V, Steinway ch. 1, p. 15]; Kawai **0.2–0.3 mm** [V, Kawai, p. 5] | uniform | Friction, noise |
| 6 | Jeu des mortaises/trous de balancier | balance bushing & hole play | Slight side play at the bushing, none fore and aft at the hole [V, Steinway ch. 1, p. 14]; the key tail lifted 1–2 mm and released falls slowly under its own weight, more slowly in dry weather [V, Kawai, p. 6] | "slight play" | uniform; depends on humidity | Friction (F) |
| 7 | **Chasse** (distance de frappe) | **blow distance** | From the underside of the string to the top of the hammer head, hammer at rest; **set by the capstan** [V, Steinway ch. 2, p. 6] | Steinway New York **1 3/4" = 44.45 mm** (S, M, L, O, A, B) / **1 7/8" = 47.63 mm** (C, D) [V, Steinway ch. 2, p. 6]; Hamburg **47 mm** [V, Spec sheet, p. 2], 45–47 mm in Steinway's Hamburg checklist [V, Steinway ch. 1, p. 30]; Yamaha **48 mm** [V, Spec sheet, p. 3]; Kawai **46 mm** [V, Kawai, p. 15]; generic range **42–47 mm**, by the piano's size and geometry [V, Spec sheet, p. 1] | **set section by section**, string height changing along the scale [V, Steinway ch. 2, p. 6] | Too shallow weakens the power stroke, too deep needs excessive key dip [V, Steinway ch. 2, p. 6]; **more blow distance = less aftertouch** [V, Steinway ch. 2, p. 11]; interacts with the key dip [V, Igrec, p. 158] |
| 8 | Ligne des marteaux | hammer line | The heads level at rest: a straightedge across the shank flanges, the key lid as a backdrop [V, Steinway ch. 2, p. 6] | — | per section | Looks, and the hammers' acceleration [V, Igrec, p. 173]; **raising the line = more aftertouch** [V, Steinway ch. 2, p. 6] |
| 9 | Hauteur de la barre de repos des marteaux | hammer rest rail height | Below the shanks at rest | Kawai **5 mm** [V, Kawai, p. 16] | — | The shanks stay off it, the weight on the capstans [V, Steinway ch. 1, p. 9] |
| 10 | Position du bâton sous le rouleau | jack (fly) position | Back of the jack in line with the back of the knuckle's wood core [V, Kawai, p. 13; Spec sheet, p. 4; Steinway ch. 1, p. 29]; Steinway New York: "the far side of the fly aligns with the far side of the knuckle insert" [V, Steinway ch. 2, p. 4] | aligned | uniform | **Too far forward**: escapes too early, little control, little power, possibly a misfire; **too deep**: excess friction, poor repetition [V, Steinway ch. 2, p. 4] |
| 11 | Centrage du bâton dans la fenêtre du levier | jack centering | Visually: the fly centered in the repetition lever's window [V, Steinway ch. 1, p. 28]; traveling paper under the wippen flange [V, Kawai, p. 11] | centered | — | Noise, friction |
| 12 | Hauteur du bâton / levier de répétition | jack height to repetition lever | Top of the jack **slightly below** the top of the lever | Yamaha **0.2 mm** [V, Spec sheet, p. 4]; Kawai **0.1–0.2 mm** [V, Kawai, p. 13]; Steinway: the lever slightly above the jack, then "rolling the flies", a slight drag felt as the fly passes under the knuckle, which makes the hammer "wink" [V, Steinway ch. 2, p. 5] | uniform | Test, pressing the jack heel: **too low**, the hammer drops and does not come back; **too high**, it does not move [V, Kawai, p. 13]. A lever too high holds the knuckle clear of the jack: **lost motion between jack and knuckle**, which this setting removes [V, Steinway ch. 2, p. 5] |
| 13 | **Échappement** | **let-off** | Hammer-to-string distance at the instant the jack leaves the knuckle; set by the let-off button | Steinway New York **1/32"–1/16" = 0.79–1.59 mm** [V, Steinway ch. 2, p. 7], **1/16"** [V, Spec sheet, p. 2]; Hamburg **1.5 mm** bass, **1.0 mm** mid and treble [V, Spec sheet, p. 2], 1 mm from the steel strings and half a bass string's thickness [V, Steinway ch. 1, p. 30]; Yamaha **3 mm** bass to **1.5 mm** treble [V, Spec sheet, p. 3]; Kawai **2.0–2.5 mm** bass, **1.0–1.5 mm** mid and treble [V, Kawai, p. 16]; generic **3 to 1.5 mm**, closer in the concert hall, **2 / 1.5 / 1** [V, Spec sheet, p. 1]; **1 to 3 mm** [V, Oleg glossary, p. 1] | **decreases toward the treble**: the strings move further in the tenor and bass [V, Igrec, p. 174] | **Set in the piano** [V, Steinway ch. 2, p. 7]; conditions the ppp [V, Igrec, p. 174]; too far: loss of power, no control in soft playing [V, Steinway ch. 2, p. 7]; too close: blocking, hard tone, a risk of breaking strings [V, Igrec, p. 174; Oleg glossary, p. 1] |
| 14 | **Chute** (ou rechute) | **drop** | How far the hammer falls just after let-off, the key going on to the bottom [V, Igrec, p. 173; Steinway ch. 2, p. 8] | Written two ways. **Below let-off**: Steinway New York **≤ 1/16" = 1.59 mm** [V, Steinway ch. 2, p. 8]; Hamburg, the hammers "fall down 2 mm after let-off" [V, Steinway ch. 1, p. 30], 2.0 mm on the sheet [V, Spec sheet, p. 2]; Igrec **~1/16" (1.5 mm)** [V, Igrec, p. 173]; Kawai: the drop screw meets the lever as the jack meets the button, so the initial fall **equals the let-off** [V, Kawai, p. 17]. **From the string**: Yamaha **let-off + 2.0 mm** [V, Spec sheet, p. 4]. **Do not use** Hamburg's 2.0 mm as a distance from the string: Steinway's Hamburg checklist makes it the fall below let-off | follows the let-off | **Must never "eclipse" the let-off** [V, Steinway ch. 2, p. 7; ch. 1, p. 9]; set to avoid double striking [V, Steinway ch. 2, p. 8]; too much drop loses the second escapement [?, not in the corpus] |
| 15 | **Enfoncement** (blanches) | **key dip** | Total travel of the key front, against a touch (dip) block, with consistent pressure [V, Steinway ch. 2, p. 9; Kawai, p. 17] | Steinway factory **0.400" = 10.16 mm**, range **0.390–0.420" = 9.91–10.67 mm** [V, Steinway ch. 2, p. 9]; the New York sheet **13/32" = 10.32 mm** [V, Spec sheet, p. 2]; Hamburg **9.9–10.6 mm** [V, Spec sheet, p. 2]; Yamaha **10 mm** [V, Spec sheet, p. 3]; Kawai block **10.1 mm** [V, Kawai, p. 17], 10.1 or 10.3 mm by model [V, Spec sheet, p. 1]; generic **9.3–10.6 mm** [V, Spec sheet, p. 1]; **10 mm** [V, Oleg glossary, p. 1] | uniform | **Finger pressure changes the reading**: light pressure, deeper dip [V, Steinway ch. 2, p. 9]; the keyblocks firmly secured [V, Steinway ch. 2, p. 10] |
| 16 | Enfoncement des dièses | sharp dip | **By feel**: the naturals' aftertouch duplicated [V, Steinway ch. 2, p. 10; Kawai, p. 18] | Kawai: the sharps **~2 mm** above the naturals at the bottom of the keystroke [V, Kawai, p. 18] | — | The sharps must not bury [V, Steinway ch. 2, p. 10] |
| 17 | **Gras / jeu après échappement** | **aftertouch** | Key travel at the front from let-off, the hammer's highest point, to the bottom of the key travel, with moderate pressure [V, Igrec, p. 173] | Steinway **3/64"–1/16" = 1.19–1.59 mm** [V, Steinway ch. 2, p. 11], **0.040–0.055" = 1.02–1.40 mm** on the New York sheet [V, Spec sheet, p. 2]; Hamburg **0.8–1.2 mm** [V, Spec sheet, p. 2]; Igrec **~0.040" = 1.0 mm**, reached within **0.030–0.060" (0.8–1.5 mm)** after let-off [V, Igrec, p. 173] | uniform (by feel) | **Set by the key dip and/or the blow distance, never by the let-off or the drop** [V, Igrec, p. 173]; also moved by key level [V, Steinway ch. 2, p. 11]; very sensitive to the curvature of the keybed: **set in the piano** [V, Igrec, p. 173] |
| 18 | Course libre du bâton en fond | jack free travel | Key fully down, some free travel left between the jack and its stop felt [V, Steinway ch. 2, p. 11] | "some" | — | The jack clears properly |
| 19 | **Attrapé** | **checking / back check distance** | Height of the head below the string when the hammer is checked, key held | Steinway New York **1/2"–5/8" = 12.7–15.9 mm** [V, Spec sheet, p. 2; Steinway ch. 2, p. 13]; Hamburg **15 mm** [V, Spec sheet, p. 2; Steinway ch. 1, p. 30]; Kawai **15 mm** [V, Kawai, pp. 19–20]; Yamaha **16 mm** bass to **14 mm** treble [V, Spec sheet, p. 4]; Igrec **1/2"–5/8" (12–15 mm)** [V, Igrec, p. 174]; generic **12.7–16 mm** [V, Spec sheet, p. 1] | slight gradation, bass to treble (Yamaha) | Too close: the tail drags or blocks in fortissimo [V, Kawai, p. 19; Steinway ch. 2, p. 13]; too low: the hammer travels further, the fly returns later, repetition slows [V, Steinway ch. 2, p. 12] |
| 20 | Alignement/équerrage/pente des attrapes | backcheck squaring, alignment, angle | Parallel pliers to square, wire-bending pliers to align [V, Steinway ch. 2, p. 12]; test by rocking key and hammer together [V, Steinway ch. 2, p. 13] | square to the tail; back surfaces parallel to the hammer line [V, Kawai, p. 19] | — | Out of square: wear of the shank's center-pin bushing and of the backcheck leather [V, Steinway ch. 2, p. 12] |
| 21 | Tension des ressorts de répétition | repetition spring tension | Strike the key, let the hammer check, release the key **slowly**: the hammer rises until the lever is stopped by the drop screw [V, Steinway ch. 2, p. 14] | No number in the trade sources; the criterion is a rise "steady and quick, but not snappy" [V, Steinway ch. 2, p. 15], "without jumping or catching" [V, Kawai, p. 12]. Measured spring laws of single actions are in [FOUNDATIONS > Springs inside the wippen assembly](../FOUNDATIONS.md#springs-inside-the-wippen-assembly) | **graduated**: slower in the bass, progressively faster in the treble [V, Steinway ch. 2, p. 15]; whether the bass should rise much more slowly or at nearly the treble's speed is a question for the owner ([FOUNDATIONS > Open questions for the owner](../FOUNDATIONS.md#open-questions-for-the-owner)) | Adjusted through the **coil** only [V, Kawai, p. 12]; re-checked **after** the backchecks [V, Kawai, p. 20] and **in the piano** [V, Steinway ch. 2, p. 15; Kawai, p. 12] |
| 22 | Vis de réglage de portée du châssis | balance rail studs / glides | Knock test: turn the stud down until the knocking starts, then stops [V, Steinway ch. 1, p. 12]; re-check after pressing the shift pedal three or four times [V, Steinway ch. 1, p. 12] and with the damper pedal down [V, Kawai, p. 21] | just touching, **never lifting** the rail [V, Kawai, p. 21; Steinway ch. 1, p. 13] | — | Kawai's order: the tenor break, then the bass end, re-check, then toward the treble [V, Kawai, p. 21]; **never used to add key dip** [V, Steinway ch. 1, p. 13] |
| 23 | Barre de contre-touche | key stop rail | Clearance above the keys | Steinway **~1/16" = 1.59 mm** [V, Steinway ch. 1, p. 17] | — | — |
| 24 | Distance de transposition | shift / keyframe shift | Shift screw on the treble side of the rim, or in the treble keyblock on older Steinways [V, Steinway ch. 2, p. 16]; in the right cheekblock [V, Kawai, p. 27] | Kawai: only **2 strings of 3** struck, the left one missed [V, Kawai, p. 27]; Steinway Hamburg: "only two strings should be struck" [V, Steinway ch. 1, p. 30]; Steinway New York: a partial shift is acceptable, changing the color rather than the volume [V, Steinway ch. 2, p. 16]; Yamaha, the hammer's left side grazing the left string [?, not in the corpus] | trichords, tenor and treble [V, Kawai, p. 27] | No dimension in millimeters in the corpus: the shift is set by where the hammers strike (row 26) |
| 25 | Point de frappe / position du châssis | strike point / keyframe position | Kawai: hammer 88 strikes **2.16–3.0 mm** from the capo bar, the final position chosen **by ear** [V, Kawai, p. 4] | 2.16–3.0 mm | — | A shim glued to the block behind the key frame, or the block filed [V, Kawai, p. 4] |
| 26 | Jours / mise des marteaux sur les cordes | spacing hammers to strings | Kawai: on the trichords the hammer's bass side overlaps the left string by **1 mm** [V, Kawai, p. 10]; Steinway: off center toward the treble, **about twice as much felt exposed on the treble side** [V, Steinway ch. 1, p. 22], a third on the left and two thirds on the right in Hamburg [V, Steinway ch. 1, p. 29] | — | unichords centered; bichords centered [V, Kawai, p. 9] or slightly favoring the bass [V, Steinway ch. 1, p. 23] | Conditions the una corda voicing [V, Steinway ch. 1, p. 22] |
| 27 | Défauchage ; chauffage des manches | traveling ; burning shanks | Défauchage: lift a group with a straightedge and watch the sideways drift, then shim the flange on one side of its screw with a paper (*cale de défauchage*) [V, Steinway ch. 1, p. 20; Kawai, p. 8]; wippens are « défauchés » the same way, at their flanges. Chauffage: heat the shank and twist it, so that the hammer head stands parallel to the plane it swings in [V, Steinway ch. 1, p. 21] | a straight, vertical path; a hammer that does not lean | — | A prerequisite to the rest |
| 28 | Portée du châssis | bedding the keyframe | Knock test, fingers on the front rail and keybed [V, Kawai, p. 7] | no knocking | — | Redone with the humidity: the front rail lifts at its ends in dry weather [V, Kawai, p. 7]; out of the model's scope ([FOUNDATIONS > What the model is](../FOUNDATIONS.md#what-the-model-is)) |

### Damper regulation settings

| FR | EN | Definition | Value | Dependencies |
|---|---|---|---|---|
| Départ individuel (par la touche) | individual damper lift timing | Where the hammer is when the damper head starts to move; set by the **depth of the wire in the under-lever** [V, Steinway ch. 3, p. 3] | **At half the blow distance**: Kawai **23 mm** [V, Kawai, p. 23], Yamaha **24 mm** (±1) [V, Spec sheet, p. 4], Redekop about 23 mm, "half of the 46mm" [V, Redekop, p. 3]; Steinway: the key end meets the under-lever at **~1/8" = 3.18 mm** of key travel, so the damper starts **~1/3** into the hammer travel, and **1/3 to 1/2** is acceptable [V, Steinway ch. 3, p. 3] | Depends on **blow distance and key dip** [V, Igrec, p. 158]; later timing, lighter touch [V, Steinway ch. 3, p. 3]; Redekop's note "Too light interferes with the let-off" follows a truncated sentence [?, its subject is lost: Redekop, p. 3] |
| Ligne des bascules | underlever line | A straightedge laid on the under-levers; set with the underlever capstans | straight and parallel to the keybed [V, Redekop, p. 2; Kawai, p. 23] | The capstans are not the collective lift [V, Redekop, p. 2; Kawai, p. 24] |
| Départ général (par la pédale) | collective lift | All the under-levers starting together: four sample notes marked in pencil at 23 mm, then the wire block screws set the rest [V, Redekop, p. 2] | — | Fluttering the pedal shows the early lifters, squeezing it the late ones [V, Redekop, p. 3] |
| Jeu de la barre de forte **[?]** | lift tray lost motion | Clearance between the lift rail and the line of under-levers at rest | Steinway **~1/8" = 3.18 mm** [V, Steinway ch. 3, p. 7]; Redekop **~2 mm** [V, Redekop, p. 3]; Kawai **1–2 mm**, set at the pedal rod head [V, Kawai, p. 25] | — |
| Barre de butée des étouffoirs | damper (up)stop rail | Play left in the head lifted by hand, a sharp held down | Steinway **~1/16" = 1.59 mm** treble, **~3/32" = 2.38 mm** bass [V, Steinway ch. 3, p. 6]; Kawai **1–2 mm** [V, Kawai, p. 24]; Redekop **0.5 mm** sharps / **1 mm** naturals [V, Redekop, p. 3] | Set **after** the key dip [V, Igrec, p. 158] |
| Course de la pédale forte | damper pedal travel | A felt block between the keybed's underside and the trapwork lever [V, Steinway ch. 3, p. 6] | The pedal lifts the dampers at **one third of its travel** [V, Redekop, pp. 3–4]; it lifts them as high as the sharps do, set by Kawai's pedal stop capstan [V, Kawai, p. 25], or slightly lower [V, Steinway ch. 3, p. 6]; always below the stop rail | A pedal lifting further makes the dampers bounce [V, Redekop, p. 4] |
| Hauteur de levée de la tête | damper lift height off strings | — | No standard in the corpus; Schimmel **3.5–4 mm**, flat dampers rising **~8 mm** in practice [?, not in the corpus] | — |
| Alignement du fil, 4 coudes | damper wire bends 1–4 | Vertical, centered, front and back seated together [V, Steinway ch. 3, pp. 4–5] | The wire bears **lightly** on one side of its bushing: bass wires on the treble side, treble wires on the bass side [V, Steinway ch. 3, p. 4] | The lean halves the head's side-to-side motion on a bass string [V, Igrec, p. 184] |
| Assise / seating | damper seating | Tap the front and the back of the head: the same sponginess [V, Redekop, p. 2] | the back lifts **very slightly before** the front [V, Kawai, p. 22] | String leveling is essential [V, Redekop, p. 4] |

### Sostenuto

| Setting | Value | Source |
|---|---|---|
| Rotation of the rod at rest, seen from above through the strings | **~1 mm** of the blade showing toward the tabs | [V, Steinway ch. 3, p. 11], Hamburg system |
| Height: tabs below the rod | **~1.5 mm** below the rod | [V, Steinway ch. 3, p. 11] |
| Distance rod to tabs at rest | **~2 mm** | [V, Steinway ch. 3, p. 11]; Redekop also ~2 mm [V, Redekop, p. 3] |
| Angle of the blade | Redekop gives **45° at rest, 90° depressed**, then **45° played, 90° at rest** | [?, Redekop contradicts himself, p. 3 against p. 4] |
| Pedal travel | The damper rises **slightly higher** than with the damper pedal, **without** the under-lever jamming against the stop rail | [V, Steinway ch. 3, p. 10] |
| Height of the rod | Slightly higher than the tabs; the angle set with the capstan under the keybed; the pedal holds the dampers at the same height as the key does | [V, Kawai, p. 26] |
| Functional tests | (a) damper pedal down, then the sostenuto; release the damper pedal: every damper stays up; (b) release the sostenuto slowly: all fall; (c) sostenuto held, forte blows on the notes not held: **no catching** | [V, Steinway ch. 3, p. 8; Kawai, p. 26] |

### Touchweight: the quantitative core

The protocols are Stanwood's *Standard Protocols of the New Touchweight Metrology*
(February 2000), which supersede those of his 1996 paper ([FOUNDATIONS > The trade's protocols are the acceptance test](../FOUNDATIONS.md#the-trades-protocols-are-the-acceptance-test)). Every
weight is centered on the **measuring point**, "the datum point on the top of the
key 13mm or 1/2" back from the front lip of the key" [V, Stanwood 2000, glossary,
p. 23]; the 1996 paper puts it "13mm in from the front vertical edge of the key"
[V, Stanwood 1996, p. 3]. The glossary does not say which lip; on a key whose
covering overhangs the key front, the simulator takes the covering's, the owner's
choice, and holds the distance as one setting, 13 mm by default, that every
touchweight reads ([FOUNDATIONS > The trade's protocols are the acceptance test](../FOUNDATIONS.md#the-trades-protocols-are-the-acceptance-test)).

#### Quantities

| FR quantity | EN | Symbol | Definition | Values |
|---|---|---|---|---|
| Poids d'enfoncement (also *poids descendant*) | down weight | **D** | The least weight, to the gram, on the measuring point that makes the key go down while the hammer moves slowly and steadily [V, Stanwood 2000, glossary, p. 22]. Spurlock places it on the end of the key and stops at the jack tender first touching the let-off button [V, Spurlock, p. 1]. Dampers out of circuit, action on the bench or pedal down [V, Spurlock, p. 1] | about **50–60 g** in most modern grands in good condition [V, Spurlock, p. 2]; Steinway Hamburg **47 g** in every section (S, M, L, O, A, B), **52 g** bass to **47 g** treble (C, D); Steinway New York **50 g** bass to **47 g** high treble (S, M, L, O, A, B), **51 to 46 g** (C, D) [V, Spec sheet, pp. 2–3] |
| Poids de remontée (also *poids ascendant*) | up weight | **U** | The greatest weight, to the gram, on the measuring point that the key lifts while the hammer moves slowly and steadily [V, Stanwood 2000, glossary, p. 23]; Spurlock starts it with the jack tender just touching the let-off button, the jack not tripped [V, Spurlock, p. 1] | **20 g** "is usually cited as a minimum upweight for good repetition, although this figure leaves little safety factor" [V, Spurlock, p. 2]; Steinway **20 g or more**, every model [V, Spec sheet, p. 3]; slow key return below about **18 g** [V, Spurlock, p. 5] |
| Friction (also *frottement*) | friction weight | **F** | **F = (D − U)/2** [V, Stanwood 2000, glossary, p. 22; Spurlock, p. 1] | **12–15 g** in the bass tapering to **8–10 g** in the high treble; **±1 g** from note to note is a fine tolerance [V, Spurlock, pp. 2, 7] |
| Poids d'équilibre | balance weight | **BW** | **BW = (D + U)/2**, the weight on the measuring point that balances the key [V, Stanwood 2000, glossary, p. 22] | — |
| Poids avant | front weight | **FW** | The static weight, to 0.1 g, that "the level key, tipped on its balance-pin point, exerts at the measuring point" [V, Stanwood 2000, glossary, p. 22] | ceiling recommended by Stanwood: **41.3 g** at note 1 to **3.7 g** at note 88 [V, Stanwood 2000-03, Table IV, p. 25] |
| Poids d'attaque (du marteau; also *poids de frappe*) | strike weight | **SW** | The weight, to 0.1 g, of shank and hammer pivoted without friction at the hammer center, shank level, measured at the strike line radius [V, Stanwood 2000, glossary, p. 23]; SW = HW + SS, HW being the hammer alone [V, Stanwood 2000, p. 21] | normal zone **8.0–14.8 g** at note 1 to **3.5–6.9 g** at note 88 [V, Stanwood 2000-03, Table I, p. 21] |
| Poids d'attaque du manche | shank strike weight | **SS** | The shank alone, measured as for SW [V, Stanwood 2000, glossary, p. 23] | **1.7 g** as a generic average [V, Stanwood 2000-03, note to Table I, p. 21] |
| Rapport de poids d'attaque | strike weight ratio | **R** | **R = SBW / SW**: the weight on the measuring point that balances 1 g of strike weight [V, Stanwood 2000, glossary, p. 23]. FOUNDATIONS writes it SR in the equation of balance; it is not Reyburn's shank ratio of [The action ratio: four definitions, four values](#the-action-ratio-four-definitions-four-values) | normal zone **5.0–7.0**; *low* 5.0–5.7 (high leverage), *medium* 5.7–6.3, *high* 6.3–7.0 (low leverage), the leverage being the pianist's on the hammer [V, Stanwood 2000-03, Table III, p. 24] |
| Poids radial du chevalet | wippen weight | **WW** | The weight, to 0.1 g, of the level wippen, pivoted without friction at its center, measured at the capstan/heel contact point [V, Stanwood 2000, glossary, p. 23] | — |
| Rapport de poids de clavier | key weight ratio | **KR** | The force at the measuring point over the force at the capstan: the weight on the measuring point that balances 1 g at the capstan/heel contact point [V, Stanwood 2000, glossary, pp. 22–23] | about **0.5** [V, Reyburn, p. 2]; 0.50 to 0.60 on most actions, 0.68 on an 1878 Steinway concert grand ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)) |

**Do not use** the reciprocal as KR. Stanwood's own wording, "the ratio of downward
force on the capstan/heel versus the corresponding upward force at the measuring
point" [V, Stanwood 2000, glossary, p. 22], like his 1996 wording [V, Stanwood
1996, p. 5], names the capstan first, but his number is the front over the capstan:
10 g at the capstan reading −5.5 g at the front gives 0.55 [V, Stanwood 2000,
p. 22]. The reciprocal, front arm over capstan arm, "nominally 2:1" [V, Stanwood,
pianotech list], is the geometric convention: put into the equation of balance it
inflates the wippen term by 1/KR², about 3.3, and makes a wippen balance weight
larger than the wippen weight ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)).

#### The equation of balance

```
BW + FW = (WW × KR) + (SW × R)
```

[V, Stanwood 1996, p. 2]; [FOUNDATIONS > The trade's protocols are the acceptance test](../FOUNDATIONS.md#the-trades-protocols-are-the-acceptance-test) writes SR for R. With TBW = BW + FW
(top action balance weight), WBW = KR × WW (wippen balance weight) and
SBW = TBW − WBW (strike balance weight) [V, Stanwood 2000, glossary, pp. 22–23],
it reads TBW = WBW + SBW, and FW = (WW × KR) + (SW × R) − BW.

#### The protocols

- **Front weight.** The key sits on a wedge jig: an extra-long balance-rail pin
  passes vertically through the edge of a square-section wedge, the wedge's edge
  lies on the axis of the rod it is fastened to, and the key is placed on that pin,
  so it pivots about the balance pin's axis and cannot tip over, whatever the pin's
  angle. Its front rests on a roller bearing on the scale pan, directly below the
  measuring point, and the key is set horizontal with a **bubble level**, which
  replaces the 1996 protocol's "horizontal attitude similar to that when the key is
  at rest" [V, Stanwood 2000, p. 21; glossary, p. 23; Stanwood 1996, p. 3].
- **Negative front weight**, a key heavier behind its pin, which will not stay down
  on the jig: the scale is zeroed with a dummy weight and the roller jig on the
  pan; the key goes on the jig, the dummy weight on the key over the roller
  bearing; the front weight is read off the scale as a negative value [V, Stanwood
  2000, p. 21].
- **Key weight ratio.** The key set up as for a negative front weight, the dummy
  weight holding its front down on the scale, the scale zeroed, a **10 g** weight
  on the back of the key, centered on the vertical through the capstan/heel contact
  point: a reading of −5.5 gives KR = 0.55, to two decimal places [V, Stanwood
  2000, p. 22]. It replaces the 1996 protocol, 100 g at the capstan with the front
  held down by enough weight to make the front weight at least 70 g, which
  Stanwood finds more accurate than needed [V, Stanwood 2000, pp. 21–22; Stanwood
  1996, p. 5].
- **Key Friction Weight (KF)**, the share of the friction weight that comes from
  the key bushings: on the jig, following the negative front weight protocol,
  temporary weights are slid along the key until its front weight is zero; then,
  without moving them, the key goes back on its frame, its back pushed down onto the
  back rail cloth, and KF is the least weight, to the gram, on the measuring point
  that makes the key drop, stack removed. Alternatively, a key leveling lead holds
  the back of the key down on the frame and KF = (D − U)/2, stack off [V, Stanwood
  2000, p. 22; glossary, p. 22]. The bench reads it both ways, labeled: zeroed
  on the leveled jig, as Stanwood does, and zeroed where the key rests on its
  frame, which reads the bushings alone ([FOUNDATIONS > The protocols, as they are](../FOUNDATIONS.md#the-protocols-as-they-are)).
- **Support Spring Balance Weight (BWS)**: the balance weight with the wippen
  support spring disengaged, minus the balance weight with it engaged. "Engaging the
  support spring equally reduces Up Weight (U), Down Weight (D) and Balance Weight
  (BW)", so the preferred protocol measures U with the spring disengaged, then
  engaged [V, Stanwood 2000, p. 22; glossary, p. 23]. **Do not use** that sentence
  to argue that the support spring is the only spring that can move BW: it holds for
  any static load depending on position under friction not depending on load, and
  the repetition spring moves D, U and BW too whenever the lever bears on the
  knuckle or on the drop screw during the stroke ([FOUNDATIONS > Springs inside the wippen assembly](../FOUNDATIONS.md#springs-inside-the-wippen-assembly)).
- **Strike weight**: the hammer tail resting on the scale pan, the hammer facing up,
  the flange vertical on a low-friction roller bearing whose height sets the shank
  horizontal, checked with a bubble level on the shank; shank strike weight the same
  way, a wedge under the strike line [V, Stanwood 2000, pp. 20–21].

#### Sensitivities (for the interface's sliders)

- **+1 g of hammer weight gives +5 to +7 g on both D and U** [V, Spurlock, p. 2]:
  the strike ratio ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)).
- A moderate filing removes about ¼ g, **−1.5 g** on D and U; after many filings a
  hammer loses **over 1 g**, **−5 to −10 g** of touchweight [V, Spurlock, p. 2].
- Fifty-year-old hammers can make the touch **10 g lighter** than original [V,
  Spurlock, p. 4].
- **Lead added to the front of the key lowers D and U by the same amount** [V,
  Spurlock, p. 2] **and adds inertia**: the wrong remedy where the leading is
  already average [V, Spurlock, p. 4].
- Unusually low friction on one key is a sure sign of a loose hammershank center
  [V, Spurlock, p. 2].
- Touching the knuckles with a finger can add **2 g** of friction to a note for a
  while [V, Spurlock, p. 6].
- **Do not use "+0.1 mm of capstan ⇒ −1.4 g of down weight and −0.3 mm of
  aftertouch"**: it comes from a display mock-up, and raising the capstan raises the
  hammer line, which increases aftertouch [V, Steinway ch. 2, p. 11] ([FOUNDATIONS > Levels](../FOUNDATIONS.md#levels)).

#### Sources of friction, in decreasing order

**knuckle > hammershank center pin > wippen center pin > key bushings > capstan**,
in actions in good adjustment [V, Spurlock, p. 2]. Friction is higher in the bass
for two reasons: shank pinning is tighter there, heavy hammers swinging the same
number of times as light ones needing tighter pins; and the heavier hammers press
the knuckle harder against the wippen [V, Spurlock, p. 2].

### Friction of the centers: numbers

**Steinway factory standard**: every action center within **1–4 g of rotating
friction**, minimal side play, read with a gram gauge applied to the flange [V,
Steinway ch. 1, pp. 18–19]; some centers read near zero and work perfectly well
[V, Steinway ch. 1, p. 19]. Igrec reports that Steinway recommends the low end of
the ranges of his Table 5 [V, Igrec, p. 143]. A piano action has **350 to 600
centers** [V, Oleg glossary, p. 1].

**Igrec's Table 5**, torque = force × distance [V, Igrec, p. 143]:

| Part | Distance from the center pin | Friction (g) | Torque (mm·g) |
|---|---|---|---|
| Hammer shank flange | 25 mm | 1.5–4.0 | 37–100 |
| Repetition lever (spring disengaged) | buckskin at the drop screw (58 mm) | 2.0–6.0 | 120–360 |
| Wippen flange | 25 mm | 1.5–4.0 | 37–100 |
| Damper under-lever flange | 25 mm | 1.0–4.0 | 25–100 |
| Under-lever top flange | 25 mm | 1.0–4.0 | 25–100 |
| Sostenuto tab (spring disengaged) | tip (15 mm) | 1.0–6.0 | 15–90 |
| Jack (spring disengaged) | tip of the tender (25 mm) | 0.5–4.0 | 12–100 |
| Upright damper lever | ~50 mm | 1.0–8.0 | 50–400 |

The two directions of rotation should agree within **50 mm·g**; beyond that, look
for debris or splinters [V, Igrec, p. 143]. WNG measures flanges at **20 mm**,
Steinway's specifications assume **~32 mm** [V, Igrec, p. 143]: hence friction
stored as a **torque** with the radius it was measured at, never as grams alone
([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)).

---

## Leverage ratios and geometry

### Reyburn's ten measurements

Reyburn's geometry sheet takes ten measurements on a key, a wippen and a hammer,
shank and flange, from which the radial arms of the three levers follow [V,
Reyburn, pp. 1–2]. They check an assembled model's arms; they are not enough to
build one, which needs every pivot and contact profile in the sagittal section
([FOUNDATIONS > What the model is](../FOUNDATIONS.md#what-the-model-is), [FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap)).

| # | Parameter | Rounded to | Reyburn's example, and other values |
|---|---|---|---|
| 1 | Key front to the center of the balance rail hole, on the bottom of the key | 0.5 mm | **259.5 mm** [V, Reyburn, p. 3] |
| 2 | Key front to the capstan lift point, on the side of the key, the key top's overhang excluded | 0.5 mm | **398 mm** [V, Reyburn, p. 3] |
| 3 | Key bottom to the top of the capstan, at the lift point | 0.5 mm | — |
| 4 | Height of an external key shoe at the balance rail | 0.5 mm | **4–6 mm** when present, otherwise 0 [V, Reyburn, p. 4] |
| 5 | Wippen center to the back of the jack | 0.1 mm | **92.55 mm** [V, Reyburn, p. 5, photograph]; WNG standard 99 ± 1 mm [?, not in the WNG document of the corpus] |
| 6 | Wippen center to the capstan lift point | 0.1 mm | **65.93 mm** [V, Reyburn, p. 5, photograph]; WNG heels 50.0–69.5 mm from the wippen center pin [V, WNG, pp. 4–5] |
| 7 | Shank center pin to the hammer center line ("hang") | 0.1 mm | **129.40 mm** [V, Reyburn, p. 6, photograph] |
| 8 | Shank center pin to the knuckle center | 0.1 mm | **17.0 mm** on most modern parts; also 16.2, 16.0 or 15.5 mm [V, Reyburn, p. 6] |
| 9 | Strike distance (bore): string height minus the hammer flange center height | 0.1 mm | **51.64 mm** [V, Reyburn, p. 7, photograph] |
| 10 | Knuckle diameter | 0.1 mm | **10.0 mm**; also 9.5 mm [V, Reyburn, p. 8] |
| — | Shank thickness at the center | — | **6 mm** [V, Reyburn, p. 6] |

The **spread**, wippen flange center pin to shank flange center pin: Yamaha G1–C7
**112.5 mm** [V, Spec sheet, p. 4].

### The action ratio: four definitions, four values

The trade defines the action ratio in four ways, and on one action the four differ;
the engine is to report each, labeled, as an output ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs), planned).

| Definition | Formula | Values |
|---|---|---|
| **Product of the levers** | AR = KR × WR × SR: KR the key ratio, capstan arm over front arm, "about 0.5"; WR the wippen ratio, measurement 5 over measurement 6 of [Reyburn's ten measurements](#reyburns-ten-measurements), "about 1.4"; SR the shank ratio, "about 6 to 6.5" [V, Reyburn, p. 2] | "An AR of about 5 is normal, 4.9 to 5.1 is the best range" [V, Reyburn, p. 2]. The typical factors multiply to 0.5 × 1.4 × 6 = 4.2 and 0.5 × 1.4 × 6.5 = 4.55, short of 5 |
| **Gauge at the start of the stroke** | Hammer rise over a 6 mm key travel from rest, the key depressed by a tapered 6 mm block, not far enough to trip the jack | "generally varies from 5 to 6.2, with most pianos measuring between 5.3 and 5.9" [V, Spurlock, pp. 3–4] |
| **Stanwood's strike ratio** | R = SBW / SW ([Touchweight: the quantitative core](#touchweight-the-quantitative-core)), a ratio of forces read by weighing | normal zone 5.0 to 7.0 [V, Stanwood 2000-03, p. 24]; 5.5 recommended as a modern standard [V, Stanwood, pianotech list] |
| **Ratio of useful travels** | (blow distance − let-off) / (key dip − aftertouch): the hammer's travel to let-off over the key's travel to let-off [?, no source in the corpus states this definition] | (45 − 2) / (10 − 1.5) = 5.06 with typical settings; 45 / 10 = 4.5 mixes the hammer's travel to the string with the key's travel past let-off |

**Do not use** the geometric key ratio, front arm over capstan arm ("nominally
2:1"), in the product of the levers: Reyburn's KR is the capstan arm over the front
arm, about 0.5, and its reciprocal multiplies AR by about 4 ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)).

**The ratio is not constant.** Arms change as contact points migrate, so every lever
ratio varies along the stroke; the small-angle ratios above are linearizations about
rest ([FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap)). **Do not use** the "BYU Design Review" figures (an
aftertouch of 1.25–1.5 mm, a ratio falling from about 5.5 at rest to about 5.0 at
let-off, modern actions between 4.9 and 5.3): they trace to no source in the
corpus.

**Do not use** the distance from the shank center to the knuckle's contact point as
the knuckle's lever arm, nor a table turning the knuckle distance into an
"effective arm" by Pythagoras (17.0 mm across and 13.0 mm down giving 21.4 mm): the
arm is the moment arm of the contact normal about the center pin, and it moves as
the knuckle rolls on the jack ([FOUNDATIONS > The lever arm, and the gradient of a gap](../FOUNDATIONS.md#the-lever-arm-and-the-gradient-of-a-gap)).

### Regulation trade-offs imposed by the ratio

Spurlock's bands, for his gauge ratio of [The action ratio: four definitions, four values](#the-action-ratio-four-definitions-four-values) [V, Spurlock, p. 3]:

| Ratio | Consequences |
|---|---|
| **≤ 5** | Seldom any weight problem, but **very deep key dip** and/or a **short blow distance**; the sharps set extra high so they do not bury; power reduced by the short blow |
| **5.3–5.9** | Most pianos |
| **≥ 6.2** | Heavy unless the hammers are very light; reaching the down weight with many leads makes the action **terribly heavy when played fast**; the better compromise is fewer leads and a **D of 60–65 g** |

**The levers for changing the ratio**: the **capstan's position**, the simpler one,
and the **knuckle's distance from the shank center pin** [V, Spurlock, pp. 3, 5];
and the key's **bearing point**: cutting the balance rail punchings so the key bears
just behind its pin drops the action ratio by three to four tenths [V, Stanwood
2014, p. 8] ([FOUNDATIONS > Named simplifications](../FOUNDATIONS.md#named-simplifications), [FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)).

### Half-stroke line ("half-stroke line" / "magic line" / "line of centers")

Design condition: **the balance point of the key, the capstan/heel contact point and
the wippen center on one straight line when the key is at half stroke** [V, WNG,
p. 6]. Stanwood puts the capstan contact point "in a line between the wip center
and the balance point of the key" [V, Stanwood, pianotech list]. Spurlock asks that
the capstan/wippen contact point cross the line of centers somewhere in the key's
travel, to minimize capstan friction, and corrects it when it does not come within
**2 mm** of the line [V, Spurlock, p. 5]. The naturals' and sharps' balance points
differ, so one line cannot serve both: WNG raises the sharps' heels by typically
**2 mm**, giving two half-stroke lines [V, WNG, p. 6]. Corrections: the wippen heel,
the knuckle diameter, the hammer bore, the **height of the action brackets on the
key frame** [V, Spurlock, p. 5].

### The makers' specifications side by side

| Dimension | Steinway New York (S, M, L, O, A, B / C, D) | Steinway Hamburg (S–B / C, D) | Yamaha G1–C7 | Kawai RX, SK, GX, EX |
|---|---|---|---|---|
| Natural height | 2 19/32" = 65.88 mm **or** 2 21/32" = 67.47 mm [?, the two sources disagree] | 63 / 65 mm | 64 mm (+1/−3) | 66 or 68 mm |
| Sharps above | 1/2" = 12.7 mm | — | 12 mm (+0.5/−0) | 12.5 mm |
| Key dip | 0.400" = 10.16 mm (0.390–0.420"); 13/32" on the sheet | 9.9–10.6 mm | 10 mm | 10.1 mm (block); 10.1 or 10.3 mm by model |
| Blow distance | 1 3/4" = 44.45 mm / 1 7/8" = 47.63 mm | 47 mm; 45–47 mm in the checklist | 48 mm | 46 mm |
| Let-off | 1/32–1/16" = 0.79–1.59 mm | 1.5 / 1.0 / 1.0 mm | 3 to 1.5 mm | 2.0–2.5 / 1.0–1.5 mm |
| Drop | ≤ 1/16" **below** let-off | 2 mm **below** let-off | let-off **+ 2.0 mm**, from the string | the initial fall **equals** the let-off |
| Aftertouch | 0.040–0.055" = 1.02–1.40 mm (or 3/64–1/16") | 0.8–1.2 mm | — | by feel |
| Checking | 1/2–5/8" = 12.7–15.9 mm | 15 mm | 16 to 14 mm | 15 mm |
| Damper lift timing | 1/8" of key travel, ~1/3 of the hammer travel | — | 24 mm (±1) | 23 mm |
| Lever / jack height | the lever "slightly above", then rolling the flies | the flies lower than the balanciers | 0.2 mm | 0.1–0.2 mm |
| Jack position | far side of the fly at the far side of the knuckle insert | back edge of the fly at the back edge of the knuckle core | rear edge of the jack at the rear edge of the knuckle core | back of the jack at the back of the knuckle's wood core |
| Spread | — | — | 112.5 mm | — |
| D / U | 50 to 47 g (S–B), 51 to 46 g (C, D), bass to high treble / ≥ 20 g | 47 g (S–B), 52 to 47 g (C, D) / ≥ 20 g | — | — |
| Sources [V] | Steinway ch. 1, pp. 16–17; ch. 2, pp. 4–13; ch. 3, p. 3; Spec sheet, pp. 2–3 | Spec sheet, pp. 2–3; Steinway ch. 1, pp. 29–30 | Spec sheet, pp. 3–4 | Kawai, pp. 13–23; Spec sheet, p. 1 |

Each cell's page is given in [Master table of the keyboard and action settings](#master-table-of-the-keyboard-and-action-settings) and [Damper regulation settings](#damper-regulation-settings).

---

## The order of regulation (basis of the training mode)

### French reference sequence: the Potter checklist, 53 steps

The grand piano regulation checklist of *The Piano Action Handbook* (Randy Potter
School), in Marc Valdeyron's 1999 French translation [V, Potter checklist, p. 2]:

**Preparing the piano**: 1 remove the action · 2 clean the interior · 3 tighten the screws

**Preparing the action**: 4 file the hammers · 5 repairs · 6 clean the action · 7 key frame: bedding the front rail · 8 key frame: bedding the balance rail · 9 keytops · 10 key bushings · 11 key spacing / alignment · 12 front mortise play · 13 balance mortise play · 14 balance hole play · 15 damper guide play · 16 traveling the hammers · 17 burning the hammer shanks · 18 align action / piano (struck) · 19 string spacing · 20 spacing the hammers to the strings · 21 check the spring tension (indispensable minimum) · 22 check let-off / drop (indispensable minimum)

**Action regulation** (first regulate 3 to 5 reference notes with steps 23–46, to check the dimensions): 23 level the naturals · 24 level the sharps · 25 wippen / knuckle / capstan alignment · 26 jack / repetition lever centering · 27 jack / knuckle alignment · 28 jack / repetition lever height · 29 **blow distance** · 30 hammer stop rail height · 31 **let-off** · 32 **drop** · 33 **key dip, naturals** · 34 sharp dip · 35 check the **aftertouch** · 36 back check alignment · 37 back check slopes · 38 **checking** · 39 repetition spring tension · 40 key stop rail · 41 dampers: individual lift timing (key) · 42 dampers: collective lift (pedal) · 43 damper pedal · 44 damper stop rails · 45 sostenuto · 46 shift pedal (una corda)

**Voicing**: 47 fine tuning 440 Hz · 48 hammer seating/impregnation · 49 pre-voicing · 50 voicing finish

**Final check**: 51 repetition · 52 touch of individual keys · 53 test play

**"Quick regulation" (4 steps)**: blow distance (29) → springs (39) → let-off (31) → checking (38)

**"Fine regulation check"**: 29 blow distance/keyboard level, 31 even let-off, 32 even drop, 33/34 key dip, 35 even aftertouch

> This is **the** skeleton for the training mode ([FOUNDATIONS > Stages](../FOUNDATIONS.md#stages), stage 3, planned):
> French, ordered, separating preparation, regulation and checking, with two short
> paths already given ("quick regulation" and "fine check").

### Kawai sequence, 24 steps [V, Kawai, p. 2]

1 tighten the plate screws · 2 strike point & key frame position · 3 easing the keys (front holes, then balance holes) · 4 bedding the key frame · 5 travel & hammer angle · 6 spacing the hammers to the strings · 7 levers to knuckles, wippens to capstans · 8 repetition spring · 9 jack alignment & height · 10 key height and squaring · 11 blow distance · 12 hammer rest rail · 13 let-off · 14 drop · 15 key dip (naturals, then sharps) · 16 back checks (angle, alignment) · 17 checking (back stop, 15 mm) · 18 repetition spring (re-check) · 19 balance rail studs · 20 dampers · 21 damper upstop rail · 22 damper pedal · 23 sostenuto · 24 una corda

### Steinway sequence [V, Steinway ch. 1, p. 2; ch. 2, p. 2; ch. 3, p. 2]

**Preparation (ch. 1)**: preliminaries → bedding the key frame → balance rail studs → easing the keys → leveling the naturals → sharp height → servicing the centers (friction 1–4 g) → traveling → burning → spacing the hammers to the strings → spacing the wippens → spacing the flies.

**Regulation (ch. 2)**: fly position → height of the *balancier* (repetition lever) → blow distance / hammer line → let-off → drop → key dip → sharp dip → aftertouch → back checks → repetition springs → key frame shift screw.

**Dampers and sostenuto (ch. 3)**: damper "touch" (individual lift timing) → bends below the guide rail → bends above it → re-check of the lift → stop rail → damper pedal travel → height of the lift tray → sostenuto (alignment, tabs in line, height to the gauge, fore-and-aft position, travel).

### A 50-point "complete shop" checklist (Jurjens) [?, not in the corpus]

A grouping useful for the interface: **I. Keys and key frame** (1–14) · **II. Action off the key frame** (15–25: centers, traveling, re-graphiting the top of the jack and the lever's window, rounding and filing the knuckles…) · **III. Alignments, action reinstalled** (26–30) · **IV. Touch** (31–39: jack/lever height, blow distance, let-off, drop, aftertouch, checking, spring, **weighing the 88 notes**, stop rail) · **V. Dampers and pedals** (40–50).

### Bench or piano: a map of the two applications

**On the bench, action out (the single-key bench)**: jack position and centering
[V, Steinway ch. 1, p. 28]; jack to repetition lever height [V, Kawai, p. 13];
spacing the wippens, jacks and levers [V, Steinway ch. 1, pp. 27–28]; traveling and
burning [V, Steinway ch. 1, pp. 20–21]; hammer rest rail height; re-graphiting, and
lubricating the knuckles and the spring notches [V, Igrec, pp. 157–158]; a **first**
setting of the repetition springs, finished in the piano [V, Steinway ch. 2, p. 15];
the **drop**, action on the table [V, Steinway ch. 1, p. 30], or keyboard half on
the lap or on the padded front lid [V, Igrec, p. 173]; **most damper settings**,
"Almost all damper adjustments must be done with the action removed" [V, Redekop,
p. 1]; the sostenuto rod's height, to the gauge, action on the bench [V, Steinway
ch. 3, p. 9]; weighing D and U, dampers out of circuit [V, Spurlock, p. 1].

**In the piano**: the **let-off**, "This operation must be done in the piano for
exacting regulation" [V, Steinway ch. 2, p. 7]; the **aftertouch**, "very sensitive
to the curvature of the key bed/bench, and should be adjusted inside the piano" [V,
Igrec, p. 173]; the blow distance of each section's **guide** hammers, the rest then
set on the bench [V, Steinway ch. 2, p. 6]; the backchecking of the guides [V,
Steinway ch. 2, p. 13]; the **balance rail studs**, keyblocks tight, re-checked
after pressing the shift pedal three or four times [V, Steinway ch. 1, p. 12] and
with the damper pedal down [V, Kawai, p. 21]; a re-check of the **drop** [V, Kawai,
p. 17]; the **final inspection of the repetition springs** [V, Steinway ch. 2,
p. 15]; collective damper lift, pedals, the sostenuto rod's position [V, Steinway
ch. 3, p. 10]; the una corda stop [V, Kawai, p. 27]; ppp tests, "playing off the
jacks" [V, Igrec, p. 174].

**Test-bench pitfalls**, to reproduce as artifacts in the single-key bench (planned):
- the keyboard on the lap: the key frame's front rail bends upward and **the key dip
  decreases**, so keys may reach the bottom before let-off [V, Igrec, p. 173];
- finger pressure changes the key dip reading, light pressure giving a deeper dip
  [V, Steinway ch. 2, p. 9];
- the hammer's rest height **changes from one bounce to the next**: it depends on
  the repetition spring's tension, its friction against the lever, the friction
  between lever and knuckle and that of the lever's center pin [V, Igrec,
  pp. 173–174].

---

## Pathologies to reproduce (diagnostic engine, planned)

Each pathology is a **predicate on simulation observables** (strike velocity, number
of string contacts, the instant the jack leaves the knuckle, checking height, return
time), **never** a rule on the parameters: it then emerges from the model, which is
the point of teaching with one.

| Symptom | Simulated observable | Setting(s) at fault | Source |
|---|---|---|---|
| **Blocking** of the hammer against the string, "pinched", hard tone | string contact held beyond a threshold; near-zero velocity at impact | let-off **too close**; drop too small; jack too deep under the knuckle | [V, Igrec, p. 174; Oleg glossary, p. 1]; the jack's depth [?, the corpus ties a deep jack to friction and poor repetition, Steinway ch. 2, p. 4, not to blocking] |
| **Loss of power**, no control in soft playing | reduced strike velocity; a higher threshold for sound | let-off **too far**; blow distance too short | [V, Steinway ch. 2, pp. 6–7] |
| **Hammer bobbling** ("fluttering", "rattling") in soft playing, an unintended double strike | ≥ 2 string contacts for one stroke; the hammer bouncing between string and jack | **insufficient aftertouch**, the jack not escaping far enough from under the knuckle; insufficient key dip; centers **too loose**, bobbling on hard blows; repetition spring **too strong** | [V, Igrec, pp. 142, 173; Oleg glossary, p. 1]; the spring [?, not in the corpus] |
| **Failed double escapement**, weak repetition | the repetition lever does not lift the knuckle back; slow re-arming of the jack | repetition spring too weak; checking too low; drop too large | [V, Steinway ch. 2, pp. 12, 14]; the drop [?, not in the corpus] |
| The hammer **does not check**, a double or triple strike | no checking detected; the hammer returns toward the string | checking too far; a back check too vertical, the tapped hammer going all the way down; worn leather | [V, Kawai, p. 19; Steinway ch. 2, p. 12]; the double strike [?, not in the corpus] |
| The hammer tail **drags or blocks** on the back check in fortissimo | non-zero tail/back check force during the rise | back check too close; angle **too steep** | [V, Kawai, p. 19; Steinway ch. 2, p. 13] |
| **Lost motion**: precision and power lost | delay between the start of key travel and the start of hammer travel | capstan too low, the shanks resting on their rest rail instead of on the capstans; repetition lever **too high**, holding the knuckle clear of the jack; felts packed down | [V, Steinway ch. 1, p. 9; ch. 2, p. 5; Kawai, p. 13; Oleg glossary, p. 1, for the upright] |
| The hammer **drops and does not come back** when the jack heel is pressed | static test | repetition lever **too low** | [V, Kawai, p. 13] |
| The hammer **does not move** when the jack heel is pressed | static test | repetition lever **too high** | [V, Kawai, p. 13] |
| **Sharps buried** below the naturals at the bottom of the dip | the sharp below the natural at the end of travel | key dip too deep; a low ratio, the sharps to be set extra high | [V, Steinway ch. 2, p. 9; Spurlock, p. 3] |
| **Heavy touch** with a normal D | high inertia, normal D and U | heavy hammers **with extra lead**; ratio ≥ 6.2 | [V, Spurlock, pp. 3–4] |
| **Heavy touch** with an abnormally high D | F over 15 g in the bass or 12 g in the treble | friction: knuckles dirty, oily or not lubricated; tight bushings, balance holes or centers; neighboring parts rubbing; wool fibers | [V, Spurlock, pp. 2, 4, 6] |
| **Light touch**, low D | D below range | very worn or filed hammers; **very loose** shank centers | [V, Spurlock, p. 4] |
| **Slow repetition** | long key return | **U below about 18 g**: high friction, hammers too light, excess lead | [V, Spurlock, p. 5] |
| One note **much heavier** than its neighbors | local F outside the ±1 g tolerance | tight bushing, **or** rubbing with a neighbor (test: depress the adjacent keys) | [V, Spurlock, p. 7] |
| One note with an **abnormally low F** | local F below average | **loose shank center**, a sure sign | [V, Spurlock, p. 2] |
| **Dull tone**, shorter sustain, no acoustic cause | longer hammer/string contact | **high shank center friction** | [V, Igrec, p. 143] |
| **Late** damper lift: a lighter touch | the damper lifting beyond half the hammer travel | individual lift timing set late | [V, Steinway ch. 3, p. 3] |
| **Early** damper lift, interfering with let-off | the under-lever's force felt before let-off | individual lift timing early; lift rail too high | [?, Redekop's "Too light interferes with the let-off", p. 3, follows a truncated sentence] |
| **Damper bounce** on the pedal | the damper head oscillates | the pedal lifts **higher** than the keys; pedal stop badly set | [V, Redekop, p. 4; Kawai, p. 25] |
| **Catching** on the sostenuto, notes not held staying up | a tab caught when it should not be | rod **too close** to the tabs | [V, Steinway ch. 3, p. 8] |
| Some dampers **fall** when the damper pedal is released with the sostenuto down | hold lost | rod **too far** from the tabs | [V, Steinway ch. 3, p. 8] |
| **"Whoosh" / "oink"** of the trichord wedge dampers | felt rubbing the string during the lift | excess felt below the string line, the head's seating, the una corda shift | [V, Steinway ch. 3, p. 7; Redekop, p. 4] |
| **Knocking** from the key frame | intermittent key frame/keybed contact | key frame bedding, balance rail studs, humidity (the front rail lifts at its ends in dry weather) | [V, Steinway ch. 1, pp. 10, 12; Kawai, p. 7] |
| "**Zing**", string vibration felt in the key at let-off in the bass | the hammer touching the string during a slow pass through let-off | let-off too close in the tenor and bass, where the strings move further | [V, Igrec, p. 174] |

**Diagnostic tests, the instruments of the app (planned)**: "playing off the jacks",
at ppp with the damper pedal down, every note sounding; "dropping the weight", about
100 g dropped onto the key, pedal down, a faint sound expected, the weight reduced
until notes go missing; the "bump" test, four adjacent naturals compared for where
the bump in their travel begins; a fast double strike with the pedal down, a let-off
too close giving a harsh second strike [V, Igrec, p. 174]; the free fall of the key
tail lifted 1–2 mm [V, Kawai, p. 6].

---

## Bilingual glossary FR ↔ EN (+ DE) and code identifiers

Identifiers are English snake_case, in five namespaces: parts, bare ([Parts](#parts)); `spec.`
the regulation settings, `tw.` the touchweight quantities and `geom.` the geometry
([Regulation settings and quantities](#regulation-settings-and-quantities)); `op.` the operations ([Operations](#operations)). A value in a trade unit carries its unit as a
suffix (`spec.key_dip_mm`, `tw.downweight_g`); ratios and SI values carry none.
**Planned**: these identifiers key the persisted schema
([FOUNDATIONS > The parameter registry and the validation layer](../FOUNDATIONS.md#the-parameter-registry-and-the-validation-layer)). The built code's field names, such as `capsule_height`
and `front_board`, do not follow them. The page's French catalog (built)
words the engine's codes with the French names of these tables
([FOUNDATIONS > A bilingual interface](../FOUNDATIONS.md#a-bilingual-interface)).

The French column shows the name the interface uses first, then, in italics, other
names a reader will meet for the same part. The German column holds only the terms
a French–English–German trade legend gives for the part; where none does, it is
empty rather than filled from a dictionary.

### Parts

| FR | EN | DE | `identifier` |
|---|---|---|---|
| Plateau de clavier / fond de clavier | keybed | Stuhlboden | `keybed` |
| Châssis de clavier | key frame | Klaviaturrahmen | `key_frame` |
| Barre de balancier | balance rail | — | `balance_rail` |
| Pointe de balancier | balance rail pin | — | `balance_rail_pin` |
| Mouche / rondelle de balancier | balance rail punching / bearing | — | `balance_rail_punching` |
| Barre d'enfoncement | front rail | — | `front_rail` |
| Pointe d'enfoncement | front rail pin | — | `front_rail_pin` |
| Mouche d'enfoncement | front rail punching | — | `front_rail_punching` |
| Mouche en papier (also *mouche de papier*; the thicker ones, *mouches en carton*) | paper punching | — | `paper_punching`: laid under a felt punching; the papers on hand are an *assortiment de mouches*, and those under one punching a *pile* |
| Barre de repos arrière | back rail | — | `back_rail` |
| Drap de repos arrière | back rail cloth | — | `back_rail_cloth` |
| Vis de réglage de portée du châssis (also *vis de portée*) | balance rail stud / glide bolt | — | `balance_rail_glide` |
| Barre de contre-touche (also *barre d'arrêt des touches*) | key stop rail | — | `key_stop_rail` |
| Bloc / joue de clavier | keyblock / cheekblock | — | `keyblock` |
| Montant de mécanique | action bracket | Mechanikbacken | `action_bracket` |
| Barre des chevalets | wippen rail | Hebegliedleiste | `wippen_rail` |
| Barre des marteaux | hammer rail / main action rail | Mechanikbalken | `hammer_rail` |
| **Sommier** | **pinblock / wrest plank** | — | `pinblock`. The action rails are « barres » ([Key frame, rails, structural members](#key-frame-rails-structural-members)) |
| Barre d'échappement (also *barre des boutons d'échappement*) | let-off rail / regulating rail | Auslösepuppenleiste | `let_off_rail`: one rail, under the hammer rail, carrying the let-off buttons; the interface says **barre d'échappement** |
| Barre de repos des marteaux (also *barre de butée des marteaux*, *barre de butée des manches*; its cloth: *drap de butée des manches*) | hammer rest rail | Hammerruheleiste | `hammer_rest_rail` |
| Touche | key | Taste | `key` |
| Corps de la touche | keystick | — | `keystick`: the key's wooden body, without its covering, its key button and the blocks glued on it |
| Touche blanche / marche | natural / white key | — | `natural_key` |
| Dièse / feinte | sharp / black key | — | `sharp_key` |
| Revêtement de touche | keytop / key covering | — | `keytop` |
| Fronton | key front | — | `key_front` |
| Capsule de touche | key button / chase | — | `key_button` |
| Mortaise (de balancier / d'enfoncement) | balance / front mortise | — | `balance_mortise`, `front_mortise` |
| Garniture de mortaise | bushing (cloth) | — | `mortise_bushing`; the balance mortise's, set into its two long sides in its upper part, `balance_mortise_bushing` |
| Casimir | bushing cloth | — | `bushing_cloth`: the thin woven wool a mortise is bushed with |
| Trou de balancier | balance hole | — | `balance_hole`: through the floor under the balance mortise, round, fitted to the pin, which it holds fore and aft |
| Trou latéral (de la capsule de touche) | key button side hole (descriptive) | — | `key_button_side_hole`: through the key button's wall, from the balance mortise's slot to its side, taller than the cloth, where the bushing's cloth turns out and is glued against its top, flush with the side |
| Patin de balancier | key shoe | — | `key_shoe` |
| Plomb de touche | key lead | — | `key_lead` |
| **Pilote (de touche)** | **capstan (screw)** | — | `capstan` |
| Tête du pilote | capstan head | — | the capstan's wider cylindrical part, above its threaded shank |
| Dôme (du pilote) | capstan dome | — | the rounded top of the head, a spherical cap, that bears on the wippen's heel; its radius is never under the head's |
| Bloc d'attrape | back check block | — | `back_check_block`: the wooden block glued on the key that carries the back check |
| Dégagement | wippen rail clearance | — | `top_clearance`: the shallow cut in the top of the key, between the capstan and the back check block, that clears the wippen rail; on many keys, not all |
| Bout de touche (also *arrière de la touche*) | key end / tail | — | `key_tail`. Not « talon », the wippen's heel |
| Bout aminci | thinned tail | — | the end of the key, thinner than the keystick |
| Épaulement | the step down to the thinned tail | — | where the keystick's full height steps down to the thinned tail |
| Chanfrein | chamfer (under the tail) | — | the bevel under the tail, cut so that the key rests flat on the back rail cloth |
| Feutre de bout de touche | key tail felt / key end felt | — | `key_tail_felt`: the felt on the tail that lifts the damper under-lever |
| **Chevalet (de mécanique)** | **wippen assembly** | Hebeglied | `wippen_assembly`: the whole wooden sub-assembly, body, heel, jack, repetition lever, springs (note below) |
| Corps du chevalet | wippen (the body alone) | — | `wippen`: the single rigid body that carries one generalized coordinate |
| Chevalet (de table d'harmonie) | bridge | — | `bridge`: the wooden bar coupling the strings to the soundboard |

> **« Chevalet » names two different things in a grand**: on the *structure
> harmonique*, the wooden bar that links the strings to the soundboard; in the
> *action*, **the assembly of wooden parts**: heel, repetition lever, jack and the
> rest. Every grand piano technician knows both senses and tells them apart at a
> glance from the context, so the interface says plain **chevalet** in the
> action's panels and qualifies, **chevalet de mécanique** and **chevalet de
> table**, only where the bridge appears too.
>
> The second sense is not only a naming point. In the planned dynamics the wippen
> body, the jack relative to the wippen and the repetition lever relative to the
> wippen are the three coordinates whose couplings form one block of the mass
> matrix, the only one with off-diagonal terms; the jack and the lever couple to
> the body but not to each other, `m_jr = 0` exactly ([FOUNDATIONS > Relative coordinates for the jack and the repetition lever](../FOUNDATIONS.md#relative-coordinates-for-the-jack-and-the-repetition-lever)). That block
> is the *chevalet*, written as a matrix. A technician buys and replaces a chevalet
> as one part; the solver treats the same three bodies as one block.
>
> Consequence for identifiers: `wippen` is the **body alone**, since it is the body
> that carries a generalized coordinate. Anything addressing the assembly (a preset,
> a part swap, a price, a mass) is `wippen_assembly`.

| FR | EN | DE | `identifier` |
|---|---|---|---|
| Talon de chevalet | wippen heel | Hebegliedsattel | `wippen_heel` |
| Feutre / drap de talon | heel cloth | — | `wippen_heel_cloth` |
| Fourche de chevalet | wippen flange | Hebegliedkapsel | `wippen_flange` |
| Coussin de chevalet | wippen cushion | — | `wippen_cushion` |
| **Bâton (doigt) d'échappement** | **jack** (Steinway: *fly*) | Stoßzunge | `jack` |
| Pied du bâton d'échappement | jack tender / heel | Stoßzungenarm | `jack_tender` |
| Ressort de bâton | jack spring | — | `jack_spring` |
| Cuillère (de chevalet) | jack stop (descriptive): what the button carried by the jack rests against, which sets the jack's rest under the knuckle | — | `jack_stop`, with `form: Spoon`, a metal spoon on the wippen, on wippens with a chevron repetition spring; `form: FeltedBlock`, a wooden stop with a felt pad, on wippens with a Schwander-type spring. Not the « cuillère de bascule » (`underlever_spoon`) |
| **Bouton d'échappement** | **let-off button / regulating button** | Stoßzungenpuppe | `let_off_button`: the generic term |
| Bouton d'échappement, in its dowel form | long wooden let-off dowel | Stoßzungenpuppe | `let_off_button` with `form: Dowel`: the Schwander form, screwed to a threaded point in the underside of the hammer rail. Still **bouton d'échappement** in French: the form is a property of the part, not a second name |
| Ressort auxiliaire (de chevalet) | wippen support spring (also *wippen assist spring*, *helper spring*; a wippen fitted with one: "turbo wippen") | — | `wippen_support_spring`: one spring, under every English name, the one Stanwood's support spring balance weight is read with |
| Tige de bouton d'échappement | let-off dowel / wire | — | `let_off_dowel` |
| **Levier de répétition** | **repetition lever** (Steinway: *balancier*) | Repetierschenkel | `repetition_lever` |
| **Vis de chute** | **drop screw** | Abnickschraube | `drop_screw`: above the repetition lever, it stops the lever after let-off and so sets the drop |
| Vis de réglage de la hauteur du levier de répétition | repetition lever height screw (descriptive) | Abstellschraube | `repetition_lever_height_screw`: on the wippen, it sets the lever's height against the top of the jack. At the very back of a wippen with a chevron spring; toward the middle of a wippen with a Schwander-type spring, above the felted stop of the jack's button |
| **Ressort de répétition** | **repetition spring** | Repetierfeder | `repetition_spring`: two forms on modern wippens, the chevron spring (« ressort papillon », Herz type), adjustable by a screw on some and by bending on others, and the Schwander-type spring |
| Vis de tension du ressort | repetition spring tension screw | Repetierfederschraube | `repetition_spring_screw`: at the back of a wippen with a Schwander-type spring; on a wippen with a chevron spring, where there is one, toward the middle, at the top of the chevron; where there is none, the spring is bent to adjust its tension |
| **Attrape** | **back check** | Fänger | `back_check` |
| Garniture / peau d'attrape | back check leather / felt | — | `back_check_leather` |
| Fil d'attrape | back check wire | — | `back_check_wire` |
| Contre-attrape *(piano droit uniquement)* | catcher | — | `catcher` *(upright)* |
| Lanière *(piano droit uniquement)* | bridle (tape) | — | `bridle_tape` *(upright)* |
| Marteau (ensemble) | hammer assembly | — | `hammer` |
| Tête de marteau | hammer head | Hammerkopf | `hammer_head` |
| Feutre de marteau | hammer felt | — | `hammer_felt` |
| Manche de marteau | hammer shank | Hammerstiel | `hammer_shank` |
| Olive (du manche de marteau) | hammer shank flange | Hammerstielkapsel | `hammer_flange` |
| **Rouleau** | **knuckle** | Rolle | `knuckle` |
| Âme du rouleau | knuckle core / spline | — | `knuckle_core` |
| Queue de marteau | hammer tail | — | `hammer_tail` |
| Noix (de marteau) *(piano droit)* | hammer butt | — | `hammer_butt` *(upright)* |
| Axe / center / pivot | center pin | — | `center_pin` |
| Bascule / levier d'étouffoir | damper (under)lever | Dämpferarm | `damper_underlever` |
| Fourche de bascule (d'étouffoir) | underlever flange | Dämpferkapsel | `underlever_flange` |
| Noix d'étouffoir | damper block (clamps the wire) | Dämpferklotz | `damper_block`: the small wooden block at the top of the under-lever's upright, drilled for the damper wire and clamping it with a screw; on some under-levers fused with the upright, on others a separate block on an axle, `form: Fixed` or `form: Articulated`. The interface says **noix d'étouffoir**; **taquet d'étouffoir** names the same block. « Bloc d'étouffoir » is said loosely for the whole damper assembly, not for this block |
| Tige d'étouffoir | damper wire | Dämpferdraht | `damper_wire` |
| Galerie des étouffoirs | damper guide rail | Führungsleiste | `damper_guide_rail` |
| Tête d'étouffoir | damper head | Dämpferpüschel | `damper_head` |
| Barre de forte | damper lift rail | Dämpferabhebeleiste | `damper_lift_rail` |
| Barre des bascules d'étouffoir | under-lever rail | Dämpferleiste | `underlever_rail` |
| Barre de butée des bascules d'étouffoir | under-lever stop rail | Dämpferpralleiste | `underlever_stop_rail` |
| Bec de sostenuto (also *bec de tonale*) | sostenuto tab | — | `sostenuto_tab`: the tab on each damper that the rod catches. A maker's notice calls it « languette de sostenuto » and gives « bec » to the rod's lip instead: read « bec » in a text by what it is fixed to |
| Barre de sostenuto (also *tringle de sostenuto*, *barre de tonale*; its bracket *support de sostenuto*, *support de tonale*) | sostenuto rod / blade | — | `sostenuto_rod` |
| Fourche supérieure | underlever top flange | — | `underlever_top_flange` |
| Pilote de bascule d'étouffoir | underlever capstan | Regulierpilote | `underlever_capstan` |
| Cuillère de bascule | underlever spoon | Dämpferlöffel | `underlever_spoon` |
| Vis de tige d'étouffoir | damper wire screw | — | `damper_wire_screw` |
| Feutre d'étouffoir (plat / coin) | damper felt (flat/block, wedge) | — | `damper_felt_flat`, `damper_felt_wedge` |
| Coulisseau d'étouffoir | guide rail bushing | — | `damper_guide_bushing` |
| Barre de butée des étouffoirs | damper (up)stop rail | Dämpferpralleiste | `damper_stop_rail` |
| Ressort d'étouffoir | damper (assist) spring | — | `damper_spring` |
| Lyre | lyre | — | `lyre` |
| Pédale forte (pédale de soutien; also *pédale des étouffoirs*) | damper / sustaining pedal | — | `damper_pedal` |
| Pédale sostenuto (pédale centrale; also *pédale tonale*) | sostenuto pedal | — | `sostenuto_pedal` |
| Pédale douce (also *pédale una corda*, *pédale de transposition*) | soft / shift / una corda pedal | — | `shift_pedal` |
| Écrou de réglage de tige de pédale | pedal rod head / capstan | — | `pedal_rod_head`. Not « poupée » for the let-off button |
| Mouvement / levier de pédale | trapwork lever | — | `trapwork_lever` |
| Prisonnier (tourillon de transmission du forte) | pitman (dowel to the damper lift) | — | `lift_dowel`. « Pitman » is the English trade's word, not said in French |
| Tige de pédale | pedal rod | — | `pedal_rod` |
| Coupelle de pédale | pedal cup | — | `pedal_cup` |
| — **[?]** | keyframe shift screw | — | `keyframe_shift_screw`: the screw that stops the key frame's shift; its French name is not attested, and is the owner's to give |
| Bloc d'arrêt du châssis | action stop block | — | `action_stop_block` |
| Cale arrière du châssis de clavier | dag block | Klaviaturrahmenhaken | `dag_block` |

> **Open in these tables**: the rows marked **[?]**.

### Regulation settings and quantities

| FR | EN | `identifier` | Unit |
|---|---|---|---|
| Hauteur de touche | key height | `spec.key_height_mm` | mm |
| Hauteur des dièses | sharp height | `spec.sharp_height_mm` | mm |
| Enfoncement | key dip | `spec.key_dip_mm` | mm |
| Enfoncement des dièses | sharp dip | `spec.sharp_dip_mm` | mm |
| **Chasse** | **blow distance** | `spec.blow_distance_mm` | mm |
| Ligne des marteaux | hammer line | `spec.hammer_line_mm` | mm |
| **Échappement** | **let-off** | `spec.let_off_mm` | mm (from the string) |
| **Chute** | **drop** | `spec.drop_abs_mm` / `spec.drop_below_let_off_mm` | mm, stored with its reference ([FOUNDATIONS > Persistence](../FOUNDATIONS.md#persistence)) |
| **Gras / jeu après échappement** | **aftertouch** | `spec.aftertouch_mm` | mm |
| **Attrapé** | **checking / back check distance** | `spec.check_height_mm` | mm (from the string) |
| Pente d'attrape | back check angle | `spec.back_check_angle_deg` | ° |
| Hauteur du bâton / levier | jack-to-repetition-lever height | `spec.jack_height_mm` | mm; set by the **vis de réglage de la hauteur du levier de répétition** |
| Position du bâton | jack position | `spec.jack_position_mm` | mm |
| Jeu au pilote | lost motion | `spec.lost_motion_mm` | mm |
| Tension du ressort de répétition | repetition spring tension | `spec.rep_spring_tension` | not chosen (N·mm or a gram equivalent); the suffix follows the choice |
| Hauteur de la barre de repos des marteaux | hammer rest rail height | `spec.hammer_rest_height_mm` | mm |
| Barre de contre-touche | key stop rail clearance | `spec.key_stop_clearance_mm` | mm |
| Départ individuel d'étouffoir | individual damper lift timing | `spec.damper_lift_timing_mm` | mm of blow distance |
| Départ général | collective damper lift | `spec.collective_lift_mm` | mm |
| Jeu de la barre de forte **[?]** | lift tray lost motion | `spec.tray_lost_motion_mm` | mm; the French phrase is composed from the rail's name, to confirm |
| Butée d'étouffoir | damper stop rail clearance | `spec.damper_stop_clearance_mm` | mm |
| Course de pédale forte | damper pedal travel | `spec.damper_pedal_travel_mm` | mm |
| Jeu de pédale | pedal lost motion | `spec.pedal_lost_motion_mm` | mm |
| Distance de transposition | shift travel | `spec.shift_travel_mm` | mm |
| Hauteur / rotation / distance du sostenuto | sostenuto rod height / rotation / clearance | `spec.sostenuto_height_mm`, `spec.sostenuto_rotation_deg`, `spec.sostenuto_clearance_mm` | mm, ° |
| Jeu des mortaises | bushing play | `spec.bushing_play_mm` | mm |
| Portée du châssis | keyframe bedding | `spec.keyframe_bedding` | — |
| Point de mesure (standard) | measuring point | `tw.measuring_point_mm` | mm back from the covering's front edge, 13 by default ([Touchweight: the quantitative core](#touchweight-the-quantitative-core)) |
| Poids d'enfoncement (D; also *poids descendant*) | downweight | `tw.downweight_g` | g |
| Poids de remontée (U; also *poids ascendant*) | upweight | `tw.upweight_g` | g |
| Friction (F; also *frottement*) | friction weight | `tw.friction_g` | g |
| Friction de la touche (KF) | key friction weight | `tw.key_friction_g` | g; the part of F the key's bushings give alone |
| Poids d'équilibre (BW) | balance weight | `tw.balance_weight_g` | g |
| Poids avant (FW) | front weight | `tw.front_weight_g` | g |
| Poids d'attaque (SW; also *poids de frappe*) | strike weight | `tw.strike_weight_g` | g |
| Rapport de poids d'attaque (R) | strike weight ratio | `tw.strike_weight_ratio` | — |
| Poids radial du chevalet (WW) | wippen weight | `tw.wippen_weight_g` | g |
| Rapport de poids de clavier (KR) | key weight ratio | `tw.key_ratio` | — (the force convention, about 0.5: [FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)) |
| Inertie | inertia | `tw.inertia` | kg·m² (no trade protocol measures it: [FOUNDATIONS > Named simplifications](../FOUNDATIONS.md#named-simplifications)) |
| **Rapport mécanique** (also *rapport de mécanique*) | **action ratio** | `geom.action_ratio` | — (four definitions, [The action ratio: four definitions, four values](#the-action-ratio-four-definitions-four-values)) |
| Rapport de clavier / du chevalet / du manche | key / wippen / shank ratio | `geom.key_ratio`, `geom.wippen_ratio`, `geom.shank_ratio` | — (Reyburn's lever ratios, [The action ratio: four definitions, four values](#the-action-ratio-four-definitions-four-values); `geom.key_ratio` is the capstan arm over the front arm, about 0.5, never its reciprocal) |
| Écartement mécanique (*écartement*; « spread ») | spread | `geom.spread_mm` | mm |
| Distance du rouleau | knuckle distance | `geom.knuckle_distance_mm` | mm |
| Diamètre du rouleau | knuckle diameter | `geom.knuckle_diameter_mm` | mm |
| Distance de perçage | bore distance | `geom.bore_distance_mm` | mm |
| Distance de pendage | hang distance | `geom.hang_distance_mm` | mm |
| Ligne magique (also *ligne de mi-course*) | half-stroke line / magic line | `geom.half_stroke_line_mm` | mm of deviation |

> « Hauteur de touche » is the height, a measured quantity; « dressage » is the
> leveling of the keys to one another, an operation ([Operations](#operations)).
>
> **The symbols stay Stanwood's English ones** (D, U, F, BW, FW, KR, KF, SW, SS,
> WW, R) beside the French names, so that a formula reads the same in both
> languages: BW + FW = (WW × KR) + (SW × R). French print has its own (Penf,
> Prem, Péq, Pav, Rcl, Pat, Pch; elsewhere Pd, Pa), which a reader will meet and
> the interface does not use.
>
> **Poids de jeu**, also *poids au toucher*, names the three weights together: a
> change « of the poids de jeu » moves D, U and BW by the same amount.
> **Do not translate** the English "nominally 2:1" as « rapport de clavier »: the
> French writes the key's lever ratio back arm over front arm, about 0.5, the
> same way round as KR and as `geom.key_ratio`.

### Operations

| FR | EN | `identifier` |
|---|---|---|
| Dressage des touches (also *niveler les touches*) | key leveling | `op.level_keys` |
| Défauchage | traveling (hammers, wippens): correcting a part that « fauche », rises drifting sideways, by shimming its flange | `op.travel_hammers` |
| Chauffage des manches | burning shanks: heating and twisting the shank so the hammer head stops leaning | `op.burn_shanks` |
| Donner du jeu (aux garnitures de mortaise) | easing keys | `op.ease_keys`. Not « dégagement », the clearance cut in the key ([Parts](#parts)) |
| Portée du châssis | bedding the keyframe | `op.bed_keyframe` |
| Mise des marteaux sur les cordes | spacing hammers to strings | `op.space_hammers` |
| Jours (de touches, de cordes) | spacing / clearances | `op.spacing` |
| Ponçage des têtes | hammer filing / reshaping | `op.file_hammers` |
| Harmonization | voicing | `op.voicing` |
| Plombage | key leading / weighing off | `op.key_leading` |
| Pesée (also *réglage du poids de jeu*) | weigh-off | `op.weigh_off` |
| Rebrochage / regoupillage | repinning | `op.repin` |
| Regarnissage | rebushing | `op.rebush` |

> « Faucher » names the defect, a part whose plane of rotation is not vertical and
> which drifts sideways as it rises; « défauchage » corrects it at the flange.
> « Fauchage » is not an operation: do not use it for burning, which is
> « chauffage des manches », a different correction of a different fault.

### At the bench: weighing and laying papers

The words the bench (built) shows around the weighing and the regulation, and its
views; its French catalog (built) takes them from here. The touchweight quantities themselves are in [Regulation settings and quantities](#regulation-settings-and-quantities).

| FR | EN | What it is |
|---|---|---|
| Banc d'essai | the bench | the page itself: the model of the grand action shown to be judged and corrected, not yet an application; the owner's name for it for the time being |
| Relevé du poids de jeu | weighing sheet | the sheet the readings are written on |
| Poids de pesée (a set: *jeu de poids*; stacked: *pile de poids*) | gram weights | the weights laid on the measuring point |
| Bord avant du revêtement | the covering's front lip | the edge the measuring point is measured back from |
| Balance (de précision) | scale | |
| Protocole Stanwood | Stanwood's protocol | |
| Appareil de mesure à faible friction | Stanwood's jig | the key pivoting at its balance hole on a point, its measuring point on the scale |
| Support à faible friction | the jig's roller | what the measuring point rests on, on the scale's pan |
| Pesée (de la touche) | weighing the key | the key weighed whole: its mass on a scale, its front weight on the appareil de mesure à faible friction, and where the roller stood. « Pesée » is also the weigh-off ([Operations](#operations)); the context tells them apart |
| Poids provisoire | temporary weight | a weight laid on the key for one reading, then taken off |
| Cote (de réglage) | target | the value a setting is regulated to |
| Position de repos | top seat | the key at rest on the back rail cloth |
| À plat sur le drap; sur le bord du drap; sur une arête de la touche | flat on the cloth; on the edge of the cloth; on the key's own corner | how the tail bears on the back rail cloth: its chamfer flat, the cloth's front or back edge pressing into the chamfer, or the key's edge (where the chamfer begins, or the key's end) pressing into the cloth. Where a tail does not bed flat, either edge happens |
| Équilibre des poids | the equation of balance | Stanwood's BW + FW = (WW × KR) + (SW × R) ([The equation of balance](#the-equation-of-balance)) |
| Écart de niveau, touche enfoncée | out of level, key down | how far the key stands from level when fully down, in degrees, its target 0°; nose-down positive |
| Assise du bout de touche | bedding at the back | the angle between the chamfer under the tail and the back rail cloth, the key at rest: 0° where the tail beds flat, positive where its rear end digs in. Not « portée », which is the key frame's on the keybed |
| Point d'appui sur le drap | where the key bears | how far behind the balance pin the tail bears on the back rail cloth, the key at rest |
| Fond de course (*position d'enfoncement maximal*) | bottom seat | the key down on its front punching |
| Course de la touche | stroke, travel | |
| Pile de mouches | punching stack | the papers under one felt punching |
| Assortiment de mouches | drawer (of papers) | the paper thicknesses on hand |
| Ajouter une mouche de 0,2 | add a 0.2 mm paper | how a paper is named and laid at the bench: by its thickness in millimeters, never as a stack's total (« régler à 0,37 » is not said) |
| Coupe | section (the view) | the key on its frame drawn flat, in the sagittal section |
| 3D | solid (the view) | the same key and frame in three dimensions, seen three-quarters, turned by hand |
| Courbe d'enfoncement et de remontée | the band | the force on the measuring point through the stroke, its *courbe d'enfoncement* on the way down and its *courbe de remontée* on the way up, the key held by friction anywhere between them: the bench's own measurement, which no weight reads |

---

## Terms easy to get wrong

Standard trade usage in France. Where two words circulate, the last column gives the
one this project uses.

| Term | What it is | The project uses |
|---|---|---|
| « sautereau » | the harpsichord's and spinet's jack, the vertical slip carrying the plectrum; not the piano's jack | **bâton d'échappement** (also *doigt d'échappement*) → `jack` |
| « bascule » | on a grand, the damper under-lever, **bascule d'étouffoir**; not the wippen, which is the **chevalet**: two distinct parts and two distinct words | **chevalet** → `wippen`; **bascule d'étouffoir** → `damper_underlever` |
| « limbe » | not in trade use for the action's frame or rails | **barres de mécanique** + **montants de mécanique**; **plateau**, **châssis de clavier** |
| « équerre » | not the trade term for the action bracket | **montant de mécanique** → `action_bracket` |
| « noix » | on a grand, the **noix d'étouffoir**, the block clamping the damper wire to the under-lever; the hammer butt (*noix*) is the upright's; never a flange | **fourche de chevalet**, **olive**; **noix d'étouffoir** → `damper_block` |
| « distance de frappe » | understandable, not the trade term | **chasse** |
| « post-échappement » | in circulation, for the aftertouch | **jeu après échappement**, **gras**, **réserve** |
| « chute » / « rechute » | both circulate for the quantity | **chute** for the quantity, **vis de chute** for the part |
| « cuillère » | two parts of a grand: the damper under-lever's spoon, on the lift rail, and the wippen's metal stop for the jack's button, on wippens with a chevron repetition spring | **cuillère de bascule** → `underlever_spoon`; **cuillère de chevalet** → `jack_stop` |
| « una corda » | the pedal's other name, after the strings a shifted hammer once struck | **pédale douce** |
| « feutre, cuir, buffle » | correct | — |

### Settled terms

| Question | Answer |
|---|---|
| let-off button | **bouton d'échappement**, whatever its form, a button turning on a screw or a long wooden dowel; « pilote d'échappement » is said of the dowel form, and the interface does not use it. Do not use « poupée » for it: in print « poupée » is the pedal rod's adjusting nut |
| shank flange | **olive du manche de marteau**; with the **fourche de chevalet**, the two flanges fixed to the flat of their rails |
| under-lever tray / lift rail | **barre de forte**; « barre de levée des étouffoirs » is a second name for the rail; « plateau de levée » is not said |
| action brackets | **montants de mécanique** |
| key stop rail | **barre de contre-touche** |
| glide bolt | **vis de réglage de portée du châssis**, named after the very thing it corrects, which also gives *portée du châssis* for frame bedding |
| key bearing | **support de touche au balancier**: a hemispherical stud on Steinway, **mouches de balancier** on other makes. The key bears on the punching, not on the pin; the bench pivots it at the pin, the bearing's offset a parameter set to zero ([FOUNDATIONS > Named simplifications](../FOUNDATIONS.md#named-simplifications)) |
| sostenuto | **sostenuto**: *barre de sostenuto*, its tabs the *becs de sostenuto* (a maker's « languettes », [Parts](#parts)); *tonale* (*barre de tonale*, *support de tonale*, *bec de tonale*) names the same parts, and the interface does not use it |
| trapwork | **mouvement de pédale** / **levier de pédale**; the dowel onto the damper lift is the **prisonnier** |
| damper parts | **tête**, **feutre**, **tige**, **galerie**, **noix**, **fourche**, **bascule**, **plomb de bascule**, **barre de butée des bascules** |

The open questions, each written so the owner can answer it without opening a file,
are in `docs/vocabulary-worksheet.md`.

---

## Consequences for the data model (planned)

1. **The layers of [FOUNDATIONS > The data model](../FOUNDATIONS.md#the-data-model)**: action design (geometry, measured masses,
   bearing point); regulation state, the settings of [The regulation settings](#the-regulation-settings), each with its value,
   range, source, keyboard zone and actuator, which is what the technician turns;
   material set, each interface's contact law (felt, leather, buckskin, cloth,
   back check leather, graphite or PTFE); per-note data, graduation curves over
   notes 1 to 88; string plan.
2. **Every dimension carries a range**: `{ value, min, max, unit, source_preset,
   section }`. The sections: `bass`, `tenor`, `mid`, `treble`, `high_treble`, the
   division of Steinway's touchweight specifications [V, Spec sheet, pp. 2–3].
3. **Two unit systems**, millimeters and fractional inches (1/32, 3/64, 1/16,
   13/32…) ([FOUNDATIONS > The acceptance criterion](../FOUNDATIONS.md#the-acceptance-criterion)): Steinway's New York figures are in inches; Hamburg,
   Yamaha and Kawai's in millimeters.
4. **The action ratio is an output**, in the four definitions of [The action ratio: four definitions, four values](#the-action-ratio-four-definitions-four-values), each
   labeled ([FOUNDATIONS > The two key ratio conventions; ratios are outputs](../FOUNDATIONS.md#the-two-key-ratio-conventions-ratios-are-outputs)); it varies along the stroke, the definitions differ
   on one action, and the technician meets all four in the field.
5. **Drop is stored with its reference**, from the string or below let-off, because
   makers write it both ways ([FOUNDATIONS > Persistence](../FOUNDATIONS.md#persistence)); the other form is derived for
   display, and a drop that does not clear the let-off, "eclipsing" it [V, Steinway
   ch. 2, p. 7], is reported.
6. **Dependency graph of the settings.** Each setting declares what it invalidates,
   shown as an out-of-specification badge, never corrected automatically
   ([FOUNDATIONS > Upstream settings drift; faults are reported, not refused](../FOUNDATIONS.md#upstream-settings-drift-faults-are-reported-not-refused)). From the sources: blow distance → aftertouch [V, Steinway
   ch. 2, p. 11], damper lift timing [V, Igrec, p. 158], hammer line [V, Steinway
   ch. 2, p. 6]; key dip → aftertouch [V, Steinway ch. 2, p. 11], damper stop rail
   [V, Igrec, p. 158]; let-off → drop [V, Steinway ch. 2, p. 7; Kawai, p. 17];
   checking → repetition spring [V, Kawai, p. 20]; key height → key dip, which
   runs from it, and aftertouch [V, Steinway ch. 2, p. 11]; lever height → lost
   motion [V, Steinway ch. 2, p. 5]; balance rail studs → key dip, forbidden [V,
   Steinway ch. 1, p. 13]. This graph drives the training mode ([FOUNDATIONS > Stages](../FOUNDATIONS.md#stages),
   stage 3).
7. **The 88 notes share the topology, not the values.** Graduation curves carry the
   strike weight (8.0–14.8 g at note 1 to 3.5–6.9 g at note 88), the front weight
   ceiling (41.3 to 3.7 g), friction (12–15 g to 8–10 g), let-off (3 to 1.5 mm, or
   1.5 to 1.0 mm), checking (16 to 14 mm), the repetition spring (slower in the
   bass in Steinway's guide, [FOUNDATIONS > Open questions for the owner](../FOUNDATIONS.md#open-questions-for-the-owner)), the under-lever leads (WNG standard
   2/1/1/0), with the locators of [The regulation settings](#the-regulation-settings).
8. **WNG's part ranges bound the sliders**: knuckle 15–19 mm in 0.5 mm steps, Ø 8,
   9, 10 or 11 mm; heel position 50.0–69.5 mm in 1.5 mm steps, heel height 16–24 mm
   in 1 mm steps, the sharps' heels +2 mm; spoon 73.6, 76.6 or 81.6 mm [V, WNG,
   pp. 3–6, 9]. They are values a supplier makes, so they bound an admissible range
   ([FOUNDATIONS > The parameter registry and the validation layer](../FOUNDATIONS.md#the-parameter-registry-and-the-validation-layer)); they do not validate a model, which takes independent
   measurements ([FOUNDATIONS > What a test proves; drive in position, compare forces](../FOUNDATIONS.md#what-a-test-proves-drive-in-position-compare-forces)).
9. **Bench artifacts as environment parameters**: the keyboard on the lap, whose
   front rail bows upward and reduces the key dip [V, Igrec, p. 173]; finger
   pressure when reading the key dip [V, Steinway ch. 2, p. 9]; humidity, through
   the bushings' tightness [V, Kawai, p. 6]. Key frame bedding itself is out of
   scope: a deflecting frame couples the notes, and the bench states a perfectly
   bedded frame ([FOUNDATIONS > What the model is](../FOUNDATIONS.md#what-the-model-is)).
10. **Friction is stored as a torque with the radius it was measured at**, never as
    grams alone, because the published values are taken at different radii:
    Steinway about 32 mm, WNG 20 mm, Igrec's Table 5 25 mm for most parts [V, Igrec,
    p. 143] ([FOUNDATIONS > Set-valued dry friction, at the axes and at sliding contacts](../FOUNDATIONS.md#set-valued-dry-friction-at-the-axes-and-at-sliding-contacts)).

---

## Sources

The corpus is in `data/references/`, outside version control. Short names as used
in the locators; page numbers are each document's own.

- **Steinway ch. 1, ch. 2, ch. 3**: Steinway & Sons, *World-Wide Technical
  Reference Guide*: chapter 1 (foreword, "Grand Preparation" and the Hamburg
  preparation checklist), chapter 2 ("Grand Regulation"), chapter 3 ("Grand
  Dampers and Sostenuto").
- **Kawai**: Kawai, *Grand Piano Regulation Manual*, version 1.4 (2011), Piano
  Laboratory.
- **Spec sheet**: *Grand Piano Regulation Specifications*, a four-page compilation of
  generally acceptable ranges and of Kawai, Steinway and Yamaha specifications.
- **Igrec**: Mario Igrec, *Pianos Inside Out* (2013), chapter 5, "Regulating",
  sample pages 135, 142–143, 157–158, 173–174, 184, 192 and 195.
- **Spurlock**: Bill Spurlock, *Grand Action Touchweight* (2006), pp. 1–9.
  **Spurlock FR**: its French translation by Marc Valdeyron (2013), *Le poids au
  toucher de la mécanique du piano à queue*.
- **Reyburn**: Reyburn, *Action Geometry Measurement Directions*, pp. 1–8.
- **WNG**: Wessell, Nickel & Gross, *Instructions for the Wessell, Nickel & Gross
  Piano Action Order Sheet*, pp. 1–11.
- **Redekop**: Fred Redekop, *Grand Piano Dampers: An Everyday Guide* (2012),
  pp. 1–5.
- **Potter checklist**: the grand piano regulation checklist of *The Piano Action
  Handbook* (Randy Potter School), French translation by Marc Valdeyron (1999),
  *Réglage du piano à queue*, pp. 1–2.
- **Oleg glossary**: Isaac Oleg, a French glossary of piano terms (2005), pp. 1–3,
  cited for its figures only.
- **Stanwood 1996**: David C. Stanwood, "The New Touchweight Metrology", *Piano
  Technicians Journal*, June 1996, reformatted print, pp. 1–7.
- **Stanwood 2000**: "Standard Protocols of the New Touchweight Metrology", *Piano
  Technicians Journal*, February 2000, pp. 20–23, the glossary pp. 22–23.
- **Stanwood 2000-03**: "Looking at Grand Pianos Through the Eyes of the New
  Touchweight Metrology", *Piano Technicians Journal*, March 2000, pp. 21–27.
- **Stanwood 2014**: "Cutting the Balance Rail Punching to Effect Ratio", *Piano
  Technicians Journal*, December 2014, pp. 8–9.
- **Stanwood, pianotech list**: "Strike Weight/Ratio", a one-page post to the
  pianotech mailing list, April 1995.
- **Thorin 2017**: Thorin, Boutillon, Lozada and Merlhiot, "Non-smooth dynamics for
  an efficient simulation of the grand piano action", *Meccanica* 52, 2017.
