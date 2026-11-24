//! Compose the 4K Cycles upper panel and lower extension, with live strip artwork.
//!
//! cargo run --release -p ca72-panel --example render_full_3d -- <upper pack> <lower build> <output>

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use ca72_panel::art::PANEL_H;
use ca72_panel::panel3d::{FaceSpec, Panel3DAssets, Panel3DRenderer, Panel3DState};
use ca72_panel::presets::BarScene;
use ca72_panel::strip::{self, Field, StripRenderer, StripScene};
use resvg::tiny_skia::{IntRect, Pixmap, PixmapPaint, Transform};
use serde::Deserialize;

#[derive(Deserialize)]
struct LowerLayout {
    cheek_w: f32,
    controls: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    kind: String,
    x: f32,
    y: f32,
    #[serde(default)]
    param: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    sprite: String,
    #[serde(default)]
    value: f32,
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

fn lower_assets(upper: &Panel3DAssets, build: &Path, layout: LowerLayout) -> Panel3DAssets {
    let face = Pixmap::load_png(build.join("lower-3d.png")).expect("load lower Cycles render");
    let mut manifest = upper.manifest.clone();
    let scale = manifest.face.px_per_unit;
    let cheek = layout.cheek_w * scale;
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
    manifest.controls = layout
        .controls
        .into_iter()
        .filter(|fixture| fixture.kind == "knob")
        .map(|fixture| {
            let mut knob = upper
                .manifest
                .controls
                .iter()
                .find(|control| control.kind == "knob" && control.sprite == fixture.sprite)
                .expect("lower knob uses an upper knob atlas")
                .clone();
            knob.param = fixture.param;
            knob.label = fixture.label;
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

fn main() {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [upper_dir, lower_dir, output] = paths.as_slice() else {
        panic!("usage: render_full_3d <upper pack> <lower build> <output>");
    };
    std::fs::create_dir_all(output).expect("create output directory");
    let upper = Panel3DAssets::load_from_dir(upper_dir).expect("load upper Cycles pack");
    let layout: LowerLayout = serde_json::from_slice(
        &std::fs::read(lower_dir.join("lower.json")).expect("read lower layout"),
    )
    .expect("parse lower layout");
    let scale = f64::from(upper.manifest.face.px_per_unit);
    let top_h = upper.manifest.face.faceplate_rect_px[3].round() as u32;
    let lower = lower_assets(&upper, lower_dir, layout);
    let left = lower.manifest.face.faceplate_rect_px[0].round() as i32;
    let lower_state = Panel3DState::from_defaults(&lower.manifest);
    assert_eq!(lower_state.values.len(), strip::KNOBS.len());
    let mut lower = Panel3DRenderer::new(lower);
    lower.render_dirty(&lower_state);

    let tab = Pixmap::load_png(lower_dir.join("lower-tab-lit.png")).expect("load lit 3D tab");
    let mut strip = StripRenderer::with_cycles_hardware(scale, tab);
    strip.set_under_top((PANEL_H * scale).round() as i32);
    let (w, h) = strip::strip_size(scale);
    assert_eq!(h, lower.frame().height());
    let under = lower
        .frame()
        .clone_rect(IntRect::from_xywh(left, 0, w, h).expect("strip bounds"))
        .expect("strip inside the cheeks");
    let scene = demo_strip();
    let now = Instant::now();
    assert!(strip.render_at(&scene, &under, None, now));
    let shown = strip.frame().clone();

    let state = upper_state(&upper);
    let mut upper = Panel3DRenderer::new(upper);
    upper.render_dirty(&state);
    let w = upper.frame().width();
    assert_eq!(w, lower.frame().width());
    let mut full = Pixmap::new(w, top_h + h).expect("full 4K-wide panel");
    put(&mut full, upper.frame(), 0, 0);
    put(&mut full, lower.frame(), 0, top_h as i32);
    put(&mut full, &shown, left, top_h as i32);
    full.save_png(output.join("ca72-full-3d-4k.png"))
        .expect("save full 4K panel");
    full.clone_rect(IntRect::from_xywh(0, top_h as i32, w, h).expect("lower crop"))
        .expect("full lower strip")
        .save_png(output.join("ca72-lower-3d-4k.png"))
        .expect("save lower strip");

    // Exercise an actual tab and readout update over the modeled strip.
    assert!(!strip.render_at(&scene, &under, None, now));
    let mut changed = scene;
    changed.mode = Some(2);
    changed.readouts[0] = ("5".into(), true);
    assert!(strip.render_at(&changed, &under, None, now));
    assert_ne!(strip.frame().data(), shown.data());
    let patch = strip.damage().expect("tab/readout damage");
    assert!(patch[0] > 0 && patch[2] < strip.frame().width() as i32);
    let mut changed_knob = lower_state;
    changed_knob.set_control(lower.manifest(), "voices", 0.45);
    assert!(!lower.render_dirty(&changed_knob).is_empty());
    println!(
        "{} × {}. Verified lower knob, tab and readout updates.",
        full.width(),
        full.height()
    );
}
