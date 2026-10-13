//! The features of a pair's two sides, read once from the drawn outlines in
//! their bodies' frames.

use super::{ContactPair, PairGeometry, Part, PartId, Role, Surface, Sweep};
use crate::geometry::{Dir, Disc, Face, Point, Seg};
use crate::mechanism::BodyId;

/// A piece of a surface that a row reads.
///
/// Every piece is drawn: a segment of a part's silhouette, the vertex where
/// one starts, or the centre of a drawn face.
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
    /// The centre of a drawn face: where a turned part's face crosses its
    /// axis, or the middle of the face's chord.
    Centre {
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
    /// The centre of its drawn face.
    pub centre: (Piece, Point),
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
        let centre_of_b = matches!(pair.geometry, PairGeometry::CentreBearing);
        Self {
            a: side(&pair.a, parts, false),
            b: side(&pair.b, parts, centre_of_b),
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

/// The face a straight segment presents: its right-hand normal on a part of
/// material, its left-hand one on a cut, whose material lies outside it.
fn face(seg: Seg, role: Role) -> Option<Face> {
    let Seg::Line { a, b } = seg else {
        return None;
    };
    let d = Dir::new(b.x - a.x, b.y - a.y)?.vector();
    let normal = match role {
        Role::Cut => Dir::new(-d.y, d.x)?,
        Role::Solid | Role::Lining => Dir::new(d.y, -d.x)?,
    };
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
            centre: next.start(),
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

/// Reads one side: its faces, and unless its parts are cuts, its arcs and
/// convex vertices, in the order the ranges run; and the centre of its
/// drawn face.
fn side(surface: &Surface, parts: &[Part], centre_wanted: bool) -> SideFeatures {
    let silhouettes: Vec<Vec<Seg>> = surface
        .ranges
        .iter()
        .map(|r| parts[r.part.index()].silhouette())
        .collect();
    let body = parts[surface.ranges[0].part.index()].body;
    let mut faces = Vec::new();
    let mut discs = Vec::new();
    for (k, range) in surface.ranges.iter().enumerate() {
        let part = &parts[range.part.index()];
        let sil = &silhouettes[k];
        let n = sil.len();
        let material = part.role != Role::Cut;
        // The range just before, when it ends where this one starts.
        let joined = k > 0 && {
            let prev = &surface.ranges[k - 1];
            same(
                silhouettes[k - 1][usize::from(prev.segs.end) - 1].end(),
                sil[usize::from(range.segs.start)].start(),
            )
        };
        for s in range.segs.clone() {
            let i = usize::from(s);
            let seg = sil[i];
            if let Some(f) = face(seg, part.role) {
                faces.push((
                    Piece::Segment {
                        part: range.part,
                        seg: s,
                    },
                    f,
                ));
            }
            if !material {
                continue;
            }
            if let Seg::Arc { c, r, from, to } = seg {
                discs.push((
                    Piece::Arc {
                        part: range.part,
                        seg: s,
                    },
                    Disc {
                        centre: c,
                        radius: r,
                        from,
                        to,
                    },
                ));
            }
            // The vertex this segment starts at: after the segment before it
            // in this range, or after the last of the range before when the
            // two are one chain, or else after its own outline's previous.
            let previous = if s > range.segs.start {
                sil[i - 1]
            } else if joined {
                let prev = &surface.ranges[k - 1];
                silhouettes[k - 1][usize::from(prev.segs.end) - 1]
            } else {
                sil[(i + n - 1) % n]
            };
            if let Some(d) = vertex(previous, seg) {
                discs.push((
                    Piece::Vertex {
                        part: range.part,
                        seg: s,
                    },
                    d,
                ));
            }
        }
        // The vertex the range ends at, unless the next range starts there
        // and reads it as its own.
        let last = usize::from(range.segs.end) - 1;
        let continued = surface.ranges.get(k + 1).is_some_and(|next| {
            same(
                sil[last].end(),
                silhouettes[k + 1][usize::from(next.segs.start)].start(),
            )
        });
        let following = (last + 1) % n;
        // A range round the whole outline has already read the vertex it
        // closes on, as the start of its first segment.
        let wrapped = following == usize::from(range.segs.start);
        if material && !continued && !wrapped {
            if let Some(d) = vertex(sil[last], sil[following]) {
                discs.push((
                    Piece::Vertex {
                        part: range.part,
                        seg: u16::try_from(following).expect("a silhouette's index"),
                    },
                    d,
                ));
            }
        }
    }
    let centre = centre(surface, parts, &silhouettes, centre_wanted);
    SideFeatures {
        body,
        faces,
        discs,
        centre,
    }
}

/// The centre of a side's drawn face: on a turned part, the vertex where the
/// range crosses the part's axis, its silhouette being symmetric about it;
/// otherwise the middle of the chord from the side's start to its end.
fn centre(
    surface: &Surface,
    parts: &[Part],
    silhouettes: &[Vec<Seg>],
    wanted: bool,
) -> (Piece, Point) {
    let first = &surface.ranges[0];
    let piece = Piece::Centre { part: first.part };
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
    use super::super::{Material, PartCode, Provenance, SurfaceRef};
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
            sweep: Sweep::Extruded { width: 0.02 },
            role,
            material: Material::KeyWood,
            sources: vec![(0..n, Provenance::Assumed("a test"))],
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
        let corners: Vec<Point> = side.discs.iter().map(|(_, d)| d.centre).collect();
        assert_eq!(
            corners,
            vec![Point::new(0.0, 0.0), Point::new(3.0, 0.0)],
            "only the two ends of the chain are corners"
        );
        // Read as two separate surfaces, each part has its own corner at the
        // joint: that is what a chain spares.
        let alone = side_of(&parts, 0, 0..2);
        assert_eq!(alone.discs.len(), 2);
        assert_eq!(alone.discs[1].1.centre, Point::new(2.0, 0.0));
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
                .all(|(_, d)| d.centre != Point::new(1.0, 1.0)),
            "the inside corner of the L is not a feature"
        );
    }
}
