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
  BackBearing,
  Contact,
  Fault,
  FrictionRefusal,
  Input,
  Procedure,
  Refusal,
  Regulation,
  RegulationRefusal,
  Riding,
  Target,
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

/** A slider's label, and the unit its value is shown in. */
export interface Control {
  readonly label: string
  readonly unit: string
}

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
  }

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

  readonly sheet: {
    readonly heading: string
    readonly measuringPoint: Control
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
    readonly keyMass: Control
    readonly outOfLevel: Control
    readonly bedding: Control
    readonly bearsAt: Control
    /** How the key bears on its cloth, named. */
    bearsHow(how: string): string
    readonly bearingNote: string
    readonly balanceNote: string
  }
  readonly procedure: Named<Procedure>
  readonly because: Named<Input>
  readonly unit: Named<Unit>
  readonly backBearing: Named<BackBearing>
  readonly refusal: Worded<Refusal>

  readonly regulation: {
    readonly heading: string
    readonly keyHeight: Control
    readonly keyDip: Control
    readonly regulate: string
    /** Which target fields hold no number, each by its target. */
    needsNumber(fields: readonly Target[]): string
    /** How a regulation came out. */
    outcome(r: Regulation): string
    readonly why: Worded<RegulationRefusal>
    readonly layingNote: string
    readonly pileNote: string
  }

  readonly papers: {
    readonly balance: string
    readonly front: string
    /** A stack's title: its total and how many papers. */
    stack(title: string, totalMm: string, count: number): string
    readonly add: string
    /** One paper on the pile. */
    paper(mm: string): string
    readonly liftOff: string
    readonly bare: string
  }

  readonly friction: {
    readonly control: Control
    /** A key friction weight typed and refused. */
    refused(why: FrictionRefusal): string
    readonly why: Worded<FrictionRefusal>
    readonly note: string
  }

  readonly punchings: {
    readonly heading: string
    readonly note: string
    readonly balanceFelt: Control
    readonly frontFelt: Control
    readonly frontPin: Control
    readonly frontMortise: Control
    /** What stops the key at the bottom: the pin, the punching, or unread. */
    stopsAtTheBottom(byTheMortise: boolean | null): string
    readonly backCloth: Control
  }

  readonly key: {
    readonly heading: string
    readonly ratioNote: string
    readonly capstan: Control
    readonly frontEdge: Control
    readonly capstanHeight: Control
  }

  readonly weighing: {
    readonly heading: string
    readonly intro: string
    readonly mass: Control
    readonly frontWeight: Control
    readonly readAt: Control
    readonly weigh: string
    /** Which weighing fields hold no number. */
    needsNumber(fields: readonly ('mass' | 'front_weight' | 'from_lip')[]): string
    /** A weighing refused. */
    refused(why: WeighingRefusal): string
    readonly why: Worded<WeighingRefusal>
    /** The key's mass as the bench holds it: derived from its volume. */
    notWeighed(massG: number | null): string
    /** The key's weighing as it was typed. */
    weighed(massG: number, frontWeightG: number, fromLipMm: number): string
    readonly note: string
  }

  readonly frame: {
    readonly heading: string
    readonly balanceRail: Control
    readonly frontRail: Control
    readonly backRail: Control
    readonly railsNote: string
    readonly frontRailAt: Control
    readonly backRailAt: Control
    readonly place: string
    readonly placeNote: string
  }

  readonly touch: {
    readonly heading: string
    readonly intro: string
    readonly weight: Control
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
    centreNote(overlaps: readonly { pair: Contact; mm: number }[]): string
    readonly noPressNote: string
  }
}
