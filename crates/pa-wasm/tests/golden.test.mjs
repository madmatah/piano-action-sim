/**
 * C10, consistency: the golden sheet's bits are the same in WebAssembly as
 * natively.
 *
 * The native test suite holds the engine to `crates/pa-app/tests/golden/`,
 * every reading of five benches in full bits. This loads the module the
 * browser loads, built by wasm-pack, asks it for the same sheets and compares
 * them line for line. It is what makes the claim that the bench and
 * `pa-cli weigh` read the same numbers a test rather than a hope: every
 * transcendental goes through `libm`, every loop runs in a fixed order, and a
 * difference in one last bit fails here.
 *
 * Run by `pnpm test` (`node --test`), after the wasm build.
 */
import assert from 'node:assert/strict'
import { readdirSync, readFileSync } from 'node:fs'
import { test } from 'node:test'
import { golden_sheet, initSync } from '../pkg/pa_wasm.js'

initSync({ module: readFileSync(new URL('../pkg/pa_wasm_bg.wasm', import.meta.url)) })

const golden = new URL('../../pa-app/tests/golden/', import.meta.url)

test('every golden sheet reads the same bits in WebAssembly as natively', () => {
  const files = readdirSync(golden).filter((f) => f.endsWith('.txt'))
  assert.ok(files.length >= 5, `only ${files.length} golden sheets`)
  for (const file of files) {
    const recorded = readFileSync(new URL(file, golden), 'utf8')
    const computed = golden_sheet(file.replace(/\.txt$/, ''))
    assert.equal(computed, recorded, `${file}: the WebAssembly engine reads other bits`)
  }
})
