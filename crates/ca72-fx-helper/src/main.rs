//! The standalone application's effects' helper (docs/fx-host.md): DAW-v3's plug-in helper
//! (third_party/daw-v3), one process per effect, started and watched by `ca72-fx`. Plug-in
//! code runs only here: a crash or a hang ends this process, never the synth.
//!
//! `ca72-fx-helper --scan <file>` prints the plug-ins in a file as JSON and exits.

fn main() {
    std::process::exit(daw_helper::run());
}
