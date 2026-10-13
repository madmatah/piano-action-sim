//! The single-key weighing bench, as a use case.
//!
//! This is the application layer's first citizen: it owns a key, a frame and a
//! regulation, answers the questions a technician asks of them, and accepts the
//! operations he performs. It knows nothing about JavaScript, about rendering or
//! about files — `pa-wasm` adapts it for the browser and `pa-cli` for the shell,
//! and neither of them is allowed to add physics.
//!
//! # Why the leading workshop lives here and not in a panel
//!
//! Placing, moving and removing a lead are **operations on the model**, not
//! widget state. Putting them here means the same three calls serve the browser,
//! the command line and the test suite, and that an exercise can be scripted and
//! replayed. A lead dragged in a panel that only updated a React store would be
//! an animation of leading, not leading.

use pa_core::contact::{Bearing as ContactBearing, Part, Rows, ROW_MARGIN};
pub use pa_core::contact::{ContactCode, HowItBears, Material, PartCode, Provenance, Role, Sweep};
use pa_core::geometry::{KeyBearing, Point};
use pa_core::measure::jig::KeyJig;
use pa_core::measure::{Branches, KeyOnFrame, KeyPinFriction, TouchweightProtocol};
use pa_core::mechanism::{BodyId, Placement, Pose};
use pa_core::parts::assembly::{nose_up_angle, Settled};
pub use pa_core::parts::faults::Fault;
/// The punching thicknesses a drawer holds, re-exported so an adapter can
/// offer the drawer without reaching past the application layer.
pub use pa_core::parts::frame::PAPER_THICKNESSES;
use pa_core::parts::frame::{papers_for, KeyKind};
use pa_core::parts::mounts::{self, Mounted};
use pa_core::parts::{
    FrameDesign, FrameRanges, KeyAssembly, KeyDesign, KeyRanges, Lead, PunchingStack, Regulation,
};

use crate::composite_key;

/// Everything the bench can report, in the trade's own units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchReadout {
    /// Front weight, grams, signed: the key run on Stanwood's jig, level, its
    /// scale under the measuring point. Negative when the key is back-heavy.
    pub front_weight_g: f64,
    /// Key weight ratio, dimensionless, force convention: the tared jig read
    /// with 10 g on the capstan's contact point.
    pub key_ratio: f64,
    /// Key height at the front edge, millimetres above the keybed.
    pub key_height_mm: f64,
    /// Key dip at the front edge, millimetres.
    pub key_dip_mm: f64,
    /// Down weight at the measuring point, grams.
    pub down_weight_g: f64,
    /// Up weight at the measuring point, grams.
    pub up_weight_g: f64,
    /// Balance weight, grams.
    pub balance_weight_g: f64,
    /// Friction weight, grams.
    pub friction_weight_g: f64,
    /// Total mass of the key with its leads, grams.
    pub key_mass_g: f64,
    /// Number of leads fitted.
    pub lead_count: usize,
    /// Gap at the front rail punching right now, millimetres: the felt's
    /// drawn top, at its centre, to the key's underside. Zero when the key
    /// is down on it; `None` when the punching stands where no underside
    /// passes over its centre.
    pub gap_front_mm: Option<f64>,
    /// Gap at the back rail cloth right now, millimetres: the least of the
    /// cloth's drawn top to the key's drawn underside, chamfer included.
    /// `None` when no feature of the two can meet.
    pub gap_back_mm: Option<f64>,
    /// Key dip as **pressing the key measures it**, millimetres: the distance
    /// the front of the key moves between the two states the contact solve
    /// puts it in.
    ///
    /// On a stop, `settle` returns the stop angle [`KeyAssembly::travel`]
    /// uses, so whenever this can be measured it is the same number as
    /// [`Self::key_dip_mm`], bit for bit. Shown beside it, it is the same
    /// computation twice, not a cross-check.
    pub measured_dip_mm: Option<f64>,
    /// Key friction weight as the **protocol measures it**, grams.
    ///
    /// The result of running Stanwood's procedure on the model — front weight
    /// zeroed with a temporary weight, key on its frame, least weight at the
    /// measuring point that makes it fall — rather than the field read back.
    /// With a constant friction bound the procedure returns that bound through
    /// the measuring arm at the key's resting attitude, so the two agree by
    /// construction to within that arm's tilt: side by side they check the
    /// procedure, not the friction model.
    pub measured_kf_g: Option<f64>,
    /// Half-width of the friction band as a **torque about the bearing**,
    /// referred to the measuring point in grams.
    ///
    /// Reported beside `friction_weight_g` because they are the same thing seen
    /// two ways, and because the band is what friction *is*: the admissible
    /// torque is an interval `[−b, +b]`, not a value. A model with a single
    /// friction force would have no band and the key would have one threshold
    /// instead of two.
    pub friction_band_g: f64,
    /// Key height the regulation was asked for, millimetres.
    pub key_height_target_mm: f64,
    /// Key dip the regulation was asked for, millimetres.
    pub key_dip_target_mm: f64,
    /// What a weight at the measuring point is doing to the key.
    pub finger: FingerVerdict,
    /// How far from level the key stands at the **bottom** of its travel,
    /// degrees, nose-down positive.
    ///
    /// A keyboard is set up so this is zero: level when the key is fully down,
    /// and so nose-up by the dip when released. A design goal rather than a
    /// law — rail heights, punchings and cloth are what reach it, and plenty
    /// of keyboards miss — so the bench reports it instead of enforcing it.
    pub out_of_level_deg: f64,
    /// How far the key's bearing face is from lying flat on its cloth at rest,
    /// degrees.
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
    /// reported because it is the lever arm every number on this sheet is
    /// computed on, and it is not where a reader would assume it is.
    pub bears_at_the_back_mm: f64,
    /// Which of the three ways the key is bearing on its cloth.
    pub back_bearing: HowItBears,
    /// Whether it is the pin bottoming in its mortise, rather than the
    /// punching, that is stopping the key.
    pub stopped_by_the_mortise: bool,
}

/// What is carrying the key, re-exported so callers need not reach into the
/// domain for it.
pub use pa_core::parts::assembly::HeldBy as FingerVerdict;

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

/// The key's body in the bench's scene. The ground, which carries the frame,
/// is body 0.
pub const KEY: BodyId = BodyId(1);

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
    assembly: KeyAssembly,
    /// Whether this key is a natural or a sharp — which row of pins it stands
    /// on, and later which graduation band it belongs to.
    kind: KeyKind,
    friction: KeyPinFriction,
    /// Where the key is, as a fraction of its dip: 0 at rest, 1 at the bottom.
    depression: f64,
    /// The weight sitting on the measuring point, grams.
    finger_weight_g: f64,
    /// The regulation the bench was asked for, kept beside what it achieved.
    target: Regulation,
    /// The papers actually stacked under the balance punching, in the order
    /// they were laid.
    balance_papers: Vec<f64>,
    /// The papers on the front punching.
    front_papers: Vec<f64>,
}

impl KeyBench {
    /// A bench regulated to the given targets.
    ///
    /// `None` when the targets cannot be met on this geometry — which is a real
    /// answer and not an error to swallow.
    pub fn new(key: KeyDesign, mut frame: FrameDesign, target: Regulation) -> Option<Self> {
        let (balance, front) = target.solve(&key, &frame, 0.0)?;
        let (balance_felt, front_felt) = (frame.balance_punching.felt, frame.front_punching.felt);
        // Keep the felt the technician fitted and put the difference in paper,
        // which is what he would actually do.
        frame.balance_punching = PunchingStack::new(
            balance - frame.balance_punching.felt,
            frame.balance_punching.felt,
        );
        frame.front_punching =
            PunchingStack::new(front - frame.front_punching.felt, frame.front_punching.felt);
        if frame.balance_punching.paper < 0.0 || frame.front_punching.paper < 0.0 {
            return None;
        }

        let assembly = KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
            },
            // Stanwood's protocol until the bench is told otherwise; see
            // `set_measuring_point_from_lip`.
            protocol: TouchweightProtocol::default(),
        };
        // Six grams is the order of Key Friction Weight on a well-centred key;
        // it is a starting value to be replaced by a measurement, and it is the
        // single parameter the two non-confusion tests move. [E] The radius is
        // this key's measuring point, read where every protocol reads it, not
        // a number retyped from the default key: that would give any other key
        // a friction bound measured on someone else's arm.
        let friction = KeyPinFriction::FromKeyFrictionWeight {
            kf_g: 6.0,
            arm: assembly.measuring_point(),
        };
        let mut bench = Self {
            assembly,
            kind: KeyKind::Natural,
            friction,
            depression: 0.0,
            finger_weight_g: 0.0,
            target,
            // The solve gave a continuous thickness; express it as a stack a
            // technician could actually build.
            balance_papers: papers_for(balance - balance_felt),
            front_papers: papers_for(front - front_felt),
        };
        // And then adopt what that stack really measures. Without this the
        // geometry holds the continuous value while the drawer holds a
        // slightly thinner pile, and the first paper laid silently corrects
        // the difference as well as adding its own thickness, so adding and
        // removing one paper does not cancel out.
        bench.sync_stacks();
        Some(bench)
    }

    /// The default bench: a middle natural, regulated to Steinway New York.
    pub fn standard() -> Self {
        Self::new(
            KeyDesign::default(),
            FrameDesign::default(),
            Regulation {
                key_height: 0.0650,
                key_dip: 0.0102,
            },
        )
        .expect("the standard bench must be regulable")
    }

    /// Read-only view of the assembly.
    pub fn assembly(&self) -> &KeyAssembly {
        &self.assembly
    }

    /// How far the key is pressed, 0 at rest and 1 at the bottom.
    pub fn depression(&self) -> f64 {
        self.depression
    }

    /// Presses the key to a fraction of its dip, clamped to `[0, 1]`.
    ///
    /// Clamped rather than refused: a finger cannot push a key past its punching
    /// nor pull it above its rest, so values outside the range describe nothing.
    pub fn set_depression(&mut self, f: f64) {
        self.depression = f.clamp(0.0, 1.0);
    }

    /// Sine of the key's current angle.
    pub fn sin_angle(&self) -> f64 {
        self.attitude().0
    }

    /// The key's attitude, and how far it stands off its balance punching.
    ///
    /// The lift is zero on any frame a key can sit in, which is every frame
    /// the bench can weigh. It is **not** zero on one edited past that point:
    /// a back rail cloth holding the tail higher than the front punching lets
    /// the front come down leaves no angle satisfying both, and the key
    /// answers by riding on the two of them and lifting off its bearing. The
    /// statics refuse such a frame — [`KeyBench::frame_fault`] says so — but
    /// the key is somewhere, and it is not inside the wood.
    fn attitude(&self) -> (f64, f64) {
        // **Ask whether the stops fight, not whether the travel exists.**
        // `travel()` needs only the two stop angles, and both exist on a frame
        // no key can sit in, so a fallback keyed off it is never reached: the
        // key would be drawn at the back rail's angle, through its front
        // punching and still touching its balance punching, and the picture
        // would say nothing was wrong.
        if self.stops_fight() {
            if let Some(ride) = self.assembly.riding_on_both_stops() {
                return ride;
            }
        }
        match self.assembly.travel() {
            Some(t) => (
                t.sin_rest + self.depression * (t.sin_bottom - t.sin_rest),
                0.0,
            ),
            None => self.assembly.riding_on_both_stops().unwrap_or((0.0, 0.0)),
        }
    }

    /// Whether the two stops leave the key no angle that satisfies both.
    ///
    /// One predicate, because the drawing and the diagnosis have to agree
    /// about which frames are impossible.
    fn stops_fight(&self) -> bool {
        matches!(
            (
                self.assembly.sin_angle_on_back_rail(),
                self.assembly.sin_angle_on_front_rail()
            ),
            (Some(back), Some(front)) if back > front
        )
    }

    /// Why this frame cannot be weighed, if it cannot.
    ///
    /// Two different failures, and a technician needs to be told which. One is
    /// a frame whose stops fight: assembled, real, and the key rides clear of
    /// its balance punching — a degree of freedom this model does not have.
    /// The other is geometry edited into nonsense.
    pub fn frame_fault(&self) -> Option<&'static str> {
        if self.stops_fight() {
            // Which pair is fighting matters: raising the pin until it bottoms
            // early over-constrains the key just as a thick cloth does, and
            // blaming the cloth then names two parts that are innocent.
            return Some(if self.assembly.stopped_by_the_mortise() {
                "the front rail pin bottoms in its mortise before the back rail cloth lets the \
                 tail come up, so the key rides on both and lifts off its balance punching. A \
                 real keyboard does exactly that; this bench cannot weigh it, because the \
                 bearing is modelled as a pivot."
            } else {
                "the back rail cloth holds the tail higher than the front punching lets the \
                 front come down, so the key rides on both and lifts off its balance punching. \
                 A real keyboard does exactly that; this bench cannot weigh it, because the \
                 bearing is modelled as a pivot."
            });
        }
        let a = &self.assembly;
        match (a.sin_angle_on_back_rail(), a.sin_angle_on_front_rail()) {
            (Some(_), Some(_)) => None,
            _ => Some("the geometry is degenerate."),
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
        let frame = self.assembly.frame.parts(
            &self.assembly.key,
            self.kind,
            &self.balance_papers,
            &self.front_papers,
            &mut parts,
        );
        let key = self.assembly.key.parts(KEY, &mut parts);
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

    /// Where each body of the scene stands: the ground's frame is the world's,
    /// and the key's stands where its statics put it. One placement per body,
    /// in body order, applied by the view as it is given.
    pub fn poses(&self) -> Vec<Placement> {
        vec![Placement::WORLD, self.key_placement()]
    }

    /// Where the key's frame stands in the keybed's: its origin, at the
    /// balance pin on the underside, and its rotation, positive nose-up.
    ///
    /// The key turns about its bearing on the balance punching, and stands
    /// off it by the lift a frame whose stops fight gives it; nothing else
    /// places a point of the key.
    pub fn key_placement(&self) -> Placement {
        let (s, lift) = self.attitude();
        let c = (1.0 - s * s).max(0.0).sqrt();
        // The key turns about its bearing, `bx` along the underside from the
        // origin of its frame, which stands wherever that turn carries it.
        let bx = -self.assembly.bearing.offset_from_pin;
        let by = self.assembly.frame.balance_bearing_height() + lift;
        Placement {
            origin: Point::new(bx - c * bx, by + s * bx),
            cos: c,
            // The attitude's sine is positive front-down, a clockwise turn.
            sin: -s,
        }
    }

    /// The key on its frame as a mechanism: its parts and its frame's, the
    /// contacts between them ([`mounts::key_on_frame`]), and the key turning
    /// about its balance bearing, raised by `lift` off it, metres, as a frame
    /// whose stops fight raises it.
    ///
    /// The parts are [`Self::parts`]' own, the ones the view draws, so every
    /// contact reads a range of a drawn part. `None` only if the parts cannot
    /// be read, which the builders never produce.
    pub fn key_on_frame(&self, lift: f64) -> Option<Mounted> {
        let (parts, key, frame) = self.parts();
        let pairs = mounts::key_on_frame(&key, &frame);
        let bearing = Point::new(
            -self.assembly.bearing.offset_from_pin,
            self.assembly.frame.balance_bearing_height() + lift,
        );
        mounts::pivot(
            composite_key::mass(&self.assembly.key),
            bearing,
            parts,
            pairs,
        )
        .ok()
    }

    /// Every contact row of the key on its frame at an attitude of the
    /// statics, `sin_theta` positive front-down, raised by `lift`, within
    /// `margin`.
    fn rows_at(&self, sin_theta: f64, lift: f64, margin: f64) -> Option<(Mounted, Pose, Rows)> {
        let mounted = self.key_on_frame(lift)?;
        let pose = mounted.mechanism.pose(&[nose_up_angle(sin_theta)]);
        let mut rows = Rows::new();
        mounted.mechanism.rows(&pose, margin, &mut rows);
        Some((mounted, pose, rows))
    }

    /// The contacts the view marks: every row within the margin the static
    /// solve reads, at the attitude the key is drawn in, its frame's lift
    /// included.
    pub fn contacts(&self) -> Vec<ShownContact> {
        let (s, lift) = self.attitude();
        let Some((mounted, _, rows)) = self.rows_at(s, lift, ROW_MARGIN) else {
            return Vec::new();
        };
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

    /// How the key bears on its back rail cloth at rest, the attitude the
    /// statics put it in tail-down on the cloth: on the cloth's edge, flat on
    /// it, or on its own corner, where, and its face's slope over the
    /// cloth's.
    pub fn bearing_at_the_back(&self) -> Option<ContactBearing> {
        let travel = self.assembly.travel()?;
        let (mounted, pose, rows) = self.rows_at(travel.sin_rest, 0.0, f64::INFINITY)?;
        let pair = pair_of(&mounted, ContactCode::BackRailCloth)?;
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
        Point::new(
            lip.x - self.assembly.protocol.measuring_point_from_lip,
            lip.y,
        )
    }

    /// The leads, in the key's frame, as centre and radius in metres.
    ///
    /// Radius from the mass, taking lead at 11 340 kg/m³ and a disc the width of
    /// the key: a 10 g lead comes out near 5 mm, which is the size of the real
    /// thing. Drawing them to scale matters — a technician judges a leading by
    /// looking at it.
    pub fn lead_discs(&self) -> Vec<(Point, f64)> {
        let w = self.assembly.key.width;
        self.assembly
            .key
            .leads
            .iter()
            .map(|l| {
                let radius = (l.mass / (11_340.0 * core::f64::consts::PI * w)).sqrt();
                (Point::new(l.from_balance, l.height), radius)
            })
            .collect()
    }

    /// Weighs the key and reads every observable the bench reports.
    ///
    /// Front weight and the key weight ratio are the jig run on the model, the
    /// key's mass the composite key's derivation ([`composite_key`]). The
    /// weights on the frame are still the closed form taken with the key
    /// level.
    pub fn readout(&self) -> Option<BenchReadout> {
        let key = &self.assembly.key;
        let props = key.mass_properties()?;
        let travel = self.assembly.travel()?;

        let jig = KeyJig::new(key, &self.assembly.protocol, composite_key::mass(key)).ok()?;
        let on_frame = KeyOnFrame {
            mass: props.mass,
            static_moment: self.assembly.static_moment_about_bearing()?,
            measuring_arm: self.assembly.measuring_arm(),
            friction: self.friction,
        };
        let Branches { down, up } = on_frame.branches()?;

        // The gaps where the key stands now, read off the contact rows at
        // the attitude it is drawn in.
        let (s, lift) = self.attitude();
        let (mounted, _, rows) = self.rows_at(s, lift, f64::INFINITY)?;
        let gap_mm = |code| Some(rows.gap(pair_of(&mounted, code)?)? * 1000.0);

        // Where the tail bears at rest, and how: one reading of the cloth's
        // rows at the attitude the statics rest the key at, so the three
        // numbers are one answer rather than three questions asked
        // separately.
        let back = self.bearing_at_the_back()?;

        Some(BenchReadout {
            bedding_at_the_back_deg: back.slope?.atan().to_degrees(),
            bears_at_the_back_mm: -back.point.x * 1000.0,
            back_bearing: back.how,
            stopped_by_the_mortise: self.assembly.stopped_by_the_mortise(),
            out_of_level_deg: self.assembly.sin_angle_on_front_rail()?.asin().to_degrees(),
            gap_front_mm: gap_mm(ContactCode::FrontRailPunching),
            gap_back_mm: gap_mm(ContactCode::BackRailCloth),
            friction_band_g: 1000.0 * self.friction_bound() / self.assembly.measuring_arm(),
            measured_kf_g: self.measure_key_friction_weight(),
            measured_dip_mm: self.measure_key_dip().map(|d| d * 1000.0),
            key_height_target_mm: self.target.key_height * 1000.0,
            key_dip_target_mm: self.target.key_dip * 1000.0,
            finger: self.finger_verdict()?,
            front_weight_g: jig.front_weight_g().ok()?,
            key_ratio: jig.key_ratio().ok()?,
            key_height_mm: travel.key_height * 1000.0,
            key_dip_mm: travel.key_dip * 1000.0,
            down_weight_g: down,
            up_weight_g: up,
            balance_weight_g: 0.5 * (down + up),
            friction_weight_g: 0.5 * (down - up),
            key_mass_g: props.mass * 1000.0,
            lead_count: key.leads.len(),
        })
    }

    // ---- the leading workshop -------------------------------------------

    /// Fits a lead at `from_balance` metres, of `mass_g` grams.
    pub fn add_lead(&mut self, from_balance: f64, mass_g: f64) {
        self.assembly
            .key
            .leads
            .push(Lead::at(from_balance, mass_g * 1e-3));
    }

    /// Moves lead `i` to a new position. No effect if there is no such lead.
    pub fn move_lead(&mut self, i: usize, from_balance: f64) {
        if let Some(l) = self.assembly.key.leads.get_mut(i) {
            l.from_balance = from_balance;
        }
    }

    /// Removes lead `i`. No effect if there is no such lead.
    pub fn remove_lead(&mut self, i: usize) {
        if i < self.assembly.key.leads.len() {
            self.assembly.key.leads.remove(i);
        }
    }

    /// The leads as `(position in metres, mass in grams)`.
    pub fn leads(&self) -> Vec<(f64, f64)> {
        self.assembly
            .key
            .leads
            .iter()
            .map(|l| (l.from_balance, l.mass * 1000.0))
            .collect()
    }

    // ---- regulation ------------------------------------------------------

    /// **Runs Stanwood's Key Friction Weight protocol on the simulated key.**
    ///
    /// Not a parameter read back: the procedure, performed.
    ///
    /// > *"Key Friction Weight (KF) — a component of Friction Weight which is
    /// > the minimum amount of weight, to the nearest gram, placed on the
    /// > measuring point of a key that causes the key to fall, with the Front
    /// > Weight (FW) set to zero by temporary weight and with the key on its
    /// > frame and the stack removed."* [V]
    ///
    /// So three steps, in order, and each one is a real operation on the model:
    ///
    /// 1. **Zero the front weight** with a temporary weight slid along the key
    ///    until the scale reads nothing. Zeroing it first is what separates
    ///    friction from balance — without it the reading would be the sum of
    ///    the two, which is the down weight and not the friction.
    /// 2. **Put the key back on its frame**, back end down on the cloth.
    /// 3. **Find the least weight at the measuring point that makes it fall**,
    ///    by asking the contact solve, not by evaluating a formula. The key
    ///    falls when it leaves the back rail cloth, and that is a question
    ///    about unilateral contacts and a friction cone, answered by
    ///    `settle`.
    ///
    /// This is the simulator measuring itself with the technician's protocol.
    /// Round-tripping it — set the friction, run the procedure, read it back —
    /// exercises the chain from the contact geometry to the weighing sheet. It
    /// is not independent of the parameter: with a constant friction bound the
    /// procedure reads that bound back through the measuring arm at the key's
    /// resting attitude, so what the round trip checks is the zeroing, the arm
    /// and the contact state, not the friction model.
    ///
    /// The search is a **fixed** 32-step bisection, never a tolerance.
    pub fn measure_key_friction_weight(&self) -> Option<f64> {
        let props = self.assembly.key.mass_properties()?;

        // Step 2 first, because step 1 depends on it: the attitude the key is
        // in on its frame, back end down on its cloth.
        //
        // Stanwood zeroes the front weight on the levelled jig and only then
        // puts the key back on its frame [V, Stanwood 2000, p. 22]. This
        // zeroes it in the frame attitude instead, which cancels the gravity
        // moment exactly where the key is weighed. The two differ, because
        // tilting the key changes each mass's arm by its height above the
        // pivot: about 0.3 g on the default key, under the gram the protocol
        // reads to. A known departure from the protocol (FOUNDATIONS §2.2),
        // to go when the protocols run on the general solver.
        let on_cloth = self.assembly.sin_angle_on_back_rail()?;
        let torque = self.assembly.gravity_torque(on_cloth)?;

        // Step 1: the temporary weight that brings front weight to zero. A
        // technician slides it; here its position is fixed and its mass solved,
        // which is the same cancellation. The torque is exactly linear in that
        // mass, so one probe gives the answer rather than a search.
        let at = -0.100_f64;
        let probe_mass = 0.010_f64;
        let mut probe = self.assembly.clone();
        probe.key.leads.push(Lead::at(at, probe_mass));
        let per_kg = (probe.gravity_torque(on_cloth)? - torque) / probe_mass;
        let counterweight = -torque / per_kg;
        if !counterweight.is_finite() || counterweight <= 0.0 {
            return None;
        }
        let mut zeroed = self.assembly.clone();
        zeroed.key.leads.push(Lead::at(at, counterweight));
        debug_assert!(
            zeroed
                .gravity_torque(on_cloth)
                .is_some_and(|m| m.abs() < 1e-12),
            "the temporary weight must leave the key balanced in the attitude \
             it is measured in"
        );

        let bound = self.friction.bound(props.mass + counterweight);

        // Step 3: the least weight that lifts it off that cloth. Bracket wide
        // enough that any plausible friction is inside, then halve.
        let falls = |w: f64| {
            zeroed
                .settle(on_cloth, w, bound)
                .is_some_and(|s| s.held_by != pa_core::parts::assembly::HeldBy::BackRailCloth)
        };
        let (mut lo, mut hi) = (0.0_f64, 200.0_f64);
        if !falls(hi) {
            return None;
        }
        for _ in 0..32 {
            let mid = 0.5 * (lo + hi);
            if falls(mid) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(0.5 * (lo + hi))
    }

    /// **Measures the key dip by pressing the key, not by solving for it.**
    ///
    /// A technician does not compute a dip. He holds the back of the key down,
    /// reads the front, presses the key to the bottom, and reads it again; the
    /// difference is the dip, and a dip block is how he checks it against a
    /// specification. This performs that, and every step goes through the
    /// contact solve:
    ///
    /// 1. **Hold the back down** — a lift at the measuring point large enough
    ///    to put the key on its back rail cloth — and read the front of the key.
    /// 2. **Press it to the bottom** — a weight large enough to put it on the
    ///    front rail punching — and read the front again.
    /// 3. The dip is what moved.
    ///
    /// Both ends are where `settle` says the key comes to rest. On a stop,
    /// `settle` returns that stop's own angle, the one
    /// [`KeyAssembly::travel`] uses, and the dip is then read by the same
    /// expression, so this gives the analytic travel again, bit for bit. It is
    /// the same computation as the solved dip, not a check on it.
    ///
    /// `None` if either end cannot be reached, which an edited geometry can
    /// produce.
    pub fn measure_key_dip(&self) -> Option<f64> {
        let bound = self.friction_bound();
        let heavy = 400.0;

        let at_rest = self.assembly.settle(self.sin_angle(), -heavy, bound)?;
        if at_rest.held_by != pa_core::parts::assembly::HeldBy::BackRailCloth {
            return None;
        }
        let at_bottom = self.assembly.settle(at_rest.sin_theta, heavy, bound)?;
        if at_bottom.held_by != pa_core::parts::assembly::HeldBy::FrontPunching {
            return None;
        }

        use pa_core::parts::assembly::ReadAt;
        Some(
            self.assembly
                .key_height_at(ReadAt::FrontEdge, at_rest.sin_theta)
                - self
                    .assembly
                    .key_height_at(ReadAt::FrontEdge, at_bottom.sin_theta),
        )
    }

    /// Lays one more paper on a stack. Key levelling, or key dip.
    pub fn add_paper(&mut self, which: Which, thickness: f64) {
        match which {
            Which::Balance => self.balance_papers.push(thickness.max(0.0)),
            Which::Front => self.front_papers.push(thickness.max(0.0)),
        }
        self.sync_stacks();
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
        self.assembly.frame.balance_punching.paper = self.balance_papers.iter().sum();
        self.assembly.frame.front_punching.paper = self.front_papers.iter().sum();
    }

    /// Sets the paper under the balance punching to a given total, metres.
    ///
    /// Kept for the tests and the command line, where a thickness is the
    /// natural way to express an experiment. The bench's own control is
    /// [`Self::add_paper`], because that is the operation.
    pub fn set_balance_paper(&mut self, paper: f64) {
        self.balance_papers = papers_for(paper);
        self.sync_stacks();
    }

    /// Sets the paper on the front punching to a given total, metres.
    pub fn set_front_paper(&mut self, paper: f64) {
        self.front_papers = papers_for(paper);
        self.sync_stacks();
    }

    /// The two paper stacks, metres.
    pub fn papers(&self) -> (f64, f64) {
        (
            self.assembly.frame.balance_punching.paper,
            self.assembly.frame.front_punching.paper,
        )
    }

    /// Puts a weight on the measuring point and lets the physics decide.
    ///
    /// Quasi-static, because there is no integrator yet: the key arrives where
    /// [`Self::settle`] puts it rather than travelling there. Reporting
    /// that honestly is better than animating a motion nothing has computed —
    /// and the *interesting* part is visible either way, which is that nothing
    /// happens at all between the two branches.
    pub fn set_finger_weight(&mut self, grams: f64) {
        self.finger_weight_g = grams;
        self.come_to_rest();
    }

    /// Lets the key come to rest under the weight on the measuring point.
    fn come_to_rest(&mut self) {
        if let Some(s) = self.settle() {
            self.depression = self.depression_of(s.sin_theta);
        }
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
    /// A weight already on the key moves with the point, so the key settles
    /// again under it. Clamped to the covering, from its lip to its back end:
    /// in front of the lip there is no key to put a weight on, and behind the
    /// covering the point would no longer stand on the top the statics put it
    /// on. A value that is not a number is not a distance, and leaves the point
    /// where it was.
    pub fn set_measuring_point_from_lip(&mut self, from_lip: f64) {
        if !from_lip.is_finite() {
            return;
        }
        let covering = self.assembly.key.keytop_length.max(0.0);
        self.assembly.protocol.measuring_point_from_lip = from_lip.clamp(0.0, covering);
        self.come_to_rest();
    }

    /// How far back from the front lip of the key every weight is centred,
    /// metres.
    pub fn measuring_point_from_lip(&self) -> f64 {
        self.assembly.protocol.measuring_point_from_lip
    }

    /// Solves where the key rests under the current weight, with both stops
    /// unilateral and the centre friction set-valued.
    pub fn settle(&self) -> Option<Settled> {
        self.assembly.settle(
            self.sin_angle(),
            self.finger_weight_g,
            self.friction_bound(),
        )
    }

    /// The friction bound as a mass-moment about the bearing, kg·m.
    ///
    /// The load that sets it is the one the **bearing** actually carries, not
    /// the whole weight on the key: when the key is down on a stop, that stop
    /// is holding part of it. For the constant model this makes no difference;
    /// for the load-proportional one it is the difference between a friction
    /// that follows the physics and one that charges the centre for weight it
    /// is not holding.
    fn friction_bound(&self) -> f64 {
        let total = self
            .assembly
            .key
            .mass_properties()
            .map_or(0.0, |p| p.mass + self.finger_weight_g * 1e-3);

        // Chicken and egg: the bound decides where the key settles, and where
        // it settles decides the load. Resolve it in one pass — the bound from
        // the total weight is an upper estimate, good enough to find the
        // contact state, and the state then gives the load the bound is
        // really taken at. One pass and not a loop, because a second pass
        // moves the answer by less than the scale can read.
        let coarse = self.friction.bound(total);
        match self
            .assembly
            .settle(self.sin_angle(), self.finger_weight_g, coarse)
            .and_then(|s| {
                self.assembly
                    .bearing_load_with_stops(s, self.finger_weight_g)
            }) {
            Some((at_bearing, _)) => self.friction.bound(at_bearing),
            None => coarse,
        }
    }

    /// Converts an angle back into a fraction of the dip.
    fn depression_of(&self, sin_theta: f64) -> f64 {
        match self.assembly.travel() {
            Some(t) if (t.sin_bottom - t.sin_rest).abs() > 1e-15 => {
                ((sin_theta - t.sin_rest) / (t.sin_bottom - t.sin_rest)).clamp(0.0, 1.0)
            }
            _ => self.depression,
        }
    }

    /// The weight currently on the measuring point, grams.
    pub fn finger_weight(&self) -> f64 {
        self.finger_weight_g
    }

    /// What is carrying the key right now. `None` for a degenerate geometry.
    pub fn finger_verdict(&self) -> Option<FingerVerdict> {
        Some(self.settle()?.held_by)
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
        self.assembly.key.capstan = from_balance.min(-0.010);
    }

    /// Moves the front edge of the key, metres in front of the balance pin.
    ///
    /// This is the key's length in front of its pivot. The measuring point
    /// moves with it, a set distance back from the covering's lip, so with
    /// the capstan and that distance it sets the key weight ratio.
    pub fn set_front_edge(&mut self, from_balance: f64) {
        self.assembly.key.front_edge = from_balance.max(0.050);
    }

    /// Screws the capstan up or down, metres above the keystick.
    pub fn set_capstan_height(&mut self, height: f64) {
        self.assembly.key.capstan_screw.height = height.clamp(0.002, 0.030);
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
        self.assembly.frame = self.assembly.frame.clone().placed_under(&self.assembly.key);
    }

    /// Where the front rail stands, metres in front of the balance pin.
    pub fn set_front_rail(&mut self, from_balance: f64) {
        self.assembly.frame.front_rail = from_balance.max(0.010);
    }

    /// Where the back rail stands, metres behind the balance pin (negative).
    pub fn set_back_rail(&mut self, from_balance: f64) {
        self.assembly.frame.back_rail = from_balance.min(-0.010);
    }

    /// Where the two placed rails stand, metres: front rail, back rail.
    ///
    /// The back one is signed as it is stored — negative, behind the pin.
    pub fn rail_positions(&self) -> (f64, f64) {
        (
            self.assembly.frame.front_rail,
            self.assembly.frame.back_rail,
        )
    }

    /// The three dimensions the bench exposes, in metres.
    pub fn key_dimensions(&self) -> (f64, f64, f64) {
        (
            self.assembly.key.capstan,
            self.assembly.key.front_edge,
            self.assembly.key.capstan_screw.height,
        )
    }

    // ---- the frame's own dimensions --------------------------------------

    /// Height of the bare balance rail above the keybed, metres.
    pub fn set_balance_rail_height(&mut self, h: f64) {
        self.assembly.frame.balance_rail_height = h.max(0.0);
    }

    /// Height of the bare front rail above the keybed, metres.
    pub fn set_front_rail_height(&mut self, h: f64) {
        self.assembly.frame.front_rail_height = h.max(0.0);
    }

    /// Height of the bare back rail above the keybed, metres.
    pub fn set_back_rail_height(&mut self, h: f64) {
        self.assembly.frame.back_rail_height = h.max(0.0);
    }

    /// Thickness of the back rail cloth, metres.
    pub fn set_back_rail_cloth(&mut self, t: f64) {
        self.assembly.frame.back_rail_cloth = t.max(0.0);
    }

    /// How far the front rail pin stands above the bare rail, metres.
    pub fn set_front_pin_height(&mut self, h: f64) {
        self.assembly.frame.sections.front.pin_height = h.max(0.0);
    }

    /// Depth of the key's front mortise above its underside, metres.
    pub fn set_front_mortise_depth(&mut self, d: f64) {
        self.assembly.key.front_mortise_depth = d.max(0.0);
    }

    /// The front rail pin's height above the rail and the mortise it enters,
    /// metres.
    pub fn front_pin_and_mortise(&self) -> (f64, f64) {
        (
            self.assembly.frame.sections.front.pin_height,
            self.assembly.key.front_mortise_depth,
        )
    }

    /// The four frame dimensions, metres: balance rail, front rail, back rail,
    /// back rail cloth.
    pub fn frame_dimensions(&self) -> (f64, f64, f64, f64) {
        let f = &self.assembly.frame;
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
        self.assembly.frame.balance_punching.felt = felt.max(0.0);
    }

    /// Thickness of the felt washer on the front punching, metres.
    pub fn set_front_felt(&mut self, felt: f64) {
        self.assembly.frame.front_punching.felt = felt.max(0.0);
    }

    /// The two felt washers, metres.
    pub fn felts(&self) -> (f64, f64) {
        (
            self.assembly.frame.balance_punching.felt,
            self.assembly.frame.front_punching.felt,
        )
    }

    /// Sets the key friction weight, grams at the measuring point.
    ///
    /// Stored as a torque with the arm it is read at, the measuring point as
    /// it stands now. Moving the measuring point afterwards leaves the torque
    /// alone, so the key friction weight the protocol reads changes by the
    /// lever: the bushings have not changed, only where they are weighed.
    pub fn set_key_friction_weight(&mut self, kf_g: f64) {
        self.friction = KeyPinFriction::FromKeyFrictionWeight {
            kf_g: kf_g.max(0.0),
            arm: self.assembly.measuring_point(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bench reports the regulation it was asked for.
    #[test]
    fn a_standard_bench_reports_what_it_achieved_beside_what_it_was_asked_for() {
        let r = KeyBench::standard()
            .readout()
            .expect("the standard bench reads");

        // Target and achieved are two numbers, and the bench carries both. They
        // differ for three physical reasons, all of them reported rather than
        // hidden: the punchings are *placed* by a small-angle solve and the
        // result is *read* off the exact contact; the reading includes the
        // vertical drop of the key top as it turns; and the stack is built
        // from real papers, which cannot hit an arbitrary thickness.
        //
        // So the bound is not a tolerance anybody chose — it is **one
        // paper**. The thinnest in the drawer is 0.08 mm, which at the front
        // of the key is 0.08 x 258/225. A regulation that lands inside that is
        // as close as the drawer allows; one that does not has a real error in
        // it.
        let one_paper_mm = PAPER_THICKNESSES[0] * 1000.0 * (258.0 / 225.0);

        assert!((r.key_height_target_mm - 65.0).abs() < 1e-12);
        assert!((r.key_dip_target_mm - 10.2).abs() < 1e-12);
        assert!(
            (r.key_height_mm - r.key_height_target_mm).abs() < one_paper_mm,
            "height achieved {} against target {}, missing by more than one paper",
            r.key_height_mm,
            r.key_height_target_mm
        );
        assert!(
            (r.key_dip_mm - r.key_dip_target_mm).abs() < one_paper_mm,
            "dip achieved {} against target {}, missing by more than one paper",
            r.key_dip_mm,
            r.key_dip_target_mm
        );

        assert!(
            (0.50..=0.60).contains(&r.key_ratio),
            "ratio {}",
            r.key_ratio
        );
        assert!(
            r.key_mass_g > 40.0 && r.key_mass_g < 160.0,
            "mass {} g",
            r.key_mass_g
        );
    }

    /// The friction band is a band, and the key does nothing inside it.
    ///
    /// Put a weight on the measuring point and three things can happen, not
    /// two. Between the up weight and the down weight the key **stays exactly
    /// where it is** — that gap is the friction, and it is the most visible
    /// consequence of modelling dry friction as set-valued rather than as a
    /// `tanh`. A regularized model would creep instead.
    ///
    /// The key is balanced about its bearing first, with a lead behind the
    /// pin, so that the band straddles zero and the test exercises the
    /// *floating* state rather than a stop.
    #[test]
    fn nothing_happens_inside_the_friction_band() {
        let mut b = KeyBench::standard();
        // `add_lead` takes metres, like every other length in the domain.
        b.add_lead(-0.120, 30.0);

        // Park the key in the middle of its travel, off both stops.
        b.set_depression(0.5);
        let parked = b.sin_angle();

        // The weight that balances it there, and a gram either side of it.
        let balancing =
            -1000.0 * b.assembly().net_torque(parked, 0.0).unwrap() / b.assembly().measuring_arm();
        for nudge in [-1.0, 0.0, 1.0] {
            b.set_finger_weight(balancing + nudge);
            assert_eq!(
                b.finger_verdict(),
                Some(FingerVerdict::FrictionAlone),
                "at {} g the key should be held by friction alone",
                balancing + nudge
            );
            assert!(
                (b.sin_angle() - parked).abs() < 1e-9,
                "the key moved inside its own friction band"
            );
        }

        // Well past the band, it reaches a stop — and is then held by the stop,
        // not reported as for ever descending.
        b.set_finger_weight(balancing + 40.0);
        assert_eq!(b.finger_verdict(), Some(FingerVerdict::FrontPunching));
        assert!(b.sin_angle() > parked, "past the band the key must descend");

        b.set_finger_weight(balancing - 60.0);
        assert_eq!(b.finger_verdict(), Some(FingerVerdict::BackRailCloth));
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
        b.set_key_friction_weight(2.0);
        assert_eq!(
            b.finger_verdict(),
            Some(FingerVerdict::FrontPunching),
            "a front-heavy key sits at the bottom of its dip, and the \
             punching carries it"
        );
    }

    /// Widening the friction widens the band, and does not move its centre.
    #[test]
    fn more_friction_makes_a_wider_band_around_the_same_centre() {
        let mut b = KeyBench::standard();
        let before = b.readout().unwrap();
        b.set_key_friction_weight(12.0);
        let after = b.readout().unwrap();

        assert!(
            (after.down_weight_g - after.up_weight_g) > (before.down_weight_g - before.up_weight_g),
            "doubling the friction must widen the band"
        );
        assert!(
            (after.balance_weight_g - before.balance_weight_g).abs() < 1e-9,
            "and must not move its centre"
        );
    }

    /// The leading workshop is three operations on the model, and each one
    /// moves the balance weight in the direction a technician expects.
    #[test]
    fn leading_moves_the_balance_weight_and_unleading_puts_it_back() {
        let mut b = KeyBench::standard();
        let before = b.readout().unwrap();

        b.add_lead(0.150, 10.0);
        let leaded = b.readout().unwrap();
        assert_eq!(leaded.lead_count, 1);
        assert!(
            leaded.balance_weight_g < before.balance_weight_g,
            "lead in front must lighten the touch: {} -> {}",
            before.balance_weight_g,
            leaded.balance_weight_g
        );
        assert!(
            leaded.front_weight_g > before.front_weight_g,
            "and must raise the front weight"
        );

        // Moving it further forward does more of the same.
        b.move_lead(0, 0.200);
        let moved = b.readout().unwrap();
        assert!(moved.balance_weight_g < leaded.balance_weight_g);

        b.remove_lead(0);
        let after = b.readout().unwrap();
        assert_eq!(after.lead_count, 0);
        assert!((after.balance_weight_g - before.balance_weight_g).abs() < 1e-9);
    }

    /// Pressing the key is a travel between two computed attitudes, and it is
    /// clamped at both ends because a finger cannot do otherwise.
    #[test]
    fn the_key_travels_between_rest_and_the_punching_and_no_further() {
        let mut b = KeyBench::standard();
        let rest = b.sin_angle();

        b.set_depression(1.0);
        let bottom = b.sin_angle();
        assert!(bottom > rest, "pressing must turn the key front-down");

        b.set_depression(4.0);
        assert!(
            (b.sin_angle() - bottom).abs() < 1e-15,
            "the key cannot go below its front punching"
        );
        b.set_depression(-1.0);
        assert!((b.sin_angle() - rest).abs() < 1e-15, "nor above its rest");
    }

    /// What is drawn moves when the key moves, and the frame does not: the
    /// key's pose turns it, its outlines stay as they are in its own frame,
    /// and the ground's pose is the world's.
    #[test]
    fn pressing_the_key_moves_only_the_key() {
        let mut b = KeyBench::standard();
        let (at_rest, rest_poses) = (b.outlines(), b.poses());
        let rest_pt = b.key_placement().world(b.measuring_point_on_key());
        b.set_depression(1.0);
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

    /// The scene's poses place the key where its statics put it: at rest, its
    /// frame's origin stands on the balance bearing, and turning it by the
    /// pose carries its underside's tail onto the cloth's height or above.
    #[test]
    fn the_key_stands_on_its_balance_bearing_at_rest() {
        let b = KeyBench::standard();
        let pose = b.key_placement();
        let bearing = b.assembly().frame.balance_bearing_height();
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
            let protocol = b.assembly().protocol;
            let weighed = protocol.measuring_point_on_key(&b.assembly().key);
            let marked = b.measuring_point_on_key();
            assert_eq!(marked.x.to_bits(), weighed.x.to_bits());
            assert_eq!(marked.y.to_bits(), weighed.y.to_bits());
        }
    }

    /// Adding paper under the balance punching raises the key, as on the bench.
    #[test]
    fn key_levelling_works_through_the_bench() {
        let mut b = KeyBench::standard();
        let before = b.readout().unwrap();
        let (paper, _) = b.papers();
        b.set_balance_paper(paper + 0.0005);
        let after = b.readout().unwrap();
        assert!(
            after.key_height_mm > before.key_height_mm,
            "0.5 mm of paper must raise the key: {} -> {}",
            before.key_height_mm,
            after.key_height_mm
        );
    }

    /// Leads are drawn to scale, and a heavier one is bigger.
    #[test]
    fn leads_are_drawn_at_the_size_of_real_leads() {
        let mut b = KeyBench::standard();
        b.add_lead(0.150, 10.0);
        b.add_lead(0.100, 20.0);
        let discs = b.lead_discs();
        assert_eq!(discs.len(), 2);
        assert!(
            (0.003..0.008).contains(&discs[0].1),
            "a 10 g lead should be about 5 mm in radius, got {} m",
            discs[0].1
        );
        assert!(discs[1].1 > discs[0].1, "20 g must draw bigger than 10 g");
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
        let (_, paper) = b.papers();

        let shim = 0.0005;
        b.set_front_paper(paper + shim);
        let after = b.readout().unwrap();

        // The front rail pin is 225 mm from the balance pin and the key front
        // 258 mm, so a shim at the punching shows up at the key front, where
        // the dip is read, multiplied by 258/225.
        let expected = shim * 1000.0 * (258.0 / 225.0);
        let moved = before.key_dip_mm - after.key_dip_mm;
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
        let f = &b.assembly.frame;
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
            (r.key_dip_mm - r.key_dip_target_mm).abs() < 2.0 * two_papers,
            "and the travel that attitude leaves is the dip"
        );
    }

    /// On a frame no key can sit in, the key is drawn where it would ride.
    ///
    /// The refusal and the diagnosis are not enough: the drawing has to reach
    /// the riding position too. Asking `travel()`, which answers for any frame
    /// with two stop angles, instead of whether those two angles leave an
    /// admissible one, draws the key at the back rail's angle — through its
    /// front punching, and with the balance punching still touching, which is
    /// the one thing the picture has to show is no longer true.
    #[test]
    fn a_wedged_frame_draws_the_key_riding_clear_of_its_bearing() {
        let mut b = KeyBench::standard();
        let level = b.assembly.frame.balance_bearing_height();
        assert!(b.frame_fault().is_none(), "the standard bench is buildable");
        assert!(
            b.attitude().1.abs() < 1e-15,
            "and on it the key sits on its bearing, not above it"
        );

        // Hold the tail higher than the punching lets the front come down.
        b.set_back_rail_height(level + 0.005 - b.assembly.frame.back_rail_cloth);
        assert!(
            b.frame_fault().is_some(),
            "this frame is meant to be refused"
        );
        assert!(b.readout().is_none(), "and refused by the weighing too");

        let (_, lift) = b.attitude();
        assert!(
            lift > 0.0,
            "the key must be drawn clear of the balance punching, not through \
             the wood: it lifted by {lift} m"
        );

        // And where it is drawn, it really is on both stops.
        let front = b.assembly.front_stop();
        let back = b.assembly.back_stop();
        let (sin, _) = b.attitude();
        let cos = (1.0 - sin * sin).max(0.0).sqrt();
        for (name, stop) in [("front", front), ("back", back)] {
            // Signed distance from each of the stop's edge centres to the
            // underside line, which passes through the lifted bearing along
            // (cos, -sin) and has (sin, cos) for its upward normal. A centre
            // sits one radius below the line when the key rests on that edge,
            // so this reads zero on the edge that carries the key and negative
            // on the one it clears.
            let clearances: Vec<f64> = stop
                .edges()
                .iter()
                .map(|c| c.x * sin + (c.y - (level + lift)) * cos + stop.edge_radius)
                .collect();
            assert!(
                clearances.iter().all(|d| *d < 1e-9),
                "the key is drawn through the {name} stop: {clearances:?}"
            );
            // A strip is carried on one of its two edges, not on its middle,
            // and which one depends on how the frame is wedged — so the claim
            // is that one of them is touching, not that a named one is.
            assert!(
                clearances.iter().any(|d| d.abs() < 1e-9),
                "the riding key must touch the {name} stop; it stands {clearances:?} off it"
            );
        }
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
            let target = Regulation {
                key_height: 0.0650,
                key_dip: 0.0102,
            };
            let Some(b) = KeyBench::new(key, frame, target) else {
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
    /// statics put it there: when it rests on a stop, that stop's gap reads
    /// zero, not "small", and the other stands open. If it did not, the drawn
    /// position and the physics would be two different states.
    #[test]
    fn the_reported_gaps_agree_with_where_the_key_is_held() {
        let mut b = KeyBench::standard();
        let mut held = Vec::new();
        for weight in [-60.0, 0.0, 60.0] {
            b.set_finger_weight(weight);
            let r = b.readout().unwrap();
            let front = r.gap_front_mm.expect("the punching stands under the flat");
            let back = r.gap_back_mm.expect("the cloth meets the underside");
            match r.finger {
                FingerVerdict::FrontPunching => {
                    assert!(
                        front.abs() < 1e-9,
                        "held by the front punching, yet its gap reads {front} mm"
                    );
                    assert!(back > 0.0, "and the back must be open");
                }
                FingerVerdict::BackRailCloth => {
                    assert!(back.abs() < 1e-9, "held by the cloth, its gap {back} mm");
                    assert!(front > 0.0);
                }
                FingerVerdict::FrictionAlone => {
                    assert!(front > 0.0 && back > 0.0);
                }
            }
            held.push(r.finger);
        }
        assert!(held.contains(&FingerVerdict::FrontPunching));
        assert!(held.contains(&FingerVerdict::BackRailCloth));
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
            let mounted = b.key_on_frame(0.0).expect("the key mounts on its frame");
            assert_eq!(
                mounted.mechanism.parts(),
                &drawn[..],
                "{}: the contacts read the parts the view draws",
                fixture.name
            );
            assert_eq!(mounted.mechanism.pairs().len(), 4, "{}", fixture.name);
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
    /// cloth, the cloth's back edge closed on the chamfer, nothing else within
    /// a millimetre (the cloth's front edge is no contact of its own, its
    /// nearest point to the chamfer being where it meets the face, and the
    /// face falling away from it towards the back edge); down on its
    /// punching, the punching closed at its centre. A closed contact's two
    /// points coincide.
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
        assert_eq!(closed(&b), vec![ContactCode::BackRailCloth]);
        let at_rest = b.contacts();
        let back_edge = b.assembly().back_stop().edges()[0].x;
        for c in &at_rest {
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
        assert_eq!(at_rest.len(), 1, "{at_rest:?}");

        b.set_finger_weight(60.0);
        assert_eq!(closed(&b), vec![ContactCode::FrontRailPunching]);
        let punching = b.assembly().punching_stop();
        for c in b.contacts() {
            if c.state() == ContactState::Closed {
                assert!(
                    (c.on_frame.x - punching.at).abs() < 1e-12,
                    "at the punching's centre"
                );
                assert!((c.on_frame.y - punching.top).abs() < 1e-12);
            }
        }
    }

    /// The friction band and the friction weight are the same thing, twice.
    #[test]
    fn the_friction_band_is_the_friction_weight() {
        let mut b = KeyBench::standard();
        for kf in [2.0, 6.0, 13.5] {
            b.set_key_friction_weight(kf);
            let r = b.readout().unwrap();
            assert!(
                (r.friction_band_g - kf).abs() < 1e-9,
                "band {} against a key friction weight of {kf} g",
                r.friction_band_g
            );
            assert!(
                (r.friction_weight_g - kf).abs() < 1e-9,
                "and (D-U)/2 must be the same number"
            );
        }
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
        let n = b.papers_on(Which::Balance).len();

        b.add_paper(Which::Balance, 0.0002);
        assert_eq!(b.papers_on(Which::Balance).len(), n + 1);

        let after = b.readout().unwrap();
        let risen = after.key_height_mm - before.key_height_mm;

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
        let before = b.readout().unwrap();

        b.add_paper(Which::Front, 0.0004);
        let shimmed = b.readout().unwrap();
        assert!(
            shimmed.key_dip_mm < before.key_dip_mm,
            "paper must shallow the dip"
        );

        let last = b.papers_on(Which::Front).len() - 1;
        b.remove_paper(Which::Front, last);
        let after = b.readout().unwrap();
        assert!(
            (after.key_dip_mm - before.key_dip_mm).abs() < 1e-9,
            "removing the paper must restore the dip exactly: {} -> {}",
            before.key_dip_mm,
            after.key_dip_mm
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
        let r = KeyBench::standard().readout().unwrap();
        assert!((r.key_dip_mm - r.key_dip_target_mm).abs() < 0.1);
    }
}

#[cfg(test)]
mod protocol_roundtrip_tests {
    use super::*;

    /// Level 4, and the one that matters most: **the simulator measures its own
    /// friction with the trade's protocol and gets back what was put in.**
    ///
    /// Set the key friction weight, then run Stanwood's procedure on the model
    /// — zero the front weight with a temporary weight, put the key on its
    /// frame, find the least weight at the measuring point that makes it fall
    /// — and read the answer off. The two must agree.
    ///
    /// It is not independent of the parameter. With a constant friction bound
    /// the procedure returns that bound read back through the measuring arm at
    /// the key's resting attitude, so the round trip checks the zeroing, the
    /// arm and the contact state — a wrong lever arm, datum or sign would show
    /// up here — and does not test the friction model itself.
    ///
    /// The tolerance is **derived from the key's attitude, not chosen.** The
    /// weighing algebra states the measuring arm with the key level, while the
    /// protocol runs through the exact contact solve on a key resting nose-up
    /// by the whole of its dip — and a tilted key presents a shorter arm,
    /// `cos θ · dx + sin θ · dy`, by about half a percent. So the protocol
    /// returns the parameter scaled by that ratio, and the bound is the ratio
    /// rather than a number anybody picked. Closing it means making the
    /// weighing algebra attitude-aware, which is not this test's business.
    #[test]
    fn the_bench_measures_its_own_friction_by_the_protocol() {
        for kf in [2.0, 6.0, 9.5, 14.0] {
            let mut b = KeyBench::standard();
            b.set_key_friction_weight(kf);
            let on_cloth = b.assembly.sin_angle_on_back_rail().expect("an attitude");
            let tilt_slip =
                (b.assembly.measuring_arm() / b.assembly.measuring_arm_at(on_cloth) - 1.0).abs();

            let measured = b
                .measure_key_friction_weight()
                .expect("the protocol must be runnable on a standard bench");
            assert!(
                (measured - kf).abs() < kf * tilt_slip + 0.005,
                "set {kf} g of key friction, the protocol measured {measured} g — \
                 further out than the {:.2} % the key's attitude accounts for",
                tilt_slip * 100.0
            );
        }
    }

    /// Leading the key does not change what the protocol measures.
    ///
    /// The procedure zeroes the front weight first, precisely so that the
    /// answer is friction and not balance. If leads moved the reading, that
    /// first step would not be doing its job — and the measure would be a
    /// down weight wearing a different name.
    #[test]
    fn leading_the_key_does_not_change_its_measured_friction() {
        let mut b = KeyBench::standard();
        b.set_key_friction_weight(7.0);
        let bare = b.measure_key_friction_weight().unwrap();

        b.add_lead(0.160, 18.0);
        b.add_lead(0.110, 12.0);
        let leaded = b.measure_key_friction_weight().unwrap();

        assert!(
            (leaded - bare).abs() < 0.05,
            "30 g of lead changed the measured friction from {bare} to {leaded}"
        );
    }

    /// A frictionless key measures no friction.
    #[test]
    fn a_frictionless_key_measures_nothing() {
        let mut b = KeyBench::standard();
        b.set_key_friction_weight(0.0);
        let measured = b.measure_key_friction_weight().unwrap();
        assert!(
            measured.abs() < 0.05,
            "a key with no friction should need no weight to fall, got {measured} g"
        );
    }
}

#[cfg(test)]
mod dip_procedure_tests {
    use super::*;

    /// Level 4: **pressing the key gives the same dip as solving for it.**
    ///
    /// Not two independent routes: on a stop `settle` returns the very angle
    /// the contact solve gives, and both readings take the same difference of
    /// key heights, so the two agree bit for bit by construction. What this
    /// checks is that a large lift and a large weight really do put the key on
    /// its back rail cloth and on its front punching.
    #[test]
    fn pressing_the_key_measures_the_dip_the_geometry_predicts() {
        let b = KeyBench::standard();
        let solved = b.readout().unwrap().key_dip_mm;
        let pressed = b.measure_key_dip().expect("the key must be pressable") * 1000.0;

        assert!(
            (pressed - solved).abs() < 1e-9,
            "pressed {pressed} mm against a solved {solved} mm"
        );
    }

    /// And it follows the punchings, which is the point of measuring at all.
    #[test]
    fn the_measured_dip_follows_the_papers() {
        let mut b = KeyBench::standard();
        let before = b.measure_key_dip().unwrap();

        b.add_paper(Which::Front, 0.0004);
        let after = b.measure_key_dip().unwrap();

        // 0.4 mm of paper at the front rail pin, seen at the front edge.
        let expected = 0.0004 * (258.0 / 225.0);
        assert!(
            ((before - after) - expected).abs() < 20e-6,
            "a 0.4 mm paper shallowed the measured dip by {} m, expected {expected}",
            before - after
        );
    }

    /// More friction does not change the dip.
    ///
    /// Dip is a geometry, friction is a threshold. A measurement that drove
    /// the key with weights could easily confuse them — stop a hair short of
    /// the punching and the dip reads small. The procedure uses a weight large
    /// enough that friction cannot matter, and this is what checks it.
    #[test]
    fn friction_does_not_change_the_measured_dip() {
        let mut b = KeyBench::standard();
        let light = b.measure_key_dip().unwrap();
        b.set_key_friction_weight(18.0);
        let heavy = b.measure_key_dip().unwrap();

        assert!(
            (heavy - light).abs() < 1e-12,
            "tripling the friction moved the measured dip from {light} to {heavy}"
        );
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
    /// not move at all.
    #[test]
    fn moving_the_measuring_point_changes_front_weight_exactly_by_the_lever() {
        let mut b = KeyBench::standard();
        let key = b.assembly().key.clone();
        let lip = key.front_edge + key.keytop_overhang;
        let moment_g_m = 1000.0 * key.mass_properties().unwrap().static_moment_x();

        // The default: 13 mm back from the covering's lip, which stands 2 mm
        // proud of the key front at 258 mm, so the arm is 247 mm.
        assert!((b.measuring_point_from_lip() - 0.013).abs() < 1e-15);
        assert!(
            (b.assembly().measuring_point() - (0.258 + 0.002 - 0.013)).abs() < 1e-12,
            "the default measuring point is 247 mm in front of the balance pin, not {} m",
            b.assembly().measuring_point()
        );

        let before = b.readout().unwrap();
        // The bench is built with 6 g of key friction weight taken at its own
        // measuring point, so its sheet reads the 6 g back. A friction radius
        // left at another point would read as another number here.
        assert!(
            (before.friction_weight_g - 6.0).abs() < 1e-12,
            "the standard bench reads {} g of friction, not the 6 g it was built with",
            before.friction_weight_g
        );
        let arm_before = lip - b.measuring_point_from_lip();
        b.set_measuring_point_from_lip(b.measuring_point_from_lip() + 0.007);
        let after = b.readout().unwrap();
        let arm_after = lip - b.measuring_point_from_lip();

        // The same static moment, read on each arm.
        for (fw, arm) in [
            (before.front_weight_g, arm_before),
            (after.front_weight_g, arm_after),
        ] {
            assert!(
                (fw * arm - moment_g_m).abs() < 1e-12,
                "{fw} g on a {arm} m arm is not the key's static moment, {moment_g_m} g·m"
            );
        }
        let expected = before.front_weight_g * arm_before / arm_after;
        assert!(
            (after.front_weight_g - expected).abs() < 1e-12,
            "7 mm further back, front weight read {} g against the lever's {expected} g",
            after.front_weight_g
        );

        // The key itself has not changed, and neither has its key front.
        assert_eq!(after.key_mass_g, before.key_mass_g);
        assert_eq!(after.key_height_mm, before.key_height_mm);
        assert_eq!(after.key_dip_mm, before.key_dip_mm);
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
    /// * the closed form on the frame, with the bearing at the pin:
    ///   `D = (b − M) / arm`, `U = (−b − M) / arm`, so `BW = −M / arm` and
    ///   `F = b / arm`;
    /// * key friction weight, run as a procedure: the friction torque over the
    ///   arm the key presents at rest, nose-up, `cos θ · arm + sin θ · h`;
    /// * the weight the settle solve puts on the key, on that same arm.
    ///
    /// The friction is set before the point moves, so F also checks that it is
    /// kept as a torque with its radius: read at the new point it is the same
    /// torque over a different arm, not the same grams.
    #[test]
    fn every_protocol_reads_the_same_measuring_point() {
        let mut b = KeyBench::standard();
        let key = b.assembly().key.clone();
        let lip = key.front_edge + key.keytop_overhang;

        let kf_g = 6.0;
        b.set_key_friction_weight(kf_g);
        let torque = kf_g * 1e-3 * (lip - b.measuring_point_from_lip());

        b.set_measuring_point_from_lip(MEASURING_POINT_HALF_INCH);
        let arm = lip - b.measuring_point_from_lip();
        assert!((b.measuring_point_from_lip() - MEASURING_POINT_HALF_INCH).abs() < 1e-15);
        assert!((b.assembly().measuring_point() - arm).abs() < 1e-15);

        let r = b.readout().unwrap();
        let m = key.mass_properties().unwrap().static_moment_x();
        let read_on = [
            ("front weight", 1000.0 * m / r.front_weight_g),
            ("key weight ratio", -key.capstan / r.key_ratio),
            ("down weight", 1000.0 * (torque - m) / r.down_weight_g),
            ("up weight", 1000.0 * (-torque - m) / r.up_weight_g),
            ("balance weight", -1000.0 * m / r.balance_weight_g),
            ("friction weight", 1000.0 * torque / r.friction_weight_g),
        ];
        for (what, on) in read_on {
            assert!(
                (on - arm).abs() < 1e-12,
                "{what} was read on a {on} m arm; the measuring point's is {arm} m"
            );
        }

        // Key friction weight, by the procedure. It reads the torque over the
        // arm the key presents on its cloth, from the same point. The search
        // halves a 200 g bracket 32 times, so the reading is within one final
        // interval, 200 / 2^32 g, of the least weight: that is the tolerance.
        let rest = b.assembly().sin_angle_on_back_rail().unwrap();
        let tilted =
            (1.0 - rest * rest).sqrt() * arm + rest * (key.stick_height + key.keytop_thickness);
        let kf = r
            .measured_kf_g
            .expect("the procedure runs on the standard bench");
        let resolution_g = 200.0 / 2f64.powi(32);
        assert!(
            (1000.0 * torque / kf - tilted).abs() < tilted * resolution_g / kf,
            "key friction weight was read on a {} m arm; the measuring point presents {tilted} m",
            1000.0 * torque / kf
        );

        // And the weight the settle solve puts on the key acts there too.
        let a = b.assembly();
        let w = 25.0;
        let from_weight = a.net_torque(rest, w).unwrap() - a.net_torque(rest, 0.0).unwrap();
        assert!(
            (from_weight - w * 1e-3 * tilted).abs() < 1e-15,
            "{w} g on the key acts on a {} m arm, not the measuring point's {tilted} m",
            from_weight / (w * 1e-3)
        );
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
        assert_eq!(b.measuring_point_from_lip(), b.assembly().key.keytop_length);
        // And a value that is not a distance changes nothing.
        b.set_measuring_point_from_lip(f64::NAN);
        assert_eq!(b.measuring_point_from_lip(), b.assembly().key.keytop_length);
    }

    /// A weight on the key moves with the measuring point, and the key comes
    /// to rest again under it.
    ///
    /// A back-heavy key rests on its cloth with 10.6 g on a measuring point
    /// 30 mm back from the lip: on that arm the weight cannot overcome the
    /// key's balance and its friction. Moved to 5 mm, the same weight acts
    /// on an arm 25 mm longer and takes the key down to its punching, with
    /// nobody touching the weight. The key leaves its cloth above 11.2 g at
    /// 30 mm and above 10.1 g at 5 mm, so 10.6 g is half a gram clear of
    /// either.
    #[test]
    fn moving_the_point_moves_the_weight_on_it_and_the_key_settles_again() {
        let mut b = KeyBench::standard();
        b.add_lead(-0.120, 30.0);
        b.set_key_friction_weight(2.0);
        b.set_measuring_point_from_lip(0.030);
        b.set_finger_weight(10.6);
        assert_eq!(b.finger_verdict(), Some(FingerVerdict::BackRailCloth));
        assert_eq!(b.depression(), 0.0, "the key must still be at rest");

        b.set_measuring_point_from_lip(0.005);
        assert_eq!(b.finger_verdict(), Some(FingerVerdict::FrontPunching));
        assert!(
            (b.depression() - 1.0).abs() < 1e-12,
            "the key must have gone down to its punching, not stayed at {}",
            b.depression()
        );
    }
}
