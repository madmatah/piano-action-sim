//! Contact rows: every declared pair read at a pose, each row a signed gap
//! with its gradient and its slip over all coordinates.
//!
//! # A row
//!
//! One pair of features of one pair of surfaces ([`Feature`]): the signed gap
//! between them, the normal from side B toward side A, and the point midway
//! between the two surfaces on that normal, all in the world. With it, one
//! entry per coordinate of the mechanism:
//!
//! - the **gradient** of the gap, `∂g/∂q_c = n · (J_A,c(P) − J_B,c(P))`, the
//!   lever-arm identity (`docs/FOUNDATIONS.md` > The lever arm, and the gradient of a gap) extended to every
//!   ancestor joint: a coordinate that moves both bodies moves both
//!   material points alike and cancels exactly;
//! - the **slip gradient**, `t · (J_A,c(P) − J_B,c(P))` with `t = ẑ × n`: how
//!   fast the two material points at the contact point slide past each other
//!   per unit of the coordinate, which is what friction at a sliding contact
//!   will read. Unlike the gradient it depends on where along the normal the
//!   point is taken, by the relative rotation times half the gap, which is
//!   why the point is the midway one.
//!
//! # Only overlaps the parts hold
//!
//! A row against a face reads the distance to the face's line, so a point far
//! behind a thin part reads as deep inside it though it has passed out
//! through the far side. A row deeper than the row margin is kept only where
//! its point stands inside the face's side's drawn material, and, of a
//! corner's rows that deep against several faces, only the shallowest: the
//! nearest face is the one it comes out through. A face's center, which a
//! center bearing reads, gives a row deeper than the margin only where no
//! other side of the material stands nearer it by more than the margin: the
//! face is then the way it came in.
//!
//! # Within a margin
//!
//! A row is generated when its gap is at most the margin asked for. The
//! static solve ([`crate::solve`]) asks for [`ROW_MARGIN`] and bounds its
//! steps so that no two declared surfaces approach each other by more than
//! that between two readings, so that no pair of features can be stepped
//! over. A reading of how far apart two parts stand asks for every row, with
//! an infinite margin.

use super::features::{Feature, PairFeatures, Piece};
use super::{PairGeometry, PartId};
use crate::geometry::{
    center_on_face, disc_on_disc, disc_on_face, Dir, Disc, Face, Point, Proximity, Seg,
};
use crate::mechanism::{BodyId, Mechanism, Placement, Pose};

/// How close two declared surfaces must come for the static solve to read a
/// row between them, meters: a millimeter.
///
/// A margin and not a contact tolerance: a row within it is read with its
/// real gap, open or closed, so the solve sees a stop before it reaches it.
/// The static solve bounds every step so that no declared pair's surfaces
/// approach by more than the margin, which is what keeps it from stepping
/// over a feature; the margin's size trades the rows it reads against the
/// steps it may take, and a millimeter is about twelve of the thinnest
/// papers.
pub const ROW_MARGIN: f64 = 1e-3;

/// One contact row, at one pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    /// The pair, an index into the mechanism's pairs.
    pub pair: u16,
    /// Which pieces of the two sides meet.
    pub feature: Feature,
    /// Signed gap, meters: positive apart, negative overlapping.
    pub gap: f64,
    /// Unit normal, from side B toward side A, in the world.
    pub normal: Dir,
    /// The point midway between the two surfaces on the normal, in the
    /// world.
    pub point: Point,
}

impl Row {
    /// Where side A's surface is, in the world: midway plus half the gap
    /// along the normal.
    pub fn on_a(&self) -> Point {
        self.proximity().on_a()
    }

    /// Where side B's surface is, in the world: midway less half the gap
    /// along the normal.
    pub fn on_b(&self) -> Point {
        self.proximity().on_b()
    }

    fn proximity(&self) -> Proximity {
        Proximity {
            gap: self.gap,
            normal: self.normal,
            point: self.point,
        }
    }
}

/// The rows of a mechanism at one pose, with their gradients and slip
/// gradients, one entry per coordinate each.
///
/// A buffer filled by [`Mechanism::rows`] and kept between readings: reading
/// again into it reuses what it has already allocated.
#[derive(Debug, Clone, Default)]
pub struct Rows {
    coords: usize,
    rows: Vec<Row>,
    grad: Vec<f64>,
    slip: Vec<f64>,
    faces_a: Vec<(Piece, Face)>,
    faces_b: Vec<(Piece, Face)>,
    discs_a: Vec<(Piece, Disc)>,
    discs_b: Vec<(Piece, Disc)>,
    pending: Vec<(Candidate, Proximity)>,
}

impl Rows {
    /// An empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// The rows, pair by pair in the mechanism's order, each pair's features
    /// in a fixed order.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// How many rows there are.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Row `i`'s gradient, one entry per coordinate: meters per radian on an
    /// angle, meters per meter on a length.
    ///
    /// # Panics
    ///
    /// When there is no row `i`.
    pub fn grad(&self, i: usize) -> &[f64] {
        &self.grad[i * self.coords..(i + 1) * self.coords]
    }

    /// Row `i`'s slip gradient, one entry per coordinate, in the same units.
    ///
    /// # Panics
    ///
    /// When there is no row `i`.
    pub fn slip(&self, i: usize) -> &[f64] {
        &self.slip[i * self.coords..(i + 1) * self.coords]
    }

    /// The rows of one pair, with their indices.
    pub fn of_pair(&self, pair: u16) -> impl Iterator<Item = (usize, &Row)> {
        self.rows
            .iter()
            .enumerate()
            .filter(move |(_, r)| r.pair == pair)
    }

    /// The row of one pair with the least gap, the first of them on a tie:
    /// the one that governs the pair, since a rigid contact is the least of
    /// its features' gaps. `None` when the pair has no row.
    pub fn governing(&self, pair: u16) -> Option<usize> {
        let mut best: Option<(usize, f64)> = None;
        for (i, r) in self.of_pair(pair) {
            if best.is_none_or(|(_, g)| r.gap < g) {
                best = Some((i, r.gap));
            }
        }
        best.map(|(i, _)| i)
    }

    /// The least gap of one pair, meters, or `None` when it has no row.
    pub fn gap(&self, pair: u16) -> Option<f64> {
        self.governing(pair).map(|i| self.rows[i].gap)
    }

    fn clear(&mut self, coords: usize) {
        self.coords = coords;
        self.rows.clear();
        self.grad.clear();
        self.slip.clear();
    }
}

/// A direction of a body's frame, turned into the world.
fn turned(placement: &Placement, d: Dir) -> Dir {
    if *placement == Placement::WORLD {
        return d;
    }
    let v = placement.rotate(d.vector());
    Dir::new(v.x, v.y).unwrap_or(d)
}

fn world_face(placement: &Placement, f: &Face) -> Face {
    Face {
        a: placement.world(f.a),
        b: placement.world(f.b),
        normal: turned(placement, f.normal),
    }
}

fn world_disc(placement: &Placement, d: &Disc) -> Disc {
    Disc {
        center: placement.world(d.center),
        radius: d.radius,
        from: turned(placement, d.from),
        to: turned(placement, d.to),
    }
}

/// Which side of a pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    A,
    B,
}

/// A side's drawn material, as polygons: its parts, less its cuts.
struct Material {
    solids: Vec<Vec<Point>>,
    cuts: Vec<Vec<Point>>,
}

impl Material {
    /// Whether `p` is inside a part and outside every cut.
    fn holds(&self, p: Point) -> bool {
        self.solids.iter().any(|s| inside(s, p)) && !self.cuts.iter().any(|c| inside(c, p))
    }

    /// How deep `p` stands inside the material: its distance to the nearest
    /// edge of a part or a cut; zero outside.
    fn depth(&self, p: Point) -> f64 {
        if !self.holds(p) {
            return 0.0;
        }
        let edge = |a: Point, b: Point| {
            let (ex, ey) = (b.x - a.x, b.y - a.y);
            let e2 = ex * ex + ey * ey;
            let t = if e2 > 0.0 {
                (((p.x - a.x) * ex + (p.y - a.y) * ey) / e2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (dx, dy) = (p.x - a.x - t * ex, p.y - a.y - t * ey);
            (dx * dx + dy * dy).sqrt()
        };
        self.solids
            .iter()
            .chain(&self.cuts)
            .flat_map(|poly| {
                let n = poly.len();
                (0..n).map(move |i| (poly[i], poly[(i + 1) % n]))
            })
            .map(|(a, b)| edge(a, b))
            .fold(f64::INFINITY, f64::min)
    }
}

/// Whether `p` is inside a closed polygon, by the parity of a ray's
/// crossings.
fn inside(polygon: &[Point], p: Point) -> bool {
    let mut odd = false;
    let n = polygon.len();
    for i in 0..n {
        let (a, b) = (polygon[i], polygon[(i + 1) % n]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y);
            if p.x < x {
                odd = !odd;
            }
        }
    }
    odd
}

/// What one row is being built from: its pair, its feature, the two bodies
/// its sides are carried by, and the corner, edge or center whose overlaps
/// compete, where the other piece is a face.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    pair: u16,
    feature: Feature,
    body_a: BodyId,
    body_b: BodyId,
    disc: Option<Piece>,
}

impl Mechanism {
    /// Reads every declared pair at `pose` into `out`: one row for each pair
    /// of features that meet with a gap of at most `margin`, meters, in the
    /// pairs' order and in a fixed order within each, with its gradient and
    /// slip gradient over every coordinate.
    ///
    /// A pair of [`PairGeometry::Features`] gives the rows of side A's discs
    /// against side B's faces, then of B's discs against A's faces, then of
    /// A's discs against B's discs; one of [`PairGeometry::CenterBearing`]
    /// gives the rows of B's center against A's faces. A pair whose features
    /// meet nowhere within their spans and sectors gives none.
    ///
    /// # Panics
    ///
    /// When `pose` was not placed by this mechanism.
    pub fn rows(&self, pose: &Pose, margin: f64, out: &mut Rows) {
        out.clear(self.coords().len());
        for (p, (pair, features)) in self.pairs().iter().zip(&self.features).enumerate() {
            let pair_index = u16::try_from(p).expect("fewer than 65 536 pairs");
            self.read_pair(pose, pair_index, pair.geometry, features, margin, out);
        }
    }

    fn read_pair(
        &self,
        pose: &Pose,
        pair: u16,
        geometry: PairGeometry,
        features: &PairFeatures,
        margin: f64,
        out: &mut Rows,
    ) {
        let (a, b) = (&features.a, &features.b);
        let (place_a, place_b) = (pose.placement(a.body), pose.placement(b.body));
        out.faces_a.clear();
        out.faces_a.extend(
            a.faces
                .iter()
                .map(|(piece, f)| (*piece, world_face(&place_a, f))),
        );
        let candidate = |a_piece: Piece, b_piece: Piece, disc: Option<Piece>| Candidate {
            pair,
            feature: Feature {
                a: a_piece,
                b: b_piece,
            },
            body_a: a.body,
            body_b: b.body,
            disc,
        };
        out.pending.clear();
        match geometry {
            PairGeometry::CenterBearing => {
                let (piece, center) = b.center;
                let center = place_b.world(center);
                // A center deeper than the margin behind the face's line is
                // read only where the face is the way it came in: where no
                // other side of the material stands nearer it by more than
                // the margin. A punching's center a hair past the key's front
                // edge comes in through the key front once the key turns,
                // and the line of the underside is then the long way out.
                for k in 0..out.faces_a.len() {
                    let (a_piece, face) = out.faces_a[k];
                    if let Some(x) = center_on_face(center, &face) {
                        if x.gap >= -ROW_MARGIN
                            || self.material(pose, pair, Side::A).depth(center)
                                >= -x.gap - ROW_MARGIN
                        {
                            out.pending
                                .push((candidate(a_piece, piece, Some(piece)), x));
                        }
                    }
                }
            }
            PairGeometry::Features => {
                out.faces_b.clear();
                out.faces_b.extend(
                    b.faces
                        .iter()
                        .map(|(piece, f)| (*piece, world_face(&place_b, f))),
                );
                out.discs_a.clear();
                out.discs_a.extend(
                    a.discs
                        .iter()
                        .map(|(piece, d)| (*piece, world_disc(&place_a, d))),
                );
                out.discs_b.clear();
                out.discs_b.extend(
                    b.discs
                        .iter()
                        .map(|(piece, d)| (*piece, world_disc(&place_b, d))),
                );
                for i in 0..out.discs_a.len() {
                    for k in 0..out.faces_b.len() {
                        let ((a_piece, disc), (b_piece, face)) = (out.discs_a[i], out.faces_b[k]);
                        if let Some(x) = disc_on_face(&disc, &face) {
                            if self.real(pose, pair, Side::B, x.gap, x.on_a()) {
                                out.pending
                                    .push((candidate(a_piece, b_piece, Some(a_piece)), x));
                            }
                        }
                    }
                }
                for i in 0..out.discs_b.len() {
                    for k in 0..out.faces_a.len() {
                        let ((b_piece, disc), (a_piece, face)) = (out.discs_b[i], out.faces_a[k]);
                        if let Some(x) = disc_on_face(&disc, &face) {
                            let x = x.reversed();
                            if self.real(pose, pair, Side::A, x.gap, x.on_b()) {
                                out.pending
                                    .push((candidate(a_piece, b_piece, Some(b_piece)), x));
                            }
                        }
                    }
                }
                for i in 0..out.discs_a.len() {
                    for k in 0..out.discs_b.len() {
                        let ((a_piece, da), (b_piece, db)) = (out.discs_a[i], out.discs_b[k]);
                        if let Some(x) = disc_on_disc(&da, &db) {
                            out.pending.push((candidate(a_piece, b_piece, None), x));
                        }
                    }
                }
            }
        }
        // A corner sunk deeper than the margin into the material round two
        // faces reads both, and only the nearer is where it comes out: of a
        // corner's overlaps, those deeper than the margin are kept only if
        // none is shallower.
        let pending = core::mem::take(&mut out.pending);
        for (i, &(c, x)) in pending.iter().enumerate() {
            let shallower = c.disc.is_some_and(|disc| {
                x.gap < -ROW_MARGIN
                    && pending.iter().enumerate().any(|(j, (d, y))| {
                        d.disc == Some(disc)
                            && y.gap < 0.0
                            && match y.gap.partial_cmp(&x.gap) {
                                Some(core::cmp::Ordering::Greater) => true,
                                Some(core::cmp::Ordering::Equal) => j < i,
                                _ => false,
                            }
                    })
            });
            if !shallower {
                self.push(pose, c, x, margin, out);
            }
        }
        out.pending = pending;
    }

    /// Whether a row of pair `pair` against a face of its side `side` is an
    /// overlap the drawn parts hold: always, but for a row deeper than the
    /// row margin, whose point `p` on the other side must then stand inside
    /// `side`'s material, the parts its ranges belong to, or every part of
    /// its body for a side that is a cut's range, less the body's cuts but
    /// for a pair read at its center.
    ///
    /// A face's row reads the distance to the face's line, which is the
    /// depth of an overlap only near the face: a point deeper than the margin
    /// may have passed out through the material's far side, as a pin standing
    /// clear in front of a shortened key stands far past the line of its
    /// mortise's wall. No step of a static solve sinks a point deeper than
    /// the margin, so a deep row is a start's, and only a real overlap is
    /// one to resolve.
    fn real(&self, pose: &Pose, pair: u16, side: Side, gap: f64, p: Point) -> bool {
        gap >= -ROW_MARGIN || self.material(pose, pair, side).holds(p)
    }

    /// How deep the drawn parts of the declared pairs stand in one another at
    /// `pose`, meters: the deepest that a point of the parts one side's
    /// ranges belong to, sampled along their outlines, stands inside the
    /// other side's drawn material; zero where nothing overlaps.
    ///
    /// Read off the outlines, not the rows: a part sunk far into another,
    /// past an edge's center or across a corner's sector, may read no row at
    /// all (`docs/FOUNDATIONS.md` > Seventeen contacts per note), and this is what a solve that must
    /// start somewhere uses to start where nothing is sunk.
    pub fn overlap(&self, pose: &Pose) -> f64 {
        (0..self.pairs().len())
            .map(|p| self.pair_overlap(pose, u16::try_from(p).expect("fewer than 65 536 pairs")))
            .fold(0.0, f64::max)
    }

    /// How deep the two sides of pair `pair` stand in one another at `pose`,
    /// meters, as [`Self::overlap`] reads it. A pair read at its center
    /// counts its center and nothing else of its face: the face's edges
    /// overlapping a tilted key is the idealization it declares.
    pub fn pair_overlap(&self, pose: &Pose, pair: u16) -> f64 {
        let parts = self.parts();
        let declared = &self.pairs()[usize::from(pair)];
        let mut deepest = 0.0_f64;
        if declared.geometry == PairGeometry::CenterBearing {
            let sides = &self.features[usize::from(pair)];
            let center = pose.placement(sides.b.body).world(sides.b.center.1);
            return self.material(pose, pair, Side::A).depth(center);
        }
        for (from, into) in [(Side::A, Side::B), (Side::B, Side::A)] {
            let surface = match from {
                Side::A => &declared.a,
                Side::B => &declared.b,
            };
            // The whole outline of every part the side's ranges belong to,
            // not only the ranges: a pin standing up through a key, its top
            // clear above the key's, is sunk in the wood by its sides.
            let owners: Vec<usize> = surface
                .ranges
                .iter()
                .map(|r| r.part.index())
                .filter(|&i| parts[i].role != super::Role::Cut)
                .collect();
            deepest = deepest.max(self.sunk(pose, &owners, &self.material(pose, pair, into)));
        }
        deepest
    }

    /// How deep the parts `a` and the parts `b` stand in one another at
    /// `pose`, meters: what [`Self::pair_overlap`] reads of a pair's sides,
    /// read of parts no pair declares, each set's material its parts less
    /// the cuts of the bodies that carry them.
    pub fn parts_overlap(&self, pose: &Pose, a: &[PartId], b: &[PartId]) -> f64 {
        let index = |ids: &[PartId]| -> Vec<usize> {
            ids.iter()
                .map(|id| id.index())
                .filter(|&i| self.parts()[i].role != super::Role::Cut)
                .collect()
        };
        let (a, b) = (index(a), index(b));
        let into_b = self.sunk(pose, &a, &self.material_of(pose, &b));
        let into_a = self.sunk(pose, &b, &self.material_of(pose, &a));
        into_b.max(into_a)
    }

    /// The deepest that a point sampled along the outline of one of the
    /// parts `owners` stands inside `material` at `pose`, meters: seventeen
    /// points along each line, an arc as it is drawn, and none along a
    /// segment that is a cut's mouth, the underside running on straight
    /// across a mortise, which bounds no material.
    fn sunk(&self, pose: &Pose, owners: &[usize], material: &Material) -> f64 {
        let parts = self.parts();
        let mut owners = owners.to_vec();
        owners.sort_unstable();
        owners.dedup();
        let same =
            |p: Point, q: Point| p.x.to_bits() == q.x.to_bits() && p.y.to_bits() == q.y.to_bits();
        let mouths: Vec<(Point, Point)> = parts
            .iter()
            .filter(|c| c.role == super::Role::Cut)
            .flat_map(|c| c.silhouette().into_iter().map(|g| (g.start(), g.end())))
            .collect();
        let mut deepest = 0.0_f64;
        for i in owners {
            let placement = pose.placement(parts[i].body);
            for seg in parts[i].silhouette() {
                let (a, b) = (seg.start(), seg.end());
                if mouths
                    .iter()
                    .any(|&(c, d)| (same(a, c) && same(b, d)) || (same(a, d) && same(b, c)))
                {
                    continue;
                }
                let points: Vec<Point> = match seg {
                    Seg::Line { a, b } => (0..=16)
                        .map(|k| {
                            let t = f64::from(k) / 16.0;
                            Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y))
                        })
                        .collect(),
                    Seg::Arc { .. } => seg.drawn(3),
                };
                for q in points {
                    deepest = deepest.max(material.depth(placement.world(q)));
                }
            }
        }
        deepest
    }

    /// The drawn material of the parts `owners` at `pose`: their outlines,
    /// less the cuts of every body that carries one of them.
    fn material_of(&self, pose: &Pose, owners: &[usize]) -> Material {
        let parts = self.parts();
        let polygon = |i: usize| -> Vec<Point> {
            let placement = pose.placement(parts[i].body);
            parts[i]
                .silhouette()
                .iter()
                .flat_map(|seg| seg.drawn(3))
                .map(|q| placement.world(q))
                .collect()
        };
        let bodies: Vec<BodyId> = owners.iter().map(|&i| parts[i].body).collect();
        Material {
            solids: owners.iter().map(|&i| polygon(i)).collect(),
            cuts: (0..parts.len())
                .filter(|&i| parts[i].role == super::Role::Cut && bodies.contains(&parts[i].body))
                .map(polygon)
                .collect(),
        }
    }

    /// The drawn material of side `side` of pair `pair` at `pose`: the parts
    /// its ranges belong to, or every part of its body for a side that is a
    /// cut's range, less the body's cuts but for a pair read at its center.
    fn material(&self, pose: &Pose, pair: u16, side: Side) -> Material {
        let declared = &self.pairs()[usize::from(pair)];
        let surface = match side {
            Side::A => &declared.a,
            Side::B => &declared.b,
        };
        let parts = self.parts();
        let body = parts[surface.ranges[0].part.index()].body;
        let placement = pose.placement(body);
        let own: Vec<usize> = surface
            .ranges
            .iter()
            .map(|r| r.part.index())
            .filter(|&i| parts[i].role != super::Role::Cut)
            .collect();
        let polygon = |i: usize| -> Vec<Point> {
            parts[i]
                .silhouette()
                .iter()
                .flat_map(|seg| seg.drawn(3))
                .map(|q| placement.world(q))
                .collect()
        };
        let centered = declared.geometry == PairGeometry::CenterBearing;
        let mut material = Material {
            solids: Vec::new(),
            cuts: Vec::new(),
        };
        for (i, part) in parts.iter().enumerate() {
            if part.body != body {
                continue;
            }
            match part.role {
                super::Role::Cut => {
                    if !centered {
                        material.cuts.push(polygon(i));
                    }
                }
                _ => {
                    if own.is_empty() || own.contains(&i) {
                        material.solids.push(polygon(i));
                    }
                }
            }
        }
        material
    }

    /// Adds the row, with its gradient and slip gradient, if its gap is
    /// within the margin. A gap that is not a number is within nothing.
    fn push(&self, pose: &Pose, candidate: Candidate, x: Proximity, margin: f64, out: &mut Rows) {
        if x.gap.is_nan() || x.gap > margin {
            return;
        }
        out.rows.push(Row {
            pair: candidate.pair,
            feature: candidate.feature,
            gap: x.gap,
            normal: x.normal,
            point: x.point,
        });
        let n = x.normal.vector();
        let t = Point::new(-n.y, n.x);
        let start = out.grad.len();
        out.grad.resize(start + out.coords, 0.0);
        out.slip.resize(start + out.coords, 0.0);
        let (grad, slip) = (&mut out.grad[start..], &mut out.slip[start..]);
        self.for_each_column_at(pose, candidate.body_a, x.point, |c, j| {
            grad[c.0] += n.x * j.x + n.y * j.y;
            slip[c.0] += t.x * j.x + t.y * j.y;
        });
        self.for_each_column_at(pose, candidate.body_b, x.point, |c, j| {
            grad[c.0] -= n.x * j.x + n.y * j.y;
            slip[c.0] -= t.x * j.x + t.y * j.y;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures;
    use super::super::{Role, Seg};
    use super::*;
    use crate::mechanism::fixtures::{random_q, reach, XorShift};
    use crate::mechanism::{CoordId, CoordKind, MaterialPoint};

    /// Where the world point `p` is in `body`'s frame, at `pose`.
    fn local(pose: &Pose, body: BodyId, p: Point) -> Point {
        let pl = pose.placement(body);
        let (dx, dy) = (p.x - pl.origin.x, p.y - pl.origin.y);
        Point::new(pl.cos * dx + pl.sin * dy, -pl.sin * dx + pl.cos * dy)
    }

    fn part_of(piece: Piece) -> crate::contact::PartId {
        match piece {
            Piece::Segment { part, .. }
            | Piece::Arc { part, .. }
            | Piece::Vertex { part, .. }
            | Piece::Center { part } => part,
        }
    }

    /// Whether `coord` moves `body`: it is the joint of the body or of one of
    /// its ancestors.
    fn moves(m: &Mechanism, body: BodyId, coord: CoordId) -> bool {
        let mut b = body;
        while b != BodyId::GROUND {
            let joint = m.body(b).joint;
            if joint.coord() == Some(coord) {
                return true;
            }
            b = m.body(b).parent;
        }
        false
    }

    /// A piece as drawn, in the world, written again here from the part's
    /// silhouette: the segment, or the disc's center, radius and the
    /// directions its sweep runs between.
    enum Drawn {
        Line {
            a: Point,
            b: Point,
            normal: Point,
        },
        Disc {
            c: Point,
            r: f64,
            from: Point,
            to: Point,
        },
        Center(Point),
    }

    fn drawn(
        m: &Mechanism,
        pose: &Pose,
        piece: Piece,
        pair: &crate::contact::ContactPair,
    ) -> Drawn {
        let part = &m.parts()[part_of(piece).index()];
        let sil = part.silhouette();
        let pl = pose.placement(part.body);
        let turn = |v: Point| pl.rotate(v);
        match piece {
            Piece::Segment { seg, .. } => {
                let Seg::Line { a, b } = sil[usize::from(seg)] else {
                    panic!("a segment piece is a line");
                };
                let (wa, wb) = (pl.world(a), pl.world(b));
                let (dx, dy) = (wb.x - wa.x, wb.y - wa.y);
                let len = (dx * dx + dy * dy).sqrt();
                let right = Point::new(dy / len, -dx / len);
                let normal = if part.role == Role::Cut {
                    Point::new(-right.x, -right.y)
                } else {
                    right
                };
                Drawn::Line {
                    a: wa,
                    b: wb,
                    normal,
                }
            }
            Piece::Arc { seg, .. } => {
                let Seg::Arc { c, r, from, to } = sil[usize::from(seg)] else {
                    panic!("an arc piece is an arc");
                };
                Drawn::Disc {
                    c: pl.world(c),
                    r,
                    from: turn(from.vector()),
                    to: turn(to.vector()),
                }
            }
            Piece::Vertex { seg, .. } => {
                let n = sil.len();
                let i = usize::from(seg);
                let (previous, next) = (sil[(i + n - 1) % n], sil[i]);
                let normal_out = |s: Seg, at_end: bool| match s {
                    Seg::Line { a, b } => {
                        let (dx, dy) = (b.x - a.x, b.y - a.y);
                        let len = (dx * dx + dy * dy).sqrt();
                        Point::new(dy / len, -dx / len)
                    }
                    Seg::Arc { from, to, .. } => {
                        if at_end {
                            to.vector()
                        } else {
                            from.vector()
                        }
                    }
                };
                Drawn::Disc {
                    c: pl.world(next.start()),
                    r: 0.0,
                    from: turn(normal_out(previous, true)),
                    to: turn(normal_out(next, false)),
                }
            }
            Piece::Center { .. } => {
                let range = &pair.b.ranges[0];
                let first = sil[usize::from(range.segs.start)].start();
                let last = sil[usize::from(range.segs.end) - 1].end();
                Drawn::Center(pl.world(Point::new(
                    0.5 * (first.x + last.x),
                    0.5 * (first.y + last.y),
                )))
            }
        }
    }

    fn dot(u: Point, v: Point) -> f64 {
        u.x * v.x + u.y * v.y
    }

    fn sub(u: Point, v: Point) -> Point {
        Point::new(u.x - v.x, u.y - v.y)
    }

    fn norm(u: Point) -> f64 {
        dot(u, u).sqrt()
    }

    /// How far `p` stands from the drawn piece: from the segment, the circle,
    /// the vertex or the center. Within its span or sweep it is the distance
    /// to the line or the circle; a vertex and a center are points. A vertex
    /// read with a neighbor from another range is drawn here with its own
    /// outline's, which a chain joined straight never reads.
    fn off(p: Point, piece: &Drawn) -> f64 {
        match *piece {
            Drawn::Line { a, b, normal } => {
                let e = sub(b, a);
                let t = dot(sub(p, a), e) / dot(e, e);
                assert!(
                    (-1e-9..=1.0 + 1e-9).contains(&t),
                    "the point projects off its segment, at {t}"
                );
                dot(sub(p, a), normal).abs()
            }
            Drawn::Disc { c, r, from, to } => {
                // Within the sweep: turned from `from` no further than `to`
                // is, counter-clockwise, by the angles `libm` gives.
                let u = sub(p, c);
                if r > 0.0 && norm(u) > 0.0 {
                    let turn = |v: Point| {
                        let a = libm::atan2(from.x * v.y - from.y * v.x, dot(from, v));
                        if a < 0.0 {
                            a + 2.0 * core::f64::consts::PI
                        } else {
                            a
                        }
                    };
                    assert!(
                        turn(u) <= turn(to) + 1e-9 || turn(u) >= 2.0 * core::f64::consts::PI - 1e-9,
                        "the point is off its arc's sweep"
                    );
                }
                (norm(u) - r).abs()
            }
            Drawn::Center(c) => norm(sub(p, c)),
        }
    }

    /// C2, consistency: **every row's gradient and slip gradient against
    /// central differences**, at seeded random poses of the mechanism tests'
    /// chain carrying parts of every kind: lines, arcs, convex and concave
    /// vertices, a cut, a turned part, a side chained across two ranges; pairs
    /// between a body and its descendant, two branches, a body and the
    /// ground. Its random poses put parts deep into one another, deeper than
    /// any radius. With it, at every row: the normal unit; the contact point
    /// midway, each surface's point on its drawn piece; the gap signed at
    /// every depth, by an independent side test against the drawn piece; and
    /// a coordinate that moves both bodies canceled exactly.
    ///
    /// The bounds are derived. A coordinate that moves one side turns its
    /// points rigidly about a pivot within the reach `L` (the two contact
    /// points' chains, the gap and the largest radius added), and a gap
    /// against a line is then a sinusoid in the angle of amplitude at most
    /// `L`: a central difference over `±h` is off by at most `h²L/6`. Between
    /// two discs the gap is the distance between their centers less the
    /// radii, whose third derivative is at most `6ρ³/f² + 6ρ²/f + ρ`, `f` the
    /// least distance between the centers over the step and `ρ ≤ L`. On a
    /// slide everything moves linearly and the difference is exact. Each gap
    /// is within `256 ε L` of its exact value, positions carrying the `64 ε L`
    /// of the chain's placements (C1), so the difference adds `256 ε L / h`,
    /// and the gradient's own rounding `128 ε L`. The slip's difference is of
    /// the two material points' separation along the tangent, two sinusoids
    /// of amplitude `L`.
    #[test]
    fn every_row_gradient_and_slip_match_central_differences() {
        let m = fixtures::mechanism();
        let mut rng = XorShift::new(0x00c2_c2c2);
        let h = 1e-5;
        let eps = f64::EPSILON;
        let (mut checked, mut deep, mut canceled) = (0usize, 0usize, 0usize);
        let mut pairs_seen = vec![false; m.pairs().len()];
        let coords = m.coords().len();
        let mut rows = Rows::new();
        let (mut ups, mut downs) = (vec![Rows::new(); coords], vec![Rows::new(); coords]);
        for _ in 0..2000 {
            let q = random_q(&m, &mut rng);
            let pose = m.pose(&q);
            m.rows(&pose, f64::INFINITY, &mut rows);
            let mut shifted = Vec::with_capacity(coords);
            for c in 0..coords {
                let (mut up, mut down) = (q.clone(), q.clone());
                up[c] += h;
                down[c] -= h;
                let (pose_up, pose_down) = (m.pose(&up), m.pose(&down));
                m.rows(&pose_up, f64::INFINITY, &mut ups[c]);
                m.rows(&pose_down, f64::INFINITY, &mut downs[c]);
                shifted.push((up[c] - down[c], pose_up, pose_down));
            }
            for (i, row) in rows.rows().iter().enumerate() {
                let pair = &m.pairs()[usize::from(row.pair)];
                pairs_seen[usize::from(row.pair)] = true;
                let body_a = m.parts()[part_of(row.feature.a).index()].body;
                let body_b = m.parts()[part_of(row.feature.b).index()].body;
                let (at_a, at_b) = (
                    local(&pose, body_a, row.point),
                    local(&pose, body_b, row.point),
                );
                let radius = |piece: Piece| match drawn(&m, &pose, piece, pair) {
                    Drawn::Disc { r, .. } => r,
                    _ => 0.0,
                };
                let (r_a, r_b) = (radius(row.feature.a), radius(row.feature.b));
                let reach_l = reach(
                    &m,
                    &q,
                    MaterialPoint {
                        body: body_a,
                        at: at_a,
                    },
                ) + reach(
                    &m,
                    &q,
                    MaterialPoint {
                        body: body_b,
                        at: at_b,
                    },
                ) + row.gap.abs()
                    + 0.015;

                let n = row.normal.vector();
                assert!(
                    (dot(n, n) - 1.0).abs() <= 4.0 * eps,
                    "row {row:?}: the normal is not unit"
                );

                // Each surface's point on its drawn piece, midway on the normal.
                let tol = 256.0 * eps * reach_l;
                let piece_a = drawn(&m, &pose, row.feature.a, pair);
                let piece_b = drawn(&m, &pose, row.feature.b, pair);
                assert!(
                    off(row.on_a(), &piece_a) <= tol,
                    "{row:?}: A's point is off its piece"
                );
                assert!(
                    off(row.on_b(), &piece_b) <= tol,
                    "{row:?}: B's point is off its piece"
                );

                // Signed at every depth, by the side of the drawn piece.
                let side = match (&piece_a, &piece_b) {
                    (Drawn::Disc { c: ca, r: ra, .. }, Drawn::Disc { c: cb, r: rb, .. }) => {
                        norm(sub(*ca, *cb)) - ra - rb
                    }
                    (_, Drawn::Line { a, normal, .. }) => dot(sub(row.on_a(), *a), *normal),
                    (Drawn::Line { a, normal, .. }, _) => dot(sub(row.on_b(), *a), *normal),
                    _ => panic!("no row pairs two centers"),
                };
                if row.gap.abs() > tol {
                    assert_eq!(
                        row.gap < 0.0,
                        side < 0.0,
                        "{row:?}: the gap's sign is not the side of the drawn piece, {side}"
                    );
                }
                if row.gap < -2.0 * 0.015 {
                    deep += 1;
                }

                for (c, coord) in m.coords().iter().enumerate() {
                    let id = CoordId(c);
                    let (g, s) = (rows.grad(i)[c], rows.slip(i)[c]);
                    if moves(&m, body_a, id) && moves(&m, body_b, id) {
                        assert_eq!(
                            (g.to_bits(), s.to_bits()),
                            (0, 0),
                            "{row:?}: coordinate {c} moves both sides and did not cancel"
                        );
                        canceled += 1;
                    }
                    let (step, pose_up, pose_down) = &shifted[c];
                    let step = *step;
                    let same = |r: &&Row| r.pair == row.pair && r.feature == row.feature;
                    let (Some(plus), Some(minus)) = (
                        ups[c].rows().iter().find(same),
                        downs[c].rows().iter().find(same),
                    ) else {
                        continue;
                    };
                    let half = 0.5 * step;
                    let truncation = match coord.kind {
                        CoordKind::Length => 0.0,
                        CoordKind::Angle => {
                            let third = if row.feature.a.is_disc() && row.feature.b.is_disc() {
                                let f = row.gap + r_a + r_b - reach_l * half;
                                if f <= 0.0 {
                                    continue;
                                }
                                let rho = reach_l;
                                6.0 * rho * rho * rho / (f * f) + 6.0 * rho * rho / f + rho
                            } else {
                                reach_l
                            };
                            half * half / 6.0 * third
                        }
                    };
                    let bound = truncation + 256.0 * eps * reach_l / half + 128.0 * eps * reach_l;
                    let fd = (plus.gap - minus.gap) / step;
                    assert!(
                        (fd - g).abs() <= bound,
                        "{row:?}, coordinate {c} at {q:?}: gradient {g}, central difference \
                         {fd}, off by {:e} > {bound:e}",
                        (fd - g).abs()
                    );

                    // The slip: the two material points at the contact point,
                    // their separation along the tangent.
                    let t = Point::new(-n.y, n.x);
                    let separation = |p: &Pose| {
                        let a = p.world(MaterialPoint {
                            body: body_a,
                            at: at_a,
                        });
                        let b = p.world(MaterialPoint {
                            body: body_b,
                            at: at_b,
                        });
                        dot(sub(a, b), t)
                    };
                    let slip_fd = (separation(pose_up) - separation(pose_down)) / step;
                    let slip_truncation = match coord.kind {
                        CoordKind::Length => 0.0,
                        CoordKind::Angle => half * half / 6.0 * 2.0 * reach_l,
                    };
                    let slip_bound =
                        slip_truncation + 256.0 * eps * reach_l / half + 128.0 * eps * reach_l;
                    assert!(
                        (slip_fd - s).abs() <= slip_bound,
                        "{row:?}, coordinate {c}: slip {s}, central difference {slip_fd}, off \
                         by {:e} > {slip_bound:e}",
                        (slip_fd - s).abs()
                    );
                    checked += 1;
                }
            }
        }
        assert!(
            pairs_seen.iter().all(|&s| s),
            "every pair gave rows somewhere: {pairs_seen:?}"
        );
        assert!(checked > 10_000, "only {checked} entries were checked");
        assert!(deep > 1_000, "only {deep} rows were deeper than any radius");
        assert!(canceled > 2_000, "only {canceled} entries canceled");
    }

    /// Rows are generated within the margin and no further: the rows within
    /// a millimeter are exactly the rows of every margin whose gap is at
    /// most a millimeter, in the same order, with the same gradients.
    #[test]
    fn a_row_is_generated_within_the_margin_and_no_further() {
        let m = fixtures::mechanism();
        let mut rng = XorShift::new(0x0000_3a26);
        let (mut all, mut near) = (Rows::new(), Rows::new());
        let mut kept = 0;
        for _ in 0..200 {
            let pose = m.pose(&random_q(&m, &mut rng));
            m.rows(&pose, f64::INFINITY, &mut all);
            m.rows(&pose, ROW_MARGIN, &mut near);
            let expected: Vec<usize> = (0..all.len())
                .filter(|&i| all.rows()[i].gap <= ROW_MARGIN)
                .collect();
            assert_eq!(expected.len(), near.len());
            for (k, &i) in expected.iter().enumerate() {
                assert_eq!(all.rows()[i], near.rows()[k]);
                assert_eq!(all.grad(i), near.grad(k));
                assert_eq!(all.slip(i), near.slip(k));
            }
            kept += near.len();
        }
        assert!(kept > 0, "some rows fell within the margin");
    }
}

/// An axis-aligned box in the world: its least and greatest corner.
#[derive(Debug, Clone, Copy)]
struct Bounds {
    lo: Point,
    hi: Point,
}

impl Bounds {
    fn of(points: &[Point], radius: f64) -> Self {
        let mut lo = Point::new(f64::INFINITY, f64::INFINITY);
        let mut hi = Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
        for p in points {
            lo = Point::new(lo.x.min(p.x - radius), lo.y.min(p.y - radius));
            hi = Point::new(hi.x.max(p.x + radius), hi.y.max(p.y + radius));
        }
        Self { lo, hi }
    }

    /// How far apart two boxes stand: zero where they overlap.
    fn apart(&self, other: &Self) -> f64 {
        let dx = (other.lo.x - self.hi.x)
            .max(self.lo.x - other.hi.x)
            .max(0.0);
        let dy = (other.lo.y - self.hi.y)
            .max(self.lo.y - other.hi.y)
            .max(0.0);
        (dx * dx + dy * dy).sqrt()
    }
}

impl Mechanism {
    /// A lower bound on how far apart the two sides of pair `pair` stand at
    /// `pose`, meters: the least distance between a box round a piece of one
    /// side and a box round a piece of the other, each box holding its
    /// segment, or its arc's whole circle. Zero where two boxes overlap.
    ///
    /// What a static solve bounds its steps by for a pair it reads no row
    /// of at any distance, where no gap says how far off the pair stands.
    pub(crate) fn pair_bound(&self, pose: &Pose, pair: u16) -> f64 {
        let sides = &self.features[usize::from(pair)];
        let boxes = |side: &super::features::SideFeatures| -> Vec<Bounds> {
            let placement = pose.placement(side.body);
            let mut out: Vec<Bounds> = side
                .faces
                .iter()
                .map(|(_, f)| Bounds::of(&[placement.world(f.a), placement.world(f.b)], 0.0))
                .collect();
            out.extend(
                side.discs
                    .iter()
                    .map(|(_, d)| Bounds::of(&[placement.world(d.center)], d.radius)),
            );
            out
        };
        let (a, b) = (boxes(&sides.a), boxes(&sides.b));
        let mut least = f64::INFINITY;
        for x in &a {
            for y in &b {
                least = least.min(x.apart(y));
            }
        }
        least
    }
}
