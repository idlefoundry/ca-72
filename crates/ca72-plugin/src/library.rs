//! The presets (decisions.md R10), the library DAW-v3's Minimoog shares (its DMM-P): a sound
//! is named panel settings, the dials' plain values by parameter ID (a choice by its index,
//! a switch 0 or 1, POLY and VOICES too; never the PITCH wheel or the bypass).
//!
//! - **The factory's** are built in (`sounds/minimoog.toml`, DAW-v3's file byte for byte)
//!   and never change.
//! - **The user's** are files in the library's folder, `<name>.toml` (`format`, `name`,
//!   `description`, `tags`, `favorite`, `replaces`, `values`), written whole and moved into
//!   place so the other program never reads half of one. A user preset named as a factory
//!   one (or saying it `replaces` one) stands in for it: an edit or a rename of a factory
//!   preset, taken back by [`Library::revert`].
//! - **The factory's tags, favourites and deletions** are the folder's `factory.toml`
//!   (`[presets."<name>"]`: `tags`, `favorite`, `hidden`); [`Library::restore`] brings the
//!   deleted back.
//! - The folder is `<shared>/CA-72/Presets`, `<shared>` being `$IDLE_FOUNDRY_SHARED`, else the
//!   user's data folder's `Idle Foundry` (`~/.local/share`, `~/Library/Application Support`,
//!   `%APPDATA%`).
//!
//! Names are 1 to 64 characters, none of `/ \ : * ? " < > |`, not starting or ending with a
//! space or a dot, unique ignoring case.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use toml::Value;

/// The format of the files this reads and writes.
pub const FORMAT: i64 = 1;

/// The parameters a preset never sets: the PITCH wheel (where the player leaves it) and the
/// bypass (the host's).
pub const KEPT: &[&str] = &["pitch_wheel", "bypass"];

const MOST_TAGS: usize = 16;
const LONGEST_TAG: usize = 32;

/// A preset's sound: its values (plain) and what a browser shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub values: Vec<(String, f64)>,
}

/// Where a preset comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Factory,
    /// The user's version of a factory preset (edited or renamed), standing in for it.
    Edited,
    User,
}

/// A preset as the library offers it.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub sound: Sound,
    pub favorite: bool,
    pub origin: Origin,
    /// The factory preset it stands in for.
    pub replaces: Option<String>,
    file: Option<PathBuf>,
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// A sound from its table (`name`, `description?`, `tags?`, `values`).
fn sound_of(t: &toml::Table) -> Result<Sound, String> {
    let name = t
        .get("name")
        .and_then(Value::as_str)
        .ok_or("a preset has no `name`")?
        .to_owned();
    let description = t
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let tags = match t.get("tags") {
        None => Vec::new(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| v.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| format!("{name}: `tags` holds text only"))?,
        Some(_) => return Err(format!("{name}: `tags` is a list")),
    };
    let mut values = Vec::new();
    if let Some(v) = t.get("values") {
        let v = v
            .as_table()
            .ok_or_else(|| format!("{name}: `values` is a table"))?;
        for (k, x) in v {
            let x = match x {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                Value::Boolean(b) => f64::from(u8::from(*b)),
                _ => return Err(format!("{name}: `{k}` is not a number")),
            };
            values.push((k.clone(), x));
        }
    }
    Ok(Sound {
        name,
        description,
        tags,
        values,
    })
}

/// The factory's presets.
pub fn factory() -> &'static [Sound] {
    static FACTORY: OnceLock<Vec<Sound>> = OnceLock::new();
    FACTORY
        .get_or_init(|| parse_factory(include_str!("../sounds/minimoog.toml")).unwrap_or_default())
}

/// A factory file's presets.
pub fn parse_factory(text: &str) -> Result<Vec<Sound>, String> {
    let t: toml::Table = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
    match t.get("format").and_then(Value::as_integer) {
        Some(FORMAT) => {}
        other => return Err(format!("presets of format {other:?}, not {FORMAT}")),
    }
    match t.get("preset") {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => a
            .iter()
            .map(|p| {
                p.as_table()
                    .ok_or_else(|| "a preset is a table".to_owned())
                    .and_then(sound_of)
            })
            .collect(),
        Some(_) => Err("`preset` is a list".into()),
    }
}

/// Whether `name` can be a preset's (and its file's) name.
pub fn check_name(name: &str) -> Result<(), String> {
    let ok = !name.is_empty()
        && name.chars().count() <= 64
        && !name.starts_with(['.', ' '])
        && !name.ends_with([' ', '.'])
        && !name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        && !name.chars().any(char::is_control);
    if ok {
        Ok(())
    } else {
        Err("A name is 1 to 64 characters, without / \\ : * ? \" < > |".into())
    }
}

/// Tags as typed (`bass, dark`): trimmed, without repeats (ignoring case) or empty ones.
pub fn parse_tags(text: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for t in text.split(',').map(str::trim).filter(|t| !t.is_empty()) {
        if t.chars().count() > LONGEST_TAG || t.chars().any(char::is_control) {
            return Err(format!("A tag is 1 to {LONGEST_TAG} characters: {t}"));
        }
        if !out.iter().any(|o| same(o, t)) {
            out.push(t.to_owned());
        }
    }
    if out.len() > MOST_TAGS {
        return Err(format!("At most {MOST_TAGS} tags"));
    }
    Ok(out)
}

/// The folder of the libraries the plug-in shares with DAW-v3.
pub fn shared_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("IDLE_FOUNDRY_SHARED") {
        return PathBuf::from(d);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(windows) {
        if let Some(a) = std::env::var_os("APPDATA") {
            return PathBuf::from(a).join("Idle Foundry");
        }
    } else if cfg!(target_os = "macos") {
        if let Some(h) = &home {
            return h.join("Library/Application Support/Idle Foundry");
        }
    } else if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(x).join("Idle Foundry");
    } else if let Some(h) = &home {
        return h.join(".local/share/Idle Foundry");
    }
    PathBuf::from("idle-foundry-shared")
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FactoryState {
    tags: Option<Vec<String>>,
    favorite: bool,
    hidden: bool,
}

fn strings(list: &[String]) -> Value {
    Value::Array(list.iter().map(|s| Value::String(s.clone())).collect())
}

/// The library: its folder.
#[derive(Clone, Debug)]
pub struct Library {
    dir: PathBuf,
}

impl Library {
    /// The library DAW-v3 shares: `<shared>/CA-72/Presets`.
    pub fn shared() -> Self {
        Library::at(shared_dir().join("CA-72").join("Presets"))
    }

    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Library { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn overlay_file(&self) -> PathBuf {
        self.dir.join("factory.toml")
    }

    fn read_overlay(&self) -> BTreeMap<String, FactoryState> {
        let Some(t) = std::fs::read_to_string(self.overlay_file())
            .ok()
            .and_then(|s| s.parse::<toml::Table>().ok())
        else {
            return BTreeMap::new();
        };
        let Some(Value::Table(p)) = t.get("presets") else {
            return BTreeMap::new();
        };
        p.iter()
            .filter_map(|(name, s)| {
                let s = s.as_table()?;
                Some((
                    name.clone(),
                    FactoryState {
                        tags: s.get("tags").and_then(Value::as_array).map(|a| {
                            a.iter()
                                .filter_map(|t| t.as_str().map(str::to_owned))
                                .collect()
                        }),
                        favorite: s.get("favorite").and_then(Value::as_bool).unwrap_or(false),
                        hidden: s.get("hidden").and_then(Value::as_bool).unwrap_or(false),
                    },
                ))
            })
            .collect()
    }

    fn write_overlay(&self, states: &BTreeMap<String, FactoryState>) -> Result<(), String> {
        let mut presets = toml::Table::new();
        for (name, s) in states {
            if *s == FactoryState::default() {
                continue;
            }
            let mut t = toml::Table::new();
            if let Some(tags) = &s.tags {
                t.insert("tags".into(), strings(tags));
            }
            if s.favorite {
                t.insert("favorite".into(), Value::Boolean(true));
            }
            if s.hidden {
                t.insert("hidden".into(), Value::Boolean(true));
            }
            presets.insert(name.clone(), Value::Table(t));
        }
        let mut t = toml::Table::new();
        t.insert("format".into(), Value::Integer(FORMAT));
        t.insert("presets".into(), Value::Table(presets));
        write(&self.overlay_file(), &t)
    }

    fn user_file(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.toml"))
    }

    /// The user's presets (with their files, favourites and what they stand in for), and
    /// the files that could not be read.
    #[allow(clippy::type_complexity)]
    fn users(&self) -> (Vec<(PathBuf, Sound, bool, Option<String>)>, Vec<String>) {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).collect())
            .unwrap_or_default();
        files.sort();
        let (mut out, mut bad) = (Vec::new(), Vec::new());
        for f in files {
            if f.extension().is_none_or(|e| e != "toml")
                || f.file_name().is_some_and(|n| n == "factory.toml")
            {
                continue;
            }
            let read = std::fs::read_to_string(&f)
                .map_err(|e| e.to_string())
                .and_then(|s| s.parse::<toml::Table>().map_err(|e| e.to_string()))
                .and_then(|t| {
                    match t.get("format").and_then(Value::as_integer) {
                        Some(FORMAT) | None => {}
                        Some(other) => return Err(format!("format {other}")),
                    }
                    let sound = sound_of(&t)?;
                    let fav = t.get("favorite").and_then(Value::as_bool).unwrap_or(false);
                    let rep = t.get("replaces").and_then(Value::as_str).map(str::to_owned);
                    Ok((sound, fav, rep))
                });
            match read {
                Ok((s, fav, rep)) => out.push((f, s, fav, rep)),
                Err(e) => bad.push(format!("{}: {e}", f.display())),
            }
        }
        (out, bad)
    }

    /// Every preset, by name (ignoring case), and the files that could not be read.
    pub fn entries(&self) -> (Vec<Entry>, Vec<String>) {
        let factory = factory();
        let overlay = self.read_overlay();
        let (users, bad) = self.users();
        let stands_for = |s: &Sound, r: &Option<String>| -> Option<String> {
            r.clone().or_else(|| {
                factory
                    .iter()
                    .find(|f| same(&f.name, &s.name))
                    .map(|f| f.name.clone())
            })
        };
        let replaced: Vec<String> = users
            .iter()
            .filter_map(|(_, s, _, r)| stands_for(s, r))
            .collect();
        let mut list: Vec<Entry> = factory
            .iter()
            .filter(|f| {
                !overlay.get(&f.name).is_some_and(|s| s.hidden)
                    && !replaced.iter().any(|r| same(r, &f.name))
            })
            .map(|f| {
                let state = overlay.get(&f.name).cloned().unwrap_or_default();
                let mut sound = f.clone();
                if let Some(t) = state.tags {
                    sound.tags = t;
                }
                Entry {
                    sound,
                    favorite: state.favorite,
                    origin: Origin::Factory,
                    replaces: None,
                    file: None,
                }
            })
            .collect();
        for (file, sound, favorite, r) in users {
            let replaces =
                stands_for(&sound, &r).filter(|r| factory.iter().any(|f| same(&f.name, r)));
            list.push(Entry {
                origin: if replaces.is_some() {
                    Origin::Edited
                } else {
                    Origin::User
                },
                sound,
                favorite,
                replaces,
                file: Some(file),
            });
        }
        list.sort_by(|a, b| {
            a.sound
                .name
                .to_lowercase()
                .cmp(&b.sound.name.to_lowercase())
                .then(a.sound.name.cmp(&b.sound.name))
        });
        (list, bad)
    }

    /// The preset of this name (ignoring case).
    pub fn find(&self, name: &str) -> Option<Entry> {
        self.entries()
            .0
            .into_iter()
            .find(|e| same(&e.sound.name, name))
    }

    /// The presets that match: each of `query`'s words in the name, the description or a
    /// tag (ignoring case); every one of `tags`; favourites only; the user's only (and the
    /// factory's edited).
    pub fn search(&self, query: &str, tags: &[String], favorite: bool, mine: bool) -> Vec<Entry> {
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        self.entries()
            .0
            .into_iter()
            .filter(|e| {
                let text = format!(
                    "{} {} {}",
                    e.sound.name,
                    e.sound.description,
                    e.sound.tags.join(" ")
                )
                .to_lowercase();
                words.iter().all(|w| text.contains(w.as_str()))
                    && tags.iter().all(|t| e.sound.tags.iter().any(|x| same(x, t)))
                    && (!favorite || e.favorite)
                    && (!mine || e.origin != Origin::Factory)
            })
            .collect()
    }

    /// Every tag in use, by name.
    pub fn tags(&self) -> Vec<String> {
        let mut all: Vec<String> = Vec::new();
        for e in self.entries().0 {
            for t in e.sound.tags {
                if !all.iter().any(|a| same(a, &t)) {
                    all.push(t);
                }
            }
        }
        all.sort_by_key(|t| t.to_lowercase());
        all
    }

    fn write_user(
        &self,
        sound: &Sound,
        favorite: bool,
        replaces: Option<&str>,
    ) -> Result<PathBuf, String> {
        let mut t = toml::Table::new();
        t.insert("format".into(), Value::Integer(FORMAT));
        t.insert("name".into(), Value::String(sound.name.clone()));
        if !sound.description.is_empty() {
            t.insert(
                "description".into(),
                Value::String(sound.description.clone()),
            );
        }
        t.insert("tags".into(), strings(&sound.tags));
        t.insert("favorite".into(), Value::Boolean(favorite));
        if let Some(r) = replaces {
            t.insert("replaces".into(), Value::String(r.to_owned()));
        }
        let mut values = toml::Table::new();
        for (k, x) in &sound.values {
            let v = if x.fract() == 0.0 && x.abs() < 1e6 {
                Value::Integer(*x as i64)
            } else {
                Value::Float(*x)
            };
            values.insert(k.clone(), v);
        }
        t.insert("values".into(), Value::Table(values));
        let file = self.user_file(&sound.name);
        write(&file, &t)?;
        Ok(file)
    }

    /// The current sound saved as `name` (`values` plain, by parameter ID): replacing the
    /// user's preset of that name (its description, tags and favourite kept unless `tags`
    /// is given), or, named as a factory preset, the user's version standing in for it.
    pub fn save(
        &self,
        name: &str,
        values: Vec<(String, f64)>,
        tags: Option<Vec<String>>,
    ) -> Result<Entry, String> {
        let name = name.trim();
        check_name(name)?;
        let existing = self.find(name);
        let factory = factory()
            .iter()
            .find(|f| same(&f.name, name))
            .map(|f| f.name.clone());
        let (description, old_tags, favorite, replaces, old_file) = match existing {
            Some(e) => (
                e.sound.description,
                e.sound.tags,
                e.favorite,
                e.replaces.or(factory),
                e.file,
            ),
            None => (String::new(), Vec::new(), false, factory, None),
        };
        let sound = Sound {
            name: name.to_owned(),
            description,
            tags: tags.unwrap_or(old_tags),
            values,
        };
        let file = self.write_user(&sound, favorite, replaces.as_deref())?;
        if let Some(old) = old_file
            && old != file
            && !same(&old.to_string_lossy(), &file.to_string_lossy())
        {
            let _ = std::fs::remove_file(old);
        }
        self.find(name)
            .ok_or_else(|| format!("{name} was not saved"))
    }

    /// A preset renamed; a factory preset renamed is the user's version standing in for it.
    pub fn rename(&self, name: &str, to: &str) -> Result<Entry, String> {
        let to = to.trim();
        check_name(to)?;
        let e = self.find(name).ok_or_else(|| format!("No preset {name}"))?;
        if self
            .entries()
            .0
            .iter()
            .any(|o| same(&o.sound.name, to) && !same(&o.sound.name, &e.sound.name))
        {
            return Err(format!("There is already a preset {to}"));
        }
        let replaces = e
            .replaces
            .clone()
            .or_else(|| (e.origin == Origin::Factory).then(|| e.sound.name.clone()));
        let mut sound = e.sound.clone();
        sound.name = to.to_owned();
        let target = self.user_file(to);
        let case_only = e
            .file
            .as_ref()
            .is_some_and(|f| *f != target && same(&f.to_string_lossy(), &target.to_string_lossy()));
        if case_only && let Some(old) = &e.file {
            std::fs::remove_file(old).map_err(|err| err.to_string())?;
        }
        let file = self.write_user(&sound, e.favorite, replaces.as_deref())?;
        if let Some(old) = &e.file
            && *old != file
            && old.exists()
            && !case_only
        {
            std::fs::remove_file(old).map_err(|err| err.to_string())?;
        }
        self.find(to).ok_or_else(|| format!("{to} was not saved"))
    }

    fn mark(
        &self,
        e: &Entry,
        tags: Option<Vec<String>>,
        favorite: Option<bool>,
    ) -> Result<(), String> {
        match &e.file {
            Some(_) => {
                let mut sound = e.sound.clone();
                if let Some(t) = tags {
                    sound.tags = t;
                }
                self.write_user(
                    &sound,
                    favorite.unwrap_or(e.favorite),
                    e.replaces.as_deref(),
                )
                .map(|_| ())
            }
            None => {
                let mut overlay = self.read_overlay();
                let s = overlay.entry(e.sound.name.clone()).or_default();
                if let Some(t) = tags {
                    s.tags = Some(t);
                }
                if let Some(f) = favorite {
                    s.favorite = f;
                }
                self.write_overlay(&overlay)
            }
        }
    }

    /// A preset's tags, replaced.
    pub fn tag(&self, name: &str, tags: Vec<String>) -> Result<(), String> {
        let e = self.find(name).ok_or_else(|| format!("No preset {name}"))?;
        self.mark(&e, Some(tags), None)
    }

    /// A preset made a favourite, or not.
    pub fn favorite(&self, name: &str, on: bool) -> Result<(), String> {
        let e = self.find(name).ok_or_else(|| format!("No preset {name}"))?;
        self.mark(&e, None, Some(on))
    }

    /// A preset deleted: the user's file; a factory preset (or the one the user's version
    /// stood in for) hidden until [`Library::restore`].
    pub fn remove(&self, name: &str) -> Result<(), String> {
        let e = self.find(name).ok_or_else(|| format!("No preset {name}"))?;
        if let Some(f) = &e.file {
            std::fs::remove_file(f).map_err(|err| err.to_string())?;
        }
        let hide = match e.origin {
            Origin::Factory => Some(e.sound.name.clone()),
            Origin::Edited => e.replaces.clone(),
            Origin::User => None,
        };
        if let Some(h) = hide {
            let mut overlay = self.read_overlay();
            overlay.entry(h).or_default().hidden = true;
            self.write_overlay(&overlay)?;
        }
        Ok(())
    }

    /// The user's version of a factory preset taken back: the factory's shown again (its
    /// name).
    pub fn revert(&self, name: &str) -> Result<String, String> {
        let e = self.find(name).ok_or_else(|| format!("No preset {name}"))?;
        let (Origin::Edited, Some(file), Some(factory)) = (e.origin, &e.file, &e.replaces) else {
            return Err(format!("{name} is not a factory preset edited"));
        };
        std::fs::remove_file(file).map_err(|err| err.to_string())?;
        let mut overlay = self.read_overlay();
        if let Some(s) = overlay.get_mut(factory) {
            s.hidden = false;
            self.write_overlay(&overlay)?;
        }
        Ok(factory.clone())
    }

    /// The factory presets deleted shown again: their names.
    pub fn restore(&self) -> Result<Vec<String>, String> {
        let mut overlay = self.read_overlay();
        let mut restored = Vec::new();
        for (name, s) in &mut overlay {
            if s.hidden {
                s.hidden = false;
                restored.push(name.clone());
            }
        }
        self.write_overlay(&overlay)?;
        Ok(restored)
    }
}

/// A table written whole, then moved into place.
fn write(file: &Path, t: &toml::Table) -> Result<(), String> {
    let text = toml::to_string(t).map_err(|e| e.to_string())?;
    if let Some(d) = file.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let tmp = file.with_extension("toml.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, file).map_err(|e| format!("{}: {e}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[Entry]) -> Vec<&str> {
        list.iter().map(|e| e.sound.name.as_str()).collect()
    }

    #[test]
    fn the_factory_file_is_read_whole() {
        let f = factory();
        assert_eq!(f.len(), 17);
        assert_eq!(f[0].name, "Bass");
        assert!(f.iter().all(|s| !s.values.is_empty()));
        assert!(
            f.iter()
                .all(|s| s.values.iter().all(|(k, _)| !KEPT.contains(&k.as_str())))
        );
    }

    #[test]
    fn the_users_presets_are_saved_found_tagged_renamed_and_removed() {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        let values = vec![("cutoff".to_owned(), -2.5), ("osc1_range".to_owned(), 3.0)];
        let e = lib
            .save(
                "Deep Bass",
                values,
                Some(parse_tags("bass, midnight, Bass").unwrap()),
            )
            .unwrap();
        assert_eq!(e.origin, Origin::User);
        assert_eq!(e.sound.tags, ["bass", "midnight"]);
        // The file as DAW-v3 reads it.
        let t: toml::Table = std::fs::read_to_string(dir.path().join("Deep Bass.toml"))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(t["format"].as_integer(), Some(1));
        assert_eq!(t["values"]["cutoff"].as_float(), Some(-2.5));
        assert_eq!(t["values"]["osc1_range"].as_integer(), Some(3));
        assert_eq!(
            names(&lib.search("ROUND deep", &[], false, false)),
            Vec::<&str>::new()
        );
        assert_eq!(names(&lib.search("deep", &[], false, false)), ["Deep Bass"]);
        assert_eq!(
            names(&lib.search("", &["midnight".into()], false, false)),
            ["Deep Bass"]
        );
        assert_eq!(names(&lib.search("", &[], false, true)), ["Deep Bass"]);
        lib.favorite("deep bass", true).unwrap();
        lib.favorite("Lead", true).unwrap();
        assert_eq!(
            names(&lib.search("", &[], true, false)),
            ["Deep Bass", "Lead"]
        );
        lib.tag("Bass", vec!["bass".into(), "mine".into()]).unwrap();
        assert_eq!(
            names(&lib.search("", &["mine".into()], false, false)),
            ["Bass"]
        );
        assert!(lib.tags().contains(&"midnight".to_owned()));
        lib.rename("Deep Bass", "Deeper Bass").unwrap();
        assert!(!dir.path().join("Deep Bass.toml").exists());
        assert!(lib.find("Deeper Bass").unwrap().favorite);
        assert!(lib.rename("Deeper Bass", "lead").is_err());
        lib.remove("Deeper Bass").unwrap();
        assert!(lib.search("", &[], false, true).is_empty());
        assert!(check_name("a/b").is_err() && check_name(" x").is_err() && check_name("").is_err());
    }

    #[test]
    fn a_factory_preset_edited_renamed_or_removed_comes_back() {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::at(dir.path());
        let edited = lib
            .save("lead", vec![("emphasis".into(), 9.0)], None)
            .unwrap();
        assert_eq!(edited.origin, Origin::Edited);
        assert_eq!(edited.replaces.as_deref(), Some("Lead"));
        assert_eq!(edited.sound.tags, ["lead", "glide"]);
        assert_eq!(
            lib.entries()
                .0
                .iter()
                .filter(|e| same(&e.sound.name, "lead"))
                .count(),
            1
        );
        assert_eq!(lib.revert("lead").unwrap(), "Lead");
        assert_eq!(lib.find("Lead").unwrap().origin, Origin::Factory);
        lib.rename("Bass", "Round Bass").unwrap();
        assert!(lib.find("Bass").is_none());
        assert_eq!(lib.find("Round Bass").unwrap().origin, Origin::Edited);
        lib.revert("Round Bass").unwrap();
        assert!(lib.find("Bass").is_some());
        lib.remove("Bass").unwrap();
        lib.save("Lead", vec![], None).unwrap();
        lib.remove("Lead").unwrap();
        assert!(lib.find("Bass").is_none() && lib.find("Lead").is_none());
        assert!(!dir.path().join("Lead.toml").exists());
        assert_eq!(lib.restore().unwrap(), ["Bass", "Lead"]);
        assert_eq!(lib.entries().0.len(), factory().len());
        assert!(lib.revert("Bass").is_err());
    }

    /// DAW-v3's files (written by its own TOML writer) read here, and an unreadable one
    /// reported, not fatal.
    #[test]
    fn a_preset_written_by_the_daw_is_read() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Wide Pad.toml"),
            "format = 1\nname = \"Wide Pad\"\ntags = [\"pad\"]\nfavorite = false\n\n[values]\ncutoff = 1.5\nloudness_attack = 6\npoly = 1\nvoices = 6\nanalog = 40\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("Broken.toml"), "name = ").unwrap();
        let lib = Library::at(dir.path());
        let (list, bad) = lib.entries();
        let pad = list.iter().find(|e| e.sound.name == "Wide Pad").unwrap();
        assert_eq!(pad.origin, Origin::User);
        assert!(pad.sound.values.contains(&("voices".to_owned(), 6.0)));
        assert_eq!(bad.len(), 1);
    }
}

/// Each program reads the other's files: with `CA72_CROSS_DIR` set (a library folder), writes
/// a preset there as the plug-in does and lists what is there (DAW-v3's included).
#[cfg(test)]
mod cross {
    use super::*;

    #[test]
    #[ignore = "a check against DAW-v3's files, run by hand"]
    fn cross_check() {
        let Some(dir) = std::env::var_os("CA72_CROSS_DIR") else {
            return;
        };
        let lib = Library::at(dir);
        lib.save(
            "From The Plug-in",
            vec![
                ("cutoff".into(), -1.25),
                ("osc1_range".into(), 1.0),
                ("voices".into(), 6.0),
            ],
            Some(vec!["cross".into()]),
        )
        .unwrap();
        lib.favorite("Pulse Strut", true).unwrap();
        for e in lib
            .entries()
            .0
            .iter()
            .filter(|e| e.origin != Origin::Factory || e.favorite)
        {
            eprintln!(
                "CROSS {} {:?} fav={} tags={:?} values={}",
                e.sound.name,
                e.origin,
                e.favorite,
                e.sound.tags,
                e.sound.values.len()
            );
        }
    }
}
