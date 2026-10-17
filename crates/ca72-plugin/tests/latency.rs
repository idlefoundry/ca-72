//! Each QUALITY mode's latency, measured (decisions.md R-LATENCY): a note's onset, its
//! waveform's lag behind ULTRA's, and an impulse's through EXTERNAL INPUT.
//!
//! By hand: `cargo test --release -p ca72-plugin --test latency -- --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

use ca72::tuning::Range;
use ca72::voice::{Quality, Voice, Waveform};
use ca72_plugin::engine::{Controls, Engine, Event};

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

/// One sawtooth, 32', through the filter wide open; the contours' attack as short as it goes
/// and their sustain full; no noise, no external input unless `ext`.
fn patch(mode: &str, ext: bool) -> Controls {
    let mut c = Controls::default();
    c.potato = mode == "LO";
    c.ultra = mode == "ULTRA";
    c.entropy = 0.0;
    c.lock = true;
    let p = &mut c.panel;
    for o in &mut p.osc {
        o.on = false;
    }
    if !ext {
        p.osc[0].on = true;
        p.osc[0].volume = 1.0;
        p.osc[0].waveform = Waveform::Sawtooth;
        p.osc[0].range = Range::R32;
    }
    p.noise_on = false;
    p.ext_on = ext;
    p.ext_volume = 1.0;
    p.cutoff = 1.0;
    p.emphasis = 0.0;
    p.contour_amount = 0.0;
    p.filter_contour.attack = 0.0;
    p.filter_contour.sustain = 1.0;
    p.loudness_contour.attack = 0.0;
    p.loudness_contour.sustain = 1.0;
    c
}

/// `seconds` of the engine's left output with a key pressed at `at` (samples) and held, and
/// `ext` at EXTERNAL INPUT.
fn play(c: &Controls, at: usize, seconds: f64, ext: &[f32]) -> Vec<f64> {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, 1);
    let n = (seconds * RATE) as usize / BLOCK * BLOCK;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    for b in (0..n).step_by(BLOCK) {
        let x: &[f32] = if ext.len() >= b + BLOCK {
            &ext[b..b + BLOCK]
        } else {
            &[]
        };
        if (b..b + BLOCK).contains(&at) {
            let k = at - b;
            let (x0, x1) = x.split_at(k.min(x.len()));
            e.render(x0, &mut l[b..at], &mut r[b..at]);
            e.event(Event::Note { key: 33, on: true });
            e.render(x1, &mut l[at..b + BLOCK], &mut r[at..b + BLOCK]);
        } else {
            e.render(x, &mut l[b..b + BLOCK], &mut r[b..b + BLOCK]);
        }
        e.end_block(BLOCK);
    }
    l.iter().map(|&v| f64::from(v)).collect()
}

/// The lag (samples, fractional) at which `b` best matches `a` delayed, within `reach`.
fn lag(a: &[f64], b: &[f64], reach: isize) -> f64 {
    let n = a.len() as isize;
    let score = |d: isize| -> f64 {
        let mut s = 0.0;
        for i in reach..n - reach {
            s += a[i as usize] * b[(i + d) as usize];
        }
        s
    };
    let scores: Vec<f64> = (-reach..=reach).map(score).collect();
    let k = (0..scores.len())
        .max_by(|&i, &j| scores[i].total_cmp(&scores[j]))
        .unwrap();
    let (y0, y1, y2) = (
        scores[k.saturating_sub(1)],
        scores[k],
        scores[(k + 1).min(scores.len() - 1)],
    );
    let frac = 0.5 * (y0 - y2) / (y0 - 2.0 * y1 + y2);
    k as f64 - reach as f64 + if frac.is_finite() { frac } else { 0.0 }
}

/// The first sample at which `x`'s magnitude reaches `share` of its peak after `from`.
fn onset(x: &[f64], from: usize, share: f64) -> f64 {
    let peak = x[from..].iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let i = (from..x.len()).find(|&i| x[i].abs() >= share * peak).unwrap();
    // (Between the samples either side of the crossing.)
    let (a, b) = (x[i - 1].abs(), x[i].abs());
    (i - 1) as f64 + (share * peak - a) / (b - a)
}

#[test]
#[ignore = "a measurement, run by hand"]
fn each_modes_latency() {
    let at = (0.25 * RATE) as usize;
    let modes = ["LO", "HI", "ULTRA"];
    let notes: Vec<Vec<f64>> = modes
        .iter()
        .map(|m| play(&patch(m, false), at, 1.5, &[]))
        .collect();
    // Onsets: a tenth and a half of the peak.
    for (m, x) in modes.iter().zip(&notes) {
        eprintln!(
            "{m:<6} onset at 10 % {:.2}, at 50 % {:.2} samples after the key",
            onset(x, at, 0.1) - at as f64,
            onset(x, at, 0.5) - at as f64
        );
    }
    // The waveform's lag behind ULTRA's, the note sustained.
    let seg = |x: &[f64]| x[(0.75 * RATE) as usize..(1.45 * RATE) as usize].to_vec();
    let u = seg(&notes[2]);
    for (m, x) in modes.iter().zip(&notes) {
        eprintln!(
            "{m:<6} waveform {:.2} samples behind ULTRA's",
            -lag(&seg(x), &u, 300)
        );
    }
    // An impulse at EXTERNAL INPUT a second in, a key held (the VCA open).
    let mut ext = vec![0.0f32; (1.5 * RATE) as usize];
    let t1 = RATE as usize;
    ext[t1] = 0.5;
    for m in modes {
        let y = play(&patch(m, true), at, 1.5, &ext);
        let w = &y[t1 - 200..t1 + 800];
        let k = (0..w.len())
            .max_by(|&i, &j| w[i].abs().total_cmp(&w[j].abs()))
            .unwrap();
        let (y0, y1, y2) = (w[k - 1].abs(), w[k].abs(), w[k + 1].abs());
        let peak = k as f64 - 200.0 + 0.5 * (y0 - y2) / (y0 - 2.0 * y1 + y2);
        eprintln!("{m:<6} EXTERNAL INPUT's impulse peaks {peak:.2} samples later");
    }
}

/// The circuit's oscillator path at a quality: oscillator 1's sawtooth (16') through the
/// filter wide open, a key held; each of its resets' exact time (the oscillator's own
/// account) against where the output's jump is steepest. The mean over the resets from half
/// a second to a second and a half: samples.
fn oscillator_path(q: Quality) -> f64 {
    let mut c = patch("ULTRA", false);
    c.panel.osc[0].range = Range::R16;
    let mut v = Voice::new(RATE, c.panel);
    v.panel.quality = q;
    v.note(57, true);
    let n = (1.6 * RATE) as usize;
    let mut y = Vec::with_capacity(n);
    let mut resets = Vec::new();
    let mut last = v.probe_resets(0).1;
    for i in 0..n {
        y.push(v.tick());
        let (t, count) = v.probe_resets(0);
        if count != last {
            last = count;
            // (The reset fell within this sample's tick: `t` - ceil(`t`) of a sample before.)
            resets.push(i as f64 + (t - t.ceil()));
        }
    }
    let mut sum = 0.0;
    let mut k = 0;
    for &r in resets
        .iter()
        .filter(|&&r| r > 0.5 * RATE && r < 1.5 * RATE)
    {
        // The jump's steepest point within 200 samples after the reset (the derivative's
        // peak between samples, interpolated).
        let from = r as usize;
        let d: Vec<f64> = (from..from + 200).map(|i| y[i + 1] - y[i]).collect();
        let j = (1..d.len() - 1)
            .max_by(|&a, &b| d[a].abs().total_cmp(&d[b].abs()))
            .unwrap();
        let (a, b, c) = (d[j - 1].abs(), d[j].abs(), d[j + 1].abs());
        let at = (from + j) as f64 + 0.5 + 0.5 * (a - c) / (a - 2.0 * b + c);
        sum += at - r;
        k += 1;
    }
    sum / k as f64
}

#[test]
#[ignore = "a measurement, run by hand"]
fn each_modes_oscillator_path_and_onset() {
    for (name, q) in [("HI", Quality::Potato), ("ULTRA", Quality::NoCompromises)] {
        eprintln!(
            "{name:<6} the oscillators' jumps reach the output {:.2} samples after them",
            oscillator_path(q)
        );
    }
    // Onsets: an A5 on 2', its envelope (the RMS over a period) reaching a tenth and half
    // of its held level.
    let at = (0.25 * RATE) as usize;
    for m in ["LO", "HI", "ULTRA"] {
        let mut c = patch(m, false);
        c.panel.osc[0].range = Range::R2;
        let x = play(&c, at, 1.0, &[]);
        let w = 55;
        let env: Vec<f64> = (0..x.len() - w)
            .map(|i| (x[i..i + w].iter().map(|v| v * v).sum::<f64>() / w as f64).sqrt())
            .collect();
        let held = env[(0.8 * RATE) as usize];
        let cross = |share: f64| {
            let i = (at - w..env.len())
                .find(|&i| env[i] >= share * held)
                .unwrap();
            let (a, b) = (env[i - 1], env[i]);
            // (The window's centre.)
            (i - 1) as f64 + (share * held - a) / (b - a) + w as f64 / 2.0 - at as f64
        };
        eprintln!(
            "{m:<6} envelope at 10 % {:.1}, 50 % {:.1}, 90 % {:.1} samples after the key",
            cross(0.1),
            cross(0.5),
            cross(0.9)
        );
    }
}
