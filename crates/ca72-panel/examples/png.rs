//! Renders the panel to a PNG: `cargo run -p ca72-panel --example png -- out.png [scale]`
//! (pixels a panel unit, 0.5 by default); `CA72_SKIN=worn` in the worn skin; `CA72_MONO=1` the
//! strip in MONO without DOUBLE (the one voice lit at one of VOICES's places); `CA72_LETTERING=
//! large` in the large lettering.
//! `CA72_STANDALONE=1` shows the standalone keyboard's TYPING and PANIC controls and its TEMPO
//! screen (`CA72_TEMPO=96` another tempo).
//! `CA72_ARP=1` drops the arpeggiator's drawer over the lower page (decisions.md R-ARP): ON and
//! UP lit, a sixteenth, two octaves, GATE 60 %, SWING 25 %, the ARP key lit; `CA72_ARP_HOLD=1`
//! HOLD lit too, `CA72_ARP_DIRECTION=4` another DIRECTION (ORDER), `CA72_ARP_OCTAVES=5` other
//! OCTAVES.
//! `CA72_FX=1` the standalone's rail (its FX key, lit) and the effects' drawer over the lower
//! page (decisions.md R-FX-DRAWER): three effects, the third bypassed, the fourth slot empty;
//! `CA72_FX=list` with the effects' list dropped for the fourth; `CA72_FX=crashed` the second
//! given up on; `CA72_FX=safety` a slot turned off after a crash, one after a hang, one
//! loading; `CA72_FX=off` a safe start's slots (decisions.md R-FX-SAFETY); `CA72_FX=rail` the
//! rail alone.
//! `CA72_MPE=1` drops the expression drawer over the lower page (decisions.md R-MPE): PRESSURE
//! 50 % and 1.0 OCT, TIMBRE -30 % and 2.5 OCT (the mock-up's), INPUT reading MPE, the MPE key
//! lit; `CA72_MPE_INPUT=channel` or `none` INPUT so.
//! `CA72_PACK=<folder>` in a skin pack's look (decisions.md R-SKIN-PACKS), what its checks
//! found printed.
//! `CA72_TIP=1` adds CUTOFF's hover tip; `CA72_MENU=settings` the built-in standalone
//! settings' menu (a review fixture, without installed packs). `CA72_PRINT_SVG=<file>` writes
//! the drawing's text as SVG, before rasterisation, for measuring its physical heights.

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
    let lettering = match std::env::var("CA72_LETTERING").as_deref() {
        Ok("large") => ca72_panel::Lettering::Large,
        _ => ca72_panel::Lettering::Standard,
    };
    let mut r = match std::env::var("CA72_PACK") {
        Ok(dir) => match ca72_panel::Look::open(std::path::Path::new(&dir)) {
            Ok(look) => {
                print!("{}", look.report);
                Renderer::with_pack(look, lettering, scale, 1.0)
            }
            Err(report) => {
                eprint!("{report}");
                std::process::exit(1);
            }
        },
        Err(_) => Renderer::with_look(skin, lettering, scale, 1.0),
    };
    let arp = std::env::var("CA72_ARP").is_ok_and(|v| v == "1");
    let fx = std::env::var("CA72_FX").ok();
    let mpe = std::env::var("CA72_MPE").is_ok_and(|v| v == "1");
    let built = t0.elapsed();
    if let Ok(path) = std::env::var("CA72_PRINT_SVG") {
        let fonts = ca72_panel::fonts::Fonts::new();
        let arrangement = r.arrangement();
        let layout = ca72_panel::Layout::measure_in(&fonts, lettering, arrangement);
        let plate = ca72_panel::art::plate();
        let body = format!(
            "{}<g transform='translate({} {})'>{}</g>",
            ca72_panel::art::background(&layout, arrangement),
            plate.origin.0,
            plate.origin.1,
            plate.body
        );
        std::fs::write(
            path,
            ca72_panel::art::document([0.0, 0.0, ca72_panel::W, ca72_panel::H], 3108, 1775, &body),
        )
        .expect("the print's SVG written");
    }
    let mut scene = Scene::default();
    // The plugin's defaults: as the mock-up was drawn; the strip's knobs at what its readouts
    // read. `CA72_ALL=0.5`: every control at one value instead.
    let all: Option<f64> = std::env::var("CA72_ALL").ok().and_then(|s| s.parse().ok());
    for (i, c) in CONTROLS.iter().enumerate() {
        scene.values[i] =
            all.unwrap_or_else(|| strip_knob(c.param).unwrap_or_else(|| default_of(c.param)));
    }
    let mono = std::env::var("CA72_MONO").is_ok_and(|v| v == "1");
    if std::env::var("CA72_STANDALONE").is_ok_and(|v| v == "1") {
        let bpm = std::env::var("CA72_TEMPO").unwrap_or_else(|_| "120".into());
        scene.tempo = Some(format!("{bpm} BPM"));
    }
    if std::env::var("CA72_TIP").is_ok_and(|v| v == "1") {
        scene.tip = Some(("CUTOFF FREQUENCY: 1.2 kHz".into(), 2081.0, 300.0));
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
    strip.set_look(r.look());
    strip.set_under_top(r.top_px());
    scene.bar.arp = arp;
    scene.bar.mpe = mpe;
    if fx.is_some() {
        strip.set_rail(ca72_panel::strip::Rail::Effects);
        scene.bar.fx = true;
    }
    strip.render(&scene, r.frame(), None);
    eprintln!("the strip {:?}", t3.elapsed());
    let mut all = r.frame().clone();
    let top = r.strip_row() as usize;
    let w = all.width() as usize;
    let sf = strip.frame();
    let rows = (sf.height() as usize).min(all.height() as usize - top);
    for y in 0..rows {
        let n = w.min(sf.width() as usize) * 4;
        let (from, to) = (y * sf.width() as usize * 4, (top + y) * w * 4);
        all.data_mut()[to..to + n].copy_from_slice(&sf.data()[from..from + n]);
    }
    if arp {
        drop_arp(&mut all, &mut r, scale, skin, lettering);
    }
    if mpe {
        drop_mpe(&mut all, &mut r, scale, skin, lettering);
    }
    if let Some(fx) = fx.filter(|v| v != "rail") {
        drop_fx(&mut all, scale, r.top_px(), skin, lettering, &fx);
    }
    // `CA72_DRAWER=1`: the presets' drawer open over the strip, as the editor shows it; `edit`,
    // a search typed and the preset being renamed; `midi`, MIDI Learn's list.
    let drawer_shown = std::env::var("CA72_DRAWER").unwrap_or_default();
    if ["1", "edit", "midi"].contains(&drawer_shown.as_str()) {
        use ca72_panel::presets::{
            Current, DRAWER_TOP, DrawerRenderer, DrawerScene, DrawerTarget, Edit, Field, FieldId,
            MidiList, MidiRow, Row,
        };
        let mut drawer = DrawerRenderer::new(scale);
        drawer.set_look(r.look());
        let presets = [
            "Brass Tutti",
            "Deep Mod Bass",
            "Glass Lead",
            "Night Pad",
            "Pluck Ladder",
        ];
        let mut scene = DrawerScene {
            chips: ["bass", "brass", "lead", "pad", "pluck"]
                .iter()
                .map(|t| ((*t).to_owned(), *t == "lead"))
                .collect(),
            rows: (0..18)
                .map(|i| Row {
                    name: format!("{} {}", presets[i % presets.len()], i / presets.len() + 1),
                    tags: "lead bright".into(),
                    origin: (i == 3).then_some("EDITED"),
                    favorite: i % 4 == 0,
                    current: i == 0,
                })
                .collect(),
            current: Some(Current {
                name: "Brass Tutti 1".into(),
                origin: None,
                tags: "brass".into(),
                favorite: false,
            }),
            total: 18,
            ..DrawerScene::default()
        };
        if drawer_shown == "edit" {
            scene.search = Field {
                text: "bra".into(),
                caret: 3,
            };
            scene.editing = Some(Edit::Rename);
            scene.edit = Field {
                text: "Brass Tutti 1".into(),
                caret: 5,
            };
            scene.focus = Some(FieldId::Edit);
            scene.hover = Some(DrawerTarget::Row(2));
        }
        if drawer_shown == "midi" {
            let row = |name: &str, assignment: &str| MidiRow {
                name: name.into(),
                assignment: assignment.into(),
                waiting: false,
            };
            scene.midi = Some(MidiList {
                rows: vec![
                    row("TUNE", "CC 21"),
                    row("GLIDE", ""),
                    row("CUTOFF", "CC 74"),
                    row("EMPHASIS", "CC 71"),
                ],
                chosen: 1,
                keys: "ENTER: LEARN \u{b7} DELETE: REMOVE".into(),
                reserved: "CC 0, 32, 120 TO 127 ARE NOT LEARNED".into(),
                status: "CHOOSE A CONTROL, THEN MOVE A CONTROLLER".into(),
                ..MidiList::default()
            });
        }
        drawer.render(&scene);
        let d = drawer.frame();
        let at = (DRAWER_TOP * scale).round() as usize - r.top_px().max(0) as usize
            + (-r.top_px()).max(0) as usize;
        let w = all.width() as usize;
        let rows = (d.height() as usize).min(all.height() as usize - at);
        for y in 0..rows {
            let n = w.min(d.width() as usize) * 4;
            let (from, to) = (y * d.width() as usize * 4, (at + y) * w * 4);
            all.data_mut()[to..to + n].copy_from_slice(&d.data()[from..from + n]);
        }
    }
    if std::env::var("CA72_MENU").as_deref() == Ok("settings") {
        settings_menu(&mut all, &r);
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

/// The built-in standalone's menu, as `learning::open_settings_menu` lays it out in 0.3.0.
/// Drawn last, over the strip too, as the editor composes its overlays.
fn settings_menu(frame: &mut ca72_panel::Pixmap, r: &Renderer) {
    use ca72_panel::art::{H, W};
    use ca72_panel::learn::{self, Menu};
    let labels = [
        "LOOK",
        "REALISTIC",
        "CLASSIC",
        "LETTERING",
        "STANDARD",
        "LARGE",
        "FREQUENCY DIALS",
        "SEMITONES",
        "HARDWARE",
        "WINDOW",
        "SIZE…",
        "WHAT'S NEW…",
        "MOD WHEEL SOURCE",
        "CC 1…",
        "STANDALONE",
        "AUDIO & MIDI…",
        "AGENTS",
        "ALLOW",
        "OFF",
    ];
    let mut menu = Menu {
        x: 0.0,
        y: 0.0,
        size: r.text_size() * 1.15,
        title: "SETTINGS · CA-72 0.3.0".into(),
        items: labels.iter().map(|t| ((*t).into(), true)).collect(),
        heads: vec![0, 3, 6, 9, 12, 14, 16],
        marked: vec![
            1,
            if r.lettering() == ca72_panel::Lettering::Large {
                5
            } else {
                4
            },
            7,
            17,
        ],
        hover: None,
    };
    let (w, _) = menu.extent(r.fonts());
    menu = menu.placed(r.fonts(), 3055.0 - w, 1017.0, [0.0, 0.0, W, H]);
    let layer = learn::menu(r.fonts(), &menu);
    let body = format!(
        "<g transform='translate({} {})'>{}</g>",
        menu.x, menu.y, layer.body
    );
    let doc = ca72_panel::art::document(
        [0.0, r.top(), W, H - r.top()],
        frame.width(),
        frame.height(),
        &body,
    );
    let options = resvg::usvg::Options {
        fontdb: r.fonts().database(),
        font_family: ca72_panel::fonts::FAMILY.into(),
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_str(&doc, &options).expect("the review menu parses");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut frame.as_mut(),
    );
    eprintln!("settings items {} logical px", menu.size * r.scale());
}

/// The plugin's default for a parameter, normalized.
fn default_of(param: &str) -> f64 {
    match param {
        "tune" | "osc2_frequency" | "osc3_frequency" | "cutoff" | "pitch_wheel" => 0.5,
        "pressure_mod" | "pressure_filter" | "timbre_mod" | "timbre_filter" => 0.5,
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

/// The arpeggiator's drawer over `all` (the editor's `compose`): its plate from its top, its
/// shadow laid over the page under its foot.
fn drop_arp(
    all: &mut ca72_panel::Pixmap,
    r: &mut Renderer,
    scale: f64,
    skin: Skin,
    lettering: ca72_panel::Lettering,
) {
    use ca72_panel::strip::arp::{self, ArpRenderer, ArpScene};
    let t = std::time::Instant::now();
    let mut drawer = ArpRenderer::new(scale, skin, lettering);
    let r2 = |s: &str| (s.to_owned(), true);
    let var = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<usize>().ok());
    let octaves = var("CA72_ARP_OCTAVES").unwrap_or(2).clamp(1, 5);
    drawer.render(
        &ArpScene {
            on: true,
            latch: std::env::var("CA72_ARP_HOLD").is_ok_and(|v| v == "1"),
            direction: var("CA72_ARP_DIRECTION").unwrap_or(0).min(4),
            readouts: [r2("16"), r2(&octaves.to_string()), r2("60"), r2("25")],
            knobs: [9.0 / 12.0, (octaves - 1) as f64 / 4.0, 0.6, 0.25],
            hover: None,
        },
        r,
    );
    eprintln!("the arpeggiator's drawer {:?}", t.elapsed());
    let (w, f) = (all.width() as usize, drawer.frame());
    // (The frame begins at the panel's top: a skin pack's taller panel's above the drawing's.)
    let top = (arp::top_px(scale) - r.top_px()) as usize;
    let (x0, x1) = (
        (arp::LEFT * scale).round() as usize,
        (arp::RIGHT * scale).round() as usize,
    );
    let depth = (arp::DEPTH * scale).round() as usize;
    for y in 0..depth.min(f.height() as usize) {
        let (from, to) = ((y * f.width() as usize + x0) * 4, ((top + y) * w + x0) * 4);
        let n = (x1 - x0) * 4;
        all.data_mut()[to..to + n].copy_from_slice(&f.data()[from..from + n]);
    }
    if let Some(shadow) = drawer.shadow() {
        all.draw_pixmap(
            0,
            (top + depth) as i32,
            shadow.as_ref(),
            &resvg::tiny_skia::PixmapPaint::default(),
            resvg::tiny_skia::Transform::identity(),
            None,
        );
    }
}

/// The expression drawer over `all` (the editor's `compose`), as the arpeggiator's.
fn drop_mpe(
    all: &mut ca72_panel::Pixmap,
    r: &mut Renderer,
    scale: f64,
    skin: Skin,
    lettering: ca72_panel::Lettering,
) {
    use ca72_panel::strip::mpe::{self, Input, MpeRenderer, MpeScene};
    let t = std::time::Instant::now();
    let mut drawer = MpeRenderer::new(scale, skin, lettering);
    let r2 = |s: &str| (s.to_owned(), true);
    let input = match std::env::var("CA72_MPE_INPUT").as_deref() {
        Ok("channel") => Input::Channel,
        Ok("none") => Input::None,
        _ => Input::Mpe,
    };
    drawer.render(
        &MpeScene {
            input,
            readouts: [
                r2(&std::env::var("CA72_MPE_P").unwrap_or_else(|_| "50".into())),
                r2("1.0"),
                r2("-30"),
                r2("2.5"),
            ],
            knobs: [0.75, 5.0 / 8.0, 0.35, 6.5 / 8.0],
        },
        r,
    );
    eprintln!("the expression drawer {:?}", t.elapsed());
    let (w, f) = (all.width() as usize, drawer.frame());
    let top = (mpe::top_px(scale) - r.top_px()) as usize;
    let (x0, x1) = (
        (mpe::LEFT * scale).round() as usize,
        (mpe::RIGHT * scale).round() as usize,
    );
    let depth = (mpe::DEPTH * scale).round() as usize;
    for y in 0..depth.min(f.height() as usize) {
        let (from, to) = ((y * f.width() as usize + x0) * 4, ((top + y) * w + x0) * 4);
        let n = (x1 - x0) * 4;
        all.data_mut()[to..to + n].copy_from_slice(&f.data()[from..from + n]);
    }
    if let Some(shadow) = drawer.shadow() {
        all.draw_pixmap(
            0,
            (top + depth) as i32,
            shadow.as_ref(),
            &resvg::tiny_skia::PixmapPaint::default(),
            resvg::tiny_skia::Transform::identity(),
            None,
        );
    }
}

/// The effects' drawer over `all` (the editor's `compose`): its plate from its top, what hangs
/// under its foot (its shadow, the list) laid over the page.
fn drop_fx(
    all: &mut ca72_panel::Pixmap,
    scale: f64,
    origin: i32,
    skin: Skin,
    lettering: ca72_panel::Lettering,
    how: &str,
) {
    use ca72_panel::strip::fx::{self, FxList, FxRenderer, FxRow, FxScene, FxSlot, FxWord};
    let t = std::time::Instant::now();
    let mut drawer = FxRenderer::new(scale, skin, lettering);
    let slot = |text: &str, lit: bool| FxSlot {
        text: text.into(),
        dim: false,
        lit,
        word: None,
    };
    let mut scene = FxScene {
        slots: [
            slot("Chorus", true),
            slot("Hall Reverb", true),
            slot("Tape Delay", false),
            slot("+ ADD EFFECT", false),
        ],
        ..FxScene::default()
    };
    if how == "crashed" {
        scene.slots[1] = FxSlot {
            text: "Hall Reverb".into(),
            dim: true,
            lit: false,
            word: Some(FxWord::Crashed),
        };
    }
    // The safety states (decisions.md R-FX-SAFETY): one off for a safe start, one turned off
    // after it crashed, one after it stopped responding, one loading.
    if how == "safety" || how == "off" {
        let word = |text: &str, word: FxWord| FxSlot {
            text: text.into(),
            dim: true,
            lit: false,
            word: Some(word),
        };
        scene.slots = if how == "off" {
            [
                word("Chorus", FxWord::Off),
                word("Hall Reverb", FxWord::Off),
                word("Tape Delay", FxWord::Off),
                slot("+ ADD EFFECT", false),
            ]
        } else {
            [
                slot("Chorus", true),
                word("Hall Reverb", FxWord::Crashed),
                word("Tape Delay", FxWord::NotResponding),
                word("Shimmer", FxWord::Loading),
            ]
        };
    }
    if how == "list" {
        let row = |name: &str, maker: &str, format: &str| FxRow {
            name: name.into(),
            maker: maker.into(),
            format: format.into(),
            action: false,
        };
        let mut rows = vec![
            row("Dragonfly Early Reflections", "Michael Willis", "CLAP"),
            row("Dragonfly Hall Reverb", "Michael Willis", "CLAP"),
            row("Dragonfly Plate Reverb", "Michael Willis", "CLAP"),
            row("Dragonfly Room Reverb", "Michael Willis", "CLAP"),
            row("Dragonfly Early Reflections", "Michael Willis", "VST3"),
            row("Dragonfly Hall Reverb", "Michael Willis", "VST3"),
            row("Gain", "DAW-v3 tests", "CLAP"),
            row("Gain", "DAW-v3 tests", "VST3"),
        ];
        rows.push(FxRow {
            name: "SCAN AGAIN".into(),
            action: true,
            ..FxRow::default()
        });
        scene.list = Some(FxList {
            slot: 3,
            title: "ADD TO SLOT 4".into(),
            note: "8 EFFECTS".into(),
            rows,
            first: 0,
            hover: Some(1),
        });
    }
    drawer.render(&scene);
    eprintln!("the effects' drawer {:?}", t.elapsed());
    let (w, f) = (all.width() as usize, drawer.frame());
    let top = (fx::top_px(scale) - origin) as usize;
    let (x0, x1) = (
        (fx::LEFT * scale).round() as usize,
        (fx::RIGHT * scale).round() as usize,
    );
    let depth = (fx::DEPTH * scale).round() as usize;
    for y in 0..depth.min(f.height() as usize) {
        let (from, to) = ((y * f.width() as usize + x0) * 4, ((top + y) * w + x0) * 4);
        let n = (x1 - x0) * 4;
        all.data_mut()[to..to + n].copy_from_slice(&f.data()[from..from + n]);
    }
    for (p, x, y) in drawer.under() {
        all.draw_pixmap(
            x,
            top as i32 + y,
            p.as_ref(),
            &resvg::tiny_skia::PixmapPaint::default(),
            resvg::tiny_skia::Transform::identity(),
            None,
        );
    }
}
