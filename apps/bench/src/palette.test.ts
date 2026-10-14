import type { How } from '@pas/wasm/types'
import { describe, expect, it } from 'vitest'
import {
  EDGE,
  HOW_COLOUR,
  KEYBED,
  LIGHTS,
  MATERIAL_FILL,
  PAGE,
  PAPER_FILL,
  UNGRADED_FILL,
} from './palette'

// ---- colour science ----------------------------------------------------------
//
// Distances are Euclidean in OKLab, times 100, the scale on which 15 is the
// floor two categories must clear for a reader of full colour vision and 6
// the floor for one blind to red or to green, simulated by Machado, Oliveira
// and Fernandes (2009) at full severity.

type Rgb = [number, number, number]
type Matrix = [Rgb, Rgb, Rgb]

const dot = (a: Rgb, b: Rgb) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
const times = (m: Matrix, x: Rgb): Rgb => [dot(m[0], x), dot(m[1], x), dot(m[2], x)]
const clamped = ([r, g, b]: Rgb): Rgb => [r, g, b].map((c) => Math.min(1, Math.max(0, c))) as Rgb

const linear = (hex: string): Rgb => {
  const c = (i: number) => {
    const s = Number.parseInt(hex.slice(1 + 2 * i, 3 + 2 * i), 16) / 255
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4
  }
  return [c(0), c(1), c(2)]
}

const encoded = (c: number) => {
  const x = Math.min(1, Math.max(0, c))
  return x <= 0.0031308 ? 12.92 * x : 1.055 * x ** (1 / 2.4) - 0.055
}

const hexOf = (rgb: Rgb) =>
  `#${rgb
    .map((c) =>
      Math.round(encoded(c) * 255)
        .toString(16)
        .padStart(2, '0'),
    )
    .join('')}`

const oklab = ([r, g, b]: Rgb): Rgb => {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ]
}

type Vision = 'normal' | 'protan' | 'deutan'

const MACHADO: Record<Exclude<Vision, 'normal'>, Matrix> = {
  protan: [
    [0.152286, 1.052583, -0.204868],
    [0.114503, 0.786281, 0.099216],
    [-0.003882, -0.048116, 1.051998],
  ],
  deutan: [
    [0.367322, 0.860646, -0.227968],
    [0.280085, 0.672501, 0.047413],
    [-0.01182, 0.04294, 0.968881],
  ],
}

const seen = (rgb: Rgb, vision: Vision): Rgb => {
  if (vision === 'normal') return rgb
  return clamped(times(MACHADO[vision], rgb))
}

/** How far apart two colours look, in OKLab times 100. */
const distance = (a: string, b: string, vision: Vision = 'normal') => {
  const [p, q] = [oklab(seen(linear(a), vision)), oklab(seen(linear(b), vision))]
  return 100 * Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2])
}

// ---- the solid's faces -------------------------------------------------------
//
// The solid paints each material with a Lambert material under `LIGHTS`, and
// react-three-fiber tone-maps it with three.js's ACES filmic curve at an
// exposure of one before encoding it in sRGB; the marks are not tone-mapped.
// This is that pipeline, which reproduces the faces a capture of the solid
// shows to the hex: the key's front face, `#ecdcb6` lit, is drawn `#dcd6c4`.

const aces = (c: Rgb): Rgb => {
  const v = c.map((x) => x / 0.6) as Rgb
  const input: Matrix = [
    [0.59719, 0.35458, 0.04823],
    [0.076, 0.90834, 0.01566],
    [0.0284, 0.13383, 0.83777],
  ]
  const output: Matrix = [
    [1.60475, -0.53108, -0.07367],
    [-0.10208, 1.10813, -0.00605],
    [-0.00327, -0.07276, 1.07602],
  ]
  const fit = (x: number) =>
    (x * (x + 0.0245786) - 0.000090537) / (x * (0.983729 * x + 0.432951) + 0.238081)
  return clamped(times(output, times(input, v).map(fit) as Rgb))
}

/** A material's colour on a face of the solid whose normal is `n`. */
const lit = (hex: string, n: Rgb) => {
  let irradiance = LIGHTS.ambient
  for (const { position, intensity } of LIGHTS.directional) {
    const length = Math.hypot(...position)
    const cos = dot(n, position) / length
    irradiance += intensity * Math.max(0, cos)
  }
  return hexOf(aces(linear(hex).map((c) => (c * irradiance) / Math.PI) as Rgb))
}

/** The faces of the solid a reader sees: towards the camera, on top, and
 * either end. */
const FACES = {
  front: [0, 0, 1],
  top: [0, 1, 0],
  left: [-1, 0, 0],
  right: [1, 0, 0],
} satisfies Record<string, Rgb>

// ---- what a marked line is drawn over ------------------------------------------

const materials: Record<string, string> = {
  ...MATERIAL_FILL,
  ...Object.fromEntries(Object.entries(PAPER_FILL).map(([k, v]) => [`${k} paper`, v])),
  'ungraded paper': UNGRADED_FILL,
  keybed: KEYBED,
}

/** Every colour a marked line can be drawn over: each material as the section
 * paints it and on each face of the solid, and the page behind both. */
const FILLS: Record<string, string> = {
  ...Object.fromEntries(Object.entries(materials).map(([k, v]) => [`${k} in the section`, v])),
  ...Object.fromEntries(
    Object.entries(materials).flatMap(([k, v]) =>
      Object.entries(FACES).map(([face, n]) => [`${k} on the solid's ${face} face`, lit(v, n)]),
    ),
  ),
  page: PAGE,
}

const hows = Object.keys(HOW_COLOUR) as How[]

describe('the provenance marks', () => {
  it('see the solid’s faces as a capture of it shows them', () => {
    // The model above against two faces read off a capture of the solid.
    expect(lit(MATERIAL_FILL.key_wood ?? '', FACES.front)).toBe('#dcd6c4')
    expect(lit(MATERIAL_FILL.cloth ?? '', FACES.top)).toBe('#68a683')
  })

  it('tell every way of knowing a number from every other, colour-blind too', () => {
    for (const [i, a] of hows.entries()) {
      for (const b of hows.slice(i + 1)) {
        const [x, y] = [HOW_COLOUR[a], HOW_COLOUR[b]]
        expect(distance(x, y), `${a} and ${b}`).toBeGreaterThanOrEqual(15)
        for (const vision of ['protan', 'deutan'] as const) {
          expect(distance(x, y, vision), `${a} and ${b}, ${vision}`).toBeGreaterThanOrEqual(6)
        }
      }
    }
  })

  it('stand apart from an unmarked edge', () => {
    for (const how of hows) {
      expect(distance(HOW_COLOUR[how], EDGE), how).toBeGreaterThanOrEqual(10)
    }
  })

  it('stand apart from every fill they are drawn over, in both views', () => {
    for (const how of hows) {
      for (const [fill, colour] of Object.entries(FILLS)) {
        expect(distance(HOW_COLOUR[how], colour), `${how} on ${fill}`).toBeGreaterThanOrEqual(10)
        for (const vision of ['protan', 'deutan'] as const) {
          expect(
            distance(HOW_COLOUR[how], colour, vision),
            `${how} on ${fill}, ${vision}`,
          ).toBeGreaterThanOrEqual(5)
        }
      }
    }
  })
})
