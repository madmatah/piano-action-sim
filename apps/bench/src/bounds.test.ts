import type { Outline } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { measureBounds } from './bounds'

/** A rectangle of the key as the engine sends it, from `back` to `front` and
 * from `bottom` to `top`, millimeters, swept as `sweep` says. */
function buildOutline(
  [back, front, bottom, top]: [number, number, number, number],
  sweep: Pick<Outline, 'sweep' | 'width_mm' | 'across_mm' | 'axis_x_mm'>,
): Outline {
  return {
    body: 1,
    code: 'balance_mortise_bushing',
    role: 'lining',
    material: 'bushing_cloth',
    paper_mm: null,
    paper_color: null,
    is_in_the_section: false,
    points_mm: [back, bottom, front, bottom, front, top, back, top],
    segs: [0, 1, 2, 3],
    sources: [],
    ...sweep,
  }
}

describe('the bounds of an outline', () => {
  it('stands a lining set beside the plane where it stands across the keyboard', () => {
    // The measured section's bushing: 1.3 mm of cloth on the slot's treble
    // wall, its middle 2.35 mm from the plane.
    const bushing = buildOutline([-5.1, 5.1, 27.5, 30.5], {
      sweep: 'extruded',
      width_mm: 1.3,
      across_mm: -2.35,
      axis_x_mm: 0,
    })
    const { minMm, maxMm } = measureBounds(bushing)
    // One halving and one sum of millimeters under ten, each rounded once.
    const near = (found: number, expected: number) =>
      Math.abs(found - expected) <= 4 * Number.EPSILON * 10
    expect(near(minMm[2], -3) && near(maxMm[2], -1.7)).toBe(true)
  })

  it('bounds a milled cut as wide across the keyboard as its cutter', () => {
    // A 3 mm cutter moved 2 mm, 10 mm along the key, 2.5 mm deep, each
    // coordinate a short binary fraction, so that the bounds are exact.
    const [middleMm, cutterMm, travelMm, depthMm] = [10, 3, 2, 2.5]
    const [backMm, frontMm] = [
      middleMm - (travelMm + cutterMm) / 2,
      middleMm + (travelMm + cutterMm) / 2,
    ]
    const mortise = buildOutline([backMm, frontMm, 0, depthMm], {
      sweep: 'milled',
      width_mm: cutterMm,
      across_mm: 0,
      axis_x_mm: 0,
    })
    expect(measureBounds(mortise)).toEqual({
      minMm: [backMm, 0, -cutterMm / 2],
      maxMm: [frontMm, depthMm, cutterMm / 2],
    })
  })
})
