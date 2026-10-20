/**
 * How the solid draws the parts it fades: whatever order it draws them in.
 *
 * three.js draws transparent objects one after another, sorted by how far
 * their middles stand from the camera, each blended over what is already
 * drawn, so where two faded parts lie one behind the other the one drawn
 * last gives the pixel its color. The key button and the keystick are woods
 * of two colors, one glued on the other, and a camera turned past the angle
 * where their middles stand as far swaps them: the key button would change
 * color all at once. The solid draws its faded parts instead by weighted
 * blended order-independent transparency (McGuire and Bavoil, "Weighted
 * Blended Order-Independent Transparency", Journal of Computer Graphics
 * Techniques 2(2):122–141, 2013, http://jcgt.org/published/0002/02/09/),
 * written here from the paper's equations: apart from the rest, into two
 * images whose blending commutes, the sum of their colors weighted by
 * their opacity and the product of what each leaves of what is behind it;
 * then it lays them over the opaque parts. Every face stays drawn, each
 * mortise's walls included, and no pixel's color depends on which part is
 * drawn first, to the half precision the images hold. A browser that
 * cannot draw into such images draws the solid as three sorts it.
 *
 * Every face has an equal weight, each color premultiplied by its opacity,
 * and no weight for depth: where faded faces overlap, the composite is the
 * mean of their colors weighted by their opacities, whatever their order,
 * so a nearly opaque face in front shows mixed with what stands behind it.
 * A depth weight steep enough to favor the nearer of two faces a few
 * millimeters apart shades a single face unevenly across its own depth, and
 * a gentle one changes nothing a reader can see.
 */

import * as THREE from 'three'

/** What a faded part writes, by the pass it is drawn in: as any part, over
 * what is drawn; its color weighted by its opacity, which the first image
 * sums; or its opacity, which the second image multiplies out of what is
 * behind it. */
export const PASS = { blended: 0, accumulated: 1, revealed: 2 } as const

/** A pass a faded part is drawn in. */
export type Pass = (typeof PASS)[keyof typeof PASS]

/** The uniform every faded part's program reads its pass from: one for a
 * whole view, written before each of its passes. */
export type PassUniform = { value: Pass }

/** The flag in a material's `userData` that has the solid draw it apart
 * where it is faded: a material its shader is patched for
 * ([`patchFadedShader`]). */
export const DRAWN_APART = 'isDrawnApart'

/** The flag in an edge's material's `userData` that has the solid draw it
 * with the part it outlines, a part drawn whole: under the faded parts, so
 * that a faded key in front of it dims it as it dims the part. */
export const EDGE_OF_A_WHOLE_PART = 'isEdgeOfAWholePart'

/**
 * Patches a three.js fragment shader so that its part, drawn in a pass
 * other than [`PASS.blended`], writes what that pass gathers, its uniform
 * `fadedPass` saying which: in a switch after the shader's last step, so
 * that it gathers the color the part would have been blended in.
 */
export function patchFadedShader(fragment: string): string {
  return fragment.replace('#include <common>', '#include <common>\nuniform int fadedPass;').replace(
    '#include <dithering_fragment>',
    `#include <dithering_fragment>
\tif (fadedPass == ${PASS.accumulated}) gl_FragColor = vec4(gl_FragColor.rgb * gl_FragColor.a, gl_FragColor.a);
\telse if (fadedPass == ${PASS.revealed}) gl_FragColor = vec4(gl_FragColor.a);`,
  )
}

/** What a material's compile hands its hook ([`THREE.Material.onBeforeCompile`]):
 * the uniforms its program reads, and the sources it builds it from, both
 * the hook's to change. */
export type CompilingShader = {
  uniforms: Record<string, THREE.IUniform>
  vertexShader: string
  fragmentShader: string
}

/** Patches a compiling shader so that its part, faded, can be drawn apart,
 * reading its pass from `pass` ([`patchFadedShader`]). */
export function readThePass(shader: CompilingShader, pass: PassUniform): void {
  shader.uniforms.fadedPass = pass
  shader.fragmentShader = patchFadedShader(shader.fragmentShader)
}

/** How a pass blends what a faded part writes into its image: the first
 * adds it, the second multiplies the image by what the part leaves of what
 * is behind it. Both commute, so the order the parts come in changes
 * nothing. */
export function blendingOf(pass: typeof PASS.accumulated | typeof PASS.revealed): {
  source: THREE.BlendingSrcFactor
  destination: THREE.BlendingDstFactor
} {
  return pass === PASS.accumulated
    ? { source: THREE.OneFactor, destination: THREE.OneFactor }
    : { source: THREE.ZeroFactor, destination: THREE.OneMinusSrcColorFactor }
}

/** The objects a view draws, divided by how the solid draws them. */
export type Division = {
  /** The faded parts, drawn apart. */
  readonly faded: readonly THREE.Object3D[]
  /** The edges of the parts drawn whole, drawn with them, under the faded
   * parts. */
  readonly edgesOfWholeParts: readonly THREE.Object3D[]
  /** Every other transparent object, a faded part's edges and the marks,
   * drawn over the rest as three sorts them. */
  readonly overlays: readonly THREE.Object3D[]
  /** The opaque ones. */
  readonly opaque: readonly THREE.Object3D[]
}

/** Divides what `scene` draws, the objects it shows that have a material,
 * by how the solid draws them ([`Division`]). */
export function divideForTransparency(scene: THREE.Object3D): Division {
  const faded: THREE.Object3D[] = []
  const edgesOfWholeParts: THREE.Object3D[] = []
  const overlays: THREE.Object3D[] = []
  const opaque: THREE.Object3D[] = []
  scene.traverseVisible((object) => {
    const { material } = object as { material?: THREE.Material | THREE.Material[] }
    if (material === undefined || Array.isArray(material)) return
    if (material.transparent && material.opacity < 1 && material.userData[DRAWN_APART] === true) {
      faded.push(object)
    } else if (material.userData[EDGE_OF_A_WHOLE_PART] === true) {
      edgesOfWholeParts.push(object)
    } else if (material.transparent) {
      overlays.push(object)
    } else {
      opaque.push(object)
    }
  })
  return { faded, edgesOfWholeParts, overlays, opaque }
}

/**
 * How many samples both images take of a pixel: the most that every
 * image's format offers, `formatCounts` listing each format's, and no more
 * than the canvas takes, `canvasSamples`, so that the faded parts' edges
 * are smoothed as the opaque ones are; none where no count is shared. One
 * count for both images keeps their smoothed edges in step where the
 * composite divides one by the other.
 */
export function chooseSampleCount(
  canvasSamples: number,
  ...formatCounts: ReadonlyArray<readonly number[]>
): number {
  const [first = [], ...others] = formatCounts
  const shared = first.filter(
    (count) => count <= canvasSamples && others.every((counts) => counts.includes(count)),
  )
  return shared.length > 0 ? Math.max(...shared) : 0
}

/** The sample counts the browser offers an image of `format`, none where
 * it cannot draw into one. */
function offerSampleCounts(gl: WebGL2RenderingContext, format: number): number[] {
  const counts: unknown = gl.getInternalformatParameter(gl.RENDERBUFFER, format, gl.SAMPLES)
  return counts instanceof Int32Array ? [...counts] : []
}

const COMPOSITE_VERTEX = `varying vec2 vUv;
void main() {
\tvUv = uv;
\tgl_Position = vec4(position.xy, 0.0, 1.0);
}`

const COMPOSITE_FRAGMENT = `uniform sampler2D accumulated;
uniform sampler2D revealed;
varying vec2 vUv;
void main() {
\tvec4 sum = texture2D(accumulated, vUv);
\tfloat behind = texture2D(revealed, vUv).r;
\tgl_FragColor = vec4(sum.rgb / max(sum.a, 1e-5), 1.0 - behind);
\t#include <tonemapping_fragment>
\t#include <colorspace_fragment>
}`

/**
 * Draws a view, its faded parts apart ([`divideForTransparency`]): the
 * opaque parts and the edges of the parts drawn whole on the canvas; then,
 * into each of two images, the faded parts, depth-tested against the opaque
 * parts drawn first in the same render with no color; then the two images
 * laid over the canvas; then a faded part's edges and the marks. A view
 * with no faded part, or a browser that cannot draw into an image of
 * fractional colors, is drawn as three draws it, sorted.
 */
export class FadedDrawing {
  readonly #renderer: THREE.WebGLRenderer
  readonly #canDraw: boolean
  readonly #accumulated: THREE.WebGLRenderTarget
  readonly #revealed: THREE.WebGLRenderTarget
  /** Each image, the pass that draws into it, and what it is cleared to:
   * the sum starts from nothing, and what is behind from all of it. */
  readonly #images: ReadonlyArray<{
    image: THREE.WebGLRenderTarget
    pass: typeof PASS.accumulated | typeof PASS.revealed
    cleared: THREE.Color
    clearedAlpha: number
  }>
  readonly #composite: THREE.Mesh<THREE.PlaneGeometry, THREE.ShaderMaterial>
  readonly #compositeCamera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1)
  readonly #pixels = new THREE.Vector2()
  readonly #clearColor = new THREE.Color()
  readonly #colorWrites = new Map<THREE.Material, boolean>()

  /** Prepares to draw with `renderer`, whose canvas and browser decide how
   * the images are made. */
  constructor(renderer: THREE.WebGLRenderer) {
    this.#renderer = renderer
    this.#canDraw =
      renderer.extensions.has('EXT_color_buffer_float') ||
      renderer.extensions.has('EXT_color_buffer_half_float')
    // three draws only into a WebGL 2 context: asking tells the types so.
    const gl = renderer.getContext()
    const samples =
      this.#canDraw && gl instanceof WebGL2RenderingContext
        ? chooseSampleCount(
            renderer.capabilities.samples,
            offerSampleCounts(gl, gl.RGBA16F),
            offerSampleCounts(gl, gl.R16F),
          )
        : 0
    // The depth is read only while each image is drawn, never once it is
    // resolved, as three's own transmission image does; not keeping it
    // saves memory only where three can discard a buffer, which is not on
    // a desktop browser.
    const options = {
      type: THREE.HalfFloatType,
      samples,
      resolveDepthBuffer: false,
      storeMultisampledDepthBuffer: false,
    }
    this.#accumulated = new THREE.WebGLRenderTarget(1, 1, options)
    // The composite reads one channel of what is behind.
    this.#revealed = new THREE.WebGLRenderTarget(1, 1, { ...options, format: THREE.RedFormat })
    this.#images = [
      {
        image: this.#accumulated,
        pass: PASS.accumulated,
        cleared: new THREE.Color(0, 0, 0),
        clearedAlpha: 0,
      },
      {
        image: this.#revealed,
        pass: PASS.revealed,
        cleared: new THREE.Color(1, 1, 1),
        clearedAlpha: 1,
      },
    ]
    this.#composite = new THREE.Mesh(
      new THREE.PlaneGeometry(2, 2),
      new THREE.ShaderMaterial({
        uniforms: {
          accumulated: { value: this.#accumulated.texture },
          revealed: { value: this.#revealed.texture },
        },
        vertexShader: COMPOSITE_VERTEX,
        fragmentShader: COMPOSITE_FRAGMENT,
        transparent: true,
        depthTest: false,
        depthWrite: false,
      }),
    )
  }

  /** Draws `scene` as `camera` sees it, its faded parts' programs reading
   * their pass from `pass`. */
  draw(scene: THREE.Scene, camera: THREE.Camera, pass: PassUniform): void {
    const renderer = this.#renderer
    const { faded, edgesOfWholeParts, overlays, opaque } = divideForTransparency(scene)
    renderer.setRenderTarget(null)
    if (faded.length === 0 || !this.#canDraw) {
      // Shrunk, the images give back their memory until a part fades.
      this.#accumulated.setSize(1, 1)
      this.#revealed.setSize(1, 1)
      renderer.autoClear = true
      renderer.render(scene, camera)
      return
    }
    renderer.getDrawingBufferSize(this.#pixels)
    this.#accumulated.setSize(this.#pixels.x, this.#pixels.y)
    this.#revealed.setSize(this.#pixels.x, this.#pixels.y)
    renderer.getClearColor(this.#clearColor)
    const clearAlpha = renderer.getClearAlpha()
    renderer.autoClear = false

    show(faded, false)
    show(overlays, false)
    renderer.clear()
    renderer.render(scene, camera)

    // three draws its opaque list before its transparent one, and writes
    // color again once it is done, so the opaque parts lay their depth
    // first in each image, each with its own material, a carved part
    // leaving its cuts open for the faded parts standing in them.
    show(edgesOfWholeParts, false)
    show(faded, true)
    this.#colorWrites.clear()
    for (const object of opaque) {
      const material = (object as THREE.Mesh).material as THREE.Material
      this.#colorWrites.set(material, material.colorWrite)
    }
    for (const material of this.#colorWrites.keys()) material.colorWrite = false
    for (const { image, pass: imagePass, cleared, clearedAlpha } of this.#images) {
      renderer.setRenderTarget(image)
      renderer.setClearColor(cleared, clearedAlpha)
      renderer.clear()
      blendFaded(faded, imagePass)
      pass.value = imagePass
      renderer.render(scene, camera)
    }
    for (const [material, colorWrite] of this.#colorWrites) material.colorWrite = colorWrite
    this.#colorWrites.clear()
    pass.value = PASS.blended
    blendFaded(faded, PASS.blended)
    renderer.setClearColor(this.#clearColor, clearAlpha)
    renderer.setRenderTarget(null)
    renderer.render(this.#composite, this.#compositeCamera)

    show(opaque, false)
    show(faded, false)
    show(overlays, true)
    renderer.render(scene, camera)
    show(opaque, true)
    show(faded, true)
    show(edgesOfWholeParts, true)
  }

  /** Frees the images and the program that lays them over the canvas. */
  dispose(): void {
    this.#accumulated.dispose()
    this.#revealed.dispose()
    this.#composite.geometry.dispose()
    this.#composite.material.dispose()
  }
}

function show(objects: readonly THREE.Object3D[], visible: boolean): void {
  for (const object of objects) object.visible = visible
}

function blendFaded(faded: readonly THREE.Object3D[], pass: Pass): void {
  if (pass === PASS.blended) {
    for (const object of faded) {
      const material = (object as THREE.Mesh).material as THREE.Material
      material.blending = THREE.NormalBlending
    }
    return
  }
  const { source, destination } = blendingOf(pass)
  for (const object of faded) {
    const material = (object as THREE.Mesh).material as THREE.Material
    material.blending = THREE.CustomBlending
    material.blendEquation = THREE.AddEquation
    material.blendSrc = source
    material.blendDst = destination
    material.blendSrcAlpha = source
    material.blendDstAlpha = destination
  }
}
