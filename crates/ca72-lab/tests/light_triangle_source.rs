//! LO approximates the physical DC source curve, with an explicit approximation budget.
use ca72::{
    light,
    vco::{self, CoreParams, ShaperParams},
};

#[test]
fn finite_knees_follow_the_ngspice_derived_source_curve() {
    const N: usize = 65536;
    let core = CoreParams::default();
    let sh = ShaperParams::default();
    let high = sh.buf_gain * core.v_top0 + sh.buf_offset;
    let low = sh.buf_gain * core.v_threshold + sh.buf_offset;
    let scale = (high - low) * sh.saw_ratio * 0.5;
    let physical: Vec<f64> = (0..N)
        .map(|i| vco::triangle(high - (high - low) * i as f64 / N as f64))
        .collect();
    let mean = physical.iter().sum::<f64>() / N as f64;
    let mut reference: Vec<f64> = physical.iter().map(|x| (x - mean) / scale).collect();
    let mut approximation: Vec<f64> = (0..N)
        .map(|i| f64::from(light::triangle_naive(i as f32 / N as f32)))
        .collect();
    let rms = (reference
        .iter()
        .zip(&approximation)
        .map(|(r, a)| (r - a).powi(2))
        .sum::<f64>()
        / N as f64)
        .sqrt();
    let max = reference
        .iter()
        .zip(&approximation)
        .map(|(r, a)| (r - a).abs())
        .fold(0., f64::max);
    assert!(rms < 0.0008, "DC source RMS{rms}");
    assert!(max < 0.005, "DC source maximum{max}");
    let mut ri = vec![0.; N];
    let mut ai = vec![0.; N];
    ca72_analysis::fft::fft(&mut reference, &mut ri);
    ca72_analysis::fft::fft(&mut approximation, &mut ai);
    let mut sum = 0.;
    let mut maximum = 0f64;
    for k in 2..=24 {
        let r = reference[k].hypot(ri[k]);
        let a = approximation[k].hypot(ai[k]);
        let d = 20. * (a / r).log10();
        sum += d * d;
        maximum = maximum.max(d.abs());
    }
    let harmonic_rms = (sum / 23.).sqrt();
    eprintln!("LO source RMS{rms:.6},max{max:.6};H2–24RMS{harmonic_rms:.3}dB,max{maximum:.3}dB");
    assert!(harmonic_rms < 0.5, "harmonic source RMS{harmonic_rms}");
    assert!(maximum < 1.2, "harmonic source maximum{maximum}");
}
