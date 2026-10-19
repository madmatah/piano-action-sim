//! The features of a pair's two sides, read once from the drawn outlines in
//! their bodies' frames.

use super::{ContactPair, PairGeometry, Part, PartId, Role, Surface, SurfaceRef, Sweep};
use crate::geometry::{Dir, Disc, Face, Point, Seg};
use crate::mechanism::BodyId;

/// A piece of a surface that a row reads.
///
/// Every piece is drawn: a segment of a part's silhouette, the vertex where
/// one starts, or the center of a drawn face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    /// A straight segment, read as a face.
    Segment {
        /// The part.
        part: PartId,
        /// The segment, an index into the part's silhouette.
        seg: u16,
    },
    /// An arc, read as a disc whose sector is its sweep.
    Arc {
        /// The part.
        part: PartId,
        /// The segment, an index into the part's silhouette.
        seg: u16,
    },
    /// The convex vertex where a segment starts, read as a disc of radius
    /// zero whose sector is the vertex's exterior angle.
    Vertex {
        /// The part.
        part: PartId,
        /// The segment that starts at the vertex, an index into the part's
        /// silhouette.
        seg: u16,
    },
    /// The center of a drawn face: where a turned part's face crosses its
    /// axis, or the middle of the face's chord.
    Center {
        /// The part.
        part: PartId,
    },
}

impl Piece {
    /// Whether the piece is read as a disc: an arc or a vertex.
    pub fn is_disc(self) -> bool {
        matches!(self, Self::Arc { .. } | Self::Vertex { .. })
    }
}

/// What a row pairs: a piece of side A against a piece of side B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Feature {
    /// Side A's piece.
    pub a: Piece,
    /// Side B's piece.
    pub b: Piece,
}

/// One side of a pair, read into its body's frame.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SideFeatures {
    /// The body carrying it.
    pub body: BodyId,
    /// Its straight segments, presenting their outward side.
    pub faces: Vec<(Piece, Face)>,
    /// Its arcs and convex vertices.
    pub discs: Vec<(Piece, Disc)>,
    /// The center of its drawn face.
    pub center: (Piece, Point),
}

/// A pair's two sides, read once when the mechanism is built.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PairFeatures {
    /// Side A.
    pub a: SideFeatures,
    /// Side B.
    pub b: SideFeatures,
}

impl PairFeatures {
    /// Reads both sides of `pair` from `parts`, whose ranges the mechanism
    /// has already checked are drawn.
    pub fn of(pair: &ContactPair, parts: &[Part]) -> Self {
        let center_of_b = matches!(pair.geometry, PairGeometry::CenterBearing);
        Self {
            a: side(&pair.a, parts, false),
            b: side(&pair.b, parts, center_of_b),
        }
    }
}

/// The outward normal at the start and at the end of a segment of a
/// material part's outline, which runs counter-clockwise: a line's
/// right-hand normal, an arc's radial direction. `None` for a line of no
/// length.
fn ends_normals(seg: Seg) -> Option<(Dir, Dir)> {
    match seg {
        Seg::Line { a, b } => {
            let d = Dir::new(b.x - a.x, b.y - a.y)?.vector();
            let n = Dir::new(d.y, -d.x)?;
            Some((n, n))
        }
        Seg::Arc { from, to, .. } => Some((from, to)),
    }
}

/// The face a straight segment presents, running with material on its left:
/// its right-hand normal, out of the material.
fn face(seg: Seg) -> Option<Face> {
    let Seg::Line { a, b } = seg else {
        return None;
    };
    let d = Dir::new(b.x - a.x, b.y - a.y)?.vector();
    let normal = Dir::new(d.y, -d.x)?;
    Some(Face { a, b, normal })
}

/// The vertex where `next` starts after `previous` ends, as a disc of radius
/// zero, if it is convex: the outline turns left there, from the outward
/// normal at `previous`'s end to the one at `next`'s start. A vertex where
/// the two run straight on, or turn right, is no feature.
fn vertex(previous: Seg, next: Seg) -> Option<Disc> {
    let (_, from) = ends_normals(previous)?;
    let (to, _) = ends_normals(next)?;
    let (f, t) = (from.vector(), to.vector());
    let turn = f.x * t.y - f.y * t.x;
    let back = f.x * t.x + f.y * t.y;
    if turn > 0.0 || (turn == 0.0 && back < 0.0) {
        Some(Disc {
            center: next.start(),
            radius: 0.0,
            from,
            to,
        })
    } else {
        None
    }
}

fn same(p: Point, q: Point) -> bool {
    p.x.to_bits() == q.x.to_bits() && p.y.to_bits() == q.y.to_bits()
}

/// A segment run the other way. An arc cannot be: it always runs
/// counter-clockwise, so a cut's arc, concave from the wood, has no
/// reversed reading, and `None` stands for it.
fn run_back(seg: Seg) -> Option<Seg> {
    match seg {
        Seg::Line { a, b } => Some(Seg::Line { a: b, b: a }),
        Seg::Arc { .. } => None,
    }
}

/// One range, as the material it bounds runs: each segment with the
/// material on its left, and the index of the segment of the part's
/// silhouette it is.
///
/// A material part's outline already runs so, counter-clockwise. A cut's
/// runs counter-clockwise round its void, which is clockwise round the wood
/// it is cut from: its range is read backwards, each segment reversed, so
/// that it continues the wood's own outline where the cut opens onto it, as
/// the walls of a mortise open onto the underside at its mouth. Also the
/// segment before the range and the one after it, so read, in the part's own
/// outline. `None` for a segment that has no such reading: a cut's arc.
struct Traversal {
    segs: Vec<(u16, Option<Seg>)>,
    before: Option<Seg>,
    after: Option<Seg>,
    /// The part's silhouette index of the segment that starts at each
    /// segment's start, as the part draws it: what names the vertex there.
    starts: Vec<u16>,
}

fn traverse(range: &SurfaceRef, sil: &[Seg], role: Role) -> Traversal {
    let n = sil.len();
    let (start, end) = (usize::from(range.segs.start), usize::from(range.segs.end));
    let index = |i: usize| u16::try_from(i % n).expect("a silhouette's index");
    if role == Role::Cut {
        let segs: Vec<(u16, Option<Seg>)> = (start..end)
            .rev()
            .map(|i| (index(i), run_back(sil[i])))
            .collect();
        // Run backwards, a segment starts where it ends as drawn, which is
        // where the next one drawn starts.
        let starts = (start..end).rev().map(|i| index(i + 1)).collect();
        Traversal {
            segs,
            before: run_back(sil[end % n]),
            after: run_back(sil[(start + n - 1) % n]),
            starts,
        }
    } else {
        Traversal {
            segs: (start..end).map(|i| (index(i), Some(sil[i]))).collect(),
            before: Some(sil[(start + n - 1) % n]),
            after: Some(sil[end % n]),
            starts: (start..end).map(index).collect(),
        }
    }
}

/// Reads one side: its faces, its arcs and its convex vertices, in the order
/// the ranges run; and the center of its drawn face.
///
/// Every range is read as the material it bounds runs ([`traverse`]), so a
/// face presents the side away from the material, into the void of a cut
/// (the top of a mortise faces down, onto its pin), and a vertex is a
/// feature where that material's outline turns convex. A cut's own corners
/// and arcs are concave from the wood and never are; where a cut's range
/// continues a range of the wood, as a mortise's wall continues the
/// underside at the slot's mouth, the corner between them is the wood's, and
/// convex.
fn side(surface: &Surface, parts: &[Part], center_wanted: bool) -> SideFeatures {
    let silhouettes: Vec<Vec<Seg>> = surface
        .ranges
        .iter()
        .map(|r| parts[r.part.index()].silhouette())
        .collect();
    let traversals: Vec<Traversal> = surface
        .ranges
        .iter()
        .enumerate()
        .map(|(k, r)| traverse(r, &silhouettes[k], parts[r.part.index()].role))
        .collect();
    let first_start = |t: &Traversal| t.segs.first().and_then(|(_, s)| s.map(|s| s.start()));
    let last_end = |t: &Traversal| t.segs.last().and_then(|(_, s)| s.map(|s| s.end()));
    let last_seg = |t: &Traversal| t.segs.last().and_then(|&(_, s)| s);
    let body = parts[surface.ranges[0].part.index()].body;
    let mut faces = Vec::new();
    let mut discs = Vec::new();
    for (k, range) in surface.ranges.iter().enumerate() {
        let part = &parts[range.part.index()];
        let t = &traversals[k];
        let material = part.role != Role::Cut;
        // The range just before, when it ends where this one starts.
        let joined = k > 0
            && match (last_end(&traversals[k - 1]), first_start(t)) {
                (Some(p), Some(q)) => same(p, q),
                _ => false,
            };
        for (idx, &(s, seg)) in t.segs.iter().enumerate() {
            let Some(seg) = seg else { continue };
            if let Some(f) = face(seg) {
                faces.push((
                    Piece::Segment {
                        part: range.part,
                        seg: s,
                    },
                    f,
                ));
            }
            if let (true, Seg::Arc { c, r, from, to }) = (material, seg) {
                discs.push((
                    Piece::Arc {
                        part: range.part,
                        seg: s,
                    },
                    Disc {
                        center: c,
                        radius: r,
                        from,
                        to,
                    },
                ));
            }
            // The vertex this segment starts at: after the segment before it
            // in this range, or after the last of the range before when the
            // two are one chain, or else after its own outline's previous.
            let previous = if idx > 0 {
                t.segs[idx - 1].1
            } else if joined {
                last_seg(&traversals[k - 1])
            } else {
                t.before
            };
            if let Some(d) = previous.and_then(|p| vertex(p, seg)) {
                discs.push((
                    Piece::Vertex {
                        part: range.part,
                        seg: t.starts[idx],
                    },
                    d,
                ));
            }
        }
        // The vertex the range ends at, unless the next range starts there
        // and reads it as its own, or the range runs round the whole outline
        // and has read it as the start of its first segment.
        let continued =
            traversals
                .get(k + 1)
                .is_some_and(|next| match (last_end(t), first_start(next)) {
                    (Some(p), Some(q)) => same(p, q),
                    _ => false,
                });
        let wrapped = usize::from(range.segs.end - range.segs.start) == silhouettes[k].len();
        if !continued && !wrapped {
            if let (Some(last), Some(after)) = (last_seg(t), t.after) {
                if let Some(d) = vertex(last, after) {
                    let n = silhouettes[k].len();
                    let at = match part.role {
                        Role::Cut => usize::from(range.segs.start),
                        _ => usize::from(range.segs.end) % n,
                    };
                    discs.push((
                        Piece::Vertex {
                            part: range.part,
                            seg: u16::try_from(at).expect("a silhouette's index"),
                        },
                        d,
                    ));
                }
            }
        }
    }
    let center = center(surface, parts, &silhouettes, center_wanted);
    SideFeatures {
        body,
        faces,
        discs,
        center,
    }
}

/// The center of a side's drawn face: on a turned part, the vertex where the
/// range crosses the part's axis, its silhouette being symmetric about it;
/// otherwise the middle of the chord from the side's start to its end.
fn center(
    surface: &Surface,
    parts: &[Part],
    silhouettes: &[Vec<Seg>],
    wanted: bool,
) -> (Piece, Point) {
    let first = &surface.ranges[0];
    let piece = Piece::Center { part: first.part };
    if wanted {
        if let Sweep::Revolved { axis_x } = parts[first.part.index()].sweep {
            let sil = &silhouettes[0];
            for s in first.segs.clone().skip(1) {
                let p = sil[usize::from(s)].start();
                if p.x.to_bits() == axis_x.to_bits() {
                    return (piece, p);
                }
            }
        }
    }
    let last = surface.ranges.len() - 1;
    let start = silhouettes[0][usize::from(first.segs.start)].start();
    let end = silhouettes[last][usize::from(surface.ranges[last].segs.end) - 1].end();
    (
        piece,
        Point::new(0.5 * (start.x + end.x), 0.5 * (start.y + end.y)),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{Material, PartCode, SurfaceRef};
    use super::*;

    fn polygon(points: &[(f64, f64)]) -> Vec<Seg> {
        (0..points.len())
            .map(|i| {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                Seg::Line {
                    a: Point::new(a.0, a.1),
                    b: Point::new(b.0, b.1),
                }
            })
            .collect()
    }

    fn part(outline: Vec<Seg>, role: Role) -> Part {
        let n = u16::try_from(outline.len()).unwrap();
        Part {
            body: BodyId(1),
            code: PartCode::Keystick,
            outline,
            sweep: Sweep::Extruded {
                width: 0.02,
                across: 0.0,
            },
            role,
            material: Material::KeyWood,
            placed_by: vec![(0..n, "a test")],
        }
    }

    fn range(part: u16, segs: core::ops::Range<u16>) -> SurfaceRef {
        SurfaceRef {
            part: PartId(part),
            segs,
        }
    }

    /// A block whose bottom is split in two collinear segments, and a second
    /// block continuing its bottom: a straight joint is no corner, whether
    /// within one outline or between two parts, and the chain's two ends are.
    #[test]
    fn a_straight_joint_is_no_feature_and_a_chain_has_its_two_corners() {
        let parts = vec![
            part(
                polygon(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]),
                Role::Solid,
            ),
            part(
                polygon(&[(2.0, 0.0), (3.0, 0.0), (3.0, 1.0), (2.0, 1.0)]),
                Role::Solid,
            ),
        ];
        let underside = Surface::of(&[range(0, 0..2), range(1, 0..1)]);
        let side = side(&underside, &parts, false);
        assert_eq!(side.faces.len(), 3);
        let corners: Vec<Point> = side.discs.iter().map(|(_, d)| d.center).collect();
        assert_eq!(
            corners,
            vec![Point::new(0.0, 0.0), Point::new(3.0, 0.0)],
            "only the two ends of the chain are corners"
        );
        // Read as two separate surfaces, each part has its own corner at the
        // joint: that is what a chain spares.
        let alone = side_of(&parts, 0, 0..2);
        assert_eq!(alone.discs.len(), 2);
        assert_eq!(alone.discs[1].1.center, Point::new(2.0, 0.0));
    }

    fn side_of(parts: &[Part], p: u16, segs: core::ops::Range<u16>) -> SideFeatures {
        side(&Surface::one(range(p, segs)), parts, false)
    }

    /// A cut presents its left-hand side, into the void, and no disc at all:
    /// its corners are concave from the wood's side.
    #[test]
    fn a_cut_presents_its_inside_and_no_corner() {
        let parts = vec![part(
            polygon(&[(0.0, 0.0), (1.0, 0.0), (1.0, 2.0), (0.0, 2.0)]),
            Role::Cut,
        )];
        let top = side_of(&parts, 0, 2..3);
        assert!(top.discs.is_empty());
        assert_eq!(top.faces.len(), 1);
        assert_eq!(
            top.faces[0].1.normal,
            Dir::NEG_Y,
            "the top of a cut faces down"
        );
    }

    /// The wood round a slot cut up from its underside: the underside behind
    /// the slot, the slot's back wall, its front wall, the underside in
    /// front, the walls read off the cut as the wood runs. The corners of the
    /// wood at the slot's mouth are convex, with the sectors a corner of 90°
    /// has there; the slot's own top corners are concave and no feature; a
    /// wall presents its face into the slot.
    #[test]
    fn a_slot_cut_from_the_underside_has_convex_corners_at_its_mouth() {
        let parts = vec![
            part(
                polygon(&[
                    (0.0, 0.0),
                    (1.5, 0.0),
                    (2.5, 0.0),
                    (4.0, 0.0),
                    (4.0, 1.0),
                    (0.0, 1.0),
                ]),
                Role::Solid,
            ),
            part(
                polygon(&[(1.5, 0.0), (2.5, 0.0), (2.5, 0.6), (1.5, 0.6)]),
                Role::Cut,
            ),
        ];
        let wood = Surface::of(&[
            range(0, 0..1),
            range(1, 3..4),
            range(1, 1..2),
            range(0, 2..3),
        ]);
        let side = side(&wood, &parts, false);
        let corners: Vec<(Point, Dir, Dir)> = side
            .discs
            .iter()
            .map(|(_, d)| (d.center, d.from, d.to))
            .collect();
        assert_eq!(
            corners,
            vec![
                (Point::new(0.0, 0.0), Dir::NEG_X, Dir::NEG_Y),
                (Point::new(1.5, 0.0), Dir::NEG_Y, Dir::X),
                (Point::new(2.5, 0.0), Dir::NEG_X, Dir::NEG_Y),
                (Point::new(4.0, 0.0), Dir::NEG_Y, Dir::X),
            ],
            "the block's two corners and the slot's two, nothing at the slot's top"
        );
        let normals: Vec<Dir> = side.faces.iter().map(|(_, f)| f.normal).collect();
        assert_eq!(normals, vec![Dir::NEG_Y, Dir::X, Dir::NEG_X, Dir::NEG_Y]);
    }

    /// A concave corner is no feature: an L-shaped block has five convex
    /// corners and one concave.
    #[test]
    fn a_concave_corner_is_no_feature() {
        let parts = vec![part(
            polygon(&[
                (0.0, 0.0),
                (3.0, 0.0),
                (3.0, 1.0),
                (1.0, 1.0),
                (1.0, 2.0),
                (0.0, 2.0),
            ]),
            Role::Solid,
        )];
        let all = side_of(&parts, 0, 0..6);
        assert_eq!(all.discs.len(), 5);
        assert!(
            all.discs
                .iter()
                .all(|(_, d)| d.center != Point::new(1.0, 1.0)),
            "the inside corner of the L is not a feature"
        );
    }
}
