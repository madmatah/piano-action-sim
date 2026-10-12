# CLAUDE.md

Rules for this repository, each true of the tree or naming where it is not yet,
and each ending with its check or *not mechanically checked*. What the simulator
is for, what is built or planned, and why: `docs/FOUNDATIONS.md`. Where the work
stands: `private/HANDOFF.md`, read before touching anything. Where either
disagrees with the code, the code is right. One criterion governs everything
(FOUNDATIONS §1.2): **the simulator must measure its own regulation using the
trade's own protocols, and come back with the expected values.**

## Documents and commits

- **Present tense, no history**: justification stays, narration goes. Every
  component a published document describes is labelled **built** or
  **planned**; an unsettled decision is a **hypothesis** with its test. *Not
  mechanically checked.*
- **`private/` is one-way**: gitignored and its own repository, it may cite
  `docs/`; no published file cites it (this file, the agents' way in, is the
  exception), and a refuted claim gets an inline *do not use* warning instead.
  `git mv` into it leaves the file tracked: untrack it too. Check: `grep -rln
  'private/' README.md docs data tools crates apps scripts` and `git ls-files
  private` find nothing.
- **Nothing committed names the owner's instruments or other projects.** Check:
  `git diff --cached | grep -iFf private/forbidden-names.txt` finds nothing.
- **A commit message says why**: the reason, the constraint, the observation
  or the rejected alternative that the diff cannot show. The diff already says
  what changed; a message that restates it is noise. *Not mechanically checked.*
- **English** throughout, except the French interface catalogue (planned) and
  the bilingual glossaries. *Not mechanically checked.*
- **"Done" means a test or a grep in the tree shows it**, never a message.

## Code

1. **One part definition feeds contacts and drawing**: both are built in Rust
   from the same fields and exported, and the browser only renders them. They
   are two constructions held together by tests until contact surfaces become
   ranges into the drawn outline (planned, FOUNDATIONS §7.1). Check:
   `the_declared_bearing_surface_is_the_whole_underside`,
   `what_is_drawn_is_what_is_touched`.
2. **Regulation is trade targets solved to geometry; the coordinate is
   normative**, the target kept beside it with its achieved value. Check:
   `the_regulated_key_lands_within_the_placement_error_of_its_targets`.
3. **The simulator measures itself with the trade's protocols** run on the
   model, never a parameter read back; where a displayed value is still a
   formula or an input echoed back, FOUNDATIONS §2.2 says so. Check:
   `the_bench_measures_its_own_friction_by_the_protocol`.
4. **Axis friction is set-valued**, never `sign()` or `tanh()`, and stored as a
   torque with its measurement radius. Check: `grep -rn tanh crates/*/src` finds
   only comments; `KeyPinFriction` carries its arm.
5. **No number without provenance**: a record in `data/params/` for every value
   the engine starts from, a script in `tools/derivations/` for every derived
   one and every `[E]`; a source in `private/` counts, a lost one does not.
   *Not mechanically checked* (`data/params/README.md` lists unrecorded ones).
6. **No general-purpose physics engine in the core** (FOUNDATIONS §3.9); MuJoCo
   only ever as an optional oracle. Check: `cargo tree`, the `package.json`s.
7. **`pa-core` has no I/O and no web dependency, and justifies each
   dependency**: none today; `libm` with the first transcendental and `serde`
   behind a feature with persistence are the planned ones. `pa-wasm` and
   `pa-cli` adapt `pa-app` and add no physics. Check: the three manifests;
   `grep -rnE 'std::(fs|io|net)|wasm_bindgen' crates/pa-core/src` finds nothing.
8. **Determinism** (FOUNDATIONS §4.4): IEEE-754 only, a fixed order of
   operations and contacts, no `HashMap` iteration in the solver, every
   transcendental in `pa-core` (`pow` included) through the `libm` crate, no
   relaxed SIMD. Convergence-tested loops are allowed with a hard cap and a
   reported count; a metrological path converges, by tolerance under a cap or by
   a bisection whose fixed count resolves below what is read; fixed budgets
   elsewhere only where real time needs them. No time step, solver count or felt
   law is fixed by rule: each is a FOUNDATIONS §4 hypothesis, settled by its
   test. Check: `grep -rn HashMap crates/pa-core/src` finds nothing, nor does
   `grep -rnE '\.(sin|cos|tan|exp|ln|powf|powi|atan2?)\(' crates/pa-core/src`
   outside tests.
9. **SI unless the name says otherwise.** Geometry and mechanics are SI; a value
   in a trade unit (grams in the weighing algebra; millimetres and grams at the
   boundary, in the display and in saved files) carries it in its name (`kf_g`,
   `key_dip_mm`). An unsuffixed trade-unit field is a bug, as is a `*_torque`
   returning kg·m: `Touchweight` and `Branches` (grams) and `gravity_torque`,
   `net_torque` (kg·m) still have them. Check: review; `grep -rnE '_(mm|g)\b'
   crates/pa-core/src` lists the suffixed names.
10. **Upstream settings drift, they do not cascade; faults are reported, not
    refused.** Only what cannot be computed or drawn truthfully is refused or
    clamped, with its reason; the validation layer that catches all of it is
    planned (FOUNDATIONS §7.3). Check:
    `a_fault_shrinks_to_nothing_as_the_frame_comes_right`.
11. **Ratios are outputs**, changed only through geometry. The key ratio's force
    convention (≈ 0.50–0.60, the one in the equation of balance) and geometric
    one ("nominally 2:1") are reciprocals; swapping them inflates the wippen
    term about 3.3 times. Check: `grep -rnE 'set_[a-z_]*ratio' crates apps`
    finds nothing; `Touchweight::conventions_are_sane`,
    `the_inverted_key_ratio_is_rejected`.

## Validation

- **Drive in position, compare forces; keep the Bad-Model guard** (FOUNDATIONS
  §5.1), from the dynamics prototype on. *Not mechanically checked.*
- **Keep the non-confusion tests**: 1 g of lead moves BW, not F; axis friction
  ×1.2 moves F, not BW. Check: `one_gram_of_lead_moves_balance_weight_*`,
  `scaling_friction_moves_friction_weight_*`, `tools/smoke/`.
- **A validation test says so in its doc comment**; every other test is a
  consistency check, and a round trip through the model's own code never
  validates (FOUNDATIONS §5.1). *Not mechanically checked.*

## The owner, and staging

He is the authority on the trade: vocabulary, priorities and what a part looks
like are his call; ask when a term is in doubt. The French–English mapping is
`docs/research/A3-nomenclature-and-regulation.md` §6, its open questions in
`docs/vocabulary-worksheet.md`; do not restate it, and cite no source for a
trade term. Speak in trade quantities and his names; say what a modelling
choice changes for regulating, not which code moved; ask for his measurements
before drawing a part. Build by sub-assembly, engine and view together
(FOUNDATIONS §8): he validates by looking, so each stage ends on a number he can
judge by the trade's protocol. *Not mechanically checked.*

## Do not use

FOUNDATIONS warns inline against numbers and arguments that are wrong and easy
to derive again. Keep those warnings, add new ones where the error would be
re-derived, and keep this index current: 12 µs, 570:1, 2/ω as the stability
limit (§4.1); 280 Hz, 180 µs (§4.5); the time-step case against regularized
friction (§3.5); the coupling I_cm + m(r·u) (§3.6); "no game engine has joint
dry friction" (§3.9); the additive-damping equivalence, "no published
rate-independent felt model", "a factor of 56" (§4.3); "a convergence test
breaks reproducibility" (§4.4); annex A1's per-step costs (§4.6); "+0.1 mm of
capstan ⇒ −1.4 g", ΔJ/J as a ranking, gradient descent on settings (§5.3);
Thorin's demonstrator touchweights read as wear (§5.2); "only the support spring
moves BW" (§6.1); the spurious gap-gradient term (§6.2); Pw2 as the wippen
pivot, 5.84 (§6.4); "~40 % of friction unexplained", F ≈ 9 g (§6.5). Check: a
grep for a listed value finds only its warning; *not enforced*.

## Layout

```
crates/pa-core     domain: geometry · parts · measure. No I/O, no web, no deps.
crates/pa-app      application: the single-key bench (KeyBench)
crates/pa-wasm     browser adapter of pa-app, the only crate that knows JS;
                   its package.json lets Turborepo drive wasm-pack
crates/pa-cli      shell adapter of pa-app: `weigh`
apps/bench         the bench: React, react-three-fiber
data/params/       parameter records with provenance
data/references/   primary corpus: gitignored, third-party copyright
docs/              FOUNDATIONS.md · vocabulary-worksheet.md · research/ A1–A5
tools/derivations/ the script behind every [E] number (python3 -I)
tools/smoke/       drives the real page over the Chrome DevTools protocol
scripts/           verify.sh (every gate) · smoke.sh (headless Chrome)
private/           gitignored, its own repository: owner's material, tracking
```

## Commands

```bash
pnpm install                         # first run only
pnpm verify                          # every CI gate but the frozen install
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p pa-wasm --target wasm32-unknown-unknown
cargo run -p pa-cli -- weigh         # the weighing sheet, no browser
pnpm check                           # Biome (`pnpm exec biome`, never npx)
PATH="$HOME/.cargo/bin:$PATH" pnpm typecheck    # builds the wasm, then tsc
PATH="$HOME/.cargo/bin:$PATH" pnpm build && bash scripts/smoke.sh
PATH="$HOME/.cargo/bin:$PATH" pnpm test
pnpm --filter @pas/app-bench dev     # after a build; ?view=section
```

Node 26 and Rust 1.99.0 are pinned (`.nvmrc`, `rust-toolchain.toml`) so tools,
lints and formatting match CI; `nvm` is on the owner's machine. `wasm-pack` is
in `~/.cargo/bin`, off a non-interactive shell's `PATH`, hence the prefix on
every task that builds the wasm first (`pnpm verify` sets it itself):
`wasm-pack: not found` is a shell difference, not a broken build. Re-check the
wasm-threads blocker on `pa-wasm`, a `cdylib`: a library crate does not link,
and passes with the same flags.

## Resuming after a gap

Read `private/HANDOFF.md`; `private/README.md` lists its siblings (decision
history, corrections, measurement requests, the audit record). None is
published, and nothing published depends on them. Add a line to this file only
if it would stay true when the work moves on: a rule does, a status does not,
except a rule's known exceptions, which stay with the rule until fixed.
