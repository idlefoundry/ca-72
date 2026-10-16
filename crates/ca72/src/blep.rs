//! The lab's band-limiting candidates (decisions.md R-BLEP; a prototype, not the plug-in's
//! sound): band-limited steps (BLEP), impulses and ramps (BLAMP) from one kernel each, as
//! tables, for [`crate::resample::BlepLine`] (the circuit's oscillators) and the light
//! model's oscillators.
//!
//! Nothing here is used unless the lab asks for it: by [`set`], or by the environment's
//! `CA72_BLEP` (read once), so that the plug-in's own tests can measure a candidate's cost:
//! `poly2`, `bspline4`, `lagrange4`, `sinc8`, `sinc16`, `sinc32`, `min8`, `min16`, `min32`,
//! each with `+blamp` for the ramps' corrections (`blamp` alone: `poly2+blamp`), and
//! `:b=<beta>` and `:fc=<cutoff, cycles a sample>` for the windowed sincs.

use std::sync::{Arc, Mutex};

/// A band-limiting kernel: the impulse the steps, impulses and corners are filtered by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kernel {
    /// Two-point polyBLEP: a triangle of two samples (the plug-in's today).
    PolyBlep2,
    /// Four-point polyBLEP from the cubic B-spline (smooth, low aliasing, a duller top).
    BSpline4,
    /// Four-point polyBLEP from cubic Lagrange interpolation (a flatter top, more aliasing).
    Lagrange4,
    /// A Kaiser-windowed sinc of `taps` samples, linear phase (half its length late).
    Sinc { taps: usize, beta: f64, fc: f64 },
    /// The same sinc made minimum phase (minBLEP): no latency, its ringing after the edge.
    MinSinc { taps: usize, beta: f64, fc: f64 },
}

/// What the lab asked for: a kernel, and whether the ramps' corners are corrected too.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub kernel: Kernel,
    pub blamp: bool,
}

impl Lab {
    /// Parses `CA72_BLEP`'s form (see the module's notes).
    pub fn parse(s: &str) -> Result<Lab, String> {
        let s = s.trim();
        let (body, blamp) = match s.strip_suffix("+blamp") {
            Some(b) => (b, true),
            None if s == "blamp" => ("poly2", true),
            None => (s, false),
        };
        let mut parts = body.split(':');
        let name = parts.next().unwrap_or("");
        let mut beta = None;
        let mut fc = None;
        for p in parts {
            if let Some(v) = p.strip_prefix("b=") {
                beta = Some(v.parse::<f64>().map_err(|e| format!("{p}: {e}"))?);
            } else if let Some(v) = p.strip_prefix("fc=") {
                fc = Some(v.parse::<f64>().map_err(|e| format!("{p}: {e}"))?);
            } else {
                return Err(format!("CA72_BLEP: unknown option {p}"));
            }
        }
        let sinc = |taps: usize| {
            let (b, f) = default_window(taps);
            (taps, beta.unwrap_or(b), fc.unwrap_or(f))
        };
        let kernel = match name {
            "poly2" => Kernel::PolyBlep2,
            "bspline4" => Kernel::BSpline4,
            "lagrange4" => Kernel::Lagrange4,
            _ => {
                if let Some(t) = name.strip_prefix("sinc") {
                    let (taps, beta, fc) = sinc(t.parse().map_err(|e| format!("{name}: {e}"))?);
                    Kernel::Sinc { taps, beta, fc }
                } else if let Some(t) = name.strip_prefix("min") {
                    let (taps, beta, fc) = sinc(t.parse().map_err(|e| format!("{name}: {e}"))?);
                    Kernel::MinSinc { taps, beta, fc }
                } else {
                    return Err(format!("CA72_BLEP: unknown kernel {name}"));
                }
            }
        };
        if let Kernel::Sinc { taps, .. } | Kernel::MinSinc { taps, .. } = kernel
            && !(4..=48).contains(&taps)
        {
            return Err(format!("CA72_BLEP: {taps} taps (4 to 48)"));
        }
        Ok(Lab { kernel, blamp })
    }
}

/// The Kaiser window's beta and the cutoff (cycles a sample) for each length, chosen by the
/// lab's sweep (decisions.md R-BLEP): the least aliasing of the sawtooth from 20 Hz to
/// 20 kHz at 44.1 and 48 kHz (A4 to A7) with every harmonic below 20 kHz within about a
/// decibel of the oscillator's at 8x.
pub fn default_window(taps: usize) -> (f64, f64) {
    match taps {
        0..=8 => (1.0, 0.5),
        9..=16 => (4.0, 0.5),
        _ => (8.0, 0.48),
    }
}

static CHOSEN: Mutex<Option<Option<Lab>>> = Mutex::new(None);

/// Sets the lab's choice for oscillators made from now on (`None`: the plug-in's own).
pub fn set(lab: Option<Lab>) {
    if let Ok(mut c) = CHOSEN.lock() {
        *c = Some(lab);
    }
}

/// The lab's choice: [`set`]'s, else `CA72_BLEP`'s, else none (the plug-in's own).
pub fn chosen() -> Option<Lab> {
    let Ok(mut c) = CHOSEN.lock() else {
        return None;
    };
    if let Some(l) = *c {
        return l;
    }
    let l = std::env::var("CA72_BLEP")
        .ok()
        .map(|s| match Lab::parse(&s) {
            Ok(l) => l,
            Err(e) => panic!("{e}"),
        });
    *c = Some(l);
    l
}

/// Points a sample in the tables.
const P: usize = 256;

/// A kernel's tables over `x` (samples from the event to the sample corrected) in
/// `[x0, x0 + span]`: the impulse, the band-limited step (0 to 1) and its integral (the
/// band-limited ramp, 0 to its slope's 1 a sample).
#[derive(Debug)]
pub struct Tables {
    pub kernel: Kernel,
    pub x0: f64,
    pub span: usize,
    imp: Vec<f64>,
    step: Vec<f64>,
    ramp: Vec<f64>,
    /// The ramp's correction integrated from `x0` (for corners spread over a width, a
    /// fillet), and its total.
    ramp_sum: Vec<f64>,
    ramp_total: f64,
    /// How many output samples a corrected line holds back (linear phase: half the span).
    pub delay: usize,
}

impl Tables {
    /// The tables for a kernel (made once a process for each).
    pub fn of(kernel: Kernel) -> Arc<Tables> {
        static MADE: Mutex<Vec<Arc<Tables>>> = Mutex::new(Vec::new());
        let Ok(mut made) = MADE.lock() else {
            return Arc::new(Tables::new(kernel));
        };
        if let Some(t) = made.iter().find(|t| t.kernel == kernel) {
            return t.clone();
        }
        let t = Arc::new(Tables::new(kernel));
        made.push(t.clone());
        t
    }

    fn new(kernel: Kernel) -> Tables {
        // The impulse at G points a sample, integrated there and kept at P.
        const G: usize = 4 * P;
        let (x0, span, h): (f64, usize, Vec<f64>) = match kernel {
            Kernel::PolyBlep2 => (-1.0, 2, sampled(-1.0, 2, G, |x| (1.0 - x.abs()).max(0.0))),
            Kernel::BSpline4 => (-2.0, 4, sampled(-2.0, 4, G, bspline3)),
            Kernel::Lagrange4 => (-2.0, 4, sampled(-2.0, 4, G, lagrange3)),
            Kernel::Sinc { taps, beta, fc } => {
                let k = taps as f64 / 2.0;
                (
                    -k,
                    taps,
                    sampled(-k, taps, G, |x| windowed_sinc(x, k, beta, fc)),
                )
            }
            Kernel::MinSinc { taps, beta, fc } => {
                let k = taps as f64 / 2.0;
                let lin = sampled(-k, taps, G, |x| windowed_sinc(x, k, beta, fc));
                (0.0, taps, minimum_phase(&lin, taps, G))
            }
        };
        let g = G as f64;
        let area: f64 = trapezoid_total(&h) / g;
        let h: Vec<f64> = h.iter().map(|v| v / area).collect();
        // Cumulative integrals (trapezoid at G a sample).
        let mut step = vec![0.0; h.len()];
        for i in 1..h.len() {
            step[i] = step[i - 1] + 0.5 * (h[i - 1] + h[i]) / g;
        }
        let mut ramp = vec![0.0; h.len()];
        for i in 1..h.len() {
            ramp[i] = ramp[i - 1] + 0.5 * (step[i - 1] + step[i]) / g;
        }
        let keep = |v: &[f64]| (0..=span * P).map(|i| v[i * (G / P)]).collect::<Vec<f64>>();
        let delay = if x0 < 0.0 { (-x0) as usize } else { 0 };
        let mut t = Tables {
            kernel,
            x0,
            span,
            imp: keep(&h),
            step: keep(&step),
            ramp: keep(&ramp),
            ramp_sum: Vec::new(),
            ramp_total: 0.0,
            delay,
        };
        // The ramp's correction's integral, at G a sample.
        let mut sum = vec![0.0; span * G + 1];
        let mut prev = t.ramp_residual(x0);
        for i in 1..sum.len() {
            let r = t.ramp_residual(x0 + i as f64 / g);
            sum[i] = sum[i - 1] + 0.5 * (prev + r) / g;
            prev = r;
        }
        t.ramp_total = sum[sum.len() - 1];
        t.ramp_sum = keep(&sum);
        t
    }

    #[inline]
    fn at(&self, v: &[f64], x: f64) -> f64 {
        let u = (x - self.x0) * P as f64;
        let i = u as usize;
        if u < 0.0 || i >= self.span * P {
            return v[if u < 0.0 { 0 } else { self.span * P }];
        }
        let f = u - i as f64;
        v[i] + (v[i + 1] - v[i]) * f
    }

    /// The step's correction at `x` samples after it: the band-limited step less the step.
    #[inline]
    pub fn step_residual(&self, x: f64) -> f64 {
        if x <= self.x0 || x >= self.x0 + self.span as f64 {
            return 0.0;
        }
        self.at(&self.step, x) - if x >= 0.0 { 1.0 } else { 0.0 }
    }

    /// The impulse's value at `x` (unit area).
    #[inline]
    pub fn impulse(&self, x: f64) -> f64 {
        if x <= self.x0 || x >= self.x0 + self.span as f64 {
            return 0.0;
        }
        self.at(&self.imp, x)
    }

    /// A corner's correction at `x` for a slope that grows by one a sample: the band-limited
    /// ramp less the ramp. (A minimum-phase kernel's ramp lags by its delay; its correction
    /// is made to end at zero by a tail fading it out, which a linear-phase kernel's needs
    /// not.)
    #[inline]
    pub fn ramp_residual(&self, x: f64) -> f64 {
        if x <= self.x0 || x >= self.x0 + self.span as f64 {
            return 0.0;
        }
        let r = self.at(&self.ramp, x) - x.max(0.0);
        if self.x0 < 0.0 {
            r
        } else {
            // (Minimum phase: faded out over the last half of the span.)
            let end = self.span as f64;
            let fade = ((end - x) / (0.5 * end)).clamp(0.0, 1.0);
            let r_end = self.at(&self.ramp, end - 1e-9) - (end - 1e-9);
            r - r_end * (1.0 - fade)
        }
    }
}

impl Tables {
    /// The ramp's correction integrated from `x0` to `x`.
    #[inline]
    fn ramp_integral(&self, x: f64) -> f64 {
        if x <= self.x0 {
            0.0
        } else if x >= self.x0 + self.span as f64 {
            self.ramp_total
        } else {
            self.at(&self.ramp_sum, x)
        }
    }

    /// A fillet's correction at `x`: the slope grows by one a sample evenly over `w`
    /// samples from `x` = 0 (the ramp's correction averaged over the width).
    #[inline]
    pub fn fillet_residual(&self, x: f64, w: f64) -> f64 {
        if w < 1e-3 {
            return self.ramp_residual(x);
        }
        (self.ramp_integral(x) - self.ramp_integral(x - w)) / w
    }
}

fn sampled(x0: f64, span: usize, g: usize, f: impl Fn(f64) -> f64) -> Vec<f64> {
    (0..=span * g)
        .map(|i| f(x0 + i as f64 / g as f64))
        .collect()
}

fn trapezoid_total(h: &[f64]) -> f64 {
    let s: f64 = h.iter().sum();
    s - 0.5 * (h[0] + h[h.len() - 1])
}

/// The cubic B-spline (support of four samples).
fn bspline3(x: f64) -> f64 {
    let a = x.abs();
    if a < 1.0 {
        (4.0 - 6.0 * a * a + 3.0 * a * a * a) / 6.0
    } else if a < 2.0 {
        (2.0 - a).powi(3) / 6.0
    } else {
        0.0
    }
}

/// Cubic Lagrange interpolation's kernel (support of four samples).
fn lagrange3(x: f64) -> f64 {
    let a = x.abs();
    if a < 1.0 {
        (a + 1.0) * (a - 1.0) * (a - 2.0) / 2.0
    } else if a < 2.0 {
        -(a - 1.0) * (a - 2.0) * (a - 3.0) / 6.0
    } else {
        0.0
    }
}

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let y = x * x / 4.0;
    for k in 1..80 {
        term *= y / (k as f64 * k as f64);
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

/// A low-pass at `fc` cycles a sample, Kaiser-windowed to half-width `k` samples.
fn windowed_sinc(x: f64, k: f64, beta: f64, fc: f64) -> f64 {
    let r = x / k;
    if r.abs() >= 1.0 {
        return 0.0;
    }
    let w = bessel_i0(beta * (1.0 - r * r).sqrt()) / bessel_i0(beta);
    let t = 2.0 * fc * x;
    let s = if t.abs() < 1e-12 {
        1.0
    } else {
        (std::f64::consts::PI * t).sin() / (std::f64::consts::PI * t)
    };
    2.0 * fc * s * w
}

/// The minimum-phase impulse with `h`'s magnitude (the real cepstrum, folded), cut to
/// `taps` samples at `g` points a sample with a short fade at its end.
fn minimum_phase(h: &[f64], taps: usize, g: usize) -> Vec<f64> {
    let n = (h.len() * 16).next_power_of_two();
    let mut re = vec![0.0; n];
    let mut im = vec![0.0; n];
    re[..h.len()].copy_from_slice(h);
    fft(&mut re, &mut im, false);
    let peak = re
        .iter()
        .zip(&im)
        .map(|(a, b)| (a * a + b * b).sqrt())
        .fold(0.0f64, f64::max);
    for i in 0..n {
        let m = (re[i] * re[i] + im[i] * im[i]).sqrt().max(peak * 1e-10);
        re[i] = m.ln();
        im[i] = 0.0;
    }
    fft(&mut re, &mut im, true);
    // Fold: keep c[0] and c[n/2], double 1..n/2, drop the rest.
    for i in 1..n / 2 {
        re[i] *= 2.0;
        im[i] *= 2.0;
    }
    for i in n / 2 + 1..n {
        re[i] = 0.0;
        im[i] = 0.0;
    }
    fft(&mut re, &mut im, false);
    for i in 0..n {
        let (a, b) = (re[i].exp(), im[i]);
        re[i] = a * b.cos();
        im[i] = a * b.sin();
    }
    fft(&mut re, &mut im, true);
    let len = taps * g + 1;
    let fade_from = len - g; // the last sample faded out
    (0..len)
        .map(|i| {
            let w = if i < fade_from {
                1.0
            } else {
                let t = (i - fade_from) as f64 / g as f64;
                0.5 + 0.5 * (std::f64::consts::PI * t).cos()
            };
            re[i] * w
        })
        .collect()
}

/// In-place radix-2 FFT (`inverse`: scaled by 1/n).
fn fft(re: &mut [f64], im: &mut [f64], inverse: bool) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let ang = sign * 2.0 * std::f64::consts::PI / len as f64;
        let (wr0, wi0) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut wr, mut wi) = (1.0, 0.0);
            for k in 0..half {
                let (a, b) = (start + k, start + k + half);
                let tr = re[b] * wr - im[b] * wi;
                let ti = re[b] * wi + im[b] * wr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let w = wr * wr0 - wi * wi0;
                wi = wr * wi0 + wi * wr0;
                wr = w;
            }
        }
        len <<= 1;
    }
    if inverse {
        let s = 1.0 / n as f64;
        re.iter_mut().for_each(|v| *v *= s);
        im.iter_mut().for_each(|v| *v *= s);
    }
}

/// Ring of the samples a line holds back (more than any kernel's span and fillet).
const RING: usize = 128;

/// The widest fillet corrected, samples.
pub const MAX_FILLET: f64 = 24.0;

/// A signal corrected by a kernel's tables: steps, impulses and corners at sub-sample
/// times, its output [`Tables::delay`] samples late.
#[derive(Debug, Clone)]
pub struct Line {
    pub tables: Arc<Tables>,
    ring: [f64; RING],
    /// The coming sample's place in the ring.
    c: usize,
}

impl Line {
    pub fn new(tables: Arc<Tables>) -> Line {
        Line {
            tables,
            ring: [0.0; RING],
            c: 0,
        }
    }

    pub fn clear(&mut self) {
        self.ring = [0.0; RING];
        self.c = 0;
    }

    /// Adds `f(x)` to each sample the kernel reaches, `x` its distance in samples from an
    /// event at `frac` of the way from the previous sample to the coming one.
    #[inline]
    fn spread(&mut self, frac: f64, f: impl Fn(&Tables, f64) -> f64) {
        let t = &*self.tables;
        let d = (1.0 - frac).clamp(0.0, 1.0);
        let lo = (t.x0 - d).ceil() as isize;
        let hi = (t.x0 + t.span as f64 - d).floor() as isize;
        for j in lo..=hi {
            let x = j as f64 + d;
            let i = (self.c as isize + j).rem_euclid(RING as isize) as usize;
            self.ring[i] += f(t, x);
        }
    }

    pub fn step(&mut self, delta: f64, frac: f64) {
        self.spread(frac, |t, x| delta * t.step_residual(x));
    }

    pub fn impulse(&mut self, area: f64, frac: f64) {
        self.spread(frac, |t, x| area * t.impulse(x));
    }

    /// A corner: the slope grows by `slope` (value a sample) at `frac`.
    pub fn ramp(&mut self, slope: f64, frac: f64) {
        self.spread(frac, |t, x| slope * t.ramp_residual(x));
    }

    /// A rounded corner: the slope grows by `slope` evenly over `width` samples from
    /// `frac` (at most [`MAX_FILLET`]; a wider one is left as it is sampled).
    pub fn fillet(&mut self, slope: f64, frac: f64, width: f64) {
        if width > MAX_FILLET {
            return;
        }
        let t = &*self.tables;
        let d = (1.0 - frac).clamp(0.0, 1.0);
        let lo = (t.x0 - d).ceil() as isize;
        let hi = (t.x0 + t.span as f64 + width - d).floor() as isize;
        for j in lo..=hi {
            let x = j as f64 + d;
            let i = (self.c as isize + j).rem_euclid(RING as isize) as usize;
            self.ring[i] += slope * t.fillet_residual(x, width);
        }
    }

    #[inline]
    pub fn push(&mut self, naive: f64) -> f64 {
        self.ring[self.c] += naive;
        let o = (self.c + RING - self.tables.delay) % RING;
        let out = self.ring[o];
        self.ring[o] = 0.0;
        self.c = (self.c + 1) % RING;
        out
    }
}

/// The light model's corrections, worked out from its phase (no samples held back): the
/// sum over every edge (phase `at`, every cycle) within the kernel's reach of `f(x)`, `x`
/// the sample's distance after it in samples, for a phase `p` and increment `dt` a sample.
#[inline]
pub fn around(t: &Tables, p: f32, at: f32, inv: f32, f: impl Fn(&Tables, f64) -> f64) -> f32 {
    let mut q = p - at;
    if q < 0.0 {
        q += 1.0;
    }
    let (q, inv) = (f64::from(q), f64::from(inv));
    let (lo, hi) = (t.x0, t.x0 + t.span as f64);
    let mut sum = 0.0;
    // Edges passed (x >= 0 for k >= 0) and to come (k < 0).
    let mut k = 0.0;
    loop {
        let x = (q + k) * inv;
        if x >= hi {
            break;
        }
        if x > lo {
            sum += f(t, x);
        }
        k += 1.0;
    }
    let mut k = -1.0;
    loop {
        let x = (q + k) * inv;
        if x <= lo {
            break;
        }
        if x < hi {
            sum += f(t, x);
        }
        k -= 1.0;
    }
    sum as f32
}

/// The tables the lab chose, for the light model's oscillators (which are `Copy`: made once
/// a process for each kernel and kept), and whether its corners are corrected.
pub fn light() -> Option<(&'static Tables, bool)> {
    static KEPT: Mutex<Vec<&'static Tables>> = Mutex::new(Vec::new());
    let lab = chosen()?;
    let mut kept = KEPT.lock().ok()?;
    if let Some(t) = kept.iter().find(|t| t.kernel == lab.kernel) {
        return Some((*t, lab.blamp));
    }
    let t: &'static Tables = Box::leak(Box::new(Tables::new(lab.kernel)));
    kept.push(t);
    Some((t, lab.blamp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poly2_tables_match_the_closed_form() {
        let t = Tables::new(Kernel::PolyBlep2);
        for i in 1..40 {
            let d = i as f64 / 40.0;
            // The sample before the step (x = d - 1) and the one after (x = d).
            assert!((t.step_residual(d - 1.0) - d * d * 0.5).abs() < 1e-5);
            assert!((t.step_residual(d) + (1.0 - d) * (1.0 - d) * 0.5).abs() < 1e-5);
            assert!((t.ramp_residual(d - 1.0) - d * d * d / 6.0).abs() < 1e-5);
            assert!((t.ramp_residual(d) - (1.0 - d).powi(3) / 6.0).abs() < 1e-5);
        }
    }

    #[test]
    fn every_kernels_step_ends_at_one() {
        for k in [
            Kernel::BSpline4,
            Kernel::Lagrange4,
            Kernel::Sinc {
                taps: 16,
                beta: 7.0,
                fc: 0.5,
            },
            Kernel::MinSinc {
                taps: 16,
                beta: 7.0,
                fc: 0.5,
            },
        ] {
            let t = Tables::new(k);
            let end = t.x0 + t.span as f64 - 1e-6;
            assert!(t.step_residual(end).abs() < 1e-4, "{k:?}");
            assert!(t.ramp_residual(end).abs() < 1e-3, "{k:?}");
        }
    }
}
