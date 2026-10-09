//! The VCA's gain against the loudness contour as the hardware reference's (docs/calibration,
//! change 31: R43 at 180K). Session F released a held 1 kHz tone over seconds (LOUDNESS
//! DECAY at its 10 s mark and fully clockwise) while the LOUD CONT jack was recorded; the
//! output's level against the jack's voltage, less its 0.18 V offset (change 26), is the
//! VCA's law from SUSTAIN 10 to 48 dB under it.

use ca72::vca::{Vca, VcaCircuit};
use std::f64::consts::PI;

/// LOUD CONT's reading (V) and the reference's output there against SUSTAIN 10's (4.611 V
/// at the jack), dB.
const REFERENCE: [(f64, f64); 13] = [
    (4.0, -0.4),
    (3.0, -3.2),
    (2.5, -5.0),
    (2.0, -7.3),
    (1.5, -10.6),
    (1.0, -15.9),
    (0.8, -19.4),
    (0.7, -21.9),
    (0.6, -25.2),
    (0.5, -30.2),
    (0.45, -34.0),
    (0.4, -39.6),
    (0.35, -48.3),
];

/// The jack's reading over the contour the VCA sees (change 26).
const JACK_OFFSET: f64 = 0.18;

fn gain(c: VcaCircuit, cont: f64) -> f64 {
    let sr = 48e3;
    let mut v = Vca::new(c, sr, cont);
    v.settle(cont);
    let (n0, n1) = ((0.3 * sr) as usize, (0.2 * sr) as usize);
    let (mut re, mut im) = (0.0, 0.0);
    for i in 0..n0 + n1 {
        let p = 2.0 * PI * 1000.0 * i as f64 / sr;
        let o = v.tick(0.1 * p.sin(), cont);
        if i >= n0 {
            re += o * p.cos();
            im += o * p.sin();
        }
    }
    re.hypot(im)
}

#[test]
fn the_vca_follows_the_references_law() {
    let c = VcaCircuit::default().calibrated();
    assert_eq!(c.r43, 180e3);
    let g0 = gain(c, 4.611 - JACK_OFFSET);
    let mut report = String::new();
    let mut fail = false;
    for (reading, db) in REFERENCE {
        let got = 20.0 * (gain(c, reading - JACK_OFFSET) / g0).log10();
        // A tenth of a volt moves the gain 5 to 9 dB near the bottom.
        let budget = if db > -20.0 {
            0.8
        } else if db > -35.0 {
            1.5
        } else {
            3.0
        };
        fail |= (got - db).abs() > budget;
        report += &format!("LOUD CONT {reading} V: {got:.1} dB, the reference's {db:.1}\n");
    }
    eprintln!("{report}");
    assert!(
        !fail,
        "budget: 0.8 dB to -20, 1.5 to -35, 3 below\n{report}"
    );
    // As drawn (270K) the VCA shut 20 dB sooner at the bottom.
    let drawn = VcaCircuit {
        r43: 270e3,
        ..VcaCircuit::default()
    }
    .calibrated();
    let g = gain(drawn, 0.45 - JACK_OFFSET) / gain(drawn, 4.611 - JACK_OFFSET);
    assert!(
        20.0 * g.log10() < -45.0,
        "R43 270K at 0.45 V: {:.1} dB",
        20.0 * g.log10()
    );
}
