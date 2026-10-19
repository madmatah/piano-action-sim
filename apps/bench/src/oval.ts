/**
 * An oval part as the solid draws it: the front rail pin's head, standing
 * upright, its plan an ellipse whose axes are the box its outline bounds,
 * along the key and across the keyboard.
 */

import * as THREE from 'three'
import type { Box } from './bounds'
import { standUp } from './upright'

/** An oval part's solid: in plan, the ellipse its box holds, swept up its
 * height, `segments` faces round it. */
export function sweepOval({ minMm, maxMm }: Box, segments: number): THREE.BufferGeometry {
  const [backMm, bottomMm, trebleMm] = minMm
  const [frontMm, topMm, bassMm] = maxMm
  const plan = new THREE.Shape()
  plan.absellipse(
    (backMm + frontMm) / 2,
    (trebleMm + bassMm) / 2,
    (frontMm - backMm) / 2,
    (bassMm - trebleMm) / 2,
    0,
    2 * Math.PI,
    false,
    0,
  )
  // three draws an ellipse through twice as many points as it is given
  // segments.
  return standUp(plan, bottomMm, topMm, segments / 2)
}
