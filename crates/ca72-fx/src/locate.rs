//! Finding the helper and the effects (docs/fx-host.md). Plug-in files are only ever opened
//! by a helper (`ca72-fx-helper --scan`), never by the application. After DAW-v3's
//! `daw-sandbox/src/locate.rs` (commit cb15558), with the standalone's own variables.

use daw_ipc::protocol::PluginDesc;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The helper's file name.
pub const HELPER: &str = if cfg!(windows) {
    "ca72-fx-helper.exe"
} else {
    "ca72-fx-helper"
};

/// The helper: `$CA72_FX_HELPER`, or [`helper_for`] the running program.
pub fn helper_path() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("CA72_FX_HELPER") {
        return Ok(PathBuf::from(p));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    helper_for(&exe).ok_or_else(|| {
        format!(
            "the effects' helper `{HELPER}` was not found beside {}; set CA72_FX_HELPER to its path",
            exe.display()
        )
    })
}

/// The helper of the program `exe`: `ca72-fx-helper` beside it (Linux, Windows, a development
/// build), else, in a macOS application (`exe` in `X.app/Contents/MacOS`), the helper's own
/// application bundle in `X.app/Contents/Helpers` (`<name> Effects Host.app`, whose
/// `Contents/MacOS` holds it), the first by name if there were several. The helper is never
/// looked for loose in `Contents/Helpers`: there it took the synth's bundle identifier and
/// showed in the Dock as a second application (decisions.md R-FX-DOCK).
pub fn helper_for(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    let beside = dir.join(HELPER);
    if beside.is_file() {
        return Some(beside);
    }
    let helpers = dir.parent()?.join("Helpers");
    let mut bundles: Vec<PathBuf> = std::fs::read_dir(&helpers)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "app"))
        .collect();
    bundles.sort();
    bundles
        .into_iter()
        .map(|b| b.join("Contents/MacOS").join(HELPER))
        .find(|p| p.is_file())
}

/// Whether the system's standard plug-in folders are searched: yes, unless
/// `CA72_FX_STANDARD_FOLDERS` is `0`. The tests turn them off, so that they never load the
/// plug-ins installed on the machine that runs them (DAW-v3 D79).
pub fn standard_folders() -> bool {
    std::env::var_os("CA72_FX_STANDARD_FOLDERS").is_none_or(|v| v != "0")
}

/// The folders searched for effects, in order: `$CA72_FX_PATH` (the platform's path list),
/// then, unless [`standard_folders`] says not, the platform's standard CLAP and VST3
/// folders. `CLAP_PATH` and `VST3_PATH` are not read: they belong to the player's hosts.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("CA72_FX_PATH")
        .map(|v| std::env::split_paths(&v).collect())
        .unwrap_or_default();
    if standard_folders() {
        dirs.extend(standard_dirs());
    }
    dirs
}

/// The platform's standard folders for CLAP and VST3 plug-ins (the formats' own
/// documentation), the player's before the system's.
pub fn standard_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut dirs = Vec::new();
    if cfg!(target_os = "macos") {
        for format in ["CLAP", "VST3"] {
            if let Some(h) = &home {
                dirs.push(h.join("Library/Audio/Plug-Ins").join(format));
            }
            dirs.push(PathBuf::from("/Library/Audio/Plug-Ins").join(format));
        }
    } else if cfg!(windows) {
        for format in ["CLAP", "VST3"] {
            if let Some(l) = std::env::var_os("LOCALAPPDATA") {
                dirs.push(PathBuf::from(l).join("Programs/Common").join(format));
            }
            if let Some(c) = std::env::var_os("COMMONPROGRAMFILES") {
                dirs.push(PathBuf::from(c).join(format));
            }
        }
    } else {
        if let Some(h) = &home {
            dirs.push(h.join(".clap"));
            dirs.push(h.join(".vst3"));
        }
        for d in [
            "/usr/lib/clap",
            "/usr/local/lib/clap",
            "/usr/lib/vst3",
            "/usr/local/lib/vst3",
        ] {
            dirs.push(PathBuf::from(d));
        }
    }
    dirs
}

/// Whether `p` names a plug-in file or bundle: `.clap` or `.vst3`.
pub fn is_plugin_file(p: &Path) -> bool {
    p.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("clap") || e.eq_ignore_ascii_case("vst3"))
}

/// The plug-in files and bundles in `dirs`, folders looked into three levels deep, a bundle
/// never; sorted within each folder, each once.
pub fn plugin_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        entries.sort();
        for p in entries {
            if is_plugin_file(&p) {
                if !out.contains(&p) {
                    out.push(p);
                }
            } else if p.is_dir() && depth > 0 {
                walk(&p, depth - 1, out);
            }
        }
    }
    let mut out = Vec::new();
    for d in dirs {
        if is_plugin_file(d) {
            if !out.contains(d) {
                out.push(d.clone());
            }
        } else {
            walk(d, 3, &mut out);
        }
    }
    out
}

/// On Windows, starts a helper without a console of its own (the application has none, and
/// a window would flash). Plug-in windows are unaffected.
pub fn no_console(command: &mut Command) {
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(command, 0x0800_0000);
    #[cfg(not(windows))]
    let _ = command;
}

/// The plug-ins in a file, listed by a helper process of its own, killed after `timeout`: a
/// plug-in that crashes or hangs when loaded costs its own entry, nothing more.
pub fn scan(helper: &Path, file: &Path, timeout: Duration) -> Result<Vec<PluginDesc>, String> {
    let mut command = Command::new(helper);
    command
        .arg("--scan")
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    no_console(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", helper.display()))?;
    // Read as it comes, so that a plug-in that writes much cannot fill the pipe and stall.
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let out = std::thread::spawn(move || {
        let mut v = Vec::new();
        if let Some(s) = stdout.as_mut() {
            let _ = std::io::Read::read_to_end(s, &mut v);
        }
        v
    });
    let err = std::thread::spawn(move || {
        let mut v = Vec::new();
        if let Some(s) = stderr.as_mut() {
            let _ = std::io::Read::read_to_end(s, &mut v);
        }
        v
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("listing its plug-ins took longer than {timeout:?}"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => return Err(e.to_string()),
        }
    };
    let out = out.join().unwrap_or_default();
    let err = err.join().unwrap_or_default();
    if !status.success() {
        let last = String::from_utf8_lossy(&err);
        let last = last.lines().last().unwrap_or("").trim().to_owned();
        return Err(if last.is_empty() {
            format!("its helper {}", crate::helper::describe(status))
        } else {
            last
        });
    }
    // The listing is the last line: a plug-in may print before it.
    let text = String::from_utf8_lossy(&out);
    let line = text
        .lines()
        .rev()
        .find(|l| l.starts_with('['))
        .unwrap_or("");
    serde_json::from_str(line).map_err(|e| format!("its listing could not be read: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(p: &Path) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"").unwrap();
    }

    /// The helper beside the program (Linux, Windows, a development build), or in its own
    /// bundle inside a macOS application's `Contents/Helpers`, never loose there.
    #[test]
    fn the_helper_is_found_beside_the_program_or_in_its_own_bundle() {
        let root = tempfile::tempdir().unwrap();

        let bin = root.path().join("bin");
        file(&bin.join("ca72-standalone"));
        assert_eq!(helper_for(&bin.join("ca72-standalone")), None);
        file(&bin.join(HELPER));
        assert_eq!(
            helper_for(&bin.join("ca72-standalone")),
            Some(bin.join(HELPER))
        );

        let app = root.path().join("CA-72.app/Contents");
        let exe = app.join("MacOS/ca72-standalone");
        file(&exe);
        file(&app.join("Helpers").join(HELPER));
        assert_eq!(
            helper_for(&exe),
            None,
            "loose in Contents/Helpers: not used"
        );
        file(&app.join("Helpers/Notes.txt"));
        let host = app.join("Helpers/CA-72 Effects Host.app/Contents/MacOS");
        std::fs::create_dir_all(&host).unwrap();
        assert_eq!(helper_for(&exe), None, "a bundle without the helper");
        file(&host.join(HELPER));
        assert_eq!(helper_for(&exe), Some(host.join(HELPER)));
        // A renamed build's (CA-72 TEST's) bundle, beside nothing else, is found as well.
        let renamed = app.join("Helpers/CA-72 TEST Effects Host.app/Contents/MacOS");
        file(&renamed.join(HELPER));
        std::fs::remove_dir_all(app.join("Helpers/CA-72 Effects Host.app")).unwrap();
        assert_eq!(helper_for(&exe), Some(renamed.join(HELPER)));
        // Beside the program first.
        file(&app.join("MacOS").join(HELPER));
        assert_eq!(helper_for(&exe), Some(app.join("MacOS").join(HELPER)));
    }
}
