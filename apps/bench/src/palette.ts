/**
 * The bench's colours: what each material is painted in, the page and the
 * edges, the contacts, the lines marked by provenance, and the lights the
 * solid is lit by. What a part is made of is the engine's; how it looks is
 * the view's, and lives here so that a test can hold the marks to the fills
 * they are drawn over (`palette.test.ts`).
 */

import type { ContactState, How, PaperColour } from '@pas/wasm/types'

/**
 * Each material's colour: what a part is made of is the engine's, how it
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
}

/**
 * Paper punchings, painted in the colour they are supplied in.
 *
 * In a technician's drawer the colour *is* the thickness — you reach for a
 * two-tenths by its colour, without measuring. Which thickness comes in which
 * colour is the stock's, recorded in `data/params/punching-papers.json`, and
 * the engine sends each paper's colour with it; a different supplier grades
 * otherwise, which is why the mapping is data and not a palette here. What is
 * the view's is only how each named colour is painted. Above half a
 * millimetre the papers are plain card, told apart by feel and by what the
 * technician reached for.
 */
export const PAPER_FILL: Record<PaperColour, string> = {
  white: '#f1efe7',
  pastel_green: '#b4d8a8',
  sky_blue: '#86c5e8',
  camel: '#c19a6b',
  black: '#2e2e2e',
  card: '#9a7b52',
}

/** A paper whose colour no record gives: no drawer holds it. */
export const UNGRADED_FILL = '#9b9b9b'

/** The canvas behind everything: a grey light enough for the wood, dark
 * enough that a white keytop still reads as a part. */
export const PAGE = '#e7e4dd'

/** The keybed, the datum every height is measured from, under both views. */
export const KEYBED = '#b9b2a2'

/** Edges in the section, opaque so that one hidden under the key's wood stays
 * hidden. */
export const EDGE = '#5b544a'

/**
 * The contacts' colours, by state: where the key bears, a gap of under a
 * millimetre still open, and drawn parts overlapping, which a rigid contact
 * should never show and a frame out of fit with its key does.
 */
export const CONTACT_COLOUR: Record<ContactState, string> = {
  closed: '#1d63a8',
  open: '#8a8a8a',
  overlapping: '#e8590c',
}

/**
 * The lines' colours where they are marked by provenance: how the number
 * that places each is known, in the legend's order.
 *
 * A marked line is one pixel wide and drawn over every fill above, flat in
 * the section and lit on the solid's faces, so each colour is chosen apart
 * from all of them as well as from the others and from an unmarked edge: a
 * hue no material is painted in, saturated where the fills are not, so a
 * line does not sink into the felt, the cloth or the wood it outlines. Only
 * what is set on the bench is black, the one colour of ink. Reds, greens and
 * blues alone would collapse for a reader blind to red or green, so the
 * colours also step in lightness, from the dark green of a measurement to the
 * light orange of a trace; the pair such a reader tells apart least,
 * borrowed and assumed, are the two ways of not knowing a number on this
 * instrument. `palette.test.ts` holds the distances.
 */
export const HOW_COLOUR: Record<How, string> = {
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
 * each shining from its position towards the origin. The section is painted
 * unlit.
 */
export const LIGHTS = {
  ambient: 1.6,
  directional: [
    { position: [-200, 300, 400] as [number, number, number], intensity: 2.0 },
    { position: [300, 140, -250] as [number, number, number], intensity: 0.9 },
  ],
} as const
