import type { Outline, Part } from '@pas/wasm/types'
import { type Box3, ShaderLib } from 'three'
import { describe, expect, it } from 'vitest'
import type { Box } from './bounds'
import {
  buildCarvingUniforms,
  type Hollow,
  MAX_CUTS,
  millWalls,
  overlaps,
  packCuts,
  patchShader,
  planCarving,
  writeCarving,
} from './carving'

// The measured section's key, millimeters (`data/profiles/measured-section.json`):
// a keystick 24.5 mm high and 22 mm wide, a key button 62 mm long and 6 mm
// high on it, its slot milled 10.2 mm long by a 6 mm cutter moved 4.2 mm,
// down to a 2.5 mm floor, the 3.4 mm balance hole milled through it, and
// the bushing 1.3 mm thick, 3 mm deep, along the slot's straight walls, set
// 2.35 mm either side of the sagittal plane; at its foot the cloth turns
// out through a 2.6 mm side hole on each side, 7 mm from the plane and 8 mm
// wide, from the slot's wall to the key button's side, glued against the
// hole's top. The front mortise is milled 8 mm long by a 6 mm cutter, 22 mm
// up from the underside, 221 to 229 mm in front of the balance pin, through
// a groove 3 mm deep across the key along its straight walls, 224 to 226 mm,
// each side of it from its wall, 3 mm from the plane, to the key's side; its
// bushing runs up each wall 3 mm past the groove's floor, then 5 mm flat on
// that floor, glued against it.
const STICK_TOP_MM = 24.5
const KEY_WIDTH_MM = 22
const BUTTON_TOP_MM = 30.5
const SLOT_HALF_MM = 5.1
const SLOT_WIDTH_MM = 6
const WALL_HALF_MM = 2.1
const FLOOR_MM = 2.5
const HOLE_HALF_MM = 1.7
const BUSHING_MM = 1.3
const BUSHING_ACROSS_MM = 2.35
const BUSHING_FOOT_MM = BUTTON_TOP_MM - 3
const SIDE_HOLE_TOP_MM = BUSHING_FOOT_MM + BUSHING_MM
const SIDE_HOLE_FOOT_MM = SIDE_HOLE_TOP_MM - 2.6
const SIDE_ACROSS_MM = 7
const SIDE_WIDTH_MM = 8
const GROOVE_TOP_MM = 3
const FRONT_FOOT_MM = GROOVE_TOP_MM - BUSHING_MM
const FLAT_ACROSS_MM = 5.5
const FLAT_WIDTH_MM = 5

/** An extruded part of a key, as the engine sends it. */
function buildOutline(
  code: Part,
  role: Outline['role'],
  points_mm: number[],
  width_mm: number,
  across_mm = 0,
): Outline {
  return {
    body: 1,
    code,
    role,
    material: role === 'lining' ? 'bushing_cloth' : 'key_wood',
    paper_mm: null,
    paper_color: null,
    sweep: 'extruded',
    width_mm,
    across_mm,
    is_in_the_section: across_mm === 0,
    axis_x_mm: 0,
    points_mm,
    segs: points_mm.filter((_, index) => index % 2 === 0).map((_, segment) => segment),
    sources: [],
  }
}

const SLOT: Outline = {
  ...buildOutline(
    'balance_mortise',
    'cut',
    [
      -SLOT_HALF_MM,
      FLOOR_MM,
      -HOLE_HALF_MM,
      FLOOR_MM,
      HOLE_HALF_MM,
      FLOOR_MM,
      SLOT_HALF_MM,
      FLOOR_MM,
      SLOT_HALF_MM,
      BUTTON_TOP_MM,
      -SLOT_HALF_MM,
      BUTTON_TOP_MM,
    ],
    SLOT_WIDTH_MM,
  ),
  sweep: 'milled',
}
const HOLE: Outline = {
  ...buildOutline(
    'balance_hole',
    'cut',
    [-HOLE_HALF_MM, 0, HOLE_HALF_MM, 0, HOLE_HALF_MM, FLOOR_MM, -HOLE_HALF_MM, FLOOR_MM],
    2 * HOLE_HALF_MM,
  ),
  sweep: 'milled',
}
const KEYSTICK = buildOutline(
  'keystick',
  'solid',
  [-240, 0, 258, 0, 258, STICK_TOP_MM, -240, STICK_TOP_MM],
  KEY_WIDTH_MM,
)
const KEY_BUTTON = buildOutline(
  'key_button',
  'solid',
  [-31, STICK_TOP_MM, 31, STICK_TOP_MM, 31, BUTTON_TOP_MM, -31, BUTTON_TOP_MM],
  KEY_WIDTH_MM,
)
/** A rectangle along the slot's straight walls, from `bottom` to `top`. */
function alongTheWalls(bottom: number, top: number): number[] {
  return [-WALL_HALF_MM, bottom, WALL_HALF_MM, bottom, WALL_HALF_MM, top, -WALL_HALF_MM, top]
}
const FRONT_MORTISE: Outline = {
  ...buildOutline('front_mortise', 'cut', [221, 0, 229, 0, 229, 22, 221, 22], SLOT_WIDTH_MM),
  sweep: 'milled',
}
/** A rectangle along the front mortise's straight walls, from `bottom` to
 * `top`. */
function alongTheFrontWalls(bottom: number, top: number): number[] {
  return [224, bottom, 226, bottom, 226, top, 224, top]
}
const GROOVES = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) =>
  buildOutline(
    'front_mortise_groove',
    'cut',
    alongTheFrontWalls(0, GROOVE_TOP_MM),
    SIDE_WIDTH_MM,
    across,
  ),
)
const UP_THE_FRONT_WALLS = [-BUSHING_ACROSS_MM, BUSHING_ACROSS_MM].map((across) =>
  buildOutline(
    'front_mortise_bushing',
    'lining',
    alongTheFrontWalls(FRONT_FOOT_MM, GROOVE_TOP_MM + 3),
    BUSHING_MM,
    across,
  ),
)
const FLAT_IN_THE_GROOVE = [-FLAT_ACROSS_MM, FLAT_ACROSS_MM].map((across) =>
  buildOutline(
    'front_mortise_bushing',
    'lining',
    alongTheFrontWalls(FRONT_FOOT_MM, GROOVE_TOP_MM),
    FLAT_WIDTH_MM,
    across,
  ),
)
const DOWN_THE_WALLS = [-BUSHING_ACROSS_MM, BUSHING_ACROSS_MM].map((across) =>
  buildOutline(
    'balance_mortise_bushing',
    'lining',
    alongTheWalls(BUSHING_FOOT_MM, BUTTON_TOP_MM),
    BUSHING_MM,
    across,
  ),
)
const SIDE_HOLES = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) =>
  buildOutline(
    'key_button_side_hole',
    'cut',
    alongTheWalls(SIDE_HOLE_FOOT_MM, SIDE_HOLE_TOP_MM),
    SIDE_WIDTH_MM,
    across,
  ),
)
const OUT_THROUGH_THE_SIDES = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) =>
  buildOutline(
    'balance_mortise_bushing',
    'lining',
    alongTheWalls(BUSHING_FOOT_MM, SIDE_HOLE_TOP_MM),
    SIDE_WIDTH_MM,
    across,
  ),
)
const KEY = [
  SLOT,
  HOLE,
  FRONT_MORTISE,
  ...GROOVES,
  ...SIDE_HOLES,
  KEYSTICK,
  KEY_BUTTON,
  ...UP_THE_FRONT_WALLS,
  ...FLAT_IN_THE_GROOVE,
  ...DOWN_THE_WALLS,
  ...OUT_THROUGH_THE_SIDES,
]

const SLOT_BOX: Box = {
  minMm: [-SLOT_HALF_MM, FLOOR_MM, -SLOT_WIDTH_MM / 2],
  maxMm: [SLOT_HALF_MM, BUTTON_TOP_MM, SLOT_WIDTH_MM / 2],
}
const HOLE_BOX: Box = {
  minMm: [-HOLE_HALF_MM, 0, -HOLE_HALF_MM],
  maxMm: [HOLE_HALF_MM, FLOOR_MM, HOLE_HALF_MM],
}
const FRONT_MORTISE_BOX: Box = {
  minMm: [221, 0, -SLOT_WIDTH_MM / 2],
  maxMm: [229, 22, SLOT_WIDTH_MM / 2],
}
const SIDE_HOLE_BOXES: Box[] = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) => ({
  minMm: [-WALL_HALF_MM, SIDE_HOLE_FOOT_MM, across - SIDE_WIDTH_MM / 2],
  maxMm: [WALL_HALF_MM, SIDE_HOLE_TOP_MM, across + SIDE_WIDTH_MM / 2],
}))
const GROOVE_BOXES: Box[] = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) => ({
  minMm: [224, 0, across - SIDE_WIDTH_MM / 2],
  maxMm: [226, GROOVE_TOP_MM, across + SIDE_WIDTH_MM / 2],
}))
const KEY_BUTTON_BOX: Box = {
  minMm: [-31, STICK_TOP_MM, -KEY_WIDTH_MM / 2],
  maxMm: [31, BUTTON_TOP_MM, KEY_WIDTH_MM / 2],
}
const SLOT_HOLLOW: Hollow = { box: SLOT_BOX, hasRoundEnds: true }
const HOLE_HOLLOW: Hollow = { box: HOLE_BOX, hasRoundEnds: true }
const FRONT_MORTISE_HOLLOW: Hollow = { box: FRONT_MORTISE_BOX, hasRoundEnds: true }
const SIDE_HOLE_HOLLOWS: Hollow[] = SIDE_HOLE_BOXES.map((box) => ({ box, hasRoundEnds: false }))
const GROOVE_HOLLOWS: Hollow[] = GROOVE_BOXES.map((box) => ({ box, hasRoundEnds: false }))

describe('the carving of the solid', () => {
  it('carves an extruded cut with a width as the box its outline bounds, across that width', () => {
    const sideHoles = planCarving(SIDE_HOLES).walls.map((walls) => walls.hollow)
    expect(sideHoles).toEqual(SIDE_HOLE_HOLLOWS)
  })

  it('carves a milled mortise in the box its outline bounds across its cutter, its ends rounded', () => {
    expect(planCarving([SLOT]).walls.map((walls) => walls.hollow)).toEqual([SLOT_HOLLOW])
  })

  it('carves a milled hole as long as its cutter is wide, its rounded ends making it round', () => {
    expect(planCarving([HOLE]).walls.map((walls) => walls.hollow)).toEqual([HOLE_HOLLOW])
  })

  it('carves only a cut that has a width, extruded or milled', () => {
    const noWidth = { ...FRONT_MORTISE, width_mm: 0 }
    for (const sweep of ['extruded', 'milled'] as const) {
      expect(planCarving([{ ...noWidth, sweep }, KEYSTICK]).walls).toEqual([])
    }
  })

  it('leaves uncarved a cut whose outline is no rectangle, which its box would misdraw', () => {
    const notched = [0, 0, 10, 0, 10, 5, 5, 5, 5, 10, 0, 10]
    const trapezoid = [0, 0, 10, 0, 8, 10, 2, 10]
    const chamfered = [0, 0, 10, 0, 10, 8, 8, 10, 0, 10]
    for (const points of [notched, trapezoid, chamfered]) {
      expect(planCarving([buildOutline('balance_mortise', 'cut', points, 4)]).walls).toEqual([])
    }
  })

  it('carves the keystick by the slot, the balance hole, the front mortise and its groove, and never a bushing standing in them', () => {
    const { cutsThrough } = planCarving(KEY)
    expect(cutsThrough.get(KEYSTICK)).toEqual([
      SLOT_HOLLOW,
      HOLE_HOLLOW,
      FRONT_MORTISE_HOLLOW,
      ...GROOVE_HOLLOWS,
    ])
    for (const leg of [...DOWN_THE_WALLS, ...UP_THE_FRONT_WALLS, ...FLAT_IN_THE_GROOVE]) {
      expect(cutsThrough.has(leg)).toBe(false)
    }
  })

  it('draws the front mortise’s walls and its groove’s in the keystick, each open into the other', () => {
    const { walls } = planCarving(KEY)
    const mortise = walls.find((planned) => planned.cut === FRONT_MORTISE)
    expect([mortise?.through, mortise?.into]).toEqual([[KEYSTICK], GROOVE_HOLLOWS])
    for (const groove of GROOVES) {
      const planned = walls.find((each) => each.cut === groove)
      expect([planned?.through, planned?.into]).toEqual([[KEYSTICK], [FRONT_MORTISE_HOLLOW]])
    }
  })

  it('carves the key button by the slot and its two side holes, and never the cloth through them', () => {
    const { cutsThrough } = planCarving(KEY)
    expect(cutsThrough.get(KEY_BUTTON)).toEqual([SLOT_HOLLOW, ...SIDE_HOLE_HOLLOWS])
    for (const leg of OUT_THROUGH_THE_SIDES) expect(cutsThrough.has(leg)).toBe(false)
  })

  it('draws the slot’s walls in the wood it runs through, open into the balance hole and the side holes', () => {
    const slot = planCarving(KEY).walls.find((walls) => walls.cut === SLOT)
    expect(slot?.through).toEqual([KEYSTICK, KEY_BUTTON])
    expect(slot?.into).toEqual([HOLE_HOLLOW, ...SIDE_HOLE_HOLLOWS])
  })

  it('draws each side hole’s walls in the key button, open into the slot', () => {
    for (const sideHole of SIDE_HOLES) {
      const walls = planCarving(KEY).walls.find((planned) => planned.cut === sideHole)
      expect([walls?.through, walls?.into]).toEqual([[KEY_BUTTON], [SLOT_HOLLOW]])
    }
  })

  it('opens the slot into side holes whose faces single precision rounds off its walls', () => {
    // A 6.1 mm slot in a 22 mm key: each side hole 7.95 mm wide, 7.025 mm
    // from the plane, its inner face 0.0000002 mm outside the slot's wall
    // once each number is rounded to single precision, as the engine sends
    // it.
    const slotWidthMm = Math.fround(6.1)
    const slot = buildOutline('balance_mortise', 'cut', SLOT.points_mm, slotWidthMm)
    const sideHoles = [-1, 1].map((towardTheBass) =>
      buildOutline(
        'key_button_side_hole',
        'cut',
        alongTheWalls(SIDE_HOLE_FOOT_MM, SIDE_HOLE_TOP_MM),
        Math.fround(7.95),
        towardTheBass * Math.fround(7.025),
      ),
    )
    const { walls } = planCarving([slot, ...sideHoles])
    expect(walls.map((planned) => planned.into.length)).toEqual([2, 1, 1])
  })

  it('draws a milled cut’s walls on its plan, its box less the corners beyond its cutter', () => {
    const walls = millWalls(SLOT_BOX, 48)
    walls.computeBoundingBox()
    const { min, max } = walls.boundingBox as Box3
    // A few roundings of millimeters under forty in three's single precision.
    const near = (found: number, expected: number) => Math.abs(found - expected) <= 1e-5
    expect(
      [min.x, min.y, min.z, max.x, max.y, max.z].every((found, index) =>
        near(found, [...SLOT_BOX.minMm, ...SLOT_BOX.maxMm][index] as number),
      ),
    ).toBe(true)
    const position = walls.getAttribute('position')
    const radiusMm = SLOT_WIDTH_MM / 2
    let farthestMm = Number.NEGATIVE_INFINITY
    for (let vertex = 0; vertex < position.count; vertex += 1) {
      const pastTheWallMm = Math.max(Math.abs(position.getX(vertex)) - WALL_HALF_MM, 0)
      farthestMm = Math.max(farthestMm, Math.hypot(pastTheWallMm, position.getZ(vertex)) - radiusMm)
    }
    expect(near(farthestMm, 0)).toBe(true)
  })

  it('tells a box that shares a volume from one that only touches it', () => {
    const keyButton: Box = { minMm: [-31, 24.5, -11], maxMm: [31, 30.5, 11] }
    const above: Box = { minMm: [-31, 30.5, -11], maxMm: [31, 40, 11] }
    expect([overlaps(SLOT_BOX, keyButton), overlaps(SLOT_BOX, above)]).toEqual([true, false])
  })

  it('refuses more cuts through one part than the shader carves', () => {
    expect(() =>
      packCuts(
        Array.from({ length: MAX_CUTS + 1 }, () => SLOT_HOLLOW),
        null,
      ),
    ).toThrow()
  })

  it('keeps a part nothing cuts whole', () => {
    // A meter each way, farther than any part of a key on its frame stands
    // from its body's origin.
    const pastEveryPartMm = 1000
    const { cutCount, keepMin, keepMax } = packCuts([], null)
    expect(cutCount).toBe(0)
    expect(keepMin.every((coordinate) => coordinate < -pastEveryPartMm)).toBe(true)
    expect(keepMax.every((coordinate) => coordinate > pastEveryPartMm)).toBe(true)
  })

  it('tells the shader which cuts have their ends rounded', () => {
    const { fragmentShader } = ShaderLib.lambert
    const { fragment } = patchShader('', fragmentShader)
    expect(fragment).toContain('uniform bool cutHasRoundEnds[MAX_CUTS];')
    expect(fragment).toContain(
      '!cutHasRoundEnds[cutIndex] || measureBeyondTheCutter(vCarve, low, high)',
    )
    const [sideHole] = SIDE_HOLE_HOLLOWS as [Hollow]
    expect(packCuts([SLOT_HOLLOW, sideHole, HOLE_HOLLOW], null).cutHasRoundEnds).toEqual([
      true,
      false,
      true,
      ...Array.from({ length: MAX_CUTS - 3 }, () => false),
    ])
  })

  it('writes a part’s packed cuts into its uniforms in place of the cuts they held', () => {
    const uniforms = buildCarvingUniforms()
    writeCarving(uniforms, packCuts([SLOT_HOLLOW, HOLE_HOLLOW, ...SIDE_HOLE_HOLLOWS], null))
    writeCarving(uniforms, packCuts([SLOT_HOLLOW], KEY_BUTTON_BOX))
    const unused = Array.from({ length: MAX_CUTS - 1 }, () => [0, 0, 0])
    expect({
      cutMin: uniforms.cutMin.value.map((corner) => corner.toArray()),
      cutMax: uniforms.cutMax.value.map((corner) => corner.toArray()),
      cutHasRoundEnds: uniforms.cutHasRoundEnds.value,
      cutCount: uniforms.cutCount.value,
      keepMin: uniforms.keepMin.value.toArray(),
      keepMax: uniforms.keepMax.value.toArray(),
    }).toEqual({
      cutMin: [SLOT_BOX.minMm, ...unused],
      cutMax: [SLOT_BOX.maxMm, ...unused],
      cutHasRoundEnds: [1, ...Array.from({ length: MAX_CUTS - 1 }, () => 0)],
      cutCount: 1,
      keepMin: KEY_BUTTON_BOX.minMm,
      keepMax: KEY_BUTTON_BOX.maxMm,
    })
  })

  it('patches three’s Lambert shader where it reads positions and clips', () => {
    const { vertexShader, fragmentShader } = ShaderLib.lambert
    const { vertex, fragment } = patchShader(vertexShader, fragmentShader)
    expect(vertex).toContain('#include <begin_vertex>\n\tvCarve = transformed;')
    expect(fragment).toContain('#include <clipping_planes_fragment>\n\tif (any(lessThan(vCarve')
  })
})
