//! The effects' drawer, the standalone application's alone (decisions.md R-FX-DRAWER; the
//! owner's FX-A, 2026-10-18): one row dropped from under the rail by its FX key, as the
//! arpeggiator's drawer drops (`lower`: the same plate, edge and shadow), over the top of
//! whichever lower page shows. Four slots side by side, the chain's order left to right, each a
//! SLOT legend over a dot screen of twelve places (the rail's name's dots, glass and bezel) and an
//! ON tab beside it, lit while its effect plays. An empty slot reads "+ ADD EFFECT"; an effect
//! that is not playing for a reason of its own (loading, restarting, turned off after it crashed
//! or stopped responding, not installed here, off for a safe start) has its name dimmed and the
//! reason in small print under it.
//!
//! The effects' list drops from under the drawer's foot below the slot it is for: a glass display
//! of dots, as the presets' list is, on a plate of its own hanging from the drawer, each effect a
//! line of its name, maker and format.
//!
//! What does not change is drawn once a scale on a thread of its own, begun as the editor opens;
//! then only the slots that changed, and the list when it does.

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use plugin_kit_materials::{Disc, Lighting, discs, font, glass, sprite};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

use super::TAB;
use super::lower::{self, LitTabs, Look};
pub use super::lower::{LEFT, RIGHT, SHADOW, TOP, top_px};
use super::worn::{CAP_LIFT, DOT, ORANGE, ORANGE_GLOW, copy, meets, svg_over, union, within};
use crate::art::Lettering;
use crate::fonts::FAMILY;
use crate::skin::Skin;
use crate::svg::{N, Svg, colour, put};

// ---- The layout (the drawing's units, the drawer open).

/// How deep it is (the owner's FX-A: "about 170").
pub const DEPTH: f64 = 170.0;
/// The chain's length (the owner: four).
pub const SLOTS: usize = 4;
/// A slot's screen (the rail's name's dots, twelve places: "about 400 units wide").
pub const SCREEN: (f64, f64) = (400.0, 50.0);
/// How many characters a screen shows.
pub const PLACES: usize = 12;
/// The gap between a screen and its tab.
const GAP: f64 = 20.0;
/// A slot across: its screen, the gap and its tab.
const UNIT: f64 = SCREEN.0 + GAP + TAB.0;
/// The row's middle; the legends' and the small print's under the screens.
pub const ROW: f64 = TOP + 96.0;
const LEGEND_Y: f64 = TOP + 40.0;
const WORD_Y: f64 = TOP + 147.0;
/// How far round a screen its surround stands (the rail's).
const SURROUND: f64 = 6.0;

/// Slot `i`'s middle across: the four in equal shares of the drawer.
fn middle(i: usize) -> f64 {
    LEFT + (RIGHT - LEFT) / SLOTS as f64 * (i.min(SLOTS - 1) as f64 + 0.5)
}

/// Slot `i`'s screen's window: left, top, width, height.
pub fn screen(i: usize) -> (f64, f64, f64, f64) {
    (
        middle(i) - UNIT / 2.0,
        ROW - SCREEN.1 / 2.0,
        SCREEN.0,
        SCREEN.1,
    )
}

/// Slot `i`'s ON tab's middle.
pub fn tab(i: usize) -> (f64, f64) {
    (middle(i) - UNIT / 2.0 + SCREEN.0 + GAP + TAB.0 / 2.0, ROW)
}

/// What a pointer finds on the drawer or its list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FxTarget {
    /// Slot `i`'s screen: its effect's window opened, the slot dragged elsewhere, or the list
    /// for it.
    Name(usize),
    /// Slot `i`'s ON tab.
    Tab(usize),
    /// The list's line for its row `i` (counted from its first row, not the first in view).
    Row(usize),
    /// A grouping word on FIND’s line.
    Order(FxOrder),
    /// FIND, including a character’s place in its visible text.
    Find(usize),
    /// The list, but none of its rows.
    List,
}

/// Whether (`x`, `y`) is on the drawer, open.
pub fn inside(x: f64, y: f64) -> bool {
    (LEFT..RIGHT).contains(&x) && (TOP..TOP + DEPTH).contains(&y)
}

/// What a pointer at (`x`, `y`) finds: on the list, if it is open and there; else on the drawer.
pub fn hit(x: f64, y: f64, list: Option<&FxList>) -> Option<FxTarget> {
    if let Some(l) = list
        && let Some(row) = list_hit(l, x, y)
    {
        return Some(row);
    }
    if !inside(x, y) {
        return None;
    }
    (0..SLOTS).find_map(|i| {
        let (sx, sy, sw, sh) = screen(i);
        let (tx, ty) = tab(i);
        if (sx - SURROUND..sx + sw + SURROUND).contains(&x)
            && (sy - SURROUND..sy + sh + SURROUND).contains(&y)
        {
            Some(FxTarget::Name(i))
        } else if (x - tx).abs() <= TAB.0 / 2.0 && (y - ty).abs() <= TAB.1 / 2.0 + 4.0 {
            Some(FxTarget::Tab(i))
        } else {
            None
        }
    })
}

/// A slot's part's middle (the tests', and a menu's place).
pub fn centre(t: FxTarget) -> (f64, f64) {
    match t {
        FxTarget::Name(i) => {
            let (x, y, w, h) = screen(i);
            (x + w / 2.0, y + h / 2.0)
        }
        FxTarget::Tab(i) => tab(i),
        FxTarget::Row(_) | FxTarget::List | FxTarget::Find(_) | FxTarget::Order(_) => (0.0, 0.0),
    }
}

/// Why an effect is not playing, said in small print under its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FxWord {
    /// Being loaded (or put in another's place).
    Loading,
    /// Its helper died or hung; it is being started again.
    Restarting,
    /// Turned off after it crashed (as it loaded, or again and again, or by its settings): dry,
    /// and off at the next start until tried again (decisions.md R-FX-SAFETY).
    Crashed,
    /// Turned off after it stopped responding (as it loaded, or again and again).
    NotResponding,
    /// Its plug-in could not be loaded here (not installed, say): dry, its state kept.
    Missing,
    /// Chosen from the list, it would not load (said for a few seconds where it was to go); or
    /// its settings set aside, given back, still crashed it.
    Failed,
    /// Not loaded this time: a safe start (R-FX-SAFETY).
    Off,
}

impl FxWord {
    pub const ALL: [FxWord; 7] = [
        FxWord::Loading,
        FxWord::Restarting,
        FxWord::Crashed,
        FxWord::NotResponding,
        FxWord::Missing,
        FxWord::Failed,
        FxWord::Off,
    ];

    pub fn text(self) -> &'static str {
        match self {
            FxWord::Loading => "LOADING",
            FxWord::Restarting => "RESTARTING",
            FxWord::Crashed => "CRASHED",
            FxWord::NotResponding => "NOT RESPONDING",
            FxWord::Missing => "MISSING",
            FxWord::Failed => "FAILED",
            FxWord::Off => "OFF",
        }
    }
}

/// What a slot shows: its screen's text (none: dark; "+ ADD EFFECT"; an effect's name), dimmed
/// or not; its tab lit (its effect playing); why its effect does not play.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FxSlot {
    pub text: String,
    pub dim: bool,
    pub lit: bool,
    pub word: Option<FxWord>,
}

/// The effects’ order, NAME unless the player chooses another word (R-FX-SEARCH).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FxOrder {
    #[default]
    Name,
    Maker,
    Type,
    Recent,
}

impl FxOrder {
    pub const ALL: [Self; 4] = [Self::Name, Self::Maker, Self::Type, Self::Recent];

    pub fn word(self) -> &'static str {
        match self {
            Self::Name => "NAME",
            Self::Maker => "MAKER",
            Self::Type => "TYPE",
            Self::Recent => "RECENT",
        }
    }
}

/// A line of the effects' list: an effect's name, its maker and its format; or something to do
/// (scan again), its words in the name's place.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FxRow {
    pub name: String,
    pub maker: String,
    pub format: String,
    pub action: bool,
    pub kind: String,
    /// A group’s count, only on a head (heads cannot be chosen).
    pub count: Option<usize>,
}

/// The effects' list, dropped for slot `slot`: its title and what it says at the title's right
/// (scanning, how many it found), its rows from `first` in view, the one under the pointer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FxList {
    pub slot: usize,
    pub title: String,
    pub note: String,
    pub rows: Vec<FxRow>,
    pub first: usize,
    pub hover: Option<usize>,
    pub chosen: Option<usize>,
    pub query: crate::presets::Field,
    pub order: FxOrder,
}

/// What the drawer shows: its slots; the pointer's part; while a slot is dragged, which it came
/// from and which it is over; the effects' list, open.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FxScene {
    pub slots: [FxSlot; SLOTS],
    pub hover: Option<FxTarget>,
    pub drag: Option<(usize, Option<usize>)>,
    pub list: Option<FxList>,
}

// ---- The list's layout.

/// How many rows it shows (the rest scrolled to): always as many, as the presets' list does, so
/// that its plate is made once, as the editor opens.
pub const LIST_ROWS: usize = 10;
/// Its plate's width, the margin round its display, the display's own inside its glass, a line's
/// height and its dots' pitch.
const LIST_W: f64 = 1480.0;
const MARGIN: f64 = 20.0;
const PAD: f64 = 10.0;
pub const LINE: f64 = 40.0;
const LIST_DOT: f64 = 4.0;
/// Its columns, in characters: the pointer's mark, the name, the maker, the format (the arrows
/// for more above and below in the last).
const COLS: (usize, usize, usize, usize, usize) = (0, 2, 27, 42, 53);
const NAME_CHARS: usize = 24;
const MAKER_CHARS: usize = 14;
const TYPE_CHARS: usize = 10;
const FORMAT_CHARS: usize = 4;

/// Its plate's height.
const LIST_H: f64 = 2.0 * (MARGIN + PAD) + LINE * (2 + LIST_ROWS) as f64;
/// How many characters a line holds.
const LIST_CHARS: usize = ((LIST_W - 2.0 * (MARGIN + PAD)) / (6.0 * LIST_DOT)) as usize;

/// Its plate: left, top, width, height (hanging from the drawer's foot under its slot's screen,
/// within the drawer's ends).
pub fn list_rect(list: &FxList) -> (f64, f64, f64, f64) {
    let x = (screen(list.slot).0 - MARGIN).clamp(LEFT + 8.0, RIGHT - 8.0 - LIST_W);
    (x, TOP + DEPTH, LIST_W, LIST_H)
}

/// Its display's window, inside the plate (the plate's units, from its top left corner).
const DISPLAY: (f64, f64, f64, f64) =
    (MARGIN, MARGIN, LIST_W - 2.0 * MARGIN, LIST_H - 2.0 * MARGIN);

/// Line `k`'s middle down from the plate's top (0 the title's).
fn line_y(k: usize) -> f64 {
    DISPLAY.1 + PAD + LINE * (k as f64 + 0.5)
}

/// The grouping words’ first character on FIND’s line, matching mock-up A.
pub fn order_col(order: FxOrder) -> usize {
    match order {
        FxOrder::Name => 34,
        FxOrder::Maker => 40,
        FxOrder::Type => 47,
        FxOrder::Recent => 53,
    }
}

/// What a pointer finds on the list; heads and blank lines are inert.
pub fn list_hit(list: &FxList, x: f64, y: f64) -> Option<FxTarget> {
    let (lx, ly, lw, lh) = list_rect(list);
    if !(lx..lx + lw).contains(&x) || !(ly..ly + lh).contains(&y) {
        return None;
    }
    let k = ((y - ly - DISPLAY.1 - PAD) / LINE).floor();
    if k == 1.0 {
        let col = (x - lx - DISPLAY.0 - PAD) / (6.0 * LIST_DOT);
        for order in FxOrder::ALL {
            let start = order_col(order) as f64;
            if (start..start + order.word().len() as f64).contains(&col) {
                return Some(FxTarget::Order(order));
            }
        }
        return Some(FxTarget::Find((col - 5.0).round().max(0.0) as usize));
    }
    if k < 2.0 || k as usize >= LIST_ROWS + 2 {
        return Some(FxTarget::List);
    }
    let row = list.first + k as usize - 2;
    Some(match list.rows.get(row) {
        Some(r) if r.count.is_none() && !r.name.is_empty() => FxTarget::Row(row),
        _ => FxTarget::List,
    })
}

/// Row `row`'s middle, in view (the tests').
pub fn row_centre(list: &FxList, row: usize) -> (f64, f64) {
    let (x, y, _, _) = list_rect(list);
    (
        x + DISPLAY.0 + DISPLAY.2 / 2.0,
        y + line_y(row.saturating_sub(list.first) + 2),
    )
}

// ---- Drawing.

/// What does not change at a scale: the plate with its print, the screens' bezels, glass and
/// unlit dots, the unlit tabs; the tabs' words and the small print's words apart; the shadow.
struct Still {
    look: Look,
    plate: Pixmap,
    words: Vec<(i32, i32, Pixmap)>,
    /// Each slot's small print, a patch for each word, in `FxWord::ALL`'s order.
    small: Vec<Vec<(i32, i32, Pixmap)>>,
    shadow: Pixmap,
    lighting: Lighting,
    /// The effects' list's plate (its face, its display's bezel, glass and unlit dots, its
    /// shadow), its top left corner its pixels' (the lamp's light laid over it where it drops).
    list: Pixmap,
}

/// The list as drawn: what it showed, its plate with the lamp's light where it is, its frame,
/// and where that is in the drawer's frame's pixels.
struct ListFrame {
    shown: FxList,
    plate: Pixmap,
    frame: Pixmap,
    left: i32,
    top: i32,
}

/// The drawer's renderer at the panel's scale.
pub struct FxRenderer {
    scale: f64,
    skin: Skin,
    lettering: Lettering,
    options: usvg::Options<'static>,
    warming: Option<JoinHandle<Still>>,
    still: Option<Arc<Still>>,
    frame: Pixmap,
    cover: Vec<u8>,
    lit: LitTabs,
    list: Option<ListFrame>,
    shown: Option<FxScene>,
    damage: Option<[i32; 4]>,
}

impl fmt::Debug for FxRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FxRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl FxRenderer {
    /// At `scale`, in `skin` and `lettering`: what does not change made on a thread of its own,
    /// begun now (as the editor opens, so that the drawer's first drop waits for none of it).
    pub fn new(scale: f64, skin: Skin, lettering: Lettering) -> Self {
        let mut r = FxRenderer {
            scale,
            skin,
            lettering,
            // (What it draws as it goes, the dots and the pointer's light, has no lettering.)
            options: usvg::Options::default(),
            warming: None,
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            cover: Vec::new(),
            lit: LitTabs::new(TAB),
            list: None,
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
            .name("ca72-fx".into())
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
        self.list = None;
        self.shown = None;
        self.warm();
    }

    /// The frame: the drawer's, from its top (`top_px`), the drawing's width (only its plate,
    /// `LEFT` to `RIGHT`, is shown), `DEPTH` deep.
    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// What is laid over the page under the drawer's foot (premultiplied, partly clear), each
    /// with its top left corner in the frame's pixels: the shadow it casts, and the effects'
    /// list, open, with its own.
    pub fn under(&self) -> Vec<(&Pixmap, i32, i32)> {
        let foot = (DEPTH * self.scale).round() as i32;
        let mut out = Vec::new();
        if let Some(s) = &self.still {
            out.push((&s.shadow, 0, foot));
        }
        if let Some(l) = &self.list {
            out.push((&l.frame, l.left, l.top));
        }
        out
    }

    /// The pixels the last render changed, in the frame's (left, top, right, bottom; below its
    /// foot where the list changed).
    pub fn damage(&self) -> Option<[i32; 4]> {
        self.damage
    }

    /// Drawn for `scene`: whether anything changed.
    pub fn render(&mut self, scene: &FxScene) -> bool {
        self.damage = None;
        if self.shown.as_ref() == Some(scene) && self.still.is_some() {
            return false;
        }
        if self.still.is_none() {
            let still = self.still_now();
            self.frame = still.plate.clone();
            self.cover = vec![0; (self.frame.width() * self.frame.height()) as usize];
            self.still = Some(still);
            self.shown = None;
        }
        let all = [0, 0, self.frame.width() as i32, self.frame.height() as i32];
        let was = self.shown.take();
        let mut dirty: Option<[i32; 4]> = None;
        for i in 0..SLOTS {
            if was.as_ref().is_none_or(|w| slot_changed(w, scene, i)) {
                dirty = Some(union(dirty, self.slot_part(i)));
            }
        }
        // (Each slot's part lies apart from the others': a slot changed is drawn again whole.)
        if let Some(r) = dirty.and_then(|r| within(r, all)) {
            let r = self.closed(r);
            self.paint(scene, r);
            self.damage = Some(r);
        }
        if was.as_ref().map(|w| &w.list) != Some(&scene.list) {
            let before = self.list_part();
            self.draw_list(scene.list.as_ref());
            for p in [before, self.list_part()].into_iter().flatten() {
                self.damage = Some(union(self.damage, p));
            }
        }
        self.shown = Some(scene.clone());
        self.damage.is_some()
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

    /// Where slot `i` draws: its screen and its dots' glow, the ring round it, its tab and the
    /// tab's light round it, its small print.
    fn slot_part(&self, i: usize) -> [i32; 4] {
        let (x, _, _, _) = screen(i);
        let (tx, ty) = tab(i);
        self.pixels((
            x - SURROUND - 8.0,
            ty - TAB.1 / 2.0 - 42.0,
            tx + TAB.0 / 2.0 + 42.0,
            WORD_Y + 14.0,
        ))
    }

    /// `r` grown to the whole of each slot's part it reaches into.
    fn closed(&self, mut r: [i32; 4]) -> [i32; 4] {
        for i in 0..SLOTS {
            let p = self.slot_part(i);
            if meets(p, r) {
                r = union(Some(r), p);
            }
        }
        r
    }

    /// The frame drawn again inside `r` (each slot reaching into it lying inside it).
    fn paint(&mut self, scene: &FxScene, r: [i32; 4]) {
        let Some(still) = self.still.clone() else {
            return;
        };
        let s = self.scale;
        let oy = f64::from(top_px(s)) / s;
        copy(&mut self.frame, &still.plate, r);
        let mut light = Svg::default();
        for (i, slot) in scene.slots.iter().enumerate() {
            if !meets(self.slot_part(i), r) {
                continue;
            }
            // The tab: lit while the effect plays; its word over it.
            if slot.lit {
                self.lit
                    .light(&mut self.frame, &still.lighting, s, tab(i), oy);
            }
            if let Some((x, y, p)) = still.words.get(i) {
                crate::render::put(&mut self.frame, p, (*x, *y), 1.0);
            }
            // The screen's dots (the dragged slot's dimmed while it is carried); the small print.
            let carried = scene.drag.is_some_and(|(from, _)| from == i);
            let level = if slot.dim || carried { 0.45 } else { 1.0 };
            let (x, y, w, h) = screen(i);
            let (left, top) = screen_grid((x, y - oy, w, h));
            text_dots(
                &mut self.frame,
                &mut self.cover,
                s,
                (left, top, DOT),
                &centred(&slot.text, PLACES),
                level,
            );
            if let Some(word) = slot.word
                && let Some(k) = FxWord::ALL.iter().position(|w| *w == word)
                && let Some((x, y, p)) = still.small.get(i).and_then(|v| v.get(k))
            {
                crate::render::put(&mut self.frame, p, (*x, *y), 1.0);
            }
            // The pointer's part lightened; the slot a dragged one would go to, ringed.
            let pointed = match scene.hover {
                Some(FxTarget::Name(j)) if j == i && scene.drag.is_none() => {
                    Some((x - 2.0, y - 2.0, w + 4.0, h + 4.0, 6.0))
                }
                Some(FxTarget::Tab(j)) if j == i && scene.drag.is_none() => {
                    let (tx, ty) = tab(i);
                    Some((
                        tx - TAB.0 / 2.0 + 2.0,
                        ty - TAB.1 / 2.0 + 2.0,
                        TAB.0 - 4.0,
                        TAB.1 - 4.0,
                        8.0,
                    ))
                }
                _ => None,
            };
            if let Some((px, py, pw, ph, rx)) = pointed {
                put!(
                    light,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='#fff' fill-opacity='0.07'/>",
                    N(px),
                    N(py - oy),
                    N(pw),
                    N(ph),
                    N(rx)
                );
            }
            if scene.drag.and_then(|(_, o)| o) == Some(i) {
                put!(
                    light,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='7' fill='none' stroke='{}' stroke-opacity='0.8' stroke-width='3'/>",
                    N(x - SURROUND - 3.0),
                    N(y - SURROUND - 3.0 - oy),
                    N(w + 2.0 * SURROUND + 6.0),
                    N(h + 2.0 * SURROUND + 6.0),
                    crate::learn::ACCENT
                );
            }
        }
        if !light.0.is_empty() {
            svg_over(&self.options, &mut self.frame, s, &light.0);
        }
    }

    /// Where the list is drawn, in the frame's pixels (its shadow included), if it is open.
    fn list_part(&self) -> Option<[i32; 4]> {
        self.list.as_ref().map(|l| {
            [
                l.left,
                l.top,
                l.left + l.frame.width() as i32,
                l.top + l.frame.height() as i32,
            ]
        })
    }

    /// The list drawn for `list`, or put away.
    fn draw_list(&mut self, list: Option<&FxList>) {
        let (Some(list), Some(still)) = (list, &self.still) else {
            self.list = None;
            return;
        };
        let s = self.scale;
        let (x, y, _, _) = list_rect(list);
        let (left, top) = ((x * s).round() as i32, (y * s).round() as i32 - top_px(s));
        let plate = match self.list.take().filter(|l| (l.left, l.top) == (left, top)) {
            Some(l) => l.plate,
            None => {
                let mut p = still.list.clone();
                lower::lamp(&mut p, s, self.skin, (left, top + top_px(s)));
                p
            }
        };
        let mut frame = plate.clone();
        let mut cover = vec![0; (frame.width() * frame.height()) as usize];
        let at = |col: usize, k: usize| {
            (
                DISPLAY.0 + PAD + col as f64 * 6.0 * LIST_DOT,
                line_y(k) - CAP_LIFT - 3.0 * LIST_DOT,
            )
        };
        let first = list.query.caret.saturating_sub(FIND_CHARS - 1);
        {
            let mut line = |frame: &mut Pixmap, (lx, ly): (f64, f64), text: &[char], level: f64| {
                text_dots(frame, &mut cover, s, (lx, ly, LIST_DOT), text, level);
            };
            // The title, dim; what the list says at its right.
            let (tx, ty) = at(0, 0);
            line(&mut frame, (tx, ty), &fit(&list.title, LIST_CHARS), 0.55);
            let note = fit(&list.note, LIST_CHARS);
            line(
                &mut frame,
                (at(LIST_CHARS - note.len(), 0).0, ty),
                &note,
                0.55,
            );
            line(&mut frame, at(0, 1), &fit("FIND", 4), 0.55);
            let chars: Vec<char> = list.query.text.chars().collect();
            if chars.is_empty() {
                line(
                    &mut frame,
                    at(5, 1),
                    &fit("A NAME, MAKER OR TYPE", FIND_CHARS),
                    0.22,
                );
            } else {
                let text: Vec<char> = chars.iter().skip(first).take(FIND_CHARS).copied().collect();
                line(&mut frame, at(5, 1), &text, 1.0);
            }
        }
        underline(
            &mut frame,
            &mut cover,
            s,
            at(5 + list.query.caret - first, 1),
            1,
            1.0,
        );
        for order in FxOrder::ALL {
            let on = list.order == order;
            let pos = at(order_col(order), 1);
            text_dots(
                &mut frame,
                &mut cover,
                s,
                (pos.0, pos.1, LIST_DOT),
                &fit(order.word(), order.word().len()),
                if on { 1.0 } else { 0.45 },
            );
            if on {
                underline(&mut frame, &mut cover, s, pos, order.word().len(), 1.0);
            }
        }
        for k in 0..LIST_ROWS {
            let Some(row) = list.rows.get(list.first + k) else {
                break;
            };
            let index = list.first + k;
            let pointed = list.hover == Some(index) || list.chosen == Some(index);
            let level = if pointed { 1.0 } else { 0.72 };
            let mut draw = |col, text: &str, width, level, query: &str| {
                matched(
                    &mut frame,
                    &mut cover,
                    s,
                    at(col, k + 2),
                    text,
                    width,
                    level,
                    query,
                );
            };
            if let Some(count) = row.count {
                let name: String = fit(&row.name.to_uppercase(), LIST_CHARS - 8)
                    .into_iter()
                    .collect();
                let n = name.chars().count();
                draw(0, &name, n, 0.9, "");
                draw(
                    n + 1,
                    &"·".repeat(LIST_CHARS.saturating_sub(n + 8)),
                    LIST_CHARS - n - 1,
                    0.22,
                    "",
                );
                let count = count.to_string();
                draw(LIST_CHARS - 3 - count.len(), &count, count.len(), 0.55, "");
                continue;
            }
            if list.chosen.or(list.hover) == Some(index) && !row.name.is_empty() {
                draw(COLS.0, ">", 1, 1.0, "");
            }
            draw(COLS.1, &row.name, NAME_CHARS, level, &list.query.text);
            if !row.action {
                if list.order != FxOrder::Maker {
                    draw(
                        COLS.2,
                        &row.maker,
                        MAKER_CHARS,
                        level * 0.75,
                        &list.query.text,
                    );
                }
                if list.order != FxOrder::Type {
                    let col = if list.order == FxOrder::Maker {
                        COLS.2
                    } else {
                        COLS.3
                    };
                    draw(col, &row.kind, TYPE_CHARS, level * 0.75, &list.query.text);
                }
                draw(COLS.4, &row.format, FORMAT_CHARS, level * 0.6, "");
            }
        }
        // More above or below: the presets' list's arrows, at the lines' right end.
        let more = [
            (list.first > 0, 2, crate::presets::UP),
            (
                list.first + LIST_ROWS < list.rows.len(),
                LIST_ROWS + 1,
                crate::presets::DOWN,
            ),
        ];
        for (show, k, sign) in more {
            if !show {
                continue;
            }
            let (sx, sy) = at(LIST_CHARS - 1, k);
            let mut core = Vec::new();
            for (j, bits) in sign.iter().enumerate() {
                for col in 0..5 {
                    if bits >> (4 - col) & 1 == 1 {
                        core.push((
                            ((sx + col as f64 * LIST_DOT) * s) as f32,
                            ((sy + (j + 1) as f64 * LIST_DOT) * s) as f32,
                            (LIST_DOT * 0.36 * s) as f32,
                        ));
                    }
                }
            }
            discs(&mut frame, &mut cover, &core, ORANGE, 0.6);
        }
        self.list = Some(ListFrame {
            shown: list.clone(),
            plate,
            frame,
            left,
            top,
        });
    }

    /// The list as last drawn (the tests').
    pub fn list_shown(&self) -> Option<&FxList> {
        self.list.as_ref().map(|l| &l.shown)
    }
}

/// Whether slot `i` shows differently in `now` than in `was`.
fn slot_changed(was: &FxScene, now: &FxScene, i: usize) -> bool {
    let pointed = |s: &FxScene| match s.hover {
        Some(FxTarget::Name(j) | FxTarget::Tab(j)) if j == i && s.drag.is_none() => s.hover,
        _ => None,
    };
    let dragged = |s: &FxScene| s.drag.map(|(from, over)| (from == i, over == Some(i)));
    was.slots[i] != now.slots[i] || pointed(was) != pointed(now) || dragged(was) != dragged(now)
}

/// A screen's (left, top, width, height, its frame's units) dots' first column's left and first
/// row's top: `PLACES` characters centred across, a capital's middle `CAP_LIFT` above the middle
/// (the rail's name's).
fn screen_grid((x, y, w, h): (f64, f64, f64, f64)) -> (f64, f64) {
    (
        x + (w - (6 * PLACES - 1) as f64 * DOT) / 2.0,
        y + h / 2.0 - CAP_LIFT - 3.0 * DOT,
    )
}

/// `text` in `places` characters, centred: those that fit, the last an ellipsis where it is cut.
fn centred(text: &str, places: usize) -> Vec<char> {
    let chars = fit(text, places);
    let mut out = vec![' '; (places - chars.len()) / 2];
    out.extend(chars);
    out
}

/// `text` cut to `n` characters (its last an ellipsis where it is cut; control characters
/// dropped).
fn fit(text: &str, n: usize) -> Vec<char> {
    let mut chars: Vec<char> = text.chars().filter(|c| !c.is_control()).collect();
    if chars.len() > n {
        chars.truncate(n.saturating_sub(1));
        chars.push('\u{2026}');
    }
    chars
}

/// `chars` lit in dots over `frame` from (`left`, `top`) (its units) at `pitch`: their glow, then
/// their dots, at `level` (1 bright).
fn text_dots(
    frame: &mut Pixmap,
    cover: &mut [u8],
    s: f64,
    (left, top, pitch): (f64, f64, f64),
    chars: &[char],
    level: f64,
) {
    let r = (pitch * 0.36 * s) as f32;
    let mut core: Vec<Disc> = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        if *c == ' ' {
            continue;
        }
        for (j, bits) in font::glyph(*c).iter().enumerate() {
            for col in 0..5 {
                if bits >> (4 - col) & 1 == 1 {
                    core.push((
                        ((left + (6 * i + col) as f64 * pitch) * s) as f32,
                        ((top + j as f64 * pitch) * s) as f32,
                        r,
                    ));
                }
            }
        }
    }
    if core.is_empty() {
        return;
    }
    let glow: Vec<Disc> = core.iter().map(|&(x, y, r)| (x, y, r * 2.1)).collect();
    discs(frame, cover, &glow, ORANGE_GLOW, (0.16 * level) as f32);
    discs(frame, cover, &core, ORANGE, (0.25 + 0.75 * level) as f32);
}

const FIND_CHARS: usize = 28;

fn underline(
    frame: &mut Pixmap,
    cover: &mut [u8],
    s: f64,
    (x, y): (f64, f64),
    n: usize,
    level: f64,
) {
    let dots: Vec<Disc> = (0..6 * n - 1)
        .map(|c| {
            (
                ((x + c as f64 * LIST_DOT) * s) as f32,
                ((y + 8.0 * LIST_DOT) * s) as f32,
                (LIST_DOT * 0.36 * s) as f32,
            )
        })
        .collect();
    discs(frame, cover, &dots, ORANGE, (0.25 + 0.75 * level) as f32);
}

/// Lit, underlined matches at character boundaries, including non-ASCII case folding.
#[allow(clippy::too_many_arguments)]
fn matched(
    frame: &mut Pixmap,
    cover: &mut [u8],
    s: f64,
    (x, y): (f64, f64),
    text: &str,
    width: usize,
    level: f64,
    query: &str,
) {
    let chars = fit(text, width);
    let mut lit = vec![false; chars.len()];
    let mut lower = String::new();
    let mut owners = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        let folded = c.to_lowercase().collect::<String>();
        owners.extend(std::iter::repeat_n(i, folded.len()));
        lower.push_str(&folded);
    }
    for word in query.split_whitespace().map(str::to_lowercase) {
        for (at, _) in lower
            .char_indices()
            .filter(|(at, _)| lower[*at..].starts_with(&word))
        {
            for &i in &owners[at..at + word.len()] {
                lit[i] = true;
            }
        }
    }
    let mut first = 0;
    while first < chars.len() {
        let mut end = first + 1;
        while end < chars.len() && lit[end] == lit[first] {
            end += 1;
        }
        let pos = (x + (6 * first) as f64 * LIST_DOT, y);
        text_dots(
            frame,
            cover,
            s,
            (pos.0, pos.1, LIST_DOT),
            &chars[first..end],
            if lit[first] { 1.0 } else { level },
        );
        if lit[first] {
            underline(frame, cover, s, pos, end - first, 1.0);
        }
        first = end;
    }
}

/// Every dot of `places` characters on each line (`lines`: each line's top's offset) from
/// (`left`, `top`) at `pitch`, nine rows a line, faintly there.
fn unlit_dots(
    frame: &mut Pixmap,
    cover: &mut [u8],
    s: f64,
    (left, top, pitch): (f64, f64, f64),
    places: usize,
    lines: &[f64],
) {
    let r = (pitch * 0.36 * s) as f32;
    let mut at: Vec<Disc> = Vec::new();
    for &dy in lines {
        for i in 0..places {
            for j in 0..9 {
                for col in 0..5 {
                    at.push((
                        ((left + (6 * i + col) as f64 * pitch) * s) as f32,
                        ((top + dy + j as f64 * pitch) * s) as f32,
                        r,
                    ));
                }
            }
        }
    }
    discs(frame, cover, &at, ORANGE, 0.06);
}

/// What never changes for `look`: the plate, the legends worn into it, the screens' bezels, glass
/// and unlit dots, the unlit tabs; the tabs' words; each slot's small print's words; the shadow.
fn make_still((scale, skin, lettering): Look) -> Still {
    let s = f64::from_bits(scale);
    let options = lower::lettered();
    let oy = f64::from(top_px(s)) / s;
    let mut plate = lower::plate(s, skin, DEPTH, &options);
    // The legends, over each slot's screen and tab.
    let mut print = Svg(String::new(), lettering);
    for i in 0..SLOTS {
        put!(
            print,
            "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' font-weight='700' text-anchor='middle' dominant-baseline='central' fill='{}'>SLOT {}</text>",
            N(middle(i)),
            N(LEGEND_Y - oy),
            N(lettering.strip_legend()),
            colour::LEGEND,
            i + 1
        );
    }
    lower::print(&mut plate, s, skin, &options, &print.0);
    // The screens: the rail's name's bezel and glass, every dot faintly there.
    let glass_picture = Pixmap::decode_png(crate::skin::GLASS).expect("a built-in picture decodes");
    let mut bezels = Svg::default();
    for i in 0..SLOTS {
        let (x, y, w, h) = screen(i);
        super::screen_bezel(&mut bezels, (x, y - oy, w, h));
    }
    svg_over(&options, &mut plate, s, &bezels.0);
    let mut cover = vec![0; (plate.width() * plate.height()) as usize];
    for i in 0..SLOTS {
        let (x, y, w, h) = screen(i);
        glass(&mut plate, &glass_picture, s, (x, y - oy, w, h), 3.0);
        let (left, top) = screen_grid((x, y - oy, w, h));
        unlit_dots(&mut plate, &mut cover, s, (left, top, DOT), PLACES, &[0.0]);
    }
    let lighting = lower::tab_lighting(TAB);
    for i in 0..SLOTS {
        let (x, y) = tab(i);
        sprite(&mut plate, &lighting.unlit, s, (x, y - oy), TAB);
    }
    lower::lamp(&mut plate, s, skin, (0, top_px(s)));
    let tabs: Vec<((f64, f64), &str)> = (0..SLOTS).map(|i| (tab(i), "ON")).collect();
    let words = lower::tab_words(&options, s, oy, TAB, &tabs);
    // The small print under each screen, each word on a patch of its own.
    let small = (0..SLOTS)
        .map(|i| {
            let (x, _, w, _) = screen(i);
            FxWord::ALL
                .iter()
                .filter_map(|word| {
                    let (pw, ph) = ((w * s).ceil() as u32, (28.0 * s).ceil().max(1.0) as u32);
                    let (left, ptop) = ((x * s).floor(), ((WORD_Y - 14.0 - oy) * s).floor());
                    let mut p = Pixmap::new(pw.max(1), ph)?;
                    let mut o = Svg::default();
                    put!(
                        o,
                        "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' font-weight='700' letter-spacing='1.5' text-anchor='middle' dominant-baseline='central' fill='{}' fill-opacity='0.72'>{}</text>",
                        N(x + w / 2.0 - left / s),
                        N(WORD_Y - oy - ptop / s),
                        N(lettering.strip_small()),
                        colour::LEGEND,
                        word.text()
                    );
                    svg_over(&options, &mut p, s, &o.0);
                    Some((left as i32, ptop as i32, p))
                })
                .collect()
        })
        .collect();
    Still {
        look: (scale, skin, lettering),
        plate,
        words,
        small,
        shadow: lower::cast(s, &options),
        lighting,
        list: list_plate(s, skin, &glass_picture),
    }
}

/// The list's plate at `s`: its face, its edges, its display's bezel, glass and unlit dots, its
/// shadow below and to its right; its top left corner its pixels' (the lamp's light is laid over
/// it where it drops).
fn list_plate(s: f64, skin: Skin, glass_picture: &Pixmap) -> Pixmap {
    let (w, h) = (LIST_W, LIST_H);
    let (pw, ph) = (
        ((w + SHADOW) * s).ceil() as u32 + 2,
        ((h + SHADOW) * s).ceil() as u32 + 2,
    );
    let mut p = Pixmap::new(pw, ph).expect("a list of at least a pixel");
    let options = usvg::Options::default();
    // Its shadow, down and to the right, softest furthest from it (gradients, not a blur: a
    // blur over the whole list cost most of its drop).
    let mut o = Svg::default();
    put!(
        o,
        "<defs><linearGradient id='list-down' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.55'/><stop offset='0.35' stop-color='#000' stop-opacity='0.28'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient><linearGradient id='list-right' x1='0' y1='0' x2='1' y2='0'><stop offset='0' stop-color='#000' stop-opacity='0.45'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient></defs><rect x='4' y='{}' width='{}' height='{}' fill='url(#list-down)'/><rect x='{}' y='6' width='10' height='{}' fill='url(#list-right)'/>",
        N(h),
        N(w),
        N(SHADOW),
        N(w),
        N(h - 6.0)
    );
    svg_over(&options, &mut p, s, &o.0);
    // Its face: the drawer's, from another place; edges as the drawer's.
    let mut face = Pixmap::new(pw, ph).expect("a face");
    let (fw, fh) = ((w * s).round(), (h * s).round());
    if skin == Skin::Worn {
        let picture = crate::skin::face_picture();
        let k = (640.0 * s) as f32 / picture.width() as f32;
        crate::render::fill(
            &mut face,
            &picture,
            (0.0, 0.0, fw, fh),
            None,
            Transform::from_scale(k, k).post_translate((-130.0 * s) as f32, (-310.0 * s) as f32),
        );
    } else {
        let mut o = Svg::default();
        put!(
            o,
            "<rect x='0' y='0' width='{}' height='{}' fill='{}'/>",
            N(w),
            N(h),
            colour::PANEL
        );
        svg_over(&options, &mut face, s, &o.0);
    }
    // The edges; the display in a bezel as the strip's readouts are (the rail's name's, its
    // shadow blurred all round, cost half the list's drop).
    let (dx, dy, dw, dh) = DISPLAY;
    let mut o = Svg::default();
    put!(
        o,
        "<defs><linearGradient id='list-top' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.55'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient><linearGradient id='list-left' x1='0' y1='0' x2='1' y2='0'><stop offset='0' stop-color='#ffe6c8' stop-opacity='0.14'/><stop offset='1' stop-color='#ffe6c8' stop-opacity='0'/></linearGradient></defs>"
    );
    put!(
        o,
        "<rect x='0' y='0' width='{}' height='12' fill='url(#list-top)'/><rect x='0' y='8' width='5' height='{}' fill='url(#list-left)'/><rect x='0' y='{}' width='{}' height='1.5' fill='#000' fill-opacity='0.55'/><rect x='{}' y='0' width='1.5' height='{}' fill='#000' fill-opacity='0.45'/>",
        N(w),
        N(h - 8.0),
        N(h - 1.5),
        N(w),
        N(w - 1.5),
        N(h)
    );
    put!(
        o,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#0b0a09'/><rect x='{}' y='{}' width='{}' height='{}' rx='7.5' fill='none' stroke='#fff' stroke-opacity='0.07' stroke-width='1.5'/>",
        N(dx - 6.0),
        N(dy - 6.0),
        N(dw + 12.0),
        N(dh + 12.0),
        N(dx - 5.0),
        N(dy - 5.0),
        N(dw + 10.0),
        N(dh + 10.0)
    );
    svg_over(&options, &mut face, s, &o.0);
    // (The presets' list's glass, smoothed: drawn out over a display many times its size, the
    // picture's faint lines are bands across it.)
    crate::presets::display_glass(
        &mut face,
        &crate::presets::smooth_glass(glass_picture),
        s,
        (dx, dy, dw, dh),
    );
    // Every dot faintly there.
    let lines: Vec<f64> = (0..LIST_ROWS + 2).map(|k| LINE * k as f64).collect();
    let mut cover = vec![0; (pw * ph) as usize];
    unlit_dots(
        &mut face,
        &mut cover,
        s,
        (
            dx + PAD,
            dy + PAD + LINE / 2.0 - CAP_LIFT - 3.0 * LIST_DOT,
            LIST_DOT,
        ),
        LIST_CHARS,
        &lines,
    );
    // The face where the plate is, over its shadow.
    let (x1, y1) = ((fw as u32).min(pw), (fh as u32).min(ph));
    let n = (x1 * 4) as usize;
    for py in 0..y1 {
        let i = (py * pw * 4) as usize;
        p.data_mut()[i..i + n].copy_from_slice(&face.data()[i..i + n]);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(slot: usize, rows: usize) -> FxList {
        FxList {
            slot,
            title: "ADD TO SLOT".into(),
            note: String::new(),
            rows: (0..rows)
                .map(|i| FxRow {
                    name: format!("Effect {i}"),
                    maker: "Maker".into(),
                    format: "CLAP".into(),
                    action: false,
                    ..FxRow::default()
                })
                .collect(),
            first: 0,
            hover: None,
            ..FxList::default()
        }
    }

    /// Each slot's screen and tab are found where they are drawn, inside the drawer, every tab
    /// with room beside its screen, the slots apart from each other, the legends, the row and the
    /// small print inside the drawer under the rail's shadow; the list finds its rows where it
    /// draws them, from the first in view, within the drawer's ends, below its foot, inside the
    /// window.
    #[test]
    fn the_drawer_finds_each_part_where_it_is_drawn() {
        for i in 0..SLOTS {
            for t in [FxTarget::Name(i), FxTarget::Tab(i)] {
                let (x, y) = centre(t);
                assert_eq!(hit(x, y, None), Some(t));
                assert!(inside(x, y));
            }
            let (x, _, w, _) = screen(i);
            let (tx, _) = tab(i);
            assert!(x - SURROUND > LEFT && tx + TAB.0 / 2.0 < RIGHT);
            assert!(tx - TAB.0 / 2.0 > x + w + SURROUND, "the tab has room");
            if i + 1 < SLOTS {
                assert!(tx + TAB.0 / 2.0 + 42.0 < screen(i + 1).0 - SURROUND - 8.0);
            }
        }
        assert_eq!(hit(LEFT + 10.0, ROW, None), None);
        assert!(!inside(1000.0, TOP + DEPTH + 1.0) && !inside(1000.0, TOP - 1.0));
        let (_, y, _, h) = screen(0);
        assert!(LEGEND_Y - 12.0 > TOP + 14.0 && y - SURROUND > LEGEND_Y + 12.0);
        assert!(y + h + SURROUND < WORD_Y - 8.0 && WORD_Y + 8.0 < TOP + DEPTH - 9.0);
        for slot in 0..SLOTS {
            let mut l = list(slot, 20);
            l.first = 3;
            let (x, y, w, h) = list_rect(&l);
            assert!(x >= LEFT && x + w <= RIGHT && y >= TOP + DEPTH);
            assert!(y + h < crate::art::H, "inside the window");
            for row in 3..3 + LIST_ROWS {
                let (rx, ry) = row_centre(&l, row);
                assert_eq!(hit(rx, ry, Some(&l)), Some(FxTarget::Row(row)));
            }
            assert_eq!(
                hit(x + 30.0, y + MARGIN + 2.0, Some(&l)),
                Some(FxTarget::List)
            );
            let short = list(slot, 2);
            let (rx, ry) = row_centre(&short, 1);
            assert_eq!(hit(rx, ry, Some(&short)), Some(FxTarget::Row(1)));
            assert_eq!(
                hit(rx, ry + LINE, Some(&short)),
                Some(FxTarget::List),
                "below its last row"
            );
        }
        // Every column inside the line, the arrows' place apart.
        assert!(COLS.4 + FORMAT_CHARS < LIST_CHARS - 1);
        assert!(
            COLS.1 + NAME_CHARS < COLS.2
                && COLS.2 + MAKER_CHARS < COLS.3
                && COLS.3 + TYPE_CHARS < COLS.4
        );
    }

    #[test]
    fn find_and_group_words_are_found_and_heads_cannot_be_chosen() {
        let mut l = list(3, 12);
        let (x, y, w, h) = list_rect(&l);
        assert_eq!((w, h), (1480.0, 540.0));
        for order in FxOrder::ALL {
            let px = x + DISPLAY.0 + PAD + (order_col(order) as f64 + 0.5) * 6.0 * LIST_DOT;
            assert_eq!(
                hit(px, y + line_y(1), Some(&l)),
                Some(FxTarget::Order(order))
            );
        }
        let px = x + DISPLAY.0 + PAD + 7.0 * 6.0 * LIST_DOT;
        assert_eq!(hit(px, y + line_y(1), Some(&l)), Some(FxTarget::Find(2)));
        l.rows[0].count = Some(10);
        let (rx, ry) = row_centre(&l, 0);
        assert_eq!(hit(rx, ry, Some(&l)), Some(FxTarget::List));
        l.rows[1].name.clear();
        let (rx, ry) = row_centre(&l, 1);
        assert_eq!(hit(rx, ry, Some(&l)), Some(FxTarget::List));
    }

    #[test]
    fn all_occurrences_of_each_word_are_underlined_at_character_boundaries() {
        let mut p = Pixmap::new(600, 60).expect("pixels");
        let mut cover = vec![0; 600 * 60];
        matched(
            &mut p,
            &mut cover,
            1.0,
            (4.0, 4.0),
            "Écho Echo",
            24,
            0.4,
            "ÉC ec",
        );
        // The underline is eight dot rows below the letters, with a gap at the space.
        let alpha = |col: usize| p.pixel((4 + col * 24) as u32, 36).expect("dot").alpha();
        for col in [0, 1, 5, 6] {
            assert!(alpha(col) > 0, "matched character {col}");
        }
        for col in [2, 3, 4, 7, 8] {
            assert_eq!(alpha(col), 0, "unmatched character {col}");
        }
        let mut p = Pixmap::new(240, 60).expect("pixels");
        let mut cover = vec![0; 240 * 60];
        matched(&mut p, &mut cover, 1.0, (4.0, 4.0), "Banana", 6, 0.4, "ana");
        for col in 1..6 {
            assert!(p.pixel(4 + col * 24, 36).expect("underline").alpha() > 0);
        }
    }

    /// A name longer than the screen's twelve places ends in an ellipsis; a short one is centred.
    #[test]
    fn a_name_is_cut_to_the_screen_and_centred() {
        let long = centred("Dragonfly Hall Reverb", PLACES);
        assert_eq!(long.len(), PLACES);
        assert_eq!(long[PLACES - 1], '\u{2026}');
        let short = centred("OTT", PLACES);
        assert_eq!(short.iter().filter(|c| **c == ' ').count(), 4);
        assert_eq!(short.iter().collect::<String>().trim(), "OTT");
        assert_eq!(centred("+ ADD EFFECT", PLACES).len(), PLACES);
    }

    fn scene() -> FxScene {
        let slot = |t: &str, lit: bool| FxSlot {
            text: t.into(),
            dim: false,
            lit,
            word: None,
        };
        FxScene {
            slots: [
                slot("Chorus", true),
                slot("Hall Reverb", true),
                slot("+ ADD EFFECT", false),
                FxSlot::default(),
            ],
            ..FxScene::default()
        }
    }

    /// The drawer drawn in parts is the drawer drawn whole: a tab let go, a slot crashed, the
    /// pointer over a name and a tab, a slot dragged over another, the list opened, scrolled and
    /// pointed at and put away, at the window's opening scale and a Retina screen's, in both
    /// skins.
    #[test]
    fn drawn_in_parts_it_is_drawn_whole() {
        for (scale, skin) in [(0.44, Skin::Worn), (0.87, Skin::Drawn)] {
            let mut r = FxRenderer::new(scale, skin, Lettering::Standard);
            let mut scene = scene();
            assert!(r.render(&scene));
            let steps: [&dyn Fn(&mut FxScene); 11] = [
                &|s| s.slots[1].lit = false,
                &|s| {
                    s.slots[0].dim = true;
                    s.slots[0].lit = false;
                    s.slots[0].word = Some(FxWord::Crashed);
                },
                &|s| s.hover = Some(FxTarget::Name(1)),
                &|s| s.hover = Some(FxTarget::Tab(2)),
                &|s| s.drag = Some((0, Some(1))),
                &|s| {
                    s.drag = None;
                    s.list = Some(list(2, 14));
                },
                &|s| {
                    if let Some(l) = &mut s.list {
                        l.first = 2;
                        l.hover = Some(4);
                    }
                },
                &|s| {
                    let l = s.list.as_mut().expect("list");
                    l.query = crate::presets::Field::new("Ef 4");
                    l.chosen = Some(4);
                },
                &|s| {
                    let l = s.list.as_mut().expect("list");
                    l.order = FxOrder::Maker;
                    l.rows[2].count = Some(10);
                },
                &|s| {
                    let l = s.list.as_mut().expect("list");
                    l.query = crate::presets::Field::new(
                        "a very long query to scroll beyond the visible field é",
                    );
                    l.order = FxOrder::Type;
                },
                &|s| s.list = None,
            ];
            let under = |r: &FxRenderer| {
                r.under()
                    .into_iter()
                    .map(|(p, x, y)| (p.clone(), x, y))
                    .collect::<Vec<_>>()
            };
            for step in steps {
                step(&mut scene);
                assert!(r.render(&scene), "{scene:?}");
                assert!(r.damage().is_some());
                let mut whole = FxRenderer::new(scale, skin, Lettering::Standard);
                whole.render(&scene);
                assert!(
                    r.frame() == whole.frame(),
                    "{scale} {skin:?}: {scene:?} drawn in parts"
                );
                assert!(under(&r) == under(&whole), "the list drawn in parts");
            }
            assert!(!r.render(&scene), "nothing changed");
        }
    }
}
