/**
 * The bench in French: every sentence it shows, composed from the codes and
 * numbers the engine sends, in the trade's French.
 *
 * Its trade terms are the French names of
 * `docs/research/A3-nomenclature-and-regulation.md` > Bilingual glossary FR ↔ EN (+ DE) and code identifiers,
 * with Stanwood's English symbols beside them, as the glossary says. A
 * number is written with a decimal comma. French punctuation takes a
 * no-break space, U+00A0, before a colon, a semicolon, a question mark or an
 * exclamation mark, and inside guillemets: it is written as the character
 * itself.
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

/** `x` to `digits` places, with a decimal comma, never -0. */
function number(quantity: number, digits: number): string {
  return fixed(quantity, digits).replace('.', ',')
}

/** `x` to the decimals `resolution` is shown to. */
const to = (quantity: number, resolution: number): string => number(quantity, decimals(resolution))

/** Reasons as a note or a tooltip lists them, each a clause of its own. */
const reasons = (clauses: readonly string[]): string => clauses.join(' ; ')

/** Items as a sentence lists them: « a », « a et b », « a, b et c ». */
function list(terms: readonly string[]): string {
  return terms.length < 2
    ? (terms[0] ?? '')
    : `${terms.slice(0, -1).join(', ')} et ${terms[terms.length - 1]}`
}

/** A stop, as a sentence names it with the key's possessive: no comma inside
 * a name, so that a list of them reads as one. */
const ITS_STOP: Named<Contact> = {
  balance_rail_punching: 'sa mouche de balancier',
  front_rail_punching: 'sa mouche d’enfoncement',
  back_rail_cloth: 'son drap de repos arrière',
  front_pin_in_the_mortise: 'sa pointe d’enfoncement en haut de sa mortaise',
  front_pin_on_the_wood: 'sa pointe d’enfoncement contre le bois à côté de sa mortaise',
}

/** The stops, as a sentence lists them: « sa mouche d’enfoncement et son
 * drap de repos arrière »; « ses butées » where there are none. */
function stopsNamed(codes: readonly Contact[]): string {
  return codes.length === 0 ? 'ses butées' : list(codes.map((code) => ITS_STOP[code]))
}

/** Where a stop is, as the touch panel says the key rests on it. */
const ON_THE: Named<Contact> = {
  balance_rail_punching: 'sur la mouche de balancier',
  front_rail_punching: 'sur la mouche d’enfoncement',
  front_pin_in_the_mortise: 'sur la pointe d’enfoncement, en haut de sa mortaise',
  front_pin_on_the_wood: 'sur la pointe d’enfoncement, contre le bois autour de sa mortaise',
  back_rail_cloth: 'sur le drap de repos arrière',
}

/** A contact as a note names it in passing, where something is measured at
 * it. */
const AT_THE: Named<Contact> = {
  balance_rail_punching: 'à la mouche de balancier',
  front_rail_punching: 'à la mouche d’enfoncement',
  back_rail_cloth: 'au drap de repos arrière',
  front_pin_in_the_mortise: 'à la pointe d’enfoncement dans sa mortaise',
  front_pin_on_the_wood: 'à la pointe d’enfoncement sur le bois',
}

/** A part as a fault names it, with its article. */
const THE_PART: Named<Part> = {
  keystick: 'le corps de la touche',
  keytop: 'le revêtement de touche',
  key_front: 'le fronton',
  key_button: 'la capsule de touche',
  back_check_block: 'le bloc d’attrape',
  top_clearance: 'le dégagement',
  key_tail_felt: 'le feutre de bout de touche',
  capstan: 'le pilote',
  front_mortise: 'la mortaise d’enfoncement',
  balance_mortise: 'la mortaise de balancier',
  balance_mortise_bushing: 'la garniture de la mortaise de balancier',
  balance_hole: 'le trou de balancier',
  key_button_side_hole: 'le trou latéral de la capsule de touche',
  balance_rail: 'la barre de balancier',
  balance_rail_pin: 'la pointe de balancier',
  balance_rail_punching: 'la mouche de balancier',
  front_rail: 'la barre d’enfoncement',
  front_rail_pin: 'la pointe d’enfoncement',
  front_rail_punching: 'la mouche d’enfoncement',
  paper_punching: 'la mouche en papier',
  back_rail: 'la barre de repos arrière',
  back_rail_cloth: 'le drap de repos arrière',
}

/** A part after « de »: « du pilote », « de la capsule de touche ». */
function ofThe(part: Part): string {
  const the = THE_PART[part]
  return the.startsWith('le ') ? `du ${the.slice(3)}` : `de ${the}`
}

/** How the key rides on its stops, naming them and the lift. */
function riding(how: Riding): string {
  return `la touche est portée par ${stopsNamed([how.front, 'back_rail_cloth'])}, ${to(how.lift_mm, LENGTH_MM)} mm au-dessus de sa mouche de balancier : ses butées la tiennent plus haut que sa mouche, elle n’a donc pas de course, et aucun poids ne la fait passer d’une position à l’autre.`
}

/** How far past a target, in millimeters: to the hundredth, a shortfall
 * under that said as under a hundredth. */
function hundredths(mm: number): string {
  return mm < LENGTH_MM / 2 ? `moins de ${to(LENGTH_MM, LENGTH_MM)} mm` : `${to(mm, LENGTH_MM)} mm`
}

/** A regulation target, as a sentence names it, with its article. */
const A_TARGET: Named<Target> = {
  key_height: 'une hauteur de touche',
  key_dip: 'un enfoncement',
}

/** A static solve that reached its cap of steps, by which one it was. */
const NOT_CONVERGED: Named<Solve> = {
  seat: 'la touche n’a pas pu être posée sur son châssis dans le nombre maximal d’itérations du calcul statique.',
  walk: 'le calcul statique a atteint son nombre maximal d’itérations pendant la descente ou la remontée de la touche.',
  sweep:
    'le calcul statique a atteint son nombre maximal d’itérations sur un pas du tracé de la courbe d’enfoncement et de remontée.',
}

/** Which way the key was going. */
const STROKE: Named<Stroke> = {
  down: 'à la descente',
  up: 'à la remontée',
}

/** Each panel's heading, which a sentence pointing to the panel quotes. */
const panel: Catalog['panel'] = {
  instrument: 'L’instrument',
  sheet: 'Relevé du poids de jeu',
  regulation: 'Réglage',
  punchings: 'Mouches et drap',
  key: 'La touche',
  weighing: 'La touche, pesée entière',
  frame: 'Le châssis',
  touch: 'Toucher',
}

/** Why a reading is refused, or why the key cannot be seated. */
const refusal: Catalog['refusal'] = {
  geometry_degenerate: () => 'la géométrie est dégénérée.',
  key_not_weighed: () =>
    `personne n’a pesé cette touche, le banc d’essai ne lui connaît donc aucune masse : pesez-la entière et saisissez la pesée sous « ${panel.weighing} ».`,
  scale_not_in_front_of_the_pivot: () =>
    'le point de mesure se trouve à l’aplomb de la pointe de balancier ou en arrière, là où la balance de l’appareil de mesure à faible friction ne peut pas tenir la touche de niveau.',
  cannot_seat: ({ fighting }) =>
    `la touche n’a pas de position sur ce châssis : ${stopsNamed(fighting)} ${fighting.length === 1 ? 'pénètre' : 'pénètrent'} dans le dessin de la touche, et le modèle ne trouve aucune position de la touche sur son châssis qui en soit dégagée.`,
  lifts_off_its_frame: ({ leaves_its_pin_mm }) =>
    leaves_its_pin_mm < 0
      ? `la mouche de balancier se trouve ${to(-leaves_its_pin_mm, LENGTH_MM)} mm au-dessus du sommet de la pointe de balancier : une touche posée dessus est hors de sa pointe, et hors de son châssis.`
      : `ses butées tiendraient la touche à plus de ${to(leaves_its_pin_mm, LENGTH_MM)} mm au-dessus de sa mouche de balancier, hauteur où elle quitte sa pointe de balancier : hors de sa pointe, et hors de son châssis.`,
  no_stop_below_the_front: () =>
    'rien n’arrête l’avant de la touche : le centre de la mouche d’enfoncement est en dehors du dessous de la touche, qui ne porte donc sur rien en fond de course.',
  no_stop_under_the_tail: () =>
    'rien ne retient l’arrière de la touche : le drap de repos arrière se trouve là où le bout de touche ne peut pas l’atteindre.',
  capstan_not_behind_the_bearing: () =>
    'le pilote se trouve à l’aplomb du support de touche au balancier ou en avant, là où la mécanique ne peut pas retenir l’arrière de la touche.',
  measuring_point_not_in_front: () =>
    'le point de mesure se trouve à l’aplomb du support de touche au balancier ou en arrière, là où un poids ne peut pas faire descendre la touche.',
  not_converged: ({ solve }) => NOT_CONVERGED[solve],
  no_travel: ({ riding: ridingOn }) =>
    ridingOn === null
      ? 'la touche n’a pas de course : au repos, elle est déjà en fond de course.'
      : riding(ridingOn),
  no_lever: () => 'un poids sur le point de mesure ne fait pas tourner la touche.',
  does_not_reach_the_end_of_its_stroke: ({ cap_g }) =>
    `un poids de ${to(cap_g, WEIGHT_G)} g sur le point de mesure n’a pas mené la touche en fond de course.`,
  no_equilibrium: () =>
    'la touche n’a trouvé aucun équilibre à la descente ni à la remontée : ses butées s’opposent.',
  walk_lifts_off_its_frame: () =>
    'un poids posé sur le point de mesure pendant la recherche gramme par gramme soulève la touche hors de son châssis.',
  sweep_lifts_off_its_punching: ({ stroke, travel_mm, force_g }) =>
    `à ${to(travel_mm, LENGTH_MM)} mm de course ${STROKE[stroke]}, un doigt devrait exercer ${force_g < 0 ? 'une traction' : 'une pression'} de ${to(Math.abs(force_g), WEIGHT_G)} g au point de mesure, et la mouche de balancier ne porterait plus rien : la touche se soulèverait de sa mouche au lieu de tourner, et la force ne serait donc pas celle qu’on lit.`,
  sweep_lifts_off_its_frame: () =>
    'un pas du tracé de la courbe a sorti la touche de sa pointe de balancier, et de son châssis.',
  no_key_top_at_the_station: ({ station_mm }) =>
    `aucun dessus de touche ne se trouve à ${to(Math.abs(station_mm), STATION_MM)} mm ${station_mm < 0 ? 'en arrière' : 'en avant'} de la pointe de balancier, où va le poids provisoire.`,
  from_a_refused_reading: ({ reading }) =>
    `cette mesure se calcule à partir ${reading === 'down_weight' ? 'du poids d’enfoncement, refusé' : reading === 'up_weight' ? 'du poids de remontée, refusé' : 'd’une mesure refusée'}.`,
}

/** The temporary weight a key friction weight stood on the key. */
function temporary(weight: TemporaryWeight | null): string {
  if (weight === null) return ''
  return ` (${to(weight.mass_g, TEMPORARY_WEIGHT_G)} g, à ${to(Math.abs(weight.at_mm), STATION_MM)} mm ${weight.at_mm < 0 ? 'en arrière' : 'en avant'} de la pointe de balancier)`
}

/** Nothing under the front of the key, which keeps either target out of
 * reach for the same reason. */
const NO_STOP_BELOW_THE_FRONT =
  'rien n’arrête l’avant de la touche, le centre de la mouche d’enfoncement étant en dehors de son dessous'

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
      `il faudrait plus de ${to(searchMm, SEARCH_MM)} mm de mouches en papier sous la mouche de balancier, la barre de balancier étant trop basse`,
    key_dip: (searchMm) =>
      `il faudrait plus de ${to(searchMm, SEARCH_MM)} mm de mouches en papier sous la mouche d’enfoncement, la barre d’enfoncement étant trop basse`,
  },
  front_pin_in_mortise: {
    key_height: () =>
      'la pointe d’enfoncement arriverait en haut de sa mortaise avec la touche aussi basse au repos',
    key_dip: () =>
      'la pointe d’enfoncement arrive en haut de sa mortaise avant que la touche ne descende aussi bas',
  },
  front_pin_on_the_wood: {
    key_height: () =>
      'la pointe d’enfoncement se trouve sous le bois de la touche, et seule une touche relevée au-dessus d’elle a une position',
    key_dip: () =>
      'la pointe d’enfoncement rencontre le bois de la touche à côté de sa mortaise avant que la touche ne descende aussi bas',
  },
  felt_thicker_than_the_stack: {
    key_height: () =>
      'le feutre de la mouche de balancier, seul, sans mouche en papier dessous, porte déjà la touche plus haut',
    key_dip: () =>
      'le feutre de la mouche d’enfoncement, seul, sans mouche en papier dessous, arrête déjà la touche plus tôt',
  },
  stops_fight: {
    key_height: () =>
      'l’avant de la touche descendrait sur sa mouche d’enfoncement au repos, le bout de touche sur son drap',
    key_dip: () =>
      'la mouche d’enfoncement toucherait la touche au repos avant un enfoncement aussi faible',
  },
  no_stop_below_the_front: {
    key_height: () => NO_STOP_BELOW_THE_FRONT,
    key_dip: () => NO_STOP_BELOW_THE_FRONT,
  },
  off_its_balance_pin: {
    key_height: () =>
      'la touche quitterait d’abord sa pointe de balancier, les mouches en papier montant sa mouche de balancier au-dessus du sommet de la pointe',
    key_dip: () =>
      'la touche sortirait de sa pointe de balancier avant un enfoncement aussi faible',
  },
}

const regulationWhy: Catalog['regulation']['why'] = {
  regulation_infeasible: ({ target, wanted_mm, limited_by, shortfall_mm, search_mm }) =>
    `${A_TARGET[target]} de ${to(wanted_mm, LENGTH_MM)} mm est hors d’atteinte, il s’en faut de ${hundredths(shortfall_mm)} : ${LIMIT_WHY[limited_by][target](search_mm)}.`,
  no_seat: ({ why }) =>
    `la touche n’a pas pu être posée sur son châssis pendant l’essai des mouches : ${word(refusal, why.code, why)}`,
}

/** Why a key friction weight typed was refused. */
const frictionWhy: Worded<FrictionRefusal> = {
  protocol_refused: ({ why }) =>
    `le protocole Stanwood ne peut pas mesurer la friction de cette touche : ${word(refusal, why.code, why)}`,
  under_no_friction: ({ typed_g, least_g }) =>
    `${to(typed_g, MASS_G)} g, c’est moins que ce que le protocole Stanwood lit sur cette touche sans aucune friction, ${to(least_g, MASS_G)} g : ce que l’inclinaison de la touche au repos laisse de son propre équilibre et de celui du poids provisoire.`,
  beyond_the_search: ({ typed_g, most_g, holds_g }) =>
    `${to(typed_g, MASS_G)} g, c’est plus que ce que lisent les garnitures les plus serrées que le banc d’essai tente, ${to(most_g, MASS_G)} g : elles retiennent ${to(holds_g, WEIGHT_G)} g au point de mesure, la touche de niveau, et tiendraient la touche partout où on la poserait.`,
}

/** Why a weighing of the key was refused. */
const weighingWhy: Worded<WeighingRefusal> = {
  roller_off_the_covering: ({ from_lip_mm, covering_mm }) =>
    `un poids avant se lit sur le revêtement, en avant de la pointe de balancier, de son bord jusqu’à ${to(covering_mm, FROM_LIP_MM)} mm en arrière : à ${to(from_lip_mm, FROM_LIP_MM)} mm du bord, on est hors du revêtement.`,
  center_of_mass_off_the_key: ({ at_mm }) =>
    `ce poids avant sur cette masse place le centre de gravité de la touche à ${to(Math.abs(at_mm), STATION_MM)} mm ${at_mm < 0 ? 'en arrière' : 'en avant'} de la pointe de balancier, au-delà ${at_mm < 0 ? 'de son bout de touche' : 'du bord de son revêtement'} : hors de la touche.`,
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
    ? `le bout aminci de la touche et son feutre finiraient ${by} mm en avant de l’extrémité arrière ${ofThe(front)}, et le bloc est collé là où le corps de la touche a toute sa hauteur`
    : front === 'keystick'
      ? `${THE_PART[back]} chevaucherait de ${by} mm l’épaulement vers le bout aminci de la touche, et un bloc se pose d’un côté ou de l’autre`
      : back === 'keystick' && front === 'keytop'
        ? `le bout aminci de la touche passerait ${by} mm sous le revêtement de touche, qui est collé là où le corps de la touche a toute sa hauteur`
        : front === 'capstan'
          ? `${THE_PART[back]} passerait ${by} mm sous la tête du pilote, qui se visse là où le corps de la touche a toute sa hauteur`
          : `${THE_PART[back]} et ${THE_PART[front]} se chevaucheraient de ${by} mm le long de la touche`

/** A setting's unit, as a value in it is written. */
const UNIT: Named<SettingUnit> = { mm: 'mm', g: 'g', gmm: 'g·mm' }

/** What no part could be built as, or what the bench has not. */
const problem: Worded<Problem> = {
  not_finite: () => 'une valeur qui n’est pas un nombre.',
  out_of_range: ({ param, value, min, max }) => {
    const unit = unitIn(param, UNIT)
    return `${to(value, OUT_OF_RANGE)}${unit} n’est pas une valeur possible pour cette pièce${min === null ? '' : `, pas moins de ${to(min, OUT_OF_RANGE)}${unit}`}${max === null ? '' : `${min === null ? ',' : ' et'} pas plus de ${to(max, OUT_OF_RANGE)}${unit}`}.`
  },
  not_in_the_drawer: ({ thickness_mm, drawer_mm }) =>
    `aucun assortiment ne contient de mouche de ${to(thickness_mm, LENGTH_MM)} mm : celui-ci contient des mouches de ${list(drawer_mm.map((mm) => `${to(mm, LENGTH_MM)} mm`))}.`,
  mass_not_positive: () => 'une touche pèse plus que rien.',
  outline_crosses_itself: ({ back, front, by_mm }) =>
    `${OVERLAP(back, front, to(by_mm, PART_MM))} : aucune touche n’est taillée ainsi.`,
  feature_outside_keystick: ({ part, by_mm }) =>
    part === 'top_clearance'
      ? `le dégagement s’étendrait de ${to(by_mm, PART_MM)} mm vers l’arrière sur le bout aminci de la touche : il se taille là où le corps de la touche a toute sa hauteur.`
      : `${THE_PART[part]} se trouverait en dehors du plat du dessous de la touche, de ${to(by_mm, PART_MM)} mm.`,
  capstan_behind_back_check_block: ({ by_mm }) =>
    `le pilote se trouverait derrière le bloc d’attrape : sa place est entre le bloc et la capsule de touche, et il dégage le bloc ${to(by_mm, PART_MM)} mm plus en avant.`,
  capstan_in_block: ({ block, by_mm }) =>
    `le pilote entrerait de ${to(by_mm, PART_MM)} mm dans ${THE_PART[block]}.`,
  capstan_off_the_full_height: ({ by_mm }) =>
    `le pilote se trouverait au-dessus du bout aminci de la touche, et il se visse là où le corps de la touche a toute sa hauteur : sa tête y est tout entière ${to(by_mm, PART_MM)} mm plus en avant.`,
  capstan_dome_narrower_than_its_head: ({ dome_mm, head_mm }) =>
    `un dôme de ${to(dome_mm, LENGTH_MM)} mm ne peut pas coiffer une tête de pilote de ${to(head_mm, LENGTH_MM)} mm de rayon.`,
  clearance_not_a_cut: ({ length_mm, depth_mm }) =>
    `un dégagement de ${to(length_mm, PART_MM)} mm de long et ${to(depth_mm, PART_MM)} mm de profondeur n’entaille rien : il se taille sous le dessus du corps de la touche, là où il a toute sa hauteur, son extrémité avant en avant de son extrémité arrière.`,
  clearance_through_keystick: ({ depth_mm, wood_mm }) =>
    `un dégagement de ${to(depth_mm, PART_MM)} mm de profondeur traverse le corps de la touche, qui n’a que ${to(wood_mm, PART_MM)} mm de bois à cet endroit.`,
  capsule_ends_overlap: ({ by_mm }) =>
    `les bouts arrondis de la capsule de touche empiéteraient de ${to(by_mm, PART_MM)} mm l’un sur l’autre sur son dessus : ils se rejoignent au plus.`,
  does_not_fit: ({ part, within, by_mm }) =>
    `${THE_PART[part]} ne tiendrait pas dans ${THE_PART[within]} : ${to(by_mm, PART_MM)} mm de trop.`,
  mortise_through_key_top: ({ depth_mm, wood_mm }) =>
    `une mortaise d’enfoncement de ${to(depth_mm, PART_MM)} mm de profondeur traverse les ${to(wood_mm, PART_MM)} mm du corps de la touche.`,
  measuring_point_off_covering: ({ from_lip_mm, covering_mm }) =>
    `le point de mesure se place sur le revêtement, de son bord jusqu’à ${to(covering_mm, FROM_LIP_MM)} mm en arrière : à ${to(from_lip_mm, FROM_LIP_MM)} mm du bord, on est hors du revêtement.`,
  unknown_path: ({ path }) => `le banc d’essai n’a pas de réglage ${path}.`,
  unknown_profile: ({ profile }) => `le banc d’essai ne propose pas d’instrument ${profile}.`,
  set_by_action: ({ path }) =>
    `${path} ne se saisit pas directement : il s’obtient en posant des mouches, en réglant ou en pesant.`,
  no_such_paper: ({ index, papers }) =>
    `la pile compte ${papers} ${papers < 2 ? 'mouche' : 'mouches'} : il n’y a pas de mouche n° ${index + 1} à retirer.`,
  not_an_action: () => 'le banc d’essai ne connaît pas cette opération.',
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
  `Refusé, rien n’a changé : ${word(editRefusal, why.code, why)}`

/** What each control is worded by, by its label's code. */
const control: { readonly [K in Label]: Labeled } = {
  profile: { label: 'sur le banc d’essai' },
  measuring_point: { label: 'point de mesure, en arrière du bord avant du revêtement' },
  key_height: { label: 'hauteur de touche' },
  key_dip: { label: 'enfoncement' },
  regulate: { label: 'régler à ces cotes' },
  balance_stack: { label: 'mouche de balancier — hauteur de touche' },
  front_stack: { label: 'mouche d’enfoncement — enfoncement' },
  key_friction_weight: { label: 'friction de la touche' },
  balance_felt: { label: 'mouche de balancier en feutre' },
  front_felt: { label: 'mouche d’enfoncement en feutre' },
  front_pin: { label: 'pointe d’enfoncement, au-dessus de la barre' },
  balance_pin: { label: 'pointe de balancier, au-dessus de la barre' },
  front_mortise: { label: 'mortaise d’enfoncement, depuis le dessous de la touche' },
  front_mortise_cutter_diameter: { label: 'diamètre de la fraise de la mortaise d’enfoncement' },
  front_mortise_travel: { label: 'course de la fraise de la mortaise d’enfoncement' },
  back_cloth: { label: 'drap de repos arrière' },
  capstan: { label: 'pilote, en arrière de la pointe de balancier' },
  front_edge: { label: 'fronton, en avant de la pointe de balancier' },
  capstan_height: { label: 'hauteur du pilote' },
  capsule_height: { label: 'hauteur de la capsule de touche' },
  capsule_length: { label: 'longueur de la capsule de touche' },
  capsule_end_radius: { label: 'rayon des bouts de la capsule de touche' },
  balance_mortise_cutter_diameter: { label: 'diamètre de la fraise de la mortaise de balancier' },
  balance_mortise_travel: { label: 'course de la fraise de la mortaise de balancier' },
  balance_mortise_floor: {
    label: 'fond de la mortaise de balancier, depuis le dessous de la touche',
  },
  balance_hole: { label: 'diamètre du trou de balancier' },
  bushing_thickness: { label: 'épaisseur de la garniture de la mortaise de balancier' },
  bushing_depth: {
    label: 'garniture de la mortaise de balancier, depuis le haut de la capsule de touche',
  },
  side_hole_height: { label: 'hauteur du trou latéral de la capsule de touche' },
  key_mass: { label: 'masse de la touche' },
  front_weight: { label: 'poids avant' },
  read_at: { label: 'lu à', unit: (mm) => `${mm} du bord` },
  weigh: { label: 'peser la touche' },
  balance_rail: { label: 'barre de balancier' },
  front_rail: { label: 'barre d’enfoncement' },
  back_rail: { label: 'barre de repos arrière' },
  front_rail_at: { label: 'barre d’enfoncement, en avant de la pointe de balancier' },
  back_rail_at: { label: 'barre de repos arrière, en arrière de la pointe de balancier' },
  place: { label: 'placer le châssis sous la touche' },
  weight: { label: 'poids sur le point de mesure' },
}

/** A form's field, as a sentence names it, with its article. */
const theField = (field: Label): string =>
  field === 'key_height'
    ? 'la hauteur de touche'
    : field === 'key_dip'
      ? 'l’enfoncement'
      : field === 'key_mass'
        ? 'la masse de la touche'
        : field === 'front_weight'
          ? 'le poids avant'
          : field === 'read_at'
            ? 'la distance au bord'
            : `« ${control[field].label} »`

/** What holds the key, after « la touche ». */
const verdict: Worded<Verdict> = {
  resting_on: ({ stop }) => `repose ${ON_THE[stop]}`,
  held_by_friction: ({ touching }) =>
    touching === null
      ? 'est tenue par la seule friction'
      : `repose ${ON_THE[touching]}, tenue par la friction`,
  rides_on_both_stops: ({ front, lift_mm }) =>
    `est portée ${ON_THE[front]} et sur le drap de repos arrière, ${to(lift_mm, LENGTH_MM)} mm au-dessus de sa mouche de balancier`,
  lifts_off_balance_punching: ({ touching, needs_friction, lift_mm }) =>
    touching.length === 0
      ? `est soulevée de ${to(lift_mm, LENGTH_MM)} mm de sa mouche de balancier, sans rien toucher`
      : needs_friction
        ? `est soulevée de ${to(lift_mm, LENGTH_MM)} mm de sa mouche de balancier, sur ${stopsNamed(touching)}, où sa friction la retient`
        : `est soulevée de ${to(lift_mm, LENGTH_MM)} mm de sa mouche de balancier, retenue par ${stopsNamed(touching)}`,
  lifts_off_its_frame: () =>
    'est soulevée hors de son châssis, hors de sa pointe de balancier : aucun équilibre du modèle ne la tient là',
}

export const fr: Catalog = {
  number,
  list,
  reasons,

  booting: 'démarrage du moteur…',
  title: 'Banc d’essai d’une touche',
  tabTitle: 'Banc d’essai de mécanique de piano à queue — une touche',
  languageSelector: 'Langue',
  protocol:
    'Pesée selon les *Standard Protocols of the New Touchweight Metrology* de Stanwood, février 2000. Chaque poids est centré sur le point de mesure, marqué sur la coupe, à une distance fixe en arrière du bord avant du revêtement.',

  view: {
    section: 'coupe',
    solid: '3D',
    hint: 'glisser pour tourner · molette pour zoomer',
    bears: 'porte',
    open: 'libre, à moins de 1 mm',
    overlapping: 'pièces dessinées qui se chevauchent',
    band: 'courbe d’enfoncement et de remontée',
    provenance: 'd’où vient chaque trait',
    how: {
      measured: 'mesuré',
      published: 'publié',
      traced: 'relevé sur une figure',
      derived: 'déduit par une règle',
      borrowed: 'emprunté à un autre instrument',
      assumed: 'supposé',
      set_on_the_bench: 'réglé sur le banc d’essai',
    },
    keyOpacity: 'opacité de la touche',
    notPlaced: {
      unplaced:
        'La touche ne trouve aucune position d’équilibre sur ce châssis : elle est dessinée en transparence, à plat sur sa mouche de balancier, pour montrer ses pièces face à celles du châssis, pas là où elle se trouverait.',
      lifted_off:
        'Le poids sur le point de mesure soulève la touche hors de son châssis : elle est dessinée en transparence là où l’a laissée sa dernière position d’équilibre, qui ne la tient plus, et ne porte sur rien.',
    },
  },

  profile: {
    hirschkorn: 'Touche de Hirschkorn, Boston GP-178, note 52, sur de vrais feutres',
    'measured-section':
      'la coupe de touche mesurée, sur le châssis mesuré, son fronton supposé, livrée non pesée',
  },

  faults: {
    heading: 'Ce châssis ne va pas avec cette touche.',
    placement: `Les barres sont placées sous la touche quand le châssis est construit, puis elles restent fixées là où elles sont. Si l’on change la touche ensuite, elles ne suivent pas, sur un vrai châssis non plus. *Placer le châssis sous la touche*, sous « ${panel.frame} », les détache et les place là où cette touche en a besoin.`,
  },
  fault: {
    pin_outside_mortise: ({ by_mm }) =>
      `la pointe d’enfoncement se trouve à ${to(by_mm, PART_MM)} mm en dehors de la mortaise de la touche`,
    cloth_in_front_of_chamfer: ({ by_mm }) =>
      `le drap de repos arrière dépasse de ${to(by_mm, PART_MM)} mm en avant du chanfrein du bout de touche, si bien que la touche ne peut le toucher que sur une arête`,
    cloth_behind_key: ({ by_mm }) =>
      `le drap de repos arrière dépasse de ${to(by_mm, PART_MM)} mm au-delà du bout de la touche`,
    punching_off_underside: ({ rail, by_mm }) =>
      `${THE_PART[rail]} se trouve à ${to(by_mm, PART_MM)} mm en dehors du plat du dessous de la touche`,
  },
  noWeighing: (why) =>
    `Aucune pesée n’est possible : ${why} Les mesures ci-dessous qui demandent la touche en place sur son châssis sont refusées, chacune avec sa raison.`,
  stopsFight: (riding) =>
    `Les butées s’opposent : ${riding} La touche est dessinée portée par elles ; ce qui ne demande pas de course est mesuré ci-dessous, et les poids qui en demandent une sont refusés.`,
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
    regulate: (why) =>
      `Refusé, les mouches restent telles quelles : ${word(editRefusal, why.code, why)}`,
    weigh_key: (why) =>
      `Refusé, la pesée de la touche reste telle quelle : ${word(editRefusal, why.code, why)}`,
    set_friction_from_key_friction_weight: (why) =>
      `Refusé, les garnitures restent telles quelles : ${word(editRefusal, why.code, why)}`,
  },
  needsNumber: {
    regulate: (fields) =>
      `il manque un nombre, en millimètres, pour ${list(fields.map(theField))} : rien n’est réglé tant que ${fields.length === 1 ? 'ce champ est vide' : 'ces champs sont vides'}.`,
    weigh_key: (fields) =>
      `il manque un nombre pour ${list(fields.map(theField))} : la touche est pesée quand les trois champs en ont un.`,
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
      'Le bord est celui du revêtement, qui déborde du fronton. Déplacer le point change chaque poids de ce relevé par le levier, et rien de la touche : le même moment statique, et le même couple de friction, lus sur un autre bras. La hauteur de touche et l’enfoncement se lisent au fronton, et ne bougent pas.',
    gramWeights:
      'Chaque poids se lit comme le donnent les poids de pesée : le poids d’enfoncement et la friction de la touche comme le plus petit nombre entier de grammes qui fait descendre la touche, le poids de remontée comme le plus grand nombre entier de grammes qu’elle soulève en remontant, le poids d’équilibre et la friction à partir de ces deux-là, donc au demi-gramme ; le poids avant au dixième, comme le lit une balance. La valeur complète figure en petit à côté de chacun. D se cherche en augmentant le poids depuis la touche au repos sur son drap, U en le diminuant depuis la touche sur sa mouche d’enfoncement, gramme par gramme jusqu’à ce que la touche aille au bout de sa course.',
    keyFrictionTwice: (jig, frame) =>
      `La friction de la touche se lit deux fois. Le protocole Stanwood ramène le poids avant à zéro sur l’appareil de mesure à faible friction mis de niveau, avec un poids provisoire${temporary(jig)}, puis remet la touche sur son châssis et cherche le plus petit poids qui la fait descendre. L’autre lecture annule l’équilibre de la touche là où elle repose sur son châssis${temporary(frame)}, ce qui lit les garnitures seules. Les deux diffèrent de ce que l’inclinaison de la touche au repos fait aux bras de levier des poids.`,
    indicative: (because) =>
      `Ces valeurs sont indicatives, ce ne sont pas des mesures : ${reasons(because)}.`,
    refused: 'refusé',
    isRefused: (label, why) => `${label} : mesure refusée, car ${why}`,
    target: (value) => `cote ${value}`,
    seatingNote:
      'Les deux se lisent à l’avant de la touche, la touche placée comme la place un technicien : l’arrière de la touche tenu au pilote, là où la mécanique le tient, pour la hauteur ; puis l’avant enfoncé jusqu’à ce qu’une butée l’arrête, pour l’enfoncement, la mouche, ou la pointe en haut de sa mortaise si elle arrive la première. Quoi que fasse à la touche le poids ci-dessous, ces deux valeurs ne bougent pas.',
    keyMass: { label: 'masse de la touche', unit: 'g' },
    outOfLevel: { label: 'écart de niveau, touche enfoncée', unit: '°' },
    bedding: { label: 'assise du bout de touche', unit: '°' },
    bearsAt: { label: 'point d’appui sur le drap, derrière la pointe de balancier', unit: 'mm' },
    bearsHow: (how) => `La touche porte **${how}**.`,
    bearingNote:
      'Un clavier se règle pour que la touche soit **de niveau quand elle est enfoncée à fond**, donc légèrement relevée à l’avant quand elle est relâchée, et le chanfrein sous le bout de touche est taillé à ce même angle pour que le bout de touche porte **à plat** sur son drap. Ce sont deux objectifs de conception, pas des lois : les hauteurs des barres, les mouches et le drap sont ce qui les atteint, et la plupart des touches reposent plutôt sur un bord, tantôt sur le **bord du drap**, tantôt sur une **arête de la touche**. Pour le réglage, cela compte peu : les mouches en papier sous la mouche de balancier fixent la hauteur de la touche, et le drap sert à amortir le retour de la touche et à lui donner le même repos à chaque fois ; portant à plat, le bout de touche use seulement son drap plus lentement. Le banc d’essai indique donc les trois valeurs plutôt que de refuser le châssis. Un avant qui plonge compte en positif, de même qu’un arrière qui s’enfonce dans le drap.',
    balanceNote:
      'Sans mécanique sur le pilote, **BW = −FW** : c’est l’équilibre des poids sans son membre de droite, à l’effet près de l’inclinaison de la touche au repos, lue ici à ses deux positions et non de niveau. Une touche nue plus lourde à l’avant descend d’elle-même, d’où ces valeurs négatives.',
  },
  procedure: {
    front_weight: 'poids avant FW',
    key_weight_ratio: 'rapport de poids de clavier KR',
    key_height: 'hauteur de touche',
    key_dip: 'enfoncement',
    down_weight: 'poids d’enfoncement D',
    up_weight: 'poids de remontée U',
    balance_weight: 'poids d’équilibre BW',
    friction_weight: 'friction F',
    key_friction_weight_zeroed_on_the_jig:
      'friction de la touche KF, touche équilibrée sur l’appareil de mesure à faible friction',
    key_friction_weight_zeroed_on_the_frame:
      'friction de la touche KF, touche équilibrée sur son châssis',
    down_weight_on_the_band: 'la plus grande force de la courbe d’enfoncement',
    up_weight_on_the_band: 'la plus petite force de la courbe de remontée',
  },
  because: {
    key_mass: 'personne n’a pesé la touche',
    center_of_mass_height:
      'la hauteur du centre de gravité de la touche, qu’aucune balance ne donne, est estimée d’après sa coupe dessinée',
    key_friction:
      'la friction de ses garnitures est une hypothèse du banc d’essai tant que vous ne la réglez pas',
    geometry: 'certaines de ses dimensions n’ont pas de source',
  },
  unit: { g: 'g', mm: 'mm', ratio: '', gmm: UNIT.gmm },
  backBearing: {
    flat: 'à plat sur le drap',
    on_the_cloth_edge: 'sur le bord du drap',
    on_the_key_corner: 'sur une arête de la touche',
  },
  refusal,

  regulation: {
    laid: (papers) =>
      `${to(papers.balance_mm, LENGTH_MM)} mm de mouches en papier posés sous la mouche de balancier et ${to(papers.front_mm, LENGTH_MM)} mm sous la mouche d’enfoncement : ${to(papers.key_height_mm, LENGTH_MM)} mm de hauteur de touche et ${to(papers.key_dip_mm, LENGTH_MM)} mm d’enfoncement, pour ${to(papers.asked_key_height_mm, LENGTH_MM)} et ${to(papers.asked_key_dip_mm, LENGTH_MM)} mm demandés, chaque pile étant la plus proche que l’assortiment permet par en dessous.`,
    why: regulationWhy,
    layingNote:
      'Le banc d’essai pose les mouches comme un technicien : la hauteur de touche d’abord, à la mouche de balancier, puis l’enfoncement à la mouche d’enfoncement, la touche telle qu’elle se trouve alors, chaque fois avec la pile la plus proche que l’assortiment permet par en dessous. Il arrive donc à moins d’une mouche de chaque cote, pas pile dessus, et le relevé montre les deux. Une cote qu’aucune pile n’atteint est refusée, avec ce qui l’en empêche et de combien de millimètres, et rien n’est changé.',
    pileNote:
      'Sous une mouche, on empile des **mouches en papier** : c’est une pile, pas une épaisseur. On dit « ajouter une mouche de 0,2 », jamais « régler à 0,37 » : voici donc les mouches de l’assortiment et la pile que vous avez montée.',
  },

  papers: {
    stack: (title, totalMm, count) =>
      `${title} **${totalMm} mm** en ${count} ${count < 2 ? 'mouche' : 'mouches'}`,
    add: 'ajouter',
    paper: (mm) => `${mm} mm`,
    liftOff: 'retirer',
    bare: 'aucune mouche en papier',
  },

  friction: {
    why: frictionWhy,
    note: 'La friction de la touche se saisit telle que la lit le protocole Stanwood, touche équilibrée sur l’appareil de mesure à faible friction mis de niveau : le banc d’essai règle les garnitures au couple pour lequel ce protocole, appliqué à cette touche, relit ce qui est saisi, et le relevé l’affiche. Les garnitures restent ensuite telles quelles. Déplacer le point de mesure ou changer la touche change les grammes qu’elles donnent, comme à l’établi, et ce curseur suit le relevé.',
  },

  punchings: {
    note: 'Le papier sert à régler, le feutre se monte. La géométrie au repos ne voit que le total ; la répartition comptera quand les feutres pourront se comprimer.',
    stopsAtTheBottom: (byTheMortise) =>
      `La touche s’arrête sur sa mouche d’enfoncement *ou* sur la pointe arrivée en haut de sa mortaise, selon ce qui vient en premier.${
        byTheMortise === null
          ? ''
          : byTheMortise
            ? ' Ici, c’est la pointe : l’enfoncement ci-dessous est plus court que ne le donneraient les mouches seules.'
            : ' Ici, c’est la mouche, comme sur un châssis sain.'
      }`,
  },

  key: {
    ratioNote:
      'Le rapport de poids de clavier n’est **pas** un réglage. C’est le rapport de deux bras de levier, il découle donc de la géométrie, exactement comme le rapport mécanique ; un curseur pour lui devrait choisir quelle dimension déplacer. Changer une dimension ne recalcule pas non plus les mouches : le réglage se dérègle et le relevé montre l’écart, car c’est ce qui arrive sur un vrai clavier.',
  },

  weighing: {
    intro:
      'Une touche arrive plombée, avec des plombs dont personne ne connaît la masse : on la pèse donc entière, plombs compris. Saisissez ce que donnent pour elle la balance et l’appareil de mesure à faible friction.',
    why: weighingWhy,
    notWeighed:
      'Pas encore pesée : cette touche n’a pas de masse dans le modèle, donc chaque mesure qui en demande une est refusée. Pesez-la entière et saisissez ce que donnent la balance et l’appareil de mesure à faible friction : sa masse, son poids avant, et à quelle distance du bord se trouvait le support à faible friction.',
    weighed: (massG, frontWeightG, fromLipMm) =>
      `Pesée : ${to(massG, MASS_G)} g, son poids avant ${to(frontWeightG, MASS_G)} g lu à ${to(fromLipMm, FROM_LIP_MM)} mm en arrière du bord.`,
    published: (massG) =>
      `Telle que publiée : ${massG === null ? 'sa masse' : `${to(massG, MASS_G)} g`}, et son centre de gravité, hauteur comprise, tels que sa source les donne.`,
    note: 'La **masse de la touche** est la touche entière sur une balance, pilote et plombs compris : le poids que la pesanteur applique en son centre de gravité. Le **poids avant** est ce que pèse la touche mise de niveau sur la balance de l’appareil de mesure à faible friction, la touche tournant sur sa pointe de balancier, le support à faible friction sous le point de mesure ; multiplié par le bras où il a été lu, c’est le moment statique de la touche autour de la pointe, qui situe son centre de gravité le long de la touche. **Lu à** est l’endroit où se trouvait le support, en arrière du bord du revêtement : le bras où il a été lu. Le banc d’essai garde ce moment statique : déplacer ensuite le point de mesure, ou changer la touche, lit le même moment sur un autre bras, et le poids avant du relevé suit par le levier. La hauteur du centre de gravité, qu’aucune balance ne donne, reste celle avec laquelle la touche est arrivée : publiée, si sa source en donne une, sinon estimée d’après la coupe dessinée de la touche, ce que le relevé signale sur chaque mesure qu’elle atteint. Elle n’agit que par l’inclinaison de la touche sur son châssis, quelques centièmes de gramme de poids d’enfoncement par millimètre.',
  },

  frame: {
    railsNote:
      'Et où **se trouvent**, le long de la touche, la barre d’enfoncement et la barre de repos arrière. Ce sont les dimensions du châssis, pas celles de la touche : une barre est fixée au châssis, donc allonger une touche déplace sa mortaise et son chanfrein et laisse les barres où elles sont. C’est une conception qu’il vaut la peine de voir échouer, et c’est pourquoi rien ici ne se corrige tout seul : le relevé nomme l’écart à la place.',
    placeNote:
      'Déplace les barres et **rien d’autre**. La touche porte ensuite sur le drap à un autre bras de levier, donc la hauteur de touche et l’enfoncement auront quitté leurs cotes, et le relevé le montrera : les remettre à la cote est une opération à part, à l’établi aussi.',
  },

  band: {
    title: 'La courbe d’enfoncement et de remontée',
    caption: (stepMm) =>
      `La touche menée lentement sur toute sa course, à la descente puis à la remontée, comme la mènerait un doigt : tous les ${stepMm} mm de course au point de mesure, la force qui l’y tient. Entre les deux courbes, la touche reste où elle est, tenue par sa friction : leur écart vient de la friction, leur milieu est l’équilibre de la touche. Le plus que demande la descente et le moins que retient la remontée sont marqués en entier, au centième de gramme : ce sont les nombres des courbes, qu’aucun poids ne lit. Le **D** et le **U** du relevé sont les mesures du métier, avec des poids de pesée posés ou retirés gramme par gramme depuis la position de repos et le fond de course, et lus comme les poids les donnent.`,
    travel: 'course au point de mesure, sous la position de repos (mm)',
    force: 'force sur le point de mesure, vers le bas (g)',
    down: 'courbe d’enfoncement : ce qui fait descendre la touche',
    up: 'courbe de remontée : ce qui la laisse remonter',
    held: 'tenue par la friction',
    weight: 'poids sur le point de mesure',
    key: 'où ce poids laisse la touche',
    topSeat: 'position de repos',
    bottomSeat: 'fond de course',
    downMarker: (grams) => `descente, maximum ${grams} g`,
    upMarker: (grams) => `remontée, minimum ${grams} g`,
    extremeAt: (label, travelMm) => `${label}, à ${travelMm} mm sous la position de repos`,
    weightLine: (grams) => `${grams} g`,
    readout: (travelMm, downG, upG) => `${travelMm} mm · descente ${downG} g · remontée ${upG} g`,
    seatsNote:
      'À chaque position, la butée reprend tout ce que le doigt laisse : les courbes commencent donc un pas en deçà de la position de repos et du fond de course, et leurs extrêmes s’y lisent, à quelques millièmes de gramme près du D et du U complets du relevé, qui cherche avec des poids de pesée depuis ces positions mêmes. Près d’un gramme entier, les deux peuvent tomber de part et d’autre, et c’est pourquoi la courbe ne lit aucun gramme : le gramme est celui du relevé.',
    stale:
      'La touche a changé depuis le tracé de cette courbe : choisissez **courbe d’enfoncement et de remontée** pour la tracer à nouveau.',
    staleRefusal:
      'La touche a changé depuis le refus de la courbe : choisissez **courbe d’enfoncement et de remontée** pour réessayer.',
    refused: (why) => `La courbe ne peut pas être tracée : ${why}`,
    table:
      'La courbe pas à pas : course au point de mesure, et force à la descente et à la remontée',
  },

  touch: {
    intro:
      'Posez un poids sur le point de mesure. Trois choses peuvent arriver, pas deux : au-dessus du poids d’enfoncement la touche descend, au-dessous du poids de remontée elle remonte, et **entre les deux, elle ne bouge pas du tout**. Cet écart vient de la friction : F en est la moitié.',
    verdict,
    theKeyIs: (verdict) => `la touche ${verdict}`,
    margin: (marginG) =>
      `le poids est entre U et D, à ${to(marginG, MASS_G)} g de faire bouger la touche`,
    gaps: (frontMm, backMm) => {
      const gap = (mm: number | null) => (mm === null ? 'aucun' : `${to(mm, GAP_MM)} mm`)
      return `jeu à la mouche d’enfoncement ${gap(frontMm)} · au drap de repos arrière ${gap(backMm)}`
    },
    bushings: (kf) =>
      kf === null
        ? 'la friction des garnitures est un intervalle, pas une valeur ; le protocole Stanwood ne peut pas la lire sur ce châssis'
        : `la friction des garnitures est un intervalle, pas une valeur : le protocole Stanwood la lit comme une friction de la touche de ${kf} g`,
    centerNote: (overlaps) =>
      `Les mouches se lisent en leur centre : une rondelle de feutre sous une touche porte la charge sur toute sa face, et le banc d’essai la prend au milieu, sous la pointe de balancier pour la mouche de balancier. Une touche qui n’est pas de niveau sur l’une d’elles chevauche le bord du feutre dessiné de ce dont le feutre se comprimerait${
        overlaps.length === 0
          ? ''
          : `, ici ${overlaps.map((overlap) => `${to(overlap.overlap_mm, LENGTH_MM)} mm ${AT_THE[overlap.pair]}`).join(' et ')}`
      }. La touche est libre de se soulever de sa mouche de balancier : là où ses butées s’opposent, elle est portée par elles, et une traction que sa mouche ne peut pas reprendre la soulève. Là où un équilibre place la touche, les contacts marqués sur la vue sont ceux où elle porte, et ce qui se trouve à moins d’un millimètre d’elle.`,
    notWeighed:
      'Cette touche n’est pas pesée : ce que fait un poids posé dessus dépend de l’équilibre propre de la touche, que seule une pesée donne, donc la touche est dessinée l’arrière posé sur son drap, comme pour lire la hauteur de touche, quel que soit le poids saisi. Pesez-la pour essayer les poids.',
    noPressNote:
      'Il n’y a pas de commande « enfoncer la touche ». La touche n’est jamais que là où la physique la met : un curseur sur une course calculée pourrait la garer dans une position qu’aucun poids ne produit, et c’est la seule chose qu’un instrument de mesure ne doit pas faire.',
  },
}
