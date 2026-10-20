//! The LFOs at their worst (decisions.md R-LFO-STRESS): the test presets of
//! `docs/lfo-stress/presets` (each aimed at one weakness: fast LFOs on pitch and CUTOFF at strong
//! EMPHASIS, SHARP and SOFT edges, four LFOs clamped, MONO off with RETRIG, RISE and ENV, timed at
//! 200 BPM, a CONTINUOUS WAVEFORM, FEEDBACK and DETUNE, the heaviest patch), read as the library
//! reads them and played through the plug-in's own `process` as a host renders offline:
//! 256-frame blocks at 48 kHz, each preset's MIDI phrase, the transport playing.
//!
//! - `zipper`: each preset rendered as built (the LFOs moving their knobs every `lfo::STEP`
//!   samples, 32) and as a reference with the LFOs every sample
//!   (`engine::set_modulation_step_for_measurement`, the feature `lfo-lab`), twice: as set, and
//!   with each LFO's PHASE moved back by the 15.5 samples a 32-sample step holds its value late
//!   on average (aligned). A render that moves a pitch drifts from the reference in phase (the
//!   steps' average pitch is a hair from the smooth one's), so no figure here compares
//!   waveforms with the reference. Per preset and QUALITY it prints:
//!   - the peak, and the samples not finite and clipped (`bad`);
//!   - `pulse`: how strongly the high band's envelope pulses at 1.5 kHz and its next two
//!     harmonics, against the aligned reference, dB (`tests/cutoff_steps.rs`'s figure, issue
//!     #107), and its floor, the reference as set against the aligned one;
//!   - `bands`: what the render adds, band by band, to the aligned reference ([`band_nmr`]'s
//!     99th percentile), and its floor the same way; only the difference is read;
//!   - `grid`: the zipper by the grid, the render against itself with the steps' grid moved by
//!     half a step (the host's first block 16 samples long), dB against the music: every odd
//!     image about 1.5 kHz from each partial changes sign with it and nothing else does. Its
//!     floor (the reference rendered both ways) must be the same to the bit; `?` marks where it
//!     is not, `*` where the preset moves a pitch (an upper bound then, [`moves_a_pitch`]);
//!   - `edge`: how much LFO 1's edges stand out in the high band (clicks), and the reference's;
//!   - `jump`: the largest change of level from one 2 ms window to the next, and the
//!     reference's;
//!   - the processor time a second of audio took, every thread (Linux), and the reference's.
//!
//!   `CA72_LFO_OUT=<dir>` writes each render and its reference as WAVs there, at one gain a
//!   preset (the louder's peak at -14 dBFS), and `metrics-<label>.csv`. `CA72_LFO_STEP` renders
//!   at another step instead of the built one (for measuring the choice).
//! - `cost`: each preset's controls in the engine on one thread, no workers, 4 s of its
//!   phrase, best of three, against the same with nothing assigned (and at `CA72_LFO_STEP`).
//! - `paced`: the heaviest preset in real time, 256-frame blocks with the plug-in's deadline and
//!   its own workers (`CA72_WORKERS` to give fewer), against the same with nothing assigned: the
//!   voices' runs not heard and the blocks late.
//!
//! By hand, `--features lfo-lab` (the reference's switch, never in a plug-in build):
//! `cargo test --release -p ca72-plugin --features lfo-lab --test lfo_stress -- --ignored
//! --nocapture --test-threads 1`. `CA72_QUALITY` (`HI`; a list, `HI,LO,ULTRA`), `CA72_ONLY`
//! (the presets' numbers, `03,15`), `CA72_LFO_PRESETS` (another folder of presets),
//! `CA72_LFO_LABEL` (a word in the WAVs' names, `built`).

#![allow(unsafe_code, clippy::unwrap_used)]

mod common;

use ca72_analysis::fft::{hann, power};
use ca72_plugin::Ca72;
use ca72_plugin::engine::{AUTO, Controls, DEADLINE, Engine, Event};
use ca72_plugin::library::{Library, Sound};
use ca72_plugin::params::Ca72Params;
use ca72_plugin::presets::sound_params;
use common::env;
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;
/// Each render's length, s (the notes from 0.25 s to 7.4 s, then their release).
const SECONDS: f64 = 8.0;
/// The part measured: the notes held, past their start.
const MEASURED: (f64, f64) = (0.6, 7.3);
/// The LFOs' step (`lfo::STEP`): 1.5 kHz at 48 kHz, the rate the pulse is measured at.
const STEP: usize = ca72_plugin::lfo::STEP;
const SEED: u64 = 42;
/// The WAVs' loudest peak, dBFS.
const PEAK_DBFS: f64 = -14.0;

struct Init;
impl InitContext<Ca72> for Init {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn execute(&self, _: ()) {}
    fn set_latency_samples(&self, _: u32) {}
    fn set_current_voice_capacity(&self, _: u32) {}
}

/// The test presets, by number (`LFO TEST nn ...`), as the library reads their folder.
fn presets() -> Vec<(u32, Sound)> {
    let dir = std::env::var_os("CA72_LFO_PRESETS").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lfo-stress/presets"),
        PathBuf::from,
    );
    let (entries, bad) = Library::at(&dir).entries();
    assert!(bad.is_empty(), "presets that do not read: {bad:?}");
    let only: Option<Vec<u32>> = std::env::var("CA72_ONLY")
        .ok()
        .map(|s| s.split(',').map(|n| n.trim().parse().unwrap()).collect());
    let mut out: Vec<(u32, Sound)> = entries
        .into_iter()
        .filter_map(|e| {
            let n: u32 = e
                .sound
                .name
                .strip_prefix("LFO TEST ")?
                .get(..2)?
                .parse()
                .ok()?;
            Some((n, e.sound))
        })
        .filter(|(n, _)| only.as_ref().is_none_or(|o| o.contains(n)))
        .collect();
    out.sort_by_key(|(n, _)| *n);
    assert!(!out.is_empty(), "no LFO TEST preset in {}", dir.display());
    out
}

fn value(s: &Sound, id: &str) -> Option<f64> {
    s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v)
}

/// The host's tempo for a preset: 200 BPM for the timed one, else 120.
fn tempo(s: &Sound) -> f64 {
    let timed = (1..=4).any(|n| value(s, &format!("lfo{n}_timed")).is_some_and(|v| v >= 0.5));
    if timed { 200.0 } else { 120.0 }
}

/// A DIVISION's length in quarter notes (the timed oscillators' 18, 4 bars first).
fn beats_of(division: usize) -> f64 {
    const B: [f64; 18] = [
        16.0,
        8.0,
        4.0,
        3.0,
        8.0 / 3.0,
        2.0,
        1.5,
        4.0 / 3.0,
        1.0,
        0.75,
        2.0 / 3.0,
        0.5,
        0.375,
        1.0 / 3.0,
        0.25,
        0.1875,
        1.0 / 6.0,
        0.125,
    ];
    B[division.min(17)]
}

/// The preset's values with each assigned LFO's PHASE moved back by `samples` of its cycle.
fn delayed(s: &Sound, samples: f64) -> Sound {
    let mut s = s.clone();
    let bpm = tempo(&s);
    for n in 1..=4 {
        let assigned = s
            .values
            .iter()
            .any(|(k, v)| k.starts_with(&format!("lfo{n}_to_")) && *v != 0.0);
        if !assigned {
            continue;
        }
        let hz = if value(&s, &format!("lfo{n}_timed")).is_some_and(|v| v >= 0.5) {
            let d = value(&s, &format!("lfo{n}_division")).unwrap_or(8.0) as usize;
            bpm / 60.0 / beats_of(d)
        } else {
            value(&s, &format!("lfo{n}_rate")).unwrap_or(2.0)
        };
        let id = format!("lfo{n}_phase");
        let phase = value(&s, &id).unwrap_or(0.0);
        let moved = (phase - samples * hz / RATE * 360.0).rem_euclid(360.0);
        s.values.retain(|(k, _)| *k != id);
        s.values.push((id, moved));
    }
    s
}

/// A preset's phrase: each key's (on, off) in seconds.
fn phrase(n: u32) -> Vec<(f64, f64, u8)> {
    let held = |keys: &[u8]| keys.iter().map(|&k| (0.25, 7.4, k)).collect::<Vec<_>>();
    match n {
        // POLY chords.
        2 | 4 | 5 | 6 | 9 | 14 => held(&[48, 55, 60, 64]),
        // Fast POLY notes, RETRIGGER on: a sixteenth line overlapping, a chord every 0.8 s,
        // more notes at once than voices.
        10 => {
            let mut out = Vec::new();
            let line = [60u8, 63, 67, 70, 72, 70, 67, 63, 58, 62, 65, 69];
            let mut t = 0.25;
            let mut k = 0;
            while t < 7.2 {
                out.push((t, t + 0.14, line[k % line.len()]));
                if k % 8 == 0 {
                    for d in [0u8, 3, 7, 10, 14] {
                        out.push((t, t + 0.6, 43 + d + (k / 8 % 4) as u8));
                    }
                }
                t += 0.1;
                k += 1;
            }
            out
        }
        // A note a second (RISE and ENV start at each).
        11 => (0..7)
            .map(|k| {
                let t = 0.25 + f64::from(k);
                (t, t + 0.8, [48u8, 51, 55, 58, 53, 50, 46][k as usize])
            })
            .collect(),
        // Ten-note chords every half second, held seven eighths of it (`preset_cost.rs`'s).
        15 => {
            const ROOTS: [u8; 8] = [45, 41, 43, 48, 40, 45, 50, 43];
            const SHAPE: [u8; 10] = [0, 7, 12, 16, 19, 24, 28, 31, 34, 36];
            let mut out = Vec::new();
            for b in 0..14usize {
                let t = 0.25 + 0.5 * b as f64;
                for d in SHAPE {
                    out.push((t, t + 0.4375, ROOTS[b % 8] + d));
                }
            }
            out
        }
        // One held C3.
        _ => held(&[48]),
    }
}

/// The phrase as note events at their samples, in order (offs first at a sample).
fn events(n: u32) -> Vec<(usize, u8, bool)> {
    let at = |t: f64| (t * RATE).round() as usize;
    let mut e: Vec<(usize, u8, bool)> = phrase(n)
        .into_iter()
        .flat_map(|(on, off, k)| [(at(on), k, true), (at(off), k, false)])
        .collect();
    e.sort_by_key(|&(t, k, on)| (t, on, k));
    e
}

fn quality_normalized(q: &str) -> f32 {
    match q {
        "LO" => 0.0,
        "HI" => 0.5,
        "ULTRA" => 1.0,
        other => panic!("QUALITY {other}: LO, HI or ULTRA"),
    }
}

/// The preset set on `a` as a MIDI pad's preset is (each parameter its value, else its
/// default, landing at once; its LFO assignments and EDGES), and QUALITY `q`, by `c`.
fn set_preset(a: &Arc<Ca72Params>, c: &TestProcessContext<Ca72>, s: &Sound, q: &str) {
    let map = a.param_map();
    let ptr = |id: &str| map.iter().find(|(i, ..)| i == id).map(|(_, p, _)| *p);
    let params = sound_params(a);
    for (k, _) in &s.values {
        let patch = k.starts_with("lfo") && (k.contains("_to_") || k.ends_with("_edges"));
        assert!(
            patch || params.iter().any(|(id, _)| id == k),
            "{}: no parameter {k}",
            s.name
        );
    }
    let landing = a.lands.begin();
    for (id, q) in &params {
        if ca72_plugin::presets::PLAYERS.contains(id) {
            continue;
        }
        let v = value(s, id).map_or_else(|| q.default_normalized(), |v| q.normalized_of(v));
        a.lands.mark(landing, a, id, v);
        assert!(c.automate(ptr(id).unwrap(), v), "{id}");
    }
    assert!(c.automate(a.quality.as_ptr(), quality_normalized(q)));
    a.lfo_patch.load_values(&s.values);
}

/// The processor time this process has had, s (Linux; 0 elsewhere).
fn cpu_seconds() -> f64 {
    let Ok(stat) = std::fs::read_to_string("/proc/self/stat") else {
        return 0.0;
    };
    let f: Vec<&str> = stat
        .rsplit(')')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect();
    let tick = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    (tick(11) + tick(12)) / 100.0
}

/// A render: left, right, and the processor time and wall time it took, s.
struct Render {
    l: Vec<f32>,
    r: Vec<f32>,
    cpu: f64,
}

/// Preset `s` (number `n`) played through the plug-in's `process` at QUALITY `q`, offline,
/// the LFOs moving their knobs every `step` samples (0: as built), the host's blocks
/// [`BLOCK`] long but the first, `first` long (the same sound, its steps' grid moved).
fn render(n: u32, s: &Sound, q: &str, step: usize, first: usize) -> Render {
    ca72_plugin::engine::set_modulation_step_for_measurement(step);
    let mut p = Ca72::default();
    p.parameters()
        .noise_seed
        .store(SEED, std::sync::atomic::Ordering::Relaxed);
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: BLOCK as u32,
        process_mode: ProcessMode::Offline,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    let a = p.parameters().clone();
    let mut c = TestProcessContext::<Ca72>::new(p.params(), RATE as f32, ProcessMode::Offline);
    set_preset(&a, &c, s, q);
    let bpm = tempo(s);
    let len = (SECONDS * RATE) as usize;
    let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
    let (mut bl, mut br) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    let ev = events(n);
    let mut next = ev.iter().peekable();
    let cpu = cpu_seconds();
    let mut start = 0;
    while start < len {
        let block = if start == 0 { first } else { BLOCK }.min(len - start);
        let beats = start as f64 / RATE * bpm / 60.0;
        c.set_transport(true, Some(bpm), Some(beats), Some((4, 4)));
        while let Some(&&(t, key, on)) = next.peek() {
            if t >= start + block {
                break;
            }
            let timing = (t - start) as u32;
            c.push_event(if on {
                NoteEvent::NoteOn {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note: key,
                    velocity: 1.0,
                }
            } else {
                NoteEvent::NoteOff {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note: key,
                    velocity: 0.0,
                }
            });
            next.next();
        }
        let mut buffer = Buffer::default();
        // SAFETY: both slices are at least `block` long and outlive the buffer.
        unsafe {
            buffer.set_slices(block, |b| {
                b.clear();
                b.push(&mut bl[..block]);
                b.push(&mut br[..block]);
            });
        }
        p.process(&mut buffer, &mut aux, &mut c);
        drop(buffer);
        l[start..start + block].copy_from_slice(&bl[..block]);
        r[start..start + block].copy_from_slice(&br[..block]);
        start += block;
    }
    let took = cpu_seconds() - cpu;
    p.deactivate();
    ca72_plugin::engine::set_modulation_step_for_measurement(0);
    Render { l, r, cpu: took }
}

fn mono(x: &Render) -> Vec<f64> {
    x.l.iter()
        .zip(&x.r)
        .map(|(a, b)| 0.5 * (f64::from(*a) + f64::from(*b)))
        .collect()
}

fn span(x: &[f64]) -> &[f64] {
    &x[(MEASURED.0 * RATE) as usize..(MEASURED.1 * RATE) as usize]
}

fn rms(x: &[f64]) -> f64 {
    (x.iter().map(|v| v * v).sum::<f64>() / x.len().max(1) as f64).sqrt()
}

fn db(x: f64) -> f64 {
    20.0 * x.max(1e-300).log10()
}

fn peak(x: &Render) -> f64 {
    x.l.iter()
        .chain(&x.r)
        .fold(0.0f64, |m, v| m.max(f64::from(v.abs())))
}

fn not_finite(x: &Render) -> usize {
    x.l.iter().chain(&x.r).filter(|v| !v.is_finite()).count()
}

fn clipped(x: &Render) -> usize {
    x.l.iter().chain(&x.r).filter(|v| v.abs() >= 1.0).count()
}

/// The high band's envelope (the third difference's magnitude averaged over 4 samples) and its
/// power spectrum over all of `x` (Hann, zero-padded): the bins' power and their width, Hz.
fn envelope_spectrum(x: &[f64]) -> (Vec<f64>, f64) {
    const BIN: usize = 4;
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

/// The envelope's power at the step rate (1.5 kHz) and its next two harmonics, each the
/// strongest bin within two of it, summed.
fn at_step_rate(spectrum: &(Vec<f64>, f64)) -> f64 {
    let (p, width) = spectrum;
    let hz = RATE / STEP as f64;
    (1..=3)
        .map(|h| {
            let k = (h as f64 * hz / width).round() as usize;
            (k.saturating_sub(2)..=(k + 2).min(p.len() - 1))
                .map(|i| p[i])
                .fold(0.0, f64::max)
        })
        .sum()
}

/// The critical bands' edges, Hz (Zwicker's 24, and the last to 20 kHz).
const BARK: [f64; 26] = [
    0.0, 100.0, 200.0, 300.0, 400.0, 510.0, 630.0, 770.0, 920.0, 1080.0, 1270.0, 1480.0, 1720.0,
    2000.0, 2320.0, 2700.0, 3150.0, 3700.0, 4400.0, 5300.0, 6400.0, 7700.0, 9500.0, 12000.0,
    15500.0, 20000.0,
];

/// The threshold of hearing at `hz`, dB SPL (Terhardt).
fn quiet(hz: f64) -> f64 {
    let f = (hz / 1000.0).max(0.02);
    3.64 * f.powf(-0.8) - 6.5 * (-0.6 * (f - 3.3).powi(2)).exp() + 1e-3 * f.powi(4)
}

/// `x`'s power spectrum, a Welch average of Hann frames of `n` half overlapping, each bin
/// against a full-scale sine's power through the window (0 dB).
fn psd(x: &[f64], n: usize) -> Vec<f64> {
    let w = hann(n);
    let sine: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 997.0 * i as f64 / RATE).sin())
        .collect();
    let full: f64 = power(&sine, &w).iter().sum();
    let mut acc = vec![0.0; n / 2 + 1];
    let mut frames = 0;
    let mut i = 0;
    while i + n <= x.len() {
        for (a, p) in acc.iter_mut().zip(power(&x[i..i + n], &w)) {
            *a += p;
        }
        frames += 1;
        i += n / 2;
    }
    acc.iter()
        .map(|a| a / full / f64::from(frames.max(1)))
        .collect()
}

/// What render `a` adds to the music `b` heard as the ear's bands hear it: each frame's (2048
/// samples, Hann, half overlapping) power in each critical band, `a`'s excess over `b`'s
/// against `b`'s masking: the music's power spread over the bands (27 dB a Bark below a band, 15
/// above) and lowered by 14.5 + z dB (a tonal masker's offset), never below the threshold of
/// hearing with the music played at `gain` (full scale at 100 dB SPL). Strict: two renders
/// every sample with the LFOs 15.5 samples apart already score high, so only the difference
/// between a figure and that floor is read. Blind to a partial's phase and to its pitch within
/// a band, so a render whose oscillators wander a cent from the other's still compares. Returns
/// the frames' largest, their 99th percentile and the share over 0 dB.
fn band_nmr(a: &[f64], b: &[f64], gain: f64) -> (f64, f64, f64) {
    const N: usize = 2048;
    let bin = RATE / N as f64;
    let bands = BARK.len() - 1;
    let band_of: Vec<Option<usize>> = (0..=N / 2)
        .map(|k| {
            let f = k as f64 * bin;
            (0..bands).find(|&j| f >= BARK[j] && f < BARK[j + 1])
        })
        .collect();
    let g2 = gain * gain;
    let mut frames = Vec::new();
    let mut i = 0;
    while i + N <= a.len().min(b.len()) {
        let (pa, pb) = (psd(&a[i..i + N], N), psd(&b[i..i + N], N));
        let (mut m, mut e) = (vec![0.0f64; bands], vec![0.0f64; bands]);
        for k in 0..pa.len() {
            if let Some(j) = band_of[k] {
                m[j] += pb[k] * g2;
                e[j] += pa[k] * g2;
            }
        }
        let worst = (0..bands)
            .map(|j| {
                let zj = j as f64 + 0.5;
                let spread: f64 = (0..bands)
                    .map(|k| {
                        let dz = zj - (k as f64 + 0.5);
                        let s = if dz >= 0.0 { -15.0 * dz } else { 27.0 * dz };
                        m[k] * 10f64.powf(s / 10.0)
                    })
                    .sum();
                let mask = spread * 10f64.powf(-(14.5 + zj) / 10.0);
                let hearing = 10f64.powf((quiet(0.5 * (BARK[j] + BARK[j + 1])) - 100.0) / 10.0);
                10.0 * ((e[j] - m[j]).max(1e-30) / mask.max(hearing)).log10()
            })
            .fold(f64::MIN, f64::max);
        frames.push(worst);
        i += N / 2;
    }
    frames.sort_by(f64::total_cmp);
    let over = frames.iter().filter(|v| **v > 0.0).count() as f64 / frames.len() as f64;
    (
        *frames.last().unwrap(),
        frames[(frames.len() * 99 / 100).min(frames.len() - 1)],
        over,
    )
}

/// How much an LFO's edges stand out in the high band: the third difference's power folded
/// over the LFO's cycle (`hz`) into 100 places, the largest place against their median, dB.
/// A click at an edge is a spike there; a smooth edge none.
fn edge_prominence(x: &[f64], hz: f64) -> f64 {
    const PLACES: usize = 100;
    let mut fold = [0.0f64; PLACES];
    let mut count = [0usize; PLACES];
    for i in 3..x.len() {
        let d = x[i] - 3.0 * x[i - 1] + 3.0 * x[i - 2] - x[i - 3];
        let k = ((i as f64 / RATE * hz).fract() * PLACES as f64) as usize % PLACES;
        fold[k] += d * d;
        count[k] += 1;
    }
    let mut mean: Vec<f64> = fold
        .iter()
        .zip(&count)
        .map(|(f, c)| f / (*c).max(1) as f64)
        .collect();
    let top = mean.iter().copied().fold(0.0, f64::max);
    mean.sort_by(f64::total_cmp);
    10.0 * (top / mean[PLACES / 2].max(1e-300)).log10()
}

/// LFO 1's rate, Hz, at the preset's tempo.
fn lfo1_hz(s: &Sound) -> f64 {
    if value(s, "lfo1_timed").is_some_and(|v| v >= 0.5) {
        let d = value(s, "lfo1_division").unwrap_or(8.0) as usize;
        tempo(s) / 60.0 / beats_of(d)
    } else {
        value(s, "lfo1_rate").unwrap_or(2.0)
    }
}

/// The largest change of level from one 2 ms window to the next, dB (windows above -60 dBFS).
fn largest_jump(x: &[f64]) -> f64 {
    let w = (0.002 * RATE) as usize;
    let levels: Vec<f64> = x.chunks_exact(w).map(rms).collect();
    levels
        .windows(2)
        .filter(|p| p[0] > 1e-3 && p[1] > 1e-3)
        .map(|p| db(p[1] / p[0]).abs())
        .fold(0.0, f64::max)
}

/// A WAV of 24-bit samples, `gain` applied.
fn write_wav(path: &Path, l: &[f32], r: &[f32], gain: f64) {
    let frames = l.len().min(r.len());
    let data = (frames * 6) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(RATE as u32).to_le_bytes());
    b.extend_from_slice(&(RATE as u32 * 6).to_le_bytes());
    b.extend_from_slice(&6u16.to_le_bytes());
    b.extend_from_slice(&24u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for i in 0..frames {
        for x in [l[i], r[i]] {
            let v = (f64::from(x) * gain * 8_388_607.0)
                .round()
                .clamp(-8_388_608.0, 8_388_607.0) as i32;
            b.extend_from_slice(&v.to_le_bytes()[..3]);
        }
    }
    std::fs::write(path, b).unwrap();
}

fn file_stem(s: &Sound) -> String {
    s.name
        .strip_prefix("LFO TEST ")
        .unwrap_or(&s.name)
        .replace(' ', "-")
}

fn qualities() -> Vec<String> {
    std::env::var("CA72_QUALITY")
        .unwrap_or_else(|_| "HI".into())
        .split(',')
        .map(|q| q.trim().to_uppercase())
        .collect()
}

/// Whether the preset's LFOs move a pitch: TUNE, a FREQUENCY, DETUNE, or CUTOFF or EMPHASIS
/// with EMPHASIS at 8 or more (the filter ringing or whistling at its cutoff). The steps then
/// also move the average pitch a little, by where their grid falls, so the two stepped renders
/// drift apart in phase and their difference ([`zipper`]'s grid figure) is an upper bound.
fn moves_a_pitch(s: &Sound) -> bool {
    let ringing = value(s, "emphasis").is_some_and(|e| e >= 8.0);
    s.values.iter().any(|(k, v)| {
        *v != 0.0
            && k.starts_with("lfo")
            && k.split_once("_to_").is_some_and(|(_, t)| {
                matches!(t, "tune" | "osc2_frequency" | "osc3_frequency" | "double")
                    || (ringing && matches!(t, "cutoff" | "emphasis"))
            })
    })
}

/// The step a render of preset `s` at QUALITY `q` moves its knobs by: `tried` where it is not 0,
/// else as built (`lfo::FAST_STEP` where a pitch or CUTOFF is moved at HI or ULTRA, else
/// `lfo::STEP`).
fn step_of(s: &Sound, q: &str, tried: usize) -> usize {
    if tried != 0 {
        return tried;
    }
    let fast = s.values.iter().any(|(k, v)| {
        *v != 0.0
            && k.starts_with("lfo")
            && k.split_once("_to_")
                .and_then(|(_, t)| ca72_plugin::lfo::Target::of_id(t))
                .is_some_and(|t| t.fast())
    });
    if fast && q != "LO" {
        ca72_plugin::lfo::FAST_STEP
    } else {
        STEP
    }
}

#[test]
#[ignore = "a measurement, run by hand (--features lfo-lab)"]
fn zipper() {
    let out = std::env::var_os("CA72_LFO_OUT").map(PathBuf::from);
    let label = std::env::var("CA72_LFO_LABEL").unwrap_or_else(|_| "built".into());
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    let mut csv = String::from(
        "preset,quality,label,peak_dbfs,peak_reference_dbfs,not_finite,clipped,pulse_db,pulse_floor_db,bands_p99_db,bands_floor_p99_db,grid_db,grid_floor_db,moves_a_pitch,edge_db,edge_reference_db,jump_db,jump_reference_db,cpu_per_s,cpu_reference_per_s\n",
    );
    eprintln!(
        "{:<32} {:>5} {:>6} {:>5} {:>13} {:>13} {:>9} {:>11} {:>11} {:>11}",
        "preset",
        "Q",
        "peak",
        "bad",
        "pulse/floor",
        "bands/floor",
        "grid",
        "edge/ref",
        "jump/ref",
        "CPU s/s"
    );
    // The step tried (`CA72_LFO_STEP`, 0: as built).
    let tried: usize = env("CA72_LFO_STEP", 0);
    for q in qualities() {
        for (n, s) in presets() {
            // (Half the render's step: the second render's first block that long moves the
            // steps' grid by it. The aligned reference is late by the steps' average delay.)
            let step = step_of(&s, &q, tried);
            let half = (step / 2).max(1);
            let built = render(n, &s, &q, tried, BLOCK);
            let moved = render(n, &s, &q, tried, half);
            let set = render(n, &s, &q, 1, BLOCK);
            let set_moved = render(n, &s, &q, 1, half);
            let aligned = render(n, &delayed(&s, (step as f64 - 1.0) / 2.0), &q, 1, BLOCK);
            let (mb, mm, ms, msm, ma) = (
                mono(&built),
                mono(&moved),
                mono(&set),
                mono(&set_moved),
                mono(&aligned),
            );
            let loudest = peak(&built).max(peak(&set)).max(1e-12);
            let gain = 10f64.powf(PEAK_DBFS / 20.0) / loudest;
            // (1) The high band's pulse at 1.5 kHz against the reference's (aligned), and the
            // same for the reference as set (the figure's own floor).
            let pulse_of = |x: &[f64]| {
                db(at_step_rate(&envelope_spectrum(span(x)))
                    / at_step_rate(&envelope_spectrum(span(&ma))))
                    / 2.0
            };
            let (pu, pu0) = (pulse_of(&mb), pulse_of(&ms));
            // (2) What the steps add as the ear's bands hear it, and the floor: the reference
            // as set against the aligned one (the LFOs 15.5 samples apart, every sample).
            let (_, bp, _) = band_nmr(span(&mb), span(&ma), gain);
            let (_, bp0, _) = band_nmr(span(&ms), span(&ma), gain);
            // (3) The zipper by the grid: what the steps' grid moved by half a step changes,
            // halved. The images about 1.5 kHz (and 4.5, 7.5 kHz...) from every partial change
            // sign with it; the sound, its delay and the LFOs' timing stay. The images about 3
            // and 6 kHz cancel, so this is about three quarters of the zipper's power: raised by
            // 4/3 (a sawtooth error's harmonics, 1/k each). Its floor: the references rendered
            // the same two ways, which must be the same to the bit.
            let z: Vec<f64> = mb
                .iter()
                .zip(&mm)
                .map(|(a, b)| 0.5 * (a - b) * (4f64 / 3.0).sqrt())
                .collect();
            let z0: Vec<f64> = ms.iter().zip(&msm).map(|(a, b)| 0.5 * (a - b)).collect();
            let level = rms(span(&ms));
            let (grid, grid0) = (db(rms(span(&z)) / level), db(rms(span(&z0)) / level));
            let pitch = moves_a_pitch(&s);
            // (4) Clicks at LFO 1's edges; (5) the largest 2 ms jump of level.
            let hz = lfo1_hz(&s);
            let (eb, es) = (
                edge_prominence(span(&mb), hz),
                edge_prominence(span(&ms), hz),
            );
            let (jb, js) = (largest_jump(span(&mb)), largest_jump(span(&ms)));
            let bad = [&built, &moved, &set, &set_moved, &aligned]
                .iter()
                .map(|x| not_finite(x))
                .sum::<usize>();
            let clip = clipped(&built) + clipped(&set);
            let (cpu, cpu_ref) = (built.cpu / SECONDS, set.cpu / SECONDS);
            let grid_shown = if grid0 > -100.0 {
                format!("{grid:.1}?")
            } else if pitch {
                format!("{grid:.1}*")
            } else {
                format!("{grid:.1}")
            };
            eprintln!(
                "{:<32} {:>5} {:>6.1} {:>5} {:>+6.1}/{:<+6.1} {:>+6.1}/{:<+6.1} {:>9} {:>5.1}/{:<5.1} {:>5.1}/{:<5.1} {:>4.2}/{:<4.2}",
                s.name.strip_prefix("LFO TEST ").unwrap_or(&s.name),
                q,
                db(peak(&built)),
                format!("{bad}/{clip}"),
                pu,
                pu0,
                bp,
                bp0,
                grid_shown,
                eb,
                es,
                jb,
                js,
                cpu,
                cpu_ref,
            );
            let _ = writeln!(
                csv,
                "\"{}\",{q},{label},{:.2},{:.2},{bad},{clip},{pu:.2},{pu0:.2},{bp:.2},{bp0:.2},{grid:.2},{grid0:.2},{pitch},{eb:.2},{es:.2},{jb:.2},{js:.2},{cpu:.3},{cpu_ref:.3}",
                s.name,
                db(peak(&built)),
                db(peak(&set)),
            );
            if let Some(d) = &out {
                let stem = format!("{}-{q}", file_stem(&s));
                write_wav(
                    &d.join(format!("{stem}-{label}.wav")),
                    &built.l,
                    &built.r,
                    gain,
                );
                write_wav(
                    &d.join(format!("{stem}-reference-every-sample.wav")),
                    &set.l,
                    &set.r,
                    gain,
                );
            }
        }
    }
    if let Some(d) = &out {
        std::fs::write(d.join(format!("metrics-{label}.csv")), csv).unwrap();
    }
}

/// Preset `s`'s controls at QUALITY `q`, as its plug-in has them once set.
fn controls_of_preset(s: &Sound, q: &str) -> Controls {
    let p = Ca72::default();
    let a = p.parameters().clone();
    let c = TestProcessContext::<Ca72>::new(p.params(), RATE as f32, ProcessMode::Offline);
    set_preset(&a, &c, s, q);
    a.controls()
}

/// The phrase of preset `n` played by an engine with `c`, `block` frames at a time, given the
/// plug-in's deadline and played at its time if `paced`, with `workers` workers: the seconds
/// one second of audio took, the voices' runs not heard and the blocks late.
fn play(c: &Controls, n: u32, seconds: f64, workers: usize, paced: bool) -> (f64, u64, usize) {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, SEED);
    e.start_workers(workers, None);
    let ev = events(n);
    let mut next = ev.iter().peekable();
    let len = (seconds * RATE) as usize / BLOCK * BLOCK;
    let (mut l, mut r) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
    let period = Duration::from_secs_f64(BLOCK as f64 / RATE);
    let start = Instant::now();
    let mut busy = Duration::ZERO;
    let mut late = 0;
    for (i, b) in (0..len).step_by(BLOCK).enumerate() {
        if paced {
            let due = start + period * i as u32;
            if let Some(w) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(w);
            }
            e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            if e.take_serving() {
                e.crew().serve();
            }
        }
        let t = Instant::now();
        let mut k = b;
        while k < b + BLOCK {
            while let Some(&&(_, key, on)) = next.peek().filter(|x| x.0 <= k) {
                e.event(Event::Note { key, on });
                next.next();
            }
            let stop = next
                .peek()
                .map_or(b + BLOCK, |x| x.0.clamp(k + 1, b + BLOCK));
            e.render(&[], &mut l[k - b..stop - b], &mut r[k - b..stop - b]);
            k = stop;
        }
        e.end_block(BLOCK);
        if e.take_mending() {
            e.spares().mend();
        }
        let took = t.elapsed();
        busy += took;
        late += usize::from(paced && took > period);
    }
    (busy.as_secs_f64() / seconds, e.silent_runs(), late)
}

#[test]
#[ignore = "a measurement, run by hand (--features lfo-lab)"]
fn cost() {
    for q in qualities() {
        for (n, s) in presets() {
            if tempo(&s) != 120.0 {
                // (The engine alone has no host clock: the timed preset is the plug-in's.)
                continue;
            }
            let c = controls_of_preset(&s, &q);
            let mut none = c;
            none.lfo = Default::default();
            let best = |c: &Controls| {
                (0..3)
                    .map(|_| play(c, n, 4.0, 0, false).0)
                    .fold(f64::MAX, f64::min)
            };
            let (off, on) = (best(&none), best(&c));
            // (And at the step `CA72_LFO_STEP` asks for, if any.)
            let step: usize = env("CA72_LFO_STEP", 0);
            let at_step = if step == 0 {
                String::new()
            } else {
                ca72_plugin::engine::set_modulation_step_for_measurement(step);
                let t = best(&c);
                ca72_plugin::engine::set_modulation_step_for_measurement(0);
                format!(
                    ", every {step} samples {:>6.1} ({:+.1} %)",
                    t * 1e3,
                    (t / off - 1.0) * 100.0
                )
            };
            eprintln!(
                "{:<34} {q:>5}: nothing assigned {:>6.1} ms a second of audio, as built {:>6.1} ({:+.1} %){at_step}",
                s.name.strip_prefix("LFO TEST ").unwrap_or(&s.name),
                off * 1e3,
                on * 1e3,
                (on / off - 1.0) * 100.0
            );
        }
    }
}

#[test]
#[ignore = "a measurement, run by hand (--features lfo-lab)"]
fn paced() {
    let workers: usize = env("CA72_WORKERS", AUTO);
    if workers != AUTO {
        ca72_plugin::engine::set_budget_for_measurement(workers);
    }
    let (n, s) = presets()
        .into_iter()
        .find(|(n, _)| *n == 15)
        .expect("LFO TEST 15");
    for q in qualities() {
        let c = controls_of_preset(&s, &q);
        let mut none = c;
        none.lfo = Default::default();
        for (what, c) in [("as set", c), ("nothing assigned", none)] {
            let (busy, silent, late) = play(&c, n, SECONDS, workers, true);
            eprintln!(
                "paced {} {q} {what}: {silent} voices' runs not heard, {late} blocks late, the mean block {:.0} % of its period",
                s.name,
                100.0 * busy
            );
        }
    }
}

/// Whether a render depends on where the host's blocks fall: each preset with nothing assigned,
/// and its reference every sample, the first block 256 and 16 samples long. (At HI every one
/// is the same to the bit; at LO the one instrument's are not, whatever is assigned: its light
/// voice plays a run at a time, R-LFO-STRESS.)
#[test]
#[ignore = "a measurement, run by hand (--features lfo-lab)"]
fn layouts() {
    for q in qualities() {
        for (n, s) in presets() {
            let mut none = s.clone();
            none.values.retain(|(k, _)| !k.contains("_to_"));
            let same = |a: &Render, b: &Render| {
                a.l.iter()
                    .chain(&a.r)
                    .zip(b.l.iter().chain(&b.r))
                    .all(|(x, y)| x.to_bits() == y.to_bits())
            };
            let apart = |a: &Render, b: &Render| {
                let (ma, mb) = (mono(a), mono(b));
                let d: Vec<f64> = ma.iter().zip(&mb).map(|(x, y)| x - y).collect();
                if same(a, b) {
                    "the same to the bit".to_owned()
                } else {
                    format!("apart by {:.1} dB", db(rms(span(&d)) / rms(span(&ma))))
                }
            };
            let plain = apart(
                &render(n, &none, &q, 0, BLOCK),
                &render(n, &none, &q, 0, 16),
            );
            let every = apart(&render(n, &s, &q, 1, BLOCK), &render(n, &s, &q, 1, 16));
            eprintln!(
                "{} {q}: nothing assigned {plain}; every sample {every}",
                s.name
            );
        }
    }
}
