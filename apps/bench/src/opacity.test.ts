/**
 * The frame is drawn wholly, the key as the reader chose, and a key no
 * equilibrium places fainter than chosen.
 */
import { describe, expect, it } from 'vitest'
import { bodyOpacity, UNPLACED } from './opacity'

/** The opacity a reader chose for the key. */
const CHOSEN = 0.6

describe('a body of the view', () => {
  it('is the frame drawn wholly, whatever is chosen for the key or wherever it stands', () => {
    expect(bodyOpacity({ grounded: true, chosen: CHOSEN, isPlaced: false })).toBe(1)
  })

  it('is a placed key drawn as the reader chose', () => {
    expect(bodyOpacity({ grounded: false, chosen: CHOSEN, isPlaced: true })).toBe(CHOSEN)
  })

  it('is a key no equilibrium places drawn fainter than chosen', () => {
    expect(bodyOpacity({ grounded: false, chosen: CHOSEN, isPlaced: false })).toBe(
      CHOSEN * UNPLACED,
    )
    expect(bodyOpacity({ grounded: false, chosen: 1, isPlaced: false })).toBeLessThan(1)
  })
})
