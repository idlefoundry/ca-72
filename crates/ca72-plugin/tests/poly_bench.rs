//! POLY in real time (decisions.md, "POLY in real time"): the engine as a host calls it, on
//! one thread promoted as an audio thread, `CA72_BLOCK`-frame blocks (256) at 48 kHz, each
//! at its time and never back to back (a late block is followed by the next period's
//! start), for `CA72_SECONDS` (60) after 2 s: `CA72_VOICES` voices (10) playing a changing
//! chord of as many notes every half second (each held seven eighths of it, so the next
//! chord takes voices still releasing), with the modulation moving (oscillator 3 on the
//! filter through the MODULATION wheel, GLIDE on, ANALOG at 50 %, SPREAD at 70 %, the
//! cutoff swept). Prints each block's time, the late blocks and the watchdog's demotions.
//!
//! By hand, on a quiet machine: `cargo test --release -p ca72-plugin --test poly_bench --
//! --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

use ca72_plugin::engine::{Controls, Engine, Event};
use std::time::{Duration, Instant};

fn env<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore]
fn poly_chords_in_real_time() {
    let voices: usize = env("CA72_VOICES", 10);
    let seconds: f64 = env("CA72_SECONDS", 60.0);
    let block: usize = env("CA72_BLOCK", 256);
    let rate = 48_000.0;
    let mut c = Controls {
        poly: true,
        voices,
        analog: 0.5,
        spread: 0.7,
        ..Controls::default()
    };
    c.panel.osc[1].on = true;
    c.panel.osc[1].freq = 0.52;
    c.panel.osc[2].on = true;
    c.panel.osc[2].range = ca72::tuning::Range::Lo;
    c.panel.filter_mod = true;
    c.panel.mod_wheel = 0.4;
    c.panel.glide_on = true;
    c.panel.glide = 0.15;
    c.panel.emphasis = 0.5;
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(rate, 1);
    let period = Duration::from_secs_f64(block as f64 / rate);
    let rt = ca72_rt::promote(period, period / 2);
    ca72_rt::watch();
    let roots = [45u8, 41, 43, 48, 40, 45, 50, 43];
    let shape = [0u8, 7, 12, 16, 19, 24, 28, 31, 34, 36];
    let beat = (0.5 * rate) as usize;
    let warm = (2.0 * rate) as usize / block;
    let blocks = (seconds * rate / block as f64) as usize;
    let mut took = Vec::with_capacity(blocks);
    let mut peak = 0.0f32;
    let start = Instant::now();
    let mut next = 0u32;
    let mut held: Vec<u8> = Vec::with_capacity(16);
    for i in 0..warm + blocks {
        let due = start + period * next;
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
        let t = Instant::now();
        let at = i * block;
        for s in 0..block {
            let n = at + s;
            if n % beat == beat * 7 / 8 {
                for k in held.drain(..) {
                    e.event(Event::Note { key: k, on: false });
                }
            }
            if n.is_multiple_of(beat) {
                let b = n / beat;
                let r = roots[b % roots.len()] + (b / roots.len() % 3) as u8;
                for s in shape.iter().take(voices) {
                    e.event(Event::Note { key: r + s, on: true });
                    held.push(r + s);
                }
            }
            if s == 0 {
                c.panel.cutoff = 0.5 + 0.2 * (n as f64 * 2.0 * std::f64::consts::PI / rate).sin();
                e.set(&c);
            }
            let (l, r) = e.tick_stereo(0.0);
            peak = peak.max(l.abs()).max(r.abs());
        }
        e.end_block(block);
        if i >= warm {
            took.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let elapsed = start.elapsed().as_secs_f64() / period.as_secs_f64();
        next = (next + 1).max(elapsed.ceil() as u32);
    }
    let p_ms = period.as_secs_f64() * 1e3;
    let late = took.iter().filter(|&&t| t > p_ms).count();
    let mut sorted = took.clone();
    sorted.sort_by(f64::total_cmp);
    let q = |x: f64| sorted[((sorted.len() - 1) as f64 * x) as usize];
    let mean = took.iter().sum::<f64>() / took.len() as f64;
    eprintln!(
        "CA-72: {voices} POLY voices at {block}-frame blocks ({p_ms:.3} ms), {seconds} s paced on one thread: mean {mean:.3} ms, p50 {:.3}, p99 {:.3}, p99.9 {:.3}, worst {:.3}; {late} of {} late; {:.1} % of the thread; peak {peak:.3}; real-time: {rt:?}, demoted by the watchdog {} times",
        q(0.5),
        q(0.99),
        q(0.999),
        sorted[sorted.len() - 1],
        took.len(),
        mean / p_ms * 100.0,
        ca72_rt::demotions(),
    );
    assert!(peak > 0.01, "the chords were not heard");
}
