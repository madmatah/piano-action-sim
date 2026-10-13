//! The single-key weighing bench, as a use case.
//!
//! This is the application layer's first citizen: it owns a key, a frame and a
//! regulation, answers the questions a technician asks of them, and accepts the
//! operations he performs. It knows nothing about JavaScript, about rendering or
//! about files — `pa-wasm` adapts it for the browser and `pa-cli` for the shell,
//! and neither of them is allowed to add physics.
//!
//! # Why the key's weighing lives here and not in a panel
//!
//! Weighing the key is an **operation on the model**, not widget state: the
//! same call serves the browser, the command line and the test suite, and an
//! exercise can be scripted and replayed. A mass and a front weight typed into
//! a panel that only updated a React store would be labels beside a key that
//! weighs what it weighed before.

use pa_core::contact::{Bearing as ContactBearing, Part, Rows, ROW_MARGIN};
pub use pa_core::contact::{ContactCode, HowItBears, Material, PartCode, Provenance, Role, Sweep};
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
/// Why a walk cannot be taken to the end of the key's stroke.
pub use pa_core::measure::walk::WalkRefusal;
use pa_core::measure::weighing::Weighing;
/// The weighing sheet and why a reading on it is refused, and why no
/// bushings read a key friction weight typed.
pub use pa_core::measure::weighing::{BushingsRefusal, Refused, Sheet};
use pa_core::measure::TouchweightProtocol;
use pa_core::mechanism::{
    AxisFriction, BodyId, MassProps, MaterialPoint, Placement, PointLoad, Problem,
};
pub use pa_core::parts::faults::Fault;
/// The punching thicknesses a drawer holds, re-exported so an adapter can
/// offer the drawer without reaching past the application layer.
pub use pa_core::parts::frame::PAPER_THICKNESSES;
use pa_core::parts::frame::{papers_for, KeyKind};
use pa_core::parts::mounts::{self, KeyMount, Mounted, OnItsFrame};
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
use pa_core::solve::{Friction, Statics};
use pa_core::G;

use crate::composite_key;

/// What the bench reports beside its weighing sheet ([`KeyBench::sheet`]),
/// in the trade's own units: how the key stands on its frame, what holds it
/// under the touch weight, and the settings the sheet was read with.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchReadout {
    /// The key's mass, grams: as it was weighed ([`KeyBench::weigh_key`]), or
    /// until it is, the composite key's derived from its volume
    /// ([`composite_key`]), not a weighing.
    pub key_mass_g: f64,
    /// Gap at the front rail punching where the key stands now, under the
    /// touch weight, millimetres: the felt's drawn top, at its centre, to the
    /// key's underside. Zero when the key is down on it; `None` when the
    /// punching stands where no underside passes over its centre.
    pub gap_front_mm: Option<f64>,
    /// Gap at the back rail cloth where the key stands now, millimetres: the
    /// least of the cloth's drawn top to the key's drawn underside, chamfer
    /// included. `None` when no feature of the two can meet.
    pub gap_back_mm: Option<f64>,
    /// The bushings' friction as the bench is set, the torque read as grams
    /// at the measuring point as it stands now, with the key level: the half
    /// width of the band of weights there that leave the level key still.
    /// Not the key friction weight it was typed as, which Stanwood's
    /// protocol reads on the key resting nose-up ([`KeyBench::sheet`]). A
    /// torque ([`AxisFriction`]), so moving the measuring point changes this
    /// and not the bushings.
    pub key_friction_weight_g: f64,
    /// Key height the regulation was asked for, millimetres.
    pub key_height_target_mm: f64,
    /// Key dip the regulation was asked for, millimetres.
    pub key_dip_target_mm: f64,
    /// What holds the key where the weight on its measuring point has left
    /// it: a stop, or its friction.
    pub finger: TouchVerdict,
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
    /// Positive is the face rising towards the player, so its rear end is the
    /// low one and that is what bears. The chamfer under the tail exists so
    /// that this is zero — the owner's own account of why it is there — and it
    /// is reported rather than enforced: reaching it is a property of the key's
    /// shape against the rest attitude the frame produces, and a real keyboard
    /// may miss.
    pub bedding_at_the_back_deg: f64,
    /// Where the key bears on the back rail cloth, millimetres **behind** the
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
    /// How far above its balance punching it stands, metres.
    pub lift: f64,
}

/// How the key stands at its two seats, read by the trade's gestures
/// ([`KeyBench::seat_report`]).
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

    /// Where the key bears on its cloth at the top, millimetres behind the
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
/// and the gaps at its two outer stops there ([`KeyBench::touch`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Touch {
    /// What holds it.
    pub verdict: TouchVerdict,
    /// The gap at the front punching, millimetres
    /// ([`BenchReadout::gap_front_mm`]).
    pub gap_front_mm: Option<f64>,
    /// The gap at the back rail cloth, millimetres
    /// ([`BenchReadout::gap_back_mm`]).
    pub gap_back_mm: Option<f64>,
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
        /// How far above its balance punching it stands, millimetres.
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
        /// How far above its balance punching it stands, millimetres.
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
/// increment left them, the placement they give each body, and how far above
/// its balance punching that left the key, metres.
#[derive(Debug, Clone, PartialEq)]
struct State {
    q: Vec<f64>,
    placements: Vec<Placement>,
    lift: f64,
}

/// Why a regulation the bench was asked for was not laid. The papers, the
/// targets and the key stay as they were.
#[derive(Debug, Clone, PartialEq)]
pub enum NotRegulated {
    /// A target no stack reaches: which, what keeps it out of reach, and how
    /// far it lies out of reach in its own reading, metres.
    Infeasible {
        /// Which target.
        target: Target,
        /// What keeps it out of reach.
        limited_by: Limit,
        /// How far, metres.
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
    /// The signed gap between the two, metres: negative where the drawn parts
    /// overlap.
    pub gap: f64,
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

/// How near zero a gap must be for the contact to count as closed, metres:
/// a tenth of a micron.
///
/// A thousandth of the thinnest paper in the drawer, far below anything the
/// section can show, and a hundred million times the rounding of a contact
/// the statics have closed, which leaves a few femtometres.
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

/// Which punching stack an operation applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    /// Under the key at the balance pin. Sets key height.
    Balance,
    /// On the front rail. Sets key dip.
    Front,
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
/// ([`KeyBench::poses`]): the outline does not move when the key does.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// The body that carries it: [`BodyId::GROUND`] or [`KEY`].
    pub body: BodyId,
    /// What it is.
    pub code: PartCode,
    /// Material, void or lining.
    pub role: Role,
    /// What it is made of. The view chooses its colour from this.
    pub material: Material,
    /// How the outline becomes a solid.
    pub sweep: Sweep,
    /// Closed polygon, metres, in the body's frame; for a turned part, its
    /// profile in radius and height about the sweep's axis.
    pub points: Vec<Point>,
    /// The segment of the part's outline each point starts or lies on.
    pub segs: Vec<u16>,
    /// Which segments come from where.
    pub sources: Vec<(core::ops::Range<u16>, Provenance)>,
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

/// The bench.
#[derive(Debug, Clone)]
pub struct KeyBench {
    /// The key.
    key: KeyDesign,
    /// What is known of the key's mass: an estimate, the composite key's
    /// volume model, until it is weighed whole.
    mass: KeyMass,
    /// The frame it sits on.
    frame: FrameDesign,
    /// How the key is weighed: where on it every weight is centred. Held
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
    finger_weight_g: f64,
    /// The targets of the regulation last laid, kept beside what the key
    /// reads.
    targets: Targets,
    /// The papers actually stacked under the balance punching, in the order
    /// they were laid.
    balance_papers: Vec<f64>,
    /// The papers on the front punching.
    front_papers: Vec<f64>,
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
    /// Whether the last action put the key back by the initialization
    /// instead of moving it from where it stood.
    reinitialized: bool,
}

impl KeyBench {
    /// A bench regulated to the given targets: the key on the frame with the
    /// felts it is given, and on them the papers the regulation solve lays
    /// ([`pa_core::regulation`]).
    ///
    /// Refused when the targets cannot be met on this geometry, with why:
    /// a real answer and not an error to swallow.
    pub fn new(key: KeyDesign, frame: FrameDesign, targets: Targets) -> Result<Self, NotRegulated> {
        // Stanwood's protocol until the bench is told otherwise; see
        // `set_measuring_point_from_lip`.
        let protocol = TouchweightProtocol::default();
        // Six grams is the order of Key Friction Weight on a well-centred key;
        // it is a starting value to be replaced by a measurement, and it is the
        // single parameter the two non-confusion tests move. [S] It is set
        // straight as the torque 6 g overcome at the measuring point with the
        // key level, an assumption and not a reading: Stanwood's protocol,
        // run on the key resting nose-up, reads these bushings a few tenths of
        // a gram heavier. A key friction weight typed is turned into a torque
        // by that protocol instead (`set_key_friction_weight`). The radius is
        // this key's measuring point, read where every protocol reads it, not
        // a number retyped from the default key: that would give any other key
        // a friction bound measured on someone else's arm.
        let friction = AxisFriction::of_weight_at(6.0, protocol.measuring_point(&key));
        let mut bench = Self {
            key,
            mass: KeyMass::Estimated,
            frame,
            protocol,
            kind: KeyKind::Natural,
            friction,
            friction_basis: Basis::Assumed,
            finger_weight_g: 0.0,
            targets,
            balance_papers: Vec::new(),
            front_papers: Vec::new(),
            state: None,
            lifts_off: false,
            reinitialized: false,
        };
        let (stacks, _) = bench.solve(targets)?;
        bench.lay(&stacks);
        // Then put the key on the frame so built, with nothing on it.
        bench.initialize();
        Ok(bench)
    }

    /// The default bench: a middle natural, regulated to Steinway New York.
    pub fn standard() -> Self {
        Self::new(
            KeyDesign::default(),
            FrameDesign::default(),
            Targets {
                key_height: 0.0650,
                key_dip: 0.0102,
            },
        )
        .expect("the standard bench must be regulable")
    }

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
        self.balance_papers = stacks.balance.papers.clone();
        self.front_papers = stacks.front.papers.clone();
        self.sync_stacks();
    }

    /// **Regulates the key to `targets`**: key height on the balance
    /// punching first, then the dip on the front punching on the key as it
    /// then stands, each laid in papers from the drawer, the closest total
    /// from below ([`pa_core::regulation`]). The papers replace what was on
    /// the punchings; the felts stay. Returns what the papers give, read by
    /// the seated pose, which the sheet shows beside the targets: within one
    /// paper of them.
    ///
    /// A target no stack reaches is refused, naming what keeps it out of
    /// reach and by how much, and nothing changes: the papers, the targets
    /// and the key stay as they were. A dip the front pin forbids is refused,
    /// never laid as the nearest stack and shown as a miss.
    pub fn regulate(&mut self, targets: Targets) -> Result<Achieved, NotRegulated> {
        let (stacks, achieved) = self.solve(targets)?;
        self.targets = targets;
        self.lay(&stacks);
        self.relax();
        Ok(achieved)
    }

    /// The targets of the regulation last laid, metres.
    pub fn targets(&self) -> Targets {
        self.targets
    }

    /// The key, as it stands.
    pub fn key(&self) -> &KeyDesign {
        &self.key
    }

    /// The frame, as it stands, the papers laid on its punchings as totals.
    pub fn frame(&self) -> &FrameDesign {
        &self.frame
    }

    /// The protocol the key is weighed by.
    pub fn protocol(&self) -> &TouchweightProtocol {
        &self.protocol
    }

    /// The measuring point, metres in front of the balance pin: the
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
    /// increment could be solved (`docs/FOUNDATIONS.md` §6.5).
    pub fn reinitialized(&self) -> bool {
        self.reinitialized
    }

    /// The initialization: the key put on its frame from its nominal pose
    /// under the weight on its measuring point, friction off
    /// (`docs/FOUNDATIONS.md` §6.5); where it finds no state from there, from
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
                        .and_then(|eq| Self::state_at(&k.mounted, eq.q))
                };
                if mounts::overlap_on_its_frame(&k.mounted, &start) > ROW_MARGIN {
                    self.lifts_off = true;
                    None
                } else {
                    match place(&start) {
                        Ok(eq) => Self::state_at(&k.mounted, eq.q).or_else(from_above),
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

    /// Applies the bench's last action to the key, as one static increment
    /// from where it stands with its friction on: a weight on the measuring
    /// point, a weighing, a paper, any edit of the key or of its frame
    /// (`docs/FOUNDATIONS.md` §6.5). The key stays where it was left if it
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
                Ok(eq) => Self::state_at(&k.mounted, eq.q).map(Ok),
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

    /// The state at `q`, or none where the key's drawn parts stand sunk in
    /// its frame's deeper than [`SEAT_OVERLAP`], where no contact reads them
    /// and the key is not on its frame at all.
    fn state_at(mounted: &Mounted, q: Vec<f64>) -> Option<State> {
        let pose = mounted.mechanism.pose(&q);
        if mounted.mechanism.overlap(&pose) > SEAT_OVERLAP {
            return None;
        }
        Some(State {
            placements: (0..mounted.mechanism.bodies().len())
                .map(|b| pose.placement(BodyId(b)))
                .collect(),
            lift: mounted.lift_at(&q),
            q,
        })
    }

    /// The weight on the measuring point, as a load on the key.
    fn touch_load(&self, mounted: &Mounted) -> PointLoad {
        PointLoad {
            at: MaterialPoint {
                body: mounted.key,
                at: self.protocol.measuring_point_on_key(&self.key),
            },
            force: Point::new(0.0, -self.finger_weight_g * 1e-3 * G),
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
            // Nose-down positive; a product, not a platform's maths library.
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
        let frame = self.frame.parts(
            &self.key,
            self.kind,
            &self.balance_papers,
            &self.front_papers,
            &mut parts,
        );
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
                    sources: part.sources,
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
    /// bears on its balance punching at the punching's centre, on the pin's
    /// axis: where the felt is drawn, which is where every key the bench
    /// builds bears.
    fn on_frame(&self) -> Result<OnItsFrame, Problem> {
        mounts::on_its_frame(
            MOUNT,
            &self.key,
            &self.frame,
            self.kind,
            &self.balance_papers,
            &self.front_papers,
            self.mass_props(),
            self.friction.bound_n_m,
        )
    }

    /// The key's mass and centre of mass as the statics take them
    /// ([`KeyMass::props`]): as it was weighed, the height of its centre of
    /// mass the volume model's, which no scale gives; until it is weighed,
    /// wholly the volume model's ([`composite_key`]). `None` where the volume
    /// model encloses nothing and so estimates nothing.
    fn mass_props(&self) -> Option<MassProps> {
        composite_key::mass(&self.key).map(|estimate| self.mass.props(estimate))
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
    fn seated(&self) -> Result<(OnItsFrame, Seats), SeatRefusal> {
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
    pub fn contacts(&self) -> Vec<ShownContact> {
        let Some(mounted) = self.key_on_frame() else {
            return Vec::new();
        };
        let rows = Self::rows_at(&mounted, &self.drawn(&mounted), ROW_MARGIN);
        let pairs = mounted.mechanism.pairs();
        rows.rows()
            .iter()
            .map(|r| ShownContact {
                code: pairs[usize::from(r.pair)].code,
                on_key: r.on_a(),
                on_frame: r.on_b(),
                gap: r.gap,
            })
            .collect()
    }

    /// How far each punching the key bears on, read at its centre, overlaps
    /// it where the key is drawn, metres: what its felt would compress under
    /// a key out of level on it, which the rigid reading at the centre
    /// leaves aside ([`edge_overlaps`]). Under the key at rest on its
    /// balance punching, nose-up by its dip, a quarter of a millimetre.
    pub fn edge_overlaps(&self) -> Vec<(ContactCode, f64)> {
        let Some(mounted) = self.key_on_frame() else {
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
    /// and its variant zeroed on the frame. The key's mass is its weighing
    /// ([`Self::weigh_key`]), or the composite key's derivation until it is
    /// weighed ([`composite_key`]).
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
            bases: self.bases(),
        }))
    }

    /// What the bench's inputs rest on: the key's mass and static moment as
    /// it was weighed, by the bench's user, or derived from its volume at an
    /// assumed density until it is; the height of its centre of mass from
    /// that volume either way, since no scale gives it; its friction the
    /// bench's assumption until it is set; its dimensions partly without a
    /// record (`data/params/README.md`).
    pub fn bases(&self) -> Bases {
        Bases {
            key_mass: match self.mass {
                KeyMass::Estimated => Basis::Estimated,
                KeyMass::Weighed { .. } => Basis::SetByHand,
            },
            centre_of_mass_height: Basis::Estimated,
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
        let props = self.mass_props()?;
        let (k, seats) = self.seated().ok()?;
        let finger = self.finger_verdict()?;
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
            key_mass_g: props.mass * 1000.0,
        })
    }

    /// The gaps at the front punching and at the back rail cloth where the
    /// key is drawn, millimetres, read off the contact rows at the attitude
    /// it stands in: `None` where no underside passes over the punching's
    /// centre, or where no feature of the cloth and the key can meet.
    fn gaps_mm(&self, mounted: &Mounted) -> (Option<f64>, Option<f64>) {
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
        let (gap_front_mm, gap_back_mm) = self
            .key_on_frame()
            .map_or((None, None), |mounted| self.gaps_mm(&mounted));
        Some(Touch {
            verdict,
            gap_front_mm,
            gap_back_mm,
        })
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

    // ---- the key's weighing ---------------------------------------------

    /// **Weighs the key whole**, as it arrives, its leads in it: `mass_g`
    /// grams on a scale, and a front weight of `front_weight_g` grams read on
    /// Stanwood's levelled jig with its roller `from_lip` metres back from the
    /// covering's lip ([`KeyMass::weighed`]).
    ///
    /// The bench keeps the mass and the static moment the front weight gives
    /// on the arm it was read at, with that reading as its datum: moving the
    /// measuring point afterwards, or editing the key, reads the same moment
    /// on another arm, and the front weight on the sheet follows by the lever.
    /// The height of the key's centre of mass, which no scale gives, stays the
    /// volume model's estimate. The key is solved again from where it stands,
    /// its weight changed.
    ///
    /// Refused, the bench left as it was, where the numbers are no key's,
    /// with why ([`NotAWeighing`]).
    pub fn weigh_key(
        &mut self,
        mass_g: f64,
        front_weight_g: f64,
        from_lip: f64,
    ) -> Result<(), NotAWeighing> {
        self.mass = KeyMass::weighed(&self.key, mass_g, front_weight_g, from_lip)?;
        self.relax();
        Ok(())
    }

    /// What is known of the key's mass: estimated, or as it was weighed,
    /// with the front weight reading it keeps.
    pub fn key_mass(&self) -> KeyMass {
        self.mass
    }

    /// The key's mass, grams, as the statics take it: as it was weighed, or
    /// until it is, the composite key's derived from its volume. `None`
    /// where the volume model encloses nothing.
    pub fn key_mass_g(&self) -> Option<f64> {
        self.mass_props().map(|props| props.mass * 1000.0)
    }

    // ---- regulation ------------------------------------------------------

    /// Lays one more paper on a stack. Key levelling, or key dip.
    pub fn add_paper(&mut self, which: Which, thickness: f64) {
        match which {
            Which::Balance => self.balance_papers.push(thickness.max(0.0)),
            Which::Front => self.front_papers.push(thickness.max(0.0)),
        }
        self.sync_stacks();
        self.relax();
    }

    /// Takes paper `i` back out of a stack.
    pub fn remove_paper(&mut self, which: Which, i: usize) {
        let stack = match which {
            Which::Balance => &mut self.balance_papers,
            Which::Front => &mut self.front_papers,
        };
        if i < stack.len() {
            stack.remove(i);
        }
        self.sync_stacks();
        self.relax();
    }

    /// The papers on a stack, metres, in the order they were laid.
    pub fn papers_on(&self, which: Which) -> &[f64] {
        match which {
            Which::Balance => &self.balance_papers,
            Which::Front => &self.front_papers,
        }
    }

    /// Pushes the two stack totals into the geometry.
    fn sync_stacks(&mut self) {
        self.frame.balance_punching.paper = self.balance_papers.iter().sum();
        self.frame.front_punching.paper = self.front_papers.iter().sum();
    }

    /// Sets the paper under the balance punching to a given total, metres.
    ///
    /// Kept for the tests and the command line, where a thickness is the
    /// natural way to express an experiment. The bench's own control is
    /// [`Self::add_paper`], because that is the operation.
    pub fn set_balance_paper(&mut self, paper: f64) {
        self.balance_papers = papers_for(paper);
        self.sync_stacks();
        self.relax();
    }

    /// Sets the paper on the front punching to a given total, metres.
    pub fn set_front_paper(&mut self, paper: f64) {
        self.front_papers = papers_for(paper);
        self.sync_stacks();
        self.relax();
    }

    /// The two paper stacks, metres.
    pub fn papers(&self) -> (f64, f64) {
        (
            self.frame.balance_punching.paper,
            self.frame.front_punching.paper,
        )
    }

    /// Puts a weight on the measuring point and lets the physics decide.
    ///
    /// Quasi-static, because there is no integrator yet: the weight is one
    /// static increment from where the key stands, friction on, and the key
    /// arrives where that increment puts it rather than travelling there.
    /// Reporting that honestly is better than animating a motion nothing has
    /// computed — and the *interesting* part is visible either way, which is
    /// that between the up weight and the down weight nothing happens at
    /// all: the key stays where it was, and its friction holds it.
    pub fn set_finger_weight(&mut self, grams: f64) {
        self.finger_weight_g = grams;
        self.relax();
    }

    /// Sets how far back from the front lip of the key every weight is
    /// centred, metres: the measuring point of every touchweight protocol.
    ///
    /// The lip is the covering's, which overhangs the key front. Every
    /// touchweight follows by the lever — front weight and the key weight
    /// ratio, D, U, BW and F, key friction weight — and nothing about the key
    /// changes: the same static moment, and the same friction torque, read on
    /// a different arm. Key height and key dip do not move; they are read at
    /// the key front.
    ///
    /// A weight already on the key moves with the point, so the key is
    /// solved again under it, from where it stands. Clamped to the covering, from its lip to its back end:
    /// in front of the lip there is no key to put a weight on, and behind the
    /// covering the point would no longer stand on the top the statics put it
    /// on. A value that is not a number is not a distance, and leaves the point
    /// where it was.
    pub fn set_measuring_point_from_lip(&mut self, from_lip: f64) {
        if !from_lip.is_finite() {
            return;
        }
        let covering = self.key.keytop_length.max(0.0);
        self.protocol.measuring_point_from_lip = from_lip.clamp(0.0, covering);
        self.relax();
    }

    /// How far back from the front lip of the key every weight is centred,
    /// metres.
    pub fn measuring_point_from_lip(&self) -> f64 {
        self.protocol.measuring_point_from_lip
    }

    /// The weight currently on the measuring point, grams.
    pub fn finger_weight(&self) -> f64 {
        self.finger_weight_g
    }

    /// What holds the key where the weight on its measuring point has left
    /// it: a stop, its friction, its two stops off its balance punching; or
    /// that the weight lifts it off its frame. `None` where nothing holds
    /// it for any other reason: a key that has not been put on its frame,
    /// with nothing under it.
    pub fn finger_verdict(&self) -> Option<TouchVerdict> {
        if self.lifts_off {
            return Some(TouchVerdict::LiftsOffItsFrame);
        }
        let mounted = self.key_on_frame()?;
        self.verdict(&mounted, self.state.as_ref()?)
    }

    // ---- the key's own dimensions ---------------------------------------
    //
    // The key weight ratio is deliberately **not** settable. It is the ratio of
    // two arms, so it is an output of the geometry exactly as the action ratio
    // is: you move the capstan, or you change where the key is measured
    // (`set_measuring_point_from_lip`), and the ratio follows. A control that
    // set it directly would have to silently choose which dimension to move,
    // which is why neither ratio is a setting.
    //
    // Changing a dimension does **not** re-solve the punchings. The regulation
    // drifts and the bench shows the miss, because that is what happens on a
    // real keyboard: lengthen the lever and the dip you set yesterday is wrong
    // today. Auto-correcting would hide the lesson.

    /// Moves the capstan along the key, metres from the balance pin.
    ///
    /// Negative: the capstan is behind the pin. Clamped to stay behind it,
    /// since a capstan in front of the balance pin is not a grand action.
    pub fn set_capstan(&mut self, from_balance: f64) {
        self.key.capstan = from_balance.min(-0.010);
        self.relax();
    }

    /// Moves the front edge of the key, metres in front of the balance pin.
    ///
    /// This is the key's length in front of its pivot. The measuring point
    /// moves with it, a set distance back from the covering's lip, so with
    /// the capstan and that distance it sets the key weight ratio.
    pub fn set_front_edge(&mut self, from_balance: f64) {
        self.key.front_edge = from_balance.max(0.050);
        self.relax();
    }

    /// Screws the capstan up or down, metres above the keystick.
    pub fn set_capstan_height(&mut self, height: f64) {
        self.key.capstan_screw.height = height.clamp(0.002, 0.030);
        self.relax();
    }

    /// Bolts the rails where this key needs them, and does nothing else.
    ///
    /// The deliberate act that ends a drift. Changing a key's length moves its
    /// mortise and its chamfer and leaves the rails alone — which is what
    /// happens on a keybed, and what the faults on the readout report. This is
    /// the other half: move the rails, which on a real keyboard means
    /// unscrewing them, and then regulate the keyboard you now have.
    ///
    /// It moves the rails and **nothing else**. The arm the key bears on at
    /// the back has just changed, so the key height and the dip will have
    /// drifted off their targets — and the sheet will show the miss, because
    /// shimming is a separate operation on a real bench too. Re-regulating
    /// here would roll two trade operations into one button and hide which of
    /// them the reader is looking at.
    pub fn place_the_frame_under_the_key(&mut self) {
        self.frame = self.frame.clone().placed_under(&self.key);
        self.relax();
    }

    /// Where the front rail stands, metres in front of the balance pin.
    pub fn set_front_rail(&mut self, from_balance: f64) {
        self.frame.front_rail = from_balance.max(0.010);
        self.relax();
    }

    /// Where the back rail stands, metres behind the balance pin (negative).
    pub fn set_back_rail(&mut self, from_balance: f64) {
        self.frame.back_rail = from_balance.min(-0.010);
        self.relax();
    }

    /// Where the two placed rails stand, metres: front rail, back rail.
    ///
    /// The back one is signed as it is stored — negative, behind the pin.
    pub fn rail_positions(&self) -> (f64, f64) {
        (self.frame.front_rail, self.frame.back_rail)
    }

    /// The three dimensions the bench exposes, in metres.
    pub fn key_dimensions(&self) -> (f64, f64, f64) {
        (
            self.key.capstan,
            self.key.front_edge,
            self.key.capstan_screw.height,
        )
    }

    // ---- the frame's own dimensions --------------------------------------

    /// Height of the bare balance rail above the keybed, metres.
    pub fn set_balance_rail_height(&mut self, h: f64) {
        self.frame.balance_rail_height = h.max(0.0);
        self.relax();
    }

    /// Height of the bare front rail above the keybed, metres.
    pub fn set_front_rail_height(&mut self, h: f64) {
        self.frame.front_rail_height = h.max(0.0);
        self.relax();
    }

    /// Height of the bare back rail above the keybed, metres.
    pub fn set_back_rail_height(&mut self, h: f64) {
        self.frame.back_rail_height = h.max(0.0);
        self.relax();
    }

    /// Thickness of the back rail cloth, metres.
    pub fn set_back_rail_cloth(&mut self, t: f64) {
        self.frame.back_rail_cloth = t.max(0.0);
        self.relax();
    }

    /// How far the front rail pin stands above the bare rail, metres.
    pub fn set_front_pin_height(&mut self, h: f64) {
        self.frame.sections.front.pin_height = h.max(0.0);
        self.relax();
    }

    /// Depth of the key's front mortise above its underside, metres.
    pub fn set_front_mortise_depth(&mut self, d: f64) {
        self.key.front_mortise_depth = d.max(0.0);
        self.relax();
    }

    /// The front rail pin's height above the rail and the mortise it enters,
    /// metres.
    pub fn front_pin_and_mortise(&self) -> (f64, f64) {
        (
            self.frame.sections.front.pin_height,
            self.key.front_mortise_depth,
        )
    }

    /// The four frame dimensions, metres: balance rail, front rail, back rail,
    /// back rail cloth.
    pub fn frame_dimensions(&self) -> (f64, f64, f64, f64) {
        let f = &self.frame;
        (
            f.balance_rail_height,
            f.front_rail_height,
            f.back_rail_height,
            f.back_rail_cloth,
        )
    }

    /// Thickness of the felt washer under the balance punching, metres.
    ///
    /// Fitting a different washer, as against shimming with paper. The rest
    /// geometry cares only about the total; the split matters because paper is
    /// how you adjust and felt is what you fit, and because the felt is the
    /// part that will compress once the contact law arrives.
    pub fn set_balance_felt(&mut self, felt: f64) {
        self.frame.balance_punching.felt = felt.max(0.0);
        self.relax();
    }

    /// Thickness of the felt washer on the front punching, metres.
    pub fn set_front_felt(&mut self, felt: f64) {
        self.frame.front_punching.felt = felt.max(0.0);
        self.relax();
    }

    /// Sets the bushings' friction from a key friction weight, grams at the
    /// measuring point, as the trade measures it: by Stanwood's protocol,
    /// zeroed on the levelled jig, the least weight that makes the key fall
    /// from its cloth.
    ///
    /// The bushings are the torque at which that protocol, run on this key,
    /// reads `kf_g` in full ([`Weighing::bushings_reading`]), so the sheet
    /// reads it back as typed, to the walk's resolution, and shows it as
    /// typed. Stored as that torque, with the measuring point it was typed at
    /// ([`AxisFriction`]): moving the measuring point or editing the key
    /// afterwards leaves the bushings alone, and changes the grams every
    /// protocol reads them as, as it would at the bench.
    ///
    /// Refused, the bushings left as they were, where no bushings read it:
    /// the protocol refused on this key or frame, a weight under what it
    /// reads with no friction at all or past the search, or not a number,
    /// with why ([`BushingsRefusal`]).
    pub fn set_key_friction_weight(&mut self, kf_g: f64) -> Result<(), BushingsRefusal> {
        if !kf_g.is_finite() {
            return Err(BushingsRefusal::NotANumber);
        }
        let found = self
            .weigh(|w| w.bushings_reading(&mut Statics::new(), kf_g))
            .map_err(BushingsRefusal::Procedure)??;
        self.friction = AxisFriction {
            bound_n_m: found.bound_n_m,
            radius_m: Some(self.measuring_point()),
        };
        self.friction_basis = Basis::SetByHand;
        self.relax();
        Ok(())
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

/// Setting the bushings straight, for the tests.
#[cfg(test)]
impl KeyBench {
    /// Bushings that hold `grams` at the measuring point with the key level,
    /// set as that torque, the way the bench sets its default: for the tests
    /// whose arithmetic reads the friction on the level arm, not through
    /// Stanwood's protocol.
    pub(crate) fn set_bushings_holding_g(&mut self, grams: f64) {
        self.friction = AxisFriction::of_weight_at(grams, self.measuring_point());
        self.friction_basis = Basis::SetByHand;
        self.relax();
    }
}

/// Reading the bench's sheet, for the tests.
#[cfg(test)]
pub(crate) mod read {
    use super::{KeyBench, Measurement, ProcedureId};
    use pa_core::measure::walk::{HALVINGS, STEP_G};
    use pa_core::solve::statics::RESIDUAL_TOLERANCE;
    use pa_core::G;

    /// A reading of `b`'s sheet, which must not be refused.
    pub(crate) fn measured(b: &KeyBench, p: ProcedureId) -> Measurement {
        match b.sheet().get(p) {
            Some(Ok(m)) => m.clone(),
            other => panic!("{p:?}: {other:?}"),
        }
    }

    /// Its value: grams, the ratio, or metres.
    pub(crate) fn value(b: &KeyBench, p: ProcedureId) -> f64 {
        measured(b, p).value
    }

    /// A height or a dip, millimetres.
    pub(crate) fn mm(b: &KeyBench, p: ProcedureId) -> f64 {
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

    /// The bench reports the regulation it was asked for.
    #[test]
    fn a_standard_bench_reports_what_it_achieved_beside_what_it_was_asked_for() {
        let b = KeyBench::standard();
        let r = b.readout().expect("the standard bench reads");
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
        let paper_mm = PAPER_THICKNESSES[0] * 1000.0;
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
        assert!((0.50..=0.60).contains(&ratio), "ratio {ratio}");
        assert!(
            r.key_mass_g > 40.0 && r.key_mass_g < 160.0,
            "mass {} g",
            r.key_mass_g
        );
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
    /// modelling dry friction as set-valued rather than as a `tanh`. A
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
        let mut b = KeyBench::standard();
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
            b.set_finger_weight(onto);
            assert_eq!(b.finger_verdict(), rest_on(stop), "{onto} g");
            let there = b.sin_angle();
            for w in [up + 1.0, middle, down - 1.0, near] {
                b.set_finger_weight(w);
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
            b.set_finger_weight(past);
            assert_ne!(
                b.finger_verdict(),
                rest_on(stop),
                "{past} g left the key there"
            );
        }
    }

    /// The initialization puts an unweighted key where its own weight takes
    /// it, friction off: the standard key, front-heavy by about 6.5 g, down on
    /// its front punching, and one weighed back-heavy on its cloth. The
    /// friction does not hold either in the air, because the initialization
    /// is solved without it.
    #[test]
    fn an_unweighted_key_starts_where_its_own_weight_puts_it() {
        let b = KeyBench::standard();
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            })
        );
        assert!(!b.reinitialized());
        let mut heavy = KeyBench::standard();
        heavy
            .weigh_key(151.1, -8.1, 0.013)
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
        let mut b = KeyBench::standard();
        b.set_key_friction_weight(2.0)
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

    /// Widening the friction widens the band, and leaves its centre where
    /// the seats' tilt lets it: D is read at the top seat and U at the
    /// bottom, where the measuring point stands a little nearer the pin's
    /// vertical at the top, so the extra friction reads a hair heavier there.
    /// The bound is derived from the seats' arms, with the walks' resolution.
    #[test]
    fn more_friction_makes_a_wider_band_around_the_same_centre() {
        let mut b = KeyBench::standard();
        let band = |b: &KeyBench| {
            [
                ProcedureId::DownWeight,
                ProcedureId::UpWeight,
                ProcedureId::BalanceWeight,
            ]
            .map(|p| read::value(b, p))
        };
        let [d0, u0, bw0] = band(&b);
        b.set_bushings_holding_g(12.0);
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
            "the centre moved by {} g, the seats' arms account for {tilt} g",
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
    /// the height of its centre of mass, which no scale gives. The weighing
    /// is kept as typed.
    #[test]
    fn a_key_weighed_whole_is_read_as_it_was_weighed() {
        let mut b = KeyBench::standard();
        b.weigh_key(121.5, 7.5, 0.013).expect("a key's weighing");
        let mass = b.readout().expect("the bench reads").key_mass_g;
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
                    Input::CentreOfMassHeight,
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
        b.set_measuring_point_from_lip(0.020);
        let arm_20 = b.measuring_point();
        let further = read::value(&b, ProcedureId::FrontWeight);
        assert!(
            (further * arm_20 - 7.5 * arm_13).abs() <= 2.0 * 16.0 * f64::EPSILON * 7.5 * arm_13,
            "7 mm further back the jig reads {further} g, not the same moment"
        );
    }

    /// Consistency: **a key weighed at what its volume model gives reads
    /// what the volume model reads.** Weighed at the readout's mass and the
    /// sheet's front weight in full, at the measuring point the sheet reads,
    /// its mass and its static moment are the volume model's to round-off and
    /// the height of its centre of mass is the same: the front weight and the
    /// key weight ratio read the same to round-off; key height and dip the
    /// same seats, closed by the static solve within `GAP_TOLERANCE`, seen at
    /// the key front through the lever from the cloth, under 2.3 times it,
    /// and from the front rail, 258 over 225 mm; and every weight walked
    /// within the two walks' resolutions.
    #[test]
    fn a_key_weighed_as_its_volume_model_gives_it_reads_the_same_sheet() {
        let estimated = KeyBench::standard();
        let mass_g = estimated.readout().expect("the bench reads").key_mass_g;
        let front_g = read::value(&estimated, ProcedureId::FrontWeight);
        let mut weighed = KeyBench::standard();
        weighed
            .weigh_key(mass_g, front_g, estimated.measuring_point_from_lip())
            .expect("the volume model's own numbers");
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
                "{p:?}: {was} by the volume model, {is} weighed at its numbers"
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
        let before = KeyBench::standard();
        let mass_g = before.readout().expect("the bench reads").key_mass_g;
        let front_g = read::value(&before, ProcedureId::FrontWeight);
        let mut after = before.clone();
        after
            .weigh_key(mass_g, front_g + 1.0, before.measuring_point_from_lip())
            .expect("a gram heavier in front is a key's weighing");
        let (k, seats) = before.seated().expect("the standard bench seats");
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
    /// centre of mass past the lip or past the tail, and a value that is not
    /// a number: each time the bench says why, by the refusal's code and its
    /// numbers, and the sheet, the readout and the key's mass stay as they
    /// were.
    #[test]
    fn a_weighing_no_key_has_is_refused_and_nothing_changes() {
        let everything =
            |b: &KeyBench| format!("{:?} {:?} {:?}", b.key_mass(), b.sheet(), b.readout());
        let mut b = KeyBench::standard();
        let before = everything(&b);
        // A weighing typed, grams, grams and metres, and whether a refusal
        // is the one it must be.
        type Case = ((f64, f64, f64), fn(&NotAWeighing) -> bool);
        let cases: [Case; 5] = [
            ((0.0, 6.5, 0.013), |w| *w == NotAWeighing::MassNotPositive),
            ((121.1, 6.5, 0.200), |w| {
                *w == NotAWeighing::RollerOffTheCovering { from_lip_m: 0.200 }
            }),
            (
                (40.0, 50.0, 0.013),
                |w| matches!(w, NotAWeighing::CentreOfMassOffTheKey { at_m } if *at_m > 0.0),
            ),
            (
                (121.1, -150.0, 0.013),
                |w| matches!(w, NotAWeighing::CentreOfMassOffTheKey { at_m } if *at_m < 0.0),
            ),
            ((f64::NAN, 6.5, 0.013), |w| *w == NotAWeighing::NotANumber),
        ];
        for ((mass_g, front_g, from_lip), says) in cases {
            let why = b
                .weigh_key(mass_g, front_g, from_lip)
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
        let mut b = KeyBench::standard();
        b.weigh_key(121.5, 7.5, 0.013).expect("a key's weighing");
        let moment = 7.5 * b.measuring_point();
        let (_, front_edge, _) = b.key_dimensions();
        b.set_front_edge(front_edge - 0.010);
        let shorter = read::value(&b, ProcedureId::FrontWeight) * b.measuring_point();
        assert!(
            (shorter - moment).abs() <= 2.0 * 16.0 * f64::EPSILON * moment,
            "{shorter} g·m read on the shorter arm, {moment} g·m weighed"
        );
    }

    /// A weight moves the key between its two seats and no further: a heavy
    /// one presses it down onto the bottom seat, whatever its size, because
    /// past a stop a weight only loads the stop, and a lift raises it onto
    /// the top seat, its balance punching still carrying it.
    ///
    /// The bound is derived. The touch weight and the seat's preparation
    /// loads put the key on the same rigid stop, and each static solve
    /// closes it within `GAP_TOLERANCE` (a picometre), which an arm of at
    /// least 0.2 m from the bearing to the stop turns into an angle.
    #[test]
    fn a_weight_takes_the_key_to_its_seats_and_no_further() {
        let mut b = KeyBench::standard();
        let (k, seats) = b.seated().expect("the standard bench seats");
        let angle = k.mounted.angle.0;
        let bound = 2.0 * 1e-12 / 0.2;
        for (weights, seat) in [
            (&[60.0, 400.0][..], &seats.bottom),
            (&[-30.0, -60.0][..], &seats.top),
        ] {
            for &w in weights {
                b.set_finger_weight(w);
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
        let mut b = KeyBench::standard();
        let (k, seats) = b.seated().expect("the standard bench seats");
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
        b.set_finger_weight(-held_to);
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
            b.set_finger_weight(-past);
            assert_eq!(
                b.finger_verdict(),
                Some(TouchVerdict::LiftsOffItsFrame),
                "-{past} g"
            );
            assert!(!b.reinitialized(), "the key stays where it was");
            let r = b.readout().expect("the sheet still reads");
            assert_eq!(r.finger, TouchVerdict::LiftsOffItsFrame);
            let still = &b.state.as_ref().expect("the last equilibrium").q;
            assert_eq!(
                still.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                q.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                "drawn where the last equilibrium left it"
            );
        }
        b.set_finger_weight(0.0);
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
    /// punching on the standard bench. Pulled up at its measuring point past
    /// what its punching carries, the key turns up on its cloth off its
    /// punching as far as its friction lets it: under a millimetre at
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
        let base = KeyBench::standard();
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
            b.set_finger_weight(-grams);
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
        b.set_finger_weight(-63.0);
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
    }

    /// Consistency: **a key its friction holds off its punching stays
    /// there.** Pulled 62.7 g at its measuring point, a few hundredths of a
    /// gram past what its punching carries, the key turns up on its cloth
    /// and leaves its punching by under a millimetre, where its friction
    /// holds it. Pulled 60.7 g, inside the band its friction holds it in,
    /// it stays where it was to the bit, still lifted, on its cloth: the
    /// increment starts from where the last one left the key's slide, never
    /// from the key put back on its punching, which would carry it down.
    #[test]
    fn a_key_its_friction_holds_off_its_punching_stays_there() {
        let mut b = KeyBench::standard();
        b.set_finger_weight(-62.7);
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
        b.set_finger_weight(-60.7);
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
        let mut b = KeyBench::standard();
        b.set_finger_weight(-60.0);
        let (at_rest, rest_poses) = (b.outlines(), b.poses());
        let rest_pt = b.key_placement().world(b.measuring_point_on_key());
        b.set_finger_weight(60.0);
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
        let mut b = KeyBench::standard();
        b.set_finger_weight(-60.0);
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
        let mut b = KeyBench::standard();
        for from_lip in [0.013, 0.0127, 0.030] {
            b.set_measuring_point_from_lip(from_lip);
            let protocol = *b.protocol();
            let weighed = protocol.measuring_point_on_key(b.key());
            let marked = b.measuring_point_on_key();
            assert_eq!(marked.x.to_bits(), weighed.x.to_bits());
            assert_eq!(marked.y.to_bits(), weighed.y.to_bits());
        }
    }

    /// Adding paper under the balance punching raises the key, as on the bench.
    #[test]
    fn key_levelling_works_through_the_bench() {
        let mut b = KeyBench::standard();
        let before = read::mm(&b, ProcedureId::KeyHeight);
        let (paper, _) = b.papers();
        b.set_balance_paper(paper + 0.0005);
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
        let mut b = KeyBench::standard();
        let before = b.readout().unwrap();
        let dip_before = read::mm(&b, ProcedureId::KeyDip);
        let (_, paper) = b.papers();

        let shim = 0.0005;
        b.set_front_paper(paper + shim);
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
        let b = KeyBench::standard();
        let f = &b.frame;
        let out = (f.front_bearing_height() - f.balance_bearing_height()) * 1000.0;
        let two_papers = 2.0 * PAPER_THICKNESSES[0] * 1000.0;
        assert!(
            out.abs() < two_papers,
            "the key is out of level by {out:.3} mm at the bottom of its travel: \
             the front punching tops out at {:.3} mm and the balance at {:.3}",
            f.front_bearing_height() * 1000.0,
            f.balance_bearing_height() * 1000.0
        );

        // And released, it is nose-up rather than nose-down: the tail is on
        // the cloth and the front hangs clear of its punching by the dip.
        let r = b.readout().expect("the standard bench reads");
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

    /// Consistency: **a front pin a few tenths of a millimetre into the wood
    /// beside its slot draws the key on the pin, never through it.** Its
    /// front end 0.3 mm into the wood in front of the slot, its top well up
    /// in the slot: the bench puts the key on such a frame as the seats do,
    /// lowered from above where its nominal pose finds no state, and the key
    /// rides on the pin and its cloth, nothing drawn sunk, the frame seated.
    #[test]
    fn a_pin_a_few_tenths_into_the_wood_draws_the_key_on_it() {
        let mut b = KeyBench::standard();
        let pin = b.frame().sections.front.pin.expect("a pin").length();
        let (_, front_end) = b.key().front_mortise_span();
        b.set_front_rail(front_end - 0.5 * pin + 0.0003);
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
        let mut b = KeyBench::standard();
        b.set_back_rail_height(0.055);
        assert_eq!(b.finger_verdict(), Some(TouchVerdict::LiftsOffItsFrame));
        assert!(b.riding().is_none() && b.frame_fault().is_some());

        let mut b = KeyBench::standard();
        b.frame.sections.balance.pin_height += 0.020;
        b.relax();
        assert!(!b.reinitialized(), "a taller pin moves nothing");
        b.set_back_rail_height(0.055);
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
        let wedged = |b: &mut KeyBench| {
            let level = b.frame().balance_bearing_height();
            let cloth = b.frame().back_rail_cloth;
            b.set_back_rail_height(level + 0.005 - cloth);
        };
        let short = |b: &mut KeyBench| {
            let (_, front_edge, _) = b.key_dimensions();
            b.set_front_edge(front_edge - 0.017);
            let h = b.frame().back_rail_height;
            b.set_back_rail_height(h - 0.003);
        };
        for edit in [&wedged as &dyn Fn(&mut KeyBench), &short] {
            let mut b = KeyBench::standard();
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
        let mut b = KeyBench::standard();
        b.set_front_rail_height(0.0417);
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
        let mut b = KeyBench::standard();
        b.set_key_friction_weight(3.0).expect("bushings read 3 g");
        let stood = b.key_placement();
        b.set_finger_weight(-62.0);
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
        let mut b = KeyBench::standard();
        let (_, front_edge, _) = b.key_dimensions();
        b.set_front_edge(front_edge + 0.0023725);
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
    /// the millimetre the view marks below the key, not at all; the key is
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
        let mut b = KeyBench::standard();
        let level = b.frame().balance_bearing_height();
        let cloth = b.frame().back_rail_cloth;
        b.set_back_rail_height(level + 0.005 - cloth);
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
        // edge's radius times the turn's square above it, tens of nanometres.
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
            let key = KeyDesign::default();
            let frame = FrameDesign {
                back_rail,
                ..FrameDesign::default()
            };
            let target = Targets {
                key_height: 0.0650,
                key_dip: 0.0102,
            };
            let Ok(b) = KeyBench::new(key, frame, target) else {
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
    /// picometre, `1e-9` mm.
    #[test]
    fn the_reported_gaps_agree_with_where_the_key_is_held() {
        let mut b = KeyBench::standard();
        let mut held = Vec::new();
        for weight in [-60.0, 0.0, -6.5, 60.0, -6.5] {
            b.set_finger_weight(weight);
            let r = b.readout().unwrap();
            let front = r.gap_front_mm.expect("the punching stands under the flat");
            let back = r.gap_back_mm.expect("the cloth meets the underside");
            let on = match r.finger {
                TouchVerdict::RestingOn { stop } => Some(stop),
                TouchVerdict::HeldByFriction { touching, .. } => touching,
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
            .any(|v| matches!(v, TouchVerdict::RestingOn { .. })));
        assert!(held
            .iter()
            .any(|v| matches!(v, TouchVerdict::HeldByFriction { .. })));
    }

    /// Consistency, and the check of `CLAUDE.md` rule 1: **every contact
    /// surface is a range of a drawn part.**
    ///
    /// By type, a contact pair holds nothing but ranges of parts. Here, on
    /// the bench and on every bench of the golden sheet: the mechanism the
    /// contacts are read on carries exactly the parts the view draws, and
    /// every range of every pair names one of them and segments its drawn
    /// silhouette has.
    #[test]
    fn every_contact_surface_is_a_range_of_a_drawn_part() {
        for fixture in &crate::golden::FIXTURES {
            let b = (fixture.build)();
            let (drawn, _, _) = b.parts();
            let mounted = b.key_on_frame().expect("the key mounts on its frame");
            assert_eq!(
                mounted.mechanism.parts(),
                &drawn[..],
                "{}: the contacts read the parts the view draws",
                fixture.name
            );
            assert_eq!(mounted.mechanism.pairs().len(), 5, "{}", fixture.name);
            for pair in mounted.mechanism.pairs() {
                for range in pair.a.ranges.iter().chain(&pair.b.ranges) {
                    let part = &drawn[range.part.index()];
                    assert!(
                        range.segs.start < range.segs.end
                            && usize::from(range.segs.end) <= part.silhouette().len(),
                        "{}: {:?} reads segments {:?} of {:?}",
                        fixture.name,
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
    /// of the cloth within a millimetre (the cloth's front edge is no contact
    /// of its own, its nearest point to the chamfer being where it meets the
    /// face, and the face falling away from it towards the back edge); down
    /// on its punching, the punching closed at its centre; and the balance
    /// punching closed at its centre all along, the key bearing on it. A
    /// closed contact's two points coincide.
    ///
    /// The one other row within a millimetre at rest is the front pin's,
    /// open, beside the front wall of its mortise: the slot moves back along
    /// the pin by the height of the pin's top corner above the bearing times
    /// the key's turn, 0.04 rad at rest, which brings the wall to within
    /// about 0.8 mm of the corner, from the 1.5 mm it stands off with the key
    /// level.
    #[test]
    fn the_contacts_the_view_marks_are_where_the_key_touches() {
        let mut b = KeyBench::standard();
        let closed = |b: &KeyBench| -> Vec<ContactCode> {
            b.contacts()
                .iter()
                .filter(|c| c.state() == ContactState::Closed)
                .map(|c| c.code)
                .collect()
        };
        b.set_finger_weight(-60.0);
        assert_eq!(
            closed(&b),
            vec![ContactCode::BackRailCloth, ContactCode::BalanceRailPunching]
        );
        let at_rest = b.contacts();
        let section = b.frame().sections.back;
        let back_edge = section.centre_x(b.frame().back_rail) - section.half_length;
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
                    "at the punching's centre: {c:?}"
                );
                continue;
            }
            assert_eq!(
                c.code,
                ContactCode::BackRailCloth,
                "nothing else within a millimetre"
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

        b.set_finger_weight(60.0);
        assert_eq!(
            closed(&b),
            vec![
                ContactCode::FrontRailPunching,
                ContactCode::BalanceRailPunching
            ]
        );
        let f = b.frame();
        let centre = Point::new(
            f.sections.front.pin_x(f.front_rail, KeyKind::Natural),
            f.front_bearing_height(),
        );
        for c in b.contacts() {
            if c.state() == ContactState::Closed && c.code == ContactCode::FrontRailPunching {
                assert!(
                    (c.on_frame.x - centre.x).abs() < 1e-12,
                    "at the punching's centre"
                );
                assert!((c.on_frame.y - centre.y).abs() < 1e-12);
            }
        }
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
        let mut b = KeyBench::standard();
        let stanwood =
            |b: &KeyBench| read::measured(b, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
        for kf in [2.0, 6.0, 13.0] {
            b.set_key_friction_weight(kf)
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
        b.set_measuring_point_from_lip(b.measuring_point_from_lip() + 0.007);
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

        b.weigh_key(131.1, 12.5, 0.013)
            .expect("a heavier key is weighed");
        b.set_capstan(-0.125);
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
        let everything = |b: &KeyBench| {
            format!(
                "{:?} {:?} {:?} {:?}",
                b.friction,
                b.friction_basis,
                b.sheet(),
                b.readout()
            )
        };
        let mut b = KeyBench::standard();
        let before = everything(&b);
        let refused = b
            .set_key_friction_weight(0.0)
            .expect_err("no bushings read 0 g on this key");
        let BushingsRefusal::UnderNoFriction { least_g } = refused else {
            panic!("{refused:?}");
        };
        assert!(least_g > 0.2 && least_g < 0.5, "{least_g}");
        assert_eq!(everything(&b), before, "a refusal changed the bench");
        assert_eq!(
            b.set_key_friction_weight(f64::NAN),
            Err(BushingsRefusal::NotANumber)
        );
        assert_eq!(everything(&b), before, "a refusal changed the bench");

        let level = b.frame().balance_bearing_height();
        let cloth = b.frame().back_rail_cloth;
        b.set_back_rail_height(level + 0.005 - cloth);
        let before = everything(&b);
        let refused = b
            .set_key_friction_weight(6.0)
            .expect_err("on a frame whose stops fight no key friction weight is read");
        assert_eq!(
            refused,
            BushingsRefusal::Procedure(Refused::Walk(WalkRefusal::NoTravel))
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
        let b = KeyBench::standard();
        for which in [Which::Balance, Which::Front] {
            let stack = b.papers_on(which);
            assert!(!stack.is_empty(), "{which:?} stack is empty");
            for &p in stack {
                assert!(
                    PAPER_THICKNESSES.iter().any(|t| (t - p).abs() < 1e-12),
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
        let mut b = KeyBench::standard();
        let before = b.readout().unwrap();
        let height_before = read::mm(&b, ProcedureId::KeyHeight);
        let n = b.papers_on(Which::Balance).len();

        b.add_paper(Which::Balance, 0.0002);
        assert_eq!(b.papers_on(Which::Balance).len(), n + 1);

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
        let expected = 0.2 * (arm + KeyDesign::default().front_edge * 1000.0) / arm;
        assert!(
            (risen - expected).abs() < 0.02,
            "a two-tenths raised the key by {risen} mm, expected about {expected}"
        );
    }

    /// And taking it back out puts the key back exactly where it was.
    #[test]
    fn taking_the_paper_back_out_is_exactly_reversible() {
        let mut b = KeyBench::standard();
        let before = read::mm(&b, ProcedureId::KeyDip);

        b.add_paper(Which::Front, 0.0004);
        let shimmed = read::mm(&b, ProcedureId::KeyDip);
        assert!(shimmed < before, "paper must shallow the dip");

        let last = b.papers_on(Which::Front).len() - 1;
        b.remove_paper(Which::Front, last);
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
        let stack = papers_for(wanted);
        let total: f64 = stack.iter().sum();

        assert!(total <= wanted + 1e-12, "a stack must never overshoot");
        assert!(
            wanted - total < PAPER_THICKNESSES[0],
            "and must get within the thinnest paper there is"
        );
        // Which is why the bench reports the achieved dip beside the target:
        // the miss is physical, not numerical.
        let b = KeyBench::standard();
        let r = b.readout().unwrap();
        assert!((read::mm(&b, ProcedureId::KeyDip) - r.key_dip_target_mm).abs() < 0.1);
    }
}

#[cfg(test)]
mod regulation_tests {
    use super::*;

    /// Everything the sheet, the readout and the papers say, to the bit.
    fn everything(b: &KeyBench) -> String {
        format!(
            "{:?} {:?} {:?} {:?} {:?}",
            b.sheet(),
            b.readout(),
            b.papers_on(Which::Balance),
            b.papers_on(Which::Front),
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
        let mut b = KeyBench::standard();
        let targets = Targets {
            key_height: 0.0660,
            key_dip: 0.0100,
        };
        let achieved = b.regulate(targets).expect("66 mm and 10 mm are reachable");
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
        for which in [Which::Balance, Which::Front] {
            assert!(b
                .papers_on(which)
                .iter()
                .all(|p| PAPER_THICKNESSES.contains(p)));
        }
        assert!(!b.reinitialized(), "the key follows its new stacks");
    }

    /// Consistency, and what the owner sees: **a dip the front pin forbids is
    /// refused, with its shortfall in millimetres, and nothing changes.** On
    /// the bench as built, 12 mm of dip is deeper than the front rail pin
    /// lets the key go before it bottoms in its mortise, by 0.79 mm: refused,
    /// naming the pin, and the papers, the targets, the key and every reading
    /// stay as they were.
    #[test]
    fn a_dip_the_front_pin_forbids_is_refused_and_nothing_changes() {
        let mut b = KeyBench::standard();
        let before = everything(&b);
        let targets = Targets {
            key_height: 0.0650,
            key_dip: 0.012,
        };
        let refused = b.regulate(targets).expect_err("the pin forbids 12 mm");
        let NotRegulated::Infeasible {
            target,
            limited_by,
            shortfall_m,
        } = refused
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
        let mut b = KeyBench::standard();
        let before = everything(&b);
        let refused = b
            .regulate(Targets {
                key_height: f64::NAN,
                key_dip: 0.0102,
            })
            .expect_err("not a number");
        assert_eq!(
            refused,
            NotRegulated::Unsolvable(Unsolvable::NotFinite(Target::KeyHeight))
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
            let mut b = KeyBench::standard();
            b.set_key_friction_weight(kf)
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
    /// was, to the walk's resolution. Stanwood's, zeroed on the levelled jig,
    /// keeps what the tilt does to the key and the temporary weight, a few
    /// tenths of a gram here, under the gram it is read to.
    #[test]
    fn weighing_the_key_heavier_does_not_change_its_measured_friction() {
        let mut b = KeyBench::standard();
        b.set_key_friction_weight(7.0)
            .expect("Stanwood's protocol reads 7 g on this key");
        let bare = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheFrame);
        let stanwood = read::value(&b, ProcedureId::KeyFrictionWeightZeroedOnTheJig);
        let mass_g = b.readout().expect("the bench reads").key_mass_g;
        let front_g = read::value(&b, ProcedureId::FrontWeight);

        b.weigh_key(mass_g + 30.0, front_g + 17.0, b.measuring_point_from_lip())
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
        let mut b = KeyBench::standard();
        b.set_bushings_holding_g(0.0);
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

    /// Consistency: **the procedures never read the session's state.** The
    /// whole sheet, D and U and the key friction weights with key height and
    /// dip, and the seats' bearing at the back and out of level at the
    /// bottom, are the same bits whatever the weight on the measuring point
    /// has done to the key: each procedure seats the key by rule, friction
    /// off, from its nominal pose. Friction moves the weighings and leaves
    /// the seats as they were.
    #[test]
    fn the_sheet_reads_the_seats_whatever_the_touch_weight_did() {
        let mut b = KeyBench::standard();
        let seated = |b: &KeyBench| {
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
            b.set_finger_weight(weight);
            assert_eq!(b.sheet(), sheet, "{weight} g");
            assert_eq!(seated(&b), before, "{weight} g");
        }
        b.set_key_friction_weight(18.0)
            .expect("Stanwood's protocol reads 18 g on this key");
        assert_eq!(seated(&b), before, "18 g of friction");
    }

    /// An edit the key can follow moves it from where it stood: a paper laid
    /// on the front punching under a key down on it lifts the key by what
    /// the paper reaches, an overlap inside the row margin, and the key
    /// stays on its punching.
    #[test]
    fn an_edit_the_key_can_follow_moves_it_from_where_it_stood() {
        let mut b = KeyBench::standard();
        b.set_finger_weight(60.0);
        let pressed = b.key_placement().world(b.measuring_point_on_key());
        b.add_paper(Which::Front, 0.0002);
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
        let mut b = KeyBench::standard();
        b.set_finger_weight(60.0);
        let (_, front, _, _) = b.frame_dimensions();
        b.set_front_rail_height(front + 0.003);
        assert!(b.reinitialized(), "sunk 3 mm, the key must be put back");
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            })
        );
        b.set_finger_weight(61.0);
        assert!(
            !b.reinitialized(),
            "and the next action is an increment again"
        );
    }

    /// Consistency: **a weight on a frame nobody touched is solved as one
    /// increment from where the key stands, every time.** Lifted and pressed
    /// in turn, the key travels from stop to stop with its friction acting,
    /// and the last step of a travel may land it a few nanometres into the
    /// stop: the box problem accepts its multipliers to a fraction of their
    /// scale, which a soft metric turns into nanometres of gap. The increment
    /// must close that overlap, not stop on it until its cap, after which the
    /// bench would start the key again by its loading rule, friction off.
    /// These weights on these two benches reach such a landing, so a solver
    /// that stops on the overlap fails here.
    #[test]
    fn a_weight_on_an_untouched_frame_is_always_one_increment() {
        let weights = [
            -40.0, 23.0, -40.0, 22.0, -40.0, 30.0, -60.0, 60.0, -60.0, 10.0, -60.0, 1.0,
        ];
        let mut stiff = KeyBench::standard();
        stiff.set_bushings_holding_g(15.0);
        let mut moved = KeyBench::standard();
        let (front, _) = moved.rail_positions();
        let (balance, front_height, _, _) = moved.frame_dimensions();
        moved.set_front_rail(front - 0.0008);
        moved.set_balance_rail_height(balance - 0.0015);
        moved.set_front_rail_height(front_height + 0.0003);
        for (name, b) in [
            ("15 g of friction", &mut stiff),
            ("rails moved", &mut moved),
        ] {
            for w in weights {
                b.set_finger_weight(w);
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
            let mut b = KeyBench::standard();
            b.set_finger_weight(-20.0);
            b.weigh_key(131.1, 12.5, 0.013)
                .expect("a heavier key is weighed");
            b.add_paper(Which::Balance, 0.0002);
            b.set_finger_weight(-4.0);
            b.set_key_friction_weight(9.0)
                .expect("Stanwood's protocol reads 9 g on this key");
            b.set_finger_weight(30.0);
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
    use pa_core::measure::MEASURING_POINT_HALF_INCH;

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
        let mut b = KeyBench::standard();
        let key = b.key().clone();
        let lip = key.front_edge + key.keytop_overhang;
        let moment_g_m = 1000.0 * key.mass_properties().unwrap().static_moment_x();

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
            "the standard bench is set to {} g of key friction, not the 6 g it was built with",
            before.key_friction_weight_g
        );
        let fw_before = read::value(&b, ProcedureId::FrontWeight);
        let (height_before, dip_before) = (
            read::mm(&b, ProcedureId::KeyHeight),
            read::mm(&b, ProcedureId::KeyDip),
        );
        let arm_before = lip - b.measuring_point_from_lip();
        b.set_measuring_point_from_lip(b.measuring_point_from_lip() + 0.007);
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
        let mut b = KeyBench::standard();
        let key = b.key().clone();
        let lip = key.front_edge + key.keytop_overhang;
        let height = key.stick_height + key.keytop_thickness;

        let kf_g = 6.0;
        b.set_bushings_holding_g(kf_g);
        let torque = kf_g * 1e-3 * (lip - b.measuring_point_from_lip());

        b.set_measuring_point_from_lip(MEASURING_POINT_HALF_INCH);
        let arm = lip - b.measuring_point_from_lip();
        assert!((b.measuring_point_from_lip() - MEASURING_POINT_HALF_INCH).abs() < 1e-15);
        assert!((b.measuring_point() - arm).abs() < 1e-15);

        let m = key.mass_properties().unwrap().static_moment_x();
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
            let mut b = KeyBench::standard();
            b.set_finger_weight(weight);
            let s = b.sin_angle();
            let before = b.key_placement().world(b.measuring_point_on_key());

            let back = 0.007;
            b.set_measuring_point_from_lip(b.measuring_point_from_lip() + back);
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

    /// The measuring point stays on the covering.
    ///
    /// In front of the lip there is no key to put a weight on; behind the
    /// covering the point would no longer stand on the top the statics put it
    /// on. Both are clamped, not refused: a finger cannot put a weight in the
    /// air either.
    #[test]
    fn the_measuring_point_stays_on_the_covering() {
        let mut b = KeyBench::standard();
        b.set_measuring_point_from_lip(-0.005);
        assert_eq!(b.measuring_point_from_lip(), 0.0);
        b.set_measuring_point_from_lip(0.500);
        assert_eq!(b.measuring_point_from_lip(), b.key().keytop_length);
        // And a value that is not a distance changes nothing.
        b.set_measuring_point_from_lip(f64::NAN);
        assert_eq!(b.measuring_point_from_lip(), b.key().keytop_length);
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
        let mut b = KeyBench::standard();
        b.weigh_key(151.1, -8.1, 0.013)
            .expect("a back-heavy key is weighed");
        b.set_bushings_holding_g(2.0);
        b.set_measuring_point_from_lip(0.030);
        b.set_finger_weight(10.6);
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

        b.set_measuring_point_from_lip(0.005);
        assert_eq!(
            b.finger_verdict(),
            Some(TouchVerdict::RestingOn {
                stop: ContactCode::FrontRailPunching
            }),
            "the key must have gone down to its punching"
        );
    }
}
