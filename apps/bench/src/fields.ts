/**
 * What a typed field holds, as a number, or `null` when it holds none.
 *
 * `Number('')` is 0, so a field emptied and read with `Number` would send a
 * target of nothing at all, and the engine would refuse a regulation nobody
 * asked for, a key height of 0.00 mm out of reach by 63 mm. A number input
 * that holds no valid number reports its value as the empty string, so that
 * reads as no number too; so does anything that is not a finite number.
 */
export function typedNumber(text: string): number | null {
  if (text.trim() === '') return null
  const n = Number(text)
  return Number.isFinite(n) ? n : null
}
