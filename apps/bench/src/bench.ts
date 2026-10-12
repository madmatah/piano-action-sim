/**
 * The bench's model-facing layer.
 *
 * Everything here is a thin pass-through to the Rust engine. It exists so that
 * the React tree never imports wasm directly and never holds a number the engine
 * did not produce: a value computed in TypeScript would be a second physics, and
 * the moment it disagreed with the first we would not know which one the user is
 * reading.
 */
import init, { Bench } from '@pas/wasm'

/**
 * One closed outline, flat `[x0, y0, x1, y1, …]`, in millimetres.
 *
 * For an extruded part those pairs are `x` and `y` in the keybed frame. For a
 * turned one they are **radius and height** about `axis`, which is why a turned
 * part is placed rather than carried round: a radius does not rotate.
 */
export interface Outline {
  id: string
  points: number[]
  compliant: boolean
  /** Extrusion across the sagittal plane, millimetres. Zero when turned. */
  width: number
  /** How the profile becomes a solid. A screw is not a prism, and a punching
   * is not a strip of felt. */
  sweep: 'extruded' | 'revolved'
  /** Where a turned part stands — x, y, millimetres — and the sine of its
   * angle. Ignored when extruded. */
  axis: [number, number, number]
  /** For a paper punching, its own thickness in millimetres. The trade
   * identifies a paper by colour and the colour is the thickness, so the
   * palette is keyed on this rather than invented per part. */
  paper_mm: number | null
}

/** A lead, as the circle to draw and the numbers to label it with. */
export interface Lead {
  x: number
  y: number
  radius: number
  mass_g: number
  from_balance_mm: number
}

export interface Scene {
  outlines: Outline[]
  leads: Lead[]
  measuring_point: [number, number]
  /** Why this frame cannot be weighed, when it cannot. It travels with the
   * scene because the readout is precisely what is missing in that case. */
  frame_fault: string | null
  /** Everything about this key and this frame that no keyboard could be built
   * from, already phrased. Empty on a keyboard that goes together — and on the
   * scene rather than the sheet, because a frame too broken to weigh is
   * exactly the one that has these. */
  faults: string[]
}

export interface Readout {
  front_weight_g: number
  key_ratio: number
  key_height_mm: number
  key_dip_mm: number
  down_weight_g: number
  up_weight_g: number
  balance_weight_g: number
  friction_weight_g: number
  key_mass_g: number
  lead_count: number
  balance_paper_mm: number
  front_paper_mm: number
  key_height_target_mm: number
  key_dip_target_mm: number
  /** What is carrying the key: a stop, or its own friction. */
  finger: string
  /** How far from level the key stands at the bottom of its travel, degrees,
   * nose-down positive. A design goal is zero; many keyboards miss. */
  out_of_level_deg: number
  /** How far the key's bearing face is from lying flat on its cloth at rest,
   * degrees. Positive is its rear end digging in. The chamfer under the tail
   * exists so that this is zero. */
  bedding_at_the_back_deg: number
  /** Where the key bears on the cloth, millimetres behind the balance pin.
   * Not the middle of the strip unless the two faces are flat on one
   * another, and it is the arm every number here is computed on. */
  bears_at_the_back_mm: number
  /** Flat on the cloth, on the cloth's edge, or on the key's own corner. */
  back_bearing: string
  /** Whether the pin bottoming in its mortise, rather than the punching, is
   * what stops the key. */
  stopped_by_the_mortise: boolean
  front_pin_mm: number
  front_mortise_mm: number
  finger_weight_g: number
  gap_front_mm: number
  gap_back_mm: number
  friction_band_g: number
  measured_kf_g: number | null
  measured_dip_mm: number | null
  balance_papers_mm: number[]
  front_papers_mm: number[]
  paper_sizes_mm: number[]
  capstan_behind_mm: number
  front_edge_mm: number
  front_rail_at_mm: number
  back_rail_at_mm: number
  capstan_height_mm: number
  balance_felt_mm: number
  front_felt_mm: number
  balance_rail_mm: number
  front_rail_mm: number
  back_rail_mm: number
  back_cloth_mm: number
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
    // `start().then(b => applyUrlState(b))` applies it twice: a URL asking for
    // three leads fits six, and the only symptom is a key mass 66 g too high.
    // It is the same shape of bug as booting twice.
    apply?.(bench)
    return bench
  })()
  return booting
}

export function scene(b: Bench): Scene {
  return JSON.parse(b.scene()) as Scene
}

/**
 * The weighing sheet, or `null` when the geometry has been edited into
 * something degenerate. The caller shows the last good values and says so,
 * rather than rendering `NaN` as though it were a measurement.
 */
export function readout(b: Bench): Readout | null {
  return JSON.parse(b.readout()) as Readout | null
}
