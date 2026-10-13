/**
 * The bench's controls, generated from the engine's description of them.
 *
 * The registry (`describe()`) gives every path with its unit, what it
 * admits and how it is reached, and the page: its panels in order, each its
 * controls in order, each control with its path or its action, the range it
 * offers and the code its label is worded by. One component draws any of
 * them, its value read from the settings at its path, its wording from the
 * catalogue by its label's code, its range, step and unit from the engine.
 * The page holds no literal of its own for any of these: a control the
 * engine adds, moves or bounds otherwise is drawn as the engine says.
 *
 * Each control carries the engine's identifiers as attributes, `data-path`
 * for a setting, `data-action` for what runs an action, so that a test
 * drives it by what it is and not by its wording, which a catalogue in
 * another language changes.
 */
import type {
  Acted,
  ActionCode,
  ControlLayout,
  Diagnostics,
  FieldLayout,
  Label,
  PanelLayout,
  Param,
  RefusedEdit,
  Registry,
  Settings,
  Sheet,
} from '@pas/wasm/types'
import { Fragment, type ReactNode, useState } from 'react'
import { type Arguments, reading } from './bench'
import { numberAt, numbersAt, param } from './document'
import { typedNumber } from './fields'
import { decimals, LENGTH_MM, RESOLUTION } from './format'
import { t, unitBeside, word } from './i18n'
import { labelOf, madeBy, seedText } from './layout'
import { Rich } from './Rich'

/** What the controls read of the engine: what it said when last asked. */
export interface Read {
  readonly registry: Registry
  readonly settings: Settings
  readonly sheet: Sheet
  readonly diagnostics: Diagnostics
}

/** How a control changes the bench: a path set to a value, or an action run
 * with its arguments. The page re-reads the engine after either. */
export interface Edits {
  set(path: string, value: number): void
  act(action: ActionCode, args: Arguments): void
}

/** The engine's last answer to an action that says more than done: a
 * regulation's papers laid. What the engine keeps of an edit refused is in
 * its diagnostics instead. */
export interface Answer {
  readonly action: ActionCode
  readonly acted: Acted
}

/** `x` to the decimals `resolution` is shown to, as the catalogue writes it. */
export function shown(x: number, resolution: number): string {
  return t.number(x, decimals(resolution))
}

/** What a setting is shown to, by what it measures. */
function resolutionOf(p: Param | undefined): number {
  return p === undefined ? LENGTH_MM : RESOLUTION[p.quantity]
}

/** The unit beside a control's value: its label's wording of its setting's
 * unit. */
function unitOf(label: Label, p: Param | undefined): string {
  return unitBeside(t.control[label], p === undefined ? '' : t.unit[p.unit])
}

/**
 * The edit the engine refused last, said where it was made: the problem in
 * the trade's units, and nothing changed. Its codes are on the element, where
 * a test reads them.
 */
export function Refused({ refused }: { refused: RefusedEdit }) {
  const { edit, why } = refused
  return (
    <p
      className="edit-refused degenerate"
      data-outcome="refused"
      data-for-path={edit.edit === 'set' ? edit.path : undefined}
      data-for-action={edit.edit === 'act' ? edit.action.action : undefined}
      data-code={why.code}
      data-why={why.code === 'protocol_refused' ? why.why.code : undefined}
      data-by-mm={'by_mm' in why ? why.by_mm : undefined}
      data-limited-by={why.code === 'regulation_infeasible' ? why.limited_by : undefined}
      data-shortfall-mm={why.code === 'regulation_infeasible' ? why.shortfall_mm : undefined}
    >
      {t.refused[edit.edit === 'set' ? 'set' : edit.action.action](why)}
    </p>
  )
}

/** A slider over a setting: its value as the settings hold it. */
function Slider({
  c,
  read,
  edits,
}: {
  c: Extract<ControlLayout, { control: 'slider' }>
  read: Read
  edits: Edits
}) {
  const p = param(read.registry, c.path)
  const value = numberAt(read.settings, c.path)
  // Every path a slider reads holds a number (the engine's test of its
  // registry); a slider with none is not drawn rather than drawn at a value
  // the engine does not hold.
  if (value === null) return null
  return (
    <label className="slider" data-path={c.path} data-value={value}>
      <span>
        {t.control[c.label].label}{' '}
        <b>
          {shown(value, resolutionOf(p))} {unitOf(c.label, p)}
        </b>
      </span>
      <input
        type="range"
        min={c.min}
        max={c.max}
        step={c.step}
        value={value}
        onChange={(e) => edits.set(c.path, Number(e.target.value))}
      />
    </label>
  )
}

/**
 * A punching stack: the papers on it, and the drawer to add from.
 *
 * Discrete on purpose. A continuous thickness would let the bench reach stacks
 * no drawer can build and teach a resolution the trade does not have.
 */
function Stack({
  c,
  read,
  edits,
}: {
  c: Extract<ControlLayout, { control: 'stack' }>
  read: Read
  edits: Edits
}) {
  const p = param(read.registry, c.path)
  const sizes = p?.step.step === 'drawer' ? p.step.sizes_mm : []
  const papers = numbersAt(read.settings, c.path)
  // The engine's own total: summing the papers here instead would be the one
  // number on the page computed outside the engine.
  const total = numberAt(read.settings, c.total) ?? 0
  const resolution = resolutionOf(p)
  return (
    <div className="stack" data-path={c.path} data-stack={c.stack}>
      <div className="stacktitle">
        <Rich
          text={t.papers.stack(t.control[c.label].label, shown(total, resolution), papers.length)}
        />
      </div>
      <div className="drawer">
        {t.papers.add}
        {sizes.map((mm) => (
          <button
            key={mm}
            type="button"
            data-action="add_paper"
            data-thickness-mm={mm}
            onClick={() => edits.act('add_paper', { stack: c.stack, thickness_mm: mm })}
          >
            {shown(mm, resolution)}
          </button>
        ))}
      </div>
      <ul className="pile">
        {papers.map((mm, i) => (
          <li key={i}>
            <span>{t.papers.paper(shown(mm, resolution))}</span>
            <button
              type="button"
              data-action="remove_paper"
              data-index={i}
              onClick={() => edits.act('remove_paper', { stack: c.stack, index: i })}
            >
              {t.papers.liftOff}
            </button>
          </li>
        ))}
        {papers.length === 0 && <li className="empty">{t.papers.bare}</li>}
      </ul>
    </div>
  )
}

/** A field's first value, as the page opens: what the bench holds for that
 * quantity, where the registry says to read it. */
function seedOf(f: FieldLayout, read: Read): string {
  const x =
    f.seed.from === 'setting'
      ? numberAt(read.settings, f.seed.at)
      : (() => {
          const m = reading(read.sheet, f.seed.procedure)
          return m?.outcome === 'read' ? m.shown : null
        })()
  return x === null ? '' : seedText(x, param(read.registry, f.path)?.kind, f.step)
}

/**
 * Fields typed and sent together by one action: the regulation's targets,
 * the key's weighing. Each field holds what is typed until it is sent, read
 * as a number only when it holds one (`typedNumber`), so that an emptied
 * field is not sent as 0: the button waits, and the form says which field
 * needs one.
 */
function Form({
  c,
  read,
  edits,
  answer,
}: {
  c: Extract<ControlLayout, { control: 'form' }>
  read: Read
  edits: Edits
  answer: Answer | null
}) {
  // Seeded once, when the page opens on the engine: what is typed afterwards
  // is the technician's until it is sent.
  const [typedIn, setTypedIn] = useState<Readonly<Record<string, string>>>(() =>
    Object.fromEntries(c.fields.map((f) => [f.argument, seedOf(f, read)])),
  )
  const numbers = c.fields.map((f) => ({ f, n: typedNumber(typedIn[f.argument] ?? '') }))
  const needs = numbers.filter(({ n }) => n === null).map(({ f }) => f)
  const send = () => {
    const args: Record<string, number> = {}
    for (const { f, n } of numbers) {
      if (n === null) return
      args[f.argument] = n
    }
    edits.act(c.action, args)
  }
  const needsNumber = t.needsNumber[c.action]
  const laid = answer?.action === c.action && answer.acted.outcome === 'laid' ? answer.acted : null
  return (
    <>
      <div className="targets" data-form={c.action}>
        {c.fields.map((f) => (
          <label key={f.argument} className="field" data-path={f.path} data-argument={f.argument}>
            {t.control[f.label].label}
            <input
              type="number"
              min={f.min ?? undefined}
              max={f.max ?? undefined}
              step={f.step}
              value={typedIn[f.argument] ?? ''}
              onChange={(e) => setTypedIn({ ...typedIn, [f.argument]: e.target.value })}
            />
            {unitOf(f.label, param(read.registry, f.path))}
          </label>
        ))}
        <button type="button" data-action={c.action} onClick={send} disabled={needs.length > 0}>
          {t.control[c.button].label}
        </button>
      </div>
      {needs.length > 0 && needsNumber !== null && (
        <p
          className="needs-number degenerate"
          data-for-action={c.action}
          data-needs={needs.map((f) => f.argument).join(' ')}
        >
          {needsNumber(needs.map((f) => f.label))}
        </p>
      )}
      {laid && (
        <p className="laid note" data-for-action={c.action} data-outcome="laid">
          {t.regulation.laid(laid)}
        </p>
      )}
    </>
  )
}

/**
 * The bushings' control: the key friction weight Stanwood's protocol reads
 * them as, the sheet's own reading, as the trade reads it with the value in
 * full beside it. Typed in grams, stored as the torque that reads them back.
 * Where the protocol cannot be read on this frame there is no key friction
 * weight to show or to set, and the control says why instead.
 */
function KeyFrictionWeight({
  c,
  read,
  edits,
}: {
  c: Extract<ControlLayout, { control: 'key_friction_weight' }>
  read: Read
  edits: Edits
}) {
  const kf = reading(read.sheet, c.reads)
  const r = kf?.outcome === 'read' ? kf : null
  const why = kf?.outcome === 'refused' ? kf.why : null
  const to = r === null ? 0 : decimals(r.resolution)
  return (
    <label
      className="slider friction"
      data-action={c.action}
      data-reads={c.reads}
      data-shown={r?.shown}
      data-exact={r?.value}
      data-refusal={why?.code}
      title={why === null ? '' : word(t.refusal, why.code, why)}
    >
      <span>
        {t.control[c.label].label}{' '}
        {r === null ? (
          <b>{t.sheet.refused}</b>
        ) : (
          <>
            <b>
              {t.number(r.shown, to)} {unitBeside(t.control[c.label], t.unit[r.unit])}
            </b>{' '}
            <small>{t.number(r.value, to + 2)}</small>
          </>
        )}
      </span>
      <input
        type="range"
        min={c.min}
        max={c.max}
        step={c.step}
        value={r?.value ?? 0}
        disabled={r === null}
        onChange={(e) => edits.act(c.action, { [c.argument]: Number(e.target.value) })}
      />
    </label>
  )
}

/**
 * One control of the page, whichever the engine describes, followed by the
 * edit of its the engine refused last, if the last refused is one of its.
 */
export function Control({
  c,
  read,
  edits,
  answer,
}: {
  c: ControlLayout
  read: Read
  edits: Edits
  answer: Answer | null
}) {
  const refused = read.diagnostics.refused_edit
  return (
    <>
      {c.control === 'slider' ? (
        <Slider c={c} read={read} edits={edits} />
      ) : c.control === 'stack' ? (
        <Stack c={c} read={read} edits={edits} />
      ) : c.control === 'form' ? (
        <Form c={c} read={read} edits={edits} answer={answer} />
      ) : c.control === 'key_friction_weight' ? (
        <KeyFrictionWeight c={c} read={read} edits={edits} />
      ) : (
        <button
          type="button"
          className="action"
          data-action={c.action}
          onClick={() => edits.act(c.action, {})}
        >
          {t.control[c.label].label}
        </button>
      )}
      {refused !== null && madeBy(c, refused) && <Refused refused={refused} />}
    </>
  )
}

/** What a panel shows beside its controls: before them, and after each, by
 * its label's code. */
export interface Around {
  readonly before?: ReactNode
  readonly after?: { readonly [K in Label]?: ReactNode }
}

/** A panel of the page: its heading, then its controls in the engine's
 * order, each followed by what the page says after it. */
export function Panel({
  layout,
  around,
  read,
  edits,
  answer,
}: {
  layout: PanelLayout
  around: Around
  read: Read
  edits: Edits
  answer: Answer | null
}) {
  return (
    <>
      <h2 data-panel={layout.panel}>{t.panel[layout.panel]}</h2>
      {around.before}
      {layout.controls.map((c) => {
        const label = labelOf(c)
        return (
          <Fragment key={label}>
            <Control c={c} read={read} edits={edits} answer={answer} />
            {around.after?.[label]}
          </Fragment>
        )
      })}
    </>
  )
}
