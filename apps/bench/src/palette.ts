/**
 * The bench's colors: what each material is painted in, the page and the
 * edges, the contacts, the lines marked by provenance, and the lights the
 * solid is lit by. What a part is made of is the engine's; how it looks is
 * the view's, and lives here so that a test can hold the marks to the fills
 * they are drawn over (`palette.test.ts`).
 */

import type { ContactState, How, PaperColor } from '@pas/wasm/types'

/**
 * Each material's color: what a part is made of is the engine's, how it
 * looks is this view's.
 */
export const MATERIAL_FILL: Record<string, string> = {
  key_wood: '#ecdcb6',
  // The key button and the back check block: a wood other than the
  // keystick's, as the owner described them.
  block_wood: '#a8713f',
  // The keytop and the key front, white.
  covering: '#ffffff',
  key_front: '#ffffff',
  rail_wood: '#c6a477',
  steel: '#a9b1ba',
  punching_felt: '#c8655d',
  hard_felt: '#5d5a6b',
  cloth: '#5e9a78',
  // The cloth a balance mortise is bushed with, red as the owner drew it.
  bushing_cloth: '#b84864',
  action_wood: '#d9b98a',
  heel_cloth: '#4f6d8f',
}

/**
 * Paper punchings, painted in the color they are supplied in.
 *
 * In a technician's drawer the color *is* the thickness — you reach for a
 * two-tenths by its color, without measuring. Which thickness comes in which
 * color is the stock's, recorded in `data/params/punching-papers.json`, and
 * the engine sends each paper's color with it; a different supplier grades
 * otherwise, which is why the mapping is data and not a palette here. What is
 * the view's is only how each named color is painted. Above half a
 * millimeter the papers are plain card, told apart by feel and by what the
 * technician reached for.
 */
export const PAPER_FILL: Record<PaperColor, string> = {
  white: '#f1efe7',
  pastel_green: '#b4d8a8',
  sky_blue: '#86c5e8',
  camel: '#c19a6b',
  black: '#2e2e2e',
  card: '#9a7b52',
}

/** A paper whose color no record gives: no drawer holds it. */
export const UNGRADED_FILL = '#9b9b9b'

/** The canvas behind everything: a gray light enough for the wood, dark
 * enough that a white keytop still reads as a part. */
export const PAGE = '#e7e4dd'

/** The keybed, the datum every height is measured from, under both views. */
export const KEYBED = '#b9b2a2'

/** Edges in the section, opaque so that one hidden under the key's wood stays
 * hidden. */
export const EDGE = '#5b544a'

/**
 * The contacts' colors, by state: where the key bears, a gap of under a
 * millimeter still open, and drawn parts overlapping, which a rigid contact
 * should never show and a frame out of fit with its key does.
 */
export const CONTACT_COLOR: Record<ContactState, string> = {
  closed: '#1d63a8',
  open: '#8a8a8a',
  overlapping: '#e8590c',
}

/** The measuring point's marker, drawn over everything: a red no part is
 * painted in, so that the smoke test finds it among the parts. */
export const MEASURING_POINT_COLOR = '#c8302a'

/**
 * The lines' colors where they are marked by provenance: how the number
 * that places each is known, in the legend's order.
 *
 * A marked line is one pixel wide and drawn over every fill above, flat in
 * the section and lit on the solid's faces, so each color is chosen apart
 * from all of them as well as from the others and from an unmarked edge: a
 * hue no material is painted in, saturated where the fills are not, so a
 * line does not sink into the felt, the cloth or the wood it outlines. Only
 * what is set on the bench is black, the one color of ink. Reds, greens and
 * blues alone would collapse for a reader blind to red or green, so the
 * colors also step in lightness, from the dark green of a measurement to the
 * light orange of a trace; the pair such a reader tells apart least,
 * borrowed and assumed, are the two ways of not knowing a number on this
 * instrument. `palette.test.ts` holds the distances.
 */
export const HOW_COLOR: Record<How, string> = {
  measured: '#1a601e',
  published: '#4244b9',
  traced: '#ff7e2d',
  derived: '#a523f8',
  borrowed: '#2b97ca',
  assumed: '#fc19c4',
  set_on_the_bench: '#000000',
}

/**
 * The lights the solid is lit by: an ambient light and two directional ones,
 * each shining from its position toward the origin. The section is painted
 * unlit.
 */
export const LIGHTS = {
  ambient: 1.6,
  directional: [
    { position: [-200, 300, 400] as [number, number, number], intensity: 2.0 },
    { position: [300, 140, -250] as [number, number, number], intensity: 0.9 },
  ],
} as const

/**
 * The band's colors. The down curve and the up curve are the two series a
 * reader must tell apart, so they take two hues that read as different for
 * every reader, one cool and one warm, both dark enough to stand three to
 * one against the sheet they are drawn on; `palette.test.ts` holds the
 * distances. Everything else is ink: the band between them a wash of it,
 * the weight on the measuring point and where it has left the key the ink
 * itself, the grid and the stops a hairline of the rule. The sheet is the
 * page's paper, so the band reads as one more sheet beside the weighing
 * sheet.
 */
export const BAND_COLOR = {
  surface: '#fbfaf7',
  down: '#2a78d6',
  up: '#d0541c',
  held: 'rgba(35, 32, 27, 0.07)',
  ink: '#23201b',
  muted: '#6b6559',
  rule: '#d8d2c4',
} as const
