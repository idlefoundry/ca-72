//! Rendering the panel: the background once for a scale, each moving part's layer again only
//! when its art changes, and the frame composed of them.

use std::fmt;

use resvg::tiny_skia::{Color, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use crate::art::{self, H, Layer, Layout, W};
use crate::controls::{CONTROLS, Kind};
use crate::fonts::{FAMILY, Fonts};

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
    /// The name plate's name (an index of [`art::PLATE_NAMES`]) and its turn, in degrees.
    pub plate: (usize, f64),
    /// A tip: its text, centred above (x, y) in the drawing.
    pub tip: Option<(String, f64, f64)>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            values: [0.0; CONTROLS.len()],
            midi: (0.5, 0.0),
            power: true,
            overload: 0.0,
            plate: (0, 0.0),
            tip: None,
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
    /// A renderer drawing `scale` pixels a panel unit, `ui` pixels a logical pixel.
    pub fn new(scale: f64, ui: f64) -> Self {
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
        };
        r.draw_background();
        r
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

    /// The layers showing `scene`, in the order they are drawn.
    fn layers(&self, scene: &Scene) -> Vec<Layer> {
        let mut out: Vec<Layer> = CONTROLS
            .iter()
            .zip(scene.values)
            .map(|(c, v)| {
                let midi = match c.kind {
                    Kind::Wheel { detent: true, .. } => scene.midi.0,
                    Kind::Wheel { .. } => scene.midi.1,
                    _ => 0.0,
                };
                art::control(c, v, midi)
            })
            .collect();
        out.push(art::power(scene.power));
        out.push(art::lamp(scene.power));
        out.push(art::overload(scene.overload));
        out.push(art::plate(&self.layout, scene.plate.0, scene.plate.1));
        if let Some((t, x, y)) = &scene.tip {
            out.push(art::tip(
                &self.fonts,
                *x,
                *y,
                t,
                TIP_TEXT * self.ui / self.scale,
            ));
        }
        out
    }

    /// A layer's pixels, aligned to the frame's: their place and their image.
    fn render_layer(&self, layer: &Layer) -> Option<(i32, i32, Pixmap)> {
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
        let doc = art::document(view, w, h, &body);
        let tree = usvg::Tree::from_str(&doc, &self.options).ok()?;
        let mut pixmap = Pixmap::new(w, h)?;
        resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
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
