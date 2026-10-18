/**
 * What the band's view writes beside its curves: the force at a step, and
 * the extreme of each curve where it falls, every one in full to the
 * hundredth of a gram.
 *
 * The band is a curve, read by no weight. Its extremes are not the sheet's
 * down and up weights, which the breakaway walk reads with gram weights from
 * the seats, the trade's protocol, and rounds as the weights give them: the
 * two stand within a few thousandths of a gram of each other in full, and so
 * may fall either side of a whole gram. The owner's section weighed at 130 g
 * with a front weight of 40 g reads its up weight -47 g on the sheet, from
 * -46.00019 g, where the up curve's least force is -45.99997 g. So the band
 * labels its extremes in full and never by the gram.
 */
import type { Reading } from '@pas/wasm/types'
import type { Catalog } from './i18n'

/** A reading read, as the band carries it. */
export type Read = Extract<Reading, { outcome: 'read' }>

/** The decimals a force on the band is shown to: the hundredth of a gram. */
export const FORCE_DECIMALS = 2
/** The decimals a travel is shown to: the hundredth of a millimeter. */
export const TRAVEL_DECIMALS = 2

/**
 * The label of a curve's extreme, marked where it falls, as `t` words it: its
 * force in full, to the hundredth of a gram, and never the gram a protocol
 * reads.
 */
export function extremeLabel(t: Catalog, m: Read): string {
  const grams = t.number(m.value, FORCE_DECIMALS)
  return m.procedure === 'down_weight_on_the_band'
    ? t.band.downMarker(grams)
    : t.band.upMarker(grams)
}

/** The same, with where it falls, for a reader without the chart. */
export function extremeAt(t: Catalog, m: Read): string {
  const atMm = m.conditions.sweep?.at_mm ?? 0
  return t.band.extremeAt(extremeLabel(t, m), t.number(atMm, TRAVEL_DECIMALS))
}
