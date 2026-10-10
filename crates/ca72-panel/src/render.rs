//! Rendering the panel: the background once for a scale, each moving part's layer again only
//! when its art changes, and the frame composed of them; in either skin (`crate::skin`), the
//! worn one's pictures turned and moved in the drawn parts' places.

use std::fmt;

use resvg::tiny_skia::{
    Color, FillRule, FilterQuality, Paint, PathBuilder, Pattern, Pixmap, PixmapPaint, Rect,
    SpreadMode, Transform,
};
use resvg::usvg;

use crate::art::{self, H, Layer, Layout, W};
use crate::controls::{CONTROLS, Kind, Mark, feedback_silent};
use crate::fonts::{FAMILY, Fonts};
use crate::learn::{self, Menu, Note};
use crate::skin::{Part, Pictures, Skin};

/// What the panel shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    /// Each control's value, normalized, by its index in [`CONTROLS`].
    pub values: [f64; CONTROLS.len()],
    /// A MIDI keyboard's pitch bend (0 fully down, 0.5 centred, 1 fully up) and its
    /// modulation wheel (0..1): the panel's wheels are drawn where they take them.
    pub midi: (f64, f64),
    /// On (playing), or bypassed.
    pub power: bool,
    /// The OVERLOAD lamp's level, 0 dark to 1 fully lit.
    pub overload: f64,
    /// ULTRA's lamp (decisions.md R-ULTRA): how far it has gone, 0 shut (or dark) to 1 up and
    /// lit, and whether the shutter is off (the light only).
    pub ultra: f64,
    pub ultra_still: bool,
    /// A tip: its text, centred above (x, y) in the drawing.
    pub tip: Option<(String, f64, f64)>,
    /// MIDI Learn (decisions.md R34): the control being learned, by its index in [`CONTROLS`]
    /// (ringed), a note over the panel saying what it does, and a control's menu.
    pub learning: Option<usize>,
    pub note: Option<Note>,
    pub menu: Option<Menu>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            values: [0.0; CONTROLS.len()],
            midi: (0.5, 0.0),
            power: true,
            overload: 0.0,
            ultra: 0.0,
            ultra_still: false,
            tip: None,
            learning: None,
            note: None,
            menu: None,
        }
    }
}

/// A layer as last rendered: its art, and its pixels at their place in the frame.
#[derive(Default)]
struct Slot {
    layer: Option<Layer>,
    pixels: Option<(i32, i32, Pixmap)>,
}

/// The panel's renderer at a scale.
pub struct Renderer {
    fonts: Fonts,
    layout: Layout,
    options: usvg::Options<'static>,
    /// Pixels a panel unit; pixels a logical pixel (for the tip, sized for reading).
    scale: f64,
    ui: f64,
    background: Pixmap,
    frame: Pixmap,
    slots: Vec<Slot>,
    /// The scene the frame shows.
    shown: Option<Scene>,
    /// The pixels the last render changed: left, top, right, bottom (the last two past the
    /// change).
    damage: Option<[i32; 4]>,
    skin: Skin,
    /// The worn skin's pictures, once it has been shown.
    pictures: Option<Pictures>,
}

impl fmt::Debug for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Renderer")
            .field("scale", &self.scale)
            .field("size", &self.size())
            .finish()
    }
}

/// The frame's size at `scale` pixels a panel unit.
pub fn size_at(scale: f64) -> (u32, u32) {
    (
        (W * scale).ceil().max(1.0) as u32,
        (H * scale).ceil().max(1.0) as u32,
    )
}

/// The tip's lettering, in logical pixels.
const TIP_TEXT: f64 = 12.0;

impl Renderer {
    /// A renderer drawing `scale` pixels a panel unit, `ui` pixels a logical pixel, in the
    /// drawn skin.
    pub fn new(scale: f64, ui: f64) -> Self {
        Self::with_skin(Skin::Drawn, scale, ui)
    }

    /// A renderer drawing the panel in `skin`.
    pub fn with_skin(skin: Skin, scale: f64, ui: f64) -> Self {
        let fonts = Fonts::new();
        let layout = Layout::measure(&fonts);
        let options = usvg::Options {
            fontdb: fonts.database(),
            font_family: FAMILY.into(),
            ..usvg::Options::default()
        };
        let (w, h) = size_at(scale);
        let empty = || Pixmap::new(w, h).expect("a frame of at least a pixel");
        let mut r = Self {
            fonts,
            layout,
            options,
            scale,
            ui,
            background: empty(),
            frame: empty(),
            slots: Vec::new(),
            shown: None,
            damage: None,
            skin,
            pictures: None,
        };
        r.draw_background();
        r
    }

    /// The skin the panel is drawn in.
    pub fn skin(&self) -> Skin {
        self.skin
    }

    /// Redraws everything at a new scale.
    pub fn rescale(&mut self, scale: f64, ui: f64) {
        if scale == self.scale && ui == self.ui {
            return;
        }
        let (w, h) = size_at(scale);
        self.scale = scale;
        self.ui = ui;
        self.background = Pixmap::new(w, h).expect("a frame of at least a pixel");
        self.frame = self.background.clone();
        self.slots.clear();
        self.shown = None;
        if let Some(p) = &mut self.pictures {
            p.forget_scale();
        }
        self.draw_background();
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn size(&self) -> (u32, u32) {
        (self.frame.width(), self.frame.height())
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// The frame last composed (premultiplied RGBA, opaque).
    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    fn draw_background(&mut self) {
        if self.skin == Skin::Worn {
            self.draw_worn_background();
            return;
        }
        let (w, h) = (self.background.width(), self.background.height());
        let view = [
            0.0,
            0.0,
            f64::from(w) / self.scale,
            f64::from(h) / self.scale,
        ];
        let doc = art::document(view, w, h, &art::background(&self.layout));
        self.background
            .fill(Color::from_rgba8(0x3b, 0x22, 0x13, 0xff));
        if let Ok(tree) = usvg::Tree::from_str(&doc, &self.options) {
            resvg::render(&tree, Transform::identity(), &mut self.background.as_mut());
        }
        self.frame = self.background.clone();
    }

    /// The worn skin's background: walnut, the faces' textured black, the shadows and the
    /// trim, the print over them, the screws and jacks, and the lamp's falloff over all.
    fn draw_worn_background(&mut self) {
        let s = self.scale;
        let (w, h) = (self.background.width(), self.background.height());
        let view = [0.0, 0.0, f64::from(w) / s, f64::from(h) / s];
        self.background
            .fill(Color::from_rgba8(0x1d, 0x1b, 0x1a, 0xff));
        let pictures = self.pictures.get_or_insert_with(Pictures::load);
        let bg = &mut self.background;
        // The wood: its grain along the top strip and the strip's rail; each board its own band
        // of the picture, drawn out along the grain.
        let wood = &pictures.walnut;
        let along = (art::W * s) as f32 / wood.width() as f32;
        let across = along / 2.2;
        for (y, hh, band) in [
            (0.0, art::TOP, 40.0),
            (art::PANEL_H, crate::strip::RAIL, 260.0),
        ] {
            fill(
                bg,
                wood,
                (0.0, y * s, art::W * s, hh * s),
                None,
                Transform::from_row(along, 0.0, 0.0, across, 0.0, (y * s) as f32 - band * across),
            );
        }
        // The faces: the picture 640 units across, mirrored tile to tile.
        let face = &pictures.face;
        let k = (640.0 * s) as f32 / face.width() as f32;
        let at = |x: f64, y: f64| {
            Transform::from_scale(k, k).post_translate((x * s) as f32, (y * s) as f32)
        };
        fill(
            bg,
            face,
            (art::COL * s, art::TOP * s, art::PW * s, art::PH * s),
            None,
            at(art::COL, art::TOP),
        );
        let strip_top = art::PANEL_H + crate::strip::RAIL;
        fill(
            bg,
            face,
            (0.0, strip_top * s, art::W * s, (art::H - strip_top) * s),
            None,
            at(320.0, strip_top),
        );
        let options = &self.options;
        let svg_over = |frame: &mut Pixmap, body: &str| {
            let doc = art::document(view, w, h, body);
            if let Ok(tree) = usvg::Tree::from_str(&doc, options) {
                resvg::render(&tree, Transform::identity(), &mut frame.as_mut());
            }
        };
        svg_over(bg, &art::worn_overlay());
        // The print, worn into the face: its texture showing through it a little.
        if let Some(mut print) = Pixmap::new(w, h) {
            svg_over(&mut print, &art::printed(&self.layout));
            worn_into(&mut print, bg);
            bg.draw_pixmap(
                0,
                0,
                print.as_ref(),
                &PixmapPaint {
                    opacity: 0.94,
                    ..PixmapPaint::default()
                },
                Transform::identity(),
                None,
            );
        }
        // The screws, each turned as it was last tightened, and the jacks.
        for (i, (x, y, r)) in art::screws().into_iter().enumerate() {
            let d = (2.0 * r * s).round().max(1.0) as u32;
            let deg = (i as f32 * 47.0) % 180.0;
            put_picture(
                bg,
                pictures.part(Part::Screw, d, d),
                (x * s, y * s),
                deg,
                (false, false),
                (1.0, 1.0),
            );
        }
        for (x, y, r, nut) in art::jacks() {
            let d = (2.0 * if nut { r + 6.0 } else { r * 1.12 } * s)
                .round()
                .max(1.0) as u32;
            put_picture(
                bg,
                pictures.part(Part::Jack, d, d),
                (x * s, y * s),
                20.0,
                (false, false),
                (1.0, 1.0),
            );
        }
        // The lamp's light over all of it.
        lamp(bg, s);
        self.frame = self.background.clone();
    }

    /// The layers showing `scene`, in the order they are drawn.
    fn layers(&self, scene: &Scene) -> Vec<Layer> {
        let silent = feedback_silent(&scene.values);
        let worn = self.skin == Skin::Worn;
        let control = if worn {
            art::control_worn
        } else {
            art::control
        };
        let mut out: Vec<Layer> = CONTROLS
            .iter()
            .zip(scene.values)
            .map(|(c, v)| {
                let midi = match c.kind {
                    Kind::Wheel { detent: true, .. } => scene.midi.0,
                    Kind::Wheel { .. } => scene.midi.1,
                    _ => 0.0,
                };
                let mut layer = control(c, v, midi);
                // FEEDBACK while EXTERNAL INPUT is closed: dimmed as the parts with no
                // parameter are, still operable.
                if silent && c.param == "feedback" {
                    if worn {
                        // (A picture is not dimmed: the knob is shaded.)
                        layer.over.push_str(&format!(
                            "<circle r='{}' fill='#000' fill-opacity='0.5'/>",
                            crate::svg::N(art::STD.skirt + 1.0)
                        ));
                    } else {
                        layer.body = format!("<g opacity='0.5'>{}</g>", layer.body);
                    }
                }
                layer
            })
            .collect();
        if worn {
            out.push(art::power_worn(scene.power));
            out.push(art::lamp_worn(scene.power));
            out.push(art::overload_worn(scene.overload));
            out.push(crate::ultra::ultra_worn(scene.ultra, scene.ultra_still));
        } else {
            out.push(art::power(scene.power));
            out.push(art::lamp(scene.power));
            out.push(art::overload(scene.overload));
            out.push(crate::ultra::ultra(scene.ultra, scene.ultra_still));
        }
        out.push(if worn {
            art::plate_worn()
        } else {
            art::plate()
        });
        if let Some(c) = scene.learning.and_then(|i| CONTROLS.get(i)) {
            out.push(learn::ring(c));
        }
        if let Some((t, x, y)) = &scene.tip {
            out.push(art::tip(
                &self.fonts,
                *x,
                *y,
                t,
                TIP_TEXT * self.ui / self.scale,
            ));
        }
        if let Some(n) = &scene.note {
            out.push(learn::note(&self.fonts, n));
        }
        if let Some(m) = &scene.menu {
            out.push(learn::menu(&self.fonts, m));
        }
        out
    }

    /// The lettering's size of the tip, a note and a menu, in drawing units (its logical pixels at
    /// this scale).
    pub fn text_size(&self) -> f64 {
        TIP_TEXT * self.ui / self.scale
    }

    /// A layer's pixels, aligned to the frame's: their place and their image (its body, its
    /// pictures turned in place, then what goes over them).
    fn render_layer(&mut self, layer: &Layer) -> Option<(i32, i32, Pixmap)> {
        let (ox, oy) = layer.origin;
        let b = layer.bounds;
        let s = self.scale;
        let (x0, y0) = (((ox + b[0]) * s).floor(), ((oy + b[1]) * s).floor());
        let (x1, y1) = (((ox + b[2]) * s).ceil(), ((oy + b[3]) * s).ceil());
        let (w, h) = ((x1 - x0).max(1.0) as u32, (y1 - y0).max(1.0) as u32);
        let view = [x0 / s, y0 / s, f64::from(w) / s, f64::from(h) / s];
        let body = format!(
            "<g transform='translate({} {})'>{}</g>",
            crate::svg::N(ox),
            crate::svg::N(oy),
            layer.body
        );
        let mut pixmap = Pixmap::new(w, h)?;
        if !layer.body.is_empty() {
            let doc = art::document(view, w, h, &body);
            let tree = usvg::Tree::from_str(&doc, &self.options).ok()?;
            resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
        }
        if !layer.sprites.is_empty() {
            let pictures = self.pictures.get_or_insert_with(Pictures::load);
            for sp in &layer.sprites {
                let (pw, ph) = (
                    (sp.size.0 * s).round().max(1.0) as u32,
                    (sp.size.1 * s).round().max(1.0) as u32,
                );
                let at = ((ox + sp.at.0) * s - x0, (oy + sp.at.1) * s - y0);
                put_picture(
                    &mut pixmap,
                    pictures.part(sp.part, pw, ph),
                    at,
                    sp.deg as f32,
                    sp.flip,
                    (sp.zoom as f32, sp.alpha as f32),
                );
            }
        }
        if let Some(wheel) = layer.wheel {
            draw_wheel(&mut pixmap, (ox * s - x0, oy * s - y0), s, wheel);
        }
        if let Some(paddle) = layer.paddle {
            light_paddle(&mut pixmap, (ox * s - x0, oy * s - y0), s, paddle);
        }
        if !layer.over.is_empty() {
            let over = format!(
                "<g transform='translate({} {})'>{}</g>",
                crate::svg::N(ox),
                crate::svg::N(oy),
                layer.over
            );
            let doc = art::document(view, w, h, &over);
            let tree = usvg::Tree::from_str(&doc, &self.options).ok()?;
            resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
        }
        Some((x0 as i32, y0 as i32, pixmap))
    }

    /// Brings the frame up to `scene`, rendering only the layers whose art changed: whether
    /// the frame changed.
    pub fn render(&mut self, scene: &Scene) -> bool {
        self.damage = None;
        if self.shown.as_ref() == Some(scene) {
            return false;
        }
        let first = self.shown.is_none();
        self.shown = Some(scene.clone());
        let layers = self.layers(scene);
        let mut changed = self.slots.len() != layers.len();
        // Where the layers changed, as they were and as they are.
        let mut damage: Option<[i32; 4]> = None;
        let mut add = |pixels: &Option<(i32, i32, Pixmap)>| {
            if let Some((x, y, p)) = pixels {
                let r = [*x, *y, x + p.width() as i32, y + p.height() as i32];
                damage = Some(damage.map_or(r, |d| {
                    [
                        d[0].min(r[0]),
                        d[1].min(r[1]),
                        d[2].max(r[2]),
                        d[3].max(r[3]),
                    ]
                }));
            }
        };
        for slot in self.slots.iter().skip(layers.len()) {
            add(&slot.pixels);
        }
        self.slots.resize_with(layers.len(), Slot::default);
        for (i, layer) in layers.into_iter().enumerate() {
            if self.slots[i].layer.as_ref() == Some(&layer) {
                continue;
            }
            add(&self.slots[i].pixels);
            self.slots[i].pixels = self.render_layer(&layer);
            add(&self.slots[i].pixels);
            self.slots[i].layer = Some(layer);
            changed = true;
        }
        self.damage = if first {
            Some([0, 0, self.frame.width() as i32, self.frame.height() as i32])
        } else {
            damage
        };
        if changed && let Some(d) = self.damage {
            self.recompose(d);
        }
        changed
    }

    /// The frame put together again inside `rect` (pixels: left, top, right, bottom), from the
    /// background and every layer over it there, in order: each of its pixels as when the whole
    /// frame is put together.
    fn recompose(&mut self, rect: [i32; 4]) {
        let (fw, fh) = (self.frame.width() as i32, self.frame.height() as i32);
        let [x0, y0, x1, y1] = [
            rect[0].clamp(0, fw),
            rect[1].clamp(0, fh),
            rect[2].clamp(0, fw),
            rect[3].clamp(0, fh),
        ];
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let whole = [x0, y0, x1, y1] == [0, 0, fw, fh];
        let mut patch = if whole {
            self.background.clone()
        } else {
            let Some(mut p) = Pixmap::new((x1 - x0) as u32, (y1 - y0) as u32) else {
                return;
            };
            let n = (x1 - x0) as usize * 4;
            for y in y0..y1 {
                let from = (y as usize * fw as usize + x0 as usize) * 4;
                let to = (y - y0) as usize * n;
                p.data_mut()[to..to + n].copy_from_slice(&self.background.data()[from..from + n]);
            }
            p
        };
        for slot in &self.slots {
            if let Some((x, y, p)) = &slot.pixels {
                let (r, b) = (x + p.width() as i32, y + p.height() as i32);
                if *x < x1 && r > x0 && *y < y1 && b > y0 {
                    patch.draw_pixmap(
                        x - x0,
                        y - y0,
                        p.as_ref(),
                        &PixmapPaint::default(),
                        Transform::identity(),
                        None,
                    );
                }
            }
        }
        if whole {
            self.frame = patch;
            return;
        }
        let n = (x1 - x0) as usize * 4;
        for y in y0..y1 {
            let to = (y as usize * fw as usize + x0 as usize) * 4;
            let from = (y - y0) as usize * n;
            self.frame.data_mut()[to..to + n].copy_from_slice(&patch.data()[from..from + n]);
        }
    }

    /// The pixels the last render changed (left, top, right, bottom; the last two past the
    /// change), if any: where the strip's parts are drawn again over the frame
    /// (`crate::strip`) and the editor's frame is put together again.
    pub fn damage(&self) -> Option<[i32; 4]> {
        self.damage
    }

    /// The layers that float over everything (MIDI Learn's ring, the tip, the note, a menu)
    /// drawn again over `onto`, a frame the drawing's size at this scale: the editor's, over the
    /// strip's own parts (`crate::strip`), which are drawn over this frame's strip.
    pub fn draw_floating(&self, onto: &mut Pixmap) {
        // (The controls', then POWER's, its lamp's, OVERLOAD's, ULTRA's and the name plate's:
        // fixed.)
        let fixed = CONTROLS.len() + FIXED;
        for slot in self.slots.iter().skip(fixed) {
            if let Some((x, y, p)) = &slot.pixels {
                onto.draw_pixmap(
                    *x,
                    *y,
                    p.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    /// Whether any layer floats now ([`Renderer::draw_floating`]).
    pub fn floating(&self) -> bool {
        self.slots.len() > CONTROLS.len() + FIXED
    }
}

/// The layers drawn after the controls whatever the scene: POWER, its lamp, OVERLOAD, ULTRA's
/// lamp and the name plate.
const FIXED: usize = 5;

/// The panel's lamp, up and to the left, over the whole drawing at `s` pixels a unit: brighter
/// near it (the face's sheen, catching the specks of its texture), falling off towards the far
/// corner. Each pixel's light multiplied (`LAMP`: at the lamp, far from it, and how far its
/// light reaches across and down, as shares of the drawing).
fn lamp(frame: &mut Pixmap, s: f64) {
    const LAMP: (f64, f64, f64, f64) = (1.26, 0.84, 0.55, 0.9);
    let (w, h) = (frame.width() as usize, frame.height() as usize);
    let across: Vec<f64> = (0..w)
        .map(|x| {
            let u = (x as f64 + 0.5) / s / art::W;
            (-((u - 0.2) / LAMP.2).powi(2)).exp()
        })
        .collect();
    let down: Vec<f64> = (0..h)
        .map(|y| {
            let v = (y as f64 + 0.5) / s / art::H;
            (-(v / LAMP.3).powi(2)).exp()
        })
        .collect();
    let data = frame.data_mut();
    for (y, d) in down.iter().enumerate() {
        for (x, a) in across.iter().enumerate() {
            let k = LAMP.1 + (LAMP.0 - LAMP.1) * a * d;
            let i = 4 * (y * w + x);
            for c in &mut data[i..i + 3] {
                *c = (f64::from(*c) * k).round().min(255.0) as u8;
            }
        }
    }
}

/// The print as worn into the face under it: each pixel of ink a little darker where the face's
/// texture is darker than its mean, so the texture shows through it, and faded a shade.
fn worn_into(ink: &mut Pixmap, face: &Pixmap) {
    let level =
        |r: u8, g: u8, b: u8| 0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b);
    let (sum, n) = face
        .pixels()
        .iter()
        .step_by(97)
        .fold((0.0, 0.0), |(s, n), p| {
            (s + level(p.red(), p.green(), p.blue()), n + 1.0)
        });
    let mean = (sum / n).max(1.0);
    for (p, f) in ink.pixels_mut().iter_mut().zip(face.pixels()) {
        if p.alpha() == 0 {
            continue;
        }
        let ratio = level(f.red(), f.green(), f.blue()) / mean;
        let k = (0.9 + 0.22 * (ratio - 1.0)).clamp(0.74, 1.0);
        let at = |v: u8| (f64::from(v) * k).round() as u8;
        if let Some(c) = resvg::tiny_skia::PremultipliedColorU8::from_rgba(
            at(p.red()),
            at(p.green()),
            at(p.blue()),
            p.alpha(),
        ) {
            *p = c;
        }
    }
}

/// The panel's lamp (above, and up and to the left of, the panel) as a unit vector, x right, y
/// down the panel, z out of it; and the half way between it and the eye.
fn lamp_vectors() -> ((f64, f64, f64), (f64, f64, f64)) {
    let unit = |(x, y, z): (f64, f64, f64)| {
        let n = (x * x + y * y + z * z).sqrt();
        (x / n, y / n, z / n)
    };
    let light = unit((-0.45, -0.55, 0.7));
    (light, unit((light.0, light.1, light.2 + 1.0)))
}

/// A rocker's paddle lit by its shape, `at` its middle (pixels) at `s` pixels a unit: the
/// pressed half low and flat; the raised half rising from the pivot in a smooth hump that rounds
/// over at its end; its long sides rounded. Each pixel of its picture (lit evenly) is made as
/// much lighter or darker as its surface faces the lamp more or less than a flat one does, with
/// a little of the lamp's gloss on it.
fn light_paddle(frame: &mut Pixmap, at: (f64, f64), s: f64, p: art::PaddleArt) {
    let (light, half) = lamp_vectors();
    let flat = 0.3 + 0.7 * light.2;
    let (hl, hw) = (p.pw / 2.0, p.ph / 2.0);
    // The hump's height, and where it crests along the raised half (a share of it).
    let (rise, crest) = (0.32 * p.ph, 0.78);
    let (sin, cos) = p.deg.to_radians().sin_cos();
    let flip = if p.on { -1.0 } else { 1.0 };
    let reach = ((hl + 2.0).hypot(hw + 2.0) * s).ceil();
    let (fw, fh) = (frame.width() as i32, frame.height() as i32);
    let x0 = ((at.0 - reach) as i32).max(0);
    let x1 = ((at.0 + reach) as i32 + 1).min(fw);
    let y0 = ((at.1 - reach) as i32).max(0);
    let y1 = ((at.1 + reach) as i32 + 1).min(fh);
    let data = frame.data_mut();
    for py in y0..y1 {
        for px in x0..x1 {
            let (dx, dy) = (
                (f64::from(px) + 0.5 - at.0) / s,
                (f64::from(py) + 0.5 - at.1) / s,
            );
            // Into the paddle's own frame: along it (raised half positive) and across.
            let u = flip * (dx * cos + dy * sin);
            let v = -dx * sin + dy * cos;
            if u.abs() > hl - 1.0 || v.abs() > hw - 1.0 {
                continue;
            }
            // The slope along it: up the hump to its crest, then rounding over at the end.
            let t = u / hl;
            let du = if t <= 0.0 {
                0.04
            } else if t < crest {
                let x = t / crest;
                rise * 6.0 * x * (1.0 - x) / (crest * hl)
            } else {
                let x = (t - crest) / (1.0 - crest);
                -rise * 0.8 * x * x * 3.0 / ((1.0 - crest) * hl)
            };
            // The long sides rounded over their last few units.
            let e = ((v.abs() - (hw - 5.0)) / 5.0).clamp(0.0, 1.0);
            let dv = v.signum() * e * e * 1.4;
            // The surface's normal in the paddle's frame, then on the panel.
            let (nu, nv) = (-du, -dv);
            let m = (nu * nu + nv * nv + 1.0).sqrt();
            let (nu, nv, nz) = (nu / m, nv / m, 1.0 / m);
            let nu = nu * flip;
            let (nx, ny) = (nu * cos - nv * sin, nu * sin + nv * cos);
            let diffuse = (nx * light.0 + ny * light.1 + nz * light.2).max(0.0);
            let k = (0.3 + 0.7 * diffuse) / flat;
            let gloss = (nx * half.0 + ny * half.1 + nz * half.2).max(0.0).powi(28) * 60.0;
            let i = 4 * (py * fw + px) as usize;
            let a = f64::from(data[i + 3]);
            if a == 0.0 {
                continue;
            }
            for c in 0..3 {
                let v = f64::from(data[i + c]) * k + gloss * a / 255.0;
                data[i + c] = v.round().clamp(0.0, a) as u8;
            }
        }
    }
}

/// A wheel seen from above in its slot, `at` its middle (pixels) at `s` pixels a unit: white
/// ridged thermoset, a cylinder turning about its axis across the slot, lit by the panel's lamp
/// (above, and up and to the left of, the panel): its ridges every 8 degrees rolling with it,
/// each lit on its flank towards the lamp; its rim rounded over at its sides; its ends going
/// down into the slot's shade; PITCH's line or MOD.'s dot rolling with it.
fn draw_wheel(frame: &mut Pixmap, at: (f64, f64), s: f64, wheel: art::WheelArt) {
    let k = art::WHEEL_SCALE;
    let (hw, hh, r, corner) = (24.0 * k, 67.0 * k, 84.0 * k, 10.0 * k);
    let unit = |(x, y, z): (f64, f64, f64)| {
        let n = (x * x + y * y + z * z).sqrt();
        (x / n, y / n, z / n)
    };
    let (light, half) = lamp_vectors();
    let albedo = [0.80, 0.765, 0.67];
    let (fw, fh) = (frame.width() as i32, frame.height() as i32);
    let x0 = ((at.0 - hw * s).floor() as i32).max(0);
    let x1 = ((at.0 + hw * s).ceil() as i32).min(fw);
    let y0 = ((at.1 - hh * s).floor() as i32).max(0);
    let y1 = ((at.1 + hh * s).ceil() as i32).min(fh);
    let data = frame.data_mut();
    for py in y0..y1 {
        for px in x0..x1 {
            let (dx, dy) = (
                (f64::from(px) + 0.5 - at.0) / s,
                (f64::from(py) + 0.5 - at.1) / s,
            );
            // Inside the wheel's rounded window, smoothed over a pixel.
            let (qx, qy) = (
                (dx.abs() - (hw - corner)).max(0.0),
                (dy.abs() - (hh - corner)).max(0.0),
            );
            let cover = ((corner - qx.hypot(qy)) * s + 0.5).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            // Where on the cylinder: the angle from its top, towards the panel's foot.
            let th = (dy / r).clamp(-0.999, 0.999).asin();
            let deg = th.to_degrees();
            // A ridge's flanks tilt the surface along the wheel.
            let mut ph = ((deg + wheel.turn) / 8.0).rem_euclid(1.0);
            if ph > 0.5 {
                ph -= 1.0;
            }
            let tilt = if ph.abs() < 0.18 {
                -0.55 * (std::f64::consts::PI * ph / 0.18).sin()
            } else {
                0.0
            };
            let a = th + tilt;
            let side = (dx / hw).signum() * (dx.abs() / hw).powi(4) * 0.75;
            let n = unit((side, a.sin(), a.cos() * (1.0 - side * side).max(0.0).sqrt()));
            let diffuse = (n.0 * light.0 + n.1 * light.1 + n.2 * light.2).max(0.0);
            let spec = (n.0 * half.0 + n.1 * half.1 + n.2 * half.2)
                .max(0.0)
                .powi(36)
                * 0.3;
            // Its ends go down into the slot's shade.
            let t = ((dy.abs() / hh - 0.55) / 0.45).clamp(0.0, 1.0);
            let shade = 1.0 - 0.6 * t * t * (3.0 - 2.0 * t);
            let marked = match wheel.mark {
                Mark::Line => (deg + wheel.turn).abs() < 1.3,
                Mark::Dot => {
                    let at = (7.0 - wheel.turn).to_radians();
                    at.cos() > 0.0 && dx.hypot(dy - r * at.sin()) < 5.0 * k
                }
            };
            let i = 4 * (py * fw + px) as usize;
            for (c, alb) in albedo.iter().enumerate() {
                let lit = if marked {
                    0.012
                } else {
                    alb * (0.2 + 0.85 * diffuse) * shade + spec * shade
                };
                let v = 255.0 * lit.clamp(0.0, 1.0).powf(1.0 / 2.2);
                data[i + c] = (v * cover + f64::from(data[i + c]) * (1.0 - cover)).round() as u8;
            }
            data[i + 3] = (255.0 * cover + f64::from(data[i + 3]) * (1.0 - cover)).round() as u8;
        }
    }
}

/// A picture drawn with its middle at `at` (pixels), mirrored as `flip` (across, down) says,
/// then turned `deg` clockwise.
fn put_picture(
    frame: &mut Pixmap,
    p: &Pixmap,
    at: (f64, f64),
    deg: f32,
    flip: (bool, bool),
    (zoom, alpha): (f32, f32),
) {
    let (pw, ph) = (p.width() as f32, p.height() as f32);
    let sign = |f: bool| if f { -zoom } else { zoom };
    frame.draw_pixmap(
        0,
        0,
        p.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bicubic,
            opacity: alpha.clamp(0.0, 1.0),
            ..PixmapPaint::default()
        },
        Transform::from_translate(-pw / 2.0, -ph / 2.0)
            .post_scale(sign(flip.0), sign(flip.1))
            .post_rotate(deg)
            .post_translate(at.0 as f32, at.1 as f32),
        None,
    );
}

/// A rectangle (pixels: left, top, width, height), its corners rounded by `r` if given, filled
/// with a picture placed by `place`, mirrored tile to tile beyond it.
fn fill(
    frame: &mut Pixmap,
    p: &Pixmap,
    (x, y, w, h): (f64, f64, f64, f64),
    r: Option<f64>,
    place: Transform,
) {
    let paint = Paint {
        shader: Pattern::new(
            p.as_ref(),
            SpreadMode::Reflect,
            FilterQuality::Bicubic,
            1.0,
            place,
        ),
        anti_alias: true,
        ..Paint::default()
    };
    let path = match r {
        Some(r) => rounded_rect(x, y, w, h, r),
        None => Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).map(PathBuilder::from_rect),
    };
    if let Some(path) = path {
        frame.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

/// A rounded rectangle's outline (pixels).
pub(crate) fn rounded_rect(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: f64,
) -> Option<resvg::tiny_skia::Path> {
    let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
    let r = (r as f32).min(w / 2.0).min(h / 2.0);
    // A quarter circle's control points, as a share of its radius.
    let k = 0.552_284_8 * r;
    let mut p = PathBuilder::new();
    p.move_to(x + r, y);
    p.line_to(x + w - r, y);
    p.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    p.line_to(x + w, y + h - r);
    p.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    p.line_to(x + r, y + h);
    p.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    p.line_to(x, y + r);
    p.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    p.close();
    p.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::index;

    /// A frame put together again where it changed is the frame put together whole, to the
    /// bit: through knobs turned above the strip and on it, a switch, a tip coming and going.
    #[test]
    fn a_frame_put_together_in_part_is_the_frame_put_together_whole() {
        let s = 0.25;
        let mut r = Renderer::new(s, 1.0);
        let mut scene = Scene {
            values: [0.5; CONTROLS.len()],
            ..Scene::default()
        };
        r.render(&scene);
        let at = |p: &str| index(p).expect(p);
        for i in 0..12 {
            match i % 6 {
                0 => scene.values[at("cutoff")] = 0.1 * f64::from(i),
                1 => scene.values[at("drive")] = 0.08 * f64::from(i),
                2 => scene.values[at("pitch_wheel")] = 0.05 * f64::from(i),
                3 => scene.values[at("osc1_on")] = f64::from(i / 6 % 2),
                4 => scene.tip = Some(("CUTOFF: 1.2 kHz".into(), 2100.0, 300.0)),
                _ => scene.tip = None,
            }
            assert!(r.render(&scene));
            let mut whole = Renderer::new(s, 1.0);
            whole.render(&scene);
            assert!(
                r.frame() == whole.frame(),
                "step {i}: not as put together whole"
            );
        }
    }

    /// The renderer says where its frame changed: a knob turned on the panel above the strip
    /// changes nothing on the strip, one of the strip's does (its parts are drawn again over
    /// it then, `crate::strip`); the first frame changes everything.
    #[test]
    fn it_says_where_its_frame_changed() {
        let s = 0.2;
        let mut r = Renderer::new(s, 1.0);
        let mut scene = Scene::default();
        assert!(r.render(&scene));
        let (w, h) = r.size();
        assert_eq!(r.damage(), Some([0, 0, w as i32, h as i32]));
        let strip = (art::PANEL_H * s).round() as i32;
        scene.values[index("cutoff").expect("CUTOFF")] = 0.7;
        assert!(r.render(&scene));
        assert!(
            r.damage()
                .is_some_and(|d| d[3] <= strip && d[2] - d[0] < w as i32 / 8)
        );
        scene.values[index("drive").expect("DRIVE")] = 0.7;
        assert!(r.render(&scene));
        assert!(r.damage().is_some_and(|d| d[1] >= strip));
        assert!(!r.render(&scene));
        assert_eq!(r.damage(), None);
    }

    /// The pixels two frames differ in, inside and outside `within` (the drawing's units:
    /// left, top, right, bottom).
    fn differ(a: &Pixmap, b: &Pixmap, scale: f64, within: [f64; 4]) -> (usize, usize) {
        let (mut inside, mut outside) = (0, 0);
        for (i, (p, q)) in a.pixels().iter().zip(b.pixels()).enumerate() {
            if p == q {
                continue;
            }
            let (x, y) = (
                f64::from(i as u32 % a.width()) / scale,
                f64::from(i as u32 / a.width()) / scale,
            );
            if (within[0]..within[2]).contains(&x) && (within[1]..within[3]).contains(&y) {
                inside += 1;
            } else {
                outside += 1;
            }
        }
        (inside, outside)
    }

    /// The worn skin draws the panel in its pictures, not as drawn; a knob turned changes its
    /// own place and nothing else, and a rocker switched its own.
    #[test]
    fn the_worn_skin_turns_its_knobs_in_place() {
        let scale = 0.4;
        let scene = Scene {
            values: [0.5; CONTROLS.len()],
            ..Scene::default()
        };
        let mut worn = Renderer::with_skin(Skin::Worn, scale, 1.0);
        assert_eq!(worn.skin(), Skin::Worn);
        assert!(worn.render(&scene));
        let before = worn.frame().clone();
        let mut drawn = Renderer::new(scale, 1.0);
        drawn.render(&scene);
        let (all, _) = differ(&before, drawn.frame(), scale, [0.0, 0.0, W, H]);
        assert!(
            all > before.pixels().len() / 2,
            "{all} pixels from the drawn panel's"
        );
        for (param, v) in [("cutoff", 0.95), ("osc1_range", 0.0), ("filter_mod", 0.0)] {
            let i = index(param).expect("a control");
            let mut turned = scene.clone();
            turned.values[i] = v;
            assert!(worn.render(&turned), "{param} not drawn again");
            let c = &CONTROLS[i];
            let (cx, cy) = c.centre();
            // (The worn layer's extent: a rocker's reaches its own shadow.)
            let k = art::control_worn(c, v, 0.0).bounds;
            // (Its layer is whole pixels: two past its extent.)
            let m = 2.0 / scale;
            let (inside, outside) = differ(
                &before,
                worn.frame(),
                scale,
                [cx + k[0] - m, cy + k[1] - m, cx + k[2] + m, cy + k[3] + m],
            );
            assert!(inside > 50, "{param}: {inside} pixels of its own changed");
            assert_eq!(outside, 0, "{param}: {outside} pixels elsewhere changed");
            worn.render(&scene);
        }
    }

    /// A worn knob's light stays where the lamp is, up and to the left: turned to either end,
    /// its upper left is lighter than its lower right, by about as much (the pictures lit
    /// evenly all round; the lamp's light laid over them, not turned with them).
    #[test]
    fn a_worn_knobs_light_stays_with_the_lamp() {
        let scale = 0.8;
        let i = index("emphasis").expect("a control");
        let (cx, cy) = CONTROLS[i].centre();
        let light = |v: f64| {
            let mut scene = Scene {
                values: [0.5; CONTROLS.len()],
                ..Scene::default()
            };
            scene.values[i] = v;
            let mut r = Renderer::with_skin(Skin::Worn, scale, 1.0);
            r.render(&scene);
            // The mean level of a patch of the skirt, `dx`, `dy` units from the axis.
            let patch = |dx: f64, dy: f64| {
                let (x0, y0) = (((cx + dx) * scale) as u32, ((cy + dy) * scale) as u32);
                let mut sum = 0.0;
                for y in y0 - 3..y0 + 3 {
                    for x in x0 - 3..x0 + 3 {
                        let p = r.frame().pixel(x, y).expect("inside").demultiply();
                        sum += 0.2126 * f64::from(p.red())
                            + 0.7152 * f64::from(p.green())
                            + 0.0722 * f64::from(p.blue());
                    }
                }
                sum / 36.0
            };
            // Cap and grip, each at the upper left and the lower right.
            let (cap, grip) = (
                20.0 / std::f64::consts::SQRT_2,
                42.0 / std::f64::consts::SQRT_2,
            );
            (
                patch(-cap, -cap) - patch(cap, cap),
                patch(-grip, -grip) - patch(grip, grip),
            )
        };
        let (low, high) = (light(0.0), light(1.0));
        // (The cap is drawn unturned; the grip turns, its picture lit evenly all round.)
        for ((a, b), part, apart) in [
            ((low.0, high.0), "cap", 0.5),
            ((low.1, high.1), "grip", 6.0),
        ] {
            assert!(
                a > 1.0 && b > 1.0,
                "{part}: upper left {a:.1} and {b:.1} levels lighter"
            );
            assert!(
                (a - b).abs() < apart,
                "{part}: {a:.1} against {b:.1} levels as it turns"
            );
        }
    }

    /// FEEDBACK's knob is drawn dimmed while EXTERNAL INPUT is switched off, and nothing
    /// else on the panel changes.
    #[test]
    fn feedback_dims_while_external_input_is_closed() {
        let open = Scene {
            values: [0.5; CONTROLS.len()],
            ..Scene::default()
        };
        let mut closed = open.clone();
        let feedback = index("feedback").expect("a control");
        let ext = index("ext_on").expect("a control");
        closed.values[ext] = 0.0;
        let frame = |scene: &Scene| {
            let mut r = Renderer::new(0.5, 1.0);
            r.render(scene);
            (r.frame().clone(), r.scale())
        };
        let ((a, s), (b, _)) = (frame(&open), frame(&closed));
        let c = &CONTROLS[feedback];
        let (cx, cy) = c.centre();
        let k = art::bounds(&c.kind);
        let knob = |x: u32, y: u32| {
            let (x, y) = (f64::from(x) / s, f64::from(y) / s);
            (cx + k[0]..cx + k[2]).contains(&x) && (cy + k[1]..cy + k[3]).contains(&y)
        };
        // EXTERNAL INPUT's own switch moves too.
        let switch = &CONTROLS[ext];
        let (sx, sy) = switch.centre();
        let sk = art::bounds(&switch.kind);
        let rocker = |x: u32, y: u32| {
            let (x, y) = (f64::from(x) / s, f64::from(y) / s);
            (sx + sk[0]..sx + sk[2]).contains(&x) && (sy + sk[1]..sy + sk[3]).contains(&y)
        };
        let (mut inside, mut outside) = (0, 0);
        for (i, (p, q)) in a.pixels().iter().zip(b.pixels()).enumerate() {
            let (x, y) = (i as u32 % a.width(), i as u32 / a.width());
            if p != q {
                if knob(x, y) {
                    inside += 1;
                } else if !rocker(x, y) {
                    outside += 1;
                }
            }
        }
        assert!(inside > 500, "{inside} of the knob's pixels dimmed");
        assert_eq!(outside, 0, "{outside} pixels elsewhere changed");
    }
}
