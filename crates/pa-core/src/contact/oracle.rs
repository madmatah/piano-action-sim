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
//! cut's range, every part of its body; for a center bearing the cuts are not
//! taken out, since the key bears on the punching beside its slot, out of the
//! plane. Only the pair's own parts count, because pins stand through a key
//! in its mortises by design.

use super::{ContactPair, PairGeometry, Part, Role, Surface};
use crate::geometry::{Point, Seg};
use crate::mechanism::{Mechanism, Placement, Pose};

/// How far apart the samples are along an outline, at most, meters: five
/// hundredths of a millimeter.
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

    /// How deep `p` stands inside the material, meters: its distance to the
    /// nearest sample of the material's boundary, its parts' outlines and its
    /// cuts'; zero outside. A point on the boundary, touching, stands within
    /// half a spacing of a sample, whichever side the crossing count puts it.
    pub fn depth(&self, p: Point) -> f64 {
        if !self.holds(p) {
            return 0.0;
        }
        self.solids
            .iter()
            .chain(&self.cuts)
            .map(|b| nearest(p, &b.polygon))
            .fold(f64::INFINITY, f64::min)
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
    /// For a center bearing, side B's face read at its center, the middle of
    /// its two ends.
    pub center: Option<Point>,
}

impl Pair {
    /// Reads `pair` at `pose`.
    pub fn read(mechanism: &Mechanism, pose: &Pose, pair: &ContactPair) -> Self {
        let parts = mechanism.parts();
        let place = |s: &Surface| pose.placement(parts[s.ranges[0].part.index()].body);
        let a = surface(parts, &pair.a, &place(&pair.a));
        let b = surface(parts, &pair.b, &place(&pair.b));
        let center_bearing = pair.geometry == PairGeometry::CenterBearing;
        let center = center_bearing.then(|| {
            let (first, last) = (b[0], b[b.len() - 1]);
            Point::new(0.5 * (first.x + last.x), 0.5 * (first.y + last.y))
        });
        Self {
            in_a: Material::of(mechanism, pose, &pair.a, center_bearing),
            in_b: Material::of(mechanism, pose, &pair.b, center_bearing),
            a,
            b,
            center,
        }
    }

    /// How far apart the two surfaces stand, meters: the closest two
    /// samples; for a center bearing, the center from side A's surface.
    pub fn apart(&self) -> f64 {
        match self.center {
            Some(c) => nearest(c, &self.a),
            None => self
                .a
                .iter()
                .map(|&p| nearest(p, &self.b))
                .fold(f64::INFINITY, f64::min),
        }
    }

    /// How deep any sample of either surface, or the center, stands inside
    /// the other side's material, meters: zero where none does. A surface
    /// touching the material, through this pair or another, reads within
    /// half a spacing.
    pub fn overlap(&self) -> f64 {
        match self.center {
            Some(c) => self.in_a.depth(c),
            None => self
                .a
                .iter()
                .map(|&p| self.in_b.depth(p))
                .chain(self.b.iter().map(|&p| self.in_a.depth(p)))
                .fold(0.0, f64::max),
        }
    }
}

/// The brute-force oracle's polylines: each drawn line whole, each arc cut
/// into chords no longer than [`SPACING`], in the world. A line needs no
/// sample between its ends for a crossing or a distance to be read on it
/// exactly; an arc's chords stand inside it by at most their sagitta,
/// `SPACING² / 8r`. Still no feature, no primitive, no code the rows run:
/// the outline, cut into straight pieces, and nothing else.
pub mod polyline {
    use super::SPACING;
    use crate::contact::{Part, Role, Surface};
    use crate::geometry::{Point, Seg};
    use crate::mechanism::{BodyId, Mechanism, Placement, Pose};

    /// A segment's points, its start and, along an arc, the chords' ends
    /// before its end; the end left out, as the next segment starts there.
    fn points(seg: Seg, out: &mut Vec<Point>) {
        match seg {
            Seg::Line { a, .. } => out.push(a),
            Seg::Arc { c, r, from, to } => {
                let (f, t) = (from.vector(), to.vector());
                let start = libm::atan2(f.y, f.x);
                let mut sweep = libm::atan2(f.x * t.y - f.y * t.x, f.x * t.x + f.y * t.y);
                if sweep <= 0.0 {
                    sweep += 2.0 * core::f64::consts::PI;
                }
                let n = ((r * sweep) / SPACING).ceil().max(1.0) as usize;
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

    /// A part's drawn section, as a closed polygon, in the world.
    pub fn polygon(part: &Part, placement: &Placement) -> Vec<Point> {
        let mut out = Vec::new();
        for seg in part.silhouette() {
            points(seg, &mut out);
        }
        out.into_iter().map(|p| placement.world(p)).collect()
    }

    /// One surface, as polylines, one per range, in the world.
    pub fn surface(parts: &[Part], surface: &Surface, placement: &Placement) -> Vec<Vec<Point>> {
        surface
            .ranges
            .iter()
            .map(|range| {
                let sil = parts[range.part.index()].silhouette();
                let mut out = Vec::new();
                for s in range.segs.clone() {
                    points(sil[usize::from(s)], &mut out);
                }
                out.push(sil[usize::from(range.segs.end) - 1].end());
                out.into_iter().map(|p| placement.world(p)).collect()
            })
            .collect()
    }

    fn sub(u: Point, v: Point) -> Point {
        Point::new(u.x - v.x, u.y - v.y)
    }

    fn dot(u: Point, v: Point) -> f64 {
        u.x * v.x + u.y * v.y
    }

    /// How far `p` stands from the segment from `a` to `b`.
    pub fn to_segment(p: Point, a: Point, b: Point) -> f64 {
        let e = sub(b, a);
        let e2 = dot(e, e);
        let t = if e2 > 0.0 {
            (dot(sub(p, a), e) / e2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let d = sub(p, Point::new(a.x + t * e.x, a.y + t * e.y));
        dot(d, d).sqrt()
    }

    /// How far `p` stands from an open polyline.
    pub fn to_polyline(p: Point, line: &[Point]) -> f64 {
        match line.len() {
            0 => f64::INFINITY,
            1 => to_segment(p, line[0], line[0]),
            _ => line
                .windows(2)
                .map(|w| to_segment(p, w[0], w[1]))
                .fold(f64::INFINITY, f64::min),
        }
    }

    /// Whether two segments cross.
    fn cross(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
        let side = |p: Point, q: Point, r: Point| {
            let (u, v) = (sub(q, p), sub(r, p));
            u.x * v.y - u.y * v.x
        };
        let (d1, d2) = (side(b0, b1, a0), side(b0, b1, a1));
        let (d3, d4) = (side(a0, a1, b0), side(a0, a1, b1));
        d1 * d2 < 0.0 && d3 * d4 < 0.0
    }

    /// How far apart two open polylines stand: zero where they cross.
    pub fn apart(a: &[Point], b: &[Point]) -> f64 {
        let mut best = f64::INFINITY;
        for wa in a.windows(2) {
            for wb in b.windows(2) {
                if cross(wa[0], wa[1], wb[0], wb[1]) {
                    return 0.0;
                }
                best = best
                    .min(to_segment(wa[0], wb[0], wb[1]))
                    .min(to_segment(wa[1], wb[0], wb[1]))
                    .min(to_segment(wb[0], wa[0], wa[1]))
                    .min(to_segment(wb[1], wa[0], wa[1]));
            }
        }
        best
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

    /// How far `p` stands from a closed polygon's boundary.
    fn to_boundary(polygon: &[Point], p: Point) -> f64 {
        let n = polygon.len();
        (0..n)
            .map(|i| to_segment(p, polygon[i], polygon[(i + 1) % n]))
            .fold(f64::INFINITY, f64::min)
    }

    /// One body's material: the sections of the parts that make it, less its
    /// cuts.
    pub struct Region {
        solids: Vec<Vec<Point>>,
        cuts: Vec<Vec<Point>>,
    }

    impl Region {
        /// `body`'s material at `pose`, of the parts `keep` keeps, less every
        /// cut of the body unless `beside_the_cuts`: a key bears on a punching
        /// beside its slot, out of the plane.
        pub fn of(
            mechanism: &Mechanism,
            pose: &Pose,
            body: BodyId,
            keep: &dyn Fn(usize) -> bool,
            beside_the_cuts: bool,
        ) -> Self {
            let placement = pose.placement(body);
            let mut solids = Vec::new();
            let mut cuts = Vec::new();
            for (i, part) in mechanism.parts().iter().enumerate() {
                if part.body != body {
                    continue;
                }
                match part.role {
                    Role::Cut if !beside_the_cuts => cuts.push(polygon(part, &placement)),
                    Role::Cut => {}
                    Role::Solid | Role::Lining => {
                        if keep(i) {
                            solids.push(polygon(part, &placement));
                        }
                    }
                }
            }
            Self { solids, cuts }
        }

        /// How deep `p` stands inside the material, meters: its distance to
        /// the nearest boundary of a part or a cut, zero outside.
        pub fn depth(&self, p: Point) -> f64 {
            let held =
                self.solids.iter().any(|s| inside(s, p)) && !self.cuts.iter().any(|c| inside(c, p));
            if !held {
                return 0.0;
            }
            self.solids
                .iter()
                .chain(&self.cuts)
                .map(|s| to_boundary(s, p))
                .fold(f64::INFINITY, f64::min)
        }
    }
}
