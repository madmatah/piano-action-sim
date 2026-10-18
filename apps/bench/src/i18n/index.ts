/**
 * The wording the page shows, from the catalog of the language chosen. The
 * page provides that catalog to every component by context, so that choosing
 * another language rewords the page and changes nothing else.
 */
import { createContext, useContext } from 'react'
import type { Catalog } from './catalog'

export type { Catalog, Labeled, Laid, Reported } from './catalog'
export { unitBeside, word } from './catalog'
export { CATALOG, ENDONYM, LANGUAGES, type Language, pickLanguage } from './language'

/** The catalog of the language chosen, which the page provides. */
export const CatalogContext = createContext<Catalog | null>(null)

/** The catalog of the language chosen. A component rendered outside the
 * page's provider has none, and throws rather than show another language
 * than the page's. */
export function useCatalog(): Catalog {
  const catalog = useContext(CatalogContext)
  if (catalog === null) throw new Error('a component read the catalog outside the page')
  return catalog
}
