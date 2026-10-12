/**
 * Smoke test: does the bench actually respond to being used?
 *
 * # Why this exists
 *
 * A bench can pass every unit test, render correctly and screenshot correctly
 * while being completely dead: a slider or a button changes nothing because the
 * engine traps with `RuntimeError: memory access out of bounds` on the first
 * call after boot. Neither the Rust suite, the TypeScript suite, the type
 * checker nor a screenshot can see that — the failure is in the *liveness* of
 * the chain, and only driving the real page finds it.
 *
 * It needs no dependency. Node's built-in `WebSocket` speaks the Chrome
 * DevTools protocol directly, which is all this requires.
 *
 * One thing to know before trusting what you see through this protocol: a
 * capture taken within a few seconds of an edit catches Vite mid reload, and a
 * reloading page screenshots blank. A blank frame right after a save is almost
 * never the bug you just introduced. Wait, then capture.
 *
 * # Usage
 *
 * `scripts/smoke.sh` runs it against the built bench: it starts `vite preview`
 * and a headless Chrome, waits for both, runs this, and stops them. By hand,
 * against the dev server:
 *
 *   pnpm --filter @pas/app-bench dev --port 5199 &
 *   google-chrome --headless=new --remote-debugging-port=9222 \
 *       --use-angle=swiftshader --enable-unsafe-swiftshader http://localhost:5199/ &
 *   node tools/smoke/bench-interaction.mjs 9222 5199
 *
 * The two arguments are the DevTools port and the bench's port. It waits up to
 * 30 s for the page to appear and the engine to boot, then exits non-zero on the
 * first thing that fails to change.
 */

const [port = '9222', appPort = '5199'] = process.argv.slice(2)

/** Polls `probe` until it returns something truthy, or fails after 30 s. */
async function waitFor(what, probe) {
  const end = Date.now() + 30_000
  for (;;) {
    const found = await probe()
    if (found) return found
    if (Date.now() > end) fail(`waited 30 s for ${what}`)
    await new Promise((r) => setTimeout(r, 250))
  }
}

const page = await waitFor(`a page open on :${appPort}`, async () => {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()
  return targets.find((t) => t.type === 'page' && t.url.includes(appPort))
})

const ws = new WebSocket(page.webSocketDebuggerUrl)
let nextId = 0
const pending = new Map()
const thrown = []

ws.addEventListener('message', (e) => {
  const m = JSON.parse(e.data)
  if (m.id && pending.has(m.id)) {
    pending.get(m.id)(m)
    pending.delete(m.id)
  }
  if (m.method === 'Runtime.exceptionThrown') {
    const d = m.params.exceptionDetails
    thrown.push(d.exception?.description ?? d.text)
  }
})
await new Promise((r) => ws.addEventListener('open', r))

const send = (method, params = {}) =>
  new Promise((res) => {
    const id = ++nextId
    pending.set(id, res)
    ws.send(JSON.stringify({ id, method, params }))
  })

await send('Runtime.enable')

const evaluate = async (expression) => {
  const r = await send('Runtime.evaluate', {
    expression,
    awaitPromise: true,
    returnByValue: true,
  })
  if (r.result?.exceptionDetails) {
    fail(`evaluate threw: ${r.result.exceptionDetails.exception?.description ?? ''}`)
  }
  return r.result?.result?.value
}

const settle = () => new Promise((r) => setTimeout(r, 400))

/** The whole weighing sheet, as `label=value` pairs. */
const sheet = () =>
  evaluate(`
    JSON.stringify(Object.fromEntries([...document.querySelectorAll('.row')].map(
      (r) => [r.querySelector('.label').textContent, r.querySelector('.value').textContent])))
  `).then(JSON.parse)

const moveSlider = (label, value) =>
  evaluate(`
    (() => {
      const l = [...document.querySelectorAll('label.slider')]
        .find((x) => x.textContent.includes(${JSON.stringify(label)}))
      if (!l) return 'no such slider'
      const input = l.querySelector('input')
      const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set
      set.call(input, '${value}')
      input.dispatchEvent(new Event('input', { bubbles: true }))
      return 'ok'
    })()
  `)

const clickButton = (text) =>
  evaluate(`
    (() => {
      const b = [...document.querySelectorAll('button')].find((x) => x.textContent.trim() === ${JSON.stringify(text)})
      if (!b) return 'no such button'
      b.click()
      return 'ok'
    })()
  `)

/**
 * Clicks inside one named punching stack.
 *
 * The scoping matters: `lift off` appears once per paper on *both* stacks, so an
 * unscoped click takes a paper off whichever pile the DOM happens to list first
 * — a different thickness, on a different rail. The failure then looks like an
 * engine bug and is a test bug.
 */
const clickInStack = (titleFragment, text, which = 'first') =>
  evaluate(`
    (() => {
      const stack = [...document.querySelectorAll('.stack')]
        .find((s) => s.querySelector('.stacktitle').textContent.includes(${JSON.stringify(titleFragment)}))
      if (!stack) return 'no such stack'
      const all = [...stack.querySelectorAll('button')].filter((x) => x.textContent.trim() === ${JSON.stringify(text)})
      const b = ${JSON.stringify(which)} === 'last' ? all[all.length - 1] : all[0]
      if (!b) return 'no such button in that stack'
      b.click()
      return 'ok'
    })()
  `)

function fail(message) {
  console.error(`FAIL ${message}`)
  process.exit(1)
}

function expectChanged(what, before, after, key) {
  if (before[key] === after[key]) {
    fail(`${what}: "${key}" did not change, still ${before[key]}`)
  }
  console.log(`  ok  ${what}: ${key} ${before[key]} -> ${after[key]}`)
}

function expectUnchanged(what, before, after, key) {
  if (before[key] !== after[key]) {
    fail(`${what}: "${key}" should not have changed, ${before[key]} -> ${after[key]}`)
  }
  console.log(`  ok  ${what}: ${key} held at ${before[key]}`)
}

console.log('bench interaction smoke test')

// The page shows "starting the engine…" until the wasm has booted, and only
// then the weighing sheet.
const start = await waitFor('the engine to boot and fill the weighing sheet', async () => {
  const s = await sheet()
  return Object.keys(s).length > 0 ? s : undefined
})

// If the page was opened with leads in its URL, exactly that many must be
// fitted. React's StrictMode runs an effect body twice, so one-time setup
// placed in the caller's effect is applied twice: a URL asking for three leads
// fits six, and the only visible symptom is a key mass 66 g too high.
// Boot-time setup belongs inside the memoised boot promise.
const asked = await evaluate(
  `(new URLSearchParams(location.search).get('leads') ?? '').split(',').filter(Boolean).length`,
)
if (asked > 0) {
  const fitted = await evaluate(`document.querySelectorAll('.leads li').length`)
  if (fitted !== asked) {
    fail(`the URL asked for ${asked} lead(s) and ${fitted} are fitted`)
  }
  console.log(`  ok  url state: ${asked} lead(s) asked for, ${fitted} fitted`)
}

// The non-confusion test, run through the whole chain rather than in a unit
// test: more friction must widen the band and leave its centre alone.
if ((await moveSlider('key friction weight', 14)) !== 'ok') fail('friction slider missing')
await settle()
const moreFriction = await sheet()
expectChanged('friction slider', start, moreFriction, 'friction weight F')
expectUnchanged('friction slider', start, moreFriction, 'balance weight BW')

// The leading workshop must reach the engine.
if ((await clickButton('150 mm')) !== 'ok') fail('lead button missing')
await settle()
const leaded = await sheet()
expectChanged('add lead', moreFriction, leaded, 'key mass')
expectChanged('add lead', moreFriction, leaded, 'balance weight BW')
expectUnchanged('add lead', moreFriction, leaded, 'friction weight F')

// Regulation must reach the engine too. A punching stack is a pile of papers,
// so the operation is a button in the drawer and not a slider.
if ((await clickInStack('key height', '0.20')) !== 'ok') fail('no 0.20 paper in the drawer')
await settle()
const levelled = await sheet()
expectChanged('lay a paper', leaded, levelled, 'key height')

// And lifting *that* paper back off must restore the key exactly. A regulation
// that is not reversible is not a regulation.
//
// The `last` matters: clicking the first "lift off" takes a paper off the
// *balance* stack's bottom, which is a different thickness. The wrong click
// reads 0.49 mm low — exactly 0.20 added minus 0.40 removed through the 2.47
// back-rail lever — and that is a test bug, not an engine bug.
if ((await clickInStack('key height', 'lift off', 'last')) !== 'ok') {
  fail('no way to lift a paper off')
}
await settle()
const unlevelled = await sheet()
if (unlevelled['key height'] !== leaded['key height']) {
  fail(
    `lifting the paper off left key height at ${unlevelled['key height']}, was ${leaded['key height']}`,
  )
}
console.log(`  ok  lift it off: key height back to ${unlevelled['key height']}`)

// The measuring point is a protocol setting every touchweight reads. Moving it
// must change front weight and the key weight ratio by the lever, carry the
// friction torque onto the new arm, and leave key height and dip alone: they
// are read at the key front, not at the measuring point.
if ((await moveSlider('measuring point, back from', 20)) !== 'ok') {
  fail('measuring point slider missing')
}
await settle()
const remeasured = await sheet()
expectChanged('measuring point', unlevelled, remeasured, 'front weight FW')
expectChanged('measuring point', unlevelled, remeasured, 'key weight ratio KR')
expectChanged('measuring point', unlevelled, remeasured, 'friction weight F')
expectUnchanged('measuring point', unlevelled, remeasured, 'key height')
expectUnchanged('measuring point', unlevelled, remeasured, 'key dip')

if (thrown.length > 0) fail(`the page threw: ${thrown[0].split('\n')[0]}`)

console.log('all interactions reached the engine')
ws.close()
