import { describe, expect, it } from 'vitest'
import { at, numberAt, numbersAt } from './document'

/** A document nested as the settings are, a path's value at its path. */
const settings = {
  protocol: { measuring_point_from_lip_mm: 13 },
  key: { mass_g: null, weighing: null },
  frame: { balance_punching: { felt_mm: 1.5, papers_mm: [0.5, 0.2], paper_total_mm: 0.7 } },
}

describe('a value by its address', () => {
  it('walks the document along the path', () => {
    expect(numberAt(settings, 'protocol.measuring_point_from_lip_mm')).toBe(13)
    expect(numberAt(settings, 'frame.balance_punching.paper_total_mm')).toBe(0.7)
    expect(numbersAt(settings, 'frame.balance_punching.papers_mm')).toEqual([0.5, 0.2])
  })

  it('finds no number where the document holds none, or something else', () => {
    expect(numberAt(settings, 'key.mass_g')).toBeNull()
    expect(numberAt(settings, 'key.weighing.mass_g')).toBeNull()
    expect(numberAt(settings, 'frame.balance_punching')).toBeNull()
    expect(numberAt(settings, 'frame.balance_punching.papers_mm')).toBeNull()
    expect(numberAt(settings, 'no.such.path_mm')).toBeNull()
    expect(numbersAt(settings, 'frame.balance_punching.felt_mm')).toEqual([])
  })

  it('walks into no list and no inherited property', () => {
    expect(at(settings, 'frame.balance_punching.papers_mm.0')).toBeUndefined()
    expect(at(settings, 'protocol.toString')).toBeUndefined()
  })
})
