//! Named, reusable measurements for experiments. No instrument-specific objectives or
//! automatic transcription: a region can describe a note, chord, attack or release.

use super::{db, energy, validate};
use crate::fft::{fft, hann};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::f64::consts::TAU;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub id: String,
    pub start: f64,
    pub seconds: f64,
    #[serde(default)]
    pub fundamentals_hz: Vec<f64>,
    pub modulation_hz: Option<f64>,
}

pub fn validate_regions(regions: &[Region], frames: usize, rate: u32) -> Result<(), String> {
    if regions.is_empty() || regions.len() > 32 {
        return Err("regions must contain 1 to 32 named windows".into());
    }
    let mut ids = BTreeSet::new();
    let mut total = 0.0;
    for r in regions {
        if r.id.is_empty() || r.id.len() > 64 || !ids.insert(&r.id) {
            return Err("region ids must be unique, nonempty and at most 64 bytes".into());
        }
        if !r.start.is_finite() || r.start < 0.0 || !r.seconds.is_finite() || r.seconds <= 0.0 {
            return Err(format!("{}: invalid region start or seconds", r.id));
        }
        let first = (r.start * f64::from(rate)).round() as usize;
        let count = (r.seconds * f64::from(rate)).round() as usize;
        if count < 1024 || first.checked_add(count).is_none_or(|end| end > frames) {
            return Err(format!(
                "{}: region needs 1024 samples and must fit the selected clip",
                r.id
            ));
        }
        if r.fundamentals_hz.len() > 10
            || r.fundamentals_hz
                .iter()
                .any(|f| !f.is_finite() || *f < 20.0 || *f >= f64::from(rate) / 4.0)
            || r.modulation_hz
                .is_some_and(|f| !f.is_finite() || !(0.1..=30.0).contains(&f))
        {
            return Err(format!(
                "{}: supply at most 10 fundamentals in20Hz..Nyquist/2 and modulation in0.1..30Hz",
                r.id
            ));
        }
        total += r.seconds;
    }
    if total > 120.0 {
        return Err("total region duration, including overlaps, exceeds120 seconds".into());
    }
    Ok(())
}

fn crop(ch: &[Vec<f32>], r: &Region, rate: u32) -> Vec<Vec<f32>> {
    let a = (r.start * f64::from(rate)).round() as usize;
    let b = a + (r.seconds * f64::from(rate)).round() as usize;
    ch.iter().map(|c| c[a..b].to_vec()).collect()
}

fn spectrum(ch: &[Vec<f32>], rate: u32) -> (Vec<f64>, f64, f64) {
    // Window actual samples, then zero-pad. Padding never pretends to improve physical
    // resolution. Long windows average overlapping frames, including their final frame.
    let size = ch[0].len().min(65536);
    let n = size.next_power_of_two();
    let window = hann(size);
    let norm = n as f64 * window.iter().map(|w| w * w).sum::<f64>() * ch.len() as f64;
    let mut starts: Vec<_> = (0..=ch[0].len() - size)
        .step_by((size / 2).max(1))
        .collect();
    if starts.last().copied() != Some(ch[0].len() - size) {
        starts.push(ch[0].len() - size);
    }
    let mut bins = vec![0.0; n / 2 + 1];
    for start in &starts {
        for c in ch {
            let mean = c[*start..*start + size]
                .iter()
                .map(|v| f64::from(*v))
                .sum::<f64>()
                / size as f64;
            let mut re = vec![0.0; n];
            let mut im = vec![0.0; n];
            for i in 0..size {
                re[i] = (f64::from(c[start + i]) - mean) * window[i];
            }
            fft(&mut re, &mut im);
            for (i, p) in bins.iter_mut().enumerate() {
                *p += (re[i] * re[i] + im[i] * im[i])
                    * if i == 0 || i == n / 2 { 1.0 } else { 2.0 }
                    / norm
                    / starts.len() as f64;
            }
        }
    }
    (
        bins,
        f64::from(rate) / n as f64,
        f64::from(rate) / size as f64,
    )
}

fn band(bins: &[f64], bin: f64, lo: f64, hi: f64) -> f64 {
    let a = (lo.max(0.0) / bin).ceil() as usize;
    let b = (hi.max(0.0) / bin).ceil() as usize;
    bins.get(a.min(bins.len())..b.min(bins.len()))
        .unwrap_or(&[])
        .iter()
        .sum()
}

fn near(
    bins: &[f64],
    bin: f64,
    expected: f64,
    resolution: f64,
    modulation_hz: Option<f64>,
) -> Result<(f64, f64, bool), String> {
    let lo = ((expected * 0.96 / bin).ceil() as usize).max(1);
    let hi = ((expected * 1.04 / bin).floor() as usize).min(bins.len() - 2);
    let Some(k) = (lo..=hi)
        .filter(|i| bins[*i] >= bins[*i - 1] && bins[*i] > bins[*i + 1])
        .max_by(|a, b| bins[*a].total_cmp(&bins[*b]))
    else {
        return Err(
            "no FFT bins resolve the supplied fundamental search range; use a longer region".into(),
        );
    };
    let maximum = bins.iter().copied().fold(0.0, f64::max);
    let support = 10.0 * (bins[k].max(1e-30) / maximum.max(1e-30)).log10();
    if support < -40.0 {
        return Err("fundamental unresolved: the local peak is below−40dB of the strongest peak; recheck pitch hint or choose a clearer/longer region".into());
    }
    let a = bins[k - 1].max(1e-30).ln();
    let b = bins[k].max(1e-30).ln();
    let c = bins[k + 1].max(1e-30).ln();
    let den = a - 2.0 * b + c;
    let delta = if den.abs() > 1e-12 {
        (0.5 * (a - c) / den).clamp(-0.5, 0.5)
    } else {
        0.0
    };
    // Comparable separated peaks can be FM sidebands or multiple detuned voices. A
    // strongest spectral peak is still useful as a harmonic-band anchor, but must not
    // masquerade as a resolved carrier for pitch budgets or complex demodulation.
    let a = ((expected * 0.92 / bin).ceil() as usize).max(1);
    let b = ((expected * 1.08 / bin).floor() as usize).min(bins.len() - 2);
    let separation = (2.0 * resolution).max(modulation_hz.unwrap_or(0.0) * 1.5);
    let power = |i: usize| {
        let hz = i as f64 * bin;
        band(bins, bin, hz - resolution, hz + resolution)
    };
    let resolved = !(a..=b).any(|i| {
        (i.abs_diff(k) as f64 * bin) > separation
            && bins[i] >= bins[i - 1]
            && bins[i] > bins[i + 1]
            && power(i) >= power(k) * 10.0f64.powf(-6.0 / 10.0)
    });
    Ok((
        ((k as f64 + delta) * bin).clamp(expected * 0.96, expected * 1.04),
        support,
        resolved,
    ))
}

fn ac_strongest(ch: &[Vec<f32>]) -> Vec<f64> {
    ch.iter()
        .map(|c| {
            let mean = c.iter().map(|x| f64::from(*x)).sum::<f64>() / c.len() as f64;
            c.iter().map(|x| f64::from(*x) - mean).collect::<Vec<_>>()
        })
        .max_by(|a, b| {
            a.iter()
                .map(|v| v * v)
                .sum::<f64>()
                .total_cmp(&b.iter().map(|v| v * v).sum::<f64>())
        })
        .expect("validated channels")
}

#[derive(Debug, Clone, Serialize)]
pub struct Partial {
    pub fundamental_index: usize,
    pub number: usize,
    pub hz: f64,
    pub relative_db: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Peak {
    pub hz: f64,
    pub relative_db: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Modulation {
    pub rate_hz: f64,
    pub am_depth_db: f64,
    pub fm_depth_cents: f64,
    pub frames: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Measurement {
    pub id: String,
    pub start: f64,
    pub seconds: f64,
    pub fundamental_estimates_hz: Vec<Option<f64>>,
    pub harmonic_anchors_hz: Vec<f64>,
    pub fundamental_peak_support_db: Vec<f64>,
    pub fft_bin_hz: f64,
    pub physical_resolution_hz: f64,
    pub normalization: &'static str,
    pub harmonics: Vec<Partial>,
    pub spectral_band_centres_hz: Vec<f64>,
    pub spectral_bands_db: Vec<f64>,
    pub upper_band_db: Option<f64>,
    pub peaks: Vec<Peak>,
    pub modulation: Option<Modulation>,
    pub warnings: Vec<String>,
}

/// Fit a known periodic rate after removing a constant and linear trend. Solve the
/// complete least-squares system so detrending does not bias a noninteger cycle count.
fn periodic(t: &[f64], y: &[f64], hz: f64) -> Option<f64> {
    if t.len() < 12 || t.last()? - t.first()? < 3.0 / hz {
        return None;
    }
    let mut a = [[0.0; 5]; 4];
    for (t, y) in t.iter().zip(y) {
        let row = [1.0, *t, (TAU * hz * t).sin(), (TAU * hz * t).cos()];
        for i in 0..4 {
            for j in 0..4 {
                a[i][j] += row[i] * row[j];
            }
            a[i][4] += row[i] * y;
        }
    }
    for i in 0..4 {
        let k = (i..4).max_by(|x, y| a[*x][i].abs().total_cmp(&a[*y][i].abs()))?;
        a.swap(i, k);
        let pivot = a[i][i];
        if pivot.abs() < 1e-10 {
            return None;
        }
        for x in a[i].iter_mut().skip(i) {
            *x /= pivot;
        }
        let row = a[i];
        for (k, other) in a.iter_mut().enumerate() {
            if k != i {
                let v = other[i];
                for (x, y) in other.iter_mut().zip(row).skip(i) {
                    *x -= v * y;
                }
            }
        }
    }
    Some(a[2][4].hypot(a[3][4]))
}

fn modulation(ch: &[Vec<f32>], rate: u32, f0: f64, hz: f64) -> Option<Modulation> {
    let mono = ac_strongest(ch);
    let size = (f64::from(rate) * 0.05).round() as usize;
    let hop = (rate / 100) as usize;
    if mono.len() < size {
        return None;
    }
    let w = hann(size);
    let response = w
        .iter()
        .enumerate()
        .map(|(i, w)| w * (TAU * hz * (i as f64 - size as f64 * 0.5) / f64::from(rate)).cos())
        .sum::<f64>()
        / w.iter().sum::<f64>();
    // Do not amplify an ill-conditioned window response into a plausible depth.
    if response < 0.8 {
        return None;
    }
    let derivative_response = (std::f64::consts::PI * hz * hop as f64 / f64::from(rate)).sin()
        / (std::f64::consts::PI * hz * hop as f64 / f64::from(rate));
    let mut times = Vec::new();
    let mut amps = Vec::new();
    let mut phases = Vec::new();
    let mut unwrapped = 0.0;
    let mut last = None;
    for at in (0..=mono.len() - size).step_by(hop) {
        let mut re = 0.0;
        let mut im = 0.0;
        let angle = TAU * f0 / f64::from(rate);
        let (ds, dc) = angle.sin_cos();
        let (mut s, mut c) = (TAU * f0 * at as f64 / f64::from(rate)).sin_cos();
        for i in 0..size {
            let v = mono[at + i] * w[i];
            re += v * c;
            im -= v * s;
            (s, c) = (s * dc + c * ds, c * dc - s * ds);
        }
        let phase = im.atan2(re);
        if let Some(prev) = last {
            let delta: f64 = phase - prev;
            unwrapped += (delta + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI;
        }
        last = Some(phase);
        times.push((at + size / 2) as f64 / f64::from(rate));
        amps.push(db(re * re + im * im));
        phases.push(unwrapped);
    }
    if amps.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - amps.iter().copied().fold(f64::INFINITY, f64::min)
        > 40.0
    {
        return None;
    }
    let mut pitch = Vec::new();
    let mut pt = Vec::new();
    for i in 1..phases.len() {
        let f = f0 + (phases[i] - phases[i - 1]) / (times[i] - times[i - 1]) / TAU;
        if f <= 0.0 {
            return None;
        }
        pitch.push(1200.0 * (f / f0).log2());
        pt.push((times[i] + times[i - 1]) * 0.5);
    }
    Some(Modulation {
        rate_hz: hz,
        am_depth_db: periodic(&times, &amps, hz)? / response,
        fm_depth_cents: periodic(&pt, &pitch, hz)? / (response * derivative_response),
        frames: times.len(),
    })
}

pub fn analyze(ch: &[Vec<f32>], rate: u32, regions: &[Region]) -> Result<Vec<Measurement>, String> {
    validate(ch, rate)?;
    validate_regions(regions, ch[0].len(), rate)?;
    regions.iter().map(|r| measure(ch, rate, r)).collect()
}

fn measure(ch: &[Vec<f32>], rate: u32, r: &Region) -> Result<Measurement, String> {
    let clip = crop(ch, r, rate);
    if energy(&clip) <= 1e-12 {
        return Err(format!(
            "{}: silent region has no normalization reference",
            r.id
        ));
    }
    let (p, bin, resolution) = spectrum(&clip, rate);
    if r.fundamentals_hz.iter().any(|f| *f < 4.0 * resolution) {
        return Err(format!(
            "{}: fundamental unresolved; use a region containing at least four periods",
            r.id
        ));
    }
    let refined: Vec<_> = r
        .fundamentals_hz
        .iter()
        .map(|f| near(&p, bin, *f, resolution, r.modulation_hz))
        .collect::<Result<_, _>>()?;
    let f0s: Vec<_> = refined.iter().map(|(f, _, _)| *f).collect();
    let support: Vec<_> = refined.iter().map(|(_, s, _)| *s).collect();
    let estimates: Vec<_> = refined
        .iter()
        .map(|(f, _, valid)| valid.then_some(*f))
        .collect();
    let width = |f: f64| 7.0f64.max(f * 0.015).max(2.0 * resolution);
    let norm = if f0s.is_empty() {
        band(&p, bin, 50.0, 1000.0)
    } else {
        f0s.iter()
            .map(|f| band(&p, bin, *f - width(*f), *f + width(*f)))
            .sum()
    };
    if norm <= 1e-16 {
        return Err(format!(
            "{}: insufficient power in fundamental/50–1000Hz normalization bands",
            r.id
        ));
    }
    let rel = |v: f64| 10.0 * (v.max(1e-30) / norm).log10();
    let mut harmonics = Vec::new();
    for (i, f) in f0s.iter().enumerate() {
        for h in 1..=8 {
            let hz = *f * h as f64;
            if hz + width(hz) < f64::from(rate) * 0.5 {
                harmonics.push(Partial {
                    fundamental_index: i,
                    number: h,
                    hz,
                    relative_db: rel(band(&p, bin, hz - width(hz), hz + width(hz))),
                });
            }
        }
    }
    let centres: Vec<_> = (0..120)
        .map(|i| 20.0 * 2.0f64.powf(i as f64 / 12.0))
        .take_while(|f| *f * 2.0f64.powf(1.0 / 24.0) < (f64::from(rate) * 0.5).min(20000.0))
        .collect();
    let bands = centres
        .iter()
        .map(|f| {
            rel(band(
                &p,
                bin,
                *f / 2.0f64.powf(1.0 / 24.0),
                *f * 2.0f64.powf(1.0 / 24.0),
            ))
        })
        .collect();
    let mut indices: Vec<_> = (1..p.len() - 1)
        .filter(|i| {
            *i as f64 * bin >= 50.0
                && *i as f64 * bin <= 20000.0
                && p[*i] > p[*i - 1]
                && p[*i] > p[*i + 1]
        })
        .collect();
    indices.sort_by(|a, b| p[*b].total_cmp(&p[*a]));
    let mut peaks: Vec<Peak> = Vec::new();
    for i in indices {
        let hz = i as f64 * bin;
        if peaks
            .iter()
            .any(|q| (q.hz - hz).abs() < 30.0f64.max(2.0 * resolution))
        {
            continue;
        }
        peaks.push(Peak {
            hz,
            relative_db: rel(band(
                &p,
                bin,
                hz - 15.0f64.max(resolution),
                hz + 15.0f64.max(resolution),
            )),
        });
        if peaks.len() == 24 {
            break;
        }
    }
    let motion = if f0s.len() == 1 && estimates[0].is_some() {
        r.modulation_hz
            .and_then(|hz| modulation(&clip, rate, f0s[0], hz))
    } else {
        None
    };
    let mut warnings = Vec::new();
    if estimates.iter().any(Option::is_none) {
        warnings.push("Carrier unresolved: comparable peaks within ±8% of a supplied fundamental (integrated power within 6 dB, separated by more than max(two physical-resolution bins, 1.5 times the supplied modulation rate)). Harmonic anchors remain spectral peaks; pitch and AM/FM checks are unresolved, not passed. Use a clearer region for carrier measurements.".into());
    }
    if f0s.len() > 1 {
        warnings.push("Chord harmonic bands may overlap: energies are aggregate, not separated voices; normalization is shared across all fundamentals.".into());
    }
    if r.modulation_hz.is_some() && motion.is_none() {
        warnings.push("AM/FM unresolved: use one isolated fundamental, at least three modulation cycles, a stable audible region and a reliable50ms window response (rates above roughly11Hz are unresolved). Null does not mean no movement.".into());
    }
    if resolution > 7.0 {
        warnings.push(format!("Physical spectral resolution is approximately{resolution:0.2}Hz; zero padding does not improve it."));
    }
    Ok(Measurement {
        id: r.id.clone(),
        start: r.start,
        seconds: r.seconds,
        fft_bin_hz: bin,
        physical_resolution_hz: resolution,
        normalization: if f0s.is_empty() {
            "50–1000Hz power"
        } else {
            "sum of supplied fundamental bands; chord overlap retained"
        },
        fundamental_estimates_hz: estimates,
        harmonic_anchors_hz: f0s,
        fundamental_peak_support_db: support,
        harmonics,
        spectral_band_centres_hz: centres,
        spectral_bands_db: bands,
        upper_band_db: (rate > 12000)
            .then(|| rel(band(&p, bin, 6000.0, (f64::from(rate) * 0.5).min(20000.0)))),
        peaks,
        modulation: motion,
        warnings,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct Difference {
    pub id: String,
    pub harmonic_rmse_db: Option<f64>,
    pub spectrum_rmse_db: f64,
    pub upper_band_difference_db: Option<f64>,
    pub fundamental_differences_cents: Vec<Option<f64>>,
    pub am_depth_difference_db: Option<f64>,
    pub fm_depth_difference_cents: Option<f64>,
    pub unexplained_peaks: Vec<Peak>,
}

pub fn differences(a: &[Measurement], b: &[Measurement]) -> Result<Vec<Difference>, String> {
    if a.len() != b.len() {
        return Err("region measurement counts differ".into());
    }
    a.iter()
        .zip(b)
        .map(|(a, b)| {
            if a.id != b.id
                || a.harmonics.len() != b.harmonics.len()
                || a.spectral_band_centres_hz != b.spectral_band_centres_hz
            {
                return Err("region measurements are incompatible".into());
            }
            let rms = |d: Vec<f64>| (d.iter().map(|d| d * d).sum::<f64>() / d.len() as f64).sqrt();
            let harmonics = (!a.harmonics.is_empty()).then(|| {
                rms(a
                    .harmonics
                    .iter()
                    .zip(&b.harmonics)
                    .map(|(a, b)| a.relative_db.max(-60.0) - b.relative_db.max(-60.0))
                    .collect())
            });
            let motion = a.modulation.as_ref().zip(b.modulation.as_ref());
            let peaks = b
                .peaks
                .iter()
                .filter(|p| {
                    p.relative_db > -35.0
                        && !a.peaks.iter().any(|q| {
                            (q.hz - p.hz).abs() < 30.0f64.max(p.hz * 0.02)
                                && q.relative_db >= p.relative_db - 6.0
                        })
                })
                .cloned()
                .collect();
            Ok(Difference {
                id: a.id.clone(),
                harmonic_rmse_db: harmonics,
                spectrum_rmse_db: rms(a
                    .spectral_bands_db
                    .iter()
                    .zip(&b.spectral_bands_db)
                    .map(|(a, b)| a.max(-80.0) - b.max(-80.0))
                    .collect()),
                upper_band_difference_db: a.upper_band_db.zip(b.upper_band_db).map(|(a, b)| b - a),
                fundamental_differences_cents: a
                    .fundamental_estimates_hz
                    .iter()
                    .zip(&b.fundamental_estimates_hz)
                    .map(|(a, b)| a.zip(*b).map(|(a, b)| 1200.0 * (b / a).log2()))
                    .collect(),
                am_depth_difference_db: motion.map(|(a, b)| b.am_depth_db - a.am_depth_db),
                fm_depth_difference_cents: motion.map(|(a, b)| b.fm_depth_cents - a.fm_depth_cents),
                unexplained_peaks: peaks,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub envelope_rmse_db: Option<f64>,
    pub envelope_p95_db: Option<f64>,
    pub am_depth_db: Option<f64>,
    pub fm_depth_cents: Option<f64>,
    pub pitch_cents: Option<f64>,
}

impl Limits {
    pub fn validate_for(&self, regions: &[Region]) -> Result<(), String> {
        self.validate()?;
        if (self.am_depth_db.is_some() || self.fm_depth_cents.is_some())
            && !regions.iter().any(|r| r.modulation_hz.is_some())
        {
            return Err("AM/FM limits require at least one region with modulation_hz".into());
        }
        if self.pitch_cents.is_some() && !regions.iter().any(|r| !r.fundamentals_hz.is_empty()) {
            return Err("pitch_cents requires a region with supplied fundamentals".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if [
            self.envelope_rmse_db,
            self.envelope_p95_db,
            self.am_depth_db,
            self.fm_depth_cents,
            self.pitch_cents,
        ]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite() || v < 0.0)
        {
            return Err("preservation limits must be finite and nonnegative".into());
        }
        Ok(())
    }
}

fn envelope(ch: &[Vec<f32>], rate: u32) -> Vec<(f64, f64)> {
    let size = (f64::from(rate) * 0.05).round() as usize;
    let hop = (rate / 100) as usize;
    if ch[0].len() < size {
        return Vec::new();
    }
    (0..=ch[0].len() - size)
        .step_by(hop)
        .map(|at| {
            let p = ch
                .iter()
                .map(|c| {
                    let mean =
                        c[at..at + size].iter().map(|v| f64::from(*v)).sum::<f64>() / size as f64;
                    c[at..at + size]
                        .iter()
                        .map(|v| (f64::from(*v) - mean).powi(2))
                        .sum::<f64>()
                        / size as f64
                })
                .sum::<f64>()
                / ch.len() as f64;
            ((at + size / 2) as f64 / f64::from(rate), p)
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct PreservationRegion {
    pub id: String,
    pub envelope_frames: usize,
    pub requested_checks: usize,
    pub envelope_rmse_db: Option<f64>,
    pub envelope_p95_db: Option<f64>,
    pub am_depth_change_db: Option<f64>,
    pub fm_depth_change_cents: Option<f64>,
    pub fundamental_shifts_cents: Vec<Option<f64>>,
    /// Null when no limits were requested or a requested measurement is unresolved.
    pub within_limits: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Preservation {
    pub candidate_gain_db: f64,
    pub limits: Limits,
    pub regions: Vec<PreservationRegion>,
    pub within_limits: Option<bool>,
    pub interpretation: &'static str,
}

pub fn preservation(
    approved: &[Vec<f32>],
    candidate: &[Vec<f32>],
    rate: u32,
    regions: &[Region],
    a: &[Measurement],
    b: &[Measurement],
    limits: &Limits,
) -> Result<Preservation, String> {
    validate(approved, rate)?;
    validate(candidate, rate)?;
    limits.validate_for(regions)?;
    if approved[0].len() != candidate[0].len() {
        return Err("approved and candidate durations differ".into());
    }
    let ea = envelope(approved, rate);
    let eb = envelope(candidate, rate);
    let peak = ea.iter().map(|(_, p)| *p).fold(0.0, f64::max);
    if peak <= 1e-12 {
        return Err("approved signal has no audible AC envelope".into());
    }
    let floor = peak * 10.0f64.powf(-45.0 / 10.0);
    let (pa, pb) = ea
        .iter()
        .zip(&eb)
        .filter(|((_, p), _)| *p >= floor)
        .fold((0.0, 0.0), |(a, b), ((_, x), (_, y))| (a + x, b + y));
    if pb <= 1e-16 {
        return Err("candidate has no AC energy during approved audible frames".into());
    }
    let gain = pa / pb;
    let gain_db = 10.0 * gain.log10();
    let mut out = Vec::new();
    let deltas = differences(a, b)?;
    let whole = Region {
        id: "__whole_clip".into(),
        start: 0.0,
        seconds: approved[0].len() as f64 / f64::from(rate),
        fundamentals_hz: Vec::new(),
        modulation_hz: None,
    };
    for (index, r) in std::iter::once(&whole).chain(regions).enumerate() {
        let mut d: Vec<_> = ea
            .iter()
            .zip(&eb)
            .filter(|((t, x), (_, y))| {
                *t >= r.start && *t < r.start + r.seconds && x.max(y * gain) >= floor
            })
            .map(|((_, x), (_, y))| {
                (10.0 * (y * gain).max(1e-20).log10() - 10.0 * x.max(1e-20).log10()).abs()
            })
            .collect();
        let rms =
            (!d.is_empty()).then(|| (d.iter().map(|v| v * v).sum::<f64>() / d.len() as f64).sqrt());
        d.sort_by(f64::total_cmp);
        let p95 = (!d.is_empty()).then(|| d[((d.len() - 1) as f64 * 0.95).round() as usize]);
        let motion = index.checked_sub(1).and_then(|i| deltas.get(i));
        let am = motion.and_then(|d| d.am_depth_difference_db).map(f64::abs);
        let fm = motion
            .and_then(|d| d.fm_depth_difference_cents)
            .map(f64::abs);
        let shifts = motion.map_or_else(Vec::new, |d| d.fundamental_differences_cents.clone());
        let pitch_change = if shifts.iter().any(Option::is_none) {
            None
        } else {
            shifts.iter().flatten().map(|v| v.abs()).reduce(f64::max)
        };
        let checks = [
            (
                if index == 0 || r.fundamentals_hz.is_empty() {
                    None
                } else {
                    limits.pitch_cents
                },
                pitch_change,
            ),
            (limits.envelope_rmse_db, rms),
            (limits.envelope_p95_db, p95),
            (
                if index == 0 || r.modulation_hz.is_none() {
                    None
                } else {
                    limits.am_depth_db
                },
                am,
            ),
            (
                if index == 0 || r.modulation_hz.is_none() {
                    None
                } else {
                    limits.fm_depth_cents
                },
                fm,
            ),
        ];
        let requested: Vec<_> = checks
            .iter()
            .filter_map(|(limit, value)| limit.map(|l| value.map(|v| v <= l)))
            .collect();
        let verdict = if requested.contains(&Some(false)) {
            Some(false)
        } else if requested.is_empty() || requested.contains(&None) {
            None
        } else {
            Some(true)
        };
        out.push(PreservationRegion {
            id: r.id.clone(),
            envelope_frames: d.len(),
            requested_checks: requested.len(),
            envelope_rmse_db: rms,
            envelope_p95_db: p95,
            am_depth_change_db: am,
            fm_depth_change_cents: fm,
            fundamental_shifts_cents: shifts,
            within_limits: verdict,
        });
    }
    let checked: Vec<_> = out.iter().filter(|r| r.requested_checks > 0).collect();
    let verdict = if checked.iter().any(|r| r.within_limits == Some(false)) {
        Some(false)
    } else if checked.is_empty() || checked.iter().any(|r| r.within_limits.is_none()) {
        None
    } else {
        Some(true)
    };
    Ok(Preservation {
        candidate_gain_db: gain_db,
        limits: limits.clone(),
        regions: out,
        within_limits: verdict,
        interpretation: "One common AC-energy gain from approved audible frames;50ms AC RMS/10ms hop; union of approved/candidate audible frames includes added tails. Explicit engineering limits are not perceptual approval. Modulation needs three resolved cycles in a monophonic region; unresolved requested checks stay null.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const RATE: u32 = 48000;
    fn tone(seconds: f64, f0: f64, am: f64, fm: f64, m: f64) -> Vec<Vec<f32>> {
        let mut phase = 0.0;
        vec![
            (0..(seconds * f64::from(RATE)) as usize)
                .map(|i| {
                    let t = i as f64 / f64::from(RATE);
                    let motion = (TAU * m * t).sin();
                    phase += TAU * f0 * 2.0f64.powf(fm * motion / 1200.0) / f64::from(RATE);
                    (0.1 * 10.0f64.powf(am * motion / 20.0) * phase.sin()) as f32
                })
                .collect(),
        ]
    }
    fn region(seconds: f64) -> Region {
        Region {
            id: "held".into(),
            start: 0.0,
            seconds,
            fundamentals_hz: vec![440.0],
            modulation_hz: None,
        }
    }
    fn env_limits() -> Limits {
        Limits {
            envelope_rmse_db: Some(0.5),
            envelope_p95_db: Some(1.0),
            ..Limits::default()
        }
    }
    #[test]
    fn refuses_invalid_windows_and_unresolved_fundamentals() {
        let x = tone(1.0, 440.0, 0.0, 0.0, 4.2);
        let r = region(1.0);
        assert!(analyze(&x, RATE, &[r.clone(), r.clone()]).is_err());
        let mut bad = r.clone();
        bad.start = 0.1;
        assert!(analyze(&x, RATE, &[bad]).is_err());
        let mut bad = r;
        bad.start = f64::NAN;
        assert!(analyze(&x, RATE, &[bad]).is_err());
        let mut r = region(1024.0 / 192000.0);
        r.fundamentals_hz = vec![20.0];
        assert!(analyze(&[vec![0.1; 1024]], 192000, &[r]).is_err());
        assert!(analyze(&[vec![0.0; 48000]], RATE, &[region(1.0)]).is_err());
        for actual in [465.0, 475.0, 880.0] {
            assert!(
                analyze(&tone(1.0, actual, 0.0, 0.0, 4.2), RATE, &[region(1.0)]).is_err(),
                "out-of-search tone {actual} must not become a confident pitch from leakage"
            );
        }
    }
    #[test]
    fn refines_pitch_separately_and_preserves_stereo_chord_balance() {
        let r = region(1.0);
        let a = analyze(
            &tone(1.0, 440.0, 0.0, 0.0, 4.2),
            RATE,
            std::slice::from_ref(&r),
        )
        .expect("a");
        let x = tone(1.0, 444.0, 0.0, 0.0, 4.2);
        let stereo = vec![x[0].clone(), x[0].iter().map(|v| -v).collect()];
        let b = analyze(&stereo, RATE, &[r]).expect("b");
        let d = differences(&a, &b).expect("d");
        assert!(
            (d[0].fundamental_differences_cents[0].expect("resolved")
                - 1200.0 * (444.0f64 / 440.0).log2())
            .abs()
                < 0.1
        );
        assert!(d[0].harmonic_rmse_db.expect("harmonics") < 0.1);
        let limits = Limits {
            pitch_cents: Some(2.0),
            ..Limits::default()
        };
        assert_eq!(
            preservation(
                &tone(1.0, 440.0, 0.0, 0.0, 4.2),
                &stereo,
                RATE,
                &[region(1.0)],
                &a,
                &b,
                &limits
            )
            .expect("pitch guard")
            .within_limits,
            Some(false)
        );
        let mut chord = region(1.0);
        chord.fundamentals_hz = vec![261.6256, 329.6276, 391.9954];
        let make = |levels: [f64; 3]| {
            vec![
                (0..48000)
                    .map(|i| {
                        (levels
                            .iter()
                            .zip(&chord.fundamentals_hz)
                            .map(|(a, f)| a * (TAU * f * i as f64 / 48000.0).sin())
                            .sum::<f64>()) as f32
                    })
                    .collect(),
            ]
        };
        let a =
            analyze(&make([0.1, 0.2, 0.3]), RATE, std::slice::from_ref(&chord)).expect("chord a");
        let b =
            analyze(&make([0.3, 0.2, 0.1]), RATE, std::slice::from_ref(&chord)).expect("chord b");
        assert!(
            differences(&a, &b).expect("chord")[0]
                .harmonic_rmse_db
                .expect("harmonics")
                > 1.0
        );
        assert!(!a[0].warnings.is_empty());
    }
    #[test]
    fn identifies_tones_outside_the_harmonic_objective() {
        let a = tone(1.0, 440.0, 0.0, 0.0, 4.2);
        let mut b = a.clone();
        for (i, x) in b[0].iter_mut().enumerate() {
            *x += (0.05 * (TAU * 10000.0 * i as f64 / 48000.0).sin()) as f32;
        }
        let spec = [region(1.0)];
        let aa = analyze(&a, RATE, &spec).expect("a");
        let bb = analyze(&b, RATE, &spec).expect("b");
        let d = differences(&aa, &bb).expect("d");
        assert!(d[0].harmonic_rmse_db.expect("h") < 0.1);
        assert!(
            d[0].unexplained_peaks
                .iter()
                .any(|p| (p.hz - 10000.0).abs() < 3.0)
        );
        assert!(d[0].spectrum_rmse_db > 1.0);
    }
    #[test]
    fn calibrated_motion_and_dc_heavy_channel_selection() {
        for hz in [4.2, 10.0] {
            let x = tone(2.0, 440.0, 3.0, 20.0, hz);
            let mut r = region(2.0);
            r.modulation_hz = Some(hz);
            let a = analyze(&x, RATE, std::slice::from_ref(&r)).expect("a");
            let m = a[0].modulation.as_ref().expect("resolved");
            assert!((m.am_depth_db - 3.0).abs() < 0.15, "{m:?}");
            assert!((m.fm_depth_cents - 20.0).abs() < 0.8, "{m:?}");
            let stereo = vec![vec![10.0; 96000], x[0].iter().map(|v| *v + 2.0).collect()];
            let b = analyze(&stereo, RATE, &[r]).expect("stereo");
            let n = b[0].modulation.as_ref().expect("AC selection");
            assert!((m.am_depth_db - n.am_depth_db).abs() < 0.001);
            assert!((m.fm_depth_cents - n.fm_depth_cents).abs() < 0.01);
        }
        for hz in [20.0, 30.0] {
            let x = tone(2.0, 440.0, 3.0, 20.0, hz);
            let mut r = region(2.0);
            r.modulation_hz = Some(hz);
            assert!(
                analyze(&x, RATE, &[r]).expect("unresolved")[0]
                    .modulation
                    .is_none()
            );
        }
    }
    #[test]
    fn sideband_dominated_carriers_cannot_pass_pitch_or_modulation_checks() {
        for (depth, am) in [(50.0, 0.0), (100.0, 0.0), (50.0, 3.0), (100.0, 3.0)] {
            let x = tone(2.0, 440.0, am, depth, 4.2);
            let mut r = region(2.0);
            r.modulation_hz = Some(4.2);
            let specs = [r];
            let a = analyze(&x, RATE, &specs).expect("spectral measurements still useful");
            assert!(a[0].fundamental_estimates_hz[0].is_none(), "{depth}: {a:?}");
            assert!(a[0].modulation.is_none());
            assert!(!a[0].harmonics.is_empty());
            let limits = Limits {
                pitch_cents: Some(1.0),
                fm_depth_cents: Some(1.0),
                ..Limits::default()
            };
            assert_eq!(
                preservation(&x, &x, RATE, &specs, &a, &a, &limits)
                    .expect("uncertain even for identical clips")
                    .within_limits,
                None
            );
        }
    }
    #[test]
    fn expression_uses_common_gain_but_separate_short_note_checks() {
        let a = tone(4.0, 440.0, 0.0, 0.0, 4.2);
        let mut b = a.clone();
        for (i, x) in b[0].iter_mut().enumerate() {
            if (168000..175200).contains(&i) {
                *x *= 0.5;
            }
        }
        let mut short = region(0.15);
        short.id = "short".into();
        short.start = 3.5;
        let specs = [region(4.0), short];
        let aa = analyze(&a, RATE, &specs).expect("a");
        let bb = analyze(&b, RATE, &specs).expect("b");
        let p = preservation(&a, &b, RATE, &specs, &aa, &bb, &env_limits()).expect("preserve");
        assert_eq!(p.within_limits, Some(false));
        assert_eq!(p.regions[2].within_limits, Some(false));
        let scaled = vec![a[0].iter().map(|v| v * 0.5 + 3.0).collect()];
        let cc = analyze(&scaled, RATE, &specs).expect("dc");
        let p = preservation(&a, &scaled, RATE, &specs, &aa, &cc, &env_limits())
            .expect("gain invariant");
        assert_eq!(p.within_limits, Some(true));
        assert!(p.regions[0].envelope_rmse_db.expect("rms") < 0.001);
    }
    #[test]
    fn added_tails_and_unresolved_requested_checks_do_not_pass() {
        let mut a = tone(2.0, 440.0, 0.0, 0.0, 4.2);
        for v in &mut a[0][48000..] {
            *v = 0.0;
        }
        let b = tone(2.0, 440.0, 0.0, 0.0, 4.2);
        let specs = [region(0.8)];
        let aa = analyze(&a, RATE, &specs).expect("a");
        let bb = analyze(&b, RATE, &specs).expect("b");
        assert_eq!(
            preservation(&a, &b, RATE, &specs, &aa, &bb, &env_limits())
                .expect("tail")
                .within_limits,
            Some(false)
        );
        let mut r = region(0.2);
        r.modulation_hz = Some(4.2);
        let specs = [r];
        let aa = analyze(&a, RATE, &specs).expect("short");
        let limits = Limits {
            am_depth_db: Some(0.1),
            ..env_limits()
        };
        assert_eq!(
            preservation(&a, &a, RATE, &specs, &aa, &aa, &limits)
                .expect("unresolved")
                .within_limits,
            None
        );
        assert!(preservation(&a, &a, RATE, &[region(0.8)], &aa, &aa, &limits).is_err());
    }
}
