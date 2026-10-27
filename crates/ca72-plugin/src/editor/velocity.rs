//! The velocity in the editor (decisions.md R-VELOCITY): the left hand's VELOCITY jack, shown
//! with the wheels (not with the macros, which are laid over them), an assignment dragged from it
//! as from an LFO's OUT; its meter lit to the latest note; its menu and tip. Everything of
//! modulation (the cord, the rings, the pop-up, the menus' heads, Alt-drag) is `lfo_page`'s, over
//! sources: the velocity is `Source::Velocity`. And the on-screen keyboard's notes struck harder
//! the further down a key the pointer is.

use ca72_panel::keyboard;
use ca72_panel::modulation as mods;
use ca72_panel::strip::velocity as velocity_ui;

use super::{Editing, MENU_TEXT};
use crate::learning::{ModItem, Row};
use crate::lfo::Source;

/// The softest and the hardest a key of the on-screen keyboard is struck (MIDI's 1 to 127), at
/// its top and at its foot.
const KEY_SOFTEST: f64 = 1.0;
const KEY_HARDEST: f64 = 127.0;

impl Editing {
    /// Whether the left hand's VELOCITY shows: the window's lower part, the wheels (not the
    /// macros) in its left area.
    pub(super) fn velocity_shows(&self) -> bool {
        self.expanded() && !self.macros_show()
    }

    /// A press on the jack (it showing): an assignment dragged from it. Whether it was the jack's.
    pub(super) fn velocity_press(&mut self, (x, y): (f64, f64)) -> bool {
        if !self.velocity_shows() || !velocity_ui::hit_jack(x, y) {
            return false;
        }
        self.modulating.last = Some(Source::Velocity);
        self.modulating.patching = Some(super::lfo_page::Patching {
            source: Source::Velocity,
            at: (x, y),
            from: (x, y),
            away: false,
        });
        self.modulating.popup = None;
        true
    }

    /// Whether the pointer is on the jack (it showing): its tip, its cords. Where the pointer is
    /// now, not where a hover last left it (a cord dragged off it is no hover).
    pub(super) fn on_velocity_jack(&self) -> bool {
        let (x, y) = self.in_drawing(self.pointer);
        self.velocity_shows() && velocity_ui::hit_jack(x, y)
    }

    /// The jack's right-click menu: REMOVE ALL ASSIGNMENTS, then a head for each knob it
    /// modulates with DEPTH… and REMOVE. Whether the press was the jack's.
    pub(super) fn velocity_menu(&mut self, (x, y): (f64, f64)) -> bool {
        if !self.velocity_shows() || !velocity_ui::hit_jack(x, y) {
            return false;
        }
        let s = Source::Velocity;
        let mut rows = vec![Row::Item(
            ModItem::RemoveAll(s),
            "REMOVE ALL ASSIGNMENTS".into(),
            self.params.lfo_patch.count(s) > 0,
            false,
        )];
        for a in self.params.lfo_patch.assignments(s).into_iter().flatten() {
            let Some(i) = super::lfo_page::control_of(a.target) else {
                continue;
            };
            rows.push(Row::Head(format!(
                "{} \u{b7} {}",
                self.target_label(a.target),
                mods::percent(f64::from(a.depth)),
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
        let size = self.renderer.text_size() * MENU_TEXT;
        self.learning
            .open_rows_menu(self.renderer.fonts(), (x, y), size, "VELOCITY".into(), rows);
        true
    }

    /// The jack's tip: what it modulates, else what it does.
    pub(super) fn velocity_tip(&self) -> Option<(String, f64, f64)> {
        if !self.on_velocity_jack() || !self.velocity_shows() || self.modulating.patching.is_some()
        {
            return None;
        }
        let a: Vec<String> = self
            .params
            .lfo_patch
            .assignments(Source::Velocity)
            .into_iter()
            .flatten()
            .map(|a| {
                format!(
                    "{} {}",
                    self.target_label(a.target),
                    mods::percent(f64::from(a.depth))
                )
            })
            .collect();
        let text = if a.is_empty() {
            "VELOCITY: DRAG TO A KNOB TO MODULATE IT".to_owned()
        } else {
            format!("VELOCITY MODULATES {}", a.join(" \u{b7} "))
        };
        let (x, y) = velocity_ui::jack_at();
        let size = self.renderer.text_size();
        let w = self
            .renderer
            .fonts()
            .advance(&text, size, ca72_panel::fonts::Weight::Regular, 0.0)
            + 0.8 * size;
        Some((text, x.max(w / 2.0 + 10.0), y - velocity_ui::JACK_R - 18.0))
    }
}

/// How hard the on-screen keyboard's key `note` is struck with the pointer at height `y` in the
/// drawing (decisions.md R-VELOCITY): softest at the key's top, hardest at its foot, as a finger
/// further down a key strikes it with more of its weight. MIDI's 1 to 127.
pub(super) fn key_velocity(root: u8, note: u8, y: f64) -> u8 {
    let Some([_, top, _, bottom]) = keyboard::rect(root, note) else {
        return crate::local_notes::TYPED_VELOCITY;
    };
    let down = if bottom > top {
        ((y - top) / (bottom - top)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    (KEY_SOFTEST + down * (KEY_HARDEST - KEY_SOFTEST)).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_struck_harder_further_down() {
        let root = 41;
        let note = 48;
        let [_, top, _, bottom] = keyboard::rect(root, note).expect("on the keyboard");
        assert_eq!(key_velocity(root, note, top), 1);
        assert_eq!(key_velocity(root, note, bottom), 127);
        let mid = key_velocity(root, note, (top + bottom) / 2.0);
        assert!((60..=68).contains(&mid), "{mid}");
        assert_eq!(key_velocity(root, note, bottom + 100.0), 127);
    }
}
