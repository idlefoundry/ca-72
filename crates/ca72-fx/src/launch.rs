//! Whether the application closed properly the last time it ran with this session folder
//! (decisions.md R-FX-SAFETY): a marker written as it starts and taken away as it closes. One
//! left behind means the last run ended without closing (it crashed, was killed, or the
//! computer went down): if the chain had effects, this start is a safe one, none of them
//! loaded until the player turns them back on. It says nothing about the effects themselves,
//! which are not turned off for it: the next start that follows a proper close loads them.
//!
//! A session folder in use by another instance (its `instance.lock` locked: the standalone
//! runtime's own lock) is left alone: that instance's marker is its own, and this one will
//! not run.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// The marker's name in the session folder.
pub const MARKER: &str = "running.marker";
/// The standalone runtime's lock in the session folder (plugin-kit-standalone's).
pub const LOCK: &str = "instance.lock";

/// How the last run ended, as this one starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Last {
    /// It closed properly, or there was none.
    Closed,
    /// It did not close (the marker was there).
    DidNotClose,
    /// Another instance runs with this session folder now.
    Running,
}

/// Why this start is a safe one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeStart {
    /// The last run did not close, and the chain has effects to load.
    DidNotClose,
    /// The player asked (Shift held as it opened; Option too on macOS; `--safe-start`).
    Asked,
}

/// Whether this start is a safe one (no effect loaded until the player turns them back on):
/// asked for, or after a run that did not close when there are effects to load.
pub fn safe_start(last: Last, has_effects: bool, asked: bool) -> Option<SafeStart> {
    if asked {
        Some(SafeStart::Asked)
    } else if last == Last::DidNotClose && has_effects {
        Some(SafeStart::DidNotClose)
    } else {
        None
    }
}

/// This run's marker: taken away by [`Launch::end`] when it closes properly.
#[derive(Debug)]
pub struct Launch {
    marker: Option<PathBuf>,
}

impl Launch {
    /// The run beginning in session folder `dir`: how the last one ended, and this one's
    /// marker written (none if another instance runs there).
    pub fn begin(dir: &Path) -> (Launch, Last) {
        if locked_by_another(dir) {
            return (Launch { marker: None }, Last::Running);
        }
        let marker = dir.join(MARKER);
        let last = if marker.exists() {
            Last::DidNotClose
        } else {
            Last::Closed
        };
        let text = format!(
            "{}\n{}\n",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        );
        if let Err(e) = crate::write_atomically(&marker, text.as_bytes()) {
            eprintln!("CA-72 effects: {}: {e}", marker.display());
        }
        (
            Launch {
                marker: Some(marker),
            },
            last,
        )
    }

    /// The run closed properly: its marker taken away.
    pub fn end(self) {
        if let Some(m) = &self.marker
            && let Err(e) = std::fs::remove_file(m)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("CA-72 effects: {}: {e}", m.display());
        }
    }
}

/// Whether another process holds the session folder's run lock now (a lock that cannot be
/// taken; a folder or lock that cannot be opened is taken for free).
fn locked_by_another(dir: &Path) -> bool {
    let Ok(f) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(LOCK))
    else {
        return false;
    };
    match f.try_lock() {
        Ok(()) => {
            let _ = f.unlock();
            false
        }
        Err(std::fs::TryLockError::WouldBlock) => true,
        Err(std::fs::TryLockError::Error(_)) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run that closes takes its marker away; one that does not leaves it, and the next
    /// start finds it; a folder another instance holds is left alone.
    #[test]
    fn a_run_that_did_not_close_is_found_at_the_next_start() {
        let dir = tempfile::tempdir().unwrap();
        let (run, last) = Launch::begin(dir.path());
        assert_eq!(last, Last::Closed, "a first run");
        assert!(dir.path().join(MARKER).exists());
        run.end();
        assert!(!dir.path().join(MARKER).exists());
        let (_crashed, last) = Launch::begin(dir.path());
        assert_eq!(last, Last::Closed);
        // (`_crashed` is not ended: the run "crashed".)
        let (run, last) = Launch::begin(dir.path());
        assert_eq!(last, Last::DidNotClose);
        run.end();
        assert_eq!(Launch::begin(dir.path()).1, Last::Closed);
        assert_eq!(
            safe_start(Last::DidNotClose, true, false),
            Some(SafeStart::DidNotClose)
        );
        assert_eq!(
            safe_start(Last::DidNotClose, false, false),
            None,
            "nothing to load"
        );
        assert_eq!(safe_start(Last::Closed, true, false), None);
        assert_eq!(
            safe_start(Last::Closed, false, true),
            Some(SafeStart::Asked)
        );

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.path().join(LOCK))
            .unwrap();
        lock.lock().unwrap();
        // (A lock of another open file, as another process's would be.)
        let (other, last) = Launch::begin(dir.path());
        assert_eq!(last, Last::Running);
        other.end();
        assert!(
            dir.path().join(MARKER).exists(),
            "the running instance's marker is left as it is"
        );
    }
}
