//! What a frame costs the editor in the worn skin, for each kind of change: `cargo run --release
//! -p ca72-panel --example frames -- [scale ...]` (pixels a drawing unit; 0.44 and 0.87 by
//! default, the window on a 1920 by 1080 screen, and the same on a Retina one). Each is the
//! panel's renderer, the strip's over its foot, the editor's composing of the two
//! (`ca72-plugin`'s `draw` and `compose`) and the frame's conversion to the window's pixels (its
//! `present`, softbuffer's 0RGB), timed together: the median and the slowest of 40. Then the
//! arpeggiator's drawer (decisions.md R-ARP) and the standalone's effects' drawer (R-FX-DRAWER);
//! `CA72_RAIL=effects` the strip with the standalone's rail (its FX key).

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
    if std::env::var("CA72_RAIL").is_ok_and(|v| v == "effects") {
        strip.set_rail(ca72_panel::strip::Rail::Effects);
    }
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
    arp(s, &mut panel);
    fx(s, &panel);
}

/// The effects' drawer (decisions.md R-FX-DRAWER): its first drop (made as the editor opens, on
/// its thread: joined at once, the worst case, and later), a tab let go, the pointer over a name,
/// a slot carried over another, its list dropped, pointed at and scrolled, and nothing changed;
/// each its render and its plate and what hangs under it put in the window's frame.
fn fx(s: f64, panel: &Renderer) {
    use ca72_panel::strip::fx::{self, FxList, FxRenderer, FxRow, FxScene, FxSlot, FxTarget};
    let slot = |t: &str, lit: bool| FxSlot {
        text: t.into(),
        dim: false,
        lit,
        word: None,
    };
    let mut scene = FxScene {
        slots: [
            slot("Chorus", true),
            slot("Hall Reverb", true),
            slot("Tape Delay", true),
            slot("+ ADD EFFECT", false),
        ],
        ..FxScene::default()
    };
    let mut composed = panel.frame().clone();
    let put = |d: &FxRenderer, c: &mut Pixmap| {
        let top = fx::top_px(s) as usize;
        let f = d.frame();
        let (x0, x1) = (
            (fx::LEFT * s).round() as usize,
            (fx::RIGHT * s).round() as usize,
        );
        let w = c.width() as usize;
        for y in 0..f.height() as usize {
            let (from, to) = ((y * f.width() as usize + x0) * 4, ((top + y) * w + x0) * 4);
            let n = (x1 - x0) * 4;
            c.data_mut()[to..to + n].copy_from_slice(&f.data()[from..from + n]);
        }
        for (p, x, y) in d.under() {
            c.draw_pixmap(
                x,
                top as i32 + y,
                p.as_ref(),
                &resvg::tiny_skia::PixmapPaint::default(),
                resvg::tiny_skia::Transform::identity(),
                None,
            );
        }
    };
    let t = Instant::now();
    let mut d = FxRenderer::new(s, Skin::Worn, ca72_panel::Lettering::Standard);
    let made = t.elapsed();
    d.render(&scene);
    put(&d, &mut composed);
    println!(
        "  {:<36} {:5.1} ms (its plate made on its thread, waited for)",
        "the effects' first drop, at once",
        ms(t.elapsed())
    );
    println!(
        "  {:<36} {:5.2} ms (on the editor's thread as it opens)",
        "the effects' renderer made",
        ms(made)
    );
    let mut d = FxRenderer::new(s, Skin::Worn, ca72_panel::Lettering::Standard);
    std::thread::sleep(Duration::from_millis(1500));
    let t = Instant::now();
    d.render(&scene);
    put(&d, &mut composed);
    println!(
        "  {:<36} {:5.1} ms (its plate made already)",
        "the effects' first drop, later",
        ms(t.elapsed())
    );
    let mut times = |name: &str, step: &mut dyn FnMut(usize, &mut FxScene)| {
        let mut v = Vec::new();
        for i in 0..40 {
            step(i, &mut scene);
            let t = Instant::now();
            if d.render(&scene) {
                put(&d, &mut composed);
            }
            v.push(t.elapsed());
        }
        v.sort();
        println!(
            "  {name:<36} median {:5.1} ms, slowest {:5.1} ms",
            ms(v[v.len() / 2]),
            ms(v[v.len() - 1])
        );
    };
    times("an effects' tab pressed", &mut |i, sc| {
        sc.slots[1].lit = i % 2 == 0;
    });
    times("the pointer over an effect's name", &mut |i, sc| {
        sc.hover = Some(FxTarget::Name(i % 3));
    });
    times("an effect carried over another", &mut |i, sc| {
        sc.hover = None;
        sc.drag = Some((0, Some(1 + i % 2)));
    });
    let rows: Vec<FxRow> = (0..30)
        .map(|i| FxRow {
            name: format!("Effect number {i}"),
            maker: "A Maker".into(),
            format: if i % 2 == 0 { "CLAP" } else { "VST3" }.into(),
            action: false,
        })
        .collect();
    let list = FxList {
        slot: 3,
        title: "ADD TO SLOT 4".into(),
        note: "30 EFFECTS".into(),
        rows,
        first: 0,
        hover: None,
    };
    times("the effects' list dropped", &mut |i, sc| {
        sc.drag = None;
        sc.list = (i % 2 == 0).then(|| list.clone());
    });
    times("the effects' list pointed at", &mut |i, sc| {
        let mut l = list.clone();
        l.hover = Some(i % 11);
        sc.list = Some(l);
    });
    times("the effects' list scrolled", &mut |i, sc| {
        let mut l = list.clone();
        l.first = i % 19;
        sc.list = Some(l);
    });
    times("the effects open, nothing changed", &mut |_, _| {});
}

/// The arpeggiator's drawer (decisions.md R-ARP): its first drop (made as the editor opens: on
/// its thread, joined at once here, the worst case; and joined after the thread is done, as when
/// the drawer is first opened a moment after the editor), a knob turned in it, a tab lit, and
/// nothing changed; each its render and its plate put in the window's frame.
fn arp(s: f64, panel: &mut Renderer) {
    use ca72_panel::strip::arp::{self, ArpRenderer, ArpScene};
    let r = |t: &str| (t.to_owned(), true);
    let mut scene = ArpScene {
        on: true,
        readouts: [r("16"), r("2"), r("60"), r("25")],
        knobs: [0.75, 1.0 / 3.0, 0.6, 0.25],
        ..ArpScene::default()
    };
    let mut composed = panel.frame().clone();
    let put = |d: &ArpRenderer, c: &mut Pixmap| {
        let top = arp::top_px(s) as usize;
        let f = d.frame();
        let (x0, x1) = (
            (arp::LEFT * s).round() as usize,
            (arp::RIGHT * s).round() as usize,
        );
        let w = c.width() as usize;
        for y in 0..f.height() as usize {
            let (from, to) = ((y * f.width() as usize + x0) * 4, ((top + y) * w + x0) * 4);
            let n = (x1 - x0) * 4;
            c.data_mut()[to..to + n].copy_from_slice(&f.data()[from..from + n]);
        }
    };
    let t = Instant::now();
    let mut d = ArpRenderer::new(s, Skin::Worn, ca72_panel::Lettering::Standard);
    let made = t.elapsed();
    d.render(&scene, panel);
    put(&d, &mut composed);
    println!(
        "  {:<36} {:5.1} ms (its plate made on its thread, waited for)",
        "the drawer's first drop, at once",
        ms(t.elapsed())
    );
    println!(
        "  {:<36} {:5.2} ms (on the editor's thread as it opens)",
        "the drawer's renderer made",
        ms(made)
    );
    let mut d = ArpRenderer::new(s, Skin::Worn, ca72_panel::Lettering::Standard);
    std::thread::sleep(Duration::from_millis(1500));
    let t = Instant::now();
    d.render(&scene, panel);
    put(&d, &mut composed);
    println!(
        "  {:<36} {:5.1} ms (its plate made already)",
        "the drawer's first drop, later",
        ms(t.elapsed())
    );
    let mut times = |name: &str, step: &mut dyn FnMut(usize, &mut ArpScene)| {
        let mut v = Vec::new();
        for i in 0..40 {
            step(i, &mut scene);
            let t = Instant::now();
            if d.render(&scene, panel) {
                put(&d, &mut composed);
            }
            v.push(t.elapsed());
        }
        v.sort();
        println!(
            "  {name:<36} median {:5.1} ms, slowest {:5.1} ms",
            ms(v[v.len() / 2]),
            ms(v[v.len() - 1])
        );
    };
    times("a drawer knob turned (GATE)", &mut |i, sc| {
        sc.knobs[2] = 0.3 + 0.01 * i as f64;
        sc.readouts[2].0 = format!("{}", 30 + i);
    });
    times("a drawer tab pressed (DIRECTION)", &mut |i, sc| {
        sc.direction = i % 4
    });
    times("the drawer open, nothing changed", &mut |_, _| {});
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
            panel: true,
            name: "Brass Tutti".into(),
            found: true,
            favorite: true,
            ..Default::default()
        },
        ..StripScene::default()
    }
}
