//! Renders the panel to a PNG: `cargo run -p ca72-panel --example png -- out.png [scale]`
//! (pixels a panel unit, 0.5 by default); `CA72_SKIN=worn` in the worn skin; `CA72_MONO=1` the
//! strip in MONO without DOUBLE (the one voice lit at one of VOICES's places).
//! `CA72_STANDALONE=1` shows the standalone keyboard's TYPING and PANIC controls.

use ca72_panel::{CONTROLS, Renderer, Scene, Skin};

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "panel.png".into());
    let scale: f64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let t0 = std::time::Instant::now();
    let skin = match std::env::var("CA72_SKIN").as_deref() {
        Ok("worn") => Skin::Worn,
        _ => Skin::Drawn,
    };
    let mut r = Renderer::with_skin(skin, scale, 1.0);
    let built = t0.elapsed();
    let mut scene = Scene::default();
    // The plugin's defaults: as the mock-up was drawn; the strip's knobs at what its readouts
    // read. `CA72_ALL=0.5`: every control at one value instead.
    let all: Option<f64> = std::env::var("CA72_ALL").ok().and_then(|s| s.parse().ok());
    for (i, c) in CONTROLS.iter().enumerate() {
        scene.values[i] =
            all.unwrap_or_else(|| strip_knob(c.param).unwrap_or_else(|| default_of(c.param)));
    }
    let mono = std::env::var("CA72_MONO").is_ok_and(|v| v == "1");
    let t1 = std::time::Instant::now();
    r.render(&scene);
    let first = t1.elapsed();
    scene.values[0] = 0.8;
    let t2 = std::time::Instant::now();
    r.render(&scene);
    let one = t2.elapsed();
    scene.values[0] = default_of("tune");
    r.render(&scene);
    eprintln!("background {built:?}, every layer {first:?}, one knob {one:?}");
    // The strip's own parts over the drawing's strip (A6), as the editor shows them.
    let mut strip = ca72_panel::strip::StripRenderer::new(scale);
    let t3 = std::time::Instant::now();
    let mut scene = example_strip(mono);
    if std::env::var("CA72_KEYBOARD").is_ok_and(|v| v == "1") {
        let held = if std::env::var("CA72_KEYS_PLAYED").is_ok_and(|v| v == "1") {
            [(1u64 << 60), (1u64 << 0) | (1u64 << 3)]
        } else {
            [0, 0]
        };
        scene.keyboard = Some(ca72_panel::keyboard::Scene {
            skin,
            held,
            typing: std::env::var("CA72_TYPING").is_ok_and(|v| v == "1"),
            standalone: std::env::var("CA72_STANDALONE").is_ok_and(|v| v == "1"),
            ..Default::default()
        });
        scene.bar.keyboard = true;
    }
    strip.render(&scene, r.frame(), None);
    eprintln!("the strip {:?}", t3.elapsed());
    let mut all = r.frame().clone();
    let top = (ca72_panel::art::PANEL_H * scale).round() as usize;
    let w = all.width() as usize;
    let sf = strip.frame();
    let rows = (sf.height() as usize).min(all.height() as usize - top);
    for y in 0..rows {
        let n = w.min(sf.width() as usize) * 4;
        let (from, to) = (y * sf.width() as usize * 4, (top + y) * w * 4);
        all.data_mut()[to..to + n].copy_from_slice(&sf.data()[from..from + n]);
    }
    if std::env::var("CA72_KEYBOARD_CROP").is_ok_and(|v| v == "1") {
        let [x0, y0, x1, y1] = [370.0, 925.0, 3080.0, 1750.0].map(|v| (v * scale).round() as u32);
        let mut cropped = ca72_panel::Pixmap::new(x1 - x0, y1 - y0).expect("keyboard proof crop");
        let n = (x1 - x0) as usize * 4;
        for y in y0..y1 {
            let src = (y as usize * all.width() as usize + x0 as usize) * 4;
            let dst = (y - y0) as usize * n;
            cropped.data_mut()[dst..dst + n].copy_from_slice(&all.data()[src..src + n]);
        }
        all = cropped;
    }
    if let Err(e) = all.save_png(&out) {
        eprintln!("{out}: {e}");
        std::process::exit(1);
    }
}

/// The plugin's default for a parameter, normalized.
fn default_of(param: &str) -> f64 {
    match param {
        "tune" | "osc2_frequency" | "osc3_frequency" | "cutoff" | "pitch_wheel" => 0.5,
        "osc1_range" | "osc2_range" | "osc3_range" => 0.6,
        "osc1_waveform" | "osc2_waveform" | "osc3_waveform" => 0.4,
        "osc1_volume" | "osc2_volume" | "osc3_volume" => 0.7,
        "osc1_on" | "osc2_on" | "osc3_on" | "main_output" | "keyboard_control_1"
        | "keyboard_control_2" | "osc_mod" => 1.0,
        "emphasis" | "glide" | "mod_mix" | "mod_wheel" | "ext_volume" | "noise_volume" => 0.0,
        "contour_amount" | "filter_sustain" | "loudness_sustain" | "volume" => 0.5,
        "filter_attack" | "loudness_attack" => 0.1,
        "filter_decay" | "loudness_decay" => 0.4,
        _ => 0.0,
    }
}

/// The strip's knobs at what [`example_strip`]'s readouts read (normalized): VOICES 8 (of 2 to
/// 10), ENTROPY 40 %, WIDTH 70 %, INNER 40 %, DRIVE 8.4 dB (of 24), LEVEL 0 dB (of -30 to +12);
/// DETUNE off.
fn strip_knob(param: &str) -> Option<f64> {
    Some(match param {
        "voices" => 0.75,
        "entropy" | "inner" => 0.4,
        "spread" => 0.7,
        "drive" => 8.4 / 24.0,
        "level" => 30.0 / 42.0,
        _ => return None,
    })
}

/// The strip as the mock-up showed it: POLY, SCATTER, EVEN, AUTO GAIN on, eight voices of which
/// four sound, a preset chosen. In MONO (without DOUBLE) the one voice sounds at one of the eight
/// places, the one its last attack took (decisions.md R62), the readouts lit as in POLY.
fn example_strip(mono: bool) -> ca72_panel::strip::StripScene {
    use ca72_panel::strip::{Field, StripScene};
    let r = |t: &str, on: bool| (t.to_owned(), on);
    StripScene {
        mode: Some(if mono { 0 } else { 1 }),
        stereo: Some(0),
        placement: Some(0),
        auto: true,
        readouts: [
            r("8", true),
            r("40", true),
            r("70", true),
            r("OFF", false),
            r("-7.2", true),
            r("8.4", true),
            r("0.0", true),
            r("40", true),
        ],
        // EVEN's places for eight voices, each in its side's band at WIDTH 70 %, INNER 40 %.
        field: Field::Scatter(
            [-7.0, 7.0, -3.0, 3.0, -5.0, 5.0, -1.0, 1.0]
                .iter()
                .map(|p: &f64| p / 7.0)
                .enumerate()
                .map(|(i, p)| {
                    let on = if mono { i == 2 } else { i % 2 == 0 };
                    (0.7 * p.signum() * (0.4 + 0.6 * p.abs()), on)
                })
                .collect(),
        ),
        bar: ca72_panel::presets::BarScene {
            panel: true,
            name: "Brass Tutti".into(),
            found: true,
            favorite: true,
            ..Default::default()
        },
        ..StripScene::default()
    }
}
