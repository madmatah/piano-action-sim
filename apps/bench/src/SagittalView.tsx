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
 * or lathes it when the part is turned. A separately modeled mesh would be a
 * second geometry, and the moment it disagreed with the first the displayed
 * dimensions would stop describing what is measured. So the section is not a
 * reduced view of the solid: the solid *is* the section, swept.
 *
 * # Two views
 *
 * The **section** is the sagittal plane itself, drawn flat: each part's
 * outline, a turned part's profile beside its mirror, so a washer shows the
 * hole its pin stands in. A cut, a mortise or the balance hole, is a void in
 * its body's solids: drawn into the stencil buffer first, it keeps the key's
 * wood, and the wood's edges, from being painted there, so the pin in it
 * shows through, while a pin standing inside the key's wood stays hidden by
 * it, unless the key is drawn faded, as the reader may choose and as a key no
 * equilibrium places is. A cut's mouth, where it opens through the key's
 * outline or into another cut, is no edge of the section. A lining set into
 * one side of a slot, the balance mortise's bushing, stands out of the
 * plane: the one on the far side is drawn as seen beyond the plane, through
 * the void and behind the pin, and the near one, cut away with the near
 * half of the key, is not drawn. The **solid** sweeps every part across the
 * keyboard, each lining on its side of the slot, and leaves the cuts out:
 * they are not carved from the key's wood.
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
import { Canvas } from '@react-three/fiber'
import { createContext, useContext, useEffect, useMemo } from 'react'
import * as THREE from 'three'
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
import {
  CONTACT_COLOR,
  EDGE,
  HOW_COLOR,
  KEYBED,
  LIGHTS,
  MATERIAL_FILL,
  PAGE,
  PAPER_FILL,
  UNGRADED_FILL,
} from './palette'

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

/** Two viewpoints: the technical section, and the solid in three-quarters. */
export type ViewMode = 'section' | 'solid'

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

function Extruded({
  points,
  width,
  across,
  color,
}: {
  points: number[]
  width: number
  /** Where its middle stands across the keyboard, millimeters toward the
   * bass. */
  across: number
  color: string
}) {
  const geometry = useMemo(() => {
    const g = new THREE.ExtrudeGeometry(toShape(points), { depth: width, bevelEnabled: false })
    // Extrusion runs along +z from the outline's plane; center it on where
    // the engine puts its middle, the sagittal plane for all but a lining.
    g.translate(0, 0, across - width / 2)
    g.computeVertexNormals()
    return g
  }, [points, width, across])
  const edges = useMemo(() => new THREE.EdgesGeometry(geometry, 25), [geometry])
  const opacity = useContext(Opacity)
  useDisposal(geometry)
  useDisposal(edges)
  return (
    <group>
      <mesh geometry={geometry}>
        <meshLambertMaterial key={fadeKey(opacity)} color={color} {...fadeTo(opacity)} />
      </mesh>
      <lineSegments geometry={edges}>
        <lineBasicMaterial color="#3a342b" transparent opacity={0.4 * opacity} />
      </lineSegments>
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
    return new THREE.LatheGeometry(points, 48)
  }, [profile])
  const opacity = useContext(Opacity)
  useDisposal(geometry)
  return (
    <mesh geometry={geometry} position={[axisX, 0, 0]}>
      <meshLambertMaterial
        key={fadeKey(opacity)}
        color={color}
        side={THREE.DoubleSide}
        {...fadeTo(opacity)}
      />
    </mesh>
  )
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
  if (outline.sweep === 'extruded') {
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
  }
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
      <meshBasicMaterial color="#c8302a" depthTest={false} toneMapped={false} transparent />
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

/**
 * One body: its parts in its own frame, carried by its pose. The group's
 * matrix is the pose and nothing else; three.js is told not to rebuild it from
 * a position and a rotation.
 */
function Body({
  body,
  pose,
  outlines,
  mode,
  marker,
  marked,
  opacity,
}: {
  body: number
  pose: Pose
  outlines: Outline[]
  mode: ViewMode
  marker: Marker | undefined
  /** Whether its lines are marked by provenance. */
  marked: boolean
  /** How opaque its parts are drawn. */
  opacity: number
}) {
  const matrix = useMemo(() => poseMatrix(pose), [pose])
  const cuts = outlines.filter((o) => o.role === 'cut')
  const parts = outlines.filter((o) => o.role !== 'cut')
  const inPlane = parts.filter((o) => o.is_in_the_section)
  // The camera looks at the section from +z, the bass side: what stands at
  // negative z, out of the plane, is beyond it.
  const beyond = parts.filter((o) => !o.is_in_the_section && o.across_mm < 0)
  const grounded = body === GROUND
  return (
    <group matrix={matrix} matrixAutoUpdate={false}>
      <Opacity.Provider value={opacity}>
        {mode === 'solid' ? (
          parts.map((o, i) =>
            o.sweep === 'revolved' ? (
              <Lathed
                key={`${o.code}-${i}`}
                profile={o.points_mm}
                axisX={o.axis_x_mm}
                color={colorOf(o)}
              />
            ) : (
              <group key={`${o.code}-${i}`}>
                <Extruded
                  points={o.points_mm}
                  width={o.width_mm}
                  across={o.across_mm}
                  color={colorOf(o)}
                />
                {/* Marked by provenance, on the face toward the camera. */}
                {marked && (
                  <OutlineEdges
                    outline={o}
                    skip={new Set()}
                    layer={LAYER.marks}
                    marked
                    z={o.across_mm + o.width_mm / 2 + 0.05}
                    depthTest
                  />
                )}
              </group>
            ),
          )
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

const CAMERA: Record<
  ViewMode,
  { position: [number, number, number]; target: [number, number, number]; zoom: number }
> = {
  // Straight on: a technical section, read for its dimensions.
  section: { position: [-30, 24, 420], target: [-30, 24, 0], zoom: 2.0 },
  // Three-quarters, and **low**: the solid, read for how it is stacked.
  // Looking down from 25° shows nothing but the top of the key — the rails,
  // the punchings and the cloth all hide under it. What a reader needs to see
  // here is the stack, so the elevation is about 12°.
  solid: { position: [-190, 115, 430], target: [-30, 20, 0], zoom: 1.75 },
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
  const preset = CAMERA[mode]

  return (
    <Canvas
      // Orthographic in both modes: this is a measuring instrument, and
      // perspective would make two equal dimensions measure differently
      // depending on where they sit.
      orthographic
      camera={{ position: preset.position, zoom: preset.zoom }}
      // The stencil buffer is what lets a mortise be a void in the key's wood
      // without being painted over the pin standing in it.
      gl={{ stencil: true }}
      style={{ background: PAGE }}
    >
      <ambientLight intensity={LIGHTS.ambient} />
      {LIGHTS.directional.map(({ position, intensity }) => (
        <directionalLight key={position.join()} position={position} intensity={intensity} />
      ))}

      {/* Mirrored in x so the player's end of the key is on the LEFT, which is
          how the trade's own sectional plates are drawn. A view convention and
          nothing more: the engine's x still runs toward the player, and no
          number changes. */}
      <group scale={[-1, 1, 1]}>
        <Keybed />
        {state.poses.map((pose, body) => (
          <Body
            key={body}
            body={body}
            pose={pose}
            outlines={scene.outlines.filter((o) => o.body === body)}
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
      </group>

      {/* Re-keyed on the mode so the two buttons are two viewpoints, not two
          starting points you then have to re-aim. */}
      <OrbitControls key={mode} makeDefault target={preset.target} enableDamping={false} />
    </Canvas>
  )
}
