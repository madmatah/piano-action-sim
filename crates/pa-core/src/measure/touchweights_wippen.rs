//! The touchweights of a key carrying its wippen, on the test wippen of
//! [`crate::test_fixture`]: D, U, BW, F and the band read through the
//! wippen the key carries, against the band worked by hand, and the
//! verdicts on the folded coordinates. Tests only.
//!
//! The hand works each attitude from the drawn parts and the masses alone:
//! the wippen's angle where the two pieces its link rests on touch, found by
//! halving their gap written out as two discs or a disc and a line; how it
//! follows the key's angle, by the central difference of that angle; and the
//! generalized forces of the two weights, each mass's arm about its pivot
//! ([`Carrying`]).

use crate::contact::{Feature, PartCode, Piece, Rows, ROW_MARGIN};
use crate::geometry::{Point, Seg};
use crate::measure::oracle::{weight_on_its_angle, Branches, Carrying};
use crate::measure::seat::{Seat, Seating, Seats, SEAT_MARGIN};
use crate::measure::sweep::{Band, Sample};
use crate::measure::walk::{Breakaway, Stroke, HALVINGS, STEP_G};
use crate::mechanism::{AxisFriction, MaterialPoint, Prescribed};
use crate::parts::fixtures::{mass_of, on_frame_carrying, regulated, test_wippen, OnFrame, Setup};
use crate::parts::mounts::{Motion, Mounted};
use crate::parts::verdicts::TOUCHING;
use crate::parts::{WippenAssembly, WippenAssemblyMass};
use crate::solve::statics::RESIDUAL_TOLERANCE;
use crate::solve::{Friction, Statics};
use crate::G;

/// The bushings of the test bench: 6 g at the measuring point with the key
/// level, as the bare key's tests weigh it.
const KEY_FRICTION_G: f64 = 6.0;

/// A key on its frame carrying a wippen, as the bench mounts it.
struct Bench {
    a: Setup,
    wippen: WippenAssembly,
    wippen_mass: WippenAssemblyMass,
    k: OnFrame,
    /// The bushings' bound, newton-meters.
    bushings: f64,
}

impl Bench {
    /// `a`'s key carrying `wippen` of `wippen_mass`, its bushings holding
    /// `kf_g` grams at its measuring point with the key level.
    fn new(a: &Setup, wippen: WippenAssembly, wippen_mass: WippenAssemblyMass, kf_g: f64) -> Self {
        let bushings = AxisFriction::of_weight_at(kf_g, a.measuring_point()).bound_n_m;
        let k = on_frame_carrying(a, bushings, true, (&wippen, &wippen_mass));
        Self {
            a: a.clone(),
            wippen,
            wippen_mass,
            k,
            bushings,
        }
    }

    fn seating(&self) -> Seating<'_> {
        Seating {
            mounted: &self.k.mounted,
            capstan: self.k.capstan,
            measuring_point: self.k.measuring_point,
            margin: SEAT_MARGIN,
            standing: &[],
        }
    }

    fn mounted(&self) -> &Mounted {
        &self.k.mounted
    }

    fn seats(&self) -> Seats {
        self.seating()
            .seats(&mut Statics::new())
            .expect("the key carrying its wippen seats")
    }

    /// The walk's D and U from the seats.
    fn walks(&self, seats: &Seats) -> [Breakaway; 2] {
        let mut statics = Statics::new();
        [Stroke::Down, Stroke::Up].map(|stroke| {
            self.seating()
                .walk(&mut statics, seats, stroke)
                .expect("the key carrying its wippen walks")
        })
    }

    fn band(&self, seats: &Seats) -> Band {
        self.seating()
            .sweep(&mut Statics::new(), seats)
            .expect("the band is drawn")
    }

    /// The wippen center's friction bound, newton-meters.
    fn wippen_friction(&self) -> f64 {
        self.wippen.friction.bound_n_m
    }

    /// The pieces the link that carries the wippen at `q` rests it on, by
    /// `motion` where two link rows are closed: going down the one whose
    /// hand-worked ratio is the greater, coming up the lesser.
    fn carrying(&self, q: &[f64], motion: Motion) -> Feature {
        let m = self.mounted();
        let w = m.wippen.expect("a wippen");
        let mut rows = Rows::new();
        m.mechanism
            .rows(&m.mechanism.pose(q), ROW_MARGIN, &mut rows);
        let closed: Vec<Feature> = rows
            .rows()
            .iter()
            .filter(|row| w.links.contains(&row.pair) && row.gap <= TOUCHING)
            .map(|row| row.feature)
            .collect();
        let ratio = |f: &Feature| self.follows_by_hand(q, *f).abs();
        closed
            .into_iter()
            .reduce(|a, b| {
                let greater = ratio(&b) > ratio(&a);
                match motion {
                    Motion::Down if greater => b,
                    Motion::Up if !greater && ratio(&b) < ratio(&a) => b,
                    _ => a,
                }
            })
            .expect("a link carries the wippen")
    }

    /// The gap the two pieces of `feature` leave at `q`, by hand: two discs,
    /// a vertex a disc of no radius, as far apart as their centers less their
    /// radii; a disc and a line, as far as the center stands outside the
    /// line, on the right of a drawn outline that runs counter-clockwise,
    /// less the radius.
    fn gap_by_hand(&self, q: &[f64], feature: Feature) -> f64 {
        let shape = |piece: Piece| self.shape(q, piece);
        match (shape(feature.a), shape(feature.b)) {
            (Shape::Disc(c, r), Shape::Disc(d, s)) => distance(c, d) - r - s,
            (Shape::Disc(c, r), Shape::Line(p, l)) | (Shape::Line(p, l), Shape::Disc(c, r)) => {
                let (dx, dy) = (l.x - p.x, l.y - p.y);
                ((c.x - p.x) * dy - (c.y - p.y) * dx) / (dx * dx + dy * dy).sqrt() - r
            }
            (Shape::Line(..), Shape::Line(..)) => unreachable!("two faces meet at a corner"),
        }
    }

    /// The drawn piece `piece` written out in the world at `q`: an arc as
    /// its disc, a vertex as a disc of no radius, a segment as its line.
    fn shape(&self, q: &[f64], piece: Piece) -> Shape {
        let m = &self.mounted().mechanism;
        let pose = m.pose(q);
        let part = &m.parts()[piece.part().index()];
        let at = pose.placement(part.body);
        let silhouette = part.silhouette();
        match piece {
            Piece::Arc { seg, .. } => match silhouette[usize::from(seg)] {
                Seg::Arc { c, r, .. } => Shape::Disc(at.world(c), r),
                Seg::Line { .. } => unreachable!("an arc's segment"),
            },
            Piece::Vertex { seg, .. } => {
                Shape::Disc(at.world(silhouette[usize::from(seg)].start()), 0.0)
            }
            Piece::Segment { seg, .. } => {
                let line = silhouette[usize::from(seg)];
                Shape::Line(at.world(line.start()), at.world(line.end()))
            }
            Piece::Center { .. } => unreachable!("a link reads no center"),
        }
    }

    /// Where the two pieces of `feature` meet at `q`, by hand, in the world:
    /// side A's surface point, then side B's. A disc on a line meets it at
    /// the foot of the perpendicular from its center, the disc's own point
    /// its radius from the center toward the line; two discs meet on the
    /// line of their centers, each surface its radius from its own center.
    /// The gap moves neither point: they are the two surfaces' points along
    /// the normal at `q`, whatever the gap.
    fn hand_contact(&self, q: &[f64], feature: Feature) -> (Point, Point) {
        let on_a_line = |c: Point, r: f64, p: Point, l: Point| {
            let (dx, dy) = (l.x - p.x, l.y - p.y);
            let length = (dx * dx + dy * dy).sqrt();
            let (tx, ty) = (dx / length, dy / length);
            let along = (c.x - p.x) * tx + (c.y - p.y) * ty;
            let foot = Point::new(p.x + along * tx, p.y + along * ty);
            // The outline runs counter-clockwise, so the material's outside
            // stands on the line's right.
            let (nx, ny) = (ty, -tx);
            (Point::new(c.x - r * nx, c.y - r * ny), foot)
        };
        match (self.shape(q, feature.a), self.shape(q, feature.b)) {
            (Shape::Disc(c, r), Shape::Disc(d, t)) => {
                let length = distance(c, d);
                let (ux, uy) = ((d.x - c.x) / length, (d.y - c.y) / length);
                (
                    Point::new(c.x + r * ux, c.y + r * uy),
                    Point::new(d.x - t * ux, d.y - t * uy),
                )
            }
            (Shape::Disc(c, r), Shape::Line(p, l)) => on_a_line(c, r, p, l),
            (Shape::Line(p, l), Shape::Disc(c, r)) => {
                let (on_the_disc, foot) = on_a_line(c, r, p, l);
                (foot, on_the_disc)
            }
            (Shape::Line(..), Shape::Line(..)) => unreachable!("two faces meet at a corner"),
        }
    }

    /// The normal along which the two pieces of `feature` meet at `q`, by
    /// hand, a unit vector in the world, either way along it: two discs'
    /// line of centers, or the normal of the line a disc meets.
    fn hand_normal(&self, q: &[f64], feature: Feature) -> Point {
        let unit = |dx: f64, dy: f64| {
            let length = (dx * dx + dy * dy).sqrt();
            Point::new(dx / length, dy / length)
        };
        match (self.shape(q, feature.a), self.shape(q, feature.b)) {
            (Shape::Disc(c, _), Shape::Disc(d, _)) => unit(d.x - c.x, d.y - c.y),
            (Shape::Disc(..), Shape::Line(p, l)) | (Shape::Line(p, l), Shape::Disc(..)) => {
                unit(l.y - p.y, p.x - l.x)
            }
            (Shape::Line(..), Shape::Line(..)) => unreachable!("two faces meet at a corner"),
        }
    }

    /// The wippen's angle at which the pieces of `feature` touch with the
    /// key's coordinates of `q`, by hand: halved within a hundredth of a
    /// radian of `q`'s, the heel lifting off the key as the wippen turns up.
    fn wippen_by_hand(&self, q: &[f64], feature: Feature) -> f64 {
        let w = self.mounted().wippen.expect("a wippen").angle.0;
        let mut trial = q.to_vec();
        let (mut low, mut high) = (q[w] - 0.01, q[w] + 0.01);
        // Sixty halvings of two hundredths of a radian leave less than the
        // last bit of the angle.
        for _ in 0..60 {
            let middle = 0.5 * (low + high);
            trial[w] = middle;
            if self.gap_by_hand(&trial, feature) > 0.0 {
                high = middle;
            } else {
                low = middle;
            }
        }
        0.5 * (low + high)
    }

    /// How the wippen's angle follows the key's at `q` through the pieces of
    /// `feature`, by hand: keeping their gap as it stands,
    /// `−(∂g/∂θk)/(∂g/∂θw)`, each derivative the central difference of the
    /// gap over `h` radians either way.
    fn follows_by(&self, q: &[f64], feature: Feature, h: f64) -> f64 {
        self.follows_on(q, feature, self.mounted().angle.0, h)
    }

    /// How the wippen's angle follows the key's coordinate `coord` at `q`
    /// through the pieces of `feature`, by hand: keeping their gap as it
    /// stands, `−(∂g/∂q_c)/(∂g/∂θw)`, each derivative the central difference
    /// of the gap over `h` either way, radians or meters.
    fn follows_on(&self, q: &[f64], feature: Feature, coord: usize, h: f64) -> f64 {
        let w = self.mounted().wippen.expect("a wippen").angle.0;
        let along = |coord: usize| {
            let moved = |by: f64| {
                let mut trial = q.to_vec();
                trial[coord] += by;
                self.gap_by_hand(&trial, feature)
            };
            moved(h) - moved(-h)
        };
        -along(coord) / along(w)
    }

    /// The pieces the wippen stands on at `q`: of the link rows closed
    /// within touching, the one whose gap is the least.
    fn resting(&self, q: &[f64]) -> Feature {
        let m = self.mounted();
        let w = m.wippen.expect("a wippen");
        let mut rows = Rows::new();
        m.mechanism
            .rows(&m.mechanism.pose(q), ROW_MARGIN, &mut rows);
        rows.rows()
            .iter()
            .filter(|row| w.links.contains(&row.pair) && row.gap <= TOUCHING)
            .min_by(|a, b| a.gap.total_cmp(&b.gap))
            .expect("a link carries the wippen")
            .feature
    }

    /// [`Self::follows_by`] over a microradian of the key either way.
    fn follows_by_hand(&self, q: &[f64], feature: Feature) -> f64 {
        self.follows_by(q, feature, FOLLOWS_STEP)
    }

    /// The key carrying its wippen at the key's coordinates of `q`, by hand:
    /// the wippen where it stands on the pieces it rests on
    /// ([`Self::resting`]), carried through the pieces of `feature`.
    fn by_hand(&self, q: &[f64], feature: Feature) -> Carrying {
        let m = self.mounted();
        let w = m.wippen.expect("a wippen");
        let mut at = q.to_vec();
        at[w.angle.0] = self.wippen_by_hand(q, self.resting(q));
        let pose = m.mechanism.pose(&at);
        let key_mass = mass_of(&self.a.key).expect("the test key is weighed");
        let wippen_mass = self.wippen_mass.props(&self.wippen);
        let bearing = pose.world(MaterialPoint {
            body: m.key,
            at: Point::new(0.0, 0.0),
        });
        let world = |body, p| pose.world(MaterialPoint { body, at: p });
        Carrying {
            key_force_n_m: weight_on_its_angle(key_mass.mass, world(m.key, key_mass.com), bearing),
            wippen_force_n_m: weight_on_its_angle(
                wippen_mass.mass,
                world(w.body, wippen_mass.com),
                self.wippen.center,
            ),
            follows: self.follows_by_hand(&at, feature),
            bushings_n_m: self.bushings,
            wippen_friction_n_m: self.wippen_friction(),
            measuring_arm_m: world(m.key, self.k.measuring_point).x - bearing.x,
        }
    }

    /// How far the hand's ratio may stand from the one it stands for, at
    /// `q` through `feature`: the central differences' error, `h²/6` times
    /// the third derivative, which halving `h` divides by four, so a third
    /// of what the two steps read apart; and the rounding of the gaps they
    /// take apart, a few units in the last place of lengths under 0.3 m,
    /// over the difference of the gap across the wippen's step, its arm of
    /// at least 50 mm, both derivatives' carried through the ratio.
    fn follows_within(&self, q: &[f64], feature: Feature) -> f64 {
        let (whole, half) = (
            self.follows_by(q, feature, 2.0 * FOLLOWS_STEP),
            self.follows_by(q, feature, FOLLOWS_STEP),
        );
        let rounding = 4.0 * f64::EPSILON * 0.3 / (2.0 * FOLLOWS_STEP * 0.05);
        (whole - half).abs() / 3.0 + rounding * (1.0 + half.abs())
    }

    /// How far a reading of the band, worked by the procedure and by hand,
    /// may stand apart at `q`, grams: what the static solve leaves out of
    /// balance on each coordinate, its residual's tolerance of the load
    /// scale, under two newtons, over its reach, under 0.3 m, the wippen's
    /// carried through the ratio onto the key's; the ratio's error times
    /// the wippen's weight and friction; and rounding, 64 units in the last
    /// place of the terms, all read at the measuring point's arm, at least
    /// 0.2 m.
    fn hand_within(&self, q: &[f64], feature: Feature, hand: &Carrying) -> f64 {
        let ratio = hand.follows.abs();
        let decided = RESIDUAL_TOLERANCE * 2.0 * 0.3 * (1.0 + ratio);
        let follows = self.follows_within(q, feature)
            * (hand.wippen_force_n_m.abs() + hand.wippen_friction_n_m);
        let terms = hand.key_force_n_m.abs()
            + ratio * hand.wippen_force_n_m.abs()
            + hand.bushings_n_m
            + ratio * hand.wippen_friction_n_m;
        let rounding = 64.0 * f64::EPSILON * terms;
        1000.0 * (decided + follows + rounding) / (G * 0.2)
    }

    /// The states the sweep holds the key at, as it holds them: each step's
    /// angle from where the last left the key, friction on, the rest solved.
    fn states(&self, from: &[f64], curve: &[Sample]) -> Vec<Vec<f64>> {
        let m = self.mounted();
        let mut statics = Statics::new();
        let mut q = from.to_vec();
        curve
            .iter()
            .map(|s| {
                let held = [Prescribed {
                    coord: m.angle,
                    value: s.angle,
                }];
                q = m
                    .solve_held(&mut statics, &q, &[], &held, Friction::On)
                    .expect("the sweep held the key there")
                    .q;
                q.clone()
            })
            .collect()
    }

    /// The band by hand at the states the sweep holds the key at, each with
    /// how far it may stand from the sweep's reading and the pieces the
    /// wippen rests on there: down, then up.
    fn band_by_hand(&self, seats: &Seats, band: &Band) -> [Vec<ByHand>; 2] {
        let down = self.states(&seats.top.q, &band.down);
        let up = self.states(&seats.bottom.q, &band.up);
        let hand = |states: Vec<Vec<f64>>, motion: Motion| {
            states
                .iter()
                .map(|q| {
                    let feature = self.carrying(q, motion);
                    let hand = self.by_hand(q, feature);
                    let branches = hand.branches();
                    ByHand {
                        grams: match motion {
                            Motion::Down => branches.down_g,
                            Motion::Up => branches.up_g,
                        },
                        within: self.hand_within(q, feature, &hand),
                        feature,
                    }
                })
                .collect()
        };
        [hand(down, Motion::Down), hand(up, Motion::Up)]
    }

    /// The band by hand at `seat`, the wippen resting on the link the walk
    /// from there starts it on.
    fn at_seat(&self, seat: &Seat, motion: Motion) -> (Branches, f64) {
        let feature = self.carrying(&seat.q, motion);
        let hand = self.by_hand(&seat.q, feature);
        (hand.branches(), self.hand_within(&seat.q, feature, &hand))
    }
}

/// One step of the band, by hand.
#[derive(Debug, Clone, Copy)]
struct ByHand {
    /// The force on the measuring point, grams.
    grams: f64,
    /// How far the sweep's reading may stand from it, grams.
    within: f64,
    /// The pieces the wippen rests on.
    feature: Feature,
}

/// How far either way of the key's angle the hand's central difference
/// reaches, radians: a microradian.
const FOLLOWS_STEP: f64 = 1e-6;

/// A piece of a drawn outline, written out in the world.
#[derive(Debug, Clone, Copy)]
enum Shape {
    /// A disc: its center and its radius, a vertex's none.
    Disc(Point, f64),
    /// A line through two points.
    Line(Point, Point),
}

fn distance(a: Point, b: Point) -> f64 {
    ((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt()
}

/// What a walk resolves a threshold to, grams: half the bracket its
/// halvings leave, and what the static solve may leave out of balance where
/// it decides whether the key moves, its residual's tolerance of the load
/// scale, under two newtons, over each coordinate's reach, under 0.3 m, the
/// wippen's carried onto the key's at a ratio under three, read as grams at
/// a measuring arm of at least 0.2 m.
fn walk_resolution() -> f64 {
    let halved = 0.5 * STEP_G / f64::from(1u32 << HALVINGS);
    let decided = 1000.0 * RESIDUAL_TOLERANCE * 2.0 * 0.3 * 3.0 / (G * 0.2);
    halved + decided
}

/// How far the extreme a curve reads may stand from the extreme of the
/// force it samples, grams: the curve's greatest change over one step, and
/// twice its greatest change of slope over one, as the bare key's sweep
/// derives it, piece by piece of the heel the wippen rests on. Where the
/// link passes from one piece to another the force jumps with the ratio,
/// and its extreme stands at the passing, within a step of the first
/// reading past it on the piece it passes onto.
fn within_a_step(hand: &[ByHand]) -> f64 {
    hand.chunk_by(|a, b| a.feature == b.feature)
        .map(|piece| {
            let grams: Vec<f64> = piece.iter().map(|h| h.grams).collect();
            within_a_step_of(&grams)
        })
        .fold(0.0, f64::max)
}

/// [`within_a_step`] over one smooth curve.
fn within_a_step_of(curve: &[f64]) -> f64 {
    let step = curve
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0_f64, f64::max);
    let bend = curve
        .windows(3)
        .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
        .fold(0.0_f64, f64::max);
    step + 2.0 * bend
}

/// A center pin at which the test wippen on a capstan 30 mm tall is pushed
/// past its range at the key's top seat, meters
/// ([`super::seat_wippen::on_a_tall_capstan`]).
const REFUSED_AT_THE_TOP_SEAT: f64 = 0.069;

/// The test bench carrying the test wippen.
fn the_test_bench() -> Bench {
    let (wippen, mass) = test_wippen();
    Bench::new(&regulated(), wippen, mass, KEY_FRICTION_G)
}

/// The test bench with its capstan `behind_mm` behind the balance pin.
fn capstan_at(behind_mm: f64) -> Bench {
    let mut a = regulated();
    a.key.capstan = -behind_mm * 1e-3;
    let (wippen, mass) = test_wippen();
    Bench::new(&a, wippen, mass, KEY_FRICTION_G)
}

/// The test bench with the wippen's center pin raised or lowered to the
/// half-stroke line: the key's bearing, the point where the capstan bears
/// on the heel and the center pin on one straight line at half stroke, the
/// key's angle halfway between its seats. Halved on the height between
/// 70 mm, where the bearing point stands above that line, and 80 mm, where
/// it stands below, to a hundredth of a micron.
fn at_the_half_stroke_line() -> Bench {
    let a = regulated();
    let at = |height: f64| {
        let (mut wippen, mass) = test_wippen();
        wippen.center.y = height;
        Bench::new(&a, wippen, mass, KEY_FRICTION_G)
    };
    let above_the_line = |b: &Bench| {
        let m = b.mounted();
        let seats = b.seats();
        let half = 0.5 * (seats.top.q[m.angle.0] + seats.bottom.q[m.angle.0]);
        let held = [Prescribed {
            coord: m.angle,
            value: half,
        }];
        let q = m
            .solve_held(&mut Statics::new(), &seats.top.q, &[], &held, Friction::Off)
            .expect("the key is held at half stroke")
            .q;
        let pose = m.mechanism.pose(&q);
        let mut rows = Rows::new();
        m.mechanism.rows(&pose, ROW_MARGIN, &mut rows);
        let links = m.wippen.expect("a wippen").links;
        let bears = rows
            .rows()
            .iter()
            .find(|row| links.contains(&row.pair) && row.gap <= TOUCHING)
            .expect("the capstan bears on the heel")
            .point;
        let bearing = pose.world(MaterialPoint {
            body: m.key,
            at: Point::new(0.0, 0.0),
        });
        let pin = b.wippen.center;
        let along = (bears.x - bearing.x) / (pin.x - bearing.x);
        bears.y - (bearing.y + along * (pin.y - bearing.y))
    };
    let (mut low, mut high) = (0.070, 0.080);
    while high - low > 1e-8 {
        let middle = 0.5 * (low + high);
        if above_the_line(&at(middle)) > 0.0 {
            low = middle;
        } else {
            high = middle;
        }
    }
    at(0.5 * (low + high))
}

/// The capstan's place behind the balance pin, millimeters, at which the
/// test bench's capstan, at its bottom seat, bears in the valley where the
/// heel cloth's back end meets the heel's bottom, on both: halved between
/// 127 mm, where it bears on the cloth there, and 128 mm, on the heel's
/// wood, to the last bits, and taken on the cloth's side.
fn in_the_valley() -> Bench {
    let on_the_cloth = |b: &Bench, q: &[f64]| {
        crate::parts::verdicts::wippen_carried_by(b.mounted(), q)
            .iter()
            .any(|c| c.parts.contains(&PartCode::WippenHeelCloth))
    };
    let (mut low, mut high) = (127.0, 128.0);
    // Sixty halvings of a millimeter leave less than the last bit of the
    // place.
    for _ in 0..60 {
        let middle = 0.5 * (low + high);
        let b = capstan_at(middle);
        if on_the_cloth(&b, &b.seats().bottom.q) {
            low = middle;
        } else {
            high = middle;
        }
    }
    capstan_at(low)
}

/// The benches the band is checked on by hand: the test bench; the same
/// with its center pin at the half-stroke line; and with its capstan
/// 129 mm behind the balance pin, where the heel passes from its cloth onto
/// its wood partway down the stroke.
fn benches() -> Vec<(&'static str, Bench)> {
    vec![
        ("the test bench", the_test_bench()),
        ("at the half-stroke line", at_the_half_stroke_line()),
        ("the capstan at 129 mm", capstan_at(129.0)),
    ]
}

/// Consistency, by hand: **every step of the band of a key carrying its
/// wippen is the band by hand at the state it holds**,
/// `(±(b_k + |r|·b_w) + Q_k + r·Q_w) / (g a)` ([`Carrying`]): the key's
/// weight and the wippen's, the wippen's folded onto the key's angle by how
/// its angle follows the key's through the pieces its link rests on, and
/// its center's friction carried the same way onto the bushings'. The
/// procedure reads the reaction on the key's angle the static solve
/// leaves; the hand works the wippen's angle and its ratio from the drawn
/// pieces and the forces from the masses. Each step's bound is derived
/// from the solve's residual, the ratio's central difference and rounding
/// ([`Bench::hand_within`]), a few billionths of a gram.
#[test]
fn every_step_of_the_band_carrying_the_wippen_is_the_band_by_hand() {
    for (name, b) in benches() {
        let seats = b.seats();
        let band = b.band(&seats);
        let [down, up] = b.band_by_hand(&seats, &band);
        for (curve, hand) in [(&band.down, &down), (&band.up, &up)] {
            assert_eq!(curve.len(), hand.len());
            for (s, h) in curve.iter().zip(hand) {
                assert!(
                    (s.grams - h.grams).abs() <= h.within,
                    "{name}: {} g at {} mm, by hand {} g, within {} g",
                    s.grams,
                    s.travel * 1000.0,
                    h.grams,
                    h.within
                );
            }
        }
    }
}

/// Consistency, by hand: **D and U of a key carrying its wippen are the
/// extremes of the band by hand over the window**, sampled at the sweep's
/// step: the greatest force the down curve needs, the least the up curve
/// holds, wherever they fall, within the walk's resolution, the curve's
/// change over a step piece by piece of the heel ([`within_a_step`]) and
/// the hand's own bound.
#[test]
fn down_and_up_carrying_the_wippen_are_the_extremes_of_the_band_by_hand() {
    for (name, b) in benches() {
        let seats = b.seats();
        let band = b.band(&seats);
        let [down, up] = b.band_by_hand(&seats, &band);
        let [d, u] = b.walks(&seats).map(|walk| walk.grams);
        let within = |hand: &[ByHand]| {
            walk_resolution()
                + within_a_step(hand)
                + hand.iter().map(|h| h.within).fold(0.0, f64::max)
        };
        let greatest = down.iter().map(|h| h.grams).fold(f64::MIN, f64::max);
        let least = up.iter().map(|h| h.grams).fold(f64::MAX, f64::min);
        assert!(
            (d - greatest).abs() <= within(&down),
            "{name}: D {d} g, the band by hand {greatest} g, within {} g",
            within(&down)
        );
        assert!(
            (u - least).abs() <= within(&up),
            "{name}: U {u} g, the band by hand {least} g, within {} g",
            within(&up)
        );
    }
}

/// Consistency, by hand: **on the test bench, D and U of a key carrying its
/// wippen are the band by hand at the seats' attitudes**, the band falling
/// all the way down the stroke: D where the weight starts the key and its
/// wippen down from the top seat, U where it lets them rise from the bottom
/// one, each through the link the wippen rests on there, within the walk's
/// resolution and the hand's bound.
#[test]
fn down_and_up_carrying_the_wippen_are_the_band_by_hand_at_the_seats_attitudes() {
    let b = the_test_bench();
    let seats = b.seats();
    let [d, u] = b.walks(&seats).map(|walk| walk.grams);
    let (top, top_within) = b.at_seat(&seats.top, Motion::Down);
    let (bottom, bottom_within) = b.at_seat(&seats.bottom, Motion::Up);
    assert!(
        (d - top.down_g).abs() <= walk_resolution() + top_within,
        "D {d} g, by hand {} g",
        top.down_g
    );
    assert!(
        (u - bottom.up_g).abs() <= walk_resolution() + bottom_within,
        "U {u} g, by hand {} g",
        bottom.up_g
    );
}

/// Consistency, by hand: **at the half-stroke line D is the band's greatest
/// force by hand, where it falls.** With the center pin on the half-stroke
/// line the heel rolls on the capstan at half stroke, and the band by hand
/// still falls all the way down the stroke on the test bench, the key's own
/// balance and the wippen's arm about its pin changing more than the ratio:
/// its greatest force falls on the first step off the top seat, and D is
/// that force within a step of it.
#[test]
fn at_the_half_stroke_line_down_weight_is_the_bands_greatest_force_by_hand() {
    let b = at_the_half_stroke_line();
    let seats = b.seats();
    let band = b.band(&seats);
    let [down, _] = b.band_by_hand(&seats, &band);
    let [d, _] = b.walks(&seats).map(|walk| walk.grams);
    let (at, greatest) =
        down.iter()
            .enumerate()
            .map(|(at, h)| (at, h.grams))
            .fold(
                (0, f64::MIN),
                |best, next| if next.1 > best.1 { next } else { best },
            );
    let within = walk_resolution() + within_a_step(&down) + down[at].within;
    assert_eq!(at, 0, "the greatest force falls off the top seat");
    assert!(
        (d - greatest).abs() <= within,
        "D {d} g, the band's greatest force by hand {greatest} g"
    );
}

/// Consistency, by hand: **where the heel passes from its cloth onto its
/// wood partway down the stroke, the band and the walks follow it piece by
/// piece.** With the capstan 129 mm behind the balance pin it bears on the
/// heel cloth at the top seat and on the heel's wood behind the cloth at the
/// bottom one: the ratio jumps where the bearing passes the cloth's back
/// end, and the force with it. D is the greatest force the down curve needs
/// there, inside the window, and U the least the up curve holds, at the same
/// place: the walk stalls at the passing and goes on past it.
#[test]
fn where_the_heel_passes_from_its_cloth_onto_its_wood_the_band_and_the_walks_follow_it() {
    let b = capstan_at(129.0);
    let seats = b.seats();
    let band = b.band(&seats);
    let [down, up] = b.band_by_hand(&seats, &band);
    let part = |h: &ByHand| b.mounted().mechanism.parts()[h.feature.b.part().index()].code;
    let first = down.first().expect("a reading");
    let last = down.last().expect("a reading");
    assert_eq!(part(first), PartCode::WippenHeelCloth);
    assert_eq!(part(last), PartCode::WippenHeel);
    let passing = down
        .iter()
        .position(|h| part(h) == PartCode::WippenHeel)
        .expect("the heel passes onto its wood");
    let [d, u] = b.walks(&seats);
    let greatest = down.iter().map(|h| h.grams).fold(f64::MIN, f64::max);
    assert_eq!(down[passing].grams.to_bits(), greatest.to_bits());
    let within = walk_resolution() + within_a_step(&down) + down[passing].within;
    assert!(
        (d.grams - greatest).abs() <= within,
        "D {} g, by hand {greatest} g at the passing",
        d.grams
    );
    assert!(d.stalls >= 1, "the walk stalls at the passing");
    let least = up.iter().map(|h| h.grams).fold(f64::MAX, f64::min);
    let within =
        walk_resolution() + within_a_step(&up) + up.iter().map(|h| h.within).fold(0.0, f64::max);
    assert!(
        (u.grams - least).abs() <= within,
        "U {} g, by hand {least} g",
        u.grams
    );
}

/// Consistency, by hand: **with two link rows closed, the wippen is carried
/// down through the larger ratio and up through the smaller.** At its bottom
/// seat the capstan of [`in_the_valley`] bears on the heel cloth and on the
/// heel's wood at once, two rows of one link, each with its own ratio by
/// hand. Going down, the key's back rising, the row that lifts the wippen
/// most carries it; coming up, the wippen stays on the one that drops
/// least. So the fold is the larger ratio going down and the smaller coming
/// up; the up walk starts from the frictionless balance through the
/// smaller; and the verdict reads its edge toward the key's rising through
/// the smaller, its margin there, and its edge toward its going down
/// through the larger: a weight a little past the down weight that ratio
/// gives rests the key on its front punching, one a little short leaves it
/// held by its friction.
#[test]
fn two_link_rows_closed_carry_the_wippen_down_through_the_larger_ratio_and_up_through_the_smaller()
{
    use crate::parts::verdicts::{holding, Holding};
    let b = in_the_valley();
    let m = b.mounted();
    let seats = b.seats();
    let q = &seats.bottom.q;
    let w = m.wippen.expect("a wippen");
    let mut rows = Rows::new();
    m.mechanism
        .rows(&m.mechanism.pose(q), ROW_MARGIN, &mut rows);
    let closed: Vec<Feature> = rows
        .rows()
        .iter()
        .filter(|row| w.links.contains(&row.pair) && row.gap <= TOUCHING)
        .map(|row| row.feature)
        .collect();
    assert_eq!(
        closed.len(),
        2,
        "the capstan bears on the cloth and the wood"
    );
    let ratios: Vec<f64> = closed.iter().map(|&f| b.follows_by_hand(q, f)).collect();
    let within = closed
        .iter()
        .map(|&f| b.follows_within(q, f))
        .fold(0.0, f64::max);
    let (larger, smaller) = if ratios[0].abs() > ratios[1].abs() {
        (ratios[0], ratios[1])
    } else {
        (ratios[1], ratios[0])
    };
    assert!(
        larger.abs() - smaller.abs() > 100.0 * within,
        "the two rows' ratios {larger} and {smaller} stand apart"
    );
    let down = m.wippen_follows(q, Motion::Down, TOUCHING);
    let up = m.wippen_follows(q, Motion::Up, TOUCHING);
    assert!(
        (down - larger).abs() <= within,
        "down {down}, by hand {larger}"
    );
    assert!((up - smaller).abs() <= within, "up {up}, by hand {smaller}");

    let rising = b.by_hand(q, b.carrying(q, Motion::Up));
    let falling = b.by_hand(q, b.carrying(q, Motion::Down));
    let frictionless = Carrying {
        bushings_n_m: 0.0,
        wippen_friction_n_m: 0.0,
        ..rising
    }
    .branches()
    .down_g;
    let [_, walk_up] = b.walks(&seats);
    let hand_within = |hand: &Carrying| b.hand_within(q, b.carrying(q, Motion::Up), hand);
    assert!(
        (walk_up.from_g - frictionless).abs() <= hand_within(&rising),
        "the up walk starts from {} g, by hand {frictionless} g",
        walk_up.from_g
    );

    let (up_g, down_g) = (rising.branches().up_g, falling.branches().down_g);
    let inside = 0.5 * (up_g + down_g);
    let per_gram = 1e-3 * G * rising.measuring_arm_m;
    let margin = rising.bushings_n_m + rising.follows.abs() * rising.wippen_friction_n_m
        - (rising.key_force_n_m + rising.follows * rising.wippen_force_n_m - inside * per_gram);
    let weight = |grams| [b.seating().weight(grams)];
    match holding(m, q, &weight(inside)) {
        Some(Holding::HeldByFriction {
            touching: Some(_),
            margin: read,
        }) => assert!(
            (read - margin).abs() <= hand_within(&rising) * per_gram,
            "a margin of {read} N·m, by hand {margin} N·m"
        ),
        other => panic!("{other:?}"),
    }

    let other_ratio = Carrying {
        follows: smaller,
        ..falling
    }
    .branches()
    .down_g;
    let apart = (down_g - other_ratio).abs();
    assert!(
        apart > 1e-3,
        "the two ratios' down weights stand {apart} g apart"
    );
    let past = 0.25 * apart;
    assert!(matches!(
        holding(m, q, &weight(down_g + past)),
        Some(Holding::RestingOn { .. })
    ));
    assert!(matches!(
        holding(m, q, &weight(down_g - past)),
        Some(Holding::HeldByFriction { .. })
    ));
}

/// Consistency: **a step half as large finds the same weight with the
/// wippen.** Walked by half grams, D and U of the key carrying its wippen
/// agree with the walk by grams within the two walks' resolutions, where
/// the band is smooth through the stroke: on the test bench and at the
/// half-stroke line. Where the heel passes from its cloth onto its wood the
/// force jumps, and a walk reads the jump from wherever its last stall left
/// the key beside it: there the two walks agree within the band's change
/// over a step, which the hand checks each against
/// ([`down_and_up_carrying_the_wippen_are_the_extremes_of_the_band_by_hand`]).
#[test]
fn halving_the_walks_step_changes_nothing_carrying_the_wippen() {
    for (name, b) in [
        ("the test bench", the_test_bench()),
        ("at the half-stroke line", at_the_half_stroke_line()),
    ] {
        let seats = b.seats();
        let mut statics = Statics::new();
        for stroke in [Stroke::Down, Stroke::Up] {
            let whole = b.seating().walk(&mut statics, &seats, stroke).unwrap();
            let half = b
                .seating()
                .walk_by(&mut statics, &seats, stroke, 0.5 * STEP_G)
                .unwrap();
            assert!(
                (whole.grams - half.grams).abs() <= 2.0 * walk_resolution(),
                "{name}, {stroke:?}: {} g by grams, {} g by half grams",
                whole.grams,
                half.grams
            );
            assert!(half.steps >= whole.steps, "{name}: more, shorter steps");
        }
    }
}

/// Consistency: **a walk across a deep dip carries the wippen in pieces.**
/// With the front rail lowered to 14 mm, within the page's range, the key
/// dips some 0.09 rad, and the down walk's breakaway carries the key and its
/// wippen through it in one increment, each step a row margin of their
/// farthest points with the link closed: more than one cap of the static
/// solve's steps, and within its pieces.
#[test]
fn a_walk_across_a_deep_dip_carries_the_wippen_in_pieces() {
    use crate::parts::mounts::SOLVE_PIECES;
    use crate::solve::statics::ITERATION_CAP;
    let mut a = regulated();
    a.frame.front_rail_height = 0.014;
    let (wippen, mass) = test_wippen();
    let b = Bench::new(&a, wippen, mass, KEY_FRICTION_G);
    let seats = b.seats();
    let [down, _] = b.walks(&seats);
    assert!(
        down.report.most_iterations > ITERATION_CAP,
        "the breakaway took {} steps",
        down.report.most_iterations
    );
    assert!(down.report.most_iterations <= SOLVE_PIECES * ITERATION_CAP);
    assert!(down.report.converged);
}

/// The balance weight and the friction weight in full, from D and U as the
/// walks read them from `b`'s seats with `standing` on the key throughout,
/// and the seats.
fn balance_and_friction(b: &Bench, standing: &[crate::mechanism::PointLoad]) -> (f64, f64, Seats) {
    let seating = Seating {
        standing,
        ..b.seating()
    };
    let mut statics = Statics::new();
    let seats = seating.seats(&mut statics).expect("the key seats");
    let [d, u] = [Stroke::Down, Stroke::Up].map(|stroke| {
        seating
            .walk(&mut statics, &seats, stroke)
            .expect("the key walks")
            .grams
    });
    (0.5 * (d + u), 0.5 * (d - u), seats)
}

/// D and U of `b` by hand at its seats, and their bound.
fn by_hand_at_the_seats(b: &Bench, seats: &Seats) -> (f64, f64, f64) {
    let (top, top_within) = b.at_seat(&seats.top, Motion::Down);
    let (bottom, bottom_within) = b.at_seat(&seats.bottom, Motion::Up);
    (top.down_g, bottom.up_g, top_within + bottom_within)
}

/// Level 0, consistency, the first non-confusion test on the wippen,
/// through the procedures: **one gram on the wippen moves the balance weight
/// and not the friction weight.** A gram added to the wippen's body at its
/// own center of mass weighs on the capstan through the ratio and moves D
/// and U alike, at the two seats' attitudes; the friction weight moves only
/// by what the ratio and the wippen's arm change between the two seats, the
/// tolerance, derived by hand from the seats, a few thousandths of a gram,
/// with the walks' resolution and the hand's bound.
#[test]
fn one_gram_on_the_wippen_moves_balance_weight_and_not_friction_weight() {
    let base = the_test_bench();
    let (wippen, mass) = test_wippen();
    let WippenAssemblyMass::Published {
        wippen: body,
        jack,
        repetition_lever,
    } = mass;
    let heavier = WippenAssemblyMass::Published {
        wippen: crate::parts::wippen_assembly::PartMass {
            mass: body.mass + 1e-3,
            ..body
        },
        jack,
        repetition_lever,
    };
    let loaded = Bench::new(&regulated(), wippen, heavier, KEY_FRICTION_G);
    let (bw0, f0, seats0) = balance_and_friction(&base, &[]);
    let (bw1, f1, seats1) = balance_and_friction(&loaded, &[]);
    let (d0, u0, within0) = by_hand_at_the_seats(&base, &seats0);
    let (d1, u1, within1) = by_hand_at_the_seats(&loaded, &seats1);
    let (moved_bw, moved_f) = (0.5 * (d1 + u1 - d0 - u0), 0.5 * (d1 - u1 - d0 + u0));
    let tolerance = 4.0 * walk_resolution() + within0 + within1;
    assert!(
        (bw1 - bw0 - moved_bw).abs() <= tolerance,
        "a gram on the wippen moved BW by {} g, by hand {moved_bw} g",
        bw1 - bw0
    );
    assert!(
        (f1 - f0 - moved_f).abs() <= tolerance,
        "it moved F by {} g, by hand {moved_f} g",
        f1 - f0
    );
    assert!(
        moved_f.abs() < 0.01,
        "the seats' tilt moves F by {moved_f} g"
    );
    assert!(
        moved_bw > 0.3,
        "BW grows by the gram's share, its arm about the center pin over the heel's \
         times the key's ratio, about four tenths: {moved_bw} g"
    );
}

/// Level 0, consistency, the second non-confusion test on the wippen,
/// through the procedures: **the wippen's center 1.2 times as tight moves
/// the friction weight and not the balance weight.** Its bound, carried
/// through the ratio onto the key, widens the band by `0.2·|r|·b_w` at each
/// seat's ratio and arm; the balance weight moves by half the difference of
/// the two, the tolerance, derived by hand from the seats, with the walks'
/// resolution and the hand's bound.
#[test]
fn scaling_the_wippens_friction_moves_friction_weight_and_not_balance_weight() {
    let base = the_test_bench();
    let (mut wippen, mass) = test_wippen();
    wippen.friction.bound_n_m *= 1.2;
    let tighter = Bench::new(&regulated(), wippen, mass, KEY_FRICTION_G);
    let (bw0, f0, seats0) = balance_and_friction(&base, &[]);
    let (bw1, f1, seats1) = balance_and_friction(&tighter, &[]);
    let (d0, u0, within0) = by_hand_at_the_seats(&base, &seats0);
    let (d1, u1, within1) = by_hand_at_the_seats(&tighter, &seats1);
    let (moved_bw, moved_f) = (0.5 * (d1 + u1 - d0 - u0), 0.5 * (d1 - u1 - d0 + u0));
    let tolerance = 4.0 * walk_resolution() + within0 + within1;
    assert!(
        (f1 - f0 - moved_f).abs() <= tolerance,
        "the tighter center moved F by {} g, by hand {moved_f} g",
        f1 - f0
    );
    assert!(
        (bw1 - bw0 - moved_bw).abs() <= tolerance,
        "it moved BW by {} g, by hand {moved_bw} g",
        bw1 - bw0
    );
    assert!(
        moved_bw.abs() < 0.01,
        "the seats' ratios move BW by {moved_bw} g"
    );
    assert!(
        moved_f > 0.05,
        "F grows by a fifth of the center's share: {moved_f} g"
    );
}

/// What `grams` at `at` on the key adds to the band at attitude `theta`,
/// grams at the measuring point: its moment's arm over the measuring
/// point's.
fn reflected(b: &Bench, at: Point, grams: f64, theta: f64) -> f64 {
    let point = b.k.measuring_point;
    let (sine, cosine) = (libm::sin(theta), libm::cos(theta));
    grams * (at.x * cosine - at.y * sine) / (point.x * cosine - point.y * sine)
}

/// Level 0, consistency, the key's first non-confusion test run on the
/// wippen bench, through the procedures: **one gram of mass on a key
/// carrying its wippen moves the balance weight and not the friction
/// weight.** A gram 150 mm in front of the balance pin, at the key front's
/// top height, moves each edge of the band by its moment read at the
/// measuring point at that seat's attitude, the wippen's share unmoved: BW
/// by the mean of the two, F by half their difference, what the tilt
/// between the seats does to the arms.
#[test]
fn one_gram_of_mass_moves_balance_weight_on_a_key_carrying_its_wippen() {
    let b = the_test_bench();
    let at = Point::new(0.150, b.k.front_top.y);
    let gram = crate::mechanism::PointLoad::weight(
        MaterialPoint {
            body: b.mounted().key,
            at,
        },
        1e-3,
    );
    let (bw0, f0, _) = balance_and_friction(&b, &[]);
    let (bw1, f1, seats) = balance_and_friction(&b, &[gram]);
    let angle = b.mounted().angle.0;
    let (on_top, on_bottom) = (
        reflected(&b, at, 1.0, seats.top.q[angle]),
        reflected(&b, at, 1.0, seats.bottom.q[angle]),
    );
    let tolerance = 4.0 * walk_resolution();
    assert!(
        (bw1 - bw0 + 0.5 * (on_top + on_bottom)).abs() <= tolerance,
        "1 g at 150 mm moved BW by {} g, by hand {} g",
        bw1 - bw0,
        -0.5 * (on_top + on_bottom)
    );
    let tilt = 0.5 * (on_bottom - on_top);
    assert!(
        (f1 - f0 - tilt).abs() <= tolerance,
        "the gram moved F by {} g, the tilt accounts for {tilt} g",
        f1 - f0
    );
    assert!(tilt.abs() < 0.01);
}

/// Level 0, consistency, the key's second non-confusion test run on the
/// wippen bench, through the procedures: **bushings 1.2 times as tight under
/// a key carrying its wippen move the friction weight and not the balance
/// weight.** The bound turns the band's edges by `±b / (g a)` at each seat's
/// arm, the wippen's share unmoved: F by `0.2 b` over the mean of the arms'
/// inverses, BW by `0.2 b` times half their difference.
#[test]
fn scaling_friction_moves_friction_weight_on_a_key_carrying_its_wippen() {
    let base = the_test_bench();
    let (wippen, mass) = test_wippen();
    let tighter = Bench::new(&regulated(), wippen, mass, 1.2 * KEY_FRICTION_G);
    let (bw0, f0, seats) = balance_and_friction(&base, &[]);
    let (bw1, f1, _) = balance_and_friction(&tighter, &[]);
    let angle = base.mounted().angle.0;
    let point = base.k.measuring_point;
    let arm = |theta: f64| point.x * libm::cos(theta) - point.y * libm::sin(theta);
    let (top, bottom) = (arm(seats.top.q[angle]), arm(seats.bottom.q[angle]));
    let extra = 1000.0 * (tighter.bushings - base.bushings) / G;
    let tolerance = 4.0 * walk_resolution();
    let expected_f = 0.5 * extra * (1.0 / top + 1.0 / bottom);
    assert!(
        (f1 - f0 - expected_f).abs() <= tolerance,
        "F moved by {} g, by hand {expected_f} g",
        f1 - f0
    );
    let tilt = 0.5 * extra * (1.0 / top - 1.0 / bottom);
    assert!(
        (bw1 - bw0 - tilt).abs() <= tolerance,
        "BW moved by {} g, the arms account for {tilt} g",
        bw1 - bw0
    );
    assert!(tilt.abs() < 0.01);
}

/// Consistency, by hand: **the wippen moves the balance weight by the
/// virtual work of its weight over the stroke.** Carried from the top seat
/// to the bottom one, the wippen's center of mass rises by `Δh`, work a
/// finger at the measuring point pays over the window's travel `s`: on
/// average `m_w Δh / s` grams. BW, read at the two seats, is the mean of
/// the wippen's share there, which stands from that average by at most half
/// how far the share ranges over the stroke, sampled by hand at the sweep's
/// steps; its center's friction leaves half the difference of its share at
/// the two seats, and the bare key's seats stand where the key's with its
/// wippen do, within the solve's tolerances.
#[test]
fn the_wippen_moves_balance_weight_by_the_virtual_work_of_its_weight_over_the_stroke() {
    let b = the_test_bench();
    let a = regulated();
    let bare = crate::parts::fixtures::on_frame(
        &a,
        crate::parts::frame::KeyKind::Natural,
        b.bushings,
        true,
    );
    let bare_seating = bare.seating();
    let mut statics = Statics::new();
    let bare_seats = bare_seating
        .seats(&mut statics)
        .expect("the bare key seats");
    let [bare_d, bare_u] = [Stroke::Down, Stroke::Up].map(|stroke| {
        bare_seating
            .walk(&mut statics, &bare_seats, stroke)
            .expect("the bare key walks")
            .grams
    });
    let (bw, _, seats) = balance_and_friction(&b, &[]);
    let moved = bw - 0.5 * (bare_d + bare_u);

    let m = b.mounted();
    let w = m.wippen.expect("a wippen");
    let wippen_mass = b.wippen_mass.props(&b.wippen);
    let height = |q: &[f64]| {
        m.mechanism
            .pose(q)
            .world(MaterialPoint {
                body: w.body,
                at: wippen_mass.com,
            })
            .y
    };
    let rise = height(&seats.bottom.q) - height(&seats.top.q);
    let (top, bottom) = seats.heights(m, b.k.measuring_point);
    let work = 1000.0 * wippen_mass.mass * rise / (top - bottom);

    let share = |hand: &Carrying| {
        1000.0 * hand.follows * hand.wippen_force_n_m / (G * hand.measuring_arm_m)
    };
    let center = |hand: &Carrying| {
        1000.0 * hand.follows.abs() * hand.wippen_friction_n_m / (G * hand.measuring_arm_m)
    };
    let band = b.band(&seats);
    let shares: Vec<f64> = b
        .states(&seats.top.q, &band.down)
        .iter()
        .map(|q| share(&b.by_hand(q, b.carrying(q, Motion::Down))))
        .collect();
    let ranges = shares.iter().fold(f64::MIN, |x, &y| x.max(y))
        - shares.iter().fold(f64::MAX, |x, &y| x.min(y));
    let at_top = b.by_hand(&seats.top.q, b.carrying(&seats.top.q, Motion::Down));
    let at_bottom = b.by_hand(&seats.bottom.q, b.carrying(&seats.bottom.q, Motion::Up));
    let tolerance =
        0.5 * ranges + 0.5 * (center(&at_top) - center(&at_bottom)).abs() + 4.0 * walk_resolution();
    assert!(
        (moved - work).abs() <= tolerance,
        "the wippen moved BW by {moved} g, its weight's work over the stroke {work} g, \
         within {tolerance} g"
    );
    assert!(work > 5.0, "the wippen's weight is felt: {work} g");
}

/// Consistency: **the walk names a wippen limit at the seat the sheet
/// refuses.** An increment of the walk, 60 g put on the key from its top
/// seat on a capstan 30 mm tall under a center pin 72 mm high, pushes the
/// wippen past its range: the walk names the bottom seat with the sheet's
/// depth, as the seat's rule does
/// ([`super::seat_wippen::pressed_from_the_top_seat`]).
#[test]
fn the_walk_names_a_wippen_limit_at_the_seat_the_sheet_refuses() {
    use super::seat::RefusedBy;
    use super::seat_wippen::pressed_from_the_top_seat;
    use crate::measure::walk::WalkRefusal;
    let (k, solved) = pressed_from_the_top_seat();
    let Err(refusal) = solved else {
        panic!("the weight pushes the wippen past its range")
    };
    let Some(RefusedBy::Wippen(named)) = k
        .seating()
        .name_the_wippens_refusal(&mut Statics::new(), Err(&refusal))
    else {
        panic!("the seat's rule names the wippen")
    };
    assert_eq!(
        crate::measure::walk::name_the_refusal(&k.seating(), &mut Statics::new(), refusal),
        WalkRefusal::Wippen(named)
    );
}

/// Consistency: **the band names a wippen limit at the seat the sheet
/// refuses.** On a capstan 30 mm tall, the center pin is halved to a
/// hundredth of a micron between 69 mm, where the top seat pushes the
/// wippen past its range, and 72 mm, where it fits
/// ([`super::seat_wippen::halve_between`]). A step of the band from the top
/// seat at the lowest height found to fit, the key's angle held there, its
/// wippen where that seat left it, the center pin then lowered by one step
/// of the page's slider, half a millimeter, stands the heel no deeper in
/// the key than that half millimeter, within the row margin, and pushes
/// the wippen past its range: the band names the top seat, with the depth
/// the sheet refuses it with there, to the bit, more than nothing and no
/// more than the half millimeter the center pin was lowered from a height
/// at which the top seat fits, every point of the wippen moving as much.
#[test]
fn the_band_names_a_wippen_limit_at_the_seat_the_sheet_refuses() {
    use super::seat::{SeatRefusal, WhichSeat, WippenRefusal};
    use super::seat_wippen::{halve_between, on_a_tall_capstan, FITS_AT_THE_TOP_SEAT_ONLY};
    use crate::measure::sweep::SweepRefusal;
    const ONE_PAGE_STEP: f64 = 0.0005;
    let fits = |center: f64| {
        on_a_tall_capstan(center)
            .seating()
            .top(&mut Statics::new())
            .is_ok()
    };
    let (_, seated) = halve_between(REFUSED_AT_THE_TOP_SEAT, FITS_AT_THE_TOP_SEAT_ONLY, fits);
    let before = on_a_tall_capstan(seated)
        .seating()
        .top(&mut Statics::new())
        .expect("the top seat fits");
    let k = on_a_tall_capstan(seated - ONE_PAGE_STEP);
    let Err(SeatRefusal::WippenPushedPastItsRange {
        at: WhichSeat::Top,
        heel_depth,
    }) = k.seating().seats(&mut Statics::new())
    else {
        panic!("the sheet refuses the top seat")
    };
    let angle = k.mounted.angle;
    let held = [Prescribed {
        coord: angle,
        value: before.q[angle.0],
    }];
    let mut statics = Statics::new();
    let refusal = k
        .mounted
        .solve_held(&mut statics, &before.q, &[], &held, Friction::On)
        .expect_err("the step pushes the wippen past its range");
    assert_eq!(
        crate::measure::sweep::name_the_refusal(&k.seating(), &mut statics, refusal),
        SweepRefusal::Wippen(WippenRefusal::PushedPastItsRange {
            at: WhichSeat::Top,
            heel_depth
        })
    );
    assert!(
        heel_depth > 0.0 && heel_depth <= ONE_PAGE_STEP,
        "{heel_depth} m deep"
    );
}

/// The increment of the walk that pushes the wippen past its range
/// ([`super::seat_wippen::pressed_from_the_top_seat`]), with a seating of
/// the same key whose capstan's point stands in front of the balance pin,
/// where the preparation's load at the capstan would lift the key's back:
/// the key alone, at whose seats the wippen's refusal is named, has none.
fn pressed_with_the_capstan_in_front_of_the_balance_pin() -> (OnFrame, Point, crate::solve::Refusal)
{
    use super::seat_wippen::pressed_from_the_top_seat;
    const IN_FRONT_OF_THE_BALANCE_PIN: f64 = 0.050;
    let (k, solved) = pressed_from_the_top_seat();
    let Err(refusal) = solved else {
        panic!("the weight pushes the wippen past its range")
    };
    let capstan = Point::new(IN_FRONT_OF_THE_BALANCE_PIN, k.capstan.y);
    (k, capstan, refusal)
}

/// Consistency: **a walk whose key alone has no seat names the key alone's
/// own refusal**: the increment that pushes the wippen past its range, named
/// with the capstan in front of the balance pin
/// ([`pressed_with_the_capstan_in_front_of_the_balance_pin`]).
#[test]
fn a_walk_whose_key_alone_has_no_seat_names_the_key_alones_own_refusal() {
    use super::seat::SeatRefusal;
    use crate::measure::walk::WalkRefusal;
    let (k, capstan, refusal) = pressed_with_the_capstan_in_front_of_the_balance_pin();
    let in_front = Seating {
        capstan,
        ..k.seating()
    };
    assert_eq!(
        crate::measure::walk::name_the_refusal(&in_front, &mut Statics::new(), refusal),
        WalkRefusal::KeyAlone(SeatRefusal::CapstanNotBehindTheBearing)
    );
}

/// Consistency: **a band whose key alone has no seat names the key alone's
/// own refusal**: the increment that pushes the wippen past its range, named
/// with the capstan in front of the balance pin
/// ([`pressed_with_the_capstan_in_front_of_the_balance_pin`]).
#[test]
fn a_band_whose_key_alone_has_no_seat_names_the_key_alones_own_refusal() {
    use super::seat::SeatRefusal;
    use crate::measure::sweep::SweepRefusal;
    let (k, capstan, refusal) = pressed_with_the_capstan_in_front_of_the_balance_pin();
    let in_front = Seating {
        capstan,
        ..k.seating()
    };
    assert_eq!(
        crate::measure::sweep::name_the_refusal(&in_front, &mut Statics::new(), refusal),
        SweepRefusal::KeyAlone(SeatRefusal::CapstanNotBehindTheBearing)
    );
}

/// Consistency: **the walk and the band name a pull that carries the wippen
/// to its limit as lifting the key off its frame.** On the lifting bench
/// ([`super::seat_wippen::the_lifting_bench`]) both seats stand the wippen in
/// range; a pull of 100 g from the top seat lifts the key about its back rail
/// cloth and carries the wippen to the top of its range, which the walk and
/// the band each name as the key lifting off its frame.
#[test]
fn the_walk_and_the_band_name_a_pull_that_carries_the_wippen_to_its_limit_as_lifting_the_key_off_its_frame(
) {
    use super::seat_wippen::pulled_from_the_top_seat;
    use crate::measure::sweep::SweepRefusal;
    use crate::measure::walk::WalkRefusal;
    let (k, solved) = pulled_from_the_top_seat();
    let Err(refusal) = solved else {
        panic!("the pull carries the wippen to the top of its range")
    };
    assert_eq!(
        crate::measure::walk::name_the_refusal(&k.seating(), &mut Statics::new(), refusal.clone()),
        WalkRefusal::LiftsOffItsFrame
    );
    assert_eq!(
        crate::measure::sweep::name_the_refusal(&k.seating(), &mut Statics::new(), refusal),
        SweepRefusal::LiftsOffItsFrame
    );
}

/// The test key on its frame and on the jig, its bushings the test
/// bench's, carrying `wippen` where it carries one, as the bench weighs it.
fn weighed(
    wippen: Option<(&WippenAssembly, &WippenAssemblyMass)>,
) -> (
    crate::parts::mounts::OnItsFrame,
    crate::measure::jig::KeyJig,
    Setup,
) {
    let a = regulated();
    let (on_frame, jig) = weighed_with(&a, wippen);
    (on_frame, jig, a)
}

/// `a`'s key on its frame and on the jig, its bushings the test bench's,
/// carrying `wippen` where it carries one, as the bench weighs it.
fn weighed_with(
    a: &Setup,
    wippen: Option<(&WippenAssembly, &WippenAssemblyMass)>,
) -> (
    crate::parts::mounts::OnItsFrame,
    crate::measure::jig::KeyJig,
) {
    let mass = mass_of(&a.key);
    let bushings = AxisFriction::of_weight_at(KEY_FRICTION_G, a.measuring_point()).bound_n_m;
    let on_frame = crate::parts::mounts::on_its_frame(
        crate::parts::mounts::KeyMount::OnPunching,
        &a.key,
        &a.frame,
        crate::parts::frame::KeyKind::Natural,
        mass,
        bushings,
        wippen,
    )
    .expect("the key mounts on its frame");
    let jig = crate::measure::jig::KeyJig::new(&a.key, &a.protocol, mass)
        .expect("the key mounts on the jig");
    (on_frame, jig)
}

/// The sheet of a key on its frame and on the jig, as the bench reads it.
fn sheet_of(
    on_frame: &crate::parts::mounts::OnItsFrame,
    jig: &crate::measure::jig::KeyJig,
    a: &Setup,
    carries_a_wippen: bool,
) -> crate::measure::weighing::Sheet {
    weighing_of(on_frame, jig, a, carries_a_wippen).sheet(&mut Statics::new())
}

/// The bases of the test bench's inputs: its mass weighed by hand, the
/// height of its center of mass estimated, its friction and its dimensions
/// assumed, and the wippen it carries, where it carries one, published.
fn bases_of(carries_a_wippen: bool) -> crate::measure::measurement::Bases {
    use crate::measure::measurement::{Bases, Basis, WippenAssemblyBases};
    Bases {
        key_mass: Basis::SetByHand,
        center_of_mass_height: Basis::Estimated,
        key_friction: Basis::Assumed,
        geometry: Basis::Assumed,
        wippen_assembly: carries_a_wippen.then_some(WippenAssemblyBases {
            mass: Basis::Published,
            friction: Basis::Published,
        }),
    }
}

/// The weighing of a key on its frame and on the jig, as the bench reads it.
fn weighing_of<'a>(
    on_frame: &'a crate::parts::mounts::OnItsFrame,
    jig: &'a crate::measure::jig::KeyJig,
    a: &Setup,
    carries_a_wippen: bool,
) -> crate::measure::weighing::Weighing<'a> {
    crate::measure::weighing::Weighing {
        on_frame,
        jig,
        measuring_point: a.protocol.measuring_point_on_key(&a.key),
        measuring_point_from_lip_m: a.protocol.measuring_point_from_lip,
        thinnest_paper_m: a.frame.drawer.thinnest(),
        bases: bases_of(carries_a_wippen),
    }
}

/// Consistency: **both key friction weights read the same bits with the
/// wippen and without**, and so does front weight: the key friction weights
/// weigh the key alone, its bushings, the same key on the same frame the
/// bench mounts beside the key with its wippen; the jig weighs the key alone
/// too. The key weight ratio is not among them: with its wippen it is read
/// where the capstan touches the heel at the top seat, and without on the
/// capstan's top
/// ([`the_key_weight_ratio_is_read_where_the_capstan_touches_the_heel_at_the_top_seat`]).
#[test]
fn the_key_friction_weights_read_the_same_bits_with_the_wippen_and_without() {
    use crate::measure::measurement::ProcedureId;
    let (wippen, mass) = test_wippen();
    let (with, with_jig, a) = weighed(Some((&wippen, &mass)));
    let (without, without_jig, _) = weighed(None);
    let carrying = sheet_of(&with, &with_jig, &a, true);
    let bare = sheet_of(&without, &without_jig, &a, false);
    for procedure in [
        ProcedureId::FrontWeight,
        ProcedureId::KeyFrictionWeightZeroedOnTheJig,
        ProcedureId::KeyFrictionWeightZeroedOnTheFrame,
    ] {
        let read = |sheet: &crate::measure::weighing::Sheet| {
            sheet
                .get(procedure)
                .expect("the sheet holds it")
                .clone()
                .expect("it is read")
        };
        assert_eq!(read(&carrying), read(&bare), "{procedure:?}");
    }
}

/// Consistency: **each reading taken at a seat says where the wippen rests
/// there**: key height and D at the top seat, key dip and U at the bottom
/// one, the capstan on the heel cloth at both on the test bench; and the
/// readings of the key alone, or computed from others, name nothing. Each
/// reading that reads the wippen names its inputs where they are not
/// measured: a published wippen leaves the trust as the key's inputs make
/// it.
#[test]
fn each_reading_at_a_seat_says_where_the_wippen_rests() {
    use crate::contact::ContactCode;
    use crate::measure::measurement::ProcedureId;
    use crate::parts::verdicts::CarriedBy;
    let (wippen, mass) = test_wippen();
    let (on_frame, jig, a) = weighed(Some((&wippen, &mass)));
    let sheet = sheet_of(&on_frame, &jig, &a, true);
    let on_the_cloth = vec![CarriedBy {
        link: ContactCode::CapstanOnTheHeel,
        parts: vec![PartCode::WippenHeelCloth],
    }];
    for procedure in ProcedureId::ALL {
        let reading = sheet
            .get(procedure)
            .expect("the sheet holds it")
            .as_ref()
            .unwrap_or_else(|why| panic!("{procedure:?}: {why:?}"));
        let expected = match procedure {
            ProcedureId::KeyHeight
            | ProcedureId::KeyDip
            | ProcedureId::DownWeight
            | ProcedureId::UpWeight => on_the_cloth.clone(),
            _ => Vec::new(),
        };
        assert_eq!(
            reading.conditions.wippen_carried_by, expected,
            "{procedure:?}"
        );
    }
}

/// Consistency, by hand: **a key carrying its wippen is held by its friction
/// between its up and down weights**, its margin read on the folded box. At
/// the top seat, its tail on its cloth, a weight halfway between the
/// band's two edges by hand leaves the key held there by its friction,
/// touching its cloth; the way it can move is away from the cloth, its back
/// rising and its front going down, so the margin is the edge of the box
/// toward going down, `b_k + |r|·b_w + Q_k + r·Q_w − W g a`, through the link
/// the wippen would rise on.
#[test]
fn a_key_carrying_its_wippen_is_held_by_its_friction_between_its_up_and_down_weights() {
    use crate::contact::ContactCode;
    use crate::parts::verdicts::{holding, Holding};
    let b = the_test_bench();
    let seats = b.seats();
    let q = &seats.top.q;
    let feature = b.carrying(q, Motion::Down);
    let hand = b.by_hand(q, feature);
    let branches = hand.branches();
    let inside = 0.5 * (branches.down_g + branches.up_g);
    let per_gram = 1e-3 * G * hand.measuring_arm_m;
    let margin = hand.bushings_n_m
        + hand.follows.abs() * hand.wippen_friction_n_m
        + hand.key_force_n_m
        + hand.follows * hand.wippen_force_n_m
        - inside * per_gram;
    match holding(b.mounted(), q, &[b.seating().weight(inside)]) {
        Some(Holding::HeldByFriction {
            touching: Some(ContactCode::BackRailCloth),
            margin: read,
        }) => assert!(
            (read - margin).abs() <= b.hand_within(q, feature, &hand) * per_gram,
            "a margin of {read} N·m, by hand {margin} N·m"
        ),
        other => panic!("{other:?}"),
    }
}

/// Consistency, by hand: **lifted off its balance punching, a key carrying
/// its wippen is read on its folded coordinates.** With the front rail
/// raised to 44 mm the key with its wippen tips onto its front punching,
/// its front pin against the wood beside its slot, and rides off its balance
/// punching; the two hold it with no friction, the wippen weighing on its
/// capstan. Pulled up at its measuring point, it needs its friction once
/// the pull and the weights, the wippen's folded onto the key's slide and
/// angle through the link, fall outside what the two stops can push: the
/// pull at which the verdict says so is the one the hand finds, folding the
/// wippen's weight by how its angle follows the slide and the angle, by the
/// central differences of the link's gap, and solving for the two stops'
/// pushes, each along its row.
#[test]
fn lifted_off_its_punching_a_key_carrying_its_wippen_is_read_on_its_folded_coordinates() {
    use crate::contact::ContactCode;
    use crate::measure::seat::{Start, Starting};
    use crate::parts::verdicts::{holding, Holding};
    let mut a = regulated();
    a.frame.front_rail_height = 0.044;
    let (wippen, mass) = test_wippen();
    let b = Bench::new(&a, wippen, mass, KEY_FRICTION_G);
    let m = b.mounted();
    let mut statics = Statics::new();
    let nominal = crate::parts::mounts::nominal_pose(m);
    let Ok(Start::From { q: start, .. }) =
        b.seating()
            .start(&mut statics, &nominal, None, Starting::Loading)
    else {
        panic!("the wippen fits on the key where it stands");
    };
    let unloaded = [b.seating().weight(0.0)];
    let q = m
        .solve(&mut statics, &start, &unloaded, Friction::Off)
        .expect("the key comes to rest")
        .q;
    let stops = vec![
        ContactCode::FrontRailPunching,
        ContactCode::FrontPinOnTheWood,
    ];
    let needs_friction = |grams: f64| match holding(m, &q, &[b.seating().weight(grams)]) {
        Some(Holding::LiftsOffBalancePunching {
            touching,
            needs_friction,
            ..
        }) if touching == stops => needs_friction,
        other => panic!("{other:?}"),
    };
    assert!(!needs_friction(0.0));

    let lift = m.lift.as_ref().expect("a key free to lift").coord.0;
    let feature = b.resting(&q);
    let hand = b.by_hand(&q, feature);
    let key_mass = mass_of(&b.a.key).expect("weighed").mass;
    let follows_slide = b.follows_on(&q, feature, lift, FOLLOWS_STEP);
    let folded = [
        -key_mass * G + follows_slide * hand.wippen_force_n_m,
        hand.key_force_n_m + hand.follows * hand.wippen_force_n_m,
    ];
    let per_gram = [-1e-3 * G, -1e-3 * G * hand.measuring_arm_m];
    let mut rows = Rows::new();
    m.mechanism
        .rows(&m.mechanism.pose(&q), ROW_MARGIN, &mut rows);
    let along = |code: ContactCode| {
        let i = (0..rows.len())
            .find(|&i| {
                m.mechanism.pairs()[usize::from(rows.rows()[i].pair)].code == code
                    && rows.rows()[i].gap <= TOUCHING
            })
            .expect("the stop is closed");
        [rows.grad(i)[lift], rows.grad(i)[m.angle.0]]
    };
    let (front, pin) = (along(stops[0]), along(stops[1]));
    let pushes = |grams: f64| {
        let force = [
            folded[0] + grams * per_gram[0],
            folded[1] + grams * per_gram[1],
        ];
        let det = front[0] * pin[1] - front[1] * pin[0];
        (
            (-force[0] * pin[1] + force[1] * pin[0]) / det,
            (-front[0] * force[1] + front[1] * force[0]) / det,
        )
    };
    let (at_rest, per_one) = (pushes(0.0), pushes(1.0));
    let to_nothing = |rest: f64, one: f64| rest / (rest - one);
    let pull = [
        to_nothing(at_rest.0, per_one.0),
        to_nothing(at_rest.1, per_one.1),
    ]
    .into_iter()
    .filter(|g| *g < 0.0)
    .fold(f64::MIN, f64::max);

    let (mut held, mut free) = (0.0, -200.0);
    assert!(needs_friction(free));
    while held - free > 1e-9 {
        let middle = 0.5 * (held + free);
        if needs_friction(middle) {
            free = middle;
        } else {
            held = middle;
        }
    }
    let within = 1e-9 + b.hand_within(&q, feature, &hand) * (1.0 + follows_slide.abs());
    assert!(
        (held - pull).abs() <= within,
        "the stops let go at a pull of {held} g, by hand {pull} g"
    );
}

/// How far a point the hand works where two pieces meet may stand from the
/// one the engine reads, meters: `64 ε` of the 0.3 m every coordinate stands
/// under. The hand and the engine work the same pieces at the same `q`, and
/// the row's two points stand on the two surfaces along the normal whatever
/// its gap, so the two differ by round-off alone: the row's own, the point
/// carried into its body's frame ([`crate::mechanism::Placement::local`],
/// `8 ε`), and the hand's few dozen roundings.
const HAND_CONTACT_M: f64 = 64.0 * f64::EPSILON * 0.3;

/// How far a jig reading may stand from its closed form, grams: sixteen
/// units in the last place of the largest magnitude it touches, `magnitude`
/// (the jig's tests' `agree`).
fn jig_round_off(magnitude: f64) -> f64 {
    16.0 * f64::EPSILON * magnitude
}

/// The kind of drawn piece the capstan bears with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Segment,
    Arc,
    Vertex,
}

fn kind_of(piece: Piece) -> Kind {
    match piece {
        Piece::Segment { .. } => Kind::Segment,
        Piece::Arc { .. } => Kind::Arc,
        Piece::Vertex { .. } => Kind::Vertex,
        Piece::Center { .. } => unreachable!("a link reads no center"),
    }
}

/// The regulated test setup with its capstan's head turned to a dome of
/// 9.7 mm, as the test fixture's key with every feature has it.
fn with_a_domed_capstan() -> Setup {
    let mut a = regulated();
    a.key.capstan_screw.dome_radius = Some(0.0097);
    a
}

/// The regulated test setup with its capstan `behind_mm` behind the balance
/// pin, as [`capstan_at`] builds its bench.
fn with_the_capstan_at(behind_mm: f64) -> Setup {
    let mut a = regulated();
    a.key.capstan = -behind_mm * 1e-3;
    a
}

/// The benches the capstan's touch on the heel is checked on, each with the
/// kind of piece the capstan bears with at the top seat: the test bench, on
/// its capstan's flat top, the heel's circle standing about 1.3 mm behind
/// the capstan's axis; its capstan domed to 9.7 mm, on the dome, about
/// 0.4 mm behind; and its capstan 129 mm behind the balance pin, on the
/// rim of its flat top, the heel's circle standing about 3 mm in front of
/// it.
fn touch_benches() -> Vec<(&'static str, Setup, Kind)> {
    vec![
        ("the flat capstan", regulated(), Kind::Segment),
        ("the 9.7 mm dome", with_a_domed_capstan(), Kind::Arc),
        (
            "the capstan at 129 mm",
            with_the_capstan_at(129.0),
            Kind::Vertex,
        ),
    ]
}

/// `a`'s key carrying the test wippen, as the band's tests mount it.
fn bench_of(a: &Setup) -> Bench {
    let (wippen, mass) = test_wippen();
    Bench::new(a, wippen, mass, KEY_FRICTION_G)
}

/// `a`'s sheet, its key carrying the test wippen.
fn carrying_sheet_of(a: &Setup) -> crate::measure::weighing::Sheet {
    let (wippen, mass) = test_wippen();
    let (on_frame, jig) = weighed_with(a, Some((&wippen, &mass)));
    sheet_of(&on_frame, &jig, a, true)
}

/// The key weight ratio `sheet` reads.
fn key_ratio_of(
    sheet: &crate::measure::weighing::Sheet,
) -> crate::measure::measurement::Measurement {
    use crate::measure::measurement::ProcedureId;
    sheet
        .get(ProcedureId::KeyWeightRatio)
        .expect("the sheet holds it")
        .clone()
        .expect("it is read")
}

/// Consistency, by hand: **the capstan touches the heel where its pieces
/// meet at the top seat**, on the heel cloth. On each bench the closed row
/// of the capstan's link bears with the kind of piece its capstan stands
/// under the heel with ([`touch_benches`]); the points the engine reads, on
/// the capstan in the key's frame and on the heel in the wippen's, are the
/// hand's ([`Bench::hand_contact`]) within round-off
/// ([`HAND_CONTACT_M`]).
#[test]
fn the_capstan_touches_the_heel_where_its_pieces_meet_at_the_top_seat() {
    for (name, a, kind) in touch_benches() {
        let b = bench_of(&a);
        let top = b.seats().top;
        let m = b.mounted();
        let feature = b.carrying(&top.q, Motion::Down);
        assert_eq!(kind_of(feature.a), kind, "{name}");
        let contact = m
            .capstan_on_the_heel(&top.q, TOUCHING)
            .unwrap_or_else(|| panic!("{name}: the capstan touches the heel"));
        assert_eq!(contact.part, PartCode::WippenHeelCloth, "{name}");
        let (on_a, on_b) = b.hand_contact(&top.q, feature);
        let pose = m.mechanism.pose(&top.q);
        let wippen = m.wippen.expect("a wippen").body;
        for (what, engine, hand) in [
            (
                "on the capstan",
                contact.on_the_capstan,
                pose.placement(m.key).local(on_a),
            ),
            (
                "on the heel",
                contact.on_the_heel,
                pose.placement(wippen).local(on_b),
            ),
        ] {
            assert!(
                distance(engine, hand) <= HAND_CONTACT_M,
                "{name}, {what}: {engine:?}, by hand {hand:?}"
            );
        }
    }
}

/// Consistency: **a capstan under the heel's wood touches it there**: 135 mm
/// behind the balance pin, the test bench's capstan stands in front of the
/// heel cloth's front end at the top seat, and the part it touches is the
/// heel's wood.
#[test]
fn a_capstan_under_the_heels_wood_touches_it_there() {
    let b = capstan_at(135.0);
    let top = b.seats().top;
    let contact = b
        .mounted()
        .capstan_on_the_heel(&top.q, TOUCHING)
        .expect("the capstan touches the heel");
    assert_eq!(contact.part, PartCode::WippenHeel);
}

/// Consistency: **a heel resting on the key's top is not touched by the
/// capstan**: 100 mm behind the balance pin, the test bench's capstan stands
/// in front of the heel, which rests on the key's top at the top seat.
#[test]
fn a_heel_resting_on_the_keys_top_is_not_touched_by_the_capstan() {
    let b = capstan_at(100.0);
    let top = b.seats().top;
    assert_eq!(b.mounted().capstan_on_the_heel(&top.q, TOUCHING), None);
}

/// Consistency, by hand: **in the valley the capstan touches the heel
/// through the row that carries it down.** Where the heel cloth's back end
/// meets the heel's bottom ([`in_the_valley`]), the capstan bears on both at
/// the bottom seat; the point read is the one of the row the hand finds
/// carrying the wippen down, the larger ratio, within round-off.
#[test]
fn in_the_valley_the_capstan_touches_the_heel_through_the_row_that_carries_it_down() {
    let b = in_the_valley();
    let bottom = b.seats().bottom;
    let m = b.mounted();
    let feature = b.carrying(&bottom.q, Motion::Down);
    let contact = m
        .capstan_on_the_heel(&bottom.q, TOUCHING)
        .expect("the capstan touches the heel");
    let (on_a, _) = b.hand_contact(&bottom.q, feature);
    let hand = m.mechanism.pose(&bottom.q).placement(m.key).local(on_a);
    assert!(
        distance(contact.on_the_capstan, hand) <= HAND_CONTACT_M,
        "{:?}, by hand {hand:?}",
        contact.on_the_capstan
    );
}

/// Consistency, by hand: **the key weight ratio is read where the capstan
/// touches the heel at the top seat**, *"centered on a vertical axis which
/// passes through the capstan/heel contact point"* [V, Stanwood 2000,
/// p. 22]. On the level jig the ratio is the arm over the measuring arm, so
/// read at the contact `x_c` rather than on the capstan's top `X_cap` it
/// moves by `(|x_c| − |X_cap|) / x_meas`, `x_c` worked by hand
/// ([`Bench::hand_contact`]). The bound is the hand's round-off over the
/// measuring arm and the two jig readings', each within sixteen units in
/// the last place of the readings' magnitude over the load; and the shift
/// stands more than twice that bound from nothing, so a ratio still read on
/// the capstan's top fails.
#[test]
fn the_key_weight_ratio_is_read_where_the_capstan_touches_the_heel_at_the_top_seat() {
    use crate::measure::measurement::{ContactFound, ContactPoint};
    for (name, a, _) in touch_benches() {
        let b = bench_of(&a);
        let top = b.seats().top;
        let m = b.mounted();
        let (on_a, _) = b.hand_contact(&top.q, b.carrying(&top.q, Motion::Down));
        let x_c = m.mechanism.pose(&top.q).placement(m.key).local(on_a).x;
        let x_cap = a.key.capstan_contact().x;
        let x_meas = a.measuring_point();
        let kr = key_ratio_of(&carrying_sheet_of(&a));
        let (_, jig) = weighed_with(&a, None);
        let kr_top = jig.key_ratio().expect("the jig reads it");
        let front = jig.front_weight_g().expect("the key is weighed").abs();
        let load = a.protocol.key_ratio_load_g;
        let bound = HAND_CONTACT_M / x_meas + 2.0 * jig_round_off((front + load) / load);
        let shift = (x_c.abs() - x_cap.abs()) / x_meas;
        assert!(
            matches!(
                kr.conditions.capstan_heel_contact,
                Some(ContactPoint {
                    found: ContactFound::WhereTheCapstanTouchesTheHeel {
                        part: PartCode::WippenHeelCloth
                    },
                    ..
                })
            ),
            "{name}: {:?}",
            kr.conditions.capstan_heel_contact
        );
        assert!(
            (kr.value - kr_top - shift).abs() <= bound,
            "{name}: KR {} against {kr_top} on the capstan's top, by hand {shift} apart, \
             within {bound}",
            kr.value
        );
        assert!(shift.abs() > 2.0 * bound, "{name}: {shift} against {bound}");
    }
}

/// Consistency: **with no top seat the key weight ratio is read on the
/// capstan's top.** On a capstan 30 mm tall under a center pin 69 mm high,
/// the top seat pushes the wippen past its range, so no point where the
/// capstan touches the heel is read: the 10 g is centered on the capstan's
/// top, on its axis, as on a bare key, and the ratio is the bare jig's to
/// the bit.
#[test]
fn with_no_top_seat_the_key_weight_ratio_is_read_on_the_capstans_top() {
    use super::seat_wippen::{on_a_tall_capstan, TALL_CAPSTAN};
    use crate::measure::measurement::{ContactFound, ContactPoint};
    assert!(on_a_tall_capstan(REFUSED_AT_THE_TOP_SEAT)
        .seating()
        .top(&mut Statics::new())
        .is_err());
    let mut a = regulated();
    a.key.capstan_screw.height = TALL_CAPSTAN;
    let (mut wippen, mass) = test_wippen();
    wippen.center.y = REFUSED_AT_THE_TOP_SEAT;
    let (on_frame, jig) = weighed_with(&a, Some((&wippen, &mass)));
    let kr = key_ratio_of(&sheet_of(&on_frame, &jig, &a, true));
    assert_eq!(
        kr.conditions.capstan_heel_contact,
        Some(ContactPoint {
            at: a.key.capstan_contact(),
            found: ContactFound::NoTopSeat
        })
    );
    assert_eq!(
        kr.value.to_bits(),
        jig.key_ratio().expect("the jig reads it").to_bits()
    );
}

/// Consistency: **with the heel on the key's top the key weight ratio is
/// read on the capstan's top**: 100 mm behind the balance pin, the capstan
/// does not touch the heel at the top seat, and the 10 g is centered on the
/// capstan's top, on its axis, the ratio the bare jig's to the bit.
#[test]
fn with_the_heel_on_the_keys_top_the_key_weight_ratio_is_read_on_the_capstans_top() {
    use crate::measure::measurement::{ContactFound, ContactPoint};
    let a = with_the_capstan_at(100.0);
    let (_, jig) = weighed_with(&a, None);
    let kr = key_ratio_of(&carrying_sheet_of(&a));
    assert_eq!(
        kr.conditions.capstan_heel_contact,
        Some(ContactPoint {
            at: a.key.capstan_contact(),
            found: ContactFound::CapstanOffTheHeel
        })
    );
    assert_eq!(
        kr.value.to_bits(),
        jig.key_ratio().expect("the jig reads it").to_bits()
    );
}

/// Consistency: **where the key alone's seats are refused, the contact is
/// read at the top seat.** With the front rail 300 mm in front of the
/// balance pin, past the key's front edge at 258 mm, no stop stands below
/// the key's front: the key alone's seats are refused, its top seat not,
/// and the key weight ratio and the wippen weight are read where the
/// capstan touches the heel at the top seat [`Seating::top`] seats the key
/// carrying its wippen at.
#[test]
fn where_the_key_alones_seats_are_refused_the_contact_is_read_at_the_top_seat() {
    use super::seat::SeatRefusal;
    use crate::measure::measurement::{ContactFound, ContactPoint, ProcedureId};
    const PAST_THE_KEYS_FRONT: f64 = 0.300;
    let mut a = regulated();
    a.frame.front_rail = PAST_THE_KEYS_FRONT;
    let b = bench_of(&a);
    let alone = Seating {
        mounted: b.mounted().key_alone.as_deref().expect("a key alone"),
        ..b.seating()
    };
    assert_eq!(
        alone.seats(&mut Statics::new()).err(),
        Some(SeatRefusal::NoStopBelowTheFront)
    );
    let top = b
        .seating()
        .top(&mut Statics::new())
        .expect("the top seat fits");
    let contact = b
        .mounted()
        .capstan_on_the_heel(&top.q, TOUCHING)
        .expect("the capstan touches the heel at the top seat");
    let sheet = carrying_sheet_of(&a);
    let found = ContactFound::WhereTheCapstanTouchesTheHeel { part: contact.part };
    assert_eq!(
        key_ratio_of(&sheet).conditions.capstan_heel_contact,
        Some(ContactPoint {
            at: contact.on_the_capstan,
            found
        })
    );
    assert_eq!(
        reading_of(&sheet, ProcedureId::WippenWeight)
            .conditions
            .capstan_heel_contact,
        Some(ContactPoint {
            at: contact.on_the_heel,
            found
        })
    );
}

/// Consistency: **where only the bottom seat is refused, the contact is read
/// at the top seat.** On a capstan 30 mm tall under a center pin 72 mm high,
/// the wippen fits at the key's top seat and is pushed past its range at the
/// bottom one: the sheet's seats are refused, and the key weight ratio is
/// read where the capstan touches the heel at the top seat the seats solved
/// and handed back, the one [`Seating::top`] seats, without solving it
/// again.
#[test]
fn where_only_the_bottom_seat_is_refused_the_contact_is_read_at_the_top_seat() {
    use super::seat::{SeatRefusal, WhichSeat};
    use super::seat_wippen::{on_a_tall_capstan, FITS_AT_THE_TOP_SEAT_ONLY, TALL_CAPSTAN};
    use crate::measure::measurement::{ContactFound, ContactPoint};
    let k = on_a_tall_capstan(FITS_AT_THE_TOP_SEAT_ONLY);
    assert!(matches!(
        k.seating().seats(&mut Statics::new()),
        Err(SeatRefusal::WippenPushedPastItsRange {
            at: WhichSeat::Bottom,
            ..
        })
    ));
    let top = k
        .seating()
        .top(&mut Statics::new())
        .expect("the top seat fits");
    let contact = k
        .mounted
        .capstan_on_the_heel(&top.q, TOUCHING)
        .expect("the capstan touches the heel at the top seat");
    let mut a = regulated();
    a.key.capstan_screw.height = TALL_CAPSTAN;
    let (mut wippen, mass) = test_wippen();
    wippen.center.y = FITS_AT_THE_TOP_SEAT_ONLY;
    let (on_frame, jig) = weighed_with(&a, Some((&wippen, &mass)));
    let kr = key_ratio_of(&sheet_of(&on_frame, &jig, &a, true));
    assert_eq!(
        kr.conditions.capstan_heel_contact,
        Some(ContactPoint {
            at: contact.on_the_capstan,
            found: ContactFound::WhereTheCapstanTouchesTheHeel { part: contact.part }
        })
    );
}

/// The reading of `procedure` on `sheet`, read.
fn reading_of(
    sheet: &crate::measure::weighing::Sheet,
    procedure: crate::measure::measurement::ProcedureId,
) -> crate::measure::measurement::Measurement {
    sheet
        .get(procedure)
        .expect("the sheet holds it")
        .clone()
        .unwrap_or_else(|why| panic!("{procedure:?}: {why:?}"))
}

/// The test wippen assembly's mass, its center of mass in its frame.
fn wippen_props() -> crate::mechanism::MassProps {
    let (wippen, mass) = test_wippen();
    mass.props(&wippen)
}

/// Consistency, by hand: **the wippen weight is read where the capstan
/// touches the heel at the top seat**, *"measured at the capstan/heel
/// contact point"* [V, Stanwood 2000, glossary, p. 23]: the roller under the
/// point of the heel the capstan touches, `x_mark` along the wippen, worked
/// by hand ([`Bench::hand_contact`]), it reads the assembly's static moment
/// over that arm, `1000 m com.x / x_mark`, within what the hand's round-off
/// moves it by, `WW δ / x_mark`, and the jig's, sixteen units in the last
/// place of the weight. The reading is the jig's, names where the roller
/// stood and how that point was found, and reads the wippen's mass and the
/// dimensions.
#[test]
fn the_wippen_weight_is_read_where_the_capstan_touches_the_heel_at_the_top_seat() {
    use crate::measure::measurement::{ContactFound, ContactPoint, Mount, ProcedureId, Trust};
    let mass = wippen_props();
    for (name, a, _) in touch_benches() {
        let b = bench_of(&a);
        let top = b.seats().top;
        let m = b.mounted();
        let (_, on_b) = b.hand_contact(&top.q, b.carrying(&top.q, Motion::Down));
        let wippen = m.wippen.expect("a wippen").body;
        let x_mark = m.mechanism.pose(&top.q).placement(wippen).local(on_b).x;
        let contact = m
            .capstan_on_the_heel(&top.q, TOUCHING)
            .expect("the capstan touches the heel");
        let ww = reading_of(&carrying_sheet_of(&a), ProcedureId::WippenWeight);
        let by_hand = 1000.0 * mass.mass * mass.com.x / x_mark;
        let bound = ww.value * HAND_CONTACT_M / x_mark + jig_round_off(ww.value);
        assert!(
            (ww.value - by_hand).abs() <= bound,
            "{name}: WW {} g, by hand {by_hand} g, within {bound} g",
            ww.value
        );
        assert_eq!(ww.conditions.mount, Mount::Jig, "{name}");
        assert_eq!(
            ww.conditions.capstan_heel_contact,
            Some(ContactPoint {
                at: contact.on_the_heel,
                found: ContactFound::WhereTheCapstanTouchesTheHeel {
                    part: PartCode::WippenHeelCloth
                }
            }),
            "{name}"
        );
        assert_eq!(
            ww.trust,
            Trust::of(ProcedureId::WippenWeight.reads(), &bases_of(true)),
            "{name}"
        );
    }
}

/// The roller the wippen weight of `a`'s key carrying `wippen` stands
/// under, and how its point was found.
fn roller_of(
    a: &Setup,
    wippen: &WippenAssembly,
) -> Option<crate::measure::measurement::ContactPoint> {
    use crate::measure::measurement::ProcedureId;
    let (_, mass) = test_wippen();
    let (on_frame, jig) = weighed_with(a, Some((wippen, &mass)));
    reading_of(
        &sheet_of(&on_frame, &jig, a, true),
        ProcedureId::WippenWeight,
    )
    .conditions
    .capstan_heel_contact
}

/// The lowest point of `wippen`'s heel's circle, in its frame, as the
/// trade's two quantities place it: its position along the wippen, and its
/// height below the jack's pin.
fn heel_lowest_point_of(wippen: &WippenAssembly) -> Point {
    Point::new(wippen.heel.position, wippen.jack.pin.y - wippen.heel.height)
}

/// Consistency: **with no top seat the wippen is weighed at its heel's
/// lowest point.** On a capstan 30 mm tall under a center pin 69 mm high
/// the top seat is refused, so no point where the capstan touches the heel
/// is read: the roller stands under the heel's lowest point, to the bit.
#[test]
fn with_no_top_seat_the_wippen_is_weighed_at_its_heels_lowest_point() {
    use super::seat_wippen::TALL_CAPSTAN;
    use crate::measure::measurement::{ContactFound, ContactPoint};
    let mut a = regulated();
    a.key.capstan_screw.height = TALL_CAPSTAN;
    let (mut wippen, _) = test_wippen();
    wippen.center.y = REFUSED_AT_THE_TOP_SEAT;
    assert_eq!(
        roller_of(&a, &wippen),
        Some(ContactPoint {
            at: heel_lowest_point_of(&wippen),
            found: ContactFound::NoTopSeat
        })
    );
}

/// Consistency: **with the heel on the key's top the wippen is weighed at
/// its heel's lowest point**: 100 mm behind the balance pin the capstan
/// does not touch the heel at the top seat, and the roller stands under the
/// heel's lowest point, to the bit.
#[test]
fn with_the_heel_on_the_keys_top_the_wippen_is_weighed_at_its_heels_lowest_point() {
    use crate::measure::measurement::{ContactFound, ContactPoint};
    let (wippen, _) = test_wippen();
    assert_eq!(
        roller_of(&with_the_capstan_at(100.0), &wippen),
        Some(ContactPoint {
            at: heel_lowest_point_of(&wippen),
            found: ContactFound::CapstanOffTheHeel
        })
    );
}

/// Consistency: **a heel at its center pin refuses the wippen weight,
/// naming where the roller stands.** With the heel's position at nothing,
/// its lowest point stands under the center pin, where the capstan does not
/// reach it: the roller would stand there, at no arm, and the wippen weight
/// is refused naming that place, 0 m to the bit; WBW is refused as computed
/// from it.
#[test]
fn a_heel_at_its_center_pin_refuses_the_wippen_weight_naming_where_the_roller_stands() {
    use crate::measure::measurement::ProcedureId;
    use crate::measure::weighing::Refused;
    const AT_THE_CENTER_PIN: f64 = 0.0;
    let a = regulated();
    let (mut wippen, mass) = test_wippen();
    wippen.heel.position = AT_THE_CENTER_PIN;
    let (on_frame, jig) = weighed_with(&a, Some((&wippen, &mass)));
    let sheet = sheet_of(&on_frame, &jig, &a, true);
    let Some(Err(Refused::RollerNotInFrontOfTheCenterPin { at_m })) =
        sheet.get(ProcedureId::WippenWeight)
    else {
        panic!("{:?}", sheet.get(ProcedureId::WippenWeight))
    };
    assert_eq!(at_m.to_bits(), AT_THE_CENTER_PIN.to_bits());
    assert_eq!(
        sheet.get(ProcedureId::WippenBalanceWeight),
        Some(&Err(Refused::From {
            reading: ProcedureId::WippenWeight
        }))
    );
}

/// Consistency: **WBW is read from the wippen weight and the key weight
/// ratio as the sheet reads them**: its value the product of the two in
/// full, the bits `Touchweight::wippen_balance_weight_g` gives; computed
/// from the two; its solves the wippen weight's; and refused, as computed
/// from it, where either is, the wippen weight first.
#[test]
fn wbw_is_read_from_the_wippen_weight_and_the_key_weight_ratio_as_the_sheet_reads_them() {
    use crate::measure::measurement::ProcedureId;
    use crate::measure::weighing::Refused;
    use crate::measure::Touchweight;
    let a = regulated();
    let (wippen, mass) = test_wippen();
    let (on_frame, jig) = weighed_with(&a, Some((&wippen, &mass)));
    let weighing = weighing_of(&on_frame, &jig, &a, true);
    let sheet = weighing.sheet(&mut Statics::new());
    let ww = reading_of(&sheet, ProcedureId::WippenWeight);
    let kr = reading_of(&sheet, ProcedureId::KeyWeightRatio);
    let wbw = reading_of(&sheet, ProcedureId::WippenBalanceWeight);
    let touchweight = Touchweight {
        wippen_weight_g: ww.value,
        key_ratio: kr.value,
        ..Touchweight::default()
    };
    assert_eq!(
        wbw.value.to_bits(),
        touchweight.wippen_balance_weight_g().to_bits()
    );
    assert_eq!(
        wbw.conditions.from,
        vec![ProcedureId::WippenWeight, ProcedureId::KeyWeightRatio]
    );
    assert_eq!(wbw.report, ww.report);
    let refused = Err(Refused::NoWippenAssembly);
    assert_eq!(
        weighing.wippen_balance_weight(&refused, &refused),
        Err(Refused::From {
            reading: ProcedureId::WippenWeight
        })
    );
    assert_eq!(
        weighing.wippen_balance_weight(&Ok(ww), &refused),
        Err(Refused::From {
            reading: ProcedureId::KeyWeightRatio
        })
    );
}

/// Consistency: **WBW is shown from its two readings as read, not from its
/// value in full**, as a technician computes it from what he read: a wippen
/// weight of 16.94 g, read 16.9 g, and a key weight ratio of 0.5249, read
/// 0.52, show 16.9 × 0.52 = 8.788, 8.8 g, where the value in full,
/// 8.892 g, would read 8.9 g.
#[test]
fn wbw_is_shown_from_its_two_readings_as_read_not_from_its_value_in_full() {
    use crate::measure::measurement::{ProcedureId, Rounding};
    const WW_IN_FULL: f64 = 16.94;
    const WW_READ: f64 = 16.9;
    const KR_IN_FULL: f64 = 0.5249;
    const KR_READ: f64 = 0.52;
    const SHOWN: f64 = 8.8;
    let a = regulated();
    let (wippen, mass) = test_wippen();
    let (on_frame, jig) = weighed_with(&a, Some((&wippen, &mass)));
    let weighing = weighing_of(&on_frame, &jig, &a, true);
    let sheet = weighing.sheet(&mut Statics::new());
    let named = |procedure, value, shown| {
        let mut m = reading_of(&sheet, procedure);
        m.value = value;
        m.shown = shown;
        Ok(m)
    };
    let wbw = weighing
        .wippen_balance_weight(
            &named(ProcedureId::WippenWeight, WW_IN_FULL, WW_READ),
            &named(ProcedureId::KeyWeightRatio, KR_IN_FULL, KR_READ),
        )
        .expect("WBW is read");
    assert_eq!(wbw.shown.to_bits(), SHOWN.to_bits());
    assert_ne!(
        Rounding::Nearest.apply(wbw.value, 10.0, 0.0).to_bits(),
        SHOWN.to_bits()
    );
}

/// Consistency: **a WBW on a half tenth shows the tenth above**, as a
/// technician rounds the product of what he read: 15.0 g × 0.57 = 8.55 g
/// shows 8.6 g, and 17.5 g × 0.58 = 10.15 g shows 10.2 g, where the product
/// of the two doubles falls just under the half.
#[test]
fn a_wbw_on_a_half_tenth_shows_the_tenth_above() {
    use crate::measure::measurement::ProcedureId;
    const ON_A_HALF_TENTH: [(f64, f64, f64); 2] = [(15.0, 0.57, 8.6), (17.5, 0.58, 10.2)];
    let a = regulated();
    let (wippen, mass) = test_wippen();
    let (on_frame, jig) = weighed_with(&a, Some((&wippen, &mass)));
    let weighing = weighing_of(&on_frame, &jig, &a, true);
    let sheet = weighing.sheet(&mut Statics::new());
    let read_as = |procedure, shown| {
        let mut m = reading_of(&sheet, procedure);
        m.shown = shown;
        Ok(m)
    };
    for (ww_read, kr_read, shown) in ON_A_HALF_TENTH {
        let wbw = weighing
            .wippen_balance_weight(
                &read_as(ProcedureId::WippenWeight, ww_read),
                &read_as(ProcedureId::KeyWeightRatio, kr_read),
            )
            .expect("WBW is read");
        assert_eq!(
            wbw.shown.to_bits(),
            shown.to_bits(),
            "{ww_read} g × {kr_read}"
        );
    }
}

/// Consistency: **the sheet's wippen balance weight reads the force
/// convention**: the wippen weight and the key weight ratio the sheet reads
/// pass `Touchweight::conventions_are_sane`, a key weight ratio between 0.3
/// and 1.0 and a WBW under the wippen weight, which the geometric
/// convention, the ratio's reciprocal, would fail.
#[test]
fn the_sheets_wippen_balance_weight_reads_the_force_convention() {
    use crate::measure::measurement::ProcedureId;
    use crate::measure::Touchweight;
    let sheet = carrying_sheet_of(&regulated());
    let touchweight = Touchweight {
        wippen_weight_g: reading_of(&sheet, ProcedureId::WippenWeight).value,
        key_ratio: reading_of(&sheet, ProcedureId::KeyWeightRatio).value,
        ..Touchweight::default()
    };
    assert!(touchweight.conventions_are_sane(), "{touchweight:?}");
    assert_eq!(
        touchweight.wippen_balance_weight_g().to_bits(),
        reading_of(&sheet, ProcedureId::WippenBalanceWeight)
            .value
            .to_bits()
    );
}

/// The wippen's share of the band's middle at one state, by hand, and what
/// it is made of.
#[derive(Debug, Clone, Copy)]
struct Share {
    /// `T(s) = 1000 m_w X(φ) d_k / (d_w a)`, grams at the measuring point.
    grams: f64,
    /// `X(φ) / com.x`: the wippen's attitude against the protocol's level.
    attitude: f64,
    /// `x_meas / a(θ)`: the key's attitude against the jig's level.
    key_attitude: f64,
    /// `(d_k / |x_c|)(x_mark / d_w)`: the line of action against the
    /// verticals the jigs weigh along, and the contact's roll.
    line: f64,
    /// `d_k / d_w`: how far the wippen turns per turn of the key.
    ratio: f64,
    /// The measuring point's arm about the key's bearing, meters.
    arm: f64,
}

impl Bench {
    /// The wippen's share of the band's middle at `q`, carried through the
    /// pieces of `feature`, by hand, against the arms the jigs read:
    /// `x_c` along the key, `x_mark` along the wippen, `x_meas` the
    /// measuring point's on the level jig.
    ///
    /// The contact pushes along its normal `n` through its point `P`, so the
    /// key's angle feels it at the arm `d_k = |(P − B) × n|` about the key's
    /// bearing `B` and the wippen's at `d_w = |(P − C) × n|` about its center
    /// pin `C`; the wippen's weight, `m_w g X(φ)` about `C`, its center's arm
    /// `X(φ) = com.x cos φ − com.y sin φ` at its angle `φ`, comes onto the
    /// key's angle times `d_k / d_w`, and onto the measuring point over its
    /// arm `a(θ)`.
    fn share(&self, q: &[f64], feature: Feature, x_c: f64, x_mark: f64, x_meas: f64) -> Share {
        let m = self.mounted();
        let w = m.wippen.expect("a wippen");
        let pose = m.mechanism.pose(q);
        let (p, _) = self.hand_contact(q, feature);
        let n = self.hand_normal(q, feature);
        let cross = |o: Point| ((p.x - o.x) * n.y - (p.y - o.y) * n.x).abs();
        let bearing = pose.world(MaterialPoint {
            body: m.key,
            at: Point::new(0.0, 0.0),
        });
        let (d_k, d_w) = (cross(bearing), cross(self.wippen.center));
        let mass = self.wippen_mass.props(&self.wippen);
        let phi = q[w.angle.0];
        let along = mass.com.x * libm::cos(phi) - mass.com.y * libm::sin(phi);
        let arm =
            pose.world(MaterialPoint {
                body: m.key,
                at: self.k.measuring_point,
            })
            .x - bearing.x;
        Share {
            grams: 1000.0 * mass.mass * along * d_k / (d_w * arm),
            attitude: along / mass.com.x,
            key_attitude: x_meas / arm,
            line: (d_k / x_c.abs()) * (x_mark / d_w),
            ratio: d_k / d_w,
            arm,
        }
    }
}

/// Consistency, by hand: **the wippen moves balance weight by WW × KR but
/// for the terms derived by hand.** The trade's equation of balance with no
/// hammer reads `BW + FW = WW × KR`; on the model,
/// `BW + FW − WBW = (BW_alone + FW) + (ΔBW − WBW)`, `BW_alone` the key
/// alone's and `ΔBW` what the wippen moves it by. The first bracket is the
/// bare key's tilt ([`super::weighing`]'s
/// `the_equation_of_balance_closes_on_a_bare_key`); this test holds the
/// second, term by term:
///
/// `ΔBW_hand − WBW_hand = ½ Σ_s (T(s) − WBW_hand) + H + K_s`, where
///
/// - `s` is each state the walks' thresholds fall at, `s_D` going down and
///   `s_U` coming up: on these benches the seats, the band by hand over the
///   sweep's states passing neither seat's threshold, which the test checks;
/// - `T(s) = 1000 m_w X(φ_s) d_k / (d_w a_s)` is the wippen's share of the
///   band's middle there ([`Bench::share`]), and
///   `T(s) / WBW_hand = A_w · A_k · L`: `A_w = X(φ_s) / com.x` the wippen's
///   attitude against the protocol's level, `A_k = x_meas / a(θ_s)` the
///   key's against the jig's, `L = (d_k / x_c)(x_mark / d_w)` the line of
///   action against the verticals the two jigs weigh along, with the
///   contact's roll;
/// - `WBW_hand = 1000 m_w (com.x / x_mark)(|x_c| / x_meas)`, the two jigs by
///   hand at the contact the top seat reads;
/// - `H = ½ · 1000 b_w (|r_D| / a_D − |r_U| / a_U) / g` is the wippen
///   center's friction, carried onto each threshold at its own ratio
///   `r = d_k / d_w`, which does not cancel between the two;
/// - `K_s`, the key's own band at `s_D` and `s_U` against the key alone's
///   own seats ([`band_at`]), half of each difference: nothing where the
///   states are the seats, but for what the wippen moves the seats by.
///
/// The bound adds what each side is read to: the walks' resolution for each
/// balance weight; half the band by hand's bound at each state
/// ([`Bench::hand_within`]); and `WBW_hand`'s, `WBW (δ / |x_c| + δ /
/// x_mark)` from the hand's contact and the two jigs' round-off. The
/// wippen's attitude alone moves it by `½ Σ_s (A_w(s) − 1) · WBW_hand`,
/// more than ten times that bound, so a WBW read with the wippen level at
/// its seats, or an equation closed without its terms, fails.
#[test]
fn the_wippen_moves_balance_weight_by_ww_times_kr_but_for_the_terms_derived_by_hand() {
    use crate::measure::measurement::ProcedureId;
    use crate::measure::oracle::band_at;
    for (name, a) in [
        ("the flat capstan", regulated()),
        ("the 9.7 mm dome", with_a_domed_capstan()),
    ] {
        let b = bench_of(&a);
        let seats = b.seats();
        let m = b.mounted();
        let w = m.wippen.expect("a wippen");
        let carrying = carrying_sheet_of(&a);
        let (bare_frame, bare_jig) = weighed_with(&a, None);
        let bare = sheet_of(&bare_frame, &bare_jig, &a, false);
        let value = |sheet, procedure| reading_of(sheet, procedure).value;
        let moved =
            value(&carrying, ProcedureId::BalanceWeight) - value(&bare, ProcedureId::BalanceWeight);
        let wbw = value(&carrying, ProcedureId::WippenBalanceWeight);
        let ww = value(&carrying, ProcedureId::WippenWeight);
        let kr = value(&carrying, ProcedureId::KeyWeightRatio);

        let top = &seats.top.q;
        let pose = m.mechanism.pose(top);
        let (on_a, on_b) = b.hand_contact(top, b.carrying(top, Motion::Down));
        let x_c = pose.placement(m.key).local(on_a).x;
        let x_mark = pose.placement(w.body).local(on_b).x;
        let x_meas = b.k.measuring_point.x;
        let mass = wippen_props();
        let wbw_hand = 1000.0 * mass.mass * (mass.com.x / x_mark) * (x_c.abs() / x_meas);

        let band = b.band(&seats);
        let [down, up] = b.band_by_hand(&seats, &band);
        let (at_the_top, within_d) = b.at_seat(&seats.top, Motion::Down);
        let (at_the_bottom, within_u) = b.at_seat(&seats.bottom, Motion::Up);
        assert!(
            down.iter().all(|h| h.grams <= at_the_top.down_g)
                && up.iter().all(|h| h.grams >= at_the_bottom.up_g),
            "{name}: the band by hand passes its seats' thresholds inside the sweep"
        );
        let (s_d, s_u) = (seats.top.q.clone(), seats.bottom.q.clone());
        let share_d = b.share(&s_d, b.carrying(&s_d, Motion::Down), x_c, x_mark, x_meas);
        let share_u = b.share(&s_u, b.carrying(&s_u, Motion::Up), x_c, x_mark, x_meas);

        let friction = 0.5
            * 1000.0
            * b.wippen_friction()
            * (share_d.ratio / share_d.arm - share_u.ratio / share_u.arm)
            / G;
        let key_mass = mass_of(&a.key).expect("the test key is weighed");
        let key_band = |q: &[f64]| {
            band_at(
                &[(key_mass.mass, key_mass.com)],
                b.k.measuring_point,
                b.bushings,
                q[m.angle.0],
            )
        };
        let alone = crate::parts::fixtures::on_frame(
            &a,
            crate::parts::frame::KeyKind::Natural,
            b.bushings,
            true,
        );
        let alone_seats = alone
            .seating()
            .seats(&mut Statics::new())
            .expect("the key alone seats");
        let key_own = 0.5 * (key_band(&s_d).down_g - key_band(&alone_seats.top.q).down_g)
            + 0.5 * (key_band(&s_u).up_g - key_band(&alone_seats.bottom.q).up_g);
        let by_hand =
            0.5 * ((share_d.grams - wbw_hand) + (share_u.grams - wbw_hand)) + friction + key_own;

        let jigs = jig_round_off(ww) + jig_round_off(kr) * ww;
        let wbw_within = wbw_hand * (HAND_CONTACT_M / x_c.abs() + HAND_CONTACT_M / x_mark) + jigs;
        let bound = 2.0 * walk_resolution() + 0.5 * (within_d + within_u) + wbw_within;
        let attitude = 0.5 * ((share_d.attitude - 1.0) + (share_u.attitude - 1.0)) * wbw_hand;
        let terms = format!(
            "A_w {:.6} and {:.6}, A_k {:.6} and {:.6}, L {:.6} and {:.6}, H {friction:.6} g, \
             K_s {key_own:.6} g",
            share_d.attitude,
            share_u.attitude,
            share_d.key_attitude,
            share_u.key_attitude,
            share_d.line,
            share_u.line
        );
        assert!(
            ((moved - wbw) - by_hand).abs() <= bound,
            "{name}: ΔBW {moved} g less WBW {wbw} g, by hand {by_hand} g, within {bound} g; \
             {terms}"
        );
        assert!(
            attitude.abs() > 10.0 * bound,
            "{name}: the wippen's attitude moves it {attitude} g, the bound {bound} g; {terms}"
        );
    }
}
