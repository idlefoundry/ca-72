//! What the plug-in keeps for the computer, not for the session (decisions.md R-ULTRA,
//! R-SKIN, R-SEMITONES): QUALITY's indicator shown or not, whether ULTRA's note has been read,
//! the look (the skin) the editor is drawn in and how the FREQUENCY knobs' dials read. In the
//! shared
//! folder the preset library keeps (`library::shared_dir`), `CA-72/settings.toml`: read as the
//! editor opens, and as one changes only that key written, the file read again under a lock
//! every instance takes (`settings.toml.lock`), so that one editor's change never undoes
//! another's (the CA-74's review: each had written the whole of what it read as it opened).
//! Keys it does not know are kept; a file it cannot read is never written over.

use std::path::{Path, PathBuf};

use ca72::freq_dial::Law;
use ca72_panel::Skin;

use crate::library::{shared_dir, write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// QUALITY's indicator shown (on by default): its shutter opens on what each setting shows;
    /// off, none of it, the panel blank above QUALITY (chosen in QUALITY's menu, its toggle
    /// right-clicked; it had been "always open", the lamp standing in its ring). Kept as
    /// `ultra_shutter`.
    pub shutter: bool,
    /// ULTRA's note (what it is for) has been read.
    pub ultra_note_read: bool,
    /// The look the editor is drawn in, chosen with the rail's settings key: the realistic
    /// one (`Skin::Worn`) unless CLASSIC (`Skin::Drawn`) is chosen (the owner, 2026-10-10: "all
    /// future versions default to the new skin"). Kept as `skin`, "realistic" or "classic".
    pub skin: Skin,
    /// How oscillators 2 and 3's FREQUENCY knobs' dials read, chosen with the same key: a
    /// semitone a mark unless HARDWARE is chosen (the owner, 2026-10-10: "use our perfect ones
    /// by default, but allow the user to use the exact curves"). Kept as `frequency_dials`,
    /// "semitones" or "hardware".
    pub frequency: Law,
}

/// One setting changed, as an editor keeps it (`Settings::keep`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Shutter(bool),
    NoteRead,
    Skin(Skin),
    Frequency(Law),
}

impl Change {
    /// Its key and value in the file.
    fn put(self, t: &mut toml::Table) {
        let (key, value): (&str, toml::Value) = match self {
            Change::Shutter(shown) => ("ultra_shutter", shown.into()),
            Change::NoteRead => ("ultra_note_read", true.into()),
            Change::Skin(Skin::Worn) => ("skin", "realistic".into()),
            Change::Skin(Skin::Drawn) => ("skin", "classic".into()),
            Change::Frequency(Law::Semitones) => ("frequency_dials", "semitones".into()),
            Change::Frequency(Law::Hardware) => ("frequency_dials", "hardware".into()),
        };
        t.insert(key.into(), value);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shutter: true,
            ultra_note_read: false,
            skin: Skin::Worn,
            frequency: Law::Semitones,
        }
    }
}

/// The settings' file on this computer.
pub fn file() -> PathBuf {
    shared_dir().join("CA-72").join("settings.toml")
}

impl Settings {
    /// This computer's settings (the defaults where the file is missing or says nothing).
    pub fn load() -> Settings {
        Settings::read(&file())
    }

    pub fn read(path: &Path) -> Settings {
        let d = Settings::default();
        let Some(t) = table(path).ok().flatten() else {
            return d;
        };
        let flag = |k: &str, or: bool| t.get(k).and_then(toml::Value::as_bool).unwrap_or(or);
        Settings {
            shutter: flag("ultra_shutter", d.shutter),
            ultra_note_read: flag("ultra_note_read", d.ultra_note_read),
            // (Anything but CLASSIC, the realistic look.)
            skin: match t.get("skin").and_then(toml::Value::as_str) {
                Some("classic") => Skin::Drawn,
                _ => d.skin,
            },
            // (Anything but HARDWARE, semitones.)
            frequency: match t.get("frequency_dials").and_then(toml::Value::as_str) {
                Some("hardware") => Law::Hardware,
                _ => d.frequency,
            },
        }
    }

    /// `change` made to these, in memory.
    pub fn apply(&mut self, change: Change) {
        match change {
            Change::Shutter(shown) => self.shutter = shown,
            Change::NoteRead => self.ultra_note_read = true,
            Change::Skin(skin) => self.skin = skin,
            Change::Frequency(law) => self.frequency = law,
        }
    }

    /// `change` kept in the file at `path`, and nothing else: under the lock every instance
    /// takes, the file read again and that one key changed (a file that cannot be read is left
    /// alone).
    pub fn keep(path: &Path, change: Change) -> Result<(), String> {
        let _lock = lock(path)?;
        let mut t = table(path)?.unwrap_or_default();
        t.insert("format".into(), 1.into());
        change.put(&mut t);
        write(path, &t)
    }
}

/// The lock every instance takes to change the file at `path` (`<file>.lock` beside it, held
/// until the handle is dropped).
fn lock(path: &Path) -> Result<std::fs::File, String> {
    let at = path.with_extension("toml.lock");
    if let Some(d) = at.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&at)
        .map_err(|e| format!("{}: {e}", at.display()))?;
    f.lock().map_err(|e| format!("{}: {e}", at.display()))?;
    Ok(f)
}

/// The file's table: none if there is no file, an error if there is one it cannot read.
fn table(path: &Path) -> Result<Option<toml::Table>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse::<toml::Table>()
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Written and read again; keys it does not know kept; a file it cannot read left alone.
    #[test]
    fn the_settings_are_kept_and_a_bad_file_left_alone() {
        let dir = std::env::temp_dir().join(format!("ca72-settings-{}", std::process::id()));
        let path = dir.join("CA-72").join("settings.toml");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(Settings::read(&path), Settings::default());
        for c in [
            Change::Shutter(false),
            Change::NoteRead,
            Change::Skin(Skin::Drawn),
            Change::Frequency(Law::Hardware),
        ] {
            Settings::keep(&path, c).expect("kept");
        }
        let s = Settings {
            shutter: false,
            ultra_note_read: true,
            skin: Skin::Drawn,
            frequency: Law::Hardware,
        };
        assert_eq!(Settings::read(&path), s);
        let text = std::fs::read_to_string(&path).expect("read") + "theirs = 3\n";
        std::fs::write(&path, &text).expect("written");
        Settings::keep(&path, Change::Skin(Skin::Worn)).expect("kept");
        Settings::keep(&path, Change::Frequency(Law::Semitones)).expect("kept");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(after.contains("theirs = 3"), "{after}");
        assert!(
            after.contains("ultra_shutter = false"),
            "only the key kept changed: {after}"
        );
        // The look: realistic unless the file says classic.
        assert!(after.contains("skin = \"realistic\""), "{after}");
        std::fs::write(&path, "skin = \"something new\"\n").expect("written");
        assert_eq!(Settings::read(&path).skin, Skin::Worn);
        std::fs::write(&path, "skin = \"classic\"\n").expect("written");
        assert_eq!(Settings::read(&path).skin, Skin::Drawn);
        // The FREQUENCY dials: semitones unless the file says hardware.
        assert!(after.contains("frequency_dials = \"semitones\""), "{after}");
        std::fs::write(&path, "frequency_dials = \"hardware\"\n").expect("written");
        assert_eq!(Settings::read(&path).frequency, Law::Hardware);
        std::fs::write(&path, "frequency_dials = \"other\"\n").expect("written");
        assert_eq!(Settings::read(&path).frequency, Law::Semitones);
        std::fs::write(&path, "not [ toml").expect("written");
        assert!(Settings::keep(&path, Change::NoteRead).is_err());
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "not [ toml");
        assert_eq!(Settings::read(&path), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Two editors, each with what it read as it opened: one's change does not undo the
    /// other's (the CA-74's review: an editor that had read the indicator shown wrote it shown
    /// again as it marked ULTRA's note read); and changes made at once, from threads as from
    /// processes, all land, the file never torn.
    #[test]
    fn one_editors_change_does_not_undo_anothers() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("CA-72").join("settings.toml");
        let (mut a, mut b) = (Settings::read(&path), Settings::read(&path));
        a.apply(Change::Shutter(false));
        Settings::keep(&path, Change::Shutter(false)).expect("kept");
        b.apply(Change::NoteRead);
        Settings::keep(&path, Change::NoteRead).expect("kept");
        let now = Settings::read(&path);
        assert!(!now.shutter && now.ultra_note_read, "{now:?}");
        assert!(b.shutter, "b's own copy as it read it");
        let changes = [
            Change::Shutter(true),
            Change::Skin(Skin::Drawn),
            Change::Frequency(Law::Hardware),
            Change::NoteRead,
        ];
        std::thread::scope(|s| {
            for c in changes {
                let path = &path;
                s.spawn(move || {
                    for _ in 0..25 {
                        Settings::keep(path, c).expect("kept");
                        assert!(table(path).is_ok(), "torn");
                    }
                });
            }
        });
        let all = Settings::read(&path);
        assert_eq!(
            all,
            Settings {
                shutter: true,
                ultra_note_read: true,
                skin: Skin::Drawn,
                frequency: Law::Hardware,
            }
        );
        let left: Vec<_> = std::fs::read_dir(path.parent().expect("its folder"))
            .expect("listed")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(left.is_empty(), "temporary files left: {left:?}");
    }
}
