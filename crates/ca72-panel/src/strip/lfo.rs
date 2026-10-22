//! The LFO page (decisions.md R-LFO; the owner's layout A of round 2, 2026-10-20): a lower page
//! of its own where the keyboard's is, opened by the rail's LFO key, the left hand's controls
//! kept beside it. Four columns, one an LFO, each top to bottom:
//!
//! - its scope (a CRT in the LFO's phosphor colour: graticule, the glowing trace of one cycle
//!   of its waveform from PHASE, and an orb along it where the LFO is in its cycle, at its
//!   output: under the trace while RISE fades it in, stopped at the cycle's end in ENV), MODE (the panel's three-position toggle: FREE, RETRIG, ENV) and MONO beside it;
//! - WAVEFORM (the oscillators' switch, STEPPED or CONTINUOUS), RATE (Hz, or its note values
//!   while its timed key is on), PHASE;
//! - RISE, the OUT jack its assignments are dragged from, and the MODULATES screen.
//!
//! Every control is the panel's own at the panel's size (`controls::CONTROLS`, `Place::Lfo`),
//! drawn as the panel's renderer draws controls. What does not change is drawn once a scale on a
//! thread of its own, begun as the editor opens; a scope's still screen (its face and trace) is
//! drawn again only when its trace or colour changes, by code rather than SVG filters
//! (`crate::glow`); each frame the LFO moves, only its orb's patch is laid again over it.

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use resvg::tiny_skia::{
    Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform,
};
use resvg::usvg;

use super::lower::{self, Look};
pub use super::lower::{LEFT, RIGHT, top_px};
use super::worn::{Lamp, screen_still, screen_text, svg_over};
use crate::arrangement::Arrangement;
use crate::art::{self, H, Keyed, Layout, Lettering, ModWay, W};
use crate::controls::{CONTROLS, Dial, Kind, lfo_index};
use crate::fonts::Fonts;
use crate::glow::{self, Coverage, Phosphor, crop, hex, lay};
use crate::render::Renderer;
use crate::skin::Skin;
use crate::svg::{N, Svg, colour, put};

// ---- The layout (the drawing's units; the round-2 mock-ups' space budget).

/// The page's top, under the rail, and how deep it is (to the strip's trim).
pub const TOP: f64 = lower::TOP;
pub const DEPTH: f64 = H - 6.0 - TOP;
/// The columns: from the left hand's rule to the strip's trim, four alike.
pub const X0: f64 = 370.0;
pub const X1: f64 = W - 28.0;
pub const COL_W: f64 = (X1 - X0) / 4.0;
const TITLE: f64 = super::TITLE;
const TITLE_Y: f64 = H - 50.0;
const LEGEND_UP: f64 = 109.0;
const PITCH: f64 = 212.0;
/// The rows: WAVEFORM, RATE and PHASE; RISE, the jack and the screen.
pub const ROW1: f64 = 1365.0;
pub const ROW2: f64 = 1612.0;
const TIMED_Y: f64 = ROW1 + 118.0;
/// The scope's bezel (about the column's middle): left offset, top, width, height; its glass
/// inset within it.
const SCOPE: (f64, f64, f64, f64) = (-325.0, 1060.0, 440.0, 176.0);
const GLASS_IN: f64 = 17.0;
const GLASS_RX: f64 = 22.0;
/// MODE's toggle and MONO's key (about the column's middle).
const MODE_AT: (f64, f64) = (228.0, 1152.0);
const MONO_AT: (f64, f64) = (176.0, 1214.0);
/// The ON key and POLARITY's UNI key (R-LFO's corrections), in the foot's band beside the
/// title (round 2's budget: the title 644–774 of a column 370–1047.5, about its middle ±65;
/// RISE's numbers down to 1704, the screen's surround to 1643, the foot 1769). ON's cap just
/// left of the title, as far from it as a key's word is from its cap, so that the title is its
/// word (the title is pressed as the key is); UNI's to the title's right, under the screen.
const TITLE_HALF: f64 = 65.0;
const ON_AT: (f64, f64) = (-TITLE_HALF - art::KEY_SIGN, TITLE_Y);
const UNI_AT: (f64, f64) = (128.0, TITLE_Y);
/// The jack and the screen (about the column's middle; the screen ten characters).
const JACK_AT: (f64, f64) = (-66.0, ROW2);
pub const JACK_R: f64 = 20.0;
const SCREEN: (f64, f64, f64, f64) = (-20.0, ROW2 - 25.0, 332.0, 50.0);
/// The jack's place among a column's drawn parts, after its controls ([`LFO_PARTS`]' order).
const JACK: usize = crate::controls::LFO_PARTS.len();
/// A muted LFO's trace's light (of 255).
const MUTED_TRACE: u16 = 90;
/// The samples a scope's trace is drawn through, across its one cycle.
pub const TRACE_POINTS: usize = 480;
/// The trace's margin from the glass's ends and its height's from top and foot.
const TRACE_PAD: f64 = 12.0;
const TRACE_TOP: f64 = 14.0;
/// The orb's sprite's half size, its glow included.
const ORB_R: f64 = 34.0;

/// Column `n`'s middle across.
pub const fn centre(n: usize) -> f64 {
    X0 + COL_W * (n as f64 + 0.5)
}

/// Each part's place in a column: the scope's bezel (left, top, width, height), the controls'
/// centres, the screen's window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Places {
    pub scope: (f64, f64, f64, f64),
    pub mode: (f64, f64),
    pub mono: (f64, f64),
    pub polarity: (f64, f64),
    pub on: (f64, f64),
    pub wave: (f64, f64),
    pub rate: (f64, f64),
    pub phase: (f64, f64),
    pub timed: (f64, f64),
    pub rise: (f64, f64),
    pub jack: (f64, f64),
    pub screen: (f64, f64, f64, f64),
}

/// Column `n`'s places.
pub const fn places(n: usize) -> Places {
    let c = centre(n);
    Places {
        scope: (c + SCOPE.0, SCOPE.1, SCOPE.2, SCOPE.3),
        mode: (c + MODE_AT.0, MODE_AT.1),
        mono: (c + MONO_AT.0, MONO_AT.1),
        polarity: (c + UNI_AT.0, UNI_AT.1),
        on: (c + ON_AT.0, ON_AT.1),
        wave: (c - PITCH, ROW1),
        rate: (c, ROW1),
        phase: (c + PITCH, ROW1),
        timed: (c, TIMED_Y),
        rise: (c - PITCH, ROW2),
        jack: (c + JACK_AT.0, JACK_AT.1),
        screen: (c + SCREEN.0, SCREEN.1, SCREEN.2, SCREEN.3),
    }
}

/// Column `n`'s scope's glass (left, top, width, height).
pub fn glass(n: usize) -> (f64, f64, f64, f64) {
    let (x, y, w, h) = places(n).scope;
    (
        x + GLASS_IN,
        y + GLASS_IN,
        w - 2.0 * GLASS_IN,
        h - 2.0 * GLASS_IN,
    )
}

/// What a pointer finds on the page beside its controls (`interact::hit_lfo`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LfoTarget {
    /// An LFO's scope: its colour and EDGES from its right-click menu.
    Scope(usize),
    /// Its OUT jack: an assignment dragged from it.
    Jack(usize),
    /// Its MODULATES screen: a tip listing what it modulates, a click its menu.
    Screen(usize),
    /// Its title, ON's key's word: pressed, ON turned (R-LFO's corrections).
    Title(usize),
    /// Anywhere else in its column.
    Column(usize),
}

impl LfoTarget {
    /// The LFO it is of.
    pub fn lfo(self) -> usize {
        match self {
            LfoTarget::Scope(n)
            | LfoTarget::Jack(n)
            | LfoTarget::Screen(n)
            | LfoTarget::Title(n)
            | LfoTarget::Column(n) => n,
        }
    }
}

/// Whether (`x`, `y`) is on the page.
pub fn inside(x: f64, y: f64) -> bool {
    (LEFT..RIGHT).contains(&x) && (TOP..TOP + DEPTH).contains(&y)
}

/// What a pointer at (`x`, `y`) finds on the page (its controls apart), the page showing.
pub fn hit(x: f64, y: f64) -> Option<LfoTarget> {
    if !inside(x, y) {
        return None;
    }
    let n = (((x - X0) / COL_W).floor().max(0.0) as usize).min(3);
    let p = places(n);
    let within = |(rx, ry, rw, rh): (f64, f64, f64, f64)| {
        (rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y)
    };
    Some(if (x - p.jack.0).hypot(y - p.jack.1) <= JACK_R + 10.0 {
        LfoTarget::Jack(n)
    } else if within(p.scope) {
        LfoTarget::Scope(n)
    } else if within(p.screen) {
        LfoTarget::Screen(n)
    } else if within(title(n)) {
        LfoTarget::Title(n)
    } else {
        LfoTarget::Column(n)
    })
}

/// Column `n`'s title's box (left, top, width, height): the title's print and a little round it.
pub fn title(n: usize) -> (f64, f64, f64, f64) {
    let h = TITLE * 0.8;
    (
        centre(n) - TITLE_HALF - 4.0,
        TITLE_Y - h / 2.0,
        2.0 * TITLE_HALF + 8.0,
        h,
    )
}

/// A target's middle (the tests', and where a menu opens).
pub fn target_centre(t: LfoTarget) -> (f64, f64) {
    let p = places(t.lfo());
    let mid = |(x, y, w, h): (f64, f64, f64, f64)| (x + w / 2.0, y + h / 2.0);
    match t {
        LfoTarget::Scope(_) => mid(p.scope),
        LfoTarget::Jack(_) => p.jack,
        LfoTarget::Screen(_) => mid(p.screen),
        LfoTarget::Title(n) => mid(title(n)),
        LfoTarget::Column(n) => (centre(n), TITLE_Y + 30.0),
    }
}

// ---- What it shows.

/// One LFO's column as the page shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// Its phosphor's glow (sRGB): its trace, orb, and rings.
    pub colour: [u8; 3],
    /// WAVEFORM STEPPED: its switch's value (normalized); CONTINUOUS: where it is along its
    /// travel, steps from TRIANGLE (`art::SHAPE_PLACES` at most).
    pub wave: f64,
    pub continuous: bool,
    pub shape_at: f64,
    /// RATE's knob (its Hz, or its DIVISION while timed), PHASE's, RISE's (normalized), MODE's
    /// toggle (0 FREE, 0.5 RETRIG, 1 ENV).
    pub rate: f64,
    pub phase: f64,
    pub rise: f64,
    pub mode: f64,
    /// Its timed key on (RATE in note values: its dial printed so), its division's place among
    /// the eighteen (the key's sign), MONO.
    pub timed: bool,
    pub division: usize,
    pub mono: bool,
    /// POLARITY UNI (its key lit; the trace then 0..1, above the centre line), and ON (its key
    /// lit; off, its trace dims and it has no orb).
    pub uni: bool,
    pub on: bool,
    /// Its trace drawn from settings a source moves now (decisions.md R-LFO's corrections): drawn
    /// again whenever they move, by the cheaper glow ([`draw_trace_live`]).
    pub live: bool,
    /// The trace: the waveform (-1..1; 0..1 UNI) at [`TRACE_POINTS`] even steps over one cycle
    /// from PHASE.
    pub trace: Vec<f32>,
    /// The orb: how far along the trace (0..1), and the output there (-1..1); none, no orb.
    pub orb: Option<(f64, f64)>,
    /// RISE's gain at the orb (0..1): its wake drawn at so much of the trace's height, as the
    /// orb rides at the output.
    pub orb_rise: f64,
    /// What MODULATES reads, and whether lit (dim: NOT IN USE).
    pub screen: String,
    pub in_use: bool,
}

impl Default for Column {
    fn default() -> Self {
        Column {
            colour: [0xff, 0xb0, 0x20],
            wave: 0.0,
            continuous: false,
            shape_at: 0.0,
            rate: 0.5,
            phase: 0.0,
            rise: 0.0,
            mode: 0.0,
            timed: false,
            division: 8,
            mono: true,
            uni: false,
            on: true,
            trace: vec![0.0; TRACE_POINTS],
            orb: None,
            orb_rise: 1.0,
            screen: "NOT IN USE".into(),
            in_use: false,
            live: false,
        }
    }
}

/// What the page shows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LfoScene {
    pub columns: [Column; 4],
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

/// What does not change at a scale: the plate with its print, bezels, glass and shadows; each
/// column's RATE printed in note values and WAVEFORM with CONTINUOUS's arc, as parts laid over
/// it; the jacks.
struct Still {
    look: Look,
    plate: Pixmap,
    timed: Vec<Option<Patch>>,
    arcs: Vec<Option<Patch>>,
}

/// A phosphor colour's parts at a scale: the scope's face (the unlit phosphor and graticule),
/// what lies over the trace (the tube's vignette and the glass's reflection), and the orb.
struct Tube {
    colour: [u8; 3],
    base: Pixmap,
    over: Pixmap,
    orb: Pixmap,
}

/// A column's scope as drawn: its trace and colour, the face with the trace on it (the orb's
/// patches are cut from it), and that with the glass over it (what is laid).
struct Screen {
    key: (Vec<f32>, [u8; 3], bool, bool),
    lit: Pixmap,
    shown: Pixmap,
}

/// What each element of a column was drawn for, so that only what changed is drawn again.
type Key = Vec<u64>;

/// The page's renderer at the panel's scale.
pub struct LfoRenderer {
    scale: f64,
    skin: Skin,
    lettering: Lettering,
    warming: Option<JoinHandle<Still>>,
    still: Option<Arc<Still>>,
    frame: Pixmap,
    tubes: Vec<Tube>,
    screens: [Option<Screen>; 4],
    /// How many times each column's screen has been drawn (its orb's patch is cut from it).
    generations: [u64; 4],
    orbs: [Option<(Key, Patch)>; 4],
    texts: [Option<(Key, Patch)>; 4],
    /// Each column's controls and jack ([`LFO_PARTS`]' order, then the jack), as drawn.
    controls: [[Option<(Key, Patch)>; JACK + 1]; 4],
    shown: Option<LfoScene>,
    damages: Vec<[i32; 4]>,
}

impl fmt::Debug for LfoRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LfoRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

/// An `f64` as a key's word.
fn word(v: f64) -> u64 {
    v.to_bits()
}

impl LfoRenderer {
    /// At `scale`, in `skin` and `lettering`: what does not change made on a thread of its own,
    /// begun now (as the editor opens, so that the page's first showing waits for none of it).
    pub fn new(scale: f64, skin: Skin, lettering: Lettering) -> Self {
        let mut r = LfoRenderer {
            scale,
            skin,
            lettering,
            warming: None,
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            tubes: Vec::new(),
            screens: Default::default(),
            generations: [0; 4],
            orbs: Default::default(),
            texts: Default::default(),
            controls: Default::default(),
            shown: None,
            damages: Vec::new(),
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
            .name("ca72-lfo".into())
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
        self.tubes.clear();
        self.screens = Default::default();
        self.orbs = Default::default();
        self.texts = Default::default();
        self.controls = Default::default();
        self.shown = None;
        self.warm();
    }

    /// The frame: the page's, from its top (`top_px`), the drawing's width (only `LEFT` to
    /// `RIGHT` is shown).
    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// The places the last render changed in the frame (left, top, right, bottom).
    pub fn damages(&self) -> &[[i32; 4]] {
        &self.damages
    }

    /// Drawn for `scene`, its controls as `panel` draws them: whether the frame changed.
    pub fn render(&mut self, scene: &LfoScene, panel: &mut Renderer) -> bool {
        self.damages.clear();
        if self.shown.as_ref() == Some(scene) && self.still.is_some() {
            return false;
        }
        let mut whole = false;
        if self.still.is_none() {
            let still = self.still_now();
            self.frame = still.plate.clone();
            self.still = Some(still);
            self.shown = None;
            self.screens = Default::default();
            self.orbs = Default::default();
            self.texts = Default::default();
            self.controls = Default::default();
            whole = true;
        }
        let mut dirty: Vec<[i32; 4]> = Vec::new();
        let was = self.shown.take();
        for (n, col) in scene.columns.iter().enumerate() {
            let before = was.as_ref().map(|w| &w.columns[n]);
            if before.is_none_or(|b| b.timed != col.timed) {
                dirty.extend(self.part_rect(n, true));
            }
            if before.is_none_or(|b| b.continuous != col.continuous) {
                dirty.extend(self.part_rect(n, false));
            }
            if self.screen_for(n, col) {
                dirty.push(self.glass_px(n));
            }
            dirty.extend(self.orb_for(n, col));
            dirty.extend(self.text_for(n, col));
            for k in 0..=JACK {
                dirty.extend(self.control_for(n, k, col, panel));
            }
        }
        self.shown = Some(scene.clone());
        let all = [0, 0, self.frame.width() as i32, self.frame.height() as i32];
        let places = if whole {
            vec![all]
        } else {
            merged(dirty.into_iter().filter_map(|r| within(r, all)).collect())
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

    /// The still: the thread's, made as the editor opened (waited for if it is not done), if
    /// it is still the look; else made now.
    fn still_now(&mut self) -> Arc<Still> {
        if let Some(Ok(still)) = self.warming.take().map(JoinHandle::join)
            && still.look == self.look()
        {
            return Arc::new(still);
        }
        Arc::new(make_still(self.look()))
    }

    /// The frame's pixels of a rectangle of the drawing's units (left, top, right, bottom),
    /// every pixel it touches and one more round it.
    fn pixels(&self, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> [i32; 4] {
        pixels_of(self.scale, (x0, y0, x1, y1))
    }

    /// Column `n`'s scope's glass in the frame's pixels.
    fn glass_px(&self, n: usize) -> [i32; 4] {
        glass_px(self.scale, n)
    }

    /// Where column `n`'s RATE (`rate`) or WAVEFORM part is laid.
    fn part_rect(&self, n: usize, rate: bool) -> Option<[i32; 4]> {
        let still = self.still.as_ref()?;
        let parts = if rate { &still.timed } else { &still.arcs };
        parts.get(n)?.as_ref().map(Patch::rect)
    }

    /// Column `n`'s scope's still screen drawn again if its trace or colour changed: whether it
    /// was.
    fn screen_for(&mut self, n: usize, col: &Column) -> bool {
        let key = (col.trace.clone(), col.colour, col.on, col.live);
        if self.screens[n].as_ref().is_some_and(|s| s.key == key) {
            return false;
        }
        let s = self.scale;
        let g = self.glass_px(n);
        let (w, h) = ((g[2] - g[0]) as u32, (g[3] - g[1]) as u32);
        let tube = self.tube(col.colour, (w, h));
        let mut lit = tube.base.clone();
        let mut trace = Pixmap::new(w, h).expect("a scope's size");
        // (A trace a source moves, drawn again as it moves: the glow's layers stroked straight,
        // without the still trace's blurs, R-LFO's corrections.)
        if col.live {
            draw_trace_live(&mut trace, &col.trace, s, col.colour, glass(n));
        } else {
            draw_trace(&mut trace, &col.trace, s, col.colour, glass(n));
        }
        // (Inside the glass's rounded corners only; muted, at a third.)
        for (d, b) in trace
            .data_mut()
            .chunks_exact_mut(4)
            .zip(tube.base.data().chunks_exact(4))
        {
            if b[3] < 255 {
                for c in d.iter_mut() {
                    *c = (u16::from(*c) * u16::from(b[3]) / 255) as u8;
                }
            }
            if !col.on {
                for c in d.iter_mut() {
                    *c = (u16::from(*c) * MUTED_TRACE / 255) as u8;
                }
            }
        }
        lay(&mut lit, &trace, (0, 0), [0, 0, w as i32, h as i32]);
        let mut shown = lit.clone();
        lay(&mut shown, &tube.over, (0, 0), [0, 0, w as i32, h as i32]);
        self.screens[n] = Some(Screen { key, lit, shown });
        self.generations[n] += 1;
        true
    }

    /// The tube for `colour` at this scale, `size` pixels, made the first time it is wanted.
    fn tube(&mut self, colour: [u8; 3], size: (u32, u32)) -> &Tube {
        if let Some(i) = self
            .tubes
            .iter()
            .position(|t| t.colour == colour && (t.base.width(), t.base.height()) == size)
        {
            return &self.tubes[i];
        }
        let t = make_tube(self.scale, colour, size, glass(0));
        self.tubes.push(t);
        self.tubes.last().expect("just made")
    }

    /// Column `n`'s orb's patch drawn again if it moved: where it was and is.
    fn orb_for(&mut self, n: usize, col: &Column) -> Vec<[i32; 4]> {
        // (Its screen drawn again: its patch cut from the new one.)
        let key: Key = match col.orb {
            Some((x, v)) => vec![1, word(x), word(v), word(col.orb_rise), self.generations[n]],
            None => vec![0],
        };
        if self.orbs[n].as_ref().is_some_and(|(k, _)| *k == key) {
            return Vec::new();
        }
        let mut out: Vec<[i32; 4]> = self.orbs[n]
            .take()
            .map(|(_, p)| p.rect())
            .into_iter()
            .collect();
        if let Some((x, v)) = col.orb
            && let Some(p) = self.orb_patch(n, col, x, v)
        {
            out.push(p.rect());
            self.orbs[n] = Some((key, p));
        }
        out
    }

    /// Column `n`'s orb at `x` along its trace (0..1), the output `v` there: its patch cut from
    /// the still screen, its wake along the trace behind it, the orb, the glass over them.
    fn orb_patch(&mut self, n: usize, col: &Column, x: f64, v: f64) -> Option<Patch> {
        let s = self.scale;
        let g = self.glass_px(n);
        let (gw, gh) = ((g[2] - g[0]) as u32, (g[3] - g[1]) as u32);
        let colour = col.colour;
        let (orb, over, base) = {
            let t = self.tube(colour, (gw, gh));
            (t.orb.clone(), t.over.clone(), t.base.clone())
        };
        let screen = self.screens[n].as_ref()?;
        let (_, _, uw, uh) = glass(n);
        let k = col.trace.len().max(2);
        let at = |i: usize, value: f64| trace_point(i, k, value, (uw, uh), (gw, gh));
        let i = ((x.clamp(0.0, 1.0) * (k - 1) as f64).round() as usize).min(k - 1);
        let (ox, oy) = at(i, v);
        let i0 = i.saturating_sub(k / 9);
        let wake: Vec<(f32, f32)> = (i0..=i)
            .map(|j| {
                at(
                    j,
                    f64::from(col.trace[j.min(col.trace.len() - 1)]) * col.orb_rise,
                )
            })
            .collect();
        let r = (ORB_R * s) as f32;
        let mut b = [ox - r, oy - r, ox + r, oy + r];
        for &(wx, wy) in &wake {
            b = [
                b[0].min(wx - 4.0),
                b[1].min(wy - 4.0),
                b[2].max(wx + 4.0),
                b[3].max(wy + 4.0),
            ];
        }
        let rect = [
            (b[0].floor() as i32).max(0),
            (b[1].floor() as i32).max(0),
            (b[2].ceil() as i32).min(gw as i32),
            (b[3].ceil() as i32).min(gh as i32),
        ];
        let mut p = crop(&screen.lit, rect)?;
        let off = (rect[0] as f32, rect[1] as f32);
        // The wake: brighter towards the orb.
        let ph = Phosphor::of(colour);
        if wake.len() > 1 {
            let segs = 8usize;
            for j in 0..segs {
                let (a, z) = (
                    j * (wake.len() - 1) / segs,
                    (j + 1) * (wake.len() - 1) / segs,
                );
                if z <= a {
                    continue;
                }
                let mut pb = PathBuilder::new();
                pb.move_to(wake[a].0 - off.0, wake[a].1 - off.1);
                for q in &wake[a + 1..=z] {
                    pb.line_to(q.0 - off.0, q.1 - off.1);
                }
                let Some(path) = pb.finish() else { continue };
                let alpha = 0.75 * (j + 1) as f32 / segs as f32;
                for (width, k) in [(6.5 * s as f32, 0.3f32), (3.2 * s as f32, 1.0)] {
                    let mut paint = Paint::default();
                    let c = if k < 1.0 { ph.bloom } else { ph.core };
                    paint.set_color(Color::from_rgba8(
                        c[0],
                        c[1],
                        c[2],
                        (255.0 * alpha * k) as u8,
                    ));
                    paint.anti_alias = true;
                    p.stroke_path(
                        &path,
                        &paint,
                        &Stroke {
                            width,
                            line_cap: LineCap::Round,
                            line_join: LineJoin::Round,
                            ..Stroke::default()
                        },
                        Transform::identity(),
                        None,
                    );
                }
            }
        }
        let half = (orb.width() as i32 / 2, orb.height() as i32 / 2);
        let pr = [0, 0, p.width() as i32, p.height() as i32];
        lay(
            &mut p,
            &orb,
            (
                (ox - off.0).round() as i32 - half.0,
                (oy - off.1).round() as i32 - half.1,
            ),
            pr,
        );
        lay(&mut p, &over, (-rect[0], -rect[1]), pr);
        // (Inside the glass's corners only: the face's own outside them.)
        for y in 0..p.height() as i32 {
            for x in 0..p.width() as i32 {
                let bi = (((y + rect[1]) * gw as i32 + x + rect[0]) * 4) as usize;
                if base.data()[bi + 3] < 255 {
                    let si = ((y * p.width() as i32 + x) * 4) as usize;
                    let src = &screen.shown.data()[bi..bi + 4];
                    p.data_mut()[si..si + 4].copy_from_slice(src);
                }
            }
        }
        Some(Patch {
            at: (g[0] + rect[0], g[1] + rect[1]),
            pixels: p,
        })
    }

    /// Column `n`'s MODULATES screen's dots drawn again if what it reads changed.
    fn text_for(&mut self, n: usize, col: &Column) -> Vec<[i32; 4]> {
        let key: Key = col
            .screen
            .bytes()
            .map(u64::from)
            .chain([u64::from(col.in_use)])
            .collect();
        if self.texts[n].as_ref().is_some_and(|(k, _)| *k == key) {
            return Vec::new();
        }
        let Some(still) = &self.still else {
            return Vec::new();
        };
        let s = self.scale;
        let oy = f64::from(top_px(s)) / s;
        let (x, y, w, h) = places(n).screen;
        let r = self.pixels((x, y, x + w, y + h));
        let Some(mut p) = crop(&still.plate, r) else {
            return Vec::new();
        };
        let lamp = if col.in_use {
            Lamp::ORANGE
        } else {
            Lamp::of((120, 70, 30))
        };
        screen_text(
            &mut p,
            s,
            (x - f64::from(r[0]) / s, y - oy - f64::from(r[1]) / s, w, h),
            &col.screen,
            &lamp,
        );
        let patch = Patch {
            at: (r[0], r[1]),
            pixels: p,
        };
        let mut out = vec![patch.rect()];
        if let Some((_, old)) = self.texts[n].replace((key, patch)) {
            out.push(old.rect());
        }
        out
    }

    /// Column `n`'s control `k` ([`LFO_PARTS`]' order; [`JACK`] its jack) drawn again if it changed:
    /// where it was and is.
    fn control_for(
        &mut self,
        n: usize,
        k: usize,
        col: &Column,
        panel: &mut Renderer,
    ) -> Vec<[i32; 4]> {
        // (LEVEL is its jack's, drawn as the jack: R-LFO's addendum.)
        if k == crate::controls::LFO_LEVEL_PART {
            return Vec::new();
        }
        let key: Key = match k {
            0 => vec![
                u64::from(col.continuous),
                word(if col.continuous {
                    col.shape_at
                } else {
                    col.wave
                }),
            ],
            1 => vec![word(col.rate), u64::from(col.timed)],
            2 => vec![u64::from(col.timed), col.division as u64],
            3 => vec![word(col.phase)],
            4 => vec![word(col.rise)],
            5 => vec![word(col.mode)],
            6 => vec![u64::from(col.mono)],
            7 => vec![u64::from(col.uni)],
            8 => vec![u64::from(col.on)],
            _ => vec![],
        };
        if self.controls[n][k]
            .as_ref()
            .is_some_and(|(was, _)| *was == key)
        {
            return Vec::new();
        }
        let worn = self.skin == Skin::Worn;
        let top = top_px(self.scale);
        let painted = if k == JACK {
            panel.paint_layer(&art::jack_layer(
                places(n).jack.0,
                places(n).jack.1,
                JACK_R,
                worn,
            ))
        } else {
            let i = lfo_index(n, k);
            let c = CONTROLS[i];
            match k {
                0 if col.continuous => panel.paint_layer(&art::selector_turned(
                    &c,
                    art::shape_angle(col.shape_at),
                    worn,
                )),
                0 => panel.paint_control(i, col.wave),
                2 | 6..=8 => {
                    let (on, word) = match k {
                        2 => (col.timed, None),
                        6 => (col.mono, Some("MONO")),
                        7 => (col.uni, Some("UNI")),
                        // (Its word is the title, printed beside it.)
                        _ => (col.on, Some("")),
                    };
                    let mut layer = art::key(
                        &c,
                        Keyed {
                            on,
                            division: col.division,
                            word,
                            way: ModWay::Clockwise,
                            ..Keyed::default()
                        },
                        self.lettering,
                        panel.layout().times(),
                        worn,
                    );
                    // (MONO's word is a letter longer than MOD's.)
                    layer.bounds[2] += 20.0;
                    panel.paint_layer(&layer)
                }
                1 => panel.paint_control(i, col.rate),
                3 => panel.paint_control(i, col.phase),
                4 => panel.paint_control(i, col.rise),
                _ => panel.paint_control(i, col.mode),
            }
        };
        let patch = painted.map(|(x, y, p)| Patch {
            at: (x, y - top),
            pixels: p,
        });
        let mut out: Vec<[i32; 4]> = patch.iter().map(Patch::rect).collect();
        if let Some((_, old)) = self.controls[n][k].take() {
            out.push(old.rect());
        }
        self.controls[n][k] = patch.map(|p| (key, p));
        out
    }

    /// The frame drawn again inside `r`: the plate, each column's printed parts, scope, orb,
    /// screen and controls, each laid only inside `r`.
    fn paint(&mut self, r: [i32; 4]) {
        let Some(still) = self.still.clone() else {
            return;
        };
        let Some(scene) = self.shown.clone() else {
            return;
        };
        glow::copy_in(&mut self.frame, &still.plate, (0, 0), r);
        for (n, col) in scene.columns.iter().enumerate() {
            if col.timed
                && let Some(Some(p)) = still.timed.get(n)
            {
                glow::copy_in(&mut self.frame, &p.pixels, p.at, r);
            }
            if col.continuous
                && let Some(Some(p)) = still.arcs.get(n)
            {
                glow::copy_in(&mut self.frame, &p.pixels, p.at, r);
            }
            let g = self.glass_px(n);
            if let Some(sc) = &self.screens[n] {
                lay(&mut self.frame, &sc.shown, (g[0], g[1]), r);
            }
            if let Some((_, p)) = &self.orbs[n] {
                lay(&mut self.frame, &p.pixels, p.at, r);
            }
            if let Some((_, p)) = &self.texts[n] {
                glow::copy_in(&mut self.frame, &p.pixels, p.at, r);
            }
            for c in self.controls[n].iter().flatten() {
                lay(&mut self.frame, &c.1.pixels, c.1.at, r);
            }
        }
    }
}

/// The frame's pixels of a rectangle of the drawing's units at `s`, from the page's top.
fn pixels_of(s: f64, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> [i32; 4] {
    let top = top_px(s);
    [
        (x0 * s).floor() as i32 - 1,
        (y0 * s).floor() as i32 - 1 - top,
        (x1 * s).ceil() as i32 + 1,
        (y1 * s).ceil() as i32 + 1 - top,
    ]
}

/// Column `n`'s scope's glass in the frame's pixels at `s`: its size the same for every column.
fn glass_px(s: f64, n: usize) -> [i32; 4] {
    let (x, y, w, h) = glass(n);
    let (x0, y0) = ((x * s).round() as i32, (y * s).round() as i32 - top_px(s));
    [
        x0,
        y0,
        x0 + (w * s).round() as i32,
        y0 + (h * s).round() as i32,
    ]
}

/// `r` cut to `bounds`; none if nothing of it is inside.
fn within(r: [i32; 4], b: [i32; 4]) -> Option<[i32; 4]> {
    let c = [
        r[0].max(b[0]),
        r[1].max(b[1]),
        r[2].min(b[2]),
        r[3].min(b[3]),
    ];
    (c[2] > c[0] && c[3] > c[1]).then_some(c)
}

/// Rectangles that meet joined into one, the rest kept apart.
fn merged(mut rects: Vec<[i32; 4]>) -> Vec<[i32; 4]> {
    let meets = |a: [i32; 4], b: [i32; 4]| a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3];
    let mut out: Vec<[i32; 4]> = Vec::new();
    while let Some(mut r) = rects.pop() {
        loop {
            let before = out.len();
            out.retain(|o| {
                if meets(*o, r) {
                    r = [
                        r[0].min(o[0]),
                        r[1].min(o[1]),
                        r[2].max(o[2]),
                        r[3].max(o[3]),
                    ];
                    false
                } else {
                    true
                }
            });
            if out.len() == before {
                break;
            }
        }
        out.push(r);
    }
    out
}

/// How far the trace swings from the glass's middle to a peak (1 or -1), the drawing's units.
pub fn trace_amplitude() -> f64 {
    glass(0).3 / 2.0 - TRACE_TOP
}

/// The trace's point `i` of `k` at `value` (-1..1), in a glass `glass` units in size drawn
/// `px` pixels in size: its pixels.
fn trace_point(i: usize, k: usize, value: f64, glass: (f64, f64), px: (u32, u32)) -> (f32, f32) {
    let (gw, gh) = glass;
    let (sx, sy) = (f64::from(px.0) / gw, f64::from(px.1) / gh);
    let f = i as f64 / (k - 1) as f64;
    let amp = gh / 2.0 - TRACE_TOP;
    (
        ((TRACE_PAD + (gw - 2.0 * TRACE_PAD) * f) * sx) as f32,
        ((gh / 2.0 - amp * value.clamp(-1.0, 1.0)) * sy) as f32,
    )
}

/// A trace drawn in its phosphor over `p` (the glass's pixels): its halo, its bloom and its
/// core; its jumps (a square's edges) faint, as a beam's fast sweep is.
fn draw_trace(
    p: &mut Pixmap,
    trace: &[f32],
    s: f64,
    colour: [u8; 3],
    (_, _, gw, gh): (f64, f64, f64, f64),
) {
    let k = trace.len();
    if k < 2 {
        return;
    }
    let (w, h) = (p.width(), p.height());
    let pts: Vec<(f32, f32)> = (0..k)
        .map(|i| trace_point(i, k, f64::from(trace[i]), (gw, gh), (w, h)))
        .collect();
    let (mut slow, mut fast) = (PathBuilder::new(), PathBuilder::new());
    let mut open = false;
    for win in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (win[0], win[1]);
        if (y1 - y0).abs() > 6.0 * (x1 - x0).abs().max(0.01) {
            fast.move_to(x0, y0);
            fast.line_to(x0, y1);
            open = false;
            continue;
        }
        if !open {
            slow.move_to(x0, y0);
            open = true;
        }
        slow.line_to(x1, y1);
    }
    let ph = Phosphor::of(colour);
    let s32 = s as f32;
    if let Some(slow) = slow.finish() {
        let mut halo = Coverage::new(w, h);
        halo.stroke(&slow, 9.0 * s32, 0.42);
        halo.blur(6.5 * s32);
        halo.tint(p, ph.halo, 1.0, (0, 0));
        let mut bloom = Coverage::new(w, h);
        bloom.stroke(&slow, 4.2 * s32, 0.85);
        bloom.blur(1.8 * s32);
        bloom.tint(p, ph.bloom, 1.0, (0, 0));
        let mut core = Coverage::new(w, h);
        core.stroke(&slow, (1.9 * s32).max(0.8), 1.0);
        core.tint(p, ph.core, 1.0, (0, 0));
    }
    if let Some(fast) = fast.finish() {
        let mut b = Coverage::new(w, h);
        b.stroke(&fast, 2.6 * s32, 0.3);
        b.blur(1.8 * s32);
        b.tint(p, ph.bloom, 1.0, (0, 0));
        let mut c = Coverage::new(w, h);
        c.stroke(&fast, (1.1 * s32).max(0.6), 0.3);
        c.tint(p, ph.core, 1.0, (0, 0));
    }
}

/// [`draw_trace`]'s trace drawn cheaply enough to be drawn again up to 30 times a second while a
/// source moves what it is drawn from (decisions.md R-LFO's corrections): its halo and bloom
/// each a few wide strokes of falling light (a brush blurred beforehand, in effect) rather than
/// a coverage blurred; its core and its jumps as the still one's.
fn draw_trace_live(
    p: &mut Pixmap,
    trace: &[f32],
    s: f64,
    colour: [u8; 3],
    (_, _, gw, gh): (f64, f64, f64, f64),
) {
    let k = trace.len();
    if k < 2 {
        return;
    }
    let (w, h) = (p.width(), p.height());
    let pts: Vec<(f32, f32)> = (0..k)
        .map(|i| trace_point(i, k, f64::from(trace[i]), (gw, gh), (w, h)))
        .collect();
    let (mut slow, mut fast) = (PathBuilder::new(), PathBuilder::new());
    let mut open = false;
    for win in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (win[0], win[1]);
        if (y1 - y0).abs() > 6.0 * (x1 - x0).abs().max(0.01) {
            fast.move_to(x0, y0);
            fast.line_to(x0, y1);
            open = false;
            continue;
        }
        if !open {
            slow.move_to(x0, y0);
            open = true;
        }
        slow.line_to(x1, y1);
    }
    let ph = Phosphor::of(colour);
    let s32 = s as f32;
    let line = |p: &mut Pixmap, path: &resvg::tiny_skia::Path, rgb: [u8; 3], width: f32, a: f32| {
        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], (a.clamp(0.0, 1.0) * 255.0) as u8);
        paint.anti_alias = true;
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        p.stroke_path(path, &paint, &stroke, Transform::identity(), None);
    };
    if let Some(slow) = slow.finish() {
        for (width, a) in [(22.0, 0.05), (15.0, 0.07), (9.0, 0.09)] {
            line(p, &slow, ph.halo, width * s32, a);
        }
        for (width, a) in [(6.0, 0.25), (3.6, 0.45)] {
            line(p, &slow, ph.bloom, width * s32, a);
        }
        line(p, &slow, ph.core, (1.9 * s32).max(0.8), 1.0);
    }
    if let Some(fast) = fast.finish() {
        line(p, &fast, ph.bloom, 3.4 * s32, 0.15);
        line(p, &fast, ph.core, (1.1 * s32).max(0.6), 0.3);
    }
}

/// A phosphor colour's tube at `s`, `size` pixels for a glass `glass` (only its size used):
/// its face, the glass over its trace, its orb.
fn make_tube(
    s: f64,
    colour: [u8; 3],
    size: (u32, u32),
    (_, _, gw, gh): (f64, f64, f64, f64),
) -> Tube {
    let ph = Phosphor::of(colour);
    let options = usvg::Options::default();
    let draw = |body: &str, (w, h): (u32, u32), view: [f64; 4]| {
        let mut p = Pixmap::new(w.max(1), h.max(1)).expect("a tube's size");
        let doc = art::document(view, w.max(1), h.max(1), body);
        if let Ok(tree) = usvg::Tree::from_str(&doc, &options) {
            resvg::render(&tree, Transform::identity(), &mut p.as_mut());
        }
        p
    };
    let view = [0.0, 0.0, gw, gh];
    let (face, edge) = (hex(ph.face.0), hex(ph.face.1));
    let bloom = hex(ph.bloom);
    let mut b = Svg::default();
    put!(
        b,
        "<defs><clipPath id='gl'><rect width='{}' height='{}' rx='{GLASS_RX}'/></clipPath><radialGradient id='ph' cx='0.5' cy='0.5' r='0.75'><stop offset='0' stop-color='{face}'/><stop offset='1' stop-color='{edge}'/></radialGradient></defs><g clip-path='url(#gl)'><rect width='{}' height='{}' fill='url(#ph)'/>",
        N(gw),
        N(gh),
        N(gw),
        N(gh)
    );
    let div = gh / 4.0;
    let (cx, cy) = (gw / 2.0, gh / 2.0);
    let mut d = String::new();
    let n = (gw / 2.0 / div).floor() as i32;
    for k in -n..=n {
        d.push_str(&format!("M {} 0 V {} ", N(cx + f64::from(k) * div), N(gh)));
    }
    for k in -2..=2 {
        d.push_str(&format!("M 0 {} H {} ", N(cy + f64::from(k) * div), N(gw)));
    }
    put!(
        b,
        "<path d='{d}' stroke='{bloom}' stroke-opacity='0.15' stroke-width='1.1' fill='none'/>"
    );
    let mut t = String::new();
    let m = (gw / 2.0 / (div / 5.0)).floor() as i32;
    for k in -m..=m {
        let x = cx + f64::from(k) * div / 5.0;
        t.push_str(&format!("M {} {} V {} ", N(x), N(cy - 4.0), N(cy + 4.0)));
    }
    for k in -10..=10 {
        let y = cy + f64::from(k) * div / 5.0;
        t.push_str(&format!("M {} {} H {} ", N(cx - 4.0), N(y), N(cx + 4.0)));
    }
    put!(
        b,
        "<path d='{t}' stroke='{bloom}' stroke-opacity='0.26' stroke-width='1.1' fill='none'/></g>"
    );
    let base = draw(&b.0, size, view);
    let mut over = glass_over(s, size, (gw, gh));
    // (Inside the glass's rounded corners only, as the face is.)
    for (o, b) in over
        .data_mut()
        .chunks_exact_mut(4)
        .zip(base.data().chunks_exact(4))
    {
        if b[3] < 255 {
            for c in o.iter_mut() {
                *c = (u16::from(*c) * u16::from(b[3]) / 255) as u8;
            }
        }
    }
    // The orb: its halo and bloom blurred, its white-hot middle.
    let side = ((2.0 * ORB_R * s).round() as u32).max(2);
    let mut orb = Pixmap::new(side, side).expect("an orb's size");
    let c = (side as f32 / 2.0, side as f32 / 2.0);
    let k = s as f32;
    let mut halo = Coverage::new(side, side);
    halo.disc(c, 15.0 * k, 0.8);
    halo.blur(6.5 * k);
    halo.tint(&mut orb, ph.halo, 1.0, (0, 0));
    let mut bloom = Coverage::new(side, side);
    bloom.disc(c, 6.5 * k, 1.0);
    bloom.blur(1.8 * k);
    bloom.tint(&mut orb, ph.bloom, 1.0, (0, 0));
    let mut core = Coverage::new(side, side);
    core.disc(c, (3.6 * k).max(0.8), 1.0);
    core.tint(&mut orb, [0xff, 0xfa, 0xf0], 1.0, (0, 0));
    Tube {
        colour,
        base,
        over,
        orb,
    }
}

/// What lies over a scope's trace, whatever its colour, `size` pixels for a glass `glass` units
/// in size: the tube's dark rim (a blurred stroke round its edge, by code), the glass's
/// reflection; inside the glass's rounded corners only.
fn glass_over(s: f64, size: (u32, u32), (gw, gh): (f64, f64)) -> Pixmap {
    let (w, h) = (size.0.max(1), size.1.max(1));
    let (kx, ky) = (f64::from(w) / gw, f64::from(h) / gh);
    let mut p = Pixmap::new(w, h).expect("a tube's size");
    let mut o = Svg::default();
    put!(
        o,
        "<defs><linearGradient id='refl' x1='0' y1='0' x2='0.55' y2='1'><stop offset='0' stop-color='#fff' stop-opacity='0.085'/><stop offset='0.45' stop-color='#fff' stop-opacity='0.015'/><stop offset='1' stop-color='#fff' stop-opacity='0'/></linearGradient></defs><rect width='{}' height='{}' rx='{GLASS_RX}' fill='url(#refl)'/>",
        N(gw),
        N(gh)
    );
    let mut refl = Pixmap::new(w, h).expect("a tube's size");
    let doc = art::document([0.0, 0.0, gw, gh], w, h, &o.0);
    if let Ok(tree) = usvg::Tree::from_str(&doc, &usvg::Options::default()) {
        resvg::render(&tree, Transform::identity(), &mut refl.as_mut());
    }
    // The rim: the glass's edge stroked 16 wide, blurred.
    let r = (GLASS_RX * kx.min(ky)) as f32;
    let (fw, fh) = (w as f32, h as f32);
    let mut pb = PathBuilder::new();
    pb.move_to(r, 0.0);
    pb.line_to(fw - r, 0.0);
    pb.quad_to(fw, 0.0, fw, r);
    pb.line_to(fw, fh - r);
    pb.quad_to(fw, fh, fw - r, fh);
    pb.line_to(r, fh);
    pb.quad_to(0.0, fh, 0.0, fh - r);
    pb.line_to(0.0, r);
    pb.quad_to(0.0, 0.0, r, 0.0);
    pb.close();
    if let Some(edge) = pb.finish() {
        let mut rim = Coverage::new(w, h);
        rim.stroke(&edge, (16.0 * s) as f32, 0.75);
        rim.blur((9.0 * s) as f32);
        rim.tint(&mut p, [0, 0, 0], 1.0, (0, 0));
    }
    lay(&mut p, &refl, (0, 0), [0, 0, w as i32, h as i32]);
    p
}

/// What never changes for `look`.
fn make_still((scale, skin, lettering): Look) -> Still {
    let s = f64::from_bits(scale);
    let options = lower::lettered();
    let fonts = Fonts::new();
    let layout = Layout::measure_in(&fonts, lettering, &Arrangement::official());
    let times = layout.times();
    let top = top_px(s);
    let oy = f64::from(top) / s;
    let mut plate = lower::plate(s, skin, DEPTH, &options);
    svg_over(&options, &mut plate, s, &surfaces(oy, skin));
    let under = plate.clone();
    let mean = crate::render::face_mean(&under);
    lower::print(
        &mut plate,
        s,
        skin,
        &options,
        &print_svg(oy, lettering, &layout, times),
    );
    let glass_picture = Pixmap::decode_png(crate::skin::GLASS).expect("a built-in picture decodes");
    for n in 0..4 {
        let (x, y, w, h) = places(n).screen;
        screen_still(&options, &mut plate, &glass_picture, s, (x, y - oy, w, h));
    }
    lower::lamp(&mut plate, s, skin, (0, top));
    let part = |n: usize, rate: bool| {
        let p = places(n);
        let (r, plain, printed) = if rate {
            let e = |d| {
                art::print_extent(&Kind::Knob {
                    dial: d,
                    big: false,
                })
            };
            let (a, b) = (e(Dial::LfoRate), e(Dial::Timed));
            let (x, y) = p.rate;
            (
                pixels_of(
                    s,
                    (
                        x + a[0].min(b[0]) - 4.0,
                        y + a[1].min(b[1]) - 4.0,
                        x + a[2].max(b[2]) + 4.0,
                        y + a[3].max(b[3]) + 4.0,
                    ),
                ),
                rate_print(n, false, lettering, oy, times),
                rate_print(n, true, lettering, oy, times),
            )
        } else {
            let e = art::ARC_EXTENT;
            let (x, y) = p.wave;
            (
                pixels_of(s, (x + e[0], y + e[1], x + e[2], y + e[3])),
                wave_print(n, false, lettering, &layout, oy),
                wave_print(n, true, lettering, &layout, oy),
            )
        };
        print_part(
            PartOf {
                s,
                skin,
                top,
                mean,
                options: &options,
                under: &under,
                plate: &plate,
            },
            r,
            &plain,
            &printed,
        )
    };
    let timed = (0..4).map(|n| part(n, true)).collect();
    let arcs = (0..4).map(|n| part(n, false)).collect();
    Still {
        look: (scale, skin, lettering),
        plate,
        timed,
        arcs,
    }
}

/// What a printed part is drawn against: the scale, skin, the page's top row, the face's mean
/// level under the print, the lettering's options, the plate before its print and after.
struct PartOf<'a> {
    s: f64,
    skin: Skin,
    top: i32,
    mean: f64,
    options: &'a usvg::Options<'a>,
    under: &'a Pixmap,
    plate: &'a Pixmap,
}

/// The plate's place `r` printed with `printed` instead of `plain` (each the place's print
/// alone, the frame's units), as the panel's renderer prints a timed dial's
/// (`Renderer::print_part`): the place under the print, the ink worn into it, the lamp over it,
/// and the plate's own pixels wherever the two prints' ink is the same, so that it lies without
/// a seam.
fn print_part(o: PartOf<'_>, r: [i32; 4], plain: &str, printed: &str) -> Option<Patch> {
    let mut part = crop(o.under, r)?;
    let (w, h) = (part.width(), part.height());
    let view = [
        f64::from(r[0]) / o.s,
        f64::from(r[1]) / o.s,
        f64::from(w) / o.s,
        f64::from(h) / o.s,
    ];
    let ink = |body: &str| {
        let mut p = Pixmap::new(w, h)?;
        let doc = art::document(view, w, h, body);
        let tree = usvg::Tree::from_str(&doc, o.options).ok()?;
        resvg::render(&tree, Transform::identity(), &mut p.as_mut());
        Some(p)
    };
    let (plain, printed) = (ink(plain)?, ink(printed)?);
    let mut worn = printed.clone();
    let opacity = if o.skin == Skin::Worn {
        crate::render::worn_into_by(&mut worn, &part, o.mean);
        0.94
    } else {
        1.0
    };
    part.draw_pixmap(
        0,
        0,
        worn.as_ref(),
        &PixmapPaint {
            opacity,
            ..PixmapPaint::default()
        },
        Transform::identity(),
        None,
    );
    if o.skin == Skin::Worn {
        crate::render::lamp_at(&mut part, o.s, (r[0], o.top + r[1]));
    }
    let fw = o.plate.width() as usize;
    let (a, b) = (plain.data(), printed.data());
    let bg = o.plate.data();
    let out = part.data_mut();
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = (y * w as usize + x) * 4;
            if a[i..i + 4] == b[i..i + 4] {
                let j = ((y + r[1] as usize) * fw + x + r[0] as usize) * 4;
                out[i..i + 4].copy_from_slice(&bg[j..j + 4]);
            }
        }
    }
    Some(Patch {
        at: (r[0], r[1]),
        pixels: part,
    })
}

/// The plate's surfaces (the frame's units, `oy` its top in the drawing's): each scope's bezel
/// with its screws; the knobs' and the jacks' shadows (the worn skin's, as the panel's).
fn surfaces(oy: f64, skin: Skin) -> String {
    let mut u = Svg::default();
    put!(u, "<g transform='translate(0 {})'>", N(-oy));
    for n in 0..4 {
        bezel(&mut u, n, places(n).scope);
    }
    if skin == Skin::Worn {
        put!(
            u,
            "<defs><filter id='long' x='-0.6' y='-0.6' width='2.2' height='2.2'><feGaussianBlur stdDeviation='7'/></filter><filter id='softer' x='-0.5' y='-0.5' width='2' height='2'><feGaussianBlur stdDeviation='2'/></filter></defs>"
        );
        for n in 0..4 {
            let p = places(n);
            for ((x, y), r) in [
                (p.wave, art::SELECTOR_R + 2.0),
                (p.rate, art::STD.skirt),
                (p.phase, art::STD.skirt),
                (p.rise, art::STD.skirt),
            ] {
                put!(
                    u,
                    "<circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.55' filter='url(#long)'/><circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.7' filter='url(#softer)'/>",
                    N(x + 10.0),
                    N(y + 14.0),
                    N(r),
                    N(x + 2.0),
                    N(y + 3.0),
                    N(r)
                );
            }
            let (jx, jy) = p.jack;
            put!(
                u,
                "<circle cx='{}' cy='{}' r='{}' fill='#000' fill-opacity='0.5' filter='url(#softer)'/>",
                N(jx + 2.0),
                N(jy + 3.0),
                N(JACK_R + 6.0)
            );
        }
    }
    u.0.push_str("</g>");
    u.0
}

/// A scope's bezel: a dark frame standing proud, its shadow below it, its edge catching the
/// lamp, a black lip down into the glass, a screw at each corner.
fn bezel(s: &mut Svg, n: usize, (x, y, w, h): (f64, f64, f64, f64)) {
    put!(
        s,
        "<defs><filter id='bz-sh-{n}' x='-0.1' y='-0.2' width='1.2' height='1.4'><feGaussianBlur stdDeviation='6'/></filter><linearGradient id='bz-{n}' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#2e2c28'/><stop offset='0.55' stop-color='#181715'/><stop offset='1' stop-color='#0d0c0b'/></linearGradient><linearGradient id='bz-edge-{n}' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#fff' stop-opacity='0.30'/><stop offset='0.5' stop-color='#fff' stop-opacity='0.05'/><stop offset='1' stop-color='#000' stop-opacity='0.5'/></linearGradient><linearGradient id='bz-lip-{n}' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.8'/><stop offset='0.7' stop-color='#000' stop-opacity='0'/><stop offset='1' stop-color='#fff' stop-opacity='0.18'/></linearGradient></defs>"
    );
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='16' fill='#000' fill-opacity='0.6' filter='url(#bz-sh-{n})'/><rect x='{}' y='{}' width='{}' height='{}' rx='16' fill='url(#bz-{n})'/><rect x='{}' y='{}' width='{}' height='{}' rx='15.4' fill='none' stroke='url(#bz-edge-{n})' stroke-width='1.4'/>",
        N(x + 4.0),
        N(y + 7.0),
        N(w),
        N(h),
        N(x),
        N(y),
        N(w),
        N(h),
        N(x + 0.7),
        N(y + 0.7),
        N(w - 1.4),
        N(h - 1.4)
    );
    let (gx, gy, gw, gh) = (
        x + GLASS_IN,
        y + GLASS_IN,
        w - 2.0 * GLASS_IN,
        h - 2.0 * GLASS_IN,
    );
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='26' fill='#030302'/><rect x='{}' y='{}' width='{}' height='{}' rx='25' fill='none' stroke='url(#bz-lip-{n})' stroke-width='3'/>",
        N(gx - 5.0),
        N(gy - 5.0),
        N(gw + 10.0),
        N(gh + 10.0),
        N(gx - 4.0),
        N(gy - 4.0),
        N(gw + 8.0),
        N(gh + 8.0)
    );
    for (sx, sy) in [
        (x + 10.0, y + 10.0),
        (x + w - 10.0, y + 10.0),
        (x + 10.0, y + h - 10.0),
        (x + w - 10.0, y + h - 10.0),
    ] {
        art::screw_at(s, sx, sy, 4.4);
    }
}

/// Text at (`x`, `y`), centred, the panel's legends' print.
fn text(t: &mut Svg, x: f64, y: f64, w: &str, size: f64) {
    art::text(t, x, y, w, size, art::Style::Legend, false, "");
}

/// The page's print (the frame's units): the rules between the columns, each column's title and
/// legends, its dials (RATE in Hz) and WAVEFORM's marks (STEPPED).
fn print_svg(oy: f64, lettering: Lettering, layout: &Layout, times: &str) -> String {
    let mut t = Svg(String::new(), lettering);
    put!(t, "<g transform='translate(0 {})'>", N(-oy));
    let legend = lettering.strip_legend();
    let small = lettering.legend();
    for k in 1..4 {
        let x = X0 + COL_W * f64::from(k);
        put!(
            t,
            "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{}' stroke-width='4' stroke-linecap='round'/>",
            N(x),
            N(TOP + 16.0),
            N(x),
            N(H - 14.0),
            colour::LEGEND
        );
    }
    for n in 0..4 {
        let p = places(n);
        art::text(
            &mut t,
            centre(n),
            TITLE_Y,
            &format!("LFO {}", n + 1),
            TITLE,
            art::Style::Title,
            false,
            "",
        );
        for ((x, y), w) in [
            (p.wave, "WAVEFORM"),
            (p.rate, "RATE"),
            (p.phase, "PHASE"),
            (p.rise, "RISE"),
        ] {
            text(&mut t, x, y - LEGEND_UP, w, legend);
        }
        // MODE: its title, RETRIG over the lever (out), FREE and ENV at its ends.
        let (mx, my) = p.mode;
        text(&mut t, mx, my - 68.0, "MODE", legend);
        text(&mut t, mx, my - 40.0, "RETRIG", small);
        text(&mut t, mx - 70.0, my, "FREE", small);
        text(&mut t, mx + 66.0, my, "ENV", small);
        let (jx, jy) = p.jack;
        text(&mut t, jx, jy - 44.0, "OUT", small);
        let (sx, sy, sw, _) = p.screen;
        text(&mut t, sx + sw / 2.0, sy - 27.0, "MODULATES", small);
        art::dial_print(&mut t, p.rate.0, p.rate.1, Dial::LfoRate, times);
        art::dial_print(&mut t, p.phase.0, p.phase.1, Dial::LfoPhase, times);
        art::dial_print(&mut t, p.rise.0, p.rise.1, Dial::LfoRise, times);
        art::waveform_marks(&mut t, p.wave.0, p.wave.1, layout, false);
    }
    t.0.push_str("</g>");
    t.0
}

/// Column `n`'s RATE dial's print alone (the frame's units): in Hz, or `timed` in note values.
fn rate_print(n: usize, timed: bool, lettering: Lettering, oy: f64, times: &str) -> String {
    let mut t = Svg(String::new(), lettering);
    put!(t, "<g transform='translate(0 {})'>", N(-oy));
    let (x, y) = places(n).rate;
    art::dial_print(
        &mut t,
        x,
        y,
        if timed { Dial::Timed } else { Dial::LfoRate },
        times,
    );
    t.0.push_str("</g>");
    t.0
}

/// Column `n`'s WAVEFORM's marks alone (the frame's units), with CONTINUOUS's arc (`arc`).
fn wave_print(n: usize, arc: bool, lettering: Lettering, layout: &Layout, oy: f64) -> String {
    let mut t = Svg(String::new(), lettering);
    put!(t, "<g transform='translate(0 {})'>", N(-oy));
    let (x, y) = places(n).wave;
    art::waveform_marks(&mut t, x, y, layout, arc);
    t.0.push_str("</g>");
    t.0
}

/// The parts of a column that are controls, by [`LFO_PARTS`].
pub fn part_param(n: usize, k: usize) -> &'static str {
    CONTROLS[lfo_index(n, k)].param
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::LFO_PARTS;
    use crate::interact::{Target, hit_lfo};

    /// Each control is found where it stands, and its column's other parts where they are drawn;
    /// every part inside its column, clear of the rules; the controls' parameters the engine's
    /// IDs (`lfoN_<part>`).
    #[test]
    fn the_page_finds_each_part_where_it_is_drawn() {
        let layout = Layout::measure(&Fonts::new());
        for n in 0..4 {
            for (k, part) in LFO_PARTS.iter().enumerate() {
                let i = lfo_index(n, k);
                assert_eq!(CONTROLS[i].param, format!("lfo{}_{part}", n + 1));
                let (x, y) = CONTROLS[i].centre();
                assert_eq!(hit_lfo(&layout, x, y), Some(Target::Control(i)), "{part}");
                let (c0, c1) = (X0 + COL_W * n as f64, X0 + COL_W * (n + 1) as f64);
                assert!(x > c0 + 20.0 && x < c1 - 20.0, "{part} in its column");
                assert!(inside(x, y));
            }
            for t in [
                LfoTarget::Scope(n),
                LfoTarget::Jack(n),
                LfoTarget::Screen(n),
            ] {
                let (x, y) = target_centre(t);
                assert_eq!(hit(x, y), Some(t));
                // (The jack is its LEVEL's control: R-LFO's addendum.)
                let jack = matches!(t, LfoTarget::Jack(_)).then_some(Target::Control(lfo_index(
                    n,
                    crate::controls::LFO_LEVEL_PART,
                )));
                assert_eq!(hit_lfo(&layout, x, y), jack, "{t:?}");
            }
            let p = places(n);
            let (sx, _, sw, _) = p.scope;
            assert!(sx > X0 + COL_W * n as f64 && sx + sw < p.mode.0 - 60.0);
            let (x, _, w, _) = p.screen;
            assert!(x + w < X0 + COL_W * (n + 1) as f64 - 10.0);
        }
        assert_eq!(hit(LEFT - 1.0, ROW1), None);
        assert_eq!(hit(1000.0, TOP - 1.0), None);
    }

    fn scene() -> LfoScene {
        let mut s = LfoScene::default();
        for (n, c) in s.columns.iter_mut().enumerate() {
            c.trace = (0..TRACE_POINTS)
                .map(|i| ((i as f32) / 40.0 + n as f32).sin())
                .collect();
            c.orb = Some((0.3, 0.2));
        }
        s.columns[1].colour = [0x3c, 0xe8, 0xa0];
        s
    }

    /// The page drawn in parts is the page drawn whole: a knob turned, the trace changed, the
    /// orb moved, the timed key and CONTINUOUS's arc swapped in and out, a colour changed and
    /// MODULATES' text, at the window's opening scale and a Retina screen's, in both skins.
    #[test]
    fn drawn_in_parts_it_is_drawn_whole() {
        for (scale, skin) in [(0.44, Skin::Worn), (0.87, Skin::Drawn)] {
            let mut panel = Renderer::with_skin(skin, scale, 1.0);
            let mut r = LfoRenderer::new(scale, skin, Lettering::Standard);
            let mut sc = scene();
            assert!(r.render(&sc, &mut panel));
            let steps: [&dyn Fn(&mut LfoScene); 9] = [
                &|s| s.columns[0].phase = 0.7,
                &|s| s.columns[2].orb = Some((0.8, -0.5)),
                &|s| s.columns[2].orb_rise = 0.4,
                &|s| s.columns[1].trace[100] = 0.9,
                &|s| {
                    s.columns[3].timed = true;
                    s.columns[3].division = 5;
                    s.columns[3].rate = 5.0 / 17.0;
                },
                &|s| {
                    s.columns[0].continuous = true;
                    s.columns[0].shape_at = 2.4;
                },
                &|s| s.columns[2].colour = [0xff, 0x50, 0x78],
                &|s| {
                    s.columns[1].screen = "CUTOFF -25".into();
                    s.columns[1].in_use = true;
                    s.columns[1].mono = false;
                },
                &|s| {
                    s.columns[3].timed = false;
                    s.columns[0].continuous = false;
                    s.columns[2].orb = None;
                },
            ];
            for step in steps {
                step(&mut sc);
                assert!(r.render(&sc, &mut panel));
                assert!(!r.damages().is_empty());
                let mut whole = LfoRenderer::new(scale, skin, Lettering::Standard);
                whole.render(&sc, &mut panel);
                assert!(
                    r.frame() == whole.frame(),
                    "{scale} {skin:?}: drawn in parts"
                );
            }
            assert!(!r.render(&sc, &mut panel), "nothing changed");
        }
    }

    /// An orb moving draws only its patches: a few hundredths of the page at most.
    #[test]
    fn an_orb_moving_draws_only_its_patches() {
        let scale = 0.44;
        let mut panel = Renderer::with_skin(Skin::Worn, scale, 1.0);
        let mut r = LfoRenderer::new(scale, Skin::Worn, Lettering::Standard);
        let mut sc = scene();
        r.render(&sc, &mut panel);
        sc.columns[0].orb = Some((0.35, 0.1));
        assert!(r.render(&sc, &mut panel));
        let area: i64 = r
            .damages()
            .iter()
            .map(|d| i64::from(d[2] - d[0]) * i64::from(d[3] - d[1]))
            .sum();
        let page = i64::from(r.frame().width()) * i64::from(r.frame().height());
        assert!(area * 50 < page, "{area} of {page}");
    }

    /// A RATE printed in note values laid as a part is the page printed so whole, within a few
    /// pixels of two levels (the panel's own timed dials' measure).
    #[test]
    fn a_timed_rates_part_is_its_whole_print() {
        let (s, skin, lettering) = (0.44_f64, Skin::Worn, Lettering::Standard);
        let still = make_still((s.to_bits(), skin, lettering));
        let p = still.timed[1].as_ref().expect("a part");
        // The page printed with column 1's RATE timed, whole.
        let options = lower::lettered();
        let layout = Layout::measure_in(&Fonts::new(), lettering, &Arrangement::official());
        let (top, oy) = (top_px(s), f64::from(top_px(s)) / s);
        let mut plate = lower::plate(s, skin, DEPTH, &options);
        svg_over(&options, &mut plate, s, &surfaces(oy, skin));
        let body = print_svg(oy, lettering, &layout, layout.times()).replace(
            &rate_print(1, false, lettering, oy, layout.times())
                .trim_start_matches(&format!("<g transform='translate(0 {})'>", N(-oy)))
                .trim_end_matches("</g>")
                .to_owned(),
            rate_print(1, true, lettering, oy, layout.times())
                .trim_start_matches(&format!("<g transform='translate(0 {})'>", N(-oy)))
                .trim_end_matches("</g>"),
        );
        lower::print(&mut plate, s, skin, &options, &body);
        lower::lamp(&mut plate, s, skin, (0, top));
        let (w, fw) = (p.pixels.width() as usize, plate.width() as usize);
        let mut off = 0;
        for y in 0..p.pixels.height() as usize {
            for x in 0..w {
                let a = &p.pixels.data()[(y * w + x) * 4..][..4];
                let j = ((y + p.at.1 as usize) * fw + x + p.at.0 as usize) * 4;
                let b = &plate.data()[j..j + 4];
                if a.iter().zip(b).any(|(a, b)| a.abs_diff(*b) > 2) {
                    off += 1;
                }
            }
        }
        assert!(off <= 3, "{off} pixels off");
    }
}
