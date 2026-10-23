# CLAUDE.md

The session manual for agents working in this repository: what every session must know and do. The rules about what the repository holds live in `.claude/rules/`, each once and in full with its check, and [Rules](#rules) says when to read each file. `docs/FOUNDATIONS.md` holds what the simulator is for, what is built or planned, and why. Where a document disagrees with the code, the code is right.

One criterion governs everything (`docs/FOUNDATIONS.md` > The acceptance criterion): **the simulator must measure its own regulation using the trade's own protocols, and come back with the expected values.**

## Language

American English throughout: code, identifiers, comments, documents, commit messages, file names and the text the page shows. Three kinds of text keep their own: the French interface catalog, the bilingual glossaries, and a quotation or a title copied from a source, as printed. *Not mechanically checked.*

## Project

piano-action-sim is an engine for building applications around the grand piano action, for a piano technician, its physics in Rust compiled to WebAssembly; its first application is a single-key regulation bench (`apps/bench`). Built: stage 1a, the key on its frame, and of 1b the wippen resting on the capstan, and its weight on the sheet. Planned: the support spring, the hammer, the action in motion, 88 notes (`docs/FOUNDATIONS.md` > Stages). Read `docs/FOUNDATIONS.md` before changing the physics, the architecture, the dependencies or the tooling; its Crates, apps and gates maps the repository.

## The owner

He is the authority on the trade: vocabulary, priorities and what a part looks like are his call (`.claude/rules/vocabulary.md`); ask when a term is in doubt. Speak in trade quantities and his names; say what a modeling choice changes for regulating, not which code moved; ask for his measurements before drawing a part. Build by sub-assembly, engine and view together (`docs/FOUNDATIONS.md` > Staged by sub-assembly, with the view at every stage): he validates by looking, so each stage ends on a number he can judge by the trade's protocol.

## Where the work stands

`private/` is a separate, private Git repository nested at the root of the main checkout and ignored by this one: the owner's own material and the project's tracking (`.claude/rules/repository.md`). Read `private/HANDOFF.md` before touching anything; `private/README.md` lists its siblings (decision history, corrections, measurement requests, the audit record). Linked worktrees do not have it: from one, it is at `$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")/private`.

The plan of a branch is `plans/<branch>.md` in `private/`, at the path above even from a worktree: read it before working on the branch, and commit it there each time the `plan-critic` agent finds a version of it ready. When sessions run in parallel, each writes the plan of its own branch and names it in its final report, and the coordinating session commits it.

Add a line to this file only if it would stay true when the work moves on: a rule does, a status does not.

Never run `git clean` with `-ff` at the repository root: a double `-f` deletes nested repositories, `private/` among them.

## Workflow

- **Each piece of work has its own branch, `<type>/<topic>`, in its own worktree** under `.claude/worktrees/`, started from `origin/main`, or from an unmerged branch whose work it needs. The main checkout stays on `main` for the owner. A type is one of `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore` and `ci`.
- **Plan before writing code:** a `planner` agent writes the plan of each piece of work, kept in `private/` ([Where the work stands](#where-the-work-stands)), and a `plan-critic` agent, which did not write it, checks it against the code and the rules before any code is written. Only a change that leaves the design as it is goes without a plan: a typo, a mechanical rename, a dependency bump, a bug fix inside one function. A review fix stays within the plan: only a change of design, or the change of method that "Rounds of review converge, or the owner decides" calls for, goes back to it. The code follows the plan: an implementation that finds it wrong stops and says why, and the plan is revised and checked again before the work goes on.
- **Parallel sessions:** each owns one worktree and its branch, from its plan to its review fixes, and never works in another's. A round of review fixes goes to a new `fixer` agent in that worktree, as the `pull-request` skill says, never back to the agent that wrote the branch, whose context only grows. Only the coordinating session, the one that started the others or the one the owner names, writes to `private/`, apart from the plan of each session's own branch: the others put their decisions and handoff notes in their final report.
- **Dependencies:** a session that starts in a checkout without `node_modules` installs them from the lockfile (`.claude/settings.json`); in a worktree added during a session, run `pnpm install --frozen-lockfile` there first.
- **Give searches and explorations to cheaper agents,** on the model and at the effort that `.claude/rules/configuration.md` > Claude Code automation, "Each agent runs on the model and at the effort its task needs" gives for the task, through the Agent tool's `model` and `effort`. An agent defined in `.claude/agents/` runs as its definition says, unless a procedure raises or lowers its effort.
- **Dev servers:** the owner's `pnpm --filter @pas/app-bench dev` owns Vite's port, 5173. Start yours from your worktree on another (`--port 5174`), and stop it when you are done.
- **Check a visible change in a browser** before calling it done: load the page on your own server, exercise the change, read the console. The smoke test proves that every control reaches the engine, not that what is drawn is right.
- **"Done" means a test or a grep in the tree shows it,** never a message.
- **Commits:** commit as you like while working. Before pushing, rewrite the branch into atomic commits that each hold one reason to change and pass `pnpm verify`. The header of a message is `<gitmoji>(<scope>) <subject>`, with no colon: the gitmoji (https://gitmoji.dev) whose meaning matches the commit's dominant intention, as a Unicode character and never a shortcode, then the scope in parentheses, one space and the subject: `✨(pa-core) draw the band, the force down and up over the whole stroke`. A branch takes the type that matches the intention of its main change: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore` or `ci`.
- **A commit scope names where the change belongs:** the module, as `docs/FOUNDATIONS.md` > Crates, apps and gates names it (`pa-core`, `pa-app`, `pa-wasm`, `pa-cli`, `bench`), or else the area: `data` for the records and the profiles, `tools` for the derivations, the prototype and the smoke test, `build` for the build, the packaging and the tools' configuration, `ci` for the workflows, `docs` for `README.md`, `CREDITS.md` and the documents of `docs/`, `rules` for `CLAUDE.md` and `.claude/rules/`, a skill's name for that skill and the agents it runs, and `project` for the repository's own files, such as `.gitignore`, `.claude/settings.json` or the pull request template. A scope is in lowercase, and never a file name, a path or two scopes. The documents a change must update go in its commit, under its scope; a change that spans two modules is usually two commits, and one that cannot be split takes the scope of the module or area that owns the decision.
- **A commit message says why.** The subject says what the change achieves, in the reader's terms, and not the files it touches, which the diff lists: in the imperative, in lowercase, acronyms and product names included, a symbol keeping its own casing, without a final period, in a header of around 70 characters. The body, after a blank line and wrapped at 72 columns, gives what the diff cannot show: the motivation and the behavior it replaces, the constraint, the observation or the rejected alternative. It never lists the changes. Only a change that speaks for itself goes without a body: a typo, a mechanical rename, a dependency bump.
- **A commit footer holds only what tools read:** a breaking change, as `BREAKING CHANGE:` followed by what breaks and what its users must do, and the issues the commit closes, as `Closes #42`, never one that is not true. No other trailer, such as `Co-Authored-By`, `Generated with` or `Signed-off-by`: who typed a change says nothing of its reason.
- **Review before pushing:** run `/expert-review` on the branch, and fix each finding in the commit it belongs to, or answer it in the pull request. The review itself changes no code. Then review each round of fixes on its own diff, against the findings it answers, since a fix can bring the bug the next round finds: a round whose blocking and should-fix findings are all answered in the pull request, or that has none, ends the review, and the pull request names the fixes that no expert reviewed.
- **Rounds of review converge, or the owner decides:**
  - a round that finds a defect that the previous fix brought into the same code calls for another method, not another patch: a test that covers the whole input space, such as a sweep of a setting's range, a property test or a source's own worked values, a simpler design, or a library, planned and checked as "Plan before writing code" says. One round then reviews it;
  - before a third round in all, weigh what remains against the risk of the code and the cost of the rounds so far: a should-fix in code that handles only trusted input may be answered in the pull request instead;
  - after four rounds in all, the review of the whole branch included, ask the owner whether to go on, with what remains, its severity and the cost so far.
- **A branch reaches `main` only through a pull request whose CI passes.** The owner merges with "Rebase and merge", so each commit lands as written; review fixes are folded into their commits before a push with `--force-with-lease`. Never merge into `main` yourself.
- **The `pull-request` skill holds the procedure:** starting a branch, planning its work and writing its code, stacking it on another, shaping its history, reviewing it, opening and updating its pull request, and reviewing someone else's.

## Commands

```bash
pnpm install --frozen-lockfile       # first run; the session hook does it in a new worktree
pnpm verify                          # every CI gate but the frozen install
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p pa-wasm --target wasm32-unknown-unknown
cargo run -p pa-cli -- weigh [PROFILE]   # the weighing sheet, no browser
cargo run -p pa-cli -- band [PROFILE]    # the band: D and U by the sweep
cargo run -p pa-cli -- golden        # rewrites the golden sheet: say why
pnpm check                           # Biome (`pnpm exec biome`, never npx)
PATH="$HOME/.cargo/bin:$PATH" pnpm typecheck    # builds the wasm, then tsc
PATH="$HOME/.cargo/bin:$PATH" pnpm build && bash scripts/smoke.sh
PATH="$HOME/.cargo/bin:$PATH" pnpm test
pnpm --filter @pas/app-bench dev     # after a build; ?view=section, ?view=band
```

Node 26 and Rust 1.99.0 are pinned (`.nvmrc`, `rust-toolchain.toml`) so tools, lints and formatting match CI; `nvm` is on the owner's machine. `wasm-pack` is in `~/.cargo/bin`, off a non-interactive shell's `PATH`, hence the prefix on every task that builds the wasm first (`pnpm verify` sets it itself): `wasm-pack: not found` is a shell difference, not a broken build. Re-check the wasm-threads blocker on `pa-wasm`, a `cdylib`: a library crate does not link, and passes with the same flags.

## Rules

Each file of `.claude/rules/` holds the rules of one concern, in full, each ending with its check. Claude Code loads a file with `paths` when you read or edit a file that matches them, but a subagent in its own worktree or a compacted session can miss that load: read the file before you do what its line says.

- `repository.md`, loaded in every session: what belongs in the repository and in `private/`, the owner's names kept out, the documents a change updates, and how external work is credited.
- `conventions.md`, loaded in every session: naming and the trade's symbols, units and the frame of reference, finite numbers, comments, Rust, TypeScript, and input from outside the program.
- `physics.md`, for the crates, the page, the dynamics prototype and the manifests: one part definition for contacts and drawing, set-valued friction, no general-purpose engine, determinism and the golden sheet.
- `metrology.md`, for the crates, the page, the profiles and the smoke test: regulation as targets, readings by the trade's protocols, drift reported and the impossible refused, ratios as outputs, validation.
- `provenance.md`, for the data, the documents, the tools and the code that loads profiles: the record behind every starting number, and the "Do not use" index.
- `vocabulary.md`, for any document, the catalogs, the data and the crates' sources: the owner's terms, the one glossary, trade quantities.
- `architecture.md`, for the crates, the bench and the manifests, and before adding a dependency: layers, ports and wiring, responsibilities, stored and shared data, SOLID and organization.
- `configuration.md`, for a manifest, a lockfile, a toolchain pin, a tool's configuration, a script, CI or Claude Code's settings, skills and agents, before adding or upgrading a dependency, even with a command that edits no file you read, and before giving a task to an agent: modules, dependencies, continuous integration, checks, automation, the model and the effort of each agent included.
- `tests.md`, for any test or test support, and before fixing a bug, since the fix starts with a test: writing a test, levels, test support, Rust tests, Vitest.
- `documents.md`, for any document or agent guidance, and any reference to a document, tool messages included: present tense, where each statement lives, references by heading, rules files, expert reviewers, skills and the pull request template.

## Skills

- `pull-request`: from a new branch to a merged pull request, and reviewing someone else's.
- `expert-review`: `/expert-review [<pr#>|<branch>|<range>] [--fixes <file>] [--post] [focus]`, the review of a diff before it reaches `main`, by the generalist experts and the experts of metrology, physics, provenance and vocabulary.
- `writing-commits`: the message of a commit, from its gitmoji and its scope to its body and its footer.
