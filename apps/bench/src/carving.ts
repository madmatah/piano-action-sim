/**
 * Where the solid is carved: the voids the engine cuts through a body's
 * parts, each swept across its width, the wood each runs through, and the
 * shader that leaves that wood unpainted inside it.
 *
 * The section draws a cut as the void it is, through the stencil; the solid
 * carves it out of the parts it runs through, by not painting them inside
 * it, and draws its walls from inside. Every cut the engine sends with a
 * width is a rectangle in the section, so the box its outline bounds, swept
 * across that width, is the cut itself. A cut with no width, the front
 * mortise, whose width the model does not know, is not carved, and neither
 * is one whose outline is no rectangle, which a box would misdraw.
 */

import type { Outline } from '@pas/wasm/types'

/** A box in a body's frame: `x` along the key, `y` up, `z` across the
 * keyboard toward the bass. */
export type Box = {
  readonly minMm: readonly [number, number, number]
  readonly maxMm: readonly [number, number, number]
}

/** The walls of one cut: the cut, its box, the parts of the body it runs
 * through, and the other cuts it opens into. */
export type Walls = {
  readonly cut: Outline
  readonly box: Box
  readonly through: readonly Outline[]
  readonly into: readonly Box[]
}

/** What the solid carves in one body: the cuts each part is carved by, and
 * each cut's walls. */
export type Carving = {
  readonly cutsThrough: ReadonlyMap<Outline, readonly Box[]>
  readonly walls: readonly Walls[]
}

/** How many cuts a part is carved by, or a cut's walls open into, at most:
 * a key's balance mortise and its hole, and room for two more. */
export const MAX_CUTS = 4

/** How far past a cut's top and bottom a face lying on them goes with it,
 * millimeters: the key button's top over the slot and the underside over
 * the hole lie on the cuts' own faces. */
const SLACK_MM = 0.001

/** A coordinate past every part, millimeters, for a material kept whole. */
const FAR_MM = 1e6

/** The points of an outline as the engine sends them, flat `[x0, y0, x1,
 * y1, …]`, as pairs. */
function pairUp(points: readonly number[]): Array<[number, number]> {
  const pairs: Array<[number, number]> = []
  for (let pointIndex = 0; pointIndex < points.length; pointIndex += 2) {
    pairs.push([points[pointIndex] as number, points[pointIndex + 1] as number])
  }
  return pairs
}

/** Bounds an extruded outline: the box around its points in the section,
 * across its width. */
export function measureBounds(outline: Outline): Box {
  const points = pairUp(outline.points_mm)
  const xs = points.map(([x]) => x)
  const ys = points.map(([, y]) => y)
  const half = outline.width_mm / 2
  return {
    minMm: [Math.min(...xs), Math.min(...ys), outline.across_mm - half],
    maxMm: [Math.max(...xs), Math.max(...ys), outline.across_mm + half],
  }
}

/** Whether every segment of `outline`, its closing one included, runs along
 * a side of the box its points bound: a rectangle, its sides perhaps split
 * into collinear segments. */
export function isRectangle(outline: Outline): boolean {
  const { minMm, maxMm } = measureBounds(outline)
  const points = pairUp(outline.points_mm)
  return points.every(([x0, y0], pointIndex) => {
    const [x1, y1] = points[(pointIndex + 1) % points.length] as [number, number]
    const alongAnEnd = x0 === x1 && (x0 === minMm[0] || x0 === maxMm[0])
    const alongASide = y0 === y1 && (y0 === minMm[1] || y0 === maxMm[1])
    return alongAnEnd || alongASide
  })
}

/** Whether two boxes meet: they overlap or touch along every axis, as the
 * slot and the balance hole do at the slot's floor. */
export function meets(first: Box, second: Box): boolean {
  return [0, 1, 2].every(
    (axis) =>
      (first.minMm[axis] as number) <= (second.maxMm[axis] as number) &&
      (second.minMm[axis] as number) <= (first.maxMm[axis] as number),
  )
}

/** Whether two boxes share a volume: they overlap along every axis, and do
 * not merely touch. */
export function overlaps(first: Box, second: Box): boolean {
  return [0, 1, 2].every(
    (axis) =>
      (first.minMm[axis] as number) < (second.maxMm[axis] as number) &&
      (second.minMm[axis] as number) < (first.maxMm[axis] as number),
  )
}

/**
 * Plans what the solid carves among one body's `outlines`. Each extruded cut
 * with a width, a rectangle in the section, carves the body's solid parts
 * its box shares a volume with, never a lining, which stands in the void;
 * its walls are drawn in those parts, and open into the other cuts its box
 * meets.
 */
export function planCarving(outlines: readonly Outline[]): Carving {
  const cuts = outlines
    .filter(
      (outline) =>
        outline.role === 'cut' &&
        outline.sweep === 'extruded' &&
        outline.width_mm > 0 &&
        isRectangle(outline),
    )
    .map((cut) => ({ cut, box: measureBounds(cut) }))
  const solids = outlines.filter(
    (outline) => outline.role === 'solid' && outline.sweep === 'extruded',
  )
  const cutsThrough = new Map<Outline, readonly Box[]>()
  for (const part of solids) {
    const box = measureBounds(part)
    const carvedBy = cuts.filter((cut) => overlaps(cut.box, box)).map((cut) => cut.box)
    if (carvedBy.length > 0) cutsThrough.set(part, carvedBy)
  }
  const walls = cuts.map(({ cut, box }) => ({
    cut,
    box,
    through: solids.filter((part) => overlaps(box, measureBounds(part))),
    into: cuts
      .filter((other) => other.cut !== cut && meets(other.box, box))
      .map((other) => other.box),
  }))
  return { cutsThrough, walls }
}

/**
 * Patches a three.js shader's source to leave unpainted what stands inside
 * any of the cuts its `cutMin` and `cutMax` uniforms hold, `cutCount` of
 * them, and what stands outside the box `keepMin` and `keepMax` hold, or on
 * its top or bottom, where a cut's walls open through the part's outline,
 * as the hole's through the underside: positions read in the body's frame,
 * as the part's geometry holds them.
 * Inside means strictly between a cut's sides and ends, and within a hair
 * of its top and bottom, so that a face lying on a cut's top or bottom over
 * the void goes with it, while the edges a cut's walls meet the wood along
 * stay. The source is the same for every set of boxes, so every carved part
 * shares one program.
 */
export function patchShader(
  vertex: string,
  fragment: string,
): { vertex: string; fragment: string } {
  const slack = SLACK_MM.toFixed(4)
  return {
    vertex: vertex
      .replace('#include <common>', '#include <common>\nvarying vec3 vCarve;')
      .replace('#include <begin_vertex>', '#include <begin_vertex>\n\tvCarve = transformed;'),
    fragment: fragment
      .replace(
        '#include <common>',
        `#include <common>
#define MAX_CUTS ${MAX_CUTS}
varying vec3 vCarve;
uniform vec3 cutMin[MAX_CUTS];
uniform vec3 cutMax[MAX_CUTS];
uniform int cutCount;
uniform vec3 keepMin;
uniform vec3 keepMax;`,
      )
      .replace(
        '#include <clipping_planes_fragment>',
        `#include <clipping_planes_fragment>
\tif (any(lessThan(vCarve, keepMin - ${slack})) || any(greaterThan(vCarve, keepMax + ${slack}))) discard;
\tif (abs(vCarve.y - keepMin.y) < ${slack} || abs(vCarve.y - keepMax.y) < ${slack}) discard;
\tfor (int cutIndex = 0; cutIndex < MAX_CUTS; cutIndex++) {
\t\tif (cutIndex >= cutCount) break;
\t\tvec3 low = cutMin[cutIndex];
\t\tvec3 high = cutMax[cutIndex];
\t\tif (vCarve.x > low.x && vCarve.x < high.x && vCarve.z > low.z && vCarve.z < high.z
\t\t\t&& vCarve.y > low.y - ${slack} && vCarve.y < high.y + ${slack}) discard;
\t}`,
      ),
  }
}

/** The boxes a patched shader reads, as flat triples padded to
 * [`MAX_CUTS`]: the cuts, how many, and the box to keep within, every part
 * kept where none is given. Refuses more cuts than the shader holds, as a
 * drawing that would paint wood over a void. */
export function packCuts(
  cuts: readonly Box[],
  keep: Box | null,
): {
  cutMin: Array<[number, number, number]>
  cutMax: Array<[number, number, number]>
  cutCount: number
  keepMin: [number, number, number]
  keepMax: [number, number, number]
} {
  if (cuts.length > MAX_CUTS) {
    throw new Error(
      `${cuts.length} cuts through one part, more than the ${MAX_CUTS} the solid carves`,
    )
  }
  const padded = (end: 'minMm' | 'maxMm') =>
    Array.from({ length: MAX_CUTS }, (_, cutIndex): [number, number, number] => {
      const box = cuts[cutIndex]
      return box ? [...box[end]] : [0, 0, 0]
    })
  return {
    cutMin: padded('minMm'),
    cutMax: padded('maxMm'),
    cutCount: cuts.length,
    keepMin: keep ? [...keep.minMm] : [-FAR_MM, -FAR_MM, -FAR_MM],
    keepMax: keep ? [...keep.maxMm] : [FAR_MM, FAR_MM, FAR_MM],
  }
}
