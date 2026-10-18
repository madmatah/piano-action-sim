#!/usr/bin/env bash
# The browser smoke test: drives the built bench in a headless Chrome and checks
# that every interaction reaches the engine (tools/smoke/bench-interaction.mjs).
#
#   pnpm build && bash scripts/smoke.sh
#
# It serves apps/bench/dist with `vite preview`, so build first. It starts the
# preview server and Chrome itself, on free ports and with a throwaway profile,
# and stops both on exit, whatever happens.
#
# Exit codes: 0 passed, 1 failed, 3 skipped because no Chrome binary was found.
# With SMOKE_REQUIRE_CHROME=1, which CI sets, a missing Chrome is a failure.
# CHROME=/path/to/chrome picks the binary.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

chrome=""
for candidate in "${CHROME:-}" google-chrome google-chrome-stable chromium-browser chromium; do
  if [[ -n "$candidate" ]] && command -v "$candidate" >/dev/null 2>&1; then
    chrome="$(command -v "$candidate")"
    break
  fi
done
if [[ -z "$chrome" ]]; then
  {
    echo
    echo "!!! SMOKE TEST SKIPPED: no Chrome binary found (tried google-chrome,"
    echo "!!! google-chrome-stable, chromium-browser, chromium; set CHROME to choose)."
    echo "!!! The bench was NOT driven in a browser."
    echo
  } >&2
  if [[ "${SMOKE_REQUIRE_CHROME:-0}" == "1" ]]; then
    exit 1
  fi
  exit 3
fi

if [[ ! -f apps/bench/dist/index.html ]]; then
  echo "smoke: apps/bench/dist is missing; run 'pnpm build' first" >&2
  exit 1
fi

scratch="$(mktemp -d)"
pids=()

# Each server runs as the leader of its own process group, so stopping the
# group also stops whatever it spawned (Chrome's renderers and GPU process).
stop() {
  local pid=$1
  kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || return 0
  for _ in $(seq 50); do
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 0.1
  done
  kill -KILL -- "-$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null || true
}

cleanup() {
  local code=$?
  for pid in "${pids[@]}"; do
    stop "$pid"
  done
  if [[ $code -ne 0 ]]; then
    for log in "$scratch"/*.log; do
      [[ -f "$log" ]] || continue
      echo "--- last lines of $(basename "$log")" >&2
      tail -n 20 "$log" >&2
    done
  fi
  rm -rf "$scratch"
  exit "$code"
}
trap cleanup EXIT

free_port() {
  node -e 'const s = require("net").createServer()
s.listen(0, "127.0.0.1", () => { console.log(s.address().port); s.close() })'
}

wait_for() {
  local url=$1 seconds=$2
  node -e 'const [url, seconds] = process.argv.slice(1)
const end = Date.now() + Number(seconds) * 1000
const tick = async () => {
  try {
    if ((await fetch(url)).ok) process.exit(0)
  } catch {}
  if (Date.now() > end) {
    console.error(`smoke: timed out after ${seconds} s waiting for ${url}`)
    process.exit(1)
  }
  setTimeout(tick, 250)
}
tick()' "$url" "$seconds"
}

app_port="$(free_port)"
devtools_port="$(free_port)"
while [[ "$devtools_port" == "$app_port" ]]; do
  devtools_port="$(free_port)"
done

echo "smoke: vite preview on 127.0.0.1:$app_port"
setsid apps/bench/node_modules/.bin/vite preview apps/bench \
  --host 127.0.0.1 --port "$app_port" --strictPort >"$scratch/vite-preview.log" 2>&1 &
pids+=("$!")
wait_for "http://127.0.0.1:$app_port/" 60

# WebGL through SwiftShader, so the three.js view renders with no GPU. On a CI
# runner the browser sandbox depends on the image's kernel settings; the page
# loaded is only the bench served above, so CI runs without it. The window
# has a set size rather than the binary's default, which differs from one
# Chrome build to another (a CI runner's left a page 437 px high), since the
# checks of the layout read the window's height.
chrome_flags=(
  --headless=new
  --no-first-run
  --no-default-browser-check
  --user-data-dir="$scratch/chrome-profile"
  --remote-debugging-port="$devtools_port"
  --use-angle=swiftshader
  --enable-unsafe-swiftshader
  --window-size=1280,800
)
if [[ "${CI:-}" == "true" ]]; then
  chrome_flags+=(--no-sandbox)
fi

echo "smoke: $chrome, DevTools on 127.0.0.1:$devtools_port"
setsid "$chrome" "${chrome_flags[@]}" "http://127.0.0.1:$app_port/" >"$scratch/chrome.log" 2>&1 &
pids+=("$!")
wait_for "http://127.0.0.1:$devtools_port/json/version" 60

node tools/smoke/bench-interaction.mjs "$devtools_port" "$app_port"
