//! The presets in the editor (decisions.md R10): the bar under the strip and the drawer over
//! the panel (`ca72_panel::presets` draws them), working on the library DAW-v3's Minimoog
//! shares (`crate::library`).
//!
//! Choosing a preset sets every parameter but the PITCH wheel and the bypass to its value or
//! its default (POLY and VOICES only when it names them), each a gesture of its own, and the
//! plug-in remembers it (`preset`, saved with the session); the bar marks it • once a value
//! leaves it. The drawer: typing goes to the field with the caret (the search when none);
//! the arrow keys step through the list, setting each preset; Enter commits a field (the
//! search: closes); Escape takes back an edit, else closes; a double click on a row chooses
//! it and closes the drawer. The library is read again when
//! the drawer opens, after each change and every two seconds while it is open, so DAW-v3's
//! changes show too.

use std::time::{Duration, Instant};

use ca72_panel::presets::DrawerRenderer;
use ca72_panel::presets::{
    BarScene, BarTarget, DrawerScene, DrawerTarget, Edit, Field, FieldId, ROWS_SHOWN, Row,
    RowAction, caret_at,
};
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};
use nih_plug::prelude::*;

use crate::library::{self, Entry, Library, Origin};
use crate::params::Ca72Params;

/// How long the drawer takes to slide open or shut.
pub const SLIDE: Duration = Duration::from_millis(140);
/// How often the library is read again while the drawer is open.
const REREAD: Duration = Duration::from_secs(2);
/// Two clicks on a row within this long: chosen, and the drawer closed.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);

/// The parameters POLY and VOICES: set only by a preset that names them.
const FIELDS: &[&str] = &["poly", "voices"];

/// A parameter as a preset sets it: its plain value as the files give it (a choice by its
/// index, a switch 0 or 1), the normalized value of a plain one, and one change (a gesture).
pub trait SoundParam {
    fn plain(&self) -> f64;
    fn normalized(&self) -> f32;
    fn normalized_of(&self, plain: f64) -> f32;
    fn default_normalized(&self) -> f32;
    fn set_once(&self, s: &ParamSetter<'_>, normalized: f32);
}

impl SoundParam for FloatParam {
    fn plain(&self) -> f64 {
        f64::from(self.unmodulated_plain_value())
    }
    fn normalized(&self) -> f32 {
        self.unmodulated_normalized_value()
    }
    fn normalized_of(&self, plain: f64) -> f32 {
        self.preview_normalized(plain as f32)
    }
    fn default_normalized(&self) -> f32 {
        Param::default_normalized_value(self)
    }
    fn set_once(&self, s: &ParamSetter<'_>, normalized: f32) {
        s.begin_set_parameter(self);
        s.set_parameter_normalized(self, normalized);
        s.end_set_parameter(self);
    }
}

impl SoundParam for IntParam {
    fn plain(&self) -> f64 {
        f64::from(self.unmodulated_plain_value())
    }
    fn normalized(&self) -> f32 {
        self.unmodulated_normalized_value()
    }
    fn normalized_of(&self, plain: f64) -> f32 {
        self.preview_normalized(plain.round() as i32)
    }
    fn default_normalized(&self) -> f32 {
        Param::default_normalized_value(self)
    }
    fn set_once(&self, s: &ParamSetter<'_>, normalized: f32) {
        s.begin_set_parameter(self);
        s.set_parameter_normalized(self, normalized);
        s.end_set_parameter(self);
    }
}

impl SoundParam for BoolParam {
    fn plain(&self) -> f64 {
        f64::from(u8::from(self.unmodulated_plain_value()))
    }
    fn normalized(&self) -> f32 {
        self.unmodulated_normalized_value()
    }
    fn normalized_of(&self, plain: f64) -> f32 {
        self.preview_normalized(plain >= 0.5)
    }
    fn default_normalized(&self) -> f32 {
        Param::default_normalized_value(self)
    }
    fn set_once(&self, s: &ParamSetter<'_>, normalized: f32) {
        s.begin_set_parameter(self);
        s.set_parameter_normalized(self, normalized);
        s.end_set_parameter(self);
    }
}

impl<T: Enum + PartialEq + 'static> SoundParam for EnumParam<T> {
    fn plain(&self) -> f64 {
        self.unmodulated_plain_value().to_index() as f64
    }
    fn normalized(&self) -> f32 {
        self.unmodulated_normalized_value()
    }
    fn normalized_of(&self, plain: f64) -> f32 {
        let n = T::variants().len().max(1);
        self.preview_normalized(T::from_index((plain.round().max(0.0) as usize).min(n - 1)))
    }
    fn default_normalized(&self) -> f32 {
        Param::default_normalized_value(self)
    }
    fn set_once(&self, s: &ParamSetter<'_>, normalized: f32) {
        s.begin_set_parameter(self);
        s.set_parameter_normalized(self, normalized);
        s.end_set_parameter(self);
    }
}

/// Every parameter a preset sets, by its ID (all but the PITCH wheel and the bypass).
pub fn sound_params(p: &Ca72Params) -> Vec<(&'static str, &dyn SoundParam)> {
    vec![
        ("tune", &p.tune),
        ("glide", &p.glide),
        ("mod_mix", &p.mod_mix),
        ("osc_mod", &p.osc_mod),
        ("osc3_control", &p.osc3_control),
        ("osc1_range", &p.osc1_range),
        ("osc1_waveform", &p.osc1_waveform),
        ("osc2_range", &p.osc2_range),
        ("osc2_frequency", &p.osc2_frequency),
        ("osc2_waveform", &p.osc2_waveform),
        ("osc3_range", &p.osc3_range),
        ("osc3_frequency", &p.osc3_frequency),
        ("osc3_waveform", &p.osc3_waveform),
        ("osc1_on", &p.osc1_on),
        ("osc1_volume", &p.osc1_volume),
        ("ext_on", &p.ext_on),
        ("ext_volume", &p.ext_volume),
        ("osc2_on", &p.osc2_on),
        ("osc2_volume", &p.osc2_volume),
        ("noise_on", &p.noise_on),
        ("noise_volume", &p.noise_volume),
        ("noise_type", &p.noise_type),
        ("osc3_on", &p.osc3_on),
        ("osc3_volume", &p.osc3_volume),
        ("filter_mod", &p.filter_mod),
        ("keyboard_control_1", &p.keyboard_control_1),
        ("keyboard_control_2", &p.keyboard_control_2),
        ("cutoff", &p.cutoff),
        ("emphasis", &p.emphasis),
        ("contour_amount", &p.contour_amount),
        ("filter_attack", &p.filter_attack),
        ("filter_decay", &p.filter_decay),
        ("filter_sustain", &p.filter_sustain),
        ("loudness_attack", &p.loudness_attack),
        ("loudness_decay", &p.loudness_decay),
        ("loudness_sustain", &p.loudness_sustain),
        ("volume", &p.volume),
        ("main_output", &p.main_output),
        ("a440", &p.a440),
        ("glide_on", &p.glide_on),
        ("decay_on", &p.decay_on),
        ("mod_wheel", &p.mod_wheel),
        ("midi_bend_range", &p.midi_bend_range),
        ("poly", &p.poly),
        ("voices", &p.voices),
        ("analog", &p.analog),
        ("spread", &p.spread),
        ("feedback", &p.feedback),
        ("lock", &p.lock),
    ]
}

/// What a preset sets each parameter to (normalized): its value, else the parameter's
/// default (POLY and VOICES only when it names them).
fn targets(p: &Ca72Params, e: &Entry) -> Vec<(&'static str, f32)> {
    sound_params(p)
        .into_iter()
        .filter_map(
            |(id, q)| match e.sound.values.iter().find(|(k, _)| k == id) {
                Some((_, v)) => Some((id, q.normalized_of(*v))),
                None if FIELDS.contains(&id) => None,
                None => Some((id, q.default_normalized())),
            },
        )
        .collect()
}

/// The current sound as a preset's values (plain, four decimals).
pub fn values_now(p: &Ca72Params) -> Vec<(String, f64)> {
    sound_params(p)
        .into_iter()
        .map(|(id, q)| (id.to_owned(), (q.plain() * 1e4).round() / 1e4))
        .collect()
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The presets' bar and drawer: what they show and what operating them does.
pub struct Browser {
    pub library: Library,
    pub open: bool,
    /// The drawer sliding: when it began, and whether opening.
    slide: Option<(Instant, bool)>,
    pub bar: BarScene,
    pub drawer: DrawerScene,
    /// The list as filtered, and every preset.
    list: Vec<Entry>,
    all: Vec<Entry>,
    /// The current preset's values as set (normalized), to mark it changed.
    wanted: Option<(String, Vec<(&'static str, f32)>)>,
    read: Option<Instant>,
    /// The window should take the keyboard (a field took the caret).
    pub wants_keys: bool,
    /// The drawer opens below the bar, the window growing for it; false where the host
    /// would not resize the window (it then opens over the panel).
    pub below: bool,
    /// The last row clicked, and when: a second click soon after closes the drawer.
    last_row: Option<(Instant, usize)>,
}

impl std::fmt::Debug for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Browser")
            .field("open", &self.open)
            .field("rows", &self.list.len())
            .finish()
    }
}

impl Browser {
    pub fn new(library: Library) -> Self {
        Browser {
            library,
            open: false,
            slide: None,
            bar: BarScene::default(),
            drawer: DrawerScene::default(),
            list: Vec::new(),
            all: Vec::new(),
            wanted: None,
            read: None,
            wants_keys: false,
            below: true,
            last_row: None,
        }
    }

    /// The preset the plug-in was last set to.
    fn current(p: &Ca72Params) -> String {
        p.preset.read().map(|s| s.clone()).unwrap_or_default()
    }

    fn set_current(p: &Ca72Params, name: &str) {
        if let Ok(mut s) = p.preset.write() {
            *s = name.to_owned();
        }
    }

    /// How far the drawer is open, 0 to 1.
    pub fn reveal(&self) -> f64 {
        match self.slide {
            Some((t0, opening)) => {
                let k = (t0.elapsed().as_secs_f64() / SLIDE.as_secs_f64()).min(1.0);
                // Eased out.
                let k = 1.0 - (1.0 - k) * (1.0 - k);
                if opening { k } else { 1.0 - k }
            }
            None => f64::from(u8::from(self.open)),
        }
    }

    /// Whether the drawer is moving.
    pub fn sliding(&self) -> bool {
        self.slide.is_some_and(|(t0, _)| t0.elapsed() < SLIDE)
    }

    /// The library read again: the list as the filters have it, every preset, the tags.
    pub fn reread(&mut self, p: &Ca72Params) {
        self.read = Some(Instant::now());
        let picked: Vec<String> = self
            .drawer
            .chips
            .iter()
            .filter(|(_, on)| *on)
            .map(|(t, _)| t.clone())
            .collect();
        self.all = self.library.entries().0;
        self.list = self.library.search(
            &self.drawer.search.text,
            &picked,
            self.drawer.favourites,
            self.drawer.mine,
        );
        let mut chips: Vec<(String, bool)> = self
            .library
            .tags()
            .into_iter()
            .map(|t| {
                let on = picked.iter().any(|x| same(x, &t));
                (t, on)
            })
            .collect();
        for t in picked {
            if !chips.iter().any(|(c, _)| same(c, &t)) {
                chips.push((t, true));
            }
        }
        self.drawer.chips = chips;
        self.rows(p);
    }

    fn rows(&mut self, p: &Ca72Params) {
        let current = Self::current(p);
        self.drawer.rows = self
            .list
            .iter()
            .map(|e| Row {
                name: e.sound.name.clone(),
                tags: e.sound.tags.join(" · "),
                origin: match e.origin {
                    Origin::Factory => None,
                    Origin::Edited => Some("EDITED"),
                    Origin::User => Some("YOURS"),
                },
                favorite: e.favorite,
                current: same(&e.sound.name, &current),
            })
            .collect();
        let most = self.drawer.rows.len().saturating_sub(ROWS_SHOWN);
        self.drawer.first = self.drawer.first.min(most);
    }

    /// Brings the bar and the drawer up to the parameters (each frame).
    pub fn tick(&mut self, p: &Ca72Params) {
        if self.open && self.read.is_none_or(|t| t.elapsed() > REREAD) {
            self.reread(p);
        }
        if self.slide.is_some_and(|(t0, _)| t0.elapsed() >= SLIDE) {
            self.slide = None;
        }
        let current = Self::current(p);
        if self.read.is_none() {
            self.all = self.library.entries().0;
            self.list = self.all.clone();
            self.read = Some(Instant::now());
        }
        let entry = self.all.iter().find(|e| same(&e.sound.name, &current));
        if self.wanted.as_ref().is_none_or(|(n, _)| !same(n, &current)) {
            self.wanted = entry.map(|e| (current.clone(), targets(p, e)));
        }
        let changed = self.wanted.as_ref().is_some_and(|(_, t)| {
            let now = sound_params(p);
            t.iter().any(|(id, v)| {
                now.iter()
                    .find(|(i, _)| i == id)
                    .is_some_and(|(_, q)| (q.normalized() - v).abs() > 1e-4)
            })
        });
        self.bar = BarScene {
            name: entry.map_or(current.clone(), |e| e.sound.name.clone()),
            found: entry.is_some(),
            changed: entry.is_some() && changed,
            favorite: entry.is_some_and(|e| e.favorite),
            open: self.open,
            below: self.below,
            hover: self.bar.hover,
        };
    }

    /// Opens or shuts the drawer.
    pub fn show(&mut self, on: bool, p: &Ca72Params) {
        if on == self.open {
            return;
        }
        self.open = on;
        self.slide = Some((Instant::now(), on));
        if on {
            self.reread(p);
            self.drawer.focus = Some(FieldId::Search);
            self.wants_keys = true;
        } else {
            self.drawer.focus = None;
            self.drawer.editing = None;
            self.drawer.hover = None;
        }
    }

    /// The plug-in set to a preset.
    pub fn choose(&mut self, e: &Entry, p: &Ca72Params, s: &ParamSetter<'_>) {
        let params = sound_params(p);
        for (id, v) in targets(p, e) {
            if let Some((_, q)) = params.iter().find(|(i, _)| *i == id)
                && (q.normalized() - v).abs() > 1e-6
            {
                q.set_once(s, v);
            }
        }
        Self::set_current(p, &e.sound.name);
        self.wanted = Some((e.sound.name.clone(), targets(p, e)));
        self.rows(p);
    }

    /// The next (`by` 1) or previous (-1) preset of the list, set.
    pub fn step(&mut self, by: i32, p: &Ca72Params, s: &ParamSetter<'_>) {
        if self.list.is_empty() {
            return;
        }
        let current = Self::current(p);
        let n = self.list.len() as i32;
        let at = self.list.iter().position(|e| same(&e.sound.name, &current));
        let i = match at {
            Some(i) => (i as i32 + by).rem_euclid(n),
            None if by > 0 => 0,
            None => n - 1,
        } as usize;
        let e = self.list[i].clone();
        self.choose(&e, p, s);
        self.bring_into_view(&e.sound.name);
    }

    /// The list scrolled so the preset of this name shows.
    fn bring_into_view(&mut self, name: &str) {
        let Some(i) = self.list.iter().position(|e| same(&e.sound.name, name)) else {
            return;
        };
        if i < self.drawer.first {
            self.drawer.first = i;
        } else if i >= self.drawer.first + ROWS_SHOWN {
            self.drawer.first = i + 1 - ROWS_SHOWN;
        }
    }

    /// A press on the bar.
    pub fn bar_press(&mut self, t: BarTarget, p: &Ca72Params, s: &ParamSetter<'_>) {
        match t {
            BarTarget::Name => self.show(!self.open, p),
            BarTarget::Prev => self.step(-1, p, s),
            BarTarget::Next => self.step(1, p, s),
            BarTarget::Star => {
                let current = Self::current(p);
                if let Some(e) = self.all.iter().find(|e| same(&e.sound.name, &current)) {
                    let on = !e.favorite;
                    let name = e.sound.name.clone();
                    self.report(self.library.favorite(&name, on));
                    self.reread(p);
                }
            }
            BarTarget::Save => {
                self.show(true, p);
                let current = Self::current(p);
                let e = self.all.iter().find(|e| same(&e.sound.name, &current));
                self.drawer.save_name = Field::new(e.map_or("", |e| e.sound.name.as_str()));
                self.drawer.save_tags =
                    Field::new(&e.map_or(String::new(), |e| e.sound.tags.join(", ")));
                self.drawer.focus = Some(FieldId::SaveName);
                self.saving_says();
            }
        }
    }

    fn report(&mut self, r: Result<impl Sized, String>) {
        if let Err(e) = r {
            self.drawer.hint = e;
        }
    }

    /// What saving under the name typed does, said before.
    fn saving_says(&mut self) {
        let name = self.drawer.save_name.text.trim().to_owned();
        let there = (!name.is_empty())
            .then(|| self.all.iter().find(|e| same(&e.sound.name, &name)))
            .flatten();
        self.drawer.replace = there.is_some();
        self.drawer.hint = match there {
            None => String::new(),
            Some(e) if e.origin == Origin::Factory => {
                "Saved as yours in place of the factory's (REVERT brings it back).".into()
            }
            Some(e) => format!("Replaces your {}.", e.sound.name),
        };
    }

    /// The current sound saved under the name and tags typed; the plug-in then that preset.
    fn save(&mut self, p: &Ca72Params, s: &ParamSetter<'_>) {
        let name = self.drawer.save_name.text.trim().to_owned();
        if name.is_empty() {
            self.drawer.focus = Some(FieldId::SaveName);
            return;
        }
        let tags = match library::parse_tags(&self.drawer.save_tags.text) {
            Ok(t) => t,
            Err(e) => {
                self.drawer.hint = e;
                return;
            }
        };
        match self.library.save(&name, values_now(p), Some(tags)) {
            Ok(e) => {
                Self::set_current(p, &e.sound.name);
                self.wanted = Some((e.sound.name.clone(), targets(p, &e)));
                self.drawer.hint = format!("Saved {}.", e.sound.name);
                self.drawer.save_name = Field::default();
                self.drawer.save_tags = Field::default();
                self.drawer.replace = false;
                self.drawer.focus = None;
                let _ = s;
                self.reread(p);
                self.bring_into_view(&e.sound.name);
                return;
            }
            Err(e) => self.drawer.hint = e,
        }
        self.reread(p);
    }

    /// Commits the row being renamed or tagged.
    fn commit_edit(&mut self, p: &Ca72Params) {
        let Some((i, how)) = self.drawer.editing else {
            return;
        };
        let Some(e) = self.list.get(i).cloned() else {
            return;
        };
        let text = self.drawer.edit.text.clone();
        let mut shown = e.sound.name.clone();
        let r = match how {
            Edit::Rename => self.library.rename(&e.sound.name, &text).map(|n| {
                if same(&Self::current(p), &e.sound.name) {
                    Self::set_current(p, &n.sound.name);
                }
                shown = n.sound.name;
            }),
            Edit::Tags => {
                library::parse_tags(&text).and_then(|t| self.library.tag(&e.sound.name, t))
            }
            Edit::Delete { .. } => Ok(()),
        };
        match r {
            Ok(()) => {
                self.drawer.editing = None;
                self.drawer.focus = Some(FieldId::Search);
            }
            Err(err) => self.drawer.hint = err,
        }
        self.reread(p);
        self.bring_into_view(&shown);
    }

    /// A press in the drawer, at `x` across it (panel units: a field's caret goes there).
    pub fn drawer_press(
        &mut self,
        t: DrawerTarget,
        x: f64,
        fonts: &DrawerRenderer,
        p: &Ca72Params,
        s: &ParamSetter<'_>,
    ) {
        match t {
            DrawerTarget::Field(id) => {
                let f = self.field_mut(id);
                let f2 = f.clone();
                f.caret = caret_at(fonts.fonts(), &f2, id, x);
                self.drawer.focus = Some(id);
                self.wants_keys = true;
            }
            DrawerTarget::Favourites => {
                self.drawer.favourites = !self.drawer.favourites;
                self.reread(p);
            }
            DrawerTarget::Mine => {
                self.drawer.mine = !self.drawer.mine;
                self.reread(p);
            }
            DrawerTarget::Close => self.show(false, p),
            DrawerTarget::Chip(i) => {
                if let Some(c) = self.drawer.chips.get_mut(i) {
                    c.1 = !c.1;
                }
                self.drawer.first = 0;
                self.reread(p);
            }
            DrawerTarget::Row(i) => {
                if let Some(e) = self.list.get(i).cloned() {
                    self.drawer.editing = None;
                    self.choose(&e, p, s);
                }
                // A double click chooses it and closes the drawer.
                let now = Instant::now();
                let double = self
                    .last_row
                    .is_some_and(|(t, r)| r == i && now - t < DOUBLE_CLICK);
                self.last_row = Some((now, i));
                if double {
                    self.last_row = None;
                    self.show(false, p);
                }
            }
            DrawerTarget::Star(i) => {
                if let Some(e) = self.list.get(i).cloned() {
                    self.report(self.library.favorite(&e.sound.name, !e.favorite));
                    self.reread(p);
                }
            }
            DrawerTarget::Action(i, a) => {
                let Some(e) = self.list.get(i).cloned() else {
                    return;
                };
                match a {
                    RowAction::Rename => {
                        self.drawer.editing = Some((i, Edit::Rename));
                        self.drawer.edit = Field::new(&e.sound.name);
                        self.drawer.focus = Some(FieldId::Edit);
                        self.wants_keys = true;
                    }
                    RowAction::Tags => {
                        self.drawer.editing = Some((i, Edit::Tags));
                        self.drawer.edit = Field::new(&e.sound.tags.join(", "));
                        self.drawer.focus = Some(FieldId::Edit);
                        self.wants_keys = true;
                    }
                    RowAction::Revert => {
                        self.report(self.library.revert(&e.sound.name));
                        self.reread(p);
                    }
                    RowAction::Delete => {
                        self.drawer.editing = Some((
                            i,
                            Edit::Delete {
                                factory: e.origin != Origin::User,
                            },
                        ));
                    }
                    RowAction::Confirm => {
                        self.drawer.editing = None;
                        self.report(self.library.remove(&e.sound.name));
                        self.reread(p);
                    }
                    RowAction::Cancel => self.drawer.editing = None,
                }
            }
            DrawerTarget::SaveGo => self.save(p, s),
            DrawerTarget::Restore => {
                match self.library.restore() {
                    Ok(r) if r.is_empty() => {
                        self.drawer.hint = "No factory preset was deleted.".into()
                    }
                    Ok(r) => self.drawer.hint = format!("Restored {}.", r.join(", ")),
                    Err(e) => self.drawer.hint = e,
                }
                self.reread(p);
            }
            DrawerTarget::Back => {}
        }
    }

    fn field_mut(&mut self, id: FieldId) -> &mut Field {
        match id {
            FieldId::Search => &mut self.drawer.search,
            FieldId::Edit => &mut self.drawer.edit,
            FieldId::SaveName => &mut self.drawer.save_name,
            FieldId::SaveTags => &mut self.drawer.save_tags,
        }
    }

    /// The list scrolled by `rows`.
    pub fn scroll(&mut self, rows: i32) {
        let most = self.drawer.rows.len().saturating_sub(ROWS_SHOWN) as i32;
        self.drawer.first = (self.drawer.first as i32 + rows).clamp(0, most) as usize;
    }

    /// A key: whether the drawer took it (else it is the host's).
    pub fn key(&mut self, e: &KeyboardEvent, p: &Ca72Params, s: &ParamSetter<'_>) -> bool {
        if !self.open {
            return false;
        }
        if e.state != KeyState::Down {
            return true;
        }
        // The host's shortcuts (Command, Control) stay the host's.
        if e.modifiers.intersects(Modifiers::META | Modifiers::CONTROL) {
            return false;
        }
        let focus = self.drawer.focus;
        match (&e.key, focus) {
            (Key::Escape, _) => {
                if self.drawer.editing.is_some() {
                    self.drawer.editing = None;
                    self.drawer.focus = Some(FieldId::Search);
                } else {
                    self.show(false, p);
                }
            }
            (Key::ArrowDown | Key::ArrowUp, None | Some(FieldId::Search)) => {
                self.step(if e.key == Key::ArrowDown { 1 } else { -1 }, p, s);
            }
            (Key::Enter, None | Some(FieldId::Search)) => self.show(false, p),
            (Key::Enter, Some(FieldId::Edit)) => self.commit_edit(p),
            (Key::Enter, Some(FieldId::SaveName | FieldId::SaveTags)) => self.save(p, s),
            (Key::Tab, _) => {
                self.drawer.focus = Some(match focus {
                    Some(FieldId::Search) => FieldId::SaveName,
                    Some(FieldId::SaveName) => FieldId::SaveTags,
                    _ => FieldId::Search,
                });
            }
            (k, f) => {
                let id = f.unwrap_or(FieldId::Search);
                let field = self.field_mut(id);
                match k {
                    Key::Character(c) => field.insert(c),
                    Key::Backspace => field.backspace(),
                    Key::Delete => field.delete(),
                    Key::ArrowLeft => field.left(),
                    Key::ArrowRight => field.right(),
                    Key::Home => field.home(),
                    Key::End => field.end(),
                    _ => return false,
                }
                self.drawer.focus = Some(id);
                match id {
                    FieldId::Search => {
                        self.drawer.first = 0;
                        self.reread(p);
                    }
                    FieldId::SaveName => self.saving_says(),
                    _ => {}
                }
            }
        }
        true
    }
}
