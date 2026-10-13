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
//! Every field of [`BenchReadout`] (or that there is none, when the bench
//! cannot weigh its frame), the reason it cannot, the faults, where the key is
//! drawn and the papers on both punchings. Numbers are written as the shortest
//! decimal that reads back to the same double, then as its bit pattern.
//!
//! Two readings are recorded one step earlier than the sheet shows them. The
//! bench turns the bedding at the back into degrees with `atan`, and the
//! out-of-level at the bottom with `asin`, both from the platform's maths
//! library, whose last bit is allowed to differ between machines. The sheet
//! records the slope and the sine the engine computes, which hold bit for bit
//! wherever the tests run; the degrees are a function of them and of nothing
//! else.
//!
//! # The fixtures
//!
//! The default bench, and four benches that each exercise a part of the
//! statics the default does not: a frame whose stops fight, a back-heavy key,
//! a dip stopped by the pin in its mortise, and a key shortened under a frame
//! left where it was.

use std::fmt::Write as _;

use crate::key_bench::{BenchReadout, Fault, KeyBench, Which};

/// One bench whose readings the golden sheet records.
#[derive(Debug, Clone, Copy)]
pub struct Fixture {
    /// Stable identifier, and the file's stem: `tests/golden/<name>.txt`.
    pub name: &'static str,
    /// What the bench is, in a line.
    pub what: &'static str,
    /// Builds the bench.
    pub build: fn() -> KeyBench,
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
        build: KeyBench::standard,
    },
    Fixture {
        name: "wedged-frame",
        what: "the standard bench with the back rail raised until its cloth holds the tail \
               5 mm above the balance bearing: the stops fight and the key rides on both",
        build: wedged_frame,
    },
    Fixture {
        name: "back-heavy-key",
        what: "the standard bench with 30 g of lead 120 mm behind the balance pin: a key \
               whose front weight is negative",
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
               where it was: the front pin stands outside its mortise, under the key",
        build: shortened_key,
    },
];

fn wedged_frame() -> KeyBench {
    let mut bench = KeyBench::standard();
    let frame = &bench.assembly().frame;
    let height = frame.balance_bearing_height() + 0.005 - frame.back_rail_cloth;
    bench.set_back_rail_height(height);
    bench
}

fn back_heavy_key() -> KeyBench {
    let mut bench = KeyBench::standard();
    bench.add_lead(-0.120, 30.0);
    bench
}

fn mortise_limited_dip() -> KeyBench {
    let mut bench = KeyBench::standard();
    let (pin, _) = bench.front_pin_and_mortise();
    bench.set_front_pin_height(pin + 0.001);
    bench
}

fn shortened_key() -> KeyBench {
    let mut bench = KeyBench::standard();
    let (_, front_edge, _) = bench.key_dimensions();
    bench.set_front_edge(front_edge - 0.018);
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

fn write_readout(out: &mut String, bench: &KeyBench) {
    let Some(readout) = bench.readout() else {
        line(out, "readout", "none");
        return;
    };
    // Destructured field by field and without `..`, so that a field added to
    // the readout is a compile error here until the sheet records it.
    let BenchReadout {
        front_weight_g,
        key_ratio,
        key_height_mm,
        key_dip_mm,
        down_weight_g,
        up_weight_g,
        balance_weight_g,
        friction_weight_g,
        key_mass_g,
        lead_count,
        gap_front_mm,
        gap_back_mm,
        measured_dip_mm,
        measured_kf_g,
        friction_band_g,
        key_height_target_mm,
        key_dip_target_mm,
        finger,
        out_of_level_deg: _,
        bedding_at_the_back_deg: _,
        bears_at_the_back_mm,
        back_bearing,
        stopped_by_the_mortise,
    } = readout;

    number(out, "readout.front_weight_g", front_weight_g);
    number(out, "readout.key_ratio", key_ratio);
    number(out, "readout.key_height_mm", key_height_mm);
    number(out, "readout.key_dip_mm", key_dip_mm);
    number(out, "readout.down_weight_g", down_weight_g);
    number(out, "readout.up_weight_g", up_weight_g);
    number(out, "readout.balance_weight_g", balance_weight_g);
    number(out, "readout.friction_weight_g", friction_weight_g);
    number(out, "readout.key_mass_g", key_mass_g);
    line(out, "readout.lead_count", &lead_count.to_string());
    optional(out, "readout.gap_front_mm", gap_front_mm);
    optional(out, "readout.gap_back_mm", gap_back_mm);
    optional(out, "readout.measured_dip_mm", measured_dip_mm);
    optional(out, "readout.measured_kf_g", measured_kf_g);
    number(out, "readout.friction_band_g", friction_band_g);
    number(out, "readout.key_height_target_mm", key_height_target_mm);
    number(out, "readout.key_dip_target_mm", key_dip_target_mm);
    line(out, "readout.finger", &format!("{finger:?}"));
    // The two angles, one step before the platform's `asin` and `atan` turn
    // them into the degrees the sheet shows (module documentation).
    optional(
        out,
        "readout.out_of_level.sine",
        bench.assembly().sin_angle_on_front_rail(),
    );
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

fn write_frame(out: &mut String, bench: &KeyBench) {
    line(out, "frame_fault", bench.frame_fault().unwrap_or("none"));
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
    for (which, name) in [(Which::Balance, "balance"), (Which::Front, "front")] {
        let papers = bench.papers_on(which);
        line(out, &format!("papers.{name}"), &papers.len().to_string());
        for (i, paper) in papers.iter().enumerate() {
            number(out, &format!("papers.{name}.{i}"), *paper);
        }
    }
}

fn write_pose(out: &mut String, bench: &KeyBench) {
    number(out, "pose.sin_angle", bench.sin_angle());
    number(out, "pose.depression", bench.depression());
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
        let wedged = wedged_frame();
        assert!(wedged.frame_fault().is_some(), "the stops must fight");
        assert!(wedged.readout().is_none(), "and the bench cannot weigh it");

        let back_heavy = back_heavy_key().readout().expect("a back-heavy key weighs");
        assert!(
            back_heavy.front_weight_g < 0.0,
            "{}",
            back_heavy.front_weight_g
        );

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
        assert!(short.readout().is_some(), "and the key still weighs");
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
