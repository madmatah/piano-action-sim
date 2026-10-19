/**
 * The French catalog words the engine's codes as the trade says them in
 * French: the glossary's names with Stanwood's English symbols, numbers with
 * a decimal comma, French punctuation, and each code's parameters in the
 * sentence.
 */
import type { Limit, Procedure, Target } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { word } from './catalog'
import { fr as t } from './fr'

/** The no-break space French puts before a colon and inside guillemets. */
const NO_BREAK_SPACE = '\u00a0'

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

/** A regulation refused for `limited_by`, short by 0.42 mm. */
const outOfReach = (target: Target, wanted_mm: number, limited_by: Limit): string =>
  t.regulation.why.regulation_infeasible({
    code: 'regulation_infeasible',
    target,
    wanted_mm,
    limited_by,
    shortfall_mm: 0.42,
    search_mm: 20,
  })

/** Every sentence the catalog holds as it is, with no parameter to word:
 * its notes, its labels and its names. */
function fixedSentences(node: unknown): string[] {
  if (typeof node === 'string') return [node]
  if (node === null || typeof node !== 'object') return []
  return Object.values(node).flatMap(fixedSentences)
}

describe('the punctuation', () => {
  it('puts a no-break space, never a plain one, before a colon, a semicolon, a question or an exclamation mark', () => {
    const broken = fixedSentences(t).filter((sentence) => / [:;?!]/.test(sentence))
    expect(broken).toEqual([])
  })

  it('puts a no-break space inside guillemets', () => {
    const broken = fixedSentences(t).filter((sentence) => /« | »/.test(sentence))
    expect(broken).toEqual([])
  })

  it('writes no decimal point', () => {
    const broken = fixedSentences(t).filter((sentence) => /\d\.\d/.test(sentence))
    expect(broken).toEqual([])
  })
})

describe('a number', () => {
  it('is written with a decimal comma', () => {
    expect(t.number(10.2, 2)).toBe('10,20')
  })

  it('is never written -0', () => {
    expect(t.number(-0.001, 2)).toBe('0,00')
    expect(t.number(-0.006, 2)).toBe('-0,01')
  })

  it('lists as a French sentence does', () => {
    expect(t.list([])).toBe('')
    expect(t.list(['a'])).toBe('a')
    expect(t.list(['a', 'b'])).toBe('a et b')
    expect(t.list(['a', 'b', 'c'])).toBe('a, b et c')
  })
})

describe('the sheet', () => {
  it('writes Stanwood’s English symbol after the French name of each touchweight reading', () => {
    const symbols: [Procedure, string][] = [
      ['down_weight', ' D'],
      ['up_weight', ' U'],
      ['balance_weight', ' BW'],
      ['friction_weight', ' F'],
      ['front_weight', ' FW'],
      ['key_weight_ratio', ' KR'],
    ]
    for (const [procedure, symbol] of symbols) {
      expect(t.procedure[procedure].endsWith(symbol), t.procedure[procedure]).toBe(true)
    }
    expect(t.procedure.key_friction_weight_zeroed_on_the_jig).toContain(' KF, ')
    expect(t.procedure.key_friction_weight_zeroed_on_the_frame).toContain(' KF, ')
  })

  it('gives the reasons for an indicative reading as clauses, each after a no-break space and a semicolon', () => {
    expect(t.reasons(['a', 'b'])).toBe(`a${NO_BREAK_SPACE}; b`)
  })
})

describe('the views', () => {
  it('are named as the owner names them', () => {
    expect([t.view.section, t.view.solid, t.view.band]).toEqual([
      'coupe',
      '3D',
      'courbe d’enfoncement et de remontée',
    ])
  })
})

describe('the papers', () => {
  it('are named by their thickness, as they are laid at the bench', () => {
    expect(t.regulation.pileNote).toContain(
      `«${NO_BREAK_SPACE}ajouter une mouche de 0,2${NO_BREAK_SPACE}»`,
    )
  })

  it('are counted in the singular under two, as French counts', () => {
    expect(t.papers.stack('mouche de balancier', '0,00', 0)).toBe(
      'mouche de balancier **0,00 mm** en 0 mouche',
    )
    expect(t.papers.stack('mouche de balancier', '0,20', 1)).toBe(
      'mouche de balancier **0,20 mm** en 1 mouche',
    )
    expect(t.papers.stack('mouche de balancier', '0,37', 3)).toBe(
      'mouche de balancier **0,37 mm** en 3 mouches',
    )
  })
})

describe('a refusal', () => {
  it('keeps its colon off the start of a line, with a no-break space before it', () => {
    expect(t.refused.set({ code: 'not_finite', param: 'regulation.key_dip_mm' })).toBe(
      `Refusé, rien n’a changé${NO_BREAK_SPACE}: une valeur qui n’est pas un nombre.`,
    )
  })

  it('quotes the panel it sends the reader to between guillemets', () => {
    expect(t.refusal.key_not_weighed({ code: 'key_not_weighed' })).toContain(
      `«${NO_BREAK_SPACE}${t.panel.weighing}${NO_BREAK_SPACE}»`,
    )
  })
})

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
      `Refusé, les mouches restent telles quelles${NO_BREAK_SPACE}: un enfoncement de 12,00 mm est hors d’atteinte, il s’en faut de 0,79 mm${NO_BREAK_SPACE}: la pointe d’enfoncement arrive en haut de sa mortaise avant que la touche ne descende aussi bas.`,
    )
  })

  it('says each limit its own way, for both targets', () => {
    const said: string[] = []
    for (const [target, wanted_mm, wanted] of [
      ['key_height', 65, 'une hauteur de touche de 65,00 mm'],
      ['key_dip', 10.2, 'un enfoncement de 10,20 mm'],
    ] as [Target, number, string][]) {
      for (const limited_by of LIMITS) {
        const sentence = outOfReach(target, wanted_mm, limited_by)
        expect(sentence.startsWith(wanted), sentence).toBe(true)
        expect(sentence).toContain(`il s’en faut de 0,42 mm${NO_BREAK_SPACE}: `)
        expect(sentence.endsWith('.'), sentence).toBe(true)
        said.push(sentence)
      }
    }
    expect(new Set(said).size).toBe(2 * LIMITS.length)
  })

  it('gives one reason for a front with no stop, whichever the target', () => {
    const [height, dip] = [
      outOfReach('key_height', 65, 'no_stop_below_the_front'),
      outOfReach('key_dip', 10.2, 'no_stop_below_the_front'),
    ].map((sentence) => sentence.split(`${NO_BREAK_SPACE}: `)[1])
    expect(height).toBeDefined()
    expect(height).toBe(dip)
  })

  it('says a shortfall under a hundredth as such', () => {
    expect(
      t.regulation.why.regulation_infeasible({
        code: 'regulation_infeasible',
        target: 'key_height',
        wanted_mm: 65,
        limited_by: 'rail_too_low',
        shortfall_mm: 0.004,
        search_mm: 20,
      }),
    ).toBe(
      `une hauteur de touche de 65,00 mm est hors d’atteinte, il s’en faut de moins de 0,01 mm${NO_BREAK_SPACE}: il faudrait plus de 20 mm de mouches en papier sous la mouche de balancier, la barre de balancier étant trop basse.`,
    )
  })
})

describe('a part named in a sentence', () => {
  it('takes its article', () => {
    expect(
      word(t.fault, 'punching_off_underside', {
        code: 'punching_off_underside',
        rail: 'front_rail_punching',
        by_mm: 73,
      }),
    ).toBe('la mouche d’enfoncement se trouve à 73,0 mm en dehors du plat du dessous de la touche')
    expect(
      word(t.editRefusal, 'capstan_in_block', {
        code: 'capstan_in_block',
        block: 'back_check_block',
        by_mm: 9.5,
      }),
    ).toBe('le pilote entrerait de 9,5 mm dans le bloc d’attrape.')
  })

  it('contracts « de le » into « du »', () => {
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'key_tail_felt',
        front: 'back_check_block',
        by_mm: 10,
      }),
    ).toContain('10,0 mm en avant de l’extrémité arrière du bloc d’attrape')
  })

  it('keeps « de la » before a feminine part', () => {
    expect(
      word(t.editRefusal, 'outline_crosses_itself', {
        code: 'outline_crosses_itself',
        back: 'key_tail_felt',
        front: 'key_button',
        by_mm: 10,
      }),
    ).toContain('10,0 mm en avant de l’extrémité arrière de la capsule de touche')
  })

  it('says what does not fit in what, each with its article', () => {
    expect(
      word(t.editRefusal, 'does_not_fit', {
        code: 'does_not_fit',
        part: 'balance_mortise_bushing',
        within: 'key_button',
        by_mm: 1,
      }),
    ).toBe(
      `la garniture de la mortaise de balancier ne tiendrait pas dans la capsule de touche${NO_BREAK_SPACE}: 1,0 mm de trop.`,
    )
  })

  it('says a mortise milled without travel has no wall for its bushing, naming the mortise', () => {
    for (const [mortise, named] of [
      ['balance_mortise', 'la mortaise de balancier'],
      ['front_mortise', 'la mortaise d’enfoncement'],
    ] as const) {
      expect(
        word(t.editRefusal, 'bushing_without_wall', { code: 'bushing_without_wall', mortise }),
      ).toBe(
        `la fraise de ${named} n’a pas de course${NO_BREAK_SPACE}: la mortaise est ronde, sans paroi droite où coller sa garniture.`,
      )
    }
  })

  it('says a groove too long for the front punching would hold the punching', () => {
    expect(
      word(t.editRefusal, 'punching_in_the_groove', {
        code: 'punching_in_the_groove',
        by_mm: 3.83,
      }),
    ).toBe(
      `la fente de la mortaise d’enfoncement serait trop longue de 3,8 mm pour la mouche d’enfoncement${NO_BREAK_SPACE}: à côté de la mortaise d’enfoncement, la mouche ne trouverait plus de bois où porter et se logerait dans la fente.`,
    )
  })
})

describe('a form', () => {
  it('says which fields hold no number, each with its article', () => {
    expect(t.needsNumber.regulate?.(['key_height'])).toBe(
      `il manque un nombre, en millimètres, pour la hauteur de touche${NO_BREAK_SPACE}: rien n’est réglé tant que ce champ est vide.`,
    )
    expect(t.needsNumber.regulate?.(['key_height', 'key_dip'])).toBe(
      `il manque un nombre, en millimètres, pour la hauteur de touche et l’enfoncement${NO_BREAK_SPACE}: rien n’est réglé tant que ces champs sont vides.`,
    )
    expect(t.needsNumber.add_paper).toBeNull()
  })
})

describe('what holds the key', () => {
  it('is said after « la touche »', () => {
    expect(
      t.touch.theKeyIs(
        word(t.touch.verdict, 'held_by_friction', {
          code: 'held_by_friction',
          touching: null,
          margin_g: 1,
        }),
      ),
    ).toBe('la touche est tenue par la seule friction')
  })

  it('names the stops that hold a key lifted off its balance punching, and no friction', () => {
    expect(
      t.touch.verdict.lifts_off_balance_punching({
        code: 'lifts_off_balance_punching',
        touching: ['front_rail_punching', 'front_pin_on_the_wood'],
        needs_friction: false,
        lift_mm: 3.09,
      }),
    ).toBe(
      'est soulevée de 3,09 mm de sa mouche de balancier, retenue par sa mouche d’enfoncement et sa pointe d’enfoncement contre le bois à côté de sa mortaise',
    )
  })

  it('names the friction where friction holds a lifted key on its stops', () => {
    expect(
      t.touch.verdict.lifts_off_balance_punching({
        code: 'lifts_off_balance_punching',
        touching: ['back_rail_cloth'],
        needs_friction: true,
        lift_mm: 0.31,
      }),
    ).toBe(
      'est soulevée de 0,31 mm de sa mouche de balancier, sur son drap de repos arrière, où sa friction la retient',
    )
  })
})
