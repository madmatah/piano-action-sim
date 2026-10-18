# Code conventions

These rules apply to code, tests and configuration, in every part of the repository. Each ends with its check, or says it is *not mechanically checked*.

## Naming

Names carry the meaning that comments may not carry.

- **Explicit, in the vocabulary of the domain:** the trade's quantities and the glossary's names (`down_weight_g`, `key_dip_mm`, `balance_punching`), and a concept keeps one name in the documents, the code and the tests. *Not mechanically checked.*
- **No generic names:** `data`, `info`, `value`, `item`, `result`, `temp`, `obj`, `manager`, `handler`, `helper`, `util`. *Not mechanically checked.*
- **No abbreviations and no single letters,** outside the idioms of [Allowed idioms](#allowed-idioms): `elapsed_seconds`, not `dt`; `row_index`, not `i`. *Not mechanically checked.*
- **Booleans are predicates:** `is_seated`, `has_record`. Functions start with a verb: `solve_seat`, `weigh_key`; types and values are nouns. *Not mechanically checked.*
- **Casing is the language's:** `snake_case` functions, fields and modules and `UpperCamelCase` types in Rust; `camelCase` functions and values and `PascalCase` types and components in TypeScript. A file is named after the concept it holds. Check: rustc's naming lints for Rust; *not mechanically checked* for TypeScript.

### Allowed idioms

- **The trade's symbols**, as the trade and Stanwood write them: `D`, `U`, `BW`, `F`, `FW`, `KR`, `KF`, `SW`, `WW` and their kin in the glossary.
- **The symbols of a formula transcribed from a source,** as the source prints them: Hirschkorn's `Pk2`, `A`, `B1`, `B2`; a point's `x` and `y`.
- **Rust's single-letter type parameters,** as the standard library writes them: `T`, `E`.
- **A name the platform imposes,** such as a trait method or a JavaScript API.

## Units

- **SI unless the name says otherwise.** Geometry and mechanics are SI; a value in a trade unit (grams in the weighing algebra; millimeters and grams at the boundary, in the display and in saved files) carries it in its name (`down_weight_g`, `key_dip_mm`), and a measurement its unit beside its value. An unsuffixed trade-unit field is a bug, as is a `*_torque` returning kg·m. Check: review; `grep -rnE '_(mm|g)\b' crates/pa-core/src` lists the suffixed names.
- **One frame of reference: the sagittal section, `x` toward the player, `y` up.** A body's frame has its origin at its own joint; the key's at the balance pin on its underside. Angles turn counter-clockwise in that view, so a positive angle lifts the front of a key. A trade reading keeps the trade's sign at the boundary, such as an out-of-level angle nose-down positive, and says so in its name or its documentation. Data transcribed from a source keeps the source's frames, stated in its record. *Not mechanically checked.*
- **No number that is not finite reaches the core:** the validation layer refuses a `NaN` or an infinity before any solve, and a division by a quantity that can reach zero, or a square root of one that can go negative, is guarded where it is written. Check: `no_number_that_is_not_finite_reaches_the_core` for the inputs; *not mechanically checked* for the divisions.

## Comments

The names and the structure say what the code does, and the tests state the rules, so a comment can only add a why.

- **No comments, except a why.** A comment never says what the code does; a workaround names the bug it works around, a constraint taken from a source names the source. *Not mechanically checked.*
- **Documentation comments document the public API** of the crates, as `cargo doc` reads them, and of the TypeScript modules: each says what the item is for, in the trade's terms, and never restates the signature. `pa-core` requires one on every public item. Check: `missing_docs`, a warning that Clippy's `-D warnings` makes an error.
- **Allowed directives:** Clippy's `#[allow(…)]` and `#[expect(…)]`, Biome's suppression comment, and `@ts-expect-error`; each states its reason. `@ts-ignore` and `@ts-nocheck` are refused: they outlive the errors they silence. *Not mechanically checked.*
- **A version label next to a pinned commit SHA is allowed,** such as `# v7.0.1` after an action in a workflow. *Not mechanically checked.*
- **Forbidden:** commented-out code, and banners. *Not mechanically checked.*

## Rust

- **Clippy's warnings are errors,** in every crate and every target. Check: `cargo clippy --workspace --all-targets -- -D warnings`.
- **No `unsafe` in `pa-core`.** Check: `#![forbid(unsafe_code)]`.
- **Formatting is rustfmt's.** Check: `cargo fmt --all --check`.

## TypeScript

- **Strict,** with `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`. Check: `tsconfig.base.json`, `pnpm typecheck`.
- **Named exports only;** a tool's configuration file that the tool reads through its default export is the exception. *Not mechanically checked.*
- **Erasable syntax only:** no `enum` and no `namespace`, but unions of literals and `as const` objects. *Not mechanically checked.*
- **No `any`.** *Not mechanically checked.*
- **The page computes none of the numbers it shows:** it renders what the engine sends, poses as given. *Not mechanically checked.*
- **Formatting and linting are Biome's.** Check: `pnpm check`.

## Input from outside the program

- **Outside input is checked before use:** a value set by path, a profile, a URL parameter, a command-line argument is checked for its shape and its bounds, and malformed input is refused with its reason, the instrument left as it was. Check: `each_problem_is_refused_and_leaves_the_instrument_as_it_was`, `no_number_that_is_not_finite_reaches_the_core`.
- **Outside text never reaches a parser of markup, script, URLs or command lines:** the page sets text through React, never `innerHTML` or `dangerouslySetInnerHTML`, and builds no `href` or `src` from outside text; a command's arguments go as a list, never through a shell. Check: `git grep -nE 'innerHTML|dangerouslySetInnerHTML' apps` finds nothing.
