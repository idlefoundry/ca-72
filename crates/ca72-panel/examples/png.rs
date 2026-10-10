//! Renders the panel to a PNG: `cargo run -p ca72-panel --example png -- out.png [scale]`
//! (pixels a panel unit, 0.5 by default); `CA72_SKIN=worn` in the worn skin; `CA72_MONO=1` the
//! strip in MONO without DOUBLE (VOICES's, WIDTH's and INNER's readouts unlit, PLACEMENT none).

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
    let scene = example_strip(mono);
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
/// four sound, a preset chosen. In MONO (without DOUBLE) the one voice sounds in the centre, the
/// readouts lit as the editor lights them: VOICES's, WIDTH's and INNER's not, PLACEMENT's none.
fn example_strip(mono: bool) -> ca72_panel::strip::StripScene {
    use ca72_panel::strip::{Field, StripScene};
    let r = |t: &str, on: bool| (t.to_owned(), on);
    StripScene {
        mode: Some(if mono { 0 } else { 1 }),
        stereo: Some(0),
        placement: (!mono).then_some(0),
        auto: true,
        readouts: [
            r("8", !mono),
            r("40", true),
            r("70", !mono),
            r("OFF", false),
            r("-7.2", true),
            r("8.4", true),
            r("0.0", true),
            r("40", !mono),
        ],
        field: Field::Scatter(if mono {
            vec![(0.0, true)]
        } else {
            // EVEN's places for eight voices, each in its side's band at WIDTH 70 %, INNER 40 %.
            [-7.0, 7.0, -3.0, 3.0, -5.0, 5.0, -1.0, 1.0]
                .iter()
                .map(|p: &f64| p / 7.0)
                .enumerate()
                .map(|(i, p)| (0.7 * p.signum() * (0.4 + 0.6 * p.abs()), i % 2 == 0))
                .collect()
        }),
        bar: ca72_panel::presets::BarScene {
            name: "Brass Tutti".into(),
            found: true,
            favorite: true,
            ..Default::default()
        },
        ..StripScene::default()
    }
}
