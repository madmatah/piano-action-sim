import type { Box3 } from 'three'
import { describe, expect, it } from 'vitest'
import type { Box } from './bounds'
import { sweepOval } from './oval'

// The measured frame's front rail pin, millimeters
// (`data/params/key-frame-sections.json`): an oval head 5 mm along the key and
// 3.4 mm across the keyboard, standing 24 mm proud of the rail; 225 mm in
// front of the balance pin, where the measured section's profile places its
// rail by rule (`data/profiles/measured-section.json`, `front_rail.x`), under
// the key front it assumes.
const LONG_AXIS_MM = 5
const SHORT_AXIS_MM = 3.4
const MIDDLE_MM = 225
const HEAD_MM = 24
// The solid's faces round a whole oval, a multiple of four, so that a
// vertex stands at each end of each axis.
const FACES_ROUND = 48
const PIN: Box = {
  minMm: [MIDDLE_MM - LONG_AXIS_MM / 2, 0, -SHORT_AXIS_MM / 2],
  maxMm: [MIDDLE_MM + LONG_AXIS_MM / 2, HEAD_MM, SHORT_AXIS_MM / 2],
}

describe('an oval part', () => {
  it('fills the box its outline bounds, along the key, up and across the keyboard', () => {
    const solid = sweepOval(PIN, FACES_ROUND)
    solid.computeBoundingBox()
    const { min, max } = solid.boundingBox as Box3
    // Millimeters under three hundred in three's single precision.
    const near = (found: number, expected: number) => Math.abs(found - expected) <= 1e-4
    expect(
      [min.x, min.y, min.z, max.x, max.y, max.z].every((found, index) =>
        near(found, [...PIN.minMm, ...PIN.maxMm][index] as number),
      ),
    ).toBe(true)
  })

  it('has for its plan the ellipse its axes make, drawn through as many points as it has faces', () => {
    const position = sweepOval(PIN, FACES_ROUND).getAttribute('position')
    const plan = new Set<string>()
    let farthest = Number.NEGATIVE_INFINITY
    for (let vertex = 0; vertex < position.count; vertex += 1) {
      const [x, z] = [position.getX(vertex), position.getZ(vertex)]
      plan.add(`${x.toFixed(4)} ${z.toFixed(4)}`)
      const along = (x - MIDDLE_MM) / (LONG_AXIS_MM / 2)
      const across = z / (SHORT_AXIS_MM / 2)
      farthest = Math.max(farthest, along * along + across * across)
    }
    // x is rounded once to single precision near 227.5 mm, within 8e-6 mm:
    // over the 2.5 mm half-axis, and doubled by the square, about 6e-6.
    expect(Math.abs(farthest - 1)).toBeLessThanOrEqual(1e-5)
    expect(plan.size).toBe(FACES_ROUND)
  })
})
