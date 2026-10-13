/**
 * The English catalogue words the engine's codes as the bench says them:
 * each code's parameters in the sentence, in the trade's units, and every
 * code of a family worded apart.
 */
import type { Limit, Target } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { word } from './catalogue'
import { en as t } from './en'

/** Every limit, by a record the type-check holds complete. */
const LIMITS = Object.keys({
  front_pin_in_mortise: true,
  front_pin_on_the_wood: true,
  felt_thicker_than_the_stack: true,
  stops_fight: true,
  no_stop_below_the_front: true,
  rail_too_low: true,
  off_its_balance_pin: true,
} satisfies Record<Limit, true>) as Limit[]

describe('a refused regulation', () => {
  it('says the target, the shortfall and the part that keeps it out', () => {
    expect(
      t.regulation.outcome({
        outcome: 'refused',
        why: {
          code: 'regulation_infeasible',
          target: 'key_dip',
          wanted_mm: 12,
          limited_by: 'front_pin_in_mortise',
          shortfall_mm: 0.79,
          search_mm: 20,
        },
      }),
    ).toBe(
      'Refused, the papers left as they were: 12.00 mm of key dip is out of reach, by 0.79 mm: the front rail pin bottoms in its mortise before the key goes that deep.',
    )
  })

  it('says each limit its own way, for both targets', () => {
    const said: string[] = []
    for (const [target, wanted] of [
      ['key_height', '65.00 mm of key height'],
      ['key_dip', '10.20 mm of key dip'],
    ] as [Target, string][]) {
      for (const limited_by of LIMITS) {
        const sentence = t.regulation.why.regulation_infeasible({
          code: 'regulation_infeasible',
          target,
          wanted_mm: target === 'key_height' ? 65 : 10.2,
          limited_by,
          shortfall_mm: 0.42,
          search_mm: 20,
        })
        expect(sentence.startsWith(wanted), sentence).toBe(true)
        expect(sentence).toContain('by 0.42 mm: ')
        expect(sentence.endsWith('.'), sentence).toBe(true)
        said.push(sentence)
      }
    }
    // Nothing stops the front of the key whichever the target: one reason.
    expect(new Set(said).size).toBe(2 * LIMITS.length)
  })

  it('says a shortfall under a hundredth as such', () => {
    const sentence = t.regulation.why.regulation_infeasible({
      code: 'regulation_infeasible',
      target: 'key_height',
      wanted_mm: 65,
      limited_by: 'rail_too_low',
      shortfall_mm: 0.004,
      search_mm: 20,
    })
    expect(sentence).toBe(
      '65.00 mm of key height is out of reach, by under 0.01 mm: it would take more than 20 mm of paper under the balance punching, the balance rail standing too low.',
    )
  })
})

describe('a refused weighing', () => {
  it('says why each is no key', () => {
    for (const [why, words] of [
      [{ code: 'mass_not_positive' }, 'more than nothing'],
      [
        { code: 'roller_off_the_covering', from_lip_mm: 200, covering_mm: 155 },
        '200.0 mm from the lip is off it',
      ],
      [{ code: 'centre_of_mass_off_the_key', at_mm: 1020 }, "past its covering's lip"],
      [
        { code: 'centre_of_mass_off_the_key', at_mm: -1020 },
        'behind the balance pin, past its tail',
      ],
      [{ code: 'not_finite' }, 'not a number'],
    ] as const) {
      const said = t.weighing.refused(why)
      expect(said.startsWith("Refused, the key's weighing left as it was: "), said).toBe(true)
      expect(said).toContain(words)
    }
  })
})

describe('a refused key friction weight', () => {
  it('says what no friction reads, beside what was typed', () => {
    const said = t.friction.refused({ code: 'under_no_friction', typed_g: 0, least_g: 0.3229 })
    for (const words of ['0.00 g', '0.32 g', 'no friction at all']) expect(said).toContain(words)
  })

  it('says why the protocol cannot be read, a riding key told how it rides', () => {
    const riding = { front: 'front_rail_punching', lift_mm: 2.2725 } as const
    const said = t.friction.refused({
      code: 'protocol_refused',
      why: { code: 'no_travel', riding },
    })
    expect(said).toContain("Stanwood's protocol cannot read a key friction weight on this key: ")
    expect(said).toContain(t.riding(riding))
  })
})

describe('a key riding on its stops', () => {
  it('names its stops and its lift, and says it has no stroke', () => {
    const said = t.riding({ front: 'front_rail_punching', lift_mm: 2.2725 })
    for (const words of ['front punching', 'back rail cloth', '2.27 mm', 'no stroke']) {
      expect(said).toContain(words)
    }
    expect(word(t.refusal, 'no_travel', { code: 'no_travel', riding: null })).toBe(
      'the key has no stroke: it stands at the bottom of its dip at rest.',
    )
  })
})

describe('what holds the key', () => {
  it('names what a key lifted off its punching stands on, and friction only where it needs it', () => {
    const tipped = t.touch.verdict.lifts_off_balance_punching({
      code: 'lifts_off_balance_punching',
      touching: ['front_rail_punching', 'front_pin_on_the_wood'],
      needs_friction: false,
      lift_mm: 3.09,
    })
    expect(tipped.startsWith('lifted ')).toBe(true)
    expect(tipped).toContain(
      'held there by its front punching and its front rail pin against the wood',
    )
    expect(tipped).not.toContain('cloth')
    expect(tipped).not.toContain('friction')

    const pulled = t.touch.verdict.lifts_off_balance_punching({
      code: 'lifts_off_balance_punching',
      touching: ['back_rail_cloth'],
      needs_friction: true,
      lift_mm: 0.31,
    })
    expect(pulled).toContain('on its back rail cloth, held there by its friction')
    expect(pulled).not.toContain('front')
  })

  it('names the stop friction holds it on, or none', () => {
    expect(
      t.touch.verdict.held_by_friction({
        code: 'held_by_friction',
        touching: 'back_rail_cloth',
        margin_g: 1,
      }),
    ).toBe('on the back rail cloth, held by friction')
    expect(
      t.touch.verdict.held_by_friction({ code: 'held_by_friction', touching: null, margin_g: 1 }),
    ).toBe('held by friction alone')
  })

  it('names the friction once, as Stanwood’s protocol reads it', () => {
    expect(t.touch.bushings('7')).toContain('key friction weight of 7 g')
    expect(t.touch.bushings(null)).not.toMatch(/\d/)
  })
})

describe('a fault', () => {
  it('says the part, what it is past and by how much, in millimetres', () => {
    expect(word(t.fault, 'pin_outside_mortise', { code: 'pin_outside_mortise', by_mm: 16.5 })).toBe(
      "the front rail pin stands 16.5 mm outside the key's mortise",
    )
    expect(
      word(t.fault, 'punching_off_underside', {
        code: 'punching_off_underside',
        rail: 'front_rail_punching',
        by_mm: 73,
      }),
    ).toBe("the front rail punching stands 73.0 mm off the flat of the key's underside")
  })
})

describe('a number', () => {
  it('is never written -0', () => {
    expect(t.number(-0.001, 2)).toBe('0.00')
    expect(t.number(-0.006, 2)).toBe('-0.01')
  })

  it('lists as a sentence does', () => {
    expect(t.list([])).toBe('')
    expect(t.list(['a'])).toBe('a')
    expect(t.list(['a', 'b'])).toBe('a and b')
    expect(t.list(['a', 'b', 'c'])).toBe('a, b and c')
  })
})
