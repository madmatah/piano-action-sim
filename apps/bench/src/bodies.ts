/**
 * The engine's bodies, by their place in the poses it sends (`State.poses`):
 * the ground, which carries the frame; the slide the key lifts off its
 * balance punching on; the key; and the wippen, last, where the key carries
 * one.
 */

/** The ground, which carries the frame: body 0. */
export const GROUND = 0

/** The wippen the key carries, last of the bodies: body 3. */
export const WIPPEN = 3

/** What a body of the view belongs to: the frame, the key with the slide
 * it lifts on, or the wippen the key carries. */
export type Assembly = 'frame' | 'key' | 'wippen'

/** What the body at `body` in the poses belongs to. */
export function assemblyOf(body: number): Assembly {
  if (body === GROUND) return 'frame'
  return body === WIPPEN ? 'wippen' : 'key'
}
