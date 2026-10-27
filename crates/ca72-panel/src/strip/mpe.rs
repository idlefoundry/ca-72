//! The drawer of each note's own expression (decisions.md R-MPE; the owner's A, 2026-10-20): one
//! row dropped down from under the rail by the rail's MPE key, inside the window, over the top of
//! whichever lower page shows, as the arpeggiator's is (`arp`, `lower`). Left to right: INPUT, a
//! pixel screen of the rail's kind reading what expression comes in; a rule; PRESSURE's
//! MODULATION and CUTOFF; a rule; TIMBRE's MODULATION and CUTOFF, each a knob and its readout,
//! the strip's own parts. The two groups' titles are centred below them, as GLOBAL's are.
//!
//! A second row under them (decisions.md R-VELOCITY) has the notes' velocity: LAST NOTE, a pixel
//! screen in INPUT's column reading the latest note's; RESPONSE, a screen drawing how a note's
//! velocity becomes the velocity source's value (its CURVE and FLOOR, the latest note's on it);
//! MODULATES, a pixel screen of what it moves, as an LFO's; and CURVE and FLOOR, each a knob and
//! its readout, under TIMBRE's. Its title, VELOCITY, centred below it all.
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
use crate::render::Renderer;
use crate::skin::Skin;
use crate::svg::{N, Svg, colour, put};

// ---- The layout (the drawing's units, the drawer open; the mock-up's).

/// A row's depth: room below its controls for GLOBAL's title gaps (R-DRAWER-LETTERING).
pub const ROW_DEPTH: f64 = 340.0;
/// The drawer's: two rows, the expression's and the velocity's (decisions.md R-VELOCITY).
pub const DEPTH: f64 = 2.0 * ROW_DEPTH;
/// The controls keep the arpeggiator's row; the velocity's a row's depth below.
pub const ROW: f64 = super::arp::ROW;
pub const ROW2: f64 = ROW + ROW_DEPTH;
const LEGEND_UP: f64 = 109.0;
/// INPUT's screen: left, top, width, height.
pub const INPUT: (f64, f64, f64, f64) = (INPUT_X - 120.0, ROW - 25.0, 240.0, 50.0);
const INPUT_X: f64 = 536.0;
/// The rules: after INPUT, and between PRESSURE's units and TIMBRE's.
const RULES: [f64; 2] = [700.0, 1920.0];
// The four-cell window, its % print and the next dial's outer numbers need 450 units
// between middles in LARGE (the measured ink extents are checked below).
const UNIT_PITCH: f64 = 450.0;
// Each pair's full printed extent, including OCT, is centred 270 units after its first
// middle in both lettering sizes (within half a unit; checked below).
const PAIR_CENTRE: f64 = 270.0;
const PRESSURE: f64 = (RULES[0] + RULES[1]) / 2.0 - PAIR_CENTRE;
const TIMBRE: f64 = (RULES[1] + RIGHT) / 2.0 - PAIR_CENTRE;
/// The units' middles: PRESSURE's MODULATION and CUTOFF, TIMBRE's MODULATION and CUTOFF; then,
/// on the second row under TIMBRE's, the velocity's CURVE and FLOOR. Each knob is `UNIT_KNOB`
/// left of it; its readout grows rightwards from the strip's window.
pub const UNITS: [f64; UNIT_COUNT] = [
    PRESSURE,
    PRESSURE + UNIT_PITCH,
    TIMBRE,
    TIMBRE + UNIT_PITCH,
    TIMBRE,
    TIMBRE + UNIT_PITCH,
];
pub const UNIT_COUNT: usize = 6;
/// The knobs' parameters, in the units' order.
pub const KNOB_PARAMS: [&str; UNIT_COUNT] = [
    "pressure_mod",
    "pressure_filter",
    "timbre_mod",
    "timbre_filter",
    "velocity_curve",
    "velocity_floor",
];
const LEGENDS: [&str; UNIT_COUNT] = [
    "MODULATION",
    "CUTOFF",
    "MODULATION",
    "CUTOFF",
    "CURVE",
    "FLOOR",
];

/// Unit `i`'s row: the expression's, else the velocity's.
pub const fn unit_row(i: usize) -> f64 {
    if i < 4 { ROW } else { ROW2 }
}

/// The velocity's row's screens (decisions.md R-VELOCITY): LAST NOTE in INPUT's column, RESPONSE
/// (the curve) and MODULATES: left, top, width, height.
pub const LAST: (f64, f64, f64, f64) = (INPUT_X - 120.0, ROW2 - 25.0, 240.0, 50.0);
pub const RESPONSE: (f64, f64, f64, f64) = (RESPONSE_X - 220.0, ROW2 - 80.0, 440.0, 160.0);
const RESPONSE_X: f64 = 1060.0;
pub const MODULATES: (f64, f64, f64, f64) = (MODULATES_X - 166.0, ROW2 - 25.0, 332.0, 50.0);
const MODULATES_X: f64 = 1620.0;
/// The velocity's colour as its screens light it (`velocity::RGB`), and dimmed.
const VELOCITY_LAMP: (u8, u8, u8) = super::worn::ORANGE;
/// MODULATES dim (NOT IN USE), as an LFO's.
const VELOCITY_DIM: (u8, u8, u8) = (120, 70, 30);

/// The RESPONSE screen's curve: CURVE (-1..1) and FLOOR (0..1) as their knobs have them, and the
/// latest note's velocity (0..1) where it falls, none before a note.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Response {
    pub curve: f64,
    pub floor: f64,
    pub point: Option<f64>,
}

impl Response {
    /// The source's value for velocity `v`, as the engine makes it (`lfo::Response`).
    pub fn at(&self, v: f64) -> f64 {
        let v = v.clamp(0.0, 1.0);
        let curve = self.curve.clamp(-1.0, 1.0);
        let shaped = if curve == 0.0 {
            v
        } else {
            v.powf(3f64.powf(curve))
        };
        let floor = self.floor.clamp(0.0, 1.0);
        floor + (1.0 - floor) * shaped
    }
}
/// Each readout's cells: four, for MODULATION's "-100" (CUTOFF's "-2.5" needs three), the same
/// in all four. Their windows grow to keep the strip's digit pitch and height.
pub const CELLS: usize = 4;
/// The groups' section titles, centred below the units as GLOBAL's are.
const GROUPS: [(usize, &str); 2] = [(0, "PRESSURE"), (2, "TIMBRE")];

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
    let i = i.min(UNIT_COUNT - 1);
    let x = UNITS[i] + UNIT_READOUT;
    (
        // Keep the strip's clearance from the knob's outer numbers; the extra cell grows
        // rightwards. Centring the wider window on `x` would cover MODULATION's 50.
        x - READOUT.0 / 2.0,
        unit_row(i) - READOUT.1 / 2.0,
        super::readout_width(CELLS),
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
    (0..UNIT_COUNT).find_map(|i| {
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
    pub readouts: [(String, bool); UNIT_COUNT],
    /// Each knob's value, normalized.
    pub knobs: [f64; UNIT_COUNT],
    /// The velocity's row (decisions.md R-VELOCITY): LAST NOTE's text (the latest note's
    /// velocity, MIDI's 0 to 127; none before a note), MODULATES' (and whether lit, as an
    /// LFO's), the RESPONSE screen's curve.
    pub last: String,
    pub modulates: (String, bool),
    pub response: Response,
}

impl Default for MpeScene {
    fn default() -> Self {
        MpeScene {
            input: Input::None,
            readouts: std::array::from_fn(|_| (String::new(), true)),
            knobs: [0.5; UNIT_COUNT],
            last: String::new(),
            modulates: ("NOT IN USE".into(), false),
            response: Response::default(),
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
    readouts: [Option<Reading>; UNIT_COUNT],
    /// Each knob as drawn: its value, and its pixels at their place in the frame.
    knobs: [Option<(f64, i32, i32, Pixmap)>; UNIT_COUNT],
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
                if was.last != scene.last {
                    dirty = Some(union(dirty, self.screen_part(LAST)));
                }
                if was.modulates != scene.modulates {
                    dirty = Some(union(dirty, self.screen_part(MODULATES)));
                }
                if was.response != scene.response {
                    dirty = Some(union(dirty, self.screen_part(RESPONSE)));
                }
                for i in 0..UNIT_COUNT {
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
        self.screen_part(INPUT)
    }

    /// Where a screen's dots (or the RESPONSE screen's curve) and their glow draw.
    fn screen_part(&self, (x, y, w, h): (f64, f64, f64, f64)) -> [i32; 4] {
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
        let mut parts = vec![
            self.input_part(),
            self.screen_part(LAST),
            self.screen_part(MODULATES),
            self.screen_part(RESPONSE),
        ];
        parts.extend((0..UNIT_COUNT).map(|i| self.readout_part(i)));
        parts.extend((0..UNIT_COUNT).filter_map(|k| self.knob_part(k)));
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
        // The velocity's row: LAST NOTE, MODULATES and the RESPONSE screen's curve.
        let on = Lamp::ORANGE;
        if meets(self.screen_part(LAST), r) {
            let (x, y, w, h) = LAST;
            screen_text(&mut self.frame, s, (x, y - oy, w, h), &scene.last, &on);
        }
        if meets(self.screen_part(MODULATES), r) {
            let (x, y, w, h) = MODULATES;
            let (text, lit) = &scene.modulates;
            let lamp = if *lit { on } else { Lamp::of(VELOCITY_DIM) };
            screen_text(&mut self.frame, s, (x, y - oy, w, h), text, &lamp);
        }
        if meets(self.screen_part(RESPONSE), r) {
            svg_over(
                &self.options,
                &mut self.frame,
                s,
                &response_svg(&scene.response, oy),
            );
        }
        // The readouts.
        for i in 0..UNIT_COUNT {
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
        for k in 0..UNIT_COUNT {
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
    for i in 0..UNIT_COUNT {
        let (x, y, rw, rh) = readout(i);
        glass(&mut plate, &glass_picture, s, (x, y - oy, rw, rh), 5.0);
    }
    for (x, y, w, h) in [INPUT, LAST, MODULATES] {
        screen_still(&options, &mut plate, &glass_picture, s, (x, y - oy, w, h));
    }
    let (x, y, w, h) = RESPONSE;
    glass(&mut plate, &glass_picture, s, (x, y - oy, w, h), 12.0);
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
    // (The RESPONSE screen's bezel and face, as a readout's, larger.)
    let (rx, ry, rw, rh) = RESPONSE;
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='16' fill='#0b0a09'/><rect x='{}' y='{}' width='{}' height='{}' rx='15' fill='none' stroke='#fff' stroke-opacity='0.07' stroke-width='1.5'/><rect x='{}' y='{}' width='{}' height='{}' rx='12' fill='#100a0e'/>",
        N(rx - 8.0),
        y(ry - 8.0),
        N(rw + 16.0),
        N(rh + 16.0),
        N(rx - 7.0),
        y(ry - 7.0),
        N(rw + 14.0),
        N(rh + 14.0),
        N(rx),
        y(ry),
        N(rw),
        N(rh)
    );
    for i in 0..UNIT_COUNT {
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
        for (i, u) in UNITS.into_iter().enumerate() {
            let (x, r) = (u + super::UNIT_KNOB, art::STD.skirt);
            let row = unit_row(i);
            put!(
                s,
                "<circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.55' filter='url(#long)'/><circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.7' filter='url(#softer)'/>",
                N(x + 10.0),
                y(row + 14.0),
                N(r),
                N(x + 2.0),
                y(row + 3.0),
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
    let text = |s: &mut Svg, x: f64, y: f64, t: &str, size: f64| {
        art::text(s, x, y - oy, t, size, art::Style::Legend, false, "");
    };
    // (After INPUT and LAST NOTE, the whole depth; between PRESSURE and TIMBRE, theirs; and
    // across, between the rows.)
    let line = |s: &mut Svg, (x1, y1): (f64, f64), (x2, y2): (f64, f64)| {
        put!(
            s,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{}' stroke-width='4' stroke-linecap='round'/>",
            N(x1),
            N(y1 - oy),
            N(x2),
            N(y2 - oy),
            colour::LEGEND
        );
    };
    line(
        &mut s,
        (RULES[0], TOP + 16.0),
        (RULES[0], TOP + DEPTH - 14.0),
    );
    line(
        &mut s,
        (RULES[1], TOP + 16.0),
        (RULES[1], TOP + ROW_DEPTH - 14.0),
    );
    line(
        &mut s,
        (LEFT + 24.0, TOP + ROW_DEPTH),
        (RIGHT - 24.0, TOP + ROW_DEPTH),
    );
    text(&mut s, INPUT_X, ROW - LEGEND_UP, "INPUT", legend);
    text(&mut s, INPUT_X, ROW2 - LEGEND_UP, "LAST NOTE", legend);
    text(&mut s, RESPONSE_X, ROW2 - LEGEND_UP, "RESPONSE", legend);
    text(&mut s, MODULATES_X, ROW2 - LEGEND_UP, "MODULATES", legend);
    for (g, (u, t)) in GROUPS.into_iter().enumerate() {
        let right = RULES.get(g + 1).copied().unwrap_or(RIGHT);
        put!(s, "<g id='mpe-title-{u}'>");
        art::text(
            &mut s,
            (RULES[g] + right) / 2.0,
            TOP + ROW_DEPTH - 50.0 - oy,
            t,
            super::TITLE,
            art::Style::Title,
            false,
            "",
        );
        s.0.push_str("</g>");
    }
    put!(s, "<g id='mpe-title-velocity'>");
    art::text(
        &mut s,
        (RULES[0] + RIGHT) / 2.0,
        TOP + DEPTH - 50.0 - oy,
        "VELOCITY",
        super::TITLE,
        art::Style::Title,
        false,
        "",
    );
    s.0.push_str("</g>");
    for (k, (u, t)) in UNITS.iter().zip(LEGENDS).enumerate() {
        put!(s, "<g id='mpe-unit-{k}'>");
        let x = u + super::UNIT_KNOB;
        let row = unit_row(k);
        text(&mut s, x, row - LEGEND_UP, t, legend);
        let dial = match k {
            4 => crate::controls::Dial::Reach,
            5 => crate::controls::Dial::Ten,
            _ if k % 2 == 0 => crate::controls::Dial::Reach,
            _ => crate::controls::Dial::Cutoff,
        };
        put!(s, "<g id='mpe-dial-{k}'>");
        art::draw_dial(&mut s, x, row - oy, dial);
        s.0.push_str("</g>");
        let (rx, ry, rw, rh) = readout(k);
        if k % 2 == 0 || k >= 4 {
            text(&mut s, rx + rw + 18.0, ry + rh / 2.0, "%", small);
        } else {
            text(&mut s, rx + rw + 36.0, ry + rh / 2.0, "OCT", small);
        }
        s.0.push_str("</g>");
    }
    s.0
}

/// The RESPONSE screen's curve (the frame's units, `oy` its top in the drawing's): a faint
/// graticule, FLOOR's line, the curve glowing in the velocity's colour, and the latest note's
/// place on it.
fn response_svg(r: &Response, oy: f64) -> String {
    let (x, y, w, h) = RESPONSE;
    let (pad, y) = (18.0, y - oy);
    let (x0, x1, y0, y1) = (x + pad, x + w - pad, y + h - pad, y + pad);
    let (cr, cg, cb) = VELOCITY_LAMP;
    let c = format!("#{cr:02x}{cg:02x}{cb:02x}");
    let mut s = Svg::default();
    put!(
        s,
        "<defs><filter id='rglow' x='-0.2' y='-0.2' width='1.4' height='1.4'><feGaussianBlur stdDeviation='4'/></filter></defs>"
    );
    for i in 1..8 {
        let gx = x + w * f64::from(i) / 8.0;
        put!(
            s,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{c}' stroke-opacity='0.1' stroke-width='1.5'/>",
            N(gx),
            N(y),
            N(gx),
            N(y + h)
        );
    }
    for i in 1..4 {
        let gy = y + h * f64::from(i) / 4.0;
        put!(
            s,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{c}' stroke-opacity='0.1' stroke-width='1.5'/>",
            N(x),
            N(gy),
            N(x + w),
            N(gy)
        );
    }
    let at = |v: f64| (x0 + (x1 - x0) * v, y0 + (y1 - y0) * r.at(v));
    let points: Vec<String> = (0..=64)
        .map(|i| {
            let (px, py) = at(f64::from(i) / 64.0);
            format!("{},{}", N(px), N(py))
        })
        .collect();
    let floor = y0 + (y1 - y0) * r.floor.clamp(0.0, 1.0);
    put!(
        s,
        "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{c}' stroke-opacity='0.35' stroke-width='2' stroke-dasharray='6 6'/>",
        N(x0),
        N(floor),
        N(x1),
        N(floor)
    );
    let line = points.join(" ");
    put!(
        s,
        "<polyline points='{line}' fill='none' stroke='{c}' stroke-width='7' stroke-opacity='0.5' filter='url(#rglow)'/><polyline points='{line}' fill='none' stroke='{c}' stroke-width='3.5' stroke-linejoin='round'/>"
    );
    if let Some(v) = r.point.filter(|v| v.is_finite()) {
        let (px, py) = at(v);
        put!(
            s,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{c}' stroke-opacity='0.4' stroke-width='2' stroke-dasharray='3 5'/><circle cx='{}' cy='{}' r='10' fill='{c}' fill-opacity='0.6' filter='url(#rglow)'/><circle cx='{}' cy='{}' r='6' fill='#fff'/>",
            N(px),
            N(y0),
            N(px),
            N(py),
            N(px),
            N(py),
            N(px),
            N(py)
        );
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
        const {
            assert!(INPUT.0 > LEFT && INPUT.0 + INPUT.2 < RULES[0]);
            assert!(TOP + DEPTH + SHADOW < art::H);
        }
        assert!(inside(1000.0, TOP + DEPTH - 1.0));
        assert!(!inside(LEFT - 1.0, ROW) && !inside(RIGHT + 1.0, ROW));
        assert!(!inside(1000.0, TOP + DEPTH + 1.0) && !inside(1000.0, TOP - 1.0));
        assert!(ROW - LEGEND_UP > TOP + 14.0 && ROW + READOUT.1 / 2.0 < TOP + DEPTH - 9.0);
    }

    /// Use the actual printed extents, including LARGE's outer dial numbers and units, rather
    /// than the strip's old three-cell unit width. The titles clear their rules by at least
    /// GLOBAL's smallest section-title gap, and every printed part and bezel by six units.
    /// Both vertical gaps match at least GLOBAL's largest, measured to control ink and bezels;
    /// each pair's full printed extent is centred between its section's rules or edge.
    #[test]
    fn the_wider_readouts_and_titles_clear_the_print() {
        fn parts(node: &usvg::Node, boxes: &mut Vec<usvg::Rect>) {
            if let usvg::Node::Group(g) = node {
                for child in g.children() {
                    parts(child, boxes);
                }
            } else {
                boxes.push(node.abs_stroke_bounding_box());
            }
        }
        let clears = |a: usvg::Rect, b: usvg::Rect| {
            a.right() + 6.0 < b.left()
                || b.right() + 6.0 < a.left()
                || a.bottom() + 6.0 < b.top()
                || b.bottom() + 6.0 < a.top()
        };
        for lettering in [Lettering::Standard, Lettering::Large] {
            let doc = art::document(
                [0.0, 0.0, art::W, art::H],
                3108,
                1775,
                &print_svg(0.0, lettering),
            );
            let tree = usvg::Tree::from_str(&doc, &lower::lettered()).expect("the print parses");
            // (After INPUT the whole depth; between PRESSURE and TIMBRE, their row's.)
            for (rule, depth) in [(RULES[0], DEPTH), (RULES[1], ROW_DEPTH)] {
                let ink = tree
                    .root()
                    .children()
                    .iter()
                    .map(usvg::Node::abs_stroke_bounding_box)
                    .find(|r| (f64::from(r.left()) - rule + 2.0).abs() < 0.01)
                    .expect("a section rule");
                assert!((f64::from(ink.top()) - TOP - 14.0).abs() < 0.01);
                assert!((f64::from(ink.bottom()) - TOP - depth + 12.0).abs() < 0.01);
            }
            let bounds = |id: &str| {
                tree.node_by_id(id)
                    .expect("the printed part")
                    .abs_stroke_bounding_box()
            };
            let mut global = Svg(String::new(), lettering);
            super::super::print(&mut global);
            let global = usvg::Tree::from_str(
                &art::document([0.0, 0.0, art::W, art::H], 3108, 1775, &global.0),
                &lower::lettered(),
            )
            .expect("GLOBAL's print parses");
            let mut controls = Svg(String::new(), lettering);
            for c in crate::controls::CONTROLS
                .iter()
                .filter(|c| c.place == crate::controls::Place::Strip)
            {
                if let crate::controls::Kind::Knob { dial, .. } = c.kind {
                    art::draw_dial(&mut controls, c.x, c.y, dial);
                }
            }
            let controls = usvg::Tree::from_str(
                &art::document([0.0, 0.0, art::W, art::H], 3108, 1775, &controls.0),
                &lower::lettered(),
            )
            .expect("GLOBAL's controls parse");
            let mut ink = Vec::new();
            for child in controls.root().children() {
                parts(child, &mut ink);
            }
            let (mut global_above, mut global_foot) = (0.0_f64, 0.0_f64);
            for (left, right, title) in super::super::SECTIONS.into_iter().skip(1) {
                let title_ink = global
                    .node_by_id(&format!("strip-title-{title}"))
                    .expect("a title")
                    .abs_stroke_bounding_box();
                let mut bottom = ink
                    .iter()
                    .filter(|r| f64::from(r.left()) >= left && f64::from(r.right()) <= right)
                    .map(|r| f64::from(r.bottom()))
                    .fold(0.0, f64::max);
                for r in super::super::Readout::ALL {
                    let (x, y, w, h) = r.window();
                    if x >= left && x + w <= right {
                        bottom = bottom.max(y + h + 6.0);
                    }
                }
                let (x, y, w, h) = super::super::DISPLAY;
                if x >= left && x + w <= right {
                    bottom = bottom.max(y + h + 6.0);
                }
                let above = f64::from(title_ink.top()) - bottom;
                let foot = art::H - f64::from(title_ink.bottom());
                global_above = global_above.max(above);
                global_foot = global_foot.max(foot);
                eprintln!("{lettering:?} GLOBAL {title}: above {above}, foot {foot}");
            }
            let global_gap = super::super::SECTIONS
                .into_iter()
                .skip(1)
                .map(|(left, right, title)| {
                    let ink = global
                        .node_by_id(&format!("strip-title-{title}"))
                        .expect("GLOBAL's section title")
                        .abs_stroke_bounding_box();
                    (f64::from(ink.left()) - left - 2.0).min(right - 2.0 - f64::from(ink.right()))
                })
                .fold(f64::INFINITY, f64::min);
            assert!(global_gap > 200.0, "GLOBAL's measured gap: {global_gap}");
            assert!(global_above > 30.0 && global_foot > 30.0);
            for (group, _) in GROUPS {
                let (left, right) = if group == 0 {
                    (RULES[0], RULES[1])
                } else {
                    (RULES[1], RIGHT)
                };
                let title = bounds(&format!("mpe-title-{group}"));
                assert!(
                    f64::from(title.left()) - left - 2.0 >= global_gap
                        && right - 2.0 - f64::from(title.right()) >= global_gap,
                    "{lettering:?}: {title:?}, GLOBAL's gap {global_gap}"
                );
                assert!(f64::from(title.top()) > TOP + 14.0);
                assert!(f64::from(title.bottom()) < TOP + ROW_DEPTH - 9.0);
                let mut end = left + 2.0;
                let mut first = f64::INFINITY;
                let mut bottom = 0.0_f64;
                for i in group..group + 2 {
                    let ink = bounds(&format!("mpe-unit-{i}"));
                    let (x, _, w, _) = readout(i);
                    let dial = bounds(&format!("mpe-dial-{i}"));
                    assert!(
                        x - 6.0 > f64::from(dial.right()) + 6.0,
                        "{lettering:?} unit {i}: window {x} after dial {dial:?}"
                    );
                    assert_eq!(w, 198.0);
                    let start = f64::from(ink.left()).min(x - 6.0);
                    first = first.min(start);
                    assert!(
                        start > end + 6.0,
                        "{lettering:?} unit {i}: {start} after {end}"
                    );
                    end = f64::from(ink.right()).max(x + w + 6.0);
                    assert!(f64::from(ink.top()) > TOP + 14.0);
                    assert!(f64::from(ink.bottom()) < TOP + ROW_DEPTH - 9.0);
                    let mut print = Vec::new();
                    parts(
                        tree.node_by_id(&format!("mpe-unit-{i}")).expect("a unit"),
                        &mut print,
                    );
                    for part in print {
                        assert!(
                            clears(title, part),
                            "{lettering:?}: title {title:?}, print {part:?}"
                        );
                    }
                    let (_, y, _, h) = readout(i);
                    bottom = bottom.max(f64::from(ink.bottom())).max(y + h + 6.0);
                    let bezel = usvg::Rect::from_xywh(
                        (x - 6.0) as f32,
                        (y - 6.0) as f32,
                        (w + 12.0) as f32,
                        (h + 12.0) as f32,
                    )
                    .expect("a bezel");
                    assert!(
                        clears(title, bezel),
                        "{lettering:?}: title {title:?}, bezel {bezel:?}"
                    );
                }
                assert!(end < right - 8.0, "{lettering:?}: {end} before {right}");
                assert!(
                    ((first + end - left - right) / 2.0).abs() < 0.5,
                    "{lettering:?}: pair {first}..{end} centred between {left}..{right}"
                );
                let above = f64::from(title.top()) - bottom;
                let foot = TOP + ROW_DEPTH - f64::from(title.bottom());
                assert!(
                    above >= global_above,
                    "{lettering:?}: title gap {above}, GLOBAL {global_above}"
                );
                assert!(
                    foot >= global_foot,
                    "{lettering:?}: foot gap {foot}, GLOBAL {global_foot}"
                );
                eprintln!(
                    "{lettering:?} MPE group {group}: above {above}, foot {foot}, pair {first}..{end}"
                );
            }
            // The velocity's row (decisions.md R-VELOCITY): its units and screens within it,
            // clear of its title, which keeps GLOBAL's gaps to them and to the drawer's foot.
            let title = bounds("mpe-title-velocity");
            let mut bottom = 0.0_f64;
            for i in 4..UNIT_COUNT {
                let ink = bounds(&format!("mpe-unit-{i}"));
                assert!(
                    f64::from(ink.top()) > TOP + ROW_DEPTH + 14.0,
                    "{lettering:?}"
                );
                assert!(f64::from(ink.bottom()) < TOP + DEPTH - 9.0, "{lettering:?}");
                let (_, y, _, h) = readout(i);
                bottom = bottom.max(f64::from(ink.bottom())).max(y + h + 6.0);
            }
            for (_, y, _, h) in [LAST, RESPONSE, MODULATES] {
                assert!(y > TOP + ROW_DEPTH + 14.0);
                bottom = bottom.max(y + h + 8.0);
            }
            let above = f64::from(title.top()) - bottom;
            let foot = TOP + DEPTH - f64::from(title.bottom());
            assert!(
                above >= global_above && foot >= global_foot,
                "{lettering:?}: VELOCITY's gaps {above}, {foot}; GLOBAL's {global_above}, {global_foot}"
            );
            let (rx, _, rw, _) = RESPONSE;
            let (mx, _, mw, _) = MODULATES;
            assert!(rx > RULES[0] + 20.0 && rx + rw + 20.0 < mx);
            let curve = bounds("mpe-dial-4");
            assert!(mx + mw + 20.0 < f64::from(curve.left()), "{lettering:?}");
        }
    }

    /// The drawer drawn in parts is the drawer drawn whole: INPUT's screen lit and dark, a
    /// readout changed and a knob turned, at the window's opening scale and a Retina screen's, in
    /// both skins.
    #[test]
    fn drawn_in_parts_it_is_drawn_whole() {
        for (scale, skin, lettering) in [
            (0.44, Skin::Worn, Lettering::Standard),
            (1.0, Skin::Worn, Lettering::Standard),
            (0.44, Skin::Drawn, Lettering::Standard),
            (1.0, Skin::Drawn, Lettering::Standard),
            (0.87, Skin::Drawn, Lettering::Standard),
            (0.44, Skin::Worn, Lettering::Large),
            (1.0, Skin::Worn, Lettering::Large),
            (0.44, Skin::Drawn, Lettering::Large),
            (1.0, Skin::Drawn, Lettering::Large),
        ] {
            let mut panel = Renderer::with_look(skin, lettering, scale, 1.0);
            let mut r = MpeRenderer::new(scale, skin, lettering);
            let mut scene = MpeScene {
                readouts: [
                    ("0".into(), true),
                    ("0.0".into(), true),
                    ("0".into(), true),
                    ("0.0".into(), true),
                    ("0".into(), true),
                    ("0".into(), true),
                ],
                ..MpeScene::default()
            };
            assert!(r.render(&scene, &mut panel));
            assert_eq!(r.frame().height(), (DEPTH * scale).ceil() as u32);
            assert!(
                top_px(scale)
                    + r.frame().height() as i32
                    + (r.shadow().expect("the drawer's shadow").height() as i32)
                    < (art::H * scale).round() as i32
            );
            let steps: [&dyn Fn(&mut MpeScene); 10] = [
                &|s| s.input = Input::Mpe,
                &|s| s.readouts[0] = ("-100".into(), true),
                &|s| s.knobs[3] = 0.8,
                &|s| s.input = Input::Channel,
                &|s| {
                    s.input = Input::None;
                    s.readouts[3] = ("2.5".into(), true);
                },
                // (The velocity's row: decisions.md R-VELOCITY.)
                &|s| s.last = "97".into(),
                &|s| s.modulates = ("CUTOFF +25".into(), true),
                &|s| {
                    s.response = Response {
                        curve: -0.4,
                        floor: 0.2,
                        point: Some(0.7),
                    }
                },
                &|s| {
                    s.knobs[4] = 0.3;
                    s.readouts[4] = ("-40".into(), true);
                },
                &|s| {
                    s.knobs[5] = 0.2;
                    s.readouts[5] = ("20".into(), true);
                    s.response.point = None;
                },
            ];
            for step in steps {
                step(&mut scene);
                assert!(r.render(&scene, &mut panel));
                assert!(r.damage().is_some());
                let mut whole = MpeRenderer::new(scale, skin, lettering);
                whole.render(&scene, &mut panel);
                assert!(
                    r.frame() == whole.frame(),
                    "{scale} {skin:?} {lettering:?}: {scene:?} drawn in parts"
                );
            }
            assert!(!r.render(&scene, &mut panel), "nothing changed");
        }
    }
}
