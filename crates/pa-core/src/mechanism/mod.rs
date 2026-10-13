//! Bodies in a tree: the mechanism every procedure runs on.
//!
//! # What a mechanism is
//!
//! A list of rigid bodies, the ground first, each hung from a body before it
//! by one joint: a revolute joint, a pin (the key on its balance pin, the
//! wippen on its flange), or a prismatic one, a slide (the key lifting off its
//! balance punching). Each joint has one coordinate, an angle or a length, and
//! the coordinates together place every body. A planar action needs nothing
//! more: Hirschkorn's five bodies are a tree of pins, the jack and the
//! repetition lever riding on the wippen.
//!
//! # Frames
//!
//! A body's frame has its origin at its own joint, and everything on the body
//! (its centre of mass, the points loads act at) is written in that frame,
//! once. The key's frame is the one [`crate::parts::KeyDesign`] is written in:
//! origin at the balance pin on the key's underside, `x` towards the player,
//! `y` up. Angles turn counter-clockwise in that view, so a positive angle
//! lifts the front of a key.
//!
//! # What is built
//!
//! - **Poses** ([`Mechanism::pose`]): one forward pass, parent before child,
//!   one `libm::sin` and one `libm::cos` per revolute joint and no other
//!   transcendental, so a pose is the same bits natively and in WebAssembly.
//! - **Jacobian columns** ([`Mechanism::jacobian_column`]): how fast a material
//!   point moves per unit of a coordinate, by the lever-arm identity. For a
//!   revolute joint among the point's body and its ancestors, `ẑ × (P − O)`,
//!   `O` the joint's pivot in the world; for a prismatic one, its axis in the
//!   world; for any other coordinate, zero.
//! - **Loads** ([`Mechanism::generalized_forces`], [`Mechanism::potential`]):
//!   gravity at every body's centre of mass, and forces at material points
//!   whose direction stays fixed in the world.
//! - **Prescribed coordinates and their reactions** ([`Mechanism::hold`]): a
//!   coordinate held at a value for one procedure, and the generalized force
//!   holding it takes. The jig's scale reading is one.
//!
//! Planned: contacts, coordinate limits and axis friction, and a static solve
//! that finds the coordinates a procedure leaves free. Until it exists a
//! mechanism is only ever held with every coordinate prescribed, and
//! [`Mechanism::hold`] refuses to leave one free.

mod hold;
mod load;
mod pose;

#[cfg(test)]
mod fixtures;

pub use hold::{Held, Prescribed};
pub use load::PointLoad;
pub use pose::{MaterialPoint, Placement, Pose};

use crate::geometry::{Dir, Point};

/// A body's place in its mechanism. The ground is body 0, and every body's
/// parent has a lower index than the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BodyId(pub usize);

impl BodyId {
    /// The ground, which every mechanism has first.
    pub const GROUND: Self = Self(0);
}

/// A coordinate's place in its mechanism: one per joint, in body order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoordId(pub usize);

/// What a coordinate measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordKind {
    /// The angle of a revolute joint, radians, counter-clockwise positive.
    Angle,
    /// The travel of a prismatic joint along its axis, metres.
    Length,
}

/// One generalized coordinate.
///
/// Its kind only, for now: the limits that bound it and the axis friction
/// that acts on it arrive with the static solve that reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    /// Whether it is an angle or a length.
    pub kind: CoordKind,
}

impl Coord {
    /// An angle.
    pub const ANGLE: Self = Self {
        kind: CoordKind::Angle,
    };
    /// A length.
    pub const LENGTH: Self = Self {
        kind: CoordKind::Length,
    };
}

/// How a body hangs from its parent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Joint {
    /// The ground's: fixed, with no coordinate.
    Ground,
    /// A pin. The body's frame has its origin on the pin's axis and turns
    /// about it by the coordinate; at angle zero its axes are its parent's.
    Revolute {
        /// The pin's axis, in the parent's frame.
        pivot: Point,
        /// The joint's angle.
        coord: CoordId,
    },
    /// A slide. The body's frame keeps its parent's axes, and its origin
    /// travels along `axis` from `origin` by the coordinate.
    Prismatic {
        /// Where the body's origin stands at travel zero, in the parent's
        /// frame.
        origin: Point,
        /// The direction of travel, in the parent's frame.
        axis: Dir,
        /// The joint's travel.
        coord: CoordId,
    },
}

impl Joint {
    /// The joint's coordinate, or `None` for the ground's.
    pub fn coord(&self) -> Option<CoordId> {
        match *self {
            Self::Ground => None,
            Self::Revolute { coord, .. } | Self::Prismatic { coord, .. } => Some(coord),
        }
    }
}

/// A body's mass and where it acts: what gravity needs, and all that statics
/// reads of a body's mass.
///
/// No inertia: nothing static reads one. It arrives with the dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassProps {
    /// Mass, kg.
    pub mass: f64,
    /// Centre of mass, in the body's frame.
    pub com: Point,
}

/// One rigid body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Body {
    /// The body it hangs from. The ground's is itself.
    pub parent: BodyId,
    /// How it hangs.
    pub joint: Joint,
    /// Its mass, or `None` for a body that has none in the model: a slide
    /// that only carries another body, or a key nobody has weighed.
    pub mass: Option<MassProps>,
}

impl Body {
    /// The ground.
    pub const GROUND: Self = Self {
        parent: BodyId::GROUND,
        joint: Joint::Ground,
        mass: None,
    };
}

/// Bodies in a tree, checked when built.
#[derive(Debug, Clone, PartialEq)]
pub struct Mechanism {
    bodies: Vec<Body>,
    coords: Vec<Coord>,
}

/// What the mechanism refuses: a list of bodies that is not a tree it can
/// place, or a prescription or a load it cannot be held under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    /// Body 0 must be the ground: no joint, its own parent.
    GroundNotFirst,
    /// Only body 0 is the ground.
    SecondGround {
        /// The body that claims to be it.
        body: BodyId,
    },
    /// A body's parent must come before it, so one pass places them all.
    ParentNotBefore {
        /// The body.
        body: BodyId,
    },
    /// Coordinates are one per joint, in body order, and this body's joint
    /// names another.
    CoordinateOutOfOrder {
        /// The body.
        body: BodyId,
    },
    /// A joint whose coordinate is not in the list of coordinates.
    JointWithoutCoordinate {
        /// The body.
        body: BodyId,
    },
    /// An angle on a slide or a length on a pin.
    CoordinateKindMismatch {
        /// The body whose joint and coordinate disagree.
        body: BodyId,
    },
    /// A coordinate that no joint moves.
    CoordinateWithoutJoint {
        /// The first such coordinate.
        coord: CoordId,
    },
    /// A pivot, an origin, a mass or a centre of mass that is not a finite
    /// number.
    NotFinite {
        /// The body.
        body: BodyId,
    },
    /// A mass of zero or less. A body with no mass in the model has `None`.
    MassNotPositive {
        /// The body.
        body: BodyId,
    },
    /// A coordinate the procedure did not prescribe. Nothing solves for a
    /// free coordinate until the static solve exists.
    NotPrescribed {
        /// The first coordinate left free.
        coord: CoordId,
    },
    /// A coordinate prescribed twice.
    PrescribedTwice {
        /// The coordinate.
        coord: CoordId,
    },
    /// A coordinate prescribed a value that is not a finite number.
    PrescribedNotFinite {
        /// The coordinate.
        coord: CoordId,
    },
    /// A coordinate the mechanism does not have.
    NoSuchCoordinate {
        /// The coordinate asked for.
        coord: CoordId,
    },
    /// A load on a body the mechanism does not have.
    NoSuchBody {
        /// The body asked for.
        body: BodyId,
    },
    /// A load whose point or force is not a finite number.
    LoadNotFinite {
        /// The load's place in the list it was given in.
        load: usize,
    },
}

fn point_is_finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

/// A body's mass is absent, or positive and finite with a finite centre.
fn check_mass(id: BodyId, mass: Option<MassProps>) -> Result<(), Problem> {
    if let Some(m) = mass {
        if !(m.mass.is_finite() && point_is_finite(m.com)) {
            return Err(Problem::NotFinite { body: id });
        }
        if m.mass <= 0.0 {
            return Err(Problem::MassNotPositive { body: id });
        }
    }
    Ok(())
}

impl Mechanism {
    /// Builds a mechanism from its bodies, the ground first, and its
    /// coordinates, one per joint in body order; or names the first thing
    /// that keeps it from being one.
    pub fn new(bodies: Vec<Body>, coords: Vec<Coord>) -> Result<Self, Problem> {
        match bodies.first() {
            Some(b) if b.joint == Joint::Ground && b.parent == BodyId::GROUND => {
                check_mass(BodyId::GROUND, b.mass)?;
            }
            _ => return Err(Problem::GroundNotFirst),
        }
        let mut next_coord = 0;
        for (i, body) in bodies.iter().enumerate().skip(1) {
            let id = BodyId(i);
            if body.parent.0 >= i {
                return Err(Problem::ParentNotBefore { body: id });
            }
            let (coord, kind, finite) = match body.joint {
                Joint::Ground => return Err(Problem::SecondGround { body: id }),
                Joint::Revolute { pivot, coord } => {
                    (coord, CoordKind::Angle, point_is_finite(pivot))
                }
                Joint::Prismatic { origin, coord, .. } => {
                    (coord, CoordKind::Length, point_is_finite(origin))
                }
            };
            if coord.0 != next_coord {
                return Err(Problem::CoordinateOutOfOrder { body: id });
            }
            match coords.get(coord.0) {
                Some(c) if c.kind == kind => {}
                Some(_) => return Err(Problem::CoordinateKindMismatch { body: id }),
                None => return Err(Problem::JointWithoutCoordinate { body: id }),
            }
            next_coord += 1;
            if !finite {
                return Err(Problem::NotFinite { body: id });
            }
            check_mass(id, body.mass)?;
        }
        if next_coord != coords.len() {
            return Err(Problem::CoordinateWithoutJoint {
                coord: CoordId(next_coord),
            });
        }
        Ok(Self { bodies, coords })
    }

    /// The bodies, the ground first.
    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    /// The coordinates, one per joint in body order.
    pub fn coords(&self) -> &[Coord] {
        &self.coords
    }

    /// One body.
    ///
    /// # Panics
    ///
    /// When the mechanism has no such body: an identifier from another
    /// mechanism, which is the caller's error.
    pub fn body(&self, id: BodyId) -> &Body {
        &self.bodies[id.0]
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::chain;
    use super::*;

    fn pin(parent: usize, coord: usize) -> Body {
        Body {
            parent: BodyId(parent),
            joint: Joint::Revolute {
                pivot: Point::new(0.1, 0.0),
                coord: CoordId(coord),
            },
            mass: None,
        }
    }

    /// The shared test chain is a mechanism: the checks pass what they should.
    #[test]
    fn a_tree_with_one_coordinate_per_joint_in_body_order_is_a_mechanism() {
        let m = chain();
        assert_eq!(m.bodies().len(), 6);
        assert_eq!(m.coords().len(), 5);
    }

    /// Each check names what is wrong, and with which body.
    #[test]
    fn what_is_not_a_tree_is_named() {
        let angle = vec![Coord::ANGLE];
        assert_eq!(
            Mechanism::new(vec![pin(0, 0)], angle.clone()),
            Err(Problem::GroundNotFirst)
        );
        assert_eq!(
            Mechanism::new(vec![Body::GROUND, Body::GROUND], vec![]),
            Err(Problem::SecondGround { body: BodyId(1) })
        );
        assert_eq!(
            Mechanism::new(vec![Body::GROUND, pin(1, 0)], angle.clone()),
            Err(Problem::ParentNotBefore { body: BodyId(1) })
        );
        assert_eq!(
            Mechanism::new(vec![Body::GROUND, pin(0, 1)], vec![Coord::ANGLE; 2]),
            Err(Problem::CoordinateOutOfOrder { body: BodyId(1) })
        );
        assert_eq!(
            Mechanism::new(vec![Body::GROUND, pin(0, 0)], vec![Coord::LENGTH]),
            Err(Problem::CoordinateKindMismatch { body: BodyId(1) })
        );
        assert_eq!(
            Mechanism::new(vec![Body::GROUND, pin(0, 0)], vec![Coord::ANGLE; 2]),
            Err(Problem::CoordinateWithoutJoint { coord: CoordId(1) })
        );
        assert_eq!(
            Mechanism::new(vec![Body::GROUND, pin(0, 0)], vec![]),
            Err(Problem::JointWithoutCoordinate { body: BodyId(1) })
        );
    }

    /// A mass is positive and finite, or absent, on every body, the ground's
    /// included; a pivot is finite.
    #[test]
    fn a_mass_or_a_pivot_that_is_not_a_number_is_refused() {
        let with_mass = |mass: f64, com: Point| Body {
            mass: Some(MassProps { mass, com }),
            ..pin(0, 0)
        };
        let build = |b: Body| Mechanism::new(vec![Body::GROUND, b], vec![Coord::ANGLE]);
        let origin = Point::new(0.0, 0.0);
        assert!(build(with_mass(0.1, origin)).is_ok());
        assert!(build(Body {
            mass: None,
            ..pin(0, 0)
        })
        .is_ok());
        assert_eq!(
            build(with_mass(0.0, origin)),
            Err(Problem::MassNotPositive { body: BodyId(1) })
        );
        assert_eq!(
            build(with_mass(-0.1, origin)),
            Err(Problem::MassNotPositive { body: BodyId(1) })
        );
        assert_eq!(
            build(with_mass(f64::NAN, origin)),
            Err(Problem::NotFinite { body: BodyId(1) })
        );
        assert_eq!(
            build(with_mass(0.1, Point::new(f64::INFINITY, 0.0))),
            Err(Problem::NotFinite { body: BodyId(1) })
        );
        let nowhere = Body {
            joint: Joint::Revolute {
                pivot: Point::new(f64::NAN, 0.0),
                coord: CoordId(0),
            },
            ..pin(0, 0)
        };
        assert_eq!(build(nowhere), Err(Problem::NotFinite { body: BodyId(1) }));
        let weighed_ground = Body {
            mass: Some(MassProps {
                mass: f64::NAN,
                com: origin,
            }),
            ..Body::GROUND
        };
        assert_eq!(
            Mechanism::new(vec![weighed_ground, pin(0, 0)], vec![Coord::ANGLE]),
            Err(Problem::NotFinite {
                body: BodyId::GROUND
            })
        );
    }
}
