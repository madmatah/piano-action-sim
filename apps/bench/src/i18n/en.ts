/**
 * The bench in English: every sentence it shows, composed from the codes and
 * numbers the engine sends.
 *
 * A catalog in another language has the same shape (`Catalog`), its
 * trade terms the trade's own in that language; between English and French
 * they are mapped in `docs/research/A3-nomenclature-and-regulation.md` > Bilingual glossary FR ↔ EN (+ DE) and code identifiers.
 */
import type {
  Contact,
  EditRefusal,
  FrictionRefusal,
  Label,
  Limit,
  Part,
  Problem,
  Riding,
  SettingUnit,
  Solve,
  Stroke,
  Target,
  TemporaryWeight,
  Verdict,
  WeighingRefusal,
} from '@pas/wasm/types'
import {
  decimals,
  FROM_LIP_MM,
  fixed,
  GAP_MM,
  LENGTH_MM,
  MASS_G,
  OUT_OF_RANGE,
  PART_MM,
  SEARCH_MM,
  STATION_MM,
  TEMPORARY_WEIGHT_G,
  WEIGHT_G,
} from '../format'
import { type Catalog, type Labeled, type Named, unitIn, type Worded, word } from './catalog'

/** `x` to the decimals `resolution` is shown to. */
const to = (quantity: number, resolution: number): string => fixed(quantity, decimals(resolution))

/** Reasons as a note or a tooltip lists them, each a clause of its own. */
const reasons = (clauses: readonly string[]): string => clauses.join('; ')

/** Items as a sentence lists them: "a", "a and b", "a, b and c". */
function list(terms: readonly string[]): string {
  return terms.length < 2
    ? (terms[0] ?? '')
    : `${terms.slice(0, -1).join(', ')} and ${terms[terms.length - 1]}`
}

/** What a stop is called in a sentence that may list several: no comma
 * inside a name, so that a list of them reads as one. */
const STOP: Named<Contact> = {
  balance_rail_punching: 'balance punching',
  front_rail_punching: 'front punching',
  back_rail_cloth: 'back rail cloth',
  front_pin_in_the_mortise: 'front rail pin at the top of its mortise',
  front_pin_on_the_wood: 'front rail pin against the wood beside its mortise',
}

/** The stops, as a sentence lists them: "its front punching and its back
 * rail cloth"; "its stops" where there are none. */
function stopsNamed(codes: readonly Contact[]): string {
  return codes.length === 0 ? 'its stops' : list(codes.map((code) => `its ${STOP[code]}`))
}

/** Where a stop is, as the touch panel says the key stands on it. */
const ON_THE: Named<Contact> = {
  balance_rail_punching: 'on the balance punching',
  front_rail_punching: 'on the front punching',
  front_pin_in_the_mortise: 'on the front rail pin, at the top of its mortise',
  front_pin_on_the_wood: 'on the front rail pin, against the wood round its mortise',
  back_rail_cloth: 'on the back rail cloth',
}

/** A contact as a note names it in passing. */
const CONTACT: Named<Contact> = {
  balance_rail_punching: 'balance rail punching',
  front_rail_punching: 'front rail punching',
  back_rail_cloth: 'back rail cloth',
  front_pin_in_the_mortise: 'front pin in the mortise',
  front_pin_on_the_wood: 'front pin on the wood',
}

/** A part as a fault names it. */
const PART: Named<Part> = {
  keystick: 'keystick',
  keytop: 'keytop',
  key_front: 'key front',
  key_button: 'key button',
  back_check_block: 'back check block',
  top_clearance: 'wippen rail clearance',
  key_tail_felt: 'key tail felt',
  capstan: 'capstan',
  front_mortise: 'front mortise',
  balance_mortise: 'balance mortise',
  balance_mortise_bushing: 'balance mortise bushing',
  balance_hole: 'balance hole',
  key_button_side_hole: 'key button side hole',
  balance_rail: 'balance rail',
  balance_rail_pin: 'balance rail pin',
  balance_rail_punching: 'balance rail punching',
  front_rail: 'front rail',
  front_rail_pin: 'front rail pin',
  front_rail_punching: 'front rail punching',
  paper_punching: 'paper punching',
  back_rail: 'back rail',
  back_rail_cloth: 'back rail cloth',
}

/** How the key rides on its stops, naming them and the lift. */
function riding(how: Riding): string {
  return `the key rides on ${stopsNamed([how.front, 'back_rail_cloth'])}, ${to(how.lift_mm, LENGTH_MM)} mm above its balance punching: its stops hold it higher than its punching does, so it has no stroke, and no weight takes it from one seat to the other.`
}

/** How far past a target, in millimeters: to the hundredth, a shortfall
 * under that said as under a hundredth. */
function hundredths(mm: number): string {
  return mm < LENGTH_MM / 2 ? `under ${to(LENGTH_MM, LENGTH_MM)} mm` : `${to(mm, LENGTH_MM)} mm`
}

/** A regulation target, as a sentence names it. */
const TARGET: Named<Target> = {
  key_height: 'key height',
  key_dip: 'key dip',
}

/** A static solve that reached its cap of steps, by which one it was. */
const NOT_CONVERGED: Named<Solve> = {
  seat: "the key could not be seated within the static solve's cap of steps.",
  walk: "the static solve reached its cap of steps on the key's way down or up.",
  sweep: "the static solve reached its cap of steps on a step of the band's sweep.",
}

/** Which way the key was going. */
const STROKE: Named<Stroke> = {
  down: 'on the way down',
  up: 'on the way up',
}

/** Each panel's heading, which a sentence pointing to the panel quotes. */
const panel: Catalog['panel'] = {
  instrument: 'The instrument',
  sheet: 'Weighing sheet',
  regulation: 'Regulation',
  punchings: 'Punchings and cloth',
  key: 'The key',
  weighing: 'The key, weighed whole',
  frame: 'The frame',
  touch: 'Touch',
}

/** Why a reading is refused, or why the key cannot be seated. */
const refusal: Catalog['refusal'] = {
  geometry_degenerate: () => 'the geometry is degenerate.',
  key_not_weighed: () =>
    `nobody has weighed this key, so the bench knows no mass for it: weigh it whole and type the weighing under “${panel.weighing}”.`,
  scale_not_in_front_of_the_pivot: () =>
    "the measuring point stands at or behind the balance pin, where the jig's scale cannot hold the key level.",
  cannot_seat: ({ fighting }) =>
    `the key has no seat on this frame: ${stopsNamed(fighting)} ${fighting.length === 1 ? 'stands' : 'stand'} in its drawn parts, and the model finds no pose of the key on its frame clear of ${fighting.length === 1 ? 'it' : 'them'}.`,
  lifts_off_its_frame: ({ leaves_its_pin_mm }) =>
    leaves_its_pin_mm < 0
      ? `the balance punching stands ${to(-leaves_its_pin_mm, LENGTH_MM)} mm above the top of the balance pin: a key on it is off its pin and off its frame.`
      : `its stops would hold the key more than ${to(leaves_its_pin_mm, LENGTH_MM)} mm above its balance punching, where it leaves its balance pin: off its pin and off its frame.`,
  no_stop_below_the_front: () =>
    "nothing stops the front of the key: the front punching's center stands off the key's underside, so the key bears on nothing at the bottom of its dip.",
  no_stop_under_the_tail: () =>
    "nothing holds the back of the key down: the back rail cloth stands where the key's tail cannot reach it.",
  capstan_not_behind_the_bearing: () =>
    'the capstan stands at or in front of the balance bearing, where the action cannot hold the back of the key down.',
  measuring_point_not_in_front: () =>
    'the measuring point stands at or behind the balance bearing, where a weight cannot take the key down.',
  not_converged: ({ solve }) => NOT_CONVERGED[solve],
  no_travel: ({ riding: ridingOn }) =>
    ridingOn === null
      ? 'the key has no stroke: it stands at the bottom of its dip at rest.'
      : riding(ridingOn),
  no_lever: () => 'a weight on the measuring point does not turn the key.',
  does_not_reach_the_end_of_its_stroke: ({ cap_g }) =>
    `${to(cap_g, WEIGHT_G)} g on the measuring point did not take the key to the end of its stroke.`,
  no_equilibrium: () => 'the key found no equilibrium on its way down or up: its stops fight.',
  walk_lifts_off_its_frame: () =>
    'a weight the walk put on the measuring point lifts the key off its frame.',
  sweep_lifts_off_its_punching: ({ stroke, travel_mm, force_g }) =>
    `${to(travel_mm, LENGTH_MM)} mm into the stroke ${STROKE[stroke]}, a finger would have to ${force_g < 0 ? 'pull' : 'press'} ${to(Math.abs(force_g), WEIGHT_G)} g at the measuring point, and the balance punching would carry nothing under it: the key would lift off its punching instead of turning, so the force would not be the one read.`,
  sweep_lifts_off_its_frame: () =>
    "a step of the band's sweep took the key off its balance pin, and off its frame.",
  no_key_top_at_the_station: ({ station_mm }) =>
    `no top of the key stands ${to(Math.abs(station_mm), STATION_MM)} mm ${station_mm < 0 ? 'behind' : 'in front of'} the balance pin, where the temporary weight goes.`,
  from_a_refused_reading: ({ reading }) =>
    `the ${reading === 'down_weight' ? 'down weight' : reading === 'up_weight' ? 'up weight' : 'reading'} it is computed from is refused.`,
}

/** The temporary weight a key friction weight stood on the key. */
function temporary(weight: TemporaryWeight | null): string {
  if (weight === null) return ''
  return ` (${to(weight.mass_g, TEMPORARY_WEIGHT_G)} g, ${to(Math.abs(weight.at_mm), STATION_MM)} mm ${weight.at_mm < 0 ? 'behind' : 'in front of'} the balance pin)`
}

/** Nothing under the front of the key, which keeps either target out of
 * reach for the same reason. */
const NO_STOP_BELOW_THE_FRONT =
  "nothing stops the front of the key, the front punching's center standing off its underside"

/**
 * What keeps a regulation target out of reach: a wording for every limit and
 * every target, given how much paper the search lays on a stack, so that a
 * limit or a target the engine gains and this does not word fails the
 * type-check.
 */
const LIMIT_WHY: {
  readonly [L in Limit]: { readonly [T in Target]: (searchMm: number) => string }
} = {
  rail_too_low: {
    key_height: (searchMm) =>
      `it would take more than ${to(searchMm, SEARCH_MM)} mm of paper under the balance punching, the balance rail standing too low`,
    key_dip: (searchMm) =>
      `it would take more than ${to(searchMm, SEARCH_MM)} mm of paper on the front punching, the front rail standing too low`,
  },
  front_pin_in_mortise: {
    key_height: () =>
      'the front rail pin would bottom in its mortise with the key at rest that low',
    key_dip: () => 'the front rail pin bottoms in its mortise before the key goes that deep',
  },
  front_pin_on_the_wood: {
    key_height: () =>
      "the front rail pin stands under the key's wood, and only a key raised clear of it has a seat",
    key_dip: () =>
      "the front rail pin meets the key's wood beside its mortise before the key goes that deep",
  },
  felt_thicker_than_the_stack: {
    key_height: () =>
      "the balance punching's felt alone, with no paper under it, stands the key higher",
    key_dip: () => "the front punching's felt alone, with no paper under it, stops the key short",
  },
  stops_fight: {
    key_height: () =>
      'the front of the key would come down onto its front punching at rest, its tail on the cloth',
    key_dip: () => 'the front punching would meet the key at rest before the dip is that shallow',
  },
  no_stop_below_the_front: {
    key_height: () => NO_STOP_BELOW_THE_FRONT,
    key_dip: () => NO_STOP_BELOW_THE_FRONT,
  },
  off_its_balance_pin: {
    key_height: () =>
      "the key would leave its balance pin first, the paper raising its balance punching past the pin's top",
    key_dip: () => 'the key would ride off its balance pin before the dip is that shallow',
  },
}

const regulationWhy: Catalog['regulation']['why'] = {
  regulation_infeasible: ({ target, wanted_mm, limited_by, shortfall_mm, search_mm }) =>
    `${to(wanted_mm, LENGTH_MM)} mm of ${TARGET[target]} is out of reach, by ${hundredths(shortfall_mm)}: ${LIMIT_WHY[limited_by][target](search_mm)}.`,
  no_seat: ({ why }) =>
    `the key could not be seated while the papers were being tried: ${word(refusal, why.code, why)}`,
}

/** Why a key friction weight typed was refused. */
const frictionWhy: Worded<FrictionRefusal> = {
  protocol_refused: ({ why }) =>
    `Stanwood's protocol cannot read a key friction weight on this key: ${word(refusal, why.code, why)}`,
  under_no_friction: ({ typed_g, least_g }) =>
    `${to(typed_g, MASS_G)} g is less than Stanwood's protocol reads on this key with no friction at all, ${to(least_g, MASS_G)} g: what the key's tilt at rest leaves of its own balance and of the temporary weight's.`,
  beyond_the_search: ({ typed_g, most_g, holds_g }) =>
    `${to(typed_g, MASS_G)} g is more than the tightest bushings the bench tries read, ${to(most_g, MASS_G)} g: they hold ${to(holds_g, WEIGHT_G)} g at the measuring point with the key level, and would hold the key wherever it was put.`,
}

/** Why a weighing of the key was refused. */
const weighingWhy: Worded<WeighingRefusal> = {
  roller_off_the_covering: ({ from_lip_mm, covering_mm }) =>
    `a front weight is read on the covering, in front of the balance pin, from its lip to ${to(covering_mm, FROM_LIP_MM)} mm behind it: ${to(from_lip_mm, FROM_LIP_MM)} mm from the lip is off it.`,
  center_of_mass_off_the_key: ({ at_mm }) =>
    `that front weight on that mass stands the key's center of mass ${to(Math.abs(at_mm), STATION_MM)} mm ${at_mm < 0 ? 'behind' : 'in front of'} the balance pin, past its ${at_mm < 0 ? 'tail' : "covering's lip"}: off the key.`,
}

/** Two parts that would overlap along the key, as a problem names them. A
 * felted thinned tail, which ends at or behind every glued block, is said by
 * how far in front of the block's back end it would end: on a key shortened
 * at the back by more than its tail's length, further than the two overlap.
 * A block standing across the step down to a bare tail is said by how far
 * it reaches past the step; a covering over a bare tail, by how far the tail
 * runs under it. */
const OVERLAP = (back: Part, front: Part, by: string): string =>
  back === 'key_tail_felt'
    ? `the key's thinned tail and its felt would end ${by} mm in front of the ${PART[front]}'s back end, and the block is glued on the keystick's full height`
    : front === 'keystick'
      ? `the ${PART[back]} would stand ${by} mm across the step down to the key's thinned tail, and a block stands on the one or on the other`
      : back === 'keystick' && front === 'keytop'
        ? `the key's thinned tail would run ${by} mm under the keytop, which is glued on the keystick's full height`
        : front === 'capstan'
          ? `the ${PART[back]} would run ${by} mm under the capstan's head, which screws into the keystick's full height`
          : `the ${PART[back]} and the ${PART[front]} would overlap by ${by} mm along the key`

/** A setting's unit, as a value in it is written. */
const UNIT: Named<SettingUnit> = { mm: 'mm', g: 'g', gmm: 'g·mm' }

/** What no part could be built as, or what the bench has not. */
const problem: Worded<Problem> = {
  not_finite: () => 'a value that is not a number.',
  out_of_range: ({ param, value, min, max }) => {
    const unit = unitIn(param, UNIT)
    return `${to(value, OUT_OF_RANGE)}${unit} is not a value this part can have${min === null ? '' : `, no less than ${to(min, OUT_OF_RANGE)}${unit}`}${max === null ? '' : `${min === null ? ',' : ' and'} no more than ${to(max, OUT_OF_RANGE)}${unit}`}.`
  },
  not_in_the_drawer: ({ thickness_mm, drawer_mm }) =>
    `no drawer holds a ${to(thickness_mm, LENGTH_MM)} mm paper: it holds ${list(drawer_mm.map((mm) => to(mm, LENGTH_MM)))} mm.`,
  mass_not_positive: () => 'a key weighs more than nothing.',
  outline_crosses_itself: ({ back, front, by_mm }) =>
    `${OVERLAP(back, front, to(by_mm, PART_MM))}: no key is cut like that.`,
  feature_outside_keystick: ({ part, by_mm }) =>
    part === 'top_clearance'
      ? `the ${PART[part]} would run ${to(by_mm, PART_MM)} mm back over the key's thinned tail: it is cut in the keystick's full height.`
      : `the ${PART[part]} would stand off the flat of the key's underside, by ${to(by_mm, PART_MM)} mm.`,
  capstan_behind_back_check_block: ({ by_mm }) =>
    `the capstan would stand behind the back check block: its place is between the block and the key button, and it clears the block ${to(by_mm, PART_MM)} mm further forward.`,
  capstan_in_block: ({ block, by_mm }) =>
    `the capstan would stand ${to(by_mm, PART_MM)} mm into the ${PART[block]}.`,
  capstan_off_the_full_height: ({ by_mm }) =>
    `the capstan would stand over the key's thinned tail, and it screws into the keystick's full height: its head stands wholly over it ${to(by_mm, PART_MM)} mm further forward.`,
  capstan_dome_narrower_than_its_head: ({ dome_mm, head_mm }) =>
    `a dome of ${to(dome_mm, LENGTH_MM)} mm cannot cap a capstan head of ${to(head_mm, LENGTH_MM)} mm radius.`,
  clearance_not_a_cut: ({ length_mm, depth_mm }) =>
    `a ${PART.top_clearance} ${to(length_mm, PART_MM)} mm long and ${to(depth_mm, PART_MM)} mm deep cuts nothing: it is cut below the keystick's full top, its front end in front of its back end.`,
  clearance_through_keystick: ({ depth_mm, wood_mm }) =>
    `a ${PART.top_clearance} ${to(depth_mm, PART_MM)} mm deep goes through the keystick, which has ${to(wood_mm, PART_MM)} mm of wood under it.`,
  capsule_ends_overlap: ({ by_mm }) =>
    `the key button's rounded ends would reach ${to(by_mm, PART_MM)} mm past each other along its top: they meet at most.`,
  does_not_fit: ({ part, within, by_mm }) =>
    `the ${PART[part]} would not fit in the ${PART[within]}, by ${to(by_mm, PART_MM)} mm.`,
  mortise_through_key_top: ({ depth_mm, wood_mm }) =>
    `a front mortise ${to(depth_mm, PART_MM)} mm deep goes through the ${to(wood_mm, PART_MM)} mm of the keystick.`,
  measuring_point_off_covering: ({ from_lip_mm, covering_mm }) =>
    `the measuring point stands on the covering, from its lip to ${to(covering_mm, FROM_LIP_MM)} mm behind it: ${to(from_lip_mm, FROM_LIP_MM)} mm from the lip is off it.`,
  unknown_path: ({ path }) => `the bench has no setting ${path}.`,
  unknown_profile: ({ profile }) => `the bench ships no instrument ${profile}.`,
  set_by_action: ({ path }) => `${path} is not set directly: it is laid, regulated or weighed.`,
  no_such_paper: ({ index, papers }) =>
    `the stack holds ${papers} ${papers === 1 ? 'paper' : 'papers'}: there is no paper ${index + 1} to lift off.`,
  not_an_action: () => 'the bench knows no such operation.',
}

/** Why any edit was refused. */
const editRefusal: Worded<EditRefusal> = {
  ...problem,
  ...regulationWhy,
  ...weighingWhy,
  ...frictionWhy,
}

/** An edit refused that leaves nothing in particular as it was. */
const nothingChanged = (why: EditRefusal): string =>
  `Refused, nothing changed: ${word(editRefusal, why.code, why)}`

/** What each control is worded by, by its label's code. */
const control: { readonly [K in Label]: Labeled } = {
  profile: { label: 'on the bench' },
  measuring_point: { label: 'measuring point, back from the key’s front lip' },
  key_height: { label: 'key height' },
  key_dip: { label: 'key dip' },
  regulate: { label: 'regulate to these targets' },
  balance_stack: { label: 'balance punching — key height' },
  front_stack: { label: 'front punching — key dip' },
  key_friction_weight: { label: 'key friction weight' },
  balance_felt: { label: 'balance felt' },
  front_felt: { label: 'front felt' },
  front_pin: { label: 'front rail pin, above the rail' },
  balance_pin: { label: 'balance rail pin, above the rail' },
  front_mortise: { label: 'front mortise, above the underside' },
  front_mortise_cutter_diameter: { label: 'front mortise cutter diameter' },
  front_mortise_travel: { label: 'front mortise cutter travel' },
  back_cloth: { label: 'back rail cloth' },
  capstan: { label: 'capstan behind the balance pin' },
  front_edge: { label: 'front edge from the balance pin' },
  capstan_height: { label: 'capstan height' },
  capsule_height: { label: 'key button height' },
  capsule_length: { label: 'key button length' },
  capsule_end_radius: { label: 'key button end radius' },
  balance_mortise_cutter_diameter: { label: 'balance mortise cutter diameter' },
  balance_mortise_travel: { label: 'balance mortise cutter travel' },
  balance_mortise_floor: { label: 'balance mortise floor, above the underside' },
  balance_hole: { label: 'balance hole diameter' },
  bushing_thickness: { label: 'balance mortise bushing thickness' },
  bushing_depth: { label: 'balance mortise bushing, down from the key button’s top' },
  side_hole_height: { label: 'key button side hole height' },
  key_mass: { label: 'key mass' },
  front_weight: { label: 'front weight' },
  read_at: { label: 'read at', unit: (mm) => `${mm} from the lip` },
  weigh: { label: 'weigh the key' },
  balance_rail: { label: 'balance rail' },
  front_rail: { label: 'front rail' },
  back_rail: { label: 'back rail' },
  front_rail_at: { label: 'front rail, in front of the balance pin' },
  back_rail_at: { label: 'back rail, behind the balance pin' },
  place: { label: 'place the frame under the key' },
  weight: { label: 'weight on the measuring point' },
}

/** A weighing's field, as a sentence names it. */
const weighed = (field: Label): string =>
  field === 'key_mass'
    ? 'the key mass'
    : field === 'front_weight'
      ? 'the front weight'
      : field === 'read_at'
        ? 'the distance from the lip'
        : `the ${control[field].label}`

/** What holds the key, after "the key is". */
const verdict: Worded<Verdict> = {
  resting_on: ({ stop }) => ON_THE[stop],
  held_by_friction: ({ touching }) =>
    touching === null ? 'held by friction alone' : `${ON_THE[touching]}, held by friction`,
  rides_on_both_stops: ({ front, lift_mm }) =>
    `riding ${ON_THE[front]} and on the back rail cloth, ${to(lift_mm, LENGTH_MM)} mm above its balance punching`,
  lifts_off_balance_punching: ({ touching, needs_friction, lift_mm }) =>
    touching.length === 0
      ? `lifted ${to(lift_mm, LENGTH_MM)} mm off its balance punching, touching nothing`
      : needs_friction
        ? `lifted ${to(lift_mm, LENGTH_MM)} mm off its balance punching, on ${stopsNamed(touching)}, held there by its friction`
        : `lifted ${to(lift_mm, LENGTH_MM)} mm off its balance punching, held there by ${stopsNamed(touching)}`,
  lifts_off_its_frame: () =>
    'lifted off its frame, off its balance pin: no equilibrium in the model holds it there',
}

export const en: Catalog = {
  number: fixed,
  list,
  reasons,

  booting: 'starting the engine…',
  title: 'Single-key bench',
  tabTitle: 'Piano action bench — single key',
  languageSelector: 'Language',
  protocol:
    'Weighed by Stanwood’s *Standard Protocols of the New Touchweight Metrology*, February 2000. Every weight is centered on the measuring point, marked on the section, a set distance back from the front lip of the key.',

  view: {
    section: 'section',
    solid: 'solid',
    hint: 'drag to turn · scroll to zoom',
    bears: 'bears',
    open: 'open, under 1 mm',
    overlapping: 'drawn parts overlap',
    band: 'band',
    provenance: 'where each line comes from',
    how: {
      measured: 'measured',
      published: 'published',
      traced: 'traced from a figure',
      derived: 'derived by a rule',
      borrowed: 'borrowed from another instrument',
      assumed: 'assumed',
      set_on_the_bench: 'set on the bench',
    },
    keyOpacity: 'key opacity',
    notPlaced: {
      unplaced:
        'The key finds no equilibrium on this frame: it is drawn faded, level on its balance punching, to show its parts against the frame’s, not where it would stand.',
      lifted_off:
        'The weight on the measuring point lifts the key off its frame: it is drawn faded where its last equilibrium left it, which no longer holds it, and bears on nothing.',
    },
  },

  profile: {
    hirschkorn: 'Hirschkorn’s key, Boston GP-178, note 52, on real felts',
    'measured-section':
      'the measured key section on the measured key frame, its key front assumed, arriving unweighed',
  },

  faults: {
    heading: 'This frame does not fit this key.',
    placement:
      'The rails are placed under the key when the frame is built, and then they stay bolted where they are. Change the key afterwards and they do not follow — a real key frame does not either. *Place the frame under the key*, under The frame, unbolts them and stands them where this key needs them.',
  },
  fault: {
    pin_outside_mortise: ({ by_mm }) =>
      `the front rail pin stands ${to(by_mm, PART_MM)} mm outside the key's mortise`,
    cloth_in_front_of_chamfer: ({ by_mm }) =>
      `the back rail cloth reaches ${to(by_mm, PART_MM)} mm in front of the tail's chamfer, so the key can only meet it on a corner`,
    cloth_behind_key: ({ by_mm }) =>
      `the back rail cloth reaches ${to(by_mm, PART_MM)} mm past the end of the key`,
    punching_off_underside: ({ rail, by_mm }) =>
      `the ${PART[rail]} stands ${to(by_mm, PART_MM)} mm off the flat of the key's underside`,
  },
  noWeighing: (why) =>
    `No weighing is possible: ${why} The readings below that need the key seated on its frame are refused, each with why.`,
  stopsFight: (riding) =>
    `The stops fight: ${riding} The key is drawn riding on them; what needs no stroke is read below, and the weights that need one are refused.`,
  riding,
  problem,
  editRefusal,
  refused: {
    set: nothingChanged,
    add_paper: nothingChanged,
    remove_paper: nothingChanged,
    place_frame_under_key: nothingChanged,
    reset: nothingChanged,
    select_profile: nothingChanged,
    regulate: (why) => `Refused, the papers left as they were: ${word(editRefusal, why.code, why)}`,
    weigh_key: (why) =>
      `Refused, the key's weighing left as it was: ${word(editRefusal, why.code, why)}`,
    set_friction_from_key_friction_weight: (why) =>
      `Refused, the bushings left as they were: ${word(editRefusal, why.code, why)}`,
  },
  needsNumber: {
    regulate: (fields) => {
      const names = fields.map((f) => control[f].label)
      const one = names.length === 1
      return `${names.join(' and ')} ${one ? 'needs' : 'need'} a number, in millimeters: nothing is regulated until ${one ? 'it has' : 'they have'} one.`
    },
    weigh_key: (fields) => {
      const names = fields.map(weighed)
      return `${list(names)} ${names.length === 1 ? 'needs' : 'need'} a number: the key is weighed when all three have one.`
    },
    add_paper: null,
    remove_paper: null,
    place_frame_under_key: null,
    set_friction_from_key_friction_weight: null,
    reset: null,
    select_profile: null,
  },

  panel,
  control,

  sheet: {
    measuringPointNote:
      'The lip is the covering’s, which overhangs the key front. Moving the point changes every weight on this sheet by the lever and nothing about the key: the same static moment, and the same friction torque, read on another arm. Key height and dip are read at the key front, and do not move.',
    gramWeights:
      'Every weight is read the way gram weights give it: the down weight and the key friction weight as the least whole gram that takes the key down, the up weight as the most whole grams it lifts, balance and friction weight from those two, so to the half gram; the front weight to the tenth, as a scale reads it. The value in full stands small beside each. D is walked up from the key resting on its cloth, U down from the key on its front punching, each a gram at a time until the key goes all the way.',
    keyFrictionTwice: (jig, frame) =>
      `The key friction weight is read twice. Stanwood’s protocol zeroes the front weight on the leveled jig with a temporary weight${temporary(jig)}, then puts the key back on its frame and finds the least weight that makes it fall. The other cancels the key’s balance where it rests on its frame${temporary(frame)}, which reads the bushings alone. They differ by what the key’s tilt at rest does to the weights’ arms.`,
    indicative: (because) =>
      `These readings are indicative, not measurements: ${reasons(because)}.`,
    refused: 'refused',
    isRefused: (label, why) => `${label} is refused: ${why}`,
    target: (value) => `target ${value}`,
    seatingNote:
      'Both read at the front of the key with the key seated the way a technician seats it: the back of the key held down at the capstan, where the action holds it, for the height; the front then pressed down until a stop takes it, for the dip — the punching, or the pin in its mortise if that comes first. Whatever the weight below is doing to the key, these do not move.',
    keyMass: { label: 'key mass', unit: 'g' },
    outOfLevel: { label: 'out of level, key down', unit: '°' },
    bedding: { label: 'bedding at the back', unit: '°' },
    bearsAt: { label: 'bears, behind the balance pin', unit: 'mm' },
    bearsHow: (how) => `The key is **${how}**.`,
    bearingNote:
      'A keyboard is set up so the key is **level when it is fully down**, and so slightly nose-up when released — and the chamfer under the tail is cut at that same angle so the tail beds **flat** on its cloth. Both are design goals, not laws: rail heights, punchings and cloth are what reach them, and most keys rest on an **edge** instead, sometimes the cloth’s, sometimes the key’s own. For regulating that hardly matters: the papers under the balance punching set the key’s height, and the cloth’s work is to damp the key’s return and give it the same rest every time; bedded flat, the tail only wears its cloth more slowly. So the bench reports all three rather than refusing the frame. Nose-down is positive, and so is a rear end digging in.',
    balanceNote:
      'With no action on the capstan, **BW = −FW**: the equation of balance with its right-hand side removed, but for what the key’s rest attitude adds, read here at its two seats rather than level. A bare front-heavy key falls on its own, which is why these branches are negative.',
  },
  procedure: {
    front_weight: 'front weight FW',
    key_weight_ratio: 'key weight ratio KR',
    key_height: 'key height',
    key_dip: 'key dip',
    down_weight: 'down weight D',
    up_weight: 'up weight U',
    balance_weight: 'balance weight BW',
    friction_weight: 'friction weight F',
    key_friction_weight_zeroed_on_the_jig: 'key friction weight KF, zeroed on the jig',
    key_friction_weight_zeroed_on_the_frame: 'key friction weight KF, zeroed on the frame',
    down_weight_on_the_band: 'the band’s greatest force on the way down',
    up_weight_on_the_band: 'the band’s least force on the way up',
  },
  because: {
    key_mass: 'nobody has weighed the key',
    center_of_mass_height:
      'the height of the key’s center of mass, which no scale gives, is estimated from its drawn section',
    key_friction: 'its bushings’ friction is the bench’s assumption until you set it',
    geometry: 'some of its dimensions have no record',
  },
  unit: { g: 'g', mm: 'mm', ratio: '', gmm: UNIT.gmm },
  backBearing: {
    flat: 'flat on the cloth',
    on_the_cloth_edge: 'on the edge of the cloth',
    on_the_key_corner: "on the key's own corner",
  },
  refusal,

  regulation: {
    laid: (papers) =>
      `Laid ${to(papers.balance_mm, LENGTH_MM)} mm of paper under the balance punching and ${to(papers.front_mm, LENGTH_MM)} mm on the front punching: ${to(papers.key_height_mm, LENGTH_MM)} mm of key height and ${to(papers.key_dip_mm, LENGTH_MM)} mm of dip, as near ${to(papers.asked_key_height_mm, LENGTH_MM)} and ${to(papers.asked_key_dip_mm, LENGTH_MM)} mm as the drawer's papers come from below.`,
    why: regulationWhy,
    layingNote:
      'The bench lays the papers as a technician does: the key height first, on the balance punching, then the dip on the front punching with the key as it then stands, each the closest stack the drawer makes from below. So it lands within one paper of each target, not on it, and the sheet shows both. A target no stack reaches is refused, with what keeps it out and by how many millimeters, and nothing is changed.',
    pileNote:
      'A punching stack is a **pile of papers**, not a thickness. The operation is “lay a two-tenths”, never “set it to 0.37”, so these are the papers in the drawer and the pile you have built.',
  },

  papers: {
    stack: (title, totalMm, count) =>
      `${title} **${totalMm} mm** in ${count} ${count === 1 ? 'paper' : 'papers'}`,
    add: 'add',
    paper: (mm) => `${mm} mm`,
    liftOff: 'lift off',
    bare: 'bare rail',
  },

  friction: {
    why: frictionWhy,
    note: 'The key friction weight is typed as Stanwood’s protocol reads it, zeroed on the leveled jig: the bench sets the bushings to the torque at which that protocol, run on this key, reads back what is typed, and the sheet shows it. The bushings then stay as they are. Moving the measuring point or changing the key changes the grams they read, as it would at the bench, and this slider follows the sheet.',
  },

  punchings: {
    note: 'Paper is how you adjust, felt is what you fit. The rest geometry cares only about the total; the split will matter once the felts are allowed to compress.',
    stopsAtTheBottom: (byTheMortise) =>
      `The key stops on its front punching *or* on the pin reaching the top of its mortise, whichever comes first.${
        byTheMortise === null
          ? ''
          : byTheMortise
            ? ' Right now it is the pin: the dip below is shorter than the punchings alone would give.'
            : ' Right now it is the punching, as on a sane frame.'
      }`,
  },

  key: {
    ratioNote:
      'The key weight ratio is **not** a control. It is the ratio of two arms, so it follows from the geometry exactly as the action ratio does; a slider for it would have to pick which dimension to move. Changing a dimension does not re-solve the punchings either: the regulation drifts and the sheet shows the miss, because that is what happens on a real keyboard.',
  },

  weighing: {
    intro:
      'A key arrives leaded, with leads nobody knows the masses of, so it is weighed whole, its leads in it: type what the scale and the jig give for it.',
    why: weighingWhy,
    notWeighed:
      'Not weighed yet: this key has no mass in the model, so every reading that needs one is refused. Weigh it whole and type what the scale and the jig give: its mass, its front weight, and how far back from the lip the jig’s roller stood.',
    weighed: (massG, frontWeightG, fromLipMm) =>
      `Weighed: ${to(massG, MASS_G)} g, its front weight ${to(frontWeightG, MASS_G)} g read ${to(fromLipMm, FROM_LIP_MM)} mm back from the lip.`,
    published: (massG) =>
      `As published: ${massG === null ? 'its mass' : `${to(massG, MASS_G)} g`}, and its center of mass, its height included, as its source gives them.`,
    note: "The **key mass** is the whole key on a scale, capstan and leads included: the weight gravity pulls at its center of mass. The **front weight** is the leveled key's weight on the jig's scale, the key turning on its balance pin and the roller under the measuring point; times the arm it was read on, it is the key's static moment about the pin, which places its center of mass along the key. **Read at** is where the roller stood, back from the covering's lip: the arm it was read on. The bench keeps that static moment, so moving the measuring point afterwards, or changing the key, reads the same moment on another arm, and the front weight on the sheet follows by the lever. The height of the center of mass, which no scale gives, stays what the key came with: published, where its source gives one, or else estimated from the key's drawn section, which the sheet marks on every reading it reaches. It acts only through the key's tilt on its frame, a few hundredths of a gram of down weight per millimeter.",
  },

  frame: {
    railsNote:
      "And where the two outer rails **stand** along the key. These are the frame's own numbers, not the key's: a rail is part of the key frame, so lengthening a key moves its mortise and its chamfer and leaves the rails behind. That is a design worth seeing go wrong, which is why nothing here corrects itself — the sheet names the miss instead.",
    placeNote:
      'Moves the rails and **nothing else**. The key bears on the cloth at a new arm afterwards, so the key height and the dip will have drifted off their targets and the sheet will show it — shimming is a separate operation on a real bench too.',
  },

  band: {
    title: 'The band',
    caption: (stepMm) =>
      `The key driven slowly through its stroke, down and back, as a finger driving it would: at every ${stepMm} mm of travel at the measuring point, the force that holds it there. Between the two curves the key stays wherever it is, held by its friction: their distance apart is the friction, their middle the key’s balance. The most the way down needs anywhere and the least the way up holds are marked in full, to the hundredth of a gram: the curves’ own numbers, which no weight reads. The sheet’s **D** and **U** are the trade’s readings, gram weights walked from the seats and read as the weights give them.`,
    travel: 'travel at the measuring point, below the top seat (mm)',
    force: 'force on the measuring point, pressing down (g)',
    down: 'down curve: what drives the key down',
    up: 'up curve: what lets it rise',
    held: 'held by friction',
    weight: 'weight on the measuring point',
    key: 'where it has left the key',
    topSeat: 'top seat',
    bottomSeat: 'bottom seat',
    downMarker: (grams) => `greatest down ${grams} g`,
    upMarker: (grams) => `least up ${grams} g`,
    extremeAt: (label, travelMm) => `${label}, ${travelMm} mm below the top seat`,
    weightLine: (grams) => `${grams} g`,
    readout: (travelMm, downG, upG) => `${travelMm} mm · down ${downG} g · up ${upG} g`,
    seatsNote:
      'At a seat the stop takes whatever the finger leaves, so the curves start a step inside each seat and their extremes are read there, within a few thousandths of a gram of the sheet’s D and U in full, which walks gram weights from the seats themselves. Near a whole gram the two can fall either side of it, which is why the band reads no gram: the gram is the sheet’s.',
    stale: 'The key has changed since this band was drawn: choose **band** to draw it again.',
    staleRefusal:
      'The key has changed since the band was refused: choose **band** to try it again.',
    refused: (why) => `The band cannot be drawn: ${why}`,
    table:
      'The band, step by step: travel at the measuring point, and the force on the way down and on the way up',
  },

  touch: {
    intro:
      'Put a weight on the measuring point. Three things can happen, not two: above the down weight the key descends, below the up weight it rises, and **between them it does nothing at all**. That gap is the friction.',
    verdict,
    theKeyIs: (verdict) => `the key is ${verdict}`,
    margin: (marginG) =>
      `the weight lies between U and D, ${to(marginG, MASS_G)} g from moving the key`,
    gaps: (frontMm, backMm) => {
      const gap = (mm: number | null) => (mm === null ? 'none' : `${to(mm, GAP_MM)} mm`)
      return `gap at the front punching ${gap(frontMm)} · at the back rail cloth ${gap(backMm)}`
    },
    bushings: (kf) =>
      kf === null
        ? 'the bushings’ friction is an interval, not a value; Stanwood’s protocol cannot read it on this frame'
        : `the bushings’ friction is an interval, not a value: Stanwood’s protocol reads it as a key friction weight of ${kf} g`,
    centerNote: (overlaps) =>
      `The punchings are read at their center: a felt washer under a key carries the load over its whole face, and the bench takes it at the middle, under the balance pin for the balance punching. A key standing out of level on one overlaps the drawn felt's edge by what the felt would compress${
        overlaps.length === 0
          ? ''
          : `, here ${overlaps.map((overlap) => `${to(overlap.overlap_mm, LENGTH_MM)} mm at the ${CONTACT[overlap.pair]}`).join(' and ')}`
      }. The key is free to lift off its balance punching: where its stops fight it rides on them, and a pull its punching cannot answer lifts it off. Where an equilibrium places the key, the contacts marked on the view are where it bears, and what stands within a millimeter of it.`,
    notWeighed:
      'This key is not weighed: what a weight on it does depends on the key’s own balance, which only a weighing gives, so the key is drawn where its back rests on its cloth, as key height is read, whatever weight is typed. Weigh it to try the weights.',
    noPressNote:
      'There is no “press the key” control. The key is only ever where the physics puts it — a slider over a computed travel could park it in a position no weight produces, which is the one thing a measuring instrument must not do.',
  },
}
