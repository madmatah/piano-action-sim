/**
 * The languages the page can be worded in, each offered by its name in
 * itself, so that a reader who cannot read the language in use finds their
 * own; the one in use is marked.
 */
import { ENDONYM, LANGUAGES, type Language, useCatalog } from './i18n'

export function LanguageChoice({
  language,
  choose,
}: {
  /** The language the page is worded in. */
  language: Language
  /** Rewords the page in `chosen`. */
  choose(chosen: Language): void
}) {
  const t = useCatalog()
  return (
    <nav className="languages" aria-label={t.languageSelector}>
      {LANGUAGES.map((offered) => (
        <button
          key={offered}
          type="button"
          lang={offered}
          className={offered === language ? 'on' : ''}
          data-language={offered}
          aria-pressed={offered === language}
          onClick={() => choose(offered)}
        >
          {ENDONYM[offered]}
        </button>
      ))}
    </nav>
  )
}
