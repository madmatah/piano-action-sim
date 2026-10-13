//! Exact planar contact kinematics.
//!
//! # What this module computes, and why it is the heart of the project
//!
//! A contact between two bodies of the action is a planar problem with a
//! closed-form answer. For each contact we need four things: the signed `gap`,
//! the unit `normal`, the contact `point`, and — the one that matters — the
//! derivative of the gap with respect to each body's angle.
//!
//! That derivative *is* the instantaneous lever arm. It is not an auxiliary
//! quantity computed from a lever ratio: the lever ratio is what you get by
//! dividing two of them. And it varies through the stroke because the contact
//! point migrates along the surfaces as the bodies turn. That migration is the
//! physical origin of the changing leverage a technician feels, which is why
//! the contacts use analytic primitives rather than a collision library's
//! approximations.
//!
//! # Fixed pivots only
//!
//! Each body is placed by a [`Frame`]: a pivot standing still and an angle
//! about it. That fits the key, the wippen, the hammer and the damper lever,
//! which turn about pins in fixed flanges and rails. It does not fit the jack
//! or the repetition lever, whose pins ride on the wippen: their pivots move,
//! and a gap that involves one of them depends on the wippen's angle as well
//! as on the two angles a query reports. Likewise `Frame::velocity_at` takes
//! the pivot as fixed, so on a riding body the sliding velocity it gives
//! leaves out the pivot's own motion.
//!
//! # The identity this module rests on
//!
//! For a contact whose unit normal is `n` and whose contact point is `P`,
//! between a body A pivoting at `Oa` and a body B pivoting at `Ob`:
//!
//! ```text
//!     d(gap)/d(theta_a) = (P - Oa) x n
//!     d(gap)/d(theta_b) = -(P - Ob) x n
//! ```
//!
//! where `x` is the scalar planar cross product. In words: **the lever arm of
//! the contact normal about the pivot**, which is exactly the moment arm a
//! technician reasons about. Two consequences worth knowing, both tested below:
//!
//! - It depends only on the normal's line of action, so it is **independent of
//!   the circle's radius**. Changing a knuckle's radius shifts the gap; it does
//!   not, by itself, change the leverage at a given contact point.
//! - It is exact, so it must agree with a central finite difference of `gap` to
//!   the limit of round-off. That agreement is the level-0 test that catches
//!   essentially every sign and lever-arm error before a solver exists.
//!
//! # No transcendentals here
//!
//! Nothing in this module calls `sin` or `cos`. Rotations arrive as a
//! [`Rot`] — a cosine/sine pair — so a query uses only add, multiply, divide
//! and `sqrt`. `sqrt` is correctly rounded by IEEE-754 and therefore the same
//! on every platform; `sin` and `cos` come from the platform's maths library
//! and can differ between them. The one constructor that takes an angle,
//! `Rot::from_angle`, exists only in the tests.

use super::{ContactQuery, Point};

/// Scalar planar cross product, `u x v`.
#[inline]
fn cross(u: Point, v: Point) -> f64 {
    u.x * v.y - u.y * v.x
}

/// Dot product.
#[inline]
fn dot(u: Point, v: Point) -> f64 {
    u.x * v.x + u.y * v.y
}

/// Rotate a vector a quarter turn counter-clockwise.
///
/// This is also the derivative of a rotation with respect to its angle, which
/// is why it appears in every velocity and every lever arm below.
#[inline]
fn perp(v: Point) -> Point {
    Point::new(-v.y, v.x)
}

#[inline]
fn sub(u: Point, v: Point) -> Point {
    Point::new(u.x - v.x, u.y - v.y)
}

/// A planar rotation, carried as its cosine and sine rather than as an angle.
///
/// Keeping the pair rather than the angle is what lets the contact kinematics
/// avoid transcendentals entirely: whoever holds an angle converts it once,
/// and every contact query then costs only multiplies and adds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rot {
    cos: f64,
    sin: f64,
}

impl Rot {
    /// The identity rotation.
    pub const IDENTITY: Self = Self { cos: 1.0, sin: 0.0 };

    /// Builds a rotation from an already-computed cosine and sine.
    ///
    /// Prefer this in tests and wherever the pair is exact: it keeps the test
    /// arithmetic exact too, so an assertion failure means a wrong formula
    /// rather than an accumulated rounding error.
    ///
    /// In debug builds the pair is checked for normalization.
    #[inline]
    pub fn from_cos_sin(cos: f64, sin: f64) -> Self {
        debug_assert!(
            (cos * cos + sin * sin - 1.0).abs() < 1e-12,
            "Rot must be built from a unit (cos, sin) pair"
        );
        Self { cos, sin }
    }

    /// Builds a rotation from an angle in radians.
    ///
    /// Test-only: it calls the platform's `sin` and `cos`, which the crate's
    /// lint refuses in production code, to build reference rotations.
    #[cfg(test)]
    #[allow(clippy::disallowed_methods)]
    pub fn from_angle(theta: f64) -> Self {
        Self {
            cos: theta.cos(),
            sin: theta.sin(),
        }
    }

    /// Rotates a direction vector. No translation.
    #[inline]
    pub fn apply(&self, v: Point) -> Point {
        Point::new(
            self.cos * v.x - self.sin * v.y,
            self.sin * v.x + self.cos * v.y,
        )
    }
}

/// The placement of one rotating body: where its pivot is, how far it has
/// turned, and how fast.
///
/// `pivot_local` is the pivot expressed in the part's own outline frame, so an
/// outline may be drawn in whatever frame is convenient for the draughtsman;
/// `pivot_world` is where that pivot actually sits in the action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// The pivot, in the part's outline frame.
    pub pivot_local: Point,
    /// The pivot, in world coordinates.
    pub pivot_world: Point,
    /// Rotation of the part about its pivot.
    pub rot: Rot,
    /// Angular velocity about the pivot, rad/s, counter-clockwise positive.
    pub omega: f64,
}

impl Frame {
    /// A frame at rest, unrotated, whose outline frame and world frame share an
    /// origin at the pivot.
    pub const fn at(pivot_world: Point) -> Self {
        Self {
            pivot_local: Point::new(0.0, 0.0),
            pivot_world,
            rot: Rot::IDENTITY,
            omega: 0.0,
        }
    }

    /// Maps a point from the part's outline frame into world coordinates.
    #[inline]
    pub fn world(&self, local: Point) -> Point {
        let r = self.rot.apply(sub(local, self.pivot_local));
        Point::new(self.pivot_world.x + r.x, self.pivot_world.y + r.y)
    }

    /// Velocity of the material point currently at world position `p`, m/s.
    #[inline]
    fn velocity_at(&self, p: Point) -> Point {
        let r = perp(sub(p, self.pivot_world));
        Point::new(self.omega * r.x, self.omega * r.y)
    }
}

/// Smallest separation below which the normal direction is taken from the line
/// rather than from the centre-to-foot vector, metres.
///
/// Only reached if a circle's centre lands exactly on the line it touches,
/// which is geometrically possible but physically means the bodies have passed
/// through each other. Returning the line's own normal keeps the query total
/// instead of producing a `NaN` that would surface much later, far from its
/// cause.
const DEGENERATE_DISTANCE: f64 = 1e-15;

/// Contact between a circle carried by body A and a line segment carried by
/// body B.
///
/// This is the workhorse: ten of the seventeen contacts of the action are of
/// this kind, including every felt punching, the let-off button against the
/// jack, the drop screw against the repetition lever, the back check against
/// the hammer tail, and the string against the hammer.
///
/// `c_local` and `r` describe the circle in body A's outline frame;
/// `a_local` and `b_local` are the segment's endpoints in body B's outline
/// frame. The returned normal points from B towards A, and `gap` is positive
/// when the two are separated.
///
/// The segment is finite: when the circle's centre projects beyond either end,
/// the nearest feature is that endpoint and the query degrades smoothly to a
/// circle-against-point contact. That case is not a corner case to be
/// tolerated but a configuration the action really visits — the jack nose
/// sliding off the edge of the let-off button is exactly it.
pub fn circle_vs_line(
    circle: &Frame,
    c_local: Point,
    r: f64,
    line: &Frame,
    a_local: Point,
    b_local: Point,
) -> ContactQuery {
    let c = circle.world(c_local);
    let pa = line.world(a_local);
    let pb = line.world(b_local);

    let edge = sub(pb, pa);
    let len2 = dot(edge, edge);

    // Foot of the perpendicular, clamped to the segment. `s` outside [0, 1]
    // means an endpoint carries the contact.
    let s = if len2 > 0.0 {
        (dot(sub(c, pa), edge) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let foot = Point::new(pa.x + s * edge.x, pa.y + s * edge.y);

    let delta = sub(c, foot);
    let dist = dot(delta, delta).sqrt();

    let normal = if dist > DEGENERATE_DISTANCE {
        Point::new(delta.x / dist, delta.y / dist)
    } else if len2 > 0.0 {
        // Centre on the line: fall back to the line's own left normal.
        let inv = len2.sqrt();
        perp(Point::new(edge.x / inv, edge.y / inv))
    } else {
        Point::new(0.0, 1.0)
    };

    let gap = dist - r;

    finish(gap, normal, foot, circle, line)
}

/// Assembles the lever arms and the sliding velocity, given the three
/// quantities every primitive computes for itself.
///
/// Factored out because the rule is the same for every pair of surfaces, and
/// writing it twice is how the two primitives would drift apart. Both arms may
/// use one shared contact point: the two bodies' own surface points differ from
/// it only *along the normal*, by `gap` and by a radius respectively, and a
/// vector along the normal contributes nothing to a cross product with the
/// normal.
#[inline]
fn finish(gap: f64, normal: Point, point: Point, a: &Frame, b: &Frame) -> ContactQuery {
    let tangent = perp(normal);
    let v_rel = sub(a.velocity_at(point), b.velocity_at(point));
    ContactQuery {
        gap,
        normal,
        point,
        d_gap_d_theta_a: cross(sub(point, a.pivot_world), normal),
        d_gap_d_theta_b: -cross(sub(point, b.pivot_world), normal),
        v_tangential: dot(v_rel, tangent),
    }
}

/// Contact between two convex circles, one on each body, touching externally.
///
/// Two of the seventeen contacts are of this kind, the capstan against the
/// wippen heel being the one that matters: it is the first link in the chain and
/// its lever arm migrates through the whole stroke, which is one of the places
/// the leverage a technician feels actually changes.
///
/// The normal points from B towards A, and `gap` is positive when separated.
/// Only the external case is modelled — no contact in a grand action has one
/// circle inside the other, and silently returning an internal-tangency answer
/// would hide a geometry error rather than surface it.
pub fn circle_vs_circle(
    a: &Frame,
    c_a_local: Point,
    r_a: f64,
    b: &Frame,
    c_b_local: Point,
    r_b: f64,
) -> ContactQuery {
    let ca = a.world(c_a_local);
    let cb = b.world(c_b_local);

    let delta = sub(ca, cb);
    let dist = dot(delta, delta).sqrt();

    let normal = if dist > DEGENERATE_DISTANCE {
        Point::new(delta.x / dist, delta.y / dist)
    } else {
        // Concentric: the normal is undefined. Return a total, finite answer
        // rather than a NaN that would surface much later, far from its cause.
        Point::new(0.0, 1.0)
    };

    // The point on B's surface. Any point on the common normal would give the
    // same lever arms; this one is the one a drawing would mark.
    let point = Point::new(cb.x + r_b * normal.x, cb.y + r_b * normal.y);

    finish(dist - r_a - r_b, normal, point, a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit (cos, sin) pair from the 3-4-5 triangle: exact in binary floating
    /// point, so tests built on it isolate formula errors from rounding.
    fn rot_3_4_5() -> Rot {
        Rot::from_cos_sin(0.8, 0.6)
    }

    /// Level 0, the reference case computed by hand.
    ///
    /// A circle of radius 2 mm centred 10 mm above the origin, against the
    /// horizontal line y = 0 carried by a body pivoting at the origin. The gap
    /// must be 8 mm, the normal straight up, and the contact point directly
    /// below the centre.
    #[test]
    fn circle_above_horizontal_line_has_hand_computable_geometry() {
        let circle = Frame::at(Point::new(0.0, 0.0));
        let line = Frame::at(Point::new(0.0, 0.0));
        let q = circle_vs_line(
            &circle,
            Point::new(0.0, 0.010),
            0.002,
            &line,
            Point::new(-0.050, 0.0),
            Point::new(0.050, 0.0),
        );

        assert!((q.gap - 0.008).abs() < 1e-15, "gap was {}", q.gap);
        assert!(
            q.normal.y > 0.0,
            "normal must point from the line to the circle"
        );
        assert!(q.normal.x.abs() < 1e-15);
        assert!(q.point.x.abs() < 1e-15 && q.point.y.abs() < 1e-15);
    }

    /// Level 0: the lever arm is the moment arm of the normal about the pivot.
    ///
    /// Circle centre 10 mm above and 30 mm to the right of body A's pivot,
    /// normal vertical. The moment arm about A is then the 30 mm horizontal
    /// offset, with the sign of `(P - Oa) x n`.
    #[test]
    fn lever_arm_is_the_moment_arm_of_the_normal() {
        let circle = Frame::at(Point::new(0.0, 0.0));
        let line = Frame::at(Point::new(0.0, 0.0));
        let q = circle_vs_line(
            &circle,
            Point::new(0.030, 0.010),
            0.002,
            &line,
            Point::new(-0.050, 0.0),
            Point::new(0.050, 0.0),
        );

        // P = (0.030, 0), n = (0, 1)  =>  P x n = 0.030 * 1 - 0 * 0 = 0.030
        assert!(
            (q.d_gap_d_theta_a - 0.030).abs() < 1e-15,
            "lever arm on A was {}",
            q.d_gap_d_theta_a
        );
        assert!(
            (q.d_gap_d_theta_b + 0.030).abs() < 1e-15,
            "lever arm on B was {}",
            q.d_gap_d_theta_b
        );
    }

    /// Level 0, the strongest of them: the analytic derivative must agree with a
    /// central finite difference of the gap.
    ///
    /// This is run over a grid of angles and offsets, through the face case and
    /// the endpoint case alike. It is the test that catches a wrong sign, a
    /// transposed cross product, or a lever arm taken about the wrong point —
    /// and it needs no solver, no time step and no measured parameter.
    #[test]
    fn analytic_lever_arms_match_central_finite_differences() {
        let h = 1e-7;
        let mut checked = 0;

        for ia in -4i32..=4 {
            for ib in -4i32..=4 {
                for ix in -3i32..=3 {
                    let theta_a = f64::from(ia) * 0.05;
                    let theta_b = f64::from(ib) * 0.05;
                    let c_local = Point::new(f64::from(ix) * 0.020, 0.010);

                    let oa = Point::new(0.0, -0.004);
                    let ob = Point::new(0.012, 0.003);
                    let a_local = Point::new(-0.030, 0.0);
                    let b_local = Point::new(0.030, 0.0);
                    let r = 0.002;

                    let frame_a = |t: f64| Frame {
                        pivot_local: Point::new(0.0, 0.0),
                        pivot_world: oa,
                        rot: Rot::from_angle(t),
                        omega: 0.0,
                    };
                    let frame_b = |t: f64| Frame {
                        pivot_local: Point::new(0.0, 0.0),
                        pivot_world: ob,
                        rot: Rot::from_angle(t),
                        omega: 0.0,
                    };

                    let gap = |ta: f64, tb: f64| {
                        circle_vs_line(&frame_a(ta), c_local, r, &frame_b(tb), a_local, b_local).gap
                    };

                    let q = circle_vs_line(
                        &frame_a(theta_a),
                        c_local,
                        r,
                        &frame_b(theta_b),
                        a_local,
                        b_local,
                    );

                    let fd_a = (gap(theta_a + h, theta_b) - gap(theta_a - h, theta_b)) / (2.0 * h);
                    let fd_b = (gap(theta_a, theta_b + h) - gap(theta_a, theta_b - h)) / (2.0 * h);

                    // Tolerance is set by the finite difference, not by the
                    // analytic formula: h^2 truncation plus cancellation on a
                    // gap of order 1e-2 lands around 1e-9 m/rad.
                    assert!(
                        (q.d_gap_d_theta_a - fd_a).abs() < 1e-8,
                        "A: analytic {} vs fd {} at ({theta_a}, {theta_b})",
                        q.d_gap_d_theta_a,
                        fd_a
                    );
                    assert!(
                        (q.d_gap_d_theta_b - fd_b).abs() < 1e-8,
                        "B: analytic {} vs fd {} at ({theta_a}, {theta_b})",
                        q.d_gap_d_theta_b,
                        fd_b
                    );
                    checked += 1;
                }
            }
        }

        assert_eq!(checked, 9 * 9 * 7, "the grid must actually have been swept");
    }

    /// Level 0: the lever arm does not depend on the circle's radius.
    ///
    /// Physically: leverage is a property of the normal's line of action. A
    /// larger knuckle changes where the contact sits and therefore the gap, but
    /// at a *given* contact point it does not change the moment arm. A model
    /// that failed this would make knuckle diameter a touchweight parameter in
    /// its own right, which it is not.
    #[test]
    fn lever_arm_is_independent_of_radius() {
        let circle = Frame {
            pivot_local: Point::new(0.0, 0.0),
            pivot_world: Point::new(0.0, -0.004),
            rot: rot_3_4_5(),
            omega: 0.0,
        };
        let line = Frame::at(Point::new(0.012, 0.003));

        let query = |r: f64| {
            circle_vs_line(
                &circle,
                Point::new(0.020, 0.010),
                r,
                &line,
                Point::new(-0.030, 0.0),
                Point::new(0.030, 0.0),
            )
        };

        let small = query(0.001);
        let large = query(0.008);

        assert!(
            (small.d_gap_d_theta_a - large.d_gap_d_theta_a).abs() < 1e-15,
            "radius changed the lever arm on A"
        );
        assert!(
            (small.d_gap_d_theta_b - large.d_gap_d_theta_b).abs() < 1e-15,
            "radius changed the lever arm on B"
        );
        // ... while the gap differs by exactly the radius difference.
        assert!((small.gap - large.gap - 0.007).abs() < 1e-15);
    }

    /// Level 0: the gap stays continuous as the contact leaves the face of the
    /// segment and passes onto its endpoint.
    ///
    /// The jack nose really does slide off the edge of the let-off button, so a
    /// jump here would appear as an impulsive force at let-off — a plausible
    /// looking artefact that would be read as physics.
    #[test]
    fn gap_is_continuous_across_the_face_to_endpoint_transition() {
        let line = Frame::at(Point::new(0.0, 0.0));
        let a_local = Point::new(-0.010, 0.0);
        let b_local = Point::new(0.010, 0.0);

        let gap_at = |x: f64| {
            circle_vs_line(
                &Frame::at(Point::new(0.0, 0.0)),
                Point::new(x, 0.004),
                0.002,
                &line,
                a_local,
                b_local,
            )
            .gap
        };

        // A gap built from a Euclidean distance is 1-Lipschitz in the position
        // of the circle, so over a step of `step` it cannot move by more than
        // `step`. That is the constant to assert: just past the endpoint the
        // slope is genuinely of order 1e-2, so a 1 um step legitimately moves
        // the gap by 1e-8, and a bound tighter than the true slope tests the
        // tolerance rather than the geometry.
        let step = 1e-6;
        let mut previous = gap_at(0.0090);
        let mut x = 0.0090;
        while x < 0.0110 {
            x += step;
            let current = gap_at(x);
            assert!(
                (current - previous).abs() <= step * (1.0 + 1e-9),
                "gap moved by {} over a step of {step} at x = {x}, which breaks the \
                 Lipschitz bound and therefore means a real discontinuity",
                current - previous
            );
            previous = current;
        }

        // What must be continuous at the transition itself is the *lever arm*:
        // a jump there would be an impulsive torque, and it would appear
        // exactly at let-off, where it would be read as physics.
        let arm_at = |x: f64| {
            circle_vs_line(
                &Frame::at(Point::new(0.0, 0.0)),
                Point::new(x, 0.004),
                0.002,
                &line,
                a_local,
                b_local,
            )
            .d_gap_d_theta_a
        };
        let before = arm_at(0.010 - 1e-9);
        let after = arm_at(0.010 + 1e-9);
        assert!(
            (before - after).abs() < 1e-9,
            "lever arm jumped from {before} to {after} across the face-to-endpoint \
             transition"
        );

        // And past the end the contact is governed by the endpoint, so the gap
        // grows with distance from it rather than staying flat.
        assert!(gap_at(0.0130) > gap_at(0.0110));
    }

    /// The normal is a unit vector in every configuration the grid visits.
    #[test]
    fn normal_is_always_unit_length() {
        for i in -6i32..=6 {
            for j in -6i32..=6 {
                let q = circle_vs_line(
                    &Frame {
                        pivot_local: Point::new(0.0, 0.0),
                        pivot_world: Point::new(0.0, -0.004),
                        rot: Rot::from_angle(f64::from(i) * 0.1),
                        omega: 0.0,
                    },
                    Point::new(f64::from(j) * 0.005, 0.010),
                    0.002,
                    &Frame::at(Point::new(0.012, 0.003)),
                    Point::new(-0.030, 0.0),
                    Point::new(0.030, 0.0),
                );
                let n2 = q.normal.x * q.normal.x + q.normal.y * q.normal.y;
                assert!((n2 - 1.0).abs() < 1e-15, "normal squared length was {n2}");
            }
        }
    }

    /// Level 0 for the second primitive: two circles whose geometry is known by
    /// hand.
    ///
    /// Centres 50 mm apart horizontally, radii 10 and 15 mm, so the gap is
    /// 25 mm and the normal is horizontal.
    #[test]
    fn two_circles_have_hand_computable_geometry() {
        let q = circle_vs_circle(
            &Frame::at(Point::new(0.0, 0.0)),
            Point::new(0.050, 0.0),
            0.010,
            &Frame::at(Point::new(0.0, 0.0)),
            Point::new(0.0, 0.0),
            0.015,
        );
        assert!((q.gap - 0.025).abs() < 1e-15, "gap was {}", q.gap);
        assert!((q.normal.x - 1.0).abs() < 1e-15 && q.normal.y.abs() < 1e-15);
        // The marked point sits on B's surface, 15 mm out along the normal.
        assert!((q.point.x - 0.015).abs() < 1e-15 && q.point.y.abs() < 1e-15);
    }

    /// Level 0, the strongest test again, now on the circle-circle primitive:
    /// the analytic lever arms must match a central finite difference.
    ///
    /// Run over a grid of both angles and of the capstan's position, because it
    /// is the migration of this particular contact point through the stroke
    /// that makes the first lever ratio of the chain vary.
    #[test]
    fn circle_circle_lever_arms_match_central_finite_differences() {
        let h = 1e-7;
        let mut checked = 0;

        for ia in -4i32..=4 {
            for ib in -4i32..=4 {
                for ix in -2i32..=2 {
                    let (ta, tb) = (f64::from(ia) * 0.05, f64::from(ib) * 0.05);
                    let oa = Point::new(0.0, -0.004);
                    let ob = Point::new(0.030, 0.006);
                    let c_a = Point::new(f64::from(ix) * 0.004 + 0.014, 0.004);
                    let c_b = Point::new(-0.010, -0.002);
                    let (ra, rb) = (0.003, 0.0045);

                    let frame = |o: Point, th: f64| Frame {
                        pivot_local: Point::new(0.0, 0.0),
                        pivot_world: o,
                        rot: Rot::from_angle(th),
                        omega: 0.0,
                    };
                    let gap = |x: f64, y: f64| {
                        circle_vs_circle(&frame(oa, x), c_a, ra, &frame(ob, y), c_b, rb).gap
                    };

                    let q = circle_vs_circle(&frame(oa, ta), c_a, ra, &frame(ob, tb), c_b, rb);
                    let fd_a = (gap(ta + h, tb) - gap(ta - h, tb)) / (2.0 * h);
                    let fd_b = (gap(ta, tb + h) - gap(ta, tb - h)) / (2.0 * h);

                    assert!(
                        (q.d_gap_d_theta_a - fd_a).abs() < 1e-8,
                        "A: analytic {} vs fd {fd_a} at ({ta}, {tb})",
                        q.d_gap_d_theta_a
                    );
                    assert!(
                        (q.d_gap_d_theta_b - fd_b).abs() < 1e-8,
                        "B: analytic {} vs fd {fd_b} at ({ta}, {tb})",
                        q.d_gap_d_theta_b
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 9 * 9 * 5);
    }

    /// Level 0: neither radius changes the lever arm, on either primitive.
    ///
    /// The same invariant as for circle-against-line, and it has the same trade
    /// meaning: a larger capstan or a thicker heel cloth moves the contact and
    /// therefore the gap, but at a given contact point it does not change the
    /// moment arm. Leverage is a property of the normal's line of action.
    #[test]
    fn circle_circle_lever_arm_is_independent_of_both_radii() {
        let a = Frame {
            pivot_local: Point::new(0.0, 0.0),
            pivot_world: Point::new(0.0, -0.004),
            rot: rot_3_4_5(),
            omega: 0.0,
        };
        let b = Frame::at(Point::new(0.030, 0.006));
        let q = |ra: f64, rb: f64| {
            circle_vs_circle(
                &a,
                Point::new(0.014, 0.004),
                ra,
                &b,
                Point::new(-0.010, -0.002),
                rb,
            )
        };

        let small = q(0.002, 0.003);
        let large = q(0.006, 0.009);
        assert!((small.d_gap_d_theta_a - large.d_gap_d_theta_a).abs() < 1e-15);
        assert!((small.d_gap_d_theta_b - large.d_gap_d_theta_b).abs() < 1e-15);
        // The gap falls by exactly the sum of the radius increases.
        assert!((small.gap - large.gap - 0.010).abs() < 1e-15);
    }

    /// The two primitives agree where they describe the same contact, and they
    /// agree *at the rate curvature says they should*.
    ///
    /// A circle against a very large circle is a circle against a line, in the
    /// limit. The residual disagreement is the sagitta of the big circle over
    /// the contact offset, `x^2 / 2R`, so it must fall like `1/R`. Asserting
    /// that rate is a much stronger test than asserting a tolerance: a constant
    /// offset, a sign that drifted between the two primitives, or a radius used
    /// twice would all survive a loose bound and all fail a convergence rate.
    ///
    /// (A fixed tolerance is the wrong assertion here, and easy to get wrong by
    /// three orders: the sagitta is 5e-6 m in this configuration, not 2e-8.
    /// Asserting the rate instead cannot be miscalibrated.)
    #[test]
    fn a_very_large_circle_converges_to_a_line_at_the_rate_curvature_dictates() {
        let a = Frame {
            pivot_local: Point::new(0.0, 0.0),
            pivot_world: Point::new(0.0, -0.004),
            rot: rot_3_4_5(),
            omega: 0.0,
        };
        let b = Frame::at(Point::new(0.0, 0.0));
        let c_a = Point::new(0.020, 0.010);
        let r_a = 0.002;

        let as_line = circle_vs_line(
            &a,
            c_a,
            r_a,
            &b,
            Point::new(-0.500, 0.0),
            Point::new(0.500, 0.0),
        );

        // The contact sits at this horizontal offset from the big circle's top,
        // which is what the sagitta is measured over.
        let x = as_line.point.x;

        let mut previous: Option<f64> = None;
        for &big_r in &[10.0_f64, 100.0, 1000.0] {
            let as_circle = circle_vs_circle(&a, c_a, r_a, &b, Point::new(0.0, -big_r), big_r);

            // Same normal direction, to the same order.
            assert!(
                as_circle.normal.y > 0.0 && as_line.normal.y > 0.0,
                "the two primitives must agree on which way the normal points"
            );

            let error = (as_circle.gap - as_line.gap).abs();
            let sagitta = x * x / (2.0 * big_r);
            assert!(
                (error - sagitta).abs() < 0.02 * sagitta,
                "R = {big_r}: gap error {error} is not the sagitta {sagitta}"
            );

            if let Some(prev) = previous {
                let ratio = prev / error;
                assert!(
                    (ratio - 10.0).abs() < 0.5,
                    "R = {big_r}: error fell by {ratio}x for a 10x radius, expected 10x"
                );
            }
            previous = Some(error);
        }

        // And the lever arm on A, which is the quantity that actually matters,
        // converges too — but at its own rate. The arm tracks the *tilt* of the
        // normal, `x / R`, not the sagitta, so at R = 1e5 the normal is off by
        // 1e-7 rad and an arm of order 25 mm is off by about 2.5e-9 m/rad.
        // That bound is derived, not tuned.
        let huge = circle_vs_circle(&a, c_a, r_a, &b, Point::new(0.0, -1.0e5), 1.0e5);
        let tilt_bound = 0.025 * (x / 1.0e5).abs() * 2.0;
        assert!(
            (huge.d_gap_d_theta_a - as_line.d_gap_d_theta_a).abs() < tilt_bound,
            "arm on A: circle {} vs line {}",
            huge.d_gap_d_theta_a,
            as_line.d_gap_d_theta_a
        );
    }

    /// Sliding velocity has the sign of the relative tangential motion, and
    /// vanishes when both bodies are still.
    #[test]
    fn tangential_velocity_is_zero_at_rest_and_signed_when_sliding() {
        let line = Frame::at(Point::new(0.0, 0.0));
        let still = circle_vs_line(
            &Frame::at(Point::new(0.0, 0.0)),
            Point::new(0.030, 0.010),
            0.002,
            &line,
            Point::new(-0.050, 0.0),
            Point::new(0.050, 0.0),
        );
        assert!(still.v_tangential.abs() < 1e-15);

        let turning = circle_vs_line(
            &Frame {
                pivot_local: Point::new(0.0, 0.0),
                pivot_world: Point::new(0.0, 0.0),
                rot: Rot::IDENTITY,
                omega: 1.0,
            },
            Point::new(0.030, 0.010),
            0.002,
            &line,
            Point::new(-0.050, 0.0),
            Point::new(0.050, 0.0),
        );
        // Contact point at (0.030, 0), normal (0, 1), tangent (-1, 0).
        // Velocity of A's material point there is omega * perp(P) = (0, 0.030),
        // so the tangential component is zero — the surfaces are rolling, not
        // sliding, at the instant the normal passes through the pivot offset.
        // Move the pivot sideways and sliding appears.
        assert!(turning.v_tangential.abs() < 1e-15);

        let sliding = circle_vs_line(
            &Frame {
                pivot_local: Point::new(0.0, 0.0),
                pivot_world: Point::new(0.0, 0.020),
                rot: Rot::IDENTITY,
                omega: 1.0,
            },
            Point::new(0.030, 0.010),
            0.002,
            &line,
            Point::new(-0.050, 0.0),
            Point::new(0.050, 0.0),
        );
        assert!(
            sliding.v_tangential.abs() > 1e-6,
            "a pivot offset normal to the contact must produce sliding, got {}",
            sliding.v_tangential
        );
    }
}
