# Versioned parameter data

One record per value, each with its provenance: where the number came from, its
unit, and how far it can be trusted. The rule this directory serves: **no
number without provenance.** A number transcribed into prose, citing a
derivation that lives in a temporary directory, reads as sourced and is
reproducible nowhere else. A derivation therefore lives in `tools/derivations/`,
under version control, and runs with `python3 -I`.

## What the engine does with these records

**Built:** the engine starts from no number of its own. An instrument is
built from a profile (`data/profiles/`), which binds every number it starts
from either to a record here, by its file and a JSON pointer, or to an
assumption it states with its reason; `pa-app` compiles the profiles and the
records they read into the engine, checks each record's unit against the
number's quantity and converts it to SI once. A value the engine starts from
that has no record here is stated in the profile that uses it, marked `S` or
`E` with its rule, and says why: the owner's measured section's key front,
which no measurement places along the key yet, its rail heights, felts and
cloth, its key's width, its capstan's proportions, its targets, the bushings
it starts on, and the rest, each in `data/profiles/measured-section.json`;
what Hirschkorn's thesis does not give, his keybed, rails, papers and pins
borrowed from the measured section and his felts' rails placed by a rule from
his rounds, in `data/profiles/hirschkorn.json`. No mass is a record or an
assumption here: a key's mass is published, as Hirschkorn's is, or weighed on
the bench, and the measured section's key arrives unweighed. The test
`every_default_has_a_record_or_a_stated_assumption` fails on a number of a
shipped profile that is neither, on one read from a record in the wrong
unit, and on one stated without its reason.

## Two instruments, two names

**Hirschkorn's Boston GP-178, note 52** (Hirschkorn 2004, Appendix A) is the
engine's base action: the only published set with all thirteen contact load
laws measured on one action, and the bench's standard instrument
(`data/profiles/hirschkorn.json`).
Its record, `hirschkorn-boston-gp178-note52.json`, states each body's frame
convention at the top: every frame's origin is the body's own pivot, and `Pw2`,
`Pw3` are the jack and repetition lever pins on the wippen (his Figure A.1). With no compression bench, every
contact law the engine is to use is borrowed from this action, and anything
computed from a borrowed law is to be marked as such (planned,
[FOUNDATIONS > The data model](../../docs/FOUNDATIONS.md#the-data-model)).

Read bounds at the note the data describes. Hirschkorn's action is note 52, so
Stanwood's figures for note 1 do not apply to it.

**The owner's measured keys and key frame** are the source of
`key-section.json` and `key-frame-sections.json`, which the owner's measured
section (`data/profiles/measured-section.json`) is built from. They are a
generic base, not any maker's specification. What is still to measure is
listed in each file (`still_to_measure`); the measured section's profile says
beside each assumption what would replace it: first, one of these keys
weighed whole (its mass, its front weight by Stanwood's protocol, and how far
back from the covering's lip the weight was centered), and the distance from
the center of its balance hole to its key front.

## Record shape

```json
{
  "value": 17.0,
  "unit": "mm",
  "source": { "doc": "reyburn_geom.pdf", "table": "Action Geometry Measurement Directions", "page": 1 },
  "reliability": "V"
}
```

`reliability` is one of:

- `V`: verified against a primary source, with its locator;
- `E`: derived by calculation, the calculation committed under
  `tools/derivations/` or written inline as a rule;
- `S`: assumed, pending measurement.

Five conventions come up as soon as a real instrument is measured:

- **A measurement taken by the owner is `V`.** A calliper on an identified
  action is a primary source, and a stronger one than a transcribed table. It
  carries the instrument and the date instead of a document and a page, because
  those are what another technician would need to repeat it; an instrument that
  is not named publicly is described instead.
- **Where every record in a file shares one source, the source sits at the top
  of the file** rather than on each record. A record from elsewhere carries its
  own and overrides it; where the records share a document but not a page, each
  carries its own `locator`.
- **A measurement that contradicts another measurement is not entered.** It is
  recorded under `does_not_reconcile` with the arithmetic that refuses it, and
  the value it would have replaced stays until the conflict is settled.
- **A value derived by a rule short enough to write out states the rule
  inline**, as `derived_from` and `rule`. A script is for a derivation a reader
  could not check in their head.
- **A source names the instrument it was measured or read on**, as
  `instrument`, in the words every file of that instrument uses: a profile
  names the instruments it describes the same way, and a number it reads
  from a file of another instrument is borrowed (`data/profiles/README.md`).
  Stock and protocols, of no instrument, name none.

## Contents

| File | What it holds |
|---|---|
| `stanwood-weight-curves.json` | Stanwood's strike-weight and hammer-weight curve standards, all 88 notes × 13 levels, transcribed from the published table and cross-checked |
| `key-frame-sections.json` | the three rails as parts: pin shapes and sizes, punching and hole diameters, and the offset between the naturals' and the sharps' rows, measured on one key frame |
| `punching-papers.json` | the punching papers a regulation is shimmed with: eight thicknesses, and the color each is supplied in |
| `key-section.json` | the keystick in profile: the wood's thickness, the covering and key front, the capsule, the block that carries the back check, the thinned tail and its felt, the chamfer under it, the front mortise, and where the capstan stands behind the balance pin; still to measure, the clearance cut in the top for the wippen rail |
| `balance-mortise.json` | how the key button and the balance mortise are built, as the owner states them with his sketches; still to measure, every dimension of them on a real key |
| `touchweight-protocol.json` | the conventions Stanwood's touchweight protocols run under: the measuring point 13 mm or 1/2 inch back from the front lip of the key, and the 10 g the key weight ratio is read with |
| `hirschkorn-boston-gp178-note52.json` | all of Hirschkorn's Appendix A, the engine's base action: masses, centers of mass and inertias (A.1), the points and surfaces of the ground and the five bodies, each in its own body's frame with the frame convention stated at the top (A.2–A.7), the thirteen contact laws with their friction (A.8), the axis frictions (A.9), the springs, and his regulation; 235 values, each with its table, row and page, and what the thesis does not give. `tools/derivations/10` reads it |
| `hirschkorn-key-outline.json` | Hirschkorn's key where his tables give no number for its shape, traced from his Figure A.1: the positions of the figure's markers and of the key's outline in pixels of the page rendered at 600 dpi, with how they were read, and the numbers his profile binds from them or derives from them by a rule it states, each `E` by `tools/derivations/11`, which reads the pixels and his record, fits the trace on three printed points and prints the residuals at the others as its uncertainty |

## Planned records

- Thorin 2013, chapter 4.5, and Lozada 2007, transcribed with a lineage tag on
  every value: measured, assumed equal to another joint, or placeholder. The
  traps are in [FOUNDATIONS > Parameter lineage](../../docs/FOUNDATIONS.md#parameter-lineage).
- Manufacturer regulation specifications, as presets whose cells carry a value,
  a range and a source, with both values kept where two documents of one maker
  disagree.
