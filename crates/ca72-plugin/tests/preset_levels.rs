//! Each factory preset played a phrase in its register, mono or POLY as the preset sets it, and
//! its loudness: the momentary maximum (EBU R 128) by which MAIN OUTPUT VOLUME levels the
//! presets (decisions.md R15: about -18 LUFS), measured the same way before and after a
//! change to the sound. The phrases: drums four hits on their note (57; the snare 54); the
//! riser one note held 6 s; POLY presets three six-note chords; basses three notes from C2,
//! the others three from C4, each held 1.5 s (3 s for the slow ones).
//!
//! By hand: `cargo test --release -p ca72-plugin --test preset_levels -- --ignored --nocapture`.
//! `CA72_ONLY` (a part of a preset's name), `CA72_SET` (`key=value,...`: a preset's plain
//! values changed, to try one), `CA72_LEVELS_OUT` (a directory: each render as
//! `<preset>.f32`, stereo, interleaved, little-endian), `CA72_PHRASE` (a phrase file of
//! `scripts/calibration/midi_capture.py`, played instead: the notes and modulation wheel sent
//! to the hardware reference, at their times, and 3 s after the last).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::loudness::loudness;
use ca72_plugin::engine::{Engine, Event};
use ca72_plugin::library::{Sound, factory};
use common::{controls_of, env, phrase};

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

/// `CA72_PHRASE`'s notes and modulation wheel: `[{"t": seconds, "kind": "on" | "off",
/// "note": n}, {"t": seconds, "kind": "cc", "cc": 1, "value": v}, ...]`.
#[allow(clippy::type_complexity)]
fn phrase_file() -> Option<(Vec<(f64, f64, Vec<u8>)>, Vec<(f64, Event)>)> {
    let path = std::env::var_os("CA72_PHRASE")?;
    let ev: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut held: Vec<(u8, f64)> = Vec::new();
    let (mut notes, mut wheel) = (Vec::new(), Vec::new());
    for e in &ev {
        let t = e["t"].as_f64().unwrap();
        match e["kind"].as_str().unwrap() {
            "on" => held.push((e["note"].as_u64().unwrap() as u8, t)),
            "off" => {
                let note = e["note"].as_u64().unwrap() as u8;
                let i = held.iter().position(|h| h.0 == note).unwrap();
                let (_, t0) = held.remove(i);
                notes.push((t0, t - t0, vec![note]));
            }
            "cc" if e["cc"].as_u64() == Some(1) => {
                let v = e["value"].as_u64().unwrap() as f32 / 127.0;
                wheel.push((t, Event::Modulation(v)));
            }
            _ => {}
        }
    }
    Some((notes, wheel))
}

/// The preset with `CA72_SET`'s values in place of its own.
fn with_overrides(s: &Sound) -> Sound {
    let mut s = s.clone();
    let set: String = env("CA72_SET", String::new());
    for kv in set.split(',').filter(|x| !x.is_empty()) {
        let (k, v) = kv.split_once('=').unwrap();
        let v: f64 = v.parse().unwrap();
        match s.values.iter_mut().find(|(key, _)| key == k) {
            Some(x) => x.1 = v,
            None => s.values.push((k.to_string(), v)),
        }
    }
    s
}

fn render(s: &Sound) -> (Vec<f32>, Vec<f32>) {
    let mut c = controls_of(s);
    for (k, v) in &s.values {
        match k.as_str() {
            "poly" => c.poly = *v >= 0.5,
            "voices" => c.voices = *v as usize,
            _ => {}
        }
    }
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(RATE, 1);
    common::workers(&mut e);
    let (notes, wheel, tail) = match phrase_file() {
        Some((n, w)) => (n, w, 3.0),
        None => (phrase(s), Vec::new(), 1.5),
    };
    let end = notes.iter().map(|n| n.0 + n.1).fold(0.0, f64::max) + tail;
    // Note and wheel events at their samples.
    let mut events: Vec<(usize, Event)> = wheel
        .iter()
        .map(|(t, e)| ((t * RATE) as usize, *e))
        .collect();
    for (t0, len, keys) in &notes {
        for &key in keys {
            events.push(((t0 * RATE) as usize, Event::Note { key, on: true }));
            events.push((((t0 + len) * RATE) as usize, Event::Note { key, on: false }));
        }
    }
    events.sort_by_key(|e| e.0);
    let n = (end * RATE / BLOCK as f64) as usize * BLOCK;
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

#[test]
#[ignore = "a measurement, run by hand"]
fn each_preset_in_its_register() {
    let only: String = env("CA72_ONLY", String::new());
    let out: Option<std::path::PathBuf> = std::env::var_os("CA72_LEVELS_OUT").map(Into::into);
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    println!("preset               momentary max   integrated  (LUFS)");
    for s in factory().iter().filter(|s| s.name.contains(&only)) {
        let s = with_overrides(s);
        let (l, r) = render(&s);
        let m = loudness(&[&l, &r], RATE as u32);
        println!(
            "{:20} {:>13.2} {:>12.2}",
            s.name,
            m.momentary_max.unwrap_or(f64::NEG_INFINITY),
            m.integrated.unwrap_or(f64::NEG_INFINITY)
        );
        if let Some(d) = &out {
            let mut bytes = Vec::with_capacity(8 * l.len());
            for (a, b) in l.iter().zip(&r) {
                bytes.extend_from_slice(&a.to_le_bytes());
                bytes.extend_from_slice(&b.to_le_bytes());
            }
            std::fs::write(d.join(format!("{}.f32", s.name.replace(' ', "_"))), bytes).unwrap();
        }
    }
}
