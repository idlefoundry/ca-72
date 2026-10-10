//! The presets in the editor (decisions.md R10): the bar under the strip and the drawer over
//! the panel (`ca72_panel::presets` draws them), working on the library shared with the
//! DAW the model was developed in (`crate::library`).
//!
//! Choosing a preset sets every parameter but the PITCH wheel and the bypass to its value or
//! its default (POLY, VOICES, ENTROPY and SPREAD among them: R12), each a gesture of its own;
//! MIDI BEND RANGE, the player's keyboard's, only when the preset names it (R18). The plug-in
//! remembers it (`preset`, saved with the session); the bar marks it • once a value leaves
//! it. The drawer (the mock-up's, A6: decisions.md R-LOOK): typing goes to the field with the
//! caret (the search when none); the arrow keys step through the list, setting each preset;
//! Enter commits a field (the search: closes) or, once DELETE has asked, deletes; Tab moves to
//! the next field shown; Escape takes back what a key began, else closes; a double click on a
//! row chooses it and closes the drawer. The PRESET keys RENAME, TAGS, DELETE and REVERT act
//! on the preset the plug-in is set to; what they begin stays with that preset as the list
//! changes (a filter, another program's change) and ends when it is gone or another is set,
//! so it never acts on another (R18). SAVE AS begins a save (the bar's SAVE as well), and saves
//! when pressed again. The library is read again when the drawer opens, after each change and
//! every two seconds while it is open, so that DAW's changes show too (its files parsed only
//! when the folder changed); typing in the search and the filters work on the presets as read
//! (R18).

use std::time::{Duration, Instant};

use ca72_panel::presets::{
    BarScene, BarTarget, Current, DrawerScene, DrawerTarget, Edit, Field, FieldId, PresetKey,
    ROWS_SHOWN, Row, TAG_LINES, Tab, caret_at,
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
        ("filter_mode", &p.filter_mode),
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
        ("entropy", &p.entropy),
        ("spread", &p.spread),
        ("placement", &p.placement),
        ("unison", &p.unison),
        ("double", &p.double),
        ("drive", &p.drive),
        ("level", &p.level),
        ("auto_gain", &p.auto_gain),
        ("doubled", &p.doubled),
        ("feedback", &p.feedback),
        ("lock", &p.lock),
    ]
}

/// The player's parameters, not the sound's: MIDI BEND RANGE suits the keyboard played. A
/// preset saved does not name it, and choosing one that does not leaves it as it is (one that
/// does, another program's, sets it) (decisions.md R18).
pub const PLAYERS: &[&str] = &["midi_bend_range"];

/// What a preset sets each parameter to (normalized): its value, else the parameter's
/// default (but [`PLAYERS`]'s). POLY, VOICES, ENTROPY and SPREAD are part of a preset like
/// any other (decisions.md R12; R10 had left POLY and VOICES as they were when a preset did not
/// name them).
fn targets(p: &Ca72Params, e: &Entry) -> Vec<(&'static str, f32)> {
    let value = |id: &str| {
        e.sound
            .values
            .iter()
            .find(|(k, _)| k == id)
            .map(|(_, v)| *v)
    };
    sound_params(p)
        .into_iter()
        .filter_map(|(id, q)| match value(id) {
            Some(v) => Some((id, q.normalized_of(v))),
            None if PLAYERS.contains(&id) => None,
            // (A preset saved before DOUBLE was a switch of its own: DOUBLE where its DETUNE
            // is above 0.)
            None if id == "doubled" => Some((
                id,
                q.normalized_of(f64::from(u8::from(
                    value("double").is_some_and(|d| d > 0.0),
                ))),
            )),
            None => Some((id, q.default_normalized())),
        })
        .collect()
}

/// The current sound as a preset's values (plain, four decimals): not [`PLAYERS`]'s.
pub fn values_now(p: &Ca72Params) -> Vec<(String, f64)> {
    sound_params(p)
        .into_iter()
        .filter(|(id, _)| !PLAYERS.contains(id))
        .map(|(id, q)| (id.to_owned(), (q.plain() * 1e4).round() / 1e4))
        .collect()
}

/// Where a preset is from, as a row says it: `EDITED` (the user's version of a factory
/// preset) or `YOURS`; nothing for the factory's.
fn origin_word(o: Origin) -> Option<&'static str> {
    match o {
        Origin::Factory => None,
        Origin::Edited => Some("EDITED"),
        Origin::User => Some("YOURS"),
    }
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
    /// The preset being renamed, tagged or asked about deleting, by its name and origin: the
    /// edit stays with it as the list changes, and ends when it is gone or another preset is
    /// set, so it never acts on another (decisions.md R18).
    editing: Option<(String, Origin)>,
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
            editing: None,
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

    /// The drawer's slide as if begun at `t0` (tests: a moment of it, whatever the load).
    #[cfg(test)]
    pub(crate) fn slide_began(&mut self, t0: Instant) {
        if let Some((_, opening)) = self.slide {
            self.slide = Some((t0, opening));
        }
    }

    /// The library read again (once; unparsed when its folder has not changed), then the
    /// list filtered.
    pub fn reread(&mut self, p: &Ca72Params) {
        self.read = Some(Instant::now());
        self.all = self.library.entries().0;
        self.refilter(p);
    }

    /// The list as the filters have it and the tags, from the presets as read.
    fn refilter(&mut self, p: &Ca72Params) {
        let picked: Vec<String> = self
            .drawer
            .chips
            .iter()
            .filter(|(_, on)| *on)
            .map(|(t, _)| t.clone())
            .collect();
        self.list = library::matching(
            &self.all,
            &self.drawer.search.text,
            &picked,
            self.drawer.favourites,
            self.drawer.mine,
        );
        let mut chips: Vec<(String, bool)> = library::tags_of(&self.all)
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
        let most = self
            .drawer
            .chips
            .len()
            .div_ceil(2)
            .saturating_sub(TAG_LINES);
        self.drawer.tags_first = self.drawer.tags_first.min(most);
        // What a key began on a preset ends when the preset has gone.
        if self.editing.is_some() && self.edited_entry().is_none() {
            self.end_edit();
        }
        self.rows(p);
    }

    /// The preset the plug-in is set to edited (`how`): renamed, tagged or asked about
    /// deleting.
    fn begin_edit(&mut self, e: &Entry, how: Edit) {
        self.drawer.editing = Some(how);
        self.drawer.hint.clear();
        self.editing = Some((e.sound.name.clone(), e.origin));
    }

    /// What a key began ended (done, taken back, or its preset gone); a field it had, the
    /// search's.
    fn end_edit(&mut self) {
        if matches!(
            self.drawer.focus,
            Some(FieldId::Edit | FieldId::SaveName | FieldId::SaveTags)
        ) {
            self.drawer.focus = Some(FieldId::Search);
        }
        self.drawer.editing = None;
        self.editing = None;
    }

    /// The preset being edited, among the presets as read.
    fn edited_entry(&self) -> Option<&Entry> {
        let (name, origin) = self.editing.as_ref()?;
        self.all
            .iter()
            .find(|e| same(&e.sound.name, name) && e.origin == *origin)
    }

    /// The preset being edited, the library read again first: none when it has gone (said).
    fn edited_now(&mut self, p: &Ca72Params) -> Option<Entry> {
        let who = self.editing.clone();
        self.reread(p);
        let e = self.edited_entry().cloned();
        if e.is_none()
            && let Some((name, _)) = who
        {
            self.drawer.hint = format!("{name} is no longer there.");
        }
        e
    }

    fn rows(&mut self, p: &Ca72Params) {
        let current = Self::current(p);
        self.drawer.rows = self
            .list
            .iter()
            .map(|e| Row {
                name: e.sound.name.clone(),
                tags: e.sound.tags.join(" "),
                origin: origin_word(e.origin),
                favorite: e.favorite,
                current: same(&e.sound.name, &current),
            })
            .collect();
        self.drawer.total = self.all.len();
        self.drawer.current = self
            .all
            .iter()
            .find(|e| same(&e.sound.name, &current))
            .map(|e| Current {
                name: e.sound.name.clone(),
                origin: origin_word(e.origin),
                tags: e.sound.tags.join(" "),
                favorite: e.favorite,
            });
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
        // Another preset set (the host, the bar): what a key began on the last one ends, and
        // the drawer shows the one set.
        if self
            .editing
            .as_ref()
            .is_some_and(|(name, _)| !same(name, &current))
        {
            self.end_edit();
        }
        if self.open
            && self.drawer.current.as_ref().map(|c| c.name.as_str()) != Some(current.as_str())
            && self.all.iter().any(|e| same(&e.sound.name, &current))
        {
            self.rows(p);
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
            self.end_edit();
            self.drawer.focus = None;
            self.drawer.hover = None;
        }
    }

    /// The plug-in set to a preset.
    pub fn choose(&mut self, e: &Entry, p: &Ca72Params, s: &ParamSetter<'_>) {
        // (What a key began on the last preset, or a save of its sound, ends.)
        self.end_edit();
        self.drawer.hint.clear();
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
                self.begin_save(p);
            }
        }
    }

    /// A save begun: the PRESET display asks for a name and tags, the preset's to begin with.
    fn begin_save(&mut self, p: &Ca72Params) {
        self.end_edit();
        let current = Self::current(p);
        let e = self.all.iter().find(|e| same(&e.sound.name, &current));
        self.drawer.save_name = Field::new(e.map_or("", |e| e.sound.name.as_str()));
        self.drawer.save_tags = Field::new(&e.map_or(String::new(), |e| e.sound.tags.join(", ")));
        self.drawer.editing = Some(Edit::Save);
        self.drawer.focus = Some(FieldId::SaveName);
        self.wants_keys = true;
        self.saving_says();
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
                "Takes the factory's place (REVERT brings it back).".into()
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
                self.end_edit();
                let _ = s;
                self.reread(p);
                self.bring_into_view(&e.sound.name);
                return;
            }
            Err(e) => self.drawer.hint = e,
        }
        self.reread(p);
    }

    /// Commits the preset being renamed or tagged, wherever its row is now; none if it has
    /// gone.
    fn commit_edit(&mut self, p: &Ca72Params) {
        let Some(how @ (Edit::Rename | Edit::Tags)) = self.drawer.editing else {
            return;
        };
        let Some(e) = self.edited_now(p) else {
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
            _ => library::parse_tags(&text).and_then(|t| self.library.tag(&e.sound.name, t)),
        };
        match r {
            Ok(()) => {
                self.end_edit();
                self.drawer.focus = Some(FieldId::Search);
            }
            Err(err) => self.drawer.hint = err,
        }
        self.reread(p);
        self.bring_into_view(&shown);
    }

    /// A press in the drawer, at `x` across it (panel units: a field's caret goes there).
    pub fn drawer_press(&mut self, t: DrawerTarget, x: f64, p: &Ca72Params, s: &ParamSetter<'_>) {
        match t {
            DrawerTarget::Field(id) => {
                let caret = caret_at(&self.drawer, id, x);
                self.field_mut(id).caret = caret;
                self.drawer.focus = Some(id);
                self.wants_keys = true;
            }
            DrawerTarget::Tab(t) => {
                (self.drawer.favourites, self.drawer.mine) = match t {
                    Tab::All => (false, false),
                    Tab::Favourites => (true, false),
                    Tab::Mine => (false, true),
                };
                self.drawer.first = 0;
                self.refilter(p);
            }
            DrawerTarget::Chip(i) => {
                if let Some(c) = self.drawer.chips.get_mut(i) {
                    c.1 = !c.1;
                }
                self.drawer.first = 0;
                self.refilter(p);
            }
            DrawerTarget::Row(i) => {
                if let Some(e) = self.list.get(i).cloned() {
                    self.end_edit();
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
            DrawerTarget::Key(k) => self.key_press(k, p, s),
            // The update check is the editor's (`crate::update`), and so is the MIDI list
            // (`crate::learning`).
            DrawerTarget::Update
            | DrawerTarget::MidiRow(_)
            | DrawerTarget::MidiAction(..)
            | DrawerTarget::Back => {}
        }
    }

    /// A PRESET key: RENAME, TAGS, DELETE and REVERT for the preset the plug-in is set to
    /// (DELETE again, once it has asked, deletes it); SAVE AS (again, once begun, saves);
    /// RESTORE; CLOSE. (MIDI LEARN is the editor's.)
    fn key_press(&mut self, k: PresetKey, p: &Ca72Params, s: &ParamSetter<'_>) {
        let current = Self::current(p);
        let entry = self
            .all
            .iter()
            .find(|e| same(&e.sound.name, &current))
            .cloned();
        match (k, entry) {
            (PresetKey::Rename | PresetKey::Tags | PresetKey::Delete | PresetKey::Revert, None) => {
                self.end_edit();
                self.drawer.hint = "Choose a preset first.".into();
            }
            (PresetKey::Rename, Some(e)) => {
                self.begin_edit(&e, Edit::Rename);
                self.drawer.edit = Field::new(&e.sound.name);
                self.drawer.focus = Some(FieldId::Edit);
                self.wants_keys = true;
            }
            (PresetKey::Tags, Some(e)) => {
                self.begin_edit(&e, Edit::Tags);
                self.drawer.edit = Field::new(&e.sound.tags.join(", "));
                self.drawer.focus = Some(FieldId::Edit);
                self.wants_keys = true;
            }
            (PresetKey::Delete, Some(e)) => {
                if matches!(self.drawer.editing, Some(Edit::Delete { .. })) {
                    self.confirm_delete(p);
                } else {
                    let factory = e.origin != Origin::User;
                    self.begin_edit(&e, Edit::Delete { factory });
                    self.drawer.focus = Some(FieldId::Search);
                }
            }
            (PresetKey::Revert, Some(e)) => {
                self.end_edit();
                if e.origin == Origin::Edited {
                    self.drawer.hint = match self.library.revert(&e.sound.name) {
                        Ok(_) => format!("{} is the factory's again.", e.sound.name),
                        Err(err) => err,
                    };
                } else {
                    self.drawer.hint = "Only a factory preset you changed can be reverted.".into();
                }
                self.reread(p);
            }
            (PresetKey::SaveAs, _) => {
                if self.drawer.editing == Some(Edit::Save) {
                    self.save(p, s);
                } else {
                    self.begin_save(p);
                }
            }
            (PresetKey::Restore, _) => {
                self.end_edit();
                match self.library.restore() {
                    Ok(r) if r.is_empty() => {
                        self.drawer.hint = "No factory preset was deleted.".into()
                    }
                    Ok(r) => self.drawer.hint = format!("Restored {}.", r.join(", ")),
                    Err(e) => self.drawer.hint = e,
                }
                self.reread(p);
            }
            (PresetKey::Close, _) => self.show(false, p),
            (PresetKey::MidiLearn, _) => {}
        }
    }

    /// The preset DELETE asked about deleted, wherever it is now; none if it has gone.
    fn confirm_delete(&mut self, p: &Ca72Params) {
        if matches!(self.drawer.editing, Some(Edit::Delete { .. }))
            && let Some(e) = self.edited_now(p)
        {
            self.drawer.hint = match self.library.remove(&e.sound.name) {
                Ok(_) => format!("Deleted {}.", e.sound.name),
                Err(err) => err,
            };
        }
        self.end_edit();
        self.reread(p);
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

    /// The tags scrolled by `lines` (two tags a line).
    pub fn scroll_tags(&mut self, lines: i32) {
        let most = self
            .drawer
            .chips
            .len()
            .div_ceil(2)
            .saturating_sub(TAG_LINES) as i32;
        self.drawer.tags_first = (self.drawer.tags_first as i32 + lines).clamp(0, most) as usize;
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
                    self.end_edit();
                    self.drawer.focus = Some(FieldId::Search);
                } else {
                    self.show(false, p);
                }
            }
            (Key::Enter, _) if matches!(self.drawer.editing, Some(Edit::Delete { .. })) => {
                self.confirm_delete(p);
            }
            (Key::ArrowDown | Key::ArrowUp, None | Some(FieldId::Search)) => {
                self.step(if e.key == Key::ArrowDown { 1 } else { -1 }, p, s);
            }
            (Key::Enter, None | Some(FieldId::Search)) => self.show(false, p),
            (Key::Enter, Some(FieldId::Edit)) => self.commit_edit(p),
            (Key::Enter, Some(FieldId::SaveName | FieldId::SaveTags)) => self.save(p, s),
            // The next field shown: the search, then what a key began's.
            (Key::Tab, _) => {
                self.drawer.focus = Some(match (self.drawer.editing, focus) {
                    (Some(Edit::Save), None | Some(FieldId::Search)) => FieldId::SaveName,
                    (Some(Edit::Save), Some(FieldId::SaveName)) => FieldId::SaveTags,
                    (Some(Edit::Rename | Edit::Tags), None | Some(FieldId::Search)) => {
                        FieldId::Edit
                    }
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
                        self.refilter(p);
                    }
                    FieldId::SaveName => self.saving_says(),
                    _ => {}
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host that takes every change and keeps none.
    struct Host;

    #[allow(unsafe_code)]
    impl GuiContext for Host {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Clap
        }
        fn request_resize(&self) -> bool {
            true
        }
        unsafe fn raw_begin_set_parameter(&self, _: ParamPtr) {}
        unsafe fn raw_set_parameter_normalized(&self, _: ParamPtr, _: f32) {}
        unsafe fn raw_end_set_parameter(&self, _: ParamPtr) {}
        fn get_state(&self) -> PluginState {
            panic!("the browser does not read the state")
        }
        fn set_state(&self, _: PluginState) {}
    }

    fn down(key: Key) -> KeyboardEvent {
        KeyboardEvent {
            state: KeyState::Down,
            key,
            ..KeyboardEvent::default()
        }
    }

    fn typed(b: &mut Browser, text: &str, p: &Ca72Params) {
        let s = ParamSetter::new(&Host);
        for c in text.chars() {
            b.key(&down(Key::Character(c.to_string())), p, &s);
        }
    }

    fn rows(b: &Browser) -> Vec<&str> {
        b.drawer.rows.iter().map(|r| r.name.as_str()).collect()
    }

    /// MIDI BEND RANGE is the player's: no factory preset names it, a preset saved does not,
    /// and choosing one that does not leaves it as it is; one that names it sets it
    /// (decisions.md R18).
    #[test]
    fn the_bend_range_stays_the_players() {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        let p = Ca72Params::default();
        let bend = |e: &Entry| {
            targets(&p, e)
                .into_iter()
                .find(|(k, _)| *k == "midi_bend_range")
                .map(|(_, v)| v)
        };
        for e in lib.entries().0 {
            assert_eq!(bend(&e), None, "{}", e.sound.name);
            assert_eq!(targets(&p, &e).len(), sound_params(&p).len() - 1);
        }
        assert!(values_now(&p).iter().all(|(k, _)| k != "midi_bend_range"));
        let named = lib
            .save("Wide Bend", vec![("midi_bend_range".into(), 12.0)], None)
            .unwrap();
        assert_eq!(
            bend(&named),
            Some(p.midi_bend_range.preview_normalized(12.0))
        );
    }

    /// The MIDI assignments (decisions.md R34) are the instance's, not a sound's: choosing,
    /// saving, replacing and reverting a preset leave them as they are, and no preset file holds
    /// them.
    #[test]
    fn presets_leave_the_midi_assignments_alone() {
        use crate::learn::{Cc, index};
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        let p = Ca72Params::default();
        let map = &p.midi_map;
        map.assign(index("cutoff").unwrap(), Cc { channel: 0, cc: 74 });
        map.assign(index("voices").unwrap(), Cc { channel: 9, cc: 20 });
        let before = map.saved();
        let s = ParamSetter::new(&Host);
        let mut b = Browser::new(Library::at(dir.path()));
        b.show(true, &p);
        let bass = lib.find("Bass").unwrap();
        b.choose(&bass, &p, &s);
        b.step(1, &p, &s);
        assert_eq!(map.saved(), before, "chosen");
        // SAVE AS begins a save; pressed again, it saves.
        let save_as = |b: &mut Browser, name: &str| {
            b.drawer_press(DrawerTarget::Key(PresetKey::SaveAs), 0.0, &p, &s);
            b.drawer.save_name = Field::new(name);
            b.drawer_press(DrawerTarget::Key(PresetKey::SaveAs), 0.0, &p, &s);
        };
        save_as(&mut b, "Mine");
        save_as(&mut b, "Lead");
        assert_eq!(lib.find("Lead").unwrap().origin, Origin::Edited);
        assert_eq!(map.saved(), before, "saved");
        for f in std::fs::read_dir(dir.path()).unwrap() {
            let text = std::fs::read_to_string(f.unwrap().path()).unwrap();
            assert!(
                !text.contains("midi_map") && !text.contains("assignments"),
                "{text}"
            );
        }
        // REVERT, of the preset set (the one just saved).
        b.drawer_press(DrawerTarget::Key(PresetKey::Revert), 0.0, &p, &s);
        assert_eq!(lib.find("Lead").unwrap().origin, Origin::Factory);
        assert_eq!(map.saved(), before, "reverted");
        assert!(values_now(&p).iter().all(|(k, _)| !k.contains("midi_map")));
    }

    /// RENAME, TAGS and DELETE act on the preset the plug-in is set to; what they begin stays
    /// with it as the list changes (a filter, another program's change), and ends when it has
    /// gone or another preset is set: another preset is never renamed, tagged or deleted in its
    /// place (decisions.md R18).
    #[test]
    fn an_edit_stays_with_its_preset() {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        for n in ["Pad A", "Pad B"] {
            lib.save(n, vec![], None).unwrap();
        }
        lib.favorite("Pad A", true).unwrap();
        lib.favorite("Warped Pad", true).unwrap();
        let p = Ca72Params::default();
        let s = ParamSetter::new(&Host);
        let mut b = Browser::new(Library::at(dir.path()));
        let key = |b: &mut Browser, k: PresetKey| {
            b.drawer_press(DrawerTarget::Key(k), 0.0, &p, &s);
        };
        // (A row set, as a single click: not taken with the last for a double click.)
        let set = |b: &mut Browser, name: &str| {
            let i = rows(b).iter().position(|r| *r == name).expect("listed");
            b.last_row = None;
            b.drawer_press(DrawerTarget::Row(i), 0.0, &p, &s);
        };
        b.show(true, &p);
        // With no preset set, the keys that act on one say so.
        key(&mut b, PresetKey::Rename);
        assert_eq!(b.drawer.editing, None);
        assert_eq!(b.drawer.hint, "Choose a preset first.");
        typed(&mut b, "pad", &p);
        assert_eq!(rows(&b), ["Pad A", "Pad B", "Warped Pad"]);

        // Renaming Pad B, FAVORITES leaves it out of the list: the rename stays with it.
        set(&mut b, "Pad B");
        key(&mut b, PresetKey::Rename);
        assert_eq!(b.drawer.editing, Some(Edit::Rename));
        assert_eq!(b.drawer.focus, Some(FieldId::Edit));
        b.drawer.edit = Field::new("Pad Renamed");
        b.drawer_press(DrawerTarget::Tab(Tab::Favourites), 0.0, &p, &s);
        assert_eq!(rows(&b), ["Pad A", "Warped Pad"]);
        assert_eq!(b.drawer.editing, Some(Edit::Rename));
        b.key(&down(Key::Enter), &p, &s);
        assert!(lib.find("Pad B").is_none() && lib.find("Pad Renamed").is_some());
        assert_eq!(Browser::current(&p), "Pad Renamed");
        assert_eq!(b.drawer.editing, None);
        b.drawer_press(DrawerTarget::Tab(Tab::All), 0.0, &p, &s);

        // Tagging it, a preset arriving from elsewhere: the tags go to it.
        key(&mut b, PresetKey::Tags);
        b.drawer.edit = Field::new("soft");
        lib.save("Pad 0", vec![], None).unwrap();
        b.reread(&p);
        assert_eq!(rows(&b), ["Pad 0", "Pad A", "Pad Renamed", "Warped Pad"]);
        b.key(&down(Key::Enter), &p, &s);
        assert_eq!(lib.find("Pad Renamed").unwrap().sound.tags, ["soft"]);
        assert!(lib.find("Pad A").unwrap().sound.tags.is_empty());

        // DELETE asked about it, another preset set: the question ends, and Enter deletes
        // nothing; asked of Pad A, DELETE again deletes Pad A.
        key(&mut b, PresetKey::Delete);
        assert_eq!(b.drawer.editing, Some(Edit::Delete { factory: false }));
        set(&mut b, "Pad A");
        assert_eq!(b.drawer.editing, None);
        b.key(&down(Key::Enter), &p, &s);
        assert!(lib.find("Pad Renamed").is_some());
        b.show(true, &p);
        key(&mut b, PresetKey::Delete);
        key(&mut b, PresetKey::Delete);
        assert!(lib.find("Pad A").is_none());
        assert_eq!(b.drawer.hint, "Deleted Pad A.");

        // RENAME of a preset deleted elsewhere since the list was read: said, nothing done.
        set(&mut b, "Pad Renamed");
        key(&mut b, PresetKey::Rename);
        b.drawer.edit = Field::new("Pad Again");
        lib.remove("Pad Renamed").unwrap();
        b.key(&down(Key::Enter), &p, &s);
        assert_eq!(b.drawer.hint, "Pad Renamed is no longer there.");
        assert_eq!(b.drawer.editing, None);
        assert!(lib.find("Pad Again").is_none() && lib.find("Warped Pad").is_some());

        // REVERT of a preset that is not a changed factory one: said, nothing done.
        set(&mut b, "Warped Pad");
        key(&mut b, PresetKey::Revert);
        assert_eq!(
            b.drawer.hint,
            "Only a factory preset you changed can be reverted."
        );
        // Escape takes back what a key began; again, it closes the drawer.
        key(&mut b, PresetKey::Tags);
        b.key(&down(Key::Escape), &p, &s);
        assert_eq!((b.drawer.editing, b.open), (None, true));
        b.key(&down(Key::Escape), &p, &s);
        assert!(!b.open);
    }

    /// Typing in the search and the filters work on the presets as read: no file is read for
    /// them (decisions.md R18).
    #[test]
    fn the_search_works_on_the_presets_as_read() {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        lib.save("Zed Pad", vec![], Some(vec!["zed".into()]))
            .unwrap();
        let p = Ca72Params::default();
        let mut b = Browser::new(Library::at(dir.path()));
        b.show(true, &p);
        std::fs::remove_file(dir.path().join("Zed Pad.toml")).unwrap();
        typed(&mut b, "zed", &p);
        assert_eq!(rows(&b), ["Zed Pad"]);
        assert!(b.drawer.chips.iter().any(|(t, _)| t == "zed"));
        b.reread(&p);
        assert!(rows(&b).is_empty());
    }

    /// A preset sets POLY, VOICES, ENTROPY and SPREAD like any other parameter (decisions.md
    /// R12): the factory's ensemble presets turn POLY on with their voices, ENTROPY and
    /// SPREAD, the others turn it off; a preset that does not name them sets their
    /// defaults.
    #[test]
    fn poly_voices_entropy_and_spread_are_part_of_a_preset() {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        let p = Ca72Params::default();
        let target = |e: &Entry, id: &str| {
            targets(&p, e)
                .into_iter()
                .find(|(k, _)| *k == id)
                .map(|(_, v)| v)
        };
        let brass = lib.find("Brass Tutti").unwrap();
        assert_eq!(target(&brass, "poly"), Some(1.0));
        assert_eq!(
            target(&brass, "voices"),
            Some(p.voices.preview_normalized(8))
        );
        assert_eq!(
            target(&brass, "entropy"),
            Some(p.entropy.preview_normalized(40.0))
        );
        assert_eq!(
            target(&brass, "spread"),
            Some(p.spread.preview_normalized(50.0))
        );
        let bass = lib.find("Bass").unwrap();
        assert_eq!(target(&bass, "poly"), Some(0.0));
        assert_eq!(
            target(&bass, "entropy"),
            Some(p.entropy.preview_normalized(0.0))
        );
        // Every factory preset names all four.
        for e in lib.entries().0 {
            for id in ["poly", "voices", "entropy", "spread"] {
                assert!(
                    e.sound.values.iter().any(|(k, _)| k == id),
                    "{}: no {id}",
                    e.sound.name
                );
            }
        }
        // A preset naming none of them: their defaults (POLY off).
        let saved = lib
            .save("Plain", vec![("cutoff".to_owned(), 1.0)], None)
            .unwrap();
        assert_eq!(
            target(&saved, "poly"),
            Some(p.poly.default_normalized_value())
        );
        assert_eq!(
            target(&saved, "voices"),
            Some(p.voices.default_normalized_value())
        );
    }
}
