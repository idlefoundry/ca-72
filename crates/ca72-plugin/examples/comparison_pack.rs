//! Render the comparison pack through the plug-in's engine, without opening an audio device.
//! `cargo run --release -p ca72-plugin --example comparison_pack`
//! The example uses the same preset-to-controls mapping as the existing preset measurements.

#[path = "../tests/common/mod.rs"]
mod common;

use ca72::freq_dial::{Law, Pot};
use ca72_analysis::{analyze, wav};
use ca72_panel::presets::BarScene;
use ca72_panel::strip::{StripRenderer, StripScene};
use ca72_panel::{CONTROLS, Renderer, Scene};
use ca72_plugin::engine::{Engine, Event};
use ca72_plugin::library::{Library, Origin, factory, parse_factory};
use ca72_plugin::params::Ca72Params;
use ca72_plugin::presets::sound_params;
use serde_json::{Value, json};
use std::path::Path;

const RATE: usize = 48_000;
const BLOCK: usize = 256;
const SEED: u64 = 72;

fn write_wav(path: &Path, samples: &[f32]) -> Result<(), String> {
    let bytes: Vec<u8> = samples.iter().flat_map(|x| x.to_le_bytes()).collect();
    let len = u32::try_from(bytes.len()).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(44 + bytes.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&3u16.to_le_bytes()); // IEEE float, mono.
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(RATE as u32).to_le_bytes());
    out.extend_from_slice(&(RATE as u32 * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&bytes);
    std::fs::write(path, out).map_err(|e| format!("{}: {e}", path.display()))
}

fn number(value: &Value) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("expected a number, got {value}"))
}

fn panel(path: &Path, name: &str, values: &[(String, f64)]) -> Result<(), String> {
    let params = Ca72Params::default();
    let mapping = sound_params(&params);
    let plain = |id: &str| values.iter().find(|(k, _)| k == id).map(|(_, v)| *v);
    let mut scene = Scene::default();
    for (i, control) in CONTROLS.iter().enumerate() {
        if control.param == "quality" {
            scene.values[i] = 1.0; // ULTRA, as rendered.
        } else if let Some((_, param)) = mapping.iter().find(|(k, _)| *k == control.param) {
            scene.values[i] = match plain(control.param) {
                Some(v)
                    if control.param == "osc2_frequency" || control.param == "osc3_frequency" =>
                {
                    let pot = if control.param == "osc2_frequency" {
                        Pot::Two
                    } else {
                        Pot::Three
                    };
                    pot.knob_at(Law::Semitones, (v + 7.5) / 15.0)
                }
                Some(v) => f64::from(param.normalized_of(v)),
                None => f64::from(param.default_normalized()),
            };
        }
    }
    scene.midi = (0.5, plain("mod_wheel").unwrap_or(0.0));
    scene.detune_off = true;
    scene.voices_off = true;
    let mut renderer = Renderer::new(0.5, 1.0);
    renderer.render(&scene);
    let strip_scene = StripScene {
        auto: plain("auto_gain").is_some_and(|v| v >= 0.5),
        readouts: [
            (format!("{:.0}", plain("voices").unwrap_or(2.0)), false),
            (String::from("OFF"), false),
            (String::from("OFF"), false),
            (String::from("OFF"), false),
            (String::from("0"), false),
            (String::from("0"), true),
            (String::from("0"), true),
            (String::from("OFF"), false),
        ],
        bar: BarScene {
            name: name.to_owned(),
            found: true,
            ..BarScene::default()
        },
        ..StripScene::default()
    };
    let mut strip = StripRenderer::new(0.5);
    strip.render(&strip_scene, renderer.frame(), None);
    let mut full = renderer.frame().clone();
    full.draw_pixmap(
        0,
        (ca72_panel::art::PANEL_H * 0.5).round() as i32,
        strip.frame().as_ref(),
        &Default::default(),
        Default::default(),
        None,
    );
    full.save_png(path).map_err(|e| e.to_string())
}

fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/comparison-presets");
    let sounds = parse_factory(
        &std::fs::read_to_string(root.join("bank.toml")).map_err(|e| e.to_string())?,
    )?;
    if sounds.len() != 24 {
        return Err("expected exactly 24 presets".into());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        if args != ["--panels-only"] {
            return Err("usage: comparison_pack [--panels-only]".into());
        }
        for sound in &sounds {
            panel(
                &root.join(format!("panels/{}.png", sound.name)),
                &sound.name,
                &sound.values,
            )?;
        }
        return Ok(());
    }
    let (entries, errors) = Library::at(root.join("presets")).entries();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let user_count = entries.iter().filter(|e| e.origin == Origin::User).count();
    if user_count != 24 {
        return Err(format!("expected 24 user presets, got {user_count}"));
    }
    let phrases: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("phrases.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let total = (number(&phrases["seconds"])? * RATE as f64).round() as usize;
    if total != 230_400 {
        return Err("expected exactly 4.8 seconds at 48 kHz".into());
    }
    let (_, input) =
        wav::read(&std::fs::read(root.join("external-input.wav")).map_err(|e| e.to_string())?)?;
    if input.len() != 1 || input[0].len() != total {
        return Err("external probe must be mono and the same length".into());
    }
    for folder in ["audio/raw", "audio/listening", "panels"] {
        std::fs::create_dir_all(root.join(folder)).map_err(|e| e.to_string())?;
    }
    let params = Ca72Params::default();
    let known = sound_params(&params);
    let mut measured = Vec::new();
    for (sound, phrase) in sounds.iter().zip(
        phrases["presets"]
            .as_array()
            .ok_or("presets is not an array")?,
    ) {
        if phrase["name"].as_str() != Some(sound.name.as_str()) {
            return Err("bank and phrases differ in order".into());
        }
        for (id, value) in &sound.values {
            if !value.is_finite() || !known.iter().any(|(key, _)| *key == id) {
                return Err(format!("{}: invalid parameter {id}", sound.name));
            }
        }
        if entries
            .iter()
            .find(|e| e.sound.name == sound.name)
            .map(|e| &e.sound)
            != Some(sound)
        {
            return Err(format!("{}: individual file differs from bank", sound.name));
        }
        let mut controls = common::controls_of(sound);
        if phrase["factory_source"].as_str() == Some("White Lotus") {
            let original = factory()
                .iter()
                .find(|s| s.name == "White Lotus")
                .ok_or("factory White Lotus is missing")?;
            if controls != common::controls_of(original) {
                return Err("comparison White Lotus differs from the factory's controls".into());
            }
        }
        controls.ultra = true;
        controls.poly = false;
        let mut engine = Engine::new();
        engine.set(&controls);
        engine.prepare(RATE as f64, SEED);
        engine.event(Event::PitchBend(0.0));
        if !phrase["cc1"].is_null() {
            engine.event(Event::Modulation(number(&phrase["cc1"])? as f32 / 127.0));
        }
        let mut events = Vec::new();
        let mut last_end = 0;
        for note in phrase["notes"].as_array().ok_or("notes is not an array")? {
            let start = (number(&note[0])? * RATE as f64).round() as usize;
            let end = ((number(&note[0])? + number(&note[1])?) * RATE as f64).round() as usize;
            let key = note[2].as_u64().ok_or("key is not an integer")?;
            if start < last_end || end <= start || end > total || key > 127 {
                return Err(format!(
                    "{}: notes overlap or exceed the window",
                    sound.name
                ));
            }
            events.push((start, true, key as u8));
            events.push((end, false, key as u8));
            last_end = end;
        }
        // Releases at a join precede the next attack. No two keys are held at any sample.
        events.sort_by_key(|&(at, on, _)| (at, on));
        let mut next = events.iter().peekable();
        let (mut left, mut right) = (vec![0.0; total], vec![0.0; total]);
        let external = phrase["number"].as_u64() == Some(23);
        for start in (0..total).step_by(BLOCK) {
            let end = (start + BLOCK).min(total);
            let mut at = start;
            while at < end {
                while let Some(&(_, on, key)) = next.next_if(|e| e.0 <= at) {
                    engine.event(Event::Note { key, on });
                }
                let stop = next.peek().map_or(end, |e| e.0.min(end));
                let input = if external { &input[0][at..stop] } else { &[] };
                engine.render(input, &mut left[at..stop], &mut right[at..stop]);
                at = stop;
            }
            engine.end_block(end - start);
        }
        if left
            .iter()
            .zip(&right)
            .any(|(l, r)| !l.is_finite() || !r.is_finite() || (l - r).abs() > 1e-7)
        {
            return Err(format!("{}: nonfinite or non-mono output", sound.name));
        }
        let peak = left.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
        if peak <= 1e-7 {
            return Err(format!("{}: silent output", sound.name));
        }
        let raw = analyze(std::slice::from_ref(&left), RATE as u32);
        let momentary = raw.loudness.momentary_max.ok_or("no measurable loudness")?;
        let true_peak = raw.true_peak_db.ok_or("no measurable true peak")?;
        let gain_db = (-18.0 - momentary).min(-1.0 - true_peak);
        let gain = 10f64.powf(gain_db / 20.0) as f32;
        let listening: Vec<f32> = left.iter().map(|x| x * gain).collect();
        let heard = analyze(std::slice::from_ref(&listening), RATE as u32);
        if heard.true_peak_db.is_none_or(|db| db > -0.98) {
            return Err(format!(
                "{}: listening take exceeds peak headroom",
                sound.name
            ));
        }
        write_wav(&root.join(format!("audio/raw/{}.wav", sound.name)), &left)?;
        write_wav(
            &root.join(format!("audio/listening/{}.wav", sound.name)),
            &listening,
        )?;
        panel(
            &root.join(format!("panels/{}.png", sound.name)),
            &sound.name,
            &sound.values,
        )?;
        eprintln!(
            "{}: {:.2} LUFS-M, {:.2} dBTP raw; listening gain {:+.2} dB",
            sound.name, momentary, true_peak, gain_db
        );
        measured.push(
            json!({"name": sound.name, "seconds": total as f64 / RATE as f64,
                            "channels": 1, "listening_gain_db": gain_db,
                            "last_raw_sample": left.last(), "raw": raw, "listening": heard}),
        );
    }
    let report = json!({"format": 1, "engine_version": env!("CARGO_PKG_VERSION"),
                        "quality": "ULTRA", "sample_rate": RATE, "seed": SEED,
                        "presets": measured, "hardware_recordings_included": false});
    std::fs::write(
        root.join("measurements.json"),
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
