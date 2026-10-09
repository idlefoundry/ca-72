//! Rendering the panel: the background once for a scale, each moving part's layer again only
//! when its art changes, and the frame composed of them; in either skin (`crate::skin`), the
//! worn one's pictures turned and moved in the drawn parts' places.

use std::fmt;

use resvg::tiny_skia::{
    BlendMode, Color, FillRule, FilterQuality, Paint, PathBuilder, Pattern, Pixmap, PixmapPaint,
    Rect, SpreadMode, Transform,
};
use resvg::usvg;

use crate::art::{self, H, Layer, Layout, W};
use crate::controls::{CONTROLS, Kind, feedback_silent};
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
        // The wood: its grain along the top strip and the name board, down the column; each
        // board its own band of the picture, drawn out along the grain.
        let wood = &pictures.walnut;
        let along = (art::W * s) as f32 / wood.width() as f32;
        let across = along / 2.2;
        for (y, hh, band) in [
            (0.0, art::TOP, 40.0),
            (art::TOP + art::PH, art::BOARD, 470.0),
        ] {
            fill(
                bg,
                wood,
                (0.0, y * s, art::W * s, hh * s),
                None,
                Transform::from_row(along, 0.0, 0.0, across, 0.0, (y * s) as f32 - band * across),
            );
        }
        let down = ((art::PH + 4.0) * s) as f32 / wood.width() as f32;
        fill(
            bg,
            wood,
            (0.0, (art::TOP - 2.0) * s, art::COL * s, (art::PH + 4.0) * s),
            None,
            Transform::from_row(
                0.0,
                down,
                -down / 2.2,
                0.0,
                (art::COL * s) as f32 + 120.0 * down / 2.2,
                ((art::TOP - 2.0) * s) as f32,
            ),
        );
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
        fill(
            bg,
            face,
            (art::LH_X * s, art::TOP * s, art::LPW * s, art::PH * s),
            Some(6.0 * s),
            at(art::LH_X + 200.0, art::TOP),
        );
        let options = &self.options;
        let svg_over = |frame: &mut Pixmap, body: &str| {
            let doc = art::document(view, w, h, body);
            if let Ok(tree) = usvg::Tree::from_str(&doc, options) {
                resvg::render(&tree, Transform::identity(), &mut frame.as_mut());
            }
        };
        svg_over(bg, &art::worn_overlay());
        // The print, a shade worn.
        if let Some(mut print) = Pixmap::new(w, h) {
            svg_over(&mut print, &art::printed(&self.layout));
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
            );
        }
        // The lamp's falloff, multiplied in.
        if let Some(mut light) = Pixmap::new(w, h) {
            svg_over(&mut light, &art::falloff());
            bg.draw_pixmap(
                0,
                0,
                light.as_ref(),
                &PixmapPaint {
                    blend_mode: BlendMode::Multiply,
                    ..PixmapPaint::default()
                },
                Transform::identity(),
                None,
            );
        }
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
        } else {
            out.push(art::power(scene.power));
            out.push(art::lamp(scene.power));
            out.push(art::overload(scene.overload));
        }
        out.push(art::plate());
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
                );
            }
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
        if self.shown.as_ref() == Some(scene) {
            return false;
        }
        self.shown = Some(scene.clone());
        let layers = self.layers(scene);
        let mut changed = self.slots.len() != layers.len();
        self.slots.resize_with(layers.len(), Slot::default);
        for (i, layer) in layers.into_iter().enumerate() {
            if self.slots[i].layer.as_ref() == Some(&layer) {
                continue;
            }
            self.slots[i].pixels = self.render_layer(&layer);
            self.slots[i].layer = Some(layer);
            changed = true;
        }
        if changed {
            self.frame
                .data_mut()
                .copy_from_slice(self.background.data());
            for slot in &self.slots {
                if let Some((x, y, p)) = &slot.pixels {
                    self.frame.draw_pixmap(
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
        changed
    }
}

/// A picture drawn with its middle at `at` (pixels), mirrored as `flip` (across, down) says,
/// then turned `deg` clockwise.
fn put_picture(frame: &mut Pixmap, p: &Pixmap, at: (f64, f64), deg: f32, flip: (bool, bool)) {
    let (pw, ph) = (p.width() as f32, p.height() as f32);
    let sign = |f: bool| if f { -1.0 } else { 1.0 };
    frame.draw_pixmap(
        0,
        0,
        p.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bicubic,
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
            let k = art::bounds(&c.kind);
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
