//! The presets' selector at the left of the strip and their drawer (decisions.md R10, R12;
//! the owner, 2026-10-02, asked for the presets to drop down from the bottom, in a drawer):
//! the selector names the preset the plug-in was last set to (marked • once changed), with
//! its favourite star, the previous and next preset and SAVE…, in the strip's row (the
//! owner, 2026-10-02, asked for it on the same row). The drawer (`drawer`: the mock-up's, A6)
//! drops down from under the rail over the strip's sections: FIND, the search and the tags in
//! use as filters; PRESETS, ALL, FAVORITES and MINE over the list (each row's star, its name,
//! EDITED or YOURS, its tags); PRESET, its keys (RENAME, TAGS, DELETE and REVERT, for the preset
//! the plug-in is set to; SAVE AS, the current sound saved as a preset, named and tagged;
//! RESTORE, the factory's presets deleted brought back; MIDI LEARN; CLOSE) over a display of
//! that preset, of what a key is doing and of the update check (R27). The selector is drawn
//! [`BAR_END`] panel units across the strip's row (the strip draws it: [`bar_svg`]), the
//! drawer [`DRAWER_H`] tall. What a pointer finds is [`bar_hit`] and [`drawer_hit`]; the
//! drawing and the finding share one layout.
//!
//! MIDI LEARN shows MIDI Learn's list in the list's place (the CA-72's `docs/decisions.md`
//! R34): every control that can be learned, its controller, LEARN, REMOVE and CANCEL, operated
//! from the keyboard as well ([`MidiList`]).

use crate::art;
use crate::fonts::{Fonts, Weight};
use crate::svg::{N, Svg, colour, escape, put};

mod drawer;
pub use drawer::{
    DrawerRenderer, PresetKey, ROW_H, ROWS_SHOWN, TAG_LINES, Tab, caret_at, chip_centre,
    drawer_hit, field_centre, in_tags, key_centre, midi_action_centre, midi_centre,
    midi_row_centre, row_centre, search_centre, star_centre, tab_centre, tab_of, update_centre,
    update_fits,
};

/// The selector's height, panel units: the strip's row it shares.
pub const BAR_H: f64 = 96.0;
/// Where the selector ends across the strip's row, panel units: the strip's own controls
/// are to the right of it.
pub const BAR_END: f64 = 1110.0;
/// The drawer's top, panel units down from the drawing's, and its height: dropping down from
/// under the strip's rail over its sections, to the window's foot (A6: decisions.md R-LOOK).
pub const DRAWER_TOP: f64 = art::PANEL_H + crate::strip::RAIL;
pub const DRAWER_H: f64 = art::H - DRAWER_TOP;

const DIM: &str = "#8a909c";
const RAISED: &str = "#272b32";
const BORDER: &str = "#4a4640";
const ACCENT: &str = "#f0a030";

/// The bar's text size (the strip's).
const BAR_TEXT: f64 = 26.0;

/// Signs in dots (five across, seven down, as the dot displays' capitals: the mock-up's
/// glyphs): a favourite's star, the star of a preset that is not one, an arrow down and up.
pub(crate) const STAR: [u8; 7] = [0x04, 0x04, 0x1f, 0x0e, 0x0e, 0x1b, 0x11];
pub(crate) const HOLLOW: [u8; 7] = [0x04, 0x04, 0x1b, 0x0a, 0x0a, 0x15, 0x11];
pub(crate) const DOWN: [u8; 7] = [0x00, 0x1f, 0x1f, 0x0e, 0x0e, 0x04, 0x00];
pub(crate) const UP: [u8; 7] = [0x00, 0x04, 0x0e, 0x0e, 0x1f, 0x1f, 0x00];

/// A text field: its text and the caret, a count of characters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Field {
    pub text: String,
    pub caret: usize,
}

impl Field {
    pub fn new(text: &str) -> Self {
        Field {
            text: text.to_owned(),
            caret: text.chars().count(),
        }
    }

    fn byte(&self, chars: usize) -> usize {
        self.text
            .char_indices()
            .nth(chars)
            .map_or(self.text.len(), |(i, _)| i)
    }

    /// Typed at the caret.
    pub fn insert(&mut self, s: &str) {
        let at = self.byte(self.caret);
        let clean: String = s.chars().filter(|c| !c.is_control()).collect();
        self.text.insert_str(at, &clean);
        self.caret += clean.chars().count();
    }

    pub fn backspace(&mut self) {
        if self.caret > 0 {
            let (a, b) = (self.byte(self.caret - 1), self.byte(self.caret));
            self.text.replace_range(a..b, "");
            self.caret -= 1;
        }
    }

    pub fn delete(&mut self) {
        if self.caret < self.text.chars().count() {
            let (a, b) = (self.byte(self.caret), self.byte(self.caret + 1));
            self.text.replace_range(a..b, "");
        }
    }

    pub fn left(&mut self) {
        self.caret = self.caret.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.caret = (self.caret + 1).min(self.text.chars().count());
    }

    pub fn home(&mut self) {
        self.caret = 0;
    }

    pub fn end(&mut self) {
        self.caret = self.text.chars().count();
    }
}

/// The drawer's text fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldId {
    Search,
    /// The preset's new name or tags, typed.
    Edit,
    SaveName,
    SaveTags,
}

/// What a PRESET key began, going on: the preset the plug-in is set to renamed, tagged or
/// asked about deleting; the sound being saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
    Rename,
    Tags,
    /// Asked before deleting (a factory preset's: hidden, RESTORE brings it back).
    Delete {
        factory: bool,
    },
    Save,
}

/// What a pointer finds on the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarTarget {
    Star,
    Prev,
    Name,
    Next,
    Save,
}

/// What a pointer finds in the drawer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawerTarget {
    Field(FieldId),
    Tab(Tab),
    /// A tag (by its index), a filter on or off.
    Chip(usize),
    /// A row (by its index in the list), its star.
    Row(usize),
    Star(usize),
    Key(PresetKey),
    /// The update check's button.
    Update,
    /// A row of the MIDI list (by its index), one of its words that act.
    MidiRow(usize),
    MidiAction(usize, MidiAction),
    /// The drawer, where nothing else is.
    Back,
}

/// What a word of a row of the MIDI list does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiAction {
    /// Wait for the next controller moved, and assign it.
    Learn,
    /// Stop waiting.
    Cancel,
    /// Its controller removed.
    Remove,
}

/// A row of the MIDI list: a control's name, its controller as shown (empty: none), and whether
/// it is waiting for one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiRow {
    pub name: String,
    pub assignment: String,
    pub waiting: bool,
}

/// The drawer's MIDI Learn list (decisions.md R34): its rows, the first shown, the one chosen
/// (the arrow keys move it; Enter learns it, Delete removes its controller), the keys and the
/// controllers it will not learn, and what was last done, in its colour.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MidiList {
    pub rows: Vec<MidiRow>,
    pub first: usize,
    pub chosen: usize,
    pub keys: String,
    pub reserved: String,
    pub status: String,
    pub tone: Tone,
}

/// How the update check's words are said.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    Dim,
    /// A newer release.
    News,
    /// Something went wrong.
    Trouble,
}

/// The update check, at the PRESET display's foot: what it says, and its button's words (none
/// when empty).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UpdateScene {
    pub text: String,
    pub tone: Tone,
    pub button: String,
}

/// A row of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub tags: String,
    /// `EDITED` (a factory preset the user's version stands in for) or `YOURS`.
    pub origin: Option<&'static str>,
    pub favorite: bool,
    /// The preset the plug-in was last set to.
    pub current: bool,
}

/// The preset the plug-in is set to, as the PRESET display shows it.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Current {
    pub name: String,
    /// As a row's.
    pub origin: Option<&'static str>,
    pub tags: String,
    pub favorite: bool,
}

/// What the drawer shows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DrawerScene {
    pub search: Field,
    /// The list's tab: FAVORITES, MINE (neither: ALL).
    pub favourites: bool,
    pub mine: bool,
    /// The tags in use, each picked as a filter or not, and the first line of them shown.
    pub chips: Vec<(String, bool)>,
    pub tags_first: usize,
    pub rows: Vec<Row>,
    /// The first row shown.
    pub first: usize,
    /// The preset the plug-in is set to, if it is in the library; how many there are.
    pub current: Option<Current>,
    pub total: usize,
    /// What a PRESET key began, going on.
    pub editing: Option<Edit>,
    pub edit: Field,
    pub save_name: Field,
    pub save_tags: Field,
    /// Saving replaces a preset of the name typed.
    pub replace: bool,
    /// What was last done, or why not.
    pub hint: String,
    pub focus: Option<FieldId>,
    pub hover: Option<DrawerTarget>,
    pub update: UpdateScene,
    /// The MIDI Learn list, shown in the presets' place (none: the presets).
    pub midi: Option<MidiList>,
}

/// What the bar shows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BarScene {
    /// The preset's name ("NO PRESET" when none).
    pub name: String,
    /// The preset is in the library (else the name is the last one set, gone since).
    pub found: bool,
    /// Its values changed since it was set.
    pub changed: bool,
    pub favorite: bool,
    pub open: bool,
    /// The drawer opens below the bar (else over the panel): which way the arrow points.
    pub below: bool,
    pub hover: Option<BarTarget>,
}

// ---- The selector's layout (panel units across, the strip's units down).

const BAR_MID: f64 = BAR_H / 2.0;
const BTN_H: f64 = 46.0;
const STAR_X: f64 = 40.0;
const STEP_W: f64 = 54.0;
const PREV_X: f64 = 110.0;
const NAME_X: f64 = 180.0;
const NAME_END: f64 = 860.0;
const NEXT_X: f64 = 876.0;
const SAVE_X: f64 = 946.0;
const SAVE_W: f64 = 150.0;

/// What is at (`x`, `y`) on the bar.
pub fn bar_hit(x: f64, y: f64) -> Option<BarTarget> {
    if (y - BAR_MID).abs() > BTN_H / 2.0 + 6.0 {
        return None;
    }
    let within = |x0: f64, w: f64| (x0..x0 + w).contains(&x);
    if within(STAR_X, STEP_W) {
        Some(BarTarget::Star)
    } else if within(PREV_X, STEP_W) {
        Some(BarTarget::Prev)
    } else if within(NAME_X, NAME_END - NAME_X) {
        Some(BarTarget::Name)
    } else if within(NEXT_X, STEP_W) {
        Some(BarTarget::Next)
    } else if within(SAVE_X, SAVE_W) {
        Some(BarTarget::Save)
    } else {
        None
    }
}

fn button(
    out: &mut Svg,
    (x, y, w, h): (f64, f64, f64, f64),
    label: &str,
    size: f64,
    on: bool,
    hover: bool,
    fill: &str,
) {
    put!(
        out,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='7' fill='{RAISED}' stroke='{}' stroke-width='{}'/>",
        N(x),
        N(y),
        N(w),
        N(h),
        if on { ACCENT } else { BORDER },
        if hover { 4 } else { 2 }
    );
    put!(
        out,
        "<text x='{}' y='{}' font-size='{}' fill='{fill}' text-anchor='middle' letter-spacing='1'>{}</text>",
        N(x + w / 2.0),
        N(y + h / 2.0 + size * 0.36),
        N(size),
        escape(label)
    );
}

fn text(out: &mut Svg, x: f64, y: f64, s: &str, size: f64, fill: &str, anchor: &str) {
    put!(
        out,
        "<text x='{}' y='{}' font-size='{}' fill='{fill}' text-anchor='{anchor}' letter-spacing='1'>{}</text>",
        N(x),
        N(y + size * 0.36),
        N(size),
        escape(s)
    );
}

/// A five-pointed star (the panel's typeface has none), filled for a favourite.
fn star(out: &mut Svg, cx: f64, cy: f64, r: f64, filled: bool, fill: &str) {
    let mut d = String::new();
    for k in 0..10 {
        let a = std::f64::consts::PI * (f64::from(k) / 5.0 - 0.5);
        let rr = if k % 2 == 0 { r } else { r * 0.42 };
        let (x, y) = (cx + rr * a.cos(), cy + rr * a.sin());
        d.push_str(&format!(
            "{}{} {} ",
            if k == 0 { "M" } else { "L" },
            N(x),
            N(y)
        ));
    }
    if filled {
        put!(out, "<path d='{d}Z' fill='{fill}'/>");
    } else {
        put!(
            out,
            "<path d='{d}Z' fill='none' stroke='{fill}' stroke-width='2.5' stroke-linejoin='round'/>"
        );
    }
}

/// A small triangle pointing up or down (the typeface has none).
fn triangle(out: &mut Svg, cx: f64, cy: f64, size: f64, up: bool, fill: &str) {
    let (h, w) = (size * 0.6, size * 0.7);
    let (tip, base) = if up {
        (cy - h / 2.0, cy + h / 2.0)
    } else {
        (cy + h / 2.0, cy - h / 2.0)
    };
    put!(
        out,
        "<path d='M {} {} L {} {} L {} {} Z' fill='{fill}'/>",
        N(cx),
        N(tip),
        N(cx - w / 2.0),
        N(base),
        N(cx + w / 2.0),
        N(base)
    );
}

/// The selector, drawn on the strip's row (which draws its background).
pub fn bar_svg(fonts: &Fonts, s: &BarScene) -> String {
    let mut out = Svg::default();
    let y = BAR_MID - BTN_H / 2.0;
    let hover = |t| s.hover == Some(t);
    let star_fill = if !s.found {
        BORDER
    } else if s.favorite {
        ACCENT
    } else {
        DIM
    };
    button(
        &mut out,
        (STAR_X, y, STEP_W, BTN_H),
        "",
        30.0,
        false,
        hover(BarTarget::Star),
        star_fill,
    );
    star(
        &mut out,
        STAR_X + STEP_W / 2.0,
        BAR_MID + 1.0,
        15.0,
        s.favorite,
        star_fill,
    );
    button(
        &mut out,
        (PREV_X, y, STEP_W, BTN_H),
        "‹",
        34.0,
        false,
        hover(BarTarget::Prev),
        colour::LEGEND,
    );
    button(
        &mut out,
        (NEXT_X, y, STEP_W, BTN_H),
        "›",
        34.0,
        false,
        hover(BarTarget::Next),
        colour::LEGEND,
    );
    // The name: left, clipped short of the arrow that opens the drawer.
    put!(
        out,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='7' fill='{RAISED}' stroke='{}' stroke-width='{}'/>",
        N(NAME_X),
        N(y),
        N(NAME_END - NAME_X),
        N(BTN_H),
        if s.open { ACCENT } else { BORDER },
        if hover(BarTarget::Name) { 4 } else { 2 }
    );
    let shown = if s.name.is_empty() {
        "NO PRESET".to_owned()
    } else if s.changed {
        format!("{} •", s.name)
    } else {
        s.name.clone()
    };
    let room = NAME_END - NAME_X - 90.0;
    let shown = fit(fonts, &shown, BAR_TEXT, room);
    let fill = if s.name.is_empty() || !s.found {
        DIM
    } else {
        colour::LEGEND
    };
    text(
        &mut out,
        NAME_X + 20.0,
        BAR_MID,
        &shown,
        BAR_TEXT,
        fill,
        "start",
    );
    // Pointing where the drawer goes: down to open below, up to close it again.
    triangle(
        &mut out,
        NAME_END - 34.0,
        BAR_MID,
        26.0,
        s.open == s.below,
        DIM,
    );
    button(
        &mut out,
        (SAVE_X, y, SAVE_W, BTN_H),
        "SAVE…",
        BAR_TEXT,
        false,
        hover(BarTarget::Save),
        colour::LEGEND,
    );
    out.0
}

/// `s` shortened with an ellipsis to fit `room` at `size`.
fn fit(fonts: &Fonts, s: &str, size: f64, room: f64) -> String {
    let w = |t: &str| fonts.advance(t, size, Weight::Regular, 1.0);
    if w(s) <= room {
        return s.to_owned();
    }
    let mut chars: Vec<char> = s.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let t: String = chars.iter().collect::<String>() + "…";
        if w(&t) <= room {
            return t;
        }
    }
    "…".into()
}

/// Where a bar control's middle is (bar units).
pub fn bar_centre(t: BarTarget) -> (f64, f64) {
    let x = match t {
        BarTarget::Star => STAR_X + STEP_W / 2.0,
        BarTarget::Prev => PREV_X + STEP_W / 2.0,
        BarTarget::Name => (NAME_X + NAME_END) / 2.0,
        BarTarget::Next => NEXT_X + STEP_W / 2.0,
        BarTarget::Save => SAVE_X + SAVE_W / 2.0,
    };
    (x, BAR_MID)
}

#[cfg(test)]
mod tests {
    use super::*;
    use resvg::tiny_skia::Pixmap;

    pub(super) fn scene() -> DrawerScene {
        DrawerScene {
            chips: vec![("bass".into(), false), ("lead".into(), true)],
            rows: (0..30)
                .map(|i| Row {
                    name: format!("Preset {i}"),
                    tags: "bass dark".into(),
                    origin: (i == 2).then_some("EDITED"),
                    favorite: i % 3 == 0,
                    current: i == 1,
                })
                .collect(),
            current: Some(Current {
                name: "Preset 1".into(),
                origin: None,
                tags: "bass dark".into(),
                favorite: false,
            }),
            total: 30,
            ..DrawerScene::default()
        }
    }

    #[test]
    fn a_field_edits_at_its_caret() {
        let mut f = Field::new("Bàss");
        f.left();
        f.backspace();
        assert_eq!((f.text.as_str(), f.caret), ("Bàs", 2));
        f.insert("ü\u{7}");
        assert_eq!((f.text.as_str(), f.caret), ("Bàüs", 3));
        f.home();
        f.delete();
        f.end();
        f.insert("!");
        assert_eq!(f.text, "àüs!");
    }

    #[test]
    fn the_bar_finds_each_control_where_it_is_drawn() {
        assert_eq!(bar_hit(STAR_X + 10.0, BAR_MID), Some(BarTarget::Star));
        assert_eq!(bar_hit(PREV_X + 10.0, BAR_MID), Some(BarTarget::Prev));
        assert_eq!(
            bar_hit((NAME_X + NAME_END) / 2.0, BAR_MID),
            Some(BarTarget::Name)
        );
        assert_eq!(bar_hit(BAR_END + 10.0, BAR_MID), None);
        assert_eq!(bar_hit(NEXT_X + 10.0, BAR_MID), Some(BarTarget::Next));
        assert_eq!(bar_hit(SAVE_X + 10.0, BAR_MID), Some(BarTarget::Save));
        assert_eq!(bar_hit(100.0, BAR_MID), None);
        assert_eq!(bar_hit(1200.0, 2.0), None);
    }

    /// The drawer draws, and draws again only when what it shows changes: not for a pointer
    /// moving over nothing.
    #[test]
    fn it_draws_when_its_scene_changes() {
        let mut d = DrawerRenderer::new(0.4);
        let mut s = scene();
        s.focus = Some(FieldId::Search);
        s.search = Field::new("bass");
        assert!(d.render(&s));
        assert!(!d.render(&s));
        s.first = 3;
        assert!(d.render(&s));
        s.hover = Some(DrawerTarget::Back);
        assert!(!d.render(&s), "nothing under the pointer changes");
        // Something is drawn: the list's rows differ from the background.
        let px = d.frame().pixels();
        let distinct = px
            .iter()
            .map(|p| (p.red(), p.green(), p.blue()))
            .collect::<std::collections::BTreeSet<_>>();
        assert!(distinct.len() > 20, "{} colours", distinct.len());
    }

    /// A control character in a preset's name or a tag (a file written elsewhere) is drawn
    /// as if it were not there: it had blanked the whole drawer, and the strip while that
    /// preset was the current one.
    #[test]
    fn a_control_character_in_a_name_blanks_nothing() {
        let drawer = |s: &DrawerScene| {
            let mut d = DrawerRenderer::new(0.4);
            d.render(s);
            d.frame().clone()
        };
        let mut odd = scene();
        odd.rows[0].name = "Preset\u{1} 0".into();
        odd.rows[1].tags = "bass\u{7} dark".into();
        assert_eq!(drawer(&odd).data(), drawer(&scene()).data());
        let under = Pixmap::new((art::W * 0.4).round() as u32, (art::H * 0.4).round() as u32)
            .expect("a frame");
        let strip = |name: &str| {
            let mut r = crate::strip::StripRenderer::new(0.4);
            r.render(
                &crate::strip::StripScene {
                    bar: BarScene {
                        name: name.into(),
                        found: true,
                        ..BarScene::default()
                    },
                    ..crate::strip::StripScene::default()
                },
                &under,
                None,
            );
            r.frame().clone()
        };
        assert_eq!(strip("Warped\u{1b} Pad").data(), strip("Warped Pad").data());
        assert_ne!(strip("Warped Pad").data(), strip("").data());
    }
}

/// The strip with the selector and a drawer drawn to PNG files for a look (`CA72_PRESETS_PNG=<dir> cargo test
/// -p ca72-panel -- --ignored presets_png`).
#[cfg(test)]
mod png {
    use super::*;
    use resvg::tiny_skia::Pixmap;

    /// Parts close up (two pixels a unit) for looking at: a knob, a big knob, a selector and a
    /// lit tab of the strip, each `part-<name>.png` in `$CA72_PARTS_PNG`, the folder.
    #[test]
    #[ignore = "writes images for a look"]
    fn parts_png() {
        let Some(dir) = std::env::var_os("CA72_PARTS_PNG") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let k = 2.0;
        let mut panel = crate::Renderer::with_skin(crate::Skin::Worn, k, 1.0);
        let mut scene = crate::Scene::default();
        for v in scene.values.iter_mut() {
            *v = 0.5;
        }
        panel.render(&scene);
        let crop = |p: &Pixmap, (x, y): (f64, f64), r: f64, name: &str| {
            let c = p
                .clone_rect(
                    resvg::tiny_skia::IntRect::from_xywh(
                        ((x - r) * k) as i32,
                        ((y - r) * k) as i32,
                        (2.0 * r * k) as u32,
                        (2.0 * r * k) as u32,
                    )
                    .unwrap(),
                )
                .unwrap();
            c.save_png(dir.join(format!("part-{name}.png"))).unwrap();
        };
        for (param, r) in [
            ("cutoff", 90.0),
            ("osc2_frequency", 100.0),
            ("osc1_range", 90.0),
        ] {
            let c = &crate::CONTROLS[crate::controls::index(param).unwrap()];
            crop(panel.frame(), c.centre(), r, param);
        }
        let mut strip = crate::strip::StripRenderer::new(k);
        strip.render(&crate::strip::StripScene::default(), panel.frame(), None);
        let mut whole = panel.frame().clone();
        whole.draw_pixmap(
            0,
            (art::PANEL_H * k).round() as i32,
            strip.frame().as_ref(),
            &resvg::tiny_skia::PixmapPaint::default(),
            resvg::tiny_skia::Transform::identity(),
            None,
        );
        whole.save_png(dir.join("part-whole.png")).unwrap();
    }

    /// The rail close up (a pixel a unit) over the worn panel's wood, a few names on its
    /// display (one with a descender, one changed, none), each `rail-<n>.png` in
    /// `$CA72_RAIL_PNG`, the folder: for looking at the name's display.
    #[test]
    #[ignore = "writes images for a look"]
    fn rail_png() {
        let Some(dir) = std::env::var_os("CA72_RAIL_PNG") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let k = 1.0;
        let mut panel = crate::Renderer::with_skin(crate::Skin::Worn, k, 1.0);
        panel.render(&crate::Scene::default());
        let mut strip = crate::strip::StripRenderer::new(k);
        for (i, (name, changed)) in [
            ("Bass", false),
            ("Ringing Saw Line", false),
            ("Undertow Growl", true),
            ("", false),
        ]
        .into_iter()
        .enumerate()
        {
            strip.render(
                &crate::strip::StripScene {
                    bar: BarScene {
                        name: name.into(),
                        found: true,
                        changed,
                        favorite: i == 0,
                        open: false,
                        below: false,
                        hover: None,
                    },
                    ..crate::strip::StripScene::default()
                },
                panel.frame(),
                None,
            );
            let mut whole = panel.frame().clone();
            whole.draw_pixmap(
                0,
                (art::PANEL_H * k).round() as i32,
                strip.frame().as_ref(),
                &resvg::tiny_skia::PixmapPaint::default(),
                resvg::tiny_skia::Transform::identity(),
                None,
            );
            let (y0, h) = (art::PANEL_H * k, crate::strip::RAIL * k);
            let rail = whole
                .clone_rect(
                    resvg::tiny_skia::IntRect::from_xywh(0, y0 as i32, whole.width(), h as u32)
                        .unwrap(),
                )
                .unwrap();
            rail.save_png(dir.join(format!("rail-{i}.png"))).unwrap();
        }
    }

    /// A drawer as one might find it: tags picked, the list scrolled, a favourite set, a
    /// preset under the pointer, the update check with news.
    fn browsing() -> DrawerScene {
        let presets = [
            ("Bass", "bass", None, false),
            ("Lead", "lead glide", None, true),
            ("Three Saw Slab", "bass fat prog", None, false),
            ("Elastic Octaves", "bass funk glide", None, false),
            ("Pulse Strut", "bass funk feedback", None, false),
            ("Undertow Growl", "bass dark growl", Some("EDITED"), true),
            ("Upright Pluck", "bass plucked", None, false),
            ("Hollow Glider", "lead glide prog", None, false),
            ("Cruising Whistle", "lead whistle", None, false),
            ("Stacked Fifths", "lead chord prog", None, false),
            ("Ringing Saw Line", "lead resonant", None, false),
            ("Slow Horn Swell", "lead brass slow", None, false),
            ("Brass Tutti", "brass", None, true),
            ("Breath Flute", "woodwind flute", None, false),
            ("Wooden Mallet", "percussive mallet", None, false),
            ("Shoreline Wash", "fx noise", None, false),
            ("Slow Bow", "strings slow", None, false),
            ("Ladder Kick", "drums kick", None, false),
            ("Sunday Bass", "bass", Some("YOURS"), false),
            ("Glass Lead", "lead", Some("YOURS"), false),
        ];
        let chips = [
            "bass",
            "brass",
            "chord",
            "dark",
            "drums",
            "fat",
            "feedback",
            "flute",
            "funk",
            "glide",
            "growl",
            "kick",
            "lead",
            "mallet",
            "noise",
            "percussive",
            "plucked",
            "prog",
            "resonant",
            "slow",
            "strings",
            "whistle",
            "woodwind",
        ];
        DrawerScene {
            search: Field::default(),
            focus: Some(FieldId::Search),
            chips: chips
                .iter()
                .map(|t| ((*t).to_owned(), *t == "glide"))
                .collect(),
            rows: presets
                .iter()
                .map(|(n, t, o, f)| Row {
                    name: (*n).into(),
                    tags: (*t).into(),
                    origin: *o,
                    favorite: *f,
                    current: *n == "Brass Tutti",
                })
                .collect(),
            first: 1,
            current: Some(Current {
                name: "Brass Tutti".into(),
                origin: None,
                tags: "brass".into(),
                favorite: true,
            }),
            total: 26,
            hover: Some(DrawerTarget::Row(7)),
            update: UpdateScene {
                text: "0.2.0 IS OUT (THIS IS 0.1.5)".into(),
                tone: Tone::News,
                button: "DOWNLOAD".into(),
            },
            ..DrawerScene::default()
        }
    }

    /// The whole window in the worn skin at a pixel a unit, the panel as it opens over the
    /// strip (MONO's tab lit, a favourite's name on the rail, five of eight voices sounding), as
    /// `window.png` in `$CA72_WINDOW_PNG`, the folder: for looking at the strip; and with the
    /// drawer down over it, as `window-drawer.png`.
    #[test]
    #[ignore = "writes an image for a look"]
    fn window_png() {
        let Some(dir) = std::env::var_os("CA72_WINDOW_PNG") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let k = 1.0;
        let mut panel = crate::Renderer::with_skin(crate::Skin::Worn, k, 1.0);
        panel.render(&crate::Scene {
            values: [0.5; crate::CONTROLS.len()],
            ..crate::Scene::default()
        });
        let mut strip = crate::strip::StripRenderer::new(k);
        // (A favourite's name on the rail, five of eight voices sounding.)
        strip.render(
            &crate::strip::StripScene {
                field: crate::strip::Field::Scatter(
                    (0..8)
                        .map(|i| (f64::from(i) / 3.5 - 1.0, i % 3 != 1))
                        .collect(),
                ),
                bar: BarScene {
                    name: "Cycling Bass".into(),
                    found: true,
                    favorite: true,
                    ..BarScene::default()
                },
                ..crate::strip::StripScene::default()
            },
            panel.frame(),
            None,
        );
        let mut whole = panel.frame().clone();
        whole.draw_pixmap(
            0,
            (art::PANEL_H * k).round() as i32,
            strip.frame().as_ref(),
            &resvg::tiny_skia::PixmapPaint::default(),
            resvg::tiny_skia::Transform::identity(),
            None,
        );
        whole.save_png(dir.join("window.png")).unwrap();
        let mut drawer = DrawerRenderer::new(k);
        drawer.render(&browsing());
        whole.draw_pixmap(
            0,
            (DRAWER_TOP * k).round() as i32,
            drawer.frame().as_ref(),
            &resvg::tiny_skia::PixmapPaint::default(),
            resvg::tiny_skia::Transform::identity(),
            None,
        );
        whole.save_png(dir.join("window-drawer.png")).unwrap();
    }

    /// The strip with the selector, and the drawer as found, renaming, deleting, saving and
    /// showing MIDI Learn's list, each `drawer-<what>.png` in `$CA72_PRESETS_PNG`, the folder
    /// (two pixels a unit, as on a Retina screen).
    #[test]
    #[ignore = "writes images for a look"]
    fn presets_png() {
        let Some(dir) = std::env::var_os("CA72_PRESETS_PNG") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut strip = crate::strip::StripRenderer::new(0.4);
        let under = Pixmap::new((art::W * 0.4).round() as u32, (art::H * 0.4).round() as u32)
            .expect("a frame");
        strip.render(
            &crate::strip::StripScene {
                bar: BarScene {
                    name: "Undertow Growl".into(),
                    found: true,
                    changed: true,
                    favorite: true,
                    open: true,
                    below: false,
                    hover: None,
                },
                ..crate::strip::StripScene::default()
            },
            &under,
            None,
        );
        strip.frame().save_png(dir.join("strip.png")).unwrap();
        let mut d = DrawerRenderer::new(2.0);
        // (Its first opening as in the editor: a moment after the editor opens.)
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut put = |name: &str, s: &DrawerScene| {
            let t0 = std::time::Instant::now();
            d.render(s);
            println!("{name}: {:.1} ms", t0.elapsed().as_secs_f64() * 1e3);
            d.frame()
                .save_png(dir.join(format!("drawer-{name}.png")))
                .unwrap();
        };
        let found = browsing();
        put("found", &found);
        put(
            "pointing",
            &DrawerScene {
                hover: Some(DrawerTarget::Row(8)),
                ..found.clone()
            },
        );
        put(
            "searching",
            &DrawerScene {
                search: Field::new("glide"),
                hover: Some(DrawerTarget::Key(PresetKey::Rename)),
                ..found.clone()
            },
        );
        put(
            "renaming",
            &DrawerScene {
                editing: Some(Edit::Rename),
                edit: Field::new("Brass Tutti Two"),
                focus: Some(FieldId::Edit),
                ..found.clone()
            },
        );
        put(
            "deleting",
            &DrawerScene {
                editing: Some(Edit::Delete { factory: true }),
                ..found.clone()
            },
        );
        put(
            "saving",
            &DrawerScene {
                editing: Some(Edit::Save),
                save_name: Field::new("Undertow Growl"),
                save_tags: Field::new("bass, dark"),
                replace: true,
                hint: "Takes the factory's place (REVERT brings it back).".into(),
                focus: Some(FieldId::SaveTags),
                ..found.clone()
            },
        );
        // The MIDI Learn list (decisions.md R34).
        put(
            "midi",
            &DrawerScene {
                midi: Some(MidiList {
                    rows: [
                        ("TUNE", ""),
                        ("OSCILLATOR MODULATION", ""),
                        ("GLIDE", "CH 1 \u{b7} CC 5"),
                        ("MODULATION MIX", ""),
                        ("OSCILLATOR-1 RANGE", "CH 2 \u{b7} CC 20"),
                        ("OSCILLATOR-2 RANGE", ""),
                        ("OSCILLATOR-3 RANGE", ""),
                        ("OSCILLATOR-2 FREQUENCY", "CH 1 \u{b7} CC 74"),
                        ("OSCILLATOR-3 FREQUENCY", ""),
                    ]
                    .iter()
                    .enumerate()
                    .map(|(i, (n, a))| MidiRow {
                        name: (*n).into(),
                        assignment: (*a).into(),
                        waiting: i == 3,
                    })
                    .collect(),
                    first: 0,
                    chosen: 2,
                    keys: "UP, DOWN: CHOOSE \u{b7} ENTER: LEARN \u{b7} DELETE: REMOVE \u{b7} ESC: CANCEL".into(),
                    reserved: "NOT LEARNED: CC 0 AND 32 (BANK), 1 (MODULATION WHEEL), 6, 38 AND 96-101 (DATA ENTRY, RPN, NRPN), 120-127 (CHANNEL MODE)".into(),
                    status: "OSCILLATOR-2 FREQUENCY: CH 1 \u{b7} CC 74, TAKEN FROM CUTOFF FREQUENCY.".into(),
                    tone: Tone::News,
                }),
                hover: Some(DrawerTarget::MidiRow(4)),
                update: UpdateScene {
                    text: "CA-72 0.1.5".into(),
                    tone: Tone::Dim,
                    button: "CHECK FOR UPDATES".into(),
                },
                ..found.clone()
            },
        );
    }
}
