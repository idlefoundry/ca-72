//! Oscillators 2 and 3's FREQUENCY pots measured on the voice (decisions.md R-SEMITONES): how
//! many semitones each pot position puts the oscillator from its centre, against
//! `freq_dial`'s curve, so that every mark of the dial read in semitones is its number.

use ca72::freq_dial::{DIAL_END, Law, Pot};
use ca72::voice::{Panel, Voice};

const SR: f64 = 48_000.0;

/// Oscillator `n`'s frequency over `seconds`, from its resets, after `settle` seconds.
fn frequency(v: &mut Voice, n: usize, settle: f64, seconds: f64) -> f64 {
    for _ in 0..(settle * SR) as usize {
        v.tick();
    }
    let a = v.probe_resets(n);
    for _ in 0..(seconds * SR) as usize {
        v.tick();
    }
    let b = v.probe_resets(n);
    (b.1 - a.1) as f64 / (b.0 - a.0) * SR
}

/// A voice playing oscillator `n` alone (OSC. 3 CONTROL on), MIDI note `note` held.
fn alone(n: usize, note: i32) -> Voice {
    let mut panel = Panel::default();
    for (k, o) in panel.osc.iter_mut().enumerate() {
        o.on = k == n;
    }
    panel.osc3_control = true;
    let mut v = Voice::new(SR, panel);
    v.note(note, true);
    v
}

/// Oscillator `n`'s semitones from its centre at each pot position of `wipers`, note `note`.
fn semitones(n: usize, note: i32, wipers: &[f64]) -> Vec<f64> {
    let mut v = alone(n, note);
    v.panel.osc[n].freq = 0.5;
    let f0 = frequency(&mut v, n, 0.1, 0.5);
    wipers
        .iter()
        .map(|&w| {
            v.panel.osc[n].freq = w;
            let f = frequency(&mut v, n, 0.05, 0.5);
            12.0 * (f / f0).log2()
        })
        .collect()
}

/// Each mark of the dial in semitones (the default law), the voice measured from its resets:
/// within 0.1 cent of its number on A3, where the curve was measured, and 0.25 cent on A2 and
/// A5 (the circuit's own: on A5 its upper marks widen, +7 by 0.18 cent).
#[test]
fn every_mark_is_its_number_of_semitones() {
    let marks = [-7.5, -7.0, -5.0, -3.0, -1.0, 1.0, 3.0, 5.0, 7.0, 7.5];
    for (n, pot) in [(1, Pot::Two), (2, Pot::Three)] {
        for note in [45, 57, 81] {
            let wipers: Vec<f64> = marks
                .iter()
                .map(|m| pot.wiper_at(Law::Semitones, (m + DIAL_END) / (2.0 * DIAL_END)))
                .collect();
            let got = semitones(n, note, &wipers);
            for (m, s) in marks.iter().zip(&got) {
                let cents = (s - m) * 100.0;
                eprintln!(
                    "oscillator {} note {note} mark {m:+}: {cents:+.3} cents",
                    n + 1
                );
                let within = if note == 57 { 0.1 } else { 0.25 };
                assert!(
                    cents.abs() < within,
                    "oscillator {} note {note}: mark {m} at {s} semitones",
                    n + 1
                );
            }
        }
    }
}

#[test]
#[ignore = "prints the pots' curves: cargo test --release -p ca72 --test freq_dial -- --ignored --nocapture"]
fn print_the_curves() {
    let wipers: Vec<f64> = (0..=60).map(|k| k as f64 / 60.0).collect();
    for n in [1, 2] {
        for note in [45, 57, 69] {
            let st = semitones(n, note, &wipers);
            eprintln!("oscillator {} note {note}:", n + 1);
            for (w, s) in wipers.iter().zip(&st) {
                eprintln!("  {w:.6} {s:+.5}");
            }
        }
    }
}
