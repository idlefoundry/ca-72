//! The standalone application's effects (docs/fx-host.md, decisions.md R-FX-HOST): the
//! chain's audio side, handed to the synth's processor when the application makes it. The
//! chain itself (`ca72_fx`) knows nothing of the synth, and the synth of the chain only this.

use std::sync::Mutex;

/// The share of a block's period from its start by which every effect must have returned
/// its block, after the voices' own (`engine::DEADLINE`, 0.75): an effect late past it
/// passes the sound on dry for that block.
pub const DEADLINE: f64 = 0.9;

static RUNNER: Mutex<Option<ca72_fx::Runner>> = Mutex::new(None);

/// The chain's audio side, for the next instance of the synth made: the standalone's (one
/// instance), before it starts.
pub fn install(runner: ca72_fx::Runner) {
    *RUNNER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(runner);
}

/// The audio side installed, if any (the plug-ins: none).
pub(crate) fn take_runner() -> Option<ca72_fx::Runner> {
    RUNNER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
