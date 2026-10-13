/**
 * Wording with its emphasis: `**bold**` and `*italic*` marked in a
 * catalogue's sentence, and nothing else. The marks travel with the words, so
 * a translation emphasises what its own sentence stresses.
 */
import type { ReactNode } from 'react'

/** Bold, then italic, each run unbroken by another mark. */
const MARK = /\*\*([^*]+)\*\*|\*([^*]+)\*/g

/** The pieces of `text`: plain runs, and the marked ones with their mark. */
export function pieces(text: string): { text: string; mark: 'b' | 'i' | null }[] {
  const out: { text: string; mark: 'b' | 'i' | null }[] = []
  let last = 0
  for (const m of text.matchAll(MARK)) {
    const at = m.index ?? 0
    if (at > last) out.push({ text: text.slice(last, at), mark: null })
    out.push(m[1] === undefined ? { text: m[2] ?? '', mark: 'i' } : { text: m[1], mark: 'b' })
    last = at + m[0].length
  }
  if (last < text.length) out.push({ text: text.slice(last), mark: null })
  return out
}

/** `text` with its emphasis. */
export function Rich({ text }: { text: string }) {
  const nodes: ReactNode[] = pieces(text).map((p, i) =>
    p.mark === 'b' ? <b key={i}>{p.text}</b> : p.mark === 'i' ? <i key={i}>{p.text}</i> : p.text,
  )
  return <>{nodes}</>
}
