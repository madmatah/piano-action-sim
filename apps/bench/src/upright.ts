/**
 * A plan stood upright: a shape drawn in plan, in a body's `x` and `z`,
 * swept up between two heights, as the solid draws a milled cut's walls and
 * an oval part.
 */

import * as THREE from 'three'

/** `plan`, drawn in `x` and `z`, swept up from `bottomMm` to `topMm`, each
 * curve of it drawn with `curveSegments`. */
export function standUp(
  plan: THREE.Shape,
  bottomMm: number,
  topMm: number,
  curveSegments: number,
): THREE.BufferGeometry {
  const solid = new THREE.ExtrudeGeometry(plan, {
    depth: topMm - bottomMm,
    bevelEnabled: false,
    curveSegments,
  })
  // three sweeps a shape along its own third axis: a quarter turn about x
  // sweeps it down from the top.
  solid.rotateX(Math.PI / 2)
  solid.translate(0, topMm, 0)
  return solid
}
