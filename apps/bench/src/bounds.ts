/**
 * Where an outline the engine sends stands in its body's frame: the box it
 * fills, swept across the keyboard as the engine sweeps it. The section
 * reads it to tell what a void opens onto, and the solid to carve.
 */

import type { Outline } from '@pas/wasm/types'

/** A box in a body's frame: `x` along the key, `y` up, `z` across the
 * keyboard toward the bass. */
export type Box = {
  readonly minMm: readonly [number, number, number]
  readonly maxMm: readonly [number, number, number]
}

/** The points of an outline as the engine sends them, flat `[x0, y0, x1,
 * y1, …]`, as pairs. */
export function pairUp(points: readonly number[]): Array<[number, number]> {
  const pairs: Array<[number, number]> = []
  for (let pointIndex = 0; pointIndex < points.length; pointIndex += 2) {
    pairs.push([points[pointIndex] as number, points[pointIndex + 1] as number])
  }
  return pairs
}

/** Bounds an extruded or a bored outline: the box around its points in the
 * section, across its width, or, bored round, about the axis the engine
 * sends, as far from it across the keyboard as along the key. */
export function measureBounds(outline: Outline): Box {
  const points = pairUp(outline.points_mm)
  const xs = points.map(([x]) => x)
  const ys = points.map(([, y]) => y)
  const [minY, maxY] = [Math.min(...ys), Math.max(...ys)]
  if (outline.sweep === 'bored') {
    const axis = outline.axis_x_mm
    const radiusMm = Math.max(...xs.map((x) => Math.abs(x - axis)))
    return {
      minMm: [axis - radiusMm, minY, -radiusMm],
      maxMm: [axis + radiusMm, maxY, radiusMm],
    }
  }
  const half = outline.width_mm / 2
  return {
    minMm: [Math.min(...xs), minY, outline.across_mm - half],
    maxMm: [Math.max(...xs), maxY, outline.across_mm + half],
  }
}
