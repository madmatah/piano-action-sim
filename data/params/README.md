# Versioned parameter data

One record per value, each with its provenance: where the number came from, its
unit, and how far it can be trusted. The rule this directory serves: **no
number without provenance.** A number transcribed into prose, citing a
derivation that lives in a temporary directory, reads as sourced and is
reproducible nowhere else. A derivation therefore lives in `tools/derivations/`,
under version control, and runs with `python3 -I`.

## What the engine does with these records

**Built:** nothing reads them. The engine's defaults are Rust literals that
repeat the records (`KeyDesign::default`, `FrameDesign::default`,
`PAPER_THICKNESSES`, `MEASURING_POINT_FROM_LIP`, `MEASURING_POINT_HALF_INCH`,
`KR_PROTOCOL_LOAD_G`), their doc comments pointing here, and nothing checks
that the two agree.

**Planned:** the engine loads its defaults from these records through the
parameter registry (`docs/FOUNDATIONS.md` §7.3), and a test fails when a default
has no record or disagrees with it.

### Values the engine starts from that have no record

| Value | Where it is used | What is known |
|---|---|---|
| wood density 430 kg/m³ | the key's mass | assumed |
| keystick width 22 mm | the key's mass | assumed: integrated as a uniform prism over the whole key, although behind the head a grand keystick is much narrower |
| capstan 129.5 mm behind the balance pin | key weight ratio | no source gives it: Thorin gives 128.7 mm (his §4.5.2) and 129 mm (his Table 2.1). The owner's measured 122 mm is recorded in `key-section.json` and not used |
| key front 258 mm in front of the balance pin | key height, key dip, and every touchweight through the measuring point, 13 mm behind the covering's lip 2 mm further forward, so at 247 mm | 13 mm ahead of Thorin's key arm, L = 0.245 m (his Table 2.1); no record |
| rail heights 26.6, 25.9 and 19.0 mm (balance, front, back) | key height, key dip | assumed |
| key friction weight 6 g | the friction band | assumed, of the order the trade measures |
| felt washers 1.5 and 2.0 mm (balance, front), back rail cloth 3.0 mm | key height, key dip | assumed |
| front mortise depth 22 mm | the mortise stop | 3 mm of recess and 19 mm of mortise, written only in a note of `key-frame-sections.json` |
| balance crown height 6 mm, balance pin 11.2 mm proud | the drawing | assumed |
| back rail 24 mm along the key | where the tail bears on its cloth | assumed, to be measured |
| the capstan's proportions, in steel at 7850 kg/m³ | the key's mass | proportions scaled from a photograph, material assumed |
| a lead's centre 10 mm above the key's underside | the key's mass properties at an angle | assumed |
| regulation targets: key height 65.0 mm, key dip 10.2 mm | the default bench | close to, not equal to, the published Steinway New York figures |

## Two instruments, two names

**Hirschkorn's Boston GP-178, note 52** (Hirschkorn 2004, Appendix A) is the
published action the dynamics are built on first: the only published set with
all thirteen contact load laws measured on one action. Its records are planned
(below), each body's frame convention transcribed with them: every frame's
origin is the body's own pivot, and `Pw2`, `Pw3` are the jack and repetition
lever pins on the wippen (his Figure A.1). With no compression bench, every
contact law the engine is to use is borrowed from this action, and anything
computed from a borrowed law is to be marked as such (planned,
`docs/FOUNDATIONS.md` §7.5).

Read bounds at the note the data describes. Hirschkorn's action is note 52, so
Stanwood's figures for note 1 do not apply to it.

**The owner's measured keys and key frame** are the source of
`key-section.json` and `key-frame-sections.json`. They are a generic base, not
any maker's specification.

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

Four conventions come up as soon as a real instrument is measured:

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

## Contents

| File | What it holds |
|---|---|
| `stanwood-weight-curves.json` | Stanwood's strike-weight and hammer-weight curve standards, all 88 notes × 13 levels, transcribed from the published table and cross-checked |
| `key-frame-sections.json` | the three rails as parts: pin shapes and sizes, punching and hole diameters, and the offset between the naturals' and the sharps' rows, measured on one key frame |
| `punching-papers.json` | the punching papers a regulation is shimmed with: eight thicknesses, and the colour each is supplied in |
| `key-section.json` | the keystick in profile: the wood's thickness, the covering and key front, the capsule, the block that carries the back check, the thinned tail and its felt, and the chamfer under it |
| `touchweight-protocol.json` | the conventions Stanwood's touchweight protocols run under: the measuring point 13 mm or 1/2 inch back from the front lip of the key, and the 10 g the key weight ratio is read with |

## Planned records

- Hirschkorn 2004, Appendix A: inertias (A.1), geometry (A.2–A.7) with the
  frame convention above, the thirteen contact laws (A.8), axis friction (A.9).
- Thorin 2013, chapter 4.5, and Lozada 2007, transcribed with a lineage tag on
  every value: measured, assumed equal to another joint, or placeholder. The
  traps are in `docs/FOUNDATIONS.md` §6.3.
- Manufacturer regulation specifications, as presets whose cells carry a value,
  a range and a source, with both values kept where two documents of one maker
  disagree.
