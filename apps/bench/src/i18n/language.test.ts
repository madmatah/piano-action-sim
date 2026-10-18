/**
 * The page opens in the reader's language where the bench speaks it, and
 * offers every language it has a catalog for.
 */
import { describe, expect, it } from 'vitest'
import { ENDONYM, LANGUAGES, pickLanguage } from './language'

describe('the language the page opens in', () => {
  it('is the first the reader prefers that the bench speaks, whatever its region', () => {
    expect(pickLanguage(['de-DE', 'fr-CA', 'en-US'])).toBe('fr')
  })

  it('is read from a language tag whatever its case', () => {
    expect(pickLanguage(['EN-gb', 'fr'])).toBe('en')
  })

  it('is English where the bench speaks none of the reader’s languages', () => {
    expect(pickLanguage(['de', 'es-ES'])).toBe('en')
    expect(pickLanguage([])).toBe('en')
  })

  it('is never a name that every JavaScript object carries', () => {
    expect(pickLanguage(['constructor', '__proto__', 'toString'])).toBe('en')
  })
})

describe('the selector', () => {
  it('names each language in itself', () => {
    expect(LANGUAGES.map((language) => ENDONYM[language])).toEqual(['English', 'Français'])
  })
})
