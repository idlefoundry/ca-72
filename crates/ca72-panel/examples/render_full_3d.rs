//! Compose the 4K Cycles upper panel and lower extension, with live strip artwork.
//!
//! cargo run --release -p ca72-panel --example render_full_3d -- <upper pack> <lower build> <output>
//!
//! On Linux, use `--window` in place of `<output>` for an interactive visual preview.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use ca72_panel::art::PANEL_H;
use ca72_panel::panel3d::{FaceRect, FaceSpec, Panel3DAssets, Panel3DRenderer, Panel3DState};
use ca72_panel::presets::{BarScene, BarTarget};
use ca72_panel::strip::{self, Field, StripRenderer, StripScene};
use resvg::tiny_skia::{IntRect, Pixmap, PixmapPaint, Transform};
use serde::Deserialize;

#[derive(Deserialize)]
struct LowerLayout {
    cheek_w: f32,
    controls: Vec<Fixture>,
    #[serde(default)]
    bounds: Option<[f32; 4]>,
}

#[derive(Deserialize)]
struct Fixture {
    kind: String,
    x: f32,
    y: f32,
    #[serde(default)]
    w: f32,
    #[serde(default)]
    h: f32,
    #[serde(default)]
    param: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    sprite: String,
    #[serde(default)]
    value: f32,
    #[serde(default)]
    hit_r: Option<f32>,
    #[serde(default)]
    across: Option<bool>,
    #[serde(default)]
    steps: Option<usize>,
    #[serde(default)]
    shape: Vec<[f32; 2]>,
}

fn upper_state(assets: &Panel3DAssets) -> Panel3DState {
    let manifest = &assets.manifest;
    let mut state = Panel3DState::from_defaults(manifest);
    for (id, value) in [
        ("glide", 0.22),
        ("mod_mix", 0.35),
        ("osc1_range", 0.60),
        ("osc2_range", 0.60),
        ("osc2_frequency", 0.52),
        ("osc3_frequency", 0.48),
        ("osc1_on", 1.0),
        ("osc2_on", 1.0),
        ("osc3_on", 1.0),
        ("osc1_volume", 0.78),
        ("osc2_volume", 0.72),
        ("osc3_volume", 0.65),
        ("cutoff", 0.61),
        ("emphasis", 0.38),
        ("volume", 0.74),
        ("main_output", 1.0),
    ] {
        state.set_control(manifest, id, value);
    }
    state.set_lamp(manifest, "power", 0.78);
    state
}

fn demo_strip() -> StripScene {
    StripScene {
        mode: Some(1),
        stereo: Some(0),
        placement: Some(0),
        auto: true,
        readouts: [
            ("8", true),
            ("40", true),
            ("70", true),
            ("OFF", false),
            ("-7.2", true),
            ("8.4", true),
            ("0.0", true),
            ("30", true),
        ]
        .map(|(text, lit)| (text.into(), lit)),
        field: Field::Scatter(
            [-0.78, -0.52, -0.31, -0.13, 0.22, 0.43, 0.62, 0.82]
                .map(|pan| (pan, true))
                .to_vec(),
        ),
        levels: vec![0.25, 0.55, 0.65, 0.4, 0.48, 0.7, 0.59, 0.35],
        bar: BarScene {
            panel: true,
            found: true,
            favorite: true,
            name: "Brass Tutti".into(),
            ..BarScene::default()
        },
        ..StripScene::default()
    }
}

fn lower_assets(
    upper: &Panel3DAssets,
    face: Pixmap,
    cheek_w: f32,
    fixtures: &[Fixture],
) -> Panel3DAssets {
    let mut manifest = upper.manifest.clone();
    let scale = manifest.face.px_per_unit;
    let cheek = cheek_w * scale;
    manifest.face = FaceSpec {
        file: "lower-3d.png".into(),
        w: face.width(),
        h: face.height(),
        faceplate_rect_px: [
            cheek,
            0.0,
            face.width() as f32 - cheek,
            face.height() as f32,
        ],
        ..manifest.face
    };
    manifest.lamps.clear();
    manifest.quality_opening = None;
    manifest.controls = fixtures
        .iter()
        .filter(|fixture| {
            matches!(fixture.kind.as_str(), "knob" | "selector" | "toggle")
                && !fixture.sprite.is_empty()
        })
        .map(|fixture| {
            let mut knob = upper
                .manifest
                .controls
                .iter()
                .find(|control| control.sprite == fixture.sprite)
                .or_else(|| {
                    upper
                        .manifest
                        .controls
                        .iter()
                        .find(|control| control.kind == fixture.kind)
                })
                .expect("lower hardware uses an upper atlas")
                .clone();
            knob.param.clone_from(&fixture.param);
            knob.label.clone_from(&fixture.label);
            knob.kind.clone_from(&fixture.kind);
            knob.sprite.clone_from(&fixture.sprite);
            knob.steps = fixture.steps.or(knob.steps);
            if knob.kind == "toggle" {
                knob.across = fixture.across;
                knob.hit_w = Some(fixture.w * scale);
                knob.hit_h = Some(fixture.h * scale);
            }
            if let Some(r) = fixture.hit_r {
                knob.hit_r = Some(r * scale);
            }
            knob.pos = [cheek + fixture.x * scale, fixture.y * scale];
            knob.default = fixture.value;
            knob
        })
        .collect();
    manifest.sprites.retain(|name, _| {
        manifest
            .controls
            .iter()
            .any(|control| control.sprite == *name)
    });
    let atlases: HashMap<_, _> = manifest
        .sprites
        .keys()
        .map(|name| (name.clone(), upper.atlases[name].clone()))
        .collect();
    Panel3DAssets {
        manifest,
        face,
        atlases,
    }
}

fn put(frame: &mut Pixmap, part: &Pixmap, x: i32, y: i32) {
    frame.draw_pixmap(
        x,
        y,
        part.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

#[path = "full_3d/preview.rs"]
mod preview;

#[path = "full_3d/pages.rs"]
mod pages;

#[path = "full_3d/presets.rs"]
mod presets;

#[cfg(target_os = "linux")]
#[path = "full_3d/x11.rs"]
mod x11;

fn main() {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [upper_dir, lower_dir, output] = paths.as_slice() else {
        panic!("usage: render_full_3d <upper pack> <lower build> <output | --window>");
    };
    let mut panel = preview::Preview::load(upper_dir, lower_dir);
    if output == Path::new("--window") {
        #[cfg(target_os = "linux")]
        {
            x11::open(panel).expect("open the CA-72 preview");
            return;
        }
        #[cfg(not(target_os = "linux"))]
        panic!("the native review window is available on Linux");
    }
    std::fs::create_dir_all(output).expect("create output directory");
    panel
        .frame
        .save_png(output.join("ca72-full-3d-4k.png"))
        .expect("save full 4K panel");
    panel
        .frame
        .clone_rect(
            IntRect::from_xywh(
                0,
                panel.top,
                panel.frame.width(),
                panel.lower.frame().height(),
            )
            .expect("nonempty lower panel dimensions"),
        )
        .expect("lower crop")
        .save_png(output.join("ca72-lower-3d-4k.png"))
        .expect("save lower strip");

    assert!(panel.redraw().is_empty());
    let before = panel.frame.clone();
    panel.scene.mode = Some(2);
    panel
        .lower_state
        .set_control(panel.lower.manifest(), "voices", 0.45);
    panel.refresh_readout("voices");
    let damage = panel.redraw();
    assert!(!damage.is_empty() && damage.iter().all(|r| r.y0 >= panel.top));
    assert_ne!(panel.frame.data(), before.data());
    println!(
        "{} × {}. Verified lower knob, tab and readout updates.",
        panel.frame.width(),
        panel.frame.height()
    );

    for (name, target) in [
        ("keys", BarTarget::Keyboard),
        ("lfo", BarTarget::Lfo),
        ("expression", BarTarget::Mpe),
        ("macros", BarTarget::Macro),
    ] {
        if panel.pages.has(name) {
            panel.pages.select(BarTarget::Global, &mut panel.scene.bar);
            panel.pages.select(target, &mut panel.scene.bar);
            let damage = panel.redraw();
            assert!(!damage.is_empty() && damage.iter().all(|r| r.y0 >= panel.top));
            assert!(panel.redraw().is_empty());
            panel
                .frame
                .save_png(output.join(format!("ca72-{name}-4k.png")))
                .expect("save live page proof");
            #[cfg(target_os = "linux")]
            if target == BarTarget::Mpe {
                let p = &panel.lower.manifest().face;
                let i = ca72_panel::controls::index("velocity_curve").expect("curve control");
                let (x, y) = ca72_panel::controls::CONTROLS[i].centre();
                let scale = p.px_per_unit;
                let left = p.faceplate_rect_px[0];
                let x = left + x as f32 * scale;
                let y = panel.top as f32 + (y - PANEL_H) as f32 * scale;
                let before = panel.frame.clone();
                panel.press(x, y);
                panel.motion(Some((x, y - 40.0)), 0.0, -40.0, false);
                panel.release();
                let damage = panel.redraw();
                assert!(!damage.is_empty() && damage.iter().all(|r| r.y0 >= panel.top));
                for (x, y, w, h) in [strip::mpe::readout(4), strip::mpe::RESPONSE] {
                    let glass = IntRect::from_xywh(
                        (left + x as f32 * scale).round() as i32,
                        panel.top + ((y - PANEL_H) as f32 * scale).round() as i32,
                        (w as f32 * scale).round() as u32,
                        (h as f32 * scale).round() as u32,
                    )
                    .expect("nonempty Expression glass");
                    assert_ne!(panel.frame.clone_rect(glass), before.clone_rect(glass));
                }
                assert!(panel.redraw().is_empty());
                println!("Verified an Expression drag updates the knob, readout and response.");
            }
        }
    }

    panel.pages.select(BarTarget::Global, &mut panel.scene.bar);
    if panel.scene.bar.macros {
        panel.pages.select(BarTarget::Macro, &mut panel.scene.bar);
    }
    panel.redraw();
    let closed = panel.frame.clone();
    if panel.set_presets_open(true) {
        assert!(!panel.redraw().is_empty());
        assert!(panel.redraw().is_empty());
        panel
            .frame
            .save_png(output.join("ca72-presets-4k.png"))
            .expect("save live preset browser proof");
        panel.set_presets_open(false);
        assert!(!panel.redraw().is_empty());
        assert_eq!(panel.frame.data(), closed.data());
        assert!(panel.redraw().is_empty());
        println!("Verified opening and closing the modeled preset browser.");
    }
}
