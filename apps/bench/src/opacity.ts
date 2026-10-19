/**
 * How opaque each body of the view is drawn: the frame always wholly, the
 * key as the reader chose, and fainter still where no equilibrium places
 * it, so that a key with no place on its frame never reads as standing
 * there.
 */

/** How much fainter than chosen a key no equilibrium places is drawn:
 * enough that it reads as no place of the key's, and that the frame's parts
 * it is drawn through show. */
export const UNPLACED = 0.3

/** How opaque a body's parts are drawn. */
export function bodyOpacity({
  grounded,
  chosen,
  isPlaced,
}: {
  /** Whether the body is the ground, which carries the frame. */
  grounded: boolean
  /** How opaque the reader chose the key to be. */
  chosen: number
  /** Whether an equilibrium placed the key where it is drawn. */
  isPlaced: boolean
}): number {
  if (grounded) return 1
  return isPlaced ? chosen : chosen * UNPLACED
}
