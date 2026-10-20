/**
 * How a part of the solid is shaded: one compile hook under one cache key
 * for every part, carved by the cuts that run through it and drawn apart
 * where it is faded.
 *
 * three builds a material's program once for each key it meets, and runs
 * the material's compile hook, keeping that compile's uniforms, only for a
 * key it has not met; a built-in material's key holds its
 * `customProgramCacheKey`, not its source, beside what three draws it with,
 * such as its side and the color space of the image it is drawn into, so
 * materials under one cache key share a program wherever they share the
 * rest. Every part of the solid is therefore compiled the same way,
 * whatever its cuts and whether or not its view gives it a pass: through
 * the carving patch, then the faded one, under one cache key. A part
 * nothing cuts is carved by no cut and kept within a box past every part,
 * so it is painted whole; its cuts are uniforms, written in place as they
 * change.
 */

import type { Box } from './bounds'
import {
  type CarvingUniforms,
  type Hollow,
  type PackedCuts,
  packCuts,
  patchShader,
} from './carving'
import {
  type CompilingShader,
  DRAWN_APART,
  PASS,
  type PassUniform,
  readThePass,
} from './transparency'

/** The one cache key every part of the solid compiles under, with one
 * compile hook. */
export const SOLID_PART_PROGRAM = 'solid part'

/** What a part of the solid compiles from: the cuts through it, the box it
 * is kept within where it is a cut's walls, and the pass its view draws it
 * in, where its view draws faded parts apart. */
export type SolidPart = {
  readonly cuts: readonly Hollow[]
  readonly keep: Box | null
  readonly pass: PassUniform | null
}

/** What a part of the solid's material takes: how it compiles, and the cuts
 * its uniforms are to hold. */
export type SolidPartShading = {
  /** The material's compile hook, its program's cache key, and the flag
   * that has the solid draw it apart where it is faded. */
  readonly material: {
    readonly onBeforeCompile: (shader: CompilingShader) => void
    readonly customProgramCacheKey: () => string
    readonly userData: Record<string, unknown>
  }
  /** The cuts to write into `carving` ([`writeCarving`]). */
  readonly packed: PackedCuts
}

/** Shades a part of the solid, its cuts read from `carving`, the material's
 * own uniforms, made once. */
export function shadeSolidPart(
  { cuts, keep, pass }: SolidPart,
  carving: CarvingUniforms,
): SolidPartShading {
  const drawnIn = pass ?? { value: PASS.blended }
  return {
    material: {
      onBeforeCompile: (shader) => {
        Object.assign(shader.uniforms, carving)
        const patched = patchShader(shader.vertexShader, shader.fragmentShader)
        shader.vertexShader = patched.vertex
        shader.fragmentShader = patched.fragment
        readThePass(shader, drawnIn)
      },
      customProgramCacheKey: () => SOLID_PART_PROGRAM,
      userData: { [DRAWN_APART]: true },
    },
    packed: packCuts(cuts, keep),
  }
}
