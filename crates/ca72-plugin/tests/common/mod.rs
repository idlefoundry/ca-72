//! What the factory presets' measurements share (`preset_cost.rs`, `preset_render.rs`,
//! `preset_levels.rs`, `drive_auto.rs`, `factory_curves.rs`, `factory_presets.rs`): a preset's
//! controls, its levelling phrase, notes played through the engine, POLY's chords played into
//! it a block at a time, and ([`host`]) a preset played through the plug-in's own `process`.

#![allow(dead_code, clippy::unwrap_used)]

use ca72_plugin::drive::{Curve, STEPS};
use ca72_plugin::engine::{Controls, Engine, Event};
use ca72_plugin::library::{Sound, factory, parse_factory};
use ca72_plugin::params::Ca72Params;

#[allow(unsafe_code)]
pub mod host;

/// The presets retired from the plug-in (`sounds/retired.toml`, decisions.md R55), which the
/// measurements made on them keep.
pub fn retired() -> Vec<Sound> {
    parse_factory(include_str!("../../sounds/retired.toml")).unwrap()
}

/// The preset named `name`, the factory's or a retired one; else the first whose name holds
/// it (the factory's first).
pub fn preset(name: &str) -> Sound {
    let all: Vec<Sound> = factory().iter().cloned().chain(retired()).collect();
    let exact = all.iter().find(|s| s.name == name);
    exact
        .or_else(|| all.iter().find(|s| s.name.contains(name)))
        .cloned()
        .unwrap_or_else(|| panic!("no preset {name}"))
}

/// The variable `name` read as a `T`, `default` if it is not set; a value set that does not
/// read as one stops the run, as a default in its place would measure something not asked for
/// (zsh's `env $e`, with `e="CA72_ULTRA=1 CA72_VOICES=4"`, sets one variable: decisions.md
/// R-RENDER-SETTINGS).
pub fn env<T: std::str::FromStr>(name: &str, default: T) -> T {
    match std::env::var(name) {
        Err(_) => default,
        Ok(s) => s
            .parse()
            .unwrap_or_else(|_| panic!("{name}={s:?} does not read as its value")),
    }
}

/// Whether the switch `name` is set: unset, off; `1` or empty, on; anything else stops the run
/// (as [`env`]).
pub fn switch(name: &str) -> bool {
    match std::env::var(name) {
        Err(_) => false,
        Ok(s) if s.is_empty() || s == "1" => true,
        Ok(s) => panic!("{name}={s:?}: set it to 1 (or leave it unset)"),
    }
}

/// The engine's controls a preset sets, as the plug-in plays it: the preset loaded into the
/// plug-in's parameters as its drawer loads one ([`host::apply`]: every parameter its value,
/// else its default; what its LFOs, macros and velocity modulate; its macros' names), then the
/// plug-in's own mapping, `Ca72Params::controls`. POLY and VOICES are left at their defaults
/// ([`played_as`] sets them); QUALITY too.
pub fn controls_of(s: &Sound) -> Controls {
    let fresh = Ca72Params::default().controls();
    let mut c = host::params_of(s).controls();
    (c.poly, c.voices) = (fresh.poly, fresh.voices);
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

/// When a phrase's first note starts, s: at once, as the factory set was levelled (with
/// [`host::SEED`], the oscillators' phases at the note's start the same).
pub const LEAD: f64 = 0.0;

/// A preset's levelling phrase, by its tags: (start, length) in seconds and the keys held,
/// from [`LEAD`] on. The arpeggiator's (`arp`) a key (a bass) or a four-note chord held 6 s
/// (with HOLD, let go after a second, the arpeggiator playing on); drums four hits on their
/// note (57; the snare 54; a tom's down the keys, 45 to 60); a percussive effect four hits from
/// the top (72 to 60); another effect one note held 6 s; POLY presets three six-note chords;
/// basses three notes from C2, the others three from C4, each held 1.5 s (3 s for the slow
/// ones).
pub fn phrase(s: &Sound) -> Vec<(f64, f64, Vec<u8>)> {
    let tag = |t: &str| s.tags.iter().any(|x| x == t);
    let value = |id: &str| s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v);
    let poly = value("poly").is_some_and(|v| v >= 0.5);
    if tag("arp") {
        let keys = if tag("bass") {
            vec![36]
        } else {
            vec![48, 52, 55, 59]
        };
        let held = if value("arp_latch").is_some_and(|v| v >= 0.5) {
            1.0
        } else {
            6.0
        };
        return vec![(LEAD, held, keys)];
    }
    let hits = |keys: [u8; 4]| {
        keys.iter()
            .enumerate()
            .map(|(k, &key)| (LEAD + 0.6 * k as f64, 0.25, vec![key]))
            .collect()
    };
    if tag("drums") {
        return hits(if tag("tom") {
            [45, 50, 55, 60]
        } else if tag("snare") {
            [54; 4]
        } else {
            [57; 4]
        });
    }
    if tag("fx") && tag("percussive") {
        return hits([72, 67, 64, 60]);
    }
    if tag("fx") {
        return vec![(LEAD, 6.0, vec![48])];
    }
    if poly {
        return [48u8, 53, 55]
            .iter()
            .enumerate()
            .map(|(k, &r)| {
                let keys = [0u8, 4, 7, 12, 16, 19].iter().map(|d| r + d).collect();
                (LEAD + 3.0 * k as f64, 2.5, keys)
            })
            .collect();
    }
    let held = if tag("slow") { 3.0 } else { 1.5 };
    let roots: [u8; 3] = if tag("bass") {
        [36, 43, 40]
    } else {
        [60, 67, 64]
    };
    roots
        .iter()
        .enumerate()
        .map(|(k, &r)| (LEAD + (held + 0.5) * k as f64, held, vec![r]))
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
