# piano-action-sim

An engine for building applications around the **grand piano action**,
educational and playful alike: the trade's physics, a configurator, a saved
state, diagnostics in the trade's terms and a 3D rendering, running in a
browser, built for a piano technician. Its first application is a single-key
regulation bench. Its acceptance test is the trade's own:
the simulator must measure its own regulation with the trade's protocols (key
height, key dip, blow distance, let-off, drop, aftertouch, checking, damper lift
point, down weight, up weight, friction, action ratio) and come back with the
expected values.

## What is built

**Stage 1a, the key on its frame.** A parametric key (keystick, covering, key
front, capsule, back check block, thinned tail and chamfer, a clearance cut in
its top for the wippen rail where it has one, a turned capstan)
stands on a parametric frame (balance, front and back rails, their pins, and
punchings laid from a drawer of real papers). A key is weighed whole, its leads
in it: its mass and its front weight are typed, and the bench keeps the static
moment the front weight gives on the arm it was read at; the height of its
center of mass, which no scale gives, is estimated from its drawn section. An
outline never weighs a key: a key nobody has weighed has no mass, and every
reading that needs one is refused, saying so, while key height, dip, the key
weight ratio and the key friction weight zeroed on the frame are read. The key turns on its
balance pin against contacts read off the drawn parts, with set-valued friction
at the balance: a static solve puts it where its weight and the weight on it
leave it, a weight between the up weight and the down weight moves nothing, and
the bench says that friction holds it. Key height and key dip are read with the
key seated by the trade's gestures, its back held down at the capstan and its
front pressed down, and solved from targets to punching stacks by that seated
pose, laid in papers within one paper of the targets, a target a stop forbids
refused with its shortfall in millimeters; front weight and key weight ratio are
read by running Stanwood's jig on the model, the key a body on a pin held level
and the scale reading the force that holds it; down weight and up weight are
walked a gram at a time from the key resting at the top and at the bottom of its
stroke until it goes all the way, and balance and friction weight are read from
those two; the band is drawn by driving the key through its stroke at the
measuring point, down and back, the force that holds it there read at every
step, and D and U read on it a second way, its extremes, within a few
thousandths of a gram of the walk's and shown in full, the gram being the
walk's; the key friction weight is read by
Stanwood's protocol, zeroed on the leveled jig, and by its variant zeroed
where the key rests on its frame. A key
friction weight is typed as Stanwood's protocol reads it, and the bushings are
set to the torque at which that protocol, run on the model, reads it back; one
no bushings read is refused with why. Weights are read as gram weights give
them, down weight and key friction weight rounded up to the gram, up weight
down, a threshold the walk cannot tell from a whole gram read as that gram, with
each value in full beside it, its trust, and the counts of the solves it took.
Every touchweight is read at one measuring point, a setting of the bench, 13 mm
back from the covering's lip by default. A frame edited out of fit with its key
is drawn as it is, its faults named in millimeters. The key's contacts with its
frame are read off the drawn parts, as signed gaps with their gradients over
every coordinate, and marked on the section and the solid. Each crosses to
the page with its normal and the force it carries, which the view does not
draw yet.

The bench is an instrument an application drives by two calls. Every setting
has a path with its trade unit in its name (`frame.front_rail.pin_height_mm`)
and is set by it; every operation (laying or lifting a paper, regulating,
weighing the key, typing its key friction weight, placing the frame) is an
action. A registry describes every path, what it admits and the page that
presents it, and the browser page is drawn from it: every control takes its
range, step and unit from the engine, and its label from the page's
catalog. An edit that describes a key no part could be built as is refused
before any solve, with the limiting dimension and the shortfall, and the bench
is left as it was: a key front brought back until its covering runs into the
key button, a capstan in a glued block or over the thinned tail, a front
mortise cut through the key top, a measuring point off the covering, a mass
of nothing, a value that is not a number. Everything an assembled keyboard can be, however badly
regulated, is computed and reported. Every edit taken is logged, and the log
replayed gives the same state, which a hash names. The bench starts from no
number of its own: an instrument is built from a profile (`data/profiles/`),
which binds every number it starts from to a record in `data/params/` or to an
assumption stated with its reason, and a test fails on one that is neither.
The bench opens on Hirschkorn's published Boston GP-178 key, note 52, on his
printed frame with real felts, its outline traced from his drawing where his
tables give no number, its mass, center of mass and bushings his; resetting
returns an instrument to its own profile. The page chooses between the two
profiles shipped: his key, and the owner's measured key section on the key
frame he measured, its key front assumed and its key unweighed until one is
weighed on the bench. Every line of
the section, and the outlines of the extruded parts in the solid, can be
marked by how the number that draws it is known: measured, published, traced,
derived by a rule, borrowed from another instrument, assumed, or set on the
bench.

Every reading on the weighing sheet is a procedure run on the model; their
closed forms are the tests' oracles. What the stage simplifies is listed in
[`docs/FOUNDATIONS.md` > Named simplifications](docs/FOUNDATIONS.md#named-simplifications); on Hirschkorn's key every
reading is an indication of its geometry, his keybed, rails, felts and papers
being borrowed or assumed.

| Part | What it is |
|---|---|
| `crates/pa-core` | the domain in Rust: geometry, bodies in a tree, contacts, the static solve, parts, the weighing protocols |
| `crates/pa-app` | the single-key bench as an instrument, built from a profile, set by path and by action, its registry and its documents; the golden sheet that records its every reading |
| `data/profiles` | what each instrument starts from, every number bound to a record or a stated assumption |
| `crates/pa-cli` | `weigh [PROFILE]`: the weighing sheet in a terminal, each reading with its procedure, trust and solves; `band [PROFILE]`: the band, its extremes in full beside the walk's D and U, and both curves step by step; `golden`: rewrites the golden sheet |
| `crates/pa-wasm`, `apps/bench` | the same bench in a browser, in section, in solid and as the band, force against travel, drawn when it is chosen (React, react-three-fiber), from the same Rust compiled to WebAssembly |

## What is planned

In this order, each stage with its engine and its view
([`docs/FOUNDATIONS.md` > Stages](docs/FOUNDATIONS.md#stages)):

- a general quasi-static core with the key as its first client and an
  interface in English and French (its first parts, the mechanism, the
  contacts over all coordinates, the static solve, every reading of the sheet
  by procedure, a boundary that sends codes and numbers which the page words
  in English, the registry and the validation layer, the profiles that load
  `data/params/`, Hirschkorn's key the default and the owner's measured
  section beside it, the band drawn by the driven sweep, are built; the French
  interface is planned);
- the felt law and the flexible hammer, which the one-note dynamics prototype
  (`tools/dynamics-prototype/`, built) left open after settling the time step
  and the solver;
- the wippen, then the hammer, until Stanwood's equation of balance closes
  (stages 1b to 1d);
- the jack, the repetition lever, let-off, drop and checking: the action in
  motion (stage 2);
- regulation training (stage 3);
- 88 notes with dampers and three pedals, playing a score (stage 4).

## Running it

Rust 1.99.0 with the `wasm32-unknown-unknown` target (pinned in
`rust-toolchain.toml`), Node 26 (`.nvmrc`), pnpm, and wasm-pack. The browser
smoke test needs a Chrome or Chromium binary; without one, `pnpm verify` skips
it and says so.

```bash
pnpm install                               # first run
pnpm verify                                # CI's gates, its install aside: Rust
                                           # format, clippy, tests, wasm build;
                                           # Biome, type-check, build, tests,
                                           # browser smoke test
cargo run -p pa-cli -- weigh               # Hirschkorn's weighing sheet, no browser
cargo run -p pa-cli -- weigh measured-section  # the owner's section, unweighed
cargo run -p pa-cli -- band                # the band, step by step
PATH="$HOME/.cargo/bin:$PATH" pnpm build   # the wasm through wasm-pack, then the bench
pnpm --filter @pas/app-bench dev           # the bench; ?view=section or ?view=band
```

`wasm-pack` installs into `~/.cargo/bin`; where a shell does not put that on
`PATH`, the prefix above is needed (`pnpm verify` adds it itself).

## Where to read what

- [`docs/FOUNDATIONS.md`](docs/FOUNDATIONS.md): what the simulator is for, what
  is built and planned, the decisions with their reasons, and the hypotheses
  with the tests that will settle them.
- [`docs/research/`](docs/research/): the research annexes (physics engines,
  the literature, nomenclature and regulation, the web toolchain, the numerical
  formulation); where one disagrees with FOUNDATIONS, FOUNDATIONS holds.
- [`data/params/`](data/params/): parameter records, each with its source.
- [`data/profiles/`](data/profiles/): what each instrument starts from, every
  number bound to a record or to an assumption stated with its reason.
- [`tools/derivations/`](tools/derivations/): the script behind every `[E]`
  number in FOUNDATIONS.
- [`CREDITS.md`](CREDITS.md): the libraries and the published data the
  project reuses, with their terms.
- [`CLAUDE.md`](CLAUDE.md) and [`.claude/rules/`](.claude/rules/): how an
  agent works here, and the rules the repository keeps, each with its check.

piano-action-sim is released under the MIT License ([`LICENSE`](LICENSE)). The
published works it draws on keep their own terms.
