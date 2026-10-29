//! The PARAMETERS list and B1's pins (decisions.md R-FX-PARAMS). Only opening the list asks
//! for the catalogue; the tenth-second poll reads the pins, keeping them while the chain is busy.

use std::collections::BTreeMap;

use ca72_panel::presets::Field;
use ca72_panel::strip::fx::{FxFind, FxPin};
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};

use super::*;

#[derive(Debug)]
pub(super) struct Parameters {
    pub slot: usize,
    pub request: u64,
    pub field: Field,
    pub pinned: bool,
    pub rows: Vec<(ParamRow, String)>,
    found: Vec<Option<usize>>,
    touched: Vec<u32>,
    replace: Option<usize>,
    pub first: usize,
    pub chosen: usize,
    pub note: Option<String>,
}

pub(super) type RecentReply = (usize, u64, Result<Vec<ParamRow>, String>);

pub(super) type ParameterReply = (usize, u64, Result<Vec<(ParamRow, String)>, String>);

impl Parameters {
    pub(super) fn found(&self) -> &[Option<usize>] {
        &self.found
    }

    pub(super) fn refilter(&mut self) {
        let usual = search_params(
            &self.rows.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>(),
            &self.field.text,
            self.pinned,
        );
        self.found.clear();
        if self.field.text.is_empty() && !self.pinned {
            let recent: Vec<_> = self
                .touched
                .iter()
                .filter_map(|id| self.rows.iter().position(|(r, _)| r.id == *id))
                .take(4)
                .collect();
            if !recent.is_empty() {
                self.found.push(None);
                self.found.extend(recent.into_iter().map(Some));
                self.found.push(None);
            }
        }
        self.found.extend(usual.into_iter().map(Some));
        if self.found.get(self.chosen) == Some(&None) {
            self.chosen = self.found.iter().position(Option::is_some).unwrap_or(0);
        }
    }

    fn navigate(&mut self, by: i32) {
        let n = self.found.len();
        if n == 0 {
            return;
        }
        let next = (self.chosen as i32 + by).clamp(0, n as i32 - 1) as usize;
        let chosen = if by < 0 {
            (0..=next)
                .rev()
                .find(|&i| self.found[i].is_some())
                .or_else(|| (next..n).find(|&i| self.found[i].is_some()))
        } else {
            (next..n)
                .find(|&i| self.found[i].is_some())
                .or_else(|| (0..=next).rev().find(|&i| self.found[i].is_some()))
        };
        if let Some(chosen) = chosen {
            self.chosen = chosen;
        }
    }

    fn keep_in_view(&mut self) {
        let n = self.found().len();
        self.chosen = self.chosen.min(n.saturating_sub(1));
        let shown = LIST_ROWS;
        self.first = self.first.min(n.saturating_sub(shown));
        if self.chosen < self.first {
            self.first = self.chosen;
        }
        if self.chosen >= self.first + shown {
            self.first = self.chosen + 1 - shown;
        }
    }
}

impl FxView {
    pub fn pins(&self) -> &[PinInfo] {
        &self.pins
    }

    pub fn pin_info(&self, target: usize) -> Option<&PinInfo> {
        self.pins.iter().find(|p| p.target == target)
    }

    pub fn pin_text(&self, target: usize) -> &str {
        self.pin_texts.get(&target).map_or("…", String::as_str)
    }

    pub(super) fn poll_parameters(&mut self, now: Instant) {
        let Some(e) = self.effects.clone() else {
            return;
        };
        for slot in 0..SLOTS {
            if let Some(ids) = e.touched(slot)
                && (ids != self.touched[slot]
                    || self.recent_retry[slot].is_some_and(|at| now >= at))
            {
                self.touched[slot] = ids;
                self.recent[slot].clear();
                self.recent_request[slot] += 1;
                self.recent_retry[slot] = None;
                if !self.touched[slot].is_empty() {
                    self.submit(Job::Recent(slot, self.recent_request[slot]));
                }
            }
        }
        if let Some(pins) = e.pins() {
            if pins != self.pins {
                self.pin_texts.retain(|target, _| {
                    self.pins
                        .iter()
                        .find(|p| p.target == *target)
                        .is_some_and(|old| {
                            pins.iter().any(|new| {
                                new.target == *target
                                    && new.id == old.id
                                    && new.effect == old.effect
                            })
                        })
                });
                self.submit(Job::Texts(pins.clone()));
            }
            self.pins = pins;
        }
        let mut p = lock(&self.pending);
        for (slot, request, rows) in p.recent.drain(..) {
            if self.recent_request[slot] == request {
                match rows {
                    Ok(rows) => self.recent[slot] = rows,
                    Err(_) => self.recent_retry[slot] = Some(now + Duration::from_secs(1)),
                }
            }
        }
        if let Some((slot, request, reply)) = p.parameters.take()
            && let Some(list) = &mut self.parameters
            && list.slot == slot
            && list.request == request
        {
            match reply {
                Ok(rows) => {
                    list.rows = rows;
                    list.refilter();
                    list.note = None;
                }
                Err(error) => list.note = Some(error),
            }
        }
        if let Some(error) = p.pin_error.take()
            && let Some(list) = &mut self.parameters
        {
            list.note = Some(error);
        }
        for (pin, text) in p.pin_texts.drain(..) {
            if self
                .pins
                .iter()
                .any(|now| now.target == pin.target && now.id == pin.id && now.effect == pin.effect)
            {
                self.pin_texts.insert(pin.target, text);
            }
        }
        drop(p);
        if let Some(list) = &mut self.parameters {
            let mut changed = false;
            for (row, text) in &mut list.rows {
                let target = self
                    .pins
                    .iter()
                    .find(|p| p.slot == list.slot && p.id == row.id)
                    .map(|p| p.target);
                changed |= row.pinned != target;
                row.pinned = target;
                if let Some(t) = row.pinned
                    && let Some(now) = self.pin_texts.get(&t)
                {
                    text.clone_from(now);
                }
            }
            if changed {
                list.refilter();
            }
            list.keep_in_view();
        }
    }

    pub(super) fn offer(&self, slot: usize) -> Option<&ParamRow> {
        let id = *self.touched.get(slot)?.first()?;
        if self.pins.iter().any(|p| p.slot == slot && p.id == id) {
            return None;
        }
        self.recent[slot].iter().find(|r| r.id == id)
    }

    pub fn open_repin(&mut self, target: usize) {
        let Some(pin) = self.pin_info(target) else {
            return;
        };
        let slot = pin.slot;
        self.open_parameters(slot);
        if let Some(list) = &mut self.parameters {
            list.replace = Some(target);
        }
    }

    /// The helper is asked once, on the drawer's load thread; the list shows LOADING meanwhile.
    pub fn open_parameters(&mut self, slot: usize) {
        if self.effect(slot).is_none() {
            return;
        }
        self.close_list();
        self.request += 1;
        self.parameters = Some(Parameters {
            slot,
            request: self.request,
            field: Field::default(),
            pinned: false,
            rows: Vec::new(),
            found: Vec::new(),
            touched: self.touched[slot].clone(),
            replace: None,
            first: 0,
            chosen: 0,
            note: Some("LOADING…".into()),
        });
        self.submit(Job::Parameters(slot, self.request));
    }

    pub(super) fn parameters_list(&self) -> Option<FxList> {
        let p = self.parameters.as_ref()?;
        let found = p.found();
        let count = self.pins.iter().filter(|pin| pin.slot == p.slot).count();
        Some(FxList {
            slot: p.slot,
            title: p
                .replace
                .and_then(|t| self.pin_info(t))
                .map(|pin| format!("REPLACE {}", pin.name.to_uppercase()))
                .unwrap_or_else(|| {
                    format!(
                        "{}: PARAMETERS",
                        self.effect(p.slot)
                            .map_or("", |e| e.name.as_str())
                            .to_uppercase()
                    )
                }),
            note: p
                .note
                .clone()
                .unwrap_or_else(|| format!("{count} OF 4 PINNED")),
            rows: found
                .iter()
                .enumerate()
                .map(|(pos, &i)| {
                    let Some(i) = i else {
                        return FxRow {
                            name: if pos == 0 {
                                "JUST TOUCHED"
                            } else {
                                "ALL PARAMETERS"
                            }
                            .into(),
                            count: Some(if pos == 0 {
                                p.touched
                                    .iter()
                                    .filter(|id| p.rows.iter().any(|(r, _)| r.id == **id))
                                    .count()
                            } else {
                                p.rows.len()
                            }),
                            ..FxRow::default()
                        };
                    };
                    let (r, text) = &p.rows[i];
                    FxRow {
                        name: r.name.clone(),
                        maker: text.clone(),
                        pinned: r.pinned,
                        matches: matched(&r.name, &p.field.text),
                        ..FxRow::default()
                    }
                })
                .collect(),
            first: p.first,
            hover: (!found.is_empty()).then_some(p.chosen),
            find: Some(FxFind {
                field: p.field.clone(),
                pinned: p.pinned,
            }),
            ..FxList::default()
        })
    }

    pub(super) fn drawn_pins(&self) -> Vec<FxPin> {
        self.pins
            .iter()
            .map(|p| FxPin {
                target: p.target,
                slot: p.slot,
                name: if p.missing {
                    "MISSING".into()
                } else if p.short.is_empty() {
                    p.name.to_uppercase()
                } else {
                    p.short.to_uppercase()
                },
                value: p.value,
                dim: !p.live,
                learning: false,
                text: None,
            })
            .collect()
    }

    pub(super) fn choose_parameter(&mut self, row: usize) {
        let Some(list) = &mut self.parameters else {
            return;
        };
        let Some(&Some(i)) = list.found().get(row) else {
            return;
        };
        let (r, _) = &list.rows[i];
        let job = if let Some(target) = list.replace {
            Job::Repin(target, r.id)
        } else {
            match r.pinned {
                Some(t) => Job::Unpin(t),
                None => Job::Pin(list.slot, r.id),
            }
        };
        list.note = None;
        self.submit(job);
    }

    pub fn unpin(&mut self, target: usize) {
        self.submit(Job::Unpin(target));
    }

    /// A turn is a base value, stepped on the parameter's own travel. An inert pin takes none.
    pub fn set_pin_value(&mut self, target: usize, travel: f64) {
        let Some(pin) = self.pins.iter_mut().find(|p| p.target == target && p.live) else {
            return;
        };
        let mut value = travel.clamp(0.0, 1.0);
        if pin.steps > 0 {
            value = (value * f64::from(pin.steps)).round() / f64::from(pin.steps);
        }
        if let Some(e) = &self.effects
            && e.set_pin_value(target, value).is_ok()
        {
            pin.value = value;
            self.polled = None;
        }
    }

    /// The list owns all keys while open, using the presets' text field and editing rules.
    pub fn list_key(&mut self, key: &KeyboardEvent) -> bool {
        if !self.list_open() {
            return false;
        }
        if key.state != KeyState::Down {
            return true;
        }
        let Some(p) = &mut self.parameters else {
            if key.key == Key::Escape {
                self.close_list();
            }
            return true;
        };
        let before = p.field.text.clone();
        let pinned = p.pinned;
        match &key.key {
            Key::Escape if p.field.text.is_empty() => self.close_list(),
            Key::Escape => {
                p.field = Field::default();
                p.chosen = 0;
                p.first = 0;
            }
            Key::Enter => {
                let row = p.chosen;
                self.choose_parameter(row);
            }
            Key::ArrowUp => p.navigate(-1),
            Key::ArrowDown => p.navigate(1),
            Key::PageUp => p.navigate(-(LIST_ROWS as i32)),
            Key::PageDown => p.navigate(LIST_ROWS as i32),
            Key::ArrowLeft => p.field.left(),
            Key::ArrowRight => p.field.right(),
            Key::Home => p.field.home(),
            Key::End => p.field.end(),
            Key::Backspace => p.field.backspace(),
            Key::Delete => p.field.delete(),
            Key::Tab => {
                p.pinned = !p.pinned;
                p.chosen = 0;
                p.first = 0;
            }
            Key::Character(c)
                if !key
                    .modifiers
                    .intersects(Modifiers::CONTROL | Modifiers::META | Modifiers::ALT) =>
            {
                p.field.insert(c)
            }
            _ => {}
        }
        if let Some(p) = &mut self.parameters {
            if p.field.text != before || p.pinned != pinned {
                p.chosen = 0;
                p.first = 0;
                p.refilter();
            }
            p.keep_in_view();
        }
        true
    }
}

fn matched(name: &str, typed: &str) -> Vec<usize> {
    let folded: Vec<(char, usize)> = name
        .chars()
        .enumerate()
        .flat_map(|(i, c)| c.to_lowercase().map(move |c| (c, i)))
        .collect();
    let chars: Vec<char> = folded.iter().map(|(c, _)| *c).collect();
    let mut out = Vec::new();
    for word in typed.split_whitespace() {
        let word: Vec<char> = word.to_lowercase().chars().collect();
        for (i, part) in chars.windows(word.len()).enumerate() {
            if part == word {
                out.extend(folded[i..i + word.len()].iter().map(|(_, i)| *i));
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

pub(super) fn texts(e: &dyn Effects, pins: Vec<PinInfo>) -> Vec<(PinInfo, String)> {
    let mut groups: BTreeMap<usize, Vec<PinInfo>> = BTreeMap::new();
    for p in pins.into_iter().filter(|p| p.live && p.plain.is_some()) {
        groups.entry(p.slot).or_default().push(p);
    }
    let mut out = Vec::new();
    for (slot, pins) in groups {
        let values: Vec<_> = pins
            .iter()
            .map(|p| (p.id, p.plain.unwrap_or_default()))
            .collect();
        for (pin, text) in pins.into_iter().zip(e.param_text(slot, &values)) {
            out.push((pin, text.unwrap_or_else(|| "…".into())));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx_view::fake::Fake;
    use std::sync::atomic::Ordering;

    fn settle(v: &mut FxView, done: impl Fn(&FxView) -> bool) {
        let until = Instant::now() + Duration::from_secs(3);
        loop {
            v.polled = None;
            v.poll(Instant::now());
            if done(v) {
                return;
            }
            assert!(Instant::now() < until, "the drawer's job finished");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn key(v: &mut FxView, key: Key) {
        assert!(v.list_key(&KeyboardEvent {
            key,
            state: KeyState::Down,
            ..Default::default()
        }));
    }

    #[test]
    fn parameters_are_searched_in_name_order_and_take_the_keys() {
        let fake = Fake::with(&["Hall"], &[]);
        fake.parameters(
            0,
            &[
                "Mix",
                "Pre-delay",
                "Decay",
                "Damping",
                "Early Mix",
                "Width",
                "Size",
                "Tone",
                "Low Cut",
                "High Cut",
                "Diffusion",
                "Shape",
                "Late Mix",
            ],
        );
        let mut v = FxView::new(Some(fake.clone()));
        v.poll(Instant::now());
        v.open_parameters(0);
        settle(&mut v, |v| {
            v.parameters.as_ref().is_some_and(|p| p.rows.len() == 13)
        });
        let names = || {
            [
                "Damping",
                "Decay",
                "Diffusion",
                "Early Mix",
                "High Cut",
                "Late Mix",
                "Low Cut",
                "Mix",
                "Pre-delay",
                "Shape",
                "Size",
                "Tone",
                "Width",
            ]
        };
        assert_eq!(
            v.scene()
                .list
                .unwrap()
                .rows
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>(),
            names()
        );
        key(&mut v, Key::PageDown);
        assert_eq!(v.scene().list.unwrap().hover, Some(10));
        key(&mut v, Key::ArrowDown);
        assert_eq!(v.scene().list.unwrap().hover, Some(11));
        key(&mut v, Key::PageUp);
        key(&mut v, Key::ArrowUp);
        assert_eq!(v.scene().list.unwrap().hover, Some(0));
        key(&mut v, Key::Character("mix".into()));
        let l = v.scene().list.unwrap();
        assert_eq!(
            l.rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["Early Mix", "Late Mix", "Mix"]
        );
        assert_eq!(l.rows[0].matches, [6, 7, 8]);
        key(&mut v, Key::Home);
        key(&mut v, Key::Delete);
        key(&mut v, Key::End);
        key(&mut v, Key::ArrowLeft);
        key(&mut v, Key::Backspace);
        assert_eq!(v.parameters.as_ref().unwrap().field.text, "x");
        key(&mut v, Key::Escape);
        assert!(v.list_open());
        assert_eq!(v.parameters.as_ref().unwrap().field.text, "");
        assert_eq!(
            fake.param_reads.load(Ordering::Relaxed),
            1,
            "typing never asks the helper for its catalogue"
        );
        key(&mut v, Key::Escape);
        assert!(!v.list_open());
    }

    #[test]
    fn parameters_pin_unpin_and_refuse_the_fifth_on_the_plate() {
        let fake = Fake::with(&["Hall"], &[]);
        fake.parameters(0, &["Decay", "Damping", "Diffusion", "Mix", "Width"]);
        let mut v = FxView::new(Some(fake.clone()));
        v.poll(Instant::now());
        v.open_parameters(0);
        settle(&mut v, |v| v.parameters.as_ref().unwrap().rows.len() == 5);
        for row in 0..4 {
            if row == 0 {
                let l = v.scene().list.unwrap();
                let (x, y) = fx_ui::row_centre(&l, row);
                v.press(x, y);
            } else {
                v.parameters.as_mut().unwrap().chosen = row;
                key(&mut v, Key::Enter);
            }
            settle(&mut v, |v| v.pins.len() == row + 1);
        }
        assert_eq!(v.scene().depth(), fx_ui::PIN_DEPTH);
        v.choose_parameter(4);
        settle(&mut v, |v| {
            v.parameters.as_ref().unwrap().note.as_deref() == Some("4 OF 4 PINNED")
        });
        assert_eq!(v.pins.len(), 4);
        let l = v.scene().list.unwrap();
        let (x, y, w, _) = fx_ui::list_rect(&l);
        v.press(x + w - 160.0, y + 90.0);
        assert!(v.parameters.as_ref().unwrap().pinned);
        assert_eq!(v.scene().list.unwrap().rows.len(), 4);
        key(&mut v, Key::Tab);
        assert_eq!(v.scene().list.unwrap().rows.len(), 5);
        key(&mut v, Key::Tab);
        assert_eq!(v.scene().list.unwrap().rows.len(), 4);
        key(&mut v, Key::Enter);
        settle(&mut v, |v| v.pins.len() == 3);
        let list = v.scene().list.unwrap();
        let (rx, ry) = fx_ui::row_centre(&list, 0);
        v.press(rx, ry);
        settle(&mut v, |v| v.pins.len() == 2);
        let keep = v.pins.clone();
        fake.pins_busy.store(true, Ordering::Relaxed);
        v.polled = None;
        v.poll(Instant::now());
        assert_eq!(v.pins, keep, "a busy chain keeps the last pins");
        fake.pins_busy.store(false, Ordering::Relaxed);
        for p in keep {
            v.unpin(p.target);
        }
        settle(&mut v, |v| v.pins.is_empty());
        assert_eq!(v.scene().depth(), fx_ui::PIN_DEPTH);
    }
    #[test]
    fn loaded_effects_offer_the_last_touch_and_the_split_field_and_swap_take_one_click() {
        let fake = Fake::with(&["Room"], &[]);
        fake.parameters(0, &["Mix", "Rate", "Decay", "Size", "Damping"]);
        let mut v = FxView::new(Some(fake.clone()));
        v.poll(Instant::now());
        assert_eq!(v.scene().depth(), fx_ui::PIN_DEPTH);
        let field = |place| {
            let (x, y, _, h) = fx_ui::pin_screen(0, place);
            (x + 10.0, y + h / 2.0)
        };
        let (x, y) = field(0);
        v.press(x, y);
        settle(&mut v, |v| {
            v.parameters.as_ref().is_some_and(|p| p.rows.len() == 5)
        });
        key(&mut v, Key::Escape);
        fake.touched.lock().unwrap().insert(0, vec![1, 2, 0]);
        settle(&mut v, |v| v.offer(0).is_some_and(|r| r.id == 1));
        assert_eq!(v.scene().offers[0].as_deref(), Some("+ RATE"));
        let (sx, sy, sw, sh) = fx_ui::pin_screen(0, 0);
        v.press(sx + sw - 12.0, sy + sh / 2.0);
        settle(&mut v, |v| {
            v.parameters.as_ref().is_some_and(|p| p.rows.len() == 5)
        });
        let list = v.scene().list.unwrap();
        assert_eq!(list.rows[0].name, "JUST TOUCHED");
        assert!(list.rows[0].count.is_some());
        assert_eq!(list.rows[1].name, "Rate");
        assert_eq!(list.rows[2].name, "Decay");
        assert_eq!(list.hover, Some(1));
        key(&mut v, Key::PageDown);
        key(&mut v, Key::PageUp);
        assert_eq!(
            v.scene().list.unwrap().hover,
            Some(1),
            "Page Up skips the first head"
        );
        key(&mut v, Key::ArrowDown);
        key(&mut v, Key::ArrowUp);
        assert_eq!(v.scene().list.unwrap().hover, Some(1));
        key(&mut v, Key::Character("mix".into()));
        assert!(
            v.scene()
                .list
                .unwrap()
                .rows
                .iter()
                .all(|r| r.count.is_none())
        );
        key(&mut v, Key::Escape);
        key(&mut v, Key::Tab);
        assert!(
            v.scene()
                .list
                .unwrap()
                .rows
                .iter()
                .all(|r| r.count.is_none())
        );
        key(&mut v, Key::Escape);
        v.press(x, y);
        settle(&mut v, |v| v.pins.len() == 1);
        assert_eq!(v.pins[0].id, 1);
        assert!(
            v.scene().offers[0].is_none(),
            "a pinned touch offers nothing"
        );
        fake.touched.lock().unwrap().insert(0, vec![999, 1]);
        settle(&mut v, |v| v.touched[0].first() == Some(&999));
        assert!(
            v.scene().offers[0].is_none(),
            "an unpinnable touch offers nothing"
        );
        fake.touched.lock().unwrap().insert(0, vec![2, 1]);
        settle(&mut v, |v| v.offer(0).is_some_and(|r| r.id == 2));
        assert_eq!(v.scene().offers[0].as_deref(), Some("+ DECAY"));
        v.press(x, y);
        settle(&mut v, |v| {
            v.parameters.as_ref().is_some_and(|p| p.rows.len() == 5)
        });
        assert_eq!(v.scene().list.unwrap().title, "REPLACE RATE");
        let before = v.pins.clone();
        key(&mut v, Key::Escape);
        assert_eq!(v.pins, before, "Escape swaps nothing");
        v.press(x, y);
        settle(&mut v, |v| {
            v.parameters.as_ref().is_some_and(|p| p.rows.len() == 5)
        });
        v.choose_parameter(1); // Decay, at the head of JUST TOUCHED.
        settle(&mut v, |v| v.pins[0].id == 2);
        assert_eq!(v.pins[0].target, before[0].target);
        assert!(v.scene().offers[0].is_none());
        v.close_list();
        for id in [0, 1] {
            fake.pin(0, id).unwrap();
        }
        fake.touched.lock().unwrap().insert(0, vec![3, 2, 1, 0]);
        settle(&mut v, |v| {
            v.pins.len() == 3 && v.offer(0).is_some_and(|r| r.id == 3)
        });
        let (x, y, w, h) = fx_ui::pin_screen(0, 3);
        v.press(x + w - 10.0, y + h / 2.0);
        assert!(
            v.list_open(),
            "the last free field keeps its way to the list"
        );
        v.close_list();
        v.press(x + 10.0, y + h / 2.0);
        settle(&mut v, |v| v.pins.len() == 4);
        assert_eq!(v.pins.last().unwrap().id, 3);
        assert!(v.scene().offers[0].is_none());
        fake.slots.lock().unwrap().clear();
        fake.pins.lock().unwrap().clear();
        settle(&mut v, |v| v.scene().occupied == 0);
        assert_eq!(v.scene().depth(), fx_ui::DEPTH);
    }
    #[test]
    fn offers_follow_effect_identity_even_when_their_local_ids_match() {
        let fake = Fake::with(&["Chorus", "Room"], &[]);
        fake.parameters(0, &["Mix", "Rate"]);
        fake.parameters(1, &["Mix", "Decay"]);
        for row in fake.parameter_rows.lock().unwrap().get_mut(&1).unwrap() {
            row.id -= 100;
        }
        fake.touched
            .lock()
            .unwrap()
            .extend([(0, vec![1]), (1, vec![1])]);
        let mut v = FxView::new(Some(fake.clone()));
        settle(&mut v, |v| v.offer(0).is_some() && v.offer(1).is_some());
        assert_eq!(v.scene().offers[0].as_deref(), Some("+ RATE"));
        assert_eq!(v.scene().offers[1].as_deref(), Some("+ DECAY"));
        let stale = v.recent_request[0];
        let old_rows = v.recent[0].clone();
        fake.move_to(0, 1).unwrap();
        v.polled = None;
        v.poll(Instant::now());
        assert_ne!(v.recent_request[0], stale);
        lock(&v.pending).recent.push((0, stale, Ok(old_rows)));
        v.polled = None;
        v.poll(Instant::now());
        assert!(
            v.offer(0).is_none_or(|r| r.name == "Decay"),
            "a stale reply cannot show the previous effect's offer"
        );
        settle(&mut v, |v| {
            v.offer(0).is_some_and(|r| r.name == "Decay")
                && v.offer(1).is_some_and(|r| r.name == "Rate")
        });
        assert_eq!(v.scene().offers[0].as_deref(), Some("+ DECAY"));
        let (x, y) = fx_ui::centre(FxTarget::Field(0, 0));
        v.press(x, y);
        settle(&mut v, |v| !v.pins.is_empty());
        assert_eq!((v.pins[0].slot, v.pins[0].name.as_str()), (0, "Decay"));
    }

    #[test]
    fn a_failed_recent_catalogue_is_retried_with_the_same_touch() {
        let fake = Fake::with(&["Chorus"], &[]);
        fake.parameters(0, &["Mix", "Rate"]);
        fake.touched.lock().unwrap().insert(0, vec![1]);
        fake.params_fail.store(true, Ordering::Relaxed);
        let mut v = FxView::new(Some(fake.clone()));
        settle(&mut v, |v| v.recent_retry[0].is_some());
        assert!(v.offer(0).is_none());
        let reads = fake.param_reads.load(Ordering::Relaxed);
        fake.params_fail.store(false, Ordering::Relaxed);
        v.polled = None;
        v.poll(Instant::now());
        assert_eq!(
            fake.param_reads.load(Ordering::Relaxed),
            reads,
            "a failure cannot spin requests"
        );
        settle(&mut v, |v| v.offer(0).is_some_and(|r| r.name == "Rate"));
        assert_eq!(v.scene().offers[0].as_deref(), Some("+ RATE"));
    }
}
