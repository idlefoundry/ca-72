//! CUTOFF swept at strong EMPHASIS, the voices told its value every sample (the knob's own
//! continuous turn, the reference) or in steps: every 32 samples (a learned controller's glide,
//! `learn::DEZIP_STEP`), once a host block of 64 to 1024 frames (the editor's and automation's
//! changes reach the voices at a block's start), and as a mouse moves it, a new value 60 times
//! a second taken at the next 256-frame block (issue #107). Prints each stepped render's
//! difference from the reference (its level against the reference's, and the share of it above
//! 2 kHz), and the same for a 10 ms linear glide to each new value in 32-sample steps (as
//! `learn::Dezip` glides a learned knob), for comparison.
//!
//! By hand: `cargo test --release -p ca72-plugin --test cutoff_steps -- --ignored --nocapture`
//! (`CA72_CRACKLE_OUT=<dir>` writes the renders there as WAVs; `CA72_EMPHASIS`, 0 to 10, 9).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::fft::{hann, power};
use ca72_plugin::engine::{Controls, Engine, Event};
use common::{controls_of, env, preset};
use std::path::PathBuf;

const RATE: f64 = 48_000.0;
const SECONDS: f64 = 3.0;

/// CUTOFF's travel (0..1) at sample `n`: up from 25 to 75 % over the first two seconds, held.
fn cutoff_at(n: usize) -> f64 {
    let t = (n as f64 / RATE / 2.0).min(1.0);
    0.25 + 0.5 * t
}

/// How the voices are told CUTOFF: at the start of every `apply` samples (a host's block),
/// the value the sweep had at the last of its events (one every `events` samples: 1, the knob's
/// value at the block's start), reached at once, or with `glide` (samples) by a linear glide
/// from where the voices had it, or with `pole` (samples, the time constant) by a one-pole
/// glide, in steps of 32 samples.
struct Steps {
    apply: usize,
    events: usize,
    glide: usize,
    pole: f64,
}

/// The sweep played with `base`'s panel, one sample at a time (the one instrument at HI plays
/// so however its host's blocks fall: `Engine::render`), CUTOFF told the voices as `s` has it.
fn render(base: &Controls, s: &Steps) -> (Vec<f32>, Vec<f32>) {
    let mut c = *base;
    c.panel.cutoff = cutoff_at(0);
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(RATE, 1);
    e.event(Event::Note { key: 48, on: true });
    let n = (SECONDS * RATE) as usize;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let (mut from, mut to, mut since) = (c.panel.cutoff, c.panel.cutoff, 0usize);
    for k in 0..n {
        if k % s.apply == 0 {
            from = c.panel.cutoff;
            to = cutoff_at(k / s.events * s.events);
            since = 0;
        }
        let now = if s.pole > 0.0 {
            if k % 32 == 0 {
                c.panel.cutoff + (to - c.panel.cutoff) * (1.0 - (-32.0 / s.pole).exp())
            } else {
                c.panel.cutoff
            }
        } else if s.glide == 0 || since >= s.glide {
            to
        } else {
            from + (to - from) * (since / 32 * 32) as f64 / s.glide as f64
        };
        if c.panel.cutoff != now {
            c.panel.cutoff = now;
            e.set(&c);
        }
        e.render(&[], &mut l[k..=k], &mut r[k..=k]);
        since += 1;
    }
    (l, r)
}

fn rms(x: &[f64]) -> f64 {
    (x.iter().map(|v| v * v).sum::<f64>() / x.len().max(1) as f64).sqrt()
}

/// The share of `x`'s power above `hz` (Hann windows of 4096, half overlapping).
fn share_above(x: &[f64], hz: f64) -> f64 {
    const N: usize = 4096;
    let w = hann(N);
    let (mut above, mut all) = (0.0, 0.0);
    let mut i = 0;
    while i + N <= x.len() {
        let p = power(&x[i..i + N], &w);
        for (k, v) in p.iter().enumerate() {
            all += v;
            if k as f64 * RATE / N as f64 > hz {
                above += v;
            }
        }
        i += N / 2;
    }
    above / all.max(1e-300)
}

/// The high band's envelope (the third difference's magnitude, averaged over 8 samples) over
/// `x`, and its power spectrum (a Hann window over all of it, zero-padded to a power of two):
/// the bins' power and their width, Hz.
fn envelope_spectrum(x: &[f64]) -> (Vec<f64>, f64) {
    const BIN: usize = 8;
    let d: Vec<f64> = (3..x.len())
        .map(|i| (x[i] - 3.0 * x[i - 1] + 3.0 * x[i - 2] - x[i - 3]).abs())
        .collect();
    let mut e: Vec<f64> = d
        .chunks(BIN)
        .map(|c| c.iter().sum::<f64>() / BIN as f64)
        .collect();
    let mean = e.iter().sum::<f64>() / e.len() as f64;
    e.iter_mut().for_each(|v| *v -= mean);
    let n = e.len().next_power_of_two();
    let w = hann(e.len());
    let mut frame = vec![0.0; n];
    let mut win = vec![0.0; n];
    frame[..e.len()].copy_from_slice(&e);
    win[..e.len()].copy_from_slice(&w);
    (power(&frame, &win), RATE / BIN as f64 / n as f64)
}

/// The envelope's power at `hz` and its first three harmonics (the strongest bin within two of
/// each), summed.
fn at_rate(spectrum: &(Vec<f64>, f64), hz: f64) -> f64 {
    let (p, width) = spectrum;
    (1..=4)
        .map(|h| {
            let k = (h as f64 * hz / width).round() as usize;
            (k.saturating_sub(2)..=(k + 2).min(p.len() - 1))
                .map(|i| p[i])
                .fold(0.0, f64::max)
        })
        .sum()
}

#[test]
#[ignore = "a measurement, run by hand"]
fn cutoff_sweep_stepped_against_continuous() {
    let mut c = controls_of(&preset("Brass Choir"));
    c.poly = false;
    // (At HI, as the report had it: `controls_of` starts from a new instance's LO.)
    (c.potato, c.ultra) = (common::switch("CA72_POTATO"), false);
    c.double = 0.0;
    c.spread = 0.0;
    c.panel.emphasis = env("CA72_EMPHASIS", 9.0) / 10.0;
    c.panel.contour_amount = 0.0;
    c.panel.osc[1].on = false;
    c.panel.osc[2].on = false;
    let out = std::env::var_os("CA72_CRACKLE_OUT").map(PathBuf::from);
    let reference = render(
        &c,
        &Steps {
            apply: 1,
            events: 1,
            glide: 0,
            pole: 0.0,
        },
    );
    let mono = |(l, r): &(Vec<f32>, Vec<f32>)| -> Vec<f64> {
        l.iter()
            .zip(r)
            .map(|(a, b)| 0.5 * f64::from(a + b))
            .collect()
    };
    let refm = mono(&reference);
    // (The sweep's two seconds only.)
    let sweep = (2.0 * RATE) as usize;
    let ref_rms = rms(&refm[..sweep]);
    eprintln!(
        "reference (every sample): RMS {:.1} dBFS",
        20.0 * ref_rms.log10()
    );
    let ref_env = envelope_spectrum(&refm[..sweep]);
    let mut cases: Vec<(String, Steps)> = Vec::new();
    for apply in [32usize, 64, 128, 256, 512, 1024] {
        let s = Steps {
            apply,
            events: 1,
            glide: 0,
            pole: 0.0,
        };
        cases.push((format!("block-{apply}"), s));
    }
    // A mouse's 60 events a second, each reaching the voices at the next 256-frame block.
    let mouse = Steps {
        apply: 256,
        events: 800,
        glide: 0,
        pole: 0.0,
    };
    cases.push(("mouse-60hz-at-256".into(), mouse));
    for apply in [256usize, 512, 1024] {
        let s = Steps {
            apply,
            events: 1,
            glide: 480,
            pole: 0.0,
        };
        cases.push((format!("block-{apply}-glide-10ms"), s));
    }
    let mouse = Steps {
        apply: 256,
        events: 800,
        glide: 480,
        pole: 0.0,
    };
    cases.push(("mouse-60hz-at-256-glide-10ms".into(), mouse));
    for (name, apply, events, glide) in [
        ("block-1024", 1024, 1, 1024),
        ("mouse-60hz-at-256", 256, 800, 960),
        ("mouse-60hz-at-256", 256, 800, 1440),
    ] {
        let s = Steps {
            apply,
            events,
            glide,
            pole: 0.0,
        };
        cases.push((format!("{name}-glide-{glide}"), s));
    }
    for (name, apply, events) in [("block-1024", 1024, 1), ("mouse-60hz-at-256", 256, 800)] {
        let s = Steps {
            apply,
            events,
            glide: 0,
            pole: 480.0,
        };
        cases.push((format!("{name}-pole-10ms"), s));
    }
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    // (Level-safe: every render at one gain, the reference's peak at -20 dBFS.)
    let peak = reference
        .0
        .iter()
        .chain(&reference.1)
        .fold(0.0f32, |m, v| m.max(v.abs()));
    let gain = 0.1 / peak.max(1e-6);
    let write = |name: &str, (l, r): &(Vec<f32>, Vec<f32>)| {
        if let Some(d) = &out {
            write_wav(&d.join(format!("cutoff-{name}.wav")), l, r, gain);
        }
    };
    write("every-sample", &reference);
    for (name, s) in &cases {
        let x = render(&c, s);
        let xm = mono(&x);
        let d: Vec<f64> = xm[..sweep]
            .iter()
            .zip(&refm[..sweep])
            .map(|(a, b)| a - b)
            .collect();
        let level = 20.0 * (rms(&d) / ref_rms).log10();
        // (The rate the voices are told a new value at: a block's, or the mouse's events.)
        let hz = RATE / s.apply.max(s.events) as f64;
        let env = envelope_spectrum(&xm[..sweep]);
        let zipper = 10.0 * (at_rate(&env, hz) / at_rate(&ref_env, hz)).log10();
        eprintln!(
            "{name}: difference from every sample {level:.1} dB ({:.0} % of it above 2 kHz); the high band's envelope at {hz:.1} Hz and its harmonics {zipper:+.1} dB against every sample's",
            100.0 * share_above(&d, 2000.0)
        );
        write(name, &x);
    }
}

fn write_wav(path: &std::path::Path, l: &[f32], r: &[f32], gain: f32) {
    let frames = l.len().min(r.len());
    let data = (frames * 8) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(RATE as u32).to_le_bytes());
    b.extend_from_slice(&(RATE as u32 * 8).to_le_bytes());
    b.extend_from_slice(&8u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for i in 0..frames {
        b.extend_from_slice(&(l[i] * gain).to_le_bytes());
        b.extend_from_slice(&(r[i] * gain).to_le_bytes());
    }
    std::fs::write(path, b).unwrap();
}
