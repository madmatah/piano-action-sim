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

/**
 * Every reading of the weighing sheet, by the engine's identifier: what the
 * trade reads it as and its value in full. The rounded reading is what a
 * technician compares; the value in full is what a derived bound can hold.
 */
const measured = () =>
  evaluate(`
    JSON.stringify(Object.fromEntries([...document.querySelectorAll('[data-procedure]')].map(
      (r) => [r.dataset.procedure, { shown: Number(r.dataset.shown), exact: Number(r.dataset.exact) }])))
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

/** Types a number into a labelled field, as a technician types a target. */
const setField = (label, value) =>
  evaluate(`
    (() => {
      const l = [...document.querySelectorAll('label.field')]
        .find((x) => x.textContent.includes(${JSON.stringify(label)}))
      if (!l) return 'no such field'
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

/** A reading's value in full moved, by more than nothing. */
function expectMoved(what, before, after, key) {
  const [a, b] = [before[key]?.exact, after[key]?.exact]
  if (!Number.isFinite(a) || !Number.isFinite(b) || a === b) {
    fail(`${what}: ${key} in full did not move, ${a} -> ${b}`)
  }
  console.log(`  ok  ${what}: ${key} in full ${a} -> ${b}`)
}

/** A reading's value in full moved by no more than `bound`, its derivation said beside. */
function expectWithin(what, before, after, key, bound) {
  const moved = Math.abs(after[key]?.exact - before[key]?.exact)
  if (!(moved <= bound)) {
    fail(`${what}: ${key} in full moved by ${moved}, more than ${bound}`)
  }
  console.log(`  ok  ${what}: ${key} in full moved by ${moved.toFixed(4)}, within ${bound}`)
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

// Every weight is read the way gram weights give it: the down weight and the
// key friction weights as the least whole gram that does it, the up weight as
// the most the key lifts. None of these stands within the walk's bracket of a
// whole gram, a part in 65 536 of it, which would read as that gram. Both key
// friction weights are on the sheet, labelled.
const startRead = await measured()
for (const [key, rule] of [
  ['down_weight', 'up'],
  ['key_friction_weight_zeroed_on_the_jig', 'up'],
  ['key_friction_weight_zeroed_on_the_frame', 'up'],
  ['up_weight', 'down'],
]) {
  const m = startRead[key]
  const read = rule === 'up' ? Math.ceil(m?.exact) : Math.floor(m?.exact)
  if (!m || !Number.isInteger(m.shown) || m.shown !== read) {
    fail(`${key} is not read ${rule} to the gram: ${JSON.stringify(m)}`)
  }
  console.log(`  ok  read to the gram: ${key} ${m.exact} reads ${m.shown} g`)
}
for (const label of [
  'key friction weight KF, zeroed on the jig',
  'key friction weight KF, zeroed on the frame',
]) {
  if (!start[label]) fail(`the sheet has no row "${label}"`)
}

// The friction slider shows what Stanwood's protocol reads the bushings as, in
// full, from the sheet: at boot, after a weight is typed or refused, and after
// anything that changes what the same bushings read. It is the key friction
// weight the owner types, so a number there the sheet does not read is a
// setting the model does not hold.
const frictionShown = () =>
  evaluate(`
    [...document.querySelectorAll('label.slider')]
      .find((x) => x.textContent.includes('key friction weight'))
      ?.querySelector('b')?.textContent ?? null
  `)
const expectFrictionShown = async (what, exact) => {
  const shown = await frictionShown()
  if (!Number.isFinite(exact) || shown !== `${exact.toFixed(2)} g`) {
    fail(
      `${what}: the friction slider shows ${shown}, not the ${exact} g Stanwood's protocol reads`,
    )
  }
  console.log(`  ok  the friction slider shows the sheet's reading, ${what}: ${shown}`)
}
await expectFrictionShown('at boot', startRead.key_friction_weight_zeroed_on_the_jig?.exact)

// The non-confusion test, run through the whole chain rather than in a unit
// test: more friction must widen the band and leave its centre alone. D is
// read at the top seat and U at the bottom, where the measuring point's arm
// differs by under one per cent, so 8 g more friction moves the balance
// weight in full by under 8 × 1 % / 2 = 0.04 g, and not the gram it is read
// to.
if ((await moveSlider('key friction weight', 14)) !== 'ok') fail('friction slider missing')
await settle()
const moreFriction = await sheet()
const moreFrictionRead = await measured()
expectChanged('friction slider', start, moreFriction, 'friction weight F')
expectUnchanged('friction slider', start, moreFriction, 'balance weight BW')
expectWithin('friction slider', startRead, moreFrictionRead, 'balance_weight', 0.05)

// The key friction weight is typed as Stanwood's protocol reads it, and reads
// back as typed: the engine sets the bushings to the torque at which the
// protocol, run on the model, reads the typed grams in full, to the walk's
// resolution, half a bracket of 1/65 536 g. The value in full crosses the
// boundary narrowed to f32, half a unit in its last place under a millionth of
// a gram at 14 g; so it stands within 1e-5 g of 14, and 14 g shows as 14 g.
const typedBack = moreFrictionRead.key_friction_weight_zeroed_on_the_jig
if (typedBack?.shown !== 14 || !(Math.abs(typedBack.exact - 14) <= 1e-5)) {
  fail(
    `14 g of key friction weight typed does not read back as typed: ${JSON.stringify(typedBack)}`,
  )
}
console.log(`  ok  typed back: 14 g reads ${typedBack.exact} g, shown ${typedBack.shown} g`)
await expectFrictionShown('14 g typed', typedBack.exact)

// A key friction weight no bushings read is refused with its reason, and the
// bushings stay as they were: with no friction at all this key reads about a
// third of a gram by Stanwood's protocol, what its tilt at rest leaves of its
// own balance and the temporary weight's, so 0 g is out of reach.
if ((await moveSlider('key friction weight', 0)) !== 'ok') fail('friction slider missing')
await settle()
const frictionRefused = await evaluate(
  `document.querySelector('.friction-refused')?.textContent ?? null`,
)
if (frictionRefused === null || !/no friction at all/.test(frictionRefused)) {
  fail(`0 g of key friction weight was not refused with its reason: ${frictionRefused}`)
}
const keptRead = await measured()
for (const key of ['key_friction_weight_zeroed_on_the_jig', 'friction_weight']) {
  if (keptRead[key]?.exact !== moreFrictionRead[key]?.exact) {
    fail(`a refused key friction weight moved ${key}: ${JSON.stringify(keptRead[key])}`)
  }
}
console.log(`  ok  refused: ${frictionRefused}`)
await expectFrictionShown('0 g refused', keptRead.key_friction_weight_zeroed_on_the_jig?.exact)

// The leading workshop must reach the engine, and 10 g of lead move the
// balance weight and not the friction weight: the lead's arm and the
// measuring point's change with the key's tilt between its two seats by
// under a thousandth of their ratio, so the friction weight in full moves by
// under 10 g × 0.001 / 2, well within 0.05 g.
if ((await clickButton('150 mm')) !== 'ok') fail('lead button missing')
await settle()
const leaded = await sheet()
const leadedRead = await measured()
expectChanged('add lead', moreFriction, leaded, 'key mass')
expectChanged('add lead', moreFriction, leaded, 'balance weight BW')
expectUnchanged('add lead', moreFriction, leaded, 'friction weight F')
expectWithin('add lead', moreFrictionRead, leadedRead, 'friction_weight', 0.05)

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

// Regulation from targets reaches the engine, and so does its refusal. On this
// frame the front rail pin bottoms in its mortise about 0.8 mm short of a
// 12 mm dip: the bench must refuse it, say so in millimetres and name the pin,
// and change nothing. A dip the pin allows is laid, and the sheet reads what
// the papers give.
const regulation = () =>
  evaluate(`
    (() => {
      const p = document.querySelector('.regulated')
      return p ? { laid: p.classList.contains('note'), text: p.textContent } : null
    })()
  `)
if ((await setField('key dip', 12)) !== 'ok') fail('no key dip target')
await settle()
if ((await clickButton('regulate to these targets')) !== 'ok') fail('no regulate button')
await settle()
const refused = await regulation()
if (
  !refused ||
  refused.laid ||
  !/mortise/.test(refused.text) ||
  !/\d\.\d\d mm/.test(refused.text)
) {
  fail(`a 12 mm dip was not refused with its shortfall: ${JSON.stringify(refused)}`)
}
const afterRefusal = await sheet()
expectUnchanged('a refused regulation', unlevelled, afterRefusal, 'key dip')
expectUnchanged('a refused regulation', unlevelled, afterRefusal, 'key height')
console.log(`  ok  regulation refused: ${refused.text}`)

if ((await setField('key dip', 10.6)) !== 'ok') fail('no key dip target')
await settle()
if ((await clickButton('regulate to these targets')) !== 'ok') fail('no regulate button')
await settle()
const laid = await regulation()
if (!laid?.laid) fail(`a 10.6 mm dip was not laid: ${JSON.stringify(laid)}`)
const regulated = await sheet()
expectChanged('a regulation laid', afterRefusal, regulated, 'key dip')
console.log(`  ok  regulation laid: ${laid.text}`)

// An emptied target field is no target. Read with `Number`, an empty field is
// 0, and sent it asked for a key height of nothing, which the engine refused
// as 0.00 mm out of reach. The page must not send it: the button waits, the
// page says the field needs a number, and nothing under the button or on the
// sheet changes.
const regulateDisabled = () =>
  evaluate(`
    (() => {
      const b = [...document.querySelectorAll('button')]
        .find((x) => x.textContent.trim() === 'regulate to these targets')
      return b ? b.disabled : null
    })()
  `)
const fieldValue = (label) =>
  evaluate(`
    [...document.querySelectorAll('label.field')]
      .find((x) => x.textContent.includes(${JSON.stringify(label)}))?.querySelector('input')?.value ?? null
  `)
const heightTyped = await fieldValue('key height')
if (heightTyped === null || heightTyped === '') fail('no key height target to empty')
if ((await setField('key height', '')) !== 'ok') fail('no key height target')
await settle()
if ((await regulateDisabled()) !== true) fail('an empty key height can still be sent')
const needs = await evaluate(`document.querySelector('.needs-number')?.textContent ?? null`)
if (needs === null || !/key height needs a number/.test(needs)) {
  fail(`the page does not say the key height needs a number: ${needs}`)
}
await clickButton('regulate to these targets')
await settle()
if (JSON.stringify(await regulation()) !== JSON.stringify(laid)) {
  fail(`an empty key height was sent: ${JSON.stringify(await regulation())}`)
}
const untouched = await sheet()
expectUnchanged('an empty target', regulated, untouched, 'key height')
expectUnchanged('an empty target', regulated, untouched, 'key dip')
console.log(`  ok  an empty target is not sent: ${needs}`)
if ((await setField('key height', heightTyped)) !== 'ok') fail('no key height target')
await settle()
if ((await regulateDisabled()) !== false) fail('the key height typed back does not free the button')

// The measuring point is a protocol setting every touchweight reads. Moving it
// must change front weight and the key weight ratio by the lever, carry the
// friction torque onto the new arm, and leave key height and dip alone: they
// are read at the key front, not at the measuring point.
const regulatedRead = await measured()
if ((await moveSlider('measuring point, back from', 20)) !== 'ok') {
  fail('measuring point slider missing')
}
await settle()
const remeasured = await sheet()
const remeasuredRead = await measured()
expectMoved('measuring point', regulatedRead, remeasuredRead, 'front_weight')
expectMoved('measuring point', regulatedRead, remeasuredRead, 'key_weight_ratio')
expectMoved('measuring point', regulatedRead, remeasuredRead, 'friction_weight')
expectUnchanged('measuring point', regulated, remeasured, 'key height')
expectUnchanged('measuring point', regulated, remeasured, 'key dip')
await expectFrictionShown(
  'the measuring point moved',
  remeasuredRead.key_friction_weight_zeroed_on_the_jig?.exact,
)

// The contacts the engine reads off the drawn parts reach the view: a weight
// lifting the key puts it on its back rail cloth, a weight pressing it puts it
// on its front punching, and the view names what it bears on. The engine's
// identifiers, read off the view's element, not off a label.
const touching = async () => {
  const names = await evaluate(`document.querySelector('.view')?.dataset.touching ?? null`)
  return names === null ? null : names.split(' ').filter(Boolean)
}
for (const [weight, contact] of [
  [-60, 'back_rail_cloth'],
  [60, 'front_rail_punching'],
]) {
  if ((await moveSlider('weight on the measuring point', weight)) !== 'ok') {
    fail('weight slider missing')
  }
  await settle()
  const bears = await touching()
  if (bears === null || !bears.includes(contact)) {
    fail(`with ${weight} g on the key the view marks it bearing on ${bears}, not ${contact}`)
  }
  console.log(`  ok  contacts: with ${weight} g the key bears on ${bears.join(', ')}`)
}

// A weight between the up weight and the down weight leaves the key where the
// last weight left it, and the bench says that friction holds it there: on its
// punching after a press, on its cloth after a lift. The weight is the sheet's
// balance weight in full, the middle of the band, to the slider's half gram;
// the band is the 14 g of friction set above, either side of it.
const verdict = () =>
  evaluate(`
    (() => {
      const v = document.querySelector('.verdict')
      return v ? { held: v.classList.contains('verdict-held'), text: v.textContent } : null
    })()
  `)
const middle = Math.round(2 * (await measured()).balance_weight?.exact) / 2
if (!Number.isFinite(middle)) fail('the sheet shows no balance weight')
for (const [via, contact] of [
  [60, 'front_rail_punching'],
  [-60, 'back_rail_cloth'],
]) {
  if ((await moveSlider('weight on the measuring point', via)) !== 'ok') {
    fail('weight slider missing')
  }
  await settle()
  if ((await moveSlider('weight on the measuring point', middle)) !== 'ok') {
    fail('weight slider missing')
  }
  await settle()
  const bears = await touching()
  if (bears === null || !bears.includes(contact)) {
    fail(`${middle} g, between U and D, moved the key off its ${contact}: it bears on ${bears}`)
  }
  const said = await verdict()
  if (!said?.held) {
    fail(`with ${middle} g, between U and D, the bench says: ${said?.text ?? 'nothing'}`)
  }
  console.log(`  ok  friction holds it: ${middle} g leaves the key on its ${contact}`)
}

// On a frame whose stops fight the key cannot be seated, so Stanwood's protocol
// cannot be run and no bushings can be found for a key friction weight typed
// there: it is refused with the frame's fault, and the slider goes back to what
// the bushings read, although no sheet can be read to say so. The balance rail
// 6.6 mm low leaves the key riding on its front punching and its back cloth.
const heldRead = await measured()
const balanceRail = await evaluate(`
  [...document.querySelectorAll('label.slider')]
    .find((x) => x.textContent.includes('balance rail'))?.querySelector('input')?.value ?? null
`)
if (balanceRail === null) fail('balance rail slider missing')
if ((await moveSlider('balance rail', 20)) !== 'ok') fail('balance rail slider missing')
await settle()
if ((await moveSlider('key friction weight', 9)) !== 'ok') fail('friction slider missing')
await settle()
const faultRefusal = await evaluate(
  `document.querySelector('.friction-refused')?.textContent ?? null`,
)
if (faultRefusal === null || !/cannot read a key friction weight/.test(faultRefusal)) {
  fail(`9 g typed on a frame whose stops fight was not refused with why: ${faultRefusal}`)
}
await expectFrictionShown(
  '9 g refused on a frame whose stops fight',
  heldRead.key_friction_weight_zeroed_on_the_jig?.exact,
)
console.log(`  ok  refused on a frame whose stops fight: ${faultRefusal}`)
if ((await moveSlider('balance rail', balanceRail)) !== 'ok') fail('balance rail slider missing')
await settle()
// Put back by its slider in millimetres, the rail may stand a last bit away
// from where it stood, so the readings are held to a thousandth of a gram: 9 g
// set in silence would move them by grams.
const backOnItsRail = await measured()
for (const key of ['key_friction_weight_zeroed_on_the_jig', 'friction_weight']) {
  if (!(Math.abs(backOnItsRail[key]?.exact - heldRead[key]?.exact) <= 1e-3)) {
    fail(`the balance rail put back, ${key} reads ${JSON.stringify(backOnItsRail[key])}`)
  }
}

// The section draws every part flat, each in its own body's frame, a cut as a
// void in the key's wood. It is a different drawing from the solid's, and a
// renderer that throws there blanks that view alone, so both are shown.
if ((await clickButton('section')) !== 'ok') fail('no section button')
await settle()
if ((await clickButton('solid')) !== 'ok') fail('no solid button')
await settle()
console.log('  ok  both views drawn')

if (thrown.length > 0) fail(`the page threw: ${thrown[0].split('\n')[0]}`)

console.log('all interactions reached the engine')
ws.close()
