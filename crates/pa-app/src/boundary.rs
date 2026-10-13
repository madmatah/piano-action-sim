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
//! from one catalogue per language (`apps/bench/src/i18n/`). A code carries
//! every number its wording needs, the instrument's own limits included, so
//! no wording of a code holds a number of the engine's.
//!
//! The codes mirror the domain's enums in `pa-core` through `match`es with
//! no catch-all arm: a refusal added there and not mapped here is a compile
//! error, and one mapped here and not worded there fails the interface's
//! type-check.
//!
//! # Six documents, apart, and two calls
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
//! marks, each with its normal and the force it carries; and [`Scene`] what
//! is drawn, each part in its own body's frame. An interface changes the
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
use pa_core::parts::frame::PAPER;
use pa_core::parts::mounts::KeyMount;
use pa_core::parts::{KeyMass, NotAWeighing};
use pa_core::registry::{
    Kind as CoreKind, Layer as CoreLayer, Quantity as CoreQuantity, Step as CoreStep,
};
use pa_core::regulation::{Limit as CoreLimit, Target as CoreTarget, Unsolvable, STACK_SEARCH};
use pa_core::validate::Problem as CoreProblem;
use pa_core::G;

use crate::instrument::{
    Action, ActionCode, ContactState as CoreContactState, Edit as CoreEdit,
    EditRefusal as CoreEditRefusal, Instrument, NotRegulated, Outcome, Riding as CoreRiding,
    SeatReport, Stack, TouchVerdict, KEY, PAPER_THICKNESSES,
};
use crate::registry::{self, path, Control, Entry, Field, Label, Panel, Weighing as WeighingField};

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

/// Every setting of the instrument, in the trade's units, nested as its
/// paths are: the value at `frame.front_rail.pin_height_mm` is
/// `frame.front_rail.pin_height_mm` here ([`crate::registry`]). Always
/// valid, whatever the frame does, so every control reads its value here.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Settings {
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
    /// How far back from the covering's lip every weight is centred,
    /// millimetres: the measuring point of every touchweight.
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
    /// Its capstan.
    pub capstan: CapstanSettings,
    /// Its front mortise.
    pub front_mortise: MortiseSettings,
    /// Its bushings.
    pub friction: FrictionSettings,
    /// Its weighing as it was typed; `null` while its mass is derived from
    /// its volume.
    pub weighing: Option<Weighed>,
    /// Its mass as the statics take it, grams: as weighed, or derived from
    /// its volume until it is; `null` where the volume model encloses
    /// nothing. Not a setting: what the weighing, or the estimate, gives.
    pub mass_g: Option<f64>,
}

/// The capstan.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct CapstanSettings {
    /// Where it stands, millimetres behind the balance pin.
    pub behind_mm: f64,
    /// How far it stands above the keystick, millimetres.
    pub height_mm: f64,
}

/// The front mortise.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct MortiseSettings {
    /// Its depth above the underside, millimetres.
    pub depth_mm: f64,
}

/// The bushings.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FrictionSettings {
    /// Their friction as stored: the torque they hold, gram-force
    /// millimetres. What Stanwood's protocol reads them as is a reading, on
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
    /// How far back from the covering's lip the roller stood, millimetres.
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
    /// The bare rail's height above the keybed, millimetres.
    pub height_mm: f64,
}

/// The front rail.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct FrontRailSettings {
    /// The bare rail's height above the keybed, millimetres.
    pub height_mm: f64,
    /// Where it stands, millimetres in front of the balance pin.
    pub at_mm: f64,
    /// How far its pin stands above the bare rail, millimetres.
    pub pin_height_mm: f64,
}

/// The back rail.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct BackRailSettings {
    /// The bare rail's height above the keybed, millimetres.
    pub height_mm: f64,
    /// Where it stands, millimetres behind the balance pin.
    pub behind_mm: f64,
    /// Its cloth's thickness, millimetres.
    pub cloth_mm: f64,
}

/// A punching: its felt, and the papers under it, the coordinate regulation
/// sets.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct PunchingSettings {
    /// The felt washer, millimetres.
    pub felt_mm: f64,
    /// The papers, millimetres each, in the order they were laid, bottom
    /// first.
    pub papers_mm: Vec<f64>,
    /// Their total, millimetres, as the engine sums it.
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
                },
                friction: FrictionSettings {
                    torque_gmm: at(path::FRICTION_TORQUE),
                },
                weighing: match instrument.key_mass() {
                    KeyMass::Estimated => None,
                    KeyMass::Weighed { .. } => Some(Weighed {
                        mass_g: at(path::WEIGHED_MASS),
                        front_weight_g: at(path::WEIGHED_FRONT_WEIGHT),
                        from_lip_mm: at(path::WEIGHED_FROM_LIP),
                    }),
                },
                mass_g: instrument.key_mass_g(),
            },
            frame: FrameSettings {
                balance_rail: BalanceRailSettings {
                    height_mm: at(path::BALANCE_RAIL_HEIGHT),
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
    /// Millimetres.
    Mm,
    /// Grams.
    G,
    /// Gram-force millimetres.
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
        /// Their thicknesses, millimetres, thinnest first.
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
    /// The registry of `instrument`'s paths.
    pub fn of(_instrument: &Instrument) -> Self {
        Self {
            params: registry::ENTRIES.iter().map(Param::of).collect(),
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
    fn of(e: &Entry) -> Self {
        let declared = |hard: (f64, f64), quantity, layer, kind, step: CoreStep| {
            let (a, b) = (e.from_si(hard.0), e.from_si(hard.1));
            let (min, max) = if a <= b { (a, b) } else { (b, a) };
            (
                quantity,
                layer,
                kind,
                bound(min),
                bound(max),
                Step::of(step),
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
                registry::Unit::Millimetre => SettingUnit::Mm,
                registry::Unit::Gram => SettingUnit::G,
                registry::Unit::GramMillimetre => SettingUnit::Gmm,
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
    fn of(step: CoreStep) -> Self {
        match step {
            CoreStep::Continuous => Self::Continuous,
            CoreStep::Drawer(sizes) => Self::Drawer {
                sizes_mm: sizes.iter().map(|&s| mm(s)).collect(),
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

/// Where one body's frame stands: its origin, millimetres, and the cosine
/// and sine of its rotation. The view applies it as given and computes none.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct Pose {
    /// The origin, millimetres.
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
    /// Where, millimetres, in the body's frame.
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

/// A contact the view marks, in the keybed's frame: one active row, where
/// the contact rows put its two points, not where a body's pose carries
/// them, with its normal and what it carries.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
pub struct MarkedContact {
    /// Which contact.
    pub pair: Contact,
    /// Where the key's surface is, millimetres.
    pub on_key_mm: [f32; 2],
    /// Where the frame's surface is, millimetres.
    pub on_frame_mm: [f32; 2],
    /// The signed gap between them, millimetres: negative where the drawn
    /// parts overlap.
    pub gap_mm: f32,
    /// Closed, open or overlapping.
    pub state: ContactState,
    /// The row's unit normal, from the frame's surface towards the key's.
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
    /// The measuring point, on the key: where the engine weighs.
    pub measuring_point: Marker,
    /// The contacts within a millimetre of closing, where the key is drawn.
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
    pub width_mm: f32,
    /// Where a turned part's axis stands along its body's `x`, millimetres;
    /// zero when extruded.
    pub axis_x_mm: f32,
    /// Flat `[x0, y0, x1, y1, …]`, millimetres, in the body's frame; for a
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
        Self {
            outlines: instrument
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
                        width_mm: width,
                        axis_x_mm: axis_x,
                        points_mm: o
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
        /// What was asked of it, millimetres, as it was asked.
        wanted_mm: f64,
        /// What keeps it out of reach.
        limited_by: Limit,
        /// How far out of reach, in its own reading, millimetres.
        shortfall_mm: f64,
        /// The most paper the search lays on a stack, millimetres.
        search_mm: f64,
    },
    /// The key could not be seated at one of the stacks the search tried.
    NoSeat {
        /// Why.
        why: Refusal,
    },
}

impl RegulationRefusal {
    /// Why a regulation to a key height and a key dip, millimetres as they
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
    /// Why a weighing of `instrument`'s key was refused; a number that is
    /// not one, or a mass of nothing, never reaches the weighing
    /// ([`Problem`]).
    pub fn of(instrument: &Instrument, why: NotAWeighing) -> Self {
        match why {
            NotAWeighing::RollerOffTheCovering { from_lip_m } => Self::RollerOffTheCovering {
                from_lip_mm: mm(from_lip_m),
                covering_mm: mm(instrument.key().keytop_length),
            },
            NotAWeighing::CentreOfMassOffTheKey { at_m } => {
                Self::CentreOfMassOffTheKey { at_mm: mm(at_m) }
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
        /// The thickness asked for, millimetres.
        thickness_mm: f64,
        /// What the drawer holds, millimetres.
        drawer_mm: Vec<f64>,
    },
    /// A key weighed at no mass, or less.
    MassNotPositive {
        /// The mass typed, grams.
        mass_g: f64,
    },
    /// Two parts on the key's top overlap, or its chamfer runs past its
    /// front: the outline crosses itself.
    OutlineCrossesItself {
        /// The part behind.
        back: Part,
        /// The part in front.
        front: Part,
        /// How far they overlap along the key, millimetres.
        by_mm: f64,
    },
    /// A feature off the keystick's wood where it must stand.
    FeatureOutsideKeystick {
        /// The feature.
        part: Part,
        /// How far off, millimetres along the key.
        by_mm: f64,
    },
    /// The capstan stands in a glued block.
    CapstanInBlock {
        /// The block.
        block: Part,
        /// How far its head stands into it, millimetres along the key.
        by_mm: f64,
    },
    /// The front mortise is cut through the top of the keystick.
    MortiseThroughKeyTop {
        /// Its depth, millimetres.
        depth_mm: f64,
        /// The keystick's wood it is cut into, millimetres.
        wood_mm: f64,
        /// How far it reaches past the wood's top, millimetres.
        by_mm: f64,
    },
    /// The measuring point stands off the covering.
    MeasuringPointOffCovering {
        /// Where it was asked for, millimetres back from the lip.
        from_lip_mm: f64,
        /// How far the covering runs back from its lip, millimetres.
        covering_mm: f64,
    },
    /// The instrument has no such path.
    UnknownPath {
        /// The path asked for.
        path: String,
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
    /// path.
    pub fn of(p: &CoreProblem) -> Self {
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
                drawer_mm: PAPER_THICKNESSES.iter().map(|&p| mm(p)).collect(),
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
            CoreProblem::CapstanInBlock { block, by } => Self::CapstanInBlock {
                block: block.into(),
                by_mm: mm(by),
            },
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
            CoreEditRefusal::Problem(p) => Self::Problem(Problem::of(p)),
            CoreEditRefusal::UnknownPath { path } => {
                Self::Problem(Problem::UnknownPath { path: path.clone() })
            }
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
        let mut bench = Instrument::standard();
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
                }
            ))),
            r#"{"code":"capstan_in_block","block":"back_check_block","by_mm":9.5}"#
        );
        assert_eq!(
            written(&Problem::of(&CoreProblem::OutOfRange {
                param: "capstan.x",
                value: 0.005,
                min: f64::NEG_INFINITY,
                max: 0.0,
            })),
            r#"{"code":"out_of_range","param":"key.capstan.behind_mm","value":-5.0,"min":-0.0,"max":null}"#
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
        let mut bench = Instrument::standard();
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

        let standard = Instrument::standard();
        assert_eq!(Diagnostics::of(&standard).riding, None);
        assert!(Sheet::of(&standard)
            .readings
            .iter()
            .all(|r| matches!(r, Reading::Read { .. })));
    }

    /// Consistency, and what the owner sees: **a refused edit answers in
    /// codes and numbers, changes nothing, and the diagnostics keep it until
    /// an edit is taken.** A 12 mm dip the front pin forbids, a mass of
    /// nothing, a front weight read off the covering, a front weight that is
    /// not a number, a key friction weight under what no friction reads, a
    /// key front pushed into its key button, a path the instrument has not,
    /// and JSON that is no action: each answers its code with the numbers
    /// its wording needs, and the documents read as before. A dip the pin
    /// allows is laid, and says what it was asked as it was asked.
    #[test]
    fn a_refused_edit_answers_in_codes_and_changes_nothing() {
        let mut bench = Instrument::standard();
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
            (Target::KeyDip, Limit::FrontPinInMortise)
        );
        assert_eq!(wanted_mm.to_bits(), 12.0_f64.to_bits());
        assert!((shortfall_mm - 0.79).abs() < 0.005, "{shortfall_mm}");
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
            let mut bench = Instrument::standard();
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
        let mut bench = Instrument::standard();
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
                sizes_mm: PAPER_THICKNESSES.iter().map(|&p| p * 1000.0).collect()
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
                }
            }
        }
    }

    /// Consistency, the boundary's half of the contacts the view shows: **a
    /// contact crosses with its row's normal and the force it carries.** On
    /// the standard bench at rest, the key's weight down on its front
    /// punching and its balance punching, the rows closed there push it up,
    /// along unit normals; the forces of the closed rows carry the key's
    /// weight, read on its balance, within the rounding of the crossing.
    #[test]
    fn a_contact_crosses_with_its_normal_and_its_force() {
        let bench = Instrument::standard();
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
    /// rail raised to 41.7 mm tips the key forward off its balance punching,
    /// onto its front punching and its front pin against the wall of its
    /// slot, its cloth open: those two hold it, with no friction. Pulled
    /// 62.7 g, turned up on its cloth off its punching, the cloth alone
    /// holds it, with its friction.
    #[test]
    fn what_holds_the_key_crosses_as_its_code() {
        let mut bench = Instrument::standard();
        bench
            .set(path::FRONT_RAIL_HEIGHT, 41.7)
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

        let mut bench = Instrument::standard();
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

    /// Consistency: the targets crossing are the targets regulated to.
    #[test]
    fn the_settings_hold_the_targets_regulated_to() {
        let bench = Instrument::standard();
        let CoreTargets {
            key_height,
            key_dip,
        } = bench.targets();
        let r = Settings::of(&bench).regulation;
        assert_eq!(r.key_height_mm.to_bits(), (key_height * 1000.0).to_bits());
        assert_eq!(r.key_dip_mm.to_bits(), (key_dip * 1000.0).to_bits());
    }
}
