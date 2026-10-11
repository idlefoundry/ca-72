//! What the plug-in keeps for the computer, not for the session (decisions.md R-ULTRA,
//! R-SKIN, R-SEMITONES): QUALITY's indicator shown or not, whether ULTRA's note has been read,
//! the look (the skin) the editor is drawn in and how the FREQUENCY knobs' dials read. In the
//! shared
//! folder the preset library keeps (`library::shared_dir`), `CA-72/settings.toml`: read as the
//! editor opens, written as one changes. Keys it does not know are kept; a file it cannot read
//! is never written over.

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

    /// Kept on this computer (a file that cannot be read is left alone).
    pub fn save(&self) -> Result<(), String> {
        self.write_to(&file())
    }

    pub fn write_to(&self, path: &Path) -> Result<(), String> {
        let mut t = table(path)?.unwrap_or_default();
        t.insert("format".into(), 1.into());
        t.insert("ultra_shutter".into(), self.shutter.into());
        t.insert("ultra_note_read".into(), self.ultra_note_read.into());
        let skin = match self.skin {
            Skin::Worn => "realistic",
            Skin::Drawn => "classic",
        };
        t.insert("skin".into(), skin.into());
        let frequency = match self.frequency {
            Law::Semitones => "semitones",
            Law::Hardware => "hardware",
        };
        t.insert("frequency_dials".into(), frequency.into());
        write(path, &t)
    }
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
        let s = Settings {
            shutter: false,
            ultra_note_read: true,
            skin: Skin::Drawn,
            frequency: Law::Hardware,
        };
        s.write_to(&path).expect("written");
        assert_eq!(Settings::read(&path), s);
        let text = std::fs::read_to_string(&path).expect("read") + "theirs = 3\n";
        std::fs::write(&path, &text).expect("written");
        Settings::default().write_to(&path).expect("written");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(after.contains("theirs = 3"), "{after}");
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
        assert!(Settings::default().write_to(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "not [ toml");
        assert_eq!(Settings::read(&path), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
