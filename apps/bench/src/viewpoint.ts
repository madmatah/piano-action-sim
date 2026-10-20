/**
 * Where the camera looks at the bench from in each of its two views. Plain
 * tuples and type-only imports, so that the smoke test, which Node loads by
 * stripping its types, turns the camera from the viewpoint the page opens
 * on and not from a copy of it.
 */

/** Two viewpoints: the technical section, and the solid in three-quarters. */
export type ViewMode = 'section' | 'solid'

/** A viewpoint: where the camera stands, what it looks at, both in
 * millimeters, and how many pixels a millimeter is drawn across. */
export type Viewpoint = {
  readonly position: [number, number, number]
  readonly target: [number, number, number]
  readonly zoom: number
}

/** Each view's viewpoint. */
export const VIEWPOINT: Record<ViewMode, Viewpoint> = {
  // Straight on: a technical section, read for its dimensions.
  section: { position: [-30, 24, 420], target: [-30, 24, 0], zoom: 2.0 },
  // Three-quarters, and **low**: the solid, read for how it is stacked.
  // Looking down from 25° shows nothing but the top of the key — the rails,
  // the punchings and the cloth all hide under it. What a reader needs to see
  // here is the stack, so the elevation is about 12°.
  solid: { position: [-190, 115, 430], target: [-30, 20, 0], zoom: 1.75 },
}
