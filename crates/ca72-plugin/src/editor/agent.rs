//! The editor's side of agents (`crate::agent`; decisions.md R-MCP): the requests an agent's
//! bridge sends, taken from the socket's inbox at each frame and carried out on the window's
//! thread through the editor's own paths. A parameter is set through the host's context, a
//! gesture of its own, as a click or a typed value sets it (DETUNE and VOICES refused where the
//! panel does not let them be turned); a preset chosen as the drawer chooses it, saved,
//! renamed, tagged, starred and deleted through the drawer's library, its hint said in the
//! drawer; a setting changed as the SETTINGS menu changes it; a note played as the on-screen
//! keyboard plays one (through the local notes' queue, the audio thread's only way in). Every
//! change is the editor's as far as the rest of the plug-in knows: the panel shows it at once,
//! the preset's mark follows, and AUTO GAIN's curve is asked for as after the player's edits.

use std::time::Instant;

use nih_plug::prelude::Param;

use serde_json::{Value, json};

use super::{Change, Editing, Law, Lettering, ParamSetter, Skin};
use crate::agent::server::Server;
use crate::agent::values::{self, Entry};
use crate::agent::{ApiError, Reply, arg, codes, flag_arg, text_arg};
use crate::learn::{self, Cc, LEARNABLE};
use crate::library::{self, Origin};
use crate::presets::Browser;
use ca72_panel::keyboard;
use ca72_panel::presets::BarTarget;
use ca72_panel::strip::StripTarget;

/// How many requests a frame carries out at most (the rest wait for the next).
const PER_FRAME: usize = 32;

/// The longest an agent's note may be played for (`notes.play`).
const LONGEST_NOTE_S: f64 = 30.0;

/// Agents, as the editor keeps them: the socket while they are allowed, why it could not be
/// opened, and the notes an agent plays for a while.
#[derive(Debug, Default)]
pub(super) struct Agents {
    pub server: Option<Server>,
    pub error: Option<String>,
    /// Each note an agent plays for a while, and when it is let go.
    pub timed: Vec<(u8, Instant)>,
    /// Agents turned off by an agent: the socket closed once its answer has gone.
    pub stop: bool,
}

impl Editing {
    /// Whether agents may operate this editor: the standalone's, allowed in the settings or
    /// for this session (`--agents`).
    pub(super) fn agents_allowed(&self) -> bool {
        self.playing.standalone && (self.settings.agents || crate::agent::forced())
    }

    /// The socket open while agents are allowed, closed when they are not (the tests' editors
    /// open none of their own in the user's folder: they are given one).
    pub(super) fn sync_agents(&mut self) {
        let allowed = self.agents_allowed() && !self.agents.stop;
        match (&self.agents.server, allowed) {
            (None, true) if self.agents.error.is_none() && !cfg!(test) => {
                match Server::start("standalone") {
                    Ok(s) => self.agents.server = Some(s),
                    Err(e) => self.agents.error = Some(e),
                }
            }
            (Some(_), false) => {
                self.agents.server = None;
                for (note, _) in std::mem::take(&mut self.agents.timed) {
                    self.playing.agent(note, false);
                }
            }
            _ => {}
        }
    }

    /// The agents' requests waiting, carried out (at most [`PER_FRAME`]), and their timed
    /// notes let go when due: whether anything was done.
    pub(super) fn serve_agents(&mut self) -> bool {
        let now = Instant::now();
        let mut done = false;
        if self.agents.timed.iter().any(|(_, at)| *at <= now) {
            let (due, kept): (Vec<_>, Vec<_>) =
                self.agents.timed.drain(..).partition(|(_, at)| *at <= now);
            self.agents.timed = kept;
            for (note, _) in due {
                self.playing.agent(note, false);
            }
            done = true;
        }
        let Some(inbox) = self
            .agents
            .server
            .as_ref()
            .map(|s| std::sync::Arc::clone(s.inbox()))
        else {
            return done;
        };
        for _ in 0..PER_FRAME {
            let Some(job) = inbox.take() else {
                break;
            };
            let r = self.agent_call(&job.method, &job.params);
            job.answer(r);
            done = true;
        }
        if self.agents.stop {
            self.sync_agents();
        }
        done
    }

    /// One request carried out: its answer, or why it was refused.
    pub(super) fn agent_call(&mut self, method: &str, args: &Value) -> Reply {
        let r = match method {
            "describe" => Ok(self.describe()),
            "parameters.get" => self.get(args),
            "parameters.set" => self.set(args),
            "parameters.reset" => self.reset(args),
            "presets.list" => self.list_presets(args),
            "presets.load" => self.load_preset(args),
            "presets.step" => self.step_preset(args),
            "presets.save" => self.save_preset(args),
            "presets.rename" => self.rename_preset(args),
            "presets.tags" => self.tag_preset(args),
            "presets.favorite" => self.favor_preset(args),
            "presets.delete" => self.delete_preset(args),
            "presets.revert" => self.revert_preset(args),
            "presets.restore" => self.restore_presets(),
            "settings.get" => Ok(self.settings_state()),
            "settings.set" => self.set_settings(args),
            "view.set" => self.set_view(args),
            "notes.on" => self.notes_on(args, None),
            "notes.off" => self.notes_off(args),
            "notes.play" => self.notes_play(args),
            "notes.panic" => {
                self.panic();
                self.agents.timed.clear();
                Ok(json!({"panic": true}))
            }
            "midi.learn" => self.midi_learn(args),
            "midi.list" => Ok(self.midi_list()),
            "midi.remove" => self.midi_remove(args),
            "audio.get" => Ok(audio_state()),
            "version" => self.version(args),
            other => Err(ApiError::new(
                codes::METHOD_NOT_FOUND,
                format!("unknown method `{other}`"),
            )),
        };
        // (Anything an agent may have changed: AUTO GAIN's curve asked for once nothing is
        // held, as after the player's edits; the frame drawn again.)
        if r.is_ok()
            && !matches!(
                method,
                "describe"
                    | "parameters.get"
                    | "settings.get"
                    | "midi.list"
                    | "audio.get"
                    | "presets.list"
            )
        {
            self.touched = true;
            self.composed = None;
        }
        r
    }

    fn preset_state(&mut self) -> Value {
        self.browser.tick(&self.params);
        let b = &self.browser.bar;
        json!({
            "name": (!b.name.is_empty()).then(|| b.name.clone()),
            "found": b.found,
            "changed": b.changed,
            "favorite": b.favorite,
        })
    }

    fn mode(&self) -> &'static str {
        let p = &self.params;
        if p.unison.value() {
            "UNISON"
        } else if p.poly.value() {
            "POLY"
        } else {
            "MONO"
        }
    }

    fn view_state(&self) -> Value {
        let root = self.playing.root;
        json!({
            "presets": self.browser.open,
            "panel": self.expanded(),
            "keyboard": self.playing.visible,
            "keyboard_range": [keyboard::label(root), keyboard::label(root.saturating_add(keyboard::KEYS as u8 - 1))],
            "midi_list": self.browser.open && self.learning.list,
            "typing": self.playing.typing,
            "width": self.width,
        })
    }

    fn describe(&mut self) -> Value {
        let preset = self.preset_state();
        let table = values::table(&self.params);
        let parameters: Vec<Value> = table
            .iter()
            .map(|e| {
                let mut d = e.describe();
                if let Err(why) = self.operable_now(e.id, None) {
                    d["inert"] = json!(why);
                }
                d
            })
            .collect();
        let held = self.params_notes_held();
        json!({
            "instrument": "CA-72",
            "version": env!("CARGO_PKG_VERSION"),
            "kind": if self.playing.standalone { "standalone" } else { "plugin" },
            "power": !self.params.bypass.value(),
            "mode": self.mode(),
            "parameters": parameters,
            "preset": preset,
            "view": self.view_state(),
            "settings": self.settings_state(),
            "midi_learn": {
                "learning": self.params.midi_map.armed().map(|i| LEARNABLE[i].id),
                "assigned": self.params.midi_map.assignments().iter().filter(|a| a.is_some()).count(),
            },
            "notes": {
                "yours": self.playing.agent_notes(),
                "sounding_keys": held,
                "voices_sounding": self.meters.voices.load(std::sync::atomic::Ordering::Relaxed).count_ones(),
            },
            "audio": audio_summary(),
        })
    }

    /// The keys held now (every source: MIDI devices, the on-screen keyboard, agents), as the
    /// audio thread last published them.
    fn params_notes_held(&self) -> Vec<u8> {
        let held = self.playing.notes.held();
        (0..128u8)
            .filter(|n| held[usize::from(*n / 64)] & (1 << (n % 64)) != 0)
            .collect()
    }

    fn entries_for(&self, args: &Value) -> Result<Vec<String>, ApiError> {
        let table = values::table(&self.params);
        match arg(args, "ids") {
            None => Ok(table.iter().map(|e| e.id.to_owned()).collect()),
            Some(Value::Array(ids)) => ids
                .iter()
                .map(|k| {
                    let k = k
                        .as_str()
                        .ok_or_else(|| ApiError::params("`ids` are strings"))?;
                    values::find(&table, k)
                        .map(|e| e.id.to_owned())
                        .ok_or_else(|| unknown(k, &table))
                })
                .collect(),
            Some(_) => Err(ApiError::params("`ids` must be a list")),
        }
    }

    fn get(&mut self, args: &Value) -> Reply {
        let ids = self.entries_for(args)?;
        let table = values::table(&self.params);
        let list: Vec<Value> = ids
            .iter()
            .filter_map(|id| values::find(&table, id))
            .map(Entry::describe)
            .collect();
        Ok(json!({"parameters": list}))
    }

    /// Whether the panel would let parameter `id` be turned now (DETUNE only in DOUBLE; VOICES
    /// only where it does something), the changes planned before it taken into account.
    fn operable_now(&self, id: &str, planned: Option<&[(String, f32)]>) -> Result<(), String> {
        let p = &self.params;
        let will = |key: &str, now: f32| {
            planned
                .and_then(|l| l.iter().rev().find(|(k, _)| k == key).map(|(_, v)| *v))
                .unwrap_or(now)
        };
        let on = |key: &str, now: bool| will(key, f32::from(u8::from(now))) >= 0.5;
        match id {
            "double" if !on("doubled", p.doubled.value()) => Err(
                "DETUNE does nothing while the voices play as SCATTER: set `doubled` true (VOICES PLAYED AS: DOUBLE) first, or in the same call".into(),
            ),
            "voices" => {
                let poly = on("poly", p.poly.value());
                let unison = on("unison", p.unison.value());
                let doubled = on("doubled", p.doubled.value());
                let spread = will("spread", p.spread.unmodulated_normalized_value());
                if !poly && !unison && (doubled || spread <= 0.0) {
                    Err("VOICES does nothing in MONO with DOUBLE or with WIDTH at 0: choose POLY or UNISON, or SCATTER with WIDTH above 0, first".into())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn set(&mut self, args: &Value) -> Reply {
        let Some(Value::Object(values)) = arg(args, "values") else {
            return Err(ApiError::params(
                "`values` must be an object of parameter ids and values",
            ));
        };
        if values.is_empty() {
            return Err(ApiError::params("`values` names no parameter"));
        }
        let table = values::table(&self.params);
        // Every value checked before any is set; MODE as the tabs set POLY and UNISON.
        let mut plan: Vec<(String, f32)> = Vec::new();
        for (key, v) in values {
            if key.eq_ignore_ascii_case("mode") {
                let tab = match v.as_str().map(str::to_ascii_uppercase).as_deref() {
                    Some("MONO") => 0,
                    Some("POLY") => 1,
                    Some("UNISON") => 2,
                    _ => {
                        return Err(ApiError::params("`mode` is MONO, POLY or UNISON"));
                    }
                };
                let p = &self.params;
                let poly = match tab {
                    0 => 0.0,
                    1 => 1.0,
                    _ => f32::from(u8::from(p.poly.value())),
                };
                plan.push(("unison".into(), f32::from(u8::from(tab == 2))));
                plan.push(("poly".into(), poly));
                continue;
            }
            let e = values::find(&table, key).ok_or_else(|| unknown(key, &table))?;
            let n = e.parse(v)?;
            plan.push((e.id.to_owned(), n));
        }
        // As the call leaves them: DETUNE taken with DOUBLE set in the same call. And set in
        // that order, what makes a control count before it (a JSON object's keys may come in
        // any order).
        for (id, _) in &plan {
            self.operable_now(id, Some(&plan))
                .map_err(|why| ApiError::refused(why).with_data(json!({"id": id})))?;
        }
        const FIRST: [&str; 4] = ["unison", "poly", "doubled", "spread"];
        plan.sort_by_key(|(id, _)| !FIRST.contains(&id.as_str()));
        let setter = ParamSetter::new(self.context.as_ref());
        let mut changed = Vec::new();
        for (id, n) in &plan {
            let Some(e) = values::find(&table, id) else {
                continue;
            };
            if (e.param.normalized() - n).abs() > 1e-6 {
                e.param.set_once(&setter, *n);
                changed.push(id.clone());
            }
        }
        let after: Vec<Value> = plan
            .iter()
            .filter_map(|(id, _)| values::find(&table, id))
            .map(Entry::read)
            .collect();
        drop(table);
        Ok(json!({
            "values": after,
            "changed": changed,
            "mode": self.mode(),
            "preset": self.preset_state(),
        }))
    }

    fn reset(&mut self, args: &Value) -> Reply {
        let ids = match arg(args, "ids") {
            Some(Value::Array(a)) if !a.is_empty() => self.entries_for(args)?,
            _ => return Err(ApiError::params("`ids` must name at least one parameter")),
        };
        let table = values::table(&self.params);
        // The defaults, as values in the panel's units.
        let defaults: serde_json::Map<String, Value> = ids
            .iter()
            .filter_map(|id| values::find(&table, id))
            .map(|e| (e.id.to_owned(), e.describe()["default"].clone()))
            .collect();
        drop(table);
        self.set(&json!({"values": defaults}))
    }

    fn list_presets(&mut self, args: &Value) -> Reply {
        let search = match arg(args, "search") {
            Some(Value::String(s)) => s.clone(),
            None => String::new(),
            Some(_) => return Err(ApiError::params("`search` must be a string")),
        };
        let tags: Vec<String> = match arg(args, "tags") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|t| t.as_str().map(str::to_owned))
                .collect(),
            None => Vec::new(),
            Some(_) => return Err(ApiError::params("`tags` must be a list of strings")),
        };
        let favorites = flag_arg(args, "favorites", false)?;
        let mine = flag_arg(args, "mine", false)?;
        let (all, unreadable) = self.browser.library.entries();
        let current = Browser::current_name(&self.params);
        let list: Vec<Value> = library::matching(&all, &search, &tags, favorites, mine)
            .iter()
            .map(|e| {
                json!({
                    "name": e.sound.name,
                    "tags": e.sound.tags,
                    "description": (!e.sound.description.is_empty()).then(|| e.sound.description.clone()),
                    "origin": origin(e.origin),
                    "favorite": e.favorite,
                    "current": e.sound.name.eq_ignore_ascii_case(&current),
                })
            })
            .collect();
        Ok(json!({
            "presets": list,
            "total": all.len(),
            "tags": library::tags_of(&all),
            "current": self.preset_state(),
            "unreadable_files": unreadable,
        }))
    }

    fn find_preset(&self, name: &str) -> Result<library::Entry, ApiError> {
        self.browser.library.find(name).ok_or_else(|| {
            let (all, _) = self.browser.library.entries();
            let near: Vec<String> = library::matching(&all, name, &[], false, false)
                .into_iter()
                .take(8)
                .map(|e| e.sound.name)
                .collect();
            ApiError::params(format!("no preset is named `{name}`"))
                .with_data(json!({"similar": near}))
        })
    }

    fn load_preset(&mut self, args: &Value) -> Reply {
        let e = self.find_preset(text_arg(args, "name")?)?;
        self.playing_kept();
        let setter = ParamSetter::new(self.context.as_ref());
        self.browser.choose(&e, &self.params, &setter);
        Ok(json!({"preset": self.preset_state()}))
    }

    /// (What the drawer does before choosing: no gesture of the player's left open under it.)
    fn playing_kept(&mut self) {
        self.end_gestures();
    }

    fn step_preset(&mut self, args: &Value) -> Reply {
        let by = match arg(args, "by").and_then(Value::as_i64) {
            Some(b @ (-1 | 1)) => b as i32,
            _ => return Err(ApiError::params("`by` is 1 (next) or -1 (previous)")),
        };
        self.playing_kept();
        // (The list as the drawer's filters have it, read once if it has not been.)
        self.browser.tick(&self.params);
        let setter = ParamSetter::new(self.context.as_ref());
        self.browser.step(by, &self.params, &setter);
        Ok(json!({"preset": self.preset_state()}))
    }

    fn tags_of(args: &Value) -> Result<Option<Vec<String>>, ApiError> {
        match arg(args, "tags") {
            None => Ok(None),
            Some(Value::Array(a)) => {
                let mut tags = Vec::new();
                for t in a {
                    let t = t
                        .as_str()
                        .ok_or_else(|| ApiError::params("tags are strings"))?;
                    if t.contains(',') {
                        return Err(ApiError::params(format!("the tag `{t}` has a comma in it")));
                    }
                    tags.push(t.to_owned());
                }
                library::parse_tags(&tags.join(", "))
                    .map(Some)
                    .map_err(ApiError::params)
            }
            Some(_) => Err(ApiError::params("`tags` must be a list of strings")),
        }
    }

    fn save_preset(&mut self, args: &Value) -> Reply {
        let name = text_arg(args, "name")?.trim().to_owned();
        library::check_name(&name).map_err(ApiError::params)?;
        let tags = Self::tags_of(args)?;
        let replace = flag_arg(args, "replace", false)?;
        let there = self.browser.library.find(&name);
        if let Some(e) = &there
            && !replace
        {
            let what = match e.origin {
                Origin::Factory => "a factory preset",
                Origin::Edited => "your edit of a factory preset",
                Origin::User => "a preset of yours",
            };
            return Err(ApiError::refused(format!(
                "{} is {what}: give `replace` true to save over it",
                e.sound.name
            )));
        }
        let tags = tags.unwrap_or_else(|| there.map(|e| e.sound.tags).unwrap_or_default());
        match self.browser.save_sound(&name, tags, &self.params) {
            Ok(e) => {
                self.browser.drawer.hint = format!("Saved {}.", e.sound.name);
                Ok(
                    json!({"saved": e.sound.name, "origin": origin(e.origin), "preset": self.preset_state()}),
                )
            }
            Err(e) => Err(ApiError::refused(e)),
        }
    }

    fn rename_preset(&mut self, args: &Value) -> Reply {
        let e = self.find_preset(text_arg(args, "name")?)?;
        let to = text_arg(args, "to")?;
        match self.browser.rename_preset(&e.sound.name, to, &self.params) {
            Ok(n) => Ok(
                json!({"renamed": e.sound.name, "to": n.sound.name, "preset": self.preset_state()}),
            ),
            Err(err) => Err(ApiError::refused(err)),
        }
    }

    fn tag_preset(&mut self, args: &Value) -> Reply {
        let e = self.find_preset(text_arg(args, "name")?)?;
        let tags = Self::tags_of(args)?.ok_or_else(|| ApiError::params("`tags` is required"))?;
        self.browser
            .library
            .tag(&e.sound.name, tags.clone())
            .map_err(ApiError::refused)?;
        self.browser.reread(&self.params);
        Ok(json!({"preset": e.sound.name, "tags": tags}))
    }

    fn favor_preset(&mut self, args: &Value) -> Reply {
        let e = self.find_preset(text_arg(args, "name")?)?;
        let on = match arg(args, "favorite") {
            Some(Value::Bool(b)) => *b,
            _ => return Err(ApiError::params("`favorite` is true or false")),
        };
        self.browser
            .library
            .favorite(&e.sound.name, on)
            .map_err(ApiError::refused)?;
        self.browser.reread(&self.params);
        Ok(json!({"preset": e.sound.name, "favorite": on}))
    }

    fn delete_preset(&mut self, args: &Value) -> Reply {
        let e = self.find_preset(text_arg(args, "name")?)?;
        self.browser
            .library
            .remove(&e.sound.name)
            .map_err(ApiError::refused)?;
        self.browser.drawer.hint = format!("Deleted {}.", e.sound.name);
        self.browser.reread(&self.params);
        Ok(json!({
            "deleted": e.sound.name,
            "hidden": e.origin != Origin::User,
            "note": (e.origin != Origin::User).then_some("a factory preset is hidden; ca72_presets_restore brings it back"),
        }))
    }

    fn revert_preset(&mut self, args: &Value) -> Reply {
        let e = self.find_preset(text_arg(args, "name")?)?;
        if e.origin != Origin::Edited {
            return Err(ApiError::refused(
                "only a factory preset you changed can be reverted",
            ));
        }
        let back = self
            .browser
            .library
            .revert(&e.sound.name)
            .map_err(ApiError::refused)?;
        self.browser.drawer.hint = format!("{} is the factory's again.", e.sound.name);
        self.browser.reread(&self.params);
        Ok(json!({"reverted": e.sound.name, "factory": back}))
    }

    fn restore_presets(&mut self) -> Reply {
        let r = self.browser.library.restore().map_err(ApiError::refused)?;
        self.browser.drawer.hint = if r.is_empty() {
            "No factory preset was deleted.".into()
        } else {
            format!("Restored {}.", r.join(", "))
        };
        self.browser.reread(&self.params);
        Ok(json!({"restored": r}))
    }

    pub(super) fn settings_state(&self) -> Value {
        let s = &self.settings;
        let sizes: Vec<Value> = self
            .sizes()
            .into_iter()
            .map(|(label, width, fits)| json!({"label": label, "width": width, "fits": fits}))
            .collect();
        json!({
            "look": match s.skin { Skin::Worn => "REALISTIC", Skin::Drawn => "CLASSIC" },
            "lettering": match s.lettering { Lettering::Standard => "STANDARD", Lettering::Large => "LARGE" },
            "frequency_dials": match s.frequency { Law::Semitones => "SEMITONES", Law::Hardware => "HARDWARE" },
            "quality_indicator": s.shutter,
            "agents": s.agents,
            "agents_for_this_session": crate::agent::forced(),
            "size": {"width": self.width, "sizes": sizes},
        })
    }

    fn set_settings(&mut self, args: &Value) -> Reply {
        let Some(obj) = args.as_object() else {
            return Err(ApiError::params("settings are an object"));
        };
        let word = |k: &str| -> Result<Option<String>, ApiError> {
            match arg(args, k) {
                None => Ok(None),
                Some(Value::String(s)) => Ok(Some(s.to_ascii_uppercase())),
                Some(_) => Err(ApiError::params(format!("`{k}` must be a string"))),
            }
        };
        let look = match word("look")?.as_deref() {
            None => None,
            Some("REALISTIC") => Some(Skin::Worn),
            Some("CLASSIC") => Some(Skin::Drawn),
            Some(o) => {
                return Err(ApiError::params(format!(
                    "`look` is REALISTIC or CLASSIC, not {o}"
                )));
            }
        };
        let lettering = match word("lettering")?.as_deref() {
            None => None,
            Some("STANDARD") => Some(Lettering::Standard),
            Some("LARGE") => Some(Lettering::Large),
            Some(o) => {
                return Err(ApiError::params(format!(
                    "`lettering` is STANDARD or LARGE, not {o}"
                )));
            }
        };
        let dials = match word("frequency_dials")?.as_deref() {
            None => None,
            Some("SEMITONES") => Some(Law::Semitones),
            Some("HARDWARE") => Some(Law::Hardware),
            Some(o) => {
                return Err(ApiError::params(format!(
                    "`frequency_dials` is SEMITONES or HARDWARE, not {o}"
                )));
            }
        };
        let indicator = match arg(args, "quality_indicator") {
            None => None,
            Some(Value::Bool(b)) => Some(*b),
            Some(_) => return Err(ApiError::params("`quality_indicator` is true or false")),
        };
        let size = match arg(args, "size") {
            None => None,
            Some(v) => {
                let sizes = self.sizes();
                let chosen = match v {
                    Value::String(s)
                        if s.eq_ignore_ascii_case("fit")
                            || s.eq_ignore_ascii_case("fit to screen") =>
                    {
                        sizes.iter().find(|(l, ..)| *l == "FIT TO SCREEN")
                    }
                    Value::Number(n) => sizes
                        .iter()
                        .find(|(_, w, _)| Some(u64::from(*w)) == n.as_u64()),
                    _ => None,
                };
                match chosen {
                    Some((_, w, true)) => Some(*w),
                    Some((l, ..)) => {
                        return Err(ApiError::refused(format!("{l} does not fit the screen")));
                    }
                    None => {
                        return Err(ApiError::params("`size` is one of the widths ca72_settings lists, or FIT")
                            .with_data(json!({"sizes": sizes.iter().map(|(l, w, f)| json!({"label": l, "width": w, "fits": f})).collect::<Vec<_>>()})));
                    }
                }
            }
        };
        let agents = match arg(args, "agents") {
            None => None,
            Some(Value::Bool(false)) => Some(false),
            Some(_) => {
                return Err(ApiError::refused(
                    "an agent can only turn agents off; the player turns them on",
                ));
            }
        };
        let known = [
            "look",
            "lettering",
            "frequency_dials",
            "quality_indicator",
            "size",
            "agents",
        ];
        if let Some(k) = obj.keys().find(|k| !known.contains(&k.as_str())) {
            return Err(ApiError::params(format!(
                "no setting `{k}`; there are {}",
                known.join(", ")
            )));
        }
        if let Some(s) = look {
            self.set_look(s);
        }
        if let Some(l) = lettering {
            self.set_lettering(l);
        }
        if let Some(l) = dials {
            self.set_frequency_dials(l);
        }
        if let Some(shown) = indicator {
            self.keep_settings(Change::Shutter(shown));
        }
        if let Some(w) = size {
            // Asked of the host at the next frame, as the SIZE menu's are.
            self.gripped = Some(w);
        }
        if agents == Some(false) {
            self.keep_settings(Change::Agents(false));
            crate::agent::force(false);
            self.agents.stop = true;
        }
        let mut state = self.settings_state();
        if let Some(w) = size {
            state["size"]["width"] = json!(w);
        }
        Ok(state)
    }

    fn set_view(&mut self, args: &Value) -> Reply {
        let flag = |k: &str| -> Result<Option<bool>, ApiError> {
            match arg(args, k) {
                None => Ok(None),
                Some(Value::Bool(b)) => Ok(Some(*b)),
                Some(_) => Err(ApiError::params(format!("`{k}` is true or false"))),
            }
        };
        let (presets, panel, kb, midi) = (
            flag("presets")?,
            flag("panel")?,
            flag("keyboard")?,
            flag("midi_list")?,
        );
        let octave = match arg(args, "keyboard_octave") {
            None => None,
            Some(Value::String(s)) if s == "up" => Some(true),
            Some(Value::String(s)) if s == "down" => Some(false),
            Some(_) => return Err(ApiError::params("`keyboard_octave` is up or down")),
        };
        if let Some(on) = panel
            && on != self.expanded()
        {
            self.toggle_panel();
        }
        if let Some(on) = kb
            && on != self.playing.visible
        {
            self.strip_press(StripTarget::Bar(BarTarget::Keyboard));
        }
        if let Some(up) = octave {
            self.playing.shift(up);
            self.keep_settings(Change::KeyboardRoot(self.playing.root));
        }
        if let Some(on) = presets
            && on != self.browser.open
        {
            self.browser.show(on, &self.params);
            self.follow_drawer();
        }
        match midi {
            Some(true) => self.show_midi_list(None),
            Some(false) => self.learning.list = false,
            None => {}
        }
        Ok(self.view_state())
    }

    fn notes_of(args: &Value, required: bool) -> Result<Option<Vec<u8>>, ApiError> {
        match arg(args, "notes") {
            None if required => Err(ApiError::params("`notes` is required")),
            None => Ok(None),
            Some(Value::Array(a)) => a
                .iter()
                .map(|n| match n {
                    Value::Number(x) => x
                        .as_u64()
                        .filter(|x| *x <= 127)
                        .map(|x| x as u8)
                        .ok_or_else(|| ApiError::params(format!("note {x} is not 0 to 127"))),
                    Value::String(s) => note_named(s).ok_or_else(|| {
                        ApiError::params(format!("`{s}` is not a note (C4 is 60, middle C)"))
                    }),
                    _ => Err(ApiError::params("notes are numbers or names")),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Some),
            Some(_) => Err(ApiError::params("`notes` must be a list")),
        }
    }

    fn notes_on(&mut self, args: &Value, until: Option<Instant>) -> Reply {
        let notes = Self::notes_of(args, true)?.unwrap_or_default();
        if notes.len() > 16 {
            return Err(ApiError::params("at most 16 notes at once"));
        }
        let mut refused = Vec::new();
        for &n in &notes {
            self.playing.agent(n, true);
            if !self.playing.agent_notes().contains(&n) {
                refused.push(n);
            }
            if let Some(at) = until {
                self.agents.timed.retain(|(m, _)| *m != n);
                self.agents.timed.push((n, at));
            } else {
                self.agents.timed.retain(|(m, _)| *m != n);
            }
        }
        let mut v = json!({"holding": self.playing.agent_notes(), "names": self.playing.agent_notes().into_iter().map(keyboard::label).collect::<Vec<_>>()});
        if !refused.is_empty() {
            v["not_played"] = json!(refused);
            v["why"] = json!("the local notes' queue was full: every local note was let go");
        }
        if self.browser.open
            || self.typing.is_some()
            || self.learning.holds_keys(&self.params.midi_map)
        {
            v["note"] = json!(
                "while the presets' drawer, a typed value or MIDI Learn holds the keyboard, the editor lets local notes go at its next frame (as it does the on-screen keyboard's): close it first"
            );
        }
        Ok(v)
    }

    fn notes_off(&mut self, args: &Value) -> Reply {
        let notes = Self::notes_of(args, false)?.unwrap_or_else(|| self.playing.agent_notes());
        for n in notes {
            self.playing.agent(n, false);
            self.agents.timed.retain(|(m, _)| *m != n);
        }
        Ok(json!({"holding": self.playing.agent_notes()}))
    }

    fn notes_play(&mut self, args: &Value) -> Reply {
        let s = arg(args, "seconds")
            .and_then(Value::as_f64)
            .filter(|s| (0.01..=LONGEST_NOTE_S).contains(s))
            .ok_or_else(|| ApiError::params(format!("`seconds` is 0.01 to {LONGEST_NOTE_S}")))?;
        let until = Instant::now() + std::time::Duration::from_secs_f64(s);
        let mut v = self.notes_on(args, Some(until))?;
        v["seconds"] = json!(s);
        Ok(v)
    }

    fn learnable(&self, args: &Value, key: &str) -> Result<usize, ApiError> {
        let k = text_arg(args, key)?;
        let table = values::table(&self.params);
        let e = values::find(&table, k).ok_or_else(|| unknown(k, &table))?;
        learn::index(e.id).ok_or_else(|| {
            ApiError::refused(format!(
                "{} is not learned by MIDI Learn (as on the panel: POWER, QUALITY, the wheels and MIDI BEND RANGE are not)",
                e.label
            ))
        })
    }

    fn midi_learn(&mut self, args: &Value) -> Reply {
        let map = std::sync::Arc::clone(&self.params.midi_map);
        if flag_arg(args, "cancel", false)? {
            self.learning.cancel(&map);
            return Ok(self.midi_list());
        }
        let i = self.learnable(args, "control")?;
        match (arg(args, "channel"), arg(args, "cc")) {
            (None, None) => {
                self.learning.learn(&map, i);
                let mut v = self.midi_list();
                v["waiting"] = json!(format!(
                    "move a controller on a MIDI device: it will be {}'s",
                    LEARNABLE[i].label
                ));
                Ok(v)
            }
            (Some(ch), Some(cc)) => {
                let ch = ch
                    .as_u64()
                    .filter(|c| (1..=16).contains(c))
                    .ok_or_else(|| ApiError::params("`channel` is 1 to 16"))?;
                let cc = cc
                    .as_u64()
                    .filter(|c| *c <= 127)
                    .ok_or_else(|| ApiError::params("`cc` is 0 to 127"))?
                    as u8;
                if let Some(why) = learn::reserved(cc) {
                    return Err(ApiError::refused(format!(
                        "CC {cc} is {why}, which MIDI Learn never takes"
                    )));
                }
                let a = map.assign(
                    i,
                    Cc {
                        channel: (ch - 1) as u8,
                        cc,
                    },
                );
                let mut v = self.midi_list();
                if let Some(a) = a {
                    v["assigned"] = json!({
                        "control": LEARNABLE[a.param].id,
                        "controller": a.cc.text(),
                        "replaced": a.replaced.map(Cc::text),
                        "taken_from": a.displaced.map(|d| LEARNABLE[d].id),
                    });
                }
                Ok(v)
            }
            _ => Err(ApiError::params("give both `channel` and `cc`, or neither")),
        }
    }

    fn midi_list(&self) -> Value {
        let map = &self.params.midi_map;
        let armed = map.armed();
        let list: Vec<Value> = LEARNABLE
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let a = map.assignment(i);
                json!({
                    "control": l.id,
                    "label": l.label,
                    "controller": a.map(Cc::text),
                    "channel": a.map(|c| c.channel + 1),
                    "cc": a.map(|c| c.cc),
                    "learning": armed == Some(i),
                })
            })
            .collect();
        json!({
            "assignments": list,
            "learning": armed.map(|i| LEARNABLE[i].id),
            "never_learned": learn::RESERVED_TEXT,
        })
    }

    fn midi_remove(&mut self, args: &Value) -> Reply {
        let i = self.learnable(args, "control")?;
        let map = std::sync::Arc::clone(&self.params.midi_map);
        let had = map.assignment(i);
        self.learning.remove(&map, i);
        Ok(json!({"control": LEARNABLE[i].id, "removed": had.map(Cc::text)}))
    }

    fn version(&mut self, args: &Value) -> Reply {
        let check = flag_arg(args, "check", false)?;
        if check && self.update.checks() {
            self.update.press();
        }
        let s = self.update.scene();
        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "update": {"says": s.text, "button": s.button},
        }))
    }
}

fn origin(o: Origin) -> &'static str {
    match o {
        Origin::Factory => "factory",
        Origin::Edited => "edited",
        Origin::User => "yours",
    }
}

fn unknown(key: &str, table: &[Entry<'_>]) -> ApiError {
    ApiError::params(format!("no parameter `{key}`"))
        .with_data(json!({"ids": table.iter().map(|e| e.id).collect::<Vec<_>>()}))
}

/// A note's MIDI number from its name: a letter, sharps (#) or flats (b), the octave (C4 is
/// 60, C-1 is 0).
pub(super) fn note_named(s: &str) -> Option<u8> {
    let t = s.trim();
    let mut chars = t.chars();
    let letter = chars.next()?.to_ascii_uppercase();
    let mut pc: i32 = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest: String = chars.collect();
    let mut octave = rest.as_str();
    while let Some(c) = octave.chars().next() {
        match c {
            '#' | '\u{266f}' => pc += 1,
            'b' | '\u{266d}' => pc -= 1,
            _ => break,
        }
        octave = &octave[c.len_utf8()..];
    }
    let o: i32 = octave.parse().ok()?;
    let n = (o + 1) * 12 + pc;
    u8::try_from(n).ok().filter(|n| *n <= 127)
}

/// The standalone's audio and MIDI, as AUDIO & MIDI… shows them.
fn audio_state() -> Value {
    #[cfg(feature = "standalone-production")]
    if let Some(app) = plugin_kit_standalone::AppHandle::current() {
        let s = app.snapshot();
        let c = &s.config;
        let dev = |d: &Option<plugin_kit_standalone::DeviceId>| d.as_ref().map(|d| d.name.clone());
        return json!({
            "available": true,
            "state": format!("{:?}", s.phase).to_uppercase(),
            "error": s.error,
            "warning": s.warning,
            "output": dev(&c.output).unwrap_or_else(|| "System default".into()),
            "output_channels": c.output_channels.map(|x| x + 1),
            "external_input": dev(&c.input),
            "input_channels": c.input_channels.map(|x| x + 1),
            "sample_rate_requested": c.sample_rate,
            "buffer_frames_requested": c.buffer_frames,
            "sample_rate": s.actual_sample_rate,
            "buffer_frames": s.actual_buffer_frames,
            "buffer_ms": s.buffer_duration_ms(),
            "midi_inputs": if c.auto_midi { json!("automatic") } else { json!(c.midi_inputs) },
            "midi_channel": c.midi_channel.map_or(json!("ALL"), |ch| json!(ch)),
            "xruns": s.xruns,
            "devices": {
                "backend": s.devices.backend,
                "audio": s.devices.audio.iter().map(|d| json!({"name": d.id.name, "outputs": d.id.output_channels, "inputs": d.id.input_channels, "sample_rates": d.sample_rates})).collect::<Vec<_>>(),
                "midi_inputs": s.devices.midi_inputs,
            },
        });
    }
    json!({"available": false, "why": "only the standalone application has audio and MIDI settings of its own"})
}

/// A line of it for `describe`.
fn audio_summary() -> Value {
    let a = audio_state();
    if a["available"] == json!(true) {
        json!({"state": a["state"], "sample_rate": a["sample_rate"], "buffer_frames": a["buffer_frames"], "error": a["error"]})
    } else {
        json!(null)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use nih_plug::context::PluginApi;
    use nih_plug::context::process::TestProcessContext;
    use nih_plug::prelude::*;
    use nih_plug::wrapper::state::PluginState;

    use super::*;
    use crate::Ca72;
    use crate::agent::{bridge::Bridge, mcp};
    use crate::editor::Meters;
    use crate::library::Library;
    use crate::params::Ca72Params;

    /// A host that applies the editor's changes, as the standalone's does (the value set at
    /// once), through nih-plug's context for tests.
    struct Applying {
        api: PluginApi,
        context: Mutex<TestProcessContext<Ca72>>,
    }

    // SAFETY: the context's parameter pointers point into the `Arc<Ca72Params>` it keeps alive,
    // and every use of them is under its lock.
    #[allow(unsafe_code)]
    unsafe impl Send for Applying {}
    // SAFETY: as above.
    #[allow(unsafe_code)]
    unsafe impl Sync for Applying {}

    #[allow(unsafe_code)]
    impl GuiContext for Applying {
        fn plugin_api(&self) -> PluginApi {
            self.api
        }
        fn request_resize(&self) -> bool {
            true
        }
        unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {}
        unsafe fn raw_set_parameter_normalized(&self, param: ParamPtr, normalized: f32) {
            self.context
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .automate(param, normalized);
        }
        unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}
        fn get_state(&self) -> PluginState {
            panic!("not read")
        }
        fn set_state(&self, _state: PluginState) {}
    }

    fn editing(library: Library) -> (Editing, Arc<Ca72Params>) {
        let params = Arc::new(Ca72Params::default());
        let shared: Arc<dyn Params> = params.clone();
        let host: Arc<dyn GuiContext> = Arc::new(Applying {
            api: PluginApi::Standalone,
            context: Mutex::new(TestProcessContext::new(
                shared,
                48_000.0,
                ProcessMode::Realtime,
            )),
        });
        let e = Editing::new(
            host,
            Arc::clone(&params),
            Arc::new(Meters::default()),
            (1380, 1380, 1.0),
            library,
            |_| None,
        );
        (e, params)
    }

    /// An agent's MCP client: a bridge served on another thread, as its client starts it,
    /// spoken to through pipes in MCP's text.
    struct Agent {
        to: Option<std::io::PipeWriter>,
        from: std::sync::mpsc::Receiver<Value>,
        threads: Vec<std::thread::JoinHandle<()>>,
        next: u64,
    }

    impl Agent {
        fn new(bridge: Bridge) -> Self {
            use std::io::{BufRead, BufReader};
            let (bridge_in, to) = std::io::pipe().expect("pipe");
            let (from_bridge, bridge_out) = std::io::pipe().expect("pipe");
            let mut bridge = bridge;
            let serving = std::thread::spawn(move || {
                mcp::serve(&mut bridge, BufReader::new(bridge_in), bridge_out).expect("serve");
            });
            let (tx, from) = std::sync::mpsc::channel();
            let reading = std::thread::spawn(move || {
                for line in BufReader::new(from_bridge).lines() {
                    let Ok(line) = line else { break };
                    if tx.send(serde_json::from_str(&line).expect("json")).is_err() {
                        break;
                    }
                }
            });
            Agent {
                to: Some(to),
                from,
                threads: vec![serving, reading],
                next: 0,
            }
        }

        /// A message sent, and its answer, the editor's frames run meanwhile.
        fn ask(&mut self, e: &mut Editing, mut msg: Value) -> Value {
            use std::io::Write;
            self.next += 1;
            msg["jsonrpc"] = json!("2.0");
            msg["id"] = json!(self.next);
            writeln!(self.to.as_mut().expect("open"), "{msg}").expect("write");
            let began = Instant::now();
            loop {
                e.serve_agents();
                let p = Arc::clone(&e.params);
                e.browser.tick(&p);
                if let Ok(v) = self.from.try_recv() {
                    assert_eq!(v["id"], json!(self.next), "{v}");
                    return v;
                }
                assert!(
                    began.elapsed() < Duration::from_secs(30),
                    "no answer to {msg}"
                );
                std::thread::sleep(Duration::from_millis(2));
            }
        }

        /// A tool called: its result (`isError` or not).
        fn tool(&mut self, e: &mut Editing, name: &str, args: Value) -> Value {
            let v = self.ask(
                e,
                json!({"method": "tools/call", "params": {"name": name, "arguments": args}}),
            );
            v["result"].clone()
        }
    }

    impl Drop for Agent {
        fn drop(&mut self) {
            drop(self.to.take());
            for t in self.threads.drain(..) {
                let _ = t.join();
            }
        }
    }

    fn ok(r: &Value) -> &Value {
        assert_eq!(r["isError"], json!(false), "{r}");
        &r["structuredContent"]
    }

    /// The error a failed call says.
    fn refused(r: &Value) -> String {
        assert_eq!(r["isError"], json!(true), "{r}");
        r["structuredContent"]["error"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    }

    fn agent_and_editor() -> (tempfile::TempDir, Editing, Arc<Ca72Params>, Agent) {
        let t = tempfile::tempdir().expect("tempdir");
        let (mut e, params) = editing(Library::at(t.path().join("presets")));
        let dir = t.path().join("agents");
        e.agents.server =
            Some(Server::start_in(&dir, "standalone", "standalone-test").expect("server"));
        let agent = Agent::new(Bridge::at(dir));
        (t, e, params, agent)
    }

    /// An agent through the bridge and the socket to the editor reads the CA-72 and sets its
    /// parameters in the panel's units, refused outside their range (nothing of the call set)
    /// and where the panel refuses them.
    #[test]
    fn an_agent_reads_and_sets_the_parameters_through_the_bridge() {
        let (_t, mut e, params, mut a) = agent_and_editor();
        let hi = a.ask(
            &mut e,
            json!({"method": "initialize", "params": {"protocolVersion": "2025-06-18", "clientInfo": {"name": "test-agent"}}}),
        );
        assert_eq!(hi["result"]["serverInfo"]["name"], json!("ca-72"));
        let tools = a.ask(&mut e, json!({"method": "tools/list"}));
        assert!(tools["result"]["tools"].as_array().expect("tools").len() >= 20);
        let d = a.tool(&mut e, "ca72_describe", json!({}));
        let d = ok(&d);
        assert_eq!(d["version"], json!(env!("CARGO_PKG_VERSION")));
        assert_eq!(d["kind"], json!("standalone"));
        assert_eq!(d["mode"], json!("MONO"));
        assert_eq!(
            d["parameters"].as_array().expect("params").len(),
            values::ids(&params).len()
        );
        let s = a.tool(
            &mut e,
            "ca72_set",
            json!({"values": {"cutoff": 2.5, "osc1_waveform": "Square", "osc2_frequency": 7, "mode": "POLY", "voices": 6}}),
        );
        assert_eq!(ok(&s)["mode"], json!("POLY"));
        assert_eq!(params.cutoff.value(), 2.5);
        assert_eq!(params.osc1_waveform.value(), crate::params::Wave::Square);
        let st = ca72::freq_dial::Pot::Two
            .semitones(crate::params::wiper_of(params.osc2_frequency.value()));
        assert!((st - 7.0).abs() < 1e-3, "{st}");
        assert!(params.poly.value() && !params.unison.value());
        assert_eq!(params.voices.value(), 6);
        assert!(
            e.touched,
            "AUTO GAIN's curve asked for, as after the player's edits"
        );
        let r = a.tool(
            &mut e,
            "ca72_set",
            json!({"values": {"cutoff": 0, "emphasis": 11}}),
        );
        assert!(refused(&r).contains("EMPHASIS"), "{r}");
        assert_eq!(r["structuredContent"]["details"]["max"], json!(10.0));
        assert_eq!(params.cutoff.value(), 2.5, "nothing of the call set");
        let r = a.tool(&mut e, "ca72_set", json!({"values": {"double": 25}}));
        assert!(refused(&r).contains("SCATTER"));
        ok(&a.tool(
            &mut e,
            "ca72_set",
            json!({"values": {"doubled": true, "double": 25}}),
        ));
        assert!(params.doubled.value() && (params.double.value() - 25.0).abs() < 1e-3);
        let g = a.tool(
            &mut e,
            "ca72_get",
            json!({"ids": ["cutoff", "OSCILLATOR-2 FREQUENCY"]}),
        );
        let g = ok(&g)["parameters"].clone();
        assert_eq!(
            (g[0]["value"].clone(), g[1]["id"].clone()),
            (json!(2.5), json!("osc2_frequency"))
        );
        assert!(
            refused(&a.tool(&mut e, "ca72_get", json!({"ids": ["nothing"]}))).contains("nothing")
        );
        ok(&a.tool(&mut e, "ca72_reset", json!({"ids": ["cutoff"]})));
        assert_eq!(params.cutoff.value(), 0.0);
        let unknown = a.ask(
            &mut e,
            json!({"method": "tools/call", "params": {"name": "ca72_nothing", "arguments": {}}}),
        );
        assert_eq!(unknown["error"]["code"], json!(codes::INVALID_PARAMS));
    }

    /// Presets saved, refused over another without `replace`, marked changed, loaded,
    /// listed, renamed, starred and deleted, through the drawer's library.
    #[test]
    fn an_agent_keeps_presets_as_the_drawer_does() {
        let (_t, mut e, params, mut a) = agent_and_editor();
        ok(&a.tool(&mut e, "ca72_set", json!({"values": {"cutoff": 2.5}})));
        let saved = a.tool(
            &mut e,
            "ca72_preset_save",
            json!({"name": "Agent Test", "tags": ["test", "bass"]}),
        );
        let p = ok(&saved)["preset"].clone();
        assert_eq!(
            (p["name"].clone(), p["changed"].clone()),
            (json!("Agent Test"), json!(false))
        );
        assert_eq!(e.browser.drawer.hint, "Saved Agent Test.");
        assert!(
            refused(&a.tool(&mut e, "ca72_preset_save", json!({"name": "agent test"})))
                .contains("replace")
        );
        assert!(refused(&a.tool(&mut e, "ca72_preset_save", json!({"name": "a/b"}))).contains('/'));
        let moved = a.tool(&mut e, "ca72_set", json!({"values": {"cutoff": -1}}));
        assert_eq!(ok(&moved)["preset"]["changed"], json!(true));
        let loaded = a.tool(&mut e, "ca72_preset_load", json!({"name": "agent test"}));
        assert_eq!(ok(&loaded)["preset"]["changed"], json!(false));
        assert_eq!(params.cutoff.value(), 2.5, "the preset's cutoff again");
        let list = a.tool(&mut e, "ca72_presets", json!({"search": "agent"}));
        let list = ok(&list)["presets"].clone();
        assert_eq!(list.as_array().map(Vec::len), Some(1));
        assert_eq!(
            (list[0]["origin"].clone(), list[0]["tags"].clone()),
            (json!("yours"), json!(["test", "bass"]))
        );
        ok(&a.tool(
            &mut e,
            "ca72_preset_rename",
            json!({"name": "Agent Test", "to": "Agent Two"}),
        ));
        assert_eq!(Browser::current_name(&params), "Agent Two");
        ok(&a.tool(
            &mut e,
            "ca72_preset_favorite",
            json!({"name": "Agent Two", "favorite": true}),
        ));
        ok(&a.tool(
            &mut e,
            "ca72_preset_tags",
            json!({"name": "Agent Two", "tags": ["lead"]}),
        ));
        let favs = a.tool(
            &mut e,
            "ca72_presets",
            json!({"favorites": true, "mine": true}),
        );
        let favs = ok(&favs)["presets"].clone();
        assert_eq!(
            (favs[0]["name"].clone(), favs[0]["tags"].clone()),
            (json!("Agent Two"), json!(["lead"]))
        );
        let all = a.tool(&mut e, "ca72_presets", json!({}));
        let factory = ok(&all)["presets"]
            .as_array()
            .expect("list")
            .iter()
            .find(|p| p["origin"] == json!("factory"))
            .expect("a factory preset")["name"]
            .as_str()
            .expect("name")
            .to_owned();
        let stepped = a.tool(&mut e, "ca72_preset_step", json!({"by": 1}));
        assert!(ok(&stepped)["preset"]["name"].is_string());
        ok(&a.tool(&mut e, "ca72_preset_delete", json!({"name": "Agent Two"})));
        assert!(e.browser.library.find("Agent Two").is_none());
        // A factory preset hidden, and brought back.
        let gone = a.tool(&mut e, "ca72_preset_delete", json!({"name": factory}));
        assert_eq!(ok(&gone)["hidden"], json!(true));
        assert!(e.browser.library.find(&factory).is_none());
        let back = a.tool(&mut e, "ca72_presets_restore", json!({}));
        assert_eq!(ok(&back)["restored"], json!([factory]));
        let r = a.tool(
            &mut e,
            "ca72_preset_load",
            json!({"name": "Nothing Like It"}),
        );
        assert!(refused(&r).contains("Nothing Like It"));
        assert!(
            refused(&a.tool(&mut e, "ca72_preset_revert", json!({"name": factory})))
                .contains("only")
        );
    }

    /// Notes played through the local notes' queue, as the on-screen keyboard plays them,
    /// a timed one let go when due; settings, the view and MIDI Learn changed as the menu,
    /// the rail and a control's menu change them; agents turned off, after which the agent is
    /// told none listen.
    #[test]
    fn an_agent_plays_notes_and_changes_settings_view_and_midi_learn() {
        let (_t, mut e, params, mut a) = agent_and_editor();
        let notes = Arc::clone(&e.playing.notes);
        let on = a.tool(&mut e, "ca72_note_on", json!({"notes": ["C4", 64]}));
        assert_eq!(ok(&on)["holding"], json!([60, 64]));
        assert_eq!(ok(&on)["names"], json!(["C4", "E4"]));
        assert_eq!(
            ok(&a.tool(&mut e, "ca72_note_off", json!({})))["holding"],
            json!([])
        );
        let played = a.tool(
            &mut e,
            "ca72_play",
            json!({"notes": ["G4"], "seconds": 0.05}),
        );
        assert_eq!(ok(&played)["holding"], json!([67]));
        assert!(refused(&a.tool(&mut e, "ca72_note_on", json!({"notes": ["H4"]}))).contains("H4"));
        assert!(
            refused(&a.tool(&mut e, "ca72_play", json!({"notes": [60], "seconds": 31})))
                .contains("seconds")
        );
        let began = Instant::now();
        while !e.playing.agent_notes().is_empty() {
            e.serve_agents();
            assert!(began.elapsed() < Duration::from_secs(2), "G4 never let go");
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut seen = Vec::new();
        while let Some(ev) = notes.pop() {
            seen.push((ev.note, ev.on));
        }
        assert_eq!(
            seen,
            [
                (60, true),
                (64, true),
                (60, false),
                (64, false),
                (67, true),
                (67, false)
            ]
        );
        let set = a.tool(
            &mut e,
            "ca72_settings_set",
            json!({"look": "CLASSIC", "frequency_dials": "HARDWARE", "quality_indicator": false}),
        );
        let set = ok(&set).clone();
        assert_eq!(
            (set["look"].clone(), set["frequency_dials"].clone()),
            (json!("CLASSIC"), json!("HARDWARE"))
        );
        assert_eq!(e.renderer.skin(), Skin::Drawn);
        assert_eq!(params.freq_law(), Law::Hardware);
        assert!(!e.settings.shutter);
        assert!(
            refused(&a.tool(&mut e, "ca72_settings_set", json!({"agents": true})))
                .contains("player")
        );
        assert!(
            refused(&a.tool(&mut e, "ca72_settings_set", json!({"colour": "red"})))
                .contains("colour")
        );
        assert!(
            refused(&a.tool(&mut e, "ca72_settings_set", json!({"size": 1234}))).contains("size")
        );
        let size = a.tool(&mut e, "ca72_settings_set", json!({"size": 1035}));
        assert_eq!(ok(&size)["size"]["width"], json!(1035));
        assert_eq!(
            e.gripped,
            Some(1035),
            "asked of the host at the next frame, as the menu's"
        );
        let shown = e.playing.visible;
        let view = a.tool(
            &mut e,
            "ca72_view",
            json!({"keyboard": !shown, "presets": true}),
        );
        assert_eq!(
            (ok(&view)["keyboard"].clone(), ok(&view)["presets"].clone()),
            (json!(!shown), json!(true))
        );
        assert!(e.browser.open);
        let view = a.tool(
            &mut e,
            "ca72_view",
            json!({"presets": false, "keyboard_octave": "up"}),
        );
        assert_eq!(ok(&view)["presets"], json!(false));
        let learned = a.tool(
            &mut e,
            "ca72_midi_learn",
            json!({"control": "cutoff", "channel": 1, "cc": 74}),
        );
        assert_eq!(
            ok(&learned)["assigned"]["controller"],
            json!("CH 1 · CC 74")
        );
        assert!(
            refused(&a.tool(
                &mut e,
                "ca72_midi_learn",
                json!({"control": "PITCH (wheel)"})
            ))
            .contains("not learned")
        );
        assert!(
            refused(&a.tool(
                &mut e,
                "ca72_midi_learn",
                json!({"control": "cutoff", "channel": 1, "cc": 1})
            ))
            .contains("modulation wheel")
        );
        let waiting = a.tool(&mut e, "ca72_midi_learn", json!({"control": "EMPHASIS"}));
        assert_eq!(ok(&waiting)["learning"], json!("emphasis"));
        assert_eq!(params.midi_map.armed(), learn::index("emphasis"));
        ok(&a.tool(&mut e, "ca72_midi_learn", json!({"cancel": true})));
        assert_eq!(params.midi_map.armed(), None);
        ok(&a.tool(&mut e, "ca72_midi_forget", json!({"control": "cutoff"})));
        assert_eq!(
            params
                .midi_map
                .assignment(learn::index("cutoff").expect("cutoff")),
            None
        );
        let v = a.tool(&mut e, "ca72_version", json!({}));
        assert_eq!(ok(&v)["version"], json!(env!("CARGO_PKG_VERSION")));
        assert_eq!(
            ok(&a.tool(&mut e, "ca72_audio", json!({})))["available"],
            json!(false)
        );
        ok(&a.tool(&mut e, "ca72_panic", json!({})));
        // Agents off: the socket closed, its record gone, and the agent told.
        ok(&a.tool(&mut e, "ca72_settings_set", json!({"agents": false})));
        assert!(e.agents.server.is_none());
        let after = a.tool(&mut e, "ca72_describe", json!({}));
        assert!(refused(&after).contains("AGENTS"), "{after}");
        assert_eq!(
            after["structuredContent"]["code"],
            json!(codes::NO_INSTANCE)
        );
    }

    /// A client that sends a request and goes before the editor's next frame: the editor goes
    /// on, nothing waits on it, and the next agent is served.
    #[test]
    fn a_client_gone_mid_request_leaves_the_editor_serving() {
        use std::io::Write;
        let (_t, mut e, params, mut a) = agent_and_editor();
        {
            let path = e
                .agents
                .server
                .as_ref()
                .expect("server")
                .record()
                .endpoint
                .clone();
            let mut c = std::os::unix::net::UnixStream::connect(path).expect("connect");
            writeln!(
                c,
                r#"{{"jsonrpc":"2.0","id":1,"method":"parameters.set","params":{{"values":{{"cutoff":3}}}}}}"#
            )
            .expect("write");
        }
        let began = Instant::now();
        while began.elapsed() < Duration::from_millis(400) {
            e.serve_agents();
            std::thread::sleep(Duration::from_millis(5));
        }
        // (Set if the editor took it before its connection noticed the close; never half.)
        assert!(matches!(params.cutoff.value(), 0.0 | 3.0));
        ok(&a.tool(&mut e, "ca72_set", json!({"values": {"cutoff": 4}})));
        assert_eq!(params.cutoff.value(), 4.0);
    }

    #[test]
    fn notes_are_named_as_the_keyboard_names_them() {
        assert_eq!(note_named("C4"), Some(60));
        assert_eq!(note_named("c-1"), Some(0));
        assert_eq!(note_named("G9"), Some(127));
        assert_eq!(note_named("G#9"), None);
        assert_eq!(note_named("F#2"), Some(42));
        assert_eq!(note_named("Bb3"), Some(58));
        assert_eq!(note_named("A4"), Some(69));
        assert_eq!(note_named("H4"), None);
        for n in 0..=127u8 {
            assert_eq!(
                note_named(&keyboard::label(n)),
                Some(n),
                "{}",
                keyboard::label(n)
            );
        }
    }
}
