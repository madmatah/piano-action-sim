//! The seated pose and the breakaway walk over a corpus of keys and frames,
//! against the brute-force oracle. Tests only.
//!
//! C8, consistency, for the seats and the walks: every case seats the key or
//! refuses with a code, never a panic or a number that is not one; every seat
//! the core finds passes the oracle, which samples the drawn outlines and
//! shares no code with the rows; and every key that seats walks from each
//! seat to the other, its down weight above its up weight by the band its
//! friction makes. Four classes:
//!
//! - **near**: the regulated bench, every rail, felt, cloth, pin and
//!   mortise moved within a few millimetres, the chamfer from none to
//!   3.5 mm;
//! - **wide**: the bench's controls anywhere in their ranges, most of which
//!   no keyboard could be built from (the audit's BWA-12);
//! - **RUST-9's**: a front punching standing 0.1 to 0.4 mm past the key's
//!   front edge, its pin pulled, so that nothing stands under the front of
//!   the key at the bottom of its stroke;
//! - **PHY-8's**: a tail with no chamfer, the key bearing on its cloth with
//!   the flat of its underside, the back rail raised and lowered through
//!   4 mm.

use crate::contact::oracle::{self, polyline, SPACING};
use crate::contact::{ContactCode, PairGeometry, PartCode, Role, Surface, SurfaceRef};
use crate::geometry::Point;
use crate::measure::seat::SeatRefusal;
use crate::measure::walk::{Stroke, CAP};
use crate::mechanism::fixtures::XorShift;
use crate::parts::assembly::KeyAssembly;
use crate::parts::fixtures::{on_frame, regulated, OnFrame};
use crate::parts::frame::KeyKind;
use crate::solve::statics::{tests::assert_virtual_work, GAP_TOLERANCE, ITERATION_CAP};
use crate::solve::{Friction, Statics};

/// What the oracle reads of a key on its frame at one pose, metres.
#[derive(Debug, Clone, Copy, Default)]
struct Clearance {
    /// The deepest any point of a contact's surfaces stands in the other
    /// side's material, a centre bearing's face aside.
    overlap: f64,
    /// How far a centre bearing's drawn face stands into the key past what
    /// the key's tilt declares: half the face's width times the tilt's sine.
    undeclared: f64,
    /// The deepest any point of the front rail pin's outline stands in the
    /// key's wood.
    pin_in_wood: f64,
    /// The deepest any point of the key's outlines stands in the front or
    /// the back rail, or any point of theirs in the key's wood: the rails
    /// carry no contact, and a key that has gone through a stop stands in
    /// the rail under it.
    in_rail: f64,
}

/// How far a surface sampled on an arc may read inside a region it only
/// touches, and how far apart two touching surfaces may read, metres: the
/// sagitta of a chord of one spacing on the smallest radius drawn, the
/// cloth's half-millimetre edges, and the gap a static solve converges to.
fn resolution() -> f64 {
    SPACING * SPACING / (8.0 * 0.0005) + 2.0 * GAP_TOLERANCE
}

fn clearance(k: &OnFrame, q: &[f64]) -> Clearance {
    let m = &k.mounted.mechanism;
    let pose = m.pose(q);
    let parts = m.parts();
    let tilt = libm::sin(q[k.mounted.angle.0]).abs();
    let body = |s: &Surface| parts[s.ranges[0].part.index()].body;
    let mut c = Clearance::default();
    for pair in m.pairs() {
        // Each side's material: the parts its ranges belong to, or, for a
        // side that is a cut's range, every part of its body.
        let region = |s: &Surface, beside: bool| {
            let own: Vec<usize> = s
                .ranges
                .iter()
                .map(|r| r.part.index())
                .filter(|&i| parts[i].role != Role::Cut)
                .collect();
            polyline::Region::of(
                m,
                &pose,
                body(s),
                &|i| own.is_empty() || own.contains(&i),
                beside,
            )
        };
        let centre = pair.geometry == PairGeometry::CentreBearing;
        let (in_a, in_b) = (region(&pair.a, centre), region(&pair.b, centre));
        let a = oracle::surface(parts, &pair.a, &pose.placement(body(&pair.a)));
        let b = oracle::surface(parts, &pair.b, &pose.placement(body(&pair.b)));
        if centre {
            // The face's centre, which bears, and its edges, which a tilted
            // key overlaps by what a felt would compress.
            let (first, last) = (b[0], b[b.len() - 1]);
            let half = 0.5 * ((last.x - first.x).abs());
            let middle = Point::new(0.5 * (first.x + last.x), 0.5 * (first.y + last.y));
            c.overlap = c.overlap.max(in_a.depth(middle));
            let deepest = b.iter().map(|&p| in_a.depth(p)).fold(0.0, f64::max);
            c.undeclared = c.undeclared.max(deepest - half * tilt);
        } else {
            let into_b = a.iter().map(|&p| in_b.depth(p)).fold(0.0, f64::max);
            let into_a = b.iter().map(|&p| in_a.depth(p)).fold(0.0, f64::max);
            c.overlap = c.overlap.max(into_b).max(into_a);
        }
    }
    // The front rail pin, every point of it, against the key's wood: every
    // part of the key less its cuts.
    if let Some(i) = parts.iter().position(|p| p.code == PartCode::FrontRailPin) {
        let wood = polyline::Region::of(m, &pose, k.mounted.key, &|_| true, false);
        let n = u16::try_from(parts[i].silhouette().len()).unwrap();
        let pin = Surface::one(SurfaceRef {
            part: crate::contact::PartId(u16::try_from(i).unwrap()),
            segs: 0..n,
        });
        let samples = oracle::surface(parts, &pin, &pose.placement(parts[i].body));
        c.pin_in_wood = samples.iter().map(|&p| wood.depth(p)).fold(0.0, f64::max);
    }
    // The key against its rails, every outline of each.
    let is_rail = |i: usize| matches!(parts[i].code, PartCode::FrontRail | PartCode::BackRail);
    let rails = polyline::Region::of(m, &pose, crate::mechanism::BodyId::GROUND, &is_rail, false);
    let wood = polyline::Region::of(m, &pose, k.mounted.key, &|_| true, false);
    for (i, part) in parts.iter().enumerate() {
        let into = if part.body == k.mounted.key && part.role != Role::Cut {
            &rails
        } else if is_rail(i) {
            &wood
        } else {
            continue;
        };
        let n = u16::try_from(part.silhouette().len()).unwrap();
        let outline = Surface::one(SurfaceRef {
            part: crate::contact::PartId(u16::try_from(i).unwrap()),
            segs: 0..n,
        });
        let samples = oracle::surface(parts, &outline, &pose.placement(part.body));
        c.in_rail = samples
            .iter()
            .map(|&p| into.depth(p))
            .fold(c.in_rail, f64::max);
    }
    c
}

/// How far apart the oracle reads the surfaces of the pair `code` at `q`;
/// for a centre bearing, the face's centre from the key.
fn apart(k: &OnFrame, q: &[f64], code: ContactCode) -> f64 {
    let m = &k.mounted.mechanism;
    let pose = m.pose(q);
    let parts = m.parts();
    let pair = m.pairs().iter().find(|p| p.code == code).expect("declared");
    let body = |s: &Surface| parts[s.ranges[0].part.index()].body;
    let a = polyline::surface(parts, &pair.a, &pose.placement(body(&pair.a)));
    let b = polyline::surface(parts, &pair.b, &pose.placement(body(&pair.b)));
    if pair.geometry == PairGeometry::CentreBearing {
        let samples = oracle::surface(parts, &pair.b, &pose.placement(body(&pair.b)));
        let (first, last) = (samples[0], samples[samples.len() - 1]);
        let middle = Point::new(0.5 * (first.x + last.x), 0.5 * (first.y + last.y));
        return a
            .iter()
            .map(|line| polyline::to_polyline(middle, line))
            .fold(f64::INFINITY, f64::min);
    }
    let mut best = f64::INFINITY;
    for la in &a {
        for lb in &b {
            best = best.min(polyline::apart(la, lb));
        }
    }
    best
}

/// Whether the oracle finds a seat sound: nothing overlaps past the
/// resolution, and the contacts said to carry it touch.
fn sound(k: &OnFrame, q: &[f64], held_by: &[ContactCode]) -> Result<(), String> {
    let c = clearance(k, q);
    let r = resolution();
    if c.overlap > r || c.undeclared > r || c.pin_in_wood > r || c.in_rail > r {
        return Err(format!("overlaps: {c:?}"));
    }
    for &code in held_by {
        let d = apart(k, q, code);
        if d > r {
            return Err(format!("{code:?} said to carry it stands {d} m off"));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Near,
    Wide,
    Rust9,
    Phy8,
}

fn mm(x: f64) -> f64 {
    x * 1e-3
}

/// The corpus, seeded.
fn corpus() -> Vec<(Class, KeyAssembly)> {
    let mut rng = XorShift::new(0x00c8_c8c8);
    let mut out = Vec::new();
    for _ in 0..160 {
        let mut a = regulated();
        let f = &mut a.frame;
        f.balance_rail_height += mm(rng.uniform(-1.5, 1.5));
        f.front_rail_height += mm(rng.uniform(-1.5, 1.5));
        f.back_rail_height += mm(rng.uniform(-2.0, 2.0));
        f.back_rail_cloth = mm(rng.uniform(1.0, 5.0));
        f.balance_punching.felt = mm(rng.uniform(1.0, 3.0));
        f.front_punching.felt = mm(rng.uniform(1.0, 3.0));
        f.front_rail += mm(rng.uniform(-1.4, 1.4));
        f.back_rail = mm(rng.uniform(-235.0, -150.0));
        f.sections.front.pin_height = mm(rng.uniform(22.0, 26.0));
        a.key.front_mortise_depth = mm(rng.uniform(20.0, 24.0));
        a.key.tail_rise = mm(rng.uniform(0.0, 3.5));
        a.key.tail_rise_run = mm(rng.uniform(50.0, 90.0));
        out.push((Class::Near, a));
    }
    for _ in 0..80 {
        let mut a = regulated();
        let f = &mut a.frame;
        f.balance_punching.felt = mm(rng.uniform(0.0, 4.0));
        f.front_punching.felt = mm(rng.uniform(0.0, 4.0));
        f.sections.front.pin_height = mm(rng.uniform(5.0, 40.0));
        a.key.front_mortise_depth = mm(rng.uniform(5.0, 40.0));
        f.back_rail_cloth = mm(rng.uniform(0.0, 6.0));
        a.key.capstan = -mm(rng.uniform(60.0, 200.0));
        a.key.front_edge = mm(rng.uniform(150.0, 340.0));
        a.key.capstan_screw.height = mm(rng.uniform(4.0, 24.0));
        f.balance_rail_height = mm(rng.uniform(10.0, 60.0));
        f.front_rail_height = mm(rng.uniform(10.0, 50.0));
        f.back_rail_height = mm(rng.uniform(10.0, 60.0));
        f.front_rail = mm(rng.uniform(80.0, 320.0));
        f.back_rail = -mm(rng.uniform(120.0, 280.0));
        a.protocol.measuring_point_from_lip = mm(rng.uniform(5.0, 30.0));
        out.push((Class::Wide, a));
    }
    for k in 0..16 {
        // The punching's centre just past the key's front edge, its pin
        // pulled so that only the punching stands under the front.
        let mut a = regulated();
        a.frame.front_rail = a.key.front_edge + mm(0.1 + 0.3 * f64::from(k) / 15.0);
        a.frame.sections.front.pin = None;
        out.push((Class::Rust9, a));
    }
    for k in 0..40 {
        let mut a = regulated();
        a.key.tail_rise = 0.0;
        a.frame.back_rail_height += mm(-2.0 + 4.0 * f64::from(k) / 39.0);
        out.push((Class::Phy8, a));
    }
    out
}

/// C8, consistency, for the seats: every seat the core finds stands where
/// the oracle confirms it, and every refusal names its reason.
///
/// A seat of a key a keyboard could be built from takes at most a quarter
/// of the static solve's cap of steps, which `CLAUDE.md` rule 8 asks of a
/// cap.
#[test]
fn the_seats_stand_where_the_oracle_confirms_them() {
    let mut statics = Statics::new();
    let mut tally = std::collections::BTreeMap::new();
    let mut most_iterations = std::collections::BTreeMap::new();
    for (class, a) in corpus() {
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let core = k.seating().seats(&mut statics);
        let outcome = match &core {
            Ok(_) => "seated".to_owned(),
            Err(r) => format!("{r:?}")
                .split([' ', '{'])
                .next()
                .unwrap()
                .to_owned(),
        };
        *tally.entry((format!("{class:?}"), outcome)).or_insert(0) += 1;
        match core {
            Ok(seats) => {
                let most = most_iterations.entry(format!("{class:?}")).or_insert(0);
                *most = (*most)
                    .max(seats.top.report.iterations)
                    .max(seats.bottom.report.iterations);
                for (seat, which) in [(&seats.top, "top"), (&seats.bottom, "bottom")] {
                    assert!(seat.q.iter().all(|x| x.is_finite()));
                    if let Err(e) = sound(&k, &seat.q, &seat.held_by) {
                        panic!("{class:?}, the core's {which} seat: {e}\n{a:?}");
                    }
                }
                if class == Class::Near {
                    let key = k.mounted.key;
                    let load = |at: Point, f: f64| crate::mechanism::PointLoad {
                        at: crate::mechanism::MaterialPoint { body: key, at },
                        force: Point::new(0.0, -f),
                    };
                    let loads = [load(k.capstan, seats.capstan_load)];
                    let eq = statics
                        .solve(
                            &k.mounted.mechanism,
                            &seats.top.q,
                            &loads,
                            &[],
                            Friction::Off,
                        )
                        .expect("a seat solves again");
                    assert_virtual_work(
                        &k.mounted.mechanism,
                        &eq,
                        &loads,
                        Friction::Off,
                        &eq.q.clone(),
                        &[0],
                        1e-6,
                        0xc6c8,
                    );
                }
            }
            Err(SeatRefusal::NotConverged { .. } | SeatRefusal::Mechanism(_)) => {
                panic!("{class:?}: {core:?}\n{a:?}")
            }
            Err(_) => {}
        }
    }
    let count = |class: &str, outcome: &str| {
        tally
            .get(&(class.to_owned(), outcome.to_owned()))
            .copied()
            .unwrap_or(0)
    };
    assert_eq!(count("Near", "seated"), 160, "{tally:?}");
    assert_eq!(count("Rust9", "NoStopBelowTheFront"), 16, "{tally:?}");
    assert_eq!(count("Phy8", "seated"), 40, "{tally:?}");
    for class in ["Near", "Phy8"] {
        let most = most_iterations.get(class).copied().unwrap_or(0);
        assert!(
            most <= ITERATION_CAP / 4,
            "a {class} seat took {most} steps of the {ITERATION_CAP}"
        );
    }
}

/// C8, consistency, for the walks: on every key a keyboard could be built
/// from, a 6 g band of friction at its bushings, each walk reaches the other
/// seat, within a quarter of its cap of steps, and the down weight stands
/// above the up weight by at least twice the friction read through the
/// longest arm the measuring point has at either seat:
/// `D − U = b (1/a_top + 1/a_bottom) / g`, plus what the key's own balance
/// moves between the seats, which on these keys, whose centre of mass stands
/// above the pin, only adds to it.
///
/// Every increment stays within half the static solve's cap, not the
/// quarter a seat keeps to, and the bound is derived. A trial the halvings
/// put a hair past the threshold starts the key with a step of its excess
/// force over the metric, as short as the residual's tolerance lets the
/// excess be, about 3e-13 N·m, a femtometre at the key's front; each step
/// accepted is three times the last, so reaching the row margin's
/// millimetre takes about 26 steps; the stroke, 10 mm at the key front at
/// most a margin a step, about ten more. The corpus reaches 35.
#[test]
fn every_walk_reaches_the_other_seat_and_reads_d_above_u() {
    let mut statics = Statics::new();
    let mut walked = 0;
    for (class, a) in corpus() {
        if !matches!(class, Class::Near | Class::Phy8) {
            continue;
        }
        let arm = a.protocol.measuring_point(&a.key);
        let bound = crate::mechanism::AxisFriction::of_weight_at(6.0, arm).bound_n_m;
        let k = on_frame(&a, KeyKind::Natural, bound, true);
        let seating = k.seating();
        let seats = seating.seats(&mut statics).expect("a near key seats");
        let walk = |statics: &mut Statics, stroke| {
            seating
                .walk(statics, &seats, stroke)
                .unwrap_or_else(|r| panic!("{class:?}, {stroke:?}: {r:?}\n{a:?}"))
        };
        let (down, up) = (
            walk(&mut statics, Stroke::Down),
            walk(&mut statics, Stroke::Up),
        );
        for w in [&down, &up] {
            assert!(w.grams.is_finite());
            assert!(w.steps <= CAP / 4, "{class:?}: {} steps", w.steps);
            assert!(
                w.report.most_iterations <= ITERATION_CAP / 2,
                "{class:?}: an increment took {} steps",
                w.report.most_iterations
            );
        }
        // Twice the friction through the longest arm the point can have.
        let reach = (k.measuring_point.x * k.measuring_point.x
            + k.measuring_point.y * k.measuring_point.y)
            .sqrt();
        let least = 2.0 * 1000.0 * bound / (crate::G * reach);
        assert!(
            down.grams - up.grams >= least,
            "{class:?}: D {} g, U {} g, less than the {least} g its friction makes",
            down.grams,
            up.grams
        );
        walked += 1;
    }
    assert_eq!(walked, 200);
}
