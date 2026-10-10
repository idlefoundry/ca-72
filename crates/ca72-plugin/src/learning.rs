//! MIDI Learn in the editor (decisions.md R34): a control's menu (a right click: MIDI LEARN,
//! REMOVE MIDI ASSIGNMENT, MIDI ASSIGNMENTS…), the ring and the note over the control being
//! learned, what was done said over it for a while, and the drawer's MIDI list, which the keyboard
//! operates. The assignments themselves are [`crate::learn::MidiMap`]'s; this is what the editor
//! shows of them and asks of it. Nothing here runs on the audio thread.

use std::time::{Duration, Instant};

use ca72_panel::Target;
use ca72_panel::controls::{CONTROLS, index as control_index};
use ca72_panel::fonts::Fonts;
use ca72_panel::learn::{Menu, Note, PANEL, Tone, above};
use ca72_panel::presets::{MidiAction, MidiList, MidiRow, ROWS_SHOWN, Tone as DrawerTone};
use ca72_panel::strip::{StripControl, span};
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};

use crate::learn::{Assigned, Cc, LEARNABLE, MidiMap, RESERVED_TEXT, index, reserved};

/// How long what was done stays said over its control.
const NOTICE: Duration = Duration::from_secs(4);

/// The keys the drawer's MIDI list takes, as it says.
pub const KEYS_TEXT: &str = "UP, DOWN: CHOOSE · ENTER: LEARN · DELETE: REMOVE · ESC: CANCEL";

/// Where a learnable parameter's control is: the panel's (its index in [`CONTROLS`]: the
/// strip's knobs and the left hand's are the panel's controls too), the strip's tabs, or none
/// (LOCK: the drawer's list alone learns it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Panel(usize),
    Strip(StripControl),
}

/// Where learnable parameter `i`'s control is.
pub fn place(i: usize) -> Option<Place> {
    let id = LEARNABLE.get(i)?.id;
    Some(match id {
        "poly" => Place::Strip(StripControl::Poly),
        "unison" => Place::Strip(StripControl::Unison),
        "placement" => Place::Strip(StripControl::Placement),
        "auto_gain" => Place::Strip(StripControl::AutoGain),
        _ => Place::Panel(control_index(id)?),
    })
}

/// The learnable parameter a place's control operates.
pub fn learnable_at(p: Place) -> Option<usize> {
    match p {
        Place::Panel(c) => index(CONTROLS.get(c)?.param),
        Place::Strip(s) => index(match s {
            StripControl::Poly => "poly",
            StripControl::Unison => "unison",
            StripControl::Placement => "placement",
            StripControl::AutoGain => "auto_gain",
        }),
    }
}

/// Where a note about a place's control goes: centred above it.
fn note_point(p: Place) -> (f64, f64) {
    match p {
        Place::Panel(c) => above(&CONTROLS[c]),
        Place::Strip(s) => {
            let (x0, y0, x1, _) = span(s);
            ((x0 + x1) / 2.0, y0)
        }
    }
}

/// What a menu's items do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Learn,
    Cancel,
    Remove,
    List,
    /// Says why the control cannot be learned; does nothing.
    Not,
}

/// A menu open: for which learnable parameter (none: a control that cannot be learned), what
/// its items do, and how it is drawn.
#[derive(Clone, Debug, PartialEq)]
struct Open {
    learnable: Option<usize>,
    items: Vec<Item>,
    menu: Menu,
}

/// What a press on the panel's menu did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    /// Nothing open: the press is the panel's.
    Nothing,
    /// Closed (pressed outside it, or on its title), or an item done: the press is the menu's.
    Done,
    /// MIDI ASSIGNMENTS…: the drawer's MIDI list wanted, this parameter chosen.
    List(Option<usize>),
}

/// What was last done, said over a control for a while: its lines, the learnable parameter
/// whose control it is, and when.
type Notice = (Vec<(String, Tone)>, usize, Instant);

/// The editor's MIDI Learn.
#[derive(Debug, Default)]
pub struct Learning {
    open: Option<Open>,
    /// What was last done: its lines, the parameter whose control they are said over, and when.
    notice: Option<Notice>,
    /// The drawer shows the MIDI list (not the presets), its chosen row and first shown, and
    /// what was last done (its status line).
    pub list: bool,
    chosen: usize,
    first: usize,
    status: (String, DrawerTone),
    /// The window should take the keyboard (a menu opened, or learning begun: Escape is ours),
    /// asked once.
    pub take_keys: bool,
}

/// A parameter's name.
fn label(i: usize) -> &'static str {
    LEARNABLE.get(i).map_or("", |l| l.label)
}

/// What a control that cannot be learned says in its menu.
fn not_learnable(t: Target) -> Option<String> {
    Some(match t {
        Target::Control(i) | Target::Legend(i, _) => match CONTROLS.get(i)?.param {
            "pitch_wheel" => "PITCH: MIDI PITCH BEND MOVES IT".into(),
            "mod_wheel" => "MODULATION: THE MODULATION WHEEL (CC 1) MOVES IT".into(),
            "quality" => "QUALITY: SET FOR THE COMPUTER, NOT THE SOUND".into(),
            _ => return None,
        },
        Target::Power => "POWER: THE HOST'S BYPASS".into(),
        Target::Grip => return None,
    })
}

impl Learning {
    /// Whether the editor should keep the keyboard: learning, or a menu open.
    pub fn holds_keys(&self, map: &MidiMap) -> bool {
        self.open.is_some() || map.armed().is_some()
    }

    /// Whether a menu is open.
    pub fn menu_open(&self) -> bool {
        self.open.is_some()
    }

    /// The menu open, as drawn.
    pub fn menu(&self) -> Option<&Menu> {
        self.open.as_ref().map(|o| &o.menu)
    }

    /// A control's menu opened at (`x`, `y`) on the panel (drawing units) for `place`'s control,
    /// or for a control that cannot be learned (`not`: what it says); `size` the lettering's.
    pub fn open_menu(
        &mut self,
        fonts: &Fonts,
        map: &MidiMap,
        place: Option<Place>,
        not: Option<Target>,
        (x, y): (f64, f64),
        size: f64,
    ) {
        let learnable = place.and_then(learnable_at);
        let (title, items) = match learnable {
            Some(i) => {
                let cc = map
                    .assignment(i)
                    .map_or("NO MIDI CONTROLLER".to_owned(), Cc::text);
                let learn = if map.armed() == Some(i) {
                    (Item::Cancel, "CANCEL MIDI LEARN")
                } else {
                    (Item::Learn, "MIDI LEARN")
                };
                (
                    format!("{} · {cc}", label(i)),
                    vec![
                        learn,
                        (Item::Remove, "REMOVE MIDI ASSIGNMENT"),
                        (Item::List, "MIDI ASSIGNMENTS…"),
                    ],
                )
            }
            None => match not.and_then(not_learnable) {
                Some(t) => (
                    t,
                    vec![
                        (Item::Not, "NOT LEARNED BY MIDI LEARN"),
                        (Item::List, "MIDI ASSIGNMENTS…"),
                    ],
                ),
                None => {
                    self.open = None;
                    return;
                }
            },
        };
        let assigned = learnable.is_some_and(|i| map.assignment(i).is_some());
        let menu = Menu {
            x,
            y,
            size,
            title,
            items: items
                .iter()
                .map(|(it, t)| {
                    let enabled = match it {
                        Item::Remove => assigned,
                        Item::Not => false,
                        _ => true,
                    };
                    ((*t).to_owned(), enabled)
                })
                .collect(),
            hover: None,
        }
        .placed(fonts, x, y, PANEL);
        self.open = Some(Open {
            learnable,
            items: items.into_iter().map(|(it, _)| it).collect(),
            menu,
        });
        self.take_keys = true;
    }

    /// The menu closed.
    pub fn close_menu(&mut self) {
        self.open = None;
    }

    /// The pointer over the panel at (`x`, `y`): the menu's item under it lit.
    pub fn hover(&mut self, fonts: &Fonts, (x, y): (f64, f64)) {
        if let Some(o) = &mut self.open {
            o.menu.hover = o.menu.hit(fonts, x, y).flatten();
        }
    }

    /// A press at (`x`, `y`) on the panel while a menu may be open: its item done, or the menu
    /// closed by a press elsewhere (which does nothing else).
    pub fn press(&mut self, fonts: &Fonts, map: &MidiMap, (x, y): (f64, f64)) -> Pressed {
        let Some(o) = self.open.take() else {
            return Pressed::Nothing;
        };
        let Some(Some(k)) = o.menu.hit(fonts, x, y) else {
            return Pressed::Done;
        };
        if !o.menu.items.get(k).is_some_and(|(_, enabled)| *enabled) {
            // (An item that cannot be chosen leaves the menu open.)
            self.open = Some(o);
            return Pressed::Done;
        }
        match (o.items.get(k), o.learnable) {
            (Some(Item::Learn), Some(i)) => self.learn(map, i),
            (Some(Item::Cancel), _) => self.cancel(map),
            (Some(Item::Remove), Some(i)) => self.remove(map, i),
            (Some(Item::List), l) => return Pressed::List(l),
            _ => {}
        }
        Pressed::Done
    }

    /// Learnable parameter `i` learned: the next controller moved is its own (another being
    /// learned is no longer).
    pub fn learn(&mut self, map: &MidiMap, i: usize) {
        map.arm(i);
        self.notice = None;
        self.chosen = i;
        self.say(
            format!("{}: MOVE A CONTROLLER ON YOUR MIDI DEVICE.", label(i)),
            DrawerTone::News,
        );
        self.take_keys = true;
    }

    /// Learning cancelled; the assignments as they were.
    pub fn cancel(&mut self, map: &MidiMap) {
        if let Some(i) = map.armed() {
            self.say(
                format!("{}: MIDI LEARN CANCELLED.", label(i)),
                DrawerTone::Dim,
            );
        }
        map.cancel();
    }

    /// Learnable parameter `i`'s controller removed (the sound as it is).
    pub fn remove(&mut self, map: &MidiMap, i: usize) {
        if let Some(cc) = map.remove(i) {
            let line = format!("{}: {} REMOVED", label(i), cc.text());
            self.say(format!("{line}."), DrawerTone::Dim);
            self.notice = Some((vec![(line, Tone::Plain)], i, Instant::now()));
        }
    }

    fn say(&mut self, text: String, tone: DrawerTone) {
        self.status = (text, tone);
    }

    /// The editor closing: a controller the audio thread has already caught is assigned (the
    /// message that came is the assignment), then learning ends.
    pub fn close(&mut self, map: &MidiMap) {
        self.poll(map);
        map.cancel();
        self.open = None;
    }

    /// What the audio thread caught for the parameter being learned, assigned, and said.
    pub fn poll(&mut self, map: &MidiMap) -> Option<Assigned> {
        let a = map.poll()?;
        let head = format!("{}: {}", label(a.param), a.cc.text());
        let mut lines = vec![(head.clone(), Tone::Accent)];
        let mut status = head;
        if let Some(d) = a.displaced {
            let t = format!("TAKEN FROM {}, WHICH HAS NONE NOW", label(d));
            status = format!("{status}, {t}");
            lines.push((t, Tone::Plain));
        }
        if let Some(r) = a.replaced {
            let t = format!("IN PLACE OF {}", r.text());
            status = format!("{status}, {t}");
            lines.push((t, Tone::Dim));
        }
        self.say(format!("{status}."), DrawerTone::News);
        self.notice = Some((lines, a.param, Instant::now()));
        Some(a)
    }

    /// The note over the control being learned (or the one last done, for a while), and the
    /// panel's control to ring and the strip's.
    pub fn scene(
        &mut self,
        map: &MidiMap,
        size: f64,
    ) -> (
        Option<Note>,
        Option<usize>,
        Option<StripControl>,
        Option<Menu>,
    ) {
        if self
            .notice
            .as_ref()
            .is_some_and(|(_, _, t)| t.elapsed() > NOTICE)
        {
            self.notice = None;
        }
        let armed = map.armed();
        let ring = armed.and_then(place);
        let (panel, strip) = match ring {
            Some(Place::Panel(c)) => (Some(c), None),
            Some(Place::Strip(s)) => (None, Some(s)),
            None => (None, None),
        };
        let note = match armed {
            Some(i) => place(i).map(|p| {
                let third = match map.refused() {
                    Some(cc) => (
                        format!(
                            "CC {cc} IS {}: NOT LEARNED. MOVE ANOTHER CONTROLLER.",
                            reserved(cc).unwrap_or("reserved").to_uppercase()
                        ),
                        Tone::Warn,
                    ),
                    None => (RESERVED_TEXT.to_owned(), Tone::Dim),
                };
                let (x, y) = note_point(p);
                Note {
                    x,
                    y,
                    size,
                    lines: vec![
                        (format!("MIDI LEARN: {}", label(i)), Tone::Accent),
                        (
                            "MOVE A CONTROLLER ON YOUR MIDI DEVICE · ESC OR CANCEL STOPS".into(),
                            Tone::Plain,
                        ),
                        third,
                    ],
                }
            }),
            None => self.notice.as_ref().and_then(|(lines, i, _)| {
                place(*i).map(|p| {
                    let (x, y) = note_point(p);
                    Note {
                        x,
                        y,
                        size,
                        lines: lines.clone(),
                    }
                })
            }),
        };
        (
            note,
            panel,
            strip,
            self.open.as_ref().map(|o| o.menu.clone()),
        )
    }

    /// The drawer's MIDI list as it is now.
    pub fn list_scene(&mut self, map: &MidiMap) -> MidiList {
        let assigned = map.assignments();
        let armed = map.armed();
        let rows: Vec<MidiRow> = LEARNABLE
            .iter()
            .enumerate()
            .map(|(i, l)| MidiRow {
                name: l.label.to_owned(),
                assignment: assigned[i].map(Cc::text).unwrap_or_default(),
                waiting: armed == Some(i),
            })
            .collect();
        self.chosen = self.chosen.min(rows.len().saturating_sub(1));
        let refused = map.refused().map(|cc| {
            (
                format!(
                    "CC {cc} IS {}: NOT LEARNED. MOVE ANOTHER CONTROLLER.",
                    reserved(cc).unwrap_or("reserved").to_uppercase()
                ),
                DrawerTone::Trouble,
            )
        });
        let (status, tone) = refused.unwrap_or_else(|| self.status.clone());
        MidiList {
            rows,
            first: self.first,
            chosen: self.chosen,
            keys: KEYS_TEXT.into(),
            reserved: RESERVED_TEXT.into(),
            status,
            tone,
        }
    }

    /// The list shown with learnable parameter `i` chosen (in view).
    pub fn show_list(&mut self, chosen: Option<usize>) {
        self.list = true;
        if let Some(i) = chosen {
            self.choose(i);
        }
    }

    fn choose(&mut self, i: usize) {
        self.chosen = i.min(LEARNABLE.len() - 1);
        if self.chosen < self.first {
            self.first = self.chosen;
        } else if self.chosen >= self.first + ROWS_SHOWN {
            self.first = self.chosen + 1 - ROWS_SHOWN;
        }
    }

    /// The list scrolled by `rows`.
    pub fn scroll(&mut self, rows: i32) {
        let most = LEARNABLE.len().saturating_sub(ROWS_SHOWN) as i32;
        self.first = (self.first as i32 + rows).clamp(0, most) as usize;
    }

    /// A press on the list: a row chosen, or its button done.
    pub fn list_press(&mut self, map: &MidiMap, row: usize, action: Option<MidiAction>) {
        self.choose(row);
        match action {
            Some(MidiAction::Learn) => self.learn(map, row),
            Some(MidiAction::Cancel) => self.cancel(map),
            Some(MidiAction::Remove) => self.remove(map, row),
            None => {}
        }
    }

    /// A key while the drawer shows the list: whether it was the list's. `close` shuts the
    /// drawer (Escape with nothing being learned).
    pub fn list_key(&mut self, e: &KeyboardEvent, map: &MidiMap, close: &mut bool) -> bool {
        if e.state != KeyState::Down {
            return true;
        }
        // The host's shortcuts (Command, Control) stay the host's.
        if e.modifiers.intersects(Modifiers::META | Modifiers::CONTROL) {
            return false;
        }
        match &e.key {
            Key::ArrowDown => self.choose(self.chosen + 1),
            Key::ArrowUp => self.choose(self.chosen.saturating_sub(1)),
            Key::PageDown => self.choose(self.chosen + ROWS_SHOWN),
            Key::PageUp => self.choose(self.chosen.saturating_sub(ROWS_SHOWN)),
            Key::Home => self.choose(0),
            Key::End => self.choose(LEARNABLE.len() - 1),
            Key::Enter => {
                if map.armed() == Some(self.chosen) {
                    self.cancel(map);
                } else {
                    self.learn(map, self.chosen);
                }
            }
            Key::Delete | Key::Backspace => self.remove(map, self.chosen),
            Key::Escape => {
                if map.armed().is_some() {
                    self.cancel(map);
                } else {
                    *close = true;
                }
            }
            // A letter chooses the next control whose name begins with it.
            Key::Character(c) => {
                let c = c.to_uppercase();
                let n = LEARNABLE.len();
                if let Some(i) = (1..=n)
                    .map(|k| (self.chosen + k) % n)
                    .find(|&i| LEARNABLE[i].label.starts_with(&c))
                {
                    self.choose(i);
                }
            }
            _ => return false,
        }
        true
    }

    /// The parameter chosen in the list.
    pub fn chosen(&self) -> usize {
        self.chosen
    }
}
