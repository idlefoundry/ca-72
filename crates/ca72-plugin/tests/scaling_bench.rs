//! How the plug-in scales with its workers (decisions.md R-THREADS): `CA72_INSTANCES` instances (1)
//! in one process, each on a host audio thread of its own, all paced to the same
//! `CA72_BLOCK`-frame period (256) at 48 kHz and started together, as a host's tracks are;
//! `CA72_WORKERS` workers (4) made audio threads for the whole process, out of one budget of
//! that size (`engine::set_budget_for_measurement`). `CA72_SHARE=fair` (the default) asks for
//! an even share of them for each instance; `first` asks for all of them for each, as the
//! plug-in does today (the first instance started takes them, R18).
//!
//! Each instance plays `poly_bench`'s load: POLY on, `CA72_VOICES` voices (10 at HI, 4 at
//! ULTRA) playing a changing chord of as many notes every half second, the modulation moving;
//! the instances' chords on different roots, changing at the same blocks. `CA72_QUALITY`:
//! `hi` (the default) or `ultra`. After 2 s, `CA72_SECONDS` (10) measured. With
//! `CA72_HOST_WG=1` (the default) each host thread works under a work interval of its own,
//! as a CoreAudio IO thread does (macOS).
//!
//! Prints one line, `scaling: key=value ...`: the shares of the period every instance's
//! blocks took (pooled), the late blocks, the voices' runs not heard (a worker past the
//! deadline) and the watchdog's demotions.
//!
//! By hand, on a quiet machine: `cargo test --release -p ca72-plugin --test scaling_bench --
//! --ignored --nocapture` (`scripts/scaling-sweep.sh` runs the grid).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::engine::{Controls, DEADLINE, Engine, Event, set_budget_for_measurement};
use common::env;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

struct Ran {
    took: Vec<f64>,
    silent: u64,
    peak: f32,
}

#[test]
#[ignore = "a measurement, run by hand"]
fn workers_and_instances_in_real_time() {
    let instances: usize = env("CA72_INSTANCES", 1);
    let workers: usize = env("CA72_WORKERS", 4);
    let block: usize = env("CA72_BLOCK", 256);
    let seconds: f64 = env("CA72_SECONDS", 10.0);
    let ultra = std::env::var("CA72_QUALITY").is_ok_and(|q| q == "ultra");
    let voices: usize = env("CA72_VOICES", if ultra { 4 } else { 10 });
    let fair = std::env::var("CA72_SHARE").map_or(true, |s| s != "first");
    let host_wg = env("CA72_HOST_WG", 1) == 1;
    let rate = 48_000.0;
    let period = Duration::from_secs_f64(block as f64 / rate);
    set_budget_for_measurement(workers.max(1));

    let mut c = Controls {
        poly: true,
        voices,
        entropy: 0.5,
        spread: 0.7,
        ultra,
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

    // Every instance made and its workers started here, in turn, off the audio threads (as a
    // host activates its plug-ins one by one).
    let mut engines = Vec::with_capacity(instances);
    let mut got = Vec::with_capacity(instances);
    for k in 0..instances {
        let mut e = Engine::new();
        e.set(&c);
        e.prepare(rate, 1 + k as u64);
        let ask = if workers == 0 {
            0
        } else if fair {
            workers / instances + usize::from(k < workers % instances)
        } else {
            workers
        };
        got.push(e.start_workers(ask, Some(period)));
        engines.push(e);
    }
    let load = load_average();

    let go = Arc::new(Barrier::new(instances));
    let start_at = Arc::new(std::sync::Mutex::new(None::<Instant>));
    let threads: Vec<_> = engines
        .into_iter()
        .enumerate()
        .map(|(k, mut e)| {
            let (go, start_at, c) = (go.clone(), start_at.clone(), c);
            std::thread::Builder::new()
                .name(format!("ca72-host-{k}"))
                .spawn(move || {
                    host(
                        k, &mut e, c, block, seconds, period, host_wg, &go, &start_at,
                    )
                })
                .unwrap()
        })
        .collect();
    let ran: Vec<Ran> = threads.into_iter().map(|t| t.join().unwrap()).collect();

    let p_ms = period.as_secs_f64() * 1e3;
    let mut all: Vec<f64> = ran.iter().flat_map(|r| r.took.iter().copied()).collect();
    let late = all.iter().filter(|&&t| t > p_ms).count();
    let blocks = all.len();
    all.sort_by(f64::total_cmp);
    let q = |x: f64| all[((all.len() - 1) as f64 * x) as usize] / p_ms * 100.0;
    let mean = all.iter().sum::<f64>() / all.len() as f64 / p_ms * 100.0;
    // Each instance's worst, and the worst of the instances' means.
    let worst_mean = ran
        .iter()
        .map(|r| r.took.iter().sum::<f64>() / r.took.len() as f64 / p_ms * 100.0)
        .fold(0.0, f64::max);
    let silent: u64 = ran.iter().map(|r| r.silent).sum();
    let got_s: Vec<String> = got.iter().map(ToString::to_string).collect();
    println!(
        "scaling: quality={} block={block} instances={instances} workers={workers} share={} got={} voices={voices} seconds={seconds} mean={mean:.1} worst_instance_mean={worst_mean:.1} p50={:.1} p99={:.1} p999={:.1} worst={:.1} late={late} blocks={blocks} silent_runs={silent} demotions={} load={load}",
        if ultra { "ultra" } else { "hi" },
        if fair { "fair" } else { "first" },
        got_s.join("+"),
        q(0.5),
        q(0.99),
        q(0.999),
        all[all.len() - 1] / p_ms * 100.0,
        ca72_rt::demotions(),
    );
    set_budget_for_measurement(0);
    assert!(
        ran.iter().all(|r| r.peak > 0.01),
        "an instance's chords were not heard"
    );
}

/// One instance on its host thread: promoted, its blocks at their times from the shared
/// start, `poly_bench`'s chords on roots of its own.
#[allow(clippy::too_many_arguments)]
fn host(
    k: usize,
    e: &mut Engine,
    mut c: Controls,
    block: usize,
    seconds: f64,
    period: Duration,
    host_wg: bool,
    go: &Barrier,
    start_at: &std::sync::Mutex<Option<Instant>>,
) -> Ran {
    let rate = 48_000.0;
    let voices = c.voices;
    let _rt = ca72_rt::promote(period, period / 2);
    ca72_rt::watch();
    let wg = host_wg
        .then(|| ca72_rt::WorkInterval::new("scaling_bench host"))
        .flatten();
    // (Left before the thread ends: a thread that ends joined stops the process.)
    let token = wg.as_ref().and_then(ca72_rt::WorkInterval::join);
    let roots = [45u8, 41, 43, 48, 40, 45, 50, 43];
    let shape = [0u8, 7, 12, 16, 19, 24, 28, 31, 34, 36];
    let beat = (0.5 * rate) as usize;
    let warm = (2.0 * rate) as usize / block;
    let blocks = (seconds * rate / block as f64) as usize;
    let mut took = Vec::with_capacity(blocks);
    let mut peak = 0.0f32;
    let mut held: Vec<u8> = Vec::with_capacity(16);
    let mut events = Vec::with_capacity(64);
    let (mut lb, mut rb) = (vec![0.0f32; block], vec![0.0f32; block]);
    go.wait();
    let start = *start_at
        .lock()
        .unwrap()
        .get_or_insert_with(|| Instant::now() + Duration::from_millis(5));
    let mut next = 0u32;
    let mut silent_from = 0;
    for i in 0..warm + blocks {
        let due = start + period * next;
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
        let t = Instant::now();
        if let Some(w) = &wg {
            w.start(period);
        }
        e.set_deadline(Some(t + period.mul_f64(DEADLINE)));
        let at = i * block;
        events.clear();
        for s in 0..block {
            let n = at + s;
            if n % beat == beat * 7 / 8 {
                for key in held.drain(..) {
                    events.push((s, Event::Note { key, on: false }));
                }
            }
            if n.is_multiple_of(beat) {
                let b = n / beat + k * 3;
                let r = roots[b % roots.len()] + (b / roots.len() % 3) as u8;
                for s2 in shape.iter().take(voices) {
                    events.push((
                        s,
                        Event::Note {
                            key: r + s2,
                            on: true,
                        },
                    ));
                    held.push(r + s2);
                }
            }
        }
        let (mut j, mut ev) = (0, 0);
        while j < block {
            while ev < events.len() && events[ev].0 <= j {
                e.event(events[ev].1);
                ev += 1;
            }
            if j == 0 {
                c.panel.cutoff = 0.5 + 0.2 * (at as f64 * 2.0 * std::f64::consts::PI / rate).sin();
                e.set(&c);
            }
            let end = events.get(ev).map_or(block, |x| x.0).min(block);
            e.render(&[], &mut lb[j..end], &mut rb[j..end]);
            j = end;
        }
        for (l, r) in lb.iter().zip(&rb) {
            peak = peak.max(l.abs()).max(r.abs());
        }
        e.end_block(block);
        if let Some(w) = &wg {
            w.finish();
        }
        if i == warm {
            silent_from = e.silent_runs();
        }
        if i >= warm {
            took.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let elapsed = start.elapsed().as_secs_f64() / period.as_secs_f64();
        next = (next + 1).max(elapsed.ceil() as u32);
    }
    if let (Some(w), Some(t)) = (&wg, token) {
        w.leave(t);
    }
    Ran {
        took,
        silent: e.silent_runs() - silent_from,
        peak,
    }
}

/// The system's one-minute load average, for the record (macOS and Linux).
fn load_average() -> String {
    std::process::Command::new("uptime")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.rsplit(':').next().map(|x| {
                x.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches(',')
                    .to_owned()
            })
        })
        .unwrap_or_default()
}
