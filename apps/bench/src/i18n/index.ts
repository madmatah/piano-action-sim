/**
 * The wording the page shows, from its catalogue. English is the one built:
 * every component reads `t`, so a second catalogue replaces it whole.
 */
import type { Catalogue } from './catalogue'
import { en } from './en'

export type { Catalogue, Labelled, Laid, Reported } from './catalogue'
export { unitBeside, word } from './catalogue'

/** The catalogue in use. */
export const t: Catalogue = en
