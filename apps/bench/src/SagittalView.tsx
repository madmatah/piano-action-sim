/**
 * The bench's 3-D view.
 *
 * Built in millimeters: `x` toward the player, `y` up, `z` across the
 * keyboard. Every part arrives in its own body's frame, the frame and its
 * rails on the ground, the key's pieces on the key, and every body arrives
 * with its pose. This component applies the poses it is given, one transform
 * per body, and computes none: if a part stands in the wrong place, the
 * engine put it there.
 *
 * # Why the solids are swept outlines and not models
 *
 * There is **one** definition of each part: the outline the engine builds,
 * which the faults read and the contacts are held to. This view extrudes it,
 * lathes it when the part is turned, or turns the half of its section about
 * its middle and squeezes it to its oval for the front rail pin's head. A
 * separately modeled mesh would be a
 * second geometry, and the moment it disagreed with the first the displayed
 * dimensions would stop describing what is measured. So the section is not a
 * reduced view of the solid: the solid *is* the section, swept.
 *
 * # Two views
 *
 * The **section** is the sagittal plane itself, drawn flat: each part's
 * outline, a turned part's profile beside its mirror, so a washer shows the
 * hole its pin stands in. A cut the plane cuts, a mortise or the balance
 * hole, is a void in its body's solids: drawn into the stencil buffer first,
 * it keeps the key's wood, and the wood's edges, from being painted there,
 * so the pin in it shows through, while a pin standing inside the key's wood
 * stays hidden by it, unless the key is drawn faded, as the reader may
 * choose and as a key no equilibrium places is. A cut's mouth, where it
 * opens through the key's outline or into another cut, is no edge of the
 * section. The parts beside the plane, the bushings, the key button's side
 * holes and the front mortise's groove, are not cut by it (`section.ts`): a
 * part on the far side is drawn as seen beyond the plane where a void opens
 * onto it, each bushing's leg along its mortise's wall through the mortise
 * and behind the pin, and not the balance mortise's leg out through the
 * side hole, inside the wall, nor the front mortise's leg flat in the
 * groove, behind the leg up the wall; the near ones, cut away with the near
 * half of the key, are not drawn. The **solid** sweeps each extruded part
 * across the keyboard, each lining on its side of its mortise, lathes each
 * turned part and the front rail pin's oval head,
 * squeezed across, and carves each rectangular cut that has a width out of
 * the wood it runs through, a milled one, the two mortises and the balance hole, with
 * its ends rounded to its cutter's radius: it leaves that wood unpainted
 * inside it and draws its walls from inside, in the color of the wood they
 * are cut in (`carving.ts`).
 *
 * The faded parts, a key drawn faint as the reader chose or one no
 * equilibrium places, are drawn apart in the solid, whatever order they
 * come in (`transparency.ts`): every face of them stays drawn, the
 * mortises' walls included, and no angle changes their colors. Every part
 * of the solid compiles through one compile hook under one cache key,
 * carved and drawn apart alike
 * (`shading.ts`).
 *
 * Over both views, the contacts the engine's rows find within a millimeter
 * of closing are marked where the rows put them, in the keybed's frame: a
 * dot where the key bears, a line across a gap still open, and another where
 * drawn parts overlap.
 *
 * Marked by provenance, every line is drawn in the color of how the number
 * that places it is known, measured, published, traced from a figure,
 * derived by a rule, borrowed from another instrument, assumed, or set on
 * the bench since the instrument was built: the engine sends, with each
 * stretch of an outline, the number that places it and how its profile knows
 * it, and the view only paints it, in colors chosen apart from every fill
 * they are drawn over (`palette.ts`).
 *
 * Colors come from what a part is made of, never from what it is: the
 * keytop and the key front are white, the key button and the back check block
 * one wood and the keystick another, a paper the color its thickness is sold
 * in. The section paints them as they are, with no tone mapping, so white is
 * white there; the solid is lit, and tone-mapped so the light does not burn
 * its faces out.
 *
 * # Library traps met here
 *
 * drei's `Line` rendered nothing while a plain mesh in the same scene rendered
 * fine, so borders are plain `lineSegments`. drei's `Text` fetches a font over
 * the network and suspends until it arrives, which blanks the canvas on a bench
 * with no route out, so every label is in HTML.
 *
 * And from R3F itself: the `camera` prop is read **once, at creation**.
 * Changing it later does not move the camera, which is why the two viewpoints
 * re-key `OrbitControls` on the mode instead of animating one camera between
 * them.
 */

import type { How, MarkedContact, Marker, Outline, Pose, Scene, State } from '@pas/wasm/types'
import { OrbitControls } from '@react-three/drei'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { createContext, useContext, useEffect, useLayoutEffect, useMemo } from 'react'
import * as THREE from 'three'
import { type Box, measureBounds } from './bounds'
import {
  buildCarvingUniforms,
  type Carving,
  type Hollow,
  millWalls,
  planCarving,
  type Walls,
  writeCarving,
} from './carving'
import { bodyOpacity } from './opacity'
import {
  axisSegments,
  contactStrokes,
  mirrored,
  poseMatrix,
  sharedSegments,
  strokes,
  toShape,
} from './outline'
import { turnOval } from './oval'
import {
  CONTACT_COLOR,
  EDGE,
  HOW_COLOR,
  KEYBED,
  LIGHTS,
  MATERIAL_FILL,
  MEASURING_POINT_COLOR,
  PAGE,
  PAPER_FILL,
  UNGRADED_FILL,
} from './palette'
import { divideTheSection, type Section } from './section'
import { shadeSolidPart } from './shading'
import { EDGE_OF_A_WHOLE_PART, FadedDrawing, PASS, type PassUniform } from './transparency'
import { VIEWPOINT, type ViewMode } from './viewpoint'

/** Each segment of an outline by how the number that places it is known. */
function howOf(o: Outline): Map<number, How> {
  const by = new Map<number, How>()
  for (const s of o.sources) for (let k = s.from; k < s.to; k += 1) by.set(k, s.how)
  return by
}

/**
 * An outline's edges as line pieces, one set per way of knowing the number
 * that places them, the segments in `skip` left out.
 */
function piecesByHow(o: Outline, skip: ReadonlySet<number>): Array<{ how: How; pieces: number[] }> {
  const by = howOf(o)
  const hows = [...new Set(by.values())]
  return hows.map((how) => {
    const other = new Set(skip)
    for (const seg of o.segs) if (by.get(seg) !== how) other.add(seg)
    return { how, pieces: strokes(o.points_mm, o.segs, other) }
  })
}

/** The color a part is drawn in: a paper's as it is supplied, anything
 * else's from its material. */
function colorOf(o: Outline): string {
  if (o.material === 'paper') return o.paper_color ? PAPER_FILL[o.paper_color] : UNGRADED_FILL
  return MATERIAL_FILL[o.material] ?? '#9b9b9b'
}

/**
 * The order the section is painted in, back to front. The section is flat, so
 * nothing is depth-tested in it: the ground's parts first, then the key's
 * cuts into the stencil, the key's wood around them and its edges, then the
 * capstan over the wood it is screwed into, then what is marked on the key.
 */
const LAYER = {
  // Seen beyond the plane, through a void: under everything in it.
  beyond: 0,
  ground: 1,
  groundEdges: 2,
  cut: 3,
  key: 4,
  keyEdges: 5,
  turned: 6,
  turnedEdges: 7,
  marks: 8,
  contacts: 9,
} as const

/** The engine's ground, which carries the frame: body 0. */
const GROUND = 0

/** The stencil value a cut leaves where the key's wood must not be painted. */
const CUT_STENCIL = 1

/** How opaque the parts of the body being drawn are: wholly, unless the
 * body is drawn faded. */
const Opacity = createContext(1)

/** A material's transparency at `opacity`: a part drawn faded writes no
 * depth, so that what stands behind it shows through. */
function fadeTo(opacity: number) {
  return { transparent: opacity < 1, opacity, depthWrite: opacity === 1 }
}

/** The key of a material at `opacity`, so that three builds it again when it
 * turns transparent or opaque, which a changed property alone does not. */
function fadeKey(opacity: number): string {
  return opacity < 1 ? 'faded' : 'opaque'
}

/** The uniform the solid's parts read the pass they are drawn in from, one
 * for the view ([`FadedDrawing`]). */
const FadedPass = createContext<PassUniform | null>(null)

/** Draws the view in place of react-three-fiber, its faded parts apart,
 * whatever order they come in ([`FadedDrawing`]). */
function DrawFadedApart({ pass }: { pass: PassUniform }) {
  const renderer = useThree((state) => state.gl)
  const drawing = useMemo(() => new FadedDrawing(renderer), [renderer])
  useEffect(() => () => drawing.dispose(), [drawing])
  // A positive priority takes the drawing over from react-three-fiber.
  useFrame(({ scene, camera }) => drawing.draw(scene, camera, pass), 1)
  return null
}

/**
 * Disposes a geometry when it is replaced or unmounted.
 *
 * A geometry handed to a mesh through the `geometry` prop is not an instance
 * in the tree, so react-three-fiber never disposes it. Every edit of the
 * geometry rebuilds the scene from a fresh `JSON.parse`, so without this a
 * slider drag leaves a thousand orphaned buffers on the GPU — in an
 * application meant to stay open while a technician works.
 */
function useDisposal(geometry: THREE.BufferGeometry) {
  useEffect(() => () => geometry.dispose(), [geometry])
}

// ---- the solid ------------------------------------------------------------

/** No cut through a part. */
const UNCARVED: readonly Hollow[] = []

/** How many faces a round part is drawn with around its axis. */
const ROUND_SEGMENTS = 48

/** A body with no part: nothing to draw, carve or cut. */
const NO_BODY: BodyParts = {
  outlines: [],
  carving: { cutsThrough: new Map(), walls: [] },
  section: { cuts: [], inPlane: [], beyond: [] },
}

/** An outline swept across the keyboard, its middle where the engine puts
 * it: the sagittal plane for all but the parts beside it. */
function extrude(points: number[], width: number, across: number): THREE.BufferGeometry {
  const extrusion = new THREE.ExtrudeGeometry(toShape(points), {
    depth: width,
    bevelEnabled: false,
  })
  extrusion.translate(0, 0, across - width / 2)
  extrusion.computeVertexNormals()
  return extrusion
}

/** [`extrude`], built again only when its outline, width or place across
 * changes. */
function useExtrusion(points: number[], width: number, across: number) {
  const geometry = useMemo(() => extrude(points, width, across), [points, width, across])
  useDisposal(geometry)
  return geometry
}

/** The surface a cut's walls are drawn on: its outline swept across its
 * width, or, milled, its plan swept up its height. */
function useWallsSurface({ cut, hollow }: Walls) {
  const geometry = useMemo(
    () =>
      hollow.hasRoundEnds
        ? millWalls(hollow.box, ROUND_SEGMENTS)
        : extrude(cut.points_mm, cut.width_mm, cut.across_mm),
    [cut, hollow],
  )
  useDisposal(geometry)
  return geometry
}

/** The edges of a solid, faint, where its faces meet at an angle: drawn
 * with their part where it is drawn whole, so that a faded part in front
 * dims them as it dims the part ([`FadedDrawing`]). */
function SolidEdges({ geometry }: { geometry: THREE.BufferGeometry }) {
  const edges = useMemo(() => new THREE.EdgesGeometry(geometry, 25), [geometry])
  const opacity = useContext(Opacity)
  const userData = useMemo(() => ({ [EDGE_OF_A_WHOLE_PART]: opacity === 1 }), [opacity])
  useDisposal(edges)
  return (
    <lineSegments geometry={edges}>
      <lineBasicMaterial color="#3a342b" transparent opacity={0.4 * opacity} userData={userData} />
    </lineSegments>
  )
}

/**
 * The Lambert material of every part of the solid, faded with its body:
 * carved by `cuts` and kept within `keep`, where either is given, and drawn
 * apart where it is faded, all through the one compile hook under one cache
 * key that every part shares ([`shadeSolidPart`]), so that a material made
 * anew as it fades or turns whole compiles as the one it replaces. Its cuts
 * are uniforms, written as they change; the view is drawn on demand, so it
 * then asks for a frame.
 */
function SolidMaterial({
  color,
  cuts,
  keep,
  side,
}: {
  color: string
  cuts: readonly Hollow[]
  keep: Box | null
  side: THREE.Side
}) {
  const opacity = useContext(Opacity)
  const pass = useContext(FadedPass)
  const invalidate = useThree((state) => state.invalidate)
  const carving = useMemo(buildCarvingUniforms, [])
  const { material, packed } = useMemo(
    () => shadeSolidPart({ cuts, keep, pass }, carving),
    [cuts, keep, pass, carving],
  )
  useLayoutEffect(() => {
    writeCarving(carving, packed)
    invalidate()
  }, [carving, packed, invalidate])
  return (
    <meshLambertMaterial
      key={fadeKey(opacity)}
      color={color}
      side={side}
      {...material}
      {...fadeTo(opacity)}
    />
  )
}

function Extruded({
  points,
  width,
  across,
  color,
  cuts,
}: {
  points: number[]
  width: number
  /** Where its middle stands across the keyboard, millimeters toward the
   * bass. */
  across: number
  color: string
  /** The cuts through it, which leave it unpainted inside them. */
  cuts: readonly Hollow[]
}) {
  const geometry = useExtrusion(points, width, across)
  return (
    <group>
      <mesh geometry={geometry}>
        <SolidMaterial color={color} cuts={cuts} keep={null} side={THREE.FrontSide} />
      </mesh>
      <SolidEdges geometry={geometry} />
    </group>
  )
}

/**
 * The walls of a cut through a body's wood, seen from inside it: its box,
 * or a milled cut's rounded plan swept up its height, drawn from within, in each part it runs
 * through, in that part's color and only where it runs through that part,
 * and open where it opens into another cut, as the slot's floor into the
 * balance hole; and its edges.
 */
function CutWalls({ walls }: { walls: Walls }) {
  const { through, into } = walls
  const geometry = useWallsSurface(walls)
  const keeps = useMemo(() => through.map(measureBounds), [through])
  return (
    <group>
      {through.map((part, partIndex) => (
        <mesh key={`${part.code}-${partIndex}`} geometry={geometry}>
          <SolidMaterial
            color={colorOf(part)}
            cuts={into}
            keep={keeps[partIndex] ?? null}
            side={THREE.BackSide}
          />
        </mesh>
      ))}
      <SolidEdges geometry={geometry} />
    </group>
  )
}

/** An oval part, the front rail pin's head: its section along the key
 * turned about its middle and squeezed across to its width, its tip rounded
 * as the section rounds it, and its edges. */
function Oval({ outline }: { outline: Outline }) {
  const geometry = useMemo(
    () => turnOval(outline.points_mm, outline.width_mm, ROUND_SEGMENTS),
    [outline],
  )
  useDisposal(geometry)
  return (
    <group>
      <mesh geometry={geometry}>
        <SolidMaterial
          color={colorOf(outline)}
          cuts={UNCARVED}
          keep={null}
          side={THREE.FrontSide}
        />
      </mesh>
      <SolidEdges geometry={geometry} />
    </group>
  )
}

/**
 * A turned part: the profile lathed about its own vertical axis.
 *
 * A capstan is a screw, and extruding its silhouette across the 22 mm of the
 * key draws a block of wood. A punching is a **washer**, and extruding it
 * draws a strip of felt the length of the rail. A balance pin is round. A
 * profile whose first radius is not zero leaves the hole its pin passes
 * through, the feature that says the key is located and not merely resting.
 */
function Lathed({ profile, axisX, color }: { profile: number[]; axisX: number; color: string }) {
  const geometry = useMemo(() => {
    const points: THREE.Vector2[] = []
    for (let i = 0; i < profile.length; i += 2) {
      points.push(new THREE.Vector2(profile[i] as number, profile[i + 1] as number))
    }
    // Close the profile so the lathe wraps a solid with both faces — a washer
    // rather than an open tube.
    const first = points[0]
    if (first) points.push(first.clone())
    return new THREE.LatheGeometry(points, ROUND_SEGMENTS)
  }, [profile])
  useDisposal(geometry)
  return (
    <mesh geometry={geometry} position={[axisX, 0, 0]}>
      <SolidMaterial color={color} cuts={UNCARVED} keep={null} side={THREE.DoubleSide} />
    </mesh>
  )
}

/**
 * One part in the solid: an extruded part swept across the keyboard, carved
 * by `cuts`, its lines marked by provenance on the face toward the camera
 * where `marked`; an oval one turned and squeezed and a turned one lathed,
 * which have no flat face for their marks, the section marking them. A
 * milled part is a cut, which the solid carves out of the wood and never
 * draws.
 */
function SolidPart({
  outline,
  cuts,
  marked,
}: {
  outline: Outline
  cuts: readonly Hollow[]
  marked: boolean
}) {
  switch (outline.sweep) {
    case 'extruded':
      return (
        <group>
          <Extruded
            points={outline.points_mm}
            width={outline.width_mm}
            across={outline.across_mm}
            color={colorOf(outline)}
            cuts={cuts}
          />
          {marked && (
            <OutlineEdges
              outline={outline}
              skip={new Set()}
              layer={LAYER.marks}
              marked
              z={outline.across_mm + outline.width_mm / 2 + 0.05}
              depthTest
            />
          )}
        </group>
      )
    case 'oval':
      return <Oval outline={outline} />
    case 'revolved':
      return (
        <Lathed profile={outline.points_mm} axisX={outline.axis_x_mm} color={colorOf(outline)} />
      )
    case 'milled':
      return null
  }
}

// ---- the section ----------------------------------------------------------

/** A flat fill in the sagittal plane, painted in its layer. */
function Fill({
  points,
  color,
  layer,
  masked = false,
}: {
  points: number[]
  color: string
  layer: number
  /** Whether a cut keeps it from being painted: the key's own parts are. */
  masked?: boolean
}) {
  const geometry = useMemo(() => new THREE.ShapeGeometry(toShape(points)), [points])
  const opacity = useContext(Opacity)
  useDisposal(geometry)
  return (
    <mesh geometry={geometry} renderOrder={layer}>
      <meshBasicMaterial
        key={fadeKey(opacity)}
        {...fadeTo(opacity)}
        color={color}
        side={THREE.DoubleSide}
        depthTest={false}
        depthWrite={false}
        toneMapped={false}
        stencilWrite={masked}
        stencilRef={CUT_STENCIL}
        stencilFunc={THREE.NotEqualStencilFunc}
        stencilFail={THREE.KeepStencilOp}
        stencilZFail={THREE.KeepStencilOp}
        stencilZPass={THREE.KeepStencilOp}
      />
    </mesh>
  )
}

/** A cut, written into the stencil buffer and nowhere else. */
function Void({ points }: { points: number[] }) {
  const geometry = useMemo(() => new THREE.ShapeGeometry(toShape(points)), [points])
  useDisposal(geometry)
  return (
    <mesh geometry={geometry} renderOrder={LAYER.cut}>
      <meshBasicMaterial
        colorWrite={false}
        side={THREE.DoubleSide}
        depthTest={false}
        depthWrite={false}
        stencilWrite
        stencilRef={CUT_STENCIL}
        stencilFunc={THREE.AlwaysStencilFunc}
        stencilZPass={THREE.ReplaceStencilOp}
      />
    </mesh>
  )
}

/** Edges, as flat `[x0, y0, x1, y1, …]` pieces. */
function Edges({
  pieces,
  layer,
  color = EDGE,
  z = 0,
  depthTest = false,
  overlay = false,
  masked = false,
}: {
  pieces: number[]
  layer: number
  color?: string
  /** How far across the sagittal plane the edges stand: on a solid's face. */
  z?: number
  /** Whether the solids in front hide them, as on a solid's face. */
  depthTest?: boolean
  /** Whether they are marks drawn over every part, a faded key's included. */
  overlay?: boolean
  /** Whether a cut keeps them from being drawn: the key's own parts' are. */
  masked?: boolean
}) {
  const geometry = useMemo(() => {
    const g = new THREE.BufferGeometry()
    const xyz: number[] = []
    for (let i = 0; i < pieces.length; i += 2) {
      xyz.push(pieces[i] as number, pieces[i + 1] as number, z)
    }
    g.setAttribute('position', new THREE.Float32BufferAttribute(xyz, 3))
    return g
  }, [pieces, z])
  const opacity = useContext(Opacity)
  useDisposal(geometry)
  return (
    <lineSegments geometry={geometry} renderOrder={layer}>
      <lineBasicMaterial
        key={fadeKey(opacity)}
        {...fadeTo(opacity)}
        transparent={overlay || opacity < 1}
        color={color}
        depthTest={depthTest}
        depthWrite={false}
        toneMapped={false}
        stencilWrite={masked}
        stencilRef={CUT_STENCIL}
        stencilFunc={THREE.NotEqualStencilFunc}
        stencilFail={THREE.KeepStencilOp}
        stencilZFail={THREE.KeepStencilOp}
        stencilZPass={THREE.KeepStencilOp}
      />
    </lineSegments>
  )
}

/** An outline's edges, the segments in `skip` left out: in the edge color,
 * or, marked by provenance, each in the color of how its number is known. */
function OutlineEdges({
  outline,
  skip,
  layer,
  marked,
  mirror = false,
  z = 0,
  depthTest = false,
  masked = false,
}: {
  outline: Outline
  skip: ReadonlySet<number>
  layer: number
  marked: boolean
  mirror?: boolean
  z?: number
  depthTest?: boolean
  masked?: boolean
}) {
  const sets = marked
    ? piecesByHow(outline, skip).map(({ how, pieces }) => ({ color: HOW_COLOR[how], pieces }))
    : [{ color: EDGE, pieces: strokes(outline.points_mm, outline.segs, skip) }]
  return (
    <>
      {sets.map(({ color, pieces }) => (
        <Edges
          key={color}
          pieces={mirror ? mirrored(pieces) : pieces}
          layer={layer}
          color={color}
          z={z}
          depthTest={depthTest}
          masked={masked}
        />
      ))}
    </>
  )
}

/**
 * One part, cut by the sagittal plane: an extruded part's outline; a turned
 * part's profile and its mirror about the axis, neither edge drawn down the
 * axis where they meet.
 */
function SectionPart({
  outline,
  layer,
  edgeLayer,
  masked,
  open,
  marked,
}: {
  outline: Outline
  layer: number
  edgeLayer: number
  masked: boolean
  /** Segments that are no edge: where the part runs along a cut's mouth. */
  open: ReadonlySet<number>
  /** Whether its lines are marked by provenance. */
  marked: boolean
}) {
  const color = colorOf(outline)
  switch (outline.sweep) {
    // A milled or an oval part's outline is its section through its
    // middle, as drawn.
    case 'extruded':
    case 'milled':
    case 'oval':
      return (
        <>
          <Fill points={outline.points_mm} color={color} layer={layer} masked={masked} />
          <OutlineEdges
            outline={outline}
            skip={open}
            layer={edgeLayer}
            marked={marked}
            masked={masked}
          />
        </>
      )
    case 'revolved': {
      const onAxis = axisSegments(outline.points_mm, outline.segs)
      return (
        <group position={[outline.axis_x_mm, 0, 0]}>
          <Fill points={outline.points_mm} color={color} layer={layer} masked={masked} />
          <Fill points={mirrored(outline.points_mm)} color={color} layer={layer} masked={masked} />
          <OutlineEdges
            outline={outline}
            skip={onAxis}
            layer={edgeLayer}
            marked={marked}
            masked={masked}
          />
          <OutlineEdges
            outline={outline}
            skip={onAxis}
            layer={edgeLayer}
            marked={marked}
            masked={masked}
            mirror
          />
        </group>
      )
    }
  }
}

// ---- both -----------------------------------------------------------------

/**
 * The measuring point: a marker at the datum every touchweight is read at.
 *
 * Marked on purpose. A weight is only a measurement if you can see where it was
 * taken, and a set distance back from the covering's lip is not somewhere a
 * reader can guess, least of all once the distance has been changed. The
 * engine places it on the key, so it follows the setting and the key both.
 */
function MeasuringPoint({ at }: { at: [number, number] }) {
  const [x, y] = at
  return (
    <mesh position={[x, y + 3.5, 0]} rotation={[Math.PI, 0, 0]} renderOrder={LAYER.marks}>
      <coneGeometry args={[2.2, 6, 4]} />
      <meshBasicMaterial
        color={MEASURING_POINT_COLOR}
        depthTest={false}
        toneMapped={false}
        transparent
      />
    </mesh>
  )
}

/**
 * The contacts within a millimeter of closing: a dot where the frame's
 * surface is, and across an open or overlapping contact a line to where the
 * key's is. Drawn in the keybed's frame, over everything, in both views: the
 * points are where the engine's contact rows put them.
 *
 * Their materials are transparent, though wholly opaque, as the measuring
 * point's is: three draws every opaque material before every transparent
 * one, whatever their render order, so marks left opaque would be drawn
 * under a faded key.
 */
function Contacts({ contacts }: { contacts: MarkedContact[] }) {
  return (
    <>
      {(['open', 'overlapping'] as const).map((state) => (
        <Edges
          key={state}
          pieces={contactStrokes(contacts, state)}
          layer={LAYER.contacts}
          color={CONTACT_COLOR[state]}
          overlay
        />
      ))}
      {contacts.map((c, i) => (
        <mesh
          key={i}
          position={[c.on_frame_mm[0], c.on_frame_mm[1], 0]}
          renderOrder={LAYER.contacts}
        >
          <sphereGeometry args={[0.9, 16, 12]} />
          <meshBasicMaterial
            color={CONTACT_COLOR[c.state]}
            depthTest={false}
            depthWrite={false}
            toneMapped={false}
            transparent
          />
        </mesh>
      ))}
    </>
  )
}

/** The keybed: the datum every height is measured from. */
function Keybed() {
  return (
    <mesh position={[30, -3, 0]}>
      <boxGeometry args={[560, 6, 86]} />
      <meshLambertMaterial color={KEYBED} />
    </mesh>
  )
}

/** One body's parts, what the solid carves in them and what the section
 * draws of them: the same for every pose of a scene. */
type BodyParts = {
  readonly outlines: readonly Outline[]
  readonly carving: Carving
  readonly section: Section
}

/**
 * One body: its parts in its own frame, carried by its pose. The group's
 * matrix is the pose and nothing else; three.js is told not to rebuild it from
 * a position and a rotation.
 */
function Body({
  body,
  pose,
  parts: { outlines, carving, section },
  mode,
  marker,
  marked,
  opacity,
}: {
  body: number
  pose: Pose
  parts: BodyParts
  mode: ViewMode
  marker: Marker | undefined
  /** Whether its lines are marked by provenance. */
  marked: boolean
  /** How opaque its parts are drawn. */
  opacity: number
}) {
  const matrix = useMemo(() => poseMatrix(pose), [pose])
  const parts = useMemo(() => outlines.filter((outline) => outline.role !== 'cut'), [outlines])
  const { cuts, inPlane, beyond } = section
  const grounded = body === GROUND
  return (
    <group matrix={matrix} matrixAutoUpdate={false}>
      <Opacity.Provider value={opacity}>
        {mode === 'solid' ? (
          <>
            {parts.map((part, partIndex) => (
              <SolidPart
                key={`${part.code}-${partIndex}`}
                outline={part}
                cuts={carving.cutsThrough.get(part) ?? UNCARVED}
                marked={marked}
              />
            ))}
            {carving.walls.map((walls, cutIndex) => (
              <CutWalls key={`${walls.cut.code}-${cutIndex}`} walls={walls} />
            ))}
          </>
        ) : (
          <>
            {beyond.map((o, i) => (
              <group key={`${o.code}-beyond-${i}`}>
                <Fill points={o.points_mm} color={colorOf(o)} layer={LAYER.beyond} />
                <OutlineEdges outline={o} skip={new Set()} layer={LAYER.beyond} marked={marked} />
              </group>
            ))}
            {cuts.map((o, i) => (
              <Void key={`${o.code}-${i}`} points={o.points_mm} />
            ))}
            {cuts.map((o, i) => {
              // A cut's mouth runs along its body's outline, or opens into
              // another cut: no edge there.
              const mouth = new Set<number>()
              for (const p of [...parts, ...cuts.filter((c) => c !== o)]) {
                for (const s of sharedSegments(o, p)) mouth.add(s)
              }
              return (
                <OutlineEdges
                  key={`${o.code}-edges-${i}`}
                  outline={o}
                  skip={mouth}
                  layer={LAYER.keyEdges}
                  marked={marked}
                />
              )
            })}
            {inPlane.map((o, i) => {
              const open = new Set<number>()
              for (const c of cuts) for (const s of sharedSegments(o, c)) open.add(s)
              const turned = o.sweep === 'revolved'
              const [layer, edgeLayer] = grounded
                ? [LAYER.ground, LAYER.groundEdges]
                : turned
                  ? [LAYER.turned, LAYER.turnedEdges]
                  : [LAYER.key, LAYER.keyEdges]
              return (
                <SectionPart
                  key={`${o.code}-${i}`}
                  outline={o}
                  layer={layer}
                  edgeLayer={edgeLayer}
                  masked={!grounded && cuts.length > 0}
                  open={open}
                  marked={marked}
                />
              )
            })}
          </>
        )}
      </Opacity.Provider>
      {marker && <MeasuringPoint at={marker.at_mm} />}
    </group>
  )
}

export function SagittalView({
  scene,
  state,
  mode,
  marked,
  keyOpacity,
}: {
  /** What is drawn: every part in its own body's frame. */
  scene: Scene
  /** Where each body stands, and the contacts to mark. */
  state: State
  mode: ViewMode
  /** Whether every line is marked by how its number is known. */
  marked: boolean
  /** How opaque the key is drawn, as the reader chose: faded further where
   * no equilibrium places it. */
  keyOpacity: number
}) {
  const preset = VIEWPOINT[mode]
  const pass = useMemo((): PassUniform => ({ value: PASS.blended }), [])
  // Sorted once a scene, so that a weight on the key, which moves only the
  // poses, leaves every part's cuts as they were.
  const bodies = useMemo(
    () =>
      state.poses.map((_, body): BodyParts => {
        const outlines = scene.outlines.filter((outline) => outline.body === body)
        return { outlines, carving: planCarving(outlines), section: divideTheSection(outlines) }
      }),
    [scene, state.poses.length],
  )

  return (
    <Canvas
      // Orthographic in both modes: this is a measuring instrument, and
      // perspective would make two equal dimensions measure differently
      // depending on where they sit.
      orthographic
      camera={{ position: preset.position, zoom: preset.zoom }}
      // Drawn when something changes, a prop, the camera or the canvas's
      // size: a still view costs nothing.
      frameloop="demand"
      // The stencil buffer is what lets a mortise be a void in the key's wood
      // without being painted over the pin standing in it.
      gl={{ stencil: true }}
      style={{ background: PAGE }}
    >
      <DrawFadedApart pass={pass} />
      <ambientLight intensity={LIGHTS.ambient} />
      {LIGHTS.directional.map(({ position, intensity }) => (
        <directionalLight key={position.join()} position={position} intensity={intensity} />
      ))}

      {/* Mirrored in x so the player's end of the key is on the LEFT, which is
          how the trade's own sectional plates are drawn. A view convention and
          nothing more: the engine's x still runs toward the player, and no
          number changes. */}
      <group scale={[-1, 1, 1]}>
        <FadedPass.Provider value={pass}>
          <Keybed />
          {state.poses.map((pose, body) => (
            <Body
              key={body}
              body={body}
              pose={pose}
              parts={bodies[body] ?? NO_BODY}
              mode={mode}
              marker={state.measuring_point.body === body ? state.measuring_point : undefined}
              marked={marked}
              opacity={bodyOpacity({
                grounded: body === GROUND,
                chosen: keyOpacity,
                isPlaced: state.place === 'placed',
              })}
            />
          ))}
          <Contacts contacts={state.contacts} />
        </FadedPass.Provider>
      </group>

      {/* Re-keyed on the mode so the two buttons are two viewpoints, not two
          starting points you then have to re-aim. */}
      <OrbitControls key={mode} makeDefault target={preset.target} enableDamping={false} />
    </Canvas>
  )
}
