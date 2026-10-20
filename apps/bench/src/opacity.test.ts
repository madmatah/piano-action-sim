/**
 * The frame is drawn wholly, the key as the reader chose, its two mortises'
 * bushings apart, and a key no equilibrium places fainter than chosen.
 */
import type { Part } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { bodyOpacity, partOpacity, UNPLACED } from './opacity'

/** The opacity a reader chose for the key. */
const CHOSEN = 0.6

/** The opacity a reader chose for the key's two mortises' bushings. */
const CHOSEN_FOR_BUSHINGS = 0.8

describe('a body of the view', () => {
  it('is the frame drawn wholly, whatever is chosen for the key or wherever it stands', () => {
    expect(
      bodyOpacity({
        grounded: true,
        chosen: CHOSEN,
        chosenForBushings: CHOSEN_FOR_BUSHINGS,
        isPlaced: false,
      }),
    ).toEqual({ parts: 1, bushings: 1 })
  })

  it('is a placed key drawn as the reader chose, its bushings apart', () => {
    expect(
      bodyOpacity({
        grounded: false,
        chosen: CHOSEN,
        chosenForBushings: CHOSEN_FOR_BUSHINGS,
        isPlaced: true,
      }),
    ).toEqual({ parts: CHOSEN, bushings: CHOSEN_FOR_BUSHINGS })
  })

  it('is a key no equilibrium places drawn fainter than chosen, its bushings with it', () => {
    expect(
      bodyOpacity({
        grounded: false,
        chosen: CHOSEN,
        chosenForBushings: CHOSEN_FOR_BUSHINGS,
        isPlaced: false,
      }),
    ).toEqual({ parts: CHOSEN * UNPLACED, bushings: CHOSEN_FOR_BUSHINGS * UNPLACED })
  })

  it('is a key no equilibrium places never drawn whole, even chosen whole', () => {
    const { parts, bushings } = bodyOpacity({
      grounded: false,
      chosen: 1,
      chosenForBushings: 1,
      isPlaced: false,
    })
    expect([parts < 1, bushings < 1]).toEqual([true, true])
  })
})

describe('a part of a body', () => {
  const opacity = { parts: CHOSEN, bushings: CHOSEN_FOR_BUSHINGS }

  /** Which opacity each part the engine sends follows: the two mortises'
   * bushings the bushings', every other part, the key tail felt among the
   * linings, the parts'. */
  const FOLLOWS: Record<Part, 'parts' | 'bushings'> = {
    keystick: 'parts',
    keytop: 'parts',
    key_front: 'parts',
    key_button: 'parts',
    back_check_block: 'parts',
    top_clearance: 'parts',
    key_tail_felt: 'parts',
    capstan: 'parts',
    front_mortise: 'parts',
    front_mortise_groove: 'parts',
    front_mortise_bushing: 'bushings',
    balance_mortise: 'parts',
    balance_mortise_bushing: 'bushings',
    balance_hole: 'parts',
    key_button_side_hole: 'parts',
    balance_rail: 'parts',
    balance_rail_pin: 'parts',
    balance_rail_punching: 'parts',
    front_rail: 'parts',
    front_rail_pin: 'parts',
    front_rail_punching: 'parts',
    paper_punching: 'parts',
    back_rail: 'parts',
    back_rail_cloth: 'parts',
  }

  it('is a mortise’s bushing drawn as the body’s bushings, and any other part as its parts', () => {
    const codes = Object.keys(FOLLOWS) as Part[]
    expect(codes.map((code) => partOpacity({ code }, opacity))).toEqual(
      codes.map((code) => opacity[FOLLOWS[code]]),
    )
  })
})
