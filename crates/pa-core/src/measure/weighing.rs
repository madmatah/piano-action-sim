//! Every reading of the weighing sheet, by its procedure run on the model.
//!
//! # The readings and their procedures
//!
//! | Reading | Procedure | Read as |
//! |---|---|---|
//! | front weight | the key leveled on Stanwood's jig ([`super::jig`]) | to the nearest 0.1 g |
//! | key weight ratio | the tared jig, 10 g on the capstan's contact point | to the nearest 0.01 |
//! | key height, key dip | the seated pose ([`super::seat`]) | as found, a paper the resolution |
//! | down weight D | the breakaway walk from the top seat ([`super::walk`]) | up to the gram |
//! | up weight U | the breakaway walk from the bottom seat | down to the gram |
//! | balance weight, friction weight | `(D + U) / 2`, `(D − U) / 2`, of the two as read | from those, to the half gram |
//! | key friction weight, zeroed on the jig | Stanwood's: the front weight zeroed on the leveled jig by a temporary weight, the key back on its frame, D | up to the gram |
//! | key friction weight, zeroed on the frame | the temporary weight canceling the key's balance where it rests on its frame, D | up to the gram |
//!
//! The band is read apart from the sheet, when it is shown
//! ([`Weighing::band`]): the driven sweep from the top seat to the bottom
//! one and back ([`super::sweep`]), and on it the down weight, the greatest
//! force of its down curve, up to the gram, and the up weight, the least of
//! its up curve, down to it: D and U a second time, by a procedure that
//! shares nothing with the walk but the seats and the static solve.
//!
//! # The two key friction weights
//!
//! Stanwood zeroes the front weight on the leveled jig, with temporary
//! weights slid along the key, then puts the key back on its frame and reads
//! the least weight that makes it fall [V, Stanwood 2000, p. 22]. On its
//! frame the key rests nose-up, and what the temporary weight left of its
//! moment level is not what it leaves of it tilted: each mass's arm changes
//! by its height above the pin times the tilt. So the bench reads it both
//! ways, labeled: Stanwood's, and the temporary weight chosen instead to
//! cancel the key's balance where it rests, which reads the bushings alone.
//! The two differ by those tilt terms, a few tenths of a gram on either shipped
//! key, under the gram the protocol reads to.
//!
//! The temporary weight stands on the key's top at one station: 100 mm behind
//! the balance pin on a key heavy in front, 150 mm in front of it on one
//! heavy behind ([`STATION_BEHIND`], [`STATION_IN_FRONT`]), its mass the one
//! that zeroes, exactly, since the reading is linear in it. A technician
//! slides a weight he has; the bench sizes a weight where it stands, the same
//! cancellation.
//!
//! # The wippen the key carries
//!
//! A key carrying a wippen is seated, walked and swept with it: key height
//! and dip, D, U, BW, F and the band read the key as the bench has its
//! action, each reading saying where the wippen rests at its seat. Both key
//! friction weights read the key alone, the same key on the same frame
//! without its wippen ([`Mounted::key_alone`]): they weigh the key's
//! bushings, which the trade reads with the action off the key, and they
//! read the same bits with the wippen and without. Front weight and the key
//! weight ratio are the jig's, where the key stands alone.
//!
//! # A key friction weight typed
//!
//! The trade measures a key's bushings as Stanwood's key friction weight, and
//! that is what the owner of a key types. The bushings are a torque, and the
//! weight that protocol reads is not that torque over the measuring point's
//! level arm: on its frame the key rests nose-up, and the reading carries
//! what the tilt leaves of the key's and the temporary weight's balance, and
//! the arm the measuring point has there. So a typed key friction weight is
//! turned into a torque by running the protocol itself, inverted
//! ([`Weighing::bushings_reading`]): the torque at which it reads the typed
//! grams in full, found by bisection, which reads them back as typed. Moving
//! the measuring point or editing the key afterwards leaves the torque where
//! it is, and changes the grams the protocol reads.

use super::jig::{JigRefusal, KeyJig};
use super::measurement::{
    Bases, Conditions, Measurement, Mount, Preparation, ProcedureId, Rounding, Swept,
    TemporaryWeight, Trust, Unit, Walked, Zeroed,
};
use super::seat::{Seat, SeatRefusal, Seating, Seats, SEAT_MARGIN};
use super::sweep::{Band, Sample, SweepRefusal};
use super::walk::{Breakaway, Stroke, WalkRefusal, HALVINGS, ROUNDING_TOLERANCE_G, STEP_G};
use crate::geometry::Point;
use crate::mechanism::{AxisFriction, MaterialPoint, PointLoad, Problem};
use crate::parts::mounts::{Mounted, OnItsFrame};
use crate::parts::verdicts::wippen_carried_by;
use crate::solve::{SolveReport, Statics, SOLVER_FINGERPRINT};
use crate::G;

/// Where the temporary weight stands on a key heavy in front, meters along
/// it: 100 mm behind the balance pin.
pub const STATION_BEHIND: f64 = -0.100;

/// Where it stands on a key heavy behind: 150 mm in front of the balance
/// pin, on the covering.
pub const STATION_IN_FRONT: f64 = 0.150;

/// How far the search for the bushings a key friction weight reads on
/// reaches, grams at the measuring point with the key level: 100 g
/// ([`Weighing::bushings_reading`]).
///
/// Bushings hold a key friction weight of a few grams. Bushings that held
/// 100 g would hold the key wherever it was put, and a key friction weight
/// the search does not reach is refused as such
/// ([`BushingsRefusal::BeyondTheSearch`]).
pub const FRICTION_SEARCH_G: f64 = 100.0;

/// How many times that search halves its range: thirty-two.
///
/// It leaves 100 g over 2³², about 2·10⁻⁸ g at the measuring point, under a
/// six-hundredth of the bracket the walk reads a threshold to
/// ([`ROUNDING_TOLERANCE_G`]). So the two bushings it ends between read on
/// neighboring brackets of the walk, either side of the weight typed, and
/// the one nearer it reads within half a bracket of it, the walk's own
/// resolution: far below the gram the trade reads to.
pub const FRICTION_HALVINGS: u32 = 32;

/// Why a reading cannot be taken.
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// The key or its frame cannot be mounted as given.
    Mechanism(Problem),
    /// The reading is of the key's own weight, and nobody has weighed it:
    /// it has no mass in the model.
    KeyNotWeighed,
    /// The jig cannot read it: a scale standing where it cannot hold the key
    /// level.
    Jig(JigRefusal),
    /// The key cannot be seated on its frame.
    Seat(SeatRefusal),
    /// The walk cannot be taken to the end of the key's stroke.
    Walk(WalkRefusal),
    /// The sweep cannot be taken through the key's stroke.
    Sweep(SweepRefusal),
    /// No top of the key stands where the temporary weight goes, meters
    /// along the key.
    NoKeyTopAtTheStation {
        /// Where it goes.
        station_m: f64,
    },
    /// It is computed from a reading that is refused.
    From {
        /// That reading.
        reading: ProcedureId,
    },
}

/// One reading of the sheet: a measurement, or why it cannot be taken.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    /// Which reading.
    pub procedure: ProcedureId,
    /// It, or why not.
    pub result: Result<Measurement, Refused>,
}

/// The weighing sheet: every reading, in [`ProcedureId::ALL`]'s order, each
/// a measurement or the reason it is refused.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// The readings.
    pub readings: Vec<Reading>,
}

impl Sheet {
    /// A sheet whose every reading is refused for `why`.
    pub fn refused(why: &Refused) -> Self {
        Self {
            readings: ProcedureId::ALL
                .iter()
                .map(|&procedure| Reading {
                    procedure,
                    result: Err(why.clone()),
                })
                .collect(),
        }
    }

    /// One reading, or `None` where the sheet does not hold it.
    pub fn get(&self, procedure: ProcedureId) -> Option<&Result<Measurement, Refused>> {
        self.readings
            .iter()
            .find(|r| r.procedure == procedure)
            .map(|r| &r.result)
    }
}

/// The band, and the down and up weights read on it ([`Weighing::band`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Banded {
    /// The band: the force on the measuring point through the stroke, down
    /// and back.
    pub band: Band,
    /// The down weight read on it, [`ProcedureId::DownWeightOnTheBand`].
    pub down_weight: Measurement,
    /// The up weight read on it, [`ProcedureId::UpWeightOnTheBand`].
    pub up_weight: Measurement,
}

/// The bushings a key friction weight reads on ([`Weighing::bushings_reading`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Bushings {
    /// Their friction's bound, newton-meters.
    pub bound_n_m: f64,
    /// What Stanwood's key friction weight reads on them, grams, in full:
    /// within half a bracket of the walk, its resolution, of the weight they
    /// were found for.
    pub reads_g: f64,
    /// The static solves of every trial of the search, added up.
    pub report: SolveReport,
}

/// Why no bushings read a key friction weight by Stanwood's protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum BushingsRefusal {
    /// The weight is not a number.
    NotANumber,
    /// Stanwood's protocol is refused on this key and frame, and with it
    /// every key friction weight it would read.
    Procedure(Refused),
    /// With no friction at all the protocol reads more than the weight: what
    /// the key's tilt at rest leaves of its own balance and the temporary
    /// weight's.
    UnderNoFriction {
        /// What it reads with no friction, grams, in full.
        least_g: f64,
    },
    /// The weight lies past what the search reaches ([`FRICTION_SEARCH_G`]).
    BeyondTheSearch {
        /// The most the search read, grams, in full.
        most_g: f64,
    },
}

/// A key to weigh: on its frame as the bench mounts it, and on the jig.
#[derive(Debug, Clone, Copy)]
pub struct Weighing<'a> {
    /// The key on its frame.
    pub on_frame: &'a OnItsFrame,
    /// The same key on Stanwood's jig.
    pub jig: &'a KeyJig,
    /// The measuring point, in the key's frame.
    pub measuring_point: Point,
    /// How far back from the covering's lip it stands, meters.
    pub measuring_point_from_lip_m: f64,
    /// The thinnest paper the frame's drawer holds, meters: what key height
    /// and dip are adjusted by, and read to.
    pub thinnest_paper_m: f64,
    /// What the bench's inputs rest on.
    pub bases: Bases,
}

impl Weighing<'_> {
    /// Every reading, by its procedure, in the sheet's order. The key is
    /// seated once for key height, dip, D and U; each key friction weight
    /// seats it again with its temporary weight on, the key alone where it
    /// carries a wippen, whose own top seat the one zeroed on the frame is
    /// zeroed at: the one its seats started from, seated again only where
    /// they did not seat it there.
    pub fn sheet(&self, statics: &mut Statics) -> Sheet {
        let alone = self
            .on_frame
            .mounted
            .key_alone
            .as_ref()
            .map(|_| self.alone(&[]).seats(statics));
        let seats = match &alone {
            Some(alone) => self.seating(&[]).seats_from_the_key_alones(statics, alone),
            None => self.seating(&[]).seats(statics),
        }
        .map_err(Refused::Seat);
        let alone_top = match alone {
            Some(Ok(alone)) => Ok(alone.top),
            Some(Err(_)) => self.alone(&[]).top(statics).map_err(Refused::Seat),
            None => seats.as_ref().map(|s| s.top.clone()).map_err(Clone::clone),
        };
        let (height, dip) = self.height_and_dip(&seats);
        let [down, up, balance, friction] = self.weights(statics, &seats);
        let results = [
            (ProcedureId::FrontWeight, self.front_weight()),
            (ProcedureId::KeyWeightRatio, self.key_ratio()),
            (ProcedureId::KeyHeight, height),
            (ProcedureId::KeyDip, dip),
            (ProcedureId::DownWeight, down),
            (ProcedureId::UpWeight, up),
            (ProcedureId::BalanceWeight, balance),
            (ProcedureId::FrictionWeight, friction),
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheJig,
                self.key_friction_weight_zeroed_on_the_jig(statics),
            ),
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
                self.key_friction_weight_zeroed_on_the_frame(statics, &alone_top),
            ),
        ];
        Sheet {
            readings: results
                .into_iter()
                .map(|(procedure, result)| Reading { procedure, result })
                .collect(),
        }
    }

    /// The seated-pose procedure on this key, `standing` on it throughout.
    pub fn seating<'b>(&'b self, standing: &'b [PointLoad]) -> Seating<'b> {
        self.seating_on(&self.on_frame.mounted, standing)
    }

    /// The seated-pose procedure on the key alone, without the wippen it
    /// carries ([`Mounted::key_alone`]), `standing` on it throughout; on the
    /// key itself where it carries none.
    pub fn alone<'b>(&'b self, standing: &'b [PointLoad]) -> Seating<'b> {
        let mounted = &self.on_frame.mounted;
        self.seating_on(mounted.key_alone.as_deref().unwrap_or(mounted), standing)
    }

    /// The seated-pose procedure on `mounted`, this key's, `standing` on it.
    fn seating_on<'b>(&'b self, mounted: &'b Mounted, standing: &'b [PointLoad]) -> Seating<'b> {
        Seating {
            mounted,
            capstan: self.on_frame.capstan,
            measuring_point: self.measuring_point,
            margin: SEAT_MARGIN,
            standing,
        }
    }

    /// A measurement of `procedure`, its trust from what it reads, read to
    /// steps of `1 / per_unit` by `rounding`, a value within `within` of a
    /// whole step read as that step ([`Rounding::apply`]).
    #[allow(clippy::too_many_arguments)]
    fn measured(
        &self,
        procedure: ProcedureId,
        value: f64,
        unit: Unit,
        resolution: f64,
        per_unit: f64,
        rounding: Rounding,
        within: f64,
        conditions: Conditions,
        report: SolveReport,
    ) -> Measurement {
        Measurement {
            procedure,
            value,
            shown: rounding.apply(value, per_unit, within),
            unit,
            resolution,
            rounding,
            conditions,
            trust: Trust::of(procedure.reads(), &self.bases),
            report,
            fingerprint: SOLVER_FINGERPRINT,
        }
    }

    /// Conditions on the key's frame, in its mount, at this measuring
    /// point.
    fn on_its_frame(&self) -> Conditions {
        Conditions::at(
            Mount::Frame(self.on_frame.mount),
            self.measuring_point_from_lip_m,
        )
    }

    /// Whether the key has a mass in the model: a key nobody has weighed
    /// has none, and every reading of its own weight is refused.
    fn weighed(&self) -> Result<(), Refused> {
        let m = &self.on_frame.mounted;
        match m.mechanism.body(m.key).mass {
            Some(_) => Ok(()),
            None => Err(Refused::KeyNotWeighed),
        }
    }

    /// **Front weight**, grams, signed, to the nearest tenth: *"the amount of
    /// static weight, to the nearest 0.1 gram, that the level key, tipped on
    /// its balance-pin point, exerts at the measuring point"* [V, Stanwood
    /// 2000, glossary, p. 22].
    pub fn front_weight(&self) -> Result<Measurement, Refused> {
        let grams = self.jig.front_weight_g().map_err(jig_refused)?;
        Ok(self.measured(
            ProcedureId::FrontWeight,
            grams,
            Unit::Gram,
            0.1,
            10.0,
            Rounding::Nearest,
            0.0,
            Conditions::at(Mount::Jig, self.measuring_point_from_lip_m),
            SolveReport::NOTHING,
        ))
    }

    /// **Key weight ratio**, force convention, to the hundredth: the tared
    /// jig's reading with 10 g on the capstan, its sign turned, over the load.
    pub fn key_ratio(&self) -> Result<Measurement, Refused> {
        let ratio = self.jig.key_ratio().map_err(jig_refused)?;
        Ok(self.measured(
            ProcedureId::KeyWeightRatio,
            ratio,
            Unit::One,
            0.01,
            100.0,
            Rounding::Nearest,
            0.0,
            Conditions::at(Mount::Jig, self.measuring_point_from_lip_m),
            SolveReport::NOTHING,
        ))
    }

    /// **Key height** and **key dip**, meters: the key front's top at the
    /// top seat, and how far it falls to the bottom seat, whichever stop
    /// binds first. Read as found; the thinnest paper is the resolution,
    /// what the trade's adjustment can tell apart.
    pub fn height_and_dip(
        &self,
        seats: &Result<Seats, Refused>,
    ) -> (Result<Measurement, Refused>, Result<Measurement, Refused>) {
        let seats = match seats {
            Ok(seats) => seats,
            Err(why) => return (Err(why.clone()), Err(why.clone())),
        };
        let (top, bottom) = seats.heights(&self.on_frame.mounted, self.on_frame.front_top);
        let paper = self.thinnest_paper_m;
        let height = self.measured(
            ProcedureId::KeyHeight,
            top,
            Unit::Meter,
            paper,
            1.0,
            Rounding::None,
            0.0,
            Conditions {
                held_by: seats.top.held_by.clone(),
                wippen_carried_by: wippen_carried_by(&self.on_frame.mounted, &seats.top.q),
                preparation: Some(Preparation {
                    capstan_load_n: seats.capstan_load,
                    press_n: 0.0,
                }),
                ..self.on_its_frame()
            },
            seats.top.report,
        );
        let dip = self.measured(
            ProcedureId::KeyDip,
            top - bottom,
            Unit::Meter,
            paper,
            1.0,
            Rounding::None,
            0.0,
            Conditions {
                held_by: seats.bottom.held_by.clone(),
                wippen_carried_by: wippen_carried_by(&self.on_frame.mounted, &seats.bottom.q),
                preparation: Some(Preparation {
                    capstan_load_n: seats.capstan_load,
                    press_n: seats.press,
                }),
                ..self.on_its_frame()
            },
            seats.top.report.and(seats.bottom.report),
        );
        (Ok(height), Ok(dip))
    }

    /// **Down weight**, **up weight**, **balance weight** and **friction
    /// weight**, grams, in that order. D and U by the breakaway walk from the
    /// top seat and from the bottom one: up to the gram for D, the least
    /// whole gram that takes the key down, down to it for U, the most the key
    /// lifts. BW and F, `(D + U) / 2` and `(D − U) / 2`: shown from D and U as
    /// the trade reads them, so to the half gram; their values from D and U in
    /// full. A key nobody has weighed is refused: what the walks would read
    /// is its friction alone. Each reading's solves are its own: the seats
    /// and its walk, or the seats and both walks.
    pub fn weights(
        &self,
        statics: &mut Statics,
        seats: &Result<Seats, Refused>,
    ) -> [Result<Measurement, Refused>; 4] {
        let from = |reading| Refused::From { reading };
        let refused = |why: Refused| {
            [
                Err(why.clone()),
                Err(why),
                Err(from(ProcedureId::DownWeight)),
                Err(from(ProcedureId::DownWeight)),
            ]
        };
        if let Err(why) = self.weighed() {
            return refused(why);
        }
        let seats = match seats {
            Ok(seats) => seats,
            Err(why) => return refused(why.clone()),
        };
        let seating = self.seating(&[]);
        let walks = [Stroke::Down, Stroke::Up].map(|stroke| seating.walk(statics, seats, stroke));
        let [down, up] = walks.map(|walk| walk.map_err(Refused::Walk));
        let seated = seats.top.report.and(seats.bottom.report);
        let d = down.as_ref().map(|walk| {
            self.walk_measured(ProcedureId::DownWeight, Stroke::Down, seats, walk, None)
        });
        let u = up
            .as_ref()
            .map(|walk| self.walk_measured(ProcedureId::UpWeight, Stroke::Up, seats, walk, None));
        let (balance, friction) = match (&d, &u, &down, &up) {
            (Ok(d), Ok(u), Ok(down), Ok(up)) => {
                let report = seated.and(down.report).and(up.report);
                let derived = |procedure: ProcedureId, value: f64, shown: f64| {
                    let mut m = self.measured(
                        procedure,
                        value,
                        Unit::Gram,
                        0.5,
                        2.0,
                        Rounding::FromReadings,
                        0.0,
                        Conditions {
                            from: vec![ProcedureId::DownWeight, ProcedureId::UpWeight],
                            ..self.on_its_frame()
                        },
                        report,
                    );
                    m.shown = shown + 0.0;
                    m
                };
                (
                    Ok(derived(
                        ProcedureId::BalanceWeight,
                        0.5 * (d.value + u.value),
                        0.5 * (d.shown + u.shown),
                    )),
                    Ok(derived(
                        ProcedureId::FrictionWeight,
                        0.5 * (d.value - u.value),
                        0.5 * (d.shown - u.shown),
                    )),
                )
            }
            (Err(_), ..) => (
                Err(from(ProcedureId::DownWeight)),
                Err(from(ProcedureId::DownWeight)),
            ),
            _ => (
                Err(from(ProcedureId::UpWeight)),
                Err(from(ProcedureId::UpWeight)),
            ),
        };
        [
            d.map_err(Clone::clone),
            u.map_err(Clone::clone),
            balance,
            friction,
        ]
    }

    /// A walk's threshold as a measurement: up to the gram going down, the
    /// least gram weights that do it, down to it going up, the most the key
    /// lifts; a threshold the walk cannot tell from a whole gram, within its
    /// last bracket of it, read as that gram ([`ROUNDING_TOLERANCE_G`]). Its
    /// solves are the seats' and the walk's.
    fn walk_measured(
        &self,
        procedure: ProcedureId,
        stroke: Stroke,
        seats: &Seats,
        walk: &Breakaway,
        temporary_weight: Option<TemporaryWeight>,
    ) -> Measurement {
        // The seat a walk starts from is prepared as that seat is: the load
        // at the capstan, and at the bottom the press besides.
        let (rounding, step_g, press) = match stroke {
            Stroke::Down => (Rounding::Up, STEP_G, 0.0),
            Stroke::Up => (Rounding::Down, -STEP_G, seats.press),
        };
        self.measured(
            procedure,
            walk.grams,
            Unit::Gram,
            1.0,
            1.0,
            rounding,
            ROUNDING_TOLERANCE_G,
            Conditions {
                held_by: walk.held_by.clone(),
                wippen_carried_by: walk.wippen_carried_by.clone(),
                preparation: Some(Preparation {
                    capstan_load_n: seats.capstan_load,
                    press_n: press,
                }),
                walk: Some(Walked {
                    from_g: walk.from_g,
                    step_g,
                    steps: walk.steps,
                    stalls: walk.stalls,
                    halvings: HALVINGS,
                }),
                temporary_weight,
                ..self.on_its_frame()
            },
            seats.top.report.and(seats.bottom.report).and(walk.report),
        )
    }

    /// **The band**: the key seated, then swept from its top seat to its
    /// bottom one and back ([`super::sweep`]), and on it the **down weight**,
    /// the greatest force of the down curve, up to the gram, and the **up
    /// weight**, the least of the up curve, down to it: read as the walk's
    /// are, a value within a walk's bracket of a whole gram read as that
    /// gram, so that the two procedures are read alike. A key nobody has
    /// weighed is refused, as the walks are. Its solves are the seats' and
    /// the sweep's.
    pub fn band(&self, statics: &mut Statics) -> Result<Banded, Refused> {
        self.weighed()?;
        let seating = self.seating(&[]);
        let seats = seating.seats(statics).map_err(Refused::Seat)?;
        let band = seating.sweep(statics, &seats).map_err(Refused::Sweep)?;
        let samples = u32::try_from(band.down.len()).unwrap_or(u32::MAX);
        let report = seats.top.report.and(seats.bottom.report).and(band.report);
        let read = |procedure, extreme: &Sample, rounding, seat: &Seat, press_n| {
            self.measured(
                procedure,
                extreme.grams,
                Unit::Gram,
                1.0,
                1.0,
                rounding,
                ROUNDING_TOLERANCE_G,
                Conditions {
                    held_by: seat.held_by.clone(),
                    wippen_carried_by: wippen_carried_by(&self.on_frame.mounted, &seat.q),
                    preparation: Some(Preparation {
                        capstan_load_n: seats.capstan_load,
                        press_n,
                    }),
                    sweep: Some(Swept {
                        step: band.step,
                        samples,
                        window: band.window,
                        at: extreme.travel,
                    }),
                    ..self.on_its_frame()
                },
                report,
            )
        };
        let down_weight = read(
            ProcedureId::DownWeightOnTheBand,
            &band.down_weight,
            Rounding::Up,
            &seats.top,
            0.0,
        );
        let up_weight = read(
            ProcedureId::UpWeightOnTheBand,
            &band.up_weight,
            Rounding::Down,
            &seats.bottom,
            seats.press,
        );
        Ok(Banded {
            band,
            down_weight,
            up_weight,
        })
    }

    /// **Key friction weight, zeroed on the jig**: Stanwood's protocol. The
    /// front weight is zeroed on the leveled jig by a temporary weight on
    /// the key's top, behind the pin on a key heavy in front and in front of
    /// it on one heavy behind; the key goes back on its frame with the weight
    /// on it, its back pushed down onto its cloth; the least weight on the
    /// measuring point that makes it fall is read, up to the gram, on the key
    /// alone, without the wippen it may carry ([`Self::alone`]). A key nobody
    /// has weighed has no front weight to zero, and is refused.
    pub fn key_friction_weight_zeroed_on_the_jig(
        &self,
        statics: &mut Statics,
    ) -> Result<Measurement, Refused> {
        let front_weight = self.jig.front_weight_g().map_err(jig_refused)?;
        let temporary = if front_weight == 0.0 {
            None
        } else {
            let station = self.station(front_weight > 0.0)?;
            let per_gram = self
                .jig
                .tared_reading_g(&[self.jig.weight_on_the_key(station, 1.0)])
                .map_err(jig_refused)?;
            Some(TemporaryWeight {
                mass_g: -front_weight / per_gram,
                station,
                zeroed: Zeroed::OnTheJig,
            })
        };
        self.key_friction_weight(
            statics,
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
            temporary,
        )
    }

    /// **Key friction weight, zeroed on the frame**: the temporary weight's
    /// mass chosen to cancel the generalized force of gravity on the key
    /// where it rests at the top of its stroke, `top`, the key alone's top
    /// seat ([`Self::alone`]), so that what is left to overcome is its
    /// bushings' friction alone, read through the arm the measuring point
    /// has there. An unweighed key needs no temporary weight and is read all
    /// the same. Read on the key alone, as Stanwood's is.
    pub fn key_friction_weight_zeroed_on_the_frame(
        &self,
        statics: &mut Statics,
        top: &Result<Seat, Refused>,
    ) -> Result<Measurement, Refused> {
        let top = top.as_ref().map_err(Clone::clone)?;
        let mounted = self.alone(&[]).mounted;
        let m = &mounted.mechanism;
        let pose = m.pose(&top.q);
        let mut forces = vec![0.0; m.coords().len()];
        m.generalized_forces(&pose, &[], &mut forces);
        // Positive lifts the front: a key heavy in front turns the other way.
        let balance = forces[mounted.angle.0];
        let temporary = if balance == 0.0 {
            None
        } else {
            let station = self.station(balance < 0.0)?;
            let at = MaterialPoint {
                body: mounted.key,
                at: station,
            };
            let per_gram = -m.jacobian_column(&pose, at, mounted.angle).y * 1e-3 * G;
            Some(TemporaryWeight {
                mass_g: -balance / per_gram,
                station,
                zeroed: Zeroed::OnTheFrame,
            })
        };
        self.key_friction_weight(
            statics,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
            temporary,
        )
    }

    /// Where the temporary weight stands on the key's top: behind the pin on
    /// a key heavy in front, in front of it on one heavy behind.
    fn station(&self, heavy_in_front: bool) -> Result<Point, Refused> {
        let x = if heavy_in_front {
            STATION_BEHIND
        } else {
            STATION_IN_FRONT
        };
        self.on_frame
            .key
            .top_at(self.on_frame.mounted.mechanism.parts(), x)
            .ok_or(Refused::NoKeyTopAtTheStation { station_m: x })
    }

    /// The down weight of the key alone with `temporary` standing on it,
    /// seated again with it on: a key friction weight.
    fn key_friction_weight(
        &self,
        statics: &mut Statics,
        procedure: ProcedureId,
        temporary: Option<TemporaryWeight>,
    ) -> Result<Measurement, Refused> {
        let standing: Vec<PointLoad> = temporary
            .iter()
            .map(|t| {
                PointLoad::weight(
                    MaterialPoint {
                        body: self.on_frame.mounted.key,
                        at: t.station,
                    },
                    t.mass_g * 1e-3,
                )
            })
            .collect();
        let seating = self.alone(&standing);
        let seats = seating.seats(statics).map_err(Refused::Seat)?;
        let walk = seating
            .walk(statics, &seats, Stroke::Down)
            .map_err(Refused::Walk)?;
        Ok(self.walk_measured(procedure, Stroke::Down, &seats, &walk, temporary))
    }

    /// **The bushings a key friction weight reads on**: Stanwood's protocol
    /// inverted, the friction bound at which
    /// [`Self::key_friction_weight_zeroed_on_the_jig`] reads `kf_g` grams in
    /// full on this key, so that a key friction weight measured by that
    /// protocol and typed reads back as typed, and shows as typed.
    ///
    /// Each trial is the protocol itself, run on this key with other
    /// bushings, and its reading grows with their bound: a tighter key needs
    /// more weight to fall at every attitude of its stroke. So a bisection
    /// finds it. First the protocol with no friction: a key reading more than
    /// `kf_g` with none is refused, unless by less than the walk's bracket,
    /// when no friction is the answer. Then [`FRICTION_HALVINGS`]
    /// halvings of the bounds up to the one [`FRICTION_SEARCH_G`] at the
    /// measuring point overcome with the key level, each trial sending the
    /// search up where it reads under `kf_g` and down where it reads it or
    /// more; of the two bounds the last bracket ends on, the one whose
    /// reading stands nearer `kf_g`, within half the walk's bracket of it. A
    /// weight no trial reached is refused as past the search. A trial the
    /// protocol refuses refuses the search with its reason, and so does a
    /// weight that is not a number.
    ///
    /// Whatever bushings the key on its frame was mounted with are not read:
    /// every trial replaces them.
    pub fn bushings_reading(
        &self,
        statics: &mut Statics,
        kf_g: f64,
    ) -> Result<Bushings, BushingsRefusal> {
        if !kf_g.is_finite() {
            return Err(BushingsRefusal::NotANumber);
        }
        let mut report = SolveReport::NOTHING;
        let mut read = |statics: &mut Statics, bound_n_m: f64| {
            let on_frame = self
                .with_bushings(bound_n_m)
                .map_err(|p| BushingsRefusal::Procedure(Refused::Mechanism(p)))?;
            let trial = Weighing {
                on_frame: &on_frame,
                jig: self.jig,
                measuring_point: self.measuring_point,
                measuring_point_from_lip_m: self.measuring_point_from_lip_m,
                thinnest_paper_m: self.thinnest_paper_m,
                bases: self.bases,
            };
            let kf = trial
                .key_friction_weight_zeroed_on_the_jig(statics)
                .map_err(BushingsRefusal::Procedure)?;
            report = report.and(kf.report);
            Ok(kf.value)
        };

        let least_g = read(statics, 0.0)?;
        if least_g >= kf_g {
            if least_g - kf_g > ROUNDING_TOLERANCE_G {
                return Err(BushingsRefusal::UnderNoFriction { least_g });
            }
            return Ok(Bushings {
                bound_n_m: 0.0,
                reads_g: least_g,
                report,
            });
        }

        let (mut lo, mut lo_g) = (0.0, least_g);
        let mut hi =
            AxisFriction::of_weight_at(FRICTION_SEARCH_G, self.measuring_point.x).bound_n_m;
        let mut hi_g = None;
        for _ in 0..FRICTION_HALVINGS {
            let mid = 0.5 * (lo + hi);
            let g = read(statics, mid)?;
            if g < kf_g {
                (lo, lo_g) = (mid, g);
            } else {
                (hi, hi_g) = (mid, Some(g));
            }
        }
        let Some(hi_g) = hi_g else {
            return Err(BushingsRefusal::BeyondTheSearch { most_g: lo_g });
        };
        let (bound_n_m, reads_g) = if kf_g - lo_g < hi_g - kf_g {
            (lo, lo_g)
        } else {
            (hi, hi_g)
        };
        Ok(Bushings {
            bound_n_m,
            reads_g,
            report,
        })
    }

    /// This key on its frame with bushings bounded by `bound_n_m`, everything
    /// else as it is.
    fn with_bushings(&self, bound_n_m: f64) -> Result<OnItsFrame, Problem> {
        let mut on_frame = self.on_frame.clone();
        on_frame.mounted = on_frame.mounted.with_friction(bound_n_m)?;
        Ok(on_frame)
    }
}

/// The jig's refusal, as the sheet's: a key nobody has weighed is refused as
/// such, whatever reads it.
fn jig_refused(why: JigRefusal) -> Refused {
    match why {
        JigRefusal::KeyNotWeighed => Refused::KeyNotWeighed,
        other => Refused::Jig(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact::ContactCode;
    use crate::measure::measurement::{Basis, Input};
    use crate::measure::oracle::{band_at, Branches, KeyOnFrame};
    use crate::measure::walk::HALVINGS;
    use crate::measure::TouchweightProtocol;
    use crate::mechanism::{AxisFriction, MassProps};
    use crate::parts::fixtures::{mass_of, regulated, Setup};
    use crate::parts::frame::KeyKind;
    use crate::parts::mounts::{on_its_frame, KeyMount};
    use crate::parts::KeyMass;
    use crate::solve::statics::RESIDUAL_TOLERANCE;
    use crate::test_fixture::MEASURING_POINT_HALF_INCH;

    /// The test bench's inputs: its mass weighed by hand, the height of its
    /// center of mass estimated, its friction and its dimensions assumed.
    const TEST: Bases = Bases {
        key_mass: Basis::SetByHand,
        center_of_mass_height: Basis::Estimated,
        key_friction: Basis::Assumed,
        geometry: Basis::Assumed,
        wippen_assembly: None,
    };

    /// A key on its frame and on the jig, as the bench mounts them.
    struct Bench {
        on_frame: OnItsFrame,
        jig: KeyJig,
        a: Setup,
        friction: AxisFriction,
        mass: Option<MassProps>,
    }

    /// `a`'s key, its bushings holding `kf_g` grams at its measuring point
    /// with the key level, of mass `mass`.
    fn bench_with(a: &Setup, kf_g: f64, mass: Option<MassProps>) -> Bench {
        let friction = AxisFriction::of_weight_at(kf_g, a.protocol.measuring_point(&a.key));
        bench_on(a, friction, mass)
    }

    /// `a`'s key on bushings of `friction`, of mass `mass`.
    fn bench_on(a: &Setup, friction: AxisFriction, mass: Option<MassProps>) -> Bench {
        Bench {
            on_frame: on_its_frame(
                KeyMount::OnPunching,
                &a.key,
                &a.frame,
                KeyKind::Natural,
                mass,
                friction.bound_n_m,
                None,
            )
            .expect("the key mounts on its frame"),
            jig: KeyJig::new(&a.key, &a.protocol, mass).expect("the key mounts on the jig"),
            a: a.clone(),
            friction,
            mass,
        }
    }

    /// `a`'s key weighed by the fixture's test weighing.
    fn bench(a: &Setup, kf_g: f64) -> Bench {
        bench_with(a, kf_g, mass_of(&a.key))
    }

    /// `a`'s key weighed whole, back-heavy: 151.1 g, and a front weight of
    /// −8.1 g read 13 mm back from its lip; the height of its center of mass
    /// its drawn section's centroid.
    fn back_heavy(a: &Setup) -> Option<MassProps> {
        let height = a
            .key
            .section_centroid()
            .expect("a key that is not degenerate")
            .y;
        let weighed = KeyMass::weighed(&a.key, 151.1, -8.1, 0.013).expect("a key's weighing");
        weighed.props(height)
    }

    impl Bench {
        fn weighing(&self) -> Weighing<'_> {
            Weighing {
                on_frame: &self.on_frame,
                jig: &self.jig,
                measuring_point: self.a.protocol.measuring_point_on_key(&self.a.key),
                measuring_point_from_lip_m: self.a.protocol.measuring_point_from_lip,
                thinnest_paper_m: self.a.frame.drawer.thinnest(),
                bases: TEST,
            }
        }

        fn sheet(&self) -> Sheet {
            self.weighing().sheet(&mut Statics::new())
        }

        /// The key's angle at its two seats, radians nose-up.
        fn seats(&self) -> (f64, f64) {
            let seats = self
                .weighing()
                .seating(&[])
                .seats(&mut Statics::new())
                .expect("the key seats");
            let angle = self.on_frame.mounted.angle.0;
            (seats.top.q[angle], seats.bottom.q[angle])
        }

        /// The band by hand at `theta`, the key's own mass and `extra`.
        fn band(&self, theta: f64, extra: &[(f64, Point)]) -> Branches {
            let mut masses: Vec<(f64, Point)> = self.mass.iter().map(|m| (m.mass, m.com)).collect();
            masses.extend_from_slice(extra);
            band_at(
                &masses,
                self.a.protocol.measuring_point_on_key(&self.a.key),
                self.friction.bound_n_m,
                theta,
            )
        }
    }

    fn read(sheet: &Sheet, procedure: ProcedureId) -> &Measurement {
        sheet
            .get(procedure)
            .expect("the sheet holds every reading")
            .as_ref()
            .unwrap_or_else(|why| panic!("{procedure:?} refused: {why:?}"))
    }

    fn value(sheet: &Sheet, procedure: ProcedureId) -> f64 {
        read(sheet, procedure).value
    }

    /// What a walk resolves a threshold to, grams: half the bracket its
    /// halvings leave, and what the static solve may leave out of balance
    /// where it decides whether the key moves, its residual's tolerance of
    /// the load scale, about two newtons on these keys, over each
    /// coordinate's reach, under 0.3 m, read as grams at a measuring arm of
    /// at least 0.2 m.
    fn resolution() -> f64 {
        let halved = 0.5 * STEP_G / f64::from(1u32 << HALVINGS);
        let decided = 1000.0 * RESIDUAL_TOLERANCE * 2.0 * 0.3 / (G * 0.2);
        halved + decided
    }

    /// The benches the tests weigh: the regulated bench, the same key with
    /// three times the friction, with none, and weighed back-heavy.
    fn keys() -> Vec<(&'static str, Bench)> {
        let a = regulated();
        vec![
            ("the regulated bench", bench(&a, 6.0)),
            ("three times the friction", bench(&a, 18.0)),
            ("no friction", bench(&a, 0.0)),
            ("a back-heavy key", bench_with(&a, 6.0, back_heavy(&a))),
        ]
    }

    /// C7, consistency: **the walks read the band at the seats' attitudes,
    /// exactly.** On a key turning about its balance pin, D is where the
    /// weight on the measuring point overcomes the key's balance and its
    /// friction at the top seat's attitude, and U where it gives way to them
    /// at the bottom seat's: `(±b − M(θ)) / (g a(θ))`, by hand, each mass's
    /// and the measuring point's arm `x cos θ − y sin θ`. Agreement within
    /// the walk's resolution says the walk starts the key exactly where the
    /// hand says, and carries it on to the other seat.
    #[test]
    fn down_and_up_are_the_band_at_the_seats_attitudes() {
        for (name, b) in keys() {
            let sheet = b.sheet();
            let (top, bottom) = b.seats();
            let (d, u) = (
                value(&sheet, ProcedureId::DownWeight),
                value(&sheet, ProcedureId::UpWeight),
            );
            let (at_top, at_bottom) = (b.band(top, &[]), b.band(bottom, &[]));
            assert!(
                (d - at_top.down_g).abs() <= resolution(),
                "{name}: D {d} g, by hand {} g",
                at_top.down_g
            );
            assert!(
                (u - at_bottom.up_g).abs() <= resolution(),
                "{name}: U {u} g, by hand {} g",
                at_bottom.up_g
            );
        }
    }

    /// The tilt bound of a branch: how far the band's edge `±b` at attitude
    /// `theta` can stand from its value with the key level, grams. Derived:
    /// `D(θ) − D(0) = [(b − M₀)(a₀ − a) + (M₀ − M) a₀] / (g a₀ a)`, with
    /// `|a₀ − a| ≤ x_m (1 − cos θ) + |y_m sin θ|` for the measuring point and
    /// `|M₀ − M| ≤ m g (|x_c| (1 − cos θ) + |y_c sin θ|)` for the center of
    /// mass.
    fn tilt(b: &Bench, bound_n_m: f64, theta: f64) -> f64 {
        let mass = b.mass.expect("a weighed key");
        let point = b.a.protocol.measuring_point_on_key(&b.a.key);
        let (s, c) = (libm::sin(theta).abs(), libm::cos(theta));
        let level_arm = point.x;
        let arm = point.x * c - point.y * s;
        let moment = mass.mass * G * mass.com.x;
        let arms = point.x * (1.0 - c) + point.y.abs() * s;
        let moments = mass.mass * G * (mass.com.x.abs() * (1.0 - c) + mass.com.y.abs() * s);
        1000.0 * ((bound_n_m - moment).abs() * arms + moments * level_arm) / (G * level_arm * arm)
    }

    /// C7, consistency: **the walks read the level closed form, within what
    /// the seats' tilt moves it.** The closed form with the key level,
    /// `(±b − M) / (g d)` ([`KeyOnFrame`]), reads D and U as if the key
    /// rested level; it rests nose-up by its dip, and each edge of the band
    /// moves by its tilt terms, bounded by hand ([`tilt`]).
    #[test]
    fn down_and_up_are_the_level_closed_form_within_the_tilt() {
        for (name, b) in keys() {
            let sheet = b.sheet();
            let (top, bottom) = b.seats();
            let mass = b.mass.unwrap();
            let level = KeyOnFrame {
                weight_moment_n_m: mass.mass * G * mass.com.x,
                measuring_arm_m: b.a.protocol.measuring_point(&b.a.key),
                friction_n_m: b.friction.bound_n_m,
            }
            .branches();
            let (d, u) = (
                value(&sheet, ProcedureId::DownWeight),
                value(&sheet, ProcedureId::UpWeight),
            );
            let bound_d = tilt(&b, b.friction.bound_n_m, top) + resolution();
            let bound_u = tilt(&b, -b.friction.bound_n_m, bottom) + resolution();
            assert!(
                (d - level.down_g).abs() <= bound_d,
                "{name}: D {d} g, level {} g, apart by more than {bound_d} g",
                level.down_g
            );
            assert!(
                (u - level.up_g).abs() <= bound_u,
                "{name}: U {u} g, level {} g, apart by more than {bound_u} g",
                level.up_g
            );
        }
    }

    /// Level 4, consistency: **with no action on the key, the equation of
    /// balance closes on zero**, within what the seats' tilt adds.
    /// `BW + FW = (WW × KR) + (SW × R)`, and on a bare key the right-hand side
    /// is zero: the balance weight read on the frame by the walks is minus
    /// the front weight read on the jig, but for the tilt terms of the two
    /// seats, each bounded by hand. The whole equation, checked where its
    /// answer is known, and a sign error no plausible number hides.
    #[test]
    fn the_equation_of_balance_closes_on_a_bare_key() {
        let b = bench(&regulated(), 6.0);
        let sheet = b.sheet();
        let (top, bottom) = b.seats();
        let bw = value(&sheet, ProcedureId::BalanceWeight);
        let fw = value(&sheet, ProcedureId::FrontWeight);
        let bound = 0.5
            * (tilt(&b, b.friction.bound_n_m, top) + tilt(&b, -b.friction.bound_n_m, bottom))
            + resolution();
        assert!(
            (bw + fw).abs() <= bound,
            "BW {bw} + FW {fw} must be zero but for the tilt, {bound} g"
        );
        assert!((bw + fw).abs() > 0.01, "and on this key the tilt shows");
    }

    /// C12, consistency: **a step half as large finds the same weight.** On
    /// rigid stops the key breaks away at its seat and runs on: walked by
    /// half grams, D and U agree with the walk by grams within the two walks'
    /// resolutions.
    #[test]
    fn halving_the_walks_step_changes_nothing() {
        for (name, b) in keys() {
            let w = b.weighing();
            let seating = w.seating(&[]);
            let mut statics = Statics::new();
            let seats = seating.seats(&mut statics).unwrap();
            for stroke in [Stroke::Down, Stroke::Up] {
                let whole = seating.walk(&mut statics, &seats, stroke).unwrap();
                let half = seating
                    .walk_by(&mut statics, &seats, stroke, 0.5 * STEP_G)
                    .unwrap();
                assert!(
                    (whole.grams - half.grams).abs() <= 2.0 * resolution(),
                    "{name}, {stroke:?}: {} g by grams, {} g by half grams",
                    whole.grams,
                    half.grams
                );
                assert!(half.steps >= whole.steps, "{name}: more, shorter steps");
            }
        }
    }

    /// What `grams` at `at` on the key adds to the band's center at attitude
    /// `theta`, grams at the measuring point: its moment's arm over the
    /// measuring point's.
    fn reflected(b: &Bench, at: Point, grams: f64, theta: f64) -> f64 {
        let point = b.a.protocol.measuring_point_on_key(&b.a.key);
        let (s, c) = (libm::sin(theta), libm::cos(theta));
        grams * (at.x * c - at.y * s) / (point.x * c - point.y * s)
    }

    /// The test weight of the first non-confusion test: `grams` set on the
    /// key's top `x` meters in front of its balance pin, riding on the key
    /// and pulling straight down. Built here and nowhere else: outside the
    /// tests no mass is set on a key but the weights its protocols use, a
    /// key's leads being in its weighing.
    fn test_weight(b: &Bench, x: f64, grams: f64) -> PointLoad {
        let at = b
            .on_frame
            .key
            .top_at(b.on_frame.mounted.mechanism.parts(), x)
            .expect("the key has a top there");
        PointLoad::weight(
            MaterialPoint {
                body: b.on_frame.mounted.key,
                at,
            },
            grams * 1e-3,
        )
    }

    /// The sheet's balance and friction weight in full, with `standing` on
    /// the key through every procedure they come from: the key seated with
    /// it on, and walked from each seat with it on; and the key's angle at
    /// its two seats.
    fn balance_and_friction(b: &Bench, standing: &[PointLoad]) -> (f64, f64, f64, f64) {
        let weighing = b.weighing();
        let seating = weighing.seating(standing);
        let mut statics = Statics::new();
        let seats = seating.seats(&mut statics).expect("the key seats");
        let [d, u] = [Stroke::Down, Stroke::Up].map(|stroke| {
            seating
                .walk(&mut statics, &seats, stroke)
                .expect("the key walks")
                .grams
        });
        let angle = b.on_frame.mounted.angle.0;
        (
            0.5 * (d + u),
            0.5 * (d - u),
            seats.top.q[angle],
            seats.bottom.q[angle],
        )
    }

    /// Level 0, consistency, the first non-confusion test, through the
    /// procedures: **one gram of mass set on the key moves the balance
    /// weight and leaves the friction weight alone.** A technician checks
    /// this before anything else, and a developer never would.
    ///
    /// The test weight stands on the key's top 150 mm in front of its pin
    /// while the key is seated and walked from each seat, as the sheet reads
    /// balance and friction weight, and with nothing on it those are the
    /// sheet's to the bit. Its weight moves each edge of the band by its
    /// moment read at the measuring point, at the attitude of the seat that
    /// edge is read at: so the balance weight by the mean of the two, about
    /// 0.61 g, and the friction weight by half their difference, what the
    /// key's tilt between its two seats does to the weight's arm and the
    /// measuring point's: the tolerance, derived from the two seats' arms, a
    /// few thousandths of a gram, with the walks' resolution.
    #[test]
    fn one_gram_of_mass_moves_balance_weight_without_touching_friction() {
        let b = bench(&regulated(), 6.0);
        let sheet = b.sheet();
        let (bw0, f0, ..) = balance_and_friction(&b, &[]);
        assert_eq!(
            bw0.to_bits(),
            value(&sheet, ProcedureId::BalanceWeight).to_bits()
        );
        assert_eq!(
            f0.to_bits(),
            value(&sheet, ProcedureId::FrictionWeight).to_bits()
        );
        let weight = test_weight(&b, 0.150, 1.0);
        let (bw1, f1, top, bottom) = balance_and_friction(&b, &[weight]);
        let at = weight.at.at;
        let (on_top, on_bottom) = (reflected(&b, at, 1.0, top), reflected(&b, at, 1.0, bottom));
        let (moved_bw, moved_f) = (bw1 - bw0, f1 - f0);
        let tolerance = 2.0 * resolution();
        assert!(
            (moved_bw + 0.5 * (on_top + on_bottom)).abs() <= tolerance,
            "1 g at 150 mm moved BW by {moved_bw} g, by hand {} g",
            -0.5 * (on_top + on_bottom)
        );
        let tilt = 0.5 * (on_bottom - on_top);
        assert!(
            (moved_f - tilt).abs() <= tolerance,
            "the gram moved F by {moved_f} g, the tilt between the seats accounts for {tilt} g"
        );
        assert!(tilt.abs() < 0.01, "and that is a few thousandths of a gram");
        assert!(moved_bw < -0.5, "BW must fall by about 0.61 g: {moved_bw}");
    }

    /// Level 0, consistency, the second non-confusion test, through the
    /// procedures: **bushings 1.2 times as tight move the friction weight and
    /// leave the balance weight alone.** The bound `b` turns the band's
    /// edges by `±b / (g a)` at each seat's arm, so F grows by `0.2 b` over
    /// the mean of the two arms' inverses, and BW moves by `0.2 b` times half
    /// their difference, the measuring point standing a little nearer the
    /// pin's vertical at the top seat than at the bottom: the tolerance,
    /// derived from the two seats' arms, with the walks' resolution.
    #[test]
    fn scaling_friction_moves_friction_weight_without_touching_balance_weight() {
        let base = bench(&regulated(), 6.0);
        let tighter = bench(&regulated(), 6.0 * 1.2);
        let (s0, s1) = (base.sheet(), tighter.sheet());
        let (top, bottom) = base.seats();
        let point = base.a.protocol.measuring_point_on_key(&base.a.key);
        let arm = |theta: f64| point.x * libm::cos(theta) - point.y * libm::sin(theta);
        let extra = 1000.0 * (tighter.friction.bound_n_m - base.friction.bound_n_m) / G;
        let moved_f =
            value(&s1, ProcedureId::FrictionWeight) - value(&s0, ProcedureId::FrictionWeight);
        let moved_bw =
            value(&s1, ProcedureId::BalanceWeight) - value(&s0, ProcedureId::BalanceWeight);
        let tolerance = 2.0 * resolution();
        let expected_f = 0.5 * extra * (1.0 / arm(top) + 1.0 / arm(bottom));
        assert!(
            (moved_f - expected_f).abs() <= tolerance,
            "F moved by {moved_f} g, by hand {expected_f} g"
        );
        assert!(
            (value(&s1, ProcedureId::FrictionWeight) / value(&s0, ProcedureId::FrictionWeight)
                - 1.2)
                .abs()
                < 0.01,
            "about 1.2 times the friction weight"
        );
        let tilt = 0.5 * extra * (1.0 / arm(top) - 1.0 / arm(bottom));
        assert!(
            (moved_bw - tilt).abs() <= tolerance,
            "BW moved by {moved_bw} g, the seats' arms account for {tilt} g"
        );
        assert!(tilt.abs() < 0.01, "and that is under a hundredth of a gram");
    }

    /// The temporary weight on a key's top at its station, as a mass on the
    /// band's hand calculation.
    fn temporary(m: &Measurement) -> (f64, Point) {
        let t = m
            .conditions
            .temporary_weight
            .expect("a temporary weight stands on the key");
        (t.mass_g * 1e-3, t.station)
    }

    /// C7, consistency: **Stanwood's key friction weight is the bushings'
    /// friction and what the tilt leaves of the key's balance.** The
    /// temporary weight zeroes the leveled key's front weight on the jig,
    /// exactly: the jig reads nothing with it on. On the frame, nose-up, the
    /// key and the weight lean by their heights times the tilt, and the walk
    /// reads the band's down edge at the top seat's attitude with both on,
    /// by hand. On a key heavy in front the weight stands 100 mm behind the
    /// pin on the keystick's top; on one heavy behind, 150 mm in front on the
    /// covering's (the audit's RUST-7: the protocol runs on both).
    #[test]
    fn stanwoods_key_friction_weight_is_the_friction_and_the_tilts_residue() {
        for (name, b) in keys() {
            let a = &b.a;
            let sheet = b.sheet();
            let kf = read(&sheet, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
            let (mass, station) = temporary(kf);
            let front_weight = value(&sheet, ProcedureId::FrontWeight);
            let (x, y) = if front_weight > 0.0 {
                (STATION_BEHIND, a.key.stick_height)
            } else {
                (
                    STATION_IN_FRONT,
                    a.key.stick_height + a.key.keytop_thickness,
                )
            };
            assert_eq!(station.x.to_bits(), x.to_bits(), "{name}");
            assert!(
                (station.y - y).abs() <= 4.0 * f64::EPSILON * y,
                "{name}: {station:?}"
            );
            let zeroed = b
                .jig
                .reading_g(&[b.jig.weight_on_the_key(station, mass * 1e3)])
                .unwrap();
            assert!(
                zeroed.abs() <= 64.0 * f64::EPSILON * front_weight.abs(),
                "{name}: the jig reads {zeroed} g with the temporary weight on"
            );
            let (top, _) = b.seats();
            let by_hand = b.band(top, &[(mass, station)]).down_g;
            assert!(
                (kf.value - by_hand).abs() <= resolution(),
                "{name}: Stanwood's key friction weight {} g, by hand {by_hand} g",
                kf.value
            );
        }
    }

    /// C7, consistency: **the key friction weight zeroed on the frame is the
    /// bushings' friction alone**, read through the arm the measuring point
    /// has where the key rests: `b / (g a(θ_top))`, which the bushings'
    /// friction set at the level arm reads half a percent heavier.
    #[test]
    fn the_key_friction_weight_zeroed_on_the_frame_is_the_friction_over_the_tilted_arm() {
        for (name, b) in keys() {
            let a = &b.a;
            let sheet = b.sheet();
            let kf = value(&sheet, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
            let (top, _) = b.seats();
            let point = a.protocol.measuring_point_on_key(&a.key);
            let arm = point.x * libm::cos(top) - point.y * libm::sin(top);
            let by_hand = 1000.0 * b.friction.bound_n_m / (G * arm);
            assert!(
                (kf - by_hand).abs() <= resolution(),
                "{name}: {kf} g, by hand {by_hand} g"
            );
        }
    }

    /// Consistency: **read to the gram as gram weights give them.** D and
    /// the key friction weights up, U down, each within the walk's bracket
    /// of a whole gram read as that gram; BW and F from the two as read, a
    /// half gram possible; the front weight to the tenth, the ratio to the
    /// hundredth; key height and dip as found. On the regulated bench, a D
    /// of −0.2 g reads 0 g, never −0 g: the key goes down on its own, and no
    /// gram weight is needed.
    #[test]
    fn every_reading_is_read_as_the_trade_reads_it() {
        let sheet = bench(&regulated(), 6.0).sheet();
        let d = read(&sheet, ProcedureId::DownWeight);
        let u = read(&sheet, ProcedureId::UpWeight);
        let up = |value: f64| (value - ROUNDING_TOLERANCE_G).ceil() + 0.0;
        let down = |value: f64| (value + ROUNDING_TOLERANCE_G).floor() + 0.0;
        assert_eq!(d.shown.to_bits(), up(d.value).to_bits());
        assert_eq!(d.shown.to_bits(), 0.0_f64.to_bits(), "{d:?}");
        assert_eq!(u.shown.to_bits(), down(u.value).to_bits());
        for (p, from) in [
            (ProcedureId::BalanceWeight, 0.5 * (d.shown + u.shown)),
            (ProcedureId::FrictionWeight, 0.5 * (d.shown - u.shown)),
        ] {
            let m = read(&sheet, p);
            assert_eq!(m.shown.to_bits(), (from + 0.0).to_bits(), "{p:?}");
            assert_eq!(m.rounding, Rounding::FromReadings);
        }
        for p in [
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
        ] {
            let m = read(&sheet, p);
            assert_eq!(m.shown.to_bits(), up(m.value).to_bits(), "{p:?}");
        }
        let fw = read(&sheet, ProcedureId::FrontWeight);
        assert_eq!(
            fw.shown.to_bits(),
            ((fw.value * 10.0).round() / 10.0).to_bits()
        );
        let kr = read(&sheet, ProcedureId::KeyWeightRatio);
        assert_eq!(
            kr.shown.to_bits(),
            ((kr.value * 100.0).round() / 100.0).to_bits()
        );
        for p in [ProcedureId::KeyHeight, ProcedureId::KeyDip] {
            let m = read(&sheet, p);
            assert_eq!(m.shown.to_bits(), m.value.to_bits(), "{p:?}");
            assert_eq!(m.unit, Unit::Meter);
        }
    }

    /// Consistency: **balance and friction weight are computed from D and U
    /// as read, not rounded from their own values in full.** For the balance
    /// weight the two rules part only where the fractional parts of D and U
    /// add up to under a half or past one and a half, which the regulated
    /// bench's 6 g of friction does not give (they add up to 1.3). With
    /// bushings of 6.4 g at the measuring point, D comes to about 0.18 g, read
    /// 1 g, and U to about −12.88 g, read −13 g: a technician computes −6 g of
    /// balance weight and 7 g of friction weight, where their values in full,
    /// −6.35 g and 6.53 g, would round to −6.5 g and 6.5 g.
    #[test]
    fn balance_and_friction_weight_are_computed_from_d_and_u_as_read() {
        let sheet = bench(&regulated(), 6.4).sheet();
        let (d, u) = (
            read(&sheet, ProcedureId::DownWeight),
            read(&sheet, ProcedureId::UpWeight),
        );
        assert_eq!(d.shown.to_bits(), 1.0_f64.to_bits(), "{d:?}");
        assert_eq!(u.shown.to_bits(), (-13.0_f64).to_bits(), "{u:?}");
        for (p, as_read) in [
            (ProcedureId::BalanceWeight, -6.0_f64),
            (ProcedureId::FrictionWeight, 7.0),
        ] {
            let m = read(&sheet, p);
            assert_eq!(m.shown.to_bits(), as_read.to_bits(), "{p:?}: {m:?}");
            assert_ne!(
                ((2.0 * m.value).round() / 2.0).to_bits(),
                m.shown.to_bits(),
                "{p:?}: this case must tell the two rules apart"
            );
        }
    }

    /// Consistency: **a reading carries what makes it a datum**: the mount it
    /// was read in, the stops that held the seat it was read at or started
    /// from, the loads that seated the key, the walk and its start, the
    /// temporary weight, the readings it is computed from, its trust, and
    /// the solver's fingerprint.
    #[test]
    fn every_reading_carries_its_conditions() {
        let b = bench(&regulated(), 6.0);
        let sheet = b.sheet();
        assert_eq!(sheet.readings.len(), ProcedureId::ALL.len());
        for (r, p) in sheet.readings.iter().zip(ProcedureId::ALL) {
            assert_eq!(r.procedure, p);
            let m = r.result.as_ref().unwrap();
            assert_eq!(m.procedure, p);
            assert_eq!(m.fingerprint, crate::solve::SOLVER_FINGERPRINT);
            assert_eq!(m.trust, Trust::of(p.reads(), &TEST));
            assert!(matches!(m.trust, Trust::Indicative { .. }), "{p:?}");
        }
        let cloth = vec![ContactCode::BackRailCloth];
        let punching = vec![ContactCode::FrontRailPunching];
        let d = read(&sheet, ProcedureId::DownWeight);
        assert_eq!(d.conditions.mount, Mount::Frame(KeyMount::OnPunching));
        assert_eq!(d.conditions.held_by, cloth);
        let walked = d.conditions.walk.expect("a walk");
        let (top, bottom) = b.seats();
        let center = b.band(top, &[]);
        assert!(
            (walked.from_g - center.balance_weight_g()).abs()
                <= 64.0 * f64::EPSILON * center.down_g.abs().max(center.up_g.abs()),
            "the walk starts from the frictionless balance at the top seat"
        );
        assert!(walked.steps >= 1 && walked.stalls == 0);
        let u = read(&sheet, ProcedureId::UpWeight);
        assert_eq!(u.conditions.held_by, punching);
        assert!(u.conditions.walk.unwrap().step_g < 0.0);
        assert!(u.conditions.preparation.unwrap().press_n > 0.0);
        assert!(
            (u.conditions.walk.unwrap().from_g - b.band(bottom, &[]).balance_weight_g()).abs()
                < 1e-9
        );
        assert_eq!(
            read(&sheet, ProcedureId::KeyDip).conditions.held_by,
            punching
        );
        assert_eq!(
            read(&sheet, ProcedureId::KeyHeight).conditions.held_by,
            cloth
        );
        assert_eq!(
            read(&sheet, ProcedureId::FrontWeight).conditions.mount,
            Mount::Jig
        );
        assert_eq!(
            read(&sheet, ProcedureId::BalanceWeight).conditions.from,
            vec![ProcedureId::DownWeight, ProcedureId::UpWeight]
        );
        for (p, zeroed) in [
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheJig,
                Zeroed::OnTheJig,
            ),
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
                Zeroed::OnTheFrame,
            ),
        ] {
            let t = read(&sheet, p).conditions.temporary_weight.unwrap();
            assert_eq!(t.zeroed, zeroed);
            assert!(t.mass_g > 0.0);
        }
    }

    /// Consistency: **a key nobody has weighed reads what needs no mass.**
    /// With rigid stops, key height, key dip and the key weight ratio do not
    /// depend on the key's mass, nor does the key friction weight zeroed on
    /// the frame, which needs no temporary weight on a key with no balance
    /// to cancel; front weight, D, U, BW, F and Stanwood's key friction
    /// weight read the key's own weight, and are refused.
    #[test]
    fn an_unweighed_key_reads_what_needs_no_mass() {
        let b = bench_with(&regulated(), 6.0, None);
        let sheet = b.sheet();
        for p in [
            ProcedureId::KeyWeightRatio,
            ProcedureId::KeyHeight,
            ProcedureId::KeyDip,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
        ] {
            assert!(sheet.get(p).unwrap().is_ok(), "{p:?}");
        }
        let kf = read(&sheet, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        assert!(kf.conditions.temporary_weight.is_none());
        assert_eq!(
            Trust::of(kf.procedure.reads(), &TEST),
            Trust::Indicative {
                because: vec![Input::KeyFriction, Input::Geometry]
            }
        );
        for p in [
            ProcedureId::FrontWeight,
            ProcedureId::DownWeight,
            ProcedureId::UpWeight,
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
        ] {
            assert_eq!(
                sheet.get(p).unwrap().as_ref().err(),
                Some(&Refused::KeyNotWeighed),
                "{p:?}"
            );
        }
        for p in [ProcedureId::BalanceWeight, ProcedureId::FrictionWeight] {
            assert_eq!(
                sheet.get(p).unwrap().as_ref().err(),
                Some(&Refused::From {
                    reading: ProcedureId::DownWeight
                }),
                "{p:?}"
            );
        }
        assert_eq!(
            b.weighing().band(&mut Statics::new()),
            Err(Refused::KeyNotWeighed),
            "the band reads the key's own weight, as the walks do"
        );
    }

    /// Level 0, consistency: **every reader of the measuring point follows
    /// the protocol.** Set to the glossary's other value, half an inch, so
    /// that a reading still taken at a stale 13 mm cannot pass by agreeing
    /// with the default: the jig's front weight is the key's moment over the
    /// arm to the covering's lip less half an inch, the ratio the capstan's
    /// arm over the same; D is the band's edge at the top seat with the
    /// weight centered there, by hand, the point worked from the key's own
    /// fields; and the band's sweep reads its first step off the top seat
    /// there too, by hand at the attitude it holds, to rounding: a few
    /// products of the key's weight and arms under 0.3 m, over an arm of at
    /// least 0.2 m, 64 units in their last place.
    #[test]
    fn every_reader_of_the_measuring_point_follows_the_protocol() {
        let mut a = regulated();
        a.protocol = TouchweightProtocol {
            measuring_point_from_lip: MEASURING_POINT_HALF_INCH,
            ..crate::test_fixture::protocol()
        };
        let b = bench(&a, 6.0);
        let sheet = b.sheet();
        let lip = a.key.front_edge + a.key.keytop_overhang;
        let point = Point::new(
            lip - MEASURING_POINT_HALF_INCH,
            a.key.stick_height + a.key.keytop_thickness,
        );
        let mass = b.mass.unwrap();
        let fw = value(&sheet, ProcedureId::FrontWeight);
        assert!(
            (fw - 1000.0 * mass.mass * mass.com.x / point.x).abs() <= 1e-12,
            "front weight {fw} g is not read at {} m",
            point.x
        );
        let kr = value(&sheet, ProcedureId::KeyWeightRatio);
        assert!((kr - (-a.key.capstan_contact().x / point.x)).abs() <= 1e-12);
        let (top, _) = b.seats();
        let by_hand = band_at(&[(mass.mass, mass.com)], point, b.friction.bound_n_m, top);
        let d = value(&sheet, ProcedureId::DownWeight);
        assert!(
            (d - by_hand.down_g).abs() <= resolution(),
            "D {d} g, by hand at the half inch {} g",
            by_hand.down_g
        );
        let banded = b
            .weighing()
            .band(&mut Statics::new())
            .expect("the band is drawn");
        let first = banded.band.down[0];
        let by_hand = band_at(
            &[(mass.mass, mass.com)],
            point,
            b.friction.bound_n_m,
            first.angle,
        );
        let rounding =
            64.0 * f64::EPSILON * 1000.0 * (mass.mass * G * 0.3 + b.friction.bound_n_m) / (G * 0.2);
        assert!(
            (first.grams - by_hand.down_g).abs() <= rounding,
            "the band's first step {} g, by hand at the half inch {} g",
            first.grams,
            by_hand.down_g
        );
        assert_eq!(
            banded.down_weight.conditions.measuring_point_from_lip_m,
            MEASURING_POINT_HALF_INCH
        );
    }

    /// C10, consistency, determinism: the same key weighed twice reads the
    /// same bits, every reading.
    #[test]
    fn the_same_weighing_twice_is_the_same_bits() {
        let b = bench(&regulated(), 6.0);
        let (one, two) = (b.sheet(), b.sheet());
        assert_eq!(one, two);
        for (x, y) in one.readings.iter().zip(&two.readings) {
            let (x, y) = (x.result.as_ref().unwrap(), y.result.as_ref().unwrap());
            assert_eq!(x.value.to_bits(), y.value.to_bits());
        }
    }

    /// Consistency, and the bug it guards: **bushings with no friction read
    /// no key friction weight.** Zeroed on the frame, the temporary weight
    /// cancels the key's balance where it rests, so the threshold is nothing
    /// but the rounding of that cancellation, and the walk reads it half its
    /// last bracket up, under a hundred-thousandth of a gram: rounded up
    /// with no tolerance, that read 1 g. Within the walk's bracket of a whole
    /// gram it reads that gram, here 0 g, on a key heavy in front and on one
    /// heavy behind.
    #[test]
    fn bushings_with_no_friction_read_no_key_friction_weight() {
        let a = regulated();
        for (name, b) in [
            ("the regulated bench", bench(&a, 0.0)),
            ("a back-heavy key", bench_with(&a, 0.0, back_heavy(&a))),
        ] {
            let sheet = b.sheet();
            let kf = read(&sheet, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
            assert!(
                kf.value > 0.0 && kf.value <= resolution(),
                "{name}: the walk read a threshold of nothing as {} g",
                kf.value
            );
            assert_eq!(kf.shown.to_bits(), 0.0_f64.to_bits(), "{name}: {kf:?}");
        }
    }

    /// The keys a key friction weight is typed on, each with its mass:
    /// heavy in front, heavy behind, and weighed at the half inch.
    fn typed_on() -> Vec<(&'static str, Setup, Option<MassProps>)> {
        let a = regulated();
        let mut half_inch = regulated();
        half_inch.protocol = TouchweightProtocol {
            measuring_point_from_lip: MEASURING_POINT_HALF_INCH,
            ..crate::test_fixture::protocol()
        };
        let half_inch_mass = mass_of(&half_inch.key);
        vec![
            ("the regulated bench", a.clone(), mass_of(&a.key)),
            ("a back-heavy key", a.clone(), back_heavy(&a)),
            ("at the half inch", half_inch, half_inch_mass),
        ]
    }

    /// C7, consistency: **a key friction weight typed reads back by
    /// Stanwood's procedure, and shows as typed.** The bushings found for it,
    /// put on the key, read it in full to the walk's resolution, the very
    /// bits the search read, and a weight typed in whole grams shows as
    /// typed. Their bound is the band's by hand: the weight typed less what
    /// the protocol reads with no friction, through the arm the measuring
    /// point has at the top seat, `(W − W₀) g a(θ_top)`, within what the
    /// walk's resolution makes of a torque there, twice, for the search's
    /// reading and the hand's. On keys heavy in front and behind, and weighed
    /// at the half inch.
    #[test]
    fn a_key_friction_weight_typed_reads_back_by_stanwoods_procedure() {
        for (name, a, mass) in typed_on() {
            let point = a.protocol.measuring_point_on_key(&a.key);
            // Bushings to start from: the search replaces them.
            let b = bench_with(&a, 6.0, mass);
            for typed in [1.0, 6.0, 9.5, 14.0] {
                let found = b
                    .weighing()
                    .bushings_reading(&mut Statics::new(), typed)
                    .unwrap_or_else(|why| panic!("{name}, {typed} g: {why:?}"));
                let on = bench_on(
                    &a,
                    AxisFriction {
                        bound_n_m: found.bound_n_m,
                        radius_m: Some(point.x),
                    },
                    mass,
                );
                let sheet = on.sheet();
                let kf = read(&sheet, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
                assert_eq!(
                    kf.value.to_bits(),
                    found.reads_g.to_bits(),
                    "{name}, {typed} g: the sheet reads what the search read"
                );
                assert!(
                    (kf.value - typed).abs() <= resolution(),
                    "{name}: typed {typed} g, read back {} g",
                    kf.value
                );
                assert_eq!(
                    kf.shown.to_bits(),
                    typed.ceil().to_bits(),
                    "{name}: typed {typed} g, shown {} g",
                    kf.shown
                );

                let (top, _) = on.seats();
                let arm = point.x * libm::cos(top) - point.y * libm::sin(top);
                let mut masses: Vec<(f64, Point)> = mass.iter().map(|m| (m.mass, m.com)).collect();
                masses.push(temporary(kf));
                let none = band_at(&masses, point, 0.0, top).down_g;
                let per_gram = 1e-3 * G * arm;
                let by_hand = (typed - none) * per_gram;
                assert!(
                    (found.bound_n_m - by_hand).abs() <= 2.0 * resolution() * per_gram,
                    "{name}, {typed} g: bushings of {} N·m, by hand {by_hand} N·m",
                    found.bound_n_m
                );
            }
        }
    }

    /// Consistency: **a key friction weight no bushings read is refused, with
    /// why.** Under what the protocol reads with no friction at all, a third
    /// of a gram on the regulated bench, refused with that reading; within
    /// the walk's bracket of it, no friction is the answer; past the search,
    /// refused with the most the search read; not a number, refused; and on
    /// a frame whose stops fight, the protocol's own refusal: the key rides
    /// on its two stops and has no stroke for the protocol to walk.
    #[test]
    fn a_key_friction_weight_no_bushings_read_is_refused() {
        let a = regulated();
        let b = bench(&a, 6.0);
        let w = b.weighing();
        let least = value(
            &bench(&a, 0.0).sheet(),
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
        );
        assert!(least > 0.1 && least < 1.0, "{least}");
        assert_eq!(
            w.bushings_reading(&mut Statics::new(), 0.0),
            Err(BushingsRefusal::UnderNoFriction { least_g: least })
        );
        let just = w
            .bushings_reading(&mut Statics::new(), least - 0.5 * ROUNDING_TOLERANCE_G)
            .expect("within the walk's bracket of what no friction reads");
        assert_eq!(just.bound_n_m.to_bits(), 0.0_f64.to_bits());
        assert_eq!(just.reads_g.to_bits(), least.to_bits());
        match w.bushings_reading(&mut Statics::new(), FRICTION_SEARCH_G + 10.0) {
            Err(BushingsRefusal::BeyondTheSearch { most_g }) => assert!(
                most_g > FRICTION_SEARCH_G - 1.0 && most_g < FRICTION_SEARCH_G + 10.0,
                "{most_g}"
            ),
            other => panic!("{other:?}"),
        }
        assert_eq!(
            w.bushings_reading(&mut Statics::new(), f64::NAN),
            Err(BushingsRefusal::NotANumber)
        );

        let mut wedged = regulated();
        wedged.frame.back_rail_height =
            wedged.frame.balance_bearing_height() + 0.005 - wedged.frame.back_rail_cloth;
        let refused = bench(&wedged, 6.0)
            .weighing()
            .bushings_reading(&mut Statics::new(), 6.0);
        assert_eq!(
            refused,
            Err(BushingsRefusal::Procedure(Refused::Walk(
                WalkRefusal::NoTravel
            )))
        );
    }
}
