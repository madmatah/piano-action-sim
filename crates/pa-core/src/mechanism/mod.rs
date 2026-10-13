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
//! - **Parts and contacts** ([`Mechanism::with_contacts`]): the drawn parts
//!   its bodies carry and the pairs of their ranges declared as contacts,
//!   read at any pose into rows over every coordinate
//!   ([`Mechanism::rows`], in [`crate::contact`]).
//! - **Limits and friction** ([`Coord`]): the interval each coordinate may
//!   not leave, and its joint's dry friction, a set-valued bound.
//!
//! The static solve ([`crate::solve`]) finds the coordinates a procedure
//! leaves free; [`Mechanism::hold`] holds a mechanism with every coordinate
//! prescribed, as the jig holds the key, and refuses to leave one free.

mod hold;
mod load;
mod pose;

#[cfg(test)]
pub(crate) mod fixtures;

pub use hold::{Held, Prescribed};
pub use load::PointLoad;
pub use pose::{MaterialPoint, Placement, Pose};

use crate::contact::{ContactPair, PairFeatures, Part, PartId, SurfaceRef};
use crate::geometry::{Dir, Point};
use crate::registry::{Kind, Layer, ParamSpec, Quantity, Step};

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

/// One generalized coordinate: what it measures, the range the modelled
/// world gives it, and the dry friction of its joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    /// Whether it is an angle or a length.
    pub kind: CoordKind,
    /// The least and the greatest value it may take, radians or metres: the
    /// edge of the modelled world on either side. A static solve reads each
    /// as a unilateral row, and refuses a state that leans on one
    /// ([`crate::solve::Refusal::NoEquilibrium`]), since nothing modelled
    /// stands there. Infinite for a coordinate only ever prescribed, as the
    /// jig's; a solve that leaves a coordinate free needs both finite, so
    /// that what it searches is bounded.
    pub limits: (f64, f64),
    /// The bound of its joint's dry friction: a torque in newton-metres on an
    /// angle, a force in newtons on a length. Set-valued: while the joint is
    /// still it takes any value within plus or minus this
    /// (`docs/FOUNDATIONS.md` §3.5). Zero for a joint that turns freely.
    pub friction: f64,
}

impl Coord {
    /// An angle, unbounded and frictionless.
    pub const ANGLE: Self = Self {
        kind: CoordKind::Angle,
        limits: (f64::NEG_INFINITY, f64::INFINITY),
        friction: 0.0,
    };
    /// A length, unbounded and frictionless.
    pub const LENGTH: Self = Self {
        kind: CoordKind::Length,
        limits: (f64::NEG_INFINITY, f64::INFINITY),
        friction: 0.0,
    };

    /// This coordinate, bounded to `[lo, hi]`.
    pub const fn limited(self, lo: f64, hi: f64) -> Self {
        Self {
            limits: (lo, hi),
            ..self
        }
    }

    /// This coordinate, its joint's dry friction bounded by `bound`.
    pub const fn with_friction(self, bound: f64) -> Self {
        Self {
            friction: bound,
            ..self
        }
    }
}

/// A pin's dry friction as it is known: a torque bound, and the radius it
/// was measured at (`CLAUDE.md` rule 4).
///
/// The trade measures a key's bushings as a key friction weight, grams at the
/// measuring point, and publishes pivot frictions as torques taken at radii
/// that differ from one source to the next. So the bound is kept as what it
/// is, a torque, with the radius a weight in grams was read at: moving the
/// measuring point changes the grams it reads as, never the bushings. What a
/// mechanism's coordinate carries is the bound alone ([`Coord::friction`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisFriction {
    /// The bound, newton-metres: any torque within plus or minus this holds
    /// the pin still.
    pub bound_n_m: f64,
    /// The radius a weight read it at, metres; `None` where the source gives
    /// a torque and no radius, as Hirschkorn's table does.
    pub radius_m: Option<f64>,
}

impl AxisFriction {
    /// The bushings' setting: the torque they hold, no less than nothing.
    /// A torque set as such is read at no radius, as a published one is;
    /// one typed as a key friction weight keeps the radius it was typed at.
    pub const PARAMS: [ParamSpec<Self>; 1] = [ParamSpec {
        name: "friction.bound",
        quantity: Quantity::Torque,
        hard: (0.0, f64::INFINITY),
        step: Step::Continuous,
        layer: Layer::Design,
        kind: Kind::Actuated,
        get: |f| f.bound_n_m,
        set: |f, v| {
            f.bound_n_m = v;
            f.radius_m = None;
        },
    }];

    /// The friction a weight of `grams` overcomes at `radius_m` from the pin,
    /// with the lever level: bushings set straight in grams on the level arm,
    /// as the bench's default is. A key friction weight measured by
    /// Stanwood's protocol is read on the key resting nose-up, and is turned
    /// into a torque by running that protocol instead
    /// ([`crate::measure::weighing::Weighing::bushings_reading`]).
    pub fn of_weight_at(grams: f64, radius_m: f64) -> Self {
        Self {
            bound_n_m: grams * 1e-3 * radius_m * crate::G,
            radius_m: Some(radius_m),
        }
    }

    /// The bound as the weight in grams that overcomes it at `radius_m`.
    pub fn weight_at(&self, radius_m: f64) -> f64 {
        1000.0 * self.bound_n_m / (crate::G * radius_m)
    }

    /// This friction multiplied by `k`, read at the same radius: the
    /// non-confusion test's bushings made 1.2 times as tight.
    pub fn scaled(&self, k: f64) -> Self {
        Self {
            bound_n_m: self.bound_n_m * k,
            ..*self
        }
    }
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

/// Bodies in a tree, checked when built, with the parts they carry and the
/// contacts declared between them.
#[derive(Debug, Clone, PartialEq)]
pub struct Mechanism {
    bodies: Vec<Body>,
    coords: Vec<Coord>,
    parts: Vec<Part>,
    pairs: Vec<ContactPair>,
    /// Each pair's features, read once from the parts when the pairs are
    /// declared.
    pub(crate) features: Vec<PairFeatures>,
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
    /// A coordinate whose limits are not a range: a lower limit above the
    /// upper one, or one that is not a number.
    LimitsNotARange {
        /// The coordinate.
        coord: CoordId,
    },
    /// A coordinate whose friction bound is negative or not a finite number.
    FrictionNotABound {
        /// The coordinate.
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
    /// A coordinate left free where every one must be prescribed: holding a
    /// mechanism still ([`Mechanism::hold`]) solves for nothing, and a
    /// coordinate it is to find belongs to the static solve
    /// ([`crate::solve::Statics`]).
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
    /// A coordinate a solve leaves free whose limits are not both finite:
    /// the search for its equilibrium would not be bounded.
    Unlimited {
        /// The coordinate.
        coord: CoordId,
    },
    /// A coordinate a solve leaves free that moves nothing drawn, weighed or
    /// loaded: nothing could act on it or measure how far it moves.
    MovesNothing {
        /// The coordinate.
        coord: CoordId,
    },
    /// A solve started from a value that is not a finite number.
    StartNotFinite {
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
    /// A part carried by a body the mechanism does not have.
    PartOnNoSuchBody {
        /// The part.
        part: PartId,
    },
    /// A part whose outline is not closed bit for bit, or whose points are
    /// not finite numbers: its segments' ends would not say where its
    /// corners are.
    OutlineNotClosed {
        /// The part.
        part: PartId,
    },
    /// A pair with a side that holds no range, or a range that holds no
    /// segment, or one that names a part or segments that are not drawn.
    RangeNotDrawn {
        /// The pair's place in the list.
        pair: usize,
    },
    /// A pair with a side whose ranges are carried by two bodies.
    SideOnTwoBodies {
        /// The pair's place in the list.
        pair: usize,
    },
    /// A pair whose two sides are carried by one body: nothing could move
    /// one against the other.
    PairOnOneBody {
        /// The pair's place in the list.
        pair: usize,
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
        for (c, coord) in coords.iter().enumerate() {
            let (lo, hi) = coord.limits;
            if lo.is_nan() || hi.is_nan() || lo > hi {
                return Err(Problem::LimitsNotARange { coord: CoordId(c) });
            }
            if !(coord.friction.is_finite() && coord.friction >= 0.0) {
                return Err(Problem::FrictionNotABound { coord: CoordId(c) });
            }
        }
        Ok(Self {
            bodies,
            coords,
            parts: Vec::new(),
            pairs: Vec::new(),
            features: Vec::new(),
        })
    }

    /// This mechanism with `parts` on its bodies and `pairs` declared between
    /// them, in the order the rows will read them; or the first thing that
    /// keeps them from being read.
    ///
    /// Every part stands on a body the mechanism has, its outline closed bit
    /// for bit; every pair's two sides hold ranges of those parts' drawn
    /// silhouettes, each side on one body and the two on different bodies.
    /// The pairs' features are read from the parts here, once.
    pub fn with_contacts(
        mut self,
        parts: Vec<Part>,
        pairs: Vec<ContactPair>,
    ) -> Result<Self, Problem> {
        for (i, part) in parts.iter().enumerate() {
            let id = PartId(u16::try_from(i).expect("a mechanism carries fewer than 65 536 parts"));
            if part.body.0 >= self.bodies.len() {
                return Err(Problem::PartOnNoSuchBody { part: id });
            }
            let finite = part
                .outline
                .iter()
                .all(|s| point_is_finite(s.start()) && point_is_finite(s.end()));
            if !finite || !part.is_closed() {
                return Err(Problem::OutlineNotClosed { part: id });
            }
        }
        for (i, pair) in pairs.iter().enumerate() {
            let body_a = side_body(&pair.a.ranges, &parts, i)?;
            let body_b = side_body(&pair.b.ranges, &parts, i)?;
            if body_a == body_b {
                return Err(Problem::PairOnOneBody { pair: i });
            }
        }
        self.features = pairs
            .iter()
            .map(|pair| PairFeatures::of(pair, &parts))
            .collect();
        self.parts = parts;
        self.pairs = pairs;
        Ok(self)
    }

    /// This mechanism with the dry friction of `coord`'s joint bounded by
    /// `bound` newton-metres instead; or why not: a coordinate it does not
    /// have, or a bound that is negative or not a finite number. Everything
    /// else is kept as it was, bit for bit: a procedure that tries the same
    /// key on other bushings reads the same parts, contacts and masses.
    pub fn with_friction(mut self, coord: CoordId, bound: f64) -> Result<Self, Problem> {
        let c = self
            .coords
            .get_mut(coord.0)
            .ok_or(Problem::NoSuchCoordinate { coord })?;
        if !(bound.is_finite() && bound >= 0.0) {
            return Err(Problem::FrictionNotABound { coord });
        }
        c.friction = bound;
        Ok(self)
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

    /// The parts its bodies carry: what is drawn, and all a contact reads.
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    /// The contacts declared between its bodies, in the order rows read them.
    pub fn pairs(&self) -> &[ContactPair] {
        &self.pairs
    }
}

/// The body one side of pair `pair` is carried by, every range of it being
/// drawn: a part the list holds, and at least one segment its silhouette has.
fn side_body(ranges: &[SurfaceRef], parts: &[Part], pair: usize) -> Result<BodyId, Problem> {
    let mut body = None;
    for range in ranges {
        let drawn = parts.get(range.part.index()).filter(|part| {
            range.segs.start < range.segs.end
                && usize::from(range.segs.end) <= part.silhouette().len()
        });
        let Some(part) = drawn else {
            return Err(Problem::RangeNotDrawn { pair });
        };
        match body {
            None => body = Some(part.body),
            Some(b) if b == part.body => {}
            Some(_) => return Err(Problem::SideOnTwoBodies { pair }),
        }
    }
    body.ok_or(Problem::RangeNotDrawn { pair })
}

#[cfg(test)]
mod tests {
    use super::fixtures::chain;
    use super::*;
    use crate::contact::fixtures::{pairs, parts};
    use crate::contact::Surface;

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

    /// The contacts a mechanism is given are checked as the bodies are: each
    /// part on a body it has, closed; each pair's sides drawn ranges of those
    /// parts, each side on one body, the two on two.
    #[test]
    fn contacts_that_are_not_drawn_are_named() {
        let with = |edit: &dyn Fn(&mut Vec<Part>, &mut Vec<ContactPair>)| {
            let p = parts();
            let (mut list, mut declared) = (p.parts.clone(), pairs(&p));
            edit(&mut list, &mut declared);
            chain().with_contacts(list, declared)
        };
        assert!(with(&|_, _| {}).is_ok());
        assert_eq!(
            with(&|parts, _| parts[0].body = BodyId(9)).err(),
            Some(Problem::PartOnNoSuchBody { part: PartId(0) })
        );
        assert_eq!(
            with(&|parts, _| {
                if let crate::geometry::Seg::Line { b, .. } = &mut parts[1].outline[0] {
                    b.x = f64::from_bits(b.x.to_bits() + 1);
                }
            })
            .err(),
            Some(Problem::OutlineNotClosed { part: PartId(1) })
        );
        assert_eq!(
            with(&|_, pairs| pairs[2].a.ranges[0].segs = 2..9).err(),
            Some(Problem::RangeNotDrawn { pair: 2 })
        );
        assert_eq!(
            with(&|_, pairs| pairs[2].a = Surface { ranges: Vec::new() }).err(),
            Some(Problem::RangeNotDrawn { pair: 2 })
        );
        assert_eq!(
            with(&|_, pairs| pairs[0].a.ranges[0].segs = 3..3).err(),
            Some(Problem::RangeNotDrawn { pair: 0 })
        );
        assert_eq!(
            with(&|_, pairs| {
                let strip = pairs[0].b.ranges[0].clone();
                pairs[0].a.ranges.push(strip);
            })
            .err(),
            Some(Problem::SideOnTwoBodies { pair: 0 })
        );
        assert_eq!(
            with(&|_, pairs| pairs[1].b = pairs[1].a.clone()).err(),
            Some(Problem::PairOnOneBody { pair: 1 })
        );
    }

    /// Other bushings on the same mechanism: one coordinate's friction bound
    /// changes and nothing else does; a bound that is not one, or a
    /// coordinate the mechanism does not have, is named.
    #[test]
    fn other_bushings_change_one_bound_and_nothing_else() {
        let p = parts();
        let m = chain()
            .with_contacts(p.parts.clone(), pairs(&p))
            .expect("the test chain's contacts are drawn");
        let tighter = m.clone().with_friction(CoordId(2), 0.25).expect("a bound");
        for (c, (was, now)) in m.coords().iter().zip(tighter.coords()).enumerate() {
            let expected = if c == 2 { 0.25 } else { was.friction };
            assert_eq!(now.friction.to_bits(), expected.to_bits(), "coordinate {c}");
            assert_eq!(
                Coord {
                    friction: 0.0,
                    ..*now
                },
                Coord {
                    friction: 0.0,
                    ..*was
                }
            );
        }
        let back = tighter
            .with_friction(CoordId(2), m.coords()[2].friction)
            .expect("a bound");
        assert_eq!(back, m, "the same bushings again, the same mechanism");
        for bad in [-1e-9, f64::NAN, f64::INFINITY] {
            assert_eq!(
                m.clone().with_friction(CoordId(2), bad),
                Err(Problem::FrictionNotABound { coord: CoordId(2) }),
                "{bad}"
            );
        }
        assert_eq!(
            m.clone().with_friction(CoordId(9), 0.1),
            Err(Problem::NoSuchCoordinate { coord: CoordId(9) })
        );
    }
}
