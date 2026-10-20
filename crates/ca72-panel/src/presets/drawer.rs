//! The drawer as the mock-up has it (A6: decisions.md R44; the owner, 2026-10-10: "we didn't
//! update the preset search page to the new one that you had sketched out"): on the strip's
//! face under the rail, three sections in the panel's print, their displays dots behind glass
//! as the rail's name is: FIND, the search and the tags in use; PRESETS, ALL, FAVORITES and MINE
//! over the list; PRESET, its keys (RENAME, TAGS, DELETE and REVERT, each for the preset the
//! plug-in is set to; SAVE AS, RESTORE, MIDI LEARN and CLOSE) over a display of that preset, of
//! what a key is doing (a name or tags typed, a question asked) and of the update check. MIDI
//! Learn's list shows in the list's place, its keys and the controllers it will not learn in
//! the tags'. Panel units across, drawer units down from its top.
//!
//! What does not change is drawn once a scale; a display drawn again only when what it shows
//! does, the keys only when the pointer or what they are doing does.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, Weak};
use std::thread::JoinHandle;

use plugin_kit_materials::{Disc, discs, font, resample, sprite, widened};
use resvg::tiny_skia::{FilterQuality, IntRect, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use super::{
    DOWN, DRAWER_H, DrawerScene, DrawerTarget, Edit, Field, FieldId, HOLLOW, MidiAction, MidiList,
    MidiRow, STAR, Tone, UP, UpdateScene,
};
use crate::art::{self, W};
use crate::fonts::{FAMILY, Fonts, Weight};
use crate::svg::{N, Svg, colour, put};

/// A display: its window (left, top, width, height) and its dots' pitch.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Display {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    pitch: f64,
}

impl Display {
    fn cols(&self) -> usize {
        (self.w / self.pitch) as usize
    }

    fn rows(&self) -> usize {
        (self.h / self.pitch) as usize
    }

    /// The first dot's middle (the dots centred in the window).
    fn origin(&self) -> (f64, f64) {
        let (c, r) = (self.cols() as f64, self.rows() as f64);
        (
            self.x + (self.w - c * self.pitch) / 2.0 + self.pitch / 2.0,
            self.y + (self.h - r * self.pitch) / 2.0 + self.pitch / 2.0,
        )
    }

    /// A dot's middle (fractions between).
    fn at(&self, col: f64, row: f64) -> (f64, f64) {
        let (ox, oy) = self.origin();
        (ox + col * self.pitch, oy + row * self.pitch)
    }

    /// Where (`x`, `y`) is in dots (the first's middle at 0, 0), if over the window.
    fn dots(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let inside =
            (self.x..self.x + self.w).contains(&x) && (self.y..self.y + self.h).contains(&y);
        let (ox, oy) = self.origin();
        inside.then(|| ((x - ox) / self.pitch, (y - oy) / self.pitch))
    }

    /// How many characters a line from dot `col` holds, each five dots and a dot apart, a dot
    /// left at its end.
    fn chars(&self, col: usize) -> usize {
        (self.cols().saturating_sub(col + 1) + 1) / 6
    }
}

/// The displays' foot, short of the drawer's as the mock-up's.
const FOOT: f64 = DRAWER_H - 27.0;
/// FIND's: the search, the tags; PRESETS' list; PRESET's. (The mock-up's places a tenth
/// smaller, as A6 drew them; its right section moved right to end as far from the trim as the
/// left one starts, the list longer for it.)
const SEARCH: Display = Display {
    x: 63.0,
    y: 51.0,
    w: 687.0,
    h: 47.0,
    pitch: 4.5,
};
const TAGS: Display = Display {
    x: 63.0,
    y: 110.0,
    w: 687.0,
    h: FOOT - 110.0,
    pitch: 4.0,
};
const LIST: Display = Display {
    x: 814.0,
    y: 51.0,
    w: 1476.0,
    h: FOOT - 51.0,
    pitch: 4.0,
};
const STATUS: Display = Display {
    x: 2354.0,
    y: 244.0,
    w: 678.0,
    h: FOOT - 244.0,
    pitch: 4.0,
};
const DISPLAYS: [Display; 4] = [SEARCH, TAGS, LIST, STATUS];

/// The sections' titles, printed over their displays: their middles' height and size.
const LABEL_Y: f64 = 31.0;
const LABEL: f64 = 26.0;
/// A section's title centred over it (`drawer_titles = "centre"`).
const CENTRED_LABEL: f64 = 27.0;
/// The keys: their size, the middles of their columns and rows, their legends' size.
const KEY: (f64, f64) = (164.0, 61.0);
const KEY_X: [f64; 4] = [2422.0, 2603.0, 2784.0, 2965.0];
const KEY_Y: [f64; 2] = [99.0, 197.0];
const KEY_LEGEND: f64 = 28.0;

/// A line of characters is nine dots tall (two of them a descender's); lines this many apart.
const LINE: usize = 9;
/// The search's line: its first column and row.
const SEARCH_AT: (usize, usize) = (2, 1);
/// The list: its first column; the tabs' row; the first row's; a row's height in drawer units.
const COL: usize = 2;
const TAB_ROW: usize = 1;
const ROW_TOP: usize = 11;
pub const ROW_H: f64 = LINE as f64 * LIST.pitch;
/// How many of the list's rows show.
pub const ROWS_SHOWN: usize = ((LIST.h / LIST.pitch) as usize - ROW_TOP - (LINE - 1)) / LINE + 1;
/// A row's characters: the current one's mark, the star, the name, where it is from, the tags.
const STAR_AT: usize = 2;
const NAME_AT: usize = 4;
const NAME_CHARS: usize = 21;
/// Where a row's origin (YOURS, EDITED) and its tags begin.
const ORIGIN_AT: usize = NAME_AT + NAME_CHARS + 1;
const TAGS_AT: usize = ORIGIN_AT + 7;
/// The MIDI list's: the controller after the name.
const MIDI_NAME_CHARS: usize = 24;
const MIDI_AT: usize = 28;
/// The tags: two columns, from the first row; how many lines show.
const TAG_COL: [usize; 2] = [3, 87];
const TAG_CHARS: usize = 13;
const TAG_TOP: usize = 1;
pub const TAG_LINES: usize = ((TAGS.h / TAGS.pitch) as usize - TAG_TOP - (LINE - 1)) / LINE + 1;
/// The PRESET display: its first column and row, its lines this many apart, how many; the
/// update check on its last three.
const STATUS_COL: usize = 3;
const STATUS_TOP: usize = 2;
const STATUS_LINE: usize = 10;
const STATUS_LINES: usize =
    ((STATUS.h / STATUS.pitch) as usize - STATUS_TOP - (LINE - 1)) / STATUS_LINE + 1;
const UPDATE_LINE: usize = STATUS_LINES - 1;

/// How bright a line is: the current preset's, a tab's or tag's chosen, a field's; one under
/// the pointer; the list's; what is said aside; what is barely there.
const BRIGHT: f32 = 1.0;
const POINTED: f32 = 1.0;
const NORMAL: f32 = 0.65;
const DIM: f32 = 0.475;
const FAINT: f32 = 0.4;
/// Keep the preceding appearance within hovered, focused or chosen controls, including
/// their quieter placeholders and action labels. Only unselected items get darker.
const ACTIVE_NORMAL: f32 = 0.85;
const ACTIVE_DIM: f32 = 0.7;
const ACTIVE_FAINT: f32 = 0.6;
/// Orange strokes close to the original display against quieter glass, with a little red glow.
/// Unlit dots stay below the lettering's contrast even at the minimum window width.
const DOT_ON: (u8, u8, u8) = (255, 140, 58);
const DOT_SELECTED: (u8, u8, u8) = (255, 197, 110);
const DOT_GLOW: (u8, u8, u8) = (255, 72, 10);
const UNLIT: f32 = 0.025;

/// The PRESET section's keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresetKey {
    Rename,
    Tags,
    Delete,
    Revert,
    SaveAs,
    /// The factory's presets deleted brought back.
    Restore,
    /// MIDI Learn's list in the presets' place, or back.
    MidiLearn,
    Close,
    /// Restore the sound of a fresh instance, keeping the player's preferences.
    Init,
}

impl PresetKey {
    pub const ALL: [PresetKey; 9] = [
        PresetKey::Rename,
        PresetKey::Tags,
        PresetKey::Delete,
        PresetKey::Revert,
        PresetKey::SaveAs,
        PresetKey::Restore,
        PresetKey::MidiLearn,
        PresetKey::Close,
        PresetKey::Init,
    ];

    fn legend(self) -> &'static str {
        match self {
            PresetKey::Rename => "RENAME",
            PresetKey::Tags => "TAGS",
            PresetKey::Delete => "DELETE",
            PresetKey::Revert => "REVERT",
            PresetKey::SaveAs => "SAVE AS",
            PresetKey::Restore => "RESTORE",
            PresetKey::MidiLearn => "MIDI LEARN",
            PresetKey::Close => "CLOSE",
            PresetKey::Init => "INIT",
        }
    }

    fn centre(self) -> (f64, f64) {
        if self == PresetKey::Init {
            // A dedicated action beside PRESET, above the existing two rows of keys.
            return (KEY_X[3], LABEL_Y);
        }
        let i = PresetKey::ALL.iter().position(|k| *k == self).unwrap_or(0);
        (KEY_X[i % 4], KEY_Y[i / 4])
    }

    fn size(self) -> (f64, f64) {
        if self == PresetKey::Init {
            (136.0, 45.0)
        } else {
            KEY
        }
    }
}

/// The list's tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    All,
    Favourites,
    Mine,
}

const TABS: [(Tab, &str); 3] = [
    (Tab::All, "ALL"),
    (Tab::Favourites, "FAVORITES"),
    (Tab::Mine, "MINE"),
];

/// A tab's characters, from the list's first: each three apart.
fn tab_span(t: Tab) -> (usize, usize) {
    let mut c = 0;
    for (k, w) in TABS {
        if k == t {
            return (c, c + w.len());
        }
        c += w.len() + 3;
    }
    (0, 0)
}

/// The tab the list shows.
pub fn tab_of(s: &DrawerScene) -> Tab {
    if s.mine {
        Tab::Mine
    } else if s.favourites {
        Tab::Favourites
    } else {
        Tab::All
    }
}

// ---- What the displays show.

/// The current preset alone uses gold; the rest of the display keeps its orange ink.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Ink {
    #[default]
    Amber,
    Selected,
}

impl Ink {
    /// Its colour in the built-in look.
    #[cfg(test)]
    fn colour(self) -> (u8, u8, u8) {
        self.in_dress(&Dress::default())
    }

    /// Its colour in `dress`: its lamps', or its current preset's.
    fn in_dress(self, dress: &Dress) -> (u8, u8, u8) {
        match self {
            Ink::Amber => dress.dot,
            Ink::Selected => dress.selected,
        }
    }
}

/// Characters on a display: from dot `col`, its top at dot `row`, `lv` bright, in `ink`;
/// `field`: a field's text or caret (set in a typeface, the caret goes where the typeface puts
/// the characters before it).
#[derive(Clone, Debug, PartialEq)]
struct Item {
    text: String,
    lv: f32,
    col: usize,
    row: usize,
    ink: Ink,
    field: bool,
}

/// (A control character, from a file written elsewhere, is shown as if it were not there.)
fn item(text: &str, lv: f32, col: usize, row: usize) -> Item {
    Item {
        text: text.chars().filter(|c| !c.is_control()).collect(),
        lv,
        col,
        row,
        ink: Ink::Amber,
        field: false,
    }
}

/// The caret: a line of dots under the character it is before, as a character display's cursor
/// (steady: it does not blink).
const CARET: char = '\u{2581}';
/// A selected tag's one-column bar, aligned at the left of its glyph cell. Two blank dot
/// columns then separate it from the tag; a five-column arrow would overlap its first letter.
const TAG_SELECTED: char = '\u{258f}';
/// A filled arrow for the current preset, distinct from the hover row's outline arrow.
const ROW_SELECTED: char = '\u{25b6}';

/// A character's dots, nine rows of five (the last two below the line): the font's, the
/// signs' and the caret's.
fn glyph(c: char) -> [u8; 9] {
    let seven = |r: &[u8; 7]| {
        let mut o = [0; 9];
        o[..7].copy_from_slice(r);
        o
    };
    match c {
        '\u{2605}' => seven(&STAR),
        '\u{2606}' => seven(&HOLLOW),
        '\u{25b2}' => seven(&UP),
        '\u{25bc}' => seven(&DOWN),
        CARET => [0, 0, 0, 0, 0, 0, 0, 0, 0x1f],
        TAG_SELECTED => seven(&[0x10; 7]),
        ROW_SELECTED => seven(&[0x10, 0x18, 0x1c, 0x1f, 0x1c, 0x18, 0x10]),
        _ => font::glyph(c),
    }
}

/// `s` cut to `n` characters, an ellipsis last where it was longer.
fn cut(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().filter(|c| !c.is_control()).collect();
    if chars.len() <= n {
        return chars.into_iter().collect();
    }
    let mut out: String = chars[..n.saturating_sub(1)].iter().collect();
    if n > 0 {
        out.push('\u{2026}');
    }
    out
}

/// `s` in lines of at most `n` characters, broken at spaces where it can be.
fn wrap(s: &str, n: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let mut word: Vec<char> = word.chars().filter(|c| !c.is_control()).collect();
        let len = line.chars().count();
        if len > 0 && len + 1 + word.len() <= n {
            line.push(' ');
            line.extend(word);
            continue;
        }
        if len > 0 {
            lines.push(std::mem::take(&mut line));
        }
        while word.len() > n {
            lines.push(word.drain(..n).collect());
        }
        line = word.into_iter().collect();
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// A display line's top row.
fn status_row(k: usize) -> usize {
    STATUS_TOP + k * STATUS_LINE
}

fn list_row(k: usize) -> usize {
    ROW_TOP + k * LINE
}

fn tag_row(k: usize) -> usize {
    TAG_TOP + k * LINE
}

/// The field `id`.
fn field_of(s: &DrawerScene, id: FieldId) -> &Field {
    match id {
        FieldId::Search => &s.search,
        FieldId::Edit => &s.edit,
        FieldId::SaveName => &s.save_name,
        FieldId::SaveTags => &s.save_tags,
    }
}

/// Where a field shows, if it does: its display, its first column, its top row, how many of
/// its characters.
fn field_place(s: &DrawerScene, id: FieldId) -> Option<(Display, usize, usize, usize)> {
    let status = |k: usize| (STATUS, STATUS_COL, status_row(k), STATUS.chars(STATUS_COL));
    match (id, s.midi.is_some(), s.editing) {
        (FieldId::Search, ..) => {
            Some((SEARCH, SEARCH_AT.0, SEARCH_AT.1, SEARCH.chars(SEARCH_AT.0)))
        }
        (_, true, _) => None,
        (FieldId::Edit, _, Some(Edit::Rename | Edit::Tags)) => Some(status(2)),
        (FieldId::SaveName, _, Some(Edit::Save)) => Some(labelled(status(2))),
        (FieldId::SaveTags, _, Some(Edit::Save)) => Some(labelled(status(3))),
        _ => None,
    }
}

/// A field after its label (NAME, TAGS: four letters and a space).
fn labelled(
    (d, col, row, chars): (Display, usize, usize, usize),
) -> (Display, usize, usize, usize) {
    (d, col + 6 * LABEL_CHARS, row, chars - LABEL_CHARS)
}

const LABEL_CHARS: usize = 5;

/// The first character a field holding `chars` shows: its caret kept in view.
fn shown_from(f: &Field, chars: usize) -> usize {
    f.caret.saturating_sub(chars.saturating_sub(1))
}

fn field_active(s: &DrawerScene, id: FieldId) -> bool {
    s.focus == Some(id) || s.hover == Some(DrawerTarget::Field(id))
}

/// A field's line: its text (else, empty, what it is for, faintly), and its caret when it has
/// the keyboard.
fn field_items(out: &mut Vec<Item>, s: &DrawerScene, id: FieldId, placeholder: &str) {
    let Some((_, col, row, chars)) = field_place(s, id) else {
        return;
    };
    let f = field_of(s, id);
    let from = shown_from(f, chars);
    if f.text.is_empty() {
        let lv = if field_active(s, id) {
            ACTIVE_FAINT
        } else {
            FAINT
        };
        out.push(item(placeholder, lv, col, row));
    } else {
        let shown: String = f.text.chars().skip(from).take(chars).collect();
        out.push(Item {
            field: true,
            ..item(&shown, BRIGHT, col, row)
        });
    }
    if s.focus == Some(id) && s.midi.is_none() {
        let at = f.caret.saturating_sub(from);
        out.push(Item {
            text: CARET.to_string(),
            lv: BRIGHT,
            col: col + 6 * at,
            row,
            ink: Ink::Amber,
            field: true,
        });
    }
}

fn search_items(s: &DrawerScene) -> Vec<Item> {
    let mut out = Vec::new();
    field_items(&mut out, s, FieldId::Search, "SEARCH");
    out
}

/// The tags in use, two to a line, each chosen as a filter bright; or, while MIDI Learn's
/// list shows, its keys and the controllers it will not learn.
fn tags_items(s: &DrawerScene) -> Vec<Item> {
    let mut out = Vec::new();
    let n = TAGS.chars(TAG_COL[0]);
    if let Some(m) = &s.midi {
        let mut k = 0;
        for l in m.keys.split(" \u{b7} ") {
            out.push(item(&cut(l, n), DIM, TAG_COL[0], tag_row(k)));
            k += 1;
        }
        k += 1;
        for l in wrap(&m.reserved, n) {
            if k >= TAG_LINES {
                break;
            }
            out.push(item(&l, FAINT, TAG_COL[0], tag_row(k)));
            k += 1;
        }
        return out;
    }
    for (i, (t, on)) in s.chips.iter().enumerate() {
        let line = i / 2;
        if line < s.tags_first || line >= s.tags_first + TAG_LINES {
            continue;
        }
        let lv = if *on {
            BRIGHT
        } else if s.hover == Some(DrawerTarget::Chip(i)) {
            POINTED
        } else {
            DIM
        };
        out.push(item(
            &cut(t, TAG_CHARS),
            lv,
            TAG_COL[i % 2],
            tag_row(line - s.tags_first),
        ));
        if *on {
            out.push(item(
                &TAG_SELECTED.to_string(),
                BRIGHT,
                TAG_COL[i % 2] - 3,
                tag_row(line - s.tags_first),
            ));
        }
    }
    let lines = s.chips.len().div_ceil(2);
    let sign = TAG_COL[0] + 6 * (n - 1);
    if s.tags_first > 0 {
        out.push(item("\u{25b2}", DIM, sign, tag_row(0)));
    }
    if s.tags_first + TAG_LINES < lines {
        out.push(item("\u{25bc}", DIM, sign, tag_row(TAG_LINES - 1)));
    }
    out
}

/// The list: the tabs, then each row shown (the current preset's marked and bright; its star,
/// its name, YOURS or EDITED, its tags), and whether there are more above or below; or MIDI
/// Learn's list.
fn list_items(s: &DrawerScene) -> Vec<Item> {
    if let Some(m) = &s.midi {
        return midi_items(s, m);
    }
    let mut out = Vec::new();
    let n = LIST.chars(COL);
    let tab = tab_of(s);
    for (t, w) in TABS {
        let lv = if t == tab {
            BRIGHT
        } else if s.hover == Some(DrawerTarget::Tab(t)) {
            ACTIVE_NORMAL
        } else {
            FAINT
        };
        out.push(item(w, lv, COL + 6 * tab_span(t).0, TAB_ROW));
        if t == tab {
            out.push(item(
                &CARET.to_string().repeat(w.len()),
                BRIGHT,
                COL + 6 * tab_span(t).0,
                TAB_ROW,
            ));
        }
    }
    if s.rows.is_empty() {
        out.push(item(
            "NO PRESET MATCHES",
            DIM,
            COL + 6 * NAME_AT,
            list_row(0),
        ));
    }
    for (i, r) in s.rows.iter().enumerate().skip(s.first).take(ROWS_SHOWN) {
        let row = list_row(i - s.first);
        let pointed =
            matches!(s.hover, Some(DrawerTarget::Row(h) | DrawerTarget::Star(h)) if h == i);
        let lv = if r.current {
            BRIGHT
        } else if pointed {
            POINTED
        } else {
            NORMAL
        };
        let star = if r.favorite { '\u{2605}' } else { '\u{2606}' };
        let ink = if r.current { Ink::Selected } else { Ink::Amber };
        let at = |text: &str, col: usize| Item {
            ink,
            ..item(text, lv, col, row)
        };
        // (Its columns each from its own place, as one line cut to the list's width would
        // have them: a typeface sets them as it does, and they still line up.)
        if r.current {
            out.push(at(&ROW_SELECTED.to_string(), COL));
        } else if pointed {
            out.push(at(">", COL));
        }
        out.push(at(&star.to_string(), COL + 6 * STAR_AT));
        out.push(at(&cut(&r.name, NAME_CHARS), COL + 6 * NAME_AT));
        if let Some(o) = r.origin {
            out.push(at(o, COL + 6 * ORIGIN_AT));
        }
        out.push(at(
            &cut(&r.tags, (n - 2).saturating_sub(TAGS_AT)),
            COL + 6 * TAGS_AT,
        ));
        if s.hover == Some(DrawerTarget::Star(i)) {
            out.push(Item {
                ink,
                ..item(&star.to_string(), BRIGHT, COL + 6 * STAR_AT, row)
            });
        }
    }
    more(&mut out, s.first, s.rows.len(), n);
    out
}

/// The signs at the list's right that there are more rows above or below.
fn more(out: &mut Vec<Item>, first: usize, rows: usize, n: usize) {
    let sign = COL + 6 * (n - 1);
    if first > 0 {
        out.push(item("\u{25b2}", DIM, sign, list_row(0)));
    }
    if first + ROWS_SHOWN < rows {
        out.push(item("\u{25bc}", DIM, sign, list_row(ROWS_SHOWN - 1)));
    }
}

/// A MIDI row's words that act, right to left from the row's end: what each does and says.
fn midi_actions(r: &MidiRow) -> Vec<(MidiAction, &'static str)> {
    if r.waiting {
        vec![(MidiAction::Cancel, "CANCEL")]
    } else if r.assignment.is_empty() {
        vec![(MidiAction::Learn, "LEARN")]
    } else {
        vec![(MidiAction::Remove, "REMOVE"), (MidiAction::Learn, "LEARN")]
    }
}

/// A MIDI row's words that act, each with its characters (from, past).
fn midi_spans(r: &MidiRow) -> Vec<(MidiAction, &'static str, usize, usize)> {
    let mut end = LIST.chars(COL) - 2;
    midi_actions(r)
        .into_iter()
        .map(|(a, w)| {
            let from = end - w.len();
            end = from - 2;
            (a, w, from, from + w.len())
        })
        .collect()
}

/// MIDI Learn's list (decisions.md R34), in the list's place: its title where the tabs are,
/// each control's row (the chosen one marked and bright; its name; its controller, or that it
/// waits for one; LEARN, REMOVE or CANCEL on the chosen row, a row under the pointer and one
/// waiting).
fn midi_items(s: &DrawerScene, m: &MidiList) -> Vec<Item> {
    let mut out = Vec::new();
    let n = LIST.chars(COL);
    out.push(item("MIDI LEARN", BRIGHT, COL, TAB_ROW));
    out.push(item(
        "A CONTROLLER FOR EACH CONTROL",
        DIM,
        COL + 6 * 13,
        TAB_ROW,
    ));
    for (i, r) in m.rows.iter().enumerate().skip(m.first).take(ROWS_SHOWN) {
        let row = list_row(i - m.first);
        let pointed = matches!(s.hover, Some(DrawerTarget::MidiRow(h) | DrawerTarget::MidiAction(h, _)) if h == i);
        let chosen = m.chosen == i;
        let lv = if chosen {
            BRIGHT
        } else if pointed {
            POINTED
        } else {
            NORMAL
        };
        if chosen {
            out.push(item(">", lv, COL, row));
        }
        out.push(item(&cut(&r.name, MIDI_NAME_CHARS), lv, COL + 12, row));
        let acts = if chosen || pointed || r.waiting {
            midi_spans(r)
        } else {
            Vec::new()
        };
        let end = acts.last().map_or(n - 2, |a| a.2 - 2);
        let (said, said_lv) = if r.waiting {
            ("MOVE A CONTROLLER NOW", BRIGHT)
        } else if r.assignment.is_empty() {
            ("-", if chosen || pointed { ACTIVE_DIM } else { DIM })
        } else {
            (r.assignment.as_str(), lv)
        };
        out.push(item(
            &cut(said, end - MIDI_AT),
            said_lv,
            COL + 6 * MIDI_AT,
            row,
        ));
        for (a, w, from, _) in acts {
            let lv = if s.hover == Some(DrawerTarget::MidiAction(i, a)) {
                BRIGHT
            } else if chosen || pointed {
                ACTIVE_NORMAL
            } else {
                NORMAL
            };
            out.push(item(w, lv, COL + 6 * from, row));
        }
    }
    more(&mut out, m.first, m.rows.len(), n);
    out
}

/// How bright the update check's or MIDI Learn's words are: news and trouble bright.
fn tone_level(t: Tone) -> f32 {
    match t {
        Tone::Dim => DIM,
        Tone::News | Tone::Trouble => BRIGHT,
    }
}

/// Whether the PRESET display shows the update check: not while a key's work goes on.
fn update_shown(s: &DrawerScene) -> bool {
    s.editing.is_none() || s.midi.is_some()
}

/// The update check's button as its line says it.
fn button_text(u: &UpdateScene) -> String {
    format!("[ {} ]", u.button)
}

/// The PRESET display: the preset the plug-in is set to (its name; where it is from, a star if
/// a favourite; its tags; what was last done; how many presets the list shows), or what a key
/// is doing; or MIDI Learn's; the update check on its last lines.
fn status_items(s: &DrawerScene) -> Vec<Item> {
    let mut out = Vec::new();
    let n = STATUS.chars(STATUS_COL);
    let line = |out: &mut Vec<Item>, k: usize, text: &str, lv: f32| {
        out.push(item(&cut(text, n), lv, STATUS_COL, status_row(k)));
    };
    let hint = |out: &mut Vec<Item>, from: usize, lines: usize| {
        for (k, l) in wrap(&s.hint, n).into_iter().take(lines).enumerate() {
            out.push(item(&l, NORMAL, STATUS_COL, status_row(from + k)));
        }
    };
    match (&s.midi, s.editing) {
        (Some(m), _) => {
            line(&mut out, 0, "MIDI LEARN", BRIGHT);
            for (k, l) in wrap(&m.status, n).into_iter().take(6).enumerate() {
                line(&mut out, 2 + k, &l, tone_level(m.tone));
            }
        }
        (None, Some(Edit::Rename)) => {
            line(&mut out, 0, "RENAME IT", BRIGHT);
            field_items(&mut out, s, FieldId::Edit, "");
            hint(&mut out, 4, 2);
            line(&mut out, 8, "ENTER: RENAME  ESC: KEEP", DIM);
        }
        (None, Some(Edit::Tags)) => {
            line(&mut out, 0, "ITS TAGS, COMMAS BETWEEN", BRIGHT);
            field_items(&mut out, s, FieldId::Edit, "");
            hint(&mut out, 4, 2);
            line(&mut out, 8, "ENTER: SET  ESC: KEEP", DIM);
        }
        (None, Some(Edit::Delete { factory })) => {
            let name = s.current.as_ref().map_or("", |c| c.name.as_str());
            line(
                &mut out,
                0,
                &format!("DELETE {}?", cut(name, n - 8)),
                BRIGHT,
            );
            if factory {
                line(&mut out, 2, "RESTORE BRINGS IT BACK", DIM);
            }
            hint(&mut out, 4, 2);
            line(&mut out, 8, "ENTER: DELETE  ESC: KEEP", DIM);
        }
        (None, Some(Edit::Save)) => {
            line(&mut out, 0, "SAVE THE SOUND AS", BRIGHT);
            for (k, id, label) in [
                (2, FieldId::SaveName, "NAME"),
                (3, FieldId::SaveTags, "TAGS"),
            ] {
                let lv = if field_active(s, id) { ACTIVE_DIM } else { DIM };
                line(&mut out, k, label, lv);
            }
            field_items(&mut out, s, FieldId::SaveName, "");
            field_items(&mut out, s, FieldId::SaveTags, "bass, dark");
            hint(&mut out, 5, 2);
            line(&mut out, 8, "ENTER: SAVE  ESC: CANCEL", DIM);
        }
        (None, None) => {
            match &s.current {
                Some(c) => {
                    line(&mut out, 0, &c.name, BRIGHT);
                    let from = match c.origin {
                        Some("YOURS") => "YOUR PRESET",
                        Some("EDITED") => "YOUR EDIT OF THE FACTORY'S",
                        _ => "FACTORY PRESET",
                    };
                    let from = if c.favorite {
                        format!("\u{2605} {from}")
                    } else {
                        from.to_owned()
                    };
                    line(&mut out, 1, &from, DIM);
                    line(&mut out, 2, &c.tags, DIM);
                }
                None => line(&mut out, 0, "NO PRESET", DIM),
            }
            hint(&mut out, 4, 3);
            let shown = if s.rows.len() == s.total {
                format!("{} PRESETS", s.total)
            } else {
                format!("{} OF {} PRESETS SHOWN", s.rows.len(), s.total)
            };
            line(&mut out, 7, &shown, FAINT);
        }
    }
    // The update check, while no key's work goes on: what it says, on the lines above its
    // button's.
    if !update_shown(s) {
        return out;
    }
    let u = &s.update;
    let said = wrap(&u.text, n);
    let said = &said[said.len().saturating_sub(2)..];
    for (k, l) in said.iter().enumerate() {
        line(
            &mut out,
            UPDATE_LINE - said.len() + k,
            l,
            tone_level(u.tone),
        );
    }
    if !u.button.is_empty() {
        let lv = if s.hover == Some(DrawerTarget::Update) {
            BRIGHT
        } else {
            NORMAL
        };
        line(&mut out, UPDATE_LINE, &button_text(u), lv);
    }
    out
}

/// What the keys show: the one under the pointer, and the ones held down (what they began is
/// going on).
#[derive(Clone, Debug, PartialEq, Default)]
struct KeysShown {
    pointed: Option<PresetKey>,
    held: Vec<PresetKey>,
    unavailable: Vec<PresetKey>,
}

fn keys_shown(s: &DrawerScene) -> KeysShown {
    let mut held = Vec::new();
    if s.midi.is_some() {
        held.push(PresetKey::MidiLearn);
    } else {
        match s.editing {
            Some(Edit::Rename) => held.push(PresetKey::Rename),
            Some(Edit::Tags) => held.push(PresetKey::Tags),
            Some(Edit::Delete { .. }) => held.push(PresetKey::Delete),
            Some(Edit::Save) => held.push(PresetKey::SaveAs),
            None => {}
        }
    }
    KeysShown {
        pointed: match s.hover {
            Some(DrawerTarget::Key(k)) => Some(k),
            _ => None,
        },
        held,
        unavailable: PresetKey::ALL
            .into_iter()
            .filter(|k| match k {
                PresetKey::Rename | PresetKey::Tags | PresetKey::Delete => s.current.is_none(),
                PresetKey::Revert => s
                    .current
                    .as_ref()
                    .is_none_or(|c| c.origin != Some("EDITED")),
                _ => false,
            })
            .collect(),
    }
}

// ---- What a pointer finds.

/// The line of a display (lines `apart` dots apart from row `top`, `count` of them) at row
/// `r` (fractions between the dots' rows), if any.
fn line_at(r: f64, top: usize, apart: usize, count: usize) -> Option<usize> {
    let k = (r - top as f64 + 0.5) / apart as f64;
    (k >= 0.0 && (k as usize) < count).then_some(k as usize)
}

/// The character (from dot `col0`) at column `c`, if right of `col0`.
fn char_at(c: f64, col0: usize) -> Option<usize> {
    let k = (c - col0 as f64 + 0.5) / 6.0;
    (k >= 0.0).then_some(k as usize)
}

/// What is at (`x`, `y`) in the drawer (drawer units).
pub fn drawer_hit(s: &DrawerScene, x: f64, y: f64) -> Option<DrawerTarget> {
    if !(0.0..DRAWER_H).contains(&y) || !(0.0..W).contains(&x) {
        return None;
    }
    for k in PresetKey::ALL {
        let (kx, ky) = k.centre();
        let (w, h) = k.size();
        if (x - kx).abs() <= w / 2.0 + 4.0 && (y - ky).abs() <= h / 2.0 + 4.0 {
            return Some(DrawerTarget::Key(k));
        }
    }
    if SEARCH.dots(x, y).is_some() {
        return Some(DrawerTarget::Field(FieldId::Search));
    }
    if let Some((c, r)) = TAGS.dots(x, y) {
        return Some(tag_hit(s, c, r));
    }
    if let Some((c, r)) = LIST.dots(x, y) {
        return Some(match &s.midi {
            Some(m) => midi_hit(m, c, r),
            None => list_hit(s, c, r),
        });
    }
    if let Some((c, r)) = STATUS.dots(x, y) {
        return Some(status_hit(s, c, r));
    }
    Some(DrawerTarget::Back)
}

fn tag_hit(s: &DrawerScene, c: f64, r: f64) -> DrawerTarget {
    if s.midi.is_some() {
        return DrawerTarget::Back;
    }
    let Some(k) = line_at(r, TAG_TOP, LINE, TAG_LINES) else {
        return DrawerTarget::Back;
    };
    let column = usize::from(c >= TAG_COL[1] as f64 - 3.0);
    let i = (s.tags_first + k) * 2 + column;
    if i < s.chips.len() {
        DrawerTarget::Chip(i)
    } else {
        DrawerTarget::Back
    }
}

fn list_hit(s: &DrawerScene, c: f64, r: f64) -> DrawerTarget {
    if r < ROW_TOP as f64 - 0.5 {
        let ch = char_at(c, COL);
        for (t, _) in TABS {
            let (from, past) = tab_span(t);
            if ch.is_some_and(|ch| (from..past).contains(&ch)) {
                return DrawerTarget::Tab(t);
            }
        }
        return DrawerTarget::Back;
    }
    let Some(k) = line_at(r, ROW_TOP, LINE, ROWS_SHOWN) else {
        return DrawerTarget::Back;
    };
    let i = s.first + k;
    if i >= s.rows.len() {
        return DrawerTarget::Back;
    }
    match char_at(c, COL) {
        Some(ch) if (STAR_AT - 1..=STAR_AT + 1).contains(&ch) => DrawerTarget::Star(i),
        _ => DrawerTarget::Row(i),
    }
}

fn midi_hit(m: &MidiList, c: f64, r: f64) -> DrawerTarget {
    let Some(k) = line_at(r, ROW_TOP, LINE, ROWS_SHOWN) else {
        return DrawerTarget::Back;
    };
    let i = m.first + k;
    let Some(row) = m.rows.get(i) else {
        return DrawerTarget::Back;
    };
    if let Some(ch) = char_at(c, COL) {
        for (a, _, from, past) in midi_spans(row) {
            if (from..past).contains(&ch) {
                return DrawerTarget::MidiAction(i, a);
            }
        }
    }
    DrawerTarget::MidiRow(i)
}

fn status_hit(s: &DrawerScene, c: f64, r: f64) -> DrawerTarget {
    let Some(k) = line_at(r, STATUS_TOP, STATUS_LINE, STATUS_LINES) else {
        return DrawerTarget::Back;
    };
    if k == UPDATE_LINE
        && update_shown(s)
        && !s.update.button.is_empty()
        && char_at(c, STATUS_COL).is_some_and(|ch| ch < button_text(&s.update).chars().count())
    {
        return DrawerTarget::Update;
    }
    for id in [FieldId::Edit, FieldId::SaveName, FieldId::SaveTags] {
        if field_place(s, id).is_some_and(|(_, _, row, _)| row == status_row(k)) {
            return DrawerTarget::Field(id);
        }
    }
    DrawerTarget::Back
}

/// The caret's place in a field for a pointer at `x` across it (the nearest gap).
pub fn caret_at(s: &DrawerScene, id: FieldId, x: f64) -> usize {
    let f = field_of(s, id);
    let Some((d, col, _, chars)) = field_place(s, id) else {
        return f.caret;
    };
    let (ox, _) = d.origin();
    let gap = (((x - ox) / d.pitch - col as f64 + 0.5) / 6.0)
        .round()
        .max(0.0) as usize;
    (shown_from(f, chars) + gap).min(f.text.chars().count())
}

/// Whether (`x`, `y`) is over the tags (the wheel scrolls them there).
pub fn in_tags(x: f64, y: f64) -> bool {
    TAGS.dots(x, y).is_some()
}

/// Whether the update check's `scene` is shown whole: its words on two lines at most, none
/// cut, and its button's on one.
pub fn update_fits(scene: &UpdateScene) -> bool {
    let n = STATUS.chars(STATUS_COL);
    let lines = wrap(&scene.text, n);
    lines.len() <= 2
        && lines.iter().all(|l| l.chars().count() <= n)
        && button_text(scene).chars().count() <= n
}

// ---- Where things are (drawer units), for the editor's tests and its own.

/// The middle of characters `ch` (fractions between) of a line from dot `col0`, row `row`.
fn char_middle(d: &Display, col0: usize, ch: f64, row: usize) -> (f64, f64) {
    d.at(col0 as f64 + ch * 6.0 + 2.0, row as f64 + 3.0)
}

pub fn key_centre(k: PresetKey) -> (f64, f64) {
    k.centre()
}

/// MIDI LEARN's key.
pub fn midi_centre() -> (f64, f64) {
    key_centre(PresetKey::MidiLearn)
}

pub fn search_centre() -> (f64, f64) {
    (SEARCH.x + SEARCH.w / 2.0, SEARCH.y + SEARCH.h / 2.0)
}

pub fn tab_centre(t: Tab) -> (f64, f64) {
    let (from, past) = tab_span(t);
    char_middle(&LIST, COL, (from + past) as f64 / 2.0 - 0.5, TAB_ROW)
}

/// Where tag `i`'s middle is, if it shows.
pub fn chip_centre(s: &DrawerScene, i: usize) -> Option<(f64, f64)> {
    let line = (i / 2).checked_sub(s.tags_first)?;
    (i < s.chips.len() && line < TAG_LINES)
        .then(|| char_middle(&TAGS, TAG_COL[i % 2], 1.0, tag_row(line)))
}

/// Where row `i`'s name is, if it shows.
pub fn row_centre(s: &DrawerScene, i: usize) -> Option<(f64, f64)> {
    let k = i.checked_sub(s.first)?;
    (i < s.rows.len() && k < ROWS_SHOWN)
        .then(|| char_middle(&LIST, COL, (NAME_AT + 4) as f64, list_row(k)))
}

/// Where row `i`'s star is, if it shows.
pub fn star_centre(s: &DrawerScene, i: usize) -> Option<(f64, f64)> {
    let k = i.checked_sub(s.first)?;
    (i < s.rows.len() && k < ROWS_SHOWN)
        .then(|| char_middle(&LIST, COL, STAR_AT as f64, list_row(k)))
}

/// Where a field's line's middle is, if it shows.
pub fn field_centre(s: &DrawerScene, id: FieldId) -> Option<(f64, f64)> {
    let (d, col, row, chars) = field_place(s, id)?;
    Some(char_middle(&d, col, chars as f64 / 2.0, row))
}

/// Where the update check's button is.
pub fn update_centre() -> (f64, f64) {
    char_middle(&STATUS, STATUS_COL, 3.0, status_row(UPDATE_LINE))
}

/// Where the MIDI list's row `i`'s name is, if it shows.
pub fn midi_row_centre(m: &MidiList, i: usize) -> Option<(f64, f64)> {
    let k = i.checked_sub(m.first)?;
    (i < m.rows.len() && k < ROWS_SHOWN).then(|| char_middle(&LIST, COL, 6.0, list_row(k)))
}

/// Where the MIDI list's row `i`'s word doing `a` is, if the row has it.
pub fn midi_action_centre(m: &MidiList, i: usize, a: MidiAction) -> Option<(f64, f64)> {
    let k = i.checked_sub(m.first)?;
    if k >= ROWS_SHOWN {
        return None;
    }
    let (_, _, from, past) = midi_spans(m.rows.get(i)?)
        .into_iter()
        .find(|(b, ..)| *b == a)?;
    Some(char_middle(
        &LIST,
        COL,
        (from + past) as f64 / 2.0 - 0.5,
        list_row(k),
    ))
}

// ---- Drawing.

/// The drawer's renderer at the panel's scale.
pub struct DrawerRenderer {
    scale: f64,
    /// For the keys' marks (no lettering).
    options: usvg::Options<'static>,
    /// What its look gives it, and its typeface for its displays (where they are lettered in
    /// it).
    dress: Arc<Dress>,
    typed: usvg::Options<'static>,
    typeface: Arc<Fonts>,
    /// The pictures and what does not change at the scale it opened at, made on a thread of
    /// their own as the editor opens (none once taken).
    warming: Option<JoinHandle<Warmed>>,
    pictures: Option<Arc<Pictures>>,
    /// What does not change at this scale (shared by the drawers open at it), and the frame
    /// drawn over it (made as it is first drawn).
    still: Option<Arc<Pixmap>>,
    frame: Pixmap,
    shown: Option<DrawerScene>,
    /// What each display shows, and the keys.
    drawn: [Option<Vec<Item>>; 4],
    keys: Option<KeysShown>,
}

impl fmt::Debug for DrawerRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DrawerRenderer")
            .field("scale", &self.scale)
            .finish()
    }
}

/// What the thread begun as the editor opens hands back: the pictures, the scale and the
/// dressing it made the still in, the still.
type Warmed = (Arc<Pictures>, f64, u64, Arc<Pixmap>);

/// What a look gives the drawer (decisions.md R-SKIN-PACKS): its face (as the panel's), its
/// displays' glass, its keys' cap, its lamps' colour (the dots' and the lettering lit), its
/// print's colours and typeface, a drawing over its face, and whether its displays are lettered
/// in that typeface; the built-in ones where it gives none. `key` tells one look's dressing from another's: 0, the built-in one (every
/// official look's).
#[derive(Clone)]
struct Dress {
    key: u64,
    face: Option<Pixmap>,
    glass: Option<Pixmap>,
    button: Option<Pixmap>,
    dot: (u8, u8, u8),
    glow: (u8, u8, u8),
    /// The current preset's lettering: gold over the built-in lamps, the lamps' own colour
    /// over a look's.
    selected: (u8, u8, u8),
    /// The sections' titles'; the keys' legends' on their caps; the displays' lettering not
    /// lit.
    print: String,
    legend: String,
    unlit: String,
    /// The current row's band (none: faintly the lamps' glow).
    selection: Option<(u8, u8, u8)>,
    fonts: Option<(&'static [u8], &'static [u8])>,
    art: Option<Arc<usvg::Tree>>,
    typed: bool,
    centred: bool,
}

impl Default for Dress {
    fn default() -> Self {
        Dress {
            key: 0,
            face: None,
            glass: None,
            button: None,
            dot: DOT_ON,
            glow: DOT_GLOW,
            selected: DOT_SELECTED,
            print: colour::LEGEND.into(),
            legend: colour::LEGEND.into(),
            unlit: TYPED_UNLIT.into(),
            selection: None,
            fonts: None,
            art: None,
            typed: false,
            centred: false,
        }
    }
}

/// The displays' lettering not lit, in a typeface, where a look gives no colour for it: a
/// cream.
const TYPED_UNLIT: &str = "#e6dcc4";

impl Dress {
    fn of(look: &crate::look::Look) -> Dress {
        let pack = |name: &str| {
            let size = crate::skin::slot_size(name).unwrap_or((1, 1));
            look.pictures.get(name).and_then(|p| p.picture(size))
        };
        let hex = |(r, g, b): (u8, u8, u8)| format!("#{r:02x}{g:02x}{b:02x}");
        let c = look.colours;
        let lamp = c.lamp.map(crate::strip::Lamp::of);
        let built_in = Dress::default();
        let mut d = Dress {
            face: pack("face").map(|f| match look.face_tint {
                Some(t) => crate::skin::tinted(&f, t),
                None => f,
            }),
            glass: pack("glass"),
            button: pack("key"),
            dot: lamp.map_or(DOT_ON, |l| l.on),
            glow: lamp.map_or(DOT_GLOW, |l| l.glow),
            selected: lamp.map_or(DOT_SELECTED, |l| l.on),
            print: c.tab_print.map_or(built_in.print, hex),
            legend: c.key_print.map_or(built_in.legend, hex),
            unlit: c.display.or(c.tab_print).map_or(built_in.unlit, hex),
            selection: c.selection,
            fonts: look.fonts,
            art: look.drawer_art.clone(),
            typed: look.drawer_type,
            centred: look.drawer_centred,
            key: 0,
        };
        let own = d.face.is_some()
            || d.glass.is_some()
            || d.button.is_some()
            || d.fonts.is_some()
            || d.art.is_some()
            || d.typed
            || d.centred
            || c.lamp.is_some()
            || c.tab_print.is_some()
            || c.key_print.is_some()
            || c.display.is_some()
            || c.selection.is_some();
        if own {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (&look.id, &look.version).hash(&mut h);
            d.key = h.finish().max(1);
        }
        d
    }

    /// Its typeface (the built-in one where it brings none or its own does not load).
    fn fonts(&self) -> Fonts {
        self.fonts
            .and_then(|(regular, bold)| Fonts::with(regular, bold).ok())
            .unwrap_or_default()
    }
}

/// The pictures a drawer is made of, decoded: the face's, the displays' glass (smoothed), the
/// keys' cap.
struct Pictures {
    face: Pixmap,
    glass: Pixmap,
    button: Pixmap,
}

impl Pictures {
    fn load(dress: &Dress) -> Pictures {
        let png = |b: &[u8]| Pixmap::decode_png(b).expect("a built-in picture decodes");
        Pictures {
            face: dress.face.clone().unwrap_or_else(crate::skin::face_picture),
            glass: smooth_glass(
                &dress
                    .glass
                    .clone()
                    .unwrap_or_else(|| png(crate::skin::GLASS)),
            ),
            button: dress
                .button
                .clone()
                .unwrap_or_else(|| png(crate::skin::BUTTON)),
        }
    }
}

/// What does not change at `scale` in `dress`: another drawer's open at it, else made (and
/// kept for others while one has it: each instance's editor has its own drawer).
fn shared_still(scale: f64, pictures: &Pictures, dress: &Dress) -> Arc<Pixmap> {
    /// A still's scale (its bits) and dressing.
    type Key = (u64, u64);
    static SHARED: Mutex<Vec<(Key, Weak<Pixmap>)>> = Mutex::new(Vec::new());
    let key = (scale.to_bits(), dress.key);
    let found = {
        let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
        shared.retain(|(_, w)| w.strong_count() > 0);
        shared
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, w)| w.upgrade())
    };
    if let Some(still) = found {
        return still;
    }
    let still = Arc::new(make_still(scale, pictures, dress));
    SHARED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, Arc::downgrade(&still)));
    still
}

fn size_at(scale: f64) -> (u32, u32) {
    (
        (W * scale).ceil().max(1.0) as u32,
        (DRAWER_H * scale).ceil().max(1.0) as u32,
    )
}

/// An SVG body drawn over `frame` (drawer units at `scale`).
fn svg_over(options: &usvg::Options<'_>, frame: &mut Pixmap, scale: f64, body: &str) {
    let (w, h) = (frame.width(), frame.height());
    let view = [0.0, 0.0, f64::from(w) / scale, f64::from(h) / scale];
    let doc = art::document(view, w, h, body);
    if let Ok(tree) = usvg::Tree::from_str(&doc, options) {
        resvg::render(&tree, Transform::identity(), &mut frame.as_mut());
    }
}

/// `from`'s pixels in the rectangle (drawer units: left, top, right, bottom) put in `to`.
fn restore(to: &mut Pixmap, from: &Pixmap, scale: f64, (x0, y0, x1, y1): (f64, f64, f64, f64)) {
    let (w, h) = (to.width() as i64, to.height() as i64);
    let px = |v: f64, most: i64| ((v * scale) as i64).clamp(0, most);
    let (l, r) = (px(x0.floor(), w), px(x1.ceil() + 1.0, w));
    let (t, b) = (px(y0.floor(), h), px(y1.ceil() + 1.0, h));
    if l >= r {
        return;
    }
    let (row, a, z) = (w as usize * 4, l as usize * 4, r as usize * 4);
    for y in t as usize..b as usize {
        let i = y * row;
        to.data_mut()[i + a..i + z].copy_from_slice(&from.data()[i + a..i + z]);
    }
}

/// The plate's surfaces: the trim at its ends and foot, as the strip's; the rail's shadow
/// across its top; each display's bezel; each key's shadow.
fn plate_svg() -> String {
    let mut s = Svg::default();
    put!(
        s,
        "<defs><linearGradient id='rail-shade' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.6'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient><linearGradient id='bz-frame' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#2c2a27'/><stop offset='0.5' stop-color='#1a1917'/><stop offset='1' stop-color='#0e0d0c'/></linearGradient><linearGradient id='bz-edge' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#fff' stop-opacity='0.28'/><stop offset='0.45' stop-color='#fff' stop-opacity='0.04'/><stop offset='1' stop-color='#000' stop-opacity='0.6'/></linearGradient></defs>"
    );
    art::strip_trims(&mut s);
    put!(
        s,
        "<rect x='0' y='0' width='{}' height='14' fill='url(#rail-shade)'/>",
        N(W)
    );
    for d in DISPLAYS {
        let o = 6.0;
        let (fx, fy, fw, fh) = (d.x - o, d.y - o, d.w + 2.0 * o, d.h + 2.0 * o);
        // (Each only as a rim round what covers its middle: the shadow round the frame, the
        // frame round the window's black, the black round the glass. Filled whole, a display's
        // size again and again had cost a tenth of a second at a Retina screen's scale.)
        let frame = (fx + 2.0, fy + 2.0, fw - 4.0, fh - 4.0, 6.0);
        soft_shadow(&mut s, (fx + 1.5, fy + 3.0, fw, fh), 8.0, 2.5, 0.55, frame);
        let black = (d.x - 2.0, d.y - 2.0, d.w + 4.0, d.h + 4.0, 5.0);
        let glass = (d.x + 3.0, d.y + 3.0, d.w - 6.0, d.h - 6.0, 1.0);
        put!(
            s,
            "<path d='{}{}' fill-rule='evenodd' fill='url(#bz-frame)'/><path d='{}' fill='none' stroke='url(#bz-edge)' stroke-width='1.2'/>",
            rounded((fx, fy, fw, fh, 8.0)),
            rounded(black),
            rounded((fx + 0.6, fy + 0.6, fw - 1.2, fh - 1.2, 7.5))
        );
        put!(
            s,
            "<path d='{}{}' fill-rule='evenodd' fill='#050504'/>",
            rounded(black),
            rounded(glass)
        );
    }
    for k in PresetKey::ALL {
        let (x, y) = k.centre();
        let (w, h) = k.size();
        soft_shadow(
            &mut s,
            (x - w / 2.0 + 3.0, y - h / 2.0 + 5.0, w, h),
            9.0,
            3.0,
            0.6,
            (
                x - w / 2.0 + 8.0,
                y - h / 2.0 + 8.0,
                w - 16.0,
                h - 16.0,
                4.0,
            ),
        );
    }
    s.0
}

/// A soft shadow without blurring (a blur over a display's size took most of a second at a
/// Retina screen's scale): the shape again and again, each larger by a step up to `spread` and
/// fainter, together as dark as `alpha` at its middle; none of it inside `hole`, which what
/// casts it covers.
fn soft_shadow(
    s: &mut Svg,
    (x, y, w, h): (f64, f64, f64, f64),
    rx: f64,
    spread: f64,
    alpha: f64,
    hole: (f64, f64, f64, f64, f64),
) {
    let steps = 5;
    let each = 1.0 - (1.0 - alpha).powf(1.0 / f64::from(steps));
    for i in 0..steps {
        let o = spread * (f64::from(i) / f64::from(steps - 1) - 0.5) * 2.0;
        put!(
            s,
            "<path d='{}{}' fill-rule='evenodd' fill='#000' fill-opacity='{}'/>",
            rounded((x - o, y - o, w + 2.0 * o, h + 2.0 * o, (rx + o).max(0.0))),
            rounded(hole),
            N(each)
        );
    }
}

/// A rounded rectangle (left, top, width, height, its corners' radius) as a path's outline.
fn rounded((x, y, w, h, r): (f64, f64, f64, f64, f64)) -> String {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    format!(
        "M{} {}H{}A{r} {r} 0 0 1 {} {}V{}A{r} {r} 0 0 1 {} {}H{}A{r} {r} 0 0 1 {} {}V{}A{r} {r} 0 0 1 {} {}Z",
        N(x + r),
        N(y),
        N(x + w - r),
        N(x + w),
        N(y + r),
        N(y + h - r),
        N(x + w - r),
        N(y + h),
        N(x + r),
        N(x),
        N(y + h - r),
        N(y + r),
        N(x + r),
        N(y),
        r = N(r)
    )
}

/// `tile` laid over the whole of `frame`, mirrored tile to tile (as a pattern that reflects),
/// its first at (`tx`, `ty`) pixels: copied, each row's run at a time, not drawn.
fn tile_reflected(frame: &mut Pixmap, tile: &Pixmap, (tx, ty): (i64, i64)) {
    let (tw, th) = (i64::from(tile.width()), i64::from(tile.height()));
    let w = frame.width() as usize;
    let (n, period) = (tw as usize, 2 * tw as usize);
    // Where along the tile and its mirror image (a period) the frame's first pixel falls.
    let start = (-tx).rem_euclid(2 * tw) as usize;
    let mut wide = vec![0u8; period * 4];
    for (y, out) in frame.data_mut().chunks_exact_mut(w * 4).enumerate() {
        let m = (y as i64 - ty).rem_euclid(2 * th);
        let sy = (if m < th { m } else { 2 * th - 1 - m }) as usize;
        let row = &tile.data()[sy * n * 4..(sy + 1) * n * 4];
        wide[..n * 4].copy_from_slice(row);
        for (x, p) in row.chunks_exact(4).enumerate() {
            let at = (period - 1 - x) * 4;
            wide[at..at + 4].copy_from_slice(p);
        }
        let (mut x, mut at) = (0, start);
        while x < w {
            let k = (period - at).min(w - x);
            out[x * 4..(x + k) * 4].copy_from_slice(&wide[at * 4..(at + k) * 4]);
            x += k;
            at = 0;
        }
    }
}

/// The glass for the drawer's displays: the rail's, its rows and its columns evened out (a
/// band brighter or darker than those round it brought to theirs) and its mesh softened away.
/// (Drawn out over a display many times its size, the picture's faint lines had been hard
/// bands across the list: the owner, 2026-10-10, "there's several harsh horizontal lines,
/// please get rid of them". The dots behind it are the displays' texture.)
pub(crate) fn smooth_glass(p: &Pixmap) -> Pixmap {
    let (w, h) = (p.width() as usize, p.height() as usize);
    let mut px: Vec<[f32; 3]> = p
        .pixels()
        .iter()
        .map(|c| {
            [
                f32::from(c.red()),
                f32::from(c.green()),
                f32::from(c.blue()),
            ]
        })
        .collect();
    let level = |c: &[f32; 3]| c[0] + c[1] + c[2];
    // Each row's (then each column's) mean against the mean of the rows near it.
    let even =
        |px: &mut Vec<[f32; 3]>, n: usize, along: usize, at: &dyn Fn(usize, usize) -> usize| {
            let means: Vec<f32> = (0..n)
                .map(|i| (0..along).map(|j| level(&px[at(i, j)])).sum::<f32>() / along as f32)
                .collect();
            let reach = 24;
            for i in 0..n {
                let (a, b) = (i.saturating_sub(reach), (i + reach + 1).min(n));
                let near = means[a..b].iter().sum::<f32>() / (b - a) as f32;
                let k = if means[i] > 0.0 { near / means[i] } else { 1.0 };
                for j in 0..along {
                    for v in px[at(i, j)].iter_mut() {
                        *v = (*v * k).min(255.0);
                    }
                }
            }
        };
    even(&mut px, h, w, &|i, j| i * w + j);
    even(&mut px, w, h, &|i, j| j * w + i);
    let mut out = p.clone();
    for (o, c) in out.pixels_mut().iter_mut().zip(&px) {
        let v = |x: f32| x.round().clamp(0.0, 255.0) as u8;
        if let Some(n) =
            resvg::tiny_skia::PremultipliedColorU8::from_rgba(v(c[0]), v(c[1]), v(c[2]), 255)
        {
            *o = n;
        }
    }
    // The mesh: the picture at a sixth of its size (drawn out smoothly again over a display).
    resample(&out, (p.width() / 6).max(1), (p.height() / 6).max(1))
}

/// A large display's glass (the smoothed picture: [`smooth_glass`]) filling the window `(x, y, w,
/// h)` (the frame's units): the effects' list's (decisions.md R-FX-DRAWER), as the drawer's.
pub(crate) fn display_glass(
    frame: &mut Pixmap,
    p: &Pixmap,
    scale: f64,
    (x, y, w, h): (f64, f64, f64, f64),
) {
    glass(
        frame,
        p,
        scale,
        &Display {
            x,
            y,
            w,
            h,
            pitch: 1.0,
        },
    );
}

/// A display's glass filling its window, covering it about its middle (as the kit's `glass`),
/// drawn out bilinearly by hand: each of the picture's rows across once, then each pixel's row
/// between two of those. (The smoothed picture is small and drawn out many times its size;
/// filled through a pattern, the four displays had taken a sixteenth of a second at a Retina
/// screen's scale.)
fn glass(frame: &mut Pixmap, p: &Pixmap, scale: f64, d: &Display) {
    let (x, y, w, h) = (d.x * scale, d.y * scale, d.w * scale, d.h * scale);
    let (pw, ph) = (p.width() as usize, p.height() as usize);
    let k = (w / pw as f64).max(h / ph as f64);
    let (ox, oy) = (x + (w - pw as f64 * k) / 2.0, y + (h - ph as f64 * k) / 2.0);
    let (fw, fh) = (i64::from(frame.width()), i64::from(frame.height()));
    let (x0, x1) = ((x.round() as i64).max(0), ((x + w).round() as i64).min(fw));
    let (y0, y1) = ((y.round() as i64).max(0), ((y + h).round() as i64).min(fh));
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    // A pixel's place in the picture: the two pixels it falls between, and how far along.
    let between = |o: i64, origin: f64, n: usize| {
        let u = ((o as f64 + 0.5 - origin) / k - 0.5).clamp(0.0, n as f64 - 1.0);
        let i = u.floor() as usize;
        (i, (i + 1).min(n - 1), (u - i as f64) as f32)
    };
    let cols: Vec<(usize, usize, f32)> = (x0..x1).map(|px| between(px, ox, pw)).collect();
    let src = p.pixels();
    // (A row drawn out across in 256ths of a level, so a pixel's row between two is mixed in
    // whole numbers.)
    let across = |j: usize| -> Vec<[u32; 3]> {
        let row = &src[j * pw..(j + 1) * pw];
        cols.iter()
            .map(|&(a, b, t)| {
                let (a, b) = (row[a], row[b]);
                let mix = |u: u8, v: u8| {
                    ((f32::from(u) + (f32::from(v) - f32::from(u)) * t) * 0.6 * 256.0) as u32
                };
                [
                    mix(a.red(), b.red()),
                    mix(a.green(), b.green()),
                    mix(a.blue(), b.blue()),
                ]
            })
            .collect()
    };
    let mut upper: (usize, Vec<[u32; 3]>) = (usize::MAX, Vec::new());
    let mut lower = upper.clone();
    let r = 4.0 * scale;
    let fw = fw as usize;
    let data = frame.data_mut();
    for py in y0..y1 {
        let (j0, j1, t) = between(py, oy, ph);
        if upper.0 != j0 {
            upper = if lower.0 == j0 {
                std::mem::take(&mut lower)
            } else {
                (j0, across(j0))
            };
        }
        if lower.0 != j1 {
            lower = (j1, across(j1));
        }
        // The rounded corners: the row's ends drawn in where it crosses one.
        let yc = py as f64 + 0.5;
        let e = (y + r - yc).max(yc - (y + h - r));
        let inset = if e > 0.0 {
            r - (r * r - e * e).max(0.0).sqrt()
        } else {
            0.0
        };
        let (a, b) = (
            ((x + inset).round() as i64).max(x0),
            ((x + w - inset).round() as i64).min(x1),
        );
        let row = py as usize * fw * 4;
        let t = (t * 256.0) as u32;
        for px in a..b {
            let i = (px - x0) as usize;
            let (u, v) = (upper.1[i], lower.1[i]);
            let o = row + px as usize * 4;
            for c in 0..3 {
                data[o + c] = ((u[c] * (256 - t) + v[c] * t + (1 << 15)) >> 16).min(255) as u8;
            }
            data[o + 3] = 255;
        }
    }
}

/// Every dot of a display, faintly (`alpha`): a row's dots drawn once for each eighth of a
/// pixel a row can fall at, and laid row by row. (A hundred thousand discs, each drawn, had
/// taken a twentieth of a second at a Retina screen's scale.)
fn grid(frame: &mut Pixmap, scale: f64, d: &Display, alpha: f32, (r, g, b): (u8, u8, u8)) {
    let (cols, rows) = (d.cols(), d.rows());
    if cols == 0 || rows == 0 {
        return;
    }
    let (ox, oy) = d.origin();
    let rad = (d.pitch * 0.36 * scale) as f32;
    let xs: Vec<f32> = (0..cols)
        .map(|c| ((ox + c as f64 * d.pitch) * scale) as f32)
        .collect();
    let (fw, fh) = (frame.width() as i64, frame.height() as i64);
    let left = ((xs[0] - rad - 1.0).floor() as i64).max(0);
    let right = ((xs[cols - 1] + rad + 1.0).ceil() as i64 + 1).min(fw);
    if left >= right {
        return;
    }
    let width = (right - left) as usize;
    let pad = rad.ceil() as i64 + 1;
    let tall = (2 * pad + 1) as usize;
    // A disc's cover of a pixel, its rim smoothed over a pixel (as the kit's `discs`; a small
    // disc's scaled to cover its area).
    let edge = |dx: f32, dy: f32| (rad - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);
    let k = if rad < 2.0 {
        let n = pad as i32;
        let sum: f32 = (-n..=n)
            .flat_map(|j| (-n..=n).map(move |i| (i, j)))
            .map(|(i, j)| edge(i as f32 + 0.5 - 0.5, j as f32 + 0.5 - 0.5))
            .sum();
        if sum > 0.0 {
            (std::f32::consts::PI * rad * rad / sum).min(1.0)
        } else {
            0.0
        }
    } else {
        1.0
    };
    let mut strips: Vec<Option<Vec<u8>>> = vec![None; 8];
    let data = frame.data_mut();
    for j in 0..rows {
        let y = ((oy + j as f64 * d.pitch) * scale) as f32;
        let (mut yi, mut q) = (y.floor() as i64, ((y - y.floor()) * 8.0).round() as usize);
        if q == 8 {
            (yi, q) = (yi + 1, 0);
        }
        let strip = strips[q].get_or_insert_with(|| {
            let cy = pad as f32 + q as f32 / 8.0;
            let mut s = vec![0u8; tall * width];
            for &cx in &xs {
                let (a, z) = (
                    ((cx - rad - 1.0).floor() as i64 - left).max(0) as usize,
                    (((cx + rad + 1.0).ceil() as i64 - left).max(0) as usize).min(width),
                );
                for py in 0..tall {
                    for px in a..z {
                        let c = edge((px as i64 + left) as f32 + 0.5 - cx, py as f32 + 0.5 - cy)
                            * k
                            * alpha;
                        let v = &mut s[py * width + px];
                        *v = (*v).max((c * 255.0).round() as u8);
                    }
                }
            }
            s
        });
        for py in 0..tall {
            let fy = yi - pad + py as i64;
            if !(0..fh).contains(&fy) {
                continue;
            }
            let row = fy as usize * fw as usize * 4;
            for (px, &a) in strip[py * width..(py + 1) * width].iter().enumerate() {
                if a == 0 {
                    continue;
                }
                let a = u32::from(a);
                let o = row + (left as usize + px) * 4;
                for (i, v) in [r, g, b, 255].into_iter().enumerate() {
                    let p = &mut data[o + i];
                    *p = ((u32::from(v) * a + u32::from(*p) * (255 - a) + 127) / 255) as u8;
                }
            }
        }
    }
}

/// The print: the sections' titles over their displays.
fn print_svg(dress: &Dress) -> String {
    let mut s = Svg::default();
    for (d, t) in [(SEARCH, "FIND"), (LIST, "PRESETS"), (STATUS, "PRESET")] {
        if dress.centred {
            let at = (d.x + d.w / 2.0, LABEL_Y - 4.0);
            legend(&mut s, at, t, CENTRED_LABEL, "middle", &dress.print);
        } else {
            legend(&mut s, (d.x, LABEL_Y), t, LABEL, "start", &dress.print);
        }
    }
    s.0
}

/// Action names on their caps, in clear print rather than tiny worn captions above them (in
/// every look: `drawer_legends` no longer changes where they are), in the look's `key_print`
/// and typeface. Keep the longest one, MIDI LEARN, inside the cap with the same margin as the
/// others.
fn key_print_svg(fonts: &Fonts, dress: &Dress) -> String {
    let mut s = Svg::default();
    for k in PresetKey::ALL {
        let (x, y) = k.centre();
        let size = key_legend_size(fonts, k);
        let baseline = fonts.baseline_for_centre(k.legend(), size, Weight::Bold, y);
        put!(
            s,
            "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' font-weight='700' text-anchor='middle' fill='{}'>{}</text>",
            N(x),
            N(baseline),
            N(size),
            dress.legend,
            k.legend()
        );
    }
    s.0
}

fn key_legend_size(fonts: &Fonts, k: PresetKey) -> f64 {
    let advance = fonts.advance(k.legend(), KEY_LEGEND, Weight::Bold, 0.0);
    KEY_LEGEND * ((k.size().0 - 16.0) / advance).min(1.0)
}

fn legend(s: &mut Svg, (x, y): (f64, f64), t: &str, size: f64, anchor: &str, fill: &str) {
    put!(
        s,
        "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' font-weight='700' text-anchor='{anchor}' dominant-baseline='central' fill='{fill}'>{t}</text>",
        N(x),
        N(y),
        N(size)
    );
}

impl DrawerRenderer {
    /// At `scale`: its pictures decoded and what does not change at it made on a thread of
    /// their own, begun now (as the editor opens, so the drawer's first opening waits for
    /// neither: a tenth of a second at a Retina screen's scale).
    pub fn new(scale: f64) -> Self {
        let dress = Arc::new(Dress::default());
        DrawerRenderer {
            scale,
            options: usvg::Options::default(),
            warming: warm(scale, dress.clone()),
            typed: usvg::Options::default(),
            typeface: Arc::new(Fonts::new()),
            dress,
            pictures: None,
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            shown: None,
            drawn: Default::default(),
            keys: None,
        }
    }

    /// Drawn in `look` from now on (decisions.md R-SKIN-PACKS): what it gives the drawer, the
    /// built-in ones where it gives none; what does not change made again on a thread of its
    /// own, begun now.
    pub fn set_look(&mut self, look: &crate::look::Look) {
        let dress = Dress::of(look);
        if dress.key == self.dress.key {
            return;
        }
        self.typeface = Arc::new(self.dress_fonts(&dress));
        self.dress = Arc::new(dress);
        self.typed = typed_options(&self.dress, &self.typeface);
        self.warming = warm(self.scale, self.dress.clone());
        self.pictures = None;
        self.still = None;
        self.shown = None;
        self.drawn = Default::default();
        self.keys = None;
    }

    fn dress_fonts(&self, dress: &Dress) -> Fonts {
        if dress.typed {
            dress.fonts()
        } else {
            Fonts::new()
        }
    }

    /// Where a press at `x` (panel units) across a field falls, as the dots would have it: in a
    /// look whose displays are lettered in a typeface, the place of the character gap nearest
    /// it as that typeface sets the field (so that [`caret_at`] puts the caret there); else `x`.
    pub fn press_x(&self, s: &DrawerScene, t: DrawerTarget, x: f64) -> f64 {
        let DrawerTarget::Field(id) = t else {
            return x;
        };
        let Some((d, col, _, chars)) = field_place(s, id) else {
            return x;
        };
        if !self.dress.typed {
            return x;
        }
        let f = field_of(s, id);
        let shown: Vec<char> = f
            .text
            .chars()
            .skip(shown_from(f, chars))
            .take(chars)
            .collect();
        let left = typed_left(&d, col);
        let size = typed_size(&d);
        let gap = (0..=shown.len())
            .map(|k| {
                let before: String = shown[..k].iter().collect();
                let at = left + self.typeface.advance(&before, size, Weight::Regular, 0.0);
                (k, (at - x).abs())
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(0, |(k, _)| k);
        d.at((col + 6 * gap) as f64 - 0.5, 0.0).0
    }

    pub fn rescale(&mut self, scale: f64) {
        if scale == self.scale {
            return;
        }
        self.scale = scale;
        self.still = None;
        self.shown = None;
        self.drawn = Default::default();
        self.keys = None;
    }

    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// Draws `scene`: whether the frame changed.
    pub fn render(&mut self, scene: &DrawerScene) -> bool {
        if self.shown.as_ref() == Some(scene) {
            return false;
        }
        self.shown = Some(scene.clone());
        let mut changed = false;
        if self.still.is_none() {
            let still = self.still_now();
            self.frame = Pixmap::clone(&still);
            self.still = Some(still);
            self.drawn = Default::default();
            self.keys = None;
            changed = true;
        }
        let Some(still) = self.still.clone() else {
            return changed;
        };
        let parts = [
            search_items(scene),
            tags_items(scene),
            list_items(scene),
            status_items(scene),
        ];
        for (k, (d, items)) in DISPLAYS.iter().zip(parts).enumerate() {
            let bands = match &self.drawn[k] {
                Some(was) if *was == items => continue,
                Some(was) => changed_bands(was, &items),
                None => vec![(-1.5, d.rows() as f64 + 0.5)],
            };
            if self.dress.typed {
                for band in bands {
                    let how = (self.dress.as_ref(), &self.typed, self.typeface.as_ref());
                    retype(&mut self.frame, &still, self.scale, d, &items, how, band);
                }
            } else {
                let lit = lit_dots(d, &items);
                for band in bands {
                    redraw(
                        &mut self.frame,
                        &still,
                        self.scale,
                        d,
                        &lit,
                        &self.dress,
                        band,
                    );
                }
            }
            self.drawn[k] = Some(items);
            changed = true;
        }
        let keys = keys_shown(scene);
        if self.keys.as_ref() != Some(&keys) {
            let (w, h) = (KEY.0 / 2.0 + 6.0, KEY.1 / 2.0 + 8.0);
            restore(
                &mut self.frame,
                &still,
                self.scale,
                (KEY_X[0] - w, 0.0, KEY_X[3] + w, KEY_Y[1] + h),
            );
            let mut o = Svg::default();
            for k in &keys.held {
                let (x, y) = k.centre();
                let (w, h) = k.size();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#000' fill-opacity='0.34'/><rect x='{}' y='{}' width='{}' height='5' rx='2.5' fill='#000' fill-opacity='0.3'/>",
                    N(x - w / 2.0 + 2.0),
                    N(y - h / 2.0 + 2.0),
                    N(w - 4.0),
                    N(h - 4.0),
                    N(x - w / 2.0 + 6.0),
                    N(y - h / 2.0 + 3.0),
                    N(w - 12.0)
                );
            }
            for k in &keys.unavailable {
                let (x, y) = k.centre();
                let (w, h) = k.size();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#000' fill-opacity='0.32'/>",
                    N(x - w / 2.0),
                    N(y - h / 2.0),
                    N(w),
                    N(h)
                );
            }
            if let Some(k) = keys.pointed {
                let (x, y) = k.centre();
                let (w, h) = k.size();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#fff' fill-opacity='0.07'/>",
                    N(x - w / 2.0 + 2.0),
                    N(y - h / 2.0 + 2.0),
                    N(w - 4.0),
                    N(h - 4.0)
                );
            }
            if !o.0.is_empty() {
                svg_over(&self.options, &mut self.frame, self.scale, &o.0);
            }
            self.keys = Some(keys);
            changed = true;
        }
        changed
    }

    /// What does not change at this scale: the thread's, made as the editor opened (waited for
    /// if it is not done), if it is still the scale; else made now.
    fn still_now(&mut self) -> Arc<Pixmap> {
        if let Some(Ok((pictures, scale, key, still))) = self.warming.take().map(JoinHandle::join)
            && key == self.dress.key
        {
            self.pictures = Some(pictures);
            if scale == self.scale {
                return still;
            }
        }
        let dress = self.dress.clone();
        let pictures = self
            .pictures
            .get_or_insert_with(|| Arc::new(Pictures::load(&dress)));
        shared_still(self.scale, pictures, &dress)
    }
}

/// The pictures decoded and what does not change at `scale` in `dress` made, on a thread of
/// their own.
fn warm(scale: f64, dress: Arc<Dress>) -> Option<JoinHandle<Warmed>> {
    std::thread::Builder::new()
        .name("ca72-drawer".into())
        .spawn(move || {
            let pictures = Pictures::load(&dress);
            let still = shared_still(scale, &pictures, &dress);
            (Arc::new(pictures), scale, dress.key, still)
        })
        .ok()
}

/// For lettering the displays in `dress`'s typeface (`fonts`).
fn typed_options(dress: &Dress, fonts: &Fonts) -> usvg::Options<'static> {
    if !dress.typed {
        return usvg::Options::default();
    }
    usvg::Options {
        fontdb: fonts.database(),
        font_family: FAMILY.into(),
        ..usvg::Options::default()
    }
}

/// How strongly a line not lit shows in a typeface, as bright as the plug-in makes it: what is
/// said aside (DIM) nearly as the list, only what is barely there (a field's placeholder)
/// faint. (Scaled as the dots' brightness, the dim lines had been grey beside the list: the
/// reviewing agent, 2026-10-17.)
fn typed_alpha(lv: f32) -> f64 {
    if lv >= POINTED {
        1.0
    } else if lv >= NORMAL {
        0.92
    } else if lv >= DIM {
        0.84
    } else {
        0.5
    }
}

/// A display's lettering's size in a typeface: its capitals about the dots' seven rows.
fn typed_size(d: &Display) -> f64 {
    9.6 * d.pitch
}

/// Where a field's characters from its first column `col` begin, set in a typeface.
fn typed_left(d: &Display, col: usize) -> f64 {
    d.at(col as f64 - 0.5, 0.0).0
}

/// What never changes at `scale`: the face, as the strip's under it (the picture 640 units
/// across, mirrored tile to tile, placed as the strip's), its trim and the rail's shadow; the
/// print, worn into it; the displays' bezels and glass, every dot faintly there; the keys' caps.
fn make_still(scale: f64, pictures: &Pictures, dress: &Dress) -> Pixmap {
    let s = scale;
    let fonts = dress.fonts();
    let options = usvg::Options {
        fontdb: fonts.database(),
        font_family: FAMILY.into(),
        ..usvg::Options::default()
    };
    let (w, h) = size_at(s);
    let mut still = Pixmap::new(w, h).expect("a frame of at least a pixel");
    // (Scaled to its size once, bilinearly, then laid tile to tile as it is: drawn out from
    // the picture as it was, a third of a second at a Retina screen's scale.)
    let k = 640.0 * s / f64::from(pictures.face.width());
    let (tw, th) = (
        (f64::from(pictures.face.width()) * k).round().max(1.0) as u32,
        (f64::from(pictures.face.height()) * k).round().max(1.0) as u32,
    );
    let mut face = Pixmap::new(tw, th).expect("a picture of at least a pixel");
    face.draw_pixmap(
        0,
        0,
        pictures.face.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        Transform::from_scale(
            tw as f32 / pictures.face.width() as f32,
            th as f32 / pictures.face.height() as f32,
        ),
        None,
    );
    tile_reflected(&mut still, &face, ((320.0 * s).round() as i64, 0));
    svg_over(&options, &mut still, s, &plate_svg());
    // A look's drawing over the face, under the print, the glass and the keys.
    if let Some(art) = &dress.art {
        let size = art.size();
        if let Some(mut layer) = Pixmap::new(w, h)
            && skin_pack::svg::render(
                art,
                Transform::from_scale(
                    (W * s) as f32 / size.width(),
                    (DRAWER_H * s) as f32 / size.height(),
                ),
                &mut layer.as_mut(),
            )
        {
            still.draw_pixmap(
                0,
                0,
                layer.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
    }
    // The print, in the band across the top it is in.
    let band = ((PRINT_H * s).ceil() as u32).min(h);
    if let (Some(mut print), Some(under)) = (
        Pixmap::new(w, band),
        IntRect::from_xywh(0, 0, w, band).and_then(|r| still.clone_rect(r)),
    ) {
        svg_over(&options, &mut print, s, &print_svg(dress));
        crate::render::worn_into(&mut print, &under);
        still.draw_pixmap(
            0,
            0,
            print.as_ref(),
            &PixmapPaint {
                opacity: 0.94,
                ..PixmapPaint::default()
            },
            Transform::identity(),
            None,
        );
    }
    for d in DISPLAYS {
        glass(&mut still, &pictures.glass, s, &d);
        // (Lettered in a typeface, a display has no dots to show unlit.)
        if !dress.typed {
            grid(&mut still, s, &d, UNLIT, dress.dot);
        }
    }
    let cap = widened(&pictures.button, KEY.0 / KEY.1);
    let init_size = PresetKey::Init.size();
    let init_cap = widened(&pictures.button, init_size.0 / init_size.1);
    for k in PresetKey::ALL {
        let picture = if k == PresetKey::Init {
            &init_cap
        } else {
            &cap
        };
        sprite(&mut still, picture, s, k.centre(), k.size());
    }
    svg_over(&options, &mut still, s, &key_print_svg(&fonts, dress));
    still
}

/// Only the titles are worn into the face; the action names are clear on the key caps.
const PRINT_H: f64 = 50.0;

#[derive(Clone, Copy, Debug, Default)]
struct LitDot {
    lv: f32,
    ink: Ink,
}

/// Each dot of a display lit by `items`, as bright as the brightest over it (none: 0).
/// Gold wins a tie, so hovering the selected star cannot turn it back to ordinary orange.
fn lit_dots(d: &Display, items: &[Item]) -> Vec<LitDot> {
    let (cols, rows) = (d.cols(), d.rows());
    let mut lit = vec![LitDot::default(); cols * rows];
    for it in items {
        for (n, ch) in it.text.chars().enumerate() {
            let g = glyph(ch);
            for (j, bits) in g.iter().enumerate() {
                for b in 0..5 {
                    let (c, row) = (it.col + 6 * n + b, it.row + j);
                    if bits >> (4 - b) & 1 == 1 && c < cols && row < rows {
                        let v = &mut lit[row * cols + c];
                        if it.lv > v.lv || (it.lv == v.lv && it.ink > v.ink) {
                            *v = LitDot {
                                lv: it.lv,
                                ink: it.ink,
                            };
                        }
                    }
                }
            }
        }
    }
    lit
}

/// The bands of a display's dot rows (from, to: a row's middle a whole number) that changed:
/// each line whose characters changed, a dot round it (its light reaches into the lines
/// beside it), the bands that meet made one.
fn changed_bands(was: &[Item], now: &[Item]) -> Vec<(f64, f64)> {
    let lines = |items: &[Item]| {
        let mut m: BTreeMap<usize, Vec<Item>> = BTreeMap::new();
        for it in items {
            m.entry(it.row).or_default().push(it.clone());
        }
        m
    };
    let (a, b) = (lines(was), lines(now));
    let mut rows: Vec<usize> = a
        .keys()
        .chain(b.keys())
        .copied()
        .filter(|r| a.get(r) != b.get(r))
        .collect();
    rows.sort_unstable();
    rows.dedup();
    let mut bands: Vec<(f64, f64)> = Vec::new();
    for r in rows {
        let (from, to) = (r as f64 - 1.5, (r + LINE) as f64 + 0.5);
        match bands.last_mut() {
            Some(last) if from <= last.1 => last.1 = last.1.max(to),
            _ => bands.push((from, to)),
        }
    }
    bands
}

/// A band of a display's dot rows drawn again: the still under it, then every lit dot whose
/// light reaches it (drawn apart, then put in the frame, so a dot across its edge is not drawn
/// twice over what is there; apart a little beyond the band above and below, as the kit's
/// `discs` covers a small dot cut off at an edge differently).
fn redraw(
    frame: &mut Pixmap,
    still: &Pixmap,
    scale: f64,
    d: &Display,
    lit: &[LitDot],
    dress: &Dress,
    (from, to): (f64, f64),
) {
    let (_, oy) = d.origin();
    let pad = 4.0;
    let (fw, fh) = (i64::from(frame.width()), i64::from(frame.height()));
    let px = |v: f64| (v * scale).round() as i64;
    let (x0, x1) = (px(d.x - pad).max(0), px(d.x + d.w + pad).min(fw));
    let y0 = px((oy + from * d.pitch).max(d.y - pad)).max(0);
    let y1 = px((oy + to * d.pitch).min(d.y + d.h + pad)).min(fh);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    // Nearly joined at the minimum width: a lit stroke keeps its coverage when each dot is
    // about a pixel across, instead of becoming a faint scatter of separate pinpoints.
    let r = (d.pitch * 0.45 * scale) as f32;
    let margin = (2.1 * r).ceil() as i64 + 2;
    let (top, foot) = ((y0 - margin).max(0), (y1 + margin).min(fh));
    let (bw, bh) = ((x1 - x0) as usize, (foot - top) as usize);
    let Some(mut band) = Pixmap::new(bw as u32, bh as u32) else {
        return;
    };
    let row = fw as usize * 4;
    for y in 0..bh {
        let i = (top as usize + y) * row + x0 as usize * 4;
        band.data_mut()[y * bw * 4..(y + 1) * bw * 4].copy_from_slice(&still.data()[i..i + bw * 4]);
    }
    let mut cover = vec![0u8; bw * bh];
    let (cols, rows) = (d.cols(), d.rows());
    let first = (from - 1.0).floor().max(0.0) as usize;
    let last = ((to + 1.0).ceil().max(0.0) as usize).min(rows);
    let (ox, _) = d.origin();
    // Half the preceding brightness step permits a midpoint without rounding ordinary
    // text all the way back. Existing active levels still land on their exact old steps.
    let mut by: BTreeMap<(Ink, u8), Vec<Disc>> = BTreeMap::new();
    for j in first..last {
        for c in 0..cols {
            let LitDot { lv: v, ink } = lit[j * cols + c];
            if v > 0.0 {
                let x = ((ox + c as f64 * d.pitch) * scale) as f32 - x0 as f32;
                let y = ((oy + j as f64 * d.pitch) * scale) as f32 - top as f32;
                by.entry((ink, (v * 40.0).round() as u8))
                    .or_default()
                    .push((x, y, r));
            }
        }
    }
    for (&(_, q), ds) in &by {
        let glows: Vec<Disc> = ds.iter().map(|&(x, y, r)| (x, y, r * 2.1)).collect();
        discs(
            &mut band,
            &mut cover,
            &glows,
            dress.glow,
            0.08 * f32::from(q) / 40.0,
        );
    }
    for (&(ink, q), ds) in &by {
        let on = ink.in_dress(dress);
        discs(&mut band, &mut cover, ds, on, f32::from(q) / 40.0);
    }
    for y in y0..y1 {
        let (i, b) = (
            y as usize * row + x0 as usize * 4,
            (y - top) as usize * bw * 4,
        );
        frame.data_mut()[i..i + bw * 4].copy_from_slice(&band.data()[b..b + bw * 4]);
    }
}

/// A band of a display drawn again in a look's typeface (`drawer_lettering = "type"`; its rows
/// as [`redraw`]'s): the still under it, then every item whose line reaches it, as text in the
/// window, put in the frame. (Drawn apart a little beyond the band, so a letter across its edge
/// is drawn as it was.)
fn retype(
    frame: &mut Pixmap,
    still: &Pixmap,
    scale: f64,
    d: &Display,
    items: &[Item],
    (dress, options, fonts): (&Dress, &usvg::Options<'_>, &Fonts),
    (from, to): (f64, f64),
) {
    let (_, oy) = d.origin();
    let pad = 4.0;
    let (fw, fh) = (i64::from(frame.width()), i64::from(frame.height()));
    let px = |v: f64| (v * scale).round() as i64;
    let (x0, x1) = (px(d.x - pad).max(0), px(d.x + d.w + pad).min(fw));
    let y0 = px((oy + from * d.pitch).max(d.y - pad)).max(0);
    let y1 = px((oy + to * d.pitch).min(d.y + d.h + pad)).min(fh);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let margin = px(3.0 * d.pitch) + 2;
    let (top, foot) = ((y0 - margin).max(0), (y1 + margin).min(fh));
    let (bw, bh) = ((x1 - x0) as usize, (foot - top) as usize);
    let Some(mut band) = Pixmap::new(bw as u32, bh as u32) else {
        return;
    };
    let row = fw as usize * 4;
    for y in 0..bh {
        let i = (top as usize + y) * row + x0 as usize * 4;
        band.data_mut()[y * bw * 4..(y + 1) * bw * 4].copy_from_slice(&still.data()[i..i + bw * 4]);
    }
    let (first, last) = (from - 2.0, to + 2.0);
    let mut s = Svg::default();
    put!(
        s,
        "<defs><clipPath id='window'><rect x='{}' y='{}' width='{}' height='{}'/></clipPath></defs><g clip-path='url(#window)'>",
        N(d.x),
        N(d.y),
        N(d.w),
        N(d.h)
    );
    for it in items {
        let (r0, r1) = (it.row as f64, (it.row + LINE) as f64);
        if r1 >= first && r0 <= last {
            // A field's caret after the characters before it, as the typeface sets them.
            let caret_x = (it.field && it.text == CARET.to_string()).then(|| {
                let text = items
                    .iter()
                    .find(|o| o.field && o.row == it.row && o.col <= it.col && o.text != it.text);
                let left = typed_left(d, text.map_or(it.col, |o| o.col));
                let before: String = text.map_or(String::new(), |o| {
                    o.text.chars().take((it.col - o.col) / 6).collect()
                });
                left + fonts.advance(&before, typed_size(d), Weight::Regular, 0.0)
            });
            typed(&mut s, d, it, dress, caret_x);
        }
    }
    put!(s, "</g>");
    let view = [
        x0 as f64 / scale,
        top as f64 / scale,
        bw as f64 / scale,
        bh as f64 / scale,
    ];
    let doc = art::document(view, bw as u32, bh as u32, &s.0);
    if let Ok(tree) = usvg::Tree::from_str(&doc, options) {
        resvg::render(&tree, Transform::identity(), &mut band.as_mut());
    }
    for y in y0..y1 {
        let (i, b) = (
            y as usize * row + x0 as usize * 4,
            (y - top) as usize * bw * 4,
        );
        frame.data_mut()[i..i + bw * 4].copy_from_slice(&band.data()[b..b + bw * 4]);
    }
}

/// An item in a look's typeface: lit (as bright as the brightest) in its lamps' colour, else
/// in its print's, fainter as the item is; its text set from its first character's place (a
/// field's each character in its own), its signs drawn (the stars, the arrows, the current
/// one's mark and the one under the pointer, a chosen tag's bar, the caret, a chosen tab's
/// underline).
fn typed(s: &mut Svg, d: &Display, it: &Item, dress: &Dress, caret_x: Option<f64>) {
    let lit = it.lv >= BRIGHT;
    let hex = |(r, g, b): (u8, u8, u8)| format!("#{r:02x}{g:02x}{b:02x}");
    // Lit: the lamps' colour, or the current preset's.
    let lamp = hex(it.ink.in_dress(dress));
    let p = d.pitch;
    let at = |c: f64, r: f64| d.at(c, r);
    // A chosen tab's underline: one line under its characters' cells.
    if !it.field && !it.text.is_empty() && it.text.chars().all(|c| c == CARET) {
        let (x, y) = at(it.col as f64 - 0.5, it.row as f64 + 8.0);
        put!(
            s,
            "<rect x='{}' y='{}' width='{}' height='{}' fill='{lamp}'/>",
            N(x),
            N(y - 0.4 * p),
            N((6 * it.text.chars().count()) as f64 * p - p),
            N(0.8 * p)
        );
        return;
    }
    // (What is typed in a field in the print's colour, fully; its caret lit.)
    let typing = it.field && caret_x.is_none();
    let (fill, alpha) = if typing && lit {
        (dress.unlit.clone(), 1.0)
    } else if lit {
        (lamp.clone(), 1.0)
    } else {
        (dress.unlit.clone(), typed_alpha(it.lv))
    };
    // A place in dots (as `Display::at`): a character's cell from `col - 0.5` to `col + 5.5`,
    // its capitals from row 0 to 7, its line's foot at row 6.5.
    let size = typed_size(d);
    let glow = if lit && !typing {
        format!(
            " stroke='{}' stroke-opacity='0.22' stroke-width='{}' paint-order='stroke'",
            hex(dress.glow),
            N(0.55 * p)
        )
    } else {
        String::new()
    };
    let text = |s: &mut Svg, x: f64, t: &str| {
        let (_, y) = at(0.0, it.row as f64 + 6.5);
        put!(
            s,
            "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' fill='{fill}' fill-opacity='{}'{glow}>{}</text>",
            N(x),
            N(y),
            N(size),
            N(alpha),
            crate::svg::escape(t)
        );
    };
    let chars: Vec<char> = it.text.chars().collect();
    let col = |k: usize| (it.col + 6 * k) as f64;
    let mut run = String::new();
    let mut run_at = 0;
    let flush = |s: &mut Svg, run: &mut String, run_at: usize| {
        let t = run.trim_end();
        if !t.is_empty() {
            let lead = t.chars().take_while(|c| *c == ' ').count();
            text(
                s,
                typed_left(d, it.col + 6 * (run_at + lead)),
                t.trim_start(),
            );
        }
        run.clear();
    };
    for (k, &c) in chars.iter().enumerate() {
        let sign = matches!(
            c,
            '\u{2605}' | '\u{2606}' | '\u{25b2}' | '\u{25bc}' | CARET | ROW_SELECTED | TAG_SELECTED
        ) || (c == '>' && chars.len() == 1);
        if !sign {
            if run.is_empty() {
                run_at = k;
            }
            run.push(c);
            continue;
        }
        flush(s, &mut run, run_at);
        let (cx, cy) = at(col(k) + 2.0, it.row as f64 + 3.0);
        match c {
            '\u{2605}' | '\u{2606}' => {
                let pts: Vec<String> = (0..10)
                    .map(|i| {
                        let a = std::f64::consts::PI * (f64::from(i) / 5.0 - 0.5);
                        let r = if i % 2 == 0 { 3.4 * p } else { 1.4 * p };
                        format!("{},{}", N(cx + r * a.cos()), N(cy + 0.25 * p + r * a.sin()))
                    })
                    .collect();
                if c == '\u{2605}' {
                    put!(
                        s,
                        "<polygon points='{}' fill='{lamp}' fill-opacity='{}'/>",
                        pts.join(" "),
                        N(alpha.max(0.85))
                    );
                } else {
                    put!(
                        s,
                        "<polygon points='{}' fill='none' stroke='{fill}' stroke-opacity='{}' stroke-width='{}' stroke-linejoin='round'/>",
                        pts.join(" "),
                        N(alpha),
                        N(0.42 * p)
                    );
                }
            }
            '\u{25b2}' | '\u{25bc}' => {
                let k = if c == '\u{25b2}' { -1.0 } else { 1.0 };
                put!(
                    s,
                    "<path d='M{} {}L{} {}L{} {}Z' fill='{}'/>",
                    N(cx - 2.2 * p),
                    N(cy - k * 1.6 * p),
                    N(cx + 2.2 * p),
                    N(cy - k * 1.6 * p),
                    N(cx),
                    N(cy + k * 1.8 * p),
                    hex(dress.dot)
                );
            }
            ROW_SELECTED => {
                // (Its line's band first, under what follows: in the look's `selection`
                // colour, else faintly in the lamps' glow.)
                let (_, top) = at(0.0, it.row as f64 - 1.2);
                let (band, opacity) = match dress.selection {
                    Some(c) => (hex(c), 0.9),
                    None => (hex(dress.glow), 0.1),
                };
                put!(
                    s,
                    "<rect x='{}' y='{}' width='{}' height='{}' fill='{band}' fill-opacity='{}'/>",
                    N(d.x + 3.0),
                    N(top),
                    N(d.w - 6.0),
                    N(9.4 * p),
                    N(opacity)
                );
                let (x, y) = at(col(k), it.row as f64 + 3.0);
                put!(
                    s,
                    "<path d='M{} {}L{} {}L{} {}Z' fill='{}'/>",
                    N(x - 0.3 * p),
                    N(y - 2.8 * p),
                    N(x + 3.8 * p),
                    N(y),
                    N(x - 0.3 * p),
                    N(y + 2.8 * p),
                    lamp
                );
            }
            // The row under the pointer's: an arrow's outline.
            '>' => {
                let (x, y) = at(col(k), it.row as f64 + 3.0);
                put!(
                    s,
                    "<path d='M{} {}L{} {}L{} {}Z' fill='none' stroke='{fill}' stroke-opacity='{}' stroke-width='{}' stroke-linejoin='round'/>",
                    N(x),
                    N(y - 2.5 * p),
                    N(x + 3.5 * p),
                    N(y),
                    N(x),
                    N(y + 2.5 * p),
                    N(alpha),
                    N(0.42 * p)
                );
            }
            TAG_SELECTED => {
                let (x, y) = at(col(k) - 0.4, it.row as f64 - 0.4);
                put!(
                    s,
                    "<rect x='{}' y='{}' width='{}' height='{}' fill='{lamp}'/>",
                    N(x),
                    N(y),
                    N(0.8 * p),
                    N(6.8 * p)
                );
            }
            CARET => {
                let (x, y) = at(col(k) - 0.5, it.row as f64 - 0.5);
                let x = caret_x.unwrap_or(x);
                put!(
                    s,
                    "<rect x='{}' y='{}' width='{}' height='{}' fill='{}'/>",
                    N(x - 0.2 * p),
                    N(y - 0.6 * p),
                    N(0.4 * p),
                    N(8.8 * p),
                    lamp
                );
            }
            _ => {}
        }
    }
    flush(s, &mut run, run_at);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::{Current, Row};

    fn scene() -> DrawerScene {
        DrawerScene {
            chips: ["bass", "brass", "dark", "lead", "pad"]
                .iter()
                .map(|t| ((*t).to_owned(), *t == "lead"))
                .collect(),
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

    /// Each control is found where it is drawn: the search, the tabs, the tags, a row and its
    /// star, each key, the update check's button when it has one; scrolled, the rows shown; a
    /// field only while what it is for goes on.
    #[test]
    fn the_drawer_finds_each_control_where_it_is_drawn() {
        let s = scene();
        let at = |s: &DrawerScene, (x, y): (f64, f64)| drawer_hit(s, x, y);
        assert_eq!(
            at(&s, search_centre()),
            Some(DrawerTarget::Field(FieldId::Search))
        );
        for t in [Tab::All, Tab::Favourites, Tab::Mine] {
            assert_eq!(at(&s, tab_centre(t)), Some(DrawerTarget::Tab(t)));
        }
        for i in 0..5 {
            let c = chip_centre(&s, i).expect("shown");
            assert_eq!(at(&s, c), Some(DrawerTarget::Chip(i)), "{i}");
        }
        assert_eq!(chip_centre(&s, 5), None);
        for i in [0, 3, ROWS_SHOWN - 1] {
            let r = row_centre(&s, i).expect("shown");
            assert_eq!(at(&s, r), Some(DrawerTarget::Row(i)));
            let star = star_centre(&s, i).expect("shown");
            assert_eq!(at(&s, star), Some(DrawerTarget::Star(i)));
        }
        assert_eq!(row_centre(&s, ROWS_SHOWN), None);
        for k in PresetKey::ALL {
            assert_eq!(at(&s, key_centre(k)), Some(DrawerTarget::Key(k)), "{k:?}");
        }
        assert_eq!(at(&s, update_centre()), Some(DrawerTarget::Back));
        let mut news = scene();
        news.update.button = "DOWNLOAD".into();
        assert_eq!(at(&news, update_centre()), Some(DrawerTarget::Update));
        // Scrolled: the first row shown is the list's `first`, where the first was.
        let mut scrolled = scene();
        scrolled.first = 10;
        assert_eq!(row_centre(&scrolled, 10), row_centre(&s, 0));
        assert_eq!(
            at(&scrolled, row_centre(&s, 0).unwrap()),
            Some(DrawerTarget::Row(10))
        );
        // A row past the list's end is nothing.
        let mut few = scene();
        few.rows.truncate(2);
        assert_eq!(
            at(&few, row_centre(&s, 5).unwrap()),
            Some(DrawerTarget::Back)
        );
        // The fields of what a key began, only while it goes on.
        assert_eq!(field_centre(&s, FieldId::Edit), None);
        assert_eq!(field_centre(&s, FieldId::SaveName), None);
        let mut renaming = scene();
        renaming.editing = Some(Edit::Rename);
        let c = field_centre(&renaming, FieldId::Edit).expect("shown");
        assert_eq!(at(&renaming, c), Some(DrawerTarget::Field(FieldId::Edit)));
        let mut saving = scene();
        saving.editing = Some(Edit::Save);
        for id in [FieldId::SaveName, FieldId::SaveTags] {
            let c = field_centre(&saving, id).expect("shown");
            assert_eq!(at(&saving, c), Some(DrawerTarget::Field(id)));
        }
        assert_eq!(drawer_hit(&s, 10.0, DRAWER_H + 1.0), None);
        const { assert!(ROWS_SHOWN >= 15 && TAG_LINES >= 16) };
    }

    /// Init is available before any preset is selected, while editing, and in MIDI Learn.
    #[test]
    fn init_is_found_without_a_preset_or_selection() {
        for s in [
            DrawerScene::default(),
            DrawerScene {
                editing: Some(Edit::Save),
                ..DrawerScene::default()
            },
            DrawerScene {
                midi: Some(MidiList::default()),
                ..DrawerScene::default()
            },
        ] {
            let (x, y) = key_centre(PresetKey::Init);
            assert_eq!(
                drawer_hit(&s, x, y),
                Some(DrawerTarget::Key(PresetKey::Init))
            );
            assert!(y + PresetKey::Init.size().1 / 2.0 < KEY_Y[0] - KEY.1 / 2.0);
        }
    }

    /// The enlarged captions stay inside their buttons; their hit regions stay separate at
    /// the minimum supported width too. Init keeps the existing actions in their places.
    #[test]
    fn readable_key_captions_fit_separate_buttons() {
        let fonts = Fonts::new();
        for (i, k) in PresetKey::ALL.into_iter().enumerate() {
            let (x, y) = k.centre();
            let (w, h) = k.size();
            let size = key_legend_size(&fonts, k);
            assert!(size >= 25.0, "{k:?}: {size}");
            assert!(fonts.advance(k.legend(), size, Weight::Bold, 0.0) <= w - 15.9);
            assert!(x - w / 2.0 > 0.0 && x + w / 2.0 < W);
            assert!(y - h / 2.0 > 0.0 && y + h / 2.0 < STATUS.y);
            for other in &PresetKey::ALL[i + 1..] {
                let (ox, oy) = other.centre();
                let (ow, oh) = other.size();
                assert!(
                    (x - ox).abs() > (w + ow) / 2.0 + 8.0 || (y - oy).abs() > (h + oh) / 2.0 + 8.0,
                    "{k:?} overlaps {other:?}"
                );
            }
        }
    }

    /// Each selected tag's marker leaves a clear gap before its caption in both columns;
    /// even a full-length caption remains inside the display.
    #[test]
    fn selected_tag_markers_stay_clear_of_both_captions() {
        let s = DrawerScene {
            chips: vec![("abcdefghijklm".into(), true); 2],
            ..DrawerScene::default()
        };
        let items = tags_items(&s);
        for col in TAG_COL {
            let caption = items.iter().find(|it| it.col == col).expect("tag caption");
            let marker = items
                .iter()
                .find(|it| it.col == col - 3)
                .expect("selected marker");
            let occupied = |it: &Item| {
                it.text
                    .chars()
                    .enumerate()
                    .flat_map(|(n, ch)| {
                        glyph(ch).into_iter().flat_map(move |bits| {
                            (0..5)
                                .filter(move |b| bits >> (4 - b) & 1 == 1)
                                .map(move |b| it.col + 6 * n + b)
                        })
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(caption.text, "abcdefghijklm");
            assert!(
                occupied(marker).into_iter().all(|c| c + 1 < caption.col),
                "marker overlaps the caption at column {col}"
            );
            assert!(
                occupied(caption)
                    .into_iter()
                    .chain(occupied(marker))
                    .all(|c| c < TAGS.cols())
            );
            assert!(caption.row + LINE <= TAGS.rows());
        }
    }

    /// Every field, action and selection transition is the same when drawn in parts as in a
    /// new frame, including the minimum width and the largest supported width.
    #[test]
    fn readable_states_redraw_as_a_whole_frame() {
        for scale in [860.0 / W, 1380.0 / W, 1.0] {
            let base = scene();
            let states = [
                base.clone(),
                DrawerScene {
                    focus: Some(FieldId::Search),
                    search: Field::new("bass"),
                    favourites: true,
                    hover: Some(DrawerTarget::Key(PresetKey::SaveAs)),
                    ..base.clone()
                },
                DrawerScene {
                    hover: Some(DrawerTarget::Key(PresetKey::Init)),
                    ..base.clone()
                },
                DrawerScene {
                    editing: Some(Edit::Save),
                    save_name: Field::new("New sound"),
                    focus: Some(FieldId::SaveName),
                    ..base.clone()
                },
                DrawerScene::default(),
                base,
            ];
            let mut parts = DrawerRenderer::new(scale);
            for state in states {
                parts.render(&state);
                let mut whole = DrawerRenderer::new(scale);
                whole.render(&state);
                assert_eq!(
                    parts.frame().data(),
                    whole.frame().data(),
                    "changed parts differ at scale {scale}"
                );
            }
        }
    }

    /// MIDI Learn's list in the list's place: its rows and their words found where drawn; the
    /// tags then show its keys, and no field of the presets'.
    #[test]
    fn the_midi_list_is_found_where_it_is_drawn() {
        let mut s = scene();
        let m = MidiList {
            rows: (0..40)
                .map(|i| MidiRow {
                    name: format!("CONTROL {i}"),
                    assignment: if i % 2 == 0 {
                        format!("CH 1 \u{b7} CC {i}")
                    } else {
                        String::new()
                    },
                    waiting: i == 5,
                })
                .collect(),
            chosen: 2,
            keys: "UP, DOWN: CHOOSE \u{b7} ENTER: LEARN".into(),
            ..MidiList::default()
        };
        s.midi = Some(m.clone());
        let at = |(x, y): (f64, f64)| drawer_hit(&s, x, y);
        for i in [0, 2, 5] {
            assert_eq!(
                at(midi_row_centre(&m, i).expect("shown")),
                Some(DrawerTarget::MidiRow(i))
            );
        }
        for (i, a) in [
            (2, MidiAction::Learn),
            (2, MidiAction::Remove),
            (3, MidiAction::Learn),
            (5, MidiAction::Cancel),
        ] {
            let c = midi_action_centre(&m, i, a).expect("it has it");
            assert_eq!(at(c), Some(DrawerTarget::MidiAction(i, a)), "{i} {a:?}");
        }
        assert_eq!(midi_action_centre(&m, 3, MidiAction::Remove), None);
        assert_eq!(
            at(chip_centre(&scene(), 0).unwrap()),
            Some(DrawerTarget::Back)
        );
        let tags = tags_items(&s);
        assert_eq!(tags[0].text, "UP, DOWN: CHOOSE");
        assert_eq!(
            at(midi_centre()),
            Some(DrawerTarget::Key(PresetKey::MidiLearn))
        );
    }

    /// A click in a field puts the caret at the nearest gap, counted from the first character
    /// shown.
    #[test]
    fn a_click_puts_the_caret_at_the_nearest_gap() {
        let mut s = scene();
        s.search = Field::new("Deep Bass");
        let (ox, _) = SEARCH.origin();
        let gap = |k: f64| ox + (SEARCH_AT.0 as f64 + 6.0 * k - 0.5) * SEARCH.pitch;
        assert_eq!(caret_at(&s, FieldId::Search, gap(0.0)), 0);
        assert_eq!(caret_at(&s, FieldId::Search, gap(4.0) + 3.0), 4);
        assert_eq!(caret_at(&s, FieldId::Search, gap(4.0) - 3.0), 4);
        assert_eq!(caret_at(&s, FieldId::Search, 5000.0), 9);
        // A long name, its end in view: the first character shown counts.
        let long: String = "x".repeat(40);
        s.search = Field::new(&long);
        let n = SEARCH.chars(SEARCH_AT.0);
        assert_eq!(caret_at(&s, FieldId::Search, gap(0.0)), 40 - (n - 1));
    }

    /// Lines broken at spaces (a word longer than a line cut where it must be); a name cut
    /// with an ellipsis; the update check's words on two lines at most.
    #[test]
    fn words_wrap_and_cut() {
        assert_eq!(
            wrap("CLOSE THE DAW, THEN INSTALL IT", 22),
            ["CLOSE THE DAW, THEN", "INSTALL IT"]
        );
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
        assert!(wrap("  ", 5).is_empty());
        assert_eq!(cut("Ringing Saw Line", 8), "Ringing\u{2026}");
        assert_eq!(cut("Bass", 8), "Bass");
        assert_eq!(cut("Ba\u{7}ss", 8), "Bass");
        let mut u = UpdateScene {
            text: "0.2.0 IS OUT (THIS IS 0.1.5)".into(),
            tone: Tone::News,
            button: "DOWNLOAD".into(),
        };
        assert!(update_fits(&u));
        u.text = "A ".repeat(40);
        assert!(!update_fits(&u));
    }

    /// A row shows its mark, its star, its name, where it is from and its tags, each in its own
    /// columns.
    #[test]
    fn a_row_shows_its_parts_in_their_columns() {
        let mut s = scene();
        s.rows[2].name = "A Name Far Too Long To Show Whole".into();
        let items = list_items(&s);
        // A row's line as the dots show it: each of its items' characters in its place.
        let row = |k: usize| {
            let mut line = vec![' '; LIST.chars(COL)];
            let mut found = false;
            for it in items.iter().filter(|it| it.row == list_row(k)) {
                found = true;
                for (i, c) in it.text.chars().enumerate() {
                    line[(it.col - COL) / 6 + i] = c;
                }
            }
            assert!(found, "a row");
            line.into_iter().collect::<String>()
        };
        let chars =
            |t: &str, from: usize, n: usize| t.chars().skip(from).take(n).collect::<String>();
        let current = row(1);
        assert_eq!(chars(&current, 0, 3), "\u{25b6} \u{2606}");
        assert_eq!(chars(&current, NAME_AT, 9), "Preset 1 ");
        let edited = row(2);
        assert_eq!(
            chars(&edited, NAME_AT, NAME_CHARS),
            "A Name Far Too Long \u{2026}"
        );
        assert_eq!(chars(&edited, NAME_AT + NAME_CHARS + 1, 6), "EDITED");
        assert_eq!(chars(&edited, NAME_AT + NAME_CHARS + 8, 9), "bass dark");
        assert_eq!(chars(&row(0), 2, 1), "\u{2605}");
    }

    /// Selection is gold with a filled marker; hover is brighter orange with an outline
    /// marker; ordinary presets keep the user's darker orange and unmarked rows.
    #[test]
    fn selected_hovered_and_ordinary_presets_have_distinct_treatments() {
        let mut s = scene();
        s.hover = Some(DrawerTarget::Row(3));
        let items = list_items(&s);
        // A row's name, and its mark (none: unmarked).
        let at = |i, col| {
            items
                .iter()
                .find(|it| it.row == list_row(i) && it.col == col)
        };
        let row = |i| at(i, COL + 6 * NAME_AT).expect("preset row");
        let mark = |i| at(i, COL).map(|it| it.text.clone());
        let (selected, hovered, ordinary) = (row(1), row(3), row(7));
        assert_eq!(selected.ink.colour(), (255, 197, 110));
        assert_eq!(selected.lv, BRIGHT);
        assert_eq!(mark(1), Some(ROW_SELECTED.to_string()));
        assert_eq!(hovered.ink.colour(), (255, 140, 58));
        assert_eq!(hovered.lv, POINTED);
        assert_eq!(mark(3).as_deref(), Some(">"));
        assert_eq!(ordinary.ink.colour(), (255, 140, 58));
        assert_eq!(ordinary.lv, 0.65);
        assert_eq!(mark(7), None);
        assert!(hovered.lv > ordinary.lv);
        // Hovering its star must not repaint part of the current row in orange.
        s.hover = Some(DrawerTarget::Star(1));
        assert!(
            list_items(&s)
                .iter()
                .filter(|it| it.row == list_row(1))
                .all(|it| it.ink == Ink::Selected)
        );
    }

    /// Darkening unselected text must retain even the quieter parts of a hovered/focused
    /// control, rather than changing placeholders, hovered tabs or chosen MIDI row actions.
    #[test]
    fn cursor_selected_items_keep_their_previous_brightness() {
        for id in [FieldId::Search, FieldId::SaveName, FieldId::SaveTags] {
            let mut s = DrawerScene {
                editing: Some(Edit::Save),
                ..scene()
            };
            let placeholder = |s: &DrawerScene| {
                let mut items = Vec::new();
                field_items(&mut items, s, id, "EMPTY");
                items[0].lv
            };
            assert_eq!(placeholder(&s), 0.4);
            s.focus = Some(id);
            assert_eq!(placeholder(&s), 0.6);
            s.focus = None;
            s.hover = Some(DrawerTarget::Field(id));
            assert_eq!(placeholder(&s), 0.6);
            if id != FieldId::Search {
                let label = if id == FieldId::SaveName {
                    "NAME"
                } else {
                    "TAGS"
                };
                let level = |s: &DrawerScene| {
                    status_items(s)
                        .iter()
                        .find(|it| it.text == label)
                        .unwrap()
                        .lv
                };
                assert_eq!(level(&s), 0.7);
                s.hover = None;
                s.focus = Some(id);
                assert_eq!(level(&s), 0.7);
                s.focus = None;
                assert_eq!(level(&s), 0.475);
            }
        }
        let mut s = scene();
        s.hover = Some(DrawerTarget::Tab(Tab::Favourites));
        let tab = |s: &DrawerScene| {
            list_items(s)
                .iter()
                .find(|it| it.text == "FAVORITES")
                .unwrap()
                .lv
        };
        assert_eq!(tab(&s), 0.85);
        s.hover = None;
        assert_eq!(tab(&s), 0.4);
        s.midi = Some(MidiList {
            chosen: 0,
            rows: (0..3)
                .map(|i| MidiRow {
                    name: format!("CONTROL {i}"),
                    assignment: String::new(),
                    waiting: false,
                })
                .collect(),
            ..MidiList::default()
        });
        for hover in [
            DrawerTarget::MidiRow(1),
            DrawerTarget::MidiAction(1, MidiAction::Learn),
        ] {
            s.hover = Some(hover);
            let items = list_items(&s);
            for i in [0, 1] {
                // (The chosen row's mark aside.)
                let row: Vec<_> = items
                    .iter()
                    .filter(|it| it.row == list_row(i) && it.text != ">")
                    .collect();
                assert_eq!(row[0].lv, 1.0);
                assert_eq!(row[1].text, "-");
                assert_eq!(row[1].lv, 0.7);
                let action_level = if hover == DrawerTarget::MidiAction(i, MidiAction::Learn) {
                    1.0
                } else {
                    0.85
                };
                assert_eq!(row[2].lv, action_level);
            }
            let inactive: Vec<_> = items.iter().filter(|it| it.row == list_row(2)).collect();
            assert_eq!(inactive[0].lv, 0.65);
            assert_eq!(inactive[1].lv, 0.475);
        }
    }

    /// Moving/clearing selection and moving hover independently clears the old gold and
    /// both markers, matching a fresh frame each time. An uninvolved ordinary row stays
    /// identical, so changing selection cannot recolour the rest of the list.
    #[test]
    fn moving_selection_and_hover_redraws_without_stale_gold() {
        for scale in [860.0 / W, 1035.0 / W, 1.0] {
            let mut s = scene();
            s.hover = Some(DrawerTarget::Row(3));
            let mut states = vec![s.clone()];
            s.rows[1].current = false;
            s.rows[2].current = true;
            s.current.as_mut().unwrap().name = "Preset 2".into();
            states.push(s.clone());
            s.hover = Some(DrawerTarget::Row(4));
            states.push(s.clone());
            s.rows[2].current = false;
            s.current = None;
            states.push(s.clone());
            s.hover = None;
            states.push(s.clone());
            s.rows[1].current = true;
            s.current = scene().current;
            s.hover = Some(DrawerTarget::Star(1));
            states.push(s);

            let (_, oy) = LIST.origin();
            let top = ((oy + (list_row(7) as f64 - 1.5) * LIST.pitch) * scale).round();
            let bottom = ((oy + (list_row(7) as f64 + 9.5) * LIST.pitch) * scale).round();
            let ordinary_rect = IntRect::from_xywh(
                (LIST.x * scale).round() as i32,
                top as i32,
                (LIST.w * scale).round() as u32,
                (bottom - top) as u32,
            )
            .unwrap();
            let mut parts = DrawerRenderer::new(scale);
            let mut ordinary_before = None;
            for state in states {
                parts.render(&state);
                let mut whole = DrawerRenderer::new(scale);
                whole.render(&state);
                assert!(
                    parts.frame().data() == whole.frame().data(),
                    "selection/hover left stale pixels at scale {scale}"
                );
                let ordinary = parts.frame().clone_rect(ordinary_rect).unwrap();
                match &ordinary_before {
                    Some(before) => assert_eq!(ordinary.data(), before),
                    None => ordinary_before = Some(ordinary.data().to_vec()),
                }
            }
        }
    }

    /// Drawn line by line as by the whole: a frame drawn again where a pointer moved from row
    /// to row is the same as one drawn whole for where it is now, and it was drawn again only
    /// about the rows it left and found.
    #[test]
    fn drawn_by_lines_as_drawn_whole() {
        let mut s = scene();
        s.hover = Some(DrawerTarget::Row(3));
        let mut d = DrawerRenderer::new(0.4);
        d.render(&s);
        let before = d.frame().clone();
        s.hover = Some(DrawerTarget::Row(4));
        assert!(d.render(&s));
        let mut whole = DrawerRenderer::new(0.4);
        whole.render(&s);
        let w = d.frame().width() as usize;
        let differ: Vec<(usize, usize)> = (0..d.frame().data().len() / 4)
            .filter(|i| {
                d.frame().data()[i * 4..i * 4 + 4] != whole.frame().data()[i * 4..i * 4 + 4]
            })
            .map(|i| (i % w, i / w))
            .collect();
        assert!(
            differ.is_empty(),
            "{} pixels differ, the first {:?}",
            differ.len(),
            &differ[..differ.len().min(12)]
        );
        // Only about rows 3 and 4 (and the dot's reach round them).
        let rows_changed: Vec<usize> = (0..d.frame().height() as usize)
            .filter(|y| {
                let r = y * w * 4..(y + 1) * w * 4;
                d.frame().data()[r.clone()] != before.data()[r]
            })
            .collect();
        let (_, oy) = LIST.origin();
        let y_of = |row: f64| (oy + row * LIST.pitch) * 0.4;
        let (lo, hi) = (
            y_of(list_row(3) as f64 - 1.5),
            y_of(list_row(4) as f64 + 9.5),
        );
        assert!(!rows_changed.is_empty());
        assert!(
            rows_changed
                .iter()
                .all(|&y| (lo.floor() as usize..=hi.ceil() as usize).contains(&y)),
            "{rows_changed:?} not within {lo}..{hi}"
        );
    }

    /// The smoothed glass has no row much brighter or darker than those beside it (the
    /// picture's lines were hard bands across the list).
    #[test]
    fn the_glass_has_no_lines() {
        let g = smooth_glass(
            &Pixmap::decode_png(crate::skin::GLASS).expect("a built-in picture decodes"),
        );
        let w = g.width() as usize;
        let rows: Vec<f64> = g
            .pixels()
            .chunks_exact(w)
            .map(|r| {
                r.iter()
                    .map(|p| f64::from(p.red()) + f64::from(p.green()) + f64::from(p.blue()))
                    .sum::<f64>()
                    / w as f64
            })
            .collect();
        for k in 1..rows.len() - 1 {
            let near = (rows[k - 1] + rows[k + 1]) / 2.0;
            assert!(
                (rows[k] - near).abs() <= 0.04 * near.max(1.0),
                "row {k}: {} against {near}",
                rows[k]
            );
        }
    }
}
