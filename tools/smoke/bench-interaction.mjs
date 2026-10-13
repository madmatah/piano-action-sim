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

// The section draws every part flat, each in its own body's frame, a cut as a
// void in the key's wood. It is a different drawing from the solid's, and a
// renderer that throws there blanks that view alone, so both are shown.
if ((await showView('section')) !== 'ok') fail('no section button')
await settle()
if ((await showView('solid')) !== 'ok') fail('no solid button')
await settle()
console.log('  ok  both views drawn')

if (thrown.length > 0) fail(`the page threw: ${thrown[0].split('\n')[0]}`)

console.log('all interactions reached the engine')
ws.close()
