//! The circuit's voice measured for Potato mode's own model (`measure`): every law from a
//! panel control to pitch, level, cutoff, resonance and time, on `Voice` at 48 kHz in
//! Potato, printed as a report and, with `CA72_REFERENCE_JSON=<file>`, written as JSON for
//! the new model's calibration and comparison.
//!
//! By hand: `cargo test --release -p ca72-plugin --test potato_reference -- --ignored
//! --nocapture` (a few minutes).

#![allow(clippy::unwrap_used)]

mod measure;

use ca72::voice::Voice;
use measure::{Bench, measure_all, report};

#[test]
#[ignore = "a measurement, run by hand"]
fn the_circuits_laws() {
    let bench = Bench::new(|| Voice::prototype(48_000.0));
    let r = measure_all(&bench, &|v: &Voice| v.overload());
    eprintln!("{}", report(&r));
    if let Ok(path) = std::env::var("CA72_REFERENCE_JSON") {
        std::fs::write(&path, serde_json::to_string_pretty(&r).unwrap()).unwrap();
        eprintln!("written to {path}");
    }
}

/// The plug-in's own on the circuit's voice (`measure::more`): DRIVE, FILTER MODE HI and
/// FEEDBACK; with `CA72_REFERENCE_MORE_JSON=<file>`, written as JSON.
#[test]
#[ignore = "a measurement, run by hand"]
fn the_circuits_drive_mode_and_feedback() {
    let bench = Bench::new(|| Voice::prototype(48_000.0));
    let r = measure::more(&bench);
    if let Ok(path) = std::env::var("CA72_REFERENCE_MORE_JSON") {
        std::fs::write(&path, serde_json::to_string_pretty(&r).unwrap()).unwrap();
        eprintln!("written to {path}");
    }
}
