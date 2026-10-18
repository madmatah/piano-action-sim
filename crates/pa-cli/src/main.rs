//! Command-line adapter of `pa-app`.
//!
//! The same instrument that runs in the browser runs here natively, changed
//! by the same calls. Three commands:
//! `weigh [PROFILE]` prints the key weighing sheet of the standard
//! instrument, or of the shipped profile named, so that the numbers can be
//! read and checked without a browser in the way; `band [PROFILE]` prints the
//! band the driven sweep draws, its down and up curves and their extremes in
//! full beside the walk's down and up weights; `golden` writes the golden sheet,
//! every reading of the default bench and of the fixture benches in full bits,
//! which `crates/pa-app/tests/golden.rs` compares the engine against.
//!
//! A refusal prints as the code the browser words, with its parameters
//! (`pa_app::boundary`): the engine builds no sentence, and this tool keeps
//! no second wording of its own.

use std::path::{Path, PathBuf};

use pa_app::boundary::{self, json, Acted, Refusal, Riding};
use pa_app::golden;
use pa_app::instrument::{Action, Instrument, KeyMass, ProcedureId, Sheet, Trust, Unit};
use pa_app::profile::Profile;

/// Where `golden` writes when it is not told, relative to the repository root.
const GOLDEN_DIR: &str = "crates/pa-app/tests/golden";

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("weigh") | None => weigh(args.next().as_deref()),
        Some("band") => band(args.next().as_deref()),
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
            eprintln!(
                "pa-cli: unknown command {other:?}. Try: weigh [PROFILE], band [PROFILE], \
                 golden [DIR]"
            );
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

/// The standard instrument, or the shipped profile named, as it arrives;
/// the command `command` says it cannot be built otherwise, and exits.
fn instrument(command: &str, profile: Option<&str>) -> Instrument {
    match profile {
        None => Instrument::standard(),
        Some(name) => match Profile::named(name).map(Instrument::of) {
            Some(Ok(bench)) => bench,
            Some(Err(why)) => {
                eprintln!("pa-cli {command}: the profile {name:?} builds no instrument: {why:?}");
                std::process::exit(1);
            }
            None => {
                let names: Vec<&str> = Profile::names().collect();
                eprintln!(
                    "pa-cli {command}: no profile {name:?}. Shipped: {}",
                    names.join(", ")
                );
                std::process::exit(2);
            }
        },
    }
}

/// Draws the band of the standard bench, or of the shipped profile named:
/// the down curve's greatest force and the up curve's least, in full and
/// never as a gram, beside the walk's down and up weights, where each falls
/// on the stroke, the sweep's solves and how long it took, then both curves
/// a reading a line, travel and force, for a plot.
///
/// The band is a curve, read by no weight: the gram is the walk's, the
/// trade's protocol with gram weights. The two stand within a few
/// thousandths of a gram in full, and so may read the gram either side of a
/// whole one.
fn band(profile: Option<&str>) {
    let bench = instrument("band", profile);
    let sheet = bench.sheet();
    let started = std::time::Instant::now();
    let banded = bench.band();
    let took = started.elapsed();
    println!("  THE BAND");
    println!("  the key driven through its stroke at the measuring point, down and back,");
    println!(
        "  the measuring point {:.1} mm back from the front lip of the key;",
        bench.measuring_point_from_lip() * 1000.0
    );
    println!("  the force that drives it there, in grams, at every step.");
    println!();
    println!("  profile: {}", bench.profile().name);
    let banded = match banded {
        Ok(banded) => banded,
        Err(why) => {
            println!(
                "  the band is refused: {}",
                json(&Refusal::reading(&why, bench.riding().map(Riding::from)))
            );
            return;
        }
    };
    let b = &banded.band;
    let walked = |p: ProcedureId| match sheet.get(p) {
        Some(Ok(m)) => format!("{} g from {:.4} g", shown(m.shown), m.value),
        _ => "refused".to_owned(),
    };
    for (m, walk, name) in [
        (&banded.down_weight, ProcedureId::DownWeight, "D"),
        (&banded.up_weight, ProcedureId::UpWeight, "U"),
    ] {
        let at = m.conditions.sweep.map_or(f64::NAN, |s| s.at * 1000.0);
        println!(
            "  {:<22}{:>10.4} g   at {at:6.2} mm of travel; the walk's {name} {}",
            label(m.procedure),
            m.value,
            walked(walk)
        );
    }
    let (d, u) = (banded.down_weight.value, banded.up_weight.value);
    println!("  {:<22}{:>10.4} g   its middle", "", 0.5 * (d + u));
    println!("  {:<22}{:>10.4} g   its width", "", d - u);
    println!(
        "  window {:.3} mm, a reading every {:.2} mm, {} each way; {} static solves, {} steps, \
         in {:.2} ms",
        b.window * 1000.0,
        b.step * 1000.0,
        b.down.len(),
        b.report.increments,
        b.report.iterations,
        took.as_secs_f64() * 1000.0
    );
    println!();
    println!("  stroke  travel_mm  force_g");
    for (stroke, curve) in [("down", &b.down), ("up", &b.up)] {
        for s in curve {
            println!("  {stroke:<6} {:>10.4} {:>8.4}", s.travel * 1000.0, s.grams);
        }
    }
}

/// Weighs the key of the standard bench, or of the shipped profile named,
/// as it arrives, then re-weighed with its front weight a gram heavier, and
/// prints both.
fn weigh(profile: Option<&str>) {
    let mut bench = instrument("weigh", profile);
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
    println!("  profile: {}", bench.profile().name);
    for fault in bench.faults() {
        println!("  fault: {fault:?}");
    }
    println!();
    print_sheet("the key as it arrives", &bench, &before);

    // The key re-weighed as a technician weighs one, whole: the same mass on
    // the scale, its front weight read a gram heavier where the sheet reads
    // it.
    let mass_g = bench.key_mass_g();
    let front_g = match before.get(ProcedureId::FrontWeight) {
        Some(Ok(m)) => Some(m.value),
        _ => None,
    };
    let (Some(mass_g), Some(front_g)) = (mass_g, front_g) else {
        println!();
        println!("  the key cannot be re-weighed: its mass or its front weight is not read.");
        if bench.key_mass_g().is_none() {
            println!("  nobody has weighed it: type its weighing on the bench, or in a profile.");
        }
        return;
    };
    let reweighed = Action::WeighKey {
        mass_g,
        front_weight_g: front_g + 1.0,
        from_lip_mm: bench.measuring_point_from_lip() * 1000.0,
    };
    if let Acted::Refused { why } = boundary::act(&mut bench, reweighed) {
        println!();
        println!("  the key's weighing is refused: {}", json(&why));
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
        ProcedureId::DownWeightOnTheBand => "band, greatest down",
        ProcedureId::UpWeightOnTheBand => "band, least up",
    }
}

fn print_sheet(title: &str, bench: &Instrument, sheet: &Sheet) {
    let riding = bench.riding();
    println!("  {title}");
    println!("  {}", "-".repeat(title.len().max(40)));
    for reading in &sheet.readings {
        let name = label(reading.procedure);
        match &reading.result {
            Ok(m) => {
                let (scale, unit, decimals) = match m.unit {
                    Unit::Meter => (1000.0, "mm", 2),
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
            Err(why) => println!(
                "  {name:<22}refused: {}",
                json(&Refusal::reading(why, riding.map(Riding::from)))
            ),
        }
    }
    if let Some(riding) = riding {
        println!(
            "  the key rides on its stops: {}",
            json(&Riding::from(riding))
        );
    }
    match bench.readout() {
        Some(_) => match (bench.key_mass(), bench.key_mass_g()) {
            (KeyMass::Published { center, .. }, Some(mass_g)) => println!(
                "  {:<22}{:>8.2} g    as published, its center of mass at ({:.1}, {:.1}) mm",
                "key mass",
                mass_g,
                center.x * 1000.0,
                center.y * 1000.0
            ),
            (KeyMass::Weighed { datum, .. }, Some(mass_g)) => println!(
                "  {:<22}{:>8.2} g    weighed whole, its front weight {:.2} g read {:.1} mm \
                 back from the lip",
                "key mass",
                mass_g,
                datum.grams,
                datum.from_lip_m * 1000.0
            ),
            _ => println!(
                "  {:<22}{:>8}      not weighed: {}",
                "key mass",
                "refused",
                json(&Refusal::reading(
                    &pa_app::instrument::Refused::KeyNotWeighed,
                    None
                ))
            ),
        },
        None => println!(
            "  the key cannot be seated: {}",
            bench
                .frame_fault()
                .map_or_else(|| "null".to_owned(), |why| json(&Refusal::seat(&why)))
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
