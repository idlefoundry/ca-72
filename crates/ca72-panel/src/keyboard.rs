//! The keyboard occupies the extension controls' space, keeping the rail and left hand.
//! Coordinates are panel units; MIDI 60 is labelled C4. Black keys take hit-test priority.

use resvg::{
    tiny_skia::{FillRule, Mask, PathBuilder, Pixmap, Transform},
    usvg,
};

use crate::{
    Skin, art,
    fonts::{FAMILY, Fonts, Weight},
};

/// The keys' sprites (`assets/keyboard/README.md`).
pub(crate) static WHITE_KEY: &[u8] = include_bytes!("../assets/keyboard/white-key-v4.png");
pub(crate) static BLACK_KEY: &[u8] = include_bytes!("../assets/keyboard/black-key-v4.png");

pub const AREA: [f64; 4] = [
    370.0,
    art::PANEL_H + crate::strip::RAIL,
    art::W - 28.0,
    art::H - 6.0,
];
pub const BED: [f64; 4] = [414.0, 1122.0, 3038.0, 1728.0];
const WHITE_H: f64 = BED[3] - BED[1];
const BLACK_H: f64 = WHITE_H * 0.66;
pub const KEYS: usize = 44;
pub const MAPPING: &str = "awsedftgyhujkolp;";
/// A starts on C3, independently of the visible F-to-C keyboard and the sound controls.
pub const TYPE_ROOT: u8 = 48;
const WHITE_W: f64 = (BED[2] - BED[0]) / 26.0;
const CLEARANCE: f64 = 3.0;
const NOTCH_Y: f64 = BED[1] + BLACK_H + 5.0;
const KEYBED_TOP: f64 = BED[1] - 12.0;
/// The visible wood ends at the key bed's outer border, three units above its frame.
pub const HEADER: f64 = (AREA[1] + KEYBED_TOP - 3.0) / 2.0;
pub const BUTTON_H: f64 = crate::strip::KEY_H;
const ARROW_W: f64 = crate::strip::KEY_W;
const HEADER_GAP: f64 = 14.0;
const RANGE_W: f64 = 348.0;
const RANGE_H: f64 = 50.0;
const RANGE_X: f64 = crate::strip::CW - RANGE_W / 2.0;
const RANGE_SURROUND: f64 = (BUTTON_H - RANGE_H) / 2.0;
const ACTION_W: f64 = 230.0;
pub const UP_X: f64 = RANGE_X + RANGE_W + RANGE_SURROUND + HEADER_GAP + ARROW_W / 2.0;
pub const TYPING_X: f64 = crate::strip::CO - (ACTION_W + HEADER_GAP) / 2.0;
pub const PANIC_X: f64 = crate::strip::CO + (ACTION_W + HEADER_GAP) / 2.0;
/// A held key keeps its material and shading, with a darker surface over the key bed.
const PRESSED_LEVEL: f32 = 0.65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Note(u8),
    Down,
    Up,
    Type,
    Panic,
    Back,
}

const BUTTONS: [(f64, f64, &str, Target); 4] = [
    (
        RANGE_X - RANGE_SURROUND - HEADER_GAP - ARROW_W / 2.0,
        ARROW_W,
        "−",
        Target::Down,
    ),
    (UP_X, ARROW_W, "+", Target::Up),
    (TYPING_X, ACTION_W, "TYPING", Target::Type),
    (PANIC_X, ACTION_W, "PANIC", Target::Panic),
];

fn buttons(standalone: bool) -> impl Iterator<Item = (f64, f64, &'static str, Target)> {
    BUTTONS.into_iter().filter(move |(_, _, _, target)| {
        standalone || !matches!(target, Target::Type | Target::Panic)
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub root: u8,
    pub standalone: bool,
    pub typing: bool,
    pub typing_root: u8,
    pub held: [u64; 2],
    pub hover: Option<Target>,
    pub skin: Skin,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            root: 41,
            standalone: false,
            typing: false,
            typing_root: TYPE_ROOT,
            held: [0, 0],
            hover: None,
            skin: Skin::Worn,
        }
    }
}

pub fn inside(x: f64, y: f64) -> bool {
    (AREA[0]..AREA[2]).contains(&x) && (AREA[1]..AREA[3]).contains(&y)
}

pub fn black(note: u8) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

pub fn label(note: u8) -> String {
    format!(
        "{}{}",
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
        ][usize::from(note % 12)],
        i16::from(note) / 12 - 1
    )
}

/// Octave transposition retains the F-to-C span except at the MIDI range limits. The final octave leaves notes above MIDI 127 disabled.
pub fn shifted(root: u8, up: bool) -> u8 {
    if up {
        root.saturating_add(12).min(84)
    } else {
        root.saturating_sub(12)
    }
}

/// TYPE always starts on C, with one octave a press and no MIDI-range wrapping.
pub fn shifted_typing(root: u8, up: bool) -> u8 {
    let root = root.min(120) / 12 * 12;
    if up {
        root.saturating_add(12).min(120)
    } else {
        root.saturating_sub(12)
    }
}

pub fn rect(root: u8, note: u8) -> Option<[f64; 4]> {
    let relative = note.checked_sub(root)?;
    if usize::from(relative) >= KEYS {
        return None;
    }
    let whites = (0..relative).filter(|i| !black(root + i)).count() as f64;
    let x = BED[0] + whites * WHITE_W;
    Some(if black(note) {
        [
            x - WHITE_W * 0.25,
            BED[1],
            x + WHITE_W * 0.25,
            BED[1] + BLACK_H,
        ]
    } else {
        [x, BED[1], x + WHITE_W - 2.0, BED[3]]
    })
}

/// The rear of each natural key is a stem, with real spaces for its neighbouring
/// accidentals. The full-width front remains aligned across the whole keyboard.
pub fn white_shape(root: u8, note: u8) -> Option<[[f64; 2]; 8]> {
    if black(note) {
        return None;
    }
    let [x0, y0, x1, y1] = rect(root, note)?;
    let left = note > root && black(note - 1);
    let right = usize::from(note - root) + 1 < KEYS && note < 127 && black(note + 1);
    let stem_left = x0
        + if left {
            WHITE_W * 0.25 + CLEARANCE
        } else {
            0.0
        };
    let stem_right = if right {
        x0 + WHITE_W * 0.75 - CLEARANCE
    } else {
        x1
    };
    Some([
        [stem_left, y0],
        [stem_right, y0],
        [stem_right, NOTCH_Y],
        [x1, NOTCH_Y],
        [x1, y1],
        [x0, y1],
        [x0, NOTCH_Y],
        [stem_left, NOTCH_Y],
    ])
}

fn white_contains(root: u8, note: u8, x: f64, y: f64) -> bool {
    let Some(shape) = white_shape(root, note) else {
        return false;
    };
    let (left, right) = if y < NOTCH_Y {
        (shape[0][0], shape[1][0])
    } else {
        (shape[5][0], shape[4][0])
    };
    (left..right).contains(&x) && (BED[1]..BED[3]).contains(&y)
}

fn shape_svg(shape: [[f64; 2]; 8]) -> String {
    let mut path = format!("M {} {}", shape[0][0], shape[0][1]);
    for [x, y] in &shape[1..] {
        path += &format!(" L {x} {y}");
    }
    path + " Z"
}

pub fn hit(root: u8, x: f64, y: f64, standalone: bool) -> Option<Target> {
    if !inside(x, y) {
        return None;
    }
    for (cx, w, _, target) in buttons(standalone) {
        if (x - cx).abs() <= w / 2.0 && (y - HEADER).abs() <= BUTTON_H / 2.0 {
            return Some(target);
        }
    }
    for dark in [true, false] {
        for relative in 0..KEYS {
            let note = usize::from(root) + relative;
            if note > 127 {
                continue;
            }
            let note = note as u8;
            if black(note) != dark {
                continue;
            }
            let in_key = if dark {
                rect(root, note)
                    .is_some_and(|r| (r[0]..r[2]).contains(&x) && (r[1]..r[3]).contains(&y))
            } else {
                white_contains(root, note, x, y)
            };
            if in_key {
                return Some(Target::Note(note));
            }
        }
    }
    Some(Target::Back)
}

pub fn damage(was: Option<&Scene>, now: Option<&Scene>) -> Vec<[f64; 4]> {
    match (was, now) {
        (None, None) => Vec::new(),
        (Some(a), Some(b))
            if a.root == b.root
                && a.standalone == b.standalone
                && a.typing == b.typing
                && a.typing_root == b.typing_root
                && a.skin == b.skin
                && a.hover == b.hover =>
        {
            (0..128)
                .filter(|n| (a.held[n / 64] ^ b.held[n / 64]) & (1 << (n % 64)) != 0)
                .filter_map(|n| rect(b.root, n as u8))
                .map(|[x0, y0, x1, y1]| [x0 - 8.0, y0 - 8.0, x1 + 12.0, y1 + 12.0])
                .collect()
        }
        _ => vec![AREA],
    }
}

/// Kept at one scale: nothing is drawn for unchanged notes. The resulting overlay is copied
/// only in the caller's damaged rectangle, leaving its rail and left-hand pixels untouched.
pub struct Renderer {
    scale: f64,
    options: usvg::Options<'static>,
    fonts: Fonts,
    glass: Pixmap,
    frame: Pixmap,
    shown: Option<Scene>,
    materials: Option<Materials>,
    white: Option<Pixmap>,
    black: Option<Pixmap>,
    white_pressed: Option<Pixmap>,
    black_pressed: Option<Pixmap>,
    base: Option<(Scene, Pixmap)>,
    /// A look's dressing for it (decisions.md R-SKIN-PACKS), where it has one.
    dressing: Dressing,
}

/// What a look gives the keyboard view: its case's wood, its keys, its header keys' cap, its
/// glass, its lamps' colour and a drawing over its case (the built-in ones where it gives none).
#[derive(Clone, Default)]
struct Dressing {
    wood: Option<Pixmap>,
    /// The case in the face's picture (tiled as the strip's face) rather than wood's.
    face: Option<Pixmap>,
    white: Option<Pixmap>,
    black: Option<Pixmap>,
    button: Option<Pixmap>,
    lamp: Option<crate::strip::Lamp>,
    art: Option<std::sync::Arc<usvg::Tree>>,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl Renderer {
    pub fn new(scale: f64) -> Self {
        let fonts = Fonts::new();
        Self {
            scale,
            options: usvg::Options {
                fontdb: fonts.database(),
                font_family: FAMILY.into(),
                ..Default::default()
            },
            fonts,
            glass: Pixmap::decode_png(crate::skin::GLASS).expect("preset display glass"),
            frame: crate::strip::blank(scale).expect("a strip frame"),
            shown: None,
            materials: None,
            white: None,
            black: None,
            white_pressed: None,
            black_pressed: None,
            base: None,
            dressing: Dressing::default(),
        }
    }

    /// Drawn in `look`'s pictures, typeface, drawing and lamp colour (`lamp`) for it, where it
    /// has them.
    pub(crate) fn set_look(&mut self, look: &crate::look::Look, lamp: crate::strip::Lamp) {
        let pack = |name: &str| {
            let size = crate::skin::slot_size(name).unwrap_or((1, 1));
            look.pictures.get(name).and_then(|p| p.picture(size))
        };
        self.dressing = Dressing {
            wood: pack("walnut"),
            face: look.keyboard_face.then(|| {
                let face = pack("face").unwrap_or_else(crate::skin::face_picture);
                match (look.pictures.contains_key("face"), look.face_tint) {
                    (true, Some(t)) => crate::skin::tinted(&face, t),
                    _ => face,
                }
            }),
            white: pack("white-key"),
            black: pack("black-key"),
            button: pack("key"),
            lamp: Some(lamp),
            art: look.keyboard_art.clone(),
        };
        self.glass = pack("glass").unwrap_or_else(|| {
            Pixmap::decode_png(crate::skin::GLASS).expect("preset display glass")
        });
        self.fonts = look
            .fonts
            .and_then(|(regular, bold)| Fonts::with(regular, bold).ok())
            .unwrap_or_default();
        self.options = usvg::Options {
            fontdb: self.fonts.database(),
            font_family: FAMILY.into(),
            ..Default::default()
        };
        (self.materials, self.white, self.black) = (None, None, None);
        (self.white_pressed, self.black_pressed) = (None, None);
        (self.base, self.shown) = (None, None);
    }

    pub fn rescale(&mut self, scale: f64) {
        if self.scale != scale {
            self.scale = scale;
            self.frame = crate::strip::blank(scale).expect("a strip frame");
            self.shown = None;
            self.base = None;
            self.white = None;
            self.black = None;
            self.white_pressed = None;
            self.black_pressed = None;
        }
    }

    pub fn over(&mut self, destination: &mut Pixmap, scene: &Scene, clip: [i32; 4]) {
        if scene.skin == Skin::Worn && self.white.is_none() {
            let dressing = &self.dressing;
            let materials = self
                .materials
                .get_or_insert_with(|| Materials::with(dressing));
            self.white = Some(key_size(&materials.white, self.scale, false));
            self.black = Some(key_size(&materials.black, self.scale, true));
            self.white_pressed = self.white.as_ref().map(pressed_key);
            self.black_pressed = self.black.as_ref().map(pressed_key);
        }
        if self.shown.as_ref() != Some(scene) {
            let mut still = scene.clone();
            still.held = [0, 0];
            let rebuilt = self.base.as_ref().is_none_or(|(was, _)| *was != still);
            if rebuilt {
                self.frame.fill(resvg::tiny_skia::Color::TRANSPARENT);
                if scene.skin == Skin::Worn {
                    self.materials
                        .as_ref()
                        .expect("worn materials loaded")
                        .draw(&mut self.frame, self.scale, scene.standalone);
                    // A look's drawing over the case, under the key bed and the print.
                    if let Some(art) = &self.dressing.art {
                        let (w, h) = (self.frame.width(), self.frame.height());
                        let size = art.size();
                        if let Some(mut layer) = Pixmap::new(w, h) {
                            resvg::render(
                                art,
                                Transform::from_scale(
                                    w as f32 / size.width(),
                                    h as f32 / size.height(),
                                ),
                                &mut layer.as_mut(),
                            );
                            self.frame.draw_pixmap(
                                0,
                                0,
                                layer.as_ref(),
                                &resvg::tiny_skia::PixmapPaint::default(),
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
                let body = drawing(&still, &self.fonts);
                let document = art::document(
                    [
                        0.0,
                        art::PANEL_H,
                        f64::from(self.frame.width()) / self.scale,
                        f64::from(self.frame.height()) / self.scale,
                    ],
                    self.frame.width(),
                    self.frame.height(),
                    &body,
                );
                let tree = usvg::Tree::from_str(&document, &self.options)
                    .expect("the keyboard's built-in SVG");
                resvg::render(&tree, Transform::identity(), &mut self.frame.as_mut());
                self.range_display(scene);
                self.base = Some((still, self.frame.clone()));
            }
            if scene.skin == Skin::Worn {
                let changed = if rebuilt {
                    vec![AREA]
                } else {
                    damage(self.shown.as_ref(), Some(scene))
                };
                for rect in changed {
                    self.draw_keys(scene, rect);
                }
            } else {
                self.frame.fill(resvg::tiny_skia::Color::TRANSPARENT);
                {
                    let body = drawing(scene, &self.fonts);
                    let document = art::document(
                        [
                            0.0,
                            art::PANEL_H,
                            f64::from(self.frame.width()) / self.scale,
                            f64::from(self.frame.height()) / self.scale,
                        ],
                        self.frame.width(),
                        self.frame.height(),
                        &body,
                    );
                    let tree =
                        usvg::Tree::from_str(&document, &self.options).expect("keyboard SVG");
                    resvg::render(&tree, Transform::identity(), &mut self.frame.as_mut());
                    self.range_display(scene);
                }
            }
            self.shown = Some(scene.clone());
        }
        let s = self.scale;
        let bounds = [
            (AREA[0] * s).floor() as i32,
            ((AREA[1] - art::PANEL_H) * s).floor() as i32,
            (AREA[2] * s).ceil() as i32,
            ((AREA[3] - art::PANEL_H) * s).ceil() as i32,
        ];
        let x0 = clip[0].max(bounds[0]).max(0);
        let x1 = clip[2]
            .min(bounds[2])
            .min(destination.width() as i32)
            .min(self.frame.width() as i32);
        let y0 = clip[1].max(bounds[1]).max(0);
        let y1 = clip[3]
            .min(bounds[3])
            .min(destination.height() as i32)
            .min(self.frame.height() as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let dw = destination.width() as usize;
        let sw = self.frame.width() as usize;
        for y in y0..y1 {
            let dst = (y as usize * dw + x0 as usize) * 4;
            let src = (y as usize * sw + x0 as usize) * 4;
            let n = (x1 - x0) as usize * 4;
            // Keep the fractional boundary pixels. Inward clipping left a dark
            // strip before the silver trim; blend coverage over that trim.
            let from = &self.frame.data()[src..src + n];
            let to = &mut destination.data_mut()[dst..dst + n];
            for (to, from) in to.chunks_exact_mut(4).zip(from.chunks_exact(4)) {
                let keep = 255 - from[3];
                for channel in 0..4 {
                    to[channel] = from[channel] + crate::render::mul255(to[channel], keep);
                }
            }
        }
    }

    fn range_display(&mut self, scene: &Scene) {
        crate::strip::pixel_screen(
            &self.options,
            &mut self.frame,
            &self.glass,
            self.scale,
            (
                RANGE_X,
                HEADER - art::PANEL_H - RANGE_H / 2.0,
                RANGE_W,
                RANGE_H,
            ),
            &format!(
                "{} - {}",
                label(scene.root),
                label(scene.root.saturating_add(43).min(127))
            ),
            &self.dressing.lamp.unwrap_or(crate::strip::Lamp::ORANGE),
        );
    }
    fn draw_keys(&mut self, scene: &Scene, dirty: [f64; 4]) {
        let s = self.scale;
        let x0 = (dirty[0] * s).floor().max(0.0) as i32;
        let y0 = ((dirty[1] - art::PANEL_H) * s).floor().max(0.0) as i32;
        let x1 = (dirty[2] * s).ceil().min(f64::from(self.frame.width())) as i32;
        let y1 = ((dirty[3] - art::PANEL_H) * s)
            .ceil()
            .min(f64::from(self.frame.height())) as i32;
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let mut patch = Pixmap::new((x1 - x0) as u32, (y1 - y0) as u32).expect("key damage");
        let (_, base) = self.base.as_ref().expect("keyboard base");
        let n = (x1 - x0) as usize * 4;
        for y in y0..y1 {
            let src = (y as usize * base.width() as usize + x0 as usize) * 4;
            let dst = (y - y0) as usize * n;
            patch.data_mut()[dst..dst + n].copy_from_slice(&base.data()[src..src + n]);
        }
        for dark in [false, true] {
            for relative in 0..KEYS {
                let note = scene.root + relative as u8;
                if black(note) != dark {
                    continue;
                }
                let r = rect(scene.root, note).expect("visible key");
                if r[2] + 12.0 < dirty[0]
                    || r[0] - 8.0 > dirty[2]
                    || r[3] + 12.0 < dirty[1]
                    || r[1] - 8.0 > dirty[3]
                {
                    continue;
                }
                let held = scene.held[usize::from(note) / 64] & (1 << (note % 64)) != 0;
                let key = match (dark, held) {
                    (true, true) => self.black_pressed.as_ref(),
                    (false, true) => self.white_pressed.as_ref(),
                    (true, false) => self.black.as_ref(),
                    (false, false) => self.white.as_ref(),
                }
                .expect("key material loaded");
                let key_x = (r[0] * s).round() as i32;
                let key_y = ((r[1] - art::PANEL_H) * s).round() as i32;
                let clipped = if dark {
                    None
                } else {
                    let points = white_shape(scene.root, note).expect("natural key");
                    let mut path = PathBuilder::new();
                    path.move_to(points[0][0] as f32, points[0][1] as f32);
                    for [x, y] in &points[1..] {
                        path.line_to(*x as f32, *y as f32);
                    }
                    path.close();
                    let mut mask = Mask::new(key.width(), key.height()).expect("key silhouette");
                    mask.fill_path(
                        &path.finish().expect("closed natural key"),
                        FillRule::Winding,
                        true,
                        Transform::from_scale(s as f32, s as f32).post_translate(
                            -key_x as f32,
                            -(art::PANEL_H * s) as f32 - key_y as f32,
                        ),
                    );
                    // Keep the mask in the key's coordinates. Moving the damage rectangle
                    // must not change edge coverage at fractional scales.
                    let mut clipped = key.clone();
                    clipped.apply_mask(&mask);
                    Some(clipped)
                };
                crate::render::put(
                    &mut patch,
                    clipped.as_ref().unwrap_or(key),
                    (key_x - x0, key_y - y0),
                    1.0,
                );
            }
        }
        for y in y0..y1 {
            let dst = (y as usize * self.frame.width() as usize + x0 as usize) * 4;
            let src = (y - y0) as usize * n;
            self.frame.data_mut()[dst..dst + n].copy_from_slice(&patch.data()[src..src + n]);
        }
    }
}

fn drawing(scene: &Scene, fonts: &Fonts) -> String {
    let mut out = String::from(
        "<defs><linearGradient id='keyface' x2='0' y2='1'><stop stop-color='#252523'/><stop offset='1' stop-color='#181918'/></linearGradient><linearGradient id='ivory' x2='.2' y2='1'><stop stop-color='#ccc8b6'/><stop offset='.2' stop-color='#e1dcc8'/><stop offset='1' stop-color='#c9c2af'/></linearGradient><linearGradient id='blackkey' x2='1' y2='1'><stop stop-color='#434441'/><stop offset='.3' stop-color='#242623'/><stop offset='1' stop-color='#101210'/></linearGradient><linearGradient id='keycap' x2='0' y2='1'><stop stop-color='#514e47'/><stop offset='.35' stop-color='#393833'/><stop offset='1' stop-color='#242522'/></linearGradient></defs>",
    );
    // Darker versions of the same gradients; alpha stays opaque even on black keys.
    out += "<defs><linearGradient id='ivory-pressed' x2='.2' y2='1'><stop stop-color='#858276'/><stop offset='.2' stop-color='#928f82'/><stop offset='1' stop-color='#837e72'/></linearGradient><linearGradient id='blackkey-pressed' x2='1' y2='1'><stop stop-color='#2c2c2a'/><stop offset='.3' stop-color='#171917'/><stop offset='1' stop-color='#0a0c0a'/></linearGradient></defs>";
    out += "<defs><linearGradient id='keyboard-wood' x2='0' y2='1'><stop stop-color='#604332'/><stop offset='.5' stop-color='#4a3325'/><stop offset='1' stop-color='#39291f'/></linearGradient></defs>";
    let button_y = HEADER - BUTTON_H / 2.0;
    let text_y = |text: &str, size| fonts.baseline_for_centre(text, size, Weight::Regular, HEADER);
    let octave_y = text_y("OCTAVE", 28.0);
    out += &format!(
        "<g font-family='{FAMILY}'><rect x='370' y='1041' width='2710' height='728' fill='{}'/><path d='M 371 1041 V 1769' stroke='#c2bdab' stroke-width='2'/><text x='{}' y='{octave_y}' font-size='28' text-anchor='middle' fill='#d9d2c0'>OCTAVE</text>",
        if scene.skin == Skin::Worn {
            "none"
        } else {
            "url(#keyboard-wood)"
        },
        crate::strip::CV,
    );
    for (x, w, label, target) in buttons(scene.standalone) {
        let on = target == Target::Type && scene.typing;
        let fill = if scene.skin == Skin::Worn && !on {
            "none"
        } else if on {
            "#b0702b"
        } else {
            "url(#keycap)"
        };
        let stroke = if scene.hover == Some(target) {
            "#858174"
        } else if scene.skin == Skin::Worn {
            "none"
        } else {
            "#1a1917"
        };
        out += &format!(
            "<rect x='{}' y='{button_y}' width='{w}' height='{BUTTON_H}' fill='{fill}' fill-opacity='{}' rx='7' stroke='{stroke}' stroke-width='5'/><text x='{x}' y='{}' font-size='20' font-weight='700' text-anchor='middle' fill='{}'>{label}</text>",
            x - w / 2.0,
            if scene.skin == Skin::Worn {
                "0.28"
            } else {
                "1"
            },
            fonts.baseline_for_centre(label, 20.0, Weight::Bold, HEADER),
            if on { "#2a1006" } else { "#d9d2c0" }
        );
    }
    out += &format!(
        "<rect x='402' y='{KEYBED_TOP}' width='2648' height='632' fill='#0c100e' rx='7' stroke='#241d15' stroke-width='6'/>"
    );
    if scene.skin == Skin::Drawn {
        for dark in [false, true] {
            for relative in 0..KEYS {
                let note = scene.root + relative as u8;
                if black(note) != dark {
                    continue;
                }
                let [x0, y0, x1, y1] = rect(scene.root, note).expect("a displayed note");
                let down =
                    note <= 127 && scene.held[usize::from(note) / 64] & (1 << (note % 64)) != 0;
                let fill = match (dark, down, note > 127) {
                    (_, _, true) => "#626157",
                    (true, true, false) => "url(#blackkey-pressed)",
                    (false, true, false) => "url(#ivory-pressed)",
                    (true, false, false) => "url(#blackkey)",
                    (false, false, false) => "url(#ivory)",
                };
                if dark {
                    out += &format!(
                        "<rect x='{}' y='{}' width='{}' height='{}' rx='4' fill='#060806' opacity='.4'/>",
                        x0 + 5.0,
                        y0 + 10.0,
                        x1 - x0,
                        y1 - y0
                    );
                }
                if dark {
                    out += &format!(
                        "<rect x='{x0}' y='{y0}' width='{}' height='{}' rx='2' fill='{fill}' stroke='#111310' stroke-width='2'/>",
                        x1 - x0,
                        y1 - y0,
                    );
                } else {
                    out += &format!(
                        "<path d='{}' fill='{fill}' stroke='#827b6d' stroke-width='2'/>",
                        shape_svg(white_shape(scene.root, note).expect("natural key")),
                    );
                }
                if !dark && note.is_multiple_of(12) {
                    out += &format!(
                        "<text x='{}' y='1683' font-size='22' text-anchor='middle' fill='#4d493f'>{}</text>",
                        (x0 + x1) / 2.0,
                        label(note)
                    );
                }
            }
        }
    }
    let mapping = if scene.standalone && scene.typing {
        format!("A = {} · Z / X OCTAVE", label(scene.typing_root))
    } else {
        String::new()
    };
    out += &format!(
        "<text x='{}' y='{}' font-size='18' text-anchor='middle' fill='#a9a696'>{mapping}</text></g>",
        crate::strip::CI,
        text_y(&mapping, 18.0)
    );
    out
}

struct Materials {
    wood: Pixmap,
    face: Option<Pixmap>,
    white: Pixmap,
    black: Pixmap,
    button: Pixmap,
}
impl Materials {
    /// The built-in ones, a look's `dressing` where it has its own.
    fn with(dressing: &Dressing) -> Self {
        let decode = |b: &[u8]| Pixmap::decode_png(b).expect("built-in keyboard material");
        Self {
            wood: dressing
                .wood
                .clone()
                .unwrap_or_else(|| decode(crate::skin::WALNUT)),
            face: dressing.face.clone(),
            white: dressing
                .white
                .clone()
                .unwrap_or_else(|| crop(&decode(WHITE_KEY), [427, 81, 598, 1466])),
            black: dressing
                .black
                .clone()
                .unwrap_or_else(|| crop(&decode(BLACK_KEY), [497, 49, 678, 1281])),
            button: dressing
                .button
                .clone()
                .unwrap_or_else(|| decode(crate::skin::BUTTON)),
        }
    }
    fn draw(&self, frame: &mut Pixmap, s: f64, standalone: bool) {
        let along = (art::W * s) as f32 / self.wood.width() as f32;
        let across = along / 2.2;
        // One continuous wooden case surrounds the recessed key bed. Separate
        // rails over a metal face left a stepped material seam down the side.
        let area = (
            AREA[0] * s,
            (AREA[1] - art::PANEL_H) * s,
            (AREA[2] - AREA[0]) * s,
            (AREA[3] - AREA[1]) * s,
        );
        match &self.face {
            // The face's picture, 640 units a tile, as the strip's face under it is laid.
            Some(face) => {
                let k = (640.0 * s) as f32 / face.width() as f32;
                let at = Transform::from_scale(k, k)
                    .post_translate((320.0 * s) as f32, (crate::strip::RAIL * s) as f32);
                crate::render::fill(frame, face, area, None, at);
            }
            None => crate::render::fill(
                frame,
                &self.wood,
                area,
                None,
                Transform::from_row(along, 0.0, 0.0, across, 0.0, -260.0 * across),
            ),
        }
        for (x, w, _, _) in buttons(standalone) {
            let cap = plugin_kit_materials::widened(&self.button, w / BUTTON_H);
            plugin_kit_materials::sprite(frame, &cap, s, (x, HEADER - art::PANEL_H), (w, BUTTON_H));
        }
    }
}
fn crop(source: &Pixmap, [x0, y0, x1, y1]: [u32; 4]) -> Pixmap {
    let mut out = Pixmap::new(x1 - x0, y1 - y0).expect("key crop");
    let n = (x1 - x0) as usize * 4;
    for y in y0..y1 {
        let at = (y as usize * source.width() as usize + x0 as usize) * 4;
        let to = (y - y0) as usize * n;
        out.data_mut()[to..to + n].copy_from_slice(&source.data()[at..at + n]);
    }
    out
}
fn key_size(source: &Pixmap, s: f64, dark: bool) -> Pixmap {
    let width = if dark { WHITE_W * 0.50 } else { WHITE_W - 2.0 };
    let height = if dark { BLACK_H } else { WHITE_H };
    plugin_kit_materials::resample(
        source,
        (width * s).round().max(1.0) as u32,
        (height * s).round().max(1.0) as u32,
    )
}

fn pressed_key(source: &Pixmap) -> Pixmap {
    let mut key = source.clone();
    // Preserve the silhouette's alpha; translucency could brighten a dark key's face.
    for pixel in key.data_mut().chunks_exact_mut(4) {
        for channel in &mut pixel[..3] {
            *channel = (f32::from(*channel) * PRESSED_LEVEL).round() as u8;
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_octave_indicator_is_redrawn_when_its_root_changes() {
        let was = Scene {
            standalone: true,
            typing: true,
            ..Scene::default()
        };
        let now = Scene {
            typing_root: 60,
            ..was.clone()
        };
        assert_eq!(damage(Some(&was), Some(&now)), vec![AREA]);
        let fonts = Fonts::new();
        assert!(drawing(&was, &fonts).contains("A = C3 · Z / X OCTAVE"));
        assert!(drawing(&now, &fonts).contains("A = C4 · Z / X OCTAVE"));
    }

    #[test]
    fn natural_keys_have_real_notches_and_aligned_fronts() {
        for note in [36, 38, 40, 41, 43, 45, 47] {
            let shape = white_shape(36, note).expect("natural");
            let r = rect(36, note).expect("visible");
            assert_eq!(shape[4][1], BED[3]);
            assert_eq!(shape[5][1], BED[3]);
            assert_eq!(
                hit(36, (r[0] + r[2]) / 2.0, BED[3] - 5.0, false),
                Some(Target::Note(note))
            );
            if note > 36 && black(note - 1) {
                assert!(shape[0][0] > r[0]);
                assert_eq!(
                    hit(36, shape[0][0] - CLEARANCE / 2.0, BED[1] + 20.0, false),
                    Some(Target::Back)
                );
                assert_eq!(
                    hit(36, r[0] + 5.0, NOTCH_Y - 2.0, false),
                    Some(Target::Back)
                );
            }
            if black(note + 1) {
                assert!(shape[1][0] < r[2]);
                assert_eq!(
                    hit(36, shape[1][0] + CLEARANCE / 2.0, BED[1] + 20.0, false),
                    Some(Target::Back)
                );
                assert_eq!(
                    hit(36, r[2] - 5.0, NOTCH_Y - 2.0, false),
                    Some(Target::Back)
                );
            }
        }
        let first = white_shape(41, 41).expect("first F");
        let last = white_shape(41, 84).expect("last C");
        assert_eq!(first[0][0], first[5][0]);
        assert_eq!(last[1][0], last[4][0]);
        assert!(white_shape(41, 42).is_none());
    }

    #[test]
    fn black_keys_win_and_every_midi_note_is_reachable() {
        let r = rect(36, 37).expect("C sharp");
        assert_eq!(
            hit(36, (r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0, false),
            Some(Target::Note(37))
        );
        assert_eq!(hit(36, r[0], BED[3] - 10.0, false), Some(Target::Note(36)));
        for note in 0..128u8 {
            let root = (note / 12 * 12).min(84);
            let r = rect(root, note).expect("visible");
            let y = if black(note) {
                (r[1] + r[3]) / 2.0
            } else {
                r[3] - 10.0
            };
            assert_eq!(
                hit(root, (r[0] + r[2]) / 2.0, y, false),
                Some(Target::Note(note)),
                "{note}"
            );
        }
        assert_eq!(shifted(84, true), 84);
        assert_eq!(shifted(0, false), 0);
        assert_eq!(
            hit(84, BED[2] - 5.0, BED[3] - 10.0, false),
            Some(Target::Note(127))
        );
        assert_eq!(hit(36, 100.0, 1400.0, false), None);
    }
}
