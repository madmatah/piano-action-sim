//! Planar primitives: points, directions, the two kinds of segment every
//! outline is made of, and the signed distances contacts are measured by.
//!
//! An outline is a closed chain of [`Seg`]s, lines and arcs. The parts built
//! from outlines, and the ranges into them that a contact may read, are in
//! [`crate::contact`]. The three signed primitives every contact row is made
//! of, disc on face, disc on disc and centre on face, are in [`signed`].

pub mod signed;

pub use signed::{centre_on_face, disc_on_disc, disc_on_face, Disc, Face, Proximity};

/// A point in a part's local frame, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Horizontal coordinate, metres. Positive towards the player.
    pub x: f64,
    /// Vertical coordinate, metres. Positive upwards.
    pub y: f64,
}

impl Point {
    /// Constructs a point.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A direction: a vector of unit length, in a body's frame or in the world.
///
/// Built only by normalizing, so an axis cannot carry a length that would
/// scale its coordinate without anybody noticing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dir {
    x: f64,
    y: f64,
}

impl Dir {
    /// Towards the player.
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    /// Upwards.
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
    /// Away from the player.
    pub const NEG_X: Self = Self { x: -1.0, y: 0.0 };
    /// Downwards.
    pub const NEG_Y: Self = Self { x: 0.0, y: -1.0 };

    /// The direction of `(x, y)`, or `None` for a vector with no length or
    /// with a component that is not a finite number.
    pub fn new(x: f64, y: f64) -> Option<Self> {
        let length = (x * x + y * y).sqrt();
        if length.is_finite() && length > 0.0 {
            Some(Self {
                x: x / length,
                y: y / length,
            })
        } else {
            None
        }
    }

    /// The direction as a vector of unit length.
    pub const fn vector(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// The direction a counter-clockwise turn from `self` to `to` passes
    /// halfway: a bisection with no angle computed, so no transcendental.
    ///
    /// Under a half turn it is the normalized sum of the two; over a half
    /// turn, its opposite; at exactly a half turn, `self` turned a quarter
    /// turn. Two equal directions bisect to themselves.
    fn halfway_to(self, to: Self) -> Self {
        let cross = self.x * to.y - self.y * to.x;
        let dot = self.x * to.x + self.y * to.y;
        let (sx, sy) = (self.x + to.x, self.y + to.y);
        let bisector = if cross > 0.0 {
            Self::new(sx, sy)
        } else if cross < 0.0 {
            Self::new(-sx, -sy)
        } else if dot < 0.0 {
            Some(Self {
                x: -self.y,
                y: self.x,
            })
        } else {
            None
        };
        bisector.unwrap_or(self)
    }
}

/// One segment of a part outline.
///
/// These two primitives are enough for the whole action: Hirschkorn and Thorin
/// independently converged on circle-against-line, circle-against-circle, and a
/// hybrid profile made of an arc joined to two tangent lines. The hybrid profile
/// is expressed here as consecutive `Arc` and `Line` segments; because the lines
/// are tangent to the arc, the contact normal is C1 across the junction, which
/// is exactly why the jack nose can slide on the knuckle without the
/// transmitted torque jumping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seg {
    /// Straight segment from `a` to `b`.
    Line {
        /// Start point, local frame.
        a: Point,
        /// End point, local frame.
        b: Point,
    },
    /// Circular arc about `c`, swept counter-clockwise from the direction
    /// `from` to the direction `to`.
    ///
    /// The two end directions are stored rather than two angles, so that
    /// neither its ends nor its drawing need an `atan2`: the arc starts at
    /// `c + r·from` and ends at `c + r·to`. Only a point placed a fraction of
    /// the way round it ([`crate::contact::Anchor::point`]) takes the sweep's
    /// angle back.
    Arc {
        /// Centre, local frame.
        c: Point,
        /// Radius, metres.
        r: f64,
        /// Where the arc starts, seen from its centre.
        from: Dir,
        /// Where the arc ends, seen from its centre.
        to: Dir,
    },
}

/// `c + r·d`: a point of a circle, given the direction it lies in.
fn on_circle(c: Point, r: f64, d: Dir) -> Point {
    Point::new(c.x + r * d.x, c.y + r * d.y)
}

impl Seg {
    /// Where the segment starts.
    pub fn start(&self) -> Point {
        match *self {
            Self::Line { a, .. } => a,
            Self::Arc { c, r, from, .. } => on_circle(c, r, from),
        }
    }

    /// Where the segment ends.
    pub fn end(&self) -> Point {
        match *self {
            Self::Line { b, .. } => b,
            Self::Arc { c, r, to, .. } => on_circle(c, r, to),
        }
    }

    /// The segment as points to draw through, its end left out, so that the
    /// segments of a closed outline chain into a polygon with no point
    /// repeated.
    ///
    /// A line is its start. An arc is its start and the points that cut it
    /// into `2^halvings` equal pieces, found by bisecting its end directions
    /// and never by an angle, so the drawing costs no transcendental and is
    /// the same on every machine. **For drawing only**: nothing that touches
    /// reads it, and a contact reads the arc itself.
    pub fn drawn(&self, halvings: u32) -> Vec<Point> {
        match *self {
            Self::Line { a, .. } => vec![a],
            Self::Arc { c, r, from, to } => {
                let mut directions = vec![from, to];
                for _ in 0..halvings {
                    let mut finer = Vec::with_capacity(2 * directions.len() - 1);
                    for pair in directions.windows(2) {
                        finer.push(pair[0]);
                        finer.push(pair[0].halfway_to(pair[1]));
                    }
                    finer.push(to);
                    directions = finer;
                }
                directions.pop();
                directions.into_iter().map(|d| on_circle(c, r, d)).collect()
            }
        }
    }
}

/// The effective bearing of a key.
///
/// A key does not pivot on its balance rail pin: it pivots on the *bearing
/// surface of the punching*, and where that bearing sits is adjustable. Stanwood
/// documents cutting the front half of the balance rail punching away to shift
/// the effective bearing point behind the pin, which drops the action ratio by
/// 0.3 to 0.4 — a deliberate, reversible regulation action. Modelling the pivot
/// as the pin would make that whole technique inexpressible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyBearing {
    /// Offset of the effective bearing point from the balance rail pin, metres.
    /// Positive towards the back of the key.
    pub offset_from_pin: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An arc starts and ends where its stored directions point, at its radius.
    #[test]
    fn an_arc_runs_from_its_first_direction_to_its_second() {
        let arc = Seg::Arc {
            c: Point::new(0.010, 0.020),
            r: 0.0005,
            from: Dir::X,
            to: Dir::Y,
        };
        let (start, end) = (arc.start(), arc.end());
        assert_eq!(start.x.to_bits(), (0.010_f64 + 0.0005).to_bits());
        assert_eq!(start.y.to_bits(), (0.020_f64 + 0.0).to_bits());
        assert_eq!(end.x.to_bits(), (0.010_f64 + 0.0).to_bits());
        assert_eq!(end.y.to_bits(), (0.020_f64 + 0.0005).to_bits());
    }

    /// Consistency: every point drawn along an arc lies on its circle, the
    /// points turn counter-clockwise from its start, and there are as many as
    /// the halvings ask for.
    ///
    /// The bound is derived: each bisected direction is normalized, within a
    /// few ulps of unit length, and the point `c + r·d` adds two roundings of
    /// the coordinates' size; 16 ε of the reach covers both.
    #[test]
    fn an_arc_is_drawn_on_its_circle_counter_clockwise() {
        let c = Point::new(-0.2, 0.022);
        let r = 0.0005;
        for (from, to) in [
            (Dir::X, Dir::Y),
            (Dir::Y, Dir::NEG_X),
            (Dir::X, Dir::NEG_X),
            (Dir::NEG_Y, Dir::new(1.0, -1.0).unwrap()),
        ] {
            let arc = Seg::Arc { c, r, from, to };
            let points = arc.drawn(3);
            assert_eq!(points.len(), 8);
            assert_eq!(points[0], arc.start());
            let reach = c.x.abs() + c.y.abs() + r;
            let mut previous: Option<Point> = None;
            for p in points.iter().copied().chain([arc.end()]) {
                let (dx, dy) = (p.x - c.x, p.y - c.y);
                let off = ((dx * dx + dy * dy).sqrt() - r).abs();
                assert!(
                    off <= 16.0 * f64::EPSILON * reach,
                    "{p:?} is {off:e} off the circle"
                );
                if let Some(q) = previous {
                    let turn = (q.x - c.x) * dy - (q.y - c.y) * dx;
                    assert!(turn > 0.0, "{q:?} to {p:?} does not turn counter-clockwise");
                }
                previous = Some(p);
            }
        }
    }

    /// A line is drawn as its start, which the next segment's start follows.
    #[test]
    fn a_line_is_drawn_as_its_start() {
        let line = Seg::Line {
            a: Point::new(0.0, 0.0),
            b: Point::new(1.0, 0.0),
        };
        assert_eq!(line.drawn(3), vec![Point::new(0.0, 0.0)]);
    }
}
