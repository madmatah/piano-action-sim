/**
 * Single-key bench.
 *
 * Two halves: the sagittal section, which a technician validates by looking,
 * and the weighing sheet, which he validates by reading. Both come from the
 * same engine, so they cannot disagree.
 *
 * Every number on the page is the engine's: each slider shows the value the
 * settings hold, the key friction weight's as the sheet reads it, each
 * reading the sheet's, and nothing here keeps a copy of either to drift from
 * them; the typed fields hold only what is typed until it is sent. Every
 * sentence is the catalogue's (`i18n/`), composed from the codes and numbers
 * the engine sends.
 */

import type { Bench } from '@pas/wasm'
import type {
  Diagnostics,
  FrictionRefusal,
  Procedure,
  Reading,
  Regulation,
  Scene,
  Settings,
  Sheet,
  State,
  WeighingRefusal,
} from '@pas/wasm/types'
import { useCallback, useEffect, useRef, useState } from 'react'
import {
  diagnostics,
  reading,
  regulate,
  scene,
  setKeyFrictionWeight,
  settings,
  sheet,
  start,
  state,
  weighKey,
} from './bench'
import { typedNumber } from './fields'
import { ANGLE_DEG, decimals, fixed, LENGTH_MM, MASS_G, typed } from './format'
import { type Control, t, word } from './i18n'
import { Rich } from './Rich'
import { CONTACT_COLOUR, SagittalView, type ViewMode } from './SagittalView'

/** Everything the engine said when it was last asked. */
interface Engine {
  settings: Settings
  sheet: Sheet
  diagnostics: Diagnostics
  state: State
  scene: Scene
}

/**
 * Applies a bench state given in the URL, so that a particular setup can be
 * linked to rather than described.
 *
 *   ?weight=-20
 *
 * `weight` is the grams on the measuring point. Anything unparseable is
 * ignored rather than guessed at: a malformed link should open the standard
 * bench, not a bench nobody asked for. There is no way to place the key by
 * hand: it is only ever where a weight puts it.
 */
function applyUrlState(b: Bench): void {
  const q = new URLSearchParams(window.location.search)

  const weight = Number(q.get('weight'))
  if (Number.isFinite(weight) && q.has('weight')) b.set_finger_weight(weight)
}

/** `x` to the decimals `resolution` is shown to, as the catalogue writes it. */
function shown(x: number, resolution: number): string {
  return t.number(x, decimals(resolution))
}

/**
 * A quantity the bench reports beside its sheet, to the resolution its kind
 * is shown to, with a target beside it where it has one; or refused, why on
 * hover.
 */
function Row({
  control,
  value,
  resolution,
  target,
  why,
}: {
  control: Control
  /** `null` where it cannot be read. */
  value: number | null
  resolution: number
  /** A second number shown beside the first: what a design aims at. Beside it
      and never instead of it — the reading is what counts, and seeing the
      two together is the point. */
  // `| undefined` is explicit because the project runs with
  // `exactOptionalPropertyTypes`: an optional property and a property that may
  // be undefined are different types, and the distinction is worth keeping.
  target?: number | undefined
  /** Why it cannot be read. */
  why: string | null
}) {
  if (value === null) {
    return (
      <div className="row refused" title={why ?? ''}>
        <span className="label">{control.label}</span>
        <span className="value">{t.sheet.refused}</span>
        <span className="unit" />
        <span className="target" />
      </div>
    )
  }
  return (
    <div className="row">
      <span className="label">{control.label}</span>
      <span className="value">{shown(value, resolution)}</span>
      <span className="unit">{control.unit}</span>
      <span className="target">
        {target === undefined ? '' : t.sheet.target(shown(target, resolution))}
      </span>
    </div>
  )
}

/**
 * One reading of the sheet as the trade reads it, by the protocol's rounding
 * to its resolution, the value in full small beside it, or the target where
 * the reading has one; or that it is refused, the reason on hover and in the
 * note below the sheet.
 */
function Measured({
  s,
  procedure,
  target,
}: {
  s: Sheet
  procedure: Procedure
  target?: number | undefined
}) {
  const m = reading(s, procedure)
  const label = t.procedure[procedure]
  if (m === undefined || m.outcome === 'refused') {
    const why = m?.outcome === 'refused' ? m.why : null
    return (
      <div
        className="row refused"
        data-procedure={procedure}
        data-refusal={why?.code}
        data-riding={why?.code === 'no_travel' ? why.riding?.front : undefined}
        title={why === null ? '' : word(t.refusal, why.code, why)}
      >
        <span className="label">{label}</span>
        <span className="value">{t.sheet.refused}</span>
        <span className="unit" />
        <span className="target" />
      </div>
    )
  }
  const to = decimals(m.resolution)
  const beside =
    target === undefined ? t.number(m.value, to + 2) : t.sheet.target(shown(target, LENGTH_MM))
  return (
    <div
      className={`row ${m.trust.code}`}
      data-procedure={procedure}
      data-shown={m.shown}
      data-exact={m.value}
      title={
        m.trust.code === 'indicative' ? m.trust.because.map((b) => t.because[b]).join('; ') : ''
      }
    >
      <span className="label">{label}</span>
      <span className="value">{t.number(m.shown, to)}</span>
      <span className="unit">{t.unit[m.unit]}</span>
      <span className="target exact">{beside}</span>
    </div>
  )
}

/**
 * One punching stack: the papers on it, and the drawer to add from.
 *
 * Discrete on purpose. A continuous thickness would let the bench reach stacks
 * no drawer can build and teach a resolution the trade does not have.
 */
function PaperStack({
  title,
  papers,
  sizes,
  total,
  onAdd,
  onRemove,
}: {
  title: string
  papers: number[]
  sizes: number[]
  /** The engine's own total for this stack. Summing the papers here instead
   * would be the one number on the page computed outside the engine. */
  total: number
  onAdd: (mm: number) => void
  onRemove: (i: number) => void
}) {
  return (
    <div className="stack">
      <div className="stacktitle">
        <Rich text={t.papers.stack(title, shown(total, LENGTH_MM), papers.length)} />
      </div>
      <div className="drawer">
        {t.papers.add}
        {sizes.map((mm) => (
          <button key={mm} type="button" onClick={() => onAdd(mm)}>
            {shown(mm, LENGTH_MM)}
          </button>
        ))}
      </div>
      <ul className="pile">
        {papers.map((mm, i) => (
          <li key={i}>
            <span>{t.papers.paper(shown(mm, LENGTH_MM))}</span>
            <button type="button" onClick={() => onRemove(i)}>
              {t.papers.liftOff}
            </button>
          </li>
        ))}
        {papers.length === 0 && <li className="empty">{t.papers.bare}</li>}
      </ul>
    </div>
  )
}

/** A control over one setting: its value as the settings hold it. */
function Slider({
  control,
  value,
  min,
  max,
  step,
  resolution = LENGTH_MM,
  onChange,
}: {
  control: Control
  value: number
  min: number
  max: number
  step: number
  /** What its value is shown to: a length to the hundredth of a millimetre
   * unless said otherwise. */
  resolution?: number
  onChange: (v: number) => void
}) {
  return (
    <label className="slider">
      <span>
        {control.label}{' '}
        <b>
          {shown(value, resolution)} {control.unit}
        </b>
      </span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  )
}

/**
 * The bushings' control: the key friction weight Stanwood's protocol reads
 * them as, the sheet's own reading, as the trade reads it with the value in
 * full beside it. Typed in grams, stored as the torque that reads them back.
 * Where the protocol cannot be read on this frame there is no key friction
 * weight to show or to set, and the control says why instead.
 */
function FrictionControl({
  kf,
  onChange,
}: {
  kf: Reading | undefined
  onChange: (g: number) => void
}) {
  const read = kf?.outcome === 'read' ? kf : null
  const why = kf?.outcome === 'refused' ? kf.why : null
  const control = t.friction.control
  const to = read === null ? 0 : decimals(read.resolution)
  return (
    <label
      className="slider friction"
      data-shown={read?.shown}
      data-exact={read?.value}
      data-refusal={why?.code}
      title={why === null ? '' : word(t.refusal, why.code, why)}
    >
      <span>
        {control.label}{' '}
        {read === null ? (
          <b>{t.sheet.refused}</b>
        ) : (
          <>
            <b>
              {t.number(read.shown, to)} {control.unit}
            </b>{' '}
            <small>{t.number(read.value, to + 2)}</small>
          </>
        )}
      </span>
      <input
        type="range"
        min={0}
        max={20}
        step={0.5}
        value={read?.value ?? 0}
        disabled={read === null}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  )
}

/**
 * What the sheet's readings rest on that nobody measured, and why any is
 * refused: one note each, under the weights.
 */
function SheetNotes({ s }: { s: Sheet }) {
  const because = [
    ...new Set(
      s.readings.flatMap((m) =>
        m.outcome === 'read' && m.trust.code === 'indicative' ? m.trust.because : [],
      ),
    ),
  ]
  return (
    <>
      {because.length > 0 && (
        <p className="note">{t.sheet.indicative(because.map((b) => t.because[b]))}</p>
      )}
      {s.readings.map(
        (m) =>
          m.outcome === 'refused' && (
            <p key={m.procedure} className="note degenerate">
              {t.sheet.isRefused(t.procedure[m.procedure], word(t.refusal, m.why.code, m.why))}
            </p>
          ),
      )}
    </>
  )
}

/** The temporary weight a key friction weight stood on the key, if it was read. */
function temporaryWeight(s: Sheet, procedure: Procedure) {
  const m = reading(s, procedure)
  return m?.outcome === 'read' ? m.conditions.temporary_weight : null
}

export function App() {
  const bench = useRef<Bench | undefined>(undefined)
  const [engine, setEngine] = useState<Engine>()
  // The scene as the engine last wrote it: kept while the geometry does not
  // change, so a weight on the key does not rebuild every part's mesh.
  const drawn = useRef<{ json: string; scene: Scene } | undefined>(undefined)
  const [view, setView] = useState<ViewMode>(
    // `?view=section` opens on the flat technical section rather than the
    // solid, so a particular reading can be linked to.
    new URLSearchParams(window.location.search).get('view') === 'section' ? 'section' : 'solid',
  )
  // What is being typed for the next regulation and the next weighing,
  // seeded from the engine at boot: kept as the text in the field, read as a
  // number only when it holds one (`typedNumber`), so that an emptied field
  // is not sent as 0. And what the engine answered the last action that it
  // may refuse, shown until anything else is done.
  const [heightTarget, setHeightTarget] = useState('')
  const [dipTarget, setDipTarget] = useState('')
  const [regulation, setRegulation] = useState<Regulation | null>(null)
  const [frictionRefused, setFrictionRefused] = useState<FrictionRefusal | null>(null)
  const [weighMass, setWeighMass] = useState('')
  const [weighFront, setWeighFront] = useState('')
  const [weighFrom, setWeighFrom] = useState('')
  const [weighRefused, setWeighRefused] = useState<WeighingRefusal | null>(null)

  // Every mutation goes through the engine and the whole page is re-read
  // from it. No value on this page is computed in TypeScript, and none is
  // kept: a frame the key cannot be seated on still has every setting, and
  // its sheet says reading by reading what cannot be read.
  const read = useCallback((b: Bench): Engine => {
    drawn.current = scene(b, drawn.current)
    return {
      settings: settings(b),
      sheet: sheet(b),
      diagnostics: diagnostics(b),
      state: state(b),
      scene: drawn.current.scene,
    }
  }, [])
  const refresh = useCallback(() => {
    const b = bench.current
    if (b) setEngine(read(b))
  }, [read])

  useEffect(() => {
    void start((b) => {
      // Runs exactly once, inside the boot promise: see `start`.
      b.set_finger_weight(0)
      applyUrlState(b)
    }).then((b) => {
      bench.current = b
      const e = read(b)
      // The typed fields start from what the engine holds: the targets it
      // was regulated to; the key's mass, its front weight as the scale
      // reads it, at the measuring point the sheet reads it at.
      setHeightTarget(typed(e.settings.targets.key_height_mm, LENGTH_MM))
      setDipTarget(typed(e.settings.targets.key_dip_mm, LENGTH_MM))
      const fw = reading(e.sheet, 'front_weight')
      setWeighMass(e.settings.key.mass_g === null ? '' : fixed(e.settings.key.mass_g, 1))
      setWeighFront(fw?.outcome === 'read' ? fixed(fw.shown, 1) : '')
      setWeighFrom(fixed(e.settings.measuring_point_from_lip_mm, 1))
      setEngine(e)
    })
  }, [read])

  const act = <T,>(f: (b: Bench) => T): T | undefined => {
    const b = bench.current
    if (!b) return undefined
    const out = f(b)
    setRegulation(null)
    setFrictionRefused(null)
    setWeighRefused(null)
    refresh()
    return out
  }

  // A target field that holds no number is not sent: the button waits, and
  // the page says which field needs one.
  const height = typedNumber(heightTarget)
  const dip = typedNumber(dipTarget)
  const needsNumber = [
    ...(height === null ? (['key_height'] as const) : []),
    ...(dip === null ? (['key_dip'] as const) : []),
  ]
  const regulateTo = () => {
    const b = bench.current
    if (!b || height === null || dip === null) return
    setRegulation(regulate(b, height, dip))
    refresh()
  }

  // The same for the key's weighing: nothing is sent until all three fields
  // hold a number.
  const weighedMass = typedNumber(weighMass)
  const weighedFront = typedNumber(weighFront)
  const weighedFrom = typedNumber(weighFrom)
  const weighNeeds = [
    ...(weighedMass === null ? (['mass'] as const) : []),
    ...(weighedFront === null ? (['front_weight'] as const) : []),
    ...(weighedFrom === null ? (['from_lip'] as const) : []),
  ]
  const weighTheKey = () => {
    if (weighedMass === null || weighedFront === null || weighedFrom === null) return
    const refused = act((b) => weighKey(b, weighedMass, weighedFront, weighedFrom)) ?? null
    setWeighRefused(refused)
  }

  if (!engine) return <div className="booting">{t.booting}</div>
  const s = engine.settings
  const d = engine.diagnostics
  const sh = engine.sheet

  // The contacts the key bears on where it is drawn, by the engine's
  // identifiers: on the view's element too, where the smoke test reads them.
  const touching = engine.state.contacts.filter((c) => c.state === 'closed').map((c) => c.pair)
  // Why the key cannot be seated on this frame, which every reading that
  // seats it is refused for too.
  const seatWhy =
    d.seat_refused === null ? null : word(t.refusal, d.seat_refused.code, d.seat_refused)
  const kf = reading(sh, 'key_friction_weight_zeroed_on_the_jig')
  const seats = d.seats

  return (
    <div className="bench">
      <div className="view" data-touching={touching.join(' ')}>
        <SagittalView scene={engine.scene} state={engine.state} mode={view} />
        <div className="viewtabs">
          <button
            type="button"
            className={view === 'section' ? 'on' : ''}
            onClick={() => setView('section')}
          >
            {t.view.section}
          </button>
          <button
            type="button"
            className={view === 'solid' ? 'on' : ''}
            onClick={() => setView('solid')}
          >
            {t.view.solid}
          </button>
          <span className="hint">{t.view.hint}</span>
          <span className="legend">
            <i style={{ background: CONTACT_COLOUR.closed }} /> {t.view.bears}
            <i style={{ background: CONTACT_COLOUR.open }} /> {t.view.open}
            <i style={{ background: CONTACT_COLOUR.overlapping }} /> {t.view.overlapping}
          </span>
        </div>
      </div>

      <aside className="panel">
        <h1>{t.title}</h1>
        <p className="protocol">
          <Rich text={t.protocol} />
        </p>

        {d.faults.length > 0 && (
          <div className="faults">
            <b>{t.faults.heading}</b>
            <ul>
              {d.faults.map((f, i) => (
                <li key={i} data-code={f.code}>
                  {word(t.fault, f.code, f)}
                </li>
              ))}
            </ul>
            <p>
              <Rich text={t.faults.placement} />
            </p>
          </div>
        )}
        {seatWhy !== null && (
          <p className="degenerate no-weighing" data-code={d.seat_refused?.code}>
            {t.noWeighing(seatWhy)}
          </p>
        )}
        {d.riding && (
          <p
            className="riding degenerate"
            data-front={d.riding.front}
            data-lift-mm={d.riding.lift_mm}
          >
            {t.stopsFight(t.riding(d.riding))}
          </p>
        )}

        <h2>{t.sheet.heading}</h2>
        <Slider
          control={t.sheet.measuringPoint}
          value={s.measuring_point_from_lip_mm}
          min={5}
          max={30}
          // A tenth of a millimetre, so the glossary's 1/2 inch, 12.7 mm, can
          // be set as well as its 13 mm.
          step={0.1}
          onChange={(v) => act((b) => b.set_measuring_point_from_lip(v))}
        />
        <p className="note">{t.sheet.measuringPointNote}</p>
        <Measured s={sh} procedure="down_weight" />
        <Measured s={sh} procedure="up_weight" />
        <Measured s={sh} procedure="balance_weight" />
        <Measured s={sh} procedure="friction_weight" />
        <Measured s={sh} procedure="key_friction_weight_zeroed_on_the_jig" />
        <Measured s={sh} procedure="key_friction_weight_zeroed_on_the_frame" />
        <Measured s={sh} procedure="front_weight" />
        <Measured s={sh} procedure="key_weight_ratio" />
        <p className="note">{t.sheet.gramWeights}</p>
        <p className="note">
          {t.sheet.keyFrictionTwice(
            temporaryWeight(sh, 'key_friction_weight_zeroed_on_the_jig'),
            temporaryWeight(sh, 'key_friction_weight_zeroed_on_the_frame'),
          )}
        </p>
        <SheetNotes s={sh} />
        <Measured s={sh} procedure="key_height" target={s.targets.key_height_mm} />
        <Measured s={sh} procedure="key_dip" target={s.targets.key_dip_mm} />
        <p className="note">{t.sheet.seatingNote}</p>
        <Row
          control={t.sheet.keyMass}
          value={s.key.mass_g}
          resolution={MASS_G}
          why={t.refusal.key_not_weighed({ code: 'key_not_weighed' })}
        />
        <Row
          control={t.sheet.outOfLevel}
          value={seats?.out_of_level_at_bottom_deg ?? null}
          resolution={ANGLE_DEG}
          target={0}
          why={seatWhy}
        />
        <Row
          control={t.sheet.bedding}
          value={seats?.bedding_at_the_back_deg ?? null}
          resolution={ANGLE_DEG}
          target={0}
          why={seatWhy}
        />
        <Row
          control={t.sheet.bearsAt}
          value={seats?.bears_at_the_back_mm ?? null}
          resolution={LENGTH_MM}
          why={seatWhy}
        />
        <p className="note">
          <Rich
            text={
              seats === null
                ? t.sheet.bearingNote
                : `${t.sheet.bearsHow(t.backBearing[seats.back_bearing])} ${t.sheet.bearingNote}`
            }
          />
        </p>
        <p className="note">
          <Rich text={t.sheet.balanceNote} />
        </p>

        <h2>{t.regulation.heading}</h2>
        <div className="targets">
          <label className="field">
            {t.regulation.keyHeight.label}
            <input
              type="number"
              min={40}
              max={90}
              step={0.01}
              value={heightTarget}
              onChange={(e) => setHeightTarget(e.target.value)}
            />
            {t.regulation.keyHeight.unit}
          </label>
          <label className="field">
            {t.regulation.keyDip.label}
            <input
              type="number"
              min={0}
              max={20}
              step={0.01}
              value={dipTarget}
              onChange={(e) => setDipTarget(e.target.value)}
            />
            {t.regulation.keyDip.unit}
          </label>
          <button type="button" onClick={regulateTo} disabled={needsNumber.length > 0}>
            {t.regulation.regulate}
          </button>
        </div>
        {needsNumber.length > 0 && (
          <p className="needs-number degenerate" data-needs={needsNumber.join(' ')}>
            {t.regulation.needsNumber(needsNumber)}
          </p>
        )}
        {regulation && (
          <p
            className={`regulated ${regulation.outcome === 'laid' ? 'note' : 'degenerate'}`}
            data-outcome={regulation.outcome}
            data-code={regulation.outcome === 'refused' ? regulation.why.code : undefined}
            data-limited-by={
              regulation.outcome === 'refused' && regulation.why.code === 'regulation_infeasible'
                ? regulation.why.limited_by
                : undefined
            }
            data-shortfall-mm={
              regulation.outcome === 'refused' && regulation.why.code === 'regulation_infeasible'
                ? regulation.why.shortfall_mm
                : undefined
            }
          >
            {t.regulation.outcome(regulation)}
          </p>
        )}
        <p className="note">{t.regulation.layingNote}</p>
        <p className="note">
          <Rich text={t.regulation.pileNote} />
        </p>
        <PaperStack
          title={t.papers.balance}
          papers={s.papers.balance_mm}
          total={s.papers.balance_total_mm}
          sizes={s.papers.drawer_mm}
          onAdd={(mm) => act((b) => b.add_balance_paper(mm))}
          onRemove={(i) => act((b) => b.remove_balance_paper(i))}
        />
        <PaperStack
          title={t.papers.front}
          papers={s.papers.front_mm}
          total={s.papers.front_total_mm}
          sizes={s.papers.drawer_mm}
          onAdd={(mm) => act((b) => b.add_front_paper(mm))}
          onRemove={(i) => act((b) => b.remove_front_paper(i))}
        />
        <FrictionControl
          kf={kf}
          onChange={(v) => {
            // A weight the engine refuses leaves the bushings as they were,
            // and the control goes on showing what they read.
            const refused = act((b) => setKeyFrictionWeight(b, v)) ?? null
            setFrictionRefused(refused)
          }}
        />
        {frictionRefused && (
          <p
            className="friction-refused degenerate"
            data-code={frictionRefused.code}
            data-why={
              frictionRefused.code === 'protocol_refused' ? frictionRefused.why.code : undefined
            }
          >
            {t.friction.refused(frictionRefused)}
          </p>
        )}
        <p className="note">{t.friction.note}</p>

        <h2>{t.punchings.heading}</h2>
        <p className="note">{t.punchings.note}</p>
        <Slider
          control={t.punchings.balanceFelt}
          value={s.frame.balance_felt_mm}
          min={0}
          max={4}
          step={0.05}
          onChange={(v) => act((b) => b.set_balance_felt(v))}
        />
        <Slider
          control={t.punchings.frontFelt}
          value={s.frame.front_felt_mm}
          min={0}
          max={4}
          step={0.05}
          onChange={(v) => act((b) => b.set_front_felt(v))}
        />
        <Slider
          control={t.punchings.frontPin}
          value={s.frame.front_pin_mm}
          min={5}
          max={40}
          step={0.5}
          onChange={(v) => act((b) => b.set_front_pin_height(v))}
        />
        <Slider
          control={t.punchings.frontMortise}
          value={s.key.front_mortise_depth_mm}
          min={5}
          max={40}
          step={0.5}
          onChange={(v) => act((b) => b.set_front_mortise_depth(v))}
        />
        <p className="note">
          <Rich
            text={t.punchings.stopsAtTheBottom(
              seats === null ? null : seats.bottom_held_by.includes('front_pin_in_the_mortise'),
            )}
          />
        </p>
        <Slider
          control={t.punchings.backCloth}
          value={s.frame.back_rail_cloth_mm}
          min={0}
          max={6}
          step={0.05}
          onChange={(v) => act((b) => b.set_back_rail_cloth(v))}
        />

        <h2>{t.key.heading}</h2>
        <p className="note">
          <Rich text={t.key.ratioNote} />
        </p>
        <Slider
          control={t.key.capstan}
          value={s.key.capstan_behind_mm}
          min={60}
          max={200}
          step={1}
          onChange={(v) => act((b) => b.set_capstan_behind(v))}
        />
        <Slider
          control={t.key.frontEdge}
          value={s.key.front_edge_mm}
          min={150}
          max={340}
          step={1}
          onChange={(v) => act((b) => b.set_front_edge(v))}
        />
        <Slider
          control={t.key.capstanHeight}
          value={s.key.capstan_height_mm}
          min={4}
          max={24}
          step={0.5}
          onChange={(v) => act((b) => b.set_capstan_height(v))}
        />

        <h2>{t.weighing.heading}</h2>
        <p className="note">{t.weighing.intro}</p>
        <div className="targets">
          <label className="field">
            {t.weighing.mass.label}
            <input
              type="number"
              min={1}
              step={0.1}
              value={weighMass}
              onChange={(e) => setWeighMass(e.target.value)}
            />
            {t.weighing.mass.unit}
          </label>
          <label className="field">
            {t.weighing.frontWeight.label}
            <input
              type="number"
              step={0.1}
              value={weighFront}
              onChange={(e) => setWeighFront(e.target.value)}
            />
            {t.weighing.frontWeight.unit}
          </label>
          <label className="field">
            {t.weighing.readAt.label}
            <input
              type="number"
              min={0}
              step={0.1}
              value={weighFrom}
              onChange={(e) => setWeighFrom(e.target.value)}
            />
            {t.weighing.readAt.unit}
          </label>
          <button type="button" onClick={weighTheKey} disabled={weighNeeds.length > 0}>
            {t.weighing.weigh}
          </button>
        </div>
        {weighNeeds.length > 0 && (
          <p className="weighing-needs degenerate" data-needs={weighNeeds.join(' ')}>
            {t.weighing.needsNumber(weighNeeds)}
          </p>
        )}
        {weighRefused && (
          <p className="weigh-refused degenerate" data-code={weighRefused.code}>
            {t.weighing.refused(weighRefused)}
          </p>
        )}
        <p
          className="note weighed"
          data-mass-g={s.key.weighed?.mass_g}
          data-front-weight-g={s.key.weighed?.front_weight_g}
          data-from-lip-mm={s.key.weighed?.from_lip_mm}
        >
          {s.key.weighed === null
            ? t.weighing.notWeighed(s.key.mass_g)
            : t.weighing.weighed(
                s.key.weighed.mass_g,
                s.key.weighed.front_weight_g,
                s.key.weighed.from_lip_mm,
              )}
        </p>
        <p className="note">
          <Rich text={t.weighing.note} />
        </p>

        <h2>{t.frame.heading}</h2>
        <Slider
          control={t.frame.balanceRail}
          value={s.frame.balance_rail_mm}
          min={10}
          max={60}
          step={0.1}
          onChange={(v) => act((b) => b.set_balance_rail_height(v))}
        />
        <Slider
          control={t.frame.frontRail}
          value={s.frame.front_rail_mm}
          min={10}
          max={50}
          step={0.5}
          onChange={(v) => act((b) => b.set_front_rail_height(v))}
        />
        <Slider
          control={t.frame.backRail}
          value={s.frame.back_rail_mm}
          min={10}
          max={60}
          step={0.1}
          onChange={(v) => act((b) => b.set_back_rail_height(v))}
        />
        <p className="note">
          <Rich text={t.frame.railsNote} />
        </p>
        <Slider
          control={t.frame.frontRailAt}
          value={s.frame.front_rail_at_mm}
          min={80}
          max={320}
          step={1}
          onChange={(v) => act((b) => b.set_front_rail_at(v))}
        />
        <Slider
          control={t.frame.backRailAt}
          value={s.frame.back_rail_behind_mm}
          min={120}
          max={280}
          step={1}
          onChange={(v) => act((b) => b.set_back_rail_at(v))}
        />
        <button
          type="button"
          className="replace"
          onClick={() => act((b) => b.place_the_frame_under_the_key())}
        >
          {t.frame.place}
        </button>
        <p className="note">
          <Rich text={t.frame.placeNote} />
        </p>

        <h2>{t.touch.heading}</h2>
        <p className="note">
          <Rich text={t.touch.intro} />
        </p>
        <Slider
          control={t.touch.weight}
          value={s.weight_on_the_measuring_point_g}
          min={-60}
          max={60}
          step={0.5}
          resolution={MASS_G}
          onChange={(v) => act((b) => b.set_finger_weight(v))}
        />
        {d.touch && (
          <div
            className={`verdict ${d.touch.verdict.code === 'held_by_friction' ? 'verdict-held' : 'verdict-stop'}`}
            data-verdict={d.touch.verdict.code}
          >
            {t.touch.theKeyIs(word(t.touch.verdict, d.touch.verdict.code, d.touch.verdict))}
            {d.touch.verdict.code === 'held_by_friction' && (
              <div className="gaps">{t.touch.margin(d.touch.verdict.margin_g)}</div>
            )}
            <div className="gaps">{t.touch.gaps(d.touch.gap_front_mm, d.touch.gap_back_mm)}</div>
            <div className="gaps">
              {t.touch.bushings(
                kf?.outcome === 'read' ? t.number(kf.shown, decimals(kf.resolution)) : null,
              )}
            </div>
          </div>
        )}
        <p className="note">{t.touch.centreNote(d.edge_overlaps)}</p>
        <p className="note">{t.touch.noPressNote}</p>
      </aside>
    </div>
  )
}
