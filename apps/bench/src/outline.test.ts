import * as THREE from 'three'
import { describe, expect, it } from 'vitest'
import { toShape } from './outline'

describe('toShape', () => {
  // An L-shaped outline as the engine sends it: flat x, y pairs, millimetres.
  // Not symmetric, so a swapped pair or a wrong stride changes the answer.
  const flat = [0, 0, 30, 0, 30, 4, 10, 4, 10, 12, 0, 12]

  it('keeps every vertex, in order, and closes the outline', () => {
    const points = toShape(flat)
      .getPoints()
      .map((p) => [p.x, p.y])
    expect(points).toEqual([
      [0, 0],
      [30, 0],
      [30, 4],
      [10, 4],
      [10, 12],
      [0, 12],
      [0, 0],
    ])
  })

  it('encloses the area of the outline, counter-clockwise', () => {
    // 30 × 4 plus 10 × 8. Reading the pairs as (y, x) would flip the sign.
    expect(THREE.ShapeUtils.area(toShape(flat).getPoints())).toBe(200)
  })

  it('refuses a buffer that is not a whole number of points', () => {
    // An odd length means the wasm boundary is wrong, not that the part is
    // unusual. Drawing half of it would be worse than stopping.
    expect(() => toShape([0, 0, 1, 1, 2])).toThrow(/not a polygon/)
  })

  it('refuses fewer than three points', () => {
    expect(() => toShape([0, 0, 1, 1])).toThrow(/not a polygon/)
  })
})
