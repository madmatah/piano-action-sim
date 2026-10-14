/**
 * A value of an engine document by its address.
 *
 * The settings are nested as their paths are: the value at
 * `frame.front_rail.pin_height_mm` is `settings.frame.front_rail.pin_height_mm`.
 * So a control the registry describes by its path finds its value by walking
 * the settings along it, and the page names no setting a control reads. What
 * the walk finds is checked for what it must be, a number or a list of
 * numbers, never cast to it; a test of the engine's
 * (`the_registry_describes_every_path_and_the_page`) holds every address the
 * page is given to the shape it reads there.
 */
import type { Param, Registry } from '@pas/wasm/types'

/** What stands at `address` in `document`, or `undefined` where nothing does. */
export function at(document: object, address: string): unknown {
  let here: unknown = document
  for (const key of address.split('.')) {
    if (typeof here !== 'object' || here === null || Array.isArray(here)) return undefined
    here = Object.entries(here).find(([k]) => k === key)?.[1]
  }
  return here
}

/** The number at `address`, or `null` where none is. */
export function numberAt(document: object, address: string): number | null {
  const value = at(document, address)
  return typeof value === 'number' ? value : null
}

/** The text at `address`, or `null` where none is. */
export function textAt(document: object, address: string): string | null {
  const value = at(document, address)
  return typeof value === 'string' ? value : null
}

/** The numbers listed at `address`: none where no list is. */
export function numbersAt(document: object, address: string): number[] {
  const value = at(document, address)
  return Array.isArray(value) ? value.filter((x): x is number => typeof x === 'number') : []
}

/** What the registry says of the setting at `path`. */
export function param(registry: Registry, path: string): Param | undefined {
  return registry.params.find((p) => p.path === path)
}
