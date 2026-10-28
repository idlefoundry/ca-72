//! A preset played through the plug-in's own `process`, as a host renders offline: everything
//! a preset sets is heard (the arpeggiator, the LFOs, the macros, velocity, the timed and
//! CONTINUOUS oscillators, pressure and timbre), at the host's tempo or the standalone's.

use ca72_plugin::Ca72;
use ca72_plugin::library::Sound;
use ca72_plugin::params::Ca72Params;
use ca72_plugin::presets::{sound_params, targets_for_sound};
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use std::path::Path;
use std::sync::Arc;

pub const RATE: f64 = 48_000.0;
pub const BLOCK: usize = 256;
/// The factory presets' loudness (decisions.md R15, R-PRESETS-0.3.0): the momentary maximum on
/// the preset's phrase, LUFS, midway between QUALITY LO and HI, and how far either may be.
pub const TARGET_LUFS: f64 = -18.0;
pub const TOLERANCE_DB: f64 = 1.5;
/// The session's seed every render takes (`noise_seed`), so that a render repeats to the bit: a
/// new instance's parts (ENTROPY) and its free-running oscillators' and noise's phases are
/// otherwise its own, and they move a preset's momentary maximum by up to about a decibel. The
/// engine's own measurements' seed (`Engine::prepare(rate, 1)`), by which the factory set was
/// levelled: through `process` a preset reads as it did there.
pub const SEED: u64 = 1;
/// The notes' velocity (MIDI 100): a velocity a preset reads (decisions.md R-VELOCITY) plays
/// as a firm touch, not the hardest.
pub const VELOCITY: f32 = 100.0 / 127.0;

pub struct Init;
impl InitContext<Ca72> for Init {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn execute(&self, _: ()) {}
    fn set_latency_samples(&self, _: u32) {}
    fn set_current_voice_capacity(&self, _: u32) {}
}

/// QUALITY.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Lo,
    Hi,
    Ultra,
}

impl Quality {
    pub const ALL: [Quality; 3] = [Quality::Lo, Quality::Hi, Quality::Ultra];

    /// The parameter's normalized value.
    pub fn normalized(self) -> f32 {
        match self {
            Quality::Lo => 0.0,
            Quality::Hi => 0.5,
            Quality::Ultra => 1.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Quality::Lo => "lo",
            Quality::Hi => "hi",
            Quality::Ultra => "ultra",
        }
    }

    /// `lo`, `hi` or `ultra`.
    pub fn of(name: &str) -> Quality {
        Quality::ALL
            .into_iter()
            .find(|q| q.name() == name.trim().to_lowercase())
            .unwrap_or_else(|| panic!("{name:?}: QUALITY is lo, hi or ultra"))
    }
}

/// Every value of `s` names one of the plug-in's sound parameters, or what an LFO, a macro or
/// velocity modulates (an assignment from a real source to a real target: one to none would
/// be dropped as it loads, without a word).
pub fn check(s: &Sound) {
    let p = Ca72Params::default();
    let params = sound_params(&p);
    for (k, _) in &s.values {
        assert!(
            ca72_plugin::lfo::Patch::is_value_id(k) || params.iter().any(|(id, _)| id == k),
            "{}: no parameter {k}",
            s.name
        );
        if let Some((source, target)) = k.split_once("_to_") {
            assert!(
                ca72_plugin::lfo::Source::of_id(source).is_some()
                    && ca72_plugin::lfo::Target::of_id(target).is_some(),
                "{}: no source or target in {k}",
                s.name
            );
        }
    }
}

/// Preset `s` set on `a` as the preset drawer sets one (`presets.rs`' `choose`): each parameter
/// its value, else its default (but the player's), landing at once; what its LFOs, macros and
/// velocity modulate, and its macros' names.
pub fn apply(a: &Arc<Ca72Params>, c: &TestProcessContext<Ca72>, s: &Sound) {
    check(s);
    let map = a.param_map();
    let landing = a.lands.begin();
    for (id, v) in targets_for_sound(a, s) {
        let (_, ptr, _) = map.iter().find(|(i, ..)| i == id).unwrap();
        a.lands.mark(landing, a, id, v);
        assert!(c.automate(*ptr, v), "{id}");
    }
    a.lfo_patch.load_values(&s.values);
    a.lfo_patch.load_names(&s.macro_names);
}

/// The plug-in's parameters with preset `s` loaded ([`apply`]), QUALITY its default.
pub fn params_of(s: &Sound) -> Arc<Ca72Params> {
    let a = Arc::new(Ca72Params::default());
    let c = TestProcessContext::<Ca72>::new(a.clone(), RATE as f32, ProcessMode::Offline);
    apply(&a, &c, s);
    a
}

/// [`apply`] at QUALITY `q`, with the preset's AUTO GAIN curve for it in force, as the drawer
/// loads it.
pub fn load(a: &Arc<Ca72Params>, c: &TestProcessContext<Ca72>, s: &Sound, q: Quality) {
    apply(a, c, s);
    assert!(c.automate(a.quality.as_ptr(), q.normalized()));
    if let Some(db) = s.curve(q == Quality::Lo) {
        a.drive_curve.load_preset(db);
    }
}

/// How a preset is played.
#[derive(Clone, Debug)]
pub struct Play {
    pub quality: Quality,
    /// (start, length) in seconds and the keys held.
    pub notes: Vec<(f64, f64, Vec<u8>)>,
    /// Controllers: (time in seconds, number, value 0..1).
    pub cc: Vec<(f64, u8, f32)>,
    /// How long it plays, s.
    pub seconds: f64,
    /// The host's transport playing at this tempo from the first bar (`Some`), else the
    /// standalone's: no transport, the standalone's TEMPO ([`Play::standalone_tempo`]).
    pub host_tempo: Option<f64>,
    pub standalone_tempo: f64,
    /// Channel pressure swelling from none to full and back over each note.
    pub pressure: bool,
    /// The notes' velocity, 0 to 1 ([`VELOCITY`]).
    pub velocity: f32,
}

impl Play {
    /// `notes` at `quality`, to two seconds past the last note's end, the host at 120 BPM.
    pub fn new(notes: Vec<(f64, f64, Vec<u8>)>, quality: Quality) -> Play {
        let seconds = notes.iter().map(|n| n.0 + n.1).fold(0.0, f64::max) + 2.0;
        Play {
            quality,
            notes,
            cc: Vec::new(),
            seconds,
            host_tempo: Some(120.0),
            standalone_tempo: ca72_plugin::params::DEFAULT_TEMPO,
            pressure: false,
            velocity: VELOCITY,
        }
    }

    /// The preset's own phrase ([`super::phrase`]); with the arpeggiator's HOLD, which plays on
    /// once the keys are let go, eight seconds of it.
    pub fn phrase(s: &Sound, quality: Quality) -> Play {
        let mut p = Play::new(super::phrase(s), quality);
        let on = |id: &str| s.values.iter().any(|(k, v)| k == id && *v >= 0.5);
        if on("arp_on") && on("arp_latch") {
            p.seconds = p.seconds.max(8.0);
        }
        p
    }
}

/// The notes and controllers as events at their samples, in order.
fn events(play: &Play) -> Vec<(usize, NoteEvent<()>)> {
    let at = |t: f64| (t * RATE).round() as usize;
    let mut e = Vec::new();
    for (t0, len, keys) in &play.notes {
        for &note in keys {
            let on = NoteEvent::NoteOn {
                timing: 0,
                voice_id: None,
                channel: 0,
                note,
                velocity: play.velocity,
            };
            let off = NoteEvent::NoteOff {
                timing: 0,
                voice_id: None,
                channel: 0,
                note,
                velocity: 0.0,
            };
            e.push((at(*t0), 2, note, on));
            e.push((at(t0 + len), 0, note, off));
        }
    }
    for &(t, cc, value) in &play.cc {
        let ev = NoteEvent::MidiCC {
            timing: 0,
            channel: 0,
            cc,
            value,
        };
        e.push((at(t), 1, cc, ev));
    }
    // (At one sample: releases, then controllers, then attacks.)
    e.sort_by_key(|&(t, order, n, _)| (t, order, n));
    e.into_iter().map(|(t, _, _, ev)| (t, ev)).collect()
}

/// Preset `s` played as `play` says through a new instance's `process`, offline, in
/// [`BLOCK`]-sample blocks: left and right.
pub fn render(s: &Sound, play: &Play) -> (Vec<f32>, Vec<f32>) {
    let len = (play.seconds * RATE / BLOCK as f64).ceil() as usize * BLOCK;
    let mut p = Ca72::default();
    let a = p.parameters().clone();
    // (Before `initialize`, which seeds the voices and the arpeggiator from it.)
    a.noise_seed
        .store(SEED, std::sync::atomic::Ordering::Relaxed);
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: BLOCK as u32,
        process_mode: ProcessMode::Offline,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    // (`initialize` reads this computer's settings: a new computer's in their place, so that a
    // render is the same on any, and the standalone's TEMPO asked for.)
    let d = ca72_plugin::settings::Settings::default();
    a.set_freq_law(d.frequency);
    a.midi_map.set_wheel_source(d.mod_wheel_source);
    a.set_mpe_mode(d.mpe);
    a.set_timbre_rest(d.timbre_rest);
    a.set_tempo(play.standalone_tempo);
    let mut c = TestProcessContext::<Ca72>::new(p.params(), RATE as f32, ProcessMode::Offline);
    if play.host_tempo.is_some() {
        c.set_plugin_api(PluginApi::Clap);
    }
    load(&a, &c, s, play.quality);
    let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
    let (mut bl, mut br) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    let events = events(play);
    let mut next = events.iter().peekable();
    let mut start = 0;
    while start < len {
        if let Some(bpm) = play.host_tempo {
            c.set_transport(
                true,
                Some(bpm),
                Some(start as f64 / RATE * bpm / 60.0),
                Some((4, 4)),
            );
        }
        if play.pressure {
            let t = start as f64 / RATE;
            let swell = play
                .notes
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
        while let Some((t, ev)) = next.next_if(|(t, _)| *t < start + BLOCK) {
            let mut ev = *ev;
            let timing = (t - start) as u32;
            match &mut ev {
                NoteEvent::NoteOn { timing: x, .. }
                | NoteEvent::NoteOff { timing: x, .. }
                | NoteEvent::MidiCC { timing: x, .. } => *x = timing,
                _ => {}
            }
            c.push_event(ev);
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

pub fn peak(l: &[f32], r: &[f32]) -> f64 {
    l.iter()
        .chain(r)
        .fold(0.0f64, |m, x| m.max(f64::from(x.abs())))
}

pub fn rms(l: &[f32], r: &[f32]) -> f64 {
    let n = (l.len() + r.len()).max(1) as f64;
    (l.iter()
        .chain(r)
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
}

/// Whether `s` uses anything new in 0.3: the arpeggiator, an LFO, a macro or velocity
/// assigned, a timed oscillator, a CONTINUOUS waveform, or pressure and timbre.
pub fn uses_0_3(s: &Sound) -> bool {
    s.values.iter().any(|(k, v)| {
        *v != 0.0
            && (k == "arp_on"
                || k.contains("_to_")
                || k.ends_with("_timed")
                || k.ends_with("_shape_mode")
                || k.starts_with("pressure_")
                || k.starts_with("timbre_"))
    })
}

/// Whether `s` follows the tempo: the arpeggiator, a timed oscillator or a timed LFO.
pub fn timed(s: &Sound) -> bool {
    s.values.iter().any(|(k, v)| {
        *v >= 0.5
            && (k == "arp_on"
                || (k.ends_with("_timed") && (k.starts_with("osc") || k.starts_with("lfo"))))
    })
}

/// `l` and `r` at `gain` as a WAV, 48 kHz, 24 bits.
pub fn write_wav(path: &Path, l: &[f32], r: &[f32], gain: f64) {
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
