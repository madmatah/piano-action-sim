import * as THREE from 'three'
import { describe, expect, it } from 'vitest'
import {
  blendingOf,
  chooseSampleCount,
  DRAWN_APART,
  divideForTransparency,
  EDGE_OF_A_WHOLE_PART,
  PASS,
  type PassUniform,
  patchFadedShader,
  readThePass,
} from './transparency'

/** How opaque the reader chose the key to be. */
const FADED = 0.45

/** Another opacity, so that two parts are drawn at two: a blend that kept
 * only the part drawn last would then leave each order its own image. */
const FADED_FURTHER = 0.2

/** A part of the solid as `SagittalView` draws it: Lambert, drawn apart
 * where faded. */
function buildSolidPart(opacity: number): THREE.Mesh {
  const material = new THREE.MeshLambertMaterial({ transparent: opacity < 1, opacity })
  material.userData[DRAWN_APART] = true
  return new THREE.Mesh(new THREE.BufferGeometry(), material)
}

/** A part's edges as `SagittalView` draws them: faint lines, flagged as a
 * whole part's where `isOfAWholePart`. */
function buildEdges(opacity: number, isOfAWholePart: boolean): THREE.LineSegments {
  const material = new THREE.LineBasicMaterial({ transparent: true, opacity })
  material.userData[EDGE_OF_A_WHOLE_PART] = isOfAWholePart
  return new THREE.LineSegments(new THREE.BufferGeometry(), material)
}

/** What a faded part of `color` and `opacity` writes in `pass`, as the
 * patched shader writes it. */
function writeFragment(color: [number, number, number], opacity: number, pass: number): number[] {
  return pass === PASS.accumulated
    ? [...color.map((channel) => channel * opacity), opacity]
    : [opacity, opacity, opacity, opacity]
}

/** What `source` leaves in `destination` blended by `pass`, channel by
 * channel, as WebGL blends with the factors the pass names. */
function blend(
  destination: number[],
  source: number[],
  pass: typeof PASS.accumulated | typeof PASS.revealed,
): number[] {
  const { source: sourceFactor, destination: destinationFactor } = blendingOf(pass)
  const factor = (named: number, channel: number) => {
    if (named === THREE.OneFactor) return 1
    if (named === THREE.ZeroFactor) return 0
    if (named === THREE.OneMinusSrcColorFactor) return 1 - (source[channel] as number)
    throw new Error(`no factor ${named} in this test`)
  }
  return destination.map(
    (channel, index) =>
      (source[index] as number) * factor(sourceFactor, index) +
      channel * factor(destinationFactor, index),
  )
}

describe('a faded part of the solid', () => {
  it('reads its pass from a uniform, so that one compile serves every pass', () => {
    expect(patchFadedShader(THREE.ShaderLib.lambert.fragmentShader)).toContain(
      '#include <common>\nuniform int fadedPass;',
    )
  })

  it('writes its color weighted by its opacity to the sum, and its opacity to what is behind it', () => {
    const fragment = patchFadedShader(THREE.ShaderLib.lambert.fragmentShader)
    expect(fragment).toContain(
      `if (fadedPass == ${PASS.accumulated}) gl_FragColor = vec4(gl_FragColor.rgb * gl_FragColor.a, gl_FragColor.a);`,
    )
    expect(fragment).toContain(
      `else if (fadedPass == ${PASS.revealed}) gl_FragColor = vec4(gl_FragColor.a);`,
    )
  })

  it('writes what its pass gathers in a switch that follows the fragment’s last `#include`', () => {
    const fragment = patchFadedShader(THREE.ShaderLib.lambert.fragmentShader)
    expect(fragment.indexOf('fadedPass ==')).toBeGreaterThan(fragment.lastIndexOf('#include <'))
  })

  it('compiles reading its pass from its view’s uniform', () => {
    const { lambert } = THREE.ShaderLib
    const viewPass: PassUniform = { value: PASS.blended }
    const shader = {
      uniforms: THREE.UniformsUtils.clone(lambert.uniforms),
      vertexShader: lambert.vertexShader,
      fragmentShader: lambert.fragmentShader,
    }
    readThePass(shader, viewPass)
    expect(shader.uniforms.fadedPass).toBe(viewPass)
    expect(shader.fragmentShader).toBe(patchFadedShader(lambert.fragmentShader))
  })
})

describe('the images the faded parts are drawn into', () => {
  const keyButton = { color: [0.66, 0.44, 0.25] as [number, number, number], opacity: FADED }
  const keystick = {
    color: [0.93, 0.86, 0.71] as [number, number, number],
    opacity: FADED_FURTHER,
  }

  /** An image drawn the key button first, then the keystick first. */
  function drawInBothOrders(
    pass: typeof PASS.accumulated | typeof PASS.revealed,
    cleared: number[],
  ): [number[], number[]] {
    const draw = (first: typeof keyButton, second: typeof keyButton) =>
      blend(
        blend(cleared, writeFragment(first.color, first.opacity, pass), pass),
        writeFragment(second.color, second.opacity, pass),
        pass,
      )
    return [draw(keyButton, keystick), draw(keystick, keyButton)]
  }

  it('sum the same colors whichever part is drawn first', () => {
    const [buttonFirst, stickFirst] = drawInBothOrders(PASS.accumulated, [0, 0, 0, 0])
    expect(buttonFirst).toEqual(stickFirst)
  })

  it('leave the same of what is behind whichever part is drawn first', () => {
    const [buttonFirst, stickFirst] = drawInBothOrders(PASS.revealed, [1, 1, 1, 1])
    expect(buttonFirst).toEqual(stickFirst)
  })
})

describe('how many samples the images take of a pixel', () => {
  /** The samples the canvas takes of a pixel. */
  const CANVAS_SAMPLES = 4

  /** The counts a format offers, as browsers list them, most first. */
  const OFFERED = [8, 4, 2]

  /** The fewest samples a smoothed image takes. */
  const FEWEST = 2

  /** three's count for an image that takes one sample of a pixel. */
  const SINGLE_SAMPLE = 0

  it('takes as many as the canvas where every format offers them', () => {
    expect(chooseSampleCount(CANVAS_SAMPLES, OFFERED, OFFERED)).toBe(CANVAS_SAMPLES)
  })

  it('takes the most that every format offers, where their offers differ', () => {
    expect(chooseSampleCount(CANVAS_SAMPLES, OFFERED, [FEWEST])).toBe(FEWEST)
  })

  it('takes a single sample where a format offers no count', () => {
    expect(chooseSampleCount(CANVAS_SAMPLES, OFFERED, [])).toBe(SINGLE_SAMPLE)
  })

  it('takes a single sample where the canvas takes one', () => {
    expect(chooseSampleCount(SINGLE_SAMPLE, OFFERED, OFFERED)).toBe(SINGLE_SAMPLE)
  })
})

describe('what a view draws', () => {
  it('draws a faded part of the solid apart', () => {
    const scene = new THREE.Scene()
    const part = buildSolidPart(FADED)
    scene.add(part)
    expect(divideForTransparency(scene).faded).toEqual([part])
  })

  it('draws a part of the solid drawn whole with the opaque ones', () => {
    const scene = new THREE.Scene()
    const part = buildSolidPart(1)
    scene.add(part)
    expect(divideForTransparency(scene)).toEqual({
      faded: [],
      edgesOfWholeParts: [],
      overlays: [],
      opaque: [part],
    })
  })

  it('draws the edges of a part drawn whole with it, under the faded parts', () => {
    const scene = new THREE.Scene()
    const edges = buildEdges(FADED, true)
    scene.add(edges)
    expect(divideForTransparency(scene)).toEqual({
      faded: [],
      edgesOfWholeParts: [edges],
      overlays: [],
      opaque: [],
    })
  })

  it('draws a faded part’s edges and a mark over the rest', () => {
    const scene = new THREE.Scene()
    const edges = buildEdges(FADED, false)
    const mark = new THREE.Mesh(
      new THREE.BufferGeometry(),
      new THREE.MeshBasicMaterial({ transparent: true, depthTest: false }),
    )
    scene.add(edges, mark)
    expect(divideForTransparency(scene)).toEqual({
      faded: [],
      edgesOfWholeParts: [],
      overlays: [edges, mark],
      opaque: [],
    })
  })

  it('leaves out what it hides', () => {
    const scene = new THREE.Scene()
    const hidden = new THREE.Group()
    hidden.visible = false
    hidden.add(buildSolidPart(FADED))
    scene.add(hidden)
    expect(divideForTransparency(scene)).toEqual({
      faded: [],
      edgesOfWholeParts: [],
      overlays: [],
      opaque: [],
    })
  })
})
