/**
 * What a catalog must word: every sentence the bench shows, and every code
 * the engine sends.
 *
 * The engine sends codes and numbers, never sentences
 * (`crates/pa-app/src/boundary.rs`), and a catalog, one per language, words
 * them in its language. A code family is typed from the generated types, so a
 * code the engine gains and a catalog does not word fails `pnpm typecheck`.
 *
 * Emphasis is marked in the wording itself, `**bold**` and `*italic*`, and
 * nothing else, so that a translation moves it with the words it belongs to.
 */
import type {
  Acted,
  ActionCode,
  BackBearing,
  Contact,
  EditRefusal,
  Fault,
  FrictionRefusal,
  HeelContactFound,
  How,
  Input,
  KeyPlace,
  Label,
  Panel,
  Problem,
  Procedure,
  Refusal,
  RegulationRefusal,
  Riding,
  SettingUnit,
  TemporaryWeight,
  Unit,
  Verdict,
  WeighingRefusal,
  WippenCarriedBy,
  WippenRefused,
} from '@pas/wasm/types'

/** A sentence for each way the capstan/heel contact point is found, given
 * where the load was centered, millimeters along the body weighed. */
type Placed = { readonly [C in HeelContactFound['code']]: (atMm: number) => string }

/** Each member of a family of codes, by its code. */
type ByCode<T extends { code: string }> = { [K in T['code']]: Extract<T, { code: K }> }

/** A wording for every code of a family, each given that code's parameters. */
export type Worded<T extends { code: string }> = {
  readonly [K in T['code']]: (p: ByCode<T>[K]) => string
}

/**
 * `item` worded by `table`, the wording of its own `code`: pass the item's
 * code beside it, `word(t.fault, f.code, f)`, so that TypeScript sees that
 * the wording looked up is the one whose parameters the item has.
 */
export function word<M, K extends keyof M>(
  table: { readonly [P in keyof M]: (p: M[P]) => string },
  code: K,
  item: M[K],
): string {
  return table[code](item)
}

/** `sentence` begun with a capital letter. */
export function capitalized(sentence: string): string {
  return sentence.charAt(0).toUpperCase() + sentence.slice(1)
}

/** A name for every member of a union of codes. */
export type Named<T extends string> = { readonly [K in T]: string }

/** A quantity the bench reports beside its sheet: its label, and the unit
 * its value is shown in. */
export interface Reported {
  readonly label: string
  readonly unit: string
}

/**
 * What a control is worded by: its label, and, where the unit beside its
 * value reads as more than the unit, how, given the unit's own word.
 */
export interface Labeled {
  readonly label: string
  unit?(unit: string): string
}

/** The unit beside a control's value: its wording's, or the unit alone. */
export function unitBeside(labeled: Labeled, unit: string): string {
  return labeled.unit === undefined ? unit : labeled.unit(unit)
}

/** The units a path's name may end in, the longest first. */
const SUFFIXES: readonly SettingUnit[] = ['gmm', 'mm', 'g']

/** The unit a path's name ends in, as `unit` writes it after a value, space
 * included: every path carries its unit in its name. */
export function unitIn(path: string, unit: Named<SettingUnit>): string {
  const found = SUFFIXES.find((u) => path.endsWith(`_${u}`))
  return found === undefined ? '' : ` ${unit[found]}`
}

/** A regulation laid: the papers, and what they give. */
export type Laid = Extract<Acted, { outcome: 'laid' }>

export interface Catalog {
  /** `x` to `decimals` places as this language writes a number, never -0. */
  number(x: number, decimals: number): string
  /** Items as a sentence lists them: "a", "a and b", "a, b and c". */
  list(terms: readonly string[]): string
  /** Reasons as a note or a tooltip lists them, each a clause of its own:
   * "a; b". */
  reasons(clauses: readonly string[]): string

  readonly booting: string
  readonly title: string
  /** The title the browser's tab shows. */
  readonly tabTitle: string
  /** What the language selector is called, for a reader who cannot see it. */
  readonly languageSelector: string
  readonly protocol: string

  readonly view: {
    readonly section: string
    readonly solid: string
    readonly hint: string
    readonly bears: string
    readonly open: string
    readonly overlapping: string
    /** The view that draws the band instead of the key. */
    readonly band: string
    /** The toggle that marks each drawn line by where its number comes
     * from. */
    readonly provenance: string
    /** Where a line's number comes from, as the overlay's legend names it. */
    readonly how: Named<How>
    /** Why the key is drawn faded, by why no equilibrium places it where
     * it is drawn. */
    readonly notPlaced: Named<Exclude<KeyPlace, 'placed'>>
    /** The control that makes the key more or less transparent. */
    readonly keyOpacity: string
    /** The control that makes the key's two mortises' bushings more or
     * less transparent, apart from the rest of it. */
    readonly bushingsOpacity: string
  }

  /** A shipped profile's name, as the page offers it: the engine sends
   * profiles by their file's name, which a profile added and not worded
   * here shows as it is. */
  readonly profile: Readonly<Record<string, string>>

  readonly faults: {
    readonly heading: string
    readonly placement: string
  }
  readonly fault: Worded<Fault>
  /** Why no weighing is possible, given why the key cannot be seated. */
  noWeighing(why: string): string
  /** The stops fight, given how the key rides on them. */
  stopsFight(riding: string): string
  /** How the key rides on its stops, as a sentence. */
  riding(riding: Riding): string
  /** What carries the wippen the key carries where it is drawn, each link
   * with the parts of the heel it bears on, a sentence each. */
  wippenCarriedBy(links: readonly WippenCarriedBy[]): string
  /** Why the wippen the key carries refused the last loading or increment,
   * as a sentence. */
  readonly wippenRefused: Worded<WippenRefused>
  /** What no part could be built as, or what the bench has not. */
  readonly problem: Worded<Problem>
  /** Why any edit was refused: a problem, or the refusal of the
   * regulation, the weighing or the bushings. */
  readonly editRefusal: Worded<EditRefusal>
  /** An edit refused, nothing changed, by what was edited: a setting set,
   * or an action by its code, each saying what it left as it was; given
   * why. */
  readonly refused: { readonly [E in ActionCode | 'set']: (why: EditRefusal) => string }
  /** Which fields of a form hold no number, by the action the form sends,
   * each field by its label's code: nothing is sent until each has one.
   * `null` for an action no form sends. */
  readonly needsNumber: {
    readonly [A in ActionCode]: ((fields: readonly Label[]) => string) | null
  }

  /** A panel's heading, by its code. */
  readonly panel: Named<Panel>
  /** What each control is worded by, by the code the engine labels it
   * with. */
  readonly control: { readonly [K in Label]: Labeled }

  readonly sheet: {
    readonly measuringPointNote: string
    readonly gramWeights: string
    /** The two key friction weights, each with its temporary weight. */
    keyFrictionTwice(jig: TemporaryWeight | null, frame: TemporaryWeight | null): string
    /** What makes the readings indicative, the inputs named. */
    indicative(because: readonly string[]): string
    readonly refused: string
    /** A reading refused, and why. */
    isRefused(label: string, why: string): string
    /** The regulation target beside a reading. */
    target(value: string): string
    readonly seatingNote: string
    readonly keyMass: Reported
    readonly outOfLevel: Reported
    readonly bedding: Reported
    readonly bearsAt: Reported
    /** How the key bears on its cloth, named. */
    bearsHow(how: string): string
    readonly bearingNote: string
    /** The equation of balance on a key that carries no wippen. */
    readonly balanceNote: string
    /** The equation of balance on a key carrying its wippen. */
    readonly balanceNoteCarrying: string
    /** Where the key weight ratio's 10 g was centered, millimeters from the
     * balance pin, by how that point was found. */
    readonly keyRatioCentered: Placed
    /** Where the wippen weight's roller stood, millimeters from the
     * wippen's center pin, by how that point was found. */
    readonly wippenWeightCentered: Placed
    readonly wippenBalanceNote: string
  }
  readonly procedure: Named<Procedure>
  readonly because: Named<Input>
  /** A unit, a reading's or a setting's. */
  readonly unit: Named<Unit | SettingUnit>
  readonly backBearing: Named<BackBearing>
  readonly refusal: Worded<Refusal>

  readonly regulation: {
    /** The papers a regulation laid, and what they give. */
    laid(r: Laid): string
    readonly why: Worded<RegulationRefusal>
    readonly layingNote: string
    readonly pileNote: string
  }

  readonly papers: {
    /** A stack's title: its total and how many papers. */
    stack(title: string, totalMm: string, count: number): string
    readonly add: string
    /** One paper on the pile. */
    paper(mm: string): string
    readonly liftOff: string
    readonly bare: string
  }

  readonly friction: {
    readonly why: Worded<FrictionRefusal>
    readonly note: string
  }

  readonly punchings: {
    readonly note: string
    /** What stops the key at the bottom: the pin, the punching, or unread. */
    stopsAtTheBottom(byTheMortise: boolean | null): string
  }

  readonly key: {
    readonly ratioNote: string
  }

  readonly weighing: {
    readonly intro: string
    readonly why: Worded<WeighingRefusal>
    /** A key nobody has weighed: it has no mass, and what needs one is
     * refused until it is weighed. */
    readonly notWeighed: string
    /** The key's weighing as it was typed. */
    weighed(massG: number, frontWeightG: number, fromLipMm: number): string
    /** The key's mass as its profile's source publishes it, whole. */
    published(massG: number | null): string
    readonly note: string
  }

  readonly frame: {
    readonly railsNote: string
    readonly placeNote: string
  }

  /** The band, drawn: the force on the measuring point through the key's
   * stroke, down and back. */
  readonly band: {
    readonly title: string
    /** What the band is, how its extremes are marked and how they differ
     * from the sheet's D and U, given the step it is swept by. */
    caption(stepMm: string): string
    /** The horizontal axis, its unit named. */
    readonly travel: string
    /** The vertical axis, its unit named. */
    readonly force: string
    /** The down curve, as the legend names it. */
    readonly down: string
    /** The up curve. */
    readonly up: string
    /** The band between them. */
    readonly held: string
    /** The weight on the measuring point, as the legend names its line. */
    readonly weight: string
    /** Where the weight has left the key, as the legend names its dot. */
    readonly key: string
    /** The ends of the window, under the axis. */
    readonly topSeat: string
    readonly bottomSeat: string
    /** The down curve's greatest force, marked where it falls: in full, to
     * the hundredth of a gram, never as a gram weight reads it. */
    downMarker(grams: string): string
    /** The up curve's least force, marked where it falls, likewise. */
    upMarker(grams: string): string
    /** An extreme's label and how far below the top seat it falls, for a
     * reader without the chart. */
    extremeAt(label: string, travelMm: string): string
    /** The weight on the measuring point, beside its line. */
    weightLine(grams: string): string
    /** The two forces at one travel, where the pointer is. */
    readout(travelMm: string, downG: string, upG: string): string
    /** Why the band's extremes stand a step inside each seat, and so a
     * few thousandths of a gram from the sheet's D and U in full. */
    readonly seatsNote: string
    /** The band drawn before the last edit that changed it. */
    readonly stale: string
    /** The band refused before the last edit that changed the key. */
    readonly staleRefusal: string
    /** Why the band cannot be drawn. */
    refused(why: string): string
    /** The table that carries every step, for a reader without the chart. */
    readonly table: string
  }

  readonly touch: {
    readonly intro: string
    /** What holds the key, after "the key is". */
    readonly verdict: Worded<Verdict>
    /** The sentence a verdict stands in. */
    theKeyIs(verdict: string): string
    /** How far the weight stands from moving the key, grams. */
    margin(g: number): string
    /** The gaps at the front punching and the back rail cloth. */
    gaps(frontMm: number | null, backMm: number | null): string
    /** The bushings' friction, named by Stanwood's reading of it. */
    bushings(keyFrictionWeight: string | null): string
    /** Why a key nobody has weighed has no touch verdict. */
    readonly notWeighed: string
    /** The note on center bearings, with the edges overlapping the key. */
    centerNote(overlaps: readonly { pair: Contact; overlap_mm: number }[]): string
    readonly noPressNote: string
  }
}
