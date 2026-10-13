//! What the contact tests share: the mechanism tests' chain, carrying parts
//! of every kind a contact reads, and pairs between them.

use super::{
    ContactCode, ContactLaw, ContactPair, Material, PairGeometry, Part, PartCode, PartId,
    Provenance, Role, Surface, SurfaceRef, Sweep,
};
use crate::geometry::{Dir, Point, Seg};
use crate::mechanism::fixtures::chain;
use crate::mechanism::{BodyId, Mechanism};

fn line(a: Point, b: Point) -> Seg {
    Seg::Line { a, b }
}

fn polygon(points: &[(f64, f64)]) -> Vec<Seg> {
    (0..points.len())
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % points.len()]);
            line(Point::new(a.0, a.1), Point::new(b.0, b.1))
        })
        .collect()
}

fn part(body: usize, outline: Vec<Seg>, role: Role, sweep: Sweep) -> Part {
    let n = u16::try_from(outline.len()).expect("a short outline");
    Part {
        body: BodyId(body),
        code: PartCode::Keystick,
        outline,
        sweep,
        role,
        material: Material::KeyWood,
        sources: vec![(0..n, Provenance::Assumed("a test"))],
    }
}

fn prism() -> Sweep {
    Sweep::Extruded { width: 0.02 }
}

/// A block with a square corner, a rounded one and a chamfer, the shape of
/// a key's end: on its bottom, from the chamfer's foot forward, an arc round
/// the front corner, a front, a top, the chamfer. Each line takes its ends
/// from the arc beside it, so the outline closes bit for bit.
fn block(body: usize) -> Part {
    let arc = Seg::Arc {
        c: Point::new(0.04, 0.01),
        r: 0.01,
        from: Dir::NEG_Y,
        to: Dir::X,
    };
    let foot = Point::new(-0.04, 0.0);
    let top = Point::new(-0.04, 0.03);
    let heel = Point::new(-0.05, 0.02);
    let outline = vec![
        line(foot, arc.start()),
        arc,
        line(arc.end(), Point::new(arc.end().x, 0.03)),
        line(Point::new(arc.end().x, 0.03), top),
        line(top, heel),
        line(heel, foot),
    ];
    part(body, outline, Role::Solid, prism())
}

/// A strip with its two top edges rounded, as the back rail cloth is.
fn strip(body: usize) -> Part {
    let r = 0.002;
    let front = Seg::Arc {
        c: Point::new(0.028, 0.008),
        r,
        from: Dir::X,
        to: Dir::Y,
    };
    let back = Seg::Arc {
        c: Point::new(-0.028, 0.008),
        r,
        from: Dir::Y,
        to: Dir::NEG_X,
    };
    let (front_low, back_low) = (front.start(), back.end());
    let outline = vec![
        line(Point::new(back_low.x, 0.0), Point::new(front_low.x, 0.0)),
        line(Point::new(front_low.x, 0.0), front_low),
        front,
        line(front.end(), back.start()),
        back,
        line(back_low, Point::new(back_low.x, 0.0)),
    ];
    part(body, outline, Role::Lining, prism())
}

/// A wheel: a half disc over a bottom of two lines and a small arc, with a
/// concave junction among its corners, for arcs meeting arcs.
fn wheel(body: usize) -> Part {
    let big = Seg::Arc {
        c: Point::new(0.0, 0.0),
        r: 0.015,
        from: Dir::X,
        to: Dir::NEG_X,
    };
    let small = Seg::Arc {
        c: Point::new(0.0, -0.004),
        r: 0.004,
        from: Dir::NEG_Y,
        to: Dir::X,
    };
    let outline = vec![
        big,
        line(big.end(), small.start()),
        small,
        line(small.end(), big.start()),
    ];
    part(body, outline, Role::Solid, prism())
}

/// The fixture's parts, in order, and the ids the pairs read them by.
pub struct Parts {
    /// Every part.
    pub parts: Vec<Part>,
    /// The block, on `b`.
    pub block: PartId,
    /// The strip, on `d`.
    pub strip: PartId,
    /// The wheel, on `a`.
    pub wheel: PartId,
    /// A slot cut in `c`, the slide `d` hangs from: a cut over the strip.
    pub slot: PartId,
    /// A post on `e`, far enough out on it to sweep through where `b` is.
    pub post: PartId,
    /// A washer turned on the ground.
    pub washer: PartId,
}

/// Parts on every body of the chain, the ground included.
pub fn parts() -> Parts {
    let mut parts = vec![block(2), strip(4), wheel(1)];
    parts.push(part(
        3,
        polygon(&[(-0.04, 0.0), (0.06, 0.0), (0.06, 0.04), (-0.04, 0.04)]),
        Role::Cut,
        Sweep::Extruded { width: 0.0 },
    ));
    parts.push(part(
        5,
        polygon(&[(0.26, -0.03), (0.30, -0.03), (0.30, 0.01), (0.26, 0.01)]),
        Role::Solid,
        prism(),
    ));
    parts.push(part(
        0,
        polygon(&[(0.002, 0.0), (0.011, 0.0), (0.011, 0.003), (0.002, 0.003)]),
        Role::Lining,
        Sweep::Revolved { axis_x: 0.1 },
    ));
    Parts {
        parts,
        block: PartId(0),
        strip: PartId(1),
        wheel: PartId(2),
        slot: PartId(3),
        post: PartId(4),
        washer: PartId(5),
    }
}

fn range(part: PartId, segs: core::ops::Range<u16>) -> SurfaceRef {
    SurfaceRef { part, segs }
}

/// Pairs of every geometry, between bodies related every way: a body and
/// its descendant, a parent and its child through a slide, two branches, a
/// body and the ground; a cut on one side; a side that is a chain of two
/// ranges round the end of an outline; a turned part.
pub fn pairs(p: &Parts) -> Vec<ContactPair> {
    let pair = |code, a, b, geometry| ContactPair {
        code,
        a,
        b,
        geometry,
        law: ContactLaw::Rigid,
    };
    // A turned part's silhouette has its top in the two segments either side
    // of the axis, numbered by its profile's count (`Part::silhouette`).
    let profile = u16::try_from(p.parts[p.washer.index()].outline.len()).unwrap();
    let silhouette = u16::try_from(p.parts[p.washer.index()].silhouette().len()).unwrap();
    vec![
        // The block on the strip: `b` and its descendant `d`.
        pair(
            ContactCode::BackRailCloth,
            Surface::one(range(p.block, 0..6)),
            Surface::one(range(p.strip, 1..6)),
            PairGeometry::Features,
        ),
        // The wheel on the block: `a` and its child `b`, arcs on arcs.
        pair(
            ContactCode::BackRailCloth,
            Surface::one(range(p.wheel, 0..4)),
            Surface::one(range(p.block, 0..6)),
            PairGeometry::Features,
        ),
        // The slot's top, a cut in the slide, on the strip's top below it.
        pair(
            ContactCode::FrontPinInTheMortise,
            Surface::one(range(p.slot, 2..3)),
            Surface::one(range(p.strip, 2..5)),
            PairGeometry::Features,
        ),
        // The washer's centre under the block's chamfer and bottom, a chain
        // round the end of the block's outline.
        pair(
            ContactCode::FrontRailPunching,
            Surface::of(&[range(p.block, 5..6), range(p.block, 0..1)]),
            Surface::one(range(p.washer, profile - 2..profile)),
            PairGeometry::CentreBearing,
        ),
        // The post on the block: two branches.
        pair(
            ContactCode::FrontPinOnTheWood,
            Surface::one(range(p.post, 0..4)),
            Surface::one(range(p.block, 0..6)),
            PairGeometry::Features,
        ),
        // The post on the washer: a branch and the ground, a turned part.
        pair(
            ContactCode::FrontPinOnTheWood,
            Surface::one(range(p.post, 0..4)),
            Surface::one(range(p.washer, 0..silhouette)),
            PairGeometry::Features,
        ),
    ]
}

/// The chain, with its parts and pairs.
pub fn mechanism() -> Mechanism {
    let p = parts();
    let pairs = pairs(&p);
    chain()
        .with_contacts(p.parts, pairs)
        .expect("the fixture's contacts are drawn")
}
