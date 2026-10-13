import { describe, expect, it } from 'vitest'
import { typedNumber } from './fields'

describe('typedNumber', () => {
  it('reads what is typed as the number it is', () => {
    expect(typedNumber('65')).toBe(65)
    expect(typedNumber('10.2')).toBe(10.2)
    expect(typedNumber(' 10.2 ')).toBe(10.2)
    expect(typedNumber('-1.5')).toBe(-1.5)
    // A zero typed is a zero: only an empty field is no number.
    expect(typedNumber('0')).toBe(0)
  })

  it('reads an emptied field as no number, never as 0', () => {
    // `Number('')` and `Number(' ')` are both 0, which sent a key height of
    // nothing to the engine.
    expect(Number('')).toBe(0)
    expect(typedNumber('')).toBeNull()
    expect(typedNumber('   ')).toBeNull()
  })

  it('reads what is not a finite number as no number', () => {
    for (const text of ['abc', '1e', '10,2', 'NaN', 'Infinity', '-Infinity']) {
      expect(typedNumber(text), text).toBeNull()
    }
  })
})
