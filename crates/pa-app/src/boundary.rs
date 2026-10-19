//! What crosses to an interface: the instrument's settings, its registry,
//! its weighing sheet, its diagnostics, where its bodies stand and what it
//! draws, each a document of its own, in codes and numbers; and what an
//! edit answers.
//!
//! # Codes and numbers, never sentences
//!
//! An interface words what it shows in its reader's language, and wording
//! built here could not be translated. So a refusal, a fault or a verdict
//! crosses as a code with its parameters,
//! `{"code":"pin_outside_mortise","by_mm":16.5}`, the parameters in the
//! trade's units with the unit in their names, and the interface words it
//! from one catalog per language (`apps/bench/src/i18n/`). A code carries
//! every number its wording needs, the instrument's own limits included, so
//! no wording of a code holds a number of the engine's.
//!
//! The codes mirror the domain's enums in `pa-core` through `match`es with
//! no catch-all arm: a refusal added there and not mapped here is a compile
//! error, and one mapped here and not worded there fails the interface's
//! type-check.
//!
//! # Seven documents, apart, and two calls
//!
//! [`Settings`] are always valid, whatever the frame does, nested as the
//! paths are: every control reads its value at its path, so a frame that
//! cannot be weighed still shows every control and the way back out of it.
//! [`Registry`] lists every path with what it admits and how it is
//! reached, and the page that presents them, each control with its panel,
//! its label's code and the range it offers. [`Sheet`] carries every
//! reading, each read or refused with its reason, also always: a frame that
//! cannot seat its key reads its jig and refuses the rest line by line.
//! [`Diagnostics`] say how the key stands and why, and the last edit
//! refused; [`State`] where each body stands and the contacts the view
//! marks, each with its normal and the force it carries; [`Scene`] what is
//! drawn, each part in its own body's frame; and [`Band`] the band, taken only
//! when an interface asks for it, as it shows it. An interface changes the
//! instrument by two calls, [`set`] a path to a value and [`act`] an
//! [`Action`], each answering [`Acted`].
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
    Bearing, ContactCode, HowItBears, Material as CoreMaterial, PartCode, Role as CoreRole,
    Sweep as CoreSweep,
};
use pa_core::measure::jig::JigRefusal;
use pa_core::measure::measurement::{
    Input as CoreInput, Measurement, Mount as CoreMount, ProcedureId, Rounding as CoreRounding,
    Trust as CoreTrust, Unit as CoreUnit, Zeroed,
};
use pa_core::measure::seat::SeatRefusal;
use pa_core::measure::sweep::{Sample, SweepRefusal};
use pa_core::measure::walk::{Stroke as CoreStroke, WalkRefusal, CAP, STEP_G};
use pa_core::measure::weighing::{BushingsRefusal, Refused, FRICTION_SEARCH_G};
use pa_core::parts::faults::Fault as CoreFault;
use pa_core::parts::frame::{Drawer, PAPER};
use pa_core::parts::mounts::KeyMount;
use pa_core::parts::{KeyMass, NotAWeighing};
use pa_core::registry::{
    Kind as CoreKind, Layer as CoreLayer, Quantity as CoreQuantity, Step as CoreStep,
};
use pa_core::regulation::{Limit as CoreLimit, Target as CoreTarget, Unsolvable, STACK_SEARCH};
use pa_core::validate::{Problem as CoreProblem, DRAWER};
use pa_core::G;

use crate::instrument::{
    Action, ActionCode, ContactState as CoreContactState, Edit as CoreEdit,
    EditRefusal as CoreEditRefusal, Instrument, KeyPlace as CoreKeyPlace, NotRegulated, Outcome,
    Riding as CoreRiding, SeatReport, Stack, TouchVerdict, KEY,
};
use crate::profile::{Known, PaperColor, Profile};
use crate::registry::{self, path, Control, Entry, Field, Label, Panel, Weighing as WeighingField};

/// A document as the interface reads it: JSON, the shape of its generated
/// TypeScript type.
pub fn json<T: Serialize>(document: &T) -> String {
    // Every document is a tree of structs, enums, numbers and strings, whose
    // serialization cannot fail; a number that is not finite is written
    // `null`, as JSON has no other way to say it.
    serde_json::to_string(document).unwrap_or_else(|_| "null".to_owned())
}

/// Meters to millimeters.
fn mm(meters: f64) -> f64 {
    meters * 1000.0
}

/// Meters to millimeters, narrowed for drawing.
fn mm32(meters: f64) -> f32 {
    (meters * 1000.0) as f32
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
    TopClearance,
    KeyTailFelt,
    Capstan,
    FrontMortise,
    BalanceMortise,
    BalanceMortiseBushing,
    BalanceHole,
    KeyButtonSideHole,
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
            PartCode::TopClearance => Self::TopClearance,
            PartCode::KeyTailFelt => Self::KeyTailFelt,
            PartCode::Capstan => Self::Capstan,
            PartCode::FrontMortise => Self::FrontMortise,
            PartCode::BalanceMortise => Self::BalanceMortise,
            PartCode::BalanceMortiseBushing => Self::BalanceMortiseBushing,
            PartCode::BalanceHole => Self::BalanceHole,
            PartCode::KeyButtonSideHole => Self::KeyButtonSideHole,
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
    /// Front weight, on Stanwood's leveled jig.
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
    /// Key friction weight by Stanwood's protocol, zeroed on the leveled
    /// jig.
    KeyFrictionWeightZeroedOnTheJig,
    /// Key friction weight zeroed where the key rests on its frame.
    KeyFrictionWeightZeroedOnTheFrame,
    /// Down weight read on the band: the greatest force of the driven
    /// sweep's down curve, the walk's in full within a few thousandths of a
    /// gram. Shown in full and never as a gram: within that of a whole gram
    /// the two read the gram either side of it, and the gram is the walk's.
    DownWeightOnTheBand,
    /// Up weight read on the band: the least force of its up curve, shown
    /// in full likewise.
    UpWeightOnTheBand,
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
            ProcedureId::DownWeightOnTheBand => Self::DownWeightOnTheBand,
            ProcedureId::UpWeightOnTheBand => Self::UpWeightOnTheBand,
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
    /// How far above its balance punching it stands, millimeters.
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
    /// A step of the band's sweep, on the key's way down or up.
    Sweep,
}

/// Which way the key goes: a walk's or a sweep's stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Stroke {
    /// From the top seat down.
    Down,
    /// From the bottom seat up.
    Up,
}

impl From<CoreStroke> for Stroke {
    fn from(s: CoreStroke) -> Self {
        match s {
            CoreStroke::Down => Self::Down,
            CoreStroke::Up => Self::Up,
        }
    }
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
        /// millimeters: negative where the punching stands above the pin's
        /// top, by that much.
        leaves_its_pin_mm: f64,
    },
    /// The front punching's center stands off the key's underside: nothing
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
    /// A step of the band's sweep would lift the key off its balance
    /// punching: under the force a finger at the measuring point would need
    /// there, the punching would carry nothing, or a stop would hold the key
    /// off it.
    SweepLiftsOffItsPunching {
        /// On the way down or up.
        stroke: Stroke,
        /// How far the measuring point stood below the top seat,
        /// millimeters.
        travel_mm: f64,
        /// The force the finger would need there, grams, positive pressing
        /// down.
        force_g: f64,
    },
    /// A step of the band's sweep took the key off its balance pin, and off
    /// its frame.
    SweepLiftsOffItsFrame,
    /// No top of the key stands where the temporary weight goes.
    NoKeyTopAtTheStation {
        /// Where it goes, millimeters from the balance pin, positive in
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
            Refused::Sweep(sweep) => match sweep {
                SweepRefusal::NoTravel => Self::NoTravel { riding },
                SweepRefusal::NoLever => Self::NoLever,
                SweepRefusal::LiftsOffItsPunching {
                    stroke,
                    travel,
                    grams,
                } => Self::SweepLiftsOffItsPunching {
                    stroke: (*stroke).into(),
                    travel_mm: mm(*travel),
                    force_g: *grams,
                },
                SweepRefusal::LiftsOffItsFrame => Self::SweepLiftsOffItsFrame,
                SweepRefusal::NoEquilibrium { on } => Self::NoEquilibrium { on: contacts(on) },
                SweepRefusal::NotConverged {
                    iterations,
                    residual,
                } => Self::NotConverged {
                    solve: Solve::Sweep,
                    iterations: *iterations,
                    residual_n: *residual,
                },
                SweepRefusal::Mechanism(_) => Self::GeometryDegenerate,
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

/// Every setting of the instrument, in the trade's units, nested as its
/// paths are: the value at `frame.front_rail.pin_height_mm` is
/// `frame.front_rail.pin_height_mm` here ([`crate::registry`]). Always
/// valid, whatever the frame does, so every control reads its value here.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Settings {
    /// The profile the instrument was built from, which a reset returns
    /// to, by its name (`data/profiles/<name>.json`).
    pub profile: String,
    /// The protocol the key is weighed by.
    pub protocol: ProtocolSettings,
    /// What an experiment puts on the key.
    pub experiment: ExperimentSettings,
    /// The regulation's targets.
    pub regulation: RegulationSettings,
    /// The key.
    pub key: KeySettings,
    /// The frame it sits on.
    pub frame: FrameSettings,
}

/// The protocol's setting.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct ProtocolSettings {
    /// How far back from the covering's lip every weight is centered,
    /// millimeters: the measuring point of every touchweight.
    pub measuring_point_from_lip_mm: f64,
}

/// What an experiment puts on the key.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct ExperimentSettings {
    /// The weight on the measuring point, grams: negative pulls up.
    pub weight_g: f64,
}

/// The targets of the regulation last laid.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct RegulationSettings {
    /// Key height, millimeters.
    pub key_height_mm: f64,
    /// Key dip, millimeters.
    pub key_dip_mm: f64,
}

/// The key.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct KeySettings {
    /// Its front edge, millimeters in front of the balance pin.
    pub front_edge_mm: f64,
    /// Its capstan.
    pub capstan: CapstanSettings,
    /// Its front mortise.
    pub front_mortise: MortiseSettings,
    /// Its key button's height above the keystick, millimeters.
    pub capsule_height_mm: f64,
    /// Its key button's length along the key, millimeters.
    pub capsule_length_mm: f64,
    /// The radius its key button's ends are rounded by, millimeters: none
    /// for square ends.
    pub capsule_end_radius_mm: f64,
    /// Its balance mortise.
    pub balance_mortise: BalanceMortiseSettings,
    /// Its bushings.
    pub friction: FrictionSettings,
    /// Its weighing as it was typed; `null` while it is published or not
    /// weighed.
    pub weighing: Option<Weighed>,
    /// Its mass as the statics take it, grams: as published or as weighed;
    /// `null` until it is weighed. Not a setting: what the weighing or the
    /// source gives.
    pub mass_g: Option<f64>,
    /// How its mass is known.
    pub mass_known: MassKnown,
}

/// How the key's mass is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum MassKnown {
    /// Nobody has weighed it: it has no mass in the model, and every reading
    /// that needs one is refused until it is weighed.
    Unweighed,
    /// Weighed whole on the bench, as typed.
    Weighed,
    /// Weighed and located by a published source, whole, its center of
    /// mass's height included.
    Published,
}

/// The capstan.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct CapstanSettings {
    /// Where it stands, millimeters behind the balance pin.
    pub behind_mm: f64,
    /// How far it stands above the keystick, millimeters.
    pub height_mm: f64,
}

/// The front mortise.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct MortiseSettings {
    /// Its depth above the underside, millimeters.
    pub depth_mm: f64,
    /// The diameter of the cutter it is milled with, its width across the
    /// key, millimeters.
    pub cutter_diameter_mm: f64,
    /// How far that cutter moves along the key, millimeters.
    pub travel_mm: f64,
}

/// The balance mortise: the slot through the key button, its cavity in the
/// keystick, its hole, its bushing and the key button's side holes the
/// bushing turns out through.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct BalanceMortiseSettings {
    /// The diameter of the cutter the slot is milled with, its width across
    /// the key, millimeters.
    pub cutter_diameter_mm: f64,
    /// How far that cutter moves along the key, millimeters.
    pub travel_mm: f64,
    /// The cavity's floor above the underside, millimeters.
    pub floor_mm: f64,
    /// The balance hole's diameter, millimeters.
    pub hole_diameter_mm: f64,
    /// The bushing's thickness on each side of the slot, millimeters.
    pub bushing_thickness_mm: f64,
    /// How far down from the key button's top the bushing lines the slot,
    /// millimeters.
    pub bushing_depth_mm: f64,
    /// The height of the key button's side holes, millimeters.
    pub side_hole_height_mm: f64,
}

/// The bushings.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FrictionSettings {
    /// Their friction as stored: the torque they hold, gram-force
    /// millimeters. What Stanwood's protocol reads them as is a reading, on
    /// the sheet.
    pub torque_gmm: f64,
}

/// A key weighed whole, as it was typed.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Weighed {
    /// Its mass on the scale, grams.
    pub mass_g: f64,
    /// Its front weight as the jig's scale read it, grams.
    pub front_weight_g: f64,
    /// How far back from the covering's lip the roller stood, millimeters.
    pub from_lip_mm: f64,
}

/// The frame's own dimensions, which the key does not move, and the papers
/// laid on its punchings.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FrameSettings {
    /// The balance rail.
    pub balance_rail: BalanceRailSettings,
    /// The front rail.
    pub front_rail: FrontRailSettings,
    /// The back rail.
    pub back_rail: BackRailSettings,
    /// The balance punching.
    pub balance_punching: PunchingSettings,
    /// The front punching.
    pub front_punching: PunchingSettings,
}

/// The balance rail.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct BalanceRailSettings {
    /// The bare rail's height above the keybed, millimeters.
    pub height_mm: f64,
    /// How far its pin stands above the bare rail, millimeters.
    pub pin_height_mm: f64,
}

/// The front rail.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FrontRailSettings {
    /// The bare rail's height above the keybed, millimeters.
    pub height_mm: f64,
    /// Where it stands, millimeters in front of the balance pin.
    pub at_mm: f64,
    /// How far its pin stands above the bare rail, millimeters.
    pub pin_height_mm: f64,
}

/// The back rail.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct BackRailSettings {
    /// The bare rail's height above the keybed, millimeters.
    pub height_mm: f64,
    /// Where it stands, millimeters behind the balance pin.
    pub behind_mm: f64,
    /// Its cloth's thickness, millimeters.
    pub cloth_mm: f64,
}

/// A punching: its felt, and the papers under it, the coordinate regulation
/// sets.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct PunchingSettings {
    /// The felt washer, millimeters.
    pub felt_mm: f64,
    /// The papers, millimeters each, in the order they were laid, bottom
    /// first.
    pub papers_mm: Vec<f64>,
    /// Their total, millimeters, as the engine sums it.
    pub paper_total_mm: f64,
}

impl Settings {
    /// The settings of `instrument`.
    pub fn of(instrument: &Instrument) -> Self {
        let at = |path: &str| {
            instrument
                .value(path)
                .unwrap_or_else(|| panic!("{path} holds a value"))
        };
        let punching = |stack: Stack, felt: &str| {
            let papers = instrument.papers(stack);
            PunchingSettings {
                felt_mm: at(felt),
                papers_mm: papers.iter().map(|&p| mm(p)).collect(),
                paper_total_mm: mm(papers.iter().sum()),
            }
        };
        Self {
            profile: instrument.profile().name.clone(),
            protocol: ProtocolSettings {
                measuring_point_from_lip_mm: at(path::MEASURING_POINT),
            },
            experiment: ExperimentSettings {
                weight_g: at(path::WEIGHT),
            },
            regulation: RegulationSettings {
                key_height_mm: at(path::KEY_HEIGHT),
                key_dip_mm: at(path::KEY_DIP),
            },
            key: KeySettings {
                front_edge_mm: at(path::FRONT_EDGE),
                capstan: CapstanSettings {
                    behind_mm: at(path::CAPSTAN_BEHIND),
                    height_mm: at(path::CAPSTAN_HEIGHT),
                },
                front_mortise: MortiseSettings {
                    depth_mm: at(path::FRONT_MORTISE_DEPTH),
                    cutter_diameter_mm: at(path::FRONT_MORTISE_CUTTER_DIAMETER),
                    travel_mm: at(path::FRONT_MORTISE_TRAVEL),
                },
                capsule_height_mm: at(path::CAPSULE_HEIGHT),
                capsule_length_mm: at(path::CAPSULE_LENGTH),
                capsule_end_radius_mm: at(path::CAPSULE_END_RADIUS),
                balance_mortise: BalanceMortiseSettings {
                    cutter_diameter_mm: at(path::BALANCE_MORTISE_CUTTER_DIAMETER),
                    travel_mm: at(path::BALANCE_MORTISE_TRAVEL),
                    floor_mm: at(path::BALANCE_MORTISE_FLOOR),
                    hole_diameter_mm: at(path::BALANCE_HOLE_DIAMETER),
                    bushing_thickness_mm: at(path::BUSHING_THICKNESS),
                    bushing_depth_mm: at(path::BUSHING_DEPTH),
                    side_hole_height_mm: at(path::SIDE_HOLE_HEIGHT),
                },
                friction: FrictionSettings {
                    torque_gmm: at(path::FRICTION_TORQUE),
                },
                weighing: match instrument.key_mass() {
                    KeyMass::Unweighed | KeyMass::Published { .. } => None,
                    KeyMass::Weighed { .. } => Some(Weighed {
                        mass_g: at(path::WEIGHED_MASS),
                        front_weight_g: at(path::WEIGHED_FRONT_WEIGHT),
                        from_lip_mm: at(path::WEIGHED_FROM_LIP),
                    }),
                },
                mass_g: instrument.key_mass_g(),
                mass_known: match instrument.key_mass() {
                    KeyMass::Unweighed => MassKnown::Unweighed,
                    KeyMass::Weighed { .. } => MassKnown::Weighed,
                    KeyMass::Published { .. } => MassKnown::Published,
                },
            },
            frame: FrameSettings {
                balance_rail: BalanceRailSettings {
                    height_mm: at(path::BALANCE_RAIL_HEIGHT),
                    pin_height_mm: at(path::BALANCE_PIN_HEIGHT),
                },
                front_rail: FrontRailSettings {
                    height_mm: at(path::FRONT_RAIL_HEIGHT),
                    at_mm: at(path::FRONT_RAIL_AT),
                    pin_height_mm: at(path::FRONT_PIN_HEIGHT),
                },
                back_rail: BackRailSettings {
                    height_mm: at(path::BACK_RAIL_HEIGHT),
                    behind_mm: at(path::BACK_RAIL_BEHIND),
                    cloth_mm: at(path::BACK_RAIL_CLOTH),
                },
                balance_punching: punching(Stack::Balance, path::BALANCE_FELT),
                front_punching: punching(Stack::Front, path::FRONT_FELT),
            },
        }
    }
}

// ---- the registry ----------------------------------------------------------

/// The unit a setting's value is in, as its path's name says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum SettingUnit {
    /// Millimeters.
    Mm,
    /// Grams.
    G,
    /// Gram-force millimeters.
    Gmm,
}

/// What a setting measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Quantity {
    /// A length or a position.
    Length,
    /// A mass or a weight.
    Mass,
    /// A torque.
    Torque,
}

/// Which layer of the instrument a setting belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// What the choice of parts fixes.
    Design,
    /// What a technician turns or lays to regulate.
    Regulation,
    /// The conventions a protocol is run under.
    Protocol,
    /// What an experiment puts on the key.
    Experiment,
}

/// How a setting is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Set directly.
    Actuated,
    /// Aimed at, the regulation solve reaching it.
    Target,
    /// Typed as it was read on a real part.
    Measured,
}

/// How a setting moves.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum Step {
    /// Continuously: a dimension edited, a screw turned.
    Continuous,
    /// By the papers a drawer holds.
    Drawer {
        /// Their thicknesses, millimeters, thinnest first.
        sizes_mm: Vec<f64>,
    },
}

/// How a setting is reached from an interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "by", rename_all = "snake_case")]
pub enum SetBy {
    /// `set(path, value)`.
    Set,
    /// `act`, by this action.
    Act {
        /// The action.
        action: ActionCode,
    },
}

/// One setting of the instrument: its path, what it admits, how it is
/// reached.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Param {
    /// The path, its unit in its name.
    pub path: String,
    /// The unit.
    pub unit: SettingUnit,
    /// What it measures.
    pub quantity: Quantity,
    /// Its layer.
    pub layer: Layer,
    /// How it is reached.
    pub kind: Kind,
    /// Whether its value is a list: a punching's papers.
    pub list: bool,
    /// The least it admits, in its unit; `null` where nothing bounds it
    /// below, or where what bounds it is another setting.
    pub min: Option<f64>,
    /// The most it admits; `null` likewise.
    pub max: Option<f64>,
    /// How it moves.
    pub step: Step,
    /// How an interface reaches it.
    pub set_by: SetBy,
}

/// A panel of the page.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct PanelLayout {
    /// Which.
    pub panel: Panel,
    /// Its controls, in order.
    pub controls: Vec<ControlLayout>,
}

/// A field of a form, in its unit.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FieldLayout {
    /// The action's argument it fills.
    pub argument: String,
    /// The path the argument stands for: its unit, what it admits, how it
    /// is reached.
    pub path: String,
    /// Where its first value is read, when the page opens.
    pub seed: Seed,
    /// What it is worded by.
    pub label: Label,
    /// The least it offers, if bounded.
    pub min: Option<f64>,
    /// The most it offers, if bounded.
    pub max: Option<f64>,
    /// The step it moves in.
    pub step: f64,
}

/// Where a form's field reads its first value: what the bench holds for
/// that quantity when the page opens.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum Seed {
    /// The value at this address of the settings document, nested as the
    /// paths are: a path, or what the settings report beside them.
    Setting {
        /// The address.
        at: String,
    },
    /// A reading of the sheet, as the trade reads it.
    Reading {
        /// The reading.
        procedure: Procedure,
    },
}

/// One control of the page.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "control", rename_all = "snake_case")]
pub enum ControlLayout {
    /// A slider over a path `set` reaches, its range the page's offer
    /// inside the hard one.
    Slider {
        /// The path.
        path: String,
        /// What it is worded by.
        label: Label,
        /// The least it offers, in the path's unit.
        min: f64,
        /// The most it offers.
        max: f64,
        /// The step it sweeps in.
        step: f64,
    },
    /// A punching's pile of papers: laid from the drawer by `add_paper`,
    /// lifted by `remove_paper`.
    Stack {
        /// The papers' path.
        path: String,
        /// The address, in the settings document, of the engine's own
        /// total of them: the page sums nothing.
        total: String,
        /// Which punching.
        stack: Stack,
        /// What it is worded by.
        label: Label,
    },
    /// Fields typed and sent together by one action.
    Form {
        /// The action.
        action: ActionCode,
        /// What its button is worded by.
        button: Label,
        /// Its fields.
        fields: Vec<FieldLayout>,
    },
    /// The bushings' control: it shows the sheet's reading `reads`, and
    /// sets the bushings through `action`, in grams.
    KeyFrictionWeight {
        /// What it is worded by.
        label: Label,
        /// The reading it shows.
        reads: Procedure,
        /// The action it sets through.
        action: ActionCode,
        /// The action's argument the grams fill.
        argument: String,
        /// The least it offers, grams.
        min: f64,
        /// The most it offers, grams.
        max: f64,
        /// The step it sweeps in, grams.
        step: f64,
    },
    /// A button running an action with no argument.
    Button {
        /// The action.
        action: ActionCode,
        /// What it is worded by.
        label: Label,
    },
    /// A choice of what is on the bench, among the profiles it ships, run by
    /// `action` with the chosen one's name as `argument`.
    Profile {
        /// The action.
        action: ActionCode,
        /// The action's argument the name fills.
        argument: String,
        /// What it is worded by.
        label: Label,
        /// The profiles to choose from, by name, in the order shipped.
        options: Vec<String>,
        /// The address, in the settings document, of the name of the one on
        /// the bench.
        at: String,
    },
}

/// The registry: every path of the instrument, what it admits and how it is
/// reached, and the page that presents them, panel by panel.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Registry {
    /// Every path, in a fixed order.
    pub params: Vec<Param>,
    /// The page: its panels, in order.
    pub page: Vec<PanelLayout>,
}

impl Registry {
    /// The registry of `instrument`'s paths, its papers from its drawer.
    pub fn of(instrument: &Instrument) -> Self {
        let drawer = &instrument.frame().drawer;
        Self {
            params: registry::ENTRIES
                .iter()
                .map(|e| Param::of(e, drawer))
                .collect(),
            page: registry::PAGE
                .iter()
                .map(|(panel, controls)| PanelLayout {
                    panel: *panel,
                    controls: controls.iter().map(ControlLayout::of).collect(),
                })
                .collect(),
        }
    }
}

/// A bound as it crosses: `null` for an infinity.
fn bound(x: f64) -> Option<f64> {
    x.is_finite().then_some(x)
}

impl Param {
    fn of(e: &Entry, drawer: &Drawer) -> Self {
        let declared = |hard: (f64, f64), quantity, layer, kind, step: CoreStep| {
            let (a, b) = (e.from_si(hard.0), e.from_si(hard.1));
            let (min, max) = if a <= b { (a, b) } else { (b, a) };
            (
                quantity,
                layer,
                kind,
                bound(min),
                bound(max),
                Step::of(step, drawer),
            )
        };
        let (quantity, layer, kind, min, max, step) = match e.field {
            Field::Key(s) => declared(s.hard, s.quantity, s.layer, s.kind, s.step),
            Field::Frame(s) => declared(s.hard, s.quantity, s.layer, s.kind, s.step),
            Field::Protocol(s) => declared(s.hard, s.quantity, s.layer, s.kind, s.step),
            Field::Friction(s) => declared(s.hard, s.quantity, s.layer, s.kind, s.step),
            Field::Weight(s) => declared(s.hard, s.quantity, s.layer, s.kind, s.step),
            Field::Target(s) => declared(s.hard, s.quantity, s.layer, s.kind, s.step),
            Field::Papers(_) => declared(
                PAPER.hard,
                PAPER.quantity,
                PAPER.layer,
                PAPER.kind,
                PAPER.step,
            ),
            // A key's weighing: a mass more than nothing, any front weight,
            // a roller on the covering, from its lip back to its end, which
            // is the key's and not a bound of its own.
            Field::Weighing(which) => (
                CoreQuantity::Length,
                CoreLayer::Design,
                CoreKind::Measured,
                match which {
                    WeighingField::FrontWeight => None,
                    WeighingField::Mass | WeighingField::FromLip => Some(0.0),
                },
                None,
                Step::Continuous,
            ),
        };
        let quantity = match e.field {
            Field::Weighing(WeighingField::Mass | WeighingField::FrontWeight) => CoreQuantity::Mass,
            _ => quantity,
        };
        Self {
            path: e.path.to_owned(),
            unit: match e.unit {
                registry::Unit::Millimeter => SettingUnit::Mm,
                registry::Unit::Gram => SettingUnit::G,
                registry::Unit::GramMillimeter => SettingUnit::Gmm,
            },
            quantity: match quantity {
                CoreQuantity::Length => Quantity::Length,
                CoreQuantity::Mass => Quantity::Mass,
                CoreQuantity::Torque => Quantity::Torque,
            },
            layer: match layer {
                CoreLayer::Design => Layer::Design,
                CoreLayer::Regulation => Layer::Regulation,
                CoreLayer::Protocol => Layer::Protocol,
                CoreLayer::Experiment => Layer::Experiment,
            },
            kind: match kind {
                CoreKind::Actuated => Kind::Actuated,
                CoreKind::Target => Kind::Target,
                CoreKind::Measured => Kind::Measured,
            },
            list: matches!(e.field, Field::Papers(_)),
            min,
            max,
            step,
            set_by: match e.set_by() {
                None => SetBy::Set,
                Some(action) => SetBy::Act { action },
            },
        }
    }
}

impl Step {
    fn of(step: CoreStep, drawer: &Drawer) -> Self {
        match step {
            CoreStep::Continuous => Self::Continuous,
            CoreStep::Drawer => Self::Drawer {
                sizes_mm: drawer.thicknesses().iter().map(|&s| mm(s)).collect(),
            },
        }
    }
}

impl ControlLayout {
    fn of(control: &Control) -> Self {
        match *control {
            Control::Slider { path, label, soft } => Self::Slider {
                path: path.to_owned(),
                label,
                min: soft.min.unwrap_or(f64::NEG_INFINITY),
                max: soft.max.unwrap_or(f64::INFINITY),
                step: soft.step,
            },
            Control::Stack {
                path,
                total,
                stack,
                label,
            } => Self::Stack {
                path: path.to_owned(),
                total: total.to_owned(),
                stack,
                label,
            },
            Control::Form {
                action,
                button,
                fields,
            } => Self::Form {
                action,
                button,
                fields: fields
                    .iter()
                    .map(|f| FieldLayout {
                        argument: f.argument.to_owned(),
                        path: f.path.to_owned(),
                        seed: match f.seed {
                            registry::Seed::Setting(at) => Seed::Setting { at: at.to_owned() },
                            registry::Seed::Reading(procedure) => Seed::Reading {
                                procedure: procedure.into(),
                            },
                        },
                        label: f.label,
                        min: f.soft.min,
                        max: f.soft.max,
                        step: f.soft.step,
                    })
                    .collect(),
            },
            Control::KeyFrictionWeight { label, soft } => Self::KeyFrictionWeight {
                label,
                reads: Procedure::KeyFrictionWeightZeroedOnTheJig,
                action: ActionCode::SetFrictionFromKeyFrictionWeight,
                argument: "kf_g".to_owned(),
                min: soft.min.unwrap_or(0.0),
                max: soft.max.unwrap_or(f64::INFINITY),
                step: soft.step,
            },
            Control::Button { action, label } => Self::Button { action, label },
            Control::Profile {
                action,
                argument,
                label,
            } => Self::Profile {
                action,
                argument: argument.to_owned(),
                label,
                options: Profile::names().map(str::to_owned).collect(),
                at: registry::PROFILE.to_owned(),
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
    /// Millimeters.
    #[serde(rename = "mm")]
    Millimeter,
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
    /// The height of the key's center of mass.
    CenterOfMassHeight,
    /// The friction of the key's bushings.
    KeyFriction,
    /// The dimensions of the key and its frame.
    Geometry,
}

impl From<CoreInput> for Input {
    fn from(i: CoreInput) -> Self {
        match i {
            CoreInput::KeyMass => Self::KeyMass,
            CoreInput::CenterOfMassHeight => Self::CenterOfMassHeight,
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
    /// On Stanwood's jig, leveled, turning without friction about its
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

/// A driven sweep, as it went, for a reading taken on the band.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Swept {
    /// How far the measuring point traveled between two readings,
    /// millimeters.
    pub step_mm: f64,
    /// The readings each way.
    pub samples: u32,
    /// How far the measuring point travels from the top seat to the bottom
    /// one, millimeters.
    pub window_mm: f64,
    /// Where on the stroke the reading fell: how far the measuring point
    /// stood below the top seat, millimeters.
    pub at_mm: f64,
}

/// What a temporary weight zeroes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Zeroing {
    /// The leveled key's front weight on the jig, Stanwood's.
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
    /// Where it stands, millimeters from the balance pin, positive in front.
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
    /// The measuring point, millimeters back from the covering's lip.
    pub measuring_point_from_lip_mm: f64,
    /// The contacts holding the seat the reading is taken at or starts from.
    pub held_by: Vec<Contact>,
    /// The breakaway walk, where the reading is one.
    pub walk: Option<Walk>,
    /// The driven sweep, where the reading is taken on the band.
    pub sweep: Option<Swept>,
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
        // Boxed: a reading read would stand several times the size of one
        // refused.
        conditions: Box<Conditions>,
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
            CoreUnit::Meter => (1000.0, Unit::Millimeter),
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
            conditions: Box::new(Conditions {
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
                sweep: c.sweep.map(|w| Swept {
                    step_mm: mm(w.step),
                    samples: w.samples,
                    window_mm: mm(w.window),
                    at_mm: mm(w.at),
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
            }),
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
    /// The sheet of `instrument`.
    pub fn of(instrument: &Instrument) -> Self {
        let sheet = instrument.sheet();
        // A key with no stroke is told why: read once, for every reading
        // that needs a stroke.
        let no_travel = sheet
            .readings
            .iter()
            .any(|r| r.result == Err(Refused::Walk(WalkRefusal::NoTravel)));
        let riding = if no_travel {
            instrument.riding().map(Riding::from)
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

// ---- the band --------------------------------------------------------------

/// One reading of a curve of the band.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct BandPoint {
    /// How far the measuring point stands below where it stands at the top
    /// seat, millimeters.
    pub travel_mm: f64,
    /// The force on the measuring point that drives the key through that
    /// travel, grams, positive pressing down.
    pub force_g: f64,
}

impl From<&Sample> for BandPoint {
    fn from(s: &Sample) -> Self {
        Self {
            travel_mm: mm(s.travel),
            force_g: s.grams,
        }
    }
}

/// The two curves of the band.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Curves {
    /// How far the measuring point travels from the top seat to the bottom
    /// one, millimeters: the window both curves lie in.
    pub window_mm: f64,
    /// How far it travels between two readings, millimeters.
    pub step_mm: f64,
    /// The down curve, from the first step off the top seat down.
    pub down: Vec<BandPoint>,
    /// The up curve, from the first step off the bottom seat up.
    pub up: Vec<BandPoint>,
}

/// The band: the force on the measuring point through the key's stroke,
/// down and back, the driven sweep's, and the down and up weights read on
/// it; or why it cannot be drawn. Taken when an interface asks for it, as it
/// shows the band, never with every edit.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Band {
    /// The down weight and the up weight read on the band, each read or
    /// refused with why.
    pub readings: Vec<Reading>,
    /// The two curves; `null` where the sweep is refused.
    pub curves: Option<Curves>,
}

impl Band {
    /// The band of `instrument`.
    pub fn of(instrument: &Instrument) -> Self {
        match instrument.band() {
            Ok(banded) => Self {
                readings: vec![
                    Reading::read(&banded.down_weight),
                    Reading::read(&banded.up_weight),
                ],
                curves: Some(Curves {
                    window_mm: mm(banded.band.window),
                    step_mm: mm(banded.band.step),
                    down: banded.band.down.iter().map(BandPoint::from).collect(),
                    up: banded.band.up.iter().map(BandPoint::from).collect(),
                }),
            },
            Err(why) => {
                let riding = matches!(why, Refused::Sweep(SweepRefusal::NoTravel))
                    .then(|| instrument.riding().map(Riding::from))
                    .flatten();
                let why = Refusal::reading(&why, riding);
                Self {
                    readings: ProcedureId::BAND
                        .iter()
                        .map(|&p| Reading::Refused {
                            procedure: p.into(),
                            why: why.clone(),
                        })
                        .collect(),
                    curves: None,
                }
            }
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
        /// By how much, millimeters.
        by_mm: f64,
    },
    /// The back rail cloth reaches in front of where the tail's chamfer
    /// begins.
    ClothInFrontOfChamfer {
        /// By how much, millimeters.
        by_mm: f64,
    },
    /// The back rail cloth reaches past the end of the key.
    ClothBehindKey {
        /// By how much, millimeters.
        by_mm: f64,
    },
    /// A punching stands partly off the flat of the key's underside.
    PunchingOffUnderside {
        /// Which punching.
        rail: Part,
        /// By how much, millimeters.
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
        /// How far above its balance punching it stands, millimeters.
        lift_mm: f64,
    },
    /// Off its balance punching, held by what it touches.
    LiftsOffBalancePunching {
        /// The stops it touches, in the order the key's contacts are
        /// declared.
        touching: Vec<Contact>,
        /// Whether they need its friction to hold it there.
        needs_friction: bool,
        /// How far above its balance punching it stands, millimeters.
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
    /// The gap at the front punching, read at its center, millimeters;
    /// `null` where no underside passes over its center.
    pub gap_front_mm: Option<f64>,
    /// The gap at the back rail cloth, millimeters; `null` where no feature
    /// of the two can meet.
    pub gap_back_mm: Option<f64>,
    /// How far the measuring point stands below where it stands with the
    /// key seated at the top of its stroke, millimeters: where the weight
    /// has left the key on the band's travel; `null` where the key cannot
    /// be seated at the top.
    pub travel_mm: Option<f64>,
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
    /// Where it bears on the cloth, millimeters behind the balance pin.
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

/// A punching read at its center whose drawn edge overlaps the key.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct EdgeOverlap {
    /// Which punching.
    pub pair: Contact,
    /// How far its drawn edge stands into the key, millimeters: what its
    /// felt would compress under a key out of level on it.
    pub overlap_mm: f64,
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
    /// The last edit refused since one was taken, and why; `null` where
    /// none was.
    pub refused_edit: Option<RefusedEdit>,
}

impl Diagnostics {
    /// The diagnostics of `instrument`.
    pub fn of(instrument: &Instrument) -> Self {
        let report = instrument.seat_report();
        Self {
            seat_refused: report.as_ref().err().map(Refusal::seat),
            riding: report
                .as_ref()
                .ok()
                .and_then(|r| r.riding)
                .map(Riding::from),
            faults: instrument.faults().iter().map(Fault::from).collect(),
            seats: report.as_ref().ok().and_then(Seats::of),
            touch: instrument.touch().map(|t| Touch {
                verdict: (&t.verdict).into(),
                gap_front_mm: t.gap_front_mm,
                gap_back_mm: t.gap_back_mm,
                travel_mm: t.travel_mm,
            }),
            edge_overlaps: instrument
                .edge_overlaps()
                .into_iter()
                .map(|(pair, overlap)| EdgeOverlap {
                    pair: pair.into(),
                    overlap_mm: mm(overlap),
                })
                .collect(),
            refused_edit: instrument.refused_edit().map(|r| RefusedEdit {
                edit: (&r.edit).into(),
                why: EditRefusal::of(instrument, &r.edit, &r.why),
            }),
        }
    }
}

// ---- the state -------------------------------------------------------------

/// Where one body's frame stands: its origin, millimeters, and the cosine
/// and sine of its rotation. The view applies it as given and computes none.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Pose {
    /// The origin, millimeters.
    pub origin_mm: [f32; 2],
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
    /// Where, millimeters, in the body's frame.
    pub at_mm: [f32; 2],
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

/// Where the key stands on its frame, as the view draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum KeyPlace {
    /// An equilibrium places it where it is drawn.
    Placed,
    /// No equilibrium has held it since it was put on its frame: it is
    /// drawn level on its balance punching, which places nothing.
    Unplaced,
    /// The weight on its measuring point lifts it off its frame: it is
    /// drawn where its last equilibrium left it, which no longer holds it.
    LiftedOff,
}

/// A contact the view marks, in the keybed's frame: one active row, where
/// the contact rows put its two points, not where a body's pose carries
/// them, with its normal and what it carries.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct MarkedContact {
    /// Which contact.
    pub pair: Contact,
    /// Where the key's surface is, millimeters.
    pub on_key_mm: [f32; 2],
    /// Where the frame's surface is, millimeters.
    pub on_frame_mm: [f32; 2],
    /// The signed gap between them, millimeters: negative where the drawn
    /// parts overlap.
    pub gap_mm: f32,
    /// Closed, open or overlapping.
    pub state: ContactState,
    /// The row's unit normal, from the frame's surface toward the key's.
    pub normal: [f32; 2],
    /// The force the row pushed the key with in the last equilibrium,
    /// grams-force: its multiplier, one admissible split of the loads where
    /// friction or redundant rows share them, never a verdict; zero on a
    /// row that carried nothing.
    pub force_g: f32,
}

/// Where each body stands, and what the view marks on them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct State {
    /// One per body, in body order: the ground, the slide the key lifts off
    /// its balance punching on, the key.
    pub poses: Vec<Pose>,
    /// Where the key stands on its frame, as it is drawn: placed by an
    /// equilibrium, or not and why not.
    pub place: KeyPlace,
    /// The measuring point, on the key: where the engine weighs.
    pub measuring_point: Marker,
    /// The contacts within a millimeter of closing, where the key is drawn;
    /// where no equilibrium places it, only those where its drawn parts
    /// overlap the frame's.
    pub contacts: Vec<MarkedContact>,
}

impl State {
    /// The state of `instrument`.
    pub fn of(instrument: &Instrument) -> Self {
        let m = instrument.measuring_point_on_key();
        Self {
            poses: instrument
                .poses()
                .into_iter()
                .map(|p| Pose {
                    origin_mm: [mm32(p.origin.x), mm32(p.origin.y)],
                    cos: p.cos as f32,
                    sin: p.sin as f32,
                })
                .collect(),
            place: match instrument.key_place() {
                CoreKeyPlace::Placed => KeyPlace::Placed,
                CoreKeyPlace::Unplaced => KeyPlace::Unplaced,
                CoreKeyPlace::LiftedOff => KeyPlace::LiftedOff,
            },
            measuring_point: Marker {
                body: body_index(KEY.0),
                at_mm: [mm32(m.x), mm32(m.y)],
            },
            contacts: instrument
                .contacts()
                .into_iter()
                .map(|c| MarkedContact {
                    pair: c.code.into(),
                    on_key_mm: [mm32(c.on_key.x), mm32(c.on_key.y)],
                    on_frame_mm: [mm32(c.on_frame.x), mm32(c.on_frame.y)],
                    gap_mm: mm32(c.gap),
                    state: match c.state() {
                        CoreContactState::Closed => ContactState::Closed,
                        CoreContactState::Open => ContactState::Open,
                        CoreContactState::Overlapping => ContactState::Overlapping,
                    },
                    normal: [c.normal.x as f32, c.normal.y as f32],
                    force_g: (c.force / G * 1000.0) as f32,
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

/// What a part is made of; the view picks its color.
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
    BushingCloth,
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
    /// Milled by a cutter turning about a vertical axis, moved along the
    /// key: a mortise, or the round balance hole. Its outline is its section
    /// through its middle, swept across the keyboard to its width, the
    /// cutter's diameter, its two ends rounded to half of it.
    Milled,
    /// Oval in plan, standing upright: the front rail pin's head. Its
    /// outline is its section through its middle along the key, the long
    /// axis, the same either side of its middle, its tip rounded; turned
    /// about that middle, it is squeezed across the keyboard to its width at
    /// its foot.
    Oval,
}

/// How the number that places a stretch of an outline is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum How {
    /// Measured on a real part.
    Measured,
    /// Read from a published source.
    Published,
    /// Traced from a figure.
    Traced,
    /// Derived by a rule from other numbers.
    Derived,
    /// Borrowed from another instrument: read from a record of an
    /// instrument the profile does not describe.
    Borrowed,
    /// Assumed, pending measurement.
    Assumed,
    /// Set on the bench since the instrument was built: no longer the
    /// number its profile gives.
    SetOnTheBench,
}

impl From<Known> for How {
    fn from(k: Known) -> Self {
        match k {
            Known::Measured => Self::Measured,
            Known::Published => Self::Published,
            Known::Traced => Self::Traced,
            Known::Derived => Self::Derived,
            Known::Borrowed => Self::Borrowed,
            Known::Assumed => Self::Assumed,
        }
    }
}

/// Where a stretch of an outline comes from: the number of the design that
/// places it, and how that number is known, by the provenance the
/// instrument's profile keeps.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Source {
    /// Its first segment.
    pub from: u16,
    /// One past its last.
    pub to: u16,
    /// The number that places it, as the engine's validation names it.
    pub number: String,
    /// How that number is known.
    pub how: How,
    /// The file that records or states it, relative to the repository's
    /// root; `null` where it was set on the bench.
    pub at: Option<String>,
}

impl Source {
    /// The stretch `range` of an outline of `instrument`'s, placed by the
    /// number `name`: known as its profile knows it while it holds the
    /// profile's value, set on the bench once it does not. A paper is one
    /// the drawer holds, known as the drawer is.
    fn of(
        instrument: &Instrument,
        numbers: &[(&'static str, f64)],
        range: &core::ops::Range<u16>,
        name: &'static str,
    ) -> Self {
        let profile = instrument.profile();
        let name = if name == PAPER.name { DRAWER } else { name };
        let held = numbers.iter().find(|(n, _)| *n == name).map(|(_, v)| *v);
        let started = profile.provenance(name).map(|p| p.value);
        let edited = name != DRAWER
            && matches!((held, started), (Some(h), Some(s)) if h.to_bits() != s.to_bits());
        let (how, at) = match profile.known(name) {
            _ if edited => (How::SetOnTheBench, None),
            Some((known, file)) => (known.into(), Some(file)),
            None => (How::SetOnTheBench, None),
        };
        Self {
            from: range.start,
            to: range.end,
            number: name.to_owned(),
            how,
            at,
        }
    }
}

/// One part, in millimeters, in its own body's frame, arcs cut into lines.
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
    /// For a paper punching, its own thickness, millimeters: the trade knows
    /// a paper by its color, and its color by its thickness.
    pub paper_mm: Option<f32>,
    /// For a paper punching, the color its record says a paper of that
    /// thickness is supplied in (`data/params/punching-papers.json`); none
    /// for a part that is no paper, or a paper the drawer does not hold.
    pub paper_color: Option<PaperColor>,
    /// How the outline becomes a solid.
    pub sweep: Sweep,
    /// Extrusion across the sagittal plane, a milled cut's cutter's
    /// diameter, or an oval part's width at its foot, its short axis but on
    /// a pin that is all tip, millimeters; zero when turned.
    pub width_mm: f32,
    /// Where the extrusion's middle stands across the keyboard, millimeters
    /// from the sagittal plane, positive toward the bass, the player's left:
    /// zero but for the parts beside the plane, which the section does not
    /// cut, the balance mortise's bushing and the key button's side holes;
    /// zero when turned, milled or oval.
    pub across_mm: f32,
    /// Whether the sagittal plane cuts it, so that the section draws it in
    /// its plane: every part but those beside it.
    pub is_in_the_section: bool,
    /// Where a turned part's axis stands along its body's `x`, millimeters;
    /// zero otherwise.
    pub axis_x_mm: f32,
    /// Flat `[x0, y0, x1, y1, …]`, millimeters, in the body's frame; for a
    /// turned part, its profile in radius and height.
    pub points_mm: Vec<f32>,
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
    /// The scene of `instrument`.
    pub fn of(instrument: &Instrument) -> Self {
        let numbers = instrument.numbers();
        Self {
            outlines: instrument
                .outlines()
                .into_iter()
                .map(|o| {
                    let (sweep, width, across, axis_x) = match o.sweep {
                        CoreSweep::Extruded { width, across } => {
                            (Sweep::Extruded, mm32(width), mm32(across), 0.0)
                        }
                        CoreSweep::Revolved { axis_x } => (Sweep::Revolved, 0.0, 0.0, mm32(axis_x)),
                        CoreSweep::Milled { width } => (Sweep::Milled, mm32(width), 0.0, 0.0),
                        CoreSweep::Oval { width } => (Sweep::Oval, mm32(width), 0.0, 0.0),
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
                            CoreMaterial::BushingCloth => Material::BushingCloth,
                            CoreMaterial::Paper { .. } => Material::Paper,
                        },
                        paper_mm: match o.material {
                            CoreMaterial::Paper { thickness } => Some(mm32(thickness)),
                            _ => None,
                        },
                        paper_color: match o.material {
                            CoreMaterial::Paper { thickness } => {
                                instrument.profile().paper_color(thickness)
                            }
                            _ => None,
                        },
                        sweep,
                        width_mm: width,
                        across_mm: across,
                        is_in_the_section: o.sweep.is_in_the_section(),
                        axis_x_mm: axis_x,
                        points_mm: o
                            .points
                            .iter()
                            .flat_map(|p| [mm32(p.x), mm32(p.y)])
                            .collect(),
                        segs: o.segs,
                        sources: o
                            .placed_by
                            .iter()
                            .map(|(range, name)| Source::of(instrument, &numbers, range, name))
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}

// ---- what an edit answers --------------------------------------------------

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
        /// What was asked of it, millimeters, as it was asked.
        wanted_mm: f64,
        /// What keeps it out of reach.
        limited_by: Limit,
        /// How far out of reach, in its own reading, millimeters.
        shortfall_mm: f64,
        /// The most paper the search lays on a stack, millimeters.
        search_mm: f64,
    },
    /// The key could not be seated at one of the stacks the search tried.
    NoSeat {
        /// Why.
        why: Refusal,
    },
}

impl RegulationRefusal {
    /// Why a regulation to a key height and a key dip, millimeters as they
    /// were asked, was not laid; a target that is not a number never
    /// reaches the solve ([`Problem::NotFinite`]).
    pub fn of(why: &NotRegulated, key_height_mm: f64, key_dip_mm: f64) -> Self {
        match why {
            NotRegulated::Infeasible {
                target,
                limited_by,
                shortfall_m,
            } => Self::RegulationInfeasible {
                target: (*target).into(),
                wanted_mm: match target {
                    CoreTarget::KeyHeight => key_height_mm,
                    CoreTarget::KeyDip => key_dip_mm,
                },
                limited_by: (*limited_by).into(),
                shortfall_mm: mm(*shortfall_m),
                search_mm: mm(STACK_SEARCH),
            },
            NotRegulated::Unsolvable(Unsolvable::NoSeat(seat)) => Self::NoSeat {
                why: Refusal::seat(seat),
            },
            NotRegulated::Unsolvable(Unsolvable::NotFinite(target)) => {
                unreachable!("a {target:?} that is not a number reached the solve")
            }
        }
    }
}

/// Why a weighing of the key was refused: the numbers typed are no key's.
/// The key's weighing stays as it was.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum WeighingRefusal {
    /// The jig's roller would stand off the covering's top.
    RollerOffTheCovering {
        /// Where it was said to stand, millimeters back from the lip.
        from_lip_mm: f64,
        /// How far the covering runs back from its lip, millimeters.
        covering_mm: f64,
    },
    /// The two readings stand the key's center of mass off the key.
    CenterOfMassOffTheKey {
        /// Where, millimeters from the balance pin, positive in front.
        at_mm: f64,
    },
}

impl WeighingRefusal {
    /// Why a weighing of `instrument`'s key was refused; a number that is
    /// not one, or a mass of nothing, never reaches the weighing
    /// ([`Problem`]).
    pub fn of(instrument: &Instrument, why: NotAWeighing) -> Self {
        match why {
            NotAWeighing::RollerOffTheCovering { from_lip_m } => Self::RollerOffTheCovering {
                from_lip_mm: mm(from_lip_m),
                covering_mm: mm(instrument.key().keytop_length),
            },
            NotAWeighing::CenterOfMassOffTheKey { at_m } => {
                Self::CenterOfMassOffTheKey { at_mm: mm(at_m) }
            }
            NotAWeighing::NotANumber | NotAWeighing::MassNotPositive => {
                unreachable!("{why:?} reached the weighing")
            }
        }
    }
}

/// Why no bushings read a key friction weight typed. The bushings stay as
/// they were.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum FrictionRefusal {
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
    /// `instrument`: a protocol refused for want of a stroke says how the
    /// key rides. A number that is not one never reaches the protocol
    /// ([`Problem`]).
    pub fn of(instrument: &Instrument, typed_g: f64, why: &BushingsRefusal) -> Self {
        match why {
            BushingsRefusal::Procedure(refused) => {
                let riding = match refused {
                    Refused::Walk(WalkRefusal::NoTravel) => instrument.riding().map(Riding::from),
                    _ => None,
                };
                Self::ProtocolRefused {
                    why: Refusal::reading(refused, riding),
                }
            }
            BushingsRefusal::UnderNoFriction { least_g } => Self::UnderNoFriction {
                typed_g,
                least_g: *least_g,
            },
            BushingsRefusal::BeyondTheSearch { most_g } => Self::BeyondTheSearch {
                typed_g,
                most_g: *most_g,
                holds_g: FRICTION_SEARCH_G,
            },
            BushingsRefusal::NotANumber => unreachable!("a weight that is not a number"),
        }
    }
}

/// Why an edit would make what no part could be built as, or names what
/// the instrument has not: the validation layer's problems, in the trade's
/// units with the unit in each name, and the edit's own.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Problem {
    /// A value that is not a number, or an infinity.
    NotFinite {
        /// The path it was set at, or the action's argument it was typed
        /// as; the domain's name of a dimension no path sets.
        param: String,
    },
    /// A value outside its setting's hard range.
    OutOfRange {
        /// The path.
        param: String,
        /// The value asked for, in the path's unit.
        value: f64,
        /// The least it admits; `null` where nothing bounds it below.
        min: Option<f64>,
        /// The most it admits; `null` where nothing bounds it above.
        max: Option<f64>,
    },
    /// A paper no drawer holds.
    NotInTheDrawer {
        /// The thickness asked for, millimeters.
        thickness_mm: f64,
        /// What the drawer holds, millimeters.
        drawer_mm: Vec<f64>,
    },
    /// A key weighed at no mass, or less.
    MassNotPositive {
        /// The mass typed, grams.
        mass_g: f64,
    },
    /// Two parts on the key's top overlap, the clearance cut in it runs into
    /// one of them or under the capstan's head, its thinned tail runs on
    /// under its back check block, or its chamfer runs past its front: the
    /// outline crosses itself.
    OutlineCrossesItself {
        /// The part behind.
        back: Part,
        /// The part in front.
        front: Part,
        /// How far the front end of the part behind stands in front of the
        /// back end of the part in front, millimeters along the key: how far
        /// they overlap, where neither runs past the other's far end.
        by_mm: f64,
    },
    /// A feature off the keystick's wood where it must stand.
    FeatureOutsideKeystick {
        /// The feature.
        part: Part,
        /// How far off, millimeters along the key.
        by_mm: f64,
    },
    /// The capstan stands wholly behind the back check block.
    CapstanBehindBackCheckBlock {
        /// How far forward it must come to clear the block, millimeters
        /// along the key.
        by_mm: f64,
    },
    /// The capstan stands on the thinned tail, or across the step down to
    /// it, and not on the keystick's full height.
    CapstanOffTheFullHeight {
        /// How far forward it must come for its head to stand wholly over
        /// the full height, millimeters along the key.
        by_mm: f64,
    },
    /// The capstan's dome is narrower than its head.
    CapstanDomeNarrowerThanItsHead {
        /// The dome's radius, millimeters.
        dome_mm: f64,
        /// The head's radius, millimeters.
        head_mm: f64,
    },
    /// The capstan stands in a glued block.
    CapstanInBlock {
        /// The block.
        block: Part,
        /// How far its head stands into it, millimeters along the key.
        by_mm: f64,
    },
    /// The clearance in the keystick's top is no cut: its front end not in
    /// front of its back end, or its depth nothing or less.
    ClearanceNotACut {
        /// Its front end less its back end, millimeters along the key.
        length_mm: f64,
        /// Its depth below the keystick's full top, millimeters.
        depth_mm: f64,
    },
    /// The clearance in the keystick's top is cut through it.
    ClearanceThroughKeystick {
        /// Its depth below the keystick's full top, millimeters.
        depth_mm: f64,
        /// The wood under its back end, where the keystick is thinnest under
        /// it: its full height, less the chamfer's rise where that runs
        /// under it, millimeters.
        wood_mm: f64,
        /// How far it reaches past the keystick's underside, millimeters.
        by_mm: f64,
    },
    /// The key button's two rounded ends reach past each other along its
    /// top.
    CapsuleEndsOverlap {
        /// How far, millimeters along the key.
        by_mm: f64,
    },
    /// A part of the balance mortise does not fit where it is cut or set.
    DoesNotFit {
        /// The part that does not fit.
        part: Part,
        /// What it does not fit in.
        within: Part,
        /// By how much, millimeters.
        by_mm: f64,
    },
    /// The balance mortise's cutter does not travel along the key: the slot
    /// is round, with no straight wall for its bushing.
    BushingWithoutWall,
    /// The front mortise is cut through the top of the keystick.
    MortiseThroughKeyTop {
        /// Its depth, millimeters.
        depth_mm: f64,
        /// The keystick's wood it is cut into, millimeters.
        wood_mm: f64,
        /// How far it reaches past the wood's top, millimeters.
        by_mm: f64,
    },
    /// The measuring point stands off the covering.
    MeasuringPointOffCovering {
        /// Where it was asked for, millimeters back from the lip.
        from_lip_mm: f64,
        /// How far the covering runs back from its lip, millimeters.
        covering_mm: f64,
    },
    /// The instrument has no such path.
    UnknownPath {
        /// The path asked for.
        path: String,
    },
    /// No profile the bench ships has that name.
    UnknownProfile {
        /// The name asked for.
        profile: String,
    },
    /// The path is reached through an action, not set.
    SetByAction {
        /// The path.
        path: String,
        /// The action that sets it.
        action: ActionCode,
    },
    /// A stack holds no paper of that index.
    NoSuchPaper {
        /// The stack.
        stack: Stack,
        /// The index asked for, from the bottom.
        index: u32,
        /// How many papers it holds.
        papers: u32,
    },
    /// What was sent is no action the instrument knows.
    NotAnAction,
}

impl Problem {
    /// `p`, a domain problem, in the trade's units, each setting by its
    /// path, a paper against the drawer it is not in.
    pub fn of(p: &CoreProblem, drawer: &Drawer) -> Self {
        let named = |param: &'static str| registry::by_param(param).map_or(param, |e| e.path);
        match *p {
            CoreProblem::NotFinite { param } => Self::NotFinite {
                param: named(param).to_owned(),
            },
            CoreProblem::OutOfRange {
                param,
                value,
                min,
                max,
            } => {
                let unit = |x: f64| registry::by_param(param).map_or(x, |e| e.from_si(x));
                let (a, b) = (unit(min), unit(max));
                let (min, max) = if a <= b { (a, b) } else { (b, a) };
                Self::OutOfRange {
                    param: named(param).to_owned(),
                    value: unit(value),
                    min: bound(min),
                    max: bound(max),
                }
            }
            CoreProblem::NotInTheDrawer { thickness, .. } => Self::NotInTheDrawer {
                thickness_mm: mm(thickness),
                drawer_mm: drawer.thicknesses().iter().map(|&p| mm(p)).collect(),
            },
            CoreProblem::MassNotPositive { mass } => Self::MassNotPositive {
                mass_g: mass * 1000.0,
            },
            CoreProblem::OutlineCrossesItself { back, front, by } => Self::OutlineCrossesItself {
                back: back.into(),
                front: front.into(),
                by_mm: mm(by),
            },
            CoreProblem::FeatureOutsideKeystick { part, by } => Self::FeatureOutsideKeystick {
                part: part.into(),
                by_mm: mm(by),
            },
            CoreProblem::CapstanBehindBackCheckBlock { by } => {
                Self::CapstanBehindBackCheckBlock { by_mm: mm(by) }
            }
            CoreProblem::CapstanOffTheFullHeight { by } => {
                Self::CapstanOffTheFullHeight { by_mm: mm(by) }
            }
            CoreProblem::CapstanDomeNarrowerThanItsHead { dome, head } => {
                Self::CapstanDomeNarrowerThanItsHead {
                    dome_mm: mm(dome),
                    head_mm: mm(head),
                }
            }
            CoreProblem::CapstanInBlock { block, by } => Self::CapstanInBlock {
                block: block.into(),
                by_mm: mm(by),
            },
            CoreProblem::ClearanceNotACut { length, depth } => Self::ClearanceNotACut {
                length_mm: mm(length),
                depth_mm: mm(depth),
            },
            CoreProblem::ClearanceThroughKeystick { depth, wood } => {
                Self::ClearanceThroughKeystick {
                    depth_mm: mm(depth),
                    wood_mm: mm(wood),
                    by_mm: mm(depth - wood),
                }
            }
            CoreProblem::CapsuleEndsOverlap { by } => Self::CapsuleEndsOverlap { by_mm: mm(by) },
            CoreProblem::DoesNotFit { part, within, by } => Self::DoesNotFit {
                part: part.into(),
                within: within.into(),
                by_mm: mm(by),
            },
            CoreProblem::BushingWithoutWall => Self::BushingWithoutWall,
            CoreProblem::MortiseThroughKeyTop { depth, wood } => Self::MortiseThroughKeyTop {
                depth_mm: mm(depth),
                wood_mm: mm(wood),
                by_mm: mm(depth - wood),
            },
            CoreProblem::MeasuringPointOffCovering { from_lip, covering } => {
                Self::MeasuringPointOffCovering {
                    from_lip_mm: mm(from_lip),
                    covering_mm: mm(covering),
                }
            }
        }
    }
}

/// Why an edit was refused, the instrument left as it was: one code among
/// the problems, the regulation's refusals, the weighing's and the
/// bushings', which share none.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(untagged)]
pub enum EditRefusal {
    /// What no part could be built as, or what the instrument has not.
    Problem(Problem),
    /// A regulation not laid.
    Regulation(RegulationRefusal),
    /// A weighing no key has.
    Weighing(WeighingRefusal),
    /// A key friction weight no bushings read.
    Friction(FrictionRefusal),
}

impl EditRefusal {
    /// Why `edit` was refused on `instrument`, `why` being the instrument's
    /// own answer.
    pub fn of(instrument: &Instrument, edit: &CoreEdit, why: &CoreEditRefusal) -> Self {
        match why {
            CoreEditRefusal::Problem(p) => {
                Self::Problem(Problem::of(p, &instrument.frame().drawer))
            }
            CoreEditRefusal::UnknownPath { path } => {
                Self::Problem(Problem::UnknownPath { path: path.clone() })
            }
            CoreEditRefusal::UnknownProfile { profile } => Self::Problem(Problem::UnknownProfile {
                profile: profile.clone(),
            }),
            CoreEditRefusal::SetByAction { path, action } => Self::Problem(Problem::SetByAction {
                path: (*path).to_owned(),
                action: *action,
            }),
            CoreEditRefusal::NoSuchPaper {
                stack,
                index,
                papers,
            } => Self::Problem(Problem::NoSuchPaper {
                stack: *stack,
                index: *index,
                papers: u32::try_from(*papers).unwrap_or(u32::MAX),
            }),
            CoreEditRefusal::Regulation(not) => {
                let (height, dip) = match edit {
                    CoreEdit::Act(Action::Regulate {
                        key_height_mm,
                        key_dip_mm,
                    }) => (*key_height_mm, *key_dip_mm),
                    _ => (f64::NAN, f64::NAN),
                };
                Self::Regulation(RegulationRefusal::of(not, height, dip))
            }
            CoreEditRefusal::Weighing(not) => Self::Weighing(WeighingRefusal::of(instrument, *not)),
            CoreEditRefusal::Friction(not) => {
                let typed_g = match edit {
                    CoreEdit::Act(Action::SetFrictionFromKeyFrictionWeight { kf_g }) => *kf_g,
                    _ => f64::NAN,
                };
                Self::Friction(FrictionRefusal::of(instrument, typed_g, not))
            }
        }
    }
}

/// An edit, as a refusal names it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "edit", rename_all = "snake_case")]
pub enum Edit {
    /// A setting set by its path.
    Set {
        /// The path, as it was sent.
        path: String,
        /// The value, in the path's unit; `null` where it is not a number.
        value: Option<f64>,
    },
    /// An action.
    Act {
        /// The action.
        action: Action,
    },
}

impl From<&CoreEdit> for Edit {
    fn from(edit: &CoreEdit) -> Self {
        match edit {
            CoreEdit::Set { path, value } => Self::Set {
                path: path.clone(),
                value: bound(*value),
            },
            CoreEdit::Act(action) => Self::Act {
                action: action.clone(),
            },
        }
    }
}

/// The last edit refused, and why: what the diagnostics keep until an edit
/// is taken.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct RefusedEdit {
    /// The edit.
    pub edit: Edit,
    /// Why it was refused.
    pub why: EditRefusal,
}

/// What an edit answers.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Acted {
    /// It was taken.
    Done,
    /// The regulation laid its papers.
    Laid {
        /// The paper laid under the balance punching, millimeters.
        balance_mm: f64,
        /// The paper laid on the front punching, millimeters.
        front_mm: f64,
        /// The key height the papers give, millimeters.
        key_height_mm: f64,
        /// The key dip the papers give, millimeters.
        key_dip_mm: f64,
        /// The key height asked for, millimeters, as it was asked.
        asked_key_height_mm: f64,
        /// The key dip asked for, millimeters, as it was asked.
        asked_key_dip_mm: f64,
    },
    /// Refused, and nothing changed.
    Refused {
        /// Why.
        why: EditRefusal,
    },
}

impl Acted {
    /// What `instrument` answered `edit` with.
    fn of(
        instrument: &Instrument,
        edit: &CoreEdit,
        answer: Result<Outcome, CoreEditRefusal>,
    ) -> Self {
        match answer {
            Ok(Outcome::Done) => Self::Done,
            Ok(Outcome::Laid(achieved)) => {
                let (asked_key_height_mm, asked_key_dip_mm) = match edit {
                    CoreEdit::Act(Action::Regulate {
                        key_height_mm,
                        key_dip_mm,
                    }) => (*key_height_mm, *key_dip_mm),
                    _ => (f64::NAN, f64::NAN),
                };
                let total = |stack| mm(instrument.papers(stack).iter().sum());
                Self::Laid {
                    balance_mm: total(Stack::Balance),
                    front_mm: total(Stack::Front),
                    key_height_mm: mm(achieved.key_height),
                    key_dip_mm: mm(achieved.key_dip),
                    asked_key_height_mm,
                    asked_key_dip_mm,
                }
            }
            Err(why) => Self::Refused {
                why: EditRefusal::of(instrument, edit, &why),
            },
        }
    }
}

/// Sets `path` to `value`, in the path's unit, on `instrument`: done, or
/// why not, nothing changed.
pub fn set(instrument: &mut Instrument, path: &str, value: f64) -> Acted {
    let answer = instrument.set(path, value).map(|()| Outcome::Done);
    let edit = CoreEdit::Set {
        path: path.to_owned(),
        value,
    };
    Acted::of(instrument, &edit, answer)
}

/// Runs `action` on `instrument`: done, the papers a regulation laid and
/// what they give, or why not, nothing changed.
pub fn act(instrument: &mut Instrument, action: Action) -> Acted {
    let edit = CoreEdit::Act(action.clone());
    let answer = instrument.act(action);
    Acted::of(instrument, &edit, answer)
}

/// Runs the action `json` describes ([`Action`]) on `instrument`; JSON that
/// is no action is refused as [`Problem::NotAnAction`], nothing changed.
pub fn act_json(instrument: &mut Instrument, json: &str) -> Acted {
    match serde_json::from_str::<Action>(json) {
        Ok(action) => act(instrument, action),
        Err(_) => Acted::Refused {
            why: EditRefusal::Problem(Problem::NotAnAction),
        },
    }
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
        // The documents, the actions an interface sends, and what an edit
        // answers.
        generate.visit::<Settings>();
        generate.visit::<Registry>();
        generate.visit::<Sheet>();
        generate.visit::<Band>();
        generate.visit::<Diagnostics>();
        generate.visit::<State>();
        generate.visit::<Scene>();
        generate.visit::<Action>();
        generate.visit::<Acted>();
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
    use pa_core::regulation::Targets as CoreTargets;

    /// The instrument with its back rail raised until its cloth holds the
    /// tail 5 mm above the balance bearing: its stops fight, and the key
    /// rides on them.
    fn wedged() -> Instrument {
        let mut bench = Instrument::weighed_section();
        let frame = bench.frame();
        let height = frame.balance_bearing_height() + 0.005 - frame.back_rail_cloth;
        bench
            .set(path::BACK_RAIL_HEIGHT, height * 1000.0)
            .expect("a back rail raised");
        bench
    }

    /// `value` as JSON, a string's quotes included.
    fn written<T: Serialize>(value: &T) -> String {
        json(value)
    }

    /// The value at `path` in the settings as they cross, by walking the
    /// JSON as an interface walks it.
    fn at(settings: &Settings, path: &str) -> serde_json::Value {
        let mut v: serde_json::Value =
            serde_json::from_str(&json(settings)).expect("settings are JSON");
        for key in path.split('.') {
            v = v
                .get(key)
                .cloned()
                .unwrap_or_else(|| panic!("no {key} on the way to {path}"));
        }
        v
    }

    /// Consistency: **the scene sends where each part stands across the
    /// keyboard, and whether the section cuts it**: on the measured section,
    /// the bushing's legs down the slot's walls 2.35 mm from the sagittal
    /// plane, and the key button's side holes, with the bushing's legs out
    /// through them, 7 mm from it, between the slot's wall at 3 mm and the
    /// key button's side at 11 mm, each on the treble side then on the bass
    /// side, out of the section; every other part centered on the plane, in
    /// it.
    #[test]
    fn the_scene_sends_where_each_part_stands_across_the_keyboard() {
        let scene = Scene::of(&Instrument::weighed_section());
        let is_beside = |outline: &&Outline| {
            matches!(
                outline.code,
                Part::BalanceMortiseBushing | Part::KeyButtonSideHole
            )
        };
        let (aside, centered): (Vec<&Outline>, Vec<&Outline>) =
            scene.outlines.iter().partition(is_beside);
        let across = |code: Part| -> Vec<f32> {
            aside
                .iter()
                .filter(|o| o.code == code)
                .map(|o| o.across_mm)
                .collect()
        };
        // Millimeters in single precision, a few of its ulps at each.
        let near = |found: Vec<f32>, expected: &[f32]| {
            found.len() == expected.len()
                && found.iter().zip(expected).all(|(found, expected)| {
                    (found - expected).abs() <= 4.0 * f32::EPSILON * expected.abs()
                })
        };
        assert!(
            near(
                across(Part::BalanceMortiseBushing),
                &[-2.35, 2.35, -7.0, 7.0]
            ),
            "{:?}",
            across(Part::BalanceMortiseBushing)
        );
        assert!(
            near(across(Part::KeyButtonSideHole), &[-7.0, 7.0]),
            "{:?}",
            across(Part::KeyButtonSideHole)
        );
        assert!(aside.iter().all(|o| !o.is_in_the_section));
        assert!(centered
            .iter()
            .all(|o| o.across_mm == 0.0 && o.is_in_the_section));
    }

    /// Consistency: **the mortises and the balance hole cross milled as wide
    /// as their cutters**, on the measured section's key: the balance
    /// mortise and the front mortise 6 mm, the balance hole 3.4 mm.
    #[test]
    fn the_mortises_and_the_balance_hole_cross_milled_as_wide_as_their_cutters() {
        let scene = Scene::of(&Instrument::weighed_section());
        // Millimeters in single precision, a few of its ulps at each.
        let near =
            |found: f32, expected: f32| (found - expected).abs() <= 4.0 * f32::EPSILON * expected;
        for (code, cutter_mm) in [
            (Part::BalanceMortise, 6.0),
            (Part::FrontMortise, 6.0),
            (Part::BalanceHole, 3.4),
        ] {
            let outline = scene
                .outlines
                .iter()
                .find(|outline| outline.code == code)
                .expect("a milled cut");
            assert_eq!(
                (outline.sweep, outline.across_mm, outline.axis_x_mm),
                (Sweep::Milled, 0.0, 0.0)
            );
            assert!(
                near(outline.width_mm, cutter_mm),
                "{code:?}: {} mm",
                outline.width_mm
            );
        }
    }

    /// Consistency: **the front rail pin crosses oval**, on the measured
    /// section's frame: its outline its section along the key, 5 mm long,
    /// its width across the keyboard the 3.4 mm of the oval's short axis.
    #[test]
    fn the_front_rail_pin_crosses_oval() {
        let scene = Scene::of(&Instrument::weighed_section());
        let pin = scene
            .outlines
            .iter()
            .find(|outline| outline.code == Part::FrontRailPin)
            .expect("a front rail pin");
        assert_eq!(
            (pin.sweep, pin.across_mm, pin.axis_x_mm),
            (Sweep::Oval, 0.0, 0.0)
        );
        let along_mm: Vec<f32> = pin.points_mm.iter().step_by(2).copied().collect();
        let (back_mm, front_mm) = along_mm
            .iter()
            .fold((f32::MAX, f32::MIN), |(back_mm, front_mm), &x| {
                (back_mm.min(x), front_mm.max(x))
            });
        // Millimeters in single precision, a few of its ulps at each: the
        // head's two ends near 225 mm, its width a single rounding of 3.4 mm.
        assert!(
            (front_mm - back_mm - 5.0).abs() <= 4.0 * f32::EPSILON * 230.0,
            "{back_mm} to {front_mm} mm"
        );
        assert!(
            (pin.width_mm - 3.4).abs() <= 4.0 * f32::EPSILON * 3.4,
            "{} mm",
            pin.width_mm
        );
    }

    /// Consistency: **the balance hole crosses as long along the key as its
    /// cutter is wide**, on the measured section's key: its outline, its
    /// section through the pin's axis, runs from −1.7 to +1.7 mm, so that
    /// its milled ends make it round.
    #[test]
    fn the_balance_hole_crosses_as_long_as_its_cutter_is_wide() {
        let scene = Scene::of(&Instrument::weighed_section());
        let hole = scene
            .outlines
            .iter()
            .find(|outline| outline.code == Part::BalanceHole)
            .expect("a balance hole");
        let along: Vec<f32> = hole.points_mm.iter().step_by(2).copied().collect();
        let (back, front) = along
            .iter()
            .fold((f32::MAX, f32::MIN), |(back, front), &x| {
                (back.min(x), front.max(x))
            });
        // Millimeters in single precision, a few of its ulps at 1.7 mm.
        assert!(
            (front - 1.7).abs() <= 4.0 * f32::EPSILON * 1.7,
            "{front} mm"
        );
        assert_eq!(back.to_bits(), (-front).to_bits());
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
            PartCode::TopClearance,
            PartCode::KeyTailFelt,
            PartCode::Capstan,
            PartCode::FrontMortise,
            PartCode::BalanceMortise,
            PartCode::BalanceMortiseBushing,
            PartCode::BalanceHole,
            PartCode::KeyButtonSideHole,
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
            CoreInput::CenterOfMassHeight,
            CoreInput::KeyFriction,
            CoreInput::Geometry,
        ] {
            assert_eq!(written(&Input::from(i)), format!("\"{}\"", i.code()));
        }
        for (u, code) in [
            (CoreUnit::Gram, "\"g\""),
            (CoreUnit::Meter, "\"mm\""),
            (CoreUnit::One, "\"ratio\""),
        ] {
            let unit = match u {
                CoreUnit::Gram => Unit::Gram,
                CoreUnit::Meter => Unit::Millimeter,
                CoreUnit::One => Unit::Ratio,
            };
            assert_eq!(written(&unit), code);
        }
    }

    /// Consistency: **a code crosses with its parameters, in the trade's
    /// units with the unit in their names**: a fault in millimeters, a
    /// regulation refused with its target as it was asked, what keeps it
    /// out, its shortfall and the search's reach, a problem with the part
    /// that limits it and the shortfall.
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
            65.0,
            12.5,
        );
        assert_eq!(
            written(&refused),
            r#"{"code":"regulation_infeasible","target":"key_dip","wanted_mm":12.5,"limited_by":"front_pin_in_mortise","shortfall_mm":0.75,"search_mm":20.0}"#
        );
        assert_eq!(
            written(&EditRefusal::Problem(Problem::of(
                &CoreProblem::CapstanInBlock {
                    block: PartCode::BackCheckBlock,
                    by: 0.0095,
                },
                &pa_core::test_fixture::drawer()
            ))),
            r#"{"code":"capstan_in_block","block":"back_check_block","by_mm":9.5}"#
        );
        assert_eq!(
            written(&EditRefusal::Problem(Problem::of(
                &CoreProblem::CapstanBehindBackCheckBlock { by: 0.0395 },
                &pa_core::test_fixture::drawer()
            ))),
            r#"{"code":"capstan_behind_back_check_block","by_mm":39.5}"#
        );
        assert_eq!(
            written(&EditRefusal::Problem(Problem::of(
                &CoreProblem::OutlineCrossesItself {
                    back: PartCode::KeyTailFelt,
                    front: PartCode::BackCheckBlock,
                    by: 0.0125,
                },
                &pa_core::test_fixture::drawer()
            ))),
            r#"{"code":"outline_crosses_itself","back":"key_tail_felt","front":"back_check_block","by_mm":12.5}"#
        );
        assert_eq!(
            written(&Problem::of(
                &CoreProblem::OutOfRange {
                    param: "capstan.x",
                    value: 0.005,
                    min: f64::NEG_INFINITY,
                    max: 0.0,
                },
                &pa_core::test_fixture::drawer()
            )),
            r#"{"code":"out_of_range","param":"key.capstan.behind_mm","value":-5.0,"min":-0.0,"max":null}"#
        );
    }

    /// Consistency: **the sheet and the settings cross whole, wherever the
    /// key stands.** With the front rail raised to 50 mm the key on a short
    /// balance pin has no seat on its frame: its stops would hold it off its
    /// pin. Every
    /// setting still crosses, the front rail at its 50 mm among them; every
    /// reading crosses, the jig's read and every other refused, key height
    /// with the very refusal the diagnostics give the frame; and what holds
    /// the key under the weight on it is said as it stands now.
    #[test]
    fn a_frame_that_cannot_be_seated_sends_every_setting_and_every_reading() {
        let mut bench = Instrument::weighed_section().short_pinned();
        bench
            .set(path::FRONT_RAIL_HEIGHT, 50.0)
            .expect("a front rail raised");
        let why = bench.frame_fault().expect("no seat on this frame");

        let settings = Settings::of(&bench);
        assert_eq!(
            settings.frame.front_rail.height_mm.to_bits(),
            50.0_f64.to_bits()
        );
        assert_eq!(
            settings.frame.balance_punching.papers_mm.len(),
            bench.papers(Stack::Balance).len()
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
    /// no seat. On the tests' bench nothing rides.
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

        let standard = Instrument::weighed_section();
        assert_eq!(Diagnostics::of(&standard).riding, None);
        assert!(Sheet::of(&standard)
            .readings
            .iter()
            .all(|r| matches!(r, Reading::Read { .. })));
    }

    /// Consistency, and what the owner sees: **a refused edit answers in
    /// codes and numbers, changes nothing, and the diagnostics keep it until
    /// an edit is taken.** A 12 mm dip the front punching's felt forbids, a
    /// mass of nothing, a front weight read off the covering, a front weight
    /// that is not a number, a key friction weight under what no friction
    /// reads, a key front pushed into its key button, a path the instrument
    /// has not, and JSON that is no action: each answers its code with the
    /// numbers its wording needs, and the documents read as before. A dip
    /// the felt allows is laid, and says what it was asked as it was asked.
    #[test]
    fn a_refused_edit_answers_in_codes_and_changes_nothing() {
        let mut bench = Instrument::weighed_section();
        let documents =
            |b: &Instrument| json(&(Settings::of(b), Sheet::of(b), State::of(b), b.state_hash()));
        let before = documents(&bench);
        let refused = |acted: Acted| match acted {
            Acted::Refused { why } => why,
            other => panic!("{other:?}"),
        };

        let why = refused(act(
            &mut bench,
            Action::Regulate {
                key_height_mm: 65.0,
                key_dip_mm: 12.0,
            },
        ));
        let EditRefusal::Regulation(RegulationRefusal::RegulationInfeasible {
            target,
            wanted_mm,
            limited_by,
            shortfall_mm,
            search_mm,
        }) = why
        else {
            panic!("{why:?}")
        };
        assert_eq!(
            (target, limited_by),
            (Target::KeyDip, Limit::FeltThickerThanTheStack)
        );
        assert_eq!(wanted_mm.to_bits(), 12.0_f64.to_bits());
        assert!((shortfall_mm - 0.78).abs() < 0.005, "{shortfall_mm}");
        assert!((search_mm - 20.0).abs() < 1e-12);
        let kept = Diagnostics::of(&bench).refused_edit.expect("kept");
        assert_eq!(
            kept.edit,
            Edit::Act {
                action: Action::Regulate {
                    key_height_mm: 65.0,
                    key_dip_mm: 12.0
                }
            }
        );

        let weigh = |mass_g, front_weight_g, from_lip_mm| Action::WeighKey {
            mass_g,
            front_weight_g,
            from_lip_mm,
        };
        assert_eq!(
            refused(act(&mut bench, weigh(0.0, 6.5, 13.0))),
            EditRefusal::Problem(Problem::MassNotPositive { mass_g: 0.0 })
        );
        assert_eq!(
            refused(act(&mut bench, weigh(121.1, 6.5, 200.0))),
            EditRefusal::Weighing(WeighingRefusal::RollerOffTheCovering {
                from_lip_mm: 0.2 * 1000.0,
                covering_mm: bench.key().keytop_length * 1000.0,
            })
        );
        assert_eq!(
            refused(act(&mut bench, weigh(121.1, f64::NAN, 13.0))),
            EditRefusal::Problem(Problem::NotFinite {
                param: path::WEIGHED_FRONT_WEIGHT.to_owned()
            })
        );
        let EditRefusal::Friction(FrictionRefusal::UnderNoFriction { typed_g, least_g }) =
            refused(act(
                &mut bench,
                Action::SetFrictionFromKeyFrictionWeight { kf_g: 0.0 },
            ))
        else {
            panic!("no bushings read 0 g on this key")
        };
        assert_eq!(typed_g.to_bits(), 0.0_f64.to_bits());
        assert!(least_g > 0.2 && least_g < 0.5, "{least_g}");
        let EditRefusal::Problem(Problem::OutlineCrossesItself { back, front, by_mm }) =
            refused(set(&mut bench, path::FRONT_EDGE, 170.0))
        else {
            panic!("a key front at 170 mm runs its covering into its key button")
        };
        assert_eq!((back, front), (Part::KeyButton, Part::Keytop));
        assert!((by_mm - 14.0).abs() < 1e-9, "{by_mm}");
        assert_eq!(
            refused(set(&mut bench, "key.length_mm", 300.0)),
            EditRefusal::Problem(Problem::UnknownPath {
                path: "key.length_mm".to_owned()
            })
        );
        assert_eq!(
            refused(act_json(&mut bench, r#"{"action":"press_the_key"}"#)),
            EditRefusal::Problem(Problem::NotAnAction)
        );
        assert_eq!(
            documents(&bench),
            before,
            "a refusal changed the instrument"
        );
        assert!(bench.log().is_empty(), "a refusal is not logged");

        let mut wedged = wedged();
        let riding = wedged.riding().map(Riding::from);
        assert_eq!(
            refused(act(
                &mut wedged,
                Action::SetFrictionFromKeyFrictionWeight { kf_g: 6.0 }
            )),
            EditRefusal::Friction(FrictionRefusal::ProtocolRefused {
                why: Refusal::NoTravel { riding },
            })
        );

        let Acted::Laid {
            balance_mm,
            front_mm,
            asked_key_height_mm,
            asked_key_dip_mm,
            ..
        } = act(
            &mut bench,
            Action::Regulate {
                key_height_mm: 65.0,
                key_dip_mm: 10.6,
            },
        )
        else {
            panic!("10.6 mm of dip is reachable")
        };
        let frame = Settings::of(&bench).frame;
        assert_eq!(
            balance_mm.to_bits(),
            frame.balance_punching.paper_total_mm.to_bits()
        );
        assert_eq!(
            front_mm.to_bits(),
            frame.front_punching.paper_total_mm.to_bits()
        );
        assert_eq!(
            (asked_key_height_mm.to_bits(), asked_key_dip_mm.to_bits()),
            (65.0_f64.to_bits(), 10.6_f64.to_bits())
        );
        assert_eq!(
            Diagnostics::of(&bench).refused_edit,
            None,
            "an edit taken forgets the refusal"
        );
    }

    /// Consistency: **every path round-trips**: set in its unit, the
    /// settings read it back at that very path, in that unit, within the
    /// rounding of turning it into SI and back, half an ulp each way. The
    /// back rail and the capstan read as distances behind the balance pin,
    /// the bushings as the torque they hold. A path an action sets is
    /// refused by `set`, naming the action.
    #[test]
    fn every_path_round_trips() {
        let slider = |path: &str| {
            registry::PAGE
                .iter()
                .flat_map(|(_, controls)| controls.iter())
                .find_map(|c| match *c {
                    Control::Slider { path: p, soft, .. } if p == path => Some(soft),
                    _ => None,
                })
        };
        for entry in &registry::ENTRIES {
            let mut bench = Instrument::weighed_section();
            if let Some(action) = entry.set_by() {
                assert_eq!(
                    bench.set(entry.path, 1.0),
                    Err(CoreEditRefusal::SetByAction {
                        path: entry.path,
                        action
                    })
                );
                continue;
            }
            // Within what the page offers, a little off its middle so that
            // no default sits there; a torque the page shows as a reading.
            let value = slider(entry.path).map_or(1500.25, |soft| {
                let (min, max) = (soft.min.expect("bounded"), soft.max.expect("bounded"));
                min + 0.4 * (max - min)
            });
            bench
                .set(entry.path, value)
                .unwrap_or_else(|why| panic!("{} = {value}: {why:?}", entry.path));
            let read = at(&Settings::of(&bench), entry.path)
                .as_f64()
                .unwrap_or_else(|| panic!("{} reads no number", entry.path));
            assert!(
                (read - value).abs() <= f64::EPSILON * value.abs(),
                "{} set to {value} reads {read}",
                entry.path
            );
            let described = Registry::of(&bench);
            assert!(
                described.params.iter().any(|p| p.path == entry.path),
                "{} is not described",
                entry.path
            );
        }
        // The lists and the weighing, set by their actions, read back too.
        let mut bench = Instrument::weighed_section();
        bench
            .act(Action::WeighKey {
                mass_g: 125.5,
                front_weight_g: 7.25,
                from_lip_mm: 12.7,
            })
            .expect("a key's weighing");
        let settings = Settings::of(&bench);
        for (path, typed) in [
            (path::WEIGHED_MASS, 125.5),
            (path::WEIGHED_FRONT_WEIGHT, 7.25),
            (path::WEIGHED_FROM_LIP, 12.7),
        ] {
            let read = at(&settings, path).as_f64().expect("a number");
            assert!(
                (read - typed).abs() <= f64::EPSILON * typed,
                "{path}: {read}"
            );
        }
        let papers = at(&settings, path::BALANCE_PAPERS);
        assert_eq!(
            papers.as_array().map(Vec::len),
            Some(bench.papers(Stack::Balance).len())
        );
    }

    /// Consistency: **the registry describes every path and the page.**
    /// Each setting crosses with its unit, the range it admits, its step and
    /// how it is reached: the capstan's height a screw turned continuously,
    /// the papers by the drawer, the targets aimed at through the
    /// regulation, the weighing through the key's weighing; and the page
    /// lists its panels in order, each control reading a path described.
    #[test]
    fn the_registry_describes_every_path_and_the_page() {
        let r = Registry::of(&Instrument::standard());
        assert_eq!(r.params.len(), registry::ENTRIES.len());
        let param = |path: &str| {
            r.params
                .iter()
                .find(|p| p.path == path)
                .unwrap_or_else(|| panic!("{path}"))
                .clone()
        };
        let capstan = param(path::CAPSTAN_HEIGHT);
        assert_eq!(capstan.step, Step::Continuous);
        assert_eq!(capstan.layer, Layer::Regulation);
        assert_eq!((capstan.min, capstan.max), (Some(2.0), Some(30.0)));
        let behind = param(path::CAPSTAN_BEHIND);
        assert_eq!((behind.min, behind.max), (Some(-0.0), None));
        let papers = param(path::FRONT_PAPERS);
        assert!(papers.list);
        assert_eq!(
            papers.step,
            Step::Drawer {
                sizes_mm: pa_core::test_fixture::PAPER_THICKNESSES
                    .iter()
                    .map(|&p| p * 1000.0)
                    .collect()
            }
        );
        assert_eq!(
            papers.set_by,
            SetBy::Act {
                action: ActionCode::AddPaper
            }
        );
        assert_eq!(param(path::KEY_DIP).kind, Kind::Target);
        assert_eq!(param(path::WEIGHED_MASS).kind, Kind::Measured);
        assert_eq!(param(path::FRICTION_TORQUE).unit, SettingUnit::Gmm);
        assert_eq!(
            r.page.iter().map(|p| p.panel).collect::<Vec<_>>(),
            [
                Panel::Instrument,
                Panel::Sheet,
                Panel::Regulation,
                Panel::Punchings,
                Panel::Key,
                Panel::Weighing,
                Panel::Frame,
                Panel::Touch
            ]
        );
        // Every address a control reads in the settings is there, a number
        // or a list as the control reads it; every seed of a form is a
        // number on the standard bench, and the friction control's argument
        // is its action's.
        let settings = Settings::of(&Instrument::standard());
        let sheet = Sheet::of(&Instrument::standard());
        for panel in &r.page {
            for control in &panel.controls {
                match control {
                    ControlLayout::Slider { path, .. } => {
                        assert!(r.params.iter().any(|p| &p.path == path), "{path}");
                        assert!(at(&settings, path).is_f64(), "{path}");
                    }
                    ControlLayout::Stack { path, total, .. } => {
                        assert!(r.params.iter().any(|p| &p.path == path), "{path}");
                        assert!(at(&settings, path).is_array(), "{path}");
                        assert!(at(&settings, total).is_f64(), "{total}");
                    }
                    ControlLayout::Form { fields, .. } => {
                        for f in fields {
                            assert!(r.params.iter().any(|p| p.path == f.path), "{}", f.path);
                            match &f.seed {
                                Seed::Setting { at: address } => {
                                    assert!(at(&settings, address).is_f64(), "{address}");
                                }
                                Seed::Reading { procedure } => assert!(
                                    sheet.readings.iter().any(|m| matches!(
                                        m,
                                        Reading::Read { procedure: p, .. } if p == procedure
                                    )),
                                    "{procedure:?}"
                                ),
                            }
                        }
                    }
                    ControlLayout::KeyFrictionWeight {
                        action, argument, ..
                    } => {
                        let typed = format!(r#"{{"action":{},"{argument}":7}}"#, written(action));
                        assert!(serde_json::from_str::<Action>(&typed).is_ok(), "{typed}");
                    }
                    ControlLayout::Button { .. } => {}
                    ControlLayout::Profile {
                        action,
                        argument,
                        options,
                        at: address,
                        ..
                    } => {
                        // Every option is a profile the action takes, and the
                        // one on the bench is among them.
                        for option in options {
                            let typed = format!(
                                r#"{{"action":{},"{argument}":"{option}"}}"#,
                                written(action)
                            );
                            assert!(serde_json::from_str::<Action>(&typed).is_ok(), "{typed}");
                        }
                        let on = at(&settings, address);
                        assert!(options.iter().any(|o| Some(o.as_str()) == on.as_str()));
                    }
                }
            }
        }
    }

    /// Consistency: **every drawn stretch is known by its profile.** On
    /// every shipped profile's instrument as it arrives, each stretch of
    /// every part's outline is placed by a number its profile binds, and
    /// carries how that number is known and the file that records or states
    /// it, every segment of every part covered once; on Hirschkorn's key
    /// the chamfer is published, his key front traced, the key button's
    /// height traced, his front punching's felt assumed, and the pins and
    /// punchings his profile reads of the owner's key frame borrowed, which
    /// the measured section's are measured; no stretch of his is drawn as
    /// measured but a paper's, the drawer's stock being no instrument's, and
    /// none of the measured section's as borrowed. A key front set on the bench is no
    /// longer the profile's: the stretches it places say so, and nothing
    /// else does.
    #[test]
    fn every_drawn_stretch_is_known_by_its_profile() {
        for name in Profile::names() {
            let b = Instrument::of(Profile::named(name).unwrap()).unwrap();
            for o in Scene::of(&b).outlines {
                let segments = usize::from(*o.segs.last().unwrap()) + 1;
                let covered: usize = o.sources.iter().map(|s| usize::from(s.to - s.from)).sum();
                assert_eq!(covered, segments, "{name}: {:?}", o.code);
                for source in &o.sources {
                    assert!(
                        source.how != How::SetOnTheBench && source.at.is_some(),
                        "{name}: {:?} placed by {} is not known",
                        o.code,
                        source.number
                    );
                }
            }
        }
        let his = Instrument::of(Profile::named("hirschkorn").unwrap()).unwrap();
        let how = |b: &Instrument, part: Part, number: &str| -> How {
            Scene::of(b)
                .outlines
                .into_iter()
                .find(|o| o.code == part)
                .and_then(|o| o.sources.into_iter().find(|s| s.number == number))
                .unwrap_or_else(|| panic!("{part:?} placed by {number}"))
                .how
        };
        assert_eq!(how(&his, Part::Keystick, "tail_rise"), How::Published);
        assert_eq!(how(&his, Part::KeyFront, "front_edge"), How::Traced);
        assert_eq!(how(&his, Part::KeyButton, "capsule_height"), How::Traced);
        assert_eq!(
            how(&his, Part::FrontRailPunching, "front_punching.felt"),
            How::Assumed
        );
        assert_eq!(how(&his, Part::Capstan, "capstan.height"), How::Published);
        let section = Instrument::of(Profile::named(crate::profile::SECTION).unwrap()).unwrap();
        for (part, number) in [
            (Part::FrontRailPin, "front_rail.pin.length"),
            (Part::FrontRailPin, "front_rail.pin_height"),
            (Part::BalanceRailPin, "balance_rail.pin.diameter"),
            (Part::FrontRailPunching, "front_rail.punching_radius"),
            (Part::FrontRailPunching, "front_rail.punching_hole_radius"),
            (Part::BalanceRailPunching, "balance_rail.punching_radius"),
            (
                Part::BalanceRailPunching,
                "balance_rail.punching_hole_radius",
            ),
        ] {
            assert_eq!(how(&his, part, number), How::Borrowed, "{number}");
            assert_eq!(how(&section, part, number), How::Measured, "{number}");
        }
        let drawn = |b: &Instrument| -> Vec<(Part, How)> {
            Scene::of(b)
                .outlines
                .into_iter()
                .flat_map(|o| o.sources.into_iter().map(move |s| (o.code, s.how)))
                .collect()
        };
        assert!(drawn(&his)
            .iter()
            .all(|&(part, how)| how != How::Measured || part == Part::PaperPunching));
        assert!(drawn(&section).iter().all(|&(_, how)| how != How::Borrowed));
        let mut set = his.clone();
        set.set(path::FRONT_EDGE, 236.0).expect("a shorter key");
        assert_eq!(how(&set, Part::KeyFront, "front_edge"), How::SetOnTheBench);
        assert_eq!(how(&set, Part::KeyFront, "front_board"), How::Traced);
    }

    /// Consistency, the boundary's half of the contacts the view shows: **a
    /// contact crosses with its row's normal and the force it carries.** On
    /// the tests' bench at rest, the key's weight down on its front
    /// punching and its balance punching, the rows closed there push it up,
    /// along unit normals; the forces of the closed rows carry the key's
    /// weight, read on its balance, within the rounding of the crossing.
    #[test]
    fn a_contact_crosses_with_its_normal_and_its_force() {
        let bench = Instrument::weighed_section();
        let state = State::of(&bench);
        let closed: Vec<_> = state
            .contacts
            .iter()
            .filter(|c| c.state == ContactState::Closed)
            .collect();
        assert!(!closed.is_empty());
        for c in &state.contacts {
            let [x, y] = c.normal;
            assert!(((x * x + y * y) - 1.0).abs() < 1e-6, "{c:?}");
            assert!(c.force_g >= 0.0, "a contact pushes: {c:?}");
        }
        let pushed: f32 = closed.iter().map(|c| c.force_g * c.normal[1]).sum();
        let mass_g = bench.key_mass_g().expect("a mass") as f32;
        assert!(
            (pushed - mass_g).abs() < 1e-3 * mass_g,
            "the closed rows carry {pushed} g of a {mass_g} g key"
        );
        assert!(closed
            .iter()
            .any(|c| c.pair == Contact::FrontRailPunching && c.force_g > 0.0));
    }

    /// Consistency: **what holds the key crosses as its code.** The front
    /// rail raised to 44 mm tips the key forward off its balance punching,
    /// onto its front punching and its front pin against the wall of its
    /// slot, its cloth open: those two hold it, with no friction. Pulled
    /// 62.7 g, turned up on its cloth off its punching, the cloth alone
    /// holds it, with its friction.
    #[test]
    fn what_holds_the_key_crosses_as_its_code() {
        let mut bench = Instrument::weighed_section();
        bench
            .set(path::FRONT_RAIL_HEIGHT, 44.0)
            .expect("a front rail raised");
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

        let mut bench = Instrument::weighed_section();
        bench.set(path::WEIGHT, -62.7).expect("a pull");
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

    /// Consistency: **the band crosses as codes and numbers.** Its two
    /// readings by their procedures, read as the sheet's walks are read to
    /// the gram, with the sweep that took them; its curves in millimeters of
    /// travel and grams, the down curve from the first step off the top seat
    /// down and the up curve from the first step off the bottom seat up,
    /// inside the window; and where the key has no stroke, both readings
    /// refused for it, saying how the key rides, and no curves. The touch's
    /// travel puts the key at the bottom of the window where the weight on
    /// it takes it down, and at its top seat where a pull brings it back,
    /// within a millionth of a millimeter.
    #[test]
    fn the_band_crosses_as_codes_and_numbers() {
        let bench = Instrument::weighed_section();
        let band = Band::of(&bench);
        let curves = band.curves.as_ref().expect("the band is drawn");
        let procedures: Vec<Procedure> = band
            .readings
            .iter()
            .map(|r| match r {
                Reading::Read { procedure, .. } | Reading::Refused { procedure, .. } => *procedure,
            })
            .collect();
        assert_eq!(
            procedures,
            vec![Procedure::DownWeightOnTheBand, Procedure::UpWeightOnTheBand]
        );
        let json = written(&band);
        assert!(
            json.contains(r#""procedure":"down_weight_on_the_band""#),
            "{json}"
        );
        let Reading::Read {
            unit, conditions, ..
        } = &band.readings[0]
        else {
            panic!("D is read on the band")
        };
        assert_eq!(*unit, Unit::Gram);
        let swept = conditions.sweep.as_ref().expect("the sweep it was read by");
        assert_eq!(swept.step_mm, curves.step_mm);
        assert_eq!(swept.window_mm, curves.window_mm);
        assert_eq!(swept.samples as usize, curves.down.len());
        assert_eq!(curves.down.len(), curves.up.len());
        let first = |c: &[BandPoint]| c.first().map(|p| p.travel_mm).unwrap_or(f64::NAN);
        let last = |c: &[BandPoint]| c.last().map(|p| p.travel_mm).unwrap_or(f64::NAN);
        assert!((first(&curves.down) - curves.step_mm).abs() < 1e-9);
        assert!(last(&curves.down) < curves.window_mm);
        assert_eq!(first(&curves.up), last(&curves.down));
        assert!(curves
            .down
            .iter()
            .zip(curves.up.iter().rev())
            .all(|(d, u)| d.force_g > u.force_g));

        let rides = Band::of(&wedged());
        assert!(rides.curves.is_none());
        for r in &rides.readings {
            let Reading::Refused {
                why: Refusal::NoTravel { riding: Some(_) },
                ..
            } = r
            else {
                panic!("a riding key has no band: {r:?}")
            };
        }

        let travel = |weight_g: f64| {
            let mut b = Instrument::weighed_section();
            b.set(path::WEIGHT, weight_g).expect("a weight");
            Diagnostics::of(&b)
                .touch
                .and_then(|t| t.travel_mm)
                .expect("seated")
        };
        // The tests' key goes down on its own, D being under nothing; a pull
        // past U brings it back up to its top seat.
        let (bottom, top) = (travel(0.0), travel(-15.0));
        assert!(
            (bottom - curves.window_mm).abs() < 1e-6,
            "nothing on it, the key goes to the bottom of its window: {bottom} mm of {} mm",
            curves.window_mm
        );
        assert!(
            top.abs() < 1e-6,
            "a pull of 15 g lifts it to its top seat: {top} mm"
        );
    }

    /// Consistency: the targets crossing are the targets regulated to.
    #[test]
    fn the_settings_hold_the_targets_regulated_to() {
        let bench = Instrument::weighed_section();
        let CoreTargets {
            key_height,
            key_dip,
        } = bench.targets();
        let r = Settings::of(&bench).regulation;
        assert_eq!(r.key_height_mm.to_bits(), (key_height * 1000.0).to_bits());
        assert_eq!(r.key_dip_mm.to_bits(), (key_dip * 1000.0).to_bits());
    }
}
