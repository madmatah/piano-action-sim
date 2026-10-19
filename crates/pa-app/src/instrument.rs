//! The instrument: the single-key bench as an application composes it.
//!
//! It holds a key, its frame, the protocol it is weighed by and what an
//! experiment puts on it; it answers the questions a technician asks of them
//! and accepts the operations he performs, each by one generic call. It
//! knows nothing about JavaScript, about rendering or about files: `pa-wasm`
//! adapts it for the browser and `pa-cli` for the shell, and neither of them
//! is allowed to add physics.
//!
//! # Set and act
//!
//! Every setting has a path, its trade unit in its name, and is set by
//! [`Instrument::set`]: `frame.front_rail.pin_height_mm`. What is an
//! operation and not a value is an [`Action`], run by [`Instrument::act`]:
//! laying a paper or lifting one, regulating to targets, weighing the key
//! whole, typing the key friction weight Stanwood's protocol reads,
//! placing the frame under the key. [`crate::registry`] lists the paths and
//! what each admits.
//!
//! An edit is refused, the instrument left as it was to the bit, where it
//! would make what no part could be built as ([`pa_core::validate`]); it is
//! said why, with the limiting dimension and the shortfall, and the
//! diagnostics keep the edit refused. Everything an assembled keyboard can be,
//! however badly regulated, is taken, computed and reported: upstream
//! settings drift, they do not cascade. After every edit taken the key is
//! solved one increment from where it stood, so it is drawn at an
//! equilibrium wherever one holds it; where none has held it since it was
//! put on its frame, level on its balance punching, which places nothing;
//! and where the weight on its measuring point lifts it off its frame, where
//! its last equilibrium left it ([`Instrument::key_place`]).
//!
//! # Replayable
//!
//! Every edit taken is logged ([`Instrument::log`]), and the log replayed on
//! the instrument it started from gives the same state, bit for bit
//! ([`Instrument::replay`], [`Instrument::state_hash`]): the key's place
//! depends on the path it was taken along, since friction holds it where
//! the last weight left it, so the log keeps the path.
//!
//! # Why the key's weighing lives here and not in a panel
//!
//! Weighing the key is an **operation on the model**, not widget state: the
//! same call serves the browser, the command line and the test suite, and an
//! exercise can be scripted and replayed. A mass and a front weight typed into
//! a panel that only updated a React store would be labels beside a key that
//! weighs what it weighed before.

use pa_core::contact::{Bearing as ContactBearing, Part, Piece, Rows, ROW_MARGIN};
pub use pa_core::contact::{ContactCode, HowItBears, Material, PartCode, PlacedBy, Role, Sweep};
use pa_core::geometry::Point;
use pa_core::measure::jig::KeyJig;
/// A reading of the sheet and what it carries, re-exported so an adapter can
/// show each without reaching past the application layer.
pub use pa_core::measure::measurement::{
    Bases, Basis, Input, Measurement, ProcedureId, Rounding, Trust, Unit, Zeroed,
};
/// Why the key cannot be seated, re-exported so an adapter can say why a
/// frame cannot be weighed without reaching past the application layer.
pub use pa_core::measure::seat::SeatRefusal;
use pa_core::measure::seat::{Seating, Seats, SEAT_MARGIN, SEAT_OVERLAP};
/// The band the driven sweep draws, and why it cannot be drawn.
pub use pa_core::measure::sweep::{Band, Sample, SweepRefusal};
/// Why a walk cannot be taken to the end of the key's stroke, and which way
/// a walk or a sweep goes.
pub use pa_core::measure::walk::{Stroke, WalkRefusal};
use pa_core::measure::weighing::Weighing;
/// The weighing sheet and why a reading on it is refused, the band and what
/// is read on it, and why no bushings read a key friction weight typed.
pub use pa_core::measure::weighing::{Banded, BushingsRefusal, Refused, Sheet};
use pa_core::measure::TouchweightProtocol;
use pa_core::mechanism::{
    AxisFriction, BodyId, MassProps, MaterialPoint, Placement, PointLoad,
    Problem as MechanismProblem,
};
pub use pa_core::parts::faults::Fault;
use pa_core::parts::frame::KeyKind;
use pa_core::parts::mounts::{self, Carried, KeyMount, Mounted, OnItsFrame};
use pa_core::parts::verdicts::{edge_overlaps, holding, Holding, TOUCHING};
use pa_core::parts::{FrameDesign, FrameRanges, KeyDesign, KeyRanges};
/// What is known of the key's mass, the reading its weighing keeps, and why a
/// weighing is refused, re-exported so an adapter can show and say each
/// without reaching past the application layer.
pub use pa_core::parts::{FrontWeightReading, KeyMass, NotAWeighing};
/// The regulation's targets and outcomes, re-exported so an adapter can ask
/// for a regulation and say how it came out without reaching past the
/// application layer.
pub use pa_core::regulation::{Achieved, Limit, Target, Targets, Unsolvable};
use pa_core::regulation::{Regulating, Stacks, TargetOutcome};
use pa_core::solve::{Friction, Holder, Statics};
/// Why an edit would make what no part could be built as, re-exported so an
/// adapter can say why without reaching past the application layer.
pub use pa_core::validate::Problem;
use pa_core::validate::{numbers, validate, Design};
use pa_core::G;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use ts_rs::TS;

use crate::boundary;
use crate::profile::{Arrival, Profile, Start};
use crate::registry::{self, Field, Weighing as WeighingField};

/// What the bench reports beside its weighing sheet ([`Instrument::sheet`]),
/// in the trade's own units: how the key stands on its frame, what holds it
/// under the touch weight, and the settings the sheet was read with.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchReadout {
    /// The key's mass, grams: as a source publishes it, or as it was weighed
    /// ([`Action::WeighKey`]); `None` until it is weighed.
    pub key_mass_g: Option<f64>,
    /// Gap at the front rail punching where the key stands now, under the
    /// touch weight, millimeters: the felt's drawn top, at its center, to the
    /// key's underside. Zero when the key is down on it; `None` when the
    /// punching stands where no underside passes over its center.
    pub gap_front_mm: Option<f64>,
    /// Gap at the back rail cloth where the key stands now, millimeters: the
    /// least of the cloth's drawn top to the key's drawn underside, chamfer
    /// included. `None` when no feature of the two can meet.
    pub gap_back_mm: Option<f64>,
    /// The bushings' friction as the bench is set, the torque read as grams
    /// at the measuring point as it stands now, with the key level: the half
    /// width of the band of weights there that leave the level key still.
    /// Not the key friction weight it was typed as, which Stanwood's
    /// protocol reads on the key resting nose-up ([`Instrument::sheet`]). A
    /// torque ([`AxisFriction`]), so moving the measuring point changes this
    /// and not the bushings.
    pub key_friction_weight_g: f64,
    /// Key height the regulation was asked for, millimeters.
    pub key_height_target_mm: f64,
    /// Key dip the regulation was asked for, millimeters.
    pub key_dip_target_mm: f64,
    /// What holds the key where the weight on its measuring point has left
    /// it: a stop, or its friction; `None` for a key nobody has weighed,
    /// where a weight would leave it depending on a balance only a weighing
    /// gives.
    pub finger: Option<TouchVerdict>,
    /// How far from level the key stands at the **bottom** of its travel,
    /// seated there, degrees, nose-down positive.
    ///
    /// A keyboard is set up so this is zero: level when the key is fully down,
    /// and so nose-up by the dip when released. A design goal rather than a
    /// law — rail heights, punchings and cloth are what reach it, and plenty
    /// of keyboards miss — so the bench reports it instead of enforcing it.
    pub out_of_level_deg: f64,
    /// How far the key's bearing face is from lying flat on its cloth,
    /// seated at the top of its stroke, degrees.
    ///
    /// Positive is the face rising toward the player, so its rear end is the
    /// low one and that is what bears. The chamfer under the tail exists so
    /// that this is zero — the owner's own account of why it is there — and it
    /// is reported rather than enforced: reaching it is a property of the key's
    /// shape against the rest attitude the frame produces, and a real keyboard
    /// may miss.
    pub bedding_at_the_back_deg: f64,
    /// Where the key bears on the back rail cloth, millimeters **behind** the
    /// balance pin.
    ///
    /// Not the middle of the cloth unless the two faces are flat on one
    /// another: a key standing nose-up meets the strip at its back edge. It is
    /// reported because it is the lever arm the key turns about at rest, and
    /// it is not where a reader would assume it is.
    pub bears_at_the_back_mm: f64,
    /// Which of the three ways the key is bearing on its cloth.
    pub back_bearing: HowItBears,
    /// Whether it is the pin bottoming in its mortise, rather than the
    /// punching, that stops the key at the bottom seat.
    pub stopped_by_the_mortise: bool,
}

/// How a key whose stops fight stands on them, seated: off its balance
/// punching, on its front stop and its back rail cloth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Riding {
    /// The front stop it rides on: the front punching, or the front rail pin
    /// in its mortise or on the wood beside it.
    pub front: ContactCode,
    /// How far above its balance punching it stands, meters.
    pub lift: f64,
}

/// How the key stands at its two seats, read by the trade's gestures
/// ([`Instrument::seat_report`]).
#[derive(Debug, Clone, PartialEq)]
pub struct SeatReport {
    /// How it rides on its stops, off its balance punching, where they
    /// fight; `None` where its punching carries it.
    pub riding: Option<Riding>,
    /// How far from level it stands at the bottom of its travel, degrees,
    /// nose-down positive ([`BenchReadout::out_of_level_deg`]).
    pub out_of_level_deg: f64,
    /// What stops it at the bottom: its front punching, or the front rail
    /// pin at the top of its mortise or against the wood beside it, in the
    /// order the key's contacts are declared.
    pub bottom_held_by: Vec<ContactCode>,
    /// How it bears on its back rail cloth at the top; `None` where the
    /// cloth's rows read no bearing.
    pub back: Option<ContactBearing>,
}

impl SeatReport {
    /// How far the key's bearing face is from lying flat on its cloth at
    /// the top, degrees ([`BenchReadout::bedding_at_the_back_deg`]); `None`
    /// where no bearing or no slope is read.
    pub fn bedding_at_the_back_deg(&self) -> Option<f64> {
        Some(self.back?.slope?.atan().to_degrees())
    }

    /// Where the key bears on its cloth at the top, millimeters behind the
    /// balance pin ([`BenchReadout::bears_at_the_back_mm`]).
    pub fn bears_at_the_back_mm(&self) -> Option<f64> {
        self.back.map(|back| -back.point.x * 1000.0)
    }

    /// Whether it is the pin bottoming in its mortise, rather than the
    /// punching, that stops the key at the bottom.
    pub fn stopped_by_the_mortise(&self) -> bool {
        self.bottom_held_by
            .contains(&ContactCode::FrontPinInTheMortise)
    }
}

/// What holds the key where the weight on its measuring point has left it,
/// and the gaps at its two outer stops there ([`Instrument::touch`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Touch {
    /// What holds it.
    pub verdict: TouchVerdict,
    /// The gap at the front punching, millimeters
    /// ([`BenchReadout::gap_front_mm`]).
    pub gap_front_mm: Option<f64>,
    /// The gap at the back rail cloth, millimeters
    /// ([`BenchReadout::gap_back_mm`]).
    pub gap_back_mm: Option<f64>,
    /// How far the measuring point stands below where it stands with the
    /// key seated at the top of its stroke, millimeters: where the weight
    /// has left the key on the band's travel ([`Instrument::band`]). `None`
    /// where the key cannot be seated at the top.
    pub travel_mm: Option<f64>,
}

/// What holds the key where the weight on its measuring point has left it.
///
/// Read from determinate quantities only ([`pa_core::parts::verdicts`]):
/// which contacts are closed, and whether the generalized force of the key's
/// weight and the weight on it, every stop removed, lies inside its
/// friction's band. Never from how a static solve split a load between a
/// stop and the friction, which rigid statics leave open.
#[derive(Debug, Clone, PartialEq)]
pub enum TouchVerdict {
    /// The weight pushes the key into this stop past what its friction
    /// holds: the stop carries it.
    RestingOn {
        /// The stop.
        stop: ContactCode,
    },
    /// Friction alone holds the key where it is, touching a stop or not: the
    /// weight on it lies between its up weight and its down weight.
    HeldByFriction {
        /// The stop it touches, if it touches one.
        touching: Option<ContactCode>,
        /// How far the weight stands from moving the key, grams at the
        /// measuring point: on the cloth, from the down weight read from
        /// there, the sheet's D; on the front stop, from the up weight, the
        /// sheet's U; touching neither, from the nearer of the two at this
        /// attitude.
        margin_g: f64,
    },
    /// Off its balance punching, on its front stop and its back rail cloth:
    /// its stops fight, and hold it higher than its punching would.
    RidesOnBothStops {
        /// The front stop it rides on.
        front: ContactCode,
        /// How far above its balance punching it stands, millimeters.
        lift_mm: f64,
    },
    /// Off its balance punching, held by what it touches, with its friction
    /// or without: its cloth and its friction where a pull has turned it up
    /// on it, its front punching and its front pin against the wall of its
    /// slot, which hold it alone, where a front rail raised under it has
    /// tipped it forward off its punching.
    LiftsOffBalancePunching {
        /// The stops it touches, from the contacts closed where it stands,
        /// in the order the key's contacts are declared.
        touching: Vec<ContactCode>,
        /// Whether they need its friction to hold it there: false where
        /// they balance its weight and the weight on it alone.
        needs_friction: bool,
        /// How far above its balance punching it stands, millimeters.
        lift_mm: f64,
    },
    /// The key is lifted off its frame: no equilibrium in the model holds it
    /// on its balance pin, a weight or its stops taking its slide to the top
    /// of its range, where the key leaves the pin's drawn top, or its
    /// punching standing above that top. The key is drawn where the last
    /// equilibrium left it, or level on its punching where none has held it
    /// since the frame was edited.
    LiftsOffItsFrame,
}

/// Where the key stands on its frame: its coordinates as the last static
/// increment left them, the placement they give each body, how far above
/// its balance punching that left the key, meters, and what each contact
/// row carried there.
#[derive(Debug, Clone, PartialEq)]
struct State {
    q: Vec<f64>,
    placements: Vec<Placement>,
    lift: f64,
    carried: Carried,
}

/// Why a regulation the instrument was asked for was not laid. The papers,
/// the targets and the key stay as they were.
#[derive(Debug, Clone, PartialEq)]
pub enum NotRegulated {
    /// A target no stack reaches: which, what keeps it out of reach, and how
    /// far it lies out of reach in its own reading, meters.
    Infeasible {
        /// Which target.
        target: Target,
        /// What keeps it out of reach.
        limited_by: Limit,
        /// How far, meters.
        shortfall_m: f64,
    },
    /// The solve cannot run at all on this key and frame.
    Unsolvable(Unsolvable),
}

/// A contact as the view marks it: one row of one of the key's contacts with
/// its frame, at the attitude the key is drawn in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShownContact {
    /// Which contact.
    pub code: ContactCode,
    /// Where the key's surface is, in the keybed's frame.
    pub on_key: Point,
    /// Where the frame's surface is, in the keybed's frame.
    pub on_frame: Point,
    /// The signed gap between the two, meters: negative where the drawn parts
    /// overlap.
    pub gap: f64,
    /// The row's unit normal, in the keybed's frame, from the frame's
    /// surface toward the key's.
    pub normal: Point,
    /// The force the row pushed the key with in the last equilibrium,
    /// newtons: its multiplier, one admissible split of the loads
    /// ([`Carried`]); zero on a row that carried nothing.
    pub force: f64,
}

/// Whether a contact the view marks is closed, open or overlapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactState {
    /// Within [`CLOSED_WITHIN`] of zero either way: the two parts touch.
    Closed,
    /// Apart.
    Open,
    /// The drawn parts overlap by more than [`CLOSED_WITHIN`].
    Overlapping,
}

/// How near zero a gap must be for the contact to count as closed, meters:
/// a tenth of a micron.
///
/// A thousandth of the thinnest paper in the drawer, far below anything the
/// section can show, and a hundred million times the rounding of a contact
/// the statics have closed, which leaves a few femtometers.
pub const CLOSED_WITHIN: f64 = 1e-7;

impl ShownContact {
    /// Closed, open or overlapping.
    pub fn state(&self) -> ContactState {
        if self.gap.abs() <= CLOSED_WITHIN {
            ContactState::Closed
        } else if self.gap > 0.0 {
            ContactState::Open
        } else {
            ContactState::Overlapping
        }
    }
}

/// Where the key stands on its frame, as the view draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// The key's body in the bench's scene: the ground, which carries the frame,
/// is body 0, and the slide the key lifts off its balance punching on,
/// which carries nothing drawn, body 1.
pub const KEY: BodyId = BodyId(2);

/// How the bench stands its key on its frame: free to lift off its balance
/// punching.
const MOUNT: KeyMount = KeyMount::OnPunching;

/// One part, as the view draws it: its outline in its body's own frame, arcs
/// cut into short lines.
///
/// The physics is planar; the *part* is not. A key is 22 mm wide and the rails
/// run the length of the keyboard, so a single-key view shows a slice of them:
/// the sweep carries the width, or the axis a turned part is lathed about,
/// rather than the renderer guessing it, which keeps the solid and the section
/// the same object. Where the part stands comes separately, as its body's pose
/// ([`Instrument::poses`]): the outline does not move when the key does.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// The body that carries it: [`BodyId::GROUND`] or [`KEY`].
    pub body: BodyId,
    /// What it is.
    pub code: PartCode,
    /// Material, void or lining.
    pub role: Role,
    /// What it is made of. The view chooses its color from this.
    pub material: Material,
    /// How the outline becomes a solid.
    pub sweep: Sweep,
    /// Closed polygon, meters, in the body's frame; for a turned part, its
    /// profile in radius and height about the sweep's axis.
    pub points: Vec<Point>,
    /// The segment of the part's outline each point starts or lies on.
    pub segs: Vec<u16>,
    /// Which number of the design places each run of its segments
    /// ([`PlacedBy`]).
    pub placed_by: Vec<(core::ops::Range<u16>, PlacedBy)>,
}

/// How many times an arc's sweep is halved for drawing: a quarter circle in
/// eight pieces, whose chords stand within 2.5 µm of the cloth's 0.5 mm edges.
const ARC_HALVINGS: u32 = 3;

/// Where a contact is in a mounted key's list of pairs, as the rows name it.
fn pair_of(mounted: &Mounted, code: ContactCode) -> Option<u16> {
    let i = mounted
        .mechanism
        .pairs()
        .iter()
        .position(|p| p.code == code)?;
    u16::try_from(i).ok()
}

/// Which punching stack an action lays a paper on or lifts one from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Stack {
    /// Under the key at the balance pin. Sets key height.
    Balance,
    /// On the front rail. Sets key dip.
    Front,
}

/// An operation on the instrument, in the trade's units, as it is typed and
/// as the log keeps it: what is done to the key and its frame, not a value
/// set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Lays one more paper from the drawer on a stack, its thickness one the
    /// drawer holds, millimeters. Key leveling, or key dip.
    AddPaper {
        /// The stack.
        stack: Stack,
        /// The paper's thickness, millimeters.
        thickness_mm: f64,
    },
    /// Lifts paper `index`, counted from the bottom, off a stack.
    RemovePaper {
        /// The stack.
        stack: Stack,
        /// Which paper, from the bottom.
        index: u32,
    },
    /// Unbolts the rails and stands them where this key needs them, and
    /// does nothing else: the key height and the dip drift off their targets
    /// until the key is regulated again, shimming being a separate
    /// operation on a real bench too.
    PlaceFrameUnderKey,
    /// Regulates the key to a key height and a key dip, millimeters: key
    /// height on the balance punching first, then the dip on the front
    /// punching on the key as it then stands, each laid in papers from the
    /// drawer, the closest total from below ([`pa_core::regulation`]). The
    /// papers replace what was on the punchings; the felts stay. A target no
    /// stack reaches is refused, naming what keeps it out of reach and by
    /// how much; a dip the front pin forbids is never laid as the nearest
    /// stack and shown as a miss.
    Regulate {
        /// Key height, millimeters.
        key_height_mm: f64,
        /// Key dip, millimeters.
        key_dip_mm: f64,
    },
    /// Weighs the key whole, as it arrives, its leads in it: its mass on a
    /// scale, and its front weight read on Stanwood's leveled jig, the
    /// roller that far back from the covering's lip ([`KeyMass::weighed`]).
    /// The instrument keeps the mass and the static moment the front weight
    /// gives on the arm it was read at, that reading its datum: moving the
    /// measuring point afterwards, or editing the key, reads the same moment
    /// on another arm, and the front weight on the sheet follows by the
    /// lever. The height of the center of mass, which no scale gives, is
    /// the one the profile publishes where it publishes one, and otherwise
    /// estimated from the key's drawn section
    /// ([`KeyDesign::section_centroid`]).
    WeighKey {
        /// The key's mass, grams.
        mass_g: f64,
        /// Its front weight as the jig's scale read it, grams.
        front_weight_g: f64,
        /// How far back from the covering's lip the roller stood,
        /// millimeters.
        from_lip_mm: f64,
    },
    /// Sets the bushings from a key friction weight, grams at the measuring
    /// point, as the trade measures it: by Stanwood's protocol, zeroed on the
    /// leveled jig, the least weight that makes the key fall from its
    /// cloth. The bushings become the torque at which that protocol, run on
    /// this key, reads it in full ([`Weighing::bushings_reading`]), stored
    /// with the measuring point it was typed at: moving the measuring point
    /// or editing the key afterwards leaves the bushings alone and changes
    /// the grams every protocol reads them as, as it would at the bench.
    SetFrictionFromKeyFrictionWeight {
        /// The key friction weight, grams.
        kf_g: f64,
    },
    /// Puts the instrument back as its own profile builds it
    /// ([`Instrument::profile`]), whatever it was edited into; the log
    /// kept.
    Reset,
    /// Puts another instrument on the bench: the one the shipped profile of
    /// that name builds ([`Profile::names`]), as it arrives, which a reset
    /// then returns to; the log kept.
    SelectProfile {
        /// The profile's name.
        profile: String,
    },
}

/// An action by its name, as a page names the one a control runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum ActionCode {
    AddPaper,
    RemovePaper,
    PlaceFrameUnderKey,
    Regulate,
    WeighKey,
    SetFrictionFromKeyFrictionWeight,
    Reset,
    SelectProfile,
}

impl Action {
    /// Its name.
    pub fn code(&self) -> ActionCode {
        match self {
            Self::AddPaper { .. } => ActionCode::AddPaper,
            Self::RemovePaper { .. } => ActionCode::RemovePaper,
            Self::PlaceFrameUnderKey => ActionCode::PlaceFrameUnderKey,
            Self::Regulate { .. } => ActionCode::Regulate,
            Self::WeighKey { .. } => ActionCode::WeighKey,
            Self::SetFrictionFromKeyFrictionWeight { .. } => {
                ActionCode::SetFrictionFromKeyFrictionWeight
            }
            Self::Reset => ActionCode::Reset,
            Self::SelectProfile { .. } => ActionCode::SelectProfile,
        }
    }
}

/// An edit, as the log keeps it and as a refusal names it: a setting set by
/// its path, in the path's unit, or an action.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// [`Instrument::set`].
    Set {
        /// The path.
        path: String,
        /// The value, in the path's unit.
        value: f64,
    },
    /// [`Instrument::act`].
    Act(Action),
}

/// What an action taken gives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    /// It was done.
    Done,
    /// The regulation laid its papers, and they give this, read again by
    /// the seated pose: within one paper of the targets.
    Laid(Achieved),
}

/// Why an edit was refused. The instrument stays as it was, to the bit.
#[derive(Debug, Clone, PartialEq)]
pub enum EditRefusal {
    /// It would make what no part could be built as
    /// ([`pa_core::validate`]).
    Problem(Problem),
    /// The instrument has no such path.
    UnknownPath {
        /// The path asked for.
        path: String,
    },
    /// The path is reached through an action, not set.
    SetByAction {
        /// The path.
        path: &'static str,
        /// The action that sets it.
        action: ActionCode,
    },
    /// A stack holds no paper of that index.
    NoSuchPaper {
        /// The stack.
        stack: Stack,
        /// The index asked for.
        index: u32,
        /// How many papers the stack holds.
        papers: usize,
    },
    /// A target no stack reaches, or a solve that cannot run.
    Regulation(NotRegulated),
    /// A weighing no key has.
    Weighing(NotAWeighing),
    /// A key friction weight no bushings read.
    Friction(BushingsRefusal),
    /// No shipped profile has that name.
    UnknownProfile {
        /// The name asked for.
        profile: String,
    },
}

impl From<Problem> for EditRefusal {
    fn from(p: Problem) -> Self {
        Self::Problem(p)
    }
}

/// An edit the instrument refused, and why: what the diagnostics keep, the
/// last refused since an edit was last taken.
#[derive(Debug, Clone, PartialEq)]
pub struct RefusedEdit {
    /// The edit.
    pub edit: Edit,
    /// Why.
    pub why: EditRefusal,
}

/// FNV-1a, 64 bits, over bytes in a fixed order: a hash whose algorithm is
/// fixed here, as `std`'s `DefaultHasher`'s is not, so a state hashes alike
/// in every build and every release.
struct Fnv(u64);

impl Fnv {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;

    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn f64(&mut self, x: f64) {
        self.bytes(&x.to_bits().to_le_bytes());
    }

    fn u64(&mut self, x: u64) {
        self.bytes(&x.to_le_bytes());
    }

    fn flag(&mut self, b: bool) {
        self.bytes(&[u8::from(b)]);
    }

    /// A row's piece, by its kind, its part and its segment.
    fn piece(&mut self, piece: Piece) {
        let (kind, part, seg) = match piece {
            Piece::Segment { part, seg } => (0, part, seg),
            Piece::Arc { part, seg } => (1, part, seg),
            Piece::Vertex { part, seg } => (2, part, seg),
            Piece::Center { part } => (3, part, 0),
        };
        self.bytes(&[kind]);
        self.u64(u64::from(part.0));
        self.u64(u64::from(seg));
    }

    /// What each row carried: the rows in the solve's order, each by what
    /// it is, with its gap, its force and its normal, then the balance
    /// punching's push.
    fn carried(&mut self, carried: &Carried) {
        self.u64(carried.supports().len() as u64);
        for s in carried.supports() {
            match s.by {
                Holder::Contact { pair, feature } => {
                    self.bytes(&[0]);
                    self.u64(u64::from(pair));
                    self.piece(feature.a);
                    self.piece(feature.b);
                }
                Holder::Limit { coord, upper } => {
                    self.bytes(&[1]);
                    self.u64(coord.0 as u64);
                    self.flag(upper);
                }
            }
            for x in [s.gap, s.force, s.normal.x, s.normal.y] {
                self.f64(x);
            }
        }
        match carried.punching() {
            None => self.flag(false),
            Some((pair, push)) => {
                self.flag(true);
                self.u64(u64::from(pair));
                self.f64(push);
            }
        }
    }
}

/// The instrument: a key on its frame, weighed by a protocol, an experiment
/// on it.
#[derive(Debug, Clone)]
pub struct Instrument {
    /// The profile it was built from, and returns to on
    /// [`Action::Reset`]: where every number it started from comes from.
    profile: &'static Profile,
    /// The key.
    key: KeyDesign,
    /// What is known of the key's mass: as a source publishes it, as it was
    /// weighed whole, or nothing until it is.
    mass: KeyMass,
    /// The frame it sits on, the papers laid on its punchings with it: the
    /// coordinate regulation sets, held there once.
    frame: FrameDesign,
    /// How the key is weighed: where on it every weight is centered. Held
    /// once, because every touchweight reads it, so none can be weighed at a
    /// point the others are not.
    protocol: TouchweightProtocol,
    /// Whether this key is a natural or a sharp — which row of pins it stands
    /// on, and later which graduation band it belongs to.
    kind: KeyKind,
    /// The bushings' friction, a torque with the radius it was typed at.
    friction: AxisFriction,
    /// What that friction rests on: the bench's assumption until it is set.
    friction_basis: Basis,
    /// The weight sitting on the measuring point, grams.
    weight_g: f64,
    /// The targets of the regulation last laid, kept beside what the key
    /// reads.
    targets: Targets,
    /// Where the key stands on its frame, as the last equilibrium left it:
    /// what every touch weight and every edit is applied to, as an
    /// increment. `None` where no equilibrium has held the key since it was
    /// put on its frame.
    state: Option<State>,
    /// Whether the weight on the measuring point lifts the key off its
    /// frame: the last increment found no equilibrium, the key's slide
    /// leaning on the top of its range. The state is then the last
    /// equilibrium, where the key is drawn.
    lifts_off: bool,
    /// Whether the last edit put the key back by the initialization
    /// instead of moving it from where it stood.
    reinitialized: bool,
    /// Every edit taken, in order.
    log: Vec<Edit>,
    /// The last edit refused since one was taken, and why.
    refused: Option<RefusedEdit>,
}

impl Instrument {
    /// The instrument `profile` describes, built as it arrives.
    pub fn of(profile: &'static Profile) -> Result<Self, EditRefusal> {
        Self::new(profile.start.clone(), profile)
    }

    /// An instrument that starts from `start`, `profile` being the one it
    /// returns to on [`Action::Reset`]: the key on its frame with the felts
    /// it is given and, where it arrives regulated, on them the papers the
    /// regulation solve lays to its targets ([`pa_core::regulation`]), the
    /// papers the frame held replaced; where it arrives as recorded, the
    /// papers it holds. The bushings are read on this key: bushings given
    /// as a weight at the measuring point hold that weight on this key's
    /// arm, not on the arm of the key a profile describes.
    ///
    /// Refused where no part could be built as the design ([`validate`]), or
    /// where the targets cannot be met on this geometry, with why: a real
    /// answer and not an error to swallow.
    pub fn new(start: Start, profile: &'static Profile) -> Result<Self, EditRefusal> {
        let Start {
            key,
            kind,
            mass,
            frame,
            protocol,
            bushings,
            friction_basis,
            targets,
            weight_g,
            arrival,
        } = start;
        let friction = bushings.on(&key, &protocol);
        let mut instrument = Self {
            profile,
            key,
            mass,
            frame,
            protocol,
            kind,
            friction,
            friction_basis,
            weight_g,
            targets,
            state: None,
            lifts_off: false,
            reinitialized: false,
            log: Vec::new(),
            refused: None,
        };
        validate(&instrument.design())?;
        if arrival == Arrival::Regulated {
            let (stacks, _) = instrument.solve(targets).map_err(EditRefusal::Regulation)?;
            instrument.lay(&stacks);
        }
        // Then put the key on the frame so built, with what is on it.
        instrument.initialize();
        Ok(instrument)
    }

    /// The standard instrument: the standard profile's
    /// ([`crate::profile::STANDARD`]), Hirschkorn's key on his printed frame,
    /// as it arrives.
    pub fn standard() -> Self {
        Self::of(Profile::standard()).expect("the standard instrument is built")
    }

    /// The owner's measured section weighed by the tests' weighing
    /// ([`Profile::weighed_section`]), as it arrives.
    #[cfg(test)]
    pub(crate) fn weighed_section() -> Self {
        Self::of(Profile::weighed_section()).expect("the weighed section is regulable")
    }

    /// The profile this instrument was built from: where every number it
    /// started from comes from, and what [`Action::Reset`] returns it to.
    pub fn profile(&self) -> &'static Profile {
        self.profile
    }

    // ---- set and act -----------------------------------------------------

    /// **Sets the setting at `path`** to `value`, in the path's unit
    /// ([`registry`]), and solves the key one increment from where it
    /// stands.
    ///
    /// Refused, the instrument left as it was to the bit, where the path is
    /// none of the instrument's, is reached through an action, or the value
    /// would make what no part could be built as: not a number, outside the
    /// setting's hard range, or a design [`validate`] refuses. Everything
    /// else is taken, however badly it regulates the key: a dimension
    /// changed does not re-solve the punchings, the regulation drifts and
    /// the sheet shows the miss, as on a real keyboard.
    ///
    /// The ratios have no path: the key weight ratio is the ratio of two
    /// arms, an output of the geometry, read on the sheet.
    pub fn set(&mut self, path: &str, value: f64) -> Result<(), EditRefusal> {
        match self.try_set(path, value) {
            Ok(()) => {
                self.taken(Edit::Set {
                    path: path.to_owned(),
                    value,
                });
                self.relax();
                Ok(())
            }
            Err(why) => Err(self.refuse(
                Edit::Set {
                    path: path.to_owned(),
                    value,
                },
                why,
            )),
        }
    }

    /// **Runs `action`** ([`Action`]), and solves the key one increment from
    /// where it stands; or says why not, the instrument left as it was to
    /// the bit.
    pub fn act(&mut self, action: Action) -> Result<Outcome, EditRefusal> {
        match self.try_act(&action) {
            Ok(outcome) => {
                self.taken(Edit::Act(action));
                Ok(outcome)
            }
            Err(why) => Err(self.refuse(Edit::Act(action), why)),
        }
    }

    /// Every edit taken, in order: [`Self::replay`] on the instrument this
    /// one started as gives this one again.
    pub fn log(&self) -> &[Edit] {
        &self.log
    }

    /// This instrument with `log` replayed on it, edit by edit; or the first
    /// edit refused, by its place in the log, and why.
    pub fn replay(mut self, log: &[Edit]) -> Result<Self, (usize, EditRefusal)> {
        for (i, edit) in log.iter().enumerate() {
            match edit {
                Edit::Set { path, value } => self.set(path, *value),
                Edit::Act(action) => self.act(action.clone()).map(|_| ()),
            }
            .map_err(|why| (i, why))?;
        }
        Ok(self)
    }

    /// The last edit refused since one was taken, and why.
    pub fn refused_edit(&self) -> Option<&RefusedEdit> {
        self.refused.as_ref()
    }

    /// **The state's hash**: FNV-1a over the name of the profile a reset
    /// returns to and the bit patterns of every normative coordinate, the
    /// key's, its weighing's, the frame's with every paper on its punchings
    /// and the drawer they are laid from, the protocol's, the bushings' and
    /// the experiment's, the targets, and the hidden state, where the last
    /// equilibrium left the key, what each contact row carried there and
    /// whether it lifts off its frame, in a fixed order: a hash whose
    /// algorithm is fixed here, as `std`'s `DefaultHasher`'s is not, so a
    /// state hashes alike in every build and every release. Two instruments
    /// that hash alike read the same, and replay a log alike; the log is not
    /// hashed, since two paths to one state are one state.
    pub fn state_hash(&self) -> u64 {
        // Every field, so that one added and not hashed fails to compile:
        // the key, its weighing, the frame, the protocol and the bushings
        // are read through the design's numbers, which read every field of
        // theirs; the profile by its name, which a reset rebuilds; the log
        // and the last refusal are not state.
        let Self {
            profile,
            key: _,
            mass: _,
            frame,
            protocol: _,
            kind,
            friction: _,
            friction_basis,
            weight_g,
            targets,
            state,
            lifts_off,
            reinitialized,
            log: _,
            refused: _,
        } = self;
        let mut h = Fnv(Fnv::OFFSET);
        h.bytes(profile.name.as_bytes());
        for (name, x) in numbers(&self.design()) {
            h.bytes(name.as_bytes());
            h.f64(x);
        }
        for papers in [&frame.balance_punching.papers, &frame.front_punching.papers] {
            h.u64(papers.len() as u64);
        }
        h.flag(match kind {
            KeyKind::Natural => false,
            KeyKind::Sharp => true,
        });
        h.u64(match friction_basis {
            Basis::Measured => 0,
            Basis::Published => 1,
            Basis::Estimated => 2,
            Basis::Assumed => 3,
            Basis::SetByHand => 4,
        });
        h.f64(*weight_g);
        h.f64(targets.key_height);
        h.f64(targets.key_dip);
        match state {
            None => h.flag(false),
            Some(State {
                q,
                placements: _,
                lift,
                carried,
            }) => {
                // The placements are the coordinates' pose; the split of
                // the loads is the solve's, one admissible among others,
                // and the state reads it.
                h.flag(true);
                h.u64(q.len() as u64);
                for &x in q {
                    h.f64(x);
                }
                h.f64(*lift);
                h.carried(carried);
            }
        }
        h.flag(*lifts_off);
        h.flag(*reinitialized);
        h.0
    }

    /// The value at `path`, in its unit; `None` for a path the instrument
    /// has not, for a list, or for a weighing nobody has taken.
    pub fn value(&self, path: &str) -> Option<f64> {
        let entry = registry::find(path)?;
        let si = match entry.field {
            Field::Key(spec) => (spec.get)(&self.key),
            Field::Frame(spec) => (spec.get)(&self.frame),
            Field::Protocol(spec) => (spec.get)(&self.protocol),
            Field::Friction(spec) => (spec.get)(&self.friction),
            Field::Weight(spec) => (spec.get)(&self.weight_g),
            Field::Target(spec) => (spec.get)(&self.targets),
            Field::Weighing(which) => {
                let KeyMass::Weighed { mass, datum, .. } = self.mass else {
                    return None;
                };
                match which {
                    // The mass, kilograms; the front weight, grams as the
                    // scale read it; the roller's distance, meters.
                    WeighingField::Mass => return Some(mass * 1000.0),
                    WeighingField::FrontWeight => return Some(datum.grams),
                    WeighingField::FromLip => datum.from_lip_m,
                }
            }
            Field::Papers(_) => return None,
        };
        Some(entry.from_si(si))
    }

    /// The papers on `stack`, meters each, bottom first.
    pub fn papers(&self, stack: Stack) -> &[f64] {
        &self.punching(stack).papers
    }

    /// Every setting, in the trade's units ([`boundary::Settings`]).
    pub fn settings(&self) -> boundary::Settings {
        boundary::Settings::of(self)
    }

    /// Every path, what it admits and how it is reached, and the page that
    /// presents them ([`boundary::Registry`]).
    pub fn describe(&self) -> boundary::Registry {
        boundary::Registry::of(self)
    }

    /// How the key stands on its frame and why, and the last edit refused
    /// ([`boundary::Diagnostics`]).
    pub fn diagnostics(&self) -> boundary::Diagnostics {
        boundary::Diagnostics::of(self)
    }

    /// Where each body stands, and the contacts the view marks
    /// ([`boundary::State`]).
    pub fn state(&self) -> boundary::State {
        boundary::State::of(self)
    }

    /// What is drawn, each part in its own body's frame
    /// ([`boundary::Scene`]).
    pub fn scene(&self) -> boundary::Scene {
        boundary::Scene::of(self)
    }

    /// One reading of the weighing sheet, by its procedure run on the model,
    /// or why it is refused ([`Self::sheet`]).
    pub fn measure(&self, procedure: ProcedureId) -> Result<Measurement, Refused> {
        self.sheet()
            .get(procedure)
            .cloned()
            .expect("the sheet reads every procedure")
    }

    /// Every number of the instrument's design, by name, as
    /// [`pa_core::validate::numbers`] reads them.
    pub fn numbers(&self) -> Vec<(&'static str, f64)> {
        numbers(&self.design())
    }

    /// The instrument's design, as [`validate`] reads it.
    fn design(&self) -> Design<'_> {
        Design {
            key: &self.key,
            mass: &self.mass,
            frame: &self.frame,
            protocol: &self.protocol,
            friction: &self.friction,
        }
    }

    /// An edit taken: logged, and the last refusal forgotten.
    fn taken(&mut self, edit: Edit) {
        self.log.push(edit);
        self.refused = None;
    }

    /// An edit refused, kept for the diagnostics; returns why.
    fn refuse(&mut self, edit: Edit, why: EditRefusal) -> EditRefusal {
        self.refused = Some(RefusedEdit {
            edit,
            why: why.clone(),
        });
        why
    }

    /// The stack of papers and felt on `stack`'s punching.
    fn punching(&self, stack: Stack) -> &pa_core::parts::PunchingStack {
        match stack {
            Stack::Balance => &self.frame.balance_punching,
            Stack::Front => &self.frame.front_punching,
        }
    }

    /// [`Self::set`], the instrument written only where it is taken.
    fn try_set(&mut self, path: &str, value: f64) -> Result<(), EditRefusal> {
        let entry = registry::find(path).ok_or_else(|| EditRefusal::UnknownPath {
            path: path.to_owned(),
        })?;
        if let Some(action) = entry.set_by() {
            return Err(EditRefusal::SetByAction {
                path: entry.path,
                action,
            });
        }
        let si = entry.to_si(value);
        match entry.field {
            Field::Key(spec) => {
                let mut key = self.key.clone();
                spec.apply(&mut key, si)?;
                validate(&Design {
                    key: &key,
                    ..self.design()
                })?;
                self.key = key;
            }
            Field::Frame(spec) => {
                let mut frame = self.frame.clone();
                spec.apply(&mut frame, si)?;
                validate(&Design {
                    frame: &frame,
                    ..self.design()
                })?;
                self.frame = frame;
            }
            Field::Protocol(spec) => {
                let mut protocol = self.protocol;
                spec.apply(&mut protocol, si)?;
                validate(&Design {
                    protocol: &protocol,
                    ..self.design()
                })?;
                self.protocol = protocol;
            }
            Field::Friction(spec) => {
                let mut friction = self.friction;
                spec.apply(&mut friction, si)?;
                validate(&Design {
                    friction: &friction,
                    ..self.design()
                })?;
                self.friction = friction;
                self.friction_basis = Basis::SetByHand;
            }
            Field::Weight(spec) => spec.apply(&mut self.weight_g, si)?,
            Field::Target(_) | Field::Weighing(_) | Field::Papers(_) => {
                unreachable!("{path} is set by an action")
            }
        }
        Ok(())
    }

    /// [`Self::act`], the instrument written only where it is taken.
    fn try_act(&mut self, action: &Action) -> Result<Outcome, EditRefusal> {
        match *action {
            Action::SelectProfile { ref profile } => {
                let named = Profile::named(profile).ok_or_else(|| EditRefusal::UnknownProfile {
                    profile: profile.clone(),
                })?;
                let fresh = Self::of(named)?;
                let log = core::mem::take(&mut self.log);
                *self = fresh;
                self.log = log;
                return Ok(Outcome::Done);
            }
            Action::AddPaper {
                stack,
                thickness_mm,
            } => {
                // The drawer's own thickness, as the settings send it in
                // millimeters or as it is in meters; anything else is said
                // why by the drawer.
                let thickness_m = thickness_mm / 1000.0;
                let drawer = &self.frame.drawer;
                let paper = match drawer.thicknesses().iter().copied().find(|t| {
                    (t * 1000.0).to_bits() == thickness_mm.to_bits()
                        || t.to_bits() == thickness_m.to_bits()
                }) {
                    Some(paper) => paper,
                    None => {
                        drawer.admits(thickness_m)?;
                        thickness_m
                    }
                };
                let mut frame = self.frame.clone();
                match stack {
                    Stack::Balance => frame.balance_punching.papers.push(paper),
                    Stack::Front => frame.front_punching.papers.push(paper),
                }
                validate(&Design {
                    frame: &frame,
                    ..self.design()
                })?;
                self.frame = frame;
            }
            Action::RemovePaper { stack, index } => {
                let papers = self.punching(stack).papers.len();
                let i = usize::try_from(index).ok().filter(|&i| i < papers).ok_or(
                    EditRefusal::NoSuchPaper {
                        stack,
                        index,
                        papers,
                    },
                )?;
                match stack {
                    Stack::Balance => self.frame.balance_punching.papers.remove(i),
                    Stack::Front => self.frame.front_punching.papers.remove(i),
                };
            }
            Action::PlaceFrameUnderKey => {
                let frame = self.frame.clone().placed_under(&self.key);
                validate(&Design {
                    frame: &frame,
                    ..self.design()
                })?;
                self.frame = frame;
            }
            Action::Regulate {
                key_height_mm,
                key_dip_mm,
            } => {
                let mut targets = self.targets;
                for (spec, mm) in Targets::PARAMS.iter().zip([key_height_mm, key_dip_mm]) {
                    spec.apply(&mut targets, mm / 1000.0)?;
                }
                let (stacks, achieved) = self.solve(targets).map_err(EditRefusal::Regulation)?;
                self.targets = targets;
                self.lay(&stacks);
                self.relax();
                return Ok(Outcome::Laid(achieved));
            }
            Action::WeighKey {
                mass_g,
                front_weight_g,
                from_lip_mm,
            } => {
                for (param, x) in [
                    (registry::WEIGHED_MASS, mass_g),
                    (registry::WEIGHED_FRONT_WEIGHT, front_weight_g),
                    (registry::WEIGHED_FROM_LIP, from_lip_mm),
                ] {
                    if !x.is_finite() {
                        return Err(Problem::NotFinite { param }.into());
                    }
                }
                if mass_g <= 0.0 {
                    return Err(Problem::MassNotPositive {
                        mass: mass_g / 1000.0,
                    }
                    .into());
                }
                let mass =
                    KeyMass::weighed(&self.key, mass_g, front_weight_g, from_lip_mm / 1000.0)
                        .map_err(EditRefusal::Weighing)?;
                validate(&Design {
                    mass: &mass,
                    ..self.design()
                })?;
                self.mass = mass;
            }
            Action::SetFrictionFromKeyFrictionWeight { kf_g } => {
                if !kf_g.is_finite() {
                    return Err(Problem::NotFinite {
                        param: registry::KEY_FRICTION_WEIGHT,
                    }
                    .into());
                }
                let found = self
                    .weigh(|w| w.bushings_reading(&mut Statics::new(), kf_g))
                    .map_err(|r| EditRefusal::Friction(BushingsRefusal::Procedure(r)))?
                    .map_err(EditRefusal::Friction)?;
                let friction = AxisFriction {
                    bound_n_m: found.bound_n_m,
                    radius_m: Some(self.measuring_point()),
                };
                validate(&Design {
                    friction: &friction,
                    ..self.design()
                })?;
                self.friction = friction;
                self.friction_basis = Basis::SetByHand;
            }
            Action::Reset => {
                let fresh = Self::of(self.profile)?;
                let log = core::mem::take(&mut self.log);
                *self = fresh;
                self.log = log;
                return Ok(Outcome::Done);
            }
        }
        self.relax();
        Ok(Outcome::Done)
    }

    // ---- the key on its frame ----------------------------------------------

    /// The regulation solve on this key and frame, as the bench weighs and
    /// reads them: the papers each target asks for, laid from the drawer,
    /// and what they give.
    fn solve(&self, targets: Targets) -> Result<(Stacks, Achieved), NotRegulated> {
        let outcome = Regulating {
            key: &self.key,
            frame: &self.frame,
            kind: self.kind,
            mass: self.mass_props(),
            protocol: &self.protocol,
        }
        .solve(targets)
        .map_err(NotRegulated::Unsolvable)?;
        match outcome {
            TargetOutcome::Met {
                stacks, achieved, ..
            } => Ok((stacks, achieved)),
            TargetOutcome::Infeasible {
                target,
                limited_by,
                shortfall_m,
            } => Err(NotRegulated::Infeasible {
                target,
                limited_by,
                shortfall_m,
            }),
        }
    }

    /// Lays the stacks' papers on the two punchings, in place of what was
    /// there: the felts stay, the papers are the ones the drawer holds.
    fn lay(&mut self, stacks: &Stacks) {
        self.frame.balance_punching.papers = stacks.balance.papers.clone();
        self.frame.front_punching.papers = stacks.front.papers.clone();
    }

    /// The targets of the regulation last laid, meters.
    pub fn targets(&self) -> Targets {
        self.targets
    }

    /// The key, as it stands.
    pub fn key(&self) -> &KeyDesign {
        &self.key
    }

    /// The frame, as it stands, the papers laid on its punchings.
    pub fn frame(&self) -> &FrameDesign {
        &self.frame
    }

    /// The protocol the key is weighed by.
    pub fn protocol(&self) -> &TouchweightProtocol {
        &self.protocol
    }

    /// The measuring point, meters in front of the balance pin: the
    /// protocol's, on this key ([`TouchweightProtocol::measuring_point`]).
    pub fn measuring_point(&self) -> f64 {
        self.protocol.measuring_point(&self.key)
    }

    /// Sine of the key's current angle, positive front-down: where the last
    /// equilibrium left it, riding on its stops off its balance punching on a
    /// frame whose stops fight, or level where nothing has held it.
    pub fn sin_angle(&self) -> f64 {
        -self.key_placement().sin
    }

    /// Whether the last action put the key back by the initialization
    /// instead of moving it from where it stood: an edit that left it sunk
    /// in its frame deeper than the row margin, or one from which no static
    /// increment could be solved (`docs/FOUNDATIONS.md` > Friction).
    pub fn reinitialized(&self) -> bool {
        self.reinitialized
    }

    /// The initialization: the key put on its frame from its nominal pose
    /// under the weight on its measuring point, friction off
    /// (`docs/FOUNDATIONS.md` > Friction); where it finds no state from there, from
    /// the key lowered onto its frame from above
    /// ([`mounts::lowered_onto_its_frame`]), as the seats are. Building the
    /// bench runs it, and so does an edit the key cannot follow. The state it
    /// finds needs no friction, so with friction on the key stays there.
    ///
    /// A key with no pose on its balance pin within the row margin of
    /// clearing its frame, its nominal pose sunk in the frame deeper than
    /// that, clears its frame only off its pin: it is off its frame, as the
    /// seats find it, and no solve starts from where no row reads it. A key
    /// its nominal pose carries off its pin, a start at the top of the
    /// slide's range a whole degree from where it rides below, is put on its
    /// frame from above instead, and is off it only where that finds no
    /// state either.
    fn initialize(&mut self) {
        self.lifts_off = false;
        if self.mass_props().is_none() {
            self.state = self.seated_at_the_top();
            return;
        }
        self.state = match self.on_frame() {
            Ok(k) => {
                let loads = [self.touch_load(&k.mounted)];
                let place = |start: &[f64]| {
                    k.mounted
                        .solve(&mut Statics::new(), start, &loads, Friction::Off)
                };
                let start = mounts::nominal_pose(&k.mounted);
                let from_above = || {
                    mounts::lowered_onto_its_frame(&k.mounted)
                        .filter(|above| *above != start)
                        .and_then(|above| place(&above).ok())
                        .and_then(|eq| Self::state_at(&k.mounted, &eq))
                };
                if mounts::overlap_on_its_frame(&k.mounted, &start) > ROW_MARGIN {
                    self.lifts_off = true;
                    None
                } else {
                    match place(&start) {
                        Ok(eq) => Self::state_at(&k.mounted, &eq).or_else(from_above),
                        // Carried off its pin from its nominal pose, which
                        // can stand at the top of the slide's range a whole
                        // degree from where the key rides below it: off its
                        // frame unless it finds a state from above.
                        Err(refusal) if k.mounted.lifts_off_its_frame(&refusal) => {
                            let state = from_above();
                            self.lifts_off = state.is_none();
                            state
                        }
                        Err(_) => from_above(),
                    }
                }
            }
            Err(_) => None,
        };
    }

    /// A key nobody has weighed, where the seated pose puts it at the top of
    /// its stroke, its back held down on its cloth by the load standing for
    /// the action, as key height is read: where a weight would leave a key
    /// nobody has weighed depends on a balance only a weighing gives, so it
    /// is drawn there whatever the weight on its measuring point. `None` where it
    /// cannot be seated.
    fn seated_at_the_top(&self) -> Option<State> {
        let (k, seats) = self.seated().ok()?;
        let eq = k
            .mounted
            .solve(&mut Statics::new(), &seats.top.q, &[], Friction::Off)
            .ok()?;
        Self::state_at(&k.mounted, &eq)
    }

    /// Applies the bench's last action to the key, as one static increment
    /// from where it stands with its friction on: a weight on the measuring
    /// point, a weighing, a paper, any edit of the key or of its frame
    /// (`docs/FOUNDATIONS.md` > Friction). The key stays where it was left if it
    /// still can, which is the band the touch exercise teaches; a key resting
    /// on its balance punching rises and falls with it as papers are laid
    /// under it, as a key on its punching does. A weight that lifts the key
    /// off its frame finds no equilibrium: the key stays drawn where the last
    /// one left it, and the bench says the weight lifts it off
    /// ([`TouchVerdict::LiftsOffItsFrame`]), whether the increment found it
    /// so or, failing, the initialization did. An edit that leaves it sunk
    /// in its frame deeper than the row margin, its rails included, which no
    /// contact reads ([`mounts::overlap_on_its_frame`]), or from which no
    /// increment can be solved otherwise, starts it again from the
    /// initialization, and [`Self::reinitialized`] says so.
    fn relax(&mut self) {
        if self.mass_props().is_none() {
            self.initialize();
            self.reinitialized = false;
            return;
        }
        // Whether the key's last state, unmoved, still stands clear of its
        // frame, so that it can stay drawn there if no equilibrium is found.
        let mut clear = false;
        let outcome = self.state.as_ref().and_then(|state| {
            let k = self.on_frame().ok()?;
            let mut from = state.q.clone();
            if let Some(lift) = &k.mounted.lift {
                if state.lift <= TOUCHING {
                    from[lift.coord.0] = lift.on_punching + state.lift;
                }
            }
            if mounts::overlap_on_its_frame(&k.mounted, &from) > ROW_MARGIN {
                return None;
            }
            clear = from == state.q;
            let loads = [self.touch_load(&k.mounted)];
            match k
                .mounted
                .solve(&mut Statics::new(), &from, &loads, Friction::On)
            {
                Ok(eq) => Self::state_at(&k.mounted, &eq).map(Ok),
                Err(refusal) if k.mounted.lifts_off_its_frame(&refusal) => Some(Err(())),
                Err(_) => None,
            }
        });
        self.reinitialized = outcome.is_none() && self.state.is_some();
        match outcome {
            Some(Ok(state)) => {
                self.state = Some(state);
                self.lifts_off = false;
            }
            Some(Err(())) => self.lifts_off = true,
            None => {
                let last = self.state.clone();
                self.initialize();
                // The initialization finds the key lifting off its frame
                // where the increment found nothing: the last equilibrium
                // still stands clear of the frame, and the key is drawn
                // there.
                if clear && self.lifts_off && self.state.is_none() {
                    self.state = last;
                    self.reinitialized = false;
                }
            }
        }
    }

    /// The state `eq` leaves, or none where the key's drawn parts stand sunk
    /// in its frame's deeper than [`SEAT_OVERLAP`], where no contact reads
    /// them and the key is not on its frame at all.
    fn state_at(mounted: &Mounted, eq: &pa_core::solve::Equilibrium) -> Option<State> {
        let pose = mounted.mechanism.pose(&eq.q);
        if mounted.mechanism.overlap(&pose) > SEAT_OVERLAP {
            return None;
        }
        Some(State {
            placements: (0..mounted.mechanism.bodies().len())
                .map(|b| pose.placement(BodyId(b)))
                .collect(),
            lift: mounted.lift_at(&eq.q),
            carried: mounted.carried(eq),
            q: eq.q.clone(),
        })
    }

    /// The weight on the measuring point, as a load on the key.
    fn touch_load(&self, mounted: &Mounted) -> PointLoad {
        PointLoad {
            at: MaterialPoint {
                body: mounted.key,
                at: self.protocol.measuring_point_on_key(&self.key),
            },
            force: Point::new(0.0, -self.weight_g * 1e-3 * G),
        }
    }

    /// The key's coordinates where it is drawn: where the last equilibrium
    /// left it, or level on its balance punching where nothing has held it.
    fn drawn(&self, mounted: &Mounted) -> Vec<f64> {
        self.state
            .as_ref()
            .map_or_else(|| mounted.level(), |state| state.q.clone())
    }

    /// Why this frame cannot be weighed, if it cannot: the seated pose's
    /// refusal ([`SeatRefusal`]), which every reading that seats the key
    /// carries too.
    ///
    /// A frame whose stops fight is not one: its key is seated riding on
    /// them, off its balance punching ([`Self::riding`]), and only what needs
    /// a stroke is refused. What is left has no seat in the model: a stop
    /// standing in the key's drawn parts that no motion of the key on its
    /// frame clears, stops that would hold it off its frame, nothing under
    /// its front or its tail, or geometry edited into nonsense.
    pub fn frame_fault(&self) -> Option<SeatRefusal> {
        self.seated().err()
    }

    /// How the key rides on its stops, seated at the top of its stroke: off
    /// its balance punching, on its front stop and its back rail cloth, where
    /// its stops fight and hold it higher than its punching would. `None`
    /// where its punching carries it, or where it cannot be seated.
    pub fn riding(&self) -> Option<Riding> {
        let (_, seats) = self.seated().ok()?;
        Self::riding_at(&seats)
    }

    /// How the key rides on its stops at the top seat of `seats`, if it
    /// does.
    fn riding_at(seats: &Seats) -> Option<Riding> {
        let top = &seats.top;
        top.rides().then(|| Riding {
            front: top
                .held_by
                .iter()
                .copied()
                .find(|&c| c != ContactCode::BackRailCloth)
                .unwrap_or(ContactCode::FrontRailPunching),
            lift: top.lift,
        })
    }

    /// How the key stands at its two seats, read once: whether it rides on
    /// its stops, how far from level it stands at the bottom and what stops
    /// it there, and how it bears on its cloth at the top; or why it cannot
    /// be seated ([`Self::frame_fault`]).
    pub fn seat_report(&self) -> Result<SeatReport, SeatRefusal> {
        let (k, seats) = self.seated()?;
        Ok(Self::report_on(&k.mounted, &seats))
    }

    /// The report of `seats`, the key mounted as `mounted`.
    fn report_on(mounted: &Mounted, seats: &Seats) -> SeatReport {
        let bottom_angle = seats.bottom.q[mounted.angle.0];
        SeatReport {
            riding: Self::riding_at(seats),
            // Nose-down positive; a product, not a platform's math library.
            out_of_level_deg: (-bottom_angle).to_degrees(),
            bottom_held_by: seats.bottom.held_by.clone(),
            back: Self::back_bearing(mounted, seats),
        }
    }

    /// Everything about this key and this frame that no keyboard could be
    /// built from, each with how far past the limit it is, read off the drawn
    /// parts.
    ///
    /// Empty on a keyboard that goes together. It sits here with
    /// [`Self::frame_fault`] and not on the weighing sheet, for the same
    /// reason: a frame edited until it cannot be weighed is exactly when a
    /// reader needs to be told what is wrong with it, and that is exactly when
    /// [`Self::readout`] returns nothing. Faults are a property of the
    /// geometry, which is always there.
    ///
    /// It reports rather than refuses — see [`Fault`].
    pub fn faults(&self) -> Vec<Fault> {
        let (parts, key, frame) = self.parts();
        pa_core::parts::faults::faults(&parts, &key, &frame)
    }

    /// Every part of the key and of its frame, the frame's first, on the
    /// ground, then the key's, on [`KEY`]; with what each declares.
    ///
    /// The frame's papers are the ones actually laid, one part each, because
    /// that is what the drawer holds and what a technician counts.
    pub fn parts(&self) -> (Vec<Part>, KeyRanges, FrameRanges) {
        let mut parts = Vec::new();
        let frame = self.frame.parts(&self.key, self.kind, &mut parts);
        let key = self.key.parts(KEY, &mut parts);
        (parts, key, frame)
    }

    /// Every part to draw, each in its own body's frame, arcs cut into
    /// lines; where each body stands is [`Self::poses`].
    pub fn outlines(&self) -> Vec<Outline> {
        let (parts, _, _) = self.parts();
        parts
            .into_iter()
            .map(|part| {
                let mut points = Vec::new();
                let mut segs = Vec::new();
                for (i, seg) in (0u16..).zip(&part.outline) {
                    for p in seg.drawn(ARC_HALVINGS) {
                        points.push(p);
                        segs.push(i);
                    }
                }
                Outline {
                    body: part.body,
                    code: part.code,
                    role: part.role,
                    material: part.material,
                    sweep: part.sweep,
                    points,
                    segs,
                    placed_by: part.placed_by,
                }
            })
            .collect()
    }

    /// Where each body of the scene stands: the ground's frame is the
    /// world's, the slide the key lifts on stands on the balance pin's axis,
    /// and the key's frame where its statics put it. One placement per body,
    /// in body order, applied by the view as it is given.
    pub fn poses(&self) -> Vec<Placement> {
        match &self.state {
            Some(state) => state.placements.clone(),
            None => {
                let level = self.level();
                vec![Placement::WORLD, level, level]
            }
        }
    }

    /// The key's frame level on its balance punching, where it is drawn when
    /// nothing has held it: its origin on the balance pin's axis, at the
    /// punching's top.
    fn level(&self) -> Placement {
        Placement {
            origin: Point::new(
                self.frame.sections.balance.pin_x(0.0, self.kind),
                self.frame.balance_bearing_height(),
            ),
            ..Placement::WORLD
        }
    }

    /// Where the key stands on its frame, as it is drawn
    /// ([`Self::key_placement`]): placed by an equilibrium, or not and why
    /// not.
    pub fn key_place(&self) -> KeyPlace {
        match (&self.state, self.lifts_off) {
            (None, _) => KeyPlace::Unplaced,
            (Some(_), true) => KeyPlace::LiftedOff,
            (Some(_), false) => KeyPlace::Placed,
        }
    }

    /// Whether an equilibrium places the key where it is drawn
    /// ([`Self::key_place`]).
    pub fn is_placed(&self) -> bool {
        self.key_place() == KeyPlace::Placed
    }

    /// Where the key's frame stands in the keybed's: its origin, at the
    /// balance pin on the underside, and its rotation, positive nose-up.
    ///
    /// Where the last equilibrium left the key, as its mechanism places it:
    /// on its balance punching, or riding on its stops off it; level on its
    /// punching where nothing has held it. Nothing else places a point of the
    /// key.
    pub fn key_placement(&self) -> Placement {
        self.state
            .as_ref()
            .map_or_else(|| self.level(), |state| state.placements[KEY.0])
    }

    /// The key on its frame as a mechanism, with the points the procedures
    /// load and read ([`mounts::on_its_frame`], which the regulation solve
    /// mounts the key with too): free to lift off its balance punching, its
    /// bushings' friction the bench's bound.
    ///
    /// The key turns about its balance pin, at the origin of its frame, and
    /// bears on its balance punching at the punching's center, on the pin's
    /// axis: where the felt is drawn, which is where every key the bench
    /// builds bears.
    fn on_frame(&self) -> Result<OnItsFrame, MechanismProblem> {
        mounts::on_its_frame(
            MOUNT,
            &self.key,
            &self.frame,
            self.kind,
            self.mass_props(),
            self.friction.bound_n_m,
        )
    }

    /// The key's mass and center of mass as the statics take them
    /// ([`KeyMass::props`]): as a source publishes them, whole; as it was
    /// weighed, at the height [`Self::center_of_mass_height`] gives, which
    /// no scale does. `None` until the key is weighed, and where the key's
    /// section encloses nothing.
    fn mass_props(&self) -> Option<MassProps> {
        self.mass.props(self.center_of_mass_height()?)
    }

    /// The height of the key's center of mass, which no scale gives: the
    /// one its profile publishes, where it publishes one, which a key of
    /// that profile weighed again on the bench keeps; otherwise its drawn
    /// section's area centroid's, an estimate ([`KeyDesign::section_centroid`]).
    /// `None` where the section encloses nothing.
    fn center_of_mass_height(&self) -> Option<f64> {
        match self.profile.start.mass {
            KeyMass::Published { center, .. } => Some(center.y),
            KeyMass::Unweighed | KeyMass::Weighed { .. } => {
                self.key.section_centroid().map(|c| c.y)
            }
        }
    }

    /// The key on its frame as a mechanism: its parts and its frame's, the
    /// contacts between them ([`mounts::key_on_frame`],
    /// [`mounts::balance_punching`]), and the key free to lift off its
    /// balance punching.
    ///
    /// The parts are [`Self::parts`]' own, the ones the view draws, so every
    /// contact reads a range of a drawn part. `None` only if the parts cannot
    /// be read, which the builders never produce.
    pub fn key_on_frame(&self) -> Option<Mounted> {
        self.on_frame().ok().map(|k| k.mounted)
    }

    /// The key seated at the top of its stroke and at the bottom, by the
    /// trade's gestures ([`pa_core::measure::seat`]): where key height, dip
    /// and the bearings at the back and at the bottom are read, whatever the
    /// weight on the measuring point is doing.
    pub(crate) fn seated(&self) -> Result<(OnItsFrame, Seats), SeatRefusal> {
        let k = self.on_frame().map_err(SeatRefusal::Mechanism)?;
        let seats = Seating {
            mounted: &k.mounted,
            capstan: k.capstan,
            measuring_point: self.protocol.measuring_point_on_key(&self.key),
            margin: SEAT_MARGIN,
            standing: &[],
        }
        .seats(&mut Statics::new())?;
        Ok((k, seats))
    }

    /// Every contact row of `mounted` at `q`, within `margin`.
    fn rows_at(mounted: &Mounted, q: &[f64], margin: f64) -> Rows {
        let pose = mounted.mechanism.pose(q);
        let mut rows = Rows::new();
        mounted.mechanism.rows(&pose, margin, &mut rows);
        rows
    }

    /// The contacts the view marks: every row within the margin the static
    /// solve reads, where the key is drawn, its balance punching's included.
    /// Where no equilibrium places the key ([`Self::is_placed`]), only the
    /// rows where its drawn parts overlap the frame's: a gap read off a pose
    /// no equilibrium holds says nothing of what bears.
    pub fn contacts(&self) -> Vec<ShownContact> {
        let Some(mounted) = self.key_on_frame() else {
            return Vec::new();
        };
        let rows = Self::rows_at(&mounted, &self.drawn(&mounted), ROW_MARGIN);
        let pairs = mounted.mechanism.pairs();
        let nothing = Carried::NOTHING;
        let carried = self.state.as_ref().map_or(&nothing, |state| &state.carried);
        rows.rows()
            .iter()
            .map(|r| ShownContact {
                code: pairs[usize::from(r.pair)].code,
                on_key: r.on_a(),
                on_frame: r.on_b(),
                gap: r.gap,
                normal: r.normal.vector(),
                force: carried.force(r.pair, r.feature),
            })
            .filter(|contact| self.is_placed() || contact.state() == ContactState::Overlapping)
            .collect()
    }

    /// How far each punching the key bears on, read at its center, overlaps
    /// it where the key is drawn, meters: what its felt would compress under
    /// a key out of level on it, which the rigid reading at the center
    /// leaves aside ([`edge_overlaps`]). Under the key at rest on its
    /// balance punching, nose-up by its dip, a quarter of a millimeter. None
    /// where no equilibrium placed the key ([`Self::is_placed`]): it bears on
    /// no punching there.
    pub fn edge_overlaps(&self) -> Vec<(ContactCode, f64)> {
        let Some(mounted) = self.key_on_frame().filter(|_| self.is_placed()) else {
            return Vec::new();
        };
        edge_overlaps(&mounted, &self.drawn(&mounted))
    }

    /// How the key bears on its back rail cloth seated at the top of its
    /// stroke: on the cloth's edge, flat on it, or on its own corner, where,
    /// and its face's slope over the cloth's.
    pub fn bearing_at_the_back(&self) -> Option<ContactBearing> {
        let (k, seats) = self.seated().ok()?;
        Self::back_bearing(&k.mounted, &seats)
    }

    /// How the key bears on its cloth at the top seat of `seats`.
    fn back_bearing(mounted: &Mounted, seats: &Seats) -> Option<ContactBearing> {
        let pose = mounted.mechanism.pose(&seats.top.q);
        let rows = Self::rows_at(mounted, &seats.top.q, f64::INFINITY);
        let pair = pair_of(mounted, ContactCode::BackRailCloth)?;
        mounted.mechanism.bearing(&pose, &rows, pair)
    }

    /// The measuring point, in the key's frame: on the keytop's top, the
    /// protocol's distance back from the drawn lip.
    ///
    /// Not an anchor, because it moves with a setting of the protocol and not
    /// with the wood. Exposed because the view must mark it — a reader has to
    /// see *where* the engine thinks it is weighing — and because a named
    /// point is the only honest probe for "has the front of the key moved":
    /// the outline's extreme vertex changes identity as the key turns.
    pub fn measuring_point_on_key(&self) -> Point {
        let (parts, key, _) = self.parts();
        let lip = key.lip.point(&parts);
        Point::new(lip.x - self.protocol.measuring_point_from_lip, lip.y)
    }

    /// **The weighing sheet**: every reading, by its procedure run on the
    /// model ([`pa_core::measure::weighing`]), each a measurement or the
    /// reason it is refused. Front weight and the key weight ratio on the jig,
    /// key height and dip from the seated pose, D and U by the breakaway walk
    /// from each seat, balance and friction weight from those two as read,
    /// and the key friction weight both ways, Stanwood's zeroed on the jig
    /// and its variant zeroed on the frame. The key's mass is its source's or
    /// its weighing ([`Action::WeighKey`]); until it is weighed, every
    /// reading that reads it is refused ([`Refused::KeyNotWeighed`]).
    ///
    /// The procedures seat the key by rule, so the sheet is the same whatever
    /// the touch weight has done to it.
    pub fn sheet(&self) -> Sheet {
        self.weigh(|w| w.sheet(&mut Statics::new()))
            .unwrap_or_else(|why| Sheet::refused(&why))
    }

    /// The bench's key put where the procedures read it, on its frame on
    /// the bench's bushings and on the jig, at its measuring point, and
    /// handed to `read`; refused where either mount is degenerate.
    fn weigh<T>(&self, read: impl FnOnce(&Weighing<'_>) -> T) -> Result<T, Refused> {
        let key = &self.key;
        let protocol = &self.protocol;
        let k = self.on_frame().map_err(Refused::Mechanism)?;
        let jig = KeyJig::new(key, protocol, self.mass_props()).map_err(Refused::Mechanism)?;
        Ok(read(&Weighing {
            on_frame: &k,
            jig: &jig,
            measuring_point: protocol.measuring_point_on_key(key),
            measuring_point_from_lip_m: protocol.measuring_point_from_lip,
            thinnest_paper_m: self.frame.drawer.thinnest(),
            bases: self.bases(),
        }))
    }

    /// What the bench's inputs rest on: the key's mass and static moment as
    /// a source publishes them or as the bench's user weighed it; the
    /// height of its center of mass as published where the profile
    /// publishes one, estimated from its drawn section otherwise, since no
    /// scale gives it; its friction the profile's until it is set; its
    /// dimensions partly without a record, assumptions its profile states
    /// (`data/profiles/`).
    pub fn bases(&self) -> Bases {
        let published = |mass: KeyMass| matches!(mass, KeyMass::Published { .. });
        Bases {
            key_mass: match self.mass {
                // Read by no reading: every reading that reads the key's mass
                // is refused until it is weighed.
                KeyMass::Unweighed => Basis::Assumed,
                KeyMass::Weighed { .. } => Basis::SetByHand,
                KeyMass::Published { .. } => Basis::Published,
            },
            center_of_mass_height: if published(self.mass) || published(self.profile.start.mass) {
                Basis::Published
            } else {
                Basis::Estimated
            },
            key_friction: self.friction_basis,
            geometry: Basis::Assumed,
        }
    }

    /// How the key stands on its frame and what holds it there: the seats'
    /// bearing at the back and out of level at the bottom, the gaps and the
    /// verdict where the weight on the measuring point has left the key, and
    /// the settings the sheet is read with.
    ///
    /// `None` where the key cannot be seated ([`Self::frame_fault`] says
    /// why) or has nowhere to stand.
    pub fn readout(&self) -> Option<BenchReadout> {
        let (k, seats) = self.seated().ok()?;
        let finger = match self.mass_props() {
            Some(_) => Some(self.finger_verdict()?),
            None => None,
        };
        let (gap_front_mm, gap_back_mm) = self.gaps_mm(&k.mounted);
        let report = Self::report_on(&k.mounted, &seats);
        let back = report.back?;

        Some(BenchReadout {
            bedding_at_the_back_deg: report.bedding_at_the_back_deg()?,
            bears_at_the_back_mm: -back.point.x * 1000.0,
            back_bearing: back.how,
            stopped_by_the_mortise: report.stopped_by_the_mortise(),
            out_of_level_deg: report.out_of_level_deg,
            gap_front_mm,
            gap_back_mm,
            key_friction_weight_g: self.key_friction_weight_g(),
            key_height_target_mm: self.targets.key_height * 1000.0,
            key_dip_target_mm: self.targets.key_dip * 1000.0,
            finger,
            key_mass_g: self.key_mass_g(),
        })
    }

    /// The gaps at the front punching and at the back rail cloth where the
    /// key is drawn, millimeters, read off the contact rows at the attitude
    /// it stands in: `None` where no underside passes over the punching's
    /// center, or where no feature of the cloth and the key can meet, and
    /// where no equilibrium placed the key, whose drawn attitude is none of
    /// its own ([`Self::is_placed`]).
    fn gaps_mm(&self, mounted: &Mounted) -> (Option<f64>, Option<f64>) {
        if !self.is_placed() {
            return (None, None);
        }
        let rows = Self::rows_at(mounted, &self.drawn(mounted), f64::INFINITY);
        let gap_mm = |code| Some(rows.gap(pair_of(mounted, code)?)? * 1000.0);
        (
            gap_mm(ContactCode::FrontRailPunching),
            gap_mm(ContactCode::BackRailCloth),
        )
    }

    /// What holds the key where the weight on its measuring point has left
    /// it, and the gaps at its front punching and its back rail cloth
    /// there: wherever the key stands on its frame, seated by the
    /// procedures or not, since a weight on a key needs no seat. `None`
    /// where nothing holds it ([`Self::finger_verdict`]).
    pub fn touch(&self) -> Option<Touch> {
        let verdict = self.finger_verdict()?;
        let k = self.on_frame().ok();
        let (gap_front_mm, gap_back_mm) = k
            .as_ref()
            .map_or((None, None), |k| self.gaps_mm(&k.mounted));
        Some(Touch {
            verdict,
            gap_front_mm,
            gap_back_mm,
            travel_mm: k.and_then(|k| self.travel_mm(&k)),
        })
    }

    /// How far the measuring point stands below where it stands with the
    /// key seated at the top of its stroke, where the key is drawn,
    /// millimeters; `None` where the key cannot be seated at the top, or
    /// where no equilibrium placed it ([`Self::is_placed`]).
    fn travel_mm(&self, k: &OnItsFrame) -> Option<f64> {
        if !self.is_placed() {
            return None;
        }
        let point = self.protocol.measuring_point_on_key(&self.key);
        let top = Seating {
            mounted: &k.mounted,
            capstan: k.capstan,
            measuring_point: point,
            margin: SEAT_MARGIN,
            standing: &[],
        }
        .top(&mut Statics::new())
        .ok()?;
        let at = MaterialPoint {
            body: k.mounted.key,
            at: point,
        };
        let height = |q: &[f64]| k.mounted.mechanism.pose(q).world(at).y;
        Some((height(&top.q) - height(&self.drawn(&k.mounted))) * 1000.0)
    }

    /// **The band**: the key swept through its stroke, from its top seat to
    /// its bottom one and back, the force on the measuring point read at
    /// every step, and the down and up weights read on it
    /// ([`Weighing::band`]); or why not. Taken when it is asked for, never
    /// with the sheet: an interface asks for it when it shows the band.
    pub fn band(&self) -> Result<Banded, Refused> {
        self.weigh(|w| w.band(&mut Statics::new()))?
    }

    /// What holds the key at `state` under the weight on its measuring
    /// point, the friction's margin turned into grams at the measuring
    /// point by the arm the key presents there.
    fn verdict(&self, mounted: &Mounted, state: &State) -> Option<TouchVerdict> {
        let load = self.touch_load(mounted);
        Some(match holding(mounted, &state.q, &[load])? {
            Holding::RestingOn { stop } => TouchVerdict::RestingOn { stop },
            Holding::RidesOnBothStops { front, lift } => TouchVerdict::RidesOnBothStops {
                front,
                lift_mm: lift * 1000.0,
            },
            Holding::LiftsOffBalancePunching {
                touching,
                needs_friction,
                lift,
            } => TouchVerdict::LiftsOffBalancePunching {
                touching,
                needs_friction,
                lift_mm: lift * 1000.0,
            },
            Holding::HeldByFriction { touching, margin } => {
                // A gram at the measuring point turns the key by its weight
                // times the point's vertical speed per radian.
                let m = &mounted.mechanism;
                let arm = m
                    .jacobian_column(&m.pose(&state.q), load.at, mounted.angle)
                    .y
                    .abs();
                TouchVerdict::HeldByFriction {
                    touching,
                    margin_g: 1000.0 * margin / (G * arm),
                }
            }
        })
    }

    /// What is known of the key's mass: as a source publishes it, as it was
    /// weighed, with the front weight reading it keeps, or nothing.
    pub fn key_mass(&self) -> KeyMass {
        self.mass
    }

    /// The key's mass, grams, as the statics take it: as a source publishes
    /// it, or as it was weighed. `None` until it is weighed.
    pub fn key_mass_g(&self) -> Option<f64> {
        self.mass_props().map(|props| props.mass * 1000.0)
    }

    /// How far back from the front lip of the key every weight is centered,
    /// meters: the protocol's measuring point.
    pub fn measuring_point_from_lip(&self) -> f64 {
        self.protocol.measuring_point_from_lip
    }

    /// The weight currently on the measuring point, grams.
    pub fn weight_g(&self) -> f64 {
        self.weight_g
    }

    /// What holds the key where the weight on its measuring point has left
    /// it: a stop, its friction, its two stops off its balance punching; or
    /// that the weight lifts it off its frame. `None` for a key nobody has
    /// weighed, whose balance only a weighing gives; and where
    /// nothing holds it for any other reason: a key that has not been put on
    /// its frame, with nothing under it.
    pub fn finger_verdict(&self) -> Option<TouchVerdict> {
        self.mass_props()?;
        if self.lifts_off {
            return Some(TouchVerdict::LiftsOffItsFrame);
        }
        let mounted = self.key_on_frame()?;
        self.verdict(&mounted, self.state.as_ref()?)
    }

    /// The bushings' friction as it is stored: a torque, with the radius it
    /// was typed at.
    pub fn friction(&self) -> AxisFriction {
        self.friction
    }

    /// The bushings' friction as the bench is set, grams at the measuring
    /// point as it stands now, the key level: the torque read on that arm.
    /// Not what Stanwood's protocol reads them as, on the key resting
    /// nose-up: the sheet's key friction weight.
    pub fn key_friction_weight_g(&self) -> f64 {
        self.friction.weight_at(self.measuring_point())
    }
}

/// Reading the instrument's sheet, and editing it the way the tests'
/// arithmetic reads it, for the tests.
#[cfg(test)]
pub(crate) mod read {
    use super::{
        Action, EditRefusal, FrameDesign, Instrument, KeyDesign, Measurement, ProcedureId, Profile,
        Stack, Start, Targets,
    };
    use crate::registry::path;
    use pa_core::measure::walk::{HALVINGS, STEP_G};
    use pa_core::solve::statics::RESIDUAL_TOLERANCE;
    use pa_core::G;

    /// Bushings that hold `grams` at the measuring point with the key
    /// level, set as the torque that is, through its path: for the tests
    /// whose arithmetic reads the friction on the level arm, not through
    /// Stanwood's protocol.
    pub(crate) fn bushings_holding_g(b: &mut Instrument, grams: f64) {
        let torque_gmm = grams * b.measuring_point() * 1000.0;
        b.set(path::FRICTION_TORQUE, torque_gmm)
            .expect("bushings that hold a weight");
    }

    /// `stack` laid again from the drawer for `total` meters of paper: every
    /// paper lifted off, then the drawer's closest stack from below laid,
    /// one action a paper.
    pub(crate) fn lay(b: &mut Instrument, stack: Stack, total: f64) {
        while !b.papers(stack).is_empty() {
            b.act(Action::RemovePaper { stack, index: 0 })
                .expect("a paper lifted");
        }
        let papers = b.frame().drawer.papers_for(total);
        for paper in papers {
            b.act(Action::AddPaper {
                stack,
                thickness_mm: paper * 1000.0,
            })
            .expect("a paper from the drawer");
        }
    }

    /// An instrument on the measured section weighed by the tests'
    /// weighing, built from this key and frame and regulated to these
    /// targets.
    pub(crate) fn built(
        key: KeyDesign,
        frame: FrameDesign,
        targets: Targets,
    ) -> Result<Instrument, EditRefusal> {
        let profile = Profile::weighed_section();
        Instrument::new(
            Start {
                key,
                frame,
                targets,
                ..profile.start.clone()
            },
            profile,
        )
    }

    /// The static moment of `b`'s weighed key about its balance pin,
    /// kilogram-meters: its mass times where its center of mass stands
    /// along the key.
    pub(crate) fn static_moment_kg_m(b: &Instrument) -> f64 {
        match b.key_mass() {
            super::KeyMass::Weighed {
                static_moment_n_m, ..
            } => static_moment_n_m / G,
            other => panic!("a weighed key, not {other:?}"),
        }
    }

    /// A reading of `b`'s sheet, which must not be refused.
    pub(crate) fn measured(b: &Instrument, p: ProcedureId) -> Measurement {
        match b.sheet().get(p) {
            Some(Ok(m)) => m.clone(),
            other => panic!("{p:?}: {other:?}"),
        }
    }

    /// Its value: grams, the ratio, or meters.
    pub(crate) fn value(b: &Instrument, p: ProcedureId) -> f64 {
        measured(b, p).value
    }

    /// A height or a dip, millimeters.
    pub(crate) fn mm(b: &Instrument, p: ProcedureId) -> f64 {
        1000.0 * value(b, p)
    }

    /// What a walk resolves a threshold to, grams: half the bracket its
    /// halvings leave, and what the static solve may leave out of balance
    /// where it decides whether the key moves, its residual's tolerance of a
    /// load scale under two newtons, over a reach under 0.3 m, read at a
    /// measuring arm of at least 0.2 m.
    pub(crate) fn resolution() -> f64 {
        0.5 * STEP_G / f64::from(1u32 << HALVINGS)
            + 1000.0 * RESIDUAL_TOLERANCE * 2.0 * 0.3 / (G * 0.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::path;

    /// The bench reports the regulation it was asked for.
    #[test]
    fn a_bench_regulated_on_arrival_reports_what_it_achieved_beside_what_it_was_asked_for() {
        let b = Instrument::weighed_section();
        let r = b.readout().expect("the bench reads");
        let (height, dip) = (
            read::mm(&b, ProcedureId::KeyHeight),
            read::mm(&b, ProcedureId::KeyDip),
        );

        // Target and achieved are two numbers, and the bench carries both. They
        // differ for one physical reason, reported rather than hidden: the
        // stack is built from real papers, which cannot hit an arbitrary
        // thickness. The regulation solve asks for exactly what each target
        // needs, read by the same seated pose as the sheet, and the drawer lays
        // the closest total from below.
        //
        // So the bound is not a tolerance anybody chose — it is **one
        // paper**, seen through each stack's lever. The thinnest in the drawer
        // is 0.08 mm: under the balance punching it raises the key front by
        // 0.08 x (258 + 212) / 212, the key turning about where it bears on its
        // cloth, 212 mm behind the pin; on the front punching it shallows the
        // dip by 0.08 x 258/225. A regulation that lands inside that is as
        // close as the drawer allows; one that does not has a real error in it.
        let paper_mm = pa_core::test_fixture::PAPER_THICKNESSES[0] * 1000.0;
        let behind = r.bears_at_the_back_mm;
        let one_paper_height_mm = paper_mm * (258.0 + behind) / behind;
        let one_paper_dip_mm = paper_mm * (258.0 / 225.0);

        assert!((r.key_height_target_mm - 65.0).abs() < 1e-12);
        assert!((r.key_dip_target_mm - 10.2).abs() < 1e-12);
        assert!(
            (height - r.key_height_target_mm).abs() < one_paper_height_mm,
            "height achieved {height} against target {}, missing by more than one paper",
            r.key_height_target_mm
        );
        assert!(
            (dip - r.key_dip_target_mm).abs() < one_paper_dip_mm,
            "dip achieved {dip} against target {}, missing by more than one paper",
            r.key_dip_target_mm
        );

        let ratio = read::value(&b, ProcedureId::KeyWeightRatio);
        assert!((0.3..=1.0).contains(&ratio), "ratio {ratio}");
        let mass_g = r.key_mass_g.expect("a weighed key");
        assert!(mass_g > 40.0 && mass_g < 160.0, "mass {mass_g} g");
    }

    /// Consistency, the audit's BWA-10: **a weight between the up weight and
    /// the down weight leaves the key where it is, and the bench says that
    /// friction holds it, and how far the weight stands from moving it, by
    /// the sheet's own D and U.**
    ///
    /// Put a weight on the measuring point and three things can happen, not
    /// two. Past the down weight the key goes down onto its front punching,
    /// past the up weight it comes up onto its cloth, and between them it
    /// **stays exactly where the last weight left it**, on either stop — that
    /// gap is the friction, and it is the most visible consequence of
    /// modeling dry friction as set-valued rather than as a `tanh`. A
    /// regularized model would creep instead.
    ///
    /// D is read by the walk from the key resting on its cloth, U from the
    /// key down on its punching, and the touch weight leaves the key on those
    /// same stops at the same attitudes. So on the cloth the margin the
    /// verdict reports is how far the weight stands below D, and on the
    /// punching how far above U, both to the walk's resolution: the panel's
    /// "between U and D" and the sheet are one reading.
    #[test]
    fn a_weight_between_up_and_down_weight_leaves_the_key_where_it_is() {
        let mut b = Instrument::weighed_section();
        let (down, up) = (
            read::value(&b, ProcedureId::DownWeight),
            read::value(&b, ProcedureId::UpWeight),
        );
        let middle = 0.5 * (down + up);
        let rest_on = |stop| Some(TouchVerdict::RestingOn { stop });
        // A hundredth of a gram from the edge each stop is left past: on the
        // cloth D, on the punching U.
        for (onto, past, stop, near) in [
            (
                up - 1.0,
                down + 1.0,
                ContactCode::BackRailCloth,
                down - 0.01,
            ),
            (
                down + 1.0,
                up - 1.0,
                ContactCode::FrontRailPunching,
                up + 0.01,
            ),
        ] {
            b.set(path::WEIGHT, onto).expect("a weight");
            assert_eq!(b.finger_verdict(), rest_on(stop), "{onto} g");
            let there = b.sin_angle();
            for w in [up + 1.0, middle, down - 1.0, near] {
                b.set(path::WEIGHT, w).expect("a weight");
                let Some(TouchVerdict::HeldByFriction { touching, margin_g }) = b.finger_verdict()
                else {
                    panic!("{w} g on the {stop:?}: {:?}", b.finger_verdict());
                };
                assert_eq!(touching, Some(stop), "{w} g");
                assert_eq!(
                    b.sin_angle().to_bits(),
                    there.to_bits(),
                    "{w} g moved the key off the {stop:?}"
                );
                let expected = match stop {
                    ContactCode::BackRailCloth => down - w,
                    _ => w - up,
                };
                assert!(
                    (margin_g - expected).abs() <= read::resolution(),
                    "{w} g on the {stop:?}: a margin of {margin_g} g, the sheet's D and U give \
                     {expected} g"
                );
                assert!(margin_g > 0.0);
            }
            b.set(path::WEIGHT, past).expect("a weight");
            assert_ne!(
                b.finger_verdict(),
                rest_on(stop),
                "{past} g left the key there"
            );
        }
    }

    /// The initialization puts an unweighted key where its own weight takes
    /// it, friction off: the tests' key, front-heavy by 6.5 g, down on
    /// its front punching, and one weighed back-heavy on its cloth. The
    /// friction does not hold either in the air, because the initialization
    /// is solved without it.
    #[test]
    fn an_unweighted_key_starts_where_its_own_weight_puts_it() {
        let b = Instrument::weighed_section();
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            })
        );
        assert!(!b.reinitialized());
        let mut heavy = Instrument::weighed_section();
        heavy
            .act(Action::WeighKey {
                mass_g: 151.1,
                front_weight_g: -8.1,
                from_lip_mm: 1000.0 * 0.013,
            })
            .expect("a back-heavy key is weighed");
        assert_eq!(
            heavy.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::BackRailCloth
            })
        );
    }

    /// A key already sitting on its punching is **held there**, not reported
    /// as for ever descending.
    ///
    /// The friction is turned down for this, and that is the point rather
    /// than a convenience: an unleaded key of this length is very nearly
    /// balanced — about 6.5 g of front weight against a 6 g friction band — so
    /// with its own friction it falls by a fraction of a gram, and a little
    /// less front weight would leave it wherever it is put, which is also
    /// correct.
    /// The bug this guards is the other case: a key whose front weight beats
    /// its friction, sitting at the bottom of its dip, and the branches
    /// insisting it is still going down.
    #[test]
    fn a_key_at_the_bottom_of_its_dip_is_held_by_the_punching() {
        let mut b = Instrument::weighed_section();
        b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 2.0 })
            .expect("Stanwood's protocol reads 2 g on this key");
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            }),
            "a front-heavy key sits at the bottom of its dip, and the \
             punching carries it"
        );
    }

    /// Widening the friction widens the band, and leaves its center where
    /// the seats' tilt lets it: D is read at the top seat and U at the
    /// bottom, where the measuring point stands a little nearer the pin's
    /// vertical at the top, so the extra friction reads a hair heavier there.
    /// The bound is derived from the seats' arms, with the walks' resolution.
    #[test]
    fn more_friction_makes_a_wider_band_around_the_same_center() {
        let mut b = Instrument::weighed_section();
        let band = |b: &Instrument| {
            [
                ProcedureId::DownWeight,
                ProcedureId::UpWeight,
                ProcedureId::BalanceWeight,
            ]
            .map(|p| read::value(b, p))
        };
        let [d0, u0, bw0] = band(&b);
        read::bushings_holding_g(&mut b, 12.0);
        let [d1, u1, bw1] = band(&b);
        assert!(
            (d1 - u1) - (d0 - u0) > 2.0 * 6.0,
            "doubling the friction must widen the band by twice the extra, or more"
        );

        let (k, seats) = b.seated().unwrap();
        let point = b.measuring_point_on_key();
        let arm = |q: f64| point.x * q.cos() - point.y * q.sin();
        let angle = k.mounted.angle.0;
        let tilt = 0.5
            * 6.0
            * point.x
            * (1.0 / arm(seats.top.q[angle]) - 1.0 / arm(seats.bottom.q[angle]));
        assert!(
            (bw1 - bw0 - tilt).abs() <= 2.0 * read::resolution(),
            "the center moved by {} g, the seats' arms account for {tilt} g",
            bw1 - bw0
        );
        assert!(tilt.abs() < 0.05, "a few hundredths of a gram: {tilt}");
    }

    /// Consistency: **a key weighed whole is read as it was weighed.** The
    /// readout's mass is the one typed, held in kilograms and shown in grams
    /// again, two roundings within `ε` of it; the jig reads the front weight
    /// back where it was read, to round-off, sixteen rounded operations each
    /// within `ε/2` of it; and read 7 mm further back, the same static moment
    /// on the shorter arm. The front weight then rests on nothing estimated
    /// about the key's mass, while every weighing on the frame still names
    /// the height of its center of mass, which no scale gives. The weighing
    /// is kept as typed.
    #[test]
    fn a_key_weighed_whole_is_read_as_it_was_weighed() {
        let mut b = Instrument::weighed_section();
        b.act(Action::WeighKey {
            mass_g: 121.5,
            front_weight_g: 7.5,
            from_lip_mm: 1000.0 * 0.013,
        })
        .expect("a key's weighing");
        let mass = b.key_mass_g().expect("a weighed key");
        assert!((mass - 121.5).abs() <= f64::EPSILON * 121.5, "{mass}");
        let fw = read::measured(&b, ProcedureId::FrontWeight);
        assert!(
            (fw.value - 7.5).abs() <= 16.0 * f64::EPSILON * 7.5,
            "7.5 g weighed, the jig reads {} g",
            fw.value
        );
        assert_eq!(
            fw.trust,
            Trust::Indicative {
                because: vec![Input::Geometry]
            }
        );
        assert_eq!(
            read::measured(&b, ProcedureId::DownWeight).trust,
            Trust::Indicative {
                because: vec![
                    Input::CenterOfMassHeight,
                    Input::KeyFriction,
                    Input::Geometry
                ]
            }
        );
        let KeyMass::Weighed { datum, .. } = b.key_mass() else {
            panic!("{:?}", b.key_mass());
        };
        assert_eq!(datum.grams.to_bits(), 7.5_f64.to_bits());
        assert_eq!(datum.from_lip_m.to_bits(), 0.013_f64.to_bits());

        let arm_13 = b.measuring_point();
        b.set(path::MEASURING_POINT, 1000.0 * 0.020)
            .expect("a measuring point");
        let arm_20 = b.measuring_point();
        let further = read::value(&b, ProcedureId::FrontWeight);
        assert!(
            (further * arm_20 - 7.5 * arm_13).abs() <= 2.0 * 16.0 * f64::EPSILON * 7.5 * arm_13,
            "7 mm further back the jig reads {further} g, not the same moment"
        );
    }

    /// Consistency: **a key weighed again at what its sheet reads reads the
    /// same sheet.** Weighed at its own mass and the sheet's front weight in
    /// full, at the measuring point the sheet reads, its mass and its static
    /// moment are what they were to round-off and the height of its center
    /// of mass is the same: the front weight and the key weight ratio read
    /// the same to round-off; key height and dip the same seats, closed by
    /// the static solve within `GAP_TOLERANCE`, seen at the key front
    /// through the lever from the cloth, under 2.3 times it, and from the
    /// front rail, 258 over 225 mm; and every weight walked within the two
    /// walks' resolutions.
    #[test]
    fn a_key_weighed_again_at_what_its_sheet_reads_reads_the_same_sheet() {
        let estimated = Instrument::weighed_section();
        let mass_g = estimated.key_mass_g().expect("a weighed key");
        let front_g = read::value(&estimated, ProcedureId::FrontWeight);
        let mut weighed = Instrument::weighed_section();
        weighed
            .act(Action::WeighKey {
                mass_g,
                front_weight_g: front_g,
                from_lip_mm: 1000.0 * (estimated.measuring_point_from_lip()),
            })
            .expect("the sheet's own numbers");
        let gap = pa_core::solve::statics::GAP_TOLERANCE;
        for (p, bound) in [
            (
                ProcedureId::FrontWeight,
                16.0 * f64::EPSILON * front_g.abs(),
            ),
            (ProcedureId::KeyWeightRatio, 16.0 * f64::EPSILON),
            (ProcedureId::KeyHeight, 2.0 * gap * 2.3),
            (ProcedureId::KeyDip, 2.0 * gap * (2.3 + 258.0 / 225.0)),
            (ProcedureId::DownWeight, 2.0 * read::resolution()),
            (ProcedureId::UpWeight, 2.0 * read::resolution()),
            (ProcedureId::BalanceWeight, 2.0 * read::resolution()),
            (ProcedureId::FrictionWeight, 2.0 * read::resolution()),
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheJig,
                2.0 * read::resolution(),
            ),
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
                2.0 * read::resolution(),
            ),
        ] {
            let (was, is) = (read::value(&estimated, p), read::value(&weighed, p));
            assert!(
                (is - was).abs() <= bound,
                "{p:?}: {was} as weighed, {is} weighed again at its sheet's numbers"
            );
        }
    }

    /// Consistency, the re-weighing `pa-cli weigh` prints and the smoke test
    /// runs through the page: **a key weighed with its front weight a gram
    /// heavier, its mass the same, moves the balance weight and not the
    /// friction weight.** The front weight's gram is a static moment of a
    /// gram on the measuring point's level arm; at each seat the key's tilt
    /// turns it by its cosine, and the measuring point's arm is
    /// `x cos θ − y sin θ`, so each edge of the band moves by minus that
    /// gram over `1 − (y / x) tan θ`, the balance weight by the mean of the
    /// two and the friction weight by half their difference, a few
    /// thousandths of a gram: by hand, within the two walks' resolutions and
    /// what rounding leaves of the static moment, under 64 `ε` of the front
    /// weight.
    #[test]
    fn a_front_weight_one_gram_heavier_moves_balance_weight_not_friction() {
        let before = Instrument::weighed_section();
        let mass_g = before.key_mass_g().expect("a weighed key");
        let front_g = read::value(&before, ProcedureId::FrontWeight);
        let mut after = before.clone();
        after
            .act(Action::WeighKey {
                mass_g,
                front_weight_g: front_g + 1.0,
                from_lip_mm: 1000.0 * (before.measuring_point_from_lip()),
            })
            .expect("a gram heavier in front is a key's weighing");
        let (k, seats) = before.seated().expect("the bench seats");
        let point = before.measuring_point_on_key();
        let edge = |q: &[f64]| {
            let theta = q[k.mounted.angle.0];
            -1.0 / (1.0 - point.y / point.x * theta.tan())
        };
        let (on_top, on_bottom) = (edge(&seats.top.q), edge(&seats.bottom.q));
        let moved = |p| read::value(&after, p) - read::value(&before, p);
        let tolerance = 2.0 * read::resolution() + 64.0 * f64::EPSILON * front_g.abs();
        let (bw, f) = (
            moved(ProcedureId::BalanceWeight),
            moved(ProcedureId::FrictionWeight),
        );
        assert!(
            (bw - 0.5 * (on_top + on_bottom)).abs() <= tolerance,
            "BW moved by {bw} g, by hand {} g",
            0.5 * (on_top + on_bottom)
        );
        assert!(
            (f - 0.5 * (on_top - on_bottom)).abs() <= tolerance,
            "F moved by {f} g, the tilt between the seats accounts for {} g",
            0.5 * (on_top - on_bottom)
        );
        assert!(
            bw < -0.99,
            "a gram of front weight is about a gram of BW: {bw}"
        );
        assert!(f.abs() < 0.01, "and a few thousandths of F: {f}");
    }

    /// Consistency, and what the owner sees: **a weighing no key has is
    /// refused, with why, and nothing changes.** A mass of nothing, a front
    /// weight read off the covering, a front weight that would stand the
    /// center of mass past the lip or past the tail, and a value that is not
    /// a number: each time the bench says why, by the refusal's code and its
    /// numbers, and the sheet, the readout and the key's mass stay as they
    /// were.
    #[test]
    fn a_weighing_no_key_has_is_refused_and_nothing_changes() {
        let everything =
            |b: &Instrument| format!("{:?} {:?} {:?}", b.key_mass(), b.sheet(), b.readout());
        let mut b = Instrument::weighed_section();
        let before = everything(&b);
        // A weighing typed, grams, grams and meters, and whether a refusal
        // is the one it must be.
        type Case = ((f64, f64, f64), fn(&EditRefusal) -> bool);
        let cases: [Case; 5] = [
            ((0.0, 6.5, 0.013), |w| {
                *w == EditRefusal::Problem(Problem::MassNotPositive { mass: 0.0 })
            }),
            ((121.1, 6.5, 0.200), |w| {
                *w == EditRefusal::Weighing(NotAWeighing::RollerOffTheCovering {
                    from_lip_m: 0.200,
                })
            }),
            (
                (40.0, 50.0, 0.013),
                |w| matches!(w, EditRefusal::Weighing(NotAWeighing::CenterOfMassOffTheKey { at_m }) if *at_m > 0.0),
            ),
            (
                (121.1, -150.0, 0.013),
                |w| matches!(w, EditRefusal::Weighing(NotAWeighing::CenterOfMassOffTheKey { at_m }) if *at_m < 0.0),
            ),
            ((f64::NAN, 6.5, 0.013), |w| {
                *w == EditRefusal::Problem(Problem::NotFinite {
                    param: crate::registry::WEIGHED_MASS,
                })
            }),
        ];
        for ((mass_g, front_g, from_lip), says) in cases {
            let why = b
                .act(Action::WeighKey {
                    mass_g,
                    front_weight_g: front_g,
                    from_lip_mm: 1000.0 * from_lip,
                })
                .expect_err("no key's weighing");
            assert!(
                says(&why),
                "{mass_g} g, {front_g} g at {from_lip} m: {why:?}"
            );
            assert_eq!(everything(&b), before, "a refusal changed the bench");
        }
    }

    /// Consistency: **the weighing keeps the key's static moment, and an edit
    /// of the key reads it on its new arm.** The key front brought in 10 mm
    /// after it was weighed moves the measuring point with the lip, and the
    /// jig reads the same moment over the shorter arm, to round-off.
    #[test]
    fn a_weighed_key_keeps_its_static_moment_when_its_front_is_moved() {
        let mut b = Instrument::weighed_section();
        b.act(Action::WeighKey {
            mass_g: 121.5,
            front_weight_g: 7.5,
            from_lip_mm: 1000.0 * 0.013,
        })
        .expect("a key's weighing");
        let moment = 7.5 * b.measuring_point();
        let front_edge = b.key().front_edge;
        b.set(path::FRONT_EDGE, 1000.0 * (front_edge - 0.010))
            .expect("a key front");
        let shorter = read::value(&b, ProcedureId::FrontWeight) * b.measuring_point();
        assert!(
            (shorter - moment).abs() <= 2.0 * 16.0 * f64::EPSILON * moment,
            "{shorter} g·m read on the shorter arm, {moment} g·m weighed"
        );
    }

    /// Consistency: **a front weight read elsewhere than the bench's
    /// measuring point is moved to it by the lever.** Weighed with the jig's
    /// roller 20 mm back from the lip while the bench reads at its own 13 mm,
    /// the key's static moment is the typed grams on the arm they were read
    /// on, so the jig reads them times that arm over its own, to round-off,
    /// as when the measuring point is moved after the weighing; read on the
    /// bench's arm instead, the static moment would be taken about the
    /// wrong datum, and the jig would read the typed grams back.
    #[test]
    fn a_front_weight_read_elsewhere_is_moved_to_the_measuring_point_by_the_lever() {
        let mut b = Instrument::weighed_section();
        b.act(Action::WeighKey {
            mass_g: 121.5,
            front_weight_g: 7.5,
            from_lip_mm: 20.0,
        })
        .expect("a key's weighing");
        let read_on = b.key().lip() - 20.0 / 1000.0;
        let expected = 7.5 * read_on / b.measuring_point();
        let fw = read::value(&b, ProcedureId::FrontWeight);
        let bound = 2.0 * 16.0 * f64::EPSILON * 7.5;
        assert!(
            (fw - expected).abs() <= bound,
            "7.5 g read 20 mm back from the lip, the jig reads {fw} g at 13 mm, not {expected} g"
        );
        assert!(
            (expected - 7.5).abs() > 1000.0 * bound,
            "the two arms must tell the datums apart"
        );
    }

    /// A weight moves the key between its two seats and no further: a heavy
    /// one presses it down onto the bottom seat, whatever its size, because
    /// past a stop a weight only loads the stop, and a lift raises it onto
    /// the top seat, its balance punching still carrying it.
    ///
    /// The bound is derived. The touch weight and the seat's preparation
    /// loads put the key on the same rigid stop, and each static solve
    /// closes it within `GAP_TOLERANCE` (a picometer), which an arm of at
    /// least 0.2 m from the bearing to the stop turns into an angle.
    #[test]
    fn a_weight_takes_the_key_to_its_seats_and_no_further() {
        let mut b = Instrument::weighed_section();
        let (k, seats) = b.seated().expect("the bench seats");
        let angle = k.mounted.angle.0;
        let bound = 2.0 * 1e-12 / 0.2;
        for (weights, seat) in [
            (&[60.0, 400.0][..], &seats.bottom),
            (&[-30.0, -60.0][..], &seats.top),
        ] {
            for &w in weights {
                b.set(path::WEIGHT, w).expect("a weight");
                let q = &b.state.as_ref().expect("a state").q;
                assert!(
                    (q[angle] - seat.q[angle]).abs() <= bound,
                    "{w} g left the key at {} rad, its seat is at {} rad",
                    q[angle],
                    seat.q[angle]
                );
                assert_eq!(k.mounted.lift_at(q).to_bits(), 0.0_f64.to_bits(), "{w} g");
            }
        }
    }

    /// The audit's RUST-8, consistency: **a lift the key's own vertical
    /// equilibrium cannot hold lifts it off its frame, and the bench says
    /// so**, never certifying it held.
    ///
    /// On the key's cloth, a pull at the measuring point turns the key about
    /// the cloth's edge: its balance punching carries the key's weight less
    /// the pull less what the cloth takes, and the cloth takes the pull's
    /// moment about the pin over its arm, less what the key's own weight and
    /// its friction do. By hand, with the key at its top seat's attitude, its
    /// arms read off the bench and the friction against the motion that took
    /// it there: the punching is left carrying nothing at a pull of about
    /// 62 g. A gram short of it the key stays on its cloth and its punching,
    /// within a tenth of a micron of it. A few hundredths of a gram past it,
    /// friction can still hold the key turned up on its cloth, lifted off
    /// its punching within its pin
    /// (`a_pull_never_draws_the_key_above_its_balance_pin`); a few tenths
    /// past it and beyond, with nothing above the key to hold it, the key
    /// turns up off its punching past where it leaves its balance pin, off
    /// its frame, and the bench says the weight lifts it off: no
    /// equilibrium, the key drawn where the last one left it, never reported
    /// held.
    #[test]
    fn a_pull_past_what_the_punching_carries_lifts_the_key_off_its_frame() {
        let mut b = Instrument::weighed_section();
        let (k, seats) = b.seated().expect("the bench seats");
        let m = &k.mounted.mechanism;
        let mass = m.body(k.mounted.key).mass.expect("weighed");
        let pose = m.pose(&seats.top.q);
        let place = pose.placement(k.mounted.key);
        let cloth = b.bearing_at_the_back().expect("on its cloth").point.x;
        let com = place.world(mass.com);
        let at = place.world(b.measuring_point_on_key());
        let bound = b.friction.bound_n_m;
        // The pull `p` newtons at which the punching carries nothing: the
        // cloth then carries the key's weight less the pull, and the moments
        // about the cloth's edge balance with the friction against the turn.
        let w = mass.mass * G;
        let pull = (w * (com.x - cloth) + bound) / (at.x - cloth);
        let grams = 1000.0 * pull / G;
        let held_to = grams.floor() - 1.0;
        assert!(held_to > 55.0, "the punching carries the key to {grams} g");
        b.set(path::WEIGHT, -held_to).expect("a weight");
        let q = b.state.as_ref().expect("held").q.clone();
        assert!(
            k.mounted.lift_at(&q).abs() <= TOUCHING,
            "{held_to} g lifted the key"
        );
        assert!(
            matches!(
                b.finger_verdict(),
                Some(TouchVerdict::RestingOn {
                    stop: ContactCode::BackRailCloth
                })
            ),
            "{:?}",
            b.finger_verdict()
        );
        for past in [grams.ceil() + 2.0, 100.0, 400.0] {
            b.set(path::WEIGHT, -past).expect("a weight");
            assert_eq!(
                b.finger_verdict(),
                Some(TouchVerdict::LiftsOffItsFrame),
                "-{past} g"
            );
            assert!(!b.reinitialized(), "the key stays where it was");
            let r = b.readout().expect("the sheet still reads");
            assert_eq!(r.finger, Some(TouchVerdict::LiftsOffItsFrame));
            let still = &b.state.as_ref().expect("the last equilibrium").q;
            assert_eq!(
                still.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                q.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                "drawn where the last equilibrium left it"
            );
        }
        b.set(path::WEIGHT, 0.0).expect("a weight");
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            }),
            "and back down on its punching once the weight comes off"
        );
    }

    /// Consistency: **a pull never draws the key above its balance pin.**
    /// The key leaves its pin where its balance hole, the key frame's origin
    /// on the pin's axis, clears the drawn pin's top, 9 mm above its
    /// punching on the tests' bench. Pulled up at its measuring point past
    /// what its punching carries, the key turns up on its cloth off its
    /// punching as far as its friction lets it: under a millimeter at
    /// 62.7 g, 8.5 mm at 62.85 g. A little more and it would turn up past
    /// the pin's top, 13 mm at 63 g, where it has left its pin: the bench
    /// says the weight lifts it off its frame, and draws it where the last
    /// equilibrium left it. At every pull from 62.5 to 63.5 g, by hundredths
    /// of a gram, each from the key at rest, the key's balance hole is drawn
    /// at or below the pin's drawn top, read off the part the view draws,
    /// and a key said to be lifted off its punching is drawn that far above
    /// it.
    #[test]
    fn a_pull_never_draws_the_key_above_its_balance_pin() {
        let base = Instrument::weighed_section();
        let pin_top = base
            .outlines()
            .iter()
            .find(|o| o.code == PartCode::BalanceRailPin)
            .expect("the bench draws a balance pin")
            .points
            .iter()
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);
        let bearing = base.frame().balance_bearing_height();
        let (mut lifted, mut off) = (0, 0);
        for k in 0..=100 {
            let grams = 62.5 + 0.01 * f64::from(k);
            let mut b = base.clone();
            b.set(path::WEIGHT, -grams).expect("a weight");
            let hole = b.key_placement().origin.y;
            assert!(
                hole <= pin_top,
                "{grams} g draws the key's balance hole {} mm above its pin's top",
                (hole - pin_top) * 1000.0
            );
            match b.finger_verdict() {
                Some(TouchVerdict::LiftsOffBalancePunching { lift_mm, .. }) => {
                    assert!(
                        ((hole - bearing) * 1000.0 - lift_mm).abs() < 1e-9,
                        "{grams} g: drawn {} mm up, said {lift_mm} mm",
                        (hole - bearing) * 1000.0
                    );
                    lifted += 1;
                }
                Some(TouchVerdict::LiftsOffItsFrame) => off += 1,
                _ => {}
            }
        }
        assert!(
            lifted > 0 && off > 0,
            "{lifted} pulls held the key lifted within its pin, {off} lifted it off"
        );
        let mut b = base.clone();
        b.set(path::WEIGHT, -63.0).expect("a weight");
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
    }

    /// Consistency: **a key its friction holds off its punching stays
    /// there.** Pulled 62.7 g at its measuring point, a few hundredths of a
    /// gram past what its punching carries, the key turns up on its cloth
    /// and leaves its punching by under a millimeter, where its friction
    /// holds it. Pulled 60.7 g, inside the band its friction holds it in,
    /// it stays where it was to the bit, still lifted, on its cloth: the
    /// increment starts from where the last one left the key's slide, never
    /// from the key put back on its punching, which would carry it down.
    #[test]
    fn a_key_its_friction_holds_off_its_punching_stays_there() {
        let mut b = Instrument::weighed_section();
        b.set(path::WEIGHT, -62.7).expect("a weight");
        let lifted = b.finger_verdict();
        let Some(TouchVerdict::LiftsOffBalancePunching {
            touching,
            needs_friction,
            lift_mm,
        }) = lifted.clone()
        else {
            panic!("62.7 g: {lifted:?}");
        };
        assert_eq!(touching, vec![ContactCode::BackRailCloth]);
        assert!(needs_friction, "the cloth alone holds no pull");
        assert!(lift_mm > 0.1 && lift_mm < 1.0, "lifted {lift_mm} mm");
        let held = b.key_placement();
        b.set(path::WEIGHT, -60.7).expect("a weight");
        assert!(!b.reinitialized());
        assert_eq!(b.finger_verdict(), lifted);
        let still = b.key_placement();
        for (x, y) in [
            (held.origin.x, still.origin.x),
            (held.origin.y, still.origin.y),
            (held.cos, still.cos),
            (held.sin, still.sin),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "60.7 g moved the key");
        }
    }

    /// What is drawn moves when the key moves, and the frame does not: the
    /// key's pose turns it, its outlines stay as they are in its own frame,
    /// and the ground's pose is the world's.
    #[test]
    fn pressing_the_key_moves_only_the_key() {
        let mut b = Instrument::weighed_section();
        b.set(path::WEIGHT, -60.0).expect("a weight");
        let (at_rest, rest_poses) = (b.outlines(), b.poses());
        let rest_pt = b.key_placement().world(b.measuring_point_on_key());
        b.set(path::WEIGHT, 60.0).expect("a weight");
        let (pressed, pressed_poses) = (b.outlines(), b.poses());
        let pressed_pt = b.key_placement().world(b.measuring_point_on_key());

        // Probe the *named* measuring point. Probing the outline's extreme
        // vertex does not work: the rotation mixes x and y, so which of the two
        // front-edge corners has the greater x changes with the sign of the
        // angle, and the comparison silently switches corners.
        assert!(
            pressed_pt.y < rest_pt.y,
            "the measuring point must descend: {} -> {}",
            rest_pt.y,
            pressed_pt.y
        );
        // And the tail must rise by the same rotation, which is the half of it
        // a panel-shaped model would forget. Point 0 of the keystick is the
        // tail's corner on the underside, and the outline's order is fixed:
        // following one material point by its index is the only way to be sure
        // it is the same one.
        let tail = |o: &[Outline], poses: &[Placement]| {
            let stick = o.iter().find(|x| x.code == PartCode::Keystick).unwrap();
            poses[stick.body.0].world(stick.points[0]).y
        };
        assert!(
            tail(&pressed, &pressed_poses) > tail(&at_rest, &rest_poses),
            "the key's tail must rise"
        );
        assert_eq!(
            at_rest, pressed,
            "every outline stays where it is in its own frame"
        );
        assert_eq!(rest_poses[0], Placement::WORLD, "the frame does not move");
        assert_eq!(pressed_poses[0], Placement::WORLD);
    }

    /// The scene's poses place the key where its statics put it: lifted
    /// onto its cloth, its frame's origin stands on the balance bearing, and
    /// it stands nose-up.
    #[test]
    fn the_key_stands_on_its_balance_bearing_at_rest() {
        let mut b = Instrument::weighed_section();
        b.set(path::WEIGHT, -60.0).expect("a weight");
        let pose = b.key_placement();
        let bearing = b.frame().balance_bearing_height();
        assert!(pose.origin.x.abs() < 1e-15 && (pose.origin.y - bearing).abs() < 1e-15);
        assert!(
            (pose.cos * pose.cos + pose.sin * pose.sin - 1.0).abs() < 4.0 * f64::EPSILON,
            "a pose turns, it does not stretch"
        );
        assert!(pose.sin > 0.0, "at rest the key stands nose-up");
    }

    /// The measuring point the view marks, taken back from the drawn lip, is
    /// the one the jig weighs at, to the bit.
    #[test]
    fn the_marked_measuring_point_is_the_one_the_jig_weighs_at() {
        let mut b = Instrument::weighed_section();
        for from_lip in [0.013, 0.0127, 0.030] {
            b.set(path::MEASURING_POINT, 1000.0 * from_lip)
                .expect("a measuring point");
            let protocol = *b.protocol();
            let weighed = protocol.measuring_point_on_key(b.key());
            let marked = b.measuring_point_on_key();
            assert_eq!(marked.x.to_bits(), weighed.x.to_bits());
            assert_eq!(marked.y.to_bits(), weighed.y.to_bits());
        }
    }

    /// Adding paper under the balance punching raises the key, as on the bench.
    #[test]
    fn key_leveling_works_through_the_bench() {
        let mut b = Instrument::weighed_section();
        let before = read::mm(&b, ProcedureId::KeyHeight);
        let paper = b.frame().balance_punching.paper();
        read::lay(&mut b, Stack::Balance, paper + 0.0005);
        let after = read::mm(&b, ProcedureId::KeyHeight);
        assert!(
            after > before,
            "0.5 mm of paper must raise the key: {before} -> {after}"
        );
    }
}

#[cfg(test)]
mod contact_readout_tests {
    use super::*;
    use crate::registry::path;

    /// The dip the bench reports is the travel between the two **gaps
    /// closing**, not a number carried over from the target.
    ///
    /// Shimming the front punching moves the stop; the gap therefore closes
    /// earlier; the dip reported must shrink by what was shimmed, referred
    /// through the lever. If the readout were echoing the regulation target it
    /// would not move at all, and if it were reading a stored coordinate it
    /// would move by the wrong amount.
    #[test]
    fn the_dip_follows_the_contact_and_not_the_target() {
        let mut b = Instrument::weighed_section();
        let before = b.readout().unwrap();
        let dip_before = read::mm(&b, ProcedureId::KeyDip);
        let paper = b.frame().front_punching.paper();

        let shim = 0.0005;
        read::lay(&mut b, Stack::Front, paper + shim);
        let after = b.readout().unwrap();
        let dip_after = read::mm(&b, ProcedureId::KeyDip);

        // The front rail pin is 225 mm from the balance pin and the key front
        // 258 mm, so a shim at the punching shows up at the key front, where
        // the dip is read, multiplied by 258/225.
        let expected = shim * 1000.0 * (258.0 / 225.0);
        let moved = dip_before - dip_after;
        assert!(
            (moved - expected).abs() < 0.02,
            "a {shim} m shim moved the dip by {moved} mm, expected about {expected}"
        );
        assert!(
            (after.key_dip_target_mm - before.key_dip_target_mm).abs() < 1e-12,
            "the target must not have moved: only the achieved value has"
        );
    }

    /// The **default** frame is set so that the key is level at the bottom of
    /// its travel.
    ///
    /// That is a **design goal, not a law**, and the distinction is the point
    /// of this comment. Rail heights, punchings and cloth are what a maker
    /// uses to reach it, and plenty of keyboards do not: a key that is out of
    /// level when it is down is a real instrument in a real state, and the
    /// bench has to be able to show one. So nothing in the engine enforces
    /// this — it is asserted of *this default frame*, which stands for a
    /// well-set one.
    ///
    /// It is a statement about the **back rail**, not the front. The rest
    /// attitude is set by the balance bearing and the back rail cloth
    /// together; the front punching only catches the key where that attitude
    /// brings it. A key level when fully down is therefore nose-up when
    /// released, by exactly the dip — which is what a keyboard at rest
    /// actually looks like. With a straight underside the condition reads off
    /// the finished frame: the **front punching comes up to the same height as
    /// the balance punching**.
    ///
    /// The tolerance is derived, not chosen. Both stacks are placed
    /// continuously by the solve and then rebuilt from real papers, so each
    /// bearing sits up to one paper below where the solve put it, and the two
    /// errors are independent. Two papers is the bound.
    #[test]
    fn the_default_frame_puts_the_key_level_at_the_bottom_of_its_travel() {
        let b = Instrument::weighed_section();
        let f = &b.frame;
        let out = (f.front_bearing_height() - f.balance_bearing_height()) * 1000.0;
        let two_papers = 2.0 * pa_core::test_fixture::PAPER_THICKNESSES[0] * 1000.0;
        assert!(
            out.abs() < two_papers,
            "the key is out of level by {out:.3} mm at the bottom of its travel: \
             the front punching tops out at {:.3} mm and the balance at {:.3}",
            f.front_bearing_height() * 1000.0,
            f.balance_bearing_height() * 1000.0
        );

        // And released, it is nose-up rather than nose-down: the tail is on
        // the cloth and the front hangs clear of its punching by the dip.
        let r = b.readout().expect("the bench reads");
        assert!(
            f.back_bearing_height() < f.balance_bearing_height(),
            "the back rail must stand below the balance bearing, or the key \
             rests nose-down"
        );
        assert!(
            (read::mm(&b, ProcedureId::KeyDip) - r.key_dip_target_mm).abs() < 2.0 * two_papers,
            "and the travel that attitude leaves is the dip"
        );
    }

    /// Consistency: **a front pin a few tenths of a millimeter into the wood
    /// beside its slot draws the key on the pin, never through it.** Its
    /// front end 0.3 mm into the wood in front of the slot, its top well up
    /// in the slot: the bench puts the key on such a frame as the seats do,
    /// lowered from above where its nominal pose finds no state, and the key
    /// rides on the pin and its cloth, nothing drawn sunk, the frame seated.
    #[test]
    fn a_pin_a_few_tenths_into_the_wood_draws_the_key_on_it() {
        let mut b = Instrument::weighed_section();
        let pin = b.frame().sections.front.pin.expect("a pin").length();
        let (_, front_end) = b.key().front_mortise_span();
        b.set(
            path::FRONT_RAIL_AT,
            1000.0 * (front_end - 0.5 * pin + 0.0003),
        )
        .expect("a front rail");
        assert_eq!(b.frame_fault(), None);
        assert!(
            matches!(
                b.finger_verdict(),
                Some(TouchVerdict::RidesOnBothStops {
                    front: ContactCode::FrontPinOnTheWood,
                    ..
                })
            ),
            "{:?}",
            b.finger_verdict()
        );
        let m = b.key_on_frame().expect("mounted").mechanism;
        let q = &b.state.as_ref().expect("a state").q;
        assert!(m.overlap(&m.pose(q)) <= SEAT_OVERLAP, "nothing drawn sunk");
    }

    /// Consistency: **a back rail raised through the key's tail in one move
    /// draws the key riding as it is seated.** From where the bench builds
    /// it to 55 mm in one move, as a click on its slider's track moves it,
    /// the rail and its cloth stand in the tail of the key where it rested,
    /// the cloth clear above it: no contact reads a rail, and only the key
    /// read against its rails is found sunk. The cloth then holds the tail
    /// so high that the key would ride 15 mm above its balance punching,
    /// past the top of the bench's pin, 9 mm up: on that pin the key is off
    /// its frame, and the bench says so. On a balance pin 20 mm taller, put
    /// back by the loading rule, the key is drawn on its front punching and
    /// its cloth, as high above its balance punching as its seats ride,
    /// never on its punching with the rail drawn through it.
    #[test]
    fn a_back_rail_raised_through_the_key_in_one_move_draws_it_riding() {
        let mut b = Instrument::weighed_section();
        b.set(path::BACK_RAIL_HEIGHT, 1000.0 * 0.055)
            .expect("a back rail");
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
        assert!(b.riding().is_none() && b.frame_fault().is_some());

        let mut b = Instrument::weighed_section();
        b.frame.sections.balance.pin_height += 0.020;
        b.relax();
        assert!(!b.reinitialized(), "a taller pin moves nothing");
        b.set(path::BACK_RAIL_HEIGHT, 1000.0 * 0.055)
            .expect("a back rail");
        assert!(b.reinitialized(), "put back by the loading rule");
        let level = b.frame().balance_bearing_height();
        let riding = b.riding().expect("the key rides on its stops");
        let drawn_lift = b.key_placement().origin.y - level;
        assert!(
            (drawn_lift - riding.lift).abs() < 1e-9,
            "drawn {drawn_lift} m above its punching, seated {} m",
            riding.lift
        );
        let closed: Vec<ContactCode> = b
            .contacts()
            .iter()
            .filter(|c| c.state() == ContactState::Closed)
            .map(|c| c.code)
            .collect();
        assert_eq!(
            closed,
            vec![ContactCode::FrontRailPunching, ContactCode::BackRailCloth]
        );
    }

    /// Consistency: **a key riding just under its balance pin's top is drawn
    /// riding there, not off its frame.** The cloth 5 mm above the balance
    /// bearing, the key rides 2.27 mm up on its front punching and its
    /// cloth; the key 17 mm short and its cloth 3 mm lower, it rides 4.45 mm
    /// up on its front pin standing in its wood and its cloth. With the
    /// balance pin cut to 0.3 mm above where the key rides, the bench finds
    /// no frame fault and seats it riding where it rode, and the loading
    /// rule, run again, draws it there: no attitude a whole degree apart
    /// clears the frame below the pin's top, each whole degree either side
    /// of where the key rides costing more lift than the pin leaves, and the
    /// attitude is narrowed between them.
    #[test]
    fn a_key_riding_just_under_its_pins_top_is_drawn_riding_there() {
        let wedged = |b: &mut Instrument| {
            let level = b.frame().balance_bearing_height();
            let cloth = b.frame().back_rail_cloth;
            b.set(path::BACK_RAIL_HEIGHT, 1000.0 * (level + 0.005 - cloth))
                .expect("a back rail");
        };
        let short = |b: &mut Instrument| {
            let front_edge = b.key().front_edge;
            b.set(path::FRONT_EDGE, 1000.0 * (front_edge - 0.017))
                .expect("a key front");
            let h = b.frame().back_rail_height;
            b.set(path::BACK_RAIL_HEIGHT, 1000.0 * (h - 0.003))
                .expect("a back rail");
        };
        for edit in [&wedged as &dyn Fn(&mut Instrument), &short] {
            let mut b = Instrument::weighed_section();
            edit(&mut b);
            let ride = b.riding().expect("the key rides on its stops").lift;
            let f = &mut b.frame;
            let room = f.sections.balance.pin_height - f.balance_punching.height();
            f.sections.balance.pin_height += ride + 0.0003 - room;
            b.relax();
            assert_eq!(b.frame_fault(), None);
            let riding = b.riding().expect("the key rides on its pin");
            assert!(
                (riding.lift - ride).abs() < 1e-9,
                "rides {} m up, {ride} m on the bench's pin",
                riding.lift
            );
            b.initialize();
            assert!(
                matches!(
                    b.finger_verdict(),
                    Some(TouchVerdict::RidesOnBothStops { .. })
                ),
                "{:?}",
                b.finger_verdict()
            );
            let drawn = b.key_placement().origin.y - b.frame().balance_bearing_height();
            assert!(
                (drawn - ride).abs() < 1e-9,
                "drawn {drawn} m above its punching, rides {ride} m"
            );
        }
    }

    /// Consistency: **a key off its balance punching is said to touch what
    /// the view marks it touching.** The front rail raised to 41.7 mm stands
    /// its punching so high that the key, front-heavy, tips forward onto it
    /// and leaves its balance punching: the front punching carries it, the
    /// front pin's top corner meets the wall of the key's slot, and its tail
    /// stands clear of its cloth. The touch verdict names those two stops,
    /// the ones the view marks closed besides the balance punching, which
    /// stands open below the key, and not the cloth.
    #[test]
    fn a_key_off_its_punching_is_said_to_touch_what_the_view_marks() {
        let mut b = Instrument::weighed_section();
        b.set(path::FRONT_RAIL_HEIGHT, 1000.0 * 0.0417)
            .expect("a front rail");
        let verdict = b.finger_verdict();
        let Some(TouchVerdict::LiftsOffBalancePunching { touching, .. }) = verdict else {
            panic!("{verdict:?}");
        };
        let mut closed: Vec<ContactCode> = b
            .contacts()
            .iter()
            .filter(|c| c.state() == ContactState::Closed)
            .map(|c| c.code)
            .collect();
        closed.dedup();
        assert_eq!(touching, closed);
        assert_eq!(
            touching,
            vec![
                ContactCode::FrontRailPunching,
                ContactCode::FrontPinOnTheWood
            ]
        );
    }

    /// Consistency: **a weight that lifts the key off its frame leaves it
    /// drawn where it stood, whichever solve finds it lifting off.** On
    /// bushings of 3 g, 62 g pulling at the measuring point of the key at
    /// rest is past what its punching and its cloth hold: the increment from
    /// where the key stands stops at its cap before the key's slide meets
    /// the top of its range, and the initialization, from the key's nominal
    /// pose, finds the key lifting off its frame. The bench says the weight
    /// lifts the key off, and the key stays drawn where it stood, bit for
    /// bit, not put back by the loading rule and drawn level on its
    /// punching.
    #[test]
    fn a_pull_that_lifts_the_key_off_its_frame_leaves_it_where_it_stood() {
        let mut b = Instrument::weighed_section();
        b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 3.0 })
            .expect("bushings read 3 g");
        let stood = b.key_placement();
        b.set(path::WEIGHT, -62.0).expect("a weight");
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
        assert!(!b.reinitialized(), "the key stays where it was");
        let drawn = b.key_placement();
        for (x, y) in [
            (drawn.origin.x, stood.origin.x),
            (drawn.origin.y, stood.origin.y),
            (drawn.cos, stood.cos),
            (drawn.sin, stood.sin),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "drawn where it stood");
        }
    }

    /// Consistency: **a riding key has no stroke, and its dip reads
    /// nothing.** The key front 2.3725 mm forward of where the bench builds
    /// it stands the front rail pin 0.87 mm into the wood behind the key's
    /// front mortise, and the key rides on the pin and its cloth. Its top
    /// seat is solved free to lift from where it stands level on its
    /// punching, and converges only with the two rows that carry it closed;
    /// its bottom seat, solved from the top, is the same pose, so the dip
    /// reads nothing to the bit, where rows read closed while microns open
    /// would leave 23 nm between the seats. No weight that needs a stroke is
    /// walked between them: D, U and both key friction weights are refused
    /// for want of a stroke.
    #[test]
    fn a_riding_key_has_no_stroke_however_its_seats_are_rounded() {
        let mut b = Instrument::weighed_section();
        let front_edge = b.key().front_edge;
        b.set(path::FRONT_EDGE, 1000.0 * (front_edge + 0.0023725))
            .expect("a key front");
        let riding = b.riding().expect("the key rides on the pin and its cloth");
        assert_eq!(riding.front, ContactCode::FrontPinOnTheWood);
        assert_eq!(
            read::value(&b, ProcedureId::KeyDip).to_bits(),
            0.0_f64.to_bits(),
            "a riding key has no dip"
        );
        let sheet = b.sheet();
        for p in [
            ProcedureId::DownWeight,
            ProcedureId::UpWeight,
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
        ] {
            assert_eq!(
                sheet.get(p),
                Some(&Err(Refused::Walk(WalkRefusal::NoTravel))),
                "{p:?}"
            );
        }
    }

    /// What the owner sees on a frame whose stops fight, the audit's RUST-2:
    /// **the key drawn riding on its two stops, off its balance punching, on
    /// its parts as drawn, and the reason its touchweights are refused.** Its
    /// cloth holding the tail 5 mm above the balance bearing, the key at rest
    /// rides on the cloth and the front punching, 2.27 mm above its balance
    /// punching: the view marks those two closed, and the punching, more than
    /// the millimeter the view marks below the key, not at all; the key is
    /// drawn that far above it, where a straight underside laid on the two
    /// stops would stand 0.81 mm clear of the cloth, under the chamfer. Read
    /// off the drawn outlines, the drawn underside,
    /// chamfer included, stands on the drawn cloth's face or edge within the
    /// sagitta of a drawn arc, and nowhere in it. The jig's readings and the
    /// key height are read, the dip is nothing, and every weight that needs a
    /// stroke is refused for want of one, the seats saying how the key rides:
    /// on which stops, and how far above its punching. No frame fault: the
    /// key is seated.
    #[test]
    fn a_wedged_frame_is_drawn_riding_on_its_stops_and_says_why_it_cannot_be_weighed() {
        let mut b = Instrument::weighed_section();
        let level = b.frame().balance_bearing_height();
        let cloth = b.frame().back_rail_cloth;
        b.set(path::BACK_RAIL_HEIGHT, 1000.0 * (level + 0.005 - cloth))
            .expect("a back rail");
        let riding = b.riding().expect("the key rides on its stops");
        assert_eq!(riding.front, ContactCode::FrontRailPunching);
        assert!(
            (riding.lift - 0.00227).abs() < 0.00001,
            "lifted {} m",
            riding.lift
        );
        assert!(b.frame_fault().is_none());
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RidesOnBothStops {
                front: ContactCode::FrontRailPunching,
                lift_mm: 1000.0 * (b.key_placement().origin.y - level),
            })
        );
        let shown = b.contacts();
        let state_of = |code| {
            shown
                .iter()
                .find(|c| c.code == code)
                .map(ShownContact::state)
        };
        assert_eq!(
            state_of(ContactCode::BackRailCloth),
            Some(ContactState::Closed)
        );
        assert_eq!(
            state_of(ContactCode::FrontRailPunching),
            Some(ContactState::Closed)
        );
        assert_eq!(state_of(ContactCode::BalanceRailPunching), None);
        let drawn_lift = b.key_placement().origin.y - level;
        assert!(
            (drawn_lift - riding.lift).abs() < 1e-9,
            "drawn {drawn_lift} m above its punching, seated {} m",
            riding.lift
        );

        // The drawn underside over the drawn cloth's face, where the cloth
        // stands at its full height: nowhere in it, and its lowest there
        // within a micron of it, a thousandth of the 0.81 mm a straight
        // underside left. The key bears on the cloth's front edge, just past
        // the face, nose-down by its ride: over the face's end it stands the
        // edge's radius times the turn's square above it, tens of nanometers.
        let drawn = b.outlines();
        let poses = b.poses();
        let cloth = drawn
            .iter()
            .find(|o| o.code == PartCode::BackRailCloth)
            .expect("the cloth is drawn");
        let top = cloth.points.iter().fold(f64::MIN, |a, p| a.max(p.y));
        let (x0, x1) = cloth
            .points
            .iter()
            .filter(|p| p.y > top - 1e-12)
            .fold((f64::MAX, f64::MIN), |(lo, hi), p| {
                (lo.min(p.x), hi.max(p.x))
            });
        let (parts, ranges, _) = b.parts();
        let mut lowest = f64::MAX;
        for range in &ranges.underside {
            for seg in range.segs(&parts) {
                let (a, c) = (
                    poses[KEY.0].world(seg.start()),
                    poses[KEY.0].world(seg.end()),
                );
                for step in 0..=4000 {
                    let t = f64::from(step) / 4000.0;
                    let x = a.x + t * (c.x - a.x);
                    if x >= x0 && x <= x1 {
                        lowest = lowest.min(a.y + t * (c.y - a.y) - top);
                    }
                }
            }
        }
        assert!(
            lowest > -1e-9 && lowest < 1e-6,
            "the drawn key stands {lowest} m off the cloth's drawn face"
        );

        let sheet = b.sheet();
        for p in [
            ProcedureId::FrontWeight,
            ProcedureId::KeyWeightRatio,
            ProcedureId::KeyHeight,
        ] {
            assert!(sheet.get(p).is_some_and(Result::is_ok), "{p:?}");
        }
        assert_eq!(
            read::value(&b, ProcedureId::KeyDip).to_bits(),
            0.0_f64.to_bits()
        );
        for p in [
            ProcedureId::DownWeight,
            ProcedureId::UpWeight,
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
        ] {
            let why = sheet.get(p).unwrap().as_ref().expect_err("no stroke");
            assert_eq!(why, &Refused::Walk(WalkRefusal::NoTravel), "{p:?}");
        }
        let report = b.seat_report().expect("the key is seated, riding");
        assert_eq!(report.riding, Some(riding), "the seats say how it rides");
        assert!(b.readout().is_some(), "and the readout reads");
    }

    /// The key is drawn resting on its cloth, not inside it.
    ///
    /// Wherever the rail is put — in front of the chamfer, under it, or hung
    /// off the end of the key — the wood must stop at the felt. The three
    /// cases are the three ways the two faces can meet, and the test is run on
    /// the **drawn** outlines because that is the geometry a reader judges.
    #[test]
    fn the_drawn_key_rests_on_the_drawn_cloth_and_not_in_it() {
        // Read **what is drawn**, not what the contact says, over the whole
        // width of the cloth. Faults of this kind — a pin out through the top
        // of a key, a key down inside a rail — are invisible to a test that
        // asks the contact code whether it is happy.
        for back_rail in [-0.150_f64, -0.200, -0.232] {
            let key = pa_core::test_fixture::key();
            let frame = FrameDesign {
                back_rail,
                ..pa_core::test_fixture::frame()
            };
            let target = Targets {
                key_height: 0.0650,
                key_dip: 0.0102,
            };
            let Ok(b) = read::built(key, frame, target) else {
                continue;
            };
            let drawn = b.outlines();
            let poses = b.poses();
            let cloth = drawn
                .iter()
                .find(|o| o.code == PartCode::BackRailCloth)
                .expect("the cloth is drawn");
            let top = cloth.points.iter().fold(f64::MIN, |a, p| a.max(p.y));
            // The cloth's face, between its rounded edges: where it is flat at
            // its full height.
            let (x0, x1) = cloth
                .points
                .iter()
                .filter(|p| p.y > top - 1e-12)
                .fold((f64::MAX, f64::MIN), |(lo, hi), p| {
                    (lo.min(p.x), hi.max(p.x))
                });

            // The underside, as drawn: the keystick's chain from its tail,
            // then the key front's bottom, carried round by the key's pose
            // into the attitude the bench put it in.
            let (parts, ranges, _) = b.parts();
            let mut underside = Vec::new();
            for range in &ranges.underside {
                for seg in range.segs(&parts) {
                    underside.push((seg.start(), seg.end()));
                }
            }
            let pose = poses[KEY.0];
            for (a, b) in underside {
                let (a, b) = (pose.world(a), pose.world(b));
                for step in 0..=20 {
                    let t = f64::from(step) / 20.0;
                    let x = a.x + t * (b.x - a.x);
                    if x < x0 || x > x1 {
                        continue;
                    }
                    let y = a.y + t * (b.y - a.y);
                    assert!(
                        y > top - 1e-9,
                        "with the rail at {back_rail} m the key is drawn {} mm inside its \
                         cloth at {x} m",
                        (top - y) * 1000.0
                    );
                }
            }
        }
    }

    /// The gaps the bench reports agree with what holds the key.
    ///
    /// They are read off the contact rows where the key is drawn, and the
    /// statics put it there: when it rests on a stop, or friction holds it
    /// touching one, that stop's gap reads zero, not "small", and the other
    /// stands open. If it did not, the drawn position and the physics would
    /// be two different states. A static solve closes a gap within a
    /// picometer, `1e-9` mm.
    #[test]
    fn the_reported_gaps_agree_with_where_the_key_is_held() {
        let mut b = Instrument::weighed_section();
        let mut held = Vec::new();
        for weight in [-60.0, 0.0, -6.5, 60.0, -6.5] {
            b.set(path::WEIGHT, weight).expect("a weight");
            let r = b.readout().unwrap();
            let front = r.gap_front_mm.expect("the punching stands under the flat");
            let back = r.gap_back_mm.expect("the cloth meets the underside");
            let on = match r.finger {
                Some(TouchVerdict::RestingOn { stop }) => Some(stop),
                Some(TouchVerdict::HeldByFriction { touching, .. }) => touching,
                other => panic!("{weight} g: {other:?}"),
            };
            match on {
                Some(ContactCode::FrontRailPunching) => {
                    assert!(
                        front.abs() < 1e-9,
                        "on the front punching, yet its gap reads {front} mm"
                    );
                    assert!(back > 0.0, "and the back must be open");
                }
                Some(ContactCode::BackRailCloth) => {
                    assert!(back.abs() < 1e-9, "on the cloth, its gap {back} mm");
                    assert!(front > 0.0);
                }
                other => panic!("{weight} g: the key is on {other:?}"),
            }
            held.push(r.finger);
        }
        assert!(held
            .iter()
            .any(|v| matches!(v, Some(TouchVerdict::RestingOn { .. }))));
        assert!(held
            .iter()
            .any(|v| matches!(v, Some(TouchVerdict::HeldByFriction { .. }))));
    }

    /// Consistency, and the check of `.claude/rules/physics.md` > One part definition feeds contacts and drawing: **every contact
    /// surface is a range of a drawn part.**
    ///
    /// By type, a contact pair holds nothing but ranges of parts. Here, on
    /// every bench of the golden sheet and on every shipped profile's: the
    /// mechanism the contacts are read on carries exactly the parts the view
    /// draws, and every range of every pair names one of them and segments
    /// its drawn silhouette has.
    #[test]
    fn every_contact_surface_is_a_range_of_a_drawn_part() {
        let fixtures = crate::golden::FIXTURES
            .iter()
            .map(|fixture| (fixture.name, (fixture.build)()));
        let profiles = Profile::names().map(|name| {
            let profile = Profile::named(name).expect("shipped");
            (name, Instrument::of(profile).expect("built"))
        });
        for (name, b) in fixtures.chain(profiles) {
            let (drawn, _, _) = b.parts();
            let mounted = b.key_on_frame().expect("the key mounts on its frame");
            assert_eq!(
                mounted.mechanism.parts(),
                &drawn[..],
                "{name}: the contacts read the parts the view draws"
            );
            assert_eq!(mounted.mechanism.pairs().len(), 5, "{name}");
            for pair in mounted.mechanism.pairs() {
                for range in pair.a.ranges.iter().chain(&pair.b.ranges) {
                    let part = &drawn[range.part.index()];
                    assert!(
                        range.segs.start < range.segs.end
                            && usize::from(range.segs.end) <= part.silhouette().len(),
                        "{name}: {:?} reads segments {:?} of {:?}",
                        pair.code,
                        range.segs,
                        part.code
                    );
                }
            }
        }
    }

    /// The contacts the view marks are where the key touches: resting on its
    /// cloth, the cloth's back edge closed on the chamfer, and nothing else
    /// of the cloth within a millimeter (the cloth's front edge is no contact
    /// of its own, its nearest point to the chamfer being where it meets the
    /// face, and the face falling away from it toward the back edge); down
    /// on its punching, the punching closed at its center; and the balance
    /// punching closed at its center all along, the key bearing on it. A
    /// closed contact's two points coincide.
    ///
    /// The one other row within a millimeter at rest is the front pin's,
    /// open, beside the front wall of its mortise: the slot moves back along
    /// the pin by the height of the pin's top corner above the bearing times
    /// the key's turn, 0.04 rad at rest, which brings the wall to within
    /// about 0.8 mm of the corner, from the 1.5 mm it stands off with the key
    /// level.
    #[test]
    fn the_contacts_the_view_marks_are_where_the_key_touches() {
        let mut b = Instrument::weighed_section();
        let closed = |b: &Instrument| -> Vec<ContactCode> {
            b.contacts()
                .iter()
                .filter(|c| c.state() == ContactState::Closed)
                .map(|c| c.code)
                .collect()
        };
        b.set(path::WEIGHT, -60.0).expect("a weight");
        assert_eq!(
            closed(&b),
            vec![ContactCode::BackRailCloth, ContactCode::BalanceRailPunching]
        );
        let at_rest = b.contacts();
        let section = b.frame().sections.back;
        let back_edge = section.center_x(b.frame().back_rail) - section.half_length;
        for c in &at_rest {
            if c.code == ContactCode::FrontPinOnTheWood {
                assert!(
                    c.gap > 0.0 && c.gap < 0.0015 && c.gap <= ROW_MARGIN,
                    "the pin stands {} m off its mortise's wall",
                    c.gap
                );
                continue;
            }
            if c.code == ContactCode::BalanceRailPunching {
                assert_eq!(c.state(), ContactState::Closed, "{c:?}");
                assert!(
                    c.on_frame.x.abs() < 1e-12,
                    "at the punching's center: {c:?}"
                );
                continue;
            }
            assert_eq!(
                c.code,
                ContactCode::BackRailCloth,
                "nothing else within a millimeter"
            );
            match c.state() {
                ContactState::Closed => {
                    let (dx, dy) = (c.on_key.x - c.on_frame.x, c.on_key.y - c.on_frame.y);
                    assert!((dx * dx + dy * dy).sqrt() <= CLOSED_WITHIN);
                    assert!(
                        (c.on_frame.x - back_edge).abs() < 0.001,
                        "the key rests on the cloth's back edge, not at {} m",
                        c.on_frame.x
                    );
                }
                ContactState::Open => assert!(c.gap > 0.0 && c.gap <= ROW_MARGIN),
                ContactState::Overlapping => panic!("the key is drawn into its cloth: {c:?}"),
            }
        }
        assert_eq!(
            at_rest
                .iter()
                .filter(|c| c.code == ContactCode::BackRailCloth)
                .count(),
            1,
            "{at_rest:?}"
        );

        b.set(path::WEIGHT, 60.0).expect("a weight");
        assert_eq!(
            closed(&b),
            vec![
                ContactCode::FrontRailPunching,
                ContactCode::BalanceRailPunching
            ]
        );
        let f = b.frame();
        let center = Point::new(
            f.sections.front.pin_x(f.front_rail, KeyKind::Natural),
            f.front_bearing_height(),
        );
        for c in b.contacts() {
            if c.state() == ContactState::Closed && c.code == ContactCode::FrontRailPunching {
                assert!(
                    (c.on_frame.x - center.x).abs() < 1e-12,
                    "at the punching's center"
                );
                assert!((c.on_frame.y - center.y).abs() < 1e-12);
            }
        }
    }

    /// Consistency: **a key no equilibrium places is marked bearing on
    /// nothing.** With the front rail moved back to 80 mm in front of the
    /// balance pin, its pin stands under the wood of Hirschkorn's key and
    /// would lift it past the 9 mm where it leaves its balance pin: no
    /// equilibrium places the key on this frame. It is drawn level on its
    /// balance punching, which it touches there and does not bear on, so no
    /// contact is marked closed, no punching is said to overlap its edge,
    /// and no gap is read; only where its drawn parts overlap the frame's is
    /// marked, the pin in the wood.
    #[test]
    fn a_key_no_equilibrium_places_is_marked_bearing_on_nothing() {
        let front_rail_under_the_wood_mm = 80.0;
        let mut b = Instrument::standard();
        b.set(path::FRONT_RAIL_AT, front_rail_under_the_wood_mm)
            .expect("a frame edit is taken");
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
        assert_eq!(b.key_place(), KeyPlace::Unplaced);
        let marked = b.contacts();
        assert!(
            marked
                .iter()
                .all(|c| c.state() == ContactState::Overlapping),
            "{marked:?}"
        );
        assert!(
            marked
                .iter()
                .any(|c| c.code == ContactCode::FrontPinOnTheWood),
            "{marked:?}"
        );
        assert_eq!(b.edge_overlaps(), Vec::new());
        let touch = b.touch().expect("the bench says what holds the key");
        assert_eq!(
            (touch.gap_front_mm, touch.gap_back_mm, touch.travel_mm),
            (None, None, None)
        );
    }

    /// Consistency: **a key a pull lifts off its frame is marked bearing on
    /// nothing.** Pulled 63 g at its measuring point, past what its balance
    /// punching carries, the weighed section lifts off its frame: it is
    /// drawn where its last equilibrium left it, which no longer holds it,
    /// so no contact is marked closed there and no gap is read.
    #[test]
    fn a_key_a_pull_lifts_off_its_frame_is_marked_bearing_on_nothing() {
        let pull_off_its_frame_g = -63.0;
        let mut b = Instrument::weighed_section();
        b.set(path::WEIGHT, pull_off_its_frame_g)
            .expect("a weight is taken");
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
        assert_eq!(b.key_place(), KeyPlace::LiftedOff);
        let marked = b.contacts();
        assert!(
            marked
                .iter()
                .all(|c| c.state() == ContactState::Overlapping),
            "{marked:?}"
        );
        let touch = b.touch().expect("the bench says what holds the key");
        assert_eq!((touch.gap_front_mm, touch.gap_back_mm), (None, None));
    }

    /// Consistency, and the owner's way of setting friction: **the key
    /// friction weight is typed in grams at the measuring point, as
    /// Stanwood's protocol reads it, and kept as a torque.** Typed at 13 mm
    /// from the lip, the protocol reads it back in full, to the walk's
    /// resolution, and a whole gram shows as that gram; turned into a torque
    /// on the level arm instead, 6 g typed would read back 6.35 g and show
    /// 7 g. Then the bushings stay as they were set. With the measuring point
    /// moved 7 mm further back the torque keeps its bits, read on the level
    /// arm heavier in the ratio of the arms; the key friction weight zeroed
    /// on the frame moves by the ratio of the two arms the key presents where
    /// it rests, nose-up, `x cos θ − y sin θ`, to each walk's resolution; and
    /// Stanwood's reads the same bushings as more grams, the arm 3 % shorter.
    /// The key weighed heavier and the capstan moved leave the torque's bits
    /// too, and move what Stanwood's protocol reads.
    #[test]
    fn the_key_friction_weight_is_typed_in_grams_and_kept_as_a_torque() {
        let mut b = Instrument::weighed_section();
        let stanwood =
            |b: &Instrument| read::measured(b, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
        for kf in [2.0, 6.0, 13.0] {
            b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: kf })
                .expect("Stanwood's protocol reads it on this key");
            let back = stanwood(&b);
            assert!(
                (back.value - kf).abs() <= read::resolution(),
                "typed {kf} g, read back {} g",
                back.value
            );
            assert_eq!(
                back.shown.to_bits(),
                kf.to_bits(),
                "typed {kf} g, shown {} g",
                back.shown
            );
        }
        let at_13 = b.measuring_point();
        let torque = b.friction;
        assert_eq!(torque.radius_m, Some(at_13), "kept with where it was typed");
        let typed = stanwood(&b).value;
        let level = b.readout().unwrap().key_friction_weight_g;
        let protocol = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        b.set(
            path::MEASURING_POINT,
            1000.0 * (b.measuring_point_from_lip() + 0.007),
        )
        .expect("a measuring point");
        let at_20 = b.measuring_point();
        assert_eq!(b.friction, torque, "moving the point changed the bushings");
        let moved = b.readout().unwrap().key_friction_weight_g;
        assert!(
            (moved - level * at_13 / at_20).abs() <= 1e-12,
            "the same bushings read {moved} g on the level arm, not {} g, 7 mm further back",
            level * at_13 / at_20
        );
        let reread = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        let (k, seats) = b.seated().unwrap();
        let rest = seats.top.q[k.mounted.angle.0];
        let height = b.measuring_point_on_key().y;
        let arm = |x: f64| x * rest.cos() - height * rest.sin();
        let ratio = arm(at_13) / arm(at_20);
        assert!(
            (reread - protocol * ratio).abs() <= (1.0 + ratio) * read::resolution(),
            "the protocol read {protocol} g, then {reread} g, not {} g",
            protocol * ratio
        );
        let further = stanwood(&b).value;
        assert!(
            further - typed > 0.1,
            "13 g typed read {further} g 7 mm further back"
        );

        b.act(Action::WeighKey {
            mass_g: 131.1,
            front_weight_g: 12.5,
            from_lip_mm: 1000.0 * 0.013,
        })
        .expect("a heavier key is weighed");
        b.set(path::CAPSTAN_BEHIND, -1000.0 * (-0.125))
            .expect("a capstan");
        assert_eq!(b.friction, torque, "editing the key changed the bushings");
        assert_ne!(
            stanwood(&b).value.to_bits(),
            further.to_bits(),
            "and Stanwood's protocol reads them as other grams"
        );
    }

    /// Consistency, and what the owner sees: **a key friction weight no
    /// bushings read is refused, with why, and nothing changes.** Under what
    /// Stanwood's protocol reads on this key with no friction at all, about a
    /// third of a gram, it is refused with that reading; on a frame whose
    /// stops fight the protocol cannot be read at all, and the refusal names
    /// the fault the frame shows; a weight that is not a number is refused.
    /// Each time the bushings, the sheet and the readout stay as they were:
    /// never set to something else in silence.
    #[test]
    fn a_key_friction_weight_no_bushings_read_is_refused_and_nothing_changes() {
        let everything = |b: &Instrument| {
            format!(
                "{:?} {:?} {:?} {:?}",
                b.friction,
                b.friction_basis,
                b.sheet(),
                b.readout()
            )
        };
        let mut b = Instrument::weighed_section();
        let before = everything(&b);
        let refused = b
            .act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 0.0 })
            .expect_err("no bushings read 0 g on this key");
        let EditRefusal::Friction(BushingsRefusal::UnderNoFriction { least_g }) = refused else {
            panic!("{refused:?}");
        };
        assert!(least_g > 0.2 && least_g < 0.5, "{least_g}");
        assert_eq!(everything(&b), before, "a refusal changed the bench");
        assert_eq!(
            b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: f64::NAN }),
            Err(EditRefusal::Problem(Problem::NotFinite {
                param: crate::registry::KEY_FRICTION_WEIGHT,
            }))
        );
        assert_eq!(everything(&b), before, "a refusal changed the bench");

        let level = b.frame().balance_bearing_height();
        let cloth = b.frame().back_rail_cloth;
        b.set(path::BACK_RAIL_HEIGHT, 1000.0 * (level + 0.005 - cloth))
            .expect("a back rail");
        let before = everything(&b);
        let refused = b
            .act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 6.0 })
            .expect_err("on a frame whose stops fight no key friction weight is read");
        assert_eq!(
            refused,
            EditRefusal::Friction(BushingsRefusal::Procedure(Refused::Walk(
                WalkRefusal::NoTravel
            )))
        );
        assert_eq!(everything(&b), before, "a refusal changed the bench");
        assert!(b.riding().is_some(), "the stops fight, the key rides");
    }
}

#[cfg(test)]
mod paper_tests {
    use super::*;

    /// A stack is a pile of real papers, and only thicknesses that exist.
    #[test]
    fn every_paper_in_a_stack_is_one_a_technician_could_pick_up() {
        let b = Instrument::weighed_section();
        for which in [Stack::Balance, Stack::Front] {
            let stack = b.papers(which);
            assert!(!stack.is_empty(), "{which:?} stack is empty");
            for &p in stack {
                assert!(
                    pa_core::test_fixture::PAPER_THICKNESSES
                        .iter()
                        .any(|t| (t - p).abs() < 1e-12),
                    "{p} m is not a punching paper anybody stocks"
                );
            }
        }
    }

    /// Laying one more paper raises the key by exactly that paper.
    ///
    /// The operation is "add a two-tenths", not "set the stack to 0.37", and
    /// the key must move by what was added — not by a rounded version of it.
    #[test]
    fn laying_a_paper_raises_the_key_by_that_paper() {
        let mut b = Instrument::weighed_section();
        let before = b.readout().unwrap();
        let height_before = read::mm(&b, ProcedureId::KeyHeight);
        let n = b.papers(Stack::Balance).len();

        b.act(Action::AddPaper {
            stack: Stack::Balance,
            thickness_mm: 1000.0 * 0.0002,
        })
        .expect("a paper from the drawer");
        assert_eq!(b.papers(Stack::Balance).len(), n + 1);

        let risen = read::mm(&b, ProcedureId::KeyHeight) - height_before;

        // The lever is the **back rail contact's**, not the balance pin's, and
        // that is worth knowing twice over. The key still rests tail-down on
        // the cloth, so raising the bearing rotates it about where it touches
        // that cloth, and the front edge rises by the paper times (contact to
        // front edge) over (contact to bearing). And the contact is not the
        // middle of the rail: the key stands nose-up, so it meets the strip at
        // its back edge, which is 12 mm further out and 5 % of this answer. So
        // the arm is read off what the bench says it is bearing at.
        let arm = before.bears_at_the_back_mm;
        let expected = 0.2 * (arm + pa_core::test_fixture::key().front_edge * 1000.0) / arm;
        assert!(
            (risen - expected).abs() < 0.02,
            "a two-tenths raised the key by {risen} mm, expected about {expected}"
        );
    }

    /// And taking it back out puts the key back exactly where it was.
    #[test]
    fn taking_the_paper_back_out_is_exactly_reversible() {
        let mut b = Instrument::weighed_section();
        let before = read::mm(&b, ProcedureId::KeyDip);

        b.act(Action::AddPaper {
            stack: Stack::Front,
            thickness_mm: 1000.0 * 0.0004,
        })
        .expect("a paper from the drawer");
        let shimmed = read::mm(&b, ProcedureId::KeyDip);
        assert!(shimmed < before, "paper must shallow the dip");

        let last = b.papers(Stack::Front).len() - 1;
        b.act(Action::RemovePaper {
            stack: Stack::Front,
            index: u32::try_from(last).expect("an index"),
        })
        .expect("a paper lifted");
        let after = read::mm(&b, ProcedureId::KeyDip);
        assert_eq!(
            after.to_bits(),
            before.to_bits(),
            "removing the paper must restore the dip exactly: {before} -> {after}"
        );
    }

    /// A stack built from real papers cannot hit an arbitrary thickness, and
    /// the leftover is reported rather than hidden.
    #[test]
    fn a_stack_of_real_papers_misses_an_arbitrary_target_and_says_so() {
        let wanted = 0.00037;
        let stack = pa_core::test_fixture::drawer().papers_for(wanted);
        let total: f64 = stack.iter().sum();

        assert!(total <= wanted + 1e-12, "a stack must never overshoot");
        assert!(
            wanted - total < pa_core::test_fixture::PAPER_THICKNESSES[0],
            "and must get within the thinnest paper there is"
        );
        // Which is why the bench reports the achieved dip beside the target:
        // the miss is physical, not numerical.
        let b = Instrument::weighed_section();
        let r = b.readout().unwrap();
        assert!((read::mm(&b, ProcedureId::KeyDip) - r.key_dip_target_mm).abs() < 0.1);
    }
}

#[cfg(test)]
mod regulation_tests {
    use super::*;

    /// Everything the sheet, the readout and the papers say, to the bit.
    fn everything(b: &Instrument) -> String {
        format!(
            "{:?} {:?} {:?} {:?} {:?}",
            b.sheet(),
            b.readout(),
            b.papers(Stack::Balance),
            b.papers(Stack::Front),
            b.targets()
        )
    }

    /// Consistency: **regulating the bench lays the papers the solve asks
    /// for, and the sheet reads what the solve says they give**, to the bit:
    /// the solve reads the seated pose on the key mounted exactly as the
    /// bench mounts it, the papers laid included. The targets on the sheet
    /// become the new ones, every paper is one the drawer holds, and the key
    /// is moved onto its new stacks from where it stood.
    #[test]
    fn regulating_lays_the_papers_and_the_sheet_reads_what_they_give() {
        let mut b = Instrument::weighed_section();
        let targets = Targets {
            key_height: 0.0660,
            key_dip: 0.0100,
        };
        let Ok(Outcome::Laid(achieved)) = b.act(Action::Regulate {
            key_height_mm: 66.0,
            key_dip_mm: 10.0,
        }) else {
            panic!("66 mm and 10 mm are reachable");
        };
        let r = b.readout().expect("the regulated bench reads");
        assert_eq!(
            read::value(&b, ProcedureId::KeyHeight).to_bits(),
            achieved.key_height.to_bits()
        );
        assert_eq!(
            read::value(&b, ProcedureId::KeyDip).to_bits(),
            achieved.key_dip.to_bits()
        );
        assert_eq!(b.targets(), targets);
        assert!((r.key_height_target_mm - 66.0).abs() < 1e-12);
        assert!((r.key_dip_target_mm - 10.0).abs() < 1e-12);
        for which in [Stack::Balance, Stack::Front] {
            assert!(b
                .papers(which)
                .iter()
                .all(|p| pa_core::test_fixture::PAPER_THICKNESSES.contains(p)));
        }
        assert!(!b.reinitialized(), "the key follows its new stacks");
    }

    /// Consistency, and what the owner sees: **a dip the front pin forbids is
    /// refused, with its shortfall in millimeters, and nothing changes.** On
    /// the bench as built, 12 mm of dip is deeper than the front rail pin
    /// lets the key go before it bottoms in its mortise, by 0.79 mm: refused,
    /// naming the pin, and the papers, the targets, the key and every reading
    /// stay as they were.
    #[test]
    fn a_dip_the_front_pin_forbids_is_refused_and_nothing_changes() {
        let mut b = Instrument::weighed_section();
        let before = everything(&b);
        let targets = Targets {
            key_height: 0.0650,
            key_dip: 0.012,
        };
        let refused = b
            .act(Action::Regulate {
                key_height_mm: 1000.0 * targets.key_height,
                key_dip_mm: 1000.0 * targets.key_dip,
            })
            .expect_err("the pin forbids 12 mm");
        let EditRefusal::Regulation(NotRegulated::Infeasible {
            target,
            limited_by,
            shortfall_m,
        }) = refused
        else {
            panic!("{refused:?}");
        };
        assert_eq!(
            (target, limited_by),
            (Target::KeyDip, Limit::FrontPinInMortise)
        );
        assert!(
            (shortfall_m - 0.000_79).abs() < 0.000_005,
            "short by {shortfall_m} m"
        );
        assert_eq!(
            everything(&b),
            before,
            "a refused regulation changes nothing"
        );
    }

    /// A target that is not a number is refused, and changes nothing.
    #[test]
    fn a_target_that_is_not_a_number_changes_nothing() {
        let mut b = Instrument::weighed_section();
        let before = everything(&b);
        let refused = b
            .act(Action::Regulate {
                key_height_mm: f64::NAN,
                key_dip_mm: 10.2,
            })
            .expect_err("not a number");
        assert_eq!(
            refused,
            EditRefusal::Problem(Problem::NotFinite {
                param: Targets::PARAMS[0].name
            })
        );
        assert_eq!(everything(&b), before);
    }
}

#[cfg(test)]
mod protocol_roundtrip_tests {
    use super::*;

    /// Level 4, consistency, and the one that matters most: **the simulator
    /// measures its own friction with the trade's protocol and gets back
    /// what was put in**, read where the protocol reads it.
    ///
    /// Type a key friction weight, then run the protocols on the model.
    /// Stanwood's, by which it was typed, reads it back in full, to the
    /// walk's resolution, and to the gram as gram weights give it, as typed.
    /// The variant zeroed on the frame cancels the key's balance where it
    /// rests and reads the bushings alone: the torque the typed weight was
    /// turned into, through the arm the measuring point has where the key
    /// rests, nose-up by its dip, `x cos θ − y sin θ`, to the walk's
    /// resolution. Not independent of the parameter: a wrong arm, datum or
    /// sign shows here, the friction model does not.
    #[test]
    fn the_bench_measures_its_own_friction_by_the_protocol() {
        for kf in [2.0, 6.0, 9.5, 14.0] {
            let mut b = Instrument::weighed_section();
            b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: kf })
                .expect("Stanwood's protocol reads it on this key");
            let stanwood = read::measured(&b, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
            assert!(
                (stanwood.value - kf).abs() <= read::resolution(),
                "typed {kf} g of key friction, Stanwood's protocol measured {} g",
                stanwood.value
            );
            assert_eq!(
                stanwood.shown.to_bits(),
                kf.ceil().to_bits(),
                "read to the gram as gram weights give it"
            );
            let (k, seats) = b.seated().unwrap();
            let point = b.measuring_point_on_key();
            let rest = seats.top.q[k.mounted.angle.0];
            let tilted = point.x * rest.cos() - point.y * rest.sin();
            let measured = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
            let expected = 1000.0 * b.friction.bound_n_m / (G * tilted);
            assert!(
                (measured - expected).abs() <= read::resolution(),
                "the bushings set for {kf} g measured {measured} g zeroed on the frame, not \
                 the {expected} g their torque gives over the arm at rest"
            );
        }
    }

    /// A heavier key does not change what the protocol measures.
    ///
    /// The procedure zeroes the key's balance first, precisely so that the
    /// answer is friction and not balance: zeroed on the frame, the key
    /// weighed 30 g heavier with 17 g more front weight leaves it where it
    /// was, to the walk's resolution. Stanwood's, zeroed on the leveled jig,
    /// keeps what the tilt does to the key and the temporary weight, a few
    /// tenths of a gram here, under the gram it is read to.
    #[test]
    fn weighing_the_key_heavier_does_not_change_its_measured_friction() {
        let mut b = Instrument::weighed_section();
        b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 7.0 })
            .expect("Stanwood's protocol reads 7 g on this key");
        let bare = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        let stanwood = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
        let mass_g = b.key_mass_g().expect("a weighed key");
        let front_g = read::value(&b, ProcedureId::FrontWeight);

        b.act(Action::WeighKey {
            mass_g: mass_g + 30.0,
            front_weight_g: front_g + 17.0,
            from_lip_mm: 1000.0 * (b.measuring_point_from_lip()),
        })
        .expect("a heavier key is weighed");
        let heavier = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        assert!(
            (heavier - bare).abs() <= 2.0 * read::resolution(),
            "30 g more key changed the measured friction from {bare} to {heavier}"
        );
        let heavier_stanwood = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
        assert!(
            (heavier_stanwood - stanwood).abs() < 0.5,
            "Stanwood's from {stanwood} g to {heavier_stanwood} g"
        );
    }

    /// A frictionless key measures no friction, and reads none: it goes
    /// down as soon as its balance is overcome, which the walk finds to its
    /// resolution, and a threshold of nothing but the walk's own rounding
    /// reads 0 g, not the 1 g rounding it up would make of it. The bushings
    /// are set straight, with no friction: Stanwood's protocol reads about a
    /// third of a gram on this key even then, so no key friction weight
    /// typed gives them.
    #[test]
    fn a_frictionless_key_measures_nothing() {
        let mut b = Instrument::weighed_section();
        read::bushings_holding_g(&mut b, 0.0);
        let m = read::measured(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        assert!(
            m.value.abs() <= read::resolution(),
            "a key with no friction should need no weight to fall, got {} g",
            m.value
        );
        assert_eq!(
            m.shown.to_bits(),
            0.0_f64.to_bits(),
            "and reads 0 g, not {} g",
            m.shown
        );
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;
    use crate::registry::path;

    /// Consistency: **the procedures never read the session's state.** The
    /// whole sheet, D and U and the key friction weights with key height and
    /// dip, and the seats' bearing at the back and out of level at the
    /// bottom, are the same bits whatever the weight on the measuring point
    /// has done to the key: each procedure seats the key by rule, friction
    /// off, from its nominal pose. Friction moves the weighings and leaves
    /// the seats as they were.
    #[test]
    fn the_sheet_reads_the_seats_whatever_the_touch_weight_did() {
        let mut b = Instrument::weighed_section();
        let seated = |b: &Instrument| {
            let r = b.readout().unwrap();
            [
                read::value(b, ProcedureId::KeyHeight),
                read::value(b, ProcedureId::KeyDip),
                r.out_of_level_deg,
                r.bedding_at_the_back_deg,
                r.bears_at_the_back_mm,
            ]
            .map(f64::to_bits)
        };
        let (sheet, before) = (b.sheet(), seated(&b));
        for weight in [-60.0, -6.0, 60.0, 0.0] {
            b.set(path::WEIGHT, weight).expect("a weight");
            assert_eq!(b.sheet(), sheet, "{weight} g");
            assert_eq!(seated(&b), before, "{weight} g");
        }
        b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 18.0 })
            .expect("Stanwood's protocol reads 18 g on this key");
        assert_eq!(seated(&b), before, "18 g of friction");
    }

    /// An edit the key can follow moves it from where it stood: a paper laid
    /// on the front punching under a key down on it lifts the key by what
    /// the paper reaches, an overlap inside the row margin, and the key
    /// stays on its punching.
    #[test]
    fn an_edit_the_key_can_follow_moves_it_from_where_it_stood() {
        let mut b = Instrument::weighed_section();
        b.set(path::WEIGHT, 60.0).expect("a weight");
        let pressed = b.key_placement().world(b.measuring_point_on_key());
        b.act(Action::AddPaper {
            stack: Stack::Front,
            thickness_mm: 1000.0 * 0.0002,
        })
        .expect("a paper from the drawer");
        assert!(!b.reinitialized(), "a paper is followed, not started again");
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            })
        );
        let lifted = b.key_placement().world(b.measuring_point_on_key());
        assert!(
            lifted.y > pressed.y,
            "the paper must lift the front of the key"
        );
    }

    /// An edit the key cannot follow starts it again from the
    /// initialization, and the bench says so: a front rail raised 3 mm under
    /// a key down on its punching sinks the punching 3 mm into the key, past
    /// the row margin, where no increment reads it truthfully. The key is
    /// put back on its frame, on the punching where it now stands.
    #[test]
    fn an_edit_the_key_cannot_follow_starts_it_again() {
        let mut b = Instrument::weighed_section();
        b.set(path::WEIGHT, 60.0).expect("a weight");
        let front = b.frame().front_rail_height;
        b.set(path::FRONT_RAIL_HEIGHT, 1000.0 * (front + 0.003))
            .expect("a front rail");
        assert!(b.reinitialized(), "sunk 3 mm, the key must be put back");
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            })
        );
        b.set(path::WEIGHT, 61.0).expect("a weight");
        assert!(
            !b.reinitialized(),
            "and the next action is an increment again"
        );
    }

    /// Consistency: **a weight on a frame nobody touched is solved as one
    /// increment from where the key stands, every time.** Lifted and pressed
    /// in turn, the key travels from stop to stop with its friction acting,
    /// and the last step of a travel may land it a few nanometers into the
    /// stop: the box problem accepts its multipliers to a fraction of their
    /// scale, which a soft metric turns into nanometers of gap. The increment
    /// must close that overlap, not stop on it until its cap, after which the
    /// bench would start the key again by its loading rule, friction off.
    /// These weights on these two benches reach such a landing, so a solver
    /// that stops on the overlap fails here.
    #[test]
    fn a_weight_on_an_untouched_frame_is_always_one_increment() {
        let weights = [
            -40.0, 23.0, -40.0, 22.0, -40.0, 30.0, -60.0, 60.0, -60.0, 10.0, -60.0, 1.0,
        ];
        let mut stiff = Instrument::weighed_section();
        read::bushings_holding_g(&mut stiff, 15.0);
        let mut moved = Instrument::weighed_section();
        let front = moved.frame().front_rail;
        let (balance, front_height) = (
            moved.frame().balance_rail_height,
            moved.frame().front_rail_height,
        );
        moved
            .set(path::FRONT_RAIL_AT, 1000.0 * (front - 0.0008))
            .expect("a front rail");
        moved
            .set(path::BALANCE_RAIL_HEIGHT, 1000.0 * (balance - 0.0015))
            .expect("a balance rail");
        moved
            .set(path::FRONT_RAIL_HEIGHT, 1000.0 * (front_height + 0.0003))
            .expect("a front rail");
        for (name, b) in [
            ("15 g of friction", &mut stiff),
            ("rails moved", &mut moved),
        ] {
            for w in weights {
                b.set(path::WEIGHT, w).expect("a weight");
                assert!(
                    !b.reinitialized(),
                    "{name}, {w} g: the increment failed and the key was started again"
                );
            }
        }
    }

    /// Consistency, determinism: the same actions on two benches give the
    /// same bits, the state, the drawing and every reading.
    #[test]
    fn the_same_actions_give_the_same_bits() {
        let run = || {
            let mut b = Instrument::weighed_section();
            b.set(path::WEIGHT, -20.0).expect("a weight");
            b.act(Action::WeighKey {
                mass_g: 131.1,
                front_weight_g: 12.5,
                from_lip_mm: 1000.0 * 0.013,
            })
            .expect("a heavier key is weighed");
            b.act(Action::AddPaper {
                stack: Stack::Balance,
                thickness_mm: 1000.0 * 0.0002,
            })
            .expect("a paper from the drawer");
            b.set(path::WEIGHT, -4.0).expect("a weight");
            b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 9.0 })
                .expect("Stanwood's protocol reads 9 g on this key");
            b.set(path::WEIGHT, 30.0).expect("a weight");
            let r = b.readout().unwrap();
            let p = b.key_placement();
            (
                b.state
                    .clone()
                    .map(|s| s.q.iter().map(|x| x.to_bits()).collect::<Vec<_>>()),
                [p.origin.x, p.origin.y, p.cos, p.sin].map(f64::to_bits),
                format!("{r:?} {:?}", b.sheet()),
            )
        };
        assert_eq!(run(), run());
    }
}

#[cfg(test)]
mod measuring_point_tests {
    use super::*;
    use crate::registry::path;
    use pa_core::test_fixture::MEASURING_POINT_HALF_INCH;

    /// Level 0: **moving the measuring point changes front weight exactly as
    /// the lever says**, and nothing about the key.
    ///
    /// Front weight is the key's static moment about the pin over the arm to
    /// the measuring point. Moved 7 mm further back from the lip, the moment is
    /// the same and the arm 7 mm shorter, so the reading rises in the ratio of
    /// the two arms. The arms are worked from the key's own front edge and
    /// covering overhang, not from the code that places the point. Key mass,
    /// height and dip are read off the key and at the key front, so they must
    /// not move: the mass and the height to the bit, since nothing they are
    /// read from reads the measuring point; the dip to within what the
    /// bottom seat converges to, since the press that seats the key there
    /// acts at the measuring point, and a static solve under another load
    /// takes another path to the same stop, which it closes within
    /// `GAP_TOLERANCE`, seen at the key front through the lever from the
    /// front rail, 258 over 225 mm.
    #[test]
    fn moving_the_measuring_point_changes_front_weight_exactly_by_the_lever() {
        let mut b = Instrument::weighed_section();
        let key = b.key().clone();
        let lip = key.front_edge + key.keytop_overhang;
        let moment_g_m = 1000.0 * read::static_moment_kg_m(&b);

        // The default: 13 mm back from the covering's lip, which stands 2 mm
        // proud of the key front at 258 mm, so the arm is 247 mm.
        assert!((b.measuring_point_from_lip() - 0.013).abs() < 1e-15);
        assert!(
            (b.measuring_point() - (0.258 + 0.002 - 0.013)).abs() < 1e-12,
            "the default measuring point is 247 mm in front of the balance pin, not {} m",
            b.measuring_point()
        );

        let before = b.readout().unwrap();
        // The bench is built with 6 g of key friction weight taken at its own
        // measuring point, so it reads the 6 g back there. A friction radius
        // left at another point would read as another number here.
        assert!(
            (before.key_friction_weight_g - 6.0).abs() < 1e-12,
            "the bench is set to {} g of key friction, not the 6 g it was built with",
            before.key_friction_weight_g
        );
        let fw_before = read::value(&b, ProcedureId::FrontWeight);
        let (height_before, dip_before) = (
            read::mm(&b, ProcedureId::KeyHeight),
            read::mm(&b, ProcedureId::KeyDip),
        );
        let arm_before = lip - b.measuring_point_from_lip();
        b.set(
            path::MEASURING_POINT,
            1000.0 * (b.measuring_point_from_lip() + 0.007),
        )
        .expect("a measuring point");
        let after = b.readout().unwrap();
        let fw_after = read::value(&b, ProcedureId::FrontWeight);
        let arm_after = lip - b.measuring_point_from_lip();

        // The same static moment, read on each arm.
        for (fw, arm) in [(fw_before, arm_before), (fw_after, arm_after)] {
            assert!(
                (fw * arm - moment_g_m).abs() < 1e-12,
                "{fw} g on a {arm} m arm is not the key's static moment, {moment_g_m} g·m"
            );
        }
        let expected = fw_before * arm_before / arm_after;
        assert!(
            (fw_after - expected).abs() < 1e-12,
            "7 mm further back, front weight read {fw_after} g against the lever's {expected} g"
        );

        // The key itself has not changed, and neither has its key front.
        assert_eq!(after.key_mass_g, before.key_mass_g);
        assert_eq!(read::mm(&b, ProcedureId::KeyHeight), height_before);
        let bottom = 2.0 * pa_core::solve::statics::GAP_TOLERANCE * 1000.0 * (258.0 / 225.0);
        let dip_after = read::mm(&b, ProcedureId::KeyDip);
        assert!(
            (dip_after - dip_before).abs() <= bottom,
            "the dip moved from {dip_before} to {dip_after} mm"
        );
    }

    /// Level 0: **every protocol reads the same measuring point.**
    ///
    /// Set to the glossary's other value, 1/2 inch, so that a reading still
    /// taken at a stale 13 mm cannot pass by agreeing with the default. Each
    /// protocol's own output is then turned back into the arm it was read on,
    /// and every one must be the covering's lip less the setting:
    ///
    /// * the jig: front weight `FW = M / arm`, key weight ratio
    ///   `KR = capstan arm / arm`;
    /// * the key friction weight zeroed on the frame, by the walk: the
    ///   friction torque over the arm the key presents at rest, nose-up,
    ///   `cos θ · arm − sin θ · h`, to the walk's resolution;
    /// * the touch weight, which acts on the key at that point.
    ///
    /// The friction is set before the point moves, so the key friction weight
    /// also checks that it is kept as a torque with its radius: read at the
    /// new point it is the same torque over a different arm, not the same
    /// grams.
    #[test]
    fn every_protocol_reads_the_same_measuring_point() {
        let mut b = Instrument::weighed_section();
        let key = b.key().clone();
        let lip = key.front_edge + key.keytop_overhang;
        let height = key.stick_height + key.keytop_thickness;

        let kf_g = 6.0;
        read::bushings_holding_g(&mut b, kf_g);
        let torque = kf_g * 1e-3 * (lip - b.measuring_point_from_lip());

        b.set(path::MEASURING_POINT, 1000.0 * MEASURING_POINT_HALF_INCH)
            .expect("a measuring point");
        let arm = lip - b.measuring_point_from_lip();
        assert!((b.measuring_point_from_lip() - MEASURING_POINT_HALF_INCH).abs() < 1e-15);
        assert!((b.measuring_point() - arm).abs() < 1e-15);

        let m = read::static_moment_kg_m(&b);
        let read_on = [
            (
                "front weight",
                1000.0 * m / read::value(&b, ProcedureId::FrontWeight),
            ),
            (
                "key weight ratio",
                -key.capstan / read::value(&b, ProcedureId::KeyWeightRatio),
            ),
        ];
        for (what, on) in read_on {
            assert!(
                (on - arm).abs() < 1e-12,
                "{what} was read on a {on} m arm; the measuring point's is {arm} m"
            );
        }

        // The key friction weight, by the walk, over the arm the key presents
        // on its cloth from the same point.
        let (k, seats) = b.seated().unwrap();
        let rest = seats.top.q[k.mounted.angle.0];
        let tilted = rest.cos() * arm - rest.sin() * height;
        let kf = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        assert!(
            (kf - 1000.0 * torque / tilted).abs() <= read::resolution(),
            "the key friction weight was read on a {} m arm; the measuring point presents \
             {tilted} m",
            1000.0 * torque / kf
        );

        // And the touch weight acts there too.
        let mounted = b.key_on_frame().unwrap();
        let at = b.touch_load(&mounted).at.at;
        assert_eq!(
            at.x.to_bits(),
            arm.to_bits(),
            "the touch weight stands at {at:?}"
        );
        assert_eq!(at.y.to_bits(), height.to_bits());
    }

    /// The marker on the section follows the setting.
    ///
    /// Moved 7 mm further back from the lip, the drawn marker moves 7 mm along
    /// the top of the key, whichever stop the key is on. The weight on the
    /// measuring point holds the key on that stop, so the attitude does not
    /// change and the whole of the marker's movement is the setting's.
    #[test]
    fn the_drawn_measuring_point_follows_the_setting() {
        for weight in [20.0, -20.0] {
            let mut b = Instrument::weighed_section();
            b.set(path::WEIGHT, weight).expect("a weight");
            let s = b.sin_angle();
            let before = b.key_placement().world(b.measuring_point_on_key());

            let back = 0.007;
            b.set(
                path::MEASURING_POINT,
                1000.0 * (b.measuring_point_from_lip() + back),
            )
            .expect("a measuring point");
            assert!(
                (b.sin_angle() - s).abs() < 1e-15,
                "under {weight} g the key must stay on its stop"
            );
            let after = b.key_placement().world(b.measuring_point_on_key());

            // Back along the key's top: in the keybed frame that is
            // `(−cos θ, sin θ)` times the distance moved.
            let c = (1.0 - s * s).sqrt();
            assert!(
                (after.x - before.x + c * back).abs() < 1e-12
                    && (after.y - before.y - s * back).abs() < 1e-12,
                "under {weight} g the marker moved ({}, {}) m, not {back} m along the key",
                after.x - before.x,
                after.y - before.y
            );
        }
    }

    /// Consistency, the validation layer: **the measuring point stays on
    /// the covering, refused anywhere else, and nothing changes.** In front
    /// of the lip there is no key to put a weight on; behind the covering
    /// the point would no longer stand on the top the statics put it on; a
    /// value that is not a distance is none. Each is refused with why, the
    /// covering's length beside it, and the instrument stays as it was.
    #[test]
    fn the_measuring_point_stays_on_the_covering() {
        let mut b = Instrument::weighed_section();
        let before = b.state_hash();
        for from_lip_mm in [-5.0, 500.0] {
            assert_eq!(
                b.set(path::MEASURING_POINT, from_lip_mm),
                Err(EditRefusal::Problem(Problem::MeasuringPointOffCovering {
                    from_lip: from_lip_mm / 1000.0,
                    covering: b.key().keytop_length,
                }))
            );
            assert_eq!(b.state_hash(), before);
        }
        assert!(matches!(
            b.set(path::MEASURING_POINT, f64::NAN),
            Err(EditRefusal::Problem(Problem::NotFinite { .. }))
        ));
        assert_eq!(b.state_hash(), before);
        b.set(path::MEASURING_POINT, 155.0)
            .expect("the covering's back end is on it");
    }

    /// A weight on the key moves with the measuring point, and the key is
    /// solved again under it, from where it stands.
    ///
    /// A key weighed back-heavy rests on its cloth with 10.6 g on a measuring
    /// point 30 mm back from the lip: on that arm the weight cannot overcome
    /// the key's balance and its friction, and friction holds it there.
    /// Moved to 5 mm, the same weight acts on an arm 25 mm longer and takes
    /// the key down to its punching, with nobody touching the weight. The key
    /// leaves its cloth above 11.2 g at 30 mm and above 10.1 g at 5 mm, so
    /// 10.6 g is half a gram clear of either.
    #[test]
    fn moving_the_point_moves_the_weight_on_it_and_the_key_settles_again() {
        let mut b = Instrument::weighed_section();
        b.act(Action::WeighKey {
            mass_g: 151.1,
            front_weight_g: -8.1,
            from_lip_mm: 1000.0 * 0.013,
        })
        .expect("a back-heavy key is weighed");
        read::bushings_holding_g(&mut b, 2.0);
        b.set(path::MEASURING_POINT, 1000.0 * 0.030)
            .expect("a measuring point");
        b.set(path::WEIGHT, 10.6).expect("a weight");
        assert!(
            matches!(
                b.finger_verdict(),
                Some(TouchVerdict::HeldByFriction {
                    touching: Some(ContactCode::BackRailCloth),
                    ..
                })
            ),
            "the key must still be on its cloth: {:?}",
            b.finger_verdict()
        );

        b.set(path::MEASURING_POINT, 1000.0 * 0.005)
            .expect("a measuring point");
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            }),
            "the key must have gone down to its punching"
        );
    }
}

#[cfg(test)]
mod edit_tests {
    use super::*;
    use crate::registry::{path, Control, ENTRIES, PAGE};

    /// Everything the instrument says, to the bit: its hash, its settings,
    /// its sheet, where the key is drawn and its log.
    fn everything(b: &Instrument) -> String {
        format!(
            "{:016x} {} {:?} {:?} {:?}",
            b.state_hash(),
            crate::boundary::json(&b.settings()),
            b.sheet(),
            b.poses(),
            b.log()
        )
    }

    /// `b` refuses `edit` with `why`, and stays as it was to the bit; the
    /// diagnostics keep the edit.
    fn refuses(b: &mut Instrument, edit: Edit, why: &dyn Fn(&EditRefusal) -> bool) {
        let before = everything(b);
        let answer = match edit.clone() {
            Edit::Set { path, value } => b.set(&path, value).map(|()| Outcome::Done),
            Edit::Act(action) => b.act(action),
        };
        let refused = answer.expect_err("refused");
        assert!(why(&refused), "{edit:?}: {refused:?}");
        assert_eq!(everything(b), before, "{edit:?} changed the instrument");
        // Compared as written, a value that is not a number being unequal to
        // itself.
        assert_eq!(
            format!("{:?}", b.refused_edit()),
            format!("{:?}", Some(&RefusedEdit { edit, why: refused }))
        );
    }

    fn set(path: &str, value: f64) -> Edit {
        Edit::Set {
            path: path.to_owned(),
            value,
        }
    }

    /// Consistency, the validation layer: **each problem has a probe that
    /// triggers it, and a refused edit leaves the instrument as it was**,
    /// its hash, its settings, its sheet, where the key is drawn and its
    /// log, to the bit. A key front brought back to 170 mm runs its covering
    /// 14 mm into its key button (the audit's RUST-6); a capstan 200 mm
    /// behind the pin stands 9.5 mm in the back check block, 20 mm behind
    /// it in the key button, and 230 mm behind it, on the thinned tail,
    /// behind the back check block, 39.5 mm short of clearing it; a front
    /// mortise 30 mm deep is cut through the 24.5 mm of the keystick; a
    /// measuring point 160 mm back from the lip is off a 155 mm covering; a
    /// felt thinner than nothing is out of range; a key weighed at no mass,
    /// or less, is no key (the audit's BWA-3); a paper no drawer holds is no
    /// paper; a paper not on its stack cannot be lifted; a path the
    /// instrument has not, or one an action sets, is named so.
    #[test]
    fn each_problem_is_refused_and_leaves_the_instrument_as_it_was() {
        let mut b = Instrument::weighed_section();
        refuses(&mut b, set(path::FRONT_EDGE, 170.0), &|w| {
            matches!(w, EditRefusal::Problem(Problem::OutlineCrossesItself {
                back: PartCode::KeyButton,
                front: PartCode::Keytop,
                by,
            }) if (by - 0.014).abs() < 1e-12)
        });
        refuses(&mut b, set(path::CAPSTAN_BEHIND, 200.0), &|w| {
            matches!(w, EditRefusal::Problem(Problem::CapstanInBlock {
                block: PartCode::BackCheckBlock,
                by,
            }) if (by - 0.0095).abs() < 1e-12)
        });
        refuses(&mut b, set(path::CAPSTAN_BEHIND, 20.0), &|w| {
            matches!(
                w,
                EditRefusal::Problem(Problem::CapstanInBlock {
                    block: PartCode::KeyButton,
                    ..
                })
            )
        });
        refuses(&mut b, set(path::CAPSTAN_BEHIND, 230.0), &|w| {
            matches!(w, EditRefusal::Problem(Problem::CapstanBehindBackCheckBlock {
                by,
            }) if (by - 0.0395).abs() < 1e-12)
        });
        refuses(&mut b, set(path::FRONT_MORTISE_DEPTH, 30.0), &|w| {
            *w == EditRefusal::Problem(Problem::MortiseThroughKeyTop {
                depth: 0.030,
                wood: 0.0245,
            })
        });
        refuses(&mut b, set(path::MEASURING_POINT, 160.0), &|w| {
            *w == EditRefusal::Problem(Problem::MeasuringPointOffCovering {
                from_lip: 0.160,
                covering: 0.155,
            })
        });
        refuses(&mut b, set(path::BALANCE_FELT, -1.0), &|w| {
            *w == EditRefusal::Problem(Problem::OutOfRange {
                param: "balance_punching.felt",
                value: -0.001,
                min: 0.0,
                max: f64::INFINITY,
            })
        });
        for mass_g in [0.0, -5.0] {
            refuses(
                &mut b,
                Edit::Act(Action::WeighKey {
                    mass_g,
                    front_weight_g: 6.5,
                    from_lip_mm: 13.0,
                }),
                &|w| matches!(w, EditRefusal::Problem(Problem::MassNotPositive { .. })),
            );
        }
        refuses(
            &mut b,
            Edit::Act(Action::AddPaper {
                stack: Stack::Front,
                thickness_mm: 0.3,
            }),
            &|w| matches!(w, EditRefusal::Problem(Problem::NotInTheDrawer { .. })),
        );
        let papers = b.papers(Stack::Front).len();
        refuses(
            &mut b,
            Edit::Act(Action::RemovePaper {
                stack: Stack::Front,
                index: 9,
            }),
            &|w| {
                *w == EditRefusal::NoSuchPaper {
                    stack: Stack::Front,
                    index: 9,
                    papers,
                }
            },
        );
        refuses(&mut b, set("key.length_mm", 300.0), &|w| {
            matches!(w, EditRefusal::UnknownPath { .. })
        });
        refuses(&mut b, set(path::KEY_DIP, 10.0), &|w| {
            *w == EditRefusal::SetByAction {
                path: path::KEY_DIP,
                action: ActionCode::Regulate,
            }
        });
        assert!(b.log().is_empty(), "nothing refused is logged");
    }

    /// Consistency, the audit's RUST-17: **no number that is not finite
    /// reaches the core**, on any path or in any action's argument: NaN and
    /// both infinities are refused as not finite, naming the path or the
    /// argument, and the instrument stays as it was. A path an action sets
    /// is refused by `set` before its value is read, naming the action.
    #[test]
    fn no_number_that_is_not_finite_reaches_the_core() {
        let mut b = Instrument::weighed_section();
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for e in &ENTRIES {
                let param = e.param();
                refuses(&mut b, set(e.path, bad), &|w| match e.set_by() {
                    Some(action) => {
                        *w == EditRefusal::SetByAction {
                            path: e.path,
                            action,
                        }
                    }
                    None => *w == EditRefusal::Problem(Problem::NotFinite { param }),
                });
            }
            let not_finite =
                |w: &EditRefusal| matches!(w, EditRefusal::Problem(Problem::NotFinite { .. }));
            for action in [
                Action::Regulate {
                    key_height_mm: bad,
                    key_dip_mm: 10.2,
                },
                Action::Regulate {
                    key_height_mm: 65.0,
                    key_dip_mm: bad,
                },
                Action::WeighKey {
                    mass_g: bad,
                    front_weight_g: 6.5,
                    from_lip_mm: 13.0,
                },
                Action::WeighKey {
                    mass_g: 121.0,
                    front_weight_g: bad,
                    from_lip_mm: 13.0,
                },
                Action::WeighKey {
                    mass_g: 121.0,
                    front_weight_g: 6.5,
                    from_lip_mm: bad,
                },
                Action::AddPaper {
                    stack: Stack::Balance,
                    thickness_mm: bad,
                },
                Action::SetFrictionFromKeyFrictionWeight { kf_g: bad },
            ] {
                refuses(&mut b, Edit::Act(action), &not_finite);
            }
        }
    }

    /// Consistency, and the check of `.claude/rules/metrology.md` > Upstream settings drift, they do not cascade: **everything an assembled
    /// keyboard can be is taken, computed and reported, not refused.** A
    /// front pin standing 40 mm proud of its rail, out through what its
    /// mortise allows, is a state: the key rides on it. A front rail moved
    /// 60 mm back under a key that does not fit it is a frame drifted from
    /// its key, its fault named in millimeters. A regulation that cannot be
    /// laid is refused, not the frame.
    #[test]
    fn everything_an_assembled_keyboard_can_be_is_taken() {
        let mut b = Instrument::weighed_section();
        b.set(path::FRONT_PIN_HEIGHT, 40.0)
            .expect("a proud pin is a state");
        assert!(b.finger_verdict().is_some(), "and something holds the key");
        let mut c = Instrument::weighed_section();
        c.set(path::FRONT_RAIL_AT, 165.0)
            .expect("a frame placed for another key");
        assert!(!c.faults().is_empty(), "its fault is named");
    }

    /// The edits `b` takes for the replay and the hash: a front rail raised
    /// and the bench reset, weights that leave the key on its cloth by
    /// friction, a paper laid and another lifted, the key front moved, the
    /// key weighed, its friction typed, a regulation, the measuring point
    /// moved, the frame placed under the key; and an edit refused among
    /// them. Every action is among them, so none goes unlogged unseen.
    fn exercised() -> Instrument {
        let mut b = Instrument::weighed_section();
        b.set(path::FRONT_RAIL_HEIGHT, 30.0)
            .expect("a front rail raised");
        b.act(Action::Reset).expect("the bench reset");
        b.set(path::WEIGHT, -60.0).expect("a pull");
        b.set(path::WEIGHT, -5.0)
            .expect("a pull its friction holds");
        b.act(Action::AddPaper {
            stack: Stack::Front,
            thickness_mm: 0.2,
        })
        .expect("a paper");
        b.act(Action::RemovePaper {
            stack: Stack::Front,
            index: 0,
        })
        .expect("the bottom paper lifted");
        b.set(path::FRONT_EDGE, 259.0).expect("a key front");
        b.act(Action::WeighKey {
            mass_g: 125.5,
            front_weight_g: 7.25,
            from_lip_mm: 13.0,
        })
        .expect("a weighing");
        b.act(Action::SetFrictionFromKeyFrictionWeight { kf_g: 8.0 })
            .expect("a key friction weight");
        b.set(path::FRONT_EDGE, 170.0)
            .expect_err("a covering in the key button");
        b.act(Action::Regulate {
            key_height_mm: 65.5,
            key_dip_mm: 10.0,
        })
        .expect("a regulation");
        b.set(path::MEASURING_POINT, 12.7).expect("half an inch");
        b.act(Action::PlaceFrameUnderKey).expect("the rails moved");
        b
    }

    /// Consistency: **the log replayed on the instrument it started from
    /// gives the same state**, its hash and every document to the bit, the
    /// refused edit having no part in it. The path matters: friction holds
    /// the key where the last weight left it, so the same weights without
    /// the pull that lifted the key leave it elsewhere, and hash otherwise.
    #[test]
    fn the_log_replayed_from_the_profile_reproduces_the_state() {
        let b = exercised();
        assert_eq!(b.log().len(), 12, "twelve edits taken, one refused");
        let codes: Vec<ActionCode> = b
            .log()
            .iter()
            .filter_map(|e| match e {
                Edit::Act(a) => Some(a.code()),
                Edit::Set { .. } => None,
            })
            .collect();
        for code in [
            ActionCode::AddPaper,
            ActionCode::RemovePaper,
            ActionCode::PlaceFrameUnderKey,
            ActionCode::Regulate,
            ActionCode::WeighKey,
            ActionCode::SetFrictionFromKeyFrictionWeight,
            ActionCode::Reset,
        ] {
            assert!(codes.contains(&code), "{code:?} is not logged");
        }
        let again = Instrument::weighed_section()
            .replay(b.log())
            .expect("the log replays");
        assert_eq!(again.state_hash(), b.state_hash());
        assert_eq!(everything(&again), everything(&b));
        let pull = 2;
        assert_eq!(b.log()[pull], set(path::WEIGHT, -60.0));
        let without_the_pull = Instrument::weighed_section()
            .replay(&[&b.log()[..pull], &b.log()[pull + 1..]].concat())
            .expect("the log replays");
        assert_ne!(without_the_pull.state_hash(), b.state_hash());
    }

    /// Consistency: **the state hash changes with each normative coordinate
    /// and with the hidden state, and with nothing else.** From the tests'
    /// instrument, every path set to another value, every action taken,
    /// moves the hash; two instruments built alike hash alike, whatever
    /// their logs, and a refused edit does not move it. Two instruments with
    /// every coordinate the same, a 5 g pull on each, hash apart when one
    /// key was lifted onto its cloth first, where friction holds it, and
    /// the other was not: the key's place is the hidden state.
    #[test]
    fn the_state_hash_changes_with_every_coordinate_and_the_hidden_state() {
        let standard = Instrument::weighed_section().state_hash();
        assert_eq!(Instrument::weighed_section().state_hash(), standard);
        let soft = |path: &str| {
            PAGE.iter()
                .flat_map(|(_, controls)| controls.iter())
                .find_map(|c| match *c {
                    Control::Slider { path: p, soft, .. } if p == path => Some(soft),
                    _ => None,
                })
        };
        for e in ENTRIES.iter().filter(|e| e.set_by().is_none()) {
            let mut b = Instrument::weighed_section();
            let now = b.value(e.path).expect("a value");
            // A tenth of what the page offers away from where it stands,
            // forward or else back, whichever a key can be built at.
            let step = soft(e.path).map_or(0.25 * now, |s| {
                0.1 * (s.max.expect("bounded") - s.min.expect("bounded"))
            });
            let other = [now + step, now - step]
                .into_iter()
                .find(|&v| b.clone().set(e.path, v).is_ok())
                .unwrap_or_else(|| panic!("{} takes no value near {now}", e.path));
            b.set(e.path, other).expect("taken");
            assert_ne!(b.state_hash(), standard, "{} = {other}", e.path);
        }
        for action in [
            Action::AddPaper {
                stack: Stack::Balance,
                thickness_mm: 0.08,
            },
            Action::RemovePaper {
                stack: Stack::Front,
                index: 0,
            },
            Action::Regulate {
                key_height_mm: 65.5,
                key_dip_mm: 10.0,
            },
            Action::WeighKey {
                mass_g: 125.5,
                front_weight_g: 7.25,
                from_lip_mm: 13.0,
            },
            Action::SetFrictionFromKeyFrictionWeight { kf_g: 8.0 },
        ] {
            let mut b = Instrument::weighed_section();
            b.act(action.clone()).expect("taken");
            assert_ne!(b.state_hash(), standard, "{action:?}");
        }
        let mut refused = Instrument::weighed_section();
        refused
            .set(path::FRONT_EDGE, 170.0)
            .expect_err("a covering in the key button");
        assert_eq!(refused.state_hash(), standard);

        let mut down = Instrument::weighed_section();
        down.set(path::WEIGHT, -5.0).expect("a pull");
        let mut up = Instrument::weighed_section();
        up.set(path::WEIGHT, -60.0).expect("a pull");
        up.set(path::WEIGHT, -5.0).expect("a pull");
        assert_eq!(
            crate::boundary::json(&down.settings()),
            crate::boundary::json(&up.settings()),
            "every coordinate the same"
        );
        assert_ne!(down.sin_angle(), up.sin_angle(), "the key stands apart");
        assert_ne!(down.state_hash(), up.state_hash());
        let mut reset = up.clone();
        reset.act(Action::Reset).expect("reset");
        assert_eq!(reset.state_hash(), standard, "reset is the profile");
    }

    /// Consistency: **each number of the state moves the hash on its own.**
    /// Changed alone, by the least step its bits take, with no increment
    /// solved after it to move the key too: every setting a path reaches,
    /// each of a weighing's numbers, a paper moved from one stack to the
    /// other, the key's row, what its friction rests on, each coordinate,
    /// the lift, what the rows carried, and the flags of the last
    /// increment. The log and the last refusal are not state: they leave
    /// it.
    #[test]
    fn each_number_of_the_state_moves_the_hash_on_its_own() {
        /// The least step from `x`, its bits one apart.
        fn next(x: f64) -> f64 {
            f64::from_bits(x.to_bits() + 1)
        }
        let moves = |what: &str, b: &Instrument, edit: &dyn Fn(&mut Instrument)| {
            let mut c = b.clone();
            edit(&mut c);
            assert_ne!(c.state_hash(), b.state_hash(), "{what}");
        };
        let b = Instrument::weighed_section();
        for e in &ENTRIES {
            let what = e.path;
            match e.field {
                Field::Key(s) => moves(what, &b, &|c| {
                    let x = next((s.get)(&c.key));
                    (s.set)(&mut c.key, x);
                }),
                Field::Frame(s) => moves(what, &b, &|c| {
                    let x = next((s.get)(&c.frame));
                    (s.set)(&mut c.frame, x);
                }),
                Field::Protocol(s) => moves(what, &b, &|c| {
                    let x = next((s.get)(&c.protocol));
                    (s.set)(&mut c.protocol, x);
                }),
                Field::Friction(s) => moves(what, &b, &|c| {
                    let x = next((s.get)(&c.friction));
                    (s.set)(&mut c.friction, x);
                }),
                Field::Weight(s) => moves(what, &b, &|c| {
                    let x = next((s.get)(&c.weight_g));
                    (s.set)(&mut c.weight_g, x);
                }),
                Field::Target(s) => moves(what, &b, &|c| {
                    let x = next((s.get)(&c.targets));
                    (s.set)(&mut c.targets, x);
                }),
                Field::Weighing(_) | Field::Papers(_) => {}
            }
        }
        let mut weighed = b.clone();
        weighed
            .act(Action::WeighKey {
                mass_g: 125.5,
                front_weight_g: 7.25,
                from_lip_mm: 13.0,
            })
            .expect("a weighing");
        for which in 0..5 {
            moves("a weighing's number", &weighed, &|c| {
                let KeyMass::Weighed {
                    mass,
                    static_moment_n_m,
                    datum,
                } = &mut c.mass
                else {
                    unreachable!("weighed")
                };
                let x = [
                    mass,
                    static_moment_n_m,
                    &mut datum.grams,
                    &mut datum.from_lip_m,
                    &mut datum.arm_m,
                ]
                .into_iter()
                .nth(which)
                .expect("five");
                *x = next(*x);
            });
        }
        moves("a paper moved to the other stack", &b, &|c| {
            let paper = c.frame.balance_punching.papers.pop().expect("a paper");
            c.frame.front_punching.papers.insert(0, paper);
        });
        moves("the key's row", &b, &|c| c.kind = KeyKind::Sharp);
        moves("what the friction rests on", &b, &|c| {
            c.friction_basis = Basis::Measured;
        });
        let q = b.state.as_ref().expect("a state").q.len();
        for i in 0..q {
            moves("a coordinate", &b, &|c| {
                let x = &mut c.state.as_mut().expect("a state").q[i];
                *x = next(*x);
            });
        }
        moves("the lift", &b, &|c| {
            let x = &mut c.state.as_mut().expect("a state").lift;
            *x = next(*x);
        });
        moves("what the rows carried", &b, &|c| {
            c.state.as_mut().expect("a state").carried = Carried::NOTHING;
        });
        moves("no state", &b, &|c| c.state = None);
        moves("lifting off", &b, &|c| c.lifts_off = !c.lifts_off);
        moves("reinitialized", &b, &|c| c.reinitialized = !c.reinitialized);

        let mut kept = b.clone();
        kept.log.push(set(path::WEIGHT, 1.0));
        kept.refused = Some(RefusedEdit {
            edit: set(path::FRONT_EDGE, 170.0),
            why: EditRefusal::UnknownPath {
                path: String::new(),
            },
        });
        assert_eq!(kept.state_hash(), b.state_hash());
    }

    /// Consistency, the validation layer on load: **an instrument is not
    /// built from a design no part could be built as**, any more than it
    /// is edited into one: a felt thinner than nothing, a capstan screwed
    /// down to a millimeter, a key front in its key button, a key shortened
    /// at the back by 10 mm, its back check block left where the action
    /// puts it, which runs its thinned tail 10 mm under the block. Lengthened
    /// at the back by 30 mm instead, on a frame placed under it, the key is
    /// built, and regulated to the measured section's targets.
    #[test]
    fn a_design_no_part_could_be_built_as_is_refused_on_load() {
        let targets = Instrument::weighed_section().targets();
        let mut frame = pa_core::test_fixture::frame();
        frame.balance_punching.felt = -0.001;
        assert!(matches!(
            read::built(pa_core::test_fixture::key(), frame, targets),
            Err(EditRefusal::Problem(Problem::OutOfRange {
                param: "balance_punching.felt",
                ..
            }))
        ));
        let mut key = pa_core::test_fixture::key();
        key.capstan_screw.height = 0.001;
        assert!(matches!(
            read::built(key, pa_core::test_fixture::frame(), targets),
            Err(EditRefusal::Problem(Problem::OutOfRange {
                param: "capstan.height",
                ..
            }))
        ));
        let key = KeyDesign {
            front_edge: 0.170,
            ..pa_core::test_fixture::key()
        };
        assert!(matches!(
            read::built(key, pa_core::test_fixture::frame(), targets),
            Err(EditRefusal::Problem(Problem::OutlineCrossesItself { .. }))
        ));
        let mut key = pa_core::test_fixture::key();
        key.tail_end += 0.010;
        assert!(matches!(
            read::built(key, pa_core::test_fixture::frame(), targets),
            Err(EditRefusal::Problem(Problem::OutlineCrossesItself {
                back: PartCode::KeyTailFelt,
                front: PartCode::BackCheckBlock,
                by,
            })) if (by - 0.010).abs() < 1e-12
        ));
        let mut key = pa_core::test_fixture::key();
        key.tail_end -= 0.030;
        let frame = pa_core::test_fixture::frame().placed_under(&key);
        let built = read::built(key, frame, targets);
        assert!(built.is_ok(), "{:?}", built.err());
    }

    /// Consistency, on Hirschkorn's key: **his capstan slider is refused
    /// only where the capstan would stand on lowered wood or run into
    /// another part.** Over the slider's range, by its step, a capstan
    /// taken stands where it is set, seated on the keystick's full height;
    /// one refused is refused as its head standing in the key button or the
    /// back check block, or over the thinned tail, which his key's full
    /// height ends at 132.45 mm behind the pin, each by how far as the
    /// parts' places say, and the instrument is left as it was. His capstan's
    /// head, 3.92 mm in radius, clears the key button from 42.01 mm behind
    /// the pin and stands over the full height to 128.53 mm, over the tail
    /// from there, and in the back check block, which ends 186.87 mm behind
    /// the pin, from 182.95 mm: so the slider, from 60 to 200 mm by the
    /// millimeter, takes the capstan from 60 to 128 mm, and refuses it over
    /// the tail from 129 to 182 mm and in the block from 183 mm.
    #[test]
    fn his_capstan_slider_is_refused_only_on_lowered_wood_or_in_a_part() {
        let his = Instrument::standard();
        let key = his.key().clone();
        let soft = PAGE
            .iter()
            .flat_map(|(_, controls)| controls.iter())
            .find_map(|c| match *c {
                Control::Slider { path: p, soft, .. } if p == path::CAPSTAN_BEHIND => Some(soft),
                _ => None,
            })
            .expect("a capstan slider");
        let (min, max) = (soft.min.expect("bounded"), soft.max.expect("bounded"));
        let r = 0.5
            * key
                .capstan_screw
                .head_diameter
                .max(key.capstan_screw.diameter);
        let (button_back, button_front) = key.key_button_span();
        let check = key.back_check_block();
        let step = key.tail_front();
        let (mut taken, mut on_the_tail, mut in_the_block) = (Vec::new(), Vec::new(), Vec::new());
        let mut behind_mm = min;
        while behind_mm <= max {
            let x = -1e-3 * behind_mm;
            let (back, front) = (x - r, x + r);
            let refused = if front > button_back && back < button_front {
                Some(Problem::CapstanInBlock {
                    block: PartCode::KeyButton,
                    by: front - button_back,
                })
            } else if back < check.front() {
                Some(Problem::CapstanInBlock {
                    block: PartCode::BackCheckBlock,
                    by: check.front() - back,
                })
            } else if back < step {
                Some(Problem::CapstanOffTheFullHeight { by: step - back })
            } else {
                None
            };
            let mut b = his.clone();
            match (b.set(path::CAPSTAN_BEHIND, behind_mm), refused) {
                (Ok(()), None) => {
                    assert_eq!(b.key().capstan_seat().to_bits(), key.stick_height.to_bits());
                    assert!(
                        (b.key().capstan - x).abs() <= f64::EPSILON * 0.2,
                        "{behind_mm} mm"
                    );
                    taken.push(behind_mm);
                }
                (Err(EditRefusal::Problem(got)), Some(want)) => {
                    let (got_by, want_by) = match (got, want) {
                        (
                            Problem::CapstanInBlock { block, by },
                            Problem::CapstanInBlock {
                                block: want_block,
                                by: want_by,
                            },
                        ) if block == want_block => {
                            if block == PartCode::BackCheckBlock {
                                in_the_block.push(behind_mm);
                            }
                            (by, want_by)
                        }
                        (
                            Problem::CapstanOffTheFullHeight { by },
                            Problem::CapstanOffTheFullHeight { by: want_by },
                        ) => {
                            on_the_tail.push(behind_mm);
                            (by, want_by)
                        }
                        (got, want) => panic!("{behind_mm} mm: {got:?}, not {want:?}"),
                    };
                    assert!(
                        (got_by - want_by).abs() < 1e-12,
                        "{behind_mm} mm: {got_by}, not {want_by}"
                    );
                    assert_eq!(b.key(), his.key(), "{behind_mm} mm: refused and changed");
                    assert_eq!(b.state_hash(), his.state_hash());
                }
                (got, want) => panic!("{behind_mm} mm: {got:?}, the parts' places say {want:?}"),
            }
            behind_mm += soft.step;
        }
        assert!(
            (button_back - r + 0.042_005).abs() < 1e-9,
            "{}",
            button_back - r
        );
        assert!((step + r + 0.128_53).abs() < 1e-9, "{}", step + r);
        assert!(
            (check.front() + r + 0.182_95).abs() < 1e-9,
            "{}",
            check.front() + r
        );
        let mm = |from: u32, to: u32| (from..=to).map(f64::from).collect::<Vec<f64>>();
        assert_eq!(taken, mm(60, 128));
        assert_eq!(on_the_tail, mm(129, 182));
        assert_eq!(in_the_block, mm(183, 200));
    }
}

/// The band on every instrument a profile ships: its extremes against the
/// walks, a step half as long, and the non-confusion pair through it.
#[cfg(test)]
mod band_tests {
    use super::*;
    use crate::registry::path;
    use pa_core::measure::sweep::STEP;

    /// Every shipped profile's instrument as it arrives, a key nobody has
    /// weighed weighed by the tests' weighing, since a key with no mass has
    /// no band.
    fn shipped() -> Vec<(&'static str, Instrument)> {
        Profile::names()
            .map(|name| {
                let profile = match name {
                    crate::profile::SECTION => Profile::weighed_section(),
                    _ => Profile::named(name).expect("a shipped profile"),
                };
                (name, Instrument::of(profile).expect("it builds"))
            })
            .collect()
    }

    /// The band of `b`, which must be drawn.
    fn band(b: &Instrument) -> Band {
        b.band().expect("the band is drawn").band
    }

    /// `b`'s band in steps of `step`, from the seats the sheet reads.
    fn band_by(b: &Instrument, step: f64) -> Band {
        let (k, seats) = b.seated().expect("the key seats");
        Seating {
            mounted: &k.mounted,
            capstan: k.capstan,
            measuring_point: b.measuring_point_on_key(),
            margin: SEAT_MARGIN,
            standing: &[],
        }
        .sweep_by(&mut Statics::new(), &seats, step)
        .expect("the band is drawn")
    }

    /// How far the extreme a curve reads may stand from the extreme of the
    /// force it samples, grams: the curve's greatest change over one step,
    /// and twice its greatest change of slope over one. Derived as
    /// `pa_core::measure::sweep`'s tests derive it: an extreme inside the
    /// stroke stands within half a step of a reading; one at a seat is read
    /// a step inside it at most, and the change over that step differs from
    /// the change over the next by the curve's bend over the two.
    fn within_a_step(curve: &[Sample]) -> f64 {
        let g: Vec<f64> = curve.iter().map(|s| s.grams).collect();
        let step = g
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f64, f64::max);
        let bend = g
            .windows(3)
            .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
            .fold(0.0_f64, f64::max);
        step + 2.0 * bend
    }

    /// How far a reading of `b` may stand from the same reading worked
    /// another way by rounding alone, grams: a few products of the key's
    /// weight times an arm under 0.3 m and the bushings' bound, each rounded
    /// at its last bit, over the measuring point's arm, at least 0.2 m; 64
    /// units in the last place of that.
    fn rounding(b: &Instrument) -> f64 {
        let mass = b.key_mass_g().expect("a weighed key") * 1e-3;
        64.0 * f64::EPSILON * 1000.0 * (mass * G * 0.3 + b.friction().bound_n_m) / (G * 0.2)
    }

    /// The curve's steepest change per radian between two readings, grams.
    fn per_radian(curve: &[Sample]) -> f64 {
        curve
            .windows(2)
            .map(|w| ((w[1].grams - w[0].grams) / (w[1].angle - w[0].angle)).abs())
            .fold(0.0_f64, f64::max)
    }

    /// The measuring point's arm about the pin at attitude `theta`, over its
    /// level arm.
    fn tilted(b: &Instrument, theta: f64) -> f64 {
        let p = b.measuring_point_on_key();
        1.0 - p.y / p.x * theta.tan()
    }

    /// The owner's section weighed as he is likeliest to weigh one of its
    /// keys: whole grams of mass and of front weight, the front weight read
    /// 13 mm back from the lip. The key bottoms out about level, where its
    /// balance is its front weight and its bushings hold their 6 g, so its up
    /// weight, read off the bottom seat, stands within a few ten-thousandths
    /// of a gram of a whole gram: the walk resolves it to a part in 65 536
    /// of a gram, the sweep, a step off the seat, to about a thousandth, and
    /// the two fall either side of that gram.
    fn weighed_in_whole_grams() -> Vec<(String, Instrument)> {
        [
            (130.0, 40.0),
            (120.0, 25.0),
            (125.0, 30.0),
            (135.0, 45.0),
            (140.0, 35.0),
        ]
        .into_iter()
        .map(|(mass_g, front_weight_g)| {
            let mut b = Instrument::of(Profile::named(crate::profile::SECTION).unwrap()).unwrap();
            b.act(Action::WeighKey {
                mass_g,
                front_weight_g,
                from_lip_mm: 13.0,
            })
            .expect("a key's weighing");
            (format!("{mass_g} g, {front_weight_g} g at 13 mm"), b)
        })
        .collect()
    }

    /// Consistency, the second code path, on both shipped keys and on the
    /// section weighed in whole grams: **the band's extremes are the
    /// sheet's down and up weights in full.** The greatest force of the
    /// down curve against the walk's D, the least of the up curve against
    /// its U, within the walk's resolution and the curve's change over the
    /// step the sweep reads its extreme from, with rounding. Their grams
    /// are not compared: a value in full within that bound of a whole gram
    /// reads the gram on one side or the other, as the section weighed at
    /// 130 g and 40 g shows, its U read -47 g by the walk from -46.00019 g
    /// and -46 g on the band from -45.99997 g. The gram is the walk's, the
    /// trade's protocol with gram weights; the band is a curve, and the page
    /// labels its extremes in full.
    #[test]
    fn the_bands_extremes_are_the_sheets_in_full_on_every_shipped_profile() {
        let keys = shipped()
            .into_iter()
            .map(|(name, b)| (name.to_owned(), b))
            .chain(weighed_in_whole_grams());
        for (name, b) in keys {
            let banded = b.band().expect("the band is drawn");
            for (on_band, walked, curve) in [
                (
                    &banded.down_weight,
                    ProcedureId::DownWeight,
                    &banded.band.down,
                ),
                (&banded.up_weight, ProcedureId::UpWeight, &banded.band.up),
            ] {
                let sheet = read::measured(&b, walked);
                let bound = read::resolution() + within_a_step(curve) + rounding(&b);
                assert!(
                    (on_band.value - sheet.value).abs() <= bound,
                    "{name}: {:?} {} g on the band, {} g by the walk, more than {bound} g apart",
                    on_band.procedure,
                    on_band.value,
                    sheet.value
                );
                assert!(
                    bound < 0.01,
                    "{name}: a step reads under a hundredth of a gram"
                );
                assert_eq!(on_band.trust, sheet.trust, "{name}: trusted alike");
                assert_eq!(on_band.fingerprint, sheet.fingerprint);
            }
        }
    }

    /// C12, consistency, on both shipped keys: **a step half as long reads
    /// the same extremes**, within the curve's change over a step of the
    /// sweep at its own step, with rounding.
    #[test]
    fn halving_the_sweeps_step_on_every_shipped_profile() {
        for (name, b) in shipped() {
            let (whole, half) = (band_by(&b, STEP), band_by(&b, 0.5 * STEP));
            assert_eq!(whole, band(&b), "{name}: the sweep the instrument draws");
            for (w, h, curve) in [
                (whole.down_weight, half.down_weight, &whole.down),
                (whole.up_weight, half.up_weight, &whole.up),
            ] {
                let bound = within_a_step(curve) + rounding(&b);
                assert!(
                    (w.grams - h.grams).abs() <= bound,
                    "{name}: {} g by whole steps, {} g by half steps, more than {bound} g apart",
                    w.grams,
                    h.grams
                );
            }
        }
    }

    /// Level 0, consistency, the first non-confusion test through the band,
    /// on both shipped keys, the way the bench changes a key: **weighed with
    /// its front weight a gram heavier, the band's middle moves and its
    /// width does not.** The gram is a static moment on the measuring
    /// point's level arm; at an attitude `θ` it turns the key by its cosine,
    /// read at the measuring point's arm `x cos θ − y sin θ`, so both curves
    /// fall by `1 / (1 − (y / x) tan θ)` at every travel: the middle by
    /// that, the width by nothing. At the extremes, the middle `(D + U) / 2`
    /// falls by the mean of the two attitudes' and the width `D − U` moves
    /// by their difference, a few thousandths of a gram. The tolerance is
    /// rounding, that of the static moment the weighing keeps, 64 `ε` of the
    /// front weight, and what the reweighed key's seats move each step's
    /// angle by, read, through the curve's steepest slope per radian, twice
    /// for its bend.
    #[test]
    fn a_front_weight_one_gram_heavier_moves_the_bands_middle_not_its_width() {
        for (name, before) in shipped() {
            let mass_g = before.key_mass_g().expect("a weighed key");
            let front_g = read::value(&before, ProcedureId::FrontWeight);
            let mut after = before.clone();
            after
                .act(Action::WeighKey {
                    mass_g,
                    front_weight_g: front_g + 1.0,
                    from_lip_mm: 1000.0 * before.measuring_point_from_lip(),
                })
                .expect("a gram heavier in front is a key's weighing");
            let (b0, b1) = (band(&before), band(&after));
            let slope = 2.0 * per_radian(&b0.down).max(per_radian(&b0.up));
            let fixed = 2.0 * rounding(&before) + 64.0 * f64::EPSILON * front_g.abs();
            let moved = |s0: &Sample, s1: &Sample| slope * (s1.angle - s0.angle).abs();
            let fell = |theta: f64| -1.0 / tilted(&before, theta);
            for (s0, s1) in b0.down.iter().zip(&b1.down).chain(b0.up.iter().zip(&b1.up)) {
                let by_hand = fell(s0.angle);
                assert!(
                    (s1.grams - s0.grams - by_hand).abs() <= fixed + moved(s0, s1),
                    "{name}: at {} mm the gram moved the curve by {} g, by hand {by_hand} g",
                    s0.travel * 1000.0,
                    s1.grams - s0.grams
                );
            }
            for ((d0, u0), (d1, u1)) in b0
                .down
                .iter()
                .zip(b0.up.iter().rev())
                .zip(b1.down.iter().zip(b1.up.iter().rev()))
            {
                let width = (d1.grams - u1.grams) - (d0.grams - u0.grams);
                let bound = 2.0 * fixed + moved(d0, d1) + moved(u0, u1);
                assert!(width.abs() <= bound, "{name}: the width moved by {width} g");
            }
            let (d0, u0, d1, u1) = (b0.down_weight, b0.up_weight, b1.down_weight, b1.up_weight);
            let tolerance = 2.0 * fixed + moved(&d0, &d1) + moved(&u0, &u1);
            let (on_d, on_u) = (fell(d0.angle), fell(u0.angle));
            let middle = 0.5 * (d1.grams + u1.grams) - 0.5 * (d0.grams + u0.grams);
            let width = (d1.grams - u1.grams) - (d0.grams - u0.grams);
            assert!(
                (middle - 0.5 * (on_d + on_u)).abs() <= tolerance,
                "{name}: the middle moved by {middle} g, by hand {} g",
                0.5 * (on_d + on_u)
            );
            assert!(
                (width - (on_d - on_u)).abs() <= tolerance,
                "{name}: the width moved by {width} g, the tilt between the extremes accounts \
                 for {} g",
                on_d - on_u
            );
            assert!(middle < -0.99, "{name}: a gram of front weight: {middle}");
            assert!(
                width.abs() < 0.01,
                "{name}: and a few thousandths of width: {width}"
            );
        }
    }

    /// Level 0, consistency, the second non-confusion test through the band,
    /// on both shipped keys: **bushings 1.2 times as tight widen the band and
    /// leave its middle.** The bushings' torque set a fifth higher, the
    /// curves stand that bound over the measuring point's arm either side of
    /// the key's balance at every travel: the width grows by a fifth, the
    /// middle moves by rounding alone. At the extremes the width grows by
    /// the extra bound read at the two attitudes' arms, the middle by half
    /// their difference, under a hundredth of a gram. Friction does not
    /// seat the key, so every step holds the same angle, to the bit.
    #[test]
    fn bushings_a_fifth_tighter_widen_the_band_and_leave_its_middle() {
        for (name, base) in shipped() {
            let mut tighter = base.clone();
            let torque = base
                .value(path::FRICTION_TORQUE)
                .expect("the bushings' torque");
            tighter
                .set(path::FRICTION_TORQUE, 1.2 * torque)
                .expect("tighter bushings");
            let (b0, b1) = (band(&base), band(&tighter));
            let tolerance = 2.0 * rounding(&tighter);
            let level = 1000.0 * (tighter.friction().bound_n_m - base.friction().bound_n_m)
                / (G * base.measuring_point_on_key().x);
            let extra = |theta: f64| level / tilted(&base, theta) / theta.cos();
            for (s0, s1) in b0.down.iter().zip(&b1.down).chain(b0.up.iter().zip(&b1.up)) {
                assert_eq!(s0.angle.to_bits(), s1.angle.to_bits(), "{name}");
            }
            for ((d0, u0), (d1, u1)) in b0
                .down
                .iter()
                .zip(b0.up.iter().rev())
                .zip(b1.down.iter().zip(b1.up.iter().rev()))
            {
                let middle = 0.5 * (d1.grams + u1.grams) - 0.5 * (d0.grams + u0.grams);
                assert!(
                    middle.abs() <= tolerance,
                    "{name}: the middle moved by {middle} g"
                );
                let widened = (d1.grams - u1.grams) - (d0.grams - u0.grams);
                assert!(
                    (widened - 2.0 * extra(d0.angle)).abs() <= 2.0 * tolerance,
                    "{name}: at {} mm the band widened by {widened} g, by hand {} g",
                    d0.travel * 1000.0,
                    2.0 * extra(d0.angle)
                );
            }
            let (top, bottom) = (b0.down_weight.angle, b0.up_weight.angle);
            let width = (b1.down_weight.grams - b1.up_weight.grams)
                - (b0.down_weight.grams - b0.up_weight.grams);
            let middle = 0.5 * (b1.down_weight.grams + b1.up_weight.grams)
                - 0.5 * (b0.down_weight.grams + b0.up_weight.grams);
            assert!(
                (width - (extra(top) + extra(bottom))).abs() <= 2.0 * tolerance,
                "{name}: the width moved by {width} g, by hand {} g",
                extra(top) + extra(bottom)
            );
            let tilt = 0.5 * (extra(top) - extra(bottom));
            assert!(
                (middle - tilt).abs() <= 2.0 * tolerance,
                "{name}: the middle moved by {middle} g, the extremes' arms account for {tilt} g"
            );
            assert!(tilt.abs() < 0.01, "{name}: under a hundredth of a gram");
            let was = b0.down_weight.grams - b0.up_weight.grams;
            assert!(
                (width / was - 0.2).abs() < 0.01,
                "{name}: about a fifth wider"
            );
        }
    }

    /// Consistency: **the band is taken when it is asked for, and changes
    /// nothing**: the instrument's state hash and its sheet are the same
    /// after it as before, so a page that draws it has not edited the key.
    #[test]
    fn drawing_the_band_changes_nothing() {
        let b = Instrument::weighed_section();
        let (hash, sheet) = (b.state_hash(), b.sheet());
        let _ = b.band();
        assert_eq!(b.state_hash(), hash);
        assert_eq!(b.sheet(), sheet);
    }
}

/// The owner's measured section as it arrives: a key nobody has weighed.
#[cfg(test)]
mod section_tests {
    use super::*;
    use crate::profile::SECTION;
    use crate::registry::path;

    /// The measured section as it arrives, regulated and unweighed.
    fn unweighed() -> Instrument {
        Instrument::of(Profile::named(SECTION).expect("shipped")).expect("regulated on arrival")
    }

    /// Consistency, design 5.3: **a key nobody has weighed reads what needs
    /// no mass and refuses the rest, saying why.** Front weight, the down and
    /// up weights, Stanwood's key friction weight and the band read the
    /// key's mass, and are refused: the key is not weighed; the balance and
    /// friction weights, read from the down and up weights, are refused with
    /// them. Key height and dip are its seats under the trade's gestures,
    /// the key weight ratio the jig's tare, and the key friction weight
    /// zeroed on the frame cancels the key's balance where it rests: they
    /// are read. The faults are read off the drawing. Its readout carries no
    /// mass and no touch verdict, where a weight leaves the key depending on
    /// a balance only a weighing gives, and the key is drawn where its top
    /// seat puts it, whatever weight stands on it.
    #[test]
    fn a_key_nobody_has_weighed_reads_what_needs_no_mass_and_refuses_the_rest() {
        let mut b = unweighed();
        assert_eq!(b.key_mass(), KeyMass::Unweighed);
        let sheet = b.sheet();
        for p in [
            ProcedureId::FrontWeight,
            ProcedureId::DownWeight,
            ProcedureId::UpWeight,
            ProcedureId::KeyFrictionWeightZeroedOnTheJig,
        ] {
            assert_eq!(sheet.get(p), Some(&Err(Refused::KeyNotWeighed)), "{p:?}");
        }
        for p in [ProcedureId::BalanceWeight, ProcedureId::FrictionWeight] {
            assert!(
                matches!(sheet.get(p), Some(Err(Refused::From { .. }))),
                "{p:?}"
            );
        }
        for p in [
            ProcedureId::KeyWeightRatio,
            ProcedureId::KeyHeight,
            ProcedureId::KeyDip,
            ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
        ] {
            assert!(
                matches!(sheet.get(p), Some(Ok(_))),
                "{p:?}: {:?}",
                sheet.get(p)
            );
        }
        assert!(matches!(b.band(), Err(Refused::KeyNotWeighed)));
        assert!(b.faults().is_empty());
        let r = b.readout().expect("an unweighed key is seated");
        assert_eq!((r.key_mass_g, r.finger), (None, None));
        assert!(b.touch().is_none());
        let (_, seats) = b.seated().expect("seated");
        let top = seats.top.q.clone();
        assert_eq!(b.state.as_ref().map(|s| s.q.clone()), Some(top.clone()));
        b.set(path::WEIGHT, 30.0).expect("a weight on it");
        assert_eq!(b.state.as_ref().map(|s| s.q.clone()), Some(top));
        assert!(b.touch().is_none());
    }

    /// Consistency: **what reads no mass reads the same weighed or not.**
    /// Weighed by the tests' weighing, the key reads the key weight ratio of
    /// the key nobody weighed to round-off, the ratio being a tare; its key
    /// height and dip the same seats on rigid stops, the load that seats it
    /// sized to its weight, each closed by the static solve within
    /// `GAP_TOLERANCE` and seen at the key front through the lever from the
    /// cloth, under 2.3 times it, and from the front rail, 258 over 225 mm;
    /// and the key friction weight zeroed on the frame within twice the
    /// walk's resolution, its temporary weight canceling the weighed key's
    /// balance where it rests, as nothing needs canceling on a key with no
    /// mass. Weighed, the key is given a touch verdict and a mass.
    #[test]
    fn what_reads_no_mass_reads_the_same_weighed_or_not() {
        let unweighed = unweighed();
        let mut weighed = unweighed.clone();
        weighed
            .act(Action::WeighKey {
                mass_g: pa_core::test_fixture::WEIGHED_MASS_G,
                front_weight_g: pa_core::test_fixture::WEIGHED_FRONT_WEIGHT_G,
                from_lip_mm: 1000.0 * pa_core::test_fixture::WEIGHED_FROM_LIP,
            })
            .expect("the tests' weighing");
        let gap = pa_core::solve::statics::GAP_TOLERANCE;
        for (p, bound) in [
            (ProcedureId::KeyWeightRatio, 16.0 * f64::EPSILON),
            (ProcedureId::KeyHeight, 2.0 * gap * 2.3),
            (ProcedureId::KeyDip, 2.0 * gap * (2.3 + 258.0 / 225.0)),
            (
                ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
                2.0 * read::resolution(),
            ),
        ] {
            let (was, is) = (read::value(&unweighed, p), read::value(&weighed, p));
            assert!(
                (is - was).abs() <= bound,
                "{p:?}: {was} unweighed, {is} weighed"
            );
        }
        assert_eq!(
            weighed.key_mass_g().map(f64::to_bits),
            Some((pa_core::test_fixture::WEIGHED_MASS_G / 1000.0 * 1000.0).to_bits())
        );
        assert!(weighed.touch().is_some());
    }

    /// Consistency: **a weighed key's center of mass stands at its drawn
    /// section's centroid's height, and every reading on the tilted key
    /// says so.** The statics take its mass and its static moment as weighed
    /// and the height from [`KeyDesign::section_centroid`], to the bit; the
    /// down weight, read on the key tilted on its frame, is indicative for
    /// that height among its inputs, and the front weight, read on the
    /// leveled jig, is not.
    #[test]
    fn a_weighed_keys_center_of_mass_stands_at_its_sections_centroid() {
        let b = Instrument::weighed_section();
        let props = b.mass_props().expect("a weighed key");
        let centroid = b.key().section_centroid().expect("the key encloses wood");
        assert_eq!(props.com.y.to_bits(), centroid.y.to_bits());
        let because = |p: ProcedureId| match read::measured(&b, p).trust {
            Trust::Indicative { because } => because,
            Trust::Metrological => Vec::new(),
        };
        assert!(because(ProcedureId::DownWeight).contains(&Input::CenterOfMassHeight));
        assert!(!because(ProcedureId::FrontWeight).contains(&Input::CenterOfMassHeight));
    }
}
