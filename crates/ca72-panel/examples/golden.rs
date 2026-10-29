//! Frames of the panel for comparing two builds pixel for pixel: `cargo run --release -p
//! ca72-panel --example golden -- <dir>` writes one PNG per case (each skin and lettering, two
//! scales, scenes drawn in sequence so that the renderer's partial redraws are compared too).
//! Two directories written by two builds are compared with `--compare <a> <b>`.
//! `CA72_GOLDEN_LAYOUT=<file>` uses another layout, for comparing the lettering with a frozen
//! pack's without rebuilding the renderer.

use ca72_panel::{CONTROLS, Lettering, Renderer, Scene, Skin};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--compare") {
        compare(&args[1], &args[2]);
        return;
    }
    let dir = args.first().expect("an output directory");
    std::fs::create_dir_all(dir).expect("the directory");
    for (skin, sk) in [(Skin::Drawn, "drawn"), (Skin::Worn, "worn")] {
        for (lettering, lt) in [(Lettering::Standard, "std"), (Lettering::Large, "large")] {
            for scale in [0.44, 1.0] {
                let mut look = ca72_panel::Look::of(skin);
                if let Ok(path) = std::env::var("CA72_GOLDEN_LAYOUT") {
                    let text = std::fs::read_to_string(path).expect("the layout read");
                    let mut report = skin_pack::Report::default();
                    look.arrangement = std::sync::Arc::new(
                        ca72_panel::Arrangement::read(&text, &mut report)
                            .unwrap_or_else(|| panic!("{report}")),
                    );
                }
                let mut r = Renderer::with_pack(look, lettering, scale, 1.0);
                for (k, scene) in scenes().into_iter().enumerate() {
                    r.render(&scene);
                    let path = format!("{dir}/{sk}-{lt}-{scale}-{k}.png");
                    r.frame().save_png(&path).expect("written");
                }
            }
        }
    }
}

/// The scenes, drawn one after another by one renderer.
fn scenes() -> Vec<Scene> {
    let mut out = Vec::new();
    let half = Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    };
    out.push(half.clone());
    let mut varied = half.clone();
    for (i, v) in varied.values.iter_mut().enumerate() {
        *v = (i as f64 * 0.137) % 1.0;
    }
    varied.midi = (0.8, 0.3);
    out.push(varied.clone());
    let mut off = varied.clone();
    off.power = false;
    off.overload = 0.7;
    off.detune_off = true;
    off.voices_off = true;
    off.learning = Some(3);
    off.tip = Some(("CUTOFF FREQUENCY 1.2".into(), 2081.0, 300.0));
    out.push(off);
    out.push(Scene::default());
    out
}

fn compare(a: &str, b: &str) {
    let mut names: Vec<_> = std::fs::read_dir(a)
        .expect("a")
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.ends_with(".png"))
        .collect();
    names.sort();
    let mut differing = 0;
    for n in &names {
        let pa = ca72_panel::Pixmap::load_png(format!("{a}/{n}")).expect("a's");
        let Ok(pb) = ca72_panel::Pixmap::load_png(format!("{b}/{n}")) else {
            println!("{n}: missing in {b}");
            differing += 1;
            continue;
        };
        if pa.width() != pb.width() || pa.height() != pb.height() {
            println!("{n}: sizes differ");
            differing += 1;
            continue;
        }
        let (mut count, mut most) = (0usize, 0u8);
        for (p, q) in pa.pixels().iter().zip(pb.pixels()) {
            let d = [
                p.red().abs_diff(q.red()),
                p.green().abs_diff(q.green()),
                p.blue().abs_diff(q.blue()),
                p.alpha().abs_diff(q.alpha()),
            ]
            .into_iter()
            .max()
            .unwrap_or(0);
            if d > 0 {
                count += 1;
                most = most.max(d);
            }
        }
        if count > 0 {
            differing += 1;
            println!("{n}: {count} pixels differ, by at most {most}");
        }
    }
    println!("{} frames compared, {differing} differ", names.len());
}
