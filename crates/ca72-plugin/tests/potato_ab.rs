//! Each factory preset played in the circuit's model (QUALITY at HI) and in Potato mode (at
//! LO; decisions.md R-POTATO), for listening to them side by side and for a first look at how
//! far apart they are: the
//! preset as its own POLY and VOICES have it, the one instrument a bass line of eighths at
//! 120 BPM (a legato pair among them, the last note held), POLY chords of its voices. Prints,
//! per preset, each mode's level (RMS, dB) and brightness (the spectrum's centroid, Hz).
//!
//! By hand: `CA72_AB_OUT=<dir> cargo test --release -p ca72-plugin --test potato_ab --
//! --ignored --nocapture` writes `<dir>/<preset> - full.wav` and `<dir>/<preset> -
//! potato.wav` (stereo, 32-bit float, 48 kHz). `CA72_ONLY`, a part of a preset's name;
//! `CA72_BANDS`, set: each octave band's difference too.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::fft::{hann, power};
use ca72_plugin::engine::{Engine, Event};
use ca72_plugin::library::{Sound, factory};
use common::controls_of;

/// The sample rate: 48 kHz, or `CA72_RATE`.
fn rate() -> f64 {
    std::env::var("CA72_RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(48_000.0)
}
const BLOCK: usize = 256;

/// The preset's phrase: each note's start and end (s) and key.
fn phrase(poly: bool, voices: usize) -> Vec<(f64, f64, u8)> {
    if poly {
        const SHAPE: [u8; 10] = [0, 7, 12, 16, 19, 24, 28, 31, 34, 36];
        let mut notes = Vec::new();
        for (i, root) in [45u8, 41, 43, 48].iter().enumerate() {
            let t = i as f64 * 1.5;
            for d in SHAPE.iter().take(voices) {
                notes.push((t, t + 1.3, root + d));
            }
        }
        notes
    } else {
        let line = [
            36u8, 36, 48, 36, 43, 36, 46, 48, 36, 36, 48, 36, 43, 41, 39, 38,
        ];
        let mut notes: Vec<(f64, f64, u8)> = line
            .iter()
            .enumerate()
            .map(|(i, &k)| {
                let t = i as f64 * 0.25;
                // (The sixth and seventh tied: legato.)
                let held = if i == 5 { 0.3 } else { 0.18 };
                (t, t + held, k)
            })
            .collect();
        notes.push((4.0, 5.5, 36));
        notes
    }
}

/// The preset played: left and right, 7 s.
fn render(s: &Sound, potato: bool) -> (Vec<f32>, Vec<f32>) {
    let value = |id: &str| s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v);
    let poly = value("poly").is_some_and(|v| v >= 0.5);
    let voices = value("voices").map_or(4, |v| v as usize);
    let mut c = controls_of(s);
    if std::env::var_os("CA72_SILENT").is_some() {
        for o in &mut c.panel.osc {
            o.on = false;
        }
        c.panel.noise_on = false;
        c.panel.ext_on = false;
        c.feedback = 0.0;
    }
    c.poly = poly;
    c.voices = voices;
    c.potato = potato;
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(rate(), 1);
    let mut events: Vec<(usize, Event)> = Vec::new();
    for (on, off, key) in phrase(poly, voices) {
        events.push(((on * rate()) as usize, Event::Note { key, on: true }));
        events.push(((off * rate()) as usize, Event::Note { key, on: false }));
    }
    events.sort_by_key(|e| e.0);
    let n = (7.0 * rate()) as usize / BLOCK * BLOCK;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut next = events.iter().peekable();
    for start in (0..n).step_by(BLOCK) {
        let mut k = start;
        while k < start + BLOCK {
            while let Some((_, ev)) = next.next_if(|(at, _)| *at <= k) {
                e.event(*ev);
            }
            let end = next
                .peek()
                .map_or(start + BLOCK, |(at, _)| (*at).min(start + BLOCK));
            e.render(&[], &mut l[k..end], &mut r[k..end]);
            k = end;
        }
        e.end_block(BLOCK);
    }
    (l, r)
}

/// Level (RMS, dB) and the spectrum's centroid (Hz) of the sum of the channels.
fn measure(l: &[f32], r: &[f32]) -> (f64, f64) {
    let mono: Vec<f64> = l
        .iter()
        .zip(r)
        .map(|(a, b)| 0.5 * f64::from(a + b))
        .collect();
    let rms = (mono.iter().map(|x| x * x).sum::<f64>() / mono.len() as f64).sqrt();
    let size = 4096;
    let w = hann(size);
    let mut spectrum = vec![0.0; size / 2 + 1];
    for frame in mono.chunks_exact(size) {
        for (s, p) in spectrum.iter_mut().zip(power(frame, &w)) {
            *s += p;
        }
    }
    let total: f64 = spectrum.iter().sum();
    let centroid = spectrum
        .iter()
        .enumerate()
        .map(|(k, p)| k as f64 * rate() / size as f64 * p)
        .sum::<f64>()
        / total.max(1e-300);
    (20.0 * rms.max(1e-12).log10(), centroid)
}

/// Each octave band's level (dB) from 31 Hz to 16 kHz, of the sum of the channels.
fn octaves(l: &[f32], r: &[f32]) -> Vec<(f64, f64)> {
    let mono: Vec<f64> = l
        .iter()
        .zip(r)
        .map(|(a, b)| 0.5 * f64::from(a + b))
        .collect();
    let size = 8192;
    let w = hann(size);
    let mut spectrum = vec![0.0; size / 2 + 1];
    for frame in mono.chunks_exact(size) {
        for (s, p) in spectrum.iter_mut().zip(power(frame, &w)) {
            *s += p;
        }
    }
    (0..10)
        .map(|i| {
            let hz = 31.25 * f64::from(1u32 << i);
            let (lo, hi) = (hz / 2f64.sqrt(), hz * 2f64.sqrt());
            let p: f64 = spectrum
                .iter()
                .enumerate()
                .filter(|(k, _)| {
                    let f = *k as f64 * rate() / size as f64;
                    f >= lo && f < hi
                })
                .map(|(_, p)| p)
                .sum();
            (hz, 10.0 * p.max(1e-30).log10())
        })
        .collect()
}

fn write_wav(path: &std::path::Path, l: &[f32], r: &[f32]) {
    let data = (l.len() * 8) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(rate() as u32).to_le_bytes());
    b.extend_from_slice(&(rate() as u32 * 8).to_le_bytes());
    b.extend_from_slice(&8u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for (x, y) in l.iter().zip(r) {
        b.extend_from_slice(&x.to_le_bytes());
        b.extend_from_slice(&y.to_le_bytes());
    }
    std::fs::write(path, b).unwrap();
}

#[test]
#[ignore = "renders for listening, run by hand"]
fn each_preset_in_both_modes() {
    let out = std::env::var_os("CA72_AB_OUT").map(std::path::PathBuf::from);
    let only = std::env::var("CA72_ONLY").ok();
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    eprintln!(
        "{:<18} {:>9} {:>9} {:>7} | {:>9} {:>9} {:>7}",
        "preset", "full dB", "potato", "diff", "full Hz", "potato", "ratio"
    );
    for s in factory() {
        if only.as_ref().is_some_and(|o| !s.name.contains(o.as_str())) {
            continue;
        }
        let (fl, fr) = render(s, false);
        let (pl, pr) = render(s, true);
        let (fdb, fc) = measure(&fl, &fr);
        let (pdb, pc) = measure(&pl, &pr);
        eprintln!(
            "{:<18} {fdb:>9.1} {pdb:>9.1} {:>+7.1} | {fc:>9.0} {pc:>9.0} {:>7.2}",
            s.name,
            pdb - fdb,
            pc / fc
        );
        if std::env::var_os("CA72_BANDS").is_some() {
            let (a, b) = (octaves(&fl, &fr), octaves(&pl, &pr));
            let row: Vec<String> = a
                .iter()
                .zip(&b)
                .map(|((hz, x), (_, y))| format!("{hz:.0}:{:+.1}", y - x))
                .collect();
            eprintln!("    potato against full, dB by octave: {}", row.join(" "));
        }
        if let Some(d) = &out {
            write_wav(&d.join(format!("{} - full.wav", s.name)), &fl, &fr);
            write_wav(&d.join(format!("{} - potato.wav", s.name)), &pl, &pr);
        }
    }
}
