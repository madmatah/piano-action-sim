import type { Outline, Part } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { divideTheSection, isSeenBeyondThePlane } from './section'

// The measured section's key button over the balance mortise, millimeters
// (`data/profiles/measured-section.json`): a key 22 mm wide, its key button
// 62 mm long, from 24.5 to 30.5 mm above the underside, the slot 10.2 mm long and 6 mm
// wide down to a 2.5 mm floor; its bushing 1.3 mm thick and 3 mm deep, an L
// on each side, down the slot's wall, then out through the key button's
// side hole to its side, glued against the top of the 2.6 mm hole.
const KEY_WIDTH_MM = 22
const BUTTON_HALF_MM = 31
const STICK_TOP_MM = 24.5
const BUTTON_TOP_MM = 30.5
const SLOT_HALF_MM = 5.1
const SLOT_WIDTH_MM = 6
const FLOOR_MM = 2.5
const BUSHING_MM = 1.3
const BUSHING_FOOT_MM = BUTTON_TOP_MM - 3
const SIDE_HOLE_TOP_MM = BUSHING_FOOT_MM + BUSHING_MM
const SIDE_HOLE_FOOT_MM = SIDE_HOLE_TOP_MM - 2.6

/** A part of the key as the engine sends it, a rectangle from `back` to
 * `front` and from `bottom` to `top`, swept `width_mm` across the keyboard
 * about `across_mm`. */
function buildPart(
  code: Part,
  role: Outline['role'],
  [back, front, bottom, top]: [number, number, number, number],
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
    is_in_the_section: across_mm === 0 || 2 * Math.abs(across_mm) < width_mm,
    axis_x_mm: 0,
    points_mm: [back, bottom, front, bottom, front, top, back, top],
    segs: [0, 1, 2, 3],
    sources: [],
  }
}

/** The key button, its slot and the parts beside the plane, for a key
 * `keyWidthMm` wide and a slot `slotWidthMm` wide, each width as the engine
 * builds it from the two. */
function buildKeyButton(keyWidthMm: number, slotWidthMm: number) {
  const along = [-SLOT_HALF_MM, SLOT_HALF_MM] as const
  const sideMm = (keyWidthMm - slotWidthMm) / 2
  const sideAcrossMm = (keyWidthMm + slotWidthMm) / 4
  const wallAcrossMm = (slotWidthMm - BUSHING_MM) / 2
  const slot = buildPart('balance_mortise', 'cut', [...along, FLOOR_MM, BUTTON_TOP_MM], slotWidthMm)
  const keyButton = buildPart(
    'key_button',
    'solid',
    [-BUTTON_HALF_MM, BUTTON_HALF_MM, STICK_TOP_MM, BUTTON_TOP_MM],
    keyWidthMm,
  )
  const sideHoles = [-sideAcrossMm, sideAcrossMm].map((across) =>
    buildPart(
      'key_button_side_hole',
      'cut',
      [...along, SIDE_HOLE_FOOT_MM, SIDE_HOLE_TOP_MM],
      sideMm,
      across,
    ),
  )
  const downTheWalls = [-wallAcrossMm, wallAcrossMm].map((across) =>
    buildPart(
      'balance_mortise_bushing',
      'lining',
      [...along, BUSHING_FOOT_MM, BUTTON_TOP_MM],
      BUSHING_MM,
      across,
    ),
  )
  const outThroughTheSides = [-sideAcrossMm, sideAcrossMm].map((across) =>
    buildPart(
      'balance_mortise_bushing',
      'lining',
      [...along, BUSHING_FOOT_MM, SIDE_HOLE_TOP_MM],
      sideMm,
      across,
    ),
  )
  return { slot, keyButton, sideHoles, downTheWalls, outThroughTheSides }
}

describe('the section', () => {
  const { slot, keyButton, sideHoles, downTheWalls, outThroughTheSides } = buildKeyButton(
    KEY_WIDTH_MM,
    SLOT_WIDTH_MM,
  )
  const outlines = [slot, keyButton, ...sideHoles, ...downTheWalls, ...outThroughTheSides]

  it('draws the voids its plane cuts, and not the side holes beside it', () => {
    expect(divideTheSection(outlines).cuts).toEqual([slot])
  })

  it('draws the parts its plane cuts in it', () => {
    expect(divideTheSection(outlines).inPlane).toEqual([keyButton])
  })

  it('sees the bushing down the far wall beyond the plane, and not its leg out through the side hole', () => {
    expect(divideTheSection(outlines).beyond).toEqual([downTheWalls[0]])
  })

  it('keeps the leg out through the side hole hidden where single precision rounds its face past the wall', () => {
    // A 22 mm key and a 6.3 mm slot: the leg's near face, at 7.075 less
    // 7.85 halved, arrives 0.0000002 mm nearer the plane than the slot's
    // wall at 6.3 halved, each width rounded to single precision as the
    // engine sends it.
    const roundToSingle = (outline: Outline): Outline => ({
      ...outline,
      width_mm: Math.fround(outline.width_mm),
      across_mm: Math.fround(outline.across_mm),
    })
    const narrow = buildKeyButton(KEY_WIDTH_MM, 6.3)
    const leg = roundToSingle(narrow.outThroughTheSides[0] as Outline)
    const wall = roundToSingle(narrow.slot)
    // A guard on the fixture: it still rounds the leg's face past the wall.
    expect(leg.across_mm + leg.width_mm / 2).toBeGreaterThan(-wall.width_mm / 2)
    expect(isSeenBeyondThePlane(leg, [wall])).toBe(false)
  })

  it('sees nothing through a void that stands elsewhere along the key', () => {
    const behind = buildPart(
      'balance_mortise_bushing',
      'lining',
      [-120, -110, BUSHING_FOOT_MM, BUTTON_TOP_MM],
      BUSHING_MM,
      -(SLOT_WIDTH_MM - BUSHING_MM) / 2,
    )
    expect(isSeenBeyondThePlane(behind, [slot])).toBe(false)
  })
})
