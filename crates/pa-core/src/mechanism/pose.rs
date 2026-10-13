//! Poses, and how fast material points move with each coordinate.

use super::{BodyId, CoordId, Joint, Mechanism};
use crate::geometry::Point;

/// A point fixed in a body, written in that body's frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialPoint {
    /// The body it belongs to.
    pub body: BodyId,
    /// Where on the body, in the body's frame.
    pub at: Point,
}

/// Where one body's frame stands in the world: its origin, and the cosine and
/// sine of its rotation from the world's axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The frame's origin, in the world.
    pub origin: Point,
    /// Cosine of the frame's rotation.
    pub cos: f64,
    /// Sine of the frame's rotation, positive counter-clockwise.
    pub sin: f64,
}

impl Placement {
    /// The world's own frame, which is the ground's.
    pub const WORLD: Self = Self {
        origin: Point::new(0.0, 0.0),
        cos: 1.0,
        sin: 0.0,
    };

    /// A vector of this frame, in the world: turned, not moved.
    pub fn rotate(&self, v: Point) -> Point {
        Point::new(
            self.cos * v.x - self.sin * v.y,
            self.sin * v.x + self.cos * v.y,
        )
    }

    /// A point of this frame, in the world.
    pub fn world(&self, p: Point) -> Point {
        let r = self.rotate(p);
        Point::new(self.origin.x + r.x, self.origin.y + r.y)
    }
}

/// Every body placed, for one value of every coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct Pose {
    q: Vec<f64>,
    placements: Vec<Placement>,
}

impl Pose {
    /// The coordinates the pose was placed at, one per coordinate.
    pub fn q(&self) -> &[f64] {
        &self.q
    }

    /// Where a body's frame stands.
    ///
    /// # Panics
    ///
    /// When the mechanism the pose was placed for has no such body.
    pub fn placement(&self, body: BodyId) -> Placement {
        self.placements[body.0]
    }

    /// Where a material point is, in the world.
    ///
    /// # Panics
    ///
    /// When the mechanism the pose was placed for has no such body.
    pub fn world(&self, p: MaterialPoint) -> Point {
        self.placements[p.body.0].world(p.at)
    }
}

impl Mechanism {
    /// Places every body for the coordinates `q`.
    ///
    /// One pass, parent before child, which the build check makes possible.
    /// A revolute joint takes the sine and cosine of its angle from `libm`,
    /// once, and composes them with its parent's rotation; a prismatic joint
    /// moves its origin along its axis and keeps its parent's rotation. No
    /// other transcendental is called, so the same `q` gives the same bits on
    /// every machine.
    ///
    /// # Panics
    ///
    /// When `q` does not hold exactly one value per coordinate, which is the
    /// caller's error and not a state of the action.
    pub fn pose(&self, q: &[f64]) -> Pose {
        assert_eq!(
            q.len(),
            self.coords.len(),
            "a pose needs one value per coordinate"
        );
        let mut placements = Vec::with_capacity(self.bodies.len());
        placements.push(Placement::WORLD);
        for body in &self.bodies[1..] {
            let parent = placements[body.parent.0];
            let placement = match body.joint {
                // The build check puts the ground first and nowhere else.
                Joint::Ground => parent,
                Joint::Revolute { pivot, coord } => {
                    let angle = q[coord.0];
                    let (sin, cos) = (libm::sin(angle), libm::cos(angle));
                    Placement {
                        origin: parent.world(pivot),
                        cos: parent.cos * cos - parent.sin * sin,
                        sin: parent.sin * cos + parent.cos * sin,
                    }
                }
                Joint::Prismatic {
                    origin,
                    axis,
                    coord,
                } => {
                    let (travel, a) = (q[coord.0], axis.vector());
                    Placement {
                        origin: parent
                            .world(Point::new(origin.x + travel * a.x, origin.y + travel * a.y)),
                        ..parent
                    }
                }
            };
            placements.push(placement);
        }
        Pose {
            q: q.to_vec(),
            placements,
        }
    }

    /// How fast `point` moves in the world per unit of `coord`, at `pose`:
    /// one column of its Jacobian, metres per radian or metres per metre.
    ///
    /// The lever-arm identity, extended to any ancestor joint. If `coord` is
    /// the angle of a revolute joint between the ground and `point`'s body
    /// (the body's own included), the point turns about that joint's pivot
    /// `O`, so the column is `ẑ × (P − O)`: the lever arm of every force
    /// applied at `P`. If it is the travel of a prismatic joint there, the
    /// column is the joint's axis in the world. Any other coordinate does not
    /// move the point, and its column is zero.
    ///
    /// # Panics
    ///
    /// When the mechanism has no such body.
    pub fn jacobian_column(&self, pose: &Pose, point: MaterialPoint, coord: CoordId) -> Point {
        let mut column = Point::new(0.0, 0.0);
        self.for_each_column(pose, point, |c, v| {
            if c == coord {
                column = v;
            }
        });
        column
    }

    /// Calls `each(coord, column)` for every coordinate that moves `point`,
    /// from its own body's joint to the ground: the one place the lever-arm
    /// identity is written, which the Jacobian and the generalized forces
    /// both read.
    pub(super) fn for_each_column(
        &self,
        pose: &Pose,
        point: MaterialPoint,
        mut each: impl FnMut(CoordId, Point),
    ) {
        let p = pose.world(point);
        let mut body = point.body;
        while body != BodyId::GROUND {
            let b = &self.bodies[body.0];
            let placed = pose.placements[body.0];
            match b.joint {
                Joint::Ground => {}
                // A revolute body's origin is on its pin, so the pivot in the
                // world is where the body's frame stands.
                Joint::Revolute { coord, .. } => {
                    let o = placed.origin;
                    each(coord, Point::new(-(p.y - o.y), p.x - o.x));
                }
                // A prismatic body keeps its parent's rotation, so its own
                // turns the axis into the world.
                Joint::Prismatic { axis, coord, .. } => {
                    each(coord, placed.rotate(axis.vector()));
                }
            }
            body = b.parent;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::{chain, probes, random_q, reach, XorShift};
    use super::super::{CoordId, CoordKind};
    use super::*;

    /// At zero every frame stands at its joint, unrotated, so a point is its
    /// chain of offsets added up; and turning a pin a quarter turn carries
    /// everything below it round its pivot.
    #[test]
    fn a_pose_places_each_frame_at_its_joint() {
        let m = chain();
        let zero = m.pose(&[0.0; 5]);
        // d hangs from c, c slides from b, b turns on a, a turns on the
        // ground: the offsets of `fixtures::chain`, added up.
        let d = zero.placement(BodyId(4)).origin;
        let expected = Point::new(0.05 + 0.15 + 0.03 + 0.01, 0.02 - 0.03 + 0.01 + 0.02);
        assert!(
            (d.x - expected.x).abs() < 1e-15 && (d.y - expected.y).abs() < 1e-15,
            "{d:?} against {expected:?}"
        );

        // A quarter turn of a: b's pivot, 150 mm along a and 30 mm below it,
        // goes round a's pivot to stand 30 mm out and 150 mm up.
        let mut q = [0.0; 5];
        q[0] = core::f64::consts::FRAC_PI_2;
        let turned = m.pose(&q).placement(BodyId(2)).origin;
        let expected = Point::new(0.05 + 0.03, 0.02 + 0.15);
        // The quarter turn is π/2 to the nearest double, whose cosine is
        // 6e-17, not zero: 1e-15 covers it on arms under a metre.
        assert!(
            (turned.x - expected.x).abs() < 1e-15 && (turned.y - expected.y).abs() < 1e-15,
            "{turned:?} against {expected:?}"
        );
    }

    /// C1, consistency: every Jacobian column against central differences of
    /// the poses, at seeded random poses of a chain with a pin on a pin, a
    /// slide under a pin, a pin under a slide and a branch beside them.
    ///
    /// The bound is derived. Turning a pin by `±h` turns every point below it
    /// rigidly about the pin, so the exact central difference of a revolute
    /// column is `sin(h)/h` times the column, off by at most `h²/6` times the
    /// lever arm, which the chain's reach bounds; a slide moves points
    /// linearly, so its central difference has no truncation at all. Each
    /// evaluation of a point is within `64 ε L` of the exact one, `L` the
    /// reach: each of at most four rotations compounds its parent's error by
    /// at most √2 and adds `libm`'s ulp and three roundings, each placed point
    /// three more, under `40 ε L` in all. Two such evaluations over a step of
    /// `2h` contribute `64 ε L / h`, and the column's own rounding `2·64 ε L`.
    #[test]
    fn jacobian_columns_are_the_derivatives_of_the_poses() {
        let m = chain();
        let mut rng = XorShift::new(0x00c1_c1c1);
        let h = 1e-4;
        let rounding = |reach: f64| 64.0 * f64::EPSILON * reach;
        for _ in 0..64 {
            let q = random_q(&m, &mut rng);
            let pose = m.pose(&q);
            for point in probes(&m) {
                let reach = reach(&m, &q, point);
                for (c, coord) in m.coords().iter().enumerate() {
                    let column = m.jacobian_column(&pose, point, CoordId(c));
                    let (mut up, mut down) = (q.clone(), q.clone());
                    up[c] += h;
                    down[c] -= h;
                    let step = up[c] - down[c];
                    let (a, b) = (m.pose(&up).world(point), m.pose(&down).world(point));
                    let difference = Point::new((a.x - b.x) / step, (a.y - b.y) / step);
                    let truncation = match coord.kind {
                        CoordKind::Angle => (0.5 * step) * (0.5 * step) / 6.0 * reach,
                        CoordKind::Length => 0.0,
                    };
                    let bound = truncation + rounding(reach) / (0.5 * step) + 2.0 * rounding(reach);
                    let (ex, ey) = (difference.x - column.x, difference.y - column.y);
                    let error = (ex * ex + ey * ey).sqrt();
                    assert!(
                        error <= bound,
                        "coordinate {c}, point {point:?} at {q:?}: column {column:?}, \
                         central difference {difference:?}, off by {error:e} > {bound:e}"
                    );
                }
            }
        }
    }

    /// A coordinate that does not move a point has a column of exactly zero,
    /// and moving it leaves the point exactly where it was: the sibling branch
    /// and the joints below a body do not reach it.
    #[test]
    fn a_coordinate_that_does_not_move_a_point_has_no_column() {
        let m = chain();
        let mut rng = XorShift::new(0x0000_51b1);
        let q = random_q(&m, &mut rng);
        let pose = m.pose(&q);
        // On e (body 5), only e's own pin moves the point; on b (body 2), only
        // a's and b's.
        let cases = [
            (BodyId(5), [false, false, false, false, true]),
            (BodyId(2), [true, true, false, false, false]),
        ];
        for (body, moves) in cases {
            let point = MaterialPoint {
                body,
                at: Point::new(0.03, -0.01),
            };
            for (c, moved) in moves.into_iter().enumerate() {
                let column = m.jacobian_column(&pose, point, CoordId(c));
                let zero = column.x.to_bits() == 0 && column.y.to_bits() == 0;
                assert_eq!(!zero, moved, "body {body:?}, coordinate {c}: {column:?}");
                if !moved {
                    let mut nudged = q.clone();
                    nudged[c] += 0.3;
                    let (a, b) = (pose.world(point), m.pose(&nudged).world(point));
                    assert_eq!(
                        (a.x.to_bits(), a.y.to_bits()),
                        (b.x.to_bits(), b.y.to_bits())
                    );
                }
            }
        }
    }
}
