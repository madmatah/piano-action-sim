---
paths:
  - "**/Cargo.toml"
  - "**/package.json"
  - "Cargo.lock"
  - "pnpm-lock.yaml"
  - "pnpm-workspace.yaml"
  - "rust-toolchain.toml"
  - ".nvmrc"
  - "rustfmt.toml"
  - "biome.json"
  - "turbo.json"
  - "**/tsconfig*.json"
  - "**/clippy.toml"
  - "**/vite.config.ts"
  - "scripts/**"
  - ".github/workflows/**"
  - ".claude/settings.json"
  - ".claude/agents/**"
  - ".claude/skills/**"
---

# Configuration

Each rule ends with its check, or says it is *not mechanically checked*.

## Modules

- **A module is a crate or a package,** with its own manifest: the crates of `crates/` and the bench of `apps/bench/`; `pa-wasm`, the crate the browser loads, also carries a `package.json`, so that Turborepo drives wasm-pack. *Not mechanically checked.*
- **The documents list every module.** A change that adds, renames or removes one, or changes its layer or its dependencies on the others, updates `docs/FOUNDATIONS.md` > Crates, apps and gates and the table of `README.md` in the same change. *Not mechanically checked.*

## Dependencies

- **A new dependency comes from a known, maintained publisher, under a license compatible with the project's MIT License:** a permissive one, such as MIT, Apache-2.0, BSD or ISC, never a copyleft license whose terms would bind the project's code. *Not mechanically checked.*
- **A dependency of `pa-core` is justified in its manifest,** beside it. Check: `crates/pa-core/Cargo.toml`.
- **Never weaken what the lockfiles guarantee.** `Cargo.lock` and `pnpm-lock.yaml` are committed, and CI and the session hook install with the frozen lockfile; no setting lets an install bypass them, such as another registry. The tools no lockfile holds are pinned where they are read: Rust in `rust-toolchain.toml`, Node in `.nvmrc`, pnpm in `packageManager`, wasm-pack in the workflow. A version that may move a last bit of a reading, `libm`'s first, is updated on purpose and its golden sheet rewritten with the reason. Check: `pnpm install --frozen-lockfile` in CI.

## Continuous integration

- **Actions are pinned to full commit SHAs,** each with its version label beside it, because a tag can move to other code. Check: `.github/workflows/`.
- **Permissions stay read-only,** and the checkout keeps no credentials. Check: `.github/workflows/`.
- **Event data reaches a `run:` script only through `env:`,** never through `${{ }}` inside the script, where it would be spliced into shell code. *Not mechanically checked.*
- **No `pull_request_target` workflow checks out untrusted code, and no secret reaches a run from a fork.** *Not mechanically checked.*
- **CI runs the gates of `pnpm verify`**, and `scripts/verify.sh` runs them locally in the same order; a gate added to one is added to the other. *Not mechanically checked.*

## Checks

- **A check that enforces a rule is never narrowed to let a change pass:** no file excluded, no lint turned off or lowered, no tolerance widened, no step dropped from the gate or from its CI run. A golden sheet is rewritten only by a change that means to move a reading, and says why. A change to the rule itself changes its rules file and its check together, and says why. *Not mechanically checked.*

## Claude Code automation

- **No hook, skill or agent runs text taken from a pull request or a reviewed file** as a command, and no setting widens permissions. The session hook only installs the dependencies from the frozen lockfile. *Not mechanically checked.*
