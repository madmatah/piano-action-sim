/**
 * Turning the engine's flat coordinate buffers into drawable geometry.
 *
 * Separated from the view so it can be tested without a canvas, and so that the
 * one place that interprets the wasm boundary's layout is one place. Nothing
 * here decides where anything is: a pose is applied as the engine gives it,
 * and an outline is drawn through the points it arrives with.
 */

import type { ContactState, MarkedContact, Pose } from '@pas/wasm/types'
import * as THREE from 'three'

/**
 * The engine's flat `[x0, y0, x1, y1, …]` as a closed, filled shape.
 *
 * The length check is not defensive clutter: this buffer crosses the wasm
 * boundary, and an odd length would mean the boundary itself is wrong. Better
 * to stop than to draw half a part and let someone measure it.
 */
export function toShape(points: readonly number[]): THREE.Shape {
  if (points.length % 2 !== 0 || points.length < 6) {
    throw new Error(`outline has ${points.length} coordinates, which is not a polygon`)
  }
  const shape = new THREE.Shape()
  shape.moveTo(points[0] as number, points[1] as number)
  for (let i = 2; i < points.length; i += 2) {
    shape.lineTo(points[i] as number, points[i + 1] as number)
  }
  shape.closePath()
  return shape
}

/**
 * A body's pose as the matrix three.js places its group with: the turn and
 * the shift the engine computed, entered as they are. No angle is taken back
 * out of them, so the view adds no trigonometry of its own.
 */
export function poseMatrix(pose: Pose): THREE.Matrix4 {
  const [x, y] = pose.origin
  const { cos, sin } = pose
  return new THREE.Matrix4().set(cos, -sin, 0, x, sin, cos, 0, y, 0, 0, 1, 0, 0, 0, 0, 1)
}

/**
 * A turned part's profile mirrored about its axis, `(r, y)` to `(−r, y)`, and
 * run backwards so that it turns counter-clockwise again. A section through
 * the axis shows the profile and this; a washer's hole stays open between
 * them, with its pin in it.
 */
export function mirrored(points: readonly number[]): number[] {
  const out: number[] = []
  for (let i = points.length - 2; i >= 0; i -= 2) {
    out.push(-(points[i] as number), points[i + 1] as number)
  }
  return out
}

/** Where each segment of an outline starts and ends: segment `k` runs from
 * its first point to the next segment's first, the last back to the start. */
export function segmentEnds(
  points: readonly number[],
  segs: readonly number[],
): Map<number, [number, number, number, number]> {
  const firsts: Array<[number, number]> = []
  for (let i = 0; i < segs.length; i += 1) {
    const seg = segs[i] as number
    if (i === 0 || seg !== segs[i - 1]) firsts.push([seg, i])
  }
  const ends = new Map<number, [number, number, number, number]>()
  firsts.forEach(([seg, i], k) => {
    const next = (firsts[(k + 1) % firsts.length] as [number, number])[1]
    ends.set(seg, [
      points[2 * i] as number,
      points[2 * i + 1] as number,
      points[2 * next] as number,
      points[2 * next + 1] as number,
    ])
  })
  return ends
}

/**
 * The segments of `a` that run exactly along a segment of `b`, either way.
 *
 * Where a cut opens through its body's outline, the cut and the solid share
 * a segment: the mouth of a mortise is both the slot's bottom and a stretch of
 * the keystick's underside. A section draws neither as an edge, because there
 * is no material on either side of it in the plane. The engine builds both
 * from the same numbers, so the comparison is exact.
 */
export function sharedSegments(
  a: { points: readonly number[]; segs: readonly number[] },
  b: { points: readonly number[]; segs: readonly number[] },
): Set<number> {
  const theirs = [...segmentEnds(b.points, b.segs).values()]
  const shared = new Set<number>()
  for (const [seg, [x0, y0, x1, y1]] of segmentEnds(a.points, a.segs)) {
    const along = theirs.some(
      ([u0, v0, u1, v1]) =>
        (u0 === x0 && v0 === y0 && u1 === x1 && v1 === y1) ||
        (u0 === x1 && v0 === y1 && u1 === x0 && v1 === y0),
    )
    if (along) shared.add(seg)
  }
  return shared
}

/** The segments of a turned part's profile that run along its axis, where the
 * profile and its mirror meet: no edge in a section. */
export function axisSegments(points: readonly number[], segs: readonly number[]): Set<number> {
  const on = new Set<number>()
  for (const [seg, [x0, , x1]] of segmentEnds(points, segs)) {
    if (x0 === 0 && x1 === 0) on.add(seg)
  }
  return on
}

/**
 * An outline's edges as line pieces, flat `[x0, y0, x1, y1, …]` pairs, the
 * segments in `skip` left out.
 */
export function strokes(
  points: readonly number[],
  segs: readonly number[],
  skip: ReadonlySet<number> = new Set(),
): number[] {
  const out: number[] = []
  const n = segs.length
  for (let i = 0; i < n; i += 1) {
    if (skip.has(segs[i] as number)) continue
    const j = (i + 1) % n
    out.push(
      points[2 * i] as number,
      points[2 * i + 1] as number,
      points[2 * j] as number,
      points[2 * j + 1] as number,
    )
  }
  return out
}

/**
 * The contacts in one state as line pieces, flat `[x0, y0, x1, y1, …]`, each
 * from the frame's surface point to the key's: the gap a reader sees across
 * an open contact, the depth across an overlapping one. A closed contact's
 * two points coincide, and it is marked by its point alone.
 */
export function contactStrokes(contacts: readonly MarkedContact[], state: ContactState): number[] {
  const out: number[] = []
  for (const c of contacts) {
    if (c.state !== state) continue
    out.push(c.on_frame[0], c.on_frame[1], c.on_key[0], c.on_key[1])
  }
  return out
}
