//! Crackles reported in POLY at HI that WIDTH at 0 % under SCATTER cured (2026-10-19): Brass
//! Choir and Horn Bloom (DOUBLE on), and a plain POLY sawtooth, played chords, a legato line over
//! them and a cluster of more notes than VOICES (voices reused and stolen), each as set, at
//! WIDTH 0, with DOUBLE off (SCATTER) and with DOUBLE off at WIDTH 0.
//!
//! - `offline`: every variant at 44.1 and 48 kHz in blocks of 64 to 1024 frames, with and
//!   without workers, no deadline (as a host renders offline): each against the 256-frame
//!   render without workers, which must be the same to the bit, and its high-band bursts
//!   counted ([`bursts`]) beside its WIDTH 0 twin's (they mark low notes' sawtooth edges as
//!   well as clicks: compare like with like). `CA72_CRACKLE_OUT=<dir>` writes the
//!   256-frame renders as WAVs there.
//! - `paced`: the same phrase in real time, `CA72_BLOCK` frames (256) at 48 kHz, each block at
//!   its time with the plug-in's deadline and `CA72_WORKERS` workers (the plug-in's own count
//!   by default), ordinary threads (not promoted), against the offline render: the voices' runs
//!   not heard ([`Engine::silent_runs`]), the blocks late and the runs of samples that differ.
//!   Fewer workers than this machine's stand for a slower computer: on a Ryzen 7 7800X3D,
//!   `CA72_WORKERS=3` leaves Horn Bloom's twenty voices short of time (decisions.md
//!   R-POLY-CRACKLE). `CA72_CRACKLE_OUT=<dir>` writes the renders as WAVs there.
//!
//! By hand: `cargo test --release -p ca72-plugin --test crackle -- --ignored --nocapture`
//! (`CA72_ONLY`, a part of a variant's name; QUALITY at HI, or LO with `CA72_POTATO` set).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::engine::{AUTO, CHUNK, Controls, DEADLINE, Engine, Event};
use common::{controls_of, env, preset};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Blocks after which a voice had been put back clean, every render so far.
static MENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The phrase's seconds.
const SECONDS: f64 = 9.0;

/// A variant: its name and controls.
fn variants() -> Vec<(String, Controls)> {
    let brass = controls_of(&preset("Brass Choir"));
    let horn = controls_of(&preset("Horn Bloom"));
    // A plain POLY sawtooth: Brass Choir's first oscillator alone, WIDTH 100 %, no DOUBLE.
    let mut saw = brass;
    saw.panel.osc[1].on = false;
    saw.panel.osc[2].on = false;
    saw.panel.osc[0].volume = 0.8;
    saw.spread = 1.0;
    saw.double = 0.0;
    let mut out = Vec::new();
    // (QUALITY as the reports had it, HI, unless `CA72_POTATO` asks for LO: a new instance's
    // default, which `controls_of` starts from, is LO: decisions.md R-LO-DEFAULT.)
    let potato = common::switch("CA72_POTATO");
    for (name, mut c, voices) in [("brass", brass, 8), ("horn", horn, 10), ("saw", saw, 8)] {
        c.poly = true;
        c.voices = voices;
        (c.potato, c.ultra) = (potato, false);
        let mut w0 = c;
        w0.spread = 0.0;
        let mut scatter = c;
        scatter.double = 0.0;
        let mut scatter_w0 = scatter;
        scatter_w0.spread = 0.0;
        out.push((format!("{name}-as-set"), c));
        out.push((format!("{name}-width0"), w0));
        if c.double > 0.0 {
            out.push((format!("{name}-scatter"), scatter));
            out.push((format!("{name}-scatter-width0"), scatter_w0));
        }
    }
    let only = std::env::var("CA72_ONLY").ok();
    out.retain(|(n, _)| only.as_deref().is_none_or(|o| n.contains(o)));
    out
}

/// The phrase's notes, (sample, event), in order: overlapping four-note chords every 0.75 s
/// (each held a second), a legato line over them (each note pressed before the last is let
/// go), and from 6 s a cluster of twelve notes a tenth of a second apart, more than VOICES,
/// so that voices still sounding are taken.
fn phrase(rate: f64) -> Vec<(usize, Event)> {
    let at = |t: f64| (t * rate) as usize;
    let mut e = Vec::new();
    let mut note = |t0: f64, t1: f64, key: u8| {
        e.push((at(t0), Event::Note { key, on: true }));
        e.push((at(t1), Event::Note { key, on: false }));
    };
    let roots = [48u8, 53, 50, 55, 52, 57, 48, 53];
    for (k, r) in roots.iter().enumerate() {
        let t = 0.75 * k as f64;
        for d in [0u8, 4, 7, 11] {
            note(t, t + 1.0, r + d);
        }
    }
    let line = [72u8, 74, 76, 79, 77, 76, 74, 72, 71, 72, 74, 76];
    for (k, &key) in line.iter().enumerate() {
        let t = 0.3 + 0.45 * k as f64;
        note(t, t + 0.5, key);
    }
    for k in 0..12u8 {
        let t = 6.0 + 0.1 * f64::from(k);
        note(t, 8.0, 43 + 3 * k);
    }
    e.sort_by_key(|x| (x.0, matches!(x.1, Event::Note { on: true, .. })));
    e
}

/// The variant's render, `block` frames at a time at `rate` Hz with `workers` workers; each
/// block given the plug-in's deadline and played at its time if `paced`. Left and right, the
/// voices' runs not heard, and the blocks that took longer than their period (a host's
/// dropout) and the mean block's share of it.
fn render(
    c: &Controls,
    rate: f64,
    block: usize,
    workers: usize,
    paced: bool,
) -> (Vec<f32>, Vec<f32>, u64, usize, f64) {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(rate, 1);
    let period = Duration::from_secs_f64(block as f64 / rate);
    e.start_workers(workers, None);
    let events = phrase(rate);
    let n = (SECONDS * rate) as usize / block * block;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut next = events.iter().peekable();
    let start = Instant::now();
    let (mut late, mut busy, mut low) = (0, 0.0, usize::MAX);
    for (i, b) in (0..n).step_by(block).enumerate() {
        if paced {
            let due = start + period * i as u32;
            if let Some(w) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(w);
            }
            e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            // (The plug-in's helper thread's work, here between blocks: a pool asked for again,
            // its share moved, started.)
            if e.take_serving() {
                e.crew().serve();
            }
            low = low.min(e.workers());
        }
        let t = Instant::now();
        let mut k = b;
        while k < b + block {
            while let Some((_, ev)) = next.next_if(|(at, _)| *at <= k) {
                e.event(*ev);
            }
            let stop = next
                .peek()
                .map_or(b + block, |(at, _)| (*at).clamp(k + 1, b + block));
            e.render(&[], &mut l[k..stop], &mut r[k..stop]);
            k = stop;
        }
        e.end_block(block);
        let took = t.elapsed();
        if e.take_mending() {
            // (A voice gone non-finite and put back clean: decisions.md R18.)
            MENDED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            e.spares().mend();
        }
        late += usize::from(paced && took > period);
        busy += took.as_secs_f64() / period.as_secs_f64();
    }
    let silent = e.silent_runs();
    if paced || std::env::var_os("CA72_SHOW_WORKERS").is_some() {
        // (The workers' processor time, Linux: whether they played.)
        let mut ticks = 0u64;
        if let Ok(tasks) = std::fs::read_dir("/proc/self/task") {
            for t in tasks.flatten() {
                let comm = std::fs::read_to_string(t.path().join("comm")).unwrap_or_default();
                if comm.starts_with("ca72-voices") {
                    let stat = std::fs::read_to_string(t.path().join("stat")).unwrap_or_default();
                    let f: Vec<&str> = stat
                        .rsplit(')')
                        .next()
                        .unwrap_or("")
                        .split_whitespace()
                        .collect();
                    ticks += f.get(11).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                }
            }
        }
        eprintln!(
            "  (workers at the fewest: {low}; at the end: {}; their processor time {ticks} ticks)",
            e.workers()
        );
    }
    (l, r, silent, late, busy / (n / block) as f64)
}

/// High-band bursts in `x`: windows of 32 samples whose energy in the third difference (a
/// high-pass rising 18 dB an octave) is more than 30 times (14.8 dB) the median of the
/// windows within 40 of it, and above -90 dBFS; their start samples.
fn bursts(x: &[f32]) -> Vec<usize> {
    const W: usize = 32;
    const AROUND: usize = 40;
    let d: Vec<f64> = (3..x.len())
        .map(|i| {
            f64::from(x[i]) - 3.0 * f64::from(x[i - 1]) + 3.0 * f64::from(x[i - 2])
                - f64::from(x[i - 3])
        })
        .collect();
    let e: Vec<f64> = d
        .chunks(W)
        .map(|w| w.iter().map(|v| v * v).sum::<f64>() / W as f64)
        .collect();
    let mut out = Vec::new();
    for i in 0..e.len() {
        let lo = i.saturating_sub(AROUND);
        let hi = (i + AROUND + 1).min(e.len());
        let mut near: Vec<f64> = e[lo..hi].to_vec();
        near.sort_by(f64::total_cmp);
        let median = near[near.len() / 2];
        if e[i] > 30.0 * median && e[i] > 1e-9 {
            out.push(i * W + 3);
        }
    }
    out
}

/// The first sample where `a` and `b` differ, how many samples do, and how many runs of
/// [`CHUNK`] samples hold one.
fn differ(a: &[f32], b: &[f32]) -> (Option<usize>, usize, usize) {
    let first = a
        .iter()
        .zip(b)
        .position(|(x, y)| x.to_bits() != y.to_bits());
    let count = a
        .iter()
        .zip(b)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    let runs = a
        .chunks(CHUNK)
        .zip(b.chunks(CHUNK))
        .filter(|(x, y)| x.iter().zip(*y).any(|(p, q)| p.to_bits() != q.to_bits()))
        .count();
    (first, count, runs)
}

/// A stereo WAV of 32-bit floats at `rate` Hz, `gain` applied.
fn write_wav(path: &Path, rate: f64, l: &[f32], r: &[f32], gain: f32) {
    let frames = l.len().min(r.len());
    let data = (frames * 8) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(rate as u32).to_le_bytes());
    b.extend_from_slice(&(rate as u32 * 8).to_le_bytes());
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

fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0f32, |m, v| m.max(v.abs()))
}

#[test]
#[ignore = "a measurement, run by hand"]
fn offline() {
    let out = std::env::var_os("CA72_CRACKLE_OUT").map(PathBuf::from);
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    let mut same = true;
    for (name, c) in variants() {
        for rate in [44_100.0, 48_000.0] {
            let (l0, r0, ..) = render(&c, rate, 256, 0, false);
            let (bl, br) = (bursts(&l0), bursts(&r0));
            eprintln!(
                "{name} {rate}: peak {:.3}, high-band bursts left {} right {} (first {:?})",
                peak(&l0).max(peak(&r0)),
                bl.len(),
                br.len(),
                bl.iter().chain(&br).min().map(|s| *s as f64 / rate),
            );
            for (block, workers) in [(64, 0), (1024, 0), (256, 3), (128, 9), (512, 5)] {
                let (l, r, silent, ..) = render(&c, rate, block, workers, false);
                let (fl, nl, _) = differ(&l0, &l);
                let (fr, nr, _) = differ(&r0, &r);
                if nl + nr > 0 || silent > 0 {
                    same = false;
                    eprintln!(
                        "  {block} frames, {workers} workers: DIFFERS: left {nl} from {fl:?}, right {nr} from {fr:?}, {silent} runs not heard"
                    );
                }
            }
            if let Some(d) = &out
                && rate == 48_000.0
            {
                write_wav(&d.join(format!("{name}.wav")), rate, &l0, &r0, 1.0);
            }
        }
    }
    let mended = MENDED.load(std::sync::atomic::Ordering::Relaxed);
    eprintln!("voices put back clean (gone non-finite): {mended}");
    assert!(same, "a render differed from the 256-frame one");
    assert_eq!(mended, 0, "a voice went non-finite");
}

#[test]
#[ignore = "a measurement, run by hand"]
fn paced() {
    let block: usize = env("CA72_BLOCK", 256);
    let workers: usize = env("CA72_WORKERS", AUTO);
    // (A measurement's own budget of threads, as on a computer of more processors than this
    // run is given: `CA72_WORKERS` workers whatever the processors.)
    if workers != AUTO {
        ca72_plugin::engine::set_budget_for_measurement(workers);
    }
    let rate = 48_000.0;
    let out = std::env::var_os("CA72_CRACKLE_OUT").map(PathBuf::from);
    for (name, c) in variants() {
        let (l0, r0, ..) = render(&c, rate, block, 0, false);
        let (l, r, silent, late, busy) = render(&c, rate, block, workers, true);
        let started = {
            let mut e = Engine::new();
            e.set(&c);
            e.prepare(rate, 1);
            e.start_workers(workers, None)
        };
        let (_, nl, runs_l) = differ(&l0, &l);
        let (_, nr, runs_r) = differ(&r0, &r);
        let bursts_here = bursts(&l).len() + bursts(&r).len();
        let bursts_ref = bursts(&l0).len() + bursts(&r0).len();
        eprintln!(
            "paced {name}, {block} frames, {started} workers: {silent} voices' runs not heard, {late} blocks late (the mean {:.0} % of a period); samples differing from offline {} ({} runs of {CHUNK}); high-band bursts {bursts_here} (offline {bursts_ref})",
            100.0 * busy,
            nl + nr,
            runs_l.max(runs_r),
        );
        if let Some(d) = &out {
            std::fs::create_dir_all(d).unwrap();
            write_wav(
                &d.join(format!("{name}-paced-{block}.wav")),
                rate,
                &l,
                &r,
                1.0,
            );
        }
    }
}
