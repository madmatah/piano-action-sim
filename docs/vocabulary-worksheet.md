# Vocabulary

The project's French terms are the standard usage of the trade, taken from a
numbered sectional plate of a grand action and the matching chapters on the
action, the dampers and the pedals, and confirmed by the owner. The questions
still open are at the end.

**No source is cited and none is owed.** These are the ordinary terms of the trade
in France, not any one author's coinage; a word in general use is not attributable.
Nothing was reproduced from the book — no figure, no passage, only the vocabulary,
which is the common property of the trade. The photographs stay in
`data/references/`, which is gitignored.

The full record, with the reasoning, is
`docs/research/A3-nomenclature-and-regulation.md` §7, and the FR/EN/DE mapping is
its §6. This file keeps only the summary and the open questions.

This file is **deliberately bilingual**: the French terms are the subject matter.

## Settled

| Part | French | Identifier |
|---|---|---|
| jack | **bâton d'échappement** | `jack` |
| wippen assembly | **chevalet** (de mécanique) | `wippen_assembly` |
| wippen, the body alone | **corps du chevalet** | `wippen` |
| bridge | **chevalet** (de table d'harmonie) | `bridge` |
| damper under-lever | **bascule d'étouffoir** | `damper_underlever` |
| wippen flange | **fourche de chevalet** | `wippen_flange` |
| hammer shank flange | **olive du manche de marteau** | `hammer_flange` |
| let-off button | **bouton d'échappement** *(syn. pilote d'échappement)* | `let_off_button` |
| damper lift rail | **barre de forte** | `damper_lift_rail` |
| action rails | **barres de mécanique** | `*_rail` |
| action brackets | **montants de mécanique** | `action_bracket` |
| key stop rail | **barre de contre-touche** | `key_stop_rail` |
| glide bolt | **vis de réglage de portée du châssis** | `balance_rail_glide` |
| keybed | **plateau** | `keybed` |
| key button | **capsule** | `key_button` |
| wippen heel | **talon** | `wippen_heel` |
| knuckle | **rouleau** | `knuckle` |
| capstan | **pilote** | `capstan` |
| back check | **attrape** | `back_check` |
| sostenuto | **la tonale** | `sostenuto` |
| trapwork lever | **mouvement de pédale** | `trapwork_lever` |
| damper block | **noix d'étouffoir** | `damper_block` |
| aftertouch | **gras** | `spec.aftertouch_mm` |

Three terms for which a plausible alternative circulates, and the alternative is
wrong: **« sommier » is the pinblock**, not an action rail — the rails are *barres*;
the action bracket is a **montant**, not an *équerre*; the key stop rail is the
**barre de contre-touche**.

**« Chevalet » names two different things**, both of them from the owner:
on the structure, the wooden bar linking the strings to the soundboard; in the
action, **the assembly of wooden parts** — heel, repetition lever, jack and the
rest. So the French UI must disambiguate, *and* the action sense is an assembly
rather than a single part: `wippen` is the body that carries a coordinate,
`wippen_assembly` is what a technician buys and replaces. That assembly is exactly
the coupled block of the mass matrix — see A3 §6.

## Still open

1. **« Cuillère de chevalet »** — the plate lists it among the *wippen* parts,
   not on the damper under-lever where the upright's spoon sits and where it is
   tempting to assume the grand's belongs. What is it, and what does it do?
2. **« Vis de chute » against « vis de réglage de la hauteur du levier de
   répétition »** — both are numbered separately on the plate and both govern
   drop. One adjustment under two names, or two parts?
3. **« Taquet d'étouffoir »** — on the Schwander action figure, in the damper
   group. Not identified.
4. **« Sommier »** — the pinblock on the plate, but the damper chapter writes
   *« le sommier des bascules »* for what the plate calls *châssis des étouffoirs*.
   Two senses, or loose usage?
5. **Una corda pedal** — no sectional plate settles it: *pédale de transposition*,
   *de décalage*, *petite pédale*, *pédale douce*?
