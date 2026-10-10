//! Potato mode's model measured as the circuit's is (`measure`; `tests/potato_reference.rs`):
//! every law from a panel control to pitch, level, cutoff, resonance and time, on
//! `ca72::light::Light` at 48 kHz, printed as a report and, with `CA72_LIGHT_JSON=<file>`,
//! written as JSON to compare with the circuit's (decisions.md R31).
//!
//! By hand: `cargo test --release -p ca72-plugin --test potato_match -- --ignored
//! --nocapture`.

#![allow(clippy::unwrap_used)]

mod measure;

use ca72::light::Light;
use ca72::voice::Panel;
use measure::{Bench, Model, measure_all, report};

impl Model for Light {
    fn rate(&self) -> f64 {
        Light::rate(self)
    }

    fn set_panel(&mut self, p: &Panel) {
        Light::set_panel(self, p);
    }

    fn note(&mut self, midi: i32, on: bool) {
        Light::note(self, midi, on);
    }

    fn tick(&mut self, ext: f64) -> f64 {
        Light::tick(self, ext)
    }

    fn contours(&self) -> (f64, f64) {
        Light::contours(self)
    }

    fn set_drive(&mut self, gain: f64) {
        Light::set_drive(self, gain);
    }

    fn set_feedback(&mut self, share: f64) {
        self.feedback = share;
    }
}

#[test]
#[ignore = "a measurement, run by hand"]
fn the_light_models_laws() {
    let bench = Bench::new(|| Light::new(48_000.0));
    let r = measure_all(&bench, &|l: &Light| l.overload());
    eprintln!("{}", report(&r));
    if let Ok(path) = std::env::var("CA72_LIGHT_JSON") {
        std::fs::write(&path, serde_json::to_string_pretty(&r).unwrap()).unwrap();
        eprintln!("written to {path}");
    }
}

/// The plug-in's own on Potato mode's voice (`measure::more`): DRIVE, FILTER MODE HI and
/// FEEDBACK; with `CA72_LIGHT_MORE_JSON=<file>`, written as JSON.
#[test]
#[ignore = "a measurement, run by hand"]
fn the_light_models_drive_mode_and_feedback() {
    let bench = Bench::new(|| Light::new(48_000.0));
    let r = measure::more(&bench);
    if let Ok(path) = std::env::var("CA72_LIGHT_MORE_JSON") {
        std::fs::write(&path, serde_json::to_string_pretty(&r).unwrap()).unwrap();
        eprintln!("written to {path}");
    }
}
