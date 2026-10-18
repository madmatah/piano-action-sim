//! The three signed primitives every contact row is made of.
//!
//! # Signed at every depth
//!
//! A gap here is a signed distance: positive apart, negative overlapping, and
//! it keeps its sign however deep the overlap. An unsigned distance from a
//! circle's center to a segment, less the radius, grows again past one radius
//! of penetration and turns its normal round: right at contact, and wrong for
//! anything that has to see how deep two parts overlap. So:
//!
//! | Primitive | What it pairs | Gap |
//! |---|---|---|
//! | [`disc_on_face`] | a disc (an arc, or a convex vertex taken as a disc of radius zero whose sector is its exterior angle) against a straight face into whose span the disc's center projects | `(c − a)·n − r` |
//! | [`disc_on_disc`] | two discs, the line of their centers inside both sectors | `‖c_a − c_b‖ − r_a − r_b` |
//! | [`center_on_face`] | the center of a drawn face against the supporting line of a straight face, while the center projects into its span | `(p − a)·n` |
//!
//! The first and the last measure to a line, so they are signed at every
//! depth. The second keeps its sign until the two centers meet, a penetration
//! of `r_a + r_b`, and has no row past that. A corner sunk deeper than a
//! 0.5 mm edge arc's center stands over one of the faces beside the arc, and
//! [`disc_on_face`] reads it there only while the face's normal, turned
//! round, lies in the corner's own angle: past an edge's center a pair can
//! lose a corner altogether (`docs/FOUNDATIONS.md` §6.6).
//!
//! # Conventions
//!
//! Every primitive answers for two sides, A and B, in the world. The normal
//! runs from B toward A; the point is midway between the two surfaces on that
//! normal, which is where the dynamics prototype puts it (its assumption A5):
//! any point on the normal gives the same lever arms, and the midway one is
//! the one the sliding speed is taken at. So the surface point of A is
//! `point + gap/2 · normal` and that of B is `point − gap/2 · normal`.
//!
//! Each primitive returns `None` when its pair of features is not where the
//! two surfaces meet: a disc whose center projects off the face, or whose
//! sector does not hold the direction it would touch along. A contact made of
//! several features is the minimum over the rows its features give
//! ([`crate::contact`]); no primitive here searches.
//!
//! No transcendental is called: additions, products and square roots, which
//! IEEE-754 rounds exactly, so every gap is the same bits on every machine.

use super::{Dir, Point};

/// A disc: a center, a radius, and the sector of directions from the center
/// its boundary covers, counter-clockwise from `from` to `to`.
///
/// An arc of a drawn outline is a disc whose sector is the arc's sweep. A
/// convex vertex is a disc of radius zero whose sector is its exterior angle,
/// from the outward normal of the segment that ends there to that of the
/// segment that starts there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disc {
    /// The center, meters.
    pub center: Point,
    /// The radius, meters. Zero for a vertex.
    pub radius: f64,
    /// Where the sector starts, seen from the center.
    pub from: Dir,
    /// Where it ends, counter-clockwise from `from`.
    pub to: Dir,
}

/// A straight face: a segment from `a` to `b`, and the side it presents.
///
/// `normal` points out of the material the face bounds, toward whatever may
/// touch it: the right-hand normal of a segment of a counter-clockwise
/// outline, the left-hand one of a cut's, whose material lies outside it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// Where the segment starts.
    pub a: Point,
    /// Where it ends.
    pub b: Point,
    /// The side it presents, of unit length.
    pub normal: Dir,
}

/// Where two features of two sides meet: what a contact row is built from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Proximity {
    /// Signed gap, meters: positive apart, negative overlapping.
    pub gap: f64,
    /// Unit normal, from side B toward side A.
    pub normal: Dir,
    /// The point midway between the two surfaces on the normal.
    pub point: Point,
}

/// How far a direction may test outside a sector's boundary and still be
/// inside it: eight units in the last place of a unit vector.
///
/// Every direction a primitive compares has been turned from its body's
/// frame into the world, which leaves a rounding of a few ε in each of its
/// components; a direction lying exactly on a sector's boundary, as the
/// normal of a face bedded flat on a strip lies on its edge arcs', may
/// therefore test that far outside. Admitting it changes the gap by the
/// radius times the square of the slack, nothing at all.
pub const SECTOR_SLACK: f64 = 8.0 * f64::EPSILON;

fn sub(u: Point, v: Point) -> Point {
    Point::new(u.x - v.x, u.y - v.y)
}

fn dot(u: Point, v: Point) -> f64 {
    u.x * v.x + u.y * v.y
}

fn cross(u: Point, v: Point) -> f64 {
    u.x * v.y - u.y * v.x
}

/// Whether the unit vector `u` lies in the sector from `from` to `to`,
/// counter-clockwise, its boundary included within [`SECTOR_SLACK`].
///
/// Under a half turn the sector is the intersection of two half-planes, on
/// the side its bisector points to: a sector narrower than the slack, the
/// exterior angle of a vertex whose segments run on all but straight, would
/// otherwise hold the direction opposite it too. Over a half turn it is
/// everything but the inside of its complement; at exactly a half turn it is
/// one half-plane. A sector of no sweep holds nothing: a vertex whose two
/// segments run straight on is no feature.
pub fn in_sector(u: Point, from: Dir, to: Dir) -> bool {
    let (f, t) = (from.vector(), to.vector());
    let sweep = cross(f, t);
    if sweep > 0.0 {
        cross(f, u) >= -SECTOR_SLACK && cross(u, t) >= -SECTOR_SLACK && dot(u, f) + dot(u, t) > 0.0
    } else if sweep < 0.0 {
        !(cross(t, u) > SECTOR_SLACK && cross(u, f) > SECTOR_SLACK)
    } else if dot(f, t) < 0.0 {
        cross(f, u) >= -SECTOR_SLACK
    } else {
        false
    }
}

/// Where the projection of `c` falls along the face, `0` at `a` and `1` at
/// `b`, or `None` for a face of no length.
fn along(c: Point, face: &Face) -> Option<f64> {
    let edge = sub(face.b, face.a);
    let len2 = dot(edge, edge);
    if len2 > 0.0 && len2.is_finite() {
        Some(dot(sub(c, face.a), edge) / len2)
    } else {
        None
    }
}

/// A disc of side A against a straight face of side B.
///
/// The disc's center has to project into the face's span, and the direction
/// from the center toward the face, the face's normal turned round, has to
/// lie in the disc's sector: otherwise the point of the disc that would touch
/// is not on the drawn arc, or not at the drawn vertex, and another feature
/// carries the contact. The gap is the center's signed height above the
/// face's line less the radius; the normal is the face's. With the roles the
/// other way round, a disc of B against a face of A, the gap and the point
/// are the same and the normal turns round: [`Proximity::reversed`].
pub fn disc_on_face(disc: &Disc, face: &Face) -> Option<Proximity> {
    let t = along(disc.center, face)?;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    let n = face.normal.vector();
    if !in_sector(Point::new(-n.x, -n.y), disc.from, disc.to) {
        return None;
    }
    let height = dot(sub(disc.center, face.a), n);
    let gap = height - disc.radius;
    let back = disc.radius + 0.5 * gap;
    Some(Proximity {
        gap,
        normal: face.normal,
        point: Point::new(disc.center.x - back * n.x, disc.center.y - back * n.y),
    })
}

/// A disc of side A against a disc of side B.
///
/// The line of centers has to leave A's center inside A's sector, pointing
/// toward B, and B's inside B's, pointing toward A: then the two points it
/// crosses the circles at are on the drawn arcs, or at the drawn vertices.
/// Two centers that coincide have no line between them, and no row: that is
/// a penetration of the two radii added, where another feature carries the
/// contact.
pub fn disc_on_disc(a: &Disc, b: &Disc) -> Option<Proximity> {
    let d = sub(a.center, b.center);
    let normal = Dir::new(d.x, d.y)?;
    let n = normal.vector();
    if !in_sector(Point::new(-n.x, -n.y), a.from, a.to) || !in_sector(n, b.from, b.to) {
        return None;
    }
    let gap = dot(d, d).sqrt() - a.radius - b.radius;
    let out = b.radius + 0.5 * gap;
    Some(Proximity {
        gap,
        normal,
        point: Point::new(b.center.x + out * n.x, b.center.y + out * n.y),
    })
}

/// The center of a drawn face of side B against a straight face of side A.
///
/// The rigid idealization of a felt that spreads its load: the key bears on
/// a washer at its center, whatever its attitude, and the gap is the center's
/// signed height above the key's face, measured along the key's normal. The
/// center has to project into the face's span: a punching standing past the
/// end of the key's underside has no row.
pub fn center_on_face(center: Point, face: &Face) -> Option<Proximity> {
    let t = along(center, face)?;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    let n = face.normal.vector();
    let gap = dot(sub(center, face.a), n);
    let half = 0.5 * gap;
    Some(Proximity {
        gap,
        normal: Dir::new(-n.x, -n.y)?,
        point: Point::new(center.x - half * n.x, center.y - half * n.y),
    })
}

impl Proximity {
    /// The same meeting seen with the two sides swapped: the gap and the
    /// point unchanged, the normal turned round.
    pub fn reversed(self) -> Self {
        let n = self.normal.vector();
        Self {
            gap: self.gap,
            normal: Dir::new(-n.x, -n.y).unwrap_or(self.normal),
            point: self.point,
        }
    }

    /// The point of side A's surface: midway plus half the gap along the
    /// normal.
    pub fn on_a(&self) -> Point {
        let n = self.normal.vector();
        let h = 0.5 * self.gap;
        Point::new(self.point.x + h * n.x, self.point.y + h * n.y)
    }

    /// The point of side B's surface: midway less half the gap along the
    /// normal.
    pub fn on_b(&self) -> Point {
        let n = self.normal.vector();
        let h = 0.5 * self.gap;
        Point::new(self.point.x - h * n.x, self.point.y - h * n.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: Dir = Dir::Y;

    /// A horizontal face along `y = 0` from `x = −1` to `x = 1`, presenting
    /// its top.
    fn floor() -> Face {
        Face {
            a: Point::new(-1.0, 0.0),
            b: Point::new(1.0, 0.0),
            normal: UP,
        }
    }

    /// A disc whose sector is all but a sliver round a direction no test
    /// here touches along: as near a whole circle as a sector can be.
    fn round(center: Point, radius: f64) -> Disc {
        Disc {
            center,
            radius,
            from: Dir::new(0.8, -0.6 + 1e-9).expect("a direction"),
            to: Dir::new(0.8, -0.6 - 1e-9).expect("a direction"),
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 4.0 * f64::EPSILON * (1.0 + a.abs().max(b.abs()))
    }

    /// A disc above a face: the gap is its height less its radius, the
    /// normal the face's, and the point midway between the disc's bottom
    /// and the face.
    #[test]
    fn a_disc_above_a_face_has_its_height_less_its_radius_for_gap() {
        let disc = round(Point::new(0.25, 0.75), 0.25);
        let p = disc_on_face(&disc, &floor()).expect("the center is over the face");
        assert!(close(p.gap, 0.5), "{p:?}");
        assert_eq!(p.normal, UP);
        assert!(close(p.point.x, 0.25) && close(p.point.y, 0.25), "{p:?}");
        assert!(close(p.on_a().y, 0.5) && close(p.on_b().y, 0.0), "{p:?}");
    }

    /// Consistency: **signed at every depth.** Driven down through the face
    /// from two radii above to five below, the gap falls by exactly what the
    /// disc moved, and never turns back: the built circle-against-line gap
    /// does, past one radius.
    #[test]
    fn a_disc_driven_through_a_face_keeps_its_sign_at_every_depth() {
        let r = 0.1;
        let mut previous = f64::INFINITY;
        for k in 0..=70 {
            let y = 2.0 * r - f64::from(k) * 0.01;
            let p = disc_on_face(&round(Point::new(0.0, y), r), &floor()).expect("over the face");
            assert!(
                close(p.gap, y - r),
                "at a height of {y} the gap is {}",
                p.gap
            );
            assert!(p.gap < previous, "the gap turned back at {y}");
            assert_eq!(p.normal, UP, "and the normal did not turn round");
            previous = p.gap;
        }
        assert!(previous < -4.0 * r, "the sweep went deeper than the radius");
    }

    /// A disc whose center projects off the face has no row: the face's end
    /// carries that contact, as a vertex or an arc of its own.
    #[test]
    fn a_disc_off_the_end_of_a_face_has_no_row() {
        assert!(disc_on_face(&round(Point::new(1.5, 0.3), 0.1), &floor()).is_none());
        assert!(disc_on_face(&round(Point::new(1.0, 0.3), 0.1), &floor()).is_some());
    }

    /// An arc touches a face only through its own sweep: a quarter arc
    /// facing up and right reaches a face below it at its start, never
    /// through the missing quarters.
    #[test]
    fn an_arc_touches_only_through_its_own_sweep() {
        let quarter = Disc {
            center: Point::new(0.0, 0.5),
            radius: 0.1,
            from: Dir::X,
            to: Dir::Y,
        };
        assert!(
            disc_on_face(&quarter, &floor()).is_none(),
            "the face below is touched by no point of a quarter facing up"
        );
        let ceiling = Face {
            a: Point::new(1.0, 1.0),
            b: Point::new(-1.0, 1.0),
            normal: Dir::NEG_Y,
        };
        let p = disc_on_face(&quarter, &ceiling).expect("the quarter faces it");
        assert!(close(p.gap, 0.4), "{p:?}");
    }

    /// The boundary of a sector is in it, and a rounding past it too, so a
    /// face bedded flat on a strip still meets both of its edge arcs.
    #[test]
    fn a_direction_on_a_sector_boundary_is_inside_it() {
        assert!(in_sector(UP.vector(), Dir::X, Dir::Y));
        assert!(in_sector(Point::new(-1e-17, 1.0), Dir::X, Dir::Y));
        assert!(!in_sector(Point::new(-1e-9, 1.0), Dir::X, Dir::Y));
        // Over a half turn, three quarters from straight up round to
        // straight right: everything but the inside of its complement.
        assert!(in_sector(Point::new(-1.0, 0.0), Dir::Y, Dir::X));
        assert!(in_sector(Point::new(0.0, -1.0), Dir::Y, Dir::X));
        assert!(!in_sector(Point::new(0.6, 0.8), Dir::Y, Dir::X));
        // Exactly a half turn: one half-plane.
        assert!(in_sector(Point::new(0.0, -1.0), Dir::NEG_X, Dir::X));
        assert!(!in_sector(Point::new(0.0, 1.0), Dir::NEG_X, Dir::X));
        // No sweep at all: nothing.
        assert!(!in_sector(UP.vector(), UP, UP));
    }

    /// A sector narrower than the slack, the exterior angle of a vertex whose
    /// two segments run on all but straight, holds its own direction and not
    /// the one opposite it: such a vertex touches only from outside.
    #[test]
    fn a_sector_narrower_than_the_slack_holds_nothing_behind_it() {
        let (from, to) = (Dir::NEG_Y, Dir::new(1e-16, -1.0).expect("a direction"));
        assert!(cross(from.vector(), to.vector()) > 0.0);
        assert!(cross(from.vector(), to.vector()) < SECTOR_SLACK);
        assert!(in_sector(Point::new(0.0, -1.0), from, to));
        assert!(!in_sector(Point::new(0.0, 1.0), from, to));
    }

    /// Two discs: the distance between centers less both radii, the normal
    /// from B's center to A's, the point midway between the circles.
    #[test]
    fn two_discs_meet_on_their_line_of_centers() {
        let a = round(Point::new(3.0, 4.0), 1.0);
        let b = round(Point::new(0.0, 0.0), 2.0);
        let p = disc_on_disc(&a, &b).expect("both sectors hold the line");
        assert!(close(p.gap, 2.0), "{p:?}");
        let n = p.normal.vector();
        assert!(close(n.x, 0.6) && close(n.y, 0.8), "{p:?}");
        // B's surface 2 out, A's 4 out: midway at 3.
        assert!(close(p.point.x, 1.8) && close(p.point.y, 2.4), "{p:?}");
    }

    /// Disc against disc keeps its sign while the centers stay apart.
    #[test]
    fn two_discs_overlapping_have_a_negative_gap_until_their_centers_meet() {
        let b = round(Point::new(0.0, 0.0), 0.3);
        for k in 1..10 {
            let x = f64::from(k) * 0.05;
            let p = disc_on_disc(&round(Point::new(x, 0.0), 0.2), &b).expect("apart");
            assert!(close(p.gap, x - 0.5), "{p:?}");
        }
        assert!(disc_on_disc(&round(Point::new(0.0, 0.0), 0.2), &b).is_none());
    }

    /// A vertex is a disc of radius zero whose sector is its exterior angle:
    /// the top corner of a block meets a ceiling straight above it, and not
    /// one beside it.
    #[test]
    fn a_vertex_touches_through_its_exterior_angle() {
        let corner = Disc {
            center: Point::new(0.0, 0.0),
            radius: 0.0,
            from: Dir::X,
            to: Dir::Y,
        };
        let above = Disc {
            center: Point::new(0.0, 2.0),
            radius: 0.5,
            from: Dir::NEG_X,
            to: Dir::X,
        };
        let p = disc_on_disc(&corner, &above).expect("straight above, in both sectors");
        assert!(close(p.gap, 1.5), "{p:?}");
        let behind = Disc {
            center: Point::new(-2.0, 0.0),
            ..above
        };
        assert!(
            disc_on_disc(&corner, &behind).is_none(),
            "a corner facing up and right is not touched from behind"
        );
    }

    /// The center of a face against another face: its signed height above
    /// that face's line, with no radius, the normal from the center's side.
    #[test]
    fn a_center_bearing_reads_the_height_of_the_center() {
        let key = Face {
            a: Point::new(-1.0, 1.0),
            b: Point::new(1.0, 1.0),
            normal: Dir::NEG_Y,
        };
        let p = center_on_face(Point::new(0.5, 0.25), &key).expect("under the face");
        assert!(close(p.gap, 0.75), "{p:?}");
        assert_eq!(p.normal, UP, "from the center's side, B, toward the key, A");
        assert!(close(p.on_a().y, 1.0) && close(p.on_b().y, 0.25), "{p:?}");
        let deep = center_on_face(Point::new(0.5, 3.0), &key).expect("under the face");
        assert!(close(deep.gap, -2.0), "signed at every depth: {deep:?}");
        assert!(center_on_face(Point::new(1.5, 0.25), &key).is_none());
    }

    /// Swapping the sides keeps the gap and the point and turns the normal.
    #[test]
    fn reversing_a_meeting_turns_its_normal_round() {
        let p = disc_on_face(&round(Point::new(0.0, 0.5), 0.25), &floor()).unwrap();
        let r = p.reversed();
        assert_eq!(r.gap.to_bits(), p.gap.to_bits());
        assert_eq!(r.point, p.point);
        assert_eq!(r.normal, Dir::NEG_Y);
        assert_eq!(r.on_a(), p.on_b());
    }
}
