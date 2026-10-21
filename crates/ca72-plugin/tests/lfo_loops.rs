//! The LFO LOOP presets (decisions.md R-LFO's addendum: the LFOs moving each other's knobs and
//! their own, in loops): `docs/lfo-loops/presets`, read as the library reads them and played
//! through the plug-in's own `process` as a host renders offline. Each reads, moves an LFO's own
//! knob, and sounds: finite, not silent, not clipped.
//!
//! By hand, their sound for listening: `CA72_LFO_LOOPS_OUT=<dir> cargo test --release -p
//! ca72-plugin --test lfo_loops -- --ignored --nocapture` writes each one's eight seconds at HI
//! (`<dir>/<preset>.wav`, 48 kHz, 24 bits) at a safe level, its peak at -14 dBFS (the gain
//! printed with the render's own peak and RMS).

#![allow(unsafe_code, clippy::unwrap_used)]

use ca72_plugin::Ca72;
use ca72_plugin::library::{Library, Sound};
use ca72_plugin::params::Ca72Params;
use ca72_plugin::presets::sound_params;
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;
/// The WAVs' peak, dBFS: well under full scale, whatever the patch does.
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

/// The LFO LOOP presets, as the library reads their folder.
fn presets() -> Vec<Sound> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lfo-loops/presets");
    let (entries, bad) = Library::at(&dir).entries();
    assert!(bad.is_empty(), "presets that do not read: {bad:?}");
    let mut out: Vec<Sound> = entries
        .into_iter()
        .map(|e| e.sound)
        .filter(|s| s.name.starts_with("LFO LOOP "))
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(out.len(), 4, "the four LFO LOOP presets");
    out
}

fn value(s: &Sound, id: &str) -> Option<f64> {
    s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v)
}

/// What it plays: a chord held where POLY is on, else four notes a second and a half apart.
fn phrase(s: &Sound, seconds: f64) -> Vec<(usize, u8, bool)> {
    let at = |t: f64| (t * RATE).round() as usize;
    let mut e = Vec::new();
    if value(s, "poly") == Some(1.0) {
        for k in [48u8, 55, 60, 64] {
            e.push((at(0.25), k, true));
            e.push((at(seconds - 0.6), k, false));
        }
    } else {
        for (n, k) in [45u8, 48, 43, 45].into_iter().enumerate() {
            let t = 0.25 + 1.5 * n as f64;
            e.push((at(t), k, true));
            e.push((at((t + 1.3).min(seconds - 0.6)), k, false));
        }
    }
    e.sort_by_key(|&(t, k, on)| (t, on, k));
    e
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
}

/// Preset `s` played for `seconds` through `process`, offline, at QUALITY `quality`: left and
/// right.
fn render(s: &Sound, seconds: f64, quality: f32) -> (Vec<f32>, Vec<f32>) {
    let mut p = Ca72::default();
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: BLOCK as u32,
        process_mode: ProcessMode::Offline,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    let a = p.parameters().clone();
    let mut c = TestProcessContext::<Ca72>::new(p.params(), RATE as f32, ProcessMode::Offline);
    set_preset(&a, &c, s, quality);
    let len = (seconds * RATE) as usize;
    let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
    let (mut bl, mut br) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    let events = phrase(s, seconds);
    let mut next = events.iter().peekable();
    let mut start = 0;
    while start < len {
        let block = BLOCK.min(len - start);
        c.set_transport(
            true,
            Some(120.0),
            Some(start as f64 / RATE * 2.0),
            Some((4, 4)),
        );
        while let Some(&&(t, note, on)) = next.peek() {
            if t >= start + block {
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

/// Each LFO LOOP preset reads, has an LFO's own knob moved by a source, and sounds (LO, two
/// seconds): finite, not silent, not clipped.
#[test]
fn the_lfo_loop_presets_read_loop_and_sound() {
    for s in presets() {
        assert!(
            s.values
                .iter()
                .any(|(k, v)| k.contains("_to_lfo") && *v != 0.0),
            "{}: no LFO's knob moved",
            s.name
        );
        let (l, r) = render(&s, 2.0, 0.0);
        assert!(l.iter().chain(&r).all(|x| x.is_finite()), "{}", s.name);
        let (p, m) = (peak(&l, &r), rms(&l, &r));
        assert!(m > 1e-3 && p < 1.0, "{}: peak {p}, RMS {m}", s.name);
    }
}

/// Their sound for listening (`CA72_LFO_LOOPS_OUT`), eight seconds each at HI, at a safe level.
#[test]
#[ignore = "writes WAVs for listening"]
fn lfo_loop_wavs() {
    let Some(out) = std::env::var_os("CA72_LFO_LOOPS_OUT").map(PathBuf::from) else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    for s in presets() {
        let (l, r) = render(&s, 8.0, 0.5);
        let (p, m) = (peak(&l, &r), rms(&l, &r));
        let gain = 10f64.powf(PEAK_DBFS / 20.0) / p.max(1e-9);
        println!(
            "{}: peak {:.1} dBFS, RMS {:.1} dBFS; written at {:+.1} dB",
            s.name,
            20.0 * p.log10(),
            20.0 * m.log10(),
            20.0 * gain.log10()
        );
        write_wav(&out.join(format!("{}.wav", s.name)), &l, &r, gain);
    }
}

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
