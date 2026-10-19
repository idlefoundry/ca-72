//! What the plug-in keeps for the computer, not for the session (decisions.md R48,
//! R51, R54): QUALITY's indicator shown or not, whether ULTRA's note has been read,
//! the look (the skin) the editor is drawn in, its lettering's size, how the FREQUENCY
//! knobs' dials read, the standalone's TEMPO (R-ARP), and whether agents may operate the
//! standalone application (R-MCP). In the
//! shared
//! folder the preset library keeps (`library::shared_dir`), `CA-72/settings.toml`: read as the
//! editor opens, and as one changes only that key written, the file read again under a lock
//! every instance takes (`settings.toml.lock`), so that one editor's change never undoes
//! another's (the CA-74's review: each had written the whole of what it read as it opened).
//! Keys it does not know are kept; a file it cannot read is never written over.

use std::path::{Path, PathBuf};

use ca72::freq_dial::Law;
use ca72_panel::{Lettering, Skin};

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
    /// A skin pack's look chosen instead (decisions.md R-SKIN-PACKS), by its id: kept as
    /// `skin_pack`, with `skin` its base look (what an older CA-72, or this one without the
    /// pack, draws instead).
    pub pack: Option<PackId>,
    /// How large the editor's lettering is printed, chosen with the same key: as the panel
    /// has it unless LARGE is chosen (the owner, 2026-10-15: "this is just SOME people, so I
    /// want it as an option, not as a general change"; decisions.md R-LETTERING). Kept as
    /// `lettering`, "standard" or "large".
    pub lettering: Lettering,
    /// How oscillators 2 and 3's FREQUENCY knobs' dials read, chosen with the same key: a
    /// semitone a mark unless HARDWARE is chosen (the owner, 2026-10-10: "use our perfect ones
    /// by default, but allow the user to use the exact curves"). Kept as `frequency_dials`,
    /// "semitones" or "hardware".
    pub frequency: Law,
    /// Explicit lower-panel view preference. Missing uses Controls in a host, Keyboard in
    /// standalone; computer typing is deliberately transient and always starts disabled.
    pub keyboard: Option<bool>,
    /// First displayed MIDI note (F2 by default), independent of presets and host state.
    pub keyboard_root: u8,
    /// TYPE's C-based octave, independent of the visible range and sound controls.
    pub typing_root: u8,
    /// The standalone's TEMPO, beats a minute (120 unless set; decisions.md R-ARP): the
    /// arpeggiator's where no host gives one. Kept as `standalone_tempo`, 30 to 300.
    pub tempo: u16,
    /// Whether AI agents may operate the standalone application through MCP (`crate::agent`;
    /// decisions.md R-MCP): off unless the player turns it on (SETTINGS: AGENTS). Kept as
    /// `agents`, true or false.
    pub agents: bool,
}

/// A skin pack's id, as its manifest gives it (lower-case letters, digits, `-` and `.`, at
/// most 64 bytes), kept without allocating.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PackId {
    bytes: [u8; 64],
    len: u8,
}

impl PackId {
    pub fn new(id: &str) -> Option<PackId> {
        let ok = !id.is_empty()
            && id.len() <= 64
            && id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.');
        ok.then(|| {
            let mut bytes = [0; 64];
            bytes[..id.len()].copy_from_slice(id.as_bytes());
            PackId {
                bytes,
                len: id.len() as u8,
            }
        })
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }
}

impl std::fmt::Debug for PackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PackId({})", self.as_str())
    }
}

/// One setting changed, as an editor keeps it (`Settings::keep`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Shutter(bool),
    NoteRead,
    /// A built-in look (a pack's no longer chosen).
    Skin(Skin),
    /// A skin pack's look, and its base look.
    Pack(PackId, Skin),
    Lettering(Lettering),
    Frequency(Law),
    Keyboard(bool),
    KeyboardRoot(u8),
    TypingRoot(u8),
    Tempo(u16),
    Agents(bool),
}

impl Change {
    /// Its key and value in the file.
    fn put(self, t: &mut toml::Table) {
        let name = |skin| match skin {
            Skin::Worn => "realistic",
            Skin::Drawn => "classic",
        };
        let (key, value): (&str, toml::Value) = match self {
            Change::Shutter(shown) => ("ultra_shutter", shown.into()),
            Change::NoteRead => ("ultra_note_read", true.into()),
            Change::Skin(skin) => {
                t.remove("skin_pack");
                ("skin", name(skin).into())
            }
            Change::Pack(id, base) => {
                t.insert("skin_pack".into(), id.as_str().into());
                ("skin", name(base).into())
            }
            Change::Lettering(Lettering::Standard) => ("lettering", "standard".into()),
            Change::Lettering(Lettering::Large) => ("lettering", "large".into()),
            Change::Frequency(Law::Semitones) => ("frequency_dials", "semitones".into()),
            Change::Frequency(Law::Hardware) => ("frequency_dials", "hardware".into()),
            Change::Keyboard(shown) => (
                "keyboard_view",
                if shown { "keyboard" } else { "controls" }.into(),
            ),
            Change::KeyboardRoot(root) => ("keyboard_root", i64::from(root.min(84)).into()),
            Change::TypingRoot(root) => ("typing_root", i64::from(root.min(120) / 12 * 12).into()),
            Change::Tempo(bpm) => ("standalone_tempo", i64::from(tempo_within(bpm)).into()),
            Change::Agents(on) => ("agents", on.into()),
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
            pack: None,
            lettering: Lettering::Standard,
            frequency: Law::Semitones,
            keyboard: None,
            keyboard_root: 41,
            typing_root: ca72_panel::keyboard::TYPE_ROOT,
            tempo: crate::params::DEFAULT_TEMPO as u16,
            agents: false,
        }
    }
}

/// A TEMPO within its range (`params::TEMPO_RANGE`).
pub fn tempo_within(bpm: u16) -> u16 {
    let (lo, hi) = crate::params::TEMPO_RANGE;
    bpm.clamp(lo as u16, hi as u16)
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
            agents: flag("agents", d.agents),
            keyboard: match t.get("keyboard_view").and_then(toml::Value::as_str) {
                Some("keyboard") => Some(true),
                Some("controls") => Some(false),
                _ => None,
            },
            keyboard_root: t
                .get("keyboard_root")
                .and_then(toml::Value::as_integer)
                .filter(|n| (0..=84).contains(n))
                .map(|n| n as u8)
                .unwrap_or(d.keyboard_root),
            typing_root: t
                .get("typing_root")
                .and_then(toml::Value::as_integer)
                .filter(|n| (0..=120).contains(n) && n % 12 == 0)
                .map(|n| n as u8)
                .unwrap_or(d.typing_root),
            tempo: t
                .get("standalone_tempo")
                .and_then(toml::Value::as_integer)
                .and_then(|n| u16::try_from(n).ok())
                .filter(|n| tempo_within(*n) == *n)
                .unwrap_or(d.tempo),
            // (Anything but CLASSIC, the realistic look.)
            skin: match t.get("skin").and_then(toml::Value::as_str) {
                Some("classic") => Skin::Drawn,
                _ => d.skin,
            },
            pack: t
                .get("skin_pack")
                .and_then(toml::Value::as_str)
                .and_then(PackId::new),
            // (Anything but LARGE, the standard lettering.)
            lettering: match t.get("lettering").and_then(toml::Value::as_str) {
                Some("large") => Lettering::Large,
                _ => d.lettering,
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
            Change::Skin(skin) => {
                self.skin = skin;
                self.pack = None;
            }
            Change::Pack(id, base) => {
                self.skin = base;
                self.pack = Some(id);
            }
            Change::Lettering(lettering) => self.lettering = lettering,
            Change::Frequency(law) => self.frequency = law,
            Change::Keyboard(shown) => self.keyboard = Some(shown),
            Change::KeyboardRoot(root) => self.keyboard_root = root.min(84),
            Change::TypingRoot(root) => self.typing_root = root.min(120) / 12 * 12,
            Change::Tempo(bpm) => self.tempo = tempo_within(bpm),
            Change::Agents(on) => self.agents = on,
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

/// Claim this version for one visible editor across formats, hosts and processes. The
/// last version shown is a high-water mark: a downgrade never erases it. A failed read or
/// write shows nothing automatically; Settings can still open the bundled highlights.
pub(crate) fn claim_release_notes(
    path: &Path,
    installed: &str,
) -> Result<Option<crate::whats_new::Notes>, String> {
    let _lock = lock(path)?;
    let mut t = table(path)?.unwrap_or_default();
    let previous = t.get("last_version_shown").and_then(toml::Value::as_str);
    let Some(notes) = crate::whats_new::assemble(previous, installed) else {
        return Ok(None);
    };
    t.insert("format".into(), 1.into());
    t.insert("last_version_shown".into(), installed.into());
    write(path, &t)?;
    Ok(Some(notes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The standalone's TEMPO: 120 for a new computer, kept within 30 to 300, a value out of
    /// its range or not a number read as 120.
    #[test]
    fn the_tempo_is_kept_within_its_range() {
        let dir = tempfile::tempdir().expect("settings folder");
        let path = dir.path().join("settings.toml");
        assert_eq!(Settings::read(&path).tempo, 120);
        Settings::keep(&path, Change::Tempo(96)).expect("saved");
        assert_eq!(Settings::read(&path).tempo, 96);
        Settings::keep(&path, Change::Tempo(900)).expect("saved");
        assert_eq!(Settings::read(&path).tempo, 300);
        std::fs::write(&path, "standalone_tempo = 7\n").expect("settings");
        assert_eq!(Settings::read(&path).tempo, 120);
        std::fs::write(&path, "standalone_tempo = \"fast\"\n").expect("settings");
        assert_eq!(Settings::read(&path).tempo, 120);
    }

    #[test]
    fn release_notes_history_survives_restarts_settings_edits_and_downgrades() {
        let dir = tempfile::tempdir().expect("settings folder");
        let path = dir.path().join("CA-72/settings.toml");
        let first = claim_release_notes(&path, "0.2.3")
            .expect("claimed")
            .expect("first install");
        assert!(first.previous.is_none());
        assert_eq!(first.versions, ["0.2.3"]);
        assert!(
            claim_release_notes(&path, "0.2.3")
                .expect("restart")
                .is_none()
        );
        Settings::keep(&path, Change::Skin(Skin::Drawn)).expect("other editor's setting");
        let notes = claim_release_notes(&path, "0.2.7")
            .expect("claim")
            .expect("skipped update");
        assert_eq!(notes.previous.as_deref(), Some("0.2.3"));
        assert_eq!(notes.versions, ["0.2.4", "0.2.5", "0.2.6", "0.2.7"]);
        assert_eq!(Settings::read(&path).skin, Skin::Drawn);
        let saved = std::fs::read(&path).expect("saved settings");
        for version in ["0.2.7", "0.2.5", "0.2.7+other-build"] {
            assert!(
                claim_release_notes(&path, version)
                    .expect("same or older")
                    .is_none()
            );
            assert_eq!(std::fs::read(&path).expect("settings"), saved);
        }
        Settings::keep(&path, Change::Lettering(Lettering::Large)).expect("other setting");
        assert_eq!(
            table(&path).expect("table").expect("settings")["last_version_shown"].as_str(),
            Some("0.2.7")
        );
    }

    #[test]
    fn unsafe_release_history_falls_back_and_unreadable_settings_are_left_alone() {
        let dir = tempfile::tempdir().expect("settings folder");
        let path = dir.path().join("settings.toml");
        for history in ["'unknown'", "'0.2'", "75", "false"] {
            std::fs::write(
                &path,
                format!("last_version_shown = {history}\nother_app = 5\n"),
            )
            .expect("settings");
            let notes = claim_release_notes(&path, "0.2.7")
                .expect("claim")
                .expect("highlights");
            assert!(notes.previous.is_none());
            assert_eq!(notes.versions, ["0.2.7"]);
            assert_eq!(
                table(&path).expect("table").expect("settings")["other_app"].as_integer(),
                Some(5)
            );
        }
        std::fs::write(&path, "not [ toml").expect("bad file");
        assert!(claim_release_notes(&path, "0.2.7").is_err());
        assert_eq!(std::fs::read_to_string(&path).expect("file"), "not [ toml");
        let blocked = dir.path().join("is-a-directory.toml");
        std::fs::create_dir(&blocked).expect("unwritable settings path");
        assert!(claim_release_notes(&blocked, "0.2.7").is_err());
    }

    #[test]
    fn only_one_concurrent_editor_claims_each_release() {
        let dir = tempfile::tempdir().expect("settings folder");
        let path = dir.path().join("settings.toml");
        for version in ["0.2.6", "0.2.7"] {
            let barrier = std::sync::Barrier::new(12);
            let shown = std::thread::scope(|scope| {
                let claims: Vec<_> = (0..12)
                    .map(|_| {
                        scope.spawn(|| {
                            barrier.wait();
                            claim_release_notes(&path, version)
                                .expect("claim")
                                .is_some()
                        })
                    })
                    .collect();
                claims
                    .into_iter()
                    .map(|claim| claim.join().expect("editor"))
                    .filter(|shown| *shown)
                    .count()
            });
            assert_eq!(shown, 1);
        }
    }

    /// A separate host/format's claim, driven by the parent test with its own settings.
    #[test]
    #[ignore = "a child process of the concurrency test"]
    fn claim_release_notes_in_another_process() {
        let folder = std::env::var_os("CA72_NOTES_TEST_DIR").expect("test folder");
        let folder = Path::new(&folder);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !folder.join("start").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "parent never started the claims"
            );
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if claim_release_notes(&folder.join("settings.toml"), "0.2.7")
            .expect("claim")
            .is_some()
        {
            let id = std::env::var("CA72_NOTES_TEST_ID").expect("process id");
            std::fs::write(folder.join(format!("shown-{id}")), "shown").expect("evidence");
        }
    }

    #[test]
    fn independent_host_processes_share_one_atomic_claim() {
        let dir = tempfile::tempdir().expect("settings folder");
        let children: Vec<_> = (0..8)
            .map(|i| {
                std::process::Command::new(std::env::current_exe().expect("test executable"))
                    .args([
                        "--exact",
                        "settings::tests::claim_release_notes_in_another_process",
                        "--ignored",
                        "--nocapture",
                    ])
                    .env("CA72_NOTES_TEST_DIR", dir.path())
                    .env("CA72_NOTES_TEST_ID", i.to_string())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .expect("host process")
            })
            .collect();
        std::fs::write(dir.path().join("start"), "start").expect("start claims");
        for child in children {
            let output = child.wait_with_output().expect("host finished");
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert_eq!(
            std::fs::read_dir(dir.path())
                .expect("files")
                .filter_map(Result::ok)
                .filter(|file| file.file_name().to_string_lossy().starts_with("shown-"))
                .count(),
            1
        );
        assert!(
            claim_release_notes(&dir.path().join("settings.toml"), "0.2.7")
                .expect("later restart")
                .is_none()
        );
    }

    #[test]
    fn keyboard_preferences_roundtrip_without_rewriting_other_settings() {
        let dir = tempfile::tempdir().expect("settings folder");
        let path = dir.path().join("settings.toml");
        std::fs::write(&path, "skin = \"classic\"\nother_app = 5\n").expect("settings");
        Settings::keep(&path, Change::Keyboard(true)).expect("view saved");
        Settings::keep(&path, Change::KeyboardRoot(77)).expect("range saved");
        assert_eq!(
            Settings::read(&path).typing_root,
            48,
            "old range does not transpose TYPE"
        );
        Settings::keep(&path, Change::TypingRoot(60)).expect("typing octave saved");
        let read = Settings::read(&path);
        assert_eq!(read.keyboard, Some(true));
        assert_eq!(read.keyboard_root, 77);
        assert_eq!(read.typing_root, 60);
        assert_eq!(read.skin, Skin::Drawn);
        assert!(
            std::fs::read_to_string(&path)
                .expect("read")
                .contains("other_app = 5")
        );
        Settings::keep(&path, Change::Keyboard(false)).expect("controls saved");
        assert_eq!(Settings::read(&path).keyboard, Some(false));
        for invalid in [-1, 85, 255] {
            std::fs::write(
                &path,
                format!("keyboard_root = {invalid}\nkeyboard_view = \"unknown\"\n"),
            )
            .expect("invalid settings");
            assert_eq!(Settings::read(&path).keyboard_root, 41);
            assert_eq!(Settings::read(&path).keyboard, None);
        }
        for invalid in [-1, 41, 121, 255] {
            std::fs::write(&path, format!("typing_root = {invalid}\n"))
                .expect("invalid typing octave");
            assert_eq!(Settings::read(&path).typing_root, 48);
        }
    }

    /// A pack's look is kept with its base, which an older CA-72 reads; a built-in one chosen
    /// again forgets the pack (decisions.md R-SKIN-PACKS).
    #[test]
    fn a_packs_look_is_kept_with_its_base() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("settings.toml");
        let id = PackId::new("org.example.night-panel").expect("an id");
        Settings::keep(&path, Change::Pack(id, Skin::Drawn)).expect("kept");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("skin = \"classic\""), "{text}");
        assert!(
            text.contains("skin_pack = \"org.example.night-panel\""),
            "{text}"
        );
        let read = Settings::read(&path);
        assert_eq!((read.skin, read.pack), (Skin::Drawn, Some(id)));
        Settings::keep(&path, Change::Skin(Skin::Worn)).expect("kept");
        let read = Settings::read(&path);
        assert_eq!((read.skin, read.pack), (Skin::Worn, None));
        // An id a pack could not have is not read as one.
        std::fs::write(&path, "skin_pack = \"../../evil\"\n").expect("written");
        assert_eq!(Settings::read(&path).pack, None);
    }

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
            Change::Lettering(Lettering::Large),
            Change::Frequency(Law::Hardware),
            Change::Agents(true),
        ] {
            Settings::keep(&path, c).expect("kept");
        }
        let s = Settings {
            shutter: false,
            ultra_note_read: true,
            skin: Skin::Drawn,
            pack: None,
            lettering: Lettering::Large,
            frequency: Law::Hardware,
            keyboard: None,
            keyboard_root: 41,
            typing_root: ca72_panel::keyboard::TYPE_ROOT,
            tempo: 120,
            agents: true,
        };
        assert_eq!(Settings::read(&path), s);
        assert!(
            std::fs::read_to_string(&path)
                .expect("read")
                .contains("agents = true")
        );
        let text = std::fs::read_to_string(&path).expect("read") + "theirs = 3\n";
        std::fs::write(&path, &text).expect("written");
        Settings::keep(&path, Change::Skin(Skin::Worn)).expect("kept");
        Settings::keep(&path, Change::Lettering(Lettering::Standard)).expect("kept");
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
        // The lettering: standard unless the file says large.
        assert!(after.contains("lettering = \"standard\""), "{after}");
        std::fs::write(&path, "lettering = \"large\"\n").expect("written");
        assert_eq!(Settings::read(&path).lettering, Lettering::Large);
        std::fs::write(&path, "lettering = \"huge\"\n").expect("written");
        assert_eq!(Settings::read(&path).lettering, Lettering::Standard);
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
            Change::Lettering(Lettering::Large),
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
                pack: None,
                lettering: Lettering::Large,
                frequency: Law::Hardware,
                keyboard: None,
                keyboard_root: 41,
                typing_root: ca72_panel::keyboard::TYPE_ROOT,
                tempo: 120,
                agents: false,
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
