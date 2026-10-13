//! The key on its frame: rest position, key height, key dip.
//!
//! # Why this is geometry solved from targets, and not the other way round
//!
//! A technician does not choose a punching stack. He chooses a **key height**
//! and a **key dip** and then builds the stack that produces them. The data
//! model follows: the stored, normative value is the coordinate — the stack
//! thickness — and the target travels beside it as intent, with the achieved
//! value. [`Regulation::solve`] is that direction made explicit, and without it
//! the bench could only ask "what does this stack give?", never the question a
//! technician actually asks.
//!
//! # The one thing a bare key does that a key in a piano does not
//!
//! In the assembled action the wippen and hammer press down on the capstan,
//! behind the balance point, which holds the back of the key on the back rail
//! cloth and the front of the key up. **Remove the action and that is gone.** A
//! front-heavy key then simply sits at the bottom of its own dip, on the front
//! rail punching, and no amount of modelling will lift it. That is not a defect
//! of the simulation, it is what a leaded key does on the bench, and
//! [`KeyAssembly::free_rest`] reports which of the two stops the key has
//! actually found.

use super::{frame::FrameDesign, KeyDesign};
use crate::geometry::{circle_vs_line, ContactQuery, Frame, KeyBearing, Point, Rot};
use crate::measure::TouchweightProtocol;

/// A stop the key's underside can land on: a punching, or the back rail cloth.
///
/// # Why it has an extent and not just a position
///
/// A punching is a washer and the back rail cloth is a strip, so what the key
/// comes down on is a **face**, not a point. Where on that face the load sits
/// depends on how the key stands: parallel to it, the load is spread over the
/// whole of it; tilted, it moves to one edge. On the back rail that edge is
/// 12 mm from the middle — a twelfth of the arm, and a millimetre of key
/// height — so it is not a detail that can be left at the middle of the strip.
///
/// # Why the edge has a radius
///
/// A punching's bearing face and a key's underside are both flat, but the key
/// *rotates*, so two flats meet along an **edge** and not over a face. A real
/// punching's edge is rounded, and giving it a radius turns the pair into the
/// circle-against-line primitive the rest of the action already uses — with a
/// genuine gap function and a genuine lever arm — instead of a bespoke piece of
/// trigonometry that would have to be re-derived for every other stop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stop {
    /// Longitudinal position of the middle of the face, metres from the
    /// balance pin.
    pub at: f64,
    /// Half the face's extent along the key, metres.
    ///
    /// Zero makes the stop a point, which is what a punching is modelled as —
    /// [`KeyAssembly::punching_stop`] carries the reason.
    pub half_length: f64,
    /// Height of the bearing face above the keybed, metres.
    pub top: f64,
    /// Radius of the rounded edges, metres.
    pub edge_radius: f64,
}

impl Stop {
    /// The centres of the two rounded edges, the rear one first.
    pub fn edges(&self) -> [Point; 2] {
        let y = self.top - self.edge_radius;
        [
            Point::new(self.at - self.half_length, y),
            Point::new(self.at + self.half_length, y),
        ]
    }

    /// The flat of the face, as the two ends of a segment.
    pub fn face(&self) -> (Point, Point) {
        (
            Point::new(self.at - self.half_length, self.top),
            Point::new(self.at + self.half_length, self.top),
        )
    }
}

/// How the key and a stop are bearing on one another.
///
/// # Why all three, and not only the first
///
/// Face to face is what a well made keyboard does at the back rail, and it is
/// what the chamfer under the key's tail is cut for. It is **not** what every
/// keyboard does: put the wrong punchings under the balance rail and the key
/// stands at an angle its chamfer was not cut for, and then it bears on an
/// edge. That keyboard is badly regulated, not impossible — it plays, it wears
/// its cloth along one line, and a technician meets it. A model that admitted
/// only the flat case could not be used to design a keyboard, because it could
/// not show the designer what going wrong looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Touching {
    /// Face to face, the load spread over the whole of the stop.
    Flat,
    /// On the stop's own edge: the key's surface is tilted across it, so the
    /// load sits along the line where the felt ends.
    StopEdge,
    /// On the end of the key's own bearing surface, digging into the felt.
    KeyCorner,
}

/// Where the key meets a stop, and how.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Touch {
    /// Sine of the key's angle at the moment of contact.
    pub sin_theta: f64,
    /// Where the load acts, metres from the balance pin.
    ///
    /// The lever arm the statics see. It is the middle of the stop only when
    /// the two faces are parallel; otherwise it is an edge, and which edge
    /// depends on which way the key is tilted.
    pub at: f64,
    /// Slope of the key's bearing face over the stop's, rise per unit run.
    ///
    /// Zero is bedded flat. Positive means the key's face rises towards the
    /// player, so its rear end is the low one and that is what bears. A slope
    /// and not an angle, because `pa-core` may not reach a platform `atan`:
    /// the caller that wants degrees is the one that has one.
    pub slope: f64,
    /// Where the load sits on the key's own underside.
    ///
    /// The same contact in the key's frame, and not the same information as
    /// [`Self::at`]: on the chamfer the point has a **height** above the
    /// underside's flat, and any solve that places a rail from this contact
    /// has to carry it. Leaving it out costs the 1.6 mm the chamfer has risen
    /// by where the cloth meets it, straight onto the key height.
    pub on_key: Point,
    /// Which of the three ways the two are touching.
    pub how: Touching,
}

/// The slope below which a bearing counts as flat.
///
/// Over the back rail's 24 mm of cloth this is two and a half microns of
/// difference between one edge and the other — a thirtieth of the thinnest
/// paper in the drawer, and far below the felt's own compliance. A technician
/// calling a key *bedded* is making a statement of about this precision.
const FLAT_ENOUGH: f64 = 1e-4;

/// How far a contact may be from closed and still count as closed, metres.
///
/// A tenth of a micron, and it does two jobs: it is how far inside a stop the
/// key may be before it counts as through it, and how far a pair may be from
/// touching before the angle that was solved for it is rejected as not being
/// its contact at all.
///
/// A thousandth of the thinnest paper in the drawer, so it cannot admit a pair
/// that is not touching — and loose enough for the cancellation a near-tangent
/// quadratic suffers. It is a test applied to a closed-form root, not a
/// convergence criterion: nothing iterates towards it.
const CONTACT_TOLERANCE: f64 = 1e-7;

/// Solves `p·cos θ + q·sin θ = k` for `sin θ`.
///
/// Every contact condition in this module reduces to this — a perpendicular
/// distance, a height, a distance between two points, each written out in the
/// key's own rotation. Clearing the cosine by squaring gives a quadratic in
/// `sin θ`, so the answer is closed-form and there is no search here.
///
/// # The spurious root, and why it has to be rejected explicitly
///
/// Squaring admits the branch where the cosine comes out **negative**, which
/// geometrically is the stop sitting above the key's underside instead of
/// below it — the key already through the wood rather than resting on it. The
/// quadratic cannot tell the two apart, and neither can picking a side: when
/// the distance itself goes negative that heuristic selects the wrong branch.
/// Keeping it draws a key straight through its back rail while the bench
/// reports a comfortable gap, because the angle and the gap then answer
/// different questions.
///
/// So each root is put back into the **unsquared** condition and kept only if
/// it satisfies it.
fn sines_satisfying(p: f64, q: f64, k: f64) -> [Option<f64>; 2] {
    let a = p * p + q * q;
    if a < 1e-18 {
        return [None, None];
    }
    let b = -2.0 * q * k;
    let c = k * k - p * p;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return [None, None];
    }
    let root = disc.sqrt();
    let genuine = |s: f64| {
        let cos = (1.0 - s * s).max(0.0).sqrt();
        s.is_finite() && s.abs() <= 1.0 && (p * cos + q * s - k).abs() < 1e-9
    };
    let pair = [(-b + root) / (2.0 * a), (-b - root) / (2.0 * a)];
    [
        genuine(pair[0]).then_some(pair[0]),
        genuine(pair[1]).then_some(pair[1]),
    ]
}

/// The edge radius taken for a punching or a cloth, metres.
///
/// Half a millimetre: small enough that the contact point sits essentially over
/// the punching, large enough to be a real curve rather than a numerical
/// fiction. It is a shape parameter, not a tuning knob — [`KeyAssembly::travel`]
/// is tested to be insensitive to it at the micrometre the trade regulates to.
pub const PUNCHING_EDGE_RADIUS: f64 = 0.0005;

/// A way the frame and the key do not go together.
///
/// # Why this exists at all
///
/// The frame is **placed from the key**: the front rail stands under the key's
/// front mortise, the back rail under its chamfer. That keeps a keyboard
/// consistent without anybody typing a coordinate, and it is the right default
/// — but it is a default, not a law. A measured frame carries its own numbers,
/// a design under test is meant to be pushed until it breaks, and either can
/// arrive here holding a combination no keyboard could be built from.
///
/// So the rule places and this reports. A fault is **not an error**: the
/// assembly still answers every question put to it, and has to, because
/// watching a bad design misbehave is how the bench teaches. What a fault
/// changes is that the reader is told, in the trade's terms, which part of the
/// picture is not a piano.
///
/// Distances are metres, and each is *how far past the limit*, so a fault's
/// number is always positive and shrinking it is always the repair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fault {
    /// The front rail pin is not inside the key's front mortise.
    ///
    /// The key would be guided by a pin bearing against the wood beside its
    /// slot, or by nothing at all. On the bench this is what a key reads like
    /// when its length has been changed without the frame following.
    PinOutsideTheMortise {
        /// How far past the end of the slot the pin stands, metres.
        by: f64,
    },
    /// The back rail cloth reaches in front of where the tail's chamfer begins.
    ///
    /// The chamfer is cut so the key beds flat on this cloth; cloth in front of
    /// it can only ever be met by the kink, which is the one thing the chamfer
    /// exists to prevent.
    ClothInFrontOfTheChamfer {
        /// How far in front of the kink the cloth reaches, metres.
        by: f64,
    },
    /// The back rail cloth reaches past the end of the key.
    ///
    /// Then the key's own corner carries it, on a lever arm that is the key's
    /// length and not the rail's position.
    ClothBehindTheKey {
        /// How far past the tail the cloth reaches, metres.
        by: f64,
    },
    /// A punching stands partly off the flat of the key's underside.
    PunchingOffTheUnderside {
        /// Which rail, as the member id the view draws it under.
        rail: &'static str,
        /// How far past the end of the flat it reaches, metres.
        by: f64,
    },
}

/// Which stop the key is resting against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestingOn {
    /// Tail down on the back rail cloth — the position a key takes in the piano,
    /// where the action's weight on the capstan holds it there.
    BackRailCloth,
    /// Front down on the front rail punching: the bottom of the dip. A bare
    /// front-heavy key sits here.
    FrontRailPunching,
}

/// A key, its frame, and the bearing between them — and the protocol the key
/// is weighed by.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyAssembly {
    /// The key.
    pub key: KeyDesign,
    /// The frame it sits on.
    pub frame: FrameDesign,
    /// Where the key actually bears on the balance punching, and how high.
    ///
    /// Not the pin: Stanwood's technique of cutting the balance rail punching
    /// moves this deliberately, and modelling the pivot as the pin would make
    /// that technique inexpressible.
    pub bearing: KeyBearing,
    /// How the key is weighed: where on it every weight is centred.
    ///
    /// Held here, once, because every touchweight is a question asked of this
    /// assembly. The weight the settle solve puts on the key acts at the
    /// measuring point, the closed form's arm runs from the bearing to it, and
    /// the jig and key friction weight are built from it; each reads it
    /// through [`Self::measuring_point`], so none can be weighed at a point
    /// the others are not. It is not a dimension of the key, which is why it
    /// is not on [`KeyDesign`].
    pub protocol: TouchweightProtocol,
}

/// Where along the key a height or a dip is read.
///
/// # Why this is a parameter and not a constant
///
/// Key height and key dip are read **at the key front**, with a rule or a dip
/// block standing on the keybed. Touchweights are read at the measuring point,
/// a protocol distance back from the covering's lip, which stands proud of the
/// key front. Three of the five observables this bench reports are therefore
/// taken at *two different points*, and the difference is not small: on the
/// default key the two arms stand in the ratio 258 : 247, so the same travel
/// reads 4 % apart.
///
/// Naming the datum turns that into something a test can check — and something
/// a reader can see — instead of a constant buried in a formula that happens
/// to be right today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadAt {
    /// The key front, the face the covering's lip overhangs. Where a rule or
    /// a dip block sits.
    FrontEdge,
    /// The measuring point, [`KeyAssembly::protocol`]'s distance back from the
    /// covering's lip. Where every touchweight is taken.
    MeasuringPoint,
}

/// The key's rest and bottom attitudes, and the trade quantities they give.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Travel {
    /// Sine of the key angle at rest. Positive means front-down.
    pub sin_rest: f64,
    /// Sine of the key angle at the bottom of the dip.
    pub sin_bottom: f64,
    /// Height of the key's top surface at the front edge, at rest, metres.
    pub key_height: f64,
    /// Vertical travel of the front edge between rest and bottom, metres.
    pub key_dip: f64,
}

impl KeyAssembly {
    /// Position of the effective bearing along the key, metres from the balance
    /// pin, signed forwards.
    fn bearing_x(&self) -> f64 {
        -self.bearing.offset_from_pin
    }

    /// Height of the bearing above the keybed, metres.
    fn bearing_y(&self) -> f64 {
        self.frame.balance_bearing_height()
    }

    /// The front rail punching, as a stop.
    ///
    /// **A point, although the washer is 22 mm across.** The punching is felt,
    /// and the key is regulated to land on it level — that is what levelling
    /// the key at the bottom of its travel means — so the load spreads over
    /// the whole washer and its resultant is in the middle. A rigid strip
    /// would instead throw the whole load onto the forward edge the moment the
    /// key is a tenth of a degree out of level: an 11 mm step in the lever arm
    /// that felt does not take.
    ///
    /// So the simplification costs in proportion to how far out of level the
    /// key lands, and nothing when it is regulated — and the bench reports
    /// that angle, so a reader can see what it is worth. The back rail cloth
    /// is the other way round: the key stands on it nose-up *by design*, so
    /// there the extent is the whole point.
    pub fn punching_stop(&self) -> Stop {
        Stop {
            at: self.frame.front_rail,
            half_length: 0.0,
            top: self.frame.front_bearing_height(),
            edge_radius: PUNCHING_EDGE_RADIUS,
        }
    }

    /// The front rail pin bottoming in the key's mortise, as a stop.
    ///
    /// A key does not only stop on felt. It also stops when the pin standing
    /// in its front mortise reaches the top of that slot, and a frame whose
    /// pin is too proud, or whose punchings are too thin, reaches that first.
    ///
    /// Expressed at the same place as the punching, with its top at
    /// `pin tip − mortise depth`: that is the height the key's underside
    /// cannot go below without the pin coming through the wood above it.
    /// `None` where the rail carries no pin.
    pub fn mortise_stop(&self) -> Option<Stop> {
        self.frame.sections.front.pin?;
        let tip = self.frame.front_rail_height + self.frame.sections.front.pin_height;
        Some(Stop {
            at: self.frame.front_rail,
            // A pin in a mortise bears on its tip, which is a point however
            // wide the slot is.
            half_length: 0.0,
            top: tip - self.key.front_mortise_depth,
            edge_radius: PUNCHING_EDGE_RADIUS,
        })
    }

    /// Whichever of the two the key meets first on its way down.
    ///
    /// Both bound the same coordinate from the same side, so there is no new
    /// complementarity problem here: the binding one is simply the higher, and
    /// the key stops there. When it is the mortise, the dip the bench reports
    /// is genuinely shorter — which is the right answer and the one a
    /// technician would get with a dip block.
    pub fn front_stop(&self) -> Stop {
        let punching = self.punching_stop();
        match self.mortise_stop() {
            Some(mortise) if mortise.top > punching.top => mortise,
            _ => punching,
        }
    }

    /// Whether it is the mortise rather than the punching that stops the key.
    pub fn stopped_by_the_mortise(&self) -> bool {
        self.mortise_stop()
            .is_some_and(|m| m.top > self.punching_stop().top)
    }

    /// The back rail cloth, as a stop.
    ///
    /// At the **back rail**, which is where the cloth is — not at the key's tail
    /// end. The tail is where the key stops being wood, not where it touches
    /// anything, and on this key that is 90 mm apart: taking the tail as the
    /// lever arm would be wrong by a fifth of the arm.
    ///
    /// It carries the rail's own width, read from the same expression the view
    /// draws the strip with, because the whole question of whether the key beds
    /// flat is a question about that width.
    pub fn back_stop(&self) -> Stop {
        let section = &self.frame.sections.back;
        Stop {
            at: section.centre_x(self.frame.back_rail),
            half_length: section.half_length,
            top: self.frame.back_bearing_height(),
            edge_radius: PUNCHING_EDGE_RADIUS,
        }
    }

    /// Everything about this assembly that no keyboard could be built from.
    ///
    /// Empty for a frame that goes with its key. Each entry carries how far
    /// past the limit it is, so the bench can say *"the pin misses the mortise
    /// by 63 mm"* rather than merely that something is wrong — see [`Fault`]
    /// for why this reports instead of refusing.
    ///
    /// Ordered the way a technician would walk the key: front first, because
    /// that is the end he is holding.
    pub fn faults(&self, kind: crate::parts::frame::KeyKind) -> Vec<Fault> {
        let mut out = Vec::new();
        let (flat_from, flat_to) = self.key.underside_flat();

        // The pin has to stand within the slot cut for it, with its own length
        // inside: a pin whose end is past the end of the mortise is bearing on
        // wood, not running in a slot.
        if let Some(pin) = self.frame.sections.front.pin {
            let (mortise_back, mortise_front) = self.key.front_mortise_span();
            let half = 0.5 * pin.length();
            let pin_x = self.frame.sections.front.pin_x(self.frame.front_rail, kind);
            let past = (mortise_back - (pin_x - half)).max((pin_x + half) - mortise_front);
            if past > 0.0 {
                out.push(Fault::PinOutsideTheMortise { by: past });
            }
        }

        // Both punchings bear on the flat, and a washer is wide.
        for (rail, section, natural_x) in [
            ("balance_rail_punching", &self.frame.sections.balance, 0.0),
            (
                "front_rail_punching",
                &self.frame.sections.front,
                self.frame.front_rail,
            ),
        ] {
            let x = section.pin_x(natural_x, kind);
            let r = section.punching_radius;
            let past = (flat_from - (x - r)).max((x + r) - flat_to);
            if past > 0.0 {
                out.push(Fault::PunchingOffTheUnderside { rail, by: past });
            }
        }

        // And the cloth has to fall inside the chamfer, whole.
        let back = self.back_stop();
        let [cloth_back, cloth_front] = back.edges();
        if cloth_front.x > flat_from {
            out.push(Fault::ClothInFrontOfTheChamfer {
                by: cloth_front.x - flat_from,
            });
        }
        if cloth_back.x < self.key.tail_end {
            out.push(Fault::ClothBehindTheKey {
                by: self.key.tail_end - cloth_back.x,
            });
        }
        out
    }

    /// Whether any of the key's underside is inside a stop at this angle.
    ///
    /// Not the same question as the gap, and it cannot be answered with one:
    /// [`crate::geometry::circle_vs_line`] returns an **unsigned** distance, so
    /// a key driven deep through a punching reports the same small gap as one
    /// resting on it, and past the edge's centre a growing one. That is
    /// harmless where the solver uses it — at contact, where the sign is not
    /// in doubt — and useless for sorting candidate angles, half of which are
    /// angles the key could only reach by passing through felt.
    ///
    /// So this is signed arithmetic, over the three things that can be inside
    /// something: a corner of the key under the stop's face, a corner of the
    /// key inside a rounded edge, and an edge up inside the key's own wood.
    fn key_is_inside(&self, stop: Stop, sin_theta: f64) -> bool {
        let key = self.key_frame(sin_theta);
        let (x0, x1) = (stop.at - stop.half_length, stop.at + stop.half_length);
        for v in self.key.underside_corners() {
            let w = key.world(v);
            if w.x >= x0 && w.x <= x1 && w.y < stop.top - CONTACT_TOLERANCE {
                return true;
            }
            for c in stop.edges() {
                let (dx, dy) = (w.x - c.x, w.y - c.y);
                if (dx * dx + dy * dy).sqrt() < stop.edge_radius - CONTACT_TOLERANCE {
                    return true;
                }
            }
        }
        for (a, b) in self.key.underside_segments() {
            let (pa, pb) = (key.world(a), key.world(b));
            let (ux, uy) = (pb.x - pa.x, pb.y - pa.y);
            let len2 = ux * ux + uy * uy;
            if len2 < 1e-24 {
                continue;
            }
            let len = len2.sqrt();
            // The underside's outward normal points away from the wood, which
            // is above it.
            let (nx, ny) = (uy / len, -ux / len);
            for c in stop.edges() {
                let t = ((c.x - pa.x) * ux + (c.y - pa.y) * uy) / len2;
                if !(0.0..=1.0).contains(&t) {
                    continue;
                }
                if nx * (c.x - pa.x) + ny * (c.y - pa.y) < stop.edge_radius - CONTACT_TOLERANCE {
                    return true;
                }
            }
        }
        false
    }

    /// The angle each pair of features would close at.
    ///
    /// Thirteen pairs: the stop's two edges against each of the two faces of
    /// the underside, each corner of the underside against the flat of the
    /// stop's face, and each corner against each edge. All thirteen are
    /// enumerated because *which* one binds is exactly what changes between a
    /// regulated keyboard and a badly regulated one, and the point of this
    /// module is that both are keyboards.
    ///
    /// Each pair yields at most one angle: of its two roots, the one on the
    /// stop's own side of the bearing — the other turns the key away onto its
    /// far side.
    fn touch_angles(&self, stop: Stop) -> [Option<f64>; 13] {
        let (bx, by) = (self.bearing_x(), self.bearing_y());
        let mut out = [None; 13];
        let mut n = 0;
        let mut push = |p: f64, q: f64, k: f64, feature_x: f64| {
            let behind = feature_x < bx;
            out[n] = sines_satisfying(p, q, k)
                .into_iter()
                .flatten()
                .reduce(|a, s| if behind { a.min(s) } else { a.max(s) });
            n += 1;
        };

        // An edge against a face: the perpendicular distance from the edge's
        // centre to the face is one radius.
        for centre in stop.edges() {
            let (ex, ey) = (centre.x - bx, centre.y - by);
            for (a, b) in self.key.underside_segments() {
                let (ux, uy) = (b.x - a.x, b.y - a.y);
                let len = (ux * ux + uy * uy).sqrt();
                if len < 1e-12 {
                    push(0.0, 0.0, 0.0, centre.x);
                    continue;
                }
                let (nx, ny) = (uy / len, -ux / len);
                push(
                    nx * ex + ny * ey,
                    ny * ex - nx * ey,
                    stop.edge_radius - nx * (bx - a.x) + ny * a.y,
                    centre.x,
                );
            }
        }

        // A corner against the flat of the face: the corner's height is the
        // face's height.
        for v in self.key.underside_corners() {
            push(v.y, bx - v.x, stop.top - by, stop.at);
        }

        // A corner against an edge: corner to corner, which is what a key
        // whose bearing face ends over the felt comes down on.
        for v in self.key.underside_corners() {
            let (vx, vy) = (v.x - bx, v.y);
            for centre in stop.edges() {
                let (ex, ey) = (centre.x - bx, centre.y - by);
                push(
                    -2.0 * (ex * vx + ey * vy),
                    -2.0 * (ex * vy - ey * vx),
                    stop.edge_radius * stop.edge_radius - (ex * ex + ey * ey) - (vx * vx + vy * vy),
                    centre.x,
                );
            }
        }
        out
    }

    /// Where the key's underside comes down onto a stop, and how.
    ///
    /// # Both cases, because both are keyboards
    ///
    /// Two flats meet either face to face or on an edge. Face to face is what
    /// a regulated back rail does, and it is what the chamfer under the key's
    /// tail is cut for; on an edge is what a keyboard with the wrong punchings
    /// does. So this solves for both and reports which, rather than assuming
    /// the regulated one and drawing the other wrong.
    ///
    /// Closed form throughout: each pair of features closes at a root of a
    /// quadratic, the binding one is the first the key reaches, and a
    /// candidate is kept only if the key can get there without passing
    /// through the felt. No search, no tolerance on the answer.
    pub fn first_touch(&self, stop: Stop) -> Option<Touch> {
        let behind = stop.at < self.bearing_x();
        let mut best: Option<f64> = None;
        for candidate in self.touch_angles(stop) {
            let Some(s) = candidate else { continue };
            // Only a candidate that improves on the best is worth the
            // interpenetration check, which is the expensive half of this.
            if let Some(b) = best {
                if (behind && s <= b) || (!behind && s >= b) {
                    continue;
                }
            }
            // Two ways a candidate angle is not a contact, and both have to be
            // tested. The pair may close **off the end** of one of the two
            // features — the arithmetic above solves for the surfaces as if
            // they ran on for ever, so a punching's edge can be tangent to the
            // line of the underside 200 mm beyond where the wood stops. And
            // the key may have had to pass through the felt to get there.
            if self.stop_contact(stop, s).gap.abs() > CONTACT_TOLERANCE
                || self.key_is_inside(stop, s)
            {
                continue;
            }
            best = Some(s);
        }
        let sin_theta = best?;
        let (q, how, slope) = self.contact_detail(stop, sin_theta);
        let cos = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let (bx, by) = (self.bearing_x(), self.bearing_y());
        let (dx, dy) = (q.point.x - bx, q.point.y - by);
        Some(Touch {
            sin_theta,
            at: q.point.x,
            // Back into the key's frame: the same rotation the other way.
            on_key: Point::new(bx + cos * dx - sin_theta * dy, sin_theta * dx + cos * dy),
            slope,
            how,
        })
    }

    /// Where the key goes when its two stops leave it nowhere to turn.
    ///
    /// Returns the attitude and **how far the key has lifted off its balance
    /// punching**, metres. Pure kinematics, deliberately: with the bearing
    /// modelled as a pivot there is no statics here — [`Self::settle`] refuses
    /// such a frame, and must — but the key still has a definite place to be,
    /// and drawing it anywhere else means drawing wood through wood.
    ///
    /// This treats the underside as one straight line and the edges as circles
    /// of one radius. Where a stop meets the chamfer instead of the flat, the
    /// key is drawn clear of that stop by up to about 0.8 mm: a known defect,
    /// to be replaced by the general solver. Under that assumption the resting line is their **common tangent from
    /// above**: parallel to the line of centres, offset by one radius along the
    /// upward normal. Closed form, like every other contact here — but over
    /// every pair of edges rather than one, because a strip of cloth has two
    /// and which pair carries the key depends on how the heights stand. The
    /// pair that carries it is the one whose tangent leaves every other edge at
    /// or below it.
    ///
    /// One radius for all four edges, which is what this frame has. A tangent
    /// to two circles of *different* radii is not parallel to their line of
    /// centres, and the construction below would quietly be wrong.
    pub fn riding_on_both_stops(&self) -> Option<(f64, f64)> {
        let (front, back) = (self.front_stop(), self.back_stop());
        let r = front.edge_radius;
        for f in front.edges() {
            for b in back.edges() {
                let (dx, dy) = (f.x - b.x, f.y - b.y);
                let len = (dx * dx + dy * dy).sqrt();
                if len < 1e-12 {
                    continue;
                }
                // The underside runs along (cos, -sin), so the line of centres
                // gives both at once.
                let (cos, sin) = (dx / len, -dy / len);
                if cos.abs() < 1e-12 {
                    continue;
                }
                // One radius up the normal (sin, cos) puts the line onto the
                // circles.
                let (px, py) = (f.x + r * sin, f.y + r * cos);
                let clears =
                    |c: &Point| sin * (c.x - px) + cos * (c.y - py) <= -r + CONTACT_TOLERANCE;
                if !front.edges().iter().chain(back.edges().iter()).all(clears) {
                    continue;
                }
                let lift = py - (sin / cos) * (self.bearing_x() - px) - self.bearing_y();
                return Some((sin, lift));
            }
        }
        None
    }

    /// Places the key at a given angle, for a contact query.
    fn key_frame(&self, sin_theta: f64) -> Frame {
        let c = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        Frame {
            pivot_local: Point::new(self.bearing_x(), 0.0),
            pivot_world: Point::new(self.bearing_x(), self.bearing_y()),
            // Front-down is a clockwise turn in this frame, so the rotation
            // carries `-sin_theta`.
            rot: Rot::from_cos_sin(c, -sin_theta),
            omega: 0.0,
        }
    }

    /// The contact between a stop and the key's underside, at a given angle.
    ///
    /// The normal points from the key towards the stop, and the gap is positive
    /// when they are apart. `d_gap_d_theta_b` is the lever arm on the key — the
    /// same quantity, from the same primitive, that every other contact in the
    /// action will use.
    pub fn stop_contact(&self, stop: Stop, sin_theta: f64) -> ContactQuery {
        self.contact_detail(stop, sin_theta).0
    }

    /// The governing contact, with the geometry that says what is bearing.
    ///
    /// A strip of felt and a kinked underside can meet through several pairs of
    /// features, and which one governs is settled by the smallest gap. Taking
    /// the minimum is not a heuristic — it is what a rigid contact *is* — and
    /// the gap it gives is continuous, with a kink exactly where the load
    /// transfers from one pair to the next. That kink is a physical event: the
    /// moment a key tilting over its cloth stops bearing on the felt's edge and
    /// starts bearing on its own corner.
    ///
    /// The faces come from the key's own outline, as
    /// [`KeyDesign::underside_segments`] — never a line through it, which would
    /// contact the flat's extension under wood that is not there.
    fn contact_detail(&self, stop: Stop, sin_theta: f64) -> (ContactQuery, Touching, f64) {
        let fixed = Frame::at(Point::new(0.0, 0.0));
        let key = self.key_frame(sin_theta);
        let cos = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let segs = self.key.underside_segments();

        // The slope of a face of the underside, carried round with the key.
        let slope_of = |(a, b): (Point, Point)| {
            let (ux, uy) = (b.x - a.x, b.y - a.y);
            let (wx, wy) = (cos * ux + sin_theta * uy, cos * uy - sin_theta * ux);
            if wx.abs() < 1e-15 {
                0.0
            } else {
                wy / wx
            }
        };
        // Whether the stop's edge has run off the end of a face of the key, in
        // which case what bears is the key's corner and not the felt's edge.
        let ran_off = |centre: Point, a: Point, b: Point| {
            let (pa, pb) = (key.world(a), key.world(b));
            let (ux, uy) = (pb.x - pa.x, pb.y - pa.y);
            let len2 = ux * ux + uy * uy;
            if len2 < 1e-24 {
                return true;
            }
            let t = ((centre.x - pa.x) * ux + (centre.y - pa.y) * uy) / len2;
            !(0.0..=1.0).contains(&t)
        };

        let mut best: Option<(ContactQuery, Touching, f64)> = None;
        let mut offer = |c: (ContactQuery, Touching, f64)| {
            best = match best {
                Some(b) if b.0.gap <= c.0.gap => Some(b),
                _ => Some(c),
            };
        };

        // Each rounded edge against each face of the underside.
        for centre in stop.edges() {
            for (a, b) in segs {
                let slope = slope_of((a, b));
                let how = if ran_off(centre, a, b) {
                    Touching::KeyCorner
                } else if slope.abs() < FLAT_ENOUGH {
                    Touching::Flat
                } else {
                    Touching::StopEdge
                };
                offer((
                    circle_vs_line(&fixed, centre, stop.edge_radius, &key, a, b),
                    how,
                    slope,
                ));
            }
        }

        // And each corner of the underside against the flat of the face. The
        // roles are reversed in that call — the key carries the point and the
        // stop the line — so the query is turned back round afterwards. What
        // makes that exact rather than approximate: reversing the normal and
        // the sign of a cross product together leaves it unchanged.
        let (fa, fb) = stop.face();
        for (i, v) in self.key.underside_corners().into_iter().enumerate() {
            let q = circle_vs_line(&key, v, 0.0, &fixed, fa, fb);
            let turned = ContactQuery {
                gap: q.gap,
                normal: Point::new(-q.normal.x, -q.normal.y),
                point: q.point,
                d_gap_d_theta_a: q.d_gap_d_theta_b,
                d_gap_d_theta_b: q.d_gap_d_theta_a,
                v_tangential: -q.v_tangential,
            };
            // The face that would bed is the one on the stop's side of the
            // corner: a corner belongs to two, and the one hanging out over
            // nothing says nothing about bedding.
            let seg = match i {
                0 => segs[0],
                2 => segs[1],
                _ if stop.at < key.world(v).x => segs[0],
                _ => segs[1],
            };
            offer((turned, Touching::KeyCorner, slope_of(seg)));
        }

        best.expect("the underside has two faces and every stop has two edges")
    }

    /// The angle at which the key's underside closes on a stop.
    ///
    /// The angle alone, for the callers that want nothing else;
    /// [`Self::first_touch`] carries where the load lands and how the two are
    /// bearing.
    ///
    /// `None` when the stop is unreachable, which an edit can produce — and it
    /// is a real answer, not a failure: a rail raised above the plane the key
    /// turns in cannot be *landed on*, only passed through.
    pub fn sin_angle_on_stop(&self, stop: Stop) -> Option<f64> {
        self.first_touch(stop).map(|t| t.sin_theta)
    }

    /// The angle at which the tail meets the back rail cloth.
    pub fn sin_angle_on_back_rail(&self) -> Option<f64> {
        self.sin_angle_on_stop(self.back_stop())
    }

    /// The angle at which the key's underside meets the front rail punching:
    /// the bottom of the dip.
    pub fn sin_angle_on_front_rail(&self) -> Option<f64> {
        self.sin_angle_on_stop(self.front_stop())
    }

    /// **The measuring point**, metres in front of the balance pin: the
    /// covering's lip less the protocol's distance.
    ///
    /// The only route to it. The jig's arm, the closed form's arm, the weight
    /// the settle solve puts on the key, key friction weight and the drawn
    /// marker all come through here, so changing the protocol moves every one
    /// of them by the same distance, and none can be left at the old point.
    pub fn measuring_point(&self) -> f64 {
        self.protocol.measuring_point(&self.key)
    }

    /// Longitudinal position of a reading datum, metres from the balance pin.
    pub fn datum_x(&self, at: ReadAt) -> f64 {
        match at {
            ReadAt::FrontEdge => self.key.front_edge,
            ReadAt::MeasuringPoint => self.measuring_point(),
        }
    }

    /// **Key height, by the trade's protocol.**
    ///
    /// *The height of the top of the key above the keybed, read at the front
    /// of the key.* The keybed is the datum — `y = 0` in this frame — and the
    /// top surface is the keystick plus its covering. Steinway New York
    /// specifies 65.88 mm; Hamburg and Kawai publish their own. [V]
    ///
    /// Not a numerical probe: the same rule a technician stands on the keybed.
    pub fn key_height_at(&self, at: ReadAt, sin_theta: f64) -> f64 {
        let c = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let dx = self.datum_x(at) - self.bearing_x();
        let dy = self.key.stick_height + self.key.keytop_thickness;
        self.bearing_y() - sin_theta * dx + c * dy
    }

    /// **Key dip, by the trade's protocol.**
    ///
    /// *The total travel of the front of the key*, from where it rests to
    /// where it stops. Both ends are **contacts**: the key tail on the back
    /// rail cloth, and the key's underside on the front rail punching. Neither
    /// is a stored number — move the punching and this moves with it, which is
    /// what `the_dip_follows_the_contact_and_not_the_target` checks.
    pub fn key_dip_at(&self, at: ReadAt) -> Option<f64> {
        let s_rest = self.sin_angle_on_back_rail()?;
        let s_bottom = self.sin_angle_on_front_rail()?;
        Some(self.key_height_at(at, s_rest) - self.key_height_at(at, s_bottom))
    }

    /// Rest and bottom attitudes, and the two settings they define.
    ///
    /// Rest is taken with the action fitted — tail on the back rail cloth —
    /// because that is where both key height and key dip are specified. What a
    /// *bare* key does is a different question, answered by [`Self::free_rest`].
    pub fn travel(&self) -> Option<Travel> {
        let sin_rest = self.sin_angle_on_back_rail()?;
        let sin_bottom = self.sin_angle_on_front_rail()?;
        Some(Travel {
            sin_rest,
            sin_bottom,
            // Both read by the protocol, at the front of the key, from the two
            // contacts — not reconstructed from a lever arm times an angle.
            key_height: self.key_height_at(ReadAt::FrontEdge, sin_rest),
            key_dip: self.key_dip_at(ReadAt::FrontEdge)?,
        })
    }

    /// Which stop a **bare** key finds, with no action on the capstan.
    ///
    /// Decided by the sign of the static moment about the bearing: a key whose
    /// centre of gravity lies in front of the bearing falls onto the front rail
    /// punching, which is the bottom of its dip. `None` for a degenerate design.
    pub fn free_rest(&self) -> Option<RestingOn> {
        let props = self.key.mass_properties()?;
        Some(if props.com.x > self.bearing_x() {
            RestingOn::FrontRailPunching
        } else {
            RestingOn::BackRailCloth
        })
    }

    /// Static moment of the key about the **effective bearing**, kg·m.
    ///
    /// About the bearing, not the pin: this is the in-action lever, and it is
    /// the one the balance weight is built on. The weighing jig uses the pin,
    /// and the two differ by `offset_from_pin` — which is exactly the gap
    /// Stanwood's punching-cutting technique exploits.
    pub fn static_moment_about_bearing(&self) -> Option<f64> {
        let p = self.key.mass_properties()?;
        Some(p.mass * (p.com.x - self.bearing_x()))
    }

    /// Height of the key's centre of mass above the keybed, at a given angle.
    ///
    /// The potential energy divided by `m g`. Exposed because the virtual-work
    /// test needs to differentiate it, and a test that cannot see the energy
    /// cannot check that the torque is its gradient.
    pub fn centre_of_mass_height(&self, sin_theta: f64) -> Option<f64> {
        let p = self.key.mass_properties()?;
        let c = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let dx = p.com.x - self.bearing_x();
        Some(self.bearing_y() - sin_theta * dx + c * p.com.y)
    }

    /// Gravity's generalized force on the key, as a **mass-moment** in kg·m —
    /// the torque divided by `g`.
    ///
    /// Positive turns the key front-down, which is what a front-heavy key does.
    pub fn gravity_torque(&self, sin_theta: f64) -> Option<f64> {
        let p = self.key.mass_properties()?;
        let c = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let dx = p.com.x - self.bearing_x();
        // -d(y_com)/dθ, with y_com = by - sin θ·dx + cos θ·dy.
        Some(p.mass * (c * dx + sin_theta * p.com.y))
    }

    /// Lever arm from the effective bearing to the measuring point, metres,
    /// with the key level.
    pub fn measuring_arm(&self) -> f64 {
        self.measuring_arm_at(0.0)
    }

    /// The arm the measuring point presents at a given attitude, metres.
    ///
    /// Not the longitudinal distance: a tilted key offers
    /// `cos θ · dx + sin θ · dy`, the second term being the height of the key
    /// top above the bearing. It is a millimetre on a quarter of a metre — and
    /// it is exactly the difference between a Key Friction Weight that
    /// round-trips through its own protocol and one that is half a percent
    /// out, because a correctly set key is nose-up at rest by the whole dip.
    pub fn measuring_arm_at(&self, sin_theta: f64) -> f64 {
        let c = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let dx = self.measuring_point() - self.bearing_x();
        let dy = self.key.stick_height + self.key.keytop_thickness;
        c * dx + sin_theta * dy
    }
}

/// Trade targets for the key, and the geometry that meets them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Regulation {
    /// Key height: top of the key at the front edge above the keybed, metres.
    pub key_height: f64,
    /// Key dip: travel of the front edge, metres.
    pub key_dip: f64,
}

impl Regulation {
    /// Solves the two punching stacks that meet these targets.
    ///
    /// Returns the balance and front stack **totals**, in metres. How the
    /// technician splits each between paper and felt is his business and the
    /// rest geometry does not care; what it changes is the compliance, which
    /// this rigid model leaves out.
    ///
    /// The two equations are the two contacts. At rest the tail sits on the back
    /// rail cloth and the key top reaches the target height; at the bottom the
    /// underside sits on the front rail punching, the front edge having fallen
    /// by the target dip. Both are linear in the unknowns once written in terms
    /// of `sin θ`, so this is a closed-form solve and not a search.
    ///
    /// # Why it is placed twice
    ///
    /// Where the key bears on the cloth is **not** the middle of the strip. A
    /// key stands nose-up at rest, so it meets the cloth at the strip's back
    /// edge, 12 mm further from the pin — and a lever arm wrong by 12 mm in 150
    /// puts the dip 0.8 mm off its target, which is eight times the tolerance
    /// the trade works to. So the placement is made twice: once with the middle
    /// of the strip, to have a frame to ask, and once with the bearing that
    /// frame really produces. Two passes, both closed form, nothing iterating
    /// and nothing to converge.
    ///
    /// What stays approximate on purpose is the **small-angle form** of the two
    /// equations. It is used only to *place* the punchings; what the bench then
    /// reads back is the exact contact of [`KeyAssembly::first_touch`], so the
    /// residual shows up as a dip that misses its target by a few hundredths of
    /// a millimetre.
    ///
    /// `None` when the placement equations have no solution on this geometry.
    /// It does not see the front pin bottoming in its mortise: a dip the pin
    /// forbids is returned as solved and shows as a miss on the sheet, a known
    /// defect the general solver replaces.
    pub fn solve(
        &self,
        key: &KeyDesign,
        frame: &FrameDesign,
        bearing_offset_from_pin: f64,
    ) -> Option<(f64, f64)> {
        let x_b = -bearing_offset_from_pin;
        let nominal = Point::new(frame.sections.back.centre_x(frame.back_rail), 0.0);
        let (balance, front) = self.place(key, frame, x_b, nominal)?;

        // The frame that placement produces, asked where the key really bears.
        // Only the stack's total matters here, so it goes in as paper.
        let mut probed = frame.clone();
        probed.balance_punching = crate::parts::frame::PunchingStack::new(balance, 0.0);
        probed.front_punching = crate::parts::frame::PunchingStack::new(front, 0.0);
        let probe = KeyAssembly {
            key: key.clone(),
            frame: probed,
            bearing: KeyBearing {
                offset_from_pin: bearing_offset_from_pin,
                stack_height: balance,
            },
            // Never weighed: the probe is asked where the key bears on its
            // cloth, which is geometry, and nothing about it reads the
            // measuring point.
            protocol: TouchweightProtocol::default(),
        };
        self.place(
            key,
            frame,
            x_b,
            probe.first_touch(probe.back_stop())?.on_key,
        )
    }

    /// Places the two stacks, given the arm the key bears on the cloth at.
    ///
    /// The small-angle form of the two contact equations, solved for the two
    /// stack heights. Split out because it is evaluated twice — see
    /// [`Self::solve`] — and a formula written twice is a formula that drifts.
    fn place(
        &self,
        key: &KeyDesign,
        frame: &FrameDesign,
        x_b: f64,
        back_bears: Point,
    ) -> Option<(f64, f64)> {
        let back_lever = back_bears.x - x_b;
        let front_lever = key.front_edge - x_b;
        let frp_lever = frame.front_rail - x_b;
        if front_lever.abs() < 1e-9 || back_lever.abs() < 1e-9 {
            return None;
        }

        // Rest: the key top at the front edge reaches the target height.
        //   h_bearing - front_lever * sin_rest + stick + keytop = key_height
        // Rest: the point of the underside that bears sits on the cloth. That
        // point is not necessarily on the flat — under the tail it is on the
        // chamfer, which has risen by then — so its own height counts.
        //   h_bearing - back_lever * sin_rest + y_bears = h_back
        // Subtract to eliminate h_bearing.
        let h_back = frame.back_bearing_height();
        let top = key.stick_height + key.keytop_thickness;
        let sin_rest = (h_back + top - self.key_height - back_bears.y) / (front_lever - back_lever);
        let h_bearing = h_back + back_lever * sin_rest - back_bears.y;

        // Bottom: the front edge has fallen by the dip.
        let sin_bottom = sin_rest + self.key_dip / front_lever;
        let h_front_bearing = h_bearing - frp_lever * sin_bottom;

        let balance_stack = h_bearing - frame.balance_rail_height;
        let front_stack = h_front_bearing - frame.front_rail_height;
        if !(balance_stack.is_finite() && front_stack.is_finite())
            || balance_stack < 0.0
            || front_stack < 0.0
        {
            return None;
        }
        Some((balance_stack, front_stack))
    }
}

#[cfg(test)]
mod tests {
    use super::super::frame::PunchingStack;
    use super::*;

    fn regulated() -> KeyAssembly {
        let key = KeyDesign::default();
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target
            .solve(&key, &frame, 0.0)
            .expect("the default targets must be reachable");
        frame.balance_punching = PunchingStack::new(balance - 0.0020, 0.0020);
        frame.front_punching = PunchingStack::new(front - 0.0030, 0.0030);
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: balance,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// Level 4: the regulated key lands within the small-angle placement's
    /// own error of its targets.
    ///
    /// The targets go in as trade specifications. What comes back out is read
    /// from the exact contacts: the vertical distance the front edge moves
    /// between them, and the height of the key top at rest.
    #[test]
    fn the_regulated_key_lands_within_the_placement_error_of_its_targets() {
        let a = regulated();
        let t = a.travel().expect("a regulated key has a travel");

        // The punchings are placed by a closed-form small-angle solve, and what
        // is read back is the exact circle-on-line contact, so the two may
        // differ by a few micrometres. An exact placement would be as good, so
        // only the bound is asserted. The stored, normative value is the
        // coordinate, and the target travels beside it with the achieved
        // value: you shim, you measure, and the measurement is what counts.
        let miss = (t.key_dip - 0.0102).abs();

        // How far apart they may be is **derived from the attitude**, because
        // that is what the placement approximates away. A small-angle solve
        // drops terms of order θ², and the length they multiply is the key
        // top's height above the bearing — that is where turning the key
        // couples into a vertical reading. The bound therefore grows with how
        // far from level the key stands, which on a correctly set frame is the
        // whole of the dip and not a rounding angle.
        let s = a
            .sin_angle_on_back_rail()
            .unwrap()
            .abs()
            .max(a.sin_angle_on_front_rail().unwrap().abs());
        let dropped = s * s * (a.key.stick_height + a.key.keytop_thickness);
        assert!(
            miss < dropped,
            "the dip missed its target by {miss} m, more than the {dropped} m \
             the small-angle placement can account for"
        );

        let height_miss = (t.key_height - 0.0650).abs();
        assert!(
            height_miss < dropped,
            "key height missed by {height_miss} m, more than {dropped}"
        );
    }

    /// The edge radius given to a punching is a shape, not a tuning knob.
    ///
    /// Doubling it must not move the dip by anything the trade can measure. If
    /// it did, the number would depend on a modelling choice nobody made for a
    /// physical reason — which is exactly how a plausible wrong answer gets in.
    #[test]
    fn the_punching_edge_radius_does_not_set_the_dip() {
        let a = regulated();
        let dip_with = |r: f64| {
            let front = Stop {
                edge_radius: r,
                ..a.front_stop()
            };
            let back = Stop {
                edge_radius: r,
                ..a.back_stop()
            };
            let lever = a.key.front_edge - (-a.bearing.offset_from_pin);
            lever * (a.sin_angle_on_stop(front).unwrap() - a.sin_angle_on_stop(back).unwrap())
        };
        let drift = (dip_with(2.0 * PUNCHING_EDGE_RADIUS) - dip_with(PUNCHING_EDGE_RADIUS)).abs();
        assert!(
            drift < 5e-6,
            "doubling the edge radius moved the dip by {drift} m"
        );
    }

    /// Level 0: the contact's own lever arm matches a finite difference of its
    /// gap.
    ///
    /// The stops go through the same circle-against-line primitive as every
    /// other contact in the action, so they inherit its invariant: the analytic
    /// `d_gap_d_theta` is the derivative of the gap. Checking it here, on the
    /// first contact ever wired up, is cheaper than discovering a sign error
    /// once six of them are interacting.
    #[test]
    fn the_stop_contact_lever_arm_matches_a_finite_difference() {
        let a = regulated();
        let stop = a.front_stop();
        let h = 1e-7;

        for i in -3i32..=3 {
            let s = f64::from(i) * 0.008;
            let q = a.stop_contact(stop, s);
            // The key is body B of the pair, and its angle enters through
            // `sin θ`; differentiate the gap with respect to the same variable.
            let fd =
                (a.stop_contact(stop, s + h).gap - a.stop_contact(stop, s - h).gap) / (2.0 * h);
            // d(gap)/d(sin θ) = d(gap)/dθ · dθ/d(sin θ) = arm / cos θ, and the
            // primitive's sign convention for body B carries a minus.
            let c = (1.0 - s * s).sqrt();
            let analytic = -q.d_gap_d_theta_b / c;
            assert!(
                (analytic - fd).abs() < 1e-6,
                "at sin θ = {s}: analytic {analytic} vs finite difference {fd}"
            );
        }
    }

    /// The gap really does close at the angle the solver reports.
    #[test]
    fn the_stop_closes_at_the_angle_the_solver_reports() {
        let a = regulated();
        for stop in [a.front_stop(), a.back_stop()] {
            let s = a.sin_angle_on_stop(stop).expect("the stop is reachable");
            let gap = a.stop_contact(stop, s).gap;
            assert!(
                gap.abs() < 1e-12,
                "the solver says the key touches at sin θ = {s}, but the gap there is {gap}"
            );
        }
    }

    /// The dip lands inside the band the manufacturers publish.
    #[test]
    fn the_regulated_dip_is_inside_the_published_band() {
        let t = regulated().travel().unwrap();
        // Steinway NY 0.390–0.420 in = 9.91–10.67 mm; Hamburg 9.9–10.6;
        // Kawai 10.1–10.3. [V]
        assert!(
            (0.0099..=0.0107).contains(&t.key_dip),
            "dip {} m is outside every published specification",
            t.key_dip
        );
    }

    /// Adding paper under the balance punching raises the key and deepens the
    /// dip — and the second half is the part a settings panel would get wrong.
    ///
    /// Raising the pivot lifts the front at rest but leaves the front rail
    /// punching where it was, so the key now has further to fall. That coupling
    /// is real, it is why the trade sets height before dip, and here it falls
    /// out of the geometry rather than being asserted.
    #[test]
    fn levelling_the_key_also_deepens_the_dip() {
        let base = regulated();
        let mut raised = base.clone();
        raised.frame.balance_punching.paper += 0.0005;

        let (t0, t1) = (base.travel().unwrap(), raised.travel().unwrap());
        assert!(
            t1.key_height > t0.key_height,
            "0.5 mm of paper must raise the key: {} -> {}",
            t0.key_height,
            t1.key_height
        );
        assert!(
            t1.key_dip > t0.key_dip,
            "raising the pivot without moving the front punching must deepen the dip: \
             {} -> {}",
            t0.key_dip,
            t1.key_dip
        );
    }

    /// Adding paper on the front rail makes the dip shallower, and does not
    /// touch the key height.
    #[test]
    fn the_front_punching_sets_the_dip_and_only_the_dip() {
        let base = regulated();
        let mut shallower = base.clone();
        shallower.frame.front_punching.paper += 0.0005;

        let (t0, t1) = (base.travel().unwrap(), shallower.travel().unwrap());
        assert!(
            t1.key_dip < t0.key_dip,
            "paper on the front rail must reduce the dip: {} -> {}",
            t0.key_dip,
            t1.key_dip
        );
        assert!(
            (t1.key_height - t0.key_height).abs() < 1e-15,
            "the front punching must not move the key height"
        );
    }

    /// A bare key finds the stop its own balance sends it to.
    ///
    /// With no action on the capstan, a leaded key sits at the bottom of its
    /// dip. A technician knows this; a model that always rested the key on the
    /// back rail would be quietly wrong, and would hide it behind a plausible
    /// picture.
    #[test]
    fn a_bare_key_finds_the_stop_its_own_front_weight_sends_it_to() {
        let mut a = regulated();

        // The default keystick is longer in front of the balance pin than
        // behind it and carries a covering on the front half, so it is
        // front-heavy: a positive front weight. It therefore sits down.
        let moment = a.static_moment_about_bearing().unwrap();
        assert!(
            moment > 0.0,
            "the default key should be front-heavy, moment {moment}"
        );
        assert_eq!(
            a.free_rest(),
            Some(RestingOn::FrontRailPunching),
            "a bare front-heavy key sits at the bottom of its own dip"
        );

        // Move the mass behind the bearing and it flips, which is the treble
        // case on a real unleaded keyboard: measured front weights there run as
        // low as −9 g.
        a.key.leads = vec![super::super::Lead::at(-0.150, 0.060)];
        assert!(
            a.static_moment_about_bearing().unwrap() < 0.0,
            "60 g behind the pin must make the key back-heavy"
        );
        assert_eq!(
            a.free_rest(),
            Some(RestingOn::BackRailCloth),
            "a back-heavy key rests where the action would hold it anyway"
        );
    }

    /// Moving the effective bearing behind the pin changes the in-action lever
    /// without touching the weighing jig's.
    ///
    /// This is Stanwood's punching-cutting technique, and the point of the test
    /// is that the two arms are genuinely different numbers. A model that used
    /// one arm for both would make the technique a no-op.
    #[test]
    fn cutting_the_punching_moves_the_in_action_arm_only() {
        let base = regulated();
        let mut cut = base.clone();
        cut.bearing.offset_from_pin = 0.0015;

        assert!(
            cut.measuring_arm() > base.measuring_arm(),
            "moving the bearing back lengthens the arm to the measuring point"
        );
        // The jig still pivots on the pin, so its arm is the key's geometry
        // and the protocol's, and has not moved.
        assert!(
            (cut.measuring_point() - base.measuring_point()).abs() < 1e-15,
            "the weighing jig's arm belongs to the key and the protocol, not to the frame"
        );
    }

    /// Level 0: **virtual work.** The generalized force is the gradient of the
    /// potential, and nothing else.
    ///
    /// Gravity's torque on the key must equal minus the derivative of its
    /// potential energy with respect to the key's angle, at every angle the key
    /// reaches. This is the test that catches a lever arm taken about the wrong
    /// point, a centre of gravity in the wrong frame, or a sign — and it needs
    /// no measured parameter and no solver, only the claim that energy and
    /// force are two views of the same thing.
    #[test]
    // A finite difference in the angle itself: the reference angles come from
    // the platform's `asin` and `sin`, which the lint refuses outside tests.
    #[allow(clippy::disallowed_methods)]
    fn gravity_torque_is_minus_the_gradient_of_the_potential() {
        let a = regulated();
        let h = 1e-7;

        for i in -4i32..=4 {
            let s = f64::from(i) * 0.01;
            let theta = s.asin();
            // Differentiate with respect to θ itself, which is the coordinate
            // the generalized force is conjugate to.
            let v = |th: f64| a.centre_of_mass_height(th.sin()).unwrap();
            let dv_dtheta = (v(theta + h) - v(theta - h)) / (2.0 * h);

            let analytic = a.gravity_torque(s).unwrap();
            let mass = a.key.mass_properties().unwrap().mass;
            let from_energy = -mass * dv_dtheta;

            assert!(
                (analytic - from_energy).abs() < 1e-9,
                "at sin θ = {s}: torque {analytic} vs -dV/dθ {from_energy}"
            );
        }
    }

    /// And the same identity with leads fitted, since a point mass is where the
    /// two views are easiest to let drift apart.
    #[test]
    // A finite difference in the angle itself: the reference angles come from
    // the platform's `asin` and `sin`, which the lint refuses outside tests.
    #[allow(clippy::disallowed_methods)]
    fn virtual_work_still_holds_once_the_key_is_leaded() {
        let mut a = regulated();
        a.key.leads = vec![
            super::super::Lead::at(0.150, 0.014),
            super::super::Lead::at(-0.060, 0.008),
        ];
        let h = 1e-7;
        let mass = a.key.mass_properties().unwrap().mass;

        for i in -3i32..=3 {
            let s = f64::from(i) * 0.012;
            let theta = s.asin();
            let v = |th: f64| a.centre_of_mass_height(th.sin()).unwrap();
            let from_energy = -mass * (v(theta + h) - v(theta - h)) / (2.0 * h);
            assert!(
                (a.gravity_torque(s).unwrap() - from_energy).abs() < 1e-9,
                "leaded, at sin θ = {s}"
            );
        }
    }

    /// Unreachable targets are reported, not clamped.
    #[test]
    fn an_impossible_dip_is_refused() {
        let key = KeyDesign::default();
        let frame = FrameDesign::default();
        let absurd = Regulation {
            key_height: 0.0650,
            key_dip: 0.080,
        };
        assert!(
            absurd.solve(&key, &frame, 0.0).is_none(),
            "an 80 mm dip would need the front punching below the rail"
        );
    }
}

/// Where the key has settled, and what is holding it there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settled {
    /// Sine of the key's angle.
    pub sin_theta: f64,
    /// What holds it.
    pub held_by: HeldBy,
    /// Normal reaction at the stop, as a mass-moment about the bearing, kg·m.
    /// Zero when the key is floating.
    pub stop_reaction: f64,
}

/// What is carrying the key, which is three things and not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldBy {
    /// Resting on the front rail punching: the bottom of the dip.
    FrontPunching,
    /// Resting on the back rail cloth: where the action would hold it.
    BackRailCloth,
    /// Touching neither stop, held only by the friction of its own centres.
    /// Between the two branches the key stays wherever it was left — that band
    /// *is* the friction, and it is why a key has a down weight and an up
    /// weight rather than one threshold.
    FrictionAlone,
}

/// Fixed number of bisection steps used to locate a floating equilibrium.
///
/// Twenty-four halvings of a 10 mm dip resolve it to under a nanometre, about
/// four orders below the hundredth of a millimetre the trade regulates to.
pub const SETTLE_BISECTIONS: u32 = 24;

impl KeyAssembly {
    /// Total potential of the key and the weight on it, as a **height-weighted
    /// mass** in kg·m — that is, the potential energy divided by `g`.
    ///
    /// Everything that can store energy in this system is in here: the key with
    /// its leads and its capstan, and the weight sitting on the measuring
    /// point. The stops store none, being rigid; friction stores none, being
    /// dissipative. So this is the whole conservative part of the chain, and
    /// [`Self::net_torque`] must be exactly its negative gradient.
    pub fn total_potential(&self, sin_theta: f64, applied_g: f64) -> Option<f64> {
        let p = self.key.mass_properties()?;
        let c = (1.0 - sin_theta * sin_theta).max(0.0).sqrt();
        let dxm = self.measuring_point() - self.bearing_x();
        let dym = self.key.stick_height + self.key.keytop_thickness;
        let y_measuring = self.bearing_y() - sin_theta * dxm + c * dym;
        Some(p.mass * self.centre_of_mass_height(sin_theta)? + applied_g * 1e-3 * y_measuring)
    }

    /// The key's mass plus `applied_g` grams on the measuring point, kg: the
    /// vertical load the balance punching carries while the key touches
    /// neither stop.
    ///
    /// # Why this exists, and what it does not guard
    ///
    /// The balance punching is modelled as a **bilateral pivot**, not as a
    /// unilateral contact like the two stops. That is a deliberate choice and
    /// it needs a guard rather than a shrug.
    ///
    /// The reason is structural: a key resting *on* a punching it could also
    /// lift off has a vertical freedom as well as a rotational one, and that
    /// second degree of freedom — together with the mortise bushings that
    /// constrain it — is not modelled.
    /// Modelling the bearing as a contact without it would over-constrain one
    /// coordinate with three contacts and the solve would be ill-posed.
    ///
    /// So the assumption is that the key never lifts off its balance punching.
    /// While the punching's load is positive it is in compression, and a
    /// bilateral pivot is indistinguishable from a unilateral one; if the load
    /// goes negative the model is holding the key down with a washer, and the
    /// answer stops meaning anything. This sum is not that load once the key
    /// rests on a stop — [`Self::bearing_load_with_stops`] takes the stop's
    /// share out — so a guard built on it alone does not check the assumption.
    pub fn bearing_load(&self, applied_g: f64) -> Option<f64> {
        let p = self.key.mass_properties()?;
        Some(p.mass + applied_g * 1e-3)
    }

    /// The same load, **with the stops taking their share**.
    ///
    /// When the key is down on its front punching, that punching is carrying
    /// part of the weight and the balance bearing is carrying the rest. The
    /// simple sum of key and applied weight is therefore an over-estimate
    /// wherever the key is touching anything — which, for a bare key, is
    /// always.
    ///
    /// Closing the vertical equilibrium matters beyond tidiness: with a
    /// load-proportional friction model the bound is set by the load the
    /// *bearing* actually carries, so a key resting on its punching has less
    /// friction at its centre than a floating one. That is real, and a model
    /// that charged the bearing with the whole weight would overstate it.
    ///
    /// Returns the load at the bearing and the reaction at whichever stop is
    /// carrying, both in kg.
    pub fn bearing_load_with_stops(&self, settled: Settled, applied_g: f64) -> Option<(f64, f64)> {
        let total = self.bearing_load(applied_g)?;
        let stop = match settled.held_by {
            HeldBy::FrictionAlone => return Some((total, 0.0)),
            HeldBy::FrontPunching => self.front_stop(),
            HeldBy::BackRailCloth => self.back_stop(),
        };
        // The contact normal points from the key towards the stop, so its
        // vertical component is negative and the force on the key is upward by
        // that much.
        let n_y = self.stop_contact(stop, settled.sin_theta).normal.y.abs();
        let carried = settled.stop_reaction * n_y;
        Some((total - carried, carried))
    }

    /// Whether the key and the applied weight together still press down. It
    /// leaves out the stop reactions, so it does not settle whether the
    /// balance punching is in compression; see [`Self::bearing_load`].
    pub fn bearing_stays_in_compression(&self, applied_g: f64) -> Option<bool> {
        Some(self.bearing_load(applied_g)? > 0.0)
    }

    /// The net conservative torque on the key, as a mass-moment about the
    /// bearing in kg·m, with a weight of `applied_g` grams on the measuring
    /// point.
    ///
    /// Front-down positive. Both terms are exact at any angle: each is
    /// `-d(height)/dθ` for the mass concerned, which is the same identity the
    /// virtual-work test checks.
    pub fn net_torque(&self, sin_theta: f64, applied_g: f64) -> Option<f64> {
        let applied = applied_g * 1e-3 * self.measuring_arm_at(sin_theta);
        Some(self.gravity_torque(sin_theta)? + applied)
    }

    /// Solves where the key comes to rest under a weight at the measuring
    /// point, with both stops unilateral and the centre friction set-valued.
    ///
    /// # The complementarity conditions, which come out in closed form
    ///
    /// A stop can only push, so its reaction `λ ≥ 0` and it acts only where the
    /// gap is closed. Its generalized force is `λ · ∂g/∂θ`, and the signs do
    /// the rest: turning the key front-down **closes** the front gap, so
    /// `∂g_front/∂θ < 0` and the punching pushes the key back up, as it must.
    /// Writing the balance `T + τ_f + λ ∂g/∂θ = 0` with `τ_f ∈ [−b, b]` and
    /// `λ ≥ 0` then gives three conditions and no search:
    ///
    /// | State | Stays iff |
    /// |---|---|
    /// | on the front punching | `T ≥ −b` |
    /// | on the back rail cloth | `T ≤ b` |
    /// | touching neither | `|T| ≤ b` |
    ///
    /// This is the general quasi-static problem — a pair of linear programs —
    /// at one degree of freedom, where it degenerates to inequalities. Solving
    /// it rather than comparing a weight against two precomputed branches is
    /// what makes the bench answer the question a technician actually asks, and
    /// it fixes a wrong answer: a bare key already sitting on its front
    /// punching is **held there**, not perpetually descending.
    pub fn settle(
        &self,
        from_sin_theta: f64,
        applied_g: f64,
        friction_bound: f64,
    ) -> Option<Settled> {
        let s_front = self.sin_angle_on_front_rail()?;
        let s_back = self.sin_angle_on_back_rail()?;
        let b = friction_bound.max(0.0);

        let state_at = |s: f64| -> Option<Settled> {
            let t = self.net_torque(s, applied_g)?;
            // Within 1e-12 of a stop's sine, a quarter of a picometre at the
            // front of the key, treat the key as on it.
            let on_front = s >= s_front - 1e-12;
            let on_back = s <= s_back + 1e-12;

            if on_front && t >= -b {
                let j = -self.stop_contact(self.front_stop(), s).d_gap_d_theta_b;
                return Some(Settled {
                    sin_theta: s_front,
                    held_by: HeldBy::FrontPunching,
                    stop_reaction: if j.abs() > 1e-12 {
                        (t + b) / j.abs()
                    } else {
                        0.0
                    },
                });
            }
            if on_back && t <= b {
                let j = -self.stop_contact(self.back_stop(), s).d_gap_d_theta_b;
                return Some(Settled {
                    sin_theta: s_back,
                    held_by: HeldBy::BackRailCloth,
                    stop_reaction: if j.abs() > 1e-12 {
                        (b - t) / j.abs()
                    } else {
                        0.0
                    },
                });
            }
            if !on_front && !on_back && t.abs() <= b {
                return Some(Settled {
                    sin_theta: s,
                    held_by: HeldBy::FrictionAlone,
                    stop_reaction: 0.0,
                });
            }
            None
        };

        // A frame can be edited into one no key can sit in: a back rail cloth
        // that holds the tail higher than the front punching lets the front
        // come down, or a balance rail dropped below them both. The key would
        // then ride on its two outer stops and **lift off its balance
        // punching** — a degree of freedom this model does not have — so at one
        // coordinate there is no answer, and saying so beats returning a
        // position that is through the wood.
        if s_back > s_front {
            return None;
        }
        let start = from_sin_theta.clamp(s_back, s_front);
        if let Some(settled) = state_at(start) {
            return Some(settled);
        }

        // Not in equilibrium: the key moves, in the direction the net torque
        // pushes. It travels until it either re-enters the friction band or
        // reaches the stop ahead of it.
        let t0 = self.net_torque(start, applied_g)?;
        let target = if t0 > b { s_front } else { s_back };
        if let Some(settled) = state_at(target) {
            // Did it pass through an equilibrium on the way? Only if the torque
            // re-entered the band, which a sign check on the band edge detects.
            let edge = if t0 > b { b } else { -b };
            let crossed = (self.net_torque(target, applied_g)? - edge) * (t0 - edge) < 0.0;
            if !crossed {
                return Some(settled);
            }
            let (mut lo, mut hi) = (start, target);
            for _ in 0..SETTLE_BISECTIONS {
                let mid = 0.5 * (lo + hi);
                if (self.net_torque(mid, applied_g)? - edge) * (t0 - edge) > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            return Some(Settled {
                sin_theta: 0.5 * (lo + hi),
                held_by: HeldBy::FrictionAlone,
                stop_reaction: 0.0,
            });
        }
        state_at(target)
    }
}

#[cfg(test)]
mod virtual_work_tests {
    use super::super::frame::PunchingStack;
    use super::super::{FrameDesign, KeyDesign, Lead};
    use super::*;

    fn bench() -> KeyAssembly {
        let key = KeyDesign {
            leads: vec![Lead::at(0.140, 0.012), Lead::at(-0.080, 0.009)],
            ..KeyDesign::default()
        };
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target.solve(&key, &frame, 0.0).unwrap();
        frame.balance_punching = PunchingStack::new(balance - 0.0015, 0.0015);
        frame.front_punching = PunchingStack::new(front - 0.0020, 0.0020);
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: balance,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// Level 0: **virtual work over the whole chain**, not only gravity.
    ///
    /// The net generalized force the solver uses must be exactly minus the
    /// gradient of the total potential — key, leads, capstan *and* the weight
    /// on the measuring point. A separate test already checks the gravity half;
    /// this one checks that the applied weight enters by the same route rather
    /// than through an arm written out by hand, which is where a cosine or a
    /// lever gets lost.
    #[test]
    // A finite difference in the angle itself: the reference angles come from
    // the platform's `asin` and `sin`, which the lint refuses outside tests.
    #[allow(clippy::disallowed_methods)]
    fn net_torque_is_minus_the_gradient_of_the_whole_potential() {
        let a = bench();
        let h = 1e-7;

        for &w in &[-40.0_f64, -5.0, 0.0, 12.0, 55.0] {
            for i in -4i32..=4 {
                let s = f64::from(i) * 0.01;
                let theta = s.asin();
                let v = |th: f64| a.total_potential(th.sin(), w).unwrap();
                let from_energy = -(v(theta + h) - v(theta - h)) / (2.0 * h);
                let analytic = a.net_torque(s, w).unwrap();
                assert!(
                    (analytic - from_energy).abs() < 1e-9,
                    "at {w} g and sin θ = {s}: torque {analytic} vs -dV/dθ {from_energy}"
                );
            }
        }
    }

    /// Level 0: every state the solver settles on satisfies the **principle of
    /// virtual work for unilateral constraints**.
    ///
    /// At equilibrium, no admissible virtual displacement may release energy:
    /// `(T + τ_f) δθ ≤ 0` for every `δθ` the stops allow and some friction
    /// torque inside its bound. On a stop only the direction that opens the gap
    /// is admissible, which is what makes the condition an inequality rather
    /// than an equation — and that inequality is the whole reason a key can sit
    /// still on a punching with a net torque pressing it there.
    ///
    /// Checked against the energy by finite difference, so it tests the
    /// solver's three inequalities against the principle they are supposed to
    /// implement, rather than against themselves.
    #[test]
    fn every_settled_state_satisfies_the_virtual_work_inequality() {
        let a = bench();
        let b = 6.0e-3 * a.measuring_arm();
        let delta = 1e-6;
        let (s_back, s_front) = (
            a.sin_angle_on_back_rail().unwrap(),
            a.sin_angle_on_front_rail().unwrap(),
        );
        let mut seen_stop = false;
        let mut seen_floating = false;

        for i in -12i32..=12 {
            let w = f64::from(i) * 6.0;
            let settled = a.settle(0.5 * (s_back + s_front), w, b).unwrap();
            let s = settled.sin_theta;

            // The virtual displacements the stops leave admissible.
            let admissible: &[f64] = match settled.held_by {
                HeldBy::FrontPunching => &[-delta],
                HeldBy::BackRailCloth => &[delta],
                HeldBy::FrictionAlone => &[-delta, delta],
            };
            if settled.held_by == HeldBy::FrictionAlone {
                seen_floating = true;
            } else {
                seen_stop = true;
            }

            for &d in admissible {
                // Energy released by the conservative forces over this virtual
                // displacement, from the potential and not from the torque.
                let released = a.total_potential(s, w).unwrap()
                    - a.total_potential((s + d).clamp(-1.0, 1.0), w).unwrap();
                // Friction can absorb at most `b * |δθ|`.
                let absorbed = b * d.abs();
                assert!(
                    released <= absorbed + 1e-12,
                    "at {w} g, held by {:?}: a virtual move of {d} releases {released}, \
                     more than the {absorbed} friction can absorb — so this is not an \
                     equilibrium and the solver should not have stopped here",
                    settled.held_by
                );
            }
        }

        assert!(seen_stop, "the sweep must visit at least one stop");
        assert!(seen_floating, "and at least one floating equilibrium");
    }
}

#[cfg(test)]
mod bearing_tests {
    use super::super::frame::PunchingStack;
    use super::super::{FrameDesign, KeyDesign};
    use super::*;

    fn bench() -> KeyAssembly {
        let key = KeyDesign::default();
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target.solve(&key, &frame, 0.0).unwrap();
        frame.balance_punching = PunchingStack::new(balance - 0.0015, 0.0015);
        frame.front_punching = PunchingStack::new(front - 0.0020, 0.0020);
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: balance,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// A rail hung off the end of the key is carried by the key's own corner.
    ///
    /// The third case, and the one a designer reaches by accident: put the
    /// cloth so far back that the end of the key stands over the middle of it,
    /// and what bears is not a face at all but the corner where the key stops.
    /// A model that only knew faces would place the key by a contact off the
    /// end of the wood and draw the tail sunk into the felt.
    #[test]
    fn a_cloth_behind_the_end_of_the_key_is_carried_by_the_key_corner() {
        let mut a = bench();
        // The strip reaches 12 mm either side, so this hangs 4 mm of it out
        // behind the end of the key.
        a.frame.back_rail = -0.232;
        let touch = a
            .first_touch(a.back_stop())
            .expect("the cloth is reachable");
        assert_eq!(
            touch.how,
            Touching::KeyCorner,
            "the end of the key is what bears, not a face: {:?}",
            touch.how
        );
        // On the key, the load is exactly at the end of the wood: the corner
        // where the chamfer stops, at the top of its rise. Asserted there
        // rather than in the world, where the same corner has swung 0.09 mm
        // forward with the key's own attitude.
        assert!(
            (touch.on_key.x - a.key.tail_end).abs() < 1e-9
                && (touch.on_key.y - a.key.tail_rise).abs() < 1e-9,
            "it bears at ({}, {}) on the key, not at its corner ({}, {})",
            touch.on_key.x,
            touch.on_key.y,
            a.key.tail_end,
            a.key.tail_rise
        );
    }

    /// The chamfer under the tail is there so the key beds flat on its cloth.
    ///
    /// The owner's account of why it exists, and it ties three quantities
    /// together: a key that beds flat stands at the chamfer's own angle, and a
    /// key level at the bottom of its travel stands at the dip divided by the
    /// arm to the front of the key. Both at once means the chamfer's slope
    /// *is* that ratio. The test asserts the relation rather than the value,
    /// because the relation is the physics and the value is three measurements
    /// that have to agree.
    ///
    /// It also walks the rail through the three cases, because a model that
    /// only did the flat one would be no use for designing a keyboard: a
    /// fraction of a millimetre either side of the height that beds the key,
    /// the load moves to one edge of the cloth or the other, and that is a
    /// badly regulated keyboard rather than an impossible one.
    #[test]
    fn bedding_flat_at_the_back_is_the_chamfer_matching_the_rest_attitude() {
        let mut a = bench();
        // The cloth goes under the chamfer, which is where a cloth the key can
        // bed on has to be: the chamfer is the face cut to lie on it.
        a.frame.back_rail = -0.200;
        let chamfer = a.key.tail_rise / a.key.tail_rise_run;
        let flat_attitude = -chamfer / (1.0 + chamfer * chamfer).sqrt();

        // The one rail height that stands the key at the chamfer's own angle.
        let (flat_from, _) = a.key.underside_flat();
        let on_the_chamfer = Point::new(
            a.frame.back_rail,
            a.key.tail_rise * (flat_from - a.frame.back_rail) / a.key.tail_rise_run,
        );
        let bedded_height = a.key_frame(flat_attitude).world(on_the_chamfer).y;
        a.frame.back_rail_height = bedded_height - a.frame.back_rail_cloth;

        let touch = a
            .first_touch(a.back_stop())
            .expect("the cloth is reachable");
        assert_eq!(
            touch.how,
            Touching::Flat,
            "at the chamfer's own angle the tail lies flat on the cloth, \
             {:?} at a slope of {}",
            touch.how,
            touch.slope
        );
        assert!(
            touch.slope.abs() < 1e-9,
            "and flat means flat, not {}",
            touch.slope
        );
        assert!(
            (touch.sin_theta - flat_attitude).abs() < 1e-9,
            "the key rests at the chamfer's angle, not at {}",
            touch.sin_theta
        );

        // Half a millimetre of cloth too little, and the key stands more
        // nose-up than its chamfer allows: the tail digs in at the back edge.
        let mut low = a.clone();
        low.frame.back_rail_height -= 0.0005;
        let digging = low.first_touch(low.back_stop()).expect("still reachable");
        assert_ne!(digging.how, Touching::Flat);
        assert!(
            digging.slope > 0.0,
            "a key more nose-up than its chamfer digs in: {}",
            digging.slope
        );
        assert!(
            digging.at < low.back_stop().at,
            "and it digs in at the back of the cloth, not at {}",
            digging.at
        );
        // The miss is the rest attitude less the chamfer, which is the relation
        // the three measurements have to satisfy.
        let expected = -digging.sin_theta - chamfer;
        assert!(
            (digging.slope - expected).abs() < 1e-4,
            "{} against {expected}",
            digging.slope
        );

        // Half a millimetre too much, and it rides on the front edge instead.
        let mut high = a.clone();
        high.frame.back_rail_height += 0.0005;
        let riding = high.first_touch(high.back_stop()).expect("still reachable");
        assert_ne!(riding.how, Touching::Flat);
        assert!(riding.slope < 0.0, "{}", riding.slope);
        assert!(
            riding.at > high.back_stop().at,
            "the load is on the front edge of the cloth, not at {}",
            riding.at
        );
    }

    /// The angle at which the key lands on a stop really closes that stop.
    ///
    /// `sin_angle_on_stop` solves a **squared** condition, which also admits
    /// the branch with the stop's edge above the key's underside rather than
    /// below it — the key through the wood instead of resting on it. Nothing
    /// downstream would notice, because the angle and the gap would then answer
    /// different questions: the bench would place the key at an angle it calls
    /// *on the back rail cloth* and report, truthfully for that angle, a gap of
    /// 20 mm.
    #[test]
    fn the_angle_on_a_stop_really_closes_that_stop() {
        let a = bench();
        for (name, stop) in [("front", a.front_stop()), ("back", a.back_stop())] {
            let s = a
                .sin_angle_on_stop(stop)
                .unwrap_or_else(|| panic!("the {name} stop must be reachable here"));
            let gap = a.stop_contact(stop, s).gap;
            assert!(
                gap.abs() < 1e-9,
                "at the angle said to land on the {name} stop the gap reads {gap} m"
            );
        }
    }

    /// A stop above the bearing is reached by tilting, not by a phantom root.
    ///
    /// Raising the back rail cloth above the height the key turns at does not
    /// put it out of reach: turning the key nose-down lifts its tail onto it.
    /// So there *is* an angle, and it is positive. Its negative mirror is the
    /// squared equation's other branch, with the cloth above the key's
    /// underside rather than under it, and it must be rejected.
    #[test]
    fn a_stop_above_the_bearing_is_reached_by_tilting_not_by_a_phantom_root() {
        let mut a = bench();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;

        let s = a
            .sin_angle_on_back_rail()
            .expect("turning the key nose-down reaches it");
        assert!(
            s > 0.0,
            "landing on a stop above the bearing means turning the key \
             nose-down; this angle turns it the other way: {s}"
        );
        assert!(
            a.stop_contact(a.back_stop(), s).gap.abs() < 1e-9,
            "and at that angle the tail must really be on the cloth"
        );
    }

    /// Stops that fight each other leave the key nowhere to be.
    ///
    /// The cloth now holds the tail higher than the punching lets the front
    /// come down, so no single angle satisfies both. A real key answers by
    /// riding on the two of them and lifting off its balance punching — a
    /// degree of freedom this model does not have — so at one coordinate the
    /// answer is that there is none, and that beats returning a position inside
    /// the wood.
    #[test]
    fn a_frame_whose_stops_fight_is_refused_rather_than_solved() {
        let mut a = bench();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        assert!(
            a.sin_angle_on_back_rail().unwrap() > a.sin_angle_on_front_rail().unwrap(),
            "this frame is meant to be over-constrained"
        );
        assert!(a.settle(0.0, 0.0, 0.0).is_none());
    }

    /// The key stops when the pin bottoms in its mortise, not only on felt.
    #[test]
    fn a_pin_too_proud_stops_the_key_before_the_punching_does() {
        let mut a = bench();
        assert!(
            !a.stopped_by_the_mortise(),
            "on a sane frame it is the felt the key lands on"
        );
        let on_felt = a.sin_angle_on_front_rail().expect("a reachable punching");

        a.frame.sections.front.pin_height += 0.010;
        assert!(
            a.stopped_by_the_mortise(),
            "ten more millimetres of pin must bottom before the felt does"
        );
        let on_pin = a.sin_angle_on_front_rail().expect("a reachable mortise");
        assert!(
            on_pin < on_felt,
            "the key must stop higher, not carry on: {on_pin} against {on_felt}"
        );
        assert!(
            a.stop_contact(a.front_stop(), on_pin).gap.abs() < 1e-9,
            "and it must really be standing on the pin"
        );
    }

    /// The key and the applied weight together still press down over the
    /// whole range of the bench's slider.
    ///
    /// The slider runs to −60 g and the bare key weighs about 121 g, so the
    /// sum stays positive. That sum is all `bearing_stays_in_compression`
    /// looks at: it leaves out the reaction at whichever stop the key rests
    /// on, so this does not show that the balance punching stays in
    /// compression.
    #[test]
    fn key_and_applied_weight_still_press_down_over_the_slider_range() {
        let a = bench();
        for w in [-60.0, -30.0, 0.0, 30.0, 60.0] {
            assert_eq!(
                a.bearing_stays_in_compression(w),
                Some(true),
                "at {w} g the balance punching would be in tension, and a \
                 bilateral pivot would be hiding it"
            );
        }
    }

    /// And it fails loudly when the assumption really is broken.
    ///
    /// Lift harder than the key weighs and the punching would have to pull.
    /// The guard must say so rather than return a confident wrong number — the
    /// whole value of naming an assumption is that it can be violated.
    #[test]
    fn lifting_harder_than_the_key_weighs_breaks_the_assumption() {
        let a = bench();
        let mass_g = a.key.mass_properties().unwrap().mass * 1000.0;
        assert_eq!(
            a.bearing_stays_in_compression(-mass_g - 1.0),
            Some(false),
            "pulling up harder than the key weighs must be reported, not absorbed"
        );
    }
}

#[cfg(test)]
mod settle_tests {
    use super::super::frame::PunchingStack;
    use super::super::{FrameDesign, KeyDesign, Lead};
    use super::*;

    fn bench() -> KeyAssembly {
        let key = KeyDesign::default();
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target.solve(&key, &frame, 0.0).unwrap();
        frame.balance_punching = PunchingStack::new(balance - 0.0015, 0.0015);
        frame.front_punching = PunchingStack::new(front - 0.0020, 0.0020);
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: balance,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// Six grams of key friction, as a torque about the bearing.
    fn bound(a: &KeyAssembly) -> f64 {
        6.0e-3 * a.measuring_arm()
    }

    /// A bare front-heavy key is **held by its front punching**, not endlessly
    /// descending.
    ///
    /// This is the wrong answer the contact solve was built to fix. Comparing a
    /// weight against two precomputed branches says "the key goes down" for
    /// ever, because the branches describe a *floating* key and nothing in them
    /// knows the punching is already carrying it. The stop's reaction has to be
    /// in the force balance for the question to have the right answer.
    #[test]
    fn a_bare_key_resting_on_its_punching_is_held_by_it() {
        let a = bench();
        let s = a
            .settle(a.sin_angle_on_front_rail().unwrap(), 0.0, bound(&a))
            .unwrap();
        assert_eq!(s.held_by, HeldBy::FrontPunching);
        assert!(
            s.stop_reaction > 0.0,
            "the punching must be pushing back: {}",
            s.stop_reaction
        );
    }

    /// Lift it off with enough negative weight, and it rises to the cloth.
    #[test]
    fn enough_lift_takes_the_key_off_the_punching_and_onto_the_cloth() {
        let a = bench();
        let from = a.sin_angle_on_front_rail().unwrap();
        let b = bound(&a);

        // A little lift is absorbed by the stop and the friction.
        let barely = a.settle(from, -5.0, b).unwrap();
        assert_eq!(barely.held_by, HeldBy::FrontPunching);

        // Enough, and the key goes all the way to the back rail cloth.
        let lifted = a.settle(from, -60.0, b).unwrap();
        assert_eq!(lifted.held_by, HeldBy::BackRailCloth);
        assert!(
            lifted.sin_theta < barely.sin_theta,
            "the key must have risen"
        );
    }

    /// Between the branches the key stays exactly where it was left.
    ///
    /// Hysteresis, and it is the point of set-valued friction: the same weight
    /// gives two different positions depending on where the key came from. A
    /// regularized friction law has one equilibrium and would creep to it.
    #[test]
    fn the_key_keeps_the_position_it_was_left_in() {
        let mut a = bench();
        // A key balanced about its bearing, so the band straddles zero weight.
        a.key.leads = vec![Lead::at(-0.120, 0.030)];
        let b = bound(&a);
        let (lo, hi) = (
            a.sin_angle_on_back_rail().unwrap(),
            a.sin_angle_on_front_rail().unwrap(),
        );
        let middle = 0.5 * (lo + hi);

        let w = 1000.0 * a.net_torque(middle, 0.0).unwrap() / a.measuring_arm();
        // Just inside the band on either side: the key must not move at all.
        for nudge in [-1.0, 0.0, 1.0] {
            let s = a.settle(middle, -w + nudge, b).unwrap();
            assert_eq!(
                s.held_by,
                HeldBy::FrictionAlone,
                "at {} g the key should be held by friction alone",
                -w + nudge
            );
            assert!(
                (s.sin_theta - middle).abs() < 1e-9,
                "the key moved from {middle} to {} inside its own band",
                s.sin_theta
            );
        }
    }

    /// The two branches read off the solve are the two branches the protocol
    /// defines, and their gap is twice the friction.
    #[test]
    fn the_solve_reproduces_the_branches_the_protocol_reads() {
        let a = bench();
        let b = bound(&a);
        let arm = a.measuring_arm();
        // With the key floating, equilibrium holds while |T| <= bound, so the
        // two edges of the band in grams at the measuring point are these.
        let mid =
            0.5 * (a.sin_angle_on_back_rail().unwrap() + a.sin_angle_on_front_rail().unwrap());
        let g_for = |t: f64| 1000.0 * t / arm;
        let t_gravity = a.gravity_torque(mid).unwrap();

        let down = g_for(b - t_gravity);
        let up = g_for(-b - t_gravity);
        assert!(
            (0.5 * (down - up) - 6.0).abs() < 1e-6,
            "half the band must be the 6 g of friction, got {}",
            0.5 * (down - up)
        );

        // And the solve agrees: a hair past the down weight the key leaves the
        // band downwards, a hair under the up weight it leaves it upwards.
        assert_eq!(
            a.settle(mid, down + 0.5, b).unwrap().held_by,
            HeldBy::FrontPunching
        );
        assert_eq!(
            a.settle(mid, up - 0.5, b).unwrap().held_by,
            HeldBy::BackRailCloth
        );
    }

    /// A frictionless key has no band, so it is never held by friction alone.
    #[test]
    fn with_no_friction_there_is_no_band() {
        let a = bench();
        let mid =
            0.5 * (a.sin_angle_on_back_rail().unwrap() + a.sin_angle_on_front_rail().unwrap());
        let s = a.settle(mid, 0.0, 0.0).unwrap();
        assert_ne!(
            s.held_by,
            HeldBy::FrictionAlone,
            "without friction a key off its stops cannot rest anywhere"
        );
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::super::frame::PunchingStack;
    use super::super::{FrameDesign, KeyDesign};
    use super::*;

    fn bench() -> KeyAssembly {
        let key = KeyDesign::default();
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target.solve(&key, &frame, 0.0).unwrap();
        frame.balance_punching = PunchingStack::new(balance - 0.0015, 0.0015);
        frame.front_punching = PunchingStack::new(front - 0.0020, 0.0020);
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: balance,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// Level 4: **key height is read from the keybed**, and the keybed is the
    /// datum that makes it mean anything.
    ///
    /// Raise the whole frame and the key height must rise with it, one for
    /// one — that is what "above the keybed" means. A height measured from the
    /// balance rail, say, would not move at all, and would be a different and
    /// useless number.
    #[test]
    fn key_height_is_read_from_the_keybed() {
        let base = bench();
        let mut raised = base.clone();
        let lift = 0.0030;
        raised.frame.balance_rail_height += lift;
        raised.frame.front_rail_height += lift;
        raised.frame.back_rail_height += lift;

        let h0 = base.travel().unwrap().key_height;
        let h1 = raised.travel().unwrap().key_height;
        assert!(
            (h1 - h0 - lift).abs() < 1e-9,
            "lifting the whole frame by {lift} m raised the key by {}",
            h1 - h0
        );
        // And the dip, which is a travel and not a height, must not care.
        assert!(
            (raised.travel().unwrap().key_dip - base.travel().unwrap().key_dip).abs() < 1e-9,
            "a travel cannot depend on how high the frame stands"
        );
    }

    /// Level 4: **the dip's datum is load-bearing**, exactly as the front
    /// weight's is.
    ///
    /// Dip is read at the key front; touchweights at the measuring point, 13 mm
    /// back from the covering's lip, which stands 2 mm proud of the key front —
    /// so 11 mm behind it. Read the same travel at the other datum and it must
    /// come out smaller, in the ratio of the two arms. If the two agreed, one
    /// of them would be reading a stored number rather than measuring the key.
    #[test]
    fn reading_the_dip_at_the_other_datum_changes_it_by_the_arm_ratio() {
        let a = bench();
        let at_front = a.key_dip_at(ReadAt::FrontEdge).unwrap();
        let at_weighing = a.key_dip_at(ReadAt::MeasuringPoint).unwrap();

        assert!(at_weighing < at_front, "the nearer datum must travel less");

        // The ratio is *nearly* the ratio of the arms, and the residual is
        // physical rather than numerical: turning the key also drops the top
        // surface by `(cos θ_rest − cos θ_bottom) · h`, and that term is the
        // same at both datums, so it adds to each reading without scaling. A
        // test that demanded the bare arm ratio would be asserting the
        // small-angle approximation, not the geometry.
        let s_rest = a.sin_angle_on_back_rail().unwrap();
        let s_bottom = a.sin_angle_on_front_rail().unwrap();
        let common = ((1.0 - s_rest * s_rest).sqrt() - (1.0 - s_bottom * s_bottom).sqrt())
            * (a.key.stick_height + a.key.keytop_thickness);

        let arms = a.measuring_point() / a.key.front_edge;
        assert!(
            (at_weighing / at_front - arms).abs() < 1e-3,
            "the two readings are in the ratio {}, far from the arm ratio {arms}",
            at_weighing / at_front
        );
        // Take the common term out of both and the arm ratio comes back exactly.
        assert!(
            ((at_weighing - common) / (at_front - common) - arms).abs() < 1e-9,
            "with the vertical term removed the ratio must be exactly the arms: {}",
            (at_weighing - common) / (at_front - common)
        );
        // Its **sign** says which side of level the key swings on, and is not
        // a property to assert: a frame set so the key is level at the bottom
        // of its travel swings entirely on the nose-up side, and the term is a
        // small rise rather than a small drop. What matters is that it is tens
        // of micrometres either way — small, and not scaling with the arm.
        assert!(
            common.abs() < 5e-5,
            "the vertical term should be a few tens of micrometres, got {common}"
        );
        // 11 mm out of 258 is 4 %: 0.43 mm on a 10.2 mm dip, nearly five of
        // the thinnest papers as the key front sees them (0.08 mm at the front
        // rail, times 258/225). Reading it at the wrong datum is not a
        // rounding question.
        assert!(
            at_front - at_weighing > 0.0004,
            "the difference between the two datums must be big enough to matter"
        );
    }

    /// Level 0: **every reader of the measuring point follows the protocol**,
    /// and by exactly the distance it was moved.
    ///
    /// Moving the point 7 mm further back from the lip must shorten the arm the
    /// closed forms use by 7 mm, move the weight the settle solve puts on the
    /// key by 7 mm along the key's top, and move the datum a height is read at
    /// by the same 7 mm. A consumer still holding its own copy of the point —
    /// the bug a single field exists to prevent — would show up here as one
    /// reading that did not move.
    #[test]
    fn every_reader_of_the_measuring_point_follows_the_protocol() {
        let base = bench();
        let mut moved = base.clone();
        let back = 0.007;
        moved.protocol.measuring_point_from_lip += back;

        // The point itself, against the lip worked from the key's own fields.
        let lip = base.key.front_edge + base.key.keytop_overhang;
        for a in [&base, &moved] {
            assert!(
                (a.measuring_point() - (lip - a.protocol.measuring_point_from_lip)).abs() < 1e-15,
                "the measuring point must be the covering's lip less the protocol's distance"
            );
        }

        // The datum a height is read at.
        assert!(
            (base.datum_x(ReadAt::MeasuringPoint) - moved.datum_x(ReadAt::MeasuringPoint) - back)
                .abs()
                < 1e-15
        );
        // The datum key height and dip are read at does not move: it is the
        // key front's, not the protocol's.
        assert_eq!(
            base.datum_x(ReadAt::FrontEdge),
            moved.datum_x(ReadAt::FrontEdge)
        );

        // The closed forms' arm, level, and the arm the applied weight acts
        // on at any attitude: `cos θ · dx + sin θ · dy`, so moving the point
        // by `back` along the key shortens it by `back · cos θ`.
        assert!((base.measuring_arm() - moved.measuring_arm() - back).abs() < 1e-15);
        for i in -3i32..=3 {
            let s = f64::from(i) * 0.012;
            let c = (1.0 - s * s).sqrt();
            assert!(
                (base.measuring_arm_at(s) - moved.measuring_arm_at(s) - back * c).abs() < 1e-15,
                "at sin θ = {s}"
            );
            // And the torque the settle solve sees from 50 g on the point is
            // that arm, not a copy of it.
            let w = 50.0;
            let from_weight =
                |a: &KeyAssembly| a.net_torque(s, w).unwrap() - a.net_torque(s, 0.0).unwrap();
            assert!(
                (from_weight(&moved) - w * 1e-3 * moved.measuring_arm_at(s)).abs() < 1e-15,
                "the weight on the key must act at the protocol's point, at sin θ = {s}"
            );
        }
    }

    /// Level 4: the dip really is the travel between the **two contacts**.
    ///
    /// Computed here by placing the key at each contact angle, reading the
    /// front of the key and subtracting, and it must agree with what
    /// [`KeyAssembly::travel`] reports. That is the expression `key_dip_at`
    /// evaluates, so this is not an independent route: what it checks is that
    /// both ends are closed contacts and that `travel` reports `key_dip_at`.
    #[test]
    fn the_dip_is_the_distance_the_front_of_the_key_actually_moves() {
        let a = bench();
        let s_rest = a.sin_angle_on_back_rail().unwrap();
        let s_bottom = a.sin_angle_on_front_rail().unwrap();

        // Both ends are contacts, with a closed gap.
        assert!(a.stop_contact(a.back_stop(), s_rest).gap.abs() < 1e-12);
        assert!(a.stop_contact(a.front_stop(), s_bottom).gap.abs() < 1e-12);

        let by_hand = a.key_height_at(ReadAt::FrontEdge, s_rest)
            - a.key_height_at(ReadAt::FrontEdge, s_bottom);
        assert!(
            (by_hand - a.travel().unwrap().key_dip).abs() < 1e-15,
            "the reported dip is not the distance the front of the key moves"
        );
    }
}

#[cfg(test)]
mod vertical_equilibrium_tests {
    use super::super::frame::PunchingStack;
    use super::super::{FrameDesign, KeyDesign, Lead};
    use super::*;

    fn bench() -> KeyAssembly {
        let key = KeyDesign {
            leads: vec![Lead::at(-0.120, 0.030)],
            ..KeyDesign::default()
        };
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target.solve(&key, &frame, 0.0).unwrap();
        frame.balance_punching = PunchingStack::new(balance - 0.0015, 0.0015);
        frame.front_punching = PunchingStack::new(front - 0.0020, 0.0020);
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: balance,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// Level 0: **the vertical forces balance**, and the stop takes its share.
    ///
    /// Whatever is pressing on the key has to be carried by something. When
    /// the key is down on a stop, the bearing and that stop carry it between
    /// them and the two must sum to the total — no more, and none of it lost.
    /// This is the force balance the torque equation never sees, and it is
    /// where a reaction that was computed with the wrong lever arm or the
    /// wrong normal would show up.
    #[test]
    fn the_bearing_and_the_stop_carry_the_whole_weight_between_them() {
        let a = bench();
        let b = 6.0e-3 * a.measuring_arm();
        let mid =
            0.5 * (a.sin_angle_on_back_rail().unwrap() + a.sin_angle_on_front_rail().unwrap());

        for w in [-50.0, -20.0, 0.0, 20.0, 50.0] {
            let settled = a.settle(mid, w, b).unwrap();
            let total = a.bearing_load(w).unwrap();
            let (at_bearing, at_stop) = a.bearing_load_with_stops(settled, w).unwrap();

            assert!(
                (at_bearing + at_stop - total).abs() < 1e-12,
                "at {w} g: bearing {at_bearing} + stop {at_stop} != total {total}"
            );
            assert!(
                at_stop >= -1e-12,
                "a punching can only push: it is carrying {at_stop} kg at {w} g"
            );
        }
    }

    /// A key resting on a stop puts **less** load on its balance bearing than
    /// a floating one.
    ///
    /// Which is why the share matters: with a load-proportional friction model
    /// the bound follows the bearing's load, so a key sitting down has less
    /// friction at its centre. Charging the bearing with the whole weight
    /// would overstate it everywhere the key is touching something — that is,
    /// almost always.
    #[test]
    fn resting_on_a_stop_unloads_the_bearing() {
        let a = bench();
        let b = 6.0e-3 * a.measuring_arm();
        let mid =
            0.5 * (a.sin_angle_on_back_rail().unwrap() + a.sin_angle_on_front_rail().unwrap());

        // A weight well past the down weight puts the key firmly on its
        // punching.
        let down = a.settle(mid, 80.0, b).unwrap();
        assert_eq!(down.held_by, HeldBy::FrontPunching);
        let (on_stop, carried) = a.bearing_load_with_stops(down, 80.0).unwrap();
        assert!(carried > 0.0, "the punching must be carrying something");
        assert!(
            on_stop < a.bearing_load(80.0).unwrap(),
            "and the bearing must therefore carry less than everything"
        );

        // Floating, the bearing carries it all.
        let floating = Settled {
            sin_theta: mid,
            held_by: HeldBy::FrictionAlone,
            stop_reaction: 0.0,
        };
        let (all, none) = a.bearing_load_with_stops(floating, 0.0).unwrap();
        assert!((none).abs() < 1e-15);
        assert!((all - a.bearing_load(0.0).unwrap()).abs() < 1e-15);
    }
}

#[cfg(test)]
mod fault_tests {
    use super::super::frame::KeyKind;
    use super::super::{FrameDesign, KeyDesign};
    use super::*;

    fn assembly(key: KeyDesign, frame: FrameDesign) -> KeyAssembly {
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
                stack_height: 0.0020,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// A frame built for its key reports nothing.
    ///
    /// The baseline every other test here is read against: if the default
    /// assembly carried a fault, the badge would be permanent and therefore
    /// invisible, which is worse than having no badge at all.
    #[test]
    fn a_frame_built_for_its_key_has_nothing_to_report() {
        let key = KeyDesign::default();
        let frame = FrameDesign::default().placed_under(&key);
        assert_eq!(
            assembly(key, frame).faults(KeyKind::Natural),
            Vec::new(),
            "the default keyboard must be one that could be built"
        );
    }

    /// Lengthening a key leaves the front rail where it was bolted, and the
    /// bench says by how much.
    ///
    /// **This is the behaviour, not a defect.** Rails are screwed to a keybed;
    /// nothing about cutting a longer key moves them. A model that quietly
    /// slid the rail along would draw a keyboard that cannot exist and call it
    /// correct — and it would hide the one thing a keyboard designer is here
    /// to find out, which is what else has to change when a key's length does.
    #[test]
    fn lengthening_a_key_leaves_the_front_rail_behind_and_names_the_miss() {
        let frame = FrameDesign::default();
        let short = KeyDesign {
            front_edge: 0.158,
            ..KeyDesign::default()
        };
        let moved = 0.258 - 0.158;

        let faults = assembly(short, frame.clone()).faults(KeyKind::Natural);
        let miss = faults
            .iter()
            .find_map(|f| match f {
                Fault::PinOutsideTheMortise { by } => Some(*by),
                _ => None,
            })
            .expect("the pin is 100 mm from a mortise 8 mm long");
        // The mortise moved with the front edge; the pin did not. What is left
        // over is the travel, less half the slot and half the pin.
        let slack = 0.5 * (0.008 - 0.005);
        assert!(
            (miss - (moved - slack)).abs() < 1e-9,
            "the miss should be {} m, and reads {miss} m",
            moved - slack
        );

        // And the punching has walked off the end of the key with it.
        assert!(
            faults
                .iter()
                .any(|f| matches!(f, Fault::PunchingOffTheUnderside { .. })),
            "a 22 mm washer 100 mm past the end of the key is off the underside"
        );
    }

    /// Placing the frame under the longer key clears both.
    ///
    /// The repair, and the other half of the rule: drift is the default, and
    /// re-placing the frame is the deliberate act that ends it. A fault that
    /// nothing could clear would be a complaint, not a diagnosis.
    #[test]
    fn placing_the_frame_under_the_new_key_clears_the_miss() {
        let key = KeyDesign {
            front_edge: 0.158,
            ..KeyDesign::default()
        };
        let frame = FrameDesign::default().placed_under(&key);
        assert_eq!(assembly(key, frame).faults(KeyKind::Natural), Vec::new());
    }

    /// Cloth in front of the chamfer, and cloth past the end of the key, are
    /// each named.
    ///
    /// The two bounds run opposite ways and it is easy to write one of them
    /// backwards. In front of the chamfer the key can only ever meet the cloth
    /// on the kink; behind the end of the key it meets it on its own corner.
    /// Both are keyboards somebody could build by accident and neither is one
    /// that works.
    #[test]
    fn the_cloth_has_to_fall_inside_the_chamfer_at_both_ends() {
        let key = KeyDesign::default();
        let (chamfer_begins, _) = key.underside_flat();
        let half = FrameDesign::default().sections.back.half_length;

        let forward = FrameDesign {
            back_rail: chamfer_begins,
            ..FrameDesign::default()
        };
        let faults = assembly(key.clone(), forward).faults(KeyKind::Natural);
        assert!(
            faults.iter().any(
                |f| matches!(f, Fault::ClothInFrontOfTheChamfer { by } if (by - half).abs() < 1e-9)
            ),
            "cloth centred on the kink reaches half a rail past it; got {faults:?}"
        );

        let behind = FrameDesign {
            back_rail: key.tail_end,
            ..FrameDesign::default()
        };
        let faults = assembly(key.clone(), behind).faults(KeyKind::Natural);
        assert!(
            faults
                .iter()
                .any(|f| matches!(f, Fault::ClothBehindTheKey { by } if (by - half).abs() < 1e-9)),
            "cloth centred on the tail reaches half a rail past it; got {faults:?}"
        );
    }

    /// Every fault counts from the limit, so repairing one shrinks it to zero
    /// and never to a jump.
    ///
    /// A badge that read "wrong" would say the same thing 1 mm out and 100 mm
    /// out. The number is what tells a reader whether he has mis-typed a
    /// dimension or mis-designed a keyboard.
    #[test]
    fn a_fault_shrinks_to_nothing_as_the_frame_comes_right() {
        let key = KeyDesign::default();
        let mut previous = f64::MAX;
        for front_rail in [0.100_f64, 0.150, 0.200, 0.220, 0.223] {
            let frame = FrameDesign {
                front_rail,
                ..FrameDesign::default()
            };
            let by = assembly(key.clone(), frame)
                .faults(KeyKind::Natural)
                .iter()
                .find_map(|f| match f {
                    Fault::PinOutsideTheMortise { by } => Some(*by),
                    _ => None,
                })
                .unwrap_or(0.0);
            assert!(
                by < previous,
                "moving the rail towards the mortise must shrink the miss, \
                 and {front_rail} m read {by} m after {previous} m"
            );
            previous = by;
        }
        assert!(
            previous > 0.0,
            "the sweep should stop just short of correct"
        );
    }
}
