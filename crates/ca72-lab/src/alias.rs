//! The oscillators' aliasing (decisions.md R-BLEP): one oscillator alone, steady, its
//! spectrum through a seven-term Blackman-Harris window (side lobes about 180 dB down),
//! measured from 20 Hz to 20 kHz against its fundamental. Everything within eight bins of a
//! harmonic (the window's main lobe and a bin) is the harmonic's; the rest is aliasing
//! (the models are deterministic: nothing else is there). Also the harmonics' levels below
//! 20 kHz against a reference's, to show what a kernel does to the top of the sound.

use ca72::light;
use ca72::tuning::{self, Range, Tuning, osc1_inputs};
use ca72::voice::Waveform;
use ca72_analysis::fft::fft;

/// Samples analysed (2^17: 2.7 s at 48 kHz, bins of 0.37 Hz).
pub const N: usize = 1 << 17;
/// Bins either side of a harmonic that belong to it.
const LOBE: usize = 8;

/// What one oscillator's spectrum shows.
#[derive(Debug, Clone)]
pub struct Measure {
    pub f0: f64,
    /// All the power that is not a harmonic, 20 Hz to 20 kHz, against the fundamental's, dB.
    pub total_db: f64,
    /// The loudest component that is not a harmonic, against the fundamental, dB, and where.
    pub loudest_db: f64,
    pub loudest_hz: f64,
    /// The harmonics' powers below 20 kHz against the fundamental's (the first is 1).
    pub harmonics: Vec<f64>,
}

fn window(n: usize) -> Vec<f64> {
    const A: [f64; 7] = [
        0.271_051_400_693_42,
        0.433_297_939_234_48,
        0.218_122_999_543_11,
        0.065_925_446_388_03,
        0.010_811_742_098_37,
        0.000_776_584_825_22,
        0.000_013_887_217_35,
    ];
    (0..n)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            A.iter()
                .enumerate()
                .map(|(k, &a)| if k % 2 == 0 { a } else { -a } * (k as f64 * t).cos())
                .sum()
        })
        .collect()
}

/// The measure of `x` (at least [`N`] samples; the last `N` are taken) at `rate`, its
/// fundamental `f0`.
pub fn analyse(x: &[f64], rate: f64, f0: f64) -> Measure {
    let x = &x[x.len() - N..];
    let w = window(N);
    let mut re: Vec<f64> = x.iter().zip(&w).map(|(a, b)| a * b).collect();
    let mean = re.iter().sum::<f64>() / N as f64;
    // (The window's DC is well away from 20 Hz; taken out anyway.)
    let wmean = w.iter().sum::<f64>() / N as f64;
    for (r, wv) in re.iter_mut().zip(&w) {
        *r -= mean / wmean * wv;
    }
    let mut im = vec![0.0; N];
    fft(&mut re, &mut im);
    let p: Vec<f64> = (0..N / 2).map(|k| re[k] * re[k] + im[k] * im[k]).collect();
    let bin = rate / N as f64;
    let lobe = |c: f64| -> f64 {
        let c = (c / bin).round() as usize;
        p[c.saturating_sub(LOBE)..(c + LOBE + 1).min(p.len())]
            .iter()
            .sum()
    };
    let fund = lobe(f0);
    let harmonic = |k: usize| {
        let f = k as f64 * bin;
        let h = (f / f0).round().max(1.0);
        ((f - h * f0) / bin).abs() <= LOBE as f64 + 0.5
    };
    let (lo, hi) = (
        (20.0 / bin).ceil() as usize,
        (20_000.0 / bin).floor() as usize,
    );
    let mut total = 0.0;
    let mut loudest = (0.0, 0.0);
    for k in lo..=hi.min(p.len() - 1) {
        if harmonic(k) {
            continue;
        }
        total += p[k];
        // A component: a local peak, its lobe's power (its own bins that are not a
        // harmonic's).
        if p[k] >= p[k - 1] && p[k] >= p[(k + 1).min(p.len() - 1)] {
            let s: f64 = (k.saturating_sub(LOBE)..=(k + LOBE).min(p.len() - 1))
                .filter(|&j| !harmonic(j))
                .map(|j| p[j])
                .sum();
            if s > loudest.0 {
                loudest = (s, k as f64 * bin);
            }
        }
    }
    let harmonics = (1..)
        .map(|h| h as f64 * f0)
        .take_while(|&f| f < 20_000.0)
        .map(|f| lobe(f) / fund)
        .collect();
    Measure {
        f0,
        total_db: db(total / fund),
        loudest_db: db(loudest.0 / fund),
        loudest_hz: loudest.1,
        harmonics,
    }
}

fn db(p: f64) -> f64 {
    10.0 * p.max(1e-30).log10()
}

/// The harmonics' levels against a reference's: the mean difference from 10 to 20 kHz and
/// the largest anywhere below 20 kHz (harmonics within 70 dB of the fundamental), dB.
pub fn against(m: &Measure, reference: &[f64]) -> (f64, f64) {
    let (mut top, mut n, mut worst) = (0.0, 0, 0.0f64);
    for (k, (a, r)) in m.harmonics.iter().zip(reference).enumerate() {
        if *r < 1e-7 {
            continue;
        }
        let d = db(a / r);
        if d.abs() > worst.abs() {
            worst = d;
        }
        if (k + 1) as f64 * m.f0 >= 10_000.0 {
            top += d;
            n += 1;
        }
    }
    (if n > 0 { top / n as f64 } else { 0.0 }, worst)
}

/// Oscillator 1 of the circuit's model on 2' (key 4 is A-440), tuned the 1973 way at
/// `rate`; its sawtooth, triangle and rectangle for a width bias (V) over `n` samples after
/// a quarter of a second, and its frequency from its resets.
pub fn circuit(
    rate: f64,
    oversample: usize,
    key: f64,
    width: f64,
    n: usize,
) -> ([Vec<f64>; 3], f64) {
    let mut v = ca72::vco::Vco::new(rate, oversample);
    let t = tuned(rate);
    v.expo.r11 = t.r11;
    v.expo.a8 = t.a8;
    let i_in = v.expo.input_current(&osc1_inputs(&t, key, Range::R2));
    v.reset();
    let warm = (rate * 0.25) as usize;
    for _ in 0..warm {
        v.tick(i_in, width);
    }
    let r0 = v.last_reset();
    let mut out = [
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    ];
    for _ in 0..n {
        let o = v.tick(i_in, width);
        out[0].push(o.saw);
        out[1].push(o.tri);
        out[2].push(o.rect);
    }
    let r1 = v.last_reset();
    let f0 = rate * (r1.1 - r0.1) as f64 / (r1.0 - r0.0);
    (out, f0)
}

/// The tuning at a rate (made once each, at 4x and the plug-in's kernel, so that every
/// candidate plays the same frequency).
fn tuned(rate: f64) -> Tuning {
    use std::sync::Mutex;
    static MADE: Mutex<Vec<(u64, Tuning)>> = Mutex::new(Vec::new());
    let key = rate.to_bits();
    if let Ok(m) = MADE.lock()
        && let Some((_, t)) = m.iter().find(|(r, _)| *r == key)
    {
        return *t;
    }
    let lab = ca72::blep::chosen();
    ca72::blep::set(None);
    let mut v = ca72::vco::Vco::new(rate, 4);
    let (t, _) = tuning::folkman_1973(&mut v, rate, Tuning::default());
    ca72::blep::set(lab);
    if let Ok(mut m) = MADE.lock() {
        m.push((key, t));
    }
    t
}

/// One light oscillator at `hz` over `n` samples after a quarter of a second.
pub fn light(rate: f64, hz: f64, wave: Waveform, n: usize) -> Vec<f64> {
    let warm = (rate * 0.25) as usize;
    light::lab_oscillator(rate, hz, wave, warm + n)[warm..]
        .iter()
        .map(|&v| f64::from(v))
        .collect()
}

/// The light oscillator's waveform with no band-limiting at phase `p`, as its `tick` makes
/// it (its levels; the rectangles centred).
fn light_naive(p: f64, wave: Waveform) -> f64 {
    use ca72::light::laws;
    let fall = 1.0 - 2.0 * p;
    let rect = |d: f64| (if p < d { 1.0 } else { -1.0 }) - (2.0 * d - 1.0);
    match wave {
        Waveform::Triangle => {
            let b = f64::from(laws::TRIANGLE_BOTTOM);
            let t = if p < b {
                1.0 - p * (2.0 / b)
            } else {
                (p - b) * (2.0 / (1.0 - b)) - 1.0
            };
            f64::from(laws::TRIANGLE) * t
        }
        Waveform::SharkTooth => {
            let q = (p + f64::from(laws::SHARK_AHEAD)).fract();
            f64::from(laws::SHARK_TRIANGLE) * (4.0 * (q - 0.5).abs() - 1.0)
                + f64::from(laws::SHARK_SAWTOOTH) * fall
        }
        Waveform::Sawtooth => f64::from(laws::SAWTOOTH) * fall,
        Waveform::ReverseSawtooth => -f64::from(laws::REVERSE_SAWTOOTH) * fall,
        Waveform::Square => f64::from(laws::SQUARE) * rect(f64::from(laws::DUTY[0])),
        Waveform::WideRectangle => f64::from(laws::WIDE) * rect(f64::from(laws::DUTY[1])),
        Waveform::NarrowRectangle => f64::from(laws::NARROW) * rect(f64::from(laws::DUTY[2])),
    }
}

/// The light waveform's ideal harmonics' powers below 20 kHz against its fundamental's
/// (its Fourier series, from 2^16 points of one period).
pub fn light_ideal(f0: f64, wave: Waveform) -> Vec<f64> {
    let m = 1 << 16;
    let mut re: Vec<f64> = (0..m)
        .map(|i| light_naive((i as f64 + 0.5) / m as f64, wave))
        .collect();
    let mut im = vec![0.0; m];
    fft(&mut re, &mut im);
    let pw = |k: usize| re[k] * re[k] + im[k] * im[k];
    (1..)
        .take_while(|&k| k as f64 * f0 < 20_000.0)
        .map(|k| pw(k) / pw(1))
        .collect()
}
