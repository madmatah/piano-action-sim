//! The golden sheet: every reading the bench shows, in full bits, for the
//! default bench and a set of fixture benches.
//!
//! # What it is for
//!
//! The engine is rebuilt in steps, and each step is meant to move the
//! numbers it names and nothing else. A number that moves by accident moves
//! quietly: a key height a hundredth of a millimetre off reads as plausible,
//! and a test written against the old code with a tolerance lets it through.
//! So every reading of these benches is recorded bit for bit
//! (`tests/golden/`, one file per bench), and `tests/golden.rs` fails on any
//! difference. A step that means to move a reading rewrites the files with
//! `cargo run -p pa-cli -- golden` and says in its commit which readings moved,
//! by how much and why; the diff of the files is the list.
//!
//! # What a sheet holds
//!
//! Every reading of the weighing sheet ([`Instrument::sheet`]): its value in
//! full, what the trade reads it as, its trust, and for the key friction
//! weights the temporary weight and where it stood; or why it is refused.
//! Then every field of [`BenchReadout`] (or that there is none, when the
//! bench cannot seat its key), the reason it cannot, how the key rides on
//! its stops where they fight (written only there), the faults, where the
//! key is drawn and the papers on both punchings. Numbers are written as the
//! shortest decimal that reads back to the same double, then as its bit
//! pattern.
//!
//! One reading is recorded one step earlier than the sheet shows it. The
//! bench turns the bedding at the back into degrees with `atan`, from the
//! platform's maths library, whose last bit is allowed to differ between
//! machines. The sheet records the slope the engine computes, which holds bit
//! for bit wherever the tests run; the degrees are a function of it and of
//! nothing else. The out-of-level at the bottom is the bottom seat's angle
//! turned into degrees by a product, and is recorded as it is shown.
//!
//! # The fixtures
//!
//! The default bench, and four benches that each exercise a part of the
//! statics the default does not: a frame whose stops fight, its key riding
//! on them off its balance punching; a back-heavy key; a dip stopped by the
//! pin in its mortise; and a key shortened under a frame left where it was,
//! whose front pin then stands under its wood and carries it. Each is built
//! as an interface builds it, through the instrument's own calls, a path
//! set or an action taken: the value a fixture sets is the one it computes
//! in metres, sent in the path's millimetres, which turn back into the same
//! metres to the bit.

use std::fmt::Write as _;

use crate::instrument::{
    Action, BenchReadout, Fault, Instrument, Measurement, Riding, Stack, TouchVerdict, Trust,
};
use crate::registry::path;

/// One bench whose readings the golden sheet records.
#[derive(Debug, Clone, Copy)]
pub struct Fixture {
    /// Stable identifier, and the file's stem: `tests/golden/<name>.txt`.
    pub name: &'static str,
    /// What the bench is, in a line.
    pub what: &'static str,
    /// Builds the bench.
    pub build: fn() -> Instrument,
}

impl Fixture {
    /// The file this fixture's sheet is kept in, relative to `tests/golden/`.
    pub fn file_name(&self) -> String {
        format!("{}.txt", self.name)
    }
}

/// Every bench the golden sheet records, in the order the sheets are written.
pub const FIXTURES: [Fixture; 5] = [
    Fixture {
        name: "default",
        what: "the standard bench: a middle natural on its frame, regulated to 65.0 mm of \
               key height and 10.2 mm of key dip",
        build: Instrument::standard,
    },
    Fixture {
        name: "wedged-frame",
        what: "the standard bench with the back rail raised until its cloth holds the tail \
               5 mm above the balance bearing: the stops fight and the key rides on both",
        build: wedged_frame,
    },
    Fixture {
        name: "back-heavy-key",
        what: "the standard bench with its key weighed back-heavy: 151.12 g, and a front \
               weight of -8.09 g read 13 mm back from its lip",
        build: back_heavy_key,
    },
    Fixture {
        name: "mortise-limited-dip",
        what: "the standard bench with the front rail pin 1 mm prouder: the pin bottoms in \
               its mortise before the key reaches its front punching",
        build: mortise_limited_dip,
    },
    Fixture {
        name: "shortened-key",
        what: "the standard bench with the key front brought back 18 mm, the frame left \
               where it was: the front pin stands outside its mortise, under the key, and the \
               key rides on it and its cloth",
        build: shortened_key,
    },
];

fn wedged_frame() -> Instrument {
    let mut bench = Instrument::standard();
    let frame = bench.frame();
    let height = frame.balance_bearing_height() + 0.005 - frame.back_rail_cloth;
    bench
        .set(path::BACK_RAIL_HEIGHT, height * 1000.0)
        .expect("a back rail raised under its key");
    bench
}

/// The key weighed whole and back-heavy, typed in full: the mass and the
/// front weight at 13 mm of the composite key with 30 g more, 120 mm behind
/// its balance pin, so the same mass and static moment, the height of its
/// centre of mass the volume model's.
fn back_heavy_key() -> Instrument {
    let mut bench = Instrument::standard();
    bench
        .act(Action::WeighKey {
            mass_g: 151.116_689_639_242_2,
            front_weight_g: -8.093_919_058_496_057,
            from_lip_mm: 13.0,
        })
        .expect("a back-heavy key is weighed");
    bench
}

fn mortise_limited_dip() -> Instrument {
    let mut bench = Instrument::standard();
    let pin = bench.frame().sections.front.pin_height;
    bench
        .set(path::FRONT_PIN_HEIGHT, (pin + 0.001) * 1000.0)
        .expect("a front pin a millimetre prouder");
    bench
}

fn shortened_key() -> Instrument {
    let mut bench = Instrument::standard();
    let front_edge = bench.key().front_edge;
    bench
        .set(path::FRONT_EDGE, (front_edge - 0.018) * 1000.0)
        .expect("a key front brought back");
    bench
}

/// The golden sheet of one fixture, as its file holds it.
pub fn sheet(fixture: &Fixture) -> String {
    let bench = (fixture.build)();
    let mut out = String::new();
    out.push_str(&format!("# golden sheet: {}\n", fixture.name));
    out.push_str(&format!("# {}\n", fixture.what));
    out.push_str(
        "# Every reading in full bits: the shortest decimal that reads back to the\n\
         # same double, then its bit pattern. Written by `cargo run -p pa-cli -- golden`;\n\
         # a change that moves a reading rewrites this file and says why.\n",
    );
    write_sheet(&mut out, &bench);
    write_readout(&mut out, &bench);
    write_frame(&mut out, &bench);
    write_pose(&mut out, &bench);
    out
}

/// A number in full bits.
fn bits(x: f64) -> String {
    format!("{x:?} 0x{:016x}", x.to_bits())
}

fn line(out: &mut String, name: &str, value: &str) {
    // A `String` cannot fail to take a write.
    let _ = writeln!(out, "{name} {value}");
}

fn number(out: &mut String, name: &str, x: f64) {
    line(out, name, &bits(x));
}

fn optional(out: &mut String, name: &str, x: Option<f64>) {
    match x {
        Some(x) => number(out, name, x),
        None => line(out, name, "none"),
    }
}

fn write_sheet(out: &mut String, bench: &Instrument) {
    let sheet = bench.sheet();
    for reading in &sheet.readings {
        let name = format!("sheet.{}", reading.procedure.code());
        match &reading.result {
            Ok(m) => write_measurement(out, &name, m),
            Err(why) => line(out, &name, &format!("refused {why:?}")),
        }
    }
}

fn write_measurement(out: &mut String, name: &str, m: &Measurement) {
    // Destructured without `..`, so that a field added to a measurement is a
    // compile error here until the sheet records it, or says why not.
    let Measurement {
        procedure: _,
        value,
        shown,
        unit,
        resolution: _,
        rounding: _,
        conditions,
        trust,
        report: _,
        fingerprint,
    } = m;
    // The procedure is the line's name; the resolution and the rounding are
    // the procedure's own, fixed with it; the report counts the solver's
    // steps, a path and not a reading, which the fingerprint answers for.
    number(out, &format!("{name}.value"), *value);
    number(out, &format!("{name}.shown"), *shown);
    line(out, &format!("{name}.unit"), unit.code());
    let trust = match trust {
        Trust::Metrological => "metrological".to_owned(),
        Trust::Indicative { because } => format!(
            "indicative {}",
            because
                .iter()
                .map(|i| i.code())
                .collect::<Vec<_>>()
                .join(",")
        ),
    };
    line(out, &format!("{name}.trust"), &trust);
    line(out, &format!("{name}.fingerprint"), fingerprint);
    if let Some(walk) = conditions.walk {
        number(out, &format!("{name}.walk.from_g"), walk.from_g);
    }
    if let Some(t) = conditions.temporary_weight {
        number(out, &format!("{name}.temporary_weight.mass_g"), t.mass_g);
        number(
            out,
            &format!("{name}.temporary_weight.station_x"),
            t.station.x,
        );
        number(
            out,
            &format!("{name}.temporary_weight.station_y"),
            t.station.y,
        );
    }
}

fn write_readout(out: &mut String, bench: &Instrument) {
    let Some(readout) = bench.readout() else {
        line(out, "readout", "none");
        return;
    };
    // Destructured field by field and without `..`, so that a field added to
    // the readout is a compile error here until the sheet records it.
    let BenchReadout {
        key_mass_g,
        gap_front_mm,
        gap_back_mm,
        key_friction_weight_g,
        key_height_target_mm,
        key_dip_target_mm,
        finger,
        out_of_level_deg,
        bedding_at_the_back_deg: _,
        bears_at_the_back_mm,
        back_bearing,
        stopped_by_the_mortise,
    } = readout;

    number(out, "readout.key_mass_g", key_mass_g);
    optional(out, "readout.gap_front_mm", gap_front_mm);
    optional(out, "readout.gap_back_mm", gap_back_mm);
    number(out, "readout.key_friction_weight_g", key_friction_weight_g);
    number(out, "readout.key_height_target_mm", key_height_target_mm);
    number(out, "readout.key_dip_target_mm", key_dip_target_mm);
    match finger {
        TouchVerdict::RestingOn { stop } => {
            line(out, "readout.finger", &format!("RestingOn {stop:?}"));
        }
        TouchVerdict::HeldByFriction { touching, margin_g } => {
            let on = touching.map_or_else(|| "nothing".to_owned(), |c| format!("{c:?}"));
            line(
                out,
                "readout.finger",
                &format!("HeldByFriction touching {on}"),
            );
            number(out, "readout.finger.margin_g", margin_g);
        }
        TouchVerdict::RidesOnBothStops { front, lift_mm } => {
            line(
                out,
                "readout.finger",
                &format!("RidesOnBothStops {front:?}"),
            );
            number(out, "readout.finger.lift_mm", lift_mm);
        }
        TouchVerdict::LiftsOffBalancePunching {
            touching,
            needs_friction,
            lift_mm,
        } => {
            let on: Vec<String> = touching.iter().map(|c| format!("{c:?}")).collect();
            line(
                out,
                "readout.finger",
                &format!(
                    "LiftsOffBalancePunching touching {} needs_friction {needs_friction}",
                    on.join(",")
                ),
            );
            number(out, "readout.finger.lift_mm", lift_mm);
        }
        TouchVerdict::LiftsOffItsFrame => line(out, "readout.finger", "LiftsOffItsFrame"),
    }
    number(out, "readout.out_of_level_deg", out_of_level_deg);
    // The slope, one step before the platform's `atan` turns it into the
    // degrees the sheet shows (module documentation).
    optional(
        out,
        "readout.bedding_at_the_back.slope",
        bench
            .bearing_at_the_back()
            .and_then(|bearing| bearing.slope),
    );
    number(out, "readout.bears_at_the_back_mm", bears_at_the_back_mm);
    line(out, "readout.back_bearing", &format!("{back_bearing:?}"));
    line(
        out,
        "readout.stopped_by_the_mortise",
        &stopped_by_the_mortise.to_string(),
    );
}

fn write_frame(out: &mut String, bench: &Instrument) {
    match bench.frame_fault() {
        Some(why) => line(out, "frame_fault", &format!("{why:?}")),
        None => line(out, "frame_fault", "none"),
    }
    // Where the key rides on its stops, seated: written only there, so that
    // a sheet whose key bears on its punching reads as it did.
    if let Some(Riding { front, lift }) = bench.riding() {
        line(out, "riding.front", &format!("{front:?}"));
        number(out, "riding.lift", lift);
    }
    let faults = bench.faults();
    line(out, "faults", &faults.len().to_string());
    for (i, fault) in faults.iter().enumerate() {
        let (code, by) = match *fault {
            Fault::PinOutsideTheMortise { by } => ("PinOutsideTheMortise".to_owned(), by),
            Fault::ClothInFrontOfTheChamfer { by } => ("ClothInFrontOfTheChamfer".to_owned(), by),
            Fault::ClothBehindTheKey { by } => ("ClothBehindTheKey".to_owned(), by),
            Fault::PunchingOffTheUnderside { rail, by } => {
                (format!("PunchingOffTheUnderside {}", rail.code()), by)
            }
        };
        line(out, &format!("fault.{i}"), &code);
        number(out, &format!("fault.{i}.by"), by);
    }
    for (stack, name) in [(Stack::Balance, "balance"), (Stack::Front, "front")] {
        let papers = bench.papers(stack);
        line(out, &format!("papers.{name}"), &papers.len().to_string());
        for (i, paper) in papers.iter().enumerate() {
            number(out, &format!("papers.{name}.{i}"), *paper);
        }
    }
}

fn write_pose(out: &mut String, bench: &Instrument) {
    number(out, "pose.sin_angle", bench.sin_angle());
    // Where the view marks it: the key's pose applied to the point on the key.
    let point = bench.key_placement().world(bench.measuring_point_on_key());
    number(out, "pose.measuring_point.x", point.x);
    number(out, "pose.measuring_point.y", point.y);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each fixture is the bench it says it is: the property it was chosen
    /// for holds, so the sheet records that case and not the default again.
    #[test]
    fn every_fixture_exercises_what_it_names() {
        use crate::instrument::{ContactCode, ProcedureId, Refused, WalkRefusal};
        let wedged = wedged_frame();
        let riding = wedged.riding().expect("the stops must fight, the key ride");
        assert_eq!(riding.front, ContactCode::FrontRailPunching);
        assert!(wedged.frame_fault().is_none(), "a riding key is seated");
        assert!(wedged.readout().is_some(), "and reads what needs no stroke");
        assert_eq!(
            wedged.sheet().get(ProcedureId::DownWeight),
            Some(&Err(Refused::Walk(WalkRefusal::NoTravel))),
            "and has no stroke to weigh"
        );

        let back_heavy = back_heavy_key().sheet();
        let front_weight = back_heavy
            .get(crate::instrument::ProcedureId::FrontWeight)
            .and_then(|r| r.as_ref().ok())
            .expect("a back-heavy key weighs")
            .value;
        assert!(front_weight < 0.0, "{front_weight}");

        let mortise = mortise_limited_dip()
            .readout()
            .expect("a mortise-limited key weighs");
        assert!(mortise.stopped_by_the_mortise, "the pin must bottom first");

        let short = shortened_key();
        assert!(
            short
                .faults()
                .iter()
                .any(|f| matches!(f, Fault::PinOutsideTheMortise { .. })),
            "the pin must stand outside the mortise"
        );
        assert_eq!(
            short.riding().map(|r| r.front),
            Some(ContactCode::FrontPinOnTheWood),
            "and, standing under the key's wood, carry it"
        );
    }

    /// Fixture names are distinct, so no two sheets share a file.
    #[test]
    fn every_fixture_has_its_own_file() {
        let mut names: Vec<_> = FIXTURES.iter().map(|f| f.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), FIXTURES.len());
    }

    /// A number is written so that it reads back to the same bits.
    #[test]
    fn a_number_is_written_in_full_bits() {
        for x in [6.48, -0.0, 1.0 / 3.0, 1e-300, 0.1 + 0.2] {
            let written = bits(x);
            let (decimal, hex) = written.split_once(' ').expect("two fields");
            let read: f64 = decimal.parse().expect("a decimal");
            assert_eq!(read.to_bits(), x.to_bits(), "{written}");
            assert_eq!(hex, format!("0x{:016x}", x.to_bits()));
        }
    }
}
