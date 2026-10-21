//! B1's pins as drawer controls (decisions.md R-FX-PARAMS). Their control numbers follow
//! the panel's, as TEMPO's does; their targets are the chain's fixed pin numbers.

use ca72_panel::strip::fx::FxTarget;
use ca72_panel::{CONTROLS, art};
use keyboard_types::Modifiers;

use super::{Editing, Lower, MENU_TEXT, fx_ui};
use crate::learning::{ModItem, Place, Row};
use crate::lfo::{self, Target as ModTarget};

#[derive(Debug)]
pub(super) struct PinDrag {
    target: usize,
    from: f64,
    value: f64,
    fine: bool,
}

pub(super) const FIRST: usize = CONTROLS.len();

pub(super) fn target(control: usize) -> Option<usize> {
    control.checked_sub(FIRST).filter(|&n| n < lfo::FX_PINS)
}

impl Editing {
    pub(super) fn fx_pins_show(&self) -> bool {
        self.lower.which == Lower::Fx
            && self.lower.settled()
            && self.lower.open
            && self.expanded()
            && !self.browser.open
    }

    pub(super) fn fx_pin_at(&self, target: usize) -> Option<(f64, f64)> {
        let pin = self.fx_view.pin_info(target)?;
        let place = self
            .fx_view
            .pins()
            .iter()
            .filter(|p| p.slot == pin.slot)
            .position(|p| p.target == target)?;
        Some(fx_ui::pin_at(pin.slot, place))
    }

    pub(super) fn fx_pin_hit(&self, at: (f64, f64)) -> Option<usize> {
        if !self.fx_pins_show() || self.fx_view.list_open() {
            return None;
        }
        let FxTarget::Pin(slot, place) = self.fx_scene.hit(at.0, at.1)? else {
            return None;
        };
        self.fx_scene.pin(slot, place).map(|p| p.target)
    }

    pub(super) fn mod_live(&self, control: usize) -> bool {
        target(control).is_none_or(|t| self.fx_view.pin_info(t).is_some_and(|p| p.live))
    }

    pub(super) fn fx_pin_press(&mut self, at: (f64, f64), modifiers: Modifiers) -> bool {
        if self.fx_view.list_open() {
            return false;
        }
        if self.mod_press(at, modifiers) {
            return true;
        }
        let Some(t) = self.fx_pin_hit(at) else {
            return false;
        };
        let Some(pin) = self.fx_view.pin_info(t).filter(|p| p.live) else {
            return true;
        };
        self.fx_pin_drag = Some(PinDrag {
            target: t,
            from: self.pointer.1,
            value: pin.value,
            fine: modifiers.contains(Modifiers::SHIFT),
        });
        true
    }

    pub(super) fn fx_pin_moved(&mut self, modifiers: Modifiers) -> bool {
        let Some(drag) = &mut self.fx_pin_drag else {
            return false;
        };
        let fine = modifiers.contains(Modifiers::SHIFT);
        if fine != drag.fine {
            drag.from = self.pointer.1;
            drag.value = self
                .fx_view
                .pin_info(drag.target)
                .map_or(drag.value, |p| p.value);
            drag.fine = fine;
        }
        self.fx_view.set_pin_value(
            drag.target,
            drag.value + (drag.from - self.pointer.1) / 240.0 * if fine { 0.1 } else { 1.0 },
        );
        true
    }

    pub(super) fn fx_pin_menu(&mut self, at: (f64, f64)) -> bool {
        let Some(t) = self.fx_pin_hit(at) else {
            return false;
        };
        let pin = self.fx_view.pin_info(t).expect("a pin");
        let title = format!("{}, {}", pin.name, pin.effect).to_uppercase();
        let mut rows = self.depth_rows(ModTarget::Fx(t), FIRST + t);
        if !pin.live {
            for row in &mut rows {
                if let Row::Item(_, _, enabled, _) = row {
                    *enabled = false;
                }
            }
        }
        rows.push(Row::Item(ModItem::Unpin(t), "UNPIN".into(), true, false));
        self.learning.open_menu_with(
            self.renderer.fonts(),
            &self.params.midi_map,
            Some(Place::Fx(t)),
            None,
            at,
            self.renderer.text_size() * MENU_TEXT,
            rows,
        );
        self.learning.pin_menu_title(title);
        true
    }

    pub(super) fn fx_pin_tip(&self) -> Option<(String, f64, f64)> {
        if self.modulating.patching.is_some() {
            return None;
        }
        let t = self
            .fx_pin_drag
            .as_ref()
            .map(|d| d.target)
            .or_else(|| self.fx_pin_hit(self.in_drawing(self.pointer)))?;
        let p = self.fx_view.pin_info(t)?;
        let (x, y) = self.fx_pin_at(t)?;
        let text = format!(
            "{}, {}: {}{}",
            p.name.to_uppercase(),
            p.effect.to_uppercase(),
            if p.missing {
                "MISSING"
            } else {
                self.fx_view.pin_text(t)
            },
            self.depths_tip(FIRST + t)
        );
        let bounds = art::tip_bounds(self.renderer.fonts(), &text, self.renderer.text_size());
        let half = (bounds[2] - bounds[0]) / 2.0;
        Some((
            text,
            x.clamp(half.min(art::W / 2.0), (art::W - half).max(art::W / 2.0)),
            y - art::STD.skirt - 4.0,
        ))
    }

    pub(super) fn target_label(&self, t: ModTarget) -> String {
        if let ModTarget::Fx(n) = t
            && let Some(p) = self.fx_view.pin_info(n)
        {
            return format!("{}, {}", p.name, p.effect).to_uppercase();
        }
        super::lfo_page::control_of(t)
            .map_or(t.short(), super::lfo_page::control_label)
            .into()
    }

    pub(super) fn fx_scene_now(&self) -> fx_ui::FxScene {
        let mut scene = self.fx_view.scene();
        for pin in &mut scene.pins {
            pin.learning = self.params.midi_map.armed()
                == crate::learn::index(lfo::Target::Fx(pin.target).id());
        }
        if let Some(list) = &mut scene.list
            && list.find.is_some()
        {
            for row in &mut list.rows {
                if let Some(n) = row.pinned {
                    row.format = self
                        .sources_on(ModTarget::Fx(n))
                        .iter()
                        .map(|&(s, d)| {
                            format!(
                                "{} {:+.0}",
                                super::lfo_page::label(&self.params.lfo_patch, s),
                                d * 100.0
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" · ");
                }
            }
        }
        scene
    }
}
