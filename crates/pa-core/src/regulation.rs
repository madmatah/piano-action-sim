//! The regulation solve: key height and key dip as the technician states
//! them, turned into the paper laid on the two punchings.
//!
//! # The forward map is the procedure
//!
//! What a stack gives is read by the seated pose itself
//! ([`crate::measure::seat`]), on the key mounted exactly as the bench mounts
//! it ([`crate::parts::mounts::on_its_frame`]): the key height is the height of
//! the key front's top at the top seat, the dip how far it falls to the
//! bottom seat. No small-angle formula stands between a stack and what it
//! gives, so the solve sees whatever the seats see: the key bearing on its
//! cloth's edge or on its own corner, the front pin bottoming in its mortise,
//! the pin meeting the wood beside its slot. A closed form taking the key top
//! as level would miss by the key top's height times the square of the
//! attitude, about 0.02 mm of key height and dip on the default bench, a
//! quarter of the thinnest paper.
//!
//! # Sequential bisection, in the trade's order
//!
//! A technician levels the keys first, then sets the dip on the keys as they
//! now stand. So does the solve:
//!
//! 1. the paper under the balance punching from the key height target, with
//!    no paper on the front punching, by [`HALVINGS`] halvings of
//!    [`STACK_SEARCH`];
//! 2. that paper laid from the drawer ([`papers_for`]), the closest total from
//!    below;
//! 3. the paper on the front punching from the dip target, the balance stack
//!    as laid, by the same bisection;
//! 4. that paper laid the same way, and both readings taken again by the
//!    seated pose on the papers as laid.
//!
//! While the stops do not fight, key height rises with the balance stack (at
//! the top seat the key turns about where it bears on its cloth) and does not
//! depend on the front stack; the dip does not grow with the front stack, and
//! stops changing once the front pin bottoms in its mortise or meets the wood
//! beside it, where a solve that follows a slope has none to follow. A
//! bisection needs only the order. Each stack keeps the felt fitted on it;
//! the search runs over the paper laid on the felt.
//!
//! Where the stops fight at a stack, the key rides on them off its balance
//! punching, its seats one: no paper under that punching moves it, and no
//! dip can be read. A trial that rides counts as one whose stops fight, so
//! the search moves away from it. A key that still rides with all the paper
//! the search lays rides at every stack, at the height its stops set: a key
//! height under it is refused as the stops' doing, one over it as the
//! rail's, which only more paper would reach.
//!
//! A stack can also stand the key off its balance pin
//! ([`crate::parts::mounts::Mounted::leaves_its_pin`]): under the balance
//! punching, paper that raises the punching past the pin's top; on the front
//! punching, paper that makes the key ride past where it leaves its pin.
//! Neither is a seat. For key height such a trial counts as one past the
//! target, since more paper only raises the punching further, and a key
//! height no stack reaches on the pin is refused as the pin's doing; for the
//! dip it counts as one whose stops fight. A key off its pin with no paper at
//! all has no seat at any stack.
//!
//! # Within one paper, and refused with its reason
//!
//! A drawer of eight thicknesses cannot make every stack, so the regulation
//! lands within one paper of its targets, not on them: the stack laid from
//! below puts the key height under its target, and the dip past it, each by
//! less than what the thinnest paper on that stack changes it by. Both
//! readings are taken again on the papers as laid, and returned beside the
//! targets.
//!
//! A target no stack reaches is refused ([`TargetOutcome::Infeasible`]),
//! naming the part that limits it and how far the target lies out of reach,
//! in the target's own reading: a dip the front pin forbids by bottoming in
//! its mortise is refused with how much deeper it is than the pin lets the
//! key go. It is never laid as the nearest stack and shown as a miss.

use crate::contact::ContactCode;
use crate::measure::seat::{SeatRefusal, Seating, SEAT_MARGIN};
use crate::measure::TouchweightProtocol;
use crate::mechanism::MassProps;
use crate::parts::frame::{papers_for, FrameDesign, KeyKind};
use crate::parts::mounts::{on_its_frame, KeyMount, OnItsFrame};
use crate::parts::KeyDesign;
use crate::solve::{SolveReport, Statics};

/// The most paper the search lays on one stack, metres: 20 mm.
///
/// A punching stack is a felt and a millimetre or two of paper. A target
/// that asks for more than this is one the rail is too low for, and is
/// refused as such ([`Limit::RailTooLow`]).
pub const STACK_SEARCH: f64 = 0.020;

/// How many times the search halves [`STACK_SEARCH`]: forty.
///
/// It leaves 18 femtometres of paper, which moves a key height by less than
/// a tenth of a picometre: far below anything read, and below the
/// picometre the static solve closes a contact to.
pub const HALVINGS: u32 = 40;

/// The trade's two targets for a key on its frame, metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Targets {
    /// Key height: the top of the key at its front, above the keybed, with
    /// the key at the top of its stroke.
    pub key_height: f64,
    /// Key dip: how far the top of the key at its front falls from the top
    /// of its stroke to the bottom.
    pub key_dip: f64,
}

/// One of the two targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Key height, set by the balance punching.
    KeyHeight,
    /// Key dip, set by the front punching.
    KeyDip,
}

/// What keeps a target out of reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    /// The front rail pin bottoms in the key's mortise before the target is
    /// reached: a dip deeper than the pin lets the key go, or a key height
    /// lower than it lets the key stand at rest.
    FrontPinInMortise,
    /// The front rail pin meets the key's wood beside its mortise before the
    /// target is reached.
    FrontPinOnTheWood,
    /// The felt fitted on the stack is thicker than the whole stack the
    /// target asks for: with no paper at all the key stands too high, or the
    /// front felt alone stops the dip short.
    FeltThickerThanTheStack,
    /// The key's stops fight before the target is reached: the front
    /// punching meets the key at rest while its tail is down on its cloth.
    StopsFight,
    /// Nothing stops the front of the key: the front punching's centre
    /// stands off the key's underside, and no dip can be read at all.
    NoStopBelowTheFront,
    /// The rail stands too low under the key for the target: the target asks
    /// for more than [`STACK_SEARCH`] of paper on its stack.
    RailTooLow,
    /// The key would leave its balance pin before the target is reached: the
    /// paper the target asks for raises the balance punching, and the key on
    /// it, past the pin's top.
    OffItsBalancePin,
}

/// One punching stack as the solve leaves it.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// The paper the target asks for, laid on the stack's felt, metres: the
    /// continuous thickness the bisection finds, which no drawer holds.
    pub asked: f64,
    /// The papers laid from the drawer, thickest first: the closest total
    /// to [`Self::asked`] from below.
    pub papers: Vec<f64>,
}

/// The two stacks.
#[derive(Debug, Clone, PartialEq)]
pub struct Stacks {
    /// Under the key at the balance pin: key height.
    pub balance: Stack,
    /// On the front rail: key dip.
    pub front: Stack,
}

/// What the papers laid give, read again by the seated pose, metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Achieved {
    /// Key height.
    pub key_height: f64,
    /// Key dip.
    pub key_dip: f64,
}

/// How a regulation comes out.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetOutcome {
    /// Both targets met: the papers to lay, what they give, and how the
    /// seats that read them were solved, every trial of the bisections
    /// included.
    Met {
        /// The stacks.
        stacks: Stacks,
        /// What the papers laid give.
        achieved: Achieved,
        /// The static solves, added together.
        report: SolveReport,
    },
    /// A target no stack reaches.
    Infeasible {
        /// Which.
        target: Target,
        /// What keeps it out of reach.
        limited_by: Limit,
        /// How far it lies out of reach, metres of its own reading: the
        /// target less the nearest key height or dip any stack gives, by
        /// magnitude; the whole target where no dip can be read at all.
        shortfall_m: f64,
    },
}

/// Why the solve cannot run at all, whatever the stacks.
#[derive(Debug, Clone, PartialEq)]
pub enum Unsolvable {
    /// A target is not a number.
    NotFinite(Target),
    /// The key could not be seated at one of the stacks the search tried,
    /// for a reason other than its stops fighting. The solve stops there: it
    /// does not show that no other stack would seat it.
    NoSeat(SeatRefusal),
}

/// A key on its frame as the solve takes it.
#[derive(Debug, Clone, Copy)]
pub struct Regulating<'a> {
    /// The key.
    pub key: &'a KeyDesign,
    /// Its frame, with the felts fitted on the two punchings. The paper
    /// already laid on them is not read: the solve lays its own.
    pub frame: &'a FrameDesign,
    /// Which row of pins the key stands on.
    pub kind: KeyKind,
    /// The key's mass, `None` for a key nobody has weighed. With rigid stops
    /// the seats do not depend on it; the bench passes the mass it weighs
    /// with, so that what the solve reads is what the bench reads.
    pub mass: Option<MassProps>,
    /// The protocol, whose measuring point the press of the bottom seat acts
    /// at.
    pub protocol: &'a TouchweightProtocol,
}

/// One trial of a bisection: a stack, seated.
enum Trial {
    /// Seated: the reading, and the contacts holding the seat that bounds
    /// the reading (the top seat for key height, the bottom one for the dip).
    Seated {
        reading: f64,
        held_by: Vec<ContactCode>,
    },
    /// The stops fight at this stack, naming them: the key has no seat
    /// between them.
    Fights(Vec<ContactCode>),
    /// The stops fight at this stack and the key rides on them, lifted off
    /// its balance punching: the stops it rides on, and the reading where it
    /// rides, which they set and the stack does not.
    Rides {
        held_by: Vec<ContactCode>,
        reading: f64,
    },
    /// Nothing stops the front of the key.
    NoStopBelowTheFront,
    /// The key stands off its balance pin at this stack, and off its frame:
    /// how far above its punching it leaves its pin, metres, negative where
    /// the punching stands above the pin's top.
    OffItsPin { leaves_its_pin: f64 },
}

/// Which stack a bisection searches.
#[derive(Clone, Copy)]
enum Searched {
    /// The balance stack, for key height, with no paper on the front.
    Balance,
    /// The front stack, for the dip, with this much paper under the balance
    /// punching, metres.
    Front { balance: f64 },
}

/// Where a bisection ends.
enum Bisected {
    /// The paper the target asks for.
    Asked(f64),
    /// The target out of reach.
    Out(TargetOutcome),
}

/// The part that limits a target where the stops fight, from the contacts
/// that fight: the pin where it is one of them; for the dip the front
/// punching's felt, for key height the fight itself.
fn fight_limit(target: Target, fighting: &[ContactCode]) -> Limit {
    if fighting.contains(&ContactCode::FrontPinInTheMortise) {
        Limit::FrontPinInMortise
    } else if fighting.contains(&ContactCode::FrontPinOnTheWood) {
        Limit::FrontPinOnTheWood
    } else if target == Target::KeyDip && fighting.contains(&ContactCode::FrontRailPunching) {
        Limit::FeltThickerThanTheStack
    } else {
        Limit::StopsFight
    }
}

/// The part that limits a dip at its deepest, with no paper on the front
/// punching: the pin where it holds the key there, else the felt.
fn bottom_limit(held_by: &[ContactCode]) -> Limit {
    if held_by.contains(&ContactCode::FrontPinInTheMortise) {
        Limit::FrontPinInMortise
    } else if held_by.contains(&ContactCode::FrontPinOnTheWood) {
        Limit::FrontPinOnTheWood
    } else {
        Limit::FeltThickerThanTheStack
    }
}

impl Regulating<'_> {
    /// Solves the two stacks that meet `targets`, and reads what the papers
    /// laid give.
    pub fn solve(&self, targets: Targets) -> Result<TargetOutcome, Unsolvable> {
        if !targets.key_height.is_finite() {
            return Err(Unsolvable::NotFinite(Target::KeyHeight));
        }
        if !targets.key_dip.is_finite() {
            return Err(Unsolvable::NotFinite(Target::KeyDip));
        }
        let mut statics = Statics::new();
        let mut report = SolveReport::NOTHING;

        let asked_balance = match self.bisect(
            &mut statics,
            &mut report,
            Searched::Balance,
            targets.key_height,
        )? {
            Bisected::Asked(paper) => paper,
            Bisected::Out(outcome) => return Ok(outcome),
        };
        let balance_papers = papers_for(asked_balance);
        let laid_balance: f64 = balance_papers.iter().sum();

        let asked_front = match self.bisect(
            &mut statics,
            &mut report,
            Searched::Front {
                balance: laid_balance,
            },
            targets.key_dip,
        )? {
            Bisected::Asked(paper) => paper,
            Bisected::Out(outcome) => return Ok(outcome),
        };
        let front_papers = papers_for(asked_front);

        let achieved = self.read(&mut statics, &mut report, &balance_papers, &front_papers)?;
        Ok(TargetOutcome::Met {
            stacks: Stacks {
                balance: Stack {
                    asked: asked_balance,
                    papers: balance_papers,
                },
                front: Stack {
                    asked: asked_front,
                    papers: front_papers,
                },
            },
            achieved,
            report,
        })
    }

    /// The frame with `balance` and `front` metres of paper on its
    /// punchings, its felts as fitted.
    fn frame_with(&self, balance: f64, front: f64) -> FrameDesign {
        let mut frame = self.frame.clone();
        frame.balance_punching.paper = balance;
        frame.front_punching.paper = front;
        frame
    }

    /// The key on `frame`, these papers drawn on its punchings. A trial
    /// draws none, its felt then standing on the bare rail up to the
    /// bearing height its stack gives: no contact reads a paper, only the
    /// felt's top.
    fn mount(
        &self,
        frame: &FrameDesign,
        balance_papers: &[f64],
        front_papers: &[f64],
    ) -> Result<OnItsFrame, Unsolvable> {
        // Seats are solved friction off, so the bushings' bound is not read.
        on_its_frame(
            KeyMount::OnPunching,
            self.key,
            frame,
            self.kind,
            balance_papers,
            front_papers,
            self.mass,
            0.0,
        )
        .map_err(|p| Unsolvable::NoSeat(SeatRefusal::Mechanism(p)))
    }

    /// The seated-pose procedure on `k`, as the bench runs it.
    fn seating<'b>(&self, k: &'b OnItsFrame) -> Seating<'b> {
        Seating {
            mounted: &k.mounted,
            capstan: k.capstan,
            measuring_point: self.protocol.measuring_point_on_key(self.key),
            margin: SEAT_MARGIN,
            standing: &[],
        }
    }

    /// Key height with `paper` under the balance punching and none on the
    /// front: the top seat alone.
    fn height_trial(
        &self,
        statics: &mut Statics,
        report: &mut SolveReport,
        paper: f64,
    ) -> Result<Trial, Unsolvable> {
        let frame = self.frame_with(paper, 0.0);
        let k = self.mount(&frame, &[], &[])?;
        match self.seating(&k).top(statics) {
            Ok(seat) if seat.rides() => {
                *report = report.and(seat.report);
                Ok(Trial::Rides {
                    reading: seat.height(&k.mounted, k.front_top),
                    held_by: seat.held_by,
                })
            }
            Ok(seat) => {
                *report = report.and(seat.report);
                Ok(Trial::Seated {
                    reading: seat.height(&k.mounted, k.front_top),
                    held_by: seat.held_by,
                })
            }
            Err(SeatRefusal::CannotSeat { fighting }) => Ok(Trial::Fights(fighting)),
            Err(SeatRefusal::LiftsOffItsFrame { leaves_its_pin }) => {
                Ok(Trial::OffItsPin { leaves_its_pin })
            }
            Err(refusal) => Err(Unsolvable::NoSeat(refusal)),
        }
    }

    /// The dip with `balance` metres of paper under the balance punching and
    /// `paper` on the front: both seats.
    fn dip_trial(
        &self,
        statics: &mut Statics,
        report: &mut SolveReport,
        balance: f64,
        paper: f64,
    ) -> Result<Trial, Unsolvable> {
        let frame = self.frame_with(balance, paper);
        let k = self.mount(&frame, &[], &[])?;
        match self.seating(&k).seats(statics) {
            Ok(seats) if seats.top.rides() => {
                *report = report.and(seats.top.report).and(seats.bottom.report);
                let (top, bottom) = seats.heights(&k.mounted, k.front_top);
                Ok(Trial::Rides {
                    reading: top - bottom,
                    held_by: seats.top.held_by,
                })
            }
            Ok(seats) => {
                *report = report.and(seats.top.report).and(seats.bottom.report);
                let (top, bottom) = seats.heights(&k.mounted, k.front_top);
                Ok(Trial::Seated {
                    reading: top - bottom,
                    held_by: seats.bottom.held_by,
                })
            }
            Err(SeatRefusal::CannotSeat { fighting }) => Ok(Trial::Fights(fighting)),
            Err(SeatRefusal::NoStopBelowTheFront) => Ok(Trial::NoStopBelowTheFront),
            Err(SeatRefusal::LiftsOffItsFrame { leaves_its_pin }) => {
                Ok(Trial::OffItsPin { leaves_its_pin })
            }
            Err(refusal) => Err(Unsolvable::NoSeat(refusal)),
        }
    }

    /// Both readings on the papers as laid, by the seated pose on the key
    /// mounted with those very papers, as the bench mounts it.
    fn read(
        &self,
        statics: &mut Statics,
        report: &mut SolveReport,
        balance_papers: &[f64],
        front_papers: &[f64],
    ) -> Result<Achieved, Unsolvable> {
        let frame = self.frame_with(balance_papers.iter().sum(), front_papers.iter().sum());
        let k = self.mount(&frame, balance_papers, front_papers)?;
        let seats = self
            .seating(&k)
            .seats(statics)
            .map_err(Unsolvable::NoSeat)?;
        *report = report.and(seats.top.report).and(seats.bottom.report);
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        Ok(Achieved {
            key_height: top,
            key_dip: top - bottom,
        })
    }

    /// The paper one stack asks for, by bisection over `[0, STACK_SEARCH]`.
    ///
    /// Key height rises with its stack, and the stops fight below the stack
    /// at which the front of the key comes down onto its front punching or
    /// its pin, the key riding on them; the dip falls with its stack, and the
    /// stops fight above the stack at which the front punching meets the key
    /// at rest. So a trial that fights sends the search up for key height and
    /// down for the dip,
    /// and the answer is bracketed by two seated trials, one each side of the
    /// target, [`HALVINGS`] halvings apart: the one past the target for key
    /// height, so that its reading is not under it, and the one short of it
    /// for the dip, so that its reading is not shallower. A trial that
    /// stands the key off its balance pin sends the search down either way:
    /// past the target for key height, the stops fighting for the dip. A
    /// target the search cannot bracket is refused, naming what keeps it out.
    fn bisect(
        &self,
        statics: &mut Statics,
        report: &mut SolveReport,
        searched: Searched,
        wanted: f64,
    ) -> Result<Bisected, Unsolvable> {
        let (target, rising) = match searched {
            Searched::Balance => (Target::KeyHeight, true),
            Searched::Front { .. } => (Target::KeyDip, false),
        };
        let trial = |statics: &mut Statics, report: &mut SolveReport, paper: f64| match searched {
            Searched::Balance => self.height_trial(statics, report, paper),
            Searched::Front { balance } => self.dip_trial(statics, report, balance, paper),
        };
        // How far past the target a reading stands, in the direction more
        // paper takes it.
        let past = |reading: f64| {
            if rising {
                reading - wanted
            } else {
                wanted - reading
            }
        };
        let out = |limited_by: Limit, shortfall: f64| {
            Ok(Bisected::Out(TargetOutcome::Infeasible {
                target,
                limited_by,
                shortfall_m: shortfall.abs(),
            }))
        };

        // The bare felt: no paper at all.
        let mut lo_seated = false;
        let mut lo_reading = f64::NAN;
        // For key height, the reading at the low end of the bracket, seated
        // or riding; `None` where the stops fight there with no seat.
        let mut lo_at: Option<f64> = None;
        let mut last_fight: Vec<ContactCode> = Vec::new();
        match trial(statics, report, 0.0)? {
            Trial::Seated { reading, held_by } => {
                let by = past(reading);
                if by == 0.0 {
                    return Ok(Bisected::Asked(0.0));
                }
                if by > 0.0 {
                    let limit = if rising {
                        Limit::FeltThickerThanTheStack
                    } else {
                        bottom_limit(&held_by)
                    };
                    return out(limit, by);
                }
                lo_seated = true;
                lo_reading = reading;
                lo_at = Some(reading);
            }
            Trial::Fights(fighting) => {
                if !rising {
                    // No dip at all with the felt alone.
                    return out(fight_limit(target, &fighting), wanted);
                }
                last_fight = fighting;
            }
            Trial::Rides { held_by, reading } => {
                if !rising {
                    return out(fight_limit(target, &held_by), wanted);
                }
                last_fight = held_by;
                lo_at = Some(reading);
            }
            Trial::NoStopBelowTheFront => return out(Limit::NoStopBelowTheFront, wanted),
            // Off its pin with no paper at all: no stack has it on its pin.
            Trial::OffItsPin { leaves_its_pin } => {
                return Err(Unsolvable::NoSeat(SeatRefusal::LiftsOffItsFrame {
                    leaves_its_pin,
                }));
            }
        }

        // The most paper the search lays.
        let mut hi_seated = true;
        let mut hi_reading = f64::NAN;
        match trial(statics, report, STACK_SEARCH)? {
            Trial::Seated { reading, held_by } => {
                let by = past(reading);
                if by < 0.0 {
                    let limit = if rising && held_by.contains(&ContactCode::FrontPinOnTheWood) {
                        Limit::FrontPinOnTheWood
                    } else {
                        Limit::RailTooLow
                    };
                    return out(limit, by);
                }
                hi_reading = reading;
            }
            // Riding with the most paper the search lays, the key rides with
            // any less, at the height its stops set: a target under it is
            // kept out by the stops, one over it by the rail, which more
            // paper than the search lays would lift the key off them towards.
            Trial::Rides { held_by, reading } if rising => {
                let by = past(reading);
                let limit = if by < 0.0 {
                    Limit::RailTooLow
                } else {
                    fight_limit(target, &held_by)
                };
                return out(limit, by);
            }
            Trial::Fights(fighting)
            | Trial::Rides {
                held_by: fighting, ..
            } => {
                if rising {
                    return Err(Unsolvable::NoSeat(SeatRefusal::CannotSeat { fighting }));
                }
                hi_seated = false;
                last_fight = fighting;
            }
            Trial::NoStopBelowTheFront => return out(Limit::NoStopBelowTheFront, wanted),
            // Off its pin with the most paper the search lays: the search
            // goes on below it.
            Trial::OffItsPin { .. } => hi_seated = false,
        }

        let (mut lo, mut hi) = (0.0, STACK_SEARCH);
        for _ in 0..HALVINGS {
            let mid = 0.5 * (lo + hi);
            match trial(statics, report, mid)? {
                Trial::Seated { reading, .. } if past(reading) < 0.0 => {
                    lo = mid;
                    lo_seated = true;
                    lo_reading = reading;
                    lo_at = Some(reading);
                }
                Trial::Seated { reading, .. } => {
                    hi = mid;
                    hi_seated = true;
                    hi_reading = reading;
                }
                Trial::Fights(fighting) => {
                    if rising {
                        lo = mid;
                        lo_seated = false;
                        lo_at = None;
                    } else {
                        hi = mid;
                        hi_seated = false;
                    }
                    last_fight = fighting;
                }
                Trial::Rides { held_by, reading } => {
                    if rising {
                        lo = mid;
                        lo_seated = false;
                        lo_at = Some(reading);
                    } else {
                        hi = mid;
                        hi_seated = false;
                    }
                    last_fight = held_by;
                }
                Trial::NoStopBelowTheFront => return out(Limit::NoStopBelowTheFront, wanted),
                Trial::OffItsPin { .. } => {
                    hi = mid;
                    hi_seated = false;
                }
            }
        }

        if rising {
            if !hi_seated {
                // Past the target the key stands off its balance pin, so the
                // target lies past the highest key height the pin lets the
                // stack reach: the reading at the low end of the bracket,
                // seated there or riding. A key riding past the target at
                // every stack on its pin is kept out by its stops.
                return match lo_at {
                    Some(reading) if past(reading) < 0.0 => {
                        out(Limit::OffItsBalancePin, past(reading))
                    }
                    Some(reading) => out(fight_limit(target, &last_fight), past(reading)),
                    None => Err(Unsolvable::NoSeat(SeatRefusal::CannotSeat {
                        fighting: last_fight,
                    })),
                };
            }
            if !lo_seated {
                // Every seated trial stood past the target: it lies below
                // the lowest key height the stops leave.
                return out(fight_limit(target, &last_fight), past(hi_reading));
            }
            Ok(Bisected::Asked(hi))
        } else {
            if !hi_seated {
                // The target lies shallower than the shallowest dip the key
                // has before the stops fight: the one short of it, seated.
                return out(fight_limit(target, &last_fight), past(lo_reading));
            }
            Ok(Bisected::Asked(lo))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact::Rows;
    use crate::geometry::Point;
    use crate::measure::oracle;
    use crate::measure::seat::Seats;
    use crate::parts::fixtures::{mass_of, TARGETS};
    use crate::parts::frame::PAPER_THICKNESSES;
    use crate::solve::statics::GAP_TOLERANCE;

    /// A key on its frame to regulate, owned.
    struct Bench {
        key: KeyDesign,
        frame: FrameDesign,
        kind: KeyKind,
        protocol: TouchweightProtocol,
    }

    impl Bench {
        fn standard() -> Self {
            Self {
                key: KeyDesign::default(),
                frame: FrameDesign::default(),
                kind: KeyKind::Natural,
                protocol: TouchweightProtocol::default(),
            }
        }

        fn regulating(&self) -> Regulating<'_> {
            Regulating {
                key: &self.key,
                frame: &self.frame,
                kind: self.kind,
                mass: mass_of(&self.key),
                protocol: &self.protocol,
            }
        }

        fn solve(&self, targets: Targets) -> Result<TargetOutcome, Unsolvable> {
            self.regulating().solve(targets)
        }

        /// The key seated with these totals of paper on its two punchings.
        fn seated(&self, balance: f64, front: f64) -> (OnItsFrame, Seats) {
            let r = self.regulating();
            let k = r
                .mount(&r.frame_with(balance, front), &[], &[])
                .expect("the key mounts");
            let seats = r
                .seating(&k)
                .seats(&mut Statics::new())
                .expect("the key seats");
            (k, seats)
        }
    }

    /// The paper of a stack of papers, metres.
    fn total(papers: &[f64]) -> f64 {
        papers.iter().sum()
    }

    /// The thinnest paper in the drawer, metres.
    const ONE_PAPER: f64 = PAPER_THICKNESSES[0];

    /// What a seat's convergence leaves in a reading of the key front's
    /// height, metres: each contact closed to within [`GAP_TOLERANCE`], seen
    /// at the key front through the lever from the contact, at most the key
    /// front's 258 mm over the 200 mm of the nearest stop's arm; twice, for
    /// the two readings a bisection compares across its last bracket.
    const SEAT_NOISE: f64 = 2.0 * GAP_TOLERANCE * 0.258 / 0.200;

    /// The paper the bisection resolves, metres: [`STACK_SEARCH`] halved
    /// [`HALVINGS`] times, each halving exact.
    fn resolution() -> f64 {
        (0..HALVINGS).fold(STACK_SEARCH, |width, _| 0.5 * width)
    }

    fn met(outcome: Result<TargetOutcome, Unsolvable>) -> (Stacks, Achieved) {
        match outcome {
            Ok(TargetOutcome::Met {
                stacks, achieved, ..
            }) => (stacks, achieved),
            other => panic!("not met: {other:?}"),
        }
    }

    /// The cases the tests regulate: the bench's targets, a higher key and a
    /// shallower dip, a deeper dip, a lower key on thinner felts, and a
    /// sharp, turning on its own row's balance pin, its mortise over its own
    /// row's front pin.
    fn cases() -> Vec<(&'static str, Bench, Targets)> {
        let thin = {
            let mut b = Bench::standard();
            b.frame.balance_punching.felt = 0.0005;
            b.frame.front_punching.felt = 0.0008;
            b
        };
        let sharp = {
            let mut b = Bench::standard();
            b.kind = KeyKind::Sharp;
            let rows = &b.frame.sections;
            b.key.front_mortise_from_front_edge +=
                rows.front.sharp_row_offset - rows.balance.sharp_row_offset;
            b
        };
        vec![
            ("the bench's targets", Bench::standard(), TARGETS),
            (
                "a higher key, a shallower dip",
                Bench::standard(),
                Targets {
                    key_height: 0.0660,
                    key_dip: 0.0100,
                },
            ),
            (
                "a deeper dip",
                Bench::standard(),
                Targets {
                    key_height: 0.0650,
                    key_dip: 0.0110,
                },
            ),
            (
                "a lower key on thinner felts",
                thin,
                Targets {
                    key_height: 0.0640,
                    key_dip: 0.0100,
                },
            ),
            ("a sharp", sharp, TARGETS),
        ]
    }

    /// A reading, seated.
    fn reading(trial: Result<Trial, Unsolvable>) -> f64 {
        match trial {
            Ok(Trial::Seated { reading, .. }) => reading,
            Ok(Trial::Fights(fighting)) => panic!("the stops fight: {fighting:?}"),
            Ok(Trial::Rides { held_by, .. }) => panic!("the key rides on {held_by:?}"),
            Ok(Trial::NoStopBelowTheFront) => panic!("nothing stops the front"),
            Ok(Trial::OffItsPin { leaves_its_pin }) => {
                panic!("the key stands off its pin, which it leaves {leaves_its_pin} m up")
            }
            Err(e) => panic!("{e:?}"),
        }
    }

    /// Consistency, and the check of `CLAUDE.md` rule 2: **the regulated key
    /// meets its targets, then lands within one paper of them.**
    ///
    /// Meets: the paper the bisection asks for, read back by the seated pose,
    /// gives each target, the dip on the balance stack as laid. Not to
    /// within a tolerance anybody chose: within what the bisection resolves,
    /// [`STACK_SEARCH`] halved [`HALVINGS`] times, through the stack's lever
    /// on its reading (under three: 2.2 for key height, 1.15 for the dip),
    /// and what the seats' convergence leaves in a reading. Not an
    /// independent test, since the solve reads those very procedures; the
    /// independent route is the exact inverse of the key on a pivot, below.
    ///
    /// Then lands: the papers laid from the drawer, the closest total from
    /// below, give a key height at or under its target and a dip at or past
    /// it, and one more thinnest paper on either stack would carry its
    /// reading beyond the target: the regulation is as close as the drawer
    /// allows, from below, and the miss is less than one paper. Every paper
    /// is one a technician could pick up, every seat of every trial
    /// converged, and what the solve reports as achieved is the seated pose
    /// on the papers laid.
    #[test]
    fn the_regulated_key_meets_its_targets_then_lands_within_one_paper() {
        let meets = SEAT_NOISE + 3.0 * resolution();
        for (name, bench, targets) in cases() {
            let r = bench.regulating();
            let outcome = bench.solve(targets);
            let Ok(TargetOutcome::Met {
                stacks,
                achieved,
                report,
            }) = outcome
            else {
                panic!("{name}: {outcome:?}");
            };
            assert!(report.converged, "{name}: {report:?}");
            let mut statics = Statics::new();
            let mut sink = SolveReport::NOTHING;
            let (balance, front) = (&stacks.balance, &stacks.front);
            let laid_balance = total(&balance.papers);
            let laid_front = total(&front.papers);

            let height = reading(r.height_trial(&mut statics, &mut sink, balance.asked));
            let dip = reading(r.dip_trial(&mut statics, &mut sink, laid_balance, front.asked));
            assert!(
                (height - targets.key_height).abs() <= meets,
                "{name}: the balance paper asked gives {height} m for {} m",
                targets.key_height
            );
            assert!(
                (dip - targets.key_dip).abs() <= 2.0 * meets,
                "{name}: the front paper asked gives {dip} m for {} m",
                targets.key_dip
            );

            for stack in [balance, front] {
                assert!(
                    stack.papers.iter().all(|p| PAPER_THICKNESSES.contains(p)),
                    "{name}: {:?}",
                    stack.papers
                );
                let laid = total(&stack.papers);
                assert!(
                    laid <= stack.asked + 1e-12 && stack.asked - laid < ONE_PAPER,
                    "{name}: {laid} m laid for {} m asked",
                    stack.asked
                );
            }
            let height_laid = reading(r.height_trial(&mut statics, &mut sink, laid_balance));
            let dip_laid = reading(r.dip_trial(&mut statics, &mut sink, laid_balance, laid_front));
            assert!(
                (achieved.key_height - height_laid).abs() <= meets
                    && (achieved.key_dip - dip_laid).abs() <= 2.0 * meets,
                "{name}: achieved {achieved:?}, the papers read {height_laid} m and {dip_laid} m"
            );
            let one_more =
                reading(r.height_trial(&mut statics, &mut sink, laid_balance + ONE_PAPER));
            assert!(
                achieved.key_height <= targets.key_height + meets && one_more > targets.key_height,
                "{name}: key height {} m, {one_more} m with one more paper, for {} m",
                achieved.key_height,
                targets.key_height
            );
            let one_more = reading(r.dip_trial(
                &mut statics,
                &mut sink,
                laid_balance,
                laid_front + ONE_PAPER,
            ));
            assert!(
                achieved.key_dip >= targets.key_dip - 2.0 * meets && one_more < targets.key_dip,
                "{name}: key dip {} m, {one_more} m with one more paper, for {} m",
                achieved.key_dip,
                targets.key_dip
            );
        }
    }

    /// Where the key bears on its cloth at the top seat of `seats`: its own
    /// point there, in its frame, the height of the cloth's point in the
    /// keybed's, and the attitude's sine, front-down.
    fn bearing_on_the_cloth(k: &OnItsFrame, seats: &Seats) -> (Point, f64, f64) {
        let m = &k.mounted.mechanism;
        let pose = m.pose(&seats.top.q);
        let mut rows = Rows::new();
        m.rows(&pose, f64::INFINITY, &mut rows);
        let pair = m
            .pairs()
            .iter()
            .position(|p| p.code == ContactCode::BackRailCloth)
            .expect("the cloth is a pair");
        let row = rows.rows()[rows
            .governing(u16::try_from(pair).unwrap())
            .expect("the key bears on its cloth")];
        let place = pose.placement(k.mounted.key);
        let w = row.on_a();
        let (dx, dy) = (w.x - place.origin.x, w.y - place.origin.y);
        let on_key = Point::new(
            place.cos * dx + place.sin * dy,
            -place.sin * dx + place.cos * dy,
        );
        (on_key, row.on_b().y, -place.sin)
    }

    /// Consistency, the independent route: **the solve agrees with an exact
    /// inverse of the key on a pivot.** The oracle shares none of the
    /// solve's machinery: no mechanism, no rows, no static solve, only the
    /// key's two conditions at rest and one at the bottom, each linear in
    /// the sine and cosine of the attitude, solved in closed form
    /// ([`crate::measure::oracle::sines_satisfying`]). It needs where the key
    /// bears on its cloth, which slides along the chamfer with the attitude,
    /// and takes it from the seat the solve's answer gives.
    ///
    /// Three comparisons, on every case: the balance bearing's height the
    /// oracle finds for the key height target, against the felt and the
    /// paper the solve asks for on the rail; the key height the oracle reads
    /// on the balance stack as laid, against the seated pose's; and the
    /// front punching's top the oracle finds for the dip on that stack,
    /// against the felt and the paper the solve asks for there.
    ///
    /// The bound is derived. The solve's own readings stand within
    /// [`SEAT_NOISE`] and the bisection's resolution through the lever of
    /// its targets; the contact the oracle is given was closed to within
    /// [`GAP_TOLERANCE`], which reaches the key front through a lever under
    /// two; a height at the key front is a stack at the rail through a lever
    /// over one; and the closed form rounds its last bits on heights of a
    /// tenth of a metre.
    #[test]
    fn the_solve_agrees_with_an_exact_inverse_of_the_key_on_a_pivot() {
        let bound = 2.0 * SEAT_NOISE
            + 4.0 * GAP_TOLERANCE
            + 3.0 * resolution()
            + 256.0 * f64::EPSILON * 0.1;
        for (name, bench, targets) in cases() {
            let (stacks, _) = met(bench.solve(targets));
            let f = &bench.frame;

            let (k, seats) = bench.seated(stacks.balance.asked, 0.0);
            let (bears, cloth, s) = bearing_on_the_cloth(&k, &seats);
            let bearing =
                oracle::pivot_bearing_for_height(k.front_top, bears, cloth, targets.key_height, s)
                    .expect("the rest has an attitude");
            let asked = f.balance_rail_height + f.balance_punching.felt + stacks.balance.asked;
            assert!(
                (bearing - asked).abs() <= bound,
                "{name}: the oracle bears at {bearing} m, the solve at {asked} m"
            );

            let laid = total(&stacks.balance.papers);
            let (k, seats) = bench.seated(laid, stacks.front.asked);
            let (bears, cloth, s) = bearing_on_the_cloth(&k, &seats);
            let bearing = f.balance_rail_height + f.balance_punching.felt + laid;
            let (_, top) = oracle::pivot_rest(k.front_top, bears, cloth, bearing, s)
                .expect("the rest has an attitude");
            let seated = seats.top.height(&k.mounted, k.front_top);
            assert!(
                (top - seated).abs() <= bound,
                "{name}: the oracle's key height {top} m, the seated pose's {seated} m"
            );
            let bottom = -libm::sin(seats.bottom.q[k.mounted.angle.0]);
            // The front punching's centre, along the key from its own pin.
            let x = f.sections.front.pin_x(f.front_rail, bench.kind)
                - f.sections.balance.pin_x(0.0, bench.kind);
            let punching = oracle::pivot_punching_for_dip(
                k.front_top,
                bearing,
                top,
                targets.key_dip,
                x,
                bottom,
            )
            .expect("the bottom has an attitude");
            let asked = f.front_rail_height + f.front_punching.felt + stacks.front.asked;
            assert!(
                (punching - asked).abs() <= bound,
                "{name}: the oracle's front punching at {punching} m, the solve's at {asked} m"
            );
        }
    }

    /// Consistency: **the small-angle placement the solve replaced misses by
    /// no more than the terms it drops.** It takes the key top as level at
    /// both seats, so it drops the key top's height times one less the
    /// cosine of the attitude; read back by the seated pose, its stacks miss
    /// each target by less than the key top's height times the square of
    /// the larger attitude's sine, twice the dropped term's first order.
    /// Given where the key bears on its cloth at the exact answer, as the
    /// seated pose finds it.
    #[test]
    fn the_small_angle_placement_misses_by_no_more_than_the_terms_it_drops() {
        let bench = Bench::standard();
        let r = bench.regulating();
        let (stacks, _) = met(bench.solve(TARGETS));
        let (k, seats) = bench.seated(stacks.balance.asked, 0.0);
        let (bears, _, s_top) = bearing_on_the_cloth(&k, &seats);
        let (balance, front) =
            oracle::small_angle_placement(&bench.key, &bench.frame, TARGETS, bears)
                .expect("the placement has an answer");
        let f = &bench.frame;
        let (balance, front) = (
            balance - f.balance_punching.felt,
            front - f.front_punching.felt,
        );
        let mut statics = Statics::new();
        let mut sink = SolveReport::NOTHING;
        let height = reading(r.height_trial(&mut statics, &mut sink, balance));
        let dip = reading(r.dip_trial(&mut statics, &mut sink, balance, front));
        let (k, at) = bench.seated(balance, front);
        let s_bottom = -libm::sin(at.bottom.q[k.mounted.angle.0]);
        let s = s_top.abs().max(s_bottom.abs());
        let dropped = s * s * (bench.key.stick_height + bench.key.keytop_thickness);
        for (what, read, target) in [
            ("key height", height, TARGETS.key_height),
            ("key dip", dip, TARGETS.key_dip),
        ] {
            assert!(
                (read - target).abs() < dropped,
                "the small-angle {what} misses by {} m, past the {dropped} m it drops",
                read - target
            );
        }
    }

    /// The deepest dip the key has on the bench's balance stack: no paper on
    /// the front punching, read by the seated pose, with what holds the
    /// bottom.
    fn deepest(bench: &Bench, balance: f64) -> (f64, Vec<ContactCode>) {
        let (k, seats) = bench.seated(balance, 0.0);
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        (top - bottom, seats.bottom.held_by)
    }

    /// Consistency, the audit's RUST-10 and PHY-9: **a dip the front pin
    /// forbids is refused, with its shortfall**, never laid as the nearest
    /// stack and shown as a miss.
    ///
    /// On the bench, 12 mm of dip at 65 mm of key height is past where the
    /// pin bottoms in its mortise: refused, naming the pin, short by the
    /// target less the deepest dip the key has, which the seated pose reads
    /// with no paper on the front punching, the pin holding the bottom; the
    /// same bits, since it is the same reading. With thin felts, so that
    /// 62.5 mm is a key height the balance stack reaches, every dip from 9.0
    /// to 10.5 mm is refused by the same pin, each short by its own target
    /// less one deepest dip. And a key cut to 230 mm on a frame placed under
    /// it, felts bare, refuses 8 mm at 60 mm, by the pin, short by 8 mm less
    /// the deepest dip the pin leaves it on the balance stack 60 mm lays,
    /// about 2.16 mm.
    #[test]
    fn a_dip_the_front_pin_forbids_is_refused_with_its_shortfall() {
        let bench = Bench::standard();
        let laid = total(&met(bench.solve(TARGETS)).0.balance.papers);
        let (deep, held_by) = deepest(&bench, laid);
        assert_eq!(held_by, vec![ContactCode::FrontPinInTheMortise]);
        let asked = Targets {
            key_dip: 0.012,
            ..TARGETS
        };
        assert_eq!(
            bench.solve(asked),
            Ok(TargetOutcome::Infeasible {
                target: Target::KeyDip,
                limited_by: Limit::FrontPinInMortise,
                shortfall_m: 0.012 - deep,
            })
        );

        let mut thin = Bench::standard();
        thin.frame.balance_punching.felt = 0.0005;
        thin.frame.front_punching.felt = 0.0005;
        let mut shortfalls = Vec::new();
        for dip in [0.0090, 0.0100, 0.0102, 0.0105] {
            match thin.solve(Targets {
                key_height: 0.0625,
                key_dip: dip,
            }) {
                Ok(TargetOutcome::Infeasible {
                    target: Target::KeyDip,
                    limited_by: Limit::FrontPinInMortise,
                    shortfall_m,
                }) => shortfalls.push(dip - shortfall_m),
                other => panic!("62.5 mm with {dip} m of dip: {other:?}"),
            }
        }
        let reach = shortfalls[0];
        assert!(
            shortfalls.iter().all(|r| (r - reach).abs() < 1e-15),
            "one deepest dip for every target: {shortfalls:?}"
        );

        let key = KeyDesign {
            front_edge: 0.230,
            ..KeyDesign::default()
        };
        let mut cut = Bench {
            frame: FrameDesign::default().placed_under(&key),
            key,
            ..Bench::standard()
        };
        cut.frame.balance_punching.felt = 0.0;
        cut.frame.front_punching.felt = 0.0;
        let asked = Targets {
            key_height: 0.060,
            key_dip: 0.008,
        };
        let mut sink = SolveReport::NOTHING;
        let Ok(Bisected::Asked(balance)) = cut.regulating().bisect(
            &mut Statics::new(),
            &mut sink,
            Searched::Balance,
            asked.key_height,
        ) else {
            panic!("60 mm of key height is reachable on the cut key");
        };
        let (deep, held_by) = deepest(&cut, total(&papers_for(balance)));
        assert!(held_by.contains(&ContactCode::FrontPinInTheMortise));
        assert_eq!(
            cut.solve(asked),
            Ok(TargetOutcome::Infeasible {
                target: Target::KeyDip,
                limited_by: Limit::FrontPinInMortise,
                shortfall_m: 0.008 - deep,
            })
        );
    }

    /// Consistency: **every target out of reach names what keeps it out**,
    /// and how far, in its own reading.
    ///
    /// - 62.5 mm of key height on the bench's 1.5 mm balance felt: the felt
    ///   alone stands the key higher, by what the seated pose reads with no
    ///   paper on it;
    /// - 10.2 mm of dip on a 3 mm front felt: the felt alone, no paper under
    ///   it, stops the key short;
    /// - 120 mm of key height: past the highest the key stands on its
    ///   balance pin, the paper raising its punching to the pin's top, short
    ///   by what the seated pose reads there; on a pin 50 mm tall, more than
    ///   the search lays under the balance punching;
    /// - the front rail raised 10 mm, its pin pulled: at 65 mm the front of
    ///   the key would rest on the front punching's felt with its tail on
    ///   the cloth, the stops fighting; with the pin in place, the pin
    ///   bottoming in its mortise is what fights;
    /// - a key 18 mm shorter under its frame: the front pin stands under its
    ///   wood, and only a key raised clear of the pin has a seat;
    /// - a front pin 0.3 mm into the wood in front of its slot, its top well
    ///   up in the slot: lowered onto its frame, the key rides on the pin
    ///   until the balance stack raises it clear, well above the target;
    /// - the back rail cloth holding the tail 40 mm above the balance
    ///   bearing, on a balance pin 50 mm tall: the key rides on its cloth and
    ///   its front pin with all the paper the search lays, its front below
    ///   the target, which only more paper would reach; on the bench's pin it
    ///   rides past its pin's top, off its frame, and has no seat at any
    ///   stack;
    /// - a front punching whose centre stands past the key's front: no dip
    ///   at all, the whole target short.
    #[test]
    fn every_target_out_of_reach_names_what_keeps_it_out() {
        let refused = |bench: &Bench, targets: Targets| match bench.solve(targets) {
            Ok(TargetOutcome::Infeasible {
                target,
                limited_by,
                shortfall_m,
            }) => {
                assert!(shortfall_m > 0.0 && shortfall_m.is_finite());
                (target, limited_by, shortfall_m)
            }
            other => panic!("{targets:?}: {other:?}"),
        };

        let bench = Bench::standard();
        let low = Targets {
            key_height: 0.0625,
            ..TARGETS
        };
        let r = bench.regulating();
        let mut sink = SolveReport::NOTHING;
        let bare = reading(r.height_trial(&mut Statics::new(), &mut sink, 0.0));
        assert_eq!(
            refused(&bench, low),
            (
                Target::KeyHeight,
                Limit::FeltThickerThanTheStack,
                bare - 0.0625
            )
        );

        let mut felted = Bench::standard();
        felted.frame.front_punching.felt = 0.003;
        let (target, limit, _) = refused(&felted, TARGETS);
        assert_eq!(
            (target, limit),
            (Target::KeyDip, Limit::FeltThickerThanTheStack)
        );

        let tall = Targets {
            key_height: 0.120,
            ..TARGETS
        };
        // The paper that brings the punching's top to the pin's: a
        // nanometre less, and the key stands on its pin as high as any
        // stack has it there.
        let f = &bench.frame;
        let to_the_top = f.sections.balance.pin_height - f.balance_punching.felt;
        let highest = reading(r.height_trial(&mut Statics::new(), &mut sink, to_the_top - 1e-9));
        let (target, limit, short) = refused(&bench, tall);
        assert_eq!(
            (target, limit),
            (Target::KeyHeight, Limit::OffItsBalancePin)
        );
        // Within the nanometre and the bisection's resolution through the
        // stack's lever on key height, under three, and the seats' noise.
        let bound = 3.0 * (1e-9 + resolution()) + SEAT_NOISE;
        assert!(
            (short - (0.120 - highest)).abs() <= bound,
            "short by {short} m, the key stands {highest} m high at its pin's top"
        );
        let mut pinned = Bench::standard();
        pinned.frame.sections.balance.pin_height = 0.050;
        let (target, limit, short) = refused(&pinned, tall);
        assert_eq!((target, limit), (Target::KeyHeight, Limit::RailTooLow));
        assert!(short < 0.120);

        let mut raised = Bench::standard();
        raised.frame.front_rail_height += 0.010;
        let (target, limit, _) = refused(&raised, TARGETS);
        assert_eq!(
            (target, limit),
            (Target::KeyHeight, Limit::FrontPinInMortise)
        );
        raised.frame.sections.front.pin = None;
        let (target, limit, _) = refused(&raised, TARGETS);
        assert_eq!((target, limit), (Target::KeyHeight, Limit::StopsFight));

        let mut short = Bench::standard();
        short.key.front_edge -= 0.018;
        let (target, limit, _) = refused(&short, TARGETS);
        assert_eq!(
            (target, limit),
            (Target::KeyHeight, Limit::FrontPinOnTheWood)
        );

        let mut into_the_wood = Bench::standard();
        let pin = into_the_wood
            .frame
            .sections
            .front
            .pin
            .expect("a pin")
            .length();
        let (_, front_end) = into_the_wood.key.front_mortise_span();
        into_the_wood.frame.front_rail = front_end - 0.5 * pin + 0.0003;
        let (target, limit, _) = refused(&into_the_wood, TARGETS);
        assert_eq!(
            (target, limit),
            (Target::KeyHeight, Limit::FrontPinOnTheWood)
        );

        let mut wedged = Bench::standard();
        let f = &mut wedged.frame;
        f.back_rail_height = f.balance_bearing_height() + 0.040 - f.back_rail_cloth;
        assert!(
            matches!(
                wedged.solve(TARGETS),
                Err(Unsolvable::NoSeat(SeatRefusal::LiftsOffItsFrame { .. }))
            ),
            "{:?}",
            wedged.solve(TARGETS)
        );
        wedged.frame.sections.balance.pin_height = 0.050;
        let r = wedged.regulating();
        let mut sink = SolveReport::NOTHING;
        let rides = match r.height_trial(&mut Statics::new(), &mut sink, STACK_SEARCH) {
            Ok(Trial::Rides { reading, .. }) => reading,
            _ => panic!("the key rides with all the paper the search lays"),
        };
        assert!(rides < TARGETS.key_height);
        assert_eq!(
            refused(&wedged, TARGETS),
            (
                Target::KeyHeight,
                Limit::RailTooLow,
                TARGETS.key_height - rides
            )
        );

        let mut past = Bench::standard();
        past.frame.front_rail = past.key.front_edge + 0.0002;
        past.frame.sections.front.pin = None;
        assert_eq!(
            refused(&past, TARGETS),
            (Target::KeyDip, Limit::NoStopBelowTheFront, TARGETS.key_dip)
        );
    }

    /// Consistency: **stacks that stand the key off its balance pin steer
    /// the search and are never laid.** The bench's balance pin cut to
    /// 5.5 mm above its rail leaves the key 3.3 mm of lift on its punching
    /// before it leaves the pin: the search's balance stacks past 4 mm raise
    /// the punching past the pin's top, and its front stack of 20 mm makes
    /// the key ride 4.9 mm up, past it too. Each such trial is refused as
    /// off its pin and counts as one past the key height target, or as one
    /// whose stops fight for the dip, as the same trials on the bench's
    /// taller pin count, seated past the target or riding: so the papers
    /// laid and what they give are the bench's own, to the bit. 75 mm of key
    /// height, which only a punching past the cut pin's top would reach, is
    /// refused as the pin's doing.
    #[test]
    fn stacks_that_stand_the_key_off_its_pin_steer_the_search_and_are_never_laid() {
        let bench = Bench::standard();
        let mut cut = Bench::standard();
        cut.frame.sections.balance.pin_height = 0.0055;
        let (stacks, achieved) = met(bench.solve(TARGETS));
        let (cut_stacks, cut_achieved) = met(cut.solve(TARGETS));
        assert_eq!(cut_stacks, stacks);
        assert_eq!(
            cut_achieved.key_height.to_bits(),
            achieved.key_height.to_bits()
        );
        assert_eq!(cut_achieved.key_dip.to_bits(), achieved.key_dip.to_bits());

        let r = cut.regulating();
        let mut sink = SolveReport::NOTHING;
        let mut statics = Statics::new();
        let laid = total(&stacks.balance.papers);
        for trial in [
            r.height_trial(&mut statics, &mut sink, 0.005),
            r.dip_trial(&mut statics, &mut sink, laid, STACK_SEARCH),
        ] {
            assert!(
                matches!(trial, Ok(Trial::OffItsPin { .. })),
                "a trial past the cut pin's top is read as seated"
            );
        }
        match cut.solve(Targets {
            key_height: 0.075,
            ..TARGETS
        }) {
            Ok(TargetOutcome::Infeasible {
                target: Target::KeyHeight,
                limited_by: Limit::OffItsBalancePin,
                shortfall_m,
            }) => assert!(shortfall_m > 0.0 && shortfall_m < 0.01, "{shortfall_m} m"),
            other => panic!("75 mm of key height on the cut pin: {other:?}"),
        }
    }

    /// A target that is not a number is refused before anything is solved.
    #[test]
    fn a_target_that_is_not_a_number_is_refused() {
        let bench = Bench::standard();
        for (targets, which) in [
            (
                Targets {
                    key_height: f64::NAN,
                    ..TARGETS
                },
                Target::KeyHeight,
            ),
            (
                Targets {
                    key_dip: f64::INFINITY,
                    ..TARGETS
                },
                Target::KeyDip,
            ),
        ] {
            assert_eq!(bench.solve(targets), Err(Unsolvable::NotFinite(which)));
        }
    }

    /// Determinism: the same targets on the same key give the same bits.
    #[test]
    fn the_same_targets_give_the_same_bits() {
        let bench = Bench::standard();
        let (one, two) = (bench.solve(TARGETS), bench.solve(TARGETS));
        assert_eq!(one, two);
        let (a, b) = (met(one), met(two));
        assert_eq!(a.0.balance.asked.to_bits(), b.0.balance.asked.to_bits());
        assert_eq!(a.1.key_dip.to_bits(), b.1.key_dip.to_bits());
    }
}
