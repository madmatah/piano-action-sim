//! The mechanisms a key is put in to be read, and the contacts it has with
//! its frame.
//!
//! A mount is a builder, not a type: the same key, with the same mass, goes
//! into whichever mechanism a procedure needs, and the procedure reads it
//! there. Built: the jig; the key on its frame free to lift off its balance
//! punching, [`KeyMount::OnPunching`], the mount every procedure on the frame
//! reads; and the key turning about a fixed axis through its balance bearing,
//! [`KeyMount::Pivot`], kept for the comparisons with an action published
//! with its key on a pivot.
//!
//! # The key on its punching
//!
//! A key is not pinned to its frame. It rests on its balance punching, which
//! can push it up and cannot hold it down, and its balance pin only guides
//! it. So the key on its frame has two coordinates: a slide carrying the key
//! up and down the balance pin's axis, and the key's angle about the slide's
//! origin. The balance punching is a unilateral contact like the other stops,
//! read at the center of its drawn top. A frame whose stops fight, a key lifted
//! off its punching and a key seated on it come out of the same static solve:
//! the key rides on its two outer stops off the punching where they hold it
//! higher than the punching would, on the parts as drawn.
//!
//! # Solved on its punching first
//!
//! While the punching carries the key, the key turns about the punching's
//! center, which stands on the slide's axis: the key on its punching is then
//! the key on a pivot there. [`Mounted::solve`] tries that first, the slide
//! held on the punching and every other contact read as it is. Where the
//! punching then pushes the key up, that equilibrium meets every condition of
//! the full problem, the punching's multiplier being the slide's reaction, and
//! it is to the bit the one the key on a pivot gives. Where the punching would
//! have to pull, or the key stands off it, the punching is released and the
//! full problem solved.

use super::frame::{FrameDesign, FrameRanges, KeyKind};
use super::key::KeyRanges;
use super::wippen_assembly::{WippenAssembly, WippenAssemblyMass, WippenAssemblyRanges};
use super::KeyDesign;
use crate::contact::{
    ContactCode, ContactLaw, ContactPair, PairGeometry, Part, PartCode, PartId, Rows, Surface,
    ROW_MARGIN,
};
use crate::geometry::{Dir, Point};
use crate::mechanism::{
    Body, BodyId, Coord, CoordId, Joint, MassProps, Mechanism, PointLoad, Prescribed, Problem,
};
use crate::solve::statics::GAP_TOLERANCE;
use crate::solve::{Equilibrium, Friction, Holder, Refusal, SolveReport, Statics};

/// A key in a mechanism, and where it is in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Mounted {
    /// The mechanism.
    pub mechanism: Mechanism,
    /// The key's body. Its frame is the key's own: origin at the balance pin
    /// on the key's underside, `x` toward the player.
    pub key: BodyId,
    /// The key's angle, positive nose-up.
    pub angle: CoordId,
    /// The key's freedom to lift off its balance punching, on a key free to
    /// ([`KeyMount::OnPunching`]); `None` on the jig and on a pivot.
    pub lift: Option<Lift>,
    /// The wippen the key carries on its capstan, where it carries one.
    pub wippen: Option<WippenOn>,
    /// The same key on the same frame without its wippen, where it carries
    /// one: what each seat of the key with its wippen starts from, and a
    /// loading where the wippen fits nowhere
    /// ([`crate::measure::seat::Seating::start`]). Its coordinates are the
    /// key's, the first of this mount's, which carries the wippen's angle
    /// after them.
    pub key_alone: Option<Box<Mounted>>,
}

/// Which way the key moves through its stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// Its front going down, its back and its capstan rising.
    Down,
    /// Its front coming up, its back and its capstan falling.
    Up,
}

/// The wippen in a mount: the last body, turning on its center pin on the
/// ground, and the two links that carry it on the key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WippenOn {
    /// Its body. Its frame is the wippen's own: origin at its center pin,
    /// `x` toward the player at angle zero.
    pub body: BodyId,
    /// Its angle, positive lifting its heel.
    pub angle: CoordId,
    /// The two links' pairs, in the mechanism's order: the capstan on the
    /// heel ([`ContactCode::CapstanOnTheHeel`]), then the heel on the key
    /// ([`ContactCode::HeelOnTheKey`]). Both are declared before the balance
    /// punching, which stays the last pair, so that they keep their places
    /// in the key held on its punching.
    pub links: [u16; 2],
    /// How far its farthest drawn point stands from its center pin, meters:
    /// how far a turn of a radian moves it at most.
    pub reach: f64,
}

/// Which limit of its turn the wippen leans on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WippenLimit {
    /// The upper: pushed past its range, its heel lifted to the edge of its
    /// modeled world.
    PushedPastItsRange,
    /// The lower: out of the key's reach, its heel hanging to the other
    /// edge.
    OutOfReach,
}

/// A wippen to hang in a mount: where its center pin stands in the keybed's
/// frame, its mass with the jack and the repetition lever at rest on it, the
/// bound of its center's friction, newton-meters, and the places of its two
/// links among the pairs the mount is given.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WippenHung {
    /// Its center pin, in the keybed's frame.
    pub center: Point,
    /// Its mass and center of mass, in its frame.
    pub mass: MassProps,
    /// Its center's dry friction, newton-meters.
    pub friction: f64,
    /// The two links' places among the pairs: the capstan's, then the
    /// key's top's.
    pub links: [u16; 2],
}

/// The wippen's body and coordinate hung after `bodies` and `coords`, on its
/// center pin on the ground, within [`WIPPEN_ANGLE_LIMIT`] either way; its
/// reach measured off the parts of `parts` it carries
/// ([`measure_the_reach`]).
fn hang(
    bodies: &mut Vec<Body>,
    coords: &mut Vec<Coord>,
    hung: WippenHung,
    parts: &[Part],
) -> WippenOn {
    let (body, angle) = (BodyId(bodies.len()), CoordId(coords.len()));
    bodies.push(Body {
        parent: BodyId::GROUND,
        joint: Joint::Revolute {
            pivot: hung.center,
            coord: angle,
        },
        mass: Some(hung.mass),
    });
    coords.push(
        Coord::ANGLE
            .limited(-WIPPEN_ANGLE_LIMIT, WIPPEN_ANGLE_LIMIT)
            .with_friction(hung.friction),
    );
    WippenOn {
        body,
        angle,
        links: hung.links,
        reach: measure_the_reach(parts, body),
    }
}

/// How far the farthest drawn point of the parts of `parts` on `body` stands
/// from that body's origin, meters: a wippen's reach from its center pin.
/// Read off the parts as given, which the mechanism keeps as they are.
fn measure_the_reach(parts: &[Part], body: BodyId) -> f64 {
    parts
        .iter()
        .filter(|part| part.body == body)
        .flat_map(|part| part.silhouette())
        .flat_map(|segment| segment.drawn(crate::geometry::DRAWN_HALVINGS))
        .map(|point| (point.x * point.x + point.y * point.y).sqrt())
        .fold(0.0, f64::max)
}

/// A key's freedom to lift off its balance punching.
#[derive(Debug, Clone, PartialEq)]
pub struct Lift {
    /// The slide's coordinate: the height of the key frame's origin above
    /// the keybed, meters.
    pub coord: CoordId,
    /// The slide's value with the key on its balance punching: the height of
    /// the punching's drawn top, meters.
    pub on_punching: f64,
    /// The balance punching's pair: the mechanism's last.
    pub pair: u16,
    /// The key held on its punching: the same bodies, coordinates and parts,
    /// every pair but the punching's, solved with the slide held at
    /// [`Self::on_punching`]. `None` where the punching's center does not
    /// stand on the slide's axis, where the key held on it would not be the
    /// key turning about it.
    turning: Option<Mechanism>,
}

/// Stanwood's wedge jig: the key turning about its balance pin's axis, and
/// nothing else.
///
/// Two bodies: the ground, which is the jig, and the key, hung from it by a
/// revolute joint at the origin of the key's frame, on the balance pin's
/// axis. The joint turns without friction, which is an assumption: the real
/// key turns on an extra-long balance pin through its bushings. `mass` is the
/// key's, `None` for a key nobody has weighed.
pub fn jig(mass: Option<MassProps>) -> Result<Mounted, Problem> {
    let key = Body {
        parent: BodyId::GROUND,
        joint: Joint::Revolute {
            pivot: Point::new(0.0, 0.0),
            coord: CoordId(0),
        },
        mass,
    };
    Ok(Mounted {
        mechanism: Mechanism::new(vec![Body::GROUND, key], vec![Coord::ANGLE])?,
        key: BodyId(1),
        angle: CoordId(0),
        lift: None,
        wippen: None,
        key_alone: None,
    })
}

/// How a key stands on its frame while it is read.
///
/// The same key parts and the same mass go into whichever mount a procedure
/// needs; the mount says what the key may do there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMount {
    /// Turning about a fixed axis through its balance bearing, the one
    /// coordinate its angle, positive nose-up, within
    /// `±`[`KEY_ANGLE_LIMIT`], its bushings' friction acting on it.
    /// Hirschkorn's own simplification of the key, "like every joint, a
    /// simple revolute joint" (`data/params/hirschkorn-boston-gp178-note52.json`,
    /// note on Pg1), kept to compare with an action published so. The key
    /// cannot lift off its balance punching on it: a frame whose stops fight
    /// leaves it no angle at all.
    Pivot,
    /// Free to lift off its balance punching ([`on_punching`]): a massless
    /// slide up the balance pin's axis, carrying the key turning about the
    /// slide's origin, and the punching a unilateral contact read at the
    /// center of its drawn top. The mount every procedure on the frame reads.
    OnPunching,
}

impl KeyMount {
    /// The key's body in this mount: the ground's child on a pivot, the
    /// slide's on its punching.
    pub fn key_body(self) -> BodyId {
        match self {
            Self::Pivot => BodyId(1),
            Self::OnPunching => BodyId(2),
        }
    }

    /// The body a wippen the key carries hangs as in this mount: the one
    /// after the key's.
    pub fn wippen_body(self) -> BodyId {
        BodyId(self.key_body().0 + 1)
    }
}

/// How far the wippen may turn either way from its frame, radians: 30°, the
/// edge of its modeled world, as [`KEY_ANGLE_LIMIT`]'s 15° is the key's.
///
/// The key's limit carried through the capstan to the heel: the wippen
/// turns by the key's turn times the capstan's arm from the balance pin
/// over the heel's arm from the center pin, 117.6 mm over 61.2 mm on
/// Hirschkorn's action, 1.92, and 138.5 mm over 65.9 mm on Reyburn's
/// measurements 1, 2 and 6
/// (`docs/research/A3-nomenclature-and-regulation.md` > Reyburn's ten
/// measurements) [V, Reyburn, pp. 3, 5], 138.5 = 398 − 259.5, 2.10: about
/// twice. Far past any wippen's stroke, as the key's
/// limit is past its dip: Hirschkorn's turns from 1.5° (0.0267 rad) at the
/// top seat to 6.0° at the bottom, and the limit stands more than five of
/// those strokes past it. A wippen allowed further hangs nearly upright, its
/// contact all but radial to its center pin, where it holds no torque. Past it, a
/// center pin set too low for its capstan or too high to reach it, the
/// static solve leans the wippen on its limit and refuses it, named by the
/// side it leans on ([`Mounted::wippen_limit`]), at the seat the sheet
/// refuses it at, and by how far past it the heel stands there
/// ([`crate::measure::seat::WippenRefusal`]).
pub const WIPPEN_ANGLE_LIMIT: f64 = core::f64::consts::FRAC_PI_6;

/// How far the key may turn either way on its frame, radians: 15°, the edge
/// of the modeled world. Far past any stroke (a key's dip turns it about
/// 2.3°), it bounds what a static solve searches, and a key that leans on it
/// stands on nothing modeled.
pub const KEY_ANGLE_LIMIT: f64 = core::f64::consts::PI / 12.0;

/// The steps of the scan [`nominal_pose`] makes over the key's range: a
/// degree.
pub(crate) const NOMINAL_STEP: f64 = core::f64::consts::PI / 180.0;

/// How many golden-section steps narrow the attitude at which a key lowered
/// onto its frame from above, by [`nominal_pose`] or by
/// [`lowered_onto_its_frame`], clears it, where no attitude a whole degree
/// apart clears below where the key leaves its balance pin. Each step keeps
/// 0.618 of the bracket, two degrees wide: 22 steps leave under a
/// microradian, under a micron at the front of a key half a meter long,
/// where the lift a whole degree's step costs at the key's stops, up to half
/// a degree times their arm from the pin, is a millimeter or two.
pub(crate) const ATTITUDE_REFINEMENTS: u32 = 22;

/// How many times at most [`wippen_lowered`] halves the degree in which the
/// wippen's heel first touches the key: 52, a double's mantissa, past which
/// the halving stands still. It halves until what is left of the degree
/// moves the wippen's farthest drawn point less than the row margin: two
/// halvings for the test wippen, reaching 0.126 m from its center pin, a
/// degree there moving it 2.2 mm. The validation layer bounds no
/// wippen's reach, its outlines being traced from a record; the cap ends the
/// halving for any, and the start it leaves still clears the key, only
/// further above where it first touches. The count is reported with the
/// start ([`Lowered`]), and with the seat or the solve it starts.
pub(crate) const LOWERING_HALVINGS: u32 = 52;

/// The ratio of the golden section, `(√5 − 1) / 2`: the part of its bracket
/// each step of [`ATTITUDE_REFINEMENTS`] keeps.
const GOLDEN: f64 = 0.618_033_988_749_894_9;

/// In how many pieces of the static solve's cap of steps a key's problem may
/// be solved, each going on from where the last stopped ([`Mounted::solve`]):
/// four, for the key held on its balance punching as for the key free to
/// lift.
///
/// A key turning on a closed stop moves its farthest point a row margin a
/// step at most, every step bounded by the stop's margin, and a key lifting
/// off its punching under a pull turns up on its cloth until it leaves its
/// balance pin ([`Mounted::leaves_its_pin`]): 33 mm above its punching on
/// the test frame, some 70 mm at its front. A key riding on its front
/// punching, its front pin's tip against the wall of its slot, solved from
/// where it is lowered onto its frame, takes three quarters of a cap to come
/// to rest with every row that carries it closed
/// (`a_load_on_a_row_standing_open_is_no_rest`). A key carrying a wippen
/// keeps the link to its heel within the row margin through any turn, so
/// every step moves the key and the wippen together by a margin of their
/// farthest points at most, about 1.7 mrad of the key's turn on the test
/// bench: a turn of 0.13 to 0.16 rad held on its punching, a loading from
/// the level pose or an increment across the stroke, outruns one cap. Four
/// caps hold each with room; a seat of the corpus takes a few steps.
pub const SOLVE_PIECES: u32 = 4;

impl Mounted {
    /// The key level, and on its balance punching where it is free to lift:
    /// every coordinate zero but the slide, at the punching.
    pub fn level(&self) -> Vec<f64> {
        let mut q = vec![0.0; self.mechanism.coords().len()];
        if let Some(lift) = &self.lift {
            q[lift.coord.0] = lift.on_punching;
        }
        q
    }

    /// How far above its balance punching the key stands at `q`, meters:
    /// the slide less its value on the punching; zero on a mount with no
    /// slide.
    pub fn lift_at(&self, q: &[f64]) -> f64 {
        self.lift
            .as_ref()
            .map_or(0.0, |lift| q[lift.coord.0] - lift.on_punching)
    }

    /// How far above its balance punching the key leaves its balance pin,
    /// meters: the top of the slide's range, where the key's balance hole
    /// clears the pin's drawn top ([`on_punching`]), less the slide's value
    /// on the punching. Negative where the punching stands above the pin's
    /// top, the key on it already off its pin; `None` on a mount with no
    /// slide.
    pub fn leaves_its_pin(&self) -> Option<f64> {
        self.lift.as_ref().map(|lift| {
            let (_, top) = self.mechanism.coords()[lift.coord.0].limits;
            top - lift.on_punching
        })
    }

    /// Whether `refusal` is the key lifting off its frame: an increment that
    /// came to rest leaning on the slide's upper limit, where the key leaves
    /// its balance pin ([`Self::leaves_its_pin`]), or that has the key on a
    /// punching standing above the pin's top; or one leaning on the upper
    /// limit of its turn, [`KEY_ANGLE_LIMIT`] nose-up, where nothing modeled
    /// holds it, and a key turning up about a stop near its pin meets that
    /// limit before its balance hole clears a tall pin.
    pub fn lifts_off_its_frame(&self, refusal: &Refusal) -> bool {
        let Some(lift) = &self.lift else {
            return false;
        };
        match refusal {
            Refusal::NoEquilibrium { on } => [lift.coord, self.angle]
                .into_iter()
                .any(|coord| on.contains(&Holder::Limit { coord, upper: true })),
            _ => false,
        }
    }

    /// How the wippen's angle follows the key's at `q` through the link that
    /// carries it, radians of the wippen per radian of the key: through a
    /// link row closed within `within`, the wippen kept on it,
    /// `r = −(∂g/∂θk)/(∂g/∂θw)`, `g` the row's gap. Where several link rows
    /// are that close, `motion` picks the one that carries the wippen:
    /// going down, the key's back rising, the one that lifts it most, the
    /// largest `|r|`; coming up, the one it stays on, that drops least, the
    /// smallest. Nothing where no link row is that close, the key read
    /// alone, and on a mount that carries no wippen.
    ///
    /// What the closed forms and the preparation of a seat read the wippen
    /// by: its generalized force folded onto the key's angle, `Q_k + r·Q_w`
    /// ([`Self::folded`]).
    pub fn wippen_follows(&self, q: &[f64], motion: Motion, within: f64) -> f64 {
        let Some(wippen) = &self.wippen else {
            return 0.0;
        };
        let m = &self.mechanism;
        let mut rows = crate::contact::Rows::new();
        m.rows(&m.pose(q), within.max(0.0), &mut rows);
        let follows = (0..rows.len())
            .filter(|&i| {
                wippen.links.contains(&rows.rows()[i].pair) && rows.rows()[i].gap <= within
            })
            .filter_map(|i| {
                let (by_key, by_wippen) =
                    (rows.grad(i)[self.angle.0], rows.grad(i)[wippen.angle.0]);
                (by_wippen != 0.0).then(|| -by_key / by_wippen)
            });
        let pick = |a: f64, b: f64| match motion {
            Motion::Down if b.abs() > a.abs() => b,
            Motion::Up if b.abs() < a.abs() => b,
            _ => a,
        };
        follows.reduce(pick).unwrap_or(0.0)
    }

    /// The generalized force `forces` puts on the key's angle with the
    /// wippen's folded onto it by `follows` ([`Self::wippen_follows`]):
    /// `Q_k + r·Q_w`; the key's alone on a mount that carries no wippen.
    pub fn folded(&self, forces: &[f64], follows: f64) -> f64 {
        let on_the_key = forces[self.angle.0];
        match &self.wippen {
            Some(wippen) => on_the_key + follows * forces[wippen.angle.0],
            None => on_the_key,
        }
    }

    /// Whether `refusal` leans the wippen on a limit of its turn, and on
    /// which: `None` for any other refusal, and on a mount that carries no
    /// wippen.
    pub fn wippen_limit(&self, refusal: &Refusal) -> Option<WippenLimit> {
        let wippen = self.wippen.as_ref()?;
        let Refusal::NoEquilibrium { on } = refusal else {
            return None;
        };
        on.iter().find_map(|holder| match *holder {
            Holder::Limit { coord, upper } if coord == wippen.angle => Some(if upper {
                WippenLimit::PushedPastItsRange
            } else {
                WippenLimit::OutOfReach
            }),
            _ => None,
        })
    }

    /// How far past `limit` the wippen's heel stands with the key at `q`,
    /// meters: a least distance between the drawn outlines of its heel and
    /// its heel cloth and those of the key's parts, not a reading along the
    /// capstan's axis, and no correction of the capstan or of the center pin.
    /// Nothing on a mount that carries no wippen.
    ///
    /// Pushed past its range, how deep they stand in the key's parts with
    /// the wippen turned to the top of its range: by the drawn outlines
    /// where they read more than nothing; else by the links' rows, which
    /// read the arcs themselves ([`Self::measure_the_links_overlap`]). The
    /// drawn outlines take an arc as eight chords, which stand inside a
    /// convex arc of material by its sag, R(1 − cos(θ/16)) for an arc of
    /// radius R spanning θ: 21.8 µm on the test heel's cloth. Over a band of
    /// settings the drawn outlines then clear the key while the links stand
    /// the heel's cloth in it; [`wippen_lowered`] refuses the wippen there
    /// by the links, and this reads how deep they stand it: more than
    /// nothing, and no more than the sags of the convex arcs of material
    /// that meet there added together, the heel cloth's and, on a domed
    /// capstan, the dome's, which the tests pin on a flat capstan, the heel
    /// cloth's sag alone. Everywhere else the reading is the drawn one, bit
    /// for bit. Where the drawn outlines first stand in the key, the reading
    /// falls from the links' to the drawn one, which can be less: it does
    /// not grow monotonically as a setting moves further past.
    ///
    /// Out of reach, how near they come to the key's parts, the capstan and
    /// the key's top, degree by degree through the wippen's range: the least
    /// clearance at the angles the bench decides reach at, from the top of
    /// its range down ([`step_down_a_whole_degree`], [`wippen_lowered`]),
    /// carried across them ([`crate::contact::Outlines::clearance`]).
    /// Where the bench finds the wippen reaching, some one of those angles
    /// stands the heel in the key, and its clearance is nothing; moving the
    /// center pin, the heel or the capstan by a length moves the heel against
    /// the key's parts by no more than that length, so the reading is no more
    /// than how far a setting moved from one at which the wippen reached, and
    /// falls to nothing as the setting comes back there. Between two of those
    /// angles the heel turns from the nearer by at most the wippen's reach,
    /// its farthest drawn point from the center pin ([`WippenOn::reach`]),
    /// times half a degree, so the reading stands above the nearest the heel
    /// comes over the whole turn by no more than that. Both bounds hold away from a
    /// cut through a key part opening toward the heel, across whose mouth the
    /// distance outside the key is read to the part's own edge; no key the
    /// bench ships has one there. The angles and their count are the
    /// decision's, and the least is computed exactly at each, so nothing
    /// converges and nothing is approximated.
    pub fn measure_the_heel_past(&self, q: &[f64], limit: WippenLimit) -> f64 {
        let Some(wippen) = &self.wippen else {
            return 0.0;
        };
        let m = &self.mechanism;
        let (low, high) = m.coords()[wippen.angle.0].limits;
        let at = |angle: f64| {
            let mut at = q.to_vec();
            at[wippen.angle.0] = angle;
            at
        };
        let (heel, key) = find_the_heel_and_the_key(self, wippen);
        let key = m.outlines(&m.pose(q), &key);
        let heel_at = |angle: f64| m.outlines(&m.pose(&at(angle)), &heel);
        match limit {
            WippenLimit::PushedPastItsRange => {
                let drawn = heel_at(high).overlap(&key);
                if drawn > 0.0 {
                    drawn
                } else {
                    self.measure_the_links_overlap(&at(high))
                }
            }
            WippenLimit::OutOfReach => {
                let (mut angle, mut least) = (high, f64::INFINITY);
                loop {
                    least = heel_at(angle).clearance_below(&key, least);
                    if angle <= low || least <= 0.0 {
                        return least;
                    }
                    angle = step_down_a_whole_degree(angle, low);
                }
            }
        }
    }

    /// How deep the wippen's heel and its cloth stand in the key at `q` by
    /// the contacts the static solve reads, meters: the deepest overlap
    /// among the rows of its two links within the row margin, read alone
    /// as the whole mechanism's rows read them, which read the heel's arcs
    /// themselves and not their drawn chords. Nothing where none overlaps,
    /// and on a mount that carries no wippen.
    pub fn measure_the_links_overlap(&self, q: &[f64]) -> f64 {
        let Some(wippen) = &self.wippen else {
            return 0.0;
        };
        let m = &self.mechanism;
        let mut rows = Rows::new();
        m.read_the_rows_of(&m.pose(q), &wippen.links, ROW_MARGIN, &mut rows);
        rows.rows()
            .iter()
            .fold(0.0_f64, |deepest, row| deepest.max(-row.gap))
    }

    /// This key with its bushings' friction bounded by `bound` newton-meters
    /// instead, everything else kept bit for bit; or why not
    /// ([`Mechanism::with_friction`]).
    pub fn with_friction(mut self, bound: f64) -> Result<Self, Problem> {
        self.mechanism = self.mechanism.with_friction(self.angle, bound)?;
        if let Some(lift) = &mut self.lift {
            if let Some(turning) = lift.turning.take() {
                lift.turning = Some(turning.with_friction(self.angle, bound)?);
            }
        }
        Ok(self)
    }

    /// One load increment of the key from `from`, under `loads`, friction on
    /// or off: [`Statics::solve`] on the mechanism, nothing prescribed.
    ///
    /// On a key free to lift, standing on its punching to within the static
    /// solve's gap tolerance, the key is first solved on its punching: the
    /// slide held there, every contact but the punching's read. Where the
    /// slide's reaction then pushes the key up, or is nothing, that
    /// equilibrium is the full problem's: the punching's gap is closed, its
    /// multiplier that reaction over the cosine of the key's angle, and its
    /// center on the slide's axis gives the angle nothing to carry. The
    /// equilibrium returned then holds that push as the slide's reaction, and
    /// its supports do not list the punching.
    ///
    /// The punching is released, and the full problem solved from `from`,
    /// where it would have to pull the key down, or where the key's stops
    /// fight on its punching, leaving it no motion: a key free to lift may
    /// ride on them. Any other refusal of the key on its punching is the full
    /// problem's too, and is returned: a key that turns onto the edge of its
    /// range with nothing to stop it, off its punching or on it, has no
    /// equilibrium either way. A key standing off its punching is solved free
    /// from the start.
    ///
    /// The full problem is not solved from a start whose drawn parts stand in
    /// one another deeper than the row margin, which no row reads truthfully:
    /// what the pairs read ([`Mechanism::overlap`]), and the key in any part
    /// of its frame, a punching's felt and papers whole, so that a key sunk
    /// right through a felt thicker than itself, which leaves the punching's
    /// center clear, is not one. There the refusal of the key on its punching
    /// stands. The key held on its punching and the key free to lift are each
    /// solved in up to [`SOLVE_PIECES`] pieces of the static solve's cap of
    /// steps, each going on from where the last stopped.
    ///
    /// A key whose balance punching stands above its balance pin's top has
    /// no pose on its pin that clears the punching: it is refused as leaning
    /// on the slide's upper limit, off its frame, whatever the loads, and
    /// nothing is solved.
    pub fn solve(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        friction: Friction,
    ) -> Result<Equilibrium, Refusal> {
        self.solve_held(statics, from, loads, &[], friction)
    }

    /// One load increment of the key from `from`, under `loads`, with the
    /// coordinates `held` prescribed at their values, friction on or off:
    /// [`Self::solve`], which is this with nothing held, with a coordinate
    /// held as a procedure holds it, the key's angle driven through its
    /// stroke by the band's sweep ([`crate::measure::sweep`]). On its
    /// punching, the key held there is tried first, the slide held besides;
    /// the punching is released and the full problem solved, the slide free,
    /// where it would have to pull. Each held coordinate's reaction is in the
    /// equilibrium's; its friction, which no solve can set while it is held,
    /// is the caller's. `held` does not hold the slide: the slide is what
    /// tells a key on its punching from one lifted off it.
    ///
    /// A key carrying a wippen starts from a pose the start of a solve gives
    /// ([`crate::measure::seat::Seating::start`]), or from a state one has
    /// reached, its heel no deeper in the key than the row margin: a debug
    /// build says so loudly where a caller skipped the rule, rather than
    /// solve with the wippen wherever the pose left it.
    pub fn solve_held(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        held: &[Prescribed],
        friction: Friction,
    ) -> Result<Equilibrium, Refusal> {
        debug_assert!(
            heel_in_the_key(self, from) <= ROW_MARGIN,
            "a solve started with the wippen's heel {} m in the key, past the row margin: \
             every start of a key carrying its wippen goes by the start rule",
            heel_in_the_key(self, from)
        );
        if let Some(lift) = &self.lift {
            if self.leaves_its_pin().is_some_and(|room| room < 0.0) {
                return Err(Refusal::NoEquilibrium {
                    on: vec![Holder::Limit {
                        coord: lift.coord,
                        upper: true,
                    }],
                });
            }
            if let Some(turning) = &lift.turning {
                let c = lift.coord.0;
                if (from[c] - lift.on_punching).abs() <= GAP_TOLERANCE {
                    let mut on_it = vec![Prescribed {
                        coord: lift.coord,
                        value: lift.on_punching,
                    }];
                    on_it.extend_from_slice(held);
                    match in_pieces(statics, turning, from, loads, &on_it, friction) {
                        Ok(eq) if eq.reactions[c] >= 0.0 => return Ok(eq),
                        Ok(_) => {}
                        Err(Refusal::NoEquilibrium { on })
                            if on.iter().any(|h| matches!(h, Holder::Contact { .. })) =>
                        {
                            if sunk_in_its_frame(self, from) > ROW_MARGIN {
                                return Err(Refusal::NoEquilibrium { on });
                            }
                        }
                        Err(refusal) => return Err(refusal),
                    }
                }
            }
        }
        self.solve_free(statics, from, loads, held, friction)
    }

    /// The full problem, the key free to lift, from `from`, `held`
    /// prescribed: [`Statics::solve`] on the mechanism, in pieces
    /// ([`in_pieces`]).
    fn solve_free(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        held: &[Prescribed],
        friction: Friction,
    ) -> Result<Equilibrium, Refusal> {
        in_pieces(statics, &self.mechanism, from, loads, held, friction)
    }
}

/// [`Statics::solve`] on `m` from `from`, going on from where it stopped at
/// its cap, up to [`SOLVE_PIECES`] pieces in all, as one increment along its
/// path; its report one increment's, its steps all the pieces'.
fn in_pieces(
    statics: &mut Statics,
    m: &Mechanism,
    from: &[f64],
    loads: &[PointLoad],
    held: &[Prescribed],
    friction: Friction,
) -> Result<Equilibrium, Refusal> {
    let mut start = from.to_vec();
    let mut before = SolveReport::NOTHING;
    for piece in 1..=SOLVE_PIECES {
        match statics.solve(m, &start, loads, held, friction) {
            Err(Refusal::NotConverged { .. }) if piece < SOLVE_PIECES => {
                let (at, report) = statics.stopped();
                start = at.to_vec();
                before = before.continued(report);
            }
            Ok(mut eq) => {
                eq.report = before.continued(eq.report);
                return Ok(eq);
            }
            Err(Refusal::NotConverged {
                iterations,
                residual,
            }) => {
                return Err(Refusal::NotConverged {
                    iterations: before.iterations + iterations,
                    residual,
                });
            }
            Err(refusal) => return Err(refusal),
        }
    }
    unreachable!("the last piece returns")
}

/// What each contact row carries in one equilibrium of a mounted key: the
/// rows the static solve read, with the force each pushed with, and the
/// balance punching's push where the solve held the key on it.
///
/// One admissible split of the loads, the one the solve returned: where
/// friction or redundant rows share a load, rigid statics leave the split
/// open (`docs/FOUNDATIONS.md` > Parts and statics), and no verdict is read from it
/// ([`crate::parts::verdicts`]). It is what a view draws to show where the
/// key bears and roughly how hard.
#[derive(Debug, Clone, PartialEq)]
pub struct Carried {
    supports: Vec<crate::solve::Support>,
    /// The balance punching's pair and its push, newtons, where the solve
    /// held the key on its punching and the punching's rows are not among
    /// the supports.
    punching: Option<(u16, f64)>,
}

impl Carried {
    /// Nothing carried: a key that no equilibrium holds.
    pub const NOTHING: Self = Self {
        supports: Vec::new(),
        punching: None,
    };

    /// The force the row of `pair` meeting at `feature` pushes with,
    /// newtons: zero for a row the equilibrium left open or did not read.
    pub fn force(&self, pair: u16, feature: crate::contact::Feature) -> f64 {
        match self.punching {
            Some((p, push)) if p == pair => push,
            _ => self
                .supports
                .iter()
                .find(|s| s.by == Holder::Contact { pair, feature })
                .map_or(0.0, |s| s.force),
        }
    }

    /// The rows the equilibrium read, in the solve's order, each with its
    /// gap, its force and its normal.
    pub fn supports(&self) -> &[crate::solve::Support] {
        &self.supports
    }

    /// The balance punching's pair and its push, newtons, where the solve
    /// held the key on its punching and the punching's rows are not among
    /// the supports.
    pub fn punching(&self) -> Option<(u16, f64)> {
        self.punching
    }
}

impl Mounted {
    /// What each row carries in `eq`, an equilibrium of this key
    /// ([`Self::solve`]). Where the solve held the key on its punching, the
    /// slide's reaction is the punching's push straight up, and its
    /// multiplier along the underside's normal, tilted with the key, that
    /// reaction over the cosine of the key's angle.
    pub fn carried(&self, eq: &Equilibrium) -> Carried {
        let punching = self.lift.as_ref().and_then(|lift| {
            let listed = eq
                .supports
                .iter()
                .any(|s| matches!(s.by, Holder::Contact { pair, .. } if pair == lift.pair));
            let reaction = eq.reactions[lift.coord.0];
            (!listed && reaction != 0.0)
                .then(|| (lift.pair, reaction / libm::cos(eq.q[self.angle.0])))
        });
        Carried {
            supports: eq.supports.clone(),
            punching,
        }
    }
}

/// The key's nominal pose on its frame, where a static solve that seats it
/// by rule starts. It places the key only: a wippen the key carries stays
/// at its frame's angle, for the start of the solve to place
/// ([`crate::measure::seat::Seating::start`]), and the key's coordinates
/// are the same bits with it and without it.
///
/// On its balance punching, level, wherever its drawn parts overlap its
/// frame there by no more than the row margin; otherwise the nearest
/// attitude to level, a degree apart and nose-up before nose-down, at which
/// they do; either way an overlap a solve resolves through its rows. A key
/// free to lift that has no such attitude on its punching, its stops
/// fighting or a pin standing in its wood, is lowered onto its frame from
/// above instead, level unless another attitude clears its frame standing a
/// row margin lower off its punching: at each attitude the least lift that
/// clears, found by halving the slide's range to within the row margin, the
/// attitude narrowed
/// between two whole degrees, by golden-section steps, where no whole degree
/// clears. On a pivot, or where nothing clears below where the key leaves
/// its balance pin ([`Mounted::leaves_its_pin`]) at any attitude, whole or
/// narrowed, the attitude of least overlap on the punching, the nearest
/// level among equals.
///
/// The overlap is read off the drawn outlines, not the rows: a static solve
/// resolves an overlap by its contacts' rows, which read a part sunk deep
/// into another truthfully only near the surface, or not at all. It is the
/// deepest of what the declared pairs read ([`Mechanism::overlap`]), of how
/// deep the key stands in its front and back rails, which no pair declares,
/// and of how far below the keybed it reaches: a key turned until it has
/// gone right through a stop leaves the stop's parts clear, and only the
/// rail under the stop says it is there; turned further, through the rail
/// too, only the keybed the rails stand on does.
pub fn nominal_pose(mounted: &Mounted) -> Vec<f64> {
    let level = mounted.level();
    // The attitudes are scanned on the key held on its punching where it is
    // free to lift: the punching's pair reads nothing there, and the scan is
    // then the pivot's, to the bit.
    let on_punching = mounted
        .lift
        .as_ref()
        .and_then(|lift| lift.turning.as_ref())
        .unwrap_or(&mounted.mechanism);
    let (q, least) = attitude_of_least_overlap(mounted, on_punching, &level);
    match &mounted.lift {
        Some(lift) if least > ROW_MARGIN => {
            let (key, rails) = key_and_rails(mounted);
            lowered(mounted, lift, &level, (&key, &rails), ROW_MARGIN).unwrap_or(q)
        }
        _ => q,
    }
}

/// The wippen the key carries lowered onto the key ([`wippen_lowered`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Lowered {
    /// The coordinates: the key's as given, the wippen's angle where it
    /// was lowered to.
    pub q: Vec<f64>,
    /// How many times the degree it first touches in was halved, at most
    /// `LOWERING_HALVINGS`: nothing where it clears the key all the way
    /// down, and on a mount that carries no wippen.
    pub halvings: u32,
}

/// `q` with the wippen the key carries lowered onto the key where the key
/// stands, with the halvings it took: from the top of its range down, the
/// angle at which its heel or its heel cloth first touches the key's parts,
/// found a degree at a time (`NOMINAL_STEP`) and then by halving the degree
/// it first touches in until what is left of it moves the wippen's farthest
/// drawn point less than the row margin, so that it starts clear, within a
/// margin of where it first touches; at the bottom of its range where they
/// clear the key all the way down, out of the key's reach. `q` as it is on
/// a mount that carries no wippen. `None` where the heel or its cloth
/// stands in the key even at the top of the wippen's range: where the
/// wippen fits nowhere with the key standing there.
///
/// At the top of its range the heel is read twice: by its drawn outlines,
/// as every other angle is, and, where they clear the key, by the links'
/// rows ([`Mounted::measure_the_links_overlap`]). The drawn outlines take
/// a convex arc as eight chords standing inside it by its sag, 21.8 µm on
/// the test heel's cloth, and the static solve leans on the contacts,
/// which read the arc itself: a wippen the drawn outlines alone let start
/// at the top of its range, its cloth standing in the key, would be pushed
/// past its range by the solve, with nothing drawn to say how far.
///
/// A degree at a time, and not by halving the whole range: a heel lowered
/// far enough passes through a key part and out below it, clear again, and
/// only a heel that clears at every degree above clears the key all the way
/// down. A degree moves the heel by no more than the wippen's reach, its
/// farthest drawn point from the center pin ([`WippenOn::reach`]), times
/// that degree: about 2.2 mm on the test wippen's 126 mm, less than the
/// heel's own height, so no key part slips between two degrees unread.
///
/// That is the decision's resolution: a heel that grazes a key part
/// between two whole degrees, by less than the wippen's reach times half a
/// degree, reads clear, and the wippen out of the key's reach; so does one
/// whose cloth's arc grazes a key part at a whole degree by less than the
/// drawn chords stand inside the arcs that meet there, its own sag and a
/// domed capstan's added to it, below the top of its range, where only the
/// drawn outlines are read. Its nearest approach is read at the same angles
/// on the same outlines ([`Mounted::measure_the_heel_past`]), so the reading
/// and the decision agree.
///
/// Only the heel and its cloth are read, the parts the links carry the
/// wippen by: any other part of the wippen standing in the key is the
/// seat's to name ([`wippen_part_in_the_key`]). The key is never moved for
/// its wippen: a technician puts the action on a key that stands where it
/// stands.
pub fn wippen_lowered(mounted: &Mounted, q: &[f64]) -> Option<Lowered> {
    let Some(wippen) = &mounted.wippen else {
        return Some(Lowered {
            q: q.to_vec(),
            halvings: 0,
        });
    };
    let m = &mounted.mechanism;
    let (low, high) = m.coords()[wippen.angle.0].limits;
    let at = |angle: f64| {
        let mut at = q.to_vec();
        at[wippen.angle.0] = angle;
        at
    };
    let (heel, key) = find_the_heel_and_the_key(mounted, wippen);
    let key = m.outlines(&m.pose(q), &key);
    let clear = |angle: f64| m.outlines(&m.pose(&at(angle)), &heel).overlap(&key) <= 0.0;
    if !clear(high) || mounted.measure_the_links_overlap(&at(high)) > 0.0 {
        return None;
    }
    let (mut above, mut below) = (high, high);
    while clear(below) {
        if below <= low {
            return Some(Lowered {
                q: at(low),
                halvings: 0,
            });
        }
        above = below;
        below = step_down_a_whole_degree(below, low);
    }
    let mut halvings = 0;
    while (above - below) * wippen.reach > ROW_MARGIN && halvings < LOWERING_HALVINGS {
        let middle = 0.5 * (above + below);
        if clear(middle) {
            above = middle;
        } else {
            below = middle;
        }
        halvings += 1;
    }
    Some(Lowered {
        q: at(above),
        halvings,
    })
}

/// The angle a whole degree (`NOMINAL_STEP`) below `angle`, or `low`, the
/// bottom of the wippen's range, where that would pass it: the angles at
/// which the bench decides whether the wippen reaches the key, from the top
/// of its range down ([`wippen_lowered`]), and at which its heel's nearest
/// approach to the key is read out of reach
/// ([`Mounted::measure_the_heel_past`]), one step for both so that the
/// reading and the decision read the same angles to the bit.
pub fn step_down_a_whole_degree(angle: f64, low: f64) -> f64 {
    (angle - NOMINAL_STEP).max(low)
}

/// The wippen's heel and its heel cloth, the parts the links carry it by,
/// and the key's parts.
fn find_the_heel_and_the_key(mounted: &Mounted, wippen: &WippenOn) -> (Vec<PartId>, Vec<PartId>) {
    let heel = parts_where(mounted, &|p| {
        p.body == wippen.body && matches!(p.code, PartCode::WippenHeel | PartCode::WippenHeelCloth)
    });
    let key = parts_where(mounted, &|p| p.body == mounted.key);
    (heel, key)
}

/// How deep the wippen's heel and its heel cloth stand in the key's parts
/// at `q`, meters, read off their outlines: the parts the links carry the
/// wippen by, read whole, through the links and outside their ranges.
/// Nothing on a mount that carries no wippen.
pub fn heel_in_the_key(mounted: &Mounted, q: &[f64]) -> f64 {
    let Some(wippen) = &mounted.wippen else {
        return 0.0;
    };
    let (heel, key) = find_the_heel_and_the_key(mounted, wippen);
    let m = &mounted.mechanism;
    m.parts_overlap(&m.pose(q), &heel, &key)
}

/// A part of the wippen standing in a part of the key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartInTheKey {
    /// The wippen's part.
    pub part: PartCode,
    /// The key's part it stands in.
    pub key_part: PartCode,
    /// How deep, meters.
    pub depth: f64,
}

/// The deepest that one part of the wippen stands in one part of the key at
/// `q`, which two, and how deep, read off their outlines, every part of the
/// wippen against every part of the key, through the links and outside
/// their ranges; the first pair in the parts' order among equals. `None`
/// where none stands in any, and on a mount that carries no wippen. Each
/// part's outline is drawn once, and read against every part of the other
/// side.
pub fn wippen_part_in_the_key(mounted: &Mounted, q: &[f64]) -> Option<PartInTheKey> {
    let wippen = mounted.wippen.as_ref()?;
    let m = &mounted.mechanism;
    let pose = m.pose(q);
    let drawn = |ids: Vec<PartId>| -> Vec<(PartCode, crate::contact::Outlines)> {
        ids.into_iter()
            .map(|id| (m.parts()[id.index()].code, m.outlines(&pose, &[id])))
            .collect()
    };
    let key = drawn(parts_where(mounted, &|p| p.body == mounted.key));
    let mut deepest: Option<PartInTheKey> = None;
    for (part, outlines) in drawn(parts_where(mounted, &|p| p.body == wippen.body)) {
        for (key_part, key_outlines) in &key {
            let depth = outlines.overlap(key_outlines);
            if depth > 0.0 && deepest.is_none_or(|d| depth > d.depth) {
                deepest = Some(PartInTheKey {
                    part,
                    key_part: *key_part,
                    depth,
                });
            }
        }
    }
    deepest
}

/// The key lowered onto its frame from above, where it is free to lift: of
/// the attitudes [`nominal_pose`] scans, or narrowed between two of them by
/// golden-section steps where none clears, the one at which it stands clear
/// of every part of its frame, nothing overlapping at all, least far off its
/// punching, within the row margin above where it first touches. `None` on a
/// mount with no slide, and where nothing clears below where the key leaves
/// its balance pin ([`Mounted::leaves_its_pin`]): every pose clear of the
/// frame then has the key off its pin.
///
/// Clear of every part, the punchings' felts and papers whole: the pairs read
/// a punching at its center, which a key sunk right through a felt thicker
/// than the key leaves clear. A key tilted on a punching clears its felt's
/// drawn edge only half the washer's width times the tilt above where it
/// stands on the felt's center, so one riding that close under its pin's top
/// is not lowered onto its frame below it. The balance rail pin alone is
/// left out, which stands in its balance hole and its slot, where no contact
/// reads it.
///
/// The start a placement goes back to when the nominal pose, an attitude on
/// the punching whose overlap the rows read within their margin, gives the
/// key no seat: an overlap no motion of the key on its frame resolves, such
/// as a front rail pin standing a few tenths of a millimeter into the wood
/// beside its mortise, its tip well up in the slot and standing in the
/// slot's wall, which a turn draws back only through the wall's short arm
/// about the bearing and no lift short of the pin's top clears, so that
/// the solve has no motion or reaches its cap; or a nominal pose lowered
/// onto its frame at the top of the slide's range, a whole degree from
/// where the key rides below it, from which the solve carries the key off
/// its pin. A technician lowers such a key onto its frame, and it lands on
/// what stands highest. It places the key only, as [`nominal_pose`] does.
pub fn lowered_onto_its_frame(mounted: &Mounted) -> Option<Vec<f64>> {
    let lift = mounted.lift.as_ref()?;
    let (key, frame) = key_and_frame(mounted);
    lowered(mounted, lift, &mounted.level(), (&key, &frame), 0.0)
}

/// How deep the key stands in its frame at `q`, meters: what the pairs read,
/// the key's parts in every part of its frame but the balance rail pin
/// ([`lowered_onto_its_frame`]), and how far the key reaches below the
/// keybed.
fn sunk_in_its_frame(mounted: &Mounted, q: &[f64]) -> f64 {
    let (key, frame) = key_and_frame(mounted);
    overlap_at(mounted, &mounted.mechanism, &key, &frame, q)
}

/// How deep the key's drawn parts stand in its frame's at `q`, meters, read
/// as [`nominal_pose`] reads them: what the pairs read, the key in its front
/// and back rails, which no pair declares, and how far it reaches below the
/// keybed.
///
/// The depth an increment's start is held to: a rail raised into the key
/// where it stood, its cloth clear above the key, is read here and by no
/// row.
pub fn overlap_on_its_frame(mounted: &Mounted, q: &[f64]) -> f64 {
    let (key, rails) = key_and_rails(mounted);
    overlap_at(mounted, &mounted.mechanism, &key, &rails, q)
}

/// The parts of `mounted` that `keep` keeps, in their order.
fn parts_where(mounted: &Mounted, keep: &dyn Fn(&Part) -> bool) -> Vec<PartId> {
    let m = &mounted.mechanism;
    (0..m.parts().len())
        .filter(|&i| keep(&m.parts()[i]))
        .map(|i| PartId(u16::try_from(i).expect("fewer than 65 536 parts")))
        .collect()
}

/// The parts of `mounted`'s key, and its frame's front and back rails, which
/// no pair declares.
fn key_and_rails(mounted: &Mounted) -> (Vec<PartId>, Vec<PartId>) {
    let key = parts_where(mounted, &|p| p.body == mounted.key);
    let rails = parts_where(mounted, &|p| {
        p.body != mounted.key && matches!(p.code, PartCode::FrontRail | PartCode::BackRail)
    });
    (key, rails)
}

/// The parts of `mounted`'s key, and every part of its frame but the balance
/// rail pin, which stands in its balance hole and its slot, where no contact
/// reads it: the parts on the ground, the center pin a wippen turns on
/// apart, which is the action's.
fn key_and_frame(mounted: &Mounted) -> (Vec<PartId>, Vec<PartId>) {
    let key = parts_where(mounted, &|p| p.body == mounted.key);
    let frame = parts_where(mounted, &|p| {
        p.body == BodyId::GROUND
            && !matches!(p.code, PartCode::BalanceRailPin | PartCode::CenterPin)
    });
    (key, frame)
}

/// How deep the drawn parts of the key and its frame stand in one another at
/// `q`, read on `m`, `mounted`'s mechanism or the key held on its punching:
/// what its pairs read, the links to a wippen apart, the key in its rails,
/// and how far the key's outline reaches below the keybed, the ground's
/// level zero.
fn overlap_at(
    mounted: &Mounted,
    m: &Mechanism,
    key: &[PartId],
    rails: &[PartId],
    q: &[f64],
) -> f64 {
    let pose = m.pose(q);
    let links = mounted.wippen.as_ref().map_or(&[][..], |w| &w.links[..]);
    let pairs = (0..m.pairs().len())
        .map(|p| u16::try_from(p).expect("fewer than 65 536 pairs"))
        .filter(|p| !links.contains(p))
        .map(|p| m.pair_overlap(&pose, p))
        .fold(0.0, f64::max);
    let below = key
        .iter()
        .flat_map(|id| {
            let part = &m.parts()[id.index()];
            let placement = pose.placement(part.body);
            part.silhouette()
                .into_iter()
                .map(move |seg| -placement.world(seg.start()).y)
        })
        .fold(0.0, f64::max);
    pairs.max(m.parts_overlap(&pose, key, rails)).max(below)
}

/// The attitude nearest `level` at which the drawn parts overlap by no more
/// than the row margin, read on `m`, a degree at a time, nose-up first; or
/// the attitude of least overlap. With how deep they overlap there.
fn attitude_of_least_overlap(mounted: &Mounted, m: &Mechanism, level: &[f64]) -> (Vec<f64>, f64) {
    let (lo, hi) = m.coords()[mounted.angle.0].limits;
    let (key, rails) = key_and_rails(mounted);
    let at = |angle: f64| {
        let mut q = level.to_vec();
        q[mounted.angle.0] = angle;
        q
    };
    let overlap = |angle: f64| overlap_at(mounted, m, &key, &rails, &at(angle));
    let mut least = (0.0, overlap(0.0));
    let steps = (hi.min(-lo) / NOMINAL_STEP) as i32;
    'scan: for k in 1..=steps {
        for angle in [f64::from(k) * NOMINAL_STEP, -f64::from(k) * NOMINAL_STEP] {
            if least.1 <= ROW_MARGIN {
                break 'scan;
            }
            let o = overlap(angle);
            if o < least.1 {
                least = (angle, o);
            }
        }
    }
    (at(least.0), least.1)
}

/// The key lowered onto its frame from above: of the attitudes the scan
/// visits, level first and then a degree at a time, nose-up before
/// nose-down, the one at which the key clears its frame standing least far
/// off its punching, and that lift. At each attitude the lift that clears,
/// nothing overlapping by more than `within`, the key's parts read against
/// the frame's `parts.1` as well as through the pairs, is found by halving
/// the slide's range until what is left of it is under the row margin, so
/// the key starts within a margin of where it first touches; the first
/// attitude is kept, level before any other, unless another clears the
/// frame more than a margin lower.
///
/// Where no attitude a degree apart clears below the slide's upper limit,
/// where the key leaves its balance pin, the attitude is narrowed instead:
/// within a degree either side of the one at which the key, at that limit,
/// overlaps its frame least, by [`ATTITUDE_REFINEMENTS`] golden-section
/// steps toward the attitude of least overlap there. A key riding on two
/// stops clears them only near the attitude at which it rides, and each
/// whole degree either side of it costs the angle between them times the
/// stops' arms of lift, up to a millimeter or two: a key riding that close
/// under its pin's top clears its frame below it at no whole degree. `None`
/// where the key clears its frame below that limit at no attitude, whole or
/// narrowed: it clears it only off its pin.
///
/// The key lowered least far stands nearest the stops it will rest on, so a
/// key whose stops fight starts near where it rides, and the static solve
/// has only a margin or two to bring it down onto them.
fn lowered(
    mounted: &Mounted,
    lift: &Lift,
    level: &[f64],
    parts: (&[PartId], &[PartId]),
    within: f64,
) -> Option<Vec<f64>> {
    let m = &mounted.mechanism;
    let (key, frame) = parts;
    let (lo, hi) = m.coords()[mounted.angle.0].limits;
    let (_, top) = m.coords()[lift.coord.0].limits;
    let steps = (hi.min(-lo) / NOMINAL_STEP) as i32;
    let at = |angle: f64, h: f64| {
        let mut q = level.to_vec();
        q[mounted.angle.0] = angle;
        q[lift.coord.0] = h;
        q
    };
    let overlap = |q: &[f64]| overlap_at(mounted, m, key, frame, q);
    let clear = |q: &[f64]| overlap(q) <= within;
    // The least lift that clears at `angle`, to within the row margin, the
    // key clearing its frame at the slide's upper limit.
    let lowest = |angle: f64| {
        let (mut low, mut high) = (lift.on_punching, top);
        while high - low > ROW_MARGIN {
            let middle = 0.5 * (low + high);
            if clear(&at(angle, middle)) {
                high = middle;
            } else {
                low = middle;
            }
        }
        high
    };
    let angles = core::iter::once(0.0).chain(
        (1..=steps).flat_map(|k| [f64::from(k) * NOMINAL_STEP, -f64::from(k) * NOMINAL_STEP]),
    );
    let mut best: Option<(f64, Vec<f64>)> = None;
    // The attitude at which the key overlaps its frame least at the slide's
    // upper limit, and by how much: where the narrowing starts.
    let mut least: Option<(f64, f64)> = None;
    for angle in angles {
        let deep = overlap(&at(angle, top));
        if least.is_none_or(|(_, o)| deep < o) {
            least = Some((angle, deep));
        }
        if deep > within {
            continue;
        }
        let high = lowest(angle);
        // Each lift is found to within the row margin, so two attitudes
        // whose lifts differ by less are told apart by where the halving's
        // steps fall, which the slide's range sets, and not by the frame:
        // the first, level before any other, stays unless another clears a
        // margin lower.
        if best
            .as_ref()
            .is_none_or(|(lowest, _)| high < *lowest - ROW_MARGIN)
        {
            best = Some((high, at(angle, high)));
        }
    }
    if best.is_none() {
        let (around, _) = least?;
        let (mut a, mut b) = (
            (around - NOMINAL_STEP).max(lo),
            (around + NOMINAL_STEP).min(hi),
        );
        let mut c = b - GOLDEN * (b - a);
        let mut d = a + GOLDEN * (b - a);
        let (mut at_c, mut at_d) = (overlap(&at(c, top)), overlap(&at(d, top)));
        for _ in 0..ATTITUDE_REFINEMENTS {
            if at_c <= at_d {
                b = d;
                d = c;
                at_d = at_c;
                c = b - GOLDEN * (b - a);
                at_c = overlap(&at(c, top));
            } else {
                a = c;
                c = d;
                at_c = at_d;
                d = a + GOLDEN * (b - a);
                at_d = overlap(&at(d, top));
            }
        }
        let angle = if at_c <= at_d { c } else { d };
        if clear(&at(angle, top)) {
            let high = lowest(angle);
            best = Some((high, at(angle, high)));
        }
    }
    best.map(|(_, q)| q)
}

/// The key on its frame, turning about its balance bearing
/// ([`KeyMount::Pivot`]): the ground, which carries the frame, and the key
/// hung from it by a revolute joint whose axis is `bearing`, in the keybed's
/// frame, where the key's frame has its origin; its angle bounded by
/// [`KEY_ANGLE_LIMIT`] either way, and its bushings' dry friction, a torque
/// bound `friction` in newton-meters, acting on it.
///
/// `parts` are the frame's on the ground and the key's on body 1, as
/// [`super::FrameDesign::parts`] and [`super::KeyDesign::parts`] build them,
/// and a wippen's on body 2 where `wippen` hangs one there; `pairs` the
/// contacts between them ([`key_on_frame`]), and the wippen's links where
/// it has one. The key bears at its balance pin, on the underside, which is
/// where every key the bench builds bears.
pub fn pivot(
    mass: Option<MassProps>,
    bearing: Point,
    friction: f64,
    parts: Vec<Part>,
    pairs: Vec<ContactPair>,
    wippen: Option<WippenHung>,
) -> Result<Mounted, Problem> {
    let key = Body {
        parent: BodyId::GROUND,
        joint: Joint::Revolute {
            pivot: bearing,
            coord: CoordId(0),
        },
        mass,
    };
    let angle = Coord::ANGLE
        .limited(-KEY_ANGLE_LIMIT, KEY_ANGLE_LIMIT)
        .with_friction(friction);
    let (mut bodies, mut coords) = (vec![Body::GROUND, key], vec![angle]);
    let wippen = wippen.map(|hung| hang(&mut bodies, &mut coords, hung, &parts));
    let mechanism = Mechanism::new(bodies, coords)?.with_contacts(parts, pairs)?;
    Ok(Mounted {
        mechanism,
        key: BodyId(1),
        angle: CoordId(0),
        lift: None,
        wippen,
        key_alone: None,
    })
}

/// The key on its frame, free to lift off its balance punching
/// ([`KeyMount::OnPunching`]): the ground, which carries the frame; a
/// massless slide, body 1, standing on the keybed at `axis_x` along it and
/// traveling up the balance pin's axis, its coordinate the height of the
/// key frame's origin above the keybed, from the keybed to `pin_top`, with
/// no friction; and the key, body 2, turning about the slide's origin, its
/// angle bounded by [`KEY_ANGLE_LIMIT`] either way, its bushings' dry
/// friction, a torque bound `friction` in newton-meters, acting on it.
///
/// `parts` are the frame's on the ground and the key's on body 2, and a
/// wippen's on body 3 where `wippen` hangs one there; `pairs` the key's
/// contacts with its frame but the punching's ([`key_on_frame`]), and the
/// wippen's links where it has one, and `punching` the balance punching's
/// ([`balance_punching`]), declared last. `bearing` is the punching's drawn top: the key stands on its
/// punching with the slide there. `pin_top` is where the key leaves its
/// balance pin: the slide's height at which the key's balance hole, on the
/// pin's axis at the key frame's origin, clears the pin's drawn top
/// ([`on_its_frame`] reads it off the drawn pin). Past it the key has left
/// its pin and its frame: an increment that leans on it has no equilibrium
/// ([`Mounted::lifts_off_its_frame`]). It may stand below `bearing`, a
/// punching taller than its pin: no pose then has the key on its pin.
///
/// The slide is frictionless, a simplification: the bushings' friction
/// against the key sliding along its pin acts only while the key leaves its
/// punching, a state the procedures report and never weigh.
#[allow(clippy::too_many_arguments)]
pub fn on_punching(
    mass: Option<MassProps>,
    axis_x: f64,
    bearing: f64,
    pin_top: f64,
    friction: f64,
    parts: Vec<Part>,
    pairs: Vec<ContactPair>,
    punching: ContactPair,
    wippen: Option<WippenHung>,
) -> Result<Mounted, Problem> {
    let slide = Body {
        parent: BodyId::GROUND,
        joint: Joint::Prismatic {
            origin: Point::new(axis_x, 0.0),
            axis: Dir::Y,
            coord: CoordId(0),
        },
        mass: None,
    };
    let key = Body {
        parent: BodyId(1),
        joint: Joint::Revolute {
            pivot: Point::new(0.0, 0.0),
            coord: CoordId(1),
        },
        mass,
    };
    let mut bodies = vec![Body::GROUND, slide, key];
    let mut coords = vec![
        Coord::LENGTH.limited(0.0, pin_top),
        Coord::ANGLE
            .limited(-KEY_ANGLE_LIMIT, KEY_ANGLE_LIMIT)
            .with_friction(friction),
    ];
    let wippen = wippen.map(|hung| hang(&mut bodies, &mut coords, hung, &parts));
    let others = pairs.clone();
    let mut all = pairs;
    all.push(punching);
    let pair = u16::try_from(all.len() - 1).expect("fewer than 65 536 pairs");
    let mechanism =
        Mechanism::new(bodies.clone(), coords.clone())?.with_contacts(parts.clone(), all)?;
    // The punching's center, in the ground's frame, where its pair reads it.
    let center = mechanism.features[usize::from(pair)].b.center.1;
    let on_the_axis =
        center.x.to_bits() == axis_x.to_bits() && center.y.to_bits() == bearing.to_bits();
    let turning = if on_the_axis {
        Some(Mechanism::new(bodies, coords)?.with_contacts(parts, others)?)
    } else {
        None
    };
    Ok(Mounted {
        mechanism,
        key: BodyId(2),
        angle: CoordId(1),
        lift: Some(Lift {
            coord: CoordId(0),
            on_punching: bearing,
            pair,
            turning,
        }),
        wippen,
        key_alone: None,
    })
}

/// A key on its frame as the procedures read it: the key in its mount, its
/// contacts with its frame, what the two sets of parts declare, and the
/// points of the key the procedures load and read.
#[derive(Debug, Clone, PartialEq)]
pub struct OnItsFrame {
    /// How the key stands on its frame.
    pub mount: KeyMount,
    /// The mechanism: the frame's parts on the ground, the key's on its body.
    pub mounted: Mounted,
    /// What the key's parts declare.
    pub key: KeyRanges,
    /// What the frame's parts declare.
    pub frame: FrameRanges,
    /// The capstan's contact point, in the key's frame: where a load
    /// standing for the action acts.
    pub capstan: Point,
    /// The key front's top, in the key's frame: where key height and dip are
    /// read.
    pub front_top: Point,
    /// What the wippen's parts declare, where the key carries one.
    pub wippen: Option<WippenAssemblyRanges>,
}

/// `key` on `frame` in `mount`, its key standing on the row of pins of
/// `kind`, the papers its punchings hold drawn on them; the key's mass
/// `mass`, `None` for a key nobody has weighed; its bushings' friction a
/// torque bound of `friction` newton-meters; carrying `wippen` on its
/// capstan, with what is known of its mass, where it carries one.
///
/// The one place a key is put on its frame for a procedure to read, so that
/// the bench's readings and the regulation solve that lays its papers read
/// the same mechanism, part for part and bit for bit. The frame's parts come
/// first, on the ground, then the key's, on its mount's body, then the
/// wippen's, on the body after the key's, and its center pin on the ground.
/// The key turns about its own balance pin, on its row: the natural's at the
/// origin of the keybed's frame, the sharp's the row's offset behind it,
/// where the frame draws that pin and its punching. The wippen turns on its
/// center pin, and rests on the key through its two links, the key on side
/// A of both, declared after the key's contacts with its frame and before
/// its balance punching ([`wippen_links`]).
pub fn on_its_frame(
    mount: KeyMount,
    key: &KeyDesign,
    frame: &FrameDesign,
    kind: KeyKind,
    mass: Option<MassProps>,
    friction: f64,
    wippen: Option<(&WippenAssembly, &WippenAssemblyMass)>,
) -> Result<OnItsFrame, Problem> {
    let mut parts = Vec::new();
    let frame_ranges = frame.parts(key, kind, &mut parts);
    let key_ranges = key.parts(mount.key_body(), &mut parts);
    let capstan = key_ranges.capstan_contact.point(&parts);
    let front_top = key_ranges.key_front_top.point(&parts);
    let mut pairs = key_on_frame(&key_ranges, &frame_ranges);
    let (wippen_ranges, hung) = match wippen {
        Some((assembly, wippen_mass)) => {
            let ranges = assembly.parts(mount.wippen_body(), BodyId::GROUND, &mut parts);
            let first = u16::try_from(pairs.len()).expect("fewer than 65 536 pairs");
            pairs.extend(wippen_links(&key_ranges, &ranges));
            let hung = WippenHung {
                center: assembly.center,
                mass: wippen_mass.props(assembly),
                friction: assembly.friction.bound_n_m,
                links: [first, first + 1],
            };
            (Some(ranges), Some(hung))
        }
        None => (None, None),
    };
    let bearing = frame.balance_bearing_height();
    let pin = frame.sections.balance.pin_x(0.0, kind);
    let key_alone = match wippen {
        Some(_) => Some(Box::new(
            on_its_frame(mount, key, frame, kind, mass, friction, None)?.mounted,
        )),
        None => None,
    };
    let mut mounted = match mount {
        KeyMount::Pivot => pivot(mass, Point::new(pin, bearing), friction, parts, pairs, hung)?,
        KeyMount::OnPunching => {
            let punching = balance_punching(&key_ranges, &frame_ranges);
            // Where the key leaves its balance pin: the pin's drawn top,
            // which the key's balance hole, at the key frame's origin on the
            // pin's axis, clears with the slide there. A rail with no pin
            // guides nothing: a key lifted off its punching has left its
            // frame.
            let pin_top = frame_ranges
                .balance_rail_pin
                .map_or(bearing, |id| drawn_top(&parts[id.index()]));
            on_punching(
                mass, pin, bearing, pin_top, friction, parts, pairs, punching, hung,
            )?
        }
    };
    mounted.key_alone = key_alone;
    Ok(OnItsFrame {
        mount,
        mounted,
        key: key_ranges,
        frame: frame_ranges,
        capstan,
        front_top,
        wippen: wippen_ranges,
    })
}

/// The two links between a key and the wippen it carries, the key on side
/// A of both, each a pair of the ranges the two sets of parts declare, read
/// as two drawn outlines ([`PairGeometry::Features`]):
///
/// - **The capstan on the heel**: the capstan's whole drawn outline above
///   the wood, its top, flat or domed, its rims and the sides of its head,
///   against the heel's outline below the wippen's body, cloth and wood: its
///   back side, its bottom behind the cloth, the cloth's outer face, its
///   bottom in front of the cloth and its front side, one chain. The trade
///   wants the capstan on the cloth; a capstan off it bears on the heel's
///   wood, a fault of its place or of the action, which the contact names
///   from the part it bears on and does not refuse.
/// - **The heel on the key**: the key's top the heel can reach, the
///   keystick's top, its step down to the thinned tail and the tail's, and
///   the back check block, against the same outline of the heel: where the
///   heel comes down when the capstan stands out from under it.
///
/// Both read the heel's sides as well as its bottom: a range read up to a
/// corner and no further leaves the face beyond it unread, and a corner
/// gives a row only within its sector, so a capstan pressing on that face
/// would have no row and the heel would sink where nothing reads it.
///
/// Neither is a stop: what holds the key, where its stroke ends and which
/// of its stops fight never name a link ([`ContactCode::is_link`]).
pub fn wippen_links(key: &KeyRanges, wippen: &WippenAssemblyRanges) -> [ContactPair; 2] {
    [
        pair(
            ContactCode::CapstanOnTheHeel,
            Surface::one(key.capstan_above_the_wood.clone()),
            Surface::of(&wippen.heel_below_the_body),
            PairGeometry::Features,
        ),
        pair(
            ContactCode::HeelOnTheKey,
            Surface::of(&key.top_under_the_heel),
            Surface::of(&wippen.heel_below_the_body),
            PairGeometry::Features,
        ),
    ]
}

/// The height of a part's drawn top, in its body's frame: the highest point
/// of its silhouette, its arcs read where they turn over.
fn drawn_top(part: &Part) -> f64 {
    part.silhouette()
        .iter()
        .map(|seg| {
            let ends = seg.start().y.max(seg.end().y);
            match *seg {
                crate::geometry::Seg::Arc { c, r, from, to }
                    if crate::geometry::signed::in_sector(Point::new(0.0, 1.0), from, to) =>
                {
                    ends.max(c.y + r)
                }
                _ => ends,
            }
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

/// A rigid contact between two surfaces.
fn pair(code: ContactCode, a: Surface, b: Surface, geometry: PairGeometry) -> ContactPair {
    ContactPair {
        code,
        a,
        b,
        geometry,
        law: ContactLaw::Rigid,
    }
}

/// The key's contacts with its frame but its balance punching, each a pair
/// of the ranges the two sets of parts declare, the key on side A.
///
/// - **The front rail punching**: the felt's drawn top, read at its center,
///   against the underside's flat ([`PairGeometry::CenterBearing`]): a key
///   landing level loads the whole washer, and the pair says the load is at
///   its middle.
/// - **The back rail cloth**: the cloth's drawn top, its face and its two
///   rounded edges, and the sides they lead into, against the whole
///   underside, chamfer, kink and flat ([`PairGeometry::Features`]): flat on
///   the cloth, on its edge, or on the key's own corner.
/// - **The front rail pin in its mortise**: the pin's rounded tip against
///   the top of the mortise, a cut presenting its inside.
/// - **The front rail pin on the wood**: the pin's rounded tip against the
///   wood round the mortise: the underside beside it, which a
///   pin standing outside its mortise but under the key reaches; the corners
///   of the wood at the slot's mouth, which one straddling the slot's edge
///   meets; and the slot's walls along the key, which one standing off the
///   middle of its slot meets as the key turns, the slot moving along the
///   pin by its height above the bearing times the turn.
///
/// The two pin pairs are declared only where the front rail has a pin. Which
/// of the front pairs stops the key is the static solve's to find
/// ([`crate::measure::seat`]).
pub fn key_on_frame(key: &KeyRanges, frame: &FrameRanges) -> Vec<ContactPair> {
    let mut pairs = vec![
        pair(
            ContactCode::FrontRailPunching,
            Surface::of(&key.underside_flat),
            Surface::one(frame.front_bearing.clone()),
            PairGeometry::CenterBearing,
        ),
        pair(
            ContactCode::BackRailCloth,
            Surface::of(&key.underside),
            Surface::one(frame.cloth_and_sides.clone()),
            PairGeometry::Features,
        ),
    ];
    if let Some(pin) = &frame.front_pin_top {
        pairs.push(pair(
            ContactCode::FrontPinInTheMortise,
            Surface::one(key.front_mortise_top.clone()),
            Surface::one(pin.clone()),
            PairGeometry::Features,
        ));
        if !key.beside_the_mortise.is_empty() {
            pairs.push(pair(
                ContactCode::FrontPinOnTheWood,
                Surface::of(&key.beside_the_mortise),
                Surface::one(pin.clone()),
                PairGeometry::Features,
            ));
        }
    }
    pairs
}

/// The key's balance punching: the felt's drawn top, read at its center, on
/// the balance pin's axis, against the underside's flat
/// ([`PairGeometry::CenterBearing`]), the key on side A.
///
/// A felt washer under a key spreads its load, and the pair says the load is
/// at the washer's middle, where the key's frame has its origin: a rigid
/// stand-in, whose drawn edge overlaps a key tilted on it by what a felt
/// would compress, half the washer's width times the tilt
/// ([`crate::parts::verdicts::edge_overlaps`]). The key bears on the
/// punching beside its balance mortise, out of the sagittal plane, so the
/// pair reads the whole flat.
pub fn balance_punching(key: &KeyRanges, frame: &FrameRanges) -> ContactPair {
    pair(
        ContactCode::BalanceRailPunching,
        Surface::of(&key.underside_flat),
        Surface::one(frame.balance_bearing.clone()),
        PairGeometry::CenterBearing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact::oracle::{self, SPACING};
    use crate::contact::{Feature, HowItBears, Piece, Rows, Surface};
    use crate::mechanism::Pose;
    use crate::parts::fixtures::{on_frame, Setup};
    use crate::parts::frame::KeyKind;
    use crate::solve::Statics;

    /// The test key regulated on its frame, as the bench builds it.
    fn regulated() -> Setup {
        crate::parts::fixtures::regulated()
    }

    /// The regulated bench, its cloth holding the tail 5 mm above the balance
    /// bearing while its front punching stands level with it: a frame whose
    /// stops fight, as the golden sheet's fixture builds it.
    fn wedged() -> Setup {
        let mut a = regulated();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        a
    }

    /// The regulated bench, and the ways the golden sheet's fixtures and the
    /// bearing cases push it: a pin bottoming in its mortise, a key
    /// shortened under its frame, a cloth hung off the end of the key, a
    /// cloth reaching in front of the chamfer.
    fn assemblies() -> Vec<(&'static str, Setup)> {
        let base = regulated();
        let mut proud = base.clone();
        proud.frame.sections.front.pin_height += 0.001;
        let mut short = base.clone();
        short.key.front_edge -= 0.018;
        let mut overhung = base.clone();
        overhung.frame.back_rail = -0.232;
        let mut forward = base.clone();
        forward.frame.back_rail = -0.170;
        vec![
            ("the regulated bench", base),
            ("a pin bottoming in its mortise", proud),
            ("a key shortened under its frame", short),
            ("a cloth hung off the end of the key", overhung),
            ("a cloth in front of the chamfer", forward),
        ]
    }

    /// The assembly's parts as the bench draws them, the contacts between
    /// them, and the key on them, free to lift off its punching.
    fn mounted(a: &Setup) -> (Mounted, KeyRanges, FrameRanges) {
        let k = on_its_frame(
            KeyMount::OnPunching,
            &a.key,
            &a.frame,
            KeyKind::Natural,
            None,
            0.0,
            None,
        )
        .expect("the key's contacts are drawn");
        (k.mounted, k.key, k.frame)
    }

    /// The key on its punching at the angle `theta`, radians nose-up.
    fn on_it(m: &Mounted, theta: f64) -> Vec<f64> {
        let mut q = m.level();
        q[m.angle.0] = theta;
        q
    }

    /// Every row of the mounted key at `q`.
    fn rows_at(m: &Mounted, q: &[f64]) -> (Pose, Rows) {
        let pose = m.mechanism.pose(q);
        let mut rows = Rows::new();
        m.mechanism.rows(&pose, f64::INFINITY, &mut rows);
        (pose, rows)
    }

    fn pair_of(m: &Mounted, code: ContactCode) -> u16 {
        let i = m
            .mechanism
            .pairs()
            .iter()
            .position(|p| p.code == code)
            .expect("the key declares it");
        u16::try_from(i).unwrap()
    }

    /// The key's coordinates at its top seat and at its bottom one, by the
    /// seated pose; `None` where the key cannot be seated.
    fn seated(a: &Setup) -> Option<(Vec<f64>, Vec<f64>)> {
        let k = on_frame(a, KeyKind::Natural, 0.0, true);
        let seats = k.seating().seats(&mut Statics::new()).ok()?;
        Some((seats.top.q, seats.bottom.q))
    }

    /// The key's coordinates at its top seat.
    fn rest(a: &Setup) -> Vec<f64> {
        let k = on_frame(a, KeyKind::Natural, 0.0, true);
        k.seating()
            .top(&mut Statics::new())
            .expect("the key rests on its cloth")
            .q
    }

    /// The edges of the back rail cloth's face, meters along the key: the
    /// back one and the front one, where the drawn arcs are centered.
    fn cloth_edges(a: &Setup) -> (f64, f64) {
        let section = a.frame.sections.back;
        let center = section.center_x(a.frame.back_rail);
        (center - section.half_length, center + section.half_length)
    }

    /// The test wippen and the settings the contact tests read it at, each
    /// a millimeter's run of the sweeps that measure where the capstan bears
    /// (`crate::measure::seat_wippen`): on the regulated bench, on its
    /// cloth; the capstan moved back along the key to where its contact
    /// passes from the cloth onto the heel's bottom behind it at the bottom
    /// seat; the wippen's center moved back to where it passes onto the
    /// heel's bottom in front of the cloth at the top seat, and forward to
    /// where it passes off the cloth's front end; and moved so far back that
    /// the capstan stands out from under the heel, which comes down on the
    /// key's top.
    fn wippen_settings() -> Vec<(&'static str, Setup, WippenAssembly)> {
        let (wippen, _) = crate::parts::fixtures::test_wippen();
        let at = |behind_mm: f64| {
            let mut moved = wippen.clone();
            moved.center.x = -behind_mm * 1e-3;
            moved
        };
        let mut back = regulated();
        back.key.capstan = -0.1275;
        vec![
            (
                "the test wippen on the regulated bench",
                regulated(),
                wippen.clone(),
            ),
            (
                "the capstan 127.5 mm behind the balance pin",
                back,
                wippen.clone(),
            ),
            (
                "the wippen's center 178 mm behind the balance pin",
                regulated(),
                at(178.0),
            ),
            (
                "the wippen's center 195.5 mm behind the balance pin",
                regulated(),
                at(195.5),
            ),
            (
                "the wippen's center 162 mm behind the balance pin",
                regulated(),
                at(162.0),
            ),
        ]
    }

    /// The key carrying the test wippen at each of [`wippen_settings`], at
    /// its two seats and mid-stroke between them.
    fn wippen_poses() -> Vec<(String, Mounted, Vec<f64>)> {
        let mut poses = Vec::new();
        for (name, a, wippen) in wippen_settings() {
            let k = crate::parts::fixtures::carrying_the(&a, &wippen);
            let seats = k
                .seating()
                .seats(&mut Statics::new())
                .unwrap_or_else(|why| panic!("{name}: the key seats with its wippen: {why:?}"));
            let middle: Vec<f64> = seats
                .top
                .q
                .iter()
                .zip(&seats.bottom.q)
                .map(|(t, b)| 0.5 * (t + b))
                .collect();
            for (at, q) in [
                ("at rest", seats.top.q),
                ("mid-stroke", middle),
                ("at the bottom", seats.bottom.q),
            ] {
                poses.push((format!("{name}, {at}"), k.mounted.clone(), q));
            }
        }
        poses
    }

    /// Consistency: **a wippen is carried by two links, each a pair of
    /// ranges its parts declare**: the capstan's outline above the wood,
    /// then the key's top the heel can reach, each against the heel's
    /// outline below the wippen's body, cloth and wood, the key on side A
    /// of both. They come after the key's own pairs, which are those of the
    /// key alone, and before the balance punching, which stays the last;
    /// the key alone is mounted beside it with the same parts but the
    /// wippen's, and its own pairs.
    #[test]
    fn a_wippen_is_carried_by_two_links_of_ranges_its_parts_declare() {
        let a = regulated();
        let (wippen, mass) = crate::parts::fixtures::test_wippen();
        let k = on_its_frame(
            KeyMount::OnPunching,
            &a.key,
            &a.frame,
            KeyKind::Natural,
            None,
            0.0,
            Some((&wippen, &mass)),
        )
        .expect("the key mounts with its wippen");
        let heel = k
            .wippen
            .as_ref()
            .expect("the wippen's ranges")
            .heel_below_the_body
            .clone();
        let m = &k.mounted;
        let alone = m.key_alone.as_deref().expect("the key alone is mounted");
        let codes = |m: &Mounted| {
            m.mechanism
                .pairs()
                .iter()
                .map(|p| p.code)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            codes(m),
            [
                ContactCode::FrontRailPunching,
                ContactCode::BackRailCloth,
                ContactCode::FrontPinInTheMortise,
                ContactCode::FrontPinOnTheWood,
                ContactCode::CapstanOnTheHeel,
                ContactCode::HeelOnTheKey,
                ContactCode::BalanceRailPunching,
            ]
        );
        assert_eq!(codes(alone), codes(&mounted(&a).0));
        let pairs = m.mechanism.pairs();
        for (key_pair, alone_pair) in pairs.iter().zip(alone.mechanism.pairs()).take(4) {
            assert_eq!(key_pair, alone_pair, "{:?}", key_pair.code);
        }
        let links = m.wippen.as_ref().expect("the wippen is hung").links;
        assert_eq!(links, [4, 5]);
        let capstan = &pairs[usize::from(links[0])];
        assert_eq!(
            capstan.a,
            Surface::one(k.key.capstan_above_the_wood.clone())
        );
        assert_eq!(capstan.b, Surface::of(&heel));
        let top = &pairs[usize::from(links[1])];
        assert_eq!(top.a, Surface::of(&k.key.top_under_the_heel));
        assert_eq!(top.b, Surface::of(&heel));
        for pair in pairs {
            for side in [&pair.a, &pair.b] {
                for range in &side.ranges {
                    let drawn = m.mechanism.parts()[range.part.index()].silhouette().len();
                    assert!(usize::from(range.segs.end) <= drawn, "{:?}", pair.code);
                }
            }
        }
        let lift = m.lift.as_ref().expect("the key is free to lift");
        assert_eq!(usize::from(lift.pair), pairs.len() - 1);
        assert_eq!(
            &m.mechanism.parts()[..alone.mechanism.parts().len()],
            alone.mechanism.parts()
        );
    }

    /// Consistency: **the links' gradients against central differences**,
    /// over the slide, the key's angle and the wippen's angle, for every
    /// row of either link within the row margin, at each of the test
    /// wippen's poses ([`wippen_poses`]), the capstan's contact at each end
    /// of the cloth among them, where it passes between the cloth and the
    /// heel's bottom.
    ///
    /// The bound is derived as the rows' own test derives it
    /// (`crate::contact::rows`, `every_row_gradient_and_slip_match_central_differences`):
    /// a coordinate turning one side swings its point on a sinusoid of
    /// amplitude at most `L`, the point's reach from the joints that move
    /// it, the gap and the largest radius added, so a central difference
    /// over `±h` is off by `h²L/6`; between two discs, by `h²/6` of
    /// `6ρ³/f² + 6ρ²/f + ρ`, `ρ ≤ L` and `f` the least distance between
    /// their centers, at least their radii less the row margin; on the
    /// slide, by nothing. The gaps' rounding adds `256 ε L / h`, the
    /// gradient's `128 ε L`.
    #[test]
    fn the_links_gradients_match_central_differences() {
        let h = 1e-6;
        let eps = f64::EPSILON;
        let mut checked = 0;
        for (name, m, q) in wippen_poses() {
            let wippen = m.wippen.expect("a wippen is hung");
            let (pose, rows) = rows_at(&m, &q);
            let radius = |piece: Piece| match piece {
                Piece::Arc { part, seg } => {
                    match m.mechanism.parts()[part.index()].silhouette()[usize::from(seg)] {
                        crate::geometry::Seg::Arc { r, .. } => r,
                        crate::geometry::Seg::Line { .. } => 0.0,
                    }
                }
                _ => 0.0,
            };
            let joint = |body: BodyId| pose.placement(body).world(Point::new(0.0, 0.0));
            let distance =
                |p: Point, o: Point| ((p.x - o.x) * (p.x - o.x) + (p.y - o.y) * (p.y - o.y)).sqrt();
            for (i, row) in rows.rows().iter().enumerate() {
                if !wippen.links.contains(&row.pair) || row.gap > ROW_MARGIN {
                    continue;
                }
                let (r_a, r_b) = (radius(row.feature.a), radius(row.feature.b));
                let reach = distance(row.point, joint(m.key))
                    + distance(row.point, joint(wippen.body))
                    + row.gap.abs()
                    + r_a.max(r_b);
                let third = if row.feature.a.is_disc() && row.feature.b.is_disc() {
                    let f = (r_a + r_b - ROW_MARGIN).max(1e-3);
                    6.0 * reach * reach * reach / (f * f) + 6.0 * reach * reach / f + reach
                } else {
                    reach
                };
                let bound = h * h / 6.0 * third + 256.0 * eps * reach / h + 128.0 * eps * reach;
                let gap = |at: &[f64]| {
                    let (_, rows) = rows_at(&m, at);
                    rows.rows()
                        .iter()
                        .find(|r| r.pair == row.pair && r.feature == row.feature)
                        .map(|r| r.gap)
                        .unwrap_or_else(|| panic!("{name}: {row:?} vanishes within {h} of {at:?}"))
                };
                for c in 0..q.len() {
                    let (mut up, mut down) = (q.clone(), q.clone());
                    up[c] += h;
                    down[c] -= h;
                    let difference = (gap(&up) - gap(&down)) / (up[c] - down[c]);
                    let gradient = rows.grad(i)[c];
                    assert!(
                        (gradient - difference).abs() <= bound,
                        "{name}, {row:?}, coordinate {c}: the gradient is {gradient}, the \
                         central difference {difference}, off by more than {bound:e}"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 60, "only {checked} entries checked");
    }

    /// Consistency: **every contact surface of the key on its frame is a
    /// range its parts declare**: the punchings' drawn tops against the
    /// underside's flat, the cloth's drawn top and sides against the whole
    /// underside, the pin's drawn top against the mortise's and against the
    /// wood round it, the balance punching last. Nothing else is there to be
    /// read: a pair holds ranges, and the mechanism has checked each is
    /// inside its part's silhouette.
    #[test]
    fn every_contact_reads_a_range_its_parts_declare() {
        for (name, a) in assemblies() {
            let (m, key, frame) = mounted(&a);
            let pin = frame
                .front_pin_top
                .clone()
                .expect("the front rail has a pin");
            let expected = [
                (
                    ContactCode::FrontRailPunching,
                    Surface::of(&key.underside_flat),
                    Surface::one(frame.front_bearing.clone()),
                ),
                (
                    ContactCode::BackRailCloth,
                    Surface::of(&key.underside),
                    Surface::one(frame.cloth_and_sides.clone()),
                ),
                (
                    ContactCode::FrontPinInTheMortise,
                    Surface::one(key.front_mortise_top.clone()),
                    Surface::one(pin.clone()),
                ),
                (
                    ContactCode::FrontPinOnTheWood,
                    Surface::of(&key.beside_the_mortise),
                    Surface::one(pin),
                ),
                (
                    ContactCode::BalanceRailPunching,
                    Surface::of(&key.underside_flat),
                    Surface::one(frame.balance_bearing.clone()),
                ),
            ];
            let pairs = m.mechanism.pairs();
            assert_eq!(pairs.len(), expected.len(), "{name}");
            for (pair, (code, a_side, b_side)) in pairs.iter().zip(expected) {
                assert_eq!(pair.code, code, "{name}");
                assert_eq!(pair.a, a_side, "{name}: {code:?}");
                assert_eq!(pair.b, b_side, "{name}: {code:?}");
                for side in [&pair.a, &pair.b] {
                    for range in &side.ranges {
                        let drawn = m.mechanism.parts()[range.part.index()].silhouette().len();
                        assert!(usize::from(range.segs.end) <= drawn, "{name}: {code:?}");
                    }
                }
            }
            let lift = m.lift.as_ref().expect("the key is free to lift");
            assert_eq!(usize::from(lift.pair), pairs.len() - 1);
            assert!(
                lift.turning.is_some(),
                "{name}: the punching is on the axis"
            );
        }
    }

    /// Consistency, the check of `.claude/rules/physics.md` > One part definition feeds contacts and drawing by the brute-force oracle:
    /// **every contact lies on its drawn outline.** Wherever the key stands,
    /// seated at the top and at the bottom, riding on a frame whose stops
    /// fight, lifted off its punching or turned across its stroke, every row
    /// of every pair has its two surface points on the two drawn outlines,
    /// within half a spacing of a sample of each, the farthest a point of a
    /// curve can stand from the nearest sample; and a center bearing's row
    /// stands at the middle of the drawn face, within the same. The oracle
    /// shares no code with the rows: it samples what is drawn.
    #[test]
    fn every_contact_lies_on_its_drawn_outline() {
        let mut cases: Vec<(String, Mounted, Vec<f64>)> = wippen_poses();
        let mut benches = assemblies();
        benches.push(("a frame whose stops fight", wedged()));
        benches.push((
            "a key with every feature the test key lacks",
            Setup {
                key: crate::test_fixture::key_with_every_feature(),
                ..regulated()
            },
        ));
        for (name, a) in benches {
            let (m, ..) = mounted(&a);
            if let Some((top, bottom)) = seated(&a) {
                let middle: Vec<f64> = top
                    .iter()
                    .zip(&bottom)
                    .map(|(t, b)| 0.5 * (t + b))
                    .collect();
                cases.push((format!("{name}, at rest"), m.clone(), top.clone()));
                cases.push((format!("{name}, at the bottom"), m.clone(), bottom));
                cases.push((format!("{name}, mid-stroke"), m.clone(), middle));
                let mut lifted = top;
                lifted[0] += 0.0004;
                cases.push((format!("{name}, lifted"), m.clone(), lifted));
            }
            for theta in [-0.05, 0.0, 0.05] {
                cases.push((
                    format!("{name}, at {theta} rad"),
                    m.clone(),
                    on_it(&m, theta),
                ));
            }
        }
        let on = 0.5 * SPACING + 1e-12;
        let mut read = 0;
        for (name, m, q) in cases {
            let (pose, rows) = rows_at(&m, &q);
            for (p, pair) in m.mechanism.pairs().iter().enumerate() {
                let o = oracle::Pair::read(&m.mechanism, &pose, pair);
                for (_, row) in rows.of_pair(u16::try_from(p).unwrap()) {
                    let (on_a, on_b) = (row.on_a(), row.on_b());
                    assert!(
                        oracle::nearest(on_a, &o.a) <= on && oracle::nearest(on_b, &o.b) <= on,
                        "{name}, {:?}: a row's points are off the drawn outlines: {row:?}",
                        pair.code
                    );
                    if let Some(center) = o.center {
                        let (dx, dy) = (on_b.x - center.x, on_b.y - center.y);
                        assert!(
                            (dx * dx + dy * dy).sqrt() <= on,
                            "{name}, {:?}: the row bears at {on_b:?}, the face's middle is \
                             {center:?}",
                            pair.code
                        );
                    }
                    read += 1;
                }
            }
        }
        assert!(read > 100, "only {read} rows read");
    }

    /// Consistency: **the rows against the brute-force gap oracle**, the
    /// drawn outlines sampled every 0.05 mm, sharing no code with the rows.
    ///
    /// At rest, in the middle of the stroke and at the bottom of every case,
    /// where the seated pose puts the key; where a frame whose stops fight
    /// seats the key, riding on both; and with a key shortened under its
    /// frame held where the regulated key rests, its front pin standing in
    /// its wood, overlaps the static solve never leaves:
    ///
    /// - every row's two surface points stand on their drawn outlines, within
    ///   half a spacing of a sample, the farthest a point of the curve can be
    ///   from the nearest;
    /// - a pair whose rows stand apart reads the oracle's distance within
    ///   one spacing, the most the closest samples can be from the closest
    ///   points, and the oracle sees no overlap where it stands clear by more,
    ///   none deeper than the half spacing a touching surface reads at, where
    ///   another pair of the same parts touches;
    /// - a row that overlaps by more than a spacing, the least the samples
    ///   can tell, has a surface point inside the other side's drawn
    ///   material, and is at least as deep as that point stands from the
    ///   other surface, within a spacing: a row reads the distance to its own
    ///   face, which no nearer boundary of that material exceeds;
    /// - of the overlapping rows of one corner or edge, the shallowest is as
    ///   deep as the oracle reads, within a spacing, wherever it overlaps by
    ///   no more than the row margin, the depth to which a static solve ever
    ///   lets an overlap go. Deeper, inside the wood round a slot, where the
    ///   nearer face may lie outside the corner's sector, it is only at
    ///   least as deep.
    #[test]
    fn the_rows_agree_with_the_brute_force_oracle() {
        let mut cases: Vec<(String, Mounted, Vec<f64>)> = wippen_poses();
        let regulated_seats = seated(&regulated()).expect("the regulated key seats");
        for (name, a) in assemblies() {
            let m = mounted(&a).0;
            let (top, bottom) = seated(&a).unwrap_or_else(|| regulated_seats.clone());
            let middle: Vec<f64> = top
                .iter()
                .zip(&bottom)
                .map(|(t, b)| 0.5 * (t + b))
                .collect();
            for (at, q) in [
                ("at rest", top),
                ("mid-stroke", middle),
                ("at the bottom", bottom),
            ] {
                cases.push((format!("{name}, {at}"), m.clone(), q));
            }
        }
        let (riding, _) = seated(&wedged()).expect("the key rides on its stops");
        cases.push((
            "a frame whose stops fight".to_owned(),
            mounted(&wedged()).0,
            riding,
        ));
        let mut short = regulated();
        short.key.front_edge -= 0.018;
        cases.push((
            "a key shortened under its frame, where the regulated key rests".to_owned(),
            mounted(&short).0,
            regulated_seats.0.clone(),
        ));

        let on = 0.5 * SPACING + 1e-12;
        let (mut apart, mut deep) = (0, 0);
        for (name, m, q) in cases {
            let (pose, rows) = rows_at(&m, &q);
            for (p, pair) in m.mechanism.pairs().iter().enumerate() {
                let p = u16::try_from(p).unwrap();
                let o = oracle::Pair::read(&m.mechanism, &pose, pair);
                // Each overlapping corner or edge: its shallowest row, and how
                // deep the oracle reads its point.
                let mut shallowest: Vec<(Feature, f64, f64)> = Vec::new();
                for (_, row) in rows.of_pair(p) {
                    let (on_a, on_b) = (row.on_a(), row.on_b());
                    assert!(
                        oracle::nearest(on_a, &o.a) <= on && oracle::nearest(on_b, &o.b) <= on,
                        "{name}, {:?}: a row's points are off the drawn outlines: {row:?}",
                        pair.code
                    );
                    if row.gap < -SPACING {
                        let depth = if o.in_a.holds(on_b) {
                            oracle::nearest(on_b, &o.a)
                        } else if o.in_b.holds(on_a) {
                            oracle::nearest(on_a, &o.b)
                        } else {
                            panic!(
                                "{name}, {:?}: the row overlaps by {} m and neither point is \
                                 inside the other side's drawn material",
                                pair.code, row.gap
                            );
                        };
                        assert!(
                            -row.gap >= depth - SPACING,
                            "{name}, {:?}: the row is {} m deep, its point {depth} m inside",
                            pair.code,
                            -row.gap
                        );
                        // The corner or edge is the row's disc; on a face
                        // against a face's center, the center.
                        let f = row.feature;
                        let key = match (f.a.is_disc(), f.b.is_disc()) {
                            (true, false) => Feature { a: f.a, b: f.a },
                            (false, _) => Feature { a: f.b, b: f.b },
                            (true, true) => f,
                        };
                        match shallowest.iter_mut().find(|(k, ..)| *k == key) {
                            Some(entry) if -row.gap < entry.1 => {
                                entry.1 = -row.gap;
                                entry.2 = depth;
                            }
                            Some(_) => {}
                            None => shallowest.push((key, -row.gap, depth)),
                        }
                        deep += 1;
                    }
                }
                for (key, row_depth, depth) in shallowest {
                    assert!(
                        (row_depth - depth).abs() <= SPACING
                            || row_depth > crate::contact::ROW_MARGIN,
                        "{name}, {:?}: the shallowest row of {key:?} is {row_depth} m deep, the \
                         oracle {depth} m",
                        pair.code
                    );
                }
                if let Some(gap) = rows.gap(p).filter(|&g| g >= 0.0) {
                    let distance = o.apart();
                    assert!(
                        (gap - distance).abs() <= SPACING,
                        "{name}, {:?}: the rows read {gap} m, the oracle {distance} m",
                        pair.code
                    );
                    let overlap = o.overlap();
                    assert!(
                        gap <= SPACING || overlap <= 0.5 * SPACING,
                        "{name}, {:?}: {gap} m apart, and the oracle sees an overlap {overlap} m \
                         deep",
                        pair.code
                    );
                    apart += 1;
                }
            }
        }
        assert!(apart > 30, "only {apart} pairs stood apart");
        assert!(deep > 0, "no case overlapped");
    }

    /// The chamfer under the tail is there so the key beds flat on its
    /// cloth: at the one rail height that stands the key at the chamfer's own
    /// angle, both edges of the cloth close on the chamfer and the rows call
    /// it flat; half a millimeter lower the tail digs in at the cloth's back
    /// edge, half a millimeter higher it rides on the front edge.
    #[test]
    fn bedding_flat_on_the_cloth_is_both_edges_closing_on_the_chamfer() {
        let mut a = regulated();
        a.frame.back_rail = -0.200;
        let chamfer = a.key.tail_rise / a.key.tail_rise_run;
        let flat_attitude = -chamfer / (1.0 + chamfer * chamfer).sqrt();
        let (flat_from, _) = a.key.underside_flat();
        let on_the_chamfer = Point::new(
            a.frame.back_rail,
            a.key.tail_rise * (flat_from - a.frame.back_rail) / a.key.tail_rise_run,
        );
        let c = (1.0 - flat_attitude * flat_attitude).sqrt();
        let bedded_height = a.frame.balance_bearing_height() - flat_attitude * on_the_chamfer.x
            + c * on_the_chamfer.y;
        a.frame.back_rail_height = bedded_height - a.frame.back_rail_cloth;

        let read = |a: &Setup| {
            let (m, ..) = mounted(a);
            let (pose, rows) = rows_at(&m, &rest(a));
            let back = pair_of(&m, ContactCode::BackRailCloth);
            (
                m.mechanism.bearing(&pose, &rows, back).expect("a bearing"),
                cloth_edges(a),
            )
        };
        let (flat, _) = read(&a);
        assert_eq!(flat.how, HowItBears::Flat, "{flat:?}");
        assert!(flat.slope.unwrap().abs() < 1e-9, "{flat:?}");

        let mut low = a.clone();
        low.frame.back_rail_height -= 0.0005;
        let (digging, (back_edge, _)) = read(&low);
        assert_eq!(digging.how, HowItBears::StopEdge);
        assert!(digging.slope.unwrap() > 0.0, "the rear end is the low one");
        assert!(
            (digging.point.x - back_edge).abs() < 1e-3,
            "it digs in at the back edge"
        );

        let mut high = a.clone();
        high.frame.back_rail_height += 0.0005;
        let (riding, (_, front_edge)) = read(&high);
        assert_eq!(riding.how, HowItBears::StopEdge);
        assert!(riding.slope.unwrap() < 0.0);
        assert!(
            (riding.point.x - front_edge).abs() < 1e-3,
            "it rides on the front edge"
        );
    }

    /// A cloth reaching past the end of the key is carried by the key's own
    /// corner, the tail's: the governing row is that vertex on the cloth,
    /// and the slope is the chamfer's, the face of the corner that runs on
    /// over the cloth.
    #[test]
    fn a_cloth_past_the_end_of_the_key_bears_on_the_key_corner() {
        let mut a = regulated();
        a.frame.back_rail = -0.232;
        let q = rest(&a);
        let (m, key, _) = mounted(&a);
        let theta = q[m.angle.0];
        let (pose, rows) = rows_at(&m, &q);
        let back = pair_of(&m, ContactCode::BackRailCloth);
        let bearing = m.mechanism.bearing(&pose, &rows, back).expect("a bearing");
        assert_eq!(bearing.how, HowItBears::KeyCorner);
        let governing = rows.rows()[rows.governing(back).unwrap()];
        assert_eq!(
            governing.feature.a,
            Piece::Vertex {
                part: key.keystick,
                seg: 0
            },
            "the tail's corner, where the chamfer starts"
        );
        let chamfer = a.key.tail_rise / a.key.tail_rise_run;
        let expected = (chamfer - libm::sin(theta).abs() * (1.0 + chamfer * chamfer).sqrt()).abs();
        assert!(bearing.slope.is_some_and(|x| x.abs() > 0.0), "{bearing:?}");
        assert!(
            (bearing.slope.unwrap().abs() - expected).abs() < 1e-3,
            "the chamfer's slope over the cloth: {bearing:?}, expected about {expected}"
        );
    }

    /// Consistency: **a punching standing past the end of the key has no
    /// row**, the span condition of a center bearing. With the key's front a
    /// third of a millimeter behind the felt's center, the center projects
    /// past the underside both at rest and with the key level, and the pair
    /// reads nothing: no stop stands in front of the key. A third of a
    /// millimeter the other way, it reads one. The key's front is placed
    /// against the felt's drawn center, its pin's axis, which no stop of the
    /// key's own reads.
    #[test]
    fn a_punching_past_the_end_of_the_key_has_no_row() {
        let at_rest = rest(&regulated());
        for (behind, reads) in [(0.0003, false), (-0.0003, true)] {
            let mut a = regulated();
            a.key.front_edge = a
                .frame
                .sections
                .front
                .pin_x(a.frame.front_rail, KeyKind::Natural)
                - behind;
            let (m, ..) = mounted(&a);
            let front = pair_of(&m, ContactCode::FrontRailPunching);
            // Nose-up as the regulated key rests: this one, its front pin
            // now under its wood, has no seat on its punching.
            for theta in [at_rest[m.angle.0], 0.0] {
                let (_, rows) = rows_at(&m, &on_it(&m, theta));
                assert_eq!(
                    rows.gap(front).is_some(),
                    reads,
                    "the key's front {behind} m behind the punching's center, at {theta} rad"
                );
            }
        }
    }

    /// The limit `docs/FOUNDATIONS.md` > Seventeen contacts per note warns of: a corner sunk deeper
    /// than an edge arc's center has a row only while a face beside the arc
    /// lies in the corner's own angle. The key's tail corner, a quarter of a
    /// millimeter in front of where the cloth's face ends, sunk past the front
    /// edge's center with the key nose-up, faces neither the cloth's face nor
    /// its front side, and the cloth reads no row although the corner is
    /// inside the drawn cloth. The statics never put it there; a reading that
    /// sees it makes this test fail, and the warning goes with it.
    #[test]
    fn a_corner_sunk_past_an_edge_center_can_have_no_row() {
        let mut a = regulated();
        let r = a.frame.back_rail_cloth_edge_radius;
        let (_, face_front) = cloth_edges(&a);
        a.frame.back_rail += a.key.tail_end - 0.00025 - face_front;
        let (m, key, frame) = mounted(&a);
        let back = pair_of(&m, ContactCode::BackRailCloth);
        let parts = m.mechanism.parts();
        let face_front = frame
            .cloth_top
            .segs(parts)
            .iter()
            .find_map(|s| match s {
                crate::geometry::Seg::Line { a, .. } => Some(a.x),
                crate::geometry::Seg::Arc { .. } => None,
            })
            .expect("the cloth has a face");
        let corner = key.underside[0].segs(parts)[0].start();
        let top = a.frame.back_bearing_height();
        let mut sunk = 0;
        for k in 0..=60 {
            let theta = 0.03 + 0.0005 * f64::from(k);
            let pose = m.mechanism.pose(&on_it(&m, theta));
            let c = pose.placement(m.key).world(corner);
            let depth = top - c.y;
            let inside = c.x > face_front && c.x < face_front + r;
            if inside && depth > 1.5 * r && depth < a.frame.back_rail_cloth {
                let mut rows = Rows::new();
                m.mechanism.rows(&pose, f64::INFINITY, &mut rows);
                assert!(
                    rows.gap(back).is_none(),
                    "the tail corner {depth} m into the cloth now reads a row: update `docs/FOUNDATIONS.md` > Seventeen contacts per note"
                );
                sunk += 1;
            }
        }
        assert!(
            sunk > 0,
            "no attitude sank the corner past the edge's center"
        );
    }

    /// Consistency: **a front pin straddling the front edge of its slot
    /// reads the wood, at rest.** With the front rail 2 to 5 mm forward of
    /// where the bench builds it, the pin stands across the slot's front
    /// edge, and with the key at rest on its cloth, nose-up, the slot's front
    /// wall stands in the pin's top. The pin's pair on the wood has an
    /// overlapping row there, the contact the static solve and the view
    /// read: the wall's, as deep as the brute-force oracle reads the pin's
    /// top in the wood, within one spacing, the most the closest sample can
    /// be from the wall. Without the slot's walls in the wood round the
    /// mortise, nothing of the pair reads the pin at this attitude at all.
    #[test]
    fn a_pin_straddling_its_slots_edge_reads_the_wood_at_rest() {
        let at_rest = rest(&regulated());
        for forward in [0.002, 0.003, 0.004, 0.005] {
            let mut a = regulated();
            a.frame.front_rail += forward;
            let (m, _, _) = mounted(&a);
            let (pose, rows) = rows_at(&m, &at_rest);
            let p = pair_of(&m, ContactCode::FrontPinOnTheWood);
            let overlapping: Vec<f64> = rows
                .of_pair(p)
                .map(|(_, r)| r.gap)
                .filter(|&g| g < 0.0)
                .collect();
            let shallowest = overlapping
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            assert!(
                !overlapping.is_empty(),
                "{forward} m forward, the pin stands in the wood at rest with no row"
            );
            let o = oracle::Pair::read(&m.mechanism, &pose, &m.mechanism.pairs()[usize::from(p)])
                .overlap();
            assert!(
                (-shallowest - o).abs() <= SPACING,
                "{forward} m forward: the wall's row is {} m deep, the oracle reads {o} m",
                -shallowest
            );
        }
    }

    /// Consistency: **the nominal pose is level on the punching wherever the
    /// key can stand so, the nearest attitude within the row margin where it
    /// cannot, and, where the key has no attitude on its punching that
    /// clears its stops, the key lowered onto its frame from above: never a
    /// pose past a stop.**
    ///
    /// On the regulated bench nothing overlaps with the key level on its
    /// punching. A key shortened under its frame, its cloth lowered to let it
    /// stand nose-up, has its front pin in its wood with the key level and
    /// for some degrees nose-up: it starts nose-up, clear of the pin. A frame
    /// whose stops fight by 5 mm leaves no attitude on the punching within
    /// the margin, while a key turned until it has gone right through its
    /// cloth or its punching reads clear of both, standing in the rail under
    /// it: the key starts level, raised off its punching until nothing
    /// overlaps, by the margin at a time, and no lower, on its balance pin
    /// standing above its key button, though a degree nose-down clears the
    /// frame a fraction of a margin lower: the slide's range moves where the
    /// halving's steps fall, and not the frame; and so it does on a short
    /// pin, 9 mm above its punching.
    #[test]
    fn the_nominal_pose_is_level_where_it_can_be_and_never_past_a_stop() {
        let (m, _, _) = mounted(&regulated());
        assert_eq!(nominal_pose(&m), m.level());

        let mut short = regulated();
        short.key.front_edge -= 0.018;
        short.frame.back_rail_height -= 0.015;
        let (m, _, _) = mounted(&short);
        let q = nominal_pose(&m);
        assert!(q[m.angle.0] > 0.0, "nose-up, off the pin: {q:?}");
        assert_eq!(
            m.lift_at(&q).to_bits(),
            0.0_f64.to_bits(),
            "on its punching"
        );
        assert!(m.mechanism.overlap(&m.mechanism.pose(&q)) <= ROW_MARGIN);

        let mut on_a_short_pin = wedged();
        on_a_short_pin.frame.sections.balance.pin_height = crate::test_fixture::SHORT_BALANCE_PIN;
        let (m, _, _) = mounted(&on_a_short_pin);
        let q = nominal_pose(&m);
        assert_eq!(q[m.angle.0].to_bits(), 0.0_f64.to_bits(), "level: {q:?}");

        let (m, _, _) = mounted(&wedged());
        let q = nominal_pose(&m);
        assert_eq!(q[m.angle.0].to_bits(), 0.0_f64.to_bits(), "level: {q:?}");
        let lift = m.lift_at(&q);
        assert!(lift > ROW_MARGIN, "raised off its punching: {q:?}");
        let (key, rails) = key_and_rails(&m);
        assert!(overlap_at(&m, &m.mechanism, &key, &rails, &q) <= ROW_MARGIN);
        let mut lower = q.clone();
        lower[0] -= ROW_MARGIN;
        assert!(
            overlap_at(&m, &m.mechanism, &key, &rails, &lower) > ROW_MARGIN,
            "and no higher than it must"
        );
        let through = (1..=15)
            .flat_map(|k| [f64::from(k), -f64::from(k)])
            .find(|&k| {
                let pose = m.mechanism.pose(&on_it(&m, k * NOMINAL_STEP));
                m.mechanism.overlap(&pose) <= ROW_MARGIN
                    && m.mechanism.parts_overlap(&pose, &key, &rails) > ROW_MARGIN
            });
        assert!(
            through.is_some(),
            "some attitude on the punching reads clear of the stops it has gone through"
        );
    }

    /// Consistency: **the key held on its punching is the key on a pivot
    /// there, to the bit.** Its pose places every part where the pivot's
    /// does, the bits of its frame's origin and rotation included, and its
    /// rows over the angle are the pivot's rows; the slide's coordinate adds
    /// a column of its own and changes none. Across the stroke and past it.
    #[test]
    fn held_on_its_punching_the_key_is_the_key_on_a_pivot() {
        let a = regulated();
        let build = |mount| {
            on_its_frame(mount, &a.key, &a.frame, KeyKind::Natural, None, 0.0, None)
                .unwrap()
                .mounted
        };
        let (free, pinned) = (build(KeyMount::OnPunching), build(KeyMount::Pivot));
        for theta in [-0.1, -0.04, 0.0, 0.02, 0.0397, 0.1] {
            let (q_free, q_pinned) = (on_it(&free, theta), vec![theta]);
            let (pf, pp) = (
                free.mechanism.pose(&q_free),
                pinned.mechanism.pose(&q_pinned),
            );
            let (a, b) = (pf.placement(free.key), pp.placement(pinned.key));
            for (x, y) in [
                (a.origin.x, b.origin.x),
                (a.origin.y, b.origin.y),
                (a.cos, b.cos),
                (a.sin, b.sin),
            ] {
                assert_eq!(x.to_bits(), y.to_bits(), "{theta} rad");
            }
            let (mut rf, mut rp) = (Rows::new(), Rows::new());
            free.mechanism.rows(&pf, f64::INFINITY, &mut rf);
            pinned.mechanism.rows(&pp, f64::INFINITY, &mut rp);
            let lift = free.lift.as_ref().unwrap();
            let theirs: Vec<usize> = (0..rf.len())
                .filter(|&i| rf.rows()[i].pair != lift.pair)
                .collect();
            assert_eq!(theirs.len(), rp.len(), "{theta} rad");
            for (&i, j) in theirs.iter().zip(0..rp.len()) {
                let (x, y) = (rf.rows()[i], rp.rows()[j]);
                assert_eq!(x.gap.to_bits(), y.gap.to_bits(), "{theta} rad");
                assert_eq!(
                    rf.grad(i)[free.angle.0].to_bits(),
                    rp.grad(j)[pinned.angle.0].to_bits(),
                    "{theta} rad"
                );
            }
        }
    }
}
