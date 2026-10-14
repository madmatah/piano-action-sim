/**
 * Where a chart's axis ticks stand: presentation only. The numbers a chart
 * plots are the engine's; these place the round values its grid is read by.
 */

/**
 * A round step that cuts `span` into about `count` pieces: one, two or five
 * times a power of ten, the least of them that gives no more than `count`.
 */
export function niceStep(span: number, count: number): number {
  if (!(span > 0) || !(count > 0)) return 1
  const rough = span / count
  const power = 10 ** Math.floor(Math.log10(rough))
  for (const m of [1, 2, 5, 10]) {
    if (m * power >= rough) return m * power
  }
  return 10 * power
}

/** The multiples of `step` from `lo` to `hi`, both ends included where they
 * fall on one, a value that rounds to nothing written as 0. */
export function ticks(lo: number, hi: number, step: number): number[] {
  const out: number[] = []
  if (!(step > 0) || !(hi >= lo)) return out
  const first = Math.ceil(lo / step - 1e-9)
  const last = Math.floor(hi / step + 1e-9)
  for (let k = first; k <= last; k += 1) out.push(k * step + 0)
  return out
}

/** The range a chart's values stand in, `pad` of its span added either
 * side, and one unit either side where they all stand at one value. */
export function padded(values: readonly number[], pad: number): [number, number] {
  const finite = values.filter((v) => Number.isFinite(v))
  if (finite.length === 0) return [-1, 1]
  const lo = Math.min(...finite)
  const hi = Math.max(...finite)
  const room = hi > lo ? pad * (hi - lo) : 1
  return [lo - room, hi + room]
}

/** The index of the value in `sorted`, ascending, nearest `x`; `-1` where
 * there is none. */
export function nearest(sorted: readonly number[], x: number): number {
  if (sorted.length === 0) return -1
  let lo = 0
  let hi = sorted.length - 1
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1
    if ((sorted[mid] ?? 0) < x) lo = mid
    else hi = mid
  }
  return Math.abs((sorted[hi] ?? 0) - x) < Math.abs((sorted[lo] ?? 0) - x) ? hi : lo
}
