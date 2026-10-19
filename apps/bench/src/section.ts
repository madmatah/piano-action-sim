/**
 * What the section draws of one body: the voids its plane cuts, the parts it
 * cuts, and the parts beside it seen beyond it through one of those voids.
 *
 * The section is the sagittal plane, seen from the bass side, `+z`: a part
 * standing beside the plane on the far side is beyond it, seen only where a
 * void of the plane opens onto it. Which parts stand where is the engine's,
 * as each outline's `across_mm` and `is_in_the_section` say; this only sorts
 * them for drawing.
 */

import type { Outline } from '@pas/wasm/types'
import { measureBounds } from './bounds'

/**
 * How far a void must reach past a part's near face to show it, millimeters.
 * The engine sends millimeters in single precision, so a face it builds on a
 * void's face, as the bushing's leg out through the key button's side hole
 * stands on the slot's wall, arrives a few of their ulps either side of it.
 */
const PAST_THE_FACE_MM = 0.001

/** One body's outlines as the section draws them. */
export type Section = {
  /** The voids the plane cuts: the key button's side holes, beside it, are
   * not among them. */
  readonly cuts: readonly Outline[]
  /** The parts the plane cuts. */
  readonly inPlane: readonly Outline[]
  /** The parts beside the plane, on its far side, seen through a void. */
  readonly beyond: readonly Outline[]
}

/**
 * Whether `part`, standing beside the sagittal plane on its far side, is
 * seen through one of `cuts`: the void reaches past the part's near face,
 * across the keyboard, where the part stands along the key and up. The
 * bushing's leg down the slot's far wall is; its leg out through the side
 * hole, inside the wall behind it, is not.
 */
export function isSeenBeyondThePlane(part: Outline, cuts: readonly Outline[]): boolean {
  if (part.is_in_the_section || part.across_mm >= 0) return false
  const nearFaceMm = part.across_mm + part.width_mm / 2
  const { minMm, maxMm } = measureBounds(part)
  return cuts.some((cut) => {
    const opening = measureBounds(cut)
    const isFacing = [0, 1].every(
      (axis) =>
        (minMm[axis] as number) < (opening.maxMm[axis] as number) &&
        (opening.minMm[axis] as number) < (maxMm[axis] as number),
    )
    return isFacing && opening.minMm[2] < nearFaceMm - PAST_THE_FACE_MM
  })
}

/** Sorts one body's `outlines` into what the section draws. */
export function divideTheSection(outlines: readonly Outline[]): Section {
  const cuts = outlines.filter((outline) => outline.role === 'cut' && outline.is_in_the_section)
  const parts = outlines.filter((outline) => outline.role !== 'cut')
  return {
    cuts,
    inPlane: parts.filter((part) => part.is_in_the_section),
    beyond: parts.filter((part) => isSeenBeyondThePlane(part, cuts)),
  }
}
