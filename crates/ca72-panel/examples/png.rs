//! Renders the panel to a PNG: `cargo run -p ca72-panel --example png -- out.png [scale]`
//! (pixels a panel unit, 0.5 by default); `CA72_SKIN=worn` in the worn skin.

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
    // The plugin's defaults: as the mock-up was drawn.
    // `CA72_ALL=0.5`: every control at one value instead.
    let all: Option<f64> = std::env::var("CA72_ALL").ok().and_then(|s| s.parse().ok());
    for (i, c) in CONTROLS.iter().enumerate() {
        scene.values[i] = all.unwrap_or_else(|| default_of(c.param));
    }
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
    let scene = example_strip();
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

/// The strip as the mock-up showed it: POLY, SCATTER, EVEN, AUTO GAIN on, eight voices of which
/// four sound, a preset chosen.
fn example_strip() -> ca72_panel::strip::StripScene {
    use ca72_panel::strip::{Field, StripScene};
    let r = |t: &str, on: bool| (t.to_owned(), on);
    StripScene {
        mode: Some(1),
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
        ],
        field: Field::Scatter(
            [-1.0, 1.0, -0.43, 0.43, -0.71, 0.71, -0.14, 0.14]
                .iter()
                .enumerate()
                .map(|(i, &p)| (p * 0.7, i % 2 == 0))
                .collect(),
        ),
        bar: ca72_panel::presets::BarScene {
            name: "Brass Tutti".into(),
            found: true,
            favorite: true,
            ..Default::default()
        },
        ..StripScene::default()
    }
}
