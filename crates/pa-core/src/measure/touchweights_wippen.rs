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
        let m = &self.mounted().mechanism;
        let pose = m.pose(q);
        let shape = |piece: Piece| {
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
        };
        match (shape(feature.a), shape(feature.b)) {
            (Shape::Disc(c, r), Shape::Disc(d, s)) => distance(c, d) - r - s,
            (Shape::Disc(c, r), Shape::Line(p, l)) | (Shape::Line(p, l), Shape::Disc(c, r)) => {
                let (dx, dy) = (l.x - p.x, l.y - p.y);
                ((c.x - p.x) * dy - (c.y - p.y) * dx) / (dx * dx + dy * dy).sqrt() - r
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
    const REFUSED_AT_THE_TOP_SEAT: f64 = 0.069;
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
    (on_frame, jig, a)
}

/// The sheet of a key on its frame and on the jig, as the bench reads it.
fn sheet_of(
    on_frame: &crate::parts::mounts::OnItsFrame,
    jig: &crate::measure::jig::KeyJig,
    a: &Setup,
    carries_a_wippen: bool,
) -> crate::measure::weighing::Sheet {
    use crate::measure::measurement::{Bases, Basis, WippenAssemblyBases};
    crate::measure::weighing::Weighing {
        on_frame,
        jig,
        measuring_point: a.protocol.measuring_point_on_key(&a.key),
        measuring_point_from_lip_m: a.protocol.measuring_point_from_lip,
        thinnest_paper_m: a.frame.drawer.thinnest(),
        bases: Bases {
            key_mass: Basis::SetByHand,
            center_of_mass_height: Basis::Estimated,
            key_friction: Basis::Assumed,
            geometry: Basis::Assumed,
            wippen_assembly: carries_a_wippen.then_some(WippenAssemblyBases {
                mass: Basis::Published,
                friction: Basis::Published,
            }),
        },
    }
    .sheet(&mut Statics::new())
}

/// Consistency: **both key friction weights read the same bits with the
/// wippen and without**, and so do front weight and the key weight ratio:
/// the key friction weights weigh the key alone, its bushings, the same key
/// on the same frame the bench mounts beside the key with its wippen; the
/// jig weighs the key alone too.
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
        ProcedureId::KeyWeightRatio,
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
