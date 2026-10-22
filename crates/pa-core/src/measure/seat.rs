//! The seated pose: the key put on its frame by the trade's gestures, at the
//! top of its stroke and at the bottom. Key height and key dip are read from
//! it.
//!
//! # The gestures
//!
//! Friction off: a key is seated by rule, not left wherever a weight has put
//! it, so that what the sheet reads is the same whatever the user did with
//! the touch weight. In both seats a downward load at the capstan's contact
//! point stands for the action, which in the piano holds the back of the key
//! down on its cloth; for the bottom seat a press at the measuring point is
//! added to it, as a finger presses the key down. Gravity acts when the key
//! has a mass in the model; an unweighed key is seated by the two loads
//! alone. A load a procedure leaves on the key throughout, the temporary
//! weight of the key friction weight protocol, stands on it in both seats
//! ([`Seating::standing`]).
//!
//! The magnitudes follow one frozen rule. The load at the capstan seats the
//! key at the top with [`SEAT_MARGIN`] of torque over the generalized force
//! of gravity and the standing loads; the press seats it at the bottom with
//! the same margin over those and the load at the capstan together. Both are
//! taken with the key level:
//! over its stroke, gravity's generalized force on a key changes by its
//! weight times the height of its center of mass times the angle, a fraction
//! of the margin. With rigid stops the pose depends on neither magnitude,
//! only on which stops the loads put the key on
//! (`doubling_the_preparation_loads_changes_no_seat`).
//!
//! The top seat starts from the key's nominal pose
//! ([`crate::parts::mounts::nominal_pose`]): on its balance punching, level
//! wherever its drawn parts overlap its frame there by no more than the row
//! margin, or lowered onto its frame from above where no attitude on the
//! punching clears its stops. Where it finds no seat from there, an overlap
//! within the margin that no motion of the key resolves, or a solve that
//! reaches its cap with a stop standing in the key at its start, it starts
//! again from the key lowered onto its frame clear of everything
//! ([`crate::parts::mounts::lowered_onto_its_frame`]), the stalled steps
//! counted with its own. The bottom seat starts from the top seat.
//!
//! # The wippen the key carries
//!
//! A key carrying a wippen on its capstan is seated with it: the wippen
//! rests on the capstan, or on the key's top where the capstan stands out
//! from under its heel, through two links that are not stops. No seat lists
//! a link, and both stay in the vertical balance the bearing carries. Each
//! seat starts from the key alone's own seat there, seated once for both by
//! this module's rule, the wippen lowered onto the key where it stands
//! ([`Seating::start`]): the key alone reaches its seat in large steps, where
//! the key with its wippen, its link within the row margin through the whole
//! turn, would crawl a margin a step. The preparation folds the wippen's
//! weight onto the key's angle through the link it would rest on
//! ([`Mounted::wippen_follows`]), the load at the capstan now standing for
//! the hammer. A seat names the wippen's refusals: pushed past its range, or
//! out of the key's reach, at the seat that met it, with how far past that
//! limit the heel stands with the key alone there, and a wippen part
//! standing in a key part. Any other solve that leans the wippen on a limit
//! is named at the seat the sheet refuses it at
//! ([`Seating::name_the_wippens_refusal`]).
//!
//! # On its punching, or riding
//!
//! The key is free to lift off its balance punching
//! ([`crate::parts::mounts::KeyMount::OnPunching`]), so a seat is where the
//! static solve puts it on every contact it has, the punching one of them. On
//! a frame whose stops fight, the outer stops hold the key higher than its
//! punching would, and it rides on them: both seats are then one, lifted off
//! the punching ([`Seat::lift`]), and the dip reads nothing.
//!
//! # Key height and dip
//!
//! Read at the key front's top, an anchor on the keytop, the way a rule
//! standing on the keybed reads them: key height is that point's height at
//! the top seat; key dip, how far it falls from the top seat to the bottom.
//! Both are the positions the static solve puts the drawn parts in, so they
//! see whichever stop binds first: the front punching, the front pin
//! bottoming in its mortise, or the pin meeting the wood beside its slot.

use crate::contact::{ContactCode, PartCode, ROW_MARGIN};
use crate::geometry::Point;
use crate::mechanism::{MaterialPoint, PointLoad, Problem};
use crate::parts::mounts::{Motion, Mounted, WippenLimit};
use crate::parts::verdicts::TOUCHING;
use crate::solve::{Equilibrium, Friction, Holder, Refusal, SolveReport, Statics};
use crate::G;

/// The torque, newton-meters, by which each preparation load seats the key
/// past what holds it the other way.
///
/// A margin, not a magnitude anything reads: a hundredth of a newton-meter
/// is the weight of about 4 g at the measuring point, enough that the seat
/// the loads choose does not hang on rounding, and with rigid stops the pose
/// does not depend on it.
pub const SEAT_MARGIN: f64 = 0.01;

/// How deep a seat's drawn parts may stand in one another, read off their
/// outlines, meters: a micron, a hundredth of the thinnest paper. A seat the
/// static solve converges on has every contact it reads closed to a
/// picometer; drawn parts deeper in one another than this are sunk where no
/// contact reads them, past an edge's center or across a corner's sector
/// (`docs/FOUNDATIONS.md` > Seventeen contacts per note), and the seat is not one.
pub const SEAT_OVERLAP: f64 = 1e-6;

/// The key on its frame as the procedure needs it: the mounted key, and the
/// two points of it the gestures load.
#[derive(Debug, Clone, Copy)]
pub struct Seating<'a> {
    /// The key on its frame.
    pub mounted: &'a Mounted,
    /// The capstan's contact point, in the key's frame: where the load
    /// standing for the action acts.
    pub capstan: Point,
    /// The measuring point, in the key's frame: where the press acts.
    pub measuring_point: Point,
    /// The margin each load seats the key by, newton-meters:
    /// [`SEAT_MARGIN`].
    pub margin: f64,
    /// Loads that stay on the key through the whole procedure, besides its
    /// own weight: the temporary weight the key friction weight protocol
    /// zeroes the front weight with. The preparation seats the key against
    /// them as against gravity, and every solve of the procedure carries
    /// them.
    pub standing: &'a [PointLoad],
}

/// One seat.
#[derive(Debug, Clone, PartialEq)]
pub struct Seat {
    /// The key's coordinates in it.
    pub q: Vec<f64>,
    /// The stops carrying the key there, in the mechanism's order of pairs:
    /// which rows are closed and loaded, which is determinate; how the load
    /// splits between them, where several are, is not. The balance punching
    /// is the key's bearing, not one of its stops, and is not listed: it
    /// carries the key while [`Self::lift`] is nothing.
    pub held_by: Vec<ContactCode>,
    /// The vertical load the key's bearing carries there, newtons, positive
    /// pushing the key up: the balance punching stays loaded while it is.
    pub bearing_load: f64,
    /// How far above its balance punching the key stands, meters: zero on
    /// it, and on a pivot; the lift of a key riding on its stops.
    pub lift: f64,
    /// How the solve went.
    pub report: SolveReport,
}

/// The two seats, and the loads that put the key on them.
#[derive(Debug, Clone, PartialEq)]
pub struct Seats {
    /// At the top of the stroke: tail down on the cloth.
    pub top: Seat,
    /// At the bottom: front down on whichever stop binds first.
    pub bottom: Seat,
    /// The load at the capstan standing for the action, newtons.
    pub capstan_load: f64,
    /// The press at the measuring point at the bottom, newtons.
    pub press: f64,
}

/// Why the key cannot be seated.
#[derive(Debug, Clone, PartialEq)]
pub enum SeatRefusal {
    /// The mounted key cannot be solved as given.
    Mechanism(Problem),
    /// The capstan stands at or in front of the key's bearing, where a load
    /// on it cannot hold the back of the key down.
    CapstanNotBehindTheBearing,
    /// The measuring point stands at or behind the key's bearing, where a
    /// press cannot take the key down.
    MeasuringPointNotInFront,
    /// The key has no seat: these stops fight, leaving it no pose that
    /// clears them all, or the pose the solve found leaves them sunk in the
    /// key where no contact reads them. A key free to lift rides on stops
    /// that hold it higher than its balance punching, so on its punching
    /// this is a pin standing in the key's wood or a stop sunk past what a
    /// row reads; on a pivot, stops that fight.
    CannotSeat {
        /// The contacts that fight, in the mechanism's order.
        fighting: Vec<ContactCode>,
    },
    /// The key would stand off its balance pin, and off its frame: its stops
    /// hold it higher above its balance punching than where it leaves its
    /// pin ([`Mounted::leaves_its_pin`]), clear of its frame only past it, a
    /// load takes it there, or its punching stands above the pin's top.
    LiftsOffItsFrame {
        /// How far above its balance punching the key leaves its pin,
        /// meters: negative where the punching stands above the pin's top.
        leaves_its_pin: f64,
    },
    /// Nothing holds the back of the key at the top: it turns to the edge of
    /// the modeled world.
    NoStopUnderTheTail,
    /// Nothing stops the front of the key at the bottom truthfully: a front
    /// punching whose center stands past the key's underside bears on
    /// nothing, and the key turns to the edge of the modeled world, or comes
    /// down through the part of the punching drawn under it.
    NoStopBelowTheFront,
    /// The static solve reached its cap first.
    NotConverged {
        /// Steps taken.
        iterations: u32,
        /// The stationarity residual left, newtons.
        residual: f64,
    },
    /// The wippen is pushed past its range at this seat: the key lifts its
    /// heel to the edge of its modeled world, as a center pin set too low
    /// for its capstan, or a capstan set too tall, does. Also where the start
    /// of the seat finds the heel in the key even at the top of the
    /// wippen's range with the key at its own seat there
    /// ([`Seating::start`]): at the top no pose of the key on its frame
    /// stands its capstan lower, and at the bottom its stops hold it there.
    WippenPushedPastItsRange {
        /// The seat that met it.
        at: WhichSeat,
        /// How deep the heel and its cloth stand in the key's parts at the
        /// top of the wippen's range, the key alone at this seat, meters:
        /// the least distance between their drawn outlines, or where those
        /// clear the key, how deep the links' contacts stand them in it
        /// ([`Mounted::measure_the_heel_past`]); more than nothing, short of
        /// a tie within the static solve's gap tolerance; no correction of
        /// the capstan.
        heel_depth: f64,
    },
    /// The wippen does not reach the key at this seat: it hangs to the edge
    /// of its modeled world, its heel above the capstan and the key's top,
    /// as a center pin set too high does.
    WippenOutOfReach {
        /// The seat that met it.
        at: WhichSeat,
        /// How near the heel and its cloth come to the key's parts, the
        /// capstan and the key's top, degree by degree through the wippen's
        /// range, the key alone at this seat, meters: the least distance
        /// between their drawn outlines at the whole degrees the bench
        /// decides reach at ([`Mounted::measure_the_heel_past`]), nothing at
        /// the setting where the wippen comes back to reaching; no correction
        /// of the center pin.
        heel_clearance: f64,
    },
    /// A part of the wippen stands in a part of the key deeper than
    /// [`SEAT_OVERLAP`], wherever the wippen stands. Only the heel meets the
    /// key: its body, its jack or its lever in a key part is
    /// a pose no action takes, such as the body over a capstan standing
    /// beside the heel, taller than the heel hangs below the body. The heel
    /// or its cloth there, which the links read, is a contact that failed to
    /// read them.
    WippenInTheKey {
        /// The wippen's part that stands deepest in the key.
        part: PartCode,
        /// The key's part it stands in.
        key_part: PartCode,
        /// How deep, meters.
        depth: f64,
    },
}

/// Why the wippen a key carries refuses a solve of the key, named as the
/// sheet names it: a limit of its turn at a seat of the key, with how far
/// past that limit its heel stands with the key alone at that seat, or a
/// part of it standing in a part of the key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WippenRefusal {
    /// Pushed past its range at the seat `at`
    /// ([`SeatRefusal::WippenPushedPastItsRange`]).
    PushedPastItsRange {
        /// The seat.
        at: WhichSeat,
        /// How deep the heel and its cloth stand in the key's parts at the
        /// top of the wippen's range, the key alone at that seat, meters:
        /// by their drawn outlines, or by the links' contacts where those
        /// clear the key ([`Mounted::measure_the_heel_past`]).
        heel_depth: f64,
    },
    /// Out of the key's reach at the seat `at`
    /// ([`SeatRefusal::WippenOutOfReach`]).
    OutOfReach {
        /// The seat.
        at: WhichSeat,
        /// How near the heel and its cloth come to the key's parts, degree
        /// by degree through the wippen's range, the key alone at that seat,
        /// meters ([`Mounted::measure_the_heel_past`]).
        heel_clearance: f64,
    },
    /// A part of it stands in a part of the key deeper than
    /// [`SEAT_OVERLAP`]: only the heel meets the key, through the links
    /// ([`SeatRefusal::WippenInTheKey`]).
    InTheKey {
        /// The wippen's part that stands deepest in the key.
        part: PartCode,
        /// The key's part it stands in.
        key_part: PartCode,
        /// How deep, meters.
        depth: f64,
    },
}

impl WippenRefusal {
    /// The refusal at `limit` at the seat `at`, the heel standing `past` it
    /// there, meters.
    fn at_its_limit(limit: WippenLimit, at: WhichSeat, past: f64) -> Self {
        match limit {
            WippenLimit::PushedPastItsRange => Self::PushedPastItsRange {
                at,
                heel_depth: past,
            },
            WippenLimit::OutOfReach => Self::OutOfReach {
                at,
                heel_clearance: past,
            },
        }
    }
}

impl From<WippenRefusal> for SeatRefusal {
    fn from(why: WippenRefusal) -> Self {
        match why {
            WippenRefusal::PushedPastItsRange { at, heel_depth } => {
                Self::WippenPushedPastItsRange { at, heel_depth }
            }
            WippenRefusal::OutOfReach { at, heel_clearance } => {
                Self::WippenOutOfReach { at, heel_clearance }
            }
            WippenRefusal::InTheKey {
                part,
                key_part,
                depth,
            } => Self::WippenInTheKey {
                part,
                key_part,
                depth,
            },
        }
    }
}

/// What refuses a solve of a key carrying its wippen that is not a seat's,
/// as [`Seating::name_the_wippens_refusal`] names it.
#[derive(Debug, Clone, PartialEq)]
pub enum RefusedBy {
    /// The wippen, named as the sheet names it.
    Wippen(WippenRefusal),
    /// The key lifting off its frame: a solve that pushes the wippen past
    /// its range where the key alone stands it in range at both seats, by
    /// the drawn outlines and by the links' contacts alike, found the key
    /// off its balance punching, which only a pull takes it off, and
    /// a pull that lifts the key with its wippen carries the key alone up
    /// its balance pin too. A key its bushings hold off its punching, inside
    /// their friction, whose wippen an edit then brings to the top of its
    /// range, is named so as well, though it stays where it is.
    KeyLiftingOffItsFrame,
    /// The key alone has no seat to name the wippen's refusal at: its
    /// refusal is the key's own.
    KeyAlone(SeatRefusal),
}

/// What refuses a loading's or an increment's solve of a key carrying its
/// wippen, as [`Seating::name_the_started_solves_refusal`] names it.
#[derive(Debug, Clone, PartialEq)]
pub enum StartedRefusal {
    /// Refused as [`Seating::name_the_wippens_refusal`] names it.
    Refused(RefusedBy),
    /// The wippen, kept where a state left it, out of reach with its heel at
    /// nothing from the key: the solve is started again, the wippen lowered
    /// from the top of its range ([`Seating::start`], no angle kept), as the
    /// sheet starts it.
    StartAgainLoweredFromTheTop,
}

/// The wippen's refusal of a seat's solve at the seat `at`, started from
/// `the_seat`, the key alone's own seat there with the wippen's angle where
/// the start lowered it: a limit of its turn the solve's refusal leans it on
/// ([`Mounted::wippen_limit`]), with how far past it the heel stands with
/// the key there ([`Mounted::measure_the_heel_past`]); or the equilibrium
/// standing a part of it in a key part ([`name_a_wippen_part_in_the_key`]).
/// `None` for any other outcome, and on a mount that carries no wippen.
fn name_the_wippens_refusal_at(
    mounted: &Mounted,
    solved: Result<&Equilibrium, &Refusal>,
    at: WhichSeat,
    the_seat: &[f64],
) -> Option<WippenRefusal> {
    match solved {
        Err(refusal) => mounted.wippen_limit(refusal).map(|limit| {
            WippenRefusal::at_its_limit(limit, at, mounted.measure_the_heel_past(the_seat, limit))
        }),
        Ok(eq) => name_a_wippen_part_in_the_key(mounted, &eq.q),
    }
}

/// A part of the wippen `mounted` carries standing in a part of the key at
/// `q` deeper than [`SEAT_OVERLAP`]
/// ([`crate::parts::mounts::wippen_part_in_the_key`]); `None` where none
/// does, and on a mount that carries no wippen.
fn name_a_wippen_part_in_the_key(mounted: &Mounted, q: &[f64]) -> Option<WippenRefusal> {
    crate::parts::mounts::wippen_part_in_the_key(mounted, q)
        .filter(|found| found.depth > SEAT_OVERLAP)
        .map(|found| WippenRefusal::InTheKey {
            part: found.part,
            key_part: found.key_part,
            depth: found.depth,
        })
}

/// Which of the two seats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichSeat {
    /// At the top of the stroke.
    Top,
    /// At the bottom.
    Bottom,
}

/// What a solve starts for ([`Seating::start`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Starting {
    /// A seat, friction off, from the key alone's own seat there.
    Seat(WhichSeat),
    /// The loading rule putting the key on its frame, friction off, where
    /// the rule places it.
    Loading,
    /// An increment, friction on, from where the last state left the key.
    Increment,
}

/// Where a solve of the key starts ([`Seating::start`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Start {
    /// From this pose.
    From {
        /// The coordinates.
        q: Vec<f64>,
        /// How the key alone's top seat went, where a loading's start
        /// seated it there: its solves count with the solve the start
        /// stands for.
        key_alone: Option<SolveReport>,
        /// How many times the start halved the degree in which it lowered
        /// the wippen onto the key ([`crate::parts::mounts::Lowered`]):
        /// nothing where it kept the wippen where a state left it, and on a
        /// mount that carries no wippen.
        lowering_halvings: u32,
    },
    /// Not where the key stands: an increment the rule cannot start there,
    /// the wippen fitting nowhere. The key is put back by the loading rule,
    /// whose starts obey the same rule.
    ByTheLoadingRule,
}

impl Seat {
    /// Whether the key rides on its stops in this seat: lifted off its
    /// balance punching by more than [`TOUCHING`].
    pub fn rides(&self) -> bool {
        self.lift > TOUCHING
    }

    /// The height above the keybed of `point`, a point of the key in its
    /// frame, in this seat, meters.
    pub fn height(&self, mounted: &Mounted, point: Point) -> f64 {
        mounted
            .mechanism
            .pose(&self.q)
            .world(MaterialPoint {
                body: mounted.key,
                at: point,
            })
            .y
    }
}

impl Seats {
    /// The height above the keybed of `point`, a point of the key in its
    /// frame, at each seat, meters: at the top and at the bottom.
    pub fn heights(&self, mounted: &Mounted, point: Point) -> (f64, f64) {
        (
            self.top.height(mounted, point),
            self.bottom.height(mounted, point),
        )
    }
}

/// The preparation loads of the two seats, by the frozen rule, with the
/// standing loads.
struct Preparation {
    /// At the top: the load at the capstan.
    top: Vec<PointLoad>,
    /// At the bottom: the load at the capstan and the press.
    bottom: Vec<PointLoad>,
    /// The load at the capstan, newtons.
    capstan_load: f64,
    /// The press at the measuring point, newtons.
    press: f64,
}

impl Seating<'_> {
    /// Seats the key at the top and then at the bottom, as the procedure
    /// says, with `statics`.
    pub fn seats(&self, statics: &mut Statics) -> Result<Seats, SeatRefusal> {
        let m = &self.mounted.mechanism;
        let prepared = self.preparation()?;
        let (top, bottom) = match self.key_alone() {
            None => {
                let top = self.top_seat(statics, &prepared.top)?;
                let bottom = self.seat(statics, &top.q, &prepared.bottom, false)?;
                (top, bottom)
            }
            Some(alone) => {
                let alone = alone.seats(statics)?;
                (
                    self.seat_by_the_key_alone(statics, &alone.top, &prepared.top, WhichSeat::Top)?,
                    self.seat_by_the_key_alone(
                        statics,
                        &alone.bottom,
                        &prepared.bottom,
                        WhichSeat::Bottom,
                    )?,
                )
            }
        };
        // A punching read at its center bears on nothing once that center
        // stands past the key's underside; if its drawn face stands under the
        // key all the same, the key has come down through it as drawn.
        let pose = m.pose(&bottom.q);
        let mut rows = crate::contact::Rows::new();
        m.rows(&pose, f64::INFINITY, &mut rows);
        for (p, pair) in m.pairs().iter().enumerate() {
            let p = u16::try_from(p).expect("fewer than 65 536 pairs");
            if pair.geometry == crate::contact::PairGeometry::CenterBearing
                && rows.gap(p).is_none()
                && m.face_overlap(&pose, p) > crate::parts::verdicts::TOUCHING
            {
                return Err(SeatRefusal::NoStopBelowTheFront);
            }
        }
        Ok(Seats {
            top,
            bottom,
            capstan_load: prepared.capstan_load,
            press: prepared.press,
        })
    }

    /// Seats the key at the top only, as [`Self::seats`] seats it there, the
    /// same loads from the same start, so the same bits: what key height is
    /// read from, without the bottom seat a procedure reading height alone
    /// does not need.
    pub fn top(&self, statics: &mut Statics) -> Result<Seat, SeatRefusal> {
        let prepared = self.preparation()?;
        match self.key_alone() {
            None => self.top_seat(statics, &prepared.top),
            Some(alone) => {
                let alone = alone.top(statics)?;
                self.seat_by_the_key_alone(statics, &alone, &prepared.top, WhichSeat::Top)
            }
        }
    }

    /// How the wippen the key carries refuses a solve of the key that is
    /// not a seat's, `solved` as it came out: a loading, an increment, a
    /// step of a walk or of the band's sweep. `None` for any other outcome,
    /// and on a mount that carries no wippen.
    ///
    /// A solve that leans the wippen on a limit of its turn is named at the
    /// first of the key alone's seats at which the sheet's seats refuse that
    /// limit, with how far past it the heel stands with the key alone there
    /// ([`Mounted::measure_the_heel_past`]): out of the key's reach at the
    /// top seat, where every part of the key behind its balance pin stands
    /// lowest; pushed past its range at the top seat where the heel turned to
    /// the top of its range stands in the key there, else at the bottom
    /// seat, where those parts stand highest. Where neither seat stands the
    /// heel past the top of its range, a key free to lift stood off its
    /// balance punching, which only a pull takes it off: the key lifting off
    /// its frame ([`RefusedBy::KeyLiftingOffItsFrame`]). A key on a pivot
    /// cannot lift, and is named at its bottom seat there, which only a tie
    /// at the top of the wippen's range, within the static solve's gap
    /// tolerance, reaches: the heel is read there by its drawn outlines and,
    /// where those clear the key, by the links' contacts, which the solve
    /// leans on ([`Mounted::measure_the_heel_past`]). The key alone is seated by
    /// this module's rule only once the solve is refused: at its top seat for
    /// out of reach, at both seats for pushed past, and at its top seat again
    /// where it has no bottom seat. Where it has no seat its refusal is the
    /// key's own. An equilibrium standing a part of the wippen in a key part
    /// deeper than [`SEAT_OVERLAP`] is named so.
    ///
    /// Out of reach reads more than nothing, short of a tie on a key part's
    /// edge, wherever the wippen a solve started from was lowered from the
    /// top of its range, for the reason `docs/FOUNDATIONS.md` > The assembly
    /// and its place gives. A wippen kept where a state left it can read
    /// nothing here: [`Self::name_the_started_solves_refusal`] names that
    /// case for a loading or an increment.
    pub fn name_the_wippens_refusal(
        &self,
        statics: &mut Statics,
        solved: Result<&Equilibrium, &Refusal>,
    ) -> Option<RefusedBy> {
        let refusal = match solved {
            Ok(eq) => {
                return name_a_wippen_part_in_the_key(self.mounted, &eq.q).map(RefusedBy::Wippen)
            }
            Err(refusal) => refusal,
        };
        let limit = self.mounted.wippen_limit(refusal)?;
        let (wippen, alone) = (self.mounted.wippen?, self.key_alone()?);
        let measure_past = |seat: &Seat| {
            let mut q = seat.q.clone();
            q.insert(wippen.angle.0, 0.0);
            self.mounted.measure_the_heel_past(&q, limit)
        };
        let name_at = |at, past| RefusedBy::Wippen(WippenRefusal::at_its_limit(limit, at, past));
        if limit == WippenLimit::OutOfReach {
            return Some(match alone.top(statics) {
                Ok(top) => name_at(WhichSeat::Top, measure_past(&top)),
                Err(refusal) => RefusedBy::KeyAlone(refusal),
            });
        }
        let (top, bottom) = match alone.seats(statics) {
            Ok(seats) => (seats.top, Ok(seats.bottom)),
            Err(refusal) => match alone.top(statics) {
                Ok(top) => (top, Err(refusal)),
                Err(refusal) => return Some(RefusedBy::KeyAlone(refusal)),
            },
        };
        let at_the_top = measure_past(&top);
        if at_the_top > 0.0 {
            return Some(name_at(WhichSeat::Top, at_the_top));
        }
        let at_the_bottom = match bottom {
            Ok(bottom) => measure_past(&bottom),
            Err(refusal) => return Some(RefusedBy::KeyAlone(refusal)),
        };
        Some(if at_the_bottom > 0.0 || self.mounted.lift.is_none() {
            name_at(WhichSeat::Bottom, at_the_bottom)
        } else {
            RefusedBy::KeyLiftingOffItsFrame
        })
    }

    /// How the wippen the key carries refuses a loading's or an increment's
    /// solve, `solved` as it came out, started by [`Self::start`] with the
    /// wippen's angle `kept` where a state left it: as
    /// [`Self::name_the_wippens_refusal`] names it, but for a wippen kept
    /// where a state left it that the solve finds out of reach with its heel
    /// at nothing from the key. That wippen fell free below a key part an
    /// edit brought over its heel's path, where the sheet, lowering it from
    /// the top of its range, finds it on that part: it is started again so
    /// ([`StartedRefusal::StartAgainLoweredFromTheTop`]). A tie on a key
    /// part's edge reads nothing too, and is started again alike: lowered
    /// from the top, the start or the solve from there names it as the sheet
    /// does. Never started again where no angle was `kept`: a solve already
    /// started lowered from the top is named as it stands, so a caller
    /// starts a solve again at most once. `None` where nothing refuses it,
    /// and on a mount that carries no wippen.
    pub fn name_the_started_solves_refusal(
        &self,
        statics: &mut Statics,
        solved: Result<&Equilibrium, &Refusal>,
        kept: Option<f64>,
    ) -> Option<StartedRefusal> {
        Some(match self.name_the_wippens_refusal(statics, solved)? {
            RefusedBy::Wippen(WippenRefusal::OutOfReach { heel_clearance, .. })
                if kept.is_some() && heel_clearance <= 0.0 =>
            {
                StartedRefusal::StartAgainLoweredFromTheTop
            }
            refused => StartedRefusal::Refused(refused),
        })
    }

    /// The same procedure on the key without its wippen
    /// ([`Mounted::key_alone`]), where the key carries one: what each seat
    /// of a key carrying its wippen starts from.
    fn key_alone(&self) -> Option<Seating<'_>> {
        self.mounted.key_alone.as_deref().map(|alone| Seating {
            mounted: alone,
            ..*self
        })
    }

    /// The seat `at` of a key carrying its wippen, under `loads`: started
    /// from `alone`, the key alone's own seat there, the wippen lowered
    /// onto the key where it stands ([`Self::start`]), the key alone's
    /// solves and the lowering's halvings counted with the seat's, as one
    /// increment's.
    fn seat_by_the_key_alone(
        &self,
        statics: &mut Statics,
        alone: &Seat,
        loads: &[PointLoad],
        at: WhichSeat,
    ) -> Result<Seat, SeatRefusal> {
        let mut placed = alone.q.clone();
        if let Some(wippen) = &self.mounted.wippen {
            placed.insert(wippen.angle.0, 0.0);
        }
        let (from, lowering_halvings) =
            match self.start(statics, &placed, None, Starting::Seat(at))? {
                Start::From {
                    q,
                    lowering_halvings,
                    ..
                } => (q, lowering_halvings),
                // A seat's start places the wippen or refuses it; only an
                // increment is sent to the loading rule.
                Start::ByTheLoadingRule => {
                    return Err(WippenRefusal::PushedPastItsRange {
                        at,
                        heel_depth: self
                            .mounted
                            .measure_the_heel_past(&placed, WippenLimit::PushedPastItsRange),
                    }
                    .into())
                }
            };
        let mut seat = self.seat(statics, &from, loads, at == WhichSeat::Top)?;
        seat.report = alone.report.continued(seat.report);
        seat.report.lowering_halvings += lowering_halvings;
        Ok(seat)
    }

    /// The top seat under `loads`: from the key's nominal pose, and where the
    /// key is free to lift and has no seat from there, from the key lowered
    /// onto its frame from above
    /// ([`crate::parts::mounts::lowered_onto_its_frame`]), as a technician
    /// puts a key on a frame it does not go onto. Where it has no seat from
    /// above either, the first refusal stands, naming what stands in the key.
    /// No seat from the nominal pose is a stop standing in the key there, or
    /// the key carried to where it leaves its balance pin: a nominal pose
    /// lowered onto its frame can stand at the top of the slide's range,
    /// within the row margin of clearing it, at a whole degree from the
    /// attitude the key rides at below it, and the solve from there lifts
    /// the key off its pin before it turns it down onto its stops. Or it is
    /// a solve that reaches its cap of steps: a front pin a few tenths of a
    /// millimeter into the wood in front of its slot stands its rounded tip
    /// in the slot's front wall, which the key turning on its punching draws
    /// back out only through the wall's short arm about the bearing.
    ///
    /// A key free to lift that clears its frame only off its balance pin is
    /// refused as lifting off its frame, its stops holding it past where it
    /// leaves the pin: where no pose on its pin stands within the row margin
    /// of clearing its frame, so that its nominal pose is sunk in it deeper
    /// than the margin, which no row reads truthfully and no solve starts
    /// from.
    fn top_seat(&self, statics: &mut Statics, loads: &[PointLoad]) -> Result<Seat, SeatRefusal> {
        let nominal = crate::parts::mounts::nominal_pose(self.mounted);
        if let Some(leaves_its_pin) = self.mounted.leaves_its_pin() {
            if crate::parts::mounts::overlap_on_its_frame(self.mounted, &nominal) > ROW_MARGIN {
                return Err(SeatRefusal::LiftsOffItsFrame { leaves_its_pin });
            }
        }
        // A punching standing above the pin's top has the key off its pin
        // wherever it stands on it: nothing from above seats it. A solve
        // that reached its cap is tried from above only where a stop stood in
        // the key at its start.
        let from_above = |refusal: &SeatRefusal| match refusal {
            SeatRefusal::CannotSeat { .. } => true,
            SeatRefusal::NotConverged { .. } => {
                crate::parts::mounts::overlap_on_its_frame(self.mounted, &nominal) > 0.0
            }
            SeatRefusal::LiftsOffItsFrame { leaves_its_pin } => *leaves_its_pin >= 0.0,
            _ => false,
        };
        match self.seat(statics, &nominal, loads, true) {
            Err(refusal) if from_above(&refusal) => {
                let stalled = match refusal {
                    SeatRefusal::NotConverged { iterations, .. } => iterations,
                    _ => 0,
                };
                match crate::parts::mounts::lowered_onto_its_frame(self.mounted) {
                    Some(start) if start != nominal => self
                        .seat(statics, &start, loads, true)
                        .map(|mut seat| {
                            // One increment, started again: the steps it
                            // took before it stalled count with the seat's.
                            seat.report.iterations += stalled;
                            seat.report.most_iterations += stalled;
                            seat
                        })
                        .map_err(|_| refusal),
                    _ => Err(refusal),
                }
            }
            other => other,
        }
    }

    /// Where a solve of the key starts, by one rule that every start of a
    /// key carrying its wippen obeys: each seat's, the loading rule's, and
    /// each increment's. The key stands as `placed` places it: for a seat,
    /// at the key alone's own seat there, the wippen's angle appended at any
    /// value; for a loading, where the loading rule places it, nominal or
    /// lowered from above; for an increment, where the last state left it.
    /// `kept` is the wippen's angle where a state left it.
    ///
    /// **A seat** starts from the key alone's own seat ([`Mounted::key_alone`]),
    /// seated by this module's rule, the wippen lowered onto the key there by
    /// its heel and its cloth, to within the row margin of where they first
    /// touch it, or to the bottom of its range where they clear the key all
    /// the way down ([`crate::parts::mounts::wippen_lowered`]). Where they
    /// stand in the key even at the top of the wippen's range, the seat is
    /// refused: the wippen pushed past its range at that seat. At the top
    /// seat no pose of the key on its frame stands its capstan lower, the
    /// load at the capstan holding its back down on its cloth; at the bottom
    /// seat the key's stops hold it there. The key alone reaches its seat in
    /// large steps, its stops far until it meets them, where the key with its
    /// wippen, its link within the row margin through the whole turn, would
    /// crawl a margin a step from the level pose.
    ///
    /// **A loading or an increment** starts where the key stands:
    ///
    /// 1. **The wippen kept**: where a state left it, and its heel and its
    ///    cloth stand in the key no deeper than the row margin there, it
    ///    starts where it stood.
    /// 2. **The wippen lowered** onto the key where the key stands, as a
    ///    seat lowers it.
    /// 3. **Where it fits nowhere**, a loading goes by the key alone's top
    ///    seat, the wippen lowered there and refused as a seat refuses it,
    ///    the key alone's solves reported with the start; where the key alone
    ///    has no top seat, its refusal is the key's own. An increment never
    ///    starts there: it is friction on, and starting it there would move a
    ///    key that friction holds. The key is put back by the loading rule
    ///    instead ([`Start::ByTheLoadingRule`]), whose own starts obey this
    ///    rule.
    ///
    /// No solve starts where the heel or its cloth stands in the key at the
    /// top of the wippen's range: a start that deep can lose its rows and let
    /// the heel sink through. On rigid stops the key seats alone where it
    /// seats with its wippen, within the static solve's tolerances, so a
    /// decision taken at the key alone's seat differs from one taken at the
    /// seat with its wippen only where the heel clears the key at the top of
    /// the wippen's range by less than those picometers, which nothing
    /// reads. That holds because the top of the range is decided on the
    /// drawn outlines and on the links' contacts the solve reads
    /// ([`crate::parts::mounts::wippen_lowered`]): the drawn outlines alone,
    /// whose chords stand inside the heel cloth's arc by its sag, 21.8 µm on
    /// the test heel, and inside a domed capstan's by the dome's, would let
    /// a seat start where its solve then pushes the wippen past its range,
    /// over a band of settings in which the links stand the cloth in the key
    /// no deeper than those sags added together
    /// ([`Mounted::measure_the_heel_past`]). On a mount that carries no
    /// wippen, the start is `placed`.
    pub fn start(
        &self,
        statics: &mut Statics,
        placed: &[f64],
        kept: Option<f64>,
        starting: Starting,
    ) -> Result<Start, SeatRefusal> {
        use crate::parts::mounts::{heel_in_the_key, wippen_lowered};
        let Some(wippen) = &self.mounted.wippen else {
            return Ok(Start::From {
                q: placed.to_vec(),
                key_alone: None,
                lowering_halvings: 0,
            });
        };
        let at_the_seat = |q: &[f64], at: WhichSeat| {
            wippen_lowered(self.mounted, q).ok_or_else(|| {
                SeatRefusal::from(WippenRefusal::PushedPastItsRange {
                    at,
                    heel_depth: self
                        .mounted
                        .measure_the_heel_past(q, WippenLimit::PushedPastItsRange),
                })
            })
        };
        if let Starting::Seat(at) = starting {
            let lowered = at_the_seat(placed, at)?;
            return Ok(Start::From {
                q: lowered.q,
                key_alone: None,
                lowering_halvings: lowered.halvings,
            });
        }
        if let Some(angle) = kept {
            let mut q = placed.to_vec();
            q[wippen.angle.0] = angle;
            if heel_in_the_key(self.mounted, &q) <= ROW_MARGIN {
                return Ok(Start::From {
                    q,
                    key_alone: None,
                    lowering_halvings: 0,
                });
            }
        }
        if let Some(lowered) = wippen_lowered(self.mounted, placed) {
            return Ok(Start::From {
                q: lowered.q,
                key_alone: None,
                lowering_halvings: lowered.halvings,
            });
        }
        if starting == Starting::Increment {
            return Ok(Start::ByTheLoadingRule);
        }
        let alone = self
            .key_alone()
            .expect("a key carrying a wippen is mounted with its key alone")
            .top(statics)?;
        let mut q = alone.q;
        q.insert(wippen.angle.0, 0.0);
        let lowered = at_the_seat(&q, WhichSeat::Top)?;
        Ok(Start::From {
            q: lowered.q,
            key_alone: Some(alone.report),
            lowering_halvings: lowered.halvings,
        })
    }

    /// The preparation loads, by the frozen rule of the module documentation,
    /// each seat's followed by the standing loads; refused where the capstan
    /// or the measuring point stands on the wrong side of the bearing for its
    /// load to seat the key.
    fn preparation(&self) -> Result<Preparation, SeatRefusal> {
        let m = &self.mounted.mechanism;
        let n = m.coords().len();
        // The loads are sized at the level pose; a wippen that fits nowhere
        // there is the start's to refuse, at the key alone's seat.
        let level = self.mounted.level();
        let level = crate::parts::mounts::wippen_lowered(self.mounted, &level)
            .map(|lowered| lowered.q)
            .unwrap_or_else(|| {
                let mut q = level;
                if let Some(wippen) = &self.mounted.wippen {
                    q[wippen.angle.0] = m.coords()[wippen.angle.0].limits.1;
                }
                q
            });
        let pose = m.pose(&level);
        let angle = self.mounted.angle;
        let key = self.mounted.key;
        let mut forces = vec![0.0; n];
        m.generalized_forces(&pose, self.standing, &mut forces);
        let follows = |motion| self.mounted.wippen_follows(&level, motion, ROW_MARGIN);
        let rising = self.mounted.folded(&forces, follows(Motion::Up));
        let falling = self.mounted.folded(&forces, follows(Motion::Down));
        // A downward newton at a point turns the key by minus the vertical
        // speed that point has per radian.
        let per_newton = |at: Point| {
            -m.jacobian_column(&pose, MaterialPoint { body: key, at }, angle)
                .y
        };
        let (at_capstan, at_press) = (per_newton(self.capstan), per_newton(self.measuring_point));
        if at_capstan.is_nan() || at_capstan <= 0.0 {
            return Err(SeatRefusal::CapstanNotBehindTheBearing);
        }
        if at_press.is_nan() || at_press >= 0.0 {
            return Err(SeatRefusal::MeasuringPointNotInFront);
        }
        let capstan_load = (self.margin + (-rising).max(0.0)) / at_capstan;
        let press = (self.margin + (falling + capstan_load * at_capstan).max(0.0)) / -at_press;
        let down = |at: Point, newtons: f64| PointLoad {
            at: MaterialPoint { body: key, at },
            force: Point::new(0.0, -newtons),
        };
        let mut top = vec![down(self.capstan, capstan_load)];
        let mut bottom = vec![
            down(self.capstan, capstan_load),
            down(self.measuring_point, press),
        ];
        top.extend_from_slice(self.standing);
        bottom.extend_from_slice(self.standing);
        Ok(Preparation {
            top,
            bottom,
            capstan_load,
            press,
        })
    }

    /// One seat, from `from`, under `loads`, friction off. A key carrying
    /// its wippen starts with the heel clear of the key at the top of the
    /// wippen's range, the key alone at this seat ([`Self::start`]), where
    /// the key with its wippen seats too within the static solve's
    /// tolerances: its solve pushes the wippen past its range only in a tie
    /// at that limit, within its gap tolerance, the start having read the
    /// heel there by its drawn outlines and by the links' contacts the solve
    /// leans on, and is named as the sheet names it all the same.
    fn seat(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        top: bool,
    ) -> Result<Seat, SeatRefusal> {
        let m = &self.mounted.mechanism;
        let code = |pair: u16| m.pairs()[usize::from(pair)].code;
        let bearing = self.mounted.lift.as_ref().map(|lift| lift.pair);
        let at = if top {
            WhichSeat::Top
        } else {
            WhichSeat::Bottom
        };
        let eq = self
            .mounted
            .solve(statics, from, loads, Friction::Off)
            .map_err(|refusal| {
                if let Some(wippen) =
                    name_the_wippens_refusal_at(self.mounted, Err(&refusal), at, from)
                {
                    return wippen.into();
                }
                if let Some(leaves_its_pin) = self
                    .mounted
                    .leaves_its_pin()
                    .filter(|_| self.mounted.lifts_off_its_frame(&refusal))
                {
                    return SeatRefusal::LiftsOffItsFrame { leaves_its_pin };
                }
                match refusal {
                    Refusal::Mechanism(p) => SeatRefusal::Mechanism(p),
                    Refusal::NotConverged {
                        iterations,
                        residual,
                    } => SeatRefusal::NotConverged {
                        iterations,
                        residual,
                    },
                    Refusal::NoEquilibrium { on } => {
                        let mut fighting: Vec<ContactCode> = Vec::new();
                        let mut pairs: Vec<u16> = on
                            .iter()
                            .filter_map(|h| match *h {
                                Holder::Contact { pair, .. } if !code(pair).is_link() => Some(pair),
                                Holder::Contact { .. } | Holder::Limit { .. } => None,
                            })
                            .collect();
                        pairs.sort_unstable();
                        pairs.dedup();
                        for pair in pairs {
                            fighting.push(code(pair));
                        }
                        if !fighting.is_empty() {
                            SeatRefusal::CannotSeat { fighting }
                        } else if top {
                            SeatRefusal::NoStopUnderTheTail
                        } else {
                            SeatRefusal::NoStopBelowTheFront
                        }
                    }
                }
            })?;
        // Sunk where no contact reads: no seat. A wippen's part standing in
        // the key is named as such, the heel and its cloth among them, where
        // the links failed to read them.
        let pose = m.pose(&eq.q);
        let sunk: Vec<ContactCode> = (0..m.pairs().len())
            .filter(|&p| {
                !m.pairs()[p].code.is_link()
                    && m.pair_overlap(&pose, u16::try_from(p).expect("a pair")) > SEAT_OVERLAP
            })
            .map(|p| m.pairs()[p].code)
            .collect();
        if !sunk.is_empty() {
            return Err(SeatRefusal::CannotSeat { fighting: sunk });
        }
        if let Some(wippen) = name_the_wippens_refusal_at(self.mounted, Ok(&eq), at, from) {
            return Err(wippen.into());
        }
        let mut held_by: Vec<ContactCode> = Vec::new();
        let mut lift = 0.0;
        let mut pairs: Vec<u16> = Vec::new();
        // A link is no stop: it carries the wippen, and stays in the
        // vertical balance, its push on the key what the wippen weighs on it.
        for s in &eq.supports {
            if let Holder::Contact { pair, .. } = s.by {
                if s.force > 0.0 && Some(pair) != bearing {
                    lift += s.force * s.normal.y;
                    if !code(pair).is_link() {
                        pairs.push(pair);
                    }
                }
            }
        }
        pairs.sort_unstable();
        pairs.dedup();
        for pair in pairs {
            held_by.push(code(pair));
        }
        // What the bearing carries: whatever the weights and the contacts,
        // the links' pushes among them, leave of the vertical balance.
        let weight = m.bodies()[self.mounted.key.0]
            .mass
            .map_or(0.0, |mass| mass.mass * G);
        let pressed: f64 = loads.iter().map(|l| l.force.y).sum();
        let bearing_load = weight - pressed - lift;
        Ok(Seat {
            lift: self.mounted.lift_at(&eq.q),
            q: eq.q,
            held_by,
            bearing_load,
            report: eq.report,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact::oracle::polyline;
    use crate::contact::{PairGeometry, Rows, Surface};
    use crate::parts::fixtures::{on_frame, on_pivot, regulated, OnFrame, Setup};
    use crate::parts::frame::KeyKind;
    use crate::solve::statics::{tests::assert_virtual_work, GAP_TOLERANCE};

    /// The key front's height at each seat, meters.
    fn heights(a: &Setup) -> (Seats, f64, f64) {
        let k = on_frame(a, KeyKind::Natural, 0.0, true);
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("the key seats");
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        (seats, top, bottom)
    }

    /// The key's angle in a seat, radians nose-up.
    fn angle(k: &OnFrame, seat: &Seat) -> f64 {
        seat.q[k.mounted.angle.0]
    }

    /// The regulated bench on a balance pin a key can be taken off,
    /// [`crate::test_fixture::SHORT_BALANCE_PIN`] tall.
    fn short_pinned() -> Setup {
        let mut a = regulated();
        a.frame.sections.balance.pin_height = crate::test_fixture::SHORT_BALANCE_PIN;
        a
    }

    /// The regulated bench, its cloth holding the tail 5 mm above the
    /// balance bearing while the front punching stands level with it: a
    /// frame whose stops fight, as the golden sheet's fixture builds it.
    fn wedged() -> Setup {
        let mut a = regulated();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        a
    }

    /// Consistency: **key height is read from the keybed**, the datum that
    /// makes it mean anything. Raise the whole frame and the key height
    /// rises with it, one for one, and the dip, a travel and not a height,
    /// does not move. The bound is derived: each seat closes its contact
    /// within [`GAP_TOLERANCE`] either way, which the lever from the cloth's
    /// edge, 212 mm behind the pin, to the key front, 258 mm in front,
    /// carries to the key front, twice over for two benches; the heights
    /// round their last bits on numbers of about 65 mm.
    #[test]
    fn key_height_is_read_from_the_keybed() {
        let base = regulated();
        let mut raised = base.clone();
        let lift = 0.003;
        raised.frame.balance_rail_height += lift;
        raised.frame.front_rail_height += lift;
        raised.frame.back_rail_height += lift;
        let (b, b_top, b_bottom) = heights(&base);
        let (r, r_top, r_bottom) = heights(&raised);
        let bound = 4.0 * GAP_TOLERANCE * 0.258 / 0.212 + 64.0 * f64::EPSILON * 0.07;
        assert!(
            (r_top - b_top - lift).abs() <= bound,
            "lifting the frame by {lift} m raised the key by {} m",
            r_top - b_top
        );
        assert!(
            ((r_top - r_bottom) - (b_top - b_bottom)).abs() <= 2.0 * bound,
            "a travel cannot depend on how high the frame stands"
        );
        assert_eq!(b.top.held_by, vec![ContactCode::BackRailCloth]);
        assert_eq!(r.bottom.held_by, vec![ContactCode::FrontRailPunching]);
    }

    /// Consistency: **the dip's datum matters**, as the front weight's does.
    /// Key dip is read at the key front; the touchweights at the measuring
    /// point, 11 mm behind it. The same travel read there is shorter, in the
    /// ratio of the two arms, once the term both share is taken out: turning
    /// the key also moves its top by `(cos θ_top − cos θ_bottom)` times its
    /// height above the pin, the same at both points, which both carry and
    /// neither scales. Taken out, the ratio is the arms', to the rounding
    /// of heights of about 65 mm.
    #[test]
    fn the_dip_read_at_the_measuring_point_is_shorter_by_the_arms() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let seats = k.seating().seats(&mut Statics::new()).unwrap();
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        let (mp_top, mp_bottom) = seats.heights(&k.mounted, k.measuring_point);
        let (at_front, at_point) = (top - bottom, mp_top - mp_bottom);
        assert!(at_point < at_front, "the nearer datum travels less");
        assert_eq!(k.front_top.y.to_bits(), k.measuring_point.y.to_bits());
        let (t, b) = (angle(&k, &seats.top), angle(&k, &seats.bottom));
        let common = (libm::cos(t) - libm::cos(b)) * k.front_top.y;
        let arms = k.measuring_point.x / k.front_top.x;
        let rounding = 64.0 * f64::EPSILON * top / (at_front - common);
        assert!(
            ((at_point - common) / (at_front - common) - arms).abs() <= rounding,
            "{} against the arms' {arms}",
            (at_point - common) / (at_front - common)
        );
        // 11 mm in 258 is 4 %, 0.43 mm on a 10.2 mm dip: five of the
        // thinnest papers as the key front sees them.
        assert!(at_front - at_point > 0.0004);
    }

    /// The balance punching stays loaded in both seats: the load at the
    /// capstan keeps the bearing carrying the key, its weight and the press,
    /// less what the cloth or the front stop takes; and the key stands on
    /// it, lifted off it by nothing at all.
    #[test]
    fn the_balance_punching_stays_loaded_in_both_seats() {
        let (seats, ..) = heights(&regulated());
        for seat in [&seats.top, &seats.bottom] {
            assert!(seat.bearing_load > 0.0, "{seat:?}");
            assert_eq!(seat.lift.to_bits(), 0.0_f64.to_bits(), "{seat:?}");
            assert!(!seat.rides());
        }
        assert!(seats.capstan_load > 0.0 && seats.press > 0.0);
    }

    /// C12, consistency: **the preparation loads choose the seat, not the
    /// pose.** Doubled, they put the key on the same stops at the same
    /// attitudes, within what the solve converges to.
    #[test]
    fn doubling_the_preparation_loads_changes_no_seat() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let once = k.seating().seats(&mut Statics::new()).unwrap();
        let twice = Seating {
            margin: 2.0 * SEAT_MARGIN,
            ..k.seating()
        }
        .seats(&mut Statics::new())
        .unwrap();
        let bound = 2.0 * GAP_TOLERANCE / 0.2;
        for (one, two) in [(&once.top, &twice.top), (&once.bottom, &twice.bottom)] {
            assert_eq!(one.held_by, two.held_by);
            assert!(
                (angle(&k, one) - angle(&k, two)).abs() <= bound,
                "{one:?} against {two:?}"
            );
        }
        assert!(twice.capstan_load > once.capstan_load);
    }

    /// C12, consistency: **the metric shapes the path, not the seat.** Its
    /// scale multiplied by ten, the seats are the same, within what the
    /// solve converges to.
    #[test]
    fn scaling_the_metric_changes_no_seat() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let one = k.seating().seats(&mut Statics::new()).unwrap();
        let ten = k
            .seating()
            .seats(&mut Statics::with_metric_scaled(10.0))
            .unwrap();
        let bound = 2.0 * GAP_TOLERANCE / 0.2;
        for (x, y) in [(&one.top, &ten.top), (&one.bottom, &ten.bottom)] {
            assert_eq!(x.held_by, y.held_by);
            assert!(
                (angle(&k, x) - angle(&k, y)).abs() <= bound,
                "{x:?} against {y:?}"
            );
        }
    }

    /// An unweighed key is seated by the preparation loads alone, on the
    /// same stops as the weighed one: with rigid stops, its key height and
    /// dip do not depend on its mass.
    #[test]
    fn an_unweighed_key_is_seated_by_its_preparation_loads() {
        let a = regulated();
        let weighed = on_frame(&a, KeyKind::Natural, 0.0, true);
        let bare = on_frame(&a, KeyKind::Natural, 0.0, false);
        let w = weighed.seating().seats(&mut Statics::new()).unwrap();
        let b = bare.seating().seats(&mut Statics::new()).unwrap();
        assert_eq!(w.top.held_by, b.top.held_by);
        assert_eq!(w.bottom.held_by, b.bottom.held_by);
        let bound = 2.0 * GAP_TOLERANCE / 0.2;
        assert!((angle(&weighed, &w.top) - angle(&bare, &b.top)).abs() <= bound);
        assert!((angle(&weighed, &w.bottom) - angle(&bare, &b.bottom)).abs() <= bound);
    }

    /// C6, consistency: each seat is an equilibrium of its loads, by the
    /// unilateral virtual-work inequality, over both of the key's
    /// coordinates: no displacement that keeps every gap open, the balance
    /// punching's included, releases energy.
    #[test]
    fn each_seat_is_an_equilibrium_of_its_loads() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let seats = k.seating().seats(&mut Statics::new()).unwrap();
        let key = k.mounted.key;
        let down = |at: Point, f: f64| PointLoad {
            at: MaterialPoint { body: key, at },
            force: Point::new(0.0, -f),
        };
        let top = [down(k.capstan, seats.capstan_load)];
        let bottom = [
            down(k.capstan, seats.capstan_load),
            down(k.measuring_point, seats.press),
        ];
        for (seat, loads) in [(&seats.top, &top[..]), (&seats.bottom, &bottom[..])] {
            let eq = k
                .mounted
                .solve(&mut Statics::new(), &seat.q, loads, Friction::Off)
                .expect("a seat is an equilibrium");
            for (x, y) in eq.q.iter().zip(&seat.q) {
                assert_eq!(x.to_bits(), y.to_bits(), "solved again, it stays");
            }
            assert_virtual_work(
                &k.mounted.mechanism,
                &eq,
                loads,
                Friction::Off,
                &eq.q.clone(),
                &[0, 1],
                1e-6,
                0x5ea7,
            );
        }
    }

    /// The front pin bottoming in its mortise stops the key before its
    /// punching: one pin a millimeter prouder, and it is the pin that holds
    /// the bottom seat, on its rounded tip, the dip shorter by the
    /// millimeter seen through the lever.
    #[test]
    fn a_pin_too_proud_holds_the_bottom_seat() {
        let base = regulated();
        let mut proud = base.clone();
        proud.frame.sections.front.pin_height += 0.001;
        let (_, top, bottom) = heights(&base);
        let (seats, p_top, p_bottom) = heights(&proud);
        assert_eq!(
            seats.bottom.held_by,
            vec![ContactCode::FrontPinInTheMortise]
        );
        assert!((p_top - top).abs() < 1e-12, "the rest does not move");
        assert!(p_top - p_bottom < top - bottom, "the dip is shorter");
    }

    /// A pin standing under the key's wood, outside its mortise, meets the
    /// wood beside the slot. On a frame whose cloth lets the key stand
    /// nose-up enough to clear it, the key rests on its cloth and its
    /// punching, and the pin, not the punching, holds the bottom of a dip it
    /// cuts short. On the frame as built, whose cloth does not, the key rides
    /// on the pin and its cloth, lifted off its balance punching, and has no
    /// dip at all; on a pivot it would have no seat.
    #[test]
    fn a_pin_under_the_wood_beside_its_mortise_holds_the_key() {
        let mut short = regulated();
        short.key.front_edge -= 0.018;
        let (seats, top, bottom) = heights(&short);
        for seat in [&seats.top, &seats.bottom] {
            assert!(seat.rides(), "{seat:?}");
            assert!(seat.held_by.contains(&ContactCode::FrontPinOnTheWood));
            assert!(seat.held_by.contains(&ContactCode::BackRailCloth));
        }
        assert_eq!((top - bottom).to_bits(), 0.0_f64.to_bits(), "no dip");
        let k = on_pivot(&short, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                assert!(
                    fighting.contains(&ContactCode::FrontPinOnTheWood),
                    "{fighting:?}"
                );
            }
            other => panic!("{other:?}"),
        }

        short.frame.back_rail_height -= 0.015;
        let (seats, top, bottom) = heights(&short);
        assert_eq!(seats.top.held_by, vec![ContactCode::BackRailCloth]);
        assert_eq!(seats.bottom.held_by, vec![ContactCode::FrontPinOnTheWood]);
        assert!(!seats.top.rides() && !seats.bottom.rides());
        assert!(
            top - bottom > 0.001 && top - bottom < 0.010,
            "a dip cut short: {}",
            top - bottom
        );
    }

    /// The audit's RUST-2, consistency: **a frame whose stops fight seats
    /// the key riding on them, on the parts as drawn.** Its cloth holding
    /// the tail 5 mm above the balance bearing while the front punching
    /// stands level with it, the key rides on the cloth and on the front
    /// punching, lifted off its balance punching, at the top seat and at the
    /// bottom one, which is the same pose to the bit: pressed in front of
    /// the front punching, the key only leans harder on its stops, and has no
    /// dip.
    ///
    /// Read off the drawn outlines by the brute-force oracle, sharing no code
    /// with the rows: the drawn underside, chamfer included, stands on the
    /// drawn cloth and on the front punching's drawn center, within the
    /// oracle's resolution, the sagitta of one spacing on the cloth's edge,
    /// where a straight underside laid on the two stops would stand 0.81 mm
    /// clear of the cloth, under the chamfer. Its balance punching's drawn center
    /// stands below the drawn underside by the lift, the slide's height off
    /// it, within the same. On a pivot the key has no seat on this frame,
    /// the stops named.
    #[test]
    fn a_frame_whose_stops_fight_seats_the_key_riding_on_them() {
        let a = wedged();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("the key rides");
        for seat in [&seats.top, &seats.bottom] {
            assert!(seat.rides(), "{seat:?}");
            assert_eq!(
                seat.held_by,
                vec![ContactCode::FrontRailPunching, ContactCode::BackRailCloth]
            );
        }
        for (x, y) in seats.top.q.iter().zip(&seats.bottom.q) {
            assert_eq!(x.to_bits(), y.to_bits(), "one pose for both seats");
        }
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        assert_eq!((top - bottom).to_bits(), 0.0_f64.to_bits());
        let lift = seats.top.lift;
        assert!(lift > 0.001 && lift < 0.005, "lifted {lift} m");

        let m = &k.mounted.mechanism;
        let pose = m.pose(&seats.top.q);
        let resolution = polyline_resolution();
        for (code, apart) in [
            (ContactCode::BackRailCloth, 0.0),
            (ContactCode::FrontRailPunching, 0.0),
            (ContactCode::BalanceRailPunching, lift),
        ] {
            let pair = m.pairs().iter().find(|p| p.code == code).unwrap();
            let read = drawn_apart(m, &pose, pair);
            assert!(
                (read - apart).abs() <= resolution,
                "{code:?}: the drawn parts stand {read} m apart, not {apart} m"
            );
        }
        let mut rows = Rows::new();
        m.rows(&pose, f64::INFINITY, &mut rows);
        assert!(m.overlap(&pose) <= SEAT_OVERLAP, "nothing sunk");

        let k = on_pivot(&a, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                for code in [ContactCode::BackRailCloth, ContactCode::FrontRailPunching] {
                    assert!(fighting.contains(&code), "{fighting:?}");
                }
            }
            other => panic!("{other:?}"),
        }
    }

    /// How far apart the brute-force oracle reads the two sides of `pair`
    /// at `pose`, meters: the closest two points of their drawn polylines;
    /// for a center bearing, the middle of side B's drawn face from side A.
    fn drawn_apart(
        m: &crate::mechanism::Mechanism,
        pose: &crate::mechanism::Pose,
        pair: &crate::contact::ContactPair,
    ) -> f64 {
        let parts = m.parts();
        let body = |s: &Surface| parts[s.ranges[0].part.index()].body;
        let a = polyline::surface(parts, &pair.a, &pose.placement(body(&pair.a)));
        let b = polyline::surface(parts, &pair.b, &pose.placement(body(&pair.b)));
        if pair.geometry == PairGeometry::CenterBearing {
            let first = b[0][0];
            let last = *b[b.len() - 1].last().unwrap();
            let middle = Point::new(0.5 * (first.x + last.x), 0.5 * (first.y + last.y));
            return a
                .iter()
                .map(|line| polyline::to_polyline(middle, line))
                .fold(f64::INFINITY, f64::min);
        }
        let mut best = f64::INFINITY;
        for la in &a {
            for lb in &b {
                best = best.min(polyline::apart(la, lb));
            }
        }
        best
    }

    /// What the oracle's polylines can tell apart, meters: the sagitta of a
    /// chord of one spacing on the cloth's half-millimeter edges, and the gap
    /// a static solve converges to.
    fn polyline_resolution() -> f64 {
        let spacing = crate::contact::oracle::SPACING;
        spacing * spacing / (8.0 * crate::test_fixture::frame().back_rail_cloth_edge_radius)
            + 2.0 * GAP_TOLERANCE
    }

    /// Consistency: **a key its stops would hold past its balance pin has no
    /// seat: it is off its frame.** The key leaves its pin where its balance
    /// hole, at the key frame's origin on the pin's axis, clears the pin's
    /// drawn top: on the regulated bench on a short pin 9 mm above its
    /// punching, the pin's height above its rail less the punching stack, as
    /// the frame's fields
    /// give it, and a millimeter more pin is a millimeter more. The balance
    /// rail lowered takes its pin and its punching down together, and the
    /// key, held on its front punching and its cloth, rides higher above
    /// them the further the rail goes: lowered 10 mm it rides within its
    /// pin, seated where it rides on a pin 0.2 m taller; lowered 15 mm it
    /// would ride past the pin's top, as it does on the taller pin, and is
    /// refused as off its frame, the refusal naming where it leaves its pin.
    /// A stack 0.3 mm taller than the pin stands its punching above the
    /// pin's top: a key on it is off its pin wherever it stands, refused
    /// with the 0.3 mm.
    ///
    /// The bounds are derived: heights of a few centimeters, rounded at their
    /// last bits; a riding seat's two contacts each closed within
    /// [`GAP_TOLERANCE`], which the arms of the front stop and the cloth, in
    /// a ratio under two, carry to the slide.
    #[test]
    fn a_key_its_stops_hold_past_its_pin_is_off_its_frame() {
        let a = short_pinned();
        let room =
            |a: &Setup| a.frame.sections.balance.pin_height - a.frame.balance_punching.height();
        let leaves = |a: &Setup| {
            on_frame(a, KeyKind::Natural, 0.0, true)
                .mounted
                .leaves_its_pin()
                .expect("free to lift")
        };
        let rounding = 8.0 * f64::EPSILON * 0.05;
        assert!(
            (leaves(&a) - room(&a)).abs() <= rounding,
            "the key leaves its pin {} m up, the frame's fields give {} m",
            leaves(&a),
            room(&a)
        );
        assert!((room(&a) - 0.009).abs() < 1e-9, "{}", room(&a));
        let mut taller = a.clone();
        taller.frame.sections.balance.pin_height += 0.001;
        assert!((leaves(&taller) - leaves(&a) - 0.001).abs() <= rounding);

        for (lowered, within) in [(0.010, true), (0.015, false)] {
            let mut b = a.clone();
            b.frame.balance_rail_height -= lowered;
            let mut tall = b.clone();
            tall.frame.sections.balance.pin_height += 0.2;
            let (on_tall, ..) = heights(&tall);
            assert!(on_tall.top.rides(), "{lowered} m: {:?}", on_tall.top);
            let bound = leaves(&b);
            assert!((bound - room(&b)).abs() <= rounding, "{lowered} m");
            let k = on_frame(&b, KeyKind::Natural, 0.0, true);
            match k.seating().seats(&mut Statics::new()) {
                Ok(seats) => {
                    assert!(
                        within,
                        "{lowered} m: seated {} m up, past {bound} m",
                        seats.top.lift
                    );
                    assert!(seats.top.rides() && seats.top.lift < bound);
                    assert!(
                        (seats.top.lift - on_tall.top.lift).abs() <= 4.0 * GAP_TOLERANCE,
                        "{lowered} m: rides {} m up, {} m on the taller pin",
                        seats.top.lift,
                        on_tall.top.lift
                    );
                }
                Err(SeatRefusal::LiftsOffItsFrame { leaves_its_pin }) => {
                    assert!(
                        !within,
                        "{lowered} m: refused, riding {} m up",
                        on_tall.top.lift
                    );
                    assert_eq!(leaves_its_pin.to_bits(), bound.to_bits());
                    assert!(on_tall.top.lift > bound, "{lowered} m");
                }
                other => panic!("{lowered} m: {other:?}"),
            }
        }

        let mut over = a.clone();
        over.frame.balance_punching.papers.push(room(&a) + 0.0003);
        let k = on_frame(&over, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::LiftsOffItsFrame { leaves_its_pin }) => {
                assert!(
                    (leaves_its_pin + 0.0003).abs() <= rounding,
                    "the punching stands {} m above the pin's top",
                    -leaves_its_pin
                );
            }
            other => panic!("a punching above its pin: {other:?}"),
        }
    }

    /// Consistency: **a key riding just under its balance pin's top is
    /// seated where it rides.** Three frames whose stops fight: the cloth
    /// 5 mm above the balance bearing, the key riding 2.27 mm up on its
    /// front punching and its cloth, 0.58° nose-down; the key 17 mm short
    /// and its cloth 3 mm lower, riding 4.45 mm up on its front pin standing
    /// in its wood and its cloth, 4.28° nose-up; each with its balance pin cut
    /// to 0.3 mm above where the key rides on a pin 0.2 m taller. And the
    /// front punching's felt 25 mm thicker, the key riding 7.79 mm up on it
    /// and its cloth, 4.37° nose-up, 1.2 mm under the top of a short pin
    /// 9 mm above its punching. Each whole degree either side of where a key rides costs the
    /// angle between them times its stops' arms of lift, more than the pin
    /// leaves, so no attitude a whole degree apart clears the frame below the
    /// pin's top: the first key would be lowered nowhere, the second lowered
    /// at the top of the slide's range and carried off its pin from there,
    /// the third, sunk through its felt where it starts on its punching,
    /// lowered nowhere clear of everything. With the attitude narrowed
    /// between two whole degrees, each is seated riding as on the taller pin,
    /// on the same stops, at both seats.
    ///
    /// The bound is derived: a riding seat's two contacts each closed within
    /// [`GAP_TOLERANCE`], which the arms of the front stop and the cloth, in
    /// a ratio under two, carry to the slide.
    #[test]
    fn a_key_riding_just_under_its_pins_top_is_seated_where_it_rides() {
        let a = short_pinned();
        let room =
            |a: &Setup| a.frame.sections.balance.pin_height - a.frame.balance_punching.height();
        let mut wedged = a.clone();
        let f = &mut wedged.frame;
        f.back_rail_height = f.balance_bearing_height() + 0.005 - f.back_rail_cloth;
        let mut short = a.clone();
        short.key.front_edge -= 0.017;
        short.frame.back_rail_height -= 0.003;
        let mut felt = a.clone();
        felt.frame.front_punching.felt += 0.025;
        for (b, cut) in [(wedged, true), (short, true), (felt, false)] {
            let mut tall = b.clone();
            tall.frame.sections.balance.pin_height += 0.2;
            let (on_tall, ..) = heights(&tall);
            assert!(on_tall.top.rides(), "{:?}", on_tall.top);
            let mut b = b;
            if cut {
                b.frame.sections.balance.pin_height += on_tall.top.lift + 0.0003 - room(&b);
            }
            let k = on_frame(&b, KeyKind::Natural, 0.0, true);
            let bound = k.mounted.leaves_its_pin().expect("free to lift");
            assert!(
                on_tall.top.lift < bound && on_tall.top.lift > bound - 0.0013,
                "rides {} m up, leaves its pin {bound} m up",
                on_tall.top.lift
            );
            let seats = k
                .seating()
                .seats(&mut Statics::new())
                .unwrap_or_else(|r| panic!("rides {} m up: {r:?}", on_tall.top.lift));
            for (seat, on) in [(&seats.top, &on_tall.top), (&seats.bottom, &on_tall.bottom)] {
                assert!(seat.rides(), "{seat:?}");
                assert!(
                    (seat.lift - on.lift).abs() <= 4.0 * GAP_TOLERANCE,
                    "rides {} m up, {} m on the taller pin",
                    seat.lift,
                    on.lift
                );
                assert_eq!(seat.held_by, on.held_by);
            }
        }
    }

    /// Consistency: **the balance punching bears the key; it is never one of
    /// its stops.** A frame whose cloth stands the key at rest 2.6° nose-up,
    /// and whose front punching, 9.2 mm higher, stops it 2.35° nose-up,
    /// leaves the key a stroke of a quarter of a degree between two whole
    /// degrees: no attitude on its punching a degree apart stands within the
    /// row margin of clearing its stops, so the key is lowered onto its
    /// frame from above and its top seat solved free to lift. It lands on
    /// its punching and its cloth, the punching carrying it, and the seat
    /// names its cloth alone, as the seat the key on a pivot is given does;
    /// the bottom seat names its front punching; and both walks read, each
    /// ending on the other seat's stops. Listed among the top seat's stops,
    /// the punching, closed at the bottom seat too, would end the up walk
    /// before it starts.
    #[test]
    fn a_key_lowered_onto_its_punching_names_its_stops_and_not_its_bearing() {
        use crate::measure::walk::Stroke;
        use crate::mechanism::AxisFriction;
        let mut a = regulated();
        a.frame.back_rail_height -= 0.0012;
        a.frame.front_punching.papers.push(0.0092);
        let bound = AxisFriction::of_weight_at(6.0, a.measuring_point()).bound_n_m;
        let k = on_frame(&a, KeyKind::Natural, bound, true);
        let nominal = crate::parts::mounts::nominal_pose(&k.mounted);
        assert!(
            k.mounted.lift_at(&nominal) > 0.0,
            "lowered onto its frame from above: {nominal:?}"
        );
        let mut statics = Statics::new();
        let seats = k.seating().seats(&mut statics).expect("the key seats");
        for seat in [&seats.top, &seats.bottom] {
            assert!(!seat.rides() && seat.bearing_load > 0.0, "{seat:?}");
        }
        assert_eq!(seats.top.held_by, vec![ContactCode::BackRailCloth]);
        assert_eq!(seats.bottom.held_by, vec![ContactCode::FrontRailPunching]);
        let pivot = on_pivot(&a, KeyKind::Natural, bound, true)
            .seating()
            .seats(&mut statics)
            .expect("the key on a pivot seats");
        assert_eq!(seats.top.held_by, pivot.top.held_by);
        for stroke in [Stroke::Down, Stroke::Up] {
            let walk = k.seating().walk(&mut statics, &seats, stroke);
            assert!(walk.is_ok(), "{stroke:?}: {walk:?}");
        }
    }

    /// A pin straddling its slot's edge, the frame's front rail 3 mm forward
    /// of the mortise, meets the wood at the slot's mouth: the key cannot
    /// come down over it onto its cloth. On its punching it rides on the pin
    /// and the cloth; on a pivot it has no seat, the pin's pair named.
    #[test]
    fn a_pin_straddling_its_slots_edge_meets_the_wood() {
        let mut a = regulated();
        a.frame.front_rail += 0.003;
        let (seats, ..) = heights(&a);
        assert!(seats.top.rides(), "{:?}", seats.top);
        assert!(seats.top.held_by.contains(&ContactCode::FrontPinOnTheWood));
        let k = on_pivot(&a, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                assert!(
                    fighting.contains(&ContactCode::FrontPinOnTheWood),
                    "{fighting:?}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// Consistency: **a pin a few tenths of a millimeter into the wood
    /// beside its slot, its tip well up in the slot, has the key lowered
    /// onto it from above.** Its front end 0.3 mm into the wood in front of
    /// the slot, the key level on its punching overlaps the pin by less than
    /// the row margin, so that is where the nominal pose puts it, the pin's
    /// rounded tip standing in the slot's front wall, and from there the
    /// stops leave the key no motion. Lowered onto its frame from above, as
    /// a technician puts it there, the key lands on the pin's tip and rides
    /// on it and its cloth, off its balance punching, nothing sunk, the
    /// drawn wood on the drawn pin within the oracle's resolution. On a
    /// pivot, which cannot lift, it has no seat, the pin's pair named.
    #[test]
    fn a_pin_a_few_tenths_into_the_wood_has_the_key_lowered_onto_it() {
        let mut a = regulated();
        let pin = a.frame.sections.front.pin.expect("a pin").length();
        let (_, front_end) = a.key.front_mortise_span();
        a.frame.front_rail = front_end - 0.5 * pin + 0.0003;
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let m = &k.mounted.mechanism;
        let nominal = crate::parts::mounts::nominal_pose(&k.mounted);
        let overlap = m.overlap(&m.pose(&nominal));
        assert!(
            overlap > 0.0002 && overlap <= crate::contact::ROW_MARGIN,
            "{overlap} m"
        );
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("lowered onto the pin, the key rides on it");
        for seat in [&seats.top, &seats.bottom] {
            assert!(seat.rides(), "{seat:?}");
            assert_eq!(
                seat.held_by,
                vec![ContactCode::BackRailCloth, ContactCode::FrontPinOnTheWood]
            );
            let pose = m.pose(&seat.q);
            assert!(m.overlap(&pose) <= SEAT_OVERLAP, "nothing sunk");
            let pair = m
                .pairs()
                .iter()
                .find(|p| p.code == ContactCode::FrontPinOnTheWood)
                .unwrap();
            let read = drawn_apart(m, &pose, pair);
            assert!(
                read.abs() <= polyline_resolution(),
                "the drawn wood stands {read} m from the drawn pin"
            );
        }

        let k = on_pivot(&a, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                assert!(
                    fighting.contains(&ContactCode::FrontPinOnTheWood),
                    "{fighting:?}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// Consistency: **a seat whose solve stalls on a stop standing in the
    /// key is started again from above, the stalled steps counted with its
    /// own.** The pin 0.3 mm into the wood in front of its slot, as above,
    /// and a millimeter more paper under the balance punching: the key level
    /// on its punching stands the pin's rounded tip in the slot's front
    /// wall, which the key turning on its punching draws back only through
    /// the wall's short arm about the bearing, and the solve from there
    /// reaches its cap. Lowered onto its frame from above, the key rides on
    /// the pin and its cloth.
    #[test]
    fn a_seat_that_stalls_on_a_stop_in_the_key_is_started_again_from_above() {
        let mut a = regulated();
        let pin = a.frame.sections.front.pin.expect("a pin").length();
        let (_, front_end) = a.key.front_mortise_span();
        a.frame.front_rail = front_end - 0.5 * pin + 0.0003;
        a.frame.balance_punching.papers.push(0.001);
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("lowered onto the pin, the key rides on it");
        assert!(seats.top.rides(), "{:?}", seats.top);
        assert_eq!(
            seats.top.held_by,
            vec![ContactCode::BackRailCloth, ContactCode::FrontPinOnTheWood]
        );
        let capstan = PointLoad {
            at: MaterialPoint {
                body: k.mounted.key,
                at: k.capstan,
            },
            force: Point::new(0.0, -seats.capstan_load),
        };
        let nominal = crate::parts::mounts::nominal_pose(&k.mounted);
        let from_nominal =
            k.mounted
                .solve(&mut Statics::new(), &nominal, &[capstan], Friction::Off);
        let Err(Refusal::NotConverged {
            iterations: stalled,
            ..
        }) = from_nominal
        else {
            panic!("{from_nominal:?}");
        };
        assert!(
            seats.top.report.iterations > stalled,
            "{stalled} steps stalled, the seat reports {:?}",
            seats.top.report
        );
    }

    /// A key on the sharp row stands on the sharp row's punchings and pins,
    /// where its frame draws them: it turns about its own balance pin, the
    /// balance row's offset behind the natural's, and its bottom seat's
    /// contact is at the drawn front punching's center, the front row's
    /// offset behind the natural's. The sharp's mortise is cut over its row's
    /// front pin: the difference of the two offsets further back along the
    /// key.
    #[test]
    fn a_sharp_bottoms_on_the_punching_drawn_on_its_row() {
        for kind in [KeyKind::Natural, KeyKind::Sharp] {
            let mut a = regulated();
            if kind == KeyKind::Sharp {
                let rows = &a.frame.sections;
                a.key.front_mortise.from_front_edge +=
                    rows.front.sharp_row_offset - rows.balance.sharp_row_offset;
            }
            let k = on_frame(&a, kind, 0.0, true);
            let seats = k
                .seating()
                .seats(&mut Statics::new())
                .unwrap_or_else(|r| panic!("{kind:?}: {r:?}"));
            assert_eq!(seats.bottom.held_by, vec![ContactCode::FrontRailPunching]);
            let m = &k.mounted.mechanism;
            let pose = m.pose(&seats.bottom.q);
            let mut rows = crate::contact::Rows::new();
            m.rows(&pose, crate::contact::ROW_MARGIN, &mut rows);
            let front = m
                .pairs()
                .iter()
                .position(|p| p.code == ContactCode::FrontRailPunching)
                .unwrap();
            let center = m.parts()[k.frame.front_rail_punching.index()].sweep;
            let crate::contact::Sweep::Revolved { axis_x } = center else {
                panic!("a punching is turned")
            };
            let expected = a.frame.sections.front.pin_x(a.frame.front_rail, kind);
            assert_eq!(
                axis_x.to_bits(),
                expected.to_bits(),
                "{kind:?}: drawn on its row"
            );
            let i = rows
                .governing(u16::try_from(front).unwrap())
                .expect("the punching bears");
            assert!(
                (rows.rows()[i].point.x - expected).abs() < 1e-9,
                "{kind:?}: the punching bears at {} m, drawn at {expected} m",
                rows.rows()[i].point.x
            );
            let pin = a.frame.sections.balance.pin_x(0.0, kind);
            let origin = pose.placement(k.mounted.key).origin;
            assert_eq!(
                origin.x.to_bits(),
                pin.to_bits(),
                "{kind:?}: on its own pin"
            );
            assert!(!seats.top.rides() && !seats.bottom.rides(), "{kind:?}");
        }
    }

    /// Determinism: the same seats twice are the same bits.
    #[test]
    fn the_same_seats_twice_are_the_same_bits() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let mut statics = Statics::new();
        let one = k.seating().seats(&mut statics).unwrap();
        let two = k.seating().seats(&mut statics).unwrap();
        assert_eq!(one, two);
        for (x, y) in one.bottom.q.iter().zip(&two.bottom.q) {
            assert_eq!(x.to_bits(), y.to_bits());
        }
    }
}
