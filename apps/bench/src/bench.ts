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

/** Where a stretch of an outline comes from: a record under `data/params/`. */
export interface Source {
  /** The first segment. */
  from: number
  /** One past the last. */
  to: number
  how: 'measured' | 'published' | 'traced' | 'assumed'
  record: string
}

/**
 * One part, in millimetres, in its own body's frame.
 *
 * For an extruded part `points` are `x` and `y` in the body's frame. For a
 * turned one they are its profile in **radius and height** about `axis_x`,
 * which is why a turned part is lathed rather than extruded. Where the part
 * stands is not here: it is its body's pose, in `Scene.poses`.
 */
export interface Outline {
  /** The body that carries it: an index into `Scene.poses`. */
  body: number
  /** What it is, the engine's identifier. */
  code: string
  /** A cut is a void in its body's solids, drawn in the section only. */
  role: 'solid' | 'cut' | 'lining'
  /** What it is made of. The colour is the view's choice, from this. */
  material: string
  /** For a paper punching, its own thickness in millimetres. The trade
   * identifies a paper by colour and the colour is the thickness, so the
   * palette is keyed on this rather than invented per part. */
  paper_mm: number | null
  /** How the outline becomes a solid. A screw is not a prism, and a punching
   * is not a strip of felt. */
  sweep: 'extruded' | 'revolved'
  /** Extrusion across the sagittal plane, millimetres. Zero when turned. */
  width: number
  /** Where a turned part's axis stands along its body's `x`, millimetres. */
  axis_x: number
  /** Flat `[x0, y0, x1, y1, …]`, the closed outline, arcs cut into lines. */
  points: number[]
  /** The segment of the engine's outline each point starts or lies on. */
  segs: number[]
  sources: Source[]
}

/**
 * Where one body's frame stands: its origin in millimetres, and the cosine
 * and sine of its rotation. The view applies it as given and computes none.
 */
export interface Pose {
  origin: [number, number]
  cos: number
  sin: number
}

/** A lead, as the circle to draw on its body and the numbers to label it with. */
export interface Lead {
  /** The body it rides on: an index into `Scene.poses`. */
  body: number
  x: number
  y: number
  radius: number
  mass_g: number
  from_balance_mm: number
}

/** A point marked on a body, millimetres in the body's frame. */
export interface Marker {
  body: number
  at: [number, number]
}

/**
 * A contact the view marks: one row of one of the key's contacts with its
 * frame, where the key is drawn. In the keybed's frame, the ground's, which
 * the view draws untransformed: the points are where the engine's contact rows
 * put them, not where a body's pose carries them.
 */
export interface Contact {
  /** Which contact, the engine's identifier. */
  pair: string
  /** Where the key's surface is, millimetres. */
  on_key: [number, number]
  /** Where the frame's surface is, millimetres. */
  on_frame: [number, number]
  /** The signed gap between them, millimetres: negative where the drawn parts
   * overlap. */
  gap_mm: number
  /** Within a tenth of a micron of touching, apart, or overlapping. */
  state: 'closed' | 'open' | 'overlapping'
}

export interface Scene {
  outlines: Outline[]
  /** One per body, in body order. */
  poses: Pose[]
  leads: Lead[]
  measuring_point: Marker
  /** The contacts within a millimetre of closing, where the key is drawn. */
  contacts: Contact[]
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
  /** What holds the key where the weight has left it: a stop it rests on,
   * or, between the up weight and the down weight, its friction. */
  finger: string
  /** Whether friction alone holds the key where it is. */
  held_by_friction: boolean
  /** How far the weight stands inside the friction band, from its nearer
   * edge, grams; `null` when a stop carries the key. */
  friction_margin_g: number | null
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
  /** The gap at the front punching, read at its centre, millimetres; `null`
   * where no underside passes over its centre. */
  gap_front_mm: number | null
  /** The gap at the back rail cloth, millimetres; `null` where no feature of
   * the two can meet. */
  gap_back_mm: number | null
  friction_band_g: number
  measured_kf_g: number | null
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
  /** How far back from the front lip of the key every weight is centred,
   * millimetres: the measuring point every touchweight on the sheet is read
   * at. The lip is the covering's, which overhangs the key front. */
  measuring_point_from_lip_mm: number
}

/** How a regulation the page asked for came out. */
export interface Regulation {
  /** Whether the papers were laid. When not, nothing on the bench changed. */
  laid: boolean
  /** What happened, in the engine's sentence: the papers laid and what they
   * give, or the target out of reach, by how many millimetres and what keeps
   * it out. */
  said: string
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

/**
 * Regulates the key to a key height and a key dip, millimetres, and says how
 * it came out. The engine lays the papers, or refuses and changes nothing.
 */
export function regulate(b: Bench, keyHeightMm: number, keyDipMm: number): Regulation | null {
  return JSON.parse(b.regulate(keyHeightMm, keyDipMm)) as Regulation | null
}
