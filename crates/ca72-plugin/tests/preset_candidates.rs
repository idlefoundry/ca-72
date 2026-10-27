//! The candidate presets for 0.3 (`docs/preset-candidates/candidates.toml`, the factory file's
//! format): the sounds the 0.2.8 set leaves out, and the arpeggiator, the LFOs, the macros, the
//! timed and CONTINUOUS oscillators and each note's pressure and timbre in use. They are played
//! through the plug-in's own `process` as a host renders offline, its transport playing at
//! 120 BPM, so that everything a preset sets is heard, the engine-only measurements of
//! `preset_levels.rs` not knowing the 0.3 parameters.
//!
//! By hand, their levels and pitch: `cargo test --release -p ca72-plugin --test
//! preset_candidates -- --ignored --nocapture`; `CA72_ONLY` (a part of a name), `CA72_QUALITY`
//! (`lo`, `hi`; LO by default, a new instance's), `CA72_CANDIDATES_OUT` (a directory: each one's
//! render as `<preset>.wav`, 48 kHz, 24 bits, at the level it plays, and its expressive
//! presets' render again with a pressure swell as `<preset> (pressure).wav`), `CA72_SET`
//! (`key=value,...`: a preset's plain values changed, to try one), `CA72_FACTORY` (set): the
//! factory presets measured the same way instead, for comparison; `CA72_ONE_NOTE` (set): each
//! plays middle C alone for two seconds, for the pitch of those whose phrase has none.

#![allow(unsafe_code, clippy::unwrap_used)]

use ca72_analysis::loudness::loudness;
use ca72_plugin::Ca72;
use ca72_plugin::library::{Sound, parse_factory};
use ca72_plugin::params::Ca72Params;
use ca72_plugin::presets::sound_params;
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;
/// The factory presets' loudness (decisions.md R15), momentary maximum, LUFS.
const TARGET_LUFS: f64 = -18.0;

struct Init;
impl InitContext<Ca72> for Init {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn execute(&self, _: ()) {}
    fn set_latency_samples(&self, _: u32) {}
    fn set_current_voice_capacity(&self, _: u32) {}
}

fn candidates() -> Vec<Sound> {
    parse_factory(include_str!(
        "../../../docs/preset-candidates/candidates.toml"
    ))
    .unwrap()
}

/// The candidates, or with `CA72_FACTORY` the factory presets, measured alike for comparison;
/// `CA72_SET`'s values in place of their own.
fn measured() -> Vec<Sound> {
    let mut all = if std::env::var_os("CA72_FACTORY").is_some() {
        ca72_plugin::library::factory().to_vec()
    } else {
        candidates()
    };
    let set = std::env::var("CA72_SET").unwrap_or_default();
    for s in &mut all {
        for kv in set.split(',').filter(|x| !x.is_empty()) {
            let (k, v) = kv.split_once('=').unwrap();
            let v: f64 = v.parse().unwrap();
            match s.values.iter_mut().find(|(key, _)| key == k) {
                Some(x) => x.1 = v,
                None => s.values.push((k.to_owned(), v)),
            }
        }
    }
    all
}

fn value(s: &Sound, id: &str) -> Option<f64> {
    s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v)
}

fn tag(s: &Sound, t: &str) -> bool {
    s.tags.iter().any(|x| x == t)
}

/// What a candidate plays: (start, length) in seconds and the keys held, by its tags as
/// `preset_levels.rs` chooses, with the arpeggiator's a key or chord held (with HOLD, let go
/// after a second) and the percussive ones' hits.
fn phrase(s: &Sound) -> Vec<(f64, f64, Vec<u8>)> {
    if std::env::var_os("CA72_ONE_NOTE").is_some() {
        return vec![(0.25, 2.0, vec![60])];
    }
    let poly = value(s, "poly") == Some(1.0);
    if tag(s, "arp") {
        let keys = if tag(s, "bass") {
            vec![36]
        } else {
            vec![48, 52, 55, 59]
        };
        let held = if value(s, "arp_latch") == Some(1.0) {
            1.0
        } else {
            6.0
        };
        return vec![(0.25, held, keys)];
    }
    if tag(s, "drums") || tag(s, "percussive") {
        let keys: [u8; 4] = if tag(s, "tom") {
            [45, 50, 55, 60]
        } else if tag(s, "fx") {
            [72, 67, 64, 60]
        } else {
            [57; 4]
        };
        return keys
            .iter()
            .enumerate()
            .map(|(k, &key)| (0.25 + 0.6 * k as f64, 0.25, vec![key]))
            .collect();
    }
    if tag(s, "fx") {
        return vec![(0.25, 6.0, vec![48])];
    }
    if poly {
        return [48u8, 53, 55]
            .iter()
            .enumerate()
            .map(|(k, &r)| {
                let keys = [0u8, 4, 7, 12, 16, 19].iter().map(|d| r + d).collect();
                (0.25 + 3.0 * k as f64, 2.5, keys)
            })
            .collect();
    }
    let held = if tag(s, "slow") { 3.0 } else { 1.5 };
    let roots: [u8; 3] = if tag(s, "bass") {
        [36, 43, 40]
    } else {
        [60, 67, 64]
    };
    roots
        .iter()
        .enumerate()
        .map(|(k, &r)| (0.25 + (held + 0.5) * k as f64, held, vec![r]))
        .collect()
}

/// The preset set on `a` as a MIDI pad's preset is (each parameter its value, else its
/// default, landing at once; what its sources modulate), QUALITY `quality` (normalized).
fn set_preset(a: &Arc<Ca72Params>, c: &TestProcessContext<Ca72>, s: &Sound, quality: f32) {
    let map = a.param_map();
    let ptr = |id: &str| map.iter().find(|(i, ..)| i == id).map(|(_, p, _)| *p);
    let params = sound_params(a);
    for (k, _) in &s.values {
        assert!(
            ca72_plugin::lfo::Patch::is_value_id(k) || params.iter().any(|(id, _)| id == k),
            "{}: no parameter {k}",
            s.name
        );
        // (An assignment to no target would be dropped as it loads, without a word.)
        if let Some((source, target)) = k.split_once("_to_") {
            assert!(
                ca72_plugin::lfo::Source::of_id(source).is_some()
                    && ca72_plugin::lfo::Target::of_id(target).is_some(),
                "{}: no source or target in {k}",
                s.name
            );
        }
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
    assert!(c.automate(a.quality.as_ptr(), quality));
    a.lfo_patch.load_values(&s.values);
    a.lfo_patch.load_names(&s.macro_names);
}

/// The notes as events at their samples: (sample, key, on).
fn events(notes: &[(f64, f64, Vec<u8>)]) -> Vec<(usize, u8, bool)> {
    let at = |t: f64| (t * RATE).round() as usize;
    let mut e = Vec::new();
    for (t0, len, keys) in notes {
        for &k in keys {
            e.push((at(*t0), k, true));
            e.push((at(t0 + len), k, false));
        }
    }
    e.sort_by_key(|&(t, k, on)| (t, on, k));
    e
}

/// Preset `s` played through `process`, offline, at QUALITY `quality`, the transport playing
/// at 120 BPM from the first bar; with `pressure`, channel pressure swelling from none to full
/// and back over each note: left and right.
fn render(s: &Sound, quality: f32, pressure: bool) -> (Vec<f32>, Vec<f32>) {
    let notes = phrase(s);
    let end = notes.iter().map(|n| n.0 + n.1).fold(0.0, f64::max) + 2.0;
    let len = (end * RATE / BLOCK as f64).ceil() as usize * BLOCK;
    let mut p = Ca72::default();
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: BLOCK as u32,
        process_mode: ProcessMode::Offline,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    let a = p.parameters().clone();
    a.noise_seed
        .store(0xca72, std::sync::atomic::Ordering::Relaxed);
    let mut c = TestProcessContext::<Ca72>::new(p.params(), RATE as f32, ProcessMode::Offline);
    set_preset(&a, &c, s, quality);
    let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
    let (mut bl, mut br) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    let events = events(&notes);
    let mut next = events.iter().peekable();
    let mut start = 0;
    while start < len {
        c.set_transport(
            true,
            Some(120.0),
            Some(start as f64 / RATE * 2.0),
            Some((4, 4)),
        );
        if pressure {
            let t = start as f64 / RATE;
            let swell = notes
                .iter()
                .filter(|(t0, n, _)| (*t0..t0 + n).contains(&t))
                .map(|(t0, n, _)| (std::f64::consts::PI * (t - t0) / n).sin())
                .fold(0.0, f64::max);
            c.push_event(NoteEvent::MidiChannelPressure {
                timing: 0,
                channel: 0,
                pressure: swell as f32,
            });
        }
        while let Some(&&(t, note, on)) = next.peek() {
            if t >= start + BLOCK {
                break;
            }
            let timing = (t - start) as u32;
            c.push_event(if on {
                NoteEvent::NoteOn {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 1.0,
                }
            } else {
                NoteEvent::NoteOff {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 0.0,
                }
            });
            next.next();
        }
        let mut buffer = Buffer::default();
        // SAFETY: both slices are BLOCK long and outlive the buffer.
        unsafe {
            buffer.set_slices(BLOCK, |b| {
                b.clear();
                b.push(&mut bl);
                b.push(&mut br);
            });
        }
        p.process(&mut buffer, &mut aux, &mut c);
        drop(buffer);
        l[start..start + BLOCK].copy_from_slice(&bl);
        r[start..start + BLOCK].copy_from_slice(&br);
        start += BLOCK;
    }
    p.deactivate();
    (l, r)
}

fn peak(l: &[f32], r: &[f32]) -> f64 {
    l.iter()
        .chain(r)
        .fold(0.0f64, |m, x| m.max(f64::from(x.abs())))
}

fn rms(l: &[f32], r: &[f32]) -> f64 {
    let n = (l.len() + r.len()).max(1) as f64;
    (l.iter()
        .chain(r)
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
}

/// How far the first note's pitch is from its key, cents, where it has one: the strongest
/// normalized autocorrelation over the 0.4 s ending 0.25 s before its release, where the contours
/// have settled (a filter still sweeping its resonance across the harmonics moves the measured
/// period), its lag looked for within 60 cents of the key's octaves from four below to one
/// above.
fn cents_off(s: &Sound, l: &[f32], r: &[f32]) -> Option<f64> {
    let (t0, len, keys) = phrase(s).into_iter().next()?;
    if keys.len() != 1 || len < 0.8 || (tag(s, "arp") && value(s, "arp_on") == Some(1.0)) {
        return None;
    }
    let from = ((t0 + len - 0.65) * RATE) as usize;
    let n = (0.2 * RATE) as usize;
    let x: Vec<f64> = (from..from + 2 * n)
        .map(|i| f64::from(l[i] + r[i]))
        .collect();
    let corr = |lag: usize| {
        let (mut xy, mut xx, mut yy) = (0.0, 0.0, 0.0);
        for i in 0..n {
            xy += x[i] * x[i + lag];
            xx += x[i] * x[i];
            yy += x[i + lag] * x[i + lag];
        }
        xy / (xx * yy).sqrt().max(1e-30)
    };
    let key_hz = 440.0 * 2f64.powf((f64::from(keys[0]) - 69.0) / 12.0);
    let mut best = (f64::NEG_INFINITY, 0.0);
    for octave in -4..=1 {
        let hz = key_hz * 2f64.powi(octave);
        let lo = (RATE / (hz * 2f64.powf(60.0 / 1200.0))).floor() as usize;
        let hi = (RATE / (hz * 2f64.powf(-60.0 / 1200.0))).ceil() as usize;
        for lag in lo.max(2)..=hi.min(n) {
            let c = corr(lag);
            if c > best.0 {
                // The sub-sample peak, by the parabola through the lag and its neighbours.
                let (a, b) = (corr(lag - 1), corr(lag + 1));
                let shift = 0.5 * (a - b) / (a - 2.0 * c + b);
                let f = RATE / (lag as f64 + shift.clamp(-0.5, 0.5));
                best = (c, 1200.0 * (f / hz).log2());
            }
        }
    }
    (best.0 > 0.8).then_some(best.1)
}

/// The candidates' words name no trademark, as the factory file's may not (decisions.md R1;
/// `library.rs`'s `the_factory_file_names_no_trademark`), and no name is taken twice or by a
/// factory preset: each could move into the factory file as it is.
#[test]
fn the_candidates_could_join_the_factory_set() {
    let text = include_str!("../../../docs/preset-candidates/candidates.toml")
        .lines()
        .filter(|l| l.starts_with('#') || l.contains('"'))
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();
    for word in ["moog", "model d", "808", "909", "303", "roland", "minimoog"] {
        assert!(!text.contains(word), "candidates.toml says {word:?}");
    }
    let all = candidates();
    for (i, s) in all.iter().enumerate() {
        let same = |n: &str| n.eq_ignore_ascii_case(&s.name);
        assert!(!all[..i].iter().any(|t| same(&t.name)), "{} twice", s.name);
        assert!(
            !ca72_plugin::library::factory()
                .iter()
                .any(|f| same(&f.name)),
            "{} is a factory preset's name",
            s.name
        );
    }
}

/// Each candidate reads, sets only the plug-in's parameters and sources, and sounds (LO):
/// finite, not silent, not clipped.
#[test]
fn the_candidates_read_and_sound() {
    let all = candidates();
    assert!(all.len() >= 12, "{} candidates", all.len());
    for s in &all {
        let (l, r) = render(s, 0.0, false);
        assert!(l.iter().chain(&r).all(|x| x.is_finite()), "{}", s.name);
        let (p, m) = (peak(&l, &r), rms(&l, &r));
        assert!(m > 1e-3 && p < 1.0, "{}: peak {p}, RMS {m}", s.name);
    }
}

/// Their loudness, peak and pitch, and with `CA72_CANDIDATES_OUT` their sound for listening.
#[test]
#[ignore = "a measurement, run by hand"]
fn each_candidate_measured() {
    let only: String = std::env::var("CA72_ONLY").unwrap_or_default();
    let quality = match std::env::var("CA72_QUALITY").as_deref() {
        Ok("hi") => 0.5,
        _ => 0.0,
    };
    let out = std::env::var_os("CA72_CANDIDATES_OUT").map(PathBuf::from);
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    println!("preset               momentary max  integrated   peak   to -18  cents");
    for s in measured().iter().filter(|s| s.name.contains(&only)) {
        let (l, r) = render(s, quality, false);
        let m = loudness(&[&l, &r], RATE as u32);
        let max = m.momentary_max.unwrap_or(f64::NEG_INFINITY);
        println!(
            "{:20} {:>13.2} {:>11.2} {:>6.1} {:>+8.2} {:>6}",
            s.name,
            max,
            m.integrated.unwrap_or(f64::NEG_INFINITY),
            20.0 * peak(&l, &r).log10(),
            TARGET_LUFS - max,
            cents_off(s, &l, &r).map_or_else(|| "-".to_owned(), |c| format!("{c:+.0}")),
        );
        if let Some(d) = &out {
            write_wav(&d.join(format!("{}.wav", s.name)), &l, &r);
            if tag(s, "expressive") {
                let (l, r) = render(s, quality, true);
                let m = loudness(&[&l, &r], RATE as u32);
                println!(
                    "{:20} {:>13.2} (with pressure)",
                    "",
                    m.momentary_max.unwrap_or(f64::NEG_INFINITY)
                );
                write_wav(&d.join(format!("{} (pressure).wav", s.name)), &l, &r);
            }
        }
    }
}

fn write_wav(path: &Path, l: &[f32], r: &[f32]) {
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
            let v = (f64::from(x) * 8_388_607.0)
                .round()
                .clamp(-8_388_608.0, 8_388_607.0) as i32;
            b.extend_from_slice(&v.to_le_bytes()[..3]);
        }
    }
    std::fs::write(path, b).unwrap();
}
