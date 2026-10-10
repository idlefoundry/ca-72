//! What the plug-in keeps for the computer, not for the session (decisions.md R-ULTRA): ULTRA's
//! shutter shown or the light only, and whether ULTRA's note has been read. In the shared
//! folder the preset library keeps (`library::shared_dir`), `CA-72/settings.toml`: read as the
//! editor opens, written as one changes. Keys it does not know are kept; a file it cannot read
//! is never written over.

use std::path::{Path, PathBuf};

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
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shutter: true,
            ultra_note_read: false,
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
        };
        s.write_to(&path).expect("written");
        assert_eq!(Settings::read(&path), s);
        let text = std::fs::read_to_string(&path).expect("read") + "theirs = 3\n";
        std::fs::write(&path, &text).expect("written");
        Settings::default().write_to(&path).expect("written");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(after.contains("theirs = 3"), "{after}");
        std::fs::write(&path, "not [ toml").expect("written");
        assert!(Settings::default().write_to(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "not [ toml");
        assert_eq!(Settings::read(&path), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
