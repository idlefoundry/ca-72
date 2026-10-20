//! The strip's own parts, drawn over the drawing (A6): the tabs, lit amber from inside with
//! their light on the face round them (`plugin_kit_materials`, as the CA-74's buttons that
//! light), the readouts' seven segments and the voices' display behind their glass, the rail's
//! keys, and the preset's name in dots. The light is the panel's neon orange. What does not
//! change is drawn once a scale, and laid over the drawing's strip again only when that
//! changes; a readout drawn again only when what it reads does; the drops of the voices'
//! display each frame while they move (as the CA-74's, its R36), and then only their window.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Instant;

use plugin_kit_materials::{
    Glow, Lighting, Tint, Underlight, fill, font, glass, place, sprite, widened,
};
use resvg::tiny_skia::{PathBuilder, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use super::{
    Bank, DISPLAY, Field, KEY_HEIGHT, Rail, Readout, StripControl, StripScene, StripTarget, TAB,
    blank, span,
};
use crate::art::{self, PANEL_H};
use crate::fonts::{FAMILY, Fonts};
use crate::presets::{BarScene, BarTarget, DOWN, STAR, UP};
use crate::skin::{BUTTON, GLASS};
use crate::svg::{N, Svg, put};

/// The panel's neon orange (the mock-up's): lit segments, dots and drops; the glow round a
/// lit dot (the mock-up's); the drops' light (the mock-up's, added in).
pub(super) const ORANGE: (u8, u8, u8) = (255, 112, 40);
pub(super) const ORANGE_GLOW: (u8, u8, u8) = (255, 72, 10);
const DROP_LIGHT: (f64, f64, f64) = (172.0, 80.0, 30.0);
const SEG_OFF_OPACITY: f64 = 0.05;

/// What the strip lights its readouts, displays and drops with: the panel's neon orange, or a
/// look's colour (decisions.md R-SKIN-PACKS), its glow and the drops' light made from it as the
/// orange's are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Lamp {
    pub(crate) on: (u8, u8, u8),
    pub(crate) glow: (u8, u8, u8),
    drop: (f64, f64, f64),
}

impl Lamp {
    pub(crate) const ORANGE: Lamp = Lamp {
        on: ORANGE,
        glow: ORANGE_GLOW,
        drop: DROP_LIGHT,
    };

    /// A look's lamp of colour `c`.
    pub(crate) fn of((r, g, b): (u8, u8, u8)) -> Lamp {
        let k = |v: u8, a: f64, o: f64| f64::from(v) * a / o;
        let glow =
            |v: u8, a: u8, o: u8| (k(v, f64::from(a), f64::from(o))).round().min(255.0) as u8;
        Lamp {
            on: (r, g, b),
            glow: (
                glow(r, ORANGE_GLOW.0, ORANGE.0),
                glow(g, ORANGE_GLOW.1, ORANGE.1),
                glow(b, ORANGE_GLOW.2, ORANGE.2),
            ),
            drop: (
                k(r, DROP_LIGHT.0, f64::from(ORANGE.0)),
                k(g, DROP_LIGHT.1, f64::from(ORANGE.1)),
                k(b, DROP_LIGHT.2, f64::from(ORANGE.2)),
            ),
        }
    }

    /// Its lit segments' colour, `#rrggbb`.
    fn seg(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.on.0, self.on.1, self.on.2)
    }
}
/// The tabs: translucent amber plastic lit from inside (the CA-74's amber, tried there: lit
/// `[1.25, 0.55 g^1.6, 0.08 g^3]` of the glow `g`, unlit `[0.26, 0.062, 0.003]` of the shade),
/// and their light on the face round them. The glow over `TAB_KNEE` pressed down to a quarter
/// of itself, and the light round them a little over half the CA-74's and all one orange (the
/// owner, 2026-10-10: "these orange buttons get too bright on the edges. looks unnatural": the
/// cap's walls, which carry the light to its edge, and the bevel's catch light had gone a hot
/// yellow-white round it, brighter than its face).
pub(super) const AMBER: Tint = Tint {
    lit: |g| {
        let g = if g > TAB_KNEE {
            TAB_KNEE + (g - TAB_KNEE) * 0.25
        } else {
            g
        };
        [1.25 * g, 0.55 * g.powf(1.6), 0.08 * g * g * g]
    },
    unlit: |k| [0.26 * k, 0.062 * k, 0.003 * k],
};
const TAB_KNEE: f32 = 0.8;
/// How much darker a lit tab's outermost band is made (its edge, where the cap's walls carry
/// the light, a bright line round a darker band inside it; at the edge this share darker,
/// easing in from `EDGE_FROM` of the way out).
const EDGE_DARKER: f32 = 0.4;
const EDGE_FROM: f32 = 0.86;
/// And how much darker its corners are, the rim line round them too (the owner, 2026-10-10:
/// "should the corners of these buttons be darker, not lighter", "that's where the plastic
/// would be densist from the user's perspective"): this share at the corner itself, easing in
/// where the cap's sides meet its ends, from `CORNER_FROM` of the way out both ways.
const CORNER_DARKER: f32 = 0.75;
const CORNER_FROM: f32 = 0.3;

/// The lit cap with its outermost band darkened ([`EDGE_DARKER`]) and its corners more
/// ([`CORNER_DARKER`]), in linear light, as far out as the kit's cap measures it (its rounded
/// square, its middle drawn out across).
pub(super) fn edge_eased(mut l: plugin_kit_materials::Lighting) -> plugin_kit_materials::Lighting {
    let lin = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let enc = |v: f32| {
        let v = v.clamp(0.0, 1.0);
        let s = if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round() as u8
    };
    let (w, h) = (l.lit.width(), l.lit.height());
    let (half, long) = (h as f32 / 2.0, (w as f32 - h as f32).max(0.0) / 2.0);
    for (i, px) in l.lit.pixels_mut().iter_mut().enumerate() {
        let c = px.demultiply();
        if c.alpha() == 0 {
            continue;
        }
        let (x, y) = ((i as u32 % w) as f32 + 0.5, (i as u32 / w) as f32 + 0.5);
        let u = ((x - w as f32 / 2.0).abs() - long).max(0.0) / half;
        let v = (y - half).abs() / half;
        let d = (u * u * u * u + v * v * v * v).sqrt().sqrt().min(1.0);
        let t = ((d - EDGE_FROM) / (1.0 - EDGE_FROM)).clamp(0.0, 1.0);
        let ease = |z: f32| {
            let z = ((z - CORNER_FROM) / (1.0 - CORNER_FROM)).clamp(0.0, 1.0);
            z * z * (3.0 - 2.0 * z)
        };
        let corner = ease(u.min(1.0)) * ease(v.min(1.0));
        let k = (1.0 - EDGE_DARKER * t * t * (3.0 - 2.0 * t)) * (1.0 - CORNER_DARKER * corner);
        if k < 1.0 {
            let f = |v: u8| enc(lin(v) * k);
            *px = resvg::tiny_skia::ColorU8::from_rgba(
                f(c.red()),
                f(c.green()),
                f(c.blue()),
                c.alpha(),
            )
            .premultiply();
        }
    }
    l
}
/// A cap made the tabs' translucent amber, unlit and lit, drawn out to `aspect` (width over
/// height): an oscillator's TIMED key's (decisions.md R-OSC-TIMED), as the tabs are.
pub(crate) fn amber_caps(cap: &Pixmap, aspect: f64) -> Lighting {
    edge_eased(plugin_kit_materials::lighting(cap, aspect, AMBER))
}

pub(super) const AMBER_LIGHT: Underlight = Underlight {
    rim: [0.6, 0.2, 0.04],
    light: [0.6, 0.17, 0.03],
};
/// The tabs' and the keys' print.
pub(super) const TAB_PRINT: &str = "#2a1006";
const KEY_PRINT: &str = "#d9d2c0";
/// The drops move on at most this far apart, seconds (30 a second).
const STEP_S: f64 = 1.0 / 30.0;
/// The name's dots' pitch, and the drops' sizes (a sounding voice's, an idle one's: the
/// mock-up's, the owner, 2026-10-10: "I want it to go back to the red pictured here"), units.
pub(super) const DOT: f64 = 5.0;
const DROP: f64 = 25.0;
const IDLE_DROP: f64 = 15.0;
/// Where a capital's middle is, above the name's window's middle: a capital stands in the
/// window's middle, as near as the descenders below it leave room for (the owner: "the preset
/// text needs to be vertically centered").
pub(super) const CAP_LIFT: f64 = 2.0;

/// The strip's parts at a scale: its frame, what never changes, the caps and light resampled,
/// the readouts as last drawn, the drops.
pub struct StripRenderer {
    scale: f64,
    options: usvg::Options<'static>,
    frame: Pixmap,
    still: Option<Sparse>,
    print: Option<Sparse>,
    /// The drawing's strip with the still parts over it.
    base: Option<Pixmap>,
    lighting: Lighting,
    glass: Pixmap,
    button: Pixmap,
    lit_tab: Option<Pixmap>,
    glows: Vec<((f64, f64), Glow)>,
    readouts: Vec<Option<Reading>>,
    liquid: Liquid,
    last: Option<Instant>,
    moved: bool,
    shown: Option<StripScene>,
    /// The pixels the last render changed: left, top, right, bottom (the last two past the
    /// change).
    damage: Option<[i32; 4]>,
    /// Every frame drawn whole (the tests' reference for one drawn in parts).
    whole: bool,
    keyboard: crate::keyboard::Renderer,
    /// The drawing's row the frame it is drawn over begins at (the panel renderer's
    /// `top_px`: 0 for the official panel, less for a taller one).
    under_top: i32,
    /// What it lights with, and its tabs' and keys' print (a look's, else the built-in).
    lamp: Lamp,
    tab_print: String,
    key_print: String,
    /// The tabs lit from a look's own picture of them lit: no light spilt round them.
    own_lit: bool,
    /// The rail's keys: the plug-ins', or the standalone's with FX (decisions.md R-FX-DRAWER).
    rail: Rail,
}

impl fmt::Debug for StripRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StripRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

/// A readout drawn: what it reads, lit or dark, and the patch of the frame it covers.
pub(super) struct Reading {
    pub(super) text: String,
    pub(super) lit: bool,
    pub(super) left: i32,
    pub(super) top: i32,
    pub(super) patch: Pixmap,
}

impl StripRenderer {
    /// At `scale` (pixels a drawing unit), its pictures decoded.
    pub fn new(scale: f64) -> Self {
        let png = |b: &[u8]| Pixmap::decode_png(b).expect("a built-in picture decodes");
        let button = png(BUTTON);
        let fonts = Fonts::new();
        StripRenderer {
            scale,
            options: usvg::Options {
                fontdb: fonts.database(),
                font_family: FAMILY.into(),
                ..usvg::Options::default()
            },
            frame: blank(scale).expect("a frame"),
            keyboard: crate::keyboard::Renderer::new(scale),
            still: None,
            print: None,
            base: None,
            lighting: edge_eased(plugin_kit_materials::lighting(
                &button,
                TAB.0 / TAB.1,
                AMBER,
            )),
            glass: png(GLASS),
            button,
            lit_tab: None,
            glows: Vec::new(),
            readouts: Vec::new(),
            liquid: Liquid::default(),
            last: None,
            moved: false,
            shown: None,
            damage: None,
            whole: false,
            under_top: 0,
            lamp: Lamp::ORANGE,
            tab_print: TAB_PRINT.to_owned(),
            key_print: KEY_PRINT.to_owned(),
            own_lit: false,
            rail: Rail::Plugin,
        }
    }

    /// Drawn in `look`'s pictures, colours and typeface for the strip, where it has them
    /// (decisions.md R-SKIN-PACKS): the tabs' caps unlit and lit, the rail's keys' cap, the
    /// glass, the lamps' colour, the tabs' and keys' print.
    pub fn set_look(&mut self, look: &crate::look::Look) {
        let png = |b: &[u8]| Pixmap::decode_png(b).expect("a built-in picture decodes");
        let size = |name: &str| crate::skin::slot_size(name).unwrap_or((1, 1));
        let pack = |name: &str| look.pictures.get(name).and_then(|p| p.picture(size(name)));
        let button = png(BUTTON);
        self.lighting = match (pack("tab"), pack("tab-lit")) {
            // (A square cap, as the built-in one: widened to the tab's proportions.)
            (Some(unlit), Some(lit)) => plugin_kit_materials::Lighting {
                unlit: widened(&unlit, TAB.0 / TAB.1),
                lit: widened(&lit, TAB.0 / TAB.1),
            },
            (tab, _) => edge_eased(plugin_kit_materials::lighting(
                tab.as_ref().unwrap_or(&button),
                TAB.0 / TAB.1,
                AMBER,
            )),
        };
        self.own_lit = pack("tab-lit").is_some();
        self.button = pack("key").unwrap_or(button);
        self.glass = pack("glass").unwrap_or_else(|| png(GLASS));
        let hex = |(r, g, b): (u8, u8, u8)| format!("#{r:02x}{g:02x}{b:02x}");
        let c = look.colours;
        self.lamp = c.lamp.map_or(Lamp::ORANGE, Lamp::of);
        if let Some((r, g, b)) = c.drops {
            self.lamp.drop = (f64::from(r), f64::from(g), f64::from(b));
        }
        self.tab_print = c.tab_print.map_or(TAB_PRINT.to_owned(), hex);
        self.key_print = c.key_print.map_or(KEY_PRINT.to_owned(), hex);
        let fonts = look
            .fonts
            .and_then(|(regular, bold)| Fonts::with(regular, bold).ok())
            .unwrap_or_default();
        self.options = usvg::Options {
            fontdb: fonts.database(),
            font_family: FAMILY.into(),
            ..usvg::Options::default()
        };
        (self.still, self.print, self.lit_tab) = (None, None, None);
        self.base = None;
        self.glows.clear();
        self.readouts.clear();
        self.shown = None;
        self.keyboard.set_look(look, self.lamp);
    }

    /// Drawn over a panel renderer's frame beginning at the drawing's row `top` (its
    /// `top_px`): the strip is the same, its place in that frame not.
    pub fn set_under_top(&mut self, top: i32) {
        if top != self.under_top {
            self.under_top = top;
            self.base = None;
            self.shown = None;
        }
    }

    /// The rail's keys `rail`'s from now on (its still parts drawn again for them).
    pub fn set_rail(&mut self, rail: Rail) {
        if rail != self.rail {
            self.rail = rail;
            (self.still, self.print, self.base) = (None, None, None);
            self.shown = None;
        }
    }

    pub fn rail(&self) -> Rail {
        self.rail
    }

    /// Each key of the rail: its middle, width and target.
    fn keys(&self) -> impl Iterator<Item = (f64, f64, BarTarget)> + use<> {
        self.rail.keys().iter().copied()
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Drawn again at `scale`.
    pub fn rescale(&mut self, scale: f64) {
        if scale != self.scale {
            self.scale = scale;
            self.keyboard.rescale(scale);
            if let Some(f) = blank(scale) {
                self.frame = f;
            }
            (self.still, self.print, self.lit_tab) = (None, None, None);
            self.base = None;
            self.glows.clear();
            self.readouts.clear();
            self.shown = None;
        }
    }

    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// Whether the drops are still moving (draw again next frame).
    pub fn animating(&self) -> bool {
        self.moved
    }

    /// The pixels of the frame the last render changed: left, top, right, bottom (the last two
    /// past the change).
    pub fn damage(&self) -> Option<[i32; 4]> {
        self.damage
    }

    /// Drawn for `scene` over the drawing's strip (`under`: the panel's renderer's frame, the
    /// whole drawing at this scale; `changed`, the part of it changed since the last call, in
    /// its pixels: left, top, right, bottom): whether the frame changed. (Over the drawing
    /// itself, not apart from it: a lit tab's light lights the face round it.) Only the part of
    /// the frame where something changed is drawn again.
    pub fn render(
        &mut self,
        scene: &StripScene,
        under: &Pixmap,
        changed: Option<[i32; 4]>,
    ) -> bool {
        // (The drops move on at most [`STEP_S`] apart on this clock, the wall's.)
        let now = Instant::now();
        let at = match self.last {
            Some(t) if now.saturating_duration_since(t).as_secs_f64() < STEP_S => t,
            _ => now,
        };
        self.render_at(scene, under, changed, at)
    }

    /// As [`StripRenderer::render`], the drops eased to `now`, the caller's clock: they move on
    /// each time it has moved on (the editor's ticks thirty times a second with QUALITY's
    /// opening, on baseview's 15 ms frames 30 or 45 ms apart).
    pub fn render_at(
        &mut self,
        scene: &StripScene,
        under: &Pixmap,
        changed: Option<[i32; 4]>,
        now: Instant,
    ) -> bool {
        // (The drops move on as the clock does: they follow the voices, and are drawn no
        // oftener than the rest that does.)
        let dt = self
            .last
            .map_or(0.0, |t| now.saturating_duration_since(t).as_secs_f64());
        let moved = if scene.keyboard.is_some() {
            self.last = Some(now);
            false
        } else if self.last.is_none() || dt > 0.0 {
            self.last = Some(now);
            self.liquid.step(scene, dt.min(0.1)).0
        } else {
            false
        };
        self.moved = moved;
        if self.still.is_none() {
            self.make_still();
        }
        let all = [0, 0, self.frame.width() as i32, self.frame.height() as i32];
        // Where the frame is to be drawn again: where the drawing under it changed (the kept
        // drawing with the still parts over it brought up to it there), what the scene shows
        // differently, the drops' window while they move. (The drawing's change cut to the
        // strip first: a change above it, QUALITY's opening's, made the strip from the top of
        // its window over, almost all of it, at every tick.)
        let top = (PANEL_H * self.scale).round() as i32 - self.under_top;
        let mut dirty = None;
        if self.whole || self.base.is_none() {
            self.make_base(under, all);
            dirty = Some(all);
        } else if let Some(r) =
            changed.and_then(|[x0, y0, x1, y1]| within([x0, y0 - top, x1, y1 - top], all))
        {
            self.make_base(under, r);
            dirty = Some(r);
        }
        match &self.shown {
            None => dirty = Some(all),
            Some(was) => {
                for r in self.changes(was, scene) {
                    dirty = Some(union(dirty, r));
                }
            }
        }
        if moved {
            dirty = Some(union(dirty, self.window()));
        }
        self.shown = Some(scene.clone());
        let Some(r) = dirty.and_then(|r| within(r, all)) else {
            self.damage = None;
            return false;
        };
        let r = within(self.closed(scene, r), all).unwrap_or(r);
        self.paint(scene, r);
        self.damage = Some(r);
        true
    }

    /// The display's window, inside its glass, in the frame's pixels: left, top, right, bottom.
    fn window(&self) -> [i32; 4] {
        let s = self.scale;
        let (dx, dy, dw, dh) = DISPLAY;
        [
            ((dx + 4.0) * s) as i32,
            ((dy - PANEL_H + 4.0) * s) as i32,
            ((dx + dw - 4.0) * s).ceil() as i32,
            ((dy - PANEL_H + dh - 4.0) * s).ceil() as i32,
        ]
    }

    /// The voices' drops, behind the display's glass, in its `window`.
    fn drops(&mut self, window: [i32; 4], detune: f64) {
        let s = self.scale;
        let (dx, dy, dw, dh) = DISPLAY;
        self.liquid.draw(
            &mut self.frame,
            window,
            ((dx + dw / 2.0) * s, (dy - PANEL_H + dh / 2.0) * s),
            (dw / 2.0 - 40.0) * s,
            s,
            detune,
            self.lamp.drop,
        );
    }

    /// The drawing's strip, its rows from the panel's foot down, with what never changes over
    /// it, brought up to `under` inside `rect` (the strip's pixels).
    fn make_base(&mut self, under: &Pixmap, rect: [i32; 4]) {
        let s = self.scale;
        if self.base.is_none() {
            self.base = blank(s);
        }
        let Some(base) = &mut self.base else {
            return;
        };
        let (fw, fh) = (base.width() as i32, base.height() as i32);
        let top = (PANEL_H * s).round() as i32 - self.under_top;
        let (uw, uh) = (under.width() as i32, under.height() as i32);
        let Some([x0, y0, x1, y1]) = within(rect, [0, 0, fw.min(uw), fh.min((uh - top).max(0))])
        else {
            return;
        };
        let n = (x1 - x0) as usize * 4;
        for y in y0..y1 {
            let from = (((top + y) * uw + x0) * 4) as usize;
            let to = ((y * fw + x0) * 4) as usize;
            base.data_mut()[to..to + n].copy_from_slice(&under.data()[from..from + n]);
        }
        if let Some(still) = &self.still {
            still.over(base, [x0, y0, x1, y1]);
        }
    }

    /// A rectangle of the drawing's units (left, top, right, bottom; the drawing's y) as the
    /// frame's pixels, every pixel it touches and one more round it.
    fn pixels(&self, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> [i32; 4] {
        let s = self.scale;
        [
            (x0 * s).floor() as i32 - 1,
            ((y0 - PANEL_H) * s).floor() as i32 - 1,
            (x1 * s).ceil() as i32 + 1,
            ((y1 - PANEL_H) * s).ceil() as i32 + 1,
        ]
    }

    /// Where a tab at `at` draws, lit: its cap and its light round it (the light reaches 38
    /// units, `Glow`).
    fn tab_part(&self, at: (f64, f64)) -> [i32; 4] {
        let m = 40.0;
        self.pixels((
            at.0 - TAB.0 / 2.0 - m,
            at.1 - TAB.1 / 2.0 - m,
            at.0 + TAB.0 / 2.0 + m,
            at.1 + TAB.1 / 2.0 + m,
        ))
    }

    /// Where a readout draws: its window and the digits' glow round it (`reading`).
    fn readout_part(&self, r: Readout) -> [i32; 4] {
        let (x, y, w, h) = r.window();
        let m = 10.0;
        self.pixels((x - m, y - m, x + w + m, y + h + m))
    }

    /// Where the name's dots draw.
    fn name_part(&self) -> [i32; 4] {
        let (x, y, w, h) = self.rail.name();
        self.pixels((x - 8.0, y - 8.0, x + w + 8.0, y + h + 8.0))
    }

    /// Where the signs over everything draw: the name's star and arrow, the favourite's key,
    /// MIDI Learn's ring and the pointer's part.
    fn signs_part(&self, scene: &StripScene) -> [i32; 4] {
        let mut r = union(Some(self.name_part()), self.key_part(BarTarget::Star));
        if let Some(c) = scene.learning {
            r = union(Some(r), self.ring_part(c));
        }
        if let Some(h) = self.hover_part(scene.hover) {
            r = union(Some(r), h);
        }
        r
    }

    /// Where a key of the rail draws.
    fn key_part(&self, t: BarTarget) -> [i32; 4] {
        let (x, w) = self
            .keys()
            .find(|k| k.2 == t)
            .map_or((0.0, 0.0), |(x, w, _)| (x, w));
        let y = super::RAIL_Y;
        self.pixels((
            x - w / 2.0 - 2.0,
            y - KEY_HEIGHT / 2.0 - 2.0,
            x + w / 2.0 + 2.0,
            y + KEY_HEIGHT / 2.0 + 2.0,
        ))
    }

    /// Where MIDI Learn's ring round `c` draws.
    fn ring_part(&self, c: StripControl) -> [i32; 4] {
        let (x0, y0, x1, y1) = span(c);
        self.pixels((x0 - 8.0, y0 - 8.0, x1 + 8.0, y1 + 8.0))
    }

    /// Where the pointer's part is lightened ([`hovered`]).
    fn hover_part(&self, t: Option<StripTarget>) -> Option<[i32; 4]> {
        match t {
            Some(StripTarget::Bar(b)) if b != BarTarget::Name => Some(self.key_part(b)),
            Some(StripTarget::Tab(b, i)) => {
                let (x, y) = b.tab(i);
                Some(self.pixels((
                    x - TAB.0 / 2.0,
                    y - TAB.1 / 2.0,
                    x + TAB.0 / 2.0,
                    y + TAB.1 / 2.0,
                )))
            }
            _ => None,
        }
    }

    /// Where `now` shows something `was` did not (the drops apart: they move by themselves).
    fn changes(&self, was: &StripScene, now: &StripScene) -> Vec<[i32; 4]> {
        let mut out = Vec::new();
        for r in crate::keyboard::damage(was.keyboard.as_ref(), now.keyboard.as_ref()) {
            out.push(self.pixels((r[0], r[1], r[2], r[3])));
        }
        if now.keyboard.is_none() {
            for (b, x, y) in [
                (Bank::Mode, was.mode, now.mode),
                (Bank::Stereo, was.stereo, now.stereo),
                (Bank::Placement, was.placement, now.placement),
                (Bank::Auto, was.auto.then_some(0), now.auto.then_some(0)),
            ] {
                if x != y {
                    out.extend((0..b.words().len()).map(|i| self.tab_part(b.tab(i))));
                }
            }
            for (i, r) in Readout::ALL.into_iter().enumerate() {
                if was.readouts[i] != now.readouts[i] {
                    out.push(self.readout_part(r));
                }
            }
        }
        if was.bar != now.bar {
            out.push(self.name_part());
            out.extend(self.keys().map(|k| self.key_part(k.2)));
        }
        if was.hover != now.hover {
            out.extend(self.hover_part(was.hover));
            out.extend(self.hover_part(now.hover));
        }
        if was.learning != now.learning {
            out.extend(was.learning.map(|c| self.ring_part(c)));
            out.extend(now.learning.map(|c| self.ring_part(c)));
        }
        out
    }

    /// `r` grown until every part drawn over the drawing that reaches into it lies inside it
    /// (each part is drawn whole, so it is drawn again only where it is all drawn again).
    fn closed(&self, scene: &StripScene, mut r: [i32; 4]) -> [i32; 4] {
        let mut parts: Vec<[i32; 4]> = lit_tabs(scene)
            .into_iter()
            .map(|at| self.tab_part(at))
            .collect();
        if scene.keyboard.is_none() {
            parts.extend(Readout::ALL.into_iter().map(|rd| self.readout_part(rd)));
            parts.push(self.window());
        } else {
            parts.clear();
        }
        parts.push(self.name_part());
        parts.push(self.signs_part(scene));
        loop {
            let mut grew = false;
            for &p in &parts {
                if meets(p, r) && union(Some(r), p) != r {
                    r = union(Some(r), p);
                    grew = true;
                }
            }
            if !grew {
                return r;
            }
        }
    }

    /// The frame drawn again inside `r` (the frame's pixels, every part reaching into it lying
    /// inside it, [`StripRenderer::closed`]).
    fn paint(&mut self, scene: &StripScene, r: [i32; 4]) {
        let s = self.scale;
        // The drawing's strip with what never changes over it.
        if let Some(b) = &self.base {
            copy(&mut self.frame, b, r);
        }
        // The lit tabs: their light on the face round them, then the lit cap over the unlit.
        let lit = if scene.keyboard.is_some() {
            Vec::new()
        } else {
            lit_tabs(scene)
        };
        if self.lit_tab.is_none() {
            let px = |v: f64| (v * s).round().max(1.0) as u32;
            self.lit_tab = Some(plugin_kit_materials::resample(
                &self.lighting.lit,
                px(TAB.0),
                px(TAB.1),
            ));
        }
        for &at in &lit {
            if !meets(self.tab_part(at), r) {
                continue;
            }
            let glow = match self.glows.iter().position(|(g, _)| *g == at) {
                Some(i) => Some(&self.glows[i].1),
                None => {
                    if let Some(g) = Glow::new(s, (at.0, at.1 - PANEL_H), TAB, AMBER_LIGHT) {
                        self.glows.push((at, g));
                        self.glows.last().map(|(_, g)| g)
                    } else {
                        None
                    }
                }
            };
            if let Some(g) = glow.filter(|_| !self.own_lit) {
                g.light(&mut self.frame);
            }
            if let Some(cap) = &self.lit_tab {
                place(
                    &mut self.frame,
                    cap,
                    s,
                    (at.0, at.1 - PANEL_H),
                    &PixmapPaint::default(),
                );
            }
        }
        // The readouts.
        if self.readouts.len() != Readout::ALL.len() {
            self.readouts = (0..Readout::ALL.len()).map(|_| None).collect();
        }
        for (i, rd) in Readout::ALL.into_iter().enumerate() {
            if scene.keyboard.is_none() && meets(self.readout_part(rd), r) {
                let (text, on) = &scene.readouts[i];
                reading(self, i, rd, text, *on);
            }
        }
        // The voices' drops, behind the display's glass.
        let window = self.window();
        if scene.keyboard.is_none() && meets(window, r) {
            self.drops(window, scene.detune);
        }
        // The preset's name, its star and arrow (dots); MIDI Learn's ring round the control it
        // waits for; the favourite's key; the pointer's part.
        if meets(self.name_part(), r) {
            name(&mut self.frame, s, self.rail, &scene.bar, &self.lamp);
        }
        if meets(self.signs_part(scene), r) {
            let mut o = Svg::default();
            if let Some(c) = scene.learning.filter(|_| scene.keyboard.is_none()) {
                ring(&mut o, c);
            }
            star_key(&mut o, self.rail, scene.bar.favorite, &self.key_print);
            hovered(&mut o, self.rail, scene.hover);
            svg_over(&self.options, &mut self.frame, s, &o.0);
        }
        // The print over the caps.
        if let Some(p) = &self.print {
            p.over(&mut self.frame, r);
        }
        for (x, width, target) in self.keys() {
            let active = match target {
                BarTarget::Panel => scene.bar.panel,
                BarTarget::Keyboard => scene.bar.keyboard,
                BarTarget::Arp => scene.bar.arp,
                BarTarget::Fx => scene.bar.fx,
                BarTarget::Mpe => scene.bar.mpe,
                _ => false,
            };
            if active && meets(self.key_part(target), r) {
                let mut o = Svg::default();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='7' fill='#b0702b' opacity='.28'/>",
                    N(x - width / 2.0),
                    N(super::RAIL_Y - PANEL_H - super::KEY_H / 2.0),
                    N(width),
                    N(super::KEY_H)
                );
                svg_over(&self.options, &mut self.frame, s, &o.0);
            }
        }
        if let Some(keyboard) = &scene.keyboard {
            self.keyboard.over(&mut self.frame, keyboard, r);
        }
    }

    /// What never changes at this scale: the unlit tabs, the keys' caps, the glass of the
    /// readouts, of the display and of the name, the name's dots unlit; and the print over the
    /// caps, apart.
    fn make_still(&mut self) {
        let s = self.scale;
        let Some(mut still) = blank(s) else {
            return;
        };
        let at = |(x, y): (f64, f64)| (x, y - PANEL_H);
        // The tabs, unlit.
        for b in Bank::ALL {
            for i in 0..b.words().len() {
                sprite(&mut still, &self.lighting.unlit, s, at(b.tab(i)), TAB);
            }
        }
        // The keys: the charcoal cap, drawn out wider for SAVE.
        for (x, w, _) in self.keys() {
            let cap = widened(&self.button, w / KEY_HEIGHT);
            sprite(&mut still, &cap, s, at((x, super::RAIL_Y)), (w, KEY_HEIGHT));
        }
        // The glass.
        for r in Readout::ALL {
            let (x, y, w, h) = r.window();
            glass(&mut still, &self.glass, s, (x, y - PANEL_H, w, h), 5.0);
        }
        let (x, y, w, h) = DISPLAY;
        glass(&mut still, &self.glass, s, (x, y - PANEL_H, w, h), 5.0);
        // The name's: its bezel on the rail's wood, then its glass.
        let mut bezel = Svg::default();
        super::name_bezel(&mut bezel, self.rail, -PANEL_H);
        svg_over(&self.options, &mut still, s, &bezel.0);
        let (x, y, w, h) = self.rail.name();
        glass(&mut still, &self.glass, s, (x, y - PANEL_H, w, h), 3.0);
        // The name's dots, every one faintly there.
        let r = DOT * 0.36 * s;
        let mut unlit = PathBuilder::new();
        name_dots(s, self.rail, |_, _, _, (px, py)| {
            unlit.push_circle(px, py, r as f32);
        });
        fill(&mut still, unlit, self.lamp.on, 0.06);
        self.still = Some(Sparse::new(still));
        // The print: the tabs' words and the keys' marks.
        if let Some(mut print) = blank(s) {
            let mut o = Svg::default();
            for b in Bank::ALL {
                for (i, w) in b.words().iter().enumerate() {
                    let (x, y) = b.tab(i);
                    put!(
                        o,
                        "<text x='{}' y='{}' font-size='17' font-weight='700' text-anchor='middle' dominant-baseline='central' fill='{}' fill-opacity='0.85'>{w}</text>",
                        N(x),
                        N(y - PANEL_H),
                        self.tab_print
                    );
                }
            }
            key_marks(&mut o, self.rail, &self.key_print);
            svg_over(&self.options, &mut print, s, &o.0);
            self.print = Some(Sparse::new(print));
        }
    }
}

/// A layer mostly clear, laid over a frame its size where it has pixels only. (tiny-skia's
/// `draw_pixmap` blends every pixel: the still parts and the print over the whole strip had
/// cost 15 ms a frame at 0.87 of the drawing.)
struct Sparse {
    pixmap: Pixmap,
    /// Its runs of pixels not clear: each from, to (bytes).
    runs: Vec<(usize, usize)>,
}

impl Sparse {
    fn new(pixmap: Pixmap) -> Self {
        let w = pixmap.width() as usize;
        let mut runs = Vec::new();
        for (y, row) in pixmap.pixels().chunks(w.max(1)).enumerate() {
            let mut x = 0;
            while x < row.len() {
                if row[x].alpha() == 0 {
                    x += 1;
                    continue;
                }
                let from = x;
                while x < row.len() && row[x].alpha() != 0 {
                    x += 1;
                }
                runs.push(((y * w + from) * 4, (y * w + x) * 4));
            }
        }
        Sparse { pixmap, runs }
    }

    /// Laid over `frame` (its size) inside `rect` (pixels: left, top, right, bottom), source
    /// over, as tiny-skia's pipeline does (premultiplied, each channel `s + (d (255 - a) +
    /// 255) / 256`).
    fn over(&self, frame: &mut Pixmap, rect: [i32; 4]) {
        if frame.data().len() != self.pixmap.data().len() {
            return;
        }
        let w = self.pixmap.width() as usize;
        let size = [0, 0, w as i32, self.pixmap.height() as i32];
        let Some([x0, y0, x1, y1]) = within(rect, size) else {
            return;
        };
        let (x0, y0, x1, y1) = (x0 as usize, y0 as usize, x1 as usize, y1 as usize);
        let (src, dst) = (self.pixmap.data(), frame.data_mut());
        let first = self.runs.partition_point(|&(from, _)| from < y0 * w * 4);
        for &(from, to) in &self.runs[first..] {
            let y = from / (w * 4);
            if y >= y1 {
                break;
            }
            let row = y * w * 4;
            let (from, to) = (from.max(row + x0 * 4), to.min(row + x1 * 4));
            if from >= to {
                continue;
            }
            for (s, d) in src[from..to]
                .chunks_exact(4)
                .zip(dst[from..to].chunks_exact_mut(4))
            {
                let keep = 255 - u16::from(s[3]);
                for c in 0..4 {
                    d[c] = (u16::from(s[c]) + ((u16::from(d[c]) * keep + 255) >> 8)) as u8;
                }
            }
        }
    }
}

/// `from`'s pixels inside `rect` (left, top, right, bottom) put in `to`, the same size.
pub(super) fn copy(to: &mut Pixmap, from: &Pixmap, rect: [i32; 4]) {
    let (w, h) = (to.width() as i32, to.height() as i32);
    if (from.width() as i32, from.height() as i32) != (w, h) {
        return;
    }
    let Some([x0, y0, x1, y1]) = within(rect, [0, 0, w, h]) else {
        return;
    };
    let n = (x1 - x0) as usize * 4;
    for y in y0..y1 {
        let at = ((y * w + x0) * 4) as usize;
        to.data_mut()[at..at + n].copy_from_slice(&from.data()[at..at + n]);
    }
}

/// `a` and `b` (left, top, right, bottom) together: the least rectangle holding both.
pub(super) fn union(a: Option<[i32; 4]>, b: [i32; 4]) -> [i32; 4] {
    a.map_or(b, |a| {
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    })
}

/// Whether `a` and `b` share a pixel.
pub(super) fn meets(a: [i32; 4], b: [i32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// `r` cut to `bounds`; none if nothing of it is inside.
pub(super) fn within(r: [i32; 4], bounds: [i32; 4]) -> Option<[i32; 4]> {
    let c = [
        r[0].max(bounds[0]),
        r[1].max(bounds[1]),
        r[2].min(bounds[2]),
        r[3].min(bounds[3]),
    ];
    (c[2] > c[0] && c[3] > c[1]).then_some(c)
}

/// The tabs lit: each bank's choice, AUTO GAIN's ON while it is on.
fn lit_tabs(scene: &StripScene) -> Vec<(f64, f64)> {
    let mut lit = Vec::new();
    for (b, at) in [
        (Bank::Mode, scene.mode),
        (Bank::Stereo, scene.stereo),
        (Bank::Placement, scene.placement),
        (Bank::Auto, scene.auto.then_some(0)),
    ] {
        if let Some(i) = at {
            lit.push(b.tab(i));
        }
    }
    lit
}

/// An SVG body (the strip's units: the drawing's, from the strip's top) drawn over `frame`.
pub(super) fn svg_over(options: &usvg::Options<'_>, frame: &mut Pixmap, scale: f64, body: &str) {
    let (w, h) = (frame.width(), frame.height());
    let view = [0.0, 0.0, f64::from(w) / scale, f64::from(h) / scale];
    let doc = art::document(view, w, h, body);
    if let Ok(tree) = usvg::Tree::from_str(&doc, options) {
        resvg::render(&tree, Transform::identity(), &mut frame.as_mut());
    }
}

/// The keys' marks: the previous and next arrows, SAVE, the settings' gear (the star is drawn
/// as the preset is).
fn key_marks(o: &mut Svg, rail: Rail, print: &str) {
    let y = super::RAIL_Y - PANEL_H;
    for &(x, _, t) in rail.keys() {
        match t {
            BarTarget::Prev | BarTarget::Next => {
                let d = if t == BarTarget::Prev { -1.0 } else { 1.0 };
                put!(
                    o,
                    "<path d='M {} {} L {} {} L {} {}' fill='none' stroke='{print}' stroke-width='4' stroke-linejoin='round' stroke-linecap='round'/>",
                    N(x - 6.0 * d),
                    N(y - 11.0),
                    N(x + 7.0 * d),
                    N(y),
                    N(x - 6.0 * d),
                    N(y + 11.0)
                );
            }
            BarTarget::Save
            | BarTarget::Panel
            | BarTarget::Arp
            | BarTarget::Fx
            | BarTarget::Mpe => put!(
                o,
                "<text x='{}' y='{}' font-size='20' font-weight='700' text-anchor='middle' dominant-baseline='central' fill='{print}'>{}</text>",
                N(x),
                N(y),
                match t {
                    BarTarget::Panel => "PANEL",
                    BarTarget::Arp => "ARP",
                    BarTarget::Fx => "FX",
                    BarTarget::Mpe => "MPE",
                    _ => "SAVE",
                }
            ),
            BarTarget::Keyboard => put!(
                o,
                "<path d='M {} {} H {} V {} H {} Z M {} {} V {} M {} {} V {}' fill='none' stroke='{print}' stroke-width='2'/><path d='M {} {} h 5 v 14 h -5 Z M {} {} h 5 v 14 h -5 Z' fill='{print}'/>",
                N(x - 15.0),
                N(y - 13.0),
                N(x + 15.0),
                N(y + 13.0),
                N(x - 15.0),
                N(x - 5.0),
                N(y - 13.0),
                N(y + 13.0),
                N(x + 5.0),
                N(y - 13.0),
                N(y + 13.0),
                N(x - 8.0),
                N(y - 13.0),
                N(x + 2.0),
                N(y - 13.0)
            ),
            BarTarget::Settings => put!(
                o,
                "<path d='{}' fill='{print}' fill-rule='evenodd'/>",
                gear_path(x, y, 15.0)
            ),
            _ => {}
        }
    }
}

/// A gear of eight teeth about (`cx`, `cy`), `r` to its teeth's tips, its hub open (the
/// typeface has none).
fn gear_path(cx: f64, cy: f64, r: f64) -> String {
    let (inner, hub) = (r * 0.72, r * 0.34);
    let mut d = String::new();
    for k in 0..8 {
        let a = std::f64::consts::TAU * f64::from(k) / 8.0;
        // Each tooth a little narrower at its tip than at its root.
        for (da, rr) in [(-0.2, inner), (-0.13, r), (0.13, r), (0.2, inner)] {
            let (x, y) = (cx + rr * (a + da).cos(), cy + rr * (a + da).sin());
            d.push_str(&format!(
                "{}{} {} ",
                if d.is_empty() { "M" } else { "L" },
                N(x),
                N(y)
            ));
        }
    }
    d.push_str("Z ");
    d.push_str(&format!(
        "M {} {} a {h} {h} 0 1 0 {} 0 a {h} {h} 0 1 0 {} 0 Z",
        N(cx - hub),
        N(cy),
        N(2.0 * hub),
        N(-2.0 * hub),
        h = N(hub)
    ));
    d
}

/// The favourite's key: its star, filled for a favourite.
fn star_key(o: &mut Svg, rail: Rail, favorite: bool, print: &str) {
    let y = super::RAIL_Y - PANEL_H;
    let Some(&(x, _, _)) = rail.keys().iter().find(|k| k.2 == BarTarget::Star) else {
        return;
    };
    let d = plugin_kit_materials::star_path(x, y, 14.0);
    if favorite {
        put!(o, "<path d='{d}' fill='{print}'/>");
    } else {
        put!(
            o,
            "<path d='{d}' fill='none' stroke='{print}' stroke-width='2.5' stroke-linejoin='round'/>"
        );
    }
}

/// The pointer's part, lightened a little.
fn hovered(o: &mut Svg, rail: Rail, t: Option<StripTarget>) {
    let rect = match t {
        Some(StripTarget::Bar(b)) if b != BarTarget::Name => rail
            .keys()
            .iter()
            .find(|k| k.2 == b)
            .map(|&(x, w, _)| (x - w / 2.0, super::RAIL_Y - KEY_HEIGHT / 2.0, w, KEY_HEIGHT)),
        Some(StripTarget::Tab(b, i)) => {
            let (x, y) = b.tab(i);
            Some((x - TAB.0 / 2.0, y - TAB.1 / 2.0, TAB.0, TAB.1))
        }
        _ => None,
    };
    if let Some((x, y, w, h)) = rect {
        put!(
            o,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#fff' fill-opacity='0.07'/>",
            N(x + 2.0),
            N(y - PANEL_H + 2.0),
            N(w - 4.0),
            N(h - 4.0)
        );
    }
}

/// MIDI Learn's ring round a control it waits for (as the panel's: a dashed outline).
fn ring(o: &mut Svg, c: StripControl) {
    let (x0, y0, x1, y1) = span(c);
    put!(
        o,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='10' fill='none' stroke='{}' stroke-width='4' stroke-dasharray='12 8'/>",
        N(x0 - 4.0),
        N(y0 - PANEL_H - 4.0),
        N(x1 - x0 + 8.0),
        N(y1 - y0 + 8.0),
        crate::learn::ACCENT
    );
}

// ---- The readouts.

/// Readout `i` reading `text`, lit or dark, drawn over the frame: from the patch kept for it,
/// made again only when what it reads changes.
fn reading(k: &mut StripRenderer, i: usize, r: Readout, text: &str, lit: bool) {
    let scale = k.scale;
    let (x, y, w, h) = r.window();
    let Some(slot) = k.readouts.get_mut(i) else {
        return;
    };
    if slot.as_ref().is_none_or(|r| r.text != text || r.lit != lit) {
        *slot = readout_patch(
            &k.options,
            scale,
            (x, y - PANEL_H, w, h),
            text,
            lit,
            &k.lamp,
        );
    }
    if let Some(r) = slot {
        k.frame.draw_pixmap(
            r.left,
            r.top,
            r.patch.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
}

/// A readout's window `(x, y, w, h)` (its frame's units) reading `text`, lit or dark, in `lamp`'s
/// colour: its three
/// cells of seven segments and their glow, on a patch of its own a margin round it, and the patch's
/// place in the frame's pixels.
pub(super) fn readout_patch(
    options: &usvg::Options<'_>,
    scale: f64,
    window: (f64, f64, f64, f64),
    text: &str,
    lit: bool,
    lamp: &Lamp,
) -> Option<Reading> {
    readout_cells(options, scale, window, 3, text, lit, lamp)
}

/// [`readout_patch`] of `cells` cells (the expression drawer's MODULATION, four for its "-100":
/// decisions.md R-MPE).
pub(super) fn readout_cells(
    options: &usvg::Options<'_>,
    scale: f64,
    (x, y, w, h): (f64, f64, f64, f64),
    cells: usize,
    text: &str,
    lit: bool,
    lamp: &Lamp,
) -> Option<Reading> {
    // (The patch is the window and a margin for the glow.)
    let m = 10.0;
    let left = ((x - m) * scale).floor();
    let top = ((y - m) * scale).floor();
    let right = ((x + w + m) * scale).ceil();
    let bottom = ((y + h + m) * scale).ceil();
    let mut patch = Pixmap::new((right - left) as u32, (bottom - top) as u32)?;
    let mut o = Svg::default();
    put!(
        o,
        "<defs><filter id='segglow' x='-0.3' y='-0.6' width='1.6' height='2.2'><feGaussianBlur in='SourceGraphic' stdDeviation='6' result='far'/><feComponentTransfer in='far' result='farhalf'><feFuncA type='linear' slope='0.5'/></feComponentTransfer><feGaussianBlur in='SourceGraphic' stdDeviation='2' result='near'/><feMerge><feMergeNode in='farhalf'/><feMergeNode in='near'/><feMergeNode in='SourceGraphic'/></feMerge></filter></defs><g transform='translate({},{})'>",
        N(-left / scale),
        N(-top / scale)
    );
    seven(&mut o, (x, y, w, h), cells, text, lit, &lamp.seg());
    put!(o, "</g>");
    svg_over(options, &mut patch, scale, &o.0);
    Some(Reading {
        text: text.to_owned(),
        lit,
        left: left as i32,
        top: top as i32,
        patch,
    })
}

/// Seven-segment digits, `cells` of them, in the readout `(x, y, w, h)`, right-aligned, lit
/// or dark (every segment as an unlit one shows faintly through the glass). The CA-74's.
fn seven(
    s: &mut Svg,
    (x, y, w, h): (f64, f64, f64, f64),
    cells: usize,
    text: &str,
    lit: bool,
    seg: &str,
) {
    const SEGS: [(char, &str); 18] = [
        ('0', "abcdef"),
        ('1', "bc"),
        ('2', "abged"),
        ('3', "abgcd"),
        ('4', "fgbc"),
        ('5', "afgcd"),
        ('6', "afgedc"),
        ('7', "abc"),
        ('8', "abcdefg"),
        ('9', "abcdfg"),
        ('-', "g"),
        ('O', "abcdef"),
        ('F', "aefg"),
        ('N', "abcef"),
        ('E', "adefg"),
        // (A triplet's note value: RATE's "16t", decisions.md R-ARP.)
        ('t', "defg"),
        (' ', ""),
        // (A value being typed: where the next digit goes.)
        ('_', "d"),
    ];
    let mut shown: Vec<(char, bool)> = Vec::new();
    for c in text.chars() {
        match (c, shown.last_mut()) {
            ('.', Some(last)) => last.1 = true,
            _ => shown.push((c, false)),
        }
    }
    while shown.len() < cells {
        shown.insert(0, (' ', false));
    }
    let shown = &shown[shown.len() - cells..];
    let pitch = ((w - 22.0) / cells as f64).min(44.0);
    let (cw, ch, t) = (pitch * 0.68, h * 0.64, pitch * 0.19);
    let x0 = x + w - 12.0 - pitch * cells as f64 + (pitch - cw) / 2.0;
    let y0 = y + (h - ch) / 2.0;
    let hx = t / 2.0 + 1.3;
    let (y1, y2, y3, y4) = (
        t / 2.0 + 1.2,
        ch / 2.0 - 1.2,
        ch / 2.0 + 1.2,
        ch - t / 2.0 - 1.2,
    );
    let rect = |k: char| -> (f64, f64, f64, f64) {
        match k {
            'a' => (hx, 0.0, cw - 2.0 * hx, t),
            'g' => (hx, ch / 2.0 - t / 2.0, cw - 2.0 * hx, t),
            'd' => (hx, ch - t, cw - 2.0 * hx, t),
            'f' => (0.0, y1, t, y2 - y1),
            'b' => (cw - t, y1, t, y2 - y1),
            'e' => (0.0, y3, t, y4 - y3),
            _ => (cw - t, y3, t, y4 - y3),
        }
    };
    let (mut on, mut off) = (String::new(), String::new());
    for (i, &(c, dp)) in shown.iter().enumerate() {
        let segs = SEGS.iter().find(|(k, _)| *k == c).map_or("", |(_, v)| v);
        let cx = x0 + pitch * i as f64;
        let mut cell = |into_on: bool, body: String| {
            let to = if into_on && lit { &mut on } else { &mut off };
            to.push_str(&format!(
                "<g transform='translate({} {}) skewX(-7)'>{body}</g>",
                N(cx),
                N(y0)
            ));
        };
        for k in ['a', 'b', 'c', 'd', 'e', 'f', 'g'] {
            let (rx, ry, rw, rh) = rect(k);
            cell(
                segs.contains(k),
                format!(
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='{}'/>",
                    N(rx),
                    N(ry),
                    N(rw),
                    N(rh),
                    N(t / 2.0)
                ),
            );
        }
        cell(
            dp,
            format!(
                "<circle cx='{}' cy='{}' r='{}'/>",
                N(cw + 4.5),
                N(ch - 3.0),
                N(t * 0.55)
            ),
        );
    }
    put!(
        s,
        "<g fill='{seg}' fill-opacity='{SEG_OFF_OPACITY}'>{off}</g>"
    );
    if !on.is_empty() {
        put!(s, "<g fill='{seg}' filter='url(#segglow)'>{on}</g>");
    }
}

// ---- The preset's name.

/// Where the name's characters go: the first's left, how many fit, the star's and the
/// arrow's places.
fn name_places(rail: Rail) -> (f64, usize, f64, f64) {
    let (x, _, w, _) = rail.name();
    let (star_x, arrow_x) = (x + 28.0, x + w - 32.0);
    let left = star_x + 26.0;
    let places = ((arrow_x - 30.0 - left) / (6.0 * DOT)).floor().max(1.0) as usize;
    (left, places, star_x, arrow_x)
}

/// How bright the name's display is: dimmer where the preset is not in the library.
fn name_level(b: &BarScene) -> f64 {
    if b.name.is_empty() || !b.found {
        0.55
    } else {
        1.0
    }
}

/// The preset's name in dots ([`font::glyph`]), lit over the unlit dots (the still's).
fn name(frame: &mut Pixmap, scale: f64, rail: Rail, b: &BarScene, lamp: &Lamp) {
    let (_, places, ..) = name_places(rail);
    let level = name_level(b);
    let text: String = if b.name.is_empty() {
        "NO PRESET".to_owned()
    } else if b.changed {
        format!("{} \u{2022}", b.name)
    } else {
        b.name.clone()
    };
    // A control character (a file written elsewhere) is drawn as if it were not there.
    let mut chars: Vec<char> = text.chars().filter(|c| !c.is_control()).collect();
    if chars.len() > places {
        chars.truncate(places - 1);
        chars.push('\u{2026}');
    }
    let glyphs: Vec<[u8; 9]> = chars.iter().map(|&c| font::glyph(c)).collect();
    let r = DOT * 0.36 * scale;
    let mut glow = PathBuilder::new();
    let mut core = PathBuilder::new();
    name_dots(scale, rail, |i, j, col, (px, py)| {
        if glyphs.get(i).is_some_and(|g| g[j] >> (4 - col) & 1 == 1) {
            glow.push_circle(px, py, (r * 2.1) as f32);
            core.push_circle(px, py, r as f32);
        }
    });
    // The favourite's star before it and the arrow that drops the list down after it, in the
    // same dots (the owner, 2026-10-10: "turn this into a pixel star", "make sure the arrow on
    // the right side of the preset dropdown is also dots"): the star lit for a favourite, else
    // as faint as the unlit dots; the arrow up while the list is down, brighter under the
    // pointer.
    let (_, _, star_x, arrow_x) = name_places(rail);
    let mut star = PathBuilder::new();
    let mut star_glow = PathBuilder::new();
    sign_dots(scale, &STAR, star_x, |(px, py)| {
        star_glow.push_circle(px, py, (r * 2.1) as f32);
        star.push_circle(px, py, r as f32);
    });
    let mut arrow = PathBuilder::new();
    let mut arrow_glow = PathBuilder::new();
    let sign = if b.open { &UP } else { &DOWN };
    sign_dots(scale, sign, arrow_x, |(px, py)| {
        arrow_glow.push_circle(px, py, (r * 2.1) as f32);
        arrow.push_circle(px, py, r as f32);
    });
    let pointed = if b.hover == Some(BarTarget::Name) {
        1.0
    } else {
        0.7
    };
    // (In the mock-up's red: its dots' orange, their glow a deeper red; the owner: "go back to
    // that red".)
    let dim = |c: (u8, u8, u8)| {
        let d = |v: u8| (f64::from(v) * level + 20.0 * (1.0 - level)) as u8;
        (d(c.0), d(c.1), d(c.2))
    };
    fill(frame, glow, lamp.glow, 0.16 * level);
    fill(frame, core, dim(lamp.on), 0.55 + 0.45 * level);
    if b.favorite {
        fill(frame, star_glow, lamp.glow, 0.16 * level);
        fill(frame, star, dim(lamp.on), 0.55 + 0.45 * level);
    } else {
        fill(frame, star, lamp.on, 0.06);
    }
    fill(frame, arrow_glow, lamp.glow, 0.16 * level * pointed);
    fill(frame, arrow, dim(lamp.on), (0.55 + 0.45 * level) * pointed);
}

/// A sign's dots, in pixels at `scale`, about `x` (units) on the name's rows.
fn sign_dots(scale: f64, sign: &[u8; 7], x: f64, mut each: impl FnMut((f32, f32))) {
    let (_, y, _, h) = super::NAME;
    let top = y - PANEL_H + h / 2.0 - CAP_LIFT - 3.0 * DOT;
    for (j, row) in sign.iter().enumerate() {
        for col in 0..5 {
            if row >> (4 - col) & 1 == 1 {
                let px = ((x + (col as f64 - 2.0) * DOT) * scale) as f32;
                let py = ((top + j as f64 * DOT) * scale) as f32;
                each((px, py));
            }
        }
    }
}

/// Each dot of the name's display, in pixels at `scale` (the strip's frame): its character's
/// place, its row, its column. (Seven rows for a capital, its middle [`CAP_LIFT`] above the
/// window's; two below for a descender, its last dots inside the window.)
fn name_dots(scale: f64, rail: Rail, mut each: impl FnMut(usize, usize, usize, (f32, f32))) {
    let (_, y, _, h) = rail.name();
    let (left, places, ..) = name_places(rail);
    let top = y - PANEL_H + h / 2.0 - CAP_LIFT - 3.0 * DOT;
    screen_dots(scale, left, top, places, &mut each);
}

fn screen_dots(
    scale: f64,
    left: f64,
    top: f64,
    places: usize,
    mut each: impl FnMut(usize, usize, usize, (f32, f32)),
) {
    for i in 0..places {
        for j in 0..9 {
            for col in 0..5 {
                let px = ((left + (6 * i + col) as f64 * DOT) * scale) as f32;
                let py = ((top + j as f64 * DOT) * scale) as f32;
                each(i, j, col, (px, py));
            }
        }
    }
}

/// A centred readout made from the preset display's bezel, glass and pixel lettering.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pixel_screen(
    options: &usvg::Options<'_>,
    frame: &mut Pixmap,
    picture: &Pixmap,
    scale: f64,
    window: (f64, f64, f64, f64),
    text: &str,
    lamp: &Lamp,
) {
    screen_still(options, frame, picture, scale, window);
    screen_text(frame, scale, window, text, lamp);
}

/// A pixel screen's bezel and glass, which do not change with what it reads.
pub(crate) fn screen_still(
    options: &usvg::Options<'_>,
    frame: &mut Pixmap,
    picture: &Pixmap,
    scale: f64,
    (x, y, w, h): (f64, f64, f64, f64),
) {
    let mut bezel = Svg::default();
    super::screen_bezel(&mut bezel, (x, y, w, h));
    svg_over(options, frame, scale, &bezel.0);
    glass(frame, picture, scale, (x, y, w, h), 3.0);
}

/// A pixel screen's dots, `text` lit in `lamp`'s colour, centred, over its glass.
pub(crate) fn screen_text(
    frame: &mut Pixmap,
    scale: f64,
    (x, y, w, h): (f64, f64, f64, f64),
    text: &str,
    lamp: &Lamp,
) {
    let mut places = ((w - 16.0) / (6.0 * DOT)).floor().max(1.0) as usize;
    let glyphs: Vec<_> = text.chars().take(places).map(font::glyph).collect();
    // Keep centred text on the same dot grid, including an even character count.
    if places > 1 && !(places - glyphs.len()).is_multiple_of(2) {
        places -= 1;
    }
    let start = (places - glyphs.len()) / 2;
    let left = x + (w - (6 * places - 1) as f64 * DOT) / 2.0;
    let top = y + h / 2.0 - CAP_LIFT - 3.0 * DOT;
    let r = DOT * 0.36 * scale;
    let mut unlit = PathBuilder::new();
    let mut glow = PathBuilder::new();
    let mut core = PathBuilder::new();
    screen_dots(scale, left, top, places, |i, j, col, (px, py)| {
        unlit.push_circle(px, py, r as f32);
        if i >= start
            && glyphs
                .get(i - start)
                .is_some_and(|g| g[j] >> (4 - col) & 1 == 1)
        {
            glow.push_circle(px, py, (r * 2.1) as f32);
            core.push_circle(px, py, r as f32);
        }
    });
    fill(frame, unlit, lamp.on, 0.06);
    fill(frame, glow, lamp.glow, 0.16);
    fill(frame, core, lamp.on, 1.0);
}

// ---- The voices' display: drops of light (the CA-74's LIQUID, its R36).

/// A drop: where it is (-1..1 of the field), how much of it there is (0..1), how much of it
/// sounds (0..1) and how loud its voice is (0..1, as a lamp's filament follows a level: quick
/// to rise, slower to fall).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drop {
    x: f64,
    size: f64,
    lit: f64,
    level: f64,
}

/// The drops, each easing to where the scene puts it: a drop's id is its voice's (with DOUBLE
/// its twin's `TWIN +` its voice's); and DOUBLE's beat, its phase (radians), and the light's
/// sums and a drop's columns' dx squared, kept from frame to frame.
#[derive(Debug, Default)]
struct Liquid {
    drops: BTreeMap<u32, Drop>,
    beat: f64,
    sums: (Vec<f32>, Vec<f32>),
    dx2: Vec<f32>,
}

const TWIN: u32 = 1000;

/// A sounding drop's size at its voice's silence and loudest, shares of [`DROP`] (the owner:
/// "make the dots in the stereo spread a little more animated according to the volume of their
/// respective sounds"); how fast its level follows its voice's, rising and falling (per
/// second).
const QUIET_SIZE: f64 = 0.72;
const LOUD_SIZE: f64 = 1.22;
const LEVEL_UP: f64 = 30.0;
const LEVEL_DOWN: f64 = 5.0;
/// DOUBLE's detune shown (the owner: "perhaps someway to animate the detune effect being
/// placed on them as well?"): a note's two voices beat against each other as two notes so far
/// apart do, at [`BEAT_AT`] Hz (the A below middle C), one swelling as the other ebbs and the
/// two swaying in turn, by as much as [`BEAT_SWELL`] of their size and [`BEAT_SWAY`] units at
/// [`BEAT_FULL`] cents or more apart (a quarter of what they were first: "the amount of wobble
/// you are showing in the stereo bar is a bit intense. dial it back a little", then "dial back
/// the detune wobble a bit more").
const BEAT_AT: f64 = 220.0;
const BEAT_SWELL: f64 = 0.07;
const BEAT_SWAY: f64 = 1.0;
const BEAT_FULL: f64 = 10.0;
/// How far a drop's light reaches, in its radii (past it, its halo under a level's step).
const REACH: f64 = 4.5;

/// The beat of two notes `cents` apart at [`BEAT_AT`], Hz, at most [`BEAT_MOST`] (two notes
/// a semitone apart beat 13 times a second, which the drops would only flicker at).
const BEAT_MOST: f64 = 4.0;
fn beat_hz(cents: f64) -> f64 {
    (BEAT_AT * (2f64.powf(cents.max(0.0) / 1200.0) - 1.0)).min(BEAT_MOST)
}

/// Where the scene puts each drop: its id, its place, whether it sounds, and the drop it comes
/// out of when it appears (a twin, its voice).
fn targets(s: &StripScene) -> Vec<(u32, f64, bool, Option<u32>)> {
    match &s.field {
        Field::Scatter(places) => places
            .iter()
            .enumerate()
            .map(|(i, &(p, on))| (i as u32, p.clamp(-1.0, 1.0), on, None))
            .collect(),
        Field::Double(pairs) => pairs
            .iter()
            .enumerate()
            .flat_map(|(i, &(w, on))| {
                let (i, w) = (i as u32, w.clamp(0.0, 1.0));
                [(i, w, on, None), (TWIN + i, -w, on, Some(i))]
            })
            .collect(),
    }
}

impl Liquid {
    /// The drops eased `dt` seconds towards the scene's places and their voices' levels, and
    /// DOUBLE's beat moved on: whether any moved, and whether all are where they are going.
    fn step(&mut self, s: &StripScene, dt: f64) -> (bool, bool) {
        let want = targets(s);
        let mut moved = false;
        let mut settled = true;
        let level_of = |id: u32| {
            let voice = if id >= TWIN { id - TWIN } else { id } as usize;
            s.levels.get(voice).copied().unwrap_or(0.0).clamp(0.0, 1.0)
        };
        for &(id, x, on, from) in &want {
            if !self.drops.contains_key(&id) {
                let x0 = from.and_then(|f| self.drops.get(&f)).map_or(x, |d| d.x);
                let lit = f64::from(u8::from(on));
                self.drops.insert(
                    id,
                    Drop {
                        x: x0,
                        size: if dt == 0.0 { 1.0 } else { 0.0 },
                        lit,
                        level: if dt == 0.0 { level_of(id) * lit } else { 0.0 },
                    },
                );
                moved = true;
            }
        }
        let kx = 1.0 - (-dt * 7.0).exp();
        let kr = 1.0 - (-dt * 11.0).exp();
        let (up, down) = (1.0 - (-dt * LEVEL_UP).exp(), 1.0 - (-dt * LEVEL_DOWN).exp());
        let homes: BTreeMap<u32, f64> = self.drops.iter().map(|(&k, d)| (k, d.x)).collect();
        self.drops.retain(|&id, d| {
            let t = want.iter().find(|w| w.0 == id);
            let tx = match t {
                Some(t) => t.1,
                None if id >= TWIN => homes.get(&(id - TWIN)).copied().unwrap_or(d.x),
                None => d.x,
            };
            let (ts, tl) = t.map_or((0.0, 0.0), |t| (1.0, f64::from(u8::from(t.2))));
            let tv = level_of(id) * tl;
            let was = *d;
            d.x += (tx - d.x) * kx;
            d.size += (ts - d.size) * kr;
            d.lit += (tl - d.lit) * kr;
            d.level += (tv - d.level) * if tv > d.level { up } else { down };
            if (tx - d.x).abs() < 1e-3 && (ts - d.size).abs() < 1e-3 && (tl - d.lit).abs() < 1e-3 {
                (d.x, d.size, d.lit) = (tx, ts, tl);
            }
            if (tv - d.level).abs() < 2e-3 {
                d.level = tv;
            }
            moved |= *d != was;
            let stays = t.is_some() || d.size > 0.0;
            moved |= !stays;
            settled &= !stays || (d.x, d.size, d.lit, d.level) == (tx, ts, tl, tv);
            stays
        });
        // DOUBLE's beat, while a pair sounds.
        let beating = matches!(s.field, Field::Double(_))
            && s.detune > 0.0
            && self
                .drops
                .iter()
                .any(|(&id, d)| id >= TWIN && d.level > 0.0);
        if beating {
            self.beat = (self.beat + dt * beat_hz(s.detune) * std::f64::consts::TAU)
                % std::f64::consts::TAU;
            moved = true;
            settled = false;
        }
        (moved, settled)
    }

    /// A drop's place (pixels across, the field's middle at `cx`, half `half` wide) and its
    /// light's strengths, sounding and idle: with DOUBLE, a pair beating as their detune makes
    /// them (`detune`, cents).
    fn shape(
        &self,
        id: u32,
        d: &Drop,
        cx: f64,
        half: f64,
        scale: f64,
        detune: f64,
    ) -> (f64, f64, f64) {
        let paired = id >= TWIN || self.drops.contains_key(&(TWIN + id));
        let (mut x, mut grow) = (cx + d.x * half, 1.0);
        if paired && detune > 0.0 {
            let side = if id >= TWIN { -1.0 } else { 1.0 };
            let k = (detune / BEAT_FULL).min(1.0) * d.level;
            grow += side * BEAT_SWELL * k * self.beat.sin();
            x += side * BEAT_SWAY * scale * k * self.beat.cos();
        }
        let r = DROP * scale * (QUIET_SIZE + (LOUD_SIZE - QUIET_SIZE) * d.level) * grow;
        (
            x,
            d.size * d.lit * r * r,
            d.size * (1.0 - d.lit) * (IDLE_DROP * scale).powi(2),
        )
    }

    /// The drops' light added into `frame` inside `window` (pixels), the field's middle at
    /// (`cx`, `cy`) and its half width `half` (pixels): each drop's field its radius squared
    /// over the distance's (so that drops that meet pool), out to [`REACH`] radii, lit past a
    /// level with a halo, an idle voice's fainter. (Each drop adds its field only near it, and
    /// only where there is any is it lit: a few drops in a wide display cost a few drops'
    /// worth, not the display's; only the part of the display they reach is looked at again.)
    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        frame: &mut Pixmap,
        window: [i32; 4],
        (cx, cy): (f64, f64),
        half: f64,
        scale: f64,
        detune: f64,
        drop: (f64, f64, f64),
    ) {
        let (fw, fh) = (frame.width() as i32, frame.height() as i32);
        let [x0, y0, x1, y1] = [
            window[0].max(0),
            window[1].max(0),
            window[2].min(fw),
            window[3].min(fh),
        ];
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let ds: Vec<(f64, f64, f64)> = self
            .drops
            .iter()
            .map(|(&id, d)| self.shape(id, d, cx, half, scale, detune))
            .collect();
        if ds.iter().all(|&(_, a, b)| a <= 0.0 && b <= 0.0) {
            return;
        }
        let w = (x1 - x0) as usize;
        let n = w * (y1 - y0) as usize;
        let (sums_a, sums_b) = (&mut self.sums.0, &mut self.sums.1);
        sums_a.clear();
        sums_a.resize(n, 0.0);
        sums_b.clear();
        sums_b.resize(n, 0.0);
        // (Where any drop reaches: the part of the window looked at again below.)
        let mut reached = [x1, y1, x0, y0];
        for &(dx0, a, b) in &ds {
            if a <= 0.0 && b <= 0.0 {
                continue;
            }
            let reach = REACH * a.max(b).sqrt() + 2.0;
            let (lx, rx) = (
                ((dx0 - reach).floor() as i32).max(x0),
                ((dx0 + reach).ceil() as i32).min(x1),
            );
            let (ty, by) = (
                ((cy - reach).floor() as i32).max(y0),
                ((cy + reach).ceil() as i32).min(y1),
            );
            reached = [
                reached[0].min(lx),
                reached[1].min(ty),
                reached[2].max(rx),
                reached[3].max(by),
            ];
            let (a, b, dx0) = (a as f32, b as f32, dx0 as f32);
            // (Each column's dx squared once for all the rows, the same sums as before: a
            // plain loop the compiler can take four pixels at a time.)
            let dx2 = &mut self.dx2;
            dx2.clear();
            dx2.extend((lx..rx.max(lx)).map(|x| {
                let dx = x as f32 + 0.5 - dx0;
                dx * dx
            }));
            for y in ty..by {
                let dy = y as f32 + 0.5 - cy as f32;
                let dy2 = dy * dy + 1.0;
                let row = (y - y0) as usize * w;
                let span = row + (lx - x0) as usize..row + (rx.max(lx) - x0) as usize;
                let (fa, fb) = (&mut sums_a[span.clone()], &mut sums_b[span]);
                for ((sa, sb), &d) in fa.iter_mut().zip(fb.iter_mut()).zip(dx2.iter()) {
                    let q = 1.0 / (d + dy2);
                    *sa += a * q;
                    *sb += b * q;
                }
            }
        }
        let data = frame.data_mut();
        for y in reached[1]..reached[3] {
            let row = (y - y0) as usize * w;
            for x in reached[0]..reached[2] {
                let k = row + (x - x0) as usize;
                let (f, g) = (sums_a[k], sums_b[k]);
                if f == 0.0 && g == 0.0 {
                    continue;
                }
                // (The mock-up's: a flat disc where the fields sum past one, its light the
                // share of them that sounds, an idle voice's half as bright; a faint halo.)
                let (all, lit) = (f + g, f + 0.5 * g);
                let share = lit / all.max(1e-6);
                let core = ((all - 1.0) * 3.0).clamp(0.0, 1.0);
                let halo = (all * 0.35).min(1.0);
                let a = core * (0.4 + 0.6 * share) + halo * 0.22 * share;
                if a <= 0.002 {
                    continue;
                }
                let light = [drop.0 as f32, drop.1 as f32, drop.2 as f32];
                let i = ((y * fw + x) * 4) as usize;
                for (c, v) in light.iter().enumerate() {
                    data[i + c] = (f32::from(data[i + c]) + v * a).min(255.0) as u8;
                }
                data[i + 3] = data[i + 3].max((255.0 * a.min(1.0)) as u8);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use resvg::tiny_skia::{Color, Paint, Rect};

    use super::*;

    /// The drops move on each time the caller's clock does, 30 ms apart as well as 45 (the
    /// editor's ticks on baseview's 15 ms frames: at a thirtieth of a second at the least they
    /// had missed the 30 ms ones); not when it has not moved; and on the wall's clock, through
    /// `render`, at most thirty times a second.
    #[test]
    fn the_drops_move_on_each_tick_of_the_clock_they_are_given() {
        let scale = 0.2;
        let (w, h) = crate::render::size_at(scale);
        let under = Pixmap::new(w, h).expect("a frame");
        let mut r = StripRenderer::new(scale);
        let mut sc = scene(0b1111);
        let t0 = Instant::now();
        r.render_at(&sc, &under, None, t0);
        let mut at = t0;
        for (k, gap) in [30u64, 45, 30, 30, 45].into_iter().enumerate() {
            at += Duration::from_millis(gap);
            sc.levels = (0..4).map(|i| 0.2 + 0.15 * ((k + i) % 4) as f64).collect();
            r.render_at(&sc, &under, None, at);
            assert!(r.moved, "tick {k} ({gap} ms) not moved on");
        }
        r.render_at(&sc, &under, None, at);
        assert!(!r.moved, "moved on with the clock still");
        // The wall's clock: twice at once (its first frame, which makes its still parts, apart),
        // the second not moved on.
        let mut q = StripRenderer::new(0.1);
        let small = Pixmap::new(crate::render::size_at(0.1).0, crate::render::size_at(0.1).1)
            .expect("a frame");
        q.render(&sc, &small, None);
        std::thread::sleep(Duration::from_millis(40));
        q.render(&sc, &small, None);
        sc.levels = vec![0.9; 4];
        q.render(&sc, &small, None);
        assert!(!q.moved, "moved on twice within a thirtieth of a second");
    }

    /// A change in the drawing above the strip (QUALITY's opening ticking at the panel's top
    /// right) draws nothing of the strip again: with the drops moving, only their window.
    #[test]
    fn a_change_above_the_strip_draws_none_of_it() {
        let scale = 0.5;
        let (w, h) = crate::render::size_at(scale);
        let under = Pixmap::new(w, h).expect("a frame");
        let mut r = StripRenderer::new(scale);
        let mut sc = scene(0b1111);
        let mut now = Instant::now();
        r.render_at(&sc, &under, None, now);
        let above = Some([(w - 100) as i32, 30, w as i32, 130]);
        now += Duration::from_millis(33);
        sc.levels = vec![0.0; 4];
        let quiet = r.render_at(&sc, &under, above, now);
        let window = r.window();
        for k in 0..10 {
            now += Duration::from_millis(33);
            sc.levels = (0..4).map(|i| 0.3 + 0.1 * ((k + i) % 3) as f64).collect();
            if r.render_at(&sc, &under, above, now) {
                let d = r.damage().expect("where it changed");
                assert!(
                    d[0] >= window[0] - 1
                        && d[1] >= window[1] - 1
                        && d[2] <= window[2] + 1
                        && d[3] <= window[3] + 1,
                    "tick {k}: drawn {d:?}, the drops' window {window:?}"
                );
            }
        }
        // (Before the drops moved: nothing at all, but what the levels' first change moved.)
        if quiet {
            let d = r.damage().expect("where it changed");
            assert!(d[1] >= window[1] - 1, "drawn from the strip's top: {d:?}");
        }
    }

    /// What drawing the drops costs at the window's opening size and twice it (a Retina
    /// screen), ten voices sounding and moving: printed (`-- --ignored --nocapture`).
    #[test]
    #[ignore = "prints timings"]
    fn drops_timings() {
        for scale in [1720.0 / crate::art::W, 3440.0 / crate::art::W] {
            let (w, h) = crate::render::size_at(scale);
            let under = Pixmap::new(w, h).expect("a frame");
            let mut r = StripRenderer::new(scale);
            let mut now = Instant::now();
            let mut sc = scene(0);
            r.render_at(&sc, &under, None, now);
            let mut times = Vec::new();
            for k in 0..200u32 {
                sc.field = Field::Double(
                    (0..10)
                        .map(|i| (0.1 + 0.08 * f64::from(i) + 0.02 * f64::from(k % 7), true))
                        .collect(),
                );
                sc.levels = (0..10)
                    .map(|i| 0.5 + 0.4 * (f64::from(k + i) * 0.4).sin())
                    .collect();
                sc.detune = 12.0;
                now += Duration::from_millis(33);
                let t = Instant::now();
                r.render_at(&sc, &under, None, now);
                times.push(t.elapsed().as_secs_f64() * 1000.0);
            }
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            println!("scale {scale:.3}: the drops moving, {mean:.2} ms a frame on average");
        }
    }

    /// A scene with `sounding` (a bit a voice) of eight voices sounding.
    fn scene(sounding: u32) -> StripScene {
        StripScene {
            mode: Some(1),
            readouts: std::array::from_fn(|i| (format!("{}", i * 7), i % 3 != 0)),
            field: Field::Scatter(
                (0..8)
                    .map(|i| (f64::from(i) / 4.0 - 0.9, sounding >> i & 1 == 1))
                    .collect(),
            ),
            bar: BarScene {
                name: "Brass Tutti".into(),
                found: true,
                ..BarScene::default()
            },
            ..StripScene::default()
        }
    }

    /// The frames drawn again where they changed are the frames drawn whole, to the bit,
    /// through the drops moving, readouts, tabs, the rail, the pointer, MIDI Learn's ring and
    /// the drawing under the strip changing (under a lit tab, a readout, the display and
    /// elsewhere).
    #[test]
    fn a_frame_drawn_in_parts_is_the_frame_drawn_whole() {
        let s = 0.25;
        let (w, h) = crate::render::size_at(s);
        let mut under = Pixmap::new(w, h).expect("a frame");
        under.fill(Color::from_rgba8(40, 36, 30, 255));
        let mut parts = StripRenderer::new(s);
        let mut whole = StripRenderer::new(s);
        whole.whole = true;
        let mut now = Instant::now();
        let mut sc = scene(0b1010_0101);
        let mut changes = 0;
        let mut drawn = 0;
        // The drawing under a point of the strip (units) changed round it: its rectangle in
        // the drawing's pixels.
        let paint = |under: &mut Pixmap, (x, y): (f64, f64), shade: u8| {
            let mut p = Paint::default();
            p.set_color_rgba8(shade, 180, 150, 255);
            let r = Rect::from_xywh((x * s) as f32 - 12.0, (y * s) as f32 - 9.0, 24.0, 18.0)
                .expect("a rect");
            under.fill_rect(r, &p, Transform::identity(), None);
            // (Every pixel it touches, its edges falling between pixels.)
            let (x, y) = ((x * s).floor() as i32, (y * s).floor() as i32);
            Some([x - 13, y - 10, x + 14, y + 11])
        };
        let controls = [
            StripControl::Poly,
            StripControl::Unison,
            StripControl::Double,
            StripControl::Placement,
            StripControl::AutoGain,
        ];
        for i in 0..200 {
            let mut changed = None;
            match i {
                0..=24 => sc.field = scene(if i % 6 < 3 { 0b1111 } else { 0b0011_0000 }).field,
                25 => sc.readouts[4].0 = "-12".into(),
                27 => sc.readouts[0] = ("10".into(), false),
                30 => sc.mode = Some(2),
                31 => sc.auto = false,
                32 => sc.placement = None,
                33 => sc.hover = Some(StripTarget::Tab(Bank::Placement, 1)),
                34 => sc.hover = Some(StripTarget::Bar(BarTarget::Save)),
                35 => sc.hover = None,
                36..=39 => sc.learning = Some(controls[i - 36]),
                40 => sc.learning = None,
                41 => sc.bar.name = "Warm Pad".into(),
                42 => sc.bar.favorite = true,
                43 => sc.bar.hover = Some(BarTarget::Name),
                71 => sc.bar.panel = true,
                72 => sc.bar.keyboard = true,
                73 => sc.bar.panel = false,
                74 => sc.bar.keyboard = false,
                // The drawing changed under a lit tab, a readout, the display, a corner.
                44 => changed = paint(&mut under, Bank::Mode.tab(2), 90),
                45 => changed = paint(&mut under, Readout::ALL[4].at(), 120),
                46 => changed = paint(&mut under, (DISPLAY.0 + 60.0, DISPLAY.1 + 20.0), 150),
                47 => changed = paint(&mut under, (40.0, PANEL_H + 820.0), 180),
                48 => {
                    sc.field = scene(0b1100_0011).field;
                    changed = paint(&mut under, Bank::Auto.tab(0), 60);
                }
                // The drops moving while the rest changes.
                50..=70 => {
                    sc.field = scene(if i % 4 < 2 { 0b0101 } else { 0b1010 }).field;
                    sc.readouts[5].0 = format!("{}", i % 7);
                    if i % 5 == 0 {
                        changed = paint(&mut under, Readout::ALL[5].at(), (i * 3) as u8);
                    }
                }
                _ => {}
            }
            now += Duration::from_millis(16);
            if parts.render_at(&sc, &under, changed, now) {
                changes += 1;
                let [x0, y0, x1, y1] = parts.damage().expect("where it changed");
                drawn += (x1 - x0) * (y1 - y0);
            }
            whole.render_at(&sc, &under, changed, now);
            assert!(
                parts.frame().data() == whole.frame().data(),
                "frame {i}: drawn in parts, not as whole"
            );
        }
        // (Drawn again in many frames, but a small part of each; the drops settled.)
        let all = (parts.frame().width() * parts.frame().height()) as i32;
        assert!(changes > 40, "{changes} frames drawn");
        assert!(drawn < all * changes / 4, "{drawn} pixels drawn");
        assert!(!parts.animating());
    }

    #[test]
    fn keyboard_damage_matches_whole_frames_and_stays_in_its_space() {
        for scale in [0.44, 0.87] {
            for skin in [crate::Skin::Drawn, crate::Skin::Worn] {
                let (w, h) = crate::render::size_at(scale);
                let mut under = Pixmap::new(w, h).expect("frame");
                under.fill(Color::from_rgba8(40, 36, 30, 255));
                let mut parts = StripRenderer::new(scale);
                let mut whole = StripRenderer::new(scale);
                whole.whole = true;
                let mut sc = scene(0);
                sc.keyboard = Some(crate::keyboard::Scene {
                    skin,
                    standalone: true,
                    ..Default::default()
                });
                sc.bar.keyboard = true;
                let now = Instant::now();
                for i in 0..12 {
                    match i {
                        1 => sc.keyboard.as_mut().unwrap().held = [1 << 60, 0],
                        2 => sc.keyboard.as_mut().unwrap().held = [(1 << 60) | (1 << 61), 1],
                        3 => sc.keyboard.as_mut().unwrap().held = [0, 1],
                        4 => sc.keyboard.as_mut().unwrap().held = [0, 0],
                        5 => sc.keyboard.as_mut().unwrap().root = 53,
                        6 => sc.keyboard.as_mut().unwrap().typing = true,
                        7 => {
                            sc.keyboard.as_mut().unwrap().hover =
                                Some(crate::keyboard::Target::Type)
                        }
                        8 => {
                            sc.keyboard.as_mut().unwrap().standalone = false;
                        }
                        9 => {
                            sc.keyboard.as_mut().unwrap().standalone = true;
                        }
                        10 => {
                            sc.keyboard = None;
                            sc.bar.keyboard = false;
                        }
                        11 => {
                            sc.keyboard = Some(crate::keyboard::Scene {
                                skin,
                                ..Default::default()
                            });
                            sc.bar.keyboard = true;
                        }
                        _ => {}
                    }
                    assert!(parts.render_at(&sc, &under, None, now));
                    // Compare with a fresh keyboard, rather than two caches with the same
                    // repaint history, including black-key presses and releases.
                    whole.keyboard = crate::keyboard::Renderer::new(scale);
                    whole.render_at(&sc, &under, None, now);
                    if parts.frame().data() != whole.frame().data() {
                        let w = parts.frame().width() as usize;
                        let differences: Vec<_> = parts
                            .frame()
                            .data()
                            .chunks_exact(4)
                            .zip(whole.frame().data().chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .map(|(p, (a, b))| (p % w, p / w, a.to_vec(), b.to_vec()))
                            .collect();
                        panic!(
                            "{skin:?} scale {scale}, step {i}, {} differences first {:?}, damage {:?}",
                            differences.len(),
                            &differences[..differences.len().min(8)],
                            parts.damage()
                        );
                    }
                    if i == 1 {
                        let r = parts.damage().expect("note damage");
                        assert!(
                            f64::from(r[0]) >= 370.0 * scale - 2.0,
                            "left hand untouched"
                        );
                        assert!(
                            f64::from(r[1]) > crate::strip::RAIL * scale,
                            "rail untouched"
                        );
                        assert!(
                            f64::from(r[2] - r[0]) < 200.0 * scale,
                            "one pitch changes only its key"
                        );
                    }
                }
                assert!(
                    !parts.render_at(&sc, &under, None, now),
                    "unchanged keyboard frame stays kept"
                );
            }
        }
    }
}
