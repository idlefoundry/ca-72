//! The drawer as the mock-up has it (A6: decisions.md R-LOOK; the owner, 2026-10-10: "we didn't
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
use crate::fonts::{FAMILY, Fonts};
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
const LABEL: f64 = 18.0;
/// The keys: their size, the middles of their columns and rows, their legends' size and
/// height above their middles.
const KEY: (f64, f64) = (136.0, 51.0);
const KEY_X: [f64; 4] = [2422.0, 2603.0, 2784.0, 2965.0];
const KEY_Y: [f64; 2] = [99.0, 197.0];
const KEY_LEGEND: f64 = 15.5;
const LEGEND_RISE: f64 = 36.0;

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
const POINTED: f32 = 0.82;
const NORMAL: f32 = 0.62;
const DIM: f32 = 0.42;
const FAINT: f32 = 0.3;
/// The dots' colours (the rail's name's: the mock-up's orange, their glow a deeper red), and
/// how faintly every dot shows unlit.
const DOT_ON: (u8, u8, u8) = (255, 112, 40);
const DOT_GLOW: (u8, u8, u8) = (255, 72, 10);
const UNLIT: f32 = 0.055;

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
}

impl PresetKey {
    pub const ALL: [PresetKey; 8] = [
        PresetKey::Rename,
        PresetKey::Tags,
        PresetKey::Delete,
        PresetKey::Revert,
        PresetKey::SaveAs,
        PresetKey::Restore,
        PresetKey::MidiLearn,
        PresetKey::Close,
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
        }
    }

    fn centre(self) -> (f64, f64) {
        let i = PresetKey::ALL.iter().position(|k| *k == self).unwrap_or(0);
        (KEY_X[i % 4], KEY_Y[i / 4])
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

/// Characters on a display: from dot `col`, its top at dot `row`, `lv` bright.
#[derive(Clone, Debug, PartialEq)]
struct Item {
    text: String,
    lv: f32,
    col: usize,
    row: usize,
}

/// (A control character, from a file written elsewhere, is shown as if it were not there.)
fn item(text: &str, lv: f32, col: usize, row: usize) -> Item {
    Item {
        text: text.chars().filter(|c| !c.is_control()).collect(),
        lv,
        col,
        row,
    }
}

/// The caret: a line of dots under the character it is before, as a character display's cursor
/// (steady: it does not blink).
const CARET: char = '\u{2581}';

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

/// A field's line: its text (else, empty, what it is for, faintly), and its caret when it has
/// the keyboard.
fn field_items(out: &mut Vec<Item>, s: &DrawerScene, id: FieldId, placeholder: &str) {
    let Some((_, col, row, chars)) = field_place(s, id) else {
        return;
    };
    let f = field_of(s, id);
    let from = shown_from(f, chars);
    if f.text.is_empty() {
        out.push(item(placeholder, FAINT, col, row));
    } else {
        let shown: String = f.text.chars().skip(from).take(chars).collect();
        out.push(item(&shown, BRIGHT, col, row));
    }
    if s.focus == Some(id) && s.midi.is_none() {
        let at = f.caret.saturating_sub(from);
        out.push(Item {
            text: CARET.to_string(),
            lv: BRIGHT,
            col: col + 6 * at,
            row,
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
            NORMAL
        } else {
            FAINT
        };
        out.push(item(w, lv, COL + 6 * tab_span(t).0, TAB_ROW));
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
        let text = format!(
            "{} {star} {:<NAME_CHARS$} {:<6} {}",
            if r.current { ">" } else { " " },
            cut(&r.name, NAME_CHARS),
            r.origin.unwrap_or(""),
            r.tags
        );
        out.push(item(&cut(&text, n - 2), lv, COL, row));
        if s.hover == Some(DrawerTarget::Star(i)) {
            out.push(item(&star.to_string(), BRIGHT, COL + 6 * STAR_AT, row));
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
        let mark = if chosen { ">" } else { " " };
        out.push(item(
            &format!("{mark} {}", cut(&r.name, MIDI_NAME_CHARS)),
            lv,
            COL,
            row,
        ));
        let acts = if chosen || pointed || r.waiting {
            midi_spans(r)
        } else {
            Vec::new()
        };
        let end = acts.last().map_or(n - 2, |a| a.2 - 2);
        let (said, said_lv) = if r.waiting {
            ("MOVE A CONTROLLER NOW", BRIGHT)
        } else if r.assignment.is_empty() {
            ("-", DIM)
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
            line(&mut out, 2, "NAME", DIM);
            line(&mut out, 3, "TAGS", DIM);
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
        if (x - kx).abs() <= KEY.0 / 2.0 + 4.0 && (y - ky).abs() <= KEY.1 / 2.0 + 4.0 {
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

/// What the thread begun as the editor opens hands back: the pictures, the scale it made the
/// still at, the still.
type Warmed = (Arc<Pictures>, f64, Arc<Pixmap>);

/// The pictures a drawer is made of, decoded: the face's, the displays' glass (smoothed), the
/// keys' cap.
struct Pictures {
    face: Pixmap,
    glass: Pixmap,
    button: Pixmap,
}

impl Pictures {
    fn load() -> Pictures {
        let png = |b: &[u8]| Pixmap::decode_png(b).expect("a built-in picture decodes");
        Pictures {
            face: crate::skin::face_picture(),
            glass: smooth_glass(&png(crate::skin::GLASS)),
            button: png(crate::skin::BUTTON),
        }
    }
}

/// What does not change at `scale`: another drawer's open at it, else made (and kept for
/// others while one has it: each instance's editor has its own drawer).
fn shared_still(scale: f64, pictures: &Pictures) -> Arc<Pixmap> {
    static SHARED: Mutex<Vec<(u64, Weak<Pixmap>)>> = Mutex::new(Vec::new());
    let key = scale.to_bits();
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
    let still = Arc::new(make_still(scale, pictures));
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
        soft_shadow(
            &mut s,
            (x - KEY.0 / 2.0 + 3.0, y - KEY.1 / 2.0 + 5.0, KEY.0, KEY.1),
            9.0,
            3.0,
            0.6,
            (
                x - KEY.0 / 2.0 + 8.0,
                y - KEY.1 / 2.0 + 8.0,
                KEY.0 - 16.0,
                KEY.1 - 16.0,
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
fn smooth_glass(p: &Pixmap) -> Pixmap {
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
                    ((f32::from(u) + (f32::from(v) - f32::from(u)) * t) * 256.0) as u32
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

/// The print: the sections' titles over their displays, the keys' legends over them.
fn print_svg() -> String {
    let mut s = Svg::default();
    let text = |s: &mut Svg, x: f64, y: f64, t: &str, size: f64, anchor: &str| {
        put!(
            s,
            "<text x='{}' y='{}' font-family='{FAMILY}' font-size='{}' font-weight='700' text-anchor='{anchor}' dominant-baseline='central' fill='{}'>{t}</text>",
            N(x),
            N(y),
            N(size),
            colour::LEGEND
        );
    };
    for (d, t) in [(SEARCH, "FIND"), (LIST, "PRESETS"), (STATUS, "PRESET")] {
        text(&mut s, d.x, LABEL_Y, t, LABEL, "start");
    }
    for k in PresetKey::ALL {
        let (x, y) = k.centre();
        text(&mut s, x, y - LEGEND_RISE, k.legend(), KEY_LEGEND, "middle");
    }
    s.0
}

impl DrawerRenderer {
    /// At `scale`: its pictures decoded and what does not change at it made on a thread of
    /// their own, begun now (as the editor opens, so the drawer's first opening waits for
    /// neither: a tenth of a second at a Retina screen's scale).
    pub fn new(scale: f64) -> Self {
        let warming = std::thread::Builder::new()
            .name("ca72-drawer".into())
            .spawn(move || {
                let pictures = Pictures::load();
                let still = shared_still(scale, &pictures);
                (Arc::new(pictures), scale, still)
            })
            .ok();
        DrawerRenderer {
            scale,
            options: usvg::Options::default(),
            warming,
            pictures: None,
            still: None,
            frame: Pixmap::new(1, 1).expect("a pixel"),
            shown: None,
            drawn: Default::default(),
            keys: None,
        }
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
            let lit = lit_dots(d, &items);
            for band in bands {
                redraw(&mut self.frame, &still, self.scale, d, &lit, band);
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
                (KEY_X[0] - w, KEY_Y[0] - h, KEY_X[3] + w, KEY_Y[1] + h),
            );
            let mut o = Svg::default();
            for k in &keys.held {
                let (x, y) = k.centre();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#000' fill-opacity='0.34'/><rect x='{}' y='{}' width='{}' height='5' rx='2.5' fill='#000' fill-opacity='0.3'/>",
                    N(x - KEY.0 / 2.0 + 2.0),
                    N(y - KEY.1 / 2.0 + 2.0),
                    N(KEY.0 - 4.0),
                    N(KEY.1 - 4.0),
                    N(x - KEY.0 / 2.0 + 6.0),
                    N(y - KEY.1 / 2.0 + 3.0),
                    N(KEY.0 - 12.0)
                );
            }
            if let Some(k) = keys.pointed {
                let (x, y) = k.centre();
                put!(
                    o,
                    "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='#fff' fill-opacity='0.07'/>",
                    N(x - KEY.0 / 2.0 + 2.0),
                    N(y - KEY.1 / 2.0 + 2.0),
                    N(KEY.0 - 4.0),
                    N(KEY.1 - 4.0)
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
        if let Some(Ok((pictures, scale, still))) = self.warming.take().map(JoinHandle::join) {
            self.pictures = Some(pictures);
            if scale == self.scale {
                return still;
            }
        }
        let pictures = self
            .pictures
            .get_or_insert_with(|| Arc::new(Pictures::load()));
        shared_still(self.scale, pictures)
    }
}

/// What never changes at `scale`: the face, as the strip's under it (the picture 640 units
/// across, mirrored tile to tile, placed as the strip's), its trim and the rail's shadow; the
/// print, worn into it; the displays' bezels and glass, every dot faintly there; the keys' caps.
fn make_still(scale: f64, pictures: &Pictures) -> Pixmap {
    let s = scale;
    let fonts = Fonts::new();
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
    // The print, in the band across the top it is in.
    let band = ((PRINT_H * s).ceil() as u32).min(h);
    if let (Some(mut print), Some(under)) = (
        Pixmap::new(w, band),
        IntRect::from_xywh(0, 0, w, band).and_then(|r| still.clone_rect(r)),
    ) {
        svg_over(&options, &mut print, s, &print_svg());
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
        grid(&mut still, s, &d, UNLIT, DOT_ON);
    }
    let cap = widened(&pictures.button, KEY.0 / KEY.1);
    for k in PresetKey::ALL {
        sprite(&mut still, &cap, s, k.centre(), KEY);
    }
    still
}

/// The print's band across the drawer's top: the titles and the keys' legends are in it.
const PRINT_H: f64 = 190.0;

/// Each dot of a display lit by `items`, as bright as the brightest over it (none: 0).
fn lit_dots(d: &Display, items: &[Item]) -> Vec<f32> {
    let (cols, rows) = (d.cols(), d.rows());
    let mut lit = vec![0.0f32; cols * rows];
    for it in items {
        for (n, ch) in it.text.chars().enumerate() {
            let g = glyph(ch);
            for (j, bits) in g.iter().enumerate() {
                for b in 0..5 {
                    let (c, row) = (it.col + 6 * n + b, it.row + j);
                    if bits >> (4 - b) & 1 == 1 && c < cols && row < rows {
                        let v = &mut lit[row * cols + c];
                        *v = v.max(it.lv);
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
    lit: &[f32],
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
    let r = (d.pitch * 0.36 * scale) as f32;
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
    // By brightness (to a twentieth): the glows first, then the dots over them.
    let mut by: BTreeMap<u8, Vec<Disc>> = BTreeMap::new();
    for j in first..last {
        for c in 0..cols {
            let v = lit[j * cols + c];
            if v > 0.0 {
                let x = ((ox + c as f64 * d.pitch) * scale) as f32 - x0 as f32;
                let y = ((oy + j as f64 * d.pitch) * scale) as f32 - top as f32;
                by.entry((v * 20.0).round() as u8)
                    .or_default()
                    .push((x, y, r));
            }
        }
    }
    for (&q, ds) in &by {
        let glows: Vec<Disc> = ds.iter().map(|&(x, y, r)| (x, y, r * 2.1)).collect();
        discs(
            &mut band,
            &mut cover,
            &glows,
            DOT_GLOW,
            0.16 * f32::from(q) / 20.0,
        );
    }
    for (&q, ds) in &by {
        discs(&mut band, &mut cover, ds, DOT_ON, f32::from(q) / 20.0);
    }
    for y in y0..y1 {
        let (i, b) = (
            y as usize * row + x0 as usize * 4,
            (y - top) as usize * bw * 4,
        );
        frame.data_mut()[i..i + bw * 4].copy_from_slice(&band.data()[b..b + bw * 4]);
    }
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
        let row = |k: usize| {
            items
                .iter()
                .find(|it| it.row == list_row(k) && it.col == COL)
                .map(|it| it.text.clone())
                .expect("a row")
        };
        let chars =
            |t: &str, from: usize, n: usize| t.chars().skip(from).take(n).collect::<String>();
        let current = row(1);
        assert_eq!(chars(&current, 0, 3), "> \u{2606}");
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
