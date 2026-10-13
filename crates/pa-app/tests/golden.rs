//! The golden sheet: the engine's readings against the ones recorded in
//! `tests/golden/`, bit for bit.
//!
//! A consistency check of the engine against its own past, not a validation:
//! it says that a number has not moved, never that it is right. What it
//! catches is a change that moves a number nobody meant to move. A change that
//! means to move one rewrites the sheets with `cargo run -p pa-cli -- golden`
//! and says in its commit which readings moved and why.

use std::fs;
use std::path::{Path, PathBuf};

use pa_app::golden::{sheet, FIXTURES};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// The lines that differ, as `name: recorded -> computed`, each name once.
fn differences(recorded: &str, computed: &str) -> Vec<String> {
    let split = |text: &str| -> Vec<(String, String)> {
        text.lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| match l.split_once(' ') {
                Some((name, value)) => (name.to_owned(), value.to_owned()),
                None => (l.to_owned(), String::new()),
            })
            .collect()
    };
    let (old, new) = (split(recorded), split(computed));
    let mut out = Vec::new();
    for (name, value) in &new {
        match old.iter().find(|(n, _)| n == name) {
            Some((_, was)) if was == value => {}
            Some((_, was)) => out.push(format!("{name}: {was} -> {value}")),
            None => out.push(format!("{name}: not recorded -> {value}")),
        }
    }
    for (name, was) in &old {
        if !new.iter().any(|(n, _)| n == name) {
            out.push(format!("{name}: {was} -> no longer read"));
        }
    }
    if out.is_empty() && recorded != computed {
        out.push("the readings agree, the header or the order of the lines does not".to_owned());
    }
    out
}

/// Every reading of every fixture bench is the one the golden sheet records.
#[test]
fn every_reading_is_the_one_the_golden_sheet_records() {
    let mut report = Vec::new();
    for fixture in &FIXTURES {
        let path = golden_dir().join(fixture.file_name());
        let recorded = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let computed = sheet(fixture);
        if recorded != computed {
            report.push(format!("{}:", fixture.name));
            report.extend(
                differences(&recorded, &computed)
                    .into_iter()
                    .map(|d| format!("  {d}")),
            );
        }
    }
    assert!(
        report.is_empty(),
        "readings moved against the golden sheet:\n{}\n\nIf the change means to move \
         them, rewrite the sheets from the repository root with \
         `cargo run -p pa-cli -- golden` and say in the commit which moved and why.",
        report.join("\n")
    );
}

/// The directory holds a sheet for every fixture and nothing else, so a
/// fixture renamed or dropped cannot leave a stale sheet behind that no test
/// reads.
#[test]
fn the_golden_directory_holds_exactly_the_fixtures() {
    let mut on_disk: Vec<String> = fs::read_dir(golden_dir())
        .expect("tests/golden exists")
        .map(|entry| {
            entry
                .expect("a readable entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    on_disk.sort();
    let mut expected: Vec<String> = FIXTURES.iter().map(|f| f.file_name()).collect();
    expected.sort();
    assert_eq!(on_disk, expected);
}
