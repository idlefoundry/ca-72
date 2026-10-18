//! Each factory preset rendered with POLY's ten voices, for comparing a change with the code
//! before it (decisions.md R11): the engine as a host calls it, 256-frame blocks at 48 kHz,
//! the chords of `preset_cost.rs` (ten notes every half second, held seven eighths of it),
//! nothing at EXTERNAL INPUT.
//!
//! By hand: `CA72_RENDER_OUT=<dir>` writes each preset's render to `<dir>/<preset>.f32`
//! (stereo, interleaved, little-endian); `CA72_RENDER_REF=<dir>` compares each with the one
//! there and prints, per preset, whether it is the same to the bit, else the difference's
//! RMS and peak against the reference's, the level's change, and the largest change of a
//! third-octave band's level (the bands within 60 dB of the strongest). `CA72_SECONDS` (4),
//! `CA72_ONLY` (a part of a preset's name), `CA72_VOICES` (10), `CA72_PRESETS_FILE` (a file of
//! presets in the factory's format, read instead of the factory's: an earlier version's, to
//! hear its sounds, as its sessions hold them, through this engine), `CA72_FROM_0_1` (set:
//! each preset's knobs translated as a session 0.1 saved is when it opens, `compat`, those
//! it does not name at their defaults first, as 0.1 had them), `CA72_POTATO` (set: QUALITY at
//! LO, the light voices), `CA72_ULTRA` (set: QUALITY at ULTRA); else QUALITY at HI. A switch
//! is set by `1` or nothing; any other value, or a number that does not read as one, stops the
//! run. Once every render asked for is written, `CA72_RENDER_OUT` writes the run's settings
//! beside them (`settings.txt`: QUALITY, the voices, the seconds, the presets' file by a digest
//! of its contents, then the renders' files), and removes an earlier run's at the start; a
//! reference's settings, where it has them, must be the run's, and each render compared must
//! be among those it lists, else the comparison stops (decisions.md R-RENDER-SETTINGS).
//!
//! `cargo test --release -p ca72-plugin --test preset_render -- --ignored --nocapture`

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::fft::{hann, power};
use ca72_plugin::compat;
use ca72_plugin::engine::Engine;
use ca72_plugin::library::{Sound, factory, parse_factory};
use common::{Chords, controls_of, env, play_block, switch};
use std::path::PathBuf;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

/// The file beside a run's renders that names its settings ([`settings`]).
const SETTINGS: &str = "settings.txt";

/// A preset's render: left and right.
/// The presets rendered: the factory's, or `CA72_PRESETS_FILE`'s; with `CA72_FROM_0_1` set,
/// translated as 0.1's sessions are.
fn presets() -> Vec<Sound> {
    let mut sounds = match std::env::var_os("CA72_PRESETS_FILE") {
        Some(f) => parse_factory(&std::fs::read_to_string(f).unwrap()).unwrap(),
        None => factory().to_vec(),
    };
    if switch("CA72_FROM_0_1") {
        for s in &mut sounds {
            compat::translate_preset(&mut s.values);
        }
    }
    sounds
}

/// QUALITY as the run asks for it: `CA72_POTATO` LO, `CA72_ULTRA` ULTRA, else HI.
fn quality() -> &'static str {
    match (switch("CA72_POTATO"), switch("CA72_ULTRA")) {
        (true, true) => panic!("set CA72_POTATO or CA72_ULTRA, not both"),
        (true, false) => "LO",
        (false, true) => "ULTRA",
        (false, false) => "HI",
    }
}

/// What a render depends on besides the code and the preset, as one line: two renders made
/// with different settings differ however repeatable the engine is (the voices' count changes
/// the first sample: decisions.md R-RENDER-SETTINGS). A presets file is named by its contents'
/// digest (FNV-1a, 64 bits), not its path, which may hold another version's later.
fn settings(voices: usize, seconds: f64) -> String {
    let presets = match std::env::var_os("CA72_PRESETS_FILE") {
        Some(f) => {
            let digest = std::fs::read(f)
                .unwrap()
                .iter()
                .fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
                    (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
                });
            format!("file-{digest:016x}")
        }
        None => "factory".into(),
    };
    format!(
        "quality {} voices {voices} seconds {seconds} presets {presets} from_0_1 {}",
        quality(),
        switch("CA72_FROM_0_1")
    )
}

fn render(s: &Sound, voices: usize, seconds: f64) -> (Vec<f32>, Vec<f32>) {
    let mut c = controls_of(s);
    c.poly = true;
    c.voices = voices;
    let q = quality();
    c.potato = q == "LO";
    c.ultra = q == "ULTRA";
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(RATE, 1);
    common::workers(&mut e);
    let mut chords = Chords::new(voices, RATE);
    let blocks = (seconds * RATE / BLOCK as f64) as usize;
    let (mut l, mut r) = (vec![0.0f32; blocks * BLOCK], vec![0.0f32; blocks * BLOCK]);
    let mut events = Vec::with_capacity(64);
    for (i, (bl, br)) in l.chunks_mut(BLOCK).zip(r.chunks_mut(BLOCK)).enumerate() {
        play_block(&mut e, &mut chords, i * BLOCK, bl, br, &mut events);
    }
    (l, r)
}

fn file_name(name: &str) -> String {
    format!("{}.f32", name.replace(' ', "_"))
}

fn write(path: &PathBuf, l: &[f32], r: &[f32]) {
    let mut bytes = Vec::with_capacity(8 * l.len());
    for (a, b) in l.iter().zip(r) {
        bytes.extend_from_slice(&a.to_le_bytes());
        bytes.extend_from_slice(&b.to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}

fn read(path: &PathBuf) -> (Vec<f32>, Vec<f32>) {
    let bytes = std::fs::read(path).unwrap();
    let x: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    (
        x.iter().step_by(2).copied().collect(),
        x.iter().skip(1).step_by(2).copied().collect(),
    )
}

fn rms(x: impl Iterator<Item = f64> + Clone) -> f64 {
    let n = x.clone().count().max(1) as f64;
    (x.map(|v| v * v).sum::<f64>() / n).sqrt()
}

fn db(x: f64) -> f64 {
    20.0 * x.max(1e-300).log10()
}

/// Third-octave bands' power (20 Hz to 20 kHz) of the mono mix: a Welch average of Hann
/// frames of 8192.
fn bands(mono: &[f64]) -> Vec<(f64, f64)> {
    const N: usize = 8192;
    let w = hann(N);
    let mut acc = vec![0.0; N / 2 + 1];
    let mut start = 0;
    while start + N <= mono.len() {
        for (a, p) in acc.iter_mut().zip(power(&mono[start..start + N], &w)) {
            *a += p;
        }
        start += N / 2;
    }
    let bin = RATE / N as f64;
    (0..31)
        .map(|k| {
            let c = 1000.0 * 2f64.powf((k as f64 - 17.0) / 3.0);
            let (lo, hi) = (c / 2f64.powf(1.0 / 6.0), c * 2f64.powf(1.0 / 6.0));
            let e: f64 = acc
                .iter()
                .enumerate()
                .filter(|(k, _)| (lo..hi).contains(&(*k as f64 * bin)))
                .map(|(_, v)| v)
                .sum();
            (c, e)
        })
        .collect()
}

#[test]
#[ignore = "a comparison, run by hand"]
fn each_preset_against_a_reference_render() {
    let voices: usize = env("CA72_VOICES", 10);
    let seconds: f64 = env("CA72_SECONDS", 4.0);
    let only = std::env::var("CA72_ONLY").ok();
    let out = std::env::var_os("CA72_RENDER_OUT").map(PathBuf::from);
    let reference = std::env::var_os("CA72_RENDER_REF").map(PathBuf::from);
    assert!(
        out.is_some() || reference.is_some(),
        "set CA72_RENDER_OUT, CA72_RENDER_REF or both"
    );
    let made = settings(voices, seconds);
    // The reference's renders made with this run's settings (None: a reference from before
    // settings were kept, not checked). Read before an output there is begun.
    let listed: Option<Vec<String>> = reference.as_ref().and_then(|dir| {
        let Ok(theirs) = std::fs::read_to_string(dir.join(SETTINGS)) else {
            eprintln!("(no {SETTINGS} beside the reference: its settings not checked)");
            return None;
        };
        let mut lines = theirs.lines();
        assert_eq!(
            lines.next().unwrap_or_default(),
            made,
            "the reference in {} was rendered with other settings than this run",
            dir.display()
        );
        Some(lines.map(str::to_owned).collect())
    });
    // (An earlier run's settings removed until this one's renders are all written: an output
    // left half made, or holding another run's renders, lists none of them.)
    if let Some(dir) = &out {
        std::fs::create_dir_all(dir).unwrap();
        match std::fs::remove_file(dir.join(SETTINGS)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => panic!("{SETTINGS}: {e}"),
            _ => {}
        }
    }
    let mut written = Vec::new();
    eprintln!(
        "{:<18} {:>10} {:>10} {:>9} {:>11}",
        "preset", "diff RMS", "diff peak", "level", "worst band"
    );
    let mut all_same = true;
    for s in &presets() {
        if only.as_ref().is_some_and(|o| !s.name.contains(o.as_str())) {
            continue;
        }
        let (l, r) = render(s, voices, seconds);
        if let Some(dir) = &out {
            write(&dir.join(file_name(&s.name)), &l, &r);
            written.push(file_name(&s.name));
        }
        let Some(dir) = &reference else {
            continue;
        };
        if let Some(listed) = &listed {
            assert!(
                listed.contains(&file_name(&s.name)),
                "{}: not among the renders {SETTINGS} lists in {}",
                s.name,
                dir.display()
            );
        }
        let (rl, rr) = read(&dir.join(file_name(&s.name)));
        assert_eq!(rl.len(), l.len(), "{}: another length", s.name);
        let same = l
            .iter()
            .chain(&r)
            .zip(rl.iter().chain(&rr))
            .all(|(a, b)| a.to_bits() == b.to_bits());
        if same {
            eprintln!("{:<18} the same to the bit", s.name);
            continue;
        }
        all_same = false;
        let new: Vec<f64> = l.iter().chain(&r).map(|&x| f64::from(x)).collect();
        let old: Vec<f64> = rl.iter().chain(&rr).map(|&x| f64::from(x)).collect();
        let ref_rms = rms(old.iter().copied());
        let ref_peak = old.iter().fold(0.0f64, |a, x| a.max(x.abs()));
        let diff = new.iter().zip(&old).map(|(a, b)| a - b);
        let diff_peak = diff.clone().fold(0.0f64, |a, x| a.max(x.abs()));
        let mono = |a: &[f32], b: &[f32]| -> Vec<f64> {
            a.iter()
                .zip(b)
                .map(|(x, y)| 0.5 * f64::from(x + y))
                .collect()
        };
        let (bn, bo) = (bands(&mono(&l, &r)), bands(&mono(&rl, &rr)));
        let strongest = bo.iter().fold(0.0f64, |a, b| a.max(b.1));
        let worst = bn
            .iter()
            .zip(&bo)
            .filter(|(_, o)| o.1 > strongest * 1e-6)
            .map(|(n, o)| (o.0, 10.0 * (n.1 / o.1).log10()))
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .unwrap_or((0.0, 0.0));
        eprintln!(
            "{:<18} {:>7.1} dB {:>7.1} dB {:>+6.2} dB {:>+5.2} dB @ {:.0} Hz",
            s.name,
            db(rms(diff) / ref_rms),
            db(diff_peak / ref_peak),
            db(rms(new.iter().copied()) / ref_rms),
            worst.1,
            worst.0,
        );
    }
    if let Some(dir) = &out {
        let list: String = written.iter().map(|f| format!("{f}\n")).collect();
        std::fs::write(dir.join(SETTINGS), format!("{made}\n{list}")).unwrap();
    }
    if reference.is_some() && all_same {
        eprintln!("every preset the same to the bit");
    }
}
