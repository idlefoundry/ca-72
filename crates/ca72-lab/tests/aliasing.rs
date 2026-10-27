//! The oscillators' aliasing stays where the band-limiting put it (decisions.md R-BLEP):
//! oscillator 1 of the circuit's model at HI's 1x and ULTRA's 4x and the light model's
//! oscillator (LO), each waveform alone at A4 and A7 at 44.1, 48 and 96 kHz, all the power
//! from 20 Hz to 20 kHz that is not a harmonic against the fundamental's (`ca72-lab alias`
//! measures the same, with more notes and its loudest components). The budgets are the
//! original thresholds, retained for both the production width network and the explicit
//! legacy ideal bench. Per-case margins are printed, rather than assumed unchanged.

use ca72::voice::Waveform;
use ca72_lab::alias::{N, analyse, circuit, circuit_ideal, light};

const RATES: [f64; 3] = [44_100.0, 48_000.0, 96_000.0];
/// A4 and A7 on 2'.
const KEYS: [f64; 2] = [4.0, 40.0];

/// The circuit's sawtooth, triangle and three rectangles at an oversampling: the worst total
/// aliasing of the sawtooth and rectangles, and of the triangle, dB, at `rates`.
fn circuit_worst(os: usize, rates: &[f64], physical_width_source: bool) -> (f64, f64) {
    let circuit = if physical_width_source {
        circuit
    } else {
        circuit_ideal
    };
    let label = if physical_width_source {
        "production"
    } else {
        "legacy-ideal"
    };
    let (mut steps, mut tri) = (f64::MIN, f64::MIN);
    for &rate in rates {
        for key in KEYS {
            for (i, width) in [0.0, -1.5, -2.5].into_iter().enumerate() {
                let (w, f0) = circuit(rate, os, key, width, N);
                if i == 0 {
                    steps = steps.max(analyse(&w[0], rate, f0).total_db);
                    tri = tri.max(analyse(&w[1], rate, f0).total_db);
                }
                let rectangle = analyse(&w[2], rate, f0);
                eprintln!(
                    "{label} os{os} rate{rate} key{key} width{width}: rectangle total {:.2}dB, largest {:.2}dB at {:.2}Hz",
                    rectangle.total_db, rectangle.loudest_db, rectangle.loudest_hz
                );
                steps = steps.max(rectangle.total_db);
            }
        }
    }
    eprintln!("{label} os{os} worst total: sawtooth/rectangles {steps:.3}dB, triangle {tri:.3}dB");
    (steps, tri)
}

#[test]
fn production_and_legacy_alias_benches_do_not_use_the_same_pulse_source() {
    // A validation-routing regression: both benches can meet the alias budget while
    // accidentally rendering the legacy comparator, so distinguish their actual audio.
    let (production, f0) = circuit(48_000.0, 1, 4.0, -1.5, 8192);
    let (legacy, f1) = circuit_ideal(48_000.0, 1, 4.0, -1.5, 8192);
    assert!(
        (f0 / f1 - 1.0).abs() < 1e-6,
        "the source selection changed the carrier"
    );
    assert_ne!(
        production[2], legacy[2],
        "the production alias gate rendered the legacy pulse source"
    );
}

#[test]
fn hi_production_oscillators_alias_below_80_db() {
    let (steps, tri) = circuit_worst(1, &RATES, true);
    assert!(steps < -80.0, "sawtooth and rectangles: {steps:.1} dB");
    assert!(tri < -58.0, "triangle: {tri:.1} dB");
}

#[test]
fn ultra_production_oscillators_alias_below_105_db() {
    // (At 44.1 kHz too, by the oscillators' longer last halfband.)
    let (steps, tri) = circuit_worst(4, &RATES, true);
    assert!(steps < -105.0, "sawtooth and rectangles: {steps:.1} dB");
    assert!(tri < -70.0, "triangle: {tri:.1} dB");
}

#[test]
fn hi_legacy_ideal_bench_aliases_below_80_db() {
    let (steps, tri) = circuit_worst(1, &RATES, false);
    assert!(
        steps < -80.0,
        "legacy ideal sawtooth and rectangles: {steps:.1} dB"
    );
    assert!(tri < -58.0, "legacy ideal triangle: {tri:.1} dB");
}

#[test]
fn ultra_legacy_ideal_bench_aliases_below_105_db() {
    let (steps, tri) = circuit_worst(4, &RATES, false);
    assert!(
        steps < -105.0,
        "legacy ideal sawtooth and rectangles: {steps:.1} dB"
    );
    assert!(tri < -70.0, "legacy ideal triangle: {tri:.1} dB");
}

#[test]
fn lo_oscillators_alias_below_80_db() {
    let mut worst = (f64::MIN, f64::MIN);
    for rate in RATES {
        for hz in [440.0, 3520.0] {
            for wave in [
                Waveform::Sawtooth,
                Waveform::Triangle,
                Waveform::SharkTooth,
                Waveform::Square,
                Waveform::WideRectangle,
                Waveform::NarrowRectangle,
            ] {
                let a = analyse(&light(rate, hz, wave, N), rate, hz).total_db;
                if matches!(wave, Waveform::Triangle | Waveform::SharkTooth) {
                    worst.1 = worst.1.max(a);
                } else {
                    worst.0 = worst.0.max(a);
                }
            }
        }
    }
    assert!(
        worst.0 < -80.0,
        "sawtooth and rectangles: {:.1} dB",
        worst.0
    );
    assert!(
        worst.1 < -95.0,
        "triangle and shark tooth: {:.1} dB",
        worst.1
    );
}
