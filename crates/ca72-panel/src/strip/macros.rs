//! The macros (decisions.md R-MACROS; the owner's round 2, 2026-10-20: "Yes, 2x3 as
//! recommended"): six knobs in the lower page's left area in the wheels' place, on every lower
//! page, flipped by the rail's MACRO key. Each cell, top to bottom: its name, a dot screen of
//! seven characters in its colour; its knob, the panel's standard knob with a full 0 to 10 dial;
//! the jack its assignments are dragged from, in the dial's open foot, a collar in its colour
//! under its nut.
//!
//! The area is a plate of its own laid on the strip's bare face where the left hand's was
//! (`Renderer::left_face`), so that it lies without a seam: its edges are the rail's foot, the
//! strip's left trim and foot, and the left hand's rule. What does not change (the face, the
//! knobs' and jacks' shadows, the dials' print, the screens' bezels and glass, the lamp over
//! them) is drawn once a look; each name, each collar with its jack, and each knob is a patch of
//! its own, drawn again only when it changes.

use std::fmt;
use std::thread::JoinHandle;

use plugin_kit_materials::{fill, font};
use resvg::tiny_skia::{PathBuilder, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use super::lower;
use super::worn::screen_still;
use crate::art::{self, H, Lettering, PANEL_H};
use crate::controls::{CONTROLS, Kind, MACRO_FIRST};
use crate::glow::{self, crop, hex, lay};
use crate::render::Renderer;
use crate::skin::Skin;
use crate::svg::{N, Svg, put};

// ---- The layout (the drawing's units; round 2's space budget, `budget2.txt`).

/// The left area's plate (left, top, right, bottom): from the strip's left trim to just short of
/// the left hand's rule, from the rail's foot to the strip's foot trim.
pub const AREA: (f64, f64, f64, f64) = (15.0, PANEL_H + super::RAIL, 366.0, H - 6.0);
/// The macros, and how many characters a name's screen holds.
pub const MACROS: usize = 6;
pub const NAME_PLACES: usize = 7;
/// The cells' middle across, and how far each column is from it; the rows' knobs.
const CX: f64 = 188.0;
const COL_DX: f64 = 86.0;
const ROWS: [f64; 3] = [1183.0, 1413.0, 1643.0];
/// A name's screen: its width and height, its top this far above its knob's centre.
const NAME_W: f64 = 148.0;
const NAME_H: f64 = 28.0;
const NAME_UP: f64 = 106.0;
/// The jack: this far below its knob's centre, its radius, its collar's.
const JACK_DOWN: f64 = 80.0;
pub const JACK_R: f64 = 14.0;
pub const COLLAR: f64 = JACK_R + 10.5;
/// The name's dots: smaller than the rail's (3.25 units apart, not 5), for seven in its screen.
const DOT: f64 = 3.25;

/// Macro `n`'s (0 to 5) knob's centre: two across, three down.
pub const fn knob_at(n: usize) -> (f64, f64) {
    let x = if n.is_multiple_of(2) {
        CX - COL_DX
    } else {
        CX + COL_DX
    };
    (x, ROWS[(n / 2) % 3])
}

/// Macro `n`'s name's screen (left, top, width, height).
pub const fn name_at(n: usize) -> (f64, f64, f64, f64) {
    let (x, y) = knob_at(n);
    (x - NAME_W / 2.0, y - NAME_UP, NAME_W, NAME_H)
}

/// Macro `n`'s jack's centre.
pub const fn jack_at(n: usize) -> (f64, f64) {
    let (x, y) = knob_at(n);
    (x, y + JACK_DOWN)
}

/// What a pointer finds among the macros (their knobs apart: `interact::hit_macro`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroTarget {
    Name(usize),
    Jack(usize),
}

impl MacroTarget {
    pub fn macro_n(self) -> usize {
        match self {
            MacroTarget::Name(n) | MacroTarget::Jack(n) => n,
        }
    }
}

/// Whether (x, y) in the drawing is in the left area.
pub fn inside(x: f64, y: f64) -> bool {
    (0.0..=AREA.2 + 4.0).contains(&x) && (AREA.1..=H).contains(&y)
}

/// The name or jack at (x, y) in the drawing, the macros showing.
pub fn hit(x: f64, y: f64) -> Option<MacroTarget> {
    (0..MACROS).find_map(|n| {
        let (nx, ny, w, h) = name_at(n);
        if (nx - 4.0..=nx + w + 4.0).contains(&x) && (ny - 4.0..=ny + h + 4.0).contains(&y) {
            return Some(MacroTarget::Name(n));
        }
        let (jx, jy) = jack_at(n);
        ((x - jx).hypot(y - jy) <= COLLAR + 2.0).then_some(MacroTarget::Jack(n))
    })
}

// ---- What it shows.

/// One macro as the area shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    /// Its colour (sRGB): its name's dots, its collar.
    pub colour: [u8; 3],
    /// Its name as shown: its typed one, else MACRO n.
    pub name: String,
    /// Its knob (normalized).
    pub value: f64,
    /// It modulates nothing: its name and collar dimmed.
    pub dim: bool,
    /// Its name being typed: what is typed so far (shown with a caret, outlined).
    pub typing: Option<String>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            colour: [0xdc, 0xf8, 0xfa],
            name: String::new(),
            value: 0.0,
            dim: true,
            typing: None,
        }
    }
}

/// What the area shows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MacroScene {
    pub cells: [Cell; MACROS],
}

// ---- Drawing.

/// A picture laid in the frame: its top left corner (the frame's pixels) and its pixels.
#[derive(Clone)]
struct Patch {
    at: (i32, i32),
    pixels: Pixmap,
}

impl Patch {
    fn rect(&self) -> [i32; 4] {
        [
            self.at.0,
            self.at.1,
            self.at.0 + self.pixels.width() as i32,
            self.at.1 + self.pixels.height() as i32,
        ]
    }
}

/// What the still was made for: the scale, the skin, the lettering, and the area's place in the
/// panel's frame (the face under it is the panel's renderer's at that place).
type Look = (u64, Skin, Lettering, [i32; 4]);

/// What the area's still is made from, taken from the panel's renderer (so that it can be made on
/// a thread of its own): the face under the area, the scale, the frame's top in the drawing, the
/// area's place, the skin, the lettering, the typeface's ×, and the drawing's height (the lamp's).
struct StillOf {
    face: Pixmap,
    s: f64,
    top: f64,
    rect: [i32; 4],
    skin: Skin,
    lettering: Lettering,
    times: String,
    height: f64,
}

impl StillOf {
    fn of(panel: &Renderer) -> Option<StillOf> {
        Some(StillOf {
            face: panel.left_face()?.clone(),
            s: panel.scale(),
            top: panel.top(),
            rect: panel.left_rect(),
            skin: panel.skin(),
            lettering: panel.lettering(),
            times: panel.layout().times().to_owned(),
            height: panel.drawing_h(),
        })
    }
}

/// What a collar and its jack were drawn for: the macro's colour, and whether dimmed.
type JackKey = ([u8; 3], bool);

/// The area's renderer at the panel's scale.
pub struct MacroRenderer {
    look: Option<Look>,
    /// The still being made on a thread of its own, for a look.
    warming: Option<(Look, JoinHandle<Option<Pixmap>>)>,
    options: usvg::Options<'static>,
    still: Option<Pixmap>,
    frame: Pixmap,
    names: [Option<(Cell, Patch)>; MACROS],
    jacks: [Option<(JackKey, Patch)>; MACROS],
    knobs: [Option<(u64, Patch)>; MACROS],
    shown: Option<MacroScene>,
    damages: Vec<[i32; 4]>,
}

impl fmt::Debug for MacroRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MacroRenderer")
            .field("look", &self.look)
            .finish_non_exhaustive()
    }
}

impl Default for MacroRenderer {
    fn default() -> Self {
        MacroRenderer::new()
    }
}

impl MacroRenderer {
    pub fn new() -> Self {
        MacroRenderer {
            look: None,
            warming: None,
            options: lower::lettered(),
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            names: Default::default(),
            jacks: Default::default(),
            knobs: Default::default(),
            shown: None,
            damages: Vec::new(),
        }
    }

    /// What does not change made on a thread of its own for `panel`'s look, if it is not made or
    /// being made (as the editor opens, and after its scale or look changes), so that the macros'
    /// first showing waits for none of it.
    pub fn prepare(&mut self, panel: &Renderer) {
        let look = look_of(panel);
        if (self.look == Some(look) && self.still.is_some())
            || self.warming.as_ref().is_some_and(|(l, _)| *l == look)
        {
            return;
        }
        let Some(of) = StillOf::of(panel) else {
            return;
        };
        self.warming = std::thread::Builder::new()
            .name("ca72-macros".into())
            .spawn(move || make_still(&of, &lower::lettered()))
            .ok()
            .map(|h| (look, h));
    }

    /// The frame: the area's, at the panel's frame's `Renderer::left_rect`.
    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// The places the last render changed in the frame (left, top, right, bottom).
    pub fn damages(&self) -> &[[i32; 4]] {
        &self.damages
    }

    /// Drawn for `scene` on `panel`'s face, its knobs and jacks as `panel` draws them: whether
    /// the frame changed. (Drawn again whole at another scale, look or place.)
    pub fn render(&mut self, scene: &MacroScene, panel: &mut Renderer) -> bool {
        self.damages.clear();
        let look = look_of(panel);
        let mut whole = false;
        if self.look != Some(look) || self.still.is_none() {
            // (The thread's, waited for if it is not done, if it is still the look; else now.)
            let warmed = match self.warming.take() {
                Some((l, h)) if l == look => h.join().ok().flatten(),
                _ => None,
            };
            let Some(still) = warmed.or_else(|| make_still(&StillOf::of(panel)?, &self.options))
            else {
                return false;
            };
            self.frame = still.clone();
            self.still = Some(still);
            self.look = Some(look);
            self.names = Default::default();
            self.jacks = Default::default();
            self.knobs = Default::default();
            self.shown = None;
            whole = true;
        }
        if self.shown.as_ref() == Some(scene) {
            return false;
        }
        let mut dirty = Vec::new();
        for (n, cell) in scene.cells.iter().enumerate() {
            dirty.extend(self.name_for(n, cell, panel));
            dirty.extend(self.jack_for(n, cell, panel));
            dirty.extend(self.knob_for(n, cell, panel));
        }
        self.shown = Some(scene.clone());
        let all = [0, 0, self.frame.width() as i32, self.frame.height() as i32];
        let places = if whole {
            vec![all]
        } else {
            let mut v: Vec<[i32; 4]> = dirty.into_iter().filter_map(|r| within(r, all)).collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        if places.is_empty() {
            return false;
        }
        for &r in &places {
            self.paint(r);
        }
        self.damages = places;
        true
    }

    /// The frame's pixels of a rectangle of the drawing's units (left, top, right, bottom),
    /// every pixel it touches and one more round it.
    fn pixels(&self, panel: &Renderer, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> [i32; 4] {
        let (s, top) = (panel.scale(), panel.top());
        let r = panel.left_rect();
        [
            (x0 * s).floor() as i32 - 1 - r[0],
            ((y0 - top) * s).floor() as i32 - 1 - r[1],
            (x1 * s).ceil() as i32 + 1 - r[0],
            ((y1 - top) * s).ceil() as i32 + 1 - r[1],
        ]
    }

    /// Macro `n`'s name drawn again if it changed: where it was and is.
    fn name_for(&mut self, n: usize, cell: &Cell, panel: &Renderer) -> Vec<[i32; 4]> {
        let same = |c: &Cell| {
            c.colour == cell.colour
                && c.name == cell.name
                && c.dim == cell.dim
                && c.typing == cell.typing
        };
        if self.names[n].as_ref().is_some_and(|(c, _)| same(c)) {
            return Vec::new();
        }
        let Some(still) = &self.still else {
            return Vec::new();
        };
        let (x, y, w, h) = name_at(n);
        let r = self.pixels(panel, (x - 6.0, y - 6.0, x + w + 6.0, y + h + 6.0));
        let Some(mut p) = crop(still, r) else {
            return Vec::new();
        };
        let s = panel.scale();
        let (ox, oy) = origin(panel, r);
        let text = cell.typing.as_deref().unwrap_or(&cell.name);
        let colour = if cell.dim && cell.typing.is_none() {
            mix(cell.colour, [0; 3], 0.55)
        } else {
            cell.colour
        };
        name_dots(
            &mut p,
            s,
            (x - ox, y - oy, w, h),
            text,
            colour,
            cell.typing.is_some(),
        );
        if cell.typing.is_some() {
            let mut o = Svg::default();
            put!(
                o,
                "<rect x='{}' y='{}' width='{}' height='{}' rx='5' fill='none' stroke='{}' stroke-width='2.4'/>",
                N(x - ox - 4.0),
                N(y - oy - 4.0),
                N(w + 8.0),
                N(h + 8.0),
                crate::learn::ACCENT
            );
            svg_on(&self.options, &mut p, s, &o.0);
        }
        let patch = Patch {
            at: (r[0], r[1]),
            pixels: p,
        };
        let mut out = vec![patch.rect()];
        if let Some((_, old)) = self.names[n].replace((cell.clone(), patch)) {
            out.push(old.rect());
        }
        out
    }

    /// Macro `n`'s collar and jack drawn again if its colour changed: where they were and are.
    fn jack_for(&mut self, n: usize, cell: &Cell, panel: &mut Renderer) -> Vec<[i32; 4]> {
        let key = (cell.colour, cell.dim);
        if self.jacks[n].as_ref().is_some_and(|(k, _)| *k == key) {
            return Vec::new();
        }
        let Some(still) = &self.still else {
            return Vec::new();
        };
        let (jx, jy) = jack_at(n);
        let m = COLLAR + 3.0;
        let r = self.pixels(panel, (jx - m, jy - m, jx + m, jy + m));
        let Some(mut p) = crop(still, r) else {
            return Vec::new();
        };
        let s = panel.scale();
        let (ox, oy) = origin(panel, r);
        // The collar: a washer in the macro's colour under the jack's nut, its edge catching the
        // lamp above and in shade below.
        let col = if cell.dim {
            mix(cell.colour, [40; 3], 0.45)
        } else {
            cell.colour
        };
        let (x, y) = (jx - ox, jy - oy);
        let mut o = Svg::default();
        put!(
            o,
            "<circle cx='{}' cy='{}' r='{}' fill='{}'/><circle cx='{}' cy='{}' r='{}' fill='none' stroke='#fff' stroke-opacity='0.35' stroke-width='1.2'/><circle cx='{}' cy='{}' r='{}' fill='none' stroke='#000' stroke-opacity='0.5' stroke-width='1.2'/>",
            N(x),
            N(y),
            N(COLLAR),
            hex(col),
            N(x - 0.6),
            N(y - 0.6),
            N(COLLAR - 0.7),
            N(x + 0.4),
            N(y + 0.6),
            N(COLLAR)
        );
        svg_on(&self.options, &mut p, s, &o.0);
        let worn = panel.skin() == Skin::Worn;
        if let Some((px, py, j)) = panel.paint_layer(&art::jack_layer(jx, jy, JACK_R, worn)) {
            let at = in_frame(panel, (px, py));
            lay(
                &mut p,
                &j,
                (at.0 - r[0], at.1 - r[1]),
                [0, 0, i32::MAX, i32::MAX],
            );
        }
        let patch = Patch {
            at: (r[0], r[1]),
            pixels: p,
        };
        let mut out = vec![patch.rect()];
        if let Some((_, old)) = self.jacks[n].replace((key, patch)) {
            out.push(old.rect());
        }
        out
    }

    /// Macro `n`'s knob drawn again if it turned: where it was and is.
    fn knob_for(&mut self, n: usize, cell: &Cell, panel: &mut Renderer) -> Vec<[i32; 4]> {
        let key = cell.value.to_bits();
        if self.knobs[n].as_ref().is_some_and(|(k, _)| *k == key) {
            return Vec::new();
        }
        let Some((x, y, p)) = panel.paint_control(MACRO_FIRST + n, cell.value) else {
            return Vec::new();
        };
        let patch = Patch {
            at: in_frame(panel, (x, y)),
            pixels: p,
        };
        let mut out = vec![patch.rect()];
        if let Some((_, old)) = self.knobs[n].replace((key, patch)) {
            out.push(old.rect());
        }
        out
    }

    /// The frame drawn again inside `r`: the still, each name, collar and jack, then each knob.
    fn paint(&mut self, r: [i32; 4]) {
        let Some(still) = &self.still else {
            return;
        };
        glow::copy_in(&mut self.frame, still, (0, 0), r);
        for (_, p) in self.names.iter().flatten() {
            glow::copy_in(&mut self.frame, &p.pixels, p.at, r);
        }
        for (_, p) in self.jacks.iter().flatten() {
            glow::copy_in(&mut self.frame, &p.pixels, p.at, r);
        }
        for (_, p) in self.knobs.iter().flatten() {
            lay(&mut self.frame, &p.pixels, p.at, r);
        }
    }
}

/// The look `panel` draws in, as the still is made for it.
fn look_of(panel: &Renderer) -> Look {
    (
        panel.scale().to_bits(),
        panel.skin(),
        panel.lettering(),
        panel.left_rect(),
    )
}

/// A place in the drawing's pixels (as `Renderer::paint_control` gives it) in the area's frame's.
fn in_frame(panel: &Renderer, (x, y): (i32, i32)) -> (i32, i32) {
    let r = panel.left_rect();
    (x - r[0], y - panel.top_px() - r[1])
}

/// The drawing's units of a patch's top left corner, `r` in the area's frame's pixels.
fn origin(panel: &Renderer, r: [i32; 4]) -> (f64, f64) {
    let (s, a) = (panel.scale(), panel.left_rect());
    (
        f64::from(r[0] + a[0]) / s,
        panel.top() + f64::from(r[1] + a[1]) / s,
    )
}

/// `r` cut to `b`; none if nothing of it is inside.
fn within(r: [i32; 4], b: [i32; 4]) -> Option<[i32; 4]> {
    let c = [
        r[0].max(b[0]),
        r[1].max(b[1]),
        r[2].min(b[2]),
        r[3].min(b[3]),
    ];
    (c[2] > c[0] && c[3] > c[1]).then_some(c)
}

fn mix(a: [u8; 3], b: [u8; 3], k: f64) -> [u8; 3] {
    std::array::from_fn(|i| {
        (f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * k).round() as u8
    })
}

/// `body` (a patch's units: the drawing's, from its top left corner) drawn over `p`.
fn svg_on(options: &usvg::Options<'_>, p: &mut Pixmap, s: f64, body: &str) {
    let view = [
        0.0,
        0.0,
        f64::from(p.width()) / s,
        f64::from(p.height()) / s,
    ];
    let doc = art::document(view, p.width(), p.height(), body);
    if let Ok(tree) = usvg::Tree::from_str(&doc, options) {
        resvg::render(&tree, Transform::identity(), &mut p.as_mut());
    }
}

/// A name's dots in its screen at `window` (the patch's units): every place's dots faintly in
/// its colour, its characters' lit with their glow, centred on the dot grid; a caret's dots
/// after them while it is typed.
fn name_dots(
    p: &mut Pixmap,
    s: f64,
    (x, y, w, h): (f64, f64, f64, f64),
    text: &str,
    c: [u8; 3],
    caret: bool,
) {
    let places = ((w - 8.0) / (6.0 * DOT)).floor().max(1.0) as usize;
    let glyphs: Vec<[u8; 9]> = text.chars().take(places).map(font::glyph).collect();
    let n = (glyphs.len() + usize::from(caret)).min(places);
    let mut shown = places;
    if shown > 1 && !(shown - n).is_multiple_of(2) {
        shown -= 1;
    }
    let start = (shown - n) / 2;
    let left = x + (w - (6 * shown - 1) as f64 * DOT) / 2.0;
    let top = y + h / 2.0 - 0.4 * DOT - 3.0 * DOT;
    let r = (DOT * 0.36 * s) as f32;
    let (mut unlit, mut halo, mut core) =
        (PathBuilder::new(), PathBuilder::new(), PathBuilder::new());
    for i in 0..shown {
        for j in 0..9 {
            for col in 0..5 {
                let px = ((left + (6 * i + col) as f64 * DOT) * s) as f32;
                let py = ((top + j as f64 * DOT) * s) as f32;
                unlit.push_circle(px, py, r);
                let k = i.wrapping_sub(start);
                let lit = glyphs.get(k).is_some_and(|g| g[j] >> (4 - col) & 1 == 1)
                    || (caret && k == glyphs.len() && j == 7);
                if lit {
                    halo.push_circle(px, py, r * 2.1);
                    core.push_circle(px, py, r);
                }
            }
        }
    }
    let t = |c: [u8; 3]| (c[0], c[1], c[2]);
    fill(p, unlit, t(c), 0.07);
    fill(p, halo, t(mix(c, [255, 40, 0], 0.25)), 0.2);
    fill(p, core, t(c), 1.0);
}

/// The area's still: its bare face (the panel's renderer's, where the left hand's was), the
/// rail's shade across its top (the worn skin's), the knobs' and jacks' shadows, the dials' print
/// worn into the face, the names' screens, the lamp over all.
fn make_still(of: &StillOf, options: &usvg::Options<'_>) -> Option<Pixmap> {
    let mut plate = of.face.clone();
    let (s, r) = (of.s, of.rect);
    let (ox, oy) = (f64::from(r[0]) / s, of.top + f64::from(r[1]) / s);
    let worn = of.skin == Skin::Worn;
    let g = |body: &str| format!("<g transform='translate({} {})'>{body}</g>", N(-ox), N(-oy));
    if worn {
        let mut u = Svg::default();
        put!(
            u,
            "<defs><linearGradient id='under-wood' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.6'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient><filter id='long' x='-0.6' y='-0.6' width='2.2' height='2.2'><feGaussianBlur stdDeviation='7'/></filter><filter id='softer' x='-0.5' y='-0.5' width='2' height='2'><feGaussianBlur stdDeviation='2'/></filter></defs><rect x='0' y='{}' width='{}' height='14' fill='url(#under-wood)'/>",
            N(AREA.1),
            N(AREA.2 + 10.0)
        );
        for n in 0..MACROS {
            let (x, y) = knob_at(n);
            let rr = art::STD.skirt;
            put!(
                u,
                "<circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.55' filter='url(#long)'/><circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.7' filter='url(#softer)'/>",
                N(x + 10.0),
                N(y + 14.0),
                N(rr),
                N(x + 2.0),
                N(y + 3.0),
                N(rr)
            );
            let (jx, jy) = jack_at(n);
            put!(
                u,
                "<circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.5' filter='url(#softer)'/>",
                N(jx + 2.0),
                N(jy + 3.0),
                N(COLLAR + 3.0)
            );
        }
        svg_on(options, &mut plate, s, &g(&u.0));
    }
    // The dials' print, worn into the face as the panel's is.
    let mut t = Svg(String::new(), of.lettering);
    for c in CONTROLS.iter().skip(MACRO_FIRST).take(MACROS) {
        if let Kind::Knob { dial, .. } = c.kind {
            art::dial_print(&mut t, c.x, c.y, dial, &of.times);
        }
    }
    if let Some(mut print) = Pixmap::new(plate.width(), plate.height()) {
        svg_on(options, &mut print, s, &g(&t.0));
        if worn {
            crate::render::worn_into(&mut print, &plate);
        }
        plate.draw_pixmap(
            0,
            0,
            print.as_ref(),
            &PixmapPaint {
                opacity: if worn { 0.94 } else { 1.0 },
                ..PixmapPaint::default()
            },
            Transform::identity(),
            None,
        );
    }
    // The names' screens: bezel and glass, as the panel's other dot screens.
    let glass = Pixmap::decode_png(crate::skin::GLASS).ok()?;
    for n in 0..MACROS {
        let (x, y, w, h) = name_at(n);
        screen_still(options, &mut plate, &glass, s, (x - ox, y - oy, w, h));
    }
    if worn {
        crate::render::lamp_in(&mut plate, s, (r[0], r[1]), of.height);
    }
    Some(plate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Scene;

    fn scene() -> MacroScene {
        let colours = [
            [0x14, 0xbb, 0xc2],
            [0xb8, 0x7e, 0x99],
            [0xf9, 0xfb, 0x70],
            [0x8a, 0xb1, 0x81],
            [0xc9, 0xc2, 0xfc],
            [0xdc, 0xf8, 0xfa],
        ];
        let names = ["BRIGHT", "GROWL", "SPACE", "WOBBLE", "SWELL", "MACRO 6"];
        let values = [0.55, 0.3, 0.0, 0.2, 0.7, 0.0];
        MacroScene {
            cells: std::array::from_fn(|n| Cell {
                colour: colours[n],
                name: names[n].into(),
                value: values[n],
                dim: n == 5,
                typing: None,
            }),
        }
    }

    /// The area drawn whole is the area drawn in parts: a knob turned, a name typed, a colour
    /// changed each draw only their patches, and the frame is then as if drawn whole; in both
    /// skins.
    #[test]
    fn drawn_in_parts_it_is_drawn_whole() {
        for skin in [Skin::Worn, Skin::Drawn] {
            let mut panel = Renderer::with_skin(skin, 0.3, 1.0);
            panel.render(&Scene::default());
            let mut m = MacroRenderer::new();
            assert!(m.render(&scene(), &mut panel));
            assert!(!m.render(&scene(), &mut panel), "nothing changed");
            let mut next = scene();
            next.cells[1].value = 0.9;
            next.cells[3].typing = Some("WOB".into());
            next.cells[4].colour = [0xbf, 0x58, 0xfd];
            assert!(m.render(&next, &mut panel));
            assert_eq!(m.damages().len(), 4, "{:?}", m.damages());
            let mut whole = MacroRenderer::new();
            whole.render(&next, &mut panel);
            assert!(m.frame().data() == whole.frame().data(), "{skin:?}");
        }
    }

    /// Pictures of the area over the panel's frame (`CA72_MACRO_PNGS=<dir>`), at the opening
    /// size and at 1.0, in both skins.
    #[test]
    #[ignore]
    fn the_macro_pngs() {
        let dir = std::path::PathBuf::from(std::env::var("CA72_MACRO_PNGS").expect("a folder"));
        std::fs::create_dir_all(&dir).expect("the folder");
        for skin in [Skin::Worn, Skin::Drawn] {
            for s in [0.44, 1.0] {
                let mut panel = Renderer::with_skin(skin, s, 1.0);
                panel.render(&Scene::default());
                let mut m = MacroRenderer::new();
                let mut sc = scene();
                sc.cells[3].typing = Some("WOB".into());
                m.render(&sc, &mut panel);
                let mut f = panel.frame().clone();
                let r = panel.left_rect();
                crate::render::put(&mut f, m.frame(), (r[0], r[1]), 1.0);
                let crop = crate::glow::crop(
                    &f,
                    [
                        0,
                        (art::PANEL_H * s) as i32,
                        (420.0 * s) as i32,
                        f.height() as i32,
                    ],
                )
                .expect("the left area");
                crop.save_png(dir.join(format!("left-{skin:?}-{s}.png")))
                    .expect("written");
            }
        }
    }

    /// Round 2's budget: the cells where it measured them, inside the area, apart; every name,
    /// jack and knob found where it is drawn.
    #[test]
    fn the_cells_are_round_twos_and_found_where_drawn() {
        assert_eq!(knob_at(0), (102.0, 1183.0));
        assert_eq!(knob_at(5), (274.0, 1643.0));
        assert_eq!(name_at(2), (28.0, 1307.0, 148.0, 28.0));
        assert_eq!(jack_at(3), (274.0, 1493.0));
        for n in 0..MACROS {
            let (x, y, w, h) = name_at(n);
            assert!(x >= AREA.0 && x + w <= AREA.2 && y >= AREA.1 + 16.0);
            assert!(jack_at(n).1 + COLLAR <= AREA.3 - 10.0);
            assert_eq!(hit(x + w / 2.0, y + h / 2.0), Some(MacroTarget::Name(n)));
            let (jx, jy) = jack_at(n);
            assert_eq!(hit(jx, jy), Some(MacroTarget::Jack(n)));
            let (kx, ky) = knob_at(n);
            assert_eq!(hit(kx, ky), None, "the knob is the panel's control");
            assert!(inside(kx, ky));
            let c = CONTROLS[MACRO_FIRST + n];
            assert_eq!(c.centre(), (kx, ky));
            assert_eq!(
                c.param,
                ["macro1", "macro2", "macro3", "macro4", "macro5", "macro6"][n]
            );
        }
        // Seven characters a name.
        let places = ((NAME_W - 8.0) / (6.0 * DOT)).floor() as usize;
        assert_eq!(places, NAME_PLACES);
    }
}
