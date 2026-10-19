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

/// The helper: `$CA72_FX_HELPER`, or `ca72-fx-helper` beside the running program (on macOS
/// also in the application bundle's `Contents/Helpers`).
pub fn helper_path() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("CA72_FX_HELPER") {
        return Ok(PathBuf::from(p));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("the program has no directory")?;
    let mut candidates = vec![dir.join(HELPER)];
    if let Some(contents) = dir.parent() {
        candidates.push(contents.join("Helpers").join(HELPER));
    }
    candidates.into_iter().find(|p| p.is_file()).ok_or_else(|| {
        format!(
            "the effects' helper `{HELPER}` was not found beside {}; set CA72_FX_HELPER to its path",
            exe.display()
        )
    })
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
