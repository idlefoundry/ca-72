//! The strip under the panel, A6 (decisions.md R44): the presets' rail in walnut across its
//! top (the favourite's star, the previous preset, the name's display, the next, SAVE), then
//! the left hand's controls (GLIDE, DECAY and the PITCH and MOD. wheels, drawn with the panel's
//! controls: `art`), VOICES, STEREO and OUTPUT in three rows: banks of lit tabs, then knobs
//! with their readouts (VOICES over ENTROPY; STEREO's WIDTH, INNER and DETUNE side by side;
//! DRIVE over LEVEL; the knobs are the panel's controls too: `controls::CONTROLS`), and the
//! display of where the voices sound. Every place here is in the drawing's units, the strip
//! from `art::PANEL_H` down.
//!
//! What the drawing does not draw is drawn here, over it, by [`StripRenderer`]: the tabs and
//! their light, the readouts, the voices' display, the rail's keys and the preset's name.

pub mod arp;
pub mod fx;
pub mod lfo;
pub mod lower;
pub mod macros;
pub mod mpe;
mod worn;

use resvg::tiny_skia::Pixmap;

use crate::art::{self, PANEL_H, W};
use crate::presets::{BarScene, BarTarget};
use crate::svg::{N, Svg, colour, put};

pub use worn::StripRenderer;
pub(crate) use worn::{
    BLUE, KEY_PRINT, Lamp, RED, amber_caps, pixel_screen, screen_still, screen_text, tinted_caps,
};

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
    (370.0, 976.0, "VOICES"),
    (976.0, 2370.0, "STEREO"),
    (2370.0, W - 28.0, "OUTPUT"),
];
/// Where the left hand's controls are: the old column's origin moved here (its controls in
/// their places relative to it, GLIDE on the first row).
pub const LEFT_HAND: (f64, f64) = (art::LH_X + 20.0, PANEL_H + 78.0);
/// A tab's size, and a bank's pitch between its tabs.
pub const TAB: (f64, f64) = (150.0, 60.0);
pub const TAB_PITCH: f64 = TAB.0 + 6.0;
/// A bank's recess beyond its tabs' pitches, across.
pub const RECESS_PAD: f64 = 18.0;
/// A readout's size.
pub const READOUT: (f64, f64) = (156.0, 84.0);
/// The seven-segment cells' pitch and visible height, shared by every readout (R-DRAWER-LETTERING).
pub const DIGIT_PITCH: f64 = 44.0;
pub const DIGIT_HEIGHT: f64 = READOUT.1 * 0.64;

/// More cells make a wider window, never narrower digits.
pub const fn readout_width(cells: usize) -> f64 {
    cells as f64 * DIGIT_PITCH + 22.0
}
/// The grid (the mock-up's A5 and A6): every amount a unit of its knob and its readout, the
/// knob this far left of the unit's middle, the readout this far right, the unit this wide.
pub const UNIT_KNOB: f64 = -84.0;
pub const UNIT_READOUT: f64 = 106.0;
pub const UNIT_HALF: f64 = 184.0;
/// Each section's units' middles: VOICES's, STEREO's three (WIDTH's, INNER's and DETUNE's,
/// INNER's in the section's middle, the others this far either side of it), OUTPUT's.
pub const CV: f64 = (SECTIONS[1].0 + SECTIONS[1].1) / 2.0;
const STEREO_PITCH: f64 = 430.0;
pub const CI: f64 = (SECTIONS[2].0 + SECTIONS[2].1) / 2.0;
pub const CW: f64 = CI - STEREO_PITCH;
pub const CD: f64 = CI + STEREO_PITCH;
/// The gap between STEREO's two banks, the pair centred over the section.
const BANKS_GAP: f64 = 180.0;
pub const CO: f64 = (SECTIONS[3].0 + SECTIONS[3].1) / 2.0;
/// The keys on the rail: their middles across and their widths. The settings' key at the right
/// end (the owner, 2026-10-10: "a settings button on the right hand side of the preview preset
/// row"), the name's display a key shorter for it, the gaps between the keys as they were; and
/// ARP after the keyboard's, the name a key shorter again for it (the owner's C1, 2026-10-18;
/// decisions.md R-ARP); and MPE after ARP, the name a key shorter again (the owner's A,
/// 2026-10-20; R-MPE); and LFO directly after the keyboard's, the name a key shorter again (the
/// owner, 2026-10-20; R-LFO); and MACRO at the far left, over the left area it flips, the star and
/// the previous key a key right, the name a key shorter again (the owner, 2026-10-20: "in the
/// same bar as the presets ... the far left edge"; R-MACROS); and GLOBAL before KEYS,
/// the name a key shorter again (the owner, 2026-10-22; R-GLOBAL-KEY). GLOBAL is 104 units
/// wide for its lettering at 20, taking 18 more from the name; the gaps stay unchanged. The
/// previous key beside the next, after the name (players asked for the two together;
/// R-PRESET-ARROWS): the name a key left, as wide, its clearances to the star and to the
/// arrows as they were, the arrows a key's gap apart.
const KEYS: [(f64, f64, BarTarget); 12] = [
    (92.0, 86.0, BarTarget::Macro),
    (192.0, 86.0, BarTarget::Star),
    (2008.0, 86.0, BarTarget::Prev),
    (2108.0, 86.0, BarTarget::Next),
    (2252.0, 170.0, BarTarget::Save),
    (2394.0, 86.0, BarTarget::Panel),
    (2503.0, 104.0, BarTarget::Global),
    (2612.0, 86.0, BarTarget::Keyboard),
    (2712.0, 86.0, BarTarget::Lfo),
    (2812.0, 86.0, BarTarget::Arp),
    (2912.0, 86.0, BarTarget::Mpe),
    (3012.0, 86.0, BarTarget::Settings),
];
/// The standalone application's (the owner's FX-A, 2026-10-18; decisions.md R-FX-DRAWER): FX
/// after ARP, the name a key shorter again for it, the gaps between the keys as they were; MPE
/// after FX, in the plug-ins' place beside the settings' key (R-MPE); LFO after the keyboard's
/// (R-LFO); MACRO at the far left (R-MACROS); GLOBAL before KEYS, widened to 104 units
/// from the name's display as above, with the same gaps (R-GLOBAL-KEY); the previous key beside
/// the next, as above (R-PRESET-ARROWS).
const KEYS_FX: [(f64, f64, BarTarget); 13] = [
    (92.0, 86.0, BarTarget::Macro),
    (192.0, 86.0, BarTarget::Star),
    (1908.0, 86.0, BarTarget::Prev),
    (2008.0, 86.0, BarTarget::Next),
    (2152.0, 170.0, BarTarget::Save),
    (2294.0, 86.0, BarTarget::Panel),
    (2403.0, 104.0, BarTarget::Global),
    (2512.0, 86.0, BarTarget::Keyboard),
    (2612.0, 86.0, BarTarget::Lfo),
    (2712.0, 86.0, BarTarget::Arp),
    (2812.0, 86.0, BarTarget::Fx),
    (2912.0, 86.0, BarTarget::Mpe),
    (3012.0, 86.0, BarTarget::Settings),
];
pub const KEY_H: f64 = 62.0;
/// The settings' key's width (and the star's and the arrows').
pub const KEY_W: f64 = 86.0;
/// The name's display: its window (left, top, width, height), as tall with its surround as
/// the keys beside it (the owner, 2026-10-09: "Needs to be the same height as the buttons
/// flanking it").
pub const NAME: (f64, f64, f64, f64) = (262.0, RAIL_Y - 25.0, 1682.0, 50.0);
const NAME_FX: (f64, f64, f64, f64) = (262.0, RAIL_Y - 25.0, 1582.0, 50.0);

/// Key `t`'s light on the rail (decisions.md R-LFO-PAGE's rule): 0 unlit, 1 a faint glow (its
/// page or drawer closed but affecting the sound), 2 lit.
pub fn key_light_of(bar: &BarScene, t: BarTarget) -> u8 {
    match worn::key_light(bar, t) {
        worn::KeyLight::Off => 0,
        worn::KeyLight::Glow => 1,
        worn::KeyLight::Lit => 2,
    }
}

/// The rail's keys: the plug-ins', or the standalone application's with its effects' key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rail {
    #[default]
    Plugin,
    /// FX after ARP (decisions.md R-FX-DRAWER).
    Effects,
}

impl Rail {
    /// Its keys: each key's middle, width and target.
    pub fn keys(self) -> &'static [(f64, f64, BarTarget)] {
        match self {
            Rail::Plugin => &KEYS,
            Rail::Effects => &KEYS_FX,
        }
    }

    /// The name's display: its window (left, top, width, height).
    pub fn name(self) -> (f64, f64, f64, f64) {
        match self {
            Rail::Plugin => NAME,
            Rail::Effects => NAME_FX,
        }
    }
}
pub(crate) const NAME_SURROUND: f64 = 6.0;
/// The voices' display, across STEREO's last row from WIDTH's unit's left to DETUNE's right.
pub const DISPLAY: (f64, f64, f64, f64) = (
    CW - UNIT_HALF,
    ROWS[2] - 62.0,
    CD - CW + 2.0 * UNIT_HALF,
    124.0,
);
/// The section titles' size (the legends' and the small print's are the lettering's:
/// `art::Lettering`).
pub(super) const TITLE: f64 = crate::arrangement::TITLE_SIZE;

/// The banks of tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bank {
    /// MONO | POLY | UNISON.
    Mode,
    /// SCATTER | DOUBLE: how the voices are played across the field.
    Stereo,
    /// EVEN | CENTER: where SCATTER puts them (no EDGES: INNER moves them out from the centre;
    /// decisions.md R49).
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
            Bank::Placement => &["EVEN", "CENTER"],
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
            // Each bank's middle: the pair (BANKS_GAP between their recesses, SCATTER |
            // DOUBLE's on the left) centred on the section's middle.
            Bank::Stereo => CI - (BANKS_GAP + Bank::Placement.width()) / 2.0,
            Bank::Placement => CI + (BANKS_GAP + Bank::Stereo.width()) / 2.0,
            Bank::Auto => CO + UNIT_KNOB,
        }
    }

    /// Its recess's width: its tabs' pitches and the pad.
    fn width(self) -> f64 {
        self.words().len() as f64 * TAB_PITCH + RECESS_PAD
    }

    /// Tab `i`'s middle.
    pub fn tab(self, i: usize) -> (f64, f64) {
        let n = self.words().len() as f64;
        (self.x() + (i as f64 - (n - 1.0) / 2.0) * TAB_PITCH, ROWS[0])
    }

    /// The recess the bank stands in: left, top, width, height.
    pub fn recess(self) -> (f64, f64, f64, f64) {
        let w = self.width();
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
    /// INNER's, its switch (lit where it moves the voices: WIDTH up). Last, so that the others
    /// keep their places in [`Readout::ALL`].
    Inner,
}

impl Readout {
    pub const ALL: [Readout; 8] = [
        Readout::Voices,
        Readout::Entropy,
        Readout::Width,
        Readout::Detune,
        Readout::Auto,
        Readout::Drive,
        Readout::Level,
        Readout::Inner,
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
            Readout::Inner => r(CI, 1),
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
        matches!(self, Readout::Entropy | Readout::Width | Readout::Inner)
    }

    /// Whether a value may be typed into it (all but AUTO GAIN's, which reads what is
    /// measured; decisions.md R52).
    pub fn typed(self) -> bool {
        self != Readout::Auto
    }
}

/// The knobs' places on the strip (their controls are the panel's: `controls::CONTROLS`):
/// VOICES over ENTROPY, WIDTH, INNER and DETUNE side by side over the display, DRIVE over
/// LEVEL. INNER's last, so that the others keep their indices.
pub const KNOBS: [(&str, f64, f64); 7] = [
    ("voices", CV + UNIT_KNOB, ROWS[1]),
    ("entropy", CV + UNIT_KNOB, ROWS[2]),
    ("spread", CW + UNIT_KNOB, ROWS[1]),
    ("double", CD + UNIT_KNOB, ROWS[1]),
    ("drive", CO + UNIT_KNOB, ROWS[1]),
    ("level", CO + UNIT_KNOB, ROWS[2]),
    ("inner", CI + UNIT_KNOB, ROWS[1]),
];

/// What a pointer finds on the strip (but its knobs and the left hand's controls, which are
/// the panel's controls).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripTarget {
    /// A bank's tab.
    Tab(Bank, usize),
    /// A readout that is a switch.
    Switch(Readout),
    /// A readout that is not a switch but takes a value typed into it (VOICES, DETUNE, DRIVE,
    /// LEVEL: decisions.md R52; AUTO GAIN's reads what is measured and is none).
    Readout(Readout),
    /// The voices' display.
    Display,
    /// The rail's keys and the name.
    Bar(BarTarget),
}

/// The strip's controls that MIDI Learn rings here (its knobs are rung as the panel's are):
/// POLY (MONO's and POLY's tabs), UNISON, DOUBLE (SCATTER | DOUBLE), the placement, AUTO GAIN.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripControl {
    Poly,
    Unison,
    Double,
    Placement,
    AutoGain,
}

impl StripTarget {
    /// The control a press here operates, if it is one MIDI Learn rings here.
    pub fn control(self) -> Option<StripControl> {
        match self {
            StripTarget::Tab(Bank::Mode, 2) => Some(StripControl::Unison),
            StripTarget::Tab(Bank::Mode, _) => Some(StripControl::Poly),
            StripTarget::Tab(Bank::Stereo, _) => Some(StripControl::Double),
            StripTarget::Tab(Bank::Placement, _) => Some(StripControl::Placement),
            StripTarget::Tab(Bank::Auto, _) => Some(StripControl::AutoGain),
            _ => None,
        }
    }
}

/// What a pointer at (`x`, `y`) (the drawing's units) finds on the strip, its rail `rail`.
pub fn hit(rail: Rail, x: f64, y: f64) -> Option<StripTarget> {
    if y < PANEL_H {
        return None;
    }
    if (y - RAIL_Y).abs() <= KEY_H / 2.0 + 6.0 {
        for &(kx, w, t) in rail.keys() {
            if (x - kx).abs() <= w / 2.0 + 4.0 {
                return Some(StripTarget::Bar(t));
            }
        }
        let (nx, _, nw, _) = rail.name();
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
    for r in Readout::ALL
        .into_iter()
        .filter(|r| !r.switch() && r.typed())
    {
        let (rx, ry, rw, rh) = r.window();
        if (rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y) {
            return Some(StripTarget::Readout(r));
        }
    }
    let (dx, dy, dw, dh) = DISPLAY;
    ((dx..dx + dw).contains(&x) && (dy..dy + dh).contains(&y)).then_some(StripTarget::Display)
}

/// A target's middle (the tests', and a menu's place), the rail `rail`.
pub fn centre(rail: Rail, t: StripTarget) -> (f64, f64) {
    match t {
        StripTarget::Tab(b, i) => b.tab(i),
        StripTarget::Switch(r) | StripTarget::Readout(r) => r.at(),
        StripTarget::Display => (DISPLAY.0 + DISPLAY.2 / 2.0, DISPLAY.1 + DISPLAY.3 / 2.0),
        StripTarget::Bar(BarTarget::Name) => {
            let (x, _, w, _) = rail.name();
            (x + w / 2.0, RAIL_Y)
        }
        StripTarget::Bar(b) => (
            rail.keys().iter().find(|k| k.2 == b).map_or(0.0, |k| k.0),
            RAIL_Y,
        ),
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
        StripControl::Double => around(Bank::Stereo.recess()),
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
    /// Each bank's tab lit (none: none, as PLACEMENT's in MONO with DOUBLE).
    pub mode: Option<usize>,
    pub stereo: Option<usize>,
    pub placement: Option<usize>,
    pub auto: bool,
    /// Each readout's text and whether it is lit ([`Readout::ALL`]'s order).
    pub readouts: [(String, bool); 8],
    pub field: Field,
    /// Each voice's level (0 silent to 1 loud, by its index in the field), and DOUBLE's detune
    /// between a note's two voices, cents: the drops swell with their voices, and a pair beats.
    pub levels: Vec<f64>,
    pub detune: f64,
    pub bar: BarScene,
    pub hover: Option<StripTarget>,
    /// The control MIDI Learn waits for, if it is rung here.
    pub learning: Option<StripControl>,
    /// None shows the extension controls; Some replaces them with the keyboard.
    pub keyboard: Option<crate::keyboard::Scene>,
    /// The LFO page shows over the extension controls (decisions.md R-LFO; drawn apart:
    /// `lfo`), so they are neither drawn nor moved under it.
    pub lfo: bool,
}

impl StripScene {
    /// Whether the extension controls show: neither the keyboard nor the LFO page over them.
    pub fn extension(&self) -> bool {
        self.keyboard.is_none() && !self.lfo
    }
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
            levels: Vec::new(),
            detune: 0.0,
            bar: BarScene::default(),
            hover: None,
            learning: None,
            keyboard: None,
            lfo: false,
        }
    }
}

// ---- What the drawing draws for it (`art`): its surfaces and its print.

/// The strip's surfaces that do not move, for either skin: the banks' recesses, the readouts'
/// and the display's bezels. (The name's surround on the rail is the strip's renderer's, as long
/// as its rail's name is: decisions.md R-FX-DRAWER.)
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
}

/// The name's display set into the rail's wood (the owner: "we need some sort of border or edge
/// around that screen to help it feel more natural"): a dark frame standing a little proud of
/// the wood, its shadow soft below it and to the right, its top edge catching the light and its
/// bottom in shade, a black lip down into the glass, its lower inner edge catching the light.
/// Within the surround: as tall as the keys beside it. `dy` down (the worn strip draws in its
/// own frame, from the rail's top).
pub(crate) fn name_bezel(s: &mut Svg, rail: Rail, dy: f64) {
    let (x, y, w, h) = rail.name();
    screen_bezel(s, (x, y + dy, w, h));
}

/// The preset display's bezel, sized for another window in the same strip.
pub(crate) fn screen_bezel(s: &mut Svg, (x, y, w, h): (f64, f64, f64, f64)) {
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

/// A bank's legend, as far above its recess as GLOBAL's MODE (R-DRAWER-LETTERING).
pub(super) fn bank_legend(s: &mut Svg, x: f64, recess_top: f64, word: &str) {
    art::text(
        s,
        x,
        recess_top - 26.0,
        word,
        s.1.strip_legend(),
        art::Style::Legend,
        false,
        "",
    );
}

/// The strip's print: the sections' rules and titles, the banks' legends, the knobs' legends,
/// the readouts' units, the display's title and its ends. (The knobs' dials are drawn with the
/// panel's: `art`.)
pub(crate) fn print(s: &mut Svg) {
    let top = PANEL_H + RAIL;
    let foot = art::H;
    let (legend, small) = (s.1.strip_legend(), s.1.strip_small());
    let text = |s: &mut Svg, x: f64, y: f64, t: &str, size: f64| {
        art::text(s, x, y, t, size, art::Style::Legend, false, "");
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
            put!(s, "<g id='strip-title-{t}'>");
            art::text(
                s,
                (x0 + x1) / 2.0,
                foot - 50.0,
                t,
                TITLE,
                art::Style::Title,
                false,
                "",
            );
            s.0.push_str("</g>");
        }
    }
    for b in Bank::ALL {
        let (_, y, _, _) = b.recess();
        bank_legend(s, b.x(), y, b.title());
    }
    let knob_legends = [
        "VOICES", "ENTROPY", "WIDTH", "DETUNE", "DRIVE", "LEVEL", "INNER",
    ];
    for ((_, x, y), t) in KNOBS.iter().zip(knob_legends) {
        text(s, *x, y - 109.0, t, legend);
    }
    for r in Readout::ALL {
        if let Some(u) = r.unit() {
            let (x, y, w, _) = r.window();
            text(s, x + w + 26.0, y + READOUT.1 / 2.0, u, small);
        }
    }
    let (dx, dy, dw, dh) = DISPLAY;
    text(s, dx - 22.0, dy + dh / 2.0, "L", small);
    text(s, dx + dw + 22.0, dy + dh / 2.0, "R", small);
    text(
        s,
        dx + dw / 2.0,
        dy - 32.0,
        "WHERE THE VOICES SOUND",
        legend,
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Each knob's unit (the knob, its readout, `UNIT_HALF` either side of its middle) stands in
    /// its section, clear of the others on its row; STEREO's three side by side, INNER's in the
    /// middle; STEREO's banks centred over the section as a pair, `BANKS_GAP` apart; the display
    /// under WIDTH's unit to DETUNE's.
    #[test]
    fn the_strips_units_stand_in_their_sections_clear_of_each_other() {
        let section = |x: f64| {
            SECTIONS
                .iter()
                .position(|(x0, x1, _)| (*x0..*x1).contains(&x))
                .expect("in a section")
        };
        let units: Vec<(f64, f64)> = KNOBS.iter().map(|(_, x, y)| (x - UNIT_KNOB, *y)).collect();
        for (i, (c, y)) in units.iter().enumerate() {
            let s = section(*c);
            let (x0, x1, _) = SECTIONS[s];
            assert!(c - UNIT_HALF > x0 && c + UNIT_HALF < x1, "{}", KNOBS[i].0);
            for (d, _) in units[i + 1..].iter().filter(|(_, z)| z == y) {
                assert!((c - d).abs() >= 2.0 * UNIT_HALF, "{} overlaps", KNOBS[i].0);
            }
        }
        for r in Readout::ALL {
            let (x, y) = r.at();
            let (c, row) = (x - UNIT_READOUT, ROWS.iter().position(|z| *z == y));
            let knob = units.contains(&(c, y));
            assert!(knob || r == Readout::Auto, "{r:?} beside its knob");
            assert!(row.is_some(), "{r:?} on a row");
        }
        assert_eq!((CW + CD) / 2.0, CI);
        assert_eq!(section(CI), 2);
        let (s, p) = (Bank::Stereo.recess(), Bank::Placement.recess());
        assert!((p.0 - (s.0 + s.2) - BANKS_GAP).abs() < 1e-9);
        assert!(
            (s.0 + p.0 + p.2 - 2.0 * CI).abs() < 1e-9,
            "centred as a pair"
        );
        assert!(s.0 > SECTIONS[2].0 && p.0 + p.2 < SECTIONS[2].1);
        let (dx, _, dw, _) = DISPLAY;
        assert_eq!((dx, dx + dw), (CW - UNIT_HALF, CD + UNIT_HALF));
    }

    /// PLACEMENT's bank is EVEN and CENTER (no EDGES: decisions.md R49), its recess two
    /// tabs wide: each tab found where it stands, the placement's control (MIDI Learn rings the
    /// recess), and no tab past CENTER or before EVEN; every bank's tabs found so.
    #[test]
    fn placements_two_tabs_are_found_where_they_stand() {
        let b = Bank::Placement;
        assert_eq!(b.words(), ["EVEN", "CENTER"]);
        let (x, y, w, h) = b.recess();
        assert_eq!(w, 2.0 * TAB_PITCH + RECESS_PAD);
        for i in 0..2 {
            let t = StripTarget::Tab(b, i);
            let (tx, ty) = b.tab(i);
            assert_eq!(hit(Rail::Plugin, tx, ty), Some(t));
            assert_eq!(t.control(), Some(StripControl::Placement));
            assert!(
                tx - TAB.0 / 2.0 > x && tx + TAB.0 / 2.0 < x + w,
                "tab {i} in the recess"
            );
        }
        assert_eq!(span(StripControl::Placement), (x, y, x + w, y + h));
        for beyond in [b.tab(0).0 - TAB_PITCH, b.tab(1).0 + TAB_PITCH] {
            assert_eq!(
                hit(Rail::Plugin, beyond, ROWS[0]),
                None,
                "a tab at {beyond}"
            );
        }
        for b in Bank::ALL {
            for i in 0..b.words().len() {
                let (tx, ty) = b.tab(i);
                assert_eq!(
                    hit(Rail::Plugin, tx, ty),
                    Some(StripTarget::Tab(b, i)),
                    "{b:?} {i}"
                );
            }
        }
    }
}
