//! What the mechanism's tests share: a chain with every kind of joint on every
//! kind of parent, the points probed on it, and a seeded generator.

use super::{Body, BodyId, Coord, CoordId, CoordKind, Joint, MassProps, MaterialPoint, Mechanism};
use crate::geometry::{Dir, Point};

/// A seeded xorshift: the same sequence on every machine, and no dependency.
pub struct XorShift(u64);

impl XorShift {
    /// A generator from a seed; zero, which xorshift cannot leave, becomes one.
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in `[lo, hi)`, from the top 53 bits.
    pub fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        let unit = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        lo + (hi - lo) * unit
    }
}

fn mass(mass: f64, x: f64, y: f64) -> Option<MassProps> {
    Some(MassProps {
        mass,
        com: Point::new(x, y),
    })
}

/// Every kind of joint on every kind of parent, and a branch beside them:
///
/// - `a` (body 1), a pin on the ground;
/// - `b` (body 2), a pin on a pin;
/// - `c` (body 3), a slide under a pin, its axis neither vertical nor
///   horizontal;
/// - `d` (body 4), a pin under a slide, the shape of a key riding on its
///   balance punching;
/// - `e` (body 5), a pin on the ground, on a branch of its own.
///
/// Sizes are an action's: arms of a few centimetres to a few decimetres,
/// masses of a few grams to a hundred.
pub fn chain() -> Mechanism {
    let pin = |x: f64, y: f64, coord: usize| Joint::Revolute {
        pivot: Point::new(x, y),
        coord: CoordId(coord),
    };
    let bodies = vec![
        Body::GROUND,
        Body {
            parent: BodyId(0),
            joint: pin(0.05, 0.02, 0),
            mass: mass(0.100, 0.12, 0.01),
        },
        Body {
            parent: BodyId(1),
            joint: pin(0.15, -0.03, 1),
            mass: mass(0.020, 0.04, 0.02),
        },
        Body {
            parent: BodyId(2),
            joint: Joint::Prismatic {
                origin: Point::new(0.03, 0.01),
                axis: Dir::new(0.3, 1.0).expect("a direction"),
                coord: CoordId(2),
            },
            mass: mass(0.005, 0.0, 0.01),
        },
        Body {
            parent: BodyId(3),
            joint: pin(0.01, 0.02, 3),
            mass: mass(0.010, 0.05, -0.01),
        },
        Body {
            parent: BodyId(0),
            joint: pin(-0.10, 0.0, 4),
            mass: mass(0.030, -0.08, 0.02),
        },
    ];
    let coords = vec![
        Coord::ANGLE,
        Coord::ANGLE,
        Coord::LENGTH,
        Coord::ANGLE,
        Coord::ANGLE,
    ];
    Mechanism::new(bodies, coords).expect("the test chain is a mechanism")
}

/// The points probed on every body but the ground: its frame's origin, a
/// point above and in front, and one below and behind.
pub fn probes(m: &Mechanism) -> Vec<MaterialPoint> {
    (1..m.bodies().len())
        .flat_map(|b| {
            [(0.0, 0.0), (0.11, 0.04), (-0.03, -0.02)].map(|(x, y)| MaterialPoint {
                body: BodyId(b),
                at: Point::new(x, y),
            })
        })
        .collect()
}

/// Coordinates drawn uniformly: angles within a radian of zero, lengths
/// within 20 mm.
pub fn random_q(m: &Mechanism, rng: &mut XorShift) -> Vec<f64> {
    m.coords()
        .iter()
        .map(|c| match c.kind {
            CoordKind::Angle => rng.uniform(-1.0, 1.0),
            CoordKind::Length => rng.uniform(-0.02, 0.02),
        })
        .collect()
}

/// The lengths of `point`'s chain added up, metres: the point's own offset,
/// every pivot it hangs from, and every slide's origin and travel.
///
/// By the triangle inequality it bounds the point's distance from the world's
/// origin and its lever arm about every joint that moves it, which is what
/// the derived tolerances need.
pub fn reach(m: &Mechanism, q: &[f64], point: MaterialPoint) -> f64 {
    let length = |p: Point| (p.x * p.x + p.y * p.y).sqrt();
    let mut total = length(point.at);
    let mut body = point.body;
    while body != BodyId::GROUND {
        let b = m.body(body);
        total += match b.joint {
            Joint::Ground => 0.0,
            Joint::Revolute { pivot, .. } => length(pivot),
            Joint::Prismatic { origin, coord, .. } => length(origin) + q[coord.0].abs(),
        };
        body = b.parent;
    }
    total
}
