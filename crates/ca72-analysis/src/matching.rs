//! Measurements for fitting a patch to a recording. Spectral comparisons use magnitudes,
//! retain time, and match level explicitly; oscillator phase is not a measure of timbre.

use crate::fft::{hann, power};
use serde::Serialize;

pub mod regions;

pub const MAX_SECONDS: f64 = 60.0;
const FLOOR: f64 = -120.0;

/// Refuse invalid audio before any measurement (including the older analysis routines).
pub fn validate(ch: &[Vec<f32>], rate: u32) -> Result<(), String> {
    if !(8_000..=192_000).contains(&rate) || !(1..=2).contains(&ch.len()) {
        return Err("use mono or stereo audio at 8000 to 192000 Hz".into());
    }
    let n = ch[0].len();
    if n < 1024 || n as f64 / rate as f64 > MAX_SECONDS {
        return Err("select at least 1024 samples and at most 60 seconds".into());
    }
    if ch
        .iter()
        .any(|c| c.len() != n || c.iter().any(|x| !x.is_finite()))
    {
        return Err("audio has unequal channel lengths or non-finite samples".into());
    }
    Ok(())
}

fn db(p: f64) -> f64 {
    (10.0 * p.max(1e-12).log10()).max(FLOOR)
}

fn energy(ch: &[Vec<f32>]) -> f64 {
    ch.iter()
        .flatten()
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        / (ch.len() * ch[0].len()) as f64
}

/// Pitch and the older mono summaries use the channel with most energy. Averaging an
/// anti-phase stereo recording would erase it. Spectra and envelopes use both channels.
fn strongest(ch: &[Vec<f32>]) -> Vec<f64> {
    let c = ch
        .iter()
        .max_by(|a, b| {
            let e = |x: &[f32]| x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>();
            e(a).total_cmp(&e(b))
        })
        .expect("validated channels");
    c.iter().map(|v| f64::from(*v)).collect()
}

fn frame(ch: &[Vec<f32>], start: usize, n: usize, window: &[f64]) -> Vec<f64> {
    let mut bins = vec![0.0; n / 2 + 1];
    let norm = n as f64 * window.iter().map(|w| w * w).sum::<f64>() * ch.len() as f64;
    for c in ch {
        let x: Vec<f64> = (start..start + n)
            .map(|i| c.get(i).map_or(0.0, |v| f64::from(*v)))
            .collect();
        for (i, (sum, p)) in bins.iter_mut().zip(power(&x, window)).enumerate() {
            *sum += p * if i == 0 || i == n / 2 { 1.0 } else { 2.0 } / norm;
        }
    }
    bins
}

fn average(ch: &[Vec<f32>], n: usize) -> Vec<f64> {
    let window = hann(n);
    let hop = (n / 2).max(ch[0].len() / 128);
    let mut out = vec![0.0; n / 2 + 1];
    let mut count = 0;
    for start in (0..ch[0].len().saturating_sub(n).max(1)).step_by(hop) {
        for (sum, v) in out.iter_mut().zip(frame(ch, start, n, &window)) {
            *sum += v;
        }
        count += 1;
    }
    for x in &mut out {
        *x /= count as f64;
    }
    out
}

fn peak_near(bins: &[f64], hz: f64, bin_hz: f64) -> f64 {
    let at = (hz / bin_hz).round() as usize;
    let lo = at.saturating_sub(1);
    let hi = (at + 2).min(bins.len());
    bins.get(lo..hi)
        .unwrap_or(&[])
        .iter()
        .copied()
        .fold(0.0, f64::max)
}

#[derive(Debug, Clone, Serialize)]
pub struct Harmonic {
    pub number: usize,
    pub hz: f64,
    pub relative_to_fundamental_db: f64,
    pub lower_sideband_relative_db: Option<f64>,
    pub upper_sideband_relative_db: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct Features {
    pub sample_rate: u32,
    pub frames: usize,
    pub seconds: f64,
    pub rms_dbfs: f64,
    pub peak_dbfs: f64,
    pub loudness: crate::loudness::Loudness,
    pub pitch: crate::pitch::Pitch,
    pub onsets_seconds: Vec<f64>,
    /// (window start in seconds, RMS dBFS), both channels' energy, 10 ms windows.
    pub envelope: Vec<(f64, f64)>,
    pub harmonic_fundamental_hz: Option<f64>,
    pub harmonic_bin_hz: f64,
    pub harmonics: Vec<Harmonic>,
    pub modulation_hz: Option<f64>,
    pub warning: &'static str,
}

pub fn features(
    ch: &[Vec<f32>],
    rate: u32,
    fundamental: Option<f64>,
    modulation: Option<f64>,
) -> Result<Features, String> {
    validate(ch, rate)?;
    for (name, v, top) in [
        ("fundamental_hz", fundamental, rate as f64 / 4.0),
        ("modulation_hz", modulation, rate as f64 / 4.0),
    ] {
        if v.is_some_and(|v| !v.is_finite() || v <= 0.0 || v > top) {
            return Err(format!(
                "{name} must be finite, positive and no higher than {top}"
            ));
        }
    }
    let mono = strongest(ch);
    let pitch = crate::pitch::pitch(&mono, rate);
    let f0 = fundamental.or(pitch.median_hz);
    let n = ch[0].len().next_power_of_two().clamp(4096, 65536);
    let bins = average(ch, n);
    let bin_hz = rate as f64 / n as f64;
    let mut harmonics = Vec::new();
    if let Some(f0) = f0 {
        let h1 = db(peak_near(&bins, f0, bin_hz));
        for number in 1..=16 {
            let hz = f0 * number as f64;
            if hz + modulation.unwrap_or(0.0) >= rate as f64 / 2.0 {
                break;
            }
            let centre = db(peak_near(&bins, hz, bin_hz));
            let side = modulation.filter(|m| *m > 3.0 * bin_hz);
            harmonics.push(Harmonic {
                number,
                hz,
                relative_to_fundamental_db: centre - h1,
                lower_sideband_relative_db: side
                    .filter(|m| hz > *m)
                    .map(|m| db(peak_near(&bins, hz - m, bin_hz)) - centre),
                upper_sideband_relative_db: side
                    .map(|m| db(peak_near(&bins, hz + m, bin_hz)) - centre),
            });
        }
    }
    let hop = (rate / 100) as usize;
    let envelope = (0..ch[0].len())
        .step_by(hop)
        .map(|at| {
            let end = (at + hop).min(ch[0].len());
            let p = ch
                .iter()
                .map(|c| {
                    c[at..end]
                        .iter()
                        .map(|v| f64::from(*v).powi(2))
                        .sum::<f64>()
                })
                .sum::<f64>()
                / ((end - at) * ch.len()) as f64;
            (at as f64 / rate as f64, db(p))
        })
        .collect();
    let refs: Vec<&[f32]> = ch.iter().map(Vec::as_slice).collect();
    Ok(Features {
        sample_rate: rate,
        frames: ch[0].len(),
        seconds: ch[0].len() as f64 / rate as f64,
        rms_dbfs: db(energy(ch)),
        peak_dbfs: crate::peak::db(crate::peak::sample_peak(&refs)).unwrap_or(FLOOR),
        loudness: crate::loudness::loudness(&refs, rate),
        pitch,
        onsets_seconds: crate::onsets::onsets(&mono, rate),
        envelope,
        harmonic_fundamental_hz: f0,
        harmonic_bin_hz: bin_hz,
        harmonics,
        modulation_hz: modulation,
        warning: "Pitch assumes a predominant monophonic part. Harmonics and sidebands describe the selected window; isolate individual notes for a changing phrase or full mix. Unresolved sidebands are null, not absent modulation.",
    })
}

#[derive(Debug, Serialize)]
pub struct Resolution {
    pub window_samples: usize,
    pub hop_samples: usize,
    pub spectral_mae_db: f64,
    pub compared_bins: usize,
}

#[derive(Debug, Serialize)]
pub struct Match {
    /// Gain applied to the candidate for the spectral and envelope comparisons.
    pub candidate_gain_db: f64,
    pub level_matching: &'static str,
    pub resolutions: Vec<Resolution>,
    pub envelope_mae_db: f64,
    pub median_pitch_difference_cents: Option<f64>,
    pub harmonic_differences_db: Vec<(usize, f64)>,
    pub reference: Features,
    pub candidate: Features,
    pub interpretation: &'static str,
}

pub fn compare(
    reference: &[Vec<f32>],
    candidate: &[Vec<f32>],
    rate: u32,
    fundamental: Option<f64>,
    modulation: Option<f64>,
) -> Result<Match, String> {
    let a = features(reference, rate, fundamental, modulation)?;
    let b = features(candidate, rate, fundamental, modulation)?;
    if a.frames != b.frames {
        return Err("select equal durations; comparison never truncates a longer input".into());
    }
    if energy(reference) <= 1e-12 || energy(candidate) <= 1e-12 {
        return Err("cannot compare a silent reference or candidate".into());
    }
    let (gain, matching) = match (a.loudness.integrated, b.loudness.integrated) {
        // A mono file plays in both speakers. Canonicalise its layout for matching with
        // a stereo render, while Features retains the file's actual BS.1770 measurement.
        (Some(x), Some(y)) => (
            x + if reference.len() == 1 {
                10.0 * 2.0f64.log10()
            } else {
                0.0
            } - y
                - if candidate.len() == 1 {
                    10.0 * 2.0f64.log10()
                } else {
                    0.0
                },
            "integrated LUFS, mono duplicated to stereo for matching",
        ),
        _ => (
            a.rms_dbfs - b.rms_dbfs,
            "RMS (integrated LUFS unavailable for this clip)",
        ),
    };
    let mut resolutions = Vec::new();
    for n in [512, 2048, 8192] {
        let hop = n / 4;
        let window = hann(n);
        let (mut total, mut count) = (0.0, 0);
        for start in (0..a.frames).step_by(hop) {
            let pa = frame(reference, start, n, &window);
            let pb = frame(candidate, start, n, &window);
            let top = pa
                .iter()
                .copied()
                .map(db)
                .chain(pb.iter().copied().map(|p| db(p) + gain))
                .fold(FLOOR, f64::max);
            for (x, y) in pa.into_iter().zip(pb) {
                let x = db(x);
                let y = db(y) + gain;
                // Missing and extra harmonics/noise both count.
                if x.max(y) > (top - 60.0).max(-100.0) {
                    total += (x - y).abs();
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Err("no audible reference bins to compare".into());
        }
        resolutions.push(Resolution {
            window_samples: n,
            hop_samples: hop,
            spectral_mae_db: total / count as f64,
            compared_bins: count,
        });
    }
    let peak = a
        .envelope
        .iter()
        .map(|(_, x)| *x)
        .chain(b.envelope.iter().map(|(_, y)| *y + gain))
        .fold(FLOOR, f64::max);
    let differences: Vec<f64> = a
        .envelope
        .iter()
        .zip(&b.envelope)
        .filter(|((_, x), (_, y))| x.max(y + gain) > peak - 60.0)
        .map(|((_, x), (_, y))| (x - (y + gain)).abs())
        .collect();
    let cents = a
        .pitch
        .median_hz
        .zip(b.pitch.median_hz)
        .map(|(x, y)| 1200.0 * (y / x).log2());
    let harmonics = a
        .harmonics
        .iter()
        .zip(&b.harmonics)
        .map(|(x, y)| {
            (
                x.number,
                y.relative_to_fundamental_db - x.relative_to_fundamental_db,
            )
        })
        .collect();
    Ok(Match {
        candidate_gain_db: gain,
        level_matching: matching,
        resolutions,
        envelope_mae_db: differences.iter().sum::<f64>() / differences.len().max(1) as f64,
        median_pitch_difference_cents: cents,
        harmonic_differences_db: harmonics,
        reference: a,
        candidate: b,
        interpretation: "Errors are measurements, not a percentage of perceptual similarity or an automatic pass. Windows start at the supplied crop positions; no hidden time or pitch alignment is applied. Listen to the level-matched pair and inspect note-specific harmonics, envelopes and sidebands.",
    })
}

/// A bounded log-frequency spectrogram. The axes and colour scale are returned alongside
/// the PNG so clients can inspect exact coordinates; the two inputs share that scale.
pub fn spectrogram(
    ch: &[Vec<f32>],
    rate: u32,
    gain_db: f64,
    top_db: Option<f64>,
) -> Result<(Vec<u8>, f64), String> {
    validate(ch, rate)?;
    const WIDTH: usize = 640;
    const HEIGHT: usize = 320;
    const N: usize = 4096;
    let window = hann(N);
    let mut values = vec![FLOOR; WIDTH * HEIGHT];
    for x in 0..WIDTH {
        let start = x * ch[0].len().saturating_sub(N) / (WIDTH - 1);
        let bins = frame(ch, start, N, &window);
        for y in 0..HEIGHT {
            let hz = 20.0
                * ((rate as f64 / 2.0) / 20.0).powf((HEIGHT - 1 - y) as f64 / (HEIGHT - 1) as f64);
            let at = (hz * N as f64 / rate as f64).round() as usize;
            values[y * WIDTH + x] = db(bins[at.min(bins.len() - 1)]) + gain_db;
        }
    }
    let top = top_db.unwrap_or_else(|| values.iter().copied().fold(FLOOR, f64::max));
    let mut pixels = Vec::with_capacity(WIDTH * HEIGHT * 3);
    for v in values {
        let t = ((v - top + 80.0) / 80.0).clamp(0.0, 1.0);
        pixels.extend_from_slice(&[
            (255.0 * t.powf(0.7)) as u8,
            (255.0 * t.powi(2)) as u8,
            (100.0 * (1.0 - t) * t * 4.0) as u8,
        ]);
    }
    let pixels = labelled_spectrogram(
        &pixels,
        WIDTH,
        HEIGHT,
        ch[0].len().saturating_sub(N) as f64 / rate as f64,
        rate as f64 / 2.0,
        top,
    );
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 840, 370);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|e| e.to_string())?;
    }
    Ok((bytes, top))
}

// Small original bitmap lettering keeps generated plots readable without a system font,
// GUI, or platform-specific text renderer. Coordinates describe frame starts, not centres.
fn text(p: &mut [u8], width: usize, x: usize, y: usize, s: &str) {
    for (i, c) in s.chars().enumerate() {
        let rows = match c {
            '0' => [14, 17, 19, 21, 25, 17, 14],
            '1' => [4, 12, 4, 4, 4, 4, 14],
            '2' => [14, 17, 1, 2, 4, 8, 31],
            '3' => [30, 1, 1, 14, 1, 1, 30],
            '4' => [2, 6, 10, 18, 31, 2, 2],
            '5' => [31, 16, 16, 30, 1, 1, 30],
            '6' => [14, 16, 16, 30, 17, 17, 14],
            '7' => [31, 1, 2, 4, 8, 8, 8],
            '8' => [14, 17, 17, 14, 17, 17, 14],
            '9' => [14, 17, 17, 15, 1, 1, 14],
            '.' => [0, 0, 0, 0, 0, 12, 12],
            '-' => [0, 0, 0, 31, 0, 0, 0],
            'H' => [17, 17, 17, 31, 17, 17, 17],
            'z' => [0, 0, 31, 2, 4, 8, 31],
            's' => [0, 0, 15, 16, 14, 1, 30],
            'd' => [1, 1, 15, 17, 17, 17, 15],
            'B' => [30, 17, 17, 30, 17, 17, 30],
            _ => [0; 7],
        };
        for (dy, row) in rows.iter().enumerate() {
            for dx in 0..5 {
                if row & (1 << (4 - dx)) != 0 {
                    let at = ((y + dy) * width + x + i * 6 + dx) * 3;
                    if at + 3 <= p.len() {
                        p[at..at + 3].copy_from_slice(&[225, 225, 230]);
                    }
                }
            }
        }
    }
}

fn labelled_spectrogram(
    heat: &[u8],
    w: usize,
    h: usize,
    seconds: f64,
    max_hz: f64,
    top: f64,
) -> Vec<u8> {
    let mut p = vec![20; 840 * 370 * 3];
    for y in 0..h {
        let at = ((y + 20) * 840 + 70) * 3;
        p[at..at + w * 3].copy_from_slice(&heat[y * w * 3..(y + 1) * w * 3]);
    }
    text(&mut p, 840, 8, 8, "Hz");
    text(&mut p, 840, 708, 358, "s");
    text(&mut p, 840, 736, 8, "dB");
    for hz in [
        20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0, 40000.0, 80000.0,
    ] {
        if hz > max_hz {
            continue;
        }
        let y = 20 + (h - 1)
            - ((hz / 20.0f64).ln() / (max_hz / 20.0).ln() * (h - 1) as f64).round() as usize;
        text(&mut p, 840, 8, y.saturating_sub(3), &format!("{hz:.0}"));
        for x in 63..70 {
            p[(y * 840 + x) * 3..(y * 840 + x) * 3 + 3].fill(180);
        }
    }
    for i in 0..=4 {
        let x = 70 + i * (w - 1) / 4;
        text(
            &mut p,
            840,
            x.saturating_sub(8),
            350,
            &format!("{:.2}", seconds * i as f64 / 4.0),
        );
        for y in 340..345 {
            p[(y * 840 + x) * 3..(y * 840 + x) * 3 + 3].fill(180);
        }
    }
    for y in 0..h {
        let t = 1.0 - y as f64 / (h - 1) as f64;
        let rgb = [
            (255.0 * t.powf(0.7)) as u8,
            (255.0 * t.powi(2)) as u8,
            (100.0 * (1.0 - t) * t * 4.0) as u8,
        ];
        for x in 735..747 {
            let at = ((y + 20) * 840 + x) * 3;
            p[at..at + 3].copy_from_slice(&rgb);
        }
    }
    for i in 0..=4 {
        text(
            &mut p,
            840,
            758,
            17 + i * (h - 1) / 4,
            &format!("{:.1}", top - i as f64 * 20.0),
        );
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tone(gain: f32, hz: f64, harmonic: f64) -> Vec<Vec<f32>> {
        vec![
            (0..24_000)
                .map(|i| {
                    let p = std::f64::consts::TAU * hz * i as f64 / 48_000.0;
                    gain * (p.sin() + harmonic * (3.0 * p).sin()) as f32
                })
                .collect(),
        ]
    }
    #[test]
    fn level_is_separate_from_timbre_and_phase_cancellation_does_not_erase_stereo() {
        let a = tone(0.2, 200.0, 0.3);
        let b = tone(0.1, 200.0, 0.3);
        let m = compare(&a, &b, 48_000, Some(200.0), None).expect("compare");
        assert!((m.candidate_gain_db - 6.0206).abs() < 0.01);
        assert!(m.resolutions.iter().all(|r| r.spectral_mae_db < 0.02));
        assert!(m.envelope_mae_db < 0.02);
        let stereo = vec![a[0].clone(), a[0].iter().map(|v| -*v).collect()];
        let s = features(&stereo, 48_000, Some(200.0), None).expect("stereo");
        assert!(s.pitch.median_hz.is_some());
        assert!((s.rms_dbfs - m.reference.rms_dbfs).abs() < 0.001);
        let duplicated = vec![a[0].clone(), a[0].clone()];
        let m = compare(&a, &duplicated, 48_000, Some(200.0), None).expect("mono versus stereo");
        assert!(m.candidate_gain_db.abs() < 0.01);
        assert!(m.resolutions.iter().all(|r| r.spectral_mae_db < 0.02));
    }
    #[test]
    fn a_wrong_harmonic_pitch_or_envelope_is_measured() {
        let a = tone(0.2, 200.0, 0.3);
        let b = tone(0.2, 220.0, 0.0);
        let m = compare(&a, &b, 48_000, Some(200.0), None).expect("compare");
        assert!(m.resolutions.iter().all(|r| r.spectral_mae_db > 3.0));
        assert!(m.median_pitch_difference_cents.is_some_and(|x| x > 140.0));
        let mut b = a.clone();
        b[0][..6000].fill(0.0);
        assert!(
            compare(&a, &b, 48_000, None, None)
                .expect("attack")
                .envelope_mae_db
                > 10.0
        );
    }
    #[test]
    fn bad_short_and_silent_inputs_do_not_pass_as_matches() {
        assert!(validate(&[vec![0.0; 20]], 48_000).is_err());
        assert!(validate(&[vec![f32::NAN; 2000]], 48_000).is_err());
        assert!(validate(&[vec![0.0; 2000]], 0).is_err());
        assert!(
            compare(
                &tone(0.0, 200.0, 0.0),
                &tone(0.2, 200.0, 0.0),
                48_000,
                None,
                None
            )
            .is_err()
        );
        assert!(
            compare(
                &tone(0.2, 200.0, 0.0),
                &[vec![0.1; 4096]],
                48_000,
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn extra_energy_and_release_tails_count() {
        let a = tone(0.2, 200.0, 0.0);
        let b = tone(0.2, 200.0, 0.4);
        let m = compare(&a, &b, 48000, Some(200.0), None).expect("extra harmonic");
        assert!(m.resolutions.iter().all(|r| r.spectral_mae_db > 1.0));
        let mut short = a.clone();
        short[0][12000..].fill(0.0);
        let m = compare(&short, &a, 48000, None, None).expect("extra tail");
        assert!(m.envelope_mae_db > 20.0);
    }

    #[test]
    fn modulation_sidebands_are_measured_or_reported_as_unresolved() {
        let f0 = 93.75;
        let modulation = 11.71875;
        let ch = vec![
            (0..96000)
                .map(|i| {
                    let t = i as f64 / 48000.0;
                    let wave = |f: f64| (std::f64::consts::TAU * f * t).sin();
                    (0.2 * (wave(f0) + 0.1 * wave(f0 - modulation) + 0.1 * wave(f0 + modulation)))
                        as f32
                })
                .collect(),
        ];
        let f = features(&ch, 48000, Some(f0), Some(modulation)).expect("sidebands");
        let h1 = &f.harmonics[0];
        assert!((h1.lower_sideband_relative_db.expect("lower") + 20.0).abs() < 0.05);
        assert!((h1.upper_sideband_relative_db.expect("upper") + 20.0).abs() < 0.05);
        let f = features(&ch, 48000, Some(f0), Some(0.1)).expect("unresolved");
        assert!(f.harmonics[0].lower_sideband_relative_db.is_none());
    }
}
