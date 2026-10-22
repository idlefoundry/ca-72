//! The arpeggiator's drawer (decisions.md R-ARP; the owner's C1, 2026-10-18): one row dropped
//! down from under the rail, inside the window, over the top of whichever lower page shows (the
//! strip's controls or the keyboard), put away again by the rail's ARP key. Left to right: the
//! ARP bank (ON | HOLD), the DIRECTION bank (UP | DOWN | UP/DOWN | RANDOM | ORDER), a rule, then
//! RATE, OCTAVES, GATE and SWING, each a knob and its readout, the strip's own parts (its lit
//! tabs, narrower than the strip's, its knobs, its seven-segment readouts, its legends).
//!
//! The drawer is a plate of its own, as the effects' drawer is (`lower`: the face, its edges, the
//! shadow it casts, its lit tabs). What does not change is drawn once a scale on a thread of its
//! own, begun as the editor opens; then only what changed.

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use plugin_kit_materials::{Lighting, glass, sprite};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;

use super::lower::{self, LitTabs, Look};
pub use super::lower::{LEFT, RIGHT, SHADOW, TOP, top_px};
use super::worn::{Lamp, Reading, copy, meets, readout_patch, svg_over, union, within};
use super::{READOUT, RECESS_PAD, UNIT_READOUT};
use crate::art::{self, Lettering};
use crate::render::Renderer;
use crate::skin::Skin;
use crate::svg::{N, Svg, colour, put};

// ---- The layout (the drawing's units, the drawer open).

/// How deep it is, from its top under the rail (`lower::TOP`; the owner, of the slimmer drawer:
/// "make it even slimmer"; 265 units, the mock-up's).
pub const DEPTH: f64 = 265.0;
/// Its row's middle, and how far above it the legends and the banks' titles are.
pub const ROW: f64 = TOP + 142.0;
const LEGEND_UP: f64 = 109.0;
/// The drawer's tabs, narrower than the strip's (`super::TAB`) so that DIRECTION's five fit
/// (the owner's ORDER, 2026-10-19: the mock-up's 140 units), and their pitch.
pub const TAB: (f64, f64) = (140.0, super::TAB.1);
pub const TAB_PITCH: f64 = TAB.0 + 6.0;
/// The rule between the banks and the knobs, across.
const DIVIDER: f64 = 1512.0;
/// The units' middles: RATE's, OCTAVES', GATE's and SWING's (each a knob `UNIT_KNOB` left of it
/// and its readout `UNIT_READOUT` right of it, as the strip's), closer together for ORDER.
pub const UNITS: [f64; 4] = [1716.0, 2099.0, 2482.0, 2865.0];
/// The knobs' parameters, in the units' order.
pub const KNOB_PARAMS: [&str; 4] = ["arp_rate", "arp_octaves", "arp_gate", "arp_swing"];
const LEGENDS: [&str; 4] = ["RATE", "OCTAVES", "GATE", "SWING"];

/// The drawer's two banks of tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArpBank {
    /// ON | HOLD: each a switch of its own.
    Arp,
    /// UP | DOWN | UP/DOWN | RANDOM | ORDER: one of them.
    Direction,
}

impl ArpBank {
    pub const ALL: [ArpBank; 2] = [ArpBank::Arp, ArpBank::Direction];

    pub fn words(self) -> &'static [&'static str] {
        match self {
            ArpBank::Arp => &["ON", "HOLD"],
            ArpBank::Direction => &["UP", "DOWN", "UP/DOWN", "RANDOM", "ORDER"],
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            ArpBank::Arp => "ARP",
            ArpBank::Direction => "DIRECTION",
        }
    }

    /// Its middle across.
    pub fn x(self) -> f64 {
        match self {
            ArpBank::Arp => 555.0,
            ArpBank::Direction => 1114.0,
        }
    }

    fn width(self) -> f64 {
        self.words().len() as f64 * TAB_PITCH + RECESS_PAD
    }

    /// Tab `i`'s middle.
    pub fn tab(self, i: usize) -> (f64, f64) {
        let n = self.words().len() as f64;
        (self.x() + (i as f64 - (n - 1.0) / 2.0) * TAB_PITCH, ROW)
    }

    /// The recess it stands in: left, top, width, height.
    pub fn recess(self) -> (f64, f64, f64, f64) {
        let (w, h) = (self.width(), TAB.1 + 22.0);
        (self.x() - w / 2.0, ROW - h / 2.0, w, h)
    }
}

/// Readout `i`'s window (RATE, OCTAVES, GATE, SWING): left, top, width, height.
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
/// `interact::hit_arp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArpTarget {
    Tab(ArpBank, usize),
    /// A readout, which takes a value typed into it.
    Readout(usize),
}

/// Whether (`x`, `y`) is on the drawer, open.
pub fn inside(x: f64, y: f64) -> bool {
    (LEFT..RIGHT).contains(&x) && (TOP..TOP + DEPTH).contains(&y)
}

/// What a pointer at (`x`, `y`) finds on the drawer, open.
pub fn hit(x: f64, y: f64) -> Option<ArpTarget> {
    for b in ArpBank::ALL {
        for i in 0..b.words().len() {
            let (tx, ty) = b.tab(i);
            if (x - tx).abs() <= TAB.0 / 2.0 && (y - ty).abs() <= TAB.1 / 2.0 + 4.0 {
                return Some(ArpTarget::Tab(b, i));
            }
        }
    }
    (0..4).find_map(|i| {
        let (rx, ry, rw, rh) = readout(i);
        ((rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y)).then_some(ArpTarget::Readout(i))
    })
}

/// A target's middle.
pub fn centre(t: ArpTarget) -> (f64, f64) {
    match t {
        ArpTarget::Tab(b, i) => b.tab(i),
        ArpTarget::Readout(i) => {
            let (x, y, w, h) = readout(i);
            (x + w / 2.0, y + h / 2.0)
        }
    }
}

/// What the drawer shows.
#[derive(Clone, Debug, PartialEq)]
pub struct ArpScene {
    pub on: bool,
    pub latch: bool,
    /// DIRECTION's tab lit.
    pub direction: usize,
    /// Each readout's text, and whether it is lit (RATE, OCTAVES, GATE, SWING).
    pub readouts: [(String, bool); 4],
    /// Each knob's value, normalized.
    pub knobs: [f64; 4],
    pub hover: Option<ArpTarget>,
}

impl Default for ArpScene {
    fn default() -> Self {
        ArpScene {
            on: false,
            latch: false,
            direction: 0,
            readouts: std::array::from_fn(|_| (String::new(), true)),
            knobs: [0.0; 4],
            hover: None,
        }
    }
}

// ---- Drawing.

/// What does not change at a scale: the plate, and over it the print, the bezels, the glass
/// and the unlit tabs; the tabs' words apart (laid over the lit ones); the shadow it casts.
struct Still {
    look: Look,
    plate: Pixmap,
    words: Vec<(i32, i32, Pixmap)>,
    shadow: Pixmap,
    lighting: Lighting,
}

/// The drawer's renderer at the panel's scale.
pub struct ArpRenderer {
    scale: f64,
    skin: Skin,
    lettering: Lettering,
    options: usvg::Options<'static>,
    warming: Option<JoinHandle<Still>>,
    still: Option<Arc<Still>>,
    frame: Pixmap,
    lit: LitTabs,
    readouts: [Option<Reading>; 4],
    /// Each knob as drawn: its value, and its pixels at their place in the frame.
    knobs: [Option<(f64, i32, i32, Pixmap)>; 4],
    shown: Option<ArpScene>,
    damage: Option<[i32; 4]>,
}

impl fmt::Debug for ArpRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArpRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl ArpRenderer {
    /// At `scale`, in `skin` and `lettering`: what does not change made on a thread of its own,
    /// begun now (as the editor opens, so that the drawer's first drop waits for none of it).
    pub fn new(scale: f64, skin: Skin, lettering: Lettering) -> Self {
        let mut r = ArpRenderer {
            scale,
            skin,
            lettering,
            // (What it draws as it goes, the readouts' segments and the pointer's tab, has no
            // lettering: no fonts loaded for it as the editor opens.)
            options: usvg::Options::default(),
            warming: None,
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            lit: LitTabs::new(TAB),
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
            .name("ca72-arp".into())
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
        self.lit.clear();
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
    pub fn render(&mut self, scene: &ArpScene, panel: &mut Renderer) -> bool {
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
            if let Some((_, x, y, p)) = &self.knobs[k] {
                dirty = Some(union(
                    dirty,
                    [*x, *y, x + p.width() as i32, y + p.height() as i32],
                ));
            }
            let top = top_px(self.scale);
            self.knobs[k] = crate::controls::index(param)
                .and_then(|i| panel.paint_control(i, v))
                .map(|(x, y, p)| (v, x, y - top, p));
            if let Some((_, x, y, p)) = &self.knobs[k] {
                dirty = Some(union(
                    dirty,
                    [*x, *y, x + p.width() as i32, y + p.height() as i32],
                ));
            }
        }
        match &was {
            None => dirty = Some(all),
            Some(was) => {
                for r in self.changes(was, scene) {
                    dirty = Some(union(dirty, r));
                }
            }
        }
        self.shown = Some(scene.clone());
        let Some(r) = dirty.and_then(|r| within(r, all)) else {
            return false;
        };
        let r = within(self.closed(scene, r), all).unwrap_or(r);
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

    /// Where a tab draws, lit: its cap and its light round it.
    fn tab_part(&self, at: (f64, f64)) -> [i32; 4] {
        let m = 40.0;
        self.pixels((
            at.0 - TAB.0 / 2.0 - m,
            at.1 - TAB.1 / 2.0 - m,
            at.0 + TAB.0 / 2.0 + m,
            at.1 + TAB.1 / 2.0 + m,
        ))
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

    /// Where `now` shows something `was` did not (the knobs apart).
    fn changes(&self, was: &ArpScene, now: &ArpScene) -> Vec<[i32; 4]> {
        let mut out = Vec::new();
        let bank = |b: ArpBank| (0..b.words().len()).map(move |i| b.tab(i));
        if (was.on, was.latch) != (now.on, now.latch) {
            out.extend(bank(ArpBank::Arp).map(|at| self.tab_part(at)));
        }
        if was.direction != now.direction {
            out.extend(bank(ArpBank::Direction).map(|at| self.tab_part(at)));
        }
        for i in 0..4 {
            if was.readouts[i] != now.readouts[i] {
                out.push(self.readout_part(i));
            }
        }
        if was.hover != now.hover {
            for t in [was.hover, now.hover].into_iter().flatten() {
                if let ArpTarget::Tab(..) = t {
                    out.push(self.tab_part(centre(t)));
                }
            }
        }
        out
    }

    /// The tabs lit: ON and HOLD while they are on, DIRECTION's choice.
    fn lit(scene: &ArpScene) -> Vec<(f64, f64)> {
        let mut lit = Vec::new();
        if scene.on {
            lit.push(ArpBank::Arp.tab(0));
        }
        if scene.latch {
            lit.push(ArpBank::Arp.tab(1));
        }
        let last = ArpBank::Direction.words().len() - 1;
        lit.push(ArpBank::Direction.tab(scene.direction.min(last)));
        lit
    }

    /// `r` grown until every part that reaches into it lies inside it.
    fn closed(&self, scene: &ArpScene, mut r: [i32; 4]) -> [i32; 4] {
        let mut parts: Vec<[i32; 4]> = Self::lit(scene)
            .into_iter()
            .map(|at| self.tab_part(at))
            .collect();
        parts.extend((0..4).map(|i| self.readout_part(i)));
        parts.extend((0..4).filter_map(|k| self.knob_part(k)));
        if let Some(still) = &self.still {
            parts.extend(
                still
                    .words
                    .iter()
                    .map(|(x, y, p)| [*x, *y, x + p.width() as i32, y + p.height() as i32]),
            );
        }
        if let Some(ArpTarget::Tab(b, i)) = scene.hover {
            parts.push(self.tab_part(b.tab(i)));
        }
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
    fn paint(&mut self, scene: &ArpScene, r: [i32; 4]) {
        let Some(still) = self.still.clone() else {
            return;
        };
        let s = self.scale;
        let oy = f64::from(top_px(s)) / s;
        copy(&mut self.frame, &still.plate, r);
        // The lit tabs: their light on the face round them, then the lit cap over the unlit.
        for at in Self::lit(scene) {
            if meets(self.tab_part(at), r) {
                self.lit.light(&mut self.frame, &still.lighting, s, at, oy);
            }
        }
        // The tabs' words over their caps; the pointer's tab lightened.
        for (x, y, p) in &still.words {
            let part = [*x, *y, x + p.width() as i32, y + p.height() as i32];
            if meets(part, r) {
                crate::render::put(&mut self.frame, p, (*x, *y), 1.0);
            }
        }
        if let Some(ArpTarget::Tab(b, i)) = scene.hover {
            let (x, y) = b.tab(i);
            if meets(self.tab_part((x, y)), r) {
                let mut o = Svg::default();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#fff' fill-opacity='0.07'/>",
                    N(x - TAB.0 / 2.0 + 2.0),
                    N(y - oy - TAB.1 / 2.0 + 2.0),
                    N(TAB.0 - 4.0),
                    N(TAB.1 - 4.0)
                );
                svg_over(&self.options, &mut self.frame, s, &o.0);
            }
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
                self.readouts[i] = readout_patch(
                    &self.options,
                    s,
                    (x, y - oy, w, h),
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

/// What never changes for `look`: the plate, the print worn into it, the recesses, bezels and
/// glass, the knobs' shadows and the unlit tabs; the tabs' words; the shadow below.
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
    let lighting = lower::tab_lighting(TAB);
    let mut tabs = Vec::new();
    for b in ArpBank::ALL {
        for (i, word) in b.words().iter().enumerate() {
            let at = b.tab(i);
            sprite(&mut plate, &lighting.unlit, s, (at.0, at.1 - oy), TAB);
            tabs.push((at, *word));
        }
    }
    lower::lamp(&mut plate, s, skin, (0, top_px(s)));
    let words = lower::tab_words(&options, s, oy, TAB, &tabs);
    let shadow = lower::cast(s, &options);
    Still {
        look: (scale, skin, lettering),
        plate,
        words,
        shadow,
        lighting,
    }
}

/// The plate's surfaces (the frame's units, `oy` its top in the drawing's; its edges are the
/// plate's, `lower::plate`): the banks' recesses and the readouts' bezels, as the strip's; the
/// knobs' shadows (the worn skin's, as the panel's).
fn surfaces(oy: f64, skin: Skin) -> String {
    let mut s = Svg::default();
    let y = |v: f64| N(v - oy);
    put!(
        s,
        "<defs><filter id='long' x='-0.6' y='-0.6' width='2.2' height='2.2'><feGaussianBlur stdDeviation='7'/></filter><filter id='softer' x='-0.5' y='-0.5' width='2' height='2'><feGaussianBlur stdDeviation='2'/></filter></defs>"
    );
    for b in ArpBank::ALL {
        let (x, ry, w, h) = b.recess();
        put!(
            s,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='6' fill='#070605'/>",
            N(x),
            y(ry),
            N(w),
            N(h)
        );
    }
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

/// The plate's print (the frame's units): the rule between the banks and the knobs, the banks'
/// titles, the knobs' legends and dials, the readouts' units.
fn print_svg(oy: f64, lettering: Lettering) -> String {
    let mut s = Svg(String::new(), lettering);
    let (legend, small) = (lettering.strip_legend(), lettering.strip_small());
    let text = |s: &mut Svg, x: f64, y: f64, t: &str, size: f64| {
        art::text(s, x, y - oy, t, size, art::Style::Legend, false, "");
    };
    put!(
        s,
        "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{}' stroke-width='4' stroke-linecap='round'/>",
        N(DIVIDER),
        N(TOP + 16.0 - oy),
        N(DIVIDER),
        N(TOP + DEPTH - 14.0 - oy),
        colour::LEGEND
    );
    for b in ArpBank::ALL {
        art::text(
            &mut s,
            b.x(),
            TOP + 38.0 - oy,
            b.title(),
            super::TITLE,
            art::Style::Title,
            false,
            "",
        );
    }
    for (k, (u, t)) in UNITS.iter().zip(LEGENDS).enumerate() {
        let x = u + super::UNIT_KNOB;
        text(&mut s, x, ROW - LEGEND_UP, t, legend);
        let dial = match k {
            0 => crate::controls::Dial::ArpRate,
            1 => crate::controls::Dial::ArpOctaves,
            _ => crate::controls::Dial::Percent,
        };
        art::draw_dial(&mut s, x, ROW - oy, dial);
        if k >= 2 {
            let (rx, ry, rw, rh) = readout(k);
            // (Close to its readout: SWING's, so moved up for ORDER, within the trim.)
            text(&mut s, rx + rw + 18.0, ry + rh / 2.0, "%", small);
        }
    }
    s.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each tab and readout is found where it is drawn, within the drawer; nothing past its
    /// ends; the units clear of each other and of the rule, the banks before it.
    #[test]
    fn the_drawer_finds_each_part_where_it_is_drawn() {
        for b in ArpBank::ALL {
            for i in 0..b.words().len() {
                let (x, y) = b.tab(i);
                assert_eq!(hit(x, y), Some(ArpTarget::Tab(b, i)));
                assert!(inside(x, y));
            }
            let (x, _, w, _) = b.recess();
            assert!(x > LEFT && x + w < DIVIDER, "{b:?}");
        }
        for i in 0..4 {
            let at = centre(ArpTarget::Readout(i));
            assert_eq!(hit(at.0, at.1), Some(ArpTarget::Readout(i)));
        }
        let half = super::super::UNIT_HALF;
        for w in UNITS.windows(2) {
            assert!(w[1] - w[0] >= 2.0 * half);
        }
        assert!(UNITS[0] - half > DIVIDER);
        assert!(UNITS[3] + half < RIGHT);
        // SWING's % (some 16 units wide) inside the trim.
        let (rx, _, rw, _) = readout(3);
        assert!(rx + rw + 18.0 + 10.0 < RIGHT);
        assert!(!inside(LEFT - 1.0, ROW) && !inside(RIGHT + 1.0, ROW));
        assert!(!inside(1000.0, TOP + DEPTH + 1.0) && !inside(1000.0, TOP - 1.0));
        // The row and its legends inside the drawer, under the rail's shadow.
        assert!(ROW - LEGEND_UP > TOP + 14.0 && ROW + READOUT.1 / 2.0 < TOP + DEPTH - 9.0);
    }

    /// The drawer drawn in parts is the drawer drawn whole: tabs lit and let go, the pointer
    /// over a tab, a readout changed and a knob turned, at the window's opening scale and a
    /// Retina screen's, in both skins.
    #[test]
    fn drawn_in_parts_it_is_drawn_whole() {
        for (scale, skin) in [(0.44, Skin::Worn), (0.87, Skin::Drawn)] {
            let mut panel = Renderer::with_skin(skin, scale, 1.0);
            let mut r = ArpRenderer::new(scale, skin, Lettering::Standard);
            let mut scene = ArpScene {
                readouts: [
                    ("16".into(), true),
                    ("2".into(), true),
                    ("50".into(), true),
                    ("0".into(), true),
                ],
                knobs: [0.75, 1.0 / 3.0, 0.5, 0.0],
                ..ArpScene::default()
            };
            assert!(r.render(&scene, &mut panel));
            let steps: [&dyn Fn(&mut ArpScene); 6] = [
                &|s| s.on = true,
                &|s| s.direction = 4,
                &|s| s.hover = Some(ArpTarget::Tab(ArpBank::Arp, 1)),
                &|s| s.readouts[0] = ("16t".into(), true),
                &|s| s.knobs[2] = 0.8,
                &|s| {
                    s.latch = true;
                    s.hover = None;
                },
            ];
            for step in steps {
                step(&mut scene);
                assert!(r.render(&scene, &mut panel));
                assert!(r.damage().is_some());
                let mut whole = ArpRenderer::new(scale, skin, Lettering::Standard);
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
