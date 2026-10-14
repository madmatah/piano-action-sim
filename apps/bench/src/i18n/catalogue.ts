/**
 * What a catalogue must word: every sentence the bench shows, and every code
 * the engine sends.
 *
 * The engine sends codes and numbers, never sentences
 * (`crates/pa-app/src/boundary.rs`), and a catalogue, one per language, words
 * them in its language. A code family is typed from the generated types, so a
 * code the engine gains and a catalogue does not word fails `pnpm typecheck`.
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
  How,
  Input,
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
} from '@pas/wasm/types'

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
export interface Labelled {
  readonly label: string
  unit?(unit: string): string
}

/** The unit beside a control's value: its wording's, or the unit alone. */
export function unitBeside(labelled: Labelled, unit: string): string {
  return labelled.unit === undefined ? unit : labelled.unit(unit)
}

/** A regulation laid: the papers, and what they give. */
export type Laid = Extract<Acted, { outcome: 'laid' }>

export interface Catalogue {
  /** `x` to `decimals` places as this language writes a number, never -0. */
  number(x: number, decimals: number): string
  /** Items as a sentence lists them: "a", "a and b", "a, b and c". */
  list(items: readonly string[]): string

  readonly booting: string
  readonly title: string
  readonly protocol: string

  readonly view: {
    readonly section: string
    readonly solid: string
    readonly hint: string
    readonly bears: string
    readonly open: string
    readonly overlapping: string
    /** The toggle that marks each drawn line by where its number comes
     * from. */
    readonly provenance: string
    /** Where a line's number comes from, as the overlay's legend names it. */
    readonly how: Named<How>
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
  readonly control: { readonly [K in Label]: Labelled }

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
    readonly balanceNote: string
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
    /** The key's mass as the bench holds it: derived from its volume. */
    notWeighed(massG: number | null): string
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
    /** The note on centre bearings, with the edges overlapping the key. */
    centreNote(overlaps: readonly { pair: Contact; overlap_mm: number }[]): string
    readonly noPressNote: string
  }
}
