//! The VCA's balance trims as the hardware reference's (docs/calibration, change 32): the 2nd
//! harmonic the VCA adds to a sine at the filter's resonance, its size and its sign, as the
//! reference's added it in session O (takes 05, 08 and 09 less take 10, where the VCA adds
//! almost nothing), and the thump at SUSTAIN 10 (take 01, before its tone).

use ca72::vca::{R12_REFERENCE, R14_REFERENCE, Vca, VcaCircuit};
use std::f64::consts::PI;

/// Contour (V, the CA-72's at LOUDNESS SUSTAIN 10, 7 and 5), the sine's amplitude at the
/// VCA's input (V: the CA-72 filter's output for 0.01 and 0.1 of the ES-3's full scale), and
/// the reference's 2nd harmonic: dBc, and its phase against twice the fundamental's.
const REFERENCE: [(f64, f64, f64, f64); 6] = [
    (4.455, 0.438, -63.2, 178.0),
    (4.455, 0.81, -56.2, 177.0),
    (2.779, 0.438, -65.6, -176.0),
    (2.779, 0.81, -59.7, 178.0),
    (1.676, 0.438, -72.9, 174.0),
    (1.676, 0.81, -67.0, 173.0),
];

/// The reference's thump at SUSTAIN 10 against its tone for 0.01 of full scale (C3, 0.34 V
/// at the VCA's input), 8, 12, 20, 40 and 100 ms after the gate, less what the voice's
/// attack adds over the plain step of contour here (the factory trims' voice less their
/// step: 0.026, 0.021, 0.014, -0.002, -0.006).
const THUMP: [(f64, f64); 5] = [
    (8.0, 0.137),
    (12.0, 0.163),
    (20.0, 0.197),
    (40.0, 0.212),
    (100.0, 0.111),
];

/// The 2nd harmonic of the VCA's output for a sine at `hz`: dBc and degrees.
fn second(c: VcaCircuit, cont: f64, amp: f64, hz: f64) -> (f64, f64) {
    let sr = 48e3;
    let mut v = Vca::new(c, sr, cont);
    v.settle(cont);
    let (n0, n1) = ((0.5 * sr) as usize, (0.4 * sr) as usize);
    let y: Vec<f64> = (0..n0 + n1)
        .map(|i| v.tick(amp * (2.0 * PI * hz * i as f64 / sr).sin(), cont))
        .skip(n0)
        .collect();
    let bin = |h: f64| {
        let (mut re, mut im) = (0.0, 0.0);
        for (i, x) in y.iter().enumerate() {
            let w = 0.5 - 0.5 * (2.0 * PI * i as f64 / n1 as f64).cos();
            let p = 2.0 * PI * h * hz * i as f64 / sr;
            re += x * w * p.cos();
            im -= x * w * p.sin();
        }
        (re.hypot(im), im.atan2(re))
    };
    let (a1, p1) = bin(1.0);
    let (a2, p2) = bin(2.0);
    let ph = (p2 - 2.0 * p1 + PI).rem_euclid(2.0 * PI) - PI;
    (20.0 * (a2 / a1).log10(), ph.to_degrees())
}

/// The VCA's output for a C3 of `amp` V at the contour `cont`: its amplitude.
fn tone(c: VcaCircuit, cont: f64, amp: f64) -> f64 {
    let sr = 48e3;
    let hz = 130.81;
    let mut v = Vca::new(c, sr, cont);
    v.settle(cont);
    let (n0, n1) = ((0.5 * sr) as usize, (0.5 * sr) as usize);
    let (mut re, mut im, mut ws) = (0.0, 0.0, 0.0);
    for i in 0..n0 + n1 {
        let p = 2.0 * PI * hz * i as f64 / sr;
        let o = v.tick(amp * p.sin(), cont);
        if i >= n0 {
            let w = 0.5 - 0.5 * (2.0 * PI * (i - n0) as f64 / n1 as f64).cos();
            re += o * w * p.cos();
            im += o * w * p.sin();
            ws += w;
        }
    }
    2.0 * re.hypot(im) / ws
}

/// The output after the contour steps from rest to `cont`, through the interface's 0.94 Hz
/// high-pass, against [`tone`] for 0.34 V: at each of `ms` after the step.
fn thump(c: VcaCircuit, cont: f64, ms: &[f64]) -> Vec<f64> {
    let sr = 48e3;
    let rest = -0.5;
    let k = 1.0 / (1.0 + 2.0 * PI * 0.94 / sr);
    let g = tone(c, cont, 0.34);
    let mut v = Vca::new(c, sr, rest);
    v.settle(rest);
    let (mut x1, mut y1) = (0.0, 0.0);
    let step = (0.2 * sr) as usize;
    let mut out = Vec::new();
    for i in 0..step + (0.15 * sr) as usize {
        let x = v.tick(0.0, if i < step { rest } else { cont });
        let y = k * (y1 + x - x1);
        (x1, y1) = (x, y);
        out.push(y);
    }
    let before = out[step - (0.01 * sr) as usize];
    ms.iter()
        .map(|m| (out[step + (m * 1e-3 * sr) as usize] - before) / g)
        .collect()
}

#[test]
fn the_vca_adds_the_references_second_harmonic() {
    let c = VcaCircuit::default().reference_trims();
    assert_eq!((c.r14_pos, c.r12_pos), (R14_REFERENCE, R12_REFERENCE));
    let mut report = String::new();
    let mut fail = false;
    for (cont, amp, db, deg) in REFERENCE {
        let (got_db, got_deg) = second(c, cont, amp, 995.0);
        let turn = ((got_deg - deg + 180.0).rem_euclid(360.0) - 180.0).abs();
        // The trims are a compromise with the thump: at full SUSTAIN up to 8 dB weak (fitted
        // to the 2nd harmonic alone, R14 0.580 and R12 0.508, within 2 dB, and the thump 0.08
        // of the tone too big from its start).
        fail |= !((-9.0..=3.0).contains(&(got_db - db)) && turn <= 20.0);
        report += &format!(
            "contour {cont} V, {amp} V in: {got_db:.1} dBc at {got_deg:.0} deg, the reference's \
             {db:.1} at {deg:.0}\n"
        );
    }
    eprintln!("{report}");
    assert!(!fail, "budget: 9 dB under to 3 over, 20 degrees\n{report}");
    // The factory procedure's trims leave it about 20 dB stronger, in the filter's phase.
    let factory = VcaCircuit::default().calibrated();
    let (db, deg) = second(factory, 4.455, 0.81, 995.0);
    assert!(
        db > -40.0 && deg.abs() < 20.0,
        "factory trims: {db:.1} dBc at {deg:.0} deg"
    );
}

#[test]
fn the_vca_thumps_as_the_reference() {
    let c = VcaCircuit::default().reference_trims();
    let ms: Vec<f64> = THUMP.iter().map(|t| t.0).collect();
    let got = thump(c, 4.455, &ms);
    let report: String = THUMP
        .iter()
        .zip(&got)
        .map(|((m, r), g)| format!("{m} ms: {g:.3} of the tone, the reference's {r:.3}\n"))
        .collect();
    eprintln!("{report}");
    assert!(
        THUMP
            .iter()
            .zip(&got)
            .all(|((_, r), g)| (g - r).abs() <= 0.04),
        "budget: 0.04 of the tone\n{report}"
    );
}
