/**
 * The band, drawn: the force on the measuring point against the key's travel
 * there, the way down and the way back, as the engine's driven sweep reads it
 * (`pa_core::measure::sweep`).
 *
 * A technician reads D and U on it as he reads them on his scale: each is
 * marked where it falls, at the band's extremes over the stroke, with what
 * the trade reads it as, the gram, and its value in full beside it. Between
 * the two curves the key stays wherever it is, held by its friction; their
 * distance apart is the friction, their middle the key's balance. The weight
 * the page puts on the measuring point is drawn across the band, and where it
 * has left the key is a dot on that line: above the down curve the key goes
 * down, below the up curve it rises, between them it stays.
 *
 * Every number drawn is the engine's: the curves, the two readings, the
 * window, the weight and where the key is. This file only places them, and
 * picks round values for the grid. The pointer finds the reading nearest it
 * and shows both curves' forces there; a table carries every step for a
 * reader without the chart.
 */
import type { Band, BandPoint, Reading } from '@pas/wasm/types'
import { useLayoutEffect, useState } from 'react'
import { nearest, niceStep, padded, ticks } from './axis'
import { decimals, MASS_G } from './format'
import { t, word } from './i18n'
import { BAND_COLOUR } from './palette'
import { Rich } from './Rich'

/** A reading read, as the band carries it. */
type Read = Extract<Reading, { outcome: 'read' }>

/** The decimals a force at a step is shown to: the hundredth of a gram, the
 * value in full shown small beside a reading. */
const FORCE_DECIMALS = 2
/** The decimals a travel is shown to: the hundredth of a millimetre. */
const TRAVEL_DECIMALS = 2

/** The room round the plot, pixels: for the force's ticks and title on the
 * left, the travel's ticks, the seats' names and the axis title below. */
const MARGIN = { top: 16, right: 24, bottom: 62, left: 64 } as const

/** How wide a character of the page's monospaced face stands at the labels'
 * 12 px, pixels: six tenths of the size, a little more for safety. */
const CHARACTER_PX = 7.4

/**
 * A label on the plot, on a patch of the sheet's colour, so that a grid line
 * or the weight's line passing under it does not run through its words: the
 * page's face is monospaced, so its width is its characters'.
 */
function Label({
  x,
  y,
  anchor,
  className,
  text,
}: {
  x: number
  y: number
  anchor: 'start' | 'middle' | 'end'
  className: string
  text: string
}) {
  const width = text.length * CHARACTER_PX + 6
  const left = anchor === 'start' ? x - 3 : anchor === 'end' ? x - width + 3 : x - width / 2
  return (
    <g>
      <rect x={left} y={y - 12} width={width} height={16} fill={BAND_COLOUR.surface} />
      <text x={x} y={y} textAnchor={anchor} className={className}>
        {text}
      </text>
    </g>
  )
}

/**
 * The size of an element, as it is laid out, and the ref that hands the
 * element over: a callback, so that an element that appears later, the plot
 * of a band drawn after one refused, is measured when it does.
 */
function useSize<T extends Element>() {
  const [element, setElement] = useState<T | null>(null)
  const [size, setSize] = useState({ width: 0, height: 0 })
  useLayoutEffect(() => {
    if (!element) return
    const measure = () => {
      const r = element.getBoundingClientRect()
      setSize({ width: r.width, height: r.height })
    }
    measure()
    const observer = new ResizeObserver(measure)
    observer.observe(element)
    return () => observer.disconnect()
  }, [element])
  return [setElement, size] as const
}

/** A curve as an SVG path through its readings. */
function path(points: readonly BandPoint[], x: (mm: number) => number, y: (g: number) => number) {
  return points
    .map((p, i) => `${i === 0 ? 'M' : 'L'}${x(p.travel_mm).toFixed(1)},${y(p.force_g).toFixed(1)}`)
    .join('')
}

/** The reading of `band` by `procedure`, where it is read. */
function read(band: Band, procedure: Read['procedure']): Read | undefined {
  const r = band.readings.find((m) => m.procedure === procedure)
  return r?.outcome === 'read' ? r : undefined
}

export function BandView({
  band,
  stale,
  weightG,
  travelMm,
}: {
  /** The band as the engine drew it. */
  band: Band
  /** Whether the key has changed since it was drawn. */
  stale: boolean
  /** The weight on the measuring point, grams. */
  weightG: number
  /** Where that weight has left the key, millimetres of travel below its
   * top seat; `null` where the engine reads none. */
  travelMm: number | null
}) {
  const [plotRef, size] = useSize<HTMLDivElement>()
  const [at, setAt] = useState<number | null>(null)
  const curves = band.curves

  // A refusal is as much the key's as a band is: once the key has changed,
  // the frame put right perhaps, it says so as a drawn band does, or the
  // page would say the key has no stroke beside a sheet that reads one.
  if (curves === null) {
    const why = band.readings.find((r) => r.outcome === 'refused')
    return (
      <div
        className={`band${stale ? ' stale' : ''}`}
        data-band="refused"
        data-stale={stale}
        data-refusal={why?.outcome === 'refused' ? why.why.code : undefined}
      >
        <h2 className="band-title">{t.band.title}</h2>
        {stale && (
          <p className="note band-stale">
            <Rich text={t.band.staleRefusal} />
          </p>
        )}
        <p className="degenerate band-refusal">
          {why?.outcome === 'refused' ? t.band.refused(word(t.refusal, why.why.code, why.why)) : ''}
        </p>
      </div>
    )
  }

  const down = read(band, 'down_weight_on_the_band')
  const up = read(band, 'up_weight_on_the_band')
  const travels = curves.down.map((p) => p.travel_mm)
  const forces = [...curves.down.map((p) => p.force_g), ...curves.up.map((p) => p.force_g), weightG]
  const [lo, hi] = padded(forces, 0.08)
  const { width, height } = size
  const plotW = Math.max(0, width - MARGIN.left - MARGIN.right)
  const plotH = Math.max(0, height - MARGIN.top - MARGIN.bottom)
  const x = (mm: number) => MARGIN.left + (mm / curves.window_mm) * plotW
  const y = (g: number) => MARGIN.top + ((hi - g) / (hi - lo)) * plotH
  const forceTicks = ticks(lo, hi, niceStep(hi - lo, Math.max(2, Math.floor(plotH / 48))))
  const travelStep = niceStep(curves.window_mm, 10)
  const travelTicks = ticks(0, curves.window_mm, travelStep)
  const forceDecimals = decimals(niceStep(hi - lo, Math.max(2, Math.floor(plotH / 48))))
  const band_ = `${path(curves.down, x, y)}${path(curves.up, x, y).replace('M', 'L')}Z`
  const hovered = at === null ? -1 : at
  const there = curves.down[hovered]
  const back = curves.up[curves.up.length - 1 - hovered]
  const n = (v: number, d: number) => t.number(v, d)

  /** A reading marked where it falls on its curve, its label beside it,
   * towards the middle of the window and away from the band; into the band
   * instead where the weight's line runs where it would stand. */
  const marker = (
    m: Read | undefined,
    label: (shown: string, exact: string) => string,
    above: boolean,
  ) => {
    if (!m) return null
    const atMm = m.conditions.sweep?.at_mm ?? 0
    const [mx, my] = [x(atMm), y(m.value)]
    const left = atMm < curves.window_mm / 2
    const text = label(n(m.shown, decimals(m.resolution)), n(m.value, FORCE_DECIMALS))
    // The label's baseline away from the band, or into it where the weight's
    // line would cross it: a line of text stands about 12 px tall.
    const outside = my + (above ? -10 : 20)
    const inside = my + (above ? 20 : -10)
    const crossed = Math.abs(outside - 4 - y(weightG)) < 12
    const ty = crossed ? inside : outside
    return (
      <g
        className="band-reading"
        data-procedure={m.procedure}
        data-shown={m.shown}
        data-exact={m.value}
        data-at-mm={atMm}
      >
        <circle
          cx={mx}
          cy={my}
          r={4.5}
          fill={m.procedure === 'down_weight_on_the_band' ? BAND_COLOUR.down : BAND_COLOUR.up}
          stroke={BAND_COLOUR.surface}
          strokeWidth={2}
        />
        <Label
          x={mx + (left ? 10 : -10)}
          y={ty}
          anchor={left ? 'start' : 'end'}
          className="band-label"
          text={text}
        />
      </g>
    )
  }

  return (
    <div
      className={`band${stale ? ' stale' : ''}`}
      data-band="drawn"
      data-stale={stale}
      data-samples={curves.down.length}
      data-window-mm={curves.window_mm}
      data-step-mm={curves.step_mm}
    >
      <h2 className="band-title">{t.band.title}</h2>
      <p className="note band-caption">
        <Rich text={t.band.caption(n(curves.step_mm, TRAVEL_DECIMALS))} />
      </p>
      <ul className="band-legend">
        <li>
          <i className="band-key key-line" style={{ background: BAND_COLOUR.down }} /> {t.band.down}
        </li>
        <li>
          <i className="band-key key-line" style={{ background: BAND_COLOUR.up }} /> {t.band.up}
        </li>
        <li>
          <i className="band-key key-rect" style={{ background: BAND_COLOUR.held }} /> {t.band.held}
        </li>
        <li>
          <i className="band-key key-hair" style={{ background: BAND_COLOUR.ink }} />{' '}
          {t.band.weight}
        </li>
        <li>
          <i className="band-key key-dot" style={{ background: BAND_COLOUR.ink }} /> {t.band.key}
        </li>
      </ul>
      {stale && (
        <p className="note band-stale">
          <Rich text={t.band.stale} />
        </p>
      )}
      <div className="band-plot" ref={plotRef}>
        {width > 0 && height > 0 && (
          <svg
            width={width}
            height={height}
            role="img"
            aria-label={t.band.title}
            onPointerMove={(e) => {
              const box = e.currentTarget.getBoundingClientRect()
              const mm = ((e.clientX - box.left - MARGIN.left) / plotW) * curves.window_mm
              setAt(nearest(travels, mm))
            }}
            onPointerLeave={() => setAt(null)}
          >
            <rect x={0} y={0} width={width} height={height} fill={BAND_COLOUR.surface} />
            {forceTicks.map((g) => (
              <g key={`f${g}`}>
                <line
                  x1={MARGIN.left}
                  x2={MARGIN.left + plotW}
                  y1={y(g)}
                  y2={y(g)}
                  stroke={g === 0 ? BAND_COLOUR.muted : BAND_COLOUR.rule}
                  strokeWidth={1}
                />
                <text x={MARGIN.left - 8} y={y(g) + 4} textAnchor="end" className="band-tick">
                  {n(g, forceDecimals)}
                </text>
              </g>
            ))}
            {travelTicks.map((mm) => (
              <text
                key={`t${mm}`}
                x={x(mm)}
                y={MARGIN.top + plotH + 16}
                textAnchor="middle"
                className="band-tick"
              >
                {n(mm, decimals(travelStep))}
              </text>
            ))}
            {/* The two seats, where a stop takes whatever the finger leaves. */}
            {[
              { mm: 0, name: t.band.topSeat, anchor: 'start' as const },
              { mm: curves.window_mm, name: t.band.bottomSeat, anchor: 'end' as const },
            ].map(({ mm, name, anchor }) => (
              <g key={name}>
                <line
                  x1={x(mm)}
                  x2={x(mm)}
                  y1={MARGIN.top}
                  y2={MARGIN.top + plotH}
                  stroke={BAND_COLOUR.muted}
                  strokeWidth={1}
                />
                <text
                  x={x(mm)}
                  y={MARGIN.top + plotH + 32}
                  textAnchor={anchor}
                  className="band-seat"
                >
                  {name}
                </text>
              </g>
            ))}
            <text
              x={MARGIN.left + plotW / 2}
              y={MARGIN.top + plotH + 52}
              textAnchor="middle"
              className="band-axis"
            >
              {t.band.travel}
            </text>
            <text
              transform={`translate(${16},${MARGIN.top + plotH / 2}) rotate(-90)`}
              textAnchor="middle"
              className="band-axis"
            >
              {t.band.force}
            </text>

            <path d={band_} fill={BAND_COLOUR.held} stroke="none" />
            <path
              d={path(curves.down, x, y)}
              fill="none"
              stroke={BAND_COLOUR.down}
              strokeWidth={2}
              strokeLinejoin="round"
              strokeLinecap="round"
              data-curve="down"
            />
            <path
              d={path(curves.up, x, y)}
              fill="none"
              stroke={BAND_COLOUR.up}
              strokeWidth={2}
              strokeLinejoin="round"
              strokeLinecap="round"
              data-curve="up"
            />

            {/* The weight on the measuring point, and where it has left the
                key. */}
            <g
              className="band-touch"
              data-weight-g={weightG}
              data-travel-mm={travelMm ?? undefined}
            >
              <line
                x1={MARGIN.left}
                x2={MARGIN.left + plotW}
                y1={y(weightG)}
                y2={y(weightG)}
                stroke={BAND_COLOUR.ink}
                strokeWidth={1}
              />
              <Label
                x={MARGIN.left + plotW / 2}
                y={y(weightG) - 6}
                anchor="middle"
                className="band-label"
                text={t.band.weightLine(n(weightG, decimals(MASS_G)))}
              />
              {travelMm !== null && (
                <circle
                  cx={x(Math.min(Math.max(travelMm, 0), curves.window_mm))}
                  cy={y(weightG)}
                  r={4.5}
                  fill={BAND_COLOUR.ink}
                  stroke={BAND_COLOUR.surface}
                  strokeWidth={2}
                />
              )}
            </g>

            {marker(down, t.band.downMarker, true)}
            {marker(up, t.band.upMarker, false)}

            {there && back && (
              <g className="band-crosshair" pointerEvents="none">
                <line
                  x1={x(there.travel_mm)}
                  x2={x(there.travel_mm)}
                  y1={MARGIN.top}
                  y2={MARGIN.top + plotH}
                  stroke={BAND_COLOUR.muted}
                  strokeWidth={1}
                />
                <circle
                  cx={x(there.travel_mm)}
                  cy={y(there.force_g)}
                  r={3.5}
                  fill={BAND_COLOUR.down}
                />
                <circle cx={x(back.travel_mm)} cy={y(back.force_g)} r={3.5} fill={BAND_COLOUR.up} />
                <Label
                  x={x(there.travel_mm) + (there.travel_mm < curves.window_mm / 2 ? 8 : -8)}
                  y={MARGIN.top + 14}
                  anchor={there.travel_mm < curves.window_mm / 2 ? 'start' : 'end'}
                  className="band-readout"
                  text={t.band.readout(
                    n(there.travel_mm, TRAVEL_DECIMALS),
                    n(there.force_g, FORCE_DECIMALS),
                    n(back.force_g, FORCE_DECIMALS),
                  )}
                />
              </g>
            )}
          </svg>
        )}
      </div>
      <p className="note band-note">{t.band.seatsNote}</p>
      {/* Hidden by a block around it: a table's own box keeps its rows'
          height whatever it is given, and would lengthen the page by every
          step of the sweep. */}
      <div className="visually-hidden">
        <table>
          <caption>{t.band.table}</caption>
          <thead>
            <tr>
              <th scope="col">{t.band.travel}</th>
              <th scope="col">{t.band.down}</th>
              <th scope="col">{t.band.up}</th>
            </tr>
          </thead>
          <tbody>
            {curves.down.map((p, i) => (
              <tr key={p.travel_mm}>
                <td>{n(p.travel_mm, TRAVEL_DECIMALS)}</td>
                <td>{n(p.force_g, FORCE_DECIMALS)}</td>
                <td>
                  {n(curves.up[curves.up.length - 1 - i]?.force_g ?? Number.NaN, FORCE_DECIMALS)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
