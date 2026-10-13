//! Command-line adapter of `pa-app`.
//!
//! The same code that runs in the browser runs here natively. Two commands:
//! `weigh` prints the key weighing sheet, so that the numbers can be read and
//! checked without a browser in the way; `golden` writes the golden sheet,
//! every reading of the default bench and of the fixture benches in full bits,
//! which `crates/pa-app/tests/golden.rs` compares the engine against.

use std::path::{Path, PathBuf};

use pa_app::golden;
use pa_app::key_bench::{KeyBench, KeyMass, ProcedureId, Sheet, Trust, Unit};

/// Where `golden` writes when it is not told, relative to the repository root.
const GOLDEN_DIR: &str = "crates/pa-app/tests/golden";

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("weigh") | None => weigh(),
        Some("golden") => {
            let dir = args
                .next()
                .map_or_else(|| PathBuf::from(GOLDEN_DIR), PathBuf::from);
            if let Err(e) = write_golden(&dir) {
                eprintln!("pa-cli golden: {e}");
                std::process::exit(1);
            }
        }
        Some(other) => {
            eprintln!("pa-cli: unknown command {other:?}. Try: weigh, golden [DIR]");
            std::process::exit(2);
        }
    }
}

/// Writes one golden sheet per fixture into `dir`.
///
/// The directory must exist already. The default is relative to the
/// repository root, so run from anywhere else it names nothing, and that is
/// said in those words rather than as a failed write.
fn write_golden(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!(
            "{} is not a directory; run from the repository root, or name the directory",
            dir.display()
        ));
    }
    for fixture in &golden::FIXTURES {
        let path = dir.join(fixture.file_name());
        std::fs::write(&path, golden::sheet(fixture))
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

/// Weighs the standard bench key, its mass its volume model's, then
/// re-weighed with its front weight a gram heavier, and prints both.
fn weigh() {
    let mut bench = KeyBench::standard();
    let before = bench.sheet();

    println!("  KEY WEIGHING SHEET");
    println!("  single key on its frame, stack removed");
    println!("  protocol: Stanwood, Standard Protocols of the New Touchweight");
    // The bench's own setting, not a number written here: a header that
    // quoted the default would go on saying 13 mm after the bench moved.
    println!(
        "            Metrology, February 2000. Measuring point {:.1} mm back",
        bench.measuring_point_from_lip() * 1000.0
    );
    println!("            from the front lip of the key, the edge of the covering.");
    println!("  Read as gram weights give them: down weight and key friction weight");
    println!("  rounded up to the gram, up weight down, balance and friction weight");
    println!("  from those two; each value in full beside it, with its procedure,");
    println!("  its trust and the static solves it took.");
    println!();
    print_sheet("the key as its volume model gives it", &bench, &before);

    // The key re-weighed as a technician weighs one, whole: the same mass on
    // the scale, its front weight read a gram heavier where the sheet reads
    // it.
    let mass_g = bench.readout().map(|r| r.key_mass_g);
    let front_g = match before.get(ProcedureId::FrontWeight) {
        Some(Ok(m)) => Some(m.value),
        _ => None,
    };
    let (Some(mass_g), Some(front_g)) = (mass_g, front_g) else {
        println!();
        println!("  the key cannot be re-weighed: its mass or its front weight is not read.");
        return;
    };
    let from_lip = bench.measuring_point_from_lip();
    if let Err(why) = bench.weigh_key(mass_g, front_g + 1.0, from_lip) {
        println!();
        println!(
            "  the key's weighing is refused: {}",
            bench.why_not_weighed(&why)
        );
        return;
    }
    let after = bench.sheet();
    println!();
    print_sheet(
        "re-weighed: the same mass, its front weight 1 g heavier",
        &bench,
        &after,
    );

    let moved = |p: ProcedureId| match (before.get(p), after.get(p)) {
        (Some(Ok(a)), Some(Ok(b))) => format!("{:+7.2} g", shown(b.value - a.value)),
        _ => "refused".to_owned(),
    };
    println!();
    println!(
        "  1 g more FW        BW {}      FW {}      F {}",
        moved(ProcedureId::BalanceWeight),
        moved(ProcedureId::FrontWeight),
        moved(ProcedureId::FrictionWeight)
    );
    println!("                     front weight changes balance, not friction: F moves");
    println!("                     only by what the key's tilt between its two seats");
    println!("                     does to the arms, under the gram it is read to.");
}

/// What the trade calls each reading, as the sheet prints it.
fn label(p: ProcedureId) -> &'static str {
    match p {
        ProcedureId::FrontWeight => "front weight     FW",
        ProcedureId::KeyWeightRatio => "key weight ratio KR",
        ProcedureId::KeyHeight => "key height",
        ProcedureId::KeyDip => "key dip",
        ProcedureId::DownWeight => "down weight       D",
        ProcedureId::UpWeight => "up weight         U",
        ProcedureId::BalanceWeight => "balance weight   BW",
        ProcedureId::FrictionWeight => "friction weight   F",
        ProcedureId::KeyFrictionWeightZeroedOnTheJig => "key friction KF, jig",
        ProcedureId::KeyFrictionWeightZeroedOnTheFrame => "key friction KF, frame",
    }
}

fn print_sheet(title: &str, bench: &KeyBench, sheet: &Sheet) {
    println!("  {title}");
    println!("  {}", "-".repeat(title.len().max(40)));
    for reading in &sheet.readings {
        let name = label(reading.procedure);
        match &reading.result {
            Ok(m) => {
                let (scale, unit, decimals) = match m.unit {
                    Unit::Metre => (1000.0, "mm", 2),
                    Unit::Gram => (1.0, "g", decimals_of(m.resolution)),
                    Unit::One => (1.0, "", decimals_of(m.resolution)),
                };
                let trust = match &m.trust {
                    Trust::Metrological => "metrological".to_owned(),
                    Trust::Indicative { because } => format!(
                        "indicative: {}",
                        because
                            .iter()
                            .map(|i| i.code())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
                println!(
                    "  {name:<22}{:>8.decimals$} {unit:<2} {:>10.4}   {}; {trust}; {} increments, \
                     {} steps",
                    shown(m.shown * scale),
                    m.value * scale,
                    m.procedure.code(),
                    m.report.increments,
                    m.report.iterations
                );
                if let Some(t) = m.conditions.temporary_weight {
                    println!(
                        "  {:<22}temporary weight {:.2} g at {:.0} mm {} the pin",
                        "",
                        t.mass_g,
                        t.station.x.abs() * 1000.0,
                        if t.station.x < 0.0 {
                            "behind"
                        } else {
                            "in front of"
                        }
                    );
                }
            }
            Err(why) => println!("  {name:<22}refused: {}", bench.why_refused(why)),
        }
    }
    if let Some(rides) = bench.why_it_rides() {
        println!("  {rides}");
    }
    match bench.readout() {
        Some(r) => match bench.key_mass() {
            KeyMass::Estimated => println!(
                "  {:<22}{:>8.2} g    derived from its volume at an assumed density",
                "key mass", r.key_mass_g
            ),
            KeyMass::Weighed { datum, .. } => println!(
                "  {:<22}{:>8.2} g    weighed whole, its front weight {:.2} g read {:.1} mm \
                 back from the lip",
                "key mass",
                r.key_mass_g,
                datum.grams,
                datum.from_lip_m * 1000.0
            ),
        },
        None => println!(
            "  the key cannot be seated: {}",
            bench.frame_fault().unwrap_or_default()
        ),
    }
}

/// How many decimals a resolution is read to: none for the gram, one for a
/// tenth or a half, two for a hundredth.
fn decimals_of(resolution: f64) -> usize {
    if resolution >= 1.0 {
        0
    } else if resolution >= 0.1 {
        1
    } else {
        2
    }
}

/// A difference as the sheet prints it, to the hundredth of a gram.
///
/// Rounded before it is printed, so that a residue of the arithmetic far below
/// the last digit does not print as `-0.00`.
fn shown(x: f64) -> f64 {
    let r = (x * 100.0).round() / 100.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}
