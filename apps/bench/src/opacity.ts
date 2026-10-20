/**
 * How opaque each body of the view is drawn: the frame always wholly, the
 * key as the reader chose, its two mortises' bushings apart from the rest
 * of it, and fainter still where no equilibrium places it, so that a key
 * with no place on its frame never reads as standing there.
 */

import type { Outline, Part } from '@pas/wasm/types'

/** How much fainter than chosen a key no equilibrium places is drawn:
 * enough that it reads as no place of the key's, and that the frame's parts
 * it is drawn through show. */
export const UNPLACED = 0.3

/** How opaque a body's parts are drawn: the two mortises' bushings apart
 * from the rest. */
export type BodyOpacity = {
  /** Every part but the two mortises' bushings: on the key, its wood, what
   * is glued or screwed into it and its other linings, such as its key tail
   * felt. */
  readonly parts: number
  /** The balance mortise's bushing and the front mortise's, which the
   * reader sees inside the key's wood however faint the wood is drawn. */
  readonly bushings: number
}

/** A body drawn wholly. */
const WHOLE: BodyOpacity = { parts: 1, bushings: 1 }

/** The parts that follow the bushings' opacity: the two mortises'
 * bushings. */
const BUSHINGS: ReadonlySet<Part> = new Set<Part>([
  'balance_mortise_bushing',
  'front_mortise_bushing',
])

/** How opaque a body's parts are drawn. */
export function bodyOpacity({
  grounded,
  chosen,
  chosenForBushings,
  isPlaced,
}: {
  /** Whether the body is the ground, which carries the frame. */
  grounded: boolean
  /** How opaque the reader chose the key to be. */
  chosen: number
  /** How opaque the reader chose the key's two mortises' bushings to be. */
  chosenForBushings: number
  /** Whether an equilibrium placed the key where it is drawn. */
  isPlaced: boolean
}): BodyOpacity {
  if (grounded) return WHOLE
  return isPlaced
    ? { parts: chosen, bushings: chosenForBushings }
    : { parts: chosen * UNPLACED, bushings: chosenForBushings * UNPLACED }
}

/** How opaque one of a body's parts is drawn: a mortise's bushing as the
 * body's bushings, any other part as its parts. */
export function partOpacity(part: Pick<Outline, 'code'>, opacity: BodyOpacity): number {
  return BUSHINGS.has(part.code) ? opacity.bushings : opacity.parts
}
