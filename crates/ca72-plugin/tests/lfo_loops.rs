//! The LFO LOOP presets (decisions.md R-LFO's addendum: the LFOs moving each other's knobs and
//! their own, in loops), factory presets since 0.3.0 (R-PRESETS-0.3.0), played through the
//! plug-in's own `process` as a host renders offline (`common::host`). Each moves an LFO's own
//! knob, and sounds: finite, not silent, not clipped.
//!
//! By hand, their sound for listening: `CA72_LFO_LOOPS_OUT=<dir> cargo test --release -p
//! ca72-plugin --test lfo_loops -- --ignored --nocapture` writes each one's eight seconds at HI
//! (`<dir>/<preset>.wav`, 48 kHz, 24 bits) at a safe level, its peak at -14 dBFS (the gain
//! printed with the render's own peak and RMS).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::library::{Sound, factory};
use common::host::{Play, Quality, peak, render, rms, write_wav};
use std::path::PathBuf;

/// The WAVs' peak, dBFS: well under full scale, whatever the patch does.
const PEAK_DBFS: f64 = -14.0;

/// The LFO LOOP presets, as the factory set has them.
fn presets() -> Vec<Sound> {
    let mut out: Vec<Sound> = factory()
        .iter()
        .filter(|s| s.name.starts_with("LFO LOOP "))
        .cloned()
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(out.len(), 4, "the four LFO LOOP presets");
    out
}

fn value(s: &Sound, id: &str) -> Option<f64> {
    s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v)
}

/// What it plays for `seconds` at `quality`: a chord held where POLY is on, else four notes a
/// second and a half apart.
fn play(s: &Sound, seconds: f64, quality: Quality) -> Play {
    let notes = if value(s, "poly") == Some(1.0) {
        vec![(0.25, seconds - 0.85, vec![48u8, 55, 60, 64])]
    } else {
        [45u8, 48, 43, 45]
            .into_iter()
            .enumerate()
            .map(|(n, k)| {
                let t = 0.25 + 1.5 * n as f64;
                (t, (t + 1.3).min(seconds - 0.6) - t, vec![k])
            })
            .filter(|n| n.1 > 0.0)
            .collect()
    };
    let mut p = Play::new(notes, quality);
    p.seconds = seconds;
    p
}

/// Each LFO LOOP preset has an LFO's own knob moved by a source, and sounds (LO, two seconds): finite, not silent, not clipped.
#[test]
fn the_lfo_loop_presets_read_loop_and_sound() {
    for s in presets() {
        assert!(
            s.values
                .iter()
                .any(|(k, v)| k.contains("_to_lfo") && *v != 0.0),
            "{}: no LFO's knob moved",
            s.name
        );
        let (l, r) = render(&s, &play(&s, 2.0, Quality::Lo));
        assert!(l.iter().chain(&r).all(|x| x.is_finite()), "{}", s.name);
        let (p, m) = (peak(&l, &r), rms(&l, &r));
        assert!(m > 1e-3 && p < 1.0, "{}: peak {p}, RMS {m}", s.name);
    }
}

/// Their sound for listening (`CA72_LFO_LOOPS_OUT`), eight seconds each at HI, at a safe level.
#[test]
#[ignore = "writes WAVs for listening"]
fn lfo_loop_wavs() {
    let Some(out) = std::env::var_os("CA72_LFO_LOOPS_OUT").map(PathBuf::from) else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    for s in presets() {
        let (l, r) = render(&s, &play(&s, 8.0, Quality::Hi));
        let (p, m) = (peak(&l, &r), rms(&l, &r));
        let gain = 10f64.powf(PEAK_DBFS / 20.0) / p.max(1e-9);
        println!(
            "{}: peak {:.1} dBFS, RMS {:.1} dBFS; written at {:+.1} dB",
            s.name,
            20.0 * p.log10(),
            20.0 * m.log10(),
            20.0 * gain.log10()
        );
        write_wav(&out.join(format!("{}.wav", s.name)), &l, &r, gain);
    }
}
