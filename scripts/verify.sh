#!/usr/bin/env bash
# The gates CI runs, in the same order, on this machine.
#
#   pnpm verify            (or: bash scripts/verify.sh)
#
# Run `pnpm install` once first: CI's `pnpm install --frozen-lockfile`, which
# fails on a lockfile out of step with the manifests, is the one CI step not
# repeated here. It stops at the first gate that fails. The browser smoke test
# needs a Chrome binary; without one it is skipped, and the run ends on a
# SKIPPED line instead of a full pass, though it still exits 0.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# wasm-pack installs into ~/.cargo/bin, which a non-interactive shell does not
# put on PATH; without this the wasm build dies with "wasm-pack: not found".
export PATH="$HOME/.cargo/bin:$PATH"

if [[ ! -d node_modules ]]; then
  echo "verify: no node_modules; run 'pnpm install' first" >&2
  exit 1
fi

gate() {
  printf '\n==> %s\n' "$*"
  "$@"
}

gate cargo fmt --all --check
gate cargo clippy --workspace --all-targets -- -D warnings
gate cargo test --workspace
gate cargo build -p pa-wasm --target wasm32-unknown-unknown --release
gate pnpm check
gate pnpm typecheck
gate pnpm build
gate pnpm test

printf '\n==> %s\n' "bash scripts/smoke.sh"
smoke=0
bash scripts/smoke.sh || smoke=$?

if [[ $smoke -eq 3 ]]; then
  printf '\nEvery gate passed EXCEPT the browser smoke test, which was SKIPPED.\n' >&2
  exit 0
fi
if [[ $smoke -ne 0 ]]; then
  exit "$smoke"
fi
printf '\nEvery gate passed, the browser smoke test included.\n'
