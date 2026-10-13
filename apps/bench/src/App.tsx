/**
 * Single-key bench.
 *
 * Two halves: the sagittal section, which a technician validates by looking,
 * and the weighing sheet, which he validates by reading. Both come from the
 * same engine call, so they cannot disagree.
 */

import type { Bench } from '@pas/wasm'
import { useCallback, useEffect, useRef, useState } from 'react'
import {
  type Readout,
  type Regulation,
  reading,
  readout,
  regulate,
  type Scene,
  scene,
  setKeyFrictionWeight,
  start,
} from './bench'
import { typedNumber } from './fields'
import { CONTACT_COLOUR, SagittalView, type ViewMode } from './SagittalView'

interface State {
  scene: Scene
  /** The last sheet the engine could produce. Kept when it can no longer
   * produce one, so the settings that caused it stay on the page. */
  readout: Readout | null
  /** Whether that sheet is the last good one rather than the current one. */
  stale: boolean
}

/**
 * Applies a bench state given in the URL, so that a particular setup can be
 * linked to rather than described.
 *
 *   ?leads=10@150,8@190&weight=-20
 *
 * `leads` is a comma-separated list of `grams@millimetres-from-the-balance-pin`,
 * `weight` the grams on the measuring point. Anything unparseable is ignored
 * rather than guessed at: a malformed link should open the standard bench, not
 * a bench nobody asked for. There is no way to place the key by hand: it is
 * only ever where a weight puts it.
 */
function applyUrlState(b: Bench): void {
  const q = new URLSearchParams(window.location.search)

  for (const spec of (q.get('leads') ?? '').split(',')) {
    const m = /^\s*([\d.]+)@(-?[\d.]+)\s*$/.exec(spec)
    if (!m) continue
    const [mass, at] = [Number(m[1]), Number(m[2])]
    if (Number.isFinite(mass) && Number.isFinite(at)) b.add_lead(at, mass)
  }

  const weight = Number(q.get('weight'))
  if (Number.isFinite(weight) && q.has('weight')) b.set_finger_weight(weight)
}

/**
 * `x` to `digits` places, a reading that rounds to nothing shown as 0, never
 * as -0: a key level to a few attometres at the bottom of its dip is level.
 */
function fixed(x: number, digits: number): string {
  const shown = x.toFixed(digits)
  return Number(shown) === 0 ? (0).toFixed(digits) : shown
}

function Row({
  label,
  value,
  unit,
  target,
  targetLabel = 'target',
}: {
  label: string
  value: number
  unit: string
  /** A second number shown beside the first: the regulation target, or what a
      protocol measured. Beside it and never instead of it — the measurement is
      what counts, and seeing the two together is the point. */
  // `| undefined` is explicit because the project runs with
  // `exactOptionalPropertyTypes`: an optional property and a property that may
  // be undefined are different types, and the distinction is worth keeping.
  target?: number | undefined
  targetLabel?: string
}) {
  return (
    <div className="row">
      <span className="label">{label}</span>
      <span className="value">{fixed(value, 2)}</span>
      <span className="unit">{unit}</span>
      <span className="target">
        {target === undefined ? '' : `${targetLabel} ${fixed(target, 2)}`}
      </span>
    </div>
  )
}

/** What each reading of the sheet is called, by the engine's identifier. */
const LABEL: Record<string, string> = {
  front_weight: 'front weight FW',
  key_weight_ratio: 'key weight ratio KR',
  key_height: 'key height',
  key_dip: 'key dip',
  down_weight: 'down weight D',
  up_weight: 'up weight U',
  balance_weight: 'balance weight BW',
  friction_weight: 'friction weight F',
  key_friction_weight_zeroed_on_the_jig: 'key friction weight KF, zeroed on the jig',
  key_friction_weight_zeroed_on_the_frame: 'key friction weight KF, zeroed on the frame',
}

/** What makes a reading indicative, as the sheet's note says it. */
const BECAUSE: Record<string, string> = {
  key_mass: 'the key’s mass is derived from its volume at an assumed wood density',
  key_friction: 'its bushings’ friction is the bench’s assumption until you set it',
  geometry: 'some of its dimensions have no record',
}

/**
 * The decimals a reading is shown to, from the resolution the protocol reads
 * it to: none for the gram, one for a tenth or a half gram, two for a
 * hundredth or a paper.
 */
function decimals(resolution: number): number {
  return resolution >= 1 ? 0 : resolution >= 0.1 ? 1 : 2
}

/**
 * One reading of the sheet as the trade reads it, the value in full small
 * beside it, or the target where the reading has one; or that it is
 * refused, the engine's reason on hover and in the note below the sheet.
 */
function Measured({
  r,
  procedure,
  target,
}: {
  r: Readout
  procedure: string
  target?: number | undefined
}) {
  const m = reading(r, procedure)
  const label = LABEL[procedure] ?? procedure
  if (!m || m.shown === null || m.exact === null) {
    return (
      <div className="row refused" data-procedure={procedure} title={m?.refused ?? ''}>
        <span className="label">{label}</span>
        <span className="value">refused</span>
        <span className="unit" />
        <span className="target" />
      </div>
    )
  }
  const shownTo = decimals(m.resolution)
  const beside = target === undefined ? fixed(m.exact, shownTo + 2) : `target ${fixed(target, 2)}`
  return (
    <div
      className={`row ${m.trust}`}
      data-procedure={procedure}
      data-shown={m.shown}
      data-exact={m.exact}
      title={m.because.map((b) => BECAUSE[b] ?? b).join('; ')}
    >
      <span className="label">{label}</span>
      <span className="value">{fixed(m.shown, shownTo)}</span>
      <span className="unit">{m.unit}</span>
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
   * would be the one number on the page computed outside the engine — and it
   * would be summed from values already narrowed to f32. */
  total: number
  onAdd: (mm: number) => void
  onRemove: (i: number) => void
}) {
  return (
    <div className="stack">
      <div className="stacktitle">
        {title} <b>{total.toFixed(2)} mm</b> in {papers.length}{' '}
        {papers.length === 1 ? 'paper' : 'papers'}
      </div>
      <div className="drawer">
        add
        {sizes.map((mm) => (
          <button key={mm} type="button" onClick={() => onAdd(mm)}>
            {mm.toFixed(2)}
          </button>
        ))}
      </div>
      <ul className="pile">
        {papers.map((mm, i) => (
          <li key={i}>
            <span>{mm.toFixed(2)} mm</span>
            <button type="button" onClick={() => onRemove(i)}>
              lift off
            </button>
          </li>
        ))}
        {papers.length === 0 && <li className="empty">bare rail</li>}
      </ul>
    </div>
  )
}

function Slider({
  label,
  value,
  min,
  max,
  step,
  unit,
  onChange,
}: {
  label: string
  value: number
  min: number
  max: number
  step: number
  unit: string
  onChange: (v: number) => void
}) {
  return (
    <label className="slider">
      <span>
        {label}{' '}
        <b>
          {value.toFixed(2)} {unit}
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

/** The temporary weight a key friction weight stood on the key, as the note says it. */
function kfWeight(r: Readout, procedure: string): string {
  const m = reading(r, procedure)
  if (m?.temporary_weight_g == null || m.temporary_weight_at_mm == null) return ''
  const at = m.temporary_weight_at_mm
  return ` (${fixed(m.temporary_weight_g, 1)} g, ${fixed(Math.abs(at), 0)} mm ${
    at < 0 ? 'behind' : 'in front of'
  } the balance pin)`
}

/**
 * What the sheet's readings rest on that nobody measured, and why any is
 * refused: one note each, under the weights.
 */
function sheetNotes(r: Readout) {
  const because = [...new Set(r.sheet.flatMap((m) => m.because))]
  const refused = r.sheet.filter((m) => m.refused !== null)
  return (
    <>
      {because.length > 0 && (
        <p className="note">
          These readings are indicative, not measurements:{' '}
          {because.map((b) => BECAUSE[b] ?? b).join('; ')}.
        </p>
      )}
      {refused.map((m) => (
        <p key={m.procedure} className="note degenerate">
          {LABEL[m.procedure] ?? m.procedure} is refused: {m.refused}
        </p>
      ))}
    </>
  )
}

export function App() {
  const bench = useRef<Bench | undefined>(undefined)
  const [state, setState] = useState<State>()

  // The key friction weight the bushings read by Stanwood's protocol, the
  // one it is typed as: taken from the sheet at every refresh, so it follows
  // the bushings as the measuring point and the key are changed. Why the last
  // one typed was refused, the engine's sentence, shown until anything else
  // is done.
  const [friction, setFriction] = useState(Number.NaN)
  const [frictionRefused, setFrictionRefused] = useState<string | null>(null)
  const [finger, setFinger] = useState(0)
  const [view, setView] = useState<ViewMode>(
    // `?view=section` opens on the flat technical section rather than the
    // solid, so a particular reading can be linked to.
    new URLSearchParams(window.location.search).get('view') === 'section' ? 'section' : 'solid',
  )
  const [balanceFelt, setBalanceFelt] = useState(1.5)
  const [frontFelt, setFrontFelt] = useState(2.0)
  const [backCloth, setBackCloth] = useState(3.0)
  const [capstan, setCapstan] = useState(129.5)
  const [frontEdge, setFrontEdge] = useState(258)
  const [capstanHeight, setCapstanHeight] = useState(12)
  const [balanceRail, setBalanceRail] = useState(40)
  const [frontRail, setFrontRail] = useState(30.7)
  const [backRail, setBackRail] = useState(39)
  const [frontRailAt, setFrontRailAt] = useState(225)
  const [backRailAt, setBackRailAt] = useState(200)
  const [frontPin, setFrontPin] = useState(24)
  const [frontMortise, setFrontMortise] = useState(22)
  // No default is written here: the engine holds it, and the boot below seeds
  // this before the panel first renders. NaN rather than a plausible number,
  // so that an unseeded value could never pass for the protocol's.
  const [measuringPoint, setMeasuringPoint] = useState(Number.NaN)
  // The targets typed for the next regulation, as typed, seeded from the
  // engine's at boot, and how the last one asked for came out: the engine's
  // sentence, shown until anything else is done. Kept as the text in the
  // field, read as a number only when it holds one (`typedNumber`), so that an
  // emptied field is not sent as a target of 0 mm.
  const [heightTarget, setHeightTarget] = useState('')
  const [dipTarget, setDipTarget] = useState('')
  const [regulation, setRegulation] = useState<Regulation | null>(null)
  const [newLeadMass, setNewLeadMass] = useState(10)

  // Every mutation goes through the engine and the whole view is re-read from
  // it. No value on this page is computed in TypeScript.
  //
  // A frame can be edited into one no key can sit in, and then there is no
  // weighing. The page **keeps the last sheet it had** and says it is stale,
  // rather than emptying itself: the sliders live in that panel, so hiding it
  // would leave the user with no way back out of the frame they just built.
  const refresh = useCallback(() => {
    const b = bench.current
    if (!b) return
    const fresh = readout(b)
    setState((prev) => ({
      scene: scene(b),
      readout: fresh ?? prev?.readout ?? null,
      stale: fresh === null,
    }))
    // The bushings are a torque and stay as they were set; what Stanwood's
    // protocol reads them as moves with the measuring point and the key. The
    // friction slider takes it from the engine rather than keep a number the
    // key no longer reads.
    const kf = fresh ? reading(fresh, 'key_friction_weight_zeroed_on_the_jig')?.exact : null
    if (kf != null) setFriction(kf)
  }, [])

  useEffect(() => {
    void start((b) => {
      // Runs exactly once, inside the boot promise: see `start`.
      b.set_finger_weight(0)
      applyUrlState(b)
    }).then((b) => {
      bench.current = b
      const r = readout(b)
      if (r) {
        // Seed every control from the engine, so no slider starts out lying
        // about what the model holds.
        setBalanceFelt(r.balance_felt_mm)
        setFrontFelt(r.front_felt_mm)
        setBackCloth(r.back_cloth_mm)
        setCapstan(r.capstan_behind_mm)
        setFrontEdge(r.front_edge_mm)
        setCapstanHeight(r.capstan_height_mm)
        setBalanceRail(r.balance_rail_mm)
        setFrontRail(r.front_rail_mm)
        setBackRail(r.back_rail_mm)
        setFrontRailAt(r.front_rail_at_mm)
        setBackRailAt(r.back_rail_at_mm)
        // The probe too: `?weight=` can set the engine before this runs, and a
        // slider that disagrees with the sheet beside it is worse than no
        // slider.
        setFinger(r.finger_weight_g)
        setFrontPin(r.front_pin_mm)
        setFrontMortise(r.front_mortise_mm)
        setMeasuringPoint(r.measuring_point_from_lip_mm)
        setHeightTarget(String(r.key_height_target_mm))
        setDipTarget(String(r.key_dip_target_mm))
      }
      refresh()
    })
  }, [refresh])

  const act = <T,>(f: (b: Bench) => T): T | undefined => {
    const b = bench.current
    if (!b) return undefined
    const out = f(b)
    setRegulation(null)
    setFrictionRefused(null)
    refresh()
    return out
  }

  // A target field that holds no number is not sent: the button waits, and
  // the page says which field needs one.
  const height = typedNumber(heightTarget)
  const dip = typedNumber(dipTarget)
  const needsNumber = [
    ...(height === null ? ['key height'] : []),
    ...(dip === null ? ['key dip'] : []),
  ]
  const regulateTo = () => {
    const b = bench.current
    if (!b || height === null || dip === null) return
    setRegulation(regulate(b, height, dip))
    refresh()
  }

  if (!state) return <div className="booting">starting the engine…</div>
  const r = state.readout
  const fault = state.stale ? (state.scene.frame_fault ?? 'the geometry is degenerate.') : null

  // The contacts the key bears on where it is drawn, by the engine's
  // identifiers: on the view's element too, where the smoke test reads them.
  const touching = state.scene.contacts.filter((c) => c.state === 'closed').map((c) => c.pair)
  const gap = (mm: number | null) => (mm === null ? 'none' : `${fixed(mm, 3)} mm`)

  return (
    <div className="bench">
      <div className="view" data-touching={touching.join(' ')}>
        <SagittalView scene={state.scene} mode={view} />
        <div className="viewtabs">
          <button
            type="button"
            className={view === 'section' ? 'on' : ''}
            onClick={() => setView('section')}
          >
            section
          </button>
          <button
            type="button"
            className={view === 'solid' ? 'on' : ''}
            onClick={() => setView('solid')}
          >
            solid
          </button>
          <span className="hint">drag to turn · scroll to zoom</span>
          <span className="legend">
            <i style={{ background: CONTACT_COLOUR.closed }} /> bears
            <i style={{ background: CONTACT_COLOUR.open }} /> open, under 1 mm
            <i style={{ background: CONTACT_COLOUR.overlapping }} /> drawn parts overlap
          </span>
        </div>
      </div>

      <aside className="panel">
        <h1>Single-key bench</h1>
        <p className="protocol">
          Weighed by Stanwood’s <i>Standard Protocols of the New Touchweight Metrology</i>, February
          2000. Every weight is centred on the measuring point, marked on the section, a set
          distance back from the front lip of the key.
        </p>

        {state.scene.faults.length > 0 && (
          <div className="faults">
            <b>This is not a keyboard anybody could build.</b>
            <ul>
              {state.scene.faults.map((f) => (
                <li key={f}>{f}</li>
              ))}
            </ul>
            <p>
              The rails are placed under the key when the frame is built, and then they stay bolted
              where they are. Change the key afterwards and they do not follow — a real keybed does
              not either. <i>Place the frame under the key</i>, under The frame, unbolts them and
              stands them where this key needs them.
            </p>
          </div>
        )}
        {fault && (
          <p className="degenerate">
            No weighing is possible: {fault} The sheet below is the last one that was, kept so the
            settings stay reachable.
          </p>
        )}
        {r === null ? (
          <p className="degenerate">Nothing has been weighed yet.</p>
        ) : (
          <>
            <h2>Weighing sheet</h2>
            <Slider
              label="measuring point, back from the key’s front lip"
              value={measuringPoint}
              min={5}
              max={30}
              // A tenth of a millimetre, so the glossary's 1/2 inch, 12.7 mm,
              // can be set as well as its 13 mm.
              step={0.1}
              unit="mm"
              onChange={(v) => {
                setMeasuringPoint(v)
                act((b) => b.set_measuring_point_from_lip(v))
              }}
            />
            <p className="note">
              The lip is the covering’s, which overhangs the key front. Moving the point changes
              every weight on this sheet by the lever and nothing about the key: the same static
              moment, and the same friction torque, read on another arm. Key height and dip are read
              at the key front, and do not move.
            </p>
            <Measured r={r} procedure="down_weight" />
            <Measured r={r} procedure="up_weight" />
            <Measured r={r} procedure="balance_weight" />
            <Measured r={r} procedure="friction_weight" />
            <Measured r={r} procedure="key_friction_weight_zeroed_on_the_jig" />
            <Measured r={r} procedure="key_friction_weight_zeroed_on_the_frame" />
            <Measured r={r} procedure="front_weight" />
            <Measured r={r} procedure="key_weight_ratio" />
            <p className="note">
              Every weight is read the way gram weights give it: the down weight and the key
              friction weight as the least whole gram that takes the key down, the up weight as the
              most whole grams it lifts, balance and friction weight from those two, so to the half
              gram; the front weight to the tenth, as a scale reads it. The value in full stands
              small beside each. D is walked up from the key resting on its cloth, U down from the
              key on its front punching, each a gram at a time until the key goes all the way.
            </p>
            <p className="note">
              The key friction weight is read twice. Stanwood’s protocol zeroes the front weight on
              the levelled jig with a temporary weight
              {kfWeight(r, 'key_friction_weight_zeroed_on_the_jig')}, then puts the key back on its
              frame and finds the least weight that makes it fall. The other cancels the key’s
              balance where it rests on its frame
              {kfWeight(r, 'key_friction_weight_zeroed_on_the_frame')}, which reads the bushings
              alone. They differ by what the key’s tilt at rest does to the weights’ arms.
            </p>
            {sheetNotes(r)}
            <Measured r={r} procedure="key_height" target={r.key_height_target_mm} />
            <Measured r={r} procedure="key_dip" target={r.key_dip_target_mm} />
            <p className="note">
              Both read at the front of the key with the key seated the way a technician seats it:
              the back of the key held down at the capstan, where the action holds it, for the
              height; the front then pressed down until a stop takes it, for the dip — the punching,
              or the pin in its mortise if that comes first. Whatever the weight below is doing to
              the key, these do not move.
            </p>
            <Row label="key mass" value={r.key_mass_g} unit="g" />
            <Row label="out of level, key down" value={r.out_of_level_deg} unit="°" target={0} />
            <Row
              label="bedding at the back"
              value={r.bedding_at_the_back_deg}
              unit="°"
              target={0}
            />
            <Row label="bears at" value={r.bears_at_the_back_mm} unit="mm behind" />
            <p className="note">
              The key is <b>{r.back_bearing}</b>. A keyboard is set up so the key is{' '}
              <b>level when it is fully down</b>, and so slightly nose-up when released — and the
              chamfer under the tail is cut at that same angle so the tail beds <b>flat</b> on its
              cloth. Both are design goals, not laws: rail heights, punchings and cloth are what
              reach them, and plenty of keyboards miss. A key that misses bears on an edge instead,
              which is where it bears from — so the bench reports all three rather than refusing the
              frame. Nose-down is positive, and so is a rear end digging in.
            </p>
            <p className="note">
              With no action on the capstan, <b>BW = −FW</b>: the equation of balance with its
              right-hand side removed, but for what the key’s rest attitude adds, read here at its
              two seats rather than level. A bare front-heavy key falls on its own, which is why
              these branches are negative.
            </p>

            <h2>Regulation</h2>
            <div className="targets">
              <label className="field">
                key height
                <input
                  type="number"
                  min={40}
                  max={90}
                  step={0.01}
                  value={heightTarget}
                  onChange={(e) => setHeightTarget(e.target.value)}
                />
                mm
              </label>
              <label className="field">
                key dip
                <input
                  type="number"
                  min={0}
                  max={20}
                  step={0.01}
                  value={dipTarget}
                  onChange={(e) => setDipTarget(e.target.value)}
                />
                mm
              </label>
              <button type="button" onClick={regulateTo} disabled={needsNumber.length > 0}>
                regulate to these targets
              </button>
            </div>
            {needsNumber.length > 0 && (
              <p className="needs-number degenerate">
                {needsNumber.join(' and ')} {needsNumber.length === 1 ? 'needs' : 'need'} a number,
                in millimetres: nothing is regulated until{' '}
                {needsNumber.length === 1 ? 'it has' : 'they have'} one.
              </p>
            )}
            {regulation && (
              <p className={`regulated ${regulation.laid ? 'note' : 'degenerate'}`}>
                {regulation.said}
              </p>
            )}
            <p className="note">
              The bench lays the papers as a technician does: the key height first, on the balance
              punching, then the dip on the front punching with the key as it then stands, each the
              closest stack the drawer makes from below. So it lands within one paper of each
              target, not on it, and the sheet shows both. A target no stack reaches is refused,
              with what keeps it out and by how many millimetres, and nothing is changed.
            </p>
            <p className="note">
              A punching stack is a <b>pile of papers</b>, not a thickness. The operation is “lay a
              two-tenths”, never “set it to 0.37”, so these are the papers in the drawer and the
              pile you have built.
            </p>
            <PaperStack
              title="balance punching — key height"
              papers={r.balance_papers_mm}
              total={r.balance_paper_mm}
              sizes={r.paper_sizes_mm}
              onAdd={(mm) => act((b) => b.add_balance_paper(mm))}
              onRemove={(i) => act((b) => b.remove_balance_paper(i))}
            />
            <PaperStack
              title="front punching — key dip"
              papers={r.front_papers_mm}
              total={r.front_paper_mm}
              sizes={r.paper_sizes_mm}
              onAdd={(mm) => act((b) => b.add_front_paper(mm))}
              onRemove={(i) => act((b) => b.remove_front_paper(i))}
            />
            <Slider
              label="key friction weight"
              value={friction}
              min={0}
              max={20}
              step={0.5}
              unit="g"
              onChange={(v) => {
                // A weight the engine refuses leaves the bushings as they were,
                // so the slider goes back to what they read, even on a frame no
                // sheet can be read on to say so.
                const was = friction
                setFriction(v)
                const refused = act((b) => setKeyFrictionWeight(b, v)) ?? null
                if (refused !== null) setFriction(was)
                setFrictionRefused(refused)
              }}
            />
            {frictionRefused && <p className="friction-refused degenerate">{frictionRefused}</p>}
            <p className="note">
              The key friction weight is typed as Stanwood’s protocol reads it, zeroed on the
              levelled jig: the bench sets the bushings to the torque at which that protocol, run on
              this key, reads back what is typed, and the sheet shows it. The bushings then stay as
              they are. Moving the measuring point or changing the key changes the grams they read,
              as it would at the bench, and this slider follows the sheet.
            </p>

            <h2>Punchings and cloth</h2>
            <p className="note">
              Paper is how you adjust, felt is what you fit. The rest geometry cares only about the
              total; the split will matter once the felts are allowed to compress.
            </p>
            <Slider
              label="balance felt"
              value={balanceFelt}
              min={0}
              max={4}
              step={0.05}
              unit="mm"
              onChange={(v) => {
                setBalanceFelt(v)
                act((b) => b.set_balance_felt(v))
              }}
            />
            <Slider
              label="front felt"
              value={frontFelt}
              min={0}
              max={4}
              step={0.05}
              unit="mm"
              onChange={(v) => {
                setFrontFelt(v)
                act((b) => b.set_front_felt(v))
              }}
            />
            <Slider
              label="front rail pin, above the rail"
              value={frontPin}
              min={5}
              max={40}
              step={0.5}
              unit="mm"
              onChange={(v) => {
                setFrontPin(v)
                act((b) => b.set_front_pin_height(v))
              }}
            />
            <Slider
              label="front mortise, above the underside"
              value={frontMortise}
              min={5}
              max={40}
              step={0.5}
              unit="mm"
              onChange={(v) => {
                setFrontMortise(v)
                act((b) => b.set_front_mortise_depth(v))
              }}
            />
            <p className="note">
              The key stops on its front punching <i>or</i> on the pin reaching the top of its
              mortise, whichever comes first.{' '}
              {r.stopped_by_the_mortise
                ? 'Right now it is the pin: the dip below is shorter than the punchings alone would give.'
                : 'Right now it is the punching, as on a sane frame.'}
            </p>
            <Slider
              label="back rail cloth"
              value={backCloth}
              min={0}
              max={6}
              step={0.05}
              unit="mm"
              onChange={(v) => {
                setBackCloth(v)
                act((b) => b.set_back_rail_cloth(v))
              }}
            />

            <h2>The key</h2>
            <p className="note">
              The key weight ratio is <b>not</b> a control. It is the ratio of two arms, so it
              follows from the geometry exactly as the action ratio does; a slider for it would have
              to pick which dimension to move. Changing a dimension does not re-solve the punchings
              either: the regulation drifts and the sheet shows the miss, because that is what
              happens on a real keyboard.
            </p>
            <Slider
              label="capstan behind the balance pin"
              value={capstan}
              min={60}
              max={200}
              step={1}
              unit="mm"
              onChange={(v) => {
                setCapstan(v)
                act((b) => b.set_capstan_behind(v))
              }}
            />
            <Slider
              label="front edge from the balance pin"
              value={frontEdge}
              min={150}
              max={340}
              step={1}
              unit="mm"
              onChange={(v) => {
                setFrontEdge(v)
                act((b) => b.set_front_edge(v))
              }}
            />
            <Slider
              label="capstan height"
              value={capstanHeight}
              min={4}
              max={24}
              step={0.5}
              unit="mm"
              onChange={(v) => {
                setCapstanHeight(v)
                act((b) => b.set_capstan_height(v))
              }}
            />

            <h2>The frame</h2>
            <Slider
              label="balance rail"
              value={balanceRail}
              min={10}
              max={60}
              step={0.1}
              unit="mm"
              onChange={(v) => {
                setBalanceRail(v)
                act((b) => b.set_balance_rail_height(v))
              }}
            />
            <Slider
              label="front rail"
              value={frontRail}
              min={10}
              max={50}
              step={0.5}
              unit="mm"
              onChange={(v) => {
                setFrontRail(v)
                act((b) => b.set_front_rail_height(v))
              }}
            />
            <Slider
              label="back rail"
              value={backRail}
              min={10}
              max={60}
              step={0.1}
              unit="mm"
              onChange={(v) => {
                setBackRail(v)
                act((b) => b.set_back_rail_height(v))
              }}
            />
            <p className="note">
              And where the two outer rails <b>stand</b> along the key. These are the frame's own
              numbers, not the key's: a rail is screwed to the keybed, so lengthening a key moves
              its mortise and its chamfer and leaves the rails behind. That is a design worth seeing
              go wrong, which is why nothing here corrects itself — the sheet names the miss
              instead.
            </p>
            <Slider
              label="front rail, in front of the balance pin"
              value={frontRailAt}
              min={80}
              max={320}
              step={1}
              unit="mm"
              onChange={(v) => {
                setFrontRailAt(v)
                act((b) => b.set_front_rail_at(v))
              }}
            />
            <Slider
              label="back rail, behind the balance pin"
              value={backRailAt}
              min={120}
              max={280}
              step={1}
              unit="mm"
              onChange={(v) => {
                setBackRailAt(v)
                act((b) => b.set_back_rail_at(v))
              }}
            />
            <button
              type="button"
              className="replace"
              onClick={() => {
                act((b) => b.place_the_frame_under_the_key())
                const b = bench.current
                const fresh = b ? readout(b) : null
                if (fresh) {
                  setFrontRailAt(fresh.front_rail_at_mm)
                  setBackRailAt(fresh.back_rail_at_mm)
                }
              }}
            >
              place the frame under the key
            </button>
            <p className="note">
              Moves the rails and <b>nothing else</b>. The key bears on the cloth at a new arm
              afterwards, so the key height and the dip will have drifted off their targets and the
              sheet will show it — shimming is a separate operation on a real bench too.
            </p>

            <h2>Touch</h2>
            <p className="note">
              Put a weight on the measuring point. Three things can happen, not two: above the down
              weight the key descends, below the up weight it rises, and{' '}
              <b>between them it does nothing at all</b>. That gap is the friction.
            </p>
            <Slider
              label="weight on the measuring point"
              value={finger}
              min={-60}
              max={60}
              step={0.5}
              unit="g"
              onChange={(v) => {
                setFinger(v)
                act((b) => b.set_finger_weight(v))
              }}
            />
            <div className={`verdict ${r.held_by_friction ? 'verdict-held' : 'verdict-stop'}`}>
              the key is {r.finger}
              {r.friction_margin_g === null ? null : (
                <div className="gaps">
                  the weight lies between U and D, {fixed(r.friction_margin_g, 2)} g from moving the
                  key
                </div>
              )}
              <div className="gaps">
                gap at the front punching {gap(r.gap_front_mm)} · at the back rail cloth{' '}
                {gap(r.gap_back_mm)}
              </div>
              <div className="gaps">
                the bushings hold any torque within ±{fixed(r.key_friction_weight_g, 2)} g at the
                measuring point, the key level — an interval, not a value
              </div>
            </div>
            <p className="note">
              The front punching is read at its centre: a felt washer under a key landing level
              carries the load over its whole face, and the bench takes it at the middle. The
              contacts marked on the view are where the key bears, and what stands within a
              millimetre of it.
            </p>
            <p className="note">
              There is no “press the key” control. The key is only ever where the physics puts it —
              a slider over a computed travel could park it in a position no weight produces, which
              is the one thing a measuring instrument must not do.
            </p>

            <h2>Leading workshop</h2>
            <div className="leadadd">
              <input
                type="number"
                min={1}
                max={60}
                step={1}
                value={newLeadMass}
                onChange={(e) => setNewLeadMass(Number(e.target.value))}
              />
              <span>g at</span>
              <button type="button" onClick={() => act((b) => b.add_lead(150, newLeadMass))}>
                150 mm
              </button>
              <button type="button" onClick={() => act((b) => b.add_lead(190, newLeadMass))}>
                190 mm
              </button>
            </div>
            <ul className="leads">
              {state.scene.leads.map((l, i) => (
                <li key={i}>
                  <span>
                    {l.mass_g.toFixed(0)} g at {l.from_balance_mm.toFixed(0)} mm
                  </span>
                  <input
                    type="range"
                    min={20}
                    max={240}
                    step={1}
                    value={l.from_balance_mm}
                    onChange={(e) => act((b) => b.move_lead(i, Number(e.target.value)))}
                  />
                  <button type="button" onClick={() => act((b) => b.remove_lead(i))}>
                    remove
                  </button>
                </li>
              ))}
              {state.scene.leads.length === 0 && <li className="empty">no leads fitted</li>}
            </ul>
          </>
        )}
      </aside>
    </div>
  )
}
