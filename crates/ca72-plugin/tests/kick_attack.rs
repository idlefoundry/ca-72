//! Driven Kick's attack, single notes (the owner's report of 2026-10-17: "driven kick has a
//! high spikey tone at the attack on Lo"): the engine as a host plays it, one key at a time,
//! in each QUALITY mode. Prints, per note, the loudest burst above 5 kHz in its first 30 ms
//! (the whole render high-passed, so no window's edge adds any), when it peaks and its
//! frequency; writes each render as a WAV file to `CA72_KICK_OUT` if set.
//!
//! By hand: `cargo test -p ca72-plugin --test kick_attack -- --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::fft::fft;
use ca72_plugin::engine::{Engine, Event};
use common::{controls_of, preset};

const KEYS: [u8; 4] = [45, 57, 69, 57];

/// Driven Kick in `mode` at `rate`, in blocks of `block` samples, `KEYS` `gap` seconds apart
/// (each held a third of it; at 3 s the engine falls asleep between them: decisions.md R68).
/// The left output, and each key's sample.
fn render(mode: &str, rate: f64, block: usize, gap: f64) -> (Vec<f64>, Vec<usize>) {
    let mut c = controls_of(&preset("Driven Kick"));
    c.potato = mode == "LO";
    c.ultra = mode == "ULTRA";
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(rate, 1);
    let gap = (gap * rate) as usize;
    let n = (gap * (KEYS.len() + 1)).next_power_of_two();
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut at = Vec::new();
    let mut events: Vec<(usize, Event)> = Vec::new();
    for (k, &key) in KEYS.iter().enumerate() {
        let t = gap / 2 + k * gap;
        at.push(t);
        events.push((t, Event::Note { key, on: true }));
        events.push((t + gap / 3, Event::Note { key, on: false }));
    }
    let mut next = events.iter().peekable();
    for b in (0..n).step_by(block) {
        let end = (b + block).min(n);
        let mut k = b;
        while k < end {
            while let Some((_, ev)) = next.next_if(|(t, _)| *t <= k) {
                e.event(*ev);
            }
            let stop = next.peek().map_or(end, |(t, _)| (*t).clamp(k + 1, end));
            e.render(&[], &mut l[k..stop], &mut r[k..stop]);
            k = stop;
        }
        e.end_block(end - b);
    }
    (l.iter().map(|&v| f64::from(v)).collect(), at)
}

/// `x` (a power of two long) with everything below `hz` taken out.
fn high_passed(x: &[f64], rate: f64, hz: f64) -> Vec<f64> {
    let n = x.len();
    let (mut re, mut im) = (x.to_vec(), vec![0.0; n]);
    fft(&mut re, &mut im);
    for k in 0..n {
        if (k.min(n - k)) as f64 * rate / (n as f64) < hz {
            re[k] = 0.0;
            im[k] = 0.0;
        }
    }
    // (The inverse by the forward transform of the conjugate.)
    im.iter_mut().for_each(|v| *v = -*v);
    fft(&mut re, &mut im);
    re.iter().map(|v| v / n as f64).collect()
}

/// The burst above 5 kHz in the 30 ms from `t`: its peak (dBFS), when (ms) and its frequency
/// (Hz, the strongest bin of 256 samples about its peak).
fn burst(h: &[f64], t: usize, rate: f64) -> (f64, f64, f64) {
    let seg = &h[t..t + (0.03 * rate) as usize];
    let (i, peak) =
        seg.iter().enumerate().fold(
            (0, 0.0f64),
            |m, (i, v)| {
                if v.abs() > m.1 { (i, v.abs()) } else { m }
            },
        );
    let a = (t + i).saturating_sub(128);
    let mut re: Vec<f64> = (0..256)
        .map(|k| h[a + k] * (0.5 - 0.5 * (std::f64::consts::TAU * k as f64 / 256.0).cos()))
        .collect();
    let mut im = vec![0.0; 256];
    fft(&mut re, &mut im);
    let k = (1..128)
        .max_by(|&p, &q| re[p].hypot(im[p]).total_cmp(&re[q].hypot(im[q])))
        .unwrap();
    (
        20.0 * peak.max(1e-12).log10(),
        i as f64 / rate * 1000.0,
        k as f64 * rate / 256.0,
    )
}

fn wav(path: &str, rate: f64, x: &[f64]) {
    let d: Vec<u8> = x.iter().flat_map(|v| (*v as f32).to_le_bytes()).collect();
    let mut h = Vec::new();
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&(36 + d.len() as u32).to_le_bytes());
    h.extend_from_slice(b"WAVEfmt ");
    h.extend_from_slice(&16u32.to_le_bytes());
    h.extend_from_slice(&3u16.to_le_bytes());
    h.extend_from_slice(&1u16.to_le_bytes());
    h.extend_from_slice(&(rate as u32).to_le_bytes());
    h.extend_from_slice(&(rate as u32 * 4).to_le_bytes());
    h.extend_from_slice(&4u16.to_le_bytes());
    h.extend_from_slice(&32u16.to_le_bytes());
    h.extend_from_slice(b"data");
    h.extend_from_slice(&(d.len() as u32).to_le_bytes());
    h.extend_from_slice(&d);
    std::fs::write(path, h).unwrap();
}

fn report(mode: &str, rate: f64, block: usize, gap: f64) {
    let (x, at) = render(mode, rate, block, gap);
    if let Ok(dir) = std::env::var("CA72_KICK_OUT") {
        wav(
            &format!("{dir}/kick-{mode}-{rate}-{block}-{gap}.wav"),
            rate,
            &x,
        );
    }
    let h = high_passed(&x, rate, 5_000.0);
    let row: Vec<String> = at
        .iter()
        .zip(KEYS)
        .map(|(&t, key)| {
            let (db, ms, hz) = burst(&h, t, rate);
            format!("{key}: {db:6.1} dBFS at {ms:4.1} ms, {hz:5.0} Hz")
        })
        .collect();
    eprintln!("{rate} {block:>4} {gap} s {mode:<5} {}", row.join(" | "));
}

#[test]
#[ignore = "a measurement, run by hand"]
fn driven_kicks_attack() {
    for rate in [44_100.0, 48_000.0] {
        for mode in ["LO", "HI", "ULTRA"] {
            report(mode, rate, 256, 0.6);
        }
    }
}

#[test]
#[ignore = "a measurement, run by hand"]
fn driven_kicks_attack_after_sleep_and_in_other_blocks() {
    for rate in [44_100.0, 48_000.0, 96_000.0] {
        for block in [64, 128, 512, 1024] {
            report("LO", rate, block, 3.0);
        }
    }
}
