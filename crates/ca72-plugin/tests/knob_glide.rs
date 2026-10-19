//! Knob changes through the plug-in's own `process`, as a host calls it (issue #107; decisions.md
//! R-KNOB-GLIDE). The file uses only what the code before the glides has too, so the same file
//! measures before and after.
//!
//! - `cutoff_sweep_through_process`: CUTOFF swept from 25 to 75 % over two seconds at strong
//!   EMPHASIS (Brass Choir, one voice at HI, oscillator 1 alone, no contour), as a host's
//!   automation once a block of 32 to 1024 frames (in real time and offline), as a mouse's drag
//!   (60 values a second, each taken at the next 256-frame block), and as a trackpad turns it
//!   (0.2.8's editor: whole 2 % notches; this one's: 60 values a second in proportion). Prints
//!   how strongly the high band's loudness pulses at the rate the values come, against the
//!   voices told the sweep every sample (the engine itself, `tests/cutoff_steps.rs`'s figure).
//!   `CA72_GLIDE_OUT=<dir>` writes WAVs there (one gain for all, the continuous sweep's peak at
//!   -14 dBFS), named with `CA72_GLIDE_TAG` (`before`, `after`); `CA72_EMPHASIS`, 0 to 10, 9.
//! - `presets_through_process`: every factory preset played two seconds, then the next one
//!   chosen in the presets' drawer and played a second more, offline (every voice waited for),
//!   256-frame blocks: `CA72_RENDER_OUT=<dir>` writes them, `CA72_RENDER_REF=<dir>` compares
//!   with them, to the bit.
//!
//! - `glide_cost`: ten voices of Felt Bass at LO and HI, POLY, for four seconds offline, CUTOFF
//!   at rest and automated every 256-frame block (gliding the whole time here): the time
//!   `process` takes a second of audio, the best of three.
//!
//! By hand: `cargo test --release -p ca72-plugin --test knob_glide -- --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::fft::{hann, power};
use ca72_plugin::Ca72;
use ca72_plugin::engine::{Engine, Event};
use ca72_plugin::library::{Library, Sound, factory};
use ca72_plugin::params::{Ca72Params, QualityMode};
use ca72_plugin::presets::{Browser, sound_params};
use common::env;
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const RATE: f64 = 48_000.0;
const SEED: u64 = 0xca72;

struct Init;
impl InitContext<Ca72> for Init {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn execute(&self, _: ()) {}
    fn set_latency_samples(&self, _: u32) {}
    fn set_current_voice_capacity(&self, _: u32) {}
}

/// The editor's way to the host, applied as a host's automation (the presets' drawer).
struct Gui(Mutex<TestProcessContext<Ca72>>);

#[allow(unsafe_code)]
impl GuiContext for Gui {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn request_resize(&self) -> bool {
        true
    }
    unsafe fn raw_begin_set_parameter(&self, _: ParamPtr) {}
    unsafe fn raw_set_parameter_normalized(&self, ptr: ParamPtr, value: f32) {
        assert!(self.0.lock().unwrap().automate(ptr, value));
    }
    unsafe fn raw_end_set_parameter(&self, _: ParamPtr) {}
    fn get_state(&self) -> PluginState {
        unreachable!()
    }
    fn set_state(&self, _: PluginState) {
        unreachable!()
    }
}

/// A plug-in and its host, the sound `s` set (then anything `then` sets), initialised at 48 kHz
/// in blocks of `block` frames, `mode`.
fn hosted(
    s: &Sound,
    block: usize,
    mode: ProcessMode,
    then: impl Fn(&Ca72Params, &TestProcessContext<Ca72>),
) -> (Ca72, TestProcessContext<Ca72>) {
    let mut p = Ca72::default();
    let params = p.parameters().clone();
    params
        .noise_seed
        .store(SEED, std::sync::atomic::Ordering::Relaxed);
    let as_params: Arc<dyn Params> = params.clone();
    let c = TestProcessContext::<Ca72>::new(as_params, RATE as f32, mode);
    set_sound(&params, &c, s);
    then(&params, &c);
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: block as u32,
        process_mode: mode,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    p.reset();
    (p, c)
}

/// The sound's values set as the host sets parameters.
fn set_sound(params: &Ca72Params, c: &TestProcessContext<Ca72>, s: &Sound) {
    let map = params.param_map();
    let sound = sound_params(params);
    for (id, v) in &s.values {
        let (Some((_, q)), Some((_, ptr, _))) = (
            sound.iter().find(|(i, _)| i == id),
            map.iter().find(|(i, ..)| i == id),
        ) else {
            continue;
        };
        assert!(c.automate(*ptr, q.normalized_of(*v)));
    }
}

/// One block of `len` frames: left and right.
#[allow(unsafe_code)]
fn block(p: &mut Ca72, c: &mut TestProcessContext<Ca72>, len: usize) -> (Vec<f32>, Vec<f32>) {
    let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
    let mut buffer = Buffer::default();
    // SAFETY: both slices are `len` long and live until `process` has returned.
    unsafe {
        buffer.set_slices(len, |s| {
            s.clear();
            s.push(&mut l);
            s.push(&mut r);
        });
    }
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    p.process(&mut buffer, &mut aux, c);
    drop(buffer);
    (l, r)
}

fn note(timing: u32, pitch: u8, on: bool) -> NoteEvent<()> {
    if on {
        NoteEvent::NoteOn {
            timing,
            voice_id: None,
            channel: 0,
            note: pitch,
            velocity: 1.0,
        }
    } else {
        NoteEvent::NoteOff {
            timing,
            voice_id: None,
            channel: 0,
            note: pitch,
            velocity: 0.0,
        }
    }
}

const SECONDS: f64 = 3.0;

/// CUTOFF's travel (0..1) at sample `n`: up from 25 to 75 % over the first two seconds, held.
fn cutoff_at(n: usize) -> f64 {
    let t = (n as f64 / RATE / 2.0).min(1.0);
    0.25 + 0.5 * t
}

/// How CUTOFF's values reach the plug-in.
#[derive(Clone, Copy)]
enum Values {
    /// The host's automation at each block's start.
    Blocks,
    /// A new value every `every` samples (a mouse's or a trackpad's events), taken at the
    /// next block's start; with `notch`, the travel from the start in whole notches of that
    /// share (0.2.8's trackpad: 40 points to 2 %).
    Events { every: usize, notch: Option<f64> },
}

struct Case {
    name: &'static str,
    block: usize,
    mode: ProcessMode,
    values: Values,
}

impl Case {
    /// The rate new values arrive at, Hz.
    fn rate(&self) -> f64 {
        match self.values {
            Values::Blocks => RATE / self.block as f64,
            // (25 notches of 2 % over the two seconds' 50 %.)
            Values::Events { notch: Some(n), .. } => 0.5 / n / 2.0,
            Values::Events { every, .. } => RATE / every as f64,
        }
    }

    fn value_at(&self, k: usize) -> f64 {
        match self.values {
            Values::Blocks => cutoff_at(k),
            Values::Events { every, notch } => {
                let v = cutoff_at(k / every * every);
                match notch {
                    Some(n) => cutoff_at(0) + ((v - cutoff_at(0)) / n + 1e-9).floor() * n,
                    None => v,
                }
            }
        }
    }
}

/// The measurement's sound: Brass Choir, one voice at HI, oscillator 1 alone, no contour,
/// EMPHASIS as `CA72_EMPHASIS` (9), CUTOFF at the sweep's start.
fn sweep_sound(params: &Ca72Params, c: &TestProcessContext<Ca72>) {
    let n = |q: &FloatParam, v: f32| c.automate(q.as_ptr(), q.preview_normalized(v));
    assert!(c.automate(params.poly.as_ptr(), 0.0));
    assert!(c.automate(
        params.quality.as_ptr(),
        params.quality.preview_normalized(QualityMode::Hi)
    ));
    assert!(c.automate(params.doubled.as_ptr(), 0.0));
    assert!(c.automate(params.osc2_on.as_ptr(), 0.0));
    assert!(c.automate(params.osc3_on.as_ptr(), 0.0));
    assert!(n(&params.spread, 0.0));
    assert!(n(&params.contour_amount, 0.0));
    assert!(n(&params.emphasis, env("CA72_EMPHASIS", 9.0)));
    assert!(n(&params.cutoff, (cutoff_at(0) * 10.0 - 5.0) as f32));
}

/// The sweep through the plug-in as `case` has its values reach it.
fn sweep(case: &Case) -> (Vec<f32>, Vec<f32>) {
    let s = common::preset("Brass Choir");
    let (mut p, mut c) = hosted(&s, case.block, case.mode, sweep_sound);
    let params = p.parameters().clone();
    let cutoff = params.cutoff.as_ptr();
    c.push_event(note(0, 48, true));
    let n = (SECONDS * RATE) as usize;
    let (mut l, mut r) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let mut last = cutoff_at(0);
    let mut k = 0;
    while k < n {
        let v = case.value_at(k);
        if v != last {
            let normalized = params.cutoff.preview_normalized((v * 10.0 - 5.0) as f32);
            assert!(c.automate(cutoff, normalized));
            last = v;
        }
        let (bl, br) = block(&mut p, &mut c, case.block.min(n - k));
        l.extend(bl);
        r.extend(br);
        k += case.block;
    }
    (l, r)
}

/// The sweep with the voices told CUTOFF every sample: the engine itself, its controls the
/// plug-in's for the sound.
fn continuous() -> (Vec<f32>, Vec<f32>) {
    let s = common::preset("Brass Choir");
    let (p, _c) = hosted(&s, 256, ProcessMode::Offline, sweep_sound);
    let mut ctl = p.parameters().controls();
    let mut e = Engine::new();
    e.set(&ctl);
    e.prepare(RATE, SEED);
    e.event(Event::Note { key: 48, on: true });
    let n = (SECONDS * RATE) as usize;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    for k in 0..n {
        let v = cutoff_at(k);
        if ctl.panel.cutoff != v {
            ctl.panel.cutoff = v;
            e.set(&ctl);
        }
        e.render(&[], &mut l[k..=k], &mut r[k..=k]);
    }
    (l, r)
}

/// The high band's envelope (the third difference's magnitude, averaged over 8 samples) over
/// `x`, and its power spectrum: the bins' power and their width, Hz (`tests/cutoff_steps.rs`).
fn envelope_spectrum(x: &[f64]) -> (Vec<f64>, f64) {
    const BIN: usize = 8;
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

/// The envelope's power at `hz` and its first three harmonics (the strongest bin within two of
/// each), summed.
fn at_rate(spectrum: &(Vec<f64>, f64), hz: f64) -> f64 {
    let (p, width) = spectrum;
    (1..=4)
        .map(|h| {
            let k = (h as f64 * hz / width).round() as usize;
            (k.saturating_sub(2)..=(k + 2).min(p.len() - 1))
                .map(|i| p[i])
                .fold(0.0, f64::max)
        })
        .sum()
}

fn mono((l, r): &(Vec<f32>, Vec<f32>)) -> Vec<f64> {
    l.iter()
        .zip(r)
        .map(|(a, b)| 0.5 * f64::from(a + b))
        .collect()
}

fn peak((l, r): &(Vec<f32>, Vec<f32>)) -> f32 {
    l.iter().chain(r).fold(0.0f32, |m, v| m.max(v.abs()))
}

#[test]
#[ignore = "a measurement, run by hand"]
fn cutoff_sweep_through_process() {
    let out = std::env::var_os("CA72_GLIDE_OUT").map(PathBuf::from);
    let tag = std::env::var("CA72_GLIDE_TAG").unwrap_or_else(|_| "this".into());
    let reference = continuous();
    let refm = mono(&reference);
    let sweep_len = (2.0 * RATE) as usize;
    let ref_env = envelope_spectrum(&refm[..sweep_len]);
    // (Level-safe: every render at one gain, the continuous sweep's peak at -14 dBFS.)
    let gain = 10f32.powf(-14.0 / 20.0) / peak(&reference).max(1e-6);
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
        write_wav(&d.join("cutoff-continuous.wav"), &reference, gain);
    }
    let rt = ProcessMode::Realtime;
    let mouse = Values::Events {
        every: 800,
        notch: None,
    };
    let cases = [
        Case {
            name: "drag-60hz-at-256",
            block: 256,
            mode: rt,
            values: mouse,
        },
        Case {
            name: "trackpad-notches-at-256",
            block: 256,
            mode: rt,
            values: Values::Events {
                every: 800,
                notch: Some(0.02),
            },
        },
        Case {
            name: "trackpad-60hz-at-256",
            block: 256,
            mode: rt,
            values: mouse,
        },
        Case {
            name: "automation-32",
            block: 32,
            mode: rt,
            values: Values::Blocks,
        },
        Case {
            name: "automation-256",
            block: 256,
            mode: rt,
            values: Values::Blocks,
        },
        Case {
            name: "automation-512",
            block: 512,
            mode: rt,
            values: Values::Blocks,
        },
        Case {
            name: "automation-1024",
            block: 1024,
            mode: rt,
            values: Values::Blocks,
        },
        Case {
            name: "automation-1024-offline",
            block: 1024,
            mode: ProcessMode::Offline,
            values: Values::Blocks,
        },
        Case {
            name: "automation-2048-offline",
            block: 2048,
            mode: ProcessMode::Offline,
            values: Values::Blocks,
        },
    ];
    eprintln!(
        "EMPHASIS {}: the high band's loudness at the values' rate and its harmonics, against the continuous sweep's",
        env("CA72_EMPHASIS", 9.0)
    );
    let mut ends = Vec::new();
    for case in &cases {
        let x = sweep(case);
        let xm = mono(&x);
        let hz = case.rate();
        let env = envelope_spectrum(&xm[..sweep_len]);
        let zipper = 10.0 * (at_rate(&env, hz) / at_rate(&ref_env, hz)).log10();
        // (The last half second, the knob long at rest: the same in every case.)
        let tail = &x.0[x.0.len() - 24_000..];
        ends.push((case.name, tail.to_vec()));
        eprintln!(
            "{:<26} {:>7.1} Hz  {zipper:+6.1} dB   peak {:+.1} dBFS",
            case.name,
            hz,
            20.0 * (peak(&x) * gain).log10()
        );
        if let Some(d) = &out {
            write_wav(&d.join(format!("cutoff-{}-{tag}.wav", case.name)), &x, gain);
        }
    }
    // Every way lands on the same value: the held tails the same but for the oscillator's
    // phase, which the ways' different paths leave different; their levels within 0.1 dB.
    let rms =
        |x: &[f32]| (x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / x.len() as f64).sqrt();
    let first = rms(&ends[0].1);
    for (name, t) in &ends {
        let d = 20.0 * (rms(t) / first).log10();
        eprintln!("{name:<26} held tail {d:+.3} dB against the first");
        assert!(d.abs() < 0.1, "{name}");
    }
}

/// Every factory preset through `process`, then the next one chosen in the drawer.
#[test]
#[ignore = "a comparison, run by hand"]
fn presets_through_process() {
    let out = std::env::var_os("CA72_RENDER_OUT").map(PathBuf::from);
    let reference = std::env::var_os("CA72_RENDER_REF").map(PathBuf::from);
    assert!(
        out.is_some() || reference.is_some(),
        "set CA72_RENDER_OUT or CA72_RENDER_REF"
    );
    let all = factory();
    assert!(all.len() >= 19);
    let dir = tempfile::tempdir().unwrap();
    let mut library = Browser::new(Library::at(dir.path()));
    let mut differ = Vec::new();
    for (i, s) in all.iter().enumerate() {
        let next = &all[(i + 1) % all.len()];
        let (mut p, c) = hosted(s, 256, ProcessMode::Offline, |_, _| {});
        let params = p.parameters().clone();
        let gui = Gui(Mutex::new(TestProcessContext::new(
            params.clone(),
            RATE as f32,
            ProcessMode::Offline,
        )));
        let setter = ParamSetter::new(&gui);
        let mut c = c;
        let chord: &[u8] = if params.poly.value() {
            &[48, 55, 60, 64]
        } else {
            &[48]
        };
        let (mut l, mut r) = (Vec::new(), Vec::new());
        for b in 0..(3.0 * RATE / 256.0) as usize {
            match b {
                0 => chord.iter().for_each(|&k| c.push_event(note(0, k, true))),
                // Two seconds in: the next preset chosen, a new chord.
                375 => {
                    let e = library.library.find(&next.name).unwrap();
                    library.choose(&e, &params, &setter);
                    chord.iter().for_each(|&k| c.push_event(note(0, k, false)));
                    chord
                        .iter()
                        .for_each(|&k| c.push_event(note(9, k + 2, true)));
                }
                250 => c.push_event(note(17, chord[0] + 12, true)),
                _ => {}
            }
            let (bl, br) = block(&mut p, &mut c, 256);
            l.extend(bl);
            r.extend(br);
        }
        let bytes: Vec<u8> = l
            .iter()
            .zip(&r)
            .flat_map(|(a, b)| a.to_le_bytes().into_iter().chain(b.to_le_bytes()))
            .collect();
        let file = format!("{}.f32", s.name.replace(' ', "_"));
        if let Some(d) = &out {
            std::fs::create_dir_all(d).unwrap();
            std::fs::write(d.join(&file), &bytes).unwrap();
        }
        if let Some(d) = &reference {
            let want = std::fs::read(d.join(&file)).unwrap();
            let same = want == bytes;
            eprintln!(
                "{:<20} then {:<20} {}",
                s.name,
                next.name,
                if same {
                    "the same to the bit"
                } else {
                    "DIFFERS"
                }
            );
            if !same {
                differ.push(s.name.clone());
            }
        }
    }
    assert!(differ.is_empty(), "{differ:?}");
}

#[test]
#[ignore = "a measurement, run by hand"]
fn glide_cost() {
    let s = common::preset("Felt Bass");
    for quality in [QualityMode::Lo, QualityMode::Hi] {
        for moving in [false, true] {
            let mut best = f64::MAX;
            for _ in 0..3 {
                let (mut p, mut c) = hosted(&s, 256, ProcessMode::Offline, |q, c| {
                    assert!(c.automate(q.poly.as_ptr(), 1.0));
                    assert!(c.automate(q.voices.as_ptr(), 1.0));
                    assert!(c.automate(q.quality.as_ptr(), q.quality.preview_normalized(quality)));
                });
                let params = p.parameters().clone();
                for k in [36u8, 43, 48, 52, 55, 60, 64, 67, 71, 74] {
                    c.push_event(note(0, k, true));
                }
                let blocks = (4.0 * RATE / 256.0) as usize;
                let start = std::time::Instant::now();
                for b in 0..blocks {
                    if moving {
                        let v = 0.3 + 0.4 * (b as f32 * 0.01).sin().abs();
                        assert!(c.automate(params.cutoff.as_ptr(), v));
                    }
                    block(&mut p, &mut c, 256);
                }
                best = best.min(start.elapsed().as_secs_f64() / 4.0);
            }
            eprintln!(
                "{quality:?} {}: {:.1} ms a second of audio",
                if moving { "CUTOFF moving" } else { "at rest" },
                best * 1000.0
            );
        }
    }
}

fn write_wav(path: &std::path::Path, (l, r): &(Vec<f32>, Vec<f32>), gain: f32) {
    let frames = l.len().min(r.len());
    let data = (frames * 8) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(RATE as u32).to_le_bytes());
    b.extend_from_slice(&(RATE as u32 * 8).to_le_bytes());
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
