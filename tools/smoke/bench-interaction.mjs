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
 * It drives the page by the engine's identifiers, never by its wording: a
 * control by the path it sets (`data-path`) or the action it runs
 * (`data-action`), a reading by its procedure (`data-procedure`), a message by
 * its code. The wording is the catalogue's and changes with the language; the
 * identifiers do not, so the same test drives the bench in English and in
 * French.
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

import { EDGE } from '../../apps/bench/src/palette.ts'

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

/**
 * The whole weighing sheet as the trade reads it: each reading by its
 * procedure, what it reads as, and each quantity reported beside the sheet by
 * its name, its value; `refused` where it is.
 */
const sheet = () =>
  evaluate(`
    JSON.stringify(Object.fromEntries([
      ...[...document.querySelectorAll('.row[data-procedure]')].map(
        (r) => [r.dataset.procedure, r.dataset.shown ?? 'refused']),
      ...[...document.querySelectorAll('.row[data-quantity]')].map(
        (r) => [r.dataset.quantity, r.dataset.value ?? 'refused']),
    ]))
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

/** Types `value` into `input` as a hand does: the input event React reads. */
const typeInto = (selector, value) =>
  evaluate(`
    (() => {
      const input = document.querySelector(${JSON.stringify(selector)})
      if (!input) return 'no such input'
      const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set
      set.call(input, ${JSON.stringify(String(value))})
      input.dispatchEvent(new Event('input', { bubbles: true }))
      return 'ok'
    })()
  `)

/** Moves the slider over the setting at `path`. */
const moveSlider = (path, value) =>
  typeInto(`label.slider[data-path="${path}"] input[type=range]`, value)

/** Where the slider over the setting at `path` stands, and the value the
 * settings hold there, which it shows. */
const slider = (path) =>
  evaluate(`
    (() => {
      const l = document.querySelector('label.slider[data-path="${path}"]')
      return l ? { at: l.querySelector('input').value, holds: l.dataset.value } : null
    })()
  `)

/** What the field standing for the setting at `path` holds, as typed. */
const fieldValue = (path) =>
  evaluate(`document.querySelector('label.field[data-path="${path}"] input')?.value ?? null`)

/** Types a number into the field standing for `path`, as a technician types a
 * target. */
const setField = (path, value) => typeInto(`label.field[data-path="${path}"] input`, value)

/** Clicks the button that runs `action`. */
const clickAction = (action) =>
  evaluate(`
    (() => {
      const b = document.querySelector('button[data-action="${action}"]')
      if (!b) return 'no such button'
      b.click()
      return 'ok'
    })()
  `)

/** Whether the button that runs `action` waits. */
const actionDisabled = (action) =>
  evaluate(`document.querySelector('button[data-action="${action}"]')?.disabled ?? null`)

/** Shows the section or the solid. */
const showView = (view) =>
  evaluate(`
    (() => {
      const b = document.querySelector('[data-view="${view}"]')
      if (!b) return 'no such view'
      b.click()
      return 'ok'
    })()
  `)

/**
 * Clicks inside the punching stack whose papers are at `path`: a paper of
 * `add` millimetres from its drawer, or its `first` or `last` paper lifted
 * off.
 *
 * The scoping matters: a paper is lifted off *both* stacks by the same action,
 * so an unscoped click takes a paper off whichever pile the DOM happens to list
 * first — a different thickness, on a different rail. The failure then looks
 * like an engine bug and is a test bug.
 */
const clickInStack = (path, { add, lift }) =>
  evaluate(`
    (() => {
      const stack = document.querySelector('.stack[data-path="${path}"]')
      if (!stack) return 'no such stack'
      const b = ${JSON.stringify(add ?? null)} !== null
        ? [...stack.querySelectorAll('button[data-action="add_paper"]')]
            .find((x) => Math.abs(Number(x.dataset.thicknessMm) - ${Number(add)}) < 1e-9)
        : [...stack.querySelectorAll('button[data-action="remove_paper"]')]
            .at(${JSON.stringify(lift)} === 'last' ? -1 : 0)
      if (!b) return 'no such button in that stack'
      b.click()
      return 'ok'
    })()
  `)

/** The paths and actions this test drives, by the engine's names. */
const KF = 'set_friction_from_key_friction_weight'
const MASS = 'key.weighing.mass_g'
const FRONT_WEIGHT = 'key.weighing.front_weight_g'
const FROM_LIP = 'key.weighing.from_lip_mm'
const BALANCE_PAPERS = 'frame.balance_punching.papers_mm'
const KEY_HEIGHT = 'regulation.key_height_mm'
const KEY_DIP = 'regulation.key_dip_mm'
const MEASURING_POINT = 'protocol.measuring_point_from_lip_mm'
const WEIGHT = 'experiment.weight_g'
const BALANCE_RAIL = 'frame.balance_rail.height_mm'
const FRONT_EDGE = 'key.front_edge_mm'

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

// Every weight is read the way gram weights give it: the down weight and the
// key friction weights as the least whole gram that does it, the up weight as
// the most the key lifts. None of these stands within the walk's bracket of a
// whole gram, a part in 65 536 of it, which would read as that gram. Both key
// friction weights are on the sheet, each with its label.
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
for (const procedure of [
  'key_friction_weight_zeroed_on_the_jig',
  'key_friction_weight_zeroed_on_the_frame',
]) {
  const label = await evaluate(
    `document.querySelector('.row[data-procedure="${procedure}"] .label')?.textContent ?? ''`,
  )
  if (!start[procedure] || !label) fail(`the sheet has no labelled row ${procedure}`)
}

// The friction control shows what Stanwood's protocol reads the bushings as,
// the sheet's own reading: as the trade reads it and in full, at boot, after a
// weight is typed or refused, and after anything that changes what the same
// bushings read. It is the key friction weight the owner types, so a number
// there the sheet does not read is a setting the model does not hold. Where
// the protocol cannot be read it shows no number, says why by the refusal's
// code, and cannot be moved.
const frictionControl = () =>
  evaluate(`
    (() => {
      const l = document.querySelector('label[data-action="${KF}"]')
      if (!l) return null
      const number = (v) => (v === undefined ? null : Number(v))
      return {
        shown: number(l.dataset.shown),
        exact: number(l.dataset.exact),
        refusal: l.dataset.refusal ?? null,
        disabled: l.querySelector('input').disabled,
        text: l.querySelector('span')?.textContent ?? '',
      }
    })()
  `)
const expectFrictionShown = async (what, reading) => {
  const c = await frictionControl()
  if (
    !c ||
    !Number.isFinite(reading?.exact) ||
    c.shown !== reading.shown ||
    c.exact !== reading.exact ||
    c.disabled
  ) {
    fail(
      `${what}: the friction control shows ${JSON.stringify(c)}, not the reading ${JSON.stringify(reading)} of Stanwood's protocol`,
    )
  }
  console.log(`  ok  the friction control shows the sheet's reading, ${what}: ${c.text}`)
}
await expectFrictionShown('at boot', startRead.key_friction_weight_zeroed_on_the_jig)

// The non-confusion test, run through the whole chain rather than in a unit
// test: more friction must widen the band and leave its centre alone. D is
// read at the top seat and U at the bottom, where the measuring point's arm
// differs by under one per cent, so 8 g more friction moves the balance
// weight in full by under 8 × 1 % / 2 = 0.04 g, and not the gram it is read
// to.
const moveFriction = (g) => typeInto(`label[data-action="${KF}"] input[type=range]`, g)
if ((await moveFriction(14)) !== 'ok') fail('friction slider missing')
await settle()
const moreFriction = await sheet()
const moreFrictionRead = await measured()
expectChanged('friction slider', start, moreFriction, 'friction_weight')
expectUnchanged('friction slider', start, moreFriction, 'balance_weight')
expectWithin('friction slider', startRead, moreFrictionRead, 'balance_weight', 0.05)

// The key friction weight is typed as Stanwood's protocol reads it, and reads
// back as typed: the engine sets the bushings to the torque at which the
// protocol, run on the model, reads the typed grams in full, to the walk's
// resolution, half a bracket of 1/65 536 g, under 8e-6 g. The value in full
// crosses the boundary in full; so it stands within 1e-5 g of 14, and 14 g
// shows as 14 g.
const typedBack = moreFrictionRead.key_friction_weight_zeroed_on_the_jig
if (typedBack?.shown !== 14 || !(Math.abs(typedBack.exact - 14) <= 1e-5)) {
  fail(
    `14 g of key friction weight typed does not read back as typed: ${JSON.stringify(typedBack)}`,
  )
}
console.log(`  ok  typed back: 14 g reads ${typedBack.exact} g, shown ${typedBack.shown} g`)
await expectFrictionShown('14 g typed', typedBack)

// A key friction weight no bushings read is refused with its reason, and the
// bushings stay as they were: with no friction at all this key reads about a
// third of a gram by Stanwood's protocol, what its tilt at rest leaves of its
// own balance and the temporary weight's, so 0 g is out of reach.
if ((await moveFriction(0)) !== 'ok') fail('friction slider missing')
await settle()
// The engine refuses in codes and the page words them: the refusal's code is
// on the message, the message stands right after the control that made the
// edit, and it says something.
const refusalShown = (selector) =>
  evaluate(`
    (() => {
      const p = document.querySelector(${JSON.stringify(selector)})
      const before = p?.previousElementSibling
      return p
        ? {
            code: p.dataset.code ?? null,
            why: p.dataset.why ?? null,
            byMm: p.dataset.byMm === undefined ? null : Number(p.dataset.byMm),
            after: before?.dataset.path ?? before?.dataset.action ?? before?.dataset.form ?? null,
            text: p.textContent,
          }
        : null
    })()
  `)
const frictionRefusal = await refusalShown(`.edit-refused[data-for-action="${KF}"]`)
if (
  frictionRefusal?.code !== 'under_no_friction' ||
  frictionRefusal.after !== KF ||
  !frictionRefusal.text
) {
  fail(
    `0 g of key friction weight was not refused with its reason: ${JSON.stringify(frictionRefusal)}`,
  )
}
const frictionRefused = frictionRefusal.text
const keptRead = await measured()
for (const key of ['key_friction_weight_zeroed_on_the_jig', 'friction_weight']) {
  if (keptRead[key]?.exact !== moreFrictionRead[key]?.exact) {
    fail(`a refused key friction weight moved ${key}: ${JSON.stringify(keptRead[key])}`)
  }
}
console.log(`  ok  refused: ${frictionRefused}`)
await expectFrictionShown('0 g refused', keptRead.key_friction_weight_zeroed_on_the_jig)

// The key's weighing must reach the engine: the key re-weighed whole, its
// mass as the fields hold it and its front weight typed a gram heavier, read
// where the sheet reads it. The sheet's front weight then reads what was
// typed, to the round-off of turning it into a static moment and back, far
// under 1e-5 g; and the gram moves the balance weight and not the friction
// weight: the measuring point's arm differs between the two seats by under one
// per cent, so the friction weight in full moves by under 1 g × 1 % / 2, well
// within 0.05 g.
const typedFront = Number(await fieldValue(FRONT_WEIGHT))
if (!Number.isFinite(typedFront)) fail('no front weight in the weighing fields')
if ((await setField(FRONT_WEIGHT, typedFront + 1)) !== 'ok') fail('no front weight field')
await settle()
if ((await clickAction('weigh_key')) !== 'ok') fail('no button to weigh the key')
await settle()
const reweighed = await sheet()
const reweighedRead = await measured()
if (!(Math.abs(reweighedRead.front_weight?.exact - (typedFront + 1)) <= 1e-5)) {
  fail(
    `${typedFront + 1} g of front weight typed reads ${JSON.stringify(reweighedRead.front_weight)}`,
  )
}
console.log(`  ok  weighed: ${typedFront + 1} g typed, the sheet reads it back`)
expectChanged('re-weigh', moreFriction, reweighed, 'balance_weight')
expectUnchanged('re-weigh', moreFriction, reweighed, 'friction_weight')
expectWithin('re-weigh', moreFrictionRead, reweighedRead, 'friction_weight', 0.05)

// Every field of the weighing reaches the engine as typed, the key mass and
// where the front weight was read too: the key weighed at 130 g, its 10 g of
// front weight read 30 mm back from the lip. The line under the fields says
// what the engine holds, in numbers on it and in words, the sheet's key mass is
// the one typed, and the sheet reads the front weight at its own measuring
// point, 13 mm back: the same static moment on a longer arm, so lighter than
// the 10 g typed. Then the key is weighed again as re-weighed above, and the
// sheet is that one again.
const weighedLine = () =>
  evaluate(`
    (() => {
      const p = document.querySelector('.note.weighed')
      return p
        ? {
            massG: Number(p.dataset.massG),
            frontWeightG: Number(p.dataset.frontWeightG),
            fromLipMm: Number(p.dataset.fromLipMm),
            text: p.textContent,
          }
        : null
    })()
  `)
const typedMass = await fieldValue(MASS)
const typedFrom = await fieldValue(FROM_LIP)
const weighAs = async (mass, front, from) => {
  for (const [path, value] of [
    [MASS, mass],
    [FRONT_WEIGHT, front],
    [FROM_LIP, from],
  ]) {
    if ((await setField(path, value)) !== 'ok') fail(`no ${path} field`)
  }
  await settle()
  if ((await clickAction('weigh_key')) !== 'ok') fail('no button to weigh the key')
  await settle()
}
await weighAs(130, 10, 30)
const heldAs = await weighedLine()
if (
  heldAs?.massG !== 130 ||
  heldAs.frontWeightG !== 10 ||
  heldAs.fromLipMm !== 30 ||
  !heldAs.text
) {
  fail(
    `130 g, and 10 g of front weight read 30 mm back, typed; the engine holds: ${JSON.stringify(heldAs)}`,
  )
}
const further = await sheet()
const furtherRead = await measured()
if (Number(further.key_mass) !== 130) {
  fail(`130 g typed, the sheet's key mass reads ${further.key_mass}`)
}
if (!(furtherRead.front_weight?.exact < 10)) {
  fail(`10 g read 30 mm back reads ${JSON.stringify(furtherRead.front_weight)} 13 mm back`)
}
console.log(`  ok  ${heldAs.text} The sheet reads ${furtherRead.front_weight.exact} g 13 mm back`)
await weighAs(typedMass, typedFront + 1, typedFrom)
if (JSON.stringify(await measured()) !== JSON.stringify(reweighedRead)) {
  fail(`weighed again as before, the sheet reads ${JSON.stringify(await measured())}`)
}
console.log('  ok  weighed again as before: the same sheet')

// A key weighed at no mass is refused before any solve: the problem is said
// right after the weighing's fields, by its code, and the key stays weighed as
// it was, its line and its sheet to the bit. The field keeps what was typed
// until it is typed again.
const weighedBefore = await weighedLine()
if ((await setField(MASS, 0)) !== 'ok') fail(`no ${MASS} field`)
await settle()
if ((await clickAction('weigh_key')) !== 'ok') fail('no button to weigh the key')
await settle()
const massRefusal = await refusalShown('.edit-refused[data-for-action="weigh_key"]')
if (
  massRefusal?.code !== 'mass_not_positive' ||
  massRefusal.after !== 'weigh_key' ||
  !massRefusal.text
) {
  fail(`a key weighed at 0 g is not refused after its fields: ${JSON.stringify(massRefusal)}`)
}
if (JSON.stringify(await weighedLine()) !== JSON.stringify(weighedBefore)) {
  fail(
    `a refused weighing changed what the key is weighed as: ${JSON.stringify(await weighedLine())}`,
  )
}
if (JSON.stringify(await measured()) !== JSON.stringify(reweighedRead)) {
  fail(`a refused weighing changed the sheet: ${JSON.stringify(await measured())}`)
}
if ((await fieldValue(MASS)) !== '0') fail('the refused key mass was not left in its field')
console.log(`  ok  ${massRefusal.text}`)
if ((await setField(MASS, typedMass)) !== 'ok') fail(`no ${MASS} field`)
await settle()

// Regulation must reach the engine too. A punching stack is a pile of papers,
// so the operation is a button in the drawer and not a slider.
if ((await clickInStack(BALANCE_PAPERS, { add: 0.2 })) !== 'ok') {
  fail('no 0.20 paper in the drawer')
}
await settle()
const levelled = await sheet()
expectChanged('lay a paper', reweighed, levelled, 'key_height')

// And lifting *that* paper back off must restore the key exactly. A regulation
// that is not reversible is not a regulation.
//
// The `last` matters: the paper laid is on top of the pile, and lifting the
// first takes the balance stack's bottom one, a different thickness. The wrong click
// reads 0.49 mm low — exactly 0.20 added minus 0.40 removed through the 2.47
// back-rail lever — and that is a test bug, not an engine bug.
if ((await clickInStack(BALANCE_PAPERS, { lift: 'last' })) !== 'ok') {
  fail('no way to lift a paper off')
}
await settle()
const unlevelled = await sheet()
if (unlevelled.key_height !== reweighed.key_height) {
  fail(
    `lifting the paper off left key height at ${unlevelled.key_height}, was ${reweighed.key_height}`,
  )
}
console.log(`  ok  lift it off: key height back to ${unlevelled.key_height}`)

// Regulation from targets reaches the engine, and so does its refusal. On this
// frame the front rail pin bottoms in its mortise about 0.8 mm short of a
// 12 mm dip: the bench must refuse it, say so in millimetres and name the pin,
// and change nothing. A dip the pin allows is laid, and the sheet reads what
// the papers give.
const regulation = () =>
  evaluate(`
    (() => {
      const p = document.querySelector('[data-for-action="regulate"][data-outcome]')
      return p
        ? {
            laid: p.classList.contains('note'),
            outcome: p.dataset.outcome ?? null,
            code: p.dataset.code ?? null,
            limitedBy: p.dataset.limitedBy ?? null,
            shortfallMm: p.dataset.shortfallMm === undefined ? null : Number(p.dataset.shortfallMm),
            text: p.textContent,
          }
        : null
    })()
  `)
if ((await setField(KEY_DIP, 12)) !== 'ok') fail('no key dip target')
await settle()
if ((await clickAction('regulate')) !== 'ok') fail('no regulate button')
await settle()
const refused = await regulation()
if (
  !refused ||
  refused.laid ||
  refused.outcome !== 'refused' ||
  refused.code !== 'regulation_infeasible' ||
  refused.limitedBy !== 'front_pin_in_mortise' ||
  !(refused.shortfallMm > 0.7 && refused.shortfallMm < 0.9) ||
  !refused.text
) {
  fail(`a 12 mm dip was not refused with its shortfall: ${JSON.stringify(refused)}`)
}
const afterRefusal = await sheet()
expectUnchanged('a refused regulation', unlevelled, afterRefusal, 'key_dip')
expectUnchanged('a refused regulation', unlevelled, afterRefusal, 'key_height')
console.log(`  ok  regulation refused: ${refused.text}`)

if ((await setField(KEY_DIP, 10.6)) !== 'ok') fail('no key dip target')
await settle()
if ((await clickAction('regulate')) !== 'ok') fail('no regulate button')
await settle()
const laid = await regulation()
if (!laid?.laid || laid.outcome !== 'laid') {
  fail(`a 10.6 mm dip was not laid: ${JSON.stringify(laid)}`)
}
const regulated = await sheet()
expectChanged('a regulation laid', afterRefusal, regulated, 'key_dip')
console.log(`  ok  regulation laid: ${laid.text}`)

// An emptied target field is no target. Read with `Number`, an empty field is
// 0, and sent it asked for a key height of nothing, which the engine refused
// as 0.00 mm out of reach. The page must not send it: the button waits, the
// page says the field needs a number, and nothing under the button or on the
// sheet changes.
const heightTyped = await fieldValue(KEY_HEIGHT)
if (heightTyped === null || heightTyped === '') fail('no key height target to empty')
if ((await setField(KEY_HEIGHT, '')) !== 'ok') fail('no key height target')
await settle()
if ((await actionDisabled('regulate')) !== true) fail('an empty key height can still be sent')
const needsShown = await evaluate(`
  (() => {
    const p = document.querySelector('.needs-number[data-for-action="regulate"]')
    return p ? { needs: p.dataset.needs, text: p.textContent } : null
  })()
`)
if (needsShown?.needs !== 'key_height_mm' || !needsShown.text) {
  fail(`the page does not say the key height needs a number: ${JSON.stringify(needsShown)}`)
}
const needs = needsShown.text
await clickAction('regulate')
await settle()
if (JSON.stringify(await regulation()) !== JSON.stringify(laid)) {
  fail(`an empty key height was sent: ${JSON.stringify(await regulation())}`)
}
const untouched = await sheet()
expectUnchanged('an empty target', regulated, untouched, 'key_height')
expectUnchanged('an empty target', regulated, untouched, 'key_dip')
console.log(`  ok  an empty target is not sent: ${needs}`)
if ((await setField(KEY_HEIGHT, heightTyped)) !== 'ok') fail('no key height target')
await settle()
if ((await actionDisabled('regulate')) !== false) {
  fail('the key height typed back does not free the button')
}

// The measuring point is a protocol setting every touchweight reads. Moving it
// must change front weight and the key weight ratio by the lever, carry the
// friction torque onto the new arm, and leave key height and dip alone: they
// are read at the key front, not at the measuring point.
const regulatedRead = await measured()
if ((await moveSlider(MEASURING_POINT, 20)) !== 'ok') {
  fail('measuring point slider missing')
}
await settle()
const remeasured = await sheet()
const remeasuredRead = await measured()
expectMoved('measuring point', regulatedRead, remeasuredRead, 'front_weight')
expectMoved('measuring point', regulatedRead, remeasuredRead, 'key_weight_ratio')
expectMoved('measuring point', regulatedRead, remeasuredRead, 'friction_weight')
expectUnchanged('measuring point', regulated, remeasured, 'key_height')
expectUnchanged('measuring point', regulated, remeasured, 'key_dip')
await expectFrictionShown(
  'the measuring point moved',
  remeasuredRead.key_friction_weight_zeroed_on_the_jig,
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
  if ((await moveSlider(WEIGHT, weight)) !== 'ok') {
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
      return v
        ? { held: v.classList.contains('verdict-held'), code: v.dataset.verdict, text: v.textContent }
        : null
    })()
  `)
const middle = Math.round(2 * (await measured()).balance_weight?.exact) / 2
if (!Number.isFinite(middle)) fail('the sheet shows no balance weight')
for (const [via, contact] of [
  [60, 'front_rail_punching'],
  [-60, 'back_rail_cloth'],
]) {
  if ((await moveSlider(WEIGHT, via)) !== 'ok') {
    fail('weight slider missing')
  }
  await settle()
  if ((await moveSlider(WEIGHT, middle)) !== 'ok') {
    fail('weight slider missing')
  }
  await settle()
  const bears = await touching()
  if (bears === null || !bears.includes(contact)) {
    fail(`${middle} g, between U and D, moved the key off its ${contact}: it bears on ${bears}`)
  }
  const said = await verdict()
  if (!said?.held || said.code !== 'held_by_friction') {
    fail(`with ${middle} g, between U and D, the bench says: ${said?.text ?? 'nothing'}`)
  }
  console.log(`  ok  friction holds it: ${middle} g leaves the key on its ${contact}`)
}

// On a frame whose stops fight the key rides on them, off its balance punching:
// the balance rail 6.6 mm low leaves the key on its front stop and its back
// cloth. The view marks it bearing on those two and not on its balance
// punching, the page says it rides, naming the front stop and the lift, the
// key height is read and the dip is nothing, and every weight that needs a
// stroke is refused with that reason: the key has no stroke between two seats.
const heldRead = await measured()
const balanceRail = (await slider(BALANCE_RAIL))?.at ?? null
if (balanceRail === null) fail('balance rail slider missing')
if ((await moveSlider(BALANCE_RAIL, 20)) !== 'ok') fail('balance rail slider missing')
await settle()
const ridingShown = await evaluate(`
  (() => {
    const p = document.querySelector('.riding')
    return p ? { front: p.dataset.front, liftMm: Number(p.dataset.liftMm), text: p.textContent } : null
  })()
`)
if (
  ridingShown?.front !== 'front_rail_punching' ||
  !(ridingShown.liftMm > 0) ||
  !ridingShown.text
) {
  fail(`the key riding on its stops is not said: ${JSON.stringify(ridingShown)}`)
}
const riding = ridingShown.text
const ridingOn = await touching()
if (
  ridingOn === null ||
  !ridingOn.includes('back_rail_cloth') ||
  ridingOn.includes('balance_rail_punching')
) {
  fail(`the view marks the riding key bearing on ${ridingOn}`)
}
const ridingRead = await measured()
if (!Number.isFinite(ridingRead.key_height?.exact) || ridingRead.key_dip?.exact !== 0) {
  fail(
    `the riding key's height and dip: ${JSON.stringify([ridingRead.key_height, ridingRead.key_dip])}`,
  )
}
const noStroke = await evaluate(`
  (() => {
    const row = document.querySelector('[data-procedure="down_weight"]')
    return row
      ? {
          refused: row.classList.contains('refused'),
          code: row.dataset.refusal,
          riding: row.dataset.riding,
          why: row.title,
        }
      : null
  })()
`)
if (
  !noStroke?.refused ||
  noStroke.code !== 'no_travel' ||
  noStroke.riding !== ridingShown.front ||
  !noStroke.why
) {
  fail(`the riding key's down weight is not refused with why: ${JSON.stringify(noStroke)}`)
}
console.log(`  ok  rides on ${ridingOn.join(', ')}: ${riding}`)

// Stanwood's protocol then has no stroke to walk, so the bushings have no key
// friction weight to show and none can be set: the control shows none, says
// why with the sheet's refusal, and cannot be moved, rather than show a
// number the sheet read on another frame.
const faultControl = await frictionControl()
if (
  faultControl?.shown !== null ||
  faultControl.refusal !== 'no_travel' ||
  !faultControl.disabled
) {
  fail(`the friction control on a frame whose stops fight: ${JSON.stringify(faultControl)}`)
}
console.log(`  ok  no key friction weight on a frame whose stops fight: ${faultControl.text}`)
if ((await moveSlider(BALANCE_RAIL, balanceRail)) !== 'ok') fail('balance rail slider missing')
await settle()
// Put back by its slider in millimetres, the rail may stand a last bit away
// from where it stood, so the readings are held to a thousandth of a gram:
// bushings changed in silence would move them by grams. The control reads
// the bushings again, as the sheet does.
const backOnItsRail = await measured()
for (const key of ['key_friction_weight_zeroed_on_the_jig', 'friction_weight']) {
  if (!(Math.abs(backOnItsRail[key]?.exact - heldRead[key]?.exact) <= 1e-3)) {
    fail(`the balance rail put back, ${key} reads ${JSON.stringify(backOnItsRail[key])}`)
  }
}
await expectFrictionShown(
  'the balance rail put back',
  backOnItsRail.key_friction_weight_zeroed_on_the_jig,
)

// A setting no key could be built with is refused and changes nothing: the key
// front pushed back into the capsule, 170 mm from the balance pin, would make
// the key button and the keytop overlap along the key, by the 14 mm it falls
// short of the 184 mm they need. The problem is said right after the slider
// that made the edit, by its code and the overlap in millimetres; the slider
// goes back to where the setting stands, and every reading of the sheet stays
// as it was, to the bit. The next edit taken clears it.
const edgeBefore = await slider(FRONT_EDGE)
if (edgeBefore === null) fail('front edge slider missing')
const beforeEdge = await measured()
if ((await moveSlider(FRONT_EDGE, 170)) !== 'ok') fail('front edge slider missing')
await settle()
const edgeRefused = await refusalShown(`.edit-refused[data-for-path="${FRONT_EDGE}"]`)
if (
  edgeRefused?.code !== 'outline_crosses_itself' ||
  !(Math.abs(edgeRefused.byMm - 14) < 0.05) ||
  edgeRefused.after !== FRONT_EDGE ||
  !edgeRefused.text
) {
  fail(
    `a key front pushed into the capsule is not refused beside its slider: ${JSON.stringify(edgeRefused)}`,
  )
}
const edgeAfter = await slider(FRONT_EDGE)
if (edgeAfter?.at !== edgeBefore.at || edgeAfter.holds !== edgeBefore.holds) {
  fail(`the refused front edge moved its slider: ${JSON.stringify([edgeBefore, edgeAfter])}`)
}
if (JSON.stringify(await measured()) !== JSON.stringify(beforeEdge)) {
  fail(`a refused front edge changed the sheet: ${JSON.stringify(await measured())}`)
}
console.log(`  ok  refused beside its slider, at ${edgeAfter.at} mm still: ${edgeRefused.text}`)
console.log('  ok  a refused front edge leaves every reading as it was')
if ((await moveSlider(WEIGHT, 0)) !== 'ok') fail('weight slider missing')
await settle()
if ((await refusalShown('.edit-refused')) !== null) {
  fail('an edit taken leaves the refused front edge said')
}
console.log('  ok  the next edit taken clears it')

// What is on the bench is one of the profiles the engine ships, chosen by the
// action that puts it there, as it arrives, each offered by its whole name,
// which says where its numbers come from. Hirschkorn's key comes with its
// mass published whole, so its front weight is Table A.1's static moment over
// the measuring arm, 120.08 g at 51.4 mm over the traced lip's 239.28 mm less
// 13 mm, 27.28 g, read to the tenth as 27.3 g; his key mass is Table A.1's;
// and the bench says that the borrowed 24 mm cloth strip reaches in front of
// his key's chamfer. Every line drawn can be marked by how the number that
// places it is known, and what his profile reads of the owner's key frame,
// its pins and punchings, is marked borrowed, which nothing on the composite
// is. The composite chosen again is the instrument the page booted on: the
// same sheet, to the last bit.
const choose = (profile) =>
  evaluate(`
    (() => {
      const radio = document.querySelector(
        '[data-action="select_profile"] input[value=${JSON.stringify(profile)}]')
      if (!radio) return 'no profile choice'
      radio.click()
      return 'ok'
    })()
  `)
const onTheBench = () =>
  evaluate(`document.querySelector('[data-action="select_profile"]')?.dataset.profile ?? null`)
/** Each profile's name as the page lays it out: wholly inside the panel, and
 * nothing of it cut, as a drop-down list cuts a long name at its edge. */
const namesCut = await evaluate(`
  (() => {
    const panel = document.querySelector('.panel')?.getBoundingClientRect()
    const options = [...document.querySelectorAll('[data-action="select_profile"] [data-option]')]
    if (!panel || options.length < 2) return 'the profiles are not offered one by one'
    for (const o of options) {
      const box = o.getBoundingClientRect()
      const name = o.querySelector('span')
      if (box.width === 0 || box.left < panel.left || box.right > panel.right) {
        return o.dataset.option + ' is laid out outside the panel'
      }
      if (!name || name.scrollWidth > name.clientWidth || name.scrollHeight > name.clientHeight) {
        return o.dataset.option + ' is cut'
      }
    }
    return null
  })()
`)
if (namesCut !== null) fail(`a profile's name is not shown whole: ${namesCut}`)
console.log('  ok  every profile offered by its whole name')
if ((await choose('hirschkorn')) !== 'ok') fail('no profile choice')
await settle()
if ((await onTheBench()) !== 'hirschkorn')
  fail(`Hirschkorn's key not on the bench: ${await onTheBench()}`)
const his = await measured()
const tableA1 = (120.08 * 51.4) / (239.28 - 13)
if (his.front_weight?.shown !== 27.3 || !(Math.abs(his.front_weight.exact - tableA1) < 0.001)) {
  fail(
    `his front weight is not Table A.1's: ${JSON.stringify(his.front_weight)} against ${tableA1}`,
  )
}
const hisMass = (await sheet()).key_mass
if (!(Math.abs(Number(hisMass) - 120.08) < 1e-9))
  fail(`his key mass is not Table A.1's: ${hisMass}`)
const clothFault = await evaluate(
  `document.querySelector('.faults li[data-code="cloth_in_front_of_chamfer"]')?.textContent ?? null`,
)
if (!clothFault) fail('the cloth in front of his chamfer is not said')
console.log(
  `  ok  Hirschkorn's key on the bench: FW ${his.front_weight.exact} g, ${hisMass} g, ${clothFault}`,
)
// The marks are lines the view paints, in the section and on the solid's faces
// alike, two drawings apart: a toggle whose attribute and legend change while
// the lines stay as they were reaches nothing. So each view is captured below
// its buttons: marked, it differs above the legend from the same view
// unmarked, and unmarked it is drawn again pixel for pixel as it was before
// it was marked, where the legend stood included, the scene and the camera
// being the same.
const mark = () =>
  evaluate(`
    (() => {
      const b = document.querySelector('button[data-mark="provenance"]')
      if (!b) return 'no provenance toggle'
      b.click()
      return 'ok'
    })()
  `)
const markedState = () =>
  evaluate(`
    JSON.stringify({
      view: document.querySelector('.view')?.dataset.marked ?? null,
      legend: [...document.querySelectorAll('.provenance-legend [data-how]')].map((e) => e.dataset.how),
    })
  `).then(JSON.parse)
/** The view's drawing below its buttons, down to the legend's top where it
 * stands, else to the drawing's foot. */
const drawnArea = () =>
  evaluate(`
    (() => {
      const canvas = document.querySelector('.view canvas')?.getBoundingClientRect()
      const tabs = document.querySelector('.viewtabs')?.getBoundingClientRect()
      if (!canvas || !tabs) return null
      const legend = document.querySelector('.provenance-legend')?.getBoundingClientRect()
      const top = Math.ceil(tabs.bottom) + 1
      const bottom = Math.floor(legend ? legend.top : canvas.bottom) - 1
      return JSON.stringify({
        x: Math.ceil(canvas.left),
        y: top,
        width: Math.floor(canvas.width),
        height: bottom - top,
      })
    })()
  `).then((clip) => (clip ? JSON.parse(clip) : null))
const drawn = async (clip) =>
  (await send('Page.captureScreenshot', { format: 'png', clip: { ...clip, scale: 1 } })).result
    ?.data
let marked = null
for (const v of ['section', 'solid']) {
  if ((await showView(v)) !== 'ok') fail(`no ${v} button`)
  await settle()
  const whole = await drawnArea()
  if (!whole || whole.height < 50) {
    fail(`no drawing to capture in the ${v}: ${JSON.stringify(whole)}`)
  }
  const before = await drawn(whole)
  if ((await mark()) !== 'ok') fail('no provenance toggle')
  await settle()
  marked = await markedState()
  if (marked.view !== 'provenance' || marked.legend.length !== 7) {
    fail(`the lines are not marked by provenance in the ${v}: ${JSON.stringify(marked)}`)
  }
  const aboveLegend = await drawnArea()
  if (!aboveLegend || aboveLegend.height < 50) {
    fail(`no drawing above the legend in the ${v}: ${JSON.stringify(aboveLegend)}`)
  }
  const withMarks = await drawn(aboveLegend)
  if ((await mark()) !== 'ok') fail('no provenance toggle')
  await settle()
  if ((await markedState()).view !== null) fail(`the ${v} stays marked once unmarked`)
  if ((await evaluate(`document.querySelector('.provenance-legend') === null`)) !== true) {
    fail(`the provenance legend stays once unmarked in the ${v}`)
  }
  if (!withMarks || withMarks === (await drawn(aboveLegend))) {
    fail(`marking the lines changes nothing drawn in the ${v}`)
  }
  if (!before || (await drawn(whole)) !== before) {
    fail(`the ${v} unmarked is not drawn as it was before it was marked`)
  }
  console.log(`  ok  the ${v} drawn marked, and unmarked as it was before`)
}
console.log(`  ok  every line marked by how its number is known: ${marked.legend.join(', ')}`)
if ((await choose('composite')) !== 'ok') fail('no profile choice')
await settle()
if (JSON.stringify(await measured()) !== JSON.stringify(startRead)) {
  fail(
    `the composite chosen again is not the one the page booted on: ${JSON.stringify(await measured())}`,
  )
}
console.log('  ok  the composite chosen again reads the sheet the page booted on')

// The section draws every part flat, each in its own body's frame, a cut as a
// void in the key's wood. It is a different drawing from the solid's, and a
// renderer that throws there blanks that view alone, so both are shown.
if ((await showView('section')) !== 'ok') fail('no section button')
await settle()
if ((await showView('solid')) !== 'ok') fail('no solid button')
await settle()
console.log('  ok  both views drawn')

// What a profile reads of another instrument than the one it describes is
// marked borrowed, in a colour of its own: his pins and punchings, read from
// the owner's key frame, which the section draws as it draws every turned
// part; on the composite, which describes that frame, they are measured and
// nothing is borrowed. Counted on the section opened by its link, which
// faces it. Where the marked drawing differs from the unmarked one, a line is
// drawn in a mark's colour where the edge colour was, over the same fill and
// covering as much of the pixel, so the difference is the mark's colour less
// the edge's, scaled by that cover: each such pixel is the mark's whose
// difference from the edge points its way, at a quarter of its length at
// least.
await send('Page.navigate', { url: `http://127.0.0.1:${appPort}/?view=section` })
await waitFor('the engine to boot again on the section', async () => {
  const s = await sheet().catch(() => ({}))
  return Object.keys(s).length > 0 ? s : undefined
})
await settle()
const rgbOf = (hex) => [1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16))
const paintedOn = async (profile) => {
  if ((await choose(profile)) !== 'ok') fail('no profile choice')
  await settle()
  if ((await mark()) !== 'ok') fail('no provenance toggle')
  await settle()
  const area = await drawnArea()
  if (!area || area.height < 50) fail(`no drawing to capture: ${JSON.stringify(area)}`)
  const colours = await evaluate(`
    JSON.stringify(Object.fromEntries([...document.querySelectorAll('.provenance-legend [data-how]')]
      .map((e) => [e.dataset.how,
        getComputedStyle(e.querySelector('.swatch')).backgroundColor.match(/\\d+/g).map(Number)])))
  `).then(JSON.parse)
  const withMarks = await drawn(area)
  if ((await mark()) !== 'ok') fail('no provenance toggle')
  await settle()
  const plain = await drawn(area)
  return evaluate(`
    (async () => {
      const pixels = async (png) => {
        const image = await (await fetch('data:image/png;base64,' + png)).blob()
        const bitmap = await createImageBitmap(image)
        const context = new OffscreenCanvas(bitmap.width, bitmap.height).getContext('2d')
        context.drawImage(bitmap, 0, 0)
        return context.getImageData(0, 0, bitmap.width, bitmap.height).data
      }
      const [u, p] = [await pixels('${plain}'), await pixels('${withMarks}')]
      const edge = ${JSON.stringify(rgbOf(EDGE))}
      const ways = ${JSON.stringify(colours)}
      const counts = Object.fromEntries(Object.keys(ways).map((how) => [how, 0]))
      for (let i = 0; i < u.length; i += 4) {
        const d = [p[i] - u[i], p[i + 1] - u[i + 1], p[i + 2] - u[i + 2]]
        const nd = Math.hypot(...d)
        if (nd < 12) continue
        for (const [how, c] of Object.entries(ways)) {
          const e = [c[0] - edge[0], c[1] - edge[1], c[2] - edge[2]]
          const ne = Math.hypot(...e)
          const cos = (d[0] * e[0] + d[1] * e[1] + d[2] * e[2]) / (nd * ne)
          if (cos > 0.97 && nd >= 0.25 * ne) {
            counts[how] += 1
            break
          }
        }
      }
      return JSON.stringify(counts)
    })()
  `).then(JSON.parse)
}
const onHis = await paintedOn('hirschkorn')
const onOwn = await paintedOn('composite')
if (!(onHis.borrowed > 0)) {
  fail(`what his profile borrows is not marked borrowed: ${JSON.stringify(onHis)}`)
}
if (onOwn.borrowed !== 0 || !(onOwn.measured > 0)) {
  fail(`the composite is marked ${JSON.stringify(onOwn)}`)
}
console.log(`  ok  his key's section marked ${JSON.stringify(onHis)}`)
console.log(`  ok  the composite's section marked ${JSON.stringify(onOwn)}`)

// The band, drawn when it is chosen and never with an edit: the key driven
// through its stroke at the measuring point, down and back, the force that
// drives it there at every step, D the greatest of the way down and U the
// least of the way up. On both profiles it is drawn, its two curves inside
// the window the seats leave, and a technician reads D and U on it as on his
// scale: the same grams as the sheet's walks, and in full within a hundredth
// of a gram, the curve's change over the step the sweep reads them from,
// which the engine's tests derive and hold under that hundredth on both
// keys. D falls on the first step off the top seat and U within a step of
// the bottom one. The weight on the measuring point is drawn across it, the
// page's own weight.
const bandShown = () =>
  evaluate(`
    JSON.stringify((() => {
      const band = document.querySelector('.band')
      if (!band) return null
      return {
        band: { ...band.dataset },
        readings: Object.fromEntries([...band.querySelectorAll('.band-reading')].map((g) => [
          g.dataset.procedure,
          { shown: Number(g.dataset.shown), exact: Number(g.dataset.exact), atMm: Number(g.dataset.atMm) },
        ])),
        curves: [...band.querySelectorAll('path[data-curve]')].map((p) => p.dataset.curve),
        touch: { ...(band.querySelector('.band-touch')?.dataset ?? {}) },
      }
    })())
  `).then(JSON.parse)
const BAND_READS = { down_weight_on_the_band: 'down_weight', up_weight_on_the_band: 'up_weight' }
let bandOn = null
for (const profile of ['composite', 'hirschkorn']) {
  if ((await choose(profile)) !== 'ok') fail('no profile choice')
  await settle()
  if ((await showView('band')) !== 'ok') fail('no band button')
  await settle()
  const b = await bandShown()
  const windowMm = Number(b?.band.windowMm)
  const step = Number(b?.band.stepMm)
  if (
    b?.band.band !== 'drawn' ||
    b.band.stale !== 'false' ||
    !(Number(b.band.samples) > 100) ||
    JSON.stringify(b.curves) !== JSON.stringify(['down', 'up'])
  ) {
    fail(`the band is not drawn on ${profile}: ${JSON.stringify(b)}`)
  }
  const read = await measured()
  for (const [onBand, walked] of Object.entries(BAND_READS)) {
    const [m, w] = [b.readings[onBand], read[walked]]
    if (!m || m.shown !== w?.shown || !(Math.abs(m.exact - w.exact) <= 0.01)) {
      fail(`${onBand} on ${profile}'s band is not the sheet's: ${JSON.stringify([m, w])}`)
    }
    console.log(
      `  ok  ${profile}: ${onBand} ${m.shown} g (${m.exact}) at ${m.atMm.toFixed(2)} mm, the sheet's ${w.shown} g (${w.exact})`,
    )
  }
  const [d, u] = [b.readings.down_weight_on_the_band, b.readings.up_weight_on_the_band]
  if (!(Math.abs(d.atMm - step) < 1e-9) || !(windowMm - u.atMm <= step)) {
    fail(`the extremes do not fall a step off each seat: ${JSON.stringify([d, u, windowMm])}`)
  }
  if (Number(b.touch.weightG) !== Number((await slider(WEIGHT))?.holds)) {
    fail(`the weight across the band is not the page's: ${JSON.stringify(b.touch)}`)
  }
  // The band is laid out in the view, every step of it in its table too, and
  // the page stays the window's height, as it is with the key drawn.
  const grown = await evaluate(`
    (() => {
      const page = document.documentElement
      return page.scrollHeight > page.clientHeight ? [page.scrollHeight, page.clientHeight] : null
    })()
  `)
  if (grown) fail(`the band lengthens the page on ${profile}: ${JSON.stringify(grown)}`)
  bandOn = b
}

// The band reads no weight on the measuring point, so moving it leaves the
// band as it was drawn and moves where the key stands on it: a pull past U
// brings Hirschkorn's key back to its top seat, the dot to no travel. A
// change of the key itself leaves the band drawn but marked as the key's
// before, until the band is chosen again.
if ((await moveSlider(WEIGHT, -40)) !== 'ok') fail('weight slider missing')
await settle()
const pulled = await bandShown()
if (pulled.band.stale !== 'false' || Number(pulled.touch.weightG) !== -40) {
  fail(`a weight on the measuring point redrew or staled the band: ${JSON.stringify(pulled)}`)
}
if (!(Math.abs(Number(pulled.touch.travelMm)) < 0.001)) {
  fail(`a pull of 40 g leaves his key ${pulled.touch.travelMm} mm below its top seat`)
}
console.log(`  ok  40 g pulled: the band as drawn, the key at its top seat on it`)

// The non-confusion test through the band: tighter bushings widen it and
// leave its middle, D read a step off the top seat and U a step off the
// bottom, where the measuring point's arm differs by under one per cent, so
// the middle moves by under 8 g × 1 % / 2 = 0.04 g as the sheet's balance
// weight does.
const looser = bandOn.readings
if ((await moveFriction(14)) !== 'ok') fail('friction slider missing')
await settle()
if ((await bandShown()).band.stale !== 'true') fail('a key changed leaves its band unmarked')
if ((await showView('band')) !== 'ok') fail('no band button')
await settle()
const tighter = await bandShown()
if (tighter.band.stale !== 'false') fail('the band chosen again is not drawn again')
const bandMiddle = (r) => (r.down_weight_on_the_band.exact + r.up_weight_on_the_band.exact) / 2
const bandWidth = (r) => r.down_weight_on_the_band.exact - r.up_weight_on_the_band.exact
if (!(bandWidth(tighter.readings) > bandWidth(looser) + 1)) {
  fail(
    `tighter bushings did not widen the band: ${bandWidth(looser)} -> ${bandWidth(tighter.readings)}`,
  )
}
if (!(Math.abs(bandMiddle(tighter.readings) - bandMiddle(looser)) <= 0.05)) {
  fail(
    `tighter bushings moved the band's middle: ${bandMiddle(looser)} -> ${bandMiddle(tighter.readings)}`,
  )
}
console.log(
  `  ok  tighter bushings widen the band ${bandWidth(looser).toFixed(2)} -> ${bandWidth(tighter.readings).toFixed(2)} g, its middle ${bandMiddle(looser).toFixed(3)} -> ${bandMiddle(tighter.readings).toFixed(3)} g`,
)

// The other half of the pair: the key weighed a gram heavier at the front
// moves the band's middle as it moves the sheet's balance weight, and not its
// width. D and U on the band each stand within the hundredth of a gram held
// above of the walk's, so the middle moves as the sheet's balance weight in
// full does to within twice that, 0.02 g. The width is D − U, which the gram
// moves by under 1 g × 1 %, the two seats' arms apart, and each of its four
// readings stands within 0.01 g of the walk's: under 0.05 g in all.
const sheetBefore = await measured()
const frontBefore = Number(await fieldValue(FRONT_WEIGHT))
if ((await setField(FRONT_WEIGHT, frontBefore + 1)) !== 'ok') fail('no front weight field')
await settle()
if ((await clickAction('weigh_key')) !== 'ok') fail('no button to weigh the key')
await settle()
if ((await bandShown()).band.stale !== 'true') fail('a key weighed again leaves its band unmarked')
if ((await showView('band')) !== 'ok') fail('no band button')
await settle()
const heavier = await bandShown()
const sheetAfter = await measured()
const middleMoved = bandMiddle(heavier.readings) - bandMiddle(tighter.readings)
const balanceMoved = sheetAfter.balance_weight.exact - sheetBefore.balance_weight.exact
const widthMoved = bandWidth(heavier.readings) - bandWidth(tighter.readings)
if (!(Math.abs(middleMoved) > 0.5) || !(Math.abs(middleMoved - balanceMoved) <= 0.02)) {
  fail(
    `a gram at the front moved the band's middle by ${middleMoved} g, the sheet's balance weight by ${balanceMoved} g`,
  )
}
if (!(Math.abs(widthMoved) <= 0.05)) {
  fail(`a gram at the front moved the band's width by ${widthMoved} g`)
}
console.log(
  `  ok  a gram at the front moves the band's middle ${middleMoved.toFixed(3)} g (the sheet's BW ${balanceMoved.toFixed(3)} g), its width ${widthMoved.toFixed(4)} g`,
)

// A key with no stroke has no band: on the composite frame with its balance
// rail 6.6 mm low its stops fight, it rides on them, and the band says why
// with the sheet's own refusal.
if ((await choose('composite')) !== 'ok') fail('no profile choice')
await settle()
if ((await moveSlider(BALANCE_RAIL, 20)) !== 'ok') fail('balance rail slider missing')
await settle()
if ((await showView('band')) !== 'ok') fail('no band button')
await settle()
const noBand = await evaluate(`
  (() => {
    const b = document.querySelector('.band')
    return b ? { band: b.dataset.band, refusal: b.dataset.refusal ?? null, text: b.textContent } : null
  })()
`)
if (noBand?.band !== 'refused' || noBand.refusal !== 'no_travel' || !noBand.text) {
  fail(`a key riding on its stops draws a band: ${JSON.stringify(noBand)}`)
}
console.log(`  ok  no band on a key riding on its stops: ${noBand.text}`)

// The frame put back, the key has its stroke again: the refusal stays where
// it stood, marked as the key's before, as a band drawn would be, and the
// band chosen again is drawn whole, its plot laid out where the refusal stood.
if ((await moveSlider(BALANCE_RAIL, balanceRail)) !== 'ok') fail('balance rail slider missing')
await settle()
const refusedBefore = await evaluate(`
  (() => {
    const b = document.querySelector('.band')
    return b ? { band: b.dataset.band, stale: b.dataset.stale ?? null } : null
  })()
`)
if (refusedBefore?.band !== 'refused' || refusedBefore.stale !== 'true') {
  fail(`a refusal for a key since changed is not marked: ${JSON.stringify(refusedBefore)}`)
}
console.log(`  ok  the refusal marked as the key's before, once the frame is put back`)
if ((await showView('band')) !== 'ok') fail('no band button')
await settle()
const again = await bandShown()
const plotted = await evaluate(
  `document.querySelector('.band-plot svg')?.getBoundingClientRect().width ?? 0`,
)
if (
  again?.band.band !== 'drawn' ||
  JSON.stringify(again.curves) !== JSON.stringify(['down', 'up']) ||
  !(plotted > 100)
) {
  fail(`the band is not drawn again after a refusal: ${JSON.stringify([again, plotted])}`)
}
console.log(`  ok  the band drawn again once the key has its stroke back, ${plotted} px wide`)

if (thrown.length > 0) fail(`the page threw: ${thrown[0].split('\n')[0]}`)

console.log('all interactions reached the engine')
ws.close()
