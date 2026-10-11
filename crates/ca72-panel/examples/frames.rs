//! What a frame costs the editor in the worn skin, for each kind of change: `cargo run --release
//! -p ca72-panel --example frames -- [scale ...]` (pixels a drawing unit; 0.44 and 0.87 by
//! default, the window on a 1920 by 1080 screen, and the same on a Retina one). Each is the
//! panel's renderer, the strip's over its foot, the editor's composing of the two
//! (`ca72-plugin`'s `draw` and `compose`) and the frame's conversion to the window's pixels (its
//! `present`, softbuffer's 0RGB), timed together: the median and the slowest of 40.

use std::time::{Duration, Instant};

use ca72_panel::strip::{Field, StripRenderer, StripScene};
use ca72_panel::{CONTROLS, Pixmap, Renderer, Scene, Skin, art};

fn main() {
    let scales: Vec<f64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let scales = if scales.is_empty() {
        vec![0.44, 0.87]
    } else {
        scales
    };
    for s in scales {
        measure(s);
    }
}

fn measure(s: f64) {
    let t = Instant::now();
    let mut panel = Renderer::with_skin(Skin::Worn, s, 1.0);
    let mut strip = StripRenderer::new(s);
    let mut scene = Scene::default();
    for v in &mut scene.values {
        *v = 0.5;
    }
    let mut ss = strip_scene(0);
    let mut now = Instant::now();
    let mut composed: Option<Pixmap> = None;
    let mut window: Vec<u32> = Vec::new();
    let mut frame = |panel: &mut Renderer,
                     strip: &mut StripRenderer,
                     scene: &Scene,
                     ss: &StripScene,
                     now: Instant| {
        let p = panel.render(scene);
        let q = strip.render_at(ss, panel.frame(), panel.damage(), now);
        if !(p || q) {
            return;
        }
        let top = (art::PANEL_H * s).round() as usize;
        match (&mut composed, strip.damage()) {
            (Some(c), Some([_, y0, _, y1])) if !p => {
                rows(c, strip.frame(), top, top + y0 as usize, top + y1 as usize);
            }
            _ => compose(&mut composed, panel, strip.frame(), s),
        }
        if let Some(c) = &composed {
            present(&mut window, c);
        }
    };
    frame(&mut panel, &mut strip, &scene, &ss, now);
    // The drops settled.
    for _ in 0..200 {
        now += Duration::from_millis(16);
        frame(&mut panel, &mut strip, &scene, &ss, now);
    }
    println!(
        "scale {s}: {} by {} pixels, opened in {:.0} ms",
        panel.frame().width(),
        panel.frame().height(),
        ms(t.elapsed())
    );
    let index = |p: &str| CONTROLS.iter().position(|c| c.param == p).expect(p);
    // (`after`: frames drawn first, untimed.)
    let mut times =
        |name: &str, after: usize, step: &mut dyn FnMut(usize, &mut Scene, &mut StripScene)| {
            for _ in 0..after {
                now += Duration::from_millis(16);
                frame(&mut panel, &mut strip, &scene, &ss, now);
            }
            let mut d = Vec::new();
            for i in 0..40 {
                step(i, &mut scene, &mut ss);
                now += Duration::from_millis(16);
                let t = Instant::now();
                frame(&mut panel, &mut strip, &scene, &ss, now);
                d.push(t.elapsed());
            }
            d.sort();
            println!(
                "  {name:<36} median {:5.1} ms, slowest {:5.1} ms",
                ms(d[d.len() / 2]),
                ms(d[d.len() - 1])
            );
        };
    let cutoff = index("cutoff");
    times("CUTOFF turned (the panel)", 0, &mut |i, sc, _| {
        sc.values[cutoff] = 0.3 + 0.01 * i as f64;
    });
    let drive = index("drive");
    times("DRIVE turned (a strip knob)", 0, &mut |i, sc, ss| {
        sc.values[drive] = 0.3 + 0.01 * i as f64;
        ss.readouts[5].0 = format!("{:.1}", 0.3 * i as f64);
    });
    let wheel = index("pitch_wheel");
    times(
        "PITCH wheel turned (the strip's left)",
        0,
        &mut |i, sc, _| {
            sc.values[wheel] = 0.3 + 0.01 * i as f64;
        },
    );
    times("a tab pressed", 0, &mut |i, _, ss| ss.mode = Some(i % 3));
    times("notes played (the drops moving)", 0, &mut |i, _, ss| {
        *ss = strip_scene(if i % 8 < 4 { 0b1111 } else { 0b0011 });
    });
    times("the drops settling", 0, &mut |_, _, _| {});
    times("nothing changed", 100, &mut |_, _, _| {});
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// The editor's composing (`ca72-plugin`'s `compose`, the drawer shut): the panel's frame, the
/// strip's over its foot.
fn compose(composed: &mut Option<Pixmap>, panel: &Renderer, strip: &Pixmap, s: f64) {
    let frame = panel.frame();
    let c = composed.get_or_insert_with(|| frame.clone());
    c.data_mut().copy_from_slice(frame.data());
    let top = (art::PANEL_H * s).round() as usize;
    let h = c.height() as usize;
    rows(c, strip, top, top, h);
}

/// `strip`'s rows put in `c` from its row `from` to `to`, `strip` starting at `top`.
fn rows(c: &mut Pixmap, strip: &Pixmap, top: usize, from: usize, to: usize) {
    let (w, pw) = (c.width() as usize, strip.width() as usize);
    let n = w.min(pw) * 4;
    for y in from..to
        .min(c.height() as usize)
        .min(top + strip.height() as usize)
    {
        let (src, dst) = ((y - top) * pw * 4, y * w * 4);
        c.data_mut()[dst..dst + n].copy_from_slice(&strip.data()[src..src + n]);
    }
}

/// The frame as the window takes it (`ca72-plugin`'s `present`: 0RGB, the alpha dropped).
fn present(window: &mut Vec<u32>, c: &Pixmap) {
    window.resize(c.pixels().len(), 0);
    for (o, p) in window.iter_mut().zip(c.pixels()) {
        *o = u32::from(p.red()) << 16 | u32::from(p.green()) << 8 | u32::from(p.blue());
    }
}

/// POLY, SCATTER, EVEN, AUTO GAIN on, eight voices, those in `sounding` sounding.
fn strip_scene(sounding: u32) -> StripScene {
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
            r("40", true),
        ],
        field: Field::Scatter(
            // EVEN's places for eight voices, each in its side's band at WIDTH 70 %, INNER 40 %.
            [-7.0, 7.0, -3.0, 3.0, -5.0, 5.0, -1.0, 1.0]
                .iter()
                .map(|p: &f64| p / 7.0)
                .enumerate()
                .map(|(i, p)| {
                    (
                        0.7 * p.signum() * (0.4 + 0.6 * p.abs()),
                        sounding >> i & 1 == 1,
                    )
                })
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
