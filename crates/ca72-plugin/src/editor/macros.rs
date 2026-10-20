//! The macros in the editor (decisions.md R-MACROS): the rail's MACRO key flips the lower
//! page's left area between the wheels and six macros (remembered on this computer); each macro
//! a knob (the host's MACRO n), its name typed on its dot screen, its jack an assignment dragged
//! from as an LFO's OUT is. Everything of modulation (the cord, the rings, the pop-up, the menus'
//! heads, Alt-drag) is `lfo_page`'s, over sources: a macro is `Source::Macro(n)`, with its colour
//! (the computer's, right-clicking its name) and its name for a label.
//!
//! While the macros show, the wheels and their switches are neither drawn nor found by the
//! pointer, and go on working (MIDI, the host's automation).

use std::time::Instant;

use ca72_panel::controls::MACRO_FIRST;
use ca72_panel::modulation as mods;
use ca72_panel::strip::macros::{self as macro_ui, Cell, MacroScene, MacroTarget};
use ca72_panel::{CONTROLS, Target};
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};
use nih_plug::prelude::Param;

use super::{DOUBLE_CLICK, Editing, MENU_TEXT};
use crate::learning::{ModItem, Row};
use crate::lfo::{self, MACROS, MacroColor, Source};
use crate::settings::Change;

/// The editor's macros' state: a name being typed (the macro's, what is typed so far), the
/// part under the pointer (the knobs apart: `Editing::hover`), the last press on a name (for a
/// double click).
#[derive(Debug, Default)]
pub(super) struct MacroView {
    pub(super) typing: Option<(usize, String)>,
    pub(super) hover: Option<MacroTarget>,
    pub(super) last_name_press: Option<(Instant, usize)>,
}

/// The macro control `i` is, if it is one (0 to 5).
pub(super) fn macro_of_control(i: usize) -> Option<usize> {
    (MACRO_FIRST..MACRO_FIRST + MACROS)
        .contains(&i)
        .then(|| i - MACRO_FIRST)
}

impl Editing {
    /// Whether the left area shows the macros (with the window's lower part).
    pub(super) fn macros_show(&self) -> bool {
        self.settings.macros_shown && self.expanded()
    }

    /// The rail's MACRO key: the left area's macros or wheels, remembered on this computer; the
    /// lower part shown first if it is not (the macros then).
    pub(super) fn toggle_macros(&mut self) {
        let show = if self.expanded() {
            !self.settings.macros_shown
        } else {
            self.toggle_panel();
            if !self.expanded() {
                return;
            }
            true
        };
        self.end_rename(true);
        self.keep_settings(Change::Macros(show));
        self.hover = None;
        self.macro_view.hover = None;
        self.modulating.popup = None;
        self.composed = None;
    }

    /// Macro `n`'s name as shown: its typed one, else MACRO n.
    pub(super) fn macro_name(&self, n: usize) -> String {
        let name = self.params.lfo_patch.name(n);
        if name.is_empty() {
            format!("MACRO {}", n + 1)
        } else {
            name
        }
    }

    /// Macro `n`'s knob's value (0..1).
    pub(super) fn macro_value(&self, n: usize) -> f64 {
        f64::from(self.params.macro_param(n).unmodulated_normalized_value())
    }

    /// Whether macro `n` modulates anything.
    fn macro_used(&self, n: usize) -> bool {
        self.params
            .lfo_patch
            .assignments(Source::Macro(n))
            .iter()
            .flatten()
            .any(|a| a.depth != 0.0)
    }

    /// The MACRO key's faint glow, the wheels showing: an assigned macro above 0 (the rail's
    /// lights' rule, R-LFO-PAGE).
    pub(super) fn macro_glow(&self) -> bool {
        (0..MACROS).any(|n| self.macro_used(n) && self.macro_value(n) > 0.0)
    }

    /// The area's scene: each macro's colour, name, knob (a drag's where the drag has it), and
    /// whether it modulates anything; a name being typed.
    pub(super) fn macro_scene_now(&self) -> MacroScene {
        MacroScene {
            cells: std::array::from_fn(|n| Cell {
                colour: self.settings.macro_colors[n].rgb(),
                name: self.macro_name(n),
                value: self.scene.values[MACRO_FIRST + n],
                dim: !self.macro_used(n),
                typing: self
                    .macro_view
                    .typing
                    .as_ref()
                    .filter(|(k, _)| *k == n)
                    .map(|(_, t)| t.clone()),
            }),
        }
    }

    /// The pointer over the area: its knobs as the panel's, else its names and jacks.
    pub(super) fn macro_hover(&mut self, (x, y): (f64, f64)) {
        self.hover = ca72_panel::interact::hit_macro(x, y);
        self.macro_view.hover = if self.hover.is_none() {
            macro_ui::hit(x, y)
        } else {
            None
        };
        self.strip_scene.hover = None;
        self.browser.bar.hover = None;
        self.modulating.popup = None;
    }

    /// A press on the area (it showing, the pointer over it): a knob as the panel's (its macro
    /// the last touched, for Alt-drag), a jack an assignment dragged, a name double-clicked
    /// typed. Whether it was the area's.
    pub(super) fn macro_press(&mut self, (x, y): (f64, f64), modifiers: Modifiers) -> bool {
        if let Some(t @ Target::Control(i)) = ca72_panel::interact::hit_macro(x, y) {
            self.end_rename(true);
            if let Some(n) = macro_of_control(i) {
                self.modulating.last = Some(Source::Macro(n));
            }
            self.press_target(t, modifiers);
            return true;
        }
        match macro_ui::hit(x, y) {
            Some(MacroTarget::Jack(n)) => {
                self.end_rename(true);
                self.modulating.last = Some(Source::Macro(n));
                self.modulating.patching = Some(super::lfo_page::Patching {
                    source: Source::Macro(n),
                    at: (x, y),
                });
                self.modulating.popup = None;
                true
            }
            Some(MacroTarget::Name(n)) => {
                let now = Instant::now();
                if self
                    .macro_view
                    .typing
                    .as_ref()
                    .is_some_and(|(k, _)| *k == n)
                {
                    return true;
                }
                self.end_rename(true);
                if self
                    .macro_view
                    .last_name_press
                    .is_some_and(|(at, k)| k == n && now - at < DOUBLE_CLICK)
                {
                    self.macro_view.last_name_press = None;
                    self.start_rename(n);
                } else {
                    self.macro_view.last_name_press = Some((now, n));
                }
                true
            }
            None => false,
        }
    }

    /// Macro `n`'s name typed from its screen: what it has, all of it to be typed over.
    pub(super) fn start_rename(&mut self, n: usize) {
        self.playing.release();
        self.macro_view.typing = Some((n, self.params.lfo_patch.name(n)));
        self.typing_keys = true;
    }

    /// The name being typed kept (`keep`) or left as it was; the typing ended.
    pub(super) fn end_rename(&mut self, keep: bool) {
        if let Some((n, text)) = self.macro_view.typing.take()
            && keep
        {
            self.params.lfo_patch.set_name(n, &text);
        }
    }

    /// A key while a name is typed: a printable character added (capitals, at most seven), the
    /// last taken back, Enter keeping it, Escape leaving it as it was; every key is the editor's
    /// meanwhile.
    pub(super) fn rename_key(&mut self, k: &KeyboardEvent) -> bool {
        if k.state != KeyState::Down {
            return true;
        }
        let Some((_, text)) = &mut self.macro_view.typing else {
            return false;
        };
        match &k.key {
            Key::Character(c) => {
                // (Capitals, printable, seven at most; no leading space.)
                for ch in c.chars().map(|ch| ch.to_ascii_uppercase()) {
                    if (' '..='~').contains(&ch)
                        && text.chars().count() < lfo::NAME_MOST
                        && !(text.is_empty() && ch == ' ')
                    {
                        text.push(ch);
                    }
                }
            }
            Key::Backspace => {
                text.pop();
            }
            Key::Enter => self.end_rename(true),
            Key::Escape => self.end_rename(false),
            _ => {}
        }
        true
    }

    /// The rows a macro knob's right-click menu gains (after MIDI Learn's): RENAME…, REMOVE ALL
    /// ASSIGNMENTS, then a head for each knob it modulates with DEPTH… and REMOVE.
    pub(super) fn macro_rows(&self, n: usize) -> Vec<Row> {
        let s = Source::Macro(n);
        let mut rows = vec![
            Row::Item(ModItem::Rename(n), "RENAME\u{2026}".into(), true, false),
            Row::Item(
                ModItem::RemoveAll(s),
                "REMOVE ALL ASSIGNMENTS".into(),
                self.params.lfo_patch.count(s) > 0,
                false,
            ),
        ];
        for a in self.params.lfo_patch.assignments(s).into_iter().flatten() {
            let Some(i) = super::lfo_page::control_of(a.target) else {
                continue;
            };
            rows.push(Row::Head(format!(
                "{} \u{b7} {}",
                CONTROLS[i].label,
                mods::percent(f64::from(a.depth))
            )));
            rows.push(Row::Item(
                ModItem::Depth(s, a.target, i),
                "DEPTH\u{2026}".into(),
                true,
                false,
            ));
            rows.push(Row::Item(
                ModItem::Remove(s, a.target),
                "REMOVE".into(),
                true,
                false,
            ));
        }
        rows
    }

    /// A right click on the area: a knob's menu (MIDI Learn, RENAME…, what it modulates), else
    /// the macro's own (a name's or a jack's): its colour (the computer's), RENAME….
    pub(super) fn macro_menu(&mut self, (x, y): (f64, f64)) -> bool {
        let size = self.renderer.text_size() * MENU_TEXT;
        if let Some(target @ Target::Control(i)) = ca72_panel::interact::hit_macro(x, y) {
            let rows = self.knob_rows(i);
            self.learning.open_menu_with(
                self.renderer.fonts(),
                &self.params.midi_map,
                Some(crate::learning::Place::Panel(i)),
                Some(target),
                (x, y),
                size,
                rows,
            );
            return true;
        }
        let Some(t) = macro_ui::hit(x, y) else {
            return false;
        };
        let n = t.macro_n();
        let shown = self.settings.macro_colors[n];
        let mut rows = vec![Row::Item(
            ModItem::Rename(n),
            "RENAME\u{2026}".into(),
            true,
            false,
        )];
        rows.push(Row::Head("COLOR".into()));
        for c in MacroColor::ALL {
            rows.push(Row::Item(
                ModItem::MacroColor(n, c),
                c.name().into(),
                c != shown,
                c == shown,
            ));
        }
        self.learning.open_rows_menu(
            self.renderer.fonts(),
            (x, y),
            size,
            self.macro_title(n),
            rows,
        );
        true
    }

    /// A macro's title: MACRO n, its name after it where it has one.
    pub(super) fn macro_title(&self, n: usize) -> String {
        let name = self.params.lfo_patch.name(n);
        if name.is_empty() {
            format!("MACRO {}", n + 1)
        } else {
            format!("MACRO {} \u{b7} {name}", n + 1)
        }
    }

    /// The tip of the area's part under the pointer (its knobs' are the panel's): what a name
    /// does and what its macro modulates, what a jack does.
    pub(super) fn macro_tip(&self) -> Option<(String, f64, f64)> {
        let t = self
            .macro_view
            .hover
            .filter(|_| self.macros_show() && self.modulating.patching.is_none())?;
        let n = t.macro_n();
        let a: Vec<String> = self
            .params
            .lfo_patch
            .assignments(Source::Macro(n))
            .into_iter()
            .flatten()
            .map(|a| {
                let name = super::lfo_page::control_of(a.target)
                    .map_or(a.target.short(), |i| CONTROLS[i].label);
                format!("{name} {}", mods::percent(f64::from(a.depth)))
            })
            .collect();
        let title = self.macro_title(n);
        let (text, (x, y)) = match t {
            MacroTarget::Name(_) => (
                if a.is_empty() {
                    format!("{title}: DOUBLE-CLICK TO RENAME")
                } else {
                    format!("{title} MODULATES {}", a.join(" \u{b7} "))
                },
                {
                    let (x, y, w, _) = macro_ui::name_at(n);
                    (x + w / 2.0, y - 6.0)
                },
            ),
            MacroTarget::Jack(_) => (format!("{title}: DRAG TO A KNOB TO MODULATE IT"), {
                let (x, y) = macro_ui::jack_at(n);
                (x, y - macro_ui::COLLAR - 6.0)
            }),
        };
        // (Kept inside the window: a tip is centred over where it points.)
        let size = self.renderer.text_size();
        let w = self
            .renderer
            .fonts()
            .advance(&text, size, ca72_panel::fonts::Weight::Regular, 0.0)
            + 0.8 * size;
        Some((text, x.max(w / 2.0 + 10.0), y))
    }
}
