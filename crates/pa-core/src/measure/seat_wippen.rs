//! The seated pose of a key carrying its wippen, on the test wippen of
//! [`crate::test_fixture`]: where each solve starts, where the wippen rests,
//! what carries it as its center and the capstan move, and what is refused.
//! Tests only.
//!
//! Each sweep asserts the runs the test bench gives, in their order and at
//! their seats: a `NotConverged`, or the heel or its cloth named in a key
//! part, within them is a defect, never a refusal the bench gives.

use crate::contact::{ContactCode, PartCode, Rows, ROW_MARGIN};
use crate::geometry::{Point, Seg, DRAWN_HALVINGS};
use crate::measure::seat::{RefusedBy, SeatRefusal, Seating, Seats, WhichSeat, WippenRefusal};
use crate::mechanism::{MaterialPoint, PointLoad};
use crate::parts::fixtures::{
    carrying_the, on_frame, on_pivot_carrying, regulated, test_wippen, OnFrame, Setup,
};
use crate::parts::frame::KeyKind;
use crate::parts::mounts::{
    heel_in_the_key, nominal_pose, step_down_a_whole_degree, wippen_lowered, Mounted, WippenLimit,
    LOWERING_HALVINGS, NOMINAL_STEP, SOLVE_PIECES, WIPPEN_ANGLE_LIMIT,
};
use crate::parts::verdicts::{wippen_carried_by, TOUCHING};
use crate::parts::{FrameDesign, KeyDesign, WippenAssembly};
use crate::solve::statics::{tests::assert_virtual_work, GAP_TOLERANCE, ITERATION_CAP};
use crate::solve::{Equilibrium, Friction, Refusal, Statics};
use crate::G;

/// What carries the wippen at one seat, as the bench names it.
#[derive(Debug, Clone, PartialEq)]
enum Rests {
    /// The capstan on the heel cloth.
    OnTheCloth,
    /// The capstan on the heel's wood, off its cloth.
    OnTheWood,
    /// The heel on the key's top, the capstan out from under it.
    OnTheKeysTop,
    /// Anything else: no link, both, or a link listed as a stop.
    Other(String),
}

/// A seat, or its refusal, by the names the bench gives them.
#[derive(Debug, Clone, PartialEq)]
enum Outcome {
    /// Seated at the top and at the bottom, the wippen resting so at each.
    Seated { top: Rests, bottom: Rests },
    /// A part of the wippen standing in a part of the key.
    InTheKey { part: PartCode, key_part: PartCode },
    /// The wippen out of the key's reach, at that seat.
    OutOfReach(WhichSeat),
    /// The wippen pushed past its range, at that seat.
    PushedPast(WhichSeat),
    /// Any other refusal.
    Refused(String),
}

use Outcome::{InTheKey, OutOfReach, PushedPast, Seated};
use Rests::{OnTheCloth, OnTheKeysTop, OnTheWood};
use WhichSeat::{Bottom, Top};

/// Both seats resting so.
fn both(rests: Rests) -> Outcome {
    match rests {
        OnTheCloth => Seated {
            top: OnTheCloth,
            bottom: OnTheCloth,
        },
        OnTheWood => Seated {
            top: OnTheWood,
            bottom: OnTheWood,
        },
        OnTheKeysTop => Seated {
            top: OnTheKeysTop,
            bottom: OnTheKeysTop,
        },
        other => Seated {
            top: other.clone(),
            bottom: other,
        },
    }
}

/// What carries the wippen at `q`: one link with one part of the heel.
fn rests(m: &Mounted, q: &[f64]) -> Rests {
    match &wippen_carried_by(m, q)[..] {
        [one] => match (one.link, &one.parts[..]) {
            (ContactCode::CapstanOnTheHeel, [PartCode::WippenHeelCloth]) => OnTheCloth,
            (ContactCode::CapstanOnTheHeel, [PartCode::WippenHeel]) => OnTheWood,
            (ContactCode::HeelOnTheKey, _) => OnTheKeysTop,
            _ => Rests::Other(format!("{one:?}")),
        },
        carried => Rests::Other(format!("{carried:?}")),
    }
}

/// At one seat, what carries the wippen, or the stops it lists where it
/// lists a link among them.
fn rests_at(m: &Mounted, seat: &crate::measure::seat::Seat) -> Rests {
    if seat.held_by.iter().any(|code| code.is_link()) {
        return Rests::Other(format!("held by {:?}", seat.held_by));
    }
    rests(m, &seat.q)
}

/// The key of `a` carrying `wippen`, seated at the top alone first, so that
/// a refusal there is the top seat's, then at both seats; with the most
/// steps either seat took, the key alone's counted with its own.
fn outcome(a: &Setup, wippen: &WippenAssembly) -> (Outcome, u32) {
    let k = carrying_the(a, wippen);
    let seated = k
        .seating()
        .top(&mut Statics::new())
        .and_then(|_| k.seating().seats(&mut Statics::new()));
    match seated {
        Ok(seats) => (
            Seated {
                top: rests_at(&k.mounted, &seats.top),
                bottom: rests_at(&k.mounted, &seats.bottom),
            },
            seats
                .top
                .report
                .iterations
                .max(seats.bottom.report.iterations),
        ),
        Err(SeatRefusal::WippenInTheKey { part, key_part, .. }) => (InTheKey { part, key_part }, 0),
        Err(SeatRefusal::WippenOutOfReach { at, .. }) => (OutOfReach(at), 0),
        Err(SeatRefusal::WippenPushedPastItsRange { at, .. }) => (PushedPast(at), 0),
        Err(refusal) => (Outcome::Refused(format!("{refusal:?}")), 0),
    }
}

/// One millimeter of a sweep and what it gives.
type Run = (u32, Outcome);

/// Each millimeter from `first` to `last`, `set` to it: what it gives, the
/// runs `expected` as the millimeters they cover, each run from its first
/// millimeter to its last, and the most steps a seat took.
fn sweep(
    first: u32,
    last: u32,
    set: impl Fn(f64) -> (Setup, WippenAssembly),
    expected: &[(u32, u32, Outcome)],
) -> (Vec<Run>, Vec<Run>, u32) {
    let mut steps = 0;
    let gave = (first..=last)
        .map(|mm| {
            let (a, wippen) = set(f64::from(mm));
            let (outcome, taken) = outcome(&a, &wippen);
            steps = steps.max(taken);
            (mm, outcome)
        })
        .collect();
    let expected = expected
        .iter()
        .flat_map(|(from, to, outcome)| (*from..=*to).map(move |mm| (mm, outcome.clone())))
        .collect();
    (gave, expected, steps)
}

/// The test wippen with its center `behind_mm` behind the balance pin.
fn centered(behind_mm: f64) -> WippenAssembly {
    let (mut wippen, _) = test_wippen();
    wippen.center.x = -behind_mm * 1e-3;
    wippen
}

/// The key's own coordinates in a pose of the key carrying its wippen: the
/// key's come first, the wippen's angle after them.
fn the_keys(alone: &Mounted, q: &[f64]) -> Vec<f64> {
    q[..alone.mechanism.coords().len()].to_vec()
}

/// How far a seat of the key alone and the same seat with its wippen may
/// stand apart, radians of the key: each closes its stops within the
/// static solve's gap tolerance and stops moving within its step tolerance,
/// at the stops' arms from the balance pin, at least 0.2 m on the test
/// bench, both seats' errors added.
fn key_angle_within() -> f64 {
    2.0 * (GAP_TOLERANCE + crate::solve::statics::STEP_TOLERANCE) / 0.2
}

/// The test bench with its balance felt `stack` thicker, its papers taken
/// up: the key turns far nose-up to its top seat, its tail down on its
/// cloth, some 0.13 rad with 20 mm.
fn on_a_stack(stack: f64) -> Setup {
    let mut a = regulated();
    a.frame.balance_punching.papers.clear();
    a.frame.front_punching.papers.clear();
    a.frame.balance_punching.felt += stack;
    a
}

/// Consistency: **a key carrying its wippen held on its punching turns
/// through a long turn in pieces.** On 20 mm of balance paper the key turns
/// some 0.13 rad from level to its top seat, its link to the heel within
/// the row margin through the whole turn, so every step moves the key and
/// the wippen together by a margin of their farthest points at most:
/// solved from the level pose, the wippen lowered onto it, under the load
/// the seat puts at the capstan, the test bench's, which the stack does not
/// change, it takes more than one cap of steps and comes to rest within
/// [`SOLVE_PIECES`] of them, at its top seat.
#[test]
fn a_key_carrying_its_wippen_held_on_its_punching_turns_through_a_long_turn_in_pieces() {
    let a = on_a_stack(0.020);
    let (wippen, _) = test_wippen();
    let k = carrying_the(&a, &wippen);
    let top = k
        .seating()
        .top(&mut Statics::new())
        .expect("the key seats on its stack");
    let capstan_load = carrying_the(&regulated(), &wippen)
        .seating()
        .seats(&mut Statics::new())
        .expect("the test bench seats")
        .capstan_load;
    let from = wippen_lowered(&k.mounted, &k.mounted.level())
        .expect("the wippen fits level")
        .q;
    let at_capstan = PointLoad {
        at: MaterialPoint {
            body: k.mounted.key,
            at: k.capstan,
        },
        force: Point::new(0.0, -capstan_load),
    };
    let eq = k
        .mounted
        .solve(&mut Statics::new(), &from, &[at_capstan], Friction::Off)
        .expect("the key turns to its cloth in pieces");
    assert!(
        eq.report.iterations > ITERATION_CAP
            && eq.report.iterations <= SOLVE_PIECES * ITERATION_CAP,
        "{} steps",
        eq.report.iterations
    );
    let angle = k.mounted.angle.0;
    assert!(
        (eq.q[angle] - top.q[angle]).abs() <= key_angle_within(),
        "it rests at {} rad, its top seat at {} rad",
        eq.q[angle],
        top.q[angle]
    );
}

/// Consistency: **the key's nominal pose is the same to the bit with its
/// wippen and without it**: it reads the key and its frame only, and
/// leaves the wippen at its frame's angle for the start of the solve to
/// place.
#[test]
fn the_keys_nominal_pose_is_the_same_bits_with_its_wippen_and_without() {
    let (wippen, _) = test_wippen();
    for (name, a) in [
        ("the test bench", regulated()),
        ("on 20 mm of balance paper", on_a_stack(0.020)),
    ] {
        let k = carrying_the(&a, &wippen);
        let alone = k.mounted.key_alone.as_deref().expect("the key alone");
        let with = nominal_pose(&k.mounted);
        let without = nominal_pose(alone);
        let angle = k.mounted.wippen.expect("a wippen").angle.0;
        let bits = |q: &[f64]| q.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&the_keys(alone, &with)), bits(&without), "{name}");
        assert_eq!(with[angle].to_bits(), 0.0_f64.to_bits(), "{name}");
    }
}

/// Consistency: **the key seats alone where it seats with its wippen,
/// within the solve's tolerances**, at both seats of the test bench, on the
/// same stops: on rigid stops the wippen's weight moves no seat
/// ([`key_angle_within`]).
#[test]
fn the_key_seats_alone_where_it_seats_with_its_wippen_within_the_solves_tolerances() {
    let a = regulated();
    let (wippen, _) = test_wippen();
    let k = carrying_the(&a, &wippen);
    let alone = on_frame(&a, KeyKind::Natural, 0.0, true);
    let with = k.seating().seats(&mut Statics::new()).expect("it seats");
    let without = alone
        .seating()
        .seats(&mut Statics::new())
        .expect("it seats");
    for (w, b) in [(&with.top, &without.top), (&with.bottom, &without.bottom)] {
        assert_eq!(w.held_by, b.held_by);
        let keys = the_keys(&alone.mounted, &w.q);
        for (x, y) in keys.iter().zip(&b.q) {
            assert!(
                (x - y).abs() <= key_angle_within(),
                "{keys:?} against {:?}",
                b.q
            );
        }
    }
    let (top, bottom) = with.heights(&k.mounted, k.front_top);
    let (alone_top, alone_bottom) = without.heights(&alone.mounted, alone.front_top);
    let lever = k.front_top.x;
    assert!((top - alone_top).abs() <= key_angle_within() * lever);
    assert!(
        ((top - bottom) - (alone_top - alone_bottom)).abs() <= 2.0 * key_angle_within() * lever
    );
}

/// Consistency: **each seat starts from the key alone's seat and counts its
/// steps.** The key alone is seated by its own rule, and each seat with the
/// wippen starts from the key alone's seat there: its steps count the key
/// alone's with its own, and what is left of it with its wippen, a drop of
/// under a margin, takes no more than a quarter of the cap, on the test
/// bench and at the top seat on 20 mm of balance paper, where the key
/// turns far from level. The top seat alone is the same bits as the top of
/// both.
#[test]
fn each_seat_starts_from_the_key_alones_seat_and_counts_its_steps() {
    let (wippen, _) = test_wippen();
    let a = regulated();
    let k = carrying_the(&a, &wippen);
    let alone = on_frame(&a, KeyKind::Natural, 0.0, true);
    let with = k.seating().seats(&mut Statics::new()).expect("it seats");
    let without = alone
        .seating()
        .seats(&mut Statics::new())
        .expect("it seats");
    let stacked = on_a_stack(0.020);
    let high = carrying_the(&stacked, &wippen)
        .seating()
        .top(&mut Statics::new())
        .expect("it seats at the top");
    let high_alone = on_frame(&stacked, KeyKind::Natural, 0.0, true)
        .seating()
        .top(&mut Statics::new())
        .expect("it seats at the top");
    for (name, w, b) in [
        ("the top seat", &with.top, &without.top),
        ("the bottom seat", &with.bottom, &without.bottom),
        ("the top seat on 20 mm of paper", &high, &high_alone),
    ] {
        let own = w.report.iterations - b.report.iterations;
        assert!(
            w.report.iterations > b.report.iterations && own <= ITERATION_CAP / 4,
            "{name}: {} steps, the key alone's {}",
            w.report.iterations,
            b.report.iterations
        );
    }
    let top = k.seating().top(&mut Statics::new()).expect("it seats");
    assert_eq!(top, with.top);
}

/// Consistency: **the seats of a key carrying its wippen from the key
/// alone's own seats are its seats, to the bit**: the sheet seats the key
/// alone once, zeroes the key friction weight on the frame at its top seat,
/// the one the procedure seats the key alone at by itself, and seats the
/// key carrying its wippen from those seats, which are the ones
/// [`Seating::seats`] seats it at.
#[test]
fn seating_from_the_key_alones_own_seats_gives_the_seats_to_the_bit() {
    let (wippen, _) = test_wippen();
    let a = regulated();
    let k = carrying_the(&a, &wippen);
    let alone = on_frame(&a, KeyKind::Natural, 0.0, true);
    let alones = alone.seating().seats(&mut Statics::new());
    let top = alone
        .seating()
        .top(&mut Statics::new())
        .expect("the key alone seats at the top");
    assert_eq!(alones.as_ref().map(|seats| &seats.top), Ok(&top));
    assert_eq!(
        k.seating()
            .seats_from_the_key_alones(&mut Statics::new(), &alones),
        k.seating().seats(&mut Statics::new())
    );
}

/// Consistency: **each seat's report counts the lowerings it started
/// from**: the top seat's, the wippen lowered onto the key level, where the
/// preparation sizes its loads, and onto the key alone's top seat, where the
/// seat starts; the bottom seat's, onto the key alone's bottom seat. The key
/// alone's own seats lower nothing.
#[test]
fn each_seats_report_counts_the_lowerings_it_started_from() {
    let (wippen, _) = test_wippen();
    let k = carrying_the(&regulated(), &wippen);
    let alone = Seating {
        mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
        ..k.seating()
    }
    .seats(&mut Statics::new())
    .expect("the key alone seats");
    let angle = k.mounted.wippen.expect("a wippen").angle.0;
    let halvings = |q: &[f64]| {
        wippen_lowered(&k.mounted, q)
            .expect("the wippen fits")
            .halvings
    };
    let with_its_wippen = |seat: &crate::measure::seat::Seat| {
        let mut q = seat.q.clone();
        q.insert(angle, 0.0);
        q
    };
    let seats = k.seating().seats(&mut Statics::new()).expect("it seats");
    assert_eq!(
        (
            seats.top.report.lowering_halvings,
            seats.bottom.report.lowering_halvings
        ),
        (
            halvings(&k.mounted.level()) + halvings(&with_its_wippen(&alone.top)),
            halvings(&with_its_wippen(&alone.bottom))
        )
    );
}

/// Consistency: **the wippen is lowered onto the key within the row margin
/// of where its heel or its cloth first touches it, and never through it.**
/// At the key alone's top seat it clears the key, and turned down by a
/// margin at its reach, the mount's own, it touches; it clears at every
/// degree above.
/// With its center pin set 50 to 57 mm high, where its body stands in the
/// key, it is never placed under the key: wherever it is lowered, it rests
/// on the key from above.
#[test]
fn the_wippen_is_lowered_within_the_row_margin_of_where_its_heel_first_touches_the_key() {
    for mm in [
        None,
        Some(50),
        Some(51),
        Some(52),
        Some(53),
        Some(54),
        Some(55),
        Some(56),
        Some(57),
    ] {
        let (mut wippen, _) = test_wippen();
        if let Some(mm) = mm {
            wippen.center.y = f64::from(mm) * 1e-3;
        }
        let k = carrying_the(&regulated(), &wippen);
        let alone = k.mounted.key_alone.as_deref().expect("the key alone");
        let seat = Seating {
            mounted: alone,
            ..k.seating()
        }
        .top(&mut Statics::new())
        .expect("the key alone seats");
        let wippen = k.mounted.wippen.expect("a wippen");
        let (angle, reach) = (wippen.angle.0, wippen.reach);
        let mut q = seat.q;
        q.insert(angle, 0.0);
        let lowered = wippen_lowered(&k.mounted, &q).expect("the wippen fits").q;
        let at = |turn: f64| {
            let mut at = lowered.clone();
            at[angle] = turn;
            heel_in_the_key(&k.mounted, &at)
        };
        let resting = lowered[angle];
        assert!(at(resting) <= 0.0, "{mm:?}: lowered into the key");
        assert!(
            resting > -WIPPEN_ANGLE_LIMIT && at(resting - ROW_MARGIN / reach) > 0.0,
            "{mm:?}: lowered to {resting} rad, not onto the key"
        );
        let mut above = resting;
        while above < WIPPEN_ANGLE_LIMIT {
            above = (above + crate::parts::mounts::NOMINAL_STEP).min(WIPPEN_ANGLE_LIMIT);
            assert!(
                at(above) <= 0.0,
                "{mm:?}: the heel stands in the key at {above} rad"
            );
        }
    }
}

/// Consistency: **a wippen inside its range at the top seat reads key
/// height, though the key level would push it past.** On a capstan 36 mm
/// tall, beyond the page's range and within the hard one, the wippen
/// carried on the key level would stand past its 30°; at the key's top
/// seat, its back down on its cloth and its capstan lower, it stands a few
/// degrees under it, and key height is read there.
#[test]
fn a_wippen_inside_its_range_at_the_top_seat_reads_key_height_though_the_key_level_would_push_it_past(
) {
    let mut a = regulated();
    a.key.capstan_screw.height = 0.036;
    let (wippen, _) = test_wippen();
    let k = carrying_the(&a, &wippen);
    assert_eq!(wippen_lowered(&k.mounted, &nominal_pose(&k.mounted)), None);
    let top = k
        .seating()
        .top(&mut Statics::new())
        .expect("the top seat is read");
    let angle = k.mounted.wippen.expect("a wippen").angle.0;
    assert!(
        top.q[angle] < WIPPEN_ANGLE_LIMIT && top.q[angle] > WIPPEN_ANGLE_LIMIT - 0.1,
        "the wippen stands at {} rad",
        top.q[angle]
    );
    assert_eq!(rests(&k.mounted, &top.q), OnTheCloth);
}

/// Consistency, by hand: **the wippen rests where its heel's circle meets
/// the capstan.** At each seat of the test bench the capstan bears on the
/// cloth; the angle at which the circle the cloth's face lies on, placed by
/// the heel's position, height and radius, stands at the bearing's radius
/// from the piece of the capstan it bears on, the key where the seat put
/// it, is found by halving, and the seat's wippen stands there within what
/// the gap tolerance leaves, read through the bearing's arm about the
/// center pin, at least 50 mm.
#[test]
fn the_wippen_rests_at_the_closed_form_of_its_heels_circle_on_the_capstan() {
    let a = regulated();
    let (wippen, _) = test_wippen();
    let k = carrying_the(&a, &wippen);
    let m = &k.mounted;
    let w = m.wippen.expect("a wippen");
    let seats = k.seating().seats(&mut Statics::new()).expect("it seats");
    let circle = wippen.heel.center(wippen.jack.pin);
    let radius = wippen.heel.radius;
    for seat in [&seats.top, &seats.bottom] {
        let pose = m.mechanism.pose(&seat.q);
        let mut rows = Rows::new();
        m.mechanism.rows(&pose, ROW_MARGIN, &mut rows);
        let row = rows
            .of_pair(w.links[0])
            .map(|(_, row)| *row)
            .find(|row| row.gap <= TOUCHING)
            .expect("the capstan bears");
        let part = &m.mechanism.parts()[row.feature.a.part().index()];
        let key = pose.placement(m.key);
        // The capstan's piece in the world: a disc, or a line.
        let gap_at = |turn: f64| {
            let (s, c) = (libm::sin(turn), libm::cos(turn));
            let center = Point::new(
                wippen.center.x + c * circle.x - s * circle.y,
                wippen.center.y + s * circle.x + c * circle.y,
            );
            let distance = |p: Point| {
                ((center.x - p.x) * (center.x - p.x) + (center.y - p.y) * (center.y - p.y)).sqrt()
            };
            match row.feature.a {
                crate::contact::Piece::Arc { seg, .. } => match part.silhouette()[usize::from(seg)]
                {
                    crate::geometry::Seg::Arc { c, r, .. } => distance(key.world(c)) - radius - r,
                    crate::geometry::Seg::Line { .. } => unreachable!("an arc's segment"),
                },
                crate::contact::Piece::Vertex { seg, .. } => {
                    distance(key.world(part.silhouette()[usize::from(seg)].start())) - radius
                }
                crate::contact::Piece::Segment { seg, .. } => {
                    let line = part.silhouette()[usize::from(seg)];
                    let (p, q) = (key.world(line.start()), key.world(line.end()));
                    let (dx, dy) = (q.x - p.x, q.y - p.y);
                    let length = (dx * dx + dy * dy).sqrt();
                    ((center.x - p.x) * dy - (center.y - p.y) * dx).abs() / length - radius
                }
                crate::contact::Piece::Center { .. } => unreachable!("no center bearing"),
            }
        };
        let turn = seat.q[w.angle.0];
        let (mut low, mut high) = (turn - 0.05, turn + 0.05);
        for _ in 0..60 {
            let middle = 0.5 * (low + high);
            if gap_at(middle) > 0.0 {
                high = middle;
            } else {
                low = middle;
            }
        }
        assert_eq!(row.feature.b.part(), the_cloth(&k));
        let bound = 2.0 * GAP_TOLERANCE / 0.05;
        assert!(
            (turn - 0.5 * (low + high)).abs() <= bound,
            "the wippen rests at {turn} rad, its circle meets the capstan at {} rad",
            0.5 * (low + high)
        );
    }
}

/// Consistency: **key height and dip do not move with the wippen beyond the
/// solve's tolerances**, read at the key front, the key's angle carried by
/// the lever to it.
#[test]
fn key_height_and_dip_do_not_move_with_the_wippen_beyond_the_solves_tolerances() {
    let a = regulated();
    let (wippen, _) = test_wippen();
    let k = carrying_the(&a, &wippen);
    let alone = on_frame(&a, KeyKind::Natural, 0.0, true);
    let with = k.seating().seats(&mut Statics::new()).expect("it seats");
    let without = alone
        .seating()
        .seats(&mut Statics::new())
        .expect("it seats");
    let (top, bottom) = with.heights(&k.mounted, k.front_top);
    let (alone_top, alone_bottom) = without.heights(&alone.mounted, alone.front_top);
    let lever = k.front_top.x;
    assert!(
        (top - alone_top).abs() <= key_angle_within() * lever,
        "{top} against {alone_top}"
    );
    assert!(
        ((top - bottom) - (alone_top - alone_bottom)).abs() <= 2.0 * key_angle_within() * lever,
        "a dip of {} against {}",
        top - bottom,
        alone_top - alone_bottom
    );
}

/// Consistency, by hand: **the bearing carries the key's weight, the loads
/// and what the wippen weighs on the key, less what the key's stops
/// carry.** The wippen rests on its link with no friction at the seats: its
/// weight's moment about its center pin is the link's push times the arm
/// the link's normal has about the pin, so the push is that moment over
/// that arm, about 0.17 N, nearly straight down, at the test bench's seats.
/// The stops' pushes are the seat's own equilibrium's, solved again from
/// it. The bound is the solve's: each coordinate's balance closes within
/// its residual tolerance of the load scale, a few newtons, over its reach,
/// under 0.3 m, read through the link's arm about the center pin, at least
/// 50 mm, and each of the few terms added carries it.
#[test]
fn the_bearing_carries_the_keys_weight_the_loads_and_the_wippens_push_by_hand() {
    let a = regulated();
    let (wippen, wippen_mass) = test_wippen();
    let k = carrying_the(&a, &wippen);
    let m = &k.mounted;
    let w = m.wippen.expect("a wippen");
    let seats = k.seating().seats(&mut Statics::new()).expect("it seats");
    let key_weight = m.mechanism.bodies()[m.key.0].mass.expect("weighed").mass * G;
    let mass = wippen_mass.props(&wippen);
    let down = |at: Point, f: f64| PointLoad {
        at: MaterialPoint { body: m.key, at },
        force: Point::new(0.0, -f),
    };
    let top = [down(k.capstan, seats.capstan_load)];
    let bottom = [
        down(k.capstan, seats.capstan_load),
        down(k.measuring_point, seats.press),
    ];
    let bearing = m.lift.as_ref().expect("free to lift").pair;
    let bound = 4.0 * crate::solve::statics::RESIDUAL_TOLERANCE * 4.0 * 0.3 / 0.05;
    for (seat, loads) in [(&seats.top, &top[..]), (&seats.bottom, &bottom[..])] {
        let pose = m.mechanism.pose(&seat.q);
        let mut rows = Rows::new();
        m.mechanism.rows(&pose, ROW_MARGIN, &mut rows);
        let row = rows
            .of_pair(w.links[0])
            .map(|(_, row)| *row)
            .find(|row| row.gap <= TOUCHING)
            .expect("the capstan bears");
        let com = pose.world(MaterialPoint {
            body: w.body,
            at: mass.com,
        });
        let o = wippen.center;
        let moment = -(com.x - o.x) * mass.mass * G;
        let n = row.normal.vector();
        let arm = (row.point.x - o.x) * n.y - (row.point.y - o.y) * n.x;
        let weighs = -moment / arm * n.y;
        assert!(
            weighs > 0.15 && weighs < 0.2,
            "the wippen weighs {weighs} N on the key"
        );
        let eq = m
            .solve(&mut Statics::new(), &seat.q, loads, Friction::Off)
            .expect("a seat is an equilibrium");
        let stops: f64 = eq
            .supports
            .iter()
            .filter_map(|s| match s.by {
                crate::solve::Holder::Contact { pair, .. }
                    if pair != bearing
                        && !m.mechanism.pairs()[usize::from(pair)].code.is_link() =>
                {
                    Some(s.force * s.normal.y)
                }
                _ => None,
            })
            .sum();
        let pressed: f64 = loads.iter().map(|l| -l.force.y).sum();
        let by_hand = key_weight + pressed + weighs - stops;
        assert!(
            (seat.bearing_load - by_hand).abs() <= bound,
            "the bearing carries {} N, by hand {by_hand} N",
            seat.bearing_load
        );
    }
}

/// Consistency: **the heel rests on its cloth while the capstan stands
/// under it**: the wippen's center 187 mm behind the balance pin, the
/// middle of the run where the capstan bears on the cloth at both seats.
#[test]
fn the_heel_rests_on_its_cloth_while_the_capstan_stands_under_it() {
    assert_eq!(outcome(&regulated(), &centered(187.0)).0, both(OnTheCloth));
}

/// Consistency: **a capstan under the heel's wood carries it there, and
/// the bench names it**, with no refusal: the wippen's center 170 mm behind
/// the balance pin, the middle of the run where the capstan bears on the
/// heel's bottom behind the cloth, and 197.5 mm, the middle of the run in
/// front of it, at both seats.
#[test]
fn a_capstan_under_the_heels_wood_carries_it_there_and_the_bench_names_it() {
    for behind_mm in [170.0, 197.5] {
        assert_eq!(
            outcome(&regulated(), &centered(behind_mm)).0,
            both(OnTheWood),
            "{behind_mm} mm"
        );
    }
}

/// Consistency: **a capstan standing out from under the heel leaves it on
/// the key's top, and the bench names it**: the wippen's center 162 mm and
/// 203.5 mm behind the balance pin, the middles of the runs where the heel
/// comes down on the key behind the capstan and in front of it.
#[test]
fn a_capstan_standing_out_from_under_the_heel_leaves_it_on_the_keys_top_and_the_bench_names_it() {
    for behind_mm in [162.0, 203.5] {
        assert_eq!(
            outcome(&regulated(), &centered(behind_mm)).0,
            both(OnTheKeysTop),
            "{behind_mm} mm"
        );
    }
}

/// Consistency: **at the test wippen's settings, a capstan lowered to the
/// page's least carries the heel alone**: from its 9 mm down to 4 mm by
/// half millimeters, the capstan alone carries the heel, on its cloth, at
/// both seats. Nothing is claimed beyond those settings.
#[test]
fn at_the_test_wippens_settings_a_capstan_lowered_to_the_pages_least_carries_the_heel_alone() {
    let (wippen, _) = test_wippen();
    let base = regulated();
    let mut half_mm = (base.key.capstan_screw.height * 2000.0).round() as u32;
    assert_eq!(half_mm, 18, "the test capstan stands 9 mm");
    while half_mm >= 8 {
        let mut a = base.clone();
        a.key.capstan_screw.height = f64::from(half_mm) * 0.5e-3;
        assert_eq!(
            outcome(&a, &wippen).0,
            both(OnTheCloth),
            "{} mm",
            f64::from(half_mm) * 0.5
        );
        half_mm -= 1;
    }
}

/// Consistency: **a heel on the capstan and on the key's top at once is
/// carried by both.** With the center pin 71 mm high and 189 mm behind the
/// balance pin, and the capstan 4 mm tall, the heel at the bottom seat
/// rests on the key's top with its position at 52 mm, and on the capstan's
/// cloth at 53 mm. Halved between the two, the position at which it passes
/// from one to the other has the heel on both at once, each link closed
/// within touching: both links read that state, the bench names both, and
/// the seat is not refused.
#[test]
fn a_heel_on_the_capstan_and_on_the_keys_top_at_once_is_carried_by_both() {
    let mut a = regulated();
    a.key.capstan_screw.height = 0.004;
    let at = |position: f64| {
        let (mut wippen, _) = test_wippen();
        wippen.center = Point::new(-0.189, 0.071);
        wippen.heel.position = position;
        let k = carrying_the(&a, &wippen);
        let seats = k.seating().seats(&mut Statics::new()).expect("it seats");
        (k, seats)
    };
    let on_the_key = |k: &OnFrame, seats: &Seats| {
        wippen_carried_by(&k.mounted, &seats.bottom.q)
            .iter()
            .any(|c| c.link == ContactCode::HeelOnTheKey)
    };
    let (mut low, mut high) = (0.052, 0.053);
    {
        let (k, seats) = at(low);
        assert_eq!(rests(&k.mounted, &seats.bottom.q), OnTheKeysTop);
        let (k, seats) = at(high);
        assert_eq!(rests(&k.mounted, &seats.bottom.q), OnTheCloth);
    }
    for _ in 0..40 {
        let middle = 0.5 * (low + high);
        let (k, seats) = at(middle);
        if on_the_key(&k, &seats) {
            low = middle;
        } else {
            high = middle;
        }
    }
    let (k, seats) = at(low);
    let links: Vec<ContactCode> = wippen_carried_by(&k.mounted, &seats.bottom.q)
        .iter()
        .map(|c| c.link)
        .collect();
    assert_eq!(
        links,
        [ContactCode::CapstanOnTheHeel, ContactCode::HeelOnTheKey]
    );
}

/// Consistency: **where the capstan bears as it moves along the key**, by
/// the millimeter from 60 to 200 mm behind the balance pin, at both seats:
/// the jack, then the wippen's body, standing in a capstan under them; the
/// heel on the key's top in front of the capstan; the capstan under the
/// heel's wood in front of the cloth, then under its cloth, at the bottom
/// seat passing onto the wood behind it a little sooner; under the wood
/// behind the cloth; and the heel on the key's top once more, the capstan
/// out from under it.
#[test]
fn where_the_capstan_bears_as_it_moves_along_the_key() {
    let (wippen, _) = test_wippen();
    let (gave, expected, steps) = sweep(
        60,
        200,
        |mm| {
            let mut a = regulated();
            a.key.capstan = -mm * 1e-3;
            (a, wippen.clone())
        },
        &[
            (
                60,
                88,
                InTheKey {
                    part: PartCode::Jack,
                    key_part: PartCode::Capstan,
                },
            ),
            (
                89,
                96,
                InTheKey {
                    part: PartCode::Wippen,
                    key_part: PartCode::Capstan,
                },
            ),
            (97, 107, both(OnTheKeysTop)),
            (108, 111, both(OnTheWood)),
            (112, 127, both(OnTheCloth)),
            (
                128,
                130,
                Seated {
                    top: OnTheCloth,
                    bottom: OnTheWood,
                },
            ),
            (131, 141, both(OnTheWood)),
            (
                142,
                142,
                Seated {
                    top: OnTheKeysTop,
                    bottom: OnTheWood,
                },
            ),
            (143, 200, both(OnTheKeysTop)),
        ],
    );
    assert_eq!(gave, expected);
    assert!(steps <= ITERATION_CAP / 2, "a seat took {steps} steps");
}

/// Consistency: **the capstan raised over the page's range stays on the
/// cloth**, by half millimeters from 4 to 24 mm, at both seats.
#[test]
fn the_capstan_raised_over_the_pages_range_stays_on_the_cloth() {
    let (wippen, _) = test_wippen();
    for half_mm in 8..=48 {
        let mut a = regulated();
        a.key.capstan_screw.height = f64::from(half_mm) * 0.5e-3;
        assert_eq!(
            outcome(&a, &wippen).0,
            both(OnTheCloth),
            "{} mm",
            f64::from(half_mm) * 0.5
        );
    }
}

/// Consistency: **where the capstan bears as the wippen's center moves
/// along the key**, by the millimeter from 160 to 215 mm behind the
/// balance pin, at both seats: the heel on the key's top, the capstan out
/// from under it behind; the heel's wood behind the cloth; the cloth; the
/// heel's wood in front of it; the heel on the key's top once more; and the
/// wippen's body standing in the capstan.
#[test]
fn where_the_capstan_bears_as_the_wippens_center_moves_along_the_key() {
    let (gave, expected, steps) = sweep(
        160,
        215,
        |mm| (regulated(), centered(mm)),
        &[
            (160, 164, both(OnTheKeysTop)),
            (
                165,
                165,
                Seated {
                    top: OnTheKeysTop,
                    bottom: OnTheWood,
                },
            ),
            (166, 176, both(OnTheWood)),
            (
                177,
                179,
                Seated {
                    top: OnTheCloth,
                    bottom: OnTheWood,
                },
            ),
            (180, 195, both(OnTheCloth)),
            (196, 199, both(OnTheWood)),
            (200, 207, both(OnTheKeysTop)),
            (
                208,
                215,
                InTheKey {
                    part: PartCode::Wippen,
                    key_part: PartCode::Capstan,
                },
            ),
        ],
    );
    assert_eq!(gave, expected);
    assert!(steps <= ITERATION_CAP / 2, "a seat took {steps} steps");
}

/// Consistency: **where the wippen rests as its center pin is raised**, by
/// the millimeter from 50 to 170 mm above the keybed, the top seat first:
/// its body in the keystick, then in the back check block; seated on the
/// cloth, its wood under the capstan from 95 mm at the top seat and 100 mm
/// at the bottom, over A5's 71 to 92 mm and past it; and out of the key's
/// reach at the top seat. No run pushes it past its range, and no run
/// stands the jack in the key.
#[test]
fn where_the_wippen_rests_as_its_center_pin_is_raised() {
    let (wippen, _) = test_wippen();
    let (gave, expected, steps) = sweep(
        50,
        170,
        |mm| {
            let mut raised = wippen.clone();
            raised.center.y = mm * 1e-3;
            (regulated(), raised)
        },
        &[
            (
                50,
                52,
                InTheKey {
                    part: PartCode::Wippen,
                    key_part: PartCode::Keystick,
                },
            ),
            (
                53,
                68,
                InTheKey {
                    part: PartCode::Wippen,
                    key_part: PartCode::BackCheckBlock,
                },
            ),
            (69, 94, both(OnTheCloth)),
            (
                95,
                99,
                Seated {
                    top: OnTheWood,
                    bottom: OnTheCloth,
                },
            ),
            (100, 106, both(OnTheWood)),
            (107, 170, OutOfReach(Top)),
        ],
    );
    assert_eq!(gave, expected);
    assert!(steps <= ITERATION_CAP / 2, "a seat took {steps} steps");
}

/// A setting a slider of the page sweeps: the domain's setting it writes,
/// and its soft range, millimeters, as the page's registry gives them
/// (`crates/pa-app/src/registry.rs`, the panels' sliders), its sign turned
/// where the page reads it behind the balance pin.
struct Slider {
    name: &'static str,
    least_mm: f64,
    most_mm: f64,
    behind: bool,
}

const fn slider(name: &'static str, least_mm: f64, most_mm: f64, behind: bool) -> Slider {
    Slider {
        name,
        least_mm,
        most_mm,
        behind,
    }
}

/// The sliders of the page the seats read, besides the capstan's place and
/// height, which their own sweeps cover, each over its soft range; and the
/// « chevalet » panel's, over A5's ranges: the wippen's center 171 to
/// 191 mm behind the balance pin and 71 to 92 mm above the keybed, the
/// heel's position 50.0 to 69.5 mm and its height 16.0 to 24.0 mm.
///
/// Left out: the front mortise's groove and bushing, and the key button's
/// and the balance mortise's sliders, which stand beside the sagittal plane
/// or where no stop of either seat reads them; the measuring point, the key
/// friction weight and the weight, since the seats are friction off on
/// rigid stops; the targets and the paper stacks, which are the
/// regulation's.
const SLIDERS: [Slider; 18] = [
    slider("balance_punching.felt", 0.0, 4.0, false),
    slider("front_punching.felt", 0.0, 4.0, false),
    slider("front_rail.pin_height", 5.0, 40.0, false),
    slider("balance_rail.pin_height", 5.0, 60.0, false),
    slider("front_mortise.depth", 5.0, 40.0, false),
    slider("front_mortise.cutter_diameter", 3.5, 10.0, false),
    slider("front_mortise.travel", 0.1, 10.0, false),
    slider("back_rail.cloth", 0.0, 6.0, false),
    slider("front_edge", 150.0, 340.0, false),
    slider("balance_rail.height", 10.0, 60.0, false),
    slider("front_rail.height", 10.0, 50.0, false),
    slider("back_rail.height", 10.0, 60.0, false),
    slider("front_rail.x", 80.0, 320.0, false),
    slider("back_rail.x", 120.0, 280.0, true),
    slider("wippen.center.x", 171.0, 191.0, true),
    slider("wippen.center.y", 71.0, 92.0, false),
    slider("wippen_heel.position", 50.0, 69.5, false),
    slider("wippen_heel.height", 16.0, 24.0, false),
];

/// `a` and `wippen` with the setting `name` written to `value`, meters, by
/// the domain's own declaration of it.
fn set(a: &mut Setup, wippen: &mut WippenAssembly, name: &str, value: f64) {
    if let Some(spec) = KeyDesign::PARAMS.iter().find(|s| s.name == name) {
        spec.apply(&mut a.key, value)
            .expect("within the hard range");
    } else if let Some(spec) = FrameDesign::PARAMS.iter().find(|s| s.name == name) {
        spec.apply(&mut a.frame, value)
            .expect("within the hard range");
    } else {
        let spec = WippenAssembly::PARAMS
            .iter()
            .find(|s| s.name == name)
            .expect("a declared setting");
        spec.apply(wippen, value).expect("within the hard range");
    }
}

/// How a slider's step ends, by kind: a `NotConverged`, a part of the
/// wippen in the key other than its body, and any refusal not named here
/// each counted as such, for the test to fail on.
fn kind(a: &Setup, wippen: &WippenAssembly) -> &'static str {
    let mass = crate::test_fixture::weighed();
    let friction = crate::mechanism::AxisFriction {
        bound_n_m: 0.0,
        radius_m: None,
    };
    let wippen_mass = crate::test_fixture::wippen_assembly_mass();
    let design = crate::validate::Design {
        key: &a.key,
        mass: &mass,
        frame: &a.frame,
        protocol: &a.protocol,
        friction: &friction,
        wippen_assembly: Some((wippen, &wippen_mass)),
    };
    if crate::validate::validate(&design).is_err() {
        return "refused by the validation layer";
    }
    let k = carrying_the(a, wippen);
    match k.seating().seats(&mut Statics::new()) {
        Ok(_) => "seated",
        Err(SeatRefusal::NotConverged { .. }) => "not converged",
        Err(SeatRefusal::WippenInTheKey { part, key_part, .. }) => match (part, key_part) {
            (PartCode::Wippen, PartCode::Keystick) => "the body in the keystick",
            (PartCode::Wippen, PartCode::BackCheckBlock) => "the body in the back check block",
            _ => "another part in the key",
        },
        Err(SeatRefusal::WippenPushedPastItsRange { at: Top, .. }) => {
            "pushed past its range at the top"
        }
        Err(SeatRefusal::WippenPushedPastItsRange { at: Bottom, .. }) => {
            "pushed past its range at the bottom"
        }
        Err(SeatRefusal::WippenOutOfReach { .. }) => "out of reach",
        Err(SeatRefusal::CannotSeat { .. }) => "stops that fight",
        Err(SeatRefusal::NoStopBelowTheFront) => "no stop below the front",
        Err(SeatRefusal::LiftsOffItsFrame { .. }) => "off its frame",
        Err(_) => "another refusal",
    }
}

/// Consistency: **each slider the seats read, swept alone over its range,
/// seats the key with its wippen or refuses it by name**, in 50 even steps
/// with both ends, from the test bench: never `NotConverged`, never the
/// heel or its cloth in a key part, each refusal a named one, and as many
/// of each as the test bench gives.
#[test]
fn each_slider_the_seats_read_seats_the_key_with_its_wippen_or_refuses_it_by_name() {
    let expected: [&[(&str, u32)]; 18] = [
        &[("seated", 50)],
        &[("seated", 50)],
        &[("seated", 50)],
        &[("seated", 50)],
        &[("refused by the validation layer", 24), ("seated", 26)],
        &[("seated", 50)],
        &[("seated", 50)],
        &[("seated", 49), ("stops that fight", 1)],
        &[
            ("no stop below the front", 10),
            ("refused by the validation layer", 9),
            ("seated", 31),
        ],
        &[
            ("pushed past its range at the bottom", 8),
            ("seated", 22),
            ("stops that fight", 8),
            ("the body in the back check block", 6),
            ("the body in the keystick", 6),
        ],
        &[("seated", 45), ("the body in the back check block", 5)],
        &[
            ("pushed past its range at the top", 2),
            ("seated", 27),
            ("the body in the back check block", 10),
            ("the body in the keystick", 11),
        ],
        &[("no stop below the front", 13), ("seated", 37)],
        &[("off its frame", 9), ("seated", 41)],
        &[("seated", 50)],
        &[("seated", 50)],
        &[("seated", 50)],
        &[("seated", 50)],
    ];
    let (wippen, _) = test_wippen();
    for (slider, expected) in SLIDERS.iter().zip(expected) {
        let mut tally: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
        for step in 0..50 {
            let mm = slider.least_mm + (slider.most_mm - slider.least_mm) * f64::from(step) / 49.0;
            let value = if slider.behind { -mm } else { mm } * 1e-3;
            let (mut a, mut w) = (regulated(), wippen.clone());
            set(&mut a, &mut w, slider.name, value);
            *tally.entry(kind(&a, &w)).or_insert(0) += 1;
        }
        let tally: Vec<(&str, u32)> = tally.into_iter().collect();
        assert_eq!(&tally[..], expected, "{}", slider.name);
    }
}

/// Consistency: **the wippen's refusals, each on its kind and at its
/// seat, with how far past its limit the heel stands**: a center pin set
/// 120 mm high leaves the wippen out of the key's reach at the top seat; a
/// capstan 35 mm tall pushes it past its range at the bottom seat, and one
/// 40 mm tall at the top seat, where the start finds the heel in the capstan
/// even at the top of the wippen's range with the key at its own top seat;
/// a center pin set 55 mm high stands the wippen's body in the back check
/// block, named with both parts and its depth.
///
/// Each distance is bounded by what moved the part: a part moved by a
/// distance from where it stood clear of another stands no deeper in it
/// than that, and one moved from where it stood in it stands no further
/// clear of it. The center pin raised from where the heel rests on the
/// capstan, which the wippen at the bottom of its range would sink into,
/// leaves the heel clear by no more than the raise; the capstan raised from
/// the test key's 9 mm, at which the bottom seat fits, stands the heel no
/// deeper than its raise there, and from 35 mm, at which the top seat fits,
/// no deeper than 5 mm there: the key alone, on rigid stops, seats where it
/// seats whatever its capstan.
#[test]
fn the_wippens_refusals_are_named_at_their_seats() {
    let (wippen, _) = test_wippen();
    let seats =
        |a: &Setup, w: &WippenAssembly| carrying_the(a, w).seating().seats(&mut Statics::new());
    let mut high = wippen.clone();
    high.center.y = 0.120;
    match seats(&regulated(), &high) {
        Err(SeatRefusal::WippenOutOfReach {
            at: Top,
            heel_clearance,
        }) => assert!(
            heel_clearance > 0.0 && heel_clearance <= high.center.y - wippen.center.y,
            "{heel_clearance} m clear"
        ),
        other => panic!("{other:?}"),
    }
    let mut tall = regulated();
    let fits = tall.key.capstan_screw.height;
    tall.key.capstan_screw.height = 0.035;
    match seats(&tall, &wippen) {
        Err(SeatRefusal::WippenPushedPastItsRange {
            at: Bottom,
            heel_depth,
        }) => assert!(
            heel_depth > 0.0 && heel_depth <= 0.035 - fits,
            "{heel_depth} m deep"
        ),
        other => panic!("{other:?}"),
    }
    tall.key.capstan_screw.height = 0.040;
    let k = carrying_the(&tall, &wippen);
    let alone = Seating {
        mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
        ..k.seating()
    }
    .top(&mut Statics::new())
    .expect("the key alone seats");
    let mut at_its_seat = alone.q;
    at_its_seat.insert(k.mounted.wippen.expect("a wippen").angle.0, 0.0);
    assert_eq!(wippen_lowered(&k.mounted, &at_its_seat), None);
    match seats(&tall, &wippen) {
        Err(SeatRefusal::WippenPushedPastItsRange {
            at: Top,
            heel_depth,
        }) => assert!(
            heel_depth > 0.0 && heel_depth <= 0.040 - 0.035,
            "{heel_depth} m deep"
        ),
        other => panic!("{other:?}"),
    }
    let mut low = wippen.clone();
    low.center.y = 0.055;
    match seats(&regulated(), &low) {
        Err(SeatRefusal::WippenInTheKey {
            part: PartCode::Wippen,
            key_part: PartCode::BackCheckBlock,
            depth,
        }) => assert!(depth > 0.002 && depth < 0.003, "{depth} m deep"),
        other => panic!("{other:?}"),
    }
}

/// The tallest capstan its hard range takes, meters.
const TALL_CAPSTAN: f64 = 0.030;

/// A center pin at which the test wippen on [`TALL_CAPSTAN`] fits at the
/// key's top seat, a degree or two under the top of its range, and is
/// pushed past it at the bottom seat, meters.
pub(super) const FITS_AT_THE_TOP_SEAT_ONLY: f64 = 0.072;

/// The weight that takes the key down, grams on the measuring point:
/// more than its down weight with the wippen.
const PRESSED_G: f64 = 60.0;

/// The test bench on [`TALL_CAPSTAN`], the test wippen's center pin
/// `center` meters above the keybed.
pub(super) fn on_a_tall_capstan(center: f64) -> OnFrame {
    let mut a = regulated();
    a.key.capstan_screw.height = TALL_CAPSTAN;
    let (mut wippen, _) = test_wippen();
    wippen.center.y = center;
    carrying_the(&a, &wippen)
}

/// The key of [`on_a_tall_capstan`] at [`FITS_AT_THE_TOP_SEAT_ONLY`], and
/// how one increment from its top seat, friction on, under [`PRESSED_G`],
/// comes out.
pub(super) fn pressed_from_the_top_seat() -> (OnFrame, Result<Equilibrium, Refusal>) {
    let k = on_a_tall_capstan(FITS_AT_THE_TOP_SEAT_ONLY);
    let mut statics = Statics::new();
    let top = k.seating().top(&mut statics).expect("the top seat fits");
    let solved = k.mounted.solve(
        &mut statics,
        &top.q,
        &[k.seating().weight(PRESSED_G)],
        Friction::On,
    );
    (k, solved)
}

/// How far above the lowest center pin at which both seats on
/// [`TALL_CAPSTAN`] stand the wippen in range the lifting bench sets it,
/// meters: the wippen stands about a degree under the top of its range at
/// the bottom seat.
const ABOVE_THE_LOWEST_FIT: f64 = 0.001;

/// How finely the lifting bench's center pin is halved, meters: a
/// hundredth of a micron.
const HALVED_TO: f64 = 1e-8;

/// The pull on the measuring point that lifts the key carrying its wippen,
/// grams: up, more than the key and the wippen weigh there.
pub(super) const PULL_G: f64 = -100.0;

/// The lifting bench: the test bench on [`TALL_CAPSTAN`], its center pin
/// [`ABOVE_THE_LOWEST_FIT`] above the lowest at which both seats stand the
/// wippen in range, found by halving to a hundredth of a micron between
/// [`FITS_AT_THE_TOP_SEAT_ONLY`], where the bottom seat pushes it past its
/// range, and the test wippen's own 78.5 mm, where both seats fit. On the
/// test bench's own capstan a pull lifts the key off its balance pin before
/// it carries the wippen to the top of its range.
pub(super) fn the_lifting_bench() -> OnFrame {
    let fits = |center: f64| {
        on_a_tall_capstan(center)
            .seating()
            .seats(&mut Statics::new())
            .is_ok()
    };
    let (_, seated) = halve_between(
        FITS_AT_THE_TOP_SEAT_ONLY,
        crate::test_fixture::WIPPEN_CENTER.y,
        fits,
    );
    on_a_tall_capstan(seated + ABOVE_THE_LOWEST_FIT)
}

/// A setting between `refused`, where `fits` says no, and `seated`, where it
/// says yes, halved until the two stand no more than [`HALVED_TO`] apart,
/// each half keeping the end `fits` gives its middle: the refused end and
/// the seated end, either way round. Sound where the setting crosses one
/// boundary between them.
pub(super) fn halve_between(
    mut refused: f64,
    mut seated: f64,
    fits: impl Fn(f64) -> bool,
) -> (f64, f64) {
    while (seated - refused).abs() > HALVED_TO {
        let middle = 0.5 * (refused + seated);
        if fits(middle) {
            seated = middle;
        } else {
            refused = middle;
        }
    }
    (refused, seated)
}

/// The key of [`the_lifting_bench`] and how one increment from its top
/// seat, friction on, under [`PULL_G`], comes out.
pub(super) fn pulled_from_the_top_seat() -> (OnFrame, Result<Equilibrium, Refusal>) {
    let k = the_lifting_bench();
    let mut statics = Statics::new();
    let top = k.seating().top(&mut statics).expect("the top seat fits");
    let solved = k.mounted.solve(
        &mut statics,
        &top.q,
        &[k.seating().weight(PULL_G)],
        Friction::On,
    );
    (k, solved)
}

/// Consistency: **a weight that pushes the wippen past its range is named
/// at the bottom seat with the sheet's depth.** On a capstan 30 mm tall
/// under a center pin 72 mm high the wippen fits at the key's top seat a
/// degree or two under the top of its range, and the sheet refuses it
/// pushed past its range at the bottom seat. 60 g on the measuring point,
/// one increment from the top seat, friction on, leans it on the top of its
/// range as the key goes down, its heel clear of the key where the
/// increment started: the refusal names the bottom seat, with the depth the
/// sheet refuses it with, to the bit, more than nothing.
#[test]
fn a_weight_that_pushes_the_wippen_past_its_range_is_named_at_the_bottom_seat_with_the_sheets_depth(
) {
    let (k, solved) = pressed_from_the_top_seat();
    let Err(refusal) = solved else {
        panic!("the weight pushes the wippen past its range")
    };
    assert_eq!(
        k.mounted.wippen_limit(&refusal),
        Some(WippenLimit::PushedPastItsRange)
    );
    let Err(SeatRefusal::WippenPushedPastItsRange {
        at: Bottom,
        heel_depth,
    }) = k.seating().seats(&mut Statics::new())
    else {
        panic!("the sheet refuses the bottom seat")
    };
    assert_eq!(
        k.seating()
            .name_the_wippens_refusal(&mut Statics::new(), Err(&refusal)),
        Some(RefusedBy::Wippen(WippenRefusal::PushedPastItsRange {
            at: Bottom,
            heel_depth
        }))
    );
    assert!(heel_depth > 0.0, "{heel_depth} m deep");
}

/// How far below the lowest fit the band's sweeps go, micrometers of
/// center pin: past the band on [`TALL_CAPSTAN`], in which the drawn
/// outlines' eight chords to an arc clear the key at the top of the
/// wippen's range while the contacts, reading the arcs themselves, stand the
/// heel's cloth in it
/// ([`the_band_ends_within_the_sweep_below_the_lowest_fit`]).
const SWEPT_BELOW_THE_LOWEST_FIT_UM: u32 = 40;

/// A center pin set below the lowest fit, and what it gives.
#[derive(Debug)]
struct BelowTheLowestFit {
    /// How far below the lowest fit, meters.
    below: f64,
    /// How deep the heel's drawn outlines stand in the key's at the top of
    /// the wippen's range, the key alone at its bottom seat, meters.
    drawn: f64,
    /// How deep the links' rows stand the heel in the key there, meters
    /// ([`Mounted::measure_the_links_overlap`]).
    links: f64,
    /// How the sheet's seats refuse it, or `None` where they seat it.
    sheet: Option<SeatRefusal>,
    /// The limit one increment from the top seat, friction on, under
    /// [`PRESSED_G`], leans the wippen on, if any.
    leans: Option<WippenLimit>,
    /// How the bench names that increment's refusal.
    named: Option<RefusedBy>,
}

impl BelowTheLowestFit {
    /// Whether the sheet refuses it pushed past its range at the bottom
    /// seat, deeper than nothing and no deeper than the center pin stands
    /// below the lowest fit, and the bench names its increment's refusal
    /// so, to the bit. The lowest fit is the seated end of its halving, at
    /// or above the height where the wippen first fits, so the center pin
    /// stands no further below that height than below the lowest fit, and
    /// the depth grows by no more than the center pin falls.
    fn is_pushed_past_at_the_bottom_seat_as_the_sheet_says(&self) -> bool {
        let Some(SeatRefusal::WippenPushedPastItsRange {
            at: Bottom,
            heel_depth,
        }) = self.sheet
        else {
            return false;
        };
        let Some(RefusedBy::Wippen(WippenRefusal::PushedPastItsRange {
            at: Bottom,
            heel_depth: named,
        })) = self.named
        else {
            return false;
        };
        heel_depth > 0.0 && heel_depth <= self.below && named.to_bits() == heel_depth.to_bits()
    }
}

/// The bench `mount` builds at each center pin height, set from the lowest
/// at which both its seats fit, halved as [`the_lifting_bench`] halves it,
/// down a micron at a time for [`SWEPT_BELOW_THE_LOWEST_FIT_UM`], and what
/// each gives.
fn sweep_below_the_lowest_fit(mount: impl Fn(f64) -> OnFrame) -> Vec<BelowTheLowestFit> {
    let fits = |center: f64| mount(center).seating().seats(&mut Statics::new()).is_ok();
    let (_, lowest_fit) = halve_between(
        FITS_AT_THE_TOP_SEAT_ONLY,
        crate::test_fixture::WIPPEN_CENTER.y,
        fits,
    );
    (1..=SWEPT_BELOW_THE_LOWEST_FIT_UM)
        .map(|micrometers| {
            let below = f64::from(micrometers) * 1e-6;
            let k = mount(lowest_fit - below);
            let wippen = k.mounted.wippen.expect("a wippen");
            let alone = Seating {
                mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
                ..k.seating()
            }
            .seats(&mut Statics::new())
            .expect("the key alone seats")
            .bottom;
            let mut at_the_top_of_its_range = alone.q;
            at_the_top_of_its_range.insert(
                wippen.angle.0,
                k.mounted.mechanism.coords()[wippen.angle.0].limits.1,
            );
            let mut statics = Statics::new();
            let top = k.seating().top(&mut statics).expect("the top seat fits");
            let solved = k.mounted.solve(
                &mut statics,
                &top.q,
                &[k.seating().weight(PRESSED_G)],
                Friction::On,
            );
            BelowTheLowestFit {
                below,
                drawn: heel_in_the_key(&k.mounted, &at_the_top_of_its_range),
                links: k
                    .mounted
                    .measure_the_links_overlap(&at_the_top_of_its_range),
                sheet: k.seating().seats(&mut Statics::new()).err(),
                leans: solved
                    .as_ref()
                    .err()
                    .and_then(|refusal| k.mounted.wippen_limit(refusal)),
                named: k
                    .seating()
                    .name_the_wippens_refusal(&mut Statics::new(), solved.as_ref()),
            }
        })
        .collect()
}

/// The key of [`on_a_tall_capstan`] turning on a pivot through its balance
/// bearing instead, the test wippen's center pin `center` meters above the
/// keybed.
fn on_a_pivot_under_a_tall_capstan(center: f64) -> OnFrame {
    let mut a = regulated();
    a.key.capstan_screw.height = TALL_CAPSTAN;
    let (mut wippen, mass) = test_wippen();
    wippen.center.y = center;
    on_pivot_carrying(&a, 0.0, true, (&wippen, &mass))
}

/// Consistency: **every center pin just below the lowest fit is pushed
/// past at the bottom seat by more than nothing, on its punching.** On
/// [`TALL_CAPSTAN`], a micron at a time for 40 µm below the lowest center
/// pin at which both seats fit, through the band in which the drawn
/// outlines clear the key at the top of the wippen's range while the links
/// stand the heel's cloth in it: the sheet refuses the bottom seat, the
/// wippen pushed past its range by more than nothing and no more than the
/// center pin stands below the lowest fit; and one increment from the top
/// seat under [`PRESSED_G`], friction on, leaning the wippen on the top of
/// its range, is named so, to the bit, and never as the key lifting off
/// its frame.
#[test]
fn every_center_pin_just_below_the_lowest_fit_is_pushed_past_at_the_bottom_seat_by_more_than_nothing_on_its_punching(
) {
    let swept = sweep_below_the_lowest_fit(on_a_tall_capstan);
    assert!(
        swept.iter().any(|at| at.drawn == 0.0 && at.links > 0.0),
        "the band is swept: {swept:#?}"
    );
    assert!(
        swept
            .iter()
            .all(|at| at.leans == Some(WippenLimit::PushedPastItsRange)),
        "every increment leans the wippen on the top of its range: {swept:#?}"
    );
    let wrong: Vec<_> = swept
        .iter()
        .filter(|at| !at.is_pushed_past_at_the_bottom_seat_as_the_sheet_says())
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Consistency: **every center pin just below the lowest fit is pushed
/// past at the bottom seat by more than nothing, on a pivot.** The sweep of
/// the key on its punching, on the same key turning on a pivot through its
/// balance bearing ([`on_pivot_carrying`]), which cannot lift, from its own
/// lowest fit: the sheet and the bench name the bottom seat, by the same
/// depth to the bit, more than nothing.
#[test]
fn every_center_pin_just_below_the_lowest_fit_is_pushed_past_at_the_bottom_seat_by_more_than_nothing_on_a_pivot(
) {
    let swept = sweep_below_the_lowest_fit(on_a_pivot_under_a_tall_capstan);
    assert!(
        swept.iter().any(|at| at.drawn == 0.0 && at.links > 0.0),
        "the band is swept: {swept:#?}"
    );
    assert!(
        swept
            .iter()
            .all(|at| at.leans == Some(WippenLimit::PushedPastItsRange)),
        "every increment leans the wippen on the top of its range: {swept:#?}"
    );
    let wrong: Vec<_> = swept
        .iter()
        .filter(|at| !at.is_pushed_past_at_the_bottom_seat_as_the_sheet_says())
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// How far the drawn chords of the heel cloth's arc on `on_frame` stand
/// inside it at most, meters: the arc's radius less the distance from its
/// center to the middle of each chord it is drawn as ([`Seg::drawn`]). The
/// cloth's one arc is its outer face, convex, the rest of its outline
/// straight ([`crate::parts::wippen_assembly::WippenAssembly::heel_cloth_outline`]).
fn measure_the_heel_cloths_sag(on_frame: &OnFrame) -> f64 {
    let cloth = &on_frame.mounted.mechanism.parts()[the_cloth(on_frame).index()];
    let mut sag = 0.0_f64;
    for arc in cloth.silhouette() {
        if let Seg::Arc { c, r, .. } = arc {
            let mut drawn = arc.drawn(DRAWN_HALVINGS);
            drawn.push(arc.end());
            for chord in drawn.windows(2) {
                let middle = Point::new(
                    0.5 * (chord[0].x + chord[1].x) - c.x,
                    0.5 * (chord[0].y + chord[1].y) - c.y,
                );
                sag = sag.max(r - (middle.x * middle.x + middle.y * middle.y).sqrt());
            }
        }
    }
    sag
}

/// Consistency: **the test heel cloth's drawn chords stand 21.8 µm inside
/// its arc**, to a tenth of a micron: the figure the documents state.
#[test]
fn the_test_heel_cloths_drawn_chords_stand_21_8_micrometers_inside_its_arc() {
    let sag = measure_the_heel_cloths_sag(&on_a_tall_capstan(FITS_AT_THE_TOP_SEAT_ONLY));
    assert_eq!(format!("{:.1}", sag * 1e6), "21.8", "{sag} m");
}

/// Consistency: **the band stands the heel's cloth in the key no deeper
/// than its arc's sag.** On [`TALL_CAPSTAN`], its top flat, a micron at a
/// time below the lowest center pin at which both seats fit, wherever the
/// drawn outlines clear the key at the top of the wippen's range, the key
/// alone at its bottom seat, the links' rows stand the heel's cloth in it
/// no deeper than its drawn chords stand inside its arc: the depth the
/// bench names in the band.
#[test]
fn the_band_stands_the_heels_cloth_in_the_key_no_deeper_than_its_arcs_sag() {
    let sag = measure_the_heel_cloths_sag(&on_a_tall_capstan(FITS_AT_THE_TOP_SEAT_ONLY));
    let deepest = sweep_below_the_lowest_fit(on_a_tall_capstan)
        .iter()
        .filter(|at| at.drawn == 0.0)
        .fold(0.0_f64, |deepest, at| deepest.max(at.links));
    assert!(
        deepest > 0.0 && deepest <= sag,
        "{deepest} m, the sag {sag} m"
    );
}

/// Consistency: **the band ends within the sweep below the lowest fit.** On
/// [`TALL_CAPSTAN`], [`SWEPT_BELOW_THE_LOWEST_FIT_UM`] below the lowest
/// center pin at which both seats fit, the drawn outlines stand the heel in
/// the key at the top of the wippen's range, the key alone at its bottom
/// seat: the band's sweeps go past it.
#[test]
fn the_band_ends_within_the_sweep_below_the_lowest_fit() {
    let swept = sweep_below_the_lowest_fit(on_a_tall_capstan);
    let lowest = swept.last().expect("a sweep");
    assert!(lowest.drawn > 0.0, "{lowest:#?}");
}

/// Consistency: **a wippen left out of reach away from the top seat is
/// named at the top seat with the sheet's clearance.** The key of the test
/// bench at its bottom seat with its wippen, the center pin then raised to
/// 120 mm, where the wippen hangs free of the key at the bottom of its range
/// from about 112 mm: one increment from there, friction on, nothing on the
/// key, leans the wippen on the bottom of its range with the key at its
/// bottom, where its back stands highest. The refusal names the top seat,
/// where the sheet refuses it, with the clearance the sheet refuses it
/// with, to the bit: more than nothing, and no more than the 41.5 mm the
/// center pin was raised, every point of the wippen moving that much.
#[test]
fn a_wippen_left_out_of_reach_away_from_the_top_seat_is_named_at_the_top_seat_with_the_sheets_clearance(
) {
    const RAISED: f64 = 0.120;
    let (wippen, _) = test_wippen();
    let bottom = carrying_the(&regulated(), &wippen)
        .seating()
        .seats(&mut Statics::new())
        .expect("the test bench seats")
        .bottom;
    let mut high = wippen.clone();
    high.center.y = RAISED;
    let k = carrying_the(&regulated(), &high);
    let mut statics = Statics::new();
    let refusal = k
        .mounted
        .solve(&mut statics, &bottom.q, &[], Friction::On)
        .expect_err("the wippen hangs out of reach");
    assert_eq!(
        k.mounted.wippen_limit(&refusal),
        Some(WippenLimit::OutOfReach)
    );
    let Err(SeatRefusal::WippenOutOfReach {
        at: Top,
        heel_clearance,
    }) = k.seating().seats(&mut Statics::new())
    else {
        panic!("the sheet refuses the top seat")
    };
    assert_eq!(
        k.seating()
            .name_the_wippens_refusal(&mut statics, Err(&refusal)),
        Some(RefusedBy::Wippen(WippenRefusal::OutOfReach {
            at: Top,
            heel_clearance
        }))
    );
    assert!(
        heel_clearance > 0.0 && heel_clearance <= RAISED - wippen.center.y,
        "{heel_clearance} m clear"
    );
}

/// The test bench's center pin at which the top seat stands the test
/// wippen on the capstan, meters: it rests there at about −25°.
const REACHES_AT_THE_TOP_SEAT: f64 = 0.106;

/// The test bench's center pin at which the top seat refuses the test
/// wippen out of the key's reach, meters.
const OUT_OF_REACH_AT_THE_TOP_SEAT: f64 = 0.107;

/// Consistency: **the out-of-reach distance falls to nothing as the center
/// pin comes down to where the wippen reaches.** On the test bench the
/// center pin is halved to a hundredth of a micron between 106 mm, where the
/// top seat stands the wippen on the capstan, and 107 mm, where it hangs out
/// of the key's reach: raising the pin moves every point of the heel up,
/// away from every key part, so the setting crosses one boundary. Half a
/// millimeter, a tenth and a hundredth above it, the top seat is refused
/// out of reach, and its heel stands more than nothing and no further from
/// the key than the center pin stands above a height at which it reached:
/// the reading falls to nothing at the boundary, and does not jump there.
#[test]
fn the_out_of_reach_distance_falls_to_nothing_as_the_center_pin_comes_down_to_where_the_wippen_reaches(
) {
    const ABOVE_THE_BOUNDARY: [f64; 3] = [0.0005, 0.0001, 0.000_01];
    let on_the_test_bench = |center: f64| {
        let (mut wippen, _) = test_wippen();
        wippen.center.y = center;
        carrying_the(&regulated(), &wippen)
    };
    let refused_out_of_reach = |center: f64| match on_the_test_bench(center)
        .seating()
        .seats(&mut Statics::new())
    {
        Err(SeatRefusal::WippenOutOfReach {
            at: Top,
            heel_clearance,
        }) => Some(heel_clearance),
        _ => None,
    };
    assert!(
        refused_out_of_reach(OUT_OF_REACH_AT_THE_TOP_SEAT).is_some()
            && refused_out_of_reach(REACHES_AT_THE_TOP_SEAT).is_none(),
        "the halving's ends stand either side of the boundary"
    );
    let (boundary, _) = halve_between(
        OUT_OF_REACH_AT_THE_TOP_SEAT,
        REACHES_AT_THE_TOP_SEAT,
        |center| refused_out_of_reach(center).is_none(),
    );
    for above in ABOVE_THE_BOUNDARY {
        let heel_clearance = refused_out_of_reach(boundary + above);
        assert!(
            heel_clearance.is_some_and(|clear| clear > 0.0 && clear <= above + HALVED_TO),
            "{above} m above the boundary: {heel_clearance:?} m clear"
        );
    }
}

/// How finely [`nearest_over_its_whole_turn`] resolves the heel's nearest
/// approach, meters: a micron, finer than any tolerance the bench reads a
/// distance to.
const RESOLVED_TO: f64 = 1e-6;

/// How near the heel and its cloth come to the key's parts over the whole
/// turn of the wippen, the key at `q`, meters, to within [`RESOLVED_TO`]
/// above it, with the widest step between two of the angles the bench
/// decides reach at ([`step_down_a_whole_degree`]). The clearance at an angle
/// moves by no more than the wippen's reach times the turn, every sample
/// and every edge of the heel turning about the center pin within its
/// reach, so no angle between two read at `a` and `b`, reading `ca` and
/// `cb`, reads less than their mean less the reach times half the turn
/// between them: each step is halved until that bound stands within
/// [`RESOLVED_TO`] of the least read so far. Started from the whole
/// degrees, so the least read is no more than the bench's reading.
fn nearest_over_its_whole_turn(mounted: &Mounted, q: &[f64]) -> (f64, f64) {
    let m = &mounted.mechanism;
    let wippen = mounted.wippen.expect("a wippen");
    let (low, high) = m.coords()[wippen.angle.0].limits;
    let picked = |keep: &dyn Fn(&crate::contact::Part) -> bool| -> Vec<crate::contact::PartId> {
        (0..m.parts().len())
            .filter(|&i| keep(&m.parts()[i]))
            .map(|i| crate::contact::PartId(u16::try_from(i).expect("few parts")))
            .collect()
    };
    let heel = picked(&|p| {
        p.body == wippen.body && matches!(p.code, PartCode::WippenHeel | PartCode::WippenHeelCloth)
    });
    let key = m.outlines(&m.pose(q), &picked(&|p| p.body == mounted.key));
    let clearance = |angle: f64| {
        let mut at = q.to_vec();
        at[wippen.angle.0] = angle;
        m.outlines(&m.pose(&at), &heel).clearance(&key)
    };
    let mut whole = vec![(high, clearance(high))];
    let mut widest = 0.0_f64;
    let mut angle = high;
    while angle > low {
        let below = step_down_a_whole_degree(angle, low);
        widest = widest.max(angle - below);
        whole.push((below, clearance(below)));
        angle = below;
    }
    let mut least = whole
        .iter()
        .map(|&(_, read)| read)
        .fold(f64::INFINITY, f64::min);
    let mut steps: Vec<[(f64, f64); 2]> = whole.windows(2).map(|pair| [pair[0], pair[1]]).collect();
    while let Some([(above, at_above), (below, at_below)]) = steps.pop() {
        let bound = 0.5 * (at_above + at_below) - wippen.reach * 0.5 * (above - below);
        if bound >= least - RESOLVED_TO {
            continue;
        }
        let middle = 0.5 * (above + below);
        let at_middle = clearance(middle);
        least = least.min(at_middle);
        steps.push([(above, at_above), (middle, at_middle)]);
        steps.push([(middle, at_middle), (below, at_below)]);
    }
    (least, widest)
}

/// Consistency: **the reading at whole degrees stands above the wippen's
/// continuous nearest by no more than half a degree of its turn.** On the
/// test bench, its center pin raised by the millimeter from 107 mm, where
/// the top seat first hangs the wippen out of the key's reach, to 170 mm,
/// with the key alone at its top seat: the bench reads the heel's nearest
/// approach to the key at the whole degrees it decides reach at
/// ([`Mounted::measure_the_heel_past`]); between two of them the heel turns
/// by its reach times half a degree at most from the nearer, so the reading
/// stands no lower than the nearest over the whole turn
/// ([`nearest_over_its_whole_turn`]), and no higher than that by more than
/// the reach times half the widest step: the nearest is resolved from above,
/// so the excess measured is no more than the true one. A measurement of
/// the bench's resolution, not of a rule it could break: the largest
/// excess, 0.14 mm at 107 mm, is the figure `docs/FOUNDATIONS.md` > Where a
/// solve starts states.
#[test]
fn the_reading_at_whole_degrees_stands_above_the_wippens_continuous_nearest_by_no_more_than_half_a_degree_of_its_turn(
) {
    const FROM_MM: u32 = 107;
    const TO_MM: u32 = 170;
    const STATED_LARGEST_EXCESS: f64 = 0.14e-3;
    const STATED_WHERE_MM: u32 = 107;
    let k = carrying_the(&regulated(), &test_wippen().0);
    let alone = Seating {
        mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
        ..k.seating()
    }
    .top(&mut Statics::new())
    .expect("the key alone seats");
    let mut q = alone.q;
    let reach = k.mounted.wippen.expect("a wippen").reach;
    q.insert(k.mounted.wippen.expect("a wippen").angle.0, 0.0);
    let (mut largest_excess, mut largest_at_mm) = (0.0, 0);
    for mm in FROM_MM..=TO_MM {
        let (mut wippen, _) = test_wippen();
        wippen.center.y = f64::from(mm) * 1e-3;
        let mounted = carrying_the(&regulated(), &wippen).mounted;
        let reading = mounted.measure_the_heel_past(&q, WippenLimit::OutOfReach);
        let (nearest, widest) = nearest_over_its_whole_turn(&mounted, &q);
        let excess = reading - nearest;
        assert!(
            excess >= 0.0 && excess <= reach * 0.5 * widest,
            "{mm} mm: {reading} m read, {nearest} m at its nearest"
        );
        if excess > largest_excess {
            (largest_excess, largest_at_mm) = (excess, mm);
        }
    }
    assert!(
        largest_excess <= STATED_LARGEST_EXCESS && largest_at_mm == STATED_WHERE_MM,
        "the largest excess is {largest_excess} m, at {largest_at_mm} mm"
    );
}

/// Consistency: **the test wippen reaches 126 mm from its center pin**, its
/// farthest drawn point ([`crate::parts::mounts::WippenOn::reach`]), to the
/// millimeter: the figure `docs/FOUNDATIONS.md` > Where a solve starts and
/// the lowering's documentation state, a degree of its turn moving that
/// point 2.2 mm.
#[test]
fn the_test_wippen_reaches_126_mm_from_its_center_pin() {
    const STATED_REACH_MM: f64 = 126.0;
    let on_frame = carrying_the(&regulated(), &test_wippen().0);
    let reach = on_frame.mounted.wippen.expect("a wippen").reach;
    assert_eq!((reach * 1e3).round(), STATED_REACH_MM, "{reach} m of reach");
}

/// Consistency: **a pull that carries the key and its wippen to the top of
/// its range lifts the key off its frame.** On the lifting bench
/// ([`the_lifting_bench`]) the wippen stands in range at both seats, about
/// a degree under the top of its range at the bottom one, so no seat stands
/// the heel past it with the key alone there. A pull of 100 g on the
/// measuring point, one increment from the top seat, friction on, lifts
/// the key about its back rail cloth off its balance punching and carries
/// the wippen to the top of its range: the key lifting off its frame, not
/// the wippen pushed past its range.
#[test]
fn a_pull_that_carries_the_key_and_its_wippen_to_the_top_of_its_range_lifts_the_key_off_its_frame()
{
    let (k, solved) = pulled_from_the_top_seat();
    let Err(refusal) = solved else {
        panic!("the pull carries the wippen to the top of its range")
    };
    assert_eq!(
        k.mounted.wippen_limit(&refusal),
        Some(WippenLimit::PushedPastItsRange)
    );
    let alone = Seating {
        mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
        ..k.seating()
    }
    .seats(&mut Statics::new())
    .expect("the key alone seats");
    let angle = k.mounted.wippen.expect("a wippen").angle.0;
    for seat in [&alone.top, &alone.bottom] {
        let mut q = seat.q.clone();
        q.insert(angle, 0.0);
        let past = k
            .mounted
            .measure_the_heel_past(&q, WippenLimit::PushedPastItsRange);
        assert!(past <= 0.0, "the heel stands {past} m in the key");
    }
    assert_eq!(
        k.seating()
            .name_the_wippens_refusal(&mut Statics::new(), Err(&refusal)),
        Some(RefusedBy::KeyLiftingOffItsFrame)
    );
}

/// Consistency: **the wippen's lowering counts its halvings.** At the key
/// alone's top seat on the test bench the heel first touches the key more
/// than a degree above the bottom of the wippen's range, so the lowering
/// halves a whole degree until what is left of it moves the wippen's
/// farthest point by no more than the row margin: ⌈log₂(degree × reach /
/// margin)⌉ times, the mount's own reach.
#[test]
fn the_wippens_lowering_counts_its_halvings() {
    let (wippen, _) = test_wippen();
    let k = carrying_the(&regulated(), &wippen);
    let seat = Seating {
        mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
        ..k.seating()
    }
    .top(&mut Statics::new())
    .expect("the key alone seats");
    let on = k.mounted.wippen.expect("a wippen");
    let mut q = seat.q;
    q.insert(on.angle.0, 0.0);
    let lowered = wippen_lowered(&k.mounted, &q).expect("the wippen fits");
    assert!(
        lowered.q[on.angle.0] + WIPPEN_ANGLE_LIMIT > NOMINAL_STEP,
        "it touches {} rad above the bottom of its range",
        lowered.q[on.angle.0] + WIPPEN_ANGLE_LIMIT
    );
    let expected = libm::ceil(libm::log2(NOMINAL_STEP * on.reach / ROW_MARGIN));
    assert_eq!(f64::from(lowered.halvings), expected);
}

/// Consistency: **a lowering its cap ends still starts clear of the key,
/// and says so.** A reach no wippen has, 10²⁰ m, never lets what is left of
/// the degree move the wippen's farthest point by less than the row margin:
/// the halving stops at its cap, [`LOWERING_HALVINGS`], counted, and the
/// wippen stands clear of the key there, only further above where it first
/// touches.
#[test]
fn a_lowering_its_cap_ends_still_starts_clear_of_the_key_and_says_so() {
    const NO_WIPPENS_REACH: f64 = 1e20;
    let (wippen, _) = test_wippen();
    let k = carrying_the(&regulated(), &wippen);
    let alone = Seating {
        mounted: k.mounted.key_alone.as_deref().expect("the key alone"),
        ..k.seating()
    }
    .top(&mut Statics::new())
    .expect("the key alone seats");
    let mut mounted = k.mounted.clone();
    let on = mounted.wippen.as_mut().expect("a wippen");
    on.reach = NO_WIPPENS_REACH;
    let mut q = alone.q;
    q.insert(on.angle.0, 0.0);
    let lowered = wippen_lowered(&mounted, &q).expect("the wippen fits");
    assert_eq!(lowered.halvings, LOWERING_HALVINGS);
    assert_eq!(heel_in_the_key(&mounted, &lowered.q), 0.0);
}

/// C6, consistency: **each seat of a key carrying its wippen is an
/// equilibrium of its loads**, by the unilateral virtual-work inequality,
/// over the key's angle and the wippen's: no displacement that keeps every
/// gap open, the links' included, releases energy. The slide stays where
/// its punching holds it, as the bare key's test lets it move: a draw of up
/// to a micron on the slide moves the key's contacts by as much, while a
/// microradian on the angles moves the closed links' contact points by
/// their arms, about an eighth of a meter behind the balance pin for the
/// capstan and some 6 cm from the center pin for the heel, a fraction of a
/// micron, and the key's stops by less than a quarter of a micron, their
/// arms shorter than a quarter of a meter: a draw of the slide closes the
/// balance punching, drawn down, or the capstan against the heel, drawn up,
/// by more than the angles can open it again in all but a few draws, fewer
/// than the test needs.
#[test]
fn each_seat_of_a_key_carrying_its_wippen_is_an_equilibrium_of_its_loads() {
    let (wippen, _) = test_wippen();
    let k = carrying_the(&regulated(), &wippen);
    let seats = k.seating().seats(&mut Statics::new()).expect("it seats");
    let key = k.mounted.key;
    let down = |at: Point, f: f64| PointLoad {
        at: MaterialPoint { body: key, at },
        force: Point::new(0.0, -f),
    };
    let top = [down(k.capstan, seats.capstan_load)];
    let bottom = [
        down(k.capstan, seats.capstan_load),
        down(k.measuring_point, seats.press),
    ];
    for (seat, loads) in [(&seats.top, &top[..]), (&seats.bottom, &bottom[..])] {
        let eq = k
            .mounted
            .solve(&mut Statics::new(), &seat.q, loads, Friction::Off)
            .expect("a seat is an equilibrium");
        assert_virtual_work(
            &k.mounted.mechanism,
            &eq,
            loads,
            Friction::Off,
            &eq.q.clone(),
            &[1, 2],
            1e-6,
            0x3a1e,
        );
    }
}

/// The heel cloth's part of a key carrying its wippen.
fn the_cloth(k: &OnFrame) -> crate::contact::PartId {
    let parts = k.mounted.mechanism.parts();
    let i = parts
        .iter()
        .position(|p| p.code == PartCode::WippenHeelCloth)
        .expect("a heel cloth");
    crate::contact::PartId(u16::try_from(i).expect("fewer than 65 536 parts"))
}
