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
 * its code. The wording is the catalog's and changes with the language; the
 * identifiers do not. The test asks for French as the browser's language,
 * by the protocol and not by the machine's locale, so the page opens in
 * French wherever it runs; it then chooses English for the checks that read
 * a sentence the page writes, such as a band's extreme with its number.
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

import { EDGE, MEASURING_POINT_COLOR, PAGE } from '../../apps/bench/src/palette.ts'
import { VIEWPOINT } from '../../apps/bench/src/viewpoint.ts'

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

// French as the browser's language, then the page loaded again in it: an
// override that holds for every page this target loads afterwards.
const { result: browser } = await send('Browser.getVersion')
await send('Emulation.setUserAgentOverride', {
  userAgent: browser.userAgent,
  acceptLanguage: 'fr-FR,fr',
})
await send('Page.navigate', { url: `http://127.0.0.1:${appPort}/` })

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

const chooseLanguage = (language) =>
  evaluate(`
    (() => {
      const b = document.querySelector('[data-language="${language}"]')
      if (!b) return 'no such language'
      b.click()
      return 'ok'
    })()
  `)

/** Fails unless the page opened in the browser's language, French, then
 * chooses English, which the checks of the wording read. */
async function expectFrenchThenChooseEnglish() {
  const opened = await evaluate('document.documentElement.lang')
  if (opened !== 'fr') fail(`the page does not open in the browser's language, fr: ${opened}`)
  if ((await chooseLanguage('en')) !== 'ok') fail('no English in the language selector')
  await settle()
}

/**
 * Clicks inside the punching stack whose papers are at `path`: a paper of
 * `add` millimeters from its drawer, or its `first` or `last` paper lifted
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
const CAPSTAN = 'key.capstan.behind_mm'
const FRONT_EDGE = 'key.front_edge_mm'
const FRONT_RAIL_AT = 'frame.front_rail.at_mm'

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
await waitFor('the engine to boot and fill the weighing sheet', async () => {
  const s = await sheet()
  return Object.keys(s).length > 0 ? s : undefined
})
await expectFrenchThenChooseEnglish()
console.log("  ok  the page opens in the browser's language, French")

// What is on the bench is one of the profiles the engine ships, chosen by the
// action that puts it there, as it arrives. The page opens on Hirschkorn's
// key carrying his wippen on his printed frame. His key comes with its mass published whole, so
// its front weight is Table A.1's static moment over the measuring arm,
// 120.08 g at 51.4 mm over the traced lip's 239.28 mm less 13 mm, 27.28 g,
// read to the tenth as 27.3 g; his key mass is Table A.1's; and his frame
// goes with his key, the borrowed 24 mm cloth strip lying wholly under his
// chamfer, so the page lists no fault.
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
if ((await onTheBench()) !== 'hirschkorn') {
  fail(`the page does not open on Hirschkorn's key: ${await onTheBench()}`)
}
const bootRead = await measured()
const tableA1 = (120.08 * 51.4) / (239.28 - 13)
if (
  bootRead.front_weight?.shown !== 27.3 ||
  !(Math.abs(bootRead.front_weight.exact - tableA1) < 0.001)
) {
  fail(
    `his front weight is not Table A.1's: ${JSON.stringify(bootRead.front_weight)} against ${tableA1}`,
  )
}
const hisMass = (await sheet()).key_mass
if (!(Math.abs(Number(hisMass) - 120.08) < 1e-9))
  fail(`his key mass is not Table A.1's: ${hisMass}`)
const hisFaults = await evaluate(
  `[...document.querySelectorAll('.faults li')].map((li) => li.dataset.code)`,
)
if (!Array.isArray(hisFaults) || hisFaults.length !== 0)
  fail(`his frame does not go with his key: ${JSON.stringify(hisFaults)}`)
// At its top seat his key rests on the cloth's rounded front edge, which
// stands over Pg5, 203.7 mm behind the balance pin, where his printed round
// bears: a chamfer resting on an edge of 0.5 mm radius touches it within that
// radius of the edge's center, along the key.
const PG5_BEHIND_MM = 203.7
const CLOTH_EDGE_RADIUS_MM = 0.5
const hisBack = await evaluate(`
  (() => ({
    how: document.querySelector('[data-back-bearing]')?.dataset.backBearing ?? null,
    at: Number(document.querySelector('.row[data-quantity="bears_at_the_back_mm"]')?.dataset.value),
  }))()
`)
if (
  hisBack?.how !== 'on_the_cloth_edge' ||
  !(Math.abs(hisBack.at - PG5_BEHIND_MM) <= CLOTH_EDGE_RADIUS_MM)
) {
  fail(`his key does not rest on the cloth's edge over Pg5: ${JSON.stringify(hisBack)}`)
}
console.log(
  `  ok  the page opens on Hirschkorn's key: FW ${bootRead.front_weight.exact} g, ${hisMass} g, no fault, resting on the cloth's edge ${hisBack.at} mm behind the pin`,
)

// His key carries his wippen, and a seat with a wippen is the static solve's
// closure from the key alone's seat, to its step tolerance, a picometer
// (`STEP_TOLERANCE` in `crates/pa-core/src/solve/statics.rs`), and not to the
// bit: an edit of a part no seat bears on can still move key height and dip
// within that, as the front mortise widened and lengthened does by a few
// units in their last place. Every other reading keeps its bits.
const CLOSURE_MM = 1e-9
const SEATED = ['key_height', 'key_dip']
const readsAsBooted = (read) =>
  Object.keys(read).length === Object.keys(bootRead).length &&
  Object.entries(bootRead).every(([procedure, booted]) => {
    const now = read[procedure]
    if (!now) return false
    if (!SEATED.includes(procedure)) return now.shown === booted.shown && now.exact === booted.exact
    return (
      Math.abs(now.shown - booted.shown) <= CLOSURE_MM &&
      Math.abs(now.exact - booted.exact) <= CLOSURE_MM
    )
  })

// His key carries his wippen, which the view draws, his capstan bearing on
// its heel cloth. A slider of the « chevalet » panel reaches the engine:
// his wippen's center pin moved 4 mm along the key moves his down weight,
// the heel's arm and the ratio through the capstan changing with it, and
// put back, the sheet reads as the page booted.
const wippenParts = () => evaluate(`document.querySelector('.view')?.dataset.wippenParts ?? null`)
const wippenCarriedBy = () =>
  evaluate(`document.querySelector('[data-wippen-carried-by]')?.dataset.wippenCarriedBy ?? null`)
const hisWippen = (await wippenParts())?.split(' ') ?? []
if (!['wippen', 'wippen_heel', 'wippen_heel_cloth'].every((part) => hisWippen.includes(part))) {
  fail(`his wippen is not drawn: ${JSON.stringify(hisWippen)}`)
}
const hisCarriedBy = await wippenCarriedBy()
if (hisCarriedBy !== 'capstan_on_the_heel:wippen_heel_cloth') {
  fail(`his capstan does not bear on his heel cloth: ${hisCarriedBy}`)
}
const WIPPEN_CENTER = 'wippen.center.behind_mm'
const MOVED_CENTER_MM = 185
const hisCenter = (await slider(WIPPEN_CENTER))?.holds
if ((await moveSlider(WIPPEN_CENTER, MOVED_CENTER_MM)) !== 'ok') {
  fail('no slider over the wippen center pin')
}
await settle()
const centerMoved = await measured()
if (!(Math.abs(Number((await slider(WIPPEN_CENTER))?.holds) - MOVED_CENTER_MM) < 1e-9)) {
  fail(
    `the wippen center pin moved to ${MOVED_CENTER_MM} mm holds ${JSON.stringify(await slider(WIPPEN_CENTER))}`,
  )
}
if (centerMoved.down_weight?.exact === bootRead.down_weight?.exact) {
  fail(
    `his down weight does not move with his wippen's center pin: ${JSON.stringify(centerMoved.down_weight)}`,
  )
}
if ((await moveSlider(WIPPEN_CENTER, Number(hisCenter))) !== 'ok') {
  fail('no slider over the wippen center pin')
}
await settle()
if (JSON.stringify(await measured()) !== JSON.stringify(bootRead)) {
  fail(
    `his wippen's center pin put back does not read as booted: ${JSON.stringify(await measured())}`,
  )
}
console.log(
  `  ok  his wippen drawn on his capstan's heel cloth; its center pin moved to ${MOVED_CENTER_MM} mm moves D from ${bootRead.down_weight.exact} to ${centerMoved.down_weight.exact} g, and put back reads as booted`,
)

/** What the page says, by the identifiers it says it about, and what it
 * shows: every panel's heading and every reading's label, each reading's
 * value as written, and the values themselves, the readings' and the
 * settings'. */
const readWordsAndValues = () =>
  evaluate(`
    (() => {
      const by = (selector, key, value) => Object.fromEntries(
        [...document.querySelectorAll(selector)].map((e) => [key(e), value(e)]))
      return JSON.stringify({
        lang: document.documentElement.lang,
        title: document.title,
        panels: by('h2[data-panel]', (h) => h.dataset.panel, (h) => h.textContent),
        labels: by('.row[data-procedure]', (r) => r.dataset.procedure,
          (r) => r.querySelector('.label')?.textContent ?? ''),
        written: by('.row[data-procedure]:not(.refused)', (r) => r.dataset.procedure,
          (r) => r.querySelector('.value')?.textContent ?? ''),
        values: {
          readings: by('[data-procedure]', (r) => r.dataset.procedure,
            (r) => [r.dataset.shown ?? null, r.dataset.exact ?? null]),
          settings: by('label.slider[data-path]', (l) => l.dataset.path, (l) => l.dataset.value),
        },
      })
    })()
  `).then(JSON.parse)

// The language is the wording's alone: chosen in French, every panel heading
// and every reading's label is reworded and every number written with a
// decimal comma, and no value moves; chosen back in English, the page reads
// as it did.
if ((await chooseLanguage('en')) !== 'ok') fail('no English in the language selector')
await settle()
const inEnglish = await readWordsAndValues()
if ((await chooseLanguage('fr')) !== 'ok') fail('no French in the language selector')
await settle()
const inFrench = await readWordsAndValues()
if (inEnglish.lang !== 'en' || inFrench.lang !== 'fr' || inEnglish.title === inFrench.title) {
  fail(
    `the document does not follow the language: ${JSON.stringify([inEnglish.lang, inFrench.lang, inFrench.title])}`,
  )
}
for (const part of ['panels', 'labels']) {
  const codes = Object.keys(inEnglish[part])
  const same = codes.filter((code) => inFrench[part][code] === inEnglish[part][code])
  if (codes.length === 0 || same.length > 0) {
    fail(`French leaves ${part} as they were: ${JSON.stringify(same.length > 0 ? same : codes)}`)
  }
}
const misWritten = Object.keys(inEnglish.written).filter(
  (code) => inFrench.written[code] !== inEnglish.written[code].replace('.', ','),
)
if (misWritten.length > 0) {
  fail(
    `French writes these readings otherwise than with a decimal comma: ${JSON.stringify(misWritten.map((code) => [inEnglish.written[code], inFrench.written[code]]))}`,
  )
}
if (JSON.stringify(inFrench.values) !== JSON.stringify(inEnglish.values)) {
  fail('choosing French moved a value')
}
// The French names are longer than the English ones: each view's tab keeps
// its name whole on one line, the tabs wrapping under each other instead,
// and the panel holds its lines without a sideways scroll.
const frenchLayout = await evaluate(`
  (() => {
    const panel = document.querySelector('aside.panel')
    return {
      broken: [...document.querySelectorAll('.viewtabs button')]
        .filter((tab) => {
          const name = document.createRange()
          name.selectNodeContents(tab)
          return tab.scrollWidth > tab.clientWidth + 1 || name.getClientRects().length > 1
        })
        .map((tab) => tab.textContent),
      overflow: panel.scrollWidth - panel.clientWidth,
    }
  })()
`)
if (frenchLayout.broken.length > 0 || frenchLayout.overflow > 0) {
  fail(`the page does not hold its French names: ${JSON.stringify(frenchLayout)}`)
}
if ((await chooseLanguage('en')) !== 'ok') fail('no English in the language selector')
await settle()
if (JSON.stringify(await readWordsAndValues()) !== JSON.stringify(inEnglish)) {
  fail('the page does not read as it did once English is chosen again')
}
console.log(
  `  ok  French rewords ${Object.keys(inEnglish.panels).length} headings and ${Object.keys(inEnglish.labels).length} readings, "${inEnglish.labels.down_weight}" as "${inFrench.labels.down_weight}", and moves no value`,
)

// Choosing a language rewords the page and keeps what it holds: a target
// typed and not sent stays in its field, and the band drawn stays drawn and
// current.
const typedBefore = await fieldValue(KEY_HEIGHT)
if ((await setField(KEY_HEIGHT, '64.5')) !== 'ok') fail('no key height field')
if ((await showView('band')) !== 'ok') fail('no band button')
await settle()
const drawnBand = () =>
  evaluate(`
    (() => {
      const b = document.querySelector('.band')
      return b ? [b.dataset.band, b.dataset.stale, b.dataset.samples].join(' ') : null
    })()
  `)
const bandBefore = await drawnBand()
for (const language of ['fr', 'en']) {
  if ((await chooseLanguage(language)) !== 'ok') fail(`no ${language} in the language selector`)
  await settle()
  const [typed, band] = [await fieldValue(KEY_HEIGHT), await drawnBand()]
  if (typed !== '64.5' || band !== bandBefore || !band?.startsWith('drawn false')) {
    fail(
      `choosing ${language} lost what the page held: ${JSON.stringify([typed, band, bandBefore])}`,
    )
  }
}
console.log(`  ok  a language chosen keeps the target typed and the band drawn: ${bandBefore}`)
if ((await setField(KEY_HEIGHT, typedBefore ?? '')) !== 'ok') fail('no key height field')
if ((await showView('solid')) !== 'ok') fail('no solid button')
await settle()

/** The friction control: the key friction weight Stanwood's protocol reads,
 * as shown, or why it is not, and whether it can be moved. */
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

// The owner's measured section arrives unweighed: nobody has weighed one of
// its keys, so it has no mass, and every reading that needs one is refused,
// saying so: front weight, down and up weight, Stanwood's key friction weight,
// and balance and friction weight with them. Key height, dip, the key weight
// ratio and the key friction weight zeroed on the frame read no mass, and are
// read. The friction control, which shows Stanwood's reading, shows none and
// says why; the key mass is refused; the weighing's line says the key is not
// weighed, its mass and front weight fields wait for a number, and the
// weighing waits for them; the touch panel says why a weight on the key does
// nothing.
const MASS_NEEDED = [
  'front_weight',
  'down_weight',
  'up_weight',
  'key_friction_weight_zeroed_on_the_jig',
]
const NO_MASS_READ = [
  'key_weight_ratio',
  'key_height',
  'key_dip',
  'key_friction_weight_zeroed_on_the_frame',
]
if ((await choose('measured-section')) !== 'ok') fail('no measured section to choose')
await settle()
if ((await wippenParts()) !== '' || (await wippenCarriedBy()) !== null) {
  fail(`a wippen is drawn on the measured section: ${await wippenParts()}`)
}
console.log('  ok  no wippen drawn on the measured section, which carries none')
const unweighed = await evaluate(`
  JSON.stringify(Object.fromEntries([...document.querySelectorAll('.row[data-procedure]')].map(
    (r) => [r.dataset.procedure, r.classList.contains('refused') ? (r.dataset.refusal ?? 'refused') : 'read'])))
`).then(JSON.parse)
for (const p of MASS_NEEDED) {
  if (unweighed[p] !== 'key_not_weighed') {
    fail(`${p} on the unweighed key: ${unweighed[p]}, not refused as not weighed`)
  }
}
for (const p of ['balance_weight', 'friction_weight']) {
  if (unweighed[p] === 'read') fail(`${p} is read on a key nobody has weighed`)
}
for (const p of NO_MASS_READ) {
  if (unweighed[p] !== 'read') fail(`${p} needs no mass, and is not read: ${unweighed[p]}`)
}
const unweighedFriction = await frictionControl()
if (
  unweighedFriction?.shown !== null ||
  unweighedFriction.refusal !== 'key_not_weighed' ||
  !unweighedFriction.disabled
) {
  fail(`the friction control on an unweighed key: ${JSON.stringify(unweighedFriction)}`)
}
const invited = await evaluate(`
  (() => {
    const line = document.querySelector('.note.weighed')
    const needs = document.querySelector('.needs-number[data-for-action="weigh_key"]')
    return {
      line: line ? { weighed: line.dataset.massG !== undefined, text: line.textContent } : null,
      needs: needs?.dataset.needs ?? null,
      waits: document.querySelector('button[data-action="weigh_key"]')?.disabled ?? null,
      mass: document.querySelector('.row[data-quantity="key_mass"]')?.classList.contains('refused') ?? null,
      touch: document.querySelector('.note.not-weighed')?.textContent ?? null,
      verdict: document.querySelector('.verdict') !== null,
    }
  })()
`)
if (
  !invited?.line ||
  invited.line.weighed ||
  !invited.line.text ||
  invited.needs !== 'mass_g front_weight_g' ||
  invited.waits !== true ||
  invited.mass !== true ||
  !invited.touch ||
  invited.verdict
) {
  fail(`the unweighed key does not invite its weighing: ${JSON.stringify(invited)}`)
}
console.log(`  ok  the measured section, unweighed: ${invited.line.text}`)

// What a weight does to a key depends on the key's own balance, which only a
// weighing gives: the unweighed key stays drawn at its top seat, its back on
// its cloth, whatever weight stands on its measuring point, with no verdict;
// and it has no band, which says why by the sheet's own refusal.
const drawnAs = () =>
  evaluate(`
    (() => ({
      touching: document.querySelector('.view')?.dataset.touching ?? null,
      verdict: document.querySelector('.verdict') !== null,
    }))()
  `)
const atRest = await drawnAs()
if (!atRest?.touching?.split(' ').includes('back_rail_cloth') || atRest.verdict) {
  fail(`the unweighed key is not drawn at its top seat: ${JSON.stringify(atRest)}`)
}
for (const weight of [40, -40, 0]) {
  if ((await moveSlider(WEIGHT, weight)) !== 'ok') fail('no weight slider')
  await settle()
  const drawn = await drawnAs()
  if (JSON.stringify(drawn) !== JSON.stringify(atRest)) {
    fail(`${weight} g moves a key nobody has weighed: ${JSON.stringify(drawn)}`)
  }
}
console.log(`  ok  the unweighed key stays at its top seat under ±40 g: ${atRest.touching}`)
if ((await showView('band')) !== 'ok') fail('no band button')
await settle()
const unweighedBand = await evaluate(`
  (() => {
    const b = document.querySelector('.band')
    return b ? { band: b.dataset.band, refusal: b.dataset.refusal ?? null, text: b.textContent } : null
  })()
`)
if (
  unweighedBand?.band !== 'refused' ||
  unweighedBand.refusal !== 'key_not_weighed' ||
  !unweighedBand.text
) {
  fail(`the unweighed key draws a band: ${JSON.stringify(unweighedBand)}`)
}
console.log(`  ok  no band on the unweighed key: ${unweighedBand.text}`)
if ((await showView('solid')) !== 'ok') fail('no solid button')
await settle()

// Weighed through the page by the engine's tests' weighing, 121.1 g with
// 6.5 g of front weight read 13 mm back from the lip, it reads every reading,
// the front weight the one typed; the rest of this test runs on that key.
for (const [path, value] of [
  [MASS, 121.1],
  [FRONT_WEIGHT, 6.5],
  [FROM_LIP, 13],
]) {
  if ((await setField(path, value)) !== 'ok') fail(`no ${path} field`)
}
await settle()
if ((await clickAction('weigh_key')) !== 'ok') fail('no button to weigh the key')
await settle()
const start = await sheet()
if (Object.values(start).includes('refused')) {
  fail(`the weighed section still refuses: ${JSON.stringify(start)}`)
}
const weighedAs = await evaluate(`
  (() => {
    const line = document.querySelector('.note.weighed')
    return {
      mass: Number(line?.dataset.massG),
      front: Number(line?.dataset.frontWeightG),
      fromLip: Number(line?.dataset.fromLipMm),
      sheetMass: Number(document.querySelector('.row[data-quantity="key_mass"]')?.dataset.value),
      sheetFront: Number(document.querySelector('.row[data-procedure="front_weight"]')?.dataset.exact),
      touchNote: document.querySelector('.note.not-weighed') !== null,
    }
  })()
`)
// Read 13 mm back from the lip, where the sheet reads it, the front weight
// typed comes back on the same arm, to round-off.
const near = (a, b) => Math.abs(a - b) <= 1e-9
if (
  !weighedAs ||
  !near(weighedAs.mass, 121.1) ||
  !near(weighedAs.front, 6.5) ||
  !near(weighedAs.fromLip, 13) ||
  !near(weighedAs.sheetMass, 121.1) ||
  !near(weighedAs.sheetFront, 6.5) ||
  weighedAs.touchNote
) {
  fail(`the weighing typed is not the key the sheet reads: ${JSON.stringify(weighedAs)}`)
}
console.log(`  ok  the section weighed through the page: ${JSON.stringify(weighedAs)}`)

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
  if (!start[procedure] || !label) fail(`the sheet has no labeled row ${procedure}`)
}

// The friction control shows what Stanwood's protocol reads the bushings as,
// the sheet's own reading: as the trade reads it and in full, at boot, after a
// weight is typed or refused, and after anything that changes what the same
// bushings read. It is the key friction weight the owner types, so a number
// there the sheet does not read is a setting the model does not hold. Where
// the protocol cannot be read it shows no number, says why by the refusal's
// code, and cannot be moved.
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
await expectFrictionShown('once weighed', startRead.key_friction_weight_zeroed_on_the_jig)

// The non-confusion test, run through the whole chain rather than in a unit
// test: more friction must widen the band and leave its center alone. D is
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
const leveled = await sheet()
expectChanged('lay a paper', reweighed, leveled, 'key_height')

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
const unleveled = await sheet()
if (unleveled.key_height !== reweighed.key_height) {
  fail(
    `lifting the paper off left key height at ${unleveled.key_height}, was ${reweighed.key_height}`,
  )
}
console.log(`  ok  lift it off: key height back to ${unleveled.key_height}`)

// Regulation from targets reaches the engine, and so does its refusal. On this
// frame the front punching's felt, with no paper under it, stops the key about
// 0.8 mm short of a 12 mm dip: the bench must refuse it, say so in millimeters
// and name the felt, and change nothing. A dip the felt allows is laid, and the
// sheet reads what the papers give.
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
  refused.limitedBy !== 'felt_thicker_than_the_stack' ||
  !(refused.shortfallMm > 0.7 && refused.shortfallMm < 0.9) ||
  !refused.text
) {
  fail(`a 12 mm dip was not refused with its shortfall: ${JSON.stringify(refused)}`)
}
const afterRefusal = await sheet()
expectUnchanged('a refused regulation', unleveled, afterRefusal, 'key_dip')
expectUnchanged('a refused regulation', unleveled, afterRefusal, 'key_height')
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
// Put back by its slider in millimeters, the rail may stand a last bit away
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
// that made the edit, by its code and the overlap in millimeters; the slider
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

// Each profile is offered by its whole name, which says where its numbers
// come from. Hirschkorn's key chosen again is the instrument the page booted
// on: the same sheet, to the last bit. Every line drawn can be marked by how
// the number that places it is known, and what his profile reads of the
// owner's key frame and keys, its pins and punchings and the front mortise's
// depth, is marked borrowed, which nothing on the measured section is.
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
if (JSON.stringify(await measured()) !== JSON.stringify(bootRead)) {
  fail(
    `his key chosen again is not the one the page booted on: ${JSON.stringify(await measured())}`,
  )
}
console.log("  ok  Hirschkorn's key chosen again reads the sheet the page booted on")

// The key button's sliders, the two mortises' and the balance pin's reach
// the engine. A slot milled narrower than the balance hole fitted to the pin
// is refused beside its cutter's slider, by how much the hole is wider: a
// 3 mm cutter under a hole of 3.4 mm, by 0.4 mm; the setting stays as it was.
// Then each slider moved holds the value it was moved to; his key has its
// mass and its center published, and nothing the key bears on is any of these
// parts, the front mortise widened clear of its pin, so no reading of the
// sheet moves with them, but for the solve's closure of the seats. His key
// chosen again is the one the page booted on.
const SLOT_CUTTER = 'key.balance_mortise.cutter_diameter_mm'
const cutterBefore = await slider(SLOT_CUTTER)
if (cutterBefore === null) fail('slot cutter slider missing')
if ((await moveSlider(SLOT_CUTTER, 3)) !== 'ok') fail('slot cutter slider missing')
await settle()
const cutterRefused = await refusalShown(`.edit-refused[data-for-path="${SLOT_CUTTER}"]`)
if (
  cutterRefused?.code !== 'does_not_fit' ||
  !(Math.abs(cutterRefused.byMm - 0.4) < 0.005) ||
  cutterRefused.after !== SLOT_CUTTER ||
  !cutterRefused.text
) {
  fail(
    `a slot milled narrower than its balance hole is not refused beside its slider: ${JSON.stringify(cutterRefused)}`,
  )
}
if ((await slider(SLOT_CUTTER))?.holds !== cutterBefore.holds) {
  fail(`the refused slot cutter moved its setting: ${JSON.stringify(await slider(SLOT_CUTTER))}`)
}
console.log(`  ok  refused beside its slider: ${cutterRefused.text}`)
const KEY_BUTTON_AND_MORTISE_SLIDERS = [
  ['frame.balance_rail.pin_height_mm', 12],
  ['key.capsule_height_mm', 7],
  ['key.capsule_length_mm', 60],
  ['key.capsule_end_radius_mm', 10],
  ['key.balance_mortise.cutter_diameter_mm', 6.5],
  ['key.balance_mortise.travel_mm', 5.5],
  ['key.balance_mortise.floor_mm', 3],
  ['key.balance_mortise.hole_diameter_mm', 3.5],
  ['key.balance_mortise.bushing_thickness_mm', 1.2],
  ['key.balance_mortise.bushing_depth_mm', 2.5],
  ['key.balance_mortise.side_hole_height_mm', 2.4],
  ['key.front_mortise.cutter_diameter_mm', 6.5],
  ['key.front_mortise.travel_mm', 2.5],
  ['key.front_mortise.groove_depth_mm', 3.5],
  ['key.front_mortise.bushing_thickness_mm', 1.2],
  ['key.front_mortise.bushing_height_mm', 2.5],
  ['key.front_mortise.bushing_flat_width_mm', 4.5],
]
for (const [path, value] of KEY_BUTTON_AND_MORTISE_SLIDERS) {
  if ((await moveSlider(path, value)) !== 'ok') fail(`no slider over ${path}`)
  await settle()
  if (!(Math.abs(Number((await slider(path))?.holds) - value) < 1e-9)) {
    fail(`${path} moved to ${value} holds ${JSON.stringify(await slider(path))}`)
  }
}
if (!readsAsBooted(await measured())) {
  fail(
    `the key button or a mortise moved a published key's sheet: ${JSON.stringify(await measured())}`,
  )
}
console.log(
  `  ok  the key button's, the two mortises' and the balance pin's ${KEY_BUTTON_AND_MORTISE_SLIDERS.length} sliders reach the engine, and move no reading of his published key beyond the seats' closure`,
)
if ((await choose('measured-section')) !== 'ok') fail('no measured section to choose')
await settle()
if ((await choose('hirschkorn')) !== 'ok') fail('no profile choice')
await settle()
if (JSON.stringify(await measured()) !== JSON.stringify(bootRead)) {
  fail(
    `his key chosen again is not the one the page booted on: ${JSON.stringify(await measured())}`,
  )
}
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

// The section draws every part flat, each in its own body's frame, a cut as a
// void in the key's wood. It is a different drawing from the solid's, and a
// renderer that throws there blanks that view alone, so both are shown.
if ((await showView('section')) !== 'ok') fail('no section button')
await settle()
if ((await showView('solid')) !== 'ok') fail('no solid button')
await settle()
console.log('  ok  both views drawn')

// What a profile reads of another instrument than the one it describes is
// marked borrowed, in a color of its own: his pins and punchings, read from
// the owner's key frame, which the section draws as it draws every turned
// part; on the measured section, which describes that frame, they are
// measured and nothing is borrowed. Counted on the section opened by its link, which
// faces it. Where the marked drawing differs from the unmarked one, a line is
// drawn in a mark's color where the edge color was, over the same fill and
// covering as much of the pixel, so the difference is the mark's color less
// the edge's, scaled by that cover: each such pixel is the mark's whose
// difference from the edge points its way, at a quarter of its length at
// least.
await send('Page.navigate', { url: `http://127.0.0.1:${appPort}/?view=section` })
await waitFor('the engine to boot again on the section', async () => {
  const s = await sheet().catch(() => ({}))
  return Object.keys(s).length > 0 ? s : undefined
})
await expectFrenchThenChooseEnglish()
const rgbOf = (hex) => [1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16))
const paintedOn = async (profile) => {
  if ((await choose(profile)) !== 'ok') fail('no profile choice')
  await settle()
  if ((await mark()) !== 'ok') fail('no provenance toggle')
  await settle()
  const area = await drawnArea()
  if (!area || area.height < 50) fail(`no drawing to capture: ${JSON.stringify(area)}`)
  const colors = await evaluate(`
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
      const ways = ${JSON.stringify(colors)}
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
const onOwn = await paintedOn('measured-section')
if (!(onHis.borrowed > 0)) {
  fail(`what his profile borrows is not marked borrowed: ${JSON.stringify(onHis)}`)
}
if (onOwn.borrowed !== 0 || !(onOwn.measured > 0)) {
  fail(`the measured section is marked ${JSON.stringify(onOwn)}`)
}
console.log(`  ok  his key's section marked ${JSON.stringify(onHis)}`)
console.log(`  ok  the measured section's section marked ${JSON.stringify(onOwn)}`)

// The capstan screws into the keystick's full height. On Hirschkorn's key the
// full height ends 132.45 mm behind the pin, the top lowered from there to the
// key's end, and the back check block glued on the lowered part ends 186.87 mm
// behind it; the capstan's head is 3.92 mm in radius. The slider takes the
// capstan 128 mm behind the pin, its head over the full height, and the sheet
// follows; 129 mm is refused, its head standing 0.47 mm over the thinned tail,
// and 190 mm, its head 7.05 mm into the back check block; each refusal said
// beside the slider, which stays where the capstan stands.
if ((await choose('hirschkorn')) !== 'ok') fail('no profile choice')
await settle()
const capstanBefore = await measured()
if ((await moveSlider(CAPSTAN, 128)) !== 'ok') fail('capstan slider missing')
await settle()
const capstanMoved = await slider(CAPSTAN)
if ((await refusalShown('.edit-refused')) !== null || Number(capstanMoved?.holds) !== 128) {
  fail(`the capstan 128 mm behind the pin is refused: ${JSON.stringify(capstanMoved)}`)
}
if ((await measured()).key_weight_ratio?.exact === capstanBefore.key_weight_ratio?.exact) {
  fail('the capstan moved and its key weight ratio did not')
}
console.log(`  ok  the capstan taken 128 mm behind the pin, over the full height`)
for (const [behindMm, code, byMm] of [
  [129, 'capstan_off_the_full_height', 0.47],
  [190, 'capstan_in_block', 7.05],
]) {
  if ((await moveSlider(CAPSTAN, behindMm)) !== 'ok') fail('capstan slider missing')
  await settle()
  const refused = await refusalShown(`.edit-refused[data-for-path="${CAPSTAN}"]`)
  if (
    refused?.code !== code ||
    !(Math.abs(refused.byMm - byMm) < 0.005) ||
    refused.after !== CAPSTAN ||
    !refused.text ||
    Number((await slider(CAPSTAN))?.holds) !== 128
  ) {
    fail(`the capstan ${behindMm} mm behind the pin: ${JSON.stringify(refused)}`)
  }
  console.log(`  ok  the capstan ${behindMm} mm behind the pin refused: ${refused.text}`)
}

// The band, drawn when it is chosen and never with an edit: the key driven
// through its stroke at the measuring point, down and back, the force that
// drives it there at every step. It is drawn on both profiles, the measured
// section weighed, since a key with no mass has none, its two curves inside
// the window the seats leave. The band is a curve, read by no weight: it
// marks the greatest force of the way down and the least of the way up in
// full, and carries no gram reading of either. The gram is the sheet's, the
// walk's with gram weights; the band's extremes are the walk's D and U in
// full within the bound the engine's tests derive, in
// crates/pa-core/src/measure/sweep.rs,
// `the_bands_extremes_are_the_walks_down_and_up_weights`: the walk's
// resolution, the curve's change over the step the sweep reads its extreme
// from and twice its bend over one, and rounding. Each is computed here from
// the curve the page carries in full, step by step, in its table, and held
// under a hundredth of a gram. Within that bound of a whole gram the two
// read the gram either side of it, as the section weighed at 130 g with a
// front weight of 40 g does, the likeliest weighing its owner types: its U is
// -47 g on the sheet from -46.00019 g, and the up curve's least force
// -45.99997 g. D falls on the first step off the top seat and U within a step
// of the bottom one. The weight on the measuring point is drawn across the
// band, the page's own.
const bandShown = () =>
  evaluate(`
    JSON.stringify((() => {
      const band = document.querySelector('.band')
      if (!band) return null
      const forces = (curve) =>
        [...band.querySelectorAll(\`td[data-curve="\${curve}"]\`)].map((td) => Number(td.dataset.forceG))
      return {
        band: { ...band.dataset },
        readings: Object.fromEntries([...band.querySelectorAll('.band-reading')].map((g) => [
          g.dataset.procedure,
          {
            exact: Number(g.dataset.exact),
            atMm: Number(g.dataset.atMm),
            gram: g.dataset.shown ?? null,
            label: g.querySelector('text')?.textContent ?? null,
          },
        ])),
        written: [...band.querySelectorAll('.band-extremes li')].map((li) => li.textContent),
        curves: [...band.querySelectorAll('path[data-curve]')].map((p) => p.dataset.curve),
        forces: { down: forces('down'), up: forces('up') },
        touch: { ...(band.querySelector('.band-touch')?.dataset ?? {}) },
      }
    })())
  `).then(JSON.parse)
const BAND_READS = {
  down_weight_on_the_band: ['down_weight', 'down'],
  up_weight_on_the_band: ['up_weight', 'up'],
}
/**
 * How far an extreme the band reads on `curve`, its forces in full, may
 * stand from the walk's reading in full, grams, as the engine's tests derive
 * it: the walk's resolution, half the bracket its sixteen halvings leave of
 * a gram (walk.rs, STEP_G and HALVINGS) and what the static solve may leave
 * out of balance where it decides, its residual's tolerance of 1e-12 of a
 * load scale under two newtons, over a reach under 0.3 m, read at a
 * measuring arm of at least 0.2 m (statics.rs, RESIDUAL_TOLERANCE); the
 * curve's greatest change over one step and twice its greatest change of
 * slope over one; and rounding, 64 units in the last place of a key under a
 * kilogram at an arm under 0.3 m with bushings under a newton-meter, over
 * the measuring arm.
 */
const bandBound = (curve) => {
  const resolution = (0.5 * 1) / 2 ** 16 + (1000 * 1e-12 * 2 * 0.3) / (9.80665 * 0.2)
  let step = 0
  let bend = 0
  for (let i = 1; i < curve.length; i++) step = Math.max(step, Math.abs(curve[i] - curve[i - 1]))
  for (let i = 2; i < curve.length; i++) {
    bend = Math.max(bend, Math.abs(curve[i - 2] - 2 * curve[i - 1] + curve[i]))
  }
  const rounding = (64 * Number.EPSILON * 1000 * (1 * 9.80665 * 0.3 + 1)) / (9.80665 * 0.2)
  return resolution + step + 2 * bend + rounding
}
/** A value as the page writes it to `digits` decimals, no negative zero. */
const toDecimals = (x, digits) => {
  const s = x.toFixed(digits)
  return Number(s) === 0 ? (0).toFixed(digits) : s
}
/** The numbers a text writes, as written. */
const numbersIn = (text) => text?.match(/-?\d+(?:\.\d+)?/g) ?? []
/** The band's two extremes against the sheet's walks, each within its
 * bound, the band reading no gram; their bounds, by procedure. What the page
 * writes beside each extreme, on the chart and for a reader without it, is
 * read as a technician reads it: its one force is the extreme's in full, to
 * the hundredth of a gram, the written one adding only where it falls, and
 * neither names it D or U, which are the sheet's. */
const bandAgainstSheet = async (what, b) => {
  const read = await measured()
  const bounds = {}
  for (const [i, [onBand, [walked, curve]]] of Object.entries(BAND_READS).entries()) {
    const [m, w] = [b.readings[onBand], read[walked]]
    const forces = b.forces[curve]
    const bound = bandBound(forces)
    if (!m || !w || !(forces.length > 100) || !forces.every(Number.isFinite)) {
      fail(`${onBand} on ${what}'s band, or its curve, is not read: ${JSON.stringify([m, w])}`)
    }
    if (m.gram !== null) fail(`${onBand} on ${what}'s band shows a gram reading: ${m.gram}`)
    const written = b.written[i]
    const inFull = toDecimals(m.exact, 2)
    if (
      JSON.stringify(numbersIn(m.label)) !== JSON.stringify([inFull]) ||
      JSON.stringify(numbersIn(written)) !== JSON.stringify([inFull, toDecimals(m.atMm, 2)]) ||
      [m.label, written].some((text) => /\b[DU]\b/.test(text))
    ) {
      fail(
        `${onBand} on ${what}'s band is not written in full, ${inFull} g, or is named as the sheet's: ${JSON.stringify([m.label, written])}`,
      )
    }
    if (!(bound < 0.01) || !(Math.abs(m.exact - w.exact) <= bound)) {
      fail(
        `${onBand} on ${what}'s band is not the sheet's in full: ${m.exact} g against ${w.exact} g, the bound ${bound} g`,
      )
    }
    console.log(
      `  ok  ${what}: ${onBand} ${m.exact} g at ${m.atMm.toFixed(2)} mm, written "${m.label}"; the sheet's ${w.exact} g read ${w.shown} g, within ${bound.toExponential(2)} g`,
    )
    bounds[onBand] = bound
  }
  return bounds
}
let bandOn = null
/** The measured section put on the bench and weighed through the page, by
 * the tests' weighing unless another is given. */
const weighedSection = async (massG = 121.1, frontWeightG = 6.5) => {
  if ((await choose('measured-section')) !== 'ok') fail('no profile choice')
  await settle()
  for (const [path, value] of [
    [MASS, massG],
    [FRONT_WEIGHT, frontWeightG],
    [FROM_LIP, 13],
  ]) {
    if ((await setField(path, value)) !== 'ok') fail(`no ${path} field`)
  }
  await settle()
  if ((await clickAction('weigh_key')) !== 'ok') fail('no button to weigh the key')
  await settle()
}
for (const [what, put] of [
  ['the measured section', () => weighedSection()],
  ['the measured section at 130 g and 40 g', () => weighedSection(130, 40)],
  ['hirschkorn', () => choose('hirschkorn')],
]) {
  if ((await put()) === 'no profile choice') fail('no profile choice')
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
    fail(`the band is not drawn on ${what}: ${JSON.stringify(b)}`)
  }
  await bandAgainstSheet(what, b)
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
  if (grown) fail(`the band lengthens the page on ${what}: ${JSON.stringify(grown)}`)
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
// width. Each of the band's extremes stands within its bound of the walk's in
// full, before the gram and after it, and the sheet's balance weight in full
// is the middle of the walk's D and U, so the band's middle moves as the
// balance weight does to within half the four bounds added up. The width is
// D − U, which the gram moves by under 1 g × 1 %, the two seats' arms apart,
// and each of its four extremes stands within its bound of the walk's: under
// 0.01 g and the four bounds in all.
const sheetBefore = await measured()
const boundsBefore = await bandAgainstSheet('his key, its bushings tighter', tighter)
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
const boundsAfter = await bandAgainstSheet('his key, a gram heavier in front', heavier)
const fourBounds = [...Object.values(boundsBefore), ...Object.values(boundsAfter)].reduce(
  (sum, bound) => sum + bound,
  0,
)
const middleMoved = bandMiddle(heavier.readings) - bandMiddle(tighter.readings)
const balanceMoved = sheetAfter.balance_weight.exact - sheetBefore.balance_weight.exact
const widthMoved = bandWidth(heavier.readings) - bandWidth(tighter.readings)
if (!(Math.abs(middleMoved) > 0.5) || !(Math.abs(middleMoved - balanceMoved) <= fourBounds / 2)) {
  fail(
    `a gram at the front moved the band's middle by ${middleMoved} g, the sheet's balance weight by ${balanceMoved} g`,
  )
}
if (!(Math.abs(widthMoved) <= 0.01 + fourBounds)) {
  fail(`a gram at the front moved the band's width by ${widthMoved} g`)
}
console.log(
  `  ok  a gram at the front moves the band's middle ${middleMoved.toFixed(3)} g (the sheet's BW ${balanceMoved.toFixed(3)} g), its width ${widthMoved.toFixed(4)} g`,
)

// A key with no stroke has no band: on the measured section's frame, the key
// weighed, its balance rail 6.6 mm low, its stops fight, it rides on them,
// and the band says why with the sheet's own refusal.
await weighedSection()
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

// A key no equilibrium places bears on nothing: on a balance pin cut short,
// 9 mm above its punching, with the front rail moved back until its pin
// stands under the wood of the key, which it would lift off its balance pin,
// the key has no place on its frame. The view says so and marks no contact
// closed; with the rail and the pin put back, the key is placed again and
// bears on its stops.
const FRONT_RAIL_UNDER_THE_WOOD_MM = 80
const BALANCE_PIN = 'frame.balance_rail.pin_height_mm'
const SHORT_BALANCE_PIN_MM = 11.2
const readPlacement = () =>
  evaluate(`
    (() => {
      const view = document.querySelector('.view')
      return {
        place: view?.dataset.place ?? null,
        touching: view?.dataset.touching ?? null,
        note: document.querySelector('.view-note')?.textContent ?? null,
      }
    })()
  `)
if ((await showView('solid')) !== 'ok') fail('no solid button')
const railAtMm = (await slider(FRONT_RAIL_AT))?.holds
const pinMm = (await slider(BALANCE_PIN))?.holds
if ((await moveSlider(BALANCE_PIN, SHORT_BALANCE_PIN_MM)) !== 'ok')
  fail('balance pin slider missing')
if ((await moveSlider(FRONT_RAIL_AT, FRONT_RAIL_UNDER_THE_WOOD_MM)) !== 'ok') {
  fail('front rail slider missing')
}
await settle()
const noPlace = await readPlacement()
if (noPlace.place !== 'unplaced' || noPlace.touching !== '' || !noPlace.note) {
  fail(`a key no equilibrium places is drawn as if it bore: ${JSON.stringify(noPlace)}`)
}
if ((await moveSlider(FRONT_RAIL_AT, railAtMm)) !== 'ok') fail('front rail slider missing')
if ((await moveSlider(BALANCE_PIN, pinMm)) !== 'ok') fail('balance pin slider missing')
await settle()
const placedAgain = await readPlacement()
if (placedAgain.place !== 'placed' || placedAgain.touching === '' || placedAgain.note !== null) {
  fail(`the key put back on its frame is not drawn placed: ${JSON.stringify(placedAgain)}`)
}
console.log(`  ok  a key no equilibrium places is drawn bearing on nothing: ${noPlace.note}`)

// The key's opacity is the view's alone: chosen lower, the key is drawn
// fainter in either view; chosen whole again, the view is drawn as it was.
const chooseKeyOpacity = (opacity) => typeInto('[data-view-setting="key-opacity"]', opacity)
for (const v of ['section', 'solid']) {
  if ((await showView(v)) !== 'ok') fail(`no ${v} button`)
  await settle()
  const whole = await drawnArea()
  const before = await drawn(whole)
  if ((await chooseKeyOpacity(0.3)) !== 'ok') fail('no key opacity slider')
  await settle()
  const faint = await drawn(whole)
  if ((await chooseKeyOpacity(1)) !== 'ok') fail('no key opacity slider')
  await settle()
  if (!before || faint === before) fail(`a fainter key changes nothing drawn in the ${v}`)
  if ((await drawn(whole)) !== before) {
    fail(`the ${v} is not drawn as it was once the key is whole again`)
  }
}
console.log('  ok  the key drawn fainter as chosen, and whole again as it was, in both views')

// The bushings' opacity is the view's alone too, apart from the key's: in a
// faded key, chosen lower, the two mortises' bushings are drawn fainter in
// either view; chosen whole again, the view is drawn as it was.
const chooseBushingsOpacity = (opacity) =>
  typeInto('[data-view-setting="bushings-opacity"]', opacity)
for (const v of ['section', 'solid']) {
  if ((await showView(v)) !== 'ok') fail(`no ${v} button`)
  if ((await chooseKeyOpacity(0.3)) !== 'ok') fail('no key opacity slider')
  await settle()
  const whole = await drawnArea()
  const before = await drawn(whole)
  if ((await chooseBushingsOpacity(0.3)) !== 'ok') fail('no bushings opacity slider')
  await settle()
  const faint = await drawn(whole)
  if ((await chooseBushingsOpacity(1)) !== 'ok') fail('no bushings opacity slider')
  await settle()
  if (!before || faint === before) fail(`fainter bushings change nothing drawn in the ${v}`)
  if ((await drawn(whole)) !== before) {
    fail(`the ${v} is not drawn as it was once the bushings are whole again`)
  }
  if ((await chooseKeyOpacity(1)) !== 'ok') fail('no key opacity slider')
  await settle()
}
console.log('  ok  the bushings drawn fainter as chosen, and whole again as it was, in both views')

// A faded key keeps its colors whichever way the camera turns round it.
// three's own transparency draws faded parts one after another, sorted by
// how far their middles stand from the camera; on his key the keytop's
// middle stands about 159 mm in front of the keystick's and 13 mm above it
// (the key's outline in data/params/hirschkorn-key-outline.json, with its
// front_edge, front_board, tail_end, keytop_length, keytop_overhang,
// keytop_thickness and stick_height), so
// that order flips about a degree from square to the key, and a camera
// turned 5° either side of square sees the two parts in both orders.
//
// On a fresh load, since the camera is made once from the viewpoint the
// page opens in, the solid's, never dragged; his key placed; and the
// images the solid draws a faded key into available. A faded part writes no
// depth, so two faded faces of opacity a drawn in sorted order blend to
// a·c_last + (1−a)(a·c_first + (1−a)·B), B being what lies behind: swapping
// them changes the pixel by a²·(c_last − c_first), which grows with a. At
// 0.5 the white keytop over the keystick's wood leaves a few levels; 0.9
// leaves over three times that.
//
// The pixels read are those straight below the measuring point's marker,
// on the covering's top and drawn over everything: the column under the
// marker's tip, its first run of at least two identical pixels. The view is
// orthographic and a sideways drag keeps the elevation, so their rays at the
// two angles mirror each other across the plane square to the key through
// the point they look at: each enters through the keytop's top face and
// crosses the keystick's, both faded, then reaches the page, and moves at
// most about 3 mm along the key, short of the front mortise, 16.6 mm behind
// the key front (front_mortise.from_front_edge in
// data/profiles/hirschkorn.json, less half the mortise's length, its
// front_mortise_travel and front_mortise_cutter_diameter in
// data/params/key-section.json) with 2 mm of wood over it (stick_height in
// data/params/hirschkorn-key-outline.json, less front_mortise_depth in
// data/params/key-section.json), where the measuring point stands 11.5 mm
// behind it (measuring_point_from_lip in
// data/params/touchweight-protocol.json, less keytop_overhang in
// data/params/hirschkorn-key-outline.json). Each image
// rounds its blend to half precision, so the same two faces taken in either
// order differ by at most 2⁻¹⁰ of the value, one level on the canvas.
await send('Page.navigate', { url: `http://127.0.0.1:${appPort}/` })
await waitFor('the engine to boot again on the solid', async () => {
  const s = await sheet().catch(() => ({}))
  return Object.keys(s).length > 0 ? s : undefined
})
await settle()
const opened = await evaluate(`
  (() => ({
    place: document.querySelector('.view')?.dataset.place ?? null,
    solid: document.querySelector('[data-view="solid"]')?.classList.contains('on') ?? false,
  }))()
`)
if (opened?.place !== 'placed' || opened.solid !== true) {
  fail(`the bench does not open on the solid with his key placed: ${JSON.stringify(opened)}`)
}
const drawsApart = await evaluate(`
  (() => {
    const offered = document.querySelector('.view canvas')?.getContext('webgl2')
      ?.getSupportedExtensions() ?? []
    return offered.includes('EXT_color_buffer_float') || offered.includes('EXT_color_buffer_half_float')
  })()
`)
if (drawsApart !== true) {
  fail('the browser offers no image of fractional colors, so a faded key cannot be drawn apart')
}
const OPAQUE_ENOUGH_TO_TELL_THE_ORDER = 0.9
if ((await chooseKeyOpacity(OPAQUE_ENOUGH_TO_TELL_THE_ORDER)) !== 'ok') {
  fail('no key opacity slider')
}
await settle()
// three-stdlib's OrbitControls turns the camera about the vertical by
// 2π·Δx over the height of the element it listens on, a drag to the right
// lowering the azimuth, and leaves the elevation where it is.
const solidViewpoint = VIEWPOINT.solid
const fromTarget = solidViewpoint.position.map(
  (coordinate, axis) => coordinate - solidViewpoint.target[axis],
)
const openingAzimuth = Math.atan2(fromTarget[0], fromTarget[2])
const sinElevation = fromTarget[1] / Math.hypot(...fromTarget)
const SQUARE_OFF = (5 * Math.PI) / 180
const canvasBox = await evaluate(`
  (() => {
    const canvas = document.querySelector('.view canvas')
    const box = canvas.getBoundingClientRect()
    const middle = { x: Math.round(box.left + box.width / 2), y: Math.round(box.top + box.height / 2) }
    return {
      middleX: box.left + box.width / 2,
      listenerHeight: canvas.parentElement.clientHeight,
      dragFrom: middle,
      isBare: document.elementFromPoint(middle.x, middle.y) === canvas,
    }
  })()
`)
if (!canvasBox?.isBare) fail(`the drag would not start on the canvas: ${JSON.stringify(canvasBox)}`)
const pixelsToTurn = (fromAzimuth, toAzimuth) =>
  Math.round(((fromAzimuth - toAzimuth) * canvasBox.listenerHeight) / (2 * Math.PI))
const turnedBy = (fromAzimuth, pixels) =>
  fromAzimuth - (2 * Math.PI * pixels) / canvasBox.listenerHeight
const dragSideways = async (pixels) => {
  const { x, y } = canvasBox.dragFrom
  const left = { button: 'left', buttons: 1 }
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y })
  await send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, ...left, clickCount: 1 })
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: x + pixels, y, ...left })
  await send('Input.dispatchMouseEvent', {
    type: 'mouseReleased',
    x: x + pixels,
    y,
    button: 'left',
    buttons: 0,
    clickCount: 1,
  })
  await settle()
}
const turningArea = await drawnArea()
if (!turningArea || turningArea.height < 50) {
  fail(`no drawing to capture: ${JSON.stringify(turningArea)}`)
}
/** The measuring point's marker in a capture, its tip the middle of its
 * lowest row, and the first run of at least two identical pixels straight
 * below it. */
const readBelowTheMarker = (png) =>
  evaluate(`
    (async () => {
      const image = await (await fetch('data:image/png;base64,${png}')).blob()
      const bitmap = await createImageBitmap(image)
      const context = new OffscreenCanvas(bitmap.width, bitmap.height).getContext('2d')
      context.drawImage(bitmap, 0, 0)
      const { data, width, height } = context.getImageData(0, 0, bitmap.width, bitmap.height)
      const at = (column, row) => [...data.slice((row * width + column) * 4, (row * width + column) * 4 + 3)]
      const marker = ${JSON.stringify(rgbOf(MEASURING_POINT_COLOR))}
      let lowest = -1
      let columns = []
      for (let row = 0; row < height; row += 1) {
        for (let column = 0; column < width; column += 1) {
          if (!at(column, row).every((channel, index) => Math.abs(channel - marker[index]) <= 1)) continue
          if (row > lowest) {
            lowest = row
            columns = []
          }
          columns.push(column)
        }
      }
      if (lowest < 0) return null
      const column = Math.round((Math.min(...columns) + Math.max(...columns)) / 2)
      for (let row = lowest + 1; row < height; ) {
        const color = at(column, row)
        let end = row + 1
        while (end < height && at(column, end).every((channel, index) => channel === color[index])) end += 1
        if (end - row >= 2) return { tip: { column, row: lowest }, run: { color, from: row, length: end - row } }
        row = end
      }
      return { tip: { column, row: lowest }, run: null }
    })()
  `)
const unturned = await drawn(turningArea)
const firstAzimuth = turnedBy(openingAzimuth, pixelsToTurn(openingAzimuth, -SQUARE_OFF))
await dragSideways(pixelsToTurn(openingAzimuth, -SQUARE_OFF))
const turnedOnce = await drawn(turningArea)
const secondAzimuth = turnedBy(firstAzimuth, pixelsToTurn(firstAzimuth, SQUARE_OFF))
await dragSideways(pixelsToTurn(firstAzimuth, SQUARE_OFF))
const turnedTwice = await drawn(turningArea)
if (!unturned || turnedOnce === unturned || turnedTwice === turnedOnce) {
  fail('a sideways drag on the solid does not turn the camera')
}
const [beforeTheSwap, afterTheSwap] = [
  await readBelowTheMarker(turnedOnce),
  await readBelowTheMarker(turnedTwice),
]
if (!beforeTheSwap?.run || !afterTheSwap?.run) {
  fail(
    `no run of pixels under the measuring point: ${JSON.stringify([beforeTheSwap, afterTheSwap])}`,
  )
}
// Where the target projects, in the middle of the canvas, the tip stands x
// pixels aside at the first azimuth θ₁, so X·zoom = x / cos θ₁ along the
// key; turned to θ₂ at the same elevation φ, it moves down the rows by
// X·zoom·sin φ·(sin θ₂ − sin θ₁), give or take a row for each tip located.
const tipAside = beforeTheSwap.tip.column + 0.5 + turningArea.x - canvasBox.middleX
const predictedRows =
  (tipAside / Math.cos(firstAzimuth)) *
  sinElevation *
  (Math.sin(secondAzimuth) - Math.sin(firstAzimuth))
const movedRows = afterTheSwap.tip.row - beforeTheSwap.tip.row
if (!(Math.abs(movedRows - predictedRows) <= 2)) {
  fail(
    `the measuring point moved ${movedRows} rows as the camera turned, not the ${predictedRows.toFixed(1)} its azimuths predict`,
  )
}
const formatDegrees = (azimuth) => ((azimuth * 180) / Math.PI).toFixed(2)
console.log(
  `  ok  the camera turned from ${formatDegrees(openingAzimuth)}° to ${formatDegrees(firstAzimuth)}° and ${formatDegrees(secondAzimuth)}°, the measuring point moving ${movedRows} rows for ${predictedRows.toFixed(1)}`,
)
const [colorBefore, colorAfter] = [beforeTheSwap.run.color, afterTheSwap.run.color]
const pageColor = rgbOf(PAGE)
if (
  colorBefore.some((channel, index) => Math.abs(channel - colorAfter[index]) > 1) ||
  colorBefore.every((channel, index) => Math.abs(channel - pageColor[index]) <= 1)
) {
  fail(
    `the faded key under the measuring point turns from ${JSON.stringify(colorBefore)} to ${JSON.stringify(colorAfter)} as the camera turns past square`,
  )
}
console.log(
  `  ok  the faded key keeps its colors either side of square: ${JSON.stringify(colorBefore)} and ${JSON.stringify(colorAfter)}, over ${beforeTheSwap.run.length} and ${afterTheSwap.run.length} pixels`,
)

if (thrown.length > 0) fail(`the page threw: ${thrown[0].split('\n')[0]}`)

console.log('all interactions reached the engine')
ws.close()
