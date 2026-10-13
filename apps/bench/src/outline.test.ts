import type { ContactState, MarkedContact } from '@pas/wasm/types'
import * as THREE from 'three'
import { describe, expect, it } from 'vitest'
import {
  axisSegments,
  contactStrokes,
  mirrored,
  poseMatrix,
  sharedSegments,
  strokes,
  toShape,
} from './outline'

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

describe('poseMatrix', () => {
  it('turns and shifts a point exactly as the pose says', () => {
    // A quarter turn counter-clockwise about an origin at (10, 20): the point
    // one along the body's x lands one up from the origin.
    const m = poseMatrix({ origin: [10, 20], cos: 0, sin: 1 })
    const p = new THREE.Vector3(1, 0, 0).applyMatrix4(m)
    expect([p.x, p.y, p.z]).toEqual([10, 21, 0])
  })

  it('places nothing anywhere with the identity pose', () => {
    const m = poseMatrix({ origin: [0, 0], cos: 1, sin: 0 })
    expect(m.equals(new THREE.Matrix4())).toBe(true)
  })
})

describe('mirrored', () => {
  // A washer's profile, 2 to 11 mm out from its axis, 2 mm thick.
  const profile = [2, 0, 11, 0, 11, 2, 2, 2]

  it('mirrors the radius and keeps the heights', () => {
    expect(mirrored(profile)).toEqual([-2, 2, -11, 2, -11, 0, -2, 0])
  })

  it('still turns counter-clockwise', () => {
    const area = (flat: number[]) => THREE.ShapeUtils.area(toShape(flat).getPoints())
    expect(area(profile)).toBeGreaterThan(0)
    expect(area(mirrored(profile))).toBeGreaterThan(0)
  })
})

describe('strokes and shared segments', () => {
  // A keystick's underside split at a slot's edges, and the slot cut up from
  // it: the slot's bottom (its segment 0) is the keystick's segment 1.
  const stick = { points: [0, 0, 4, 0, 6, 0, 10, 0, 10, 3, 0, 3], segs: [0, 1, 2, 3, 4, 5] }
  const slot = { points: [4, 0, 6, 0, 6, 2, 4, 2], segs: [0, 1, 2, 3] }

  it('finds the mouth from either side', () => {
    expect([...sharedSegments(slot, stick)]).toEqual([0])
    expect([...sharedSegments(stick, slot)]).toEqual([1])
  })

  it('leaves the skipped segments out of the line work', () => {
    const all = strokes(slot.points, slot.segs)
    const open = strokes(slot.points, slot.segs, new Set([0]))
    expect(all.length).toBe(16)
    expect(open).toEqual([6, 0, 6, 2, 6, 2, 4, 2, 4, 2, 4, 0])
  })

  it('reads every point of an arc as part of its segment', () => {
    // Segment 1 is an arc drawn through three points.
    const points = [0, 0, 2, 0, 3, 1, 2, 2, 0, 2]
    const segs = [0, 1, 1, 1, 2]
    expect(strokes(points, segs, new Set([1]))).toEqual([0, 0, 2, 0, 0, 2, 0, 0])
  })

  it('finds where a turned profile runs down its axis', () => {
    const pin = [0, 0, 1.7, 0, 1.7, 11, 0, 11]
    expect([...axisSegments(pin, [0, 1, 2, 3])]).toEqual([3])
  })
})

describe('contactStrokes', () => {
  const contact = (state: ContactState, at: number): MarkedContact => ({
    pair: 'back_rail_cloth',
    on_key: [at, 2],
    on_frame: [at, 1],
    gap_mm: state === 'overlapping' ? -1 : 1,
    state,
  })

  it('runs from the frame to the key, for the contacts in one state only', () => {
    const contacts = [contact('open', 10), contact('closed', 20), contact('open', 30)]
    expect(contactStrokes(contacts, 'open')).toEqual([10, 1, 10, 2, 30, 1, 30, 2])
    expect(contactStrokes(contacts, 'overlapping')).toEqual([])
  })
})
