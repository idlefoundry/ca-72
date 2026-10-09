//! The panel with a factory preset set, as a PNG:
//! `cargo run -p ca72-plugin --example preset_png -- "<preset>" out.png [scale]`.

use ca72_panel::{CONTROLS, Renderer, Scene};
use ca72_plugin::library::factory;
use ca72_plugin::params::Ca72Params;
use ca72_plugin::presets::sound_params;

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().expect("a preset's name");
    let out = args.next().unwrap_or_else(|| "preset.png".into());
    let scale: f64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let sound = factory()
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no factory preset {name}"));
    let params = Ca72Params::default();
    let values = sound_params(&params);
    let normalized = |id: &str| -> Option<f64> {
        let (_, q) = values.iter().find(|(k, _)| *k == id)?;
        Some(f64::from(
            match sound.values.iter().find(|(k, _)| k == id) {
                Some((_, v)) => q.normalized_of(*v),
                None => q.default_normalized(),
            },
        ))
    };
    let mut scene = Scene::default();
    for (i, c) in CONTROLS.iter().enumerate() {
        if let Some(v) = normalized(c.param) {
            scene.values[i] = v;
        }
    }
    // The wheels are drawn where a keyboard's would put them: the preset's MOD wheel.
    scene.midi = (0.5, normalized("mod_wheel").unwrap_or(0.0));
    let mut r = Renderer::new(scale, 1.0);
    r.render(&scene);
    if let Err(e) = r.frame().save_png(&out) {
        eprintln!("{out}: {e}");
        std::process::exit(1);
    }
    eprintln!("{out}: {name}");
}
