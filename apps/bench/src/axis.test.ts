import { describe, expect, it } from 'vitest'
import { nearest, niceStep, padded, ticks } from './axis'

describe('a chart axis', () => {
  it('steps by one, two or five times a power of ten', () => {
    expect(niceStep(9.78, 8)).toBe(2)
    expect(niceStep(13, 6)).toBe(5)
    expect(niceStep(0.9, 6)).toBe(0.2)
    expect(niceStep(0, 6)).toBe(1)
  })

  it('ticks every multiple of the step in range, nothing written as -0', () => {
    expect(ticks(-13.4, 1.2, 5)).toEqual([-10, -5, 0])
    expect(ticks(0, 9.78, 2)).toEqual([0, 2, 4, 6, 8])
    expect(Object.is(ticks(-0.4, 0.4, 1)[0], 0)).toBe(true)
  })

  it('pads its range, and opens one that stands at one value', () => {
    const [lo, hi] = padded([-12.5, -0.2], 0.1)
    expect(lo).toBeCloseTo(-13.73, 12)
    expect(hi).toBeCloseTo(1.03, 12)
    expect(padded([3, 3], 0.1)).toEqual([2, 4])
    expect(padded([], 0.1)).toEqual([-1, 1])
  })

  it('finds the nearest reading of a curve', () => {
    const travels = [0.05, 0.1, 0.15, 0.2]
    expect(nearest(travels, 0)).toBe(0)
    expect(nearest(travels, 0.12)).toBe(1)
    expect(nearest(travels, 0.13)).toBe(2)
    expect(nearest(travels, 9)).toBe(3)
    expect(nearest([], 1)).toBe(-1)
  })
})
