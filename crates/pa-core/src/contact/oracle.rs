//! The brute-force gap oracle. Tests only.
//!
//! The drawn outlines sampled every [`SPACING`] along their length, and
//! nothing else: no feature, no primitive, no sector or span, no code the
//! rows run. It reads three things. Two surfaces standing apart are as far
//! apart as their closest two samples, which is the true distance or up to
//! one spacing more, each closest point lying within half a spacing of a
//! sample on its own surface. A point stands on a surface when a sample of it
//! lies within half a spacing. A point stands inside a side's material when
//! it lies in one of the side's drawn sections and in none of its body's
//! cuts, by the parity of a ray's crossings of the sampled outlines: a side's
//! material is the parts its ranges belong to, or, for a side that is a
//! cut's range, every part of its body; for a centre bearing the cuts are not
//! taken out, since the key bears on the punching beside its slot, out of the
//! plane. Only the pair's own parts count, because pins stand through a key
//! in its mortises by design.

use super::{ContactPair, PairGeometry, Part, Role, Surface};
use crate::geometry::{Point, Seg};
use crate::mechanism::{Mechanism, Placement, Pose};

/// How far apart the samples are along an outline, at most, metres: five
/// hundredths of a millimetre.
pub const SPACING: f64 = 5e-5;

/// Samples along a segment, at most [`SPACING`] apart, its start included and
/// its end left out, in the frame the segment is written in.
fn sample(seg: Seg, out: &mut Vec<Point>) {
    match seg {
        Seg::Line { a, b } => {
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let n = steps((dx * dx + dy * dy).sqrt());
            for k in 0..n {
                let t = k as f64 / n as f64;
                out.push(Point::new(a.x + t * dx, a.y + t * dy));
            }
        }
        Seg::Arc { c, r, from, to } => {
            let (f, t) = (from.vector(), to.vector());
            let start = libm::atan2(f.y, f.x);
            let mut sweep = libm::atan2(f.x * t.y - f.y * t.x, f.x * t.x + f.y * t.y);
            if sweep <= 0.0 {
                sweep += 2.0 * core::f64::consts::PI;
            }
            let n = steps(r * sweep);
            for k in 0..n {
                let angle = start + sweep * k as f64 / n as f64;
                out.push(Point::new(
                    c.x + r * libm::cos(angle),
                    c.y + r * libm::sin(angle),
                ));
            }
        }
    }
}

/// How many samples a length takes at [`SPACING`].
fn steps(length: f64) -> usize {
    (length / SPACING).ceil().max(1.0) as usize
}

/// The samples of a surface, in the world: every range's segments, and the
/// end of each range's last.
pub fn surface(parts: &[Part], surface: &Surface, placement: &Placement) -> Vec<Point> {
    let mut out = Vec::new();
    for range in &surface.ranges {
        let silhouette = parts[range.part.index()].silhouette();
        for s in range.segs.clone() {
            sample(silhouette[usize::from(s)], &mut out);
        }
        out.push(silhouette[usize::from(range.segs.end) - 1].end());
    }
    out.into_iter().map(|p| placement.world(p)).collect()
}

/// A part's whole drawn section, sampled into a polygon, in the world.
fn polygon(part: &Part, placement: &Placement) -> Vec<Point> {
    let mut out = Vec::new();
    for seg in part.silhouette() {
        sample(seg, &mut out);
    }
    out.into_iter().map(|p| placement.world(p)).collect()
}

/// Whether `p` is inside a closed polygon, by the parity of the crossings of
/// a ray from it.
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

/// A polygon with the box that holds it, which spares the crossing count for
/// every point outside the box.
struct Boxed {
    polygon: Vec<Point>,
    low: Point,
    high: Point,
}

impl Boxed {
    fn new(polygon: Vec<Point>) -> Self {
        let (mut low, mut high) = (
            Point::new(f64::INFINITY, f64::INFINITY),
            Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        );
        for p in &polygon {
            low = Point::new(low.x.min(p.x), low.y.min(p.y));
            high = Point::new(high.x.max(p.x), high.y.max(p.y));
        }
        Self { polygon, low, high }
    }

    fn holds(&self, p: Point) -> bool {
        p.x >= self.low.x
            && p.x <= self.high.x
            && p.y >= self.low.y
            && p.y <= self.high.y
            && inside(&self.polygon, p)
    }
}

/// One body's material: its parts' sections, and its cuts'.
pub struct Material {
    solids: Vec<Boxed>,
    cuts: Vec<Boxed>,
}

impl Material {
    /// The material of one side at `pose`: the parts its ranges belong to,
    /// or every part of its body for a side that is a cut's range; less its
    /// body's cuts unless `beside_the_cuts`.
    pub fn of(mechanism: &Mechanism, pose: &Pose, side: &Surface, beside_the_cuts: bool) -> Self {
        let parts = mechanism.parts();
        let body = parts[side.ranges[0].part.index()].body;
        let placement = pose.placement(body);
        let own: Vec<usize> = side
            .ranges
            .iter()
            .map(|r| r.part.index())
            .filter(|&i| parts[i].role != Role::Cut)
            .collect();
        let mut solids = Vec::new();
        let mut cuts = Vec::new();
        for (i, part) in parts.iter().enumerate().filter(|(_, p)| p.body == body) {
            match part.role {
                Role::Cut => {
                    if !beside_the_cuts {
                        cuts.push(Boxed::new(polygon(part, &placement)));
                    }
                }
                Role::Solid | Role::Lining => {
                    if own.is_empty() || own.contains(&i) {
                        solids.push(Boxed::new(polygon(part, &placement)));
                    }
                }
            }
        }
        Self { solids, cuts }
    }

    /// Whether `p` is inside the material: in a part, and not in a cut.
    pub fn holds(&self, p: Point) -> bool {
        self.solids.iter().any(|s| s.holds(p)) && !self.cuts.iter().any(|c| c.holds(p))
    }
}

fn distance(p: Point, q: Point) -> f64 {
    let (dx, dy) = (p.x - q.x, p.y - q.y);
    (dx * dx + dy * dy).sqrt()
}

/// How far `p` stands from the nearest of `samples`.
pub fn nearest(p: Point, samples: &[Point]) -> f64 {
    samples
        .iter()
        .map(|&q| distance(p, q))
        .fold(f64::INFINITY, f64::min)
}

/// One pair at one pose, as the oracle reads it.
pub struct Pair {
    /// Side A's surface, sampled, in the world.
    pub a: Vec<Point>,
    /// Side B's surface, sampled, in the world.
    pub b: Vec<Point>,
    /// Side A's material.
    pub in_a: Material,
    /// Side B's material.
    pub in_b: Material,
    /// For a centre bearing, side B's face read at its centre, the middle of
    /// its two ends.
    pub centre: Option<Point>,
}

impl Pair {
    /// Reads `pair` at `pose`.
    pub fn read(mechanism: &Mechanism, pose: &Pose, pair: &ContactPair) -> Self {
        let parts = mechanism.parts();
        let place = |s: &Surface| pose.placement(parts[s.ranges[0].part.index()].body);
        let a = surface(parts, &pair.a, &place(&pair.a));
        let b = surface(parts, &pair.b, &place(&pair.b));
        let centre_bearing = pair.geometry == PairGeometry::CentreBearing;
        let centre = centre_bearing.then(|| {
            let (first, last) = (b[0], b[b.len() - 1]);
            Point::new(0.5 * (first.x + last.x), 0.5 * (first.y + last.y))
        });
        Self {
            in_a: Material::of(mechanism, pose, &pair.a, centre_bearing),
            in_b: Material::of(mechanism, pose, &pair.b, centre_bearing),
            a,
            b,
            centre,
        }
    }

    /// How far apart the two surfaces stand, metres: the closest two
    /// samples; for a centre bearing, the centre from side A's surface.
    pub fn apart(&self) -> f64 {
        match self.centre {
            Some(c) => nearest(c, &self.a),
            None => self
                .a
                .iter()
                .map(|&p| nearest(p, &self.b))
                .fold(f64::INFINITY, f64::min),
        }
    }

    /// Whether any sample of either surface, or the centre, stands inside
    /// the other side's material.
    pub fn overlapping(&self) -> bool {
        match self.centre {
            Some(c) => self.in_a.holds(c),
            None => {
                self.a.iter().any(|&p| self.in_b.holds(p))
                    || self.b.iter().any(|&p| self.in_a.holds(p))
            }
        }
    }
}
