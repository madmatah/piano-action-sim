/**
 * The bench's model-facing layer.
 *
 * Everything here is a thin pass-through to the Rust engine. It exists so that
 * the React tree never imports wasm directly and never holds a number the engine
 * did not produce: a value computed in TypeScript would be a second physics, and
 * the moment it disagreed with the first we would not know which one the user is
 * reading.
 *
 * The engine answers in documents, each apart: the registry, every path with
 * what it admits and the page that presents it; the settings, always valid,
 * nested as the paths are; the weighing sheet, every reading read or refused
 * with its reason; the diagnostics; where each body stands; what is drawn;
 * and the band, asked for only when it is shown.
 * It is changed by two calls, a path set to a value and an action taken, each
 * answering what was done or why not. They are codes and numbers, never
 * sentences, and their types are generated from the Rust ones
 * (`@pas/wasm/types`), which a test holds to them: the page words them from
 * its catalogue (`i18n/`).
 */
import init, { Bench } from '@pas/wasm'
import type {
  Acted,
  ActionCode,
  Band,
  Diagnostics,
  Procedure,
  Reading,
  Registry,
  Scene,
  Settings,
  Sheet,
  State,
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

/** Every path, what it admits and how it is reached, and the page. */
export function describe(b: Bench): Registry {
  return parse(b.describe())
}

/** Every setting, always valid: what every control reads its value from. */
export function settings(b: Bench): Settings {
  return parse(b.settings())
}

/** The weighing sheet: every reading, read or refused with its reason. */
export function sheet(b: Bench): Sheet {
  return parse(b.sheet())
}

/**
 * The band: the key driven through its stroke, down and back, the force on
 * the measuring point at every step, and the down and up weights read on
 * it. The engine takes it only when it is asked for, so the page asks when
 * it shows the band, never with every edit.
 */
export function band(b: Bench): Band {
  return parse(b.band())
}

/** How the key stands on its frame and why, and the last edit refused. */
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
 * Sets the setting at `path` to `value`, in the path's unit: done, or why
 * the engine refused it, nothing changed.
 */
export function set(b: Bench, path: string, value: number): Acted {
  return parse(b.set(path, value))
}

/** An action's arguments, by the names the registry gives them. */
export type Arguments = Readonly<Record<string, number | string>>

/**
 * Runs the action `action` with its arguments, as a control the registry
 * describes names them: done, the papers a regulation laid and what they
 * give, or why the engine refused it, nothing changed. The engine reads the
 * action (`Action`) and refuses what is not one of its own.
 */
export function act(b: Bench, action: ActionCode, args: Arguments = {}): Acted {
  return parse(b.act(JSON.stringify({ ...args, action })))
}
