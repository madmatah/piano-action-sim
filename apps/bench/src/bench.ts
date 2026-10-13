/**
 * The bench's model-facing layer.
 *
 * Everything here is a thin pass-through to the Rust engine. It exists so that
 * the React tree never imports wasm directly and never holds a number the engine
 * did not produce: a value computed in TypeScript would be a second physics, and
 * the moment it disagreed with the first we would not know which one the user is
 * reading.
 *
 * The engine answers in documents, each apart: the settings, always valid; the
 * weighing sheet, every reading read or refused with its reason; the
 * diagnostics; where each body stands; what is drawn. They are codes and
 * numbers, never sentences, and their types are generated from the Rust ones
 * (`@pas/wasm/types`), which a test holds to them: the page words them from
 * its catalogue (`i18n/`).
 */
import init, { Bench } from '@pas/wasm'
import type {
  Diagnostics,
  FrictionRefusal,
  Procedure,
  Reading,
  Regulation,
  Scene,
  Settings,
  Sheet,
  State,
  WeighingRefusal,
} from '@pas/wasm/types'

/**
 * A document from the JSON the engine wrote it in. Its type is generated
 * from the Rust type that wrote it, and a test fails when the two differ, so
 * the parse needs no check of its own.
 */
function parse<T>(json: string): T {
  const document: T = JSON.parse(json)
  return document
}

let booting: Promise<Bench> | undefined

/**
 * Boots the engine. Genuinely idempotent.
 *
 * # The bug this shape exists to prevent
 *
 * The obvious version checks a cached instance and creates one if there is
 * none:
 *
 * ```ts
 * if (!bench) { await init(); bench = new Bench() }
 * ```
 *
 * That is **not** idempotent, because the check happens before an `await`. Two
 * concurrent callers both see no instance, both wait, and both construct one —
 * which React's StrictMode produces on every mount in development, by design.
 * The generated `init()` has the same shape, so each call instantiates its own
 * wasm module, and the glue keeps only the last one as the module-level
 * instance every later call goes through. Both instances start from the same
 * state, so the two `Bench` wrappers hold the same pointer, resolved in that
 * last instance. When the unreachable wrapper is finalized,
 * `FinalizationRegistry` frees that pointer — the survivor's own object — and
 * the survivor's next call traps with `RuntimeError: memory access out of
 * bounds`.
 *
 * Caching the **promise** rather than the result closes it: the second caller
 * awaits the first caller's construction instead of starting its own.
 */
export function start(apply?: (b: Bench) => void): Promise<Bench> {
  booting ??= (async () => {
    await init()
    const bench = new Bench()
    // Any one-time setup belongs **inside** the memoised promise, not in the
    // caller's effect. React's StrictMode runs an effect body twice, so a
    // `start().then(b => applyUrlState(b))` applies it twice: harmless for a
    // weight that is set, wrong for anything that adds, a paper laid twice
    // being two papers. It is the same shape of bug as booting twice.
    apply?.(bench)
    return bench
  })()
  return booting
}

/** Every setting, always valid: what every control reads its value from. */
export function settings(b: Bench): Settings {
  return parse(b.settings())
}

/** The weighing sheet: every reading, read or refused with its reason. */
export function sheet(b: Bench): Sheet {
  return parse(b.sheet())
}

/** How the key stands on its frame and why. */
export function diagnostics(b: Bench): Diagnostics {
  return parse(b.diagnostics())
}

/** Where each body stands, and the contacts the view marks. */
export function state(b: Bench): State {
  return parse(b.state())
}

/**
 * What is drawn, as the engine wrote it: the JSON beside the scene, so a
 * caller can keep the scene it has while the geometry has not changed, and
 * the view does not rebuild a part that has not moved.
 */
export function scene(b: Bench, last?: { json: string; scene: Scene }) {
  const json = b.scene()
  return last?.json === json ? last : { json, scene: parse<Scene>(json) }
}

/** The reading `procedure` of a sheet, if the sheet holds it. */
export function reading(s: Sheet, procedure: Procedure): Reading | undefined {
  return s.readings.find((r) => r.procedure === procedure)
}

/**
 * Regulates the key to a key height and a key dip, millimetres: the papers
 * the engine laid and what they give, or its refusal, nothing changed.
 */
export function regulate(b: Bench, keyHeightMm: number, keyDipMm: number): Regulation {
  return parse(b.regulate(keyHeightMm, keyDipMm))
}

/**
 * Sets the bushings from a key friction weight, grams, as Stanwood's protocol
 * reads it: the engine finds the torque at which the protocol reads it back.
 * Returns why it refused, the bushings left as they were, or `null` when it
 * was set.
 */
export function setKeyFrictionWeight(b: Bench, grams: number): FrictionRefusal | null {
  return parse(b.set_key_friction_weight(grams))
}

/**
 * Weighs the key whole, as it arrives, its leads in it: its mass in grams, and
 * its front weight in grams read on the levelled jig `fromLipMm` back from the
 * covering's lip. The engine keeps the static moment the front weight gives.
 * Returns why it refused, the key's weighing left as it was, or `null` when
 * it was taken.
 */
export function weighKey(
  b: Bench,
  massG: number,
  frontWeightG: number,
  fromLipMm: number,
): WeighingRefusal | null {
  return parse(b.weigh_key(massG, frontWeightG, fromLipMm))
}
