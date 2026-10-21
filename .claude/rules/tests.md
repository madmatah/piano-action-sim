---
paths:
  - "crates/**"
  - "apps/bench/src/**"
  - "apps/bench/vite.config.ts"
  - "tools/smoke/**"
  - "scripts/smoke.sh"
---

# Tests

Tests are the specification of the rules: to learn how a part of the engine behaves, read its tests. Each rule ends with its check, or says it is *not mechanically checked*.

## Writing a test

- **A test's name is a sentence that states one rule,** in the trade's vocabulary: `the_regulated_key_meets_its_targets_then_lands_within_one_paper`, `it('never writes -0')`. *Not mechanically checked.*
- **One rule per test.** A test does not branch, and every expectation can fail: an expectation on a value the test built itself checks nothing. Inputs are named values, not unexplained literals. *Not mechanically checked.*
- **A validation test says so in its doc comment;** every other test is a consistency check, and a round trip through the model's own code never validates (`docs/FOUNDATIONS.md` > What a test proves; drive in position, compare forces). *Not mechanically checked.*
- **Every test asserts.** Check: Vitest runs with `--expect.requireAssertions`.
- **No test is committed skipped or focused:** `#[ignore]`, `it.skip`, `it.only` take a test out of the gate in silence. Check: `git grep -nE '#\[ignore|\b(it|describe|test)\.(skip|only)\(' crates apps` finds nothing.
- **A bug fix starts with a test** that reproduces the bug. *Not mechanically checked.*
- **Floating point is compared with a tolerance derived from the computation,** never tuned to pass, unless bit-exactness is the rule under test, as on the golden sheet. *Not mechanically checked.*
- **No test depends on timing, the wall clock or the order of the tests;** randomness is seeded, or reports its seed. *Not mechanically checked.*
- **A deleted test, or a loosened expectation, states its reason** in its change, and so does a dropped refusal message or a weaker assertion. *Not mechanically checked.*

## Levels

- **Unit tests** live next to the code they document: a Rust `#[cfg(test)] mod tests` at the end of its file, a TypeScript `<concept>.test.ts` beside `<concept>.ts`. *Not mechanically checked.*
- **The golden sheet** records every reading of the default bench and of the fixture benches in full bits (`crates/pa-app/tests/golden/`), held by `crates/pa-app/tests/golden.rs` natively and by `crates/pa-wasm/tests/golden.test.mjs` in the browser's build. Check: `cargo test --workspace`, `pnpm test`.
- **The smoke test** drives the built page in a headless Chrome over the DevTools protocol (`tools/smoke/`, `scripts/smoke.sh`): every control reaches the engine. Check: `bash scripts/smoke.sh`, required in CI.

## Test support

- **Only tests import test support.** `pa-core`'s test fixture compiles behind the `fixtures` feature, which only dev-dependencies enable, and the readings' closed forms, the oracles, compile for tests only. Check: `grep -rn oracle crates/pa-app/src crates/pa-wasm/src crates/pa-cli/src` finds nothing.
- **A test that runs git in a repository of its own drops the `GIT_` variables it inherits:** `git rebase --exec`, which checks each commit of a branch, exports `GIT_DIR` when it runs in a worktree, and the test's git would then work on the repository under rebase instead of its own. *Not mechanically checked.*

## Rust tests

- **A test is a `#[test]` function in the file's `mod tests`,** its name the sentence of its rule; a helper shared by the tests of a file computes and never asserts. *Not mechanically checked.*
- **A refusal is asserted on the value returned,** its kind and its numbers, so that a failure for another reason fails the test. *Not mechanically checked.*

## Vitest

- **A test that asserts nothing fails,** by `--expect.requireAssertions` in the bench's test script. Check: `apps/bench/package.json`.
- **A test file runs under Node:** the bench's tests exercise modules that build no DOM. *Not mechanically checked.*
- **`toBe` and `toEqual` tell `0` from `-0`,** which a bit-exact comparison relies on. *Not mechanically checked.*
