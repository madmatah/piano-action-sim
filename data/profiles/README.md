# Profiles

**Built.** A profile is what an instrument starts from: every number of its
key, the key's capstan and mass, its frame with the rails' sections and the
drawer of papers, the protocol it is weighed by, its bushings, the
regulation's targets and the weight on the key. The engine starts from no
number of its own, so each is bound here either to a **record** in
`data/params/` or to an **assumption** stated in the profile with its reason. `pa-app` compiles the
profiles and the records they read into the engine (`pa_app::profile`), and
the test `every_default_has_a_record_or_a_stated_assumption` fails on any
number of a shipped profile left unbound, read from a record in the wrong
unit, or stated without its reason.

| File | What it describes |
|---|---|
| `hirschkorn.json` | The bench's standard instrument: Hirschkorn's Boston GP-178 key, note 52, on his printed frame with real felts, as it arrives ([FOUNDATIONS > Validation instruments](../../docs/FOUNDATIONS.md#validation-instruments), "Hirschkorn's Boston GP-178, note 52"): his printed points, lines and circles from his record, his key's outline traced from his Figure A.1 where his tables give none, his mass published whole, his bushings' torque, his dip target; his felts' tops where his rounds' tops stand, the back rail cloth lying wholly under his chamfer with its front edge over his back bearing, each placed by a rule from his frame; his keybed, rails, papers and pins borrowed from the measured section and stated so. |
| `measured-section.json` | The owner's measured key section on the key frame he measured, its rails placed under the key, regulated on arrival: his keystick, covering, blocks, tail, chamfer, front mortise and capstan's place as he measured them; its key front, rail heights, felts, cloth, capstan's proportions, bushings and targets assumed, each with what would replace it. Not weighed: until one of its keys is weighed on the bench, every reading that needs its mass is refused. |

Both are chosen on the bench, by an action that puts the profile's instrument
there as it arrives; the bench opens on Hirschkorn's.

## Shape

```json
{
  "profile": "measured-section",
  "what": "what it describes",
  "base": "another profile, whose bindings this one overrides",
  "instruments": ["a grand piano key frame", "…"],
  "key_kind": { "is": "natural", "why": "…" },
  "key_mass": { "is": "unweighed", "why": "…" },
  "arrival": { "is": "regulated", "why": "…" },
  "values": [ … ]
}
```

`base` is optional: a profile with one binds only what differs from it, each
of its bindings and choices replacing its base's. `instruments` names the
instruments the profile describes, each as its records' sources name it
(`source.instrument` in `data/params/`); a profile on a base may leave it to
its base's, and one naming an instrument no record names does not load. A
number read from a record of any other instrument is **borrowed**: the
overlay marks it so, whatever its record makes it on its own instrument,
except an assumption, which stays one wherever it was made. A record of no
instrument, as Stanwood's protocol and the drawer's stock of papers are, is
borrowed by none. So Hirschkorn's profile, which describes his Boston
GP-178, borrows the pins and punchings the owner measured on his key frame,
which the measured section, describing the owner's keys and key frame, reads
as measured. `key_kind` is `natural` or
`sharp`; `key_mass` is `unweighed` (no mass: every reading that needs one is
refused until the key is weighed on the bench), `weighed` (bound by
`weighing.mass_g`,
`weighing.front_weight_g` and `weighing.from_lip`) or `published` (a source's
mass and center of mass in the key's frame, its height included, and its
inertia: `published.mass`, `published.center.x`, `published.center.y`,
`published.inertia`); `arrival` is `regulated` (the papers are those the
regulation solve lays to the targets) or `as_recorded` (bound by
`balance_punching.papers` and `front_punching.papers`, each paper one the
drawer holds).

Each entry of `values` binds one parameter, by the name the engine's
validation layer gives it (`pa_core::validate::numbers`), in one of three ways:

```json
{ "param": "stick_height", "record": "key-section.json", "pointer": "/records/stick_thickness" }
{ "param": "front_edge", "value": 258.0, "unit": "mm", "reliability": "S", "why": "…" }
{ "param": "top_clearance", "none": true, "why": "…" }
```

- **A record**: its file under `data/params/` and a JSON pointer to a record
  holding `value`, `unit` and `reliability`. An array's element may be named
  by its `name` member instead of its index, so `/records/stick_thickness`
  stays right when the records are reordered. `"derive": "half"` takes half
  the recorded value, exactly: a radius from a diameter, a half-length from a
  length; `"derive": "behind"` takes a distance recorded behind the balance
  pin as the position the key's frame signs, negative behind it. A pointer to an array binds a list, as the drawer's papers are bound.
- **A stated value**: its value (or `values`, for a list) and unit, its
  marker, `S` for assumed pending measurement or `E` with the `rule` that
  derives it written out, and `why`. A verified value is a record, never a
  statement.
- **A stated absence**, for what a part may lack: a rail's crown or pin, a
  felt on the key's tail, a clearance in the key's top, a dome on the
  capstan's head; or for where the key button stands,
  centered on the pin where `capsule_center` is none. A rail's pin is round where
  `<rail>.pin.diameter` is bound, oblong where `<rail>.pin.length` and
  `<rail>.pin.thickness` are, and absent where `<rail>.pin` is bound to none;
  the clearance cut in the key's top for the wippen rail, between the
  capstan and the back check block, is `top_clearance.back` and
  `top_clearance.front`, its two ends signed from the balance pin, and
  `top_clearance.depth`, below the full top, or `top_clearance` bound to
  none. Neither shipped profile has one: the measured section's waits on its
  measurement (`data/params/key-section.json`), and Hirschkorn's figure
  draws his key's top lowered from just behind the capstan to its end
  instead, bound as his thinned tail.

A part a source gives as a printed line or circle is bound as printed, and the
numbers the key is described by are computed from it as the instrument is
built, each kept with the rule and the input that places it:

- **A chamfer as a line**: `chamfer.x`, `chamfer.y`, a point of it, and
  `chamfer.normal.x`, `chamfer.normal.y`, its normal, in place of `tail_rise`
  and `tail_rise_run`, which follow from where the line meets the flat and
  stands at the key's end.
- **A capstan by its dome**: `capstan.dome.x`, `capstan.dome.y`, the dome's
  center, with `capstan.dome_radius`, in place of `capstan.x` and
  `capstan.height`, the axis through the center and the top the radius above
  it, above the capstan's seat.
- **A key height as the key stands**: `key_height` bound to none, where no
  source gives one, on a profile that arrives as recorded. The target is the
  key height the seated pose reads on the key as it arrives, so regulating it
  lays the dip and leaves the key height where it stands.

Positions are signed from the balance pin, positive toward the player.
Units are checked against the parameter's quantity: a length in `mm` or `m`, a
mass in `g` or `kg`, a parameter whose name ends in `_g` in grams, a torque in
`N·m`, a second moment of mass in `kg·m²`, a unit
vector's component `dimensionless`. Each is converted to SI by moving the decimal point
of the value as written and reading it once, so a value recorded in
millimeters reads to the bits of the same length written in meters; a value
with more than fifteen significant digits is refused.

The bushings are bound either by `friction.level_weight_g`, the grams they
hold at the measuring point with the key level, which gives the torque on
whatever key the instrument is built with, or by `friction.bound`, a torque,
with `friction.radius` it was read at or none.
