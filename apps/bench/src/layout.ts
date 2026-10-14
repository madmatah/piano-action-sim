/**
 * How the page reads the engine's description of its controls: what names a
 * control on its panel, which edits are its own, and how a form's field
 * opens. Pure functions of the generated types, apart from the components
 * that draw them (`Control.tsx`), so that a test runs them without the
 * engine.
 */
import type { ActionCode, ControlLayout, Kind, Label, RefusedEdit } from '@pas/wasm/types'
import { decimals, fixed, typed } from './format'

/** The code a control is worded by, which names it on its panel. */
export function labelOf(c: ControlLayout): Label {
  return c.control === 'form' ? c.button : c.label
}

/** The actions a control runs. */
function actionsOf(c: ControlLayout): readonly ActionCode[] {
  switch (c.control) {
    case 'slider':
      return []
    case 'stack':
      return ['add_paper', 'remove_paper']
    case 'form':
    case 'key_friction_weight':
    case 'button':
    case 'profile':
      return [c.action]
  }
}

/** Whether `refused` is an edit `c` made: its path set, or one of its
 * actions run, on its own stack for a stack's. */
export function madeBy(c: ControlLayout, refused: RefusedEdit): boolean {
  const edit = refused.edit
  if (edit.edit === 'set') return c.control === 'slider' && c.path === edit.path
  const action = edit.action
  if (!actionsOf(c).includes(action.action)) return false
  return c.control !== 'stack' || ('stack' in action && action.stack === c.stack)
}

/**
 * A form's field as it opens on `x`: a target written as a technician types
 * it, without the zeros a typed number would not have; a measurement as the
 * scale or the rule shows it, to the field's step, its zeros kept.
 */
export function seedText(x: number, kind: Kind | undefined, step: number): string {
  return kind === 'measured' ? fixed(x, decimals(step)) : typed(x, step)
}
