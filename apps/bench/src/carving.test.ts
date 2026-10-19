import type { Outline, Part } from '@pas/wasm/types'
import { ShaderLib } from 'three'
import { describe, expect, it } from 'vitest'
import {
  type Box,
  MAX_CUTS,
  measureBounds,
  overlaps,
  packCuts,
  patchShader,
  planCarving,
} from './carving'

// The measured section's key, millimeters (`data/profiles/measured-section.json`):
// a keystick 24.5 mm high and 22 mm wide, a key button 62 mm long and 6 mm
// high on it, its slot 10.2 mm long and 6 mm wide down to a 2.5 mm floor,
// the 3.4 mm balance hole through it, and the bushing 1.3 mm thick, 3 mm
// deep, set 2.35 mm either side of the sagittal plane; at its foot the
// cloth turns out through a 2.6 mm side hole on each side, 7 mm from the
// plane and 8 mm wide, from the slot's wall to the key button's side,
// glued against the hole's top.
const STICK_TOP_MM = 24.5
const KEY_WIDTH_MM = 22
const BUTTON_TOP_MM = 30.5
const SLOT_HALF_MM = 5.1
const SLOT_WIDTH_MM = 6
const FLOOR_MM = 2.5
const HOLE_HALF_MM = 1.7
const BUSHING_MM = 1.3
const BUSHING_ACROSS_MM = 2.35
const BUSHING_FOOT_MM = BUTTON_TOP_MM - 3
const SIDE_HOLE_TOP_MM = BUSHING_FOOT_MM + BUSHING_MM
const SIDE_HOLE_FOOT_MM = SIDE_HOLE_TOP_MM - 2.6
const SIDE_ACROSS_MM = 7
const SIDE_WIDTH_MM = 8

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

const SLOT = buildOutline(
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
)
const HOLE = buildOutline(
  'balance_hole',
  'cut',
  [-HOLE_HALF_MM, 0, HOLE_HALF_MM, 0, HOLE_HALF_MM, FLOOR_MM, -HOLE_HALF_MM, FLOOR_MM],
  2 * HOLE_HALF_MM,
)
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
/** A rectangle along the whole slot, from `bottom` to `top`. */
function alongTheSlot(bottom: number, top: number): number[] {
  return [-SLOT_HALF_MM, bottom, SLOT_HALF_MM, bottom, SLOT_HALF_MM, top, -SLOT_HALF_MM, top]
}
const DOWN_THE_WALLS = [-BUSHING_ACROSS_MM, BUSHING_ACROSS_MM].map((across) =>
  buildOutline(
    'balance_mortise_bushing',
    'lining',
    alongTheSlot(BUSHING_FOOT_MM, BUTTON_TOP_MM),
    BUSHING_MM,
    across,
  ),
)
const SIDE_HOLES = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) =>
  buildOutline(
    'key_button_side_hole',
    'cut',
    alongTheSlot(SIDE_HOLE_FOOT_MM, SIDE_HOLE_TOP_MM),
    SIDE_WIDTH_MM,
    across,
  ),
)
const OUT_THROUGH_THE_SIDES = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) =>
  buildOutline(
    'balance_mortise_bushing',
    'lining',
    alongTheSlot(BUSHING_FOOT_MM, SIDE_HOLE_TOP_MM),
    SIDE_WIDTH_MM,
    across,
  ),
)
const KEY = [
  SLOT,
  HOLE,
  ...SIDE_HOLES,
  KEYSTICK,
  KEY_BUTTON,
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
const SIDE_HOLE_BOXES: Box[] = [-SIDE_ACROSS_MM, SIDE_ACROSS_MM].map((across) => ({
  minMm: [-SLOT_HALF_MM, SIDE_HOLE_FOOT_MM, across - SIDE_WIDTH_MM / 2],
  maxMm: [SLOT_HALF_MM, SIDE_HOLE_TOP_MM, across + SIDE_WIDTH_MM / 2],
}))

describe('the carving of the solid', () => {
  it('carves a cut with a width as the box its outline bounds, across that width', () => {
    expect(planCarving([SLOT]).walls.map((walls) => walls.box)).toEqual([SLOT_BOX])
  })

  it('carves only a cut that has a width', () => {
    const frontMortise = buildOutline('front_mortise', 'cut', [221, 0, 229, 0, 229, 22, 221, 22], 0)
    expect(planCarving([frontMortise, KEYSTICK]).walls).toEqual([])
  })

  it('leaves uncarved a cut whose outline is no rectangle, which its box would misdraw', () => {
    const notched = [0, 0, 10, 0, 10, 5, 5, 5, 5, 10, 0, 10]
    const trapezoid = [0, 0, 10, 0, 8, 10, 2, 10]
    const chamfered = [0, 0, 10, 0, 10, 8, 8, 10, 0, 10]
    for (const points of [notched, trapezoid, chamfered]) {
      expect(planCarving([buildOutline('balance_mortise', 'cut', points, 4)]).walls).toEqual([])
    }
  })

  it('carves the keystick by the slot and the balance hole, and never the bushing standing in the slot', () => {
    const { cutsThrough } = planCarving(KEY)
    expect(cutsThrough.get(KEYSTICK)).toEqual([SLOT_BOX, HOLE_BOX])
    for (const leg of DOWN_THE_WALLS) expect(cutsThrough.has(leg)).toBe(false)
  })

  it('carves the key button by the slot and its two side holes, and never the cloth through them', () => {
    const { cutsThrough } = planCarving(KEY)
    expect(cutsThrough.get(KEY_BUTTON)).toEqual([SLOT_BOX, ...SIDE_HOLE_BOXES])
    for (const leg of OUT_THROUGH_THE_SIDES) expect(cutsThrough.has(leg)).toBe(false)
  })

  it('draws the slot’s walls in the wood it runs through, open into the balance hole and the side holes', () => {
    const slot = planCarving(KEY).walls.find((walls) => walls.cut === SLOT)
    expect(slot?.through).toEqual([KEYSTICK, KEY_BUTTON])
    expect(slot?.into).toEqual([HOLE_BOX, ...SIDE_HOLE_BOXES])
  })

  it('draws each side hole’s walls in the key button, open into the slot', () => {
    for (const sideHole of SIDE_HOLES) {
      const walls = planCarving(KEY).walls.find((planned) => planned.cut === sideHole)
      expect([walls?.through, walls?.into]).toEqual([[KEY_BUTTON], [SLOT_BOX]])
    }
  })

  it('stands a lining set beside the plane where it stands across the keyboard', () => {
    const { minMm, maxMm } = measureBounds(DOWN_THE_WALLS[0] as Outline)
    // One halving and one sum of millimeters under ten, each rounded once.
    const near = (value: number, expected: number) =>
      Math.abs(value - expected) <= 4 * Number.EPSILON * 10
    expect(near(minMm[2], -3) && near(maxMm[2], -1.7)).toBe(true)
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
        alongTheSlot(SIDE_HOLE_FOOT_MM, SIDE_HOLE_TOP_MM),
        Math.fround(7.95),
        towardTheBass * Math.fround(7.025),
      ),
    )
    const { walls } = planCarving([slot, ...sideHoles])
    expect(walls.map((planned) => planned.into.length)).toEqual([2, 1, 1])
  })

  it('tells a box that shares a volume from one that only touches it', () => {
    const keyButton: Box = { minMm: [-31, 24.5, -11], maxMm: [31, 30.5, 11] }
    const above: Box = { minMm: [-31, 30.5, -11], maxMm: [31, 40, 11] }
    expect([overlaps(SLOT_BOX, keyButton), overlaps(SLOT_BOX, above)]).toEqual([true, false])
  })

  it('refuses more cuts through one part than the shader carves', () => {
    expect(() =>
      packCuts(
        Array.from({ length: MAX_CUTS + 1 }, () => SLOT_BOX),
        null,
      ),
    ).toThrow()
  })

  it('patches three’s Lambert shader where it reads positions and clips', () => {
    const { vertexShader, fragmentShader } = ShaderLib.lambert
    const { vertex, fragment } = patchShader(vertexShader, fragmentShader)
    expect([vertex.includes('vCarve = transformed'), fragment.includes('discard')]).toEqual([
      true,
      true,
    ])
  })
})
