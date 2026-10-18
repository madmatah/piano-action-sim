//! The breakaway walk: down weight from the top seat, up weight from the
//! bottom.
//!
//! # What the trade reads
//!
//! Stanwood's down weight is *"the minimum amount of weight, to the nearest
//! gram, placed on the measuring point that causes the key to drop while
//! maintaining a slow controlled motion of the hammer"*, and his up weight
//! *"the maximum amount of weight, to the nearest gram, placed on the
//! measuring point that the key can lift"* [V, Stanwood 2000, glossary,
//! pp. 22–23]. A technician stacks gram weights on the measuring point until
//! the key goes and keeps going. The walk does the same on the model, and
//! reads the threshold in full; the trade's gram is read off it later
//! ([`super::measurement::Rounding`]).
//!
//! # The procedure
//!
//! 1. Seat the key ([`super::seat`]): at the top for D, at the bottom for U.
//! 2. Find `W0`, the frictionless balance at that seat: the weight on the
//!    measuring point at which the loads' generalized force on the key's
//!    angle vanishes there, so that the seat's stop carries nothing. On a key
//!    whose one free coordinate is its angle, one linear equation.
//! 3. From the seat, friction on and the preparation loads removed, put
//!    `W0` plus one [`STEP_G`] on the measuring point, then one more, each
//!    an increment of the static solve from the state the last one left,
//!    until an increment carries the key to the other seat. The first ones
//!    leave it where it is, held by its friction; an increment that moves it
//!    and leaves it short of the other seat, a stall, is kept, and the walk
//!    goes on from there. So D is the greatest weight the down stroke needs
//!    anywhere between the seats, and U the least the up stroke needs.
//! 4. Halve the last step [`HALVINGS`] times, each trial an increment from
//!    the state that step started from: the weight that carries the key to
//!    the other seat, to a part in 65 536 of a gram.
//!
//! A walk that takes [`CAP`] steps without the key reaching the other seat
//! is refused. With rigid stops and one constant friction torque the key
//! breaks away at its seat and runs on to the other one, so a walk takes
//! about the friction weight's grams in steps, and a step half as large
//! finds the same weight (`halving_the_walks_step_changes_nothing`).
//!
//! # Read to the gram
//!
//! The threshold is read to the trade's gram later, up for D and down for U
//! ([`super::measurement::Rounding`]), and a threshold the walk cannot tell
//! from a whole gram is read as that gram: within [`ROUNDING_TOLERANCE_G`]
//! of it, one bracket of the last halving. So the rounding of a threshold
//! that is a whole gram, nothing on bushings with no friction, is never
//! read as one more gram.
//!
//! # The end of the stroke
//!
//! The window is the stroke between the two seats while the action has no
//! let-off button, which the key alone has not. The key has reached the other
//! seat when a stop that holds it there is closed, within [`TOUCHING`] of
//! touching. A key riding on its two outer stops, off its balance punching,
//! has both closed at either seat: its seats are one, it has no stroke, and
//! the walk is refused ([`WalkRefusal::NoTravel`]), whatever rounding leaves
//! between the two solves that seat it.

use crate::contact::{ContactCode, Rows, ROW_MARGIN};
use crate::geometry::Point;
use crate::mechanism::{MaterialPoint, PointLoad, Problem};
use crate::parts::verdicts::TOUCHING;
use crate::solve::{Friction, Holder, Refusal, SolveReport, Statics};
use crate::G;

use super::seat::{Seat, Seating, Seats};

/// How much the walk adds to the weight at each step, grams: the gram the
/// trade's weights come in.
pub const STEP_G: f64 = 1.0;

/// The most steps a walk takes before it is refused.
///
/// A key's friction weight is a few grams and a key with its action some
/// tens: 500 g is a weight no protocol puts on a key's front.
pub const CAP: u32 = 500;

/// How many times the walk halves its last step: sixteen, a part in 65 536
/// of a gram, far below the gram the trade reads to.
pub const HALVINGS: u32 = 16;

/// How near a whole gram a walk's threshold may stand and be read as that
/// gram, up or down: one bracket of the last halving, [`STEP_G`] over two to
/// the [`HALVINGS`], a part in 65 536 of a gram.
///
/// Derived from the walk's own resolution. It reads a threshold as the
/// middle of the last bracket its halvings leave, so within half a bracket
/// of it; the rest of what a reading carries is far smaller, what the static
/// solve's decision leaves, under a nanogram on a key, and the rounding of
/// the balance the walk starts from, a part in 10¹⁵ of its grams. Rounded
/// with no tolerance, that noise turns into a gram: on bushings with no
/// friction the key friction weight zeroed on the frame has a threshold of
/// nothing but that rounding, the walk reads it half a bracket up, and
/// rounded up it would read 1 g. Within one bracket of a whole gram, half
/// for where the threshold may stand in it and half to spare for the rest,
/// a threshold reads as the gram it cannot be told from.
pub const ROUNDING_TOLERANCE_G: f64 = STEP_G / (1u64 << HALVINGS) as f64;

/// Which way the walk goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    /// From the top seat down: weight added, the down weight.
    Down,
    /// From the bottom seat up: weight taken away, the up weight.
    Up,
}

/// What a walk found.
#[derive(Debug, Clone, PartialEq)]
pub struct Breakaway {
    /// The threshold, grams on the measuring point: the least weight that
    /// carries the key down to the bottom seat, or the greatest that lets it
    /// rise to the top one. The middle of the last bracket the halvings
    /// leave, within half of it of the threshold.
    pub grams: f64,
    /// The frictionless balance the walk started from, grams.
    pub from_g: f64,
    /// The steps taken, the last included.
    pub steps: u32,
    /// The steps that moved the key and left it short of the other seat.
    pub stalls: u32,
    /// The seat it started from, and what held it there.
    pub held_by: Vec<ContactCode>,
    /// Every increment the walk solved, added up.
    pub report: SolveReport,
}

/// Why a walk cannot be taken to its end.
#[derive(Debug, Clone, PartialEq)]
pub enum WalkRefusal {
    /// The two seats are one: the key has no stroke between them, as a key
    /// riding on its two outer stops has none.
    NoTravel,
    /// A weight on the measuring point does not turn the key at its seat.
    NoLever,
    /// The walk took its cap of steps and the key never reached the other
    /// seat.
    DoesNotReachTheEndOfItsStroke,
    /// An increment had no equilibrium in the modeled world: these
    /// contacts fight, or the key leans on the edge of its range.
    NoEquilibrium {
        /// The contacts that fight, in the mechanism's order; empty where the
        /// key leans on a limit.
        on: Vec<ContactCode>,
    },
    /// A weight the walk put on the measuring point lifts the key off its
    /// frame: past where it leaves its balance pin
    /// ([`Mounted::leaves_its_pin`](crate::parts::mounts::Mounted::leaves_its_pin)),
    /// with nothing modeled to stop it.
    LiftsOffItsFrame,
    /// An increment reached the static solve's cap first.
    NotConverged {
        /// Steps taken.
        iterations: u32,
        /// The stationarity residual left, newtons.
        residual: f64,
    },
    /// The mounted key cannot be solved as given.
    Mechanism(Problem),
}

impl Seating<'_> {
    /// Walks the key from one seat of `seats`, which must be this seating's,
    /// to the other: the down weight for [`Stroke::Down`], the up weight for
    /// [`Stroke::Up`].
    pub fn walk(
        &self,
        statics: &mut Statics,
        seats: &Seats,
        stroke: Stroke,
    ) -> Result<Breakaway, WalkRefusal> {
        self.walk_by(statics, seats, stroke, STEP_G)
    }

    /// The walk with steps of `step_g` grams instead of the trade's gram:
    /// for the test that a step half as large finds the same weight.
    pub(crate) fn walk_by(
        &self,
        statics: &mut Statics,
        seats: &Seats,
        stroke: Stroke,
        step_g: f64,
    ) -> Result<Breakaway, WalkRefusal> {
        let m = &self.mounted.mechanism;
        let (start, end, step) = match stroke {
            Stroke::Down => (&seats.top, &seats.bottom, step_g),
            Stroke::Up => (&seats.bottom, &seats.top, -step_g),
        };
        // A key riding on its stops has no stroke: its seats are one pose,
        // to what rounding leaves between the two solves, and a walk from
        // one would read that rounding as a stroke.
        if seats.top.rides() || seats.bottom.rides() {
            return Err(WalkRefusal::NoTravel);
        }
        let ends: Vec<u16> = end
            .held_by
            .iter()
            .filter_map(|&code| m.pairs().iter().position(|p| p.code == code))
            .map(|p| u16::try_from(p).expect("fewer than 65 536 pairs"))
            .collect();
        let reached = |q: &[f64]| {
            let mut rows = Rows::new();
            m.rows(&m.pose(q), ROW_MARGIN, &mut rows);
            ends.iter()
                .any(|&p| rows.gap(p).is_some_and(|g| g <= TOUCHING))
        };
        if reached(&start.q) {
            return Err(WalkRefusal::NoTravel);
        }
        let from_g = self.frictionless_balance(start)?;
        let mut report = SolveReport::NOTHING;
        let mut increment = |statics: &mut Statics, from: &[f64], grams: f64| {
            let mut loads = self.standing.to_vec();
            loads.push(self.weight(grams));
            let eq = self
                .mounted
                .solve(statics, from, &loads, Friction::On)
                .map_err(|r| refused(self.mounted, r))?;
            report = report.and(eq.report);
            Ok::<_, WalkRefusal>(eq.q)
        };

        // The steps, each from where the last one left the key.
        let mut q = start.q.clone();
        let mut stalls = 0;
        let mut k = 0;
        let last = loop {
            if k == CAP {
                return Err(WalkRefusal::DoesNotReachTheEndOfItsStroke);
            }
            k += 1;
            let next = increment(statics, &q, from_g + f64::from(k) * step)?;
            if reached(&next) {
                break q;
            }
            if next != q {
                stalls += 1;
            }
            q = next;
        };

        // The last step halved, each trial from where that step started.
        let (mut short, mut far) = (
            from_g + f64::from(k - 1) * step,
            from_g + f64::from(k) * step,
        );
        for _ in 0..HALVINGS {
            let middle = 0.5 * (short + far);
            if reached(&increment(statics, &last, middle)?) {
                far = middle;
            } else {
                short = middle;
            }
        }
        Ok(Breakaway {
            grams: 0.5 * (short + far),
            from_g,
            steps: k,
            stalls,
            held_by: start.held_by.clone(),
            report,
        })
    }

    /// A weight of `grams` on the measuring point, straight down; a negative
    /// one pulls up.
    pub fn weight(&self, grams: f64) -> PointLoad {
        PointLoad {
            at: MaterialPoint {
                body: self.mounted.key,
                at: self.measuring_point,
            },
            force: Point::new(0.0, -grams * 1e-3 * G),
        }
    }

    /// The weight on the measuring point, grams, at which the loads'
    /// generalized force on the key's angle vanishes at `seat`, gravity and
    /// the standing loads with it: the frictionless balance, where the
    /// seat's stop carries nothing.
    ///
    /// One linear equation, the generalized force being linear in the
    /// weight at a fixed pose: exact while the key's angle is the one
    /// coordinate it moves by, on a pivot or on its balance punching, which
    /// carries it at both seats wherever a walk is taken.
    fn frictionless_balance(&self, seat: &Seat) -> Result<f64, WalkRefusal> {
        let m = &self.mounted.mechanism;
        let pose = m.pose(&seat.q);
        let angle = self.mounted.angle;
        let mut forces = vec![0.0; m.coords().len()];
        m.generalized_forces(&pose, self.standing, &mut forces);
        // A gram on the measuring point turns the key by its weight times
        // the point's vertical speed per radian, downwards.
        let at = MaterialPoint {
            body: self.mounted.key,
            at: self.measuring_point,
        };
        let per_gram = -m.jacobian_column(&pose, at, angle).y * 1e-3 * G;
        let balance = -forces[angle.0] / per_gram;
        if per_gram == 0.0 || !balance.is_finite() {
            return Err(WalkRefusal::NoLever);
        }
        Ok(balance)
    }
}

/// A static solve's refusal, as the walk's.
fn refused(mounted: &crate::parts::mounts::Mounted, refusal: Refusal) -> WalkRefusal {
    if mounted.lifts_off_its_frame(&refusal) {
        return WalkRefusal::LiftsOffItsFrame;
    }
    let m = &mounted.mechanism;
    match refusal {
        Refusal::Mechanism(p) => WalkRefusal::Mechanism(p),
        Refusal::NotConverged {
            iterations,
            residual,
        } => WalkRefusal::NotConverged {
            iterations,
            residual,
        },
        Refusal::NoEquilibrium { on } => {
            let mut pairs: Vec<u16> = on
                .iter()
                .filter_map(|h| match *h {
                    Holder::Contact { pair, .. } => Some(pair),
                    Holder::Limit { .. } => None,
                })
                .collect();
            pairs.sort_unstable();
            pairs.dedup();
            WalkRefusal::NoEquilibrium {
                on: pairs
                    .into_iter()
                    .map(|p| m.pairs()[usize::from(p)].code)
                    .collect(),
            }
        }
    }
}
