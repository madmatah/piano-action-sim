---
paths:
  - "crates/**"
  - "apps/**"
  - "Cargo.toml"
  - "**/package.json"
  - "pnpm-workspace.yaml"
---

# Architecture

The project follows Clean Architecture, in four layers, and applies SOLID to modules, functions and types. Each rule ends with its check, or says it is *not mechanically checked*.

## Layers

- **Four layers, from the inside out:** the domain, `crates/pa-core`; the application, `crates/pa-app`, the single-key bench as an instrument; the adapters, `crates/pa-wasm` for the browser, `crates/pa-cli` for the shell and `apps/bench` for the page; and the composition roots, the adapters' entry points and the tests. A crate's or a package's layer is fixed by this list. *Not mechanically checked.*
- **Dependencies point inward.** `pa-core` depends on no crate of the project, `pa-app` on `pa-core`, the adapters on `pa-app`, and nothing inner names anything outer: no import, no type, no name. Check: the manifests; Cargo refuses a cycle.
- **`pa-core` has no I/O and no web dependency, and justifies each dependency** in its manifest: `libm` today, for every transcendental (`.claude/rules/physics.md` > Determinism); `serde` behind a feature, with persistence, is the planned one. Its one feature, `fixtures`, compiles its test fixture for `pa-app`'s tests, and nothing shipped enables it. Check: the manifests; `grep -rnE 'std::(fs|io|net)|wasm_bindgen' crates/pa-core/src` finds nothing.
- **The adapters add no physics:** `pa-wasm` and `pa-cli` adapt `pa-app`, and the page computes none of the numbers it shows. *Not mechanically checked.*

## Ports and wiring

- **Inner layers own their ports.** When the application needs the outside world, it declares the interface in its own vocabulary and an adapter implements it; the application reaches a technology only through that port. *Not mechanically checked.*
- **Only composition roots wire, and by hand:** the graph stays in one place and checked by the types. No DI container, no service locator, no singleton, and no module-level mutable state shared between callers. *Not mechanically checked.*
- **Plain data crosses boundaries:** the instrument sends codes and numbers in trade units, which the page words (`docs/FOUNDATIONS.md` > The WebAssembly boundary); no framework or platform type crosses a port. *Not mechanically checked.*
- **A module's public API is its single entry point:** a crate's `lib.rs`, a package's index. Nothing reaches inside another module. *Not mechanically checked.*

## Responsibilities

- **The domain holds the rules and is pure:** no I/O, no clock, no global mutable state, no framework, no platform API. *Not mechanically checked.*
- **The application orchestrates:** it composes the instrument from a profile, sets by path, acts by action, logs and replays; a rule it needs belongs in the domain. *Not mechanically checked.*
- **Adapters translate, nothing more,** and are named after their technology; composition roots decide no rule either. *Not mechanically checked.*

## Stored and shared data

- **What is stored, sent or replayed is plain data** that survives a round trip through its serialized form unchanged: the documents across the WebAssembly boundary, the profiles, the action log whose replay gives the same state hash. Persistence is planned on the same terms. Check: `the_log_replayed_from_the_profile_reproduces_the_state`, `the_state_hash_changes_with_every_coordinate_and_the_hidden_state`, `the_generated_types_are_fresh`.

## SOLID

- **Single responsibility:** a module, a function or a type has one reason to change, and a file holds one concept. *Not mechanically checked.*
- **Open for extension:** a new case is added as data, or beside the existing cases, without rewriting the code that handles them; a new key is a profile, never a type of its own. *Not mechanically checked.*
- **Substitutable implementations:** every implementation of a port, a test's included, honors its whole contract: units, ordering, refusals, missing values. *Not mechanically checked.*
- **Narrow ports,** in their client's vocabulary. *Not mechanically checked.*
- **No speculative generality:** no abstraction with a single caller and no rule behind it; a package is extracted when a second application needs it, not created empty before. *Not mechanically checked.*

## Organization

- **Modules are named after domain concepts** (`geometry`, `mechanism`, `contact`, `solve`, `parts`, `measure`, `regulation`, `registry`, `validate`), never after technical kinds: `utils`, `helpers`, `services`, `models` or `common`. A crate is `pa-<role>`; a package is scoped `@pas/`. *Not mechanically checked.*
