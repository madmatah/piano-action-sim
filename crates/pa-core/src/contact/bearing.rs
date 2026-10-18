//! How a key bears on a stop, read off the rows a pair has at a pose.
//!
//! # Three ways, and why all three
//!
//! Two flats meet face to face, or on an edge. Face to face is what a
//! regulated back rail does, and what the chamfer under the key's tail is cut
//! for; on an edge is what a keyboard with the wrong punchings does, and it
//! plays, and wears its cloth along one line. So the bench reports which,
//! rather than assuming the regulated one:
//!
//! - **on the stop's edge**: the governing row is one of the stop's discs, the
//!   edge arc that rounds its face, on a face of the key;
//! - **on the key's own corner**: the governing row is one of the key's
//!   discs, a corner of its outline, on the stop;
//! - **flat**: on the stop's edge, and the stop's face lies along the key's
//!   so closely that its other edge is within [`FLAT_WITHIN`] of closing too.
//!
//! Side A of a pair is the key's and side B the stop's, in every pair the key
//! declares; that is the only sense in which this module knows a key.

use super::features::{Feature, Piece};
use super::rows::Rows;
use crate::geometry::{Face, Point};
use crate::mechanism::{Mechanism, Placement, Pose};

/// How far the far edge of a stop's face may stand from closing when the
/// near one has closed, for the two faces to count as bedded flat, meters:
/// two and a half microns.
///
/// On the back rail's 24 mm of cloth that is a slope of a ten-thousandth
/// between the key's face and the cloth's: a thirtieth of the thinnest paper
/// in the drawer, and far below the felt's own compliance. A technician
/// calling a key bedded is making a statement of about this precision.
pub const FLAT_WITHIN: f64 = 2.5e-6;

/// Which of the three ways the key bears on a stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HowItBears {
    /// Face to face, the load spread over the whole of the stop's face.
    Flat,
    /// On the stop's own edge: the key's face is tilted across the stop, so
    /// the load sits along the line where the stop's face is rounded off.
    StopEdge,
    /// On a corner of the key's own outline, digging into the stop.
    KeyCorner,
}

/// How side A of a pair bears on side B at a pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bearing {
    /// Which of the three ways.
    pub how: HowItBears,
    /// The governing row's gap, meters: zero to the rounding when the key
    /// stands on the stop.
    pub gap: f64,
    /// Where the load sits, in the world: the governing row's point.
    pub point: Point,
    /// The slope of the key's bearing face over the stop's face: the tangent
    /// of the angle from the stop's face to the key's, counter-clockwise
    /// positive, so over a level stop positive means the key's face rises
    /// toward the player and its rear end is the low one. A slope and not an
    /// angle, because `pa-core` reaches no platform `atan`. `None` where the
    /// stop presents no straight face to measure from.
    pub slope: Option<f64>,
}

fn sub(u: Point, v: Point) -> Point {
    Point::new(u.x - v.x, u.y - v.y)
}

fn dot(u: Point, v: Point) -> f64 {
    u.x * v.x + u.y * v.y
}

fn cross(u: Point, v: Point) -> f64 {
    u.x * v.y - u.y * v.x
}

fn same(p: Point, q: Point) -> bool {
    p.x.to_bits() == q.x.to_bits() && p.y.to_bits() == q.y.to_bits()
}

/// The point of a disc piece in its body's frame: an arc's two ends, a
/// vertex's one.
fn ends_of(piece: Piece, discs: &[(Piece, crate::geometry::Disc)]) -> Vec<Point> {
    discs
        .iter()
        .filter(|(p, _)| *p == piece)
        .flat_map(|(_, d)| {
            let (f, t) = (d.from.vector(), d.to.vector());
            [
                Point::new(d.center.x + d.radius * f.x, d.center.y + d.radius * f.y),
                Point::new(d.center.x + d.radius * t.x, d.center.y + d.radius * t.y),
            ]
        })
        .collect()
}

/// The faces of a side that meet a disc piece at one of its ends, in the
/// body's frame.
fn faces_at(ends: &[Point], faces: &[(Piece, Face)]) -> Vec<Face> {
    faces
        .iter()
        .filter(|(_, f)| ends.iter().any(|&e| same(e, f.a) || same(e, f.b)))
        .map(|(_, f)| *f)
        .collect()
}

/// A face in its body's frame, and where its body stands.
#[derive(Clone, Copy)]
struct Placed {
    face: Face,
    placement: Placement,
}

impl Placed {
    /// Where the face starts and ends, in the world.
    fn ends(&self) -> (Point, Point) {
        (
            self.placement.world(self.face.a),
            self.placement.world(self.face.b),
        )
    }

    /// The way the face runs, from its start to its end, turned into the
    /// world: the run in the body's frame, turned, which is exact where the
    /// difference of two world points would carry their rounding.
    fn run(&self) -> Point {
        self.placement.rotate(sub(self.face.b, self.face.a))
    }
}

/// The slope of `key` over `stop`: the tangent of the angle from the stop's
/// run to the key's, the key's turned round first if it runs the other way.
fn slope_of(stop: &Placed, key: &Placed) -> Option<f64> {
    let s = stop.run();
    let s = crate::geometry::Dir::new(s.x, s.y)?.vector();
    let mut k = key.run();
    if dot(k, s) < 0.0 {
        k = Point::new(-k.x, -k.y);
    }
    let along = dot(s, k);
    (along != 0.0).then(|| cross(s, k) / along)
}

/// Where the projection of `p` falls along the segment from `a` to `b`, `0`
/// at its start and `1` at its end.
fn along(p: Point, (a, b): (Point, Point)) -> f64 {
    let e = sub(b, a);
    dot(sub(p, a), e) / dot(e, e)
}

impl Mechanism {
    /// How side A of pair `pair` bears on side B at `pose`, read off the
    /// pair's rows in `rows`, which must have been read at that pose.
    ///
    /// The governing row, the least gap, says where and through what: one of
    /// B's discs on one of A's faces is on the stop's edge, or flat when B's
    /// face along that edge lies within [`FLAT_WITHIN`] of A's over its whole
    /// length; one of A's discs is on the key's own corner. The slope is A's
    /// bearing face over B's: on an edge, the face the edge bears on over the
    /// face it rounds; on a corner, the face of the corner that runs on
    /// toward the stop over the stop's face. `None` when the pair has no
    /// row, or bears through its center: a center bearing is declared to bear
    /// there.
    pub fn bearing(&self, pose: &Pose, rows: &Rows, pair: u16) -> Option<Bearing> {
        let i = rows.governing(pair)?;
        let row = rows.rows()[i];
        let sides = self.features.get(usize::from(pair))?;
        let (a, b) = (&sides.a, &sides.b);
        let (place_a, place_b) = (pose.placement(a.body), pose.placement(b.body));
        let placed_a = |face: Face| Placed {
            face,
            placement: place_a,
        };
        let placed_b = |face: Face| Placed {
            face,
            placement: place_b,
        };
        let Feature {
            a: piece_a,
            b: piece_b,
        } = row.feature;
        let reading = |how, slope| Bearing {
            how,
            gap: row.gap,
            point: row.point,
            slope,
        };
        if matches!(piece_b, Piece::Center { .. }) {
            return None;
        }
        if piece_a.is_disc() {
            // A corner of the key: its slope is the corner's face that runs
            // on toward the stop, over the stop's face.
            let stop = match piece_b {
                Piece::Segment { .. } => {
                    b.faces.iter().find(|(p, _)| *p == piece_b).map(|(_, f)| *f)
                }
                _ => facing(
                    &faces_at(&ends_of(piece_b, &b.discs), &b.faces),
                    &place_b,
                    row.normal.vector(),
                ),
            }
            .map(placed_b);
            let corner = place_a.world(ends_of(piece_a, &a.discs).first().copied()?);
            let toward = stop.map_or(row.point, |f| {
                let (s, e) = f.ends();
                Point::new(0.5 * (s.x + e.x), 0.5 * (s.y + e.y))
            });
            let key = faces_at(&ends_of(piece_a, &a.discs), &a.faces)
                .into_iter()
                .map(placed_a)
                .max_by(|f, g| {
                    let reach = |h: &Placed| {
                        let (s, e) = h.ends();
                        let far = if same(s, corner) { e } else { s };
                        let run = sub(far, corner);
                        dot(sub(toward, corner), run) / dot(run, run).sqrt()
                    };
                    reach(f).total_cmp(&reach(g))
                });
            let slope = match (stop, key) {
                (Some(s), Some(k)) => slope_of(&s, &k),
                _ => None,
            };
            return Some(reading(HowItBears::KeyCorner, slope));
        }
        // One of the stop's discs on a face of the key.
        let key = a
            .faces
            .iter()
            .find(|(p, _)| *p == piece_a)
            .map(|(_, f)| placed_a(*f))?;
        let stop = facing(
            &faces_at(&ends_of(piece_b, &b.discs), &b.faces),
            &place_b,
            row.normal.vector(),
        )
        .map(placed_b);
        let Some(stop) = stop else {
            return Some(reading(HowItBears::StopEdge, None));
        };
        let slope = slope_of(&stop, &key);
        let run = stop.run();
        let length = dot(run, run).sqrt();
        let (s, e) = stop.ends();
        let spans = [s, e]
            .iter()
            .all(|&p| (0.0..=1.0).contains(&along(p, key.ends())));
        let flat = spans && slope.is_some_and(|x| x.abs() * length <= FLAT_WITHIN);
        Some(reading(
            if flat {
                HowItBears::Flat
            } else {
                HowItBears::StopEdge
            },
            slope,
        ))
    }
}

/// Of the faces meeting a disc of side B, the one that faces side A best:
/// whose outward normal, turned into the world, lies closest to the row's
/// normal from B toward A. The top of a strip, not its side.
fn facing(faces: &[Face], placement: &Placement, normal: Point) -> Option<Face> {
    faces.iter().copied().max_by(|f, g| {
        let score = |h: &Face| dot(placement.rotate(h.normal.vector()), normal);
        score(f).total_cmp(&score(g))
    })
}

impl Mechanism {
    /// How far side B's drawn faces stand past side A's faces at `pose`,
    /// meters: the deepest point of a B face, within the span of an A face,
    /// on the material side of that face's line; zero where none does.
    ///
    /// For a pair read at its center ([`super::PairGeometry::CenterBearing`])
    /// this is the drawn face's overlap with the key that its idealization
    /// leaves aside. Bounded, while the center bears, by half the face's
    /// width times the key's tilt; past every bound once the center stands
    /// off the key's underside, where the pair bears on nothing and the key
    /// would come down through the face as drawn.
    ///
    /// # Panics
    ///
    /// When the mechanism has no pair `pair`.
    pub fn face_overlap(&self, pose: &Pose, pair: u16) -> f64 {
        let sides = &self.features[usize::from(pair)];
        let (place_a, place_b) = (pose.placement(sides.a.body), pose.placement(sides.b.body));
        let mut deepest = 0.0_f64;
        for (_, fa) in &sides.a.faces {
            let (a0, a1) = (place_a.world(fa.a), place_a.world(fa.b));
            let e = sub(a1, a0);
            let e2 = dot(e, e);
            if e2.is_nan() || e2 <= 0.0 {
                continue;
            }
            let n = place_a.rotate(fa.normal.vector());
            for (_, fb) in &sides.b.faces {
                let (b0, b1) = (place_b.world(fb.a), place_b.world(fb.b));
                // Where along B's face its points stand within A's span:
                // t, A's parameter, is linear in s, B's.
                let (t0, t1) = (along(b0, (a0, a1)), along(b1, (a0, a1)));
                let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                if (t1 - t0).abs() > 0.0 {
                    let (s0, s1) = ((0.0 - t0) / (t1 - t0), (1.0 - t0) / (t1 - t0));
                    lo = lo.max(s0.min(s1));
                    hi = hi.min(s0.max(s1));
                } else if !(0.0..=1.0).contains(&t0) {
                    continue;
                }
                if lo > hi {
                    continue;
                }
                for s in [lo, hi] {
                    let p = Point::new(b0.x + s * (b1.x - b0.x), b0.y + s * (b1.y - b0.y));
                    deepest = deepest.max(-dot(sub(p, a0), n));
                }
            }
        }
        deepest
    }
}
