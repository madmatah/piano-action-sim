import * as THREE from 'three'
import { describe, expect, it } from 'vitest'
import type { Box } from './bounds'
import { buildCarvingUniforms, type Hollow, MAX_CUTS } from './carving'
import { shadeSolidPart } from './shading'
import { PASS, type PassUniform } from './transparency'

/** A cut through a part, millimeters: the balance mortise's slot in the
 * measured section's key. */
const SLOT: Box = { minMm: [-5.1, 2.5, -3], maxMm: [5.1, 30.5, 3] }

/** The box a cut's walls are kept within: the key button they run through. */
const KEY_BUTTON: Box = { minMm: [-31, 24.5, -11], maxMm: [31, 30.5, 11] }

/** The pass the view's parts read, written before each of its passes. */
const VIEW_PASS: PassUniform = { value: PASS.blended }

/** Every part the solid draws as far as its compile could tell them apart:
 * each count of cuts from the most the shader carves to none, their ends
 * rounded or not, kept within a box or not, in a view that gives it a pass
 * or not. */
const PARTS = Array.from({ length: MAX_CUTS + 1 }, (_, fewer) => MAX_CUTS - fewer).flatMap(
  (count) =>
    [true, false].flatMap((hasRoundEnds) =>
      [KEY_BUTTON, null].flatMap((keep) =>
        [VIEW_PASS, null].map((pass) => ({
          name: `${count} cuts${hasRoundEnds ? ' with rounded ends' : ''}, ${keep ? 'kept within a box' : 'no box to keep within'}, ${pass ? 'a pass' : 'no pass'}`,
          part: {
            cuts: Array.from({ length: count }, (): Hollow => ({ box: SLOT, hasRoundEnds })),
            keep,
            pass,
          },
        })),
      ),
    ),
)

/** What three builds a part's program from, compiled as three compiles it:
 * the part's hook run on a fresh copy of the Lambert shader, and every
 * uniform its sources declare that the compile gives no value. */
function compile(part: (typeof PARTS)[number]['part']) {
  const { material } = shadeSolidPart(part, buildCarvingUniforms())
  const { lambert } = THREE.ShaderLib
  const shader = {
    uniforms: THREE.UniformsUtils.clone(lambert.uniforms),
    vertexShader: lambert.vertexShader,
    fragmentShader: lambert.fragmentShader,
  }
  material.onBeforeCompile(shader)
  const declared = [
    ...`${shader.vertexShader}\n${shader.fragmentShader}`.matchAll(/^uniform \w+ (\w+)/gm),
  ].map(([, name]) => name as string)
  return {
    key: material.customProgramCacheKey(),
    vertex: shader.vertexShader,
    fragment: shader.fragmentShader,
    uniforms: Object.keys(shader.uniforms).sort(),
    userData: material.userData,
    unbound: declared.filter((name) => !(name in shader.uniforms)),
  }
}

describe('a part of the solid', () => {
  /**
   * three builds a material's program once for each key it meets, and runs
   * the material's compile hook, keeping that compile's uniforms, only for a
   * key it has not met (three 0.186.1, `WebGLRenderer`'s `getProgram`); a
   * built-in material's key holds its `customProgramCacheKey`, not its
   * source, so materials under one cache key share the program of the
   * first compile wherever three keys them alike in the rest. A part
   * compiled one way for some cuts and another way for others would, back
   * under a key it had left, be drawn with another compile's source or with
   * uniforms that hold none of its cuts.
   */
  it('compiles every part of the solid the same way under one cache key, whatever cuts run through it and whether or not its view gives it a pass', () => {
    const [reference, ...others] = PARTS.map(({ name, part }) => ({ name, program: compile(part) }))
    const fields = ['key', 'vertex', 'fragment', 'uniforms', 'userData'] as const
    const differing = others
      .map(({ name, program }) => ({
        name,
        fields: fields.filter(
          (field) => JSON.stringify(program[field]) !== JSON.stringify(reference?.program[field]),
        ),
      }))
      .filter(({ fields: differ }) => differ.length > 0)
      .map(({ name, fields: differ }) => `${name}: ${differ.join(', ')}`)
    expect(differing).toEqual([])
  })

  it('binds every uniform its patches declare', () => {
    const unbound = PARTS.map(({ name, part }) => ({ name, unbound: compile(part).unbound }))
      .filter(({ unbound: names }) => names.length > 0)
      .map(({ name, unbound: names }) => `${name}: ${names.join(', ')}`)
    expect(unbound).toEqual([])
  })

  it('compiles both patches into a part of the solid', () => {
    const { fragment } = compile({
      cuts: [{ box: SLOT, hasRoundEnds: true }],
      keep: null,
      pass: VIEW_PASS,
    })
    expect(fragment).toContain('uniform vec3 cutMin[MAX_CUTS];')
    expect(fragment).toContain('uniform int fadedPass;')
    expect(fragment.indexOf('fadedPass ==')).toBeGreaterThan(fragment.lastIndexOf('#include <'))
  })
})
