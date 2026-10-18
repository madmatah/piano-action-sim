/**
 * The wording the page shows, from its catalog. English is the one built:
 * every component reads `t`, so a second catalog replaces it whole.
 */
import type { Catalog } from './catalog'
import { en } from './en'

export type { Catalog, Labeled, Laid, Reported } from './catalog'
export { unitBeside, word } from './catalog'

/** The catalog in use. */
export const t: Catalog = en
