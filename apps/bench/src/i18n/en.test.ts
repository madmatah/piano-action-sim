/**
 * The English catalog words the engine's codes as the bench says them:
 * each code's parameters in the sentence, in the trade's units, and every
 * code of a family worded apart.
 */
import type { Limit, Target } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { unitBeside, word } from './catalog'
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
      t.refused.regulate({
        code: 'regulation_infeasible',
        target: 'key_dip',
        wanted_mm: 12,
        limited_by: 'front_pin_in_mortise',
        shortfall_mm: 0.79,
        search_mm: 20,
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
      [{ code: 'mass_not_positive', mass_g: 0 }, 'more than nothing'],
      [
        { code: 'roller_off_the_covering', from_lip_mm: 200, covering_mm: 155 },
        '200.0 mm from the lip is off it',
      ],
      [{ code: 'center_of_mass_off_the_key', at_mm: 1020 }, "past its covering's lip"],
      [
        { code: 'center_of_mass_off_the_key', at_mm: -1020 },
        'behind the balance pin, past its tail',
      ],
      [{ code: 'not_finite', param: 'key.weighing.mass_g' }, 'not a number'],
    ] as const) {
      const said = t.refused.weigh_key(why)
      expect(said.startsWith("Refused, the key's weighing left as it was: "), said).toBe(true)
      expect(said).toContain(words)
    }
  })
})

describe('a refused key friction weight', () => {
  it('says what no friction reads, beside what was typed', () => {
    const said = t.refused.set_friction_from_key_friction_weight({
      code: 'under_no_friction',
      typed_g: 0,
      least_g: 0.3229,
    })
    for (const words of ['0.00 g', '0.32 g', 'no friction at all']) expect(said).toContain(words)
  })

  it('says why the protocol cannot be read, a riding key told how it rides', () => {
    const riding = { front: 'front_rail_punching', lift_mm: 2.2725 } as const
    const said = t.refused.set_friction_from_key_friction_weight({
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

describe('a problem', () => {
  it('names what no part could be built as, the limiting dimension and how far past it', () => {
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'key_button',
        front: 'keytop',
        by_mm: 14,
      }),
    ).toBe(
      'the key button and the keytop would overlap by 14.0 mm along the key: no key is cut like that.',
    )
    expect(
      word(t.editRefusal, 'capstan_in_block', {
        code: 'capstan_in_block',
        block: 'back_check_block',
        by_mm: 9.5,
      }),
    ).toBe('the capstan would stand 9.5 mm into the back check block.')
    expect(
      word(t.editRefusal, 'capstan_behind_back_check_block', {
        code: 'capstan_behind_back_check_block',
        by_mm: 39.5,
      }),
    ).toBe(
      'the capstan would stand behind the back check block: its place is between the block and the key button, and it clears the block 39.5 mm further forward.',
    )
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'key_tail_felt',
        front: 'back_check_block',
        by_mm: 10,
      }),
    ).toBe(
      "the key's thinned tail and its felt would end 10.0 mm in front of the back check block's back end, and the block is glued on the keystick's full height: no key is cut like that.",
    )
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'back_check_block',
        front: 'keystick',
        by_mm: 1,
      }),
    ).toBe(
      "the back check block would stand 1.0 mm across the step down to the key's thinned tail, and a block stands on the one or on the other: no key is cut like that.",
    )
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'top_clearance',
        front: 'capstan',
        by_mm: 2,
      }),
    ).toBe(
      "the wippen rail clearance would run 2.0 mm under the capstan's head, which screws into the keystick's full height: no key is cut like that.",
    )
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'back_check_block',
        front: 'top_clearance',
        by_mm: 1,
      }),
    ).toBe(
      'the back check block and the wippen rail clearance would overlap by 1.0 mm along the key: no key is cut like that.',
    )
    expect(
      word(t.editRefusal, 'feature_outside_keystick', {
        code: 'feature_outside_keystick',
        part: 'top_clearance',
        by_mm: 3,
      }),
    ).toBe(
      "the wippen rail clearance would run 3.0 mm back over the key's thinned tail: it is cut in the keystick's full height.",
    )
    expect(
      word(t.editRefusal, 'capstan_off_the_full_height', {
        code: 'capstan_off_the_full_height',
        by_mm: 0.47,
      }),
    ).toBe(
      "the capstan would stand over the key's thinned tail, and it screws into the keystick's full height: its head stands wholly over it 0.5 mm further forward.",
    )
    expect(
      word(t.editRefusal, 'clearance_not_a_cut', {
        code: 'clearance_not_a_cut',
        length_mm: -5,
        depth_mm: 3,
      }),
    ).toBe(
      "a wippen rail clearance -5.0 mm long and 3.0 mm deep cuts nothing: it is cut below the keystick's full top, its front end in front of its back end.",
    )
    expect(
      word(t.editRefusal, 'clearance_through_keystick', {
        code: 'clearance_through_keystick',
        depth_mm: 25,
        wood_mm: 24.5,
        by_mm: 0.5,
      }),
    ).toBe(
      'a wippen rail clearance 25.0 mm deep goes through the keystick, which has 24.5 mm of wood under it.',
    )
    expect(
      word(t.editRefusal, 'capstan_dome_narrower_than_its_head', {
        code: 'capstan_dome_narrower_than_its_head',
        dome_mm: 3.5,
        head_mm: 3.92,
      }),
    ).toBe('a dome of 3.50 mm cannot cap a capstan head of 3.92 mm radius.')
    expect(
      word(t.editRefusal, 'unknown_profile', { code: 'unknown_profile', profile: 'nobody' }),
    ).toBe('the bench ships no instrument nobody.')
    expect(
      word(t.editRefusal, 'feature_outside_keystick', {
        code: 'feature_outside_keystick',
        part: 'front_mortise',
        by_mm: 2,
      }),
    ).toBe("the front mortise would stand off the flat of the key's underside, by 2.0 mm.")
    expect(
      word(t.editRefusal, 'mortise_through_key_top', {
        code: 'mortise_through_key_top',
        depth_mm: 30,
        wood_mm: 24.5,
        by_mm: 5.5,
      }),
    ).toBe('a front mortise 30.0 mm deep goes through the 24.5 mm of the keystick.')
    expect(
      word(t.editRefusal, 'capsule_ends_overlap', { code: 'capsule_ends_overlap', by_mm: 0.78 }),
    ).toBe(
      "the key button's rounded ends would reach 0.8 mm past each other along its top: they meet at most.",
    )
    expect(
      word(t.editRefusal, 'does_not_fit', {
        code: 'does_not_fit',
        part: 'balance_hole',
        within: 'balance_mortise',
        by_mm: 1,
      }),
    ).toBe('the balance hole would not fit in the balance mortise, by 1.0 mm.')
    expect(
      t.refused.set({
        code: 'outline_crosses_itself',
        back: 'key_button',
        front: 'keytop',
        by_mm: 14,
      }),
    ).toBe(
      'Refused, nothing changed: the key button and the keytop would overlap by 14.0 mm along the key: no key is cut like that.',
    )
  })

  it('says a value out of range in the unit its path names', () => {
    expect(
      word(t.problem, 'out_of_range', {
        code: 'out_of_range',
        param: 'regulation.key_height_mm',
        value: 500,
        min: 0,
        max: 200,
      }),
    ).toBe(
      '500.00 mm is not a value this part can have, no less than 0.00 mm and no more than 200.00 mm.',
    )
    expect(
      word(t.problem, 'out_of_range', {
        code: 'out_of_range',
        param: 'key.friction.torque_gmm',
        value: -1,
        min: 0,
        max: null,
      }),
    ).toBe('-1.00 g·mm is not a value this part can have, no less than 0.00 g·mm.')
  })
})

describe('a form', () => {
  it('says which fields hold no number, each as its form names it', () => {
    expect(t.needsNumber.regulate?.(['key_height'])).toBe(
      'key height needs a number, in millimeters: nothing is regulated until it has one.',
    )
    expect(t.needsNumber.regulate?.(['key_height', 'key_dip'])).toBe(
      'key height and key dip need a number, in millimeters: nothing is regulated until they have one.',
    )
    expect(t.needsNumber.weigh_key?.(['key_mass', 'read_at'])).toBe(
      'the key mass and the distance from the lip need a number: the key is weighed when all three have one.',
    )
    expect(t.needsNumber.add_paper).toBeNull()
  })

  it('says the papers a regulation laid and what they give', () => {
    expect(
      t.regulation.laid({
        outcome: 'laid',
        balance_mm: 0.7,
        front_mm: 0.52,
        key_height_mm: 64.99,
        key_dip_mm: 10.62,
        asked_key_height_mm: 65,
        asked_key_dip_mm: 10.6,
      }),
    ).toBe(
      "Laid 0.70 mm of paper under the balance punching and 0.52 mm on the front punching: 64.99 mm of key height and 10.62 mm of dip, as near 65.00 and 10.60 mm as the drawer's papers come from below.",
    )
  })

  it('says what the key is weighed as, each reading to its resolution', () => {
    expect(t.weighing.weighed(130, 10, 30)).toBe(
      'Weighed: 130.00 g, its front weight 10.00 g read 30.0 mm back from the lip.',
    )
    expect(t.weighing.notWeighed).toMatch(/^Not weighed yet: this key has no mass in the model/)
    // A reading refused for want of a weighing sends the reader to the panel
    // that takes one, by the heading that panel shows.
    expect(t.refusal.key_not_weighed({ code: 'key_not_weighed' })).toContain(
      `“${t.panel.weighing}”`,
    )
    expect(t.weighing.published(120.08)).toBe(
      'As published: 120.08 g, and its center of mass, its height included, as its source gives them.',
    )
  })

  it('names every profile the bench ships, and every way a line is known', () => {
    expect(Object.keys(t.profile)).toEqual(['hirschkorn', 'measured-section'])
    expect(t.view.how.traced).toBe('traced from a figure')
  })

  it('words a control by its label, its unit beside its value', () => {
    expect(t.control.front_edge.label).toBe('front edge from the balance pin')
    expect(unitBeside(t.control.front_edge, t.unit.mm)).toBe('mm')
    expect(unitBeside(t.control.read_at, t.unit.mm)).toBe('mm from the lip')
  })
})

describe('a fault', () => {
  it('says the part, what it is past and by how much, in millimeters', () => {
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
