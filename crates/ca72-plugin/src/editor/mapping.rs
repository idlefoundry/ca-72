//! The standalone's MIDI map mode (decisions.md R-MIDI-MAP; the owner, 2026-10-27: "click on
//! whatever control they're interested in mapping and then touch a key or wiggle a knob on their
//! MIDI controller and map it like that"). MIDI, after TYPING and PANIC on the walnut, turns it
//! on and off. While it is on, every learnable control in view is marked, its controller in
//! small print; a click chooses a control instead of operating it, and MIDI Learn waits on it
//! (its ring and note: R34); the next controller moved is its own (a switch's, a note played
//! too), and the mode stays on for the next. Delete or Backspace takes the chosen control's
//! controller away; Escape stops it waiting, then ends the mode, as MIDI does. TYPING, PANIC,
//! the rail's keys but the preset arrows (which are learnable), and the presets' drawer work as
//! ever, so that the other pages can be reached. A plug-in has no MIDI key: MIDI Learn's menu
//! and list are its way.

use ca72_panel::learn::MapMark;
use ca72_panel::presets::{BarTarget, DRAWER_TOP};
use ca72_panel::strip::{self, KEY_H, KEY_W, StripControl, StripTarget};
use ca72_panel::{CONTROLS, Kind, Target, art};
use keyboard_types::{Key, KeyState, KeyboardEvent};

use super::{ArpBank, DrawerPlace, Editing, Lower, arp_ui, fx_ui, lower_ui};
use crate::learn::{Binding, MOD_WHEEL, index};
use crate::learning::Place;

/// MIDI's tip.
pub(super) const KEY_TIP: &str = "MIDI MAP: CLICK A CONTROL, THEN MOVE A CONTROLLER";

/// How far a mark stands out round a knob: the learn ring's (`learn::ring`); round a switch, a
/// key or a jack less, clear of the legends printed close round them.
const OUT: f64 = 8.0;
const OUT_SWITCH: f64 = 3.0;

/// The mode: on, and the learnable control last chosen (what Delete takes the controller of).
#[derive(Debug, Default)]
pub(super) struct Mapping {
    pub(super) on: bool,
    pub(super) chosen: Option<usize>,
}

/// A controller in a mark's small print: `CC 74`, a channel but the first named (`CH 2 CC 74`),
/// a note by its number (`NOTE 36`).
fn small_print(b: Binding) -> String {
    let (channel, what) = match b {
        Binding::Cc(c) => (c.channel, format!("CC {}", c.cc)),
        Binding::Note { channel, note } => (channel, format!("NOTE {note}")),
    };
    if channel == 0 {
        what
    } else {
        format!("CH {} {what}", u32::from(channel) + 1)
    }
}

impl Editing {
    /// Whether the mode is on (the standalone's alone).
    pub(super) fn mapping_on(&self) -> bool {
        self.mapping.on && self.playing.standalone
    }

    /// MIDI pressed: the mode on, or off.
    pub(super) fn toggle_mapping(&mut self) {
        if self.mapping.on {
            self.end_mapping();
            return;
        }
        self.end_gestures();
        self.learning.close_menu();
        self.mapping = Mapping {
            on: true,
            chosen: None,
        };
        // (Escape and Delete are the mode's: the keyboard is the editor's while it is on.)
        self.learning.take_keys = true;
    }

    /// The mode off (MIDI again, Escape with nothing waiting, the editor closing): a control
    /// waiting stops, the controllers learned are kept.
    pub(super) fn end_mapping(&mut self) {
        if self.mapping.chosen.is_some() && self.params.midi_map.armed().is_some() {
            let map = std::sync::Arc::clone(&self.params.midi_map);
            self.learning.cancel(&map);
        }
        self.learning.found_at = None;
        self.mapping = Mapping::default();
    }

    /// Whether `rect` (the drawing's units) is in view under the rail: the lower part shown, the
    /// presets' drawer not over it, nor (but for its own) the drawer dropped from the rail.
    fn below_rail_shows(&self, rect: [f64; 4], own: Option<Lower>) -> bool {
        let (x, y) = ((rect[0] + rect[2]) / 2.0, (rect[1] + rect[3]) / 2.0);
        if y < DRAWER_TOP {
            return true;
        }
        if !self.expanded() || self.browser.reveal() > 0.0 {
            return false;
        }
        match self.lower_top() {
            Some(_) if own == Some(self.lower.which) => self.lower.settled(),
            Some(top) => {
                !((lower_ui::LEFT..lower_ui::RIGHT).contains(&x) && y < top + self.lower_depth())
            }
            None => own.is_none(),
        }
    }

    /// Whether the GLOBAL page shows (neither the keyboard nor the LFO page in its place).
    fn global_shows(&self) -> bool {
        self.expanded() && !self.modulating.open && !self.playing.visible
    }

    /// Every learnable control in view: its learnable parameter and where its mark goes (left,
    /// top, right, bottom). The panel's and the left area's, the page's below them (GLOBAL's
    /// knobs and tabs, the LFO page's), the drawer's dropped from the rail (the arpeggiator's,
    /// the expression's, the effects' pins), and the preset arrows on the rail. The MOD wheel's
    /// MIDI LEARN sets this computer's wheel source, not a controller: not here.
    pub(super) fn map_places(&self) -> Vec<(usize, [f64; 4])> {
        let mut out = Vec::new();
        let a = self.renderer.arrangement();
        for i in 0..CONTROLS.len() {
            let c = a.control(i);
            let shown = match c.place {
                DrawerPlace::Strip => self.global_shows(),
                DrawerPlace::Arp => self.lower.which == Lower::Arp && self.lower.open,
                DrawerPlace::Mpe => self.lower.which == Lower::Mpe && self.lower.open,
                _ => true,
            };
            if !shown || self.off_panel(i) {
                continue;
            }
            let Some(k) = self
                .learning
                .learnable_now(Place::Panel(i))
                .filter(|&k| k != MOD_WHEEL)
            else {
                continue;
            };
            let (x, y) = c.centre();
            let b = art::bounds(&c.kind);
            let pad = match c.kind {
                Kind::Knob { .. } | Kind::Selector(_) => OUT,
                _ => OUT_SWITCH,
            };
            let rect = [
                x + b[0] - pad,
                y + b[1] - pad,
                x + b[2] + pad,
                y + b[3] + pad,
            ];
            let own = match c.place {
                DrawerPlace::Arp => Some(Lower::Arp),
                DrawerPlace::Mpe => Some(Lower::Mpe),
                _ => None,
            };
            if self.below_rail_shows(rect, own) {
                out.push((k, rect));
            }
        }
        // GLOBAL's tabs.
        if self.global_shows() {
            for s in [
                StripControl::Poly,
                StripControl::Unison,
                StripControl::Double,
                StripControl::Placement,
                StripControl::AutoGain,
            ] {
                let (x0, y0, x1, y1) = strip::span(s);
                // (MONO and POLY's span and UNISON's meet: apart by a little.)
                let rect = match s {
                    StripControl::Poly => [x0, y0, x1 - 4.0, y1],
                    StripControl::Unison => [x0 + 4.0, y0, x1, y1],
                    _ => [x0, y0, x1, y1],
                };
                if let Some(k) = self.learning.learnable_now(Place::Strip(s))
                    && self.below_rail_shows(rect, None)
                {
                    out.push((k, rect));
                }
            }
        }
        // The arpeggiator's ON, HOLD and DIRECTION (its list's alone until now: R-ARP).
        if self.lower.which == Lower::Arp && self.lower.open {
            let tab = |i: usize| {
                let (x, y) = ArpBank::Arp.tab(i);
                let (w, h) = arp_ui::TAB;
                [
                    x - w / 2.0 - 6.0,
                    y - h / 2.0 - 6.0,
                    x + w / 2.0 + 6.0,
                    y + h / 2.0 + 6.0,
                ]
            };
            let (x, y, w, h) = ArpBank::Direction.recess();
            for (id, rect) in [
                ("arp_on", tab(0)),
                ("arp_latch", tab(1)),
                ("arp_direction", [x, y, x + w, y + h]),
            ] {
                if let Some(k) = index(id)
                    && self.below_rail_shows(rect, Some(Lower::Arp))
                {
                    out.push((k, rect));
                }
            }
        }
        // The effects' pins (R-FX-PARAMS): each its name's screen and its knob.
        if self.fx_pins_show() && !self.fx_view.list_open() {
            for pin in self.fx_view.pins() {
                let Some(place) = self
                    .fx_view
                    .pins()
                    .iter()
                    .filter(|p| p.slot == pin.slot)
                    .position(|p| p.target == pin.target)
                else {
                    continue;
                };
                let (x, y, w, _) = fx_ui::pin_screen(pin.slot, place);
                let (_, ky) = fx_ui::pin_at(pin.slot, place);
                let rect = [x + 2.0, y - 6.0, x + w - 2.0, ky + 48.0];
                if let Some(k) = self.learning.learnable_now(Place::Fx(pin.target)) {
                    out.push((k, rect));
                }
            }
        }
        // PREVIOUS and NEXT PRESET on the rail (R-PRESET-KEYS).
        for t in [BarTarget::Prev, BarTarget::Next] {
            let (x, y) = strip::centre(self.rail, StripTarget::Bar(t));
            let rect = [
                x - KEY_W / 2.0 - 6.0,
                y - KEY_H / 2.0 - 6.0,
                x + KEY_W / 2.0 + 6.0,
                y + KEY_H / 2.0 + 6.0,
            ];
            if let Some(k) = self.learning.learnable_now(Place::Preset(t)) {
                out.push((k, rect));
            }
        }
        out
    }

    /// The marks drawn while the mode is on (none while it is off).
    pub(super) fn map_marks(&self) -> Vec<MapMark> {
        if !self.mapping_on() {
            return Vec::new();
        }
        let map = &self.params.midi_map;
        let bindings = map.bindings();
        let armed = map.armed();
        self.map_places()
            .into_iter()
            .map(|(k, rect)| MapMark {
                rect,
                label: bindings.get(k).copied().flatten().map(small_print),
                waiting: armed == Some(k),
                chosen: self.mapping.chosen == Some(k),
            })
            .collect()
    }

    /// The learnable control whose mark is at (`x`, `y`), and its mark (the nearest middle
    /// where two meet).
    pub(super) fn map_at(&self, (x, y): (f64, f64)) -> Option<(usize, [f64; 4])> {
        let d = |r: &[f64; 4]| {
            let (cx, cy) = ((r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0);
            (x - cx).powi(2) + (y - cy).powi(2)
        };
        self.map_places()
            .into_iter()
            .filter(|(_, r)| (r[0]..r[2]).contains(&x) && (r[1]..r[3]).contains(&y))
            .min_by(|a, b| d(&a.1).total_cmp(&d(&b.1)))
    }

    /// A press while the mode is on: whether it was the mode's (a control chosen, or nothing
    /// done: a click operates nothing). TYPING, PANIC and MIDI, the presets' drawer, the rail's
    /// keys but the preset arrows, and the grip are the editor's as ever.
    pub(super) fn map_press(&mut self, (x, y): (f64, f64)) -> bool {
        if art::tempo_key_hit(self.renderer.arrangement(), x, y).is_some()
            || self.in_drawer((x, y)).is_some()
        {
            return false;
        }
        if let Some((k, rect)) = self.map_at((x, y)) {
            self.map_choose(k, rect);
            return true;
        }
        if let Some(StripTarget::Bar(_)) = strip::hit(self.rail, x, y) {
            return false;
        }
        self.hit(x, y) != Some(Target::Grip)
    }

    /// Learnable `k` chosen at its mark `rect`: MIDI Learn waits on it (already waiting, it
    /// waits on).
    fn map_choose(&mut self, k: usize, rect: [f64; 4]) {
        self.mapping.chosen = Some(k);
        self.learning.found_at = Some((k, ((rect[0] + rect[2]) / 2.0, rect[1])));
        let map = std::sync::Arc::clone(&self.params.midi_map);
        if map.armed() != Some(k) {
            self.learning.learn_with_notes(&map, k);
        }
    }

    /// A key while the mode is on: whether it was the mode's. Escape stops a control waiting
    /// (none chosen then), else ends the mode; Delete or Backspace takes the chosen control's
    /// controller away (as the MIDI list's do).
    pub(super) fn map_key(&mut self, k: &KeyboardEvent) -> bool {
        if !self.mapping_on() {
            return false;
        }
        let map = std::sync::Arc::clone(&self.params.midi_map);
        match k.key {
            Key::Escape => {
                if k.state == KeyState::Down {
                    if map.armed().is_some() {
                        self.learning.cancel(&map);
                        self.mapping.chosen = None;
                    } else {
                        self.end_mapping();
                    }
                }
                true
            }
            Key::Delete | Key::Backspace => {
                if k.state == KeyState::Down
                    && let Some(i) = self.mapping.chosen
                {
                    self.learning.remove(&map, i);
                }
                true
            }
            _ => false,
        }
    }
}
