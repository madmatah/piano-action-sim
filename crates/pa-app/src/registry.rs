//! The instrument's settings by path: the domain's declarations composed
//! under prefixes, each path with its trade unit in its name, and the page
//! that presents them.
//!
//! # The domain declares, the application presents
//!
//! What a setting admits is declared in `pa-core` beside the field it sets
//! ([`pa_core::registry::ParamSpec`]), in SI. This module composes those
//! declarations into the instrument's paths, `frame.front_rail.pin_height_mm`,
//! converting once, here, between the domain's SI and the trade's unit the
//! path names, and turning a position behind the balance pin into the
//! distance behind it a technician measures. It adds what only an
//! application knows: the page that shows the settings ([`PAGE`]), each
//! control with its panel, the label it is worded by and the range its
//! slider sweeps, a range inside the hard one that a page offers, never one
//! the engine enforces.
//!
//! Some paths are reached only through an action, because what they hold is
//! an operation and not a value: the papers are a pile laid and lifted a
//! paper at a time, the targets are what the regulation solve aims at, and a
//! key's weighing is three readings taken together.

use pa_core::measure::measurement::ProcedureId;
use pa_core::measure::{TouchweightProtocol, WEIGHT_ON_THE_MEASURING_POINT};
use pa_core::mechanism::AxisFriction;
use pa_core::parts::{FrameDesign, KeyDesign};
use pa_core::registry::ParamSpec;
use pa_core::regulation::Targets;
use pa_core::G;
use serde::Serialize;
#[cfg(test)]
use ts_rs::TS;

use crate::instrument::{ActionCode, Stack};

/// Every path, by name.
pub mod path {
    /// How far back from the covering's lip every weight is centered.
    pub const MEASURING_POINT: &str = "protocol.measuring_point_from_lip_mm";
    /// The weight on the measuring point, negative pulling up.
    pub const WEIGHT: &str = "experiment.weight_g";
    /// The key height the regulation aims at.
    pub const KEY_HEIGHT: &str = "regulation.key_height_mm";
    /// The key dip the regulation aims at.
    pub const KEY_DIP: &str = "regulation.key_dip_mm";
    /// The key front, in front of the balance pin.
    pub const FRONT_EDGE: &str = "key.front_edge_mm";
    /// The capstan, behind the balance pin.
    pub const CAPSTAN_BEHIND: &str = "key.capstan.behind_mm";
    /// How far the capstan stands above the keystick.
    pub const CAPSTAN_HEIGHT: &str = "key.capstan.height_mm";
    /// The front mortise's depth above the underside.
    pub const FRONT_MORTISE_DEPTH: &str = "key.front_mortise.depth_mm";
    /// The key button's height above the keystick.
    pub const CAPSULE_HEIGHT: &str = "key.capsule_height_mm";
    /// The key button's length along the key.
    pub const CAPSULE_LENGTH: &str = "key.capsule_length_mm";
    /// The radius the key button's ends are rounded by.
    pub const CAPSULE_END_RADIUS: &str = "key.capsule_end_radius_mm";
    /// The balance mortise's length along the key.
    pub const BALANCE_MORTISE_LENGTH: &str = "key.balance_mortise.length_mm";
    /// The balance mortise's width across the key.
    pub const BALANCE_MORTISE_WIDTH: &str = "key.balance_mortise.width_mm";
    /// The balance mortise's floor above the underside.
    pub const BALANCE_MORTISE_FLOOR: &str = "key.balance_mortise.floor_mm";
    /// The balance hole's diameter.
    pub const BALANCE_HOLE_DIAMETER: &str = "key.balance_mortise.hole_diameter_mm";
    /// The bushing's thickness in the balance mortise.
    pub const BUSHING_THICKNESS: &str = "key.balance_mortise.bushing_thickness_mm";
    /// How far down from the key button's top the bushing lines the slot.
    pub const BUSHING_DEPTH: &str = "key.balance_mortise.bushing_depth_mm";
    /// The torque the bushings hold.
    pub const FRICTION_TORQUE: &str = "key.friction.torque_gmm";
    /// The key's mass as it was weighed.
    pub const WEIGHED_MASS: &str = "key.weighing.mass_g";
    /// The key's front weight as it was read.
    pub const WEIGHED_FRONT_WEIGHT: &str = "key.weighing.front_weight_g";
    /// Where the jig's roller stood when it was read, back from the lip.
    pub const WEIGHED_FROM_LIP: &str = "key.weighing.from_lip_mm";
    /// The bare balance rail's height.
    pub const BALANCE_RAIL_HEIGHT: &str = "frame.balance_rail.height_mm";
    /// The bare front rail's height.
    pub const FRONT_RAIL_HEIGHT: &str = "frame.front_rail.height_mm";
    /// Where the front rail stands, in front of the balance pin.
    pub const FRONT_RAIL_AT: &str = "frame.front_rail.at_mm";
    /// How far the front rail pin stands above its rail.
    pub const FRONT_PIN_HEIGHT: &str = "frame.front_rail.pin_height_mm";
    /// How far the balance rail pin stands above its rail.
    pub const BALANCE_PIN_HEIGHT: &str = "frame.balance_rail.pin_height_mm";
    /// The bare back rail's height.
    pub const BACK_RAIL_HEIGHT: &str = "frame.back_rail.height_mm";
    /// Where the back rail stands, behind the balance pin.
    pub const BACK_RAIL_BEHIND: &str = "frame.back_rail.behind_mm";
    /// The back rail cloth's thickness.
    pub const BACK_RAIL_CLOTH: &str = "frame.back_rail.cloth_mm";
    /// The balance punching's felt.
    pub const BALANCE_FELT: &str = "frame.balance_punching.felt_mm";
    /// The papers under the balance punching.
    pub const BALANCE_PAPERS: &str = "frame.balance_punching.papers_mm";
    /// The front punching's felt.
    pub const FRONT_FELT: &str = "frame.front_punching.felt_mm";
    /// The papers on the front punching.
    pub const FRONT_PAPERS: &str = "frame.front_punching.papers_mm";
}

/// The trade unit a path's value is in, as its name says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Millimeters, for every length.
    Millimeter,
    /// Grams, for a weight or a mass.
    Gram,
    /// Gram-force millimeters, for a torque: the grams it holds a
    /// millimeter from its axis.
    GramMillimeter,
}

/// One of a key's weighing's three readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weighing {
    /// Its mass on a scale.
    Mass,
    /// Its front weight on Stanwood's jig.
    FrontWeight,
    /// Where the jig's roller stood, back from the covering's lip.
    FromLip,
}

/// Where a path's value is held, and the domain's declaration of what it
/// admits.
#[derive(Debug, Clone, Copy)]
pub enum Field {
    /// A dimension of the key.
    Key(ParamSpec<KeyDesign>),
    /// A dimension of the frame.
    Frame(ParamSpec<FrameDesign>),
    /// The protocol's measuring point.
    Protocol(ParamSpec<TouchweightProtocol>),
    /// The bushings' torque.
    Friction(ParamSpec<AxisFriction>),
    /// The weight on the measuring point, grams.
    Weight(ParamSpec<f64>),
    /// A regulation target: aimed at through [`ActionCode::Regulate`].
    Target(ParamSpec<Targets>),
    /// A reading of the key's weighing: typed through
    /// [`ActionCode::WeighKey`].
    Weighing(Weighing),
    /// A punching's papers: laid and lifted through
    /// [`ActionCode::AddPaper`] and [`ActionCode::RemovePaper`].
    Papers(Stack),
}

/// One path of the instrument.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// The path, its trade unit in its name.
    pub path: &'static str,
    /// That unit.
    pub unit: Unit,
    /// Whether the path reads a position behind the balance pin as the
    /// distance behind it, the domain's coordinate being negative there.
    pub behind: bool,
    /// Where its value is held.
    pub field: Field,
}

impl Entry {
    /// `value`, in this path's unit, as the domain holds it, SI. A value
    /// that is not a number stays one, for the domain to refuse.
    pub fn to_si(&self, value: f64) -> f64 {
        let signed = if self.behind { -value } else { value };
        match self.unit {
            Unit::Millimeter => signed / 1000.0,
            Unit::Gram => signed,
            // Gram-force millimeters to newton-meters: a thousand twice to
            // kilogram-force meters, then standard gravity.
            Unit::GramMillimeter => signed / 1.0e6 * G,
        }
    }

    /// `si`, as the domain holds it, in this path's unit.
    pub fn from_si(&self, si: f64) -> f64 {
        let value = match self.unit {
            Unit::Millimeter => si * 1000.0,
            Unit::Gram => si,
            Unit::GramMillimeter => si / G * 1.0e6,
        };
        if self.behind {
            -value
        } else {
            value
        }
    }

    /// The domain's name of what this path sets, as a problem names it.
    pub fn param(&self) -> &'static str {
        match self.field {
            Field::Key(s) => s.name,
            Field::Frame(s) => s.name,
            Field::Protocol(s) => s.name,
            Field::Friction(s) => s.name,
            Field::Weight(s) => s.name,
            Field::Target(s) => s.name,
            Field::Weighing(Weighing::Mass) => WEIGHED_MASS,
            Field::Weighing(Weighing::FrontWeight) => WEIGHED_FRONT_WEIGHT,
            Field::Weighing(Weighing::FromLip) => WEIGHED_FROM_LIP,
            Field::Papers(_) => pa_core::parts::frame::PAPER.name,
        }
    }

    /// The action that sets this path, where `set` does not.
    pub fn set_by(&self) -> Option<ActionCode> {
        match self.field {
            Field::Target(_) => Some(ActionCode::Regulate),
            Field::Weighing(_) => Some(ActionCode::WeighKey),
            Field::Papers(_) => Some(ActionCode::AddPaper),
            _ => None,
        }
    }
}

/// The domain's name of a weighing's mass, as a problem names it.
pub const WEIGHED_MASS: &str = "weighing.mass_g";
/// The domain's name of a weighing's front weight.
pub const WEIGHED_FRONT_WEIGHT: &str = "weighing.front_weight_g";
/// The domain's name of where a weighing's roller stood.
pub const WEIGHED_FROM_LIP: &str = "weighing.from_lip";
/// The domain's name of a key friction weight typed.
pub const KEY_FRICTION_WEIGHT: &str = "key_friction_weight_g";

const fn key(path: &'static str, spec: ParamSpec<KeyDesign>, behind: bool) -> Entry {
    Entry {
        path,
        unit: Unit::Millimeter,
        behind,
        field: Field::Key(spec),
    }
}

const fn frame(path: &'static str, spec: ParamSpec<FrameDesign>, behind: bool) -> Entry {
    Entry {
        path,
        unit: Unit::Millimeter,
        behind,
        field: Field::Frame(spec),
    }
}

const fn millimeters(path: &'static str, field: Field) -> Entry {
    Entry {
        path,
        unit: Unit::Millimeter,
        behind: false,
        field,
    }
}

/// Every path of the instrument, in a fixed order: the protocol, the
/// experiment, the regulation's targets, the key, the frame.
pub const ENTRIES: [Entry; 33] = [
    millimeters(
        path::MEASURING_POINT,
        Field::Protocol(TouchweightProtocol::PARAMS[0]),
    ),
    Entry {
        path: path::WEIGHT,
        unit: Unit::Gram,
        behind: false,
        field: Field::Weight(WEIGHT_ON_THE_MEASURING_POINT),
    },
    millimeters(path::KEY_HEIGHT, Field::Target(Targets::PARAMS[0])),
    millimeters(path::KEY_DIP, Field::Target(Targets::PARAMS[1])),
    key(path::FRONT_EDGE, KeyDesign::PARAMS[0], false),
    key(path::CAPSTAN_BEHIND, KeyDesign::PARAMS[1], true),
    key(path::CAPSTAN_HEIGHT, KeyDesign::PARAMS[2], false),
    key(path::FRONT_MORTISE_DEPTH, KeyDesign::PARAMS[3], false),
    key(path::CAPSULE_HEIGHT, KeyDesign::PARAMS[4], false),
    key(path::CAPSULE_LENGTH, KeyDesign::PARAMS[5], false),
    key(path::CAPSULE_END_RADIUS, KeyDesign::PARAMS[6], false),
    key(path::BALANCE_MORTISE_LENGTH, KeyDesign::PARAMS[7], false),
    key(path::BALANCE_MORTISE_WIDTH, KeyDesign::PARAMS[8], false),
    key(path::BALANCE_MORTISE_FLOOR, KeyDesign::PARAMS[9], false),
    key(path::BALANCE_HOLE_DIAMETER, KeyDesign::PARAMS[10], false),
    key(path::BUSHING_THICKNESS, KeyDesign::PARAMS[11], false),
    key(path::BUSHING_DEPTH, KeyDesign::PARAMS[12], false),
    Entry {
        path: path::FRICTION_TORQUE,
        unit: Unit::GramMillimeter,
        behind: false,
        field: Field::Friction(AxisFriction::PARAMS[0]),
    },
    Entry {
        path: path::WEIGHED_MASS,
        unit: Unit::Gram,
        behind: false,
        field: Field::Weighing(Weighing::Mass),
    },
    Entry {
        path: path::WEIGHED_FRONT_WEIGHT,
        unit: Unit::Gram,
        behind: false,
        field: Field::Weighing(Weighing::FrontWeight),
    },
    millimeters(path::WEIGHED_FROM_LIP, Field::Weighing(Weighing::FromLip)),
    frame(path::BALANCE_RAIL_HEIGHT, FrameDesign::PARAMS[0], false),
    frame(path::FRONT_RAIL_HEIGHT, FrameDesign::PARAMS[1], false),
    frame(path::FRONT_RAIL_AT, FrameDesign::PARAMS[3], false),
    frame(path::FRONT_PIN_HEIGHT, FrameDesign::PARAMS[5], false),
    frame(path::BALANCE_PIN_HEIGHT, FrameDesign::PARAMS[9], false),
    frame(path::BACK_RAIL_HEIGHT, FrameDesign::PARAMS[2], false),
    frame(path::BACK_RAIL_BEHIND, FrameDesign::PARAMS[4], true),
    frame(path::BACK_RAIL_CLOTH, FrameDesign::PARAMS[6], false),
    frame(path::BALANCE_FELT, FrameDesign::PARAMS[7], false),
    millimeters(path::BALANCE_PAPERS, Field::Papers(Stack::Balance)),
    frame(path::FRONT_FELT, FrameDesign::PARAMS[8], false),
    millimeters(path::FRONT_PAPERS, Field::Papers(Stack::Front)),
];

/// The entry of `path`, if the instrument has one.
pub fn find(path: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.path == path)
}

/// The entry that sets what the domain names `param`, if a path sets it.
pub fn by_param(param: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.param() == param)
}

// ---- the page --------------------------------------------------------------

/// A panel of the bench's page, in the order the page shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Panel {
    /// The instrument on the bench: which profile it is built from.
    Instrument,
    /// The weighing sheet, with the measuring point above it.
    Sheet,
    /// The regulation: targets, papers, the bushings.
    Regulation,
    /// Punchings and cloth.
    Punchings,
    /// The key's own dimensions.
    Key,
    /// The key weighed whole.
    Weighing,
    /// The frame.
    Frame,
    /// The weight on the measuring point.
    Touch,
}

/// What a control is worded by: a code the page's catalog words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum Label {
    Profile,
    MeasuringPoint,
    KeyHeight,
    KeyDip,
    Regulate,
    BalanceStack,
    FrontStack,
    KeyFrictionWeight,
    BalanceFelt,
    FrontFelt,
    FrontPin,
    BalancePin,
    FrontMortise,
    BackCloth,
    Capstan,
    FrontEdge,
    CapstanHeight,
    CapsuleHeight,
    CapsuleLength,
    CapsuleEndRadius,
    BalanceMortiseLength,
    BalanceMortiseWidth,
    BalanceMortiseFloor,
    BalanceHole,
    BushingThickness,
    BushingDepth,
    KeyMass,
    FrontWeight,
    ReadAt,
    Weigh,
    BalanceRail,
    FrontRail,
    BackRail,
    FrontRailAt,
    BackRailAt,
    Place,
    Weight,
}

/// The range a control offers, in its unit: a page's choice inside the
/// hard range, which the engine never enforces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Soft {
    /// The least it offers, if it bounds it.
    pub min: Option<f64>,
    /// The most it offers, if it bounds it.
    pub max: Option<f64>,
    /// The step it moves in. A screw turns continuously and a dimension is
    /// continuous; this is how finely a slider sweeps them.
    pub step: f64,
}

const fn soft(min: f64, max: f64, step: f64) -> Soft {
    Soft {
        min: Some(min),
        max: Some(max),
        step,
    }
}

/// A field of a form: an argument of its action, labeled, the path it
/// stands for, and where its first value is read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormField {
    /// The action's argument it fills.
    pub argument: &'static str,
    /// The path the argument stands for: its unit, what it admits, how it
    /// is reached.
    pub path: &'static str,
    /// Where the field's first value is read, when the page opens.
    pub seed: Seed,
    /// What it is worded by.
    pub label: Label,
    /// What it offers.
    pub soft: Soft,
}

/// Where a form's field reads its first value: what the bench holds for
/// that quantity when the page opens, so that the form starts from the key
/// as it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seed {
    /// The value at this address of the settings document: a path, or what
    /// the settings report beside the paths, as the key's mass the statics
    /// take before anybody weighs it.
    Setting(&'static str),
    /// A reading of the sheet, as the trade reads it.
    Reading(ProcedureId),
}

/// One control of the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    /// A slider over a path `set` reaches.
    Slider {
        /// The path.
        path: &'static str,
        /// What it is worded by.
        label: Label,
        /// What it sweeps.
        soft: Soft,
    },
    /// A punching's pile of papers, laid from the drawer and lifted a paper
    /// at a time.
    Stack {
        /// The path of the papers.
        path: &'static str,
        /// The address, in the settings document, of the engine's own total
        /// of them.
        total: &'static str,
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
        fields: &'static [FormField],
    },
    /// The bushings' control: shows the key friction weight Stanwood's
    /// protocol reads them as, and sets them through the action that
    /// inverts it.
    KeyFrictionWeight {
        /// What it is worded by.
        label: Label,
        /// What it sweeps, grams.
        soft: Soft,
    },
    /// A button that runs an action with no argument.
    Button {
        /// The action.
        action: ActionCode,
        /// What it is worded by.
        label: Label,
    },
    /// A choice among the profiles the bench ships, run by an action whose
    /// one argument is the profile's name: what is on the bench.
    Profile {
        /// The action.
        action: ActionCode,
        /// The action's argument the name fills.
        argument: &'static str,
        /// What it is worded by.
        label: Label,
    },
}

/// The name of the profile the instrument was built from, in the settings
/// document.
pub const PROFILE: &str = "profile";

/// The key's mass as the statics take it, grams, in the settings document:
/// as published or as weighed, none until it is weighed. Not a path: what
/// the weighing, or the source, gives.
pub const KEY_MASS: &str = "key.mass_g";
/// The engine's total of the papers under the balance punching, in the
/// settings document.
pub const BALANCE_PAPER_TOTAL: &str = "frame.balance_punching.paper_total_mm";
/// The engine's total of the papers on the front punching.
pub const FRONT_PAPER_TOTAL: &str = "frame.front_punching.paper_total_mm";

/// The regulation's targets, sent together.
const TARGETS: [FormField; 2] = [
    FormField {
        argument: "key_height_mm",
        path: path::KEY_HEIGHT,
        seed: Seed::Setting(path::KEY_HEIGHT),
        label: Label::KeyHeight,
        soft: soft(40.0, 90.0, 0.01),
    },
    FormField {
        argument: "key_dip_mm",
        path: path::KEY_DIP,
        seed: Seed::Setting(path::KEY_DIP),
        label: Label::KeyDip,
        soft: soft(0.0, 20.0, 0.01),
    },
];

/// The key's weighing, sent together. Each field opens on the key as the
/// bench holds it: its mass as the statics take it, its front weight as the
/// sheet reads it, at the measuring point the sheet reads it at.
const WEIGHING: [FormField; 3] = [
    FormField {
        argument: "mass_g",
        path: path::WEIGHED_MASS,
        seed: Seed::Setting(KEY_MASS),
        label: Label::KeyMass,
        soft: Soft {
            min: Some(1.0),
            max: None,
            step: 0.1,
        },
    },
    FormField {
        argument: "front_weight_g",
        path: path::WEIGHED_FRONT_WEIGHT,
        seed: Seed::Reading(ProcedureId::FrontWeight),
        label: Label::FrontWeight,
        soft: Soft {
            min: None,
            max: None,
            step: 0.1,
        },
    },
    FormField {
        argument: "from_lip_mm",
        path: path::WEIGHED_FROM_LIP,
        seed: Seed::Setting(path::MEASURING_POINT),
        label: Label::ReadAt,
        soft: Soft {
            min: Some(0.0),
            max: None,
            step: 0.1,
        },
    },
];

/// The bench's page: its panels in order, each its controls in order.
pub const PAGE: [(Panel, &[Control]); 8] = [
    (
        Panel::Instrument,
        &[Control::Profile {
            action: ActionCode::SelectProfile,
            argument: "profile",
            label: Label::Profile,
        }],
    ),
    (
        Panel::Sheet,
        &[Control::Slider {
            path: path::MEASURING_POINT,
            label: Label::MeasuringPoint,
            // A tenth of a millimeter, so the glossary's 1/2 inch, 12.7 mm,
            // can be set as well as its 13 mm.
            soft: soft(5.0, 30.0, 0.1),
        }],
    ),
    (
        Panel::Regulation,
        &[
            Control::Form {
                action: ActionCode::Regulate,
                button: Label::Regulate,
                fields: &TARGETS,
            },
            Control::Stack {
                path: path::BALANCE_PAPERS,
                total: BALANCE_PAPER_TOTAL,
                stack: Stack::Balance,
                label: Label::BalanceStack,
            },
            Control::Stack {
                path: path::FRONT_PAPERS,
                total: FRONT_PAPER_TOTAL,
                stack: Stack::Front,
                label: Label::FrontStack,
            },
            Control::KeyFrictionWeight {
                label: Label::KeyFrictionWeight,
                soft: soft(0.0, 20.0, 0.5),
            },
        ],
    ),
    (
        Panel::Punchings,
        &[
            Control::Slider {
                path: path::BALANCE_FELT,
                label: Label::BalanceFelt,
                soft: soft(0.0, 4.0, 0.05),
            },
            Control::Slider {
                path: path::FRONT_FELT,
                label: Label::FrontFelt,
                soft: soft(0.0, 4.0, 0.05),
            },
            Control::Slider {
                path: path::FRONT_PIN_HEIGHT,
                label: Label::FrontPin,
                soft: soft(5.0, 40.0, 0.5),
            },
            Control::Slider {
                path: path::BALANCE_PIN_HEIGHT,
                label: Label::BalancePin,
                soft: soft(5.0, 60.0, 0.1),
            },
            Control::Slider {
                path: path::FRONT_MORTISE_DEPTH,
                label: Label::FrontMortise,
                soft: soft(5.0, 40.0, 0.5),
            },
            Control::Slider {
                path: path::BACK_RAIL_CLOTH,
                label: Label::BackCloth,
                soft: soft(0.0, 6.0, 0.05),
            },
        ],
    ),
    (
        Panel::Key,
        &[
            Control::Slider {
                path: path::CAPSTAN_BEHIND,
                label: Label::Capstan,
                soft: soft(60.0, 200.0, 1.0),
            },
            Control::Slider {
                path: path::FRONT_EDGE,
                label: Label::FrontEdge,
                soft: soft(150.0, 340.0, 1.0),
            },
            Control::Slider {
                path: path::CAPSTAN_HEIGHT,
                label: Label::CapstanHeight,
                soft: soft(4.0, 24.0, 0.5),
            },
            Control::Slider {
                path: path::CAPSULE_HEIGHT,
                label: Label::CapsuleHeight,
                soft: soft(2.0, 12.0, 0.05),
            },
            Control::Slider {
                path: path::CAPSULE_LENGTH,
                label: Label::CapsuleLength,
                soft: soft(30.0, 90.0, 0.5),
            },
            Control::Slider {
                path: path::CAPSULE_END_RADIUS,
                label: Label::CapsuleEndRadius,
                soft: soft(0.0, 30.0, 0.1),
            },
            Control::Slider {
                path: path::BALANCE_MORTISE_LENGTH,
                label: Label::BalanceMortiseLength,
                soft: soft(4.0, 20.0, 0.1),
            },
            Control::Slider {
                path: path::BALANCE_MORTISE_WIDTH,
                label: Label::BalanceMortiseWidth,
                soft: soft(3.0, 10.0, 0.05),
            },
            Control::Slider {
                path: path::BALANCE_MORTISE_FLOOR,
                label: Label::BalanceMortiseFloor,
                soft: soft(0.5, 10.0, 0.1),
            },
            Control::Slider {
                path: path::BALANCE_HOLE_DIAMETER,
                label: Label::BalanceHole,
                soft: soft(2.0, 6.0, 0.05),
            },
            Control::Slider {
                path: path::BUSHING_THICKNESS,
                label: Label::BushingThickness,
                soft: soft(0.0, 2.5, 0.05),
            },
            Control::Slider {
                path: path::BUSHING_DEPTH,
                label: Label::BushingDepth,
                soft: soft(0.0, 8.0, 0.05),
            },
        ],
    ),
    (
        Panel::Weighing,
        &[Control::Form {
            action: ActionCode::WeighKey,
            button: Label::Weigh,
            fields: &WEIGHING,
        }],
    ),
    (
        Panel::Frame,
        &[
            Control::Slider {
                path: path::BALANCE_RAIL_HEIGHT,
                label: Label::BalanceRail,
                soft: soft(10.0, 60.0, 0.1),
            },
            Control::Slider {
                path: path::FRONT_RAIL_HEIGHT,
                label: Label::FrontRail,
                soft: soft(10.0, 50.0, 0.5),
            },
            Control::Slider {
                path: path::BACK_RAIL_HEIGHT,
                label: Label::BackRail,
                soft: soft(10.0, 60.0, 0.1),
            },
            Control::Slider {
                path: path::FRONT_RAIL_AT,
                label: Label::FrontRailAt,
                soft: soft(80.0, 320.0, 1.0),
            },
            Control::Slider {
                path: path::BACK_RAIL_BEHIND,
                label: Label::BackRailAt,
                soft: soft(120.0, 280.0, 1.0),
            },
            Control::Button {
                action: ActionCode::PlaceFrameUnderKey,
                label: Label::Place,
            },
        ],
    ),
    (
        Panel::Touch,
        &[Control::Slider {
            path: path::WEIGHT,
            label: Label::Weight,
            soft: soft(-60.0, 60.0, 0.5),
        }],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Consistency: **every path is one entry, and every setting the domain
    /// declares is reached by one path**, so a problem the domain names
    /// finds the path that set it.
    #[test]
    fn every_path_and_every_declared_setting_is_reached_once() {
        for (i, e) in ENTRIES.iter().enumerate() {
            assert!(
                ENTRIES[..i].iter().all(|o| o.path != e.path),
                "{} twice",
                e.path
            );
            assert!(
                ENTRIES[..i].iter().all(|o| o.param() != e.param())
                    || matches!(e.field, Field::Papers(_)),
                "{} sets {} twice",
                e.path,
                e.param()
            );
            let suffix = match e.unit {
                Unit::Millimeter => "_mm",
                Unit::Gram => "_g",
                Unit::GramMillimeter => "_gmm",
            };
            assert!(e.path.ends_with(suffix), "{} is not in {suffix}", e.path);
        }
        let declared = KeyDesign::PARAMS
            .iter()
            .map(|s| s.name)
            .chain(FrameDesign::PARAMS.iter().map(|s| s.name))
            .chain(TouchweightProtocol::PARAMS.iter().map(|s| s.name))
            .chain(AxisFriction::PARAMS.iter().map(|s| s.name))
            .chain(Targets::PARAMS.iter().map(|s| s.name))
            .chain([WEIGHT_ON_THE_MEASURING_POINT.name]);
        for name in declared {
            assert!(by_param(name).is_some(), "no path sets {name}");
        }
    }

    /// Consistency: **every control of the page reads a path the instrument
    /// has**, each path a slider sweeps is one `set` reaches, and the
    /// range it offers lies inside the hard one.
    #[test]
    fn every_control_reads_a_path_the_instrument_has() {
        for (_, controls) in PAGE {
            for control in controls {
                match *control {
                    Control::Slider { path, soft, .. } => {
                        let e = find(path).expect(path);
                        assert_eq!(e.set_by(), None, "{path} is set by an action");
                        let hard = |spec_hard: (f64, f64)| {
                            let (a, b) = (e.from_si(spec_hard.0), e.from_si(spec_hard.1));
                            (a.min(b), a.max(b))
                        };
                        let (lo, hi) = match e.field {
                            Field::Key(s) => hard(s.hard),
                            Field::Frame(s) => hard(s.hard),
                            Field::Protocol(s) => hard(s.hard),
                            Field::Friction(s) => hard(s.hard),
                            Field::Weight(s) => hard(s.hard),
                            _ => unreachable!("{path}"),
                        };
                        let (min, max) = (soft.min.expect(path), soft.max.expect(path));
                        assert!(lo <= min && max <= hi, "{path}: {min}..{max} in {lo}..{hi}");
                    }
                    Control::Stack { path, .. } => {
                        assert!(matches!(find(path).expect(path).field, Field::Papers(_)));
                    }
                    Control::Form { action, fields, .. } => {
                        for f in fields {
                            let e = find(f.path).expect(f.path);
                            assert_eq!(e.set_by(), Some(action), "{} is not {action:?}'s", f.path);
                        }
                    }
                    Control::KeyFrictionWeight { .. }
                    | Control::Button { .. }
                    | Control::Profile { .. } => {}
                }
            }
        }
    }

    /// Consistency: **every path names the setting it reaches.** Past its
    /// prefix and short of its unit, a path is the domain's name of what it
    /// sets: a position behind the balance pin, or a rail's place, is the
    /// domain's coordinate `x`, the bushings' torque is their bound, and a
    /// weight in grams keeps its unit, as the domain's name does. So two
    /// paths cannot reach each other's settings unseen, and a path read as
    /// a distance behind the pin says so in its name.
    #[test]
    fn every_path_names_the_setting_it_reaches() {
        for e in &ENTRIES {
            assert_eq!(e.behind, e.path.ends_with(".behind_mm"), "{}", e.path);
            if let Field::Papers(stack) = e.field {
                let named = match stack {
                    Stack::Balance => path::BALANCE_PAPERS,
                    Stack::Front => path::FRONT_PAPERS,
                };
                assert_eq!(e.path, named);
                continue;
            }
            let (_, rest) = e.path.split_once('.').expect(e.path);
            let name = match e.unit {
                Unit::Millimeter => rest.strip_suffix("_mm"),
                Unit::GramMillimeter => rest.strip_suffix("_gmm"),
                Unit::Gram => Some(rest),
            }
            .expect(e.path);
            let name = match name.rsplit_once('.') {
                Some((part, "behind" | "at")) => format!("{part}.x"),
                Some(("friction", "torque")) => "friction.bound".to_owned(),
                _ => name.to_owned(),
            };
            assert_eq!(name, e.param(), "{}", e.path);
        }
    }

    /// Consistency: a path behind the balance pin reads the domain's negative
    /// coordinate as a positive distance, and back.
    #[test]
    fn a_path_behind_the_pin_reads_the_distance_behind_it() {
        let e = find(path::CAPSTAN_BEHIND).expect("the capstan");
        assert_eq!(e.to_si(129.5).to_bits(), (-0.1295_f64).to_bits());
        assert_eq!(e.from_si(-0.1295).to_bits(), 129.5_f64.to_bits());
    }
}
