//! The effects on this computer (docs/fx-host.md): the plug-in files in the searched folders,
//! each listed by a helper process of its own, the listings kept in a cache file by each
//! file's size and modification time, so that a plug-in is loaded to be listed only when it
//! is new or has changed.

use crate::locate;
use daw_ipc::protocol::PluginDesc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

/// What a plug-in says it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// An audio effect (CLAP `audio-effect`, a VST3 `Fx` category).
    Effect,
    /// An instrument: not hosted (the owner, 2026-10-18).
    Instrument,
    /// It does not say; loading it finds out (an effect has an audio input).
    Unknown,
}

/// What a plug-in's categories make it.
pub fn kind(desc: &PluginDesc) -> Kind {
    let has = |name: &str| desc.features.iter().any(|f| f.eq_ignore_ascii_case(name));
    match desc.format.as_str() {
        "clap" if has("instrument") => Kind::Instrument,
        "clap" if has("audio-effect") => Kind::Effect,
        "vst3" if has("Instrument") || (has("Synth") && !has("Fx")) => Kind::Instrument,
        "vst3" if has("Fx") => Kind::Effect,
        _ => Kind::Unknown,
    }
}

/// A plug-in found, as its file lists it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Found {
    pub path: PathBuf,
    pub desc: PluginDesc,
    pub kind: Kind,
}

/// A plug-in file that could not be listed (it crashed, hung or is not a plug-in).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unlisted {
    pub path: PathBuf,
    pub reason: String,
}

/// Everything found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    pub plugins: Vec<Found>,
    pub unlisted: Vec<Unlisted>,
    /// Files listed afresh (not from the cache).
    pub scanned: usize,
}

impl Catalog {
    /// The effects (and the plug-ins that do not say what they are), in order found.
    pub fn effects(&self) -> impl Iterator<Item = &Found> {
        self.plugins.iter().filter(|f| f.kind != Kind::Instrument)
    }
}

const CACHE_FORMAT: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache {
    format: u32,
    files: Vec<CachedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedFile {
    path: PathBuf,
    /// The file's size and modification time (ns since the epoch); for a bundle its
    /// binary's.
    len: u64,
    modified: u128,
    #[serde(default)]
    plugins: Vec<PluginDesc>,
    #[serde(default)]
    error: Option<String>,
}

/// The size and modification time that stand for a plug-in file: a bundle's newest file
/// inside (its binary, its resources), two levels deep, else the file's own.
fn stamp(path: &Path) -> (u64, u128) {
    fn of(m: &std::fs::Metadata) -> u128 {
        m.modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_nanos())
    }
    fn walk(dir: &Path, depth: u32, best: &mut (u64, u128)) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let Ok(m) = e.metadata() else { continue };
            if m.is_dir() {
                if depth > 0 {
                    walk(&e.path(), depth - 1, best);
                }
            } else if of(&m) >= best.1 {
                *best = (m.len(), of(&m));
            }
        }
    }
    let Ok(m) = std::fs::metadata(path) else {
        return (0, 0);
    };
    if m.is_dir() {
        let mut best = (0, 0);
        walk(path, 3, &mut best);
        best
    } else {
        (m.len(), of(&m))
    }
}

/// Lists the plug-ins in `dirs` (folders, or plug-in files themselves) with `helper`, each
/// file given `timeout`. With a `cache` file, files unchanged since they were listed are
/// not loaded again, and the cache is written back with what was found.
pub fn scan(helper: &Path, dirs: &[PathBuf], cache: Option<&Path>, timeout: Duration) -> Catalog {
    let old: Cache = cache
        .and_then(|c| std::fs::read(c).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .filter(|c: &Cache| c.format == CACHE_FORMAT)
        .unwrap_or_default();
    let mut new = Cache {
        format: CACHE_FORMAT,
        files: Vec::new(),
    };
    let mut catalog = Catalog::default();
    for path in locate::plugin_files(dirs) {
        let (len, modified) = stamp(&path);
        let entry = match old
            .files
            .iter()
            .find(|f| f.path == path && f.len == len && f.modified == modified)
        {
            Some(f) => f.clone(),
            None => {
                catalog.scanned += 1;
                let (plugins, error) = match locate::scan(helper, &path, timeout) {
                    Ok(p) => (p, None),
                    Err(e) => (Vec::new(), Some(e)),
                };
                CachedFile {
                    path: path.clone(),
                    len,
                    modified,
                    plugins,
                    error,
                }
            }
        };
        if let Some(e) = &entry.error {
            catalog.unlisted.push(Unlisted {
                path: path.clone(),
                reason: e.clone(),
            });
        }
        for desc in &entry.plugins {
            catalog.plugins.push(Found {
                path: path.clone(),
                kind: kind(desc),
                desc: desc.clone(),
            });
        }
        new.files.push(entry);
    }
    if let Some(c) = cache
        && let Ok(bytes) = serde_json::to_vec_pretty(&new)
    {
        let _ = crate::write_atomically(c, &bytes);
    }
    catalog
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc(format: &str, features: &[&str]) -> PluginDesc {
        PluginDesc {
            format: format.into(),
            features: features.iter().map(|s| (*s).to_owned()).collect(),
            ..PluginDesc::default()
        }
    }

    #[test]
    fn categories_tell_effects_from_instruments() {
        assert_eq!(
            kind(&desc("clap", &["audio-effect", "reverb"])),
            Kind::Effect
        );
        assert_eq!(
            kind(&desc("clap", &["instrument", "synthesizer"])),
            Kind::Instrument
        );
        assert_eq!(kind(&desc("clap", &["note-effect"])), Kind::Unknown);
        assert_eq!(kind(&desc("vst3", &["Fx", "Reverb"])), Kind::Effect);
        assert_eq!(
            kind(&desc("vst3", &["Instrument", "Synth"])),
            Kind::Instrument
        );
        assert_eq!(kind(&desc("vst3", &["Fx", "Synth"])), Kind::Effect);
        assert_eq!(kind(&desc("vst3", &[])), Kind::Unknown);
    }
}
