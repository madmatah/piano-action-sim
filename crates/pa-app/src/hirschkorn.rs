//! Hirschkorn's key on the bench, held to what the implementations that read
//! his printed geometry make of it.
//!
//! His action's record (`data/params/hirschkorn-boston-gp178-note52.json`) is
//! read three times over: by the bench, through his profile
//! (`data/profiles/hirschkorn.json`); by `tools/derivations/10`, as a rigid
//! closure; and by the dynamics prototype (`tools/dynamics-prototype/`), with
//! compliant contacts. The last two print the key between its two stops on
//! his frame, and `tests/fixtures/hirschkorn-between-its-stops.json` keeps
//! what they print, with the command that prints it. The bench differs from
//! them by one deliberate choice, and only by it: its frame bears on real
//! felts, a flat front punching washer over Pg4 and a flat back rail cloth
//! strip lying wholly under the key's chamfer, its rounded front edge over
//! Pg5, each with its top where his round of 5 mm has its, where his model
//! bears on the rounds. That difference is derived here in closed form, from
//! his printed points and the radius his profile rounds the cloth's edges
//! to, and the bench is held to it.

use pa_core::geometry::Point;
use pa_core::measure::measurement::{ContactFound, ContactPoint, ProcedureId};
use pa_core::parts::frame::KeyKind;
use serde_json::Value;

use crate::instrument::Instrument;
use crate::profile::Profile;

/// His action's record, as compiled in.
const RECORD: &str = include_str!("../../../data/params/hirschkorn-boston-gp178-note52.json");

/// What script 10 and the prototype print of his key between its stops.
const FIXTURE: &str = include_str!("../tests/fixtures/hirschkorn-between-its-stops.json");

/// His shipped profile's text.
const PROFILE: &str = include_str!("../../../data/profiles/hirschkorn.json");

/// A record's value in SI, by its pointer in his record.
fn si(record: &Value, pointer: &str) -> f64 {
    let r = record
        .pointer(pointer)
        .unwrap_or_else(|| panic!("{pointer} in his record"));
    let value = r["value"].as_f64().expect("a value");
    match r["unit"].as_str().expect("a unit") {
        "m" | "kg" | "dimensionless" => value,
        "mm" => value / 1000.0,
        other => panic!("{pointer} in {other}"),
    }
}

/// A printed point of his, meters, in its body's frame.
fn point(record: &Value, body: &str, name: &str) -> Point {
    let at = format!("/geometry/{body}/points/{name}");
    Point::new(
        si(record, &format!("{at}/x")),
        si(record, &format!("{at}/y")),
    )
}

/// A length his profile states, in SI: `param`'s stated value, in
/// millimeters there.
fn stated(profile: &Value, param: &str) -> f64 {
    let v = profile["values"]
        .as_array()
        .expect("its values")
        .iter()
        .find(|v| v["param"] == param)
        .unwrap_or_else(|| panic!("{param} bound"));
    assert_eq!(v["unit"], "mm", "{param} in millimeters");
    v["value"].as_f64().expect("a stated value") / 1000.0
}

/// A printed line's normal, as printed.
fn normal(record: &Value, name: &str) -> Point {
    let at = format!("/geometry/key/points/{name}/normal");
    Point::new(
        si(record, &format!("{at}/x")),
        si(record, &format!("{at}/y")),
    )
}

fn turned(t: f64, p: Point) -> Point {
    let (s, c) = t.sin_cos();
    Point::new(c * p.x - s * p.y, s * p.x + c * p.y)
}

/// The angle, radians nose-up, at which the key's line through `through`
/// with the normal `n` as printed, turned about its pivot, touches a round
/// of radius `r` about `center`: his model's stop, closed as script 10 and
/// the prototype close it. A bisection of a hundred halvings over a tenth
/// of a radian.
fn on_a_round(through: Point, n: Point, center: Point, r: f64, lo: f64, hi: f64) -> f64 {
    let gap = |t: f64| {
        let (p, m) = (turned(t, through), turned(t, n));
        (center.x - p.x) * m.x + (center.y - p.y) * m.y - r
    };
    let (mut lo, mut hi) = (lo, hi);
    let lo_positive = gap(lo) > 0.0;
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        if (gap(mid) > 0.0) == lo_positive {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// His key between its stops: attitudes, radians nose-up, and the dip at
/// Pk2, meters.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Stops {
    rest: f64,
    bottom: f64,
    dip_at_pk2: f64,
}

/// How far a point of the key falls between two attitudes, the key turning
/// about its pivot.
fn fall(p: Point, rest: f64, bottom: f64) -> f64 {
    turned(rest, p).y - turned(bottom, p).y
}

/// C11, a consistency check across implementations and not a validation
/// (`docs/FOUNDATIONS.md` > What a test proves; drive in position, compare forces): the instrument built on Hirschkorn's
/// profile against what script 10 and the prototype print of his key between
/// its stops, and against Table A.1. `Err` names what disagrees.
///
/// - **His rounds, closed again here** from the printed points as script 10
///   closes them, reproduce what script 10 prints, to its last digit: this
///   test reads his frame as script 10 does.
/// - **The felts, in closed form** from his printed points and the radius
///   his profile rounds the cloth's edges to, never from the bench's frame:
///   each felt's top where his round's top is, the washer over Pg4 and the
///   cloth wholly under the chamfer, its front edge, an arc of that radius,
///   centered over Pg5, as the bench's must be; at rest the key, nose-up,
///   bears with its chamfer, Pk5's line, which rises toward the tail, on
///   that edge, as a line touches a round; at the bottom it bears on the
///   front punching read at its center, as the bench reads every punching.
///   The felts' difference from his rounds, these less those, is the bound:
///   the bench's seats stand where script 10's do, moved by that
///   difference, to within what script 10 prints them to and what the
///   static solve closes a contact to (a picometer over arms of a fifth of
///   a meter); and where the prototype's do, moved by it, to within what
///   the prototype's compliant contacts compress under the key at rest, over
///   the arm of the contact compressed. The bench's washer and the cloth's
///   front edge are held to Pg4 and Pg5 where it draws them, which names a
///   felt laid elsewhere along the key before the seats show it.
/// - **Table A.1's static moment** over the measuring arm is the front
///   weight the jig reads, to round-off; and the key weight ratio is the arm
///   of where his capstan touches his heel at the top seat, the point its
///   condition names, over the measuring arm, that point standing on his
///   dome to round-off. These see what the attitudes cannot: a center of
///   mass read in another frame.
fn c11(b: &Instrument) -> Result<(Stops, Stops), String> {
    let record: Value = serde_json::from_str(RECORD).expect("his record");
    let fixture: Value = serde_json::from_str(FIXTURE).expect("the fixture");
    let printed = |who: &str, what: &str| fixture[who][what].as_f64().expect(what);

    // His rounds, as script 10 closes them.
    let (pk4, pk5) = (point(&record, "key", "Pk4"), point(&record, "key", "Pk5"));
    let (pg4, pg5) = (
        point(&record, "ground", "Pg4"),
        point(&record, "ground", "Pg5"),
    );
    let r4 = si(&record, "/geometry/ground/points/Pg4/radius");
    let r5 = si(&record, "/geometry/ground/points/Pg5/radius");
    let pk2 = point(&record, "key", "Pk2");
    let round_rest = on_a_round(pk5, normal(&record, "Pk5"), pg5, r5, 0.0, 0.1);
    let round_bottom = on_a_round(pk4, normal(&record, "Pk4"), pg4, r4, -0.1, 0.0);
    let rounds = Stops {
        rest: round_rest,
        bottom: round_bottom,
        dip_at_pk2: fall(pk2, round_rest, round_bottom),
    };
    let rad = fixture["script_10"]["printed_to"]["rad"]
        .as_f64()
        .expect("rad");
    let print_m = fixture["script_10"]["printed_to"]["mm"]
        .as_f64()
        .expect("mm")
        / 1000.0;
    for (what, ours, theirs, to) in [
        ("rest", rounds.rest, printed("script_10", "rest_rad"), rad),
        (
            "bottom",
            rounds.bottom,
            printed("script_10", "bottom_rad"),
            rad,
        ),
        (
            "dip at Pk2",
            rounds.dip_at_pk2,
            printed("script_10", "dip_at_pk2_mm") / 1000.0,
            print_m,
        ),
    ] {
        if (ours - theirs).abs() > 0.5 * to {
            return Err(format!(
                "his rounds closed here: {what} {ours}, script 10 prints {theirs}"
            ));
        }
    }

    // The felts, in closed form from his printed points, as the owner chose
    // them: a flat washer and a flat cloth strip, each with its top where
    // his round has its, the washer read at its center over Pg4; the cloth
    // lying wholly under the chamfer, its front edge, rounded to the radius
    // his profile states, centered over Pg5; the key, nose-up at rest,
    // bearing with its chamfer on that edge. The chamfer is the line the
    // bench draws, through the kink where Pk5's line meets Pk4's flat, its
    // normal made unit; his rounds keep it as printed, as script 10 does.
    // Nothing here reads the bench's frame, so a frame read in another
    // convention moves the bench's seats and not these, by however little.
    let profile: Value = serde_json::from_str(PROFILE).expect("his profile");
    let n5 = normal(&record, "Pk5");
    if pk4.y != 0.0 || normal(&record, "Pk4") != Point::new(0.0, -1.0) {
        return Err("his underside is no longer the flat y = 0 the felts are closed on".to_owned());
    }
    let kink = pk5.x + n5.y * pk5.y / n5.x;
    let rho = stated(&profile, "back_rail.cloth_edge_radius");
    let edge = Point::new(pg5.x, pg5.y + r5 - rho);
    if edge.x + rho >= kink {
        return Err(format!(
            "the cloth reaches {} m, the chamfer's kink stands at {kink} m",
            edge.x + rho
        ));
    }
    let length = (n5.x * n5.x + n5.y * n5.y).sqrt();
    let chamfer = Point::new(n5.x / length, n5.y / length);
    let felt_rest = on_a_round(Point::new(kink, 0.0), chamfer, edge, rho, 0.0, 0.1);
    let m = turned(felt_rest, chamfer);
    let bears = Point::new(edge.x - rho * m.x, edge.y - rho * m.y);
    if bears.x < edge.x || bears.x > edge.x + rho || bears.y < edge.y {
        return Err(format!(
            "the chamfer meets the cloth's front edge at {bears:?}, off the arc about {edge:?}"
        ));
    }
    let felt_bottom = ((pg4.y + r4) / pg4.x).atan();
    let key = b.key();
    let frame = b.frame();
    let front_x = frame
        .sections
        .front
        .pin_x(frame.front_rail, KeyKind::Natural);
    let back = &frame.sections.back;
    let edge_x = back.center_x(frame.back_rail) + back.half_length;
    // The cloth's front edge is drawn where its center and half-length put
    // it, one addition, held to Pg5 within that addition's rounding.
    if front_x != pg4.x || (edge_x - pg5.x).abs() > f64::EPSILON * pg5.x.abs() {
        return Err(format!(
            "the bench's washer and cloth's front edge stand at {front_x} and {edge_x} m, his \
             rounds at {} and {} m",
            pg4.x, pg5.x
        ));
    }
    let felts = Stops {
        rest: felt_rest,
        bottom: felt_bottom,
        dip_at_pk2: fall(pk2, felt_rest, felt_bottom),
    };

    // The bench's seats.
    let (k, seats) = b
        .seated()
        .map_err(|why| format!("the bench does not seat his key: {why:?}"))?;
    if seats.top.lift != 0.0 || seats.bottom.lift != 0.0 {
        return Err("the key rides off its balance punching".to_owned());
    }
    let angle = k.mounted.angle.0;
    let (top, bottom) = seats.heights(&k.mounted, pk2);
    let bench = Stops {
        rest: seats.top.q[angle],
        bottom: seats.bottom.q[angle],
        dip_at_pk2: top - bottom,
    };

    // Script 10, moved by the felts' difference, to within its print and
    // the static solve's closure, a picometer over an arm of a fifth of a
    // meter; then the prototype, moved by it, within its compression.
    let closed = 1e-12 / 0.2;
    let compressed = |gap_mm: &str, arm: f64| printed("prototype", gap_mm).abs() / 1000.0 / arm;
    let rest_bound = compressed("rest_back_gap_mm", pg5.x.abs() - r5) + 0.5 * rad;
    let bottom_bound = compressed("bottom_front_gap_mm", pg4.x - r4) + 0.5 * rad;
    type Of = fn(&Stops) -> f64;
    let checks: [(&str, &str, Of, f64, f64, f64, f64); 3] = [
        ("rest", "rest_rad", |s| s.rest, 1.0, rad, closed, rest_bound),
        (
            "bottom",
            "bottom_rad",
            |s| s.bottom,
            1.0,
            rad,
            closed,
            bottom_bound,
        ),
        (
            "dip at Pk2",
            "dip_at_pk2_mm",
            |s| s.dip_at_pk2,
            1000.0,
            print_m,
            2.0 * closed * pk2.x,
            pk2.x * (rest_bound + bottom_bound) + 0.5 * print_m,
        ),
    ];
    for (what, name, f, per_si, to, closure, prototype_bound) in checks {
        let shift = f(&felts) - f(&rounds);
        let script_10 = printed("script_10", name) / per_si;
        let prototype = printed("prototype", name) / per_si;
        let off = (f(&bench) - (script_10 + shift)).abs();
        if off > 0.5 * to + closure {
            return Err(format!(
                "{what}: the bench reads {}, script 10's {script_10} moved by the felts' {shift} \
                 is off by {off}",
                f(&bench)
            ));
        }
        let off = (f(&bench) - (prototype + shift)).abs();
        if off > prototype_bound {
            return Err(format!(
                "{what}: the bench reads {}, the prototype's {prototype} moved by the felts' \
                 {shift} is off by {off}, past its compression's {prototype_bound}",
                f(&bench)
            ));
        }
    }

    // Table A.1, through the jig.
    let mass = si(&record, "/mass_properties/bodies/key/mass");
    let center_x = si(&record, "/mass_properties/bodies/key/center_of_mass/x");
    let arm = b.measuring_point();
    let read = |p| match b.sheet().get(p) {
        Some(Ok(m)) => Ok(m.value),
        other => Err(format!("{p:?}: {other:?}")),
    };
    let front_weight = mass * 1000.0 * center_x / arm;
    let fw = read(ProcedureId::FrontWeight)?;
    if (fw - front_weight).abs() > 64.0 * f64::EPSILON * front_weight.abs() {
        return Err(format!(
            "front weight: the jig reads {fw} g, Table A.1's static moment over the measuring \
             arm is {front_weight} g"
        ));
    }
    let sheet = b.sheet();
    let Some(Ok(kr)) = sheet.get(ProcedureId::KeyWeightRatio) else {
        return Err(format!(
            "key weight ratio: {:?}",
            sheet.get(ProcedureId::KeyWeightRatio)
        ));
    };
    let Some(ContactPoint {
        at,
        found: ContactFound::WhereTheCapstanTouchesTheHeel { .. },
    }) = kr.conditions.capstan_heel_contact
    else {
        return Err(format!(
            "key weight ratio: read at {:?}, not where his capstan touches his heel",
            kr.conditions.capstan_heel_contact
        ));
    };
    let ratio = -at.x / arm;
    if (kr.value - ratio).abs() > 64.0 * f64::EPSILON {
        return Err(format!(
            "key weight ratio: the jig reads {}, the arm of where his capstan touches his heel \
             over the measuring arm is {ratio}",
            kr.value
        ));
    }
    // The point stands on his dome, a round-off from it: the row's point
    // carried into the key's frame, within 64 units in the last place of
    // the 0.3 m its coordinates stand under.
    let dome = key
        .capstan_screw
        .dome_radius
        .ok_or("his capstan has no dome")?;
    let top = key.capstan_contact();
    let (dx, dy) = (at.x - key.capstan, at.y - (top.y - dome));
    let off = (dx * dx + dy * dy).sqrt() - dome;
    if off.abs() > 64.0 * f64::EPSILON * 0.3 {
        return Err(format!(
            "key weight ratio: read at {at:?}, {off} m off his dome"
        ));
    }
    Ok((rounds, bench))
}

/// His instrument, built from `text` as his profile.
fn built(text: &str) -> Result<Instrument, String> {
    let profile = Profile::load("hirschkorn", text).map_err(|e| format!("{e:?}"))?;
    let profile: &'static Profile = Box::leak(Box::new(profile));
    Instrument::of(profile).map_err(|e| format!("{e:?}"))
}

/// His profile with the binding of `param` replaced by `binding`.
fn planted(param: &str, binding: &str) -> String {
    let mut file: Value = serde_json::from_str(PROFILE).expect("his profile");
    let values = file["values"].as_array_mut().expect("its values");
    let i = values
        .iter()
        .position(|v| v["param"] == param)
        .unwrap_or_else(|| panic!("{param} bound"));
    values[i] = serde_json::from_str(binding).expect("a binding");
    serde_json::to_string(&file).expect("his profile, planted")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument::{Action, Outcome, Stack};
    use crate::registry::path;
    use pa_core::contact::{ContactCode, PartCode, PartId};
    use pa_core::parts::mounts::Mounted;
    use pa_core::parts::verdicts::wippen_carried_by;

    fn his() -> Instrument {
        Instrument::of(Profile::named("hirschkorn").expect("shipped")).expect("his instrument")
    }

    /// The links that carry his wippen at his key's two seats on `b`, top
    /// then bottom, each with the parts of the heel it bears on.
    fn carried_at_the_seats(b: &Instrument) -> [Vec<(ContactCode, Vec<PartCode>)>; 2] {
        let (k, seats) = b.seated().expect("his key seats");
        [&seats.top.q, &seats.bottom.q].map(|q| {
            wippen_carried_by(&k.mounted, q)
                .into_iter()
                .map(|c| (c.link, c.parts))
                .collect()
        })
    }

    /// How deep the parts of the wippen that `wippen_part` picks stand in
    /// the parts of the key that `key_part` picks, at `q`, meters, read off
    /// their outlines.
    fn standing_in_the_key(
        mounted: &Mounted,
        q: &[f64],
        wippen_part: fn(PartCode) -> bool,
        key_part: fn(PartCode) -> bool,
    ) -> f64 {
        let m = &mounted.mechanism;
        let wippen = mounted.wippen.as_ref().expect("his key carries his wippen");
        let picked = |body, keep: fn(PartCode) -> bool| -> Vec<PartId> {
            (0..m.parts().len())
                .filter(|&i| m.parts()[i].body == body && keep(m.parts()[i].code))
                .map(|i| PartId(u16::try_from(i).expect("fewer than 65 536 parts")))
                .collect()
        };
        m.parts_overlap(
            &m.pose(q),
            &picked(wippen.body, wippen_part),
            &picked(mounted.key, key_part),
        )
    }

    /// Consistency across implementations, not validation: **his wippen
    /// rests where his capstan closes on his heel.** Script 12 closes his
    /// contact 1, the capstan's dome on the heel's circle Pw5, with the key
    /// at rest: on his rounds, where script 10 closes the whole action, and
    /// on the real felts the bench bears on, which turn the key 5.6
    /// microradians further nose-up at rest, and so stand the heel 11
    /// microradians lower. The bench's top seat stands the wippen where
    /// script 12's felts do, to within its print and what the static solve
    /// closes a contact to: a picometer over the heel's arm, and the key's
    /// own closure carried through the capstan's arm over the heel's.
    #[test]
    fn his_wippen_rests_where_his_capstan_closes_on_his_heel() {
        let b = his();
        let fixture: Value = serde_json::from_str(FIXTURE).expect("the fixture");
        let printed = |what: &str| fixture["script_12"][what].as_f64().expect(what);
        let rad = fixture["script_12"]["printed_to"]["rad"]
            .as_f64()
            .expect("rad");
        let (k, seats) = b.seated().expect("his key seats");
        let wippen = k
            .mounted
            .wippen
            .as_ref()
            .expect("his key carries his wippen");
        let rest = seats.top.q[wippen.angle.0];
        let heel = Profile::named("hirschkorn")
            .expect("shipped")
            .start
            .wippen_assembly
            .as_ref()
            .expect("his profile carries his wippen")
            .assembly
            .heel
            .position;
        let ratio = -b.key().capstan / heel;
        let closed = 1e-12 / heel + ratio * 1e-12 / 0.2;
        let felts = printed("rest_wippen_rad_on_the_felts");
        assert!(
            (rest - felts).abs() <= 0.5 * rad + closed,
            "his wippen rests at {rest} rad, script 12 closes it at {felts} rad on the felts"
        );
    }

    /// Consistency: **his capstan bears on his heel cloth at both seats**,
    /// and nothing else carries his wippen: the capstan's dome under the
    /// heel's circle, 1.7 mm in front of its lowest point at the top of the
    /// stroke, well inside the cloth that runs on 5.55 mm behind and 10.84 mm
    /// in front of the circle's center.
    #[test]
    fn his_capstan_bears_on_his_heel_cloth_at_both_seats() {
        let on_the_cloth = vec![(
            ContactCode::CapstanOnTheHeel,
            vec![PartCode::WippenHeelCloth],
        )];
        assert_eq!(
            carried_at_the_seats(&his()),
            [on_the_cloth.clone(), on_the_cloth]
        );
    }

    /// Consistency: **at his settings, his capstan lowered to the page's
    /// least carries his heel alone, at both seats.** Lowered from where
    /// his profile stands it by the page's half millimeter down to the
    /// page's least 4 mm, the capstan carries his heel at each step at both
    /// seats, and his heel never comes down on his key's top beside it.
    /// Nothing is claimed with his other settings moved.
    #[test]
    fn at_his_settings_his_capstan_lowered_to_the_pages_least_carries_his_heel_alone() {
        const LEAST_MM: f64 = 4.0;
        const STEP_MM: f64 = 0.5;
        let mut b = his();
        let from = b.value(path::CAPSTAN_HEIGHT).expect("his capstan");
        let steps = ((from - LEAST_MM) / STEP_MM).floor() as u32;
        let links = |b: &Instrument| {
            carried_at_the_seats(b).map(|at| at.into_iter().map(|(link, _)| link).collect())
        };
        let mut found = Vec::new();
        for k in (0..=steps).rev() {
            let height = LEAST_MM + f64::from(k) * STEP_MM;
            b.set(path::CAPSTAN_HEIGHT, height)
                .unwrap_or_else(|why| panic!("{height} mm: {why:?}"));
            found.push((height, links(&b)));
        }
        let alone: [Vec<ContactCode>; 2] = [
            vec![ContactCode::CapstanOnTheHeel],
            vec![ContactCode::CapstanOnTheHeel],
        ];
        let expected: Vec<_> = (0..=steps)
            .rev()
            .map(|k| (LEAST_MM + f64::from(k) * STEP_MM, alone.clone()))
            .collect();
        assert!(
            steps > 0,
            "his capstan stands {from} mm, above the page's least"
        );
        assert_eq!(found, expected);
    }

    /// Consistency: **his wippen clears his key everywhere but at the heel,
    /// at both seats.** No part of his wippen assembly but the heel and its
    /// cloth stands in any part of his key, and the heel and its cloth stand
    /// in no part of it but the capstan they rest on: the body over the back
    /// check block and the tail, the jack and the repetition lever over the
    /// keystick, all clear.
    #[test]
    fn his_wippen_clears_his_key_everywhere_but_at_the_heel_at_both_seats() {
        let b = his();
        let (k, seats) = b.seated().expect("his key seats");
        let heel = |code| matches!(code, PartCode::WippenHeel | PartCode::WippenHeelCloth);
        let not_heel = |code| !matches!(code, PartCode::WippenHeel | PartCode::WippenHeelCloth);
        let not_capstan = |code| code != PartCode::Capstan;
        let any = |_| true;
        for q in [&seats.top.q, &seats.bottom.q] {
            assert_eq!(standing_in_the_key(&k.mounted, q, not_heel, any), 0.0);
            assert_eq!(standing_in_the_key(&k.mounted, q, heel, not_capstan), 0.0);
        }
    }

    /// Consistency across implementations, not validation: **the bench
    /// differs from his rounds by the felts alone** ([`c11`]). On real felts
    /// his key falls 9.3080 mm at Pk2, where script 10's rounds let it fall
    /// 9.3064 mm: 1.7 µm more. At rest it bears on the cloth's front edge,
    /// over Pg5, where his round meets its chamfer 0.06 mm further forward.
    /// The difference is his rounds' own. A line resting on a round passes
    /// over the round's center above its top, by the radius times (1/cos of
    /// its slope - 1): on his rounds of 5 mm, 0.39 µm at the back, the
    /// chamfer standing 0.72 degrees from level at rest, and 0.39 µm at the
    /// front, the flat 0.71 degrees at the bottom; on the cloth's edge of
    /// 0.5 mm 0.04 µm, and on the washer read at its center nothing. And
    /// script 10 reads Pk5's normal as printed, 0.99984 long, which stands
    /// the chamfer 0.8 µm further off its round.
    #[test]
    fn the_bench_differs_from_his_rounds_by_the_felts_alone() {
        let (rounds, bench) = c11(&his()).unwrap_or_else(|why| panic!("{why}"));
        let more = bench.dip_at_pk2 - rounds.dip_at_pk2;
        assert!(
            (0.000_001_65..0.000_001_75).contains(&more),
            "the real felts add {more} m to his dip at Pk2"
        );
    }

    /// Consistency, by hand: **his key weight ratio is read where his dome
    /// touches his heel cloth**, at his top seat. The closed row of the
    /// capstan's link there is his dome's arc on his cloth's arc; the hand
    /// puts their contact on the line of the two arcs' centers, at his top
    /// seat's coordinates through the engine's pose, the dome's surface its
    /// radius along it, and carries it into the key's frame. On the level
    /// jig the ratio is that point's arm over the measuring arm, within the
    /// hand's round-off over the measuring arm, `64 ε` of the 0.3 m its
    /// coordinates stand under, and the two jig readings', sixteen units in
    /// the last place of the readings' magnitude over the 10 g each. It reads
    /// 0.5225, his capstan touching his heel 0.64 mm behind his dome's axis,
    /// where on the dome's top it read 0.5197.
    #[test]
    fn his_key_weight_ratio_is_read_where_his_dome_touches_his_heel_cloth() {
        use pa_core::contact::{Piece, Rows, ROW_MARGIN};
        use pa_core::geometry::Seg;
        use pa_core::parts::verdicts::TOUCHING;
        let b = his();
        let (k, seats) = b.seated().expect("his key seats");
        let m = &k.mounted.mechanism;
        let capstans = k.mounted.wippen.expect("his wippen").links[0];
        let pose = m.pose(&seats.top.q);
        let mut rows = Rows::new();
        m.rows(&pose, ROW_MARGIN, &mut rows);
        let closed: Vec<_> = rows
            .of_pair(capstans)
            .filter(|(_, row)| row.gap <= TOUCHING)
            .map(|(_, row)| row.feature)
            .collect();
        let [feature] = closed[..] else {
            panic!("one row of his capstan's link is closed: {closed:?}")
        };
        let disc = |piece: Piece| {
            let part = &m.parts()[piece.part().index()];
            let Piece::Arc { seg, .. } = piece else {
                panic!("{:?} bears with {piece:?}, not an arc", part.code)
            };
            let Seg::Arc { c, r, .. } = part.silhouette()[usize::from(seg)] else {
                unreachable!("an arc's segment")
            };
            (part.code, pose.placement(part.body).world(c), r)
        };
        let (dome, c, r) = disc(feature.a);
        let (cloth, d, _) = disc(feature.b);
        assert_eq!(
            (dome, cloth),
            (PartCode::Capstan, PartCode::WippenHeelCloth)
        );
        let length = ((d.x - c.x) * (d.x - c.x) + (d.y - c.y) * (d.y - c.y)).sqrt();
        let on_the_dome = Point::new(
            c.x + r * (d.x - c.x) / length,
            c.y + r * (d.y - c.y) / length,
        );
        let x_c = pose.placement(k.mounted.key).local(on_the_dome).x;
        let arm = b.measuring_point();
        let sheet = b.sheet();
        let read = |p| match sheet.get(p) {
            Some(Ok(m)) => m.value,
            other => panic!("{p:?}: {other:?}"),
        };
        let load = b.protocol().key_ratio_load_g;
        let readings = (read(ProcedureId::FrontWeight).abs() + load) / load;
        let bound = 64.0 * f64::EPSILON * 0.3 / arm + 2.0 * 16.0 * f64::EPSILON * readings;
        let kr = read(ProcedureId::KeyWeightRatio);
        assert!(
            (kr - -x_c / arm).abs() <= bound,
            "KR {kr}, by hand {}, within {bound}",
            -x_c / arm
        );
    }

    /// Consistency: **the cross-check catches a frame read in another
    /// convention, or a micrometer off.** Table A.1's center of mass read
    /// from the key's front rather than from its pivot moves no stop, and
    /// the front weight shows it; Pg5's height read upside down, its round's
    /// top 5.5 mm above the balance bearing rather than below, puts the key's
    /// tail where no seat is, or another one; the back rail placed by its
    /// center at Pg5's x, as a rail under a round would be, rather than by
    /// its cloth's front edge, puts the cloth's face under the chamfer's
    /// kink; and a rail a micrometer off, in height at the front or at the
    /// back, or along the key at the back, moves the bench's seats off the
    /// felts his printed points place by more than script 10 prints them to,
    /// or the cloth's front edge off Pg5.
    #[test]
    fn the_cross_check_catches_a_frame_read_in_another_convention() {
        for (param, binding) in [
            (
                "published.center.x",
                r#"{ "param": "published.center.x", "value": 186.4, "unit": "mm",
                     "reliability": "S", "why": "planted: Table A.1's 51.4 mm read back from the key front at 237.8 mm" }"#,
            ),
            (
                "back_rail.height",
                r#"{ "param": "back_rail.height", "value": 31.3, "unit": "mm", "reliability": "E",
                     "rule": "planted: 28.8 + 5.5 - 3.0", "why": "planted: Pg5's height read upside down" }"#,
            ),
            (
                "back_rail.x",
                r#"{ "param": "back_rail.x", "record": "hirschkorn-boston-gp178-note52.json",
                     "pointer": "/geometry/ground/points/Pg5/x" }"#,
            ),
            (
                "front_rail.height",
                r#"{ "param": "front_rail.height", "value": 23.221, "unit": "mm", "reliability": "E",
                     "rule": "planted: 23.22 + 0.001", "why": "planted: a micrometer off" }"#,
            ),
            (
                "back_rail.height",
                r#"{ "param": "back_rail.height", "value": 20.301, "unit": "mm", "reliability": "E",
                     "rule": "planted: 20.3 + 0.001", "why": "planted: a micrometer off" }"#,
            ),
            (
                "back_rail.x",
                r#"{ "param": "back_rail.x", "value": -215.699, "unit": "mm", "reliability": "E",
                     "rule": "planted: -215.7 + 0.001", "why": "planted: a micrometer off" }"#,
            ),
        ] {
            let caught = built(&planted(param, binding)).and_then(|b| c11(&b).map(|_| ()));
            assert!(caught.is_err(), "{param} planted, and nothing disagrees");
        }
    }

    /// Consistency: **his key height target is the height his key stands
    /// at**, to the bit of the sheet's reading, since no key height is
    /// published; and **regulating him lays his dip and keeps that height**:
    /// the regulation the page sends, his targets as the settings hold them,
    /// leaves the balance punching's papers as they were and the key height
    /// where it stood, and brings the dip to within one paper past 10.0 mm.
    #[test]
    fn regulating_his_key_lays_his_dip_and_keeps_its_height() {
        let mut b = his();
        let height = |b: &Instrument| match b.sheet().get(ProcedureId::KeyHeight) {
            Some(Ok(m)) => m.value,
            other => panic!("{other:?}"),
        };
        let before = height(&b);
        assert_eq!(b.targets().key_height.to_bits(), before.to_bits());
        let balance = b.papers(Stack::Balance).to_vec();
        let (key_height_mm, key_dip_mm) = (
            b.value(path::KEY_HEIGHT).expect("a target"),
            b.value(path::KEY_DIP).expect("a target"),
        );
        assert_eq!(key_dip_mm, 10.0);
        let laid = b
            .act(Action::Regulate {
                key_height_mm,
                key_dip_mm,
            })
            .expect("his dip is regulable");
        assert!(matches!(laid, Outcome::Laid(_)));
        assert_eq!(b.papers(Stack::Balance), &balance[..]);
        assert_eq!(height(&b).to_bits(), before.to_bits());
        let dip = match b.sheet().get(ProcedureId::KeyDip) {
            Some(Ok(m)) => m.value,
            other => panic!("{other:?}"),
        };
        let paper = b.frame().drawer.thinnest();
        assert!(
            (0.0100..0.0100 + paper).contains(&dip),
            "his dip regulated to {dip} m"
        );
    }

    /// Consistency: **selecting a profile puts its instrument on the bench**,
    /// as that profile builds it, to the bit of its state hash, and a reset
    /// returns to it; a name no profile has is refused, the bench left as it
    /// was.
    #[test]
    fn selecting_a_profile_puts_its_instrument_on_the_bench() {
        let section = || {
            Instrument::of(Profile::named(crate::profile::SECTION).expect("shipped"))
                .expect("the measured section")
        };
        assert_eq!(Instrument::standard().state_hash(), his().state_hash());
        let mut b = section();
        b.set(path::WEIGHT, 20.0).expect("a weight");
        b.act(Action::SelectProfile {
            profile: "hirschkorn".to_owned(),
        })
        .expect("his profile");
        assert_eq!(b.state_hash(), his().state_hash());
        b.set(path::FRONT_EDGE, 230.0).expect("a shorter key");
        b.act(Action::Reset).expect("reset");
        assert_eq!(b.state_hash(), his().state_hash());
        let hash = b.state_hash();
        assert!(b
            .act(Action::SelectProfile {
                profile: "nobody".to_owned(),
            })
            .is_err());
        assert_eq!(b.state_hash(), hash);
        b.act(Action::SelectProfile {
            profile: crate::profile::SECTION.to_owned(),
        })
        .expect("the measured section");
        assert_eq!(b.state_hash(), section().state_hash());
        assert_eq!(b.log().len(), 5, "the log kept through every selection");
    }

    /// Consistency: **his reading at whole degrees stands above his
    /// wippen's continuous nearest by no more than half a degree of its
    /// turn.** His center pin raised by the millimeter from where his top
    /// seat first hangs his wippen out of his key's reach, found by halving
    /// between his own height and 110.1 mm, which stand either side of it,
    /// to 40 mm above it, his capstan's dome under his heel, the key alone at
    /// its top seat: the bench reads his heel's nearest approach to his key
    /// at the whole degrees it decides reach at
    /// ([`Mounted::measure_the_heel_past`]); between two of them the heel
    /// turns by its reach times half a degree at most from the nearer, so the
    /// reading stands no lower than the nearest over the whole turn
    /// ([`read::nearest_over_its_whole_turn`]), and no higher than that by
    /// more than the reach times half the widest step: the nearest is
    /// resolved from above, so the excess measured is no more than the true
    /// one. A measurement of the bench's resolution, not of a rule it could
    /// break: the largest excess, 0.11 mm a millimeter past his boundary, is
    /// the figure `docs/FOUNDATIONS.md` > Where a solve starts states.
    #[test]
    fn his_reading_at_whole_degrees_stands_above_his_wippens_continuous_nearest_by_no_more_than_half_a_degree_of_its_turn(
    ) {
        use crate::instrument::read;
        use pa_core::measure::seat::{SeatRefusal, WhichSeat};
        use pa_core::parts::mounts::WippenLimit;
        const OUT_OF_REACH: f64 = 0.1101;
        const ABOVE_HIS_BOUNDARY_MM: u32 = 40;
        const STATED_LARGEST_EXCESS: f64 = 0.11e-3;
        const STATED_ABOVE_HIS_BOUNDARY_MM: u32 = 1;
        let raised = |center: f64| {
            let mut b = his();
            b.set(path::WIPPEN_CENTER_ABOVE, center * 1e3)
                .expect("a center pin within its hard range");
            b
        };
        let reaches = |center: f64| {
            !matches!(
                raised(center).seated(),
                Err(SeatRefusal::WippenOutOfReach {
                    at: WhichSeat::Top,
                    ..
                })
            )
        };
        let own = his()
            .value(path::WIPPEN_CENTER_ABOVE)
            .expect("his wippen's center")
            * 1e-3;
        assert!(
            !reaches(OUT_OF_REACH) && reaches(own),
            "the halving's ends stand either side of his boundary"
        );
        let (boundary, _) = read::halve_between(OUT_OF_REACH, own, reaches);
        let mut largest = (0.0, 0);
        for above_mm in 0..=ABOVE_HIS_BOUNDARY_MM {
            let center = boundary + f64::from(above_mm) * 1e-3;
            let (mounted, q) = read::at_the_key_alones_top_seat(&raised(center));
            let reading = mounted.measure_the_heel_past(&q, WippenLimit::OutOfReach);
            let (nearest, widest) = read::nearest_over_its_whole_turn(&mounted, &q);
            let reach = mounted.wippen.expect("his wippen").reach;
            let excess = reading - nearest;
            assert!(
                excess >= 0.0 && excess <= reach * 0.5 * widest,
                "{center} m: {reading} m read, {nearest} m at its nearest"
            );
            if excess > largest.0 {
                largest = (excess, above_mm);
            }
        }
        assert!(
            largest.0 <= STATED_LARGEST_EXCESS && largest.1 == STATED_ABOVE_HIS_BOUNDARY_MM,
            "the largest excess is {} m, {} mm above his boundary at {boundary} m",
            largest.0,
            largest.1
        );
    }
}
