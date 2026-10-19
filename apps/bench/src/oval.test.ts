import type { Box3, BufferGeometry } from 'three'
import { describe, expect, it } from 'vitest'
import { turnOval } from './oval'

// The measured frame's front rail pin, millimeters
// (`data/params/key-frame-sections.json`): an oval head 5 mm along the key and
// 3.4 mm across the keyboard, standing 24 mm proud of the rail; 225 mm in
// front of the balance pin, where the measured section's profile places its
// rail by rule (`data/profiles/measured-section.json`, `front_rail.x`), under
// the key front it assumes. Its tip is a half sphere of half the pin's length
// along the key.
const LONG_AXIS_MM = 5
const SHORT_AXIS_MM = 3.4
const MIDDLE_MM = 225
const HEAD_MM = 24
const TIP_RADIUS_MM = LONG_AXIS_MM / 2
// A pin a millimeter tall is all tip, a cap of that sphere, whose foot
// reaches √(2 · 2.5 · 1 − 1²) = 2 mm out from its middle each side; the
// engine sends it as much narrower across at its foot, 3.4 · 2 / 2.5 mm.
const CAP_MM = 1
const CAP_REACH_MM = 2
const CAP_WIDTH_MM = (SHORT_AXIS_MM * CAP_REACH_MM) / TIP_RADIUS_MM
// The solid's faces round a whole oval, a multiple of four, so that a
// vertex stands at each end of each axis.
const FACES_ROUND = 48
// Each quarter of the tip cut into lines, as the engine sends an arc.
const TIP_LINES = 8

/** Traces a pin's section along the key, `headMm` tall, as the engine sends
 * it: counter-clockwise from its back foot, each segment by its first point,
 * its tip's two arcs cut into lines, the sides it has below its tip
 * straight. */
function traceThePin(headMm: number): number[] {
  const centerMm = headMm - TIP_RADIUS_MM
  const hasSides = centerMm > 0
  const fromAngle = hasSides ? 0 : Math.asin(-centerMm / TIP_RADIUS_MM)
  const onTheTip = (angle: number) => [
    MIDDLE_MM + TIP_RADIUS_MM * Math.cos(angle),
    centerMm + TIP_RADIUS_MM * Math.sin(angle),
  ]
  const section = hasSides
    ? [MIDDLE_MM - TIP_RADIUS_MM, 0, MIDDLE_MM + TIP_RADIUS_MM, 0]
    : onTheTip(Math.PI - fromAngle)
  const lines = 2 * TIP_LINES
  for (let line = 0; line < lines; line += 1) {
    section.push(...onTheTip(fromAngle + (line / lines) * (Math.PI - 2 * fromAngle)))
  }
  if (hasSides) section.push(...onTheTip(Math.PI))
  return section
}

/** Lists every vertex of `solid`, as `[x, y, z]`. */
function listVertices(solid: BufferGeometry): [number, number, number][] {
  const position = solid.getAttribute('position')
  const vertices: [number, number, number][] = []
  for (let vertex = 0; vertex < position.count; vertex += 1) {
    vertices.push([position.getX(vertex), position.getY(vertex), position.getZ(vertex)])
  }
  return vertices
}

// x is rounded once to single precision near 227.5 mm, within 8e-6 mm: over
// the 2.5 mm half-axis, and doubled by the square, about 6e-6; y near 24 mm
// adds about an eighth of that.
const ON_ITS_SURFACE = 1e-5
// A vertex on the axis the section turns about, the middle of its foot or of
// its top: its distance from the axis, over the half-axes and squared.
const ON_THE_AXIS = 1e-6

/** How far `[x, y, z]` stands from the tip's sphere squeezed across, its
 * radius and its center's height taken as one. */
function measureOffTheTip([x, y, z]: [number, number, number], centerMm: number): number {
  const along = (x - MIDDLE_MM) / TIP_RADIUS_MM
  const up = (y - centerMm) / TIP_RADIUS_MM
  const across = z / (SHORT_AXIS_MM / 2)
  return Math.abs(along * along + up * up + across * across - 1)
}

describe('an oval part', () => {
  it('fills the box its section bounds along the key and up, and its width across the keyboard', () => {
    const solid = turnOval(traceThePin(HEAD_MM), SHORT_AXIS_MM, FACES_ROUND)
    solid.computeBoundingBox()
    const { min, max } = solid.boundingBox as Box3
    const expected = [
      MIDDLE_MM - LONG_AXIS_MM / 2,
      0,
      -SHORT_AXIS_MM / 2,
      MIDDLE_MM + LONG_AXIS_MM / 2,
      HEAD_MM,
      SHORT_AXIS_MM / 2,
    ]
    // Millimeters under three hundred in three's single precision.
    const isNear = (found: number, wanted: number) => Math.abs(found - wanted) <= 1e-4
    expect(
      [min.x, min.y, min.z, max.x, max.y, max.z].every((found, index) =>
        isNear(found, expected[index] as number),
      ),
    ).toBe(true)
  })

  it('stands on the ellipse its axes make, drawn through as many points as it has faces', () => {
    const plan = new Set<string>()
    let farthest = 0
    for (const [x, y, z] of listVertices(
      turnOval(traceThePin(HEAD_MM), SHORT_AXIS_MM, FACES_ROUND),
    )) {
      const along = (x - MIDDLE_MM) / (LONG_AXIS_MM / 2)
      const across = z / (SHORT_AXIS_MM / 2)
      const isOnTheAxis = along * along + across * across < ON_THE_AXIS
      if (y !== 0 || isOnTheAxis) continue
      plan.add(`${x.toFixed(4)} ${z.toFixed(4)}`)
      farthest = Math.max(farthest, Math.abs(along * along + across * across - 1))
    }
    expect(farthest).toBeLessThanOrEqual(ON_ITS_SURFACE)
    expect(plan.size).toBe(FACES_ROUND)
  })

  it('turns the rounded tip of its section into that sphere squeezed across to its width', () => {
    const centerMm = HEAD_MM - TIP_RADIUS_MM
    let farthest = 0
    let tipVertices = 0
    for (const vertex of listVertices(turnOval(traceThePin(HEAD_MM), SHORT_AXIS_MM, FACES_ROUND))) {
      if (vertex[1] <= centerMm) continue
      tipVertices += 1
      farthest = Math.max(farthest, measureOffTheTip(vertex, centerMm))
    }
    expect(tipVertices).toBeGreaterThan(FACES_ROUND)
    expect(farthest).toBeLessThanOrEqual(ON_ITS_SURFACE)
  })

  it('squeezes a pin that is all tip as its oval is squeezed, from the width of its foot', () => {
    const centerMm = CAP_MM - TIP_RADIUS_MM
    const solid = turnOval(traceThePin(CAP_MM), CAP_WIDTH_MM, FACES_ROUND)
    let farthest = 0
    for (const vertex of listVertices(solid)) {
      const isOnTheAxis = Math.abs(vertex[0] - MIDDLE_MM) + Math.abs(vertex[2]) < ON_THE_AXIS
      if (isOnTheAxis) continue
      farthest = Math.max(farthest, measureOffTheTip(vertex, centerMm))
    }
    expect(farthest).toBeLessThanOrEqual(ON_ITS_SURFACE)
  })

  it('draws nothing of a section with no length', () => {
    const point = [MIDDLE_MM, HEAD_MM, MIDDLE_MM, HEAD_MM, MIDDLE_MM, HEAD_MM]
    expect(turnOval(point, SHORT_AXIS_MM, FACES_ROUND).getAttribute('position')).toBeUndefined()
  })
})
