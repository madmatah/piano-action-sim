//! The driven sweep: the band, drawn.
//!
//! # What it reads
//!
//! Gram weights read the band at its two edges: the down weight, the least
//! weight on the measuring point that takes the key from its top seat to its
//! bottom one, and the up weight, the most it lifts back ([`super::walk`]).
//! The sweep reads the band whole. It drives the key through its stroke in
//! position, as a finger driving it slowly would, and reads at every step the
//! force on the measuring point that holds it there: the down curve on the
//! way down, the up curve on the way back. Between the two curves the key
//! stays wherever it is put, held by its friction; their distance apart is
//! the friction, their middle the key's balance. The down weight is the
//! greatest force the down curve needs over the window, the up weight the
//! least the up curve holds: the same two numbers as the walk's, read by
//! another procedure, so each checks the other. It is the static form of
//! driving in position and comparing forces (`docs/FOUNDATIONS.md` > What a test proves; drive in position, compare forces).
//!
//! # The procedure
//!
//! 1. Seat the key ([`super::seat`]): at the top and at the bottom.
//! 2. From the top seat, hold the key's angle where the measuring point
//!    stands one [`STEP`] lower, then two, and so on through every whole step
//!    strictly inside the window, the measuring point's travel from the top
//!    seat to the bottom one. At each step the coordinates the angle leaves
//!    free, the key's lift off its balance punching, are solved from where
//!    the last step left them, friction on, under the key's weight and
//!    whatever stands on it; the angle's reaction is read.
//! 3. Then back from the bottom seat, through the same travels upwards.
//! 4. The reaction on the angle is a torque. Read through the measuring
//!    point's arm about the key's bearing, the arm it has at that step, it is
//!    the force a finger there would apply: on the way down with the
//!    bushings' friction at its bound against the motion, the down curve;
//!    on the way up, the up curve. Still at that travel, the key holds under
//!    any force between the two.
//!
//! The angle a travel is held at is found exactly: the key turns about its
//! bearing on its punching, so the measuring point stands on a circle about
//! it, the circle the top seat's lever gives
//! ([`crate::mechanism::Mechanism::jacobian_column`]), and an arcsine of the
//! travel gives the angle. The travel each reading is drawn at is read off
//! the pose the solve leaves, not assumed.
//!
//! # Not at the seats
//!
//! At a seat the stop that holds the key takes whatever the finger leaves,
//! so the force there is not one number: the down curve starts on the first
//! step off the top seat and the up curve on the first step off the bottom
//! one. Where an extreme lies at a seat, as it does on a key whose balance
//! changes one way through its stroke, the sweep reads it a step inside the
//! seat at most, and so inside the walk's edge by what the curve changes
//! over that step; where it lies inside the stroke, the walk reads it too,
//! through the stalls it finds.
//!
//! # The punching stays loaded
//!
//! A torque on the key's angle and a force at its measuring point turn the
//! key alike; they do not press it on its balance punching alike. A finger
//! pressing down adds to what the punching carries, and one pulling up takes
//! from it, past which the key would lift off its punching instead of
//! turning. So at every step the punching's load under the finger's force,
//! the key's weight and what stands on it less what the other contacts
//! carry, is checked, and a sweep under which it would carry nothing is
//! refused ([`SweepRefusal::LiftsOffItsPunching`]): the reaction no longer
//! reads as a force at the measuring point there. A key on a pivot is held
//! both ways and needs no check.

use crate::contact::ContactCode;
use crate::geometry::Point;
use crate::mechanism::{MaterialPoint, Prescribed, Problem};
use crate::parts::mounts::Mounted;
use crate::parts::verdicts::TOUCHING;
use crate::solve::{Equilibrium, Friction, Holder, Refusal, SolveReport, Statics};
use crate::G;

use super::seat::{Seating, Seats};
use super::walk::Stroke;

/// How far the measuring point travels between two readings of the sweep,
/// meters: 0.05 mm.
///
/// A key's dip at its measuring point is about ten millimeters, two hundred
/// steps each way, and the curves change by a few thousandths of a gram over
/// one step on the shipped keys: the extremes the sweep reads one step off a
/// seat stand that far from the walk's in full, far under the gram the walk
/// reads them to, though not always on the same side of a whole gram.
pub const STEP: f64 = 0.000_05;

/// One reading of a curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// How far the measuring point stands below where it stands at the top
    /// seat, meters, read off the pose the solve left.
    pub travel: f64,
    /// The key's angle the step held, radians, nose-up.
    pub angle: f64,
    /// The force on the measuring point that drives the key through that
    /// travel, grams, positive pressing down: the reaction on the key's
    /// angle through the measuring point's arm, with the bushings' friction
    /// against the motion.
    pub grams: f64,
}

/// The band: the force on the measuring point through the key's stroke, down
/// and back.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    /// The down curve, from the first step off the top seat to the last
    /// before the bottom one.
    pub down: Vec<Sample>,
    /// The up curve, from the first step off the bottom seat to the last
    /// before the top one: the same travels, the other way.
    pub up: Vec<Sample>,
    /// The greatest force on the down curve: the down weight, read on the
    /// band.
    pub down_weight: Sample,
    /// The least force on the up curve: the up weight, read on the band.
    pub up_weight: Sample,
    /// The window: how far the measuring point travels from the top seat to
    /// the bottom one, meters.
    pub window: f64,
    /// The step between two readings, meters.
    pub step: f64,
    /// Every step's static solve, added up.
    pub report: SolveReport,
}

/// Why a sweep cannot be taken through the key's stroke.
#[derive(Debug, Clone, PartialEq)]
pub enum SweepRefusal {
    /// The two seats are one: the key has no stroke between them, as a key
    /// riding on its two outer stops has none.
    NoTravel,
    /// The measuring point does not stand in front of the key's bearing, so
    /// a force on it does not turn the key down.
    NoLever,
    /// Under the force a finger at the measuring point would need there, the
    /// key's balance punching would carry nothing, or a stop would hold the
    /// key off it: the key would lift off its punching instead of turning,
    /// and the reaction no longer reads as that force.
    LiftsOffItsPunching {
        /// On the way down or up.
        stroke: Stroke,
        /// How far the measuring point stood below the top seat, meters.
        travel: f64,
        /// The force the finger would need there, grams.
        grams: f64,
    },
    /// A step's solve leaned on the top of the slide's range: the key left
    /// its balance pin, and its frame.
    LiftsOffItsFrame,
    /// A step found no equilibrium in the modeled world: these contacts
    /// fight at the angle it held.
    NoEquilibrium {
        /// The contacts that fight, in the mechanism's order; empty where the
        /// key leans on a limit.
        on: Vec<ContactCode>,
    },
    /// A step's solve reached its cap first.
    NotConverged {
        /// Steps taken.
        iterations: u32,
        /// The stationarity residual left, newtons.
        residual: f64,
    },
    /// The mounted key cannot be solved as given.
    Mechanism(Problem),
}

/// The measuring point on a circle about the key's bearing, as the top seat
/// stands it: what turns a travel into the angle that gives it.
struct Circle {
    /// The key's angle at the top seat, radians.
    top: f64,
    /// The measuring point's distance from the bearing, meters.
    radius: f64,
    /// How high it stands above the bearing at the top seat, meters.
    above: f64,
    /// Its direction from the bearing at the top seat, radians from the
    /// horizontal, toward the player.
    bearing: f64,
}

impl Circle {
    /// The key's angle at which the measuring point stands `travel` below
    /// where it stands at the top seat, the bearing where it stands there:
    /// `θ_top + asin((h − s) / r) − φ`.
    fn angle(&self, travel: f64) -> f64 {
        self.top + libm::asin((self.above - travel) / self.radius) - self.bearing
    }
}

impl Seating<'_> {
    /// Sweeps the key through its stroke, from the top seat of `seats`, which
    /// must be this seating's, to the bottom one and back.
    pub fn sweep(&self, statics: &mut Statics, seats: &Seats) -> Result<Band, SweepRefusal> {
        self.sweep_by(statics, seats, STEP)
    }

    /// The sweep in steps of `step` meters instead of [`STEP`]: for the test
    /// that a step half as long reads the same extremes, within the curves'
    /// change over a step, on every instrument a profile builds.
    ///
    /// # Panics
    ///
    /// When `step` is not more than nothing.
    pub fn sweep_by(
        &self,
        statics: &mut Statics,
        seats: &Seats,
        step: f64,
    ) -> Result<Band, SweepRefusal> {
        assert!(step > 0.0, "a sweep steps forward");
        if seats.top.rides() || seats.bottom.rides() {
            return Err(SweepRefusal::NoTravel);
        }
        let m = &self.mounted.mechanism;
        let angle = self.mounted.angle;
        let at = self.at();
        let height = |q: &[f64]| m.pose(q).world(at).y;
        let top = height(&seats.top.q);
        let window = top - height(&seats.bottom.q);
        if window.is_nan() || window <= 0.0 {
            return Err(SweepRefusal::NoTravel);
        }
        // The lever-arm identity: the measuring point moves per radian by
        // `ẑ × (P − O)`, so `P − O` is that speed turned back a quarter turn.
        let lever = m.jacobian_column(&m.pose(&seats.top.q), at, angle);
        let (ahead, above) = (lever.y, -lever.x);
        if ahead.is_nan() || ahead <= 0.0 {
            return Err(SweepRefusal::NoLever);
        }
        let circle = Circle {
            top: seats.top.q[angle.0],
            radius: (ahead * ahead + above * above).sqrt(),
            above,
            bearing: libm::atan2(above, ahead),
        };

        // Every whole step strictly inside the window.
        let mut travels = Vec::new();
        let mut k: u32 = 1;
        while f64::from(k) * step < window {
            travels.push(f64::from(k) * step);
            k += 1;
        }
        if travels.is_empty() {
            return Err(SweepRefusal::NoTravel);
        }

        let mut report = SolveReport::NOTHING;
        let mut pass = |stroke: Stroke, from: &[f64], travels: &mut dyn Iterator<Item = f64>| {
            let mut q = from.to_vec();
            let mut samples = Vec::new();
            for travel in travels {
                let (sample, eq) = self.reading(statics, &q, circle.angle(travel), stroke, top)?;
                report = report.and(eq.report);
                samples.push(sample);
                q = eq.q;
            }
            Ok::<_, SweepRefusal>(samples)
        };
        let down = pass(Stroke::Down, &seats.top.q, &mut travels.iter().copied())?;
        let up = pass(
            Stroke::Up,
            &seats.bottom.q,
            &mut travels.iter().rev().copied(),
        )?;

        let mut down_weight = down[0];
        for s in &down[1..] {
            if s.grams > down_weight.grams {
                down_weight = *s;
            }
        }
        let mut up_weight = up[0];
        for s in &up[1..] {
            if s.grams < up_weight.grams {
                up_weight = *s;
            }
        }
        Ok(Band {
            down,
            up,
            down_weight,
            up_weight,
            window,
            step,
            report,
        })
    }

    /// The measuring point, as a point of the key.
    fn at(&self) -> MaterialPoint {
        MaterialPoint {
            body: self.mounted.key,
            at: self.measuring_point,
        }
    }

    /// One step of the sweep: the key's angle held at `theta`, the rest
    /// solved from `from`, and the force on the measuring point read there,
    /// moving as `stroke` says; the measuring point's travel read below
    /// `top`, its height at the top seat.
    fn reading(
        &self,
        statics: &mut Statics,
        from: &[f64],
        theta: f64,
        stroke: Stroke,
        top: f64,
    ) -> Result<(Sample, Equilibrium), SweepRefusal> {
        let m = &self.mounted.mechanism;
        let angle = self.mounted.angle;
        let held = [Prescribed {
            coord: angle,
            value: theta,
        }];
        let eq = self
            .mounted
            .solve_held(statics, from, self.standing, &held, Friction::On)
            .map_err(|r| refused(self.mounted, r))?;
        let pose = m.pose(&eq.q);
        // A gram on the measuring point turns the key by its weight times the
        // point's vertical speed per radian, downwards.
        let per_gram = -m.jacobian_column(&pose, self.at(), angle).y * 1e-3 * G;
        if per_gram.is_nan() || per_gram >= 0.0 {
            return Err(SweepRefusal::NoLever);
        }
        // Moving down, the bushings turn the key up by their bound and the
        // finger must give that much more; moving up, as much less.
        let bound = m.coords()[angle.0].friction;
        let torque = match stroke {
            Stroke::Down => eq.reactions[angle.0] - bound,
            Stroke::Up => eq.reactions[angle.0] + bound,
        };
        let grams = torque / per_gram;
        let travel = top - pose.world(self.at()).y;
        if !self.punching_loaded(&eq, grams) {
            return Err(SweepRefusal::LiftsOffItsPunching {
                stroke,
                travel,
                grams,
            });
        }
        Ok((
            Sample {
                travel,
                angle: theta,
                grams,
            },
            eq,
        ))
    }

    /// Whether the key stands on its balance punching at `eq` and the
    /// punching carries it under a finger pressing `grams` on the measuring
    /// point: what the key's weight and the loads standing on it and the
    /// finger press down, less what the other contacts push up, the solve's
    /// own split of them, is more than nothing. Always on a pivot.
    fn punching_loaded(&self, eq: &Equilibrium, grams: f64) -> bool {
        let Some(lift) = &self.mounted.lift else {
            return true;
        };
        if self.mounted.lift_at(&eq.q) > TOUCHING {
            return false;
        }
        let m = &self.mounted.mechanism;
        let weight = m.bodies()[self.mounted.key.0]
            .mass
            .map_or(0.0, |mass| mass.mass * G);
        let finger = Point::new(0.0, -grams * 1e-3 * G);
        let pressed: f64 = self.standing.iter().map(|l| l.force.y).sum::<f64>() + finger.y;
        let others: f64 = eq
            .supports
            .iter()
            .filter(|s| matches!(s.by, Holder::Contact { pair, .. } if pair != lift.pair))
            .map(|s| s.force * s.normal.y)
            .sum();
        weight - pressed - others > 0.0
    }
}

/// A static solve's refusal, as the sweep's.
fn refused(mounted: &Mounted, refusal: Refusal) -> SweepRefusal {
    if mounted.lifts_off_its_frame(&refusal) {
        return SweepRefusal::LiftsOffItsFrame;
    }
    let m = &mounted.mechanism;
    match refusal {
        Refusal::Mechanism(p) => SweepRefusal::Mechanism(p),
        Refusal::NotConverged {
            iterations,
            residual,
        } => SweepRefusal::NotConverged {
            iterations,
            residual,
        },
        Refusal::NoEquilibrium { on } => {
            let mut pairs: Vec<u16> = on
                .iter()
                .filter_map(|h| match *h {
                    Holder::Contact { pair, .. } => Some(pair),
                    Holder::Limit { .. } => None,
                })
                .collect();
            pairs.sort_unstable();
            pairs.dedup();
            SweepRefusal::NoEquilibrium {
                on: pairs
                    .into_iter()
                    .map(|p| m.pairs()[usize::from(p)].code)
                    .collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::oracle::band_at;
    use crate::measure::seat::SEAT_MARGIN;
    use crate::measure::walk::{HALVINGS, STEP_G};
    use crate::mechanism::{AxisFriction, MassProps, PointLoad};
    use crate::parts::fixtures::{mass_of, regulated, Setup};
    use crate::parts::frame::KeyKind;
    use crate::parts::mounts::{on_its_frame, KeyMount, OnItsFrame};
    use crate::parts::KeyMass;
    use crate::solve::statics::{GAP_TOLERANCE, RESIDUAL_TOLERANCE};

    /// A key on its frame as the bench mounts it, its bushings and its mass.
    struct Key {
        a: Setup,
        on_frame: OnItsFrame,
        /// The bushings' bound, newton-meters.
        bound: f64,
        mass: MassProps,
    }

    impl Key {
        /// `a`'s key, in `mount`, its bushings holding `kf_g` grams at its
        /// measuring point with the key level, of mass `mass`.
        fn new(a: &Setup, mount: KeyMount, kf_g: f64, mass: MassProps) -> Self {
            let bound = AxisFriction::of_weight_at(kf_g, a.measuring_point()).bound_n_m;
            let on_frame =
                on_its_frame(mount, &a.key, &a.frame, KeyKind::Natural, Some(mass), bound)
                    .expect("the key mounts on its frame");
            Self {
                a: a.clone(),
                on_frame,
                bound,
                mass,
            }
        }

        /// The seated-pose procedure on this key, `standing` on it.
        fn seating<'a>(&'a self, standing: &'a [PointLoad]) -> Seating<'a> {
            Seating {
                mounted: &self.on_frame.mounted,
                capstan: self.on_frame.capstan,
                measuring_point: self.a.protocol.measuring_point_on_key(&self.a.key),
                margin: SEAT_MARGIN,
                standing,
            }
        }

        /// The band, `standing` on the key, in steps of `step`; and the
        /// seats it was swept between.
        fn band_by(&self, standing: &[PointLoad], step: f64) -> (Band, Seats) {
            let seating = self.seating(standing);
            let mut statics = Statics::new();
            let seats = seating.seats(&mut statics).expect("the key seats");
            let band = seating
                .sweep_by(&mut statics, &seats, step)
                .expect("the band is drawn");
            (band, seats)
        }

        /// The band in the sweep's own steps.
        fn band(&self) -> Band {
            self.band_by(&[], STEP).0
        }

        /// The walk's down and up weights from the same seats.
        fn walks(&self) -> (f64, f64) {
            let seating = self.seating(&[]);
            let mut statics = Statics::new();
            let seats = seating.seats(&mut statics).expect("the key seats");
            let [d, u] = [Stroke::Down, Stroke::Up].map(|stroke| {
                seating
                    .walk(&mut statics, &seats, stroke)
                    .expect("the key walks")
                    .grams
            });
            (d, u)
        }

        /// The measuring point's height in `seat`, meters.
        fn height(&self, seat: &crate::measure::seat::Seat) -> f64 {
            seat.height(
                &self.on_frame.mounted,
                self.a.protocol.measuring_point_on_key(&self.a.key),
            )
        }

        /// The measuring point's arm about the pin at attitude `theta`,
        /// meters.
        fn arm(&self, theta: f64) -> f64 {
            let p = self.a.protocol.measuring_point_on_key(&self.a.key);
            p.x * libm::cos(theta) - p.y * libm::sin(theta)
        }

        /// The bushings' bound as grams at the measuring point at attitude
        /// `theta`.
        fn friction_g(&self, theta: f64) -> f64 {
            1000.0 * self.bound / (G * self.arm(theta))
        }

        /// How far a reading of this key may stand from the same reading
        /// worked another way by rounding alone, grams: each is a sum of a
        /// few products, the key's weight times an arm under 0.3 m and the
        /// bushings' bound, each rounded at its last bit, over the measuring
        /// point's arm, at least 0.2 m; 64 units in the last place of that.
        fn rounding(&self) -> f64 {
            64.0 * f64::EPSILON * 1000.0 * (self.mass.mass * G * 0.3 + self.bound) / (G * 0.2)
        }
    }

    /// The keys the tests sweep: the regulated bench, the same key with three
    /// times the friction, with none, and weighed back-heavy.
    fn keys() -> Vec<(&'static str, Key)> {
        let a = regulated();
        let mass = mass_of(&a.key).expect("the test key encloses wood");
        let back_heavy = KeyMass::weighed(&a.key, 151.1, -8.1, 0.013)
            .expect("a key's weighing")
            .props(mass.com.y)
            .expect("a weighed key has a mass");
        vec![
            (
                "the regulated bench",
                Key::new(&a, KeyMount::OnPunching, 6.0, mass),
            ),
            (
                "three times the friction",
                Key::new(&a, KeyMount::OnPunching, 18.0, mass),
            ),
            ("no friction", Key::new(&a, KeyMount::OnPunching, 0.0, mass)),
            (
                "a back-heavy key",
                Key::new(&a, KeyMount::OnPunching, 6.0, back_heavy),
            ),
        ]
    }

    /// What a walk resolves a threshold to, grams: half the bracket its
    /// halvings leave, and what the static solve may leave out of balance
    /// where it decides whether the key moves, its residual's tolerance of
    /// the load scale, about two newtons on these keys, over each
    /// coordinate's reach, under 0.3 m, read as grams at a measuring arm of
    /// at least 0.2 m.
    fn walk_resolution() -> f64 {
        let halved = 0.5 * STEP_G / f64::from(1u32 << HALVINGS);
        let decided = 1000.0 * RESIDUAL_TOLERANCE * 2.0 * 0.3 / (G * 0.2);
        halved + decided
    }

    /// How far the extreme a curve reads may stand from the extreme of the
    /// force it samples, grams: the curve's greatest change over one step,
    /// and twice its greatest change of slope over one, `|F₀ − 2F₁ + F₂|`.
    ///
    /// Derived. An extreme inside the stroke stands within half a step of a
    /// reading, so within the curve's change over a step of it. One at a
    /// seat, where the force is the limit of the curve, is read a step
    /// inside it at most, the first step off that seat. The change over that
    /// step is `F'(ξ)Δ` and the change over the next `F'(η)Δ`, `ξ` and `η`
    /// at most two steps apart, so the two differ by at most `2Δ²` times the
    /// curve's greatest `|F''|` between them; a second difference is `F''Δ²`
    /// at some point of the three readings it takes, and the greatest the
    /// curve shows stands for it. That takes the curve to bend no more
    /// sharply between the seat and its first reading than anywhere the
    /// readings show it, as a key's balance and arms, smooth in its
    /// attitude, make it.
    fn within_a_step(curve: &[Sample]) -> f64 {
        let g: Vec<f64> = curve.iter().map(|s| s.grams).collect();
        let step = g
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f64, f64::max);
        let bend = g
            .windows(3)
            .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
            .fold(0.0_f64, f64::max);
        step + 2.0 * bend
    }

    /// The curve's steepest change per radian between two readings, grams:
    /// how far a reading moves for a step held at an angle a little off.
    fn per_radian(curve: &[Sample]) -> f64 {
        curve
            .windows(2)
            .map(|w| ((w[1].grams - w[0].grams) / (w[1].angle - w[0].angle)).abs())
            .fold(0.0_f64, f64::max)
    }

    /// C7, consistency: **every step of the band is the band by hand at the
    /// attitude it holds.** At each step the key's angle is held, and the
    /// force that drives it there is the key's balance and its friction
    /// against the motion, read at the measuring point's arm at that angle:
    /// `(±b − M(θ)) / (g a(θ))`, each mass's and the measuring point's arm
    /// `x cos θ − y sin θ` ([`band_at`]). The two work the same moments,
    /// the procedure through the mechanism's pose and Jacobian, the hand
    /// from the key's own fields: they agree to rounding.
    #[test]
    fn every_step_of_the_band_is_the_band_by_hand_at_its_attitude() {
        for (name, key) in keys() {
            let band = key.band();
            let point = key.a.protocol.measuring_point_on_key(&key.a.key);
            let masses = [(key.mass.mass, key.mass.com)];
            for (curve, down) in [(&band.down, true), (&band.up, false)] {
                for s in curve {
                    let hand = band_at(&masses, point, key.bound, s.angle);
                    let expected = if down { hand.down_g } else { hand.up_g };
                    assert!(
                        (s.grams - expected).abs() <= key.rounding(),
                        "{name}: {} g at {} mm, by hand {expected} g",
                        s.grams,
                        s.travel * 1000.0
                    );
                }
            }
        }
    }

    /// Consistency, the second code path: **the band's extremes are the
    /// walks' down and up weights.** The greatest force of the down curve is
    /// D and the least of the up curve U, by a procedure that drives the key
    /// in position where the walk loads it in weight, and that shares
    /// nothing with it but the seats and the static solve. They agree within
    /// the walk's resolution and the curve's change over the step the sweep
    /// reads its extreme from ([`within_a_step`]), with rounding: D a step
    /// below the walk's, read a step off the top seat, on a key whose
    /// balance grows toward the bottom of its stroke.
    #[test]
    fn the_bands_extremes_are_the_walks_down_and_up_weights() {
        for (name, key) in keys() {
            let band = key.band();
            let (d, u) = key.walks();
            let bound_d = walk_resolution() + within_a_step(&band.down) + key.rounding();
            let bound_u = walk_resolution() + within_a_step(&band.up) + key.rounding();
            let (bd, bu) = (band.down_weight.grams, band.up_weight.grams);
            assert!(
                (bd - d).abs() <= bound_d,
                "{name}: D {bd} g on the band, {d} g by the walk, apart by more than {bound_d} g"
            );
            assert!(
                (bu - u).abs() <= bound_u,
                "{name}: U {bu} g on the band, {u} g by the walk, apart by more than {bound_u} g"
            );
            assert!(
                bound_d < 0.01 && bound_u < 0.01,
                "{name}: a step reads under a hundredth"
            );
            // On these keys each extreme falls on the first step off its
            // seat: D off the top one, U off the bottom one.
            assert_eq!(band.down_weight, band.down[0], "{name}");
            assert_eq!(band.up_weight, band.up[0], "{name}");
        }
    }

    /// C12, consistency: **a step half as long reads the same extremes**,
    /// within the curve's change over a step of the sweep at its own step
    /// ([`within_a_step`]): halving the step moves the first reading off a
    /// seat half a step nearer it, and the extreme with it.
    #[test]
    fn halving_the_sweeps_step_moves_its_extremes_within_a_step() {
        for (name, key) in keys() {
            let (whole, _) = key.band_by(&[], STEP);
            let (half, _) = key.band_by(&[], 0.5 * STEP);
            assert!(
                half.down.len() >= 2 * whole.down.len(),
                "{name}: twice the readings"
            );
            for (w, h, curve) in [
                (whole.down_weight, half.down_weight, &whole.down),
                (whole.up_weight, half.up_weight, &whole.up),
            ] {
                let bound = within_a_step(curve) + key.rounding();
                assert!(
                    (w.grams - h.grams).abs() <= bound,
                    "{name}: {} g by whole steps, {} g by half steps, more than {bound} g apart",
                    w.grams,
                    h.grams
                );
            }
        }
    }

    /// Consistency: **a sweep that would lift the key off its balance
    /// punching is refused.** Bushings tight enough to hold 200 g at the
    /// measuring point make the up curve a pull of about 206 g, more than
    /// the key's 121 g: on the first step off the bottom seat the punching
    /// would carry nothing under that finger, and the reaction no longer
    /// reads as the force at the measuring point. The down curve, a push,
    /// would have loaded it.
    #[test]
    fn a_sweep_that_would_lift_the_key_off_its_punching_is_refused() {
        let a = regulated();
        let mass = mass_of(&a.key).unwrap();
        let key = Key::new(&a, KeyMount::OnPunching, 200.0, mass);
        let seating = key.seating(&[]);
        let mut statics = Statics::new();
        let seats = seating.seats(&mut statics).unwrap();
        match seating.sweep(&mut statics, &seats) {
            Err(SweepRefusal::LiftsOffItsPunching {
                stroke,
                travel,
                grams,
            }) => {
                assert_eq!(stroke, Stroke::Up);
                assert!(
                    grams < -1000.0 * mass.mass,
                    "a pull past the key's weight: {grams} g"
                );
                let window = key.height(&seats.top) - key.height(&seats.bottom);
                assert!(
                    window - travel < STEP,
                    "on the first step off the bottom seat"
                );
            }
            other => panic!("the sweep must be refused: {other:?}"),
        }
        // On a pivot the key is held both ways, and the same bushings draw
        // their band.
        let pivot = Key::new(&a, KeyMount::Pivot, 200.0, mass);
        assert!(pivot.band().up_weight.grams < -1000.0 * mass.mass);
    }

    /// The first non-confusion test's weight: one gram set on the key's top
    /// 150 mm in front of its pin, riding on it.
    fn one_gram(key: &Key) -> PointLoad {
        let at = key
            .on_frame
            .key
            .top_at(key.on_frame.mounted.mechanism.parts(), 0.150)
            .expect("the key has a top there");
        PointLoad::weight(
            MaterialPoint {
                body: key.on_frame.mounted.key,
                at,
            },
            1e-3,
        )
    }

    /// Level 0, consistency, the first non-confusion test, through the band:
    /// **one gram of mass on the key moves the band's middle and not its
    /// width.** At every travel the gram's moment, read at the measuring
    /// point at the attitude held there, moves both curves alike: the middle
    /// by that much, the width by nothing but rounding. Read the trade's way,
    /// at the extremes, the middle `(D + U) / 2` moves by the mean of the
    /// gram's moment at the two extremes' attitudes, about 0.61 g, and the
    /// width `D − U` by their difference, what the key's tilt between the two
    /// does to the gram's arm and the measuring point's: derived by hand, a
    /// few thousandths of a gram. The seats hold the key on rigid stops,
    /// which the gram moves by what rounding leaves of their solve alone, so
    /// each step holds the angle it holds without the gram but for that:
    /// the tolerance adds the angle each step moved, read, through the
    /// curve's steepest slope per radian, twice for its bend
    /// ([`per_radian`]), to the rounding of the readings.
    #[test]
    fn one_gram_of_mass_moves_the_bands_middle_and_not_its_width() {
        let a = regulated();
        let key = Key::new(&a, KeyMount::OnPunching, 6.0, mass_of(&a.key).unwrap());
        let weight = one_gram(&key);
        let bare = key.band();
        let (loaded, _) = key.band_by(&[weight], STEP);
        let at = weight.at.at;
        let moment =
            |theta: f64| (at.x * libm::cos(theta) - at.y * libm::sin(theta)) / key.arm(theta);
        let slope = 2.0 * per_radian(&bare.down).max(per_radian(&bare.up));
        let rounding = 2.0 * key.rounding();
        let moved = |b: &Sample, l: &Sample| slope * (l.angle - b.angle).abs();
        let mut most = 0.0_f64;
        for (b, l) in bare
            .down
            .iter()
            .zip(&loaded.down)
            .chain(bare.up.iter().zip(&loaded.up))
        {
            most = most.max(moved(b, l));
            assert!(
                (l.grams - b.grams + moment(b.angle)).abs() <= rounding + moved(b, l),
                "at {} mm the gram moved the curve by {} g, its moment {} g",
                b.travel * 1000.0,
                l.grams - b.grams,
                moment(b.angle)
            );
        }
        assert!(most < 1e-9, "the seats moved by rounding alone: {most} g");
        for ((b_down, b_up), (l_down, l_up)) in bare
            .down
            .iter()
            .zip(bare.up.iter().rev())
            .zip(loaded.down.iter().zip(loaded.up.iter().rev()))
        {
            let width = (l_down.grams - l_up.grams) - (b_down.grams - b_up.grams);
            let bound = 2.0 * rounding + moved(b_down, l_down) + moved(b_up, l_up);
            assert!(width.abs() <= bound, "the width moved by {width} g");
        }
        let (d0, u0) = (bare.down_weight, bare.up_weight);
        let (d1, u1) = (loaded.down_weight, loaded.up_weight);
        assert_eq!(
            (d0.travel.to_bits(), u0.travel.to_bits()),
            (d1.travel.to_bits(), u1.travel.to_bits()),
            "the extremes stay put"
        );
        let tolerance = 2.0 * rounding + moved(&d0, &d1) + moved(&u0, &u1);
        let (on_d, on_u) = (moment(d0.angle), moment(u0.angle));
        let moved_middle = 0.5 * (d1.grams + u1.grams) - 0.5 * (d0.grams + u0.grams);
        let moved_width = (d1.grams - u1.grams) - (d0.grams - u0.grams);
        assert!(
            (moved_middle + 0.5 * (on_d + on_u)).abs() <= tolerance,
            "the middle moved by {moved_middle} g, by hand {} g",
            -0.5 * (on_d + on_u)
        );
        let tilt = on_u - on_d;
        assert!(
            (moved_width - tilt).abs() <= tolerance,
            "the width moved by {moved_width} g, the tilt between the extremes accounts for {tilt} g"
        );
        assert!(tilt.abs() < 0.01, "and that is a few thousandths of a gram");
        assert!(
            moved_middle < -0.5,
            "the middle falls by about 0.61 g: {moved_middle}"
        );
    }

    /// Level 0, consistency, the second non-confusion test, through the
    /// band: **bushings 1.2 times as tight widen the band and leave its
    /// middle.** At every travel the curves stand `±b` over the measuring
    /// point's arm either side of the key's balance: the width grows by a
    /// fifth, the middle moves by nothing but rounding. Read at the
    /// extremes, the width `D − U` grows by `0.2 b` read at the two extremes'
    /// arms, and the middle moves by half the difference of the two, the
    /// measuring point standing a little nearer the pin's vertical at the
    /// top than at the bottom: derived by hand, under a hundredth of a gram.
    /// Friction does not seat the key, so the seats and every step's angle
    /// are the same bits.
    #[test]
    fn tighter_bushings_widen_the_band_and_leave_its_middle() {
        let a = regulated();
        let mass = mass_of(&a.key).unwrap();
        let base = Key::new(&a, KeyMount::OnPunching, 6.0, mass);
        let tighter = Key::new(&a, KeyMount::OnPunching, 6.0 * 1.2, mass);
        let (b0, b1) = (base.band(), tighter.band());
        let tolerance = 2.0 * tighter.rounding();
        for (s0, s1) in b0.down.iter().zip(&b1.down).chain(b0.up.iter().zip(&b1.up)) {
            assert_eq!(s0.angle.to_bits(), s1.angle.to_bits());
        }
        for ((d0, u0), (d1, u1)) in b0
            .down
            .iter()
            .zip(b0.up.iter().rev())
            .zip(b1.down.iter().zip(b1.up.iter().rev()))
        {
            let middle = 0.5 * (d1.grams + u1.grams) - 0.5 * (d0.grams + u0.grams);
            assert!(middle.abs() <= tolerance, "the middle moved by {middle} g");
            let widened = (d1.grams - u1.grams) - (d0.grams - u0.grams);
            let by_hand = 2.0 * (tighter.friction_g(d0.angle) - base.friction_g(d0.angle));
            assert!(
                (widened - by_hand).abs() <= 2.0 * tolerance,
                "at {} mm the band widened by {widened} g, by hand {by_hand} g",
                d0.travel * 1000.0
            );
        }
        let extra = |theta: f64| tighter.friction_g(theta) - base.friction_g(theta);
        let (top, bottom) = (b0.down_weight.angle, b0.up_weight.angle);
        let moved_width = (b1.down_weight.grams - b1.up_weight.grams)
            - (b0.down_weight.grams - b0.up_weight.grams);
        let moved_middle = 0.5 * (b1.down_weight.grams + b1.up_weight.grams)
            - 0.5 * (b0.down_weight.grams + b0.up_weight.grams);
        assert!(
            (moved_width - (extra(top) + extra(bottom))).abs() <= 2.0 * tolerance,
            "the width moved by {moved_width} g, by hand {} g",
            extra(top) + extra(bottom)
        );
        let tilt = 0.5 * (extra(top) - extra(bottom));
        assert!(
            (moved_middle - tilt).abs() <= 2.0 * tolerance,
            "the middle moved by {moved_middle} g, the extremes' arms account for {tilt} g"
        );
        assert!(tilt.abs() < 0.01, "and that is under a hundredth of a gram");
        assert!(
            (moved_width / (b0.down_weight.grams - b0.up_weight.grams) - 0.2).abs() < 0.01,
            "about a fifth wider"
        );
    }

    /// C12, consistency: **on its punching the band is the band on a pivot**,
    /// to the bit: the punching carries the key at every step, so the key
    /// free to lift off it turns as the key on a pivot there does.
    #[test]
    fn on_its_punching_the_band_is_the_band_on_a_pivot() {
        let a = regulated();
        let mass = mass_of(&a.key).unwrap();
        let free = Key::new(&a, KeyMount::OnPunching, 6.0, mass).band();
        let pivot = Key::new(&a, KeyMount::Pivot, 6.0, mass).band();
        assert_eq!(free.down, pivot.down);
        assert_eq!(free.up, pivot.up);
    }

    /// Consistency: **the key's lift is free at every step, and stays on the
    /// punching.** The full problem, the slide left free and the punching a
    /// contact like any other, solved at a step's angle from the key lifted
    /// a tenth of a millimeter off its punching, puts it back down on it to
    /// the static solve's gap tolerance, and reads the angle's reaction the
    /// sweep read there: the punching, under the key's axis, turns it by
    /// nothing. The bound is derived: the slide stands within the gap
    /// tolerance of the punching, and the punching's push, the key's weight,
    /// turns the key through that offset at most, read at the measuring
    /// point's arm, with rounding.
    #[test]
    fn the_keys_lift_is_free_at_every_step_and_stays_on_its_punching() {
        let a = regulated();
        let key = Key::new(&a, KeyMount::OnPunching, 6.0, mass_of(&a.key).unwrap());
        let band = key.band();
        let m = &key.on_frame.mounted;
        let lift = m.lift.as_ref().expect("the key is free to lift");
        for s in [band.down[0], band.down[band.down.len() / 2], band.up[0]] {
            let mut from = vec![0.0; m.mechanism.coords().len()];
            from[lift.coord.0] = lift.on_punching + 1e-4;
            from[m.angle.0] = s.angle;
            let held = [Prescribed {
                coord: m.angle,
                value: s.angle,
            }];
            let eq = Statics::new()
                .solve(&m.mechanism, &from, &[], &held, Friction::On)
                .expect("the full problem solves");
            assert!(
                (eq.q[lift.coord.0] - lift.on_punching).abs() <= GAP_TOLERANCE,
                "the key comes back down on its punching"
            );
            let per_gram = 1e-3 * G * key.arm(s.angle);
            let frictionless = eq.reactions[m.angle.0] / -per_gram;
            let sweep = if s == band.up[0] {
                s.grams + key.friction_g(s.angle)
            } else {
                s.grams - key.friction_g(s.angle)
            };
            let bound = 1000.0 * key.mass.mass * GAP_TOLERANCE / (0.2) + key.rounding();
            assert!(
                (frictionless - sweep).abs() <= bound,
                "at {} mm: the full problem's {frictionless} g, the sweep's {sweep} g",
                s.travel * 1000.0
            );
        }
    }

    /// Consistency: **a key riding on its stops has no band**: its seats are
    /// one, off its balance punching, and the sweep is refused as the walks
    /// are.
    #[test]
    fn a_key_riding_on_its_stops_has_no_band() {
        let mut a = regulated();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        let key = Key::new(&a, KeyMount::OnPunching, 6.0, mass_of(&a.key).unwrap());
        let seating = key.seating(&[]);
        let mut statics = Statics::new();
        let seats = seating.seats(&mut statics).expect("a riding key seats");
        assert!(seats.top.rides());
        assert_eq!(
            seating.sweep(&mut statics, &seats),
            Err(SweepRefusal::NoTravel)
        );
    }

    /// C10, consistency, determinism: the same sweep twice is the same bits.
    #[test]
    fn the_same_sweep_twice_is_the_same_bits() {
        let a = regulated();
        let key = Key::new(&a, KeyMount::OnPunching, 6.0, mass_of(&a.key).unwrap());
        assert_eq!(key.band(), key.band());
    }

    /// Consistency: **the sweep reads every whole step strictly inside its
    /// window, and nothing at a seat.** At a seat the stop that holds the key
    /// takes whatever the finger leaves, so the force there is an interval,
    /// not a reading, and a reading there would depend on how the solve
    /// splits the load between the stop and the held angle. So the down
    /// curve starts one step below the top seat, the up curve on the last
    /// whole step above the bottom seat, less than a step from it, and the
    /// two read the same travels at the same angles, to the bit, the whole
    /// sweep and one in half steps alike. Each travel, read off the pose the
    /// solve left, is its whole number of steps to rounding: heights under a
    /// third of a meter, 64 units in their last place.
    #[test]
    fn the_sweep_reads_every_whole_step_strictly_inside_its_window() {
        for (name, key) in keys() {
            for step in [STEP, 0.5 * STEP] {
                let (band, seats) = key.band_by(&[], step);
                let window = key.height(&seats.top) - key.height(&seats.bottom);
                assert_eq!(band.window.to_bits(), window.to_bits(), "{name}");
                let n = u32::try_from(band.down.len()).expect("a few hundred steps");
                assert!(
                    f64::from(n) * step < window && window <= f64::from(n + 1) * step,
                    "{name}: {n} readings of {step} m in a window of {window} m"
                );
                assert_eq!(band.up.len(), band.down.len(), "{name}");
                for (k, s) in (1..=n).zip(&band.down) {
                    assert!(
                        (s.travel - f64::from(k) * step).abs() <= 64.0 * f64::EPSILON * 0.3,
                        "{name}: reading {k} at {} m",
                        s.travel
                    );
                    assert!(
                        s.travel > 0.0 && s.travel < window,
                        "{name}: a reading at a seat"
                    );
                }
                for (d, u) in band.down.iter().rev().zip(&band.up) {
                    assert_eq!(d.angle.to_bits(), u.angle.to_bits(), "{name}");
                    assert_eq!(d.travel.to_bits(), u.travel.to_bits(), "{name}");
                }
            }
        }
    }

    /// Consistency: **the balance punching gives out where the finger's pull
    /// passes the key's weight.** Between its seats nothing but its balance
    /// punching carries the key, so by the key's vertical balance the
    /// punching carries its weight less the finger's pull, and the sweep is
    /// refused just where that runs out. By hand, at the attitude `θ` of an
    /// up step the finger pulls `(b + M(θ)) / (g a(θ))`, which reaches the
    /// key's weight at the bound `b(θ) = m g a(θ) − M(θ)`: bushings a part in
    /// a million under the least of those over the up steps draw the band,
    /// and a part in a million over it, a tenth of a milligram of pull, are
    /// refused on the first up step whose bound they pass. Friction does not
    /// seat the key, so the steps hold the same angles whatever the bushings.
    #[test]
    fn the_punching_gives_out_where_the_pull_passes_the_keys_weight() {
        let a = regulated();
        let mass = mass_of(&a.key).unwrap();
        let point = a.protocol.measuring_point_on_key(&a.key);
        let arm = |p: Point, theta: f64| p.x * libm::cos(theta) - p.y * libm::sin(theta);
        let lifts_at = |theta: f64| mass.mass * G * (arm(point, theta) - arm(mass.com, theta));
        let steps = Key::new(&a, KeyMount::OnPunching, 6.0, mass).band().up;
        let least = steps
            .iter()
            .map(|s| lifts_at(s.angle))
            .fold(f64::INFINITY, f64::min);
        // The bushings' bound as the weight the key friction weight protocol
        // reads at the measuring point with the key level.
        let key = |bound: f64| {
            Key::new(
                &a,
                KeyMount::OnPunching,
                bound / (1e-3 * G * a.measuring_point()),
                mass,
            )
        };
        let weight_g = 1000.0 * mass.mass;
        let under = key(least * (1.0 - 1e-6)).band();
        assert!(
            under.up_weight.grams > -weight_g && under.up_weight.grams < -0.999 * weight_g,
            "just under the key's weight: {} g of {weight_g} g",
            under.up_weight.grams
        );
        let over = key(least * (1.0 + 1e-6));
        let first = steps
            .iter()
            .find(|s| lifts_at(s.angle) < over.bound)
            .expect("a step it lifts at");
        let seating = over.seating(&[]);
        let mut statics = Statics::new();
        let seats = seating.seats(&mut statics).unwrap();
        match seating.sweep(&mut statics, &seats) {
            Err(SweepRefusal::LiftsOffItsPunching {
                stroke,
                travel,
                grams,
            }) => {
                assert_eq!(stroke, Stroke::Up);
                assert_eq!(travel.to_bits(), first.travel.to_bits(), "on that step");
                assert!(grams < -weight_g, "a pull past the key's weight: {grams} g");
            }
            other => panic!("the sweep must be refused: {other:?}"),
        }
    }
}
