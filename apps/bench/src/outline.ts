/**
 * Turning the engine's flat coordinate buffers into drawable geometry.
 *
 * Separated from the view so it can be tested without a canvas, and so that the
 * one place that interprets the wasm boundary's layout is one place.
 */
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
