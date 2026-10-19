/**
 * Single-key bench.
 *
 * Two halves: the sagittal section, which a technician validates by looking,
 * and the weighing sheet, which he validates by reading. Both come from the
 * same engine, so they cannot disagree. In place of the section the band can
 * be drawn, the force on the measuring point through the key's stroke, which
 * the engine sweeps when it is chosen and not with every edit.
 *
 * The panels and their controls are the engine's description of them
 * (`describe()`, drawn by `Control.tsx`): which panels, in which order, each
 * control with its path or its action, its range and the code it is worded
 * by. This file adds only what each panel says around its controls: the
 * sheet's readings, the notes, the verdicts.
 *
 * Every number on the page is the engine's: each slider shows the value the
 * settings hold, the key friction weight's as the sheet reads it, each
 * reading the sheet's, and nothing here keeps a copy of either to drift from
 * them; the typed fields hold only what is typed until it is sent. Every
 * sentence is the catalog's (`i18n/`), composed from the codes and numbers
 * the engine sends, in the language the page opened in, the reader's
 * browser's where the bench speaks it, or the one chosen since: choosing
 * another rewords the page and asks the engine nothing.
 */

import type { Bench } from '@pas/wasm'
import type {
  Acted,
  Band,
  Panel as PanelCode,
  Procedure,
  Registry,
  Scene,
  Sheet,
  State,
} from '@pas/wasm/types'
import { useCallback, useEffect, useRef, useState } from 'react'
import { BandView } from './BandView'
import {
  act,
  band,
  describe,
  diagnostics,
  reading,
  scene,
  set,
  settings,
  sheet,
  start,
  state,
} from './bench'
import { type Answer, type Around, type Edits, Panel, type Read, Refused, shown } from './Control'
import { ANGLE_DEG, decimals, LENGTH_MM, MASS_G } from './format'
import {
  CATALOG,
  CatalogContext,
  type Language,
  pickLanguage,
  type Reported,
  useCatalog,
  word,
} from './i18n'
import { LanguageChoice } from './LanguageChoice'
import { madeBy } from './layout'
import { CONTACT_COLOR, HOW_COLOR } from './palette'
import { Rich } from './Rich'
import { SagittalView, type ViewMode } from './SagittalView'

/** Everything the engine said when it was last asked: the registry, read
 * once at boot since it does not change, and every document after each
 * edit. */
interface Engine extends Read {
  readonly state: State
  readonly scene: Scene
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
  if (Number.isFinite(weight) && q.has('weight')) b.set('experiment.weight_g', weight)
}

/**
 * A quantity the bench reports beside its sheet, to the resolution its kind
 * is shown to, with a target beside it where it has one; or refused, why on
 * hover.
 */
function Row({
  quantity,
  control,
  value,
  resolution,
  target,
  why,
}: {
  /** What it is, where a test reads it. */
  quantity: string
  control: Reported
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
  const t = useCatalog()
  if (value === null) {
    return (
      <div className="row refused" data-quantity={quantity} title={why ?? ''}>
        <span className="label">{control.label}</span>
        <span className="value">{t.sheet.refused}</span>
        <span className="unit" />
        <span className="target" />
      </div>
    )
  }
  return (
    <div className="row" data-quantity={quantity} data-value={value}>
      <span className="label">{control.label}</span>
      <span className="value">{shown(t, value, resolution)}</span>
      <span className="unit">{control.unit}</span>
      <span className="target">
        {target === undefined ? '' : t.sheet.target(shown(t, target, resolution))}
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
  const t = useCatalog()
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
    target === undefined ? t.number(m.value, to + 2) : t.sheet.target(shown(t, target, LENGTH_MM))
  return (
    <div
      className={`row ${m.trust.code}`}
      data-procedure={procedure}
      data-shown={m.shown}
      data-exact={m.value}
      title={
        m.trust.code === 'indicative' ? t.reasons(m.trust.because.map((b) => t.because[b])) : ''
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
 * What the sheet's readings rest on that nobody measured, and why any is
 * refused: one note each, under the weights.
 */
function SheetNotes({ s }: { s: Sheet }) {
  const t = useCatalog()
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
  const linked = new URLSearchParams(window.location.search).get('view')
  const [view, setView] = useState<ViewMode>(
    // `?view=section` opens on the flat technical section rather than the
    // solid, so a particular reading can be linked to.
    linked === 'section' ? 'section' : 'solid',
  )
  // The band, drawn on demand: when it is chosen, never with every edit.
  // Kept with whether the key has changed since, which leaves it drawn but
  // marked until it is chosen again; only the weight on the measuring point,
  // which the band does not read, leaves it as it is. `?view=band` opens on
  // it.
  const [shownBand, setShownBand] = useState(linked === 'band')
  const [drawnBand, setDrawnBand] = useState<{ band: Band; stale: boolean } | null>(null)
  // Whether every drawn line is marked by how the number that places it is
  // known: a way of looking, the view's alone.
  const [marked, setMarked] = useState(false)
  // How opaque the key is drawn, so that what stands inside it shows, as a
  // pin in its mortise: a way of looking too.
  const [keyOpacity, setKeyOpacity] = useState(1)
  // What the engine answered the last action, shown until anything else is
  // done: the papers a regulation laid, which the engine keeps nowhere else.
  // An edit refused it keeps itself, in its diagnostics.
  const [answer, setAnswer] = useState<Answer | null>(null)
  const [language, setLanguage] = useState<Language>(() => pickLanguage(navigator.languages))
  const t = CATALOG[language]
  useEffect(() => {
    document.documentElement.lang = language
    document.title = t.tabTitle
  }, [language, t])

  // Every mutation goes through the engine and the whole page is re-read
  // from it. No value on this page is computed in TypeScript, and none is
  // kept: a frame the key cannot be seated on still has every setting, and
  // its sheet says reading by reading what cannot be read.
  const read = useCallback((b: Bench, registry: Registry): Engine => {
    drawn.current = scene(b, drawn.current)
    return {
      registry,
      settings: settings(b),
      sheet: sheet(b),
      diagnostics: diagnostics(b),
      state: state(b),
      scene: drawn.current.scene,
    }
  }, [])

  useEffect(() => {
    void start((b) => {
      // Runs exactly once, inside the boot promise: see `start`.
      b.set('experiment.weight_g', 0)
      applyUrlState(b)
    }).then((b) => {
      bench.current = b
      setEngine(read(b, describe(b)))
      if (new URLSearchParams(window.location.search).get('view') === 'band') {
        setDrawnBand({ band: band(b), stale: false })
      }
    })
  }, [read])

  /** Draws the band for the key as it is, and shows it. */
  const showTheBand = () => {
    const b = bench.current
    if (!b) return
    setDrawnBand({ band: band(b), stale: false })
    setShownBand(true)
  }

  /** Marks the band drawn as the key's before an edit the engine took, unless
   * the edit set the weight on the measuring point, which the band does not
   * read: the experiment's layer, by the registry. */
  const changesTheBand = (acted: Acted, path: string | null) => {
    if (acted.outcome === 'refused') return
    const layer = path === null ? null : engine?.registry.params.find((p) => p.path === path)?.layer
    if (layer !== 'experiment') {
      setDrawnBand((drawn) => (drawn === null ? drawn : { ...drawn, stale: true }))
    }
  }

  /** Runs `f` on the engine, keeps its answer, and re-reads the page. The
   * engine is read here and not inside a state updater, which React's
   * StrictMode runs twice. The registry is read again only where another
   * instrument came on the bench, whose drawer may hold other papers. */
  const change = (f: (b: Bench) => Answer | null) => {
    const b = bench.current
    if (!b || !engine) return
    const answered = f(b)
    setAnswer(answered)
    const another = answered?.action === 'select_profile'
    setEngine(read(b, another ? describe(b) : engine.registry))
  }
  const edits: Edits = {
    set: (path, value) =>
      change((b) => {
        changesTheBand(set(b, path, value), path)
        return null
      }),
    act: (action, args) =>
      change((b) => {
        const acted = act(b, action, args)
        changesTheBand(acted, null)
        return { action, acted }
      }),
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
  // An edit the engine refused is said beside the control that made it; one
  // no control on the page made, as a link's, is said here.
  const refused = d.refused_edit
  const refusedOffThePage =
    refused !== null &&
    !engine.registry.page.some((p) => p.controls.some((c) => madeBy(c, refused)))

  // What each panel says around its controls, by the panel's code and,
  // after a control, by its label's: the engine places the controls, and a
  // note follows the control it is about wherever the engine puts it.
  const around: { readonly [P in PanelCode]: Around } = {
    instrument: {},
    sheet: {
      after: {
        measuring_point: (
          <>
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
            <Measured s={sh} procedure="key_height" target={s.regulation.key_height_mm} />
            <Measured s={sh} procedure="key_dip" target={s.regulation.key_dip_mm} />
            <p className="note">{t.sheet.seatingNote}</p>
            <Row
              quantity="key_mass"
              control={t.sheet.keyMass}
              value={s.key.mass_g}
              resolution={MASS_G}
              why={t.refusal.key_not_weighed({ code: 'key_not_weighed' })}
            />
            <Row
              quantity="out_of_level_at_bottom_deg"
              control={t.sheet.outOfLevel}
              value={seats?.out_of_level_at_bottom_deg ?? null}
              resolution={ANGLE_DEG}
              target={0}
              why={seatWhy}
            />
            <Row
              quantity="bedding_at_the_back_deg"
              control={t.sheet.bedding}
              value={seats?.bedding_at_the_back_deg ?? null}
              resolution={ANGLE_DEG}
              target={0}
              why={seatWhy}
            />
            <Row
              quantity="bears_at_the_back_mm"
              control={t.sheet.bearsAt}
              value={seats?.bears_at_the_back_mm ?? null}
              resolution={LENGTH_MM}
              why={seatWhy}
            />
            <p className="note" data-back-bearing={seats?.back_bearing}>
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
          </>
        ),
      },
    },
    regulation: {
      after: {
        regulate: (
          <>
            <p className="note">{t.regulation.layingNote}</p>
            <p className="note">
              <Rich text={t.regulation.pileNote} />
            </p>
          </>
        ),
        key_friction_weight: <p className="note">{t.friction.note}</p>,
      },
    },
    punchings: {
      before: <p className="note">{t.punchings.note}</p>,
      after: {
        front_mortise: (
          <p className="note">
            <Rich
              text={t.punchings.stopsAtTheBottom(
                seats === null ? null : seats.bottom_held_by.includes('front_pin_in_the_mortise'),
              )}
            />
          </p>
        ),
      },
    },
    key: {
      before: (
        <p className="note">
          <Rich text={t.key.ratioNote} />
        </p>
      ),
    },
    weighing: {
      before: <p className="note">{t.weighing.intro}</p>,
      after: {
        weigh: (
          <>
            <p
              className="note weighed"
              data-mass-g={s.key.weighing?.mass_g}
              data-front-weight-g={s.key.weighing?.front_weight_g}
              data-from-lip-mm={s.key.weighing?.from_lip_mm}
            >
              {s.key.weighing === null
                ? s.key.mass_known === 'published'
                  ? t.weighing.published(s.key.mass_g)
                  : t.weighing.notWeighed
                : t.weighing.weighed(
                    s.key.weighing.mass_g,
                    s.key.weighing.front_weight_g,
                    s.key.weighing.from_lip_mm,
                  )}
            </p>
            <p className="note">
              <Rich text={t.weighing.note} />
            </p>
          </>
        ),
      },
    },
    frame: {
      after: {
        back_rail: (
          <p className="note">
            <Rich text={t.frame.railsNote} />
          </p>
        ),
        place: (
          <p className="note">
            <Rich text={t.frame.placeNote} />
          </p>
        ),
      },
    },
    touch: {
      before: (
        <p className="note">
          <Rich text={t.touch.intro} />
        </p>
      ),
      after: {
        weight: (
          <>
            {d.touch && (
              <div
                className={`verdict ${d.touch.verdict.code === 'held_by_friction' ? 'verdict-held' : 'verdict-stop'}`}
                data-verdict={d.touch.verdict.code}
              >
                {t.touch.theKeyIs(word(t.touch.verdict, d.touch.verdict.code, d.touch.verdict))}
                {d.touch.verdict.code === 'held_by_friction' && (
                  <div className="gaps">{t.touch.margin(d.touch.verdict.margin_g)}</div>
                )}
                {(d.touch.gap_front_mm !== null || d.touch.gap_back_mm !== null) && (
                  <div className="gaps">
                    {t.touch.gaps(d.touch.gap_front_mm, d.touch.gap_back_mm)}
                  </div>
                )}
                <div className="gaps">
                  {t.touch.bushings(
                    kf?.outcome === 'read' ? t.number(kf.shown, decimals(kf.resolution)) : null,
                  )}
                </div>
              </div>
            )}
            {s.key.mass_known === 'unweighed' && (
              <p className="note not-weighed">{t.touch.notWeighed}</p>
            )}
            <p className="note">{t.touch.centerNote(d.edge_overlaps)}</p>
            <p className="note">{t.touch.noPressNote}</p>
          </>
        ),
      },
    },
  }

  return (
    <CatalogContext.Provider value={t}>
      <div className="bench">
        <div
          className="view"
          data-touching={touching.join(' ')}
          data-marked={marked ? 'provenance' : undefined}
          data-place={engine.state.place}
        >
          <SagittalView
            scene={engine.scene}
            state={engine.state}
            mode={view}
            marked={marked}
            keyOpacity={keyOpacity}
          />
          {shownBand && drawnBand && (
            <BandView
              band={drawnBand.band}
              stale={drawnBand.stale}
              weightG={s.experiment.weight_g}
              travelMm={d.touch?.travel_mm ?? null}
            />
          )}
          <div className="viewtabs">
            <button
              type="button"
              className={!shownBand && view === 'section' ? 'on' : ''}
              data-view="section"
              onClick={() => {
                setView('section')
                setShownBand(false)
              }}
            >
              {t.view.section}
            </button>
            <button
              type="button"
              className={!shownBand && view === 'solid' ? 'on' : ''}
              data-view="solid"
              onClick={() => {
                setView('solid')
                setShownBand(false)
              }}
            >
              {t.view.solid}
            </button>
            <button
              type="button"
              className={shownBand ? 'on' : ''}
              data-view="band"
              onClick={showTheBand}
            >
              {t.view.band}
            </button>
            {!shownBand && (
              <>
                <button
                  type="button"
                  className={marked ? 'on' : ''}
                  data-mark="provenance"
                  aria-pressed={marked}
                  onClick={() => setMarked(!marked)}
                >
                  {t.view.provenance}
                </button>
                <label className="key-opacity">
                  {t.view.keyOpacity}
                  <input
                    type="range"
                    min={0.1}
                    max={1}
                    step={0.05}
                    value={keyOpacity}
                    data-view-setting="key-opacity"
                    onChange={(e) => setKeyOpacity(Number(e.target.value))}
                  />
                </label>
                <span className="hint">{t.view.hint}</span>
                <span className="legend">
                  <i style={{ background: CONTACT_COLOR.closed }} /> {t.view.bears}
                  <i style={{ background: CONTACT_COLOR.open }} /> {t.view.open}
                  <i style={{ background: CONTACT_COLOR.overlapping }} /> {t.view.overlapping}
                </span>
                {engine.state.place !== 'placed' && (
                  <p className="view-note">{t.view.notPlaced[engine.state.place]}</p>
                )}
              </>
            )}
          </div>
          {marked && !shownBand && (
            <div className="provenance-legend">
              {(Object.keys(HOW_COLOR) as Array<keyof typeof HOW_COLOR>).map((how) => (
                <span key={how} data-how={how}>
                  <span className="swatch" style={{ background: HOW_COLOR[how] }} />{' '}
                  {t.view.how[how]}
                </span>
              ))}
            </div>
          )}
        </div>

        <aside className="panel">
          <LanguageChoice language={language} choose={setLanguage} />
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
          {refusedOffThePage && <Refused refused={refused} />}
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

          {engine.registry.page.map((p) => (
            <Panel
              // Another instrument on the bench opens its forms on its own
              // numbers.
              key={`${s.profile}-${p.panel}`}
              layout={p}
              around={around[p.panel]}
              read={engine}
              edits={edits}
              answer={answer}
            />
          ))}
        </aside>
      </div>
    </CatalogContext.Provider>
  )
}
