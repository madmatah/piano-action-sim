//! Prescribed coordinates, and the generalized force that holds each.

use super::{CoordId, Mechanism, PointLoad, Pose, Problem};

/// A coordinate held at a value for one procedure.
///
/// It belongs to the procedure, not to the mechanism: the jig holds the key's
/// angle level, and the same key on its frame leaves it free.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prescribed {
    /// Which coordinate.
    pub coord: CoordId,
    /// The value it is held at: radians for an angle, meters for a length.
    pub value: f64,
}

/// A mechanism held still with every coordinate prescribed, and what holding
/// it takes.
#[derive(Debug, Clone, PartialEq)]
pub struct Held {
    /// Where every body is.
    pub pose: Pose,
    /// The generalized force each prescription applies to the mechanism, one
    /// per coordinate: newton-meters on an angle, newtons on a length.
    ///
    /// It balances the loads' generalized force on that coordinate, so it is
    /// minus it: positive turns an angle counter-clockwise, which lifts the
    /// front of a key. Whatever holds the coordinate in the real fixture (the
    /// scale under a key on the jig, a finger driving a key down) supplies
    /// exactly this.
    pub reactions: Vec<f64>,
}

impl Held {
    /// The generalized force holding one coordinate.
    ///
    /// # Panics
    ///
    /// When the mechanism has no such coordinate.
    pub fn reaction(&self, coord: CoordId) -> f64 {
        self.reactions[coord.0]
    }
}

impl Mechanism {
    /// Holds every coordinate at its prescribed value under `loads`, and
    /// returns the pose and the reaction of each prescription.
    ///
    /// With every coordinate prescribed there is nothing to solve: the
    /// stationarity of the potential, coordinate by coordinate, says what
    /// each prescription must apply, `R = −Q`. A free coordinate needs the
    /// static solve, which does not exist yet, so leaving one free is
    /// refused, as are a coordinate prescribed twice, a value or a load that
    /// is not a finite number, and a load on a body that is not there.
    pub fn hold(&self, prescribed: &[Prescribed], loads: &[PointLoad]) -> Result<Held, Problem> {
        let n = self.coords.len();
        let mut q = vec![0.0; n];
        let mut set = vec![false; n];
        for p in prescribed {
            let c = p.coord.0;
            if c >= n {
                return Err(Problem::NoSuchCoordinate { coord: p.coord });
            }
            if set[c] {
                return Err(Problem::PrescribedTwice { coord: p.coord });
            }
            if !p.value.is_finite() {
                return Err(Problem::PrescribedNotFinite { coord: p.coord });
            }
            q[c] = p.value;
            set[c] = true;
        }
        if let Some(c) = set.iter().position(|s| !s) {
            return Err(Problem::NotPrescribed { coord: CoordId(c) });
        }
        for (i, load) in loads.iter().enumerate() {
            if load.at.body.0 >= self.bodies.len() {
                return Err(Problem::NoSuchBody { body: load.at.body });
            }
            let numbers = [load.at.at.x, load.at.at.y, load.force.x, load.force.y];
            if !numbers.iter().all(|x| x.is_finite()) {
                return Err(Problem::LoadNotFinite { load: i });
            }
        }
        let pose = self.pose(&q);
        let mut reactions = vec![0.0; n];
        self.generalized_forces(&pose, loads, &mut reactions);
        for r in &mut reactions {
            *r = -*r;
        }
        Ok(Held { pose, reactions })
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::{chain, random_q, XorShift};
    use super::super::{Body, BodyId, Coord, Joint, MassProps, MaterialPoint};
    use super::*;
    use crate::geometry::Point;
    use crate::G;

    fn all(q: &[f64]) -> Vec<Prescribed> {
        q.iter()
            .enumerate()
            .map(|(c, &value)| Prescribed {
                coord: CoordId(c),
                value,
            })
            .collect()
    }

    /// A lever held level by its pin: the reaction is the weight times its
    /// arm, as a technician would work it out, and the sign lifts the heavy
    /// side.
    #[test]
    fn a_level_lever_is_held_by_its_weight_times_its_arm() {
        let lever = Mechanism::new(
            vec![
                Body::GROUND,
                Body {
                    parent: BodyId::GROUND,
                    joint: Joint::Revolute {
                        pivot: Point::new(0.0, 0.0),
                        coord: CoordId(0),
                    },
                    mass: Some(MassProps {
                        mass: 0.120,
                        com: Point::new(0.020, 0.0158),
                    }),
                },
            ],
            vec![Coord::ANGLE],
        )
        .expect("a lever");
        let held = lever
            .hold(
                &[Prescribed {
                    coord: CoordId(0),
                    value: 0.0,
                }],
                &[],
            )
            .expect("held level");
        // Level, so the height of the center of mass has no arm.
        let expected = 0.120 * G * 0.020;
        assert!(
            (held.reaction(CoordId(0)) - expected).abs() <= 4.0 * f64::EPSILON * expected,
            "{} against {expected}",
            held.reaction(CoordId(0))
        );

        // A weight behind the pin that outweighs it turns the reaction round.
        let behind = PointLoad::weight(
            MaterialPoint {
                body: BodyId(1),
                at: Point::new(-0.100, 0.0),
            },
            0.030,
        );
        let held = lever
            .hold(
                &[Prescribed {
                    coord: CoordId(0),
                    value: 0.0,
                }],
                &[behind],
            )
            .expect("held level");
        assert!(held.reaction(CoordId(0)) < 0.0);
    }

    /// The reactions are minus the generalized forces, whatever the pose.
    #[test]
    fn every_reaction_balances_the_loads_on_its_coordinate() {
        let m = chain();
        let mut rng = XorShift::new(0x0000_401d);
        let q = random_q(&m, &mut rng);
        let held = m.hold(&all(&q), &[]).expect("every coordinate prescribed");
        let mut forces = vec![0.0; q.len()];
        m.generalized_forces(&held.pose, &[], &mut forces);
        for (r, f) in held.reactions.iter().zip(&forces) {
            assert_eq!(r.to_bits(), (-f).to_bits());
        }
        assert_eq!(held.pose, m.pose(&q));
    }

    /// What cannot be held is named, not computed.
    #[test]
    fn a_prescription_that_cannot_be_held_is_refused() {
        let m = chain();
        let q = [0.1, -0.2, 0.003, 0.4, -0.5];
        let mut free = all(&q);
        free.remove(2);
        assert_eq!(
            m.hold(&free, &[]),
            Err(Problem::NotPrescribed { coord: CoordId(2) })
        );
        let mut twice = all(&q);
        twice.push(twice[1]);
        assert_eq!(
            m.hold(&twice, &[]),
            Err(Problem::PrescribedTwice { coord: CoordId(1) })
        );
        let mut nan = all(&q);
        nan[3].value = f64::NAN;
        assert_eq!(
            m.hold(&nan, &[]),
            Err(Problem::PrescribedNotFinite { coord: CoordId(3) })
        );
        let mut extra = all(&q);
        extra.push(Prescribed {
            coord: CoordId(5),
            value: 0.0,
        });
        assert_eq!(
            m.hold(&extra, &[]),
            Err(Problem::NoSuchCoordinate { coord: CoordId(5) })
        );
        let nowhere = PointLoad::weight(
            MaterialPoint {
                body: BodyId(6),
                at: Point::new(0.0, 0.0),
            },
            0.010,
        );
        assert_eq!(
            m.hold(&all(&q), &[nowhere]),
            Err(Problem::NoSuchBody { body: BodyId(6) })
        );
        let not_a_number = PointLoad {
            at: MaterialPoint {
                body: BodyId(1),
                at: Point::new(0.0, 0.0),
            },
            force: Point::new(0.0, f64::INFINITY),
        };
        assert_eq!(
            m.hold(&all(&q), &[not_a_number]),
            Err(Problem::LoadNotFinite { load: 0 })
        );
    }
}
