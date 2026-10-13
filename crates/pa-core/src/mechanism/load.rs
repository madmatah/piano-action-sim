//! Loads: gravity at every body's centre of mass, and forces at material
//! points, with their potential and their generalized forces.

use super::{BodyId, MaterialPoint, Mechanism, Pose};
use crate::geometry::Point;
use crate::G;

/// A force at a material point, its direction fixed in the world: a weight
/// resting on the key, the press of a finger, a scale pushing up.
///
/// Its point rides on its body and its direction stays where it was: tilt
/// the key and a weight on it still pulls straight down, on the arm the
/// tilted key presents. That arm comes out of the Jacobian, not out of a
/// formula of its own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLoad {
    /// Where it acts.
    pub at: MaterialPoint,
    /// The force, newtons, in the world.
    pub force: Point,
}

impl PointLoad {
    /// A weight of `mass` kilograms resting on `at`: `mass · G`, straight
    /// down.
    pub fn weight(at: MaterialPoint, mass: f64) -> Self {
        Self {
            at,
            force: Point::new(0.0, -mass * G),
        }
    }
}

impl Mechanism {
    /// The potential energy of gravity and of `loads` at `pose`, joules, from
    /// the world's origin.
    ///
    /// Gravity's is `m·G·y` of every centre of mass; a force fixed in the
    /// world has `−F·P` of its point. Only its differences mean anything: it
    /// is what the generalized forces are minus the gradient of.
    ///
    /// # Panics
    ///
    /// When a load names a body the mechanism does not have.
    pub fn potential(&self, pose: &Pose, loads: &[PointLoad]) -> f64 {
        let mut v = 0.0;
        for (i, body) in self.bodies.iter().enumerate() {
            if let Some(m) = body.mass {
                let c = pose.world(MaterialPoint {
                    body: BodyId(i),
                    at: m.com,
                });
                v += m.mass * G * c.y;
            }
        }
        for load in loads {
            let p = pose.world(load.at);
            v -= load.force.x * p.x + load.force.y * p.y;
        }
        v
    }

    /// The size of the terms [`Self::potential`] adds up at `pose`, joules:
    /// each weight's `m·G·|y|` and each force's `|F_x·P_x| + |F_y·P_y|`.
    /// What the potential's rounding scales with: a sum of terms each
    /// rounded at its own last bit is known to a few units in the last place
    /// of this, whatever the sum itself comes to.
    pub(crate) fn potential_magnitude(&self, pose: &Pose, loads: &[PointLoad]) -> f64 {
        let mut size = 0.0;
        for (i, body) in self.bodies.iter().enumerate() {
            if let Some(m) = body.mass {
                let c = pose.world(MaterialPoint {
                    body: BodyId(i),
                    at: m.com,
                });
                size += (m.mass * G * c.y).abs();
            }
        }
        for load in loads {
            let p = pose.world(load.at);
            size += (load.force.x * p.x).abs() + (load.force.y * p.y).abs();
        }
        size
    }

    /// The generalized forces of gravity and of `loads` at `pose`, written
    /// into `out`, one per coordinate: newton-metres on an angle, newtons on
    /// a length.
    ///
    /// Each force contributes its Jacobian column dotted with it, which on a
    /// pin is its moment about the pivot. Gravity comes first, body by body,
    /// then the loads in their order: a fixed order of operations, so the same
    /// bits every time.
    ///
    /// # Panics
    ///
    /// When `out` does not hold one value per coordinate, or a load names a
    /// body the mechanism does not have.
    pub fn generalized_forces(&self, pose: &Pose, loads: &[PointLoad], out: &mut [f64]) {
        assert_eq!(
            out.len(),
            self.coords.len(),
            "one generalized force per coordinate"
        );
        out.fill(0.0);
        for (i, body) in self.bodies.iter().enumerate() {
            if let Some(m) = body.mass {
                let at = MaterialPoint {
                    body: BodyId(i),
                    at: m.com,
                };
                self.add_force(pose, at, Point::new(0.0, -m.mass * G), out);
            }
        }
        for load in loads {
            self.add_force(pose, load.at, load.force, out);
        }
    }

    fn add_force(&self, pose: &Pose, at: MaterialPoint, force: Point, out: &mut [f64]) {
        self.for_each_column(pose, at, |coord, column| {
            out[coord.0] += column.x * force.x + column.y * force.y;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::{chain, random_q, reach, XorShift};
    use super::super::{CoordKind, MaterialPoint};
    use super::*;

    /// Loads on three bodies, in three directions, besides gravity.
    fn loads() -> Vec<PointLoad> {
        let on = |b: usize, x: f64, y: f64| MaterialPoint {
            body: BodyId(b),
            at: Point::new(x, y),
        };
        vec![
            PointLoad {
                at: on(2, 0.08, 0.01),
                force: Point::new(0.3, -1.1),
            },
            PointLoad::weight(on(4, -0.02, 0.03), 0.050),
            PointLoad {
                at: on(5, -0.05, -0.01),
                force: Point::new(-0.7, 0.0),
            },
        ]
    }

    /// C3, consistency: the generalized forces of gravity and of point loads
    /// against central differences of the potential, at seeded random poses
    /// of the chain.
    ///
    /// The bound is derived. Turning a pin turns every point below it rigidly
    /// about the pin while every force keeps its direction in the world, so
    /// the potential is a sinusoid in that angle plus a constant, and its
    /// exact central difference is `sin(h)/h` times the derivative: off by at
    /// most `h²/6 |Q|`, and `|Q|` is at most `S`, the sum over every weight
    /// and force of its magnitude times its point's reach. On a slide the
    /// potential is linear and the difference exact. Each evaluation of the
    /// potential is within `72 ε S`: `64 ε` of reach for each point, as in C1,
    /// and at most sixteen roundings within `u` for the products and the
    /// running sum. The generalized force's own rounding is under `144 ε S`.
    #[test]
    fn generalized_forces_are_minus_the_gradient_of_the_potential() {
        let m = chain();
        let loads = loads();
        let mut rng = XorShift::new(0x00c3_c3c3);
        let h = 1e-4;
        let mut q_forces = vec![0.0; m.coords().len()];
        for _ in 0..64 {
            let q = random_q(&m, &mut rng);
            let pose = m.pose(&q);
            m.generalized_forces(&pose, &loads, &mut q_forces);

            // S: every weight and force times its point's reach.
            let mut s = 0.0;
            for (i, body) in m.bodies().iter().enumerate() {
                if let Some(mp) = body.mass {
                    let at = MaterialPoint {
                        body: BodyId(i),
                        at: mp.com,
                    };
                    s += mp.mass * G * reach(&m, &q, at);
                }
            }
            for load in &loads {
                let f = (load.force.x * load.force.x + load.force.y * load.force.y).sqrt();
                s += f * reach(&m, &q, load.at);
            }

            for (c, coord) in m.coords().iter().enumerate() {
                let (mut up, mut down) = (q.clone(), q.clone());
                up[c] += h;
                down[c] -= h;
                let step = up[c] - down[c];
                let from_energy = -(m.potential(&m.pose(&up), &loads)
                    - m.potential(&m.pose(&down), &loads))
                    / step;
                let truncation = match coord.kind {
                    CoordKind::Angle => (0.5 * step) * (0.5 * step) / 6.0 * s,
                    CoordKind::Length => 0.0,
                };
                let bound =
                    truncation + 72.0 * f64::EPSILON * s / (0.5 * step) + 144.0 * f64::EPSILON * s;
                let error = (from_energy - q_forces[c]).abs();
                assert!(
                    error <= bound,
                    "coordinate {c} at {q:?}: generalized force {}, from the potential \
                     {from_energy}, off by {error:e} > {bound:e}",
                    q_forces[c]
                );
            }
        }
    }

    /// A weight is its mass times standard gravity, straight down.
    #[test]
    fn a_weight_pulls_straight_down() {
        let at = MaterialPoint {
            body: BodyId(1),
            at: Point::new(0.0, 0.0),
        };
        let w = PointLoad::weight(at, 0.010);
        assert_eq!(w.force.x.to_bits(), 0.0_f64.to_bits());
        assert_eq!(w.force.y.to_bits(), (-0.010 * G).to_bits());
    }
}
