//! What each factory preset costs with POLY's ten voices (decisions.md R10, "POLY in real
//! time"): the engine as a host calls it, 256-frame blocks at 48 kHz, ten-note chords
//! changing every half second (each held seven eighths of it), run as fast as it goes on one
//! thread (not paced, not promoted). Prints, per preset, the share of one core that real
//! time takes, and how many voices that leaves room for in 60 % of a block's period (the
//! strip's figure).
//!
//! By hand: `cargo test --release -p ca72-plugin --test preset_cost -- --ignored
//! --nocapture` (`CA72_SECONDS`, 4; `CA72_VOICES`, 10; `CA72_ONLY`, a part of a preset's
//! name; `CA72_NO_FEEDBACK`, set: FEEDBACK at 0).

#![allow(clippy::unwrap_used)]

use ca72_plugin::engine::{Controls, Engine, Event};
use ca72_plugin::library::{Sound, factory};
use ca72_plugin::params::{Ca72Params, Footage, Wave, Wave3};
use nih_plug::prelude::Enum;
use std::time::Instant;

fn env<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// The engine's controls a preset sets (the plug-in's own mapping, `Ca72Params::controls`,
/// from the preset's plain values; the rest at the parameters' defaults).
fn controls_of(s: &Sound) -> Controls {
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
            "analog" => c.analog = v / 100.0,
            "spread" => c.spread = v / 100.0,
            "feedback" => c.feedback = ca72::voice::feedback_law(v / 10.0),
            "lock" => c.lock = on(v),
            "poly" | "voices" => {}
            other => panic!("{}: no parameter {other}", s.name),
        }
    }
    c
}

#[test]
#[ignore = "a measurement, run by hand"]
fn each_presets_cost_with_ten_voices() {
    let voices: usize = env("CA72_VOICES", 10);
    let seconds: f64 = env("CA72_SECONDS", 4.0);
    let (rate, block) = (48_000.0, 256usize);
    let roots = [45u8, 41, 43, 48, 40, 45, 50, 43];
    let shape = [0u8, 7, 12, 16, 19, 24, 28, 31, 34, 36];
    let beat = (0.5 * rate) as usize;
    let blocks = (seconds * rate / block as f64) as usize;
    let mut rows = Vec::new();
    let only = std::env::var("CA72_ONLY").ok();
    let no_feedback = std::env::var_os("CA72_NO_FEEDBACK").is_some();
    for s in factory() {
        if only.as_ref().is_some_and(|o| !s.name.contains(o.as_str())) {
            continue;
        }
        let mut c = controls_of(s);
        c.poly = true;
        c.voices = voices;
        if no_feedback {
            c.feedback = 0.0;
        }
        let mut e = Engine::new();
        e.set(&c);
        e.prepare(rate, 1);
        let mut held: Vec<u8> = Vec::new();
        let mut worst = 0.0f64;
        let start = Instant::now();
        for i in 0..blocks {
            let t = Instant::now();
            for k in 0..block {
                let n = i * block + k;
                if n % beat == beat * 7 / 8 {
                    for key in held.drain(..) {
                        e.event(Event::Note { key, on: false });
                    }
                }
                if n.is_multiple_of(beat) {
                    let b = n / beat;
                    let r = roots[b % roots.len()] + (b / roots.len() % 3) as u8;
                    for d in shape.iter().take(voices) {
                        e.event(Event::Note {
                            key: r + d,
                            on: true,
                        });
                        held.push(r + d);
                    }
                }
                std::hint::black_box(e.tick_stereo(0.0));
            }
            e.end_block(block);
            worst = worst.max(t.elapsed().as_secs_f64());
        }
        let took = start.elapsed().as_secs_f64();
        let share = took / (blocks as f64 * block as f64 / rate);
        let period = block as f64 / rate;
        rows.push((s.name.clone(), share, worst / period, c.feedback > 0.0));
    }
    eprintln!("CA-72, {voices} POLY voices, 256-frame blocks at 48 kHz, one thread:");
    eprintln!(
        "{:<18} {:>8} {:>13} {:>10}",
        "preset", "of a core", "worst block", "voices fit"
    );
    for (name, share, worst, fed) in &rows {
        let fits = (0.6 / (share / voices as f64)).floor().min(10.0);
        eprintln!(
            "{name:<18} {:>7.0} % {:>11.0} % {fits:>10}{}",
            share * 100.0,
            worst * 100.0,
            if *fed { "  FEEDBACK" } else { "" }
        );
    }
}
