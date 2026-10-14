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
//! felts, a flat front punching washer and a flat back rail cloth strip,
//! each with its top where his round of 5 mm has its, where his model bears
//! on the rounds. That difference is derived here in closed form, from his
//! printed points and the felts the bench draws, and the bench is held to
//! it.

use pa_core::geometry::Point;
use pa_core::measure::measurement::ProcedureId;
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

/// A printed point of his, metres, in its body's frame.
fn point(record: &Value, body: &str, name: &str) -> Point {
    let at = format!("/geometry/{body}/points/{name}");
    Point::new(
        si(record, &format!("{at}/x")),
        si(record, &format!("{at}/y")),
    )
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
/// of radius `r` about `centre`: his model's stop, closed as script 10 and
/// the prototype close it. A bisection of a hundred halvings over a tenth
/// of a radian.
fn on_a_round(through: Point, n: Point, centre: Point, r: f64, lo: f64, hi: f64) -> f64 {
    let gap = |t: f64| {
        let (p, m) = (turned(t, through), turned(t, n));
        (centre.x - p.x) * m.x + (centre.y - p.y) * m.y - r
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
/// Pk2, metres.
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
/// (`docs/FOUNDATIONS.md` §5.1): the instrument built on Hirschkorn's
/// profile against what script 10 and the prototype print of his key between
/// its stops, and against Table A.1. `Err` names what disagrees.
///
/// - **His rounds, closed again here** from the printed points as script 10
///   closes them, reproduce what script 10 prints, to its last digit: this
///   test reads his frame as script 10 does.
/// - **The felts, in closed form** from his printed points alone, never
///   from the bench's frame: each felt's top where his round's top is, the
///   cloth centred under Pg5 and the washer under Pg4, as the bench's must
///   be; at rest the key, nose-up, bears with its chamfer's kink, where
///   Pk5's line meets Pk4's flat, on the cloth's flat face; at the bottom it
///   bears on the front punching read at its centre, as the bench reads
///   every punching. The felts' difference from his rounds, these less
///   those, is the bound: the bench's seats stand
///   where script 10's do, moved by that difference, to within what script
///   10 prints them to and what the static solve closes a contact to (a
///   picometre over arms of a fifth of a metre); and where the prototype's
///   do, moved by it, to within what the prototype's compliant contacts
///   compress under the key at rest, over the arm of the contact
///   compressed.
/// - **Table A.1's static moment** over the measuring arm is the front
///   weight the jig reads, to round-off; and the key weight ratio is the
///   capstan's arm over the measuring arm. These see what the attitudes
///   cannot: a centre of mass read in another frame.
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

    // The felts, in closed form from his printed points alone, as the owner
    // chose them: a flat cloth strip and a flat washer, each with its top
    // where his round has its, the washer read at its centre over Pg4; the
    // key, nose-up at rest, bearing with its chamfer's kink, where Pk5's
    // line meets Pk4's flat, on the cloth's face. Nothing here reads the
    // bench's frame, so a frame read in another convention moves the
    // bench's seats and not these, by however little.
    let n5 = normal(&record, "Pk5");
    if pk4.y != 0.0 || normal(&record, "Pk4") != Point::new(0.0, -1.0) {
        return Err("his underside is no longer the flat y = 0 the felts are closed on".to_owned());
    }
    let kink = pk5.x + n5.y * pk5.y / n5.x;
    let felt_rest = ((pg5.y + r5) / kink).asin();
    let felt_bottom = ((pg4.y + r4) / pg4.x).atan();
    let key = b.key();
    let frame = b.frame();
    let front_x = frame
        .sections
        .front
        .pin_x(frame.front_rail, KeyKind::Natural);
    let back_x = frame.sections.back.centre_x(frame.back_rail);
    if front_x != pg4.x || back_x != pg5.x {
        return Err(format!(
            "the bench's felts stand at {front_x} and {back_x} m, his rounds at {} and {} m",
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
    // the static solve's closure, a picometre over an arm of a fifth of a
    // metre; then the prototype, moved by it, within its compression.
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
    let centre_x = si(&record, "/mass_properties/bodies/key/centre_of_mass/x");
    let arm = b.measuring_point();
    let read = |p| match b.sheet().get(p) {
        Some(Ok(m)) => Ok(m.value),
        other => Err(format!("{p:?}: {other:?}")),
    };
    let front_weight = mass * 1000.0 * centre_x / arm;
    let fw = read(ProcedureId::FrontWeight)?;
    if (fw - front_weight).abs() > 64.0 * f64::EPSILON * front_weight.abs() {
        return Err(format!(
            "front weight: the jig reads {fw} g, Table A.1's static moment over the measuring \
             arm is {front_weight} g"
        ));
    }
    let ratio = -key.capstan / arm;
    let kr = read(ProcedureId::KeyWeightRatio)?;
    if (kr - ratio).abs() > 64.0 * f64::EPSILON {
        return Err(format!(
            "key weight ratio: the jig reads {kr}, the capstan's arm over the measuring arm is \
             {ratio}"
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

    fn his() -> Instrument {
        Instrument::of(Profile::named("hirschkorn").expect("shipped")).expect("his instrument")
    }

    /// Consistency across implementations, not validation: **the bench
    /// differs from his rounds by the felts alone** ([`c11`]). On real felts
    /// his key falls 9.2016 mm at Pk2, where script 10's rounds let it fall
    /// 9.3064 mm: 0.105 mm less, because at rest it bears with its chamfer's
    /// kink on the cloth's face, where his round met its chamfer 7.3 mm
    /// further back.
    #[test]
    fn the_bench_differs_from_his_rounds_by_the_felts_alone() {
        let (rounds, bench) = c11(&his()).unwrap_or_else(|why| panic!("{why}"));
        let less = rounds.dip_at_pk2 - bench.dip_at_pk2;
        assert!(
            (0.000_10..0.000_11).contains(&less),
            "the real felts take {less} m off his dip at Pk2"
        );
    }

    /// Consistency: **the cross-check catches a frame read in another
    /// convention.** Table A.1's centre of mass read from the key's front
    /// rather than from its pivot moves no stop, and the front weight shows
    /// it; Pg5's height read upside down, its round's top 5.5 mm above the
    /// balance bearing rather than below, puts the key's tail where no seat
    /// is, or another one; and a rail's height a micrometre off, which moves
    /// the dip at Pk2 by about a micrometre, well inside "about 0.1 mm",
    /// moves the bench's seats off the felts his printed points place.
    #[test]
    fn the_cross_check_catches_a_frame_read_in_another_convention() {
        for (param, binding) in [
            (
                "published.centre.x",
                r#"{ "param": "published.centre.x", "value": 186.4, "unit": "mm",
                     "reliability": "S", "why": "planted: Table A.1's 51.4 mm read back from the key front at 237.8 mm" }"#,
            ),
            (
                "back_rail.height",
                r#"{ "param": "back_rail.height", "value": 31.3, "unit": "mm", "reliability": "E",
                     "rule": "planted: 28.8 + 5.5 - 3.0", "why": "planted: Pg5's height read upside down" }"#,
            ),
            (
                "front_rail.height",
                r#"{ "param": "front_rail.height", "value": 23.221, "unit": "mm", "reliability": "E",
                     "rule": "planted: 23.22 + 0.001", "why": "planted: a micrometre off" }"#,
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
        let mut b = Instrument::standard();
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
            profile: "composite".to_owned(),
        })
        .expect("the composite");
        assert_eq!(b.state_hash(), Instrument::standard().state_hash());
        assert_eq!(b.log().len(), 5, "the log kept through every selection");
    }
}
