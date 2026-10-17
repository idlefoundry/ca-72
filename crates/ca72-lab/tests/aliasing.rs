//! The oscillators' aliasing stays where the band-limiting put it (decisions.md R-BLEP):
//! oscillator 1 of the circuit's model at HI's 1x and ULTRA's 4x and the light model's
//! oscillator (LO), each waveform alone at A4 and A7 at 44.1, 48 and 96 kHz, all the power
//! from 20 Hz to 20 kHz that is not a harmonic against the fundamental's (`ca72-lab alias`
//! measures the same, with more notes and its loudest components). The budgets are the
//! measured levels with about 3 dB to spare.

use ca72::voice::Waveform;
use ca72_lab::alias::{N, analyse, circuit, light};

const RATES: [f64; 3] = [44_100.0, 48_000.0, 96_000.0];
/// A4 and A7 on 2'.
const KEYS: [f64; 2] = [4.0, 40.0];

/// The circuit's sawtooth, triangle and three rectangles at an oversampling: the worst total
/// aliasing of the sawtooth and rectangles, and of the triangle, dB, at `rates`.
fn circuit_worst(os: usize, rates: &[f64]) -> (f64, f64) {
    let (mut steps, mut tri) = (f64::MIN, f64::MIN);
    for &rate in rates {
        for key in KEYS {
            for (i, width) in [0.0, -1.5, -2.5].into_iter().enumerate() {
                let (w, f0) = circuit(rate, os, key, width, N);
                if i == 0 {
                    steps = steps.max(analyse(&w[0], rate, f0).total_db);
                    tri = tri.max(analyse(&w[1], rate, f0).total_db);
                }
                steps = steps.max(analyse(&w[2], rate, f0).total_db);
            }
        }
    }
    (steps, tri)
}

#[test]
fn hi_oscillators_alias_below_80_db() {
    let (steps, tri) = circuit_worst(1, &RATES);
    assert!(steps < -80.0, "sawtooth and rectangles: {steps:.1} dB");
    assert!(tri < -58.0, "triangle: {tri:.1} dB");
}

#[test]
fn ultra_oscillators_alias_below_105_db() {
    // (At 44.1 kHz too, by the oscillators' longer last halfband.)
    let (steps, tri) = circuit_worst(4, &RATES);
    assert!(steps < -105.0, "sawtooth and rectangles: {steps:.1} dB");
    assert!(tri < -70.0, "triangle: {tri:.1} dB");
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
