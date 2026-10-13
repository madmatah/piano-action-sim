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
front, capsule, back check block, thinned tail and chamfer, a turned capstan,
leads) stands on a parametric frame (balance, front and back rails, their pins,
and punchings laid from a drawer of real papers). The key turns on its balance
pin against contacts read off the drawn parts, with set-valued friction at the
balance: a static solve puts it where its weight and the weight on it leave it,
a weight between the up weight and the down weight moves nothing, and the bench
says that friction holds it. Key height and key dip are read with the key seated
by the trade's gestures, its back held down at the capstan and its front pressed
down, and solved from targets to punching stacks by that seated pose, laid in
papers within one paper of the targets, a target a stop forbids refused with its
shortfall in millimetres; front weight and key weight ratio are read by running
Stanwood's jig on the model, the key a body on a pin held level and the scale
reading the force that holds it; down weight and up weight are walked a gram at
a time from the key resting at the top and at the bottom of its stroke until it
goes all the way, and balance and friction weight are read from those two; the
key friction weight is read by Stanwood's protocol, zeroed on the levelled jig,
and by its variant zeroed where the key rests on its frame. A key friction
weight is typed as Stanwood's protocol reads it, and the bushings are set to the
torque at which that protocol, run on the model, reads it back; one no bushings
read is refused with why. Weights are read as gram weights give them, down
weight and key friction weight rounded up to the gram, up weight down, a
threshold the walk cannot tell from a whole gram read as that gram, with each
value in full beside it, its trust, and the counts of the solves it took. Every
touchweight is read at one measuring point, a setting of the bench, 13 mm back
from the covering's lip by default. A frame edited out of fit with its key is
drawn as it is, its faults named in millimetres. The key's contacts with its
frame are read off the drawn parts, as signed gaps with their gradients over
every coordinate, and marked on the section and the solid.

Every reading on the weighing sheet is a procedure run on the model; their
closed forms are the tests' oracles. What the stage simplifies is listed in
[`docs/FOUNDATIONS.md`](docs/FOUNDATIONS.md) §2.3, first among them a mass
model that cannot predict front weight, which makes every reading on the
default bench an indication rather than a measurement.

| Part | What it is |
|---|---|
| `crates/pa-core` | the domain in Rust: geometry, bodies in a tree, contacts, the static solve, parts, the weighing protocols |
| `crates/pa-app` | the single-key bench, as a use case, and the golden sheet that records its every reading |
| `crates/pa-cli` | `weigh`: the weighing sheet in a terminal, each reading with its procedure, trust and solves; `golden`: rewrites the golden sheet |
| `crates/pa-wasm`, `apps/bench` | the same bench in a browser, in section and in solid (React, react-three-fiber), from the same Rust compiled to WebAssembly |

## What is planned

In this order, each stage with its engine and its view
([`docs/FOUNDATIONS.md`](docs/FOUNDATIONS.md) §8):

- a general quasi-static core with the key as its first client, on
  Hirschkorn's published Boston GP-178 key, a parameter registry that loads
  `data/params/`, a validation layer, and an interface in English and French
  (its first parts, the mechanism, the contacts over all coordinates, the
  static solve and every reading of the sheet by procedure, are built);
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
cargo run -p pa-cli -- weigh               # the weighing sheet, no browser
PATH="$HOME/.cargo/bin:$PATH" pnpm build   # the wasm through wasm-pack, then the bench
pnpm --filter @pas/app-bench dev           # the bench; add ?view=section for the section
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
- [`data/params/`](data/params/): parameter records, each with its source; the
  engine does not read them yet, and the README there lists the starting values
  that have no record.
- [`tools/derivations/`](tools/derivations/): the script behind every `[E]`
  number in FOUNDATIONS.
- [`CLAUDE.md`](CLAUDE.md): the working rules.

The licence is not chosen, and nothing is published until it is.
