//! DRIVE's AUTO GAIN measured on the factory presets (decisions.md R-STEREO, the CA-74's R29).
//! Each preset's curve is measured as the plug-in measures it (`drive.rs`: three short notes
//! through voices made as the engine's first are, without DRIVE and at each of its steps), on
//! one thread, timed; then the preset's levelling phrase (`preset_levels.rs`) and four held
//! chords are played at each of DRIVE's 3 dB steps with AUTO GAIN on, and their loudness (the
//! phrase's momentary maximum, the chords' integrated) set against DRIVE off: what AUTO GAIN
//! leaves over, with the preset's own curve and with the average's standing in for it. Prints
//! each preset's, the summary, the measurements' times and the presets' mean curve
//! ([`AVERAGE`] is it, rounded).
//!
//! By hand: `cargo test --release -p ca72-plugin --test drive_auto -- --ignored --nocapture`
//! (`CA72_ONLY`, a part of a preset's name; `CA72_RATE`, 48000). The test that is not ignored
//! plays one preset: AUTO GAIN brings it back within 1 dB at DRIVE's middle, and off it is
//! louder.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::loudness::loudness as measure;
use ca72_plugin::drive::{AVERAGE, Calibration, Curve, HEARD, STEPS};
use ca72_plugin::engine::Controls;
use ca72_plugin::library::{Sound, factory};
use common::{controls_of, env, phrase, play, played_as};
use std::time::{Duration, Instant};

/// DRIVE's settings played: every 3 dB, the curve's steps and the points between them.
const DRIVES: [f64; 8] = [3.0, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0];

/// Notes played: (start, length) in seconds and the keys held, and how long it all lasts.
type Material = (Vec<(f64, f64, Vec<u8>)>, f64);

/// The CA-74's: C, A minor, F and G, each held 1.4 s, voiced around middle C.
fn held() -> Material {
    let chords: [[u8; 4]; 4] = [
        [48, 60, 64, 67],
        [45, 60, 64, 69],
        [41, 60, 65, 69],
        [43, 59, 62, 67],
    ];
    let notes = chords
        .iter()
        .enumerate()
        .map(|(i, c)| (0.1 + i as f64 * 1.5, 1.4, c.to_vec()))
        .collect();
    (notes, 6.4)
}

/// A preset's levelling phrase, and its tail as `preset_levels.rs` plays it.
fn its_phrase(s: &Sound) -> Material {
    let notes = phrase(s);
    let end = notes.iter().map(|n| n.0 + n.1).fold(0.0, f64::max) + 1.5;
    (notes, end)
}

/// The curve of controls `c` as the plug-in measures it with the voices at `rate` Hz, and how
/// long that took.
fn measured(c: &Controls, rate: f64) -> (Curve<STEPS>, Duration) {
    let cal = Calibration::default();
    cal.set_rate(rate);
    cal.ask(c);
    let t = Instant::now();
    cal.measure_now();
    (Curve(cal.saved().db), t.elapsed())
}

/// The loudness of `c` (with AUTO GAIN's `curve`) playing `material`: momentary maximum, or
/// integrated.
fn loudness(
    c: &Controls,
    curve: Curve<STEPS>,
    material: &Material,
    integrated: bool,
    rate: f64,
) -> f64 {
    let (l, r) = play(c, Some(curve), &material.0, material.1, rate);
    let m = measure(&[&l, &r], rate as u32);
    if integrated {
        m.integrated
    } else {
        m.momentary_max
    }
    .unwrap_or(f64::NEG_INFINITY)
}

/// What AUTO GAIN leaves over at each of [`DRIVES`], dB louder than DRIVE off: on the phrase
/// and on the chords, with `curve`.
fn left_over(s: &Sound, curve: Curve<STEPS>, rate: f64) -> [[f64; DRIVES.len()]; 2] {
    let c = played_as(s, controls_of(s));
    let materials = [(its_phrase(s), false), (held(), true)];
    let mut out = [[0.0; DRIVES.len()]; 2];
    for (m, (material, integrated)) in materials.iter().enumerate() {
        let off = loudness(&c, curve, material, *integrated, rate);
        for (i, d) in DRIVES.iter().enumerate() {
            let mut on = c;
            (on.drive, on.auto_gain) = (*d, true);
            out[m][i] = loudness(&on, curve, material, *integrated, rate) - off;
        }
    }
    out
}

struct Row {
    name: String,
    curve: Curve<STEPS>,
    took: Duration,
    own: [[f64; DRIVES.len()]; 2],
    average: [[f64; DRIVES.len()]; 2],
}

fn summary(label: &str, errors: &[f64]) {
    let n = errors.len().max(1) as f64;
    let rms = (errors.iter().map(|e| e * e).sum::<f64>() / n).sqrt();
    let most = errors.iter().fold(0.0f64, |m, e| m.max(e.abs()));
    let over = errors.iter().filter(|e| e.abs() > 1.0).count();
    eprintln!(
        "{label:<44} {rms:>5.2} dB RMS, worst {most:>5.2} dB, {over:>3} of {} over 1 dB",
        errors.len()
    );
}

#[test]
#[ignore = "a measurement, run by hand"]
fn auto_on_the_factory_presets() {
    let rate: f64 = env("CA72_RATE", 48_000.0);
    let only = std::env::var("CA72_ONLY").ok();
    let presets: Vec<Sound> = factory()
        .iter()
        .filter(|s| only.as_ref().is_none_or(|o| s.name.contains(o.as_str())))
        .cloned()
        .collect();
    // The measurements first, one after another on this thread alone (their times).
    let curves: Vec<(Curve<STEPS>, Duration)> = presets
        .iter()
        .map(|s| measured(&controls_of(s), rate))
        .collect();
    // Then the presets played, in parallel.
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    let jobs: Vec<(Sound, Curve<STEPS>, Duration)> = presets
        .iter()
        .cloned()
        .zip(curves)
        .map(|(s, (c, t))| (s, c, t))
        .collect();
    let rows: Vec<Row> = std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .chunks(jobs.len().div_ceil(threads).max(1))
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|(s, curve, took)| Row {
                            name: s.name.clone(),
                            curve: *curve,
                            took: *took,
                            own: left_over(s, *curve, rate),
                            average: left_over(s, AVERAGE, rate),
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    eprintln!(
        "DRIVE's AUTO GAIN on the factory presets at {rate} Hz: dB left over against DRIVE off"
    );
    eprintln!(
        "{:<18} {:>8}  {:<36} {:<48} {:<48}",
        "preset", "measure", "curve (dB at 6, 12, 18, 24)", "phrase, DRIVE 3..24", "chords"
    );
    let cells = |v: &[f64]| {
        v.iter()
            .map(|x| format!("{x:+5.1}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    for r in &rows {
        let curve: Vec<f64> = r.curve.0.iter().map(|&x| f64::from(x)).collect();
        eprintln!(
            "{:<18} {:>6.0} ms  {:<36} {:<48} {:<48}",
            r.name,
            r.took.as_secs_f64() * 1e3,
            cells(&curve),
            cells(&r.own[0]),
            cells(&r.own[1]),
        );
    }
    let pick = |f: &dyn Fn(&Row) -> [[f64; DRIVES.len()]; 2], m: usize, upto: f64, from: f64| {
        rows.iter()
            .flat_map(|r| {
                let v = f(r)[m];
                DRIVES
                    .iter()
                    .zip(v)
                    .filter(|(d, _)| **d >= from && **d <= upto)
                    .map(|(_, e)| e)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<f64>>()
    };
    let own = |r: &Row| r.own;
    let average = |r: &Row| r.average;
    eprintln!();
    for (m, material) in ["the phrases", "the chords"].iter().enumerate() {
        summary(
            &format!("own curve, {material}, DRIVE 3-24"),
            &pick(&own, m, 24.0, 0.0),
        );
        summary(
            &format!("own curve, {material}, DRIVE 3-12"),
            &pick(&own, m, 12.0, 0.0),
        );
        summary(
            &format!("own curve, {material}, DRIVE 15-24"),
            &pick(&own, m, 24.0, 15.0),
        );
        summary(
            &format!("average curve, {material}, DRIVE 3-24"),
            &pick(&average, m, 24.0, 0.0),
        );
        summary(
            &format!("average curve, {material}, DRIVE 3-12"),
            &pick(&average, m, 12.0, 0.0),
        );
    }
    let mut times: Vec<f64> = rows.iter().map(|r| r.took.as_secs_f64() * 1e3).collect();
    times.sort_by(f64::total_cmp);
    eprintln!(
        "\na measurement on one thread: median {:.0} ms, longest {:.0} ms ({} renders of {} s)",
        times[times.len() / 2],
        times[times.len() - 1],
        STEPS + 1,
        HEARD
    );
    let mean: Vec<String> = (0..STEPS)
        .map(|i| {
            let m = rows.iter().map(|r| f64::from(r.curve.0[i])).sum::<f64>() / rows.len() as f64;
            format!("{m:.2}")
        })
        .collect();
    eprintln!("the presets' mean curve: [{}]", mean.join(", "));
}

/// Lead at 6 and 12 dB of DRIVE: with AUTO GAIN and its own curve, its phrase within 1 dB of
/// its loudness without DRIVE; without AUTO GAIN, louder by more than half of DRIVE. (At 16 kHz:
/// a debug build's voice is slow.)
#[test]
fn auto_brings_a_preset_back_and_off_it_is_louder() {
    let rate = 16_000.0;
    let s = factory().iter().find(|s| s.name == "Lead").unwrap();
    let c = played_as(s, controls_of(s));
    let (curve, _) = measured(&c, rate);
    let material = its_phrase(s);
    let off = loudness(&c, curve, &material, false, rate);
    for d in [6.0, 12.0] {
        let mut on = c;
        on.drive = d;
        let auto = loudness(&on, curve, &material, false, rate) - off;
        assert!(
            auto.abs() < 1.0,
            "DRIVE {d} dB with AUTO GAIN: {auto:+.2} dB"
        );
        on.auto_gain = false;
        let loud = loudness(&on, curve, &material, false, rate) - off;
        assert!(
            loud > 0.5 * d,
            "DRIVE {d} dB without AUTO GAIN: {loud:+.2} dB"
        );
    }
}

/// A measurement beside the voices (the CA-74's): a preset (`CA72_ONLY`, Brass Tutti) playing
/// POLY's ten-note chords in real time, a 256-frame block at 48 kHz every 5.3 ms, its workers
/// ordinary threads (`CA72_WORKERS`, 3), for `CA72_SECONDS` (10) with nothing else running,
/// then again with AUTO GAIN's measurements one after another on a thread of their own, as the
/// helper makes them for a sound changed without pause. Prints each block's time as a share
/// of its period (the median, the 99.9th percentile, the worst) and how many measurements were
/// made. Neither the voices nor the measurements are promoted (in a host the audio thread and
/// the workers are real-time threads, the helper is not), so this is the measurements' worst
/// case.
#[test]
#[ignore = "a measurement, run by hand"]
fn a_measurement_beside_the_voices() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    let (rate, block) = (48_000.0, 256usize);
    let seconds: f64 = env("CA72_SECONDS", 10.0);
    let workers: usize = env("CA72_WORKERS", 3);
    let name = std::env::var("CA72_ONLY").unwrap_or_else(|_| "Brass Tutti".to_owned());
    let s = factory()
        .iter()
        .find(|s| s.name.contains(name.as_str()))
        .unwrap();
    let mut c = controls_of(s);
    (c.poly, c.voices, c.drive) = (true, 10, 12.0);
    let period = Duration::from_secs_f64(block as f64 / rate);
    let blocks = (seconds / period.as_secs_f64()) as usize;
    eprintln!(
        "{}, ten POLY voices, DRIVE 12 dB, AUTO GAIN, {workers} workers, {blocks} blocks paced:",
        s.name
    );
    for busy in [false, true] {
        let stop = Arc::new(AtomicBool::new(false));
        let made = Arc::new(AtomicUsize::new(0));
        let beside = busy.then(|| {
            let (stop, made) = (stop.clone(), made.clone());
            std::thread::spawn(move || {
                let mut k = 0u32;
                while !stop.load(Ordering::Relaxed) {
                    let mut changed = c;
                    changed.panel.cutoff = f64::from(k % 20) / 20.0;
                    measured(&changed, rate);
                    made.fetch_add(1, Ordering::Relaxed);
                    k += 1;
                }
            })
        });
        let mut e = ca72_plugin::engine::Engine::new();
        e.set(&c);
        e.prepare(rate, 1);
        e.start_workers(workers, None);
        let mut chords = common::Chords::new(10, rate);
        let (mut l, mut r) = (vec![0.0f32; block], vec![0.0f32; block]);
        let mut events = Vec::with_capacity(64);
        let mut took = Vec::with_capacity(blocks);
        let start = Instant::now();
        for i in 0..blocks {
            let due = start + period * i as u32;
            if let Some(wait) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
            let t = Instant::now();
            common::play_block(&mut e, &mut chords, i * block, &mut l, &mut r, &mut events);
            std::hint::black_box((&l, &r));
            took.push(t.elapsed().as_secs_f64() / period.as_secs_f64());
        }
        stop.store(true, Ordering::Relaxed);
        if let Some(h) = beside {
            h.join().unwrap();
        }
        took.sort_by(f64::total_cmp);
        let at = |q: f64| took[((took.len() - 1) as f64 * q) as usize] * 100.0;
        eprintln!(
            "  {:<34} median {:>4.1} %, 99.9th {:>5.1} %, worst {:>5.1} % of the period{}",
            if busy {
                "with measurements beside them:"
            } else {
                "alone:"
            },
            at(0.5),
            at(0.999),
            at(1.0),
            if busy {
                format!(" ({} measurements)", made.load(Ordering::Relaxed))
            } else {
                String::new()
            }
        );
    }
}
