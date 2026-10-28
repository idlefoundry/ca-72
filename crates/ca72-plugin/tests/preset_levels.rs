//! Each factory preset played its phrase (`common::phrase`, by its tags: in its register, mono
//! or POLY as it sets them, the arpeggiator's held keys), through the plug-in's own `process`
//! as a host renders offline at 120 BPM, everything the preset sets in force (its LFOs, macros,
//! velocity, arpeggiator, timed and CONTINUOUS oscillators, its AUTO GAIN curve); and its
//! loudness: the momentary maximum (EBU R 128) by which MAIN OUTPUT VOLUME and LEVEL level the
//! presets (decisions.md R15, R-PRESETS-0.3.0: -18 LUFS midway between QUALITY LO and HI),
//! measured the same way before and after a change to the sound. Each render takes one seed
//! (`common::host::SEED`), so a run repeats to the bit.
//!
//! By hand: `cargo test --release -p ca72-plugin --test preset_levels -- --ignored --nocapture`.
//! `CA72_ONLY` (a part of a preset's name), `CA72_QUALITY` (`lo`, `hi`, `ultra`, or several
//! with commas, or `all`; LO by default, a new instance's), `CA72_SET` (`key=value,...`: a
//! preset's plain values changed, to try one), `CA72_LEVELS_OUT` (a directory: each render as
//! `<preset>.f32`, stereo, interleaved, little-endian; `<preset>_<quality>.f32` for more than
//! one quality), `CA72_WAV_OUT` (a directory: each render as `<preset>.wav`, 48 kHz, 24 bits,
//! at the level it plays; an expressive preset's again with a pressure swell),
//! `CA72_LEVELS_JSON` (a file: each preset's readings, `{"<preset>": {"lo": LUFS, ...}}`, for
//! `scripts/level_presets.py`), `CA72_PHRASE` (a phrase file of
//! `scripts/calibration/midi_capture.py`, played instead: the notes and modulation wheel sent
//! to the hardware reference, at their times, and 3 s after the last). `CA72_RETIRED` (set):
//! the presets retired from the plug-in (`sounds/retired.toml`, decisions.md R55) measured
//! instead, which its pack ships. `CA72_USER_PRESETS` (a presets folder): the user's presets in
//! it measured instead, each as the library reads it (decisions.md R-OWNER-PRESETS: the owner's
//! files against the factory's levelled versions of them). `CA72_VELOCITY` (1 to 127): the notes'
//! velocity, 100 by default. `CA72_NEVER_SLEEP` (set): the engine
//! alone plays each phrase at LO, the one instrument never sleeping (decisions.md R68), to show
//! that its sleeping changes no sample (the engine knows no arpeggiator or transport).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_analysis::loudness::loudness;
use ca72_plugin::engine::{Engine, Event};
use ca72_plugin::library::{Library, Origin, Sound, factory, parse_factory};
use common::host::{Play, Quality, RATE, TARGET_LUFS, render, write_wav};
use common::{controls_of, env, phrase, played_as};

/// `CA72_PHRASE`'s notes and modulation wheel: `[{"t": seconds, "kind": "on" | "off",
/// "note": n}, {"t": seconds, "kind": "cc", "cc": 1, "value": v}, ...]`.
#[allow(clippy::type_complexity)]
fn phrase_file() -> Option<(Vec<(f64, f64, Vec<u8>)>, Vec<(f64, u8, f32)>)> {
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
                wheel.push((t, 1, v));
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

/// How `s` is played at `q`: its phrase, or `CA72_PHRASE`'s.
fn play_of(s: &Sound, q: Quality) -> Play {
    let mut p = match phrase_file() {
        Some((notes, cc)) => {
            let mut p = Play::new(notes, q);
            p.seconds += 1.0;
            p.cc = cc;
            p
        }
        None => Play::phrase(s, q),
    };
    p.velocity = env("CA72_VELOCITY", 100.0f32) / 127.0;
    p
}

/// The engine alone playing the phrase at LO, never sleeping (`CA72_NEVER_SLEEP`).
fn engine_render(s: &Sound) -> (Vec<f32>, Vec<f32>) {
    const BLOCK: usize = 256;
    let c = played_as(s, controls_of(s));
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(RATE, 1);
    e.never_sleep(true);
    common::workers(&mut e);
    let notes = phrase(s);
    let end = notes.iter().map(|n| n.0 + n.1).fold(0.0, f64::max) + 2.0;
    let mut events: Vec<(usize, Event)> = Vec::new();
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

fn qualities() -> Vec<Quality> {
    let q: String = env("CA72_QUALITY", "lo".to_owned());
    if q == "all" {
        return Quality::ALL.to_vec();
    }
    q.split(',').map(Quality::of).collect()
}

#[test]
#[ignore = "a measurement, run by hand"]
fn each_preset_in_its_register() {
    let only: String = env("CA72_ONLY", String::new());
    let dir = |name: &str| -> Option<std::path::PathBuf> {
        let d: std::path::PathBuf = std::env::var_os(name)?.into();
        std::fs::create_dir_all(&d).unwrap();
        Some(d)
    };
    let (out, wav) = (dir("CA72_LEVELS_OUT"), dir("CA72_WAV_OUT"));
    let json = std::env::var_os("CA72_LEVELS_JSON");
    let never_sleep = std::env::var_os("CA72_NEVER_SLEEP").is_some();
    let qs = if never_sleep {
        vec![Quality::Lo]
    } else {
        qualities()
    };
    println!("preset                    quality  momentary max   integrated  to -18  (LUFS)");
    let sounds = if std::env::var_os("CA72_RETIRED").is_some() {
        parse_factory(include_str!("../sounds/retired.toml")).unwrap()
    } else if let Some(dir) = std::env::var_os("CA72_USER_PRESETS") {
        let (entries, bad) = Library::at(std::path::PathBuf::from(dir)).entries();
        assert!(bad.is_empty(), "{bad:?}");
        entries
            .into_iter()
            .filter(|e| e.origin != Origin::Factory)
            .map(|e| e.sound)
            .collect()
    } else {
        factory().to_vec()
    };
    let mut readings = serde_json::Map::new();
    for s in sounds.iter().filter(|s| s.name.contains(&only)) {
        let s = with_overrides(s);
        let mut reading = serde_json::Map::new();
        for &q in &qs {
            let (l, r) = if never_sleep {
                engine_render(&s)
            } else {
                render(&s, &play_of(&s, q))
            };
            let m = loudness(&[&l, &r], RATE as u32);
            let max = m.momentary_max.unwrap_or(f64::NEG_INFINITY);
            println!(
                "{:25} {:7} {:>13.2} {:>12.2} {:>+7.2}",
                s.name,
                q.name(),
                max,
                m.integrated.unwrap_or(f64::NEG_INFINITY),
                TARGET_LUFS - max,
            );
            reading.insert(q.name().into(), max.into());
            let file = |ext: &str| {
                if qs.len() > 1 {
                    format!("{}_{}.{ext}", s.name.replace(' ', "_"), q.name())
                } else {
                    format!("{}.{ext}", s.name.replace(' ', "_"))
                }
            };
            if let Some(d) = &out {
                let mut bytes = Vec::with_capacity(8 * l.len());
                for (a, b) in l.iter().zip(&r) {
                    bytes.extend_from_slice(&a.to_le_bytes());
                    bytes.extend_from_slice(&b.to_le_bytes());
                }
                std::fs::write(d.join(file("f32")), bytes).unwrap();
            }
            if let Some(d) = &wav {
                write_wav(&d.join(file("wav")), &l, &r, 1.0);
                if s.tags.iter().any(|t| t == "expressive") && !never_sleep {
                    let mut play = play_of(&s, q);
                    play.pressure = true;
                    let (l, r) = render(&s, &play);
                    write_wav(&d.join(file("pressure.wav")), &l, &r, 1.0);
                }
            }
        }
        readings.insert(s.name.clone(), reading.into());
    }
    if let Some(f) = json {
        let text = serde_json::to_string_pretty(&serde_json::Value::Object(readings)).unwrap();
        std::fs::write(f, text + "\n").unwrap();
    }
}
