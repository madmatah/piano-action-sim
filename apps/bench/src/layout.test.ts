/**
 * The page reads the engine's description of its controls: a refused edit is
 * said beside the control that made it and no other, and a form's field
 * opens as the quantity is written.
 */
import type { ControlLayout, RefusedEdit } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import { labelOf, madeBy, seedText } from './layout'

const frontEdge: ControlLayout = {
  control: 'slider',
  path: 'key.front_edge_mm',
  label: 'front_edge',
  min: 150,
  max: 340,
  step: 1,
}
const capstan: ControlLayout = { ...frontEdge, path: 'key.capstan.behind_mm', label: 'capstan' }
const balance: ControlLayout = {
  control: 'stack',
  path: 'frame.balance_punching.papers_mm',
  total: 'frame.balance_punching.paper_total_mm',
  stack: 'balance',
  label: 'balance_stack',
}
const front: ControlLayout = {
  ...balance,
  path: 'frame.front_punching.papers_mm',
  total: 'frame.front_punching.paper_total_mm',
  stack: 'front',
  label: 'front_stack',
}
const regulate: ControlLayout = {
  control: 'form',
  action: 'regulate',
  button: 'regulate',
  fields: [],
}
const place: ControlLayout = { control: 'button', action: 'place_frame_under_key', label: 'place' }

const setTo170: RefusedEdit = {
  edit: { edit: 'set', path: 'key.front_edge_mm', value: 170 },
  why: { code: 'outline_crosses_itself', back: 'key_button', front: 'keytop', by_mm: 14 },
}
const liftedFromFront: RefusedEdit = {
  edit: { edit: 'act', action: { action: 'remove_paper', stack: 'front', index: 3 } },
  why: { code: 'no_such_paper', stack: 'front', index: 3, papers: 2 },
}
const regulated: RefusedEdit = {
  edit: { edit: 'act', action: { action: 'regulate', key_height_mm: 65, key_dip_mm: 12 } },
  why: {
    code: 'regulation_infeasible',
    target: 'key_dip',
    wanted_mm: 12,
    limited_by: 'front_pin_in_mortise',
    shortfall_mm: 0.79,
    search_mm: 20,
  },
}

describe('a refused edit', () => {
  it('is said beside the slider whose path was set, and no other', () => {
    expect(madeBy(frontEdge, setTo170)).toBe(true)
    for (const c of [capstan, balance, front, regulate, place])
      expect(madeBy(c, setTo170)).toBe(false)
  })

  it('is said beside the stack a paper was laid on or lifted from, not the other', () => {
    expect(madeBy(front, liftedFromFront)).toBe(true)
    for (const c of [balance, frontEdge, regulate, place]) {
      expect(madeBy(c, liftedFromFront)).toBe(false)
    }
  })

  it('is said beside the control that runs its action', () => {
    expect(madeBy(regulate, regulated)).toBe(true)
    for (const c of [frontEdge, balance, front, place]) expect(madeBy(c, regulated)).toBe(false)
  })
})

describe('a control', () => {
  it('is named by its label, a form by its button', () => {
    expect(labelOf(frontEdge)).toBe('front_edge')
    expect(labelOf(balance)).toBe('balance_stack')
    expect(labelOf(regulate)).toBe('regulate')
  })
})

describe("a form's field", () => {
  it('opens on a target as a technician types it', () => {
    expect(seedText(65, 'target', 0.01)).toBe('65')
    expect(seedText(10.200000000000001, 'target', 0.01)).toBe('10.2')
  })

  it('opens on a measurement as the scale shows it, to its step', () => {
    expect(seedText(13, 'measured', 0.1)).toBe('13.0')
    expect(seedText(121.1234, 'measured', 0.1)).toBe('121.1')
  })
})
