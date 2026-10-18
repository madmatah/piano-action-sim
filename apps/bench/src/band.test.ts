import type { Procedure } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { extremeAt, extremeLabel, type Read } from './band'
import { CATALOG } from './i18n'

/** A curve's extreme as the engine sends it: its value in full, and the gram
 * the walk's rounding would read it as, which the band does not show. */
const extreme = (procedure: Procedure, value: number, shown: number, atMm: number): Read => ({
  outcome: 'read',
  procedure,
  value,
  shown,
  unit: 'g',
  resolution: 1,
  rounding: procedure === 'down_weight_on_the_band' ? 'up' : 'down',
  trust: { code: 'metrological' },
  conditions: {
    mount: 'on_its_punching',
    measuring_point_from_lip_mm: 13,
    held_by: [],
    walk: null,
    sweep: { step_mm: 0.05, samples: 190, window_mm: 9.56, at_mm: atMm },
    temporary_weight: null,
    from: [],
  },
  increments: 1,
  iterations: 1,
  fingerprint: 'statics/2',
})

describe("the band's extremes", () => {
  it('are labeled in full, to the hundredth of a gram, never by the gram', () => {
    // The owner's section weighed at 130 g with a front weight of 40 g: the
    // up curve's least force stands a few hundred-thousandths of a gram
    // inside -46 g, which the walk reads past, at -46.00019 g, and the sheet
    // reads as -47 g. The band shows neither gram.
    const up = extreme('up_weight_on_the_band', -45.99996995201605, -46, 9.55)
    expect(extremeLabel(CATALOG.en, up)).toBe('least up -46.00 g')
    const down = extreme('down_weight_on_the_band', -33.86181548042769, -33, 0.05)
    expect(extremeLabel(CATALOG.en, down)).toBe('greatest down -33.86 g')
    // A value between two grams shows as itself, not as the gram either
    // side of it.
    const between = extreme('up_weight_on_the_band', -12.4999, -13, 9.5)
    expect(extremeLabel(CATALOG.en, between)).toBe('least up -12.50 g')
    for (const m of [up, down, between]) {
      expect(extremeLabel(CATALOG.en, m)).not.toContain(`${m.shown} g`)
    }
  })

  it('say where they fall, for a reader without the chart', () => {
    const down = extreme('down_weight_on_the_band', 0.4, 1, 0.05)
    expect(extremeAt(CATALOG.en, down)).toBe('greatest down 0.40 g, 0.05 mm below the top seat')
  })

  it('are written in the language of the page, with its decimal mark', () => {
    const down = extreme('down_weight_on_the_band', 0.4, 1, 0.05)
    expect(extremeAt(CATALOG.fr, down)).toBe(
      'descente, maximum 0,40 g, à 0,05 mm sous la position de repos',
    )
  })
})
