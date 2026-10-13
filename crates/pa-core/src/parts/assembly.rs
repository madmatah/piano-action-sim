//! The key on its frame as the drawing of a frame whose stops fight reads it:
//! stops of its own, and the key riding on two of them.
//!
//! # What is left here, and until when
//!
//! The key is put on its frame by the static solve on the contacts read off
//! the drawn parts ([`crate::measure::seat`]), and every reading of the sheet
//! is a procedure run there ([`crate::measure::weighing`]). One thing that
//! solve cannot yet do is put a key on a frame whose stops fight: on a pivot
//! such a key has no attitude at all, and on a real keyboard it rides on its
//! front stop and its cloth, lifted off its balance punching. Until the key
//! has that vertical freedom, the bench draws it so from the stops here,
//! built from the same fields as the drawn parts:
//! [`KeyAssembly::riding_on_both_stops`] puts its underside, taken as one
//! straight line, on the common tangent of its two outer stops.
//! `every_stop_face_is_a_declared_range_of_a_drawn_part` holds every face
//! these stops read to a declared range of a drawn part.
//!
//! [`KeyAssembly`] itself is what the bench holds: the key, its frame, where
//! it bears, and the protocol it is weighed by.

use super::{frame::FrameDesign, KeyDesign};
use crate::geometry::{KeyBearing, Point};
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

/// How far a stop's edge may stand from the line a riding key is drawn on and
/// still count as on it, metres: a tenth of a micron, a thousandth of the
/// thinnest paper in the drawer.
pub(crate) const CONTACT_TOLERANCE: f64 = 1e-7;

/// The angle a mechanism holds the key at, radians, positive nose-up, for an
/// attitude of these stops, whose sine is positive front-down: where the
/// bench draws a key riding on its two outer stops. One `asin`, from `libm`,
/// so the angle is the same bits on every machine.
pub fn nose_up_angle(sin_theta: f64) -> f64 {
    -libm::asin(sin_theta)
}

/// The edge radius taken for a punching or a cloth, metres.
///
/// Half a millimetre: small enough that the contact point sits essentially over
/// the punching, large enough to be a real curve rather than a numerical
/// fiction. It is a shape, not a setting: the drawn cloth's edges have it, and
/// so do the stops a frame whose stops fight is drawn on.
pub const PUNCHING_EDGE_RADIUS: f64 = 0.0005;

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
    /// assembly: the jig, the walks, the key friction weights and the touch
    /// weight all read it ([`TouchweightProtocol::measuring_point`]), so none
    /// can be weighed at a point the others are not. It is not a dimension of
    /// the key, which is why it is not on [`KeyDesign`].
    pub protocol: TouchweightProtocol,
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

    /// Whichever of the two a key riding on its stops meets first on its way
    /// down: the higher. The static solve finds which binds from the drawn
    /// pin and mortise instead, and sees a pin standing outside its mortise,
    /// which this does not.
    pub fn front_stop(&self) -> Stop {
        let punching = self.punching_stop();
        match self.mortise_stop() {
            Some(mortise) if mortise.top > punching.top => mortise,
            _ => punching,
        }
    }

    /// Whether it is the pin in its mortise rather than the punching that a
    /// key riding on its stops is drawn on: the higher of the two.
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

    /// Where the key goes when its two stops leave it nowhere to turn.
    ///
    /// Returns the attitude and **how far the key has lifted off its balance
    /// punching**, metres. Pure kinematics, deliberately: with the bearing
    /// modelled as a pivot there is no statics here, and the seated pose
    /// refuses such a frame, naming the stops that fight; but the key still
    /// has a definite place to be, and drawing it anywhere else means drawing
    /// wood through wood.
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

    /// **The measuring point**, metres in front of the balance pin: the
    /// covering's lip less the protocol's distance.
    ///
    /// The protocol's, on this key ([`TouchweightProtocol::measuring_point`]),
    /// where the key friction weight a bench is set from is read.
    pub fn measuring_point(&self) -> f64 {
        self.protocol.measuring_point(&self.key)
    }
}

#[cfg(test)]
mod bridge_tests {
    use super::super::frame::{papers_for, FrameRanges, KeyKind};
    use super::super::key::KeyRanges;

    use super::*;
    use crate::contact::{Part, SurfaceRef};
    use crate::geometry::Seg;
    use crate::mechanism::BodyId;

    /// The default key regulated on the default frame, as the bench builds
    /// it.
    fn regulated() -> KeyAssembly {
        crate::parts::fixtures::regulated()
    }

    /// The regulated bench and the four ways the golden sheet's fixtures push
    /// it, with a cloth hung off the end of the key besides: a frame whose
    /// stops fight, a pin bottoming in its mortise, a key shortened under its
    /// frame.
    fn assemblies() -> Vec<KeyAssembly> {
        let base = regulated();
        let mut wedged = base.clone();
        wedged.frame.back_rail_height =
            wedged.frame.balance_bearing_height() + 0.005 - wedged.frame.back_rail_cloth;
        let mut proud = base.clone();
        proud.frame.sections.front.pin_height += 0.001;
        let mut short = base.clone();
        short.key.front_edge -= 0.018;
        let mut overhung = base.clone();
        overhung.frame.back_rail = -0.232;
        vec![base, wedged, proud, short, overhung]
    }

    /// The parts the bench draws for an assembly: the frame's, its papers
    /// laid from the drawer, then the key's.
    fn drawn(a: &KeyAssembly) -> (Vec<Part>, KeyRanges, FrameRanges) {
        let mut parts = Vec::new();
        let balance = papers_for(a.frame.balance_punching.paper);
        let front = papers_for(a.frame.front_punching.paper);
        let frame = a
            .frame
            .parts(&a.key, KeyKind::Natural, &balance, &front, &mut parts);
        let key = a.key.parts(BodyId(1), &mut parts);
        (parts, key, frame)
    }

    fn same(p: Point, q: Point) -> bool {
        p.x.to_bits() == q.x.to_bits() && p.y.to_bits() == q.y.to_bits()
    }

    /// Whether two numbers are the same, to the bit but for the sign of zero:
    /// adding `0.0` turns `−0.0` into `0.0` and leaves every other number as
    /// it is.
    fn equal(a: f64, b: f64) -> bool {
        (a + 0.0).to_bits() == (b + 0.0).to_bits()
    }

    /// The centre of a turned part's drawn top: the point on its axis where the
    /// two segments of its silhouette's top meet.
    fn centre_of(range: &SurfaceRef, parts: &[Part]) -> Point {
        let segs = range.segs(parts);
        assert_eq!(
            segs.len(),
            2,
            "a turned top is two segments, either side of the axis"
        );
        assert!(
            same(segs[0].end(), segs[1].start()),
            "they meet on the axis"
        );
        segs[0].end()
    }

    /// Consistency, and the check of `CLAUDE.md` rule 1 while the stops read a
    /// construction of their own: **every face a stop reads coincides with a
    /// declared range of a drawn part.**
    ///
    /// The stops read the key's underside from [`KeyDesign::underside_corners`]
    /// and the frame's faces from its bearing heights; the drawn parts are
    /// built from the same fields. This holds the two together, on the bench
    /// and on frames pushed the ways the golden sheet pushes them:
    ///
    /// - the underside's three corners are the drawn chain's ends and kink,
    ///   the chain unbroken across the keystick and the key front;
    /// - the back stop's two edges are the cloth's drawn arcs, centre and
    ///   radius, and its face the cloth's face, along the key to the bit and
    ///   in height within the one rounding an arc's end takes, `ε` of it;
    /// - the front punching, a centre bearing, has its point at the centre of
    ///   the felt's drawn top, on the washer's axis, as the balance bearing the
    ///   key turns on has on its own; to the bit, but for the sign of zero,
    ///   since the bearing's `−0.0` is the axis's `0.0`;
    /// - the mortise stop is the pin's drawn top less the mortise's, to the
    ///   bit, at the middle of the pin's drawn top. That middle is the mean of
    ///   the top's two ends, which round once each and their sum once: within
    ///   `ε` of the stop's distance from the balance pin, to first order, and
    ///   `2ε` bounds it.
    ///
    /// A stop moved by a tenth of a millimetre along the key or up from it
    /// fails here.
    ///
    /// Run on the natural, the bench's only kind: a sharp's punchings stand on
    /// the second row, and the front punching's stop does not follow them.
    #[test]
    fn every_stop_face_is_a_declared_range_of_a_drawn_part() {
        for a in assemblies() {
            let (parts, key, frame) = drawn(&a);

            let [tail, kink, nose] = a.key.underside_corners();
            let chain: Vec<Seg> = key
                .underside
                .iter()
                .flat_map(|range| range.segs(&parts))
                .collect();
            assert!(same(chain[0].start(), tail), "the tail");
            assert!(same(chain[0].end(), kink), "the kink");
            assert!(same(chain[chain.len() - 1].end(), nose), "the front");
            for pair in chain.windows(2) {
                assert!(same(pair[0].end(), pair[1].start()), "the underside breaks");
            }
            for seg in &chain[1..] {
                assert!(matches!(seg, Seg::Line { .. }));
                assert_eq!(seg.end().y.to_bits(), 0.0_f64.to_bits(), "off the flat");
            }

            let back = a.back_stop();
            let [edge_back, edge_front] = back.edges();
            let (face_back, face_front) = back.face();
            match frame.cloth_top.segs(&parts)[..] {
                [Seg::Arc {
                    c: front_centre,
                    r: front_radius,
                    ..
                }, Seg::Line {
                    a: face_a,
                    b: face_b,
                }, Seg::Arc {
                    c: back_centre,
                    r: back_radius,
                    ..
                }] => {
                    assert!(same(front_centre, edge_front) && same(back_centre, edge_back));
                    assert_eq!(front_radius.to_bits(), back.edge_radius.to_bits());
                    assert_eq!(back_radius.to_bits(), back.edge_radius.to_bits());
                    assert_eq!(face_a.x.to_bits(), face_front.x.to_bits());
                    assert_eq!(face_b.x.to_bits(), face_back.x.to_bits());
                    for y in [face_a.y, face_b.y] {
                        assert!(
                            (y - back.top).abs() <= f64::EPSILON * back.top,
                            "the cloth's face is drawn at {y} m, the stop's at {} m",
                            back.top
                        );
                    }
                }
                ref other => panic!("the cloth's top is an edge, a face and an edge: {other:?}"),
            }

            let punching = a.punching_stop();
            let centre = centre_of(&frame.front_bearing, &parts);
            assert!(
                equal(punching.at, centre.x) && equal(punching.top, centre.y),
                "the front punching bears at ({}, {}) m, its felt's drawn top is centred at {centre:?}",
                punching.at,
                punching.top
            );
            let centre = centre_of(&frame.balance_bearing, &parts);
            assert!(
                equal(a.bearing_x(), centre.x) && equal(a.bearing_y(), centre.y),
                "the key turns at ({}, {}) m, its balance felt's drawn top is centred at {centre:?}",
                a.bearing_x(),
                a.bearing_y()
            );

            let mortise = a.mortise_stop().expect("the front rail has a pin");
            let pin_top = frame
                .front_pin_top
                .as_ref()
                .expect("a pin has a top")
                .segs(&parts);
            let mortise_top = key.front_mortise_top.segs(&parts)[0];
            assert_eq!(
                mortise.top.to_bits(),
                (pin_top[0].start().y - mortise_top.start().y).to_bits(),
                "the key cannot go below the pin's drawn top less the mortise's drawn depth"
            );
            let (lo, hi) = pin_top
                .iter()
                .flat_map(|s| [s.start().x, s.end().x])
                .fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
            let middle = 0.5 * (lo + hi);
            assert!(
                (middle - mortise.at).abs() <= 2.0 * f64::EPSILON * mortise.at.abs(),
                "the mortise stop stands at {} m, the middle of the pin's drawn top at {middle} m",
                mortise.at
            );
        }
    }
}
