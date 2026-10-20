//! Distortion between notes in POLY patches on Windows that MONO cured (issue #60): the factory
//! presets that play the most voices, each as set and with POLY off, played in real time as a
//! host plays them (`paced`), and whole with no deadline (`offline`).
//!
//! - `paced`: [`phrase`] at 48 kHz, `CA72_BLOCKS` frames (128,256,512), each block at its time
//!   with the plug-in's deadline, at QUALITY `CA72_QUALITIES` (lo,hi), with `CA72_WORKERS`
//!   workers (the plug-in's own count by default). The measuring thread is made an audio
//!   thread as a host's is (Windows: MMCSS's Pro Audio task; Linux: `SCHED_FIFO`; macOS: a
//!   time-constraint thread in a work interval), the workers too, unless `CA72_PLAIN` is set,
//!   and it flushes denormals to zero as nih-plug's process call has it. Per case: the blocks
//!   late (a host's dropout: what is heard as crackle, the plug-in's own meter none the
//!   wiser), the voices' runs not heard, the block's time as a share of its period (mean, 99th
//!   percentile, most), and the runs of samples that differ from the offline render.
//! - `offline`: the same phrase whole, against other block sizes and the workers (shown);
//!   every sample finite, the voices put back clean none, and the loudest sample-to-sample
//!   step at each note's start and end against the loudest step anywhere else (a click as a
//!   voice is taken or let go).
//!
//! `CA72_ONLY`, a part of a sound's name. By hand:
//! `cargo test --release -p ca72-plugin --test note_to_note -- --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::engine::{AUTO, CHUNK, Controls, DEADLINE, Engine, Event};
use common::{controls_of, env, played_as, preset};
use std::time::{Duration, Instant};

const RATE: f64 = 48_000.0;
const SECONDS: f64 = 8.0;

/// The flush-to-zero bit nih-plug sets on the host's thread for each block (`ScopedFtz`).
#[cfg(target_arch = "x86_64")]
const FTZ: u32 = 1 << 15;
#[cfg(target_arch = "aarch64")]
const FTZ: u32 = 1 << 24;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const FTZ: u32 = 0;

fn list(name: &str, default: &[&str]) -> Vec<String> {
    std::env::var(name).map_or_else(
        |_| default.iter().map(|s| (*s).to_string()).collect(),
        |v| v.split(',').map(|x| x.trim().to_string()).collect(),
    )
}

/// The sounds: each POLY preset as set, and Brass Choir with POLY off (MONO, the cure reported).
fn sounds() -> Vec<(String, Controls)> {
    let mut out = Vec::new();
    for name in ["Brass Choir", "Horn Bloom", "Warp Field", "Cycle of Fifths"] {
        let s = preset(name);
        out.push((name.to_string(), played_as(&s, controls_of(&s))));
    }
    let s = preset("Brass Choir");
    let mut mono = played_as(&s, controls_of(&s));
    mono.poly = false;
    out.push(("Brass Choir, POLY off".to_string(), mono));
    let only = std::env::var("CA72_ONLY").ok();
    out.retain(|(n, _)| only.as_deref().is_none_or(|o| n.contains(o)));
    out
}

/// The notes, (sample, event): one key after another, detached and then legato (each pressed
/// before the last is let go), then chords of three and four notes changing every half second,
/// then a run of quick notes, more than VOICES, so voices still sounding are taken.
fn phrase() -> Vec<(usize, Event)> {
    let at = |t: f64| (t * RATE) as usize;
    let mut e = Vec::new();
    let mut note = |t0: f64, t1: f64, key: u8| {
        e.push((at(t0), Event::Note { key, on: true }));
        e.push((at(t1), Event::Note { key, on: false }));
    };
    let line = [60u8, 62, 64, 65, 67, 69, 71, 72];
    for (k, &key) in line.iter().enumerate() {
        let t = 0.2 + 0.25 * k as f64;
        note(t, t + 0.2, key);
    }
    for (k, &key) in line.iter().rev().enumerate() {
        let t = 2.2 + 0.2 * k as f64;
        note(t, t + 0.25, key);
    }
    let chords: [&[u8]; 6] = [
        &[48, 52, 55],
        &[50, 53, 57, 60],
        &[52, 55, 59],
        &[53, 57, 60, 64],
        &[55, 59, 62, 65],
        &[48, 52, 55, 60],
    ];
    for (k, c) in chords.iter().enumerate() {
        let t = 4.0 + 0.5 * k as f64;
        for &key in *c {
            note(t, t + 0.45, key);
        }
    }
    for k in 0..16u8 {
        let t = 7.0 + 0.04 * f64::from(k);
        note(t, t + 0.15, 60 + (k * 5) % 24);
    }
    e.sort_by_key(|x| (x.0, matches!(x.1, Event::Note { on: true, .. })));
    e
}

struct Run {
    l: Vec<f32>,
    r: Vec<f32>,
    late: usize,
    silent: u64,
    took: Vec<f64>,
    mended: usize,
}

fn render(c: &Controls, block: usize, workers: usize, paced: bool, rt: bool) -> Run {
    let period = Duration::from_secs_f64(block as f64 / RATE);
    let was = ca72_rt::flush_mode();
    ca72_rt::set_flush_mode(was | FTZ);
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, 1);
    e.start_workers(workers, (paced && rt).then_some(period));
    let host = if paced && rt {
        if let Err(e) = ca72_rt::promote(period, period.mul_f64(0.5)) {
            eprintln!("(the measuring thread not promoted: {e})");
        }
        ca72_rt::WorkInterval::new("CA-72 measurement host")
    } else {
        None
    };
    let token = host.as_ref().and_then(ca72_rt::WorkInterval::join);
    let events = phrase();
    let n = (SECONDS * RATE) as usize / block * block;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut next = events.iter().peekable();
    let (mut late, mut mended, mut took) = (0, 0, Vec::new());
    let start = Instant::now();
    for (i, b) in (0..n).step_by(block).enumerate() {
        if paced {
            let due = start + period * i as u32;
            if let Some(w) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(w);
            }
            if token.is_some()
                && let Some(h) = &host
            {
                h.start(period);
            }
            e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            if e.take_serving() {
                e.crew().serve();
                e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            }
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
        let d = t.elapsed();
        if paced {
            if token.is_some()
                && let Some(h) = &host
            {
                h.finish();
            }
            late += usize::from(d > period);
            took.push(d.as_secs_f64() / period.as_secs_f64());
        } else if e.take_serving() {
            e.crew().serve();
        }
        if e.take_mending() {
            mended += 1;
            e.spares().mend();
        }
    }
    let silent = e.silent_runs();
    drop(e);
    if let (Some(h), Some(t)) = (&host, token) {
        h.leave(t);
    }
    if paced && rt {
        ca72_rt::demote();
    }
    ca72_rt::set_flush_mode(was);
    Run {
        l,
        r,
        late,
        silent,
        took,
        mended,
    }
}

fn at_quality(c: &Controls, q: &str) -> Controls {
    let mut c = *c;
    (c.potato, c.ultra) = match q {
        "lo" => (true, false),
        "hi" => (false, false),
        "ultra" => (false, true),
        _ => panic!("CA72_QUALITIES: lo, hi or ultra, not {q}"),
    };
    c
}

#[test]
#[ignore = "a measurement, run by hand"]
fn paced() {
    let blocks: Vec<usize> = list("CA72_BLOCKS", &["128", "256", "512"])
        .iter()
        .map(|b| b.parse().unwrap())
        .collect();
    let workers: usize = env("CA72_WORKERS", AUTO);
    if workers != AUTO {
        ca72_plugin::engine::set_budget_for_measurement(workers);
    }
    let rt = std::env::var_os("CA72_PLAIN").is_none();
    for q in list("CA72_QUALITIES", &["lo", "hi"]) {
        for (name, c) in sounds() {
            let c = at_quality(&c, &q);
            for &block in &blocks {
                let offline = render(&c, block, 0, false, false);
                let mut run = render(&c, block, workers, true, rt);
                let differ = run
                    .l
                    .chunks(CHUNK)
                    .zip(run.r.chunks(CHUNK))
                    .zip(offline.l.chunks(CHUNK).zip(offline.r.chunks(CHUNK)))
                    .filter(|((a, b), (a0, b0))| {
                        a.iter().zip(*a0).any(|(x, y)| x.to_bits() != y.to_bits())
                            || b.iter().zip(*b0).any(|(x, y)| x.to_bits() != y.to_bits())
                    })
                    .count();
                run.took.sort_by(f64::total_cmp);
                let mean = run.took.iter().sum::<f64>() / run.took.len() as f64;
                let p99 = run.took[(run.took.len() * 99 / 100).min(run.took.len() - 1)];
                let most = *run.took.last().unwrap();
                eprintln!(
                    "{q} {name}, {block} frames: late {} of {}, runs not heard {}, block {:.0} % / p99 {:.0} % / most {:.0} % of the period; runs differing from offline {differ}; mended {}",
                    run.late,
                    run.took.len(),
                    run.silent,
                    100.0 * mean,
                    100.0 * p99,
                    100.0 * most,
                    run.mended
                );
            }
        }
    }
}

#[test]
#[ignore = "a measurement, run by hand"]
fn offline() {
    let events = phrase();
    let edges: Vec<usize> = events.iter().map(|e| e.0).collect();
    let mut faults = 0;
    for q in list("CA72_QUALITIES", &["lo", "hi"]) {
        for (name, c) in sounds() {
            let c = at_quality(&c, &q);
            let base = render(&c, 256, 0, false, false);
            let mut differing = Vec::new();
            for block in [64, 128, 256, 512, 1024] {
                for workers in [0, AUTO] {
                    if block == 256 && workers == 0 {
                        continue;
                    }
                    let x = render(&c, block, workers, false, false);
                    let n = x.l.len().min(base.l.len());
                    let first = x.l[..n]
                        .iter()
                        .zip(&base.l[..n])
                        .chain(x.r[..n].iter().zip(&base.r[..n]))
                        .position(|(a, b)| a.to_bits() != b.to_bits());
                    if let Some(i) = first {
                        let most = x.l[..n]
                            .iter()
                            .zip(&base.l[..n])
                            .chain(x.r[..n].iter().zip(&base.r[..n]))
                            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
                        differing.push(format!(
                            "{block} frames, {} workers from {:.3} s, by {:.1} dBFS at most",
                            if workers == AUTO { "auto" } else { "no" },
                            (i % n) as f64 / RATE,
                            20.0 * f64::from(most).log10()
                        ));
                    }
                }
            }
            let same = differing.is_empty();
            let finite = base.l.iter().chain(&base.r).all(|v| v.is_finite());
            // The loudest step within 2 ms of a note's start or end, and elsewhere.
            let near = |i: usize| edges.iter().any(|&e| i + 96 >= e && i <= e + 96);
            let (mut at_notes, mut elsewhere) = (0.0f32, 0.0f32);
            for x in [&base.l, &base.r] {
                for i in 1..x.len() {
                    let step = (x[i] - x[i - 1]).abs();
                    if near(i) {
                        at_notes = at_notes.max(step);
                    } else {
                        elsewhere = elsewhere.max(step);
                    }
                }
            }
            let peak = base
                .l
                .iter()
                .chain(&base.r)
                .fold(0.0f32, |m, v| m.max(v.abs()));
            for d in &differing {
                eprintln!("  {q} {name}: differs from 256 frames without workers at {d}");
            }
            eprintln!(
                "{q} {name}: blocks and workers the same to the bit {same}, finite {finite}, mended {}, peak {:.1} dBFS; loudest step at a note's start or end {:.4}, elsewhere {:.4}",
                base.mended,
                20.0 * f64::from(peak).log10(),
                at_notes,
                elsewhere
            );
            // (A render in other blocks may differ: the one instrument falls asleep after a
            // silent stretch counted in blocks (decisions.md R68), its free-running oscillators
            // then at another phase, and POLY's voices are let go at a block's end, so another
            // may take the next note. Shown, not a fault.)
            let _ = same;
            if !finite || base.mended > 0 || at_notes > 1.5 * elsewhere {
                faults += 1;
            }
        }
    }
    assert_eq!(faults, 0, "faults found");
}

/// What each block costs, played as fast as it goes on this thread (no deadline, no workers, no
/// pacing: the scheduler out of it): the blocks that begin a note against those that don't, at
/// `CA72_BLOCK` frames (256), `CA72_ROUNDS` times (5) for the least of each block's times. A
/// block that begins notes costing many times the others is a spike a host hears as a dropout
/// at that note, however light the average.
#[test]
#[ignore = "a measurement, run by hand"]
fn costs() {
    let block: usize = env("CA72_BLOCK", 256);
    let rounds: usize = env("CA72_ROUNDS", 5);
    let period = block as f64 / RATE;
    let events = phrase();
    for q in list("CA72_QUALITIES", &["lo", "hi"]) {
        for (name, c) in sounds() {
            let c = at_quality(&c, &q);
            let n = (SECONDS * RATE) as usize / block * block;
            let blocks = n / block;
            let mut best = vec![f64::INFINITY; blocks];
            let mut ons = vec![0usize; blocks];
            for (at, ev) in &events {
                if matches!(ev, Event::Note { on: true, .. }) && *at < n {
                    ons[at / block] += 1;
                }
            }
            for _ in 0..rounds {
                let was = ca72_rt::flush_mode();
                ca72_rt::set_flush_mode(was | FTZ);
                let mut e = Engine::new();
                e.set(&c);
                e.prepare(RATE, 1);
                let (mut l, mut r) = (vec![0.0f32; block], vec![0.0f32; block]);
                let mut next = events.iter().peekable();
                for (i, b) in (0..n).step_by(block).enumerate() {
                    let t = Instant::now();
                    let mut k = b;
                    while k < b + block {
                        while let Some((_, ev)) = next.next_if(|(at, _)| *at <= k) {
                            e.event(*ev);
                        }
                        let stop = next
                            .peek()
                            .map_or(b + block, |(at, _)| (*at).clamp(k + 1, b + block));
                        e.render(&[], &mut l[k - b..stop - b], &mut r[k - b..stop - b]);
                        k = stop;
                    }
                    e.end_block(block);
                    best[i] = best[i].min(t.elapsed().as_secs_f64() / period);
                }
                ca72_rt::set_flush_mode(was);
            }
            let stats = |pick: &dyn Fn(usize) -> bool| {
                let mut v: Vec<f64> = (0..blocks).filter(|&i| pick(i)).map(|i| best[i]).collect();
                v.sort_by(f64::total_cmp);
                if v.is_empty() {
                    return (0, 0.0, 0.0, 0.0);
                }
                let mean = v.iter().sum::<f64>() / v.len() as f64;
                (v.len(), mean, v[v.len() / 2], *v.last().unwrap())
            };
            let (n_on, mean_on, mid_on, most_on) = stats(&|i| ons[i] > 0);
            let (n_off, mean_off, mid_off, most_off) = stats(&|i| ons[i] == 0);
            let worst = (0..blocks)
                .max_by(|&a, &b| best[a].total_cmp(&best[b]))
                .unwrap();
            eprintln!(
                "{q} {name}, {block} frames: blocks beginning notes ({n_on}) {:.0} % mean, {:.0} % median, {:.0} % most; others ({n_off}) {:.0} % mean, {:.0} % median, {:.0} % most; the costliest block at {:.2} s with {} notes begun",
                100.0 * mean_on,
                100.0 * mid_on,
                100.0 * most_on,
                100.0 * mean_off,
                100.0 * mid_off,
                100.0 * most_off,
                (worst * block) as f64 / RATE,
                ons[worst]
            );
        }
    }
}
