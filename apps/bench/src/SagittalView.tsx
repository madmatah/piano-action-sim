/**
 * The bench's 3-D view.
 *
 * Built in millimetres, in the keybed frame: `x` towards the player from the
 * balance pin, `y` up from the keybed surface, `z` across the keyboard. Those
 * are the engine's own coordinates, so nothing is transformed on the way in
 * except the camera — if a part looks wrong here, the geometry is wrong, not
 * the drawing.
 *
 * # Why the solids are extruded sections and not models
 *
 * There is **one** definition of each part: the sagittal outline the engine
 * computes. The mass integrator integrates it and this view extrudes it, and
 * the engine's contacts read the same underside and bearings, built from the
 * same fields. A separately modelled mesh would be a second geometry, and
 * the moment it disagreed with the first the displayed dimensions would stop
 * describing what is measured. So the section is not a reduced view of the
 * solid — the solid *is* the section, swept.
 *
 * The outlines arrive already rotated into the key's current attitude. This
 * component does not know how a key moves and must not learn: a renderer that
 * computed the rotation would be a second kinematics.
 *
 * # Two library traps, both met here
 *
 * drei's `Line` rendered nothing while a plain mesh in the same scene rendered
 * fine, so borders are plain `lineSegments`. drei's `Text` fetches a font over
 * the network and suspends until it arrives, which blanks the canvas on a bench
 * with no route out, so every label is in HTML.
 *
 * A third, from R3F itself: the `camera` prop is read **once, at creation**.
 * Changing it later does not move the camera, which is why the two viewpoints
 * re-key `OrbitControls` on the mode instead of animating one camera between
 * them.
 */
import { OrbitControls } from '@react-three/drei'
import { Canvas } from '@react-three/fiber'
import { useEffect, useMemo } from 'react'
import * as THREE from 'three'
import type { Lead, Scene } from './bench'
import { toShape } from './outline'

/** Wood, metal, and the compliant linings, told apart at a glance. */
const FILL: Record<string, string> = {
  key: '#efe5cd',
  balance_rail: '#c6a477',
  front_rail: '#c6a477',
  back_rail: '#c6a477',
  balance_rail_pin: '#aab2bb',
  front_rail_pin: '#aab2bb',
  balance_rail_punching: '#d8706a',
  front_rail_punching: '#5e9a78',
  back_rail_cloth: '#5e9a78',
  capstan: '#c9ccd1',
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

/** The colour of a paper of this thickness. */
function paperColour(mm: number): string {
  for (const [thickness, fill] of PAPER_FILL) {
    if (Math.abs(thickness - mm) < 0.005) return fill
  }
  return CARD_FILL
}

/** Two viewpoints: the technical section, and the solid in three-quarters. */
export type ViewMode = 'section' | 'solid'

function Part({ points, width, colour }: { points: number[]; width: number; colour: string }) {
  const geometry = useMemo(() => {
    const g = new THREE.ExtrudeGeometry(toShape(points), {
      depth: width,
      bevelEnabled: false,
    })
    // Extrusion runs along +z from the outline's plane; centre it so the
    // sagittal plane really is the middle of the part.
    g.translate(0, 0, -width / 2)
    g.computeVertexNormals()
    return g
  }, [points, width])

  const edges = useMemo(() => new THREE.EdgesGeometry(geometry, 25), [geometry])

  // A geometry handed to a mesh through the `geometry` prop is not an instance
  // in the tree, so react-three-fiber never disposes it. Every edit rebuilds
  // the whole scene from a fresh `JSON.parse`, so without this a slider drag
  // leaves a thousand orphaned buffers on the GPU — in an application meant to
  // stay open while a technician works.
  useEffect(
    () => () => {
      geometry.dispose()
      edges.dispose()
    },
    [geometry, edges],
  )

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
 * Three kinds of part need this. A capstan is a screw, and extruding its
 * silhouette across the 22 mm of the key draws a block of wood. A punching is
 * a **washer**, and extruding it draws a strip of felt the length of the rail.
 * A balance pin is round.
 *
 * The profile arrives as `(radius, height)` pairs, so the part is *placed*
 * rather than deformed: the axis and the angle come together in `axis`,
 * because a radius does not rotate. A profile whose first radius is non-zero
 * leaves the hole its pin passes through — closing it would swallow the pin,
 * and that hole is the feature saying the key is located and not merely
 * resting.
 */
function Turned({
  profile,
  axis,
  colour,
}: {
  profile: number[]
  axis: [number, number, number]
  colour: string
}) {
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

  return (
    <group position={[axis[0], axis[1], 0]} rotation={[0, 0, -Math.asin(axis[2])]}>
      <mesh geometry={geometry}>
        <meshLambertMaterial color={colour} side={THREE.DoubleSide} />
      </mesh>
    </group>
  )
}

/** Leads, drawn to scale, because a technician judges a leading by looking. */
/**
 * The leads, as discs let in through the side of the key.
 *
 * `width` is the keystick's, read off the engine's own `key` outline. Writing
 * 22 mm here instead would size one half of the disc from a literal while the
 * other half — its radius — is already computed in Rust from the real width,
 * so a sharp or a measured key would draw leads protruding from the wood with
 * diameters still claiming to be to scale.
 */
function Leads({ leads, width }: { leads: Lead[]; width: number }) {
  return (
    <>
      {leads.map((l, i) => (
        <mesh key={i} position={[l.x, l.y, 0]} rotation={[Math.PI / 2, 0, 0]}>
          {/* A lead is a disc let in through the side of the key, so its axis
              lies across the keyboard and its face shows on the section. */}
          <cylinderGeometry args={[l.radius, l.radius, width * 1.02, 28]} />
          <meshLambertMaterial color="#54535c" />
        </mesh>
      ))}
    </>
  )
}

/**
 * The measuring point: a marker at the datum every touchweight is read at.
 *
 * Marked on purpose. A weight is only a measurement if you can see where it was
 * taken, and 13 mm back from the front lip is not somewhere a reader can guess.
 */
function MeasuringPoint({ at }: { at: [number, number] }) {
  const [x, y] = at
  return (
    <mesh position={[x, y + 3.5, 0]} rotation={[Math.PI, 0, 0]}>
      <coneGeometry args={[2.2, 6, 4]} />
      <meshBasicMaterial color="#c8302a" />
    </mesh>
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
  const keyWidth = scene.outlines.find((o) => o.id === 'key')?.width ?? 0

  return (
    <Canvas
      // Orthographic in both modes: this is a measuring instrument, and
      // perspective would make two equal dimensions measure differently
      // depending on where they sit.
      orthographic
      camera={{ position: preset.position, zoom: preset.zoom }}
      style={{ background: '#fbfaf7' }}
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
        {scene.outlines.map((o, i) => {
          // Papers repeat their id — a rail carries several — so the React key
          // carries the index. The colour comes from the thickness, which is
          // how a technician tells one paper from another in the drawer.
          const colour = o.paper_mm != null ? paperColour(o.paper_mm) : (FILL[o.id] ?? '#9b9b9b')
          return o.sweep === 'revolved' ? (
            <Turned key={`${o.id}-${i}`} profile={o.points} axis={o.axis} colour={colour} />
          ) : (
            <Part key={`${o.id}-${i}`} points={o.points} width={o.width} colour={colour} />
          )
        })}
        <Leads leads={scene.leads} width={keyWidth} />
        <MeasuringPoint at={scene.measuring_point} />
      </group>

      {/* Re-keyed on the mode so the two buttons are two viewpoints, not two
          starting points you then have to re-aim. */}
      <OrbitControls key={mode} makeDefault target={preset.target} enableDamping={false} />
    </Canvas>
  )
}
