//! The drawer of each note's own expression (decisions.md R-MPE; the owner's A, 2026-10-20): one
//! row dropped down from under the rail by the rail's MPE key, inside the window, over the top of
//! whichever lower page shows, as the arpeggiator's is (`arp`, `lower`). Left to right: INPUT, a
//! pixel screen of the rail's kind reading what expression comes in; a rule; PRESSURE's
//! MODULATION and CUTOFF; a rule; TIMBRE's MODULATION and CUTOFF, each a knob and its readout,
//! the strip's own parts. The two groups are each centred on their own half.
//!
//! The drawer is a plate of its own, as the arpeggiator's is. What does not change is drawn once
//! a scale on a thread of its own, begun as the editor opens; then only what changed.

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use plugin_kit_materials::glass;
use resvg::tiny_skia::Pixmap;
use resvg::usvg;

use super::lower::{self, Look};
pub use super::lower::{LEFT, RIGHT, SHADOW, TOP, top_px};
use super::worn::{
    Lamp, Reading, copy, meets, readout_cells, screen_still, screen_text, svg_over, union, within,
};
use super::{READOUT, UNIT_READOUT};
use crate::art::{self, Lettering};
use crate::fonts::FAMILY;
use crate::render::Renderer;
use crate::skin::Skin;
use crate::svg::{N, Svg, colour, put};

// ---- The layout (the drawing's units, the drawer open; the mock-up's).

/// How deep it is, and its row's middle: the arpeggiator's.
pub const DEPTH: f64 = super::arp::DEPTH;
pub const ROW: f64 = super::arp::ROW;
const LEGEND_UP: f64 = 109.0;
/// INPUT's screen: left, top, width, height.
pub const INPUT: (f64, f64, f64, f64) = (INPUT_X - 120.0, ROW - 25.0, 240.0, 50.0);
const INPUT_X: f64 = 536.0;
/// The rules: after INPUT, and between PRESSURE's units and TIMBRE's.
const RULES: [f64; 2] = [700.0, 1890.0];
/// The units' middles: PRESSURE's MODULATION and CUTOFF, TIMBRE's MODULATION and CUTOFF (each a
/// knob `UNIT_KNOB` left of it and its readout `UNIT_READOUT` right of it, as the strip's).
pub const UNITS: [f64; 4] = [1219.0, 1602.0, 2389.0, 2772.0];
/// The knobs' parameters, in the units' order.
pub const KNOB_PARAMS: [&str; 4] = [
    "pressure_mod",
    "pressure_filter",
    "timbre_mod",
    "timbre_filter",
];
const LEGENDS: [&str; 4] = ["MODULATION", "CUTOFF", "MODULATION", "CUTOFF"];
/// Each readout's cells: four, for MODULATION's "-100" (CUTOFF's "-2.5" needs three), the same
/// in all four so that their digits are one size.
pub const CELLS: usize = 4;
/// The groups' titles, each before its first unit (right-aligned this far left of its middle),
/// and their size.
const GROUPS: [(usize, &str); 2] = [(0, "PRESSURE"), (2, "TIMBRE")];
const TITLE_LEFT: f64 = 234.0;
const TITLE: f64 = 44.0;

/// What INPUT reads (the editor's, from what the audio thread heard lately).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Input {
    /// Nothing: the screen dark.
    #[default]
    None,
    /// Only ordinary channel pressure or CC 74, every voice's alike.
    Channel,
    /// Each note's own: MPE's member channels, polyphonic aftertouch, a host's note expressions.
    Mpe,
}

impl Input {
    pub fn text(self) -> &'static str {
        match self {
            Input::None => "",
            Input::Channel => "CHANNEL",
            Input::Mpe => "MPE",
        }
    }
}

/// Readout `i`'s window (by `UNITS`): left, top, width, height.
pub fn readout(i: usize) -> (f64, f64, f64, f64) {
    let x = UNITS[i.min(3)] + UNIT_READOUT;
    (
        x - READOUT.0 / 2.0,
        ROW - READOUT.1 / 2.0,
        READOUT.0,
        READOUT.1,
    )
}

/// What a pointer finds on the drawer (its knobs are the panel's controls:
/// `interact::hit_drawer`): a readout, which takes a value typed into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MpeTarget {
    Readout(usize),
}

/// Whether (`x`, `y`) is on the drawer, open.
pub fn inside(x: f64, y: f64) -> bool {
    (LEFT..RIGHT).contains(&x) && (TOP..TOP + DEPTH).contains(&y)
}

/// What a pointer at (`x`, `y`) finds on the drawer, open.
pub fn hit(x: f64, y: f64) -> Option<MpeTarget> {
    (0..4).find_map(|i| {
        let (rx, ry, rw, rh) = readout(i);
        ((rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y)).then_some(MpeTarget::Readout(i))
    })
}

/// A target's middle.
pub fn centre(t: MpeTarget) -> (f64, f64) {
    match t {
        MpeTarget::Readout(i) => {
            let (x, y, w, h) = readout(i);
            (x + w / 2.0, y + h / 2.0)
        }
    }
}

/// What the drawer shows.
#[derive(Clone, Debug, PartialEq)]
pub struct MpeScene {
    pub input: Input,
    /// Each readout's text, and whether it is lit (by `UNITS`).
    pub readouts: [(String, bool); 4],
    /// Each knob's value, normalized.
    pub knobs: [f64; 4],
}

impl Default for MpeScene {
    fn default() -> Self {
        MpeScene {
            input: Input::None,
            readouts: std::array::from_fn(|_| (String::new(), true)),
            knobs: [0.5; 4],
        }
    }
}

// ---- Drawing.

/// What does not change at a scale: the plate, and over it the print, the bezels and the glass;
/// the shadow it casts.
struct Still {
    look: Look,
    plate: Pixmap,
    shadow: Pixmap,
}

/// The drawer's renderer at the panel's scale.
pub struct MpeRenderer {
    scale: f64,
    skin: Skin,
    lettering: Lettering,
    options: usvg::Options<'static>,
    warming: Option<JoinHandle<Still>>,
    still: Option<Arc<Still>>,
    frame: Pixmap,
    readouts: [Option<Reading>; 4],
    /// Each knob as drawn: its value, and its pixels at their place in the frame.
    knobs: [Option<(f64, i32, i32, Pixmap)>; 4],
    shown: Option<MpeScene>,
    damage: Option<[i32; 4]>,
}

impl fmt::Debug for MpeRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MpeRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl MpeRenderer {
    /// At `scale`, in `skin` and `lettering`: what does not change made on a thread of its own,
    /// begun now (as the editor opens, so that the drawer's first drop waits for none of it).
    pub fn new(scale: f64, skin: Skin, lettering: Lettering) -> Self {
        let mut r = MpeRenderer {
            scale,
            skin,
            lettering,
            // (What it draws as it goes, the readouts' segments and the screen's dots, has no
            // lettering: no fonts loaded for it as the editor opens.)
            options: usvg::Options::default(),
            warming: None,
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            readouts: Default::default(),
            knobs: Default::default(),
            shown: None,
            damage: None,
        };
        r.warm();
        r
    }

    fn look(&self) -> Look {
        (self.scale.to_bits(), self.skin, self.lettering)
    }

    fn warm(&mut self) {
        let look = self.look();
        self.warming = std::thread::Builder::new()
            .name("ca72-mpe".into())
            .spawn(move || make_still(look))
            .ok();
    }

    /// Drawn again at `scale`.
    pub fn rescale(&mut self, scale: f64) {
        if scale != self.scale {
            self.scale = scale;
            self.forget();
        }
    }

    /// Drawn again in `skin` and `lettering`.
    pub fn set_look(&mut self, skin: Skin, lettering: Lettering) {
        if (skin, lettering) != (self.skin, self.lettering) {
            (self.skin, self.lettering) = (skin, lettering);
            self.forget();
        }
    }

    fn forget(&mut self) {
        self.still = None;
        self.readouts = Default::default();
        self.knobs = Default::default();
        self.shown = None;
        self.warm();
    }

    /// The frame: the drawer's, from its top (`top_px`), the drawing's width (only its plate,
    /// `LEFT` to `RIGHT`, is shown).
    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// The shadow it casts below its foot: `SHADOW` deep, from `LEFT` across, premultiplied, laid
    /// over what is under it.
    pub fn shadow(&self) -> Option<&Pixmap> {
        self.still.as_ref().map(|s| &s.shadow)
    }

    /// The pixels the last render changed in the frame: left, top, right, bottom.
    pub fn damage(&self) -> Option<[i32; 4]> {
        self.damage
    }

    /// Drawn for `scene`, its knobs as `panel` draws the controls: whether the frame changed.
    pub fn render(&mut self, scene: &MpeScene, panel: &mut Renderer) -> bool {
        self.damage = None;
        if self.shown.as_ref() == Some(scene) && self.still.is_some() {
            return false;
        }
        let mut dirty: Option<[i32; 4]> = None;
        if self.still.is_none() {
            let still = self.still_now();
            self.frame = still.plate.clone();
            self.still = Some(still);
            self.shown = None;
            self.knobs = Default::default();
            self.readouts = Default::default();
        }
        let all = [0, 0, self.frame.width() as i32, self.frame.height() as i32];
        let was = self.shown.clone();
        // The knobs that moved, drawn again (where each was, and is).
        for (k, param) in KNOB_PARAMS.iter().enumerate() {
            let v = scene.knobs[k];
            if self.knobs[k].as_ref().is_some_and(|(at, ..)| *at == v) {
                continue;
            }
            if let Some(part) = self.knob_part(k) {
                dirty = Some(union(dirty, part));
            }
            let top = top_px(self.scale);
            self.knobs[k] = crate::controls::index(param)
                .and_then(|i| panel.paint_control(i, v))
                .map(|(x, y, p)| (v, x, y - top, p));
            if let Some(part) = self.knob_part(k) {
                dirty = Some(union(dirty, part));
            }
        }
        match &was {
            None => dirty = Some(all),
            Some(was) => {
                if was.input != scene.input {
                    dirty = Some(union(dirty, self.input_part()));
                }
                for i in 0..4 {
                    if was.readouts[i] != scene.readouts[i] {
                        dirty = Some(union(dirty, self.readout_part(i)));
                    }
                }
            }
        }
        self.shown = Some(scene.clone());
        let Some(r) = dirty.and_then(|r| within(r, all)) else {
            return false;
        };
        let r = within(self.closed(r), all).unwrap_or(r);
        self.paint(scene, r);
        self.damage = Some(r);
        true
    }

    /// The still: the thread's, made as the editor opened (waited for if it is not done), if it
    /// is still the look; else made now.
    fn still_now(&mut self) -> Arc<Still> {
        if let Some(Ok(still)) = self.warming.take().map(JoinHandle::join)
            && still.look == self.look()
        {
            return Arc::new(still);
        }
        Arc::new(make_still(self.look()))
    }

    /// A rectangle of the drawing's units (left, top, right, bottom) as the frame's pixels, every
    /// pixel it touches and one more round it.
    fn pixels(&self, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> [i32; 4] {
        let (s, top) = (self.scale, top_px(self.scale));
        [
            (x0 * s).floor() as i32 - 1,
            (y0 * s).floor() as i32 - 1 - top,
            (x1 * s).ceil() as i32 + 1,
            (y1 * s).ceil() as i32 + 1 - top,
        ]
    }

    /// Where INPUT's dots and their glow draw.
    fn input_part(&self) -> [i32; 4] {
        let (x, y, w, h) = INPUT;
        self.pixels((x, y, x + w, y + h))
    }

    fn readout_part(&self, i: usize) -> [i32; 4] {
        let (x, y, w, h) = readout(i);
        let m = 10.0;
        self.pixels((x - m, y - m, x + w + m, y + h + m))
    }

    fn knob_part(&self, k: usize) -> Option<[i32; 4]> {
        self.knobs[k]
            .as_ref()
            .map(|(_, x, y, p)| [*x, *y, x + p.width() as i32, y + p.height() as i32])
    }

    /// `r` grown until every part that reaches into it lies inside it.
    fn closed(&self, mut r: [i32; 4]) -> [i32; 4] {
        let mut parts = vec![self.input_part()];
        parts.extend((0..4).map(|i| self.readout_part(i)));
        parts.extend((0..4).filter_map(|k| self.knob_part(k)));
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

    /// The frame drawn again inside `r`.
    fn paint(&mut self, scene: &MpeScene, r: [i32; 4]) {
        let Some(still) = self.still.clone() else {
            return;
        };
        let s = self.scale;
        let oy = f64::from(top_px(s)) / s;
        copy(&mut self.frame, &still.plate, r);
        // INPUT's dots over its glass (dark, its dots unlit, when nothing comes).
        if meets(self.input_part(), r) {
            let (x, y, w, h) = INPUT;
            screen_text(
                &mut self.frame,
                s,
                (x, y - oy, w, h),
                scene.input.text(),
                &Lamp::ORANGE,
            );
        }
        // The readouts.
        for i in 0..4 {
            if !meets(self.readout_part(i), r) {
                continue;
            }
            let (text, lit) = &scene.readouts[i];
            if self.readouts[i]
                .as_ref()
                .is_none_or(|d| d.text != *text || d.lit != *lit)
            {
                let (x, y, w, h) = readout(i);
                // (The built-in lamp's: a skin pack does not dress the drawer, R-SKIN-PACKS.)
                self.readouts[i] = readout_cells(
                    &self.options,
                    s,
                    (x, y - oy, w, h),
                    CELLS,
                    text,
                    *lit,
                    &Lamp::ORANGE,
                );
            }
            if let Some(d) = &self.readouts[i] {
                crate::render::put(&mut self.frame, &d.patch, (d.left, d.top), 1.0);
            }
        }
        // The knobs.
        for k in 0..4 {
            if let (Some(part), Some((_, x, y, p))) = (self.knob_part(k), &self.knobs[k])
                && meets(part, r)
            {
                crate::render::put(&mut self.frame, p, (*x, *y), 1.0);
            }
        }
    }
}

/// What never changes for `look`: the plate, the print worn into it, the bezels and glass, the
/// knobs' shadows; the shadow below.
fn make_still((scale, skin, lettering): Look) -> Still {
    let s = f64::from_bits(scale);
    let png = |b: &[u8]| Pixmap::decode_png(b).expect("a built-in picture decodes");
    let options = lower::lettered();
    let oy = f64::from(top_px(s)) / s;
    let mut plate = lower::plate(s, skin, DEPTH, &options);
    svg_over(&options, &mut plate, s, &surfaces(oy, skin));
    lower::print(&mut plate, s, skin, &options, &print_svg(oy, lettering));
    let glass_picture = png(crate::skin::GLASS);
    for i in 0..4 {
        let (x, y, rw, rh) = readout(i);
        glass(&mut plate, &glass_picture, s, (x, y - oy, rw, rh), 5.0);
    }
    let (x, y, w, h) = INPUT;
    screen_still(&options, &mut plate, &glass_picture, s, (x, y - oy, w, h));
    lower::lamp(&mut plate, s, skin, (0, top_px(s)));
    let shadow = lower::cast(s, &options);
    Still {
        look: (scale, skin, lettering),
        plate,
        shadow,
    }
}

/// The plate's surfaces (the frame's units, `oy` its top in the drawing's): the readouts'
/// bezels, as the strip's; the knobs' shadows (the worn skin's, as the panel's).
fn surfaces(oy: f64, skin: Skin) -> String {
    let mut s = Svg::default();
    let y = |v: f64| N(v - oy);
    put!(
        s,
        "<defs><filter id='long' x='-0.6' y='-0.6' width='2.2' height='2.2'><feGaussianBlur stdDeviation='7'/></filter><filter id='softer' x='-0.5' y='-0.5' width='2' height='2'><feGaussianBlur stdDeviation='2'/></filter></defs>"
    );
    for i in 0..4 {
        let (x, ry, w, h) = readout(i);
        put!(
            s,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='9' fill='#0b0a09'/><rect x='{}' y='{}' width='{}' height='{}' rx='8.5' fill='none' stroke='#fff' stroke-opacity='0.07' stroke-width='1.5'/>",
            N(x - 6.0),
            y(ry - 6.0),
            N(w + 12.0),
            N(h + 12.0),
            N(x - 5.0),
            y(ry - 5.0),
            N(w + 10.0),
            N(h + 10.0)
        );
    }
    if skin == Skin::Worn {
        for u in UNITS {
            let (x, r) = (u + super::UNIT_KNOB, art::STD.skirt);
            put!(
                s,
                "<circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.55' filter='url(#long)'/><circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.7' filter='url(#softer)'/>",
                N(x + 10.0),
                y(ROW + 14.0),
                N(r),
                N(x + 2.0),
                y(ROW + 3.0),
                N(r)
            );
        }
    }
    s.0
}

/// The plate's print (the frame's units): the rules, INPUT's legend, the groups' titles, the
/// knobs' legends and dials, the readouts' units.
fn print_svg(oy: f64, lettering: Lettering) -> String {
    let mut s = Svg(String::new(), lettering);
    let (legend, small) = (lettering.strip_legend(), lettering.strip_small());
    let text = |s: &mut Svg, x: f64, y: f64, t: &str, size: f64, anchor: &str| {
        put!(
            s,
            "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' font-weight='700' text-anchor='{anchor}' dominant-baseline='central' fill='{}'>{}</text>",
            N(x),
            N(y - oy),
            N(size),
            colour::LEGEND,
            t
        );
    };
    for x in RULES {
        put!(
            s,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{}' stroke-width='4' stroke-linecap='round'/>",
            N(x),
            N(TOP + 16.0 - oy),
            N(x),
            N(TOP + DEPTH - 14.0 - oy),
            colour::LEGEND
        );
    }
    text(&mut s, INPUT_X, ROW - LEGEND_UP, "INPUT", legend, "middle");
    for (u, t) in GROUPS {
        text(&mut s, UNITS[u] - TITLE_LEFT, ROW, t, TITLE, "end");
    }
    for (k, (u, t)) in UNITS.iter().zip(LEGENDS).enumerate() {
        let x = u + super::UNIT_KNOB;
        text(&mut s, x, ROW - LEGEND_UP, t, legend, "middle");
        let dial = if k % 2 == 0 {
            crate::controls::Dial::Reach
        } else {
            crate::controls::Dial::Cutoff
        };
        art::draw_dial(&mut s, x, ROW - oy, dial);
        let (rx, ry, rw, rh) = readout(k);
        if k % 2 == 0 {
            text(&mut s, rx + rw + 18.0, ry + rh / 2.0, "%", small, "middle");
        } else {
            text(
                &mut s,
                rx + rw + 36.0,
                ry + rh / 2.0,
                "OCT",
                small,
                "middle",
            );
        }
    }
    s.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each readout is found where it is drawn, within the drawer; nothing past its ends; the
    /// units clear of each other, of the rules and of the titles; INPUT before the first rule.
    #[test]
    fn the_drawer_finds_each_part_where_it_is_drawn() {
        for i in 0..4 {
            let at = centre(MpeTarget::Readout(i));
            assert_eq!(hit(at.0, at.1), Some(MpeTarget::Readout(i)));
            assert!(inside(at.0, at.1));
        }
        assert_eq!(hit(INPUT.0 + 10.0, ROW), None);
        let half = super::super::UNIT_HALF;
        for w in UNITS.windows(2) {
            assert!(w[1] - w[0] >= 2.0 * half);
        }
        const {
            assert!(INPUT.0 > LEFT && INPUT.0 + INPUT.2 < RULES[0]);
            // Each group between its rules, its title (some 230 units) before its first knob's
            // dial.
            assert!(UNITS[0] - TITLE_LEFT - 230.0 > RULES[0]);
            assert!(UNITS[2] - TITLE_LEFT - 230.0 > RULES[1]);
        }
        assert!(UNITS[1] + half < RULES[1] && UNITS[3] + half < RIGHT);
        // CUTOFF's OCT (some 50 units wide) inside the trim.
        let (rx, _, rw, _) = readout(3);
        assert!(rx + rw + 36.0 + 30.0 < RIGHT);
        assert!(!inside(LEFT - 1.0, ROW) && !inside(RIGHT + 1.0, ROW));
        assert!(!inside(1000.0, TOP + DEPTH + 1.0) && !inside(1000.0, TOP - 1.0));
        assert!(ROW - LEGEND_UP > TOP + 14.0 && ROW + READOUT.1 / 2.0 < TOP + DEPTH - 9.0);
    }

    /// The drawer drawn in parts is the drawer drawn whole: INPUT's screen lit and dark, a
    /// readout changed and a knob turned, at the window's opening scale and a Retina screen's, in
    /// both skins.
    #[test]
    fn drawn_in_parts_it_is_drawn_whole() {
        for (scale, skin) in [(0.44, Skin::Worn), (0.87, Skin::Drawn)] {
            let mut panel = Renderer::with_skin(skin, scale, 1.0);
            let mut r = MpeRenderer::new(scale, skin, Lettering::Standard);
            let mut scene = MpeScene {
                readouts: [
                    ("0".into(), true),
                    ("0.0".into(), true),
                    ("0".into(), true),
                    ("0.0".into(), true),
                ],
                ..MpeScene::default()
            };
            assert!(r.render(&scene, &mut panel));
            let steps: [&dyn Fn(&mut MpeScene); 5] = [
                &|s| s.input = Input::Mpe,
                &|s| s.readouts[0] = ("-100".into(), true),
                &|s| s.knobs[3] = 0.8,
                &|s| s.input = Input::Channel,
                &|s| {
                    s.input = Input::None;
                    s.readouts[3] = ("2.5".into(), true);
                },
            ];
            for step in steps {
                step(&mut scene);
                assert!(r.render(&scene, &mut panel));
                assert!(r.damage().is_some());
                let mut whole = MpeRenderer::new(scale, skin, Lettering::Standard);
                whole.render(&scene, &mut panel);
                assert!(
                    r.frame() == whole.frame(),
                    "{scale} {skin:?}: {scene:?} drawn in parts"
                );
            }
            assert!(!r.render(&scene, &mut panel), "nothing changed");
        }
    }
}
