//! What crosses to an interface: the bench's settings, its weighing sheet,
//! its diagnostics, where its bodies stand and what it draws, each a document
//! of its own, in codes and numbers.
//!
//! # Codes and numbers, never sentences
//!
//! An interface words what it shows in its reader's language, and wording
//! built here could not be translated. So a refusal, a fault or a verdict
//! crosses as a code with its parameters,
//! `{"code":"pin_outside_mortise","by_mm":16.5}`, the parameters in the
//! trade's units with the unit in their names, and the interface words it
//! from one catalogue per language (`apps/bench/src/i18n/`). A code carries
//! every number its wording needs, the bench's own limits included, so no
//! wording of a code holds a number of the engine's.
//!
//! The codes mirror the domain's enums in `pa-core` through `match`es with
//! no catch-all arm: a refusal added there and not mapped here is a compile
//! error, and one mapped here and not worded there fails the interface's
//! type-check.
//!
//! # Five documents, apart
//!
//! [`Settings`] are always valid, whatever the frame does: every control
//! reads its value from them, so a frame that cannot be weighed still shows
//! every control and the way back out of it. [`Sheet`] carries every reading,
//! each read or refused with its reason, also always: a frame that cannot
//! seat its key reads its jig and refuses the rest line by line.
//! [`Diagnostics`] say how the key stands and why, [`State`] where each body
//! stands, and [`Scene`] what is drawn, each part in its own body's frame.
//!
//! # Generated types
//!
//! The TypeScript types of these documents are generated from the Rust ones
//! by `ts-rs`, in a test of this module, into `crates/pa-wasm/types/`, which is
//! committed and never edited by hand: the test fails when the committed
//! files differ from what the types generate, and writes the new ones.

use serde::Serialize;
#[cfg(test)]
use ts_rs::TS;

use pa_core::contact::{
    Bearing, ContactCode, HowItBears, Material as CoreMaterial, PartCode, Provenance,
    Role as CoreRole, Sweep as CoreSweep,
};
use pa_core::measure::jig::JigRefusal;
use pa_core::measure::measurement::{
    Input as CoreInput, Measurement, Mount as CoreMount, ProcedureId, Rounding as CoreRounding,
    Trust as CoreTrust, Unit as CoreUnit, Zeroed,
};
use pa_core::measure::seat::SeatRefusal;
use pa_core::measure::walk::{WalkRefusal, CAP, STEP_G};
use pa_core::measure::weighing::{BushingsRefusal, Refused, FRICTION_SEARCH_G};
use pa_core::parts::faults::Fault as CoreFault;
use pa_core::parts::mounts::KeyMount;
use pa_core::parts::{KeyMass, NotAWeighing};
use pa_core::regulation::{
    Limit as CoreLimit, Target as CoreTarget, Targets as CoreTargets, Unsolvable, STACK_SEARCH,
};
use pa_core::G;

use crate::key_bench::{
    ContactState as CoreContactState, KeyBench, NotRegulated, Riding as CoreRiding, SeatReport,
    TouchVerdict, Which, KEY, PAPER_THICKNESSES,
};

/// A document as the interface reads it: JSON, the shape of its generated
/// TypeScript type.
pub fn json<T: Serialize>(document: &T) -> String {
    // Every document is a tree of structs, enums, numbers and strings, whose
    // serialization cannot fail; a number that is not finite is written
    // `null`, as JSON has no other way to say it.
    serde_json::to_string(document).unwrap_or_else(|_| "null".to_owned())
}

/// Metres to millimetres.
fn mm(metres: f64) -> f64 {
    metres * 1000.0
}

/// Metres to millimetres, narrowed for drawing.
fn mm32(metres: f64) -> f32 {
    (metres * 1000.0) as f32
}

// ---- codes shared by the documents -----------------------------------------

/// A contact between the key and its frame, named by its two parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Contact {
    /// The key's underside on the balance rail punching: what it bears on at
    /// its pin.
    BalanceRailPunching,
    /// The key's underside on the front rail punching: what it comes down on
    /// at the bottom of its dip.
    FrontRailPunching,
    /// The key's underside on the back rail cloth: what it rests on.
    BackRailCloth,
    /// The front rail pin's top against the top of the key's front mortise.
    FrontPinInTheMortise,
    /// The front rail pin's top against the wood of the key round its front
    /// mortise.
    FrontPinOnTheWood,
}

impl From<ContactCode> for Contact {
    fn from(code: ContactCode) -> Self {
        match code {
            ContactCode::BalanceRailPunching => Self::BalanceRailPunching,
            ContactCode::FrontRailPunching => Self::FrontRailPunching,
            ContactCode::BackRailCloth => Self::BackRailCloth,
            ContactCode::FrontPinInTheMortise => Self::FrontPinInTheMortise,
            ContactCode::FrontPinOnTheWood => Self::FrontPinOnTheWood,
        }
    }
}

/// The contacts `codes`, in their order.
fn contacts(codes: &[ContactCode]) -> Vec<Contact> {
    codes.iter().map(|&c| c.into()).collect()
}

/// A part of the key or of its frame, as the trade names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum Part {
    Keystick,
    Keytop,
    KeyFront,
    KeyButton,
    BackCheckBlock,
    KeyTailFelt,
    Capstan,
    FrontMortise,
    BalanceRail,
    BalanceRailPin,
    BalanceRailPunching,
    FrontRail,
    FrontRailPin,
    FrontRailPunching,
    PaperPunching,
    BackRail,
    BackRailCloth,
}

impl From<PartCode> for Part {
    fn from(code: PartCode) -> Self {
        match code {
            PartCode::Keystick => Self::Keystick,
            PartCode::Keytop => Self::Keytop,
            PartCode::KeyFront => Self::KeyFront,
            PartCode::KeyButton => Self::KeyButton,
            PartCode::BackCheckBlock => Self::BackCheckBlock,
            PartCode::KeyTailFelt => Self::KeyTailFelt,
            PartCode::Capstan => Self::Capstan,
            PartCode::FrontMortise => Self::FrontMortise,
            PartCode::BalanceRail => Self::BalanceRail,
            PartCode::BalanceRailPin => Self::BalanceRailPin,
            PartCode::BalanceRailPunching => Self::BalanceRailPunching,
            PartCode::FrontRail => Self::FrontRail,
            PartCode::FrontRailPin => Self::FrontRailPin,
            PartCode::FrontRailPunching => Self::FrontRailPunching,
            PartCode::PaperPunching => Self::PaperPunching,
            PartCode::BackRail => Self::BackRail,
            PartCode::BackRailCloth => Self::BackRailCloth,
        }
    }
}

/// A reading of the weighing sheet, by the procedure that takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Procedure {
    /// Front weight, on Stanwood's levelled jig.
    FrontWeight,
    /// Key weight ratio, force convention, on the tared jig.
    KeyWeightRatio,
    /// Key height, the key seated at the top of its stroke.
    KeyHeight,
    /// Key dip, from the top seat to the bottom one.
    KeyDip,
    /// Down weight, by the breakaway walk from the top seat.
    DownWeight,
    /// Up weight, by the breakaway walk from the bottom seat.
    UpWeight,
    /// Balance weight, from the down and up weights as read.
    BalanceWeight,
    /// Friction weight, from the down and up weights as read.
    FrictionWeight,
    /// Key friction weight by Stanwood's protocol, zeroed on the levelled
    /// jig.
    KeyFrictionWeightZeroedOnTheJig,
    /// Key friction weight zeroed where the key rests on its frame.
    KeyFrictionWeightZeroedOnTheFrame,
}

impl From<ProcedureId> for Procedure {
    fn from(p: ProcedureId) -> Self {
        match p {
            ProcedureId::FrontWeight => Self::FrontWeight,
            ProcedureId::KeyWeightRatio => Self::KeyWeightRatio,
            ProcedureId::KeyHeight => Self::KeyHeight,
            ProcedureId::KeyDip => Self::KeyDip,
            ProcedureId::DownWeight => Self::DownWeight,
            ProcedureId::UpWeight => Self::UpWeight,
            ProcedureId::BalanceWeight => Self::BalanceWeight,
            ProcedureId::FrictionWeight => Self::FrictionWeight,
            ProcedureId::KeyFrictionWeightZeroedOnTheJig => Self::KeyFrictionWeightZeroedOnTheJig,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame => {
                Self::KeyFrictionWeightZeroedOnTheFrame
            }
        }
    }
}

/// How the key rides on its stops, off its balance punching, where they
/// fight.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Riding {
    /// The front stop it rides on, its back rail cloth the other.
    pub front: Contact,
    /// How far above its balance punching it stands, millimetres.
    pub lift_mm: f64,
}

impl From<CoreRiding> for Riding {
    fn from(r: CoreRiding) -> Self {
        Self {
            front: r.front.into(),
            lift_mm: mm(r.lift),
        }
    }
}

// ---- refusals --------------------------------------------------------------

/// Which static solve stopped on its cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Solve {
    /// Seating the key at the top or the bottom of its stroke.
    Seat,
    /// A step of a breakaway walk, on the key's way down or up.
    Walk,
}

/// Why a reading cannot be taken, or why the key cannot be seated on its
/// frame.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Refusal {
    /// The key or its frame cannot be mounted as given.
    GeometryDegenerate,
    /// The reading is of the key's own weight, and nobody has weighed it.
    KeyNotWeighed,
    /// The measuring point stands at or behind the balance pin, where the
    /// jig's scale cannot hold the key level.
    ScaleNotInFrontOfThePivot,
    /// The key has no seat: these stops stand in its drawn parts, and no pose
    /// of the key on its frame clears them.
    CannotSeat {
        /// The stops, in the order the key's contacts are declared.
        fighting: Vec<Contact>,
    },
    /// The key would stand off its balance pin, and off its frame.
    LiftsOffItsFrame {
        /// How far above its balance punching the key leaves its pin,
        /// millimetres: negative where the punching stands above the pin's
        /// top, by that much.
        leaves_its_pin_mm: f64,
    },
    /// The front punching's centre stands off the key's underside: nothing
    /// stops the front of the key at the bottom.
    NoStopBelowTheFront,
    /// The back rail cloth stands where the key's tail cannot reach it.
    NoStopUnderTheTail,
    /// The capstan stands at or in front of the balance bearing.
    CapstanNotBehindTheBearing,
    /// The measuring point stands at or behind the balance bearing.
    MeasuringPointNotInFront,
    /// A static solve reached its cap of steps.
    NotConverged {
        /// Which.
        solve: Solve,
        /// The steps it took.
        iterations: u32,
        /// The stationarity residual it left, newtons.
        residual_n: f64,
    },
    /// The key has no stroke between its two seats.
    NoTravel {
        /// How it rides on its stops, where that is why; `null` where it
        /// stands at the bottom of its dip at rest.
        riding: Option<Riding>,
    },
    /// A weight on the measuring point does not turn the key.
    NoLever,
    /// The walk took its cap of weights and the key never reached the other
    /// seat.
    DoesNotReachTheEndOfItsStroke {
        /// The most the walk puts on the measuring point, grams.
        cap_g: f64,
    },
    /// A step of the walk found no equilibrium: the key's stops fight.
    NoEquilibrium {
        /// The stops that fight; empty where the key leans on a limit.
        on: Vec<Contact>,
    },
    /// A weight the walk put on the measuring point lifts the key off its
    /// frame.
    WalkLiftsOffItsFrame,
    /// No top of the key stands where the temporary weight goes.
    NoKeyTopAtTheStation {
        /// Where it goes, millimetres from the balance pin, positive in
        /// front.
        station_mm: f64,
    },
    /// It is computed from a reading that is refused.
    FromARefusedReading {
        /// That reading.
        reading: Procedure,
    },
}

impl Refusal {
    /// Why the key cannot be seated.
    pub fn seat(why: &SeatRefusal) -> Self {
        match why {
            SeatRefusal::Mechanism(_) => Self::GeometryDegenerate,
            SeatRefusal::CapstanNotBehindTheBearing => Self::CapstanNotBehindTheBearing,
            SeatRefusal::MeasuringPointNotInFront => Self::MeasuringPointNotInFront,
            SeatRefusal::CannotSeat { fighting } => Self::CannotSeat {
                fighting: contacts(fighting),
            },
            SeatRefusal::LiftsOffItsFrame { leaves_its_pin } => Self::LiftsOffItsFrame {
                leaves_its_pin_mm: mm(*leaves_its_pin),
            },
            SeatRefusal::NoStopUnderTheTail => Self::NoStopUnderTheTail,
            SeatRefusal::NoStopBelowTheFront => Self::NoStopBelowTheFront,
            SeatRefusal::NotConverged {
                iterations,
                residual,
            } => Self::NotConverged {
                solve: Solve::Seat,
                iterations: *iterations,
                residual_n: *residual,
            },
        }
    }

    /// Why a reading is refused, `riding` saying how the key rides on its
    /// stops where it does: a key with no stroke is told why.
    pub fn reading(why: &Refused, riding: Option<Riding>) -> Self {
        match why {
            Refused::Mechanism(_) => Self::GeometryDegenerate,
            Refused::KeyNotWeighed => Self::KeyNotWeighed,
            Refused::Jig(jig) => match jig {
                JigRefusal::KeyNotWeighed => Self::KeyNotWeighed,
                JigRefusal::ScaleNotInFrontOfThePivot => Self::ScaleNotInFrontOfThePivot,
                JigRefusal::Mechanism(_) => Self::GeometryDegenerate,
            },
            Refused::Seat(seat) => Self::seat(seat),
            Refused::Walk(walk) => match walk {
                WalkRefusal::NoTravel => Self::NoTravel { riding },
                WalkRefusal::NoLever => Self::NoLever,
                WalkRefusal::DoesNotReachTheEndOfItsStroke => Self::DoesNotReachTheEndOfItsStroke {
                    cap_g: f64::from(CAP) * STEP_G,
                },
                WalkRefusal::NoEquilibrium { on } => Self::NoEquilibrium { on: contacts(on) },
                WalkRefusal::LiftsOffItsFrame => Self::WalkLiftsOffItsFrame,
                WalkRefusal::NotConverged {
                    iterations,
                    residual,
                } => Self::NotConverged {
                    solve: Solve::Walk,
                    iterations: *iterations,
                    residual_n: *residual,
                },
                WalkRefusal::Mechanism(_) => Self::GeometryDegenerate,
            },
            Refused::NoKeyTopAtTheStation { station_m } => Self::NoKeyTopAtTheStation {
                station_mm: mm(*station_m),
            },
            Refused::From { reading } => Self::FromARefusedReading {
                reading: (*reading).into(),
            },
        }
    }
}

// ---- settings --------------------------------------------------------------

/// Every setting of the bench, in the trade's units: always valid, whatever
/// the frame does, so every control reads its value here.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Settings {
    /// How far back from the covering's lip every weight is centred,
    /// millimetres: the measuring point of every touchweight.
    pub measuring_point_from_lip_mm: f64,
    /// The weight on the measuring point, grams: negative pulls up.
    pub weight_on_the_measuring_point_g: f64,
    /// The regulation's targets.
    pub targets: TargetSettings,
    /// The key.
    pub key: KeySettings,
    /// The frame it sits on.
    pub frame: FrameSettings,
    /// The papers on the two punchings, and the drawer's.
    pub papers: PaperSettings,
}

/// The targets of the regulation last laid.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct TargetSettings {
    /// Key height, millimetres.
    pub key_height_mm: f64,
    /// Key dip, millimetres.
    pub key_dip_mm: f64,
}

/// The key.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct KeySettings {
    /// Its front edge, millimetres in front of the balance pin.
    pub front_edge_mm: f64,
    /// Its capstan, millimetres behind the balance pin.
    pub capstan_behind_mm: f64,
    /// How far the capstan stands above the keystick, millimetres.
    pub capstan_height_mm: f64,
    /// The front mortise's depth above the underside, millimetres.
    pub front_mortise_depth_mm: f64,
    /// Its mass, grams: as weighed, or derived from its volume until it is;
    /// `null` where the volume model encloses nothing.
    pub mass_g: Option<f64>,
    /// Its weighing as it was typed; `null` while its mass is derived from
    /// its volume.
    pub weighed: Option<Weighed>,
    /// Its bushings' friction as stored: the torque they hold, gram-force
    /// millimetres. What Stanwood's protocol reads them as is a reading, on
    /// the sheet.
    pub friction_torque_gmm: f64,
}

/// A key weighed whole, as it was typed.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Weighed {
    /// Its mass on the scale, grams.
    pub mass_g: f64,
    /// Its front weight as the jig's scale read it, grams.
    pub front_weight_g: f64,
    /// How far back from the covering's lip the roller stood, millimetres.
    pub from_lip_mm: f64,
}

/// The frame's own dimensions, which the key does not move.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FrameSettings {
    /// The bare balance rail's height above the keybed, millimetres.
    pub balance_rail_mm: f64,
    /// The bare front rail's height, millimetres.
    pub front_rail_mm: f64,
    /// The bare back rail's height, millimetres.
    pub back_rail_mm: f64,
    /// Where the front rail stands, millimetres in front of the balance pin.
    pub front_rail_at_mm: f64,
    /// Where the back rail stands, millimetres behind the balance pin.
    pub back_rail_behind_mm: f64,
    /// How far the front rail pin stands above the bare rail, millimetres.
    pub front_pin_mm: f64,
    /// The back rail cloth's thickness, millimetres.
    pub back_rail_cloth_mm: f64,
    /// The felt washer under the balance punching's papers, millimetres.
    pub balance_felt_mm: f64,
    /// The felt washer on the front punching, millimetres.
    pub front_felt_mm: f64,
}

/// The papers on the two punchings: the coordinate regulation sets.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct PaperSettings {
    /// Under the balance punching, millimetres, in the order they were laid.
    pub balance_mm: Vec<f64>,
    /// On the front punching, millimetres, in the order they were laid.
    pub front_mm: Vec<f64>,
    /// The balance stack's total, millimetres, as the engine sums it.
    pub balance_total_mm: f64,
    /// The front stack's total, millimetres, as the engine sums it.
    pub front_total_mm: f64,
    /// The thicknesses the drawer holds, millimetres.
    pub drawer_mm: Vec<f64>,
}

impl Settings {
    /// The settings of `bench`.
    pub fn of(bench: &KeyBench) -> Self {
        let key = bench.key();
        let frame = bench.frame();
        let targets = bench.targets();
        let (balance_total, front_total) = bench.papers();
        let friction = bench.friction();
        Self {
            measuring_point_from_lip_mm: mm(bench.measuring_point_from_lip()),
            weight_on_the_measuring_point_g: bench.finger_weight(),
            targets: TargetSettings {
                key_height_mm: mm(targets.key_height),
                key_dip_mm: mm(targets.key_dip),
            },
            key: KeySettings {
                front_edge_mm: mm(key.front_edge),
                capstan_behind_mm: mm(-key.capstan),
                capstan_height_mm: mm(key.capstan_screw.height),
                front_mortise_depth_mm: mm(key.front_mortise_depth),
                mass_g: bench.key_mass_g(),
                weighed: match bench.key_mass() {
                    KeyMass::Estimated => None,
                    KeyMass::Weighed { mass, datum, .. } => Some(Weighed {
                        mass_g: mass * 1000.0,
                        front_weight_g: datum.grams,
                        from_lip_mm: mm(datum.from_lip_m),
                    }),
                },
                // Newton-metres to gram-force millimetres: over standard
                // gravity to kilogram-force metres, then a thousand twice.
                friction_torque_gmm: friction.bound_n_m / G * 1.0e6,
            },
            frame: FrameSettings {
                balance_rail_mm: mm(frame.balance_rail_height),
                front_rail_mm: mm(frame.front_rail_height),
                back_rail_mm: mm(frame.back_rail_height),
                front_rail_at_mm: mm(frame.front_rail),
                back_rail_behind_mm: mm(-frame.back_rail),
                front_pin_mm: mm(frame.sections.front.pin_height),
                back_rail_cloth_mm: mm(frame.back_rail_cloth),
                balance_felt_mm: mm(frame.balance_punching.felt),
                front_felt_mm: mm(frame.front_punching.felt),
            },
            papers: PaperSettings {
                balance_mm: bench
                    .papers_on(Which::Balance)
                    .iter()
                    .map(|&p| mm(p))
                    .collect(),
                front_mm: bench
                    .papers_on(Which::Front)
                    .iter()
                    .map(|&p| mm(p))
                    .collect(),
                balance_total_mm: mm(balance_total),
                front_total_mm: mm(front_total),
                drawer_mm: PAPER_THICKNESSES.iter().map(|&p| mm(p)).collect(),
            },
        }
    }
}

// ---- the sheet -------------------------------------------------------------

/// The unit a reading is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub enum Unit {
    /// Grams at the measuring point.
    #[serde(rename = "g")]
    Gram,
    /// Millimetres.
    #[serde(rename = "mm")]
    Millimetre,
    /// A ratio, without a unit.
    #[serde(rename = "ratio")]
    Ratio,
}

/// How the trade reads a value to its resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Rounding {
    /// The least whole number of steps that does it: down weight and key
    /// friction weight.
    Up,
    /// The greatest whole number of steps: up weight.
    Down,
    /// To the nearest step, as a scale reads: front weight, the key weight
    /// ratio.
    Nearest,
    /// Computed from other readings as they were read: balance and friction
    /// weight.
    FromReadings,
    /// Not rounded: key height and key dip.
    None,
}

impl From<CoreRounding> for Rounding {
    fn from(r: CoreRounding) -> Self {
        match r {
            CoreRounding::Up => Self::Up,
            CoreRounding::Down => Self::Down,
            CoreRounding::Nearest => Self::Nearest,
            CoreRounding::FromReadings => Self::FromReadings,
            CoreRounding::None => Self::None,
        }
    }
}

/// An input a procedure reads, whose source decides whether a reading is a
/// measurement or an indication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Input {
    /// The key's mass and its static moment about the balance pin.
    KeyMass,
    /// The height of the key's centre of mass.
    CentreOfMassHeight,
    /// The friction of the key's bushings.
    KeyFriction,
    /// The dimensions of the key and its frame.
    Geometry,
}

impl From<CoreInput> for Input {
    fn from(i: CoreInput) -> Self {
        match i {
            CoreInput::KeyMass => Self::KeyMass,
            CoreInput::CentreOfMassHeight => Self::CentreOfMassHeight,
            CoreInput::KeyFriction => Self::KeyFriction,
            CoreInput::Geometry => Self::Geometry,
        }
    }
}

/// How far a reading can be trusted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Trust {
    /// Every input it reads is measured, published or set by the bench's
    /// user.
    Metrological,
    /// Some input it reads is estimated or assumed.
    Indicative {
        /// Those inputs, in the order the procedure reads them.
        because: Vec<Input>,
    },
}

impl From<&CoreTrust> for Trust {
    fn from(t: &CoreTrust) -> Self {
        match t {
            CoreTrust::Metrological => Self::Metrological,
            CoreTrust::Indicative { because } => Self::Indicative {
                because: because.iter().map(|&i| i.into()).collect(),
            },
        }
    }
}

/// Where a key was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Mount {
    /// On Stanwood's jig, levelled, turning without friction about its
    /// balance pin.
    OnTheJig,
    /// On its frame, turning about a fixed axis through its balance bearing.
    OnAPivot,
    /// On its frame, free to lift off its balance punching.
    OnItsPunching,
}

impl From<CoreMount> for Mount {
    fn from(m: CoreMount) -> Self {
        match m {
            CoreMount::Jig => Self::OnTheJig,
            CoreMount::Frame(KeyMount::Pivot) => Self::OnAPivot,
            CoreMount::Frame(KeyMount::OnPunching) => Self::OnItsPunching,
        }
    }
}

/// A breakaway walk, as it went.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Walk {
    /// The frictionless balance it started from, grams on the measuring
    /// point.
    pub from_g: f64,
    /// The weight it added at each step, grams; negative going up.
    pub step_g: f64,
    /// The steps it took.
    pub steps: u32,
    /// The steps that moved the key and left it short of the other seat.
    pub stalls: u32,
    /// The halvings of its last step.
    pub halvings: u32,
}

/// What a temporary weight zeroes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Zeroing {
    /// The levelled key's front weight on the jig, Stanwood's.
    OnTheJig,
    /// The key's balance where it rests on its frame.
    OnTheFrame,
}

/// The temporary weight a key friction weight stands on the key.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct TemporaryWeight {
    /// Its mass, grams.
    pub mass_g: f64,
    /// Where it stands, millimetres from the balance pin, positive in front.
    pub at_mm: f64,
    /// What it zeroes.
    pub zeroed: Zeroing,
}

/// What a reading needs beside its number to be a datum.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Conditions {
    /// Where the key was read.
    pub mount: Mount,
    /// The measuring point, millimetres back from the covering's lip.
    pub measuring_point_from_lip_mm: f64,
    /// The contacts holding the seat the reading is taken at or starts from.
    pub held_by: Vec<Contact>,
    /// The breakaway walk, where the reading is one.
    pub walk: Option<Walk>,
    /// The temporary weight standing on the key, where there is one.
    pub temporary_weight: Option<TemporaryWeight>,
    /// The readings it is computed from, where it is computed.
    pub from: Vec<Procedure>,
}

/// One reading of the sheet: read, with its value in full and what the trade
/// reads it as, or refused with why.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Reading {
    /// Read by its procedure, run on the model.
    Read {
        /// Which reading.
        procedure: Procedure,
        /// What the procedure found, in full, in `unit`.
        value: f64,
        /// What the trade reads it as, in `unit`: the value by the
        /// protocol's rounding, to its resolution.
        shown: f64,
        /// The unit of both.
        unit: Unit,
        /// The protocol's resolution, in `unit`.
        resolution: f64,
        /// How the trade reads the value to it.
        rounding: Rounding,
        /// How far it can be trusted.
        trust: Trust,
        /// What it needs beside its number.
        conditions: Conditions,
        /// The static solves it took, increments.
        increments: u32,
        /// The static solves it took, steps.
        iterations: u32,
        /// The solver that took it.
        fingerprint: String,
    },
    /// Refused.
    Refused {
        /// Which reading.
        procedure: Procedure,
        /// Why.
        why: Refusal,
    },
}

impl Reading {
    /// `m` in the trade's units.
    fn read(m: &Measurement) -> Self {
        let (scale, unit) = match m.unit {
            CoreUnit::Gram => (1.0, Unit::Gram),
            CoreUnit::Metre => (1000.0, Unit::Millimetre),
            CoreUnit::One => (1.0, Unit::Ratio),
        };
        let c = &m.conditions;
        Self::Read {
            procedure: m.procedure.into(),
            value: m.value * scale,
            shown: m.shown * scale,
            unit,
            resolution: m.resolution * scale,
            rounding: m.rounding.into(),
            trust: (&m.trust).into(),
            conditions: Conditions {
                mount: c.mount.into(),
                measuring_point_from_lip_mm: mm(c.measuring_point_from_lip_m),
                held_by: contacts(&c.held_by),
                walk: c.walk.map(|w| Walk {
                    from_g: w.from_g,
                    step_g: w.step_g,
                    steps: w.steps,
                    stalls: w.stalls,
                    halvings: w.halvings,
                }),
                temporary_weight: c.temporary_weight.map(|t| TemporaryWeight {
                    mass_g: t.mass_g,
                    at_mm: mm(t.station.x),
                    zeroed: match t.zeroed {
                        Zeroed::OnTheJig => Zeroing::OnTheJig,
                        Zeroed::OnTheFrame => Zeroing::OnTheFrame,
                    },
                }),
                from: c.from.iter().map(|&p| p.into()).collect(),
            },
            increments: m.report.increments,
            iterations: m.report.iterations,
            fingerprint: m.fingerprint.to_owned(),
        }
    }
}

/// The weighing sheet: every reading, in its order, each read or refused,
/// whether or not the frame lets the key be seated.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Sheet {
    /// The readings.
    pub readings: Vec<Reading>,
}

impl Sheet {
    /// The sheet of `bench`.
    pub fn of(bench: &KeyBench) -> Self {
        let sheet = bench.sheet();
        // A key with no stroke is told why: read once, for every reading
        // that needs a stroke.
        let no_travel = sheet
            .readings
            .iter()
            .any(|r| r.result == Err(Refused::Walk(WalkRefusal::NoTravel)));
        let riding = if no_travel {
            bench.riding().map(Riding::from)
        } else {
            None
        };
        Self {
            readings: sheet
                .readings
                .iter()
                .map(|reading| match &reading.result {
                    Ok(m) => Reading::read(m),
                    Err(why) => Reading::Refused {
                        procedure: reading.procedure.into(),
                        why: Refusal::reading(why, riding),
                    },
                })
                .collect(),
        }
    }
}

// ---- diagnostics -----------------------------------------------------------

/// A way the frame and the key do not go together, read off the drawn parts:
/// how far past the limit, so that shrinking the number is the repair.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Fault {
    /// The front rail pin is not inside the key's front mortise.
    PinOutsideMortise {
        /// By how much, millimetres.
        by_mm: f64,
    },
    /// The back rail cloth reaches in front of where the tail's chamfer
    /// begins.
    ClothInFrontOfChamfer {
        /// By how much, millimetres.
        by_mm: f64,
    },
    /// The back rail cloth reaches past the end of the key.
    ClothBehindKey {
        /// By how much, millimetres.
        by_mm: f64,
    },
    /// A punching stands partly off the flat of the key's underside.
    PunchingOffUnderside {
        /// Which punching.
        rail: Part,
        /// By how much, millimetres.
        by_mm: f64,
    },
}

impl From<&CoreFault> for Fault {
    fn from(f: &CoreFault) -> Self {
        match *f {
            CoreFault::PinOutsideTheMortise { by } => Self::PinOutsideMortise { by_mm: mm(by) },
            CoreFault::ClothInFrontOfTheChamfer { by } => {
                Self::ClothInFrontOfChamfer { by_mm: mm(by) }
            }
            CoreFault::ClothBehindTheKey { by } => Self::ClothBehindKey { by_mm: mm(by) },
            CoreFault::PunchingOffTheUnderside { rail, by } => Self::PunchingOffUnderside {
                rail: rail.into(),
                by_mm: mm(by),
            },
        }
    }
}

/// What holds the key where the weight on its measuring point has left it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Verdict {
    /// The weight pushes the key into this stop past what its friction
    /// holds: the stop carries it.
    RestingOn {
        /// The stop.
        stop: Contact,
    },
    /// Friction alone holds the key where it is: the weight lies between
    /// its up weight and its down weight.
    HeldByFriction {
        /// The stop it touches, if it touches one.
        touching: Option<Contact>,
        /// How far the weight stands from moving the key, grams at the
        /// measuring point.
        margin_g: f64,
    },
    /// Off its balance punching, on its front stop and its back rail cloth:
    /// its stops fight.
    RidesOnBothStops {
        /// The front stop it rides on.
        front: Contact,
        /// How far above its balance punching it stands, millimetres.
        lift_mm: f64,
    },
    /// Off its balance punching, held by what it touches.
    LiftsOffBalancePunching {
        /// The stops it touches, in the order the key's contacts are
        /// declared.
        touching: Vec<Contact>,
        /// Whether they need its friction to hold it there.
        needs_friction: bool,
        /// How far above its balance punching it stands, millimetres.
        lift_mm: f64,
    },
    /// Off its frame: no equilibrium in the model holds it on its balance
    /// pin.
    LiftsOffItsFrame,
}

impl From<&TouchVerdict> for Verdict {
    fn from(v: &TouchVerdict) -> Self {
        match v {
            TouchVerdict::RestingOn { stop } => Self::RestingOn {
                stop: (*stop).into(),
            },
            TouchVerdict::HeldByFriction { touching, margin_g } => Self::HeldByFriction {
                touching: touching.map(Contact::from),
                margin_g: *margin_g,
            },
            TouchVerdict::RidesOnBothStops { front, lift_mm } => Self::RidesOnBothStops {
                front: (*front).into(),
                lift_mm: *lift_mm,
            },
            TouchVerdict::LiftsOffBalancePunching {
                touching,
                needs_friction,
                lift_mm,
            } => Self::LiftsOffBalancePunching {
                touching: contacts(touching),
                needs_friction: *needs_friction,
                lift_mm: *lift_mm,
            },
            TouchVerdict::LiftsOffItsFrame => Self::LiftsOffItsFrame,
        }
    }
}

/// What holds the key under the weight on its measuring point, and the gaps
/// at its two outer stops where it stands.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Touch {
    /// What holds it.
    pub verdict: Verdict,
    /// The gap at the front punching, read at its centre, millimetres;
    /// `null` where no underside passes over its centre.
    pub gap_front_mm: Option<f64>,
    /// The gap at the back rail cloth, millimetres; `null` where no feature
    /// of the two can meet.
    pub gap_back_mm: Option<f64>,
}

/// How the key bears on its back rail cloth at the top of its stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum BackBearing {
    /// Face to face.
    Flat,
    /// On the cloth's rounded edge.
    OnTheClothEdge,
    /// On a corner of the key's own outline.
    OnTheKeyCorner,
}

impl From<HowItBears> for BackBearing {
    fn from(how: HowItBears) -> Self {
        match how {
            HowItBears::Flat => Self::Flat,
            HowItBears::StopEdge => Self::OnTheClothEdge,
            HowItBears::KeyCorner => Self::OnTheKeyCorner,
        }
    }
}

/// How the key stands at its two seats, as the trade's gestures seat it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Seats {
    /// How far from level it stands at the bottom of its travel, degrees,
    /// nose-down positive. A design goal is zero.
    pub out_of_level_at_bottom_deg: f64,
    /// What stops it at the bottom.
    pub bottom_held_by: Vec<Contact>,
    /// How it bears on its back rail cloth at the top.
    pub back_bearing: BackBearing,
    /// How far its bearing face is from lying flat on the cloth there,
    /// degrees; positive is its rear end digging in.
    pub bedding_at_the_back_deg: f64,
    /// Where it bears on the cloth, millimetres behind the balance pin.
    pub bears_at_the_back_mm: f64,
}

impl Seats {
    /// The seats of `report`; `None` where the cloth reads no bearing or no
    /// slope, as the bench's readout reads them.
    fn of(report: &SeatReport) -> Option<Self> {
        let back: Bearing = report.back?;
        Some(Self {
            out_of_level_at_bottom_deg: report.out_of_level_deg,
            bottom_held_by: contacts(&report.bottom_held_by),
            back_bearing: back.how.into(),
            bedding_at_the_back_deg: report.bedding_at_the_back_deg()?,
            bears_at_the_back_mm: report.bears_at_the_back_mm()?,
        })
    }
}

/// A punching read at its centre whose drawn edge overlaps the key.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct EdgeOverlap {
    /// Which punching.
    pub pair: Contact,
    /// How far its drawn edge stands into the key, millimetres: what its
    /// felt would compress under a key out of level on it.
    pub mm: f64,
}

/// How the key stands on its frame and why, each part wherever it can be
/// read.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Diagnostics {
    /// Why the key cannot be seated on this frame, which every reading that
    /// seats it is refused for too; `null` where it can.
    pub seat_refused: Option<Refusal>,
    /// How the key rides on its stops where they fight; `null` where its
    /// balance punching carries it, or where it cannot be seated.
    pub riding: Option<Riding>,
    /// Everything about this key and this frame no keyboard could be built
    /// from, front first.
    pub faults: Vec<Fault>,
    /// How it stands at its seats; `null` where it cannot be seated.
    pub seats: Option<Seats>,
    /// What holds it under the weight on its measuring point; `null` where
    /// nothing does.
    pub touch: Option<Touch>,
    /// The punchings the key bears on whose drawn edge overlaps it.
    pub edge_overlaps: Vec<EdgeOverlap>,
}

impl Diagnostics {
    /// The diagnostics of `bench`.
    pub fn of(bench: &KeyBench) -> Self {
        let report = bench.seat_report();
        Self {
            seat_refused: report.as_ref().err().map(Refusal::seat),
            riding: report
                .as_ref()
                .ok()
                .and_then(|r| r.riding)
                .map(Riding::from),
            faults: bench.faults().iter().map(Fault::from).collect(),
            seats: report.as_ref().ok().and_then(Seats::of),
            touch: bench.touch().map(|t| Touch {
                verdict: (&t.verdict).into(),
                gap_front_mm: t.gap_front_mm,
                gap_back_mm: t.gap_back_mm,
            }),
            edge_overlaps: bench
                .edge_overlaps()
                .into_iter()
                .map(|(pair, overlap)| EdgeOverlap {
                    pair: pair.into(),
                    mm: mm(overlap),
                })
                .collect(),
        }
    }
}

// ---- the state -------------------------------------------------------------

/// Where one body's frame stands: its origin, millimetres, and the cosine
/// and sine of its rotation. The view applies it as given and computes none.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Pose {
    /// The origin, millimetres.
    pub origin: [f32; 2],
    /// The rotation's cosine.
    pub cos: f32,
    /// The rotation's sine.
    pub sin: f32,
}

/// A point marked on a body.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Marker {
    /// The body, an index into the state's poses.
    pub body: u32,
    /// Where, millimetres, in the body's frame.
    pub at: [f32; 2],
}

/// Whether a contact the view marks is closed, open or overlapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum ContactState {
    /// Within a tenth of a micron of touching.
    Closed,
    /// Apart.
    Open,
    /// The drawn parts overlap.
    Overlapping,
}

/// A contact the view marks, in the keybed's frame: where the contact rows
/// put its two points, not where a body's pose carries them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct MarkedContact {
    /// Which contact.
    pub pair: Contact,
    /// Where the key's surface is, millimetres.
    pub on_key: [f32; 2],
    /// Where the frame's surface is, millimetres.
    pub on_frame: [f32; 2],
    /// The signed gap between them, millimetres: negative where the drawn
    /// parts overlap.
    pub gap_mm: f32,
    /// Closed, open or overlapping.
    pub state: ContactState,
}

/// Where each body stands, and what the view marks on them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct State {
    /// One per body, in body order: the ground, the slide the key lifts off
    /// its balance punching on, the key.
    pub poses: Vec<Pose>,
    /// The measuring point, on the key: where the engine weighs.
    pub measuring_point: Marker,
    /// The contacts within a millimetre of closing, where the key is drawn.
    pub contacts: Vec<MarkedContact>,
}

impl State {
    /// The state of `bench`.
    pub fn of(bench: &KeyBench) -> Self {
        let m = bench.measuring_point_on_key();
        Self {
            poses: bench
                .poses()
                .into_iter()
                .map(|p| Pose {
                    origin: [mm32(p.origin.x), mm32(p.origin.y)],
                    cos: p.cos as f32,
                    sin: p.sin as f32,
                })
                .collect(),
            measuring_point: Marker {
                body: body_index(KEY.0),
                at: [mm32(m.x), mm32(m.y)],
            },
            contacts: bench
                .contacts()
                .into_iter()
                .map(|c| MarkedContact {
                    pair: c.code.into(),
                    on_key: [mm32(c.on_key.x), mm32(c.on_key.y)],
                    on_frame: [mm32(c.on_frame.x), mm32(c.on_frame.y)],
                    gap_mm: mm32(c.gap),
                    state: match c.state() {
                        CoreContactState::Closed => ContactState::Closed,
                        CoreContactState::Open => ContactState::Open,
                        CoreContactState::Overlapping => ContactState::Overlapping,
                    },
                })
                .collect(),
        }
    }
}

/// A body's index as it crosses: a handful of bodies, never past `u32`.
fn body_index(i: usize) -> u32 {
    u32::try_from(i).unwrap_or(u32::MAX)
}

// ---- the scene -------------------------------------------------------------

/// What a part is to the body that carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Material.
    Solid,
    /// Material taken away: a void, drawn in the section only.
    Cut,
    /// A compliant layer: felt, cloth, paper.
    Lining,
}

/// What a part is made of; the view picks its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum Material {
    KeyWood,
    BlockWood,
    Covering,
    KeyFront,
    RailWood,
    Steel,
    PunchingFelt,
    HardFelt,
    Cloth,
    Paper,
}

/// How an outline becomes a solid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Sweep {
    /// Swept across the keyboard.
    Extruded,
    /// Turned about a vertical axis.
    Revolved,
}

/// How a stretch of an outline was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum How {
    /// Measured on a real part.
    Measured,
    /// Read from a published source.
    Published,
    /// Scaled from a figure or a photograph.
    Traced,
    /// Assumed, pending measurement.
    Assumed,
}

/// Where a stretch of an outline comes from.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Source {
    /// Its first segment.
    pub from: u16,
    /// One past its last.
    pub to: u16,
    /// How it was obtained.
    pub how: How,
    /// The record that holds it, a path under `data/params/`.
    pub record: String,
}

/// One part, in millimetres, in its own body's frame, arcs cut into lines.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Outline {
    /// The body that carries it, an index into the state's poses.
    pub body: u32,
    /// What it is.
    pub code: Part,
    /// Material, void or lining.
    pub role: Role,
    /// What it is made of.
    pub material: Material,
    /// For a paper punching, its own thickness, millimetres: the trade knows
    /// a paper by its colour, and its colour by its thickness.
    pub paper_mm: Option<f32>,
    /// How the outline becomes a solid.
    pub sweep: Sweep,
    /// Extrusion across the sagittal plane, millimetres; zero when turned.
    pub width: f32,
    /// Where a turned part's axis stands along its body's `x`, millimetres;
    /// zero when extruded.
    pub axis_x: f32,
    /// Flat `[x0, y0, x1, y1, …]`, millimetres, in the body's frame; for a
    /// turned part, its profile in radius and height.
    pub points: Vec<f32>,
    /// The segment of the part's outline each point starts or lies on.
    pub segs: Vec<u16>,
    /// Which segments come from where.
    pub sources: Vec<Source>,
}

/// Every part the view draws, each in its own body's frame.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Scene {
    /// The parts.
    pub outlines: Vec<Outline>,
}

impl Scene {
    /// The scene of `bench`.
    pub fn of(bench: &KeyBench) -> Self {
        Self {
            outlines: bench
                .outlines()
                .into_iter()
                .map(|o| {
                    let (sweep, width, axis_x) = match o.sweep {
                        CoreSweep::Extruded { width } => (Sweep::Extruded, mm32(width), 0.0),
                        CoreSweep::Revolved { axis_x } => (Sweep::Revolved, 0.0, mm32(axis_x)),
                    };
                    Outline {
                        body: body_index(o.body.0),
                        code: o.code.into(),
                        role: match o.role {
                            CoreRole::Solid => Role::Solid,
                            CoreRole::Cut => Role::Cut,
                            CoreRole::Lining => Role::Lining,
                        },
                        material: match o.material {
                            CoreMaterial::KeyWood => Material::KeyWood,
                            CoreMaterial::BlockWood => Material::BlockWood,
                            CoreMaterial::Covering => Material::Covering,
                            CoreMaterial::KeyFront => Material::KeyFront,
                            CoreMaterial::RailWood => Material::RailWood,
                            CoreMaterial::Steel => Material::Steel,
                            CoreMaterial::PunchingFelt => Material::PunchingFelt,
                            CoreMaterial::HardFelt => Material::HardFelt,
                            CoreMaterial::Cloth => Material::Cloth,
                            CoreMaterial::Paper { .. } => Material::Paper,
                        },
                        paper_mm: match o.material {
                            CoreMaterial::Paper { thickness } => Some(mm32(thickness)),
                            _ => None,
                        },
                        sweep,
                        width,
                        axis_x,
                        points: o
                            .points
                            .iter()
                            .flat_map(|p| [mm32(p.x), mm32(p.y)])
                            .collect(),
                        segs: o.segs,
                        sources: o
                            .sources
                            .iter()
                            .map(|(range, provenance)| {
                                let (how, record) = match *provenance {
                                    Provenance::Measured(r) => (How::Measured, r),
                                    Provenance::Published(r) => (How::Published, r),
                                    Provenance::Traced(r) => (How::Traced, r),
                                    Provenance::Assumed(r) => (How::Assumed, r),
                                };
                                Source {
                                    from: range.start,
                                    to: range.end,
                                    how,
                                    record: record.to_owned(),
                                }
                            })
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}

// ---- what an action answers ------------------------------------------------

/// One of the two regulation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// Key height, set by the balance punching.
    KeyHeight,
    /// Key dip, set by the front punching.
    KeyDip,
}

impl From<CoreTarget> for Target {
    fn from(t: CoreTarget) -> Self {
        match t {
            CoreTarget::KeyHeight => Self::KeyHeight,
            CoreTarget::KeyDip => Self::KeyDip,
        }
    }
}

/// What keeps a target out of reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Limit {
    /// The front rail pin bottoms in the key's mortise first.
    FrontPinInMortise,
    /// The front rail pin meets the key's wood beside its mortise first.
    FrontPinOnTheWood,
    /// The felt alone, with no paper, already goes past the target.
    FeltThickerThanTheStack,
    /// The key's stops fight before the target is reached.
    StopsFight,
    /// Nothing stops the front of the key.
    NoStopBelowTheFront,
    /// The rail stands too low: the target asks for more paper than the
    /// search lays.
    RailTooLow,
    /// The key would leave its balance pin first.
    OffItsBalancePin,
}

impl From<CoreLimit> for Limit {
    fn from(l: CoreLimit) -> Self {
        match l {
            CoreLimit::FrontPinInMortise => Self::FrontPinInMortise,
            CoreLimit::FrontPinOnTheWood => Self::FrontPinOnTheWood,
            CoreLimit::FeltThickerThanTheStack => Self::FeltThickerThanTheStack,
            CoreLimit::StopsFight => Self::StopsFight,
            CoreLimit::NoStopBelowTheFront => Self::NoStopBelowTheFront,
            CoreLimit::RailTooLow => Self::RailTooLow,
            CoreLimit::OffItsBalancePin => Self::OffItsBalancePin,
        }
    }
}

/// Why a regulation was not laid. The papers, the targets and the key stay
/// as they were.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum RegulationRefusal {
    /// A target no stack reaches.
    RegulationInfeasible {
        /// Which target.
        target: Target,
        /// What was asked of it, millimetres.
        wanted_mm: f64,
        /// What keeps it out of reach.
        limited_by: Limit,
        /// How far out of reach, in its own reading, millimetres.
        shortfall_mm: f64,
        /// The most paper the search lays on a stack, millimetres.
        search_mm: f64,
    },
    /// A target is not a number.
    TargetNotFinite {
        /// Which.
        target: Target,
    },
    /// The key could not be seated at one of the stacks the search tried.
    NoSeat {
        /// Why.
        why: Refusal,
    },
}

impl RegulationRefusal {
    /// Why `targets` were not laid.
    pub fn of(why: &NotRegulated, targets: CoreTargets) -> Self {
        match why {
            NotRegulated::Infeasible {
                target,
                limited_by,
                shortfall_m,
            } => Self::RegulationInfeasible {
                target: (*target).into(),
                wanted_mm: mm(match target {
                    CoreTarget::KeyHeight => targets.key_height,
                    CoreTarget::KeyDip => targets.key_dip,
                }),
                limited_by: (*limited_by).into(),
                shortfall_mm: mm(*shortfall_m),
                search_mm: mm(STACK_SEARCH),
            },
            NotRegulated::Unsolvable(Unsolvable::NotFinite(target)) => Self::TargetNotFinite {
                target: (*target).into(),
            },
            NotRegulated::Unsolvable(Unsolvable::NoSeat(seat)) => Self::NoSeat {
                why: Refusal::seat(seat),
            },
        }
    }
}

/// How a regulation the interface asked for came out.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Regulation {
    /// The papers were laid.
    Laid {
        /// The paper laid under the balance punching, millimetres.
        balance_mm: f64,
        /// The paper laid on the front punching, millimetres.
        front_mm: f64,
        /// The key height the papers give, millimetres.
        key_height_mm: f64,
        /// The key dip the papers give, millimetres.
        key_dip_mm: f64,
        /// The key height asked for, millimetres, as it was asked.
        asked_key_height_mm: f64,
        /// The key dip asked for, millimetres, as it was asked.
        asked_key_dip_mm: f64,
    },
    /// Refused, and nothing changed.
    Refused {
        /// Why.
        why: RegulationRefusal,
    },
}

/// Regulates `bench` to a key height and a key dip, millimetres: the papers
/// laid and what they give, or why not.
pub fn regulate(bench: &mut KeyBench, key_height_mm: f64, key_dip_mm: f64) -> Regulation {
    let targets = CoreTargets {
        key_height: key_height_mm * 1e-3,
        key_dip: key_dip_mm * 1e-3,
    };
    match bench.regulate(targets) {
        Ok(achieved) => {
            let (balance, front) = bench.papers();
            Regulation::Laid {
                balance_mm: mm(balance),
                front_mm: mm(front),
                key_height_mm: mm(achieved.key_height),
                key_dip_mm: mm(achieved.key_dip),
                asked_key_height_mm: key_height_mm,
                asked_key_dip_mm: key_dip_mm,
            }
        }
        Err(why) => Regulation::Refused {
            why: RegulationRefusal::of(&why, targets),
        },
    }
}

/// Why a weighing of the key was refused: the numbers typed are no key's.
/// The key's weighing stays as it was.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum WeighingRefusal {
    /// A mass, a front weight or a distance from the lip that is not a
    /// number.
    NotFinite,
    /// A mass of nothing or less.
    MassNotPositive,
    /// The jig's roller would stand off the covering's top.
    RollerOffTheCovering {
        /// Where it was said to stand, millimetres back from the lip.
        from_lip_mm: f64,
        /// How far the covering runs back from its lip, millimetres.
        covering_mm: f64,
    },
    /// The two readings stand the key's centre of mass off the key.
    CentreOfMassOffTheKey {
        /// Where, millimetres from the balance pin, positive in front.
        at_mm: f64,
    },
}

impl WeighingRefusal {
    /// Why a weighing of `bench`'s key was refused.
    pub fn of(bench: &KeyBench, why: NotAWeighing) -> Self {
        match why {
            NotAWeighing::NotANumber => Self::NotFinite,
            NotAWeighing::MassNotPositive => Self::MassNotPositive,
            NotAWeighing::RollerOffTheCovering { from_lip_m } => Self::RollerOffTheCovering {
                from_lip_mm: mm(from_lip_m),
                covering_mm: mm(bench.key().keytop_length),
            },
            NotAWeighing::CentreOfMassOffTheKey { at_m } => {
                Self::CentreOfMassOffTheKey { at_mm: mm(at_m) }
            }
        }
    }
}

/// Weighs `bench`'s key whole, its front weight read `from_lip_mm`
/// millimetres back from the covering's lip: `None` where it was taken, or
/// why not.
pub fn weigh_key(
    bench: &mut KeyBench,
    mass_g: f64,
    front_weight_g: f64,
    from_lip_mm: f64,
) -> Option<WeighingRefusal> {
    let why = bench
        .weigh_key(mass_g, front_weight_g, from_lip_mm * 1e-3)
        .err()?;
    Some(WeighingRefusal::of(bench, why))
}

/// Why no bushings read a key friction weight typed. The bushings stay as
/// they were.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum FrictionRefusal {
    /// The weight is not a number.
    NotFinite,
    /// Stanwood's protocol cannot be read on this key and frame.
    ProtocolRefused {
        /// Why.
        why: Refusal,
    },
    /// With no friction at all the protocol reads more than the weight
    /// typed.
    UnderNoFriction {
        /// What was typed, grams.
        typed_g: f64,
        /// What the protocol reads with no friction, grams, in full.
        least_g: f64,
    },
    /// The weight lies past what the tightest bushings the bench tries read.
    BeyondTheSearch {
        /// What was typed, grams.
        typed_g: f64,
        /// The most the search read, grams, in full.
        most_g: f64,
        /// What the tightest bushings hold at the measuring point with the
        /// key level, grams.
        holds_g: f64,
    },
}

impl FrictionRefusal {
    /// Why `typed_g` grams of key friction weight set no bushings on
    /// `bench`: a protocol refused for want of a stroke says how the key
    /// rides.
    pub fn of(bench: &KeyBench, typed_g: f64, why: BushingsRefusal) -> Self {
        match why {
            BushingsRefusal::NotANumber => Self::NotFinite,
            BushingsRefusal::Procedure(refused) => {
                let riding = match refused {
                    Refused::Walk(WalkRefusal::NoTravel) => bench.riding().map(Riding::from),
                    _ => None,
                };
                Self::ProtocolRefused {
                    why: Refusal::reading(&refused, riding),
                }
            }
            BushingsRefusal::UnderNoFriction { least_g } => {
                Self::UnderNoFriction { typed_g, least_g }
            }
            BushingsRefusal::BeyondTheSearch { most_g } => Self::BeyondTheSearch {
                typed_g,
                most_g,
                holds_g: FRICTION_SEARCH_G,
            },
        }
    }
}

/// Sets `bench`'s bushings from a key friction weight, grams, as Stanwood's
/// protocol reads it: `None` where they were set, or why not.
pub fn set_key_friction_weight(bench: &mut KeyBench, typed_g: f64) -> Option<FrictionRefusal> {
    let why = bench.set_key_friction_weight(typed_g).err()?;
    Some(FrictionRefusal::of(bench, typed_g, why))
}

/// The TypeScript types of the documents, generated and held to the
/// committed ones.
#[cfg(test)]
mod types {
    use super::*;
    use std::any::TypeId;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};
    use ts_rs::{Config, TypeVisitor, TS};

    /// Where the generated types are committed: beside the adapter whose
    /// package re-exports them.
    fn directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../pa-wasm/types")
    }

    /// Every type reachable from the documents, each with its file.
    struct Generate {
        config: Config,
        seen: BTreeSet<String>,
        ids: Vec<TypeId>,
        files: BTreeMap<PathBuf, String>,
    }

    impl TypeVisitor for Generate {
        fn visit<T: TS + 'static + ?Sized>(&mut self) {
            let Some(path) = T::output_path() else {
                return;
            };
            if self.ids.contains(&TypeId::of::<T>()) {
                return;
            }
            self.ids.push(TypeId::of::<T>());
            let name = T::ident(&self.config);
            assert!(
                self.seen.insert(name.clone()),
                "two boundary types are both called {name}"
            );
            let text = T::export_to_string(&self.config).expect("a boundary type exports");
            self.files.insert(path, text);
            T::visit_dependencies(self);
        }
    }

    /// The files the boundary's types generate, `index.ts` re-exporting
    /// them all.
    fn generated() -> BTreeMap<PathBuf, String> {
        let mut generate = Generate {
            config: Config::new().with_out_dir(directory()),
            seen: BTreeSet::new(),
            ids: Vec::new(),
            files: BTreeMap::new(),
        };
        // The documents, and what the actions that can be refused answer.
        generate.visit::<Settings>();
        generate.visit::<Sheet>();
        generate.visit::<Diagnostics>();
        generate.visit::<State>();
        generate.visit::<Scene>();
        generate.visit::<Regulation>();
        generate.visit::<WeighingRefusal>();
        generate.visit::<FrictionRefusal>();
        let mut index = String::from(
            "// Generated by `cargo test -p pa-app` from `crates/pa-app/src/boundary.rs`.\n\
             // Do not edit this file manually.\n",
        );
        for name in &generate.seen {
            index.push_str(&format!("export type {{ {name} }} from \"./{name}\";\n"));
        }
        generate.files.insert(PathBuf::from("index.ts"), index);
        generate.files
    }

    /// Consistency: **the committed TypeScript types are the ones the Rust
    /// types generate.** Where any differs, is missing or no longer
    /// generated, the test writes the generated set in their place and
    /// fails, naming them: review the difference and commit it. So a type
    /// changed in Rust and not in the interface fails `cargo test`, never a
    /// page.
    #[test]
    fn the_generated_types_are_fresh() {
        let dir = directory();
        let files = generated();
        let mut differ: Vec<String> = Vec::new();
        for (path, text) in &files {
            if std::fs::read_to_string(dir.join(path)).ok().as_deref() != Some(text.as_str()) {
                differ.push(path.display().to_string());
            }
        }
        let mut gone: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = PathBuf::from(entry.file_name());
                if name.extension().is_some_and(|e| e == "ts") && !files.contains_key(&name) {
                    gone.push(name);
                }
            }
        }
        if differ.is_empty() && gone.is_empty() {
            return;
        }
        std::fs::create_dir_all(&dir).expect("the types' directory");
        for (path, text) in &files {
            std::fs::write(dir.join(path), text).expect("a generated type is written");
        }
        for name in &gone {
            std::fs::remove_file(dir.join(name)).expect("a type no longer generated is removed");
        }
        panic!(
            "the TypeScript types in {} were out of date and have been rewritten: {differ:?}, \
             removed {gone:?}. Review the difference and commit it.",
            dir.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pa_core::measure::measurement::Input as CoreInput;

    /// The bench with its back rail raised until its cloth holds the tail
    /// 5 mm above the balance bearing: its stops fight, and the key rides on
    /// them.
    fn wedged() -> KeyBench {
        let mut bench = KeyBench::standard();
        let frame = bench.frame();
        let height = frame.balance_bearing_height() + 0.005 - frame.back_rail_cloth;
        bench.set_back_rail_height(height);
        bench
    }

    /// `value` as JSON, a string's quotes included.
    fn written<T: Serialize>(value: &T) -> String {
        json(value)
    }

    /// Consistency: **a code crosses as the engine names it**: every
    /// contact, part, procedure and input as its `snake_case` identifier in
    /// `pa-core`, so the identifiers the golden sheet and the smoke test read
    /// are the ones the interface words.
    #[test]
    fn a_code_crosses_as_the_engine_names_it() {
        for c in [
            ContactCode::BalanceRailPunching,
            ContactCode::FrontRailPunching,
            ContactCode::BackRailCloth,
            ContactCode::FrontPinInTheMortise,
            ContactCode::FrontPinOnTheWood,
        ] {
            assert_eq!(written(&Contact::from(c)), format!("\"{}\"", c.code()));
        }
        for p in ProcedureId::ALL {
            assert_eq!(written(&Procedure::from(p)), format!("\"{}\"", p.code()));
        }
        for p in [
            PartCode::Keystick,
            PartCode::Keytop,
            PartCode::KeyFront,
            PartCode::KeyButton,
            PartCode::BackCheckBlock,
            PartCode::KeyTailFelt,
            PartCode::Capstan,
            PartCode::FrontMortise,
            PartCode::BalanceRail,
            PartCode::BalanceRailPin,
            PartCode::BalanceRailPunching,
            PartCode::FrontRail,
            PartCode::FrontRailPin,
            PartCode::FrontRailPunching,
            PartCode::PaperPunching,
            PartCode::BackRail,
            PartCode::BackRailCloth,
        ] {
            assert_eq!(written(&Part::from(p)), format!("\"{}\"", p.code()));
        }
        for i in [
            CoreInput::KeyMass,
            CoreInput::CentreOfMassHeight,
            CoreInput::KeyFriction,
            CoreInput::Geometry,
        ] {
            assert_eq!(written(&Input::from(i)), format!("\"{}\"", i.code()));
        }
        for (u, code) in [
            (CoreUnit::Gram, "\"g\""),
            (CoreUnit::Metre, "\"mm\""),
            (CoreUnit::One, "\"ratio\""),
        ] {
            let unit = match u {
                CoreUnit::Gram => Unit::Gram,
                CoreUnit::Metre => Unit::Millimetre,
                CoreUnit::One => Unit::Ratio,
            };
            assert_eq!(written(&unit), code);
        }
    }

    /// Consistency: **a code crosses with its parameters, in the trade's
    /// units with the unit in their names**: a fault in millimetres, a
    /// regulation refused with its target, what keeps it out, its shortfall
    /// and the search's reach.
    #[test]
    fn a_code_crosses_with_its_parameters_in_trade_units() {
        assert_eq!(
            written(&Fault::from(&CoreFault::PinOutsideTheMortise {
                by: 0.0625
            })),
            r#"{"code":"pin_outside_mortise","by_mm":62.5}"#
        );
        assert_eq!(
            written(&Fault::from(&CoreFault::PunchingOffTheUnderside {
                rail: PartCode::FrontRailPunching,
                by: 0.003_125,
            })),
            r#"{"code":"punching_off_underside","rail":"front_rail_punching","by_mm":3.125}"#
        );
        let refused = RegulationRefusal::of(
            &NotRegulated::Infeasible {
                target: CoreTarget::KeyDip,
                limited_by: CoreLimit::FrontPinInMortise,
                shortfall_m: 0.000_75,
            },
            CoreTargets {
                key_height: 0.065,
                key_dip: 0.0125,
            },
        );
        assert_eq!(
            written(&refused),
            r#"{"code":"regulation_infeasible","target":"key_dip","wanted_mm":12.5,"limited_by":"front_pin_in_mortise","shortfall_mm":0.75,"search_mm":20.0}"#
        );
    }

    /// Consistency: **the sheet and the settings cross whole, wherever the
    /// key stands.** With the front rail raised to 50 mm the key has no seat
    /// on its frame: its stops would hold it off its balance pin. Every
    /// setting still crosses, the front rail at its 50 mm among them; every
    /// reading crosses, the jig's read and every other refused, key height
    /// with the very refusal the diagnostics give the frame; and what holds
    /// the key under the weight on it is said as it stands now.
    #[test]
    fn a_frame_that_cannot_be_seated_sends_every_setting_and_every_reading() {
        let mut bench = KeyBench::standard();
        bench.set_front_rail_height(0.050);
        let why = bench.frame_fault().expect("no seat on this frame");

        let settings = Settings::of(&bench);
        assert!((settings.frame.front_rail_mm - 50.0).abs() < 1e-9);
        assert_eq!(
            settings.papers.balance_mm.len(),
            bench.papers_on(Which::Balance).len()
        );

        let diagnostics = Diagnostics::of(&bench);
        assert_eq!(diagnostics.seat_refused, Some(Refusal::seat(&why)));
        assert_eq!(diagnostics.seats, None);
        assert_eq!(
            diagnostics.touch.map(|t| t.verdict),
            bench.finger_verdict().as_ref().map(Verdict::from),
            "what holds the key is said as it stands now"
        );

        let sheet = Sheet::of(&bench);
        assert_eq!(sheet.readings.len(), ProcedureId::ALL.len());
        for (reading, p) in sheet.readings.iter().zip(ProcedureId::ALL) {
            let read = matches!(reading, Reading::Read { .. });
            let on_the_jig = matches!(p, ProcedureId::FrontWeight | ProcedureId::KeyWeightRatio);
            assert_eq!(read, on_the_jig, "{p:?}: {reading:?}");
        }
        assert_eq!(
            sheet.readings[2],
            Reading::Refused {
                procedure: Procedure::KeyHeight,
                why: Refusal::seat(&why),
            }
        );
    }

    /// Consistency: **a key with no stroke is told why.** On the wedged
    /// frame every reading that needs a stroke is refused as `no_travel`,
    /// carrying how the key rides: on its front punching, lifted by the
    /// seats' own lift to the bit; the diagnostics say the same and refuse
    /// no seat. On the standard bench nothing rides.
    #[test]
    fn a_key_with_no_stroke_is_told_why() {
        let bench = wedged();
        let riding = bench.riding().expect("the stops fight");
        let crossed = Riding {
            front: Contact::FrontRailPunching,
            lift_mm: riding.lift * 1000.0,
        };
        let sheet = Sheet::of(&bench);
        for p in [
            Procedure::DownWeight,
            Procedure::UpWeight,
            Procedure::KeyFrictionWeightZeroedOnTheJig,
            Procedure::KeyFrictionWeightZeroedOnTheFrame,
        ] {
            assert!(
                sheet.readings.contains(&Reading::Refused {
                    procedure: p,
                    why: Refusal::NoTravel {
                        riding: Some(crossed),
                    },
                }),
                "{p:?}"
            );
        }
        let diagnostics = Diagnostics::of(&bench);
        assert_eq!(diagnostics.riding, Some(crossed));
        assert_eq!(diagnostics.seat_refused, None);

        let standard = KeyBench::standard();
        assert_eq!(Diagnostics::of(&standard).riding, None);
        assert!(Sheet::of(&standard)
            .readings
            .iter()
            .all(|r| matches!(r, Reading::Read { .. })));
    }

    /// Consistency, and what the owner sees: **a refused action answers in
    /// codes and numbers, and changes nothing.** A 12 mm dip the front pin
    /// forbids, a mass of nothing, a front weight read off the covering, a
    /// key friction weight under what no friction reads and one typed on a
    /// frame whose stops fight: each answers its code with the numbers its
    /// wording needs, and the documents read as before. A dip the pin
    /// allows is laid, and says what it was asked as it was asked.
    #[test]
    fn a_refused_action_answers_in_codes_and_changes_nothing() {
        let mut bench = KeyBench::standard();
        let documents = |b: &KeyBench| {
            json(&(
                Settings::of(b),
                Sheet::of(b),
                Diagnostics::of(b),
                State::of(b),
            ))
        };
        let before = documents(&bench);

        let Regulation::Refused { why } = regulate(&mut bench, 65.0, 12.0) else {
            panic!("the pin forbids 12 mm")
        };
        let RegulationRefusal::RegulationInfeasible {
            target,
            wanted_mm,
            limited_by,
            shortfall_mm,
            search_mm,
        } = why
        else {
            panic!("{why:?}")
        };
        assert_eq!(
            (target, limited_by),
            (Target::KeyDip, Limit::FrontPinInMortise)
        );
        assert_eq!(wanted_mm.to_bits(), (12.0 * 1e-3 * 1000.0_f64).to_bits());
        assert!((shortfall_mm - 0.79).abs() < 0.005, "{shortfall_mm}");
        assert!((search_mm - 20.0).abs() < 1e-12);

        assert_eq!(
            weigh_key(&mut bench, 0.0, 6.5, 13.0),
            Some(WeighingRefusal::MassNotPositive)
        );
        assert_eq!(
            weigh_key(&mut bench, 121.1, 6.5, 200.0),
            Some(WeighingRefusal::RollerOffTheCovering {
                from_lip_mm: 0.2 * 1000.0,
                covering_mm: bench.key().keytop_length * 1000.0,
            })
        );
        assert_eq!(
            weigh_key(&mut bench, 121.1, f64::NAN, 13.0),
            Some(WeighingRefusal::NotFinite)
        );
        let Some(FrictionRefusal::UnderNoFriction { typed_g, least_g }) =
            set_key_friction_weight(&mut bench, 0.0)
        else {
            panic!("no bushings read 0 g on this key")
        };
        assert_eq!(typed_g.to_bits(), 0.0_f64.to_bits());
        assert!(least_g > 0.2 && least_g < 0.5, "{least_g}");
        assert_eq!(documents(&bench), before, "a refusal changed the bench");

        let mut wedged = wedged();
        let riding = wedged.riding().map(Riding::from);
        assert_eq!(
            set_key_friction_weight(&mut wedged, 6.0),
            Some(FrictionRefusal::ProtocolRefused {
                why: Refusal::NoTravel { riding },
            })
        );

        let Regulation::Laid {
            balance_mm,
            front_mm,
            asked_key_height_mm,
            asked_key_dip_mm,
            ..
        } = regulate(&mut bench, 65.0, 10.6)
        else {
            panic!("10.6 mm of dip is reachable")
        };
        let papers = Settings::of(&bench).papers;
        assert_eq!(balance_mm.to_bits(), papers.balance_total_mm.to_bits());
        assert_eq!(front_mm.to_bits(), papers.front_total_mm.to_bits());
        assert_eq!(
            (asked_key_height_mm.to_bits(), asked_key_dip_mm.to_bits()),
            (65.0_f64.to_bits(), 10.6_f64.to_bits())
        );
    }

    /// Consistency: **the settings are the bench's own**, in the trade's
    /// units: what each setter set reads back in millimetres, the back rail
    /// and the capstan as distances behind the balance pin, and the bushings
    /// as the torque they hold, 6 g on the level arm of the default bench
    /// read back as 6 g over that arm.
    #[test]
    fn the_settings_are_the_benchs_own() {
        let mut bench = KeyBench::standard();
        bench.set_front_rail_height(0.0307);
        bench.set_back_rail(-0.2);
        bench.set_capstan(-0.1295);
        bench.set_finger_weight(-12.5);
        bench.set_measuring_point_from_lip(0.0127);
        let s = Settings::of(&bench);
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(
            close(s.frame.front_rail_mm, 30.7),
            "{}",
            s.frame.front_rail_mm
        );
        assert!(close(s.frame.back_rail_behind_mm, 200.0));
        assert!(close(s.key.capstan_behind_mm, 129.5));
        assert_eq!(
            s.weight_on_the_measuring_point_g.to_bits(),
            (-12.5_f64).to_bits()
        );
        assert!(close(s.measuring_point_from_lip_mm, 12.7));
        assert_eq!(s.key.weighed, None);
        assert_eq!(s.key.mass_g, bench.key_mass_g());

        let standard = KeyBench::standard();
        let torque = Settings::of(&standard).key.friction_torque_gmm;
        let arm_mm = standard.measuring_point() * 1000.0;
        assert!((torque / arm_mm - 6.0).abs() < 1e-12, "{torque} g·mm");
    }

    /// Consistency: **what holds the key crosses as its code.** The front
    /// rail raised to 41.7 mm tips the key forward off its balance punching,
    /// onto its front punching and its front pin against the wall of its
    /// slot, its cloth open: those two hold it, with no friction. Pulled
    /// 62.7 g, turned up on its cloth off its punching, the cloth alone
    /// holds it, with its friction.
    #[test]
    fn what_holds_the_key_crosses_as_its_code() {
        let mut bench = KeyBench::standard();
        bench.set_front_rail_height(0.0417);
        let touch = Diagnostics::of(&bench).touch.expect("a verdict");
        let Verdict::LiftsOffBalancePunching {
            touching,
            needs_friction,
            lift_mm,
        } = touch.verdict
        else {
            panic!("{:?}", touch.verdict)
        };
        assert_eq!(
            touching,
            vec![Contact::FrontRailPunching, Contact::FrontPinOnTheWood]
        );
        assert!(!needs_friction);
        assert!(lift_mm > 0.0);

        let mut bench = KeyBench::standard();
        bench.set_finger_weight(-62.7);
        let touch = Diagnostics::of(&bench).touch.expect("a verdict");
        let Verdict::LiftsOffBalancePunching {
            touching,
            needs_friction,
            ..
        } = touch.verdict
        else {
            panic!("{:?}", touch.verdict)
        };
        assert_eq!(touching, vec![Contact::BackRailCloth]);
        assert!(needs_friction);
    }
}
