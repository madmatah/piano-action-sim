/**
 * How many decimals a number is shown to, from the resolution it is read to.
 *
 * A reading carries its protocol's resolution and crosses already read by
 * the protocol's rounding (`shown`, beside its value in full), so the page
 * only writes it to that resolution: a down weight to the gram, a balance
 * weight to the half gram, a front weight to the tenth, a ratio to the
 * hundredth. Never `toFixed` on the value in full, which would round a down
 * weight of 46.3 g to 46 g, a weight that does not take the key down.
 *
 * What is not a reading, a setting or a quantity the bench reports, is shown
 * to the resolution its kind is read to, declared once below.
 */
import type { Quantity } from '@pas/wasm/types'

/** A length, millimeters: the hundredth the drawer's papers are told apart by. */
export const LENGTH_MM = 0.01
/** A gap between the key and a stop, millimeters: to the micron. */
export const GAP_MM = 0.001
/** An angle, degrees. */
export const ANGLE_DEG = 0.01
/** A mass or a weight typed or set, grams. */
export const MASS_G = 0.01
/** A temporary weight's mass, grams. */
export const TEMPORARY_WEIGHT_G = 0.1
/** Where a weight, or the key's center of mass, stands along the key,
 * millimeters. */
export const STATION_MM = 1
/** Where a front weight was read from the lip, millimeters. */
export const FROM_LIP_MM = 0.1
/** How far a part stands out of place, or a dimension of a cut, as a fault
 * or a refusal names it, millimeters: to the tenth. */
export const PART_MM = 0.1
/** A weight or a force a refusal names, grams: to the nearest whole gram. */
export const WEIGHT_G = 1
/** How much paper a regulation's search lays on a stack before it gives up,
 * millimeters. */
export const SEARCH_MM = 1
/** A value refused as out of its setting's range, in the unit its path
 * names: to the hundredth. */
export const OUT_OF_RANGE = 0.01

/**
 * The decimals a resolution is shown to: none for the gram, one for a tenth
 * or a half gram, two for a hundredth or a paper, three for a micron.
 */
export function decimals(resolution: number): number {
  let d = 0
  let r = resolution
  // A tenth is 0.1000…0055 in binary and a hundredth 0.0100…0002: the
  // tolerance keeps ten tenths from reading as just under one.
  while (r < 1 - 1e-9 && d < 12) {
    r *= 10
    d += 1
  }
  return d
}

/**
 * `x` to `digits` places, a number that rounds to nothing shown as 0, never
 * as -0: a key level to a few attometers at the bottom of its dip is level.
 * The decimal point is this function's; a catalog for another language
 * writes its own.
 */
export function fixed(x: number, digits: number): string {
  const shown = x.toFixed(digits)
  return Number(shown) === 0 ? (0).toFixed(digits) : shown
}

/**
 * Whether `distance`, more than nothing, rounds to nothing at `resolution`: a
 * distance the engine reads as more than nothing that the page would show as
 * 0, which a catalog words as less than its resolution instead. The value is
 * rounded as `fixed` shows it; no number is computed.
 */
export function isTooSmallToShow(distance: number, resolution: number): boolean {
  return distance > 0 && Number(fixed(distance, decimals(resolution))) === 0
}

/**
 * A number as a field holding it is seeded: to `resolution`, without the
 * zeros a typed number would not have, so 10.200000000000001 mm of dip is
 * typed as 10.2.
 */
export function typed(x: number, resolution: number): string {
  return String(Number(x.toFixed(decimals(resolution))))
}

/** A torque, gram-force millimeters. */
export const TORQUE_GMM = 0.1

/**
 * What a setting is shown to, by what it measures: a length to the hundredth
 * of a millimeter, a mass or a weight to the hundredth of a gram, a torque to
 * the tenth of a gram-millimeter.
 */
export const RESOLUTION: { readonly [Q in Quantity]: number } = {
  length: LENGTH_MM,
  mass: MASS_G,
  torque: TORQUE_GMM,
}
