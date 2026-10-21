//! The standalone application's effects (docs/fx-host.md, decisions.md R-FX-HOST): up to
//! four third-party CLAP or VST3 effects after the synth's output, each loaded and run in a
//! helper process of its own (`ca72-fx-helper`, DAW-v3's helper), so that an effect that
//! crashes or hangs costs its own sound, never the synth's: the chain passes the synth on
//! dry while the helper is started again and the effect restored.
//!
//! - [`catalog`]: the effects on this computer, listed by helpers, the listings cached;
//! - [`chain`]: the chain's control side ([`Chain`]) and audio side ([`Runner`]), saved
//!   and restored ([`SavedChain`]);
//! - [`sandbox`]: the helpers, their supervision, the effects' state snapshots;
//! - [`node`]: one effect on the audio thread, synchronous or pipelined;
//! - [`console`]: a plain line-command interface to the chain, for testing until its view
//!   is designed;
//! - [`locate`]: finding the helper and the plug-in files;
//! - [`launch`]: whether the application closed properly last time (a safe start if not).
//!
//! No effect can stop the synth starting or playing, nor keep it from starting again
//! (decisions.md R-FX-SAFETY): effects load while it plays, one that crashes or hangs as it
//! loads or keeps crashing is turned off and stays off until the player tries it again, and a
//! start after a run that did not close, or with Shift held, loads none of them.
//!
//! Nothing here loads plug-in code or links a plug-in SDK: that is the helper's. The
//! crate does not depend on the CA-72's synth, so that another instrument can take it.

pub mod catalog;
pub mod chain;
pub mod console;
pub mod helper;
pub mod launch;
pub mod locate;
pub mod node;
pub mod sandbox;

pub use catalog::{Catalog, Found, Kind};
pub use chain::{
    Chain, ChainOptions, Disabled, Health, MAX_SLOTS, Restore, Runner, SavedChain, SavedSlot,
    SlotStatus,
};
/// Where the music is as a block begins, given to the effects (the host's tempo and place).
pub use daw_ipc::Timing;
pub use sandbox::{Event, Fault, Sandbox, SandboxOptions};

use serde::{Deserialize, Serialize};
use std::path::Path;

/// How an effect exchanges blocks with its helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Each block waited for, within the block: no latency added (the default).
    Sync,
    /// A block behind: one block of latency added, the effect working while the synth
    /// plays the next.
    Pipelined,
}

impl std::str::FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Mode, String> {
        match s {
            "sync" => Ok(Mode::Sync),
            "pipelined" => Ok(Mode::Pipelined),
            _ => Err(format!("`{s}` is not a mode (sync or pipelined)")),
        }
    }
}

/// Writes `bytes` to `path` through a temporary file beside it, so that a crash or a full
/// disk leaves the old file whole.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    // One write at a time in this process: the temporary file's name is the process's.
    static WRITING: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _one = WRITING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
