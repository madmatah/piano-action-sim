/**
 * Each body the engine sends belongs to the frame, the key or the wippen,
 * by its place in the poses.
 */
import { describe, expect, it } from 'vitest'
import { assemblyOf } from './bodies'

describe('a body of the poses', () => {
  it('is the frame first, then the slide and the key, then the wippen last', () => {
    expect([0, 1, 2, 3].map(assemblyOf)).toEqual(['frame', 'key', 'key', 'wippen'])
  })
})
