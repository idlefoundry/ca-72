//! The strip under the panel, A6 (decisions.md R-LOOK): the presets' rail in walnut across its
//! top (the favourite's star, the previous preset, the name's display, the next, SAVE), then
//! the left hand's controls (GLIDE, DECAY and the PITCH and MOD. wheels, drawn with the panel's
//! controls: `art`), VOICES, STEREO and OUTPUT in three rows: banks of lit tabs, then knobs
//! with their readouts (the knobs are the panel's controls too: `controls::CONTROLS`), and the
//! display of where the voices sound. Every place here is in the drawing's units, the strip
//! from `art::PANEL_H` down.
//!
//! What the drawing does not draw is drawn here, over it, by [`StripRenderer`]: the tabs and
//! their light, the readouts, the voices' display, the rail's keys and the preset's name.

mod worn;

use resvg::tiny_skia::Pixmap;

use crate::art::{self, PANEL_H, W};
use crate::presets::{BarScene, BarTarget};
use crate::svg::{N, Svg, colour, put};

pub use worn::StripRenderer;

// ---- The layout.

/// The walnut rail's height, its foot's edge included.
pub const RAIL: f64 = 116.0;
/// The rail's keys' and the name's display's middle.
pub const RAIL_Y: f64 = PANEL_H + 55.0;
/// The three rows' middles: the tabs, then two of knobs.
pub const ROWS: [f64; 3] = [PANEL_H + 228.0, PANEL_H + 452.0, PANEL_H + 676.0];
/// The sections, across: the left hand's, VOICES, STEREO and OUTPUT (to the trim).
pub const SECTIONS: [(f64, f64, &str); 4] = [
    (0.0, 370.0, ""),
    (370.0, 1120.0, "VOICES"),
    (1120.0, 2370.0, "STEREO"),
    (2370.0, W - 28.0, "OUTPUT"),
];
/// Where the left hand's controls are: the old column's origin moved here (its controls in
/// their places relative to it, GLIDE on the first row).
pub const LEFT_HAND: (f64, f64) = (art::LH_X + 20.0, PANEL_H + 78.0);
/// A tab's size, and a bank's pitch between its tabs.
pub const TAB: (f64, f64) = (150.0, 60.0);
const TAB_PITCH: f64 = TAB.0 + 6.0;
/// A readout's size.
pub const READOUT: (f64, f64) = (156.0, 84.0);
/// The grid (the mock-up's A5 and A6): every amount a unit of its knob and its readout, the
/// knob this far left of the unit's middle, the readout this far right, the unit this wide.
pub const UNIT_KNOB: f64 = -84.0;
const UNIT_READOUT: f64 = 106.0;
const UNIT_HALF: f64 = 184.0;
/// Each section's units' middles: VOICES's, STEREO's two (WIDTH's and DETUNE's), OUTPUT's.
pub const CV: f64 = (SECTIONS[1].0 + SECTIONS[1].1) / 2.0;
pub const CW: f64 = SECTIONS[2].0 + 330.0;
pub const CD: f64 = SECTIONS[2].0 + 920.0;
pub const CO: f64 = (SECTIONS[3].0 + SECTIONS[3].1) / 2.0;
/// The keys on the rail: their middles across and their widths.
const KEYS: [(f64, f64, BarTarget); 4] = [
    (92.0, 86.0, BarTarget::Star),
    (192.0, 86.0, BarTarget::Prev),
    (2826.0, 86.0, BarTarget::Next),
    (2970.0, 170.0, BarTarget::Save),
];
const KEY_H: f64 = 62.0;
/// The name's display: its window (left, top, width, height), as tall with its surround as
/// the keys beside it (the owner, 2026-10-09: "Needs to be the same height as the buttons
/// flanking it").
pub const NAME: (f64, f64, f64, f64) = (262.0, RAIL_Y - 25.0, 2500.0, 50.0);
const NAME_SURROUND: f64 = 6.0;
/// The voices' display, across STEREO's last row from WIDTH's unit's left to DETUNE's right.
pub const DISPLAY: (f64, f64, f64, f64) = (
    CW - UNIT_HALF,
    ROWS[2] - 62.0,
    CD - CW + 2.0 * UNIT_HALF,
    124.0,
);
/// The font sizes: section titles, banks' and knobs' legends, small print.
const TITLE: f64 = 52.0;
const LEGEND: f64 = 21.0;
const SMALL: f64 = 16.0;

/// The banks of tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bank {
    /// MONO | POLY | UNISON.
    Mode,
    /// SCATTER | DOUBLE: how the voices are played across the field.
    Stereo,
    /// EVEN | EDGES | CENTER: where SCATTER puts them.
    Placement,
    /// AUTO GAIN's ON.
    Auto,
}

impl Bank {
    pub const ALL: [Bank; 4] = [Bank::Mode, Bank::Stereo, Bank::Placement, Bank::Auto];

    /// Its tabs' words.
    pub fn words(self) -> &'static [&'static str] {
        match self {
            Bank::Mode => &["MONO", "POLY", "UNISON"],
            Bank::Stereo => &["SCATTER", "DOUBLE"],
            Bank::Placement => &["EVEN", "EDGES", "CENTER"],
            Bank::Auto => &["ON"],
        }
    }

    /// Its title over it.
    pub fn title(self) -> &'static str {
        match self {
            Bank::Mode => "MODE",
            Bank::Stereo => "VOICES PLAYED AS",
            Bank::Placement => "PLACEMENT",
            Bank::Auto => "AUTO GAIN",
        }
    }

    /// Its middle across (on the first row).
    pub fn x(self) -> f64 {
        match self {
            Bank::Mode => CV,
            Bank::Stereo => CW,
            Bank::Placement => CD,
            Bank::Auto => CO + UNIT_KNOB,
        }
    }

    /// Tab `i`'s middle.
    pub fn tab(self, i: usize) -> (f64, f64) {
        let n = self.words().len() as f64;
        (self.x() + (i as f64 - (n - 1.0) / 2.0) * TAB_PITCH, ROWS[0])
    }

    /// The recess the bank stands in: left, top, width, height.
    pub fn recess(self) -> (f64, f64, f64, f64) {
        let w = self.words().len() as f64 * TAB_PITCH + 18.0;
        let h = TAB.1 + 22.0;
        (self.x() - w / 2.0, ROWS[0] - h / 2.0, w, h)
    }
}

/// The readouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readout {
    Voices,
    /// ENTROPY's, its switch (OFF when off).
    Entropy,
    /// WIDTH's, its switch.
    Width,
    /// DETUNE's: OFF with SCATTER.
    Detune,
    /// What AUTO GAIN takes back now, dB.
    Auto,
    Drive,
    Level,
}

impl Readout {
    pub const ALL: [Readout; 7] = [
        Readout::Voices,
        Readout::Entropy,
        Readout::Width,
        Readout::Detune,
        Readout::Auto,
        Readout::Drive,
        Readout::Level,
    ];

    /// Its middle.
    pub fn at(self) -> (f64, f64) {
        let r = |c: f64, row: usize| (c + UNIT_READOUT, ROWS[row]);
        match self {
            Readout::Voices => r(CV, 1),
            Readout::Entropy => r(CV, 2),
            Readout::Width => r(CW, 1),
            Readout::Detune => r(CD, 1),
            Readout::Auto => r(CO, 0),
            Readout::Drive => r(CO, 1),
            Readout::Level => r(CO, 2),
        }
    }

    /// Its window: left, top, width, height.
    pub fn window(self) -> (f64, f64, f64, f64) {
        let (x, y) = self.at();
        (
            x - READOUT.0 / 2.0,
            y - READOUT.1 / 2.0,
            READOUT.0,
            READOUT.1,
        )
    }

    /// Its unit printed beside it.
    fn unit(self) -> Option<&'static str> {
        matches!(self, Readout::Auto | Readout::Drive | Readout::Level).then_some("dB")
    }

    /// Whether it is a switch (its amount off and back on at it).
    pub fn switch(self) -> bool {
        matches!(self, Readout::Entropy | Readout::Width)
    }
}

/// The knobs' places on the strip (their controls are the panel's: `controls::CONTROLS`):
/// VOICES over ENTROPY, WIDTH and DETUNE side by side over the display, DRIVE over LEVEL.
pub const KNOBS: [(&str, f64, f64); 6] = [
    ("voices", CV + UNIT_KNOB, ROWS[1]),
    ("entropy", CV + UNIT_KNOB, ROWS[2]),
    ("spread", CW + UNIT_KNOB, ROWS[1]),
    ("double", CD + UNIT_KNOB, ROWS[1]),
    ("drive", CO + UNIT_KNOB, ROWS[1]),
    ("level", CO + UNIT_KNOB, ROWS[2]),
];

/// What a pointer finds on the strip (but its knobs and the left hand's controls, which are
/// the panel's controls).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripTarget {
    /// A bank's tab.
    Tab(Bank, usize),
    /// A readout that is a switch.
    Switch(Readout),
    /// The voices' display.
    Display,
    /// The rail's keys and the name.
    Bar(BarTarget),
}

/// The strip's controls that MIDI Learn rings here (its knobs are rung as the panel's are):
/// POLY (MONO's and POLY's tabs), UNISON, the placement, AUTO GAIN.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripControl {
    Poly,
    Unison,
    Placement,
    AutoGain,
}

impl StripTarget {
    /// The control a press here operates, if it is one MIDI Learn rings here.
    pub fn control(self) -> Option<StripControl> {
        match self {
            StripTarget::Tab(Bank::Mode, 2) => Some(StripControl::Unison),
            StripTarget::Tab(Bank::Mode, _) => Some(StripControl::Poly),
            StripTarget::Tab(Bank::Placement, _) => Some(StripControl::Placement),
            StripTarget::Tab(Bank::Auto, _) => Some(StripControl::AutoGain),
            _ => None,
        }
    }
}

/// What a pointer at (`x`, `y`) (the drawing's units) finds on the strip.
pub fn hit(x: f64, y: f64) -> Option<StripTarget> {
    if y < PANEL_H {
        return None;
    }
    if (y - RAIL_Y).abs() <= KEY_H / 2.0 + 6.0 {
        for (kx, w, t) in KEYS {
            if (x - kx).abs() <= w / 2.0 + 4.0 {
                return Some(StripTarget::Bar(t));
            }
        }
        let (nx, _, nw, _) = NAME;
        if (nx..nx + nw).contains(&x) {
            return Some(StripTarget::Bar(BarTarget::Name));
        }
        return None;
    }
    for b in Bank::ALL {
        for i in 0..b.words().len() {
            let (tx, ty) = b.tab(i);
            if (x - tx).abs() <= TAB.0 / 2.0 && (y - ty).abs() <= TAB.1 / 2.0 + 4.0 {
                return Some(StripTarget::Tab(b, i));
            }
        }
    }
    for r in Readout::ALL.into_iter().filter(|r| r.switch()) {
        let (rx, ry, rw, rh) = r.window();
        if (rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y) {
            return Some(StripTarget::Switch(r));
        }
    }
    let (dx, dy, dw, dh) = DISPLAY;
    ((dx..dx + dw).contains(&x) && (dy..dy + dh).contains(&y)).then_some(StripTarget::Display)
}

/// A target's middle (the tests', and a menu's place).
pub fn centre(t: StripTarget) -> (f64, f64) {
    match t {
        StripTarget::Tab(b, i) => b.tab(i),
        StripTarget::Switch(r) => r.at(),
        StripTarget::Display => (DISPLAY.0 + DISPLAY.2 / 2.0, DISPLAY.1 + DISPLAY.3 / 2.0),
        StripTarget::Bar(BarTarget::Name) => (NAME.0 + NAME.2 / 2.0, RAIL_Y),
        StripTarget::Bar(b) => (KEYS.iter().find(|k| k.2 == b).map_or(0.0, |k| k.0), RAIL_Y),
    }
}

/// Where a control MIDI Learn rings here is: left, top, right, bottom (its tab or bank).
pub fn span(c: StripControl) -> (f64, f64, f64, f64) {
    let around = |(x, y, w, h): (f64, f64, f64, f64)| (x, y, x + w, y + h);
    match c {
        StripControl::Poly => {
            let (x0, y0) = Bank::Mode.tab(0);
            let (x1, _) = Bank::Mode.tab(1);
            (
                x0 - TAB.0 / 2.0 - 6.0,
                y0 - TAB.1 / 2.0 - 6.0,
                x1 + TAB.0 / 2.0 + 6.0,
                y0 + TAB.1 / 2.0 + 6.0,
            )
        }
        StripControl::Unison => {
            let (x, y) = Bank::Mode.tab(2);
            (
                x - TAB.0 / 2.0 - 6.0,
                y - TAB.1 / 2.0 - 6.0,
                x + TAB.0 / 2.0 + 6.0,
                y + TAB.1 / 2.0 + 6.0,
            )
        }
        StripControl::Placement => around(Bank::Placement.recess()),
        StripControl::AutoGain => around(Bank::Auto.recess()),
    }
}

// ---- What it shows.

/// The voices' display's field: each voice's place (-1 left to 1 right) and whether it sounds;
/// with DOUBLE each note's pair (how far out, 0 to 1, its twin as far the other way).
#[derive(Clone, Debug, PartialEq)]
pub enum Field {
    Scatter(Vec<(f64, bool)>),
    Double(Vec<(f64, bool)>),
}

impl Default for Field {
    fn default() -> Self {
        Field::Scatter(vec![(0.0, false)])
    }
}

/// What the strip shows.
#[derive(Clone, Debug, PartialEq)]
pub struct StripScene {
    /// Each bank's tab lit (none: none, as PLACEMENT's in MONO without DOUBLE).
    pub mode: Option<usize>,
    pub stereo: Option<usize>,
    pub placement: Option<usize>,
    pub auto: bool,
    /// Each readout's text and whether it is lit ([`Readout::ALL`]'s order).
    pub readouts: [(String, bool); 7],
    pub field: Field,
    pub bar: BarScene,
    pub hover: Option<StripTarget>,
    /// The control MIDI Learn waits for, if it is rung here.
    pub learning: Option<StripControl>,
}

impl Default for StripScene {
    fn default() -> Self {
        StripScene {
            mode: Some(0),
            stereo: Some(0),
            placement: Some(0),
            auto: true,
            readouts: std::array::from_fn(|_| (String::new(), false)),
            field: Field::default(),
            bar: BarScene::default(),
            hover: None,
            learning: None,
        }
    }
}

// ---- What the drawing draws for it (`art`): its surfaces and its print.

/// The strip's surfaces that do not move, for either skin: the banks' recesses, the readouts'
/// and the display's bezels, the name's surround on the rail.
pub(crate) fn surfaces(s: &mut Svg) {
    let bezel = |s: &mut Svg, (x, y, w, h): (f64, f64, f64, f64), r: f64| {
        put!(
            s,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='#0b0a09'/>",
            N(x - 6.0),
            N(y - 6.0),
            N(w + 12.0),
            N(h + 12.0),
            N(r + 4.0)
        );
        put!(
            s,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='none' stroke='#fff' stroke-opacity='0.07' stroke-width='1.5'/>",
            N(x - 5.0),
            N(y - 5.0),
            N(w + 10.0),
            N(h + 10.0),
            N(r + 3.5)
        );
    };
    for b in Bank::ALL {
        let (x, y, w, h) = b.recess();
        put!(
            s,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='6' fill='#070605'/>",
            N(x),
            N(y),
            N(w),
            N(h)
        );
    }
    for r in Readout::ALL {
        bezel(s, r.window(), 5.0);
    }
    bezel(s, DISPLAY, 5.0);
    name_bezel(s, 0.0);
}

/// The name's display set into the rail's wood (the owner: "we need some sort of border or edge
/// around that screen to help it feel more natural"): a dark frame standing a little proud of
/// the wood, its shadow soft below it and to the right, its top edge catching the light and its
/// bottom in shade, a black lip down into the glass, its lower inner edge catching the light.
/// Within the surround: as tall as the keys beside it. `dy` down (the worn strip draws in its
/// own frame, from the rail's top).
pub(crate) fn name_bezel(s: &mut Svg, dy: f64) {
    let (x, y, w, h) = NAME;
    let y = y + dy;
    let o = NAME_SURROUND;
    let (fx, fy, fw, fh) = (x - o, y - o, w + 2.0 * o, h + 2.0 * o);
    put!(
        s,
        "<defs><filter id='name-shadow' x='-0.05' y='-0.5' width='1.1' height='2'><feGaussianBlur stdDeviation='2.5'/></filter><linearGradient id='name-frame' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#2c2a27'/><stop offset='0.5' stop-color='#1a1917'/><stop offset='1' stop-color='#0e0d0c'/></linearGradient><linearGradient id='name-edge' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#fff' stop-opacity='0.28'/><stop offset='0.45' stop-color='#fff' stop-opacity='0.04'/><stop offset='1' stop-color='#000' stop-opacity='0.6'/></linearGradient><linearGradient id='name-lip' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0'/><stop offset='0.8' stop-color='#000' stop-opacity='0'/><stop offset='1' stop-color='#fff' stop-opacity='0.16'/></linearGradient></defs>"
    );
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='6' fill='#000' fill-opacity='0.55' filter='url(#name-shadow)'/>",
        N(fx + 1.5),
        N(fy + 3.0),
        N(fw),
        N(fh)
    );
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='6' fill='url(#name-frame)'/><rect x='{}' y='{}' width='{}' height='{}' rx='5.5' fill='none' stroke='url(#name-edge)' stroke-width='1.2'/>",
        N(fx),
        N(fy),
        N(fw),
        N(fh),
        N(fx + 0.6),
        N(fy + 0.6),
        N(fw - 1.2),
        N(fh - 1.2)
    );
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='4.5' fill='#050504'/><rect x='{}' y='{}' width='{}' height='{}' rx='4.5' fill='none' stroke='url(#name-lip)' stroke-width='1'/>",
        N(x - 2.0),
        N(y - 2.0),
        N(w + 4.0),
        N(h + 4.0),
        N(x - 1.5),
        N(y - 1.5),
        N(w + 3.0),
        N(h + 3.0)
    );
}

/// The strip's print: the sections' rules and titles, the banks' titles, the knobs' legends,
/// the readouts' units, the display's title and its ends. (The knobs' dials are drawn with the
/// panel's: `art`.)
pub(crate) fn print(s: &mut Svg, fonts_family: &str) {
    let top = PANEL_H + RAIL;
    let foot = art::H;
    let text = |s: &mut Svg, x: f64, y: f64, t: &str, size: f64| {
        put!(
            s,
            "<text x='{}' y='{}' font-family='{fonts_family}' font-size='{}' font-weight='700' text-anchor='middle' dominant-baseline='central' fill='{}'>{}</text>",
            N(x),
            N(y),
            N(size),
            colour::LEGEND,
            t
        );
    };
    for (x0, _, _) in &SECTIONS[1..] {
        put!(
            s,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{}' stroke-width='4' stroke-linecap='round'/>",
            N(*x0),
            N(top + 16.0),
            N(*x0),
            N(foot - 14.0),
            colour::LEGEND
        );
    }
    for (x0, x1, t) in SECTIONS {
        if !t.is_empty() {
            text(s, (x0 + x1) / 2.0, foot - 50.0, t, TITLE);
        }
    }
    for b in Bank::ALL {
        let (_, y, _, _) = b.recess();
        text(s, b.x(), y - 26.0, b.title(), LEGEND);
    }
    let knob_legends = ["VOICES", "ENTROPY", "WIDTH", "DETUNE", "DRIVE", "LEVEL"];
    for ((_, x, y), t) in KNOBS.iter().zip(knob_legends) {
        text(s, *x, y - 109.0, t, LEGEND);
    }
    for r in Readout::ALL {
        if let Some(u) = r.unit() {
            let (x, y, w, _) = r.window();
            text(s, x + w + 26.0, y + READOUT.1 / 2.0, u, SMALL);
        }
    }
    let (dx, dy, dw, dh) = DISPLAY;
    text(s, dx - 22.0, dy + dh / 2.0, "L", SMALL);
    text(s, dx + dw + 22.0, dy + dh / 2.0, "R", SMALL);
    text(
        s,
        dx + dw / 2.0,
        dy - 32.0,
        "WHERE THE VOICES SOUND",
        LEGEND,
    );
}

/// The strip's keys and tabs, as the tests and the renderer find them: each key's middle,
/// width and target.
pub(crate) fn keys() -> impl Iterator<Item = (f64, f64, BarTarget)> {
    KEYS.into_iter()
}

pub(crate) const KEY_HEIGHT: f64 = KEY_H;

/// An empty frame the strip's size at `scale`.
pub(crate) fn blank(scale: f64) -> Option<Pixmap> {
    let (w, h) = strip_size(scale);
    Pixmap::new(w, h)
}

/// The strip's frame's size at `scale` (pixels): the drawing's width, the strip's height.
pub fn strip_size(scale: f64) -> (u32, u32) {
    (
        (W * scale).round().max(1.0) as u32,
        (art::STRIP_H * scale).round().max(1.0) as u32,
    )
}
