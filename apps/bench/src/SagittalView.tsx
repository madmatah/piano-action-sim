/**
 * The bench's 3-D view.
 *
 * Built in millimetres: `x` towards the player, `y` up, `z` across the
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
 * or lathes it when the part is turned. A separately modelled mesh would be a
 * second geometry, and the moment it disagreed with the first the displayed
 * dimensions would stop describing what is measured. So the section is not a
 * reduced view of the solid: the solid *is* the section, swept.
 *
 * # Two views
 *
 * The **section** is the sagittal plane itself, drawn flat: each part's
 * outline, a turned part's profile beside its mirror, so a washer shows the
 * hole its pin stands in. A cut, the front mortise, is a void in its body's
 * solids: drawn into the stencil buffer first, it keeps the key's wood from
 * being painted there, so the pin under it shows through, while a pin standing
 * inside the key's wood stays hidden by it. Its mouth, which runs along the
 * keystick's underside, is no edge of the section. The **solid** sweeps every
 * part across the keyboard, and leaves the cuts out: a mortise is inside the
 * key.
 *
 * Over both views, the contacts the engine's rows find within a millimetre
 * of closing are marked where the rows put them, in the keybed's frame: a
 * dot where the key bears, a line across a gap still open, and another where
 * drawn parts overlap.
 *
 * Colours come from what a part is made of, never from what it is: the
 * keytop and the key front are white, the key button and the back check block
 * one wood and the keystick another, a paper the colour its thickness is sold
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
import { OrbitControls } from '@react-three/drei'
import { Canvas } from '@react-three/fiber'
import { useEffect, useMemo } from 'react'
import * as THREE from 'three'
import type { Contact, Lead, Marker, Outline, Pose, Scene } from './bench'
import {
  axisSegments,
  contactStrokes,
  mirrored,
  poseMatrix,
  sharedSegments,
  strokes,
  toShape,
} from './outline'

/**
 * Each material's colour: what a part is made of is the engine's, how it
 * looks is this view's.
 */
const MATERIAL_FILL: Record<string, string> = {
  key_wood: '#ecdcb6',
  // The key button and the back check block: a wood other than the
  // keystick's, as the owner described them.
  block_wood: '#a8713f',
  // The keytop and the key front, white.
  covering: '#ffffff',
  key_front: '#ffffff',
  rail_wood: '#c6a477',
  steel: '#a9b1ba',
  punching_felt: '#c8655d',
  hard_felt: '#5d5a6b',
  cloth: '#5e9a78',
}

/**
 * Paper punchings, coloured by thickness.
 *
 * In a technician's drawer the colour *is* the thickness — you reach for a
 * two-tenths by its colour, without measuring — so the palette is keyed on the
 * number the engine sends rather than on the part. These are the owner's own
 * gradings, recorded with their provenance in
 * `data/params/punching-papers.json`; a different supplier would renumber
 * them, which is exactly why the mapping is data and not a guess.
 */
const PAPER_FILL: ReadonlyArray<readonly [number, string]> = [
  [0.08, '#f1efe7'], // white
  [0.12, '#b4d8a8'], // pastel green
  [0.2, '#86c5e8'], // sky blue
  [0.4, '#c19a6b'], // camel
  [0.5, '#2e2e2e'], // black
]

/**
 * Above half a millimetre the drawer stops colour-coding and the papers are
 * plain card — 0.75 and 1.25 mm both arrive this colour, and the technician
 * tells them apart by feel and by what he reached for.
 */
const CARD_FILL = '#9a7b52'

/** The canvas behind everything: a grey light enough for the wood, dark
 * enough that a white keytop still reads as a part. */
const PAGE = '#e7e4dd'

/** Edges in the section, opaque so that one hidden under the key's wood stays
 * hidden. */
const EDGE = '#5b544a'

/**
 * The contacts' colours, by state: where the key bears, a gap of under a
 * millimetre still open, and drawn parts overlapping, which a rigid contact
 * should never show and a frame out of fit with its key does.
 */
export const CONTACT_COLOUR: Record<Contact['state'], string> = {
  closed: '#1d63a8',
  open: '#8a8a8a',
  overlapping: '#e8590c',
}

/** The colour of a paper of this thickness. */
function paperColour(mm: number): string {
  for (const [thickness, fill] of PAPER_FILL) {
    if (Math.abs(thickness - mm) < 0.005) return fill
  }
  return CARD_FILL
}

/** The colour a part is drawn in, from its material. */
function colourOf(o: Outline): string {
  if (o.paper_mm != null) return paperColour(o.paper_mm)
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

/**
 * Disposes a geometry when it is replaced or unmounted.
 *
 * A geometry handed to a mesh through the `geometry` prop is not an instance
 * in the tree, so react-three-fiber never disposes it. Every edit rebuilds the
 * scene from a fresh `JSON.parse`, so without this a slider drag leaves a
 * thousand orphaned buffers on the GPU — in an application meant to stay open
 * while a technician works.
 */
function useDisposal(geometry: THREE.BufferGeometry) {
  useEffect(() => () => geometry.dispose(), [geometry])
}

// ---- the solid ------------------------------------------------------------

function Extruded({ points, width, colour }: { points: number[]; width: number; colour: string }) {
  const geometry = useMemo(() => {
    const g = new THREE.ExtrudeGeometry(toShape(points), { depth: width, bevelEnabled: false })
    // Extrusion runs along +z from the outline's plane; centre it so the
    // sagittal plane really is the middle of the part.
    g.translate(0, 0, -width / 2)
    g.computeVertexNormals()
    return g
  }, [points, width])
  const edges = useMemo(() => new THREE.EdgesGeometry(geometry, 25), [geometry])
  useDisposal(geometry)
  useDisposal(edges)
  return (
    <group>
      <mesh geometry={geometry}>
        <meshLambertMaterial color={colour} />
      </mesh>
      <lineSegments geometry={edges}>
        <lineBasicMaterial color="#3a342b" transparent opacity={0.4} />
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
function Lathed({ profile, axisX, colour }: { profile: number[]; axisX: number; colour: string }) {
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
  useDisposal(geometry)
  return (
    <mesh geometry={geometry} position={[axisX, 0, 0]}>
      <meshLambertMaterial color={colour} side={THREE.DoubleSide} />
    </mesh>
  )
}

/**
 * The leads, as discs let in through the side of the key: their axis lies
 * across the keyboard, so their face shows on the section.
 *
 * `width` is the keystick's, read off the engine's own outline. Writing
 * 22 mm here instead would size one half of the disc from a literal while the
 * other half — its radius — is already computed in Rust from the real width.
 */
function SolidLeads({ leads, width }: { leads: Lead[]; width: number }) {
  return (
    <>
      {leads.map((l, i) => (
        <mesh key={i} position={[l.x, l.y, 0]} rotation={[Math.PI / 2, 0, 0]}>
          <cylinderGeometry args={[l.radius, l.radius, width * 1.02, 28]} />
          <meshLambertMaterial color="#54535c" />
        </mesh>
      ))}
    </>
  )
}

// ---- the section ----------------------------------------------------------

/** A flat fill in the sagittal plane, painted in its layer. */
function Fill({
  points,
  colour,
  layer,
  masked = false,
}: {
  points: number[]
  colour: string
  layer: number
  /** Whether a cut keeps it from being painted: the key's own parts are. */
  masked?: boolean
}) {
  const geometry = useMemo(() => new THREE.ShapeGeometry(toShape(points)), [points])
  useDisposal(geometry)
  return (
    <mesh geometry={geometry} renderOrder={layer}>
      <meshBasicMaterial
        color={colour}
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
  colour = EDGE,
}: {
  pieces: number[]
  layer: number
  colour?: string
}) {
  const geometry = useMemo(() => {
    const g = new THREE.BufferGeometry()
    const xyz: number[] = []
    for (let i = 0; i < pieces.length; i += 2) {
      xyz.push(pieces[i] as number, pieces[i + 1] as number, 0)
    }
    g.setAttribute('position', new THREE.Float32BufferAttribute(xyz, 3))
    return g
  }, [pieces])
  useDisposal(geometry)
  return (
    <lineSegments geometry={geometry} renderOrder={layer}>
      <lineBasicMaterial color={colour} depthTest={false} depthWrite={false} toneMapped={false} />
    </lineSegments>
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
}: {
  outline: Outline
  layer: number
  edgeLayer: number
  masked: boolean
  /** Segments that are no edge: where the part runs along a cut's mouth. */
  open: ReadonlySet<number>
}) {
  const colour = colourOf(outline)
  if (outline.sweep === 'extruded') {
    return (
      <>
        <Fill points={outline.points} colour={colour} layer={layer} masked={masked} />
        <Edges pieces={strokes(outline.points, outline.segs, open)} layer={edgeLayer} />
      </>
    )
  }
  const onAxis = axisSegments(outline.points, outline.segs)
  const half = strokes(outline.points, outline.segs, onAxis)
  const mirror = mirrored(half)
  return (
    <group position={[outline.axis_x, 0, 0]}>
      <Fill points={outline.points} colour={colour} layer={layer} masked={masked} />
      <Fill points={mirrored(outline.points)} colour={colour} layer={layer} masked={masked} />
      <Edges pieces={half} layer={edgeLayer} />
      <Edges pieces={mirror} layer={edgeLayer} />
    </group>
  )
}

/** The leads in the section: their faces, over the key's wood. */
function SectionLeads({ leads }: { leads: Lead[] }) {
  return (
    <>
      {leads.map((l, i) => (
        <mesh key={i} position={[l.x, l.y, 0]} renderOrder={LAYER.marks}>
          <circleGeometry args={[l.radius, 28]} />
          <meshBasicMaterial
            color="#54535c"
            depthTest={false}
            side={THREE.DoubleSide}
            toneMapped={false}
          />
        </mesh>
      ))}
    </>
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
      <meshBasicMaterial color="#c8302a" depthTest={false} toneMapped={false} />
    </mesh>
  )
}

/**
 * The contacts within a millimetre of closing: a dot where the frame's
 * surface is, and across an open or overlapping contact a line to where the
 * key's is. Drawn in the keybed's frame, over everything, in both views: the
 * points are where the engine's contact rows put them.
 */
function Contacts({ contacts }: { contacts: Contact[] }) {
  return (
    <>
      {(['open', 'overlapping'] as const).map((state) => (
        <Edges
          key={state}
          pieces={contactStrokes(contacts, state)}
          layer={LAYER.contacts}
          colour={CONTACT_COLOUR[state]}
        />
      ))}
      {contacts.map((c, i) => (
        <mesh key={i} position={[c.on_frame[0], c.on_frame[1], 0]} renderOrder={LAYER.contacts}>
          <sphereGeometry args={[0.9, 16, 12]} />
          <meshBasicMaterial
            color={CONTACT_COLOUR[c.state]}
            depthTest={false}
            depthWrite={false}
            toneMapped={false}
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
      <meshLambertMaterial color="#b9b2a2" />
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
  leads,
  marker,
  keyWidth,
}: {
  body: number
  pose: Pose
  outlines: Outline[]
  mode: ViewMode
  leads: Lead[]
  marker: Marker | undefined
  keyWidth: number
}) {
  const matrix = useMemo(() => poseMatrix(pose), [pose])
  const cuts = outlines.filter((o) => o.role === 'cut')
  const parts = outlines.filter((o) => o.role !== 'cut')
  const grounded = body === GROUND
  return (
    <group matrix={matrix} matrixAutoUpdate={false}>
      {mode === 'solid' ? (
        <>
          {parts.map((o, i) =>
            o.sweep === 'revolved' ? (
              <Lathed
                key={`${o.code}-${i}`}
                profile={o.points}
                axisX={o.axis_x}
                colour={colourOf(o)}
              />
            ) : (
              <Extruded
                key={`${o.code}-${i}`}
                points={o.points}
                width={o.width}
                colour={colourOf(o)}
              />
            ),
          )}
          <SolidLeads leads={leads} width={keyWidth} />
        </>
      ) : (
        <>
          {cuts.map((o, i) => (
            <Void key={`${o.code}-${i}`} points={o.points} />
          ))}
          {cuts.map((o, i) => {
            // A cut's mouth runs along its body's outline: no edge there.
            const mouth = new Set<number>()
            for (const p of parts) for (const s of sharedSegments(o, p)) mouth.add(s)
            return (
              <Edges
                key={`${o.code}-edges-${i}`}
                pieces={strokes(o.points, o.segs, mouth)}
                layer={LAYER.keyEdges}
              />
            )
          })}
          {parts.map((o, i) => {
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
              />
            )
          })}
          <SectionLeads leads={leads} />
        </>
      )}
      {marker && <MeasuringPoint at={marker.at} />}
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

export function SagittalView({ scene, mode }: { scene: Scene; mode: ViewMode }) {
  const preset = CAMERA[mode]
  // The keystick's width, off the engine's own outline. The leads are drawn
  // across it, and their radius is already computed in Rust from the same
  // number.
  const keyWidth = scene.outlines.find((o) => o.code === 'keystick')?.width ?? 0

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
      <ambientLight intensity={1.6} />
      <directionalLight position={[-200, 300, 400]} intensity={2.0} />
      <directionalLight position={[300, 140, -250]} intensity={0.9} />

      {/* Mirrored in x so the player's end of the key is on the LEFT, which is
          how the trade's own sectional plates are drawn. A view convention and
          nothing more: the engine's x still runs towards the player, and no
          number changes. */}
      <group scale={[-1, 1, 1]}>
        <Keybed />
        {scene.poses.map((pose, body) => (
          <Body
            key={body}
            body={body}
            pose={pose}
            outlines={scene.outlines.filter((o) => o.body === body)}
            mode={mode}
            leads={scene.leads.filter((l) => l.body === body)}
            marker={scene.measuring_point.body === body ? scene.measuring_point : undefined}
            keyWidth={keyWidth}
          />
        ))}
        <Contacts contacts={scene.contacts} />
      </group>

      {/* Re-keyed on the mode so the two buttons are two viewpoints, not two
          starting points you then have to re-aim. */}
      <OrbitControls key={mode} makeDefault target={preset.target} enableDamping={false} />
    </Canvas>
  )
}
