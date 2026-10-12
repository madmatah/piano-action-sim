//! Command-line adapter of `pa-app`.
//!
//! The same code that runs in the browser runs here natively. Its one command,
//! `weigh`, prints the key weighing sheet, so that the numbers can be read and
//! checked without a browser in the way.

use pa_app::key_bench::{BenchReadout, KeyBench};

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("weigh") | None => weigh(),
        Some(other) => {
            eprintln!("pa-cli: unknown command {other:?}. Try: weigh");
            std::process::exit(2);
        }
    }
}

/// Weighs the standard bench key, bare and then leaded, and prints both.
fn weigh() {
    let mut bench = KeyBench::standard();
    let bare = bench.readout().expect("the standard bench weighs");

    println!("  KEY WEIGHING SHEET");
    println!("  single key on its frame, stack removed");
    println!("  protocol: Stanwood, Standard Protocols of the New Touchweight");
    println!("            Metrology, February 2000. Measuring point 13 mm back");
    println!("            from the front lip.");
    println!();
    print_sheet("bare key, no leads", &bare);

    // The leading workshop, as a technician would use it: put weight in and
    // watch the touch come down.
    bench.add_lead(0.150, 10.0);
    bench.add_lead(0.190, 10.0);
    let leaded = bench.readout().expect("a leaded key weighs");
    println!();
    print_sheet("with 2 x 10 g at 150 and 190 mm", &leaded);

    println!();
    println!(
        "  lead effect        BW {:+7.2} g      FW {:+7.2} g      F {:+7.2} g",
        leaded.balance_weight_g - bare.balance_weight_g,
        leaded.front_weight_g - bare.front_weight_g,
        leaded.friction_weight_g - bare.friction_weight_g
    );
    println!("                     friction must not move: leads change balance,");
    println!("                     not friction.");
}

fn print_sheet(title: &str, r: &BenchReadout) {
    println!("  {title}");
    println!("  {}", "-".repeat(title.len().max(40)));
    row("down weight       D", r.down_weight_g, "g");
    row("up weight         U", r.up_weight_g, "g");
    row("balance weight   BW", r.balance_weight_g, "g");
    row("friction weight   F", r.friction_weight_g, "g");
    row("front weight     FW", r.front_weight_g, "g");
    row("key weight ratio KR", r.key_ratio, "");
    row("key height", r.key_height_mm, "mm");
    row("key dip", r.key_dip_mm, "mm");
    row("key mass", r.key_mass_g, "g");
    println!("  {:<22}{:>8}", "leads fitted", r.lead_count);
}

fn row(label: &str, value: f64, unit: &str) {
    println!("  {label:<22}{value:>8.2} {unit}");
}
