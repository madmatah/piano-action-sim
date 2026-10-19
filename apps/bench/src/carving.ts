/**
 * Where the solid is carved: the voids the engine cuts through a body's
 * parts, each swept across its width, the wood each runs through, and the
 * shader that leaves that wood unpainted inside it.
 *
 * The section draws a cut as the void it is, through the stencil; the solid
 * carves it out of the parts it runs through, by not painting them inside
 * it, and draws its walls from inside. Every cut the engine sends with a
 * width is a rectangle in the section, so the box its outline bounds, swept
 * across that width, holds the cut: a side hole fills it, and a milled cut,
 * a mortise or the balance hole, has its two ends rounded to its cutter's
 * radius, half the box's width across the keyboard, a hole as long as its
 * cutter is wide being round. A cut with no width is not carved, and
 * neither is one whose outline is no rectangle, which a box would misdraw.
 */

import type { Outline } from '@pas/wasm/types'
import * as THREE from 'three'
import { type Box, measureBounds, pairUp } from './bounds'

/** A cut as the solid carves it: the box it stands in, in a body's frame,
 * and whether its ends are rounded, milled by a cutter as wide as the box
 * across the keyboard, moved along the key: the box less its four corners,
 * each beyond a quarter circle of the cutter's radius. */
export type Hollow = {
  readonly box: Box
  readonly hasRoundEnds: boolean
}

/** The walls of one cut: the cut, its hollow, the parts of the body it runs
 * through, and the other cuts it opens into. */
export type Walls = {
  readonly cut: Outline
  readonly hollow: Hollow
  readonly through: readonly Outline[]
  readonly into: readonly Hollow[]
}

/** What the solid carves in one body: the cuts each part is carved by, and
 * each cut's walls. */
export type Carving = {
  readonly cutsThrough: ReadonlyMap<Outline, readonly Hollow[]>
  readonly walls: readonly Walls[]
}

/** How many cuts a part is carved by, or a cut's walls open into, at most:
 * the key button is carved by the balance mortise and the two side holes,
 * which the slot's walls open into with the balance hole, and the keystick
 * by the balance mortise, its hole, the front mortise and the groove it is
 * milled through, either side of it; one more would still be carved. */
export const MAX_CUTS = 6

/** How far past a cut's faces a face lying on them goes with it,
 * millimeters: the key button's top over the slot, its sides over the side
 * holes, the underside over the hole and the groove, and the key's sides
 * over the groove lie on the cuts' own faces. */
const SLACK_MM = 0.001

/** A coordinate past every part, millimeters, for a material kept whole. */
const FAR_MM = 1e6

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

/** Whether two boxes meet: they overlap or touch along every axis, within
 * a hair, as the slot and the balance hole do at the slot's floor and the
 * side holes at the slot's walls. The engine sends millimeters in single
 * precision, so a face it builds on another arrives a few of their ulps
 * either side of it. */
export function meets(first: Box, second: Box): boolean {
  return [0, 1, 2].every(
    (axis) =>
      (first.minMm[axis] as number) <= (second.maxMm[axis] as number) + SLACK_MM &&
      (second.minMm[axis] as number) <= (first.maxMm[axis] as number) + SLACK_MM,
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
 * Plans what the solid carves among one body's `outlines`. Each cut that is
 * a rectangle in the section, extruded or milled with a width, carves the
 * body's solid parts its box shares a volume with, never a lining, which
 * stands in the void; its walls are drawn in those parts, and open into the
 * other cuts its box meets.
 */
export function planCarving(outlines: readonly Outline[]): Carving {
  const cuts = outlines
    .filter(
      (outline) =>
        outline.role === 'cut' &&
        (outline.sweep === 'extruded' || outline.sweep === 'milled') &&
        outline.width_mm > 0 &&
        isRectangle(outline),
    )
    .map((cut) => ({
      cut,
      hollow: { box: measureBounds(cut), hasRoundEnds: cut.sweep === 'milled' },
    }))
  const solids = outlines.filter(
    (outline) => outline.role === 'solid' && outline.sweep === 'extruded',
  )
  const solidBoxes = solids.map((part) => ({ part, box: measureBounds(part) }))
  const cutsThrough = new Map<Outline, readonly Hollow[]>()
  for (const { part, box } of solidBoxes) {
    const carvedBy = cuts.filter((cut) => overlaps(cut.hollow.box, box)).map((cut) => cut.hollow)
    if (carvedBy.length > 0) cutsThrough.set(part, carvedBy)
  }
  const walls = cuts.map(({ cut, hollow }) => ({
    cut,
    hollow,
    through: solidBoxes.filter(({ box }) => overlaps(hollow.box, box)).map(({ part }) => part),
    into: cuts
      .filter((other) => other.cut !== cut && meets(other.hollow.box, hollow.box))
      .map((other) => other.hollow),
  }))
  return { cutsThrough, walls }
}

/**
 * Patches a three.js shader's source to leave unpainted what stands inside
 * any of the cuts its `cutMin` and `cutMax` uniforms hold, `cutCount` of
 * them, each its ends rounded where `cutHasRoundEnds` says so, and
 * what stands outside the box `keepMin` and `keepMax` hold, or on one of its
 * faces, where a cut's walls open through the part's outline, as the hole's
 * through the underside and the side holes' through the key button's sides:
 * positions read in the body's frame, as the part's geometry holds them.
 * Inside means within a hair of the cut, so that a face lying on one of its
 * faces over the void goes with it: the key button's top over the slot,
 * its sides over the side holes, the underside over the hole.
 * The source is the same for every set of cuts, so every carved part
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
uniform bool cutHasRoundEnds[MAX_CUTS];
uniform int cutCount;
uniform vec3 keepMin;
uniform vec3 keepMax;
float measureBeyondTheCutter(vec3 position, vec3 low, vec3 high) {
	float radius = 0.5 * (high.z - low.z);
	vec2 axis = vec2(clamp(position.x, low.x + radius, high.x - radius), 0.5 * (low.z + high.z));
	return distance(position.xz, axis) - radius;
}`,
      )
      .replace(
        '#include <clipping_planes_fragment>',
        `#include <clipping_planes_fragment>
\tif (any(lessThan(vCarve, keepMin - ${slack})) || any(greaterThan(vCarve, keepMax + ${slack}))) discard;
\tif (any(lessThan(abs(vCarve - keepMin), vec3(${slack}))) || any(lessThan(abs(vCarve - keepMax), vec3(${slack})))) discard;
\tfor (int cutIndex = 0; cutIndex < MAX_CUTS; cutIndex++) {
\t\tif (cutIndex >= cutCount) break;
\t\tvec3 low = cutMin[cutIndex];
\t\tvec3 high = cutMax[cutIndex];
\t\tbool isInBox = all(greaterThan(vCarve, low - ${slack})) && all(lessThan(vCarve, high + ${slack}));
\t\tif (isInBox && (!cutHasRoundEnds[cutIndex] || measureBeyondTheCutter(vCarve, low, high) < ${slack})) discard;
\t}`,
      ),
  }
}

/**
 * A milled cut's walls, to be drawn from inside: in plan, its box with its
 * two ends rounded to the cutter's radius, half its width across the
 * keyboard, swept up its height, `segments` faces round a whole circle.
 */
export function millWalls({ minMm, maxMm }: Box, segments: number): THREE.BufferGeometry {
  const [backMm, bottomMm, trebleMm] = minMm
  const [frontMm, topMm, bassMm] = maxMm
  const radiusMm = (bassMm - trebleMm) / 2
  const middleMm = (trebleMm + bassMm) / 2
  const plan = new THREE.Shape()
  plan.moveTo(backMm + radiusMm, trebleMm)
  plan.lineTo(frontMm - radiusMm, trebleMm)
  plan.absarc(frontMm - radiusMm, middleMm, radiusMm, -Math.PI / 2, Math.PI / 2, false)
  plan.lineTo(backMm + radiusMm, bassMm)
  plan.absarc(backMm + radiusMm, middleMm, radiusMm, Math.PI / 2, (3 * Math.PI) / 2, false)
  const walls = new THREE.ExtrudeGeometry(plan, {
    depth: topMm - bottomMm,
    bevelEnabled: false,
    // three draws an arc through twice as many points as it is given
    // segments: a quarter of a circle's faces for each half circle.
    curveSegments: segments / 4,
  })
  // The plan is drawn in x and z and swept along its own third axis: a
  // quarter turn about x sweeps it down from the top.
  walls.rotateX(Math.PI / 2)
  walls.translate(0, topMm, 0)
  return walls
}

/** The cuts a patched shader reads, padded to [`MAX_CUTS`]: their boxes as
 * flat triples, whether each has its ends rounded, how many, and the box to keep
 * within, every part kept where none is given. Refuses more cuts than the
 * shader holds, as a drawing that would paint wood over a void. */
export function packCuts(
  cuts: readonly Hollow[],
  keep: Box | null,
): {
  cutMin: Array<[number, number, number]>
  cutMax: Array<[number, number, number]>
  cutHasRoundEnds: boolean[]
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
      const hollow = cuts[cutIndex]
      return hollow ? [...hollow.box[end]] : [0, 0, 0]
    })
  return {
    cutMin: padded('minMm'),
    cutMax: padded('maxMm'),
    cutHasRoundEnds: Array.from(
      { length: MAX_CUTS },
      (_, cutIndex) => cuts[cutIndex]?.hasRoundEnds ?? false,
    ),
    cutCount: cuts.length,
    keepMin: keep ? [...keep.minMm] : [-FAR_MM, -FAR_MM, -FAR_MM],
    keepMax: keep ? [...keep.maxMm] : [FAR_MM, FAR_MM, FAR_MM],
  }
}
