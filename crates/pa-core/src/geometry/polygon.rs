//! Outlines as the closed polygons they are drawn as: what the overlap
//! readings read a part's material by ([`crate::contact`]), and what the
//! validation layer reads an outline by ([`crate::validate`]). One set of
//! functions for both, so that a part refused for its outline and a part
//! read for its overlaps are read alike.

use super::{Point, Seg, DRAWN_HALVINGS};

/// An outline as the polygon it is drawn as, its arcs in their drawn chords
/// ([`Seg::drawn`]).
pub fn drawn(outline: &[Seg]) -> Vec<Point> {
    outline
        .iter()
        .flat_map(|seg| seg.drawn(DRAWN_HALVINGS))
        .collect()
}

/// Whether the polygon through `corners` encloses `point`: an odd count of
/// its edges that a ray from `point` along `x` crosses.
pub fn encloses(corners: &[Point], point: Point) -> bool {
    let mut odd = false;
    let count = corners.len();
    for index in 0..count {
        let (a, b) = (corners[index], corners[(index + 1) % count]);
        if (a.y > point.y) != (b.y > point.y) {
            let x = a.x + (point.y - a.y) * (b.x - a.x) / (b.y - a.y);
            if point.x < x {
                odd = !odd;
            }
        }
    }
    odd
}

/// The least distance from `point` to the edges of the polygon through
/// `corners`, meters: infinite for a polygon with no edge.
pub fn distance_to_edges(corners: &[Point], point: Point) -> f64 {
    let count = corners.len();
    (0..count)
        .map(|index| distance_to_edge(corners[index], corners[(index + 1) % count], point))
        .fold(f64::INFINITY, f64::min)
}

/// The distance from `point` to the edge from `a` to `b`, meters: to its
/// nearest point, an end where the foot of the perpendicular falls off it.
fn distance_to_edge(a: Point, b: Point, point: Point) -> f64 {
    let (ex, ey) = (b.x - a.x, b.y - a.y);
    let squared = ex * ex + ey * ey;
    let along = if squared > 0.0 {
        (((point.x - a.x) * ex + (point.y - a.y) * ey) / squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (point.x - a.x - along * ex, point.y - a.y - along * ey);
    (dx * dx + dy * dy).sqrt()
}

/// Whether the polygon through `corners` crosses itself: two of its edges
/// that do not meet end to end cross, each running from one side of the
/// other to its other side.
pub fn crosses_itself(corners: &[Point]) -> bool {
    let count = corners.len();
    let turn = |a: Point, b: Point, c: Point| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    (0..count).any(|first| {
        let (p, q) = (corners[first], corners[(first + 1) % count]);
        (first + 2..count)
            .filter(|&second| (second + 1) % count != first)
            .any(|second| {
                let (r, s) = (corners[second], corners[(second + 1) % count]);
                turn(r, s, p) * turn(r, s, q) < 0.0 && turn(p, q, r) * turn(p, q, s) < 0.0
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The square of side 2 m with its corner at the origin: every number
    /// below is exact in binary, so the distances are read to the bit.
    fn square() -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(0.0, 2.0),
        ]
    }

    /// Consistency: **a polygon encloses a point inside it and not one
    /// beside it**: 0.5 m in from the square's left side, and 1 m to the
    /// right of the square.
    #[test]
    fn a_polygon_encloses_a_point_inside_it_and_not_one_beside_it() {
        assert!(encloses(&square(), Point::new(0.5, 1.0)));
        assert!(!encloses(&square(), Point::new(3.0, 1.0)));
    }

    /// Consistency: **a point stands from a polygon's edges as far as from
    /// the nearest**, inside or outside: 0.5 m from the left side for the
    /// point inside, 1 m from the right side for the one beside it.
    #[test]
    fn a_point_stands_from_a_polygons_edges_as_far_as_from_the_nearest() {
        assert_eq!(distance_to_edges(&square(), Point::new(0.5, 1.0)), 0.5);
        assert_eq!(distance_to_edges(&square(), Point::new(3.0, 1.0)), 1.0);
    }

    /// Consistency: **a square does not cross itself, and the same corners
    /// taken in a bow-tie's order do.**
    #[test]
    fn a_bow_tie_crosses_itself_and_a_square_does_not() {
        let mut bow_tie = square();
        bow_tie.swap(2, 3);
        assert!(!crosses_itself(&square()));
        assert!(crosses_itself(&bow_tie));
    }
}
