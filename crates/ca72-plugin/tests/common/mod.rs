//! What the factory presets' measurements share (`preset_cost.rs`, `preset_render.rs`,
//! `preset_levels.rs`, `drive_auto.rs`): a preset's controls, its levelling phrase, notes
//! played through the engine, and POLY's chords played into it a block at a time.

#![allow(dead_code, clippy::unwrap_used)]

use ca72_plugin::drive::{Curve, STEPS};
use ca72_plugin::engine::{Controls, Engine, Event};
use ca72_plugin::library::Sound;
use ca72_plugin::params::{Ca72Params, Footage, Scatter, Wave, Wave3};
use nih_plug::prelude::Enum;

pub fn env<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// The engine's controls a preset sets (the plug-in's own mapping, `Ca72Params::controls`,
/// from the preset's plain values; the rest at the parameters' defaults).
pub fn controls_of(s: &Sound) -> Controls {
    let mut c = Ca72Params::default().controls();
    let on = |v: f64| v >= 0.5;
    for (id, v) in &s.values {
        let v = *v;
        let p = &mut c.panel;
        match id.as_str() {
            "tune" => p.tune = v / 2.5,
            "glide" => p.glide = v / 10.0,
            "mod_mix" => p.mod_mix = v / 10.0,
            "osc_mod" => p.osc_mod = on(v),
            "osc3_control" => p.osc3_control = on(v),
            "osc1_range" | "osc2_range" | "osc3_range" => {
                let i = (id.as_bytes()[3] - b'1') as usize;
                p.osc[i].range = Footage::from_index(v as usize).into();
            }
            "osc1_waveform" | "osc2_waveform" => {
                let i = (id.as_bytes()[3] - b'1') as usize;
                p.osc[i].waveform = Wave::from_index(v as usize).into();
            }
            "osc3_waveform" => p.osc[2].waveform = Wave3::from_index(v as usize).into(),
            "osc2_frequency" => p.osc[1].freq = (v + 7.5) / 15.0,
            "osc3_frequency" => p.osc[2].freq = (v + 7.5) / 15.0,
            "osc1_on" => p.osc[0].on = on(v),
            "osc2_on" => p.osc[1].on = on(v),
            "osc3_on" => p.osc[2].on = on(v),
            "osc1_volume" => p.osc[0].volume = v / 10.0,
            "osc2_volume" => p.osc[1].volume = v / 10.0,
            "osc3_volume" => p.osc[2].volume = v / 10.0,
            "ext_on" => p.ext_on = on(v),
            "ext_volume" => p.ext_volume = v / 10.0,
            "noise_on" => p.noise_on = on(v),
            "noise_volume" => p.noise_volume = v / 10.0,
            "noise_type" => p.noise_pink = on(v),
            "filter_mod" => p.filter_mod = on(v),
            "keyboard_control_1" => p.keyboard_control_1 = on(v),
            "keyboard_control_2" => p.keyboard_control_2 = on(v),
            "cutoff" => p.cutoff = (v + 5.0) / 10.0,
            "emphasis" => p.emphasis = v / 10.0,
            "contour_amount" => p.contour_amount = v / 10.0,
            "filter_attack" => p.filter_contour.attack = v / 10.0,
            "filter_decay" => p.filter_contour.decay = v / 10.0,
            "filter_sustain" => p.filter_contour.sustain = v / 10.0,
            "loudness_attack" => p.loudness_contour.attack = v / 10.0,
            "loudness_decay" => p.loudness_contour.decay = v / 10.0,
            "loudness_sustain" => p.loudness_contour.sustain = v / 10.0,
            "glide_on" => p.glide_on = on(v),
            "decay_on" => p.decay = on(v),
            "a440" => p.a440 = on(v),
            "mod_wheel" => p.mod_wheel = v,
            "volume" => c.volume = v / 10.0,
            "main_output" => c.main_output = on(v),
            "midi_bend_range" => c.bend_range = v,
            "entropy" => c.entropy = v / 100.0,
            "spread" => c.spread = v / 100.0,
            "inner" => c.inner = v / 100.0,
            "unison" => c.unison = on(v),
            "double" => c.double = v / 100.0,
            "drive" => c.drive = v,
            "level" => c.level = v,
            "auto_gain" => c.auto_gain = on(v),
            // By the plug-in's choices (0 EVEN, 1 CENTER), as `presets.rs` takes a preset's.
            "placement" => {
                let last = Scatter::variants().len() - 1;
                c.placement = Scatter::from_index((v.round().max(0.0) as usize).min(last)).into();
            }
            "feedback" => c.feedback = ca72::voice::feedback_law(v / 10.0),
            "lock" => c.lock = on(v),
            "poly" | "voices" => {}
            other => panic!("{}: no parameter {other}", s.name),
        }
    }
    c
}

/// The workers asked for (`CA72_WORKERS`, 0: every voice on the caller's thread), left
/// ordinary threads (these measurements run as fast as they go); how many run.
pub fn workers(e: &mut Engine) -> usize {
    let n: usize = env("CA72_WORKERS", 0);
    e.start_workers(n, None)
}

/// Ten-note chords (as many notes as `voices`), a new one every half second, each held
/// seven eighths of it, so the next chord takes voices still releasing: their roots walk
/// through eight, rising a semitone each round of three.
#[derive(Debug)]
pub struct Chords {
    voices: usize,
    beat: usize,
    held: Vec<u8>,
}

impl Chords {
    pub fn new(voices: usize, rate: f64) -> Chords {
        Chords {
            voices,
            beat: (0.5 * rate) as usize,
            held: Vec::with_capacity(16),
        }
    }

    /// The notes at sample `n`, in order, each with its sample within the block (`at`).
    pub fn events(&mut self, n: usize, at: usize, out: &mut Vec<(usize, Event)>) {
        const ROOTS: [u8; 8] = [45, 41, 43, 48, 40, 45, 50, 43];
        const SHAPE: [u8; 10] = [0, 7, 12, 16, 19, 24, 28, 31, 34, 36];
        let beat = self.beat;
        if n % beat == beat * 7 / 8 {
            for key in self.held.drain(..) {
                out.push((at, Event::Note { key, on: false }));
            }
        }
        if n.is_multiple_of(beat) {
            let b = n / beat;
            let r = ROOTS[b % ROOTS.len()] + (b / ROOTS.len() % 3) as u8;
            for d in SHAPE.iter().take(self.voices) {
                out.push((
                    at,
                    Event::Note {
                        key: r + d,
                        on: true,
                    },
                ));
                self.held.push(r + d);
            }
        }
    }
}

/// One block of `left.len()` samples from sample `start` on, the chords' notes at their
/// samples, with nothing at EXTERNAL INPUT, as the plug-in's `process` plays its host's
/// block. Returns the OVERLOAD lamp after it.
pub fn play_block(
    e: &mut Engine,
    chords: &mut Chords,
    start: usize,
    left: &mut [f32],
    right: &mut [f32],
    events: &mut Vec<(usize, Event)>,
) -> f32 {
    let len = left.len();
    events.clear();
    for k in 0..len {
        chords.events(start + k, k, events);
    }
    let mut next = events.iter().peekable();
    let mut k = 0;
    while k < len {
        while let Some((_, ev)) = next.next_if(|(at, _)| *at <= k) {
            e.event(*ev);
        }
        let end = next.peek().map_or(len, |(at, _)| (*at).min(len));
        e.render(&[], &mut left[k..end], &mut right[k..end]);
        k = end;
    }
    e.end_block(len)
}

/// A phrase: (start, length) in seconds and the keys held.
pub fn phrase(s: &Sound) -> Vec<(f64, f64, Vec<u8>)> {
    let tag = |t: &str| s.tags.iter().any(|x| x == t);
    let poly = s.values.iter().any(|(k, v)| k == "poly" && *v >= 0.5);
    let held = if tag("slow") { 3.0 } else { 1.5 };
    if tag("drums") {
        let key = if tag("snare") { 54 } else { 57 };
        return (0..4).map(|k| (0.6 * k as f64, 0.25, vec![key])).collect();
    }
    if tag("fx") {
        return vec![(0.0, 6.0, vec![48])];
    }
    if poly {
        return [48u8, 53, 55]
            .iter()
            .enumerate()
            .map(|(k, &r)| {
                let keys = [0u8, 4, 7, 12, 16, 19].iter().map(|d| r + d).collect();
                (3.0 * k as f64, 2.5, keys)
            })
            .collect();
    }
    let roots: [u8; 3] = if tag("bass") {
        [36, 43, 40]
    } else {
        [60, 67, 64]
    };
    roots
        .iter()
        .enumerate()
        .map(|(k, &r)| ((held + 0.5) * k as f64, held, vec![r]))
        .collect()
}

/// The preset's POLY and VOICES on `c` (the measurements leave them out of [`controls_of`]).
pub fn played_as(s: &Sound, mut c: Controls) -> Controls {
    for (k, v) in &s.values {
        match k.as_str() {
            "poly" => c.poly = *v >= 0.5,
            "voices" => c.voices = *v as usize,
            _ => {}
        }
    }
    c
}

/// `notes` ((start, length) in seconds and the keys held) played with `c`, AUTO GAIN's `curve`
/// (None: the engine's own, the average's) for `end` seconds at `rate` Hz, in 256-sample blocks,
/// each event at its sample: left and right.
pub fn play(
    c: &Controls,
    curve: Option<Curve<STEPS>>,
    notes: &[(f64, f64, Vec<u8>)],
    end: f64,
    rate: f64,
) -> (Vec<f32>, Vec<f32>) {
    const BLOCK: usize = 256;
    let mut e = Engine::new();
    if let Some(curve) = curve {
        e.set_curve(curve);
    }
    e.set(c);
    e.prepare(rate, 1);
    workers(&mut e);
    let mut events: Vec<(usize, Event)> = Vec::new();
    for (t0, len, keys) in notes {
        for &key in keys {
            events.push(((t0 * rate) as usize, Event::Note { key, on: true }));
            events.push((((t0 + len) * rate) as usize, Event::Note { key, on: false }));
        }
    }
    events.sort_by_key(|e| e.0);
    let n = (end * rate / BLOCK as f64) as usize * BLOCK;
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut next = events.iter().peekable();
    for b in (0..n).step_by(BLOCK) {
        let mut k = b;
        while k < b + BLOCK {
            while let Some((_, ev)) = next.next_if(|(at, _)| *at <= k) {
                e.event(*ev);
            }
            let stop = next
                .peek()
                .map_or(b + BLOCK, |(at, _)| (*at).clamp(k + 1, b + BLOCK));
            e.render(&[], &mut l[k..stop], &mut r[k..stop]);
            k = stop;
        }
        e.end_block(BLOCK);
    }
    (l, r)
}
