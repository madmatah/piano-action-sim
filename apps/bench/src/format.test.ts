import { describe, expect, it } from 'vitest'
import { decimals, fixed, isTooSmallToShow, typed } from './format'
import { pieces } from './Rich'

describe('decimals', () => {
  it('reads a resolution as the decimals it is shown to', () => {
    expect([1, 0.5, 0.1, 0.01, 0.001].map(decimals)).toEqual([0, 1, 1, 2, 3])
  })
})

describe('fixed', () => {
  it('never writes -0', () => {
    expect(fixed(-1e-18, 2)).toBe('0.00')
    expect(fixed(-0.25, 1)).toBe('-0.3')
  })
})

describe('isTooSmallToShow', () => {
  it('tells a distance more than nothing that rounds to nothing from one that rounds to its resolution', () => {
    expect(isTooSmallToShow(0.02, 0.1)).toBe(true)
    expect(isTooSmallToShow(0.06, 0.1)).toBe(false)
  })
  it('never calls nothing too small to show', () => {
    expect(isTooSmallToShow(0, 0.1)).toBe(false)
  })
})

describe('typed', () => {
  it('seeds a field with the number as typed, without the noise of a conversion', () => {
    expect(typed(10.200000000000001, 0.01)).toBe('10.2')
    expect(typed(65, 0.01)).toBe('65')
    expect(typed(12.999999999999998, 0.1)).toBe('13')
  })
})

describe('pieces', () => {
  it('reads bold and italic marks, and leaves the rest as it is', () => {
    expect(pieces('a **b** and *c*, d')).toEqual([
      { text: 'a ', mark: null },
      { text: 'b', mark: 'b' },
      { text: ' and ', mark: null },
      { text: 'c', mark: 'i' },
      { text: ', d', mark: null },
    ])
    expect(pieces('no marks')).toEqual([{ text: 'no marks', mark: null }])
  })
})
