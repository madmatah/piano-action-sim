/**
 * An oval part as the solid draws it: the front rail pin's head, standing
 * upright, its section along the key turned about its middle and squeezed
 * across the keyboard to its width, so that its plan is an ellipse and its
 * rounded tip a half sphere squeezed the same way.
 */

import * as THREE from 'three'

/** How near the middle a point of the section stands on the axis it turns
 * about, millimeters: well under the step of an arc cut into lines, and well
 * over a millimeter's single-precision rounding near 300 mm. */
const ON_THE_AXIS_MM = 1e-3

/** An oval part's solid: the half of its section `pointsMm`, flat
 * `[x0, y0, x1, y1, …]` and closed, in front of its middle, turned about
 * the upright through the middle, `segments` faces round it, then squeezed
 * across the keyboard so that its foot, as long as the section, is
 * `widthMm` wide; nothing where the section has no length. */
export function turnOval(
  pointsMm: readonly number[],
  widthMm: number,
  segments: number,
): THREE.BufferGeometry {
  const section: THREE.Vector2[] = []
  for (let index = 0; index + 1 < pointsMm.length; index += 2) {
    section.push(new THREE.Vector2(pointsMm[index] as number, pointsMm[index + 1] as number))
  }
  const alongMm = section.map((point) => point.x)
  const upMm = section.map((point) => point.y)
  const backMm = Math.min(...alongMm)
  const frontMm = Math.max(...alongMm)
  const lengthMm = frontMm - backMm
  if (!(lengthMm > 0)) return new THREE.BufferGeometry()
  const middleMm = (backMm + frontMm) / 2
  const isInFront = (point: THREE.Vector2) => point.x - middleMm > ON_THE_AXIS_MM
  // The section runs counter-clockwise, so the points in front of the middle
  // are one stretch of it, from the foot up: from the first that follows one
  // behind or on the axis.
  const first = section.findIndex(
    (point, index) => isInFront(point) && !isInFront(section.at(index - 1) ?? point),
  )
  const front: THREE.Vector2[] = []
  for (const point of [...section.slice(first), ...section.slice(0, first)]) {
    if (!isInFront(point)) break
    front.push(new THREE.Vector2(point.x - middleMm, point.y))
  }
  const profile = [
    new THREE.Vector2(0, Math.min(...upMm)),
    ...front,
    new THREE.Vector2(0, Math.max(...upMm)),
  ]
  const solid = new THREE.LatheGeometry(profile, segments)
  solid.scale(1, 1, widthMm / lengthMm)
  solid.translate(middleMm, 0, 0)
  return solid
}
