/**
 * The languages the bench is worded in, each by its catalog, and the one a
 * reader's browser asks for.
 */
import type { Catalog } from './catalog'
import { en } from './en'
import { fr } from './fr'

/** A language the bench speaks, by its primary language subtag. */
export type Language = 'en' | 'fr'

/** Each language's catalog. */
export const CATALOG: { readonly [L in Language]: Catalog } = { en, fr }

/** Each language named in itself, as the selector offers it to a reader who
 * may not read the language the page is in. */
export const ENDONYM: { readonly [L in Language]: string } = { en: 'English', fr: 'Français' }

/** The languages, in the order the selector offers them. */
export const LANGUAGES = Object.keys(CATALOG) as readonly Language[]

function isLanguage(subtag: string): subtag is Language {
  return Object.hasOwn(CATALOG, subtag)
}

/**
 * The language the page opens in: the first of the reader's preferred
 * languages, `navigator.languages`, that the bench speaks, whatever its
 * region (`fr-CA` is French), and English when it speaks none of them.
 */
export function pickLanguage(preferred: readonly string[]): Language {
  for (const tag of preferred) {
    const subtag = tag.split('-')[0]?.toLowerCase() ?? ''
    if (isLanguage(subtag)) return subtag
  }
  return 'en'
}
